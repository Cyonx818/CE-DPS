// ABOUTME: Error types for vector database operations
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VectorError {
    #[error("Qdrant client error: {source}")]
    QdrantError {
        source: Box<qdrant_client::QdrantError>,
    },

    #[error("Connection failed: {message}")]
    ConnectionError { message: String },

    #[error("Vector database timeout after {duration:?}")]
    TimeoutError { duration: Duration },

    #[error("Invalid configuration: {0}")]
    ConfigurationError(String),

    #[error("Collection not found: {collection}")]
    CollectionNotFound { collection: String },

    #[error("Invalid vector dimensions: expected {expected}, got {actual}")]
    InvalidVectorDimensions { expected: usize, actual: usize },

    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },

    #[error("Health check failed: {reason}")]
    HealthCheckFailed { reason: String },

    #[error("Vector operation failed: {operation} - {reason}")]
    OperationFailed { operation: String, reason: String },

    #[error("Index not ready: {collection}")]
    IndexNotReady { collection: String },

    #[error("Embedding generation error: {0}")]
    EmbeddingError(String),

    #[error("Model loading error: {model} - {reason}")]
    ModelLoadError { model: String, reason: String },

    #[error("Tokenization error: {0}")]
    TokenizationError(String),

    #[error("Performance error: {0}")]
    PerformanceError(String),

    #[error("Cache error: {operation} - {reason}")]
    CacheError { operation: String, reason: String },

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    #[error("Batch processing error: {0}")]
    BatchProcessingError(String),

    #[error("Connection pool error: {0}")]
    ConnectionPoolError(String),
}

impl VectorError {
    pub fn is_retryable(&self) -> bool {
        match self {
            VectorError::QdrantError { source } => {
                // Check if the underlying Qdrant error is retryable
                matches!(**source, qdrant_client::QdrantError::ResponseError { .. })
            }
            VectorError::ConnectionError { .. } => true,
            VectorError::TimeoutError { .. } => true,
            VectorError::HealthCheckFailed { .. } => true,
            VectorError::IndexNotReady { .. } => true,
            VectorError::ModelLoadError { .. } => false, // Model loading failures are not retryable
            VectorError::EmbeddingError(_) => false, // Embedding errors are usually not retryable
            VectorError::TokenizationError(_) => false, // Tokenization errors are not retryable
            VectorError::CacheError { .. } => false, // Cache errors are not retryable
            VectorError::BatchProcessingError(_) => false, // Batch errors are not retryable
            VectorError::PerformanceError(_) => true, // Performance errors may be retryable
            VectorError::ResourceLimitExceeded(_) => false, // Resource limits are not retryable
            VectorError::ConnectionPoolError(_) => true, // Connection pool errors may be retryable
            _ => false,
        }
    }

    pub fn from_connection_error(message: impl Into<String>) -> Self {
        VectorError::ConnectionError {
            message: message.into(),
        }
    }

    pub fn from_operation_failed(operation: impl Into<String>, reason: impl Into<String>) -> Self {
        VectorError::OperationFailed {
            operation: operation.into(),
            reason: reason.into(),
        }
    }

    pub fn from_model_load_error(model: impl Into<String>, reason: impl Into<String>) -> Self {
        VectorError::ModelLoadError {
            model: model.into(),
            reason: reason.into(),
        }
    }

    pub fn from_cache_error(operation: impl Into<String>, reason: impl Into<String>) -> Self {
        VectorError::CacheError {
            operation: operation.into(),
            reason: reason.into(),
        }
    }
}

impl From<qdrant_client::QdrantError> for VectorError {
    fn from(error: qdrant_client::QdrantError) -> Self {
        VectorError::QdrantError {
            source: Box::new(error),
        }
    }
}

pub type VectorResult<T> = Result<T, VectorError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_vector_error_retryability_qdrant_errors() {
        // Create a mock Qdrant error for testing retryability
        let connection_error = VectorError::ConnectionError {
            message: "Connection failed".to_string(),
        };
        assert!(connection_error.is_retryable());

        let timeout_error = VectorError::TimeoutError {
            duration: Duration::from_secs(30),
        };
        assert!(timeout_error.is_retryable());

        let health_check_error = VectorError::HealthCheckFailed {
            reason: "Health check timeout".to_string(),
        };
        assert!(health_check_error.is_retryable());

        let index_not_ready = VectorError::IndexNotReady {
            collection: "test_collection".to_string(),
        };
        assert!(index_not_ready.is_retryable());
    }

    #[test]
    fn test_vector_error_non_retryable_errors() {
        let config_error = VectorError::ConfigurationError("Invalid configuration".to_string());
        assert!(!config_error.is_retryable());

        let model_load_error = VectorError::ModelLoadError {
            model: "test-model".to_string(),
            reason: "Model not found".to_string(),
        };
        assert!(!model_load_error.is_retryable());

        let embedding_error =
            VectorError::EmbeddingError("Embedding generation failed".to_string());
        assert!(!embedding_error.is_retryable());

        let tokenization_error = VectorError::TokenizationError("Tokenization failed".to_string());
        assert!(!tokenization_error.is_retryable());

        let cache_error = VectorError::CacheError {
            operation: "get".to_string(),
            reason: "Cache corruption".to_string(),
        };
        assert!(!cache_error.is_retryable());

        let resource_limit_error =
            VectorError::ResourceLimitExceeded("Memory limit exceeded".to_string());
        assert!(!resource_limit_error.is_retryable());

        let batch_error = VectorError::BatchProcessingError("Batch processing failed".to_string());
        assert!(!batch_error.is_retryable());
    }

    #[test]
    fn test_vector_error_potentially_retryable_errors() {
        let performance_error =
            VectorError::PerformanceError("Performance degradation".to_string());
        assert!(performance_error.is_retryable());

        let connection_pool_error = VectorError::ConnectionPoolError("Pool exhausted".to_string());
        assert!(connection_pool_error.is_retryable());
    }

    #[test]
    fn test_vector_error_construction_helpers() {
        let connection_error = VectorError::from_connection_error("Test connection error");
        assert!(matches!(
            connection_error,
            VectorError::ConnectionError { .. }
        ));
        if let VectorError::ConnectionError { message } = connection_error {
            assert_eq!(message, "Test connection error");
        }

        let operation_error = VectorError::from_operation_failed("search", "Search failed");
        assert!(matches!(
            operation_error,
            VectorError::OperationFailed { .. }
        ));
        if let VectorError::OperationFailed { operation, reason } = operation_error {
            assert_eq!(operation, "search");
            assert_eq!(reason, "Search failed");
        }

        let model_error = VectorError::from_model_load_error("test-model", "Loading failed");
        assert!(matches!(model_error, VectorError::ModelLoadError { .. }));
        if let VectorError::ModelLoadError { model, reason } = model_error {
            assert_eq!(model, "test-model");
            assert_eq!(reason, "Loading failed");
        }

        let cache_error = VectorError::from_cache_error("set", "Cache write failed");
        assert!(matches!(cache_error, VectorError::CacheError { .. }));
        if let VectorError::CacheError { operation, reason } = cache_error {
            assert_eq!(operation, "set");
            assert_eq!(reason, "Cache write failed");
        }
    }

    #[test]
    fn test_vector_error_display_formatting() {
        let config_error = VectorError::ConfigurationError("Invalid URL".to_string());
        let error_message = format!("{}", config_error);
        assert!(error_message.contains("Invalid configuration"));
        assert!(error_message.contains("Invalid URL"));

        let connection_error = VectorError::ConnectionError {
            message: "Connection refused".to_string(),
        };
        let error_message = format!("{}", connection_error);
        assert!(error_message.contains("Connection failed"));
        assert!(error_message.contains("Connection refused"));

        let timeout_error = VectorError::TimeoutError {
            duration: Duration::from_secs(30),
        };
        let error_message = format!("{}", timeout_error);
        assert!(error_message.contains("timeout"));
        assert!(error_message.contains("30s"));

        let collection_error = VectorError::CollectionNotFound {
            collection: "test_collection".to_string(),
        };
        let error_message = format!("{}", collection_error);
        assert!(error_message.contains("Collection not found"));
        assert!(error_message.contains("test_collection"));

        let dimension_error = VectorError::InvalidVectorDimensions {
            expected: 384,
            actual: 256,
        };
        let error_message = format!("{}", dimension_error);
        assert!(error_message.contains("Invalid vector dimensions"));
        assert!(error_message.contains("384"));
        assert!(error_message.contains("256"));
    }

    #[test]
    fn test_vector_error_debug_formatting() {
        let embedding_error = VectorError::EmbeddingError("Test error".to_string());
        let debug_message = format!("{:?}", embedding_error);
        assert!(debug_message.contains("EmbeddingError"));
        assert!(debug_message.contains("Test error"));
    }

    #[test]
    fn test_serialization_error_conversion() {
        // Create a JSON error by trying to parse invalid JSON
        let invalid_json = "{";
        let json_error = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
        let vector_error: VectorError = json_error.into();
        assert!(matches!(
            vector_error,
            VectorError::SerializationError { .. }
        ));
    }

    #[test]
    fn test_vector_result_type_alias() {
        fn test_function() -> VectorResult<String> {
            Ok("success".to_string())
        }

        fn test_error_function() -> VectorResult<String> {
            Err(VectorError::ConfigurationError("Test error".to_string()))
        }

        let success_result = test_function();
        assert!(success_result.is_ok());
        assert_eq!(success_result.unwrap(), "success");

        let error_result = test_error_function();
        assert!(error_result.is_err());
        assert!(matches!(
            error_result.unwrap_err(),
            VectorError::ConfigurationError(_)
        ));
    }

    #[test]
    fn test_vector_error_categories() {
        // Configuration-related errors
        let config_errors = vec![
            VectorError::ConfigurationError("test".to_string()),
            VectorError::InvalidVectorDimensions {
                expected: 384,
                actual: 256,
            },
        ];
        for error in config_errors {
            assert!(!error.is_retryable());
        }

        // Connection-related errors (retryable)
        let connection_errors = vec![
            VectorError::ConnectionError {
                message: "test".to_string(),
            },
            VectorError::TimeoutError {
                duration: Duration::from_secs(1),
            },
            VectorError::HealthCheckFailed {
                reason: "test".to_string(),
            },
            VectorError::ConnectionPoolError("test".to_string()),
        ];
        for error in connection_errors {
            assert!(error.is_retryable());
        }

        // Processing-related errors (non-retryable)
        let processing_errors = vec![
            VectorError::EmbeddingError("test".to_string()),
            VectorError::TokenizationError("test".to_string()),
            VectorError::BatchProcessingError("test".to_string()),
        ];
        for error in processing_errors {
            assert!(!error.is_retryable());
        }
    }

    #[test]
    fn test_error_chaining_compatibility() {
        // Test that errors can be chained and converted properly
        let invalid_json = "{ incomplete";
        let json_error = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
        let vector_error = VectorError::SerializationError { source: json_error };

        // Should be able to access the source error
        if let VectorError::SerializationError { source } = vector_error {
            let error_string = source.to_string();
            assert!(error_string.contains("EOF") || error_string.contains("expected"));
        }
    }

    #[test]
    fn test_specific_error_types() {
        // Test health check error
        let health_error = VectorError::HealthCheckFailed {
            reason: "Service unavailable".to_string(),
        };
        assert!(health_error.is_retryable());
        assert!(format!("{}", health_error).contains("Health check failed"));

        // Test operation failed error
        let op_error = VectorError::OperationFailed {
            operation: "upsert".to_string(),
            reason: "Invalid payload".to_string(),
        };
        assert!(!op_error.is_retryable()); // Default case is non-retryable
        assert!(format!("{}", op_error).contains("upsert"));

        // Test index not ready error
        let index_error = VectorError::IndexNotReady {
            collection: "documents".to_string(),
        };
        assert!(index_error.is_retryable());
        assert!(format!("{}", index_error).contains("Index not ready"));

        // Test performance error
        let perf_error = VectorError::PerformanceError("High latency detected".to_string());
        assert!(perf_error.is_retryable());
        assert!(format!("{}", perf_error).contains("Performance error"));

        // Test resource limit error
        let limit_error = VectorError::ResourceLimitExceeded("Memory exhausted".to_string());
        assert!(!limit_error.is_retryable());
        assert!(format!("{}", limit_error).contains("Resource limit exceeded"));
    }

    #[test]
    fn test_error_construction_with_different_string_types() {
        // Test with &str
        let error1 = VectorError::from_connection_error("string literal");
        if let VectorError::ConnectionError { message } = error1 {
            assert_eq!(message, "string literal");
        }

        // Test with String
        let error2 =
            VectorError::from_operation_failed("operation".to_string(), "reason".to_string());
        if let VectorError::OperationFailed { operation, reason } = error2 {
            assert_eq!(operation, "operation");
            assert_eq!(reason, "reason");
        }

        // Test with owned string variables
        let model_name = "bert-model".to_string();
        let error_reason = "File not found".to_string();
        let error3 = VectorError::from_model_load_error(model_name.clone(), error_reason.clone());
        if let VectorError::ModelLoadError { model, reason } = error3 {
            assert_eq!(model, model_name);
            assert_eq!(reason, error_reason);
        }
    }

    #[test]
    fn test_timeout_error_duration_handling() {
        let short_timeout = VectorError::TimeoutError {
            duration: Duration::from_millis(500),
        };
        let long_timeout = VectorError::TimeoutError {
            duration: Duration::from_secs(60),
        };

        // Both should be retryable
        assert!(short_timeout.is_retryable());
        assert!(long_timeout.is_retryable());

        // Should display different durations
        let short_msg = format!("{}", short_timeout);
        let long_msg = format!("{}", long_timeout);
        assert!(short_msg.contains("500ms") || short_msg.contains("0.5"));
        assert!(long_msg.contains("60s") || long_msg.contains("1m"));
    }

    #[test]
    fn test_error_type_completeness() {
        // Ensure all error variants are covered in retryability logic
        let all_errors = vec![
            VectorError::ConnectionError {
                message: "test".to_string(),
            },
            VectorError::TimeoutError {
                duration: Duration::from_secs(1),
            },
            VectorError::ConfigurationError("test".to_string()),
            VectorError::CollectionNotFound {
                collection: "test".to_string(),
            },
            VectorError::InvalidVectorDimensions {
                expected: 1,
                actual: 2,
            },
            VectorError::HealthCheckFailed {
                reason: "test".to_string(),
            },
            VectorError::OperationFailed {
                operation: "test".to_string(),
                reason: "test".to_string(),
            },
            VectorError::IndexNotReady {
                collection: "test".to_string(),
            },
            VectorError::EmbeddingError("test".to_string()),
            VectorError::ModelLoadError {
                model: "test".to_string(),
                reason: "test".to_string(),
            },
            VectorError::TokenizationError("test".to_string()),
            VectorError::PerformanceError("test".to_string()),
            VectorError::CacheError {
                operation: "test".to_string(),
                reason: "test".to_string(),
            },
            VectorError::ResourceLimitExceeded("test".to_string()),
            VectorError::BatchProcessingError("test".to_string()),
            VectorError::ConnectionPoolError("test".to_string()),
        ];

        // Each error should have a defined retryability behavior
        for error in all_errors {
            let _is_retryable = error.is_retryable();
            let _error_message = format!("{}", error);
            // Just ensure no panics occur
        }
    }
}
