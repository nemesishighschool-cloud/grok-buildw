//! HTTP client for NemApi proxy integration
//!
//! This client implements NemApi's unique approach:
//! - Context is managed SERVER-SIDE by the browser extension
//! - Only the latest user message is sent in subsequent requests
//! - System prompt and tools are sent ONLY in the first request
//! - No anonymous requests are sent

use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use eventsource_stream::Eventsource;
use futures_util::stream::BoxStream;
use futures_util::{Stream, StreamExt};
use indexmap::IndexMap;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, RequestBuilder, Response};
use serde::Serialize;
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tracing::{debug, error, info, trace, warn};

use xai_grok_sampling_types::{
    ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse, ConversationRequest,
    ConversationResponse, CreateResponseWrapper, MessagesRequestWrapper, Result as SamplingResult,
    SamplingError, SentCredential,
};

use crate::config::NemApiConfig;
use crate::error::{NemApiError, Result};
use crate::parser::NemApiResponseParser;
use crate::provider::NemApiProvider;

/// HTTP client for NemApi proxy
///
/// Implements NemApi's unique conversation management:
/// - First request: sends system prompt + user message + tools
/// - Subsequent requests: sends ONLY the new user message
/// - Server maintains context via browser extension
#[derive(Debug, Clone)]
pub struct NemApiClient {
    /// HTTP client
    client: Client,

    /// Configuration
    config: NemApiConfig,

    /// Provider manager
    provider: NemApiProvider,

    /// Response parser
    parser: NemApiResponseParser,

    /// Request counter for correlation
    request_counter: Arc<Mutex<u64>>,

    /// Track if this is the first request in a conversation
    /// When true: include system prompt and tools
    /// When false: send only user message
    is_first_request: Arc<Mutex<bool>>,
}

impl NemApiClient {
    /// Create a new NemApiClient
    pub fn new(config: NemApiConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .connect_timeout(Duration::from_secs(config.connect_timeout_secs))
            .build()
            .map_err(|e| NemApiError::ConnectionError(e.to_string()))?;

        let provider = NemApiProvider::new(config.clone());
        let parser = NemApiResponseParser::new();

        // Validate configuration
        config.validate()?;

        Ok(Self {
            client,
            config,
            provider,
            parser,
            request_counter: Arc::new(Mutex::new(0)),
            is_first_request: Arc::new(Mutex::new(true)),
        })
    }

    /// Create with default configuration
    pub fn with_defaults() -> Result<Self> {
        Self::new(NemApiConfig::default())
    }

    /// Create with gemini-chat as default for testing
    pub fn with_gemini_test_config() -> Result<Self> {
        let config = NemApiConfig::gemini_test_config();
        Self::new(config)
    }

    /// Get the configuration
    pub fn config(&self) -> &NemApiConfig {
        &self.config
    }

    /// Get the provider manager
    pub fn provider(&self) -> &NemApiProvider {
        &self.provider
    }

    /// Get the next request ID
    pub async fn next_request_id(&self) -> String {
        let mut counter = self.request_counter.lock().await;
        *counter += 1;
        format!("nemapi-req-{}", *counter)
    }

    /// Reset conversation state (new conversation)
    pub async fn reset_conversation(&self) {
        let mut first = self.is_first_request.lock().await;
        *first = true;
        debug!("Conversation reset - next request will include system prompt and tools");
    }

    /// Mark that first request is done
    async fn mark_first_request_done(&self) {
        let mut first = self.is_first_request.lock().await;
        *first = false;
        debug!("First request completed - subsequent requests will send only user message");
    }

    /// Check if this is the first request in the conversation
    pub async fn is_first_request(&self) -> bool {
        let first = self.is_first_request.lock().await;
        *first
    }

    /// Build the base request with common headers
    fn build_request(&self, method: &str, path: &str) -> RequestBuilder {
        let base_url = self.config.effective_base_url();
        let url = format!("{}{}", base_url.trim_end_matches('/'), path);

        let mut builder = self.client.request(method, &url);

        // Add content type
        builder = builder.header(CONTENT_TYPE, "application/json");

        // Add user agent
        builder = builder.header(
            HeaderName::from_static("user-agent"),
            HeaderValue::from_static("NemApi-Client/1.0"),
        );

        // Add authorization if API key is configured
        if let Some(ref api_key) = self.config.api_key {
            let auth_header = match self.config.auth_scheme {
                xai_grok_sampler::AuthScheme::Bearer => {
                    format!("Bearer {}", api_key)
                }
                xai_grok_sampler::AuthScheme::XApiKey => {
                    format!("ApiKey {}", api_key)
                }
            };
            if let Ok(header_value) = HeaderValue::from_str(&auth_header) {
                builder = builder.header(AUTHORIZATION, header_value);
            }
        }

        // Add extra headers
        for (key, value) in &self.config.extra_headers {
            if let (Ok(header_name), Ok(header_value)) = (
                HeaderName::from_bytes(key.as_bytes()),
                HeaderValue::from_str(value),
            ) {
                builder = builder.header(header_name, header_value);
            }
        }

        builder
    }

    /// Send a chat completion request
    ///
    /// IMPORTANT: NemApi manages context SERVER-SIDE via browser extension.
    /// - First request in NEW conversation: sends system prompt + ALL messages + tools + user message in ONE request
    /// - Subsequent requests in SAME conversation: sends ONLY the new user message
    /// - Context is maintained by the browser extension, NOT sent back each time
    /// - For resuming a conversation: use /resume command to select session, then continue normally
    /// - Session history is displayed but NOT sent back to API - only user prompt is sent
    pub async fn send_chat_completion(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        let request_id = self.next_request_id().await;
        debug!(
            "Sending chat completion request {}: model={:?}, messages={}, stream={}",
            request_id,
            request.model,
            request.messages.len(),
            request.stream
        );

        // Resolve the model to get provider info
        let model_str = request
            .model
            .clone()
            .unwrap_or_else(|| self.config.effective_default_model().to_string());
        let (provider_id, canonical_model) = self.provider.resolve_model(&model_str)?;
        debug!(
            "Resolved model '{}' to provider '{}' with canonical model '{}'",
            model_str, provider_id, canonical_model
        );

        // Update request model with canonical model
        request.model = Some(canonical_model.clone());

        // A resumed local transcript is display-only; its saved messages must not
        // be replayed into NemApi's already-selected browser conversation.
        let is_first_request =
            self.is_first_request().await && Self::is_new_conversation_request(&request);

        // Build the request body according to NemApi's method:
        // - First request: sends system prompts, tools, and the current user message
        // - Subsequent requests: sends ONLY the latest user message
        // The browser extension maintains the context server-side
        let body =
            self.build_nemapi_body(&request, is_first_request, &provider_id, &canonical_model)?;

        // Send the request
        let response = self
            .build_request("POST", "/chat/completions")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send chat completion request: {}", e);
                e
            })?;

        // Mark first request as done (subsequent requests will send only user message)
        // Note: Server maintains context, session history is displayed but NOT sent back
        if is_first_request && response.status().is_success() {
            self.mark_first_request_done().await;
        }

        // Parse the response
        // Note: Server maintains context, so we don't need to send history back
        self.parse_chat_completion_response(response, &request_id, &provider_id)
            .await
    }

    /// Build request body for NemApi
    ///
    /// NemApi's unique approach:
    /// - If first request: include system prompts + current user message + tools
    /// - If not first: include ONLY the latest user message
    fn build_nemapi_body(
        &self,
        request: &ChatCompletionRequest,
        include_system_and_tools: bool,
        provider_id: &str,
        canonical_model: &str,
    ) -> Result<Value> {
        // Extract the latest user message
        let latest_user_message = self.extract_latest_user_message(request);

        if include_system_and_tools {
            // FIRST REQUEST: Send the system prompt, current user turn, and tools.
            debug!("Building FIRST request body with system prompt and tools");

            let mut messages = Vec::new();
            let latest_user_index = request
                .messages
                .iter()
                .rposition(|msg| msg.role == "user")
                .ok_or_else(|| {
                    NemApiError::Generic("No user message found in request".to_string())
                })?;

            for (index, msg) in request.messages.iter().enumerate() {
                if msg.role == "system" || index == latest_user_index {
                    messages.push(json!({
                        "role": msg.role.clone(),
                        "content": msg.content.clone(),
                    }));
                    let Some(msg_json) = messages.last_mut() else {
                        return Err(NemApiError::Generic(
                            "Failed to build initial NemApi messages".to_string(),
                        ));
                    };

                    // Add name if present
                    if let Some(name) = &msg.name {
                        msg_json["name"] = json!(name);
                    }

                    // Add tool calls if present
                    if let Some(tool_calls) = &msg.tool_calls {
                        msg_json["tool_calls"] = json!(tool_calls);
                    }
                }
            }

            let mut body = json!({
                "model": canonical_model,
                "messages": messages,
                "stream": request.stream,
            });

            // Add tools if present
            if !request.tools.is_empty() {
                body["tools"] = json!(request.tools);
            }

            // Add tool_choice if present
            if let Some(tool_choice) = &request.tool_choice {
                body["tool_choice"] = json!(tool_choice);
            }

            // Add max_tokens if present
            if let Some(max_tokens) = request.max_tokens {
                body["max_tokens"] = json!(max_tokens);
            }

            // Add temperature if present
            if let Some(temperature) = request.temperature {
                body["temperature"] = json!(temperature);
            }

            // Add top_p if present
            if let Some(top_p) = request.top_p {
                body["top_p"] = json!(top_p);
            }

            // Add NemApi-specific config
            self.add_nemapi_config(&mut body, provider_id, true);

            Ok(body)
        } else {
            // SUBSEQUENT REQUEST: Send ONLY the latest user message
            debug!("Building SUBSEQUENT request body with only latest user message");

            if latest_user_message.is_none() {
                return Err(NemApiError::Generic(
                    "No user message found in request".to_string(),
                ));
            }

            let mut body = json!({
                "model": canonical_model,
                "messages": [latest_user_message.unwrap()],
                "stream": request.stream,
            });

            // Add NemApi-specific config
            self.add_nemapi_config(&mut body, provider_id, false);

            Ok(body)
        }
    }

    /// Extract the latest user message from a request
    fn extract_latest_user_message(&self, request: &ChatCompletionRequest) -> Option<Value> {
        for msg in request.messages.iter().rev() {
            if msg.role == "user" {
                let mut msg_json = json!({
                    "role": "user",
                    "content": msg.content.clone()
                });

                // Add name if present
                if let Some(name) = &msg.name {
                    msg_json["name"] = json!(name);
                }

                return Some(msg_json);
            }
        }
        None
    }

    fn is_new_conversation_request(request: &ChatCompletionRequest) -> bool {
        let mut user_count = 0;
        request
            .messages
            .iter()
            .all(|message| match message.role.as_str() {
                "system" => true,
                "user" => {
                    user_count += 1;
                    true
                }
                _ => false,
            })
            && user_count == 1
    }

    /// Add NemApi-specific configuration to request body
    fn add_nemapi_config(&self, body: &mut Value, provider_id: &str, fresh_chat: bool) {
        // Add provider hint
        body["provider"] = json!(provider_id);

        // Start a browser conversation only for a new local conversation.
        body["fresh_chat"] = json!(fresh_chat && self.config.fresh_chat);

        if self.config.premium_md {
            body["premium_md"] = json!(true);
        }

        // Add context window if configured
        if self.config.context_window > 0 {
            body["context_window"] = json!(self.config.context_window);
        }
    }

    /// Parse chat completion response
    async fn parse_chat_completion_response(
        &self,
        response: Response,
        request_id: &str,
        provider_id: &str,
    ) -> Result<ChatCompletionResponse> {
        let status = response.status();

        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "Chat completion request {} failed with status {}: {}",
                request_id, status, error_body
            );

            // Try to parse error response
            if let Ok(error_json) = serde_json::from_str::<Value>(&error_body) {
                if let Some(error_msg) = error_json.get("error").and_then(|e| e.get("message")) {
                    if let Some(msg) = error_msg.as_str() {
                        return Err(NemApiError::HttpStatus {
                            status,
                            message: msg.to_string(),
                        });
                    }
                }
            }

            return Err(NemApiError::HttpStatus {
                status,
                message: error_body,
            });
        }

        // Check if streaming
        let is_stream = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .map(|h| h.contains("text/event-stream"))
            .unwrap_or(false);

        if is_stream {
            self.parse_streaming_response(response, request_id, provider_id)
                .await
        } else {
            self.parse_non_streaming_response(response, request_id, provider_id)
                .await
        }
    }

    /// Parse non-streaming response
    async fn parse_non_streaming_response(
        &self,
        response: Response,
        request_id: &str,
        provider_id: &str,
    ) -> Result<ChatCompletionResponse> {
        let body = response.text().await?;
        trace!("Non-streaming response {}: {}", request_id, body);

        // Parse the response using our custom parser
        let parsed = self
            .parser
            .parse_chat_completion_response(&body, provider_id)?;

        Ok(parsed)
    }

    /// Parse streaming response
    async fn parse_streaming_response(
        &self,
        response: Response,
        request_id: &str,
        provider_id: &str,
    ) -> Result<ChatCompletionResponse> {
        // For streaming, we need to collect all chunks and reconstruct the full response
        let bytes = response.bytes().await?;
        let body = String::from_utf8_lossy(&bytes);

        // Parse the accumulated stream
        let parsed = self.parser.parse_streaming_response(&body, provider_id)?;

        Ok(parsed)
    }

    /// Send a streaming chat completion request
    pub async fn send_chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<impl Stream<Item = Result<ChatCompletionChunk>>> {
        let request_id = self.next_request_id().await;
        debug!(
            "Sending streaming chat completion request {}: model={:?}",
            request_id, request.model
        );

        // Resolve the model
        let model_str = request
            .model
            .clone()
            .unwrap_or_else(|| self.config.effective_default_model().to_string());
        let (provider_id, canonical_model) = self.provider.resolve_model(&model_str)?;

        let is_first = self.is_first_request().await && Self::is_new_conversation_request(&request);

        // Build the request body
        let mut req_clone = request.clone();
        req_clone.model = Some(canonical_model.clone());
        let body = self.build_nemapi_body(&req_clone, is_first, &provider_id, &canonical_model)?;

        // Send the request
        let response = self
            .build_request("POST", "/chat/completions")
            .json(&body)
            .header("Accept", "text/event-stream")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send streaming request: {}", e);
                e
            })?;

        let status = response.status();
        if !status.is_success() {
            return Err(NemApiError::HttpStatus {
                status,
                message: format!("Streaming request failed with status {}", status),
            });
        }
        if is_first {
            self.mark_first_request_done().await;
        }

        // Create a stream from the response
        let byte_stream = response.bytes_stream();

        // Process the stream through our parser
        let chunk_stream = self.parser.parse_streaming_chunks(byte_stream, provider_id);

        Ok(chunk_stream)
    }

    /// Get the effective base URL
    pub fn effective_base_url(&self) -> String {
        self.config.effective_base_url()
    }

    /// Get the default model
    pub fn default_model(&self) -> &str {
        self.config.effective_default_model()
    }

    /// Get the default provider
    pub fn default_provider(&self) -> &str {
        self.config.effective_default_provider()
    }
}

impl Default for NemApiClient {
    fn default() -> Self {
        Self::with_defaults().expect("Failed to create default NemApiClient")
    }
}

/// Stream wrapper that implements the Stream trait for NemApi streaming responses
pub struct NemApiStream {
    inner: BoxStream<'static, Result<ChatCompletionChunk>>,
}

impl Stream for NemApiStream {
    type Item = Result<ChatCompletionChunk>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

impl From<BoxStream<'static, Result<ChatCompletionChunk>>> for NemApiStream {
    fn from(stream: BoxStream<'static, Result<ChatCompletionChunk>>) -> Self {
        Self { inner: stream }
    }
}
