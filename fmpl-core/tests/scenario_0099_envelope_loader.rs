//! SCENARIO-0099 evidence — Loader skips records with incompatible VM
//! version (and unknown payload kind, and corrupt checksum) without
//! aborting iteration.
//!
//! Per the ITER-0005a.1 PAR-revised card: this is a focused Rust
//! integration test (NOT a data-driven scenario step-def) because
//! per `feedback_prefer_proof_tests.md` the integration test directly
//! asserts on typed `(loaded, skipped_incompatible, skipped_corrupt,
//! skipped_unknown_kind)` u32 counters via `==` — closer to form #1
//! (typed invariants) than to form #5 (pointwise data-driven cases).
//!
//! AC-7 (`LoaderStats` public API) is deferred to ITER-0005a.2; this
//! test counts skips locally in the harness rather than through a
//! public stats surface.

use fmpl_core::persistence::envelope::{EnvelopeHeader, NO_SOURCE_HASH};
use fmpl_core::persistence::loader::{
    CorruptionReason, DecodeOutcome, IncompatibilityReason, UnknownKindReason, decode,
};
use fmpl_core::persistence::schema::{PayloadKind, VM_VERSION_MAJOR};
use zerocopy::IntoBytes;
use zerocopy::little_endian::U16;

/// Construct a complete, well-formed envelope value for `kind` carrying
/// `payload`.
fn build_record(kind: PayloadKind, payload: &[u8]) -> Vec<u8> {
    let hdr = EnvelopeHeader::new_for_current_vm(kind, payload.len() as u32, NO_SOURCE_HASH)
        .finalize_checksum(payload);
    let mut value = Vec::with_capacity(56 + payload.len());
    value.extend_from_slice(hdr.as_bytes());
    value.extend_from_slice(payload);
    value
}

#[test]
fn scenario_0099_six_record_skip_journey() {
    // Per SCENARIO-0099's expected observables (extended in the
    // ITER-0005a.1 audit fix-up to cover all three AC-3 sub-conditions
    // at the integration seam):
    //
    // - Record A: written by current VM version, schema known → loads.
    // - Record B: vm_version major one ahead → skipped (incompatible).
    // - Record C: known vm_version, UNKNOWN payload_kind → skipped (unknown kind).
    // - Record D: CRC32 deliberately corrupted → skipped (corrupt).
    // - Record E: known payload_kind, UNKNOWN schema_version → skipped (unknown kind).
    // - Record F: known payload_kind + schema, NONZERO reserved flags → skipped (unknown kind).
    //
    // Harness-local counters (AC-7 LoaderStats deferred to ITER-0005a.2):
    //   loaded = 1, skipped_incompatible = 1, skipped_unknown_kind = 3, skipped_corrupt = 1.
    //
    // The three skipped_unknown_kind sub-cases each correspond to a
    // distinct AC-3 sub-condition; the test asserts on the specific
    // reason variant per record, not just the umbrella outcome.

    // Record A — well-formed current VM, current schema.
    let record_a = build_record(PayloadKind::CompiledCode, b"payload A");

    // Record B — vm_version major one ahead. Construct manually so the
    // header carries a future major; finalize_checksum still uses our
    // current `compute()` so the checksum is well-formed for the header
    // as written.
    let record_b = {
        let payload = b"payload B";
        let mut hdr = EnvelopeHeader::new_for_current_vm(
            PayloadKind::CompiledCode,
            payload.len() as u32,
            NO_SOURCE_HASH,
        );
        hdr.vm_version_major = U16::new(VM_VERSION_MAJOR.wrapping_add(1));
        let hdr = hdr.finalize_checksum(payload);
        let mut value = Vec::with_capacity(56 + payload.len());
        value.extend_from_slice(hdr.as_bytes());
        value.extend_from_slice(payload);
        value
    };

    // Record C — unknown payload_kind (AC-3 sub-condition 1).
    let record_c = {
        let payload = b"payload C";
        let mut hdr = EnvelopeHeader::new_for_current_vm(
            PayloadKind::CompiledCode,
            payload.len() as u32,
            NO_SOURCE_HASH,
        );
        hdr.payload_kind = 0xEE; // not in the taxonomy
        let hdr = hdr.finalize_checksum(payload);
        let mut value = Vec::with_capacity(56 + payload.len());
        value.extend_from_slice(hdr.as_bytes());
        value.extend_from_slice(payload);
        value
    };

    // Record D — CRC32 deliberately corrupted (tamper with a payload
    // byte; the stamped checksum no longer matches).
    let record_d = {
        let mut value = build_record(PayloadKind::CompiledCode, b"payload D");
        let last = value.len() - 1;
        value[last] ^= 0xFF;
        value
    };

    // Record E — unknown schema_version for a known payload_kind
    // (AC-3 sub-condition 2). Audit-fix-up addition.
    let record_e = {
        let payload = b"payload E";
        let mut hdr = EnvelopeHeader::new_for_current_vm(
            PayloadKind::CompiledCode,
            payload.len() as u32,
            NO_SOURCE_HASH,
        );
        // Bump schema_version to a far-future value the current
        // PayloadKind::CompiledCode does not recognize.
        hdr.schema_version = U16::new(0xFFFF);
        let hdr = hdr.finalize_checksum(payload);
        let mut value = Vec::with_capacity(56 + payload.len());
        value.extend_from_slice(hdr.as_bytes());
        value.extend_from_slice(payload);
        value
    };

    // Record F — nonzero reserved flags (AC-3 sub-condition 3 — the
    // reserved-must-be-zero flag byte). Audit-fix-up addition.
    let record_f = {
        let payload = b"payload F";
        let mut hdr = EnvelopeHeader::new_for_current_vm(
            PayloadKind::CompiledCode,
            payload.len() as u32,
            NO_SOURCE_HASH,
        );
        hdr.flags = 0x01;
        let hdr = hdr.finalize_checksum(payload);
        let mut value = Vec::with_capacity(56 + payload.len());
        value.extend_from_slice(hdr.as_bytes());
        value.extend_from_slice(payload);
        value
    };

    // Simulate a keyspace iterator yielding these six records in order.
    let records: [(&str, &[u8]); 6] = [
        ("a", &record_a),
        ("b", &record_b),
        ("c", &record_c),
        ("d", &record_d),
        ("e", &record_e),
        ("f", &record_f),
    ];

    // Harness-local counters (AC-7 LoaderStats deferred to ITER-0005a.2).
    let mut loaded: u32 = 0;
    let mut skipped_incompatible: u32 = 0;
    let mut skipped_unknown_kind: u32 = 0;
    let mut skipped_corrupt: u32 = 0;

    for (key, value) in records {
        let (outcome, decoded) = decode(value);
        match outcome {
            DecodeOutcome::Loaded => {
                let rec = decoded.expect("loaded record yields DecodedRecord");
                // Sanity: only record A should reach here.
                assert_eq!(key, "a");
                assert_eq!(rec.kind, PayloadKind::CompiledCode);
                assert_eq!(rec.payload, b"payload A");
                loaded += 1;
            }
            DecodeOutcome::SkippedIncompatible(IncompatibilityReason::VmMajorMismatch) => {
                assert_eq!(key, "b");
                assert!(decoded.is_none());
                skipped_incompatible += 1;
            }
            DecodeOutcome::SkippedIncompatible(other) => {
                panic!("record {key} skipped incompatible for unexpected reason: {other:?}");
            }
            DecodeOutcome::SkippedUnknownKind(UnknownKindReason::UnknownPayloadKind) => {
                assert_eq!(key, "c");
                skipped_unknown_kind += 1;
            }
            DecodeOutcome::SkippedUnknownKind(UnknownKindReason::UnknownSchemaVersion) => {
                assert_eq!(key, "e");
                skipped_unknown_kind += 1;
            }
            DecodeOutcome::SkippedUnknownKind(UnknownKindReason::NonzeroReservedFlags) => {
                assert_eq!(key, "f");
                skipped_unknown_kind += 1;
            }
            DecodeOutcome::SkippedCorrupt(CorruptionReason::ChecksumMismatch) => {
                assert_eq!(key, "d");
                skipped_corrupt += 1;
            }
            DecodeOutcome::SkippedCorrupt(other) => {
                panic!("record {key} skipped corrupt for unexpected reason: {other:?}");
            }
        }
    }

    // SCENARIO-0099's expected observables.
    assert_eq!(loaded, 1, "exactly one record (A) should load");
    assert_eq!(
        skipped_incompatible, 1,
        "exactly one record (B) should skip-incompatible"
    );
    assert_eq!(
        skipped_unknown_kind, 3,
        "exactly three records (C, E, F) should skip-unknown-kind"
    );
    assert_eq!(
        skipped_corrupt, 1,
        "exactly one record (D) should skip-corrupt"
    );
}
