//! OMeta-style extensible grammars for FMPL.
//!
//! This module provides PEG-based parsing with grammar inheritance,
//! packrat memoization, and semantic actions that produce FMPL values.
//!
//! Unlike traditional PEG parsers, OMeta can parse any stream of objects:
//! - **Strings**: Character-by-character text parsing
//! - **Binary**: Byte streams for protocol/file format parsing
//! - **Objects**: Lists/trees of FMPL values for AST transformation
//!
//! # Text Parsing Example
//!
//! ```fmpl
//! grammar mud::commands <: base::parser {
//!     verb = word:v &{ valid_verb(v) } => v
//!     command = "take" spaces noun:obj => %{action: :take, target: obj}
//! }
//! "take sword" @ mud::commands.command
//! ```
//!
//! # Binary Parsing Example
//!
//! ```fmpl
//! grammar png::header <: base::binary {
//!     magic = byte(0x89) byte(0x50) byte(0x4E) byte(0x47)
//!     chunk = uint32be:len uint32be:type bytes(len):data uint32be:crc
//! }
//! file_bytes @ png::header.magic
//! ```
//!
//! # Object/Tree Parsing Example
//!
//! ```fmpl
//! grammar ast::optimizer <: base::tree {
//!     -- Constant folding: (+ 1 2) => 3
//!     add = [:add const:a const:b] => a + b
//!     const = :int(n) => n
//! }
//! ast @ ast::optimizer.add
//! ```
//!
//! # Streaming Grammar Pipelines
//!
//! Grammars can operate on async streams with full backtracking support:
//!
//! ```fmpl
//! llm_stream |> parser.tool_call |> execute_tool
//! ```
//!
//! The pipeline works like Unix pipes:
//! - Each value from `llm_stream` pushes into `parser.tool_call`
//! - When `tool_call` fully matches, its result pushes to `execute_tool`
//! - Backtracking is unlimited with buffered input (spills to Fjall)
//! - Memoization prevents re-execution of external calls
//!
//! # Durable Suspension
//!
//! Parse state can be serialized for durable suspension across process
//! restarts. This enables human-in-the-loop workflows where an agent
//! pauses mid-parse waiting for approval:
//!
//! ```rust,ignore
//! // Start parsing
//! let mut runtime = PegRuntime::new(input, &registry, grammar);
//! let state = runtime.start("rule_name");
//!
//! // Suspend: serialize state to Fjall
//! let bytes = state.to_bytes()?;
//! partition.insert(session_key, bytes)?;
//!
//! // ... process restarts, human approves ...
//!
//! // Resume: restore state from Fjall
//! let bytes = partition.get(session_key)?.unwrap();
//! let restored = ParseState::from_bytes(&bytes)?;
//! runtime.resume(restored)?;
//! ```
//!
//! See [`incremental::ParseState`] for serialization methods and
//! [`driver::ParseDriver`] for async pipeline integration.

pub mod driver;
pub mod incremental;
pub mod input;
pub mod optimizer;
pub mod parser;
pub mod runtime;
pub mod stream_input;
pub mod trampoline;

use crate::ast::Expr;
use crate::value::Value;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::collections::HashMap;
use std::sync::Arc;

/// A grammar definition with rules and optional parent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Grammar {
    /// Fully qualified grammar name (e.g., "mud::commands").
    pub name: SmolStr,
    /// Parent grammar name for registry lookup (legacy).
    pub parent: Option<SmolStr>,
    /// Direct parent grammar reference (for first-class grammars).
    pub parent_grammar: Option<Arc<Grammar>>,
    /// Named rules in this grammar.
    pub rules: HashMap<SmolStr, Rule>,
}

impl Grammar {
    pub fn new(name: SmolStr) -> Self {
        Self {
            name,
            parent: None,
            parent_grammar: None,
            rules: HashMap::new(),
        }
    }

    /// Create grammar with named parent (for registry lookup).
    pub fn with_parent(name: SmolStr, parent: SmolStr) -> Self {
        Self {
            name,
            parent: Some(parent),
            parent_grammar: None,
            rules: HashMap::new(),
        }
    }

    /// Create grammar with direct parent reference (for first-class grammars).
    pub fn with_parent_grammar(name: SmolStr, parent: Arc<Grammar>) -> Self {
        Self {
            name,
            parent: None,
            parent_grammar: Some(parent),
            rules: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, name: SmolStr, rule: Rule) {
        self.rules.insert(name, rule);
    }
}

/// A grammar rule (pattern with optional semantic action).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Rule {
    /// The pattern to match.
    #[serde(default)]
    pub pattern: Pattern,
    /// Semantic action producing a value (optional).
    #[serde(default)]
    pub action: Option<Expr>,
    /// Whether this rule uses backtracking (enabled by `?` marker).
    #[serde(default)]
    pub backtracking: bool,
    /// Parameters for function-style rules: `name(param1, param2) = expr`
    /// When non-empty, this is a helper function rather than a parsing rule.
    #[serde(default)]
    pub params: Vec<SmolStr>,
}

impl Rule {
    pub fn new(pattern: Pattern) -> Self {
        Self {
            pattern,
            action: None,
            backtracking: false,
            params: Vec::new(),
        }
    }

    pub fn with_action(pattern: Pattern, action: Expr) -> Self {
        Self {
            pattern,
            action: Some(action),
            backtracking: false,
            params: Vec::new(),
        }
    }

    pub fn with_backtracking(mut self, backtracking: bool) -> Self {
        self.backtracking = backtracking;
        self
    }

    /// Create a function-style rule: `name(params) = expr`
    pub fn function(params: Vec<SmolStr>, body: Expr) -> Self {
        Self {
            pattern: Pattern::Empty,
            action: Some(body),
            backtracking: false,
            params,
        }
    }

    /// Check if this is a function-style rule (has parameters)
    pub fn is_function(&self) -> bool {
        !self.params.is_empty()
    }
}

// Re-export the unified Pattern type and related types from crate::pattern.
// This completes Phase 8 of the pattern unification migration.
pub use crate::pattern::{
    BinaryPattern, CharPattern, CharRange, ListPattern, LiteralValue, Pattern, RepeatKind,
};

/// Input stream for parsing - can be text, binary, or object stream.
#[derive(Debug, Clone)]
pub enum Input {
    /// Character stream (for text parsing).
    Text(String),
    /// Byte stream (for binary parsing).
    Binary(Vec<u8>),
    /// Value stream (for tree/object parsing).
    Values(Vec<Value>),
}

impl Input {
    /// Create a text input from a string.
    pub fn text(s: &str) -> Self {
        Input::Text(s.to_string())
    }

    /// Create a binary input from bytes.
    pub fn binary(bytes: Vec<u8>) -> Self {
        Input::Binary(bytes)
    }

    /// Create a value stream input.
    pub fn values(values: Vec<Value>) -> Self {
        Input::Values(values)
    }

    /// Create a value stream from a single value (unwrapping lists).
    pub fn from_value(value: Value) -> Self {
        match value {
            Value::List(items) => Input::Values((*items).clone()),
            other => Input::Values(vec![other]),
        }
    }

    /// Get the length of the input.
    pub fn len(&self) -> usize {
        match self {
            Input::Text(s) => s.len(),
            Input::Binary(b) => b.len(),
            Input::Values(v) => v.len(),
        }
    }

    /// Check if input is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Result of a parse operation.
#[derive(Debug, Clone)]
pub enum ParseResult {
    /// Successful match with value and new position.
    Success(Value, usize),
    /// Failed to match.
    Failure,
}

impl ParseResult {
    pub fn is_success(&self) -> bool {
        matches!(self, ParseResult::Success(_, _))
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, ParseResult::Failure)
    }
}

/// A grammar registry for looking up grammars by name.
#[derive(Debug, Clone, Default)]
pub struct GrammarRegistry {
    grammars: HashMap<SmolStr, Arc<Grammar>>,
}

impl GrammarRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            grammars: HashMap::new(),
        };
        // Register built-in grammars
        registry.register_builtins();
        registry
    }

    pub fn register(&mut self, grammar: Grammar) {
        let name = grammar.name.clone();
        self.grammars.insert(name, Arc::new(grammar));
    }

    pub fn get(&self, name: &str) -> Option<Arc<Grammar>> {
        self.grammars.get(name).cloned()
    }

    fn register_builtins(&mut self) {
        // base::parser - fundamental parsing primitives
        let mut base = Grammar::new(SmolStr::new("base::parser"));

        // any = .
        base.add_rule(SmolStr::new("any"), Rule::new(Pattern::Any));

        // digit = [0-9]
        base.add_rule(
            SmolStr::new("digit"),
            Rule::new(Pattern::Char(CharPattern::Class(vec![CharRange::Range(
                '0', '9',
            )]))),
        );

        // letter = [a-zA-Z]
        base.add_rule(
            SmolStr::new("letter"),
            Rule::new(Pattern::Char(CharPattern::Class(vec![
                CharRange::Range('a', 'z'),
                CharRange::Range('A', 'Z'),
            ]))),
        );

        // space = [ \t\n\r]
        base.add_rule(
            SmolStr::new("space"),
            Rule::new(Pattern::Char(CharPattern::Class(vec![
                CharRange::Char(' '),
                CharRange::Char('\t'),
                CharRange::Char('\n'),
                CharRange::Char('\r'),
            ]))),
        );

        // spaces = space*
        base.add_rule(
            SmolStr::new("spaces"),
            Rule::new(Pattern::Repeat {
                pattern: Box::new(Pattern::ApplyRule(SmolStr::new("space"))),
                kind: RepeatKind::ZeroOrMore,
            }),
        );

        // word = letter+
        base.add_rule(
            SmolStr::new("word"),
            Rule::new(Pattern::Repeat {
                pattern: Box::new(Pattern::ApplyRule(SmolStr::new("letter"))),
                kind: RepeatKind::OneOrMore,
            }),
        );

        // integer = digit+
        base.add_rule(
            SmolStr::new("integer"),
            Rule::new(Pattern::Repeat {
                pattern: Box::new(Pattern::ApplyRule(SmolStr::new("digit"))),
                kind: RepeatKind::OneOrMore,
            }),
        );

        // eof = ~.
        base.add_rule(
            SmolStr::new("eof"),
            Rule::new(Pattern::Not(Box::new(Pattern::Any))),
        );

        // end = end of input
        base.add_rule(SmolStr::new("end"), Rule::new(Pattern::End));

        self.register(base);

        // base::binary - binary parsing primitives
        let mut binary = Grammar::new(SmolStr::new("base::binary"));

        // any = any single byte
        binary.add_rule(SmolStr::new("any"), Rule::new(Pattern::Any));

        // byte = uint8
        binary.add_rule(
            SmolStr::new("byte"),
            Rule::new(Pattern::Binary(BinaryPattern::UInt8)),
        );

        // uint8, uint16be, uint16le, uint32be, uint32le
        binary.add_rule(
            SmolStr::new("uint8"),
            Rule::new(Pattern::Binary(BinaryPattern::UInt8)),
        );
        binary.add_rule(
            SmolStr::new("uint16be"),
            Rule::new(Pattern::Binary(BinaryPattern::UInt16BE)),
        );
        binary.add_rule(
            SmolStr::new("uint16le"),
            Rule::new(Pattern::Binary(BinaryPattern::UInt16LE)),
        );
        binary.add_rule(
            SmolStr::new("uint32be"),
            Rule::new(Pattern::Binary(BinaryPattern::UInt32BE)),
        );
        binary.add_rule(
            SmolStr::new("uint32le"),
            Rule::new(Pattern::Binary(BinaryPattern::UInt32LE)),
        );

        // int8, int16be, int16le, int32be, int32le
        binary.add_rule(
            SmolStr::new("int8"),
            Rule::new(Pattern::Binary(BinaryPattern::Int8)),
        );
        binary.add_rule(
            SmolStr::new("int16be"),
            Rule::new(Pattern::Binary(BinaryPattern::Int16BE)),
        );
        binary.add_rule(
            SmolStr::new("int16le"),
            Rule::new(Pattern::Binary(BinaryPattern::Int16LE)),
        );
        binary.add_rule(
            SmolStr::new("int32be"),
            Rule::new(Pattern::Binary(BinaryPattern::Int32BE)),
        );
        binary.add_rule(
            SmolStr::new("int32le"),
            Rule::new(Pattern::Binary(BinaryPattern::Int32LE)),
        );

        // end = end of input
        binary.add_rule(SmolStr::new("end"), Rule::new(Pattern::End));

        self.register(binary);

        // base::tree - tree/object parsing primitives
        let mut tree = Grammar::new(SmolStr::new("base::tree"));

        // any = any single value from the stream
        tree.add_rule(SmolStr::new("any"), Rule::new(Pattern::Any));

        // null = match null value
        tree.add_rule(
            SmolStr::new("null"),
            Rule::new(Pattern::MatchValue(Value::Null)),
        );

        // bool = match any boolean
        tree.add_rule(
            SmolStr::new("bool"),
            Rule::new(Pattern::MatchType(SmolStr::new("bool"))),
        );

        // int = match any integer
        tree.add_rule(
            SmolStr::new("int"),
            Rule::new(Pattern::MatchType(SmolStr::new("int"))),
        );

        // float = match any float
        tree.add_rule(
            SmolStr::new("float"),
            Rule::new(Pattern::MatchType(SmolStr::new("float"))),
        );

        // string = match any string
        tree.add_rule(
            SmolStr::new("string"),
            Rule::new(Pattern::MatchType(SmolStr::new("string"))),
        );

        // symbol = match any symbol
        tree.add_rule(
            SmolStr::new("symbol"),
            Rule::new(Pattern::MatchType(SmolStr::new("symbol"))),
        );

        // list = match any list
        tree.add_rule(
            SmolStr::new("list"),
            Rule::new(Pattern::MatchType(SmolStr::new("list"))),
        );

        // map = match any map
        tree.add_rule(
            SmolStr::new("map"),
            Rule::new(Pattern::MatchType(SmolStr::new("map"))),
        );

        // end = end of input
        tree.add_rule(SmolStr::new("end"), Rule::new(Pattern::End));

        self.register(tree);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_range_single() {
        let range = CharRange::Char('a');
        assert!(range.matches('a'));
        assert!(!range.matches('b'));
    }

    #[test]
    fn test_char_range_range() {
        let range = CharRange::Range('a', 'z');
        assert!(range.matches('a'));
        assert!(range.matches('m'));
        assert!(range.matches('z'));
        assert!(!range.matches('A'));
        assert!(!range.matches('0'));
    }

    #[test]
    fn test_grammar_registry_builtins() {
        let registry = GrammarRegistry::new();
        let base = registry
            .get("base::parser")
            .expect("base::parser should exist");
        assert!(base.rules.contains_key("digit"));
        assert!(base.rules.contains_key("letter"));
        assert!(base.rules.contains_key("spaces"));
    }

    #[test]
    fn test_grammar_with_arc_parent() {
        // Create a parent grammar
        let mut parent = Grammar::new(SmolStr::new("parent"));
        parent.add_rule(
            SmolStr::new("foo"),
            Rule::new(Pattern::StringLiteral(SmolStr::new("foo"))),
        );
        let parent = Arc::new(parent);

        // Create child with Arc parent
        let mut child = Grammar::with_parent_grammar(SmolStr::new("child"), parent.clone());
        child.add_rule(
            SmolStr::new("bar"),
            Rule::new(Pattern::StringLiteral(SmolStr::new("bar"))),
        );

        // Child should have access to parent
        assert!(child.parent_grammar.is_some());
        assert_eq!(child.parent_grammar.as_ref().unwrap().name, "parent");
    }
}
