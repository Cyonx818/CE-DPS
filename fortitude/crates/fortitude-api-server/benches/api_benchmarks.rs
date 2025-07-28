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

//! Fortitude API Server - Performance Benchmarks
//!
//! Comprehensive benchmark suite using criterion to validate API performance
//! against Sprint 006 targets including sub-100ms response times and high throughput.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use futures::future;
use reqwest::Client;
use serde_json::json;
use std::sync::Once;
use std::time::Duration;
use tokio::runtime::Runtime;

static INIT: Once = Once::new();

/// Initialize test environment
fn init_test_env() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter("error") // Reduce noise during benchmarks
            .try_init()
            .ok();
    });
}

/// Create a test HTTP client
fn create_test_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

/// Single health check benchmark
async fn health_check_single(
    client: &Client,
    base_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.get(format!("{base_url}/health")).send().await?;

    response.error_for_status()?;
    Ok(())
}

/// Single protected health check benchmark
async fn protected_health_check_single(
    client: &Client,
    base_url: &str,
    api_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client
        .get(format!("{base_url}/api/v1/health/protected"))
        .header("X-API-Key", api_key)
        .send()
        .await?;

    response.error_for_status()?;
    Ok(())
}

/// Single research request benchmark
async fn research_request_single(
    client: &Client,
    base_url: &str,
    api_key: &str,
    query: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_body = json!({
        "query": query,
        "priority": "medium"
    });

    let response = client
        .post(format!("{base_url}/api/v1/research"))
        .header("X-API-Key", api_key)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    response.error_for_status()?;
    Ok(())
}

/// Single classification request benchmark
async fn classification_request_single(
    client: &Client,
    base_url: &str,
    api_key: &str,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_body = json!({
        "content": content
    });

    let response = client
        .post(format!("{base_url}/api/v1/classify"))
        .header("X-API-Key", api_key)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    response.error_for_status()?;
    Ok(())
}

/// Single cache stats request benchmark
async fn cache_stats_single(
    client: &Client,
    base_url: &str,
    api_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client
        .get(format!("{base_url}/api/v1/cache/stats"))
        .header("X-API-Key", api_key)
        .send()
        .await?;

    response.error_for_status()?;
    Ok(())
}

/// Health endpoint benchmarks
fn bench_health_endpoints(c: &mut Criterion) {
    init_test_env();

    let rt = Runtime::new().unwrap();
    let client = create_test_client();
    let base_url =
        std::env::var("FORTITUDE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let api_key = std::env::var("FORTITUDE_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());

    let mut group = c.benchmark_group("health_endpoints");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    // Public health check benchmark
    group.bench_function("public_health_check", |b| {
        b.iter(|| {
            rt.block_on(async {
                health_check_single(black_box(&client), black_box(&base_url))
                    .await
                    .unwrap()
            })
        })
    });

    // Protected health check benchmark
    group.bench_function("protected_health_check", |b| {
        b.iter(|| {
            rt.block_on(async {
                protected_health_check_single(
                    black_box(&client),
                    black_box(&base_url),
                    black_box(&api_key),
                )
                .await
                .unwrap()
            })
        })
    });

    group.finish();
}

/// Research endpoint benchmarks
fn bench_research_endpoints(c: &mut Criterion) {
    init_test_env();

    let rt = Runtime::new().unwrap();
    let client = create_test_client();
    let base_url =
        std::env::var("FORTITUDE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let api_key = std::env::var("FORTITUDE_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());

    let mut group = c.benchmark_group("research_endpoints");
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(50); // Fewer samples for more expensive operations

    // Test different query types
    let queries = [
        "Simple query",
        "Rust async programming best practices",
        "Machine learning algorithms for text classification",
        "Performance optimization techniques for web servers",
        "Database connection pooling strategies",
    ];

    for (i, query) in queries.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("research_request", i),
            query,
            |b, query| {
                b.iter(|| {
                    rt.block_on(async {
                        research_request_single(
                            black_box(&client),
                            black_box(&base_url),
                            black_box(&api_key),
                            black_box(query),
                        )
                        .await
                        .unwrap()
                    })
                })
            },
        );
    }

    group.finish();
}

/// Classification endpoint benchmarks
fn bench_classification_endpoints(c: &mut Criterion) {
    init_test_env();

    let rt = Runtime::new().unwrap();
    let client = create_test_client();
    let base_url =
        std::env::var("FORTITUDE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let api_key = std::env::var("FORTITUDE_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());

    let mut group = c.benchmark_group("classification_endpoints");
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(50);

    // Test different content types
    let contents = [
        "Short text for classification",
        "This is a technical document about implementing async functions in Rust with proper error handling.",
        "How to optimize database queries for better performance? I need help with indexing strategies and query planning techniques.",
        "## Technical Documentation\n\nThis guide explains advanced concepts in distributed systems architecture, focusing on scalability patterns and fault tolerance mechanisms.",
        "URGENT: Production system is experiencing high latency. Need immediate assistance with performance debugging and optimization strategies.",
    ];

    for (i, content) in contents.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("classification_request", i),
            content,
            |b, content| {
                b.iter(|| {
                    rt.block_on(async {
                        classification_request_single(
                            black_box(&client),
                            black_box(&base_url),
                            black_box(&api_key),
                            black_box(content),
                        )
                        .await
                        .unwrap()
                    })
                })
            },
        );
    }

    group.finish();
}

/// Cache endpoint benchmarks
fn bench_cache_endpoints(c: &mut Criterion) {
    init_test_env();

    let rt = Runtime::new().unwrap();
    let client = create_test_client();
    let base_url =
        std::env::var("FORTITUDE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let api_key = std::env::var("FORTITUDE_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());

    let mut group = c.benchmark_group("cache_endpoints");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    // Cache statistics benchmark
    group.bench_function("cache_stats", |b| {
        b.iter(|| {
            rt.block_on(async {
                cache_stats_single(
                    black_box(&client),
                    black_box(&base_url),
                    black_box(&api_key),
                )
                .await
                .unwrap()
            })
        })
    });

    group.finish();
}

/// Cache hit rate benchmarks - test repeated requests
fn bench_cache_hit_rate(c: &mut Criterion) {
    init_test_env();

    let rt = Runtime::new().unwrap();
    let client = create_test_client();
    let base_url =
        std::env::var("FORTITUDE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let api_key = std::env::var("FORTITUDE_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());

    let mut group = c.benchmark_group("cache_hit_rate");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    let cache_test_query = "Cache effectiveness benchmark query - should benefit from caching";

    // Prime the cache first
    rt.block_on(async {
        let _ = research_request_single(&client, &base_url, &api_key, cache_test_query).await;
    });

    // Benchmark cached requests
    group.bench_function("cached_research_request", |b| {
        b.iter(|| {
            rt.block_on(async {
                research_request_single(
                    black_box(&client),
                    black_box(&base_url),
                    black_box(&api_key),
                    black_box(cache_test_query),
                )
                .await
                .unwrap()
            })
        })
    });

    // Compare with fresh requests
    group.bench_function("fresh_research_request", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            rt.block_on(async {
                for i in 0..iters {
                    let unique_query = format!("Fresh query {i} for benchmark comparison");
                    research_request_single(
                        black_box(&client),
                        black_box(&base_url),
                        black_box(&api_key),
                        black_box(&unique_query),
                    )
                    .await
                    .unwrap();
                }
            });
            start.elapsed()
        })
    });

    group.finish();
}

/// Throughput benchmarks with different request volumes
fn bench_throughput(c: &mut Criterion) {
    init_test_env();

    let rt = Runtime::new().unwrap();
    let client = create_test_client();
    let base_url =
        std::env::var("FORTITUDE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let _api_key =
        std::env::var("FORTITUDE_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());

    let mut group = c.benchmark_group("throughput");
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(20));

    // Test different batch sizes
    for batch_size in [1, 5, 10, 20, 50].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("health_check_batch", batch_size),
            batch_size,
            |b, &batch_size| {
                b.iter_custom(|_iters| {
                    let start = std::time::Instant::now();
                    rt.block_on(async {
                        let mut tasks = Vec::with_capacity(batch_size);
                        for _ in 0..batch_size {
                            let client = client.clone();
                            let base_url = base_url.clone();
                            tasks.push(tokio::spawn(async move {
                                health_check_single(&client, &base_url).await.unwrap()
                            }));
                        }
                        future::join_all(tasks).await;
                    });
                    start.elapsed()
                })
            },
        );
    }

    group.finish();
}

/// Sprint 006 performance target validation benchmarks
fn bench_sprint_006_targets(c: &mut Criterion) {
    init_test_env();

    let rt = Runtime::new().unwrap();
    let client = create_test_client();
    let base_url =
        std::env::var("FORTITUDE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let api_key = std::env::var("FORTITUDE_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());

    let mut group = c.benchmark_group("sprint_006_targets");
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(100);

    // Target: Sub-100ms response time for cached requests
    let cache_query = "Sprint 006 cache target validation query";

    // Prime the cache
    rt.block_on(async {
        let _ = research_request_single(&client, &base_url, &api_key, cache_query).await;
    });

    group.bench_function("sub_100ms_cached_target", |b| {
        b.iter(|| {
            let start = std::time::Instant::now();
            rt.block_on(async {
                research_request_single(
                    black_box(&client),
                    black_box(&base_url),
                    black_box(&api_key),
                    black_box(cache_query),
                )
                .await
                .unwrap();
            });
            let elapsed = start.elapsed();

            // Validate target in benchmark
            if elapsed.as_millis() > 100 {
                let elapsed_ms = elapsed.as_millis();
                eprintln!("WARNING: Response time {elapsed_ms}ms exceeds 100ms target");
            }
        })
    });

    // Target: 100+ concurrent requests support
    group.bench_function("concurrent_100_requests_target", |b| {
        b.iter_custom(|_iters| {
            let start = std::time::Instant::now();
            rt.block_on(async {
                let mut tasks = Vec::with_capacity(100);
                for _i in 0..100 {
                    let client = client.clone();
                    let base_url = base_url.clone();
                    tasks.push(tokio::spawn(async move {
                        health_check_single(&client, &base_url).await.unwrap()
                    }));
                }

                let results = future::join_all(tasks).await;
                let success_count = results.iter().filter(|r| r.is_ok()).count();

                if success_count < 95 {
                    eprintln!(
                        "WARNING: Only {success_count}/100 requests succeeded (target: 95%+)"
                    );
                }
            });
            start.elapsed()
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_health_endpoints,
    bench_research_endpoints,
    bench_classification_endpoints,
    bench_cache_endpoints,
    bench_cache_hit_rate,
    bench_throughput,
    bench_sprint_006_targets
);
criterion_main!(benches);
