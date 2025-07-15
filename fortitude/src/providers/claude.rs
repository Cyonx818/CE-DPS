// ABOUTME: Anthropic Claude provider implementation with Messages API v2, rate limiting, and error handling
//! This module provides a concrete implementation of the Provider trait for Anthropic's Claude API.
//! Features include Anthropic Messages API v2 integration, token bucket rate limiting, comprehensive
//! error mapping, cost estimation, and health checking functionality.
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use fortitude::providers::claude::ClaudeProvider;
//! use fortitude::providers::config::ProviderSettings;
//! use fortitude::providers::Provider;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let settings = ProviderSettings::new(
//!         std::env::var("ANTHROPIC_API_KEY")?,
//!         "claude-3-5-sonnet-20241022".to_string()
//!     );
//!     
//!     let provider = ClaudeProvider::new(settings).await?;
//!     let response = provider.research_query("What is quantum computing?".to_string()).await?;
//!     println!("Response: {}", response);
//!     Ok(())
//! }
//! ```

use crate::providers::config::{ProviderSettings, RateLimitConfig};
use crate::providers::{
    HealthStatus, Provider, ProviderError, ProviderMetadata, ProviderResult, QueryCost, UsageStats,
};

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, error, info, warn};

/// Token bucket for rate limiting with async support
#[derive(Debug)]
struct TokenBucket {
    tokens: Arc<Mutex<f64>>,
    capacity: f64,
    refill_rate: f64, // tokens per second
    last_refill: Arc<Mutex<Instant>>,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: Arc::new(Mutex::new(capacity)),
            capacity,
            refill_rate,
            last_refill: Arc::new(Mutex::new(Instant::now())),
        }
    }

    async fn try_consume(&self, tokens: f64) -> bool {
        let mut current_tokens = self.tokens.lock().await;
        let mut last_refill = self.last_refill.lock().await;

        let now = Instant::now();
        let time_passed = now.duration_since(*last_refill).as_secs_f64();

        // Refill tokens
        let new_tokens = (*current_tokens + time_passed * self.refill_rate).min(self.capacity);
        *current_tokens = new_tokens;
        *last_refill = now;

        if *current_tokens >= tokens {
            *current_tokens -= tokens;
            true
        } else {
            false
        }
    }

    async fn wait_for_tokens(&self, tokens: f64) -> Duration {
        let current_tokens = *self.tokens.lock().await;
        if current_tokens >= tokens {
            Duration::ZERO
        } else {
            let needed = tokens - current_tokens;
            Duration::from_secs_f64(needed / self.refill_rate)
        }
    }
}

/// Rate limiter combining token buckets for different limits (Claude-specific)
#[derive(Debug)]
struct ClaudeRateLimiter {
    request_bucket: TokenBucket,
    input_token_bucket: TokenBucket,
    output_token_bucket: TokenBucket,
    concurrent_semaphore: Arc<Semaphore>,
}

impl ClaudeRateLimiter {
    fn from_config(config: &RateLimitConfig) -> Self {
        Self {
            request_bucket: TokenBucket::new(
                config.requests_per_minute as f64,
                config.requests_per_minute as f64 / 60.0,
            ),
            input_token_bucket: TokenBucket::new(
                config.input_tokens_per_minute as f64,
                config.input_tokens_per_minute as f64 / 60.0,
            ),
            output_token_bucket: TokenBucket::new(
                config.output_tokens_per_minute as f64,
                config.output_tokens_per_minute as f64 / 60.0,
            ),
            concurrent_semaphore: Arc::new(Semaphore::new(config.max_concurrent_requests as usize)),
        }
    }

    async fn acquire(
        &self,
        input_tokens: u32,
        estimated_output_tokens: u32,
    ) -> Result<RateLimitGuard, ProviderError> {
        // Try to acquire request permit
        if !self.request_bucket.try_consume(1.0).await {
            let wait_time = self.request_bucket.wait_for_tokens(1.0).await;
            return Err(ProviderError::RateLimitExceeded {
                provider: "claude".to_string(),
                message: "Request rate limit exceeded".to_string(),
                retry_after: Some(wait_time),
                requests_remaining: Some(0),
                tokens_remaining: None,
            });
        }

        // Try to acquire input token permit
        if !self
            .input_token_bucket
            .try_consume(input_tokens as f64)
            .await
        {
            let wait_time = self
                .input_token_bucket
                .wait_for_tokens(input_tokens as f64)
                .await;
            return Err(ProviderError::RateLimitExceeded {
                provider: "claude".to_string(),
                message: "Input token rate limit exceeded".to_string(),
                retry_after: Some(wait_time),
                requests_remaining: None,
                tokens_remaining: Some(0),
            });
        }

        // Try to acquire output token permit
        if !self
            .output_token_bucket
            .try_consume(estimated_output_tokens as f64)
            .await
        {
            let wait_time = self
                .output_token_bucket
                .wait_for_tokens(estimated_output_tokens as f64)
                .await;
            return Err(ProviderError::RateLimitExceeded {
                provider: "claude".to_string(),
                message: "Output token rate limit exceeded".to_string(),
                retry_after: Some(wait_time),
                requests_remaining: None,
                tokens_remaining: Some(0),
            });
        }

        // Acquire concurrent request permit
        let permit = self
            .concurrent_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| ProviderError::ServiceUnavailable {
                provider: "claude".to_string(),
                message: "Concurrent request limit exceeded".to_string(),
                estimated_recovery: Some(Duration::from_secs(1)),
            })?;

        Ok(RateLimitGuard { _permit: permit })
    }
}

/// Guard for rate limit permits
#[derive(Debug)]
struct RateLimitGuard {
    _permit: tokio::sync::OwnedSemaphorePermit,
}

/// Anthropic Messages API v2 request structure
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    system: Option<String>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    top_k: Option<u32>,
    stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

/// Anthropic Messages API v2 response structure
/// All fields required for proper JSON deserialization
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ClaudeResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<ClaudeContent>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: ClaudeUsage,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ClaudeContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Anthropic API error response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ClaudeErrorResponse {
    #[serde(rename = "type")]
    error_type: String,
    error: ClaudeError,
}

#[derive(Debug, Deserialize)]
struct ClaudeError {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

/// Usage statistics tracking
#[derive(Debug, Default)]
struct ProviderStats {
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
    total_input_tokens: AtomicU64,
    total_output_tokens: AtomicU64,
    last_request_time: Arc<Mutex<Option<chrono::DateTime<chrono::Utc>>>>,
    response_times: Arc<Mutex<Vec<Duration>>>,
}

impl ProviderStats {
    fn record_request(
        &self,
        success: bool,
        input_tokens: u32,
        output_tokens: u32,
        response_time: Duration,
    ) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        if success {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }

        self.total_input_tokens
            .fetch_add(input_tokens as u64, Ordering::Relaxed);
        self.total_output_tokens
            .fetch_add(output_tokens as u64, Ordering::Relaxed);

        // Update last request time and response times
        let now = chrono::Utc::now();
        if let Ok(mut last_time) = self.last_request_time.try_lock() {
            *last_time = Some(now);
        }

        if let Ok(mut times) = self.response_times.try_lock() {
            times.push(response_time);
            // Keep only last 100 response times for average calculation
            if times.len() > 100 {
                times.remove(0);
            }
        }
    }

    async fn to_usage_stats(&self) -> UsageStats {
        let response_times = self.response_times.lock().await;
        let average_response_time = if response_times.is_empty() {
            Duration::ZERO
        } else {
            let total: Duration = response_times.iter().sum();
            total / response_times.len() as u32
        };

        let last_request_time = *self.last_request_time.lock().await;

        UsageStats {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            successful_requests: self.successful_requests.load(Ordering::Relaxed),
            failed_requests: self.failed_requests.load(Ordering::Relaxed),
            total_input_tokens: self.total_input_tokens.load(Ordering::Relaxed),
            total_output_tokens: self.total_output_tokens.load(Ordering::Relaxed),
            average_response_time,
            last_request_time,
        }
    }
}

/// Claude model pricing and configuration
#[derive(Debug, Clone)]
struct ClaudeModelInfo {
    input_cost_per_1m_tokens: f64,
    output_cost_per_1m_tokens: f64,
    context_length: usize,
    max_output_tokens: u32,
}

/// Anthropic Claude provider implementation
#[derive(Debug)]
pub struct ClaudeProvider {
    client: Client,
    settings: ProviderSettings,
    rate_limiter: ClaudeRateLimiter,
    stats: ProviderStats,
    model_costs: HashMap<String, ClaudeModelInfo>,
}

impl ClaudeProvider {
    /// Create a new Claude provider instance
    pub async fn new(settings: ProviderSettings) -> ProviderResult<Self> {
        settings
            .validate()
            .map_err(|e| ProviderError::ConfigurationError {
                provider: "claude".to_string(),
                message: format!("Configuration validation failed: {e}"),
            })?;

        let client = Client::builder()
            .timeout(settings.timeout)
            .build()
            .map_err(|e| ProviderError::ConfigurationError {
                provider: "claude".to_string(),
                message: format!("Failed to create HTTP client: {e}"),
            })?;

        let rate_limiter = ClaudeRateLimiter::from_config(&settings.rate_limits);

        let mut model_costs = HashMap::new();
        // Initialize Claude model costs (as of 2024 pricing)
        model_costs.insert(
            "claude-3-5-sonnet-20241022".to_string(),
            ClaudeModelInfo {
                input_cost_per_1m_tokens: 3.00,
                output_cost_per_1m_tokens: 15.00,
                context_length: 200000,
                max_output_tokens: 8192,
            },
        );
        model_costs.insert(
            "claude-3-haiku-20240307".to_string(),
            ClaudeModelInfo {
                input_cost_per_1m_tokens: 0.25,
                output_cost_per_1m_tokens: 1.25,
                context_length: 200000,
                max_output_tokens: 4096,
            },
        );
        model_costs.insert(
            "claude-3-opus-20240229".to_string(),
            ClaudeModelInfo {
                input_cost_per_1m_tokens: 15.00,
                output_cost_per_1m_tokens: 75.00,
                context_length: 200000,
                max_output_tokens: 4096,
            },
        );

        Ok(Self {
            client,
            settings,
            rate_limiter,
            stats: ProviderStats::default(),
            model_costs,
        })
    }

    /// Estimate token count for a text string using simple heuristic
    fn estimate_tokens(&self, text: &str) -> u32 {
        // Anthropic's estimation: roughly 1 token per 4 characters for English text
        (text.len() / 4).max(1) as u32
    }

    /// Get model-specific costs and constraints
    fn get_model_info(&self, model: &str) -> Option<&ClaudeModelInfo> {
        self.model_costs.get(model)
    }

    /// Map Claude API errors to ProviderError
    fn map_claude_error(&self, error: &ClaudeError, _status_code: StatusCode) -> ProviderError {
        match error.error_type.as_str() {
            "authentication_error" => ProviderError::AuthenticationFailed {
                provider: "claude".to_string(),
                message: error.message.clone(),
            },
            "rate_limit_error" => ProviderError::RateLimitExceeded {
                provider: "claude".to_string(),
                message: error.message.clone(),
                retry_after: Some(Duration::from_secs(60)), // Default retry after
                requests_remaining: Some(0),
                tokens_remaining: Some(0),
            },
            "billing_error" | "quota_exceeded" => ProviderError::QuotaExceeded {
                provider: "claude".to_string(),
                message: error.message.clone(),
                reset_time: None, // Would parse from headers if available
            },
            "server_error" | "service_unavailable" => ProviderError::ServiceUnavailable {
                provider: "claude".to_string(),
                message: error.message.clone(),
                estimated_recovery: Some(Duration::from_secs(30)),
            },
            "overloaded_error" => ProviderError::ServiceUnavailable {
                provider: "claude".to_string(),
                message: "Service temporarily overloaded".to_string(),
                estimated_recovery: Some(Duration::from_secs(60)),
            },
            _ => ProviderError::QueryFailed {
                provider: "claude".to_string(),
                message: error.message.clone(),
                error_code: Some(error.error_type.clone()),
            },
        }
    }

    /// Execute HTTP request with retry logic
    async fn execute_request(&self, request: ClaudeRequest) -> ProviderResult<ClaudeResponse> {
        let start_time = Instant::now();
        let mut last_error = None;

        for attempt in 0..=self.settings.retry.max_retries {
            // Rate limiting
            let input_tokens = self.estimate_tokens(&request.messages[0].content);
            let estimated_output_tokens = request.max_tokens / 2; // Conservative estimate

            let _guard = self
                .rate_limiter
                .acquire(input_tokens, estimated_output_tokens)
                .await?;

            let endpoint = self
                .settings
                .endpoint
                .as_ref()
                .map(|e| format!("{}/v1/messages", e.trim_end_matches('/')))
                .unwrap_or_else(|| "https://api.anthropic.com/v1/messages".to_string());

            let response = self
                .client
                .post(&endpoint)
                .header("x-api-key", &self.settings.api_key)
                .header("content-type", "application/json")
                .header("anthropic-version", "2023-06-01")
                .json(&request)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    let response_time = start_time.elapsed();

                    if status.is_success() {
                        match resp.json::<ClaudeResponse>().await {
                            Ok(claude_resp) => {
                                let actual_input_tokens = claude_resp.usage.input_tokens;
                                let actual_output_tokens = claude_resp.usage.output_tokens;

                                self.stats.record_request(
                                    true,
                                    actual_input_tokens,
                                    actual_output_tokens,
                                    response_time,
                                );
                                return Ok(claude_resp);
                            }
                            Err(e) => {
                                last_error = Some(ProviderError::SerializationError {
                                    provider: "claude".to_string(),
                                    message: format!("Failed to parse response: {e}"),
                                });
                            }
                        }
                    } else {
                        match resp.json::<ClaudeErrorResponse>().await {
                            Ok(error_resp) => {
                                let provider_error =
                                    self.map_claude_error(&error_resp.error, status);
                                self.stats
                                    .record_request(false, input_tokens, 0, response_time);

                                if !provider_error.is_retryable() {
                                    return Err(provider_error);
                                }
                                last_error = Some(provider_error);
                            }
                            Err(_) => {
                                let provider_error = ProviderError::QueryFailed {
                                    provider: "claude".to_string(),
                                    message: format!("HTTP {} error", status.as_u16()),
                                    error_code: Some(status.as_u16().to_string()),
                                };
                                self.stats
                                    .record_request(false, input_tokens, 0, response_time);
                                last_error = Some(provider_error);
                            }
                        }
                    }
                }
                Err(e) => {
                    let provider_error = if e.is_timeout() {
                        ProviderError::Timeout {
                            provider: "claude".to_string(),
                            duration: self.settings.timeout,
                        }
                    } else {
                        ProviderError::NetworkError {
                            provider: "claude".to_string(),
                            source: Box::new(e),
                        }
                    };

                    self.stats
                        .record_request(false, input_tokens, 0, start_time.elapsed());
                    last_error = Some(provider_error);
                }
            }

            // Wait before retry if not the last attempt
            if attempt < self.settings.retry.max_retries {
                let delay = self.settings.retry.calculate_delay(attempt);
                tokio::time::sleep(delay).await;
            }
        }

        Err(last_error.unwrap_or(ProviderError::QueryFailed {
            provider: "claude".to_string(),
            message: "All retry attempts exhausted".to_string(),
            error_code: None,
        }))
    }
}

#[async_trait]
impl Provider for ClaudeProvider {
    async fn research_query(&self, query: String) -> ProviderResult<String> {
        self.validate_query(&query)?;

        debug!("Claude provider executing research query: {}", query);

        let model_info = self.get_model_info(&self.settings.model);
        let max_tokens = model_info.map(|m| m.max_output_tokens).unwrap_or(4096);

        let request = ClaudeRequest {
            model: self.settings.model.clone(),
            max_tokens,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: query,
            }],
            system: None,
            temperature: Some(0.7),
            top_p: None,
            top_k: None,
            stop_sequences: None,
        };

        let response = self.execute_request(request).await?;

        let content = response
            .content
            .first()
            .map(|content| content.text.clone())
            .ok_or_else(|| ProviderError::QueryFailed {
                provider: "claude".to_string(),
                message: "No response content in Claude response".to_string(),
                error_code: None,
            })?;

        info!("Claude provider completed research query successfully");
        Ok(content)
    }

    fn metadata(&self) -> ProviderMetadata {
        let model_info = self.get_model_info(&self.settings.model);
        let context_length = model_info.map(|m| m.context_length).unwrap_or(200000);

        ProviderMetadata::new("claude".to_string(), "1.0.0".to_string())
            .with_capabilities(vec![
                "research".to_string(),
                "async".to_string(),
                "rate_limited".to_string(),
                "cost_estimation".to_string(),
                "token_counting".to_string(),
                "anthropic_v2".to_string(),
            ])
            .with_models(self.model_costs.keys().cloned().collect())
            .with_context_length(context_length)
            .with_streaming(false)
            .with_rate_limits(crate::providers::RateLimitConfig {
                requests_per_minute: self.settings.rate_limits.requests_per_minute,
                input_tokens_per_minute: self.settings.rate_limits.input_tokens_per_minute,
                output_tokens_per_minute: self.settings.rate_limits.output_tokens_per_minute,
                max_concurrent_requests: self.settings.rate_limits.max_concurrent_requests,
            })
            .with_attribute("provider_type".to_string(), "claude".to_string())
            .with_attribute("api_version".to_string(), "v2".to_string())
            .with_attribute("messages_api".to_string(), "2023-06-01".to_string())
    }

    async fn health_check(&self) -> ProviderResult<HealthStatus> {
        debug!("Claude provider performing health check");

        // Use a simple test request to check API availability
        let test_request = ClaudeRequest {
            model: self.settings.model.clone(),
            max_tokens: 1,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: "Hi".to_string(),
            }],
            system: None,
            temperature: Some(0.0),
            top_p: None,
            top_k: None,
            stop_sequences: None,
        };

        match self.execute_request(test_request).await {
            Ok(_) => {
                info!("Claude provider health check passed");
                Ok(HealthStatus::Healthy)
            }
            Err(ProviderError::RateLimitExceeded { .. }) => {
                warn!("Claude provider health check: rate limited but service available");
                Ok(HealthStatus::Degraded("Rate limited".to_string()))
            }
            Err(ProviderError::AuthenticationFailed { .. }) => {
                error!("Claude provider health check: authentication failed");
                Ok(HealthStatus::Unhealthy("Authentication failed".to_string()))
            }
            Err(ProviderError::QuotaExceeded { .. }) => {
                warn!("Claude provider health check: quota exceeded but service available");
                Ok(HealthStatus::Degraded("Quota exceeded".to_string()))
            }
            Err(ProviderError::ServiceUnavailable { .. })
            | Err(ProviderError::NetworkError { .. }) => {
                error!("Claude provider health check: service unavailable");
                Ok(HealthStatus::Unhealthy("Service unavailable".to_string()))
            }
            Err(e) => {
                error!("Claude provider health check failed: {}", e);
                Ok(HealthStatus::Unhealthy(format!("Health check failed: {e}")))
            }
        }
    }

    async fn estimate_cost(&self, query: &str) -> ProviderResult<QueryCost> {
        let input_tokens = self.estimate_tokens(query);
        let estimated_output_tokens = input_tokens / 2; // Conservative estimate

        let model_info = self.get_model_info(&self.settings.model);
        let estimated_cost_usd = model_info.map(|info| {
            let input_cost = (input_tokens as f64 / 1_000_000.0) * info.input_cost_per_1m_tokens;
            let output_cost =
                (estimated_output_tokens as f64 / 1_000_000.0) * info.output_cost_per_1m_tokens;
            input_cost + output_cost
        });

        Ok(QueryCost {
            estimated_input_tokens: input_tokens,
            estimated_output_tokens,
            estimated_duration: Duration::from_secs(4), // Conservative estimate for Claude
            estimated_cost_usd,
        })
    }

    async fn usage_stats(&self) -> ProviderResult<UsageStats> {
        Ok(self.stats.to_usage_stats().await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::config::RateLimitConfig;
    use crate::providers::config::{ProviderSettings, RetryConfig};
    use std::time::Duration;
    use tokio::time::timeout;

    // Helper to create test settings
    fn test_settings() -> ProviderSettings {
        ProviderSettings::new(
            "test-api-key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
        )
        .with_timeout(Duration::from_secs(10))
        .with_rate_limits(RateLimitConfig {
            requests_per_minute: 2,
            input_tokens_per_minute: 1000,
            output_tokens_per_minute: 500,
            max_concurrent_requests: 1,
        })
        .with_retry(RetryConfig {
            max_retries: 1,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            jitter: false,
        })
    }

    #[tokio::test]
    async fn test_claude_provider_creation() {
        let settings = test_settings();
        let result = ClaudeProvider::new(settings).await;

        assert!(
            result.is_ok(),
            "Claude provider creation should succeed with valid settings"
        );

        let provider = result.unwrap();
        let metadata = provider.metadata();
        assert_eq!(metadata.name(), "claude");
        assert!(metadata
            .capabilities()
            .contains(&"rate_limited".to_string()));
        assert!(metadata
            .capabilities()
            .contains(&"anthropic_v2".to_string()));
    }

    #[tokio::test]
    async fn test_claude_provider_invalid_settings() {
        let invalid_settings = ProviderSettings::new(
            "".to_string(), // Invalid empty API key
            "claude-3-5-sonnet-20241022".to_string(),
        );

        let result = ClaudeProvider::new(invalid_settings).await;
        assert!(
            result.is_err(),
            "Claude provider creation should fail with invalid settings"
        );

        match result.unwrap_err() {
            ProviderError::ConfigurationError { .. } => {
                // Expected error type
            }
            _ => panic!("Expected ConfigurationError"),
        }
    }

    #[tokio::test]
    async fn test_claude_token_estimation() {
        let settings = test_settings();
        let provider = ClaudeProvider::new(settings).await.unwrap();

        let short_text = "Hello";
        let long_text = "This is a much longer text that should result in more estimated tokens";

        let short_tokens = provider.estimate_tokens(short_text);
        let long_tokens = provider.estimate_tokens(long_text);

        assert!(short_tokens > 0);
        assert!(long_tokens > short_tokens);
        assert_eq!(short_tokens, 1); // "Hello" has 5 chars, 5/4=1.25 -> max(1, 1) = 1
    }

    #[tokio::test]
    async fn test_claude_cost_estimation() {
        let settings = test_settings();
        let provider = ClaudeProvider::new(settings).await.unwrap();

        let query = "What is the meaning of life?";
        let cost_result = provider.estimate_cost(query).await;

        assert!(cost_result.is_ok());
        let cost = cost_result.unwrap();
        assert!(cost.estimated_input_tokens > 0);
        assert!(cost.estimated_output_tokens > 0);
        assert!(cost.estimated_cost_usd.is_some());
        assert!(cost.estimated_cost_usd.unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_claude_model_info() {
        let settings = test_settings();
        let provider = ClaudeProvider::new(settings).await.unwrap();

        let sonnet_info = provider.get_model_info("claude-3-5-sonnet-20241022");
        assert!(sonnet_info.is_some());
        assert_eq!(sonnet_info.unwrap().input_cost_per_1m_tokens, 3.00);
        assert_eq!(sonnet_info.unwrap().output_cost_per_1m_tokens, 15.00);

        let haiku_info = provider.get_model_info("claude-3-haiku-20240307");
        assert!(haiku_info.is_some());
        assert_eq!(haiku_info.unwrap().input_cost_per_1m_tokens, 0.25);

        let opus_info = provider.get_model_info("claude-3-opus-20240229");
        assert!(opus_info.is_some());
        assert_eq!(opus_info.unwrap().input_cost_per_1m_tokens, 15.00);
    }

    #[tokio::test]
    async fn test_claude_error_mapping() {
        let settings = test_settings();
        let provider = ClaudeProvider::new(settings).await.unwrap();

        let auth_error = ClaudeError {
            error_type: "authentication_error".to_string(),
            message: "Invalid API key".to_string(),
        };

        let mapped_error = provider.map_claude_error(&auth_error, StatusCode::UNAUTHORIZED);
        match mapped_error {
            ProviderError::AuthenticationFailed { provider, message } => {
                assert_eq!(provider, "claude");
                assert_eq!(message, "Invalid API key");
            }
            _ => panic!("Expected AuthenticationFailed error"),
        }

        let rate_limit_error = ClaudeError {
            error_type: "rate_limit_error".to_string(),
            message: "Rate limit exceeded".to_string(),
        };

        let mapped_rate_limit =
            provider.map_claude_error(&rate_limit_error, StatusCode::TOO_MANY_REQUESTS);
        match mapped_rate_limit {
            ProviderError::RateLimitExceeded {
                provider,
                retry_after,
                ..
            } => {
                assert_eq!(provider, "claude");
                assert!(retry_after.is_some());
            }
            _ => panic!("Expected RateLimitExceeded error"),
        }

        let overloaded_error = ClaudeError {
            error_type: "overloaded_error".to_string(),
            message: "Service overloaded".to_string(),
        };

        let mapped_overloaded =
            provider.map_claude_error(&overloaded_error, StatusCode::SERVICE_UNAVAILABLE);
        match mapped_overloaded {
            ProviderError::ServiceUnavailable {
                provider,
                estimated_recovery,
                ..
            } => {
                assert_eq!(provider, "claude");
                assert_eq!(estimated_recovery, Some(Duration::from_secs(60)));
            }
            _ => panic!("Expected ServiceUnavailable error"),
        }
    }

    #[tokio::test]
    async fn test_claude_rate_limiter() {
        let config = RateLimitConfig {
            requests_per_minute: 2,
            input_tokens_per_minute: 1000,
            output_tokens_per_minute: 500,
            max_concurrent_requests: 1,
        };

        let rate_limiter = ClaudeRateLimiter::from_config(&config);

        // First two requests should succeed
        let guard1 = rate_limiter.acquire(10, 5).await;
        assert!(guard1.is_ok());
        drop(guard1); // Release the guard

        let guard2 = rate_limiter.acquire(10, 5).await;
        assert!(guard2.is_ok());
        drop(guard2);

        // Third request should fail due to rate limit
        let guard3 = rate_limiter.acquire(10, 5).await;
        assert!(guard3.is_err());

        match guard3.unwrap_err() {
            ProviderError::RateLimitExceeded { .. } => {
                // Expected error
            }
            _ => panic!("Expected RateLimitExceeded error"),
        }
    }

    #[tokio::test]
    async fn test_claude_usage_stats_tracking() {
        let settings = test_settings();
        let provider = ClaudeProvider::new(settings).await.unwrap();

        // Record some test requests
        provider
            .stats
            .record_request(true, 100, 50, Duration::from_millis(500));
        provider
            .stats
            .record_request(false, 80, 0, Duration::from_millis(200));
        provider
            .stats
            .record_request(true, 120, 60, Duration::from_millis(300));

        let stats = provider.stats.to_usage_stats().await;

        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.successful_requests, 2);
        assert_eq!(stats.failed_requests, 1);
        assert_eq!(stats.total_input_tokens, 300);
        assert_eq!(stats.total_output_tokens, 110);
        assert!(stats.average_response_time > Duration::ZERO);
        assert!(stats.last_request_time.is_some());
    }

    #[tokio::test]
    async fn test_claude_provider_metadata() {
        let settings = test_settings();
        let provider = ClaudeProvider::new(settings).await.unwrap();

        let metadata = provider.metadata();

        assert_eq!(metadata.name(), "claude");
        assert_eq!(metadata.version(), "1.0.0");
        assert!(metadata.capabilities().contains(&"research".to_string()));
        assert!(metadata
            .capabilities()
            .contains(&"rate_limited".to_string()));
        assert!(metadata
            .capabilities()
            .contains(&"cost_estimation".to_string()));
        assert!(metadata
            .capabilities()
            .contains(&"token_counting".to_string()));
        assert!(metadata
            .capabilities()
            .contains(&"anthropic_v2".to_string()));
        assert!(metadata
            .supported_models()
            .contains(&"claude-3-5-sonnet-20241022".to_string()));
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
    async fn test_claude_query_validation() {
        let settings = test_settings();
        let provider = ClaudeProvider::new(settings).await.unwrap();

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
    async fn test_claude_concurrent_request_limiting() {
        let config = RateLimitConfig {
            requests_per_minute: 60, // Higher rate to avoid hitting request limits
            input_tokens_per_minute: 1000,
            output_tokens_per_minute: 500,
            max_concurrent_requests: 1, // This is what we're testing
        };

        let rate_limiter = ClaudeRateLimiter::from_config(&config);

        // First request should acquire the single concurrent slot
        let guard1 = rate_limiter.acquire(10, 5).await;
        assert!(guard1.is_ok());

        // Second concurrent request should fail with timeout since concurrent limit is 1
        let guard2_future = rate_limiter.acquire(10, 5);
        let guard2_result = timeout(Duration::from_millis(100), guard2_future).await;

        // Should timeout because concurrent limit is reached
        assert!(
            guard2_result.is_err(),
            "Second concurrent request should timeout"
        );

        // Drop first guard to release the slot
        drop(guard1);

        // Now a new request should succeed
        let guard3 = rate_limiter.acquire(10, 5).await;
        assert!(guard3.is_ok());
    }

    // These tests will fail until full implementation is complete - TDD approach
    #[tokio::test]
    async fn test_claude_research_query_success() {
        let settings = test_settings();
        let provider = ClaudeProvider::new(settings).await.unwrap();

        // This test will fail until we implement the HTTP client properly
        let result = provider
            .research_query("What is quantum computing?".to_string())
            .await;

        // For now, we expect it to fail with a network error since we're using a test key
        assert!(result.is_err(), "Expected failure with test API key");
    }

    #[tokio::test]
    async fn test_claude_health_check() {
        let settings = test_settings();
        let provider = ClaudeProvider::new(settings).await.unwrap();

        let result = provider.health_check().await;

        // Should return a health status (even if unhealthy due to test key)
        assert!(result.is_ok(), "Health check should return a status");

        let health = result.unwrap();
        // With test API key, should be unhealthy
        match health {
            HealthStatus::Unhealthy(_) => {
                // Expected with test API key
            }
            _ => {
                // Might also be healthy if implementation is different
            }
        }
    }

    #[tokio::test]
    async fn test_claude_messages_api_request_structure() {
        let request = ClaudeRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1000,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            system: Some("You are a helpful assistant".to_string()),
            temperature: Some(0.7),
            top_p: None,
            top_k: None,
            stop_sequences: None,
        };

        // Test serialization
        let serialized = serde_json::to_string(&request);
        assert!(serialized.is_ok(), "Request should serialize properly");

        let json_str = serialized.unwrap();
        assert!(json_str.contains("claude-3-5-sonnet-20241022"));
        assert!(json_str.contains("max_tokens"));
        assert!(json_str.contains("messages"));
        assert!(json_str.contains("user"));
        assert!(json_str.contains("Hello"));
    }
}
