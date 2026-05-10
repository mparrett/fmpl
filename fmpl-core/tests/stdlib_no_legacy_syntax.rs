//! AC-13 CI gate (ITER-0004c): the FMPL stdlib must use list-pattern syntax
//! exclusively — no legacy `:Tag(args)` constructions or patterns remain.
//!
//! This gate locks the invariant established by ITER-0004c against silent
//! regressions in the window between ITER-0004c and ITER-0004d (during which
//! the legacy parser still ACCEPTS `:Tag(args)` syntax). Without this gate,
//! a future stdlib edit could re-introduce legacy syntax and the canonical
//! representation contract would silently degrade.
//!
//! The check operates on text that has been pre-processed to strip:
//!  - FMPL line comments (`-- ... \n`)
//!  - FMPL string literals (`"..."` with escape handling)
//!
//! This avoids false positives on documentation/comment text and on code
//! emitted as string output (e.g., `lib/core/ir_to_rust.fmpl` contains a
//! Rust-source string literal with `Value::Bool(true)` etc., which are Rust
//! enum constructors, not FMPL syntax).

use std::fs;
use std::path::PathBuf;

/// Strip FMPL string literals and `--` line comments from source. Preserves
/// line counts (replaces stripped content with spaces / empty) so file:line
/// references in error messages remain accurate.
fn strip_strings_and_comments(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];

        // Line comment: `--` to end of line.
        if b == b'-' && i + 1 < bytes.len() && bytes[i + 1] == b'-' {
            // Consume to newline (preserve the newline itself).
            while i < bytes.len() && bytes[i] != b'\n' {
                out.push(' ');
                i += 1;
            }
            // Don't consume the newline; loop iteration handles it.
            continue;
        }

        // String literal: `"..."` with `\` escape handling.
        if b == b'"' {
            out.push(' ');
            i += 1;
            while i < bytes.len() {
                let c = bytes[i];
                if c == b'\\' && i + 1 < bytes.len() {
                    // Skip the escape sequence (preserve newlines if any).
                    if bytes[i + 1] == b'\n' {
                        out.push(' ');
                        out.push('\n');
                    } else {
                        out.push(' ');
                        out.push(' ');
                    }
                    i += 2;
                    continue;
                }
                if c == b'"' {
                    out.push(' ');
                    i += 1;
                    break;
                }
                if c == b'\n' {
                    out.push('\n');
                } else {
                    out.push(' ');
                }
                i += 1;
            }
            continue;
        }

        // Regular character: keep it.
        out.push(b as char);
        i += 1;
    }
    out
}

/// Return the workspace root (parent of CARGO_MANIFEST_DIR).
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR has a parent")
        .to_path_buf()
}

/// AC-13: All `lib/core/*.fmpl` files must use canonical list-pattern syntax
/// exclusively. After stripping comments and string literals, no occurrence
/// of `:Tag(` (capital-letter tag followed by paren) may remain.
#[test]
fn ac13_stdlib_no_legacy_tag_paren_syntax() {
    let lib_core = workspace_root().join("lib").join("core");
    assert!(
        lib_core.is_dir(),
        "expected lib/core directory at {:?}",
        lib_core
    );

    let entries = fs::read_dir(&lib_core)
        .expect("read lib/core directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("fmpl"))
        .map(|e| e.path())
        .collect::<Vec<_>>();
    assert!(
        !entries.is_empty(),
        "no .fmpl files found in lib/core; AC-13 check would vacuously pass"
    );

    // Hand-rolled scanner for `:[A-Z][a-zA-Z_]*\(` — avoids adding a regex
    // dev-dependency just for this gate. Returns the matched substring or None.
    fn find_legacy_syntax(line: &str) -> Option<&str> {
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b':' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_uppercase() {
                let start = i;
                i += 2; // consume `:` and the uppercase letter
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                if i < bytes.len() && bytes[i] == b'(' {
                    // Match: `:Tag(`
                    return Some(&line[start..=i]);
                }
                continue;
            }
            i += 1;
        }
        None
    }

    let mut violations: Vec<String> = Vec::new();
    for path in &entries {
        let content = fs::read_to_string(path).expect("read fmpl file");
        let stripped = strip_strings_and_comments(&content);
        for (line_no, line) in stripped.lines().enumerate() {
            if let Some(m) = find_legacy_syntax(line) {
                violations.push(format!(
                    "{}:{}: {} ({})",
                    path.display(),
                    line_no + 1,
                    m,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "AC-13 violation: legacy :Tag(args) syntax found in stdlib (after stripping \
         comments and string literals):\n{}",
        violations.join("\n")
    );
}
