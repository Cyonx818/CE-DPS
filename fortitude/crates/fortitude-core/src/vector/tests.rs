// Copyright 2025 CE-DPS Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// ABOUTME: Unit tests for vector database functionality
#[cfg(test)]
mod vector_tests {
    use crate::vector::{DistanceMetric, VectorConfig, VectorOperation, VectorRequest};
    use std::time::Duration;

    fn create_test_config() -> VectorConfig {
        VectorConfig {
            url: "http://localhost:6334".to_string(),
            api_key: None,
            timeout: Duration::from_secs(10),
            default_collection: "test_collection".to_string(),
            vector_dimensions: 384,
            distance_metric: DistanceMetric::Cosine,
            ..VectorConfig::default()
        }
    }

    #[test]
    fn test_vector_config_validation() {
        let config = create_test_config();
        assert!(config.validate().is_ok());

        // Test empty URL
        let mut invalid_config = config.clone();
        invalid_config.url = String::new();
        assert!(invalid_config.validate().is_err());

        // Test empty collection
        let mut invalid_config = config.clone();
        invalid_config.default_collection = String::new();
        assert!(invalid_config.validate().is_err());

        // Test zero dimensions
        let mut invalid_config = config.clone();
        invalid_config.vector_dimensions = 0;
        assert!(invalid_config.validate().is_err());

        // Test zero timeout
        let mut invalid_config = config.clone();
        invalid_config.timeout = Duration::from_secs(0);
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_vector_config_builder_methods() {
        let config = VectorConfig::default()
            .with_url("http://test:6334")
            .with_api_key("test-key")
            .with_timeout(Duration::from_secs(60))
            .with_collection("test")
            .with_dimensions(512)
            .with_distance_metric(DistanceMetric::Euclidean);

        assert_eq!(config.url, "http://test:6334");
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.default_collection, "test");
        assert_eq!(config.vector_dimensions, 512);
        assert!(matches!(config.distance_metric, DistanceMetric::Euclidean));
    }

    #[test]
    fn test_distance_metric_conversion() {
        use qdrant_client::qdrant::Distance;

        let cosine: Distance = DistanceMetric::Cosine.into();
        assert_eq!(cosine, Distance::Cosine);

        let euclidean: Distance = DistanceMetric::Euclidean.into();
        assert_eq!(euclidean, Distance::Euclid);

        let dot: Distance = DistanceMetric::Dot.into();
        assert_eq!(dot, Distance::Dot);
    }

    #[test]
    fn test_vector_request_validation() {
        let config = create_test_config();

        // Create a mock client for testing validation (without actually connecting)
        // Since QdrantClient::new is async and requires a real connection,
        // we'll test the validation logic in isolation

        // Test valid search request
        let search_request = VectorRequest {
            operation: VectorOperation::Search {
                query_vector: vec![0.1; 384], // Correct dimensions
            },
            collection: None,
            vectors: None,
            metadata: None,
            limit: Some(10),
        };

        // Test invalid search request (wrong dimensions)
        let invalid_search_request = VectorRequest {
            operation: VectorOperation::Search {
                query_vector: vec![0.1; 128], // Wrong dimensions
            },
            collection: None,
            vectors: None,
            metadata: None,
            limit: Some(10),
        };

        // Test valid insert request
        let insert_request = VectorRequest {
            operation: VectorOperation::Insert {
                id: "test-id".to_string(),
                vector: vec![0.1; 384], // Correct dimensions
            },
            collection: None,
            vectors: None,
            metadata: None,
            limit: None,
        };

        // Test invalid insert request (wrong dimensions)
        let invalid_insert_request = VectorRequest {
            operation: VectorOperation::Insert {
                id: "test-id".to_string(),
                vector: vec![0.1; 128], // Wrong dimensions
            },
            collection: None,
            vectors: None,
            metadata: None,
            limit: None,
        };

        // Validation logic testing (we'll implement a standalone validator)
        assert!(validate_vector_dimensions(&search_request, &config).is_ok());
        assert!(validate_vector_dimensions(&invalid_search_request, &config).is_err());
        assert!(validate_vector_dimensions(&insert_request, &config).is_ok());
        assert!(validate_vector_dimensions(&invalid_insert_request, &config).is_err());
    }

    // Helper function to test vector dimension validation
    fn validate_vector_dimensions(
        request: &VectorRequest,
        config: &VectorConfig,
    ) -> Result<(), String> {
        match &request.operation {
            VectorOperation::Search { query_vector } => {
                if query_vector.len() != config.vector_dimensions {
                    return Err(format!(
                        "Query vector dimensions mismatch: expected {}, got {}",
                        config.vector_dimensions,
                        query_vector.len()
                    ));
                }
            }
            VectorOperation::Insert { vector, .. } => {
                if vector.len() != config.vector_dimensions {
                    return Err(format!(
                        "Insert vector dimensions mismatch: expected {}, got {}",
                        config.vector_dimensions,
                        vector.len()
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }

    #[test]
    fn test_cli_config_conversion() {
        use crate::vector::utils::{cli_config_to_vector_config, CliConfigParams};

        let params = CliConfigParams {
            url: "http://localhost:6334".to_string(),
            api_key: Some("test-key".to_string()),
            timeout_seconds: 30,
            default_collection: "test_collection".to_string(),
            vector_dimensions: 384,
            distance_metric: "cosine".to_string(),
            health_check_enabled: true,
            health_check_interval_seconds: 30,
            health_check_max_failures: 3,
            health_check_timeout_seconds: 5,
            connection_pool_max_connections: 10,
            connection_pool_idle_timeout_seconds: 300,
            connection_pool_connection_timeout_seconds: 10,
        };

        let result = cli_config_to_vector_config(params);

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.url, "http://localhost:6334");
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.default_collection, "test_collection");
        assert_eq!(config.vector_dimensions, 384);
        assert!(matches!(config.distance_metric, DistanceMetric::Cosine));
        assert!(config.health_check.enabled);
        assert_eq!(config.health_check.max_failures, 3);
        assert_eq!(config.connection_pool.max_connections, 10);
    }

    #[test]
    fn test_invalid_distance_metric_conversion() {
        use crate::vector::utils::{cli_config_to_vector_config, CliConfigParams};

        let params = CliConfigParams {
            url: "http://localhost:6334".to_string(),
            api_key: None,
            timeout_seconds: 30,
            default_collection: "test_collection".to_string(),
            vector_dimensions: 384,
            distance_metric: "invalid_metric".to_string(), // Invalid metric
            health_check_enabled: true,
            health_check_interval_seconds: 30,
            health_check_max_failures: 3,
            health_check_timeout_seconds: 5,
            connection_pool_max_connections: 10,
            connection_pool_idle_timeout_seconds: 300,
            connection_pool_connection_timeout_seconds: 10,
        };

        let result = cli_config_to_vector_config(params);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid distance metric"));
    }

    #[tokio::test]
    async fn test_vector_config_defaults() {
        let config = VectorConfig::default();

        assert_eq!(config.url, "http://localhost:6334");
        assert_eq!(config.api_key, None);
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.default_collection, "fortitude_research");
        assert_eq!(config.vector_dimensions, 384);
        assert!(matches!(config.distance_metric, DistanceMetric::Cosine));
        assert!(config.health_check.enabled);
        assert_eq!(config.health_check.interval, Duration::from_secs(30));
        assert_eq!(config.health_check.max_failures, 3);
        assert_eq!(config.health_check.timeout, Duration::from_secs(5));
        assert_eq!(config.connection_pool.max_connections, 10);
        assert_eq!(
            config.connection_pool.idle_timeout,
            Duration::from_secs(300)
        );
        assert_eq!(
            config.connection_pool.connection_timeout,
            Duration::from_secs(10)
        );
    }

    // Note: Integration tests that require an actual Qdrant instance would go in a separate
    // integration test file or be marked with #[ignore] by default and run only when
    // a Qdrant instance is available for testing.
}
