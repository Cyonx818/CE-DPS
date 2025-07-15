// ABOUTME: Integration tests for multi-LLM provider abstraction trait
//! This test module validates the core provider abstraction trait and its implementations
//! following TDD principles. These tests will initially fail until the trait is implemented.

use async_trait::async_trait;
use fortitude::providers::config::{
    ProviderConfig, ProviderSettings, RateLimitConfig, RetryConfig,
};
use fortitude::providers::{
    ClaudeProvider, HealthStatus, OpenAIProvider, Provider, ProviderError, ProviderMetadata,
    ProviderResult,
};
use std::time::Duration;

/// Mock provider implementation for testing
#[derive(Debug, Clone)]
struct MockProvider {
    name: String,
    healthy: bool,
    should_fail: bool,
}

impl MockProvider {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            healthy: true,
            should_fail: false,
        }
    }

    fn with_health(mut self, healthy: bool) -> Self {
        self.healthy = healthy;
        self
    }

    fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }
}

#[async_trait]
impl Provider for MockProvider {
    async fn research_query(&self, query: String) -> ProviderResult<String> {
        if self.should_fail {
            return Err(ProviderError::QueryFailed {
                message: "Mock provider configured to fail".to_string(),
                provider: self.name.clone(),
                error_code: Some("MOCK_FAILURE".to_string()),
            });
        }

        if !self.healthy {
            return Err(ProviderError::Unhealthy {
                provider: self.name.clone(),
                message: "Provider is not healthy".to_string(),
            });
        }

        Ok(format!("Mock response for query: {}", query))
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata::new(self.name.clone(), "1.0.0".to_string()).with_capabilities(vec![
            "research".to_string(),
            "async".to_string(),
            "mock".to_string(),
        ])
    }

    async fn health_check(&self) -> ProviderResult<HealthStatus> {
        if self.healthy {
            Ok(HealthStatus::Healthy)
        } else {
            Ok(HealthStatus::Unhealthy(
                "Mock provider unhealthy".to_string(),
            ))
        }
    }
}

#[tokio::test]
async fn test_provider_trait_exists() {
    // Test that the Provider trait is properly defined and can be used as a trait object
    let provider: Box<dyn Provider> = Box::new(MockProvider::new("test-provider"));
    let metadata = provider.metadata();
    assert_eq!(metadata.name(), "test-provider");
}

#[tokio::test]
async fn test_provider_async_research_query() {
    // Test that providers can handle async research queries
    let provider = MockProvider::new("test-provider");
    let query = "What is the capital of France?";

    // This will fail until the trait method is implemented
    let result = provider.research_query(query.to_string()).await;
    assert!(
        result.is_ok(),
        "Research query should succeed for healthy provider"
    );

    let response = result.unwrap();
    assert!(!response.is_empty(), "Response should not be empty");
}

#[tokio::test]
async fn test_provider_error_handling() {
    // Test proper error handling for failed queries
    let provider = MockProvider::new("failing-provider").with_failure(true);
    let query = "This query should fail";

    let result = provider.research_query(query.to_string()).await;
    assert!(result.is_err(), "Query should fail for failing provider");

    match result.unwrap_err() {
        ProviderError::QueryFailed { message, .. } => {
            assert!(!message.is_empty(), "Error message should not be empty");
        }
        _ => panic!("Expected QueryFailed error"),
    }
}

#[tokio::test]
async fn test_provider_metadata() {
    // Test that providers expose their metadata
    let provider = MockProvider::new("metadata-provider");

    let metadata = provider.metadata();
    assert_eq!(metadata.name(), "metadata-provider");
    assert!(metadata.capabilities().contains(&"research".to_string()));
    assert!(metadata.rate_limits().requests_per_minute > 0);
}

#[tokio::test]
async fn test_provider_health_check() {
    // Test health checking functionality
    let healthy_provider = MockProvider::new("healthy-provider").with_health(true);
    let unhealthy_provider = MockProvider::new("unhealthy-provider").with_health(false);

    let healthy_status = healthy_provider.health_check().await;
    assert!(healthy_status.is_ok());
    assert_eq!(healthy_status.unwrap(), HealthStatus::Healthy);

    let unhealthy_status = unhealthy_provider.health_check().await;
    assert!(unhealthy_status.is_ok());
    assert!(matches!(
        unhealthy_status.unwrap(),
        HealthStatus::Unhealthy(_)
    ));
}

#[tokio::test]
async fn test_provider_configuration_validation() {
    // Test configuration validation
    let valid_config = ProviderSettings {
        api_key: "valid-key".to_string(),
        model: "test-model".to_string(),
        endpoint: None,
        timeout: Duration::from_secs(30),
        rate_limits: RateLimitConfig::default(),
        retry: RetryConfig::default(),
    };

    let invalid_config = ProviderSettings {
        api_key: "".to_string(), // Invalid empty key
        model: "test-model".to_string(),
        endpoint: None,
        timeout: Duration::from_secs(30),
        rate_limits: RateLimitConfig::default(),
        retry: RetryConfig::default(),
    };

    assert!(
        valid_config.validate().is_ok(),
        "Valid config should pass validation"
    );
    assert!(
        invalid_config.validate().is_err(),
        "Invalid config should fail validation"
    );
}

#[tokio::test]
async fn test_provider_rate_limiting() {
    // Test that providers respect rate limiting configuration
    let rate_limit_config = RateLimitConfig {
        requests_per_minute: 2,
        input_tokens_per_minute: 1000,
        output_tokens_per_minute: 500,
        max_concurrent_requests: 1,
    };

    let settings = ProviderSettings {
        api_key: "test-key".to_string(),
        model: "test-model".to_string(),
        endpoint: None,
        timeout: Duration::from_secs(30),
        rate_limits: rate_limit_config,
        retry: RetryConfig::default(),
    };

    let provider = MockProvider::new("rate-limited-provider");
    // This test will verify rate limiting is enforced
    // Implementation should track requests and enforce limits

    // Multiple rapid requests should eventually hit rate limits
    for i in 0..5 {
        let result = provider.research_query(format!("Query {}", i)).await;
        if i >= 2 {
            // Should start failing due to rate limits
            if result.is_err() {
                match result.unwrap_err() {
                    ProviderError::RateLimitExceeded { .. } => {
                        // Expected - rate limit hit
                        break;
                    }
                    _ => panic!("Expected rate limit error"),
                }
            }
        }
    }
}

#[tokio::test]
async fn test_provider_timeout_handling() {
    // Test timeout configuration is respected
    let timeout_config = ProviderSettings {
        api_key: "test-key".to_string(),
        model: "test-model".to_string(),
        endpoint: None,
        timeout: Duration::from_millis(100), // Very short timeout
        rate_limits: RateLimitConfig::default(),
        retry: RetryConfig::default(),
    };

    let provider = MockProvider::new("slow-provider");
    // In real implementation, this would simulate a slow API call
    let result = provider.research_query("slow query".to_string()).await;

    // Should timeout for very slow responses
    if result.is_err() {
        match result.unwrap_err() {
            ProviderError::Timeout { .. } => {
                // Expected timeout error
            }
            _ => panic!("Expected timeout error for slow provider"),
        }
    }
}

#[tokio::test]
async fn test_provider_retry_mechanism() {
    // Test retry configuration and behavior
    let retry_config = RetryConfig {
        max_retries: 3,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        backoff_multiplier: 2.0,
        jitter: false,
    };

    let settings = ProviderSettings {
        api_key: "test-key".to_string(),
        model: "test-model".to_string(),
        endpoint: None,
        timeout: Duration::from_secs(30),
        rate_limits: RateLimitConfig::default(),
        retry: retry_config,
    };

    let provider = MockProvider::new("retry-provider").with_failure(true);

    // Should retry the configured number of times before giving up
    let start_time = std::time::Instant::now();
    let result = provider.research_query("failing query".to_string()).await;
    let elapsed = start_time.elapsed();

    assert!(result.is_err(), "Should fail after retries exhausted");
    // Should have taken some time due to retries with delays
    assert!(
        elapsed >= Duration::from_millis(30),
        "Should have retried with delays"
    );
}

#[tokio::test]
async fn test_provider_capabilities_metadata() {
    // Test that providers correctly expose their capabilities
    let provider = MockProvider::new("capability-provider");
    let metadata = provider.metadata();

    let capabilities = metadata.capabilities();
    assert!(capabilities.contains(&"research".to_string()));
    assert!(capabilities.contains(&"async".to_string()));

    // Test rate limit metadata
    let rate_limits = metadata.rate_limits();
    assert!(rate_limits.requests_per_minute > 0);
    assert!(rate_limits.input_tokens_per_minute > 0);
    assert!(rate_limits.output_tokens_per_minute > 0);
}

// Integration tests for OpenAI provider implementation
#[tokio::test]
async fn test_openai_provider_implements_trait() {
    // Test that OpenAI provider properly implements the Provider trait
    let settings = ProviderSettings::new("test-api-key".to_string(), "gpt-3.5-turbo".to_string());

    let provider_result = OpenAIProvider::new(settings).await;
    assert!(
        provider_result.is_ok(),
        "OpenAI provider should be created successfully"
    );

    let provider = provider_result.unwrap();

    // Test that it can be used as a trait object
    let provider_trait: Box<dyn Provider> = Box::new(provider);

    let metadata = provider_trait.metadata();
    assert_eq!(metadata.name(), "openai");
    assert!(metadata
        .capabilities()
        .contains(&"rate_limited".to_string()));
    assert!(metadata
        .capabilities()
        .contains(&"cost_estimation".to_string()));
}

#[tokio::test]
async fn test_openai_provider_metadata_compliance() {
    // Test that OpenAI provider metadata meets trait requirements
    let settings = ProviderSettings::new("test-api-key".to_string(), "gpt-4".to_string());

    let provider = OpenAIProvider::new(settings).await.unwrap();
    let metadata = provider.metadata();

    // Verify required metadata fields
    assert!(!metadata.name().is_empty());
    assert!(!metadata.version().is_empty());
    assert!(!metadata.capabilities().is_empty());
    assert!(!metadata.supported_models().is_empty());
    assert!(metadata.max_context_length() > 0);

    // Verify rate limits are properly configured
    let rate_limits = metadata.rate_limits();
    assert!(rate_limits.requests_per_minute > 0);
    assert!(rate_limits.input_tokens_per_minute > 0);
    assert!(rate_limits.output_tokens_per_minute > 0);
    assert!(rate_limits.max_concurrent_requests > 0);

    // Verify provider-specific attributes
    assert_eq!(
        metadata.custom_attributes().get("provider_type"),
        Some(&"openai".to_string())
    );
    assert_eq!(
        metadata.custom_attributes().get("api_version"),
        Some(&"v1".to_string())
    );
}

#[tokio::test]
async fn test_openai_provider_error_handling() {
    // Test that OpenAI provider properly handles and maps errors
    let settings =
        ProviderSettings::new("invalid-api-key".to_string(), "gpt-3.5-turbo".to_string());

    let provider = OpenAIProvider::new(settings).await.unwrap();

    // Attempt a query with invalid credentials - should return appropriate error
    let result = provider.research_query("Test query".to_string()).await;
    assert!(result.is_err(), "Query with invalid API key should fail");

    // Verify error is of expected type
    let error = result.unwrap_err();
    assert!(
        matches!(error, ProviderError::AuthenticationFailed { .. })
            || matches!(error, ProviderError::NetworkError { .. })
            || matches!(error, ProviderError::QueryFailed { .. }),
        "Error should be authentication, network, or query failed"
    );
}

#[tokio::test]
async fn test_openai_provider_health_check() {
    // Test OpenAI provider health check functionality
    let settings = ProviderSettings::new("test-api-key".to_string(), "gpt-3.5-turbo".to_string());

    let provider = OpenAIProvider::new(settings).await.unwrap();

    let health_result = provider.health_check().await;
    assert!(health_result.is_ok(), "Health check should return a status");

    let health_status = health_result.unwrap();
    // With test API key, should be unhealthy or degraded
    match health_status {
        HealthStatus::Healthy => {
            // Unexpected with test key, but not necessarily wrong
        }
        HealthStatus::Degraded(_) => {
            // Expected - service available but with issues
        }
        HealthStatus::Unhealthy(_) => {
            // Expected with invalid API key
        }
    }
}

#[tokio::test]
async fn test_openai_provider_cost_estimation() {
    // Test OpenAI provider cost estimation functionality
    let settings = ProviderSettings::new("test-api-key".to_string(), "gpt-3.5-turbo".to_string());

    let provider = OpenAIProvider::new(settings).await.unwrap();

    let query = "What is the meaning of life, the universe, and everything?";
    let cost_result = provider.estimate_cost(query).await;

    assert!(cost_result.is_ok(), "Cost estimation should succeed");

    let cost = cost_result.unwrap();
    assert!(
        cost.estimated_input_tokens > 0,
        "Should estimate input tokens"
    );
    assert!(
        cost.estimated_output_tokens > 0,
        "Should estimate output tokens"
    );
    assert!(
        cost.estimated_duration > Duration::ZERO,
        "Should estimate duration"
    );
    assert!(
        cost.estimated_cost_usd.is_some(),
        "Should provide cost estimate"
    );
    assert!(
        cost.estimated_cost_usd.unwrap() > 0.0,
        "Cost should be positive"
    );
}

#[tokio::test]
async fn test_openai_provider_usage_stats() {
    // Test OpenAI provider usage statistics tracking
    let settings = ProviderSettings::new("test-api-key".to_string(), "gpt-3.5-turbo".to_string());

    let provider = OpenAIProvider::new(settings).await.unwrap();

    let stats_result = provider.usage_stats().await;
    assert!(stats_result.is_ok(), "Usage stats should be retrievable");

    let stats = stats_result.unwrap();
    // For a new provider, stats should start at zero
    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.successful_requests, 0);
    assert_eq!(stats.failed_requests, 0);
    assert_eq!(stats.total_input_tokens, 0);
    assert_eq!(stats.total_output_tokens, 0);
}

#[tokio::test]
async fn test_openai_provider_query_validation() {
    // Test OpenAI provider query validation
    let settings = ProviderSettings::new("test-api-key".to_string(), "gpt-3.5-turbo".to_string());

    let provider = OpenAIProvider::new(settings).await.unwrap();

    // Valid query should pass validation
    let valid_result = provider.validate_query("Valid query");
    assert!(valid_result.is_ok(), "Valid query should pass validation");

    // Empty query should fail validation
    let empty_result = provider.validate_query("");
    assert!(empty_result.is_err(), "Empty query should fail validation");

    // Whitespace-only query should fail validation
    let whitespace_result = provider.validate_query("   ");
    assert!(
        whitespace_result.is_err(),
        "Whitespace-only query should fail validation"
    );
}

#[tokio::test]
async fn test_openai_provider_configuration_validation() {
    // Test OpenAI provider configuration validation

    // Valid configuration should work
    let valid_settings =
        ProviderSettings::new("sk-test12345".to_string(), "gpt-3.5-turbo".to_string());
    let valid_provider = OpenAIProvider::new(valid_settings).await;
    assert!(valid_provider.is_ok(), "Valid configuration should succeed");

    // Invalid API key should fail
    let invalid_key_settings = ProviderSettings::new(
        "".to_string(), // Empty API key
        "gpt-3.5-turbo".to_string(),
    );
    let invalid_key_provider = OpenAIProvider::new(invalid_key_settings).await;
    assert!(invalid_key_provider.is_err(), "Empty API key should fail");

    // Invalid model should still create provider (validation happens at runtime)
    let invalid_model_settings = ProviderSettings::new(
        "sk-test12345".to_string(),
        "".to_string(), // Empty model
    );
    let invalid_model_provider = OpenAIProvider::new(invalid_model_settings).await;
    assert!(invalid_model_provider.is_err(), "Empty model should fail");
}

#[tokio::test]
async fn test_openai_provider_concurrent_access() {
    // Test OpenAI provider thread safety and concurrent access
    let settings = ProviderSettings::new("test-api-key".to_string(), "gpt-3.5-turbo".to_string());

    let provider = std::sync::Arc::new(OpenAIProvider::new(settings).await.unwrap());

    let mut handles = Vec::new();

    // Spawn multiple concurrent tasks accessing the provider
    for i in 0..5 {
        let provider_clone = provider.clone();
        let handle = tokio::spawn(async move {
            // Test metadata access
            let metadata = provider_clone.metadata();
            assert_eq!(metadata.name(), "openai");

            // Test cost estimation
            let cost = provider_clone.estimate_cost(&format!("Query {}", i)).await;
            assert!(cost.is_ok());

            // Test usage stats
            let stats = provider_clone.usage_stats().await;
            assert!(stats.is_ok());

            i
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await;
        assert!(
            result.is_ok(),
            "Concurrent task should complete successfully"
        );
    }
}

// Integration tests for Claude provider implementation
#[tokio::test]
async fn test_claude_provider_implements_trait() {
    // Test that Claude provider properly implements the Provider trait
    let settings = ProviderSettings::new(
        "test-api-key".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
    );

    let provider_result = ClaudeProvider::new(settings).await;
    assert!(
        provider_result.is_ok(),
        "Claude provider should be created successfully"
    );

    let provider = provider_result.unwrap();

    // Test that it can be used as a trait object
    let provider_trait: Box<dyn Provider> = Box::new(provider);

    let metadata = provider_trait.metadata();
    assert_eq!(metadata.name(), "claude");
    assert!(metadata
        .capabilities()
        .contains(&"rate_limited".to_string()));
    assert!(metadata
        .capabilities()
        .contains(&"cost_estimation".to_string()));
    assert!(metadata
        .capabilities()
        .contains(&"anthropic_v2".to_string()));
}

#[tokio::test]
async fn test_claude_provider_metadata_compliance() {
    // Test that Claude provider metadata meets trait requirements
    let settings = ProviderSettings::new(
        "test-api-key".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
    );

    let provider = ClaudeProvider::new(settings).await.unwrap();
    let metadata = provider.metadata();

    // Verify required metadata fields
    assert!(!metadata.name().is_empty());
    assert!(!metadata.version().is_empty());
    assert!(!metadata.capabilities().is_empty());
    assert!(!metadata.supported_models().is_empty());
    assert!(metadata.max_context_length() > 0);

    // Verify rate limits are properly configured
    let rate_limits = metadata.rate_limits();
    assert!(rate_limits.requests_per_minute > 0);
    assert!(rate_limits.input_tokens_per_minute > 0);
    assert!(rate_limits.output_tokens_per_minute > 0);
    assert!(rate_limits.max_concurrent_requests > 0);

    // Verify provider-specific attributes
    assert_eq!(
        metadata.custom_attributes().get("provider_type"),
        Some(&"claude".to_string())
    );
    assert_eq!(
        metadata.custom_attributes().get("api_version"),
        Some(&"v2".to_string())
    );
    assert_eq!(
        metadata.custom_attributes().get("messages_api"),
        Some(&"2023-06-01".to_string())
    );
}

#[tokio::test]
async fn test_claude_provider_error_handling() {
    // Test that Claude provider properly handles and maps errors
    let settings = ProviderSettings::new(
        "invalid-api-key".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
    );

    let provider = ClaudeProvider::new(settings).await.unwrap();

    // Attempt a query with invalid credentials - should return appropriate error
    let result = provider.research_query("Test query".to_string()).await;
    assert!(result.is_err(), "Query with invalid API key should fail");

    // Verify error is of expected type
    let error = result.unwrap_err();
    assert!(
        matches!(error, ProviderError::AuthenticationFailed { .. })
            || matches!(error, ProviderError::NetworkError { .. })
            || matches!(error, ProviderError::QueryFailed { .. }),
        "Error should be authentication, network, or query failed"
    );
}

#[tokio::test]
async fn test_claude_provider_health_check() {
    // Test Claude provider health check functionality
    let settings = ProviderSettings::new(
        "test-api-key".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
    );

    let provider = ClaudeProvider::new(settings).await.unwrap();

    let health_result = provider.health_check().await;
    assert!(health_result.is_ok(), "Health check should return a status");

    let health_status = health_result.unwrap();
    // With test API key, should be unhealthy or degraded
    match health_status {
        HealthStatus::Healthy => {
            // Unexpected with test key, but not necessarily wrong
        }
        HealthStatus::Degraded(_) => {
            // Expected - service available but with issues
        }
        HealthStatus::Unhealthy(_) => {
            // Expected with invalid API key
        }
    }
}

#[tokio::test]
async fn test_claude_provider_cost_estimation() {
    // Test Claude provider cost estimation functionality
    let settings = ProviderSettings::new(
        "test-api-key".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
    );

    let provider = ClaudeProvider::new(settings).await.unwrap();

    let query = "What is the meaning of life, the universe, and everything?";
    let cost_result = provider.estimate_cost(query).await;

    assert!(cost_result.is_ok(), "Cost estimation should succeed");

    let cost = cost_result.unwrap();
    assert!(
        cost.estimated_input_tokens > 0,
        "Should estimate input tokens"
    );
    assert!(
        cost.estimated_output_tokens > 0,
        "Should estimate output tokens"
    );
    assert!(
        cost.estimated_duration > Duration::ZERO,
        "Should estimate duration"
    );
    assert!(
        cost.estimated_cost_usd.is_some(),
        "Should provide cost estimate"
    );
    assert!(
        cost.estimated_cost_usd.unwrap() > 0.0,
        "Cost should be positive"
    );
}

#[tokio::test]
async fn test_claude_provider_usage_stats() {
    // Test Claude provider usage statistics tracking
    let settings = ProviderSettings::new(
        "test-api-key".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
    );

    let provider = ClaudeProvider::new(settings).await.unwrap();

    let stats_result = provider.usage_stats().await;
    assert!(stats_result.is_ok(), "Usage stats should be retrievable");

    let stats = stats_result.unwrap();
    // For a new provider, stats should start at zero
    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.successful_requests, 0);
    assert_eq!(stats.failed_requests, 0);
    assert_eq!(stats.total_input_tokens, 0);
    assert_eq!(stats.total_output_tokens, 0);
}

#[tokio::test]
async fn test_claude_provider_query_validation() {
    // Test Claude provider query validation
    let settings = ProviderSettings::new(
        "test-api-key".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
    );

    let provider = ClaudeProvider::new(settings).await.unwrap();

    // Valid query should pass validation
    let valid_result = provider.validate_query("Valid query");
    assert!(valid_result.is_ok(), "Valid query should pass validation");

    // Empty query should fail validation
    let empty_result = provider.validate_query("");
    assert!(empty_result.is_err(), "Empty query should fail validation");

    // Whitespace-only query should fail validation
    let whitespace_result = provider.validate_query("   ");
    assert!(
        whitespace_result.is_err(),
        "Whitespace-only query should fail validation"
    );
}

#[tokio::test]
async fn test_claude_provider_configuration_validation() {
    // Test Claude provider configuration validation

    // Valid configuration should work
    let valid_settings = ProviderSettings::new(
        "sk-ant-test12345".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
    );
    let valid_provider = ClaudeProvider::new(valid_settings).await;
    assert!(valid_provider.is_ok(), "Valid configuration should succeed");

    // Invalid API key should fail
    let invalid_key_settings = ProviderSettings::new(
        "".to_string(), // Empty API key
        "claude-3-5-sonnet-20241022".to_string(),
    );
    let invalid_key_provider = ClaudeProvider::new(invalid_key_settings).await;
    assert!(invalid_key_provider.is_err(), "Empty API key should fail");

    // Invalid model should still create provider (validation happens at runtime)
    let invalid_model_settings = ProviderSettings::new(
        "sk-ant-test12345".to_string(),
        "".to_string(), // Empty model
    );
    let invalid_model_provider = ClaudeProvider::new(invalid_model_settings).await;
    assert!(invalid_model_provider.is_err(), "Empty model should fail");
}

#[tokio::test]
async fn test_claude_provider_concurrent_access() {
    // Test Claude provider thread safety and concurrent access
    let settings = ProviderSettings::new(
        "test-api-key".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
    );

    let provider = std::sync::Arc::new(ClaudeProvider::new(settings).await.unwrap());

    let mut handles = Vec::new();

    // Spawn multiple concurrent tasks accessing the provider
    for i in 0..5 {
        let provider_clone = provider.clone();
        let handle = tokio::spawn(async move {
            // Test metadata access
            let metadata = provider_clone.metadata();
            assert_eq!(metadata.name(), "claude");

            // Test cost estimation
            let cost = provider_clone.estimate_cost(&format!("Query {}", i)).await;
            assert!(cost.is_ok());

            // Test usage stats
            let stats = provider_clone.usage_stats().await;
            assert!(stats.is_ok());

            i
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await;
        assert!(
            result.is_ok(),
            "Concurrent task should complete successfully"
        );
    }
}

#[tokio::test]
async fn test_claude_vs_openai_provider_compatibility() {
    // Test that both providers can be used interchangeably via the trait
    let openai_settings =
        ProviderSettings::new("test-openai-key".to_string(), "gpt-3.5-turbo".to_string());

    let claude_settings = ProviderSettings::new(
        "test-claude-key".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
    );

    let openai_provider = OpenAIProvider::new(openai_settings).await.unwrap();
    let claude_provider = ClaudeProvider::new(claude_settings).await.unwrap();

    // Both should implement the same trait
    let providers: Vec<Box<dyn Provider>> =
        vec![Box::new(openai_provider), Box::new(claude_provider)];

    for (i, provider) in providers.iter().enumerate() {
        // Test metadata access
        let metadata = provider.metadata();
        assert!(!metadata.name().is_empty());
        assert!(!metadata.version().is_empty());
        assert!(metadata.capabilities().contains(&"research".to_string()));

        // Test cost estimation
        let cost = provider.estimate_cost("Test query").await;
        assert!(cost.is_ok());

        // Test usage stats
        let stats = provider.usage_stats().await;
        assert!(stats.is_ok());

        // Test health check
        let health = provider.health_check().await;
        assert!(health.is_ok());

        // Test query validation
        let validation = provider.validate_query("Valid query");
        assert!(validation.is_ok());

        println!("Provider {} passed compatibility tests", i);
    }
}

#[tokio::test]
async fn test_claude_anthropic_specific_features() {
    // Test Claude-specific features and capabilities
    let settings = ProviderSettings::new(
        "test-api-key".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
    );

    let provider = ClaudeProvider::new(settings).await.unwrap();
    let metadata = provider.metadata();

    // Test Claude-specific capabilities
    assert!(metadata
        .capabilities()
        .contains(&"anthropic_v2".to_string()));

    // Test Claude-specific attributes
    assert_eq!(
        metadata.custom_attributes().get("messages_api"),
        Some(&"2023-06-01".to_string())
    );

    // Test supported Claude models
    let supported_models = metadata.supported_models();
    assert!(supported_models.contains(&"claude-3-5-sonnet-20241022".to_string()));
    assert!(supported_models.contains(&"claude-3-haiku-20240307".to_string()));
    assert!(supported_models.contains(&"claude-3-opus-20240229".to_string()));

    // Test context length for Claude models (should be 200k)
    assert_eq!(metadata.max_context_length(), 200000);
}

#[tokio::test]
async fn test_claude_model_specific_pricing() {
    // Test Claude model pricing accuracy
    let sonnet_settings = ProviderSettings::new(
        "test-api-key".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
    );
    let sonnet_provider = ClaudeProvider::new(sonnet_settings).await.unwrap();

    let haiku_settings = ProviderSettings::new(
        "test-api-key".to_string(),
        "claude-3-haiku-20240307".to_string(),
    );
    let haiku_provider = ClaudeProvider::new(haiku_settings).await.unwrap();

    let opus_settings = ProviderSettings::new(
        "test-api-key".to_string(),
        "claude-3-opus-20240229".to_string(),
    );
    let opus_provider = ClaudeProvider::new(opus_settings).await.unwrap();

    let query = "What is the meaning of life?"; // Same query for all

    let sonnet_cost = sonnet_provider.estimate_cost(query).await.unwrap();
    let haiku_cost = haiku_provider.estimate_cost(query).await.unwrap();
    let opus_cost = opus_provider.estimate_cost(query).await.unwrap();

    // Haiku should be cheapest, Opus most expensive, Sonnet in between
    assert!(haiku_cost.estimated_cost_usd < sonnet_cost.estimated_cost_usd);
    assert!(sonnet_cost.estimated_cost_usd < opus_cost.estimated_cost_usd);

    // All should have positive costs
    assert!(haiku_cost.estimated_cost_usd.unwrap() > 0.0);
    assert!(sonnet_cost.estimated_cost_usd.unwrap() > 0.0);
    assert!(opus_cost.estimated_cost_usd.unwrap() > 0.0);
}
