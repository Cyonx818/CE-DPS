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

//! Fortitude API Server - Cache Performance Benchmarks
//!
//! Specialized benchmarks to validate cache hit rate targets (>80%) and
//! measure cache effectiveness under various load conditions.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use futures::future::join_all;
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

/// Cache performance metrics
#[derive(Debug, Clone)]
struct CacheMetrics {
    cache_hits: usize,
    cache_misses: usize,
    response_times: Vec<Duration>,
    hit_response_times: Vec<Duration>,
    miss_response_times: Vec<Duration>,
}

impl CacheMetrics {
    fn new() -> Self {
        Self {
            cache_hits: 0,
            cache_misses: 0,
            response_times: Vec::new(),
            hit_response_times: Vec::new(),
            miss_response_times: Vec::new(),
        }
    }

    fn add_hit(&mut self, response_time: Duration) {
        self.cache_hits += 1;
        self.response_times.push(response_time);
        self.hit_response_times.push(response_time);
    }

    fn add_miss(&mut self, response_time: Duration) {
        self.cache_misses += 1;
        self.response_times.push(response_time);
        self.miss_response_times.push(response_time);
    }

    fn hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / total as f64) * 100.0
        }
    }

    fn average_hit_time(&self) -> Duration {
        if self.hit_response_times.is_empty() {
            Duration::ZERO
        } else {
            self.hit_response_times.iter().sum::<Duration>() / self.hit_response_times.len() as u32
        }
    }

    fn average_miss_time(&self) -> Duration {
        if self.miss_response_times.is_empty() {
            Duration::ZERO
        } else {
            self.miss_response_times.iter().sum::<Duration>()
                / self.miss_response_times.len() as u32
        }
    }

    fn cache_effectiveness(&self) -> f64 {
        let hit_time = self.average_hit_time();
        let miss_time = self.average_miss_time();

        if hit_time == Duration::ZERO || miss_time == Duration::ZERO {
            0.0
        } else {
            let improvement = miss_time.saturating_sub(hit_time);
            (improvement.as_millis() as f64 / miss_time.as_millis() as f64) * 100.0
        }
    }
}

/// Create HTTP client for cache testing
fn create_cache_test_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

/// Execute a single research request and measure response time
async fn single_research_request(
    client: &Client,
    base_url: &str,
    api_key: &str,
    query: &str,
) -> Result<Duration, Box<dyn std::error::Error + Send + Sync>> {
    let start = Instant::now();

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
    Ok(start.elapsed())
}

/// Test cache hit rate with repeated queries
async fn test_cache_hit_rate(
    base_url: &str,
    api_key: &str,
    query: &str,
    num_requests: usize,
) -> CacheMetrics {
    let client = create_cache_test_client();
    let mut metrics = CacheMetrics::new();

    // First request (cache miss)
    if let Ok(response_time) = single_research_request(&client, base_url, api_key, query).await {
        metrics.add_miss(response_time);
    }

    // Small delay to ensure first request completes
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Subsequent requests (should be cache hits)
    for _ in 1..num_requests {
        if let Ok(response_time) = single_research_request(&client, base_url, api_key, query).await
        {
            // Heuristic: if response time is significantly faster, consider it a cache hit
            let first_response_time = metrics
                .miss_response_times
                .first()
                .unwrap_or(&Duration::MAX);
            if response_time < *first_response_time / 2 {
                metrics.add_hit(response_time);
            } else {
                metrics.add_miss(response_time);
            }
        }

        // Small delay between requests
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    metrics
}

/// Test cache effectiveness under concurrent load
async fn concurrent_cache_test(
    base_url: &str,
    api_key: &str,
    queries: &[String],
    requests_per_query: usize,
    max_concurrency: usize,
) -> CacheMetrics {
    let client = create_cache_test_client();
    let metrics = Arc::new(Mutex::new(CacheMetrics::new()));
    let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrency));

    // Prime cache with each query
    for query in queries {
        let _ = single_research_request(&client, base_url, api_key, query).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let mut tasks = Vec::new();

    // Create concurrent requests for each query
    for query in queries {
        for _ in 0..requests_per_query {
            let client = client.clone();
            let base_url = base_url.to_string();
            let api_key = api_key.to_string();
            let query = query.clone();
            let metrics = metrics.clone();
            let semaphore = semaphore.clone();

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();

                if let Ok(response_time) =
                    single_research_request(&client, &base_url, &api_key, &query).await
                {
                    let mut metrics = metrics.lock().await;

                    // Heuristic: requests under 100ms are likely cache hits
                    if response_time.as_millis() < 100 {
                        metrics.add_hit(response_time);
                    } else {
                        metrics.add_miss(response_time);
                    }
                }
            });

            tasks.push(task);
        }
    }

    join_all(tasks).await;

    let metrics = metrics.lock().await;
    metrics.clone()
}

/// Test cache warming and subsequent performance
async fn cache_warming_test(
    base_url: &str,
    api_key: &str,
    num_warmup_requests: usize,
    num_test_requests: usize,
) -> (Vec<Duration>, Vec<Duration>) {
    let client = create_cache_test_client();
    let cache_query = "Cache warming test query for performance validation";

    let mut warmup_times = Vec::new();
    let mut test_times = Vec::new();

    // Warmup phase - populate cache
    for i in 0..num_warmup_requests {
        let unique_query = format!("{cache_query} - warmup {i}");
        if let Ok(response_time) =
            single_research_request(&client, base_url, api_key, &unique_query).await
        {
            warmup_times.push(response_time);
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Test phase - use cached queries
    for i in 0..num_test_requests {
        // Reuse queries from warmup phase
        let query_index = i % num_warmup_requests;
        let cached_query = format!("{cache_query} - warmup {query_index}");

        if let Ok(response_time) =
            single_research_request(&client, base_url, api_key, &cached_query).await
        {
            test_times.push(response_time);
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    (warmup_times, test_times)
}

/// Benchmark cache hit rate with single query
fn bench_cache_hit_rate_single_query(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let base_url =
        std::env::var("FORTITUDE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let api_key = std::env::var("FORTITUDE_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());

    let mut group = c.benchmark_group("cache_hit_rate_single");
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let test_configs = vec![
        (5, "Cache test query 5 requests"),
        (10, "Cache test query 10 requests"),
        (20, "Cache test query 20 requests"),
        (50, "Cache test query 50 requests"),
    ];

    for (num_requests, query) in test_configs {
        group.bench_with_input(
            BenchmarkId::new("repeated_requests", num_requests),
            &(num_requests, query),
            |b, &(num_requests, query)| {
                b.iter_custom(|_iters| {
                    let start = Instant::now();
                    let metrics = rt.block_on(async {
                        test_cache_hit_rate(
                            &base_url,
                            &api_key,
                            query,
                            num_requests,
                        ).await
                    });

                    // Validate Sprint 006 cache hit rate target
                    if metrics.hit_rate() < 80.0 && num_requests >= 10 {
                        let hit_rate = metrics.hit_rate();
                        eprintln!("WARNING: Cache hit rate target missed - {hit_rate:.1}% (target: >80%)");
                    }

                    // Print cache metrics for analysis
                    let hit_rate = metrics.hit_rate();
                    let avg_hit_time = metrics.average_hit_time().as_millis();
                    let avg_miss_time = metrics.average_miss_time().as_millis();
                    println!("Cache Metrics - Hit Rate: {hit_rate:.1}%, Avg Hit Time: {avg_hit_time:.1}ms, Avg Miss Time: {avg_miss_time:.1}ms");

                    start.elapsed()
                })
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent cache performance
fn bench_concurrent_cache_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let base_url =
        std::env::var("FORTITUDE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let api_key = std::env::var("FORTITUDE_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());

    let mut group = c.benchmark_group("concurrent_cache_performance");
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(5);

    let queries = vec![
        "Concurrent cache test query 1".to_string(),
        "Concurrent cache test query 2".to_string(),
        "Concurrent cache test query 3".to_string(),
        "Concurrent cache test query 4".to_string(),
        "Concurrent cache test query 5".to_string(),
    ];

    let test_configs = vec![
        (5, 10),  // 5 requests per query, 10 concurrent
        (10, 20), // 10 requests per query, 20 concurrent
        (20, 30), // 20 requests per query, 30 concurrent
    ];

    for (requests_per_query, max_concurrency) in test_configs {
        group.bench_with_input(
            BenchmarkId::new(
                "concurrent_cache",
                format!("{requests_per_query}req_{max_concurrency}conc"),
            ),
            &(requests_per_query, max_concurrency),
            |b, &(requests_per_query, max_concurrency)| {
                b.iter_custom(|_iters| {
                    let start = Instant::now();
                    let metrics = rt.block_on(async {
                        concurrent_cache_test(
                            &base_url,
                            &api_key,
                            &queries,
                            requests_per_query,
                            max_concurrency,
                        ).await
                    });

                    // Validate cache performance under load
                    if metrics.hit_rate() < 70.0 {
                        let hit_rate = metrics.hit_rate();
                        eprintln!(
                            "WARNING: Cache hit rate under load: {hit_rate:.1}% (target: >70% under load)"
                        );
                    }

                    if metrics.average_hit_time().as_millis() > 100 {
                        let hit_time = metrics.average_hit_time().as_millis();
                        eprintln!(
                            "WARNING: Cache hit response time: {hit_time}ms (target: <100ms)"
                        );
                    }

                    let hit_rate = metrics.hit_rate();
                    let effectiveness = metrics.cache_effectiveness();
                    println!(
                        "Concurrent Cache Metrics - Hit Rate: {hit_rate:.1}%, Effectiveness: {effectiveness:.1}%"
                    );

                    start.elapsed()
                })
            },
        );
    }

    group.finish();
}

/// Benchmark cache warming effectiveness
fn bench_cache_warming(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let base_url =
        std::env::var("FORTITUDE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let api_key = std::env::var("FORTITUDE_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());

    let mut group = c.benchmark_group("cache_warming");
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(25));
    group.sample_size(5);

    let test_configs = vec![
        (10, 20),  // 10 warmup, 20 test requests
        (20, 50),  // 20 warmup, 50 test requests
        (50, 100), // 50 warmup, 100 test requests
    ];

    for (warmup_requests, test_requests) in test_configs {
        group.bench_with_input(
            BenchmarkId::new(
                "cache_warming",
                format!("{warmup_requests}warmup_{test_requests}test"),
            ),
            &(warmup_requests, test_requests),
            |b, &(warmup_requests, test_requests)| {
                b.iter_custom(|_iters| {
                    let start = Instant::now();
                    let (warmup_times, test_times) = rt.block_on(async {
                        cache_warming_test(&base_url, &api_key, warmup_requests, test_requests).await
                    });

                    // Calculate performance improvement
                    if !warmup_times.is_empty() && !test_times.is_empty() {
                        let avg_warmup_time =
                            warmup_times.iter().sum::<Duration>() / warmup_times.len() as u32;
                        let avg_test_time =
                            test_times.iter().sum::<Duration>() / test_times.len() as u32;

                        let improvement = if avg_warmup_time > avg_test_time {
                            let diff = avg_warmup_time - avg_test_time;
                            (diff.as_millis() as f64 / avg_warmup_time.as_millis() as f64) * 100.0
                        } else {
                            0.0
                        };

                        let avg_warmup_ms = avg_warmup_time.as_millis();
                        let avg_test_ms = avg_test_time.as_millis();
                        println!(
                            "Cache Warming - Avg Warmup: {avg_warmup_ms}ms, Avg Test: {avg_test_ms}ms, Improvement: {improvement:.1}%"
                        );

                        // Validate cache warming effectiveness
                        if improvement < 30.0 {
                            eprintln!(
                                "WARNING: Cache warming improvement below expected: {improvement:.1}%"
                            );
                        }

                        if avg_test_time.as_millis() > 100 {
                            let avg_test_ms = avg_test_time.as_millis();
                            eprintln!(
                                "WARNING: Cached response time target missed: {avg_test_ms}ms"
                            );
                        }
                    }

                    start.elapsed()
                })
            },
        );
    }

    group.finish();
}

/// Benchmark cache performance under different query patterns
fn bench_cache_query_patterns(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let base_url =
        std::env::var("FORTITUDE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let api_key = std::env::var("FORTITUDE_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());

    let mut group = c.benchmark_group("cache_query_patterns");
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    // Test different query patterns
    let patterns = vec![
        ("repeated_exact", "Exact same query repeated"),
        (
            "repeated_similar",
            "Very similar query with minor variations",
        ),
        (
            "repeated_different",
            "Completely different queries each time",
        ),
    ];

    for (pattern_name, base_query) in patterns {
        group.bench_with_input(
            BenchmarkId::new("query_pattern", pattern_name),
            &(pattern_name, base_query),
            |b, &(pattern_name, base_query)| {
                b.iter_custom(|_iters| {
                    let start = Instant::now();
                    let response_times = rt.block_on(async {
                        let client = create_cache_test_client();
                        let mut response_times = Vec::new();

                        for i in 0..10 {
                            let query = match pattern_name {
                                "repeated_exact" => base_query.to_string(),
                                "repeated_similar" => format!("{base_query} {i}"),
                                "repeated_different" => format!("Unique query {i} for cache testing"),
                                _ => base_query.to_string(),
                            };

                            if let Ok(response_time) =
                                single_research_request(&client, &base_url, &api_key, &query).await
                            {
                                response_times.push(response_time);
                            }

                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                        response_times
                    });

                    // Analyze response time trends
                    if response_times.len() >= 3 {
                        let first_three_avg = response_times[0..3].iter().sum::<Duration>() / 3;
                        let last_three_avg = response_times[response_times.len() - 3..]
                            .iter()
                            .sum::<Duration>()
                            / 3;

                        let improvement = if first_three_avg > last_three_avg {
                            let diff = first_three_avg - last_three_avg;
                            (diff.as_millis() as f64 / first_three_avg.as_millis() as f64) * 100.0
                        } else {
                            0.0
                        };

                        let first_avg_ms = first_three_avg.as_millis();
                        let last_avg_ms = last_three_avg.as_millis();
                        println!(
                            "Pattern {pattern_name} - First 3 avg: {first_avg_ms}ms, Last 3 avg: {last_avg_ms}ms, Improvement: {improvement:.1}%"
                        );
                    }

                    start.elapsed()
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_cache_hit_rate_single_query,
    bench_concurrent_cache_performance,
    bench_cache_warming,
    bench_cache_query_patterns
);
criterion_main!(benches);
