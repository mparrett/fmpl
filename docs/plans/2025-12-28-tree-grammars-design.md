# Tree Grammars and Structural Pattern Matching

## Overview

Extend FMPL's OMeta-style grammars to match not just strings/tokens but arbitrary data structures: objects, lists, maps, and AST nodes. This enables grammars to serve as tree transformers, not just parsers.

## Motivation

Currently grammars parse character streams into trees. But many language tasks require tree-to-tree transformations:

- **Optimization passes**: constant folding, dead code elimination
- **Desugaring**: expand syntax sugar into core forms
- **Code generation**: emit target language from AST
- **Validation**: check structural invariants

With tree grammars, the same PEG rule syntax handles all these phases.

## Design

### Unified Pattern Language

Grammar rules use the same pattern syntax as `match` expressions, extended with PEG operators. This provides a single pattern language throughout FMPL.

#### Pattern Forms

| Pattern | Matches | Binds |
|---------|---------|-------|
| `Tag(p1, p2, ...)` | Tagged node with children matching p1, p2, ... | Child bindings |
| `%{k1: p1, k2: p2}` | Object/map with keys k1, k2 matching p1, p2 | Value bindings |
| `[p1, p2, p3]` | List with exactly 3 elements | Element bindings |
| `[h \| t]` | Non-empty list | `h` = head, `t` = tail |
| `[p*]` | List where every element matches p | List of matches |
| `[p+]` | Non-empty list where every element matches p | List of matches |
| `[p?]` | Empty list or single-element list matching p | Optional match |
| `_` | Anything | Nothing |
| `name:p` | Same as p | Binds result to `name` |
| `p when expr` | p, if guard expr is truthy | Same as p |
| `"literal"` | Exact string/symbol | Nothing |
| `123` / `3.14` | Exact number | Nothing |

#### PEG Operators on Tree Patterns

The standard PEG operators work on tree patterns:

| Operator | Meaning |
|----------|---------|
| `p1 p2` | Sequence (for string input) |
| `p1 \| p2` | Ordered choice |
| `p*` | Zero or more |
| `p+` | One or more |
| `p?` | Optional |
| `&p` | Positive lookahead |
| `!p` | Negative lookahead |
| `<rule>` | Invoke another rule |
| `=> expr` | Semantic action (produce result) |

### Rule Invocation

Grammars are first-class objects. Invoke any rule directly on any data:

```fmpl
let ast = Parser.program(sourceCode)      // string -> tree
let optimized = Optimizer.expr(ast)       // tree -> tree
let output = CodeGen.emit(optimized)      // tree -> string
```

The rule determines what input types it accepts based on its patterns.

### Examples

#### Constant Folding

```fmpl
grammar ConstFold {
  expr = Add(Num(a), Num(b)) => Num(a + b)
       | Sub(Num(a), Num(b)) => Num(a - b)
       | Mul(Num(a), Num(b)) => Num(a * b)
       | Div(Num(a), Num(b)) when b != 0 => Num(a / b)
       | Add(l:expr, r:expr) => Add(l, r)
       | Sub(l:expr, r:expr) => Sub(l, r)
       | Mul(l:expr, r:expr) => Mul(l, r)
       | Div(l:expr, r:expr) => Div(l, r)
       | x => x
}
```

#### Algebraic Simplification

```fmpl
grammar Simplify {
  expr = Add(Num(0), e:expr) => e
       | Add(e:expr, Num(0)) => e
       | Sub(e:expr, Num(0)) => e
       | Mul(Num(0), _) => Num(0)
       | Mul(_, Num(0)) => Num(0)
       | Mul(Num(1), e:expr) => e
       | Mul(e:expr, Num(1)) => e
       | Div(e:expr, Num(1)) => e
       | Add(l:expr, r:expr) => Add(l, r)
       | Mul(l:expr, r:expr) => Mul(l, r)
       | x => x
}
```

#### List Processing

```fmpl
grammar ListOps {
  // Sum all numbers in a list
  sum = [] => 0
      | [Num(n) | t:sum] => n + t

  // Filter to keep only positive
  positives = [] => []
            | [Num(n) | t:positives] when n > 0 => [Num(n) | t]
            | [_ | t:positives] => t

  // Map: double all numbers
  double = [] => []
         | [Num(n) | t:double] => [Num(n * 2) | t]
         | [x | t:double] => [x | t]

  // Match list where all elements are numbers
  allNums = [Num*]

  // Extract first two elements
  first2 = [a, b | rest] => %{first: a, second: b, rest: rest}
}
```

#### Object Transformation

```fmpl
grammar Normalize {
  // Normalize a person record
  person = %{name: n, age: a} => %{
    name: <normalizeName>(n),
    age: a,
    adult: a >= 18
  }

  normalizeName = s:String => s.trim().toLowerCase()
}
```

#### Statement Transformation

```fmpl
grammar Desugar {
  stmt = For(init:expr, cond:expr, update:expr, body:stmts) =>
           Block([init, While(cond, Block([body, update]))])
       | Unless(cond:expr, body:stmts) =>
           If(Not(cond), body, [])
       | s => s

  stmts = [stmt*]

  expr = And(l:expr, r:expr) => If(l, r, False())
       | Or(l:expr, r:expr) => If(l, True(), r)
       | e => e
}
```

### Grammar Inheritance

Tree grammars can inherit from other grammars:

```fmpl
grammar BaseTransform {
  expr = e => e
  stmt = s => s
}

grammar MyOptimizer <: BaseTransform {
  // Override just the cases we care about
  expr = Add(Num(0), e:expr) => e
       | e => <super.expr>(e)
}
```

### Mixed String/Tree Grammars

A single grammar can have rules for both string parsing and tree transformation:

```fmpl
grammar Compiler {
  // String -> Tree (parsing)
  program = stmt*
  stmt = "let" ident "=" expr ";" => Let($2, $4)
       | "return" expr ";" => Return($2)
  expr = num | ident | "(" expr ")"

  // Tree -> Tree (optimization)
  optimize = Let(n, e:optimize) => Let(n, e)
           | Add(Num(0), e:optimize) => e
           | e => e

  // Tree -> String (code generation)
  emit = Let(n, e) => "var " + n + " = " + <emit>(e) + ";"
       | Return(e) => "return " + <emit>(e) + ";"
       | Num(n) => tostr(n)
       | Var(n) => n
}

// Full pipeline
let ast = Compiler.program(source)
let opt = Compiler.optimize(ast)
let js = Compiler.emit(opt)
```

## EBNF Changes

Extend `<pattern>` to be usable in `<rule_primary>`:

```ebnf
<rule_primary> ::= <TAG>
                   <STRING>
                   <CHAR_RANGE>
                   '.'
                   '(' <rule_body> ')'
                   '[' <rule_body> ']'
                   '<' <TAG> '>'
                   '<' <TAG> <parmlist> '>'
                   ':' <TAG>
                   '{' <exp> '}'
                   <ARROW> <exp>
                   -- New: structural patterns
                   <pattern>
```

The existing `<pattern>` nonterminal already covers:
- `Tag(p1, p2, ...)` - tagged nodes
- `%{k: p, ...}` - map patterns
- `[p1, p2 | rest]` - list patterns
- `_` - wildcard
- Literals and bindings

## Implementation Notes

### Pattern Compilation

Tree patterns compile to match functions that:
1. Check the structural shape (tag, keys, length)
2. Recursively match sub-patterns
3. Collect bindings into an environment
4. Return match success/failure plus bindings

### Backtracking

PEG ordered choice (`|`) backtracks on failure. For tree patterns this means:
- Try first alternative
- On failure, restore input position (for streams) or retry with next alternative (for trees)
- Tree matching is typically non-consuming, so backtracking is just "try next pattern"

### Performance Considerations

- Memoization (Packrat parsing) works for tree grammars too
- Common pattern prefixes can be factored for efficiency
- Type tags enable fast dispatch before full pattern matching

## Streaming and Composition

### Stream Types

Grammars operate on two stream models:

| Model | Description | Use Case |
|-------|-------------|----------|
| **Pull (lazy)** | Grammar requests next item from iterator | File parsing, lazy tree traversal |
| **Push (reactive)** | Items arrive asynchronously, grammar reacts | Network protocols, event streams |

Both models use the same rule syntax. The difference is in how the grammar is invoked.

### Pull Streams

```fmpl
// Lazy iterator of tokens
let tokens = Lexer.tokenize(sourceStream)  // returns lazy stream
let ast = Parser.program(tokens)           // consumes incrementally
```

### Push Streams

```fmpl
// Reactive stream processing
let parser = Parser.program.stream()       // returns a stream processor
networkSocket |> parser |> handler         // push items through
```

### Cut Operators (Backtracking Control)

For streaming grammars, unbounded backtracking is impossible. Two cut operators control commitment:

| Operator | Name | Effect |
|----------|------|--------|
| `!` | Local cut | Discard alternatives within this rule; release buffered input |
| `!!` | Full cut | Also commit the caller's choice point |

```fmpl
grammar NetParser <: Stream {
  message = request | response

  // Local cut: if "GET" matches but path fails, can still try response
  request = "GET" ! path headers body

  // Full cut: once "HTTP" matches, committed to response - no fallback
  response = "HTTP" !! version status headers body

  // Without cut: would buffer everything for potential backtrack
  header = name ":" value crlf
}
```

### Buffering Strategy

| Input Type | Strategy |
|------------|----------|
| String / finite list | Full packrat memoization |
| Bounded tree | Full packrat memoization |
| Unbounded stream | Sliding window buffer + explicit cuts |

For unbounded streams:
- Buffer recent input for local backtracking
- `!` releases buffer before cut point
- `!!` releases buffer and commits caller
- Backtracking past released buffer = runtime error

### Pipeline Composition

Chain grammars with the pipe operator:

```fmpl
let result = source
  |> Lexer.tokens
  |> Parser.program
  |> Desugar.stmt
  |> Optimize.expr
  |> CodeGen.emit
```

Each stage can be a streaming transformer - output of one feeds input of next.

For push streams, the pipeline is reactive:

```fmpl
let pipeline = Lexer.tokens |> Parser.stmt |> Interpreter.eval
networkInput |> pipeline |> outputSink
```

### Horizontal Composition (Grammar Merging)

Combine rule sets from multiple grammars:

```fmpl
grammar Combined = BaseParser + Extensions + Overrides
```

Later grammars override earlier ones for same-named rules.

Or selectively import rules:

```fmpl
grammar MyLang {
  use BaseParser.{expr, stmt}
  use JsonParser.{value as jsonValue}

  config = jsonValue
  program = stmt*
}
```

### Nested Grammar Invocation

One grammar can invoke another mid-parse on a sub-stream:

```fmpl
grammar Template {
  document = (text | embedded)*
  text = (!("{{" | "}}") .)+

  // Switch to expression grammar for embedded code
  embedded = "{{" code:Expr.expression "}}" => Interpolate(code)
}

grammar Expr {
  expression = term (('+' | '-') term)*
  term = factor (('*' | '/') factor)*
  factor = number | '(' expression ')'
}
```

The nested invocation `Expr.expression` parses from the current position using a different grammar.

### Stream Transformation Example

A complete streaming protocol parser:

```fmpl
grammar RedisParser <: Stream {
  // Entry point - parse commands from stream
  commands = command*

  command = simpleString
          | error
          | integer
          | bulkString
          | array

  // Once we see +, committed to simple string
  simpleString = '+' ! chars:(!crlf .)* crlf => Simple(chars)

  // Full cut after type byte - no recovery
  error = '-' !! msg:(!crlf .)* crlf => Error(msg)

  integer = ':' !! n:signedInt crlf => Int(n)

  bulkString = '$' !! len:int crlf content:take(len) crlf => Bulk(content)

  // Arrays can contain any command type
  array = '*' !! len:int crlf elements:command{len} => Array(elements)

  crlf = "\r\n"
}

// Usage as push stream
tcpSocket |> RedisParser.commands |> commandHandler
```

## EBNF Changes

Extend `<pattern>` to be usable in `<rule_primary>`:

```ebnf
<rule_primary> ::= <TAG>
                   <STRING>
                   <CHAR_RANGE>
                   '.'
                   '(' <rule_body> ')'
                   '[' <rule_body> ']'
                   '<' <TAG> '>'
                   '<' <TAG> <parmlist> '>'
                   ':' <TAG>
                   '{' <exp> '}'
                   <ARROW> <exp>
                   -- New: structural patterns
                   <pattern>

-- Cut operators
<rule_term> ::= <rule_primary>
                <rule_primary> '*'
                <rule_primary> '+'
                <rule_primary> '?'
                '~' <rule_primary>
                '&' <rule_primary>
                '!' <rule_primary>
                -- New: cut operators
                '!'                    -- local cut
                '!!'                   -- full cut

-- Grammar modes
<grammar_def> ::= <GRAMMAR> <qualified_tag> <grammar_mode> <grammar_inheritance> '{' <grammar_rules> '}'

<grammar_mode> ::= [ '<:' <STREAM> ]
                   [ '<:' <STREAM> '(' <stream_options> ')' ]

<stream_options> ::= <TAG> ':' <exp>
                     <stream_options> ',' <TAG> ':' <exp>

-- Grammar composition
<grammar_def> ::= ...
                  <GRAMMAR> <qualified_tag> '=' <grammar_expr>

<grammar_expr> ::= <qualified_tag>
                   <grammar_expr> '+' <qualified_tag>
```

The existing `<pattern>` nonterminal already covers:
- `Tag(p1, p2, ...)` - tagged nodes
- `%{k: p, ...}` - map patterns
- `[p1, p2 | rest]` - list patterns
- `_` - wildcard
- Literals and bindings

## Implementation Notes

### Pattern Compilation

Tree patterns compile to match functions that:
1. Check the structural shape (tag, keys, length)
2. Recursively match sub-patterns
3. Collect bindings into an environment
4. Return match success/failure plus bindings

### Backtracking

PEG ordered choice (`|`) backtracks on failure. For tree patterns this means:
- Try first alternative
- On failure, restore input position (for streams) or retry with next alternative (for trees)
- Tree matching is typically non-consuming, so backtracking is just "try next pattern"

For streaming grammars:
- Buffer input until cut point
- Local cut (`!`) releases buffer, but caller can still backtrack to try other alternatives
- Full cut (`!!`) releases buffer and removes caller's choice points

### Performance Considerations

- Memoization (Packrat parsing) works for tree grammars too
- Common pattern prefixes can be factored for efficiency
- Type tags enable fast dispatch before full pattern matching
- Streaming grammars avoid memoization overhead on unbounded input

## Future Extensions

- **Pattern guards with multiple clauses**: `| p when g1 => e1 when g2 => e2`
- **View patterns**: `| (f -> p) => ...` applies f then matches p
- **Pattern synonyms**: `pattern Pair(a, b) = [a, b]`
- **Typed patterns**: `| n:Num : Int => ...` with type annotations
- **Parallel grammar execution**: `p1 & p2` match both patterns on same input
- **Incremental reparsing**: update parse tree after source edit
