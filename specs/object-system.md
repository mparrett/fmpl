# Object System

Live, image-based prototype objects with capability security.

## Overview

Objects live in a persistent image (Self/Smalltalk/LambdaMOO tradition). The image is the source of truth -- `object { }` blocks are bootstrapping sugar. The inspector is the primary interface.

- **Prototype inheritance** — Delegate to parent via chain lookup (`object.rs:102`)
- **Default private** — All slots private unless exposed through a facet
- **Facets** — Lightweight, sealed views on parent objects (`object.rs:23`)
- **Multi-principal** — Humans and LLM agents share the image via faceted capabilities
- **Transparent persistence** — Fjall-backed, no explicit save/load
- **spawn / bcom** — Object creation and functional state updates

## Subsections

| Spec | Scope | Key Files |
|------|-------|-----------|
| [image-model](object-system/image-model.md) | Image interaction, inspector, persistence | `object.rs`, `persistence.md` |
| [facets](object-system/facets.md) | Sealed views, syntax levels, composition, laws | `object.rs:23`, `parser.rs:188`, `vm.rs:632` |
| [visibility](object-system/visibility.md) | Default private, `.#public` sugar, enforcement | `lexer.rs:93-99`, `parser.rs:110` |
| [multi-principal](object-system/multi-principal.md) | Users, agents, VATs, tuple space coordination | `vm.rs:42`, `tuplespace/facet.rs` |
| [spawn-bcom](object-system/spawn-bcom.md) | Object creation, become pattern, transactions | `vm.rs:625` |

## Key Types

```rust
// object.rs:12
pub type ObjectId = u64;

// object.rs:23
pub struct Facet {
    pub members: Vec<SmolStr>,
    pub terminal: bool,
}

// object.rs:30
pub struct Object {
    pub id: ObjectId,
    pub parent: Option<ObjectId>,
    pub properties: HashMap<SmolStr, Value>,
    pub methods: HashMap<SmolStr, Method>,
    pub facets: HashMap<SmolStr, Facet>,
}
```

## Implementation Status

- [x] Prototype chain — `object.rs:102`
- [x] spawn with init — `vm.rs:625`
- [x] Facet parsing — `parser.rs:188`
- [x] `.as(:facet)` — `compiler.rs:1027`, `vm.rs:632`
- [x] Tuple space facets — `tuplespace/facet.rs:57`
- [ ] **Default private enforcement** → [visibility](object-system/visibility.md)
- [ ] **Facet sealing** (return sealed view, not raw ObjectId) → [facets](object-system/facets.md)
- [ ] **`.#public` desugaring** → [visibility](object-system/visibility.md)
- [ ] **Facet arity/unification** (level 2-3 syntax) → [facets](object-system/facets.md)
- [ ] **bcom** → [spawn-bcom](object-system/spawn-bcom.md)
- [ ] **User context propagation** → [multi-principal](object-system/multi-principal.md)
- [ ] **Yield injection** → [multi-principal](object-system/multi-principal.md)
- [ ] **Multi-VAT** → [multi-principal](object-system/multi-principal.md)

## Related

- [vm.md](./vm.md) — Instruction set, magical variables
- [tuplespace.md](./tuplespace.md) — Tuple space coordination
- [grammar-system.md](./grammar-system.md) — Command parsing dispatch
- [persistence.md](./persistence.md) — Fjall-backed storage
- Research: [type-inference](../docs/research/2026-02-25-type-inference-duck-typed-systems.md), [category-theory](../docs/research/2026-02-25-category-theoretic-type-system.md), [multi-user-synthesis](../docs/research/2026-02-25-multi-user-architecture-synthesis.md), [coldmud](../docs/research/2026-02-25-coldmud-architecture.md)
