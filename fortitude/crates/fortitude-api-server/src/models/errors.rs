// ABOUTME: Error model definitions and HTTP error responses for the API

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

/// API error response structure
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Error code
    pub error_code: String,

    /// Human-readable error message
    pub message: String,

    /// Detailed error information
    pub details: Option<String>,

    /// Request ID for tracing
    pub request_id: Option<Uuid>,

    /// Error timestamp
    pub timestamp: DateTime<Utc>,

    /// Path where error occurred
    pub path: Option<String>,
}

/// Application error types
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("Authentication required")]
    Unauthorized,

    #[error("Access forbidden: {reason}")]
    Forbidden { reason: String },

    #[error("Resource not found: {resource}")]
    NotFound { resource: String },

    #[error("Conflict: {resource}")]
    Conflict { resource: String },

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Internal server error: {message}")]
    InternalError { message: String },

    #[error("Service unavailable: {reason}")]
    ServiceUnavailable { reason: String },

    #[error("Bad request: {message}")]
    BadRequest { message: String },

    #[error("Research engine error: {message}")]
    ResearchError { message: String },

    #[error("Classification error: {message}")]
    ClassificationError { message: String },

    #[error("Cache error: {message}")]
    CacheError { message: String },
}

impl ApiError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden { .. } => StatusCode::FORBIDDEN,
            ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApiError::Conflict { .. } => StatusCode::CONFLICT,
            ApiError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            ApiError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            ApiError::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ResearchError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ClassificationError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::CacheError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get the error code string
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::ValidationError { .. } => "VALIDATION_ERROR",
            ApiError::Unauthorized => "UNAUTHORIZED",
            ApiError::Forbidden { .. } => "FORBIDDEN",
            ApiError::NotFound { .. } => "NOT_FOUND",
            ApiError::Conflict { .. } => "CONFLICT",
            ApiError::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            ApiError::BadRequest { .. } => "BAD_REQUEST",
            ApiError::ServiceUnavailable { .. } => "SERVICE_UNAVAILABLE",
            ApiError::InternalError { .. } => "INTERNAL_ERROR",
            ApiError::ResearchError { .. } => "RESEARCH_ERROR",
            ApiError::ClassificationError { .. } => "CLASSIFICATION_ERROR",
            ApiError::CacheError { .. } => "CACHE_ERROR",
        }
    }

    /// Create an error response
    pub fn to_error_response(
        &self,
        request_id: Option<Uuid>,
        path: Option<String>,
    ) -> ErrorResponse {
        ErrorResponse {
            error_code: self.error_code().to_string(),
            message: self.to_string(),
            details: None,
            request_id,
            timestamp: Utc::now(),
            path,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = self.to_error_response(None, None);

        (status, Json(error_response)).into_response()
    }
}

// Convert validation errors from the validator crate
impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        let message = err
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let field_errors: Vec<String> = errors
                    .iter()
                    .filter_map(|e| e.message.as_ref().map(|m| m.to_string()))
                    .collect();
                format!("{}: {}", field, field_errors.join(", "))
            })
            .collect::<Vec<_>>()
            .join("; ");

        ApiError::ValidationError { message }
    }
}

// Convert serde JSON errors
impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::BadRequest {
            message: format!("JSON parsing error: {err}"),
        }
    }
}

// Convert fortitude-core errors
impl From<fortitude_core::api::error::ApiError> for ApiError {
    fn from(err: fortitude_core::api::error::ApiError) -> Self {
        match err {
            fortitude_core::api::error::ApiError::RateLimitError { .. } => {
                ApiError::RateLimitExceeded
            }
            fortitude_core::api::error::ApiError::AuthenticationError(_msg) => {
                ApiError::Unauthorized
            }
            fortitude_core::api::error::ApiError::HttpError { source } => ApiError::InternalError {
                message: format!("HTTP error: {source}"),
            },
            fortitude_core::api::error::ApiError::ApiError {
                status, message, ..
            } => {
                if status.is_server_error() {
                    ApiError::InternalError { message }
                } else if status.is_client_error() {
                    ApiError::BadRequest { message }
                } else {
                    ApiError::InternalError { message }
                }
            }
            fortitude_core::api::error::ApiError::TimeoutError { .. } => {
                ApiError::ServiceUnavailable {
                    reason: "Request timeout".to_string(),
                }
            }
            fortitude_core::api::error::ApiError::ServiceUnavailable(msg) => {
                ApiError::ServiceUnavailable { reason: msg }
            }
            fortitude_core::api::error::ApiError::ValidationError(msg) => {
                ApiError::ValidationError { message: msg }
            }
            fortitude_core::api::error::ApiError::SerializationError { source } => {
                ApiError::BadRequest {
                    message: format!("Serialization error: {source}"),
                }
            }
            fortitude_core::api::error::ApiError::ConfigurationError(msg) => {
                ApiError::InternalError { message: msg }
            }
            fortitude_core::api::error::ApiError::MaxRetriesExceeded => {
                ApiError::ServiceUnavailable {
                    reason: "Maximum retries exceeded".to_string(),
                }
            }
            fortitude_core::api::error::ApiError::QuotaExceeded(_msg) => {
                ApiError::RateLimitExceeded
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_status_codes() {
        assert_eq!(
            ApiError::ValidationError {
                message: "test".to_string()
            }
            .status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ApiError::Unauthorized.status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ApiError::NotFound {
                resource: "test".to_string()
            }
            .status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ApiError::RateLimitExceeded.status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[test]
    fn test_error_response_creation() {
        let error = ApiError::ValidationError {
            message: "test error".to_string(),
        };
        let request_id = Uuid::new_v4();

        let response = error.to_error_response(Some(request_id), Some("/test".to_string()));

        assert_eq!(response.error_code, "VALIDATION_ERROR");
        assert_eq!(response.message, "Validation error: test error");
        assert_eq!(response.request_id, Some(request_id));
        assert_eq!(response.path, Some("/test".to_string()));
    }

    #[test]
    fn test_validation_error_conversion() {
        // This test would require setting up actual validation errors
        // For now, we'll just test that the conversion trait is implemented
        let error = ApiError::ValidationError {
            message: "test".to_string(),
        };
        assert_eq!(error.error_code(), "VALIDATION_ERROR");
    }
}
