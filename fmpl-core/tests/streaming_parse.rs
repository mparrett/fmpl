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
