// Tests for module-style function calls (json::parse, stream::observe, cursor::advance, etc.)
//
// These tests verify that qualified names like `module::function(args)` are correctly
// parsed and compiled to their builtin equivalents.

use fmpl_core::{Value, eval};

/// Test helper to evaluate code and check for errors
fn eval_ok(code: &str) -> Result<Value, String> {
    let mut vm = fmpl_core::Vm::new();
    eval(&mut vm, code).map_err(|e| e.to_string())
}

/// Test helper to evaluate and expect an error
fn eval_err(code: &str, expected_msg: &str) {
    let result = eval_ok(code);
    assert!(result.is_err(), "Expected error but got: {:?}", result);
    let err = result.unwrap_err();
    // Use contains instead of regex for simpler matching
    assert!(
        err.contains(expected_msg) || err.to_lowercase().contains(&expected_msg.to_lowercase()),
        "Expected error containing '{}', got: '{}'",
        expected_msg,
        err
    );
}

#[test]
fn test_json_parse_module_call() {
    let result = eval_ok(r#"json::parse("{\"a\": 1}")"#);
    assert!(result.is_ok(), "json::parse should work: {:?}", result);
}

#[test]
fn test_json_stringify_module_call() {
    let result = eval_ok(r#"json::stringify(%{a: 1})"#);
    assert!(result.is_ok(), "json::stringify should work: {:?}", result);
}

#[test]
fn test_sse_parse_module_call() {
    let sse_data = "event: message\ndata: hello\n\n";
    let result = eval_ok(&format!(
        "sse::parse(\"{}\")",
        sse_data.replace('\n', "\\n")
    ));
    assert!(result.is_ok(), "sse::parse should work: {:?}", result);
}

#[test]
fn test_time_sleep_module_call() {
    let result = eval_ok("time::sleep(0)");
    assert!(result.is_ok(), "time::sleep should work: {:?}", result);
}

#[test]
fn test_rand_int_module_call() {
    let result = eval_ok("rand::int(1, 10)");
    assert!(result.is_ok(), "rand::int should work: {:?}", result);

    if let Ok(Value::Int(n)) = result {
        assert!(
            (1..=10).contains(&n),
            "rand::int(1, 10) should return value in range, got: {}",
            n
        );
    }
}

#[test]
fn test_rand_float_module_call() {
    let result = eval_ok("rand::float()");
    assert!(result.is_ok(), "rand::float should work: {:?}", result);

    if let Ok(Value::Float(f)) = result {
        assert!(
            (0.0..1.0).contains(&f),
            "rand::float() should return value in [0, 1), got: {}",
            f
        );
    }
}

// ============================================================================
// Stream module tests
// ============================================================================

#[test]
fn test_stream_observe_with_list_parens() {
    // With explicit parentheses around list argument
    let result = eval_ok("stream::observe([1, 2, 3])");
    assert!(
        result.is_ok(),
        "stream::observe with list in parens should work: {:?}",
        result
    );

    match result {
        Ok(Value::Cursor(_)) => {}
        other => panic!("Expected Cursor, got: {:?}", other),
    }
}

#[test]
fn test_stream_observe_with_string() {
    let result = eval_ok("stream::observe(\"hello\")");
    assert!(
        result.is_ok(),
        "stream::observe with string should work: {:?}",
        result
    );

    match result {
        Ok(Value::Cursor(_)) => {}
        other => panic!("Expected Cursor, got: {:?}", other),
    }
}

#[test]
fn test_stream_observe_with_branch_id() {
    let result = eval_ok("stream::observe([1, 2, 3], \"my-branch\")");
    assert!(
        result.is_ok(),
        "stream::observe with branch_id should work: {:?}",
        result
    );

    match result {
        Ok(Value::Cursor(c)) => {
            assert_eq!(c.branch_id.as_str(), "my-branch", "branch_id should be set");
        }
        other => panic!("Expected Cursor with branch_id, got: {:?}", other),
    }
}

// ============================================================================
// Cursor module tests
// ============================================================================

#[test]
fn test_cursor_advance() {
    let result = eval_ok(
        "
        let c = stream::observe([1, 2, 3])
        let c2 = cursor::advance(c, 1)
        cursor::position(c2)
    ",
    );
    assert!(result.is_ok(), "cursor::advance should work: {:?}", result);

    match result {
        Ok(Value::Int(n)) => {
            assert_eq!(
                n, 1,
                "cursor::advance(c, 1) should move to position 1, got: {}",
                n
            );
        }
        other => panic!("Expected Int position, got: {:?}", other),
    }
}

#[test]
fn test_cursor_rewind() {
    let result = eval_ok(
        "
        let c = stream::observe([1, 2, 3])
        let c2 = cursor::advance(c, 2)
        let c3 = cursor::rewind(c2, 1)
        cursor::position(c3)
    ",
    );
    assert!(result.is_ok(), "cursor::rewind should work: {:?}", result);

    match result {
        Ok(Value::Int(n)) => {
            assert_eq!(n, 1, "cursor::rewind should move to position 1, got: {}", n);
        }
        other => panic!("Expected Int position, got: {:?}", other),
    }
}

#[test]
fn test_cursor_position_start() {
    let result = eval_ok(
        "
        let c = stream::observe([1, 2, 3])
        cursor::position(c)
    ",
    );
    assert!(result.is_ok(), "cursor::position should work: {:?}", result);

    match result {
        Ok(Value::Int(n)) => {
            assert_eq!(n, 0, "cursor::position at start should be 0, got: {}", n);
        }
        other => panic!("Expected Int position, got: {:?}", other),
    }
}

#[test]
fn test_cursor_cow_independence() {
    // Verify that advancing one cursor doesn't affect the original
    let result = eval_ok(
        "let c = stream::observe([1, 2, 3]); let c2 = cursor::advance(c, 1); [cursor::position(c), cursor::position(c2)]",
    );
    if let Err(e) = &result {
        eprintln!("test_cursor_cow_independence Error: {}", e);
    }
    assert!(
        result.is_ok(),
        "cursor CoW independence should work: {:?}",
        result
    );

    match result {
        Ok(Value::List(items)) => {
            assert_eq!(items.len(), 2, "Should have 2 positions");
            assert_eq!(
                items[0],
                Value::Int(0),
                "Original cursor should be at position 0"
            );
            assert_eq!(
                items[1],
                Value::Int(1),
                "Advanced cursor should be at position 1"
            );
        }
        other => panic!("Expected List of positions, got: {:?}", other),
    }
}

// ============================================================================
// Error cases
// ============================================================================

#[test]
fn test_cursor_advance_requires_cursor() {
    eval_err("cursor::advance([1,2,3], 1)", "requires cursor");
}

#[test]
fn test_cursor_advance_requires_int() {
    eval_err(
        "cursor::advance(stream::observe([1,2,3]), \"not-a-number\")",
        "integer",
    );
}

#[test]
fn test_cursor_rewind_requires_cursor() {
    eval_err("cursor::rewind([1,2,3], 1)", "requires cursor");
}

#[test]
fn test_cursor_position_requires_cursor() {
    eval_err("cursor::position([1,2,3])", "requires cursor");
}

// ============================================================================
// Integration tests: RLM-style usage
// ============================================================================

#[test]
fn test_rlm_style_cursor_processing() {
    // Test the pattern from RLM: observe -> advance -> process
    let result = eval_ok(
        "
        let data = [1, 2, 3, 4, 5]
        let cursor = stream::observe(data)
        let cursor2 = cursor::advance(cursor, 2)
        cursor::position(cursor2)
    ",
    );
    assert!(
        result.is_ok(),
        "RLM-style cursor processing should work: {:?}",
        result
    );

    match result {
        Ok(Value::Int(n)) => {
            assert_eq!(n, 2, "Should be at position 2 after advancing by 2");
        }
        other => panic!("Expected Int position, got: {:?}", other),
    }
}

#[test]
fn test_multiple_cursors_same_source() {
    // Multiple cursors on the same data should be independent
    let result = eval_ok(
        "let data = [1, 2, 3]; let c1 = stream::observe(data); let c2 = stream::observe(data); let c1_adv = cursor::advance(c1, 1); let c2_adv = cursor::advance(c2, 2); [cursor::position(c1), cursor::position(c1_adv), cursor::position(c2_adv)]",
    );
    assert!(result.is_ok(), "Multiple cursors should work: {:?}", result);

    match result {
        Ok(Value::List(items)) => {
            assert_eq!(items[0], Value::Int(0), "Original c1 should be at 0");
            assert_eq!(items[1], Value::Int(1), "Advanced c1 should be at 1");
            assert_eq!(items[2], Value::Int(2), "Advanced c2 should be at 2");
        }
        other => panic!("Expected List of positions, got: {:?}", other),
    }
}
