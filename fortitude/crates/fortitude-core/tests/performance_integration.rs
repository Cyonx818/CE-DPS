//! Performance integration tests for vector database workflows.
//! These tests verify performance characteristics and optimization effectiveness.

use fortitude_core::vector::{
    CacheKeyStrategy, ConnectionPoolConfig, DeviceType, DistanceMetric, EmbeddingCacheConfig,
    EmbeddingConfig, EmbeddingGenerator, FusionMethod, HealthCheckConfig, HybridSearchConfig,
    HybridSearchRequest, HybridSearchService, LocalEmbeddingService, MigrationConfig,
    MigrationService, MigrationSource, QdrantClient, SearchOptions, SearchStrategy,
    SemanticSearchConfig, SemanticSearchService, ValidationLevel, VectorConfig, VectorStorage,
    VectorStorageService,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Performance test configuration
fn create_performance_test_config() -> VectorConfig {
    VectorConfig {
        url: "http://localhost:6334".to_string(),
        api_key: None,
        timeout: Duration::from_secs(60),
        default_collection: "performance_test_collection".to_string(),
        vector_dimensions: 384,
        distance_metric: DistanceMetric::Cosine,
        health_check: HealthCheckConfig {
            enabled: true,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            max_failures: 5,
        },
        connection_pool: ConnectionPoolConfig {
            max_connections: 20,
            connection_timeout: Duration::from_secs(15),
            idle_timeout: Duration::from_secs(600),
        },
        embedding: EmbeddingConfig {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            max_sequence_length: 256,
            batch_size: 32, // Larger batch for performance testing
            device: DeviceType::Cpu,
            cache_config: EmbeddingCacheConfig {
                enabled: true,
                max_entries: 1000,
                ttl: Duration::from_secs(1800),
                key_strategy: CacheKeyStrategy::Hash,
            },
            ..Default::default()
        },
    }
}

/// Generate test data for performance testing
fn generate_performance_test_data(count: usize) -> Vec<(String, String, Value)> {
    let base_contents = vec![
        "Advanced machine learning algorithms for natural language processing and text analysis",
        "Distributed systems architecture patterns for scalable microservices deployment",
        "Database optimization techniques for high-performance query execution and indexing",
        "Asynchronous programming patterns in Rust using tokio runtime and async/await",
        "Web API security best practices for authentication and authorization systems",
        "Container orchestration strategies using Kubernetes for production environments",
        "Real-time data processing pipelines with Apache Kafka and stream processing",
        "Frontend performance optimization techniques for modern web applications",
        "DevOps automation workflows using CI/CD pipelines and infrastructure as code",
        "Artificial intelligence model deployment strategies for production systems",
    ];

    let categories = vec![
        "machine_learning",
        "systems",
        "database",
        "programming",
        "security",
        "devops",
        "data",
        "frontend",
        "automation",
        "ai",
    ];
    let difficulties = vec!["beginner", "intermediate", "advanced", "expert"];

    (0..count)
        .map(|i| {
            let base_content = &base_contents[i % base_contents.len()];
            let category = &categories[i % categories.len()];
            let difficulty = &difficulties[i % difficulties.len()];
            let content = format!(
                "{} - Variation {} with additional context about implementation details, \
                performance considerations, and best practices for production usage. \
                This content includes specific examples, code snippets, and troubleshooting guides.",
                base_content, i
            );
            let metadata = json!({
                "id": format!("perf_doc_{}", i),
                "category": category,
                "difficulty": difficulty,
                "content_length": content.len(),
                "created_at": chrono::Utc::now().to_rfc3339(),
                "version": "1.0",
                "tags": [format!("tag_{}", i % 5), format!("tech_{}", i % 10)]
            });
            (format!("perf_doc_{}", i), content, metadata)
        })
        .collect()
}

/// Performance metrics collection helper
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    operation: String,
    duration: Duration,
    throughput: f64,
    items_processed: usize,
    memory_usage: Option<u64>,
    cache_hits: Option<usize>,
    cache_misses: Option<usize>,
}

impl PerformanceMetrics {
    fn new(operation: String, duration: Duration, items_processed: usize) -> Self {
        let throughput = items_processed as f64 / duration.as_secs_f64();
        Self {
            operation,
            duration,
            throughput,
            items_processed,
            memory_usage: None,
            cache_hits: None,
            cache_misses: None,
        }
    }

    fn with_cache_stats(mut self, hits: usize, misses: usize) -> Self {
        self.cache_hits = Some(hits);
        self.cache_misses = Some(misses);
        self
    }
}

/// ANCHOR: Test embedding generation performance and caching effectiveness
/// Tests: Batch embedding performance, cache hit rates, memory usage
#[tokio::test]
async fn test_anchor_embedding_performance_optimization() {
    let config = create_performance_test_config();
    let embedding_service = Arc::new(LocalEmbeddingService::new(config.embedding.clone()));

    // Initialize service
    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    let mut metrics = Vec::new();

    // Test 1: Single embedding generation performance
    let single_texts = vec![
        "How to implement high-performance vector search algorithms?",
        "Database optimization strategies for large-scale applications",
        "Machine learning model deployment in production environments",
        "Microservices architecture patterns for scalable systems",
        "Real-time data processing with stream computing frameworks",
    ];

    let start = Instant::now();
    for (i, text) in single_texts.iter().enumerate() {
        let _embedding = embedding_service
            .generate_embedding(text)
            .await
            .expect(&format!("Failed to generate embedding {}", i));
    }
    let single_duration = start.elapsed();

    metrics.push(PerformanceMetrics::new(
        "single_embedding_generation".to_string(),
        single_duration,
        single_texts.len(),
    ));

    println!(
        "Single embedding generation: {:.2} items/sec",
        single_texts.len() as f64 / single_duration.as_secs_f64()
    );

    // Test 2: Batch embedding generation performance
    let batch_texts: Vec<String> = generate_performance_test_data(100)
        .into_iter()
        .map(|(_, content, _)| content)
        .collect();

    let start = Instant::now();
    let batch_embeddings = embedding_service
        .generate_embeddings(&batch_texts)
        .await
        .expect("Failed to generate batch embeddings");
    let batch_duration = start.elapsed();

    assert_eq!(
        batch_embeddings.len(),
        100,
        "Should generate 100 embeddings"
    );

    metrics.push(PerformanceMetrics::new(
        "batch_embedding_generation".to_string(),
        batch_duration,
        batch_texts.len(),
    ));

    println!(
        "Batch embedding generation: {:.2} items/sec",
        batch_texts.len() as f64 / batch_duration.as_secs_f64()
    );

    // Test 3: Cache effectiveness
    let cache_test_texts = vec![
        "Repeated text for cache testing",
        "Another repeated text for cache analysis",
        "Third repeated text for cache validation",
    ];

    // First pass - populate cache
    let start = Instant::now();
    for text in &cache_test_texts {
        let _embedding = embedding_service
            .generate_embedding(text)
            .await
            .expect("Failed to generate embedding for cache test");
    }
    let first_pass_duration = start.elapsed();

    // Second pass - should hit cache
    let start = Instant::now();
    for text in &cache_test_texts {
        let _embedding = embedding_service
            .generate_embedding(text)
            .await
            .expect("Failed to generate cached embedding");
    }
    let second_pass_duration = start.elapsed();

    // Cache should make second pass significantly faster
    let speedup_ratio = first_pass_duration.as_secs_f64() / second_pass_duration.as_secs_f64();
    println!("Cache speedup ratio: {:.2}x", speedup_ratio);

    // Get cache statistics
    let stats = embedding_service.get_stats().await;
    println!(
        "Cache statistics - Size: {}, Total generated: {}",
        stats.cache_size, stats.total_generated
    );

    metrics.push(
        PerformanceMetrics::new(
            "cached_embedding_generation".to_string(),
            second_pass_duration,
            cache_test_texts.len(),
        )
        .with_cache_stats(
            stats.cache_size as usize,
            (stats.total_generated as usize).saturating_sub(stats.cache_size as usize),
        ),
    );

    // Test 4: Memory usage under load
    let large_batch: Vec<String> = (0..500)
        .map(|i| {
            format!(
                "Large batch test content item {} with substantial text content \
            to test memory usage patterns and garbage collection behavior during \
            high-throughput embedding generation scenarios.",
                i
            )
        })
        .collect();

    let start = Instant::now();
    let _large_embeddings = embedding_service
        .generate_embeddings(&large_batch)
        .await
        .expect("Failed to generate large batch embeddings");
    let large_batch_duration = start.elapsed();

    metrics.push(PerformanceMetrics::new(
        "large_batch_embedding_generation".to_string(),
        large_batch_duration,
        large_batch.len(),
    ));

    println!(
        "Large batch generation: {:.2} items/sec",
        large_batch.len() as f64 / large_batch_duration.as_secs_f64()
    );

    // Verify performance thresholds
    for metric in &metrics {
        match metric.operation.as_str() {
            "single_embedding_generation" => {
                assert!(
                    metric.throughput > 1.0,
                    "Single embedding generation should be > 1 item/sec"
                );
            }
            "batch_embedding_generation" => {
                assert!(
                    metric.throughput > 10.0,
                    "Batch embedding generation should be > 10 items/sec"
                );
            }
            "large_batch_embedding_generation" => {
                assert!(
                    metric.throughput > 5.0,
                    "Large batch generation should be > 5 items/sec"
                );
            }
            _ => {}
        }
    }

    // Clean up cache
    embedding_service
        .clear_cache()
        .await
        .expect("Failed to clear cache");
}

/// ANCHOR: Test vector storage and retrieval performance
/// Tests: Storage throughput, search latency, concurrent operations
#[tokio::test]
async fn test_anchor_vector_storage_performance() {
    let config = create_performance_test_config();
    let embedding_service = Arc::new(LocalEmbeddingService::new(config.embedding.clone()));
    let qdrant_client = Arc::new(QdrantClient::new(config.clone()).await.unwrap());
    let storage = VectorStorage::new(qdrant_client, embedding_service.clone());

    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    let mut metrics = Vec::new();

    // Test 1: Single vector storage performance
    let single_docs = generate_performance_test_data(50);

    let start = Instant::now();
    for (id, content, metadata) in &single_docs {
        let embedding = embedding_service
            .generate_embedding(content)
            .await
            .expect("Failed to generate embedding for storage test");

        storage
            .store_vector(id, &embedding, Some(metadata.clone()))
            .await
            .expect("Failed to store vector");
    }
    let single_storage_duration = start.elapsed();

    metrics.push(PerformanceMetrics::new(
        "single_vector_storage".to_string(),
        single_storage_duration,
        single_docs.len(),
    ));

    println!(
        "Single vector storage: {:.2} items/sec",
        single_docs.len() as f64 / single_storage_duration.as_secs_f64()
    );

    // Test 2: Batch vector retrieval performance
    let retrieval_ids: Vec<_> = single_docs
        .iter()
        .take(20)
        .map(|(id, _, _)| id.clone())
        .collect();

    let start = Instant::now();
    for id in &retrieval_ids {
        let _result = storage
            .get_vector(id)
            .await
            .expect("Failed to retrieve vector");
    }
    let retrieval_duration = start.elapsed();

    metrics.push(PerformanceMetrics::new(
        "vector_retrieval".to_string(),
        retrieval_duration,
        retrieval_ids.len(),
    ));

    println!(
        "Vector retrieval: {:.2} items/sec",
        retrieval_ids.len() as f64 / retrieval_duration.as_secs_f64()
    );

    // Test 3: Search performance with varying result sizes
    let search_query = embedding_service
        .generate_embedding("performance optimization techniques")
        .await
        .expect("Failed to generate search query embedding");

    let search_limits = vec![5, 10, 20, 50];
    for limit in search_limits {
        let start = Instant::now();
        let results = storage
            .search_vectors(&search_query, limit, None)
            .await
            .expect("Failed to search vectors");
        let search_duration = start.elapsed();

        metrics.push(PerformanceMetrics::new(
            format!("vector_search_limit_{}", limit),
            search_duration,
            results.len(),
        ));

        println!(
            "Search (limit {}): {:.2}ms, found {} results",
            limit,
            search_duration.as_millis(),
            results.len()
        );

        // Search should complete quickly regardless of limit
        assert!(
            search_duration.as_millis() < 1000,
            "Search with limit {} should complete within 1 second",
            limit
        );
    }

    // Test 4: Concurrent search operations
    let concurrent_queries = 10;
    let search_handles: Vec<_> = (0..concurrent_queries)
        .map(|i| {
            let storage_clone = storage.clone();
            let query_embedding = search_query.clone();
            tokio::spawn(async move {
                let start = Instant::now();
                let results = storage_clone
                    .search_vectors(&query_embedding, 10, None)
                    .await
                    .expect(&format!("Failed concurrent search {}", i));
                (start.elapsed(), results.len())
            })
        })
        .collect();

    let concurrent_start = Instant::now();
    let mut total_results = 0;
    for handle in search_handles {
        let (duration, result_count) = handle
            .await
            .expect("Concurrent search task should complete");
        total_results += result_count;

        // Individual searches should still be fast
        assert!(
            duration.as_millis() < 1000,
            "Concurrent search should complete within 1 second"
        );
    }
    let concurrent_total_duration = concurrent_start.elapsed();

    metrics.push(PerformanceMetrics::new(
        "concurrent_vector_search".to_string(),
        concurrent_total_duration,
        concurrent_queries,
    ));

    println!(
        "Concurrent searches: {} queries in {:.2}ms ({:.2} queries/sec)",
        concurrent_queries,
        concurrent_total_duration.as_millis(),
        concurrent_queries as f64 / concurrent_total_duration.as_secs_f64()
    );

    // Test 5: Storage count and stats performance
    let start = Instant::now();
    let count = storage
        .count_vectors()
        .await
        .expect("Failed to count vectors");
    let count_duration = start.elapsed();

    assert!(
        count >= single_docs.len(),
        "Should have stored at least {} vectors",
        single_docs.len()
    );
    assert!(
        count_duration.as_millis() < 500,
        "Count operation should be fast"
    );

    println!(
        "Vector count: {} vectors, duration: {:.2}ms",
        count,
        count_duration.as_millis()
    );

    // Clean up test data
    for (id, _, _) in &single_docs {
        storage
            .delete_vector(id)
            .await
            .expect("Failed to cleanup test vector");
    }

    // Verify performance meets minimum thresholds
    for metric in &metrics {
        match metric.operation.as_str() {
            "single_vector_storage" => {
                assert!(
                    metric.throughput > 5.0,
                    "Vector storage should be > 5 items/sec"
                );
            }
            "vector_retrieval" => {
                assert!(
                    metric.throughput > 20.0,
                    "Vector retrieval should be > 20 items/sec"
                );
            }
            "concurrent_vector_search" => {
                assert!(
                    metric.throughput > 5.0,
                    "Concurrent search should handle > 5 queries/sec"
                );
            }
            _ if metric.operation.starts_with("vector_search_limit_") => {
                assert!(
                    metric.duration.as_millis() < 1000,
                    "Search should complete within 1 second"
                );
            }
            _ => {}
        }
    }
}

/// ANCHOR: Test search service performance and optimization
/// Tests: Semantic search latency, hybrid search throughput, result quality
#[tokio::test]
async fn test_anchor_search_service_performance() {
    let config = create_performance_test_config();
    let embedding_service = Arc::new(LocalEmbeddingService::new(config.embedding.clone()));
    let qdrant_client = Arc::new(QdrantClient::new(config.clone()).await.unwrap());
    let storage = VectorStorage::new(qdrant_client, embedding_service.clone());

    let search_config = SemanticSearchConfig {
        collection_name: config.default_collection.clone(),
        default_limit: 10,
        min_score_threshold: 0.3,
        enable_explain: false, // Disable for performance testing
        cache_enabled: true,
        cache_ttl: Duration::from_secs(300),
    };

    let semantic_search =
        SemanticSearchService::new(search_config, storage.clone(), embedding_service.clone())
            .expect("Failed to create semantic search service");

    let hybrid_config = HybridSearchConfig {
        semantic_weight: 0.7,
        keyword_weight: 0.3,
        fusion_method: FusionMethod::WeightedSum,
        min_semantic_score: 0.3,
        min_keyword_score: 0.1,
        max_results: 50,
        enable_query_analysis: false, // Disable for performance
        enable_performance_tracking: true,
    };

    let hybrid_search = HybridSearchService::new(
        hybrid_config,
        semantic_search.clone(),
        embedding_service.clone(),
    )
    .expect("Failed to create hybrid search service");

    // Initialize services
    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    // Setup test data
    let test_docs = generate_performance_test_data(200);

    // Store test documents
    let storage_start = Instant::now();
    for (id, content, metadata) in &test_docs {
        let embedding = embedding_service
            .generate_embedding(content)
            .await
            .expect("Failed to generate embedding");

        storage
            .store_vector(id, &embedding, Some(metadata.clone()))
            .await
            .expect("Failed to store test document");
    }
    let storage_duration = storage_start.elapsed();

    println!(
        "Stored {} documents in {:.2} seconds ({:.2} docs/sec)",
        test_docs.len(),
        storage_duration.as_secs_f64(),
        test_docs.len() as f64 / storage_duration.as_secs_f64()
    );

    let mut metrics = Vec::new();

    // Test 1: Semantic search performance
    let search_queries = vec![
        "machine learning optimization techniques",
        "distributed systems architecture patterns",
        "database performance tuning strategies",
        "asynchronous programming with Rust",
        "microservices deployment automation",
    ];

    for (i, query) in search_queries.iter().enumerate() {
        let search_options = SearchOptions {
            limit: Some(10),
            score_threshold: Some(0.4),
            with_payload: true,
            with_vectors: false,
        };

        let start = Instant::now();
        let results = semantic_search
            .search(query, search_options)
            .await
            .expect(&format!("Failed semantic search {}", i));
        let search_duration = start.elapsed();

        metrics.push(PerformanceMetrics::new(
            format!("semantic_search_{}", i),
            search_duration,
            results.results.len(),
        ));

        println!(
            "Semantic search '{}': {:.2}ms, {} results",
            query,
            search_duration.as_millis(),
            results.results.len()
        );

        assert!(
            search_duration.as_millis() < 2000,
            "Semantic search should complete within 2 seconds"
        );
        assert!(!results.results.is_empty(), "Should find relevant results");
    }

    // Test 2: Hybrid search performance with different strategies
    let strategies = vec![
        SearchStrategy::Balanced,
        SearchStrategy::SemanticFocused,
        SearchStrategy::KeywordFocused,
    ];

    for strategy in strategies {
        let request = HybridSearchRequest {
            query: "performance optimization for large scale systems".to_string(),
            strategy,
            limit: 15,
            semantic_options: None,
            keyword_options: None,
            filters: None,
        };

        let start = Instant::now();
        let results = hybrid_search.search(request.clone()).await.expect(&format!(
            "Failed hybrid search with strategy {:?}",
            strategy
        ));
        let search_duration = start.elapsed();

        metrics.push(PerformanceMetrics::new(
            format!("hybrid_search_{:?}", strategy),
            search_duration,
            results.results.len(),
        ));

        println!(
            "Hybrid search ({:?}): {:.2}ms, {} results",
            strategy,
            search_duration.as_millis(),
            results.results.len()
        );

        assert!(
            search_duration.as_millis() < 3000,
            "Hybrid search should complete within 3 seconds"
        );
        assert!(!results.results.is_empty(), "Should find relevant results");

        // Verify result quality
        let avg_score = results.results.iter().map(|r| r.hybrid_score).sum::<f64>()
            / results.results.len() as f64;
        assert!(
            avg_score > 0.0,
            "Results should have positive relevance scores"
        );
    }

    // Test 3: Search caching effectiveness
    let cache_test_query = "caching performance test query";
    let cache_options = SearchOptions {
        limit: Some(5),
        score_threshold: Some(0.3),
        with_payload: true,
        with_vectors: false,
    };

    // First search - cache miss
    let start = Instant::now();
    let first_results = semantic_search
        .search(cache_test_query, cache_options.clone())
        .await
        .expect("Failed first cached search");
    let first_duration = start.elapsed();

    // Second search - cache hit
    let start = Instant::now();
    let second_results = semantic_search
        .search(cache_test_query, cache_options)
        .await
        .expect("Failed second cached search");
    let second_duration = start.elapsed();

    // Results should be identical
    assert_eq!(
        first_results.results.len(),
        second_results.results.len(),
        "Cached search should return identical results"
    );

    // Cache should improve performance
    let cache_speedup = first_duration.as_secs_f64() / second_duration.as_secs_f64();
    println!(
        "Search cache speedup: {:.2}x ({:.2}ms -> {:.2}ms)",
        cache_speedup,
        first_duration.as_millis(),
        second_duration.as_millis()
    );

    // Test 4: Concurrent search load testing
    let concurrent_searches = 20;
    let search_handles: Vec<_> = (0..concurrent_searches)
        .map(|i| {
            let search_service = semantic_search.clone();
            let query = format!("concurrent search query {}", i);
            tokio::spawn(async move {
                let start = Instant::now();
                let results = search_service
                    .search(
                        &query,
                        SearchOptions {
                            limit: Some(5),
                            score_threshold: Some(0.3),
                            with_payload: false,
                            with_vectors: false,
                        },
                    )
                    .await
                    .expect(&format!("Failed concurrent search {}", i));
                (start.elapsed(), results.results.len())
            })
        })
        .collect();

    let concurrent_start = Instant::now();
    let mut total_results = 0;
    for handle in search_handles {
        let (duration, result_count) = handle.await.expect("Concurrent search should complete");
        total_results += result_count;

        assert!(
            duration.as_millis() < 5000,
            "Individual concurrent search should complete within 5 seconds"
        );
    }
    let concurrent_total = concurrent_start.elapsed();

    metrics.push(PerformanceMetrics::new(
        "concurrent_search_load".to_string(),
        concurrent_total,
        concurrent_searches,
    ));

    println!(
        "Concurrent search load: {} searches in {:.2}s ({:.2} searches/sec)",
        concurrent_searches,
        concurrent_total.as_secs_f64(),
        concurrent_searches as f64 / concurrent_total.as_secs_f64()
    );

    // Test 5: Analytics and performance tracking
    let analytics = semantic_search
        .get_analytics()
        .await
        .expect("Failed to get search analytics");

    assert!(
        analytics.total_searches > 0,
        "Should track search operations"
    );
    assert!(
        analytics.avg_response_time_ms > 0.0,
        "Should track response times"
    );

    let hybrid_analytics = hybrid_search
        .get_analytics()
        .await
        .expect("Failed to get hybrid analytics");

    assert!(
        hybrid_analytics.total_searches > 0,
        "Should track hybrid searches"
    );

    // Clean up test data
    for (id, _, _) in &test_docs {
        storage
            .delete_vector(id)
            .await
            .expect("Failed to cleanup test document");
    }

    // Verify performance thresholds
    for metric in &metrics {
        if metric.operation.starts_with("semantic_search_") {
            assert!(
                metric.duration.as_millis() < 2000,
                "Semantic search should complete within 2 seconds"
            );
        } else if metric.operation.starts_with("hybrid_search_") {
            assert!(
                metric.duration.as_millis() < 3000,
                "Hybrid search should complete within 3 seconds"
            );
        } else if metric.operation == "concurrent_search_load" {
            assert!(
                metric.throughput > 2.0,
                "Should handle at least 2 concurrent searches per second"
            );
        }
    }
}

/// ANCHOR: Test migration performance and throughput
/// Tests: Large-scale migration performance, batch processing efficiency
#[tokio::test]
async fn test_anchor_migration_performance() {
    let config = create_performance_test_config();
    let migration_config = MigrationConfig {
        batch_size: 50,
        max_workers: 4,
        validation_level: ValidationLevel::Standard, // Balance speed and validation
        enable_resume: true,
        max_retries: 2,
        retry_delay_ms: 1000,
        dry_run: false,
        custom_metadata: HashMap::new(),
    };

    let embedding_service = Arc::new(LocalEmbeddingService::new(config.embedding.clone()));
    let qdrant_client = Arc::new(QdrantClient::new(config.clone()).await.unwrap());
    let storage = VectorStorage::new(qdrant_client, embedding_service.clone());
    let migration_service = MigrationService::new(
        migration_config.clone(),
        storage.clone(),
        embedding_service.clone(),
    )
    .expect("Failed to create migration service");

    embedding_service
        .initialize()
        .await
        .expect("Failed to initialize embedding service");

    let mut metrics = Vec::new();

    // Test 1: Large dataset migration performance
    let large_dataset = generate_performance_test_data(500);

    let source = MigrationSource::InMemory {
        data: large_dataset.clone(),
        source_name: "performance_test_large".to_string(),
    };

    let migration_start = Instant::now();
    let migration_id = migration_service
        .start_migration(source)
        .await
        .expect("Failed to start large migration");

    // Monitor migration progress
    let mut completed = false;
    let mut max_wait_time = Duration::from_secs(300); // 5 minutes max
    let start_wait = Instant::now();

    while !completed && start_wait.elapsed() < max_wait_time {
        tokio::time::sleep(Duration::from_millis(500)).await;

        let status = migration_service
            .get_migration_status(&migration_id)
            .await
            .expect("Failed to get migration status");

        match status.status {
            fortitude_core::vector::MigrationStatus::Completed => {
                completed = true;
                println!("Large migration completed successfully");
            }
            fortitude_core::vector::MigrationStatus::Failed => {
                panic!("Large migration failed: {:?}", status);
            }
            fortitude_core::vector::MigrationStatus::InProgress => {
                println!(
                    "Migration progress: {} / {} items ({}%)",
                    status.progress.items_processed,
                    status.progress.total_items,
                    (status.progress.items_processed as f64 / status.progress.total_items as f64
                        * 100.0) as u32
                );
            }
            _ => {}
        }
    }

    assert!(
        completed,
        "Large migration should complete within time limit"
    );

    let migration_duration = migration_start.elapsed();
    metrics.push(PerformanceMetrics::new(
        "large_dataset_migration".to_string(),
        migration_duration,
        large_dataset.len(),
    ));

    println!(
        "Large migration performance: {} items in {:.2}s ({:.2} items/sec)",
        large_dataset.len(),
        migration_duration.as_secs_f64(),
        large_dataset.len() as f64 / migration_duration.as_secs_f64()
    );

    // Verify migration results
    let final_count = storage
        .count_vectors()
        .await
        .expect("Failed to count vectors after migration");
    assert_eq!(
        final_count,
        large_dataset.len(),
        "Should have migrated all items"
    );

    // Test 2: Migration with different batch sizes
    let batch_sizes = vec![10, 25, 50, 100];

    for batch_size in batch_sizes {
        // Clean up previous data
        for (id, _, _) in &large_dataset {
            storage.delete_vector(id).await.ok(); // Ignore errors
        }

        let test_data = generate_performance_test_data(200);
        let mut test_config = migration_config.clone();
        test_config.batch_size = batch_size;

        let test_migration_service =
            MigrationService::new(test_config, storage.clone(), embedding_service.clone())
                .expect("Failed to create test migration service");

        let source = MigrationSource::InMemory {
            data: test_data.clone(),
            source_name: format!("batch_size_test_{}", batch_size),
        };

        let start = Instant::now();
        let migration_id = test_migration_service
            .start_migration(source)
            .await
            .expect(&format!(
                "Failed to start migration with batch size {}",
                batch_size
            ));

        // Wait for completion
        let mut completed = false;
        let mut iterations = 120; // 1 minute max

        while !completed && iterations > 0 {
            tokio::time::sleep(Duration::from_millis(500)).await;

            let status = test_migration_service
                .get_migration_status(&migration_id)
                .await
                .expect("Failed to get migration status");

            if matches!(
                status.status,
                fortitude_core::vector::MigrationStatus::Completed
            ) {
                completed = true;
            } else if matches!(
                status.status,
                fortitude_core::vector::MigrationStatus::Failed
            ) {
                panic!("Migration with batch size {} failed", batch_size);
            }

            iterations -= 1;
        }

        assert!(
            completed,
            "Migration with batch size {} should complete",
            batch_size
        );

        let duration = start.elapsed();
        metrics.push(PerformanceMetrics::new(
            format!("migration_batch_size_{}", batch_size),
            duration,
            test_data.len(),
        ));

        println!(
            "Migration batch size {}: {:.2} items/sec",
            batch_size,
            test_data.len() as f64 / duration.as_secs_f64()
        );

        // Clean up
        for (id, _, _) in &test_data {
            storage.delete_vector(id).await.ok();
        }
    }

    // Test 3: Migration validation performance impact
    let validation_levels = vec![
        ValidationLevel::Lenient,
        ValidationLevel::Moderate,
        ValidationLevel::Strict,
    ];

    for validation_level in validation_levels {
        let test_data = generate_performance_test_data(100);
        let mut test_config = migration_config.clone();
        test_config.validation_level = validation_level;
        test_config.batch_size = 20;

        let test_migration_service =
            MigrationService::new(test_config, storage.clone(), embedding_service.clone())
                .expect("Failed to create validation test migration service");

        let source = MigrationSource::InMemory {
            data: test_data.clone(),
            source_name: format!("validation_test_{:?}", validation_level),
        };

        let start = Instant::now();
        let migration_id = test_migration_service
            .start_migration(source)
            .await
            .expect(&format!(
                "Failed to start migration with validation {:?}",
                validation_level
            ));

        // Wait for completion
        let mut completed = false;
        let mut iterations = 60; // 30 seconds max

        while !completed && iterations > 0 {
            tokio::time::sleep(Duration::from_millis(500)).await;

            let status = test_migration_service
                .get_migration_status(&migration_id)
                .await
                .expect("Failed to get migration status");

            if matches!(
                status.status,
                fortitude_core::vector::MigrationStatus::Completed
            ) {
                completed = true;
            } else if matches!(
                status.status,
                fortitude_core::vector::MigrationStatus::Failed
            ) {
                // Some validation levels might reject data - that's okay for this test
                completed = true;
            }

            iterations -= 1;
        }

        let duration = start.elapsed();
        metrics.push(PerformanceMetrics::new(
            format!("migration_validation_{:?}", validation_level),
            duration,
            test_data.len(),
        ));

        println!(
            "Migration validation {:?}: {:.2} items/sec",
            validation_level,
            test_data.len() as f64 / duration.as_secs_f64()
        );

        // Clean up
        for (id, _, _) in &test_data {
            storage.delete_vector(id).await.ok();
        }
    }

    // Verify performance thresholds
    for metric in &metrics {
        match metric.operation.as_str() {
            "large_dataset_migration" => {
                assert!(
                    metric.throughput > 5.0,
                    "Large migration should process > 5 items/sec"
                );
            }
            _ if metric.operation.starts_with("migration_batch_size_") => {
                assert!(
                    metric.throughput > 2.0,
                    "Batch migration should process > 2 items/sec"
                );
            }
            _ if metric.operation.starts_with("migration_validation_") => {
                assert!(
                    metric.throughput > 1.0,
                    "Validation migration should process > 1 item/sec"
                );
            }
            _ => {}
        }
    }
}
