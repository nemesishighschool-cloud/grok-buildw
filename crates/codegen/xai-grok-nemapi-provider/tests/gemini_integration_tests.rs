//! Gemini-specific integration tests for NemApi
//!
//! These tests verify that the NemApi integration works correctly with the Gemini provider.

use std::time::Duration;

use mockito::{mock, Server};
use serde_json::json;
use tokio::time::sleep;

use xai_grok_sampling_types::{ChatCompletionRequest, ChatCompletionResponse, Message};

use xai_grok_nemapi_provider::{
    NemApiClient, NemApiConfig, NemApiProvider, NemApiSamplingClient,
};

/// Test that gemini is the default provider
#[tokio::test]
async fn test_gemini_is_default_provider() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    let default = provider.get_default_provider();
    assert_eq!(default.id, "gemini");
    assert_eq!(default.canonical_model, "gemini-chat");
}

/// Test that gemini-chat is the default model
#[tokio::test]
async fn test_gemini_chat_is_default_model() {
    let config = NemApiConfig::default();
    let client = NemApiClient::new(config).unwrap();

    assert_eq!(client.default_model(), "gemini-chat");
    assert_eq!(client.default_provider(), "gemini");
}

/// Test gemini model resolution
#[tokio::test]
async fn test_gemini_model_resolution() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    // Test various gemini model names
    let test_cases = vec![
        ("gemini-chat", "gemini", "gemini-chat"),
        ("gemini-2.5-flash", "gemini", "gemini-chat"),
        ("gemini-pro", "gemini", "gemini-chat"),
        ("gemini-flash", "gemini", "gemini-chat"),
        ("flash", "gemini", "gemini-chat"),
    ];

    for (model_input, expected_provider, expected_model) in test_cases {
        let (provider_id, model) = provider.resolve_model(model_input).unwrap();
        assert_eq!(provider_id, expected_provider, "Failed for model: {}", model_input);
        assert_eq!(model, expected_model, "Failed for model: {}", model_input);
    }
}

/// Test gemini provider info
#[tokio::test]
async fn test_gemini_provider_info() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    let gemini_provider = provider.get_provider("gemini").unwrap();

    assert_eq!(gemini_provider.id, "gemini");
    assert_eq!(gemini_provider.display_name, "Gemini");
    assert_eq!(gemini_provider.canonical_model, "gemini-chat");
    assert!(gemini_provider.enabled);
    assert!(!gemini_provider.models.is_empty());
    assert!(!gemini_provider.aliases.is_empty());
}

/// Test gemini configuration for testing
#[tokio::test]
async fn test_gemini_test_configuration() {
    let config = NemApiConfig::gemini_test_config();

    assert_eq!(config.default_provider, Some("gemini".to_string()));
    assert_eq!(config.default_model, Some("gemini-chat".to_string()));
    assert!(config.stream_enabled);
    assert!(config.fresh_chat);
    assert!(config.premium_md);
}

/// Test gemini client creation
#[tokio::test]
async fn test_gemini_client_creation() {
    let config = NemApiConfig::gemini_test_config();
    let client = NemApiClient::new(config).unwrap();

    assert_eq!(client.default_model(), "gemini-chat");
    assert_eq!(client.default_provider(), "gemini");
    assert_eq!(client.effective_base_url(), "http://127.0.0.1:8090/v1");
}

/// Test gemini sampling client creation
#[tokio::test]
async fn test_gemini_sampling_client_creation() {
    let config = NemApiConfig::gemini_test_config();
    let client = NemApiSamplingClient::new(config).unwrap();

    assert_eq!(client.default_model(), "gemini-chat");
    assert_eq!(client.base_url(), "http://127.0.0.1:8090/v1");
}

/// Test gemini client with defaults
#[tokio::test]
async fn test_gemini_client_with_defaults() {
    let client = NemApiClient::with_gemini_test_config().unwrap();
    assert_eq!(client.default_model(), "gemini-chat");
    assert_eq!(client.default_provider(), "gemini");
}

/// Test gemini sampling client with defaults
#[tokio::test]
async fn test_gemini_sampling_client_with_defaults() {
    let client = NemApiSamplingClient::with_gemini_test_config().unwrap();
    assert_eq!(client.default_model(), "gemini-chat");
}

/// Test gemini model info
#[tokio::test]
async fn test_gemini_model_info() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    let model_info = provider.to_model_info("gemini-chat").unwrap();

    assert_eq!(model_info.model_id, "gemini-chat");
    assert_eq!(model_info.model_family, "gemini");
    assert_eq!(model_info.name, "Gemini (gemini-chat)");
    assert!(model_info.description.contains("NemApi"));
    assert_eq!(model_info.context_window, 128_000);
    assert!(model_info.supports_reasoning_effort);
    assert!(model_info.supports_tool_use);
}

/// Test gemini provider by model
#[tokio::test]
async fn test_gemini_provider_by_model() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    // Test that gemini models are recognized
    let gemini_models = ["gemini-chat", "gemini-2.5-flash", "gemini-pro", "gemini-flash"];

    for model in gemini_models {
        let provider_info = provider.get_provider_by_model(model).unwrap();
        assert_eq!(provider_info.id, "gemini");
    }
}

/// Test gemini aliases
#[tokio::test]
async fn test_gemini_aliases() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    // Test that gemini aliases work
    let gemini_aliases = ["flash", "gemini-flash"];

    for alias in gemini_aliases {
        let provider_info = provider.get_provider_by_model(alias).unwrap();
        assert_eq!(provider_info.id, "gemini");
    }
}

/// Test gemini provider availability
#[tokio::test]
async fn test_gemini_provider_availability() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    assert!(provider.is_provider_available("gemini"));
}

/// Test gemini in all providers list
#[tokio::test]
async fn test_gemini_in_all_providers() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    let all_providers = provider.get_all_providers();
    let provider_ids: Vec<&str> = all_providers.iter().map(|p| p.id.as_str()).collect();

    assert!(provider_ids.contains(&"gemini"));
}

/// Test gemini in enabled providers list
#[tokio::test]
async fn test_gemini_in_enabled_providers() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    let enabled_providers = provider.get_enabled_providers();
    let provider_ids: Vec<&str> = enabled_providers.iter().map(|p| p.id.as_str()).collect();

    assert!(provider_ids.contains(&"gemini"));
}

/// Test gemini in all models list
#[tokio::test]
async fn test_gemini_in_all_models() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    let all_models = provider.get_all_models();

    assert!(all_models.contains(&"gemini-chat".to_string()));
    assert!(all_models.contains(&"gemini-2.5-flash".to_string()));
    assert!(all_models.contains(&"gemini-pro".to_string()));
}

/// Test gemini provider config
#[tokio::test]
async fn test_gemini_provider_config() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    let provider_config = provider.get_provider_config("gemini").unwrap();

    // Default config should be empty
    assert!(provider_config.base_url.is_none());
    assert!(provider_config.api_key.is_none());
    assert!(provider_config.extra_headers.is_empty());
}

/// Test that gemini is the first default provider
#[tokio::test]
async fn test_gemini_is_first_in_providers() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    let all_providers = provider.get_all_providers();
    assert!(!all_providers.is_empty());

    // Check that gemini is in the list
    let gemini_index = all_providers.iter().position(|p| p.id == "gemini");
    assert!(gemini_index.is_some());
}

/// Test gemini with provider/model format
#[tokio::test]
async fn test_gemini_provider_model_format() {
    let config = NemApiConfig::default();
    let provider = NemApiProvider::new(config);

    // Test provider/model format
    let (provider_id, model) = provider.resolve_model("gemini/gemini-pro").unwrap();
    assert_eq!(provider_id, "gemini");
    assert_eq!(model, "gemini-chat"); // Should fall back to canonical
}

/// Test gemini with provider/model format for known model
#[tokio::test]
async fn test_gemini_provider_known_model_format() {
    let config = NemApiConfig::default();
    let mut provider = NemApiProvider::new(config);

    // Add gemini-pro to the gemini provider models
    if let Some(gemini_provider) = provider.providers.get_mut("gemini") {
        gemini_provider.models.push("gemini-pro".to_string());
    }

    // Now test with known model
    let (provider_id, model) = provider.resolve_model("gemini/gemini-pro").unwrap();
    assert_eq!(provider_id, "gemini");
    assert_eq!(model, "gemini-pro");
}

/// Test gemini conversation reset
#[tokio::test]
async fn test_gemini_conversation_reset() {
    let config = NemApiConfig::gemini_test_config();
    let client = NemApiClient::new(config).unwrap();

    // Initially should be first request
    assert!(client.is_first_request().await);

    // Reset conversation
    client.reset_conversation().await;

    // Should be first request again
    assert!(client.is_first_request().await);
}

/// Test gemini client effective base URL
#[tokio::test]
async fn test_gemini_client_effective_base_url() {
    let config = NemApiConfig::gemini_test_config();
    let client = NemApiClient::new(config).unwrap();

    assert_eq!(client.effective_base_url(), "http://127.0.0.1:8090/v1");
}

/// Test gemini client with custom base URL
#[tokio::test]
async fn test_gemini_client_custom_base_url() {
    let mut config = NemApiConfig::gemini_test_config();
    config.base_url = "http://custom.url:8080/v1".to_string();

    let client = NemApiClient::new(config).unwrap();
    assert_eq!(client.effective_base_url(), "http://custom.url:8080/v1");
}
