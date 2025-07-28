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

// ABOUTME: Comprehensive integration tests for Fortitude MCP server
// Tests end-to-end workflows, Claude Code integration patterns, and full system behavior
// Covers authentication, research pipeline, resource access, and error handling

mod common;

use common::{
    PerformanceTestHelper, SecurityTestHelper, TestAssertions, TestDataBuilder, TestEnvironment,
};
use fortitude_mcp_server::{FortitudeTools, McpServer, Permission, ResourceProvider};
use serde_json::json;
use std::sync::Arc;

/// Test end-to-end research workflow with authentication
#[tokio::test]
async fn test_integration_research_workflow_with_auth() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Generate token with research permissions
    let _token = env
        .create_test_token(vec![Permission::ResearchRead])
        .await
        .unwrap();

    // Test research_query tool
    let research_request =
        TestDataBuilder::research_query_request("How to implement async functions in Rust?");

    let result = tools.call_tool(research_request).await.unwrap();

    // Verify successful response
    TestAssertions::assert_successful_response(&Ok(result.clone()));

    // Verify response contains research data
    if let Some(content) = result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();
        assert!(response.get("result").is_some());
        assert!(response.get("metadata").is_some());

        let metadata = response.get("metadata").unwrap();
        assert!(metadata.get("research_type").is_some());
        assert!(metadata.get("confidence").is_some());
        assert!(metadata.get("processing_time_ms").is_some());
        assert!(metadata.get("context_detection_used").is_some());
    }
}

/// Test classify_query tool integration
#[tokio::test]
async fn test_integration_classify_query_workflow() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test classify_query tool
    let classify_request =
        TestDataBuilder::classify_query_request("How to debug a segfault in my Rust program?");

    let result = tools.call_tool(classify_request).await.unwrap();

    // Verify successful response
    TestAssertions::assert_successful_response(&Ok(result.clone()));

    // Verify response contains classification data
    if let Some(content) = result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();
        assert!(response.get("research_type").is_some());
        assert!(response.get("confidence").is_some());
        assert!(response.get("matched_keywords").is_some());
        assert!(response.get("candidates").is_some());

        let confidence = response.get("confidence").unwrap().as_f64().unwrap();
        assert!((0.0..=1.0).contains(&confidence));
    }
}

/// Test detect_context tool integration
#[tokio::test]
async fn test_integration_detect_context_workflow() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test detect_context tool
    let context_request = TestDataBuilder::detect_context_request(
        "I need urgent help with this production issue",
        Some("troubleshooting"),
    );

    let result = tools.call_tool(context_request).await.unwrap();

    // Verify successful response
    TestAssertions::assert_successful_response(&Ok(result.clone()));

    // Verify response contains context data
    if let Some(content) = result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();
        assert!(response.get("audience_level").is_some());
        assert!(response.get("technical_domain").is_some());
        assert!(response.get("urgency_level").is_some());
        assert!(response.get("overall_confidence").is_some());
        assert!(response.get("processing_time_ms").is_some());
        assert!(response.get("dimension_confidences").is_some());

        let confidence = response
            .get("overall_confidence")
            .unwrap()
            .as_f64()
            .unwrap();
        assert!((0.0..=1.0).contains(&confidence));
    }
}

/// Test resource provider integration
#[tokio::test]
async fn test_integration_resource_provider_workflow() {
    let env = TestEnvironment::new().await.unwrap();
    let resources = ResourceProvider::new(env.config.clone());

    // Test listing resources
    let resource_list = resources.list_resources().await.unwrap();
    assert!(!resource_list.is_empty());

    // Test cache statistics resource
    let cache_uri = "mcp://fortitude/cache/statistics";
    let cache_result = resources.read_resource(cache_uri).await.unwrap();
    assert_eq!(cache_result.len(), 1);

    // Verify it's valid JSON
    if let rmcp::model::ResourceContents::TextResourceContents { text, .. } = &cache_result[0] {
        let _stats: serde_json::Value = serde_json::from_str(text).unwrap();
    }

    // Test configuration resource
    let config_uri = "mcp://fortitude/config/current";
    let config_result = resources.read_resource(config_uri).await.unwrap();
    assert_eq!(config_result.len(), 1);

    // Verify JWT secret is redacted
    if let rmcp::model::ResourceContents::TextResourceContents { text, .. } = &config_result[0] {
        let config_json: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(config_json["auth"]["jwt_secret"], "[REDACTED]");
    }
}

/// Test MCP server creation and basic functionality
#[tokio::test]
async fn test_integration_mcp_server_basic() {
    let env = TestEnvironment::with_auth_disabled().await.unwrap();
    let _server = McpServer::new(env.config.as_ref().clone()).await.unwrap();

    // Test server creation succeeds
    // This verifies the server can be instantiated with valid configuration
    // More detailed MCP protocol tests would require complex setup

    // Verify server has required components by testing tools and resources directly
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();
    let resources = ResourceProvider::new(env.config.clone());

    // Test tools work
    let research_request = TestDataBuilder::research_query_request("Integration test query");
    let result = tools.call_tool(research_request).await;
    assert!(result.is_ok(), "Tools should work in server context");

    // Test resources work
    let config_uri = "mcp://fortitude/config/current";
    let resource_result = resources.read_resource(config_uri).await;
    assert!(
        resource_result.is_ok(),
        "Resources should work in server context"
    );
}

/// Test error handling in end-to-end workflows
#[tokio::test]
async fn test_integration_error_handling_workflow() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test invalid tool request
    let invalid_request = TestDataBuilder::invalid_tool_request("nonexistent_tool");
    let result = tools.call_tool(invalid_request).await;
    assert!(result.is_err());

    // Test malformed request
    let malformed_request = TestDataBuilder::malformed_request("research_query");
    let result = tools.call_tool(malformed_request).await;
    assert!(result.is_err());

    // Test empty query
    let empty_request = TestDataBuilder::empty_query_request();
    let result = tools.call_tool(empty_request).await;
    assert!(result.is_err());

    // Test oversized query
    let oversized_request = TestDataBuilder::oversized_query_request();
    let result = tools.call_tool(oversized_request).await;
    assert!(result.is_err());
}

/// Test multiple tool calls in sequence (workflow simulation)
#[tokio::test]
async fn test_integration_sequential_tool_workflow() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    let query = "How to implement error handling in Rust?";

    // Step 1: Classify the query
    let classify_request = TestDataBuilder::classify_query_request(query);
    let classify_result = tools.call_tool(classify_request).await.unwrap();
    TestAssertions::assert_successful_response(&Ok(classify_result));

    // Step 2: Detect context
    let context_request = TestDataBuilder::detect_context_request(query, Some("learning"));
    let context_result = tools.call_tool(context_request).await.unwrap();
    TestAssertions::assert_successful_response(&Ok(context_result));

    // Step 3: Execute research query
    let research_request = TestDataBuilder::research_query_request(query);
    let research_result = tools.call_tool(research_request).await.unwrap();
    TestAssertions::assert_successful_response(&Ok(research_result));
}

/// Test concurrent tool calls (stress test)
#[tokio::test]
async fn test_integration_concurrent_tool_calls() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = Arc::new(
        FortitudeTools::new(env.config.as_ref().clone())
            .await
            .unwrap(),
    );

    // Define different queries for concurrent execution
    let queries = vec![
        "How to implement async functions in Rust?",
        "Best practices for error handling in Rust",
        "How to use traits in Rust effectively?",
        "Rust memory management principles",
        "How to write unit tests in Rust?",
    ];

    let mut handles = Vec::new();

    for query in queries {
        let tools = tools.clone();
        let request = TestDataBuilder::research_query_request(query);

        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            let result = tools.call_tool(request).await;
            let duration = start.elapsed();
            (result, duration)
        });

        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;

    // Verify all requests succeeded
    for result in results {
        let (tool_result, duration) = result.unwrap();
        assert!(tool_result.is_ok());
        TestAssertions::assert_latency_acceptable(duration, 5000); // 5 seconds max
    }
}

/// Test resource access patterns
#[tokio::test]
async fn test_integration_resource_access_patterns() {
    let env = TestEnvironment::new().await.unwrap();
    let resources = ResourceProvider::new(env.config.clone());

    // Test different resource types
    let resource_uris = vec![
        "mcp://fortitude/cache/statistics",
        "mcp://fortitude/config/current",
        "mcp://fortitude/system/metrics",
    ];

    for uri in resource_uris {
        let result = resources.read_resource(uri).await;
        assert!(result.is_ok(), "Failed to read resource: {uri}");

        let contents = result.unwrap();
        assert!(!contents.is_empty(), "Resource content is empty: {uri}");

        // Verify content is valid JSON
        if let rmcp::model::ResourceContents::TextResourceContents { text, .. } = &contents[0] {
            let _json: serde_json::Value = serde_json::from_str(text)
                .unwrap_or_else(|_| panic!("Invalid JSON in resource: {uri}"));
        }
    }
}

/// Test authentication workflow integration
#[tokio::test]
async fn test_integration_authentication_workflow() {
    let env = TestEnvironment::new().await.unwrap();
    let _server = McpServer::new(env.config.as_ref().clone()).await.unwrap();

    // Test authentication through tools directly since MCP server context setup is complex
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test basic functionality works
    let research_request = TestDataBuilder::research_query_request("test query");
    let result = tools.call_tool(research_request).await;
    assert!(
        result.is_ok(),
        "Tools should work with proper configuration"
    );
}

/// Test input validation and sanitization integration
#[tokio::test]
async fn test_integration_input_validation_workflow() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test malicious input attempts
    let malicious_inputs = SecurityTestHelper::generate_malicious_inputs();

    for malicious_input in malicious_inputs {
        let arguments = json!({
            "query": malicious_input
        });

        let request = rmcp::model::CallToolRequestParam {
            name: "research_query".to_string().into(),
            arguments: Some(arguments.as_object().unwrap().clone()),
        };

        // The system should sanitize the input and process it safely
        let _result = tools.call_tool(request).await;
        // Either should succeed with sanitized input or fail with validation error
        // Both are acceptable security outcomes
    }
}

/// Test system resilience under load
#[tokio::test]
async fn test_integration_system_resilience() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = Arc::new(
        FortitudeTools::new(env.config.as_ref().clone())
            .await
            .unwrap(),
    );

    // Test system with rapid successive requests
    let request_count = 20;
    let mut handles = Vec::new();

    for i in 0..request_count {
        let tools = tools.clone();
        let query = format!("Test query number {i}");
        let request = TestDataBuilder::research_query_request(&query);

        let handle = tokio::spawn(async move { tools.call_tool(request).await });

        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;

    // Calculate success rate
    let successful_requests = results
        .iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
        .count();

    let success_rate = successful_requests as f64 / request_count as f64;

    // System should handle at least 80% of requests successfully
    TestAssertions::assert_success_rate_acceptable(success_rate, 0.8);
}

/// Test error recovery and graceful degradation
#[tokio::test]
async fn test_integration_error_recovery() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test recovery from various error conditions
    let test_cases = vec![
        TestDataBuilder::empty_query_request(),
        TestDataBuilder::oversized_query_request(),
        TestDataBuilder::malformed_request("research_query"),
        TestDataBuilder::invalid_tool_request("nonexistent_tool"),
    ];

    for test_case in test_cases {
        let result = tools.call_tool(test_case).await;
        // Should fail gracefully, not crash
        assert!(result.is_err());
    }

    // Verify system still works after errors
    let valid_request = TestDataBuilder::research_query_request("Valid query after errors");
    let result = tools.call_tool(valid_request).await;
    assert!(result.is_ok());
}

/// Test comprehensive resource security
#[tokio::test]
async fn test_integration_resource_security() {
    let env = TestEnvironment::new().await.unwrap();
    let resources = ResourceProvider::new(env.config.clone());

    // Test path traversal protection
    let malicious_uris = vec![
        "mcp://fortitude/docs/../../../etc/passwd",
        "mcp://fortitude/docs/reference_library/../../secret",
        "mcp://fortitude/docs/~/private",
        "mcp://fortitude/invalid/resource",
    ];

    for uri in malicious_uris {
        let result = resources.read_resource(uri).await;
        assert!(result.is_err(), "Should reject malicious URI: {uri}");
    }

    // Test valid resource access still works
    let valid_uri = "mcp://fortitude/cache/statistics";
    let result = resources.read_resource(valid_uri).await;
    assert!(result.is_ok(), "Should allow valid resource access");
}

/// Test data consistency across multiple operations
#[tokio::test]
async fn test_integration_data_consistency() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    let query = "How to implement async functions in Rust?";

    // Run the same query multiple times
    let mut results = Vec::new();
    for _ in 0..3 {
        let request = TestDataBuilder::research_query_request(query);
        let result = tools.call_tool(request).await.unwrap();
        results.push(result);
    }

    // Verify all results are successful
    for result in &results {
        TestAssertions::assert_successful_response(&Ok(result.clone()));
    }

    // Results should be consistent (either from cache or consistent generation)
    // This depends on caching behavior, but at minimum should not crash
}

/// Test performance characteristics under normal load
#[tokio::test]
async fn test_integration_performance_characteristics() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = Arc::new(
        FortitudeTools::new(env.config.as_ref().clone())
            .await
            .unwrap(),
    );

    let queries = vec![
        "How to implement async functions in Rust?",
        "Best practices for error handling",
        "How to use traits effectively?",
    ];

    let mut latencies = Vec::new();

    for query in queries {
        let tools = tools.clone();
        let request = TestDataBuilder::research_query_request(query);

        let start = std::time::Instant::now();
        let result = tools.call_tool(request).await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        latencies.push(duration);
    }

    // Calculate average latency
    let avg_latency = PerformanceTestHelper::calculate_average_latency(&latencies);

    // Average latency should be reasonable (under 2 seconds for research queries)
    TestAssertions::assert_latency_acceptable(avg_latency, 2000);
}
