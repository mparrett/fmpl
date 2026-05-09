# Workspace (live task state)

## Current task
ITER-0004c — scope-review complete, ready for implementation. Iteration split into ITER-0004c (Phase A — stdlib migration + optimizer wiring) and ITER-0004d (Phase B — parser/AST burn) per 4 rounds of PAR review.

## Open files
- docs/superpowers/iterations/roadmap.md (ITER-0004c at ~line 199, ITER-0004d follows)
- docs/superpowers/iterations/progress.md (snapshot of current phase)

## Active hypotheses
- ITER-0004c is decomposable into 11 TDD-sized tasks (transformer build → dry-run → apply → wire → SCENARIO-0103 → un-ignore tests → wrap up). T1 is the gnarliest: build a tree-grammar transformer in FMPL itself.
- The transformer will be reusable across ITER-0004c and ITER-0004d (the latter sweeps Rust-test FMPL strings — different mechanics but same general idea).

## Checkpoints
- [x] sentinel corpus baseline (55/55 parity, 1170 workspace, citation check OK)
- [x] PAR scope review (4 rounds; APPROVE on round 4)
- [x] roadmap revision committed at mrqtrkvs dc096568
- [ ] decompose into TDD tasks (deferred to next session)
- [ ] dispatch implementing-tasks (deferred)
- [ ] post-iteration scenario runs + wrap up (deferred)

## Next step
Resume in fresh session with `continue iterative development with the existing plan`. The skill's running-an-iteration entry will read the roadmap, see ITER-0004c pending, and proceed to step 6 (decompose). The decomposition list lives in task #3's description (TaskList).
