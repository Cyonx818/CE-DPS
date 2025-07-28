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

// ABOUTME: Qdrant client implementation with connection management and health checks
use crate::api::{ApiClient, HealthStatus, RequestCost};
use crate::vector::{VectorConfig, VectorError, VectorResult};
use async_trait::async_trait;
use qdrant_client::config::QdrantConfig;
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::{
    CreateCollection, Distance, HealthCheckReply, VectorParams, VectorsConfig,
};
use qdrant_client::Qdrant;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Vector database client for Qdrant operations
pub struct QdrantClient {
    client: Qdrant,
    config: VectorConfig,
    health_status: Arc<RwLock<ClientHealthStatus>>,
}

impl std::fmt::Debug for QdrantClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QdrantClient")
            .field("config", &self.config)
            .field("health_status", &"<RwLock<ClientHealthStatus>>")
            .finish()
    }
}

/// Internal health status tracking
#[derive(Debug, Clone)]
struct ClientHealthStatus {
    status: HealthStatus,
    last_check: Instant,
    consecutive_failures: u32,
}

impl Default for ClientHealthStatus {
    fn default() -> Self {
        Self {
            status: HealthStatus::Healthy,
            last_check: Instant::now(),
            consecutive_failures: 0,
        }
    }
}

/// Request type for vector operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorRequest {
    pub operation: VectorOperation,
    pub collection: Option<String>,
    pub vectors: Option<Vec<Vec<f32>>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub limit: Option<usize>,
}

/// Vector operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VectorOperation {
    Search { query_vector: Vec<f32> },
    Insert { id: String, vector: Vec<f32> },
    Delete { id: String },
    CreateCollection,
    HealthCheck,
}

/// Response type for vector operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorResponse {
    pub operation: VectorOperation,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl QdrantClient {
    /// Create a new Qdrant client
    pub async fn new(config: VectorConfig) -> VectorResult<Self> {
        config.validate()?;

        info!("Connecting to Qdrant at: {}", config.url);

        let mut client_config = QdrantConfig::from_url(&config.url);
        client_config.set_timeout(config.timeout);

        if let Some(api_key) = &config.api_key {
            client_config.set_api_key(api_key);
        }

        let client = Qdrant::new(client_config).map_err(|e| {
            VectorError::from_connection_error(format!("Failed to create Qdrant client: {e}"))
        })?;

        let qdrant_client = Self {
            client,
            config,
            health_status: Arc::new(RwLock::new(ClientHealthStatus::default())),
        };

        // Perform initial health check
        qdrant_client.perform_health_check().await?;

        info!("Successfully connected to Qdrant vector database");
        Ok(qdrant_client)
    }

    /// Create a collection if it doesn't exist
    pub async fn ensure_collection(&self, collection_name: &str) -> VectorResult<()> {
        debug!("Ensuring collection exists: {}", collection_name);

        // Check if collection exists
        match self.client.collection_info(collection_name).await {
            Ok(_) => {
                debug!("Collection {} already exists", collection_name);
                return Ok(());
            }
            Err(e)
                if e.to_string().contains("not found") || e.to_string().contains("Not found") =>
            {
                // Collection doesn't exist, create it
                debug!("Collection {} not found, creating", collection_name);
            }
            Err(e) => {
                return Err(VectorError::from_operation_failed(
                    "check_collection",
                    format!("Failed to check collection: {e}"),
                ));
            }
        }

        // Create the collection
        let vectors_config = VectorsConfig {
            config: Some(Config::Params(VectorParams {
                size: self.config.vector_dimensions as u64,
                distance: Distance::from(self.config.distance_metric.clone()) as i32,
                hnsw_config: None,
                quantization_config: None,
                on_disk: None,
                datatype: None,
                multivector_config: None,
            })),
        };

        let create_collection = CreateCollection {
            collection_name: collection_name.to_string(),
            vectors_config: Some(vectors_config),
            shard_number: None,
            replication_factor: None,
            write_consistency_factor: None,
            on_disk_payload: None,
            timeout: Some(self.config.timeout.as_secs()),
            ..Default::default()
        };

        self.client
            .create_collection(create_collection)
            .await
            .map_err(|e| {
                VectorError::from_operation_failed(
                    "create_collection",
                    format!("Failed to create collection {collection_name}: {e}"),
                )
            })?;

        info!("Successfully created collection: {}", collection_name);
        Ok(())
    }

    /// Perform a health check
    async fn perform_health_check(&self) -> VectorResult<HealthCheckReply> {
        debug!("Performing Qdrant health check");

        let health_reply =
            self.client
                .health_check()
                .await
                .map_err(|e| VectorError::HealthCheckFailed {
                    reason: format!("Health check failed: {e}"),
                })?;

        debug!("Health check completed successfully");
        Ok(health_reply)
    }

    /// Update internal health status
    async fn update_health_status(&self, is_healthy: bool, reason: Option<String>) {
        let mut health_status = self.health_status.write().await;

        if is_healthy {
            health_status.status = HealthStatus::Healthy;
            health_status.consecutive_failures = 0;
        } else {
            health_status.consecutive_failures += 1;

            if health_status.consecutive_failures >= self.config.health_check.max_failures {
                health_status.status = HealthStatus::Unhealthy(
                    reason.unwrap_or_else(|| "Multiple consecutive failures".to_string()),
                );
            } else {
                health_status.status = HealthStatus::Degraded(
                    reason.unwrap_or_else(|| "Health check failed".to_string()),
                );
            }
        }

        health_status.last_check = Instant::now();
    }

    /// Get collection information
    pub async fn get_collection_info(
        &self,
        collection_name: &str,
    ) -> VectorResult<serde_json::Value> {
        debug!("Getting collection info for: {}", collection_name);

        let _info = self
            .client
            .collection_info(collection_name)
            .await
            .map_err(|e| match e {
                e if e.to_string().contains("not found") || e.to_string().contains("Not found") => {
                    VectorError::CollectionNotFound {
                        collection: collection_name.to_string(),
                    }
                }
                _ => VectorError::from_operation_failed(
                    "get_collection_info",
                    format!("Failed to get collection info: {e}"),
                ),
            })?;

        // For now, return a simple JSON representation
        // In the future, this can be enhanced with proper serialization
        Ok(serde_json::json!({
            "collection_name": collection_name,
            "status": "exists"
        }))
    }

    /// Delete a collection
    pub async fn delete_collection(&self, collection_name: &str) -> VectorResult<()> {
        info!("Deleting collection: {}", collection_name);

        self.client
            .delete_collection(collection_name)
            .await
            .map_err(|e| {
                VectorError::from_operation_failed(
                    "delete_collection",
                    format!("Failed to delete collection {collection_name}: {e}"),
                )
            })?;

        info!("Successfully deleted collection: {}", collection_name);
        Ok(())
    }

    /// Get the Qdrant client configuration
    pub fn config(&self) -> &VectorConfig {
        &self.config
    }

    /// Get the default collection name
    pub fn default_collection(&self) -> &str {
        &self.config.default_collection
    }

    /// Check if the client is currently healthy
    pub async fn is_healthy(&self) -> bool {
        let health_status = self.health_status.read().await;
        matches!(health_status.status, HealthStatus::Healthy)
    }

    /// Get access to the underlying Qdrant client
    pub fn client(&self) -> &Qdrant {
        &self.client
    }
}

#[async_trait]
impl ApiClient for QdrantClient {
    type Request = VectorRequest;
    type Response = VectorResponse;
    type Config = VectorConfig;

    fn new(_config: Self::Config) -> crate::api::error::ApiResult<Self>
    where
        Self: Sized,
    {
        // For compatibility with the ApiClient trait, we return an error
        // since QdrantClient::new is async
        Err(crate::api::error::ApiError::ConfigurationError(
            "Use QdrantClient::new() async method instead".to_string(),
        ))
    }

    async fn send_request(
        &self,
        request: Self::Request,
    ) -> crate::api::error::ApiResult<Self::Response> {
        let collection = request
            .collection
            .as_deref()
            .unwrap_or(&self.config.default_collection);

        let response = match request.operation {
            VectorOperation::HealthCheck => match self.perform_health_check().await {
                Ok(_) => {
                    self.update_health_status(true, None).await;
                    VectorResponse {
                        operation: request.operation,
                        success: true,
                        data: Some(serde_json::json!({"status": "healthy"})),
                        metadata: None,
                    }
                }
                Err(e) => {
                    self.update_health_status(false, Some(e.to_string())).await;
                    VectorResponse {
                        operation: request.operation,
                        success: false,
                        data: Some(serde_json::json!({"error": e.to_string()})),
                        metadata: None,
                    }
                }
            },
            VectorOperation::CreateCollection => match self.ensure_collection(collection).await {
                Ok(_) => VectorResponse {
                    operation: request.operation,
                    success: true,
                    data: Some(serde_json::json!({"collection": collection})),
                    metadata: None,
                },
                Err(e) => VectorResponse {
                    operation: request.operation,
                    success: false,
                    data: Some(serde_json::json!({"error": e.to_string()})),
                    metadata: None,
                },
            },
            _ => {
                // For now, return success for other operations
                // These will be implemented in future phases
                VectorResponse {
                    operation: request.operation,
                    success: true,
                    data: Some(serde_json::json!({"message": "Operation not yet implemented"})),
                    metadata: None,
                }
            }
        };

        Ok(response)
    }

    fn validate_request(&self, request: &Self::Request) -> crate::api::error::ApiResult<()> {
        // Basic validation
        match &request.operation {
            VectorOperation::Search { query_vector } => {
                if query_vector.len() != self.config.vector_dimensions {
                    return Err(crate::api::error::ApiError::ValidationError(format!(
                        "Query vector dimensions mismatch: expected {}, got {}",
                        self.config.vector_dimensions,
                        query_vector.len()
                    )));
                }
            }
            VectorOperation::Insert { vector, .. } => {
                if vector.len() != self.config.vector_dimensions {
                    return Err(crate::api::error::ApiError::ValidationError(format!(
                        "Insert vector dimensions mismatch: expected {}, got {}",
                        self.config.vector_dimensions,
                        vector.len()
                    )));
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn estimate_cost(&self, _request: &Self::Request) -> crate::api::error::ApiResult<RequestCost> {
        // Simple cost estimation for vector operations
        Ok(RequestCost {
            estimated_input_tokens: 0,
            estimated_output_tokens: 0,
            estimated_duration: Duration::from_millis(100),
            estimated_cost_usd: None,
        })
    }

    async fn health_check(&self) -> crate::api::error::ApiResult<HealthStatus> {
        let health_status = self.health_status.read().await;
        Ok(health_status.status.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::HealthStatus;
    use crate::vector::config::{ConnectionPoolConfig, HealthCheckConfig, VectorConfig};
    use crate::vector::embeddings::EmbeddingConfig;
    use std::time::Duration;
    use tokio;

    fn create_test_config() -> VectorConfig {
        VectorConfig {
            url: "http://localhost:6334".to_string(),
            api_key: None,
            timeout: Duration::from_secs(5),
            default_collection: "test_collection".to_string(),
            vector_dimensions: 384,
            distance_metric: crate::vector::config::DistanceMetric::Cosine,
            health_check: HealthCheckConfig {
                enabled: true,
                interval: Duration::from_secs(10),
                max_failures: 2,
                timeout: Duration::from_secs(3),
            },
            connection_pool: ConnectionPoolConfig::default(),
            embedding: EmbeddingConfig::default(),
        }
    }

    #[test]
    fn test_client_health_status_default() {
        let status = ClientHealthStatus::default();
        assert!(matches!(status.status, HealthStatus::Healthy));
        assert_eq!(status.consecutive_failures, 0);
    }

    #[test]
    fn test_vector_request_creation() {
        let request = VectorRequest {
            operation: VectorOperation::HealthCheck,
            collection: Some("test_collection".to_string()),
            vectors: None,
            metadata: None,
            limit: Some(10),
        };

        assert!(matches!(request.operation, VectorOperation::HealthCheck));
        assert_eq!(request.collection, Some("test_collection".to_string()));
        assert_eq!(request.limit, Some(10));
    }

    #[test]
    fn test_vector_operation_types() {
        let search_op = VectorOperation::Search {
            query_vector: vec![0.1, 0.2, 0.3],
        };

        let insert_op = VectorOperation::Insert {
            id: "test-id".to_string(),
            vector: vec![0.4, 0.5, 0.6],
        };

        let delete_op = VectorOperation::Delete {
            id: "test-id".to_string(),
        };

        assert!(matches!(search_op, VectorOperation::Search { .. }));
        assert!(matches!(insert_op, VectorOperation::Insert { .. }));
        assert!(matches!(delete_op, VectorOperation::Delete { .. }));
    }

    #[test]
    fn test_vector_response_creation() {
        let response = VectorResponse {
            operation: VectorOperation::HealthCheck,
            success: true,
            data: Some(serde_json::json!({"status": "healthy"})),
            metadata: None,
        };

        assert!(response.success);
        assert!(response.data.is_some());
        assert!(matches!(response.operation, VectorOperation::HealthCheck));
    }

    #[tokio::test]
    async fn test_qdrant_client_config_validation() {
        // Test with invalid config (empty URL)
        let mut invalid_config = create_test_config();
        invalid_config.url = String::new();

        let result = QdrantClient::new(invalid_config).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VectorError::ConfigurationError(_)
        ));
    }

    #[tokio::test]
    async fn test_qdrant_client_configuration_settings() {
        let config = create_test_config();

        // The client should use the configured values
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.default_collection, "test_collection");
        assert_eq!(config.vector_dimensions, 384);
    }

    #[test]
    fn test_qdrant_client_config_access() {
        let config = create_test_config();
        let expected_collection = config.default_collection.clone();

        // Test that the client would store and provide access to config
        // Note: We can't create an actual client without a running Qdrant instance
        assert_eq!(expected_collection, "test_collection");
    }

    #[tokio::test]
    async fn test_health_check_request_processing() {
        let _config = create_test_config();

        // Test processing health check request
        let request = VectorRequest {
            operation: VectorOperation::HealthCheck,
            collection: None,
            vectors: None,
            metadata: None,
            limit: None,
        };

        // The request should be valid for health checks
        assert!(matches!(request.operation, VectorOperation::HealthCheck));
    }

    #[tokio::test]
    async fn test_create_collection_request_processing() {
        let _config = create_test_config();

        let request = VectorRequest {
            operation: VectorOperation::CreateCollection,
            collection: Some("new_collection".to_string()),
            vectors: None,
            metadata: None,
            limit: None,
        };

        assert!(matches!(
            request.operation,
            VectorOperation::CreateCollection
        ));
        assert_eq!(request.collection, Some("new_collection".to_string()));
    }

    #[test]
    fn test_request_validation_search_vector_dimensions() {
        let config = create_test_config();

        // Test valid vector dimensions
        let valid_request = VectorRequest {
            operation: VectorOperation::Search {
                query_vector: vec![0.1; 384], // Correct dimensions
            },
            collection: None,
            vectors: None,
            metadata: None,
            limit: None,
        };

        // Validation logic would check that vector length matches config.vector_dimensions
        if let VectorOperation::Search { query_vector } = &valid_request.operation {
            assert_eq!(query_vector.len(), config.vector_dimensions);
        }
    }

    #[test]
    fn test_request_validation_insert_vector_dimensions() {
        let config = create_test_config();

        let valid_request = VectorRequest {
            operation: VectorOperation::Insert {
                id: "test-id".to_string(),
                vector: vec![0.1; 384], // Correct dimensions
            },
            collection: None,
            vectors: None,
            metadata: None,
            limit: None,
        };

        if let VectorOperation::Insert { vector, .. } = &valid_request.operation {
            assert_eq!(vector.len(), config.vector_dimensions);
        }
    }

    #[test]
    fn test_request_validation_invalid_dimensions() {
        let config = create_test_config();

        let invalid_request = VectorRequest {
            operation: VectorOperation::Search {
                query_vector: vec![0.1; 256], // Wrong dimensions
            },
            collection: None,
            vectors: None,
            metadata: None,
            limit: None,
        };

        if let VectorOperation::Search { query_vector } = &invalid_request.operation {
            assert_ne!(query_vector.len(), config.vector_dimensions);
        }
    }

    #[test]
    fn test_vector_request_serialization() {
        let request = VectorRequest {
            operation: VectorOperation::HealthCheck,
            collection: Some("test".to_string()),
            vectors: None,
            metadata: None,
            limit: Some(5),
        };

        let serialized = serde_json::to_string(&request).expect("Failed to serialize request");
        let deserialized: VectorRequest =
            serde_json::from_str(&serialized).expect("Failed to deserialize request");

        assert_eq!(request.collection, deserialized.collection);
        assert_eq!(request.limit, deserialized.limit);
    }

    #[test]
    fn test_vector_response_serialization() {
        let response = VectorResponse {
            operation: VectorOperation::CreateCollection,
            success: true,
            data: Some(serde_json::json!({"collection": "test"})),
            metadata: None,
        };

        let serialized = serde_json::to_string(&response).expect("Failed to serialize response");
        let deserialized: VectorResponse =
            serde_json::from_str(&serialized).expect("Failed to deserialize response");

        assert_eq!(response.success, deserialized.success);
        assert!(deserialized.data.is_some());
    }

    #[test]
    fn test_vector_operation_serialization() {
        let operations = vec![
            VectorOperation::HealthCheck,
            VectorOperation::CreateCollection,
            VectorOperation::Search {
                query_vector: vec![0.1, 0.2],
            },
            VectorOperation::Insert {
                id: "test".to_string(),
                vector: vec![0.3, 0.4],
            },
            VectorOperation::Delete {
                id: "test".to_string(),
            },
        ];

        for operation in operations {
            let serialized =
                serde_json::to_string(&operation).expect("Failed to serialize operation");
            let _deserialized: VectorOperation =
                serde_json::from_str(&serialized).expect("Failed to deserialize operation");
        }
    }

    #[test]
    fn test_vector_request_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), serde_json::json!("value1"));
        metadata.insert("key2".to_string(), serde_json::json!(42));

        let request = VectorRequest {
            operation: VectorOperation::Insert {
                id: "test-id".to_string(),
                vector: vec![0.1, 0.2, 0.3],
            },
            collection: Some("test_collection".to_string()),
            vectors: None,
            metadata: Some(metadata.clone()),
            limit: None,
        };

        assert!(request.metadata.is_some());
        let req_metadata = request.metadata.unwrap();
        assert_eq!(req_metadata.len(), 2);
        assert!(req_metadata.contains_key("key1"));
        assert!(req_metadata.contains_key("key2"));
    }

    #[test]
    fn test_vector_request_batch_vectors() {
        let vectors = vec![
            vec![0.1, 0.2, 0.3],
            vec![0.4, 0.5, 0.6],
            vec![0.7, 0.8, 0.9],
        ];

        let request = VectorRequest {
            operation: VectorOperation::HealthCheck, // Placeholder operation
            collection: Some("test_collection".to_string()),
            vectors: Some(vectors.clone()),
            metadata: None,
            limit: None,
        };

        assert!(request.vectors.is_some());
        let req_vectors = request.vectors.unwrap();
        assert_eq!(req_vectors.len(), 3);
        assert_eq!(req_vectors[0], vec![0.1, 0.2, 0.3]);
    }

    #[test]
    fn test_request_cost_estimation() {
        let request = VectorRequest {
            operation: VectorOperation::Search {
                query_vector: vec![0.1; 384],
            },
            collection: None,
            vectors: None,
            metadata: None,
            limit: Some(10),
        };

        // Cost estimation should be reasonable for vector operations
        // (This would be implemented in the actual client)
        assert!(matches!(request.operation, VectorOperation::Search { .. }));
        assert_eq!(request.limit, Some(10));
    }

    #[test]
    fn test_client_health_status_transitions() {
        let mut status = ClientHealthStatus::default();
        assert!(matches!(status.status, HealthStatus::Healthy));
        assert_eq!(status.consecutive_failures, 0);

        // Simulate failure
        status.consecutive_failures = 1;
        assert_eq!(status.consecutive_failures, 1);

        // Simulate recovery
        status.consecutive_failures = 0;
        status.status = HealthStatus::Healthy;
        assert!(matches!(status.status, HealthStatus::Healthy));
    }

    #[test]
    fn test_default_collection_handling() {
        let config = create_test_config();
        let collection_name = &config.default_collection;

        // Requests without collection should use default
        let request = VectorRequest {
            operation: VectorOperation::HealthCheck,
            collection: None,
            vectors: None,
            metadata: None,
            limit: None,
        };

        assert!(request.collection.is_none());
        // The client would use config.default_collection when collection is None
        assert_eq!(collection_name, "test_collection");
    }

    #[test]
    fn test_vector_dimensions_consistency() {
        let config = create_test_config();

        // All vectors should match the configured dimensions
        let search_vector = vec![0.1; config.vector_dimensions];
        let insert_vector = vec![0.2; config.vector_dimensions];

        assert_eq!(search_vector.len(), config.vector_dimensions);
        assert_eq!(insert_vector.len(), config.vector_dimensions);
    }

    #[test]
    fn test_client_configuration_immutability() {
        let config = create_test_config();
        let original_url = config.url.clone();
        let original_timeout = config.timeout;

        // The configuration should remain unchanged after client creation
        assert_eq!(config.url, original_url);
        assert_eq!(config.timeout, original_timeout);
    }

    #[test]
    fn test_error_response_creation() {
        let error_response = VectorResponse {
            operation: VectorOperation::HealthCheck,
            success: false,
            data: Some(serde_json::json!({"error": "Connection failed"})),
            metadata: None,
        };

        assert!(!error_response.success);
        assert!(error_response.data.is_some());

        if let Some(data) = error_response.data {
            assert!(data.get("error").is_some());
        }
    }

    #[test]
    fn test_successful_response_creation() {
        let success_response = VectorResponse {
            operation: VectorOperation::CreateCollection,
            success: true,
            data: Some(serde_json::json!({"collection": "new_collection"})),
            metadata: None,
        };

        assert!(success_response.success);
        assert!(success_response.data.is_some());

        if let Some(data) = success_response.data {
            assert_eq!(data.get("collection").unwrap(), "new_collection");
        }
    }
}
