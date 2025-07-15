// ABOUTME: Configuration for vector database operations
use crate::vector::embeddings::EmbeddingConfig;
use crate::vector::error::{VectorError, VectorResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::warn;

/// Vector database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorConfig {
    /// Qdrant server URL
    pub url: String,
    /// API key for authentication (optional)
    pub api_key: Option<String>,
    /// Connection timeout
    pub timeout: Duration,
    /// Default collection name for research documents
    pub default_collection: String,
    /// Vector dimensions for embeddings
    pub vector_dimensions: usize,
    /// Distance metric for similarity search
    pub distance_metric: DistanceMetric,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Connection pool configuration
    pub connection_pool: ConnectionPoolConfig,
    /// Embedding generation configuration
    pub embedding: EmbeddingConfig,
}

/// Distance metrics supported by Qdrant
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum DistanceMetric {
    #[default]
    Cosine,
    Euclidean,
    Dot,
}

impl From<DistanceMetric> for qdrant_client::qdrant::Distance {
    fn from(metric: DistanceMetric) -> Self {
        match metric {
            DistanceMetric::Cosine => qdrant_client::qdrant::Distance::Cosine,
            DistanceMetric::Euclidean => qdrant_client::qdrant::Distance::Euclid,
            DistanceMetric::Dot => qdrant_client::qdrant::Distance::Dot,
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable periodic health checks
    pub enabled: bool,
    /// Interval between health checks
    pub interval: Duration,
    /// Maximum consecutive failures before marking unhealthy
    pub max_failures: u32,
    /// Health check timeout
    pub timeout: Duration,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(30),
            max_failures: 3,
            timeout: Duration::from_secs(5),
        }
    }
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections in the pool
    pub max_connections: usize,
    /// Connection idle timeout
    pub idle_timeout: Duration,
    /// Maximum time to wait for a connection
    pub connection_timeout: Duration,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            idle_timeout: Duration::from_secs(300),
            connection_timeout: Duration::from_secs(10),
        }
    }
}

impl Default for VectorConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6334".to_string(),
            api_key: None,
            timeout: Duration::from_secs(30),
            default_collection: "fortitude_research".to_string(),
            vector_dimensions: 384, // Common embedding dimension
            distance_metric: DistanceMetric::default(),
            health_check: HealthCheckConfig::default(),
            connection_pool: ConnectionPoolConfig::default(),
            embedding: EmbeddingConfig::default(),
        }
    }
}

impl VectorConfig {
    /// Validate the configuration
    pub fn validate(&self) -> VectorResult<()> {
        if self.url.is_empty() {
            return Err(VectorError::ConfigurationError(
                "Vector database URL cannot be empty".to_string(),
            ));
        }

        if self.default_collection.is_empty() {
            return Err(VectorError::ConfigurationError(
                "Default collection name cannot be empty".to_string(),
            ));
        }

        if self.vector_dimensions == 0 {
            return Err(VectorError::ConfigurationError(
                "Vector dimensions must be greater than zero".to_string(),
            ));
        }

        if self.timeout.is_zero() {
            return Err(VectorError::ConfigurationError(
                "Timeout must be greater than zero".to_string(),
            ));
        }

        // Validate that vector dimensions match embedding model output
        if self.vector_dimensions != self.embedding.max_sequence_length.min(2048) {
            warn!(
                "Vector dimensions ({}) may not match embedding model output",
                self.vector_dimensions
            );
        }

        Ok(())
    }

    /// Create configuration with custom URL
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    /// Create configuration with API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Create configuration with custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Create configuration with custom collection
    pub fn with_collection(mut self, collection: impl Into<String>) -> Self {
        self.default_collection = collection.into();
        self
    }

    /// Create configuration with custom vector dimensions
    pub fn with_dimensions(mut self, dimensions: usize) -> Self {
        self.vector_dimensions = dimensions;
        self
    }

    /// Create configuration with custom distance metric
    pub fn with_distance_metric(mut self, metric: DistanceMetric) -> Self {
        self.distance_metric = metric;
        self
    }

    /// Create configuration with custom embedding configuration
    pub fn with_embedding_config(mut self, embedding_config: EmbeddingConfig) -> Self {
        self.embedding = embedding_config;
        self
    }

    /// Update vector dimensions to match embedding model
    pub fn sync_dimensions_with_embedding(&mut self) {
        // For sentence-transformers models, common dimensions are 384, 512, 768
        match self.embedding.model_name.as_str() {
            "sentence-transformers/all-MiniLM-L6-v2" => self.vector_dimensions = 384,
            "sentence-transformers/all-MiniLM-L12-v2" => self.vector_dimensions = 384,
            "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2" => {
                self.vector_dimensions = 384
            }
            "sentence-transformers/all-mpnet-base-v2" => self.vector_dimensions = 768,
            "sentence-transformers/multi-qa-mpnet-base-dot-v1" => self.vector_dimensions = 768,
            _ => {
                // Keep current dimensions for unknown models
                warn!(
                    "Unknown embedding model: {}. Using configured dimensions: {}",
                    self.embedding.model_name, self.vector_dimensions
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_vector_config_default_values() {
        let config = VectorConfig::default();

        assert_eq!(config.url, "http://localhost:6334");
        assert!(config.api_key.is_none());
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.default_collection, "fortitude_research");
        assert_eq!(config.vector_dimensions, 384);
        assert!(matches!(config.distance_metric, DistanceMetric::Cosine));
        assert!(config.health_check.enabled);
        assert_eq!(config.connection_pool.max_connections, 10);
    }

    #[test]
    fn test_vector_config_validation_success() {
        let config = VectorConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_vector_config_validation_empty_url() {
        let mut config = VectorConfig::default();
        config.url = String::new();

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VectorError::ConfigurationError(_)
        ));
    }

    #[test]
    fn test_vector_config_validation_empty_collection() {
        let mut config = VectorConfig::default();
        config.default_collection = String::new();

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VectorError::ConfigurationError(_)
        ));
    }

    #[test]
    fn test_vector_config_validation_zero_dimensions() {
        let mut config = VectorConfig::default();
        config.vector_dimensions = 0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VectorError::ConfigurationError(_)
        ));
    }

    #[test]
    fn test_vector_config_validation_zero_timeout() {
        let mut config = VectorConfig::default();
        config.timeout = Duration::from_secs(0);

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VectorError::ConfigurationError(_)
        ));
    }

    #[test]
    fn test_vector_config_with_url_builder() {
        let config = VectorConfig::default().with_url("http://custom:6334");

        assert_eq!(config.url, "http://custom:6334");
    }

    #[test]
    fn test_vector_config_with_api_key_builder() {
        let config = VectorConfig::default().with_api_key("test-api-key");

        assert_eq!(config.api_key, Some("test-api-key".to_string()));
    }

    #[test]
    fn test_vector_config_with_timeout_builder() {
        let config = VectorConfig::default().with_timeout(Duration::from_secs(60));

        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_vector_config_with_collection_builder() {
        let config = VectorConfig::default().with_collection("custom_collection");

        assert_eq!(config.default_collection, "custom_collection");
    }

    #[test]
    fn test_vector_config_with_dimensions_builder() {
        let config = VectorConfig::default().with_dimensions(768);

        assert_eq!(config.vector_dimensions, 768);
    }

    #[test]
    fn test_vector_config_with_distance_metric_builder() {
        let config = VectorConfig::default().with_distance_metric(DistanceMetric::Euclidean);

        assert!(matches!(config.distance_metric, DistanceMetric::Euclidean));
    }

    #[test]
    fn test_vector_config_with_embedding_config_builder() {
        let embedding_config = EmbeddingConfig {
            model_name: "custom-model".to_string(),
            ..Default::default()
        };

        let config = VectorConfig::default().with_embedding_config(embedding_config.clone());

        assert_eq!(config.embedding.model_name, "custom-model");
    }

    #[test]
    fn test_vector_config_sync_dimensions_with_embedding_known_model() {
        let mut config = VectorConfig::default();
        config.embedding.model_name = "sentence-transformers/all-mpnet-base-v2".to_string();

        config.sync_dimensions_with_embedding();

        assert_eq!(config.vector_dimensions, 768);
    }

    #[test]
    fn test_vector_config_sync_dimensions_with_embedding_unknown_model() {
        let mut config = VectorConfig::default();
        config.vector_dimensions = 512; // Custom dimension
        config.embedding.model_name = "unknown-model".to_string();

        config.sync_dimensions_with_embedding();

        // Should keep the original dimensions for unknown models
        assert_eq!(config.vector_dimensions, 512);
    }

    #[test]
    fn test_vector_config_chained_builders() {
        let config = VectorConfig::default()
            .with_url("http://production:6334")
            .with_api_key("production-key")
            .with_timeout(Duration::from_secs(120))
            .with_collection("production_docs")
            .with_dimensions(768)
            .with_distance_metric(DistanceMetric::Dot);

        assert_eq!(config.url, "http://production:6334");
        assert_eq!(config.api_key, Some("production-key".to_string()));
        assert_eq!(config.timeout, Duration::from_secs(120));
        assert_eq!(config.default_collection, "production_docs");
        assert_eq!(config.vector_dimensions, 768);
        assert!(matches!(config.distance_metric, DistanceMetric::Dot));
    }

    #[test]
    fn test_distance_metric_default() {
        let metric = DistanceMetric::default();
        assert!(matches!(metric, DistanceMetric::Cosine));
    }

    #[test]
    fn test_distance_metric_conversion_to_qdrant() {
        let cosine: qdrant_client::qdrant::Distance = DistanceMetric::Cosine.into();
        let euclidean: qdrant_client::qdrant::Distance = DistanceMetric::Euclidean.into();
        let dot: qdrant_client::qdrant::Distance = DistanceMetric::Dot.into();

        assert_eq!(
            cosine as i32,
            qdrant_client::qdrant::Distance::Cosine as i32
        );
        assert_eq!(
            euclidean as i32,
            qdrant_client::qdrant::Distance::Euclid as i32
        );
        assert_eq!(dot as i32, qdrant_client::qdrant::Distance::Dot as i32);
    }

    #[test]
    fn test_health_check_config_default() {
        let config = HealthCheckConfig::default();

        assert!(config.enabled);
        assert_eq!(config.interval, Duration::from_secs(30));
        assert_eq!(config.max_failures, 3);
        assert_eq!(config.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_connection_pool_config_default() {
        let config = ConnectionPoolConfig::default();

        assert_eq!(config.max_connections, 10);
        assert_eq!(config.idle_timeout, Duration::from_secs(300));
        assert_eq!(config.connection_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_vector_config_serialization() {
        let config = VectorConfig::default()
            .with_url("http://test:6334")
            .with_api_key("test-key");

        let serialized = serde_json::to_string(&config).expect("Failed to serialize config");
        let deserialized: VectorConfig =
            serde_json::from_str(&serialized).expect("Failed to deserialize config");

        assert_eq!(config.url, deserialized.url);
        assert_eq!(config.api_key, deserialized.api_key);
        assert_eq!(config.vector_dimensions, deserialized.vector_dimensions);
    }

    #[test]
    fn test_vector_config_validation_with_custom_values() {
        let config = VectorConfig {
            url: "https://custom-qdrant.example.com:443".to_string(),
            api_key: Some("custom-api-key".to_string()),
            timeout: Duration::from_secs(45),
            default_collection: "custom_collection".to_string(),
            vector_dimensions: 1024,
            distance_metric: DistanceMetric::Euclidean,
            health_check: HealthCheckConfig {
                enabled: false,
                interval: Duration::from_secs(60),
                max_failures: 5,
                timeout: Duration::from_secs(10),
            },
            connection_pool: ConnectionPoolConfig {
                max_connections: 20,
                idle_timeout: Duration::from_secs(600),
                connection_timeout: Duration::from_secs(15),
            },
            embedding: EmbeddingConfig::default(),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_vector_config_edge_cases() {
        // Test very large dimensions
        let mut config = VectorConfig::default();
        config.vector_dimensions = 10000;
        assert!(config.validate().is_ok());

        // Test very short timeout
        config.timeout = Duration::from_millis(1);
        assert!(config.validate().is_ok());

        // Test very long collection name
        config.default_collection = "a".repeat(1000);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_vector_config_clone() {
        let config = VectorConfig::default()
            .with_url("http://test:6334")
            .with_api_key("test-key");

        let cloned = config.clone();

        assert_eq!(config.url, cloned.url);
        assert_eq!(config.api_key, cloned.api_key);
        assert_eq!(config.vector_dimensions, cloned.vector_dimensions);
    }

    #[test]
    fn test_vector_config_debug_formatting() {
        let config = VectorConfig::default();
        let debug_str = format!("{:?}", config);

        // Ensure debug output contains key fields
        assert!(debug_str.contains("VectorConfig"));
        assert!(debug_str.contains("url"));
        assert!(debug_str.contains("vector_dimensions"));
    }

    #[test]
    fn test_embedding_dimension_validation_warning() {
        let mut config = VectorConfig::default();
        config.vector_dimensions = 512; // Different from embedding model output
        config.embedding.max_sequence_length = 256;

        // Should not error but may warn (captured in logs)
        assert!(config.validate().is_ok());
    }
}
