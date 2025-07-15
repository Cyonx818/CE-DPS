//! Comprehensive unit tests for Provider Trait Abstraction (Task 1.1)
//! 
//! This module tests:
//! - Provider trait implementation compliance
//! - Async interface behavior and error handling  
//! - Metadata reporting accuracy
//! - Health check functionality
//! - Cost estimation and usage statistics
//! - Thread safety and concurrent access
//! - Error handling and retryability

use fortitude::providers::{
    Provider, ProviderError, ProviderResult, ProviderMetadata, HealthStatus, 
    QueryCost, UsageStats, RateLimitConfig
};
use crate::common::{MockProvider, MockProviderConfig, test_queries};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use proptest::prelude::*;

mod provider_trait_compliance_tests {
    use super::*;

    /// ANCHOR: Verifies Provider trait can be used as trait object with Send + Sync
    #[tokio::test]
    async fn test_anchor_provider_trait_object_compliance() {
        let provider: Box<dyn Provider> = Box::new(MockProvider::new("trait-object-test"));
        
        // Test all required trait methods work through trait object
        let metadata = provider.metadata();
        assert_eq!(metadata.name(), "trait-object-test");
        
        let health = provider.health_check().await;
        assert!(health.is_ok());
        
        let query = provider.research_query("test query".to_string()).await;
        assert!(query.is_ok());
        
        let validation = provider.validate_query("valid query");
        assert!(validation.is_ok());
        
        let cost = provider.estimate_cost("test").await;
        assert!(cost.is_ok());
        
        let stats = provider.usage_stats().await;
        assert!(stats.is_ok());
    }

    /// Test Provider trait can be shared across async tasks (Send + Sync compliance)
    #[tokio::test]
    async fn test_provider_thread_safety() {
        let provider = Arc::new(MockProvider::new("thread-safety-test"));
        let mut handles = Vec::new();

        // Spawn multiple concurrent tasks
        for i in 0..10 {
            let provider_clone = Arc::clone(&provider);
            let handle = tokio::spawn(async move {
                let query = format!("Concurrent query {}", i);
                let result = provider_clone.research_query(query).await;
                assert!(result.is_ok());
                
                let metadata = provider_clone.metadata();
                assert_eq!(metadata.name(), "thread-safety-test");
                
                let health = provider_clone.health_check().await;
                assert!(health.is_ok());
                
                i
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
        }
    }

    /// Test Provider trait implementation with various provider configurations
    #[tokio::test]
    async fn test_provider_trait_implementation_variants() {
        let configs = vec![
            MockProviderConfig::default(),
            MockProviderConfig {
                healthy: false,
                ..Default::default()
            },
            MockProviderConfig {
                response_delay: Duration::from_millis(500),
                ..Default::default()
            },
            MockProviderConfig {
                supported_models: vec!["model-1".to_string(), "model-2".to_string()],
                max_context_length: 16384,
                supports_streaming: true,
                ..Default::default()
            },
        ];

        for (i, config) in configs.into_iter().enumerate() {
            let provider = MockProvider::new(&format!("variant-{}", i)).with_config(config);
            
            // Test trait compliance for each variant
            let _: Box<dyn Provider> = Box::new(provider.clone());
            
            // Test metadata consistency
            let metadata = provider.metadata();
            assert_eq!(metadata.name(), format!("variant-{}", i));
            assert!(!metadata.version().is_empty());
            assert!(!metadata.capabilities().is_empty());
            
            // Test health check
            let health = provider.health_check().await;
            assert!(health.is_ok());
            
            // Test cost estimation
            let cost = provider.estimate_cost("test query").await;
            assert!(cost.is_ok());
            
            // Test usage stats
            let stats = provider.usage_stats().await;
            assert!(stats.is_ok());
        }
    }
}

mod async_interface_tests {
    use super::*;

    /// Test async research_query method behavior
    #[tokio::test]
    async fn test_async_research_query_behavior() {
        let provider = MockProvider::new("async-test");
        
        // Test basic async query
        let result = provider.research_query("test query".to_string()).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(response.contains("test query"));
        assert!(response.contains("async-test"));
    }

    /// Test async query with delays and timeouts
    #[tokio::test]
    async fn test_async_query_with_delays() {
        let slow_provider = MockProvider::new("slow-provider")
            .with_delay(Duration::from_millis(200));
        
        let start = Instant::now();
        let result = slow_provider.research_query("slow query".to_string()).await;
        let elapsed = start.elapsed();
        
        assert!(result.is_ok());
        assert!(elapsed >= Duration::from_millis(180)); // Allow some timing tolerance
    }

    /// Test async health check behavior
    #[tokio::test]
    async fn test_async_health_check_behavior() {
        let healthy_provider = MockProvider::new("healthy-async").with_health(true);
        let unhealthy_provider = MockProvider::new("unhealthy-async").with_health(false);
        
        let healthy_result = healthy_provider.health_check().await;
        assert!(healthy_result.is_ok());
        assert_eq!(healthy_result.unwrap(), HealthStatus::Healthy);
        
        let unhealthy_result = unhealthy_provider.health_check().await;
        assert!(unhealthy_result.is_ok());
        assert!(matches!(unhealthy_result.unwrap(), HealthStatus::Unhealthy(_)));
    }

    /// Test async cost estimation behavior
    #[tokio::test]
    async fn test_async_cost_estimation_behavior() {
        let provider = MockProvider::new("cost-async").with_pricing(0.001, 0.002);
        
        let cost = provider.estimate_cost("test query for cost estimation").await;
        assert!(cost.is_ok());
        
        let cost_result = cost.unwrap();
        assert!(cost_result.estimated_input_tokens > 0);
        assert!(cost_result.estimated_output_tokens > 0);
        assert!(cost_result.estimated_duration > Duration::ZERO);
        assert!(cost_result.estimated_cost_usd.is_some());
        assert!(cost_result.estimated_cost_usd.unwrap() > 0.0);
    }

    /// Test async usage statistics behavior
    #[tokio::test]
    async fn test_async_usage_stats_behavior() {
        let provider = MockProvider::new("stats-async");
        
        // Initial stats should be zero
        let initial_stats = provider.usage_stats().await;
        assert!(initial_stats.is_ok());
        
        // Make some requests to generate stats
        for i in 0..3 {
            let query = format!("Query {}", i);
            let _result = provider.research_query(query).await;
        }
        
        // Stats should be updated
        let updated_stats = provider.usage_stats().await;
        assert!(updated_stats.is_ok());
        
        let stats = updated_stats.unwrap();
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.successful_requests, 3);
        assert_eq!(stats.failed_requests, 0);
        assert!(stats.total_input_tokens > 0);
        assert!(stats.total_output_tokens > 0);
        assert!(stats.last_request_time.is_some());
    }

    /// Test concurrent async operations
    #[tokio::test]
    async fn test_concurrent_async_operations() {
        let provider = Arc::new(MockProvider::new("concurrent-async"));
        let mut handles = Vec::new();

        // Start multiple concurrent operations
        for i in 0..5 {
            let provider_clone = Arc::clone(&provider);
            let handle = tokio::spawn(async move {
                // Mix different async operations
                match i % 4 {
                    0 => {
                        let result = provider_clone.research_query(format!("Query {}", i)).await;
                        assert!(result.is_ok());
                    }
                    1 => {
                        let result = provider_clone.health_check().await;
                        assert!(result.is_ok());
                    }
                    2 => {
                        let result = provider_clone.estimate_cost(&format!("Cost query {}", i)).await;
                        assert!(result.is_ok());
                    }
                    3 => {
                        let result = provider_clone.usage_stats().await;
                        assert!(result.is_ok());
                    }
                    _ => unreachable!(),
                }
                i
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
        }
    }
}

mod error_handling_tests {
    use super::*;

    /// Test error handling for various failure scenarios
    #[tokio::test]
    async fn test_provider_error_scenarios() {
        // Test query failure
        let failing_provider = MockProvider::new("failing-provider").with_failure(true);
        let result = failing_provider.research_query("test".to_string()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProviderError::QueryFailed { .. }));

        // Test authentication failure
        let auth_failing_provider = MockProvider::new("auth-failing").with_auth_failure(true);
        let result = auth_failing_provider.research_query("test".to_string()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProviderError::AuthenticationFailed { .. }));

        // Test service unavailable
        let unavailable_provider = MockProvider::new("unavailable").with_service_unavailable(true);
        let result = unavailable_provider.research_query("test".to_string()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProviderError::ServiceUnavailable { .. }));

        // Test quota exceeded
        let quota_provider = MockProvider::new("quota-exceeded").with_quota_exceeded(true);
        let result = quota_provider.research_query("test".to_string()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProviderError::QuotaExceeded { .. }));

        // Test rate limit exceeded
        let rate_limited_provider = MockProvider::new("rate-limited").with_rate_limit_exceeded(true);
        let result = rate_limited_provider.research_query("test".to_string()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProviderError::RateLimitExceeded { .. }));
    }

    /// Test error retryability and retry information
    #[tokio::test]
    async fn test_error_retryability() {
        let rate_limit_error = ProviderError::RateLimitExceeded {
            provider: "test".to_string(),
            message: "Rate limited".to_string(),
            retry_after: Some(Duration::from_secs(60)),
            requests_remaining: Some(0),
            tokens_remaining: Some(1000),
        };

        let auth_error = ProviderError::AuthenticationFailed {
            provider: "test".to_string(),
            message: "Invalid API key".to_string(),
        };

        let timeout_error = ProviderError::Timeout {
            provider: "test".to_string(),
            duration: Duration::from_secs(30),
        };

        let network_error = ProviderError::NetworkError {
            provider: "test".to_string(),
            source: Box::new(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused")),
        };

        let service_error = ProviderError::ServiceUnavailable {
            provider: "test".to_string(),
            message: "Service down".to_string(),
            estimated_recovery: Some(Duration::from_secs(300)),
        };

        // Test retryability
        assert!(rate_limit_error.is_retryable());
        assert!(!auth_error.is_retryable());
        assert!(timeout_error.is_retryable());
        assert!(network_error.is_retryable());
        assert!(service_error.is_retryable());

        // Test retry delays
        assert_eq!(rate_limit_error.retry_after(), Some(Duration::from_secs(60)));
        assert_eq!(auth_error.retry_after(), None);
        assert_eq!(service_error.retry_after(), Some(Duration::from_secs(300)));

        // Test provider name extraction
        assert_eq!(rate_limit_error.provider(), "test");
        assert_eq!(auth_error.provider(), "test");
        assert_eq!(timeout_error.provider(), "test");
        assert_eq!(network_error.provider(), "test");
        assert_eq!(service_error.provider(), "test");
    }

    /// Test error context and metadata preservation
    #[tokio::test]
    async fn test_error_context_preservation() {
        let error = ProviderError::RateLimitExceeded {
            provider: "context-test".to_string(),
            message: "Detailed rate limit message".to_string(),
            retry_after: Some(Duration::from_secs(120)),
            requests_remaining: Some(5),
            tokens_remaining: Some(1000),
        };

        // Verify all context is preserved
        match error {
            ProviderError::RateLimitExceeded { 
                provider, 
                message, 
                retry_after, 
                requests_remaining, 
                tokens_remaining 
            } => {
                assert_eq!(provider, "context-test");
                assert_eq!(message, "Detailed rate limit message");
                assert_eq!(retry_after, Some(Duration::from_secs(120)));
                assert_eq!(requests_remaining, Some(5));
                assert_eq!(tokens_remaining, Some(1000));
            }
            _ => panic!("Expected RateLimitExceeded error"),
        }
    }
}

mod metadata_tests {
    use super::*;

    /// Test provider metadata completeness and accuracy
    #[tokio::test]
    async fn test_provider_metadata_completeness() {
        let provider = MockProvider::new("metadata-test")
            .with_models(vec!["model-1".to_string(), "model-2".to_string()])
            .with_context_length(16384)
            .with_streaming(true)
            .with_pricing(0.001, 0.002);

        let metadata = provider.metadata();

        // Test required fields
        assert!(!metadata.name().is_empty());
        assert!(!metadata.version().is_empty());
        assert!(!metadata.capabilities().is_empty());
        assert!(!metadata.supported_models().is_empty());
        assert!(metadata.max_context_length() > 0);

        // Test specific values
        assert_eq!(metadata.name(), "metadata-test");
        assert_eq!(metadata.supported_models().len(), 2);
        assert_eq!(metadata.max_context_length(), 16384);
        assert!(metadata.supports_streaming());

        // Test capabilities
        let capabilities = metadata.capabilities();
        assert!(capabilities.contains(&"research".to_string()));
        assert!(capabilities.contains(&"async".to_string()));
        assert!(capabilities.contains(&"mock".to_string()));
        assert!(capabilities.contains(&"streaming".to_string()));

        // Test rate limits
        let rate_limits = metadata.rate_limits();
        assert!(rate_limits.requests_per_minute > 0);
        assert!(rate_limits.input_tokens_per_minute > 0);
        assert!(rate_limits.output_tokens_per_minute > 0);
        assert!(rate_limits.max_concurrent_requests > 0);

        // Test custom attributes
        let attributes = metadata.custom_attributes();
        assert!(attributes.contains_key("provider_type"));
        assert_eq!(attributes.get("provider_type"), Some(&"mock".to_string()));
    }

    /// Test metadata consistency across multiple calls
    #[tokio::test]
    async fn test_metadata_consistency() {
        let provider = MockProvider::new("consistency-test");

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
    }

    /// Test metadata builder pattern
    #[tokio::test]
    async fn test_metadata_builder_pattern() {
        let metadata = ProviderMetadata::new("builder-test".to_string(), "2.0.0".to_string())
            .with_capabilities(vec!["research".to_string(), "chat".to_string(), "streaming".to_string()])
            .with_models(vec!["model-a".to_string(), "model-b".to_string(), "model-c".to_string()])
            .with_context_length(32768)
            .with_streaming(true)
            .with_attribute("region".to_string(), "us-east-1".to_string())
            .with_attribute("tier".to_string(), "premium".to_string());

        assert_eq!(metadata.name(), "builder-test");
        assert_eq!(metadata.version(), "2.0.0");
        assert_eq!(metadata.capabilities().len(), 3);
        assert_eq!(metadata.supported_models().len(), 3);
        assert_eq!(metadata.max_context_length(), 32768);
        assert!(metadata.supports_streaming());
        assert_eq!(metadata.custom_attributes().get("region"), Some(&"us-east-1".to_string()));
        assert_eq!(metadata.custom_attributes().get("tier"), Some(&"premium".to_string()));
    }
}

mod health_check_tests {
    use super::*;

    /// Test health check status reporting
    #[tokio::test]
    async fn test_health_check_status_reporting() {
        // Test healthy provider
        let healthy_provider = MockProvider::new("healthy").with_health(true);
        let health = healthy_provider.health_check().await.unwrap();
        assert_eq!(health, HealthStatus::Healthy);

        // Test unhealthy provider
        let unhealthy_provider = MockProvider::new("unhealthy").with_health(false);
        let health = unhealthy_provider.health_check().await.unwrap();
        assert!(matches!(health, HealthStatus::Unhealthy(_)));

        // Test degraded provider (rate limited)
        let degraded_provider = MockProvider::new("degraded").with_rate_limit_exceeded(true);
        let health = degraded_provider.health_check().await.unwrap();
        assert!(matches!(health, HealthStatus::Degraded(_)));

        // Test degraded provider (quota exceeded)
        let quota_degraded_provider = MockProvider::new("quota-degraded").with_quota_exceeded(true);
        let health = quota_degraded_provider.health_check().await.unwrap();
        assert!(matches!(health, HealthStatus::Degraded(_)));
    }

    /// Test health check error scenarios
    #[tokio::test]
    async fn test_health_check_error_scenarios() {
        // Test service unavailable
        let unavailable_provider = MockProvider::new("unavailable").with_service_unavailable(true);
        let health = unavailable_provider.health_check().await.unwrap();
        assert!(matches!(health, HealthStatus::Unhealthy(_)));

        // Test authentication failure
        let auth_failing_provider = MockProvider::new("auth-fail").with_auth_failure(true);
        let health = auth_failing_provider.health_check().await.unwrap();
        assert!(matches!(health, HealthStatus::Unhealthy(_)));
    }

    /// Test health check performance and timing
    #[tokio::test]
    async fn test_health_check_performance() {
        let provider = MockProvider::new("perf-health");
        
        let start = Instant::now();
        let health = provider.health_check().await;
        let duration = start.elapsed();
        
        assert!(health.is_ok());
        assert!(duration < Duration::from_secs(1)); // Health checks should be fast
    }
}

mod cost_estimation_tests {
    use super::*;

    /// Test cost estimation accuracy and consistency
    #[tokio::test]
    async fn test_cost_estimation_accuracy() {
        let provider = MockProvider::new("cost-test").with_pricing(0.001, 0.002);
        
        let test_cases = vec![
            ("short", 5), // Short query
            ("This is a medium length query for testing", 12),
            ("This is a much longer query that should result in higher token estimates and costs", 20),
        ];

        for (query, expected_min_tokens) in test_cases {
            let cost = provider.estimate_cost(query).await.unwrap();
            
            assert!(cost.estimated_input_tokens >= expected_min_tokens);
            assert!(cost.estimated_output_tokens > 0);
            assert!(cost.estimated_duration > Duration::ZERO);
            assert!(cost.estimated_cost_usd.is_some());
            assert!(cost.estimated_cost_usd.unwrap() > 0.0);
            
            // Verify cost calculation makes sense
            let expected_cost = (cost.estimated_input_tokens as f64 * 0.001) + 
                              (cost.estimated_output_tokens as f64 * 0.002);
            let actual_cost = cost.estimated_cost_usd.unwrap();
            assert!((actual_cost - expected_cost).abs() < 0.0001); // Floating point tolerance
        }
    }

    /// Test cost estimation with different pricing models
    #[tokio::test]
    async fn test_cost_estimation_different_pricing() {
        let cheap_provider = MockProvider::new("cheap").with_pricing(0.0001, 0.0001);
        let expensive_provider = MockProvider::new("expensive").with_pricing(0.01, 0.02);

        let query = "Same query for both providers";
        
        let cheap_cost = cheap_provider.estimate_cost(query).await.unwrap();
        let expensive_cost = expensive_provider.estimate_cost(query).await.unwrap();

        // Token estimates should be similar
        assert_eq!(cheap_cost.estimated_input_tokens, expensive_cost.estimated_input_tokens);
        assert_eq!(cheap_cost.estimated_output_tokens, expensive_cost.estimated_output_tokens);

        // But costs should be different
        assert!(expensive_cost.estimated_cost_usd.unwrap() > cheap_cost.estimated_cost_usd.unwrap());
    }

    /// Test cost estimation edge cases
    #[tokio::test]
    async fn test_cost_estimation_edge_cases() {
        let provider = MockProvider::new("edge-test").with_pricing(0.001, 0.002);

        // Empty query
        let empty_cost = provider.estimate_cost("").await.unwrap();
        assert!(empty_cost.estimated_input_tokens >= 0);
        assert!(empty_cost.estimated_output_tokens >= 0);

        // Very long query
        let long_query = "a".repeat(10000);
        let long_cost = provider.estimate_cost(&long_query).await.unwrap();
        assert!(long_cost.estimated_input_tokens > empty_cost.estimated_input_tokens);
        assert!(long_cost.estimated_cost_usd.unwrap() > empty_cost.estimated_cost_usd.unwrap_or(0.0));
    }
}

mod usage_statistics_tests {
    use super::*;

    /// Test usage statistics tracking and accuracy
    #[tokio::test]
    async fn test_usage_statistics_tracking() {
        let provider = MockProvider::new("usage-test");

        // Initial stats should be zero
        let initial_stats = provider.usage_stats().await.unwrap();
        assert_eq!(initial_stats.total_requests, 0);
        assert_eq!(initial_stats.successful_requests, 0);
        assert_eq!(initial_stats.failed_requests, 0);
        assert_eq!(initial_stats.total_input_tokens, 0);
        assert_eq!(initial_stats.total_output_tokens, 0);
        assert!(initial_stats.last_request_time.is_none());

        // Make successful requests
        for i in 0..3 {
            let query = format!("Query {}", i);
            let result = provider.research_query(query).await;
            assert!(result.is_ok());
        }

        // Check updated stats
        let updated_stats = provider.usage_stats().await.unwrap();
        assert_eq!(updated_stats.total_requests, 3);
        assert_eq!(updated_stats.successful_requests, 3);
        assert_eq!(updated_stats.failed_requests, 0);
        assert!(updated_stats.total_input_tokens > 0);
        assert!(updated_stats.total_output_tokens > 0);
        assert!(updated_stats.last_request_time.is_some());
        assert!(updated_stats.average_response_time > Duration::ZERO);
    }

    /// Test usage statistics with failures
    #[tokio::test]
    async fn test_usage_statistics_with_failures() {
        let failing_provider = MockProvider::new("failure-stats").with_failure(true);

        // Make failed requests
        for i in 0..2 {
            let query = format!("Failing query {}", i);
            let result = failing_provider.research_query(query).await;
            assert!(result.is_err());
        }

        // Check stats include failures
        let stats = failing_provider.usage_stats().await.unwrap();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 2);
    }

    /// Test usage statistics reset and isolation
    #[tokio::test]
    async fn test_usage_statistics_isolation() {
        let provider1 = MockProvider::new("isolated-1");
        let provider2 = MockProvider::new("isolated-2");

        // Make requests to first provider
        let _result1 = provider1.research_query("Query for provider 1".to_string()).await;

        // Make requests to second provider
        let _result2 = provider2.research_query("Query for provider 2".to_string()).await;

        // Check stats are isolated
        let stats1 = provider1.usage_stats().await.unwrap();
        let stats2 = provider2.usage_stats().await.unwrap();

        assert_eq!(stats1.total_requests, 1);
        assert_eq!(stats2.total_requests, 1);
        // Each provider should only track its own requests
    }
}

mod validation_tests {
    use super::*;

    /// Test query validation functionality
    #[tokio::test]
    async fn test_query_validation() {
        let provider = MockProvider::new("validation-test");

        // Valid query should pass
        let valid_result = provider.validate_query("This is a valid query");
        assert!(valid_result.is_ok());

        // Empty query should fail
        let empty_result = provider.validate_query("");
        assert!(empty_result.is_err());
        assert!(matches!(empty_result.unwrap_err(), ProviderError::ConfigurationError { .. }));

        // Whitespace-only query should fail
        let whitespace_result = provider.validate_query("   \t\n  ");
        assert!(whitespace_result.is_err());
        assert!(matches!(whitespace_result.unwrap_err(), ProviderError::ConfigurationError { .. }));

        // Query with content should pass
        let content_result = provider.validate_query("  Valid query with whitespace  ");
        assert!(content_result.is_ok());
    }

    /// Test custom validation logic
    #[tokio::test]
    async fn test_custom_validation_logic() {
        // Create a custom provider with additional validation
        struct CustomValidatingProvider {
            inner: MockProvider,
        }

        #[async_trait]
        impl Provider for CustomValidatingProvider {
            async fn research_query(&self, query: String) -> ProviderResult<String> {
                self.inner.research_query(query).await
            }

            fn metadata(&self) -> ProviderMetadata {
                self.inner.metadata()
            }

            async fn health_check(&self) -> ProviderResult<HealthStatus> {
                self.inner.health_check().await
            }

            fn validate_query(&self, query: &str) -> ProviderResult<()> {
                // First run default validation
                self.inner.validate_query(query)?;
                
                // Additional custom validation
                if query.len() > 10000 {
                    return Err(ProviderError::ConfigurationError {
                        provider: self.metadata().name().to_string(),
                        message: "Query too long".to_string(),
                    });
                }

                if query.contains("forbidden") {
                    return Err(ProviderError::ConfigurationError {
                        provider: self.metadata().name().to_string(),
                        message: "Query contains forbidden content".to_string(),
                    });
                }

                Ok(())
            }

            async fn estimate_cost(&self, query: &str) -> ProviderResult<QueryCost> {
                self.inner.estimate_cost(query).await
            }

            async fn usage_stats(&self) -> ProviderResult<UsageStats> {
                self.inner.usage_stats().await
            }
        }

        let custom_provider = CustomValidatingProvider {
            inner: MockProvider::new("custom-validator"),
        };

        // Test custom validation rules
        let valid_result = custom_provider.validate_query("Short valid query");
        assert!(valid_result.is_ok());

        let too_long_result = custom_provider.validate_query(&"a".repeat(10001));
        assert!(too_long_result.is_err());

        let forbidden_result = custom_provider.validate_query("This contains forbidden content");
        assert!(forbidden_result.is_err());
    }
}

// Property-based tests for Provider trait
proptest! {
    #[test]
    fn test_provider_metadata_properties(
        name in "[a-zA-Z0-9-_]{1,50}",
        version in "[0-9]{1,2}\\.[0-9]{1,2}\\.[0-9]{1,2}"
    ) {
        let provider = MockProvider::new(&name).with_version(version.clone());
        let metadata = provider.metadata();
        
        prop_assert_eq!(metadata.name(), &name);
        prop_assert_eq!(metadata.version(), &version);
        prop_assert!(!metadata.capabilities().is_empty());
        prop_assert!(metadata.max_context_length() > 0);
    }

    #[test]
    fn test_cost_estimation_properties(
        query in ".*",
        input_cost in 0.0..1.0,
        output_cost in 0.0..1.0
    ) {
        tokio_test::block_on(async {
            let provider = MockProvider::new("prop-test")
                .with_pricing(input_cost, output_cost);
            
            let cost_result = provider.estimate_cost(&query).await;
            prop_assert!(cost_result.is_ok());
            
            let cost = cost_result.unwrap();
            prop_assert!(cost.estimated_input_tokens >= 0);
            prop_assert!(cost.estimated_output_tokens >= 0);
            prop_assert!(cost.estimated_duration >= Duration::ZERO);
            
            if let Some(cost_usd) = cost.estimated_cost_usd {
                prop_assert!(cost_usd >= 0.0);
            }
        });
    }

    #[test] 
    fn test_query_validation_properties(query in ".*") {
        let provider = MockProvider::new("prop-validation");
        let result = provider.validate_query(&query);
        
        if query.trim().is_empty() {
            prop_assert!(result.is_err());
        } else {
            prop_assert!(result.is_ok());
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Integration test combining multiple provider operations
    #[tokio::test]
    async fn test_provider_full_workflow_integration() {
        let provider = MockProvider::new("workflow-test")
            .with_delay(Duration::from_millis(50))
            .with_pricing(0.001, 0.002);

        // 1. Check provider health
        let health = provider.health_check().await;
        assert!(health.is_ok());
        assert_eq!(health.unwrap(), HealthStatus::Healthy);

        // 2. Validate query
        let query = "Integration test query";
        let validation = provider.validate_query(query);
        assert!(validation.is_ok());

        // 3. Estimate cost
        let cost = provider.estimate_cost(query).await;
        assert!(cost.is_ok());
        let cost_estimate = cost.unwrap();

        // 4. Execute query
        let start_time = Instant::now();
        let result = provider.research_query(query.to_string()).await;
        let actual_duration = start_time.elapsed();
        assert!(result.is_ok());

        // 5. Check usage stats
        let stats = provider.usage_stats().await;
        assert!(stats.is_ok());
        let usage_stats = stats.unwrap();
        assert_eq!(usage_stats.total_requests, 1);
        assert_eq!(usage_stats.successful_requests, 1);

        // 6. Verify cost estimate was reasonable
        assert!(actual_duration <= cost_estimate.estimated_duration + Duration::from_millis(100));

        // 7. Get final metadata
        let metadata = provider.metadata();
        assert_eq!(metadata.name(), "workflow-test");
    }

    /// Test provider behavior under stress
    #[tokio::test]
    async fn test_provider_stress_integration() {
        let provider = Arc::new(MockProvider::new("stress-test"));
        let mut handles = Vec::new();

        // Launch many concurrent operations
        for i in 0..50 {
            let provider_clone = Arc::clone(&provider);
            let handle = tokio::spawn(async move {
                let query = format!("Stress test query {}", i);
                
                // Mix of operations
                match i % 3 {
                    0 => {
                        let result = provider_clone.research_query(query).await;
                        result.is_ok()
                    }
                    1 => {
                        let result = provider_clone.health_check().await;
                        result.is_ok()
                    }
                    2 => {
                        let result = provider_clone.estimate_cost(&query).await;
                        result.is_ok()
                    }
                    _ => unreachable!(),
                }
            });
            handles.push(handle);
        }

        // Wait for all operations
        let mut success_count = 0;
        for handle in handles {
            let result = handle.await.unwrap();
            if result {
                success_count += 1;
            }
        }

        // Should have high success rate
        assert!(success_count >= 45); // 90% success rate minimum

        // Check final stats
        let stats = provider.usage_stats().await.unwrap();
        assert!(stats.total_requests > 0);
    }
}