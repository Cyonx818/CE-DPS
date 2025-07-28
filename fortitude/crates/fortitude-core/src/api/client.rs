// Copyright 2025 CE-DPS Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// ABOUTME: Generic API client traits and configurations
use crate::api::error::ApiResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Generic API client trait for all external service integrations
#[async_trait]
pub trait ApiClient: Send + Sync {
    type Request: Send + Sync;
    type Response: Send + Sync;
    type Config: Clone + Send + Sync;

    /// Create a new API client with the given configuration
    fn new(config: Self::Config) -> ApiResult<Self>
    where
        Self: Sized;

    /// Send a request to the API
    async fn send_request(&self, request: Self::Request) -> ApiResult<Self::Response>;

    /// Validate a request before sending
    fn validate_request(&self, request: &Self::Request) -> ApiResult<()>;

    /// Estimate the cost or resource usage of a request
    fn estimate_cost(&self, request: &Self::Request) -> ApiResult<RequestCost>;

    /// Get client health status
    async fn health_check(&self) -> ApiResult<HealthStatus>;
}

/// Rate limiting configuration
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
            requests_per_minute: 50,
            input_tokens_per_minute: 40_000,
            output_tokens_per_minute: 8_000,
            max_concurrent_requests: 5,
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
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

/// Request cost estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestCost {
    pub estimated_input_tokens: u32,
    pub estimated_output_tokens: u32,
    pub estimated_duration: Duration,
    pub estimated_cost_usd: Option<f64>,
}

/// Health status of the API client
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

/// Generic API configuration trait
pub trait ApiConfig {
    fn validate(&self) -> ApiResult<()>;
    fn with_timeout(self, timeout: Duration) -> Self;
    fn with_rate_limit(self, rate_limit: RateLimitConfig) -> Self;
    fn with_retry(self, retry: RetryConfig) -> Self;
}
