// ABOUTME: Search operations performance benchmarks
//! This benchmark suite measures the performance of semantic search, hybrid search,
//! query processing, filtering, and result ranking operations.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use fortitude_core::vector::{
    FilterOperation, FusionMethod, HybridSearchConfig, SearchStrategy, SemanticSearchConfig,
    SearchFilter, SearchOptions,
    VectorConfig,
};
use std::collections::HashMap;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Helper function to create test vector config
fn create_test_vector_config() -> VectorConfig {
    VectorConfig {
        url: "http://localhost:6334".to_string(),
        api_key: None,
        timeout: Duration::from_secs(30),
        default_collection: "test_collection".to_string(),
        vector_dimensions: 384,
        distance_metric: fortitude_core::vector::DistanceMetric::Cosine,
        health_check: fortitude_core::vector::HealthCheckConfig {
            enabled: true,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            max_failures: 3,
        },
        connection_pool: fortitude_core::vector::ConnectionPoolConfig::default(),
        embedding: fortitude_core::vector::EmbeddingConfig::default(),
    }
}

/// Helper function to create semantic search config
fn create_semantic_config() -> SemanticSearchConfig {
    SemanticSearchConfig {
        default_limit: 10,
        max_limit: 100,
        default_threshold: 0.7,
        enable_explanations: true,
        enable_analytics: true,
        result_diversification: true,
        temporal_boost_decay: 0.1,
        quality_boost_factor: 1.5,
        cache_enabled: true,
        cache_ttl: Duration::from_secs(300),
        max_cache_size: 1000,
    }
}

/// Helper function to create hybrid search config
fn create_hybrid_config() -> HybridSearchConfig {
    HybridSearchConfig {
        default_fusion_method: FusionMethod::ReciprocalRankFusion,
        default_strategy: SearchStrategy::Balanced,
        vector_weight: 0.7,
        keyword_weight: 0.3,
        min_vector_threshold: 0.5,
        min_keyword_threshold: 0.1,
        enable_adaptive_strategy: true,
        max_results_per_type: 50,
        enable_diversification: false,
        enable_caching: true,
        cache_ttl_seconds: 300,
    }
}

/// Create test search options with different configurations
fn create_search_options(
    limit: usize,
    threshold: Option<f64>,
    filters: Vec<SearchFilter>,
    diversify: bool,
) -> SearchOptions {
    SearchOptions {
        limit,
        threshold,
        collection: None,
        filters,
        diversify_results: diversify,
        temporal_boost: Some(0.1),
        quality_boost: Some(1.2),
        include_explanations: true,
        min_content_length: Some(50),
        max_content_length: None,
        fuzzy_matching: false,
    }
}

/// Generate test search filters
fn create_test_filters() -> Vec<Vec<SearchFilter>> {
    vec![
        // No filters
        vec![],
        // Single equality filter
        vec![SearchFilter {
            field: "category".to_string(),
            operation: FilterOperation::Equals,
            value: serde_json::json!("research"),
        }],
        // Multiple filters
        vec![
            SearchFilter {
                field: "category".to_string(),
                operation: FilterOperation::Equals,
                value: serde_json::json!("research"),
            },
            SearchFilter {
                field: "priority".to_string(),
                operation: FilterOperation::GreaterThan,
                value: serde_json::json!(5),
            },
        ],
        // Complex filters
        vec![
            SearchFilter {
                field: "tags".to_string(),
                operation: FilterOperation::Contains,
                value: serde_json::json!("important"),
            },
            SearchFilter {
                field: "date".to_string(),
                operation: FilterOperation::After,
                value: serde_json::json!("2024-01-01"),
            },
            SearchFilter {
                field: "status".to_string(),
                operation: FilterOperation::In,
                value: serde_json::json!(["active", "pending"]),
            },
        ],
    ]
}

/// Benchmark semantic search configuration validation
fn bench_search_config_validation(c: &mut Criterion) {
    c.bench_function("semantic_config_validation", |b| {
        b.iter(|| {
            let config = create_semantic_config();
            let _result = config.validate();
        });
    });

    c.bench_function("hybrid_config_validation", |b| {
        b.iter(|| {
            let config = create_hybrid_config();
            let _result = config.validate();
        });
    });
}

/// Benchmark search options creation and validation
fn bench_search_options(c: &mut Criterion) {
    let filter_sets = create_test_filters();

    let mut group = c.benchmark_group("search_options");

    for (i, filters) in filter_sets.iter().enumerate() {
        group.throughput(Throughput::Elements(filters.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("create_options", i),
            filters,
            |b, filters| {
                b.iter(|| {
                    let options = create_search_options(
                        black_box(20),
                        black_box(Some(0.8)),
                        black_box(filters.clone()),
                        black_box(true),
                    );
                    black_box(options);
                });
            },
        );
    }
    group.finish();
}

/// Benchmark search filter validation
fn bench_filter_validation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("filter_validation");

    let test_filters = vec![
        SearchFilter {
            field: "simple_field".to_string(),
            operation: FilterOperation::Equals,
            value: serde_json::json!("value"),
        },
        SearchFilter {
            field: "numeric_field".to_string(),
            operation: FilterOperation::GreaterThan,
            value: serde_json::json!(42),
        },
        SearchFilter {
            field: "array_field".to_string(),
            operation: FilterOperation::In,
            value: serde_json::json!(["a", "b", "c", "d", "e"]),
        },
        SearchFilter {
            field: "complex_field".to_string(),
            operation: FilterOperation::Contains,
            value: serde_json::json!({"nested": {"value": "test"}}),
        },
    ];

    for (i, filter) in test_filters.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("validate_filter", i),
            filter,
            |b, filter| {
                b.to_async(&rt).iter(|| async {
                    // Simulate filter validation logic
                    let _validated = match &filter.operation {
                        FilterOperation::Equals | FilterOperation::NotEquals => true,
                        FilterOperation::GreaterThan | FilterOperation::LessThan => {
                            filter.value.is_number()
                        }
                        FilterOperation::Contains | FilterOperation::NotContains => true,
                        FilterOperation::In | FilterOperation::NotIn => filter.value.is_array(),
                        FilterOperation::After | FilterOperation::Before => {
                            filter.value.is_string()
                        }
                        _ => true,
                    };
                    black_box(_validated);
                });
            },
        );
    }
    group.finish();
}

/// Benchmark query preprocessing and analysis
fn bench_query_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("query_processing");

    let test_queries = vec![
        "simple query",
        "multi word query with more terms",
        "Complex query with special characters!@#$%^&*()",
        "Very long query that simulates real user input with multiple concepts and detailed requirements for research purposes".repeat(3),
        "Query with numbers 123 and dates 2024-01-01",
    ];

    for (i, query) in test_queries.iter().enumerate() {
        group.throughput(Throughput::Bytes(query.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("preprocess_query", i),
            query,
            |b, query| {
                b.to_async(&rt).iter(|| async {
                    // Simulate query preprocessing
                    let processed = query
                        .trim()
                        .to_lowercase()
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ");

                    // Simulate query analysis
                    let word_count = processed.split_whitespace().count();
                    let char_count = processed.len();
                    let has_special_chars = processed
                        .chars()
                        .any(|c| !c.is_alphanumeric() && !c.is_whitespace());

                    black_box((processed, word_count, char_count, has_special_chars));
                });
            },
        );
    }
    group.finish();
}

/// Benchmark vector similarity calculations
fn bench_similarity_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("similarity_calculations");

    // Test different vector sizes
    for size in [128, 256, 384, 512, 768, 1024].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("cosine_similarity", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let vec1 = vec![0.5f32; size];
                    let vec2 = vec![0.3f32; size];

                    // Calculate cosine similarity
                    let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
                    let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
                    let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();

                    let similarity = if norm1 > 0.0 && norm2 > 0.0 {
                        dot_product / (norm1 * norm2)
                    } else {
                        0.0
                    };

                    black_box(similarity);
                });
            },
        );
    }
    group.finish();
}

/// Benchmark result ranking and scoring
fn bench_result_ranking(c: &mut Criterion) {
    let mut group = c.benchmark_group("result_ranking");

    // Test different result set sizes
    for count in [10, 50, 100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::new("rank_results", count),
            count,
            |b, &count| {
                b.iter(|| {
                    // Create mock search results
                    let mut results: Vec<(f32, String)> = (0..count)
                        .map(|i| (rand::random::<f32>(), format!("result_{}", i)))
                        .collect();

                    // Sort by similarity score (descending)
                    results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

                    // Apply additional scoring factors
                    for (score, _) in &mut results {
                        *score *= 1.0 + rand::random::<f32>() * 0.2; // Quality boost
                        *score *= 1.0 - rand::random::<f32>() * 0.1; // Temporal decay
                    }

                    black_box(results);
                });
            },
        );
    }
    group.finish();
}

/// Benchmark result diversification
fn bench_result_diversification(c: &mut Criterion) {
    let mut group = c.benchmark_group("result_diversification");

    for count in [10, 20, 50, 100].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::new("diversify", count), count, |b, &count| {
            b.iter(|| {
                // Create mock results with embeddings
                let results: Vec<(f32, Vec<f32>)> = (0..count)
                    .map(|_| (rand::random::<f32>(), vec![rand::random::<f32>(); 384]))
                    .collect();

                // Simple diversification algorithm
                let mut diversified = Vec::new();
                let mut remaining = results;

                while !remaining.is_empty() && diversified.len() < count / 2 {
                    // Find the best remaining result
                    let best_idx = remaining
                        .iter()
                        .enumerate()
                        .max_by(|(_, a), (_, b)| a.0.partial_cmp(&b.0).unwrap())
                        .map(|(idx, _)| idx)
                        .unwrap();

                    let best_result = remaining.remove(best_idx);
                    diversified.push(best_result);

                    // Remove similar results (simplified)
                    remaining.retain(|(_, vec)| {
                        let similarity = diversified
                            .last()
                            .unwrap()
                            .1
                            .iter()
                            .zip(vec.iter())
                            .map(|(a, b)| (a - b).abs())
                            .sum::<f32>()
                            / vec.len() as f32;
                        similarity > 0.1 // Keep if not too similar
                    });
                }

                black_box(diversified);
            });
        });
    }
    group.finish();
}

/// Benchmark search caching operations
fn bench_search_caching(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("search_caching");

    group.bench_function("cache_key_generation", |b| {
        b.to_async(&rt).iter(|| async {
            let query = "test search query";
            let options = create_search_options(10, Some(0.8), vec![], false);

            // Generate cache key (simplified)
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            query.hash(&mut hasher);
            options.limit.hash(&mut hasher);

            let cache_key = format!("search_{:x}", hasher.finish());
            black_box(cache_key);
        });
    });

    group.bench_function("cache_lookup", |b| {
        b.to_async(&rt).iter(|| async {
            use std::collections::HashMap;

            // Simulate cache lookup
            let mut cache: HashMap<String, Vec<String>> = HashMap::new();
            cache.insert("key1".to_string(), vec!["result1".to_string()]);
            cache.insert("key2".to_string(), vec!["result2".to_string()]);

            let lookup_key = "key1";
            let _result = cache.get(black_box(lookup_key));
        });
    });

    group.finish();
}

/// Benchmark hybrid search fusion methods
fn bench_fusion_methods(c: &mut Criterion) {
    let mut group = c.benchmark_group("fusion_methods");

    let semantic_results = vec![
        (0.9, "doc1"),
        (0.8, "doc2"),
        (0.7, "doc3"),
        (0.6, "doc4"),
        (0.5, "doc5"),
    ];
    let keyword_results = vec![
        (0.8, "doc2"),
        (0.7, "doc1"),
        (0.6, "doc5"),
        (0.5, "doc6"),
        (0.4, "doc3"),
    ];

    group.bench_function("rank_fusion", |b| {
        b.iter(|| {
            // Reciprocal Rank Fusion
            let mut combined_scores = HashMap::new();

            for (rank, (_, doc)) in semantic_results.iter().enumerate() {
                let score = 1.0 / (rank + 1) as f32;
                *combined_scores.entry(doc).or_insert(0.0) += score * 0.7; // semantic weight
            }

            for (rank, (_, doc)) in keyword_results.iter().enumerate() {
                let score = 1.0 / (rank + 1) as f32;
                *combined_scores.entry(doc).or_insert(0.0) += score * 0.3; // keyword weight
            }

            let mut final_results: Vec<_> = combined_scores.into_iter().collect();
            final_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            black_box(final_results);
        });
    });

    group.bench_function("score_fusion", |b| {
        b.iter(|| {
            // Weighted Score Fusion
            let mut combined_scores = HashMap::new();

            for (score, doc) in &semantic_results {
                *combined_scores.entry(doc).or_insert(0.0) += score * 0.7;
            }

            for (score, doc) in &keyword_results {
                *combined_scores.entry(doc).or_insert(0.0) += score * 0.3;
            }

            let mut final_results: Vec<_> = combined_scores.into_iter().collect();
            final_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            black_box(final_results);
        });
    });

    group.finish();
}

/// Benchmark concurrent search operations
fn bench_concurrent_search(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_search");

    for concurrency in [1, 2, 4, 8, 16].iter() {
        group.throughput(Throughput::Elements(*concurrency as u64));
        group.bench_with_input(
            BenchmarkId::new("concurrent_queries", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();

                    for i in 0..concurrency {
                        let handle = tokio::spawn(async move {
                            let query = format!("concurrent query {}", i);
                            let options = create_search_options(10, Some(0.8), vec![], false);

                            // Simulate search operation
                            tokio::time::sleep(Duration::from_millis(10)).await;

                            black_box((query, options));
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        let _ = handle.await;
                    }
                });
            },
        );
    }
    group.finish();
}

/// Performance regression detection for search
fn bench_search_regression_detection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("search_performance_baseline", |b| {
        b.to_async(&rt).iter(|| async {
            let start = std::time::Instant::now();

            // Simulate complete search workflow
            let query = "comprehensive search performance test";
            let filters = create_test_filters()[2].clone(); // Complex filters
            let options = create_search_options(20, Some(0.7), filters, true);

            // Query preprocessing
            let processed_query = query.trim().to_lowercase();

            // Filter validation
            for filter in &options.filters {
                let _valid = match filter.operation {
                    FilterOperation::Equals => true,
                    FilterOperation::GreaterThan => filter.value.is_number(),
                    FilterOperation::Contains => true,
                    _ => true,
                };
            }

            // Similarity calculation simulation
            let query_vector = vec![0.5f32; 384];
            let doc_vectors = vec![vec![0.3f32; 384]; 100];

            let mut similarities = Vec::new();
            for doc_vec in &doc_vectors {
                let dot_product: f32 = query_vector
                    .iter()
                    .zip(doc_vec.iter())
                    .map(|(a, b)| a * b)
                    .sum();
                let norm1: f32 = query_vector.iter().map(|x| x * x).sum::<f32>().sqrt();
                let norm2: f32 = doc_vec.iter().map(|x| x * x).sum::<f32>().sqrt();

                let similarity = if norm1 > 0.0 && norm2 > 0.0 {
                    dot_product / (norm1 * norm2)
                } else {
                    0.0
                };
                similarities.push(similarity);
            }

            // Result ranking
            similarities.sort_by(|a, b| b.partial_cmp(a).unwrap());

            // Result diversification
            if options.diversify_results {
                let _diversified = similarities.into_iter().take(10).collect::<Vec<_>>();
            }

            let elapsed = start.elapsed();
            black_box(elapsed);
        });
    });
}

criterion_group!(
    benches,
    bench_search_config_validation,
    bench_search_options,
    bench_filter_validation,
    bench_query_processing,
    bench_similarity_calculations,
    bench_result_ranking,
    bench_result_diversification,
    bench_search_caching,
    bench_fusion_methods,
    bench_concurrent_search,
    bench_search_regression_detection
);

criterion_main!(benches);
