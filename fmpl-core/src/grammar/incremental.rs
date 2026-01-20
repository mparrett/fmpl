//! Incremental parsing support for streaming grammars.
//!
//! Provides ParseState for suspension/resumption and ParseNext for
//! incremental parse results.

use crate::value::Value;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::collections::HashMap;

/// State needed to resume an incremental parse.
///
/// Captures position, rule call stack, and bindings for serialization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParseState {
    /// Current position index in input.
    pub position_index: usize,
    /// Rule call stack: (rule_name, entry_position_index).
    pub rule_stack: Vec<(SmolStr, usize)>,
    /// Current variable bindings.
    pub bindings: HashMap<SmolStr, Value>,
}

/// Result of an incremental parse step.
#[derive(Debug, Clone)]
pub enum ParseNext {
    /// Rule matched, here's the result value.
    Match(Value),
    /// Need more input - here's state to resume from.
    NeedInput(ParseState),
    /// Input stream ended.
    End,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_state_serialization() {
        let state = ParseState {
            position_index: 5,
            rule_stack: vec![("digit".into(), 3), ("integer".into(), 0)],
            bindings: [("x".into(), Value::Int(42))].into_iter().collect(),
        };

        let serialized = serde_json::to_string(&state).unwrap();
        let deserialized: ParseState = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.position_index, 5);
        assert_eq!(deserialized.rule_stack.len(), 2);
        assert_eq!(deserialized.bindings.get("x"), Some(&Value::Int(42)));
    }

    #[test]
    fn test_parse_next_variants() {
        let match_result: ParseNext = ParseNext::Match(Value::Int(42));
        assert!(matches!(match_result, ParseNext::Match(Value::Int(42))));

        let need_input: ParseNext = ParseNext::NeedInput(ParseState::default());
        assert!(matches!(need_input, ParseNext::NeedInput(_)));

        let end: ParseNext = ParseNext::End;
        assert!(matches!(end, ParseNext::End));
    }
}
