# Continuations and Bytecode Design Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the opaque continuation system and a stable Indexed RPN bytecode encoding suitable for image persistence.

**Architecture:** Add a versioned bytecode encoding in `fmpl-core` with deterministic serialization and opcode table versioning. Implement SnapshotEnvelope and continuation storage in `fmpl-web`, integrating with image store bootstrapping and source blob retention.

**Tech Stack:** Rust, fmpl-core, fmpl-web (Axum), rkyv, Fjall

---

### Task 1: Define bytecode format module

**Files:**
- Create: `fmpl-core/src/bytecode/mod.rs`
- Create: `fmpl-core/src/bytecode/format.rs`
- Modify: `fmpl-core/src/lib.rs`
- Test: `fmpl-core/src/bytecode/format.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_encode_decode_roundtrip() {
    let code = CompiledCode::new();
    let bytes = encode_bytecode(&code).unwrap();
    let decoded = decode_bytecode(&bytes).unwrap();
    assert_eq!(decoded.instructions.len(), 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p fmpl-core test_encode_decode_roundtrip`
Expected: FAIL with missing module/functions.

**Step 3: Write minimal implementation**

```rust
pub const BYTECODE_VERSION: u16 = 1;

pub fn encode_bytecode(code: &CompiledCode) -> Result<Vec<u8>> { /* header + empty tables */ }
pub fn decode_bytecode(bytes: &[u8]) -> Result<CompiledCode> { /* parse header + empty tables */ }
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p fmpl-core test_encode_decode_roundtrip`
Expected: PASS

**Step 5: Commit**

```bash
git add fmpl-core/src/bytecode/mod.rs fmpl-core/src/bytecode/format.rs fmpl-core/src/lib.rs
git commit -m "feat: bytecode format scaffolding"
```

---

### Task 2: Freeze opcode table and operand encoding

**Files:**
- Modify: `fmpl-core/src/compiler.rs`
- Modify: `fmpl-core/src/bytecode/format.rs`
- Test: `fmpl-core/src/bytecode/format.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_opcode_encoding_stability() {
    let mut code = CompiledCode::new();
    code.instructions.push(Instruction::LoadInt(7));
    let bytes = encode_bytecode(&code).unwrap();
    assert!(bytes.len() > 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p fmpl-core test_opcode_encoding_stability`
Expected: FAIL with unimplemented encoding.

**Step 3: Write minimal implementation**

```rust
// format.rs
fn opcode_id(instr: &Instruction) -> u16 { /* fixed mapping */ }
fn encode_operands(...) -> Vec<u8> { /* varint or fixed */ }
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p fmpl-core test_opcode_encoding_stability`
Expected: PASS

**Step 5: Commit**

```bash
git add fmpl-core/src/compiler.rs fmpl-core/src/bytecode/format.rs
git commit -m "feat: freeze opcode table and encoding"
```

---

### Task 3: Add canonical string table

**Files:**
- Modify: `fmpl-core/src/bytecode/format.rs`
- Test: `fmpl-core/src/bytecode/format.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_string_table_dedup() {
    let mut code = CompiledCode::new();
    code.instructions.push(Instruction::LoadString(SmolStr::new("x")));
    code.instructions.push(Instruction::LoadString(SmolStr::new("x")));
    let bytes = encode_bytecode(&code).unwrap();
    let decoded = decode_bytecode(&bytes).unwrap();
    assert_eq!(decoded.instructions.len(), 2);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p fmpl-core test_string_table_dedup`
Expected: FAIL with missing table.

**Step 3: Write minimal implementation**

```rust
// format.rs
// Build string table in first-seen order, de-duplicate entries.
// Instructions reference string indices.
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p fmpl-core test_string_table_dedup`
Expected: PASS

**Step 5: Commit**

```bash
git add fmpl-core/src/bytecode/format.rs
git commit -m "feat: canonical string table"
```

---

### Task 4: SnapshotEnvelope and rkyv encoding

**Files:**
- Create: `fmpl-core/src/snapshot.rs`
- Modify: `fmpl-core/src/lib.rs`
- Test: `fmpl-core/src/snapshot.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_snapshot_envelope_roundtrip() {
    let env = SnapshotEnvelope::new(vec![1,2,3]);
    let bytes = env.to_bytes().unwrap();
    let decoded = SnapshotEnvelope::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.payload, vec![1,2,3]);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p fmpl-core test_snapshot_envelope_roundtrip`
Expected: FAIL with missing module.

**Step 3: Write minimal implementation**

```rust
pub struct SnapshotEnvelope { /* version fields + payload */ }

impl SnapshotEnvelope {
    pub fn to_bytes(&self) -> Result<Vec<u8>> { /* rkyv encode */ }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> { /* rkyv decode */ }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p fmpl-core test_snapshot_envelope_roundtrip`
Expected: PASS

**Step 5: Commit**

```bash
git add fmpl-core/src/snapshot.rs fmpl-core/src/lib.rs
git commit -m "feat: snapshot envelope"
```

---

### Task 5: Integrate bytecode persistence with source blobs

**Files:**
- Modify: `fmpl-core/src/compiler.rs`
- Modify: `fmpl-core/src/vm.rs`
- Test: `fmpl-core/src/compiler.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_compiled_code_keeps_source() {
    let code = Compiler::new().compile(&parse("1 + 2").unwrap()).unwrap();
    assert!(code.source.is_some());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p fmpl-core test_compiled_code_keeps_source`
Expected: FAIL with `source` None.

**Step 3: Write minimal implementation**

```rust
// compiler.rs
// Initialize CompiledCode with source in eval/compile entrypoints.
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p fmpl-core test_compiled_code_keeps_source`
Expected: PASS

**Step 5: Commit**

```bash
git add fmpl-core/src/compiler.rs fmpl-core/src/vm.rs
git commit -m "feat: retain source blobs in compiled code"
```

---

Plan complete and saved to `docs/plans/2025-12-24-continuations-bytecode-implementation-plan.md`. Two execution options:

1. Subagent-Driven (this session) - I dispatch fresh subagent per task, review between tasks, fast iteration
2. Parallel Session (separate) - Open new session with executing-plans, batch execution with checkpoints

Which approach?
