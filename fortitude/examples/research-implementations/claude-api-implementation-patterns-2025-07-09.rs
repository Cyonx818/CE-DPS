// Cargo.toml dependencies required:
// [dependencies]
// reqwest = { version = "0.12", features = ["json", "stream"] }
// tokio = { version = "1", features = ["full"] }
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"
// thiserror = "1.0"
// url = "2.4"
// uuid = { version = "1.0", features = ["v4"] }
// tracing = "0.1"
// backoff = { version = "0.4", features = ["futures"] }
// dashmap = "5.5"

use reqwest::{Client, ClientBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::Semaphore;
use tokio::time::{sleep, timeout};
use tracing::{error, info, warn, debug};
use backoff::{future::retry, ExponentialBackoff, Error as BackoffError};
use dashmap::DashMap;
use uuid::Uuid;

/// Claude API client errors using thiserror for comprehensive error handling
#[derive(Error, Debug)]
pub enum ClaudeError {
    #[error("HTTP request failed: {source}")]
    HttpError {
        #[from]
        source: reqwest::Error,
    },

    #[error("API error {status}: {message}")]
    ApiError {
        status: StatusCode,
        message: String,
        error_type: Option<String>,
    },

    #[error("Rate limit exceeded: {message}")]
    RateLimitError {
        message: String,
        retry_after: Option<Duration>,
        requests_remaining: Option<u32>,
        tokens_remaining: Option<u32>,
    },

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Request timeout after {duration:?}")]
    TimeoutError { duration: Duration },

    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },

    #[error("Invalid configuration: {0}")]
    ConfigurationError(String),

    #[error("Maximum retries exceeded")]
    MaxRetriesExceeded,

    #[error("Request validation failed: {0}")]
    ValidationError(String),
}

/// Type alias for Claude API results
pub type ClaudeResult<T> = Result<T, ClaudeError>;

/// Claude API request message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Claude API request structure based on the latest API specifications
#[derive(Debug, Clone, Serialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// Claude API response structure
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeResponse {
    pub id: String,
    pub r#type: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContentBlock {
    pub r#type: String,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(default)]
    pub cache_creation_input_tokens: u32,
    #[serde(default)]
    pub cache_read_input_tokens: u32,
}

/// API error response structure
#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    pub r#type: String,
    pub error: ApiErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorDetail {
    pub r#type: String,
    pub message: String,
}

/// Rate limiting configuration and tracking
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub input_tokens_per_minute: u32,
    pub output_tokens_per_minute: u32,
    pub max_concurrent_requests: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        // Tier 1 limits - adjust based on your actual tier
        Self {
            requests_per_minute: 50,
            input_tokens_per_minute: 40_000,
            output_tokens_per_minute: 8_000,
            max_concurrent_requests: 5,
        }
    }
}

/// Rate limiter implementation using token bucket algorithm
#[derive(Debug)]
pub struct RateLimiter {
    config: RateLimitConfig,
    requests_bucket: Arc<AtomicU64>,
    input_tokens_bucket: Arc<AtomicU64>,
    output_tokens_bucket: Arc<AtomicU64>,
    semaphore: Arc<Semaphore>,
    last_refill: Arc<AtomicU64>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests as usize));
        
        Self {
            requests_bucket: Arc::new(AtomicU64::new(config.requests_per_minute as u64)),
            input_tokens_bucket: Arc::new(AtomicU64::new(config.input_tokens_per_minute as u64)),
            output_tokens_bucket: Arc::new(AtomicU64::new(config.output_tokens_per_minute as u64)),
            semaphore,
            last_refill: Arc::new(AtomicU64::new(Self::current_timestamp())),
            config,
        }
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn refill_buckets(&self) {
        let now = Self::current_timestamp();
        let last_refill = self.last_refill.load(Ordering::Relaxed);
        
        if now > last_refill {
            let seconds_passed = now - last_refill;
            let refill_rate = 60; // 1 minute in seconds
            
            if seconds_passed >= refill_rate {
                // Full refill after a minute
                self.requests_bucket.store(self.config.requests_per_minute as u64, Ordering::Relaxed);
                self.input_tokens_bucket.store(self.config.input_tokens_per_minute as u64, Ordering::Relaxed);
                self.output_tokens_bucket.store(self.config.output_tokens_per_minute as u64, Ordering::Relaxed);
                self.last_refill.store(now, Ordering::Relaxed);
            } else {
                // Proportional refill
                let refill_fraction = seconds_passed as f64 / refill_rate as f64;
                
                let requests_to_add = (self.config.requests_per_minute as f64 * refill_fraction) as u64;
                let input_tokens_to_add = (self.config.input_tokens_per_minute as f64 * refill_fraction) as u64;
                let output_tokens_to_add = (self.config.output_tokens_per_minute as f64 * refill_fraction) as u64;
                
                self.requests_bucket.fetch_min(
                    self.requests_bucket.load(Ordering::Relaxed) + requests_to_add,
                    Ordering::Relaxed
                );
                self.input_tokens_bucket.fetch_min(
                    self.input_tokens_bucket.load(Ordering::Relaxed) + input_tokens_to_add,
                    Ordering::Relaxed
                );
                self.output_tokens_bucket.fetch_min(
                    self.output_tokens_bucket.load(Ordering::Relaxed) + output_tokens_to_add,
                    Ordering::Relaxed
                );
                
                if seconds_passed > 0 {
                    self.last_refill.store(now, Ordering::Relaxed);
                }
            }
        }
    }

    pub async fn acquire_permit(&self, estimated_input_tokens: u32) -> ClaudeResult<()> {
        self.refill_buckets();
        
        // Check if we have enough tokens
        let current_requests = self.requests_bucket.load(Ordering::Relaxed);
        let current_input_tokens = self.input_tokens_bucket.load(Ordering::Relaxed);
        
        if current_requests == 0 || current_input_tokens < estimated_input_tokens as u64 {
            return Err(ClaudeError::RateLimitError {
                message: "Rate limit would be exceeded".to_string(),
                retry_after: Some(Duration::from_secs(60)),
                requests_remaining: Some(current_requests as u32),
                tokens_remaining: Some(current_input_tokens as u32),
            });
        }

        // Acquire semaphore permit for concurrency control
        let _permit = self.semaphore.acquire().await.map_err(|_| {
            ClaudeError::ConfigurationError("Semaphore poisoned".to_string())
        })?;

        // Consume tokens
        self.requests_bucket.fetch_sub(1, Ordering::Relaxed);
        self.input_tokens_bucket.fetch_sub(estimated_input_tokens as u64, Ordering::Relaxed);

        Ok(())
    }

    pub fn record_response(&self, usage: &Usage) {
        // Record actual output tokens used
        self.output_tokens_bucket.fetch_sub(usage.output_tokens as u64, Ordering::Relaxed);
    }
}

/// Retry configuration using exponential backoff
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: usize,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Main Claude API client configuration
#[derive(Debug, Clone)]
pub struct ClaudeConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout: Duration,
    pub rate_limit: RateLimitConfig,
    pub retry: RetryConfig,
    pub user_agent: String,
}

impl ClaudeConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
            timeout: Duration::from_secs(300), // 5 minutes for long responses
            rate_limit: RateLimitConfig::default(),
            retry: RetryConfig::default(),
            user_agent: "fortitude-claude-client/1.0".to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_rate_limit(mut self, rate_limit: RateLimitConfig) -> Self {
        self.rate_limit = rate_limit;
        self
    }

    pub fn with_retry(mut self, retry: RetryConfig) -> Self {
        self.retry = retry;
        self
    }
}

/// Production-ready Claude API client with comprehensive error handling and retry logic
#[derive(Debug, Clone)]
pub struct ClaudeClient {
    client: Client,
    config: ClaudeConfig,
    rate_limiter: Arc<RateLimiter>,
    request_cache: Arc<DashMap<String, (ClaudeResponse, Instant)>>,
}

impl ClaudeClient {
    /// Create a new Claude client with the provided configuration
    pub fn new(config: ClaudeConfig) -> ClaudeResult<Self> {
        let client = ClientBuilder::new()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()
            .map_err(ClaudeError::HttpError)?;

        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit.clone()));
        let request_cache = Arc::new(DashMap::new());

        Ok(Self {
            client,
            config,
            rate_limiter,
            request_cache,
        })
    }

    /// Send a message to Claude with full error handling and retry logic
    pub async fn send_message(&self, request: ClaudeRequest) -> ClaudeResult<ClaudeResponse> {
        // Validate request
        self.validate_request(&request)?;

        // Generate cache key for idempotent requests
        let cache_key = self.generate_cache_key(&request);
        
        // Check cache first (for non-streaming requests)
        if request.stream.unwrap_or(false) == false {
            if let Some((cached_response, timestamp)) = self.request_cache.get(&cache_key) {
                if timestamp.elapsed() < Duration::from_secs(300) { // 5-minute cache
                    debug!("Returning cached response for request");
                    return Ok(cached_response.clone());
                }
            }
        }

        // Estimate input tokens for rate limiting (rough estimation)
        let estimated_input_tokens = self.estimate_input_tokens(&request);

        // Acquire rate limit permit
        self.rate_limiter.acquire_permit(estimated_input_tokens).await?;

        // Execute request with retry logic
        let response = self.execute_with_retry(request.clone()).await?;

        // Record usage for rate limiting
        self.rate_limiter.record_response(&response.usage);

        // Cache successful responses
        if request.stream.unwrap_or(false) == false {
            self.request_cache.insert(cache_key, (response.clone(), Instant::now()));
        }

        info!(
            "Request completed successfully. Input tokens: {}, Output tokens: {}",
            response.usage.input_tokens,
            response.usage.output_tokens
        );

        Ok(response)
    }

    /// Execute request with exponential backoff retry logic
    async fn execute_with_retry(&self, request: ClaudeRequest) -> ClaudeResult<ClaudeResponse> {
        let backoff = ExponentialBackoff {
            initial_interval: self.config.retry.initial_delay,
            max_interval: self.config.retry.max_delay,
            multiplier: self.config.retry.backoff_multiplier,
            max_elapsed_time: Some(Duration::from_secs(300)), // 5 minutes total
            ..ExponentialBackoff::default()
        };

        retry(backoff, || async {
            match self.execute_request(&request).await {
                Ok(response) => Ok(response),
                Err(ClaudeError::RateLimitError { retry_after, .. }) => {
                    warn!("Rate limit hit, backing off");
                    if let Some(delay) = retry_after {
                        sleep(delay).await;
                    }
                    Err(BackoffError::transient(ClaudeError::MaxRetriesExceeded))
                }
                Err(ClaudeError::HttpError { source }) if source.is_timeout() => {
                    warn!("Request timeout, retrying");
                    Err(BackoffError::transient(ClaudeError::TimeoutError {
                        duration: self.config.timeout,
                    }))
                }
                Err(ClaudeError::ApiError { status, .. }) 
                    if status.is_server_error() => {
                    warn!("Server error {}, retrying", status);
                    Err(BackoffError::transient(ClaudeError::MaxRetriesExceeded))
                }
                Err(e) => {
                    error!("Permanent error occurred: {}", e);
                    Err(BackoffError::permanent(e))
                }
            }
        })
        .await
        .map_err(|_| ClaudeError::MaxRetriesExceeded)
    }

    /// Execute a single HTTP request to the Claude API
    async fn execute_request(&self, request: &ClaudeRequest) -> ClaudeResult<ClaudeResponse> {
        let url = format!("{}/messages", self.config.base_url);
        let request_id = Uuid::new_v4().to_string();

        debug!("Sending request {} to Claude API", request_id);

        let response = timeout(
            self.config.timeout,
            self.client
                .post(&url)
                .header("x-api-key", &self.config.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .header("x-request-id", &request_id)
                .json(request)
                .send()
        ).await
        .map_err(|_| ClaudeError::TimeoutError {
            duration: self.config.timeout,
        })?
        .map_err(ClaudeError::HttpError)?;

        // Parse rate limit headers
        self.parse_rate_limit_headers(&response)?;

        let status = response.status();
        
        if status.is_success() {
            let claude_response: ClaudeResponse = response
                .json()
                .await
                .map_err(ClaudeError::HttpError)?;
            
            debug!("Request {} completed successfully", request_id);
            Ok(claude_response)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse structured error response
            if let Ok(api_error) = serde_json::from_str::<ApiErrorResponse>(&error_text) {
                match status {
                    StatusCode::TOO_MANY_REQUESTS => {
                        Err(ClaudeError::RateLimitError {
                            message: api_error.error.message,
                            retry_after: Some(Duration::from_secs(60)),
                            requests_remaining: None,
                            tokens_remaining: None,
                        })
                    }
                    StatusCode::UNAUTHORIZED => {
                        Err(ClaudeError::AuthenticationError(api_error.error.message))
                    }
                    _ => {
                        Err(ClaudeError::ApiError {
                            status,
                            message: api_error.error.message,
                            error_type: Some(api_error.error.r#type),
                        })
                    }
                }
            } else {
                Err(ClaudeError::ApiError {
                    status,
                    message: error_text,
                    error_type: None,
                })
            }
        }
    }

    /// Parse rate limit headers from response
    fn parse_rate_limit_headers(&self, response: &reqwest::Response) -> ClaudeResult<()> {
        // Parse headers like anthropic-ratelimit-requests-remaining
        if let Some(requests_remaining) = response.headers()
            .get("anthropic-ratelimit-requests-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
        {
            debug!("Requests remaining: {}", requests_remaining);
        }

        if let Some(reset_time) = response.headers()
            .get("anthropic-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
        {
            debug!("Rate limit reset time: {}", reset_time);
        }

        Ok(())
    }

    /// Validate request parameters
    fn validate_request(&self, request: &ClaudeRequest) -> ClaudeResult<()> {
        if request.messages.is_empty() {
            return Err(ClaudeError::ValidationError(
                "Messages cannot be empty".to_string()
            ));
        }

        if request.max_tokens == 0 || request.max_tokens > 4096 {
            return Err(ClaudeError::ValidationError(
                "max_tokens must be between 1 and 4096".to_string()
            ));
        }

        if request.messages.len() > 100_000 {
            return Err(ClaudeError::ValidationError(
                "Too many messages in conversation".to_string()
            ));
        }

        Ok(())
    }

    /// Estimate input tokens (rough approximation: 1 token ≈ 4 characters)
    fn estimate_input_tokens(&self, request: &ClaudeRequest) -> u32 {
        let mut total_chars = 0;
        
        for message in &request.messages {
            total_chars += message.content.len();
        }
        
        if let Some(system) = &request.system {
            total_chars += system.len();
        }

        // Rough estimation: 1 token ≈ 4 characters
        ((total_chars / 4) as u32).max(1)
    }

    /// Generate cache key for request
    fn generate_cache_key(&self, request: &ClaudeRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        serde_json::to_string(request).unwrap_or_default().hash(&mut hasher);
        format!("claude_request_{:x}", hasher.finish())
    }

    /// Clear response cache
    pub fn clear_cache(&self) {
        self.request_cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        let total = self.request_cache.len();
        let expired = self.request_cache
            .iter()
            .filter(|entry| entry.value().1.elapsed() > Duration::from_secs(300))
            .count();
        
        (total, expired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_client_creation() {
        let config = ClaudeConfig::new("test-key".to_string());
        let client = ClaudeClient::new(config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_request_validation() {
        let config = ClaudeConfig::new("test-key".to_string());
        let client = ClaudeClient::new(config).unwrap();
        
        let invalid_request = ClaudeRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            max_tokens: 0, // Invalid
            messages: vec![],
            system: None,
            temperature: None,
            top_p: None,
            stop_sequences: None,
            stream: None,
        };

        assert!(client.validate_request(&invalid_request).is_err());
    }

    #[tokio::test]
    async fn test_token_estimation() {
        let config = ClaudeConfig::new("test-key".to_string());
        let client = ClaudeClient::new(config).unwrap();
        
        let request = ClaudeRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            max_tokens: 1024,
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello, world!".to_string(),
            }],
            system: None,
            temperature: None,
            top_p: None,
            stop_sequences: None,
            stream: None,
        };

        let estimated = client.estimate_input_tokens(&request);
        assert!(estimated > 0);
    }
}

// Example usage demonstrating the complete client implementation
#[cfg(feature = "example")]
mod example {
    use super::*;
    use std::env;

    #[tokio::main]
    async fn main() -> ClaudeResult<()> {
        // Initialize tracing
        tracing_subscriber::init();

        // Get API key from environment
        let api_key = env::var("CLAUDE_API_KEY")
            .map_err(|_| ClaudeError::ConfigurationError(
                "CLAUDE_API_KEY environment variable not set".to_string()
            ))?;

        // Configure client with custom settings
        let config = ClaudeConfig::new(api_key)
            .with_timeout(Duration::from_secs(120))
            .with_rate_limit(RateLimitConfig {
                requests_per_minute: 100,  // Adjust based on your tier
                input_tokens_per_minute: 80_000,
                output_tokens_per_minute: 16_000,
                max_concurrent_requests: 10,
            })
            .with_retry(RetryConfig {
                max_retries: 5,
                initial_delay: Duration::from_millis(500),
                max_delay: Duration::from_secs(30),
                backoff_multiplier: 2.0,
                jitter: true,
            });

        // Create client
        let client = ClaudeClient::new(config)?;

        // Prepare request
        let request = ClaudeRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            max_tokens: 1024,
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: "Explain quantum computing in simple terms.".to_string(),
                }
            ],
            system: Some("You are a helpful AI assistant specialized in explaining complex topics clearly.".to_string()),
            temperature: Some(0.7),
            top_p: None,
            stop_sequences: None,
            stream: Some(false),
        };

        // Send request with automatic retry and rate limiting
        match client.send_message(request).await {
            Ok(response) => {
                println!("Response ID: {}", response.id);
                println!("Model: {}", response.model);
                
                for content in &response.content {
                    println!("Content: {}", content.text);
                }
                
                println!("Usage - Input: {}, Output: {}, Total: {}", 
                    response.usage.input_tokens,
                    response.usage.output_tokens,
                    response.usage.input_tokens + response.usage.output_tokens
                );

                // Cache statistics
                let (total_cached, expired) = client.cache_stats();
                println!("Cache: {} total, {} expired", total_cached, expired);
            }
            Err(e) => {
                error!("Request failed: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }
}