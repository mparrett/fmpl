# Continuations and Bytecode Design

## Goal

Define a Seaside-style continuation system with opaque tokens and a stable Indexed RPN bytecode format that supports reflection and transparent persistence in an image-based, multi-user live programming environment.

---

## Continuation Model (Opaque Tokens)

### Overview

Use server-stored snapshots keyed by opaque continuation tokens. Tokens are scoped to the current user/session and validated on lookup.

**Spike note:** execution is single-vat only; cross-vat or eventual-send semantics are deferred.

### Token Format

- `token`: 128-bit random value (CSPRNG), base64url or base32 encoded
- `key`: `cont:{session_id}:{token}`

### Lifecycle

1. `GET /play` creates a new VM state, stores a snapshot, redirects to `/play/{token}`.
2. `GET /play/{token}` loads snapshot, runs `current_storylet.render(ctx)`.
3. `ctx.link(target)` computes a new state, stores it, returns `/play/{new_token}`.

### Security

- Tokens are only valid within the authenticated session.
- Reject continuation lookups that do not match `session_id`.
- Optional: mark continuations "consumed" for non-idempotent actions.

### Retention

- Per-session cap (e.g., 200 continuations) with LRU eviction.
- TTL (e.g., 24h) with background cleanup.

---

## Snapshot Envelope

All persisted state is wrapped in a stable envelope for versioning and migration.

```text
SnapshotEnvelope {
  schema_version: u16,
  bytecode_version: u16,
  engine_version: u32,
  created_at: u64,
  payload_format: "rkyv-v1" | "serde-json-v1",
  payload: bytes
}
```

- `schema_version`: changes to VM/object graph layout
- `bytecode_version`: changes to opcode table or operand encoding
- `engine_version`: compatibility guard for runtime behavior

---

## Image-Based Live Programming Environment

### Model

- The system is an image: a persistent object graph + compiled code shared across users.
- Every object and compiled method lives in the image store.
- Evaluations mutate the live image with transactional snapshots.
- Each user operates in a session context with access to shared objects via facets/capabilities.

### Storage Layout (Fjall)

**Image store:**
- `obj:{id}` -> serialized object
- `code:{id}` -> compiled code blob
- `meta:root` -> root object or world state

**Sessions:**
- `session:{id}` -> user/session metadata (identity, permissions, active facets)

**Continuation store:**
- `cont:{session_id}:{token}` -> SnapshotEnvelope

---

## Stable Indexed RPN Bytecode

### Goals

- Deterministic serialization for snapshotting and reflection
- Stable opcode IDs across versions
- Append-only evolution for opcodes

### Encoding

- **Header**:
  - `bytecode_version: u16`
  - `opcode_table_version: u16` (optional if fixed)
  - `string_table_len: u32`
  - `instr_count: u32`
- **String table**: ordered, index-based
- **Instruction stream**: `(opcode_id, operands...)`
  - `opcode_id`: u8 or u16
  - operands: varint (u32/u64), or fixed-width for speed

### Opcode Versioning Rules

- Never reorder or reuse opcode IDs.
- New opcodes append to the table.
- Operand encoding never changes for existing opcodes.

### Canonical String Table

- Strings are sorted by first appearance in instruction emission order.
- Identical strings are de-duplicated.
- Operands refer to string table indices, not raw strings.

---

## Reflection Support

### Requirements

- `object.method_source(:name)` returns original FMPL source.
- `lambda.source()` returns source snippet when available.
- Bytecode can be decompiled into a normalized representation.

### Strategy

- Store original source in `CompiledCode.source` where available.
- Store per-object method source alongside compiled code.
- For missing source, fall back to decompiler output.

---

## Deterministic Serialization

### Rules

- Use ordered maps for serialization (`BTreeMap` or `IndexMap` with sorted keys).
- Avoid pointer-based IDs; use stable numeric IDs.
- Store data in a canonical traversal order.

### Rkyv

- Primary format for snapshots.
- Maintain a stable schema version; migrations on load.

### JSON

- Secondary format for debug/inspection only.
- Add `serde::Serialize` only when JSON/debug export is required.

---

## Open Questions

- Should continuation snapshots include VM call stack, or just the object graph + current storylet?
- How do we version object layouts when reflection changes (e.g., new fields)?
- How large can per-session continuation retention be without impacting storage?

---

## Next Steps

1. Define the opcode ID table and operand encoding for Indexed RPN v1.
2. Implement SnapshotEnvelope and rkyv schema with version tags.
3. Add continuation store in `fmpl-web` with opaque tokens and session validation.
4. Add object/code persistence hooks in `fmpl-core` and VM.
