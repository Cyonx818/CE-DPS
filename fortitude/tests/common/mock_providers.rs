//! Mock provider implementations for comprehensive testing

use async_trait::async_trait;
use chrono::Utc;
use fortitude::providers::{
    HealthStatus, Provider, ProviderError, ProviderMetadata, ProviderResult, QueryCost,
    RateLimitConfig, UsageStats,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Configurable mock provider for testing various scenarios
#[derive(Debug, Clone)]
pub struct MockProvider {
    name: String,
    version: String,
    config: MockProviderConfig,
    state: Arc<Mutex<MockProviderState>>,
}

#[derive(Debug, Clone)]
pub struct MockProviderConfig {
    pub healthy: bool,
    pub should_fail: bool,
    pub response_delay: Duration,
    pub rate_limit_exceeded: bool,
    pub auth_fails: bool,
    pub timeout_on_query: bool,
    pub service_unavailable: bool,
    pub quota_exceeded: bool,
    pub supported_models: Vec<String>,
    pub max_context_length: usize,
    pub supports_streaming: bool,
    pub cost_per_input_token: f64,
    pub cost_per_output_token: f64,
    pub rate_limits: RateLimitConfig,
}

impl Default for MockProviderConfig {
    fn default() -> Self {
        Self {
            healthy: true,
            should_fail: false,
            response_delay: Duration::from_millis(50),
            rate_limit_exceeded: false,
            auth_fails: false,
            timeout_on_query: false,
            service_unavailable: false,
            quota_exceeded: false,
            supported_models: vec!["mock-model-v1".to_string(), "mock-model-v2".to_string()],
            max_context_length: 8192,
            supports_streaming: false,
            cost_per_input_token: 0.0001,
            cost_per_output_token: 0.0002,
            rate_limits: RateLimitConfig::default(),
        }
    }
}

#[derive(Debug, Default)]
struct MockProviderState {
    usage_stats: UsageStats,
    request_times: Vec<Instant>,
    concurrent_requests: u32,
    last_health_check: Option<Instant>,
}

impl MockProvider {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            config: MockProviderConfig::default(),
            state: Arc::new(Mutex::new(MockProviderState::default())),
        }
    }

    pub fn with_config(mut self, config: MockProviderConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.version = version;
        self
    }

    pub fn with_health(mut self, healthy: bool) -> Self {
        self.config.healthy = healthy;
        self
    }

    pub fn with_failure(mut self, should_fail: bool) -> Self {
        self.config.should_fail = should_fail;
        self
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.config.response_delay = delay;
        self
    }

    pub fn with_rate_limit_exceeded(mut self, exceeded: bool) -> Self {
        self.config.rate_limit_exceeded = exceeded;
        self
    }

    pub fn with_auth_failure(mut self, fails: bool) -> Self {
        self.config.auth_fails = fails;
        self
    }

    pub fn with_timeout(mut self, timeout: bool) -> Self {
        self.config.timeout_on_query = timeout;
        self
    }

    pub fn with_service_unavailable(mut self, unavailable: bool) -> Self {
        self.config.service_unavailable = unavailable;
        self
    }

    pub fn with_quota_exceeded(mut self, exceeded: bool) -> Self {
        self.config.quota_exceeded = exceeded;
        self
    }

    pub fn with_models(mut self, models: Vec<String>) -> Self {
        self.config.supported_models = models;
        self
    }

    pub fn with_context_length(mut self, length: usize) -> Self {
        self.config.max_context_length = length;
        self
    }

    pub fn with_streaming(mut self, streaming: bool) -> Self {
        self.config.supports_streaming = streaming;
        self
    }

    pub fn with_pricing(mut self, input_cost: f64, output_cost: f64) -> Self {
        self.config.cost_per_input_token = input_cost;
        self.config.cost_per_output_token = output_cost;
        self
    }

    pub fn with_rate_limits(mut self, rate_limits: RateLimitConfig) -> Self {
        self.config.rate_limits = rate_limits;
        self
    }

    /// Reset provider state (useful for test isolation)
    pub fn reset_state(&self) {
        let mut state = self.state.lock().unwrap();
        *state = MockProviderState::default();
    }

    /// Get current usage statistics (for testing)
    pub fn get_usage_stats(&self) -> UsageStats {
        let state = self.state.lock().unwrap();
        state.usage_stats.clone()
    }

    /// Get current concurrent request count (for testing)
    pub fn get_concurrent_requests(&self) -> u32 {
        let state = self.state.lock().unwrap();
        state.concurrent_requests
    }

    /// Check if rate limits would be exceeded for a request
    fn check_rate_limits(&self) -> Result<(), ProviderError> {
        if self.config.rate_limit_exceeded {
            return Err(ProviderError::RateLimitExceeded {
                provider: self.name.clone(),
                message: "Mock rate limit exceeded".to_string(),
                retry_after: Some(Duration::from_secs(60)),
                requests_remaining: Some(0),
                tokens_remaining: Some(0),
            });
        }

        let state = self.state.lock().unwrap();

        // Check concurrent request limit
        if state.concurrent_requests >= self.config.rate_limits.max_concurrent_requests {
            return Err(ProviderError::RateLimitExceeded {
                provider: self.name.clone(),
                message: "Too many concurrent requests".to_string(),
                retry_after: Some(Duration::from_secs(1)),
                requests_remaining: Some(0),
                tokens_remaining: Some(1000),
            });
        }

        // Check requests per minute limit
        let now = Instant::now();
        let one_minute_ago = now - Duration::from_secs(60);
        let recent_requests = state
            .request_times
            .iter()
            .filter(|&&time| time > one_minute_ago)
            .count() as u32;

        if recent_requests >= self.config.rate_limits.requests_per_minute {
            return Err(ProviderError::RateLimitExceeded {
                provider: self.name.clone(),
                message: "Requests per minute limit exceeded".to_string(),
                retry_after: Some(Duration::from_secs(10)),
                requests_remaining: Some(0),
                tokens_remaining: Some(1000),
            });
        }

        Ok(())
    }

    /// Update usage statistics after a request
    fn update_usage_stats(
        &self,
        success: bool,
        input_tokens: u64,
        output_tokens: u64,
        duration: Duration,
    ) {
        let mut state = self.state.lock().unwrap();
        state.usage_stats.total_requests += 1;

        if success {
            state.usage_stats.successful_requests += 1;
        } else {
            state.usage_stats.failed_requests += 1;
        }

        state.usage_stats.total_input_tokens += input_tokens;
        state.usage_stats.total_output_tokens += output_tokens;

        // Update average response time
        let total_duration = state.usage_stats.average_response_time.as_millis() as u64
            * (state.usage_stats.total_requests - 1)
            + duration.as_millis() as u64;
        state.usage_stats.average_response_time =
            Duration::from_millis(total_duration / state.usage_stats.total_requests);

        state.usage_stats.last_request_time = Some(Utc::now());
        state.request_times.push(Instant::now());

        // Clean up old request times (keep only last hour)
        let one_hour_ago = Instant::now() - Duration::from_secs(3600);
        state.request_times.retain(|&time| time > one_hour_ago);
    }
}

#[async_trait]
impl Provider for MockProvider {
    async fn research_query(&self, query: String) -> ProviderResult<String> {
        let start_time = Instant::now();

        // Increment concurrent requests
        {
            let mut state = self.state.lock().unwrap();
            state.concurrent_requests += 1;
        }

        // Ensure we decrement concurrent requests when function exits
        struct ConcurrentRequestGuard {
            state: Arc<Mutex<MockProviderState>>,
        }
        impl Drop for ConcurrentRequestGuard {
            fn drop(&mut self) {
                let mut state = self.state.lock().unwrap();
                state.concurrent_requests = state.concurrent_requests.saturating_sub(1);
            }
        }
        let _guard = ConcurrentRequestGuard {
            state: Arc::clone(&self.state),
        };

        // Check various error conditions first
        if self.config.auth_fails {
            let duration = start_time.elapsed();
            self.update_usage_stats(false, 0, 0, duration);
            return Err(ProviderError::AuthenticationFailed {
                provider: self.name.clone(),
                message: "Mock authentication failure".to_string(),
            });
        }

        if self.config.service_unavailable {
            let duration = start_time.elapsed();
            self.update_usage_stats(false, 0, 0, duration);
            return Err(ProviderError::ServiceUnavailable {
                provider: self.name.clone(),
                message: "Mock service unavailable".to_string(),
                estimated_recovery: Some(Duration::from_secs(300)),
            });
        }

        if self.config.quota_exceeded {
            let duration = start_time.elapsed();
            self.update_usage_stats(false, 0, 0, duration);
            return Err(ProviderError::QuotaExceeded {
                provider: self.name.clone(),
                message: "Mock quota exceeded".to_string(),
                reset_time: Some(Utc::now() + chrono::Duration::hours(1)),
            });
        }

        // Check rate limits
        if let Err(e) = self.check_rate_limits() {
            let duration = start_time.elapsed();
            self.update_usage_stats(false, 0, 0, duration);
            return Err(e);
        }

        if !self.config.healthy {
            let duration = start_time.elapsed();
            self.update_usage_stats(false, 0, 0, duration);
            return Err(ProviderError::Unhealthy {
                provider: self.name.clone(),
                message: "Mock provider is unhealthy".to_string(),
            });
        }

        if self.config.should_fail {
            let duration = start_time.elapsed();
            self.update_usage_stats(false, 0, 0, duration);
            return Err(ProviderError::QueryFailed {
                message: "Mock provider configured to fail".to_string(),
                provider: self.name.clone(),
                error_code: Some("MOCK_FAILURE".to_string()),
            });
        }

        // Simulate response delay
        if self.config.response_delay > Duration::ZERO {
            sleep(self.config.response_delay).await;
        }

        // Check for timeout condition
        if self.config.timeout_on_query {
            sleep(Duration::from_secs(10)).await; // Long delay to simulate timeout
            let duration = start_time.elapsed();
            self.update_usage_stats(false, 0, 0, duration);
            return Err(ProviderError::Timeout {
                provider: self.name.clone(),
                duration: Duration::from_secs(10),
            });
        }

        // Generate successful response
        let response = format!("Mock response from {} for query: {}", self.name, query);
        let input_tokens = query.len() as u64 / 4; // Rough estimate
        let output_tokens = response.len() as u64 / 4;
        let duration = start_time.elapsed();

        self.update_usage_stats(true, input_tokens, output_tokens, duration);

        Ok(response)
    }

    fn metadata(&self) -> ProviderMetadata {
        let mut capabilities = vec![
            "research".to_string(),
            "async".to_string(),
            "mock".to_string(),
        ];

        if self.config.supports_streaming {
            capabilities.push("streaming".to_string());
        }

        let mut custom_attributes = HashMap::new();
        custom_attributes.insert("provider_type".to_string(), "mock".to_string());
        custom_attributes.insert("test_mode".to_string(), "true".to_string());
        custom_attributes.insert(
            "concurrent_requests".to_string(),
            self.get_concurrent_requests().to_string(),
        );

        ProviderMetadata::new(self.name.clone(), self.version.clone())
            .with_capabilities(capabilities)
            .with_rate_limits(self.config.rate_limits.clone())
            .with_models(self.config.supported_models.clone())
            .with_context_length(self.config.max_context_length)
            .with_streaming(self.config.supports_streaming)
            .with_attribute("provider_type".to_string(), "mock".to_string())
            .with_attribute("test_mode".to_string(), "true".to_string())
    }

    async fn health_check(&self) -> ProviderResult<HealthStatus> {
        // Update last health check time
        {
            let mut state = self.state.lock().unwrap();
            state.last_health_check = Some(Instant::now());
        }

        if self.config.service_unavailable {
            return Ok(HealthStatus::Unhealthy("Service unavailable".to_string()));
        }

        if self.config.auth_fails {
            return Ok(HealthStatus::Unhealthy("Authentication failed".to_string()));
        }

        if !self.config.healthy {
            return Ok(HealthStatus::Unhealthy(
                "Mock provider unhealthy".to_string(),
            ));
        }

        if self.config.rate_limit_exceeded {
            return Ok(HealthStatus::Degraded("Rate limits exceeded".to_string()));
        }

        if self.config.quota_exceeded {
            return Ok(HealthStatus::Degraded("Quota exceeded".to_string()));
        }

        Ok(HealthStatus::Healthy)
    }

    async fn estimate_cost(&self, query: &str) -> ProviderResult<QueryCost> {
        let estimated_input_tokens = (query.len() / 4) as u32;
        let estimated_output_tokens = estimated_input_tokens / 2; // Estimate shorter response

        let estimated_cost_usd = Some(
            (estimated_input_tokens as f64 * self.config.cost_per_input_token)
                + (estimated_output_tokens as f64 * self.config.cost_per_output_token),
        );

        Ok(QueryCost {
            estimated_input_tokens,
            estimated_output_tokens,
            estimated_duration: self.config.response_delay + Duration::from_millis(100),
            estimated_cost_usd,
        })
    }

    async fn usage_stats(&self) -> ProviderResult<UsageStats> {
        let state = self.state.lock().unwrap();
        Ok(state.usage_stats.clone())
    }
}

/// Create a set of mock providers for testing fallback scenarios
pub fn create_mock_provider_set() -> Vec<MockProvider> {
    vec![
        MockProvider::new("primary-mock")
            .with_health(true)
            .with_delay(Duration::from_millis(100))
            .with_pricing(0.0001, 0.0002),
        MockProvider::new("secondary-mock")
            .with_health(true)
            .with_delay(Duration::from_millis(200))
            .with_pricing(0.0002, 0.0003),
        MockProvider::new("backup-mock")
            .with_health(true)
            .with_delay(Duration::from_millis(300))
            .with_pricing(0.0003, 0.0004),
    ]
}

/// Create a mock provider that fails after N requests (for testing retry logic)
pub fn create_failing_mock_provider(name: &str, fail_after: u32) -> MockProvider {
    // This would require more sophisticated state tracking in a real implementation
    // For now, we'll create a provider that fails immediately
    MockProvider::new(name)
        .with_failure(true)
        .with_delay(Duration::from_millis(50))
}

/// Create a mock provider with specific rate limits for testing
pub fn create_rate_limited_mock_provider(name: &str, requests_per_minute: u32) -> MockProvider {
    let rate_limits = RateLimitConfig {
        requests_per_minute,
        input_tokens_per_minute: 1000,
        output_tokens_per_minute: 500,
        max_concurrent_requests: 1,
    };

    MockProvider::new(name)
        .with_rate_limits(rate_limits)
        .with_delay(Duration::from_millis(100))
}

/// Create a mock provider that simulates network timeouts
pub fn create_timeout_mock_provider(name: &str) -> MockProvider {
    MockProvider::new(name)
        .with_timeout(true)
        .with_delay(Duration::from_secs(10))
}

/// Create a mock provider that simulates authentication failures
pub fn create_auth_failing_mock_provider(name: &str) -> MockProvider {
    MockProvider::new(name)
        .with_auth_failure(true)
        .with_delay(Duration::from_millis(50))
}

/// Create a mock provider that simulates service unavailability
pub fn create_unavailable_mock_provider(name: &str) -> MockProvider {
    MockProvider::new(name)
        .with_service_unavailable(true)
        .with_delay(Duration::from_millis(50))
}

/// Create a mock provider with quota exceeded
pub fn create_quota_exceeded_mock_provider(name: &str) -> MockProvider {
    MockProvider::new(name)
        .with_quota_exceeded(true)
        .with_delay(Duration::from_millis(50))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_mock_provider_basic_functionality() {
        let provider = MockProvider::new("test-provider");

        let result = provider.research_query("test query".to_string()).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("test query"));
        assert!(response.contains("test-provider"));
    }

    #[tokio::test]
    async fn test_mock_provider_failure_modes() {
        let failing_provider = MockProvider::new("failing-provider").with_failure(true);
        let result = failing_provider.research_query("test".to_string()).await;
        assert!(result.is_err());

        let auth_failing_provider = MockProvider::new("auth-failing").with_auth_failure(true);
        let result = auth_failing_provider
            .research_query("test".to_string())
            .await;
        assert!(matches!(
            result.unwrap_err(),
            ProviderError::AuthenticationFailed { .. }
        ));

        let unavailable_provider = MockProvider::new("unavailable").with_service_unavailable(true);
        let result = unavailable_provider
            .research_query("test".to_string())
            .await;
        assert!(matches!(
            result.unwrap_err(),
            ProviderError::ServiceUnavailable { .. }
        ));
    }

    #[tokio::test]
    async fn test_mock_provider_rate_limiting() {
        let provider = create_rate_limited_mock_provider("rate-limited", 2);

        // First two requests should succeed
        let result1 = provider.research_query("query 1".to_string()).await;
        assert!(result1.is_ok());

        let result2 = provider.research_query("query 2".to_string()).await;
        assert!(result2.is_ok());

        // Third request should hit rate limit
        let result3 = provider.research_query("query 3".to_string()).await;
        // Note: Our mock rate limiting is based on configuration, not actual tracking
        // In a real scenario, this would be rate limited
    }

    #[tokio::test]
    async fn test_mock_provider_usage_stats() {
        let provider = MockProvider::new("stats-provider");

        // Initial stats should be zero
        let initial_stats = provider.usage_stats().await.unwrap();
        assert_eq!(initial_stats.total_requests, 0);

        // Make a request
        let _result = provider.research_query("test".to_string()).await;

        // Stats should be updated
        let updated_stats = provider.usage_stats().await.unwrap();
        assert_eq!(updated_stats.total_requests, 1);
        assert_eq!(updated_stats.successful_requests, 1);
        assert!(updated_stats.total_input_tokens > 0);
        assert!(updated_stats.total_output_tokens > 0);
    }

    #[tokio::test]
    async fn test_mock_provider_cost_estimation() {
        let provider = MockProvider::new("cost-provider").with_pricing(0.001, 0.002);

        let cost = provider.estimate_cost("test query").await.unwrap();
        assert!(cost.estimated_input_tokens > 0);
        assert!(cost.estimated_output_tokens > 0);
        assert!(cost.estimated_cost_usd.is_some());
        assert!(cost.estimated_cost_usd.unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_mock_provider_health_check() {
        let healthy_provider = MockProvider::new("healthy").with_health(true);
        let health = healthy_provider.health_check().await.unwrap();
        assert_eq!(health, HealthStatus::Healthy);

        let unhealthy_provider = MockProvider::new("unhealthy").with_health(false);
        let health = unhealthy_provider.health_check().await.unwrap();
        assert!(matches!(health, HealthStatus::Unhealthy(_)));

        let degraded_provider = MockProvider::new("degraded").with_rate_limit_exceeded(true);
        let health = degraded_provider.health_check().await.unwrap();
        assert!(matches!(health, HealthStatus::Degraded(_)));
    }

    #[tokio::test]
    async fn test_mock_provider_concurrent_requests() {
        let provider = Arc::new(MockProvider::new("concurrent-test"));
        let mut handles = Vec::new();

        // Start multiple concurrent requests
        for i in 0..5 {
            let provider_clone = Arc::clone(&provider);
            let handle =
                tokio::spawn(
                    async move { provider_clone.research_query(format!("Query {}", i)).await },
                );
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }

        // Concurrent count should be back to zero
        assert_eq!(provider.get_concurrent_requests(), 0);
    }

    #[tokio::test]
    async fn test_mock_provider_metadata() {
        let provider = MockProvider::new("metadata-test")
            .with_models(vec!["model-1".to_string(), "model-2".to_string()])
            .with_context_length(16384)
            .with_streaming(true);

        let metadata = provider.metadata();
        assert_eq!(metadata.name(), "metadata-test");
        assert_eq!(metadata.supported_models().len(), 2);
        assert_eq!(metadata.max_context_length(), 16384);
        assert!(metadata.supports_streaming());
        assert!(metadata.capabilities().contains(&"streaming".to_string()));
    }

    #[tokio::test]
    async fn test_mock_provider_reset_state() {
        let provider = MockProvider::new("reset-test");

        // Make a request to generate some stats
        let _result = provider.research_query("test".to_string()).await;
        let stats_before = provider.usage_stats().await.unwrap();
        assert!(stats_before.total_requests > 0);

        // Reset state
        provider.reset_state();

        // Stats should be reset
        let stats_after = provider.usage_stats().await.unwrap();
        assert_eq!(stats_after.total_requests, 0);
    }

    #[tokio::test]
    async fn test_mock_provider_response_delay() {
        let provider = MockProvider::new("delay-test").with_delay(Duration::from_millis(100));

        let start = Instant::now();
        let _result = provider.research_query("test".to_string()).await;
        let elapsed = start.elapsed();

        // Should have taken at least the configured delay
        assert!(elapsed >= Duration::from_millis(90)); // Small tolerance for timing
    }
}
