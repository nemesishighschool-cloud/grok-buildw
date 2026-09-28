//! Error types for NemApi provider integration
//!
//! This module defines all error types that can occur when using the NemApi provider.

use std::fmt;

use reqwest::StatusCode;
use serde_json::Error as SerdeJsonError;
use xai_grok_sampling_types::SamplingError;

/// Result type for NemApi operations
pub type Result<T> = std::result::Result<T, NemApiError>;

/// Error types for NemApi provider
#[derive(Debug)]
pub enum NemApiError {
    /// HTTP request failed
    HttpRequestFailed(String),

    /// HTTP status error with status code and message
    HttpStatus {
        status: StatusCode,
        message: String,
    },

    /// Connection error
    ConnectionError(String),

    /// Timeout error with duration in seconds
    Timeout(u64),

    /// Authentication error
    AuthenticationError(String),

    /// Provider not configured
    ProviderNotConfigured(String),

    /// Model not found for provider
    ModelNotFound(String, String),

    /// Deserialization error
    DeserializationError(String),

    /// Invalid response format
    InvalidResponseFormat(String),

    /// Extension not connected
    ExtensionNotConnected,

    /// No provider tab selected
    NoProviderTabSelected(String),

    /// Rate limit exceeded
    RateLimitExceeded(String),

    /// Stream parsing error
    StreamParsingError(String),

    /// Tool call parsing error
    ToolCallParsingError(String),

    /// Sampling error (wrapped)
    SamplingError(SamplingError),

    /// IO error
    IoError(std::io::Error),

    /// Generic error
    Generic(String),
}

impl fmt::Display for NemApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NemApiError::HttpRequestFailed(msg) => write!(f, "HTTP request failed: {}", msg),
            NemApiError::HttpStatus { status, message } => {
                write!(f, "HTTP {}: {}", status, message)
            }
            NemApiError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            NemApiError::Timeout(secs) => write!(f, "Request timeout after {} seconds", secs),
            NemApiError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            NemApiError::ProviderNotConfigured(provider) => {
                write!(f, "Provider '{}' is not configured in NemApi", provider)
            }
            NemApiError::ModelNotFound(model, provider) => {
                write!(f, "Model '{}' not found for provider '{}'", model, provider)
            }
            NemApiError::DeserializationError(msg) => {
                write!(f, "Deserialization error: {}", msg)
            }
            NemApiError::InvalidResponseFormat(msg) => {
                write!(f, "Invalid response format: {}", msg)
            }
            NemApiError::ExtensionNotConnected => {
                write!(f, "NemApi browser extension is not connected")
            }
            NemApiError::NoProviderTabSelected(provider) => {
                write!(f, "No tab selected for provider '{}' in NemApi", provider)
            }
            NemApiError::RateLimitExceeded(msg) => write!(f, "Rate limit exceeded: {}", msg),
            NemApiError::StreamParsingError(msg) => write!(f, "Stream parsing error: {}", msg),
            NemApiError::ToolCallParsingError(msg) => write!(f, "Tool call parsing error: {}", msg),
            NemApiError::SamplingError(e) => write!(f, "Sampling error: {}", e),
            NemApiError::IoError(e) => write!(f, "IO error: {}", e),
            NemApiError::Generic(msg) => write!(f, "NemApi error: {}", msg),
        }
    }
}

impl std::error::Error for NemApiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NemApiError::IoError(e) => Some(e),
            NemApiError::SamplingError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for NemApiError {
    fn from(err: std::io::Error) -> Self {
        NemApiError::IoError(err)
    }
}

impl From<SerdeJsonError> for NemApiError {
    fn from(err: SerdeJsonError) -> Self {
        NemApiError::DeserializationError(err.to_string())
    }
}

impl From<SamplingError> for NemApiError {
    fn from(err: SamplingError) -> Self {
        NemApiError::SamplingError(err)
    }
}

impl From<reqwest::Error> for NemApiError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            NemApiError::Timeout(120) // Default timeout
        } else if err.is_connect() {
            NemApiError::ConnectionError(err.to_string())
        } else if let Some(status) = err.status() {
            NemApiError::HttpStatus {
                status,
                message: err.to_string(),
            }
        } else {
            NemApiError::HttpRequestFailed(err.to_string())
        }
    }
}

impl NemApiError {
    /// Check if this is an authentication error
    pub fn is_auth_error(&self) -> bool {
        matches!(self, NemApiError::AuthenticationError(_))
    }

    /// Check if this is a connection error
    pub fn is_connection_error(&self) -> bool {
        matches!(self, NemApiError::ConnectionError(_) | NemApiError::ExtensionNotConnected)
    }

    /// Check if this is a timeout error
    pub fn is_timeout(&self) -> bool {
        matches!(self, NemApiError::Timeout(_))
    }

    /// Check if this is a rate limit error
    pub fn is_rate_limit(&self) -> bool {
        matches!(self, NemApiError::RateLimitExceeded(_))
    }

    /// Get the HTTP status code if this is an HTTP error
    pub fn http_status(&self) -> Option<StatusCode> {
        match self {
            NemApiError::HttpStatus { status, .. } => Some(*status),
            _ => None,
        }
    }
}

/// Create a NemApiError from an HTTP status code and message
pub fn http_status_error(status: StatusCode, message: impl Into<String>) -> NemApiError {
    NemApiError::HttpStatus {
        status,
        message: message.into(),
    }
}

/// Create a NemApiError for a generic error
pub fn generic_error(msg: impl Into<String>) -> NemApiError {
    NemApiError::Generic(msg.into())
}

/// Create a NemApiError for a deserialization error
pub fn deserialization_error(msg: impl Into<String>) -> NemApiError {
    NemApiError::DeserializationError(msg.into())
}

/// Create a NemApiError for an invalid response format
pub fn invalid_response_error(msg: impl Into<String>) -> NemApiError {
    NemApiError::InvalidResponseFormat(msg.into())
}
