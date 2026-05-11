# Workspace (live task state)

## Current task
ITER-0004d.0 — **DONE 2026-05-10**. Tooling precursor shipped (`fmpl_core::diagnostics` library + `no_legacy_fmpl_syntax` CI gate + baseline JSON). Old `stdlib_no_legacy_syntax.rs` deleted. Pre-iteration PAR returned REVISE; user chose "build as-spec'd, surface real data." Post-implementation PAR caught 2 inline-fixable bugs (lying `rust_byte_offset` field; operator-symbol false positives) — fixed in commit `a4752461`. 4 architectural concerns deferred to ITER-0004d.1 with explicit documentation in the iteration log.

## Ground-truth baseline now available (was speculation)
- `lib/core` = 0 hits (AC-13 preserved at lexer-token level, stricter than ITER-0004c's regex gate)
- `src/rs` = 43 hits
- `tests/fmpl` = 72 hits (dominated by orphan fixtures `ast_to_ir.fmpl` + `fmpl_parser.fmpl`; ITER-0004d.1 step 5 deletes these)
- `tests/rs` = 625 hits (concentrated in fewer than 14 files; ITER-0004d.1 step 3 sweep target)

## Open issues for ITER-0004d.1 planning (NEXT)

### Planning-level (rolled forward from prior checkpoint)
1. **AC-14 reframe still applies** — `grep -rn 'Value::Tagged(' fmpl-core/tests/` already returns 0. Per EPIC-002.md:134, AC-14's literal requirement is met today. Decide for d.1 scope: (a) drop AC-14 from d.1 (already done) and treat the FMPL-string sweep as a new AC; OR (b) reuse AC-14 as "still satisfied" + the gate-flip as new work. Recommendation: tie the gate-flip work to the existing AC-13 sentinel (which the new gate now enforces) and leave AC-14 as a Rust-test invariant that's already met. The diagnostics tool's `tests/rs` and `src/rs` surfaces are the new evidence; AC-14 needs no new ACs.
2. **ITER-0004d.2 rename map still dead-on-arrival** — both PAR R3 reviewers caught that d.1's consumer-arm deletions remove ALL the compiler.rs emit sites previously listed for renaming. Real surviving emit sites: `ir_builder.rs:238-240`, `builtins/ir.rs:336/776`, `builtins/ir_to_rust.rs:543` (missed entirely in prior plan), plus variant defs + VM handlers. Rebuild the rename map AFTER d.1 lands and `cargo check` enumerates the post-deletion landscape.
3. **`compiler.rs:2665` MatchTag does NOT survive d.1** — it's inside the deleted Pattern::Constructor arm. Only `:2533` survives. SCENARIO-0107's MatchTag grep assertion needs fixing.
4. **8+ test consumers of `Pattern::Tagged`** at `tests/pattern_unification.rs:39,43,176,279,284` and `tests/context_aware_compilation.rs:95,340,549` block d.1 compilation. Add to scope.

### New issues from ITER-0004d.0 post-implementation PAR (for d.1 to address)
5. **Macro-body coverage gap** — `syn::visit::Visit` doesn't descend into macro `TokenStream` bodies (`eval!(":Foo(1, 2)")`, `assert_eq!(eval(":Foo(1)"), ...)`). Today's codebase doesn't trip this (grep-verified), but d.1's `== 0` flip should add a `visit_macro` override or document as accepted risk.
6. **Tests-walk vs src-walk asymmetry** — `tests/` is walked flat, `src/` recursively. A future test helper under `tests/subdir/` would be silently unwatched. Cheap fix in d.1.
7. **Strict-equality baseline workflow trap** — any developer who incidentally reduces a hit count forces a `FMPL_REGEN_BASELINE=1` ritual. d.1 should switch to `>= baseline` OR document the regen workflow before the `== 0` flip removes the baseline.
8. **Coarse allowlist** — `(path-suffix, tag)` suppresses every matching tag in the file. A future legitimate `:first(args)` hit under d.1's explicit-rejection regime would be silently swallowed. Consider narrowing to `(path, byte_offset_range)` if the false-suppression window proves real.

### Format/citation polish (lower priority — rolled forward)
9. **AC-tag format violation** in ITER-0004d.1 step 14 — existing convention is `· scenario:\`SCENARIO-NNNN\`` (backticks, single scenario per AC). Step 14 proposes comma-separated without backticks. No precedent for multi-scenario tags. Pick one primary scenario per AC.
10. **STORY-0095/AC-4 rewrite text** missing backticks; seam value changed `unit` (was `integration`) without justification.
11. **Multiple compiler.rs line citations wrong** — `compiler.rs:3111` doesn't emit MatchTagged (only ExtractTaggedChild); `:3803-3812` is a third Pattern::Tagged emit arm missed; `:3638` and `:4360-4394` are missing from Pattern::TagMatch consumer enumeration.
12. **No SCENARIO entry** for the new FMPL-string CI gate (could be SCENARIO-0106's structural observable, or a new SCENARIO).
13. **SCENARIO-0104/0105 grammar-DSL surface error-message ambiguity** unresolved.

## Sentinel baseline (current, post-ITER-0004d.0)
- `cargo test -p fmpl-core --test ast_to_ir_parity`: 55 passed
- `cargo test -p fmpl-core --test scenario_0103_optimizer_pipeline`: 32 passed, 1 ignored
- `cargo test -p fmpl-core --test ac7_optimizer_pass_through`: 8 passed
- `cargo test -p fmpl-core --test no_legacy_fmpl_syntax`: 1 passed (NEW sentinel, replacing stdlib_no_legacy_syntax)
- `cargo test -p fmpl-core --test diagnostics_fmpl_source_scan`: 13 passed (NEW unit tests)
- `cargo test -p fmpl-core --test stdlib_no_legacy_syntax`: DELETED — subsumed by no_legacy_fmpl_syntax.

## Active hypotheses (next session)
- ITER-0004d.1 plan can now be replanned against concrete baseline numbers rather than roadmap-text inference. The `cargo check`-driven enumeration of deletion targets (per active hypothesis B from the prior session) is the right approach — start d.1 with a fresh sentinel run, then dispatch the parser/AST burn against the real consumer set, not speculation.
- Treat ITER-0004d.2 as deferred until ITER-0004d.1 has actually built. Rename map depends on what survives post-deletion.

## Checkpoints
- [x] REVIEW_QUEUE cleared (11 candidates rejected — all activity-log noise)
- [x] Sentinel baseline confirmed clean
- [x] Pre-iteration consistency audit (88 citations valid; EPIC-002 counter accurate)
- [x] PAR round 1 of d.0 → REVISE (user chose build-as-spec'd)
- [x] Implementer dispatched + 4 atomic commits
- [x] Post-impl PAR → REVISE → 3 inline fixes applied
- [x] Sentinel + new gate + unit tests all passing
- [x] ITER-0004d.0 marked done; iteration-log entry written
- [ ] ITER-0004d.1 fresh-session replan with real baseline data
- [ ] ITER-0004d.2 deferred until d.1 lands
- [ ] ITER-0004h Type::Tagged cleanup deferred

## Next step
ITER-0004d.1 replanning. Read this file + iteration-log entry for d.0 (especially the "Known limitations carried forward" section) + d.1's roadmap entry (lines 386-494). The 4 d.0 PAR-deferred concerns plus the 4 rolled-forward planning issues are the d.1 input set. Real baseline JSON is the authoritative ground truth — d.1's sweep target is `tests/rs=625, src/rs=43, tests/fmpl=72` (plus deletion of the 72 orphan-fixture hits via step 5).
