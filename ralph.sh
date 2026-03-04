#!/usr/bin/env bash
# ralph.sh — Two-phase headless task loop for FMPL development
#
# Phase 1: Run PROMPT.md through claude, capture full stream-json log
# Phase 2: Run ralph-analyze.py on the log to extract structured summary
#
# Usage:
#   ./ralph.sh                     # Run one iteration (default)
#   ./ralph.sh -n 5                # Run up to 5 iterations
#   ./ralph.sh -n 0                # Run until blocked or Ctrl-C
#   ./ralph.sh -p PROMPT2.md       # Use alternate prompt file
#   ./ralph.sh --analyze LOGFILE   # Skip phase 1, just analyze a log
#   ./ralph.sh --dry-run           # Show config without executing

set -euo pipefail

# --- Configuration ---
MAX_ITERATIONS=1
PROMPT_FILE="PROMPT.md"
LOG_DIR=".ralph-logs"
DRY_RUN=false
ANALYZE_ONLY=""
PURGE=false
CLAUDE_FLAGS="--dangerously-skip-permissions --verbose"

# --- Parse arguments ---
while [[ $# -gt 0 ]]; do
    case "$1" in
        -n|--max-iterations)
            MAX_ITERATIONS="$2"
            shift 2
            ;;
        -p|--prompt)
            PROMPT_FILE="$2"
            shift 2
            ;;
        --analyze)
            ANALYZE_ONLY="$2"
            shift 2
            ;;
        --purge)
            PURGE=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            cat <<'HELP'
Usage: ralph.sh [OPTIONS]

Phase 1 (execute):
  -n, --max-iterations N   Max iterations (0=unlimited, default: 1)
  -p, --prompt FILE        Prompt file (default: PROMPT.md)
  --dry-run                Show config without running

Phase 2 (analyze):
  --analyze LOGFILE        Analyze a previous iteration log (skip phase 1)
  --purge                  Remove raw .jsonl logs, keep summaries and session logs
HELP
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ANALYZE_SCRIPT="$SCRIPT_DIR/ralph-analyze.py"

# --- Analyze-only mode ---
if [[ -n "$ANALYZE_ONLY" ]]; then
    if [[ ! -f "$ANALYZE_ONLY" ]]; then
        echo "Error: $ANALYZE_ONLY not found" >&2
        exit 1
    fi
    if [[ -f "$ANALYZE_SCRIPT" ]]; then
        python3 "$ANALYZE_SCRIPT" "$ANALYZE_ONLY"
    else
        echo "Error: $ANALYZE_SCRIPT not found" >&2
        exit 1
    fi
    exit 0
fi

# --- Purge mode ---
if [[ "$PURGE" == "true" ]]; then
    if [[ ! -d "$LOG_DIR" ]]; then
        echo "Nothing to purge: $LOG_DIR does not exist"
        exit 0
    fi
    # Delete raw stream-json logs but keep summaries (*.summary.txt)
    RAW_COUNT=$(find "$LOG_DIR" \( -name 'iter-*.jsonl' -o -name 'iter-*.txt' \) -not -name '*.summary.txt' | wc -l | tr -d ' ')
    RAW_SIZE=$(find "$LOG_DIR" \( -name 'iter-*.jsonl' -o -name 'iter-*.txt' \) -not -name '*.summary.txt' -exec stat -f%z {} + 2>/dev/null | awk '{s+=$1} END {printf "%.1fMB", s/1048576}')
    if [[ "$RAW_COUNT" -eq 0 ]]; then
        echo "Nothing to purge"
        exit 0
    fi
    echo "Purging $RAW_COUNT raw logs ($RAW_SIZE)"
    echo "Keeping: *.summary.txt, session-*.log, results-*.jsonl"
    find "$LOG_DIR" \( -name 'iter-*.jsonl' -o -name 'iter-*.txt' \) -not -name '*.summary.txt' -delete
    echo "Done"
    exit 0
fi

# --- Validate ---
if [[ ! -f "$PROMPT_FILE" ]]; then
    echo "Error: $PROMPT_FILE not found" >&2
    exit 1
fi

if ! command -v claude &>/dev/null; then
    echo "Error: claude CLI not found in PATH" >&2
    exit 1
fi

# --- Setup ---
mkdir -p "$LOG_DIR"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
SESSION_LOG="$LOG_DIR/session-$TIMESTAMP.log"
RESULTS_LOG="$LOG_DIR/results-$TIMESTAMP.jsonl"

log() {
    local msg="[$(date +%H:%M:%S)] $*"
    echo "$msg"
    echo "$msg" >> "$SESSION_LOG"
}

# --- Dry run ---
if [[ "$DRY_RUN" == "true" ]]; then
    echo "Ralph Loop Configuration:"
    echo "  Prompt:         $PROMPT_FILE ($(wc -l < "$PROMPT_FILE" | tr -d ' ') lines)"
    echo "  Max iterations: $MAX_ITERATIONS (0=unlimited)"
    echo "  Log dir:        $LOG_DIR"
    echo "  Claude flags:   $CLAUDE_FLAGS"
    echo "  Analyze script: $ANALYZE_SCRIPT ($(test -f "$ANALYZE_SCRIPT" && echo 'found' || echo 'MISSING'))"
    echo ""
    echo "Would run: echo '...' | claude $CLAUDE_FLAGS -p --append-system-prompt \"\$(cat $PROMPT_FILE)\" --output-format=stream-json"
    exit 0
fi

# --- Main loop ---
ITERATION=0
COMPLETED=0
BLOCKED=0
CLOSED=0

log "Ralph loop started: prompt=$PROMPT_FILE max=$MAX_ITERATIONS"

INTERRUPTED=false

cleanup() {
    echo ""
    if [[ "$INTERRUPTED" == "true" ]]; then
        log "Interrupted by user (Ctrl-C)"
    fi
    log "=== Ralph Loop Summary ==="
    log "Iterations: $ITERATION  Completed: $COMPLETED  Blocked: $BLOCKED  Closed: $CLOSED"
    log "Session: $SESSION_LOG"
    log "========================="
}
trap cleanup EXIT
trap 'INTERRUPTED=true; exit 130' INT

while true; do
    ITERATION=$((ITERATION + 1))

    # Check max iterations (0 = unlimited)
    if [[ "$MAX_ITERATIONS" -gt 0 && "$ITERATION" -gt "$MAX_ITERATIONS" ]]; then
        log "Reached max iterations ($MAX_ITERATIONS)"
        break
    fi

    log "--- Iteration $ITERATION ---"
    ITER_START=$(date +%s)
    ITER_NUM=$(printf '%03d' "$ITERATION")
    RAW_LOG="$LOG_DIR/iter-$TIMESTAMP-$ITER_NUM.jsonl"
    SUMMARY="$LOG_DIR/iter-$TIMESTAMP-$ITER_NUM.summary.txt"

    # Phase 1: Run claude with PROMPT.md as system prompt (cacheable),
    # minimal user message to trigger execution.
    PROMPT_CONTENT=$(cat "$PROMPT_FILE")
    echo "Execute the task loop. Output COMPLETED:<id>, BLOCKED:<id>, or CLOSED:<id> when done." | \
        claude $CLAUDE_FLAGS -p \
            --append-system-prompt "$PROMPT_CONTENT" \
            --output-format=stream-json 2>/dev/null | \
        tee "$RAW_LOG" | \
        jq -j --unbuffered '
            if .type == "assistant" then
                .message.content[]? |
                if .type == "text" then .text
                elif .type == "tool_use" then "\n[tool: \(.name)]\n"
                else empty
                end
            elif .type == "result" then
                "\n[result: turns=\(.num_turns) cost=$\(.total_cost_usd)]\n"
            else empty
            end
        ' 2>/dev/null || true

    ITER_END=$(date +%s)
    ITER_DURATION=$((ITER_END - ITER_START))
    RAW_LINES=$(wc -l < "$RAW_LOG" | tr -d ' ')
    log "Raw log: $RAW_LOG ($RAW_LINES events, ${ITER_DURATION}s)"

    # Phase 2: Extract structured summary
    if [[ -f "$ANALYZE_SCRIPT" && "$RAW_LINES" -gt 0 ]]; then
        python3 "$ANALYZE_SCRIPT" "$RAW_LOG" > "$SUMMARY" 2>/dev/null || true
    fi

    # Parse result from raw log
    RESULT_LINE=""
    if [[ "$RAW_LINES" -gt 0 ]]; then
        # Extract the text output and look for our structured result line
        RESULT_LINE=$(jq -r '
            if .type == "assistant" then
                .message.content[]? | select(.type == "text") | .text
            else empty end
        ' "$RAW_LOG" 2>/dev/null | grep -E '^(COMPLETED|BLOCKED|CLOSED):' | tail -1 || true)
    fi

    if [[ -n "$RESULT_LINE" ]]; then
        TYPE=$(echo "$RESULT_LINE" | cut -d: -f1)
        ISSUE_ID=$(echo "$RESULT_LINE" | cut -d: -f2 | xargs)
        MESSAGE=$(echo "$RESULT_LINE" | cut -d: -f3- | xargs)

        case "$TYPE" in
            COMPLETED)
                COMPLETED=$((COMPLETED + 1))
                log "COMPLETED [$ISSUE_ID] $MESSAGE (${ITER_DURATION}s)"
                ;;
            BLOCKED)
                BLOCKED=$((BLOCKED + 1))
                log "BLOCKED [$ISSUE_ID] $MESSAGE (${ITER_DURATION}s)"
                log "Stopping: task is blocked"
                break
                ;;
            CLOSED)
                CLOSED=$((CLOSED + 1))
                log "CLOSED [$ISSUE_ID] $MESSAGE (${ITER_DURATION}s)"
                ;;
        esac

        echo "{\"iteration\":$ITERATION,\"type\":\"$TYPE\",\"issue\":\"$ISSUE_ID\",\"message\":\"$MESSAGE\",\"duration\":$ITER_DURATION,\"log\":\"$RAW_LOG\"}" >> "$RESULTS_LOG"
    else
        # Extract cost from result event even if no structured output
        COST=$(jq -r 'select(.type == "result") | .total_cost_usd // 0' "$RAW_LOG" 2>/dev/null | tail -1 || echo "0")
        TURNS=$(jq -r 'select(.type == "result") | .num_turns // 0' "$RAW_LOG" 2>/dev/null | tail -1 || echo "0")
        log "NO STRUCTURED OUTPUT (${ITER_DURATION}s, \$$COST, ${TURNS} turns) — check $RAW_LOG"
        echo "{\"iteration\":$ITERATION,\"type\":\"unstructured\",\"issue\":\"\",\"message\":\"no result line\",\"duration\":$ITER_DURATION,\"cost\":$COST,\"turns\":$TURNS,\"log\":\"$RAW_LOG\"}" >> "$RESULTS_LOG"

        # Stop if claude produced nothing (startup failure, auth issue, etc.)
        if [[ "$RAW_LINES" -eq 0 ]]; then
            log "Stopping: claude produced no output"
            break
        fi
        # Otherwise continue — work may have been done, just output format not followed.
    fi
done
