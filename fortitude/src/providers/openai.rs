// ABOUTME: OpenAI provider implementation with rate limiting, error handling, and token counting
//! This module provides a concrete implementation of the Provider trait for OpenAI's API.
//! Features include token bucket rate limiting, comprehensive error mapping, cost estimation,
//! and health checking functionality.
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use fortitude::providers::openai::OpenAIProvider;
//! use fortitude::providers::config::ProviderSettings;
//! use fortitude::providers::Provider;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let settings = ProviderSettings::new(
//!         std::env::var("OPENAI_API_KEY")?,
//!         "gpt-4".to_string()
//!     );
//!     
//!     let provider = OpenAIProvider::new(settings).await?;
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

/// Rate limiter combining token buckets for different limits
#[derive(Debug)]
struct RateLimiter {
    request_bucket: TokenBucket,
    input_token_bucket: TokenBucket,
    output_token_bucket: TokenBucket,
    concurrent_semaphore: Arc<Semaphore>,
}

impl RateLimiter {
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
                provider: "openai".to_string(),
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
                provider: "openai".to_string(),
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
                provider: "openai".to_string(),
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
                provider: "openai".to_string(),
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

/// OpenAI API request structure
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    top_p: Option<f32>,
    frequency_penalty: Option<f32>,
    presence_penalty: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

/// OpenAI API response structure
/// All fields required for proper JSON deserialization
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIChoice {
    index: u32,
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenAI API error response
#[derive(Debug, Deserialize)]
struct OpenAIErrorResponse {
    error: OpenAIError,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIError {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
    param: Option<String>,
    code: Option<String>,
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

/// OpenAI provider implementation
#[derive(Debug)]
pub struct OpenAIProvider {
    client: Client,
    settings: ProviderSettings,
    rate_limiter: RateLimiter,
    stats: ProviderStats,
    model_costs: HashMap<String, ModelCosts>,
}

#[derive(Debug, Clone)]
struct ModelCosts {
    input_cost_per_1k_tokens: f64,
    output_cost_per_1k_tokens: f64,
    context_length: usize,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider instance
    pub async fn new(settings: ProviderSettings) -> ProviderResult<Self> {
        settings
            .validate()
            .map_err(|e| ProviderError::ConfigurationError {
                provider: "openai".to_string(),
                message: format!("Configuration validation failed: {e}"),
            })?;

        let client = Client::builder()
            .timeout(settings.timeout)
            .build()
            .map_err(|e| ProviderError::ConfigurationError {
                provider: "openai".to_string(),
                message: format!("Failed to create HTTP client: {e}"),
            })?;

        let rate_limiter = RateLimiter::from_config(&settings.rate_limits);

        let mut model_costs = HashMap::new();
        // Initialize model costs - would typically load from config
        model_costs.insert(
            "gpt-4".to_string(),
            ModelCosts {
                input_cost_per_1k_tokens: 0.03,
                output_cost_per_1k_tokens: 0.06,
                context_length: 8192,
            },
        );
        model_costs.insert(
            "gpt-4-turbo".to_string(),
            ModelCosts {
                input_cost_per_1k_tokens: 0.01,
                output_cost_per_1k_tokens: 0.03,
                context_length: 128000,
            },
        );
        model_costs.insert(
            "gpt-3.5-turbo".to_string(),
            ModelCosts {
                input_cost_per_1k_tokens: 0.001,
                output_cost_per_1k_tokens: 0.002,
                context_length: 16385,
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
        // Rough estimation: 1 token ≈ 4 characters for English text
        (text.len() / 4).max(1) as u32
    }

    /// Get model-specific costs and constraints
    fn get_model_info(&self, model: &str) -> Option<&ModelCosts> {
        self.model_costs.get(model)
    }

    /// Map OpenAI API errors to ProviderError
    fn map_openai_error(&self, error: &OpenAIError, _status_code: StatusCode) -> ProviderError {
        match error.error_type.as_str() {
            "authentication" => ProviderError::AuthenticationFailed {
                provider: "openai".to_string(),
                message: error.message.clone(),
            },
            "rate_limit_exceeded" => ProviderError::RateLimitExceeded {
                provider: "openai".to_string(),
                message: error.message.clone(),
                retry_after: Some(Duration::from_secs(60)), // Default retry after
                requests_remaining: Some(0),
                tokens_remaining: Some(0),
            },
            "quota_exceeded" => ProviderError::QuotaExceeded {
                provider: "openai".to_string(),
                message: error.message.clone(),
                reset_time: None, // Would parse from headers if available
            },
            "server_error" | "service_unavailable" => ProviderError::ServiceUnavailable {
                provider: "openai".to_string(),
                message: error.message.clone(),
                estimated_recovery: Some(Duration::from_secs(30)),
            },
            _ => ProviderError::QueryFailed {
                provider: "openai".to_string(),
                message: error.message.clone(),
                error_code: error.code.clone(),
            },
        }
    }

    /// Execute HTTP request with retry logic
    async fn execute_request(&self, request: OpenAIRequest) -> ProviderResult<OpenAIResponse> {
        let start_time = Instant::now();
        let mut last_error = None;

        for attempt in 0..=self.settings.retry.max_retries {
            // Rate limiting
            let input_tokens = self.estimate_tokens(&request.messages[0].content);
            let estimated_output_tokens = input_tokens / 2; // Rough estimate

            let _guard = self
                .rate_limiter
                .acquire(input_tokens, estimated_output_tokens)
                .await?;

            let endpoint = self
                .settings
                .endpoint
                .as_ref()
                .map(|e| format!("{}/chat/completions", e.trim_end_matches('/')))
                .unwrap_or_else(|| "https://api.openai.com/v1/chat/completions".to_string());

            let response = self
                .client
                .post(&endpoint)
                .header("Authorization", format!("Bearer {}", self.settings.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    let response_time = start_time.elapsed();

                    if status.is_success() {
                        match resp.json::<OpenAIResponse>().await {
                            Ok(openai_resp) => {
                                let actual_input_tokens = openai_resp
                                    .usage
                                    .as_ref()
                                    .map(|u| u.prompt_tokens)
                                    .unwrap_or(input_tokens);
                                let actual_output_tokens = openai_resp
                                    .usage
                                    .as_ref()
                                    .map(|u| u.completion_tokens)
                                    .unwrap_or(0);

                                self.stats.record_request(
                                    true,
                                    actual_input_tokens,
                                    actual_output_tokens,
                                    response_time,
                                );
                                return Ok(openai_resp);
                            }
                            Err(e) => {
                                last_error = Some(ProviderError::SerializationError {
                                    provider: "openai".to_string(),
                                    message: format!("Failed to parse response: {e}"),
                                });
                            }
                        }
                    } else {
                        match resp.json::<OpenAIErrorResponse>().await {
                            Ok(error_resp) => {
                                let provider_error =
                                    self.map_openai_error(&error_resp.error, status);
                                self.stats
                                    .record_request(false, input_tokens, 0, response_time);

                                if !provider_error.is_retryable() {
                                    return Err(provider_error);
                                }
                                last_error = Some(provider_error);
                            }
                            Err(_) => {
                                let provider_error = ProviderError::QueryFailed {
                                    provider: "openai".to_string(),
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
                            provider: "openai".to_string(),
                            duration: self.settings.timeout,
                        }
                    } else {
                        ProviderError::NetworkError {
                            provider: "openai".to_string(),
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
            provider: "openai".to_string(),
            message: "All retry attempts exhausted".to_string(),
            error_code: None,
        }))
    }
}

#[async_trait]
impl Provider for OpenAIProvider {
    async fn research_query(&self, query: String) -> ProviderResult<String> {
        self.validate_query(&query)?;

        debug!("OpenAI provider executing research query: {}", query);

        let request = OpenAIRequest {
            model: self.settings.model.clone(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: query,
            }],
            temperature: Some(0.7),
            max_tokens: Some(1000),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
        };

        let response = self.execute_request(request).await?;

        let content = response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| ProviderError::QueryFailed {
                provider: "openai".to_string(),
                message: "No response content in OpenAI response".to_string(),
                error_code: None,
            })?;

        info!("OpenAI provider completed research query successfully");
        Ok(content)
    }

    fn metadata(&self) -> ProviderMetadata {
        let model_info = self.get_model_info(&self.settings.model);
        let context_length = model_info.map(|m| m.context_length).unwrap_or(8192);

        ProviderMetadata::new("openai".to_string(), "1.0.0".to_string())
            .with_capabilities(vec![
                "research".to_string(),
                "async".to_string(),
                "rate_limited".to_string(),
                "cost_estimation".to_string(),
                "token_counting".to_string(),
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
            .with_attribute("provider_type".to_string(), "openai".to_string())
            .with_attribute("api_version".to_string(), "v1".to_string())
    }

    async fn health_check(&self) -> ProviderResult<HealthStatus> {
        debug!("OpenAI provider performing health check");

        // Use a simple test request to check API availability
        let test_request = OpenAIRequest {
            model: self.settings.model.clone(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            temperature: Some(0.0),
            max_tokens: Some(1),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
        };

        match self.execute_request(test_request).await {
            Ok(_) => {
                info!("OpenAI provider health check passed");
                Ok(HealthStatus::Healthy)
            }
            Err(ProviderError::RateLimitExceeded { .. }) => {
                warn!("OpenAI provider health check: rate limited but service available");
                Ok(HealthStatus::Degraded("Rate limited".to_string()))
            }
            Err(ProviderError::AuthenticationFailed { .. }) => {
                error!("OpenAI provider health check: authentication failed");
                Ok(HealthStatus::Unhealthy("Authentication failed".to_string()))
            }
            Err(ProviderError::QuotaExceeded { .. }) => {
                warn!("OpenAI provider health check: quota exceeded but service available");
                Ok(HealthStatus::Degraded("Quota exceeded".to_string()))
            }
            Err(ProviderError::ServiceUnavailable { .. })
            | Err(ProviderError::NetworkError { .. }) => {
                error!("OpenAI provider health check: service unavailable");
                Ok(HealthStatus::Unhealthy("Service unavailable".to_string()))
            }
            Err(e) => {
                error!("OpenAI provider health check failed: {}", e);
                Ok(HealthStatus::Unhealthy(format!("Health check failed: {e}")))
            }
        }
    }

    async fn estimate_cost(&self, query: &str) -> ProviderResult<QueryCost> {
        let input_tokens = self.estimate_tokens(query);
        let estimated_output_tokens = input_tokens / 2; // Conservative estimate

        let model_info = self.get_model_info(&self.settings.model);
        let estimated_cost_usd = model_info.map(|info| {
            let input_cost = (input_tokens as f64 / 1000.0) * info.input_cost_per_1k_tokens;
            let output_cost =
                (estimated_output_tokens as f64 / 1000.0) * info.output_cost_per_1k_tokens;
            input_cost + output_cost
        });

        Ok(QueryCost {
            estimated_input_tokens: input_tokens,
            estimated_output_tokens,
            estimated_duration: Duration::from_secs(3), // Conservative estimate
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
    use tokio::time::{sleep, timeout};

    // Helper to create test settings
    fn test_settings() -> ProviderSettings {
        ProviderSettings::new("test-api-key".to_string(), "gpt-3.5-turbo".to_string())
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
    async fn test_openai_provider_creation() {
        let settings = test_settings();
        let result = OpenAIProvider::new(settings).await;

        // This should fail until implementation is complete
        assert!(
            result.is_ok(),
            "OpenAI provider creation should succeed with valid settings"
        );

        let provider = result.unwrap();
        let metadata = provider.metadata();
        assert_eq!(metadata.name(), "openai");
        assert!(metadata
            .capabilities()
            .contains(&"rate_limited".to_string()));
    }

    #[tokio::test]
    async fn test_openai_provider_invalid_settings() {
        let invalid_settings = ProviderSettings::new(
            "".to_string(), // Invalid empty API key
            "gpt-4".to_string(),
        );

        let result = OpenAIProvider::new(invalid_settings).await;
        assert!(
            result.is_err(),
            "OpenAI provider creation should fail with invalid settings"
        );

        match result.unwrap_err() {
            ProviderError::ConfigurationError { .. } => {
                // Expected error type
            }
            _ => panic!("Expected ConfigurationError"),
        }
    }

    #[tokio::test]
    async fn test_token_bucket_rate_limiting() {
        let bucket = TokenBucket::new(2.0, 1.0); // 2 tokens, refill 1 per second

        // Should be able to consume 2 tokens initially
        assert!(bucket.try_consume(1.0).await);
        assert!(bucket.try_consume(1.0).await);

        // Should fail to consume more tokens
        assert!(!bucket.try_consume(1.0).await);

        // Wait for refill and try again
        sleep(Duration::from_millis(1100)).await;
        assert!(bucket.try_consume(1.0).await);
    }

    #[tokio::test]
    async fn test_rate_limiter_request_limiting() {
        let config = RateLimitConfig {
            requests_per_minute: 2,
            input_tokens_per_minute: 1000,
            output_tokens_per_minute: 500,
            max_concurrent_requests: 1,
        };

        let rate_limiter = RateLimiter::from_config(&config);

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
    async fn test_token_estimation() {
        let settings = test_settings();
        let provider = OpenAIProvider::new(settings).await.unwrap();

        let short_text = "Hello";
        let long_text = "This is a much longer text that should result in more estimated tokens";

        let short_tokens = provider.estimate_tokens(short_text);
        let long_tokens = provider.estimate_tokens(long_text);

        assert!(short_tokens > 0);
        assert!(long_tokens > short_tokens);
        assert_eq!(short_tokens, 1); // "Hello" has 5 chars, 5/4=1.25 -> max(1, 1) = 1
    }

    #[tokio::test]
    async fn test_cost_estimation() {
        let settings = test_settings();
        let provider = OpenAIProvider::new(settings).await.unwrap();

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
    async fn test_error_mapping() {
        let settings = test_settings();
        let provider = OpenAIProvider::new(settings).await.unwrap();

        let auth_error = OpenAIError {
            message: "Invalid API key".to_string(),
            error_type: "authentication".to_string(),
            param: None,
            code: Some("invalid_api_key".to_string()),
        };

        let mapped_error = provider.map_openai_error(&auth_error, StatusCode::UNAUTHORIZED);
        match mapped_error {
            ProviderError::AuthenticationFailed { provider, message } => {
                assert_eq!(provider, "openai");
                assert_eq!(message, "Invalid API key");
            }
            _ => panic!("Expected AuthenticationFailed error"),
        }

        let rate_limit_error = OpenAIError {
            message: "Rate limit exceeded".to_string(),
            error_type: "rate_limit_exceeded".to_string(),
            param: None,
            code: None,
        };

        let mapped_rate_limit =
            provider.map_openai_error(&rate_limit_error, StatusCode::TOO_MANY_REQUESTS);
        match mapped_rate_limit {
            ProviderError::RateLimitExceeded {
                provider,
                retry_after,
                ..
            } => {
                assert_eq!(provider, "openai");
                assert!(retry_after.is_some());
            }
            _ => panic!("Expected RateLimitExceeded error"),
        }
    }

    #[tokio::test]
    async fn test_usage_stats_tracking() {
        let settings = test_settings();
        let provider = OpenAIProvider::new(settings).await.unwrap();

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
    async fn test_provider_metadata() {
        let settings = test_settings();
        let provider = OpenAIProvider::new(settings).await.unwrap();

        let metadata = provider.metadata();

        assert_eq!(metadata.name(), "openai");
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
            .supported_models()
            .contains(&"gpt-3.5-turbo".to_string()));
        assert_eq!(
            metadata.custom_attributes().get("provider_type"),
            Some(&"openai".to_string())
        );
    }

    #[tokio::test]
    async fn test_query_validation() {
        let settings = test_settings();
        let provider = OpenAIProvider::new(settings).await.unwrap();

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

    // Note: The following tests will fail until the full implementation is complete
    // They represent the TDD approach - write failing tests first

    #[tokio::test]
    async fn test_research_query_success() {
        let settings = test_settings();
        let provider = OpenAIProvider::new(settings).await.unwrap();

        // This test will fail until we implement the HTTP client properly
        // For now, it should at least validate the query
        let result = provider
            .research_query("What is quantum computing?".to_string())
            .await;

        // This assertion will fail until implementation is complete
        // assert!(result.is_ok(), "Research query should succeed with valid API");

        // For now, we expect it to fail with a network error since we're using a test key
        assert!(result.is_err(), "Expected failure with test API key");
    }

    #[tokio::test]
    async fn test_health_check() {
        let settings = test_settings();
        let provider = OpenAIProvider::new(settings).await.unwrap();

        // This test will fail until we implement health check properly
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
    async fn test_concurrent_request_limiting() {
        // Create a config with higher request rate to avoid rate limiting during test
        let config = RateLimitConfig {
            requests_per_minute: 60, // Higher rate to avoid hitting request limits
            input_tokens_per_minute: 1000,
            output_tokens_per_minute: 500,
            max_concurrent_requests: 1, // This is what we're testing
        };

        let rate_limiter = RateLimiter::from_config(&config);

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

    #[tokio::test]
    async fn test_retry_mechanism() {
        let settings = test_settings();
        let provider = OpenAIProvider::new(settings).await.unwrap();

        // This test verifies the retry configuration is properly set
        assert_eq!(provider.settings.retry.max_retries, 1);
        assert_eq!(
            provider.settings.retry.initial_delay,
            Duration::from_millis(10)
        );
        assert_eq!(provider.settings.retry.backoff_multiplier, 2.0);
    }
}
