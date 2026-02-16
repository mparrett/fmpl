# spawn and bcom

Object creation and functional state updates.

## spawn (Implemented)

Creates an instance from a parent object. Located at `vm.rs:625`.

```fmpl
let obj = spawn parent_object(args)
```

### Behavior

1. Create new object with `parent` set to the parent object
2. Look up `init` method on new object (follows prototype chain)
3. If found and arity matches, call `init` with `self` bound to new object
4. If `init` missing or arity mismatch, spawn succeeds (graceful degradation)
5. Return new `ObjectId`

### Example

```fmpl
object counter {
  init(start): { self.count = start }
  increment(): { self.count = self.count + 1 }
  get(): self.count

  .#facets
  public: [increment, get]
}

let c = spawn counter(10)
c.as(:public).get()            -- 10
c.as(:public).increment()
c.as(:public).get()            -- 11
```

### Current Implementation

```rust
// vm.rs:625
Instruction::Spawn { object, args } => {
    let parent_id = /* resolve object */;
    let new_id = self.objects.lock().unwrap().create(Some(parent_id));
    // Look up and call init if present
    // ...
    frame.set_current(Value::Object(new_id));
}
```

## bcom (Not Implemented)

Functional state updates inspired by Goblins. `bcom` replaces the current object's behavior atomically.

### Design

```fmpl
object ^cell (bcom, val) {
  get(): val
  set(new_val): bcom(^cell(bcom, new_val))

  .#facets
  public: [get, set]
}
```

`^cell` is a constructor function, not an object. `bcom` is a callback that replaces the object's state. Each call to `set` creates a new `^cell` with the new value and atomically becomes it.

### Semantics

- `bcom(new_behavior)` replaces the current object's methods and properties **at end of turn**
- If the turn errors, the `bcom` is rolled back
- External references (ObjectId) remain stable -- the object's identity doesn't change, only its behavior
- Multiple `bcom` calls in one turn: last one wins

### Implementation Plan

**`object.rs`** — Add pending bcom state:
```rust
pub struct Object {
    // ...existing fields...
    pub pending_bcom: Option<BcomState>,
}

pub struct BcomState {
    pub properties: HashMap<SmolStr, Value>,
    pub methods: HashMap<SmolStr, Method>,
    pub facets: HashMap<SmolStr, Facet>,
}
```

**`vm.rs`** — `bcom` builtin:
```rust
// When bcom is called:
// 1. Evaluate the constructor to get new properties/methods/facets
// 2. Store as pending_bcom on the current object
// 3. At end of turn (commit), apply pending_bcom
// 4. On error (rollback), discard pending_bcom
```

**`compiler.rs`** — Recognize `bcom` as a special form in constructor contexts. The `^name` syntax marks a constructor function that receives `bcom` as first argument.

### Transactions

Turns are atomic. All `bcom` calls in a turn are either committed or rolled back together:

```fmpl
$ cell.set(42)      -- bcom queued
error("Oops!")      -- turn rolls back, cell still has old value
```

## Target Files

| File | Change |
|------|--------|
| `object.rs:30` | Add `pending_bcom: Option<BcomState>` to Object |
| `vm.rs` | Implement `bcom` builtin, turn commit/rollback |
| `compiler.rs` | Parse `^name` constructor syntax, pass `bcom` callback |
| `parser.rs` | Recognize `^name` as constructor form |

## Related

- [facets](facets.md) — bcom must preserve facet definitions
- [multi-principal](multi-principal.md) — Turn atomicity ties to VAT model
