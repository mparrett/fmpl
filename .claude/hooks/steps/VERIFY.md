Run verification:

1. ONE `cargo test` run (filtered to your changes). Must pass.
2. ONE `cargo clippy` run (filtered). Must pass, zero warnings.

If tests/clippy fail, you'll return to IMPLEMENT. Fix and retry.
If both pass, you'll advance to REVIEW.
