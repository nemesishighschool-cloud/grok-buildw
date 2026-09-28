//! Adapter to integrate NemApi provider with xai-grok-sampler
//!
//! This module provides a bridge between the NemApi client and the sampler's
//! expected interface. It wraps the NemApiClient to provide the same methods
//! that the sampler expects from its SamplingClient.

use std::sync::Arc;
use std::time::Duration;

use eventsource_stream::Eventsource;
use futures_util::stream::BoxStream;
use futures_util::{Stream, StreamExt};
use indexmap::IndexMap;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde::Serialize;
use tracing::{debug, error, info, trace, warn};

use xai_grok_sampling_types::{
    ApiBackend, ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse,
    ConversationRequest, ConversationResponse, CreateResponseWrapper, MessagesRequestWrapper,
    ResponseModelMetadata, Result as SamplingResult, SamplingError, SentCredential,
};

use crate::client::NemApiClient;
use crate::config::NemApiConfig;
use crate::error::{NemApiError, Result};

/// Adapter that wraps NemApiClient to provide SamplingClient-compatible interface
///
/// This adapter allows the existing Grok Build sampler infrastructure to work
/// seamlessly with NemApi by implementing the expected interface.
#[derive(Debug, Clone)]
pub struct NemApiSamplingClient {
    /// Inner NemApi client
    inner: NemApiClient,

    /// Default headers for all requests
    default_headers: HeaderMap,

    /// Base URL for NemApi
    base_url: String,

    /// Default model
    default_model: String,

    /// API backend type
    api_backend: ApiBackend,

    /// Whether to use HTTP/1.1
    force_http1: bool,
}

impl NemApiSamplingClient {
    /// Create a new NemApi sampling client
    pub fn new(config: NemApiConfig) -> Result<Self> {
        let inner = NemApiClient::new(config.clone())?;

        // Build default headers
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Add authorization if configured
        if let Some(ref api_key) = config.api_key {
            let auth_header = match config.auth_scheme {
                xai_grok_sampler::AuthScheme::Bearer => {
                    format!("Bearer {}", api_key)
                }
                xai_grok_sampler::AuthScheme::XApiKey => {
                    format!("ApiKey {}", api_key)
                }
            };
            if let Ok(header_value) = HeaderValue::from_str(&auth_header) {
                headers.insert(AUTHORIZATION, header_value);
            }
        }

        // Add extra headers from config
        for (key, value) in &config.extra_headers {
            if let (Ok(header_name), Ok(header_value)) = (
                HeaderName::from_bytes(key.as_bytes()),
                HeaderValue::from_str(value),
            ) {
                headers.insert(header_name, header_value);
            }
        }

        Ok(Self {
            inner,
            default_headers: headers,
            base_url: config.effective_base_url(),
            default_model: config.effective_default_model().to_string(),
            api_backend: config.api_backend.clone(),
            force_http1: false,
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

    /// Get the API backend
    pub fn api_backend(&self) -> ApiBackend {
        self.api_backend.clone()
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the default model
    pub fn default_model(&self) -> &str {
        &self.default_model
    }

    /// Send a chat completion request
    ///
    /// This implements NemApi's unique approach:
    /// - First request in conversation: sends system + all messages + tools
    /// - Subsequent requests: sends ONLY the latest user message
    pub async fn chat_completion(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, SamplingError> {
        // Ensure we have a model
        if request.model.is_none() {
            request.model = Some(self.default_model.clone());
        }

        // Send via NemApi client
        match self.inner.send_chat_completion(request).await {
            Ok(response) => Ok(response),
            Err(e) => {
                error!("NemApi chat completion error: {}", e);
                Err(self.convert_nemapi_error(e))
            }
        }
    }

    /// Send a streaming chat completion request
    pub async fn chat_completion_stream(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<
        (BoxStream<'static, Result<ChatCompletionChunk, SamplingError>>, Option<ResponseModelMetadata>),
        SamplingError,
    > {
        // Ensure we have a model
        if request.model.is_none() {
            request.model = Some(self.default_model.clone());
        }

        // Send streaming request via NemApi client
        let stream = match self.inner.send_chat_completion_stream(request).await {
            Ok(s) => s,
            Err(e) => {
                error!("NemApi streaming chat completion error: {}", e);
                return Err(self.convert_nemapi_error(e));
            }
        };

        // Convert stream to use SamplingError and box it
        let converted_stream = stream
            .map(|result| result.map_err(|e| self.convert_nemapi_error(e)))
            .boxed();

        Ok((converted_stream, None))
    }

    /// Convert NemApiError to SamplingError
    fn convert_nemapi_error(&self, error: NemApiError) -> SamplingError {
        match error {
            NemApiError::HttpRequestFailed(msg) => SamplingError::Http(reqwest::Error::new(
                reqwest::error::Kind::Other,
                msg,
            )),
            NemApiError::HttpStatus { status, message } => {
                SamplingError::Http(reqwest::Error::new(
                    reqwest::error::Kind::Status(status),
                    message,
                ))
            }
            NemApiError::ConnectionError(msg) => {
                SamplingError::Http(reqwest::Error::new(
                    reqwest::error::Kind::Connect,
                    msg,
                ))
            }
            NemApiError::Timeout(secs) => {
                SamplingError::Http(reqwest::Error::new(
                    reqwest::error::Kind::Timeout,
                    format!("Request timeout after {} seconds", secs),
                ))
            }
            NemApiError::AuthenticationError(msg) => {
                SamplingError::Auth {
                    message: msg,
                    credential: SentCredential::Unknown,
                }
            }
            NemApiError::ProviderNotConfigured(provider) => {
                SamplingError::InvalidConfiguration(format!(
                    "Provider '{}' is not configured in NemApi",
                    provider
                ))
            }
            NemApiError::ModelNotFound(model, provider) => {
                SamplingError::InvalidConfiguration(format!(
                    "Model '{}' not found for provider '{}'",
                    model, provider
                ))
            }
            NemApiError::DeserializationError(msg) => {
                SamplingError::Serialization(serde_json::Error::custom(msg))
            }
            NemApiError::InvalidResponseFormat(msg) => {
                SamplingError::InvalidResponse(format!("NemApi: {}", msg))
            }
            NemApiError::ExtensionNotConnected => {
                SamplingError::ServiceUnavailable("NemApi browser extension is not connected".into())
            }
            NemApiError::NoProviderTabSelected(provider) => {
                SamplingError::InvalidConfiguration(format!(
                    "No tab selected for provider '{}' in NemApi",
                    provider
                ))
            }
            NemApiError::RateLimitExceeded(msg) => {
                SamplingError::RateLimited(msg)
            }
            NemApiError::StreamParsingError(msg) => {
                SamplingError::InvalidResponse(format!("Stream parsing error: {}", msg))
            }
            NemApiError::ToolCallParsingError(msg) => {
                SamplingError::InvalidResponse(format!("Tool call parsing error: {}", msg))
            }
            NemApiError::SamplingError(e) => e,
            NemApiError::IoError(e) => SamplingError::Http(reqwest::Error::new(
                reqwest::error::Kind::Io,
                e.to_string(),
            )),
            NemApiError::Generic(msg) => {
                SamplingError::InvalidResponse(msg)
            }
        }
    }

    /// Reset conversation state (new conversation)
    pub async fn reset_conversation(&self) {
        self.inner.reset_conversation().await;
    }

    /// Check if this is the first request
    pub async fn is_first_request(&self) -> bool {
        self.inner.is_first_request().await
    }

    /// Get the inner NemApi client
    pub fn inner(&self) -> &NemApiClient {
        &self.inner
    }

    /// Get the inner NemApi client (mutable)
    pub fn inner_mut(&mut self) -> &mut NemApiClient {
        &mut self.inner
    }
}

impl Default for NemApiSamplingClient {
    fn default() -> Self {
        Self::with_defaults().expect("Failed to create default NemApiSamplingClient")
    }
}

/// Factory function to create a NemApi-compatible sampling client
pub fn create_nemapi_sampling_client(config: NemApiConfig) -> Result<NemApiSamplingClient> {
    NemApiSamplingClient::new(config)
}

/// Create with default configuration
pub fn create_default_nemapi_sampling_client() -> Result<NemApiSamplingClient> {
    NemApiSamplingClient::with_defaults()
}

/// Create with gemini-chat as default for testing
pub fn create_gemini_test_sampling_client() -> Result<NemApiSamplingClient> {
    NemApiSamplingClient::with_gemini_test_config()
}
