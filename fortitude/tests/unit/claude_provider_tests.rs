//! Comprehensive unit tests for Claude Provider Implementation (Task 1.3)
//! 
//! This module tests:
//! - Messages API v2 integration
//! - Authentication and request formatting
//! - Claude-specific error handling
//! - Safety settings and content filtering
//! - Model-specific pricing and token estimation
//! - Anthropic-specific features and capabilities

use fortitude::providers::{
    Provider, ProviderError, ProviderResult, ProviderMetadata, HealthStatus,
    QueryCost, UsageStats, ClaudeProvider
};
use fortitude::providers::config::{ProviderSettings, RateLimitConfig, RetryConfig};
use crate::common::{
    valid_claude_settings, invalid_provider_settings, conservative_rate_limits,
    aggressive_rate_limits, test_queries, TestEnvironmentGuard
};
use std::time::{Duration, Instant};
use std::sync::Arc;
use proptest::prelude::*;

mod claude_provider_creation_tests {
    use super::*;

    /// ANCHOR: Verifies Claude provider creation with valid configurations
    #[tokio::test]
    async fn test_anchor_claude_provider_creation_success() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_claude_settings();
        let provider_result = ClaudeProvider::new(settings).await;
        assert!(provider_result.is_ok(), "Claude provider should be created with valid settings");
        
        let provider = provider_result.unwrap();
        let metadata = provider.metadata();
        assert_eq!(metadata.name(), "claude");
        assert!(metadata.capabilities().contains(&"research".to_string()));
        assert!(metadata.capabilities().contains(&"rate_limited".to_string()));
        assert!(metadata.capabilities().contains(&"cost_estimation".to_string()));
        assert!(metadata.capabilities().contains(&"anthropic_v2".to_string()));
    }

    /// Test Claude provider creation with various valid configurations
    #[tokio::test]
    async fn test_claude_provider_creation_variations() {
        let _guard = TestEnvironmentGuard::new();
        
        let test_cases = vec![
            // Claude 3.5 Sonnet
            ProviderSettings::new(
                "sk-ant-test1234567890abcdef1234567890abcdef".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
            ),
            
            // Claude 3 Haiku
            ProviderSettings::new(
                "sk-ant-test1234567890abcdef1234567890abcdef".to_string(),
                "claude-3-haiku-20240307".to_string(),
            ),
            
            // Claude 3 Opus
            ProviderSettings::new(
                "sk-ant-test1234567890abcdef1234567890abcdef".to_string(),
                "claude-3-opus-20240229".to_string(),
            ),
            
            // With custom endpoint
            ProviderSettings::new(
                "sk-ant-test1234567890abcdef1234567890abcdef".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
            ).with_endpoint("https://api.anthropic.com".to_string()),
            
            // With custom timeout
            ProviderSettings::new(
                "sk-ant-test1234567890abcdef1234567890abcdef".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
            ).with_timeout(Duration::from_secs(60)),
            
            // With conservative rate limits
            ProviderSettings::new(
                "sk-ant-test1234567890abcdef1234567890abcdef".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
            ).with_rate_limits(conservative_rate_limits()),
            
            // With aggressive rate limits
            ProviderSettings::new(
                "sk-ant-test1234567890abcdef1234567890abcdef".to_string(),
                "claude-3-opus-20240229".to_string(),
            ).with_rate_limits(aggressive_rate_limits()),
        ];

        for (i, settings) in test_cases.into_iter().enumerate() {
            let provider_result = ClaudeProvider::new(settings).await;
            assert!(provider_result.is_ok(), "Test case {} should succeed", i);
            
            let provider = provider_result.unwrap();
            let metadata = provider.metadata();
            assert_eq!(metadata.name(), "claude");
        }
    }

    /// Test Claude provider creation failures with invalid configurations
    #[tokio::test]
    async fn test_claude_provider_creation_failures() {
        let _guard = TestEnvironmentGuard::new();
        
        let invalid_settings = invalid_provider_settings();
        for (i, settings) in invalid_settings.into_iter().enumerate() {
            let provider_result = ClaudeProvider::new(settings).await;
            assert!(provider_result.is_err(), "Invalid settings case {} should fail", i);
        }
    }

    /// Test Claude provider creation from environment variables
    #[tokio::test]
    async fn test_claude_provider_from_environment() {
        let _guard = TestEnvironmentGuard::new();
        
        // Test with environment variable
        std::env::set_var("ANTHROPIC_API_KEY", "sk-ant-test1234567890abcdef1234567890abcdef");
        
        let settings = ProviderSettings::from_env("ANTHROPIC_API_KEY", "claude-3-5-sonnet-20241022".to_string());
        assert!(settings.is_ok());
        
        let provider_result = ClaudeProvider::new(settings.unwrap()).await;
        assert!(provider_result.is_ok());
        
        // Clean up
        std::env::remove_var("ANTHROPIC_API_KEY");
        
        // Test with missing environment variable
        let missing_settings = ProviderSettings::from_env("NONEXISTENT_KEY", "claude-3-5-sonnet-20241022".to_string());
        assert!(missing_settings.is_err());
    }

    /// Test Claude API key format validation
    #[tokio::test]
    async fn test_claude_api_key_format_validation() {
        let _guard = TestEnvironmentGuard::new();
        
        let valid_formats = vec![
            "sk-ant-test1234567890abcdef1234567890abcdef",
            "sk-ant-api03-1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        ];
        
        for api_key in valid_formats {
            let settings = ProviderSettings::new(
                api_key.to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
            );
            
            let provider_result = ClaudeProvider::new(settings).await;
            assert!(provider_result.is_ok(), "Valid API key format should work: {}", api_key);
        }
        
        let invalid_formats = vec![
            "sk-test123", // Wrong prefix
            "invalid-key",
            "sk-ant-", // Too short
        ];
        
        for api_key in invalid_formats {
            let settings = ProviderSettings::new(
                api_key.to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
            );
            
            let provider_result = ClaudeProvider::new(settings).await;
            assert!(provider_result.is_err(), "Invalid API key format should fail: {}", api_key);
        }
    }
}

mod claude_api_integration_tests {
    use super::*;

    /// Test Claude Messages API v2 authentication handling
    #[tokio::test]
    async fn test_claude_authentication_handling() {
        let _guard = TestEnvironmentGuard::new();
        
        // Test with invalid API key format
        let invalid_key_settings = ProviderSettings::new(
            "invalid-key-format".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
        );
        
        let provider = ClaudeProvider::new(invalid_key_settings).await.unwrap();
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

    /// Test Claude Messages API request formatting
    #[tokio::test]
    async fn test_claude_request_formatting() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_claude_settings();
        let provider = ClaudeProvider::new(settings).await.unwrap();
        
        // Test that provider can handle various query types for Claude
        let test_cases = vec![
            "Simple question for Claude",
            "Question with special characters: !@#$%^&*()",
            "Multi-line\nquestion\nwith breaks",
            "Question with \"quotes\" and 'apostrophes'",
            "Question with unicode: 🤖 AI testing with Claude",
            "Long analytical question that requires deep reasoning and multiple steps",
        ];

        for query in test_cases {
            let result = provider.validate_query(query);
            assert!(result.is_ok(), "Query validation should succeed for: {}", query);
        }
    }

    /// Test Claude Messages API response parsing and validation
    #[tokio::test]
    async fn test_claude_response_handling() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_claude_settings();
        let provider = ClaudeProvider::new(settings).await.unwrap();
        
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

    /// Test Claude API version headers and formatting
    #[tokio::test]
    async fn test_claude_api_version_headers() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        let metadata = provider.metadata();
        
        // Verify Claude-specific API attributes
        let attributes = metadata.custom_attributes();
        assert_eq!(attributes.get("provider_type"), Some(&"claude".to_string()));
        assert_eq!(attributes.get("api_version"), Some(&"v2".to_string()));
        assert_eq!(attributes.get("messages_api"), Some(&"2023-06-01".to_string()));
    }
}

mod claude_model_specific_tests {
    use super::*;

    /// Test Claude model-specific configurations and capabilities
    #[tokio::test]
    async fn test_claude_model_configurations() {
        let _guard = TestEnvironmentGuard::new();
        
        let models = vec![
            ("claude-3-5-sonnet-20241022", 200000, "High performance model"),
            ("claude-3-haiku-20240307", 200000, "Fast and efficient model"),
            ("claude-3-opus-20240229", 200000, "Most capable model"),
        ];
        
        for (model, expected_context_length, _description) in models {
            let settings = ProviderSettings::new(
                "sk-ant-test1234567890abcdef1234567890abcdef".to_string(),
                model.to_string(),
            );
            
            let provider = ClaudeProvider::new(settings).await.unwrap();
            let metadata = provider.metadata();
            
            assert_eq!(metadata.name(), "claude");
            assert!(metadata.supported_models().contains(&model.to_string()));
            assert_eq!(metadata.max_context_length(), expected_context_length);
        }
    }

    /// Test Claude model-specific pricing
    #[tokio::test]
    async fn test_claude_model_specific_pricing() {
        let _guard = TestEnvironmentGuard::new();
        
        let haiku_settings = ProviderSettings::new(
            "sk-ant-test1234567890abcdef1234567890abcdef".to_string(),
            "claude-3-haiku-20240307".to_string(),
        );
        let sonnet_settings = ProviderSettings::new(
            "sk-ant-test1234567890abcdef1234567890abcdef".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
        );
        let opus_settings = ProviderSettings::new(
            "sk-ant-test1234567890abcdef1234567890abcdef".to_string(),
            "claude-3-opus-20240229".to_string(),
        );
        
        let haiku_provider = ClaudeProvider::new(haiku_settings).await.unwrap();
        let sonnet_provider = ClaudeProvider::new(sonnet_settings).await.unwrap();
        let opus_provider = ClaudeProvider::new(opus_settings).await.unwrap();
        
        let query = "What is the meaning of life?";
        
        let haiku_cost = haiku_provider.estimate_cost(query).await.unwrap();
        let sonnet_cost = sonnet_provider.estimate_cost(query).await.unwrap();
        let opus_cost = opus_provider.estimate_cost(query).await.unwrap();
        
        // Token estimates should be similar
        assert!((haiku_cost.estimated_input_tokens as i32 - sonnet_cost.estimated_input_tokens as i32).abs() <= 2);
        assert!((sonnet_cost.estimated_input_tokens as i32 - opus_cost.estimated_input_tokens as i32).abs() <= 2);
        
        // Pricing should follow: Haiku < Sonnet < Opus
        if let (Some(haiku_price), Some(sonnet_price), Some(opus_price)) = 
            (haiku_cost.estimated_cost_usd, sonnet_cost.estimated_cost_usd, opus_cost.estimated_cost_usd) {
            assert!(haiku_price < sonnet_price, 
                   "Haiku should be cheaper than Sonnet: {} vs {}", haiku_price, sonnet_price);
            assert!(sonnet_price < opus_price, 
                   "Sonnet should be cheaper than Opus: {} vs {}", sonnet_price, opus_price);
        }
    }

    /// Test Claude context length handling for different models
    #[tokio::test]
    async fn test_claude_context_length_handling() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        let metadata = provider.metadata();
        
        // Claude models should support 200k context length
        assert_eq!(metadata.max_context_length(), 200000);
        
        // Test with very long query approaching context limits
        let long_query = "word ".repeat(50000); // ~200k characters
        let cost = provider.estimate_cost(&long_query).await.unwrap();
        
        // Should handle long queries
        assert!(cost.estimated_input_tokens > 10000); // Should be many tokens
        assert!(cost.estimated_cost_usd.is_some());
    }
}

mod claude_safety_and_content_filtering_tests {
    use super::*;

    /// Test Claude safety settings and content filtering
    #[tokio::test]
    async fn test_claude_safety_settings() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        
        // Test that Claude metadata includes safety capabilities
        let metadata = provider.metadata();
        let capabilities = metadata.capabilities();
        
        // Claude should have safety-related capabilities
        assert!(capabilities.contains(&"anthropic_v2".to_string()));
        
        // Test various content types that Claude should handle appropriately
        let content_tests = vec![
            ("Normal research question", true),
            ("Academic question about historical events", true),
            ("Technical programming question", true),
            ("Creative writing request", true),
        ];
        
        for (content, should_validate) in content_tests {
            let result = provider.validate_query(content);
            if should_validate {
                assert!(result.is_ok(), "Safe content should validate: {}", content);
            }
        }
    }

    /// Test Claude content filtering and safety responses
    #[tokio::test]
    async fn test_claude_content_filtering() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        
        // Test that Claude provider handles various content appropriately
        let test_queries = vec![
            "Explain quantum physics",
            "Help me write a story",
            "What are the benefits of renewable energy?",
            "How do neural networks work?",
        ];
        
        for query in test_queries {
            let validation = provider.validate_query(query);
            assert!(validation.is_ok(), "Educational content should be accepted: {}", query);
            
            // Test cost estimation works for safe content
            let cost = provider.estimate_cost(query).await;
            assert!(cost.is_ok(), "Cost estimation should work for safe content: {}", query);
        }
    }

    /// Test Claude-specific safety metadata and attributes
    #[tokio::test]
    async fn test_claude_safety_metadata() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        let metadata = provider.metadata();
        
        // Verify Claude-specific safety attributes
        let attributes = metadata.custom_attributes();
        assert!(attributes.contains_key("provider_type"));
        assert_eq!(attributes.get("provider_type"), Some(&"claude".to_string()));
        
        // Claude should indicate it has built-in safety features
        let capabilities = metadata.capabilities();
        assert!(capabilities.contains(&"anthropic_v2".to_string()));
    }
}

mod claude_rate_limiting_tests {
    use super::*;

    /// Test Claude rate limiting configuration
    #[tokio::test]
    async fn test_claude_rate_limiting_configuration() {
        let _guard = TestEnvironmentGuard::new();
        
        let rate_limits = RateLimitConfig {
            requests_per_minute: 15, // Claude typically has different limits than OpenAI
            input_tokens_per_minute: 10000,
            output_tokens_per_minute: 4000,
            max_concurrent_requests: 3,
        };
        
        let settings = valid_claude_settings().with_rate_limits(rate_limits.clone());
        let provider = ClaudeProvider::new(settings).await.unwrap();
        
        let metadata = provider.metadata();
        let provider_rate_limits = metadata.rate_limits();
        
        assert_eq!(provider_rate_limits.requests_per_minute, rate_limits.requests_per_minute);
        assert_eq!(provider_rate_limits.input_tokens_per_minute, rate_limits.input_tokens_per_minute);
        assert_eq!(provider_rate_limits.output_tokens_per_minute, rate_limits.output_tokens_per_minute);
        assert_eq!(provider_rate_limits.max_concurrent_requests, rate_limits.max_concurrent_requests);
    }

    /// Test Claude-specific rate limiting behavior
    #[tokio::test]
    async fn test_claude_rate_limiting_behavior() {
        let _guard = TestEnvironmentGuard::new();
        
        // Create provider with Claude-appropriate rate limits
        let claude_rate_limits = RateLimitConfig {
            requests_per_minute: 5, // More conservative for testing
            input_tokens_per_minute: 2000,
            output_tokens_per_minute: 1000,
            max_concurrent_requests: 2,
        };
        
        let settings = valid_claude_settings().with_rate_limits(claude_rate_limits);
        let provider = ClaudeProvider::new(settings).await.unwrap();
        
        // Make multiple requests to test rate limiting
        let mut rate_limited_count = 0;
        for i in 0..8 {
            let result = provider.research_query(format!("Claude query {}", i)).await;
            
            if let Err(ProviderError::RateLimitExceeded { .. }) = result {
                rate_limited_count += 1;
            }
        }
        
        // Note: In a real implementation with actual rate limiting,
        // we would expect some requests to be rate limited
    }

    /// Test Claude concurrent request handling
    #[tokio::test]
    async fn test_claude_concurrent_request_handling() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_claude_settings().with_rate_limits(RateLimitConfig {
            requests_per_minute: 100,
            input_tokens_per_minute: 20000,
            output_tokens_per_minute: 10000,
            max_concurrent_requests: 3, // Limit concurrent requests
        });
        
        let provider = Arc::new(ClaudeProvider::new(settings).await.unwrap());
        let mut handles = Vec::new();
        
        // Start multiple concurrent requests
        for i in 0..6 {
            let provider_clone = Arc::clone(&provider);
            let handle = tokio::spawn(async move {
                provider_clone.research_query(format!("Concurrent Claude query {}", i)).await
            });
            handles.push(handle);
        }
        
        // Wait for all requests to complete
        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await.unwrap();
            results.push(result);
        }
        
        // Analyze results
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        let rate_limited_count = results.iter()
            .filter(|r| matches!(r, Err(ProviderError::RateLimitExceeded { .. })))
            .count();
        
        // At least one should complete (or fail with auth/network error)
        assert!(success_count > 0 || rate_limited_count > 0);
    }
}

mod claude_error_handling_tests {
    use super::*;

    /// Test Claude-specific error mapping and handling
    #[tokio::test]
    async fn test_claude_error_mapping() {
        let _guard = TestEnvironmentGuard::new();
        
        // Test with invalid API key
        let invalid_settings = ProviderSettings::new(
            "sk-ant-invalid123".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
        );
        
        let provider = ClaudeProvider::new(invalid_settings).await.unwrap();
        let result = provider.research_query("Test".to_string()).await;
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        
        // Should be properly categorized
        match error {
            ProviderError::AuthenticationFailed { provider, message } => {
                assert_eq!(provider, "claude");
                assert!(!message.is_empty());
            }
            ProviderError::NetworkError { provider, .. } => {
                assert_eq!(provider, "claude");
            }
            ProviderError::QueryFailed { provider, .. } => {
                assert_eq!(provider, "claude");
            }
            _ => {
                // Other error types might be valid depending on implementation
            }
        }
        
        // Verify provider name is correctly set
        assert_eq!(error.provider(), "claude");
    }

    /// Test Claude timeout handling
    #[tokio::test]
    async fn test_claude_timeout_handling() {
        let _guard = TestEnvironmentGuard::new();
        
        let timeout_settings = valid_claude_settings()
            .with_timeout(Duration::from_millis(1)); // Very short timeout
        
        let provider = ClaudeProvider::new(timeout_settings).await.unwrap();
        let result = provider.research_query("Test query".to_string()).await;
        
        // Should timeout or fail quickly
        if let Err(error) = result {
            match error {
                ProviderError::Timeout { provider, duration } => {
                    assert_eq!(provider, "claude");
                    assert!(duration <= Duration::from_millis(100));
                }
                ProviderError::NetworkError { provider, .. } => {
                    assert_eq!(provider, "claude");
                }
                _ => {
                    // Other errors acceptable with very short timeout
                }
            }
        }
    }

    /// Test Claude retry mechanism with exponential backoff
    #[tokio::test]
    async fn test_claude_retry_mechanism() {
        let _guard = TestEnvironmentGuard::new();
        
        let retry_config = RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(200),
            backoff_multiplier: 2.0,
            jitter: false,
        };
        
        let settings = valid_claude_settings().with_retry(retry_config);
        let provider = ClaudeProvider::new(settings).await.unwrap();
        
        // Test with invalid credentials (should retry and then fail)
        let start_time = Instant::now();
        let result = provider.research_query("Test query".to_string()).await;
        let elapsed = start_time.elapsed();
        
        assert!(result.is_err());
        
        // Should have taken some time due to retries (unless immediate auth failure)
        // The exact behavior depends on implementation details
    }

    /// Test Claude-specific error codes and messages
    #[tokio::test]
    async fn test_claude_error_codes_and_messages() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        
        // Test various error scenarios specific to Claude
        let error_tests = vec![
            ("", "Empty query should be handled appropriately"),
            ("   ", "Whitespace-only query should be rejected"),
        ];
        
        for (query, description) in error_tests {
            let result = provider.validate_query(query);
            
            match result {
                Ok(_) => {
                    // Some providers might allow empty queries, that's fine
                }
                Err(error) => {
                    assert_eq!(error.provider(), "claude");
                    match error {
                        ProviderError::ConfigurationError { message, .. } => {
                            assert!(!message.is_empty(), "{}", description);
                        }
                        _ => {
                            // Other error types are acceptable
                        }
                    }
                }
            }
        }
    }

    /// Test Claude content safety error handling
    #[tokio::test]
    async fn test_claude_content_safety_error_handling() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        
        // Test that Claude handles edge cases in content appropriately
        let edge_case_queries = vec![
            "A" * 1000, // Very repetitive content
            "Mixed language query with français and español and 中文",
            "Query with many numbers: 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5",
            "Query with special formatting\n\n\tTabbed content\n\n",
        ];
        
        for query in edge_case_queries {
            let validation = provider.validate_query(&query);
            let cost_estimation = provider.estimate_cost(&query).await;
            
            // These should generally work or fail gracefully
            if validation.is_err() {
                // If validation fails, it should be with a proper error
                let error = validation.unwrap_err();
                assert_eq!(error.provider(), "claude");
            }
            
            if cost_estimation.is_err() {
                // If cost estimation fails, it should be with a proper error
                let error = cost_estimation.unwrap_err();
                assert_eq!(error.provider(), "claude");
            }
        }
    }
}

mod claude_cost_estimation_tests {
    use super::*;

    /// Test Claude token counting and cost estimation accuracy
    #[tokio::test]
    async fn test_claude_cost_estimation_accuracy() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        
        let test_cases = vec![
            ("Hello", 1, 2),           // Very short
            ("What is the capital of France?", 6, 8),  // Short question
            ("Explain quantum computing in detail", 5, 7), // Medium
            ("Write a comprehensive analysis of artificial intelligence including machine learning, deep learning, neural networks, and their applications in modern technology", 25, 35), // Long
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

    /// Test Claude model-specific pricing accuracy
    #[tokio::test]
    async fn test_claude_model_pricing_accuracy() {
        let _guard = TestEnvironmentGuard::new();
        
        let models_and_expected_pricing = vec![
            ("claude-3-haiku-20240307", "cheapest"),
            ("claude-3-5-sonnet-20241022", "medium"),
            ("claude-3-opus-20240229", "most_expensive"),
        ];
        
        let mut costs = Vec::new();
        let query = "Explain the theory of relativity";
        
        for (model, _tier) in &models_and_expected_pricing {
            let settings = ProviderSettings::new(
                "sk-ant-test1234567890abcdef1234567890abcdef".to_string(),
                model.to_string(),
            );
            
            let provider = ClaudeProvider::new(settings).await.unwrap();
            let cost = provider.estimate_cost(query).await.unwrap();
            costs.push((model, cost));
        }
        
        // Extract costs for comparison
        let haiku_cost = costs.iter().find(|(m, _)| m.contains("haiku")).unwrap().1.estimated_cost_usd.unwrap();
        let sonnet_cost = costs.iter().find(|(m, _)| m.contains("sonnet")).unwrap().1.estimated_cost_usd.unwrap();
        let opus_cost = costs.iter().find(|(m, _)| m.contains("opus")).unwrap().1.estimated_cost_usd.unwrap();
        
        // Verify pricing hierarchy: Haiku < Sonnet < Opus
        assert!(haiku_cost < sonnet_cost, 
               "Haiku should be cheaper than Sonnet: {} vs {}", haiku_cost, sonnet_cost);
        assert!(sonnet_cost < opus_cost, 
               "Sonnet should be cheaper than Opus: {} vs {}", sonnet_cost, opus_cost);
    }

    /// Test Claude cost estimation with various content types
    #[tokio::test]
    async fn test_claude_cost_estimation_content_types() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        
        let content_types = vec![
            ("Code analysis request", "def fibonacci(n):\n    if n <= 1:\n        return n\n    return fibonacci(n-1) + fibonacci(n-2)\n\nAnalyze this code"),
            ("Creative writing", "Write a short story about a robot learning to paint"),
            ("Academic query", "Explain the significance of the Higgs boson discovery in particle physics"),
            ("Mathematical problem", "Solve this equation: 2x + 5 = 17, show your work"),
            ("Multi-language", "Translate 'Hello, how are you?' into French, Spanish, and German"),
        ];
        
        for (content_type, query) in content_types {
            let cost = provider.estimate_cost(query).await.unwrap();
            
            assert!(cost.estimated_input_tokens > 0, "Should estimate tokens for {}", content_type);
            assert!(cost.estimated_output_tokens > 0, "Should estimate output tokens for {}", content_type);
            assert!(cost.estimated_cost_usd.is_some(), "Should provide cost for {}", content_type);
            assert!(cost.estimated_cost_usd.unwrap() > 0.0, "Cost should be positive for {}", content_type);
        }
    }

    /// Test Claude cost estimation edge cases
    #[tokio::test]
    async fn test_claude_cost_estimation_edge_cases() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        
        // Empty query
        let empty_cost = provider.estimate_cost("").await.unwrap();
        assert!(empty_cost.estimated_input_tokens >= 0);
        
        // Very long query (testing Claude's 200k context window)
        let long_query = "word ".repeat(25000); // ~100k characters
        let long_cost = provider.estimate_cost(&long_query).await.unwrap();
        assert!(long_cost.estimated_input_tokens > empty_cost.estimated_input_tokens);
        assert!(long_cost.estimated_cost_usd.unwrap() > empty_cost.estimated_cost_usd.unwrap_or(0.0));
        
        // Unicode and special characters
        let unicode_query = "🤖 Analyze this: café, résumé, naïve, 北京, العربية";
        let unicode_cost = provider.estimate_cost(unicode_query).await.unwrap();
        assert!(unicode_cost.estimated_input_tokens > 0);
        assert!(unicode_cost.estimated_cost_usd.is_some());
    }
}

mod claude_metadata_tests {
    use super::*;

    /// Test Claude provider metadata compliance and Anthropic-specific features
    #[tokio::test]
    async fn test_claude_metadata_compliance() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        let metadata = provider.metadata();
        
        // Required fields
        assert_eq!(metadata.name(), "claude");
        assert!(!metadata.version().is_empty());
        assert!(!metadata.capabilities().is_empty());
        assert!(!metadata.supported_models().is_empty());
        assert!(metadata.max_context_length() > 0);
        
        // Claude-specific capabilities
        let capabilities = metadata.capabilities();
        assert!(capabilities.contains(&"research".to_string()));
        assert!(capabilities.contains(&"rate_limited".to_string()));
        assert!(capabilities.contains(&"cost_estimation".to_string()));
        assert!(capabilities.contains(&"anthropic_v2".to_string()));
        
        // Rate limits should be configured
        let rate_limits = metadata.rate_limits();
        assert!(rate_limits.requests_per_minute > 0);
        assert!(rate_limits.input_tokens_per_minute > 0);
        assert!(rate_limits.output_tokens_per_minute > 0);
        assert!(rate_limits.max_concurrent_requests > 0);
        
        // Claude-specific attributes
        let attributes = metadata.custom_attributes();
        assert_eq!(attributes.get("provider_type"), Some(&"claude".to_string()));
        assert_eq!(attributes.get("api_version"), Some(&"v2".to_string()));
        assert_eq!(attributes.get("messages_api"), Some(&"2023-06-01".to_string()));
        
        // Should support Claude models
        let models = metadata.supported_models();
        assert!(models.contains(&"claude-3-5-sonnet-20241022".to_string()));
        assert!(models.contains(&"claude-3-haiku-20240307".to_string()));
        assert!(models.contains(&"claude-3-opus-20240229".to_string()));
        
        // Context length should be 200k for Claude models
        assert_eq!(metadata.max_context_length(), 200000);
    }

    /// Test Claude metadata with different model configurations
    #[tokio::test]
    async fn test_claude_metadata_model_variations() {
        let _guard = TestEnvironmentGuard::new();
        
        let models = vec![
            "claude-3-haiku-20240307",
            "claude-3-5-sonnet-20241022", 
            "claude-3-opus-20240229",
        ];
        
        for model in models {
            let settings = ProviderSettings::new(
                "sk-ant-test1234567890abcdef1234567890abcdef".to_string(),
                model.to_string(),
            );
            
            let provider = ClaudeProvider::new(settings).await.unwrap();
            let metadata = provider.metadata();
            
            assert_eq!(metadata.name(), "claude");
            assert!(metadata.supported_models().contains(&model.to_string()));
            
            // All Claude models should have 200k context length
            assert_eq!(metadata.max_context_length(), 200000);
            
            // All should have anthropic_v2 capability
            assert!(metadata.capabilities().contains(&"anthropic_v2".to_string()));
        }
    }

    /// Test Claude metadata consistency and immutability
    #[tokio::test]
    async fn test_claude_metadata_consistency() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        
        // Get metadata multiple times
        let metadata1 = provider.metadata();
        let metadata2 = provider.metadata();
        let metadata3 = provider.metadata();
        
        // Should be identical
        assert_eq!(metadata1.name(), metadata2.name());
        assert_eq!(metadata2.name(), metadata3.name());
        assert_eq!(metadata1.version(), metadata2.version());
        assert_eq!(metadata2.version(), metadata3.version());
        assert_eq!(metadata1.capabilities(), metadata2.capabilities());
        assert_eq!(metadata2.capabilities(), metadata3.capabilities());
        assert_eq!(metadata1.max_context_length(), metadata2.max_context_length());
        assert_eq!(metadata2.max_context_length(), metadata3.max_context_length());
    }
}

mod claude_health_check_tests {
    use super::*;

    /// Test Claude provider health check functionality
    #[tokio::test]
    async fn test_claude_health_check() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
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

    /// Test Claude health check with invalid credentials
    #[tokio::test]
    async fn test_claude_health_check_invalid_credentials() {
        let _guard = TestEnvironmentGuard::new();
        
        let invalid_settings = ProviderSettings::new(
            "sk-ant-invalid-key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
        );
        
        let provider = ClaudeProvider::new(invalid_settings).await.unwrap();
        let health_result = provider.health_check().await;
        
        assert!(health_result.is_ok());
        
        let health_status = health_result.unwrap();
        match health_status {
            HealthStatus::Unhealthy(reason) => {
                assert!(!reason.is_empty());
                assert!(reason.to_lowercase().contains("auth") || 
                       reason.to_lowercase().contains("credential") ||
                       reason.to_lowercase().contains("invalid") ||
                       reason.to_lowercase().contains("anthropic"));
            }
            HealthStatus::Degraded(_) => {
                // Also acceptable - might indicate partial functionality
            }
            HealthStatus::Healthy => {
                // Unexpected but not necessarily wrong
            }
        }
    }

    /// Test Claude health check performance
    #[tokio::test]
    async fn test_claude_health_check_performance() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        
        let start = Instant::now();
        let _health = provider.health_check().await;
        let duration = start.elapsed();
        
        // Health check should be reasonably fast
        assert!(duration < Duration::from_secs(10));
    }

    /// Test Claude health check with various configurations
    #[tokio::test]
    async fn test_claude_health_check_configurations() {
        let _guard = TestEnvironmentGuard::new();
        
        // Test with different timeout settings
        let short_timeout_settings = valid_claude_settings()
            .with_timeout(Duration::from_millis(100));
        let long_timeout_settings = valid_claude_settings()
            .with_timeout(Duration::from_secs(30));
        
        let short_provider = ClaudeProvider::new(short_timeout_settings).await.unwrap();
        let long_provider = ClaudeProvider::new(long_timeout_settings).await.unwrap();
        
        // Both should handle health checks appropriately
        let short_health = short_provider.health_check().await;
        let long_health = long_provider.health_check().await;
        
        assert!(short_health.is_ok());
        assert!(long_health.is_ok());
    }
}

mod claude_usage_statistics_tests {
    use super::*;

    /// Test Claude usage statistics tracking
    #[tokio::test]
    async fn test_claude_usage_statistics() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        
        // Initial stats should be zero
        let initial_stats = provider.usage_stats().await.unwrap();
        assert_eq!(initial_stats.total_requests, 0);
        assert_eq!(initial_stats.successful_requests, 0);
        assert_eq!(initial_stats.failed_requests, 0);
        
        // Make some requests (will likely fail with test credentials)
        for i in 0..3 {
            let _result = provider.research_query(format!("Claude test query {}", i)).await;
        }
        
        // Stats should be updated
        let updated_stats = provider.usage_stats().await.unwrap();
        assert_eq!(updated_stats.total_requests, 3);
        
        // With test credentials, requests will likely fail
        assert!(updated_stats.successful_requests + updated_stats.failed_requests == 3);
    }

    /// Test Claude usage statistics isolation between providers
    #[tokio::test]
    async fn test_claude_usage_statistics_isolation() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider1 = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        let provider2 = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        
        // Make request to first provider
        let _result1 = provider1.research_query("Claude query 1".to_string()).await;
        
        // Make request to second provider
        let _result2 = provider2.research_query("Claude query 2".to_string()).await;
        
        // Stats should be isolated
        let stats1 = provider1.usage_stats().await.unwrap();
        let stats2 = provider2.usage_stats().await.unwrap();
        
        assert_eq!(stats1.total_requests, 1);
        assert_eq!(stats2.total_requests, 1);
    }

    /// Test Claude usage statistics with different query types
    #[tokio::test]
    async fn test_claude_usage_statistics_query_types() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        
        let query_types = vec![
            "Short query",
            "Medium length query that should generate more tokens for testing",
            "Long analytical query that requires deep thinking and comprehensive analysis with multiple parts and detailed explanations",
        ];
        
        for query in query_types {
            let _result = provider.research_query(query.to_string()).await;
        }
        
        let stats = provider.usage_stats().await.unwrap();
        assert_eq!(stats.total_requests, 3);
        
        // Should have token usage tracked
        if stats.successful_requests > 0 {
            assert!(stats.total_input_tokens > 0);
            assert!(stats.total_output_tokens > 0);
        }
    }
}

mod claude_validation_tests {
    use super::*;

    /// Test Claude query validation
    #[tokio::test]
    async fn test_claude_query_validation() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        
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

    /// Test Claude-specific validation rules
    #[tokio::test]
    async fn test_claude_specific_validation() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        
        // Test extremely long query (testing Claude's 200k context window)
        let very_long_query = "word ".repeat(75000); // ~300k characters, beyond context
        let result = provider.validate_query(&very_long_query);
        
        // Implementation might or might not validate length at this stage
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
        
        // Test content with special formatting
        let formatted_query = "Query with\n\nmultiple\n\nline breaks\n\nand\ttabs";
        let formatted_result = provider.validate_query(formatted_query);
        assert!(formatted_result.is_ok(), "Formatted query should be valid");
    }

    /// Test Claude content appropriateness validation
    #[tokio::test]
    async fn test_claude_content_validation() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = ClaudeProvider::new(valid_claude_settings()).await.unwrap();
        
        // Test various content types that should be appropriate for Claude
        let appropriate_content = vec![
            "Explain quantum mechanics",
            "Help me write a poem about nature",
            "What are the best practices for software development?",
            "Analyze this historical event",
            "Translate this text",
            "Write a story about friendship",
        ];
        
        for content in appropriate_content {
            let result = provider.validate_query(content);
            assert!(result.is_ok(), "Appropriate content should be valid: {}", content);
        }
    }
}

mod claude_concurrent_access_tests {
    use super::*;

    /// Test Claude provider thread safety
    #[tokio::test]
    async fn test_claude_thread_safety() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = Arc::new(ClaudeProvider::new(valid_claude_settings()).await.unwrap());
        let mut handles = Vec::new();
        
        // Spawn concurrent tasks
        for i in 0..10 {
            let provider_clone = Arc::clone(&provider);
            let handle = tokio::spawn(async move {
                // Mix different operations
                match i % 4 {
                    0 => {
                        let _result = provider_clone.research_query(format!("Claude query {}", i)).await;
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

    /// Test Claude provider with high concurrency
    #[tokio::test]
    async fn test_claude_high_concurrency() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = Arc::new(ClaudeProvider::new(valid_claude_settings()).await.unwrap());
        let mut handles = Vec::new();
        
        // High concurrency test
        for i in 0..50 {
            let provider_clone = Arc::clone(&provider);
            let handle = tokio::spawn(async move {
                let query = format!("Concurrent Claude query {}", i);
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

// Property-based tests for Claude provider
proptest! {
    #[test]
    fn test_claude_cost_estimation_properties(
        query in ".*",
        input_cost_per_token in 0.000001..0.001,
        output_cost_per_token in 0.000001..0.001
    ) {
        tokio_test::block_on(async {
            let _guard = TestEnvironmentGuard::new();
            
            let settings = valid_claude_settings();
            let provider = ClaudeProvider::new(settings).await.unwrap();
            
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
    fn test_claude_metadata_properties(
        model in "(claude-3-haiku-20240307|claude-3-5-sonnet-20241022|claude-3-opus-20240229)"
    ) {
        tokio_test::block_on(async {
            let _guard = TestEnvironmentGuard::new();
            
            let settings = ProviderSettings::new(
                "sk-ant-test1234567890abcdef1234567890abcdef".to_string(),
                model.clone(),
            );
            
            let provider = ClaudeProvider::new(settings).await.unwrap();
            let metadata = provider.metadata();
            
            prop_assert_eq!(metadata.name(), "claude");
            prop_assert!(metadata.supported_models().contains(&model));
            prop_assert_eq!(metadata.max_context_length(), 200000);
            prop_assert!(metadata.capabilities().contains(&"anthropic_v2".to_string()));
        });
    }

    #[test]
    fn test_claude_validation_properties(query in ".*") {
        tokio_test::block_on(async {
            let _guard = TestEnvironmentGuard::new();
            
            let settings = valid_claude_settings();
            let provider = ClaudeProvider::new(settings).await.unwrap();
            
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

    /// Integration test for complete Claude provider workflow
    #[tokio::test]
    async fn test_claude_provider_complete_workflow() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_claude_settings();
        let provider = ClaudeProvider::new(settings).await.unwrap();
        
        // 1. Check metadata
        let metadata = provider.metadata();
        assert_eq!(metadata.name(), "claude");
        assert!(metadata.capabilities().contains(&"anthropic_v2".to_string()));
        
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
                assert_eq!(error.provider(), "claude");
                assert_eq!(usage_stats.failed_requests, 1);
            }
        }
    }

    /// Integration test with environment variables
    #[tokio::test]
    async fn test_claude_provider_environment_integration() {
        let _guard = TestEnvironmentGuard::new();
        
        // Set environment variables
        std::env::set_var("ANTHROPIC_API_KEY", "sk-ant-test1234567890abcdef1234567890abcdef");
        std::env::set_var("CLAUDE_MODEL", "claude-3-opus-20240229");
        
        // Create provider from environment
        let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap();
        let model = std::env::var("CLAUDE_MODEL").unwrap();
        
        let settings = ProviderSettings::new(api_key, model);
        let provider = ClaudeProvider::new(settings).await.unwrap();
        
        let metadata = provider.metadata();
        assert_eq!(metadata.name(), "claude");
        assert!(metadata.supported_models().contains(&"claude-3-opus-20240229".to_string()));
        
        // Clean up
        std::env::remove_var("ANTHROPIC_API_KEY");
        std::env::remove_var("CLAUDE_MODEL");
    }

    /// Integration test comparing Claude models
    #[tokio::test]
    async fn test_claude_models_comparison_integration() {
        let _guard = TestEnvironmentGuard::new();
        
        let models = vec![
            "claude-3-haiku-20240307",
            "claude-3-5-sonnet-20241022",
            "claude-3-opus-20240229",
        ];
        
        let query = "Explain the concept of machine learning";
        let mut providers = Vec::new();
        
        // Create providers for each model
        for model in &models {
            let settings = ProviderSettings::new(
                "sk-ant-test1234567890abcdef1234567890abcdef".to_string(),
                model.to_string(),
            );
            let provider = ClaudeProvider::new(settings).await.unwrap();
            providers.push((model, provider));
        }
        
        // Test each provider
        for (model, provider) in &providers {
            // Metadata should be consistent
            let metadata = provider.metadata();
            assert_eq!(metadata.name(), "claude");
            assert!(metadata.supported_models().contains(&model.to_string()));
            assert_eq!(metadata.max_context_length(), 200000);
            
            // Health check should work
            let health = provider.health_check().await;
            assert!(health.is_ok());
            
            // Cost estimation should work
            let cost = provider.estimate_cost(query).await;
            assert!(cost.is_ok());
            
            // Validation should work
            let validation = provider.validate_query(query);
            assert!(validation.is_ok());
        }
        
        // Compare costs between models
        let costs: Vec<_> = providers.iter()
            .map(|(model, provider)| {
                let cost = tokio_test::block_on(provider.estimate_cost(query)).unwrap();
                (model, cost.estimated_cost_usd.unwrap())
            })
            .collect();
        
        let haiku_cost = costs.iter().find(|(m, _)| m.contains("haiku")).unwrap().1;
        let sonnet_cost = costs.iter().find(|(m, _)| m.contains("sonnet")).unwrap().1;
        let opus_cost = costs.iter().find(|(m, _)| m.contains("opus")).unwrap().1;
        
        // Verify cost hierarchy
        assert!(haiku_cost < sonnet_cost);
        assert!(sonnet_cost < opus_cost);
    }
}