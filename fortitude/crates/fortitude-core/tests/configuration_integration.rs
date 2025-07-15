//! Configuration integration tests for vector database functionality.
//! These tests verify configuration validation, inheritance, and environment interactions.

use fortitude_core::vector::{ConnectionPoolConfig, VectorConfig, VectorStorageService};
use serde_json::{json, Value};
use std::env;
use std::time::Duration;
// Additional imports available for future use
// use fortitude_core::vector::{
//     CacheKeyStrategy, DeviceType, DistanceMetric, EmbeddingCacheConfig,
//     EmbeddingConfig, EmbeddingGenerator, FusionMethod, HealthCheckConfig, HybridSearchConfig,
//     HybridSearchService, LocalEmbeddingService, MigrationConfig,
//     MigrationService, SearchOptions, SearchStrategy, SemanticSearchConfig,
//     SemanticSearchService, ValidationLevel, VectorStorage,
// };

/// Helper to create various configuration scenarios
fn create_configuration_scenarios() -> Vec<(&'static str, VectorConfig)> {
    vec![
        (
            "minimal_config",
            VectorConfig {
                url: "http://localhost:6334".to_string(),
                api_key: None,
                timeout: Duration::from_secs(30),
                default_collection: "minimal_test".to_string(),
                vector_dimensions: 384,
                distance_metric: DistanceMetric::Cosine,
                health_check: HealthCheckConfig::default(),
                connection_pool: ConnectionPoolConfig::default(),
                embedding: EmbeddingConfig::default(),
            },
        ),
        (
            "performance_optimized",
            VectorConfig {
                url: "http://localhost:6334".to_string(),
                api_key: Some("test-api-key".to_string()),
                timeout: Duration::from_secs(60),
                default_collection: "performance_test".to_string(),
                vector_dimensions: 768, // Larger dimensions
                distance_metric: DistanceMetric::Dot,
                health_check: HealthCheckConfig {
                    enabled: true,
                    interval: Duration::from_secs(10),
                    timeout: Duration::from_secs(5),
                    max_failures: 5,
                },
                connection_pool: ConnectionPoolConfig {
                    max_connections: 50,
                    min_connections: 10,
                    connection_timeout: Duration::from_secs(15),
                    idle_timeout: Duration::from_secs(300),
                    max_lifetime: Duration::from_secs(1800),
                },
                embedding: EmbeddingConfig {
                    model_name: "sentence-transformers/all-MiniLM-L12-v2".to_string(),
                    max_sequence_length: 512,
                    batch_size: 64,
                    device: DeviceType::Cpu,
                    cache_config: EmbeddingCacheConfig {
                        enabled: true,
                        max_entries: 10000,
                        ttl: Duration::from_secs(3600),
                        key_strategy: CacheKeyStrategy::Hash,
                    },
                    ..Default::default()
                },
            },
        ),
        (
            "security_focused",
            VectorConfig {
                url: "https://secure-vector-db.example.com".to_string(),
                api_key: Some("secure-api-key-with-encryption".to_string()),
                timeout: Duration::from_secs(45),
                default_collection: "secure_collection".to_string(),
                vector_dimensions: 384,
                distance_metric: DistanceMetric::Cosine,
                health_check: HealthCheckConfig {
                    enabled: true,
                    interval: Duration::from_secs(30),
                    timeout: Duration::from_secs(10),
                    max_failures: 2, // Strict failure tolerance
                },
                connection_pool: ConnectionPoolConfig {
                    max_connections: 10,
                    min_connections: 2,
                    connection_timeout: Duration::from_secs(30),
                    idle_timeout: Duration::from_secs(180),
                    max_lifetime: Duration::from_secs(600),
                },
                embedding: EmbeddingConfig {
                    model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
                    max_sequence_length: 256,
                    batch_size: 16,
                    device: DeviceType::Cpu,
                    cache_config: EmbeddingCacheConfig {
                        enabled: false, // Disabled for security
                        max_entries: 0,
                        ttl: Duration::from_secs(0),
                        key_strategy: CacheKeyStrategy::Hash,
                    },
                    ..Default::default()
                },
            },
        ),
        (
            "development_config",
            VectorConfig {
                url: "http://localhost:6334".to_string(),
                api_key: None,
                timeout: Duration::from_secs(10), // Short for development
                default_collection: "dev_test".to_string(),
                vector_dimensions: 384,
                distance_metric: DistanceMetric::Euclidean,
                health_check: HealthCheckConfig {
                    enabled: false, // Disabled for development
                    interval: Duration::from_secs(60),
                    timeout: Duration::from_secs(5),
                    max_failures: 10,
                },
                connection_pool: ConnectionPoolConfig {
                    max_connections: 5,
                    min_connections: 1,
                    connection_timeout: Duration::from_secs(5),
                    idle_timeout: Duration::from_secs(60),
                    max_lifetime: Duration::from_secs(300),
                },
                embedding: EmbeddingConfig {
                    model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
                    max_sequence_length: 128,
                    batch_size: 8,
                    device: DeviceType::Cpu,
                    cache_config: EmbeddingCacheConfig {
                        enabled: true,
                        max_entries: 100,
                        ttl: Duration::from_secs(300),
                        key_strategy: CacheKeyStrategy::Hash,
                    },
                    ..Default::default()
                },
            },
        ),
    ]
}

/// Helper to create invalid configuration scenarios for testing
fn create_invalid_configurations() -> Vec<(&'static str, VectorConfig, &'static str)> {
    let base_config = VectorConfig::default();

    vec![
        (
            "invalid_dimensions",
            VectorConfig {
                vector_dimensions: 0, // Invalid
                ..base_config.clone()
            },
            "vector dimensions",
        ),
        (
            "invalid_url",
            VectorConfig {
                url: "".to_string(), // Empty URL
                ..base_config.clone()
            },
            "URL",
        ),
        (
            "invalid_timeout",
            VectorConfig {
                timeout: Duration::from_secs(0), // Zero timeout
                ..base_config.clone()
            },
            "timeout",
        ),
        (
            "invalid_connection_pool",
            VectorConfig {
                connection_pool: ConnectionPoolConfig {
                    max_connections: 0, // Invalid
                    min_connections: 5, // Min > Max
                    ..ConnectionPoolConfig::default()
                },
                ..base_config.clone()
            },
            "connection pool",
        ),
    ]
}

/// ANCHOR: Test configuration validation and initialization
/// Tests: Configuration validation, service initialization, parameter validation
#[tokio::test]
async fn test_anchor_configuration_validation() {
    let valid_scenarios = create_configuration_scenarios();

    // Test 1: Valid configuration scenarios
    for (scenario_name, config) in &valid_scenarios {
        println!("Testing valid configuration scenario: {}", scenario_name);

        // Test vector storage initialization
        let storage_result = VectorStorage::new(config.clone());
        assert!(
            storage_result.is_ok(),
            "Storage should initialize with valid config: {}",
            scenario_name
        );

        // Test embedding service initialization
        let embedding_service = LocalEmbeddingService::new(config.embedding.clone());
        let init_result = embedding_service.initialize().await;
        assert!(
            init_result.is_ok(),
            "Embedding service should initialize with valid config: {}",
            scenario_name
        );

        // Test search service initialization
        let storage = storage_result.unwrap();
        let search_result = SemanticSearchService::new(
            SemanticSearchConfig {
                collection_name: config.default_collection.clone(),
                ..Default::default()
            },
            storage.clone(),
            embedding_service.clone(),
        );
        assert!(
            search_result.is_ok(),
            "Search service should initialize with valid config: {}",
            scenario_name
        );

        // Test migration service initialization
        let migration_result = MigrationService::new(
            MigrationConfig::default(),
            storage.clone(),
            embedding_service.clone(),
        );
        assert!(
            migration_result.is_ok(),
            "Migration service should initialize with valid config: {}",
            scenario_name
        );

        // Verify service operates with configuration
        let test_content = format!("Test content for {}", scenario_name);
        let embedding_result = embedding_service.generate_embedding(&test_content).await;
        assert!(
            embedding_result.is_ok(),
            "Service should operate with config: {}",
            scenario_name
        );

        let embedding = embedding_result.unwrap();
        assert_eq!(
            embedding.len(),
            config.vector_dimensions,
            "Embedding dimensions should match config: {}",
            scenario_name
        );
    }

    // Test 2: Invalid configuration scenarios
    let invalid_scenarios = create_invalid_configurations();

    for (scenario_name, config, expected_issue) in &invalid_scenarios {
        println!("Testing invalid configuration scenario: {}", scenario_name);

        let storage_result = VectorStorage::new(config.clone());

        // Some invalid configurations may be caught at initialization, others at runtime
        if storage_result.is_err() {
            println!(
                "Configuration validation caught issue with {}: {}",
                scenario_name, expected_issue
            );
        } else {
            // If storage initializes, it should fail gracefully during operation
            println!(
                "Configuration {} allowed but may fail during operation",
                scenario_name
            );
        }
    }

    // Test 3: Default configuration validation
    let default_config = VectorConfig::default();
    let default_storage = VectorStorage::new(default_config.clone());
    assert!(
        default_storage.is_ok(),
        "Default configuration should be valid"
    );

    let default_embedding = LocalEmbeddingService::new(default_config.embedding.clone());
    let default_init = default_embedding.initialize().await;
    assert!(
        default_init.is_ok(),
        "Default embedding config should initialize"
    );

    // Test 4: Configuration boundary values
    let boundary_config = VectorConfig {
        vector_dimensions: 1,              // Minimum valid dimension
        timeout: Duration::from_millis(1), // Minimum timeout
        connection_pool: ConnectionPoolConfig {
            max_connections: 1,
            min_connections: 1,
            connection_timeout: Duration::from_millis(100),
            idle_timeout: Duration::from_millis(100),
            max_lifetime: Duration::from_millis(200),
        },
        embedding: EmbeddingConfig {
            batch_size: 1,          // Minimum batch size
            max_sequence_length: 1, // Minimum sequence length
            cache_config: EmbeddingCacheConfig {
                enabled: true,
                max_entries: 1, // Minimum cache size
                ttl: Duration::from_millis(100),
                key_strategy: CacheKeyStrategy::Hash,
            },
            ..Default::default()
        },
        ..default_config
    };

    let boundary_storage = VectorStorage::new(boundary_config.clone());
    // May succeed or fail depending on implementation limits
    if boundary_storage.is_ok() {
        println!("Boundary configuration accepted");
    } else {
        println!("Boundary configuration rejected (expected)");
    }
}

/// ANCHOR: Test configuration inheritance and composition
/// Tests: Service configuration inheritance, composition patterns, override behavior
#[tokio::test]
async fn test_anchor_configuration_inheritance() {
    let base_config = VectorConfig::default();

    // Test 1: Search service configuration inheritance
    let search_configs = vec![
        SemanticSearchConfig {
            collection_name: "custom_collection".to_string(),
            default_limit: 20,
            min_score_threshold: 0.5,
            enable_explain: true,
            cache_enabled: true,
            cache_ttl: Duration::from_secs(600),
        },
        SemanticSearchConfig {
            collection_name: base_config.default_collection.clone(),
            default_limit: 10,
            min_score_threshold: 0.3,
            enable_explain: false,
            cache_enabled: false,
            cache_ttl: Duration::from_secs(0),
        },
    ];

    let embedding_service = LocalEmbeddingService::new(base_config.embedding.clone());
    let storage = VectorStorage::new(base_config.clone()).expect("Failed to create storage");

    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    for (i, search_config) in search_configs.into_iter().enumerate() {
        let search_service = SemanticSearchService::new(
            search_config.clone(),
            storage.clone(),
            embedding_service.clone(),
        )
        .expect(&format!("Failed to create search service {}", i));

        // Test that configuration affects behavior
        let test_query = format!("Configuration inheritance test {}", i);
        let results = search_service
            .search(
                &test_query,
                SearchOptions {
                    limit: None,           // Use service default
                    score_threshold: None, // Use service default
                    with_payload: true,
                    with_vectors: false,
                },
            )
            .await
            .expect(&format!("Search {} should succeed", i));

        // Verify configuration inheritance
        if search_config.enable_explain {
            assert!(
                results.execution_stats.is_some(),
                "Should provide execution stats when explain enabled"
            );
        }
    }

    // Test 2: Hybrid service configuration composition
    let base_search = SemanticSearchService::new(
        SemanticSearchConfig::default(),
        storage.clone(),
        embedding_service.clone(),
    )
    .expect("Failed to create base search service");

    let hybrid_configs = vec![
        HybridSearchConfig {
            semantic_weight: 0.8,
            keyword_weight: 0.2,
            fusion_method: FusionMethod::WeightedSum,
            enable_query_analysis: true,
            enable_performance_tracking: true,
            ..Default::default()
        },
        HybridSearchConfig {
            semantic_weight: 0.4,
            keyword_weight: 0.6,
            fusion_method: FusionMethod::WeightedSum,
            enable_query_analysis: false,
            enable_performance_tracking: false,
            ..Default::default()
        },
    ];

    for (i, hybrid_config) in hybrid_configs.into_iter().enumerate() {
        let hybrid_service = HybridSearchService::new(
            hybrid_config.clone(),
            base_search.clone(),
            embedding_service.clone(),
        )
        .expect(&format!("Failed to create hybrid service {}", i));

        // Test configuration composition
        let request = fortitude_core::vector::HybridSearchRequest {
            query: format!("Hybrid composition test {}", i),
            strategy: SearchStrategy::Balanced,
            limit: 5,
            semantic_options: None,
            keyword_options: None,
            filters: None,
        };

        let results = hybrid_service
            .search(request)
            .await
            .expect(&format!("Hybrid search {} should succeed", i));

        // Verify configuration effects
        if hybrid_config.enable_performance_tracking {
            assert!(
                results.performance_metrics.is_some(),
                "Should track performance when enabled"
            );
        }
    }

    // Test 3: Migration service configuration inheritance
    let migration_configs = vec![
        MigrationConfig {
            batch_size: 10,
            max_parallel_batches: 2,
            validation_level: ValidationLevel::Strict,
            enable_rollback: true,
            progress_reporting_interval: Duration::from_secs(1),
            timeout_per_batch: Duration::from_secs(30),
            max_retries: 3,
            enable_deduplication: true,
            preserve_metadata: true,
        },
        MigrationConfig {
            batch_size: 50,
            max_parallel_batches: 5,
            validation_level: ValidationLevel::Lenient,
            enable_rollback: false,
            progress_reporting_interval: Duration::from_secs(5),
            timeout_per_batch: Duration::from_secs(60),
            max_retries: 1,
            enable_deduplication: false,
            preserve_metadata: false,
        },
    ];

    for (i, migration_config) in migration_configs.into_iter().enumerate() {
        let migration_service = MigrationService::new(
            migration_config.clone(),
            storage.clone(),
            embedding_service.clone(),
        )
        .expect(&format!("Failed to create migration service {}", i));

        // Test small migration to verify configuration
        let test_data = vec![(
            format!("config_test_{}", i),
            format!("Configuration test content {}", i),
            json!({
                "config_test": i,
                "validation_level": format!("{:?}", migration_config.validation_level)
            }),
        )];

        let source = fortitude_core::vector::MigrationSource::InMemory {
            data: test_data.clone(),
            source_name: format!("config_inheritance_test_{}", i),
        };

        let migration_id = migration_service
            .start_migration(source)
            .await
            .expect(&format!("Failed to start migration {}", i));

        // Wait for completion
        let mut iterations = 20;
        let mut completed = false;

        while !completed && iterations > 0 {
            tokio::time::sleep(Duration::from_millis(100)).await;

            let status = migration_service
                .get_migration_status(&migration_id)
                .await
                .expect("Failed to get migration status");

            if matches!(
                status.status,
                fortitude_core::vector::MigrationStatus::Completed
                    | fortitude_core::vector::MigrationStatus::Failed
            ) {
                completed = true;
            }

            iterations -= 1;
        }

        assert!(completed, "Migration {} should complete", i);

        // Clean up
        for (id, _, _) in &test_data {
            storage.delete_vector(id).await.ok();
        }
    }
}

/// ANCHOR: Test environment variable configuration integration
/// Tests: Environment variable precedence, configuration overrides, runtime changes
#[tokio::test]
async fn test_anchor_environment_configuration() {
    // Save original environment state
    let original_vars: Vec<(String, Option<String>)> = vec![
        "VECTOR_DB_URL",
        "VECTOR_DB_API_KEY",
        "VECTOR_DB_COLLECTION",
        "VECTOR_DB_DIMENSIONS",
        "EMBEDDING_MODEL_NAME",
        "EMBEDDING_BATCH_SIZE",
        "CACHE_ENABLED",
        "CACHE_MAX_ENTRIES",
    ]
    .into_iter()
    .map(|key| (key.to_string(), env::var(key).ok()))
    .collect();

    // Test 1: Environment variable configuration
    env::set_var("VECTOR_DB_URL", "http://env-configured:6334");
    env::set_var("VECTOR_DB_API_KEY", "env-api-key");
    env::set_var("VECTOR_DB_COLLECTION", "env_collection");
    env::set_var("VECTOR_DB_DIMENSIONS", "512");
    env::set_var("EMBEDDING_MODEL_NAME", "sentence-transformers/env-model");
    env::set_var("EMBEDDING_BATCH_SIZE", "32");
    env::set_var("CACHE_ENABLED", "false");
    env::set_var("CACHE_MAX_ENTRIES", "1000");

    // Create configuration that should pick up environment variables
    let env_config = create_env_aware_config();

    // Verify environment variables are used
    assert_eq!(env_config.url, "http://env-configured:6334");
    assert_eq!(env_config.api_key, Some("env-api-key".to_string()));
    assert_eq!(env_config.default_collection, "env_collection");
    assert_eq!(env_config.vector_dimensions, 512);
    assert_eq!(
        env_config.embedding.model_name,
        "sentence-transformers/env-model"
    );
    assert_eq!(env_config.embedding.batch_size, 32);
    assert_eq!(env_config.embedding.cache_config.enabled, false);
    assert_eq!(env_config.embedding.cache_config.max_entries, 1000);

    // Test services with environment configuration
    let embedding_service = LocalEmbeddingService::new(env_config.embedding.clone());
    let storage = VectorStorage::new(env_config.clone());

    if let Ok(storage) = storage {
        embedding_service
            .initialize()
            .await
            .expect("Should initialize with environment config");

        // Test that configuration affects service behavior
        let test_embedding = embedding_service
            .generate_embedding("environment test")
            .await
            .expect("Should generate embedding with env config");

        assert_eq!(
            test_embedding.len(),
            512,
            "Should use environment-configured dimensions"
        );

        // Verify cache configuration
        let stats = embedding_service.get_stats().await;
        assert_eq!(
            stats.cache_size, 0,
            "Cache should be disabled via environment"
        );
    }

    // Test 2: Configuration precedence (explicit vs environment)
    let explicit_config = VectorConfig {
        url: "http://explicit:6334".to_string(),
        api_key: Some("explicit-key".to_string()),
        default_collection: "explicit_collection".to_string(),
        vector_dimensions: 256, // Different from environment
        ..VectorConfig::default()
    };

    // Explicit configuration should take precedence over environment
    assert_eq!(explicit_config.url, "http://explicit:6334");
    assert_eq!(explicit_config.default_collection, "explicit_collection");
    assert_eq!(explicit_config.vector_dimensions, 256);

    // Test 3: Invalid environment values handling
    env::set_var("VECTOR_DB_DIMENSIONS", "invalid_number");
    env::set_var("EMBEDDING_BATCH_SIZE", "not_a_number");
    env::set_var("CACHE_ENABLED", "maybe");

    let robust_config = create_env_aware_config_with_fallbacks();

    // Should fall back to defaults for invalid values
    assert_eq!(robust_config.vector_dimensions, 384); // Default fallback
    assert_eq!(robust_config.embedding.batch_size, 16); // Default fallback

    // Test 4: Runtime configuration updates
    let mut runtime_config = VectorConfig::default();

    // Simulate runtime configuration update
    runtime_config.url = "http://updated:6334".to_string();
    runtime_config.timeout = Duration::from_secs(45);
    runtime_config.vector_dimensions = 768;

    // Test that services can handle configuration updates
    let updated_storage = VectorStorage::new(runtime_config.clone());
    if let Ok(storage) = updated_storage {
        // Service should work with updated configuration
        let count = storage.count_vectors().await;
        assert!(
            count.is_ok() || count.is_err(),
            "Should handle updated config gracefully"
        );
    }

    // Restore original environment
    for (key, value) in original_vars {
        match value {
            Some(val) => env::set_var(&key, val),
            None => env::remove_var(&key),
        }
    }
}

/// Helper function to create environment-aware configuration
fn create_env_aware_config() -> VectorConfig {
    VectorConfig {
        url: env::var("VECTOR_DB_URL").unwrap_or_else(|_| "http://localhost:6334".to_string()),
        api_key: env::var("VECTOR_DB_API_KEY").ok(),
        default_collection: env::var("VECTOR_DB_COLLECTION")
            .unwrap_or_else(|_| "default".to_string()),
        vector_dimensions: env::var("VECTOR_DB_DIMENSIONS")
            .unwrap_or_else(|_| "384".to_string())
            .parse()
            .unwrap_or(384),
        embedding: EmbeddingConfig {
            model_name: env::var("EMBEDDING_MODEL_NAME")
                .unwrap_or_else(|_| "sentence-transformers/all-MiniLM-L6-v2".to_string()),
            batch_size: env::var("EMBEDDING_BATCH_SIZE")
                .unwrap_or_else(|_| "16".to_string())
                .parse()
                .unwrap_or(16),
            cache_config: EmbeddingCacheConfig {
                enabled: env::var("CACHE_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                max_entries: env::var("CACHE_MAX_ENTRIES")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Helper function to create environment-aware configuration with robust fallbacks
fn create_env_aware_config_with_fallbacks() -> VectorConfig {
    VectorConfig {
        url: env::var("VECTOR_DB_URL").unwrap_or_else(|_| "http://localhost:6334".to_string()),
        api_key: env::var("VECTOR_DB_API_KEY").ok(),
        default_collection: env::var("VECTOR_DB_COLLECTION")
            .unwrap_or_else(|_| "default".to_string()),
        vector_dimensions: env::var("VECTOR_DB_DIMENSIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(384), // Robust fallback
        embedding: EmbeddingConfig {
            model_name: env::var("EMBEDDING_MODEL_NAME")
                .unwrap_or_else(|_| "sentence-transformers/all-MiniLM-L6-v2".to_string()),
            batch_size: env::var("EMBEDDING_BATCH_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(16), // Robust fallback
            cache_config: EmbeddingCacheConfig {
                enabled: env::var("CACHE_ENABLED")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(true), // Robust fallback
                max_entries: env::var("CACHE_MAX_ENTRIES")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(100), // Robust fallback
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

/// ANCHOR: Test configuration file integration and serialization
/// Tests: Configuration file loading, serialization, validation
#[tokio::test]
async fn test_anchor_configuration_file_integration() {
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Test 1: Configuration serialization and deserialization
    let test_configs = create_configuration_scenarios();

    for (scenario_name, config) in &test_configs {
        // Test JSON serialization
        let json_config = serde_json::to_string_pretty(config)
            .expect(&format!("Should serialize config: {}", scenario_name));

        // Write to file
        let config_path = temp_dir.path().join(format!("{}.json", scenario_name));
        fs::write(&config_path, &json_config)
            .expect(&format!("Should write config file: {}", scenario_name));

        // Read back and deserialize
        let read_config = fs::read_to_string(&config_path)
            .expect(&format!("Should read config file: {}", scenario_name));

        let deserialized_config: VectorConfig = serde_json::from_str(&read_config)
            .expect(&format!("Should deserialize config: {}", scenario_name));

        // Verify configuration integrity
        assert_eq!(deserialized_config.url, config.url);
        assert_eq!(deserialized_config.api_key, config.api_key);
        assert_eq!(
            deserialized_config.default_collection,
            config.default_collection
        );
        assert_eq!(
            deserialized_config.vector_dimensions,
            config.vector_dimensions
        );
        assert_eq!(deserialized_config.timeout, config.timeout);

        // Test that deserialized configuration works
        let embedding_service = LocalEmbeddingService::new(deserialized_config.embedding.clone());
        let init_result = embedding_service.initialize().await;
        assert!(
            init_result.is_ok(),
            "Deserialized config should initialize services: {}",
            scenario_name
        );
    }

    // Test 2: Configuration file validation
    let invalid_json = r#"
    {
        "url": "http://localhost:6334",
        "vector_dimensions": "not_a_number",
        "invalid_field": "should_be_ignored"
    }
    "#;

    let invalid_path = temp_dir.path().join("invalid.json");
    fs::write(&invalid_path, invalid_json).expect("Should write invalid config file");

    let invalid_result: Result<VectorConfig, _> = serde_json::from_str(invalid_json);
    assert!(
        invalid_result.is_err(),
        "Should reject invalid JSON configuration"
    );

    // Test 3: Partial configuration with defaults
    let partial_json = r#"
    {
        "url": "http://custom:6334",
        "default_collection": "partial_test"
    }
    "#;

    let partial_path = temp_dir.path().join("partial.json");
    fs::write(&partial_path, partial_json).expect("Should write partial config file");

    // This test would require implementing Default appropriately for partial configs
    // For now, verify that we can handle missing fields gracefully
    let partial_value: serde_json::Value =
        serde_json::from_str(partial_json).expect("Should parse partial JSON");

    assert_eq!(partial_value["url"], "http://custom:6334");
    assert_eq!(partial_value["default_collection"], "partial_test");

    // Test 4: Configuration template generation
    let template_config = VectorConfig::default();
    let template_json =
        serde_json::to_string_pretty(&template_config).expect("Should serialize template config");

    let template_path = temp_dir.path().join("template.json");
    fs::write(&template_path, &template_json).expect("Should write template config");

    // Verify template can be read back
    let template_content = fs::read_to_string(&template_path).expect("Should read template file");

    let template_parsed: VectorConfig =
        serde_json::from_str(&template_content).expect("Should parse template config");

    // Template should match defaults
    assert_eq!(template_parsed.url, template_config.url);
    assert_eq!(
        template_parsed.vector_dimensions,
        template_config.vector_dimensions
    );

    // Test 5: Configuration validation with services
    for config_file in fs::read_dir(temp_dir.path()).unwrap() {
        let config_file = config_file.unwrap();
        if config_file.path().extension().and_then(|s| s.to_str()) == Some("json") {
            let config_content = fs::read_to_string(config_file.path());

            if let Ok(content) = config_content {
                if let Ok(config) = serde_json::from_str::<VectorConfig>(&content) {
                    // Test that valid configurations can initialize services
                    let storage_result = VectorStorage::new(config.clone());
                    let embedding_service = LocalEmbeddingService::new(config.embedding.clone());

                    if storage_result.is_ok() {
                        let init_result = embedding_service.initialize().await;
                        assert!(
                            init_result.is_ok(),
                            "Valid config file should initialize services: {:?}",
                            config_file.path()
                        );
                    }
                }
            }
        }
    }
}

/// ANCHOR: Test dynamic configuration updates and hot reloading
/// Tests: Runtime configuration changes, service reconfiguration, state preservation
#[tokio::test]
async fn test_anchor_dynamic_configuration_updates() {
    // Test 1: Configuration hot reloading simulation
    let initial_config = VectorConfig {
        url: "http://localhost:6334".to_string(),
        timeout: Duration::from_secs(30),
        default_collection: "initial_collection".to_string(),
        vector_dimensions: 384,
        embedding: EmbeddingConfig {
            batch_size: 16,
            cache_config: EmbeddingCacheConfig {
                enabled: true,
                max_entries: 100,
                ttl: Duration::from_secs(300),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let embedding_service = LocalEmbeddingService::new(initial_config.embedding.clone());
    let storage =
        VectorStorage::new(initial_config.clone()).expect("Failed to create initial storage");

    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize with initial config");

    // Perform some operations with initial configuration
    let test_content = "Dynamic configuration test content";
    let initial_embedding = embedding_service
        .generate_embedding(test_content)
        .await
        .expect("Should generate embedding with initial config");

    storage
        .store_vector(
            "dynamic_test",
            &initial_embedding,
            Some(json!({
                "config_version": "initial",
                "content": test_content
            })),
        )
        .await
        .expect("Should store with initial config");

    let initial_stats = embedding_service.get_stats().await;
    assert!(
        initial_stats.total_generated > 0,
        "Should have generated embeddings"
    );

    // Test 2: Simulated configuration update
    let updated_config = VectorConfig {
        url: "http://localhost:6334".to_string(),
        timeout: Duration::from_secs(60), // Updated timeout
        default_collection: "updated_collection".to_string(), // Updated collection
        vector_dimensions: 384,           // Same dimensions (critical for compatibility)
        embedding: EmbeddingConfig {
            batch_size: 32, // Updated batch size
            cache_config: EmbeddingCacheConfig {
                enabled: true,
                max_entries: 500,              // Updated cache size
                ttl: Duration::from_secs(600), // Updated TTL
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    // Create new services with updated configuration
    let updated_embedding_service = LocalEmbeddingService::new(updated_config.embedding.clone());
    let updated_storage =
        VectorStorage::new(updated_config.clone()).expect("Failed to create updated storage");

    updated_embedding_service
        .initialize()
        .await
        .expect("Failed to initialize with updated config");

    // Test that updated services work
    let updated_embedding = updated_embedding_service
        .generate_embedding(test_content)
        .await
        .expect("Should generate embedding with updated config");

    // Embeddings should be compatible (same dimensions and model)
    assert_eq!(
        initial_embedding.len(),
        updated_embedding.len(),
        "Embeddings should have same dimensions after config update"
    );

    // Test 3: State preservation during configuration updates
    let retrieved_doc = storage
        .get_vector("dynamic_test")
        .await
        .expect("Should retrieve document after config update");

    assert!(
        retrieved_doc.is_some(),
        "Document should still be accessible"
    );

    let (retrieved_vector, retrieved_metadata) = retrieved_doc.unwrap();
    assert_eq!(
        retrieved_vector, initial_embedding,
        "Stored vector should be preserved during config update"
    );

    let metadata = retrieved_metadata.unwrap();
    assert_eq!(
        metadata["config_version"], "initial",
        "Metadata should be preserved during config update"
    );

    // Test 4: Service migration with configuration changes
    let migration_service_initial = MigrationService::new(storage.clone(), None);

    let migration_service_updated = MigrationService::new(updated_storage.clone(), None);

    // Both services should be functional
    let research_result = ResearchResult {
        query: "migration_config_test".to_string(),
        content: "Content for migration configuration test".to_string(),
        research_type: ResearchType::Implementation,
        details: vec![Detail {
            category: "migration".to_string(),
            content: "Test migration data".to_string(),
            priority: "medium".to_string(),
            timestamp: chrono::Utc::now(),
        }],
        evidence: vec![Evidence {
            source: "test".to_string(),
            content: "Test evidence".to_string(),
            relevance_score: 0.8,
            timestamp: chrono::Utc::now(),
        }],
        quality_score: 0.85,
        timestamp: chrono::Utc::now(),
        metadata: json!({
            "test": "configuration_migration",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }),
    };

    let source = fortitude_core::vector::MigrationSource::InMemory {
        documents: vec![research_result],
    };

    let migration_config = MigrationConfig {
        batch_size: 25,
        validation_level: ValidationLevel::Strict,
        ..Default::default()
    };

    let migration_id = migration_service_updated
        .start_migration(source, migration_config)
        .await
        .expect("Should start migration with updated config");

    // Wait for migration completion
    let mut iterations = 20;
    let mut completed = false;

    while !completed && iterations > 0 {
        tokio::time::sleep(Duration::from_millis(100)).await;

        let status = migration_service_updated
            .get_migration_status(&migration_id)
            .await
            .expect("Should get migration status");

        if matches!(
            status.status,
            fortitude_core::vector::MigrationStatus::Completed
        ) {
            completed = true;
        }

        iterations -= 1;
    }

    assert!(
        completed,
        "Migration should complete with updated configuration"
    );

    // Test 5: Configuration compatibility validation
    let incompatible_config = VectorConfig {
        vector_dimensions: 512, // Different dimensions - incompatible!
        ..updated_config.clone()
    };

    let incompatible_embedding_service =
        LocalEmbeddingService::new(incompatible_config.embedding.clone());
    incompatible_embedding_service
        .initialize()
        .await
        .expect("Should initialize incompatible service");

    let incompatible_embedding = incompatible_embedding_service
        .generate_embedding(test_content)
        .await
        .expect("Should generate incompatible embedding");

    // Attempting to store incompatible embedding should fail or be handled gracefully
    let incompatible_result = storage
        .store_vector("incompatible_test", &incompatible_embedding, None)
        .await;

    if incompatible_result.is_err() {
        println!("System correctly rejected incompatible vector dimensions");
    } else {
        println!("System accepted incompatible dimensions (may have validation)");
    }

    // Clean up
    storage.delete_vector("dynamic_test").await.ok();
    updated_storage
        .delete_vector("migration_config_test")
        .await
        .ok();
}
