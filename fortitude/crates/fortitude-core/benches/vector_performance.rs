// Copyright 2025 CE-DPS Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// ABOUTME: Comprehensive vector operations performance benchmarks
//! This benchmark suite measures the performance of core vector operations
//! including client initialization, collection management, and health checks.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use fortitude_core::{
    vector::{
        EmbeddingConfig, EmbeddingGenerator, LocalEmbeddingService, QdrantClient, VectorConfig,
        VectorOperation, VectorRequest,
    },
    ApiClient,
};
use std::time::Duration;
use tokio::runtime::Runtime;

/// Helper function to create a test vector config
fn create_test_config() -> VectorConfig {
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

/// Helper function to create test embedding config
fn create_embedding_config() -> EmbeddingConfig {
    EmbeddingConfig::default()
}

/// Benchmark vector client initialization
fn bench_client_initialization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("client_initialization", |b| {
        b.to_async(&rt).iter(|| async {
            let config = create_test_config();
            // Note: This will fail if Qdrant is not running, but measures the attempt time
            let _result = QdrantClient::new(black_box(config)).await;
        });
    });
}

/// Benchmark embedding service initialization
fn bench_embedding_initialization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("embedding_initialization", |b| {
        b.to_async(&rt).iter(|| async {
            let config = create_embedding_config();
            let service = LocalEmbeddingService::new(black_box(config));
            let _result = service.initialize().await;
        });
    });
}

/// Benchmark vector request validation
fn bench_request_validation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_config();

    // Create different types of requests for testing
    let search_request = VectorRequest {
        operation: VectorOperation::Search {
            query_vector: vec![0.5f32; 384],
        },
        collection: Some("test".to_string()),
        vectors: None,
        metadata: None,
        limit: Some(10),
    };

    let insert_request = VectorRequest {
        operation: VectorOperation::Insert {
            id: "test_id".to_string(),
            vector: vec![0.5f32; 384],
        },
        collection: Some("test".to_string()),
        vectors: None,
        metadata: None,
        limit: None,
    };

    let health_request = VectorRequest {
        operation: VectorOperation::HealthCheck,
        collection: None,
        vectors: None,
        metadata: None,
        limit: None,
    };

    c.bench_function("request_validation_search", |b| {
        b.to_async(&rt).iter(|| async {
            // Mock client for validation testing
            let client = QdrantClient::new(config.clone()).await;
            if let Ok(client) = client {
                let _result = client.validate_request(black_box(&search_request));
            }
        });
    });

    c.bench_function("request_validation_insert", |b| {
        b.to_async(&rt).iter(|| async {
            let client = QdrantClient::new(config.clone()).await;
            if let Ok(client) = client {
                let _result = client.validate_request(black_box(&insert_request));
            }
        });
    });

    c.bench_function("request_validation_health", |b| {
        b.to_async(&rt).iter(|| async {
            let client = QdrantClient::new(config.clone()).await;
            if let Ok(client) = client {
                let _result = client.validate_request(black_box(&health_request));
            }
        });
    });
}

/// Benchmark cost estimation
fn bench_cost_estimation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_config();

    let requests = vec![
        VectorRequest {
            operation: VectorOperation::Search {
                query_vector: vec![0.5f32; 384],
            },
            collection: Some("test".to_string()),
            vectors: None,
            metadata: None,
            limit: Some(10),
        },
        VectorRequest {
            operation: VectorOperation::Insert {
                id: "test_id".to_string(),
                vector: vec![0.5f32; 384],
            },
            collection: Some("test".to_string()),
            vectors: None,
            metadata: None,
            limit: None,
        },
    ];

    c.bench_function("cost_estimation", |b| {
        b.to_async(&rt).iter(|| async {
            let client = QdrantClient::new(config.clone()).await;
            if let Ok(client) = client {
                for request in &requests {
                    let _cost = client.estimate_cost(black_box(request));
                }
            }
        });
    });
}

/// Benchmark vector dimension validation with different sizes
fn bench_vector_dimension_validation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_config();

    let mut group = c.benchmark_group("vector_dimension_validation");

    for size in [128, 256, 384, 512, 768, 1024, 1536].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("validate", size), size, |b, &size| {
            b.to_async(&rt).iter(|| async {
                let request = VectorRequest {
                    operation: VectorOperation::Search {
                        query_vector: vec![0.5f32; size],
                    },
                    collection: Some("test".to_string()),
                    vectors: None,
                    metadata: None,
                    limit: Some(10),
                };

                let client = QdrantClient::new(config.clone()).await;
                if let Ok(client) = client {
                    let _result = client.validate_request(black_box(&request));
                }
            });
        });
    }
    group.finish();
}

/// Benchmark configuration validation
fn bench_config_validation(c: &mut Criterion) {
    c.bench_function("config_validation", |b| {
        b.iter(|| {
            let config = create_test_config();
            let _result = config.validate();
        });
    });
}

/// Benchmark memory usage patterns for vector operations
fn bench_vector_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_memory_patterns");

    // Test different vector sizes and their memory allocation patterns
    for size in [100, 500, 1000, 5000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("vector_creation", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let vectors: Vec<Vec<f32>> = (0..size).map(|_| vec![0.5f32; 384]).collect();
                    black_box(vectors);
                });
            },
        );
    }
    group.finish();
}

/// Benchmark concurrent vector operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_config();

    let mut group = c.benchmark_group("concurrent_operations");

    for concurrency in [1, 2, 4, 8, 16].iter() {
        group.throughput(Throughput::Elements(*concurrency as u64));
        group.bench_with_input(
            BenchmarkId::new("concurrent_validation", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let request = VectorRequest {
                        operation: VectorOperation::HealthCheck,
                        collection: None,
                        vectors: None,
                        metadata: None,
                        limit: None,
                    };

                    let mut handles = Vec::new();
                    for _ in 0..concurrency {
                        let config_clone = config.clone();
                        let request_clone = request.clone();
                        let handle = tokio::spawn(async move {
                            let client = QdrantClient::new(config_clone).await;
                            if let Ok(client) = client {
                                let _result = client.validate_request(&request_clone);
                            }
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

/// Performance regression detection benchmark
fn bench_regression_detection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("performance_baseline", |b| {
        b.to_async(&rt).iter(|| async {
            // Baseline performance test for regression detection
            let config = create_test_config();
            let embedding_config = create_embedding_config();

            // Simulate a typical workflow
            let start = std::time::Instant::now();

            // 1. Initialize services
            let embedding_service = LocalEmbeddingService::new(embedding_config);
            let _ = embedding_service.initialize().await;

            // 2. Generate some embeddings
            let text = "This is a sample text for embedding generation";
            let _embedding = embedding_service.generate_embedding(black_box(text)).await;

            // 3. Validate configuration
            let _validation = config.validate();

            let elapsed = start.elapsed();
            black_box(elapsed);
        });
    });
}

criterion_group!(
    benches,
    bench_client_initialization,
    bench_embedding_initialization,
    bench_request_validation,
    bench_cost_estimation,
    bench_vector_dimension_validation,
    bench_config_validation,
    bench_vector_memory_patterns,
    bench_concurrent_operations,
    bench_regression_detection
);

criterion_main!(benches);
