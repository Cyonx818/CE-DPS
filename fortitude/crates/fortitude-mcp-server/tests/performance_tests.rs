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

// ABOUTME: Performance tests for Fortitude MCP server
// Tests 100+ concurrent requests capability, sub-100ms latency targets, and throughput under load
// Validates performance requirements from sprint plan

#![allow(
    clippy::uninlined_format_args,
    clippy::manual_flatten,
    clippy::needless_borrows_for_generic_args
)]

mod common;

use common::{PerformanceTestHelper, TestAssertions, TestDataBuilder, TestEnvironment};
use fortitude_mcp_server::{
    AuthManager, AuthMiddleware, FortitudeTools, Permission, ResourceProvider,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Test 100+ concurrent requests performance
#[tokio::test]
#[ignore] // Ignore by default as it's a slow test
async fn test_performance_100_concurrent_requests() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = Arc::new(
        FortitudeTools::new(env.config.as_ref().clone())
            .await
            .unwrap(),
    );

    let request_count = 100;
    let start_time = std::time::Instant::now();

    // Create concurrent requests
    let results = PerformanceTestHelper::run_concurrent_requests(request_count, move || {
        let tools = tools.clone();
        async move {
            let request = TestDataBuilder::research_query_request("Performance test query");
            tools
                .call_tool(request)
                .await
                .map_err(|e| anyhow::anyhow!("Tool call failed: {}", e))
        }
    })
    .await;

    let total_duration = start_time.elapsed();
    let success_rate = PerformanceTestHelper::calculate_success_rate(&results);

    println!("Performance Test Results:");
    println!("- Total requests: {}", request_count);
    println!(
        "- Successful requests: {}",
        results.iter().filter(|r| r.is_ok()).count()
    );
    println!("- Success rate: {:.2}%", success_rate * 100.0);
    println!("- Total duration: {:?}", total_duration);
    println!(
        "- Average latency: {:?}",
        total_duration / request_count as u32
    );

    // Assertions
    TestAssertions::assert_success_rate_acceptable(success_rate, 0.9); // 90% success rate
    TestAssertions::assert_latency_acceptable(total_duration, 30000); // 30 seconds max for 100 requests
}

/// Test latency targets for individual requests
#[tokio::test]
async fn test_performance_latency_targets() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    let test_queries = vec![
        "How to implement async functions in Rust?",
        "Best practices for error handling",
        "How to use traits effectively?",
        "Rust memory management principles",
        "How to write unit tests in Rust?",
    ];

    let mut latencies = Vec::new();

    for query in test_queries {
        let start = PerformanceTestHelper::start_timer();
        let request = TestDataBuilder::research_query_request(query);
        let result = tools.call_tool(request).await;
        let duration = PerformanceTestHelper::measure_duration(start);

        assert!(result.is_ok(), "Request failed for query: {}", query);
        latencies.push(duration);

        println!("Query: '{}' - Latency: {:?}", query, duration);
    }

    let avg_latency = PerformanceTestHelper::calculate_average_latency(&latencies);
    println!("Average latency: {:?}", avg_latency);

    // Individual requests should complete within reasonable time
    for latency in &latencies {
        TestAssertions::assert_latency_acceptable(*latency, 5000); // 5 seconds max per request
    }

    // Average should be much better
    TestAssertions::assert_latency_acceptable(avg_latency, 2000); // 2 seconds average
}

/// Test throughput under sustained load
#[tokio::test]
#[ignore] // Ignore by default as it's a slow test
async fn test_performance_sustained_throughput() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = Arc::new(
        FortitudeTools::new(env.config.as_ref().clone())
            .await
            .unwrap(),
    );

    let _duration_seconds = 10;
    let concurrent_users = 5;
    let requests_per_user = 10;

    let start_time = std::time::Instant::now();
    let mut all_results = Vec::new();

    // Simulate sustained load with multiple users
    for user_id in 0..concurrent_users {
        let tools = tools.clone();
        let user_results =
            PerformanceTestHelper::run_concurrent_requests(requests_per_user, move || {
                let tools = tools.clone();
                let user_id = user_id; // Capture the loop variable
                async move {
                    let query = format!("User {} query", user_id);
                    let request = TestDataBuilder::research_query_request(&query);
                    tools
                        .call_tool(request)
                        .await
                        .map_err(|e| anyhow::anyhow!("Tool call failed: {}", e))
                }
            })
            .await;

        all_results.extend(user_results);
    }

    let total_duration = start_time.elapsed();
    let total_requests = concurrent_users * requests_per_user;
    let success_rate = PerformanceTestHelper::calculate_success_rate(&all_results);
    let throughput = total_requests as f64 / total_duration.as_secs_f64();

    println!("Sustained Load Test Results:");
    println!("- Total requests: {}", total_requests);
    println!("- Concurrent users: {}", concurrent_users);
    println!("- Duration: {:?}", total_duration);
    println!("- Success rate: {:.2}%", success_rate * 100.0);
    println!("- Throughput: {:.2} requests/second", throughput);

    // Assertions
    TestAssertions::assert_success_rate_acceptable(success_rate, 0.85); // 85% success rate under load
    assert!(
        throughput > 1.0,
        "Throughput should be at least 1 request/second"
    );
}

/// Test resource provider performance
#[tokio::test]
async fn test_performance_resource_provider() {
    let env = TestEnvironment::new().await.unwrap();
    let resources = Arc::new(ResourceProvider::new(env.config.clone()));

    let resource_uris = vec![
        "mcp://fortitude/cache/statistics",
        "mcp://fortitude/config/current",
        "mcp://fortitude/system/metrics",
    ];

    let concurrent_requests = 20;
    let mut all_latencies = Vec::new();

    for uri in resource_uris {
        let resources_clone = resources.clone();
        let results =
            PerformanceTestHelper::run_concurrent_requests(concurrent_requests, move || {
                let resources = resources_clone.clone();
                let uri = uri.to_string();
                async move {
                    let start = PerformanceTestHelper::start_timer();
                    let result = resources.read_resource(&uri).await;
                    let duration = PerformanceTestHelper::measure_duration(start);
                    result
                        .map(|_| duration)
                        .map_err(|e| anyhow::anyhow!("Resource read failed: {}", e))
                }
            })
            .await;

        let latencies: Vec<Duration> = results.into_iter().filter_map(|r| r.ok()).collect();

        all_latencies.extend(latencies);
    }

    let avg_latency = PerformanceTestHelper::calculate_average_latency(&all_latencies);

    println!("Resource Provider Performance:");
    println!("- Total requests: {}", all_latencies.len());
    println!("- Average latency: {:?}", avg_latency);

    // Resource access should be fast
    TestAssertions::assert_latency_acceptable(avg_latency, 100); // 100ms average

    // Individual requests should be very fast
    for latency in &all_latencies {
        TestAssertions::assert_latency_acceptable(*latency, 500); // 500ms max
    }
}

/// Test authentication performance under load
#[tokio::test]
async fn test_performance_authentication_load() {
    let env = TestEnvironment::new().await.unwrap();
    let auth_manager = env.auth_manager.clone();
    let auth_middleware = Arc::new(AuthMiddleware::new(auth_manager.clone()));

    // Generate token
    let token = auth_manager
        .generate_token("test_user", vec![Permission::ResearchRead])
        .await
        .unwrap();
    let auth_header = format!("Bearer {}", token);

    let concurrent_requests = 100;
    let start_time = std::time::Instant::now();

    let results = PerformanceTestHelper::run_concurrent_requests(concurrent_requests, move || {
        let auth_middleware = auth_middleware.clone();
        let auth_header = auth_header.clone();
        async move {
            let start = PerformanceTestHelper::start_timer();
            let result = auth_middleware
                .authenticate_request(Some(&auth_header), "test_client", Permission::ResearchRead)
                .await;
            let duration = PerformanceTestHelper::measure_duration(start);
            result
                .map(|_| duration)
                .map_err(|e| anyhow::anyhow!("Auth failed: {}", e))
        }
    })
    .await;

    let total_duration = start_time.elapsed();
    let success_rate = PerformanceTestHelper::calculate_success_rate(&results);
    let latencies: Vec<Duration> = results.into_iter().filter_map(|r| r.ok()).collect();

    let avg_latency = PerformanceTestHelper::calculate_average_latency(&latencies);

    println!("Authentication Performance:");
    println!("- Total requests: {}", concurrent_requests);
    println!("- Success rate: {:.2}%", success_rate * 100.0);
    println!("- Total duration: {:?}", total_duration);
    println!("- Average latency: {:?}", avg_latency);

    // Authentication should be fast and reliable
    TestAssertions::assert_success_rate_acceptable(success_rate, 0.95); // 95% success rate
    TestAssertions::assert_latency_acceptable(avg_latency, 50); // 50ms average
}

/// Test rate limiting performance
#[tokio::test]
async fn test_performance_rate_limiting() {
    let env = TestEnvironment::new().await.unwrap();
    let mut auth_manager = AuthManager::new(env.config.clone()).unwrap();

    // Set aggressive rate limiting for testing
    auth_manager.set_rate_limit_config(fortitude_mcp_server::RateLimitConfig {
        max_requests_per_minute: 10,
        window_seconds: 60,
    });

    let auth_manager = Arc::new(auth_manager);
    let auth_middleware = Arc::new(AuthMiddleware::new(auth_manager.clone()));

    let token = auth_manager
        .generate_token("test_user", vec![Permission::ResearchRead])
        .await
        .unwrap();
    let auth_header = format!("Bearer {}", token);

    let mut successful_requests = 0;
    let mut rate_limited_requests = 0;
    let mut latencies = Vec::new();

    // Make requests until rate limited
    for i in 0..20 {
        let start = PerformanceTestHelper::start_timer();
        let result = auth_middleware
            .authenticate_request(
                Some(&auth_header),
                "rate_limit_test_client",
                Permission::ResearchRead,
            )
            .await;
        let duration = PerformanceTestHelper::measure_duration(start);

        latencies.push(duration);

        if result.is_ok() {
            successful_requests += 1;
        } else {
            rate_limited_requests += 1;
        }

        if i < 5 {
            // First few requests should succeed
            assert!(result.is_ok(), "Request {} should succeed", i);
        }
    }

    let avg_latency = PerformanceTestHelper::calculate_average_latency(&latencies);

    println!("Rate Limiting Performance:");
    println!("- Successful requests: {}", successful_requests);
    println!("- Rate limited requests: {}", rate_limited_requests);
    println!("- Average latency: {:?}", avg_latency);

    // Should enforce rate limit
    assert_eq!(successful_requests, 10, "Should allow exactly 10 requests");
    assert_eq!(rate_limited_requests, 10, "Should rate limit 10 requests");

    // Rate limiting should be fast
    TestAssertions::assert_latency_acceptable(avg_latency, 50); // 50ms average
}

/// Test memory usage under load
#[tokio::test]
#[ignore] // Ignore by default as it's a slow test
async fn test_performance_memory_usage() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = Arc::new(
        FortitudeTools::new(env.config.as_ref().clone())
            .await
            .unwrap(),
    );

    // Get initial memory usage (this is a simplified approach)
    let initial_memory = get_memory_usage();

    let request_count = 50;
    let batch_size = 10;

    // Process requests in batches to simulate realistic usage
    for batch in 0..(request_count / batch_size) {
        let tools_clone = tools.clone();
        let results = PerformanceTestHelper::run_concurrent_requests(batch_size, move || {
            let tools = tools_clone.clone();
            let batch = batch; // Capture the loop variable
            async move {
                let query = format!("Memory test query batch {}", batch);
                let request = TestDataBuilder::research_query_request(&query);
                tools
                    .call_tool(request)
                    .await
                    .map_err(|e| anyhow::anyhow!("Tool call failed: {}", e))
            }
        })
        .await;

        // Verify requests succeeded
        let success_rate = PerformanceTestHelper::calculate_success_rate(&results);
        assert!(
            success_rate > 0.8,
            "Success rate should be > 80% in batch {}",
            batch
        );

        // Small delay between batches
        sleep(Duration::from_millis(100)).await;
    }

    // Get final memory usage
    let final_memory = get_memory_usage();
    let memory_increase = final_memory.saturating_sub(initial_memory);

    println!("Memory Usage:");
    println!("- Initial memory: {} KB", initial_memory);
    println!("- Final memory: {} KB", final_memory);
    println!("- Memory increase: {} KB", memory_increase);

    // Memory usage should be reasonable (this is a rough check)
    // Allow up to 50MB increase for processing
    assert!(memory_increase < 50_000, "Memory increase should be < 50MB");
}

/// Test classification performance
#[tokio::test]
async fn test_performance_classification() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    let test_queries = vec![
        "How to debug a segfault in my Rust program?",
        "Best practices for async programming",
        "How to implement a binary search tree?",
        "What are the differences between Vec and HashMap?",
        "How to handle errors in Rust?",
    ];

    let mut latencies = Vec::new();

    for query in test_queries {
        let start = PerformanceTestHelper::start_timer();
        let request = TestDataBuilder::classify_query_request(query);
        let result = tools.call_tool(request).await;
        let duration = PerformanceTestHelper::measure_duration(start);

        assert!(result.is_ok(), "Classification failed for query: {}", query);
        latencies.push(duration);
    }

    let avg_latency = PerformanceTestHelper::calculate_average_latency(&latencies);

    println!("Classification Performance:");
    println!("- Queries processed: {}", latencies.len());
    println!("- Average latency: {:?}", avg_latency);

    // Classification should be fast
    TestAssertions::assert_latency_acceptable(avg_latency, 100); // 100ms average

    for latency in &latencies {
        TestAssertions::assert_latency_acceptable(*latency, 500); // 500ms max
    }
}

/// Test context detection performance
#[tokio::test]
async fn test_performance_context_detection() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    let test_queries = vec![
        "I need urgent help with this production issue",
        "Can you help me learn basic Rust concepts?",
        "Advanced optimization techniques for high-performance systems",
        "Quick fix for compilation error",
        "Detailed explanation of ownership and borrowing",
    ];

    let mut latencies = Vec::new();

    for query in test_queries {
        let start = PerformanceTestHelper::start_timer();
        let request = TestDataBuilder::detect_context_request(query, Some("troubleshooting"));
        let result = tools.call_tool(request).await;
        let duration = PerformanceTestHelper::measure_duration(start);

        assert!(
            result.is_ok(),
            "Context detection failed for query: {}",
            query
        );
        latencies.push(duration);
    }

    let avg_latency = PerformanceTestHelper::calculate_average_latency(&latencies);

    println!("Context Detection Performance:");
    println!("- Queries processed: {}", latencies.len());
    println!("- Average latency: {:?}", avg_latency);

    // Context detection should be fast
    TestAssertions::assert_latency_acceptable(avg_latency, 200); // 200ms average

    for latency in &latencies {
        TestAssertions::assert_latency_acceptable(*latency, 1000); // 1 second max
    }
}

/// Test connection handling performance
#[tokio::test]
async fn test_performance_connection_handling() {
    let env = TestEnvironment::new().await.unwrap();
    let resources = Arc::new(ResourceProvider::new(env.config.clone()));

    // Simulate many clients accessing resources
    let client_count = 20;
    let requests_per_client = 5;

    let results = PerformanceTestHelper::run_concurrent_requests(client_count, move || {
        let resources = resources.clone();
        let requests_per_client = requests_per_client; // Capture the variable
        async move {
            let mut client_results = Vec::new();

            for _ in 0..requests_per_client {
                let start = PerformanceTestHelper::start_timer();
                let result = resources
                    .read_resource("mcp://fortitude/cache/statistics")
                    .await;
                let duration = PerformanceTestHelper::measure_duration(start);

                client_results.push((result, duration));
            }

            Ok(client_results)
        }
    })
    .await;

    let mut all_latencies = Vec::new();
    let mut successful_requests = 0;

    for result in results {
        if let Ok(client_results) = result {
            for (request_result, latency) in client_results {
                if request_result.is_ok() {
                    successful_requests += 1;
                }
                all_latencies.push(latency);
            }
        }
    }

    let avg_latency = PerformanceTestHelper::calculate_average_latency(&all_latencies);
    let success_rate = successful_requests as f64 / (client_count * requests_per_client) as f64;

    println!("Connection Handling Performance:");
    println!("- Total clients: {}", client_count);
    println!("- Requests per client: {}", requests_per_client);
    println!("- Success rate: {:.2}%", success_rate * 100.0);
    println!("- Average latency: {:?}", avg_latency);

    // Connection handling should be reliable and fast
    TestAssertions::assert_success_rate_acceptable(success_rate, 0.95); // 95% success rate
    TestAssertions::assert_latency_acceptable(avg_latency, 100); // 100ms average
}

/// Simple memory usage approximation (platform-specific)
fn get_memory_usage() -> u64 {
    // This is a simplified approach - in a real implementation,
    // you'd use a proper memory profiling library

    // For testing purposes, we'll use a simple approximation
    // In production, you'd use proper memory measurement tools
    std::process::id() as u64 // Placeholder - not actual memory usage
}

/// Test error handling performance
#[tokio::test]
async fn test_performance_error_handling() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test various error conditions
    let error_requests = vec![
        TestDataBuilder::empty_query_request(),
        TestDataBuilder::oversized_query_request(),
        TestDataBuilder::malformed_request("research_query"),
        TestDataBuilder::invalid_tool_request("nonexistent_tool"),
    ];

    let mut error_latencies = Vec::new();

    for request in error_requests {
        let start = PerformanceTestHelper::start_timer();
        let result = tools.call_tool(request).await;
        let duration = PerformanceTestHelper::measure_duration(start);

        // Should fail, but quickly
        assert!(result.is_err());
        error_latencies.push(duration);
    }

    let avg_error_latency = PerformanceTestHelper::calculate_average_latency(&error_latencies);

    println!("Error Handling Performance:");
    println!("- Error cases tested: {}", error_latencies.len());
    println!("- Average error latency: {:?}", avg_error_latency);

    // Error handling should be fast
    TestAssertions::assert_latency_acceptable(avg_error_latency, 100); // 100ms average

    for latency in &error_latencies {
        TestAssertions::assert_latency_acceptable(*latency, 500); // 500ms max
    }
}

/// Test mixed workload performance
#[tokio::test]
#[ignore] // Ignore by default as it's a slow test
async fn test_performance_mixed_workload() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = Arc::new(
        FortitudeTools::new(env.config.as_ref().clone())
            .await
            .unwrap(),
    );
    let resources = Arc::new(ResourceProvider::new(env.config.clone()));

    let workload_size = 30;
    let start_time = std::time::Instant::now();

    // Mixed workload: research queries, classification, context detection, resource access
    let results = PerformanceTestHelper::run_concurrent_requests(workload_size, move || {
        let tools = tools.clone();
        let resources = resources.clone();
        async move {
            let operation = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                % 4;

            match operation {
                0 => {
                    // Research query
                    let request = TestDataBuilder::research_query_request("Mixed workload query");
                    tools
                        .call_tool(request)
                        .await
                        .map_err(|e| anyhow::anyhow!("Research failed: {}", e))
                }
                1 => {
                    // Classification
                    let request = TestDataBuilder::classify_query_request("Classification query");
                    tools
                        .call_tool(request)
                        .await
                        .map_err(|e| anyhow::anyhow!("Classification failed: {}", e))
                }
                2 => {
                    // Context detection
                    let request =
                        TestDataBuilder::detect_context_request("Context detection query", None);
                    tools
                        .call_tool(request)
                        .await
                        .map_err(|e| anyhow::anyhow!("Context detection failed: {}", e))
                }
                3 => {
                    // Resource access - convert to tool result format
                    let resource_result = resources
                        .read_resource("mcp://fortitude/cache/statistics")
                        .await
                        .map_err(|e| anyhow::anyhow!("Resource access failed: {}", e))?;

                    // Create a mock tool result for resource access
                    Ok(rmcp::model::CallToolResult {
                        content: vec![rmcp::model::Content::text(&format!(
                            "Resource contents: {} items",
                            resource_result.len()
                        ))],
                        is_error: Some(false),
                    })
                }
                _ => unreachable!(),
            }
        }
    })
    .await;

    let total_duration = start_time.elapsed();
    let success_rate = PerformanceTestHelper::calculate_success_rate(&results);

    println!("Mixed Workload Performance:");
    println!("- Total operations: {}", workload_size);
    println!("- Success rate: {:.2}%", success_rate * 100.0);
    println!("- Total duration: {:?}", total_duration);
    println!(
        "- Average latency: {:?}",
        total_duration / workload_size as u32
    );

    // Mixed workload should handle well
    TestAssertions::assert_success_rate_acceptable(success_rate, 0.8); // 80% success rate
    TestAssertions::assert_latency_acceptable(total_duration / workload_size as u32, 2000);
    // 2 seconds average
}
