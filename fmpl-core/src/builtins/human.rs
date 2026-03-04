//! human built-in for human-in-the-loop workflows.
//!
//! Provides `human.approve()` for requesting human approval of actions.

use crate::error::{Error, Result};
use crate::stream::{StreamEvent, StreamHandle, StreamSource, next_id};
use crate::value::Value;
use smol_str::SmolStr;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::mpsc;

/// The human built-in object.
pub struct HumanBuiltin;

impl HumanBuiltin {
    /// Request human approval for an action.
    ///
    /// Returns an async stream that yields:
    /// - `%{approved: true}` if user approves
    /// - `%{denied: reason}` if user denies with an optional reason
    ///
    /// Arguments:
    /// - request: Map describing the action to approve
    ///   - Required: `action` (string) - description of action
    ///   - Optional: `details` (string) - additional context
    ///   - Optional: `timeout_ms` (int) - approval timeout in milliseconds
    pub fn approve(request: &Value, _handle: &Handle) -> Result<Value> {
        // Extract action description from request
        let (action, details, _timeout_ms) = match request {
            Value::Map(map) => {
                let action = map
                    .get("action")
                    .and_then(|v| match v {
                        Value::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        Error::Runtime(
                            "human.approve requires 'action' string in request".to_string(),
                        )
                    })?;

                let details = map.get("details").and_then(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    _ => None,
                });

                let timeout_ms = map
                    .get("timeout_ms")
                    .and_then(|v| match v {
                        Value::Int(i) => Some(*i as u64),
                        _ => None,
                    })
                    .unwrap_or(30000); // Default 30 second timeout

                (action, details, timeout_ms)
            }
            Value::String(s) => {
                // Allow simple string as action description
                (s.clone(), None, 30000)
            }
            _ => {
                return Err(Error::Runtime(
                    "human.approve requires string or map request".to_string(),
                ));
            }
        };

        // Create approval request metadata
        let mut request_map = HashMap::new();
        request_map.insert(SmolStr::new("action"), Value::String(action.clone()));
        if let Some(d) = details {
            request_map.insert(SmolStr::new("details"), Value::String(d));
        }

        // For now, create an async stream that sends the approval request
        // The actual UI integration will consume this stream
        let (tx, rx) = mpsc::channel(1);

        // Store the approval request in thread-local or global state
        // so the UI layer can access it
        // For now, we'll just queue it for the UI to handle
        let req_id = next_id();
        APPROVAL_QUEUE.with(|q| {
            q.lock().unwrap().push(ApprovalRequest {
                id: req_id,
                action: action.to_string(),
                request: Value::Map(Arc::new(request_map.clone())),
                tx: Arc::new(tokio::sync::Mutex::new(Some(tx))),
            });
        });

        // Serialize request details as JSON for durability
        let details_json = serde_json::to_string(&request_map).unwrap_or_else(|_| String::new());

        let source_meta = StreamSource::HumanApproval {
            action: action.clone(),
            details: SmolStr::new(details_json),
            buffer: None,
        };
        let stream = StreamHandle::with_source(rx, next_id(), source_meta);
        let source = Value::AsyncStream(Arc::new(std::sync::Mutex::new(stream)));

        let result: HashMap<SmolStr, Value> = [
            (SmolStr::new("source"), source),
            (SmolStr::new("sink"), Value::Null),
        ]
        .into_iter()
        .collect();

        Ok(Value::Map(Arc::new(result)))
    }
}

/// Approval request queued for UI processing
#[derive(Clone)]
pub struct ApprovalRequest {
    pub id: u64,
    pub action: String,
    pub request: Value,
    pub tx: Arc<tokio::sync::Mutex<Option<mpsc::Sender<StreamEvent>>>>,
}

thread_local! {
    /// Queue of pending approval requests
    pub static APPROVAL_QUEUE: Arc<std::sync::Mutex<Vec<ApprovalRequest>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
}
