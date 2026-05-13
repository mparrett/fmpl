//! Image persistence for FMPL.
//!
//! Lays the wire-format foundation that ITER-0005 persistence work is built
//! on top of. Per the ITER-0005a.1 scope card:
//!
//! - [`schema`] — single source of truth for VM version + per-payload-kind
//!   schema versions + the `PayloadKind` taxonomy. AC-6 of STORY-0099.
//! - [`envelope`] — fixed-layout binary header that wraps every persisted
//!   record (`Object`, `CompiledCode`, `Grammar`, `ParseState`, `MemoTable`,
//!   `VmSnapshot`). AC-1 of STORY-0099.
//! - [`checksum`] — blake3-truncated-to-32 integrity check over the
//!   envelope header (with `crc32` zeroed) and payload. AC-4 of STORY-0099.
//! - [`loader`] — keyspace iterator that decodes envelopes and skips
//!   incompatible / corrupt records without aborting iteration. AC-2,
//!   AC-3, AC-4 of STORY-0099.
//!
//! `LoaderStats` (AC-7) is intentionally deferred to ITER-0005a.2 where the
//! swept production callers will surface stats to operators (see roadmap.md
//! for the PAR-revised scope discipline).

pub mod checksum;
pub mod envelope;
pub mod loader;
pub mod schema;
