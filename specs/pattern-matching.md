# Pattern Matching

Pattern matching with the `@` operator.

**Location**: [fmpl-core/src/vm.rs](../fmpl-core/src/vm.rs)

**Key files**:
- `ast.rs:88-111` — Pattern enum (value patterns)
- `ast.rs:113-119` — MatchCase with guard
- `parser.rs:250-291` — `@` operator parsing
- `parser.rs:1043-1061` — Match case parsing with `when` guards
- `parser.rs:1125-1207` — Pattern parsing
- `compiler.rs:503-556` — Match expression compilation
- `vm.rs:579-612` — Pattern execution (ExtractMapKey, ExtractListIndex)

---

## Overview

The `@` operator provides pattern matching on values and grammar application:

- **Grammar application** — Parse input with grammar rules
- **Value matching** — Destructure and match values
- **Control flow** — Branch based on patterns

---

## Grammar Application

Apply a grammar rule to input:

```fmpl
"take sword" @ mud::parser.command
-- Returns: %{action: :take, target: "sword"}

file_bytes @ png::header.magic
-- Returns: matched value or fails
```

---

## Value Matching

Match and destructure values:

```fmpl
result @ {
  %{tool: t, args: a} => execute_tool(t, a)
  %{text: t}          => emit_text(t)
  %{error: e}         => handle_error(e)
  _                   => default_case()
}
```

### List Patterns

```fmpl
list @ {
  []              => "empty"
  [x]             => "single: " + x
  [head | tail]   => "head: " + head + ", rest: " + tail
  [a, b, c]       => "three elements"
}
```

### Map Patterns

```fmpl
data @ {
  %{type: "user", name: n, age: a} => "User " + n + " is " + a
  %{type: "bot", id: i}            => "Bot #" + i
  %{}                              => "empty map"
}
```

### Literal Patterns

```fmpl
value @ {
  42        => "the answer"
  :ok       => "success"
  "hello"   => "greeting"
  true      => "affirmative"
  null      => "nothing"
}
```

---

## Guards

Add conditions to patterns using `when` keyword (`parser.rs:1046`):

```fmpl
value @ {
  n when n > 0  => "positive"
  n when n < 0  => "negative"
  _             => "zero"
}
```

Combined with destructuring:

```fmpl
result @ {
  %{status: s, data: d} when s == 200  => process(d)
  %{status: s, error: e} when s >= 400 => handle_error(e)
}
```

Note: The `&{ condition }` syntax is for grammar predicates only (`grammar/parser.rs:248`).

---

## Binding in Patterns

Bind matched values to names using `as` (`parser.rs:1199-1204`):

```fmpl
-- Bind entire match
input @ {
  %{nested: inner} as whole => use_both(whole, inner)
}

-- Bind in lists
list @ {
  [first, second | rest] as all => ...
}
```

---

## Nested Patterns

Patterns can nest arbitrarily:

```fmpl
data @ {
  %{
    user: %{name: n, prefs: %{theme: t}},
    items: [first | _]
  } => "User " + n + " with theme " + t + " has " + first
}
```

---

## With Async Streams

Pattern match on async results:

```fmpl
<- http.get(url) @ {
  %{status: 200, body: b} => parse_json(b)
  %{status: 404}          => not_found()
  %{error: e}             => handle_error(e)
}
```

---

## In Grammar Rules

Patterns appear in grammar semantic actions:

```fmpl
grammar ToolAgent <: base::tree {
  turn = message:m => <- llm(m) |> tool_output

  tool_output =
    | %{tool: t, args: a} => execute(t, a)  -- map pattern
    | [head | tail]       => process(head)   -- list pattern
    | :done               => finish()        -- symbol pattern
}
```

---

## Exhaustiveness

Patterns are tried in order. Use `_` for catch-all:

```fmpl
value @ {
  %{type: "a"} => ...
  %{type: "b"} => ...
  _            => default()  -- catches everything else
}
```

Without `_`, unmatched values cause runtime errors.

---

## Pattern Types Summary

| Pattern | Matches | Example | Status |
|---------|---------|---------|--------|
| `_` | Anything | `_ => default()` | Implemented |
| Literal | Exact value | `42 => ...` | Parsed, not compiled |
| `:symbol` | Symbol | `:ok => ...` | Parsed, not compiled |
| `name` | Bind to name | `x => use(x)` | Implemented |
| `%{k: v}` | Map with key | `%{id: i} => ...` | Let-binding only |
| `[...]` | List | `[a, b] => ...` | Let-binding only |
| `[h \| t]` | Head/tail | `[first \| rest] => ...` | Parsed only |
| `p when g` | Pattern with guard | `n when n > 0 => ...` | Implemented |
| `p as name` | Bind match to name | `%{...} as whole => ...` | Parsed, not compiled |

Note: Match expressions (`compiler.rs:526`) only support `Var` and `Wildcard` patterns.
Let destructuring (`compiler.rs:729`) supports `Map` and fixed-length `List` patterns.

---

## Implementation Status

| Feature | Parsing | Compilation | Execution |
|---------|---------|-------------|-----------|
| Var patterns | ✓ | ✓ | ✓ |
| Wildcard patterns | ✓ | ✓ | ✓ |
| Guards (`when`) | ✓ | ✓ | ✓ |
| Let map destructure | ✓ | ✓ | ✓ |
| Let list destructure | ✓ | ✓ (fixed-length) | ✓ |
| Literal patterns | ✓ | — | — |
| Symbol patterns | ✓ | — | — |
| Head/tail lists | ✓ | — | — |
| As-patterns | ✓ | — | — |
| Constructor patterns | ✓ | — | — |

---

## Related Specs

- [grammar-system.md](./grammar-system.md) — Grammar patterns (separate system)
- [vm.md](./vm.md) — VM execution
