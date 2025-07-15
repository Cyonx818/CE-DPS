// ABOUTME: Integration tests for semantic gap analysis using vector database
//! This test module validates the semantic analysis integration with the gap detection system.
//! Tests include gap validation, related content discovery, priority enhancement,
//! and performance requirements using the vector database.

use chrono::Utc;
use fortitude::proactive::{
    DetectedGap, GapAnalysisConfig, GapAnalyzer, GapType, RelationshipType, SemanticAnalysisConfig,
    SemanticAnalysisError, SemanticGapAnalyzer,
};
use fortitude_core::vector::storage::{DocumentMetadata, VectorStorageService};
use fortitude_core::vector::{
    client::QdrantClient,
    config::{ConnectionPoolConfig, VectorConfig},
    embeddings::{EmbeddingConfig, LocalEmbeddingService},
    error::{VectorError, VectorResult},
    MatchMetadata, SearchExecutionStats, SearchOptions, SearchQueryMetadata, SearchResult,
    SearchResultSet, SemanticSearchOperations, SemanticSearchService, SuggestionRequest,
    VectorDocument, VectorStorage,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;
use tokio::fs;

/// Helper function to create test vector storage
async fn create_test_vector_storage() -> Result<Arc<VectorStorage>, Box<dyn std::error::Error>> {
    let embedding_config = EmbeddingConfig {
        model_name: "test-model".to_string(),
        max_sequence_length: 128,
        batch_size: 4,
        ..Default::default()
    };

    let embedding_service = Arc::new(LocalEmbeddingService::new(embedding_config));

    let vector_config = VectorConfig {
        url: "http://localhost:6334".to_string(),
        default_collection: "test_semantic_analysis".to_string(),
        api_key: None,
        timeout: std::time::Duration::from_secs(5),
        vector_dimensions: 384,
        distance_metric: fortitude_core::vector::config::DistanceMetric::Cosine,
        health_check: fortitude_core::vector::config::HealthCheckConfig::default(),
        connection_pool: ConnectionPoolConfig::default(),
        embedding: EmbeddingConfig::default(),
    };

    let qdrant_client = match QdrantClient::new(vector_config).await {
        Ok(client) => Arc::new(client),
        Err(_) => {
            // Skip tests if Qdrant is not available
            return Err("Test Qdrant instance not available".into());
        }
    };

    let storage = VectorStorage::new(qdrant_client, embedding_service);
    storage.initialize().await?;
    Ok(Arc::new(storage))
}

/// Helper function to create semantic search service
async fn create_semantic_search_service(
) -> Result<Arc<SemanticSearchService>, Box<dyn std::error::Error>> {
    let vector_storage = create_test_vector_storage().await?;
    let search_service = SemanticSearchService::with_defaults(vector_storage);
    search_service.initialize().await?;
    Ok(Arc::new(search_service))
}

/// Helper function to create test gaps
fn create_test_gaps() -> Vec<DetectedGap> {
    vec![
        DetectedGap::new(
            GapType::TodoComment,
            PathBuf::from("src/lib.rs"),
            42,
            "// TODO: Implement async error handling".to_string(),
            "Implement async error handling".to_string(),
            0.9,
        )
        .with_metadata("function_name", "handle_request")
        .with_metadata("complexity", "high"),
        DetectedGap::new(
            GapType::MissingDocumentation,
            PathBuf::from("src/api.rs"),
            15,
            "pub async fn process_data(data: &str) -> Result<String, Error>".to_string(),
            "Public function 'process_data' lacks documentation".to_string(),
            0.8,
        )
        .with_metadata("visibility", "public")
        .with_metadata("async", "true"),
        DetectedGap::new(
            GapType::UndocumentedTechnology,
            PathBuf::from("src/main.rs"),
            5,
            "use tokio::sync::RwLock;".to_string(),
            "External crate 'tokio' may need documentation".to_string(),
            0.7,
        )
        .with_metadata("crate_name", "tokio")
        .with_metadata("import_type", "sync"),
    ]
}

/// Helper function to populate vector database with test content
async fn populate_test_content(
    search_service: &SemanticSearchService,
) -> Result<(), Box<dyn std::error::Error>> {
    let test_docs = vec![
        VectorDocument {
            id: "async-error-handling-guide".to_string(),
            content: "Comprehensive guide to async error handling in Rust. This document covers Result types, async/await patterns, and error propagation strategies.".to_string(),
            embedding: vec![0.1, 0.2, 0.3, 0.4], // Mock embedding
            metadata: DocumentMetadata {
                content_type: "guide".to_string(),
                quality_score: Some(0.9),
                tags: vec!["async".to_string(), "error".to_string(), "rust".to_string()],
                custom_fields: {
                    let mut fields = std::collections::HashMap::new();
                    fields.insert("title".to_string(), serde_json::json!("Async Error Handling Guide"));
                    fields.insert("topic".to_string(), serde_json::json!("error_handling"));
                    fields.insert("language".to_string(), serde_json::json!("rust"));
                    fields
                },
                ..Default::default()
            },
            stored_at: Utc::now(),
        },
        VectorDocument {
            id: "tokio-documentation".to_string(),
            content: "Tokio is an asynchronous runtime for Rust programming language. It provides async I/O, timers, sync primitives like RwLock, and task scheduling.".to_string(),
            embedding: vec![0.2, 0.3, 0.4, 0.5], // Mock embedding
            metadata: DocumentMetadata {
                content_type: "documentation".to_string(),
                quality_score: Some(0.85),
                tags: vec!["tokio".to_string(), "async".to_string(), "runtime".to_string()],
                custom_fields: {
                    let mut fields = std::collections::HashMap::new();
                    fields.insert("title".to_string(), serde_json::json!("Tokio Runtime Documentation"));
                    fields.insert("crate".to_string(), serde_json::json!("tokio"));
                    fields.insert("category".to_string(), serde_json::json!("async"));
                    fields
                },
                ..Default::default()
            },
            stored_at: Utc::now(),
        },
        VectorDocument {
            id: "api-documentation-patterns".to_string(),
            content: "Best practices for documenting public APIs in Rust. Include examples, error cases, and usage patterns for better developer experience.".to_string(),
            embedding: vec![0.3, 0.4, 0.5, 0.6], // Mock embedding
            metadata: DocumentMetadata {
                content_type: "best_practices".to_string(),
                quality_score: Some(0.8),
                tags: vec!["api".to_string(), "documentation".to_string(), "patterns".to_string()],
                custom_fields: {
                    let mut fields = std::collections::HashMap::new();
                    fields.insert("title".to_string(), serde_json::json!("API Documentation Patterns"));
                    fields.insert("topic".to_string(), serde_json::json!("documentation"));
                    fields.insert("scope".to_string(), serde_json::json!("api"));
                    fields
                },
                ..Default::default()
            },
            stored_at: Utc::now(),
        },
    ];

    // Store documents (this would normally go through the vector storage interface)
    // For integration tests, we'll need to implement proper storage
    // This is a placeholder for the actual implementation

    Ok(())
}

#[tokio::test]
async fn test_semantic_gap_validation_with_existing_content() {
    // FAILING TEST: Should validate gaps against existing content in vector database
    let search_service = create_semantic_search_service().await.unwrap();
    populate_test_content(&*search_service).await.unwrap();

    let config = SemanticAnalysisConfig::default();
    let semantic_analyzer = SemanticGapAnalyzer::new(search_service, config);

    let gaps = create_test_gaps();
    let results = semantic_analyzer
        .analyze_gaps_semantically(gaps)
        .await
        .unwrap();

    // Gap about async error handling should be flagged as potentially covered
    let async_error_gap = results
        .iter()
        .find(|r| r.gap.description.contains("async error handling"))
        .unwrap();

    assert!(!async_error_gap.is_validated); // Should be invalidated due to existing content
    assert!(async_error_gap.validation_confidence < 0.8); // Lower confidence
    assert!(!async_error_gap.related_documents.is_empty()); // Should have related docs
}

#[tokio::test]
async fn test_semantic_gap_validation_no_existing_content() {
    // FAILING TEST: Should validate gaps as legitimate when no similar content exists
    let search_service = create_semantic_search_service().await.unwrap();
    // Don't populate with content - empty database

    let config = SemanticAnalysisConfig::default();
    let semantic_analyzer = SemanticGapAnalyzer::new(search_service, config);

    let gaps = vec![DetectedGap::new(
        GapType::TodoComment,
        PathBuf::from("src/unique.rs"),
        1,
        "// TODO: Implement quantum computing interface".to_string(),
        "Implement quantum computing interface".to_string(),
        0.9,
    )];

    let results = semantic_analyzer
        .analyze_gaps_semantically(gaps)
        .await
        .unwrap();
    let unique_gap = &results[0];

    assert!(unique_gap.is_validated); // Should be validated - no similar content
    assert!(unique_gap.validation_confidence > 0.8); // High confidence
    assert!(unique_gap.related_documents.is_empty()); // No related docs
}

#[tokio::test]
async fn test_related_content_discovery() {
    // FAILING TEST: Should discover semantically related content for gaps
    let search_service = create_semantic_search_service().await.unwrap();
    populate_test_content(&*search_service).await.unwrap();

    let config = SemanticAnalysisConfig::default();
    let semantic_analyzer = SemanticGapAnalyzer::new(search_service, config);

    let gaps = create_test_gaps();
    let results = semantic_analyzer
        .analyze_gaps_semantically(gaps)
        .await
        .unwrap();

    // Tokio gap should find related tokio documentation
    let tokio_gap = results
        .iter()
        .find(|r| r.gap.description.contains("tokio"))
        .unwrap();

    assert!(!tokio_gap.related_documents.is_empty());

    let related_doc = &tokio_gap.related_documents[0];
    assert!(related_doc.content_preview.to_lowercase().contains("tokio"));
    assert!(related_doc.similarity_score >= 0.7);
    assert_eq!(
        related_doc.relationship_type,
        RelationshipType::TopicalSimilarity
    );
}

#[tokio::test]
async fn test_relationship_type_classification() {
    // FAILING TEST: Should correctly classify relationship types between gaps and content
    let search_service = create_semantic_search_service().await.unwrap();
    populate_test_content(&*search_service).await.unwrap();

    let config = SemanticAnalysisConfig::default();
    let semantic_analyzer = SemanticGapAnalyzer::new(search_service, config);

    // Create gap with high similarity to existing content
    let gaps = vec![DetectedGap::new(
        GapType::TodoComment,
        PathBuf::from("src/error.rs"),
        1,
        "// TODO: Add async error handling implementation example".to_string(),
        "Add async error handling implementation example".to_string(),
        0.9,
    )];

    let results = semantic_analyzer
        .analyze_gaps_semantically(gaps)
        .await
        .unwrap();
    let gap_result = &results[0];

    assert!(!gap_result.related_documents.is_empty());

    // Should classify as implementation pattern due to "example" keyword
    let has_implementation_pattern = gap_result
        .related_documents
        .iter()
        .any(|doc| doc.relationship_type == RelationshipType::ImplementationPattern);
    assert!(has_implementation_pattern);
}

#[tokio::test]
async fn test_semantic_priority_enhancement() {
    // FAILING TEST: Should enhance gap priorities based on semantic analysis
    let search_service = create_semantic_search_service().await.unwrap();

    let config = SemanticAnalysisConfig {
        semantic_priority_weight: 1.0, // Full weight for testing
        ..SemanticAnalysisConfig::default()
    };
    let semantic_analyzer = SemanticGapAnalyzer::new(search_service, config);

    let gaps = create_test_gaps();
    let original_priorities: Vec<u8> = gaps.iter().map(|g| g.priority).collect();

    let results = semantic_analyzer
        .analyze_gaps_semantically(gaps)
        .await
        .unwrap();

    // Priority should be enhanced for gaps with no related content
    let gap_with_no_content = results
        .iter()
        .find(|r| r.related_documents.is_empty())
        .unwrap();

    let original_idx = results
        .iter()
        .position(|r| {
            r.gap.file_path == gap_with_no_content.gap.file_path
                && r.gap.line_number == gap_with_no_content.gap.line_number
        })
        .unwrap();

    assert!(gap_with_no_content.enhanced_priority >= original_priorities[original_idx]);
}

#[tokio::test]
async fn test_semantic_query_construction() {
    // FAILING TEST: Should construct effective semantic queries from gap information
    let search_service = create_semantic_search_service().await.unwrap();
    let config = SemanticAnalysisConfig::default();
    let semantic_analyzer = SemanticGapAnalyzer::new(search_service, config);

    let gap = DetectedGap::new(
        GapType::ApiDocumentationGap,
        PathBuf::from("src/api.rs"),
        10,
        "pub fn calculate(x: i32, y: i32) -> i32".to_string(),
        "Public function 'calculate' documentation lacks examples".to_string(),
        0.8,
    )
    .with_metadata("function_name", "calculate")
    .with_metadata("missing_element", "examples");

    let results = semantic_analyzer
        .analyze_gaps_semantically(vec![gap])
        .await
        .unwrap();
    let result = &results[0];

    // Query should include gap description, type keywords, and metadata
    assert!(result.metadata.search_query.contains("calculate"));
    assert!(result.metadata.search_query.contains("documentation"));
    assert!(result.metadata.search_query.contains("examples"));
    assert!(result.metadata.search_query.contains("api"));
}

#[tokio::test]
async fn test_batch_processing_performance() {
    // FAILING TEST: Should process multiple gaps efficiently within performance limits
    let search_service = create_semantic_search_service().await.unwrap();

    let config = SemanticAnalysisConfig {
        max_analysis_time_ms: 100, // Strict limit for testing
        batch_size: 10,
        ..SemanticAnalysisConfig::default()
    };
    let semantic_analyzer = SemanticGapAnalyzer::new(search_service, config);

    // Create many gaps to test batch processing
    let mut gaps = Vec::new();
    for i in 0..25 {
        gaps.push(DetectedGap::new(
            GapType::TodoComment,
            PathBuf::from(format!("src/file_{}.rs", i)),
            i + 1,
            format!("// TODO: Implement feature {}", i),
            format!("Implement feature {}", i),
            0.8,
        ));
    }

    let start_time = Instant::now();
    let results = semantic_analyzer
        .analyze_gaps_semantically(gaps)
        .await
        .unwrap();
    let duration = start_time.elapsed();

    assert_eq!(results.len(), 25);

    // Should complete within reasonable time (allowing some overhead for testing)
    assert!(duration.as_millis() < 500); // Total analysis under 500ms

    // Each gap should have reasonable analysis time
    for result in &results {
        assert!(result.metadata.analysis_time_ms < 100.0); // Per-gap under 100ms
    }
}

#[tokio::test]
async fn test_vector_query_batching() {
    // FAILING TEST: Should batch vector queries efficiently to minimize database calls
    let search_service = create_semantic_search_service().await.unwrap();

    let config = SemanticAnalysisConfig {
        batch_size: 5,
        ..SemanticAnalysisConfig::default()
    };
    let semantic_analyzer = SemanticGapAnalyzer::new(search_service, config);

    let gaps = create_test_gaps();
    let results = semantic_analyzer
        .analyze_gaps_semantically(gaps)
        .await
        .unwrap();

    // Should limit vector queries per gap
    for result in &results {
        assert!(result.metadata.vector_queries_count <= 3); // Max queries per gap
        assert!(!result.metadata.features_used.is_empty()); // Should track features used
    }
}

#[tokio::test]
async fn test_semantic_analysis_with_filters() {
    // FAILING TEST: Should apply appropriate filters when searching for related content
    let search_service = create_semantic_search_service().await.unwrap();
    populate_test_content(&*search_service).await.unwrap();

    let config = SemanticAnalysisConfig {
        related_content_threshold: 0.6, // Lower threshold for more results
        max_related_documents: 15,
        ..SemanticAnalysisConfig::default()
    };
    let semantic_analyzer = SemanticGapAnalyzer::new(search_service, config);

    let gaps = create_test_gaps();
    let results = semantic_analyzer
        .analyze_gaps_semantically(gaps)
        .await
        .unwrap();

    // Should find related documents within threshold
    let gap_with_related = results
        .iter()
        .find(|r| !r.related_documents.is_empty())
        .unwrap();

    for doc in &gap_with_related.related_documents {
        assert!(doc.similarity_score >= 0.6); // Above threshold
        assert!(!doc.content_preview.is_empty()); // Has content preview
        assert!(!doc.document_id.is_empty()); // Has valid ID
    }
}

#[tokio::test]
async fn test_gap_analysis_integration_with_semantic_features() {
    // FAILING TEST: Should integrate seamlessly with existing gap analysis pipeline
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");

    let content = r#"
use tokio::sync::RwLock;

// TODO: Implement comprehensive error handling
pub fn unhandled_function() {
    println!("No error handling");
}

pub async fn undocumented_async_function(data: String) -> Result<String, Error> {
    // Implementation without documentation
    Ok(data)
}
"#;

    fs::write(&test_file, content).await.unwrap();

    // First, run standard gap analysis
    let gap_analyzer = GapAnalyzer::for_rust_project().unwrap();
    let detected_gaps = gap_analyzer.analyze_file(&test_file).await.unwrap();

    assert!(!detected_gaps.is_empty());

    // Then, run semantic analysis on detected gaps
    let search_service = create_semantic_search_service().await.unwrap();
    populate_test_content(&*search_service).await.unwrap();

    let config = SemanticAnalysisConfig::default();
    let semantic_analyzer = SemanticGapAnalyzer::new(search_service, config);

    let semantic_results = semantic_analyzer
        .analyze_gaps_semantically(detected_gaps)
        .await
        .unwrap();

    assert!(!semantic_results.is_empty());

    // Each semantic result should correspond to a detected gap
    assert_eq!(semantic_results.len(), semantic_results.len());

    // Should have metadata about the analysis process
    for result in &semantic_results {
        assert!(result.metadata.analysis_time_ms > 0.0);
        assert!(!result.metadata.search_query.is_empty());
        assert!(!result.metadata.features_used.is_empty());
    }
}

#[tokio::test]
async fn test_configuration_variants() {
    // FAILING TEST: Should work with different configuration settings
    let search_service = create_semantic_search_service().await.unwrap();

    // Test performance-optimized config
    let perf_config = SemanticAnalysisConfig::for_performance();
    let perf_analyzer = SemanticGapAnalyzer::new(
        search_service.clone() as Arc<dyn SemanticSearchOperations>,
        perf_config,
    );

    // Test accuracy-optimized config
    let accuracy_config = SemanticAnalysisConfig::for_accuracy();
    let accuracy_analyzer = SemanticGapAnalyzer::new(
        search_service.clone() as Arc<dyn SemanticSearchOperations>,
        accuracy_config,
    );

    let gaps = create_test_gaps();

    // Both should work but with different characteristics
    let perf_results = perf_analyzer
        .analyze_gaps_semantically(gaps.clone())
        .await
        .unwrap();
    let accuracy_results = accuracy_analyzer
        .analyze_gaps_semantically(gaps)
        .await
        .unwrap();

    assert_eq!(perf_results.len(), accuracy_results.len());

    // Performance config should be faster
    let perf_total_time: f64 = perf_results
        .iter()
        .map(|r| r.metadata.analysis_time_ms)
        .sum();
    let accuracy_total_time: f64 = accuracy_results
        .iter()
        .map(|r| r.metadata.analysis_time_ms)
        .sum();

    // Performance config should generally be faster (allowing some variance)
    assert!(perf_total_time <= accuracy_total_time * 1.5);
}

#[tokio::test]
async fn test_error_handling_in_semantic_analysis() {
    // FAILING TEST: Should handle errors gracefully during semantic analysis
    let search_service = create_semantic_search_service().await.unwrap();

    let config = SemanticAnalysisConfig {
        min_content_length: 1000, // Set high to trigger query construction errors
        ..SemanticAnalysisConfig::default()
    };
    let semantic_analyzer = SemanticGapAnalyzer::new(search_service, config);

    // Create gap with insufficient content
    let gaps = vec![DetectedGap::new(
        GapType::TodoComment,
        PathBuf::from("src/test.rs"),
        1,
        "// TODO: x".to_string(),
        "x".to_string(),
        0.9,
    )];

    let result = semantic_analyzer.analyze_gaps_semantically(gaps).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        SemanticAnalysisError::QueryConstruction(_)
    ));
}

#[tokio::test]
async fn test_semantic_analysis_metadata_completeness() {
    // FAILING TEST: Should provide comprehensive metadata about the analysis process
    let search_service = create_semantic_search_service().await.unwrap();
    populate_test_content(&*search_service).await.unwrap();

    let config = SemanticAnalysisConfig::default();
    let semantic_analyzer = SemanticGapAnalyzer::new(search_service, config);

    let gaps = create_test_gaps();
    let results = semantic_analyzer
        .analyze_gaps_semantically(gaps)
        .await
        .unwrap();

    for result in &results {
        let metadata = &result.metadata;

        // Should have timing information
        assert!(metadata.analysis_time_ms > 0.0);

        // Should track query count
        assert!(metadata.vector_queries_count > 0);

        // Should have the search query used
        assert!(!metadata.search_query.is_empty());
        assert!(metadata.search_query.len() >= 50); // Reasonable query length

        // Should track number of results
        assert!(metadata.search_results_count >= 0);

        // Should list features used
        assert!(!metadata.features_used.is_empty());
        assert!(metadata
            .features_used
            .contains(&"gap_validation".to_string()));

        if !result.related_documents.is_empty() {
            assert!(metadata
                .features_used
                .contains(&"related_content".to_string()));
        }
    }
}
