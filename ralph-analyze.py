#!/usr/bin/env python3
"""Analyze a ralph loop iteration log (stream-json from claude CLI).

Reads a .jsonl file of stream-json events and produces a structured summary
for phase 2 analysis: what tools were called, what files were read/written,
how much context was used, and what the agent actually accomplished.

Usage:
    python3 ralph-analyze.py iter-*.jsonl              # Print summary
    python3 ralph-analyze.py iter-*.jsonl --json       # Machine-readable
    python3 ralph-analyze.py iter-*.jsonl --timeline   # Chronological trace
"""

import json
import sys
from collections import Counter
from pathlib import Path


def parse_events(path):
    """Parse stream-json log into structured events."""
    events = []
    for line in open(path):
        line = line.strip()
        if not line:
            continue
        try:
            events.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return events


def extract_turns(events):
    """Extract assistant/user turn pairs with tool calls."""
    turns = []
    for ev in events:
        t = ev.get("type", "")
        if t == "assistant":
            msg = ev.get("message", {})
            turn = {
                "role": "assistant",
                "text_blocks": [],
                "tool_calls": [],
                "thinking_blocks": [],
            }
            for c in msg.get("content", []):
                ct = c.get("type", "")
                if ct == "text":
                    turn["text_blocks"].append(c.get("text", ""))
                elif ct == "tool_use":
                    turn["tool_calls"].append({
                        "name": c.get("name", ""),
                        "id": c.get("id", ""),
                        "input": c.get("input", {}),
                    })
                elif ct == "thinking":
                    turn["thinking_blocks"].append(c.get("thinking", ""))
            turns.append(turn)
        elif t == "user":
            msg = ev.get("message", {})
            turn = {"role": "user", "tool_results": []}
            for c in msg.get("content", []):
                ct = c.get("type", "")
                if ct == "tool_result":
                    content = c.get("content", "")
                    if isinstance(content, list):
                        text = " ".join(
                            item.get("text", str(item))[:200]
                            for item in content
                        )
                    else:
                        text = str(content)[:500]
                    turn["tool_results"].append({
                        "tool_use_id": c.get("tool_use_id", ""),
                        "is_error": c.get("is_error", False),
                        "content_len": len(str(content)),
                        "preview": text[:200],
                    })
            turns.append(turn)
    return turns


def extract_result(events):
    """Extract the final result event."""
    for ev in events:
        if ev.get("type") == "result":
            return {
                "success": ev.get("subtype") == "success",
                "num_turns": ev.get("num_turns", 0),
                "duration_ms": ev.get("duration_ms", 0),
                "cost_usd": ev.get("total_cost_usd", 0),
                "usage": ev.get("usage", {}),
            }
    return None


def analyze_tool_usage(turns):
    """Analyze tool call patterns."""
    tool_counts = Counter()
    tool_details = []
    files_read = set()
    files_written = set()
    commands_run = []

    for turn in turns:
        if turn["role"] != "assistant":
            continue
        for tc in turn.get("tool_calls", []):
            name = tc["name"]
            inp = tc["input"]
            tool_counts[name] += 1

            detail = {"tool": name}

            if name == "Read":
                fp = inp.get("file_path", "")
                files_read.add(fp)
                detail["file"] = fp
            elif name == "Edit":
                fp = inp.get("file_path", "")
                files_written.add(fp)
                detail["file"] = fp
            elif name == "Write":
                fp = inp.get("file_path", "")
                files_written.add(fp)
                detail["file"] = fp
            elif name == "Bash":
                cmd = inp.get("command", "")[:120]
                commands_run.append(cmd)
                detail["command"] = cmd
            elif name == "Grep":
                detail["pattern"] = inp.get("pattern", "")[:60]
            elif name == "Glob":
                detail["pattern"] = inp.get("pattern", "")[:60]
            elif name == "Skill":
                detail["skill"] = inp.get("skill", "")
            elif name == "Task":
                detail["subagent"] = inp.get("subagent_type", "")
                detail["desc"] = inp.get("description", "")[:60]

            tool_details.append(detail)

    return {
        "tool_counts": dict(tool_counts),
        "total_tool_calls": sum(tool_counts.values()),
        "files_read": sorted(files_read),
        "files_written": sorted(files_written),
        "commands_run": commands_run,
        "tool_details": tool_details,
    }


def detect_waste(turns, tool_analysis):
    """Flag potential context waste patterns."""
    issues = []

    # Duplicate file reads
    read_files = []
    for d in tool_analysis["tool_details"]:
        if d["tool"] == "Read" and "file" in d:
            read_files.append(d["file"])
    dupes = [f for f in set(read_files) if read_files.count(f) > 1]
    if dupes:
        issues.append(f"Duplicate file reads: {', '.join(dupes)}")

    # Skill invocations (waste in headless mode)
    skills = [d for d in tool_analysis["tool_details"] if d["tool"] == "Skill"]
    if skills:
        names = [s.get("skill", "?") for s in skills]
        issues.append(f"Skill invocations (headless waste): {', '.join(names)}")

    # Multiple test runs
    test_cmds = [
        c for c in tool_analysis["commands_run"]
        if "cargo test" in c
    ]
    if len(test_cmds) > 2:
        issues.append(f"Excessive test runs: {len(test_cmds)} (budget: 2)")

    # Multiple build/clippy runs
    build_cmds = [
        c for c in tool_analysis["commands_run"]
        if "cargo build" in c or "cargo clippy" in c or "cargo check" in c
    ]
    if len(build_cmds) > 2:
        issues.append(f"Excessive build commands: {len(build_cmds)}")

    # Unfiltered cargo output
    unfiltered = [
        c for c in tool_analysis["commands_run"]
        if ("cargo test" in c or "cargo build" in c or "cargo clippy" in c)
        and "grep" not in c
        and "head" not in c
    ]
    if unfiltered:
        issues.append(
            f"Unfiltered cargo commands: {len(unfiltered)} "
            f"(e.g. {unfiltered[0][:80]})"
        )

    # Too many tool calls
    total = tool_analysis["total_tool_calls"]
    if total > 20:
        issues.append(f"Over budget: {total} tool calls (budget: 20)")

    return issues


def format_timeline(turns):
    """Format turns as a chronological timeline."""
    lines = []
    step = 0
    for turn in turns:
        if turn["role"] == "assistant":
            step += 1
            for tc in turn.get("tool_calls", []):
                name = tc["name"]
                inp = tc["input"]
                if name == "Read":
                    lines.append(f"  {step}. Read {inp.get('file_path', '?')}")
                elif name == "Edit":
                    lines.append(f"  {step}. Edit {inp.get('file_path', '?')}")
                elif name == "Write":
                    lines.append(f"  {step}. Write {inp.get('file_path', '?')}")
                elif name == "Bash":
                    cmd = inp.get("command", "?")[:80]
                    lines.append(f"  {step}. Bash: {cmd}")
                elif name == "Skill":
                    lines.append(f"  {step}. Skill: {inp.get('skill', '?')}")
                elif name == "Task":
                    lines.append(
                        f"  {step}. Task({inp.get('subagent_type', '?')}): "
                        f"{inp.get('description', '?')[:60]}"
                    )
                elif name == "Grep":
                    lines.append(f"  {step}. Grep: {inp.get('pattern', '?')[:40]}")
                elif name == "Glob":
                    lines.append(f"  {step}. Glob: {inp.get('pattern', '?')[:40]}")
                else:
                    lines.append(f"  {step}. {name}")
            for text in turn.get("text_blocks", []):
                # Show just first line of text output
                first = text.strip().split("\n")[0][:100]
                if first:
                    lines.append(f"  {step}. Output: {first}")
    return "\n".join(lines)


def format_summary(events, output_json=False, show_timeline=False):
    """Format the full analysis."""
    turns = extract_turns(events)
    result = extract_result(events)
    tool_analysis = analyze_tool_usage(turns)
    waste = detect_waste(turns, tool_analysis)

    if output_json:
        return json.dumps({
            "result": result,
            "tools": tool_analysis,
            "waste_flags": waste,
        }, indent=2)

    lines = []
    lines.append("=" * 60)
    lines.append("RALPH ITERATION ANALYSIS")
    lines.append("=" * 60)

    if result:
        status = "OK" if result["success"] else "FAILED"
        lines.append(
            f"Status: {status}  "
            f"Turns: {result['num_turns']}  "
            f"Cost: ${result['cost_usd']:.4f}  "
            f"Duration: {result['duration_ms']}ms"
        )
        usage = result.get("usage", {})
        lines.append(
            f"Tokens: in={usage.get('input_tokens', 0)} "
            f"out={usage.get('output_tokens', 0)} "
            f"cache_read={usage.get('cache_read_input_tokens', 0)} "
            f"cache_create={usage.get('cache_creation_input_tokens', 0)}"
        )

    lines.append("")
    lines.append(f"Tool calls: {tool_analysis['total_tool_calls']}")
    for tool, count in sorted(
        tool_analysis["tool_counts"].items(), key=lambda x: -x[1]
    ):
        lines.append(f"  {tool}: {count}")

    if tool_analysis["files_read"]:
        lines.append("")
        lines.append("Files read:")
        for f in tool_analysis["files_read"]:
            lines.append(f"  {f}")

    if tool_analysis["files_written"]:
        lines.append("")
        lines.append("Files written:")
        for f in tool_analysis["files_written"]:
            lines.append(f"  {f}")

    if tool_analysis["commands_run"]:
        lines.append("")
        lines.append("Commands:")
        for c in tool_analysis["commands_run"]:
            lines.append(f"  $ {c}")

    if waste:
        lines.append("")
        lines.append("WASTE FLAGS:")
        for w in waste:
            lines.append(f"  ! {w}")

    if show_timeline:
        lines.append("")
        lines.append("TIMELINE:")
        lines.append(format_timeline(turns))

    lines.append("")
    return "\n".join(lines)


def main():
    if len(sys.argv) < 2:
        print("Usage: ralph-analyze.py <logfile.jsonl> [--json] [--timeline]")
        sys.exit(1)

    path = sys.argv[1]
    output_json = "--json" in sys.argv
    show_timeline = "--timeline" in sys.argv

    if not Path(path).exists():
        print(f"Error: {path} not found", file=sys.stderr)
        sys.exit(1)

    events = parse_events(path)
    if not events:
        print(f"Error: no events in {path}", file=sys.stderr)
        sys.exit(1)

    print(format_summary(events, output_json, show_timeline))


if __name__ == "__main__":
    main()
