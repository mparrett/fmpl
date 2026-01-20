//! curl built-in for HTTP and other URL-based protocols.

use crate::error::Result;
use crate::stream::{StreamEvent, StreamHandle, StreamSource, next_id};
use crate::value::Value;
use smol_str::SmolStr;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::mpsc;

/// The curl built-in object.
pub struct CurlBuiltin;

impl CurlBuiltin {
    /// Perform an HTTP GET request.
    pub fn get(url: &str, handle: &Handle) -> Result<Value> {
        let url = url.to_string();
        let (tx, rx) = mpsc::channel(32);

        let url_for_task = url.clone();
        handle.spawn(async move {
            match Self::do_get(&url_for_task).await {
                Ok(body) => {
                    let _ = tx
                        .send(StreamEvent::Ok(Value::String(SmolStr::new(body))))
                        .await;
                }
                Err(e) => {
                    let error_map: HashMap<SmolStr, Value> =
                        [(SmolStr::new("message"), Value::String(SmolStr::new(e)))]
                            .into_iter()
                            .collect();
                    let _ = tx
                        .send(StreamEvent::Err(Value::Map(Arc::new(error_map))))
                        .await;
                }
            }
        });

        let source_meta = StreamSource::HttpGet {
            url: SmolStr::new(url),
            headers: HashMap::new(),
        };
        let stream = StreamHandle::with_source(rx, next_id(), source_meta);
        let source = Value::AsyncStream(Arc::new(std::sync::Mutex::new(stream)));

        // Return %{source: stream, sink: null}
        let result: HashMap<SmolStr, Value> = [
            (SmolStr::new("source"), source),
            (SmolStr::new("sink"), Value::Null),
        ]
        .into_iter()
        .collect();

        Ok(Value::Map(Arc::new(result)))
    }

    /// Perform an HTTP POST request.
    pub fn post(url: &str, body: &str, handle: &Handle) -> Result<Value> {
        let url = url.to_string();
        let body = body.to_string();
        let (tx, rx) = mpsc::channel(32);

        let url_clone = url.clone();
        let body_clone = body.clone();
        handle.spawn(async move {
            match Self::do_post(&url_clone, &body_clone).await {
                Ok(response) => {
                    let _ = tx
                        .send(StreamEvent::Ok(Value::String(SmolStr::new(response))))
                        .await;
                }
                Err(e) => {
                    let error_map: HashMap<SmolStr, Value> =
                        [(SmolStr::new("message"), Value::String(SmolStr::new(e)))]
                            .into_iter()
                            .collect();
                    let _ = tx
                        .send(StreamEvent::Err(Value::Map(Arc::new(error_map))))
                        .await;
                }
            }
        });

        let source_meta = StreamSource::HttpPost {
            url: SmolStr::new(url),
            body: SmolStr::new(body),
            headers: HashMap::new(),
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

    async fn do_get(url: &str) -> std::result::Result<String, String> {
        let url = url.to_string();
        tokio::task::spawn_blocking(move || {
            let mut easy = curl::easy::Easy::new();
            easy.url(&url).map_err(|e| e.to_string())?;

            let mut response = Vec::new();
            {
                let mut transfer = easy.transfer();
                transfer
                    .write_function(|data| {
                        response.extend_from_slice(data);
                        Ok(data.len())
                    })
                    .map_err(|e| e.to_string())?;
                transfer.perform().map_err(|e| e.to_string())?;
            }

            String::from_utf8(response).map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())?
    }

    async fn do_post(url: &str, body: &str) -> std::result::Result<String, String> {
        let url = url.to_string();
        let body = body.to_string();
        tokio::task::spawn_blocking(move || {
            let mut easy = curl::easy::Easy::new();
            easy.url(&url).map_err(|e| e.to_string())?;
            easy.post(true).map_err(|e| e.to_string())?;
            easy.post_fields_copy(body.as_bytes())
                .map_err(|e| e.to_string())?;

            let mut response = Vec::new();
            {
                let mut transfer = easy.transfer();
                transfer
                    .write_function(|data| {
                        response.extend_from_slice(data);
                        Ok(data.len())
                    })
                    .map_err(|e| e.to_string())?;
                transfer.perform().map_err(|e| e.to_string())?;
            }

            String::from_utf8(response).map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())?
    }
}
