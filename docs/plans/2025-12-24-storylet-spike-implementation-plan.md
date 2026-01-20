# Storylet Spike Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the storylet web spike with single-vat execution, opaque continuations, and HTTP event streams.

**Architecture:** Implement a minimal storylet engine in `fmpl-web` backed by Fjall (image store + continuation store). Use FMPL source seeding to bootstrap the image; compile bytecode at load/runtime. Add stream primitives in `fmpl-core` sufficient for HTTP event sources and storylet parsing, with `parse(g.rule)` + `@` operator support.

**Tech Stack:** Rust, fmpl-core, fmpl-web (Axum), Fjall, rkyv

---

### Task 1: Add stream AST + parser support (`stream {}` and operators)

**Files:**
- Modify: `fmpl-core/src/lexer.rs`
- Modify: `fmpl-core/src/ast.rs`
- Modify: `fmpl-core/src/parser.rs`
- Test: `fmpl-core/src/parser.rs` (new unit tests)

**Step 1: Write the failing test**

```rust
#[test]
fn test_parse_stream_literal() {
    let expr = parse("stream { http.request } ").unwrap();
    assert!(matches!(expr, Expr::StreamLiteral(_)));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p fmpl-core test_parse_stream_literal`
Expected: FAIL with "unexpected token" or missing `Expr::StreamLiteral`.

**Step 3: Write minimal implementation**

```rust
// ast.rs
pub enum Expr {
    // ...
    StreamLiteral(Box<Expr>),
}

// lexer.rs
#[token("stream")]
Stream,

// parser.rs
Token::Stream => self.parse_stream_literal(),

fn parse_stream_literal(&mut self) -> Result<Expr> {
    self.expect(&Token::Stream)?;
    self.expect(&Token::LBrace)?;
    let expr = self.parse_expr()?;
    self.expect(&Token::RBrace)?;
    Ok(Expr::StreamLiteral(Box::new(expr)))
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p fmpl-core test_parse_stream_literal`
Expected: PASS

**Step 5: Commit**

```bash
git add fmpl-core/src/lexer.rs fmpl-core/src/ast.rs fmpl-core/src/parser.rs
git commit -m "feat: parse stream literal"
```

---

### Task 2: Implement stream primitives (map/filter/flatMap/reduce) as special forms

**Files:**
- Modify: `fmpl-core/src/ast.rs`
- Modify: `fmpl-core/src/parser.rs`
- Modify: `fmpl-core/src/compiler.rs`
- Modify: `fmpl-core/src/vm.rs`
- Test: `fmpl-core/src/vm.rs` (new unit tests)

**Step 1: Write the failing test**

```rust
#[test]
fn test_stream_map_filter_flatmap() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let (s = stream { [1,2,3] })
        s |> map(\x x + 1) |> filter(\x x > 2)
    "#).unwrap();
    assert!(matches!(result, Value::Stream(_)));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p fmpl-core test_stream_map_filter_flatmap`
Expected: FAIL with missing stream runtime.

**Step 3: Write minimal implementation**

```rust
// ast.rs
pub enum Expr {
    // ...
    StreamOp(StreamOp, Box<Expr>, Vec<Arg>),
}

pub enum StreamOp { Map, Filter, FlatMap, Reduce }

// parser.rs
// Special forms: map/filter/flatMap/reduce parse as StreamOp

// compiler.rs / vm.rs
// Introduce Value::Stream and a minimal stream graph representation.
// Implement `Pipe` to build StreamOp nodes when left is Stream.
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p fmpl-core test_stream_map_filter_flatmap`
Expected: PASS

**Step 5: Commit**

```bash
git add fmpl-core/src/ast.rs fmpl-core/src/parser.rs fmpl-core/src/compiler.rs fmpl-core/src/vm.rs
git commit -m "feat: stream primitives and special forms"
```

---

### Task 3: Add grammar parse node for streams (`parse(g.rule)`)

**Files:**
- Modify: `fmpl-core/src/parser.rs`
- Modify: `fmpl-core/src/compiler.rs`
- Modify: `fmpl-core/src/vm.rs`
- Test: `fmpl-core/src/vm.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_stream_parse_operator() {
    let mut vm = Vm::new();
    let result = eval(&mut vm, r#"
        let (g = grammar { digit = [0-9] })
        let (s = stream { "5" })
        s |> parse(g.digit)
    "#).unwrap();
    assert!(matches!(result, Value::Stream(_)));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p fmpl-core test_stream_parse_operator`
Expected: FAIL with unknown `parse` op.

**Step 3: Write minimal implementation**

```rust
// parser.rs
// parse(...) is treated as a stream operator when left is a Stream

// vm.rs
// parse node uses existing grammar application; failures drop events
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p fmpl-core test_stream_parse_operator`
Expected: PASS

**Step 5: Commit**

```bash
git add fmpl-core/src/parser.rs fmpl-core/src/compiler.rs fmpl-core/src/vm.rs
git commit -m "feat: stream parse operator"
```

---

### Task 4: Add SnapshotEnvelope + continuation store in fmpl-web

**Files:**
- Create: `fmpl-web/src/continuations.rs`
- Modify: `fmpl-web/src/main.rs`
- Modify: `fmpl-web/Cargo.toml`
- Test: `fmpl-web/tests/continuations.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_store_and_load_continuation() {
    let store = ContinuationStore::new(temp_dir());
    let token = store.save("session", SnapshotEnvelope::dummy()).unwrap();
    let loaded = store.load("session", &token).unwrap();
    assert_eq!(loaded.schema_version, 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p fmpl-web test_store_and_load_continuation`
Expected: FAIL with missing `ContinuationStore`.

**Step 3: Write minimal implementation**

```rust
pub struct SnapshotEnvelope { /* versioned fields + payload */ }

pub struct ContinuationStore { /* fjall handle */ }

impl ContinuationStore {
    pub fn save(&self, session: &str, env: SnapshotEnvelope) -> Result<String> { /* random token */ }
    pub fn load(&self, session: &str, token: &str) -> Result<SnapshotEnvelope> { /* lookup */ }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p fmpl-web test_store_and_load_continuation`
Expected: PASS

**Step 5: Commit**

```bash
git add fmpl-web/src/continuations.rs fmpl-web/src/main.rs fmpl-web/tests/continuations.rs fmpl-web/Cargo.toml
git commit -m "feat: continuation store with snapshot envelope"
```

---

### Task 5: Seed loader + image store bootstrap

**Files:**
- Create: `fmpl-web/src/image_store.rs`
- Modify: `fmpl-web/src/main.rs`
- Create: `fmpl-web/seed/seed.fmpl`
- Test: `fmpl-web/tests/seed_loader.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_seed_loader_bootstraps_image() {
    let store = ImageStore::new(temp_dir());
    store.bootstrap_if_empty("fmpl-web/seed/seed.fmpl").unwrap();
    assert!(store.has_object("storylet"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p fmpl-web test_seed_loader_bootstraps_image`
Expected: FAIL with missing `ImageStore`.

**Step 3: Write minimal implementation**

```rust
pub struct ImageStore { /* fjall handle */ }

impl ImageStore {
    pub fn bootstrap_if_empty(&self, path: &str) -> Result<()> { /* parse FMPL, store objects */ }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p fmpl-web test_seed_loader_bootstraps_image`
Expected: PASS

**Step 5: Commit**

```bash
git add fmpl-web/src/image_store.rs fmpl-web/src/main.rs fmpl-web/seed/seed.fmpl fmpl-web/tests/seed_loader.rs
git commit -m "feat: image store bootstrap"
```

---

### Task 6: Storylet HTTP routes

**Files:**
- Modify: `fmpl-web/src/main.rs`
- Create: `fmpl-web/src/storylet.rs`
- Test: `fmpl-web/tests/storylet_http.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn test_play_route_redirects() {
    let app = build_app();
    let res = get("/play", app).await;
    assert_eq!(res.status(), 302);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p fmpl-web test_play_route_redirects`
Expected: FAIL with 404.

**Step 3: Write minimal implementation**

```rust
// main.rs
.route("/play", get(play_start))
.route("/play/:token", get(play_token))

// storylet.rs
pub async fn play_start(...) -> impl IntoResponse { /* create snapshot + redirect */ }
pub async fn play_token(...) -> impl IntoResponse { /* load snapshot + render */ }
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p fmpl-web test_play_route_redirects`
Expected: PASS

**Step 5: Commit**

```bash
git add fmpl-web/src/main.rs fmpl-web/src/storylet.rs fmpl-web/tests/storylet_http.rs
git commit -m "feat: storylet play routes"
```

---

Plan complete and saved to `docs/plans/2025-12-24-storylet-spike-implementation-plan.md`. Two execution options:

1. Subagent-Driven (this session) - I dispatch fresh subagent per task, review between tasks, fast iteration
2. Parallel Session (separate) - Open new session with executing-plans, batch execution with checkpoints

Which approach?
