1. Run full `cargo test -p fmpl-core` (unfiltered health check). `jj describe` is blocked until this passes.
2. If tests fail, you return to IMPLEMENT.
3. If tests pass, consolidate discoveries to `docs/codebase/` if applicable.
4. Commit with `jj describe -m '<conventional commit message>'`.

Then output: COMPLETED:<id> <conventional commit message>
