//! Integration tests for NemApi provider
//!
//! These tests verify that the NemApi integration works correctly with various providers.

use std::time::Duration;

use mockito::{mock, Server};
use serde_json::json;
use tokio::time::sleep;

use xai_grok_sampling_types::{ChatCompletionRequest, ChatCompletionResponse, Message};

use xai_grok_nemapi_provider::{
    NemApiClient, NemApiConfig, NemApiProvider, NemApiSamplingClient,
    NEMAPI_PROVIDERS,
};

/// Test that the default configuration works
#[tokio::test]
async fn test_default_configuration() {
    let config = NemApiConfig::default();
    assert_eq!(config.base_url, "http://127.0.0.1:8090/v1");
    assert_eq!(config.effective_default_provider(), "gemini");
    assert_eq!(config.effective_default_model(), "gemini-chat");
}

/// Test that gemini test configuration works
#[tokio::test]
async fn test_gemini_test_configuration() {
    let config = NemApiConfig::gemini_test_config();
    assert_eq!(config.default_provider, Some("gemini".to_string()));
    assert_eq!(config.default_model, Some("gemini-chat".to_string()));
    assert!(config.stream_enabled);
    assert!(config.fresh_chat);
    assert!(config.premium_md);
}

/// Test provider resolution for gemini
#[tokio::test]
async fn test_provider_resolution_gemini() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    // Test direct model
    let (provider_id, model) = provider.resolve_model("gemini-chat").unwrap();
    assert_eq!(provider_id, "gemini");
    assert_eq!(model, "gemini-chat");

    // Test alias
    let (provider_id, model) = provider.resolve_model("flash").unwrap();
    assert_eq!(provider_id, "gemini");
    assert_eq!(model, "gemini-chat");

    // Test provider/model format
    let (provider_id, model) = provider.resolve_model("gemini/gemini-pro").unwrap();
    assert_eq!(provider_id, "gemini");
}

/// Test provider resolution for claude
#[tokio::test]
async fn test_provider_resolution_claude() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    // Test direct model
    let (provider_id, model) = provider.resolve_model("claude-chat").unwrap();
    assert_eq!(provider_id, "claude");
    assert_eq!(model, "claude-chat");

    // Test alias
    let (provider_id, model) = provider.resolve_model("sonnet").unwrap();
    assert_eq!(provider_id, "claude");
    assert_eq!(model, "claude-chat");
}

/// Test provider resolution for qwen
#[tokio::test]
async fn test_provider_resolution_qwen() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    // Test direct model
    let (provider_id, model) = provider.resolve_model("qwen-chat").unwrap();
    assert_eq!(provider_id, "qwen");
    assert_eq!(model, "qwen-chat");

    // Test alias
    let (provider_id, model) = provider.resolve_model("plus").unwrap();
    assert_eq!(provider_id, "qwen");
    assert_eq!(model, "qwen-chat");
}

/// Test that all default providers are available
#[tokio::test]
async fn test_all_default_providers_available() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    for &provider_name in NEMAPI_PROVIDERS {
        assert!(
            provider.is_provider_available(provider_name),
            "Provider {} should be available",
            provider_name
        );
    }
}

/// Test client creation with default config
#[tokio::test]
async fn test_client_creation_default() {
    let config = NemApiConfig::default();
    let client = NemApiClient::new(config).unwrap();

    assert_eq!(client.default_model(), "gemini-chat");
    assert_eq!(client.default_provider(), "gemini");
    assert_eq!(client.effective_base_url(), "http://127.0.0.1:8090/v1");
}

/// Test client creation with gemini config
#[tokio::test]
async fn test_client_creation_gemini() {
    let config = NemApiConfig::gemini_test_config();
    let client = NemApiClient::new(config).unwrap();

    assert_eq!(client.default_model(), "gemini-chat");
    assert_eq!(client.default_provider(), "gemini");
}

/// Test sampling client creation
#[tokio::test]
async fn test_sampling_client_creation() {
    let config = NemApiConfig::default();
    let client = NemApiSamplingClient::new(config).unwrap();

    assert_eq!(client.default_model(), "gemini-chat");
    assert_eq!(client.base_url(), "http://127.0.0.1:8090/v1");
}

/// Test that the client can be created with defaults
#[tokio::test]
async fn test_client_with_defaults() {
    let client = NemApiClient::with_defaults().unwrap();
    assert_eq!(client.default_model(), "gemini-chat");
}

/// Test that the sampling client can be created with defaults
#[tokio::test]
async fn test_sampling_client_with_defaults() {
    let client = NemApiSamplingClient::with_defaults().unwrap();
    assert_eq!(client.default_model(), "gemini-chat");
}

/// Test conversation reset
#[tokio::test]
async fn test_conversation_reset() {
    let config = NemApiConfig::default();
    let client = NemApiClient::new(config).unwrap();

    // Initially should be first request
    assert!(client.is_first_request().await);

    // Reset conversation
    client.reset_conversation().await;

    // Should be first request again
    assert!(client.is_first_request().await);
}

/// Test model info conversion
#[tokio::test]
async fn test_model_info_conversion() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    let model_info = provider.to_model_info("gemini-chat").unwrap();

    assert_eq!(model_info.model_id, "gemini-chat");
    assert_eq!(model_info.model_family, "gemini");
    assert!(model_info.name.contains("Gemini"));
    assert!(model_info.description.contains("NemApi"));
    assert_eq!(model_info.context_window, 128_000);
}

/// Test error handling for unknown model
#[tokio::test]
async fn test_unknown_model_error() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    let result = provider.resolve_model("unknown-model-xyz");
    assert!(result.is_err());
}

/// Test error handling for unknown provider
#[tokio::test]
async fn test_unknown_provider_error() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    let result = provider.get_provider("unknown-provider");
    assert!(result.is_none());
}

/// Test configuration validation
#[tokio::test]
async fn test_config_validation() {
    // Valid config
    let config = NemApiConfig::default();
    assert!(config.validate().is_ok());

    // Empty base URL should fail
    let mut config = NemApiConfig::default();
    config.base_url = "".to_string();
    assert!(config.validate().is_err());

    // Invalid API key format should fail
    let mut config = NemApiConfig::default();
    config.api_key = Some("invalid-key".to_string());
    assert!(config.validate().is_err());

    // Valid API key format should pass
    let mut config = NemApiConfig::default();
    config.api_key = Some("nemapi-token1234567890abcdef".to_string());
    assert!(config.validate().is_ok());
}

/// Test provider configuration update
#[tokio::test]
async fn test_provider_config_update() {
    let config = NemApiConfig::default();
    let mut provider = NemApiProvider::new(config);

    use xai_grok_nemapi_provider::provider::ProviderSpecificConfig;

    let new_config = ProviderSpecificConfig {
        base_url: Some("http://custom.url".to_string()),
        api_key: Some("custom-key".to_string()),
        ..Default::default()
    };

    provider
        .update_provider_config("gemini", new_config)
        .unwrap();

    let updated_config = provider.get_provider_config("gemini").unwrap();
    assert_eq!(updated_config.base_url, Some("http://custom.url".to_string()));
}

/// Test that all providers have canonical models
#[tokio::test]
async fn test_all_providers_have_canonical_models() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    for provider_info in provider.get_all_providers() {
        assert!(
            !provider_info.canonical_model.is_empty(),
            "Provider {} should have a canonical model",
            provider_info.id
        );
    }
}

/// Test that all providers have display names
#[tokio::test]
async fn test_all_providers_have_display_names() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    for provider_info in provider.get_all_providers() {
        assert!(
            !provider_info.display_name.is_empty(),
            "Provider {} should have a display name",
            provider_info.id
        );
    }
}

/// Test get provider from model function
#[tokio::test]
async fn test_get_provider_from_model() {
    use xai_grok_nemapi_provider::provider::get_provider_from_model;

    assert_eq!(get_provider_from_model("gemini-chat"), Some("gemini"));
    assert_eq!(get_provider_from_model("claude-sonnet"), Some("claude"));
    assert_eq!(get_provider_from_model("qwen-plus"), Some("qwen"));
    assert_eq!(get_provider_from_model("deepseek-v3"), Some("deepseek"));
    assert_eq!(get_provider_from_model("gpt-4"), Some("chatgpt"));
    assert_eq!(get_provider_from_model("kimi-k2"), Some("kimi"));
    assert_eq!(get_provider_from_model("glm-4"), Some("zai"));
}

/// Test known model alias check
#[tokio::test]
async fn test_known_model_alias() {
    use xai_grok_nemapi_provider::provider::is_known_model_alias;

    assert!(is_known_model_alias("gemini-chat"));
    assert!(is_known_model_alias("claude-3-sonnet"));
    assert!(is_known_model_alias("qwen2.5-plus"));
    assert!(!is_known_model_alias("unknown-model"));
}
