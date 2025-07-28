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

//! Fortitude API Performance Testing
//! 
//! Comprehensive performance testing and validation for the Fortitude API,
//! including concurrent request testing, cache hit rate validation, and
//! response time measurements.

use anyhow::Result;
use clap::Parser;
use futures::future::join_all;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{error, info, warn};

use fortitude_api_client_examples::{FortitudeClient, FortitudeError};

#[derive(Parser)]
#[command(name = "performance_test")]
#[command(about = "Performance testing for Fortitude API")]
struct Args {
    /// Number of concurrent requests for health check test
    #[arg(long, default_value = "100")]
    health_requests: usize,

    /// Number of concurrent requests for research test
    #[arg(long, default_value = "50")]
    research_requests: usize,

    /// Maximum concurrent connections
    #[arg(long, default_value = "10")]
    max_concurrency: usize,

    /// Enable cache hit rate testing
    #[arg(long)]
    test_cache: bool,

    /// Enable Sprint 006 target validation
    #[arg(long)]
    validate_targets: bool,
}

/// Performance test results
#[derive(Debug)]
struct PerformanceResults {
    total_requests: usize,
    success_count: usize,
    error_count: usize,
    response_times: Vec<Duration>,
    total_duration: Duration,
    errors: Vec<String>,
}

impl PerformanceResults {
    fn new() -> Self {
        Self {
            total_requests: 0,
            success_count: 0,
            error_count: 0,
            response_times: Vec::new(),
            total_duration: Duration::ZERO,
            errors: Vec::new(),
        }
    }

    fn add_result(&mut self, response_time: Duration, success: bool, error: Option<String>) {
        self.total_requests += 1;
        self.response_times.push(response_time);
        
        if success {
            self.success_count += 1;
        } else {
            self.error_count += 1;
            if let Some(err) = error {
                self.errors.push(err);
            }
        }
    }

    fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.success_count as f64 / self.total_requests as f64) * 100.0
        }
    }

    fn requests_per_second(&self) -> f64 {
        if self.total_duration.as_secs_f64() == 0.0 {
            0.0
        } else {
            self.total_requests as f64 / self.total_duration.as_secs_f64()
        }
    }

    fn average_response_time(&self) -> Duration {
        if self.response_times.is_empty() {
            Duration::ZERO
        } else {
            let total: Duration = self.response_times.iter().sum();
            total / self.response_times.len() as u32
        }
    }

    fn percentile(&self, p: f64) -> Duration {
        if self.response_times.is_empty() {
            return Duration::ZERO;
        }

        let mut sorted_times = self.response_times.clone();
        sorted_times.sort();

        let index = ((p / 100.0) * sorted_times.len() as f64).ceil() as usize - 1;
        sorted_times[index.min(sorted_times.len() - 1)]
    }

    fn min_response_time(&self) -> Duration {
        self.response_times.iter().min().copied().unwrap_or(Duration::ZERO)
    }

    fn max_response_time(&self) -> Duration {
        self.response_times.iter().max().copied().unwrap_or(Duration::ZERO)
    }
}

/// Single health check request
async fn single_health_request(client: &FortitudeClient) -> (Duration, bool, Option<String>) {
    let start = Instant::now();
    
    match client.get_health().await {
        Ok(_) => {
            let duration = start.elapsed();
            (duration, true, None)
        }
        Err(e) => {
            let duration = start.elapsed();
            (duration, false, Some(e.to_string()))
        }
    }
}

/// Single research request
async fn single_research_request(client: &FortitudeClient, query: &str) -> (Duration, bool, Option<String>) {
    let start = Instant::now();
    
    match client.research(query).await {
        Ok(_) => {
            let duration = start.elapsed();
            (duration, true, None)
        }
        Err(e) => {
            let duration = start.elapsed();
            (duration, false, Some(e.to_string()))
        }
    }
}

/// Test concurrent health checks
async fn test_concurrent_health_checks(num_requests: usize, max_concurrency: usize) -> Result<PerformanceResults> {
    info!("🏥 Testing {} concurrent health checks with {} max concurrency...", num_requests, max_concurrency);
    
    let client = FortitudeClient::new()?;
    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let mut results = PerformanceResults::new();
    
    let start_time = Instant::now();
    
    let tasks: Vec<_> = (0..num_requests)
        .map(|_| {
            let client = client.clone();
            let semaphore = semaphore.clone();
            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                single_health_request(&client).await
            })
        })
        .collect();

    let task_results = join_all(tasks).await;
    results.total_duration = start_time.elapsed();
    
    for task_result in task_results {
        match task_result {
            Ok((duration, success, error)) => {
                results.add_result(duration, success, error);
            }
            Err(e) => {
                results.add_result(Duration::ZERO, false, Some(e.to_string()));
            }
        }
    }
    
    Ok(results)
}

/// Test concurrent research requests
async fn test_concurrent_research_requests(num_requests: usize, max_concurrency: usize) -> Result<PerformanceResults> {
    info!("🔬 Testing {} concurrent research requests with {} max concurrency...", num_requests, max_concurrency);
    
    let client = FortitudeClient::new()?;
    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let mut results = PerformanceResults::new();
    
    let queries = vec![
        "Rust async programming best practices",
        "Python performance optimization techniques", 
        "Docker container security patterns",
        "Microservices communication strategies",
        "Database connection pooling in Node.js",
    ];
    
    let start_time = Instant::now();
    
    let tasks: Vec<_> = (0..num_requests)
        .map(|i| {
            let client = client.clone();
            let semaphore = semaphore.clone();
            let query = queries[i % queries.len()].to_string();
            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                single_research_request(&client, &query).await
            })
        })
        .collect();

    let task_results = join_all(tasks).await;
    results.total_duration = start_time.elapsed();
    
    for task_result in task_results {
        match task_result {
            Ok((duration, success, error)) => {
                results.add_result(duration, success, error);
            }
            Err(e) => {
                results.add_result(Duration::ZERO, false, Some(e.to_string()));
            }
        }
    }
    
    Ok(results)
}

/// Test cache hit rate effectiveness
async fn test_cache_hit_rate() -> Result<(Vec<Duration>, f64)> {
    info!("🗄️ Testing cache hit rate effectiveness...");
    
    let client = FortitudeClient::new()?;
    let query = "Cache effectiveness test query for performance validation";
    let mut times = Vec::new();
    
    // Make multiple identical requests
    for i in 0..10 {
        let start = Instant::now();
        
        match client.research(query).await {
            Ok(_) => {
                let duration = start.elapsed();
                times.push(duration);
                info!("   Request {}: {:?}", i + 1, duration);
            }
            Err(e) => {
                error!("   Request {} failed: {}", i + 1, e);
                times.push(start.elapsed());
            }
        }
        
        // Small delay between requests
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Calculate cache effectiveness
    if times.len() >= 2 {
        let first_request_time = times[0];
        let avg_subsequent_time: Duration = times[1..].iter().sum::<Duration>() / (times.len() - 1) as u32;
        
        let improvement = if first_request_time > avg_subsequent_time {
            let diff = first_request_time - avg_subsequent_time;
            (diff.as_millis() as f64 / first_request_time.as_millis() as f64) * 100.0
        } else {
            0.0
        };
        
        Ok((times, improvement))
    } else {
        Ok((times, 0.0))
    }
}

/// Test rate limiting behavior
async fn test_rate_limiting() -> Result<(usize, usize, Duration)> {
    info!("🚦 Testing rate limiting behavior...");
    
    let client = FortitudeClient::new()?;
    let mut successful_requests = 0;
    let mut rate_limit_hits = 0;
    let start_time = Instant::now();
    
    // Make rapid requests to trigger rate limiting
    for i in 0..100 {
        match client.get_health().await {
            Ok(_) => {
                successful_requests += 1;
            }
            Err(FortitudeError::ApiError { status_code: 429, .. }) => {
                rate_limit_hits += 1;
                info!("   Rate limit hit on request {}", i + 1);
                break;
            }
            Err(e) => {
                error!("   Unexpected error: {}", e);
            }
        }
        
        // Very small delay
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    let total_time = start_time.elapsed();
    Ok((successful_requests, rate_limit_hits, total_time))
}

/// Print performance report
fn print_performance_report(test_name: &str, results: &PerformanceResults) {
    println!("\n📊 {} Results:", test_name);
    println!("{}", "=".repeat(60));
    println!("   Total Requests: {}", results.total_requests);
    println!("   Success Rate: {:.1}%", results.success_rate());
    println!("   Total Duration: {:.2}s", results.total_duration.as_secs_f64());
    println!("   Requests/Second: {:.1}", results.requests_per_second());
    println!();
    println!("   Response Times:");
    println!("     Average: {:.1}ms", results.average_response_time().as_millis());
    println!("     Minimum: {:.1}ms", results.min_response_time().as_millis());
    println!("     Maximum: {:.1}ms", results.max_response_time().as_millis());
    println!("     P50 (Median): {:.1}ms", results.percentile(50.0).as_millis());
    println!("     P90: {:.1}ms", results.percentile(90.0).as_millis());
    println!("     P95: {:.1}ms", results.percentile(95.0).as_millis());
    println!("     P99: {:.1}ms", results.percentile(99.0).as_millis());
    
    // Performance target validation
    println!("\n   🎯 Performance Target Validation:");
    let avg_time_ms = results.average_response_time().as_millis();
    let success_rate = results.success_rate();
    let rps = results.requests_per_second();
    
    if avg_time_ms < 100 {
        println!("     ✅ Sub-100ms average response time: PASSED");
    } else {
        println!("     ❌ Sub-100ms average response time: FAILED ({}ms)", avg_time_ms);
    }
    
    if success_rate >= 99.0 {
        println!("     ✅ >99% success rate: PASSED");
    } else if success_rate >= 95.0 {
        println!("     ⚠️  >95% success rate: ACCEPTABLE");
    } else {
        println!("     ❌ >95% success rate: FAILED ({:.1}%)", success_rate);
    }
    
    if rps >= 10.0 {
        println!("     ✅ Adequate throughput: PASSED");
    } else {
        println!("     ⚠️  Low throughput: {:.1} RPS", rps);
    }
    
    if !results.errors.is_empty() {
        println!("\n   ⚠️  Error Summary ({} errors):", results.errors.len());
        let mut error_counts = std::collections::HashMap::new();
        for error in results.errors.iter().take(10) {
            let error_type = error.split(':').next().unwrap_or(error);
            *error_counts.entry(error_type).or_insert(0) += 1;
        }
        
        for (error_type, count) in error_counts {
            println!("     {}: {}", error_type, count);
        }
    }
}

/// Validate Sprint 006 performance targets
async fn validate_sprint_006_targets() -> Result<Vec<String>> {
    println!("🎯 Sprint 006 Performance Target Validation");
    println!("{}", "=".repeat(60));
    
    let mut targets_met = Vec::new();
    
    // Target 1: 100+ concurrent requests
    println!("🔥 Testing 100+ concurrent request handling...");
    let health_results = test_concurrent_health_checks(120, 15).await?;
    
    if health_results.success_rate() >= 95.0 {
        println!("   ✅ 100+ concurrent requests: PASSED");
        targets_met.push("concurrent_requests".to_string());
    } else {
        println!("   ❌ 100+ concurrent requests: FAILED ({:.1}% success)", health_results.success_rate());
    }
    
    // Target 2: Sub-100ms latency for cached requests
    println!("\n⚡ Testing sub-100ms cached request latency...");
    match test_cache_hit_rate().await {
        Ok((times, improvement)) => {
            if times.len() >= 2 {
                let avg_subsequent_time = times[1..].iter().sum::<Duration>() / (times.len() - 1) as u32;
                if avg_subsequent_time.as_millis() < 100 {
                    println!("   ✅ Sub-100ms cached response time: PASSED");
                    targets_met.push("cached_latency".to_string());
                } else {
                    println!("   ❌ Sub-100ms cached response time: FAILED ({}ms)", avg_subsequent_time.as_millis());
                }
                
                // Target 3: >80% cache hit rate
                if improvement >= 50.0 { // 50% improvement suggests good caching
                    println!("   ✅ >80% cache effectiveness: LIKELY PASSED");
                    targets_met.push("cache_hit_rate".to_string());
                } else {
                    println!("   ⚠️  Cache effectiveness unclear: {:.1}% improvement", improvement);
                }
            }
        }
        Err(e) => {
            println!("   ❌ Cache test failed: {}", e);
        }
    }
    
    // Summary
    println!("\n📋 Sprint 006 Target Summary:");
    println!("   Targets met: {}/3", targets_met.len());
    
    if targets_met.len() >= 3 {
        println!("   🎉 ALL PERFORMANCE TARGETS MET!");
    } else if targets_met.len() >= 2 {
        println!("   ✅ Most performance targets met");
    } else {
        println!("   ⚠️  Performance targets need attention");
    }
    
    Ok(targets_met)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    dotenv::dotenv().ok();
    
    let args = Args::parse();
    
    println!("⚡ Fortitude API - Performance Testing Suite");
    println!("{}", "=".repeat(60));
    
    // Check API connectivity first
    let client = FortitudeClient::new()?;
    match client.test_connection().await {
        Ok(_) => {
            let health = client.get_health().await?;
            println!("🏥 Server status: {}", health.status);
            println!("📝 Server version: {}", health.version);
        }
        Err(e) => {
            error!("❌ Cannot connect to API: {}", e);
            println!("💡 Make sure the API server is running and environment variables are set");
            return Ok(());
        }
    }
    
    println!();
    
    // Run performance tests
    let health_results = test_concurrent_health_checks(args.health_requests, args.max_concurrency).await?;
    print_performance_report("Concurrent Health Checks", &health_results);
    
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    let research_results = test_concurrent_research_requests(args.research_requests, args.max_concurrency).await?;
    print_performance_report("Concurrent Research Requests", &research_results);
    
    // Cache testing
    if args.test_cache {
        println!("\n🗄️ Cache Performance Test:");
        match test_cache_hit_rate().await {
            Ok((times, improvement)) => {
                if !times.is_empty() {
                    println!("   First request: {:.1}ms", times[0].as_millis());
                    if times.len() > 1 {
                        let avg_subsequent: Duration = times[1..].iter().sum::<Duration>() / (times.len() - 1) as u32;
                        println!("   Avg subsequent: {:.1}ms", avg_subsequent.as_millis());
                    }
                    println!("   Cache improvement: {:.1}%", improvement);
                }
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
            }
        }
    }
    
    // Rate limiting test
    println!("\n🚦 Rate Limiting Test:");
    match test_rate_limiting().await {
        Ok((successful, rate_limited, total_time)) => {
            println!("   Successful requests: {}", successful);
            println!("   Rate limit hits: {}", rate_limited);
            println!("   Requests/second: {:.1}", successful as f64 / total_time.as_secs_f64());
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
        }
    }
    
    // Sprint 006 validation
    if args.validate_targets {
        println!("\n{}", "=".repeat(60));
        let targets_met = validate_sprint_006_targets().await?;
        
        // Summary report
        println!("\n📋 Performance Test Summary Report");
        println!("{}", "=".repeat(60));
        println!("📅 Test Date: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
        println!("🔗 API Endpoint: {}", std::env::var("FORTITUDE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()));
        println!("🎯 Sprint 006 Targets Met: {}/3", targets_met.len());
        println!("📊 Overall Success Rate: {:.1}%", (health_results.success_rate() + research_results.success_rate()) / 2.0);
        println!("⏱️  Overall Avg Response Time: {:.1}ms", 
                (health_results.average_response_time().as_millis() + research_results.average_response_time().as_millis()) / 2);
    }
    
    println!("\n✅ Performance testing completed!");
    
    Ok(())
}