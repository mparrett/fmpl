//! LLM integration errors

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("JSON serialization/deserialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Provider API error: {0}")]
    Api(String),

    #[error("No API key configured for provider")]
    MissingApiKey,

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
}

pub type Result<T> = std::result::Result<T, LlmError>;
