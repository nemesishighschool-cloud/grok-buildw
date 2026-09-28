//! NemApi Provider Integration for Grok Build
//!
//! This crate REPLACES the official xAI API integration with NemApi proxy support.
//! It provides a unified interface for all AI providers through NemApi's OpenAI-compatible endpoint.
//!
//! # Architecture
//!
//! The integration works as follows:
//! 1. Grok Build sends requests to NemApi proxy (default: http://127.0.0.1:8090/v1)
//! 2. NemApi routes to the appropriate provider (Gemini, Claude, Qwen, DeepSeek, etc.)
//! 3. NemApi uses browser automation to interact with provider web interfaces
//! 4. Responses are returned in OpenAI-compatible format
//!
//! # Key Features
//!
//! - **Multi-Provider Support**: Access all providers through a single endpoint
//! - **Browser Automation**: NemApi handles web-based providers via browser extension
//! - **OpenAI Compatibility**: Uses standard OpenAI API format
//! - **Streaming Support**: Full SSE streaming for responses
//! - **Tool Calls**: Supports function/tool calling
//! - **Customizable**: Configurable base URL, API keys, and provider settings
//!
//! # NemApi-Specific Behavior
//!
//! NemApi implements a unique conversation management approach:
//! - **Server-side context**: The browser extension maintains conversation state
//! - **First request**: Includes system prompt + all messages + tools
//! - **Subsequent requests**: Send ONLY the latest user message
//! - **No anonymous requests**: Only user prompts are sent, no suggestions or unsolicited content
//!
//! # Usage
//!
//! ```rust,no_run
//! use xai_grok_nemapi_provider::{NemApiClient, NemApiConfig, NemApiSamplingClient};
//!
//! // Create a client with default configuration
//! let config = NemApiConfig::default()
//!     .with_default_provider("gemini")
//!     .with_default_model("gemini-chat");
//!
//! let client = NemApiClient::new(config)?;
//!
//! // Or use the sampling client adapter
//! let sampling_client = NemApiSamplingClient::new(config)?;
//! ```

#![deny(clippy::indexing_slicing)]
#![warn(missing_docs)]

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod parser;
pub mod provider;
pub mod sampler_adapter;

// Re-export main types
pub use client::NemApiClient;
pub use config::NemApiConfig;
pub use error::NemApiError;
pub use models::{get_models_for_slash_command, get_nemapi_models_for_selection, is_canonical_nemapi_model, get_default_nemapi_model};
pub use parser::NemApiResponseParser;
pub use provider::NemApiProvider;
pub use sampler_adapter::{
    create_default_nemapi_sampling_client, create_nemapi_sampling_client, NemApiSamplingClient,
};

/// Default NemApi base URL
pub const DEFAULT_NEMAPI_BASE_URL: &str = "http://127.0.0.1:8090/v1";

/// Default NemApi API key prefix (for authentication)
pub const NEMAPI_TOKEN_PREFIX: &str = "nemapi-token";

/// Provider names supported by NemApi
/// Each provider has exactly ONE canonical model name (the -chat suffix)
pub const NEMAPI_PROVIDERS: &[&str] = &[
    "deepseek",
    "qwen", 
    "claude",
    "gemini",
    "chatgpt",
    "kimi",
    "zai",
];

/// Canonical model names for NemApi providers
/// These are the ONLY 7 model names that should be used - no fictional names
/// Each provider has exactly one canonical model with -chat suffix
pub const NEMAPI_CANONICAL_MODELS: &[&str] = &[
    "deepseek-chat",
    "qwen-chat",
    "claude-chat",
    "gemini-chat",
    "gpt-chat",
    "kimi-chat",
    "glm-chat",
];

/// Model aliases for NemApi providers (for backward compatibility only)
/// These aliases map to the canonical models above
/// When using / command to select models, ONLY the 7 canonical models should be shown
pub const NEMAPI_MODEL_ALIASES: &[&str] = &[
    // DeepSeek aliases
    "deepseek-coder", "deepseek-v3", "deepseek-r1",
    // Qwen aliases
    "qwen-plus", "qwen2.5-plus", "qwen3-coder-plus", "qwen-max",
    // Claude aliases
    "claude-sonnet", "claude-3-sonnet", "claude-3-haiku",
    // Gemini aliases
    "gemini-2.5-flash", "gemini-2.0-flash", "gemini-pro", "gemini-flash", "flash",
    // ChatGPT aliases
    "gpt-4", "gpt-4o", "gpt-4.1", "gpt-5", "gpt-3.5-turbo", "o1", "o3",
    // Kimi aliases
    "kimi-k2", "kimi-k3", "moonshot",
    // Z.ai / GLM aliases
    "glm-4", "glm-5", "zai-chat", "zai", "z.ai", "chatglm",
];

/// Create a NemApi sampling client with gemini-chat as default for testing
pub fn create_gemini_test_client() -> Result<NemApiSamplingClient, NemApiError> {
    let config = NemApiConfig::default()
        .with_default_provider("gemini")
        .with_default_model("gemini-chat");
    NemApiSamplingClient::new(config)
}

/// Create a NemApi client with gemini-chat as default for testing
pub fn create_gemini_test_nemapi_client() -> Result<NemApiClient, NemApiError> {
    let config = NemApiConfig::default()
        .with_default_provider("gemini")
        .with_default_model("gemini-chat");
    NemApiClient::new(config)
}
