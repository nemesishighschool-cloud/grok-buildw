//! Parser tests for NemApi responses
//!
//! These tests verify that the NemApi response parser correctly handles
//! various response formats and cleans up common issues.

use serde_json::json;

use xai_grok_nemapi_provider::{
    NemApiResponseParser, create_default_parser, create_parser_with_config,
};
use xai_grok_nemapi_provider::config::ParserConfig;

/// Test parser creation with defaults
#[test]
fn test_parser_creation_defaults() {
    let parser = NemApiResponseParser::new();
    assert_eq!(parser.config().strip_html_entities, true);
    assert_eq!(parser.config().remove_control_chars, true);
    assert_eq!(parser.config().clean_ui_chrome, true);
    assert_eq!(parser.config().strip_thinking, true);
}

/// Test parser creation with custom config
#[test]
fn test_parser_creation_custom_config() {
    let config = ParserConfig {
        strip_html_entities: false,
        remove_control_chars: false,
        clean_ui_chrome: false,
        strip_thinking: false,
        ..Default::default()
    };

    let parser = NemApiResponseParser::with_config(config);
    assert_eq!(parser.config().strip_html_entities, false);
    assert_eq!(parser.config().remove_control_chars, false);
}

/// Test default parser creation
#[test]
fn test_default_parser_creation() {
    let parser = create_default_parser();
    assert_eq!(parser.config().strip_html_entities, true);
}

/// Test parser with custom config creation
#[test]
fn test_parser_with_config_creation() {
    let config = ParserConfig::default();
    let parser = create_parser_with_config(config);
    assert_eq!(parser.config().strip_html_entities, true);
}

/// Test cleaning HTML entities
#[test]
fn test_clean_html_entities() {
    let parser = NemApiResponseParser::new();

    let input = "Hello &lt;world&gt; &amp; &quot;test&quot;";
    let cleaned = parser.clean_response_body(input, "gemini");

    assert_eq!(cleaned, "Hello <world> & \"test\"");
}

/// Test cleaning control characters
#[test]
fn test_clean_control_chars() {
    let parser = NemApiResponseParser::new();

    let input = "Hello\x00\x01\x02World\x07\x08";
    let cleaned = parser.clean_response_body(input, "gemini");

    // Control characters should be removed except newline, carriage return, tab
    assert!(!cleaned.contains('\x00'));
    assert!(!cleaned.contains('\x01'));
    assert!(!cleaned.contains('\x02'));
    assert!(!cleaned.contains('\x07'));
    assert!(!cleaned.contains('\x08'));
}

/// Test cleaning UI chrome for gemini
#[test]
fn test_clean_ui_chrome_gemini() {
    let parser = NemApiResponseParser::new();

    let input = "Hello world! Send a message. Regenerate";
    let cleaned = parser.clean_response_body(input, "gemini");

    assert!(!cleaned.contains("Send a message."));
    assert!(!cleaned.contains("Regenerate"));
    assert_eq!(cleaned.trim(), "Hello world!");
}

/// Test cleaning UI chrome for claude
#[test]
fn test_clean_ui_chrome_claude() {
    let parser = NemApiResponseParser::new();

    let input = "Hello world! Claude can make mistakes. Please double-check.";
    let cleaned = parser.clean_response_body(input, "claude");

    assert!(!cleaned.contains("Claude can make mistakes."));
    assert!(!cleaned.contains("Please double-check."));
    assert_eq!(cleaned.trim(), "Hello world!");
}

/// Test cleaning UI chrome for chatgpt
#[test]
fn test_clean_ui_chrome_chatgpt() {
    let parser = NemApiResponseParser::new();

    let input = "Hello world! ChatGPT may produce inaccurate information. Please verify.";
    let cleaned = parser.clean_response_body(input, "chatgpt");

    assert!(!cleaned.contains("ChatGPT may produce inaccurate information."));
    assert!(!cleaned.contains("Please verify."));
    assert_eq!(cleaned.trim(), "Hello world!");
}

/// Test stripping thinking blocks
#[test]
fn test_strip_thinking_blocks() {
    let parser = NemApiResponseParser::new();

    let input = "Hello <think>This is thinking</think> world!";
    let cleaned = parser.clean_response_body(input, "gemini");

    assert!(!cleaned.contains("<think>"));
    assert!(!cleaned.contains("</think>"));
    assert!(!cleaned.contains("This is thinking"));
    assert_eq!(cleaned.trim(), "Hello world!");
}

/// Test stripping thinking blocks with different format
#[test]
fn test_strip_thinking_blocks_different_format() {
    let parser = NemApiResponseParser::new();

    let input = "Hello [Think]This is thinking[/Think] world!";
    let cleaned = parser.clean_response_body(input, "gemini");

    assert!(!cleaned.contains("[Think]"));
    assert!(!cleaned.contains("[/Think]"));
    assert!(!cleaned.contains("This is thinking"));
    assert_eq!(cleaned.trim(), "Hello world!");
}

/// Test stripping thinking blocks with code format
#[test]
fn test_strip_thinking_blocks_code_format() {
    let parser = NemApiResponseParser::new();

    let input = "Hello ```think\nThis is thinking\n``` world!";
    let cleaned = parser.clean_response_body(input, "gemini");

    assert!(!cleaned.contains("```think"));
    assert!(!cleaned.contains("This is thinking"));
    assert!(!cleaned.contains("```"));
    assert_eq!(cleaned.trim(), "Hello world!");
}

/// Test cleaning text content
#[test]
fn test_clean_text_content() {
    let parser = NemApiResponseParser::new();

    let input = "Hello &lt;world&gt; &amp; <think>thinking</think>";
    let cleaned = parser.clean_text_content(input, "gemini");

    assert!(!cleaned.contains("&lt;"));
    assert!(!cleaned.contains("&gt;"));
    assert!(!cleaned.contains("&amp;"));
    assert!(!cleaned.contains("<think>"));
    assert!(!cleaned.contains("thinking"));
}

/// Test parsing a simple chat completion response
#[test]
fn test_parse_simple_chat_completion() {
    let parser = NemApiResponseParser::new();

    let response_body = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1234567890,
        "model": "gemini-chat",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello, how can I help you?"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15
        }
    }"#;

    let result = parser.parse_chat_completion_response(response_body, "gemini");
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.id, "chatcmpl-123");
    assert_eq!(response.model, "gemini-chat");
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].message.role, "assistant");
    assert_eq!(response.choices[0].message.content, "Hello, how can I help you?");
}

/// Test parsing a response with UI chrome
#[test]
fn test_parse_response_with_ui_chrome() {
    let parser = NemApiResponseParser::new();

    let response_body = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1234567890,
        "model": "gemini-chat",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello! Send a message. Regenerate"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15
        }
    }"#;

    let result = parser.parse_chat_completion_response(response_body, "gemini");
    assert!(result.is_ok());

    let response = result.unwrap();
    let content = response.choices[0].message.content;

    // UI chrome should be cleaned
    assert!(!content.contains("Send a message."));
    assert!(!content.contains("Regenerate"));
}

/// Test parsing a response with HTML entities
#[test]
fn test_parse_response_with_html_entities() {
    let parser = NemApiResponseParser::new();

    let response_body = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1234567890,
        "model": "gemini-chat",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello &lt;world&gt; &amp; friends"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15
        }
    }"#;

    let result = parser.parse_chat_completion_response(response_body, "gemini");
    assert!(result.is_ok());

    let response = result.unwrap();
    let content = response.choices[0].message.content;

    // HTML entities should be decoded
    assert!(!content.contains("&lt;"));
    assert!(!content.contains("&gt;"));
    assert!(!content.contains("&amp;"));
    assert_eq!(content, "Hello <world> & friends");
}

/// Test parsing a response with control characters
#[test]
fn test_parse_response_with_control_chars() {
    let parser = NemApiResponseParser::new();

    let response_body = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1234567890,
        "model": "gemini-chat",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello\x00\x01World"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15
        }
    }"#;

    let result = parser.parse_chat_completion_response(response_body, "gemini");
    assert!(result.is_ok());

    let response = result.unwrap();
    let content = response.choices[0].message.content;

    // Control characters should be removed
    assert!(!content.contains('\x00'));
    assert!(!content.contains('\x01'));
}

/// Test parsing a minimal response (missing choices)
#[test]
fn test_parse_minimal_response() {
    let parser = NemApiResponseParser::new();

    let response_body = r#"{
        "content": "Simple response"
    }"#;

    let result = parser.parse_chat_completion_response(response_body, "gemini");
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].message.content, "Simple response");
}

/// Test parsing an empty response
#[test]
fn test_parse_empty_response() {
    let parser = NemApiResponseParser::new();

    let response_body = "{}";

    let result = parser.parse_chat_completion_response(response_body, "gemini");
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.choices.len(), 1);
}

/// Test parser configuration
#[test]
fn test_parser_configuration() {
    let config = ParserConfig {
        strip_html_entities: false,
        remove_control_chars: false,
        clean_ui_chrome: false,
        strip_thinking: false,
        remove_empty_tool_calls: false,
        max_parse_retries: 5,
    };

    let parser = NemApiResponseParser::with_config(config);

    assert_eq!(parser.config().strip_html_entities, false);
    assert_eq!(parser.config().remove_control_chars, false);
    assert_eq!(parser.config().clean_ui_chrome, false);
    assert_eq!(parser.config().strip_thinking, false);
    assert_eq!(parser.config().max_parse_retries, 5);
}

/// Test setting parser configuration
#[test]
fn test_set_parser_configuration() {
    let mut parser = NemApiResponseParser::new();

    let new_config = ParserConfig {
        strip_html_entities: false,
        ..Default::default()
    };

    parser.set_config(new_config);

    assert_eq!(parser.config().strip_html_entities, false);
}

/// Test that parser can handle malformed JSON
#[test]
fn test_parse_malformed_json() {
    let parser = NemApiResponseParser::new();

    let response_body = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1234567890,
        "model": "gemini-chat",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello world"
            },
        "finish_reason": "stop"
    }"#;

    // This JSON is malformed (missing closing brace for choices array)
    let result = parser.parse_chat_completion_response(response_body, "gemini");

    // Should handle gracefully
    assert!(result.is_err());
}

/// Test cleaning zero-width spaces
#[test]
fn test_clean_zero_width_spaces() {
    let parser = NemApiResponseParser::new();

    let input = "Hello\u{200B}World\u{FEFF}";
    let cleaned = parser.clean_response_body(input, "gemini");

    assert!(!cleaned.contains("\u{200B}"));
    assert!(!cleaned.contains("\u{FEFF}"));
    assert_eq!(cleaned, "HelloWorld");
}

/// Test cleaning with all options disabled
#[test]
fn test_clean_with_all_options_disabled() {
    let config = ParserConfig {
        strip_html_entities: false,
        remove_control_chars: false,
        clean_ui_chrome: false,
        strip_thinking: false,
        ..Default::default()
    };

    let parser = NemApiResponseParser::with_config(config);

    let input = "Hello &lt;world&gt; <think>thinking</think> Send a message.";
    let cleaned = parser.clean_response_body(input, "gemini");

    // Nothing should be cleaned
    assert_eq!(cleaned, input);
}

/// Test cleaning with only HTML entities enabled
#[test]
fn test_clean_with_only_html_entities() {
    let config = ParserConfig {
        strip_html_entities: true,
        remove_control_chars: false,
        clean_ui_chrome: false,
        strip_thinking: false,
        ..Default::default()
    };

    let parser = NemApiResponseParser::with_config(config);

    let input = "Hello &lt;world&gt; <think>thinking</think> Send a message.";
    let cleaned = parser.clean_response_body(input, "gemini");

    // Only HTML entities should be cleaned
    assert!(!cleaned.contains("&lt;"));
    assert!(!cleaned.contains("&gt;"));
    assert!(cleaned.contains("<think>thinking</think>"));
    assert!(cleaned.contains("Send a message."));
}
