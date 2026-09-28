//! Model configuration for NemApi provider
//!
//! This module provides the list of models that should be shown when using the `/` command
//! to select models. Only the 7 canonical NemApi models should be displayed.

use serde::{Deserialize, Serialize};

use crate::{NEMAPI_CANONICAL_MODELS, NEMAPI_PROVIDERS};

/// Information about a NemApi model for display in the / command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NemApiModelInfo {
    /// Model name (canonical name with -chat suffix)
    pub name: String,

    /// Display name for the model
    pub display_name: String,

    /// Provider ID
    pub provider: String,

    /// Provider display name
    pub provider_display_name: String,

    /// Description
    pub description: String,

    /// Whether this is the default model
    #[serde(default)]
    pub is_default: bool,
}

/// Get the list of models to display in the / command
/// Only the 7 canonical models should be shown, no fictional names
pub fn get_nemapi_models_for_selection() -> Vec<NemApiModelInfo> {
    vec![
        NemApiModelInfo {
            name: "deepseek-chat".to_string(),
            display_name: "DeepSeek".to_string(),
            provider: "deepseek".to_string(),
            provider_display_name: "DeepSeek".to_string(),
            description: "DeepSeek AI model via NemApi".to_string(),
            is_default: false,
        },
        NemApiModelInfo {
            name: "qwen-chat".to_string(),
            display_name: "Qwen".to_string(),
            provider: "qwen".to_string(),
            provider_display_name: "Qwen".to_string(),
            description: "Qwen AI model via NemApi".to_string(),
            is_default: false,
        },
        NemApiModelInfo {
            name: "claude-chat".to_string(),
            display_name: "Claude".to_string(),
            provider: "claude".to_string(),
            provider_display_name: "Claude".to_string(),
            description: "Claude AI model via NemApi".to_string(),
            is_default: false,
        },
        NemApiModelInfo {
            name: "gemini-chat".to_string(),
            display_name: "Gemini".to_string(),
            provider: "gemini".to_string(),
            provider_display_name: "Gemini".to_string(),
            description: "Gemini AI model via NemApi (Default)".to_string(),
            is_default: true,
        },
        NemApiModelInfo {
            name: "gpt-chat".to_string(),
            display_name: "ChatGPT".to_string(),
            provider: "chatgpt".to_string(),
            provider_display_name: "ChatGPT".to_string(),
            description: "ChatGPT model via NemApi".to_string(),
            is_default: false,
        },
        NemApiModelInfo {
            name: "kimi-chat".to_string(),
            display_name: "Kimi".to_string(),
            provider: "kimi".to_string(),
            provider_display_name: "Kimi".to_string(),
            description: "Kimi AI model via NemApi".to_string(),
            is_default: false,
        },
        NemApiModelInfo {
            name: "glm-chat".to_string(),
            display_name: "GLM".to_string(),
            provider: "zai".to_string(),
            provider_display_name: "Z.ai / GLM".to_string(),
            description: "GLM AI model via NemApi".to_string(),
            is_default: false,
        },
    ]
}

/// Get the list of model names only (for / command)
/// Only returns the 7 canonical model names
pub fn get_nemapi_model_names() -> Vec<String> {
    NEMAPI_CANONICAL_MODELS.to_vec().iter().map(|s| s.to_string()).collect()
}

/// Get the default model name
pub fn get_default_nemapi_model() -> String {
    "gemini-chat".to_string()
}

/// Get the list of provider names
pub fn get_nemapi_provider_names() -> Vec<String> {
    NEMAPI_PROVIDERS.to_vec().iter().map(|s| s.to_string()).collect()
}

/// Get model info by name
pub fn get_model_info_by_name(name: &str) -> Option<NemApiModelInfo> {
    get_nemapi_models_for_selection().into_iter().find(|m| m.name == name)
}

/// Check if a model name is a canonical NemApi model
/// Returns true only for the 7 canonical models
pub fn is_canonical_nemapi_model(name: &str) -> bool {
    NEMAPI_CANONICAL_MODELS.contains(&name)
}

/// Get the list of models for the / command
/// This is what should be displayed when user types / to select a model
/// Only the 7 canonical models, no aliases, no fictional names
pub fn get_models_for_slash_command() -> Vec<String> {
    get_nemapi_model_names()
}

/// Format the models for display in the / command
pub fn format_models_for_slash_command() -> String {
    let models = get_models_for_slash_command();
    let mut result = String::new();
    
    for (i, model) in models.iter().enumerate() {
        if i > 0 {
            result.push_str(", ");
        }
        // Highlight the default model
        if model == "gemini-chat" {
            result.push_str("*");
            result.push_str(model);
            result.push_str("*");
        } else {
            result.push_str(model);
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_models_count() {
        let models = get_nemapi_model_names();
        assert_eq!(models.len(), 7, "Should have exactly 7 canonical models");
    }

    #[test]
    fn test_canonical_models_list() {
        let models = get_nemapi_model_names();
        let expected = vec![
            "deepseek-chat",
            "qwen-chat",
            "claude-chat",
            "gemini-chat",
            "gpt-chat",
            "kimi-chat",
            "glm-chat",
        ];
        
        for expected_model in expected {
            assert!(models.contains(&expected_model.to_string()), 
                "Should contain canonical model: {}", expected_model);
        }
    }

    #[test]
    fn test_default_model() {
        let default = get_default_nemapi_model();
        assert_eq!(default, "gemini-chat");
    }

    #[test]
    fn test_is_canonical_model() {
        assert!(is_canonical_nemapi_model("gemini-chat"));
        assert!(is_canonical_nemapi_model("deepseek-chat"));
        assert!(is_canonical_nemapi_model("claude-chat"));
        
        // These should NOT be considered canonical
        assert!(!is_canonical_nemapi_model("gemini-2.5-flash"));
        assert!(!is_canonical_nemapi_model("gpt-4"));
        assert!(!is_canonical_nemapi_model("fictional-model"));
    }

    #[test]
    fn test_provider_names() {
        let providers = get_nemapi_provider_names();
        assert_eq!(providers.len(), 7);
        assert!(providers.contains(&"gemini".to_string()));
        assert!(providers.contains(&"claude".to_string()));
    }

    #[test]
    fn test_model_info() {
        let info = get_model_info_by_name("gemini-chat");
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.name, "gemini-chat");
        assert_eq!(info.provider, "gemini");
        assert!(info.is_default);
    }

    #[test]
    fn test_format_for_slash_command() {
        let formatted = format_models_for_slash_command();
        assert!(formatted.contains("*gemini-chat*"));
        assert!(formatted.contains("deepseek-chat"));
        assert!(formatted.contains("claude-chat"));
    }
}
