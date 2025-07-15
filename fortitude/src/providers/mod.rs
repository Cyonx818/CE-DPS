// ABOUTME: Multi-LLM provider abstraction trait and core definitions
//! This module provides the core provider abstraction trait for multi-LLM support.
//! It defines async interfaces for research queries, error handling, provider metadata,
//! health checking, and rate limiting functionality.
//!
//! # Example Usage
//!
//! ```rust
//! use fortitude::providers::{Provider, ProviderResult};
//!
//! async fn example_usage<P: Provider>(provider: P) -> ProviderResult<String> {
//!     // Check provider health before use
//!     let health = provider.health_check().await?;
//!     match health {
//!         HealthStatus::Healthy => {
//!             provider.research_query("What is quantum computing?".to_string()).await
//!         }
//!         _ => Err(ProviderError::Unhealthy("Provider not available".to_string()))
//!     }
//! }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

pub mod claude;
pub mod config;
pub mod fallback;
pub mod gemini;
pub mod manager;
pub mod mock;
pub mod openai;

pub use claude::ClaudeProvider;
pub use config::*;
pub use fallback::{FallbackEngine, FallbackError, FallbackStrategy, HealthMonitor, RetryConfig};
pub use gemini::GeminiProvider;
pub use manager::{ProviderConfig, ProviderManager, ProviderManagerError, SelectionStrategy};
pub use openai::OpenAIProvider;

/// Result type for provider operations
pub type ProviderResult<T> = Result<T, ProviderError>;

/// Provider-specific error types with detailed context
#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Query failed: {message}")]
    QueryFailed {
        message: String,
        provider: String,
        error_code: Option<String>,
    },

    #[error("Rate limit exceeded for provider {provider}: {message}")]
    RateLimitExceeded {
        provider: String,
        message: String,
        retry_after: Option<Duration>,
        requests_remaining: Option<u32>,
        tokens_remaining: Option<u32>,
    },

    #[error("Authentication failed for provider {provider}: {message}")]
    AuthenticationFailed { provider: String, message: String },

    #[error("Request timeout after {duration:?} for provider {provider}")]
    Timeout {
        provider: String,
        duration: Duration,
    },

    #[error("Provider {provider} configuration invalid: {message}")]
    ConfigurationError { provider: String, message: String },

    #[error("Provider {provider} is unhealthy: {message}")]
    Unhealthy { provider: String, message: String },

    #[error("Network error for provider {provider}: {source}")]
    NetworkError {
        provider: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Serialization error for provider {provider}: {message}")]
    SerializationError { provider: String, message: String },

    #[error("Quota exceeded for provider {provider}: {message}")]
    QuotaExceeded {
        provider: String,
        message: String,
        reset_time: Option<chrono::DateTime<chrono::Utc>>,
    },

    #[error("Service unavailable for provider {provider}: {message}")]
    ServiceUnavailable {
        provider: String,
        message: String,
        estimated_recovery: Option<Duration>,
    },
}

impl ProviderError {
    /// Check if this error type is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ProviderError::RateLimitExceeded { .. }
                | ProviderError::Timeout { .. }
                | ProviderError::NetworkError { .. }
                | ProviderError::ServiceUnavailable { .. }
        )
    }

    /// Get retry delay if applicable
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            ProviderError::RateLimitExceeded { retry_after, .. } => *retry_after,
            ProviderError::ServiceUnavailable {
                estimated_recovery, ..
            } => *estimated_recovery,
            _ => None,
        }
    }

    /// Get the provider name associated with this error
    pub fn provider(&self) -> &str {
        match self {
            ProviderError::QueryFailed { provider, .. } => provider,
            ProviderError::RateLimitExceeded { provider, .. } => provider,
            ProviderError::AuthenticationFailed { provider, .. } => provider,
            ProviderError::Timeout { provider, .. } => provider,
            ProviderError::ConfigurationError { provider, .. } => provider,
            ProviderError::Unhealthy { provider, .. } => provider,
            ProviderError::NetworkError { provider, .. } => provider,
            ProviderError::SerializationError { provider, .. } => provider,
            ProviderError::QuotaExceeded { provider, .. } => provider,
            ProviderError::ServiceUnavailable { provider, .. } => provider,
        }
    }
}

/// Health status of a provider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// Provider is fully operational
    Healthy,
    /// Provider is operational but with degraded performance
    Degraded(String),
    /// Provider is not operational
    Unhealthy(String),
}

/// Rate limiting configuration for providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub input_tokens_per_minute: u32,
    pub output_tokens_per_minute: u32,
    pub max_concurrent_requests: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            input_tokens_per_minute: 50_000,
            output_tokens_per_minute: 10_000,
            max_concurrent_requests: 5,
        }
    }
}

/// Provider metadata containing capabilities and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    name: String,
    version: String,
    capabilities: Vec<String>,
    rate_limits: RateLimitConfig,
    supported_models: Vec<String>,
    max_context_length: usize,
    supports_streaming: bool,
    custom_attributes: HashMap<String, String>,
}

impl ProviderMetadata {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            capabilities: vec!["research".to_string(), "async".to_string()],
            rate_limits: RateLimitConfig::default(),
            supported_models: Vec::new(),
            max_context_length: 8192,
            supports_streaming: false,
            custom_attributes: HashMap::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    pub fn rate_limits(&self) -> &RateLimitConfig {
        &self.rate_limits
    }

    pub fn supported_models(&self) -> &[String] {
        &self.supported_models
    }

    pub fn max_context_length(&self) -> usize {
        self.max_context_length
    }

    pub fn supports_streaming(&self) -> bool {
        self.supports_streaming
    }

    pub fn custom_attributes(&self) -> &HashMap<String, String> {
        &self.custom_attributes
    }

    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn with_rate_limits(mut self, rate_limits: RateLimitConfig) -> Self {
        self.rate_limits = rate_limits;
        self
    }

    pub fn with_models(mut self, models: Vec<String>) -> Self {
        self.supported_models = models;
        self
    }

    pub fn with_context_length(mut self, length: usize) -> Self {
        self.max_context_length = length;
        self
    }

    pub fn with_streaming(mut self, streaming: bool) -> Self {
        self.supports_streaming = streaming;
        self
    }

    pub fn with_attribute(mut self, key: String, value: String) -> Self {
        self.custom_attributes.insert(key, value);
        self
    }
}

/// Core provider trait for multi-LLM abstraction
#[async_trait]
pub trait Provider: Send + Sync {
    /// Execute a research query against the provider
    async fn research_query(&self, query: String) -> ProviderResult<String>;

    /// Get provider metadata including capabilities and rate limits
    fn metadata(&self) -> ProviderMetadata;

    /// Check the health status of the provider
    async fn health_check(&self) -> ProviderResult<HealthStatus>;

    /// Validate a query before execution (optional, default implementation allows all)
    fn validate_query(&self, query: &str) -> ProviderResult<()> {
        if query.trim().is_empty() {
            return Err(ProviderError::ConfigurationError {
                provider: self.metadata().name().to_string(),
                message: "Query cannot be empty".to_string(),
            });
        }
        Ok(())
    }

    /// Estimate the cost/resource usage for a query
    async fn estimate_cost(&self, query: &str) -> ProviderResult<QueryCost> {
        // Default implementation provides basic estimation
        let estimated_tokens = query.len() / 4; // Rough token estimation
        Ok(QueryCost {
            estimated_input_tokens: estimated_tokens as u32,
            estimated_output_tokens: (estimated_tokens / 2) as u32,
            estimated_duration: Duration::from_secs(2),
            estimated_cost_usd: None,
        })
    }

    /// Get current usage statistics for the provider
    async fn usage_stats(&self) -> ProviderResult<UsageStats> {
        // Default implementation returns empty stats
        Ok(UsageStats::default())
    }
}

/// Cost estimation for a query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCost {
    pub estimated_input_tokens: u32,
    pub estimated_output_tokens: u32,
    pub estimated_duration: Duration,
    pub estimated_cost_usd: Option<f64>,
}

/// Usage statistics for a provider
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub average_response_time: Duration,
    pub last_request_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// Mock provider for testing trait implementations
    #[derive(Debug, Clone)]
    struct MockProvider {
        name: String,
        healthy: bool,
        should_fail: bool,
        response_delay: Duration,
    }

    impl MockProvider {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                healthy: true,
                should_fail: false,
                response_delay: Duration::from_millis(10),
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

        #[allow(dead_code)]
        fn with_delay(mut self, delay: Duration) -> Self {
            self.response_delay = delay;
            self
        }
    }

    #[async_trait]
    impl Provider for MockProvider {
        async fn research_query(&self, query: String) -> ProviderResult<String> {
            // Simulate response delay
            tokio::time::sleep(self.response_delay).await;

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
            ProviderMetadata::new(self.name.clone(), "1.0.0".to_string())
                .with_capabilities(vec![
                    "research".to_string(),
                    "async".to_string(),
                    "mock".to_string(),
                ])
                .with_models(vec!["mock-model-v1".to_string()])
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
    async fn test_mock_provider_successful_query() {
        let provider = MockProvider::new("test-provider");
        let result = provider.research_query("test query".to_string()).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("test query"));
    }

    #[tokio::test]
    async fn test_mock_provider_failed_query() {
        let provider = MockProvider::new("failing-provider").with_failure(true);
        let result = provider.research_query("test query".to_string()).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ProviderError::QueryFailed { .. }));
        assert_eq!(error.provider(), "failing-provider");
    }

    #[tokio::test]
    async fn test_provider_metadata() {
        let provider = MockProvider::new("metadata-provider");
        let metadata = provider.metadata();

        assert_eq!(metadata.name(), "metadata-provider");
        assert_eq!(metadata.version(), "1.0.0");
        assert!(metadata.capabilities().contains(&"research".to_string()));
        assert!(metadata.capabilities().contains(&"async".to_string()));
        assert!(metadata.capabilities().contains(&"mock".to_string()));
    }

    #[tokio::test]
    async fn test_provider_health_check() {
        let healthy_provider = MockProvider::new("healthy").with_health(true);
        let unhealthy_provider = MockProvider::new("unhealthy").with_health(false);

        let healthy_result = healthy_provider.health_check().await;
        assert!(healthy_result.is_ok());
        assert_eq!(healthy_result.unwrap(), HealthStatus::Healthy);

        let unhealthy_result = unhealthy_provider.health_check().await;
        assert!(unhealthy_result.is_ok());
        assert!(matches!(
            unhealthy_result.unwrap(),
            HealthStatus::Unhealthy(_)
        ));
    }

    #[tokio::test]
    async fn test_provider_query_validation() {
        let provider = MockProvider::new("validator");

        // Valid query should pass
        let valid_result = provider.validate_query("Valid query");
        assert!(valid_result.is_ok());

        // Empty query should fail
        let empty_result = provider.validate_query("");
        assert!(empty_result.is_err());

        // Whitespace-only query should fail
        let whitespace_result = provider.validate_query("   ");
        assert!(whitespace_result.is_err());
    }

    #[tokio::test]
    async fn test_provider_cost_estimation() {
        let provider = MockProvider::new("cost-estimator");
        let query = "What is the meaning of life?";

        let cost_result = provider.estimate_cost(query).await;
        assert!(cost_result.is_ok());

        let cost = cost_result.unwrap();
        assert!(cost.estimated_input_tokens > 0);
        assert!(cost.estimated_output_tokens > 0);
        assert!(cost.estimated_duration > Duration::ZERO);
    }

    #[tokio::test]
    async fn test_provider_usage_stats() {
        let provider = MockProvider::new("stats-provider");

        let stats_result = provider.usage_stats().await;
        assert!(stats_result.is_ok());

        let _stats = stats_result.unwrap();
        // Default implementation returns empty stats, which is expected
    }

    #[tokio::test]
    async fn test_provider_error_retryability() {
        let rate_limit_error = ProviderError::RateLimitExceeded {
            provider: "test".to_string(),
            message: "Rate limited".to_string(),
            retry_after: Some(Duration::from_secs(60)),
            requests_remaining: Some(0),
            tokens_remaining: Some(0),
        };

        let auth_error = ProviderError::AuthenticationFailed {
            provider: "test".to_string(),
            message: "Invalid key".to_string(),
        };

        assert!(rate_limit_error.is_retryable());
        assert!(!auth_error.is_retryable());

        assert_eq!(
            rate_limit_error.retry_after(),
            Some(Duration::from_secs(60))
        );
        assert_eq!(auth_error.retry_after(), None);
    }

    #[tokio::test]
    async fn test_provider_metadata_builder() {
        let metadata = ProviderMetadata::new("test-provider".to_string(), "2.0.0".to_string())
            .with_capabilities(vec!["research".to_string(), "chat".to_string()])
            .with_models(vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()])
            .with_context_length(16384)
            .with_streaming(true)
            .with_attribute("region".to_string(), "us-east-1".to_string());

        assert_eq!(metadata.name(), "test-provider");
        assert_eq!(metadata.version(), "2.0.0");
        assert_eq!(metadata.capabilities().len(), 2);
        assert_eq!(metadata.supported_models().len(), 2);
        assert_eq!(metadata.max_context_length(), 16384);
        assert!(metadata.supports_streaming());
        assert_eq!(
            metadata.custom_attributes().get("region"),
            Some(&"us-east-1".to_string())
        );
    }

    #[tokio::test]
    async fn test_provider_as_trait_object() {
        // Test that providers can be used as trait objects
        let provider: Box<dyn Provider> = Box::new(MockProvider::new("boxed-provider"));

        let metadata = provider.metadata();
        assert_eq!(metadata.name(), "boxed-provider");

        let health_result = provider.health_check().await;
        assert!(health_result.is_ok());

        let query_result = provider.research_query("test".to_string()).await;
        assert!(query_result.is_ok());
    }

    #[tokio::test]
    async fn test_provider_arc_sharing() {
        // Test that providers can be shared across async tasks
        let provider = Arc::new(MockProvider::new("shared-provider"));

        let mut handles = Vec::new();

        for i in 0..5 {
            let provider_clone = Arc::clone(&provider);
            let handle =
                tokio::spawn(
                    async move { provider_clone.research_query(format!("Query {}", i)).await },
                );
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }
}
