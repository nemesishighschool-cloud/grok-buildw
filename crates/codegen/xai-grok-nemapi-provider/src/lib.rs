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
pub mod parser;
pub mod provider;
pub mod sampler_adapter;

// Re-export main types
pub use client::NemApiClient;
pub use config::NemApiConfig;
pub use error::NemApiError;
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
pub const NEMAPI_PROVIDERS: &[&str] = &[
    "deepseek",
    "qwen",
    "claude",
    "gemini",
    "chatgpt",
    "kimi",
    "zai",
];

/// Model aliases for NemApi providers
pub const NEMAPI_MODEL_ALIASES: &[&str] = &[
    // DeepSeek
    "deepseek-chat",
    "deepseek-coder",
    "deepseek-v3",
    "deepseek-r1",
    // Qwen
    "qwen-chat",
    "qwen-plus",
    "qwen2.5-plus",
    "qwen3-coder-plus",
    // Claude
    "claude-chat",
    "claude-sonnet",
    "claude-3-sonnet",
    "claude-3-haiku",
    // Gemini
    "gemini-chat",
    "gemini-2.5-flash",
    "gemini-pro",
    // ChatGPT
    "gpt-chat",
    "gpt-4",
    "gpt-4o",
    "gpt-5",
    // Kimi
    "kimi-chat",
    "kimi-k2",
    "kimi-k3",
    // Z.ai / GLM
    "glm-chat",
    "glm-4",
    "glm-5",
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
