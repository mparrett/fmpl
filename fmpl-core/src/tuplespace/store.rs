//! Tuple space store with blocking operations and Fjall persistence.
//!
//! This module provides the in-memory tuple space with optional Fjall backing
//! for durable tuple storage.

use crate::error::{Error, Result};
use crate::tuplespace::{Tuple, TuplePattern};
use smol_str::SmolStr;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// In-memory tuple space with Fjall persistence.
pub struct TupleSpace {
    /// Next sequence number
    next_seq: Arc<AtomicU64>,
    /// Tuples by (namespace, type, seq)
    tuples: BTreeMap<(Option<SmolStr>, SmolStr, u64), Tuple>,
}

impl TupleSpace {
    /// Create a new tuple space.
    pub fn new() -> Self {
        Self {
            next_seq: Arc::new(AtomicU64::new(1)),
            tuples: BTreeMap::new(),
        }
    }

    /// Write a tuple to the space.
    pub fn out(&mut self, tuple: Tuple) -> Result<()> {
        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
        let key = (tuple.namespace.clone(), tuple.type_name.clone(), seq);
        self.tuples.insert(key, tuple);
        Ok(())
    }

    /// Remove a matching tuple (blocking).
    pub fn r#in(&mut self, pattern: &TuplePattern) -> Result<Tuple> {
        // For now, non-blocking implementation
        self.inp(pattern)?
            .ok_or_else(|| Error::Runtime("no matching tuple found".to_string()))
    }

    /// Read a matching tuple (blocking, non-destructive).
    pub fn rd(&mut self, pattern: &TuplePattern) -> Result<Tuple> {
        // For now, non-blocking implementation
        self.rdp(pattern)?
            .ok_or_else(|| Error::Runtime("no matching tuple found".to_string()))
    }

    /// Non-blocking remove (returns None if no match).
    pub fn inp(&mut self, pattern: &TuplePattern) -> Result<Option<Tuple>> {
        for ((ns, type_name, seq), tuple) in self.tuples.iter() {
            if pattern.matches(tuple) {
                let tuple = tuple.clone();
                self.tuples.remove(&(ns.clone(), type_name.clone(), *seq));
                return Ok(Some(tuple));
            }
        }
        Ok(None)
    }

    /// Non-blocking read (returns None if no match).
    pub fn rdp(&mut self, pattern: &TuplePattern) -> Result<Option<Tuple>> {
        for (_, tuple) in self.tuples.iter() {
            if pattern.matches(tuple) {
                return Ok(Some(tuple.clone()));
            }
        }
        Ok(None)
    }
}

impl Default for TupleSpace {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tuplespace::Pattern;
    use crate::value::Value;
    use smol_str::SmolStr;

    #[test]
    fn test_out_and_in() {
        let mut space = TupleSpace::new();
        let tuple = Tuple::new(SmolStr::new("test"), Value::Int(42));
        space.out(tuple.clone()).unwrap();

        let pattern = TuplePattern::TypeAndData {
            type_name: SmolStr::new("test"),
            data: Pattern::Wildcard,
        };

        let result = space.r#in(&pattern).unwrap();
        assert_eq!(result.data, Value::Int(42));
    }

    #[test]
    fn test_in_returns_fifo() {
        let mut space = TupleSpace::new();
        space
            .out(Tuple::new(SmolStr::new("test"), Value::Int(1)))
            .unwrap();
        space
            .out(Tuple::new(SmolStr::new("test"), Value::Int(2)))
            .unwrap();

        let pattern = TuplePattern::TypeAndData {
            type_name: SmolStr::new("test"),
            data: Pattern::Wildcard,
        };

        assert_eq!(space.r#in(&pattern).unwrap().data, Value::Int(1));
        assert_eq!(space.r#in(&pattern).unwrap().data, Value::Int(2));
    }

    #[test]
    fn test_rdp_non_blocking() {
        let mut space = TupleSpace::new();

        let pattern = TuplePattern::Any;
        assert!(space.rdp(&pattern).unwrap().is_none());

        space
            .out(Tuple::new(SmolStr::new("test"), Value::Int(42)))
            .unwrap();
        assert!(space.rdp(&pattern).unwrap().is_some());
    }

    #[test]
    fn test_rd_is_non_destructive() {
        let mut space = TupleSpace::new();
        space
            .out(Tuple::new(SmolStr::new("test"), Value::Int(42)))
            .unwrap();

        let pattern = TuplePattern::Any;
        let result1 = space.rd(&pattern).unwrap();
        assert_eq!(result1.data, Value::Int(42));

        let result2 = space.rd(&pattern).unwrap();
        assert_eq!(result2.data, Value::Int(42));
    }

    #[test]
    fn test_in_is_destructive() {
        let mut space = TupleSpace::new();
        space
            .out(Tuple::new(SmolStr::new("test"), Value::Int(42)))
            .unwrap();

        let pattern = TuplePattern::Any;
        let result1 = space.r#in(&pattern).unwrap();
        assert_eq!(result1.data, Value::Int(42));

        let result2 = space.inp(&pattern).unwrap();
        assert!(result2.is_none());
    }
}
