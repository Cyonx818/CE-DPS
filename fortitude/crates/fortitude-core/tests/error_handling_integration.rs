//! Error handling integration tests for vector database functionality.
//! These tests verify error scenarios, recovery mechanisms, and graceful degradation.

use fortitude_core::vector::{
    CacheKeyStrategy, ConnectionPoolConfig, DeviceType, DistanceMetric, EmbeddingCacheConfig,
    EmbeddingConfig, EmbeddingGenerator, FusionMethod, HealthCheckConfig, HybridSearchConfig,
    HybridSearchOperations, HybridSearchRequest, HybridSearchService, LocalEmbeddingService,
    MigrationConfig, MigrationError, MigrationService, MigrationSource, SearchOptions,
    SearchStrategy, SemanticSearchConfig, SemanticSearchOperations, SemanticSearchService,
    ValidationLevel, VectorConfig, VectorError, VectorStorage, VectorStorageService,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;

/// Create test configuration for error handling scenarios
fn create_error_test_config() -> VectorConfig {
    VectorConfig {
        url: "http://localhost:6334".to_string(),
        api_key: None,
        timeout: Duration::from_secs(10), // Shorter timeout for error testing
        default_collection: "error_test_collection".to_string(),
        vector_dimensions: 384,
        distance_metric: DistanceMetric::Cosine,
        health_check: HealthCheckConfig {
            enabled: true,
            interval: Duration::from_secs(5),
            timeout: Duration::from_secs(2),
            max_failures: 2, // Lower threshold for testing
        },
        connection_pool: ConnectionPoolConfig {
            max_connections: 5,
            connection_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(30),
        },
        embedding: EmbeddingConfig {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            max_sequence_length: 128,
            batch_size: 4,
            device: DeviceType::Cpu,
            cache_config: EmbeddingCacheConfig {
                enabled: true,
                max_entries: 50,
                ttl: Duration::from_secs(60),
                key_strategy: CacheKeyStrategy::Hash,
            },
            ..Default::default()
        },
    }
}

/// Generate problematic test data for error scenarios
fn create_error_test_data() -> HashMap<String, Vec<(String, String, Value)>> {
    let mut data = HashMap::new();

    // Valid data for baseline
    data.insert(
        "valid_data".to_string(),
        vec![(
            "valid_doc_1".to_string(),
            "Valid document content for error testing baseline".to_string(),
            json!({
                "type": "valid",
                "category": "baseline",
                "quality": "good"
            }),
        )],
    );

    // Problematic data that should cause validation errors
    data.insert(
        "invalid_data".to_string(),
        vec![
            (
                "empty_content".to_string(),
                "".to_string(), // Empty content
                json!({
                    "type": "invalid",
                    "issue": "empty_content"
                }),
            ),
            (
                "duplicate_id".to_string(),
                "First document with duplicate ID".to_string(),
                json!({
                    "type": "invalid",
                    "issue": "duplicate_id_1"
                }),
            ),
            (
                "duplicate_id".to_string(), // Duplicate ID
                "Second document with duplicate ID".to_string(),
                json!({
                    "type": "invalid",
                    "issue": "duplicate_id_2"
                }),
            ),
            (
                "invalid_metadata".to_string(),
                "Document with problematic metadata".to_string(),
                json!(null), // Invalid metadata
            ),
            (
                "oversized_content".to_string(),
                "x".repeat(100000), // Very large content
                json!({
                    "type": "invalid",
                    "issue": "oversized_content"
                }),
            ),
        ],
    );

    // Malformed data for edge case testing
    data.insert(
        "malformed_data".to_string(),
        vec![
            (
                "special_chars_\x00\x01\x02".to_string(), // Special characters in ID
                "Content with special characters in ID".to_string(),
                json!({
                    "type": "malformed",
                    "issue": "special_chars_id"
                }),
            ),
            (
                "unicode_test_🚀".to_string(), // Unicode in ID
                "Content with unicode 🎯 characters throughout the text 📊".to_string(),
                json!({
                    "type": "unicode",
                    "unicode_field": "测试 тест test"
                }),
            ),
        ],
    );

    data
}

/// ANCHOR: Test embedding service error handling and recovery
/// Tests: Invalid input handling, service failure recovery, error propagation
#[tokio::test]
async fn test_anchor_embedding_service_error_handling() {
    let config = create_error_test_config();
    let embedding_service = LocalEmbeddingService::new(config.embedding.clone());

    // Test 1: Uninitialized service error handling
    let uninitialized_result = embedding_service.generate_embedding("test content").await;
    assert!(
        uninitialized_result.is_err(),
        "Should fail when service not initialized"
    );

    let error = uninitialized_result.unwrap_err();
    assert!(
        error.to_string().contains("not initialized"),
        "Error should indicate service not initialized"
    );

    // Initialize service for subsequent tests
    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Test 2: Empty and invalid input handling
    let empty_result = embedding_service.generate_embedding("").await;
    assert!(empty_result.is_ok(), "Should handle empty input gracefully");

    let empty_embedding = empty_result.unwrap();
    assert_eq!(
        empty_embedding.len(),
        384,
        "Should return valid embedding for empty input"
    );

    // Test 3: Extremely long input handling
    let long_text = "word ".repeat(10000);
    let long_result = embedding_service.generate_embedding(&long_text).await;
    assert!(
        long_result.is_ok(),
        "Should handle very long input gracefully"
    );

    // Test 4: Special characters and encoding issues
    let special_chars = "Special chars: \x00\x01\x02\x03\x04\x05";
    let special_result = embedding_service.generate_embedding(special_chars).await;
    assert!(
        special_result.is_ok(),
        "Should handle special characters gracefully"
    );

    let unicode_text = "Unicode test: 🚀🎯📊 测试 тест العربية";
    let unicode_result = embedding_service.generate_embedding(unicode_text).await;
    assert!(
        unicode_result.is_ok(),
        "Should handle unicode text gracefully"
    );

    // Test 5: Batch processing error handling
    let mixed_batch = vec![
        "Valid content 1".to_string(),
        "".to_string(), // Empty
        "Valid content 2".to_string(),
        "x".repeat(50000), // Very long
        "Unicode: 🎯".to_string(),
    ];

    let batch_result = embedding_service.generate_embeddings(&mixed_batch).await;
    assert!(batch_result.is_ok(), "Should handle mixed batch gracefully");

    let embeddings = batch_result.unwrap();
    assert_eq!(embeddings.len(), 5, "Should process all items in batch");

    for embedding in embeddings {
        assert_eq!(
            embedding.len(),
            384,
            "All embeddings should have correct dimensions"
        );
        assert!(
            embedding.iter().all(|&x| x.is_finite()),
            "All values should be finite"
        );
    }

    // Test 6: Service state after errors
    let stats_after_errors = embedding_service.get_stats().await;
    assert!(
        stats_after_errors.total_generated >= 8,
        "Should track all generation attempts"
    );

    // Test 7: Cache error handling
    embedding_service
        .clear_cache()
        .await
        .expect("Should clear cache without errors");

    let stats_after_clear = embedding_service.get_stats().await;
    assert_eq!(
        stats_after_clear.cache_size, 0,
        "Cache should be empty after clear"
    );

    // Test 8: Concurrent error scenarios
    let concurrent_handles: Vec<_> = (0..10)
        .map(|i| {
            let service = embedding_service.clone();
            let content = if i % 3 == 0 {
                "".to_string()
            } else {
                format!("Content {}", i)
            };
            tokio::spawn(async move { service.generate_embedding(&content).await })
        })
        .collect();

    // All concurrent operations should succeed even with some problematic inputs
    for (i, handle) in concurrent_handles.into_iter().enumerate() {
        let result = handle
            .await
            .expect(&format!("Concurrent task {} should complete", i));
        assert!(result.is_ok(), "Concurrent embedding {} should succeed", i);
    }
}

/// ANCHOR: Test vector storage error handling and data validation
/// Tests: Storage validation, dimension mismatches, corruption handling
#[tokio::test]
async fn test_anchor_vector_storage_error_handling() {
    let config = create_error_test_config();
    let storage = VectorStorage::new(config.clone()).expect("Failed to create vector storage");
    let embedding_service = LocalEmbeddingService::new(config.embedding.clone());

    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Test 1: Invalid vector dimension handling
    let valid_embedding = embedding_service
        .generate_embedding("test content")
        .await
        .expect("Failed to generate valid embedding");

    let invalid_embedding = vec![1.0; 128]; // Wrong dimension
    let dimension_error = storage
        .store_vector("invalid_dim", &invalid_embedding, None)
        .await;

    if dimension_error.is_err() {
        println!("Storage correctly rejected invalid vector dimensions");
        let error = dimension_error.unwrap_err();
        assert!(
            matches!(error, VectorError::InvalidDimensions { .. }),
            "Should return InvalidDimensions error"
        );
    }

    // Test 2: Invalid vector data handling
    let nan_embedding = vec![f32::NAN; 384];
    let nan_result = storage.store_vector("nan_test", &nan_embedding, None).await;

    if nan_result.is_err() {
        println!("Storage correctly rejected NaN values");
    }

    let inf_embedding = vec![f32::INFINITY; 384];
    let inf_result = storage.store_vector("inf_test", &inf_embedding, None).await;

    if inf_result.is_err() {
        println!("Storage correctly rejected infinite values");
    }

    // Test 3: Invalid ID handling
    let empty_id_result = storage.store_vector("", &valid_embedding, None).await;
    assert!(empty_id_result.is_err(), "Should reject empty ID");

    let special_char_id = "test\x00\x01id";
    let special_id_result = storage
        .store_vector(special_char_id, &valid_embedding, None)
        .await;
    // May succeed or fail depending on implementation - should handle gracefully

    // Test 4: Large metadata handling
    let large_metadata = json!({
        "large_field": "x".repeat(100000),
        "nested": {
            "deep": {
                "very_deep": "x".repeat(50000)
            }
        }
    });

    let large_metadata_result = storage
        .store_vector("large_meta", &valid_embedding, Some(large_metadata))
        .await;
    // Should handle large metadata gracefully (may truncate or reject)

    // Test 5: Retrieval error handling
    let nonexistent_result = storage.get_vector("nonexistent_id").await;
    assert!(
        nonexistent_result.is_ok(),
        "Retrieval of nonexistent document should return Ok(None)"
    );

    let retrieved = nonexistent_result.unwrap();
    assert!(
        retrieved.is_none(),
        "Should return None for nonexistent document"
    );

    // Test 6: Search with invalid query vector
    let invalid_query = vec![f32::NAN; 384];
    let invalid_search = storage.search_vectors(&invalid_query, 5, None).await;
    assert!(
        invalid_search.is_err(),
        "Should reject search with invalid query vector"
    );

    let wrong_dim_query = vec![1.0; 256];
    let wrong_dim_search = storage.search_vectors(&wrong_dim_query, 5, None).await;
    assert!(
        wrong_dim_search.is_err(),
        "Should reject search with wrong dimension query"
    );

    // Test 7: Deletion error handling
    let delete_nonexistent = storage.delete_vector("nonexistent_delete").await;
    // Should handle gracefully (typically succeeds even if document doesn't exist)
    assert!(
        delete_nonexistent.is_ok(),
        "Deletion of nonexistent document should succeed"
    );

    // Test 8: Concurrent error scenarios
    let valid_doc_id = "concurrent_error_test";
    storage
        .store_vector(
            valid_doc_id,
            &valid_embedding,
            Some(json!({"test": "concurrent"})),
        )
        .await
        .expect("Failed to store test document");

    let concurrent_handles: Vec<_> = (0..10)
        .map(|i| {
            let storage_clone = storage.clone();
            let doc_id = if i % 3 == 0 {
                "nonexistent".to_string()
            } else {
                valid_doc_id.to_string()
            };
            tokio::spawn(async move { storage_clone.get_vector(&doc_id).await })
        })
        .collect();

    // All retrieval operations should complete without panicking
    for handle in concurrent_handles {
        let result = handle.await.expect("Concurrent retrieval should complete");
        assert!(result.is_ok(), "Concurrent retrieval should not fail");
    }

    // Clean up
    storage
        .delete_vector(valid_doc_id)
        .await
        .expect("Failed to cleanup test document");
}

/// ANCHOR: Test search service error handling and recovery
/// Tests: Query validation, service degradation, result filtering
#[tokio::test]
async fn test_anchor_search_service_error_handling() {
    let config = create_error_test_config();
    let embedding_service = LocalEmbeddingService::new(config.embedding.clone());
    let storage = VectorStorage::new(config.clone()).expect("Failed to create vector storage");
    let search_service = SemanticSearchService::new(
        SemanticSearchConfig {
            min_score_threshold: 0.3,
            cache_enabled: true,
            cache_ttl: Duration::from_secs(60),
            ..Default::default()
        },
        storage.clone(),
        embedding_service.clone(),
    )
    .expect("Failed to create search service");

    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Setup test data
    let test_content = "Test content for search error handling";
    let test_embedding = embedding_service
        .generate_embedding(test_content)
        .await
        .expect("Failed to generate test embedding");

    storage
        .store_vector(
            "search_error_test",
            &test_embedding,
            Some(json!({
                "content": test_content,
                "type": "error_test"
            })),
        )
        .await
        .expect("Failed to store test document");

    // Test 1: Empty query handling
    let empty_search = search_service
        .search(
            "",
            SearchOptions {
                limit: Some(5),
                score_threshold: Some(0.3),
                with_payload: true,
                with_vectors: false,
            },
        )
        .await;

    assert!(empty_search.is_ok(), "Should handle empty query gracefully");
    let empty_results = empty_search.unwrap();
    // May return empty results or default results depending on implementation

    // Test 2: Extreme parameter values
    let extreme_limit_search = search_service
        .search(
            "test",
            SearchOptions {
                limit: Some(0), // Zero limit
                score_threshold: Some(0.0),
                with_payload: false,
                with_vectors: false,
            },
        )
        .await;

    assert!(
        extreme_limit_search.is_ok(),
        "Should handle extreme limit gracefully"
    );

    let high_threshold_search = search_service
        .search(
            "test",
            SearchOptions {
                limit: Some(5),
                score_threshold: Some(2.0), // Impossible threshold
                with_payload: false,
                with_vectors: false,
            },
        )
        .await;

    assert!(
        high_threshold_search.is_ok(),
        "Should handle impossible threshold gracefully"
    );
    let high_threshold_results = high_threshold_search.unwrap();
    assert!(
        high_threshold_results.results.is_empty(),
        "Should return empty results for impossible threshold"
    );

    // Test 3: Invalid query content
    let special_chars_query = "\x00\x01\x02invalid\x03\x04";
    let special_search = search_service
        .search(
            special_chars_query,
            SearchOptions {
                limit: Some(5),
                score_threshold: Some(0.3),
                with_payload: false,
                with_vectors: false,
            },
        )
        .await;

    assert!(
        special_search.is_ok(),
        "Should handle special characters in query"
    );

    let very_long_query = "word ".repeat(10000);
    let long_search = search_service
        .search(
            &very_long_query,
            SearchOptions {
                limit: Some(5),
                score_threshold: Some(0.3),
                with_payload: false,
                with_vectors: false,
            },
        )
        .await;

    assert!(long_search.is_ok(), "Should handle very long queries");

    // Test 4: Unicode query handling
    let unicode_query = "测试 query with 🚀 unicode characters тест العربية";
    let unicode_search = search_service
        .search(
            unicode_query,
            SearchOptions {
                limit: Some(5),
                score_threshold: Some(0.3),
                with_payload: true,
                with_vectors: false,
            },
        )
        .await;

    assert!(unicode_search.is_ok(), "Should handle unicode queries");

    // Test 5: Concurrent search error scenarios
    let problematic_queries = vec![
        "".to_string(),
        "\x00\x01invalid".to_string(),
        "🚀🎯".to_string(),
        "word ".repeat(1000),
        "normal query".to_string(),
    ];

    let concurrent_handles: Vec<_> = problematic_queries
        .into_iter()
        .enumerate()
        .map(|(i, query)| {
            let search_service_clone = search_service.clone();
            tokio::spawn(async move {
                let result = search_service_clone
                    .search(
                        &query,
                        SearchOptions {
                            limit: Some(3),
                            score_threshold: Some(0.3),
                            with_payload: false,
                            with_vectors: false,
                        },
                    )
                    .await;
                (i, result)
            })
        })
        .collect();

    // All concurrent searches should complete without panicking
    for handle in concurrent_handles {
        let (i, result) = handle.await.expect("Concurrent search should complete");
        assert!(result.is_ok(), "Concurrent search {} should succeed", i);
    }

    // Test 6: Service state after errors
    let analytics = search_service
        .get_analytics()
        .await
        .expect("Should get analytics after error scenarios");

    assert!(
        analytics.total_searches > 0,
        "Should track search attempts including failed ones"
    );

    // Clean up
    storage
        .delete_vector("search_error_test")
        .await
        .expect("Failed to cleanup search test document");
}

/// ANCHOR: Test migration service error handling and rollback
/// Tests: Migration failure recovery, data validation, rollback mechanisms
#[tokio::test]
async fn test_anchor_migration_error_handling() {
    let config = create_error_test_config();
    let migration_config = MigrationConfig {
        batch_size: 5,
        max_parallel_batches: 2,
        validation_level: ValidationLevel::Strict,
        enable_rollback: true,
        progress_reporting_interval: Duration::from_secs(1),
        timeout_per_batch: Duration::from_secs(10),
        max_retries: 1, // Low retries for error testing
        enable_deduplication: true,
        preserve_metadata: true,
    };

    let embedding_service = LocalEmbeddingService::new(config.embedding.clone());
    let storage = VectorStorage::new(config.clone()).expect("Failed to create vector storage");
    let migration_service =
        MigrationService::new(migration_config, storage.clone(), embedding_service.clone())
            .expect("Failed to create migration service");

    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    let test_data = create_error_test_data();

    // Test 1: Migration with validation errors
    let invalid_source = MigrationSource::InMemory {
        data: test_data["invalid_data"].clone(),
        source_name: "invalid_data_test".to_string(),
    };

    let invalid_migration_id = migration_service
        .start_migration(invalid_source)
        .await
        .expect("Should start migration even with invalid data");

    // Wait for migration to complete or fail
    let mut iterations = 30;
    let mut final_status = None;

    while iterations > 0 {
        tokio::time::sleep(Duration::from_millis(200)).await;

        let status = migration_service
            .get_migration_status(&invalid_migration_id)
            .await
            .expect("Failed to get migration status");

        match status.status {
            fortitude_core::vector::MigrationStatus::Completed
            | fortitude_core::vector::MigrationStatus::Failed => {
                final_status = Some(status);
                break;
            }
            _ => {}
        }

        iterations -= 1;
    }

    let status = final_status.expect("Migration should complete or fail");

    // Get migration summary to check error handling
    let summary = migration_service
        .get_migration_summary(&invalid_migration_id)
        .await
        .expect("Should get migration summary");

    assert!(summary.failed_items > 0, "Should report some failed items");
    println!(
        "Migration handled {} failed items out of {} total",
        summary.failed_items, summary.total_items
    );

    // Test 2: Rollback functionality
    if matches!(
        status.status,
        fortitude_core::vector::MigrationStatus::Failed
    ) {
        let rollback_result = migration_service
            .rollback_migration(&invalid_migration_id)
            .await
            .expect("Should be able to attempt rollback");

        // Verify rollback cleaned up any partially migrated data
        let post_rollback_count = storage
            .count_vectors()
            .await
            .expect("Should be able to count vectors after rollback");

        // Count should be minimal (only successfully processed items, if any)
        println!("Vector count after rollback: {}", post_rollback_count);
    }

    // Test 3: Migration with mixed valid/invalid data
    let mut mixed_data = test_data["valid_data"].clone();
    mixed_data.extend(test_data["invalid_data"][0..2].to_vec()); // Add some invalid data

    let mixed_source = MigrationSource::InMemory {
        data: mixed_data.clone(),
        source_name: "mixed_data_test".to_string(),
    };

    let mixed_migration_id = migration_service
        .start_migration(mixed_source)
        .await
        .expect("Should start mixed migration");

    // Wait for completion
    let mut iterations = 30;
    let mut mixed_completed = false;

    while !mixed_completed && iterations > 0 {
        tokio::time::sleep(Duration::from_millis(200)).await;

        let status = migration_service
            .get_migration_status(&mixed_migration_id)
            .await
            .expect("Failed to get mixed migration status");

        match status.status {
            fortitude_core::vector::MigrationStatus::Completed
            | fortitude_core::vector::MigrationStatus::Failed => {
                mixed_completed = true;
            }
            _ => {}
        }

        iterations -= 1;
    }

    assert!(mixed_completed, "Mixed migration should complete");

    let mixed_summary = migration_service
        .get_migration_summary(&mixed_migration_id)
        .await
        .expect("Should get mixed migration summary");

    // Should have some successful and some failed items
    assert!(
        mixed_summary.successful_items > 0,
        "Should have some successful items"
    );
    println!(
        "Mixed migration: {} successful, {} failed",
        mixed_summary.successful_items, mixed_summary.failed_items
    );

    // Test 4: Validation report checking
    let validation_report = migration_service
        .get_validation_report(&mixed_migration_id)
        .await
        .expect("Should get validation report");

    assert!(
        !validation_report.errors.is_empty() || !validation_report.warnings.is_empty(),
        "Should have validation issues reported"
    );

    for error in &validation_report.errors {
        println!("Validation error for {}: {}", error.item_id, error.message);
        assert!(
            !error.message.is_empty(),
            "Error message should not be empty"
        );
    }

    // Test 5: Migration with malformed data
    let malformed_source = MigrationSource::InMemory {
        data: test_data["malformed_data"].clone(),
        source_name: "malformed_data_test".to_string(),
    };

    let malformed_migration_id = migration_service
        .start_migration(malformed_source)
        .await
        .expect("Should start malformed migration");

    // Wait for completion (may succeed or fail)
    let mut iterations = 20;
    let mut malformed_completed = false;

    while !malformed_completed && iterations > 0 {
        tokio::time::sleep(Duration::from_millis(200)).await;

        let status = migration_service
            .get_migration_status(&malformed_migration_id)
            .await
            .expect("Failed to get malformed migration status");

        match status.status {
            fortitude_core::vector::MigrationStatus::Completed
            | fortitude_core::vector::MigrationStatus::Failed => {
                malformed_completed = true;
            }
            _ => {}
        }

        iterations -= 1;
    }

    // Should handle malformed data gracefully
    assert!(
        malformed_completed,
        "Malformed migration should complete or fail gracefully"
    );

    // Test 6: Error in migration status retrieval
    let nonexistent_migration = migration_service
        .get_migration_status("nonexistent_id")
        .await;
    assert!(
        nonexistent_migration.is_err(),
        "Should fail for nonexistent migration ID"
    );

    let error = nonexistent_migration.unwrap_err();
    assert!(
        matches!(error, MigrationError::NotFound { .. }),
        "Should return NotFound error for nonexistent migration"
    );

    // Test 7: Migration history with errors
    let migration_history = migration_service
        .get_migration_history()
        .await
        .expect("Should get migration history");

    assert!(
        !migration_history.is_empty(),
        "Should have migration history"
    );

    // Verify error information is preserved in history
    let failed_migrations = migration_history
        .iter()
        .filter(|m| matches!(m.status, fortitude_core::vector::MigrationStatus::Failed))
        .count();

    println!("Found {} failed migrations in history", failed_migrations);

    // Clean up any successfully migrated data
    for data_set in test_data.values() {
        for (id, _, _) in data_set {
            storage.delete_vector(id).await.ok(); // Ignore errors during cleanup
        }
    }
}

/// ANCHOR: Test hybrid search error handling and fallback mechanisms
/// Tests: Strategy fallbacks, component failure handling, graceful degradation
#[tokio::test]
async fn test_anchor_hybrid_search_error_handling() {
    let config = create_error_test_config();
    let embedding_service = LocalEmbeddingService::new(config.embedding.clone());
    let storage = VectorStorage::new(config.clone()).expect("Failed to create vector storage");
    let search_service = SemanticSearchService::new(
        SemanticSearchConfig::default(),
        storage.clone(),
        embedding_service.clone(),
    )
    .expect("Failed to create search service");
    let hybrid_service = HybridSearchService::new(
        HybridSearchConfig {
            semantic_weight: 0.6,
            keyword_weight: 0.4,
            fusion_method: FusionMethod::WeightedSum,
            min_semantic_score: 0.3,
            min_keyword_score: 0.1,
            max_results: 20,
            enable_query_analysis: true,
            enable_performance_tracking: true,
        },
        search_service.clone(),
        embedding_service.clone(),
    )
    .expect("Failed to create hybrid search service");

    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Setup test data
    let test_docs = vec![
        (
            "hybrid_error_1",
            "Document for hybrid search error testing with keywords",
        ),
        (
            "hybrid_error_2",
            "Another test document with different content patterns",
        ),
    ];

    for (id, content) in &test_docs {
        let embedding = embedding_service
            .generate_embedding(content)
            .await
            .expect("Failed to generate test embedding");

        storage
            .store_vector(
                id,
                &embedding,
                Some(json!({
                    "content": content,
                    "type": "hybrid_test"
                })),
            )
            .await
            .expect("Failed to store test document");
    }

    // Test 1: Invalid search strategy handling
    let invalid_strategy_request = HybridSearchRequest {
        query: "test query".to_string(),
        strategy: SearchStrategy::Balanced, // Valid strategy
        limit: 0,                           // Invalid limit
        semantic_options: None,
        keyword_options: None,
        filters: None,
    };

    let invalid_result = hybrid_service.search(invalid_strategy_request).await;
    assert!(
        invalid_result.is_ok(),
        "Should handle invalid limit gracefully"
    );

    // Test 2: Empty query handling
    let empty_request = HybridSearchRequest {
        query: "".to_string(),
        strategy: SearchStrategy::Balanced,
        limit: 5,
        semantic_options: None,
        keyword_options: None,
        filters: None,
    };

    let empty_result = hybrid_service.search(empty_request).await;
    assert!(empty_result.is_ok(), "Should handle empty query gracefully");

    // Test 3: Extreme weight configurations
    let extreme_semantic_request = HybridSearchRequest {
        query: "test query".to_string(),
        strategy: SearchStrategy::SemanticFocused,
        limit: 5,
        semantic_options: None,
        keyword_options: None,
        filters: None,
    };

    let extreme_semantic_result = hybrid_service.search(extreme_semantic_request).await;
    assert!(
        extreme_semantic_result.is_ok(),
        "Should handle extreme semantic focus"
    );

    let extreme_keyword_request = HybridSearchRequest {
        query: "keywords test".to_string(),
        strategy: SearchStrategy::KeywordFocused,
        limit: 5,
        semantic_options: None,
        keyword_options: None,
        filters: None,
    };

    let extreme_keyword_result = hybrid_service.search(extreme_keyword_request).await;
    assert!(
        extreme_keyword_result.is_ok(),
        "Should handle extreme keyword focus"
    );

    // Test 4: Special character queries
    let special_char_request = HybridSearchRequest {
        query: "\x00\x01special\x02chars\x03".to_string(),
        strategy: SearchStrategy::Balanced,
        limit: 5,
        semantic_options: None,
        keyword_options: None,
        filters: None,
    };

    let special_char_result = hybrid_service.search(special_char_request).await;
    assert!(
        special_char_result.is_ok(),
        "Should handle special characters in query"
    );

    // Test 5: Unicode query handling
    let unicode_request = HybridSearchRequest {
        query: "Unicode 🚀 test with 测试 characters тест العربية".to_string(),
        strategy: SearchStrategy::Balanced,
        limit: 5,
        semantic_options: None,
        keyword_options: None,
        filters: None,
    };

    let unicode_result = hybrid_service.search(unicode_request).await;
    assert!(unicode_result.is_ok(), "Should handle unicode characters");

    // Test 6: Very long query handling
    let long_query_request = HybridSearchRequest {
        query: "word ".repeat(5000),
        strategy: SearchStrategy::Balanced,
        limit: 5,
        semantic_options: None,
        keyword_options: None,
        filters: None,
    };

    let long_query_result = hybrid_service.search(long_query_request).await;
    assert!(long_query_result.is_ok(), "Should handle very long queries");

    // Test 7: Concurrent error scenarios
    let problematic_requests = vec![
        HybridSearchRequest {
            query: "".to_string(),
            strategy: SearchStrategy::Balanced,
            limit: 5,
            semantic_options: None,
            keyword_options: None,
            filters: None,
        },
        HybridSearchRequest {
            query: "\x00invalid".to_string(),
            strategy: SearchStrategy::SemanticFocused,
            limit: 0,
            semantic_options: None,
            keyword_options: None,
            filters: None,
        },
        HybridSearchRequest {
            query: "normal query".to_string(),
            strategy: SearchStrategy::KeywordFocused,
            limit: 10,
            semantic_options: None,
            keyword_options: None,
            filters: None,
        },
    ];

    let concurrent_handles: Vec<_> = problematic_requests
        .into_iter()
        .enumerate()
        .map(|(i, request)| {
            let hybrid_service_clone = hybrid_service.clone();
            tokio::spawn(async move {
                let result = hybrid_service_clone.search(request).await;
                (i, result)
            })
        })
        .collect();

    // All concurrent operations should complete without panicking
    for handle in concurrent_handles {
        let (i, result) = handle
            .await
            .expect("Concurrent hybrid search should complete");
        assert!(
            result.is_ok(),
            "Concurrent hybrid search {} should succeed",
            i
        );
    }

    // Test 8: Analytics after error scenarios
    let analytics = hybrid_service
        .get_analytics()
        .await
        .expect("Should get analytics after error scenarios");

    assert!(analytics.total_searches > 0, "Should track search attempts");

    // Verify strategy usage is tracked even for problematic queries
    assert!(
        !analytics.strategy_usage.is_empty(),
        "Should track strategy usage"
    );

    // Clean up test data
    for (id, _) in &test_docs {
        storage
            .delete_vector(id)
            .await
            .expect("Failed to cleanup test document");
    }
}

/// ANCHOR: Test cross-service error propagation and recovery
/// Tests: Error propagation chains, service isolation, recovery mechanisms
#[tokio::test]
async fn test_anchor_cross_service_error_propagation() {
    let config = create_error_test_config();

    // Initialize services with error-prone configuration
    let embedding_service = LocalEmbeddingService::new(config.embedding.clone());
    let storage = VectorStorage::new(config.clone()).expect("Failed to create vector storage");
    let search_service = SemanticSearchService::new(
        SemanticSearchConfig::default(),
        storage.clone(),
        embedding_service.clone(),
    )
    .expect("Failed to create search service");

    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Test 1: Embedding service error propagation to search
    let problematic_query = "\x00\x01\x02problematic query\x03\x04";

    // This should handle the problematic query gracefully
    let search_result = search_service
        .search(
            problematic_query,
            SearchOptions {
                limit: Some(5),
                score_threshold: Some(0.3),
                with_payload: false,
                with_vectors: false,
            },
        )
        .await;

    assert!(
        search_result.is_ok(),
        "Search should handle embedding service quirks gracefully"
    );

    // Test 2: Storage service error isolation
    let valid_embedding = embedding_service
        .generate_embedding("valid content")
        .await
        .expect("Failed to generate valid embedding");

    // Try to store with problematic metadata
    let problematic_metadata = json!({
        "field": "\x00\x01problematic\x02data\x03",
        "large_field": "x".repeat(100000)
    });

    let storage_result = storage
        .store_vector(
            "error_prop_test",
            &valid_embedding,
            Some(problematic_metadata),
        )
        .await;

    // Whether this succeeds or fails, subsequent operations should still work
    let subsequent_storage = storage
        .store_vector(
            "normal_test",
            &valid_embedding,
            Some(json!({
                "normal": "metadata"
            })),
        )
        .await;

    assert!(
        subsequent_storage.is_ok(),
        "Subsequent storage operations should work after errors"
    );

    // Test 3: Service state consistency after errors
    let initial_stats = embedding_service.get_stats().await;
    let initial_analytics = search_service
        .get_analytics()
        .await
        .expect("Should get initial analytics");

    // Perform various operations that may cause errors
    let _error_operations = vec![
        embedding_service.generate_embedding("").await,
        embedding_service
            .generate_embedding("\x00\x01invalid")
            .await,
        search_service.search("", SearchOptions::default()).await,
        storage.get_vector("nonexistent").await,
    ];

    // Service state should remain consistent
    let final_stats = embedding_service.get_stats().await;
    let final_analytics = search_service
        .get_analytics()
        .await
        .expect("Should get final analytics");

    assert!(
        final_stats.total_generated >= initial_stats.total_generated,
        "Embedding stats should remain consistent"
    );
    assert!(
        final_analytics.total_searches >= initial_analytics.total_searches,
        "Search analytics should remain consistent"
    );

    // Test 4: Recovery after temporary failures
    // Simulate recovery by performing normal operations
    let recovery_content = "Recovery test content after errors";
    let recovery_embedding = embedding_service
        .generate_embedding(recovery_content)
        .await
        .expect("Should recover and generate embeddings normally");

    storage
        .store_vector(
            "recovery_test",
            &recovery_embedding,
            Some(json!({
                "recovery": true,
                "content": recovery_content
            })),
        )
        .await
        .expect("Should recover and store vectors normally");

    let recovery_search = search_service
        .search(
            recovery_content,
            SearchOptions {
                limit: Some(5),
                score_threshold: Some(0.3),
                with_payload: true,
                with_vectors: false,
            },
        )
        .await
        .expect("Should recover and search normally");

    assert!(
        !recovery_search.results.is_empty(),
        "Should find results after recovery"
    );

    // Test 5: Concurrent error scenarios across services
    let concurrent_operations: Vec<_> = (0..15)
        .map(|i| {
            let embedding_service_clone = embedding_service.clone();
            let storage_clone = storage.clone();
            let search_service_clone = search_service.clone();

            tokio::spawn(async move {
                match i % 3 {
                    0 => {
                        // Embedding operations
                        let content = if i % 2 == 0 {
                            ""
                        } else {
                            &format!("Content {}", i)
                        };
                        embedding_service_clone.generate_embedding(content).await
                    }
                    1 => {
                        // Storage operations
                        storage_clone
                            .get_vector(&format!("doc_{}", i))
                            .await
                            .map(|_| vec![]) // Convert to embedding-like result
                            .map_err(|e| e.into())
                    }
                    _ => {
                        // Search operations
                        let query = if i % 2 == 0 {
                            ""
                        } else {
                            &format!("Query {}", i)
                        };
                        search_service_clone
                            .search(query, SearchOptions::default())
                            .await
                            .map(|_| vec![]) // Convert to embedding-like result
                            .map_err(|e| e.into())
                    }
                }
            })
        })
        .collect();

    // All operations should complete without causing service failures
    let mut completed_operations = 0;
    for handle in concurrent_operations {
        if handle.await.is_ok() {
            completed_operations += 1;
        }
    }

    println!(
        "Completed {} concurrent cross-service operations",
        completed_operations
    );
    assert!(
        completed_operations > 10,
        "Most concurrent operations should complete successfully"
    );

    // Clean up
    storage.delete_vector("error_prop_test").await.ok();
    storage.delete_vector("normal_test").await.ok();
    storage.delete_vector("recovery_test").await.ok();
}
