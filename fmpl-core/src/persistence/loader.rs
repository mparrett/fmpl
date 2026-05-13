//! Envelope-aware loader for persisted keyspaces.
//!
//! Iterates a Fjall keyspace, decodes each value as an
//! [`EnvelopeHeader`]-prefixed record, and routes each entry into one
//! of:
//!
//! - **Loaded** — magic ok, format version known, VM major matches,
//!   `flags == 0`, payload kind known + schema version matches, and
//!   checksum verifies. The decoder returns the `(EnvelopeHeader,
//!   payload)` pair for the downstream caller.
//! - **Skipped (incompatible)** — VM major mismatch (AC-2) OR envelope
//!   format version unrecognized.
//! - **Skipped (unknown kind)** — `PayloadKind::from_byte` returned
//!   `None`, OR schema version doesn't match the current expectation
//!   for a known kind (AC-3). Also covers nonzero `flags` (reserved-
//!   must-be-zero rejection).
//! - **Skipped (corrupt)** — magic mismatch, header byte length under
//!   56, payload length doesn't match `value.len() - 56`, or checksum
//!   mismatch (AC-4).
//!
//! Each skip logs the record key and a reason; iteration continues
//! over the next `(key, value)` from the keyspace iterator. No byte
//! arithmetic — each Fjall value is its own self-contained envelope.
//!
//! [`EnvelopeHeader`]: super::envelope::EnvelopeHeader

use super::envelope::{ENVELOPE_HEADER_SIZE, EnvelopeHeader};
use super::schema::{ENVELOPE_FORMAT_VERSION, PayloadKind, VM_VERSION_MAJOR};
use zerocopy::FromBytes;

/// Result of decoding one keyspace entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeOutcome {
    /// Record loaded cleanly. Header + payload are available via the
    /// helper accessor methods on [`DecodedRecord`].
    Loaded,
    /// AC-2: VM-major mismatch or unrecognized envelope-format-version.
    SkippedIncompatible(IncompatibilityReason),
    /// AC-3: unknown payload kind, unknown schema version for a known
    /// kind, or nonzero `flags`.
    SkippedUnknownKind(UnknownKindReason),
    /// AC-4: magic mismatch, length mismatch, or checksum mismatch.
    SkippedCorrupt(CorruptionReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncompatibilityReason {
    /// VM major version on disk differs from this build's
    /// `VM_VERSION_MAJOR`.
    VmMajorMismatch,
    /// Envelope format version on disk is not 1 (or whatever the
    /// current `ENVELOPE_FORMAT_VERSION` is).
    UnknownEnvelopeFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownKindReason {
    /// `payload_kind` byte does not map to any known `PayloadKind`.
    UnknownPayloadKind,
    /// `schema_version` does not match the current value for the
    /// known `payload_kind`.
    UnknownSchemaVersion,
    /// `flags` byte is nonzero (reserved must-be-zero).
    NonzeroReservedFlags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorruptionReason {
    /// Value shorter than [`ENVELOPE_HEADER_SIZE`].
    ValueTooShort,
    /// Magic bytes are not `b"FMPL"`.
    BadMagic,
    /// `header.payload_len` does not equal `value.len() - 56`.
    PayloadLengthMismatch,
    /// Stamped CRC field doesn't match the recomputed blake3 truncation.
    ChecksumMismatch,
}

/// Decode a single keyspace value into a typed outcome plus, when
/// `Loaded`, a borrowed header + payload pair.
///
/// Borrowing rather than copying preserves the zero-copy benefit of the
/// [`zerocopy`]-derived header. Callers needing an owned header should
/// `*` the reference.
pub fn decode<'v>(value: &'v [u8]) -> (DecodeOutcome, Option<DecodedRecord<'v>>) {
    if value.len() < ENVELOPE_HEADER_SIZE {
        return (
            DecodeOutcome::SkippedCorrupt(CorruptionReason::ValueTooShort),
            None,
        );
    }

    let Ok((header, payload)) = EnvelopeHeader::ref_from_prefix(value) else {
        // ref_from_prefix only fails on size; we already checked. This
        // arm exists for defensive completeness.
        return (
            DecodeOutcome::SkippedCorrupt(CorruptionReason::ValueTooShort),
            None,
        );
    };

    if header.magic != super::envelope::MAGIC {
        return (
            DecodeOutcome::SkippedCorrupt(CorruptionReason::BadMagic),
            None,
        );
    }

    if header.envelope_format_version.get() != ENVELOPE_FORMAT_VERSION {
        return (
            DecodeOutcome::SkippedIncompatible(IncompatibilityReason::UnknownEnvelopeFormat),
            None,
        );
    }

    if header.vm_version_major.get() != VM_VERSION_MAJOR {
        return (
            DecodeOutcome::SkippedIncompatible(IncompatibilityReason::VmMajorMismatch),
            None,
        );
    }

    if header.flags != 0 {
        return (
            DecodeOutcome::SkippedUnknownKind(UnknownKindReason::NonzeroReservedFlags),
            None,
        );
    }

    let Some(kind) = PayloadKind::from_byte(header.payload_kind) else {
        return (
            DecodeOutcome::SkippedUnknownKind(UnknownKindReason::UnknownPayloadKind),
            None,
        );
    };

    if header.schema_version.get() != kind.current_schema_version() {
        return (
            DecodeOutcome::SkippedUnknownKind(UnknownKindReason::UnknownSchemaVersion),
            None,
        );
    }

    if header.payload_len.get() as usize != payload.len() {
        return (
            DecodeOutcome::SkippedCorrupt(CorruptionReason::PayloadLengthMismatch),
            None,
        );
    }

    if !header.verify_checksum(payload) {
        return (
            DecodeOutcome::SkippedCorrupt(CorruptionReason::ChecksumMismatch),
            None,
        );
    }

    (
        DecodeOutcome::Loaded,
        Some(DecodedRecord {
            header,
            payload,
            kind,
        }),
    )
}

/// Borrowed header + payload from a successfully decoded record.
#[derive(Debug)]
pub struct DecodedRecord<'v> {
    pub header: &'v EnvelopeHeader,
    pub payload: &'v [u8],
    pub kind: PayloadKind,
}

#[cfg(test)]
mod tests {
    use super::super::envelope::{EnvelopeHeader, NO_SOURCE_HASH};
    use super::super::schema::PayloadKind;
    use super::*;
    use zerocopy::IntoBytes;
    use zerocopy::little_endian::U16;

    /// Build a complete envelope value (header + payload) for a given
    /// kind. Used by tests to construct synthetic records the loader
    /// iterates over.
    fn build_record(kind: PayloadKind, payload: &[u8]) -> Vec<u8> {
        let hdr = EnvelopeHeader::new_for_current_vm(kind, payload.len() as u32, NO_SOURCE_HASH)
            .finalize_checksum(payload);
        let mut out = Vec::with_capacity(ENVELOPE_HEADER_SIZE + payload.len());
        out.extend_from_slice(hdr.as_bytes());
        out.extend_from_slice(payload);
        out
    }

    #[test]
    fn well_formed_record_loads() {
        let payload = b"hello payload";
        let value = build_record(PayloadKind::CompiledCode, payload);
        let (outcome, decoded) = decode(&value);
        assert_eq!(outcome, DecodeOutcome::Loaded);
        let rec = decoded.expect("loaded record should yield a DecodedRecord");
        assert_eq!(rec.kind, PayloadKind::CompiledCode);
        assert_eq!(rec.payload, payload);
    }

    #[test]
    fn short_value_skipped_corrupt() {
        let value = vec![0u8; 10];
        let (outcome, decoded) = decode(&value);
        assert_eq!(
            outcome,
            DecodeOutcome::SkippedCorrupt(CorruptionReason::ValueTooShort)
        );
        assert!(decoded.is_none());
    }

    #[test]
    fn bad_magic_skipped_corrupt() {
        let payload = b"x";
        let mut value = build_record(PayloadKind::ObjectIndex, payload);
        // Tamper with magic.
        value[0] = b'X';
        let (outcome, _) = decode(&value);
        assert_eq!(
            outcome,
            DecodeOutcome::SkippedCorrupt(CorruptionReason::BadMagic)
        );
    }

    #[test]
    fn vm_major_mismatch_skipped_incompatible() {
        let payload = b"x";
        let mut hdr = EnvelopeHeader::new_for_current_vm(
            PayloadKind::CompiledCode,
            payload.len() as u32,
            NO_SOURCE_HASH,
        );
        hdr.vm_version_major = U16::new(super::VM_VERSION_MAJOR.wrapping_add(1));
        let hdr = hdr.finalize_checksum(payload);
        let mut value = Vec::new();
        value.extend_from_slice(hdr.as_bytes());
        value.extend_from_slice(payload);
        let (outcome, _) = decode(&value);
        assert_eq!(
            outcome,
            DecodeOutcome::SkippedIncompatible(IncompatibilityReason::VmMajorMismatch)
        );
    }

    #[test]
    fn unknown_envelope_format_skipped_incompatible() {
        let payload = b"x";
        let mut hdr = EnvelopeHeader::new_for_current_vm(
            PayloadKind::CompiledCode,
            payload.len() as u32,
            NO_SOURCE_HASH,
        );
        hdr.envelope_format_version = U16::new(0xFFFF);
        let hdr = hdr.finalize_checksum(payload);
        let mut value = Vec::new();
        value.extend_from_slice(hdr.as_bytes());
        value.extend_from_slice(payload);
        let (outcome, _) = decode(&value);
        assert_eq!(
            outcome,
            DecodeOutcome::SkippedIncompatible(IncompatibilityReason::UnknownEnvelopeFormat)
        );
    }

    #[test]
    fn unknown_payload_kind_skipped_unknown_kind() {
        let payload = b"x";
        let mut hdr = EnvelopeHeader::new_for_current_vm(
            PayloadKind::CompiledCode,
            payload.len() as u32,
            NO_SOURCE_HASH,
        );
        hdr.payload_kind = 0xEE; // unknown
        let hdr = hdr.finalize_checksum(payload);
        let mut value = Vec::new();
        value.extend_from_slice(hdr.as_bytes());
        value.extend_from_slice(payload);
        let (outcome, _) = decode(&value);
        assert_eq!(
            outcome,
            DecodeOutcome::SkippedUnknownKind(UnknownKindReason::UnknownPayloadKind)
        );
    }

    #[test]
    fn unknown_schema_version_skipped_unknown_kind() {
        let payload = b"x";
        let mut hdr = EnvelopeHeader::new_for_current_vm(
            PayloadKind::CompiledCode,
            payload.len() as u32,
            NO_SOURCE_HASH,
        );
        hdr.schema_version = U16::new(0xFFFF); // far-future schema
        let hdr = hdr.finalize_checksum(payload);
        let mut value = Vec::new();
        value.extend_from_slice(hdr.as_bytes());
        value.extend_from_slice(payload);
        let (outcome, _) = decode(&value);
        assert_eq!(
            outcome,
            DecodeOutcome::SkippedUnknownKind(UnknownKindReason::UnknownSchemaVersion)
        );
    }

    #[test]
    fn nonzero_flags_skipped_unknown_kind() {
        let payload = b"x";
        let mut hdr = EnvelopeHeader::new_for_current_vm(
            PayloadKind::CompiledCode,
            payload.len() as u32,
            NO_SOURCE_HASH,
        );
        hdr.flags = 0x01;
        let hdr = hdr.finalize_checksum(payload);
        let mut value = Vec::new();
        value.extend_from_slice(hdr.as_bytes());
        value.extend_from_slice(payload);
        let (outcome, _) = decode(&value);
        assert_eq!(
            outcome,
            DecodeOutcome::SkippedUnknownKind(UnknownKindReason::NonzeroReservedFlags)
        );
    }

    #[test]
    fn checksum_mismatch_skipped_corrupt() {
        let payload = b"x";
        let mut value = build_record(PayloadKind::CompiledCode, payload);
        // Corrupt the last byte of the payload (after the header).
        let last = value.len() - 1;
        value[last] ^= 0xFF;
        let (outcome, _) = decode(&value);
        assert_eq!(
            outcome,
            DecodeOutcome::SkippedCorrupt(CorruptionReason::ChecksumMismatch)
        );
    }

    #[test]
    fn payload_length_mismatch_skipped_corrupt() {
        let payload = b"hello payload";
        let mut value = build_record(PayloadKind::CompiledCode, payload);
        // Truncate payload by 1 byte; the recorded payload_len won't match.
        value.pop();
        let (outcome, _) = decode(&value);
        assert_eq!(
            outcome,
            DecodeOutcome::SkippedCorrupt(CorruptionReason::PayloadLengthMismatch)
        );
    }
}
