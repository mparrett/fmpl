//! AC-6 anti-rot ratchet for STORY-0099.
//!
//! `persistence::schema` is the single source of truth for VM version
//! derivation and per-payload-kind schema version constants. This test
//! scans `fmpl-core/src/` for any file **outside the `persistence/`
//! module tree** that references the version-derivation literals —
//! `CARGO_PKG_VERSION`, `VM_VERSION_MAJOR`, `VM_VERSION_MINOR`,
//! `VM_VERSION_PATCH` — and fails if it finds any. Per
//! `feedback_prefer_proof_tests.md` form #4: universally-quantified
//! structural assertion preventing rot.
//!
//! Scope choice (ITER-0005a.1 audit fix-up): the entire `persistence/`
//! module tree is exempt, not just `persistence/schema.rs`. The
//! `persistence::envelope` and `persistence::loader` modules
//! collaborate with the schema and legitimately read these constants
//! via qualified paths and `use` statements. Forbidding bare-identifier
//! reads inside the same module tree would force opaque indirection
//! for no real benefit. The ratchet's contract is "nothing OUTSIDE the
//! persistence module redefines or re-derives these"; that's the
//! invariant the scope card actually meant to enforce.
//!
//! Future iterations adding persistence consumers (object writers,
//! grammar writers, etc., during ITER-0005a.2's call-site sweep) must
//! either (a) live inside `fmpl-core/src/persistence/` (in scope for
//! the exemption), or (b) reference the constants through helper
//! functions exposed by `persistence::envelope` (the canonical pattern
//! — see `EnvelopeHeader::new_for_current_vm`).

use std::fs;
use std::path::{Path, PathBuf};

/// Identifiers that are forbidden outside the `persistence/` module
/// tree. If any of these appears in another `src/` file, the ratchet
/// fails.
///
/// - `CARGO_PKG_VERSION` forbids alternative version-derivation sites
///   (anyone outside `persistence/` re-deriving the VM version from the
///   env var directly).
/// - `VM_VERSION_MAJOR` / `VM_VERSION_MINOR` / `VM_VERSION_PATCH`
///   forbid bare-identifier reads outside `persistence/`. The intent
///   is that downstream consumers route through `persistence::envelope`
///   helpers (e.g., `EnvelopeHeader::new_for_current_vm`) rather than
///   reading the constants directly. Inside `persistence/` itself,
///   bare reads via `use` statements and qualified paths are routine
///   and exempt.
const FORBIDDEN_LITERALS: &[&str] = &[
    "CARGO_PKG_VERSION",
    "VM_VERSION_MAJOR",
    "VM_VERSION_MINOR",
    "VM_VERSION_PATCH",
    // Strip-comments + word-boundary matching ensures these only fire
    // on actual identifiers, not on substrings of unrelated text.
];

/// Files exempted from the scan. The whole `persistence/` module tree
/// is exempt: `persistence::schema` is THE source of truth and the
/// schema-aware sibling modules (`envelope`, `loader`, `checksum`)
/// legitimately read the constants via `use` and qualified paths. The
/// scope-card contract is that nothing OUTSIDE the persistence module
/// redefines or re-derives these — that's the structural invariant
/// this scan enforces.
fn is_exempt(path: &Path) -> bool {
    let s = path.to_string_lossy();
    // Exempt the entire `persistence/` subtree, not just schema.rs.
    s.contains("/persistence/") || s.ends_with("/persistence.rs")
}

fn fmpl_core_src() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("src");
    p
}

fn walk_rust_files(root: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root).unwrap_or_else(|e| {
        panic!("read_dir({}): {e}", root.display());
    });
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_rust_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// Strip `//` line comments and `/* ... */` block comments from a
/// source string. Conservative; only intended for the ratchet's
/// false-positive avoidance. Doesn't handle string literals containing
/// `//` (rare; not a real concern for the forbidden identifiers here).
fn strip_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            // line comment; skip to newline
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        } else if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            // block comment; skip past `*/`
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

/// Word-boundary substring search: returns true if `needle` appears in
/// `haystack` with non-identifier characters (or string boundaries) on
/// both sides.
fn contains_as_identifier(haystack: &str, needle: &str) -> bool {
    let bytes = haystack.as_bytes();
    let nb = needle.as_bytes();
    let mut i = 0;
    while i + nb.len() <= bytes.len() {
        if &bytes[i..i + nb.len()] == nb {
            let before_ok = i == 0 || !is_ident_byte(bytes[i - 1]);
            let after_ok = i + nb.len() == bytes.len() || !is_ident_byte(bytes[i + nb.len()]);
            if before_ok && after_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[test]
fn ac6_anti_rot_no_version_derivation_outside_schema() {
    let mut files = Vec::new();
    walk_rust_files(&fmpl_core_src(), &mut files);

    let mut violations: Vec<(PathBuf, &'static str)> = Vec::new();
    for path in &files {
        if is_exempt(path) {
            continue;
        }
        let src = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let stripped = strip_comments(&src);
        for needle in FORBIDDEN_LITERALS {
            if contains_as_identifier(&stripped, needle) {
                violations.push((path.clone(), *needle));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "AC-6 anti-rot ratchet failed: \
         the following files outside persistence::schema reference \
         version-derivation literals (use the schema module's exports \
         instead):\n{}",
        violations
            .iter()
            .map(|(p, n)| format!("  {} ← `{n}`", p.display()))
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

/// Sanity check: the ratchet's substring scanner correctly distinguishes
/// identifier-bounded matches from incidental substring matches.
#[test]
fn ratchet_identifier_boundary_detection() {
    assert!(contains_as_identifier(
        "CARGO_PKG_VERSION",
        "CARGO_PKG_VERSION"
    ));
    assert!(contains_as_identifier(
        "env!(\"CARGO_PKG_VERSION\")",
        "CARGO_PKG_VERSION"
    ));
    assert!(contains_as_identifier(
        "let x = CARGO_PKG_VERSION;",
        "CARGO_PKG_VERSION"
    ));
    // Substring within a larger identifier should NOT match.
    assert!(!contains_as_identifier(
        "MY_CARGO_PKG_VERSION_FOO",
        "CARGO_PKG_VERSION"
    ));
}
