//! Tests for streaming parse operations.

use fmpl_core::value::{Stream, StreamOp, Value};
use smol_str::SmolStr;
use std::sync::Arc;

#[test]
fn test_async_parse_stream_op() {
    // Just verify the type can be constructed
    let op = StreamOp::AsyncParse {
        grammar: Value::Null, // placeholder
        rule: SmolStr::new("test_rule"),
    };

    assert!(matches!(op, StreamOp::AsyncParse { .. }));
}

#[test]
fn test_async_parse_in_stream() {
    // Verify AsyncParse can be added to a stream pipeline
    let stream = Stream {
        source: Value::List(Arc::new(vec![])),
        ops: vec![StreamOp::AsyncParse {
            grammar: Value::Null,
            rule: SmolStr::new("root"),
        }],
    };

    assert_eq!(stream.ops.len(), 1);
    assert!(matches!(&stream.ops[0], StreamOp::AsyncParse { rule, .. } if rule == "root"));
}

#[test]
fn test_async_parse_clone() {
    // Verify Clone trait works
    let op = StreamOp::AsyncParse {
        grammar: Value::Int(42),
        rule: SmolStr::new("expr"),
    };

    let cloned = op.clone();
    assert!(matches!(cloned, StreamOp::AsyncParse { rule, .. } if rule == "expr"));
}

// Tests for Collect, Take, Drop stream operators

#[test]
fn test_collect_stream_op() {
    let op = StreamOp::Collect;
    assert!(matches!(op, StreamOp::Collect));
}

#[test]
fn test_collect_in_stream() {
    let stream = Stream {
        source: Value::List(Arc::new(vec![Value::Int(1), Value::Int(2)])),
        ops: vec![StreamOp::Collect],
    };

    assert_eq!(stream.ops.len(), 1);
    assert!(matches!(&stream.ops[0], StreamOp::Collect));
}

#[test]
fn test_take_stream_op() {
    let op = StreamOp::Take { n: Value::Int(5) };
    assert!(matches!(op, StreamOp::Take { .. }));
}

#[test]
fn test_take_in_stream() {
    let stream = Stream {
        source: Value::List(Arc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)])),
        ops: vec![StreamOp::Take { n: Value::Int(2) }],
    };

    assert_eq!(stream.ops.len(), 1);
    assert!(matches!(&stream.ops[0], StreamOp::Take { .. }));
}

#[test]
fn test_take_clone() {
    let op = StreamOp::Take { n: Value::Int(10) };
    let cloned = op.clone();
    assert!(matches!(cloned, StreamOp::Take { .. }));
}

#[test]
fn test_drop_stream_op() {
    let op = StreamOp::Drop { n: Value::Int(3) };
    assert!(matches!(op, StreamOp::Drop { .. }));
}

#[test]
fn test_drop_in_stream() {
    let stream = Stream {
        source: Value::List(Arc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)])),
        ops: vec![StreamOp::Drop { n: Value::Int(1) }],
    };

    assert_eq!(stream.ops.len(), 1);
    assert!(matches!(&stream.ops[0], StreamOp::Drop { .. }));
}

#[test]
fn test_drop_clone() {
    let op = StreamOp::Drop { n: Value::Int(2) };
    let cloned = op.clone();
    assert!(matches!(cloned, StreamOp::Drop { .. }));
}

#[test]
fn test_stream_with_multiple_ops() {
    let stream = Stream {
        source: Value::List(Arc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)])),
        ops: vec![
            StreamOp::Take { n: Value::Int(5) },
            StreamOp::Drop { n: Value::Int(1) },
            StreamOp::Collect,
        ],
    };

    assert_eq!(stream.ops.len(), 3);
    assert!(matches!(&stream.ops[0], StreamOp::Take { .. }));
    assert!(matches!(&stream.ops[1], StreamOp::Drop { .. }));
    assert!(matches!(&stream.ops[2], StreamOp::Collect));
}
