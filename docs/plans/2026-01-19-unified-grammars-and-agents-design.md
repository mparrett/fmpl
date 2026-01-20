# Unified Grammars and Agentic Interactions

## Overview

This document proposes unifying FMPL's pattern matching, grammar rules, and tree transformations into a single primitive: **grammars over polymorphic streams**. This same abstraction naturally supports agentic workflows with human-in-the-loop patterns, context engineering, and durable pause/resume.

The goal is to reduce grammar surface area while gaining expressive power for building reliable AI agents.

---

## Core Insight

Parsing, pattern matching, and tree transformation are the same operation: applying rules to a stream of values. The differences are only in what the stream contains:

| Stream Type | Operation | Example |
|-------------|-----------|---------|
| Characters | Parsing | `"hello world" @ parser.sentence` |
| Tokens | Token parsing | `token_stream @ parser.expr` |
| List elements | List processing | `[1, 2, 3] @ summer.sum` |
| Single value | Pattern matching | `obj @ { %{type: t} => handle(t) }` |
| Messages | Agent control flow | `user_input @ agent.turn` |

---

## The `@` Operator

The `@` operator applies a grammar (or anonymous rule block) to a value:

```fmpl
-- Named grammar, named rule
input @ grammar.rule

-- Anonymous grammar (inline patterns)
input @ {
  %{type: "move", dir: d} => move(d)
  [head | tail] => process(head, tail)
  _ => default()
}
```

### Input Coercion

The `@` operator coerces its left operand to a stream based on type:

| Input Type | Stream Behavior |
|------------|-----------------|
| String | Character stream |
| List | Element stream |
| Map/Object | Single-element stream |
| Stream | Pass through |

For explicit control, use stream constructors:

```fmpl
chars("hello")      -- character stream
items([1, 2, 3])    -- element stream
once(obj)           -- single-element stream
```

### Eliminating `match`

The `match` keyword becomes unnecessary:

```fmpl
-- Before: separate match construct
match input {
  %{type: "move", dir: d} => move(d)
  _ => default()
}

-- After: just grammar application
input @ {
  %{type: "move", dir: d} => move(d)
  _ => default()
}
```

---

## Unified Pattern Language

One pattern syntax works everywhere: grammar rules, anonymous blocks, `let` destructuring.

### Pattern Forms

| Pattern | Matches | Binds |
|---------|---------|-------|
| `Tag(p1, p2)` | Tagged node with children | Child bindings |
| `%{k1: p1, k2: p2}` | Map with keys k1, k2 | Value bindings |
| `[p1, p2, p3]` | List with exactly 3 elements | Element bindings |
| `[h \| t]` | Non-empty list | `h` = head, `t` = tail |
| `[p*]` | List where every element matches p | List of matches |
| `_` | Anything | Nothing |
| `name:p` | Same as p | Binds result to `name` |
| `p when expr` | p, if guard is truthy | Same as p |
| `"literal"` | Exact string/symbol | Nothing |
| `123` | Exact number | Nothing |

### PEG Operators

Standard PEG operators work on any pattern:

| Operator | Meaning |
|----------|---------|
| `p1 p2` | Sequence |
| `p1 \| p2` | Ordered choice |
| `p*` | Zero or more |
| `p+` | One or more |
| `p?` | Optional |
| `&p` | Positive lookahead |
| `!p` | Negative lookahead |
| `=> expr` | Semantic action |

---

## Semantic Predicates for Context Engineering

Semantic predicates (`&{ expr }`) compute values mid-match. This is where context engineering lives:

```fmpl
grammar Agent {
  turn = message:m &{ build_context(m) }:ctx => {
    <- llm.complete(m, ctx)
  } @ output_handler

  build_context(m) = {
    let history = conversation_history(10)
    let retrieved = <- vector_search(m, limit: 5)
    let tools = available_tools_for(m)

    history
      |> inject_retrieved(retrieved)
      |> add_tool_descriptions(tools)
      |> truncate_to_budget(token_limit)
  }
}
```

### Predicates as Gates

```fmpl
output_handler =
  | &{ needs_approval(action) } action:a => pause_for_human(a)
  | &{ too_many_errors(ctx) } => escalate()
  | %{tool: t, args: a} => <- execute(t, a) @ output_handler
  | %{done: result} => yield(result)
```

---

## Streaming Grammars

Grammars operate on two stream models:

| Model | Description | Use Case |
|-------|-------------|----------|
| **Pull (lazy)** | Grammar requests next item | File parsing, batch processing |
| **Push (reactive)** | Items arrive asynchronously | Network protocols, LLM output |

### Cut Operators for Streaming

For streaming grammars, cut operators control backtracking and buffer release:

| Operator | Effect |
|----------|--------|
| `!` | Local cut - release buffer, but caller can still backtrack |
| `!!` | Full cut - release buffer and commit caller |

```fmpl
grammar LLMOutputParser <: Stream {
  output =
    | "```" lang:word !! code_block* "```" => emit_code(lang, code_block)
    | "TOOL:" name:word "(" args:json ")" ! => dispatch_tool(name, args)
    | chunk+ => stream_to_user(chunks)
}

-- Usage: push LLM tokens through parser
llm_stream |> LLMOutputParser.output |> user_output
```

---

## Agentic Patterns

### Agent as Grammar

An agent's control flow is a grammar over message streams:

```fmpl
grammar TaskAgent <: Agent {
  turn =
    | message:m &{ needs_approval(m) } => pause_for_human(m)
    | message:m &{ should_escalate(m) } => escalate(m)
    | message:m => process(m) @ result_handler

  result_handler =
    | %{error: e} &{ retryable(e) } => compact_error(e); <turn>
    | %{error: e} => escalate(e)
    | %{tool: t, args: a} => <- execute(t, a) @ result_handler
    | %{done: result} => yield(result)
}
```

### Human-in-the-Loop

Human interaction is just another async tool call:

```fmpl
-- require_approval pattern
execute_with_approval(tool, args) = {
  let decision = <- human.approve(%{tool: tool, args: args})
  decision @ {
    %{approved: true} => <- tools[tool](args)
    %{denied: true, reason: r} => %{error: "denied", reason: r}
  }
}

-- human_as_tool pattern
ask_human(question) = <- human.ask(question)
```

### Durable Pause/Resume

The live image plus continuations provide durable suspension:

```fmpl
object ^approval_request (bcom, tool, args, continuation) {
  .#public
  approve(): continuation.resume(<- tools[tool](args))
  deny(reason): continuation.resume(%{denied: true, reason: reason})

  .#facets
  reviewer: [approve, deny]
}

pause_for_human(action) = {
  let request = spawn ^approval_request(
    action.tool,
    action.args,
    current_continuation()
  )
  <- request.decision  -- suspends here, resumes when human responds
}
```

The continuation is an object in the image. It persists across restarts. When a human clicks "approve" hours or days later, the agent resumes exactly where it left off.

### Composing Agents

Grammar inheritance builds complex agents from focused pieces:

```fmpl
grammar BaseAgent <: Agent {
  context_for(m) = %{history: last(10), user: current_user()}
  error_handler = %{error: e} => log(e); escalate()
}

grammar RAGAgent <: BaseAgent {
  context_for(m) = {
    let base = <super.context_for>(m)
    base | %{retrieved: <- search(m)}
  }
}

grammar ToolAgent <: RAGAgent {
  context_for(m) = {
    let base = <super.context_for>(m)
    base | %{tools: tools_for_intent(m)}
  }

  result_handler =
    | %{tool: t, args: a} &{ authorized(t) } => <- tools[t](a) @ result_handler
    | <super.error_handler>
    | %{done: r} => r
}
```

---

## Mapping to 12-Factor Agents

| Factor | FMPL Implementation |
|--------|---------------------|
| 1. NL → Tool Calls | Grammar parses intent → `%{tool: t, args: a}` |
| 2. Own Your Prompts | Grammars *are* prompts - you write the patterns |
| 3. Own Your Context Window | Predicates compute context: `&{ build_context(m) }` |
| 4. Tools Are Structured Outputs | `=> %{tool: name, args: args}` |
| 5. Unify Execution + Business State | Live image - everything is objects |
| 6. Launch/Pause/Resume | Continuations serialize to image |
| 7. Contact Humans with Tool Calls | `<- human.ask()` / `<- human.approve()` |
| 8. Own Your Control Flow | Grammar *is* control flow |
| 9. Compact Errors into Context | Predicates filter/summarize: `&{ compact_errors(ctx) }` |
| 10. Small Focused Agents | Grammar inheritance composes small grammars |
| 11. Trigger from Anywhere | Streams from HTTP, WS, Slack, email, etc. |
| 12. Stateless Reducer | `bcom` pattern for functional state updates |

---

## Meta-Object Protocol (Open)

For live editing of agents, we need reflection primitives. Direction: MOP as methods on objects, possibly with syntactic sugar.

### Candidate Primitives

```fmpl
-- Introspection
obj.slots()                    -- list slot names
obj.slot(name)                 -- get slot value
obj.slot_visibility(name)      -- :private/:public/:protected
obj.source(method_name)        -- recover source text

-- Mutation (owner/capability gated)
obj.set_slot(name, value)      -- modify slot
obj.remove_slot(name)          -- delete slot
obj.recompile(name, source)    -- live update method
```

### Possible Syntax

```fmpl
-- Raw slot access (bypasses facet)
obj::name                      -- get slot directly
obj::name = value              -- set slot directly
```

This needs more exploration. Key questions:
- Mirror-style (separate meta object) vs direct methods?
- How do facets interact with MOP?
- How does ownership gate mutation?

---

## EBNF Changes

### Remove

- `<MATCH>` keyword and `<match_block>` productions

### Modify

The `@` operator becomes a primary expression form:

```ebnf
<exp> ::= ...
          <exp> '@' <qualified_tag> '.' <TAG>      -- named grammar.rule
          <exp> '@' <qualified_tag>                -- named grammar, default rule
          <exp> '@' '{' <rule_cases> '}'           -- anonymous grammar

<rule_cases> ::= <rule_case>
                 <rule_cases> <rule_case>

<rule_case> ::= <pattern> '=>' <exp> <optsemi>
                <pattern> <WHEN> <exp> '=>' <exp> <optsemi>
```

### Pattern as Rule Primary

Patterns already usable in grammar rules (from tree-grammars design):

```ebnf
<rule_primary> ::= ...
                   <pattern>
```

---

## Migration Path

1. **Keep `match` as sugar** initially - desugar to `@ { }` in parser
2. **Implement polymorphic streams** - string, list, single-value
3. **Unify pattern compilation** - one codepath for patterns everywhere
4. **Add streaming support** - cut operators, push model
5. **Build agent examples** - validate the model works for real workflows
6. **Design MOP** - based on what live editing actually needs

---

## References

- [Tree Grammars Design](2025-12-28-tree-grammars-design.md) - structural pattern matching in grammars
- [FMPL Revival Design](2025-12-19-fmpl-revival-design.md) - overall language vision
- [12 Factor Agents](https://www.humanlayer.dev/blog/12-factor-agents) - patterns for reliable LLM agents
- [HumanLayer](https://github.com/humanlayer/humanlayer) - human-in-the-loop SDK
- [OMeta](http://www.tinlizzie.org/~awarth/papers/dls07.pdf) - grammar inheritance semantics
- [Maru PEG](~/development/maru/core/parser.l) - polymorphic streams over strings and lists
