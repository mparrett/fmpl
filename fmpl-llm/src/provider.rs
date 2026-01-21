//! LLM provider abstraction

use crate::error::{LlmError, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role")]
pub enum LlmMessage {
    #[serde(rename = "user")]
    User { content: String },
    #[serde(rename = "assistant")]
    Assistant { content: String },
    #[serde(rename = "system")]
    System { content: String },
}

impl LlmMessage {
    pub fn user(content: impl Into<String>) -> Self {
        LlmMessage::User {
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        LlmMessage::Assistant {
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        LlmMessage::System {
            content: content.into(),
        }
    }
}

/// LLM response
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub finish_reason: Option<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Provider configuration
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub timeout: Duration,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            api_key: None,
            model: "llama3.2".to_string(),
            timeout: Duration::from_secs(30),
        }
    }
}

/// LLM provider trait
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    /// Chat completion (non-streaming)
    async fn chat(&self, messages: &[LlmMessage]) -> Result<LlmResponse>;

    /// Stream chat completion
    async fn chat_stream(
        &self,
        messages: &[LlmMessage],
    ) -> Result<tokio_stream::wrappers::ReceiverStream<Result<String>>>;
}

/// Ollama provider (local LLM)
pub struct OllamaProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let client = reqwest::Client::builder().timeout(config.timeout).build()?;

        Ok(Self { config, client })
    }

    pub fn localhost(model: impl Into<String>) -> Result<Self> {
        Self::new(ProviderConfig {
            base_url: "http://localhost:11434".to_string(),
            api_key: None,
            model: model.into(),
            timeout: Duration::from_secs(60),
        })
    }
}

#[async_trait::async_trait]
impl LlmProvider for OllamaProvider {
    async fn chat(&self, messages: &[LlmMessage]) -> Result<LlmResponse> {
        #[derive(Serialize)]
        struct OllamaRequest<'a> {
            model: &'a str,
            messages: &'a [LlmMessage],
            stream: bool,
        }

        #[derive(Deserialize)]
        struct OllamaResponse {
            message: OllamaMessage,
            done: bool,
        }

        #[derive(Deserialize)]
        struct OllamaMessage {
            content: String,
        }

        let req = OllamaRequest {
            model: &self.config.model,
            messages,
            stream: false,
        };

        let url = format!("{}/api/chat", self.config.base_url);
        let resp: OllamaResponse = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .json()
            .await?;

        Ok(LlmResponse {
            content: resp.message.content,
            finish_reason: if resp.done {
                Some("stop".to_string())
            } else {
                None
            },
            usage: None,
        })
    }

    async fn chat_stream(
        &self,
        messages: &[LlmMessage],
    ) -> Result<tokio_stream::wrappers::ReceiverStream<Result<String>>> {
        use futures::stream::{StreamExt, TryStreamExt};
        use tokio_stream::wrappers::ReceiverStream;

        let (tx, rx) = tokio::sync::mpsc::channel(64);

        let client = self.client.clone();
        let url = format!("{}/api/chat", self.config.base_url);
        let model = self.config.model.clone();
        let messages = messages.to_vec();

        tokio::spawn(async move {
            #[derive(Serialize)]
            struct OllamaRequest<'a> {
                model: &'a str,
                messages: &'a [LlmMessage],
                stream: bool,
            }

            let req = OllamaRequest {
                model: &model,
                messages: &messages,
                stream: true,
            };

            if let Ok(resp) = client.post(&url).json(&req).send().await {
                let mut stream = resp.bytes_stream();

                while let Some(chunk_result) = stream.next().await {
                    match chunk_result {
                        Ok(bytes) => {
                            // Parse each line as JSON (Ollama sends newline-delimited JSON)
                            if let Ok(text) = std::str::from_utf8(&bytes) {
                                for line in text.lines() {
                                    if let Ok(json) =
                                        serde_json::from_str::<serde_json::Value>(line)
                                    {
                                        if let Some(content) = json
                                            .get("message")
                                            .and_then(|m| m.get("content"))
                                            .and_then(|c| c.as_str())
                                        {
                                            let _ = tx.send(Ok(content.to_string())).await;
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Err(LlmError::Request(e))).await;
                            break;
                        }
                    }
                }
            }
        });

        Ok(ReceiverStream::new(rx))
    }
}

/// Anthropic Claude provider
pub struct AnthropicProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        if config.api_key.is_none() {
            return Err(LlmError::MissingApiKey);
        }

        let client = reqwest::Client::builder().timeout(config.timeout).build()?;

        Ok(Self { config, client })
    }

    pub fn from_env(model: impl Into<String>) -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| std::env::var("ZAI_API_KEY").ok())
            .ok_or(LlmError::MissingApiKey)?;

        Self::new(ProviderConfig {
            base_url: "https://api.anthropic.com".to_string(),
            api_key: Some(api_key),
            model: model.into(),
            timeout: Duration::from_secs(60),
        })
    }
}

#[async_trait::async_trait]
impl LlmProvider for AnthropicProvider {
    async fn chat(&self, messages: &[LlmMessage]) -> Result<LlmResponse> {
        #[derive(Serialize)]
        struct AnthropicRequest<'a> {
            model: &'a str,
            max_tokens: u32,
            messages: &'a [LlmMessage],
        }

        #[derive(Deserialize)]
        struct AnthropicResponse {
            content: Vec<AnthropicContent>,
            stop_reason: String,
            usage: AnthropicUsage,
        }

        #[derive(Deserialize)]
        struct AnthropicContent {
            text: String,
        }

        #[derive(Deserialize)]
        struct AnthropicUsage {
            input_tokens: u32,
            output_tokens: u32,
        }

        let req = AnthropicRequest {
            model: &self.config.model,
            max_tokens: 4096,
            messages,
        };

        let url = format!("{}/v1/messages", self.config.base_url);
        let api_key = self.config.api_key.as_ref().unwrap();

        let version =
            std::env::var("ANTHROPIC_VERSION").unwrap_or_else(|_| "2023-06-01".to_string());

        let resp: AnthropicResponse = self
            .client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", version)
            .header("content-type", "application/json")
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let content = resp
            .content
            .into_iter()
            .map(|c| c.text)
            .collect::<Vec<_>>()
            .join("");

        Ok(LlmResponse {
            content,
            finish_reason: Some(resp.stop_reason),
            usage: Some(Usage {
                prompt_tokens: resp.usage.input_tokens,
                completion_tokens: resp.usage.output_tokens,
                total_tokens: resp.usage.input_tokens + resp.usage.output_tokens,
            }),
        })
    }

    async fn chat_stream(
        &self,
        _messages: &[LlmMessage],
    ) -> Result<tokio_stream::wrappers::ReceiverStream<Result<String>>> {
        // TODO: Implement Anthropic SSE streaming
        Err(LlmError::Api(
            "Streaming not yet implemented for Anthropic".to_string(),
        ))
    }
}
