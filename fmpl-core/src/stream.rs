//! Async stream types for FMPL.

use crate::value::Value;
use tokio::sync::mpsc;

/// Event emitted by an async stream.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Intermediate data value.
    Data(Value),
    /// Terminal success with final value.
    Ok(Value),
    /// Terminal failure with error.
    Err(Value),
}

/// Handle to an async stream (source).
#[derive(Debug)]
pub struct StreamHandle {
    pub(crate) receiver: mpsc::Receiver<StreamEvent>,
    pub(crate) id: u64,
}

impl StreamHandle {
    /// Create a new stream handle.
    pub fn new(receiver: mpsc::Receiver<StreamEvent>, id: u64) -> Self {
        Self { receiver, id }
    }

    /// Get the stream ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Receive the next event (blocking).
    pub fn recv_blocking(&mut self) -> Option<StreamEvent> {
        self.receiver.try_recv().ok()
    }
}

/// Handle to a sink (destination for stream values).
#[derive(Debug, Clone)]
pub struct SinkHandle {
    pub(crate) sender: mpsc::Sender<Value>,
    pub(crate) id: u64,
}

impl SinkHandle {
    /// Create a new sink handle.
    pub fn new(sender: mpsc::Sender<Value>, id: u64) -> Self {
        Self { sender, id }
    }

    /// Get the sink ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Send a value to the sink.
    pub fn send_blocking(&self, value: Value) -> Result<(), Value> {
        self.sender.try_send(value).map_err(|e| match e {
            mpsc::error::TrySendError::Full(v) => v,
            mpsc::error::TrySendError::Closed(v) => v,
        })
    }
}

/// Counter for generating unique stream/sink IDs.
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Generate a unique ID for a stream or sink.
pub fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// Create a mock stream for testing.
pub fn mock_stream(events: Vec<StreamEvent>) -> StreamHandle {
    let (tx, rx) = mpsc::channel(events.len() + 1);
    for event in events {
        let _ = tx.try_send(event);
    }
    StreamHandle::new(rx, next_id())
}
