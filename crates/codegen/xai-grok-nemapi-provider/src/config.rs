//! Configuration for NemApi provider integration
//!
//! This module provides comprehensive configuration for the NemApi provider,
//! including support for multiple providers, authentication, and request settings.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use xai_grok_sampler::{AuthScheme, RequestCompression};
use xai_grok_sampling_types::{ApiBackend, ReasoningEffort, ReasoningSummary};

use crate::{error::Result, NEMAPI_TOKEN_PREFIX};

/// Configuration for NemApi provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NemApiConfig {
    /// Base URL for NemApi proxy (default: http://127.0.0.1:8090/v1)
    pub base_url: String,

    /// API key for NemApi authentication (format: nemapi-token{random})
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Enable API key authentication
    #[serde(default)]
    pub require_auth: bool,

    /// Default provider to use when not specified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_provider: Option<String>,

    /// Default model to use when not specified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,

    /// Provider-specific configurations
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub providers: IndexMap<String, ProviderConfig>,

    /// Auth scheme to use (Bearer or XApiKey)
    #[serde(default)]
    pub auth_scheme: AuthScheme,

    /// Enable request compression
    #[serde(default)]
    pub request_compression: RequestCompression,

    /// API backend type (responses, chat, messages)
    #[serde(default)]
    pub api_backend: ApiBackend,

    /// Enable streaming
    #[serde(default = "default_stream_enabled")]
    pub stream_enabled: bool,

    /// Enable fresh chat for each request
    #[serde(default)]
    pub fresh_chat: bool,

    /// Enable premium markdown formatting
    #[serde(default)]
    pub premium_md: bool,

    /// Auto-configuration enabled
    #[serde(default = "default_auto_config")]
    pub auto_config: bool,

    /// Context window size
    #[serde(default = "default_context_window")]
    pub context_window: u64,

    /// Max completion tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,

    /// Temperature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Top-p sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Reasoning effort
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,

    /// Reasoning summary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_summary: Option<ReasoningSummary>,

    /// Stream tool calls
    #[serde(default)]
    pub stream_tool_calls: bool,

    /// Extra headers to send with each request
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub extra_headers: IndexMap<String, String>,

    /// Query parameters to append to requests
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub query_params: IndexMap<String, String>,

    /// Environment variable mappings for headers
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub env_http_headers: IndexMap<String, String>,

    /// Timeout for requests in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Connection timeout in seconds
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_secs: u64,

    /// Whether to strip HTML entities from responses
    #[serde(default = "default_strip_html")]
    pub strip_html_entities: bool,

    /// Whether to remove control characters from responses
    #[serde(default = "default_remove_control_chars")]
    pub remove_control_chars: bool,

    /// Whether to clean UI chrome from responses
    #[serde(default = "default_clean_ui_chrome")]
    pub clean_ui_chrome: bool,

    /// Whether to strip thinking blocks from responses
    #[serde(default = "default_strip_thinking")]
    pub strip_thinking: bool,

    /// Maximum number of parse retries
    #[serde(default = "default_max_parse_retries")]
    pub max_parse_retries: usize,
}

fn default_stream_enabled() -> bool {
    true
}

fn default_auto_config() -> bool {
    true
}

fn default_context_window() -> u64 {
    128_000
}

fn default_timeout() -> u64 {
    120
}

fn default_connect_timeout() -> u64 {
    30
}

fn default_strip_html() -> bool {
    true
}

fn default_remove_control_chars() -> bool {
    true
}

fn default_clean_ui_chrome() -> bool {
    true
}

fn default_strip_thinking() -> bool {
    true
}

fn default_max_parse_retries() -> usize {
    3
}

impl Default for NemApiConfig {
    fn default() -> Self {
        Self {
            base_url: crate::DEFAULT_NEMAPI_BASE_URL.to_string(),
            api_key: None,
            require_auth: false,
            default_provider: Some("gemini".to_string()),
            default_model: Some("gemini-chat".to_string()),
            providers: IndexMap::new(),
            auth_scheme: AuthScheme::Bearer,
            request_compression: RequestCompression::None,
            api_backend: ApiBackend::ChatCompletions,
            stream_enabled: true,
            fresh_chat: true,
            premium_md: true,
            auto_config: true,
            context_window: 128_000,
            max_completion_tokens: None,
            temperature: None,
            top_p: None,
            reasoning_effort: None,
            reasoning_summary: None,
            stream_tool_calls: false,
            extra_headers: IndexMap::new(),
            query_params: IndexMap::new(),
            env_http_headers: IndexMap::new(),
            timeout_secs: 120,
            connect_timeout_secs: 30,
            strip_html_entities: true,
            remove_control_chars: true,
            clean_ui_chrome: true,
            strip_thinking: true,
            max_parse_retries: 3,
        }
    }
}

/// Configuration for a specific provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProviderConfig {
    /// Models available for this provider
    pub models: Vec<String>,

    /// Default model for this provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,

    /// API key for this specific provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Custom base URL for this provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    /// Enable for this provider
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Custom timeout for this provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,

    /// Custom headers for this provider
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub extra_headers: IndexMap<String, String>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            models: Vec::new(),
            default_model: None,
            api_key: None,
            base_url: None,
            enabled: true,
            timeout_secs: None,
            extra_headers: IndexMap::new(),
        }
    }
}

fn default_enabled() -> bool {
    true
}

impl NemApiConfig {
    /// Create a new NemApiConfig with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a NemApiConfig with a specific base URL
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            ..Self::default()
        }
    }

    /// Create a NemApiConfig with API key authentication
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            require_auth: true,
            ..Self::default()
        }
    }

    /// Set the default provider
    pub fn with_default_provider(mut self, provider: impl Into<String>) -> Self {
        self.default_provider = Some(provider.into());
        self
    }

    /// Set the default model
    pub fn with_default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = Some(model.into());
        self
    }

    /// Set the base URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Generate a new API key (for testing)
    pub fn generate_api_key() -> String {
        use rand::{distributions::Alphanumeric, Rng};
        let random_part: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(24)
            .map(char::from)
            .collect();
        format!("{}{}", NEMAPI_TOKEN_PREFIX, random_part)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate base URL
        if self.base_url.is_empty() {
            return Err(crate::error::NemApiError::Generic(
                "base_url cannot be empty".to_string(),
            ));
        }

        // Validate API key format if provided
        if let Some(ref api_key) = self.api_key {
            if !api_key.starts_with(NEMAPI_TOKEN_PREFIX) {
                return Err(crate::error::NemApiError::AuthenticationError(
                    format!(
                        "API key must start with '{}', got: {}",
                        NEMAPI_TOKEN_PREFIX, api_key
                    ),
                ));
            }
        }

        // Validate default provider
        if let Some(ref provider) = self.default_provider {
            if !crate::NEMAPI_PROVIDERS.contains(&provider.as_str()) {
                return Err(crate::error::NemApiError::ProviderNotConfigured(
                    provider.clone(),
                ));
            }
        }

        Ok(())
    }

    /// Get the effective base URL (from config or default)
    pub fn effective_base_url(&self) -> String {
        if self.base_url.is_empty() {
            crate::DEFAULT_NEMAPI_BASE_URL.to_string()
        } else {
            self.base_url.clone()
        }
    }

    /// Get the effective default provider
    pub fn effective_default_provider(&self) -> &str {
        self.default_provider
            .as_deref()
            .unwrap_or("gemini")
    }

    /// Get the effective default model
    pub fn effective_default_model(&self) -> &str {
        self.default_model
            .as_deref()
            .unwrap_or("gemini-chat")
    }

    /// Convert to SamplerConfig for use with the sampler
    pub fn to_sampler_config(&self) -> xai_grok_sampler::SamplerConfig {
        use std::num::NonZeroU64;

        xai_grok_sampler::SamplerConfig {
            api_key: self.api_key.clone(),
            base_url: self.effective_base_url(),
            model: self.effective_default_model().to_string(),
            api_backend: self.api_backend.clone(),
            auth_scheme: self.auth_scheme.clone(),
            request_compression: self.request_compression.clone(),
            max_completion_tokens: self.max_completion_tokens,
            temperature: self.temperature,
            top_p: self.top_p,
            reasoning_effort: self.reasoning_effort.clone(),
            reasoning_summary: self.reasoning_summary.clone(),
            stream_tool_calls: self.stream_tool_calls,
            extra_headers: self.extra_headers.clone(),
            query_params: self.query_params.clone(),
            env_http_headers: self.env_http_headers.clone(),
            ..Default::default()
        }
    }

    /// Create a config for gemini-chat testing
    pub fn gemini_test_config() -> Self {
        Self {
            base_url: crate::DEFAULT_NEMAPI_BASE_URL.to_string(),
            default_provider: Some("gemini".to_string()),
            default_model: Some("gemini-chat".to_string()),
            stream_enabled: true,
            fresh_chat: true,
            premium_md: true,
            ..Self::default()
        }
    }
}

/// Configuration for parser behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ParserConfig {
    /// Whether to strip HTML entities
    pub strip_html_entities: bool,

    /// Whether to remove control characters
    pub remove_control_chars: bool,

    /// Whether to clean UI chrome
    pub clean_ui_chrome: bool,

    /// Whether to strip thinking blocks
    pub strip_thinking: bool,

    /// Whether to remove empty tool calls
    pub remove_empty_tool_calls: bool,

    /// Maximum depth for JSON parsing retries
    pub max_parse_retries: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            strip_html_entities: true,
            remove_control_chars: true,
            clean_ui_chrome: true,
            strip_thinking: true,
            remove_empty_tool_calls: true,
            max_parse_retries: 3,
        }
    }
}

impl From<&NemApiConfig> for ParserConfig {
    fn from(config: &NemApiConfig) -> Self {
        Self {
            strip_html_entities: config.strip_html_entities,
            remove_control_chars: config.remove_control_chars,
            clean_ui_chrome: config.clean_ui_chrome,
            strip_thinking: config.strip_thinking,
            remove_empty_tool_calls: true,
            max_parse_retries: config.max_parse_retries,
        }
    }
}
