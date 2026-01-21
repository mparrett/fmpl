//! Tests for SSE (Server-Sent Events) parsing integration.

use fmpl_core::{Value, Vm, eval};

#[tokio::test]
async fn test_sse_parse_ollama_format() {
    let mut vm = Vm::new();

    let code = r#"
        let sse_text = "data: {\"response\": \"Hello\", \"done\": false}\n\ndata: {\"response\": \" world\", \"done\": false}\n\ndata: {\"response\": \"!\", \"done\": true}\n\n"
        let result = sse.parse(sse_text)
        result
    "#;

    let result = eval(&mut vm, code).unwrap();

    match result {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            // Verify first event
            if let Value::Map(map) = &items[0] {
                assert_eq!(map.get("response"), Some(&Value::String("Hello".into())));
                assert_eq!(map.get("done"), Some(&Value::Bool(false)));
            } else {
                panic!("Expected Map for first event, got {:?}", &items[0]);
            }
            // Verify second event
            if let Value::Map(map) = &items[1] {
                assert_eq!(map.get("response"), Some(&Value::String(" world".into())));
                assert_eq!(map.get("done"), Some(&Value::Bool(false)));
            } else {
                panic!("Expected Map for second event");
            }
            // Verify third event
            if let Value::Map(map) = &items[2] {
                assert_eq!(map.get("response"), Some(&Value::String("!".into())));
                assert_eq!(map.get("done"), Some(&Value::Bool(true)));
            } else {
                panic!("Expected Map for third event");
            }
        }
        other => panic!("Expected List, got {:?}", other),
    }
}

#[tokio::test]
async fn test_sse_parse_anthropic_format() {
    let mut vm = Vm::new();

    let code = r#"
        let sse_text = "data: {\"type\": \"content_block_delta\", \"delta\": {\"text\": \"Hello\"}, \"index\": 0}\n\n"
        let result = sse.parse(sse_text)
        result
    "#;

    let result = eval(&mut vm, code).unwrap();

    match result {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            if let Value::Map(map) = &items[0] {
                assert_eq!(
                    map.get("type"),
                    Some(&Value::String("content_block_delta".into()))
                );
                // Check nested delta field
                if let Some(Value::Map(delta)) = map.get("delta") {
                    assert_eq!(delta.get("text"), Some(&Value::String("Hello".into())));
                } else {
                    panic!("Expected delta to be a Map");
                }
            } else {
                panic!("Expected Map for event, got {:?}", &items[0]);
            }
        }
        other => panic!("Expected List, got {:?}", other),
    }
}

#[tokio::test]
async fn test_sse_parse_empty_input() {
    let mut vm = Vm::new();

    let code = r#"
        let result = sse.parse("")
        result
    "#;

    let result = eval(&mut vm, code).unwrap();

    match result {
        Value::List(items) => {
            assert_eq!(items.len(), 0);
        }
        other => panic!("Expected empty List, got {:?}", other),
    }
}

#[tokio::test]
async fn test_sse_parse_handles_comments() {
    let mut vm = Vm::new();

    let code = r#"
        let sse_text = ": this is a comment\ndata: {\"text\": \"test\"}\n\n"
        let result = sse.parse(sse_text)
        result
    "#;

    let result = eval(&mut vm, code).unwrap();

    match result {
        Value::List(items) => {
            assert_eq!(items.len(), 1); // Only the data line, not the comment
            if let Value::Map(map) = &items[0] {
                assert_eq!(map.get("text"), Some(&Value::String("test".into())));
            } else {
                panic!("Expected Map");
            }
        }
        other => panic!("Expected List with 1 item, got {:?}", other),
    }
}

#[tokio::test]
async fn test_sse_parse_with_multiple_fields() {
    let mut vm = Vm::new();

    // Test SSE event with multiple JSON fields
    let code = r#"
        let sse_text = "data: {\"text\": \"test\", \"index\": 0, \"done\": false}\n\n"
        let result = sse.parse(sse_text)
        result
    "#;

    let result = eval(&mut vm, code).unwrap();

    match result {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            if let Value::Map(map) = &items[0] {
                assert_eq!(map.get("text"), Some(&Value::String("test".into())));
                assert_eq!(map.get("index"), Some(&Value::Int(0)));
                assert_eq!(map.get("done"), Some(&Value::Bool(false)));
            } else {
                panic!("Expected Map");
            }
        }
        other => panic!("Expected List, got {:?}", other),
    }
}

#[tokio::test]
async fn test_sse_parse_no_final_newline() {
    let mut vm = Vm::new();

    // Test that last event is parsed even without trailing double-newline
    let code = r#"
        let sse_text = "data: {\"response\": \"test\"}\n\n"
        let result = sse.parse(sse_text)
        result
    "#;

    let result = eval(&mut vm, code).unwrap();

    match result {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
        }
        other => panic!("Expected List, got {:?}", other),
    }
}
