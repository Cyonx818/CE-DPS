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

// ABOUTME: Comprehensive error handling and edge case tests for Fortitude MCP server
// Tests error conditions, edge cases, and system resilience
// Validates proper error handling throughout the system

#![allow(clippy::uninlined_format_args, clippy::field_reassign_with_default)]

mod common;

use common::{SecurityTestHelper, TestDataBuilder, TestEnvironment};
use fortitude_mcp_server::{
    AuthManager, AuthMiddleware, FortitudeTools, Permission, ResourceProvider, ServerConfig,
};
use serde_json::json;
use std::sync::Arc;

/// Test error handling in tool execution
#[tokio::test]
async fn test_tool_error_handling() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test 1: Invalid tool name
    let invalid_tool_request = TestDataBuilder::invalid_tool_request("nonexistent_tool");
    let result = tools.call_tool(invalid_tool_request).await;
    assert!(result.is_err());

    // Test 2: Missing arguments
    let missing_args_request = rmcp::model::CallToolRequestParam {
        name: "research_query".to_string().into(),
        arguments: None,
    };
    let result = tools.call_tool(missing_args_request).await;
    assert!(result.is_err());

    // Test 3: Invalid argument types
    let invalid_args_request = rmcp::model::CallToolRequestParam {
        name: "research_query".to_string().into(),
        arguments: Some(
            json!({
                "query": 123, // Wrong type
                "invalid_param": "value"
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
    };
    let result = tools.call_tool(invalid_args_request).await;
    assert!(result.is_err());

    // Test 4: Empty required parameters
    let empty_query_request = TestDataBuilder::empty_query_request();
    let result = tools.call_tool(empty_query_request).await;
    assert!(result.is_err());

    // Test 5: Oversized parameters
    let oversized_request = TestDataBuilder::oversized_query_request();
    let result = tools.call_tool(oversized_request).await;
    assert!(result.is_err());

    // Test 6: Malformed JSON in arguments
    let malformed_request = rmcp::model::CallToolRequestParam {
        name: "research_query".to_string().into(),
        arguments: Some(
            json!({
                "query": "test",
                "malformed": "\u{0000}invalid\u{0001}chars"
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
    };
    let _result = tools.call_tool(malformed_request).await;
    // Should either succeed with sanitized input or fail with validation error

    // Test 7: Verify system remains functional after errors
    let valid_request = TestDataBuilder::research_query_request("Recovery test after errors");
    let result = tools.call_tool(valid_request).await;
    assert!(result.is_ok(), "System should recover after various errors");
}

/// Test error propagation through the system
#[tokio::test]
async fn test_error_propagation() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test error propagation from different layers
    let error_cases = vec![
        ("Empty query", TestDataBuilder::empty_query_request()),
        (
            "Invalid tool",
            TestDataBuilder::invalid_tool_request("invalid"),
        ),
        (
            "Malformed request",
            TestDataBuilder::malformed_request("research_query"),
        ),
        (
            "Oversized query",
            TestDataBuilder::oversized_query_request(),
        ),
    ];

    for (case_name, request) in error_cases {
        let result = tools.call_tool(request).await;
        assert!(result.is_err(), "Case '{}' should produce error", case_name);

        // Verify error contains useful information
        let error_msg = result.unwrap_err().to_string();
        assert!(
            !error_msg.is_empty(),
            "Error message should not be empty for case '{}'",
            case_name
        );
        assert!(
            !error_msg.contains("panic"),
            "Error should not contain panic traces for case '{}'",
            case_name
        );
    }
}

/// Test resource provider error handling
#[tokio::test]
async fn test_resource_provider_error_handling() {
    let env = TestEnvironment::new().await.unwrap();
    let resources = ResourceProvider::new(env.config.clone());

    // Test 1: Invalid resource URIs
    let invalid_uris = vec![
        "invalid://scheme/resource",
        "mcp://wrong-domain/resource",
        "mcp://fortitude/invalid/resource",
        "mcp://fortitude/docs/../../../etc/passwd",
        "mcp://fortitude/",
        "",
        "not-a-uri",
    ];

    for uri in invalid_uris {
        let result = resources.read_resource(uri).await;
        assert!(result.is_err(), "Should reject invalid URI: {}", uri);
    }

    // Test 2: Path traversal attempts
    let traversal_attempts = vec![
        "mcp://fortitude/docs/../../../etc/passwd",
        "mcp://fortitude/docs/reference_library/../../secret",
        "mcp://fortitude/docs/~/private",
        "mcp://fortitude/docs/reference_library/../../../config",
    ];

    for uri in traversal_attempts {
        let result = resources.read_resource(uri).await;
        assert!(result.is_err(), "Should reject path traversal: {}", uri);
    }

    // Test 3: Nonexistent resources
    let nonexistent_uris = vec![
        "mcp://fortitude/cache/nonexistent",
        "mcp://fortitude/config/invalid",
        "mcp://fortitude/system/unknown",
        "mcp://fortitude/docs/reference_library/missing.md",
    ];

    for uri in nonexistent_uris {
        let result = resources.read_resource(uri).await;
        assert!(
            result.is_err(),
            "Should handle nonexistent resource: {}",
            uri
        );
    }

    // Test 4: Verify system remains functional after errors
    let valid_uri = "mcp://fortitude/cache/statistics";
    let result = resources.read_resource(valid_uri).await;
    assert!(
        result.is_ok(),
        "System should recover after resource errors"
    );
}

/// Test authentication error handling
#[tokio::test]
async fn test_authentication_error_handling() {
    let env = TestEnvironment::new().await.unwrap();
    let auth_manager = env.auth_manager.clone();
    let auth_middleware = AuthMiddleware::new(auth_manager.clone());

    // Test 1: Invalid tokens
    let invalid_tokens = vec![
        "invalid_token",
        "Bearer invalid_token",
        "Bearer ",
        "",
        "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.signature",
        "Bearer not.a.jwt.token",
        "Bearer a.b.c.d.e", // Too many parts
    ];

    for token in invalid_tokens {
        let result = auth_middleware
            .authenticate_request(Some(token), "test_client", Permission::ResearchRead)
            .await;
        assert!(result.is_err(), "Should reject invalid token: {}", token);
    }

    // Test 2: Missing authorization header
    let result = auth_middleware
        .authenticate_request(None, "test_client", Permission::ResearchRead)
        .await;
    assert!(result.is_err(), "Should require authorization header");

    // Test 3: Permission denied
    let limited_token = auth_manager
        .generate_token("limited_user", vec![Permission::ResearchRead])
        .await
        .unwrap();
    let auth_header = format!("Bearer {}", limited_token);

    let result = auth_middleware
        .authenticate_request(Some(&auth_header), "test_client", Permission::Admin)
        .await;
    assert!(result.is_err(), "Should deny insufficient permissions");

    // Test 4: Rate limiting
    let mut rate_limited_auth = AuthManager::new(env.config.clone()).unwrap();
    rate_limited_auth.set_rate_limit_config(fortitude_mcp_server::RateLimitConfig {
        max_requests_per_minute: 1,
        window_seconds: 60,
    });

    let rate_limited_auth = Arc::new(rate_limited_auth);
    let rate_limited_middleware = AuthMiddleware::new(rate_limited_auth.clone());

    let token = rate_limited_auth
        .generate_token("rate_test", vec![Permission::ResearchRead])
        .await
        .unwrap();
    let auth_header = format!("Bearer {}", token);

    // First request should succeed
    let result = rate_limited_middleware
        .authenticate_request(
            Some(&auth_header),
            "rate_test_client",
            Permission::ResearchRead,
        )
        .await;
    assert!(result.is_ok());

    // Second request should fail
    let result = rate_limited_middleware
        .authenticate_request(
            Some(&auth_header),
            "rate_test_client",
            Permission::ResearchRead,
        )
        .await;
    assert!(result.is_err(), "Should be rate limited");
}

/// Test configuration error handling
#[tokio::test]
async fn test_configuration_error_handling() {
    // Test 1: Invalid configuration values
    let mut invalid_config = ServerConfig::default();
    invalid_config.port = 0; // Invalid port

    let result = invalid_config.validate();
    assert!(result.is_err(), "Should reject invalid port");

    // Test 2: Invalid JWT secret
    let mut invalid_config = ServerConfig::default();
    invalid_config.auth.jwt_secret = "short".to_string(); // Too short
    invalid_config.auth.enabled = true;

    let result = invalid_config.validate();
    assert!(result.is_err(), "Should reject short JWT secret");

    // Test 3: Invalid rate limit configuration
    let mut invalid_config = ServerConfig::default();
    invalid_config.auth.rate_limit.max_requests_per_minute = 0;

    let result = invalid_config.validate();
    assert!(result.is_err(), "Should reject invalid rate limit");

    // Test 4: Environment variable parsing errors
    std::env::set_var("MCP_SERVER_PORT", "invalid_port");
    let result = ServerConfig::from_env();
    assert!(
        result.is_err(),
        "Should reject invalid environment variable"
    );

    std::env::remove_var("MCP_SERVER_PORT");
}

/// Test concurrent error handling
#[tokio::test]
async fn test_concurrent_error_handling() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = Arc::new(
        FortitudeTools::new(env.config.as_ref().clone())
            .await
            .unwrap(),
    );

    // Create multiple concurrent invalid requests
    let mut error_handles = Vec::new();

    for i in 0..20 {
        let tools = tools.clone();
        let handle = tokio::spawn(async move {
            let request = if i % 4 == 0 {
                TestDataBuilder::invalid_tool_request("invalid")
            } else if i % 4 == 1 {
                TestDataBuilder::empty_query_request()
            } else if i % 4 == 2 {
                TestDataBuilder::malformed_request("research_query")
            } else {
                TestDataBuilder::oversized_query_request()
            };

            tools.call_tool(request).await
        });
        error_handles.push(handle);
    }

    let results = futures::future::join_all(error_handles).await;

    // All should fail gracefully
    for result in results {
        let tool_result = result.unwrap();
        assert!(
            tool_result.is_err(),
            "Concurrent error requests should fail gracefully"
        );
    }

    // System should still work after concurrent errors
    let recovery_request =
        TestDataBuilder::research_query_request("Post-concurrent-error recovery");
    let recovery_result = tools.call_tool(recovery_request).await;
    assert!(
        recovery_result.is_ok(),
        "System should recover after concurrent errors"
    );
}

/// Test error message security
#[tokio::test]
async fn test_error_message_security() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();
    let resources = ResourceProvider::new(env.config.clone());

    // Test 1: Tool errors should not expose sensitive information
    let error_requests = vec![
        TestDataBuilder::invalid_tool_request("nonexistent_tool"),
        TestDataBuilder::malformed_request("research_query"),
        TestDataBuilder::empty_query_request(),
    ];

    for request in error_requests {
        let result = tools.call_tool(request).await;
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(
            !error_msg.contains("password"),
            "Error should not contain passwords"
        );
        assert!(
            !error_msg.contains("secret"),
            "Error should not contain secrets"
        );
        assert!(
            !error_msg.contains("token"),
            "Error should not contain tokens"
        );
        assert!(
            !error_msg.to_lowercase().contains("internal"),
            "Error should not contain internal details"
        );
    }

    // Test 2: Resource errors should not expose file paths
    let invalid_uris = vec![
        "mcp://fortitude/docs/../../../etc/passwd",
        "mcp://fortitude/nonexistent/resource",
    ];

    for uri in invalid_uris {
        let result = resources.read_resource(uri).await;
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(
            !error_msg.contains("/etc/passwd"),
            "Error should not expose system paths"
        );
        assert!(
            !error_msg.contains("../"),
            "Error should not expose path traversal attempts"
        );
    }
}

/// Test error recovery scenarios
#[tokio::test]
async fn test_error_recovery_scenarios() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test 1: Recovery after validation errors
    let validation_error_request = TestDataBuilder::empty_query_request();
    let result = tools.call_tool(validation_error_request).await;
    assert!(result.is_err());

    // Should recover immediately
    let recovery_request = TestDataBuilder::research_query_request("Recovery test 1");
    let result = tools.call_tool(recovery_request).await;
    assert!(result.is_ok());

    // Test 2: Recovery after multiple errors
    let error_sequence = vec![
        TestDataBuilder::invalid_tool_request("invalid1"),
        TestDataBuilder::empty_query_request(),
        TestDataBuilder::malformed_request("research_query"),
        TestDataBuilder::oversized_query_request(),
    ];

    for request in error_sequence {
        let result = tools.call_tool(request).await;
        assert!(result.is_err());
    }

    // Should still recover
    let recovery_request = TestDataBuilder::research_query_request("Recovery test 2");
    let result = tools.call_tool(recovery_request).await;
    assert!(result.is_ok());

    // Test 3: Recovery after mixed success/failure
    let mixed_requests = vec![
        TestDataBuilder::research_query_request("Valid request 1"),
        TestDataBuilder::invalid_tool_request("invalid"),
        TestDataBuilder::classify_query_request("Valid classification"),
        TestDataBuilder::empty_query_request(),
        TestDataBuilder::detect_context_request("Valid context", None),
    ];

    for request in mixed_requests {
        let _ = tools.call_tool(request).await;
        // Don't check result - some should succeed, some should fail
    }

    // Should still work
    let final_request = TestDataBuilder::research_query_request("Final recovery test");
    let result = tools.call_tool(final_request).await;
    assert!(result.is_ok());
}

/// Test edge cases in input processing
#[tokio::test]
async fn test_input_processing_edge_cases() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test 1: Boundary length inputs
    let boundary_inputs = vec![
        "a".to_string(),  // Minimum length
        "a".repeat(999),  // Just under limit
        "a".repeat(1000), // At limit
    ];

    for input in boundary_inputs {
        let request = TestDataBuilder::research_query_request(&input);
        let result = tools.call_tool(request).await;
        assert!(
            result.is_ok(),
            "Should handle boundary input length: {}",
            input.len()
        );
    }

    // Test 2: Unicode edge cases
    let unicode_inputs = vec![
        "🦀 Rust programming",
        "测试 Chinese characters",
        "🚀💻⚡ Multiple emojis",
        "Àáâãäåæçèéêë accented chars",
        "𝒮𝒸𝓇𝒾𝓅𝓉 mathematical symbols",
    ];

    for input in unicode_inputs {
        let request = TestDataBuilder::research_query_request(input);
        let result = tools.call_tool(request).await;
        assert!(result.is_ok(), "Should handle Unicode input: {}", input);
    }

    // Test 3: Special character edge cases
    let special_inputs = vec![
        "Query with\nnewlines\nand\ttabs",
        "Query with \"quotes\" and 'apostrophes'",
        "Query with special chars: !@#$%^&*()",
        "Query with backslashes: \\n\\t\\r",
        "Query with JSON chars: {\"key\": \"value\"}",
    ];

    for input in special_inputs {
        let request = TestDataBuilder::research_query_request(input);
        let result = tools.call_tool(request).await;
        assert!(
            result.is_ok(),
            "Should handle special characters: {}",
            input
        );
    }

    // Test 4: Control character handling
    let control_char_inputs = vec![
        format!("Query with{}control{}chars", "\x00", "\x01"),
        format!("Query{}with{}more{}control", "\x02", "\x03", "\x04"),
    ];

    for input in control_char_inputs {
        let request = TestDataBuilder::research_query_request(&input);
        let _result = tools.call_tool(request).await;
        // Should either succeed with sanitized input or fail with validation error
        // Both are acceptable outcomes
    }
}

/// Test system stability under error conditions
#[tokio::test]
async fn test_system_stability_under_errors() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = Arc::new(
        FortitudeTools::new(env.config.as_ref().clone())
            .await
            .unwrap(),
    );

    // Test 1: Rapid error generation
    let rapid_error_count = 100;
    let mut handles = Vec::new();

    for i in 0..rapid_error_count {
        let tools = tools.clone();
        let handle = tokio::spawn(async move {
            let request = match i % 3 {
                0 => TestDataBuilder::invalid_tool_request("invalid"),
                1 => TestDataBuilder::empty_query_request(),
                _ => TestDataBuilder::malformed_request("research_query"),
            };
            tools.call_tool(request).await
        });
        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;

    // All should fail gracefully
    for result in results {
        let tool_result = result.unwrap();
        assert!(tool_result.is_err());
    }

    // System should still be responsive
    let health_check = TestDataBuilder::research_query_request("Health check after rapid errors");
    let result = tools.call_tool(health_check).await;
    assert!(result.is_ok());

    // Test 2: Mixed load with errors
    let mixed_load_count = 50;
    let mut mixed_handles = Vec::new();

    for i in 0..mixed_load_count {
        let tools = tools.clone();
        let handle = tokio::spawn(async move {
            let request = if i % 2 == 0 {
                TestDataBuilder::research_query_request(&format!("Valid query {}", i))
            } else {
                TestDataBuilder::invalid_tool_request("invalid")
            };
            tools.call_tool(request).await
        });
        mixed_handles.push(handle);
    }

    let mixed_results = futures::future::join_all(mixed_handles).await;

    // Should have mix of success and failures
    let successful_count = mixed_results
        .iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
        .count();

    let failed_count = mixed_results
        .iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().is_err())
        .count();

    assert!(successful_count > 0, "Should have some successful requests");
    assert!(failed_count > 0, "Should have some failed requests");
    assert_eq!(
        successful_count + failed_count,
        mixed_load_count,
        "All requests should complete"
    );
}

/// Test timeout and resource exhaustion handling
#[tokio::test]
async fn test_timeout_and_resource_handling() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = Arc::new(
        FortitudeTools::new(env.config.as_ref().clone())
            .await
            .unwrap(),
    );

    // Test 1: Many concurrent requests (stress test)
    let concurrent_count = 30;
    let mut handles = Vec::new();

    for i in 0..concurrent_count {
        let tools = tools.clone();
        let handle = tokio::spawn(async move {
            let query = format!("Concurrent stress test query {}", i);
            let request = TestDataBuilder::research_query_request(&query);

            let start = std::time::Instant::now();
            let result = tools.call_tool(request).await;
            let duration = start.elapsed();

            (result, duration)
        });
        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;

    // Most should succeed, but some might fail due to resource limits
    let successful_count = results
        .iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().0.is_ok())
        .count();

    let success_rate = successful_count as f64 / concurrent_count as f64;

    // Should handle at least 70% successfully under stress
    assert!(
        success_rate >= 0.7,
        "Success rate under stress should be >= 70%, got {:.2}%",
        success_rate * 100.0
    );

    // Check that response times are reasonable
    let response_times: Vec<std::time::Duration> = results
        .iter()
        .filter_map(|r| r.as_ref().ok())
        .filter(|(result, _)| result.is_ok())
        .map(|(_, duration)| *duration)
        .collect();

    if !response_times.is_empty() {
        let avg_response_time =
            response_times.iter().sum::<std::time::Duration>() / response_times.len() as u32;
        assert!(
            avg_response_time.as_secs() < 10,
            "Average response time should be < 10 seconds"
        );
    }

    // Test 2: System recovery after stress
    let recovery_request = TestDataBuilder::research_query_request("Recovery after stress test");
    let result = tools.call_tool(recovery_request).await;
    assert!(result.is_ok(), "System should recover after stress");
}

/// Test error handling with malicious inputs
#[tokio::test]
async fn test_malicious_input_handling() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test 1: Injection attempts
    let injection_attempts = SecurityTestHelper::generate_malicious_inputs();

    for malicious_input in injection_attempts {
        let request = TestDataBuilder::research_query_request(&malicious_input);
        let result = tools.call_tool(request).await;

        // Should either sanitize and process, or reject with validation error
        if let Ok(response) = result {
            // If processed, verify it's safe
            assert_eq!(response.is_error, Some(false));

            // Check that response doesn't contain unsanitized malicious content
            if let Some(content) = response.content[0].as_text() {
                assert!(
                    !content.text.contains("<script>"),
                    "Response should not contain script tags"
                );
                assert!(
                    !content.text.contains("DROP TABLE"),
                    "Response should not contain SQL"
                );
            }
        }
        // If rejected, that's also acceptable
    }

    // Test 2: Oversized input attempts
    let oversized_inputs = SecurityTestHelper::generate_oversized_inputs();

    for oversized_input in oversized_inputs {
        let request = TestDataBuilder::research_query_request(&oversized_input);
        let result = tools.call_tool(request).await;
        assert!(
            result.is_err(),
            "Should reject oversized input: {} chars",
            oversized_input.len()
        );
    }

    // Test 3: Unicode attack attempts
    let unicode_attacks = SecurityTestHelper::generate_unicode_attacks();

    for unicode_attack in unicode_attacks {
        let request = TestDataBuilder::research_query_request(&unicode_attack);
        let _result = tools.call_tool(request).await;
        // Should handle safely (either process or reject)
    }

    // Test 4: Verify system remains functional after attacks
    let clean_request =
        TestDataBuilder::research_query_request("Clean request after malicious inputs");
    let result = tools.call_tool(clean_request).await;
    assert!(
        result.is_ok(),
        "System should remain functional after malicious inputs"
    );
}

/// Test error handling in different components
#[tokio::test]
async fn test_component_error_isolation() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();
    let resources = ResourceProvider::new(env.config.clone());

    // Test 1: Tool errors don't affect resource provider
    let _ = tools
        .call_tool(TestDataBuilder::invalid_tool_request("invalid"))
        .await;
    let resource_result = resources
        .read_resource("mcp://fortitude/cache/statistics")
        .await;
    assert!(
        resource_result.is_ok(),
        "Resource provider should be unaffected by tool errors"
    );

    // Test 2: Resource errors don't affect tools
    let _ = resources
        .read_resource("mcp://fortitude/invalid/resource")
        .await;
    let tool_result = tools
        .call_tool(TestDataBuilder::research_query_request("Test query"))
        .await;
    assert!(
        tool_result.is_ok(),
        "Tools should be unaffected by resource errors"
    );

    // Test 3: Multiple component failures
    let _ = tools
        .call_tool(TestDataBuilder::invalid_tool_request("invalid"))
        .await;
    let _ = resources
        .read_resource("mcp://fortitude/invalid/resource")
        .await;
    let _ = tools
        .call_tool(TestDataBuilder::empty_query_request())
        .await;
    let _ = resources
        .read_resource("mcp://fortitude/docs/../../../etc/passwd")
        .await;

    // Both should still work
    let tool_result = tools
        .call_tool(TestDataBuilder::research_query_request(
            "Component isolation test",
        ))
        .await;
    assert!(
        tool_result.is_ok(),
        "Tools should work after multiple component failures"
    );

    let resource_result = resources
        .read_resource("mcp://fortitude/config/current")
        .await;
    assert!(
        resource_result.is_ok(),
        "Resources should work after multiple component failures"
    );
}

/// Test error handling with extreme inputs
#[tokio::test]
async fn test_extreme_input_handling() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test 1: Very long but valid inputs
    let long_but_valid = "a".repeat(999); // Just under limit
    let request = TestDataBuilder::research_query_request(&long_but_valid);
    let result = tools.call_tool(request).await;
    assert!(result.is_ok(), "Should handle long but valid input");

    // Test 2: Inputs at exact limits
    let at_limit = "a".repeat(1000); // At limit
    let request = TestDataBuilder::research_query_request(&at_limit);
    let result = tools.call_tool(request).await;
    assert!(result.is_ok(), "Should handle input at exact limit");

    // Test 3: Inputs just over limits
    let over_limit = "a".repeat(1001); // Over limit
    let request = TestDataBuilder::research_query_request(&over_limit);
    let result = tools.call_tool(request).await;
    assert!(result.is_err(), "Should reject input over limit");

    // Test 4: Complex nested structures
    let complex_args = json!({
        "query": "test",
        "nested": {
            "deep": {
                "structure": {
                    "with": ["arrays", "and", "objects"]
                }
            }
        }
    });

    let complex_request = rmcp::model::CallToolRequestParam {
        name: "research_query".to_string().into(),
        arguments: Some(complex_args.as_object().unwrap().clone()),
    };

    let _result = tools.call_tool(complex_request).await;
    // Should handle gracefully (either process or reject with appropriate error)
}

/// Test graceful degradation
#[tokio::test]
async fn test_graceful_degradation() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test 1: Verify system continues to work with partial functionality
    // Even if some features fail, basic functionality should remain

    // Try various operations
    let operations = vec![
        TestDataBuilder::research_query_request("Degradation test 1"),
        TestDataBuilder::classify_query_request("Degradation test 2"),
        TestDataBuilder::detect_context_request("Degradation test 3", None),
    ];

    let mut successful_operations = 0;

    for operation in operations {
        let result = tools.call_tool(operation).await;
        if result.is_ok() {
            successful_operations += 1;
        }
    }

    // At least some operations should succeed
    assert!(
        successful_operations > 0,
        "At least some operations should succeed during degradation"
    );

    // Test 2: Verify system provides meaningful feedback during degradation
    let test_request = TestDataBuilder::research_query_request("Degradation feedback test");
    let result = tools.call_tool(test_request).await;

    if result.is_ok() {
        // If successful, response should be meaningful
        let response = result.unwrap();
        assert_eq!(response.is_error, Some(false));
        assert!(!response.content.is_empty());
    }
    // If failed, that's acceptable during degradation
}

/// Test error handling consistency
#[tokio::test]
async fn test_error_handling_consistency() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test that similar errors produce consistent error types
    let empty_query_requests = vec![
        TestDataBuilder::empty_query_request(),
        rmcp::model::CallToolRequestParam {
            name: "classify_query".to_string().into(),
            arguments: Some(json!({"query": ""}).as_object().unwrap().clone()),
        },
        rmcp::model::CallToolRequestParam {
            name: "detect_context".to_string().into(),
            arguments: Some(json!({"query": ""}).as_object().unwrap().clone()),
        },
    ];

    for request in empty_query_requests {
        let result = tools.call_tool(request).await;
        assert!(result.is_err(), "Empty query should be rejected");

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("validation") || error_msg.contains("Invalid"),
            "Error should be validation-related: {}",
            error_msg
        );
    }

    // Test that invalid tool names produce consistent errors
    let invalid_tools = vec!["invalid1", "invalid2", "nonexistent"];

    for tool_name in invalid_tools {
        let request = TestDataBuilder::invalid_tool_request(tool_name);
        let result = tools.call_tool(request).await;
        assert!(result.is_err(), "Invalid tool should be rejected");

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Unknown tool"),
            "Error should mention unknown tool: {}",
            error_msg
        );
    }
}
