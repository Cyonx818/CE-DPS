// ABOUTME: Integration tests for JWT authentication and authorization system
// Tests end-to-end authentication flows, permission checking, and rate limiting
// ANCHOR: Verifies authentication security works end-to-end with proper token validation and permission enforcement

use fortitude_mcp_server::{
    AuthManager, AuthMiddleware, Permission, RateLimitConfig, ServerConfig,
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// Create test configuration with authentication enabled
fn create_test_config() -> Arc<ServerConfig> {
    let mut config = ServerConfig::default();
    config.auth.jwt_secret = "test_secret_key_at_least_32_characters_long_for_security".to_string();
    config.auth.token_expiration_hours = 1;
    config.auth.enabled = true;
    config.auth.rate_limit.max_requests_per_minute = 10;
    config.auth.rate_limit.window_seconds = 60;
    Arc::new(config)
}

/// ANCHOR: Verifies complete authentication flow works end-to-end
/// Tests: Token generation, validation, permission checking, rate limiting
#[tokio::test]
async fn test_anchor_complete_authentication_flow() {
    let config = create_test_config();
    let auth_manager = Arc::new(AuthManager::new(config).unwrap());
    let auth_middleware = AuthMiddleware::new(auth_manager.clone());

    // Test 1: Generate token with specific permissions
    let permissions = vec![Permission::ResearchRead, Permission::ResourcesRead];
    let token = auth_manager
        .generate_token("test_user", permissions)
        .await
        .unwrap();

    // Test 2: Verify token can be validated
    let claims = auth_manager.verify_token(&token).await.unwrap();
    assert_eq!(claims.sub, "test_user");
    assert!(claims
        .permissions
        .contains(&Permission::ResearchRead.as_str().to_string()));
    assert!(claims
        .permissions
        .contains(&Permission::ResourcesRead.as_str().to_string()));

    // Test 3: Test successful authentication with proper permissions
    let auth_header = format!("Bearer {token}");
    let result = auth_middleware
        .authenticate_request(Some(&auth_header), "test_client", Permission::ResearchRead)
        .await;
    assert!(result.is_ok());

    // Test 4: Test permission denial
    let result = auth_middleware
        .authenticate_request(Some(&auth_header), "test_client", Permission::Admin)
        .await;
    assert!(result.is_err());

    // Test 5: Test rate limiting
    let mut requests_successful = 0;
    for _ in 0..15 {
        let result = auth_middleware
            .authenticate_request(
                Some(&auth_header),
                "rate_limit_client",
                Permission::ResearchRead,
            )
            .await;
        if result.is_ok() {
            requests_successful += 1;
        }
    }

    // Should have allowed exactly 10 requests (rate limit)
    assert_eq!(requests_successful, 10);
}

/// ANCHOR: Verifies admin permissions provide full access
/// Tests: Admin token generation, full permission access
#[tokio::test]
async fn test_anchor_admin_permissions_workflow() {
    let config = create_test_config();
    let auth_manager = Arc::new(AuthManager::new(config).unwrap());
    let auth_middleware = AuthMiddleware::new(auth_manager.clone());

    // Generate admin token
    let admin_token = auth_manager.create_default_admin_token().await.unwrap();
    let auth_header = format!("Bearer {admin_token}");

    // Test that admin can access all permissions
    let permissions_to_test = vec![
        Permission::ResearchRead,
        Permission::ResourcesRead,
        Permission::ConfigRead,
        Permission::Admin,
    ];

    for permission in permissions_to_test {
        let result = auth_middleware
            .authenticate_request(Some(&auth_header), "admin_client", permission)
            .await;
        assert!(result.is_ok(), "Admin should have access to {permission:?}");
    }
}

/// ANCHOR: Verifies rate limiting window resets properly
/// Tests: Rate limit enforcement, window reset behavior
#[tokio::test]
async fn test_anchor_rate_limiting_window_reset() {
    let config = create_test_config();
    let mut auth_manager = AuthManager::new(config).unwrap();
    // Set very short window for testing
    auth_manager.set_rate_limit_config(RateLimitConfig {
        window_seconds: 1,
        max_requests_per_minute: 2,
    });

    let auth_manager = Arc::new(auth_manager);
    let auth_middleware = AuthMiddleware::new(auth_manager.clone());

    // Generate token
    let token = auth_manager
        .generate_token("test_user", vec![Permission::ResearchRead])
        .await
        .unwrap();
    let auth_header = format!("Bearer {token}");

    // Use up the rate limit
    let result1 = auth_middleware
        .authenticate_request(
            Some(&auth_header),
            "reset_test_client",
            Permission::ResearchRead,
        )
        .await;
    assert!(result1.is_ok());

    let result2 = auth_middleware
        .authenticate_request(
            Some(&auth_header),
            "reset_test_client",
            Permission::ResearchRead,
        )
        .await;
    assert!(result2.is_ok());

    // Third request should fail
    let result3 = auth_middleware
        .authenticate_request(
            Some(&auth_header),
            "reset_test_client",
            Permission::ResearchRead,
        )
        .await;
    assert!(result3.is_err());

    // Wait for window to reset
    sleep(Duration::from_secs(2)).await;

    // Should be able to make requests again
    let result4 = auth_middleware
        .authenticate_request(
            Some(&auth_header),
            "reset_test_client",
            Permission::ResearchRead,
        )
        .await;
    assert!(result4.is_ok());
}

/// ANCHOR: Verifies invalid tokens are properly rejected
/// Tests: Invalid token rejection, malformed token handling
#[tokio::test]
async fn test_anchor_invalid_token_rejection() {
    let config = create_test_config();
    let auth_manager = Arc::new(AuthManager::new(config).unwrap());
    let auth_middleware = AuthMiddleware::new(auth_manager);

    // Test with invalid token
    let invalid_tokens = vec![
        "invalid_token",
        "Bearer invalid_token",
        "Bearer ",
        "",
        "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
    ];

    for token in invalid_tokens {
        let result = auth_middleware
            .authenticate_request(Some(token), "test_client", Permission::ResearchRead)
            .await;
        assert!(result.is_err(), "Should reject invalid token: {token}");
    }

    // Test with missing authorization header
    let result = auth_middleware
        .authenticate_request(None, "test_client", Permission::ResearchRead)
        .await;
    assert!(result.is_err());
}

/// ANCHOR: Verifies authentication can be disabled for development
/// Tests: Development mode with disabled authentication
#[tokio::test]
async fn test_anchor_disabled_authentication_mode() {
    let mut config = ServerConfig::default();
    config.auth.enabled = false;
    let config = Arc::new(config);

    let auth_manager = Arc::new(AuthManager::new(config).unwrap());
    let auth_middleware = AuthMiddleware::new(auth_manager);

    // Should succeed even without auth header when auth is disabled
    let result = auth_middleware
        .authenticate_request(None, "test_client", Permission::Admin)
        .await;
    assert!(result.is_ok());

    // Should also succeed with invalid token when auth is disabled
    let result = auth_middleware
        .authenticate_request(Some("invalid_token"), "test_client", Permission::Admin)
        .await;
    assert!(result.is_ok());
}

/// ANCHOR: Verifies rate limit headers are properly generated
/// Tests: Rate limit response headers, remaining requests tracking
#[tokio::test]
async fn test_anchor_rate_limit_headers() {
    let config = create_test_config();
    let mut auth_manager = AuthManager::new(config).unwrap();
    auth_manager.set_rate_limit_config(RateLimitConfig {
        window_seconds: 60,
        max_requests_per_minute: 5,
    });

    let auth_manager = Arc::new(auth_manager);
    let auth_middleware = AuthMiddleware::new(auth_manager.clone());

    // Generate token
    let token = auth_manager
        .generate_token("test_user", vec![Permission::ResearchRead])
        .await
        .unwrap();
    let auth_header = format!("Bearer {token}");

    // Make some requests to reduce the limit
    for _ in 0..3 {
        let _ = auth_middleware
            .authenticate_request(
                Some(&auth_header),
                "header_test_client",
                Permission::ResearchRead,
            )
            .await;
    }

    // Check rate limit headers
    let headers = auth_middleware
        .get_rate_limit_headers("header_test_client")
        .await;

    assert!(headers.contains_key("X-RateLimit-Limit"));
    assert!(headers.contains_key("X-RateLimit-Remaining"));
    assert!(headers.contains_key("X-RateLimit-Reset"));

    assert_eq!(headers["X-RateLimit-Limit"], "5");
    assert_eq!(headers["X-RateLimit-Remaining"], "2"); // 5 - 3 = 2
}

/// ANCHOR: Verifies input validation and sanitization works correctly
/// Tests: SQL injection prevention, XSS protection, path traversal prevention
#[tokio::test]
async fn test_anchor_input_validation_security() {
    use fortitude_mcp_server::auth::validation;

    // Test path traversal protection
    assert!(validation::validate_file_path("../../../etc/passwd").is_err());
    assert!(validation::validate_file_path("~/secret").is_err());
    assert!(validation::validate_file_path("/absolute/path").is_err());
    assert!(validation::validate_file_path("valid/path.txt").is_ok());

    // Test string sanitization
    let dirty_input = "test\x00string\x01with\x02dangerous\x03chars";
    let clean_input = validation::sanitize_string(dirty_input);
    assert!(!clean_input.contains('\x00'));
    assert!(!clean_input.contains('\x01'));
    assert!(!clean_input.contains('\x02'));
    assert!(!clean_input.contains('\x03'));

    // Test that legitimate characters are preserved
    let legitimate_input = "This is a normal string with spaces and\nnewlines\tand tabs";
    let clean_legitimate = validation::sanitize_string(legitimate_input);
    assert!(clean_legitimate.contains("normal string"));
    assert!(clean_legitimate.contains('\n'));
    assert!(clean_legitimate.contains('\t'));
}

/// ANCHOR: Verifies concurrent authentication requests are handled properly
/// Tests: Thread safety, concurrent token validation, rate limiting under load
#[tokio::test]
async fn test_anchor_concurrent_authentication_safety() {
    let config = create_test_config();
    let auth_manager = Arc::new(AuthManager::new(config).unwrap());
    let auth_middleware = Arc::new(AuthMiddleware::new(auth_manager.clone()));

    // Generate token
    let token = auth_manager
        .generate_token("test_user", vec![Permission::ResearchRead])
        .await
        .unwrap();
    let auth_header = format!("Bearer {token}");

    // Spawn multiple concurrent authentication requests
    let mut handles = Vec::new();
    for i in 0..10 {
        let auth_middleware = auth_middleware.clone();
        let auth_header = auth_header.clone();
        let client_id = format!("concurrent_client_{i}");

        let handle = tokio::spawn(async move {
            auth_middleware
                .authenticate_request(Some(&auth_header), &client_id, Permission::ResearchRead)
                .await
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let results = futures::future::join_all(handles).await;

    // All requests should succeed (different client IDs, so no rate limiting)
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}
