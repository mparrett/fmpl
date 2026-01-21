//! FMPL LLM Integration
//!
//! Provider abstraction for LLM backends (Ollama, Anthropic, etc.)

pub mod error;
pub mod provider;

pub use error::LlmError;
pub use provider::{AnthropicProvider, LlmMessage, LlmProvider, LlmResponse, OllamaProvider};
