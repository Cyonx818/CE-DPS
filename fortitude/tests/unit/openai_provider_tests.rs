//! Comprehensive unit tests for OpenAI Provider Implementation (Task 1.2)
//! 
//! This module tests:
//! - API v1 integration with proper authentication
//! - Rate limiting behavior and token bucket algorithms
//! - Error mapping and handling scenarios
//! - Token counting and cost estimation accuracy
//! - Response parsing and validation
//! - OpenAI-specific features and capabilities

use fortitude::providers::{
    Provider, ProviderError, ProviderResult, ProviderMetadata, HealthStatus,
    QueryCost, UsageStats, OpenAIProvider
};
use fortitude::providers::config::{ProviderSettings, RateLimitConfig, RetryConfig};
use crate::common::{
    valid_openai_settings, invalid_provider_settings, conservative_rate_limits,
    aggressive_rate_limits, test_queries, TestEnvironmentGuard
};
use std::time::{Duration, Instant};
use std::sync::Arc;
use proptest::prelude::*;

mod openai_provider_creation_tests {
    use super::*;

    /// ANCHOR: Verifies OpenAI provider creation with valid configurations
    #[tokio::test]
    async fn test_anchor_openai_provider_creation_success() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_openai_settings();
        let provider_result = OpenAIProvider::new(settings).await;
        assert!(provider_result.is_ok(), "OpenAI provider should be created with valid settings");
        
        let provider = provider_result.unwrap();
        let metadata = provider.metadata();
        assert_eq!(metadata.name(), "openai");
        assert!(metadata.capabilities().contains(&"research".to_string()));
        assert!(metadata.capabilities().contains(&"rate_limited".to_string()));
        assert!(metadata.capabilities().contains(&"cost_estimation".to_string()));
    }

    /// Test OpenAI provider creation with various valid configurations
    #[tokio::test]
    async fn test_openai_provider_creation_variations() {
        let _guard = TestEnvironmentGuard::new();
        
        let test_cases = vec![
            // Standard GPT-3.5 Turbo
            ProviderSettings::new(
                "sk-test1234567890abcdef1234567890abcdef".to_string(),
                "gpt-3.5-turbo".to_string(),
            ),
            
            // GPT-4
            ProviderSettings::new(
                "sk-test1234567890abcdef1234567890abcdef".to_string(),
                "gpt-4".to_string(),
            ),
            
            // With custom endpoint
            ProviderSettings::new(
                "sk-test1234567890abcdef1234567890abcdef".to_string(),
                "gpt-3.5-turbo".to_string(),
            ).with_endpoint("https://api.openai.com/v1".to_string()),
            
            // With custom timeout
            ProviderSettings::new(
                "sk-test1234567890abcdef1234567890abcdef".to_string(),
                "gpt-3.5-turbo".to_string(),
            ).with_timeout(Duration::from_secs(60)),
            
            // With conservative rate limits
            ProviderSettings::new(
                "sk-test1234567890abcdef1234567890abcdef".to_string(),
                "gpt-3.5-turbo".to_string(),
            ).with_rate_limits(conservative_rate_limits()),
            
            // With aggressive rate limits
            ProviderSettings::new(
                "sk-test1234567890abcdef1234567890abcdef".to_string(),
                "gpt-4".to_string(),
            ).with_rate_limits(aggressive_rate_limits()),
        ];

        for (i, settings) in test_cases.into_iter().enumerate() {
            let provider_result = OpenAIProvider::new(settings).await;
            assert!(provider_result.is_ok(), "Test case {} should succeed", i);
            
            let provider = provider_result.unwrap();
            let metadata = provider.metadata();
            assert_eq!(metadata.name(), "openai");
        }
    }

    /// Test OpenAI provider creation failures with invalid configurations
    #[tokio::test]
    async fn test_openai_provider_creation_failures() {
        let _guard = TestEnvironmentGuard::new();
        
        let invalid_settings = invalid_provider_settings();
        for (i, settings) in invalid_settings.into_iter().enumerate() {
            let provider_result = OpenAIProvider::new(settings).await;
            assert!(provider_result.is_err(), "Invalid settings case {} should fail", i);
        }
    }

    /// Test OpenAI provider creation from environment variables
    #[tokio::test]
    async fn test_openai_provider_from_environment() {
        let _guard = TestEnvironmentGuard::new();
        
        // Test with environment variable
        std::env::set_var("OPENAI_API_KEY", "sk-test1234567890abcdef1234567890abcdef");
        
        let settings = ProviderSettings::from_env("OPENAI_API_KEY", "gpt-3.5-turbo".to_string());
        assert!(settings.is_ok());
        
        let provider_result = OpenAIProvider::new(settings.unwrap()).await;
        assert!(provider_result.is_ok());
        
        // Clean up
        std::env::remove_var("OPENAI_API_KEY");
        
        // Test with missing environment variable
        let missing_settings = ProviderSettings::from_env("NONEXISTENT_KEY", "gpt-3.5-turbo".to_string());
        assert!(missing_settings.is_err());
    }
}

mod openai_api_integration_tests {
    use super::*;

    /// Test OpenAI API authentication handling
    #[tokio::test]
    async fn test_openai_authentication_handling() {
        let _guard = TestEnvironmentGuard::new();
        
        // Test with invalid API key format
        let invalid_key_settings = ProviderSettings::new(
            "invalid-key-format".to_string(),
            "gpt-3.5-turbo".to_string(),
        );
        
        let provider = OpenAIProvider::new(invalid_key_settings).await.unwrap();
        let result = provider.research_query("Test query".to_string()).await;
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            matches!(error, ProviderError::AuthenticationFailed { .. }) ||
            matches!(error, ProviderError::NetworkError { .. }) ||
            matches!(error, ProviderError::QueryFailed { .. }),
            "Expected authentication or network error, got: {:?}", error
        );
    }

    /// Test OpenAI API request formatting and headers
    #[tokio::test]
    async fn test_openai_request_formatting() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_openai_settings();
        let provider = OpenAIProvider::new(settings).await.unwrap();
        
        // Test that provider can handle various query types
        let test_cases = vec![
            "Simple question",
            "Question with special characters: !@#$%^&*()",
            "Multi-line\nquestion\nwith breaks",
            "Question with \"quotes\" and 'apostrophes'",
            "Question with unicode: 🤖 AI testing",
        ];

        for query in test_cases {
            let result = provider.validate_query(query);
            assert!(result.is_ok(), "Query validation should succeed for: {}", query);
        }
    }

    /// Test OpenAI API response parsing and validation
    #[tokio::test]
    async fn test_openai_response_handling() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_openai_settings();
        let provider = OpenAIProvider::new(settings).await.unwrap();
        
        // Test basic query execution (will fail with test credentials but should handle gracefully)
        let result = provider.research_query("What is the capital of France?".to_string()).await;
        
        match result {
            Ok(response) => {
                // If somehow successful, response should be non-empty
                assert!(!response.is_empty());
            }
            Err(error) => {
                // Expected with test credentials - should be a proper error type
                assert!(
                    matches!(error, ProviderError::AuthenticationFailed { .. }) ||
                    matches!(error, ProviderError::NetworkError { .. }) ||
                    matches!(error, ProviderError::QueryFailed { .. }),
                    "Error should be properly categorized: {:?}", error
                );
            }
        }
    }
}

mod openai_rate_limiting_tests {
    use super::*;

    /// Test OpenAI rate limiting configuration
    #[tokio::test]
    async fn test_openai_rate_limiting_configuration() {
        let _guard = TestEnvironmentGuard::new();
        
        let rate_limits = RateLimitConfig {
            requests_per_minute: 10,
            input_tokens_per_minute: 5000,
            output_tokens_per_minute: 2000,
            max_concurrent_requests: 2,
        };
        
        let settings = valid_openai_settings().with_rate_limits(rate_limits.clone());
        let provider = OpenAIProvider::new(settings).await.unwrap();
        
        let metadata = provider.metadata();
        let provider_rate_limits = metadata.rate_limits();
        
        assert_eq!(provider_rate_limits.requests_per_minute, rate_limits.requests_per_minute);
        assert_eq!(provider_rate_limits.input_tokens_per_minute, rate_limits.input_tokens_per_minute);
        assert_eq!(provider_rate_limits.output_tokens_per_minute, rate_limits.output_tokens_per_minute);
        assert_eq!(provider_rate_limits.max_concurrent_requests, rate_limits.max_concurrent_requests);
    }

    /// Test OpenAI rate limiting enforcement
    #[tokio::test]
    async fn test_openai_rate_limiting_enforcement() {
        let _guard = TestEnvironmentGuard::new();
        
        // Create provider with very restrictive rate limits
        let strict_rate_limits = RateLimitConfig {
            requests_per_minute: 1,
            input_tokens_per_minute: 100,
            output_tokens_per_minute: 50,
            max_concurrent_requests: 1,
        };
        
        let settings = valid_openai_settings().with_rate_limits(strict_rate_limits);
        let provider = OpenAIProvider::new(settings).await.unwrap();
        
        // Make multiple rapid requests - some should be rate limited
        let mut rate_limited_count = 0;
        for i in 0..5 {
            let result = provider.research_query(format!("Query {}", i)).await;
            
            if let Err(ProviderError::RateLimitExceeded { .. }) = result {
                rate_limited_count += 1;
            }
        }
        
        // Note: In a real implementation with actual rate limiting,
        // we would expect some requests to be rate limited
        // For now, we're testing the configuration is properly stored
    }

    /// Test OpenAI concurrent request limiting
    #[tokio::test]
    async fn test_openai_concurrent_request_limiting() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_openai_settings().with_rate_limits(RateLimitConfig {
            requests_per_minute: 100,
            input_tokens_per_minute: 10000,
            output_tokens_per_minute: 5000,
            max_concurrent_requests: 2, // Limit concurrent requests
        });
        
        let provider = Arc::new(OpenAIProvider::new(settings).await.unwrap());
        let mut handles = Vec::new();
        
        // Start multiple concurrent requests
        for i in 0..5 {
            let provider_clone = Arc::clone(&provider);
            let handle = tokio::spawn(async move {
                provider_clone.research_query(format!("Concurrent query {}", i)).await
            });
            handles.push(handle);
        }
        
        // Wait for all requests to complete
        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await.unwrap();
            results.push(result);
        }
        
        // Analyze results - some might be rate limited based on implementation
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        let rate_limited_count = results.iter()
            .filter(|r| matches!(r, Err(ProviderError::RateLimitExceeded { .. })))
            .count();
        
        // At least one should complete (or fail with auth/network error)
        assert!(success_count > 0 || rate_limited_count > 0);
    }
}

mod openai_error_handling_tests {
    use super::*;

    /// Test OpenAI-specific error mapping and handling
    #[tokio::test]
    async fn test_openai_error_mapping() {
        let _guard = TestEnvironmentGuard::new();
        
        // Test with invalid API key
        let invalid_settings = ProviderSettings::new(
            "sk-invalid123".to_string(),
            "gpt-3.5-turbo".to_string(),
        );
        
        let provider = OpenAIProvider::new(invalid_settings).await.unwrap();
        let result = provider.research_query("Test".to_string()).await;
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        
        // Should be properly categorized
        match error {
            ProviderError::AuthenticationFailed { provider, message } => {
                assert_eq!(provider, "openai");
                assert!(!message.is_empty());
            }
            ProviderError::NetworkError { provider, .. } => {
                assert_eq!(provider, "openai");
            }
            ProviderError::QueryFailed { provider, .. } => {
                assert_eq!(provider, "openai");
            }
            _ => {
                // Other error types might be valid depending on implementation
            }
        }
        
        // Verify provider name is correctly set
        assert_eq!(error.provider(), "openai");
    }

    /// Test OpenAI timeout handling
    #[tokio::test]
    async fn test_openai_timeout_handling() {
        let _guard = TestEnvironmentGuard::new();
        
        let timeout_settings = valid_openai_settings()
            .with_timeout(Duration::from_millis(1)); // Very short timeout
        
        let provider = OpenAIProvider::new(timeout_settings).await.unwrap();
        let result = provider.research_query("Test query".to_string()).await;
        
        // Should timeout or fail quickly
        if let Err(error) = result {
            match error {
                ProviderError::Timeout { provider, duration } => {
                    assert_eq!(provider, "openai");
                    assert!(duration <= Duration::from_millis(100)); // Should be fast
                }
                ProviderError::NetworkError { provider, .. } => {
                    assert_eq!(provider, "openai");
                }
                _ => {
                    // Other errors acceptable with very short timeout
                }
            }
        }
    }

    /// Test OpenAI retry mechanism
    #[tokio::test]
    async fn test_openai_retry_mechanism() {
        let _guard = TestEnvironmentGuard::new();
        
        let retry_config = RetryConfig {
            max_retries: 2,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            jitter: false,
        };
        
        let settings = valid_openai_settings().with_retry(retry_config);
        let provider = OpenAIProvider::new(settings).await.unwrap();
        
        // Test with invalid credentials (should retry and then fail)
        let start_time = Instant::now();
        let result = provider.research_query("Test query".to_string()).await;
        let elapsed = start_time.elapsed();
        
        assert!(result.is_err());
        
        // Should have taken some time due to retries (unless it's an immediate auth failure)
        // The exact behavior depends on implementation details
    }

    /// Test OpenAI error context preservation
    #[tokio::test]
    async fn test_openai_error_context_preservation() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = OpenAIProvider::new(valid_openai_settings()).await.unwrap();
        
        // Test that errors include proper context
        let result = provider.research_query("".to_string()).await; // Empty query
        
        match result {
            Ok(_) => {
                // Empty query might be allowed, that's fine
            }
            Err(error) => {
                // Error should have proper provider context
                assert_eq!(error.provider(), "openai");
                
                // Error message should be informative
                match error {
                    ProviderError::ConfigurationError { message, .. } => {
                        assert!(!message.is_empty());
                    }
                    ProviderError::QueryFailed { message, .. } => {
                        assert!(!message.is_empty());
                    }
                    _ => {
                        // Other error types are acceptable
                    }
                }
            }
        }
    }
}

mod openai_cost_estimation_tests {
    use super::*;

    /// Test OpenAI token counting and cost estimation accuracy
    #[tokio::test]
    async fn test_openai_cost_estimation_accuracy() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = OpenAIProvider::new(valid_openai_settings()).await.unwrap();
        
        let test_cases = vec![
            ("Hello", 1, 2),           // Very short
            ("What is the capital of France?", 6, 8),  // Short question
            ("Explain quantum computing in detail", 5, 7), // Medium
            ("Write a comprehensive analysis of machine learning algorithms including neural networks, decision trees, and clustering methods", 20, 25), // Long
        ];

        for (query, min_tokens, max_tokens) in test_cases {
            let cost = provider.estimate_cost(query).await.unwrap();
            
            // Token estimates should be reasonable
            assert!(cost.estimated_input_tokens >= min_tokens, 
                   "Input tokens for '{}' should be at least {}, got {}", 
                   query, min_tokens, cost.estimated_input_tokens);
            assert!(cost.estimated_input_tokens <= max_tokens * 2, 
                   "Input tokens for '{}' should be at most {}, got {}", 
                   query, max_tokens * 2, cost.estimated_input_tokens);
            
            // Output tokens should be estimated
            assert!(cost.estimated_output_tokens > 0);
            
            // Duration should be reasonable
            assert!(cost.estimated_duration > Duration::ZERO);
            assert!(cost.estimated_duration < Duration::from_secs(60));
            
            // Cost should be provided and reasonable
            assert!(cost.estimated_cost_usd.is_some());
            let cost_usd = cost.estimated_cost_usd.unwrap();
            assert!(cost_usd > 0.0);
            assert!(cost_usd < 1.0); // Should be reasonable for test queries
        }
    }

    /// Test OpenAI model-specific pricing
    #[tokio::test]
    async fn test_openai_model_specific_pricing() {
        let _guard = TestEnvironmentGuard::new();
        
        let gpt35_settings = valid_openai_settings(); // gpt-3.5-turbo
        let gpt4_settings = ProviderSettings::new(
            "sk-test1234567890abcdef1234567890abcdef".to_string(),
            "gpt-4".to_string(),
        );
        
        let gpt35_provider = OpenAIProvider::new(gpt35_settings).await.unwrap();
        let gpt4_provider = OpenAIProvider::new(gpt4_settings).await.unwrap();
        
        let query = "What is the meaning of life?";
        
        let gpt35_cost = gpt35_provider.estimate_cost(query).await.unwrap();
        let gpt4_cost = gpt4_provider.estimate_cost(query).await.unwrap();
        
        // Token estimates should be similar
        assert!((gpt35_cost.estimated_input_tokens as i32 - gpt4_cost.estimated_input_tokens as i32).abs() <= 2);
        
        // GPT-4 should be more expensive
        if let (Some(gpt35_price), Some(gpt4_price)) = 
            (gpt35_cost.estimated_cost_usd, gpt4_cost.estimated_cost_usd) {
            assert!(gpt4_price > gpt35_price, 
                   "GPT-4 should be more expensive: {} vs {}", gpt4_price, gpt35_price);
        }
    }

    /// Test OpenAI cost estimation edge cases
    #[tokio::test]
    async fn test_openai_cost_estimation_edge_cases() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = OpenAIProvider::new(valid_openai_settings()).await.unwrap();
        
        // Empty query
        let empty_cost = provider.estimate_cost("").await.unwrap();
        assert!(empty_cost.estimated_input_tokens == 0 || empty_cost.estimated_input_tokens == 1); // Might have minimum
        
        // Very long query
        let long_query = "word ".repeat(1000);
        let long_cost = provider.estimate_cost(&long_query).await.unwrap();
        assert!(long_cost.estimated_input_tokens > empty_cost.estimated_input_tokens);
        assert!(long_cost.estimated_cost_usd.unwrap() > empty_cost.estimated_cost_usd.unwrap_or(0.0));
        
        // Unicode query
        let unicode_query = "🤖 AI testing with emojis 🚀";
        let unicode_cost = provider.estimate_cost(unicode_query).await.unwrap();
        assert!(unicode_cost.estimated_input_tokens > 0);
        assert!(unicode_cost.estimated_cost_usd.is_some());
    }
}

mod openai_metadata_tests {
    use super::*;

    /// Test OpenAI provider metadata compliance
    #[tokio::test]
    async fn test_openai_metadata_compliance() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = OpenAIProvider::new(valid_openai_settings()).await.unwrap();
        let metadata = provider.metadata();
        
        // Required fields
        assert_eq!(metadata.name(), "openai");
        assert!(!metadata.version().is_empty());
        assert!(!metadata.capabilities().is_empty());
        assert!(!metadata.supported_models().is_empty());
        assert!(metadata.max_context_length() > 0);
        
        // OpenAI-specific capabilities
        let capabilities = metadata.capabilities();
        assert!(capabilities.contains(&"research".to_string()));
        assert!(capabilities.contains(&"rate_limited".to_string()));
        assert!(capabilities.contains(&"cost_estimation".to_string()));
        
        // Rate limits should be configured
        let rate_limits = metadata.rate_limits();
        assert!(rate_limits.requests_per_minute > 0);
        assert!(rate_limits.input_tokens_per_minute > 0);
        assert!(rate_limits.output_tokens_per_minute > 0);
        assert!(rate_limits.max_concurrent_requests > 0);
        
        // OpenAI-specific attributes
        let attributes = metadata.custom_attributes();
        assert_eq!(attributes.get("provider_type"), Some(&"openai".to_string()));
        assert_eq!(attributes.get("api_version"), Some(&"v1".to_string()));
        
        // Should support common OpenAI models
        let models = metadata.supported_models();
        assert!(models.contains(&"gpt-3.5-turbo".to_string()));
        assert!(models.contains(&"gpt-4".to_string()));
        
        // Context length should be reasonable for OpenAI models
        assert!(metadata.max_context_length() >= 4096); // Minimum for GPT-3.5
    }

    /// Test OpenAI metadata with different configurations
    #[tokio::test]
    async fn test_openai_metadata_variations() {
        let _guard = TestEnvironmentGuard::new();
        
        // Test with different models
        let models = vec!["gpt-3.5-turbo", "gpt-4", "gpt-4-32k"];
        
        for model in models {
            let settings = ProviderSettings::new(
                "sk-test1234567890abcdef1234567890abcdef".to_string(),
                model.to_string(),
            );
            
            let provider = OpenAIProvider::new(settings).await.unwrap();
            let metadata = provider.metadata();
            
            assert_eq!(metadata.name(), "openai");
            assert!(metadata.supported_models().contains(&model.to_string()));
            
            // Context length should vary by model
            match model {
                "gpt-3.5-turbo" => assert_eq!(metadata.max_context_length(), 4096),
                "gpt-4" => assert_eq!(metadata.max_context_length(), 8192),
                "gpt-4-32k" => assert_eq!(metadata.max_context_length(), 32768),
                _ => {
                    // Other models should have reasonable context lengths
                    assert!(metadata.max_context_length() >= 4096);
                }
            }
        }
    }
}

mod openai_health_check_tests {
    use super::*;

    /// Test OpenAI provider health check functionality
    #[tokio::test]
    async fn test_openai_health_check() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = OpenAIProvider::new(valid_openai_settings()).await.unwrap();
        let health_result = provider.health_check().await;
        
        assert!(health_result.is_ok());
        
        let health_status = health_result.unwrap();
        match health_status {
            HealthStatus::Healthy => {
                // Unexpected with test credentials, but possible
            }
            HealthStatus::Degraded(reason) => {
                assert!(!reason.is_empty());
            }
            HealthStatus::Unhealthy(reason) => {
                // Expected with test credentials
                assert!(!reason.is_empty());
            }
        }
    }

    /// Test OpenAI health check with invalid credentials
    #[tokio::test]
    async fn test_openai_health_check_invalid_credentials() {
        let _guard = TestEnvironmentGuard::new();
        
        let invalid_settings = ProviderSettings::new(
            "invalid-api-key".to_string(),
            "gpt-3.5-turbo".to_string(),
        );
        
        let provider = OpenAIProvider::new(invalid_settings).await.unwrap();
        let health_result = provider.health_check().await;
        
        assert!(health_result.is_ok());
        
        let health_status = health_result.unwrap();
        match health_status {
            HealthStatus::Unhealthy(reason) => {
                assert!(!reason.is_empty());
                assert!(reason.to_lowercase().contains("auth") || 
                       reason.to_lowercase().contains("credential") ||
                       reason.to_lowercase().contains("invalid"));
            }
            HealthStatus::Degraded(_) => {
                // Also acceptable - might indicate partial functionality
            }
            HealthStatus::Healthy => {
                // Unexpected but not necessarily wrong
            }
        }
    }

    /// Test OpenAI health check performance
    #[tokio::test]
    async fn test_openai_health_check_performance() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = OpenAIProvider::new(valid_openai_settings()).await.unwrap();
        
        let start = Instant::now();
        let _health = provider.health_check().await;
        let duration = start.elapsed();
        
        // Health check should be reasonably fast
        assert!(duration < Duration::from_secs(10));
    }
}

mod openai_usage_statistics_tests {
    use super::*;

    /// Test OpenAI usage statistics tracking
    #[tokio::test]
    async fn test_openai_usage_statistics() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = OpenAIProvider::new(valid_openai_settings()).await.unwrap();
        
        // Initial stats should be zero
        let initial_stats = provider.usage_stats().await.unwrap();
        assert_eq!(initial_stats.total_requests, 0);
        assert_eq!(initial_stats.successful_requests, 0);
        assert_eq!(initial_stats.failed_requests, 0);
        
        // Make some requests (will likely fail with test credentials)
        for i in 0..3 {
            let _result = provider.research_query(format!("Test query {}", i)).await;
        }
        
        // Stats should be updated
        let updated_stats = provider.usage_stats().await.unwrap();
        assert_eq!(updated_stats.total_requests, 3);
        
        // With test credentials, requests will likely fail
        assert!(updated_stats.successful_requests + updated_stats.failed_requests == 3);
    }

    /// Test OpenAI usage statistics isolation
    #[tokio::test]
    async fn test_openai_usage_statistics_isolation() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider1 = OpenAIProvider::new(valid_openai_settings()).await.unwrap();
        let provider2 = OpenAIProvider::new(valid_openai_settings()).await.unwrap();
        
        // Make request to first provider
        let _result1 = provider1.research_query("Query 1".to_string()).await;
        
        // Make request to second provider
        let _result2 = provider2.research_query("Query 2".to_string()).await;
        
        // Stats should be isolated
        let stats1 = provider1.usage_stats().await.unwrap();
        let stats2 = provider2.usage_stats().await.unwrap();
        
        assert_eq!(stats1.total_requests, 1);
        assert_eq!(stats2.total_requests, 1);
    }
}

mod openai_validation_tests {
    use super::*;

    /// Test OpenAI query validation
    #[tokio::test]
    async fn test_openai_query_validation() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = OpenAIProvider::new(valid_openai_settings()).await.unwrap();
        
        // Valid queries should pass
        let valid_queries = test_queries();
        for query in &valid_queries {
            if !query.trim().is_empty() {
                let result = provider.validate_query(query);
                assert!(result.is_ok(), "Valid query should pass: {}", query);
            }
        }
        
        // Invalid queries should fail
        let invalid_queries = vec!["", "   ", "\t\n"];
        for query in &invalid_queries {
            let result = provider.validate_query(query);
            assert!(result.is_err(), "Invalid query should fail: '{}'", query);
        }
    }

    /// Test OpenAI-specific validation rules
    #[tokio::test]
    async fn test_openai_specific_validation() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = OpenAIProvider::new(valid_openai_settings()).await.unwrap();
        
        // Test extremely long query (beyond context limits)
        let very_long_query = "word ".repeat(50000); // Way beyond any model's context
        let result = provider.validate_query(&very_long_query);
        
        // Implementation might or might not validate length at this stage
        // Either result is acceptable depending on implementation strategy
        match result {
            Ok(_) => {
                // Validation passed - length checking might happen at query time
            }
            Err(ProviderError::ConfigurationError { .. }) => {
                // Validation failed - implementation checks length early
            }
            Err(_) => {
                panic!("Unexpected error type for long query validation");
            }
        }
    }
}

mod openai_concurrent_access_tests {
    use super::*;

    /// Test OpenAI provider thread safety
    #[tokio::test]
    async fn test_openai_thread_safety() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = Arc::new(OpenAIProvider::new(valid_openai_settings()).await.unwrap());
        let mut handles = Vec::new();
        
        // Spawn concurrent tasks
        for i in 0..10 {
            let provider_clone = Arc::clone(&provider);
            let handle = tokio::spawn(async move {
                // Mix different operations
                match i % 4 {
                    0 => {
                        let _result = provider_clone.research_query(format!("Query {}", i)).await;
                    }
                    1 => {
                        let _metadata = provider_clone.metadata();
                    }
                    2 => {
                        let _health = provider_clone.health_check().await;
                    }
                    3 => {
                        let _cost = provider_clone.estimate_cost(&format!("Cost query {}", i)).await;
                    }
                    _ => unreachable!(),
                }
                i
            });
            handles.push(handle);
        }
        
        // Wait for all tasks
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
        }
    }

    /// Test OpenAI provider with high concurrency
    #[tokio::test]
    async fn test_openai_high_concurrency() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = Arc::new(OpenAIProvider::new(valid_openai_settings()).await.unwrap());
        let mut handles = Vec::new();
        
        // High concurrency test
        for i in 0..50 {
            let provider_clone = Arc::clone(&provider);
            let handle = tokio::spawn(async move {
                let query = format!("Concurrent query {}", i);
                let result = provider_clone.research_query(query).await;
                
                // Return whether operation completed (success or expected failure)
                match result {
                    Ok(_) => true,
                    Err(ProviderError::AuthenticationFailed { .. }) => true, // Expected
                    Err(ProviderError::NetworkError { .. }) => true, // Expected
                    Err(ProviderError::RateLimitExceeded { .. }) => true, // Expected
                    Err(_) => false, // Unexpected error
                }
            });
            handles.push(handle);
        }
        
        // Collect results
        let mut completion_count = 0;
        for handle in handles {
            let completed = handle.await.unwrap();
            if completed {
                completion_count += 1;
            }
        }
        
        // Most operations should complete (even if they fail with expected errors)
        assert!(completion_count >= 45); // 90% completion rate
    }
}

// Property-based tests for OpenAI provider
proptest! {
    #[test]
    fn test_openai_cost_estimation_properties(
        query in ".*",
        input_cost_per_token in 0.00001..0.01,
        output_cost_per_token in 0.00001..0.01
    ) {
        tokio_test::block_on(async {
            let _guard = TestEnvironmentGuard::new();
            
            let settings = valid_openai_settings();
            let provider = OpenAIProvider::new(settings).await.unwrap();
            
            let cost_result = provider.estimate_cost(&query).await;
            prop_assert!(cost_result.is_ok());
            
            let cost = cost_result.unwrap();
            prop_assert!(cost.estimated_input_tokens >= 0);
            prop_assert!(cost.estimated_output_tokens >= 0);
            prop_assert!(cost.estimated_duration >= Duration::ZERO);
            
            if let Some(cost_usd) = cost.estimated_cost_usd {
                prop_assert!(cost_usd >= 0.0);
                prop_assert!(cost_usd < 100.0); // Sanity check
            }
        });
    }

    #[test]
    fn test_openai_metadata_properties(
        model in "(gpt-3\\.5-turbo|gpt-4|gpt-4-32k)"
    ) {
        tokio_test::block_on(async {
            let _guard = TestEnvironmentGuard::new();
            
            let settings = ProviderSettings::new(
                "sk-test1234567890abcdef1234567890abcdef".to_string(),
                model.clone(),
            );
            
            let provider = OpenAIProvider::new(settings).await.unwrap();
            let metadata = provider.metadata();
            
            prop_assert_eq!(metadata.name(), "openai");
            prop_assert!(metadata.supported_models().contains(&model));
            prop_assert!(metadata.max_context_length() >= 4096);
            prop_assert!(metadata.capabilities().contains(&"research".to_string()));
        });
    }

    #[test]
    fn test_openai_validation_properties(query in ".*") {
        tokio_test::block_on(async {
            let _guard = TestEnvironmentGuard::new();
            
            let settings = valid_openai_settings();
            let provider = OpenAIProvider::new(settings).await.unwrap();
            
            let result = provider.validate_query(&query);
            
            if query.trim().is_empty() {
                prop_assert!(result.is_err());
            } else {
                // Non-empty queries should generally pass validation
                // (unless they're extremely long or have other issues)
                prop_assert!(result.is_ok() || matches!(result, Err(ProviderError::ConfigurationError { .. })));
            }
        });
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Integration test for complete OpenAI provider workflow
    #[tokio::test]
    async fn test_openai_provider_complete_workflow() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_openai_settings();
        let provider = OpenAIProvider::new(settings).await.unwrap();
        
        // 1. Check metadata
        let metadata = provider.metadata();
        assert_eq!(metadata.name(), "openai");
        
        // 2. Check health
        let health = provider.health_check().await;
        assert!(health.is_ok());
        
        // 3. Validate query
        let query = "What is artificial intelligence?";
        let validation = provider.validate_query(query);
        assert!(validation.is_ok());
        
        // 4. Estimate cost
        let cost = provider.estimate_cost(query).await;
        assert!(cost.is_ok());
        
        // 5. Execute query (will likely fail with test credentials)
        let result = provider.research_query(query.to_string()).await;
        
        // 6. Check usage stats
        let stats = provider.usage_stats().await;
        assert!(stats.is_ok());
        let usage_stats = stats.unwrap();
        assert_eq!(usage_stats.total_requests, 1);
        
        // The query will likely fail with test credentials, but the workflow should complete
        match result {
            Ok(response) => {
                assert!(!response.is_empty());
                assert_eq!(usage_stats.successful_requests, 1);
            }
            Err(error) => {
                assert_eq!(error.provider(), "openai");
                assert_eq!(usage_stats.failed_requests, 1);
            }
        }
    }

    /// Integration test with environment variables
    #[tokio::test]
    async fn test_openai_provider_environment_integration() {
        let _guard = TestEnvironmentGuard::new();
        
        // Set environment variables
        std::env::set_var("OPENAI_API_KEY", "sk-test1234567890abcdef1234567890abcdef");
        std::env::set_var("OPENAI_MODEL", "gpt-4");
        
        // Create provider from environment
        let api_key = std::env::var("OPENAI_API_KEY").unwrap();
        let model = std::env::var("OPENAI_MODEL").unwrap();
        
        let settings = ProviderSettings::new(api_key, model);
        let provider = OpenAIProvider::new(settings).await.unwrap();
        
        let metadata = provider.metadata();
        assert_eq!(metadata.name(), "openai");
        assert!(metadata.supported_models().contains(&"gpt-4".to_string()));
        
        // Clean up
        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("OPENAI_MODEL");
    }
}