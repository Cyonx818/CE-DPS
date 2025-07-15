// ABOUTME: Error types for API client operations
use reqwest::StatusCode;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request failed: {source}")]
    HttpError {
        #[from]
        source: reqwest::Error,
    },

    #[error("API error {status}: {message}")]
    ApiError {
        status: StatusCode,
        message: String,
        error_type: Option<String>,
    },

    #[error("Rate limit exceeded: {message}")]
    RateLimitError {
        message: String,
        retry_after: Option<Duration>,
        requests_remaining: Option<u32>,
        tokens_remaining: Option<u32>,
    },

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Request timeout after {duration:?}")]
    TimeoutError { duration: Duration },

    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },

    #[error("Invalid configuration: {0}")]
    ConfigurationError(String),

    #[error("Maximum retries exceeded")]
    MaxRetriesExceeded,

    #[error("Request validation failed: {0}")]
    ValidationError(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),
}

impl ApiError {
    pub fn is_retryable(&self) -> bool {
        match self {
            ApiError::HttpError { source } => source.is_timeout() || source.is_connect(),
            ApiError::ApiError { status, .. } => {
                status.is_server_error() || *status == StatusCode::TOO_MANY_REQUESTS
            }
            ApiError::RateLimitError { .. } => true,
            ApiError::TimeoutError { .. } => true,
            ApiError::ServiceUnavailable(_) => true,
            _ => false,
        }
    }

    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            ApiError::RateLimitError { retry_after, .. } => *retry_after,
            _ => None,
        }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
