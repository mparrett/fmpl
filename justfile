# FMPL task runner. Run `just` to list targets.

# Build the FMPL-generated (canonical) parser, then build the whole workspace.
# See README "Build & run" for why this is two steps.
build: bootstrap
    cargo build --workspace

# Step 1 of the metacircular bootstrap: build fmpl-bootstrap (parser generation
# is skipped on this pass), then touch fmpl-core's build script so the next
# build regenerates the canonical parser from lib/core/fmpl_parser.fmpl.
bootstrap:
    FMPL_BOOTSTRAP_PHASE=1 cargo build -p fmpl-bootstrap
    touch fmpl-core/build.rs

# Run the full test suite (canonical parser active).
test: bootstrap
    cargo test --workspace

# Lint with zero warnings (workspace-wide, never per-test-target).
clippy:
    cargo clippy --workspace --all-targets

# Launch the REPL. Commands are dot-prefixed: .help, .quit
repl:
    cargo run -p fmpl-cli

# Launch the web REPL on http://localhost:3000
web:
    cargo run -p fmpl-web

# Launch the TUI (Ctrl+L for LLM chat).
tui:
    cargo run -p fmpl-tui

# Default: list available targets.
default:
    @just --list
