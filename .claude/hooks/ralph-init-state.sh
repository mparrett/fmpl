#!/usr/bin/env bash
# Initialize ralph state machine for a new iteration.
# Called by ralph.sh before each claude invocation.
#
# Usage: ralph-init-state.sh [--clear]
#   --clear: remove state file (disable state machine)

STATE_FILE="$(dirname "$0")/../.ralph-state.json"

if [[ "${1:-}" == "--clear" ]]; then
    rm -f "$STATE_FILE"
    echo "Ralph state machine disabled"
    exit 0
fi

cat > "$STATE_FILE" << 'EOF'
{
  "state": "HEALTH_CHECK",
  "task_id": null,
  "close_count": 0,
  "has_written_code": false,
  "health_fix": false,
  "decomposed": false,
  "tests_passed": false,
  "clippy_passed": false,
  "verify_failed": false
}
EOF

echo "Ralph state machine initialized: HEALTH_CHECK"
