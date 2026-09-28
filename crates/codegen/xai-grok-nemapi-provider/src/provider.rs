//! Provider management for NemApi integration
//!
//! This module manages all available providers and their configurations,
//! allowing NemApi to route requests to the appropriate AI provider.

use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use xai_grok_sampling_types::{ApiBackend, ModelInfo, ModelMetadata};

use crate::{config::NemApiConfig, error::Result, NEMAPI_MODEL_ALIASES, NEMAPI_PROVIDERS};

/// Information about a NemApi provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NemApiProviderInfo {
    /// Provider ID (e.g., "gemini", "claude")
    pub id: String,

    /// Display name
    pub display_name: String,

    /// Canonical model for this provider
    pub canonical_model: String,

    /// All supported models for this provider
    pub models: Vec<String>,

    /// Aliases that map to this provider
    pub aliases: Vec<String>,

    /// Whether this provider is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Custom configuration for this provider
    #[serde(default)]
    pub config: ProviderSpecificConfig,
}

fn default_enabled() -> bool {
    true
}

/// Provider-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderSpecificConfig {
    /// Custom base URL for this provider
    pub base_url: Option<String>,

    /// API key for this provider
    pub api_key: Option<String>,

    /// Extra headers for this provider
    pub extra_headers: IndexMap<String, String>,

    /// Whether to use fresh chat for this provider
    pub fresh_chat: Option<bool>,

    /// Custom timeout for this provider
    pub timeout_secs: Option<u64>,
}

/// NemApi Provider Manager
///
/// Manages all available providers and their configurations
#[derive(Debug, Clone)]
pub struct NemApiProvider {
    /// Configuration for NemApi
    config: NemApiConfig,

    /// Map of provider ID to provider info
    providers: HashMap<String, NemApiProviderInfo>,

    /// Map of model name to provider ID
    model_to_provider: HashMap<String, String>,

    /// Map of alias to provider ID
    alias_to_provider: HashMap<String, String>,
}

impl NemApiProvider {
    /// Create a new NemApiProvider with default configuration
    pub fn new(config: NemApiConfig) -> Self {
        let mut provider = Self {
            config,
            providers: HashMap::new(),
            model_to_provider: HashMap::new(),
            alias_to_provider: HashMap::new(),
        };

        // Initialize with default providers
        provider.initialize_default_providers();

        provider
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(NemApiConfig::default())
    }

    /// Initialize default providers
    /// Each provider has EXACTLY ONE canonical model (with -chat suffix)
    /// When using / command to select models, ONLY these 7 canonical models should be shown
    fn initialize_default_providers(&mut self) {
        // DeepSeek - ONLY canonical model: deepseek-chat
        self.add_provider(NemApiProviderInfo {
            id: "deepseek".to_string(),
            display_name: "DeepSeek".to_string(),
            canonical_model: "deepseek-chat".to_string(),
            // ONLY the canonical model in the models list
            models: vec![
                "deepseek-chat".to_string(),
            ],
            // Aliases for backward compatibility (not shown in / command)
            aliases: vec![
                "deepseek-coder".to_string(),
                "deepseek-v3".to_string(),
                "deepseek-r1".to_string(),
            ],
            enabled: true,
            config: ProviderSpecificConfig::default(),
        });

        // Qwen - ONLY canonical model: qwen-chat
        self.add_provider(NemApiProviderInfo {
            id: "qwen".to_string(),
            display_name: "Qwen".to_string(),
            canonical_model: "qwen-chat".to_string(),
            models: vec![
                "qwen-chat".to_string(),
            ],
            aliases: vec![
                "qwen-plus".to_string(),
                "qwen2.5-plus".to_string(),
                "qwen3-coder-plus".to_string(),
                "qwen-max".to_string(),
            ],
            enabled: true,
            config: ProviderSpecificConfig::default(),
        });

        // Claude - ONLY canonical model: claude-chat
        self.add_provider(NemApiProviderInfo {
            id: "claude".to_string(),
            display_name: "Claude".to_string(),
            canonical_model: "claude-chat".to_string(),
            models: vec![
                "claude-chat".to_string(),
            ],
            aliases: vec![
                "claude-sonnet".to_string(),
                "claude-3-sonnet".to_string(),
                "claude-3-haiku".to_string(),
            ],
            enabled: true,
            config: ProviderSpecificConfig::default(),
        });

        // Gemini - DEFAULT PROVIDER - ONLY canonical model: gemini-chat
        self.add_provider(NemApiProviderInfo {
            id: "gemini".to_string(),
            display_name: "Gemini".to_string(),
            canonical_model: "gemini-chat".to_string(),
            models: vec![
                "gemini-chat".to_string(),
            ],
            aliases: vec![
                "gemini-2.5-flash".to_string(),
                "gemini-2.0-flash".to_string(),
                "gemini-pro".to_string(),
                "gemini-flash".to_string(),
                "flash".to_string(),
            ],
            enabled: true,
            config: ProviderSpecificConfig::default(),
        });

        // ChatGPT - ONLY canonical model: gpt-chat
        self.add_provider(NemApiProviderInfo {
            id: "chatgpt".to_string(),
            display_name: "ChatGPT".to_string(),
            canonical_model: "gpt-chat".to_string(),
            models: vec![
                "gpt-chat".to_string(),
            ],
            aliases: vec![
                "gpt-4".to_string(),
                "gpt-4o".to_string(),
                "gpt-4.1".to_string(),
                "gpt-5".to_string(),
                "gpt-3.5-turbo".to_string(),
                "o1".to_string(),
                "o3".to_string(),
            ],
            enabled: true,
            config: ProviderSpecificConfig::default(),
        });

        // Kimi - ONLY canonical model: kimi-chat
        self.add_provider(NemApiProviderInfo {
            id: "kimi".to_string(),
            display_name: "Kimi".to_string(),
            canonical_model: "kimi-chat".to_string(),
            models: vec![
                "kimi-chat".to_string(),
            ],
            aliases: vec![
                "kimi-k2".to_string(),
                "kimi-k3".to_string(),
                "kimi".to_string(),
                "moonshot".to_string(),
            ],
            enabled: true,
            config: ProviderSpecificConfig::default(),
        });

        // Z.ai / GLM - ONLY canonical model: glm-chat
        self.add_provider(NemApiProviderInfo {
            id: "zai".to_string(),
            display_name: "Z.ai / GLM".to_string(),
            canonical_model: "glm-chat".to_string(),
            models: vec![
                "glm-chat".to_string(),
            ],
            aliases: vec![
                "glm-4".to_string(),
                "glm-5".to_string(),
                "zai-chat".to_string(),
                "zai".to_string(),
                "z.ai".to_string(),
                "glm".to_string(),
                "chatglm".to_string(),
            ],
            enabled: true,
            config: ProviderSpecificConfig::default(),
        });
    }

    /// Add a provider to the manager
    pub fn add_provider(&mut self, provider: NemApiProviderInfo) {
        let id = provider.id.clone();

        // Add provider
        self.providers.insert(id.clone(), provider);

        // Add model mappings
        for model in self.providers[&id].models.iter() {
            self.model_to_provider
                .insert(model.clone(), id.clone());
        }

        // Add alias mappings
        for alias in self.providers[&id].aliases.iter() {
            self.alias_to_provider
                .insert(alias.clone(), id.clone());
        }
    }

    /// Get provider by ID
    pub fn get_provider(&self, provider_id: &str) -> Option<&NemApiProviderInfo> {
        self.providers.get(provider_id)
    }

    /// Get provider by model name
    pub fn get_provider_by_model(&self, model: &str) -> Option<&NemApiProviderInfo> {
        // Try direct model lookup
        if let Some(provider_id) = self.model_to_provider.get(model) {
            return self.providers.get(provider_id);
        }

        // Try alias lookup
        if let Some(provider_id) = self.alias_to_provider.get(model) {
            return self.providers.get(provider_id);
        }

        // Try provider/model format
        if let Some(pos) = model.find('/') {
            let (potential_provider, potential_model) = model.split_at(pos);
            let trimmed_model = potential_model.trim_start_matches('/');

            // Check if provider exists
            if self.providers.contains_key(potential_provider) {
                return self.providers.get(potential_provider);
            }

            // Check if model maps to a provider
            if let Some(provider_id) = self.model_to_provider.get(trimmed_model) {
                return self.providers.get(provider_id);
            }
        }

        None
    }

    /// Resolve a model string to (provider_id, canonical_model)
    pub fn resolve_model(&self, model: &str) -> Result<(String, String)> {
        // Try to get provider by model
        if let Some(provider) = self.get_provider_by_model(model) {
            return Ok((provider.id.clone(), provider.canonical_model.clone()));
        }

        // If model contains '/', try to parse provider/model
        if let Some(pos) = model.find('/') {
            let (provider_id, model_name) = model.split_at(pos);
            let trimmed_model = model_name.trim_start_matches('/');

            if let Some(provider) = self.providers.get(provider_id) {
                // Use the provided model or fall back to canonical
                if provider.models.contains(&trimmed_model.to_string()) {
                    return Ok((provider_id.to_string(), trimmed_model.to_string()));
                }
                return Ok((provider_id.to_string(), provider.canonical_model.clone()));
            }
        }

        // Try fuzzy matching
        for (provider_id, provider) in &self.providers {
            // Check if model matches any alias
            for alias in &provider.aliases {
                if alias == model {
                    return Ok((provider_id.clone(), provider.canonical_model.clone()));
                }
            }

            // Check if model contains provider name
            if model.contains(provider_id.as_str()) {
                return Ok((provider_id.clone(), provider.canonical_model.clone()));
            }
        }

        Err(crate::error::NemApiError::ModelNotFound(
            model.to_string(),
            "unknown".to_string(),
        ))
    }

    /// Get all providers
    pub fn get_all_providers(&self) -> Vec<&NemApiProviderInfo> {
        self.providers.values().collect()
    }

    /// Get enabled providers
    pub fn get_enabled_providers(&self) -> Vec<&NemApiProviderInfo> {
        self.providers
            .values()
            .filter(|p| p.enabled)
            .collect()
    }

    /// Get all models across all providers
    pub fn get_all_models(&self) -> Vec<String> {
        let mut models = Vec::new();
        for provider in self.providers.values() {
            models.extend(&provider.models);
            models.extend(&provider.aliases);
        }
        models
    }

    /// Check if a provider is available
    pub fn is_provider_available(&self, provider_id: &str) -> bool {
        self.providers
            .get(provider_id)
            .map(|p| p.enabled)
            .unwrap_or(false)
    }

    /// Get configuration for a specific provider
    pub fn get_provider_config(&self, provider_id: &str) -> Option<&ProviderSpecificConfig> {
        self.providers
            .get(provider_id)
            .map(|p| &p.config)
    }

    /// Update provider configuration
    pub fn update_provider_config(
        &mut self,
        provider_id: &str,
        config: ProviderSpecificConfig,
    ) -> Result<()> {
        if let Some(provider) = self.providers.get_mut(provider_id) {
            provider.config = config;
            Ok(())
        } else {
            Err(crate::error::NemApiError::ProviderNotConfigured(
                provider_id.to_string(),
            ))
        }
    }

    /// Get the default provider
    pub fn get_default_provider(&self) -> &NemApiProviderInfo {
        let provider_id = self.config.effective_default_provider();
        self.providers
            .get(provider_id)
            .expect("Default provider not found")
    }

    /// Convert to ModelInfo for use with the sampler
    pub fn to_model_info(&self, model_id: &str) -> Result<ModelInfo> {
        let (provider_id, canonical_model) = self.resolve_model(model_id)?;

        let provider = self.get_provider(&provider_id)
            .ok_or_else(|| crate::error::NemApiError::ProviderNotConfigured(provider_id.clone()))?;

        Ok(ModelInfo {
            model_id: model_id.to_string(),
            model_family: provider.id.clone(),
            name: format!("{} ({})", provider.display_name, canonical_model),
            description: format!("{} via NemApi proxy", provider.display_name),
            context_window: self.config.context_window,
            api_backend: ApiBackend::ChatCompletions,
            supports_backend_search: false,
            system_prompt_label: provider.display_name.clone(),
            supports_reasoning_effort: true,
            reasoning_effort: self.config.reasoning_effort.clone().unwrap_or_default(),
            supports_tool_use: true,
            supports_vision: false,
            supports_audio: false,
            supports_image: false,
            supports_video: false,
            max_request_bytes: None,
            max_completion_tokens: self.config.max_completion_tokens,
        })
    }

    /// Get configuration
    pub fn config(&self) -> &NemApiConfig {
        &self.config
    }
}

impl Default for NemApiProvider {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Get provider ID from model string
pub fn get_provider_from_model(model: &str) -> Option<&'static str> {
    for &provider in NEMAPI_PROVIDERS {
        if model.starts_with(provider) || model.contains(provider) {
            return Some(provider);
        }
    }
    None
}

/// Check if a model is a known alias
pub fn is_known_model_alias(model: &str) -> bool {
    NEMAPI_MODEL_ALIASES.contains(&model)
}
