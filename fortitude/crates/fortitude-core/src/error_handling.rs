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

//! Enhanced error handling framework for Core Research Pipeline Stabilization
//!
//! This module provides structured error types, retry logic with exponential backoff,
//! and circuit breaker patterns for reliable research pipeline operations.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Comprehensive error types for research pipeline operations
#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Network error: {message}")]
    Network {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        retry_after: Option<Duration>,
    },

    #[error("Timeout error: operation took longer than {timeout_ms}ms")]
    Timeout {
        timeout_ms: u64,
        operation: String,
        correlation_id: String,
    },

    #[error("External API error: {api_name} returned {status_code}")]
    ExternalApi {
        api_name: String,
        status_code: u16,
        message: String,
        retry_after: Option<Duration>,
        correlation_id: String,
    },

    #[error("Validation error: {field}")]
    Validation {
        field: String,
        message: String,
        context: HashMap<String, String>,
    },

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Circuit breaker open for {service}")]
    CircuitBreakerOpen {
        service: String,
        retry_after: Duration,
        failure_count: u32,
    },

    #[error("Rate limit exceeded for {service}")]
    RateLimit {
        service: String,
        retry_after: Duration,
        limit: u32,
    },

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Authorization failed: {0}")]
    Authorization(String),

    #[error("Internal error: {message}")]
    Internal {
        message: String,
        correlation_id: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl Clone for PipelineError {
    fn clone(&self) -> Self {
        match self {
            Self::Network {
                message,
                retry_after,
                ..
            } => Self::Network {
                message: message.clone(),
                source: None, // Can't clone trait objects
                retry_after: *retry_after,
            },
            Self::Timeout {
                timeout_ms,
                operation,
                correlation_id,
            } => Self::Timeout {
                timeout_ms: *timeout_ms,
                operation: operation.clone(),
                correlation_id: correlation_id.clone(),
            },
            Self::ExternalApi {
                api_name,
                status_code,
                message,
                retry_after,
                correlation_id,
            } => Self::ExternalApi {
                api_name: api_name.clone(),
                status_code: *status_code,
                message: message.clone(),
                retry_after: *retry_after,
                correlation_id: correlation_id.clone(),
            },
            Self::Validation {
                field,
                message,
                context,
            } => Self::Validation {
                field: field.clone(),
                message: message.clone(),
                context: context.clone(),
            },
            Self::Configuration(msg) => Self::Configuration(msg.clone()),
            Self::CircuitBreakerOpen {
                service,
                retry_after,
                failure_count,
            } => Self::CircuitBreakerOpen {
                service: service.clone(),
                retry_after: *retry_after,
                failure_count: *failure_count,
            },
            Self::RateLimit {
                service,
                retry_after,
                limit,
            } => Self::RateLimit {
                service: service.clone(),
                retry_after: *retry_after,
                limit: *limit,
            },
            Self::Authentication(msg) => Self::Authentication(msg.clone()),
            Self::Authorization(msg) => Self::Authorization(msg.clone()),
            Self::Internal {
                message,
                correlation_id,
                ..
            } => Self::Internal {
                message: message.clone(),
                correlation_id: correlation_id.clone(),
                source: None, // Can't clone trait objects
            },
        }
    }
}

impl PipelineError {
    /// Get correlation ID for error tracking
    pub fn correlation_id(&self) -> Option<&str> {
        match self {
            PipelineError::Timeout { correlation_id, .. } => Some(correlation_id),
            PipelineError::ExternalApi { correlation_id, .. } => Some(correlation_id),
            PipelineError::Internal { correlation_id, .. } => Some(correlation_id),
            _ => None,
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            PipelineError::Network { .. } => true,
            PipelineError::Timeout { .. } => true,
            PipelineError::ExternalApi { status_code, .. } => *status_code >= 500,
            PipelineError::RateLimit { .. } => true,
            PipelineError::CircuitBreakerOpen { .. } => false,
            PipelineError::Configuration(_) => false,
            PipelineError::Validation { .. } => false,
            PipelineError::Authentication(_) => false,
            PipelineError::Authorization(_) => false,
            PipelineError::Internal { .. } => false,
        }
    }

    /// Get suggested retry delay
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            PipelineError::Network { retry_after, .. } => *retry_after,
            PipelineError::ExternalApi { retry_after, .. } => *retry_after,
            PipelineError::CircuitBreakerOpen { retry_after, .. } => Some(*retry_after),
            PipelineError::RateLimit { retry_after, .. } => Some(*retry_after),
            _ => None,
        }
    }
}

/// Retry configuration with exponential backoff
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open { until: Instant },
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout: Duration,
    pub reset_timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            reset_timeout: Duration::from_secs(30),
        }
    }
}

/// Circuit breaker implementation
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<Mutex<CircuitBreakerState>>,
    failure_count: Arc<Mutex<u32>>,
    success_count: Arc<Mutex<u32>>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(CircuitBreakerState::Closed)),
            failure_count: Arc::new(Mutex::new(0)),
            success_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Execute operation with circuit breaker protection
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T, PipelineError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, PipelineError>>,
    {
        // Check circuit breaker state
        {
            let state = self.state.lock().unwrap();
            if let CircuitBreakerState::Open { until } = *state {
                if Instant::now() < until {
                    let failure_count = *self.failure_count.lock().unwrap();
                    return Err(PipelineError::CircuitBreakerOpen {
                        service: "research_pipeline".to_string(),
                        retry_after: until.duration_since(Instant::now()),
                        failure_count,
                    });
                }
                // Move to half-open state
                drop(state);
                *self.state.lock().unwrap() = CircuitBreakerState::HalfOpen;
            }
        }

        // Execute operation with timeout
        let result = timeout(self.config.timeout, operation()).await;

        match result {
            Ok(Ok(value)) => {
                self.on_success();
                Ok(value)
            }
            Ok(Err(error)) => {
                self.on_failure();
                Err(error)
            }
            Err(_) => {
                self.on_failure();
                Err(PipelineError::Timeout {
                    timeout_ms: self.config.timeout.as_millis() as u64,
                    operation: "circuit_breaker_protected".to_string(),
                    correlation_id: Uuid::new_v4().to_string(),
                })
            }
        }
    }

    fn on_success(&self) {
        let mut success_count = self.success_count.lock().unwrap();
        let mut failure_count = self.failure_count.lock().unwrap();

        *success_count += 1;
        *failure_count = 0;

        let state = self.state.lock().unwrap();
        if matches!(*state, CircuitBreakerState::HalfOpen)
            && *success_count >= self.config.success_threshold
        {
            drop(state);
            *self.state.lock().unwrap() = CircuitBreakerState::Closed;
            *success_count = 0;
            info!("Circuit breaker closed after successful operations");
        }
    }

    fn on_failure(&self) {
        let mut failure_count = self.failure_count.lock().unwrap();
        let mut success_count = self.success_count.lock().unwrap();

        *failure_count += 1;
        *success_count = 0;

        if *failure_count >= self.config.failure_threshold {
            let until = Instant::now() + self.config.reset_timeout;
            *self.state.lock().unwrap() = CircuitBreakerState::Open { until };
            warn!("Circuit breaker opened after {} failures", *failure_count);
        }
    }

    /// Get current circuit breaker state
    pub fn state(&self) -> CircuitBreakerState {
        self.state.lock().unwrap().clone()
    }

    /// Get current failure count
    pub fn failure_count(&self) -> u32 {
        *self.failure_count.lock().unwrap()
    }
}

/// Retry executor with exponential backoff
pub struct RetryExecutor {
    config: RetryConfig,
}

impl RetryExecutor {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute operation with retry logic
    pub async fn execute<F, Fut, T>(&self, mut operation: F) -> Result<T, PipelineError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, PipelineError>>,
    {
        let mut attempt = 1;
        let mut delay = self.config.initial_delay;

        loop {
            let correlation_id = Uuid::new_v4().to_string();
            debug!(
                "Executing operation attempt {} with correlation_id: {}",
                attempt, correlation_id
            );

            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        info!(
                            "Operation succeeded on attempt {} with correlation_id: {}",
                            attempt, correlation_id
                        );
                    }
                    return Ok(result);
                }
                Err(error) => {
                    if attempt >= self.config.max_attempts || !error.is_retryable() {
                        error!(
                            "Operation failed after {} attempts with correlation_id: {}, error: {}",
                            attempt, correlation_id, error
                        );
                        return Err(error);
                    }

                    // Use suggested retry delay if available
                    let retry_delay = error.retry_after().unwrap_or(delay);

                    warn!("Operation failed on attempt {} with correlation_id: {}, retrying in {:?}ms, error: {}", 
                          attempt, correlation_id, retry_delay.as_millis(), error);

                    sleep(retry_delay).await;

                    // Calculate next delay with exponential backoff and jitter
                    delay = std::cmp::min(
                        Duration::from_millis(
                            (delay.as_millis() as f64 * self.config.backoff_multiplier) as u64,
                        ),
                        self.config.max_delay,
                    );

                    // Add jitter to prevent thundering herd
                    if self.config.jitter_factor > 0.0 {
                        let jitter = (delay.as_millis() as f64
                            * self.config.jitter_factor
                            * rand::random::<f64>()) as u64;
                        delay = Duration::from_millis(delay.as_millis() as u64 + jitter);
                    }

                    attempt += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::time::{sleep, Duration};

    // ANCHOR: Core error handling regression test
    #[tokio::test]
    async fn test_pipeline_error_correlation_id() {
        let correlation_id = "test-12345";
        let error = PipelineError::Timeout {
            timeout_ms: 5000,
            operation: "test_operation".to_string(),
            correlation_id: correlation_id.to_string(),
        };

        assert_eq!(error.correlation_id(), Some(correlation_id));
        assert!(error.is_retryable()); // Timeouts are retryable
    }

    #[tokio::test]
    async fn test_retry_logic_with_exponential_backoff() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            jitter_factor: 0.0, // No jitter for predictable testing
        };

        let executor = RetryExecutor::new(config);
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        let start = Instant::now();
        let result = executor
            .execute(move || {
                let count = attempt_count_clone.fetch_add(1, Ordering::SeqCst) + 1;
                async move {
                    if count < 3 {
                        Err(PipelineError::Network {
                            message: format!("Attempt {}", count),
                            source: None,
                            retry_after: None,
                        })
                    } else {
                        Ok("Success".to_string())
                    }
                }
            })
            .await;

        let duration = start.elapsed();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
        // Should have delays: 10ms + 20ms = 30ms minimum
        assert!(duration >= Duration::from_millis(30));
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout: Duration::from_millis(100),
            reset_timeout: Duration::from_millis(50),
        };

        let circuit_breaker = CircuitBreaker::new(config);

        // First failure
        let result1 = circuit_breaker
            .execute(|| async {
                Err::<(), _>(PipelineError::Network {
                    message: "Network error".to_string(),
                    source: None,
                    retry_after: None,
                })
            })
            .await;
        assert!(result1.is_err());
        assert_eq!(circuit_breaker.state(), CircuitBreakerState::Closed);

        // Second failure - should open circuit
        let result2 = circuit_breaker
            .execute(|| async {
                Err::<(), _>(PipelineError::Network {
                    message: "Network error".to_string(),
                    source: None,
                    retry_after: None,
                })
            })
            .await;
        assert!(result2.is_err());
        assert!(matches!(
            circuit_breaker.state(),
            CircuitBreakerState::Open { .. }
        ));

        // Third attempt should be rejected
        let result3 = circuit_breaker.execute(|| async { Ok::<(), _>(()) }).await;
        assert!(result3.is_err());
        assert!(matches!(
            result3.unwrap_err(),
            PipelineError::CircuitBreakerOpen { .. }
        ));
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,
            timeout: Duration::from_millis(100),
            reset_timeout: Duration::from_millis(10), // Short reset for testing
        };

        let circuit_breaker = CircuitBreaker::new(config);

        // Trigger failure to open circuit
        let _result = circuit_breaker
            .execute(|| async {
                Err::<(), _>(PipelineError::Network {
                    message: "Network error".to_string(),
                    source: None,
                    retry_after: None,
                })
            })
            .await;

        assert!(matches!(
            circuit_breaker.state(),
            CircuitBreakerState::Open { .. }
        ));

        // Wait for reset timeout
        sleep(Duration::from_millis(15)).await;

        // Next call should succeed and close circuit
        let result = circuit_breaker
            .execute(|| async { Ok::<&str, _>("Success") })
            .await;

        assert!(result.is_ok());
        assert_eq!(circuit_breaker.state(), CircuitBreakerState::Closed);
    }

    #[test]
    fn test_error_retryability() {
        let network_error = PipelineError::Network {
            message: "Connection failed".to_string(),
            source: None,
            retry_after: Some(Duration::from_secs(1)),
        };
        assert!(network_error.is_retryable());

        let validation_error = PipelineError::Validation {
            field: "email".to_string(),
            message: "Invalid format".to_string(),
            context: HashMap::new(),
        };
        assert!(!validation_error.is_retryable());

        let server_error = PipelineError::ExternalApi {
            api_name: "claude".to_string(),
            status_code: 500,
            message: "Internal server error".to_string(),
            retry_after: None,
            correlation_id: "test-123".to_string(),
        };
        assert!(server_error.is_retryable());

        let client_error = PipelineError::ExternalApi {
            api_name: "claude".to_string(),
            status_code: 400,
            message: "Bad request".to_string(),
            retry_after: None,
            correlation_id: "test-456".to_string(),
        };
        assert!(!client_error.is_retryable());
    }

    #[tokio::test]
    async fn test_timeout_error_with_correlation_id() {
        let config = CircuitBreakerConfig {
            timeout: Duration::from_millis(50), // Very short timeout
            ..Default::default()
        };
        let circuit_breaker = CircuitBreaker::new(config);

        let result = circuit_breaker
            .execute(|| async {
                // Simulate long operation that will definitely timeout
                sleep(Duration::from_secs(1)).await;
                Ok::<&str, PipelineError>("Should timeout")
            })
            .await;

        assert!(result.is_err());
        if let Err(PipelineError::Timeout { correlation_id, .. }) = result {
            assert!(!correlation_id.is_empty());
        } else {
            panic!("Expected timeout error");
        }
    }
}
