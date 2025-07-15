// ABOUTME: Anchor tests for Fortitude MCP server critical functionality
// These tests protect core system functionality from regression and must be maintained
// Created following decision matrix from tests/README.md with ANCHOR: docstring comments

mod common;

use common::{
    PerformanceTestHelper, SecurityTestHelper, TestAssertions, TestDataBuilder, TestEnvironment,
};
use fortitude_mcp_server::{
    AuthManager, AuthMiddleware, FortitudeTools, McpServer, Permission, ResourceProvider,
};
use rmcp::ServerHandler;
use std::sync::Arc;

/// ANCHOR: Verifies MCP protocol compliance works end-to-end
/// Tests: Protocol handshake, capabilities negotiation, tool/resource listing, request/response format
/// Protects: External API integration with MCP protocol
#[tokio::test]
async fn test_anchor_mcp_protocol_compliance() {
    let env = TestEnvironment::with_auth_disabled().await.unwrap();
    let server = McpServer::new(env.config.as_ref().clone()).await.unwrap();

    // Test 1: Server info follows MCP specification
    let server_info = server.get_info();
    assert!(
        server_info.capabilities.tools.is_some(),
        "Server must declare tool capability"
    );
    assert!(
        server_info.capabilities.resources.is_some(),
        "Server must declare resource capability"
    );

    // Test 2: Server info provides proper MCP information
    let server_info = server.get_info();
    assert!(
        server_info.capabilities.tools.is_some(),
        "Server must declare tool capability"
    );
    assert!(
        server_info.capabilities.resources.is_some(),
        "Server must declare resource capability"
    );

    // Test 3: Tool listing through FortitudeTools (simulating MCP protocol)
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();
    let tools_result = tools.list_tools();
    assert!(!tools_result.tools.is_empty());

    // Verify required tools are present
    let tool_names: Vec<&str> = tools_result.tools.iter().map(|t| t.name.as_ref()).collect();
    assert!(
        tool_names.contains(&"research_query"),
        "research_query tool must be available"
    );
    assert!(
        tool_names.contains(&"classify_query"),
        "classify_query tool must be available"
    );
    assert!(
        tool_names.contains(&"detect_context"),
        "detect_context tool must be available"
    );

    // Verify tool schemas are properly formatted
    for tool in &tools_result.tools {
        assert!(!tool.name.is_empty(), "Tool name must not be empty");
        assert!(
            tool.description.is_some(),
            "Tool description must be provided"
        );
        assert!(
            !tool.input_schema.is_empty(),
            "Tool input schema must be provided"
        );
    }

    // Test 4: Resource listing through ResourceProvider (simulating MCP protocol)
    let resources = ResourceProvider::new(env.config.clone());
    let resource_list = resources.list_resources().await.unwrap();
    let resources_result = rmcp::model::ListResourcesResult {
        resources: resource_list,
        next_cursor: None,
    };
    assert!(!resources_result.resources.is_empty());

    // Verify resources have proper URIs
    for resource in &resources_result.resources {
        assert!(
            resource.raw.uri.starts_with("mcp://fortitude/"),
            "Resource URI must use fortitude scheme"
        );
        assert!(
            !resource.raw.name.is_empty(),
            "Resource name must not be empty"
        );
    }

    // Test 5: Tool call follows MCP request/response format
    let research_request = TestDataBuilder::research_query_request("Protocol compliance test");
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();
    let tool_result = tools.call_tool(research_request).await.unwrap();
    assert_eq!(tool_result.is_error, Some(false));
    assert!(!tool_result.content.is_empty());

    // Verify response content format
    if let Some(content) = tool_result.content[0].as_text() {
        // Must be valid JSON
        let _parsed: serde_json::Value =
            serde_json::from_str(&content.text).expect("Tool response must be valid JSON");
    }

    // Test 6: Resource read follows MCP format
    let resources = ResourceProvider::new(env.config.clone());
    let resource_contents = resources
        .read_resource("mcp://fortitude/cache/statistics")
        .await
        .unwrap();
    let resource_result = rmcp::model::ReadResourceResult {
        contents: resource_contents,
    };
    assert!(!resource_result.contents.is_empty());

    // Verify resource content format
    if let rmcp::model::ResourceContents::TextResourceContents { text, uri, .. } =
        &resource_result.contents[0]
    {
        assert!(!text.is_empty(), "Resource content must not be empty");
        assert!(
            uri.starts_with("mcp://fortitude/"),
            "Resource URI must use fortitude scheme"
        );
    }
}

/// ANCHOR: Verifies research pipeline integration works end-to-end
/// Tests: Complete research workflow, classification accuracy, context detection, caching
/// Protects: Business logic functions and core domain operations
#[tokio::test]
async fn test_anchor_research_pipeline_integration() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test 1: Research query processes successfully
    let research_request =
        TestDataBuilder::research_query_request("How to implement async functions in Rust?");

    let result = tools.call_tool(research_request).await.unwrap();
    assert_eq!(result.is_error, Some(false));
    assert!(!result.content.is_empty());

    // Parse and verify research response structure
    if let Some(content) = result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();

        // Must have required fields
        assert!(
            response.get("result").is_some(),
            "Research result must be present"
        );
        assert!(
            response.get("metadata").is_some(),
            "Research metadata must be present"
        );

        let metadata = response.get("metadata").unwrap();
        assert!(
            metadata.get("research_type").is_some(),
            "Research type must be classified"
        );
        assert!(
            metadata.get("confidence").is_some(),
            "Confidence score must be present"
        );
        assert!(
            metadata.get("processing_time_ms").is_some(),
            "Processing time must be tracked"
        );
        assert!(
            metadata.get("context_detection_used").is_some(),
            "Context detection status must be recorded"
        );
        assert!(
            metadata.get("cache_key").is_some(),
            "Cache key must be provided"
        );

        // Verify confidence is in valid range
        let confidence = metadata.get("confidence").unwrap().as_f64().unwrap();
        assert!(
            (0.0..=1.0).contains(&confidence),
            "Confidence must be between 0 and 1"
        );

        // Verify processing time is reasonable
        let processing_time = metadata
            .get("processing_time_ms")
            .unwrap()
            .as_u64()
            .unwrap();
        assert!(processing_time > 0, "Processing time must be positive");
    }

    // Test 2: Classification accuracy and consistency
    let classification_request =
        TestDataBuilder::classify_query_request("How to debug a segfault in my Rust program?");

    let classify_result = tools.call_tool(classification_request).await.unwrap();
    assert_eq!(classify_result.is_error, Some(false));

    if let Some(content) = classify_result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();

        // Must have classification results
        assert!(
            response.get("research_type").is_some(),
            "Research type must be classified"
        );
        assert!(
            response.get("confidence").is_some(),
            "Classification confidence required"
        );
        assert!(
            response.get("matched_keywords").is_some(),
            "Matched keywords must be provided"
        );
        assert!(
            response.get("candidates").is_some(),
            "Alternative candidates must be provided"
        );

        // Verify classification quality
        let confidence = response.get("confidence").unwrap().as_f64().unwrap();
        assert!(
            (0.0..=1.0).contains(&confidence),
            "Classification confidence must be valid"
        );

        let matched_keywords = response
            .get("matched_keywords")
            .unwrap()
            .as_array()
            .unwrap();
        assert!(
            !matched_keywords.is_empty(),
            "Classification should match keywords"
        );

        let candidates = response.get("candidates").unwrap().as_array().unwrap();
        assert!(
            !candidates.is_empty(),
            "Classification should provide candidates"
        );
    }

    // Test 3: Context detection accuracy
    let context_request = TestDataBuilder::detect_context_request(
        "I need urgent help with this production issue",
        Some("troubleshooting"),
    );

    let context_result = tools.call_tool(context_request).await.unwrap();
    assert_eq!(context_result.is_error, Some(false));

    if let Some(content) = context_result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();

        // Must have context detection results
        assert!(
            response.get("audience_level").is_some(),
            "Audience level must be detected"
        );
        assert!(
            response.get("technical_domain").is_some(),
            "Technical domain must be detected"
        );
        assert!(
            response.get("urgency_level").is_some(),
            "Urgency level must be detected"
        );
        assert!(
            response.get("overall_confidence").is_some(),
            "Overall confidence required"
        );
        assert!(
            response.get("dimension_confidences").is_some(),
            "Dimension confidences required"
        );

        // Verify context detection quality
        let overall_confidence = response
            .get("overall_confidence")
            .unwrap()
            .as_f64()
            .unwrap();
        assert!(
            (0.0..=1.0).contains(&overall_confidence),
            "Overall confidence must be valid"
        );

        let dimension_confidences = response
            .get("dimension_confidences")
            .unwrap()
            .as_array()
            .unwrap();
        assert!(
            !dimension_confidences.is_empty(),
            "Dimension confidences must be provided"
        );
    }

    // Test 4: Pipeline integration consistency
    // Run the same query through different tools to verify consistency
    let test_query = "How to implement error handling in Rust?";

    let research_request = TestDataBuilder::research_query_request(test_query);
    let classify_request = TestDataBuilder::classify_query_request(test_query);
    let context_request = TestDataBuilder::detect_context_request(test_query, None);

    let research_result = tools.call_tool(research_request).await.unwrap();
    let classify_result = tools.call_tool(classify_request).await.unwrap();
    let context_result = tools.call_tool(context_request).await.unwrap();

    // All should succeed
    assert_eq!(research_result.is_error, Some(false));
    assert_eq!(classify_result.is_error, Some(false));
    assert_eq!(context_result.is_error, Some(false));

    // Pipeline should be consistent across tools
    // (This is a basic consistency check - more sophisticated checks could be added)
}

/// ANCHOR: Verifies authentication security works end-to-end
/// Tests: JWT token validation, permission enforcement, rate limiting, session management
/// Protects: Authentication and authorization security
#[tokio::test]
async fn test_anchor_authentication_security() {
    let env = TestEnvironment::new().await.unwrap();
    let auth_manager = env.auth_manager.clone();
    let auth_middleware = AuthMiddleware::new(auth_manager.clone());

    // Test 1: Token generation and validation
    let permissions = vec![Permission::ResearchRead, Permission::ResourcesRead];
    let token = auth_manager
        .generate_token("test_user", permissions)
        .await
        .unwrap();

    // Token should be valid
    let claims = auth_manager.verify_token(&token).await.unwrap();
    assert_eq!(claims.sub, "test_user");
    assert_eq!(claims.iss, "fortitude-mcp-server");
    assert!(claims
        .permissions
        .contains(&Permission::ResearchRead.as_str().to_string()));
    assert!(claims
        .permissions
        .contains(&Permission::ResourcesRead.as_str().to_string()));

    // Test 2: Permission enforcement
    let auth_header = format!("Bearer {token}");

    // Should succeed with proper permission
    let result = auth_middleware
        .authenticate_request(Some(&auth_header), "test_client", Permission::ResearchRead)
        .await;
    assert!(result.is_ok(), "Should allow access with proper permission");

    // Should fail without permission
    let result = auth_middleware
        .authenticate_request(Some(&auth_header), "test_client", Permission::Admin)
        .await;
    assert!(
        result.is_err(),
        "Should deny access without proper permission"
    );

    // Test 3: Invalid token rejection
    let invalid_tokens = vec![
        "invalid_token",
        "Bearer invalid_token",
        "Bearer ",
        "",
        "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.signature",
    ];

    for invalid_token in invalid_tokens {
        let result = auth_middleware
            .authenticate_request(Some(invalid_token), "test_client", Permission::ResearchRead)
            .await;
        assert!(
            result.is_err(),
            "Should reject invalid token: {invalid_token}"
        );
    }

    // Test 4: Rate limiting enforcement
    let mut auth_manager_with_limits = AuthManager::new(env.config.clone()).unwrap();
    auth_manager_with_limits.set_rate_limit_config(fortitude_mcp_server::RateLimitConfig {
        max_requests_per_minute: 3,
        window_seconds: 60,
    });

    let auth_manager_with_limits = Arc::new(auth_manager_with_limits);
    let auth_middleware_with_limits = AuthMiddleware::new(auth_manager_with_limits.clone());

    let token = auth_manager_with_limits
        .generate_token("rate_limit_test", vec![Permission::ResearchRead])
        .await
        .unwrap();
    let auth_header = format!("Bearer {token}");

    // First 3 requests should succeed
    for i in 0..3 {
        let result = auth_middleware_with_limits
            .authenticate_request(
                Some(&auth_header),
                "rate_limit_client",
                Permission::ResearchRead,
            )
            .await;
        assert!(result.is_ok(), "Request {i} should succeed");
    }

    // Fourth request should fail due to rate limiting
    let result = auth_middleware_with_limits
        .authenticate_request(
            Some(&auth_header),
            "rate_limit_client",
            Permission::ResearchRead,
        )
        .await;
    assert!(result.is_err(), "Should be rate limited");

    // Test 5: Admin privilege escalation
    let admin_token = auth_manager.create_default_admin_token().await.unwrap();
    let admin_header = format!("Bearer {admin_token}");

    // Admin should have all permissions
    let permissions_to_test = vec![
        Permission::ResearchRead,
        Permission::ResourcesRead,
        Permission::ConfigRead,
        Permission::Admin,
    ];

    for permission in permissions_to_test {
        let result = auth_middleware
            .authenticate_request(Some(&admin_header), "admin_client", permission)
            .await;
        assert!(
            result.is_ok(),
            "Admin should have all permissions: {permission:?}"
        );
    }

    // Test 6: Session security
    // Verify token expiration is enforced
    let claims = auth_manager.verify_token(&token).await.unwrap();
    assert!(
        claims.exp > chrono::Utc::now().timestamp(),
        "Token should not be expired"
    );

    // Test 7: Authentication bypass prevention
    // Verify auth is required when enabled
    let result = auth_middleware
        .authenticate_request(None, "test_client", Permission::ResearchRead)
        .await;
    assert!(
        result.is_err(),
        "Should require authentication when enabled"
    );
}

/// ANCHOR: Verifies data persistence operations work end-to-end
/// Tests: Resource file access, configuration persistence, cache operations
/// Protects: Data persistence operations and file system security
#[tokio::test]
async fn test_anchor_data_persistence_operations() {
    let env = TestEnvironment::new().await.unwrap();
    let resources = ResourceProvider::new(env.config.clone());

    // Test 1: Resource file access
    let resource_uris = vec![
        "mcp://fortitude/cache/statistics",
        "mcp://fortitude/config/current",
        "mcp://fortitude/system/metrics",
    ];

    for uri in resource_uris {
        let result = resources.read_resource(uri).await;
        assert!(result.is_ok(), "Should successfully read resource: {uri}");

        let contents = result.unwrap();
        assert!(
            !contents.is_empty(),
            "Resource content should not be empty: {uri}"
        );

        // Verify content structure
        if let rmcp::model::ResourceContents::TextResourceContents {
            text,
            uri: content_uri,
            ..
        } = &contents[0]
        {
            assert!(!text.is_empty(), "Resource text should not be empty");
            assert_eq!(content_uri, uri, "Resource URI should match request");

            // Verify content is valid JSON
            let parsed: serde_json::Value = serde_json::from_str(text)
                .unwrap_or_else(|_| panic!("Resource content should be valid JSON: {uri}"));

            // Verify expected structure based on resource type
            match uri {
                "mcp://fortitude/cache/statistics" => {
                    assert!(
                        parsed.get("total_entries").is_some(),
                        "Cache stats should have total_entries"
                    );
                    assert!(
                        parsed.get("hit_rate").is_some(),
                        "Cache stats should have hit_rate"
                    );
                    assert!(
                        parsed.get("last_updated").is_some(),
                        "Cache stats should have last_updated"
                    );
                }
                "mcp://fortitude/config/current" => {
                    assert!(parsed.get("port").is_some(), "Config should have port");
                    assert!(parsed.get("host").is_some(), "Config should have host");
                    assert!(
                        parsed.get("auth").is_some(),
                        "Config should have auth section"
                    );
                    // Verify security: JWT secret should be redacted
                    assert_eq!(
                        parsed["auth"]["jwt_secret"], "[REDACTED]",
                        "JWT secret must be redacted"
                    );
                }
                "mcp://fortitude/system/metrics" => {
                    assert!(
                        parsed.get("timestamp").is_some(),
                        "System metrics should have timestamp"
                    );
                    assert!(
                        parsed.get("uptime_seconds").is_some(),
                        "System metrics should have uptime"
                    );
                    assert!(
                        parsed.get("memory_usage").is_some(),
                        "System metrics should have memory usage"
                    );
                }
                _ => {}
            }
        }
    }

    // Test 2: File system security
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

    // Test 3: Resource listing consistency
    let resource_list = resources.list_resources().await.unwrap();
    assert!(!resource_list.is_empty(), "Should list available resources");

    // Verify all listed resources are accessible
    for resource in &resource_list {
        let uri = &resource.raw.uri;
        let result = resources.read_resource(uri).await;
        assert!(
            result.is_ok(),
            "Listed resource should be accessible: {uri}"
        );
    }

    // Test 4: Configuration persistence
    let config_uri = "mcp://fortitude/config/current";
    let config_result = resources.read_resource(config_uri).await.unwrap();

    if let rmcp::model::ResourceContents::TextResourceContents { text, .. } = &config_result[0] {
        let config: serde_json::Value = serde_json::from_str(text).unwrap();

        // Verify configuration matches expected values
        assert_eq!(config["port"], env.config.port);
        assert_eq!(config["host"], env.config.host);
        assert_eq!(config["max_connections"], env.config.max_connections);

        // Verify nested configuration
        assert_eq!(config["auth"]["enabled"], env.config.auth.enabled);
        assert_eq!(
            config["auth"]["token_expiration_hours"],
            env.config.auth.token_expiration_hours
        );
        assert_eq!(
            config["performance"]["cache_size"],
            env.config.performance.cache_size
        );
    }

    // Test 5: Cache operations
    let cache_uri = "mcp://fortitude/cache/statistics";
    let cache_result = resources.read_resource(cache_uri).await.unwrap();

    if let rmcp::model::ResourceContents::TextResourceContents { text, .. } = &cache_result[0] {
        let cache_stats: serde_json::Value = serde_json::from_str(text).unwrap();

        // Verify cache statistics structure
        assert!(cache_stats.get("total_entries").is_some());
        assert!(cache_stats.get("hit_rate").is_some());
        assert!(cache_stats.get("miss_rate").is_some());
        assert!(cache_stats.get("total_size_bytes").is_some());
        assert!(cache_stats.get("cache_enabled").is_some());
        assert!(cache_stats.get("max_size_bytes").is_some());
        assert!(cache_stats.get("ttl_seconds").is_some());

        // Verify cache statistics are valid
        let hit_rate = cache_stats.get("hit_rate").unwrap().as_f64().unwrap();
        assert!(
            (0.0..=1.0).contains(&hit_rate),
            "Hit rate must be valid percentage"
        );

        let miss_rate = cache_stats.get("miss_rate").unwrap().as_f64().unwrap();
        assert!(
            (0.0..=1.0).contains(&miss_rate),
            "Miss rate must be valid percentage"
        );
    }

    // Test 6: System metrics persistence
    let metrics_uri = "mcp://fortitude/system/metrics";
    let metrics_result = resources.read_resource(metrics_uri).await.unwrap();

    if let rmcp::model::ResourceContents::TextResourceContents { text, .. } = &metrics_result[0] {
        let metrics: serde_json::Value = serde_json::from_str(text).unwrap();

        // Verify system metrics structure
        assert!(metrics.get("timestamp").is_some());
        assert!(metrics.get("memory_usage").is_some());
        assert!(metrics.get("cpu_usage").is_some());
        assert!(metrics.get("network").is_some());
        assert!(metrics.get("disk").is_some());
        assert!(metrics.get("process").is_some());

        // Verify metrics are realistic
        let memory_usage = metrics.get("memory_usage").unwrap();
        assert!(memory_usage.get("total_bytes").is_some());
        assert!(memory_usage.get("used_bytes").is_some());
        assert!(memory_usage.get("free_bytes").is_some());

        let process_info = metrics.get("process").unwrap();
        assert!(process_info.get("pid").is_some());
        assert!(process_info.get("threads").is_some());
    }
}

/// ANCHOR: Verifies user input processing works end-to-end
/// Tests: Input validation, sanitization, parameter processing, error handling
/// Protects: User input processing and injection prevention
#[tokio::test]
async fn test_anchor_user_input_processing() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test 1: Input validation
    let invalid_inputs = vec![
        "".to_string(),   // Empty query
        "a".repeat(2000), // Oversized query
    ];

    for invalid_input in invalid_inputs {
        let request = TestDataBuilder::research_query_request(&invalid_input);
        let result = tools.call_tool(request).await;
        assert!(
            result.is_err(),
            "Should reject invalid input: {invalid_input}"
        );
    }

    // Test 2: Input sanitization
    let malicious_inputs = SecurityTestHelper::generate_malicious_inputs();

    for malicious_input in malicious_inputs {
        let request = TestDataBuilder::research_query_request(&malicious_input);
        let result = tools.call_tool(request).await;

        // Should either sanitize and process, or reject with validation error
        // Both outcomes are secure
        if let Ok(response) = result {
            assert_eq!(
                response.is_error,
                Some(false),
                "Sanitized input should process successfully"
            );
        }
        // If error, it should be a validation error, not a system crash
    }

    // Test 3: Parameter processing
    let valid_request = TestDataBuilder::research_query_request("Valid query");
    let result = tools.call_tool(valid_request).await.unwrap();

    assert_eq!(result.is_error, Some(false));
    assert!(!result.content.is_empty());

    // Verify response structure
    if let Some(content) = result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();
        assert!(response.get("result").is_some());
        assert!(response.get("metadata").is_some());
    }

    // Test 4: Optional parameter handling
    let minimal_request = rmcp::model::CallToolRequestParam {
        name: "research_query".into(),
        arguments: Some(
            serde_json::json!({
                "query": "Minimal query"
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
    };

    let result = tools.call_tool(minimal_request).await.unwrap();
    assert_eq!(result.is_error, Some(false));

    // Test 5: Type validation
    let type_invalid_request = rmcp::model::CallToolRequestParam {
        name: "research_query".into(),
        arguments: Some(
            serde_json::json!({
                "query": 123, // Wrong type
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
    };

    let result = tools.call_tool(type_invalid_request).await;
    assert!(result.is_err(), "Should reject invalid parameter types");

    // Test 6: Missing required parameters
    let missing_param_request = rmcp::model::CallToolRequestParam {
        name: "research_query".into(),
        arguments: Some(
            serde_json::json!({
                "not_query": "Missing required query parameter"
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
    };

    let result = tools.call_tool(missing_param_request).await;
    assert!(result.is_err(), "Should reject missing required parameters");

    // Test 7: Unicode and special character handling
    let unicode_inputs = vec![
        "How to handle Unicode in Rust? 🦀",
        "Rust: 中文测试",
        "Emoji test: 🚀💻⚡",
        "Special chars: àáâãäåæçèéêë",
    ];

    for unicode_input in unicode_inputs {
        let request = TestDataBuilder::research_query_request(unicode_input);
        let result = tools.call_tool(request).await;
        assert!(
            result.is_ok(),
            "Should handle Unicode input: {unicode_input}"
        );
    }

    // Test 8: Classification input processing
    let classify_request = TestDataBuilder::classify_query_request("Classification test");
    let result = tools.call_tool(classify_request).await.unwrap();

    assert_eq!(result.is_error, Some(false));
    if let Some(content) = result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();
        assert!(response.get("research_type").is_some());
        assert!(response.get("confidence").is_some());
    }

    // Test 9: Context detection input processing
    let context_request = TestDataBuilder::detect_context_request("Context test", Some("learning"));
    let result = tools.call_tool(context_request).await.unwrap();

    assert_eq!(result.is_error, Some(false));
    if let Some(content) = result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();
        assert!(response.get("audience_level").is_some());
        assert!(response.get("technical_domain").is_some());
        assert!(response.get("urgency_level").is_some());
    }

    // Test 10: Input length boundary testing
    let boundary_inputs = vec![
        "a".to_string(),  // Minimum length
        "a".repeat(999),  // Just under limit
        "a".repeat(1000), // At limit
    ];

    for boundary_input in boundary_inputs {
        let request = TestDataBuilder::research_query_request(&boundary_input);
        let result = tools.call_tool(request).await;
        assert!(
            result.is_ok(),
            "Should handle boundary input length: {}",
            boundary_input.len()
        );
    }
}

/// ANCHOR: Verifies error handling for critical paths works end-to-end
/// Tests: Authentication failures, rate limiting, invalid requests, system recovery
/// Protects: Error handling for critical paths and graceful degradation
#[tokio::test]
async fn test_anchor_error_handling_critical_paths() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = Arc::new(
        FortitudeTools::new(env.config.as_ref().clone())
            .await
            .unwrap(),
    );
    let auth_manager = env.auth_manager.clone();
    let auth_middleware = AuthMiddleware::new(auth_manager.clone());

    // Test 1: Authentication failure recovery
    let invalid_tokens = vec![
        "invalid_token",
        "Bearer invalid_token",
        "Bearer ",
        "",
        "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.signature",
    ];

    for invalid_token in invalid_tokens {
        let result = auth_middleware
            .authenticate_request(Some(invalid_token), "test_client", Permission::ResearchRead)
            .await;

        assert!(
            result.is_err(),
            "Should handle authentication failure gracefully"
        );

        // Verify system is still functional after auth failure
        let valid_token = auth_manager
            .generate_token("test_user", vec![Permission::ResearchRead])
            .await
            .unwrap();
        let valid_header = format!("Bearer {valid_token}");

        let recovery_result = auth_middleware
            .authenticate_request(
                Some(&valid_header),
                "recovery_test",
                Permission::ResearchRead,
            )
            .await;
        assert!(
            recovery_result.is_ok(),
            "System should recover after auth failure"
        );
    }

    // Test 2: Rate limiting error handling
    let mut rate_limited_auth = AuthManager::new(env.config.clone()).unwrap();
    rate_limited_auth.set_rate_limit_config(fortitude_mcp_server::RateLimitConfig {
        max_requests_per_minute: 2,
        window_seconds: 60,
    });

    let rate_limited_auth = Arc::new(rate_limited_auth);
    let rate_limited_middleware = AuthMiddleware::new(rate_limited_auth.clone());

    let token = rate_limited_auth
        .generate_token("rate_test", vec![Permission::ResearchRead])
        .await
        .unwrap();
    let auth_header = format!("Bearer {token}");

    // Exhaust rate limit
    for _ in 0..2 {
        let result = rate_limited_middleware
            .authenticate_request(
                Some(&auth_header),
                "rate_limit_client",
                Permission::ResearchRead,
            )
            .await;
        assert!(result.is_ok(), "Should succeed within rate limit");
    }

    // Should hit rate limit
    let result = rate_limited_middleware
        .authenticate_request(
            Some(&auth_header),
            "rate_limit_client",
            Permission::ResearchRead,
        )
        .await;
    assert!(result.is_err(), "Should be rate limited");

    // Different client should still work
    let result = rate_limited_middleware
        .authenticate_request(
            Some(&auth_header),
            "different_client",
            Permission::ResearchRead,
        )
        .await;
    assert!(
        result.is_ok(),
        "Different client should not be rate limited"
    );

    // Test 3: Invalid request handling
    let invalid_requests = vec![
        TestDataBuilder::invalid_tool_request("nonexistent_tool"),
        TestDataBuilder::malformed_request("research_query"),
        TestDataBuilder::empty_query_request(),
        TestDataBuilder::oversized_query_request(),
    ];

    for invalid_request in invalid_requests {
        let result = tools.call_tool(invalid_request).await;
        assert!(result.is_err(), "Should handle invalid request gracefully");

        // Verify system is still functional after invalid request
        let valid_request = TestDataBuilder::research_query_request("Recovery test");
        let recovery_result = tools.call_tool(valid_request).await;
        assert!(
            recovery_result.is_ok(),
            "System should recover after invalid request"
        );
    }

    // Test 4: Resource access error handling
    let resources = ResourceProvider::new(env.config.clone());

    let invalid_resource_uris = vec![
        "mcp://fortitude/nonexistent/resource",
        "mcp://fortitude/docs/../../../etc/passwd",
        "invalid://scheme/resource",
        "mcp://fortitude/",
    ];

    for invalid_uri in invalid_resource_uris {
        let result = resources.read_resource(invalid_uri).await;
        assert!(
            result.is_err(),
            "Should handle invalid resource URI gracefully: {invalid_uri}"
        );

        // Verify system is still functional after invalid resource access
        let valid_result = resources
            .read_resource("mcp://fortitude/cache/statistics")
            .await;
        assert!(
            valid_result.is_ok(),
            "System should recover after invalid resource access"
        );
    }

    // Test 5: Concurrent error handling
    let mut error_handles = Vec::new();

    // Spawn multiple concurrent invalid requests
    for i in 0..10 {
        let tools = tools.clone();
        let handle = tokio::spawn(async move {
            let invalid_request =
                TestDataBuilder::invalid_tool_request(&format!("invalid_tool_{i}"));
            tools.call_tool(invalid_request).await
        });
        error_handles.push(handle);
    }

    // All should fail gracefully
    let error_results = futures::future::join_all(error_handles).await;
    for result in error_results {
        let tool_result = result.unwrap();
        assert!(
            tool_result.is_err(),
            "Concurrent invalid requests should fail gracefully"
        );
    }

    // System should still work after concurrent errors
    let recovery_request = TestDataBuilder::research_query_request("Post-error recovery test");
    let recovery_result = tools.call_tool(recovery_request).await;
    assert!(
        recovery_result.is_ok(),
        "System should recover after concurrent errors"
    );

    // Test 6: System resource exhaustion handling
    // This test simulates resource exhaustion and recovery
    let resource_test_count = 100;
    let mut resource_handles = Vec::new();

    for i in 0..resource_test_count {
        let resources = ResourceProvider::new(env.config.clone());
        let handle = tokio::spawn(async move {
            let uri = if i % 2 == 0 {
                "mcp://fortitude/cache/statistics"
            } else {
                "mcp://fortitude/config/current"
            };
            resources.read_resource(uri).await
        });
        resource_handles.push(handle);
    }

    let resource_results = futures::future::join_all(resource_handles).await;
    let successful_requests = resource_results
        .iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
        .count();

    // Should handle most requests successfully
    let success_rate = successful_requests as f64 / resource_test_count as f64;
    assert!(
        success_rate > 0.8,
        "Should handle resource requests with >80% success rate"
    );

    // Test 7: Error message consistency
    let error_cases = vec![
        (TestDataBuilder::empty_query_request(), "validation"),
        (
            TestDataBuilder::invalid_tool_request("invalid"),
            "Unknown tool",
        ),
        (
            TestDataBuilder::malformed_request("research_query"),
            "Invalid arguments",
        ),
    ];

    for (error_request, expected_error_type) in error_cases {
        let result = tools.call_tool(error_request).await;
        assert!(result.is_err(), "Should produce error");

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains(expected_error_type),
            "Error message should contain '{expected_error_type}', got: {error_msg}"
        );
    }
}

/// ANCHOR: Verifies system performance under load
/// Tests: Concurrent request handling, latency targets, throughput, resource usage
/// Protects: Performance characteristics and scalability
#[tokio::test]
async fn test_anchor_performance_under_load() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = Arc::new(
        FortitudeTools::new(env.config.as_ref().clone())
            .await
            .unwrap(),
    );
    let resources = Arc::new(ResourceProvider::new(env.config.clone()));

    // Test 1: Concurrent request handling
    let concurrent_requests = 20;
    let start_time = std::time::Instant::now();

    let tools_for_concurrent = tools.clone();
    let results = PerformanceTestHelper::run_concurrent_requests(concurrent_requests, move || {
        let tools = tools_for_concurrent.clone();
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

    // Should handle concurrent requests with high success rate
    TestAssertions::assert_success_rate_acceptable(success_rate, 0.9); // 90% success rate

    // Should complete within reasonable time
    TestAssertions::assert_latency_acceptable(total_duration, 10000); // 10 seconds max

    // Test 2: Resource access performance
    let resources_for_concurrent = resources.clone();
    let resource_results =
        PerformanceTestHelper::run_concurrent_requests(concurrent_requests, move || {
            let resources = resources_for_concurrent.clone();
            async move {
                let start = std::time::Instant::now();
                let result = resources
                    .read_resource("mcp://fortitude/cache/statistics")
                    .await;
                let duration = start.elapsed();
                result
                    .map(|_| duration)
                    .map_err(|e| anyhow::anyhow!("Resource access failed: {}", e))
            }
        })
        .await;

    let resource_latencies: Vec<std::time::Duration> = resource_results
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    if !resource_latencies.is_empty() {
        let avg_resource_latency =
            PerformanceTestHelper::calculate_average_latency(&resource_latencies);
        TestAssertions::assert_latency_acceptable(avg_resource_latency, 100); // 100ms average
    }

    // Test 3: Authentication performance
    let auth_manager = env.auth_manager.clone();
    let auth_middleware = Arc::new(AuthMiddleware::new(auth_manager.clone()));

    let token = auth_manager
        .generate_token("perf_test", vec![Permission::ResearchRead])
        .await
        .unwrap();
    let auth_header = format!("Bearer {token}");

    let auth_results =
        PerformanceTestHelper::run_concurrent_requests(concurrent_requests, move || {
            let auth_middleware = auth_middleware.clone();
            let auth_header = auth_header.clone();
            async move {
                let start = std::time::Instant::now();
                let result = auth_middleware
                    .authenticate_request(
                        Some(&auth_header),
                        "perf_client",
                        Permission::ResearchRead,
                    )
                    .await;
                let duration = start.elapsed();
                result
                    .map(|_| duration)
                    .map_err(|e| anyhow::anyhow!("Auth failed: {}", e))
            }
        })
        .await;

    let auth_latencies: Vec<std::time::Duration> =
        auth_results.into_iter().filter_map(|r| r.ok()).collect();

    if !auth_latencies.is_empty() {
        let avg_auth_latency = PerformanceTestHelper::calculate_average_latency(&auth_latencies);
        TestAssertions::assert_latency_acceptable(avg_auth_latency, 50); // 50ms average
    }

    // Test 4: Mixed workload performance
    let tools_for_mixed = tools.clone();
    let resources_for_mixed = resources.clone();
    let mixed_results =
        PerformanceTestHelper::run_concurrent_requests(concurrent_requests, move || {
            let tools = tools_for_mixed.clone();
            let resources = resources_for_mixed.clone();
            async move {
                let operation = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
                    % 3) as u32;

                match operation {
                    0 => {
                        let request =
                            TestDataBuilder::research_query_request("Mixed workload research");
                        tools
                            .call_tool(request)
                            .await
                            .map_err(|e| anyhow::anyhow!("Research failed: {}", e))
                    }
                    1 => {
                        let request =
                            TestDataBuilder::classify_query_request("Mixed workload classify");
                        tools
                            .call_tool(request)
                            .await
                            .map_err(|e| anyhow::anyhow!("Classification failed: {}", e))
                    }
                    2 => {
                        // Resource access - convert to tool result format
                        let resource_result = resources
                            .read_resource("mcp://fortitude/cache/statistics")
                            .await
                            .map_err(|e| anyhow::anyhow!("Resource access failed: {}", e))?;

                        // Create a mock tool result for resource access
                        Ok(rmcp::model::CallToolResult {
                            content: vec![rmcp::model::Content::text(format!(
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

    let mixed_success_rate = PerformanceTestHelper::calculate_success_rate(&mixed_results);
    TestAssertions::assert_success_rate_acceptable(mixed_success_rate, 0.8); // 80% success rate

    // Test 5: System stability under sustained load
    let sustained_start = std::time::Instant::now();
    let sustained_requests = 10;

    let tools_for_sustained = tools.clone();
    let sustained_results =
        PerformanceTestHelper::run_concurrent_requests(sustained_requests, move || {
            let tools = tools_for_sustained.clone();
            async move {
                let mut local_results = Vec::new();
                for i in 0..5 {
                    let request =
                        TestDataBuilder::research_query_request(&format!("Sustained load {i}"));
                    let result = tools.call_tool(request).await;
                    local_results.push(result.is_ok());
                }
                Ok(local_results)
            }
        })
        .await;

    let sustained_duration = sustained_start.elapsed();

    // Count successful operations
    let total_sustained_operations = sustained_results
        .iter()
        .filter_map(|r| r.as_ref().ok())
        .map(|v| v.len())
        .sum::<usize>();

    let successful_sustained_operations = sustained_results
        .iter()
        .filter_map(|r| r.as_ref().ok())
        .map(|v| v.iter().filter(|&&success| success).count())
        .sum::<usize>();

    if total_sustained_operations > 0 {
        let sustained_success_rate =
            successful_sustained_operations as f64 / total_sustained_operations as f64;
        TestAssertions::assert_success_rate_acceptable(sustained_success_rate, 0.8);
    }

    // Should complete sustained load within reasonable time
    TestAssertions::assert_latency_acceptable(sustained_duration, 30000); // 30 seconds max
}

/// ANCHOR: Verifies system security boundaries
/// Tests: Input sanitization, path traversal prevention, privilege escalation prevention
/// Protects: Security boundaries and access control
#[tokio::test]
async fn test_anchor_security_boundaries() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();
    let resources = ResourceProvider::new(env.config.clone());
    let auth_manager = env.auth_manager.clone();
    let auth_middleware = AuthMiddleware::new(auth_manager.clone());

    // Test 1: Input sanitization effectiveness
    let malicious_inputs = SecurityTestHelper::generate_malicious_inputs();

    for malicious_input in malicious_inputs {
        let request = TestDataBuilder::research_query_request(&malicious_input);
        let result = tools.call_tool(request).await;

        // Should either sanitize and process safely, or reject
        if let Ok(response) = result {
            // If processed, verify it's safe
            assert_eq!(
                response.is_error,
                Some(false),
                "Processed input should be marked as successful"
            );

            // Verify response doesn't contain unsanitized malicious content
            if let Some(content) = response.content[0].as_text() {
                assert!(
                    !content.text.contains("<script>"),
                    "Response should not contain unsanitized script tags"
                );
                assert!(
                    !content.text.contains("DROP TABLE"),
                    "Response should not contain SQL injection"
                );
            }
        }
        // If rejected, that's also a valid security response
    }

    // Test 2: Path traversal prevention
    let path_traversal_attempts = SecurityTestHelper::generate_path_traversal_attempts();

    for traversal_attempt in path_traversal_attempts {
        let malicious_uri = format!("mcp://fortitude/docs/{traversal_attempt}");
        let result = resources.read_resource(&malicious_uri).await;
        assert!(
            result.is_err(),
            "Should reject path traversal attempt: {malicious_uri}"
        );
    }

    // Test 3: Privilege escalation prevention
    let limited_token = auth_manager
        .generate_token("limited_user", vec![Permission::ResearchRead])
        .await
        .unwrap();
    let limited_header = format!("Bearer {limited_token}");

    // Should not be able to access admin functions
    let result = auth_middleware
        .authenticate_request(Some(&limited_header), "limited_client", Permission::Admin)
        .await;
    assert!(result.is_err(), "Should prevent privilege escalation");

    // Should not be able to access config resources
    let result = auth_middleware
        .authenticate_request(
            Some(&limited_header),
            "limited_client",
            Permission::ConfigRead,
        )
        .await;
    assert!(result.is_err(), "Should prevent access to config resources");

    // Test 4: Token manipulation resistance
    let valid_token = auth_manager
        .generate_token("test_user", vec![Permission::ResearchRead])
        .await
        .unwrap();

    // Try to manipulate token
    let manipulated_tokens = vec![
        valid_token.clone() + "extra",
        valid_token.chars().rev().collect::<String>(),
        valid_token.replace('A', "B"),
        format!("{valid_token}invalid"),
    ];

    for manipulated_token in manipulated_tokens {
        let result = auth_middleware
            .authenticate_request(
                Some(&format!("Bearer {manipulated_token}")),
                "test_client",
                Permission::ResearchRead,
            )
            .await;
        assert!(result.is_err(), "Should reject manipulated token");
    }

    // Test 5: Resource access boundary enforcement
    let secure_resource_uris = vec![
        "mcp://fortitude/docs/reference_library/secret.txt",
        "mcp://fortitude/docs/../config/production.yaml",
        "mcp://fortitude/system/admin",
        "mcp://fortitude/auth/tokens",
    ];

    for secure_uri in secure_resource_uris {
        let result = resources.read_resource(secure_uri).await;
        assert!(
            result.is_err(),
            "Should enforce resource access boundaries: {secure_uri}"
        );
    }

    // Test 6: Rate limiting as security boundary
    let mut rate_limited_auth = AuthManager::new(env.config.clone()).unwrap();
    rate_limited_auth.set_rate_limit_config(fortitude_mcp_server::RateLimitConfig {
        max_requests_per_minute: 5,
        window_seconds: 60,
    });

    let rate_limited_auth = Arc::new(rate_limited_auth);
    let rate_limited_middleware = AuthMiddleware::new(rate_limited_auth.clone());

    let token = rate_limited_auth
        .generate_token("rate_test", vec![Permission::ResearchRead])
        .await
        .unwrap();
    let auth_header = format!("Bearer {token}");

    // Attempt to exceed rate limit
    let mut successful_requests = 0;
    for i in 0..10 {
        let result = rate_limited_middleware
            .authenticate_request(
                Some(&auth_header),
                "security_rate_test",
                Permission::ResearchRead,
            )
            .await;

        if result.is_ok() {
            successful_requests += 1;
        }

        if i >= 5 {
            assert!(
                result.is_err(),
                "Should enforce rate limit after {i} requests"
            );
        }
    }

    assert_eq!(successful_requests, 5, "Should allow exactly 5 requests");

    // Test 7: JWT security features
    let token_claims = auth_manager.verify_token(&valid_token).await.unwrap();

    // Verify token has proper security features
    assert!(!token_claims.sub.is_empty(), "Subject should be present");
    assert!(!token_claims.iss.is_empty(), "Issuer should be present");
    assert!(
        token_claims.exp > chrono::Utc::now().timestamp(),
        "Token should not be expired"
    );
    assert!(
        token_claims.iat <= chrono::Utc::now().timestamp(),
        "Issued at should be valid"
    );

    // Test 8: Configuration security
    let config_uri = "mcp://fortitude/config/current";
    let config_result = resources.read_resource(config_uri).await.unwrap();

    if let rmcp::model::ResourceContents::TextResourceContents { text, .. } = &config_result[0] {
        let config: serde_json::Value = serde_json::from_str(text).unwrap();

        // Verify sensitive information is redacted
        assert_eq!(
            config["auth"]["jwt_secret"], "[REDACTED]",
            "JWT secret must be redacted"
        );

        // Verify no sensitive environment variables are exposed
        assert!(
            !text.contains("password"),
            "Config should not contain passwords"
        );
        assert!(
            !text.contains("secret"),
            "Config should not contain secrets"
        );
        assert!(
            !text.contains("key"),
            "Config should not contain keys (except redacted)"
        );
    }

    // Test 9: Error information disclosure prevention
    let error_cases = vec![
        TestDataBuilder::invalid_tool_request("nonexistent_tool"),
        TestDataBuilder::malformed_request("research_query"),
        TestDataBuilder::empty_query_request(),
    ];

    for error_case in error_cases {
        let result = tools.call_tool(error_case).await;
        assert!(result.is_err(), "Should produce error");

        let error_msg = result.unwrap_err().to_string();

        // Error messages should not disclose sensitive information
        assert!(
            !error_msg.contains("password"),
            "Error should not disclose passwords"
        );
        assert!(
            !error_msg.contains("secret"),
            "Error should not disclose secrets"
        );
        assert!(
            !error_msg.contains("token"),
            "Error should not disclose tokens"
        );
        assert!(
            !error_msg.to_lowercase().contains("internal"),
            "Error should not disclose internal details"
        );
    }

    // Test 10: Concurrent security testing
    let concurrent_security_tests = 10;
    let security_results =
        PerformanceTestHelper::run_concurrent_requests(concurrent_security_tests, move || {
            let resources = ResourceProvider::new(env.config.clone());
            async move {
                let malicious_uri = "mcp://fortitude/docs/../../../etc/passwd";
                resources
                    .read_resource(malicious_uri)
                    .await
                    .map_err(|_| anyhow::anyhow!("Security test passed - request was rejected"))
            }
        })
        .await;

    // All security tests should fail (requests should be rejected)
    let security_violations = security_results.iter().filter(|r| r.is_ok()).count();

    assert_eq!(
        security_violations, 0,
        "No security violations should occur under concurrent load"
    );
}

/// ANCHOR: Verifies proactive research MCP tools work end-to-end
/// Tests: Tool listing, validation, start/stop lifecycle, status reporting, configuration
/// Protects: Proactive research integration with MCP protocol
#[tokio::test]
async fn test_anchor_proactive_research_mcp_tools() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test 1: Proactive tools are properly listed in MCP protocol
    let tools_list = tools.list_tools();
    let tool_names: Vec<&str> = tools_list.tools.iter().map(|t| t.name.as_ref()).collect();

    // Verify all proactive tools are present
    assert!(
        tool_names.contains(&"proactive_start"),
        "Should have proactive_start tool"
    );
    assert!(
        tool_names.contains(&"proactive_stop"),
        "Should have proactive_stop tool"
    );
    assert!(
        tool_names.contains(&"proactive_status"),
        "Should have proactive_status tool"
    );
    assert!(
        tool_names.contains(&"proactive_configure"),
        "Should have proactive_configure tool"
    );
    assert!(
        tool_names.contains(&"proactive_list_tasks"),
        "Should have proactive_list_tasks tool"
    );
    assert!(
        tool_names.contains(&"proactive_get_notifications"),
        "Should have proactive_get_notifications tool"
    );

    // Verify tool schemas are properly formatted
    for tool in &tools_list.tools {
        if tool.name.starts_with("proactive_") {
            assert!(
                !tool.name.is_empty(),
                "Proactive tool name must not be empty"
            );
            assert!(
                tool.description.is_some(),
                "Proactive tool description must be provided"
            );
            assert!(
                !tool.input_schema.is_empty(),
                "Proactive tool input schema must be provided"
            );
        }
    }

    // Test 2: Proactive status tool works when system is stopped
    let status_request = rmcp::model::CallToolRequestParam {
        name: "proactive_status".into(),
        arguments: Some(
            serde_json::json!({
                "detailed": true,
                "include_metrics": true
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
    };

    let status_result = tools.call_tool(status_request).await.unwrap();
    assert_eq!(status_result.is_error, Some(false));
    assert!(!status_result.content.is_empty());

    // Verify status response structure
    if let Some(content) = status_result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();

        assert!(
            response.get("is_running").is_some(),
            "Should indicate running status"
        );
        assert!(
            response.get("health_metrics").is_some(),
            "Should provide health metrics"
        );
        assert!(
            response.get("config_summary").is_some(),
            "Should provide config summary"
        );
        assert!(
            response.get("processing_time_ms").is_some(),
            "Should track processing time"
        );

        // Should indicate not running initially
        assert_eq!(
            response.get("is_running").unwrap(),
            false,
            "Should not be running initially"
        );
    }

    // Test 3: Proactive start tool functionality
    let start_request = rmcp::model::CallToolRequestParam {
        name: "proactive_start".into(),
        arguments: Some(
            serde_json::json!({
                "config": {
                    "base_directory": "/test/path",
                    "monitoring_interval_seconds": 300,
                    "max_concurrent_tasks": 5
                }
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
    };

    let start_result = tools.call_tool(start_request).await.unwrap();
    assert_eq!(start_result.is_error, Some(false));
    assert!(!start_result.content.is_empty());

    // Verify start response structure
    if let Some(content) = start_result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();

        assert!(
            response.get("is_running").is_some(),
            "Should indicate running status"
        );
        assert!(
            response.get("status").is_some(),
            "Should provide status message"
        );
        assert!(
            response.get("health_metrics").is_some(),
            "Should provide health metrics"
        );

        // Should indicate running after start
        assert_eq!(
            response.get("is_running").unwrap(),
            true,
            "Should be running after start"
        );
        assert_eq!(
            response.get("status").unwrap(),
            "started",
            "Should show started status"
        );
    }

    // Test 4: Proactive configure tool functionality
    let configure_request = rmcp::model::CallToolRequestParam {
        name: "proactive_configure".into(),
        arguments: Some(
            serde_json::json!({
                "config": {
                    "gap_interval_minutes": 15,
                    "max_concurrent_tasks": 3,
                    "auto_persist": true
                }
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
    };

    let configure_result = tools.call_tool(configure_request).await.unwrap();
    assert_eq!(configure_result.is_error, Some(false));
    assert!(!configure_result.content.is_empty());

    // Verify configure response structure
    if let Some(content) = configure_result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();

        assert!(
            response.get("updated_config").is_some(),
            "Should return updated config"
        );
        assert!(
            response.get("changes_applied").is_some(),
            "Should list changes applied"
        );
        assert!(
            response.get("restart_required").is_some(),
            "Should indicate if restart needed"
        );
    }

    // Test 5: Proactive list tasks tool functionality
    let list_tasks_request = rmcp::model::CallToolRequestParam {
        name: "proactive_list_tasks".into(),
        arguments: Some(
            serde_json::json!({
                "status": "active",
                "limit": 10,
                "offset": 0
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
    };

    let list_result = tools.call_tool(list_tasks_request).await.unwrap();
    assert_eq!(list_result.is_error, Some(false));
    assert!(!list_result.content.is_empty());

    // Verify list tasks response structure
    if let Some(content) = list_result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();

        assert!(response.get("tasks").is_some(), "Should provide tasks list");
        assert!(
            response.get("total_count").is_some(),
            "Should provide total count"
        );
        assert!(
            response.get("pagination").is_some(),
            "Should provide pagination info"
        );
        assert!(
            response.get("task_statistics").is_some(),
            "Should provide task statistics"
        );
    }

    // Test 6: Proactive get notifications tool functionality
    let notifications_request = rmcp::model::CallToolRequestParam {
        name: "proactive_get_notifications".into(),
        arguments: Some(
            serde_json::json!({
                "unread_only": true,
                "limit": 20,
                "since_minutes": 60
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
    };

    let notifications_result = tools.call_tool(notifications_request).await.unwrap();
    assert_eq!(notifications_result.is_error, Some(false));
    assert!(!notifications_result.content.is_empty());

    // Verify notifications response structure
    if let Some(content) = notifications_result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();

        assert!(
            response.get("notifications").is_some(),
            "Should provide notifications list"
        );
        assert!(
            response.get("total_count").is_some(),
            "Should provide total count"
        );
        assert!(
            response.get("unread_count").is_some(),
            "Should provide unread count"
        );
        assert!(
            response.get("notification_statistics").is_some(),
            "Should provide statistics"
        );
    }

    // Test 7: Proactive stop tool functionality
    let stop_request = rmcp::model::CallToolRequestParam {
        name: "proactive_stop".into(),
        arguments: Some(
            serde_json::json!({
                "force": false,
                "timeout_seconds": 30
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
    };

    let stop_result = tools.call_tool(stop_request).await.unwrap();
    assert_eq!(stop_result.is_error, Some(false));
    assert!(!stop_result.content.is_empty());

    // Verify stop response structure
    if let Some(content) = stop_result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();

        assert!(
            response.get("is_running").is_some(),
            "Should indicate running status"
        );
        assert!(
            response.get("status").is_some(),
            "Should provide status message"
        );
        assert!(
            response.get("health_metrics").is_some(),
            "Should provide health metrics"
        );

        // Should indicate stopped after stop
        assert_eq!(
            response.get("is_running").unwrap(),
            false,
            "Should not be running after stop"
        );
        assert_eq!(
            response.get("status").unwrap(),
            "stopped",
            "Should show stopped status"
        );
    }

    // Test 8: Input validation for proactive tools
    let invalid_configure_request = rmcp::model::CallToolRequestParam {
        name: "proactive_configure".into(),
        arguments: None, // Missing required arguments
    };

    let invalid_result = tools.call_tool(invalid_configure_request).await;
    assert!(
        invalid_result.is_err(),
        "Should reject invalid configuration request"
    );

    // Test 9: Unknown proactive tool handling
    let unknown_request = rmcp::model::CallToolRequestParam {
        name: "proactive_nonexistent".into(),
        arguments: None,
    };

    let unknown_result = tools.call_tool(unknown_request).await;
    assert!(
        unknown_result.is_err(),
        "Should reject unknown proactive tool"
    );

    // Test 10: Proactive tools error handling - try to stop when not running
    let stop_when_not_running_request = rmcp::model::CallToolRequestParam {
        name: "proactive_stop".into(),
        arguments: Some(
            serde_json::json!({
                "force": false,
                "timeout_seconds": 30
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
    };

    let stop_when_not_running_result = tools.call_tool(stop_when_not_running_request).await;
    // Should either error or indicate already stopped
    if stop_when_not_running_result.is_err() {
        // Error is acceptable
    } else {
        let response = stop_when_not_running_result.unwrap();
        if let Some(content) = response.content[0].as_text() {
            let parsed: serde_json::Value = serde_json::from_str(&content.text).unwrap();
            // Should indicate not running or similar
            assert!(parsed.get("status").is_some());
        }
    }
}
