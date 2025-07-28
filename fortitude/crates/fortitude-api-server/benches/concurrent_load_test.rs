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

//! Fortitude API Server - Concurrent Load Testing
//!
//! Specialized load testing benchmarks to validate 100+ concurrent request
//! handling and measure system behavior under high load conditions.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use futures::future::join_all;
use reqwest::Client;
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::Semaphore;
use tokio::time::sleep;

/// Load test results
#[derive(Debug, Clone)]
struct LoadTestResults {
    total_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    total_duration: Duration,
    average_response_time: Duration,
    min_response_time: Duration,
    max_response_time: Duration,
    response_times: Vec<Duration>,
}

impl LoadTestResults {
    fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_duration: Duration::ZERO,
            average_response_time: Duration::ZERO,
            min_response_time: Duration::MAX,
            max_response_time: Duration::ZERO,
            response_times: Vec::new(),
        }
    }

    fn add_result(&mut self, response_time: Duration, success: bool) {
        self.total_requests += 1;
        self.response_times.push(response_time);

        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }

        if response_time < self.min_response_time {
            self.min_response_time = response_time;
        }
        if response_time > self.max_response_time {
            self.max_response_time = response_time;
        }
    }

    fn finalize(&mut self, total_duration: Duration) {
        self.total_duration = total_duration;

        if !self.response_times.is_empty() {
            let total: Duration = self.response_times.iter().sum();
            self.average_response_time = total / self.response_times.len() as u32;
        }
    }

    fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }

    fn requests_per_second(&self) -> f64 {
        if self.total_duration.as_secs_f64() == 0.0 {
            0.0
        } else {
            self.total_requests as f64 / self.total_duration.as_secs_f64()
        }
    }

    #[allow(dead_code)]
    fn p95_response_time(&self) -> Duration {
        if self.response_times.is_empty() {
            return Duration::ZERO;
        }

        let mut sorted = self.response_times.clone();
        sorted.sort();
        let index = (0.95 * sorted.len() as f64) as usize;
        sorted[index.min(sorted.len() - 1)]
    }

    #[allow(dead_code)]
    fn p99_response_time(&self) -> Duration {
        if self.response_times.is_empty() {
            return Duration::ZERO;
        }

        let mut sorted = self.response_times.clone();
        sorted.sort();
        let index = (0.99 * sorted.len() as f64) as usize;
        sorted[index.min(sorted.len() - 1)]
    }
}

/// Create HTTP client optimized for load testing
fn create_load_test_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(50)
        .pool_idle_timeout(Duration::from_secs(30))
        .tcp_keepalive(Duration::from_secs(60))
        .build()
        .expect("Failed to create HTTP client")
}

/// Execute concurrent health check requests
async fn concurrent_health_check_test(
    num_requests: usize,
    max_concurrency: usize,
    base_url: &str,
) -> LoadTestResults {
    let client = create_load_test_client();
    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let results = Arc::new(std::sync::Mutex::new(LoadTestResults::new()));

    let start_time = Instant::now();

    let tasks: Vec<_> = (0..num_requests)
        .map(|_| {
            let client = client.clone();
            let base_url = base_url.to_string();
            let semaphore = semaphore.clone();
            let results = results.clone();

            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let request_start = Instant::now();

                let success = match client.get(format!("{base_url}/health")).send().await {
                    Ok(response) => response.status().is_success(),
                    Err(_) => false,
                };

                let response_time = request_start.elapsed();

                if let Ok(mut results) = results.lock() {
                    results.add_result(response_time, success);
                }
            })
        })
        .collect();

    join_all(tasks).await;

    let total_duration = start_time.elapsed();

    let final_results = match results.lock() {
        Ok(mut results) => {
            results.finalize(total_duration);
            results.clone()
        }
        Err(_) => LoadTestResults::new(),
    };
    final_results
}

/// Execute concurrent research requests with different queries
async fn concurrent_research_test(
    num_requests: usize,
    max_concurrency: usize,
    base_url: &str,
    api_key: &str,
) -> LoadTestResults {
    let client = create_load_test_client();
    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let results = Arc::new(std::sync::Mutex::new(LoadTestResults::new()));

    let queries = [
        "Rust async programming patterns",
        "Machine learning for text analysis",
        "Distributed systems architecture",
        "Performance optimization techniques",
        "Database design best practices",
        "Web security implementation",
        "API design principles",
        "Microservices communication",
        "Container orchestration strategies",
        "CI/CD pipeline optimization",
    ];

    let start_time = Instant::now();

    let tasks: Vec<_> = (0..num_requests)
        .map(|i| {
            let client = client.clone();
            let base_url = base_url.to_string();
            let api_key = api_key.to_string();
            let query = queries[i % queries.len()].to_string();
            let semaphore = semaphore.clone();
            let results = results.clone();

            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let request_start = Instant::now();

                let request_body = json!({
                    "query": format!("{query} - request {i}"),
                    "priority": "medium"
                });

                let success = match client
                    .post(format!("{base_url}/api/v1/research"))
                    .header("X-API-Key", api_key)
                    .header("Content-Type", "application/json")
                    .json(&request_body)
                    .send()
                    .await
                {
                    Ok(response) => response.status().is_success(),
                    Err(_) => false,
                };

                let response_time = request_start.elapsed();

                if let Ok(mut results) = results.lock() {
                    results.add_result(response_time, success);
                }
            })
        })
        .collect();

    join_all(tasks).await;

    let total_duration = start_time.elapsed();

    let final_results = match results.lock() {
        Ok(mut results) => {
            results.finalize(total_duration);
            results.clone()
        }
        Err(_) => LoadTestResults::new(),
    };
    final_results
}

/// Execute mixed workload test with different endpoint types
async fn concurrent_mixed_workload_test(
    num_requests: usize,
    max_concurrency: usize,
    base_url: &str,
    api_key: &str,
) -> LoadTestResults {
    let client = create_load_test_client();
    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let results = Arc::new(std::sync::Mutex::new(LoadTestResults::new()));

    let start_time = Instant::now();

    let tasks: Vec<_> = (0..num_requests)
        .map(|i| {
            let client = client.clone();
            let base_url = base_url.to_string();
            let api_key = api_key.to_string();
            let semaphore = semaphore.clone();
            let results = results.clone();

            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let request_start = Instant::now();

                let success = match i % 4 {
                    0 => {
                        // Health check
                        match client.get(format!("{base_url}/health")).send().await {
                            Ok(response) => response.status().is_success(),
                            Err(_) => false,
                        }
                    }
                    1 => {
                        // Protected health check
                        match client
                            .get(format!("{base_url}/api/v1/health/protected"))
                            .header("X-API-Key", &api_key)
                            .send()
                            .await
                        {
                            Ok(response) => response.status().is_success(),
                            Err(_) => false,
                        }
                    }
                    2 => {
                        // Research request
                        let request_body = json!({
                            "query": format!("Mixed workload research query {i}"),
                            "priority": "medium"
                        });

                        match client
                            .post(format!("{base_url}/api/v1/research"))
                            .header("X-API-Key", &api_key)
                            .header("Content-Type", "application/json")
                            .json(&request_body)
                            .send()
                            .await
                        {
                            Ok(response) => response.status().is_success(),
                            Err(_) => false,
                        }
                    }
                    3 => {
                        // Classification request
                        let request_body = json!({
                            "content": format!("Mixed workload classification content {i} for testing purposes")
                        });

                        match client
                            .post(format!("{base_url}/api/v1/classify"))
                            .header("X-API-Key", &api_key)
                            .header("Content-Type", "application/json")
                            .json(&request_body)
                            .send()
                            .await
                        {
                            Ok(response) => response.status().is_success(),
                            Err(_) => false,
                        }
                    }
                    _ => false,
                };

                let response_time = request_start.elapsed();

                if let Ok(mut results) = results.lock() {
                    results.add_result(response_time, success);
                }
            })
        })
        .collect();

    join_all(tasks).await;

    let total_duration = start_time.elapsed();

    let final_results = match results.lock() {
        Ok(mut results) => {
            results.finalize(total_duration);
            results.clone()
        }
        Err(_) => LoadTestResults::new(),
    };
    final_results
}

/// Sustained load test over time
async fn sustained_load_test(
    requests_per_second: usize,
    duration_seconds: u64,
    base_url: &str,
    api_key: &str,
) -> LoadTestResults {
    let client = create_load_test_client();
    let results = Arc::new(std::sync::Mutex::new(LoadTestResults::new()));
    let request_counter = Arc::new(AtomicUsize::new(0));

    let interval = Duration::from_millis(1000 / requests_per_second as u64);
    let start_time = Instant::now();
    let end_time = start_time + Duration::from_secs(duration_seconds);

    let mut tasks = Vec::new();

    while Instant::now() < end_time {
        let client = client.clone();
        let base_url = base_url.to_string();
        let _api_key = api_key.to_string();
        let results = results.clone();
        let counter = request_counter.clone();

        let task = tokio::spawn(async move {
            let request_start = Instant::now();
            let _request_id = counter.fetch_add(1, Ordering::SeqCst);

            let success = match client.get(format!("{base_url}/health")).send().await {
                Ok(response) => response.status().is_success(),
                Err(_) => false,
            };

            let response_time = request_start.elapsed();

            if let Ok(mut results) = results.lock() {
                results.add_result(response_time, success);
            }
        });

        tasks.push(task);
        sleep(interval).await;
    }

    join_all(tasks).await;

    let total_duration = start_time.elapsed();

    let final_results = match results.lock() {
        Ok(mut results) => {
            results.finalize(total_duration);
            results.clone()
        }
        Err(_) => LoadTestResults::new(),
    };
    final_results
}

/// Print load test results
#[allow(dead_code)]
fn print_load_test_results(test_name: &str, results: &LoadTestResults) {
    println!("\n📊 {test_name} Results:");
    println!("  Total Requests: {}", results.total_requests);
    println!(
        "  Successful: {} ({:.1}%)",
        results.successful_requests,
        results.success_rate()
    );
    println!("  Failed: {}", results.failed_requests);
    println!("  Duration: {:.2}s", results.total_duration.as_secs_f64());
    println!("  RPS: {:.1}", results.requests_per_second());
    println!(
        "  Avg Response Time: {:.1}ms",
        results.average_response_time.as_millis()
    );
    println!(
        "  Min Response Time: {:.1}ms",
        results.min_response_time.as_millis()
    );
    println!(
        "  Max Response Time: {:.1}ms",
        results.max_response_time.as_millis()
    );
    println!(
        "  P95 Response Time: {:.1}ms",
        results.p95_response_time().as_millis()
    );
    println!(
        "  P99 Response Time: {:.1}ms",
        results.p99_response_time().as_millis()
    );

    // Sprint 006 target validation
    println!("  🎯 Target Validation:");
    if results.success_rate() >= 95.0 {
        println!("    ✅ >95% success rate: PASSED");
    } else {
        println!(
            "    ❌ >95% success rate: FAILED ({:.1}%)",
            results.success_rate()
        );
    }

    if results.average_response_time.as_millis() < 100 {
        println!("    ✅ <100ms avg response: PASSED");
    } else {
        println!(
            "    ❌ <100ms avg response: FAILED ({}ms)",
            results.average_response_time.as_millis()
        );
    }

    if results.p95_response_time().as_millis() < 200 {
        println!("    ✅ <200ms P95 response: PASSED");
    } else {
        println!(
            "    ⚠️  <200ms P95 response: ACCEPTABLE ({}ms)",
            results.p95_response_time().as_millis()
        );
    }
}

/// Benchmark concurrent health checks at different scales
fn bench_concurrent_health_checks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let base_url =
        std::env::var("FORTITUDE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let mut group = c.benchmark_group("concurrent_health_checks");
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10); // Fewer samples for expensive concurrent tests

    // Test different concurrency levels and request counts
    let test_configs = vec![
        (50, 10),  // 50 requests, 10 concurrent
        (100, 10), // 100 requests, 10 concurrent
        (100, 20), // 100 requests, 20 concurrent
        (200, 20), // 200 requests, 20 concurrent
        (500, 50), // 500 requests, 50 concurrent
    ];

    for (num_requests, max_concurrency) in test_configs {
        group.bench_with_input(
            BenchmarkId::new(
                "health_checks",
                format!("{num_requests}req_{max_concurrency}conc"),
            ),
            &(num_requests, max_concurrency),
            |b, &(num_requests, max_concurrency)| {
                b.iter_custom(|_iters| {
                    let start = Instant::now();
                    let results = rt.block_on(async {
                        concurrent_health_check_test(num_requests, max_concurrency, &base_url).await
                    });

                    // Validate Sprint 006 targets during benchmark
                    if num_requests >= 100 && results.success_rate() < 95.0 {
                        eprintln!(
                            "WARNING: Sprint 006 target missed - {}% success rate",
                            results.success_rate()
                        );
                    }

                    start.elapsed()
                })
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent research requests
fn bench_concurrent_research_requests(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let base_url =
        std::env::var("FORTITUDE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let api_key = std::env::var("FORTITUDE_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());

    let mut group = c.benchmark_group("concurrent_research_requests");
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(45));
    group.sample_size(5); // Fewer samples for expensive research requests

    let test_configs = vec![
        (20, 5),   // 20 requests, 5 concurrent
        (50, 10),  // 50 requests, 10 concurrent
        (100, 15), // 100 requests, 15 concurrent
    ];

    for (num_requests, max_concurrency) in test_configs {
        group.bench_with_input(
            BenchmarkId::new(
                "research_requests",
                format!("{num_requests}req_{max_concurrency}conc"),
            ),
            &(num_requests, max_concurrency),
            |b, &(num_requests, max_concurrency)| {
                b.iter_custom(|_iters| {
                    let start = Instant::now();
                    let results = rt.block_on(async {
                        concurrent_research_test(num_requests, max_concurrency, &base_url, &api_key)
                            .await
                    });

                    if results.success_rate() < 90.0 {
                        eprintln!(
                            "WARNING: Low success rate for research requests: {}%",
                            results.success_rate()
                        );
                    }

                    start.elapsed()
                })
            },
        );
    }

    group.finish();
}

/// Benchmark mixed workload scenarios
fn bench_mixed_workload(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let base_url =
        std::env::var("FORTITUDE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let api_key = std::env::var("FORTITUDE_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());

    let mut group = c.benchmark_group("mixed_workload");
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);

    let test_configs = vec![
        (100, 15), // 100 mixed requests, 15 concurrent
        (200, 25), // 200 mixed requests, 25 concurrent
        (500, 50), // 500 mixed requests, 50 concurrent
    ];

    for (num_requests, max_concurrency) in test_configs {
        group.bench_with_input(
            BenchmarkId::new(
                "mixed_requests",
                format!("{num_requests}req_{max_concurrency}conc"),
            ),
            &(num_requests, max_concurrency),
            |b, &(num_requests, max_concurrency)| {
                b.iter_custom(|_iters| {
                    let start = Instant::now();
                    let results = rt.block_on(async {
                        concurrent_mixed_workload_test(
                            num_requests,
                            max_concurrency,
                            &base_url,
                            &api_key,
                        )
                        .await
                    });

                    if results.success_rate() < 95.0 {
                        eprintln!(
                            "WARNING: Mixed workload success rate below target: {}%",
                            results.success_rate()
                        );
                    }

                    start.elapsed()
                })
            },
        );
    }

    group.finish();
}

/// Benchmark sustained load over time
fn bench_sustained_load(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let base_url =
        std::env::var("FORTITUDE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let api_key = std::env::var("FORTITUDE_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());

    let mut group = c.benchmark_group("sustained_load");
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(3); // Very few samples for long-running sustained tests

    let test_configs = vec![
        (10, 30), // 10 RPS for 30 seconds
        (20, 30), // 20 RPS for 30 seconds
        (50, 15), // 50 RPS for 15 seconds
    ];

    for (rps, duration) in test_configs {
        group.bench_with_input(
            BenchmarkId::new("sustained_rps", format!("{rps}rps_{duration}s")),
            &(rps, duration),
            |b, &(rps, duration)| {
                b.iter_custom(|_iters| {
                    let start = Instant::now();
                    let results = rt.block_on(async {
                        sustained_load_test(
                            rps,
                            duration,
                            &base_url,
                            &api_key,
                        ).await
                    });

                    if results.requests_per_second() < rps as f64 * 0.8 {
                        eprintln!("WARNING: Sustained load target missed - {:.1} RPS achieved vs {} target",
                                  results.requests_per_second(), rps);
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
    bench_concurrent_health_checks,
    bench_concurrent_research_requests,
    bench_mixed_workload,
    bench_sustained_load
);
criterion_main!(benches);
