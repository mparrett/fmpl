//! LLM builtins for FMPL
//!
//! Provides `llm.chat()` and `llm.stream()` for interacting with LLM providers

use fmpl_llm::{AnthropicProvider, LlmMessage, LlmProvider, OllamaProvider};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global LLM provider registry
pub struct LlmRegistry {
    current_provider: Arc<RwLock<Box<dyn LlmProvider>>>,
}

impl LlmRegistry {
    pub fn new(provider: Box<dyn LlmProvider>) -> Self {
        Self {
            current_provider: Arc::new(RwLock::new(provider)),
        }
    }

    pub async fn set_provider(&self, provider: Box<dyn LlmProvider>) {
        let mut p = self.current_provider.write().await;
        *p = provider;
    }

    pub async fn chat(&self, prompt: &str) -> Result<String, String> {
        let provider = self.current_provider.read().await;
        let messages = vec![LlmMessage::user(prompt)];
        provider
            .chat(&messages)
            .await
            .map(|r| r.content)
            .map_err(|e| e.to_string())
    }
}

/// Default LLM registry instance (lazy-initialized)
static LLM_REGISTRY: std::sync::OnceLock<std::sync::Mutex<Arc<LlmRegistry>>> =
    std::sync::OnceLock::new();

fn get_registry() -> Arc<LlmRegistry> {
    LLM_REGISTRY
        .get_or_init(|| {
            // Default to Ollama on localhost
            let provider = OllamaProvider::localhost("llama3.2")
                .unwrap_or_else(|_| OllamaProvider::localhost("llama2").unwrap());

            std::sync::Mutex::new(Arc::new(LlmRegistry::new(Box::new(provider))))
        })
        .lock()
        .unwrap()
        .clone()
}

/// Initialize LLM builtin with a provider
pub fn init_llm(provider_name: &str, model: &str) -> Result<(), String> {
    let registry = get_registry();

    let provider: Box<dyn LlmProvider> = match provider_name {
        "ollama" => Box::new(OllamaProvider::localhost(model).map_err(|e| e.to_string())?),
        "anthropic" => Box::new(AnthropicProvider::from_env(model).map_err(|e| e.to_string())?),
        _ => return Err(format!("Unknown provider: {}", provider_name)),
    };

    // Set provider (need runtime)
    let rt =
        tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

    rt.block_on(async {
        registry.set_provider(provider).await;
    });

    Ok(())
}

/// LLM chat completion (blocking)
pub fn llm_chat(prompt: &str) -> Result<String, String> {
    let registry = get_registry();

    let rt =
        tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

    rt.block_on(async { registry.chat(prompt).await })
}
