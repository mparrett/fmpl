# Review Queue

**Pending:** 5
**Oldest staged:** 2026-05-10T20:40:55.273010+00:00

Run `python .agent/tools/list_candidates.py` for detail, then:
- `python .agent/tools/graduate.py <id> --rationale "..."` to accept
- `python .agent/tools/reject.py <id> --reason "..."` to reject
- Review in a batch so cross-candidate contradictions are caught.

## Priority order (top 10)

- **d49365f28837** (priority=33426.00, size=1857, rejections=1) — FAILURE in claude-code: Command failed: cat > /tmp/test_lex.rs << 'EOF' | THIS S
- **a6a5972d6a43** (priority=882.00, size=49, rejections=0) — FAILURE in claude-code: Command failed: rtk cargo test -p fmpl-core 2>&1 | grep 
- **1ee1e88edac8** (priority=405.00, size=30, rejections=1) — Tool Agent completed successfully
- **65dca9e74421** (priority=148.50, size=11, rejections=0) — High-stakes op completed (schema): python3 .agent/tools/reject.py 50986dd6bff7 -
- **3916ff8ccb90** (priority=121.50, size=9, rejections=0) — High-stakes op completed (migrate): jj describe -m "$(cat <<'EOF'
