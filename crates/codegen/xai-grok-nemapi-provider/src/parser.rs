//! Response parser for NemApi integration
//!
//! This module handles parsing responses from NemApi, which may have imperfections
//! due to the browser automation approach. It provides robust parsing that can
//! handle various response formats and clean up common issues.
//!
//! Key NemApi characteristics:
//! - Context is managed server-side (no need to send full history)
//! - First request includes system prompt + tools
//! - Subsequent requests only need the latest user message
//! - Responses may contain DOM artifacts that need cleaning

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use xai_grok_sampling_types::{
    ChatCompletionChunk, ChatCompletionResponse, Choice, Delta, Message, ToolCall, ToolCallChunk,
    Usage,
};

use crate::error::{NemApiError, Result};

/// Parser for NemApi responses
///
/// Handles the imperfections that can come from browser-automated responses:
/// - HTML entities in text
/// - Control characters
/// - UI chrome (buttons, labels, etc.)
/// - Thinking blocks
/// - Incomplete or malformed JSON
///
/// NemApi-specific: The context is managed server-side, so we don't need to
/// worry about maintaining conversation history in the parser.
#[derive(Debug, Clone)]
pub struct NemApiResponseParser {
    /// Track state across streaming chunks
    stream_state: NemApiStreamState,

    /// Configuration for parsing
    config: ParserConfig,
}

/// State for parsing streaming responses
#[derive(Debug, Clone, Default)]
struct NemApiStreamState {
    /// Current chunk index
    chunk_index: u32,

    /// Accumulated content for the current message
    current_content: String,

    /// Current tool calls being built
    current_tool_calls: Vec<ToolCallChunk>,

    /// Whether we're in a tool call
    in_tool_call: bool,

    /// Buffer for incomplete JSON
    json_buffer: String,
}

/// Configuration for the parser
#[derive(Debug, Clone)]
struct ParserConfig {
    /// Whether to strip HTML entities
    strip_html_entities: bool,

    /// Whether to remove control characters
    remove_control_chars: bool,

    /// Whether to clean UI chrome
    clean_ui_chrome: bool,

    /// Whether to strip thinking blocks
    strip_thinking: bool,

    /// Whether to remove empty tool calls
    remove_empty_tool_calls: bool,

    /// Maximum depth for JSON parsing retries
    max_parse_retries: usize,
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

impl NemApiResponseParser {
    /// Create a new parser with default configuration
    pub fn new() -> Self {
        Self {
            stream_state: NemApiStreamState::default(),
            config: ParserConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: ParserConfig) -> Self {
        Self {
            stream_state: NemApiStreamState::default(),
            config,
        }
    }

    /// Parse a complete chat completion response
    ///
    /// NemApi returns responses in OpenAI-compatible format, but may have
    /// artifacts from DOM extraction that need cleaning.
    pub fn parse_chat_completion_response(
        &self,
        response_text: &str,
        provider_id: &str,
    ) -> Result<ChatCompletionResponse> {
        // Clean the response first
        let cleaned = self.clean_response(response_text, provider_id);

        // Try to parse as JSON
        let parsed: Value = self.parse_json(&cleaned)?;

        // Convert to ChatCompletionResponse
        self.value_to_chat_completion_response(parsed, provider_id)
    }

    /// Parse a streaming response
    pub fn parse_streaming_response(
        &self,
        response_text: &str,
        provider_id: &str,
    ) -> Result<ChatCompletionResponse> {
        // For streaming, we need to handle the accumulated chunks
        // This is a simplified implementation that reconstructs from the stream
        
        // Clean the response
        let cleaned = self.clean_response(response_text, provider_id);

        // Try to parse as JSON (might be multiple JSON objects concatenated)
        // For now, we'll try to parse the last complete JSON object
        let parsed: Value = self.parse_json(&cleaned)?;

        // Convert to ChatCompletionResponse
        self.value_to_chat_completion_response(parsed, provider_id)
    }

    /// Parse a single chunk from a streaming response
    pub fn parse_chunk(&self, chunk_text: &str, provider_id: &str) -> Result<ChatCompletionChunk> {
        // Clean the chunk
        let cleaned = self.clean_response(chunk_text, provider_id);

        // Try to parse as JSON
        let parsed: Value = self.parse_json(&cleaned)?;

        // Convert to ChatCompletionChunk
        self.value_to_chat_completion_chunk(parsed, provider_id)
    }

    /// Clean response text from common NemApi imperfections
    fn clean_response(&self, text: &str, provider_id: &str) -> String {
        let mut cleaned = text.to_string();

        // Apply cleaning steps based on configuration
        if self.config.remove_control_chars {
            cleaned = self.remove_control_characters(&cleaned);
        }

        if self.config.strip_html_entities {
            cleaned = self.decode_html_entities(&cleaned);
        }

        if self.config.clean_ui_chrome {
            cleaned = self.remove_ui_chrome(&cleaned, provider_id);
        }

        if self.config.strip_thinking {
            cleaned = self.remove_thinking_blocks(&cleaned);
        }

        // Remove empty tool call blocks that might be artifacts
        if self.config.remove_empty_tool_calls {
            cleaned = self.remove_empty_tool_calls(&cleaned);
        }

        // Trim whitespace
        cleaned = cleaned.trim().to_string();

        cleaned
    }

    /// Remove control characters that can break JSON parsing
    fn remove_control_characters(&self, text: &str) -> String {
        // Remove control characters except newline and tab
        text.chars()
            .filter(|c| {
                let code = *c as u32;
                // Keep printable ASCII, newline, tab, and Unicode characters
                code >= 32 || code == 10 || code == 13 || code == 9 || code >= 128
            })
            .collect()
    }

    /// Decode HTML entities
    fn decode_html_entities(&self, text: &str) -> String {
        // Common HTML entities to decode
        let mut result = text.to_string();

        // Replace common entities
        result = result.replace("&amp;", "&");
        result = result.replace("&lt;", "<");
        result = result.replace("&gt;", ">");
        result = result.replace("&quot;", "\"");
        result = result.replace("&apos;", "'");
        result = result.replace("&nbsp;", " ");

        // More entities
        result = result.replace("&#39;", "'");
        result = result.replace("&#34;", "\"");
        result = result.replace("&#160;", " ");

        // Numeric entities
        result = result.replace("&#x2014;", "-");
        result = result.replace("&#x2013;", "-");

        result
    }

    /// Remove UI chrome (buttons, labels, etc.)
    fn remove_ui_chrome(&self, text: &str, provider_id: &str) -> String {
        // This is a simplified version - in practice, you'd have provider-specific patterns
        let mut result = text.to_string();

        // Remove common UI elements
        let ui_patterns = [
            "Copy", "Copied", "Download", "JSON", "Think", "Thinking",
            "Regenerate", "Retry", "Share", "Stop", "Edit", "Like", "Dislike",
            "Report", "Copy code", "Show more", "Show less", "Collapse", "Expand",
            "Send", "Send a message", "New chat", "Clear", "Reset",
        ];

        for pattern in &ui_patterns {
            // Case-insensitive removal of lines containing these patterns
            result = result
                .lines()
                .filter(|line| !line.to_lowercase().contains(&pattern.to_lowercase()))
                .collect::<Vec<_>>()
                .join("\n");
        }

        // Remove lines that are just punctuation or single words
        result = result
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && 
                !(trimmed.chars().all(|c| c.is_ascii_punctuation())) &&
                trimmed.len() > 1
            })
            .collect::<Vec<_>>()
            .join("\n");

        result
    }

    /// Remove thinking blocks
    fn remove_thinking_blocks(&self, text: &str) -> String {
        // Remove thinking blocks in various formats
        let mut result = text.to_string();

        // Remove <think>...</think> blocks
        while let Some(start) = result.find("<think>") {
            if let Some(end) = result.find("</think>") {
                result = result[..start].to_string() + &result[end + 9..];
            } else {
                break;
            }
        }

        // Remove <thinking>...</thinking> blocks
        while let Some(start) = result.find("<thinking>") {
            if let Some(end) = result.find("</thinking>") {
                result = result[..start].to_string() + &result[end + 11..];
            } else {
                break;
            }
        }

        // Remove ```think...``` blocks
        while let Some(start) = result.find("```think") {
            if let Some(end) = result.find("```") {
                result = result[..start].to_string() + &result[end + 3..];
            } else {
                break;
            }
        }

        // Remove <details>...</details> blocks (often used for thinking)
        while let Some(start) = result.find("<details") {
            if let Some(end) = result.find("</details>") {
                result = result[..start].to_string() + &result[end + 10..];
            } else {
                break;
            }
        }

        result
    }

    /// Remove empty tool call blocks
    fn remove_empty_tool_calls(&self, text: &str) -> String {
        // Remove empty tool call markers that might be artifacts
        let mut result = text.to_string();
        
        // Remove empty <tool> tags
        result = result.replace("<tool></tool>", "");
        result = result.replace("<tool> </tool>", "");
        
        // Remove empty tool call JSON
        result = result.replace("\"tool_calls\":[]", "");
        result = result.replace("\"tool_calls\": []", "");
        
        result
    }

    /// Parse JSON with retries for malformed input
    fn parse_json(&self, text: &str) -> Result<Value> {
        // Try direct parsing first
        if let Ok(parsed) = serde_json::from_str(text) {
            return Ok(parsed);
        }

        // Try to fix common issues
        for attempt in 0..self.config.max_parse_retries {
            let fixed = match attempt {
                0 => self.fix_json(text)?,
                1 => self.fix_json(&self.fix_json(text)??)?,
                _ => return Err(NemApiError::DeserializationError(
                    "Failed to parse JSON after retries".to_string(),
                )),
            };

            if let Ok(parsed) = serde_json::from_str(&fixed) {
                return Ok(parsed);
            }
        }

        Err(NemApiError::DeserializationError(
            "Failed to parse JSON".to_string(),
        ))
    }

    /// Attempt to fix common JSON issues
    fn fix_json(&self, text: &str) -> Result<String> {
        let mut result = text.to_string();

        // Fix unclosed strings
        let open_quotes = result.matches('"').count();
        if open_quotes % 2 != 0 {
            result.push('"');
        }

        // Fix unclosed braces
        let open_braces = result.matches('{').count() - result.matches('}').count();
        for _ in 0..open_braces {
            result.push('}');
        }

        // Fix unclosed brackets
        let open_brackets = result.matches('[').count() - result.matches(']').count();
        for _ in 0..open_brackets {
            result.push(']');
        }

        // Fix trailing commas
        result = result.replace(",\n}", "\n}");
        result = result.replace(",\n]", "\n]");

        // Fix double commas
        result = result.replace(",,", ",");

        Ok(result)
    }

    /// Convert Value to ChatCompletionResponse
    fn value_to_chat_completion_response(
        &self,
        value: Value,
        provider_id: &str,
    ) -> Result<ChatCompletionResponse> {
        // Handle different response formats that NemApi might return

        // Format 1: Standard OpenAI format
        if value.get("choices").is_some() {
            return self.parse_standard_format(value);
        }

        // Format 2: NemApi-specific format with result field
        if value.get("result").is_some() {
            return self.parse_nemapi_format(value, provider_id);
        }

        // Format 3: Simple text response
        if value.is_string() {
            return self.parse_simple_text_format(value);
        }

        // Format 4: Error response
        if value.get("error").is_some() {
            return Err(NemApiError::HttpStatus {
                status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                message: value.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()).unwrap_or("Unknown error").to_string(),
            });
        }

        Err(NemApiError::InvalidResponseFormat(
            "Unrecognized response format".to_string(),
        ))
    }

    /// Parse standard OpenAI format
    fn parse_standard_format(&self, value: Value) -> Result<ChatCompletionResponse> {
        let id = value.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let model = value.get("model").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
        let created = value.get("created").and_then(|v| v.as_i64()).unwrap_or(0);

        let choices: Vec<Choice> = value
            .get("choices")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|choice| self.value_to_choice(choice).ok())
                    .collect()
            })
            .unwrap_or_default();

        let usage = value
            .get("usage")
            .and_then(|v| self.value_to_usage(v).ok())
            .unwrap_or_default();

        Ok(ChatCompletionResponse {
            id,
            object: "chat.completion".to_string(),
            created,
            model,
            choices,
            usage: Some(usage),
            system_fingerprint: None,
        })
    }

    /// Parse NemApi-specific format
    fn parse_nemapi_format(&self, value: Value, provider_id: &str) -> Result<ChatCompletionResponse> {
        // NemApi might return: { "result": "...", "provider": "...", "model": "..." }
        let result_text = value.get("result").and_then(|v| v.as_str()).unwrap_or("");
        let model = value.get("model").and_then(|v| v.as_str()).unwrap_or(provider_id);

        // Clean the result text
        let cleaned_text = self.clean_response(result_text, provider_id);

        // Create a simple response
        let choice = Choice {
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content: Some(cleaned_text),
                tool_calls: None,
                name: None,
            },
            delta: None,
            finish_reason: Some("stop".to_string()),
        };

        Ok(ChatCompletionResponse {
            id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model.to_string(),
            choices: vec![choice],
            usage: Some(Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            }),
            system_fingerprint: None,
        })
    }

    /// Parse simple text format
    fn parse_simple_text_format(&self, value: Value) -> Result<ChatCompletionResponse> {
        if let Some(text) = value.as_str() {
            let cleaned_text = self.clean_response(text, "unknown");

            let choice = Choice {
                index: 0,
                message: Message {
                    role: "assistant".to_string(),
                    content: Some(cleaned_text),
                    tool_calls: None,
                    name: None,
                },
                delta: None,
                finish_reason: Some("stop".to_string()),
            };

            Ok(ChatCompletionResponse {
                id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
                object: "chat.completion".to_string(),
                created: chrono::Utc::now().timestamp(),
                model: "unknown".to_string(),
                choices: vec![choice],
                usage: Some(Usage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                }),
                system_fingerprint: None,
            })
        } else {
            Err(NemApiError::InvalidResponseFormat(
                "Expected string value".to_string(),
            ))
        }
    }

    /// Convert Value to Choice
    fn value_to_choice(&self, value: &Value) -> Result<Choice> {
        let index = value.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let finish_reason = value.get("finish_reason").and_then(|v| v.as_str()).map(|s| s.to_string());

        // Try to get message or delta
        let message = value.get("message").and_then(|v| self.value_to_message(v).ok());
        let delta = value.get("delta").and_then(|v| self.value_to_delta(v).ok());

        Ok(Choice {
            index,
            message,
            delta,
            finish_reason,
        })
    }

    /// Convert Value to Message
    fn value_to_message(&self, value: &Value) -> Result<Message> {
        let role = value.get("role").and_then(|v| v.as_str()).unwrap_or("assistant").to_string();
        let content = value.get("content").and_then(|v| v.as_str()).map(|s| s.to_string());
        let tool_calls = value.get("tool_calls").and_then(|v| self.value_to_tool_calls(v).ok());
        let name = value.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());

        // Clean content if present
        if let Some(ref content) = content {
            let cleaned = self.clean_response(content, "unknown");
            return Ok(Message {
                role,
                content: Some(cleaned),
                tool_calls,
                name,
            });
        }

        Ok(Message {
            role,
            content,
            tool_calls,
            name,
        })
    }

    /// Convert Value to Delta
    fn value_to_delta(&self, value: &Value) -> Result<Delta> {
        let content = value.get("content").and_then(|v| v.as_str()).map(|s| s.to_string());
        let role = value.get("role").and_then(|v| v.as_str()).map(|s| s.to_string());
        let tool_calls = value.get("tool_calls").and_then(|v| self.value_to_tool_call_chunks(v).ok());

        // Clean content if present
        if let Some(ref content) = content {
            let cleaned = self.clean_response(content, "unknown");
            return Ok(Delta {
                content: Some(cleaned),
                role,
                tool_calls,
            });
        }

        Ok(Delta {
            content,
            role,
            tool_calls,
        })
    }

    /// Convert Value to Vec<ToolCall>
    fn value_to_tool_calls(&self, value: &Value) -> Result<Vec<ToolCall>> {
        value
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|tc| self.value_to_tool_call(tc).ok())
                    .collect()
            })
            .ok_or_else(|| NemApiError::ToolCallParsingError("Expected array of tool calls".to_string()))
    }

    /// Convert Value to ToolCall
    fn value_to_tool_call(&self, value: &Value) -> Result<ToolCall> {
        let id = value.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let function = value.get("function").and_then(|v| self.value_to_function_call(v).ok()).unwrap_or_default();
        let r#type = value.get("type").and_then(|v| v.as_str()).unwrap_or("function").to_string();

        Ok(ToolCall {
            id,
            r#type,
            function,
        })
    }

    /// Convert Value to FunctionCall
    fn value_to_function_call(&self, value: &Value) -> Result<xai_grok_sampling_types::FunctionCall> {
        let name = value.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
        let arguments = value.get("arguments").and_then(|v| v.as_str()).map(|s| s.to_string());

        // Clean arguments if present
        if let Some(ref args) = arguments {
            let cleaned = self.clean_response(args, "unknown");
            return Ok(xai_grok_sampling_types::FunctionCall {
                name,
                arguments: Some(cleaned),
            });
        }

        Ok(xai_grok_sampling_types::FunctionCall {
            name,
            arguments,
        })
    }

    /// Convert Value to Vec<ToolCallChunk>
    fn value_to_tool_call_chunks(&self, value: &Value) -> Result<Vec<ToolCallChunk>> {
        value
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|tc| self.value_to_tool_call_chunk(tc).ok())
                    .collect()
            })
            .ok_or_else(|| NemApiError::ToolCallParsingError("Expected array of tool call chunks".to_string()))
    }

    /// Convert Value to ToolCallChunk
    fn value_to_tool_call_chunk(&self, value: &Value) -> Result<ToolCallChunk> {
        let id = value.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
        let index = value.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let r#type = value.get("type").and_then(|v| v.as_str()).unwrap_or("function").to_string();
        let function = value.get("function").and_then(|v| self.value_to_function_call_chunk(v).ok()).unwrap_or_default();

        Ok(ToolCallChunk {
            id,
            index,
            r#type,
            function,
        })
    }

    /// Convert Value to FunctionCallChunk
    fn value_to_function_call_chunk(&self, value: &Value) -> Result<xai_grok_sampling_types::FunctionCallChunk> {
        let name = value.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
        let arguments = value.get("arguments").and_then(|v| v.as_str()).map(|s| s.to_string());

        // Clean arguments if present
        if let Some(ref args) = arguments {
            let cleaned = self.clean_response(args, "unknown");
            return Ok(xai_grok_sampling_types::FunctionCallChunk {
                name,
                arguments: Some(cleaned),
            });
        }

        Ok(xai_grok_sampling_types::FunctionCallChunk {
            name,
            arguments,
        })
    }

    /// Convert Value to Usage
    fn value_to_usage(&self, value: &Value) -> Result<Usage> {
        let prompt_tokens = value.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as i64;
        let completion_tokens = value.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as i64;
        let total_tokens = value.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as i64;

        Ok(Usage {
            prompt_tokens,
            completion_tokens,
            total_tokens,
        })
    }

    /// Convert Value to ChatCompletionChunk
    fn value_to_chat_completion_chunk(&self, value: Value, provider_id: &str) -> Result<ChatCompletionChunk> {
        let id = value.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let model = value.get("model").and_then(|v| v.as_str()).unwrap_or(provider_id);
        let created = value.get("created").and_then(|v| v.as_i64()).unwrap_or(0);
        let object = value.get("object").and_then(|v| v.as_str()).unwrap_or("chat.completion.chunk");

        let choices: Vec<Choice> = value
            .get("choices")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|choice| self.value_to_choice(choice).ok())
                    .collect()
            })
            .unwrap_or_default();

        let usage = value.get("usage").and_then(|v| self.value_to_usage(v).ok());

        // For chunks, we typically have delta, not message
        // Clean the delta content
        let cleaned_choices: Vec<Choice> = choices
            .into_iter()
            .map(|mut choice| {
                if let Some(delta) = &mut choice.delta {
                    if let Some(ref content) = delta.content {
                        delta.content = Some(self.clean_response(content, provider_id));
                    }
                }
                choice
            })
            .collect();

        Ok(ChatCompletionChunk {
            id,
            object: object.to_string(),
            created,
            model: model.to_string(),
            choices: cleaned_choices,
            usage,
            system_fingerprint: None,
        })
    }
}

impl Default for NemApiResponseParser {
    fn default() -> Self {
        Self::new()
    }
}
