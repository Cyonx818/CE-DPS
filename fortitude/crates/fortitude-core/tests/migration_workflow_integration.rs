//! Integration tests for vector database migration workflows.
//! These tests verify end-to-end migration processes from various data sources.

use fortitude_core::vector::{
    CacheKeyStrategy, ConnectionPoolConfig, DataConverter, DeviceType, DistanceMetric,
    EmbeddingCacheConfig, EmbeddingConfig, EmbeddingGenerator, HealthCheckConfig,
    LocalEmbeddingService, MigrationConfig, MigrationService, MigrationSource, MigrationStatus,
    ValidationLevel, VectorConfig, VectorStorage, VectorStorageService,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Create test configuration for migration workflows
fn create_test_migration_config() -> (VectorConfig, MigrationConfig) {
    let vector_config = VectorConfig {
        url: "http://localhost:6334".to_string(),
        api_key: None,
        timeout: Duration::from_secs(30),
        default_collection: "test_migration_target".to_string(),
        vector_dimensions: 384,
        distance_metric: DistanceMetric::Cosine,
        health_check: HealthCheckConfig {
            enabled: true,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            max_failures: 3,
        },
        connection_pool: ConnectionPoolConfig {
            max_connections: 10,
            connection_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(600),
        },
        embedding: EmbeddingConfig {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            max_sequence_length: 256,
            batch_size: 16,
            device: DeviceType::Cpu,
            cache_config: EmbeddingCacheConfig {
                enabled: true,
                max_entries: 500,
                ttl: Duration::from_secs(1800),
                key_strategy: CacheKeyStrategy::Hash,
            },
            ..Default::default()
        },
    };

    let migration_config = MigrationConfig {
        batch_size: 10,
        max_workers: 3,
        validation_level: ValidationLevel::Comprehensive,
        enable_resume: true,
        max_retries: 3,
        retry_delay_ms: 1000,
        dry_run: false,
        custom_metadata: std::collections::HashMap::new(),
    };

    (vector_config, migration_config)
}

/// Create sample data sources for migration testing
fn create_sample_data_sources() -> HashMap<String, Vec<(String, String, Value)>> {
    let mut sources = HashMap::new();

    // Research documents source
    sources.insert("research_docs".to_string(), vec![
        (
            "research_1".to_string(),
            "Advanced machine learning techniques for natural language processing and text classification".to_string(),
            json!({
                "category": "machine_learning",
                "author": "Dr. Smith",
                "published": "2024-01-15",
                "tags": ["ml", "nlp", "classification"]
            })
        ),
        (
            "research_2".to_string(),
            "Distributed systems design patterns for scalable microservices architecture".to_string(),
            json!({
                "category": "systems",
                "author": "Jane Doe",
                "published": "2024-02-20",
                "tags": ["distributed", "microservices", "architecture"]
            })
        ),
        (
            "research_3".to_string(),
            "Database optimization strategies for high-performance applications with large datasets".to_string(),
            json!({
                "category": "database",
                "author": "Bob Johnson",
                "published": "2024-03-10",
                "tags": ["database", "performance", "optimization"]
            })
        ),
    ]);

    // Knowledge base source
    sources.insert("knowledge_base".to_string(), vec![
        (
            "kb_1".to_string(),
            "Rust programming language memory safety guarantees through ownership and borrowing system".to_string(),
            json!({
                "type": "programming_concept",
                "difficulty": "intermediate",
                "language": "rust",
                "last_updated": "2024-04-01"
            })
        ),
        (
            "kb_2".to_string(),
            "RESTful API design principles and best practices for maintainable web services".to_string(),
            json!({
                "type": "design_pattern",
                "difficulty": "beginner",
                "domain": "web_development",
                "last_updated": "2024-04-05"
            })
        ),
    ]);

    // Legacy data source with inconsistent format
    sources.insert(
        "legacy_data".to_string(),
        vec![
            (
                "legacy_1".to_string(),
                "Old format: Software testing methodologies".to_string(),
                json!({
                    "old_category": "testing",
                    "creation_date": "2023-12-01",
                    "format_version": "1.0"
                }),
            ),
            (
                "legacy_2".to_string(),
                "Old format: Agile development practices".to_string(),
                json!({
                    "old_category": "methodology",
                    "creation_date": "2023-11-15",
                    "format_version": "1.0"
                }),
            ),
        ],
    );

    sources
}

/// ANCHOR: Test complete migration workflow from start to finish
/// Tests: Migration initialization, batch processing, validation, completion
#[tokio::test]
async fn test_anchor_complete_migration_workflow() {
    let (vector_config, migration_config) = create_test_migration_config();
    let data_sources = create_sample_data_sources();

    // Initialize services
    let embedding_service = Arc::new(LocalEmbeddingService::new(vector_config.embedding.clone()));
    let target_storage =
        VectorStorage::new(vector_config.clone()).expect("Failed to create target storage");
    let migration_service = MigrationService::new(
        migration_config.clone(),
        target_storage.clone(),
        embedding_service.clone(),
    )
    .expect("Failed to create migration service");

    // Initialize embedding service
    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Verify initial state
    let initial_count = target_storage
        .count_vectors()
        .await
        .expect("Failed to count initial vectors");
    assert_eq!(initial_count, 0, "Target storage should start empty");

    // Test migration from research documents
    let research_source = MigrationSource::InMemory {
        data: data_sources["research_docs"].clone(),
        source_name: "research_documents".to_string(),
    };

    // Start migration
    let migration_id = migration_service
        .start_migration(research_source)
        .await
        .expect("Failed to start migration");

    // Monitor migration progress
    let mut completed = false;
    let mut max_iterations = 30; // Prevent infinite loop

    while !completed && max_iterations > 0 {
        tokio::time::sleep(Duration::from_millis(100)).await;

        let status = migration_service
            .get_migration_status(&migration_id)
            .await
            .expect("Failed to get migration status");

        match status.status {
            MigrationStatus::Completed => {
                completed = true;
                println!("Migration completed successfully");
            }
            MigrationStatus::Failed => {
                panic!("Migration failed: {:?}", status);
            }
            MigrationStatus::InProgress => {
                println!(
                    "Migration in progress: {} / {} items",
                    status.progress.items_processed, status.progress.total_items
                );
            }
            _ => {}
        }

        max_iterations -= 1;
    }

    assert!(completed, "Migration should complete successfully");

    // Verify migration results
    let final_count = target_storage
        .count_vectors()
        .await
        .expect("Failed to count final vectors");
    assert_eq!(final_count, 3, "Should have migrated 3 research documents");

    // Verify data integrity
    let migrated_doc = target_storage
        .get_vector("research_1")
        .await
        .expect("Failed to retrieve migrated document");
    assert!(migrated_doc.is_some(), "Migrated document should exist");

    let (vector, metadata) = migrated_doc.unwrap();
    assert_eq!(vector.len(), 384, "Vector should have correct dimensions");
    assert!(metadata.is_some(), "Metadata should be preserved");

    let metadata = metadata.unwrap();
    assert_eq!(
        metadata["category"], "machine_learning",
        "Metadata should be preserved"
    );
    assert!(
        metadata.get("migrated_at").is_some(),
        "Should add migration timestamp"
    );

    // Get migration summary
    let summary = migration_service
        .get_migration_summary(&migration_id)
        .await
        .expect("Failed to get migration summary");

    assert_eq!(summary.total_items, 3, "Should report correct total items");
    assert_eq!(
        summary.successful_items, 3,
        "All items should migrate successfully"
    );
    assert_eq!(summary.failed_items, 0, "No items should fail");
    assert!(
        summary.total_duration > Duration::from_millis(0),
        "Should track duration"
    );

    // Clean up migration data
    for doc in &data_sources["research_docs"] {
        target_storage
            .delete_vector(&doc.0)
            .await
            .expect("Failed to cleanup migrated data");
    }
}

/// ANCHOR: Test migration with validation and error handling
/// Tests: Data validation, error recovery, partial migration handling
#[tokio::test]
async fn test_anchor_migration_validation_and_error_handling() {
    let (vector_config, mut migration_config) = create_test_migration_config();
    migration_config.validation_level = ValidationLevel::Comprehensive;

    // Initialize services
    let embedding_service = Arc::new(LocalEmbeddingService::new(vector_config.embedding.clone()));
    let target_storage =
        VectorStorage::new(vector_config.clone()).expect("Failed to create target storage");
    let migration_service = MigrationService::new(
        migration_config,
        target_storage.clone(),
        embedding_service.clone(),
    )
    .expect("Failed to create migration service");

    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Create test data with validation issues
    let problematic_data = vec![
        (
            "valid_doc".to_string(),
            "This is a valid document with proper content".to_string(),
            json!({
                "category": "valid",
                "quality_score": 0.9
            }),
        ),
        (
            "empty_content".to_string(),
            "".to_string(), // Empty content
            json!({
                "category": "invalid",
                "quality_score": 0.1
            }),
        ),
        (
            "invalid_metadata".to_string(),
            "Content with invalid metadata structure".to_string(),
            json!(null), // Invalid metadata
        ),
        (
            "duplicate_id".to_string(),
            "First document with this ID".to_string(),
            json!({
                "category": "duplicate_test"
            }),
        ),
        (
            "duplicate_id".to_string(), // Duplicate ID
            "Second document with same ID".to_string(),
            json!({
                "category": "duplicate_test"
            }),
        ),
    ];

    let source = MigrationSource::InMemory {
        data: problematic_data,
        source_name: "problematic_data".to_string(),
    };

    // Start migration with validation
    let migration_id = migration_service
        .start_migration(source)
        .await
        .expect("Failed to start migration");

    // Wait for migration to complete
    let mut iterations = 30;
    let mut final_status = None;

    while iterations > 0 {
        tokio::time::sleep(Duration::from_millis(100)).await;

        let status = migration_service
            .get_migration_status(&migration_id)
            .await
            .expect("Failed to get migration status");

        if matches!(
            status.status,
            MigrationStatus::Completed | MigrationStatus::Failed
        ) {
            final_status = Some(status);
            break;
        }

        iterations -= 1;
    }

    let status = final_status.expect("Migration should complete or fail");

    // Verify validation results
    let summary = migration_service
        .get_migration_summary(&migration_id)
        .await
        .expect("Failed to get migration summary");

    assert!(
        summary.successful_items < summary.total_items,
        "Some items should fail validation"
    );
    assert!(
        summary.failed_items > 0,
        "Should report validation failures"
    );

    // Verify that valid documents were migrated
    let valid_doc = target_storage
        .get_vector("valid_doc")
        .await
        .expect("Failed to check for valid document");
    assert!(valid_doc.is_some(), "Valid document should be migrated");

    // Verify that invalid documents were not migrated
    let empty_doc = target_storage
        .get_vector("empty_content")
        .await
        .expect("Failed to check for empty document");
    assert!(empty_doc.is_none(), "Empty content should not be migrated");

    // Test validation report
    let validation_report = migration_service
        .get_validation_report(&migration_id)
        .await
        .expect("Failed to get validation report");

    assert!(
        !validation_report.errors.is_empty(),
        "Should report validation errors"
    );
    assert!(validation_report.warnings.len() >= 0, "May have warnings");

    // Verify error details
    let has_empty_content_error = validation_report
        .errors
        .iter()
        .any(|error| error.item_id == "empty_content" && error.message.contains("empty"));
    assert!(has_empty_content_error, "Should report empty content error");

    let has_duplicate_error = validation_report
        .errors
        .iter()
        .any(|error| error.item_id == "duplicate_id" && error.message.contains("duplicate"));
    assert!(has_duplicate_error, "Should report duplicate ID error");

    // Clean up any successfully migrated data
    if target_storage
        .get_vector("valid_doc")
        .await
        .unwrap()
        .is_some()
    {
        target_storage
            .delete_vector("valid_doc")
            .await
            .expect("Failed to cleanup valid document");
    }
}

/// ANCHOR: Test migration with data transformation and format conversion
/// Tests: Legacy data conversion, metadata transformation, format normalization
#[tokio::test]
async fn test_anchor_migration_data_transformation() {
    let (vector_config, migration_config) = create_test_migration_config();
    let data_sources = create_sample_data_sources();

    // Initialize services
    let embedding_service = Arc::new(LocalEmbeddingService::new(vector_config.embedding.clone()));
    let target_storage =
        VectorStorage::new(vector_config.clone()).expect("Failed to create target storage");

    // Create data converter for legacy format transformation
    let data_converter = DataConverter::new()
        .with_content_transformer(|content: &str| {
            // Remove "Old format: " prefix from legacy content
            if content.starts_with("Old format: ") {
                content.replace("Old format: ", "Modernized: ")
            } else {
                content.to_string()
            }
        })
        .with_metadata_transformer(|metadata: &Value| {
            let mut transformed = metadata.clone();

            // Transform legacy metadata structure
            if let Some(old_category) = metadata.get("old_category") {
                transformed["category"] = old_category.clone();
                if let Some(obj) = transformed.as_object_mut() {
                    obj.remove("old_category");
                }
            }

            // Add transformation metadata
            if let Some(obj) = transformed.as_object_mut() {
                obj.insert("transformed".to_string(), json!(true));
                obj.insert(
                    "transformation_date".to_string(),
                    json!(chrono::Utc::now().to_rfc3339()),
                );
            }

            transformed
        })
        .expect("Failed to create data converter");

    let migration_service = MigrationService::with_converter(
        migration_config,
        target_storage.clone(),
        embedding_service.clone(),
        data_converter,
    )
    .expect("Failed to create migration service with converter");

    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Test migration of legacy data with transformation
    let legacy_source = MigrationSource::InMemory {
        data: data_sources["legacy_data"].clone(),
        source_name: "legacy_system".to_string(),
    };

    let migration_id = migration_service
        .start_migration(legacy_source)
        .await
        .expect("Failed to start legacy migration");

    // Wait for migration completion
    let mut completed = false;
    let mut iterations = 30;

    while !completed && iterations > 0 {
        tokio::time::sleep(Duration::from_millis(100)).await;

        let status = migration_service
            .get_migration_status(&migration_id)
            .await
            .expect("Failed to get migration status");

        if matches!(status.status, MigrationStatus::Completed) {
            completed = true;
        } else if matches!(status.status, MigrationStatus::Failed) {
            panic!("Migration failed: {:?}", status);
        }

        iterations -= 1;
    }

    assert!(completed, "Legacy migration should complete");

    // Verify data transformation
    let transformed_doc = target_storage
        .get_vector("legacy_1")
        .await
        .expect("Failed to retrieve transformed document");
    assert!(
        transformed_doc.is_some(),
        "Transformed document should exist"
    );

    let (vector, metadata) = transformed_doc.unwrap();
    assert_eq!(vector.len(), 384, "Vector should have correct dimensions");

    let metadata = metadata.unwrap();

    // Verify content transformation (would need to check original content)
    // For now, verify metadata transformation
    assert_eq!(
        metadata["category"], "testing",
        "Should transform old_category to category"
    );
    assert!(
        metadata.get("old_category").is_none(),
        "Should remove old_category field"
    );
    assert_eq!(metadata["transformed"], true, "Should mark as transformed");
    assert!(
        metadata.get("transformation_date").is_some(),
        "Should add transformation date"
    );

    // Test batch migration with mixed sources
    let mixed_sources = vec![
        MigrationSource::InMemory {
            data: data_sources["knowledge_base"].clone(),
            source_name: "kb_source".to_string(),
        },
        MigrationSource::InMemory {
            data: data_sources["research_docs"].clone(),
            source_name: "research_source".to_string(),
        },
    ];

    // Start multiple migrations
    let mut migration_ids = Vec::new();
    for source in mixed_sources {
        let id = migration_service
            .start_migration(source)
            .await
            .expect("Failed to start batch migration");
        migration_ids.push(id);
    }

    // Wait for all migrations to complete
    for migration_id in &migration_ids {
        let mut completed = false;
        let mut iterations = 30;

        while !completed && iterations > 0 {
            tokio::time::sleep(Duration::from_millis(100)).await;

            let status = migration_service
                .get_migration_status(migration_id)
                .await
                .expect("Failed to get migration status");

            if matches!(status.status, MigrationStatus::Completed) {
                completed = true;
            } else if matches!(status.status, MigrationStatus::Failed) {
                panic!("Batch migration failed: {:?}", status);
            }

            iterations -= 1;
        }

        assert!(completed, "Batch migration should complete");
    }

    // Verify total migrated count
    let total_count = target_storage
        .count_vectors()
        .await
        .expect("Failed to count vectors after batch migration");

    // Should have: 2 legacy + 2 knowledge base + 3 research docs = 7 total
    assert_eq!(
        total_count, 7,
        "Should have migrated all documents from all sources"
    );

    // Clean up all migrated data
    let all_sources = [
        &data_sources["legacy_data"],
        &data_sources["knowledge_base"],
        &data_sources["research_docs"],
    ];
    for source in &all_sources {
        for (id, _, _) in source {
            target_storage
                .delete_vector(id)
                .await
                .expect("Failed to cleanup migrated data");
        }
    }
}

/// ANCHOR: Test migration rollback and recovery functionality
/// Tests: Migration failure handling, rollback operations, state recovery
#[tokio::test]
async fn test_anchor_migration_rollback_and_recovery() {
    let (vector_config, mut migration_config) = create_test_migration_config();
    migration_config.max_retries = 2;

    // Initialize services
    let embedding_service = Arc::new(LocalEmbeddingService::new(vector_config.embedding.clone()));
    let target_storage =
        VectorStorage::new(vector_config.clone()).expect("Failed to create target storage");
    let migration_service = MigrationService::new(
        migration_config,
        target_storage.clone(),
        embedding_service.clone(),
    )
    .expect("Failed to create migration service");

    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Store some existing data to verify rollback protection
    let existing_embedding = embedding_service
        .generate_embedding("Existing document content")
        .await
        .expect("Failed to generate existing embedding");
    let existing_metadata = json!({
        "type": "existing",
        "protected": true
    });

    target_storage
        .store_vector("existing_doc", &existing_embedding, Some(existing_metadata))
        .await
        .expect("Failed to store existing document");

    let initial_count = target_storage
        .count_vectors()
        .await
        .expect("Failed to count initial vectors");
    assert_eq!(initial_count, 1, "Should have one existing document");

    // Create test data that will partially succeed then fail
    let partially_failing_data = vec![
        (
            "success_1".to_string(),
            "This document will migrate successfully".to_string(),
            json!({
                "category": "success",
                "batch": 1
            }),
        ),
        (
            "success_2".to_string(),
            "Another successful document".to_string(),
            json!({
                "category": "success",
                "batch": 1
            }),
        ),
        // This would cause issues in a real scenario - simulating with problematic content
        (
            "problematic".to_string(),
            "x".repeat(10000), // Very long content that might cause issues
            json!({
                "category": "problematic",
                "batch": 2
            }),
        ),
    ];

    let source = MigrationSource::InMemory {
        data: partially_failing_data,
        source_name: "partial_failure_test".to_string(),
    };

    // Start migration
    let migration_id = migration_service
        .start_migration(source)
        .await
        .expect("Failed to start migration");

    // Wait for migration to complete or fail
    let mut final_status = None;
    let mut iterations = 40; // Give more time for retries

    while iterations > 0 {
        tokio::time::sleep(Duration::from_millis(150)).await;

        let status = migration_service
            .get_migration_status(&migration_id)
            .await
            .expect("Failed to get migration status");

        if matches!(
            status.status,
            MigrationStatus::Completed | MigrationStatus::Failed
        ) {
            final_status = Some(status);
            break;
        }

        iterations -= 1;
    }

    let final_status = final_status.expect("Migration should complete or fail");

    // Get migration summary
    let summary = migration_service
        .get_migration_summary(&migration_id)
        .await
        .expect("Failed to get migration summary");

    println!("Migration summary: {:?}", summary);

    // Test rollback if migration failed
    if matches!(final_status.status, MigrationStatus::Failed) {
        println!("Migration failed, testing rollback...");

        let rollback_result = migration_service
            .rollback_migration(&migration_id)
            .await
            .expect("Failed to perform rollback");

        assert!(rollback_result.is_success(), "Rollback should succeed");

        // Verify existing data is still present
        let existing_doc = target_storage
            .get_vector("existing_doc")
            .await
            .expect("Failed to check existing document");
        assert!(
            existing_doc.is_some(),
            "Existing document should remain after rollback"
        );

        // Verify migration data was removed
        let success_doc = target_storage
            .get_vector("success_1")
            .await
            .expect("Failed to check migrated document");
        assert!(
            success_doc.is_none(),
            "Migrated documents should be removed after rollback"
        );

        let post_rollback_count = target_storage
            .count_vectors()
            .await
            .expect("Failed to count vectors after rollback");
        assert_eq!(
            post_rollback_count, 1,
            "Should only have existing document after rollback"
        );
    } else {
        println!("Migration succeeded, testing partial success handling...");

        // If migration succeeded, verify that successful items are present
        let final_count = target_storage
            .count_vectors()
            .await
            .expect("Failed to count final vectors");
        assert!(
            final_count > 1,
            "Should have migrated at least some documents"
        );

        // Verify existing document is still present
        let existing_doc = target_storage
            .get_vector("existing_doc")
            .await
            .expect("Failed to check existing document");
        assert!(existing_doc.is_some(), "Existing document should remain");
    }

    // Test migration state recovery
    let recovered_status = migration_service
        .get_migration_status(&migration_id)
        .await
        .expect("Failed to recover migration status");

    assert!(
        matches!(
            recovered_status.status,
            MigrationStatus::Completed | MigrationStatus::Failed
        ),
        "Should maintain migration state"
    );

    // Verify migration history
    let migration_history = migration_service
        .get_migration_history()
        .await
        .expect("Failed to get migration history");

    assert!(
        !migration_history.is_empty(),
        "Should maintain migration history"
    );

    let our_migration = migration_history
        .iter()
        .find(|m| m.migration_id == migration_id);
    assert!(
        our_migration.is_some(),
        "Should find our migration in history"
    );

    // Clean up
    target_storage
        .delete_vector("existing_doc")
        .await
        .expect("Failed to cleanup existing document");

    // Clean up any remaining migrated documents
    for doc_id in ["success_1", "success_2", "problematic"] {
        if target_storage.get_vector(doc_id).await.unwrap().is_some() {
            target_storage
                .delete_vector(doc_id)
                .await
                .expect("Failed to cleanup migrated document");
        }
    }
}
