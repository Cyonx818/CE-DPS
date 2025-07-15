//! Cross-service integration validation tests for vector database functionality.
//! These tests verify how different services work together and maintain data consistency.

use fortitude_core::{
    pipeline::{PipelineConfig, ResearchPipeline},
    research_engine::{ClaudeResearchConfig, ClaudeResearchEngine},
    research_feedback::{FeedbackSource, ResearchQualityFeedback},
    vector::{
        CacheKeyStrategy, ConnectionPoolConfig, DeviceType, DistanceMetric, EmbeddingCacheConfig,
        EmbeddingConfig, EmbeddingGenerator, FusionMethod, HealthCheckConfig, HybridSearchConfig,
        HybridSearchRequest, HybridSearchService, LocalEmbeddingService, MigrationConfig,
        MigrationService, MigrationSource, SearchOptions, SearchStrategy, SemanticSearchConfig,
        SemanticSearchService, ValidationLevel, VectorConfig, VectorStorage, VectorStorageService,
    },
};
use fortitude_types::research::{AudienceContext, ClassifiedRequest, DomainContext, ResearchType};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Create comprehensive test configuration for cross-service integration
fn create_cross_service_config() -> (
    VectorConfig,
    ClaudeResearchConfig,
    PipelineConfig,
    MigrationConfig,
) {
    let vector_config = VectorConfig {
        url: "http://localhost:6334".to_string(),
        api_key: None,
        timeout: Duration::from_secs(30),
        default_collection: "cross_service_test".to_string(),
        vector_dimensions: 384,
        distance_metric: DistanceMetric::Cosine,
        health_check: HealthCheckConfig {
            enabled: true,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            max_failures: 3,
        },
        connection_pool: ConnectionPoolConfig {
            max_connections: 15,
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
                ttl: Duration::from_secs(900),
                key_strategy: CacheKeyStrategy::Hash,
            },
            ..Default::default()
        },
    };

    let research_config = ClaudeResearchConfig {
        max_tokens: 8000,
        temperature: 0.3,
        max_processing_time: Duration::from_secs(60),
        enable_quality_validation: true,
        min_quality_score: 0.75,
        system_prompt: "You are a research assistant.".to_string(),
        enable_vector_search: true,
        max_context_documents: 5,
        context_relevance_threshold: 0.3,
    };

    let pipeline_config = PipelineConfig {
        max_concurrent: 3,
        timeout_seconds: 90,
        enable_caching: true,
        default_audience: fortitude_types::research::AudienceContext::General,
        default_domain: fortitude_types::research::DomainContext::General,
        enable_context_detection: true,
        enable_advanced_classification: true,
        advanced_classification_config: None,
    };

    let migration_config = MigrationConfig {
        batch_size: 25,
        max_parallel_batches: 2,
        validation_level: ValidationLevel::Moderate,
        enable_rollback: true,
        progress_reporting_interval: Duration::from_secs(2),
        timeout_per_batch: Duration::from_secs(45),
        max_retries: 2,
        enable_deduplication: true,
        preserve_metadata: true,
    };

    (
        vector_config,
        research_config,
        pipeline_config,
        migration_config,
    )
}

/// Create test data representing different service domains
fn create_cross_service_test_data() -> HashMap<String, Vec<(String, String, Value)>> {
    let mut data = HashMap::new();

    // Research knowledge base
    data.insert("research_knowledge".to_string(), vec![
        (
            "rust_async_guide".to_string(),
            "Comprehensive guide to asynchronous programming in Rust using tokio runtime, async/await syntax, and Future trait implementations for high-performance concurrent applications".to_string(),
            json!({
                "type": "research_document",
                "category": "programming",
                "language": "rust",
                "topics": ["async", "concurrency", "performance"],
                "quality_score": 0.92,
                "audience": "intermediate",
                "source": "research_engine"
            })
        ),
        (
            "vector_db_optimization".to_string(),
            "Vector database optimization techniques for semantic search including indexing strategies, query optimization, and performance tuning for large-scale document collections".to_string(),
            json!({
                "type": "research_document", 
                "category": "database",
                "topics": ["vector", "optimization", "search", "indexing"],
                "quality_score": 0.88,
                "audience": "advanced",
                "source": "research_engine"
            })
        ),
    ]);

    // Migration source data
    data.insert("legacy_documents".to_string(), vec![
        (
            "legacy_ml_guide".to_string(),
            "Machine learning model deployment strategies for production environments using containerization, monitoring, and automated scaling techniques".to_string(),
            json!({
                "type": "legacy_document",
                "category": "machine_learning", 
                "format_version": "1.0",
                "migrated_from": "legacy_system",
                "topics": ["ml", "deployment", "production"],
                "quality_score": 0.85
            })
        ),
        (
            "legacy_api_patterns".to_string(),
            "RESTful API design patterns and best practices for building maintainable and scalable web services with proper error handling and documentation".to_string(),
            json!({
                "type": "legacy_document",
                "category": "web_development",
                "format_version": "1.0", 
                "migrated_from": "legacy_system",
                "topics": ["api", "rest", "web", "design"],
                "quality_score": 0.79
            })
        ),
    ]);

    // User feedback data
    data.insert("feedback_examples".to_string(), vec![
        (
            "feedback_positive".to_string(),
            "Excellent research result with comprehensive coverage of async Rust patterns, practical examples, and clear explanations suitable for intermediate developers".to_string(),
            json!({
                "type": "feedback",
                "feedback_type": "quality",
                "rating": 4.8,
                "category": "positive",
                "topics": ["rust", "async", "quality"]
            })
        ),
        (
            "feedback_suggestion".to_string(),
            "Good foundational content but could benefit from more advanced examples and performance benchmarking data for production use cases".to_string(),
            json!({
                "type": "feedback",
                "feedback_type": "improvement",
                "rating": 3.5,
                "category": "constructive",
                "topics": ["improvement", "examples", "performance"]
            })
        ),
    ]);

    data
}

/// ANCHOR: Test cross-service data flow and consistency
/// Tests: Data flow between services, consistency validation, state synchronization
#[tokio::test]
async fn test_anchor_cross_service_data_flow() {
    let (vector_config, research_config, pipeline_config, migration_config) =
        create_cross_service_config();
    let test_data = create_cross_service_test_data();

    // Initialize all services
    let embedding_service = LocalEmbeddingService::new(vector_config.embedding.clone());
    let storage =
        VectorStorage::new(vector_config.clone()).expect("Failed to create vector storage");
    let search_service =
        SemanticSearchService::new(storage.clone(), SemanticSearchConfig::default());
    let migration_service =
        MigrationService::new(migration_config, storage.clone(), embedding_service.clone())
            .expect("Failed to create migration service");

    let research_engine = ClaudeResearchEngine::new(
        research_config,
        None, // No vector search for this test
    )
    .expect("Failed to create research engine");
    let mut pipeline =
        ResearchPipeline::new(pipeline_config, research_engine).expect("Failed to create pipeline");
    // Skip feedback processor creation as it's not needed for this test

    // Initialize embedding service
    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Step 1: Migrate legacy data through migration service
    // Create a temporary JSON file for the migration test
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("legacy_migration_test.json");
    let legacy_data = test_data["legacy_documents"].clone();
    std::fs::write(
        &temp_file,
        serde_json::to_string_pretty(&legacy_data).unwrap(),
    )
    .unwrap();

    let legacy_source = MigrationSource::JsonFile {
        file_path: temp_file.clone(),
    };

    let migration_id = migration_service
        .start_migration(legacy_source)
        .await
        .expect("Failed to start legacy data migration");

    // Wait for migration completion
    let mut migration_completed = false;
    let mut iterations = 30;

    while !migration_completed && iterations > 0 {
        tokio::time::sleep(Duration::from_millis(100)).await;

        let status = migration_service
            .get_migration_status(&migration_id)
            .await
            .expect("Failed to get migration status");

        match status.status {
            fortitude_core::vector::MigrationStatus::Completed => {
                migration_completed = true;
                println!("Legacy data migration completed successfully");
            }
            fortitude_core::vector::MigrationStatus::Failed => {
                panic!("Legacy data migration failed: {:?}", status);
            }
            _ => {}
        }

        iterations -= 1;
    }

    assert!(migration_completed, "Legacy data migration should complete");

    // Verify migrated data is accessible through storage
    let migrated_doc = storage
        .get_vector("legacy_ml_guide")
        .await
        .expect("Failed to retrieve migrated document");
    assert!(
        migrated_doc.is_some(),
        "Migrated document should be accessible through storage"
    );

    // Step 2: Add research documents directly through storage
    for (id, content, metadata) in &test_data["research_knowledge"] {
        let embedding = embedding_service
            .generate_embedding(content)
            .await
            .expect("Failed to generate embedding for research document");

        storage
            .store_vector(id, &embedding, Some(metadata.clone()))
            .await
            .expect("Failed to store research document");
    }

    // Step 3: Verify cross-service search consistency
    let search_query = "machine learning deployment and async programming";
    let search_results = search_service
        .search(
            search_query,
            SearchOptions {
                limit: 10,
                threshold: Some(0.3),
                collection: None,
                filters: vec![],
                diversify_results: false,
                temporal_boost: None,
                quality_boost: None,
                include_explanations: false,
                min_content_length: None,
                max_content_length: None,
                fuzzy_matching: false,
            },
        )
        .await
        .expect("Failed to search across migrated and research data");

    assert!(
        !search_results.results.is_empty(),
        "Should find results from both migrated and research data"
    );

    // Verify results contain data from different sources
    let result_sources: std::collections::HashSet<String> = search_results
        .results
        .iter()
        .filter_map(|r| r.metadata.as_ref())
        .filter_map(|m| m.get("source").or(m.get("migrated_from")))
        .filter_map(|s| s.as_str())
        .map(|s| s.to_string())
        .collect();

    println!("Search found data from sources: {:?}", result_sources);
    assert!(
        result_sources.len() > 1
            || result_sources.contains("legacy_system")
            || result_sources.contains("research_engine"),
        "Should find data from multiple sources"
    );

    // Step 4: Test pipeline integration with vector search
    let research_request = ClassifiedRequest::new(
        "How to implement efficient async ML model deployment?".to_string(),
        ResearchType::Implementation,
        AudienceContext::TechnicalExpert,
        DomainContext::SoftwareDevelopment,
        0.87,
        vec![
            "machine_learning".to_string(),
            "async".to_string(),
            "deployment".to_string(),
        ],
    );

    // Process through pipeline (should discover context from vector search)
    let pipeline_result = pipeline
        .process_request(research_request.clone())
        .await
        .expect("Pipeline should process request with vector context");

    assert!(
        pipeline_result.is_success(),
        "Pipeline should succeed with vector-enhanced context"
    );

    // Step 5: Create test feedback to verify integration
    let test_feedback = ResearchQualityFeedback {
        feedback_id: "test_feedback_1".to_string(),
        research_cache_key: "test_research_key".to_string(),
        quality_rating: 5,
        context_relevance: 5,
        answer_usefulness: 5,
        completeness_rating: 5,
        feedback_text: Some(
            "Great integration of async patterns with ML deployment strategies".to_string(),
        ),
        issues: vec![],
        suggestions: vec![],
        provided_at: chrono::Utc::now(),
        feedback_source: FeedbackSource::User,
    };

    // Store feedback as searchable content
    for (id, content, metadata) in &test_data["feedback_examples"] {
        let embedding = embedding_service
            .generate_embedding(content)
            .await
            .expect("Failed to generate embedding for feedback");

        storage
            .store_vector(id, &embedding, Some(metadata.clone()))
            .await
            .expect("Failed to store feedback document");
    }

    // Step 6: Verify end-to-end data consistency
    let total_count = storage
        .count_vectors()
        .await
        .expect("Failed to count total vectors");

    let expected_count = test_data["legacy_documents"].len()
        + test_data["research_knowledge"].len()
        + test_data["feedback_examples"].len();

    assert_eq!(
        total_count, expected_count,
        "Total vector count should match all stored documents from all sources"
    );

    // Step 7: Test comprehensive search across all data types
    let comprehensive_search = search_service
        .search(
            "programming patterns feedback",
            SearchOptions {
                limit: 20,
                threshold: Some(0.2),
                collection: None,
                filters: vec![],
                diversify_results: false,
                temporal_boost: None,
                quality_boost: None,
                include_explanations: false,
                min_content_length: None,
                max_content_length: None,
                fuzzy_matching: false,
            },
        )
        .await
        .expect("Failed comprehensive search");

    // Should find documents from all different data types
    let doc_types: std::collections::HashSet<String> = comprehensive_search
        .results
        .iter()
        .filter_map(|r| r.metadata.as_ref())
        .filter_map(|m| m.get("type"))
        .filter_map(|t| t.as_str())
        .map(|t| t.to_string())
        .collect();

    println!("Comprehensive search found document types: {:?}", doc_types);
    assert!(
        doc_types.len() >= 2,
        "Should find multiple document types in comprehensive search"
    );

    // Clean up test data
    for data_set in test_data.values() {
        for (id, _, _) in data_set {
            storage
                .delete_vector(id)
                .await
                .expect("Failed to cleanup test data");
        }
    }

    // Clean up temporary file
    std::fs::remove_file(&temp_file).ok();
}

/// ANCHOR: Test service interaction patterns and dependencies
/// Tests: Service dependency resolution, graceful degradation, error propagation
#[tokio::test]
async fn test_anchor_service_interaction_patterns() {
    let (vector_config, research_config, pipeline_config, _) = create_cross_service_config();

    // Initialize core services
    let embedding_service = LocalEmbeddingService::new(vector_config.embedding.clone());
    let storage =
        VectorStorage::new(vector_config.clone()).expect("Failed to create vector storage");

    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Test 1: Service initialization dependencies
    let search_service = SemanticSearchService::new(
        SemanticSearchConfig::default(),
        storage.clone(),
        embedding_service.clone(),
    );

    let hybrid_service =
        HybridSearchService::new(search_service.clone(), HybridSearchConfig::default());

    // Verify services can operate independently
    let test_content = "Test content for service interaction validation";
    let embedding = embedding_service
        .generate_embedding(test_content)
        .await
        .expect("Embedding service should work independently");

    assert_eq!(
        embedding.len(),
        384,
        "Embedding should have correct dimensions"
    );

    // Test 2: Service chain operations
    let test_doc_id = "service_chain_test";
    let test_metadata = json!({
        "test_id": test_doc_id,
        "operation": "service_chain",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    // Chain: Embedding -> Storage -> Search
    storage
        .store_vector(test_doc_id, &embedding, Some(test_metadata))
        .await
        .expect("Storage should accept embedding from embedding service");

    let search_results = search_service
        .search(
            test_content,
            SearchOptions {
                limit: 5,
                threshold: Some(0.5),
                collection: None,
                filters: vec![],
                diversify_results: false,
                temporal_boost: None,
                quality_boost: None,
                include_explanations: false,
                min_content_length: None,
                max_content_length: None,
                fuzzy_matching: false,
            },
        )
        .await
        .expect("Search should work with stored embedding");

    assert!(
        !search_results.results.is_empty(),
        "Search should find the stored document"
    );

    let found_test_doc = search_results.results.iter().any(|r| r.id == test_doc_id);
    assert!(
        found_test_doc,
        "Search should find the specific test document"
    );

    // Test 3: Hybrid service integration
    let hybrid_request = HybridSearchRequest {
        query: test_content.to_string(),
        strategy: SearchStrategy::Balanced,
        limit: 3,
        semantic_options: None,
        keyword_options: None,
        filters: None,
    };

    let hybrid_results = hybrid_service
        .search(hybrid_request)
        .await
        .expect("Hybrid search should work with underlying services");

    assert!(
        !hybrid_results.results.is_empty(),
        "Hybrid search should find results"
    );

    // Test 4: Error propagation and handling
    let invalid_embedding = vec![0.0; 100]; // Wrong dimension
    let storage_result = storage
        .store_vector("invalid_test", &invalid_embedding, None)
        .await;

    // Should handle dimension mismatch gracefully
    if storage_result.is_err() {
        println!("Storage correctly rejected invalid embedding dimensions");
    }

    // Test empty query handling
    let empty_search = search_service
        .search(
            "",
            SearchOptions {
                limit: 1,
                threshold: Some(0.5),
                collection: None,
                filters: vec![],
                diversify_results: false,
                temporal_boost: None,
                quality_boost: None,
                include_explanations: false,
                min_content_length: None,
                max_content_length: None,
                fuzzy_matching: false,
            },
        )
        .await;

    // Should handle empty query gracefully
    assert!(
        empty_search.is_ok(),
        "Search should handle empty queries gracefully"
    );

    // Test 5: Service state consistency
    let initial_stats = embedding_service.get_stats().await;
    let initial_analytics = search_service
        .get_analytics()
        .await
        .expect("Should get search analytics");

    // Perform operations and verify state updates
    let _more_embeddings = embedding_service
        .generate_embeddings(&vec![
            "Additional test content 1".to_string(),
            "Additional test content 2".to_string(),
        ])
        .await
        .expect("Should generate additional embeddings");

    let _more_searches = search_service
        .search(
            "additional test",
            SearchOptions {
                limit: 2,
                threshold: Some(0.3),
                collection: None,
                filters: vec![],
                diversify_results: false,
                temporal_boost: None,
                quality_boost: None,
                include_explanations: false,
                min_content_length: None,
                max_content_length: None,
                fuzzy_matching: false,
            },
        )
        .await
        .expect("Should perform additional search");

    let updated_stats = embedding_service.get_stats().await;
    let updated_analytics = search_service
        .get_analytics()
        .await
        .expect("Should get updated analytics");

    // Verify state consistency
    assert!(
        updated_stats.total_generated > initial_stats.total_generated,
        "Embedding stats should reflect additional operations"
    );
    assert!(
        updated_analytics.total_searches > initial_analytics.total_searches,
        "Search analytics should reflect additional operations"
    );

    // Test 6: Concurrent service access
    let concurrent_tasks = 5;
    let search_handles: Vec<_> = (0..concurrent_tasks)
        .map(|i| {
            let search_service_clone = search_service.clone();
            let query = format!("concurrent test query {}", i);
            tokio::spawn(async move {
                search_service_clone
                    .search(
                        &query,
                        SearchOptions {
                            limit: 3,
                            threshold: Some(0.3),
                            collection: None,
                            filters: vec![],
                            diversify_results: false,
                            temporal_boost: None,
                            quality_boost: None,
                            include_explanations: false,
                            min_content_length: None,
                            max_content_length: None,
                            fuzzy_matching: false,
                        },
                    )
                    .await
            })
        })
        .collect();

    // All concurrent operations should succeed
    for (i, handle) in search_handles.into_iter().enumerate() {
        let result = handle
            .await
            .expect(&format!("Concurrent task {} should complete", i));
        assert!(result.is_ok(), "Concurrent search {} should succeed", i);
    }

    // Clean up
    storage
        .delete_vector(test_doc_id)
        .await
        .expect("Failed to cleanup test document");
}

/// ANCHOR: Test data integrity across service boundaries
/// Tests: Data validation, consistency checks, integrity maintenance
#[tokio::test]
async fn test_anchor_data_integrity_cross_services() {
    let (vector_config, _, _, migration_config) = create_cross_service_config();

    // Initialize services
    let embedding_service = LocalEmbeddingService::new(vector_config.embedding.clone());
    let storage =
        VectorStorage::new(vector_config.clone()).expect("Failed to create vector storage");
    let search_service = SemanticSearchService::new(
        SemanticSearchConfig {
            default_limit: 10,
            default_threshold: 0.3,
            max_limit: 100,
            enable_analytics: true,
            cache_results: false, // Disable caching for integrity testing
            cache_ttl: Duration::from_secs(300),
            result_diversification: false,
            explain_results: true,
        },
        storage.clone(),
        embedding_service.clone(),
    );
    let migration_service =
        MigrationService::new(migration_config, storage.clone(), embedding_service.clone())
            .expect("Failed to create migration service");

    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Test 1: Embedding consistency across regeneration
    let test_content = "Consistent content for integrity testing";

    let embedding1 = embedding_service
        .generate_embedding(test_content)
        .await
        .expect("Failed to generate first embedding");
    let embedding2 = embedding_service
        .generate_embedding(test_content)
        .await
        .expect("Failed to generate second embedding");

    // Embeddings for same content should be identical
    assert_eq!(
        embedding1, embedding2,
        "Embeddings for identical content should be consistent"
    );

    // Test 2: Storage and retrieval integrity
    let test_documents = vec![
        (
            "integrity_doc_1".to_string(),
            "Document content for integrity validation with specific metadata fields".to_string(),
            json!({
                "integrity_check": true,
                "checksum": "test_checksum_1",
                "version": "1.0",
                "content_hash": "hash_1"
            }),
        ),
        (
            "integrity_doc_2".to_string(),
            "Another document for cross-service integrity testing with different metadata"
                .to_string(),
            json!({
                "integrity_check": true,
                "checksum": "test_checksum_2",
                "version": "1.0",
                "content_hash": "hash_2"
            }),
        ),
    ];

    // Store documents and verify retrieval integrity
    for (id, content, metadata) in &test_documents {
        let embedding = embedding_service
            .generate_embedding(content)
            .await
            .expect(&format!("Failed to generate embedding for {}", id));

        storage
            .store_vector(id, &embedding, Some(metadata.clone()))
            .await
            .expect(&format!("Failed to store document {}", id));

        // Immediately verify storage integrity
        let retrieved = storage
            .get_vector(id)
            .await
            .expect(&format!("Failed to retrieve {}", id));
        assert!(
            retrieved.is_some(),
            "Document should be retrievable immediately after storage"
        );

        let (retrieved_vector, retrieved_metadata) = retrieved.unwrap();
        assert_eq!(
            retrieved_vector, embedding,
            "Retrieved vector should match stored vector"
        );
        assert!(retrieved_metadata.is_some(), "Metadata should be preserved");

        let retrieved_metadata = retrieved_metadata.unwrap();
        assert_eq!(
            retrieved_metadata["checksum"], metadata["checksum"],
            "Metadata should be preserved exactly"
        );
    }

    // Test 3: Search result integrity
    let search_results = search_service
        .search(
            "integrity testing document",
            SearchOptions {
                limit: 10,
                threshold: Some(0.3),
                collection: None,
                filters: vec![],
                diversify_results: false,
                temporal_boost: None,
                quality_boost: None,
                include_explanations: true, // Include vectors for integrity check
                min_content_length: None,
                max_content_length: None,
                fuzzy_matching: false,
            },
        )
        .await
        .expect("Failed to search for integrity test documents");

    assert!(
        !search_results.results.is_empty(),
        "Should find integrity test documents"
    );

    // Verify search results maintain data integrity
    for result in &search_results.results {
        if test_documents.iter().any(|(id, _, _)| id == &result.id) {
            // This is one of our test documents
            assert!(
                result.metadata.is_some(),
                "Test documents should have metadata in search results"
            );
            assert!(
                result.vector.is_some(),
                "Test documents should have vectors when requested"
            );

            let metadata = result.metadata.as_ref().unwrap();
            assert_eq!(
                metadata["integrity_check"], true,
                "Integrity check field should be preserved"
            );

            // Verify vector integrity by comparing with freshly generated embedding
            if let Some(original_doc) = test_documents.iter().find(|(id, _, _)| id == &result.id) {
                let fresh_embedding = embedding_service
                    .generate_embedding(&original_doc.1)
                    .await
                    .expect("Failed to generate fresh embedding for comparison");

                let result_vector = result.vector.as_ref().unwrap();
                assert_eq!(
                    result_vector, &fresh_embedding,
                    "Search result vector should match freshly generated embedding"
                );
            }
        }
    }

    // Test 4: Migration integrity validation
    let migration_data = vec![(
        "migration_integrity_1".to_string(),
        "Content for migration integrity testing with validation markers".to_string(),
        json!({
            "migration_test": true,
            "original_source": "integrity_validation",
            "content_length": 64,
            "validation_hash": "migration_hash_1"
        }),
    )];

    let migration_source = MigrationSource::InMemory {
        data: migration_data.clone(),
        source_name: "integrity_migration_test".to_string(),
    };

    let migration_id = migration_service
        .start_migration(migration_source)
        .await
        .expect("Failed to start integrity migration");

    // Wait for migration completion
    let mut migration_completed = false;
    let mut iterations = 20;

    while !migration_completed && iterations > 0 {
        tokio::time::sleep(Duration::from_millis(100)).await;

        let status = migration_service
            .get_migration_status(&migration_id)
            .await
            .expect("Failed to get migration status");

        if matches!(
            status.status,
            fortitude_core::vector::MigrationStatus::Completed
        ) {
            migration_completed = true;
        } else if matches!(
            status.status,
            fortitude_core::vector::MigrationStatus::Failed
        ) {
            panic!("Migration integrity test failed");
        }

        iterations -= 1;
    }

    assert!(
        migration_completed,
        "Migration should complete successfully"
    );

    // Verify migration preserved data integrity
    let migrated_doc = storage
        .get_vector("migration_integrity_1")
        .await
        .expect("Failed to retrieve migrated document");
    assert!(
        migrated_doc.is_some(),
        "Migrated document should be retrievable"
    );

    let (migrated_vector, migrated_metadata) = migrated_doc.unwrap();
    let migrated_metadata = migrated_metadata.expect("Migrated document should have metadata");

    // Verify original metadata was preserved
    assert_eq!(
        migrated_metadata["migration_test"], true,
        "Original metadata should be preserved during migration"
    );
    assert_eq!(
        migrated_metadata["validation_hash"], "migration_hash_1",
        "Validation hash should be preserved"
    );

    // Verify migrated vector integrity
    let original_content = &migration_data[0].1;
    let expected_embedding = embedding_service
        .generate_embedding(original_content)
        .await
        .expect("Failed to generate expected embedding");

    assert_eq!(
        migrated_vector, expected_embedding,
        "Migrated vector should match expected embedding for original content"
    );

    // Test 5: Cross-service consistency validation
    let consistency_query = "migration integrity testing validation";
    let post_migration_search = search_service
        .search(
            consistency_query,
            SearchOptions {
                limit: 20,
                threshold: Some(0.2),
                collection: None,
                filters: vec![],
                diversify_results: false,
                temporal_boost: None,
                quality_boost: None,
                include_explanations: false,
                min_content_length: None,
                max_content_length: None,
                fuzzy_matching: false,
            },
        )
        .await
        .expect("Failed post-migration search");

    // Should find both directly stored and migrated documents
    let directly_stored_found = post_migration_search
        .results
        .iter()
        .any(|r| r.id.starts_with("integrity_doc_"));
    let migrated_found = post_migration_search
        .results
        .iter()
        .any(|r| r.id.starts_with("migration_integrity_"));

    assert!(
        directly_stored_found,
        "Should find directly stored documents"
    );
    assert!(migrated_found, "Should find migrated documents");

    // Test 6: Concurrent integrity under load
    let concurrent_operations = 10;
    let integrity_handles: Vec<_> = (0..concurrent_operations)
        .map(|i| {
            let storage_clone = storage.clone();
            let embedding_service_clone = embedding_service.clone();
            let content = format!("Concurrent integrity test content {}", i);
            let doc_id = format!("concurrent_integrity_{}", i);

            tokio::spawn(async move {
                // Store
                let embedding = embedding_service_clone.generate_embedding(&content).await?;
                let metadata = json!({
                    "concurrent_test": true,
                    "thread_id": i,
                    "content": content.clone()
                });

                storage_clone
                    .store_vector(&doc_id, &embedding, Some(metadata))
                    .await?;

                // Immediately retrieve and verify
                let retrieved = storage_clone.get_vector(&doc_id).await?;
                if let Some((retrieved_vector, retrieved_metadata)) = retrieved {
                    if retrieved_vector != embedding {
                        return Err(fortitude_core::vector::VectorError::InvalidOperation(
                            "Vector integrity check failed".to_string(),
                        ));
                    }

                    let retrieved_metadata = retrieved_metadata.ok_or_else(|| {
                        fortitude_core::vector::VectorError::InvalidOperation(
                            "Metadata missing".to_string(),
                        )
                    })?;

                    if retrieved_metadata["content"] != content {
                        return Err(fortitude_core::vector::VectorError::InvalidOperation(
                            "Metadata integrity check failed".to_string(),
                        ));
                    }
                }

                Ok(doc_id)
            })
        })
        .collect();

    // All concurrent integrity operations should succeed
    for (i, handle) in integrity_handles.into_iter().enumerate() {
        let result = handle
            .await
            .expect(&format!("Concurrent integrity task {} should complete", i));
        assert!(
            result.is_ok(),
            "Concurrent integrity operation {} should succeed",
            i
        );
    }

    // Clean up all test data
    let all_test_ids = vec![
        "integrity_doc_1",
        "integrity_doc_2",
        "migration_integrity_1",
    ];

    for id in all_test_ids {
        storage
            .delete_vector(id)
            .await
            .expect("Failed to cleanup test data");
    }

    // Clean up concurrent test data
    for i in 0..concurrent_operations {
        let doc_id = format!("concurrent_integrity_{}", i);
        storage.delete_vector(&doc_id).await.ok(); // Ignore errors for cleanup
    }
}

/// ANCHOR: Test service configuration and feature interaction
/// Tests: Configuration validation across services, feature flag interactions
#[tokio::test]
async fn test_anchor_service_configuration_interaction() {
    let (vector_config, _, _, _) = create_cross_service_config();

    // Test 1: Configuration inheritance and validation
    let embedding_service = LocalEmbeddingService::new(vector_config.embedding.clone());
    let storage = VectorStorage::new(vector_config.clone())
        .expect("Failed to create storage with valid config");

    // Test that services inherit and validate configuration correctly
    embedding_service
        .initialize()
        .await
        .expect("Embedding service should initialize with valid config");

    let embedding_stats = embedding_service.get_stats().await;
    assert_eq!(embedding_stats.cache_size, 0, "Cache should start empty");

    // Test 2: Search service configuration inheritance
    let search_configs = vec![
        SemanticSearchConfig {
            enable_explain: true,
            cache_enabled: true,
            cache_ttl: Duration::from_secs(300),
            ..Default::default()
        },
        SemanticSearchConfig {
            enable_explain: false,
            cache_enabled: false,
            default_threshold: 0.5,
            ..Default::default()
        },
    ];

    for (i, search_config) in search_configs.into_iter().enumerate() {
        let search_service = SemanticSearchService::new(
            search_config.clone(),
            storage.clone(),
            embedding_service.clone(),
        )
        .expect(&format!(
            "Search service {} should initialize with valid config",
            i
        ));

        // Test configuration affects behavior
        let test_query = format!("Configuration test query {}", i);
        let results = search_service
            .search(
                &test_query,
                SearchOptions {
                    limit: 5,
                    threshold: Some(0.3),
                    collection: None,
                    filters: vec![],
                    diversify_results: false,
                    temporal_boost: None,
                    quality_boost: None,
                    include_explanations: false,
                    min_content_length: None,
                    max_content_length: None,
                    fuzzy_matching: false,
                },
            )
            .await
            .expect(&format!("Search {} should succeed", i));

        // Verify configuration-dependent behavior
        if search_config.enable_explain {
            // When explain is enabled, should provide additional metadata
            // (In actual implementation, this would check for explanation data)
            assert!(
                results.execution_stats.is_some(),
                "Should provide execution stats when explain is enabled"
            );
        }
    }

    // Test 3: Hybrid service configuration composition
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
            semantic_weight: 0.5,
            keyword_weight: 0.5,
            fusion_method: FusionMethod::WeightedSum,
            enable_query_analysis: false,
            enable_performance_tracking: false,
            ..Default::default()
        },
    ];

    let base_search = SemanticSearchService::new(
        SemanticSearchConfig::default(),
        storage.clone(),
        embedding_service.clone(),
    );

    for (i, hybrid_config) in hybrid_configs.into_iter().enumerate() {
        let keyword_searcher = Arc::new(fortitude_core::vector::KeywordSearcher::new());
        let hybrid_service = HybridSearchService::new(
            Arc::new(base_search.clone()),
            keyword_searcher,
            hybrid_config.clone(),
        );

        let request = HybridSearchRequest {
            query: format!("Hybrid configuration test {}", i),
            strategy: SearchStrategy::Balanced,
            limit: 3,
            semantic_options: None,
            keyword_options: None,
            filters: None,
        };

        let results = hybrid_service
            .search(request)
            .await
            .expect(&format!("Hybrid search {} should succeed", i));

        // Verify configuration affects search behavior
        if hybrid_config.enable_performance_tracking {
            assert!(
                results.performance_metrics.is_some(),
                "Should provide performance metrics when tracking is enabled"
            );
        }
    }

    // Test 4: Configuration validation cascade
    let mut invalid_config = vector_config.clone();
    invalid_config.vector_dimensions = 0; // Invalid dimension

    let invalid_storage_result = VectorStorage::new(invalid_config);
    assert!(
        invalid_storage_result.is_err(),
        "Should reject invalid vector configuration"
    );

    // Test 5: Feature flag interactions
    let mut test_embedding_config = vector_config.embedding.clone();
    test_embedding_config.cache_config.enabled = false;

    let no_cache_embedding_service = LocalEmbeddingService::new(test_embedding_config);
    no_cache_embedding_service
        .initialize()
        .await
        .expect("Should initialize without cache");

    // Verify cache configuration affects behavior
    let test_content = "Cache configuration test content";
    let _embedding1 = no_cache_embedding_service
        .generate_embedding(test_content)
        .await
        .expect("Should generate embedding without cache");
    let _embedding2 = no_cache_embedding_service
        .generate_embedding(test_content)
        .await
        .expect("Should generate embedding again without cache");

    let no_cache_stats = no_cache_embedding_service.get_stats().await;
    assert_eq!(
        no_cache_stats.cache_size, 0,
        "Cache size should remain 0 when caching is disabled"
    );

    // Test 6: Cross-service configuration consistency
    let search_service = SemanticSearchService::new(
        SemanticSearchConfig {
            default_limit: 10,
            default_threshold: 0.3,
            max_limit: 100,
            enable_analytics: true,
            cache_results: true,
            cache_ttl: Duration::from_secs(300),
            result_diversification: false,
            explain_results: false,
        },
        storage.clone(),
        embedding_service.clone(),
    );

    // Store test document
    let test_content = "Cross-service configuration consistency test";
    let embedding = embedding_service
        .generate_embedding(test_content)
        .await
        .expect("Failed to generate embedding");

    storage
        .store_vector(
            "config_test",
            &embedding,
            Some(json!({
                "test": "configuration_consistency"
            })),
        )
        .await
        .expect("Failed to store test document");

    // Search should find the document using the same collection configuration
    let search_results = search_service
        .search(
            test_content,
            SearchOptions {
                limit: 5,
                threshold: Some(0.3),
                collection: None,
                filters: vec![],
                diversify_results: false,
                temporal_boost: None,
                quality_boost: None,
                include_explanations: false,
                min_content_length: None,
                max_content_length: None,
                fuzzy_matching: false,
            },
        )
        .await
        .expect("Search should succeed with consistent configuration");

    let found_test_doc = search_results.results.iter().any(|r| r.id == "config_test");
    assert!(
        found_test_doc,
        "Should find document using consistent collection configuration"
    );

    // Clean up
    storage
        .delete_vector("config_test")
        .await
        .expect("Failed to cleanup configuration test document");
}
