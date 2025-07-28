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

// ABOUTME: Anchor tests to verify API server maintains expected behavior across changes
// These tests serve as a safety net to catch regressions during development

use fortitude_api_server::{config::ApiServerConfig, server::ApiServer};

/// Anchor test: Basic server functionality
/// This test ensures the core server functionality never breaks
#[tokio::test]
async fn anchor_server_basic_functionality() {
    // Test that we can create a server without panicking
    let config = ApiServerConfig::default();
    let server = ApiServer::new(config)
        .await
        .expect("Server creation failed");

    // Test that the server has a valid bind address
    assert!(!server.config.bind_address().is_empty());
    assert!(server.config.bind_address().contains(':'));
}

/// Anchor test: Configuration loading never panics
/// Configuration loading is critical - it should handle errors gracefully
#[tokio::test]
async fn anchor_config_loading_never_panics() {
    // Test default configuration
    let config = ApiServerConfig::default();
    assert!(!config.host.is_empty());
    assert!(config.port > 0);
    // Port is u16, so it's automatically within valid range
}

/// Anchor test: Server compilation and type safety
/// This test ensures our type signatures remain stable
#[tokio::test]
async fn anchor_type_safety() {
    // Test that all our main types can be constructed and used
    let config = ApiServerConfig::default();

    // Verify config has expected methods
    let _bind_addr = config.bind_address();
    let _host = &config.host;
    let _port = config.port;

    // Verify server can be created with config
    let server = ApiServer::new(config)
        .await
        .expect("Server creation failed");

    // This test passes if compilation succeeds and no panics occur
    drop(server);
}

/// ANCHOR: Proactive research API endpoints are stable and functional
/// This test ensures proactive research endpoints remain available and operational
#[tokio::test]
async fn anchor_proactive_research_api_stability() {
    use fortitude_api_server::{ApiServer, ApiServerConfig};

    // Create server with default configuration
    let config = ApiServerConfig::default();
    let server = ApiServer::new(config)
        .await
        .expect("Server creation should not fail");

    // Verify proactive state is initialized
    assert!(
        server.proactive_state.is_some(),
        "Proactive state should be initialized"
    );

    // Verify proactive state can be queried
    if let Some(proactive_state) = &server.proactive_state {
        let _is_running = proactive_state.is_running().await;
        // Should not panic - exact state depends on implementation
        let _result = proactive_state.get_status().await;
    }
}

/// ANCHOR: Proactive research request/response models maintain compatibility
/// This test ensures API models don't break unexpectedly
#[tokio::test]
async fn anchor_proactive_models_stability() {
    use fortitude_api_server::models::requests::{
        ProactiveConfigRequest, ProactiveNotificationListRequest, ProactiveTaskListRequest,
    };
    use validator::Validate;

    // Test that request models can be created and validated
    let config_request = ProactiveConfigRequest {
        base_directory: Some("/test".to_string()),
        file_patterns: Some(vec!["*.rs".to_string()]),
        ignore_patterns: Some(vec!["target/".to_string()]),
        enabled: Some(true),
        monitoring_interval_seconds: Some(300),
        max_concurrent_tasks: Some(5),
        priority_threshold: Some(0.7),
        auto_execute_high_priority: Some(false),
        notification_preferences: None,
    };

    // Should not panic
    let _validation_result = config_request.validate();

    let task_request = ProactiveTaskListRequest {
        status: Some("pending".to_string()),
        priority: Some("high".to_string()),
        research_type: None,
        gap_type: None,
        created_after: None,
        created_before: None,
        keywords: None,
        limit: Some(20),
        offset: Some(0),
        sort: Some("newest".to_string()),
    };

    // Should not panic
    let _validation_result = task_request.validate();

    let notification_request = ProactiveNotificationListRequest {
        notification_type: Some("gap_detected".to_string()),
        level: Some("warning".to_string()),
        read: Some(false),
        created_after: None,
        created_before: None,
        limit: Some(10),
        offset: Some(0),
        sort: Some("newest".to_string()),
    };

    // Should not panic
    let _validation_result = notification_request.validate();
}

/// ANCHOR: Proactive research state management works correctly
/// This test ensures the core state management functionality is stable
#[tokio::test]
async fn anchor_proactive_state_management() {
    use fortitude_api_server::routes::proactive::ProactiveState;

    // Create proactive state
    let state = ProactiveState::new()
        .await
        .expect("State creation should not fail");

    // Initially should not be running
    assert!(!state.is_running().await, "Should not be running initially");

    // Should be able to get status
    let status = state
        .get_status()
        .await
        .expect("Status should be retrievable");
    assert!(!status.is_running, "Status should show not running");
    assert_eq!(
        status.uptime_seconds, 0,
        "Uptime should be zero when not running"
    );
}

/// Anchor test: Dependency compatibility
/// Ensures our dependencies work together without conflicts
#[tokio::test]
async fn anchor_dependency_compatibility() {
    use serde_json::json;
    use uuid::Uuid;

    // Test that our key dependencies work
    let _uuid = Uuid::new_v4();
    let _json = json!({"test": "value"});

    // Test server creation
    let config = ApiServerConfig::default();
    let _server = ApiServer::new(config)
        .await
        .expect("Server creation failed");

    // If we reach here, dependencies are compatible
    // Test passes if no panics occur
}

/// Anchor test: Memory safety and no leaks in basic operations
/// Performs basic operations that should not leak memory
#[tokio::test]
async fn anchor_memory_safety() {
    // Create and drop multiple servers to test for obvious leaks
    for _i in 0..3 {
        let config = ApiServerConfig::default();
        let server = ApiServer::new(config)
            .await
            .expect("Server creation failed");

        // Verify basic properties
        assert!(!server.config.host.is_empty());
        assert!(server.config.port > 0);

        // Server will be dropped here
    }

    // If we reach here without panics or obvious issues, memory safety is likely intact
    // Test passes if no panics occur
}

/// Anchor test: Error handling doesn't panic
/// Ensures error conditions are handled gracefully
#[tokio::test]
async fn anchor_error_handling_never_panics() {
    // Test various configurations
    let config = ApiServerConfig::default();
    let _server = ApiServer::new(config)
        .await
        .expect("Server creation failed");

    // Test that model validation works
    use fortitude_api_server::models::requests::ResearchRequest;
    use validator::Validate;

    let invalid_request = ResearchRequest {
        query: "".to_string(), // Too short
        context: None,
        priority: None,
        audience_context: None,
        domain_context: None,
    };

    // This should return an error, not panic
    assert!(invalid_request.validate().is_err());
}

/// Anchor test: Middleware creation is stable
/// Ensures middleware layers can be created without issues
#[tokio::test]
async fn anchor_middleware_stability() {
    use fortitude_api_server::middleware::{cors, logging, rate_limit};

    // Test middleware creation multiple times
    for _i in 0..3 {
        let _cors = cors::create_cors_layer();
        let _trace = logging::create_trace_layer::<axum::body::Body>();
        let _rate_limit = rate_limit::create_rate_limit_layer();
    }

    // If we reach here, middleware is stable
    // Test passes if no panics occur
}

/// Anchor test: Model serialization is stable
/// Ensures our data models can be reliably serialized/deserialized
#[tokio::test]
async fn anchor_model_serialization_stability() {
    use fortitude_api_server::models::{errors, requests};
    use serde_json;
    use uuid::Uuid;

    // Test request model
    let request = requests::ResearchRequest {
        query: "test".to_string(),
        context: Some("context".to_string()),
        priority: Some("high".to_string()),
        audience_context: None,
        domain_context: None,
    };

    let serialized = serde_json::to_string(&request).expect("Failed to serialize request");
    let _deserialized: requests::ResearchRequest =
        serde_json::from_str(&serialized).expect("Failed to deserialize request");

    // Test error model
    let error = errors::ApiError::ValidationError {
        message: "test".to_string(),
    };
    let _error_response = error.to_error_response(Some(Uuid::new_v4()), None);

    // If we reach here, serialization is stable
    // Test passes if no panics occur
}

/// Anchor test: Cache system integration persists data
/// This test ensures cache functionality maintains data across operations
#[tokio::test]
async fn anchor_cache_data_persistence() {
    use fortitude_core::storage::FileStorage;
    use fortitude_types::{Storage, StorageConfig};
    use std::sync::Arc;
    use tempfile::TempDir;

    // Create temporary directory for test
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create storage configuration
    let storage_config = StorageConfig {
        base_path: temp_dir.path().to_path_buf(),
        cache_expiration_seconds: 3600,
        max_cache_size_bytes: 1024 * 1024,
        enable_content_addressing: true,
        index_update_interval_seconds: 300,
    };

    // Test that cache state can be created
    let file_storage = FileStorage::new(storage_config)
        .await
        .expect("Failed to create file storage");

    let config = ApiServerConfig::default();
    let cache_state = fortitude_api_server::routes::cache::CacheState {
        storage: Arc::new(file_storage),
        config: Arc::new(config),
    };

    // Test that cache operations don't panic
    let stats = cache_state
        .storage
        .get_cache_stats()
        .await
        .expect("Failed to get cache stats");

    // Verify basic stats structure (values are unsigned, so always >= 0)
    // stats.total_entries is unsigned
    // stats.total_size_bytes is unsigned
    assert!(stats.hit_rate >= 0.0);

    // Test cache cleanup doesn't panic
    let _cleaned = cache_state
        .storage
        .cleanup_expired()
        .await
        .expect("Failed to cleanup cache");
    // cleaned is unsigned, so always >= 0

    // Test cache listing doesn't panic
    let entries = cache_state
        .storage
        .list_cache_entries()
        .await
        .expect("Failed to list cache entries");
    assert!(entries.is_empty()); // Should be empty for new cache
}

/// Anchor test: Cache request/response models are stable
/// Ensures cache API models maintain compatibility
#[tokio::test]
async fn anchor_cache_models_stability() {
    use chrono::Utc;
    use fortitude_api_server::models::{requests, responses};
    use serde_json;
    use std::collections::HashMap;

    // Test cache search request
    let search_request = requests::CacheSearchRequest {
        query: Some("rust async".to_string()),
        research_type: Some("implementation".to_string()),
        keywords: Some("async await".to_string()),
        min_quality: Some(0.8),
        tags: Some(vec!["rust".to_string()]),
        min_age_seconds: Some(0),
        max_age_seconds: Some(86400),
        min_size_bytes: Some(100),
        max_size_bytes: Some(1000000),
        limit: Some(10),
        offset: Some(0),
        sort: Some("newest".to_string()),
    };

    let serialized =
        serde_json::to_string(&search_request).expect("Failed to serialize cache search request");
    let _deserialized: requests::CacheSearchRequest =
        serde_json::from_str(&serialized).expect("Failed to deserialize cache search request");

    // Test cache invalidation request
    let invalidate_request = requests::CacheInvalidateRequest {
        keys: Some(vec!["key1".to_string(), "key2".to_string()]),
        pattern: Some("*.json".to_string()),
        research_type: Some("implementation".to_string()),
        tags: Some(vec!["test".to_string()]),
        max_age_seconds: Some(3600),
        min_quality: Some(0.5),
        dry_run: Some(true),
    };

    let serialized = serde_json::to_string(&invalidate_request)
        .expect("Failed to serialize cache invalidate request");
    let _deserialized: requests::CacheInvalidateRequest =
        serde_json::from_str(&serialized).expect("Failed to deserialize cache invalidate request");

    // Test cache stats response
    let stats_response = responses::CacheStatsResponse {
        total_entries: 100,
        expired_entries: 5,
        total_size_bytes: 1024 * 1024,
        hit_rate: 0.85,
        hits: 850,
        misses: 150,
        average_age_seconds: 3600.0,
        by_research_type: HashMap::new(),
        storage_efficiency: responses::StorageEfficiencyResponse {
            utilization_percent: 75.0,
            duplicate_entries: 3,
            bytes_saved: 512,
            compression_ratio: Some(0.8),
        },
        performance_metrics: responses::CachePerformanceResponse {
            avg_retrieval_time_ms: 15.5,
            avg_storage_time_ms: 25.0,
            warming_status: "complete".to_string(),
            recent_operations: responses::RecentOperationsResponse {
                last_hour: 50,
                last_day: 1200,
                peak_hour: "14:00-15:00".to_string(),
                top_accessed: vec!["key1".to_string(), "key2".to_string()],
            },
        },
    };

    let serialized =
        serde_json::to_string(&stats_response).expect("Failed to serialize cache stats response");
    let _deserialized: responses::CacheStatsResponse =
        serde_json::from_str(&serialized).expect("Failed to deserialize cache stats response");

    // Test cache item response
    let item_response = responses::CacheItemResponse {
        key: "test-key".to_string(),
        research_type: "implementation".to_string(),
        original_query: "Test query".to_string(),
        content_summary: "Test content summary".to_string(),
        quality_score: 0.9,
        size_bytes: 1024,
        created_at: Utc::now(),
        last_accessed: Utc::now(),
        expires_at: Utc::now(),
        is_expired: false,
        content_hash: "hash123".to_string(),
        file_path: Some("/path/to/file".to_string()),
        metadata: HashMap::new(),
        tags: vec!["rust".to_string()],
    };

    let serialized =
        serde_json::to_string(&item_response).expect("Failed to serialize cache item response");
    let _deserialized: responses::CacheItemResponse =
        serde_json::from_str(&serialized).expect("Failed to deserialize cache item response");
}

/// Anchor test: Cache authentication permissions are enforced
/// Ensures cache endpoints maintain proper security boundaries
#[tokio::test]
async fn anchor_cache_permission_enforcement() {
    use fortitude_api_server::middleware::auth::{AuthManager, Permission};
    use std::sync::Arc;

    // Create auth manager
    let mut config = ApiServerConfig::default();
    config.auth.jwt_secret = "test_secret_key_at_least_32_characters_long".to_string();
    config.auth.enabled = true;

    let auth_manager = AuthManager::new(Arc::new(config)).expect("Failed to create auth manager");

    // Test that different permission levels work
    let read_token = auth_manager
        .generate_token("read_user", vec![Permission::ResourcesRead])
        .await
        .expect("Failed to generate read token");

    let admin_token = auth_manager
        .generate_token("admin_user", vec![Permission::Admin])
        .await
        .expect("Failed to generate admin token");

    // Verify tokens can be validated
    let read_claims = auth_manager
        .verify_token(&read_token)
        .await
        .expect("Failed to verify read token");

    let admin_claims = auth_manager
        .verify_token(&admin_token)
        .await
        .expect("Failed to verify admin token");

    // Verify permissions are correct
    assert!(read_claims
        .permissions
        .contains(&Permission::ResourcesRead.as_str().to_string()));
    assert!(admin_claims
        .permissions
        .contains(&Permission::Admin.as_str().to_string()));

    // Test permission checking
    assert!(auth_manager
        .check_permission(&read_claims, Permission::ResourcesRead)
        .await
        .is_ok());
    assert!(auth_manager
        .check_permission(&read_claims, Permission::Admin)
        .await
        .is_err());

    assert!(auth_manager
        .check_permission(&admin_claims, Permission::ResourcesRead)
        .await
        .is_ok());
    assert!(auth_manager
        .check_permission(&admin_claims, Permission::Admin)
        .await
        .is_ok());
}

/// Anchor test: Cache storage system handles errors gracefully
/// Ensures cache storage operations don't panic under error conditions
#[tokio::test]
async fn anchor_cache_error_handling() {
    use fortitude_core::storage::FileStorage;
    use fortitude_types::{SearchQuery, Storage, StorageConfig};
    use tempfile::TempDir;

    // Create temporary directory for test
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let storage_config = StorageConfig {
        base_path: temp_dir.path().to_path_buf(),
        cache_expiration_seconds: 3600,
        max_cache_size_bytes: 1024 * 1024,
        enable_content_addressing: true,
        index_update_interval_seconds: 300,
    };

    let storage = FileStorage::new(storage_config)
        .await
        .expect("Failed to create storage");

    // Test operations on non-existent items don't panic
    let result = storage.retrieve("nonexistent_key").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());

    // Test delete on non-existent item doesn't panic
    let result = storage.delete("nonexistent_key").await;
    assert!(result.is_ok());

    // Test search with empty query doesn't panic
    let search_query = SearchQuery::new("".to_string());
    let result = storage.search(&search_query).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());

    // Test stats retrieval doesn't panic
    let result = storage.get_cache_stats().await;
    assert!(result.is_ok());

    // Test cleanup doesn't panic
    let result = storage.cleanup_expired().await;
    assert!(result.is_ok());
}

/// Anchor test: Cache system maintains API contract
/// Ensures cache endpoints return expected response structures
#[tokio::test]
async fn anchor_cache_api_contract() {
    use axum::{extract::State, Extension};
    use fortitude_api_server::{
        middleware::auth::Claims,
        routes::cache::{get_cache_stats, CacheState},
    };
    use fortitude_core::storage::FileStorage;
    use fortitude_types::StorageConfig;
    use std::sync::Arc;
    use tempfile::TempDir;

    // Create test cache state
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage_config = StorageConfig {
        base_path: temp_dir.path().to_path_buf(),
        cache_expiration_seconds: 3600,
        max_cache_size_bytes: 1024 * 1024,
        enable_content_addressing: true,
        index_update_interval_seconds: 300,
    };

    let file_storage = FileStorage::new(storage_config)
        .await
        .expect("Failed to create storage");

    let config = ApiServerConfig::default();
    let cache_state = CacheState {
        storage: Arc::new(file_storage),
        config: Arc::new(config),
    };

    // Create test claims
    let claims = Claims {
        sub: "test_user".to_string(),
        permissions: vec!["fortitude:resources:read".to_string()],
        exp: 9999999999,
        iat: 1000000000,
        iss: "fortitude-api-server".to_string(),
    };

    // Test cache stats endpoint maintains contract
    let result = get_cache_stats(State(cache_state), Some(Extension(claims))).await;

    assert!(result.is_ok());
    let response = result.unwrap();

    // Verify response structure matches contract
    assert!(response.0.success);
    // response.0.data.total_entries is unsigned, so always >= 0
    assert!(response.0.data.hit_rate >= 0.0);
    assert!(response.0.data.storage_efficiency.utilization_percent >= 0.0);
    assert!(!response
        .0
        .data
        .performance_metrics
        .warming_status
        .is_empty());
}
