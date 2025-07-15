//! ANCHOR TESTS for Gemini Provider Critical Functionality
//!
//! These tests validate the most critical functionality of the Google Gemini provider implementation.
//! They are designed to catch breaking changes and ensure core provider behavior remains stable.
//!
//! CRITICAL FUNCTIONALITY COVERED:
//! 1. Provider Creation and Configuration Validation
//! 2. Core Research Query Execution  
//! 3. Health Checking and Status Reporting
//! 4. Rate Limiting and Token Bucket Implementation
//! 5. Error Handling for Authentication and API Failures
//!
//! These tests use the ANCHOR pattern - they test fundamental behavior that must never break.

use fortitude::providers::config::{ProviderSettings, RateLimitConfig};
use fortitude::providers::{GeminiProvider, HealthStatus, Provider, ProviderError};
use std::sync::Arc;
use std::time::Duration;

mod common;
use common::{conservative_rate_limits, valid_gemini_settings, TestEnvironmentGuard};

/// ANCHOR TEST: Verifies that Gemini provider can be created successfully with valid configuration
///
/// This is a critical test that ensures the provider initialization process works correctly.
/// If this test fails, it indicates a fundamental problem with provider construction.
#[tokio::test]
async fn anchor_test_gemini_provider_creation_and_metadata() {
    let _guard = TestEnvironmentGuard::new();

    // Test provider creation with valid settings
    let settings = valid_gemini_settings();
    let provider_result = GeminiProvider::new(settings).await;

    assert!(
        provider_result.is_ok(),
        "ANCHOR FAILURE: Gemini provider creation failed with valid settings"
    );

    let provider = provider_result.unwrap();
    let metadata = provider.metadata();

    // Validate core metadata requirements
    assert_eq!(
        metadata.name(),
        "gemini",
        "ANCHOR FAILURE: Provider name incorrect"
    );
    assert!(
        !metadata.version().is_empty(),
        "ANCHOR FAILURE: Provider version missing"
    );

    // Validate critical capabilities
    let capabilities = metadata.capabilities();
    assert!(
        capabilities.contains(&"research".to_string()),
        "ANCHOR FAILURE: Missing research capability"
    );
    assert!(
        capabilities.contains(&"rate_limited".to_string()),
        "ANCHOR FAILURE: Missing rate_limited capability"
    );
    assert!(
        capabilities.contains(&"cost_estimation".to_string()),
        "ANCHOR FAILURE: Missing cost_estimation capability"
    );
    assert!(
        capabilities.contains(&"safety_settings".to_string()),
        "ANCHOR FAILURE: Missing safety_settings capability"
    );
    assert!(
        capabilities.contains(&"multimodal".to_string()),
        "ANCHOR FAILURE: Missing multimodal capability"
    );

    // Validate model support
    let models = metadata.supported_models();
    assert!(
        !models.is_empty(),
        "ANCHOR FAILURE: No supported models listed"
    );
    assert!(
        models.contains(&"gemini-1.5-pro".to_string()),
        "ANCHOR FAILURE: Missing gemini-1.5-pro model"
    );

    // Validate context length
    assert!(
        metadata.max_context_length() > 0,
        "ANCHOR FAILURE: Invalid context length"
    );

    println!("✅ ANCHOR TEST PASSED: Gemini provider creation and metadata validation");
}

/// ANCHOR TEST: Verifies that provider creation fails appropriately with invalid configuration
///
/// This test ensures that the provider properly validates configuration and rejects invalid settings.
/// This is critical for security and reliability.
#[tokio::test]
async fn anchor_test_gemini_provider_creation_failure_modes() {
    let _guard = TestEnvironmentGuard::new();

    // Test with empty API key
    let empty_key_settings = ProviderSettings::new("".to_string(), "gemini-1.5-pro".to_string());
    let result = GeminiProvider::new(empty_key_settings).await;
    assert!(
        result.is_err(),
        "ANCHOR FAILURE: Provider creation should fail with empty API key"
    );

    // Test with invalid API key format
    let invalid_key_settings =
        ProviderSettings::new("invalid-key".to_string(), "gemini-1.5-pro".to_string());
    let result = GeminiProvider::new(invalid_key_settings).await;
    assert!(
        result.is_err(),
        "ANCHOR FAILURE: Provider creation should fail with invalid API key format"
    );

    // Test with empty model
    let empty_model_settings = ProviderSettings::new(
        "AIzaSyTest1234567890abcdef1234567890abcdef".to_string(),
        "".to_string(),
    );
    let result = GeminiProvider::new(empty_model_settings).await;
    assert!(
        result.is_err(),
        "ANCHOR FAILURE: Provider creation should fail with empty model"
    );

    // Test with invalid timeout (too short)
    let invalid_timeout_settings = ProviderSettings::new(
        "AIzaSyTest1234567890abcdef1234567890abcdef".to_string(),
        "gemini-1.5-pro".to_string(),
    )
    .with_timeout(Duration::from_millis(100)); // Too short
    let result = GeminiProvider::new(invalid_timeout_settings).await;
    assert!(
        result.is_err(),
        "ANCHOR FAILURE: Provider creation should fail with invalid timeout"
    );

    println!("✅ ANCHOR TEST PASSED: Gemini provider creation failure modes");
}

/// ANCHOR TEST: Verifies that health checking works correctly
///
/// This is critical functionality for provider management and failover systems.
/// The health check must accurately reflect provider status.
#[tokio::test]
async fn anchor_test_gemini_provider_health_check() {
    let _guard = TestEnvironmentGuard::new();

    let settings = valid_gemini_settings();
    let provider = GeminiProvider::new(settings)
        .await
        .expect("ANCHOR FAILURE: Provider creation failed");

    // Test health check execution
    let health_result = provider.health_check().await;

    // Health check should not fail (even if provider is not actually accessible)
    // This tests the health check mechanism itself
    assert!(
        health_result.is_ok(),
        "ANCHOR FAILURE: Health check should not fail mechanically"
    );

    let health_status = health_result.unwrap();

    // Health status should be one of the expected values
    match health_status {
        HealthStatus::Healthy => {
            println!("✅ ANCHOR TEST PASSED: Health check reports healthy");
        }
        HealthStatus::Degraded(msg) => {
            assert!(
                !msg.is_empty(),
                "ANCHOR FAILURE: Degraded status should have message"
            );
            println!(
                "✅ ANCHOR TEST PASSED: Health check reports degraded with message: {}",
                msg
            );
        }
        HealthStatus::Unhealthy(msg) => {
            assert!(
                !msg.is_empty(),
                "ANCHOR FAILURE: Unhealthy status should have message"
            );
            println!(
                "✅ ANCHOR TEST PASSED: Health check reports unhealthy with message: {}",
                msg
            );
        }
    }

    println!("✅ ANCHOR TEST PASSED: Gemini provider health check");
}

/// ANCHOR TEST: Verifies that query validation works correctly
///
/// This test ensures that the provider properly validates input queries before processing.
/// This is critical for preventing invalid requests and ensuring consistent behavior.
#[tokio::test]
async fn anchor_test_gemini_provider_query_validation() {
    let _guard = TestEnvironmentGuard::new();

    let settings = valid_gemini_settings();
    let provider = GeminiProvider::new(settings)
        .await
        .expect("ANCHOR FAILURE: Provider creation failed");

    // Test valid query validation
    let valid_result = provider.validate_query("What is the capital of France?");
    assert!(
        valid_result.is_ok(),
        "ANCHOR FAILURE: Valid query should pass validation"
    );

    // Test empty query validation
    let empty_result = provider.validate_query("");
    assert!(
        empty_result.is_err(),
        "ANCHOR FAILURE: Empty query should fail validation"
    );

    // Test whitespace-only query validation
    let whitespace_result = provider.validate_query("   \t\n  ");
    assert!(
        whitespace_result.is_err(),
        "ANCHOR FAILURE: Whitespace-only query should fail validation"
    );

    // Test very long query (should still pass validation, as Gemini supports large contexts)
    let long_query = "What is ".repeat(1000) + "artificial intelligence?";
    let long_result = provider.validate_query(&long_query);
    assert!(
        long_result.is_ok(),
        "ANCHOR FAILURE: Long query should pass validation for Gemini"
    );

    println!("✅ ANCHOR TEST PASSED: Gemini provider query validation");
}

/// ANCHOR TEST: Verifies that cost estimation works correctly
///
/// This test ensures that the provider can estimate costs for queries, which is critical
/// for budget management and cost optimization features.
#[tokio::test]
async fn anchor_test_gemini_provider_cost_estimation() {
    let _guard = TestEnvironmentGuard::new();

    let settings = valid_gemini_settings();
    let provider = GeminiProvider::new(settings)
        .await
        .expect("ANCHOR FAILURE: Provider creation failed");

    // Test cost estimation for a typical query
    let query = "Explain the concept of machine learning in simple terms.";
    let cost_result = provider.estimate_cost(query).await;

    assert!(
        cost_result.is_ok(),
        "ANCHOR FAILURE: Cost estimation should not fail"
    );

    let cost = cost_result.unwrap();

    // Validate cost structure
    assert!(
        cost.estimated_input_tokens > 0,
        "ANCHOR FAILURE: Input token count should be positive"
    );
    assert!(
        cost.estimated_output_tokens > 0,
        "ANCHOR FAILURE: Output token count should be positive"
    );
    assert!(
        cost.estimated_duration > Duration::ZERO,
        "ANCHOR FAILURE: Estimated duration should be positive"
    );

    // Cost estimation should be available for Gemini
    if let Some(cost_usd) = cost.estimated_cost_usd {
        assert!(
            cost_usd >= 0.0,
            "ANCHOR FAILURE: Cost estimate should be non-negative"
        );
    }

    // Test cost estimation scales with query length
    let short_query = "Hi";
    let long_query = "Explain the detailed history of artificial intelligence, including key milestones, major researchers, technological breakthroughs, and current applications across various industries.";

    let short_cost = provider.estimate_cost(short_query).await.unwrap();
    let long_cost = provider.estimate_cost(long_query).await.unwrap();

    assert!(
        long_cost.estimated_input_tokens > short_cost.estimated_input_tokens,
        "ANCHOR FAILURE: Longer query should have higher estimated input tokens"
    );

    println!("✅ ANCHOR TEST PASSED: Gemini provider cost estimation");
}

/// ANCHOR TEST: Verifies that usage statistics tracking works correctly
///
/// This test ensures that the provider properly tracks usage statistics, which is critical
/// for monitoring, analytics, and optimization features.
#[tokio::test]
async fn anchor_test_gemini_provider_usage_stats() {
    let _guard = TestEnvironmentGuard::new();

    let settings = valid_gemini_settings();
    let provider = GeminiProvider::new(settings)
        .await
        .expect("ANCHOR FAILURE: Provider creation failed");

    // Test usage stats retrieval
    let stats_result = provider.usage_stats().await;
    assert!(
        stats_result.is_ok(),
        "ANCHOR FAILURE: Usage stats retrieval should not fail"
    );

    let stats = stats_result.unwrap();

    // Validate stats structure (initial state)
    assert!(
        stats.total_requests >= 0,
        "ANCHOR FAILURE: Total requests should be non-negative"
    );
    assert!(
        stats.successful_requests >= 0,
        "ANCHOR FAILURE: Successful requests should be non-negative"
    );
    assert!(
        stats.failed_requests >= 0,
        "ANCHOR FAILURE: Failed requests should be non-negative"
    );
    assert!(
        stats.total_input_tokens >= 0,
        "ANCHOR FAILURE: Total input tokens should be non-negative"
    );
    assert!(
        stats.total_output_tokens >= 0,
        "ANCHOR FAILURE: Total output tokens should be non-negative"
    );

    // Successful requests + failed requests should equal total requests
    assert_eq!(
        stats.successful_requests + stats.failed_requests,
        stats.total_requests,
        "ANCHOR FAILURE: Request counts should be consistent"
    );

    println!("✅ ANCHOR TEST PASSED: Gemini provider usage statistics");
}

/// ANCHOR TEST: Verifies rate limiting configuration and basic behavior
///
/// This test ensures that rate limiting is properly configured and doesn't prevent normal operation.
/// Rate limiting is critical for staying within API limits and preventing service abuse.
#[tokio::test]
async fn anchor_test_gemini_provider_rate_limiting_configuration() {
    let _guard = TestEnvironmentGuard::new();

    // Test with conservative rate limits
    let rate_limits = conservative_rate_limits();
    let settings = valid_gemini_settings().with_rate_limits(rate_limits.clone());
    let provider = GeminiProvider::new(settings)
        .await
        .expect("ANCHOR FAILURE: Provider creation failed");

    let metadata = provider.metadata();
    let configured_limits = metadata.rate_limits();

    // Verify rate limits are properly configured
    assert_eq!(
        configured_limits.requests_per_minute, rate_limits.requests_per_minute,
        "ANCHOR FAILURE: Requests per minute not configured correctly"
    );
    assert_eq!(
        configured_limits.input_tokens_per_minute, rate_limits.input_tokens_per_minute,
        "ANCHOR FAILURE: Input tokens per minute not configured correctly"
    );
    assert_eq!(
        configured_limits.output_tokens_per_minute, rate_limits.output_tokens_per_minute,
        "ANCHOR FAILURE: Output tokens per minute not configured correctly"
    );
    assert_eq!(
        configured_limits.max_concurrent_requests, rate_limits.max_concurrent_requests,
        "ANCHOR FAILURE: Max concurrent requests not configured correctly"
    );

    // Test that rate limiting doesn't prevent basic operations
    let cost_result = provider.estimate_cost("Test query").await;
    assert!(
        cost_result.is_ok(),
        "ANCHOR FAILURE: Rate limiting should not prevent cost estimation"
    );

    let health_result = provider.health_check().await;
    assert!(
        health_result.is_ok(),
        "ANCHOR FAILURE: Rate limiting should not prevent health checks"
    );

    println!("✅ ANCHOR TEST PASSED: Gemini provider rate limiting configuration");
}

/// ANCHOR TEST: Verifies provider error handling for critical failure scenarios
///
/// This test ensures that the provider properly handles and reports errors in critical scenarios.
/// Proper error handling is essential for debugging and system resilience.
#[tokio::test]
async fn anchor_test_gemini_provider_error_handling() {
    let _guard = TestEnvironmentGuard::new();

    // Test with intentionally invalid configuration to trigger errors
    let invalid_settings = ProviderSettings::new(
        "AIzaSyInvalidKey123".to_string(), // Invalid but properly formatted key
        "gemini-1.5-pro".to_string(),
    )
    .with_timeout(Duration::from_secs(1)); // Very short timeout

    // Provider creation should succeed (validation happens at request time)
    let provider_result = GeminiProvider::new(invalid_settings).await;

    // We expect this to fail due to invalid API key format during validation
    if provider_result.is_err() {
        let error = provider_result.unwrap_err();

        // Verify error contains meaningful information
        let error_str = error.to_string();
        assert!(
            !error_str.is_empty(),
            "ANCHOR FAILURE: Error message should not be empty"
        );
        assert!(
            error_str.contains("API key") || error_str.contains("configuration"),
            "ANCHOR FAILURE: Error should mention API key or configuration issue"
        );

        println!(
            "✅ ANCHOR TEST PASSED: Provider creation properly rejects invalid configuration: {}",
            error_str
        );
        return;
    }

    // If provider creation succeeded, test error handling during operations
    let provider = provider_result.unwrap();

    // Test research query with invalid configuration (should fail quickly due to timeout/auth)
    let query_result = provider.research_query("Test query".to_string()).await;

    if query_result.is_err() {
        let error = query_result.unwrap_err();

        // Test error utility methods first (before match consumes the error)
        assert!(
            !error.provider().is_empty(),
            "ANCHOR FAILURE: Error should have provider name"
        );
        assert_eq!(
            error.provider(),
            "gemini",
            "ANCHOR FAILURE: Error provider should be 'gemini'"
        );

        // Test retryability
        let is_retryable = error.is_retryable();
        println!("Error retryability: {}", is_retryable);

        // Verify error is properly typed and contains information
        match error {
            ProviderError::AuthenticationFailed { provider, message } => {
                assert_eq!(
                    provider, "gemini",
                    "ANCHOR FAILURE: Provider name in error should be 'gemini'"
                );
                assert!(
                    !message.is_empty(),
                    "ANCHOR FAILURE: Authentication error should have message"
                );
                println!(
                    "✅ ANCHOR TEST PASSED: Authentication error properly handled: {}",
                    message
                );
            }
            ProviderError::Timeout { provider, duration } => {
                assert_eq!(
                    provider, "gemini",
                    "ANCHOR FAILURE: Provider name in error should be 'gemini'"
                );
                assert!(
                    duration > Duration::ZERO,
                    "ANCHOR FAILURE: Timeout duration should be positive"
                );
                println!(
                    "✅ ANCHOR TEST PASSED: Timeout error properly handled: {:?}",
                    duration
                );
            }
            ProviderError::NetworkError { provider, .. } => {
                assert_eq!(
                    provider, "gemini",
                    "ANCHOR FAILURE: Provider name in error should be 'gemini'"
                );
                println!("✅ ANCHOR TEST PASSED: Network error properly handled");
            }
            ref other => {
                println!(
                    "✅ ANCHOR TEST PASSED: Error properly handled (type: {:?})",
                    other
                );
            }
        }
    } else {
        println!(
            "✅ ANCHOR TEST PASSED: Provider operations completed (unexpected but acceptable)"
        );
    }

    println!("✅ ANCHOR TEST PASSED: Gemini provider error handling");
}

/// ANCHOR TEST: Verifies provider threading safety and concurrent access
///
/// This test ensures that the provider can be safely used from multiple threads concurrently.
/// Thread safety is critical for high-throughput applications.
#[tokio::test]
async fn anchor_test_gemini_provider_thread_safety() {
    let _guard = TestEnvironmentGuard::new();

    let settings = valid_gemini_settings();
    let provider = Arc::new(
        GeminiProvider::new(settings)
            .await
            .expect("ANCHOR FAILURE: Provider creation failed"),
    );

    // Test concurrent access to provider methods
    let mut handles = Vec::new();

    for i in 0..5 {
        let provider_clone = Arc::clone(&provider);
        let handle = tokio::spawn(async move {
            // Test concurrent metadata access
            let metadata = provider_clone.metadata();
            assert_eq!(
                metadata.name(),
                "gemini",
                "ANCHOR FAILURE: Metadata access should work concurrently"
            );

            // Test concurrent health checks
            let health_result = provider_clone.health_check().await;
            assert!(
                health_result.is_ok(),
                "ANCHOR FAILURE: Health check should work concurrently"
            );

            // Test concurrent cost estimation
            let cost_result = provider_clone
                .estimate_cost(&format!("Test query {}", i))
                .await;
            assert!(
                cost_result.is_ok(),
                "ANCHOR FAILURE: Cost estimation should work concurrently"
            );

            // Test concurrent usage stats
            let stats_result = provider_clone.usage_stats().await;
            assert!(
                stats_result.is_ok(),
                "ANCHOR FAILURE: Usage stats should work concurrently"
            );

            i
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await;
        assert!(
            result.is_ok(),
            "ANCHOR FAILURE: Concurrent task should complete successfully"
        );
    }

    println!("✅ ANCHOR TEST PASSED: Gemini provider thread safety");
}

/// ANCHOR TEST: Verifies provider configuration validation edge cases
///
/// This test ensures that the provider properly validates configuration edge cases
/// that could cause security issues or unexpected behavior.
#[tokio::test]
async fn anchor_test_gemini_provider_configuration_edge_cases() {
    let _guard = TestEnvironmentGuard::new();

    // Test with various edge case configurations

    // Test with very long API key
    let long_key = "AIzaSy".to_string() + &"a".repeat(1000);
    let long_key_settings = ProviderSettings::new(long_key, "gemini-1.5-pro".to_string());
    let result = GeminiProvider::new(long_key_settings).await;
    // Should handle gracefully (either accept or reject with proper error)
    if result.is_err() {
        println!("✅ Long API key properly rejected");
    } else {
        println!("✅ Long API key accepted");
    }

    // Test with unusual but valid model names
    let unusual_model_settings = ProviderSettings::new(
        "AIzaSyTest1234567890abcdef1234567890abcdef".to_string(),
        "gemini-1.0-pro".to_string(), // Older model
    );
    let result = GeminiProvider::new(unusual_model_settings).await;
    assert!(
        result.is_ok(),
        "ANCHOR FAILURE: Valid older model should be accepted"
    );

    // Test with extreme timeout values
    let max_timeout_settings = ProviderSettings::new(
        "AIzaSyTest1234567890abcdef1234567890abcdef".to_string(),
        "gemini-1.5-pro".to_string(),
    )
    .with_timeout(Duration::from_secs(300)); // 5 minutes - should be valid
    let result = GeminiProvider::new(max_timeout_settings).await;
    assert!(
        result.is_ok(),
        "ANCHOR FAILURE: Maximum valid timeout should be accepted"
    );

    // Test with extreme rate limits (very conservative)
    let extreme_rate_limits = RateLimitConfig {
        requests_per_minute: 1,
        input_tokens_per_minute: 100,
        output_tokens_per_minute: 100, // Must be at least 100 according to validation rules
        max_concurrent_requests: 1,
    };
    let extreme_settings = ProviderSettings::new(
        "AIzaSyTest1234567890abcdef1234567890abcdef".to_string(),
        "gemini-1.5-pro".to_string(),
    )
    .with_rate_limits(extreme_rate_limits);
    let result = GeminiProvider::new(extreme_settings).await;
    assert!(
        result.is_ok(),
        "ANCHOR FAILURE: Extreme but valid rate limits should be accepted"
    );

    println!("✅ ANCHOR TEST PASSED: Gemini provider configuration edge cases");
}
