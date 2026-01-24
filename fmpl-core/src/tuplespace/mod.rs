//! Linda-style tuple space for pattern-based coordination.
//!
//! The tuple space provides time/space decoupled coordination between agents
//! through pattern-based matching instead of direct addressing.

pub mod facet;
pub mod store;
pub mod stream;

use crate::value::Value;
use smol_str::SmolStr;
use std::collections::HashMap;

/// A tuple with metadata for pattern matching.
#[derive(Debug, Clone, PartialEq)]
pub struct Tuple {
    /// Tuple type for routing/pattern matching
    pub type_name: SmolStr,
    /// Optional namespace for isolation
    pub namespace: Option<SmolStr>,
    /// Timestamp for ordering
    pub timestamp: u64,
    /// Sequence number for deterministic ordering
    pub seq: u64,
    /// The tuple data
    pub data: Value,
}

impl Tuple {
    /// Create a new tuple.
    pub fn new(type_name: SmolStr, data: Value) -> Self {
        Self {
            type_name,
            namespace: None,
            timestamp: 0,
            seq: 0,
            data,
        }
    }

    /// Set the namespace.
    pub fn with_namespace(mut self, namespace: SmolStr) -> Self {
        self.namespace = Some(namespace);
        self
    }

    /// Set the timestamp.
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Set the sequence number.
    pub fn with_seq(mut self, seq: u64) -> Self {
        self.seq = seq;
        self
    }
}

/// Pattern for matching tuples.
#[derive(Debug, Clone, PartialEq)]
pub enum TuplePattern {
    /// Exact match on type, pattern match on data
    TypeAndData { type_name: SmolStr, data: Pattern },
    /// Match on namespace + type + data
    Full {
        namespace: SmolStr,
        type_name: SmolStr,
        data: Pattern,
    },
    /// Wildcard: matches any tuple
    Any,
}

/// Pattern for matching tuple data.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Matches any value
    Wildcard,
    /// Exact value match
    Exact(Value),
    /// Map pattern with key-value pairs
    Map { required: HashMap<SmolStr, Value> },
}

impl Pattern {
    /// Check if a value matches this pattern.
    pub fn matches(&self, value: &Value) -> bool {
        match self {
            Pattern::Wildcard => true,
            Pattern::Exact(expected) => value == expected,
            Pattern::Map { required } => {
                if let Value::Map(map) = value {
                    for (key, expected_val) in required.iter() {
                        match map.get(key) {
                            Some(actual_val) if actual_val == expected_val => {}
                            _ => return false,
                        }
                    }
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl TuplePattern {
    /// Check if a tuple matches this pattern.
    pub fn matches(&self, tuple: &Tuple) -> bool {
        match self {
            TuplePattern::Any => true,
            TuplePattern::TypeAndData { type_name, data } => {
                tuple.type_name == *type_name && data.matches(&tuple.data)
            }
            TuplePattern::Full {
                namespace,
                type_name,
                data,
            } => {
                tuple.namespace.as_ref() == Some(namespace)
                    && tuple.type_name == *type_name
                    && data.matches(&tuple.data)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn make_map(pairs: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
        let map: HashMap<SmolStr, Value> = pairs
            .into_iter()
            .map(|(k, v)| (SmolStr::new(k), v))
            .collect();
        Value::Map(Arc::new(map))
    }

    #[test]
    fn test_pattern_wildcard_matches_anything() {
        let pattern = Pattern::Wildcard;
        assert!(pattern.matches(&Value::Int(42)));
        assert!(pattern.matches(&Value::String(SmolStr::new("hello"))));
        assert!(pattern.matches(&Value::Null));
    }

    #[test]
    fn test_pattern_exact_matches_equal_values() {
        let pattern = Pattern::Exact(Value::Int(42));
        assert!(pattern.matches(&Value::Int(42)));
        assert!(!pattern.matches(&Value::Int(43)));
        assert!(!pattern.matches(&Value::String(SmolStr::new("42"))));
    }

    #[test]
    fn test_pattern_map_matches_subset() {
        let pattern = Pattern::Map {
            required: {
                let mut m = HashMap::new();
                m.insert(SmolStr::new("x"), Value::Int(1));
                m.insert(SmolStr::new("y"), Value::Int(2));
                m
            },
        };

        // Match: has all required keys with correct values
        let value = make_map(vec![
            ("x", Value::Int(1)),
            ("y", Value::Int(2)),
            ("z", Value::Int(3)),
        ]);
        assert!(pattern.matches(&value));

        // Match: exact keys
        let value = make_map(vec![("x", Value::Int(1)), ("y", Value::Int(2))]);
        assert!(pattern.matches(&value));

        // No match: missing key
        let value = make_map(vec![("x", Value::Int(1))]);
        assert!(!pattern.matches(&value));

        // No match: wrong value
        let value = make_map(vec![("x", Value::Int(1)), ("y", Value::Int(99))]);
        assert!(!pattern.matches(&value));
    }

    #[test]
    fn test_pattern_map_requires_map_value() {
        let pattern = Pattern::Map {
            required: {
                let mut m = HashMap::new();
                m.insert(SmolStr::new("x"), Value::Int(1));
                m
            },
        };

        assert!(!pattern.matches(&Value::Int(42)));
        assert!(!pattern.matches(&Value::String(SmolStr::new("hello"))));
    }

    #[test]
    fn test_tuple_pattern_any_matches_all() {
        let pattern = TuplePattern::Any;
        let tuple = Tuple::new(SmolStr::new("event"), Value::Int(42));
        assert!(pattern.matches(&tuple));
    }

    #[test]
    fn test_tuple_pattern_type_and_data() {
        let pattern = TuplePattern::TypeAndData {
            type_name: SmolStr::new("event"),
            data: Pattern::Exact(Value::Int(42)),
        };

        let tuple = Tuple::new(SmolStr::new("event"), Value::Int(42));
        assert!(pattern.matches(&tuple));

        let wrong_type = Tuple::new(SmolStr::new("other"), Value::Int(42));
        assert!(!pattern.matches(&wrong_type));

        let wrong_data = Tuple::new(SmolStr::new("event"), Value::Int(99));
        assert!(!pattern.matches(&wrong_data));
    }

    #[test]
    fn test_tuple_pattern_full() {
        let pattern = TuplePattern::Full {
            namespace: SmolStr::new("user123"),
            type_name: SmolStr::new("click"),
            data: Pattern::Map {
                required: {
                    let mut m = HashMap::new();
                    m.insert(SmolStr::new("x"), Value::Int(100));
                    m
                },
            },
        };

        let tuple = Tuple::new(
            SmolStr::new("click"),
            make_map(vec![("x", Value::Int(100)), ("y", Value::Int(200))]),
        )
        .with_namespace(SmolStr::new("user123"));
        assert!(pattern.matches(&tuple));

        let wrong_namespace = Tuple::new(
            SmolStr::new("click"),
            make_map(vec![("x", Value::Int(100))]),
        )
        .with_namespace(SmolStr::new("user456"));
        assert!(!pattern.matches(&wrong_namespace));
    }

    #[test]
    fn test_tuple_builder_methods() {
        let tuple = Tuple::new(SmolStr::new("test"), Value::Int(42))
            .with_namespace(SmolStr::new("ns"))
            .with_timestamp(123)
            .with_seq(456);

        assert_eq!(tuple.type_name, SmolStr::new("test"));
        assert_eq!(tuple.namespace, Some(SmolStr::new("ns")));
        assert_eq!(tuple.timestamp, 123);
        assert_eq!(tuple.seq, 456);
        assert_eq!(tuple.data, Value::Int(42));
    }
}
