# Image Model

Objects live in a persistent image. The image is the source of truth, not source files.

## Interaction Model

The inspector is the primary interface. Objects are created and modified live:

```fmpl
fmpl> let treasury = spawn %{}
fmpl> treasury.balance = 10000
fmpl> treasury.view_balance = \() -> self.balance
```

The `object { }` block is sugar for creating and populating in one expression:

```fmpl
object treasury {
  balance: 10000
  view_balance(): self.balance
}
```

Both produce the same object in the image.

## Inspector

Shows an object's complete state (to its owner / system):

```
treasury (#42)
  parent: <none>
  slots:
    balance: 10000                    [private]
    view_balance: () -> _             [private]
  facets:
    auditor: [view_balance]
      discovered laws:
        view_balance is idempotent
```

Through a facet, only faceted slots are visible:

```
<facet:auditor of treasury #42>
  view_balance: () -> _
```

No peeking behind the facet. The sealed view is all you get.

## Source Recovery

Source is stored alongside bytecode in the image store. Decompiler fallback for missing source (LambdaMOO-style).

- `object.method_source(:name)` returns original FMPL source
- `lambda.source()` returns source snippet when available
- Bytecode decompiler for normalized output when source is unavailable

## Persistence

Transparent via Fjall. No explicit save/load:

```fmpl
@merchant.mood = "happy"  -- automatically persisted
```

- Changes tracked in current transaction
- Commit at end of turn/tick
- Crash recovery from Fjall journal

### Storage Layout (Fjall)

```
Partition: objects
  Key: obj:{id}       Value: Object (rkyv serialized)

Partition: code
  Key: code:{id}      Value: CompiledCode + source blob

Partition: sessions
  Key: session:{id}   Value: principal metadata, active facets
```

## Target Files

| File | What to change |
|------|---------------|
| `fmpl-web/src/image_store.rs` | Expand bootstrap, add object/code partitions |
| `fmpl-core/src/object.rs:30` | Add source blob to Method struct |
| `fmpl-core/src/vm.rs` | Reflection builtins (method_source, etc.) |

## Related

- [persistence.md](../persistence.md) — Fjall storage details
- Research: [coldmud](../../docs/research/2026-02-25-coldmud-architecture.md) (source recovery model)
