//! Unified pattern type for FMPL

use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// Unified pattern type for both let bindings and grammar rules.
/// Compilation behavior depends on context (fast vs full path).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    /// Wildcard - matches anything, binds nothing
    Any,

    /// Variable binding - matches anything, binds to name
    Var(SmolStr),

    /// Literal value - matches exact value
    Literal(LiteralValue),

    /// Map pattern - %{key1: pattern1, key2: pattern2}
    Map(Vec<(SmolStr, Pattern)>),

    /// List pattern - [p1, p2, p3] or [head | tail] or [p*]
    List(ListPattern),

    /// Tagged/constructor pattern - :Tag(p1, p2, ...)
    Tagged {
        tag: SmolStr,
        patterns: Vec<Pattern>,
    },

    /// Character pattern (for strings) - 'a' or [a-z]
    Char(CharPattern),

    /// Sequence - p1 p2 p3 (ordered, all must match)
    Seq(Vec<Pattern>),

    /// Ordered choice - p1 | p2 | p3 (try first that matches)
    Choice(Vec<Pattern>),

    /// Repetition - p* (zero or more) or p+ (one or more)
    Repeat {
        pattern: Box<Pattern>,
        kind: RepeatKind,
    },

    /// Optional - p? (zero or one)
    Optional(Box<Pattern>),

    /// Lookahead - &p (positive) or !p (negative)
    Lookahead {
        pattern: Box<Pattern>,
        positive: bool,
    },

    /// Binding - name: pattern or pattern when guard
    Bind {
        name: SmolStr,
        pattern: Box<Pattern>,
    },

    /// Guard - pattern when predicate
    Guard {
        pattern: Box<Pattern>,
        predicate: GuardPredicate,
    },

    /// Action - pattern => expr
    /// The `action` field contains an expression string to evaluate on successful match.
    Action {
        pattern: Box<Pattern>,
        action: SmolStr,
    },

    /// Rule application - applies named grammar rule
    ApplyRule(SmolStr),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralValue {
    String(SmolStr),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ListPattern {
    /// Exact list pattern - matches [p1, p2, p3]
    Exact(Vec<Pattern>),
    /// Head-tail pattern - matches [h | t]
    HeadTail {
        head: Box<Pattern>,
        tail: Option<SmolStr>,
    },
    /// Repeat pattern - matches [p*]
    Repeat { element: Box<Pattern> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharPattern {
    /// Exact character match - 'a'
    Exact(char),
    /// Character class match - [a-z]
    Class(Vec<(char, char)>),
    /// Negated character class match - [^a-z]
    NegatedClass(Vec<(char, char)>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatKind {
    /// Zero or more repetitions - p*
    ZeroOrMore,
    /// One or more repetitions - p+
    OneOrMore,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardPredicate {
    /// Expression to evaluate as a boolean guard
    Expr(SmolStr),
    /// Type check guard - is_list, is_map, is_int, etc.
    TypeCheck(SmolStr),
}

/// Compilation mode for patterns - determines strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternMode {
    /// Fast path: direct extraction, no backtracking (for let bindings)
    /// Uses ExtractMapKey, ExtractListIndex, ExtractTaggedChild
    Fast,

    /// Full path: grammar matching with backtracking (for @ operator)
    /// Uses MatchSeq, MatchChoice, MatchGuard, etc.
    Full,
}

impl Pattern {
    /// Determine if pattern requires full matching (backtracking/guards)
    pub fn requires_full_mode(&self) -> bool {
        match self {
            Pattern::Seq(_) | Pattern::Choice(_) | Pattern::Repeat { .. } => true,
            Pattern::Lookahead { .. } | Pattern::Guard { .. } => true,
            Pattern::Action { .. } => true,
            Pattern::Char(_) => true, // Only for string parsing
            Pattern::List(ListPattern::Repeat { .. }) => true,
            _ => false,
        }
    }

    /// Get recommended compilation mode for this pattern
    pub fn recommended_mode(&self) -> PatternMode {
        if self.requires_full_mode() {
            PatternMode::Full
        } else {
            PatternMode::Fast
        }
    }
}
