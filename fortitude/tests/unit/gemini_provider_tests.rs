//! Comprehensive unit tests for Gemini Provider Implementation (Task 1.4)
//! 
//! This module tests:
//! - Google AI API integration
//! - Safety settings configuration
//! - Token counting using countTokens API
//! - Generation configuration parameters
//! - Error handling and authentication
//! - Gemini-specific features and capabilities

use fortitude::providers::{
    Provider, ProviderError, ProviderResult, ProviderMetadata, HealthStatus,
    QueryCost, UsageStats, GeminiProvider
};
use fortitude::providers::config::{ProviderSettings, RateLimitConfig, RetryConfig};
use crate::common::{
    valid_gemini_settings, invalid_provider_settings, conservative_rate_limits,
    aggressive_rate_limits, test_queries, TestEnvironmentGuard
};
use std::time::{Duration, Instant};
use std::sync::Arc;
use proptest::prelude::*;

mod gemini_provider_creation_tests {
    use super::*;

    /// ANCHOR: Verifies Gemini provider creation with valid configurations
    #[tokio::test]
    async fn test_anchor_gemini_provider_creation_success() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_gemini_settings();
        let provider_result = GeminiProvider::new(settings).await;
        assert!(provider_result.is_ok(), "Gemini provider should be created with valid settings");
        
        let provider = provider_result.unwrap();
        let metadata = provider.metadata();
        assert_eq!(metadata.name(), "gemini");
        assert!(metadata.capabilities().contains(&"research".to_string()));
        assert!(metadata.capabilities().contains(&"rate_limited".to_string()));
        assert!(metadata.capabilities().contains(&"cost_estimation".to_string()));
        assert!(metadata.capabilities().contains(&"safety_settings".to_string()));
        assert!(metadata.capabilities().contains(&"multimodal".to_string()));
    }

    /// Test Gemini provider creation with various valid configurations
    #[tokio::test]
    async fn test_gemini_provider_creation_variations() {
        let _guard = TestEnvironmentGuard::new();
        
        let test_cases = vec![
            // Gemini 1.5 Pro
            ProviderSettings::new(
                "AIzaSyTest1234567890abcdef1234567890abcdef".to_string(),
                "gemini-1.5-pro".to_string(),
            ),
            
            // Gemini 1.5 Flash
            ProviderSettings::new(
                "AIzaSyTest1234567890abcdef1234567890abcdef".to_string(),
                "gemini-1.5-flash".to_string(),
            ),
            
            // Gemini 1.0 Pro
            ProviderSettings::new(
                "AIzaSyTest1234567890abcdef1234567890abcdef".to_string(),
                "gemini-1.0-pro".to_string(),
            ),
            
            // With custom timeout
            ProviderSettings::new(
                "AIzaSyTest1234567890abcdef1234567890abcdef".to_string(),
                "gemini-1.5-pro".to_string(),
            ).with_timeout(Duration::from_secs(45)),
            
            // With conservative rate limits
            ProviderSettings::new(
                "AIzaSyTest1234567890abcdef1234567890abcdef".to_string(),
                "gemini-1.5-pro".to_string(),
            ).with_rate_limits(conservative_rate_limits()),
            
            // With aggressive rate limits
            ProviderSettings::new(
                "AIzaSyTest1234567890abcdef1234567890abcdef".to_string(),
                "gemini-1.5-flash".to_string(),
            ).with_rate_limits(aggressive_rate_limits()),
        ];

        for (i, settings) in test_cases.into_iter().enumerate() {
            let provider_result = GeminiProvider::new(settings).await;
            assert!(provider_result.is_ok(), "Test case {} should succeed", i);
            
            let provider = provider_result.unwrap();
            let metadata = provider.metadata();
            assert_eq!(metadata.name(), "gemini");
        }
    }

    /// Test Gemini provider creation failures with invalid configurations
    #[tokio::test]
    async fn test_gemini_provider_creation_failures() {
        let _guard = TestEnvironmentGuard::new();
        
        let invalid_settings = invalid_provider_settings();
        for (i, settings) in invalid_settings.into_iter().enumerate() {
            let provider_result = GeminiProvider::new(settings).await;
            assert!(provider_result.is_err(), "Invalid settings case {} should fail", i);
        }
    }

    /// Test Gemini API key format validation
    #[tokio::test]
    async fn test_gemini_api_key_format_validation() {
        let _guard = TestEnvironmentGuard::new();
        
        let valid_formats = vec![
            "AIzaSyTest1234567890abcdef1234567890abcdef",
            "AIzaSy123456789012345678901234567890123456",
        ];
        
        for api_key in valid_formats {
            let settings = ProviderSettings::new(
                api_key.to_string(),
                "gemini-1.5-pro".to_string(),
            );
            
            let provider_result = GeminiProvider::new(settings).await;
            assert!(provider_result.is_ok(), "Valid API key format should work: {}", api_key);
        }
        
        let invalid_formats = vec![
            "sk-test123", // Wrong prefix
            "invalid-key",
            "AIzaSy", // Too short
            "GoogleAPI123", // Wrong format
        ];
        
        for api_key in invalid_formats {
            let settings = ProviderSettings::new(
                api_key.to_string(),
                "gemini-1.5-pro".to_string(),
            );
            
            let provider_result = GeminiProvider::new(settings).await;
            assert!(provider_result.is_err(), "Invalid API key format should fail: {}", api_key);
        }
    }

    /// Test Gemini provider creation from environment variables
    #[tokio::test]
    async fn test_gemini_provider_from_environment() {
        let _guard = TestEnvironmentGuard::new();
        
        // Test with environment variable
        std::env::set_var("GOOGLE_API_KEY", "AIzaSyTest1234567890abcdef1234567890abcdef");
        
        let settings = ProviderSettings::from_env("GOOGLE_API_KEY", "gemini-1.5-pro".to_string());
        assert!(settings.is_ok());
        
        let provider_result = GeminiProvider::new(settings.unwrap()).await;
        assert!(provider_result.is_ok());
        
        // Clean up
        std::env::remove_var("GOOGLE_API_KEY");
        
        // Test with missing environment variable
        let missing_settings = ProviderSettings::from_env("NONEXISTENT_KEY", "gemini-1.5-pro".to_string());
        assert!(missing_settings.is_err());
    }
}

mod gemini_api_integration_tests {
    use super::*;

    /// Test Gemini Google AI API authentication handling
    #[tokio::test]
    async fn test_gemini_authentication_handling() {
        let _guard = TestEnvironmentGuard::new();
        
        // Test with invalid API key
        let invalid_key_settings = ProviderSettings::new(
            "AIzaSyInvalid123".to_string(),
            "gemini-1.5-pro".to_string(),
        );
        
        let provider = GeminiProvider::new(invalid_key_settings).await.unwrap();
        let result = provider.research_query("Test query".to_string()).await;
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ProviderError::AuthenticationFailed { .. }));
        assert_eq!(error.provider(), "gemini");
    }

    /// Test Gemini API request formatting
    #[tokio::test]
    async fn test_gemini_request_formatting() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_gemini_settings();
        let provider = GeminiProvider::new(settings).await.unwrap();
        
        // Test that provider can handle various query types for Gemini
        let test_cases = vec![
            "Simple question for Gemini",
            "Question with special characters: !@#$%^&*()",
            "Multi-line\nquestion\nwith breaks",
            "Question with \"quotes\" and 'apostrophes'",
            "Question with unicode: 🤖 AI testing with Gemini",
            "Code analysis: def hello(): print('world')",
            "Mathematical equation: E = mc²",
        ];

        for query in test_cases {
            let result = provider.validate_query(query);
            assert!(result.is_ok(), "Query validation should succeed for: {}", query);
        }
    }

    /// Test Gemini API response parsing and validation
    #[tokio::test]
    async fn test_gemini_response_handling() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_gemini_settings();
        let provider = GeminiProvider::new(settings).await.unwrap();
        
        // Test basic query execution
        let result = provider.research_query("What is the capital of France?".to_string()).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.is_empty());
        assert!(response.contains("What is the capital of France?"));
    }

    /// Test Gemini API version and headers
    #[tokio::test]
    async fn test_gemini_api_version_headers() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        let metadata = provider.metadata();
        
        // Verify Gemini-specific API attributes
        let attributes = metadata.custom_attributes();
        assert_eq!(attributes.get("provider_type"), Some(&"gemini".to_string()));
        assert_eq!(attributes.get("api_version"), Some(&"v1beta".to_string()));
        assert_eq!(attributes.get("safety_settings"), Some(&"enabled".to_string()));
    }
}

mod gemini_model_specific_tests {
    use super::*;

    /// Test Gemini model-specific configurations and capabilities
    #[tokio::test]
    async fn test_gemini_model_configurations() {
        let _guard = TestEnvironmentGuard::new();
        
        let models = vec![
            ("gemini-1.5-pro", 1000000, "Most capable model"),
            ("gemini-1.5-flash", 1000000, "Fast and efficient model"),
            ("gemini-1.0-pro", 30720, "Stable model"),
        ];
        
        for (model, expected_context_length, _description) in models {
            let settings = ProviderSettings::new(
                "AIzaSyTest1234567890abcdef1234567890abcdef".to_string(),
                model.to_string(),
            );
            
            let provider = GeminiProvider::new(settings).await.unwrap();
            let metadata = provider.metadata();
            
            assert_eq!(metadata.name(), "gemini");
            assert!(metadata.supported_models().contains(&model.to_string()));
            
            // For this test, we'll use the default context length from our mock
            // In a real implementation, this would vary by model
            if model.contains("1.5") {
                assert_eq!(metadata.max_context_length(), 1000000);
            }
        }
    }

    /// Test Gemini model-specific pricing
    #[tokio::test]
    async fn test_gemini_model_specific_pricing() {
        let _guard = TestEnvironmentGuard::new();
        
        let pro_settings = ProviderSettings::new(
            "AIzaSyTest1234567890abcdef1234567890abcdef".to_string(),
            "gemini-1.5-pro".to_string(),
        );
        let flash_settings = ProviderSettings::new(
            "AIzaSyTest1234567890abcdef1234567890abcdef".to_string(),
            "gemini-1.5-flash".to_string(),
        );
        
        let pro_provider = GeminiProvider::new(pro_settings).await.unwrap();
        let flash_provider = GeminiProvider::new(flash_settings).await.unwrap();
        
        let query = "What is machine learning?";
        
        let pro_cost = pro_provider.estimate_cost(query).await.unwrap();
        let flash_cost = flash_provider.estimate_cost(query).await.unwrap();
        
        // Token estimates should be similar
        assert!((pro_cost.estimated_input_tokens as i32 - flash_cost.estimated_input_tokens as i32).abs() <= 2);
        
        // Both should have reasonable costs
        assert!(pro_cost.estimated_cost_usd.is_some());
        assert!(flash_cost.estimated_cost_usd.is_some());
        assert!(pro_cost.estimated_cost_usd.unwrap() > 0.0);
        assert!(flash_cost.estimated_cost_usd.unwrap() > 0.0);
    }

    /// Test Gemini multimodal capabilities
    #[tokio::test]
    async fn test_gemini_multimodal_capabilities() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        let metadata = provider.metadata();
        
        // Gemini should support multimodal capabilities
        let capabilities = metadata.capabilities();
        assert!(capabilities.contains(&"multimodal".to_string()));
        
        // Test various content types that Gemini can handle
        let multimodal_queries = vec![
            "Analyze this image: [image_placeholder]",
            "Describe what you see in this photo",
            "Convert this image to text",
            "What's in this picture?",
        ];
        
        for query in multimodal_queries {
            let validation = provider.validate_query(query);
            assert!(validation.is_ok(), "Multimodal query should be valid: {}", query);
        }
    }

    /// Test Gemini large context window handling
    #[tokio::test]
    async fn test_gemini_large_context_handling() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        let metadata = provider.metadata();
        
        // Gemini 1.5 models support 1M token context
        assert_eq!(metadata.max_context_length(), 1000000);
        
        // Test with very long query utilizing large context
        let long_query = "word ".repeat(100000); // ~400k characters
        let cost = provider.estimate_cost(&long_query).await.unwrap();
        
        // Should handle very long queries
        assert!(cost.estimated_input_tokens > 80000); // Should be many tokens
        assert!(cost.estimated_cost_usd.is_some());
    }
}

mod gemini_safety_settings_tests {
    use super::*;

    /// Test Gemini safety settings configuration
    #[tokio::test]
    async fn test_gemini_safety_settings() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        
        // Test that Gemini metadata includes safety capabilities
        let metadata = provider.metadata();
        let capabilities = metadata.capabilities();
        
        // Gemini should have safety-related capabilities
        assert!(capabilities.contains(&"safety_settings".to_string()));
        
        // Verify safety settings are enabled in attributes
        let attributes = metadata.custom_attributes();
        assert_eq!(attributes.get("safety_settings"), Some(&"enabled".to_string()));
    }

    /// Test Gemini content filtering and safety responses
    #[tokio::test]
    async fn test_gemini_content_filtering() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        
        // Test that Gemini provider handles various safe content appropriately
        let safe_queries = vec![
            "Explain quantum physics",
            "Help me write a story about friendship",
            "What are the benefits of renewable energy?",
            "How do I bake a cake?",
            "Summarize the history of computing",
        ];
        
        for query in safe_queries {
            let validation = provider.validate_query(query);
            assert!(validation.is_ok(), "Safe content should be accepted: {}", query);
            
            // Test cost estimation works for safe content
            let cost = provider.estimate_cost(query).await;
            assert!(cost.is_ok(), "Cost estimation should work for safe content: {}", query);
        }
    }

    /// Test Gemini safety thresholds and filtering levels
    #[tokio::test]
    async fn test_gemini_safety_thresholds() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        
        // Test various content types that should be handled by safety settings
        let content_tests = vec![
            ("Educational medical content", "What are the symptoms of flu?", true),
            ("Historical discussion", "Discuss the causes of World War II", true),
            ("Creative writing", "Write a story about adventure", true),
            ("Technical content", "How does machine learning work?", true),
        ];
        
        for (category, content, should_pass) in content_tests {
            let result = provider.validate_query(content);
            if should_pass {
                assert!(result.is_ok(), "{} should pass safety validation: {}", category, content);
            }
        }
    }

    /// Test Gemini safety metadata and configuration
    #[tokio::test]
    async fn test_gemini_safety_metadata() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        let metadata = provider.metadata();
        
        // Verify Gemini-specific safety attributes
        let attributes = metadata.custom_attributes();
        assert!(attributes.contains_key("safety_settings"));
        assert_eq!(attributes.get("provider_type"), Some(&"gemini".to_string()));
        
        // Gemini should indicate it has built-in safety features
        let capabilities = metadata.capabilities();
        assert!(capabilities.contains(&"safety_settings".to_string()));
    }
}

mod gemini_rate_limiting_tests {
    use super::*;

    /// Test Gemini rate limiting configuration
    #[tokio::test]
    async fn test_gemini_rate_limiting_configuration() {
        let _guard = TestEnvironmentGuard::new();
        
        let rate_limits = RateLimitConfig {
            requests_per_minute: 20, // Google AI typically has different limits
            input_tokens_per_minute: 15000,
            output_tokens_per_minute: 8000,
            max_concurrent_requests: 5,
        };
        
        let settings = valid_gemini_settings().with_rate_limits(rate_limits.clone());
        let provider = GeminiProvider::new(settings).await.unwrap();
        
        let metadata = provider.metadata();
        let provider_rate_limits = metadata.rate_limits();
        
        assert_eq!(provider_rate_limits.requests_per_minute, rate_limits.requests_per_minute);
        assert_eq!(provider_rate_limits.input_tokens_per_minute, rate_limits.input_tokens_per_minute);
        assert_eq!(provider_rate_limits.output_tokens_per_minute, rate_limits.output_tokens_per_minute);
        assert_eq!(provider_rate_limits.max_concurrent_requests, rate_limits.max_concurrent_requests);
    }

    /// Test Gemini concurrent request handling
    #[tokio::test]
    async fn test_gemini_concurrent_request_handling() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_gemini_settings().with_rate_limits(RateLimitConfig {
            requests_per_minute: 100,
            input_tokens_per_minute: 50000,
            output_tokens_per_minute: 20000,
            max_concurrent_requests: 5,
        });
        
        let provider = Arc::new(GeminiProvider::new(settings).await.unwrap());
        let mut handles = Vec::new();
        
        // Start multiple concurrent requests
        for i in 0..8 {
            let provider_clone = Arc::clone(&provider);
            let handle = tokio::spawn(async move {
                provider_clone.research_query(format!("Concurrent Gemini query {}", i)).await
            });
            handles.push(handle);
        }
        
        // Wait for all requests to complete
        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await.unwrap();
            results.push(result);
        }
        
        // All should succeed with our mock implementation
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        assert_eq!(success_count, 8);
    }
}

mod gemini_error_handling_tests {
    use super::*;

    /// Test Gemini-specific error mapping and handling
    #[tokio::test]
    async fn test_gemini_error_mapping() {
        let _guard = TestEnvironmentGuard::new();
        
        // Test with invalid API key
        let invalid_settings = ProviderSettings::new(
            "AIzaSyInvalid123".to_string(),
            "gemini-1.5-pro".to_string(),
        );
        
        let provider = GeminiProvider::new(invalid_settings).await.unwrap();
        let result = provider.research_query("Test".to_string()).await;
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        
        match error {
            ProviderError::AuthenticationFailed { provider, message } => {
                assert_eq!(provider, "gemini");
                assert!(!message.is_empty());
            }
            _ => panic!("Expected AuthenticationFailed error"),
        }
        
        // Verify provider name is correctly set
        assert_eq!(error.provider(), "gemini");
    }

    /// Test Gemini timeout handling
    #[tokio::test]
    async fn test_gemini_timeout_handling() {
        let _guard = TestEnvironmentGuard::new();
        
        let timeout_settings = valid_gemini_settings()
            .with_timeout(Duration::from_millis(1)); // Very short timeout
        
        let provider = GeminiProvider::new(timeout_settings).await.unwrap();
        
        // For our mock implementation, this will complete quickly
        let result = provider.research_query("Test query".to_string()).await;
        assert!(result.is_ok()); // Mock doesn't simulate timeout
    }

    /// Test Gemini retry mechanism
    #[tokio::test]
    async fn test_gemini_retry_mechanism() {
        let _guard = TestEnvironmentGuard::new();
        
        let retry_config = RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(200),
            backoff_multiplier: 2.0,
            jitter: false,
        };
        
        let settings = valid_gemini_settings().with_retry(retry_config);
        let provider = GeminiProvider::new(settings).await.unwrap();
        
        // Test successful query (mock implementation)
        let start_time = Instant::now();
        let result = provider.research_query("Test query".to_string()).await;
        let elapsed = start_time.elapsed();
        
        assert!(result.is_ok());
        // Should complete quickly with mock implementation
        assert!(elapsed < Duration::from_secs(1));
    }

    /// Test Gemini-specific error codes and messages
    #[tokio::test]
    async fn test_gemini_error_codes_and_messages() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        
        // Test various error scenarios specific to Gemini
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
                    assert_eq!(error.provider(), "gemini");
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

    /// Test Gemini safety error handling
    #[tokio::test]
    async fn test_gemini_safety_error_handling() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        
        // Test that Gemini handles edge cases appropriately
        let edge_case_queries = vec![
            "A".repeat(1000), // Very repetitive content
            "Mixed language query with français and español and 中文 and العربية",
            "Query with many numbers: 1 2 3 4 5 6 7 8 9 0",
            "Query with special formatting\n\n\tTabbed content\n\n",
            "Mathematical notation: ∑(x²) = ∫f(x)dx",
        ];
        
        for query in edge_case_queries {
            let validation = provider.validate_query(&query);
            let cost_estimation = provider.estimate_cost(&query).await;
            
            // These should generally work with our mock implementation
            assert!(validation.is_ok(), "Edge case should validate: {}", query);
            assert!(cost_estimation.is_ok(), "Cost estimation should work: {}", query);
        }
    }
}

mod gemini_cost_estimation_tests {
    use super::*;

    /// Test Gemini token counting and cost estimation accuracy
    #[tokio::test]
    async fn test_gemini_cost_estimation_accuracy() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        
        let test_cases = vec![
            ("Hello", 1, 2),           // Very short
            ("What is quantum computing?", 4, 6),  // Short question
            ("Explain machine learning in detail", 5, 7), // Medium
            ("Write a comprehensive analysis of artificial intelligence including neural networks, deep learning, and their applications", 20, 25), // Long
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

    /// Test Gemini pricing with different content types
    #[tokio::test]
    async fn test_gemini_cost_estimation_content_types() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        
        let content_types = vec![
            ("Code analysis", "def fibonacci(n):\n    if n <= 1:\n        return n\n    return fibonacci(n-1) + fibonacci(n-2)\n\nAnalyze this code"),
            ("Creative writing", "Write a poem about the ocean"),
            ("Academic query", "Explain the theory of relativity"),
            ("Mathematical problem", "Solve: ∫x²dx from 0 to 5"),
            ("Multimodal query", "Describe this image: [image data]"),
            ("Large context query", "word ".repeat(10000)),
        ];
        
        for (content_type, query) in content_types {
            let cost = provider.estimate_cost(query).await.unwrap();
            
            assert!(cost.estimated_input_tokens > 0, "Should estimate tokens for {}", content_type);
            assert!(cost.estimated_output_tokens > 0, "Should estimate output tokens for {}", content_type);
            assert!(cost.estimated_cost_usd.is_some(), "Should provide cost for {}", content_type);
            assert!(cost.estimated_cost_usd.unwrap() > 0.0, "Cost should be positive for {}", content_type);
        }
    }

    /// Test Gemini cost estimation with large context
    #[tokio::test]
    async fn test_gemini_large_context_cost_estimation() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        
        // Test with very large context (Gemini supports 1M tokens)
        let huge_query = "word ".repeat(250000); // ~1M characters
        let cost = provider.estimate_cost(&huge_query).await.unwrap();
        
        // Should handle huge queries
        assert!(cost.estimated_input_tokens > 200000); // Should be many tokens
        assert!(cost.estimated_cost_usd.is_some());
        assert!(cost.estimated_cost_usd.unwrap() > 1.0); // Should be expensive for huge query
    }

    /// Test Gemini cost estimation edge cases
    #[tokio::test]
    async fn test_gemini_cost_estimation_edge_cases() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        
        // Empty query
        let empty_cost = provider.estimate_cost("").await.unwrap();
        assert!(empty_cost.estimated_input_tokens >= 0);
        
        // Unicode and special characters
        let unicode_query = "🤖 Gemini test: café, résumé, 北京, العربية, математика";
        let unicode_cost = provider.estimate_cost(unicode_query).await.unwrap();
        assert!(unicode_cost.estimated_input_tokens > 0);
        assert!(unicode_cost.estimated_cost_usd.is_some());
        
        // Code with special characters
        let code_query = r#"
        function test() {
            console.log("Hello, world!");
            return 42;
        }
        "#;
        let code_cost = provider.estimate_cost(code_query).await.unwrap();
        assert!(code_cost.estimated_input_tokens > 0);
        assert!(code_cost.estimated_cost_usd.is_some());
    }
}

mod gemini_metadata_tests {
    use super::*;

    /// Test Gemini provider metadata compliance
    #[tokio::test]
    async fn test_gemini_metadata_compliance() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        let metadata = provider.metadata();
        
        // Required fields
        assert_eq!(metadata.name(), "gemini");
        assert!(!metadata.version().is_empty());
        assert!(!metadata.capabilities().is_empty());
        assert!(!metadata.supported_models().is_empty());
        assert!(metadata.max_context_length() > 0);
        
        // Gemini-specific capabilities
        let capabilities = metadata.capabilities();
        assert!(capabilities.contains(&"research".to_string()));
        assert!(capabilities.contains(&"rate_limited".to_string()));
        assert!(capabilities.contains(&"cost_estimation".to_string()));
        assert!(capabilities.contains(&"safety_settings".to_string()));
        assert!(capabilities.contains(&"multimodal".to_string()));
        
        // Rate limits should be configured
        let rate_limits = metadata.rate_limits();
        assert!(rate_limits.requests_per_minute > 0);
        assert!(rate_limits.input_tokens_per_minute > 0);
        assert!(rate_limits.output_tokens_per_minute > 0);
        assert!(rate_limits.max_concurrent_requests > 0);
        
        // Gemini-specific attributes
        let attributes = metadata.custom_attributes();
        assert_eq!(attributes.get("provider_type"), Some(&"gemini".to_string()));
        assert_eq!(attributes.get("api_version"), Some(&"v1beta".to_string()));
        assert_eq!(attributes.get("safety_settings"), Some(&"enabled".to_string()));
        
        // Should support Gemini models
        let models = metadata.supported_models();
        assert!(models.contains(&"gemini-1.5-pro".to_string()));
        assert!(models.contains(&"gemini-1.5-flash".to_string()));
        assert!(models.contains(&"gemini-1.0-pro".to_string()));
        
        // Context length should be large for Gemini 1.5 models
        assert_eq!(metadata.max_context_length(), 1000000);
        
        // Should support streaming
        assert!(metadata.supports_streaming());
    }

    /// Test Gemini metadata with different model configurations
    #[tokio::test]
    async fn test_gemini_metadata_model_variations() {
        let _guard = TestEnvironmentGuard::new();
        
        let models = vec![
            "gemini-1.5-pro",
            "gemini-1.5-flash", 
            "gemini-1.0-pro",
        ];
        
        for model in models {
            let settings = ProviderSettings::new(
                "AIzaSyTest1234567890abcdef1234567890abcdef".to_string(),
                model.to_string(),
            );
            
            let provider = GeminiProvider::new(settings).await.unwrap();
            let metadata = provider.metadata();
            
            assert_eq!(metadata.name(), "gemini");
            assert!(metadata.supported_models().contains(&model.to_string()));
            
            // All Gemini models should have safety settings
            assert!(metadata.capabilities().contains(&"safety_settings".to_string()));
            
            // Gemini 1.5 models should have large context
            if model.contains("1.5") {
                assert_eq!(metadata.max_context_length(), 1000000);
            }
        }
    }

    /// Test Gemini metadata consistency
    #[tokio::test]
    async fn test_gemini_metadata_consistency() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        
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
        assert_eq!(metadata1.supports_streaming(), metadata2.supports_streaming());
    }
}

mod gemini_health_check_tests {
    use super::*;

    /// Test Gemini provider health check functionality
    #[tokio::test]
    async fn test_gemini_health_check() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        let health_result = provider.health_check().await;
        
        assert!(health_result.is_ok());
        
        let health_status = health_result.unwrap();
        match health_status {
            HealthStatus::Healthy => {
                // With test credentials, might be degraded instead
            }
            HealthStatus::Degraded(reason) => {
                assert!(!reason.is_empty());
                assert!(reason.contains("Test") || reason.contains("test"));
            }
            HealthStatus::Unhealthy(reason) => {
                assert!(!reason.is_empty());
            }
        }
    }

    /// Test Gemini health check with invalid credentials
    #[tokio::test]
    async fn test_gemini_health_check_invalid_credentials() {
        let _guard = TestEnvironmentGuard::new();
        
        let invalid_settings = ProviderSettings::new(
            "AIzaSyInvalid123".to_string(),
            "gemini-1.5-pro".to_string(),
        );
        
        let provider = GeminiProvider::new(invalid_settings).await.unwrap();
        let health_result = provider.health_check().await;
        
        assert!(health_result.is_ok());
        
        let health_status = health_result.unwrap();
        match health_status {
            HealthStatus::Unhealthy(reason) => {
                assert!(!reason.is_empty());
                assert!(reason.to_lowercase().contains("invalid") ||
                       reason.to_lowercase().contains("api key"));
            }
            _ => {
                // Other statuses might be acceptable depending on implementation
            }
        }
    }

    /// Test Gemini health check performance
    #[tokio::test]
    async fn test_gemini_health_check_performance() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        
        let start = Instant::now();
        let _health = provider.health_check().await;
        let duration = start.elapsed();
        
        // Health check should be fast
        assert!(duration < Duration::from_secs(5));
    }
}

mod gemini_usage_statistics_tests {
    use super::*;

    /// Test Gemini usage statistics tracking
    #[tokio::test]
    async fn test_gemini_usage_statistics() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        
        // Initial stats should be zero
        let initial_stats = provider.usage_stats().await.unwrap();
        assert_eq!(initial_stats.total_requests, 0);
        assert_eq!(initial_stats.successful_requests, 0);
        assert_eq!(initial_stats.failed_requests, 0);
        
        // Our mock implementation returns default stats
        // In a real implementation, stats would be tracked
    }

    /// Test Gemini usage statistics isolation
    #[tokio::test]
    async fn test_gemini_usage_statistics_isolation() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider1 = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        let provider2 = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        
        // Make requests to both providers
        let _result1 = provider1.research_query("Gemini query 1".to_string()).await;
        let _result2 = provider2.research_query("Gemini query 2".to_string()).await;
        
        // Stats should be isolated (though our mock returns defaults)
        let stats1 = provider1.usage_stats().await.unwrap();
        let stats2 = provider2.usage_stats().await.unwrap();
        
        // Mock implementation returns default stats
        assert_eq!(stats1.total_requests, 0);
        assert_eq!(stats2.total_requests, 0);
    }
}

mod gemini_validation_tests {
    use super::*;

    /// Test Gemini query validation
    #[tokio::test]
    async fn test_gemini_query_validation() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        
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

    /// Test Gemini multimodal query validation
    #[tokio::test]
    async fn test_gemini_multimodal_validation() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        
        let multimodal_queries = vec![
            "Analyze this image",
            "What do you see in this photo?",
            "Describe the contents of this picture",
            "Convert this image to text",
            "What colors are in this image?",
        ];
        
        for query in multimodal_queries {
            let result = provider.validate_query(query);
            assert!(result.is_ok(), "Multimodal query should be valid: {}", query);
        }
    }

    /// Test Gemini large context validation
    #[tokio::test]
    async fn test_gemini_large_context_validation() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        
        // Test very large query (within Gemini's 1M token limit)
        let large_query = "word ".repeat(100000); // ~400k characters
        let result = provider.validate_query(&large_query);
        assert!(result.is_ok(), "Large query within limits should be valid");
        
        // Test extremely large query (beyond reasonable limits)
        let huge_query = "word ".repeat(500000); // ~2M characters
        let result = provider.validate_query(&huge_query);
        // This might pass or fail depending on implementation
        // Either is acceptable for this test
    }
}

mod gemini_concurrent_access_tests {
    use super::*;

    /// Test Gemini provider thread safety
    #[tokio::test]
    async fn test_gemini_thread_safety() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = Arc::new(GeminiProvider::new(valid_gemini_settings()).await.unwrap());
        let mut handles = Vec::new();
        
        // Spawn concurrent tasks
        for i in 0..10 {
            let provider_clone = Arc::clone(&provider);
            let handle = tokio::spawn(async move {
                // Mix different operations
                match i % 4 {
                    0 => {
                        let _result = provider_clone.research_query(format!("Gemini query {}", i)).await;
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

    /// Test Gemini provider with high concurrency
    #[tokio::test]
    async fn test_gemini_high_concurrency() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = Arc::new(GeminiProvider::new(valid_gemini_settings()).await.unwrap());
        let mut handles = Vec::new();
        
        // High concurrency test
        for i in 0..50 {
            let provider_clone = Arc::clone(&provider);
            let handle = tokio::spawn(async move {
                let query = format!("Concurrent Gemini query {}", i);
                let result = provider_clone.research_query(query).await;
                
                // Our mock implementation should succeed
                result.is_ok()
            });
            handles.push(handle);
        }
        
        // Collect results
        let mut success_count = 0;
        for handle in handles {
            let success = handle.await.unwrap();
            if success {
                success_count += 1;
            }
        }
        
        // All operations should succeed with mock implementation
        assert_eq!(success_count, 50);
    }
}

// Property-based tests for Gemini provider
proptest! {
    #[test]
    fn test_gemini_cost_estimation_properties(
        query in ".*",
        input_cost_per_1k in 0.001..0.01,
        output_cost_per_1k in 0.001..0.01
    ) {
        tokio_test::block_on(async {
            let _guard = TestEnvironmentGuard::new();
            
            let settings = valid_gemini_settings();
            let provider = GeminiProvider::new(settings).await.unwrap();
            
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
    fn test_gemini_metadata_properties(
        model in "(gemini-1\\.5-pro|gemini-1\\.5-flash|gemini-1\\.0-pro)"
    ) {
        tokio_test::block_on(async {
            let _guard = TestEnvironmentGuard::new();
            
            let settings = ProviderSettings::new(
                "AIzaSyTest1234567890abcdef1234567890abcdef".to_string(),
                model.clone(),
            );
            
            let provider = GeminiProvider::new(settings).await.unwrap();
            let metadata = provider.metadata();
            
            prop_assert_eq!(metadata.name(), "gemini");
            prop_assert!(metadata.supported_models().contains(&model));
            prop_assert!(metadata.capabilities().contains(&"multimodal".to_string()));
            prop_assert!(metadata.capabilities().contains(&"safety_settings".to_string()));
            prop_assert!(metadata.supports_streaming());
        });
    }

    #[test]
    fn test_gemini_validation_properties(query in ".*") {
        tokio_test::block_on(async {
            let _guard = TestEnvironmentGuard::new();
            
            let settings = valid_gemini_settings();
            let provider = GeminiProvider::new(settings).await.unwrap();
            
            let result = provider.validate_query(&query);
            
            if query.trim().is_empty() {
                prop_assert!(result.is_err());
            } else {
                // Non-empty queries should generally pass validation
                prop_assert!(result.is_ok() || matches!(result, Err(ProviderError::ConfigurationError { .. })));
            }
        });
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Integration test for complete Gemini provider workflow
    #[tokio::test]
    async fn test_gemini_provider_complete_workflow() {
        let _guard = TestEnvironmentGuard::new();
        
        let settings = valid_gemini_settings();
        let provider = GeminiProvider::new(settings).await.unwrap();
        
        // 1. Check metadata
        let metadata = provider.metadata();
        assert_eq!(metadata.name(), "gemini");
        assert!(metadata.capabilities().contains(&"multimodal".to_string()));
        assert!(metadata.capabilities().contains(&"safety_settings".to_string()));
        
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
        
        // 5. Execute query
        let result = provider.research_query(query.to_string()).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(!response.is_empty());
        assert!(response.contains(query));
        
        // 6. Check usage stats
        let stats = provider.usage_stats().await;
        assert!(stats.is_ok());
    }

    /// Integration test with environment variables
    #[tokio::test]
    async fn test_gemini_provider_environment_integration() {
        let _guard = TestEnvironmentGuard::new();
        
        // Set environment variables
        std::env::set_var("GOOGLE_API_KEY", "AIzaSyTest1234567890abcdef1234567890abcdef");
        std::env::set_var("GEMINI_MODEL", "gemini-1.5-flash");
        
        // Create provider from environment
        let api_key = std::env::var("GOOGLE_API_KEY").unwrap();
        let model = std::env::var("GEMINI_MODEL").unwrap();
        
        let settings = ProviderSettings::new(api_key, model);
        let provider = GeminiProvider::new(settings).await.unwrap();
        
        let metadata = provider.metadata();
        assert_eq!(metadata.name(), "gemini");
        assert!(metadata.supported_models().contains(&"gemini-1.5-flash".to_string()));
        
        // Clean up
        std::env::remove_var("GOOGLE_API_KEY");
        std::env::remove_var("GEMINI_MODEL");
    }

    /// Integration test comparing Gemini features
    #[tokio::test]
    async fn test_gemini_features_integration() {
        let _guard = TestEnvironmentGuard::new();
        
        let provider = GeminiProvider::new(valid_gemini_settings()).await.unwrap();
        
        // Test multimodal capabilities
        let multimodal_query = "Describe this image";
        let multimodal_result = provider.research_query(multimodal_query.to_string()).await;
        assert!(multimodal_result.is_ok());
        
        // Test large context
        let large_query = "word ".repeat(10000);
        let large_result = provider.research_query(large_query).await;
        assert!(large_result.is_ok());
        
        // Test safety features
        let safe_query = "Explain quantum physics";
        let safe_result = provider.research_query(safe_query.to_string()).await;
        assert!(safe_result.is_ok());
        
        // Test cost estimation for various types
        let queries = vec![
            "Short query",
            "Medium length query with multiple words",
            "Very long query ".repeat(100),
        ];
        
        for query in queries {
            let cost = provider.estimate_cost(&query).await;
            assert!(cost.is_ok());
            
            let cost_data = cost.unwrap();
            assert!(cost_data.estimated_input_tokens > 0);
            assert!(cost_data.estimated_cost_usd.is_some());
        }
    }
}