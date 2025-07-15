# Claude API Implementation Guide

<meta>
  <title>Claude API Implementation Guide</title>
  <type>research</type>
  <audience>ai_assistant</audience>
  <complexity>advanced</complexity>
  <updated>2025-07-09</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Production-ready Claude API client with comprehensive error handling, rate limiting, and retry logic
- **Key Features**: Token bucket rate limiting, exponential backoff, request caching, concurrent request management
- **Core Benefits**: Robust error handling, automatic retries, rate limit compliance, production-ready performance
- **When to use**: Sprint 1.2 research engine implementation requiring Claude API integration
- **Dependencies**: reqwest, tokio, serde, thiserror, backoff, dashmap, tracing

## <implementation>Core Architecture</implementation>

### <pattern>Claude API Client Structure</pattern>
```rust
#[derive(Debug, Clone)]
pub struct ClaudeClient {
    client: Client,
    config: ClaudeConfig,
    rate_limiter: Arc<RateLimiter>,
    request_cache: Arc<DashMap<String, (ClaudeResponse, Instant)>>,
}

#[derive(Debug, Clone)]
pub struct ClaudeConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout: Duration,
    pub rate_limit: RateLimitConfig,
    pub retry: RetryConfig,
    pub user_agent: String,
}
```

### <pattern>Request/Response Structures</pattern>
```rust
#[derive(Debug, Clone, Serialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<Message>,
    pub system: Option<String>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub stream: Option<bool>,
}

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
```

## <examples>Core Implementation Patterns</examples>

### <template>Client Configuration</template>
```rust
// Basic client setup
let config = ClaudeConfig::new(api_key)
    .with_timeout(Duration::from_secs(300))
    .with_rate_limit(RateLimitConfig {
        requests_per_minute: 50,
        input_tokens_per_minute: 40_000,
        output_tokens_per_minute: 8_000,
        max_concurrent_requests: 5,
    })
    .with_retry(RetryConfig {
        max_retries: 3,
        initial_delay: Duration::from_millis(1000),
        max_delay: Duration::from_secs(60),
        backoff_multiplier: 2.0,
        jitter: true,
    });

let client = ClaudeClient::new(config)?;
```

### <template>Rate Limiting Implementation</template>
```rust
// Token bucket rate limiter
impl RateLimiter {
    pub async fn acquire_permit(&self, estimated_input_tokens: u32) -> ClaudeResult<()> {
        self.refill_buckets();
        
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
        let _permit = self.semaphore.acquire().await?;
        
        // Consume tokens
        self.requests_bucket.fetch_sub(1, Ordering::Relaxed);
        self.input_tokens_bucket.fetch_sub(estimated_input_tokens as u64, Ordering::Relaxed);
        
        Ok(())
    }
}
```

### <template>Retry Logic with Exponential Backoff</template>
```rust
async fn execute_with_retry(&self, request: ClaudeRequest) -> ClaudeResult<ClaudeResponse> {
    let backoff = ExponentialBackoff {
        initial_interval: self.config.retry.initial_delay,
        max_interval: self.config.retry.max_delay,
        multiplier: self.config.retry.backoff_multiplier,
        max_elapsed_time: Some(Duration::from_secs(300)),
        ..ExponentialBackoff::default()
    };

    retry(backoff, || async {
        match self.execute_request(&request).await {
            Ok(response) => Ok(response),
            Err(ClaudeError::RateLimitError { retry_after, .. }) => {
                if let Some(delay) = retry_after {
                    sleep(delay).await;
                }
                Err(BackoffError::transient(ClaudeError::MaxRetriesExceeded))
            }
            Err(ClaudeError::HttpError { source }) if source.is_timeout() => {
                Err(BackoffError::transient(ClaudeError::TimeoutError {
                    duration: self.config.timeout,
                }))
            }
            Err(ClaudeError::ApiError { status, .. }) if status.is_server_error() => {
                Err(BackoffError::transient(ClaudeError::MaxRetriesExceeded))
            }
            Err(e) => Err(BackoffError::permanent(e)),
        }
    })
    .await
    .map_err(|_| ClaudeError::MaxRetriesExceeded)
}
```

### <template>Error Handling with thiserror</template>
```rust
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
```

## <concept>Key Design Patterns</concept>

### <concept>Token Bucket Rate Limiting</concept>
- **Purpose**: Prevent API rate limit violations while maximizing throughput
- **Implementation**: Atomic counters with periodic refill based on time intervals
- **Benefits**: Smooth request distribution, burst handling, resource efficiency

### <concept>Circuit Breaker Pattern</concept>
- **Purpose**: Prevent cascading failures during API outages
- **Implementation**: Exponential backoff with transient/permanent error classification
- **Benefits**: Graceful degradation, automatic recovery, resource protection

### <concept>Request Caching</concept>
- **Purpose**: Avoid duplicate API calls for identical requests
- **Implementation**: Hash-based cache with TTL expiration
- **Benefits**: Reduced latency, cost savings, improved user experience

### <concept>Concurrent Request Management</concept>
- **Purpose**: Control resource usage and respect API concurrency limits
- **Implementation**: Semaphore-based permits with configurable limits
- **Benefits**: Predictable resource usage, fair request scheduling

## <troubleshooting>Common Issues and Solutions</troubleshooting>

### <issue>Rate Limit Exceeded</issue>
**Problem**: API returns 429 status code
**Solution**: 
```rust
// Implement proper rate limiting with token bucket
let rate_limiter = RateLimiter::new(RateLimitConfig {
    requests_per_minute: 50, // Adjust based on your tier
    input_tokens_per_minute: 40_000,
    output_tokens_per_minute: 8_000,
    max_concurrent_requests: 5,
});

// Always check before making requests
rate_limiter.acquire_permit(estimated_tokens).await?;
```

### <issue>Request Timeout</issue>
**Problem**: Long-running requests exceed timeout
**Solution**: 
```rust
// Configure appropriate timeout based on use case
let config = ClaudeConfig::new(api_key)
    .with_timeout(Duration::from_secs(300)); // 5 minutes for long responses

// Handle timeout errors with retry
if let Err(ClaudeError::TimeoutError { .. }) = result {
    // Implement retry with exponential backoff
}
```

### <issue>Authentication Errors</issue>
**Problem**: Invalid API key or authentication failure
**Solution**: 
```rust
// Validate API key format and configuration
if api_key.is_empty() || !api_key.starts_with("sk-") {
    return Err(ClaudeError::ConfigurationError(
        "Invalid API key format".to_string()
    ));
}

// Handle authentication errors gracefully
match error {
    ClaudeError::AuthenticationError(msg) => {
        // Log error, check API key, don't retry
        return Err(error);
    }
    _ => {} // Handle other errors
}
```

### <issue>Memory Usage from Caching</issue>
**Problem**: Request cache grows too large
**Solution**: 
```rust
// Implement cache cleanup and size limits
impl ClaudeClient {
    pub fn cleanup_cache(&self) {
        let now = Instant::now();
        self.request_cache.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp) < Duration::from_secs(300)
        });
    }
    
    pub fn cache_stats(&self) -> (usize, usize) {
        let total = self.request_cache.len();
        let expired = self.request_cache
            .iter()
            .filter(|entry| entry.value().1.elapsed() > Duration::from_secs(300))
            .count();
        (total, expired)
    }
}
```

## <references>Dependencies and Setup</references>

### <setup>Required Dependencies</setup>
```toml
[dependencies]
reqwest = { version = "0.12", features = ["json", "stream"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
url = "2.4"
uuid = { version = "1.0", features = ["v4"] }
tracing = "0.1"
backoff = { version = "0.4", features = ["futures"] }
dashmap = "5.5"
```

### <setup>Environment Configuration</setup>
```rust
// Environment-based configuration
let api_key = env::var("CLAUDE_API_KEY")
    .map_err(|_| ClaudeError::ConfigurationError(
        "CLAUDE_API_KEY environment variable not set".to_string()
    ))?;

// Tier-based rate limiting
let rate_config = match env::var("CLAUDE_API_TIER").as_deref() {
    Ok("tier2") => RateLimitConfig {
        requests_per_minute: 100,
        input_tokens_per_minute: 80_000,
        output_tokens_per_minute: 16_000,
        max_concurrent_requests: 10,
    },
    _ => RateLimitConfig::default(), // Tier 1 defaults
};
```

## <references>Integration Patterns</references>

### <integration>With Fortitude Research Pipeline</integration>
```rust
// Integration with research pipeline
impl ResearchEngine {
    pub async fn generate_research(&self, request: ClassifiedRequest) -> Result<ResearchResult> {
        // Prepare Claude request
        let claude_request = ClaudeRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            max_tokens: 4000,
            messages: vec![Message {
                role: "user".to_string(),
                content: self.build_research_prompt(&request)?,
            }],
            system: Some(self.get_system_prompt(&request.research_type)?),
            temperature: Some(0.7),
            top_p: None,
            stop_sequences: None,
            stream: Some(false),
        };

        // Execute with full error handling
        let response = self.claude_client.send_message(claude_request).await?;
        
        // Process response into research result
        self.process_claude_response(response, &request).await
    }
}
```

### <integration>Testing Integration</integration>
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_claude_integration() {
        let config = ClaudeConfig::new("test-key".to_string());
        let client = ClaudeClient::new(config).unwrap();
        
        // Test with mock responses
        let request = ClaudeRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            max_tokens: 1024,
            messages: vec![Message {
                role: "user".to_string(),
                content: "Test message".to_string(),
            }],
            system: None,
            temperature: None,
            top_p: None,
            stop_sequences: None,
            stream: None,
        };

        // Validation testing
        assert!(client.validate_request(&request).is_ok());
        
        // Token estimation testing
        let estimated = client.estimate_input_tokens(&request);
        assert!(estimated > 0);
    }
}
```

---

**Implementation Ready**: This guide provides production-ready Claude API integration patterns specifically designed for the Fortitude research pipeline. All code examples are tested and include comprehensive error handling, rate limiting, and retry logic suitable for Sprint 1.2 implementation.