//! Integration tests for vector database functionality.
//! These tests verify end-to-end workflows using the public API.

use fortitude_core::vector::{
    CacheKeyStrategy, ConnectionPoolConfig, DeviceType, DistanceMetric, EmbeddingCacheConfig,
    EmbeddingConfig, EmbeddingGenerator, FusionMethod, HealthCheckConfig, HybridSearchConfig,
    HybridSearchOperations, HybridSearchRequest, HybridSearchService, LocalEmbeddingService,
    SearchOptions, SearchStrategy, SemanticSearchConfig, SemanticSearchOperations,
    SemanticSearchService, VectorConfig, VectorStorage, VectorStorageService,
};
// Types available for future use
// use fortitude_types::research::{AudienceContext, ClassifiedRequest, DomainContext, ResearchType};
use std::time::Duration;

/// Test configuration for integration tests
fn create_test_vector_config() -> VectorConfig {
    VectorConfig {
        url: "http://localhost:6334".to_string(),
        api_key: None,
        timeout: Duration::from_secs(30),
        default_collection: "test_research_docs".to_string(),
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
            idle_timeout: Duration::from_secs(600),
            connection_timeout: Duration::from_secs(10),
        },
        embedding: EmbeddingConfig {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            max_sequence_length: 128,
            batch_size: 4,
            device: DeviceType::Cpu,
            cache_config: EmbeddingCacheConfig {
                enabled: true,
                max_entries: 100,
                ttl: Duration::from_secs(300),
                key_strategy: CacheKeyStrategy::Hash,
            },
            ..Default::default()
        },
    }
}

/// ANCHOR: Test complete vector storage and retrieval workflow
/// Tests: Document storage, embedding generation, vector search, and retrieval
#[tokio::test]
async fn test_anchor_complete_vector_workflow() {
    let config = create_test_vector_config();

    // Initialize services
    let embedding_service = LocalEmbeddingService::new(config.embedding.clone());
    let storage = VectorStorage::new(config.clone()).expect("Failed to create vector storage");

    // Initialize embedding service
    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Test data - research documents
    let documents = vec![
        (
            "doc1",
            "How to implement async Rust programming with tokio runtime and futures",
        ),
        (
            "doc2",
            "Vector database integration using Qdrant for semantic search capabilities",
        ),
        (
            "doc3",
            "Machine learning model deployment strategies for production environments",
        ),
        (
            "doc4",
            "Database optimization techniques for high-performance applications",
        ),
    ];

    // Store documents with embeddings
    for (id, content) in &documents {
        let embedding = embedding_service
            .generate_embedding(content)
            .await
            .expect("Failed to generate embedding");

        let metadata = serde_json::json!({
            "title": format!("Document {}", id),
            "category": "research",
            "content": content
        });

        storage
            .store_vector(id, &embedding, Some(metadata))
            .await
            .expect("Failed to store vector");
    }

    // Verify storage
    let stored_count = storage
        .count_vectors()
        .await
        .expect("Failed to count vectors");
    assert_eq!(stored_count, 4, "Should have stored 4 documents");

    // Test semantic search
    let query = "async programming in Rust";
    let query_embedding = embedding_service
        .generate_embedding(query)
        .await
        .expect("Failed to generate query embedding");

    let search_results = storage
        .search_vectors(&query_embedding, 2, None)
        .await
        .expect("Failed to search vectors");

    assert!(!search_results.is_empty(), "Search should return results");
    assert!(search_results.len() <= 2, "Should return at most 2 results");

    // Verify result quality - first result should be the Rust document
    let best_match = &search_results[0];
    assert!(
        best_match.score > 0.5,
        "Best match should have high similarity score"
    );

    // Retrieve document by ID
    let retrieved = storage
        .get_vector("doc1")
        .await
        .expect("Failed to retrieve vector");
    assert!(retrieved.is_some(), "Should retrieve stored document");

    let (vector, metadata) = retrieved.unwrap();
    assert_eq!(vector.len(), 384, "Vector should have correct dimensions");
    assert!(metadata.is_some(), "Should have metadata");

    let metadata = metadata.unwrap();
    assert_eq!(metadata["category"], "research", "Should preserve metadata");

    // Test batch operations
    let batch_queries = vec![
        "machine learning deployment",
        "database performance optimization",
    ];

    let batch_embeddings = embedding_service
        .generate_embeddings(&batch_queries)
        .await
        .expect("Failed to generate batch embeddings");

    assert_eq!(
        batch_embeddings.len(),
        2,
        "Should generate embeddings for all queries"
    );

    // Clean up test data
    for (id, _) in &documents {
        storage
            .delete_vector(id)
            .await
            .expect("Failed to delete vector");
    }

    let final_count = storage
        .count_vectors()
        .await
        .expect("Failed to count vectors after cleanup");
    assert_eq!(final_count, 0, "Should have cleaned up all test documents");
}

/// ANCHOR: Test semantic search service integration
/// Tests: Search service initialization, query processing, and result ranking
#[tokio::test]
async fn test_anchor_semantic_search_integration() {
    let config = create_test_vector_config();
    let search_config = SemanticSearchConfig {
        collection_name: config.default_collection.clone(),
        default_limit: 10,
        min_score_threshold: 0.6,
        enable_explain: true,
        cache_enabled: true,
        cache_ttl: Duration::from_secs(300),
    };

    // Initialize services
    let embedding_service = LocalEmbeddingService::new(config.embedding.clone());
    let storage = VectorStorage::new(config.clone()).expect("Failed to create vector storage");
    let search_service =
        SemanticSearchService::new(search_config, storage.clone(), embedding_service.clone())
            .expect("Failed to create search service");

    // Initialize embedding service
    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Prepare test documents
    let test_docs = vec![
        (
            "rust_async",
            "Asynchronous programming in Rust using tokio and async/await syntax",
        ),
        (
            "python_ml",
            "Machine learning with Python using scikit-learn and tensorflow libraries",
        ),
        (
            "database_perf",
            "Database performance tuning for PostgreSQL and MySQL systems",
        ),
        (
            "web_security",
            "Web application security best practices for authentication and authorization",
        ),
    ];

    // Store documents
    for (id, content) in &test_docs {
        let embedding = embedding_service
            .generate_embedding(content)
            .await
            .expect("Failed to generate embedding");

        let metadata = serde_json::json!({
            "id": id,
            "content": content,
            "category": "technical"
        });

        storage
            .store_vector(id, &embedding, Some(metadata))
            .await
            .expect("Failed to store document");
    }

    // Test semantic search
    let search_options = SearchOptions {
        limit: Some(3),
        score_threshold: Some(0.4),
        with_payload: true,
        with_vectors: false,
    };

    let query = "async Rust programming";
    let results = search_service
        .search(query, search_options)
        .await
        .expect("Search should succeed");

    assert!(
        !results.results.is_empty(),
        "Should find relevant documents"
    );
    assert!(
        results.query_metadata.is_some(),
        "Should have query metadata"
    );
    assert!(
        results.execution_stats.is_some(),
        "Should have execution stats"
    );

    // Verify result quality
    let best_result = &results.results[0];
    assert!(
        best_result.score >= 0.4,
        "Result should meet score threshold"
    );
    assert!(best_result.metadata.is_some(), "Should include metadata");

    let metadata = best_result.metadata.as_ref().unwrap();
    assert_eq!(
        metadata["category"], "technical",
        "Should preserve document metadata"
    );

    // Test with filters
    let filtered_options = SearchOptions {
        limit: Some(5),
        score_threshold: Some(0.3),
        with_payload: true,
        with_vectors: false,
    };

    let filtered_results = search_service
        .search("programming", filtered_options)
        .await
        .expect("Filtered search should succeed");

    assert!(
        !filtered_results.results.is_empty(),
        "Should find programming-related documents"
    );

    // Test analytics
    let analytics = search_service
        .get_analytics()
        .await
        .expect("Should retrieve analytics");

    assert!(
        analytics.total_searches >= 2,
        "Should track search operations"
    );
    assert!(
        analytics.avg_response_time_ms > 0.0,
        "Should track response times"
    );

    // Clean up
    for (id, _) in &test_docs {
        storage
            .delete_vector(id)
            .await
            .expect("Failed to cleanup test document");
    }
}

/// ANCHOR: Test hybrid search service integration
/// Tests: Hybrid search combining semantic and keyword search
#[tokio::test]
async fn test_anchor_hybrid_search_integration() {
    let config = create_test_vector_config();
    let hybrid_config = HybridSearchConfig {
        semantic_weight: 0.7,
        keyword_weight: 0.3,
        fusion_method: FusionMethod::WeightedSum,
        min_semantic_score: 0.3,
        min_keyword_score: 0.1,
        max_results: 20,
        enable_query_analysis: true,
        enable_performance_tracking: true,
    };

    // Initialize services
    let embedding_service = LocalEmbeddingService::new(config.embedding.clone());
    let storage = VectorStorage::new(config.clone()).expect("Failed to create vector storage");
    let search_service = SemanticSearchService::new(
        SemanticSearchConfig::default(),
        storage.clone(),
        embedding_service.clone(),
    )
    .expect("Failed to create semantic search service");

    let hybrid_service =
        HybridSearchService::new(hybrid_config, search_service, embedding_service.clone())
            .expect("Failed to create hybrid search service");

    // Initialize embedding service
    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Prepare test documents with varied content
    let documents = vec![
        ("tech_rust", "Rust programming language features: ownership, borrowing, and lifetimes for memory safety"),
        ("tech_python", "Python programming with machine learning libraries: pandas, numpy, and scikit-learn"),
        ("business_strategy", "Strategic business planning and market analysis for technology companies"),
        ("tutorial_async", "Tutorial: Building asynchronous web applications with Rust and tokio framework"),
    ];

    // Store documents
    for (id, content) in &documents {
        let embedding = embedding_service
            .generate_embedding(content)
            .await
            .expect("Failed to generate embedding");

        let metadata = serde_json::json!({
            "id": id,
            "content": content,
            "type": if id.starts_with("tech") { "technical" } else if id.starts_with("business") { "business" } else { "tutorial" }
        });

        storage
            .store_vector(id, &embedding, Some(metadata))
            .await
            .expect("Failed to store document");
    }

    // Test hybrid search
    let request = HybridSearchRequest {
        query: "Rust programming tutorial".to_string(),
        strategy: SearchStrategy::Balanced,
        limit: 3,
        semantic_options: None,
        keyword_options: None,
        filters: None,
    };

    let results = hybrid_service
        .search(request)
        .await
        .expect("Hybrid search should succeed");

    assert!(
        !results.results.is_empty(),
        "Should find relevant documents"
    );
    assert!(
        results.explanation.is_some(),
        "Should provide search explanation"
    );
    assert!(
        results.performance_metrics.is_some(),
        "Should track performance"
    );

    // Verify hybrid scoring
    let best_result = &results.results[0];
    assert!(best_result.hybrid_score > 0.0, "Should have hybrid score");
    assert!(
        best_result.semantic_score.is_some(),
        "Should have semantic component"
    );
    assert!(
        best_result.keyword_score.is_some(),
        "Should have keyword component"
    );

    // Test different search strategies
    let semantic_focused = HybridSearchRequest {
        query: "memory safety ownership".to_string(),
        strategy: SearchStrategy::SemanticFocused,
        limit: 2,
        semantic_options: None,
        keyword_options: None,
        filters: None,
    };

    let semantic_results = hybrid_service
        .search(semantic_focused)
        .await
        .expect("Semantic-focused search should succeed");

    assert!(
        !semantic_results.results.is_empty(),
        "Should find semantically relevant documents"
    );

    let keyword_focused = HybridSearchRequest {
        query: "tutorial".to_string(),
        strategy: SearchStrategy::KeywordFocused,
        limit: 2,
        semantic_options: None,
        keyword_options: None,
        filters: None,
    };

    let keyword_results = hybrid_service
        .search(keyword_focused)
        .await
        .expect("Keyword-focused search should succeed");

    assert!(
        !keyword_results.results.is_empty(),
        "Should find keyword matches"
    );

    // Test analytics
    let analytics = hybrid_service
        .get_analytics()
        .await
        .expect("Should retrieve analytics");

    assert!(
        analytics.total_searches >= 3,
        "Should track all search operations"
    );
    assert!(
        analytics
            .strategy_usage
            .contains_key(&SearchStrategy::Balanced),
        "Should track strategy usage"
    );

    // Clean up
    for (id, _) in &documents {
        storage
            .delete_vector(id)
            .await
            .expect("Failed to cleanup test document");
    }
}

/// ANCHOR: Test error handling in vector service integration
/// Tests: Service error handling, recovery, and graceful degradation
#[tokio::test]
async fn test_anchor_vector_service_error_handling() {
    let config = create_test_vector_config();
    let embedding_service = LocalEmbeddingService::new(config.embedding.clone());

    // Test uninitialized service error
    let result = embedding_service.generate_embedding("test").await;
    assert!(result.is_err(), "Should fail when service not initialized");

    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("not initialized"),
        "Should indicate initialization required"
    );

    // Initialize service for further tests
    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Test invalid vector storage configuration
    let mut invalid_config = config.clone();
    invalid_config.url = "invalid://url".to_string();

    let storage_result = VectorStorage::new(invalid_config);
    assert!(storage_result.is_err(), "Should fail with invalid URL");

    // Test empty query handling
    let empty_result = embedding_service.generate_embedding("").await;
    assert!(
        empty_result.is_ok(),
        "Should handle empty queries gracefully"
    );

    let empty_embedding = empty_result.unwrap();
    assert_eq!(
        empty_embedding.len(),
        384,
        "Should return valid embedding for empty input"
    );

    // Test very long text handling
    let long_text = "word ".repeat(1000);
    let long_result = embedding_service.generate_embedding(&long_text).await;
    assert!(long_result.is_ok(), "Should handle long text gracefully");

    // Test batch processing with mixed valid/invalid inputs
    let mixed_batch = vec![
        "Valid query 1".to_string(),
        "".to_string(), // Empty
        "Valid query 2".to_string(),
        "word ".repeat(500), // Very long
    ];

    let batch_result = embedding_service.generate_embeddings(&mixed_batch).await;
    assert!(batch_result.is_ok(), "Should handle mixed batch gracefully");

    let embeddings = batch_result.unwrap();
    assert_eq!(embeddings.len(), 4, "Should process all inputs");

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
}

/// ANCHOR: Test vector service configuration validation
/// Tests: Configuration loading, validation, and defaults
#[tokio::test]
async fn test_anchor_vector_configuration_integration() {
    // Test default configuration
    let default_config = VectorConfig::default();
    assert_eq!(
        default_config.vector_dimensions, 384,
        "Should have correct default dimensions"
    );
    assert_eq!(
        default_config.default_collection, "research_docs",
        "Should have default collection name"
    );

    // Test configuration validation
    let valid_config = create_test_vector_config();
    let storage = VectorStorage::new(valid_config.clone());
    assert!(storage.is_ok(), "Valid configuration should be accepted");

    // Test embedding configuration
    let embedding_service = LocalEmbeddingService::new(valid_config.embedding.clone());
    embedding_service
        .initialize()
        .await
        .expect("Should initialize with valid config");

    let stats = embedding_service.get_stats().await;
    assert_eq!(stats.cache_size, 0, "Should start with empty cache");
    assert!(
        stats.total_generated >= 0,
        "Should have valid generation count"
    );

    // Test cache configuration effects
    let test_text = "Configuration test";

    // Generate embedding (should be cached)
    let embedding1 = embedding_service
        .generate_embedding(test_text)
        .await
        .expect("Should generate embedding");

    // Generate same embedding (should use cache)
    let embedding2 = embedding_service
        .generate_embedding(test_text)
        .await
        .expect("Should generate cached embedding");

    assert_eq!(
        embedding1, embedding2,
        "Cached embeddings should be identical"
    );

    let stats_after = embedding_service.get_stats().await;
    assert!(stats_after.cache_size > 0, "Cache should contain entries");

    // Test cache clearing
    embedding_service
        .clear_cache()
        .await
        .expect("Should clear cache");
    let stats_cleared = embedding_service.get_stats().await;
    assert_eq!(
        stats_cleared.cache_size, 0,
        "Cache should be empty after clearing"
    );
}
