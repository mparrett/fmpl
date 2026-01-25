//! Async stream types for FMPL.
//!
//! Streams support serialization via [`StreamSource`] metadata, enabling
//! durable suspension and resumption of async operations across process restarts.

// Allow large error type - Value is intentionally large to support all FMPL types.
// Boxing would add allocation overhead for a rarely-used error path.
#![allow(clippy::result_large_err)]

use crate::value::Value;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::collections::HashMap;
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
    /// Stream completed successfully (no value).
    Done,
}

/// Reference to a persisted stream buffer in Fjall storage.
///
/// Allows backtracking and replay of historical stream data during
/// incremental parsing or error recovery.
#[derive(
    Debug, Clone, PartialEq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
pub struct StreamBuffer {
    /// Fjall partition name where buffer is stored.
    pub partition: SmolStr,
    /// Key prefix for buffer entries.
    pub key_prefix: SmolStr,
    /// Current read position in the buffer (for resumption).
    pub position: usize,
    /// Total bytes received and persisted so far.
    pub bytes_received: usize,
}

/// Metadata describing how to recreate a stream for durable suspension.
///
/// When a stream is serialized (e.g., for ParseState persistence), this
/// captures enough information to reconnect or recreate the stream on resume.
/// Each variant optionally includes a reference to a persisted buffer for
/// backtracking support during incremental parsing.
#[derive(
    Debug, Clone, PartialEq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[rkyv(serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, <__S as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source))]
#[rkyv(deserialize_bounds(<__D as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source))]
#[rkyv(bytecheck(bounds(__C: rkyv::validation::ArchiveContext)))]
pub enum StreamSource {
    /// HTTP GET request - can be retried/resumed.
    HttpGet {
        url: SmolStr,
        headers: HashMap<SmolStr, SmolStr>,
        /// Reference to persisted stream buffer for backtracking.
        buffer: Option<StreamBuffer>,
    },
    /// HTTP POST request - may not be safely retryable.
    HttpPost {
        url: SmolStr,
        body: SmolStr,
        headers: HashMap<SmolStr, SmolStr>,
        /// Reference to persisted stream buffer for backtracking.
        buffer: Option<StreamBuffer>,
    },
    /// WebSocket connection.
    WebSocket {
        url: SmolStr,
        /// Reference to persisted stream buffer for backtracking.
        buffer: Option<StreamBuffer>,
    },
    /// File stream - can be resumed from position.
    File {
        path: SmolStr,
        position: u64,
        /// Reference to persisted stream buffer for backtracking.
        buffer: Option<StreamBuffer>,
    },
    /// LLM completion stream - needs conversation context to resume.
    LlmCompletion {
        model: SmolStr,
        /// Serialized conversation context for resumption.
        context: SmolStr,
        /// Reference to persisted stream buffer for backtracking.
        buffer: Option<StreamBuffer>,
    },
    /// In-memory mock stream (for testing) - cannot be resumed.
    Mock,
    /// Stream created programmatically - cannot be resumed.
    Ephemeral,
    /// Disconnected placeholder after failed resume attempt.
    Disconnected {
        #[rkyv(omit_bounds)]
        original: Box<StreamSource>,
        reason: SmolStr,
    },
}

impl StreamSource {
    /// Check if this stream source can potentially be resumed.
    pub fn is_resumable(&self) -> bool {
        match self {
            StreamSource::HttpGet { .. } => true,
            StreamSource::WebSocket { .. } => true,
            StreamSource::File { .. } => true,
            StreamSource::LlmCompletion { .. } => true,
            StreamSource::HttpPost { .. } => false, // POST may not be idempotent
            StreamSource::Mock => false,
            StreamSource::Ephemeral => false,
            StreamSource::Disconnected { .. } => false,
        }
    }

    /// Get the stream buffer reference, if any.
    pub fn buffer(&self) -> Option<&StreamBuffer> {
        match self {
            StreamSource::HttpGet { buffer, .. } => buffer.as_ref(),
            StreamSource::HttpPost { buffer, .. } => buffer.as_ref(),
            StreamSource::WebSocket { buffer, .. } => buffer.as_ref(),
            StreamSource::File { buffer, .. } => buffer.as_ref(),
            StreamSource::LlmCompletion { buffer, .. } => buffer.as_ref(),
            StreamSource::Mock => None,
            StreamSource::Ephemeral => None,
            StreamSource::Disconnected { .. } => None,
        }
    }

    /// Set or update the stream buffer reference.
    pub fn with_buffer(self, buffer: StreamBuffer) -> Self {
        match self {
            StreamSource::HttpGet { url, headers, .. } => StreamSource::HttpGet {
                url,
                headers,
                buffer: Some(buffer),
            },
            StreamSource::HttpPost {
                url, body, headers, ..
            } => StreamSource::HttpPost {
                url,
                body,
                headers,
                buffer: Some(buffer),
            },
            StreamSource::WebSocket { url, .. } => StreamSource::WebSocket {
                url,
                buffer: Some(buffer),
            },
            StreamSource::File { path, position, .. } => StreamSource::File {
                path,
                position,
                buffer: Some(buffer),
            },
            StreamSource::LlmCompletion { model, context, .. } => StreamSource::LlmCompletion {
                model,
                context,
                buffer: Some(buffer),
            },
            // These variants don't support buffers
            other => other,
        }
    }
}

/// Handle to an async stream (source).
#[derive(Debug)]
pub struct StreamHandle {
    pub(crate) receiver: mpsc::Receiver<StreamEvent>,
    pub(crate) id: u64,
    /// Source metadata for reconnection on resume.
    pub(crate) source: StreamSource,
}

impl StreamHandle {
    /// Create a new stream handle with source metadata.
    pub fn new(receiver: mpsc::Receiver<StreamEvent>, id: u64) -> Self {
        Self {
            receiver,
            id,
            source: StreamSource::Ephemeral,
        }
    }

    /// Create a stream handle with explicit source metadata.
    pub fn with_source(
        receiver: mpsc::Receiver<StreamEvent>,
        id: u64,
        source: StreamSource,
    ) -> Self {
        Self {
            receiver,
            id,
            source,
        }
    }

    /// Get the stream ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get the source metadata for serialization.
    pub fn source(&self) -> &StreamSource {
        &self.source
    }

    /// Receive the next event, blocking until available or channel closed.
    /// This uses a synchronous blocking wait for REPL/non-async contexts.
    pub fn recv_blocking(&mut self) -> Option<StreamEvent> {
        // Use blocking recv with timeout to avoid deadlocks
        // In async contexts, use the receiver directly with proper await
        use std::time::Duration;

        // Try non-blocking first
        if let Ok(event) = self.receiver.try_recv() {
            return Some(event);
        }

        // Fall back to blocking recv with reasonable timeout
        // Note: tokio mpsc doesn't have true blocking recv, so we poll
        let timeout = Duration::from_secs(30);
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            std::thread::sleep(Duration::from_millis(10));
            match self.receiver.try_recv() {
                Ok(event) => return Some(event),
                Err(mpsc::error::TryRecvError::Disconnected) => return None,
                Err(mpsc::error::TryRecvError::Empty) => continue,
            }
        }

        None // Timeout
    }
}

/// Metadata describing how to recreate a sink for durable suspension.
#[derive(
    Debug, Clone, PartialEq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[rkyv(serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, <__S as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source))]
#[rkyv(deserialize_bounds(<__D as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source))]
#[rkyv(bytecheck(bounds(__C: rkyv::validation::ArchiveContext)))]
pub enum SinkSource {
    /// HTTP response body sink.
    HttpResponse { request_id: SmolStr },
    /// WebSocket connection sink.
    WebSocket { url: SmolStr },
    /// File output sink.
    File { path: SmolStr },
    /// In-memory sink (for testing) - cannot be resumed.
    Mock,
    /// Sink created programmatically - cannot be resumed.
    Ephemeral,
    /// Disconnected placeholder after failed resume attempt.
    Disconnected {
        #[rkyv(omit_bounds)]
        original: Box<SinkSource>,
        reason: SmolStr,
    },
}

impl SinkSource {
    /// Check if this sink source can potentially be resumed.
    pub fn is_resumable(&self) -> bool {
        match self {
            SinkSource::WebSocket { .. } => true,
            SinkSource::File { .. } => true,
            SinkSource::HttpResponse { .. } => false, // Response already sent
            SinkSource::Mock => false,
            SinkSource::Ephemeral => false,
            SinkSource::Disconnected { .. } => false,
        }
    }
}

/// Handle to a sink (destination for stream values).
#[derive(Debug, Clone)]
pub struct SinkHandle {
    pub(crate) sender: mpsc::Sender<Value>,
    pub(crate) id: u64,
    /// Source metadata for reconnection on resume.
    pub(crate) source: SinkSource,
}

impl SinkHandle {
    /// Create a new sink handle.
    pub fn new(sender: mpsc::Sender<Value>, id: u64) -> Self {
        Self {
            sender,
            id,
            source: SinkSource::Ephemeral,
        }
    }

    /// Create a sink handle with explicit source metadata.
    pub fn with_source(sender: mpsc::Sender<Value>, id: u64, source: SinkSource) -> Self {
        Self { sender, id, source }
    }

    /// Get the sink ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get the source metadata for serialization.
    pub fn source(&self) -> &SinkSource {
        &self.source
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
    StreamHandle::with_source(rx, next_id(), StreamSource::Mock)
}
