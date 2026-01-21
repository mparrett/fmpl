//! curl built-in for HTTP and other URL-based protocols.

use crate::error::{Error, Result};
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
    /// Extract headers from optional options map.
    /// Returns empty HashMap if no headers provided or on error.
    fn extract_headers(options: Option<&Value>) -> Result<HashMap<SmolStr, SmolStr>> {
        match options {
            None => Ok(HashMap::new()),
            Some(Value::Map(map)) => {
                // Look for "headers" key in the options map
                if let Some(Value::Map(headers_map)) = map.get("headers") {
                    let mut headers = HashMap::new();
                    for (key, value) in headers_map.iter() {
                        if let Value::String(v) = value {
                            headers.insert(key.clone(), v.clone());
                        } else {
                            return Err(Error::Runtime(format!(
                                "Header value for '{}' must be string, got {:?}",
                                key, value
                            )));
                        }
                    }
                    Ok(headers)
                } else {
                    Ok(HashMap::new())
                }
            }
            Some(_) => Ok(HashMap::new()), // Non-map options, ignore
        }
    }

    /// Perform an HTTP GET request.
    ///
    /// Arguments:
    /// - url: Request URL (string)
    /// - options: Optional map with headers: %{headers: %{...}}
    pub fn get(url: &str, handle: &Handle, options: Option<&Value>) -> Result<Value> {
        let url = url.to_string();
        let headers = Self::extract_headers(options)?;
        let headers_for_async = headers.clone();

        let (tx, rx) = mpsc::channel(32);

        let url_for_task = url.clone();
        handle.spawn(async move {
            match Self::do_get(&url_for_task, &headers_for_async).await {
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
            headers,
            buffer: None,
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
    ///
    /// Arguments:
    /// - url: Request URL (string)
    /// - body: Request body (string)
    /// - options: Optional map with headers: %{headers: %{...}}
    pub fn post(url: &str, body: &str, handle: &Handle, options: Option<&Value>) -> Result<Value> {
        let url = url.to_string();
        let body = body.to_string();
        let headers = Self::extract_headers(options)?;
        let headers_for_async = headers.clone();

        let (tx, rx) = mpsc::channel(32);

        let url_clone = url.clone();
        let body_clone = body.clone();
        handle.spawn(async move {
            match Self::do_post(&url_clone, &body_clone, &headers_for_async).await {
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
            headers,
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

    async fn do_get(
        url: &str,
        headers: &HashMap<SmolStr, SmolStr>,
    ) -> std::result::Result<String, String> {
        let url = url.to_string();
        let headers = headers.clone();
        tokio::task::spawn_blocking(move || {
            let mut easy = curl::easy::Easy::new();
            easy.url(&url).map_err(|e| e.to_string())?;

            // Add headers
            let mut list = curl::easy::List::new();
            for (key, value) in &headers {
                let header = format!("{}: {}", key, value);
                list.append(&header).map_err(|e| e.to_string())?;
            }
            easy.http_headers(list).map_err(|e| e.to_string())?;

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

    async fn do_post(
        url: &str,
        body: &str,
        headers: &HashMap<SmolStr, SmolStr>,
    ) -> std::result::Result<String, String> {
        let url = url.to_string();
        let body = body.to_string();
        let headers = headers.clone();
        tokio::task::spawn_blocking(move || {
            let mut easy = curl::easy::Easy::new();
            easy.url(&url).map_err(|e| e.to_string())?;
            easy.post(true).map_err(|e| e.to_string())?;
            easy.post_fields_copy(body.as_bytes())
                .map_err(|e| e.to_string())?;

            // Add headers
            let mut list = curl::easy::List::new();
            for (key, value) in &headers {
                let header = format!("{}: {}", key, value);
                list.append(&header).map_err(|e| e.to_string())?;
            }
            easy.http_headers(list).map_err(|e| e.to_string())?;

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
