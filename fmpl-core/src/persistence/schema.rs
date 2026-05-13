//! Schema and version registry for the persistence envelope.
//!
//! Satisfies AC-6 of STORY-0099: VM version is derived from
//! `CARGO_PKG_VERSION` at build time and embedded automatically;
//! payload-kind-specific schema versions live here so additions are
//! tracked centrally.
//!
//! The AC-6 anti-rot ratchet is enforced by a separate test that scans
//! `fmpl-core/src/` for any file outside this module referencing
//! `CARGO_PKG_VERSION` or the `vm_version_*` identifiers. See
//! `fmpl-core/tests/persistence_schema_anti_rot.rs`.

/// Envelope wire-format version. Bumped when the [`EnvelopeHeader`]
/// struct's layout itself changes (field added, removed, reordered, or
/// resized). Each version dispatches to a distinct decoder.
///
/// [`EnvelopeHeader`]: super::envelope::EnvelopeHeader
pub const ENVELOPE_FORMAT_VERSION: u16 = 1;

/// VM major version, derived at build time from `CARGO_PKG_VERSION`.
///
/// The loader's AC-2 skip path triggers when a persisted record's
/// `vm_version_major` differs from this constant.
pub const VM_VERSION_MAJOR: u16 = parse_version_part(env!("CARGO_PKG_VERSION"), 0);

/// VM minor version, derived at build time from `CARGO_PKG_VERSION`.
pub const VM_VERSION_MINOR: u16 = parse_version_part(env!("CARGO_PKG_VERSION"), 1);

/// VM patch version, derived at build time from `CARGO_PKG_VERSION`.
pub const VM_VERSION_PATCH: u16 = parse_version_part(env!("CARGO_PKG_VERSION"), 2);

/// Envelope payload-kind taxonomy.
///
/// Wire encoding is a single `u8`; unknown bytes are NOT a panic — the
/// loader's AC-3 skip path treats them as "skip this record." See
/// [`PayloadKind::from_byte`].
///
/// Variant numbering is reserved-and-documented at ITER-0005a.1 entry;
/// subsequent iterations populate the matching writer paths but MUST NOT
/// renumber existing variants (wire-format stability).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PayloadKind {
    /// Per-object body record (ITER-0005d sweep of `object.rs`).
    ObjectRecord = 0x01,
    /// The `__object_ids__` index record listing every ObjectId in the keyspace.
    /// Distinct kind because `object.rs::save_to_fjall` writes both shapes,
    /// and ITER-0005a.2's "zero raw `keyspace.insert`" invariant gate must
    /// be satisfiable across both.
    ObjectIndex = 0x02,
    /// Bytecode (`CompiledCode`) — proof case for ITER-0005c.
    CompiledCode = 0x03,
    /// Grammar definition (ITER-0005d).
    Grammar = 0x04,
    /// Top-level `GrammarRegistry` (ITER-0005d).
    GrammarRegistry = 0x05,
    /// Incremental parse state (ITER-0005d).
    ParseState = 0x06,
    /// Grammar memo cache (ITER-0005d).
    MemoTable = 0x07,
    /// Full-VM snapshot envelope (ITER-0005e).
    VmSnapshot = 0x08,
    /// Stream-position spill record for incremental grammar parsing —
    /// the inner-list spillover written by
    /// `grammar/stream_input.rs::spill_to_fjall`. Payload shape is
    /// `Option<Vec<u8>>` (a JSON-encoded optional head value, where the
    /// inner Vec<u8> is itself a serialized `Value`). Distinct from
    /// `ParseState` because the on-disk shape differs — they are wire-
    /// format ambiguous if they share a PayloadKind discriminator.
    /// Reserved at ITER-0005a.2 audit fix-up (G2) after both PAR
    /// auditors flagged the collision.
    StreamPosition = 0x09,
}

impl PayloadKind {
    /// Wire-byte → typed variant. Returns `None` for unknown bytes so the
    /// loader can route AC-3 ("unknown payload_kind → skip") gracefully.
    pub const fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::ObjectRecord),
            0x02 => Some(Self::ObjectIndex),
            0x03 => Some(Self::CompiledCode),
            0x04 => Some(Self::Grammar),
            0x05 => Some(Self::GrammarRegistry),
            0x06 => Some(Self::ParseState),
            0x07 => Some(Self::MemoTable),
            0x08 => Some(Self::VmSnapshot),
            0x09 => Some(Self::StreamPosition),
            _ => None,
        }
    }

    /// Current schema version for this payload kind. The loader's AC-3
    /// skip path triggers when a record's `schema_version` does not match
    /// this constant for the record's kind.
    ///
    /// Initial version for every kind is 1. Subsequent iterations may
    /// bump these constants when a payload schema actually changes.
    pub const fn current_schema_version(self) -> u16 {
        match self {
            Self::ObjectRecord => 1,
            Self::ObjectIndex => 1,
            Self::CompiledCode => 1,
            Self::Grammar => 1,
            Self::GrammarRegistry => 1,
            Self::ParseState => 1,
            Self::MemoTable => 1,
            Self::VmSnapshot => 1,
            Self::StreamPosition => 1,
        }
    }

    /// Wire byte representation.
    pub const fn as_byte(self) -> u8 {
        self as u8
    }
}

/// `const fn` that parses the Nth dotted component of a version string
/// like `"0.3.0"`. Index 0 → major, 1 → minor, 2 → patch. Returns 0 for
/// missing components or non-digit input — semver pre-releases (`-rc.1`)
/// are silently truncated at the first non-digit, which is acceptable
/// for the version-bump-detection use case the loader's AC-2 skip
/// needs.
const fn parse_version_part(s: &str, index: usize) -> u16 {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut current_part = 0;
    let mut value: u32 = 0;
    let mut has_digit = false;

    while i < bytes.len() {
        let b = bytes[i];
        if b == b'.' {
            if current_part == index {
                return value as u16;
            }
            current_part += 1;
            value = 0;
            has_digit = false;
            i += 1;
            continue;
        }
        if b < b'0' || b > b'9' {
            // Non-digit (e.g., `-rc.1` pre-release marker). Stop reading
            // digits for the current part; treat the part as complete.
            if current_part == index {
                return value as u16;
            }
            // Skip past the rest of this part until next `.` or end.
            while i < bytes.len() && bytes[i] != b'.' {
                i += 1;
            }
            continue;
        }
        value = value * 10 + (b - b'0') as u32;
        has_digit = true;
        i += 1;
    }

    if current_part == index && has_digit {
        return value as u16;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_parsing_handles_standard_semver() {
        assert_eq!(parse_version_part("1.2.3", 0), 1);
        assert_eq!(parse_version_part("1.2.3", 1), 2);
        assert_eq!(parse_version_part("1.2.3", 2), 3);
    }

    #[test]
    fn version_parsing_handles_zero_minor_patch() {
        assert_eq!(parse_version_part("0.3.0", 0), 0);
        assert_eq!(parse_version_part("0.3.0", 1), 3);
        assert_eq!(parse_version_part("0.3.0", 2), 0);
    }

    #[test]
    fn version_parsing_handles_pre_release_markers() {
        assert_eq!(parse_version_part("1.2.3-rc.1", 0), 1);
        assert_eq!(parse_version_part("1.2.3-rc.1", 1), 2);
        assert_eq!(parse_version_part("1.2.3-rc.1", 2), 3);
    }

    #[test]
    fn version_parsing_handles_missing_parts() {
        assert_eq!(parse_version_part("1.2", 0), 1);
        assert_eq!(parse_version_part("1.2", 1), 2);
        assert_eq!(parse_version_part("1.2", 2), 0);
    }

    #[test]
    fn payload_kind_roundtrips_through_wire_byte() {
        for kind in [
            PayloadKind::ObjectRecord,
            PayloadKind::ObjectIndex,
            PayloadKind::CompiledCode,
            PayloadKind::Grammar,
            PayloadKind::GrammarRegistry,
            PayloadKind::ParseState,
            PayloadKind::MemoTable,
            PayloadKind::VmSnapshot,
            PayloadKind::StreamPosition,
        ] {
            assert_eq!(PayloadKind::from_byte(kind.as_byte()), Some(kind));
        }
    }

    #[test]
    fn unknown_payload_byte_returns_none() {
        // Reserved variants currently unmapped — must round-trip cleanly
        // through the loader's AC-3 skip path. 0x09 is now StreamPosition
        // (added in the ITER-0005a.2 audit fix-up G2).
        for b in [0x00, 0x0A, 0x10, 0x42, 0xFF] {
            assert!(
                PayloadKind::from_byte(b).is_none(),
                "byte {b:#x} should be unknown",
            );
        }
    }

    #[test]
    fn current_schema_version_is_one_for_every_kind() {
        // Sanity check: every kind lands at version 1. A future iteration
        // bumping a single kind's schema MUST update this constant for
        // that kind. Failing this test means the contract just got broken
        // (good; that's the point).
        for kind in [
            PayloadKind::ObjectRecord,
            PayloadKind::ObjectIndex,
            PayloadKind::CompiledCode,
            PayloadKind::Grammar,
            PayloadKind::GrammarRegistry,
            PayloadKind::ParseState,
            PayloadKind::MemoTable,
            PayloadKind::VmSnapshot,
            PayloadKind::StreamPosition,
        ] {
            assert_eq!(kind.current_schema_version(), 1);
        }
    }

    #[test]
    fn vm_version_constants_are_derived_from_cargo_pkg_version() {
        // Sanity check: the constants exist and parse without panicking.
        // Actual value depends on workspace version; we can't hardcode
        // it. Just prove the const fn ran without crashing.
        let _ = VM_VERSION_MAJOR;
        let _ = VM_VERSION_MINOR;
        let _ = VM_VERSION_PATCH;
    }
}
