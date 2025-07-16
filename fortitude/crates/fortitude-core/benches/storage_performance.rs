// ABOUTME: Storage operations performance benchmarks
//! This benchmark suite measures the performance of vector storage operations,
//! database interactions, batch operations, and data migration processes.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use fortitude_core::vector::{
    DistanceMetric, EmbeddingConfig, EmbeddingGenerator, LocalEmbeddingService, MigrationConfig,
    SearchConfig, SimilaritySearchResult, VectorConfig, VectorDocument,
};
use std::collections::HashMap;
use std::time::Duration;
use tokio::runtime::Runtime;
use uuid::Uuid;

/// Helper function to create test vector config
fn create_test_vector_config() -> VectorConfig {
    VectorConfig {
        url: "http://localhost:6334".to_string(),
        api_key: None,
        timeout: Duration::from_secs(30),
        default_collection: "test_collection".to_string(),
        vector_dimensions: 384,
        distance_metric: DistanceMetric::Cosine,
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

/// Helper function to create test documents
fn create_test_documents(count: usize, content_length: usize) -> Vec<VectorDocument> {
    let base_content = "This is sample content for testing vector storage performance. ";
    let repeat_count = (content_length / base_content.len()).max(1);

    (0..count)
        .map(|i| VectorDocument {
            id: Uuid::new_v4().to_string(),
            content: format!("{}Document ID: {}", base_content.repeat(repeat_count), i),
            embedding: vec![0.5f32; 384],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("index".to_string(), serde_json::json!(i));
                meta.insert("category".to_string(), serde_json::json!("test"));
                meta.insert(
                    "timestamp".to_string(),
                    serde_json::json!("2024-01-01T00:00:00Z"),
                );
                meta
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
        .collect()
}

/// Helper function to create search config
fn create_search_config(limit: usize, threshold: f64) -> SearchConfig {
    SearchConfig {
        limit: Some(limit),
        threshold: Some(threshold),
        with_payload: true,
        with_vectors: false,
    }
}

/// Benchmark document creation and validation
fn bench_document_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("document_creation");

    for count in [1, 10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::new("create_documents", count),
            count,
            |b, &count| {
                b.iter(|| {
                    let documents = create_test_documents(black_box(count), 200);
                    black_box(documents);
                });
            },
        );
    }
    group.finish();
}

/// Benchmark document serialization/deserialization
fn bench_document_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("document_serialization");

    let test_documents = create_test_documents(100, 500);

    group.bench_function("serialize_single", |b| {
        b.iter(|| {
            let doc = &test_documents[0];
            let _serialized = serde_json::to_string(black_box(doc)).unwrap();
        });
    });

    group.bench_function("deserialize_single", |b| {
        b.iter(|| {
            let doc = &test_documents[0];
            let serialized = serde_json::to_string(doc).unwrap();
            let _deserialized: VectorDocument =
                serde_json::from_str(black_box(&serialized)).unwrap();
        });
    });

    group.bench_function("serialize_batch", |b| {
        b.iter(|| {
            let _serialized = serde_json::to_string(black_box(&test_documents)).unwrap();
        });
    });

    group.bench_function("deserialize_batch", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(&test_documents).unwrap();
            let _deserialized: Vec<VectorDocument> =
                serde_json::from_str(black_box(&serialized)).unwrap();
        });
    });

    group.finish();
}

/// Benchmark storage configuration validation
fn bench_storage_config_validation(c: &mut Criterion) {
    c.bench_function("vector_config_validation", |b| {
        b.iter(|| {
            let config = create_test_vector_config();
            let _result = config.validate();
        });
    });

    c.bench_function("search_config_validation", |b| {
        b.iter(|| {
            let config = create_search_config(50, 0.8);
            // Simulate validation
            let _valid = config.limit.is_some()
                && config.threshold.is_some()
                && config.threshold.unwrap() >= 0.0
                && config.threshold.unwrap() <= 1.0;
        });
    });
}

/// Benchmark vector operations preparation
fn bench_vector_operations_prep(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("vector_operations_prep");

    // Benchmark embedding generation for storage
    group.bench_function("embedding_for_storage", |b| {
        b.to_async(&rt).iter(|| async {
            let config = EmbeddingConfig::default();
            let service = LocalEmbeddingService::new(config);
            let _ = service.initialize().await;

            let text = "Document content for storage testing with vector embeddings";
            let _embedding = service.generate_embedding(black_box(text)).await;
        });
    });

    // Benchmark batch embedding generation
    for batch_size in [5, 10, 25, 50].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch_embeddings", batch_size),
            batch_size,
            |b, &batch_size| {
                b.to_async(&rt).iter(|| async {
                    let config = EmbeddingConfig::default();
                    let service = LocalEmbeddingService::new(config);
                    let _ = service.initialize().await;

                    let texts: Vec<String> = (0..batch_size)
                        .map(|i| format!("Batch document {} for embedding generation", i))
                        .collect();

                    let _embeddings = service.generate_embeddings(black_box(&texts)).await;
                });
            },
        );
    }

    group.finish();
}

/// Benchmark metadata processing
fn bench_metadata_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("metadata_processing");

    // Test different metadata complexity levels
    let simple_metadata = {
        let mut meta = HashMap::new();
        meta.insert("id".to_string(), serde_json::json!("simple"));
        meta
    };

    let complex_metadata = {
        let mut meta = HashMap::new();
        meta.insert("id".to_string(), serde_json::json!("complex"));
        meta.insert(
            "tags".to_string(),
            serde_json::json!(["tag1", "tag2", "tag3"]),
        );
        meta.insert(
            "properties".to_string(),
            serde_json::json!({
                "nested": {
                    "value": 42,
                    "array": [1, 2, 3, 4, 5]
                }
            }),
        );
        meta.insert(
            "timestamp".to_string(),
            serde_json::json!("2024-01-01T00:00:00Z"),
        );
        meta
    };

    group.bench_function("simple_metadata", |b| {
        b.iter(|| {
            let _processed = simple_metadata
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<HashMap<String, serde_json::Value>>();
            black_box(_processed);
        });
    });

    group.bench_function("complex_metadata", |b| {
        b.iter(|| {
            let _processed = complex_metadata
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<HashMap<String, serde_json::Value>>();
            black_box(_processed);
        });
    });

    group.finish();
}

/// Benchmark search result processing
fn bench_search_result_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_result_processing");

    // Create mock search results
    for result_count in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*result_count as u64));
        group.bench_with_input(
            BenchmarkId::new("process_results", result_count),
            result_count,
            |b, &result_count| {
                b.iter(|| {
                    let results: Vec<SimilaritySearchResult> = (0..result_count)
                        .map(|i| SimilaritySearchResult {
                            document: VectorDocument {
                                id: format!("doc_{}", i),
                                content: format!("Result document {}", i),
                                embedding: vec![0.5f32; 384],
                                metadata: HashMap::new(),
                                created_at: chrono::Utc::now(),
                                updated_at: chrono::Utc::now(),
                            },
                            score: 1.0 - (i as f64 / result_count as f64),
                        })
                        .collect();

                    // Process results (sorting, filtering, etc.)
                    let mut processed = results;
                    processed.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

                    // Apply threshold filtering
                    let filtered: Vec<_> =
                        processed.into_iter().filter(|r| r.score > 0.5).collect();

                    black_box(filtered);
                });
            },
        );
    }
    group.finish();
}

/// Benchmark batch operation preparation
fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");

    for batch_size in [10, 50, 100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("prepare_batch_insert", batch_size),
            batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let documents = create_test_documents(batch_size, 200);

                    // Prepare batch operation data
                    let batch_data: Vec<_> = documents
                        .iter()
                        .map(|doc| (doc.id.clone(), doc.embedding.clone(), doc.metadata.clone()))
                        .collect();

                    black_box(batch_data);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("prepare_batch_update", batch_size),
            batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    // Simulate batch update preparation
                    let updates: Vec<_> = (0..batch_size)
                        .map(|i| {
                            let mut metadata = HashMap::new();
                            metadata.insert("updated".to_string(), serde_json::json!(true));
                            metadata.insert("version".to_string(), serde_json::json!(i));
                            (format!("doc_{}", i), metadata)
                        })
                        .collect();

                    black_box(updates);
                });
            },
        );
    }
    group.finish();
}

/// Benchmark migration data processing
fn bench_migration_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("migration_processing");

    group.bench_function("migration_config_validation", |b| {
        b.to_async(&rt).iter(|| async {
            let config = MigrationConfig::default();

            // Simulate validation
            let _valid =
                config.batch_size > 0 && config.timeout.as_secs() > 0 && config.max_retries > 0;

            black_box(_valid);
        });
    });

    for item_count in [100, 500, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*item_count as u64));
        group.bench_with_input(
            BenchmarkId::new("process_migration_items", item_count),
            item_count,
            |b, &item_count| {
                b.to_async(&rt).iter(|| async {
                    // Simulate migration item processing
                    let items: Vec<_> = (0..item_count)
                        .map(|i| format!("migration_item_{}", i))
                        .collect();

                    // Process items in batches
                    let batch_size = 100;
                    let mut processed = 0;

                    for chunk in items.chunks(batch_size) {
                        // Simulate processing delay
                        tokio::time::sleep(Duration::from_micros(10)).await;
                        processed += chunk.len();
                    }

                    black_box(processed);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent storage operations
fn bench_concurrent_storage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_storage");

    for concurrency in [1, 2, 4, 8, 16].iter() {
        group.throughput(Throughput::Elements(*concurrency as u64));
        group.bench_with_input(
            BenchmarkId::new("concurrent_document_prep", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();

                    for i in 0..concurrency {
                        let handle = tokio::spawn(async move {
                            let documents = create_test_documents(10, 200);

                            // Simulate document processing
                            for doc in &documents {
                                let _serialized = serde_json::to_string(doc).unwrap();
                                tokio::time::sleep(Duration::from_micros(10)).await;
                            }

                            black_box(documents);
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

/// Benchmark distance metric calculations
fn bench_distance_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("distance_metrics");

    let vec1 = vec![0.5f32; 384];
    let vec2 = vec![0.3f32; 384];

    group.bench_function("cosine_distance", |b| {
        b.iter(|| {
            let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
            let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
            let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();

            let cosine_similarity = if norm1 > 0.0 && norm2 > 0.0 {
                dot_product / (norm1 * norm2)
            } else {
                0.0
            };

            let distance = 1.0 - cosine_similarity;
            black_box(distance);
        });
    });

    group.bench_function("euclidean_distance", |b| {
        b.iter(|| {
            let distance: f32 = vec1
                .iter()
                .zip(vec2.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>()
                .sqrt();

            black_box(distance);
        });
    });

    group.bench_function("manhattan_distance", |b| {
        b.iter(|| {
            let distance: f32 = vec1
                .iter()
                .zip(vec2.iter())
                .map(|(a, b)| (a - b).abs())
                .sum();

            black_box(distance);
        });
    });

    group.finish();
}

/// Benchmark memory usage patterns for storage
fn bench_storage_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_memory");

    for doc_count in [100, 500, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*doc_count as u64));
        group.bench_with_input(
            BenchmarkId::new("memory_allocation", doc_count),
            doc_count,
            |b, &doc_count| {
                b.iter(|| {
                    // Test memory allocation patterns
                    let documents = create_test_documents(doc_count, 500);

                    // Store in different data structures
                    let vec_storage: Vec<_> = documents.clone();
                    let map_storage: HashMap<String, VectorDocument> = documents
                        .into_iter()
                        .map(|doc| (doc.id.clone(), doc))
                        .collect();

                    black_box((vec_storage, map_storage));
                });
            },
        );
    }
    group.finish();
}

/// Performance regression detection for storage
fn bench_storage_regression_detection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("storage_performance_baseline", |b| {
        b.to_async(&rt).iter(|| async {
            let start = std::time::Instant::now();

            // Simulate complete storage workflow
            let config = create_test_vector_config();
            let embedding_config = EmbeddingConfig::default();
            let embedding_service = LocalEmbeddingService::new(embedding_config);
            let _ = embedding_service.initialize().await;

            // Create documents
            let documents = create_test_documents(50, 300);

            // Process embeddings
            for doc in &documents {
                let _embedding = embedding_service.generate_embedding(&doc.content).await;
            }

            // Serialize documents
            let _serialized = serde_json::to_string(&documents).unwrap();

            // Prepare search operations
            let search_config = create_search_config(20, 0.7);
            let query_vector = vec![0.5f32; 384];

            // Simulate similarity calculations
            for doc in &documents {
                let dot_product: f32 = query_vector
                    .iter()
                    .zip(doc.embedding.iter())
                    .map(|(a, b)| a * b)
                    .sum();
                let norm1: f32 = query_vector.iter().map(|x| x * x).sum::<f32>().sqrt();
                let norm2: f32 = doc.embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

                let _similarity = if norm1 > 0.0 && norm2 > 0.0 {
                    dot_product / (norm1 * norm2)
                } else {
                    0.0
                };
            }

            let elapsed = start.elapsed();
            black_box(elapsed);
        });
    });
}

criterion_group!(
    benches,
    bench_document_creation,
    bench_document_serialization,
    bench_storage_config_validation,
    bench_vector_operations_prep,
    bench_metadata_processing,
    bench_search_result_processing,
    bench_batch_operations,
    bench_migration_processing,
    bench_concurrent_storage,
    bench_distance_metrics,
    bench_storage_memory,
    bench_storage_regression_detection
);

criterion_main!(benches);
