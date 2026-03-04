#!/usr/bin/env python3
"""Interactive error browser REPL for cargo/clippy output.

When there are many errors, this tool categorizes them and lets you
drill down into specific error classes to work through them incrementally.

Usage:
    cargo clippy 2>&1 | python3 cargo-browser.py
    python3 cargo-browser.py --repl

REPL Commands:
    scan <command>     Run a cargo command and parse its output
    summary            Show error class summary
    show <code> [n] [offset]  Show errors for code (default: 10, offset 0)
    examples <code>   Show file locations only
    next <code> [n]   Continue from where you left off
    clear              Clear saved error state
    help               Show available commands
    exit/quit         Exit REPL

The tool maintains the full error list in memory across REPL commands.
"""
import argparse
import json
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

STATE_FILE = ".claude/.cargo-browser-state.json"


def parse_diagnostics(lines):
    """Parse cargo output into structured diagnostics by error code."""
    error_groups = defaultdict(list)

    i = 0
    while i < len(lines):
        line = lines[i]

        if not line.strip() or re.match(r'^warning: \S+@\S+:', line):
            i += 1
            continue
        if re.match(r'^\s*(Compiling|Checking|Downloading|Finished)', line):
            i += 1
            continue

        diag_match = re.match(r'^(warning|error(?:\[E\d+\])?): (.+)', line)
        if not diag_match:
            i += 1
            continue

        severity = diag_match.group(1)
        message = diag_match.group(2).rstrip()
        location = None
        help_lines = []

        error_code = None
        code_match = re.search(r'\[E\d+\]', severity)
        if code_match:
            error_code = code_match.group(0)
        else:
            error_code = severity

        j = i + 1
        while j < len(lines):
            ahead = lines[j]
            if not ahead.strip():
                j += 1
                continue

            loc_match = re.match(r'^\s*--> (.+)', ahead)
            if loc_match:
                if location is None:
                    location = loc_match.group(1).strip()
                j += 1
                continue
            if re.match(r'^\s*\d+\s*\|', ahead):
                j += 1
                continue

            help_match = re.match(r'^\s*=?\s*(help|note): (.+)', ahead)
            if help_match:
                help_lines.append(help_match.group(2).rstrip())
                j += 1
                continue

            if re.match(r'^(warning|error)', ahead):
                break
            if re.match(r'^\s*\|?\s*[\^~]+', ahead):
                j += 1
                continue
            if re.match(r'^\s*\|', ahead):
                j += 1
                continue
            break

        error_groups[error_code].append({
            "location": location,
            "message": message,
            "help": help_lines,
        })
        i = j
        continue

    return dict(error_groups)


def save_state(error_groups):
    state_file = Path(STATE_FILE)
    state_file.parent.mkdir(parents=True, exist_ok=True)
    serializable = {k: v for k, v in error_groups.items()}
    with open(state_file, "w") as f:
        json.dump(serializable, f, indent=2)


def load_state():
    state_file = Path(STATE_FILE)
    if not state_file.exists():
        return None
    with open(state_file) as f:
        return json.load(f)


def print_summary(error_groups):
    total = sum(len(v) for v in error_groups.values())
    print(f"# Total: {total} diagnostics in {len(error_groups)} classes")
    for code, diags in sorted(error_groups.items()):
        print(f"  {code}: {len(diags)}")


def print_errors(error_groups, error_code, limit=10, offset=0):
    if error_code not in error_groups:
        print(f"No errors for: {error_code}")
        print(f"Available: {', '.join(sorted(error_groups.keys()))}")
        return

    diags = error_groups[error_code]
    total = len(diags)

    if offset >= total:
        print(f"{error_code}: offset {offset} beyond {total} total")
        return

    end = min(offset + limit, total)
    subset = diags[offset:end]

    print(f"{error_code}: [{offset+1}-{end} of {total}]")
    for diag in subset:
        if diag["location"]:
            print(f"  {diag['location']}")
        print(f"    {diag['message']}")
        for h in diag.get("help", [])[:2]:
            print(f"    help: {h}")
        print("")


def print_examples(error_groups, error_code):
    if error_code not in error_groups:
        return

    diags = error_groups[error_code]
    locations = [d["location"] for d in diags if d["location"]]

    print(f"{error_code}: {len(locations)} files")
    for loc in locations[:20]:
        print(f"  {loc}")
    if len(locations) > 20:
        print(f"  ... and {len(locations) - 20} more")


def print_help():
    print("Cargo Browser REPL Commands:")
    print("  scan <cmd>         Run cargo command and parse output")
    print("                      Example: scan cargo clippy 2>&1")
    print("  summary            Show error class summary")
    print("  show <code> [n]    Show n errors (default: 10)")
    print("  show <code> [n] [off]  Show starting at offset")
    print("  examples <code>   Show file locations only")
    print("  next <code> [n]   Continue from last position")
    print("  clear              Clear all error state")
    print("  help               Show this message")
    print("  exit/quit          Exit REPL")


def run_repl():
    """Run the interactive REPL."""
    error_groups = {}
    offsets = {}  # Track offset per error_code

    print("Cargo Browser REPL")
    print("Type 'help' for commands, 'exit' to quit")
    print("")

    # Try to load existing state
    saved = load_state()
    if saved:
        error_groups = saved
        print(f"Loaded {sum(len(v) for v in saved.values())} diagnostics from state")
        print_summary(error_groups)
    else:
        print("No saved state. Use 'scan <command>' to load errors.")
    print("")

    while True:
        try:
            cmd = input("cargo-browser> ").strip()
            if not cmd:
                continue
            if cmd in ("exit", "quit"):
                print("Exiting...")
                break
            if cmd == "help":
                print_help()
                continue
            if cmd == "clear":
                error_groups = {}
                offsets = {}
                Path(STATE_FILE).unlink(missing_ok=True)
                print("Cleared state")
                continue
            if cmd == "summary":
                print_summary(error_groups)
                continue

            # Parse commands
            parts = cmd.split()
            cmd_name = parts[0]

            if cmd_name == "scan":
                if len(parts) < 2:
                    print("Usage: scan <command>")
                    print("Example: scan cargo clippy 2>&1")
                    continue
                cargo_cmd = " ".join(parts[1:])
                print(f"Running: {cargo_cmd}")
                result = subprocess.run(
                    cargo_cmd,
                    shell=True,
                    capture_output=True,
                    text=True
                )
                error_groups = parse_diagnostics(result.stdout.splitlines())
                save_state(error_groups)
                offsets = {}  # Reset offsets on new scan
                total = sum(len(v) for v in error_groups.values())
                print(f"Parsed {total} diagnostics")
                print_summary(error_groups)

            elif cmd_name == "show":
                if len(parts) < 2:
                    print("Usage: show <code> [limit] [offset]")
                    continue
                code = parts[1]
                limit = int(parts[2]) if len(parts) > 2 else 10
                off = int(parts[3]) if len(parts) > 3 else 0
                print_errors(error_groups, code, limit, off)

            elif cmd_name == "next":
                if len(parts) < 2:
                    print("Usage: next <code> [limit]")
                    continue
                code = parts[1]
                limit = int(parts[2]) if len(parts) > 2 else 10
                off = offsets.get(code, 0)
                print_errors(error_groups, code, limit, off)
                # Update offset for next call
                offsets[code] = off + limit

            elif cmd_name == "examples":
                if len(parts) < 2:
                    print("Usage: examples <code>")
                    continue
                print_examples(error_groups, parts[1])

            else:
                print(f"Unknown command: {cmd_name}")
                print("Type 'help' for available commands")

        except EOFError:
            print("\nExiting...")
            break
        except KeyboardInterrupt:
            print("\nExiting...")
            break
        except Exception as e:
            print(f"Error: {e}")


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Interactive error browser REPL")
    parser.add_argument("--repl", action="store_true", help="Start interactive REPL")
    parser.add_argument("--scan", metavar="CMD", help="Scan cargo command and enter REPL")
    args = parser.parse_args()

    if args.scan:
        # Run scan then enter REPL
        result = subprocess.run(args.scan, shell=True, capture_output=True, text=True)
        error_groups = parse_diagnostics(result.stdout.splitlines())
        save_state(error_groups)
        total = sum(len(v) for v in error_groups.values())
        print(f"Parsed {total} diagnostics from scan")
        print_summary(error_groups)
        print("Entering REPL...")
        run_repl()
    elif args.repl:
        run_repl()
    else:
        # Pipe mode - parse and save
        if not sys.stdin.isatty():
            lines = sys.stdin.readlines()
            error_groups = parse_diagnostics(lines)
            save_state(error_groups)
            total = sum(len(v) for v in error_groups.values())
            print(f"Parsed {total} diagnostics into {len(error_groups)} error classes")
            print("Run with --repl to enter interactive mode")
        else:
            # No input, no args - show help
            parser.print_help()


if __name__ == "__main__":
    main()
