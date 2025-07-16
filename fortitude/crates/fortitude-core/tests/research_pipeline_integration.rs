//! Integration tests for research pipeline with vector search capabilities.
//! These tests verify the enhanced research pipeline workflows using vector database integration.

use fortitude_core::{
    pipeline::{PipelineConfig, ResearchPipeline},
    research_engine::{ResearchConfig, ResearchEngine},
    research_feedback::{FeedbackCollector, FeedbackType, ResearchQuality},
    vector::{
        CacheKeyStrategy, ConnectionPoolConfig, DeviceType, DistanceMetric, DocumentMetadata, EmbeddingCacheConfig,
        EmbeddingConfig, EmbeddingGenerator, FusionMethod, HealthCheckConfig, HybridSearchConfig,
        HybridSearchOperations, HybridSearchRequest, HybridSearchService, LocalEmbeddingService,
        SearchConfig, SearchOptions, SearchStrategy, SemanticSearchConfig, SemanticSearchOperations,
        SemanticSearchService, VectorConfig, VectorStorage, VectorStorageService,
    },
};
use fortitude_types::research::{
    AudienceContext, ClassifiedRequest, ContextScore, DomainContext, ResearchAnalysis,
    ResearchResult, ResearchType,
};
use std::time::Duration;
use tokio;

/// Create test configuration for research pipeline integration
fn create_test_pipeline_config() -> (VectorConfig, ResearchConfig, PipelineConfig) {
    let vector_config = VectorConfig {
        url: "http://localhost:6334".to_string(),
        api_key: None,
        timeout: Duration::from_secs(30),
        default_collection: "test_research_pipeline".to_string(),
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
            batch_size: 8,
            device: DeviceType::Cpu,
            cache_config: EmbeddingCacheConfig {
                enabled: true,
                max_entries: 200,
                ttl: Duration::from_secs(600),
                key_strategy: CacheKeyStrategy::Hash,
            },
            ..Default::default()
        },
    };

    let research_config = ResearchConfig {
        max_concurrent_requests: 5,
        request_timeout: Duration::from_secs(60),
        cache_enabled: true,
        cache_ttl: Duration::from_secs(300),
        enable_context_discovery: true,
        enable_auto_indexing: true,
        quality_threshold: 0.7,
    };

    let pipeline_config = PipelineConfig {
        enable_classification: true,
        enable_validation: true,
        enable_caching: true,
        enable_feedback_collection: true,
        max_retries: 3,
        timeout: Duration::from_secs(120),
        quality_threshold: 0.8,
    };

    (vector_config, research_config, pipeline_config)
}

/// ANCHOR: Test enhanced research pipeline with vector context discovery
/// Tests: Context discovery, enhanced research, automatic indexing workflow
#[tokio::test]
async fn test_anchor_enhanced_research_pipeline_workflow() {
    let (vector_config, research_config, pipeline_config) = create_test_pipeline_config();

    // Initialize vector components
    let embedding_service = LocalEmbeddingService::new(vector_config.embedding.clone());
    let storage =
        VectorStorage::new_with_config(vector_config.clone()).expect("Failed to create vector storage");
    let search_service = SemanticSearchService::new(
        std::sync::Arc::new(storage.clone()),
        SemanticSearchConfig::default(),
    );

    // Initialize research pipeline components
    let research_engine =
        ResearchEngine::new(research_config).expect("Failed to create research engine");
    let mut pipeline =
        ResearchPipeline::new(pipeline_config, research_engine).expect("Failed to create pipeline");

    // Initialize embedding service
    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Seed vector database with existing research knowledge
    let knowledge_base = vec![
        (
            "rust_async_patterns",
            "Asynchronous programming patterns in Rust using tokio runtime, async/await syntax, and Future trait implementations",
            serde_json::json!({
                "category": "programming",
                "language": "rust",
                "topic": "async",
                "quality_score": 0.9
            })
        ),
        (
            "database_optimization",
            "Database query optimization techniques including indexing strategies, query planning, and performance monitoring",
            serde_json::json!({
                "category": "database",
                "topic": "optimization",
                "quality_score": 0.85
            })
        ),
        (
            "ml_deployment",
            "Machine learning model deployment strategies for production environments using containerization and orchestration",
            serde_json::json!({
                "category": "machine_learning",
                "topic": "deployment",
                "quality_score": 0.88
            })
        ),
    ];

    // Store knowledge base in vector database
    for (id, content, metadata) in &knowledge_base {
        let embedding = embedding_service
            .generate_embedding(content)
            .await
            .expect("Failed to generate knowledge embedding");

        let doc_metadata = DocumentMetadata {
            research_type: None,
            content_type: "knowledge".to_string(),
            quality_score: metadata.get("quality_score").and_then(|v| v.as_f64()),
            source: Some("test".to_string()),
            tags: vec![],
            custom_fields: std::collections::HashMap::new(),
        };
        storage
            .store_document(content, doc_metadata)
            .await
            .expect("Failed to store knowledge");
    }

    // Create research request
    let research_request = ClassifiedRequest::new(
        "How to implement efficient async database operations in Rust?".to_string(),
        ResearchType::Implementation,
        AudienceContext::TechnicalExpert,
        DomainContext::SoftwareDevelopment,
        0.92,
        vec![
            "rust".to_string(),
            "async".to_string(),
            "database".to_string(),
        ],
    );

    // Step 1: Context Discovery
    let query_embedding = embedding_service
        .generate_embedding(&research_request.original_query)
        .await
        .expect("Failed to generate query embedding");

    let context_results = search_service
        .search_with_options(
            &research_request.original_query,
            SearchOptions {
                limit: Some(5),
                score_threshold: Some(0.6),
                with_payload: true,
                with_vectors: false,
            },
        )
        .await
        .expect("Failed to discover context");

    assert!(
        !context_results.results.is_empty(),
        "Should discover relevant context"
    );

    // Verify context relevance
    let relevant_context = context_results
        .results
        .iter()
        .filter(|result| result.score > 0.7)
        .count();
    assert!(relevant_context > 0, "Should find highly relevant context");

    // Step 2: Enhanced Research with Context
    let enhanced_request = research_request.clone().with_context(
        context_results
            .results
            .iter()
            .map(|result| format!("Context: {}", result.id))
            .collect::<Vec<_>>()
            .join("; "),
    );

    // Process through pipeline
    let pipeline_result = pipeline
        .process_request(enhanced_request)
        .await
        .expect("Pipeline should process request successfully");

    assert!(pipeline_result.is_success(), "Pipeline should succeed");
    assert!(
        pipeline_result.research_result.is_some(),
        "Should produce research result"
    );

    let research_result = pipeline_result.research_result.unwrap();
    assert!(
        !research_result.content.is_empty(),
        "Should generate research content"
    );
    assert!(
        research_result.quality_score > 0.7,
        "Should meet quality threshold"
    );

    // Step 3: Automatic Indexing
    let result_embedding = embedding_service
        .generate_embedding(&research_result.content)
        .await
        .expect("Failed to generate result embedding");

    let result_metadata = serde_json::json!({
        "query": research_request.original_query,
        "research_type": format!("{:?}", research_request.research_type),
        "quality_score": research_result.quality_score,
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "context_used": true
    });

    let result_id = format!("research_result_{}", uuid::Uuid::new_v4());
    storage
        .store_vector(&result_id, &result_embedding, Some(result_metadata))
        .await
        .expect("Failed to auto-index research result");

    // Verify indexing
    let stored_result = storage
        .get_vector(&result_id)
        .await
        .expect("Failed to retrieve indexed result");
    assert!(stored_result.is_some(), "Result should be indexed");

    // Step 4: Verify Context Enhancement
    // Search for similar queries should now return our new result
    let similar_query = "async database programming in Rust";
    let enhanced_search = search_service
        .search(
            similar_query,
            SearchOptions {
                limit: Some(3),
                score_threshold: Some(0.5),
                with_payload: true,
                with_vectors: false,
            },
        )
        .await
        .expect("Failed to search enhanced knowledge base");

    let found_our_result = enhanced_search
        .results
        .iter()
        .any(|result| result.id == result_id);
    assert!(
        found_our_result,
        "New research result should be discoverable"
    );

    // Clean up
    for (id, _, _) in &knowledge_base {
        storage
            .delete_vector(id)
            .await
            .expect("Failed to cleanup knowledge");
    }
    storage
        .delete_vector(&result_id)
        .await
        .expect("Failed to cleanup result");
}

/// ANCHOR: Test research pipeline with hybrid search integration
/// Tests: Hybrid search for context discovery, multi-strategy research enhancement
#[tokio::test]
async fn test_anchor_research_pipeline_hybrid_search() {
    let (vector_config, research_config, pipeline_config) = create_test_pipeline_config();

    // Initialize vector components with hybrid search
    let embedding_service = LocalEmbeddingService::new(vector_config.embedding.clone());
    let storage =
        VectorStorage::new_with_config(vector_config.clone()).expect("Failed to create vector storage");
    let semantic_search = SemanticSearchService::new(
        SemanticSearchConfig::default(),
        storage.clone(),
        embedding_service.clone(),
    )
    .expect("Failed to create semantic search");

    let hybrid_config = HybridSearchConfig {
        semantic_weight: 0.6,
        keyword_weight: 0.4,
        fusion_method: FusionMethod::WeightedSum,
        min_semantic_score: 0.4,
        min_keyword_score: 0.2,
        max_results: 10,
        enable_query_analysis: true,
        enable_performance_tracking: true,
    };

    let hybrid_search =
        HybridSearchService::new(hybrid_config, semantic_search, embedding_service.clone())
            .expect("Failed to create hybrid search");

    // Initialize services
    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Create diverse knowledge base for hybrid search testing
    let diverse_knowledge = vec![
        (
            "rust_performance",
            "Rust performance optimization: zero-cost abstractions, memory efficiency, and compile-time guarantees",
            serde_json::json!({
                "keywords": ["rust", "performance", "optimization", "memory"],
                "category": "performance",
                "difficulty": "advanced"
            })
        ),
        (
            "web_apis",
            "RESTful API design principles and best practices for scalable web services",
            serde_json::json!({
                "keywords": ["api", "rest", "web", "services", "design"],
                "category": "web_development",
                "difficulty": "intermediate"
            })
        ),
        (
            "testing_strategies",
            "Software testing strategies: unit testing, integration testing, and test-driven development",
            serde_json::json!({
                "keywords": ["testing", "unit", "integration", "tdd"],
                "category": "testing",
                "difficulty": "beginner"
            })
        ),
    ];

    // Store diverse knowledge
    for (id, content, metadata) in &diverse_knowledge {
        let embedding = embedding_service
            .generate_embedding(content)
            .await
            .expect("Failed to generate embedding");

        let doc_metadata = DocumentMetadata {
            research_type: None,
            content_type: "knowledge".to_string(),
            quality_score: metadata.get("quality_score").and_then(|v| v.as_f64()),
            source: Some("test".to_string()),
            tags: vec![],
            custom_fields: std::collections::HashMap::new(),
        };
        storage
            .store_document(content, doc_metadata)
            .await
            .expect("Failed to store knowledge");
    }

    // Test hybrid search for context discovery
    let research_queries = vec![
        (
            "How to optimize Rust code performance?",
            SearchStrategy::SemanticFocused,
        ),
        ("API testing best practices", SearchStrategy::KeywordFocused),
        ("web service optimization", SearchStrategy::Balanced),
    ];

    for (query, strategy) in research_queries {
        let request = HybridSearchRequest {
            query: query.to_string(),
            strategy,
            limit: 3,
            semantic_options: None,
            keyword_options: None,
            filters: None,
        };

        let results = hybrid_search
            .search(request)
            .await
            .expect("Hybrid search should succeed");

        assert!(
            !results.results.is_empty(),
            "Should find relevant context for: {}",
            query
        );

        // Verify hybrid scoring
        for result in &results.results {
            assert!(result.hybrid_score > 0.0, "Should have hybrid score");
            assert!(
                result.semantic_score.is_some(),
                "Should have semantic component"
            );
            assert!(
                result.keyword_score.is_some(),
                "Should have keyword component"
            );
        }

        // Verify strategy-appropriate results
        match strategy {
            SearchStrategy::SemanticFocused => {
                let best_result = &results.results[0];
                assert!(
                    best_result.semantic_score.unwrap() > 0.4,
                    "Semantic score should be significant"
                );
            }
            SearchStrategy::KeywordFocused => {
                let has_keyword_matches = results
                    .results
                    .iter()
                    .any(|r| r.keyword_score.unwrap() > 0.2);
                assert!(has_keyword_matches, "Should have strong keyword matches");
            }
            SearchStrategy::Balanced => {
                let best_result = &results.results[0];
                assert!(
                    best_result.hybrid_score > 0.3,
                    "Balanced score should be reasonable"
                );
            }
        }
    }

    // Test research pipeline with hybrid context discovery
    let research_request = ClassifiedRequest::new(
        "Rust API testing optimization techniques".to_string(),
        ResearchType::BestPractices,
        AudienceContext::TechnicalExpert,
        DomainContext::SoftwareDevelopment,
        0.88,
        vec!["rust".to_string(), "api".to_string(), "testing".to_string()],
    );

    // Use hybrid search for comprehensive context discovery
    let context_request = HybridSearchRequest {
        query: research_request.original_query.clone(),
        strategy: SearchStrategy::Balanced,
        limit: 5,
        semantic_options: None,
        keyword_options: None,
        filters: None,
    };

    let hybrid_context = hybrid_search
        .search(context_request)
        .await
        .expect("Failed to discover hybrid context");

    assert!(
        !hybrid_context.results.is_empty(),
        "Should discover comprehensive context"
    );

    // Verify context diversity (should find multiple categories)
    let categories: std::collections::HashSet<String> = hybrid_context
        .results
        .iter()
        .filter_map(|result| result.metadata.as_ref())
        .filter_map(|metadata| metadata.get("category"))
        .filter_map(|cat| cat.as_str())
        .map(|s| s.to_string())
        .collect();

    assert!(
        categories.len() > 1,
        "Should discover diverse context categories"
    );

    // Clean up
    for (id, _, _) in &diverse_knowledge {
        storage
            .delete_vector(id)
            .await
            .expect("Failed to cleanup knowledge");
    }
}

/// ANCHOR: Test research quality feedback integration with vector search
/// Tests: Quality feedback collection, research improvement, knowledge base refinement
#[tokio::test]
async fn test_anchor_research_quality_feedback_integration() {
    let (vector_config, research_config, pipeline_config) = create_test_pipeline_config();

    // Initialize components
    let embedding_service = LocalEmbeddingService::new(vector_config.embedding.clone());
    let storage =
        VectorStorage::new_with_config(vector_config.clone()).expect("Failed to create vector storage");
    let search_service = SemanticSearchService::new(
        std::sync::Arc::new(storage.clone()),
        SemanticSearchConfig::default(),
    );

    let research_engine =
        ResearchEngine::new(research_config).expect("Failed to create research engine");
    let mut pipeline =
        ResearchPipeline::new(pipeline_config, research_engine).expect("Failed to create pipeline");

    let mut feedback_collector = FeedbackCollector::new();

    // Initialize services
    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Create initial knowledge base
    let initial_knowledge = vec![
        (
            "low_quality_content",
            "Basic information about programming concepts without depth or examples",
            serde_json::json!({
                "quality_score": 0.3,
                "feedback_count": 0,
                "avg_feedback_score": 0.0
            })
        ),
        (
            "high_quality_content",
            "Comprehensive guide to advanced programming patterns with detailed examples, best practices, and performance considerations",
            serde_json::json!({
                "quality_score": 0.9,
                "feedback_count": 5,
                "avg_feedback_score": 4.2
            })
        ),
    ];

    // Store initial knowledge
    for (id, content, metadata) in &initial_knowledge {
        let embedding = embedding_service
            .generate_embedding(content)
            .await
            .expect("Failed to generate embedding");

        let doc_metadata = DocumentMetadata {
            research_type: None,
            content_type: "knowledge".to_string(),
            quality_score: metadata.get("quality_score").and_then(|v| v.as_f64()),
            source: Some("test".to_string()),
            tags: vec![],
            custom_fields: std::collections::HashMap::new(),
        };
        storage
            .store_document(content, doc_metadata)
            .await
            .expect("Failed to store knowledge");
    }

    // Process research request
    let research_request = ClassifiedRequest::new(
        "Advanced programming design patterns".to_string(),
        ResearchType::ComprehensiveAnalysis,
        AudienceContext::TechnicalExpert,
        DomainContext::SoftwareDevelopment,
        0.85,
        vec!["programming".to_string(), "patterns".to_string()],
    );

    let pipeline_result = pipeline
        .process_request(research_request.clone())
        .await
        .expect("Pipeline should process request");

    assert!(
        pipeline_result.research_result.is_some(),
        "Should produce research result"
    );
    let research_result = pipeline_result.research_result.unwrap();

    // Collect positive feedback
    feedback_collector
        .collect_feedback(
            research_request.original_query.clone(),
            FeedbackType::Quality(ResearchQuality::Excellent),
            Some(
                "Very comprehensive and well-structured response with practical examples"
                    .to_string(),
            ),
            5.0,
        )
        .expect("Should collect positive feedback");

    feedback_collector
        .collect_feedback(
            research_request.original_query.clone(),
            FeedbackType::Relevance,
            Some("Highly relevant to the question asked".to_string()),
            4.8,
        )
        .expect("Should collect relevance feedback");

    // Store research result with feedback
    let result_embedding = embedding_service
        .generate_embedding(&research_result.content)
        .await
        .expect("Failed to generate result embedding");

    let feedback_summary = feedback_collector
        .get_feedback_summary(&research_request.original_query)
        .expect("Should have feedback summary");

    let enhanced_metadata = serde_json::json!({
        "query": research_request.original_query,
        "quality_score": research_result.quality_score,
        "feedback_count": feedback_summary.total_feedback,
        "avg_feedback_score": feedback_summary.average_score,
        "has_positive_feedback": feedback_summary.average_score > 4.0,
        "research_type": format!("{:?}", research_request.research_type)
    });

    let result_id = format!("research_with_feedback_{}", uuid::Uuid::new_v4());
    storage
        .store_vector(&result_id, &result_embedding, Some(enhanced_metadata))
        .await
        .expect("Failed to store research with feedback");

    // Test feedback-informed search
    // Should prioritize high-quality, well-rated content
    let feedback_query = "programming design patterns guide";
    let search_results = search_service
        .search(
            feedback_query,
            SearchOptions {
                limit: Some(5),
                score_threshold: Some(0.3),
                with_payload: true,
                with_vectors: false,
            },
        )
        .await
        .expect("Failed to search with feedback consideration");

    assert!(!search_results.results.is_empty(), "Should find results");

    // Verify that high-quality content is prioritized
    let high_quality_results = search_results
        .results
        .iter()
        .filter(|result| {
            result
                .metadata
                .as_ref()
                .and_then(|m| m.get("quality_score"))
                .and_then(|s| s.as_f64())
                .map(|score| score > 0.7)
                .unwrap_or(false)
        })
        .count();

    assert!(
        high_quality_results > 0,
        "Should prioritize high-quality content"
    );

    // Test negative feedback handling
    let poor_request = ClassifiedRequest::new(
        "Basic programming info".to_string(),
        ResearchType::QuickOverview,
        AudienceContext::Beginner,
        DomainContext::General,
        0.5,
        vec!["programming".to_string()],
    );

    feedback_collector
        .collect_feedback(
            poor_request.original_query.clone(),
            FeedbackType::Quality(ResearchQuality::Poor),
            Some("Too basic and lacking detail".to_string()),
            2.0,
        )
        .expect("Should collect negative feedback");

    // Verify feedback analytics
    let analytics = feedback_collector
        .get_analytics()
        .expect("Should provide feedback analytics");

    assert!(analytics.total_feedback >= 3, "Should track all feedback");
    assert!(
        analytics
            .quality_distribution
            .contains_key(&ResearchQuality::Excellent),
        "Should track quality distribution"
    );
    assert!(
        analytics.average_score > 0.0,
        "Should calculate average scores"
    );

    // Clean up
    for (id, _, _) in &initial_knowledge {
        storage
            .delete_vector(id)
            .await
            .expect("Failed to cleanup knowledge");
    }
    storage
        .delete_vector(&result_id)
        .await
        .expect("Failed to cleanup result");
}

/// ANCHOR: Test research pipeline performance with vector operations
/// Tests: Pipeline performance, caching effectiveness, concurrent request handling
#[tokio::test]
async fn test_anchor_research_pipeline_performance_integration() {
    let (vector_config, research_config, pipeline_config) = create_test_pipeline_config();

    // Initialize components with performance tracking
    let embedding_service = LocalEmbeddingService::new(vector_config.embedding.clone());
    let storage =
        VectorStorage::new_with_config(vector_config.clone()).expect("Failed to create vector storage");
    let search_service = SemanticSearchService::new(
        SemanticSearchConfig {
            enable_explain: true,
            cache_enabled: true,
            cache_ttl: Duration::from_secs(300),
            ..Default::default()
        },
        storage.clone(),
        embedding_service.clone(),
    )
    .expect("Failed to create search service");

    let research_engine =
        ResearchEngine::new(research_config).expect("Failed to create research engine");
    let mut pipeline =
        ResearchPipeline::new(pipeline_config, research_engine).expect("Failed to create pipeline");

    // Initialize services
    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Create performance test knowledge base
    let knowledge_items: Vec<_> = (0..20)
        .map(|i| {
            let content = format!(
                "Performance test knowledge item {}: detailed technical content about software development, \
                optimization techniques, and best practices for building scalable applications",
                i
            );
            let metadata = serde_json::json!({
                "item_id": i,
                "category": if i % 3 == 0 { "performance" } else if i % 3 == 1 { "development" } else { "optimization" },
                "complexity": if i % 2 == 0 { "high" } else { "medium" }
            });
            (format!("perf_item_{}", i), content, metadata)
        })
        .collect();

    // Store knowledge base
    let start_time = std::time::Instant::now();
    for (id, content, metadata) in &knowledge_items {
        let embedding = embedding_service
            .generate_embedding(content)
            .await
            .expect("Failed to generate embedding");

        let doc_metadata = DocumentMetadata {
            research_type: None,
            content_type: "knowledge".to_string(),
            quality_score: metadata.get("quality_score").and_then(|v| v.as_f64()),
            source: Some("test".to_string()),
            tags: vec![],
            custom_fields: std::collections::HashMap::new(),
        };
        storage
            .store_document(content, doc_metadata)
            .await
            .expect("Failed to store knowledge");
    }
    let storage_duration = start_time.elapsed();

    println!(
        "Stored {} items in {:?}",
        knowledge_items.len(),
        storage_duration
    );
    assert!(
        storage_duration.as_millis() < 10000,
        "Storage should be reasonably fast"
    );

    // Test concurrent research requests
    let concurrent_requests: Vec<_> = (0..5)
        .map(|i| {
            ClassifiedRequest::new(
                format!("How to optimize software performance aspect {}?", i),
                ResearchType::Implementation,
                AudienceContext::TechnicalExpert,
                DomainContext::SoftwareDevelopment,
                0.8,
                vec!["optimization".to_string(), "performance".to_string()],
            )
        })
        .collect();

    let concurrent_start = std::time::Instant::now();
    let mut handles = Vec::new();

    for request in concurrent_requests {
        let pipeline_clone = pipeline.clone();
        let handle = tokio::spawn(async move { pipeline_clone.process_request(request).await });
        handles.push(handle);
    }

    // Wait for all concurrent requests
    let mut successful_results = 0;
    for handle in handles {
        let result = handle.await.expect("Task should complete");
        if result.is_ok() && result.unwrap().is_success() {
            successful_results += 1;
        }
    }

    let concurrent_duration = concurrent_start.elapsed();
    println!(
        "Processed {} concurrent requests in {:?}",
        successful_results, concurrent_duration
    );

    assert_eq!(
        successful_results, 5,
        "All concurrent requests should succeed"
    );
    assert!(
        concurrent_duration.as_secs() < 30,
        "Concurrent processing should be efficient"
    );

    // Test caching effectiveness
    let cache_test_query = "software optimization techniques";

    // First search (cache miss)
    let first_search_start = std::time::Instant::now();
    let first_result = search_service
        .search(
            cache_test_query,
            SearchOptions {
                limit: Some(3),
                score_threshold: Some(0.4),
                with_payload: true,
                with_vectors: false,
            },
        )
        .await
        .expect("First search should succeed");
    let first_duration = first_search_start.elapsed();

    // Second search (cache hit)
    let second_search_start = std::time::Instant::now();
    let second_result = search_service
        .search(
            cache_test_query,
            SearchOptions {
                limit: Some(3),
                score_threshold: Some(0.4),
                with_payload: true,
                with_vectors: false,
            },
        )
        .await
        .expect("Second search should succeed");
    let second_duration = second_search_start.elapsed();

    assert_eq!(
        first_result.results.len(),
        second_result.results.len(),
        "Cache should return identical results"
    );

    // Cache should make second search faster (though in our mock implementation, this might not be significant)
    println!(
        "First search: {:?}, Second search: {:?}",
        first_duration, second_duration
    );

    // Test analytics and performance metrics
    let search_analytics = search_service
        .get_analytics()
        .await
        .expect("Should provide search analytics");

    assert!(
        search_analytics.total_searches >= 2,
        "Should track search operations"
    );
    assert!(
        search_analytics.avg_response_time_ms > 0.0,
        "Should track response times"
    );

    let embedding_stats = embedding_service.get_stats().await;
    assert!(
        embedding_stats.total_generated > 20,
        "Should track embedding generation"
    );
    assert!(embedding_stats.cache_size > 0, "Should use embedding cache");

    // Clean up
    for (id, _, _) in &knowledge_items {
        storage
            .delete_vector(id)
            .await
            .expect("Failed to cleanup knowledge");
    }
}
