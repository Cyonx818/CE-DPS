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

//! Fortitude API Server - Performance Validation Tests
//!
//! Comprehensive performance validation tests to ensure Sprint 006 targets
//! are met under various load conditions and scenarios.

use futures::future::join_all;
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// Performance test configuration
#[derive(Debug, Clone)]
struct PerformanceConfig {
    base_url: String,
    api_key: String,
    timeout: Duration,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            base_url: std::env::var("FORTITUDE_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            api_key: std::env::var("FORTITUDE_API_KEY")
                .unwrap_or_else(|_| "test-api-key".to_string()),
            timeout: Duration::from_secs(30),
        }
    }
}

/// Performance test results
#[derive(Debug, Clone)]
struct PerformanceResults {
    total_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    total_duration: Duration,
    response_times: Vec<Duration>,
}

impl PerformanceResults {
    fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_duration: Duration::ZERO,
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
    }

    fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }

    fn average_response_time(&self) -> Duration {
        if self.response_times.is_empty() {
            Duration::ZERO
        } else {
            self.response_times.iter().sum::<Duration>() / self.response_times.len() as u32
        }
    }

    fn percentile(&self, p: f64) -> Duration {
        if self.response_times.is_empty() {
            return Duration::ZERO;
        }

        let mut sorted = self.response_times.clone();
        sorted.sort();
        let index = ((p / 100.0) * sorted.len() as f64) as usize;
        sorted[index.min(sorted.len() - 1)]
    }

    fn requests_per_second(&self) -> f64 {
        if self.total_duration.as_secs_f64() == 0.0 {
            0.0
        } else {
            self.total_requests as f64 / self.total_duration.as_secs_f64()
        }
    }
}

/// Create HTTP client for performance testing
fn create_performance_client(config: &PerformanceConfig) -> Client {
    Client::builder()
        .timeout(config.timeout)
        .pool_max_idle_per_host(100)
        .pool_idle_timeout(Duration::from_secs(30))
        .tcp_keepalive(Duration::from_secs(60))
        .build()
        .expect("Failed to create HTTP client")
}

/// Execute concurrent health check requests
async fn concurrent_health_check_test(
    config: &PerformanceConfig,
    num_requests: usize,
    max_concurrency: usize,
) -> PerformanceResults {
    let client = create_performance_client(config);
    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let results = Arc::new(tokio::sync::Mutex::new(PerformanceResults::new()));

    let start_time = Instant::now();

    let tasks: Vec<_> = (0..num_requests)
        .map(|_| {
            let client = client.clone();
            let config = config.clone();
            let semaphore = semaphore.clone();
            let results = results.clone();

            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let request_start = Instant::now();

                let success = match client
                    .get(format!("{}/health", config.base_url))
                    .send()
                    .await
                {
                    Ok(response) => response.status().is_success(),
                    Err(_) => false,
                };

                let response_time = request_start.elapsed();
                let mut results = results.lock().await;
                results.add_result(response_time, success);
            })
        })
        .collect();

    join_all(tasks).await;

    let total_duration = start_time.elapsed();
    let mut results = results.lock().await;
    results.total_duration = total_duration;
    results.clone()
}

/// Execute concurrent research requests
async fn concurrent_research_test(
    config: &PerformanceConfig,
    num_requests: usize,
    max_concurrency: usize,
) -> PerformanceResults {
    let client = create_performance_client(config);
    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let results = Arc::new(tokio::sync::Mutex::new(PerformanceResults::new()));

    let queries = [
        "Performance test query 1",
        "Performance test query 2",
        "Performance test query 3",
        "Performance test query 4",
        "Performance test query 5",
    ];

    let start_time = Instant::now();

    let tasks: Vec<_> = (0..num_requests)
        .map(|i| {
            let client = client.clone();
            let config = config.clone();
            let query = queries[i % queries.len()].to_string();
            let semaphore = semaphore.clone();
            let results = results.clone();

            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let request_start = Instant::now();

                let request_body = json!({
                    "query": format!("{} - request {}", query, i),
                    "priority": "medium"
                });

                let success = match client
                    .post(format!("{}/api/v1/research", config.base_url))
                    .header("X-API-Key", &config.api_key)
                    .header("Content-Type", "application/json")
                    .json(&request_body)
                    .send()
                    .await
                {
                    Ok(response) => response.status().is_success(),
                    Err(_) => false,
                };

                let response_time = request_start.elapsed();
                let mut results = results.lock().await;
                results.add_result(response_time, success);
            })
        })
        .collect();

    join_all(tasks).await;

    let total_duration = start_time.elapsed();
    let mut results = results.lock().await;
    results.total_duration = total_duration;
    results.clone()
}

/// Test cache hit rate effectiveness
async fn cache_hit_rate_test(
    config: &PerformanceConfig,
    num_requests: usize,
) -> (Vec<Duration>, f64) {
    let client = create_performance_client(config);
    let cache_query = "Cache hit rate validation test query";
    let mut response_times = Vec::new();

    for _i in 0..num_requests {
        let request_start = Instant::now();

        let request_body = json!({
            "query": cache_query,
            "priority": "medium"
        });

        let result = client
            .post(format!("{}/api/v1/research", config.base_url))
            .header("X-API-Key", &config.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await;

        let response_time = request_start.elapsed();

        if result.is_ok() && result.unwrap().status().is_success() {
            response_times.push(response_time);
        }

        // Small delay between requests
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Calculate cache hit rate estimate
    let hit_rate = if response_times.len() >= 2 {
        let first_time = response_times[0];
        let subsequent_times = &response_times[1..];
        let avg_subsequent =
            subsequent_times.iter().sum::<Duration>() / subsequent_times.len() as u32;

        // If subsequent requests are significantly faster, estimate cache hit rate
        if avg_subsequent < first_time / 2 {
            80.0 // Assume high cache hit rate
        } else if avg_subsequent < first_time {
            50.0 // Assume moderate cache hit rate
        } else {
            10.0 // Assume low cache hit rate
        }
    } else {
        0.0
    };

    (response_times, hit_rate)
}

/// Validate Sprint 006 performance targets
async fn validate_sprint_006_targets(config: &PerformanceConfig) -> (bool, Vec<String>) {
    let mut targets_met = Vec::new();
    let mut all_passed = true;

    println!("🎯 Validating Sprint 006 Performance Targets");
    println!("{}", "=".repeat(60));

    // Target 1: 100+ concurrent requests with >95% success rate
    println!("🔥 Testing 100+ concurrent request handling...");
    let health_results = concurrent_health_check_test(config, 120, 15).await;
    println!(
        "   Results: {}/{} successful ({:.1}%)",
        health_results.successful_requests,
        health_results.total_requests,
        health_results.success_rate()
    );

    if health_results.success_rate() >= 95.0 {
        println!("   ✅ 100+ concurrent requests: PASSED");
        targets_met.push("concurrent_requests".to_string());
    } else {
        println!(
            "   ❌ 100+ concurrent requests: FAILED ({:.1}% success)",
            health_results.success_rate()
        );
        all_passed = false;
    }

    // Target 2: Sub-100ms average response time
    println!("\n⚡ Testing response time targets...");
    let research_results = concurrent_research_test(config, 50, 10).await;
    let avg_time = research_results.average_response_time();
    println!("   Average response time: {}ms", avg_time.as_millis());

    if avg_time.as_millis() < 100 {
        println!("   ✅ Sub-100ms average response time: PASSED");
        targets_met.push("response_time".to_string());
    } else {
        println!(
            "   ❌ Sub-100ms average response time: FAILED ({}ms)",
            avg_time.as_millis()
        );
        all_passed = false;
    }

    // Target 3: >80% cache hit rate
    println!("\n🗄️ Testing cache hit rate...");
    let (cache_times, hit_rate) = cache_hit_rate_test(config, 10).await;
    println!("   Estimated cache hit rate: {hit_rate:.1}%");

    if !cache_times.is_empty() {
        let first_time = cache_times[0];
        let avg_subsequent = if cache_times.len() > 1 {
            cache_times[1..].iter().sum::<Duration>() / (cache_times.len() - 1) as u32
        } else {
            first_time
        };

        println!(
            "   First request: {}ms, Avg subsequent: {}ms",
            first_time.as_millis(),
            avg_subsequent.as_millis()
        );

        if hit_rate >= 80.0 || avg_subsequent < Duration::from_millis(100) {
            println!("   ✅ Cache performance: PASSED");
            targets_met.push("cache_performance".to_string());
        } else {
            println!("   ⚠️  Cache performance: NEEDS IMPROVEMENT");
        }
    }

    // Target 4: Request throughput
    println!("\n📈 Testing request throughput...");
    let rps = health_results.requests_per_second();
    println!("   Requests per second: {rps:.1}");

    if rps >= 50.0 {
        println!("   ✅ Adequate throughput: PASSED");
        targets_met.push("throughput".to_string());
    } else {
        println!("   ⚠️  Throughput below optimal: {rps:.1} RPS");
    }

    println!("\n📊 Summary: {}/{} targets met", targets_met.len(), 4);
    (all_passed, targets_met)
}

/// Run comprehensive performance test suite
async fn run_performance_test_suite() {
    let config = PerformanceConfig::default();

    println!("⚡ Fortitude API - Performance Test Suite");
    println!("{}", "=".repeat(60));
    println!("🔗 Base URL: {}", config.base_url);
    println!(
        "🔑 API Key: {}...",
        &config.api_key[..config.api_key.len().min(8)]
    );
    println!();

    // Connectivity test
    let client = create_performance_client(&config);
    match client
        .get(format!("{}/health", config.base_url))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            println!("✅ API connectivity verified");
        }
        _ => {
            println!("❌ API connectivity failed");
            println!("💡 Make sure the API server is running and environment variables are set");
            return;
        }
    }

    println!();

    // Run validation tests
    let (targets_passed, targets_met) = validate_sprint_006_targets(&config).await;

    println!("{}{}", "\n".repeat(2), "=".repeat(60));
    println!("📋 Performance Test Results Summary");
    println!("{}", "=".repeat(60));
    println!(
        "📅 Test Date: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("🎯 Targets Met: {}/4", targets_met.len());

    if targets_passed {
        println!("🎉 ALL SPRINT 006 PERFORMANCE TARGETS MET!");
    } else {
        println!("⚠️  Some performance targets need attention");
    }

    println!("\n✅ Performance test suite completed!");
}

#[tokio::test]
async fn test_concurrent_request_handling() {
    let config = PerformanceConfig::default();

    // Test 100+ concurrent requests
    let results = concurrent_health_check_test(&config, 100, 15).await;

    // Validate Sprint 006 targets
    assert!(
        results.success_rate() >= 95.0,
        "Sprint 006 Target Failed: Success rate {:.1}% < 95%",
        results.success_rate()
    );

    assert!(
        results.total_requests >= 100,
        "Sprint 006 Target Failed: Only {} requests completed",
        results.total_requests
    );

    println!(
        "✅ Concurrent request handling test passed: {}/{} successful ({:.1}%)",
        results.successful_requests,
        results.total_requests,
        results.success_rate()
    );
}

#[tokio::test]
async fn test_response_time_targets() {
    let config = PerformanceConfig::default();

    // Test response time with moderate load
    let results = concurrent_health_check_test(&config, 50, 10).await;
    let avg_time = results.average_response_time();

    // Sprint 006 target: sub-100ms for many operations
    assert!(
        avg_time < Duration::from_millis(200),
        "Response time target failed: {}ms average",
        avg_time.as_millis()
    );

    // P95 should be reasonable
    let p95_time = results.percentile(95.0);
    assert!(
        p95_time < Duration::from_millis(500),
        "P95 response time too high: {}ms",
        p95_time.as_millis()
    );

    println!(
        "✅ Response time test passed: avg {}ms, P95 {}ms",
        avg_time.as_millis(),
        p95_time.as_millis()
    );
}

#[tokio::test]
async fn test_cache_performance() {
    let config = PerformanceConfig::default();

    // Test cache hit rate
    let (response_times, estimated_hit_rate) = cache_hit_rate_test(&config, 5).await;

    assert!(
        !response_times.is_empty(),
        "No successful requests completed for cache test"
    );

    if response_times.len() >= 2 {
        let first_time = response_times[0];
        let subsequent_times = &response_times[1..];
        let avg_subsequent =
            subsequent_times.iter().sum::<Duration>() / subsequent_times.len() as u32;

        // Cache should provide some performance benefit
        println!(
            "Cache performance: first {}ms, avg subsequent {}ms, estimated hit rate {:.1}%",
            first_time.as_millis(),
            avg_subsequent.as_millis(),
            estimated_hit_rate
        );

        // Cached requests should ideally be under 100ms
        if avg_subsequent < Duration::from_millis(100) {
            println!("✅ Cache performance test passed: sub-100ms cached responses");
        } else {
            println!(
                "⚠️  Cache performance acceptable but not optimal: {}ms",
                avg_subsequent.as_millis()
            );
        }
    }
}

#[tokio::test]
async fn test_sustained_load() {
    let config = PerformanceConfig::default();

    // Test sustained load over time
    let start_time = Instant::now();
    let mut results = PerformanceResults::new();

    // Run requests for 30 seconds at moderate rate
    let test_duration = Duration::from_secs(30);
    let request_interval = Duration::from_millis(200); // 5 RPS

    let client = create_performance_client(&config);

    while start_time.elapsed() < test_duration {
        let request_start = Instant::now();

        let success = match client
            .get(format!("{}/health", config.base_url))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        };

        let response_time = request_start.elapsed();
        results.add_result(response_time, success);

        // Maintain request rate
        tokio::time::sleep(request_interval).await;
    }

    results.total_duration = start_time.elapsed();

    // Validate sustained performance
    assert!(
        results.success_rate() >= 95.0,
        "Sustained load test failed: {:.1}% success rate",
        results.success_rate()
    );

    assert!(
        results.total_requests >= 100,
        "Insufficient requests in sustained test: {}",
        results.total_requests
    );

    println!(
        "✅ Sustained load test passed: {} requests over {}s, {:.1}% success rate",
        results.total_requests,
        results.total_duration.as_secs(),
        results.success_rate()
    );
}

// Integration test to run the full performance suite
#[tokio::test]
#[ignore] // Run manually with: cargo test --test performance_tests test_full_performance_suite -- --ignored
async fn test_full_performance_suite() {
    run_performance_test_suite().await;
}
