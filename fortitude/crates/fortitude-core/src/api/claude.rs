// ABOUTME: Claude API client implementation with rate limiting and retry logic
use async_trait::async_trait;
use backoff::{future::retry, Error as BackoffError, ExponentialBackoff};
use dashmap::DashMap;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;
use tracing::{debug, info, warn};

use crate::api::client::{
    ApiClient, ApiConfig, HealthStatus, RateLimitConfig, RequestCost, RetryConfig,
};
use crate::api::error::{ApiError, ApiResult};
use crate::classification::context_detector::ContextDetectionResult;
use fortitude_types::classification_result::{AudienceLevel, TechnicalDomain, UrgencyLevel};
use fortitude_types::ResearchType;

/// Claude API client configuration
#[derive(Debug, Clone)]
pub struct ClaudeConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout: Duration,
    pub rate_limit: RateLimitConfig,
    pub retry: RetryConfig,
    pub user_agent: String,
    pub model: String,
}

impl ClaudeConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com".to_string(),
            timeout: Duration::from_secs(300),
            rate_limit: RateLimitConfig::default(),
            retry: RetryConfig::default(),
            user_agent: format!("fortitude/{}", env!("CARGO_PKG_VERSION")),
            model: "claude-3-sonnet-20240229".to_string(),
        }
    }
}

impl ApiConfig for ClaudeConfig {
    fn validate(&self) -> ApiResult<()> {
        if self.api_key.is_empty() {
            return Err(ApiError::ConfigurationError(
                "API key is required".to_string(),
            ));
        }

        if !self.api_key.starts_with("sk-") {
            return Err(ApiError::ConfigurationError(
                "Invalid API key format".to_string(),
            ));
        }

        if self.timeout.as_secs() < 1 {
            return Err(ApiError::ConfigurationError(
                "Timeout must be at least 1 second".to_string(),
            ));
        }

        Ok(())
    }

    fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn with_rate_limit(mut self, rate_limit: RateLimitConfig) -> Self {
        self.rate_limit = rate_limit;
        self
    }

    fn with_retry(mut self, retry: RetryConfig) -> Self {
        self.retry = retry;
        self
    }
}

/// Claude API request structure
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

/// Context-aware Claude API request builder
#[derive(Debug, Clone)]
pub struct ContextAwareClaudeRequest {
    pub base_request: ClaudeRequest,
    pub research_type: ResearchType,
    pub context_result: Option<ContextDetectionResult>,
    pub original_query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Message {
    pub role: String,
    pub content: String,
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
}

/// Rate limiter with token bucket algorithm
#[derive(Debug)]
pub struct RateLimiter {
    requests_bucket: AtomicU32,
    input_tokens_bucket: AtomicU64,
    output_tokens_bucket: AtomicU64,
    last_refill: Arc<std::sync::RwLock<Instant>>,
    config: RateLimitConfig,
    semaphore: Arc<Semaphore>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests as usize));

        Self {
            requests_bucket: AtomicU32::new(config.requests_per_minute),
            input_tokens_bucket: AtomicU64::new(config.input_tokens_per_minute as u64),
            output_tokens_bucket: AtomicU64::new(config.output_tokens_per_minute as u64),
            last_refill: Arc::new(std::sync::RwLock::new(Instant::now())),
            config,
            semaphore,
        }
    }

    pub async fn acquire_permit(&self, estimated_input_tokens: u32) -> ApiResult<()> {
        self.refill_buckets();

        let current_requests = self.requests_bucket.load(Ordering::Relaxed);
        let current_input_tokens = self.input_tokens_bucket.load(Ordering::Relaxed);

        if current_requests == 0 || current_input_tokens < estimated_input_tokens as u64 {
            return Err(ApiError::RateLimitError {
                message: "Rate limit would be exceeded".to_string(),
                retry_after: Some(Duration::from_secs(60)),
                requests_remaining: Some(current_requests),
                tokens_remaining: Some(current_input_tokens as u32),
            });
        }

        // Acquire semaphore permit for concurrency control
        let _permit = self.semaphore.acquire().await.map_err(|_| {
            ApiError::ServiceUnavailable("Semaphore acquisition failed".to_string())
        })?;

        // Consume tokens
        self.requests_bucket.fetch_sub(1, Ordering::Relaxed);
        self.input_tokens_bucket
            .fetch_sub(estimated_input_tokens as u64, Ordering::Relaxed);

        Ok(())
    }

    fn refill_buckets(&self) {
        let now = Instant::now();
        let mut last_refill = self.last_refill.write().unwrap();
        let elapsed = now.duration_since(*last_refill);

        if elapsed >= Duration::from_secs(60) {
            let minutes_elapsed = elapsed.as_secs() / 60;

            // Refill buckets proportionally
            let requests_to_add = (self.config.requests_per_minute as u64 * minutes_elapsed) as u32;
            let input_tokens_to_add = self.config.input_tokens_per_minute as u64 * minutes_elapsed;
            let output_tokens_to_add =
                self.config.output_tokens_per_minute as u64 * minutes_elapsed;

            self.requests_bucket.fetch_min(
                self.requests_bucket.load(Ordering::Relaxed) + requests_to_add,
                Ordering::Relaxed,
            );

            self.input_tokens_bucket.fetch_min(
                self.input_tokens_bucket.load(Ordering::Relaxed) + input_tokens_to_add,
                Ordering::Relaxed,
            );

            self.output_tokens_bucket.fetch_min(
                self.output_tokens_bucket.load(Ordering::Relaxed) + output_tokens_to_add,
                Ordering::Relaxed,
            );

            *last_refill = now;
        }
    }
}

/// Claude API client implementation
#[derive(Debug, Clone)]
pub struct ClaudeClient {
    client: Client,
    config: ClaudeConfig,
    rate_limiter: Arc<RateLimiter>,
    request_cache: Arc<DashMap<String, (ClaudeResponse, Instant)>>,
}

#[async_trait]
impl ApiClient for ClaudeClient {
    type Request = ClaudeRequest;
    type Response = ClaudeResponse;
    type Config = ClaudeConfig;

    fn new(config: Self::Config) -> ApiResult<Self> {
        config.validate()?;

        let client = Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .build()
            .map_err(|e| {
                ApiError::ConfigurationError(format!("Failed to create HTTP client: {e}"))
            })?;

        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit.clone()));
        let request_cache = Arc::new(DashMap::new());

        Ok(Self {
            client,
            config,
            rate_limiter,
            request_cache,
        })
    }

    async fn send_request(&self, request: Self::Request) -> ApiResult<Self::Response> {
        self.validate_request(&request)?;

        // Check cache first
        let cache_key = self.generate_cache_key(&request);
        if let Some(cache_entry) = self.request_cache.get(&cache_key) {
            let (response, timestamp) = cache_entry.value();
            if timestamp.elapsed() < Duration::from_secs(300) {
                debug!("Cache hit for request");
                return Ok(response.clone());
            }
        }

        // Execute with retry logic
        let response = self.execute_with_retry(request).await?;

        // Cache the response
        self.request_cache
            .insert(cache_key, (response.clone(), Instant::now()));

        Ok(response)
    }

    fn validate_request(&self, request: &Self::Request) -> ApiResult<()> {
        if request.messages.is_empty() {
            return Err(ApiError::ValidationError(
                "Messages cannot be empty".to_string(),
            ));
        }

        if request.max_tokens == 0 {
            return Err(ApiError::ValidationError(
                "max_tokens must be greater than 0".to_string(),
            ));
        }

        if request.max_tokens > 4096 {
            return Err(ApiError::ValidationError(
                "max_tokens cannot exceed 4096".to_string(),
            ));
        }

        // Validate message roles
        for message in &request.messages {
            if message.role != "user" && message.role != "assistant" {
                return Err(ApiError::ValidationError(format!(
                    "Invalid message role: {}",
                    message.role
                )));
            }
        }

        Ok(())
    }

    fn estimate_cost(&self, request: &Self::Request) -> ApiResult<RequestCost> {
        let estimated_input_tokens = self.estimate_input_tokens(request);
        let estimated_output_tokens = request.max_tokens;

        Ok(RequestCost {
            estimated_input_tokens,
            estimated_output_tokens,
            estimated_duration: Duration::from_secs(10), // Conservative estimate
            estimated_cost_usd: Some(
                self.calculate_cost_usd(estimated_input_tokens, estimated_output_tokens),
            ),
        })
    }

    async fn health_check(&self) -> ApiResult<HealthStatus> {
        let test_request = ClaudeRequest {
            model: self.config.model.clone(),
            max_tokens: 1,
            messages: vec![Message {
                role: "user".to_string(),
                content: "Test".to_string(),
            }],
            system: None,
            temperature: None,
            top_p: None,
            stop_sequences: None,
            stream: Some(false),
        };

        match self.send_request(test_request).await {
            Ok(_) => Ok(HealthStatus::Healthy),
            Err(ApiError::RateLimitError { .. }) => {
                Ok(HealthStatus::Degraded("Rate limited".to_string()))
            }
            Err(e) => Ok(HealthStatus::Unhealthy(format!("API error: {e}"))),
        }
    }
}

impl ClaudeClient {
    async fn execute_with_retry(&self, request: ClaudeRequest) -> ApiResult<ClaudeResponse> {
        let backoff = ExponentialBackoff {
            initial_interval: self.config.retry.initial_delay,
            max_interval: self.config.retry.max_delay,
            multiplier: self.config.retry.backoff_multiplier,
            max_elapsed_time: Some(Duration::from_secs(300)),
            ..ExponentialBackoff::default()
        };

        let result = retry(backoff, || async {
            match self.execute_request(&request).await {
                Ok(response) => Ok(response),
                Err(ApiError::RateLimitError { retry_after, .. }) => {
                    if let Some(delay) = retry_after {
                        sleep(delay).await;
                    }
                    Err(BackoffError::transient(ApiError::MaxRetriesExceeded))
                }
                Err(ApiError::TimeoutError { .. }) => {
                    Err(BackoffError::transient(ApiError::MaxRetriesExceeded))
                }
                Err(ApiError::ApiError { status, .. }) if status.is_server_error() => {
                    Err(BackoffError::transient(ApiError::MaxRetriesExceeded))
                }
                Err(e) => Err(BackoffError::permanent(e)),
            }
        })
        .await;

        result.map_err(|_| ApiError::MaxRetriesExceeded)
    }

    async fn execute_request(&self, request: &ClaudeRequest) -> ApiResult<ClaudeResponse> {
        let estimated_tokens = self.estimate_input_tokens(request);
        self.rate_limiter.acquire_permit(estimated_tokens).await?;

        let url = format!("{}/v1/messages", self.config.base_url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(request)
            .send()
            .await
            .map_err(|e| ApiError::HttpError { source: e })?;

        let status = response.status();

        if status.is_success() {
            let claude_response: ClaudeResponse = response
                .json()
                .await
                .map_err(|e| ApiError::HttpError { source: e })?;
            info!(
                "Claude API request successful - input: {}, output: {}",
                claude_response.usage.input_tokens, claude_response.usage.output_tokens
            );
            Ok(claude_response)
        } else {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            match status {
                StatusCode::TOO_MANY_REQUESTS => {
                    warn!("Claude API rate limit exceeded");
                    Err(ApiError::RateLimitError {
                        message: error_body,
                        retry_after: Some(Duration::from_secs(60)),
                        requests_remaining: Some(0),
                        tokens_remaining: Some(0),
                    })
                }
                StatusCode::UNAUTHORIZED => Err(ApiError::AuthenticationError(error_body)),
                StatusCode::REQUEST_TIMEOUT => Err(ApiError::TimeoutError {
                    duration: self.config.timeout,
                }),
                _ => Err(ApiError::ApiError {
                    status,
                    message: error_body,
                    error_type: None,
                }),
            }
        }
    }

    fn estimate_input_tokens(&self, request: &ClaudeRequest) -> u32 {
        let mut token_count = 0;

        // Estimate tokens for messages
        for message in &request.messages {
            token_count += (message.content.len() / 4) as u32; // Rough estimate: 4 chars per token
        }

        // Add system prompt if present
        if let Some(system) = &request.system {
            token_count += (system.len() / 4) as u32;
        }

        // Add buffer for model overhead
        token_count + 50
    }

    fn calculate_cost_usd(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        // Claude 3 Sonnet pricing (example rates)
        let input_cost_per_token = 0.000003; // $3 per 1M tokens
        let output_cost_per_token = 0.000015; // $15 per 1M tokens

        (input_tokens as f64 * input_cost_per_token)
            + (output_tokens as f64 * output_cost_per_token)
    }

    fn generate_cache_key(&self, request: &ClaudeRequest) -> String {
        let mut hasher = DefaultHasher::new();
        request.messages.hash(&mut hasher);
        request.system.hash(&mut hasher);
        if let Some(temp) = request.temperature {
            temp.to_bits().hash(&mut hasher);
        }

        format!("claude_request_{:x}", hasher.finish())
    }

    /// Generate context-aware cache key for Claude request
    fn generate_context_aware_cache_key(&self, request: &ContextAwareClaudeRequest) -> String {
        let mut hasher = DefaultHasher::new();
        request.base_request.messages.hash(&mut hasher);
        request.base_request.system.hash(&mut hasher);
        request.research_type.hash(&mut hasher);
        request.original_query.hash(&mut hasher);

        if let Some(temp) = request.base_request.temperature {
            temp.to_bits().hash(&mut hasher);
        }

        // Include context information in cache key
        if let Some(ref context) = request.context_result {
            context.audience_level.display_name().hash(&mut hasher);
            context.technical_domain.display_name().hash(&mut hasher);
            context.urgency_level.display_name().hash(&mut hasher);

            // Round confidence to 2 decimal places for cache key stability
            let confidence_rounded = (context.overall_confidence * 100.0).round() as u32;
            confidence_rounded.hash(&mut hasher);
        }

        format!("claude_context_request_{:x}", hasher.finish())
    }

    /// Create a context-aware Claude request with enhanced prompting
    pub fn create_context_aware_request(
        &self,
        query: &str,
        research_type: &ResearchType,
        context_result: Option<&ContextDetectionResult>,
    ) -> ContextAwareClaudeRequest {
        let system_prompt =
            self.generate_context_aware_system_prompt(research_type, context_result);
        let user_message =
            self.generate_context_aware_user_message(query, research_type, context_result);

        let base_request = ClaudeRequest {
            model: self.config.model.clone(),
            max_tokens: self.determine_max_tokens(research_type, context_result),
            messages: vec![Message {
                role: "user".to_string(),
                content: user_message,
            }],
            system: Some(system_prompt),
            temperature: Some(self.determine_temperature(research_type, context_result)),
            top_p: None,
            stop_sequences: None,
            stream: Some(false),
        };

        ContextAwareClaudeRequest {
            base_request,
            research_type: research_type.clone(),
            context_result: context_result.cloned(),
            original_query: query.to_string(),
        }
    }

    /// Generate context-aware system prompt
    fn generate_context_aware_system_prompt(
        &self,
        research_type: &ResearchType,
        context_result: Option<&ContextDetectionResult>,
    ) -> String {
        let mut system_prompt = String::new();

        // Base system prompt based on research type
        system_prompt.push_str(match research_type {
            ResearchType::Decision => "You are a technical decision-making assistant. Your role is to analyze options, provide structured recommendations, and help users make informed technical decisions.",
            ResearchType::Implementation => "You are a technical implementation guide. Your role is to provide clear, actionable implementation steps, code examples, and best practices for technical tasks.",
            ResearchType::Troubleshooting => "You are a technical troubleshooting expert. Your role is to help diagnose problems, provide systematic debugging approaches, and suggest solutions.",
            ResearchType::Learning => "You are a technical educator. Your role is to explain concepts clearly, provide learning resources, and adapt your explanations to the user's level.",
            ResearchType::Validation => "You are a technical validation specialist. Your role is to help verify implementations, review approaches, and ensure quality standards are met.",
        });

        // Add context-specific instructions
        if let Some(context) = context_result {
            system_prompt.push_str("\n\n## Context-Specific Instructions\n");

            // Audience level adaptations
            match context.audience_level {
                AudienceLevel::Beginner => {
                    system_prompt.push_str("- The user is a beginner. Provide detailed explanations, define technical terms, and include step-by-step instructions.\n");
                    system_prompt.push_str("- Avoid assuming prior knowledge and explain concepts from first principles.\n");
                }
                AudienceLevel::Intermediate => {
                    system_prompt.push_str("- The user has intermediate knowledge. Provide balanced explanations with some technical detail.\n");
                    system_prompt.push_str("- You can assume familiarity with basic concepts but explain advanced topics.\n");
                }
                AudienceLevel::Advanced => {
                    system_prompt.push_str(
                        "- The user is advanced. Provide concise, technically precise responses.\n",
                    );
                    system_prompt.push_str(
                        "- Focus on nuanced details, edge cases, and advanced considerations.\n",
                    );
                    system_prompt.push_str(
                        "- Include optimization insights and architectural considerations.\n",
                    );
                }
            }

            // Technical domain adaptations
            match context.technical_domain {
                TechnicalDomain::Rust => {
                    system_prompt.push_str("- Focus on Rust-specific concepts: ownership, borrowing, lifetimes, and safety.\n");
                    system_prompt.push_str(
                        "- Provide idiomatic Rust code examples and mention relevant crates.\n",
                    );
                }
                TechnicalDomain::Web => {
                    system_prompt.push_str("- Focus on web development concepts: frontend, backend, APIs, and web standards.\n");
                    system_prompt.push_str("- Consider browser compatibility, performance, and security implications.\n");
                }
                TechnicalDomain::AI => {
                    system_prompt.push_str("- Focus on data science concepts: analysis, visualization, machine learning, and statistics.\n");
                    system_prompt.push_str("- Provide examples with popular data science libraries and best practices.\n");
                }
                TechnicalDomain::DevOps => {
                    system_prompt.push_str("- Focus on DevOps concepts: CI/CD, containerization, infrastructure, and automation.\n");
                    system_prompt.push_str(
                        "- Consider scalability, reliability, and operational best practices.\n",
                    );
                }
                TechnicalDomain::Systems => {
                    system_prompt.push_str("- Focus on system programming: performance, memory management, and low-level concepts.\n");
                    system_prompt.push_str(
                        "- Consider efficiency, concurrency, and resource constraints.\n",
                    );
                }
                TechnicalDomain::Database => {
                    system_prompt.push_str("- Focus on database concepts: data modeling, queries, performance, and transactions.\n");
                    system_prompt.push_str(
                        "- Consider scalability, consistency, and optimization techniques.\n",
                    );
                }
                TechnicalDomain::Security => {
                    system_prompt.push_str("- Focus on security concepts: cryptography, threat models, and secure coding practices.\n");
                    system_prompt.push_str(
                        "- Consider vulnerabilities, attack vectors, and defense strategies.\n",
                    );
                }
                TechnicalDomain::Python => {
                    system_prompt.push_str("- Focus on Python-specific concepts: virtual environments, package management, and Pythonic code.\n");
                    system_prompt.push_str(
                        "- Provide examples with popular Python libraries and best practices.\n",
                    );
                }
                TechnicalDomain::Architecture => {
                    system_prompt.push_str("- Focus on architectural concepts: system design, patterns, and structural decisions.\n");
                    system_prompt.push_str(
                        "- Consider scalability, maintainability, and design trade-offs.\n",
                    );
                }
                TechnicalDomain::General => {
                    system_prompt.push_str(
                        "- Provide general technical guidance applicable across domains.\n",
                    );
                }
            }

            // Urgency level adaptations
            match context.urgency_level {
                UrgencyLevel::Immediate => {
                    system_prompt.push_str(
                        "- This is an urgent request. Prioritize quick, actionable solutions.\n",
                    );
                    system_prompt.push_str("- Focus on immediate fixes and workarounds, with follow-up recommendations.\n");
                }
                UrgencyLevel::Planned => {
                    system_prompt.push_str("- This is a planned request. Provide comprehensive, well-structured responses.\n");
                    system_prompt.push_str(
                        "- Include implementation considerations and long-term implications.\n",
                    );
                }
                UrgencyLevel::Exploratory => {
                    system_prompt.push_str("- This is an exploratory request. Provide educational, detailed responses.\n");
                    system_prompt.push_str(
                        "- Include background information, alternatives, and learning resources.\n",
                    );
                }
            }
        }

        system_prompt.push_str("\n\n## Response Format\n");
        system_prompt.push_str("- Provide clear, structured responses with proper formatting.\n");
        system_prompt.push_str("- Use code blocks for examples and commands.\n");
        system_prompt.push_str("- Include relevant links and resources when helpful.\n");
        system_prompt.push_str("- Be concise but comprehensive.");

        system_prompt
    }

    /// Generate context-aware user message
    fn generate_context_aware_user_message(
        &self,
        query: &str,
        research_type: &ResearchType,
        context_result: Option<&ContextDetectionResult>,
    ) -> String {
        let mut user_message = String::new();

        // Add context information if available
        if let Some(context) = context_result {
            user_message.push_str(&format!(
                "[Context: {} request for {} audience in {} domain with {} urgency]\n\n",
                research_type,
                context.audience_level.display_name(),
                context.technical_domain.display_name(),
                context.urgency_level.display_name()
            ));
        }

        // Add the original query
        user_message.push_str(query);

        // Add research type specific prompts
        match research_type {
            ResearchType::Decision => {
                user_message.push_str("\n\nPlease provide a structured decision analysis with:");
                user_message.push_str("\n- Clear problem statement");
                user_message.push_str("\n- Available options with pros/cons");
                user_message.push_str("\n- Recommended approach with reasoning");
                user_message.push_str("\n- Implementation considerations");
            }
            ResearchType::Implementation => {
                user_message.push_str("\n\nPlease provide implementation guidance with:");
                user_message.push_str("\n- Step-by-step implementation plan");
                user_message.push_str("\n- Code examples where relevant");
                user_message.push_str("\n- Best practices and pitfalls to avoid");
                user_message.push_str("\n- Testing and validation approaches");
            }
            ResearchType::Troubleshooting => {
                user_message.push_str("\n\nPlease provide troubleshooting guidance with:");
                user_message.push_str("\n- Systematic diagnostic approach");
                user_message.push_str("\n- Common causes and solutions");
                user_message.push_str("\n- Debug commands and techniques");
                user_message.push_str("\n- Prevention strategies");
            }
            ResearchType::Learning => {
                user_message.push_str("\n\nPlease provide educational content with:");
                user_message.push_str("\n- Clear concept explanations");
                user_message.push_str("\n- Practical examples");
                user_message.push_str("\n- Learning progression and next steps");
                user_message.push_str("\n- Additional resources");
            }
            ResearchType::Validation => {
                user_message.push_str("\n\nPlease provide validation guidance with:");
                user_message.push_str("\n- Quality assessment criteria");
                user_message.push_str("\n- Testing strategies");
                user_message.push_str("\n- Common issues to check for");
                user_message.push_str("\n- Improvement recommendations");
            }
        }

        user_message
    }

    /// Determine optimal max tokens based on context
    fn determine_max_tokens(
        &self,
        research_type: &ResearchType,
        context_result: Option<&ContextDetectionResult>,
    ) -> u32 {
        let base_tokens = match research_type {
            ResearchType::Decision => 2048,
            ResearchType::Implementation => 3072,
            ResearchType::Troubleshooting => 2048,
            ResearchType::Learning => 2560,
            ResearchType::Validation => 1536,
        };

        let mut tokens = base_tokens;

        if let Some(context) = context_result {
            // Adjust based on audience level
            match context.audience_level {
                AudienceLevel::Beginner => tokens = (tokens as f32 * 1.3) as u32,
                AudienceLevel::Intermediate => tokens = (tokens as f32 * 1.1) as u32,
                AudienceLevel::Advanced => tokens = (tokens as f32 * 0.9) as u32,
            }

            // Adjust based on urgency
            match context.urgency_level {
                UrgencyLevel::Immediate => tokens = (tokens as f32 * 0.8) as u32,
                UrgencyLevel::Planned => tokens = (tokens as f32 * 1.2) as u32,
                UrgencyLevel::Exploratory => tokens = (tokens as f32 * 1.4) as u32,
            }
        }

        tokens.clamp(512, 4096)
    }

    /// Determine optimal temperature based on context
    fn determine_temperature(
        &self,
        research_type: &ResearchType,
        context_result: Option<&ContextDetectionResult>,
    ) -> f32 {
        let base_temperature = match research_type {
            ResearchType::Decision => 0.3,
            ResearchType::Implementation => 0.2,
            ResearchType::Troubleshooting => 0.1,
            ResearchType::Learning => 0.4,
            ResearchType::Validation => 0.2,
        };

        let mut temperature: f32 = base_temperature;

        if let Some(context) = context_result {
            // Adjust based on audience level
            match context.audience_level {
                AudienceLevel::Beginner => temperature += 0.1,
                AudienceLevel::Intermediate => temperature += 0.05,
                AudienceLevel::Advanced => temperature -= 0.05,
            }

            // Adjust based on urgency
            match context.urgency_level {
                UrgencyLevel::Immediate => temperature -= 0.1,
                UrgencyLevel::Planned => temperature += 0.0,
                UrgencyLevel::Exploratory => temperature += 0.1,
            }
        }

        temperature.clamp(0.0_f32, 1.0_f32)
    }

    /// Send context-aware request to Claude API
    pub async fn send_context_aware_request(
        &self,
        context_request: ContextAwareClaudeRequest,
    ) -> ApiResult<ClaudeResponse> {
        // Check context-aware cache first
        let cache_key = self.generate_context_aware_cache_key(&context_request);
        if let Some(cache_entry) = self.request_cache.get(&cache_key) {
            let (response, timestamp) = cache_entry.value();
            if timestamp.elapsed() < Duration::from_secs(300) {
                debug!("Context-aware cache hit for request");
                return Ok(response.clone());
            }
        }

        // Send the request
        let response = self.send_request(context_request.base_request).await?;

        // Cache the response with context-aware key
        self.request_cache
            .insert(cache_key, (response.clone(), Instant::now()));

        Ok(response)
    }

    pub fn cleanup_cache(&self) {
        let now = Instant::now();
        self.request_cache
            .retain(|_, (_, timestamp)| now.duration_since(*timestamp) < Duration::from_secs(300));
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        let total = self.request_cache.len();
        let expired = self
            .request_cache
            .iter()
            .filter(|entry| entry.value().1.elapsed() > Duration::from_secs(300))
            .count();
        (total, expired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_claude_config_validation() {
        let config = ClaudeConfig::new("sk-test-key".to_string());
        assert!(config.validate().is_ok());

        let invalid_config = ClaudeConfig::new("invalid-key".to_string());
        assert!(invalid_config.validate().is_err());
    }

    #[tokio::test]
    async fn test_claude_request_validation() {
        let config = ClaudeConfig::new("sk-test-key".to_string());
        let client = ClaudeClient::new(config).unwrap();

        let valid_request = ClaudeRequest {
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
            stream: Some(false),
        };

        assert!(client.validate_request(&valid_request).is_ok());

        let invalid_request = ClaudeRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            max_tokens: 0,
            messages: vec![],
            system: None,
            temperature: None,
            top_p: None,
            stop_sequences: None,
            stream: Some(false),
        };

        assert!(client.validate_request(&invalid_request).is_err());
    }

    #[tokio::test]
    async fn test_token_estimation() {
        let config = ClaudeConfig::new("sk-test-key".to_string());
        let client = ClaudeClient::new(config).unwrap();

        let request = ClaudeRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            max_tokens: 1024,
            messages: vec![Message {
                role: "user".to_string(),
                content: "This is a test message with some content".to_string(),
            }],
            system: None,
            temperature: None,
            top_p: None,
            stop_sequences: None,
            stream: Some(false),
        };

        let estimated = client.estimate_input_tokens(&request);
        assert!(estimated > 0);
        assert!(estimated < 100); // Should be reasonable for this small message
    }

    #[tokio::test]
    async fn test_context_aware_request_creation() {
        let config = ClaudeConfig::new("sk-test-key".to_string());
        let client = ClaudeClient::new(config).unwrap();

        let context_request = client.create_context_aware_request(
            "How do I implement async functions in Rust?",
            &ResearchType::Implementation,
            None,
        );

        assert_eq!(context_request.research_type, ResearchType::Implementation);
        assert_eq!(
            context_request.original_query,
            "How do I implement async functions in Rust?"
        );
        assert!(!context_request.base_request.messages.is_empty());
        assert!(context_request.base_request.system.is_some());
        assert!(context_request.base_request.temperature.is_some());
        assert!(context_request.base_request.max_tokens > 0);
    }

    #[test]
    fn test_context_aware_cache_key_generation() {
        let config = ClaudeConfig::new("sk-test-key".to_string());
        let client = ClaudeClient::new(config).unwrap();

        let context_request =
            client.create_context_aware_request("Test query", &ResearchType::Learning, None);

        let key1 = client.generate_context_aware_cache_key(&context_request);
        let key2 = client.generate_context_aware_cache_key(&context_request);

        assert_eq!(key1, key2);
        assert!(key1.starts_with("claude_context_request_"));
        assert!(!key1.is_empty());
    }

    #[test]
    fn test_max_tokens_determination() {
        let config = ClaudeConfig::new("sk-test-key".to_string());
        let client = ClaudeClient::new(config).unwrap();

        // Test different research types
        let decision_tokens = client.determine_max_tokens(&ResearchType::Decision, None);
        let implementation_tokens =
            client.determine_max_tokens(&ResearchType::Implementation, None);
        let learning_tokens = client.determine_max_tokens(&ResearchType::Learning, None);

        assert!(decision_tokens > 0);
        assert!(implementation_tokens > decision_tokens); // Implementation should get more tokens
        assert!(learning_tokens > 0);

        // Test bounds
        assert!(decision_tokens >= 512);
        assert!(decision_tokens <= 4096);
    }

    #[test]
    fn test_temperature_determination() {
        let config = ClaudeConfig::new("sk-test-key".to_string());
        let client = ClaudeClient::new(config).unwrap();

        // Test different research types
        let decision_temp = client.determine_temperature(&ResearchType::Decision, None);
        let troubleshooting_temp =
            client.determine_temperature(&ResearchType::Troubleshooting, None);
        let learning_temp = client.determine_temperature(&ResearchType::Learning, None);

        assert!(decision_temp >= 0.0);
        assert!(decision_temp <= 1.0);
        assert!(troubleshooting_temp < decision_temp); // Troubleshooting should be more deterministic
        assert!(learning_temp > decision_temp); // Learning should be more creative
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = RateLimitConfig {
            requests_per_minute: 10,
            input_tokens_per_minute: 1000,
            output_tokens_per_minute: 500,
            max_concurrent_requests: 2,
        };

        let rate_limiter = RateLimiter::new(config);

        // Should succeed initially
        assert!(rate_limiter.acquire_permit(100).await.is_ok());

        // Should still succeed with reasonable token usage
        assert!(rate_limiter.acquire_permit(100).await.is_ok());
    }
}
