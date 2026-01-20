# FMPL Scratchpad

## Current Focus: Streaming Grammar Push-Model Implementation - COMPLETE

All tasks for the streaming grammar push-model implementation are complete.

## Task Status

### Streaming Grammar Push-Model (docs/plans/2026-01-20-streaming-grammar-push-model-implementation-plan.md)

- [x] Task 1: ParseState/ParseNext types (53b27a0)
- [x] Task 2: Fjall backing for StreamPosition (b2c5daf)
- [x] Task 3: Incremental parse API (start/resume) (67536dc)
- [x] Task 4: ParseDriver for streaming pipelines (d137df4)
- [x] Task 5: Wire |> operator to ParseDriver (AsyncParse StreamOp) (18991d1)
- [x] Task 6: Fjall persistence for memo tables (04949ff)
- [x] Task 7: ParseState serialization (`to_bytes`/`from_bytes`) (c178edf)
- [x] Task 8: Integration tests for durable suspension (33e08a2)
- [x] Task 9: Documentation - COMPLETE

### ParseState Serialization (docs/plans/2026-01-20-parse-state-serialization-implementation-plan.md)

- [x] Task 1: Add to_bytes/from_bytes to ParseState (c178edf)
- [x] Task 2: Add Fjall Storage Helper for ParseState (save_to_fjall/load_from_fjall) (33e08a2)
- [x] Task 3: Integration Test - Durable Parse Suspension (33e08a2)
- [x] Task 4: Update Implementation Plan Status - COMPLETE
- [x] Task 5: Update streaming-grammar.md Spec - COMPLETE

## Next Steps

The streaming grammar implementation is complete. Potential next work areas:
- Other pending changes in Cargo.toml, stream.rs (rkyv support for stream types)
- New feature development from unified-grammars-and-agents-design.md
