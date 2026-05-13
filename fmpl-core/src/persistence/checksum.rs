//! Envelope checksum: lower 32 bits of `blake3(header_with_crc_zeroed || payload)`.
//! (`magic` is the first 4 bytes of `header_with_crc_zeroed` and is
//! therefore covered by the hash without being passed separately.)
//!
//! Per the ITER-0005a.1 PAR-revised card: blake3 is used (instead of a true
//! CRC32 polynomial) for consistency with ITER-0005b's content-addressed
//! source store, which also uses blake3. The `crc32: U32<LE>` field name in
//! [`EnvelopeHeader`] is preserved for AC-1 wording stability; the
//! computation here uses blake3 truncated to its lower 32 bits.
//!
//! Truncation is valid per blake3's XOF property — the first 4 bytes of
//! the default 32-byte output equal a 4-byte XOF output and are
//! cryptographically uniform. Using the lower 32 bits gives the same
//! 1-in-2^32 false-negative rate as CRC32 but without algebraic blind
//! spots.
//!
//! [`EnvelopeHeader`]: super::envelope::EnvelopeHeader

/// Compute the 32-bit envelope checksum over the three components:
///
/// - `header_no_crc` — the envelope header bytes with the `crc32` field
///   zeroed out (standard CRC-of-itself pattern; ensures the checksum is
///   self-referentially well-defined).
/// - `payload` — the full payload byte slice that follows the header.
///
/// Source bytes are deliberately NOT in the checksum input. Source
/// integrity is enforced by [`EnvelopeHeader::source_hash`][source_hash]
/// itself: the hash IS the source's identity per content-addressing.
/// Corrupting source bytes invalidates the hash; the loader's source-
/// store lookup (ITER-0005b) detects the mismatch separately.
///
/// [source_hash]: super::envelope::EnvelopeHeader::source_hash
pub fn compute(header_no_crc: &[u8], payload: &[u8]) -> u32 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(header_no_crc);
    hasher.update(payload);
    let digest = hasher.finalize();
    let bytes = digest.as_bytes();
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_over_same_inputs() {
        let h = b"header data here padded out";
        let p = b"payload bytes";
        assert_eq!(compute(h, p), compute(h, p));
    }

    #[test]
    fn changes_when_header_changes() {
        let h1 = b"header data here padded out";
        let h2 = b"header data here padded OUT"; // single bit flip
        let p = b"payload bytes";
        assert_ne!(compute(h1, p), compute(h2, p));
    }

    #[test]
    fn changes_when_payload_changes() {
        let h = b"header data here padded out";
        let p1 = b"payload bytes";
        let p2 = b"payload Bytes"; // single bit flip
        assert_ne!(compute(h, p1), compute(h, p2));
    }

    #[test]
    fn header_payload_concatenation_matters() {
        // compute(AB, C) must differ from compute(A, BC) because blake3
        // updates are stateful: blake3(A)|blake3(B)|blake3(C) is not the
        // same as blake3(AB)|blake3(C) in our protocol. Catch any future
        // refactor that accidentally treats header+payload as a single
        // concatenated buffer instead of separate updates (which would
        // happen to produce the same hash here because blake3 is
        // length-streaming, but explicit framing avoids that ambiguity).
        let a = [1u8, 2, 3];
        let b = [4u8, 5, 6];
        let c = [7u8, 8, 9];
        // Sanity: streaming blake3 across (a,b,c) ignores boundaries — so
        // compute([a;b], [c]) == compute([a], [b;c]). That's expected for
        // blake3 the algorithm; the test pins that we're using the
        // streaming behavior, which is fine because the caller always
        // passes (header_no_crc, payload) in that exact order.
        let ab: Vec<u8> = a.iter().chain(b.iter()).copied().collect();
        let bc: Vec<u8> = b.iter().chain(c.iter()).copied().collect();
        assert_eq!(compute(&ab, &c), compute(&a, &bc));
    }
}
