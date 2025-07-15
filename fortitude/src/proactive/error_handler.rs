// ABOUTME: Comprehensive error handling and retry mechanisms for proactive research components
//! This module provides centralized error handling, retry mechanisms, and circuit breaker patterns
//! for the proactive research system. Features include:
//! - Error classification and categorization for appropriate handling strategies
//! - Exponential backoff with jitter for transient failures
//! - Circuit breaker patterns for external service failures
//! - Automatic recovery strategies for different error types
//! - Comprehensive error monitoring and alerting
//! - Dead letter queue for permanently failed operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{sleep, timeout};
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

/// Comprehensive error types for classification and handling
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ProactiveError {
    #[error("Transient error (retryable): {message} - attempt {attempt}/{max_attempts}")]
    Transient {
        message: String,
        attempt: u32,
        max_attempts: u32,
        retry_after: Option<Duration>,
    },

    #[error("Permanent error (not retryable): {message}")]
    Permanent { message: String, error_code: String },

    #[error("Rate limit exceeded: {operation} - retry after {retry_after:?}")]
    RateLimit {
        operation: String,
        retry_after: Duration,
        current_requests: u32,
        limit: u32,
    },

    #[error("Circuit breaker open for {service} - last failure: {last_failure}")]
    CircuitBreakerOpen {
        service: String,
        last_failure: DateTime<Utc>,
        failure_count: u32,
    },

    #[error("Resource exhaustion: {resource} at {usage:.1}% (limit: {limit:.1}%)")]
    ResourceExhaustion {
        resource: String,
        usage: f64,
        limit: f64,
        suggested_backoff: Duration,
    },

    #[error("Timeout error: {operation} exceeded {timeout:?}")]
    Timeout {
        operation: String,
        timeout: Duration,
        elapsed: Duration,
    },

    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: Option<String>,
        constraint: Option<String>,
    },

    #[error("Network error: {message} - network status: {network_status}")]
    Network {
        message: String,
        network_status: NetworkStatus,
        retry_recommended: bool,
    },

    #[error("External service error: {service} - {message}")]
    ExternalService {
        service: String,
        message: String,
        status_code: Option<u16>,
        service_status: ServiceStatus,
    },

    #[error("Internal system error: {message}")]
    Internal {
        message: String,
        component: String,
        recoverable: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkStatus {
    Connected,
    Disconnected,
    Limited,
    Unknown,
}

impl std::fmt::Display for NetworkStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkStatus::Connected => write!(f, "connected"),
            NetworkStatus::Disconnected => write!(f, "disconnected"),
            NetworkStatus::Limited => write!(f, "limited"),
            NetworkStatus::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    Available,
    Degraded,
    Unavailable,
    Maintenance,
    Unknown,
}

impl std::fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceStatus::Available => write!(f, "available"),
            ServiceStatus::Degraded => write!(f, "degraded"),
            ServiceStatus::Unavailable => write!(f, "unavailable"),
            ServiceStatus::Maintenance => write!(f, "maintenance"),
            ServiceStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Error classification for handling strategy determination
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorClassification {
    Transient,
    Permanent,
    RateLimit,
    CircuitBreaker,
    ResourceExhaustion,
    Timeout,
    Validation,
    Network,
    ExternalService,
    Internal,
}

impl ProactiveError {
    /// Classify error for handling strategy
    pub fn classify(&self) -> ErrorClassification {
        match self {
            ProactiveError::Transient { .. } => ErrorClassification::Transient,
            ProactiveError::Permanent { .. } => ErrorClassification::Permanent,
            ProactiveError::RateLimit { .. } => ErrorClassification::RateLimit,
            ProactiveError::CircuitBreakerOpen { .. } => ErrorClassification::CircuitBreaker,
            ProactiveError::ResourceExhaustion { .. } => ErrorClassification::ResourceExhaustion,
            ProactiveError::Timeout { .. } => ErrorClassification::Timeout,
            ProactiveError::Validation { .. } => ErrorClassification::Validation,
            ProactiveError::Network { .. } => ErrorClassification::Network,
            ProactiveError::ExternalService { .. } => ErrorClassification::ExternalService,
            ProactiveError::Internal { .. } => ErrorClassification::Internal,
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            ProactiveError::Transient { .. } => true,
            ProactiveError::RateLimit { .. } => true,
            ProactiveError::ResourceExhaustion { .. } => true,
            ProactiveError::Timeout { .. } => true,
            ProactiveError::Network {
                retry_recommended, ..
            } => *retry_recommended,
            ProactiveError::ExternalService { service_status, .. } => {
                matches!(
                    service_status,
                    ServiceStatus::Degraded | ServiceStatus::Unknown
                )
            }
            ProactiveError::Internal { recoverable, .. } => *recoverable,
            _ => false,
        }
    }

    /// Get suggested retry delay
    pub fn retry_delay(&self) -> Option<Duration> {
        match self {
            ProactiveError::Transient { retry_after, .. } => *retry_after,
            ProactiveError::RateLimit { retry_after, .. } => Some(*retry_after),
            ProactiveError::ResourceExhaustion {
                suggested_backoff, ..
            } => Some(*suggested_backoff),
            ProactiveError::Timeout { .. } => Some(Duration::from_secs(5)),
            ProactiveError::Network { .. } => Some(Duration::from_secs(10)),
            ProactiveError::ExternalService { .. } => Some(Duration::from_secs(30)),
            _ => None,
        }
    }
}

/// Retry strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryStrategy {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub jitter_factor: f64,
    pub enable_jitter: bool,
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            jitter_factor: 0.1,
            enable_jitter: true,
        }
    }
}

impl RetryStrategy {
    /// Calculate delay for given attempt with exponential backoff and jitter
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return self.initial_delay;
        }

        let exponential_delay = self
            .initial_delay
            .mul_f64(self.multiplier.powi(attempt as i32 - 1));
        let bounded_delay = exponential_delay.min(self.max_delay);

        if !self.enable_jitter {
            return bounded_delay;
        }

        // Add jitter to prevent thundering herd
        let jitter_range = bounded_delay.mul_f64(self.jitter_factor);
        let jitter =
            Duration::from_nanos((rand::random::<f64>() * jitter_range.as_nanos() as f64) as u64);

        bounded_delay + jitter
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing if service recovered
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout: Duration,
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(60),
            half_open_max_calls: 5,
        }
    }
}

/// Circuit breaker for external service calls
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<RwLock<u32>>,
    success_count: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    half_open_calls: Arc<RwLock<u32>>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            success_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            half_open_calls: Arc::new(RwLock::new(0)),
        }
    }

    /// Check if call should be allowed
    pub async fn can_execute(&self) -> bool {
        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed to move to half-open
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    if last_failure.elapsed() >= self.config.timeout {
                        let mut state_guard = self.state.write().await;
                        *state_guard = CircuitState::HalfOpen;
                        let mut half_open_calls = self.half_open_calls.write().await;
                        *half_open_calls = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                let mut half_open_calls = self.half_open_calls.write().await;
                if *half_open_calls < self.config.half_open_max_calls {
                    *half_open_calls += 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Record successful operation
    pub async fn record_success(&self) {
        let state = *self.state.read().await;

        if state == CircuitState::HalfOpen {
            let mut success_count = self.success_count.write().await;
            *success_count += 1;

            if *success_count >= self.config.success_threshold {
                let mut state_guard = self.state.write().await;
                *state_guard = CircuitState::Closed;
                let mut failure_count = self.failure_count.write().await;
                *failure_count = 0;
                *success_count = 0;
            }
        }
    }

    /// Record failed operation
    pub async fn record_failure(&self) {
        let mut failure_count = self.failure_count.write().await;
        *failure_count += 1;

        let mut last_failure_time = self.last_failure_time.write().await;
        *last_failure_time = Some(Instant::now());

        if *failure_count >= self.config.failure_threshold {
            let mut state_guard = self.state.write().await;
            *state_guard = CircuitState::Open;
        }
    }

    /// Get current circuit state
    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }
}

/// Error recovery attempt record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAttempt {
    pub id: String,
    pub error: ProactiveError,
    pub attempt_number: u32,
    pub timestamp: DateTime<Utc>,
    pub strategy_used: RecoveryStrategy,
    pub success: bool,
    pub recovery_duration: Option<Duration>,
}

/// Recovery strategies for different error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    Retry {
        delay: Duration,
        attempt: u32,
        max_attempts: u32,
    },
    CircuitBreakerWait {
        service: String,
        wait_duration: Duration,
    },
    ResourceThrottling {
        backoff_duration: Duration,
        resource: String,
    },
    ServiceDegradation {
        alternative_strategy: String,
    },
    ManualIntervention {
        escalation_reason: String,
    },
}

/// Dead letter queue for permanently failed operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterEntry {
    pub id: String,
    pub original_operation: String,
    pub final_error: ProactiveError,
    pub total_attempts: u32,
    pub first_failure: DateTime<Utc>,
    pub final_failure: DateTime<Utc>,
    pub recovery_attempts: Vec<RecoveryAttempt>,
    pub escalated: bool,
}

/// Error monitoring and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub total_errors: u64,
    pub errors_by_classification: HashMap<ErrorClassification, u64>,
    pub recovery_success_rate: f64,
    pub average_recovery_time: Duration,
    pub circuit_breaker_trips: u64,
    pub dead_letter_entries: u64,
    pub last_updated: DateTime<Utc>,
}

impl Default for ErrorMetrics {
    fn default() -> Self {
        Self {
            total_errors: 0,
            errors_by_classification: HashMap::new(),
            recovery_success_rate: 0.0,
            average_recovery_time: Duration::from_secs(0),
            circuit_breaker_trips: 0,
            dead_letter_entries: 0,
            last_updated: Utc::now(),
        }
    }
}

/// Configuration for error handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlerConfig {
    pub retry_strategies: HashMap<ErrorClassification, RetryStrategy>,
    pub circuit_breaker_configs: HashMap<String, CircuitBreakerConfig>,
    pub dead_letter_queue_size: usize,
    pub error_monitoring_enabled: bool,
    pub escalation_thresholds: EscalationThresholds,
    pub recovery_timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationThresholds {
    pub max_consecutive_failures: u32,
    pub max_error_rate: f64,
    pub max_dead_letter_entries: usize,
}

impl Default for ErrorHandlerConfig {
    fn default() -> Self {
        let mut retry_strategies = HashMap::new();
        retry_strategies.insert(ErrorClassification::Transient, RetryStrategy::default());
        retry_strategies.insert(
            ErrorClassification::RateLimit,
            RetryStrategy {
                max_attempts: 5,
                initial_delay: Duration::from_secs(1),
                max_delay: Duration::from_secs(300),
                multiplier: 2.0,
                jitter_factor: 0.2,
                enable_jitter: true,
            },
        );
        retry_strategies.insert(
            ErrorClassification::ResourceExhaustion,
            RetryStrategy {
                max_attempts: 3,
                initial_delay: Duration::from_secs(5),
                max_delay: Duration::from_secs(120),
                multiplier: 1.5,
                jitter_factor: 0.1,
                enable_jitter: true,
            },
        );

        let mut circuit_breaker_configs = HashMap::new();
        circuit_breaker_configs.insert("default".to_string(), CircuitBreakerConfig::default());

        Self {
            retry_strategies,
            circuit_breaker_configs,
            dead_letter_queue_size: 1000,
            error_monitoring_enabled: true,
            escalation_thresholds: EscalationThresholds {
                max_consecutive_failures: 10,
                max_error_rate: 0.5,
                max_dead_letter_entries: 100,
            },
            recovery_timeout: Duration::from_secs(300),
        }
    }
}

/// Centralized error handler with comprehensive recovery mechanisms
#[derive(Debug)]
pub struct ErrorHandler {
    config: ErrorHandlerConfig,
    circuit_breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
    dead_letter_queue: Arc<Mutex<VecDeque<DeadLetterEntry>>>,
    #[allow(dead_code)] // TODO: Will be used for tracking and optimizing recovery strategies
    recovery_attempts: Arc<RwLock<HashMap<String, Vec<RecoveryAttempt>>>>,
    metrics: Arc<RwLock<ErrorMetrics>>,
    running: Arc<RwLock<bool>>,
}

impl ErrorHandler {
    /// Create a new error handler with the given configuration
    pub fn new(config: ErrorHandlerConfig) -> Self {
        Self {
            config,
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            dead_letter_queue: Arc::new(Mutex::new(VecDeque::new())),
            recovery_attempts: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(ErrorMetrics::default())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the error handler
    pub async fn start(&self) -> Result<(), ProactiveError> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        info!(
            "Starting error handler with monitoring enabled: {}",
            self.config.error_monitoring_enabled
        );

        if self.config.error_monitoring_enabled {
            self.start_monitoring().await;
        }

        Ok(())
    }

    /// Stop the error handler
    pub async fn stop(&self) -> Result<(), ProactiveError> {
        let mut running = self.running.write().await;
        *running = false;
        info!("Error handler stopped");
        Ok(())
    }

    /// Handle error with comprehensive recovery strategy
    #[instrument(level = "debug", skip(self, error, operation))]
    pub async fn handle_error<F, Fut, T>(
        &self,
        error: ProactiveError,
        operation: F,
        operation_name: &str,
        service_name: Option<&str>,
    ) -> Result<T, ProactiveError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, ProactiveError>>,
    {
        let classification = error.classify();
        let service = service_name.unwrap_or("default");

        // Update metrics
        self.update_error_metrics(&classification).await;

        // Check circuit breaker if applicable
        if let Some(circuit_breaker) = self.get_circuit_breaker(service).await {
            if !circuit_breaker.can_execute().await {
                circuit_breaker.record_failure().await;
                return Err(ProactiveError::CircuitBreakerOpen {
                    service: service.to_string(),
                    last_failure: Utc::now(),
                    failure_count: 1,
                });
            }
        }

        // Determine recovery strategy
        match self
            .determine_recovery_strategy(&error, operation_name)
            .await
        {
            Some(strategy) => {
                self.execute_recovery_strategy(strategy, operation, operation_name, service)
                    .await
            }
            None => {
                // No recovery possible, add to dead letter queue
                self.add_to_dead_letter_queue(operation_name, error.clone())
                    .await;
                Err(error)
            }
        }
    }

    /// Execute operation with retry logic
    pub async fn execute_with_retry<F, Fut, T>(
        &self,
        mut operation: F,
        operation_name: &str,
        service_name: Option<&str>,
    ) -> Result<T, ProactiveError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, ProactiveError>>,
    {
        let service = service_name.unwrap_or("default");
        let retry_strategy = self
            .config
            .retry_strategies
            .get(&ErrorClassification::Transient)
            .cloned()
            .unwrap_or_default();

        for attempt in 0..retry_strategy.max_attempts {
            // Check circuit breaker
            if let Some(circuit_breaker) = self.get_circuit_breaker(service).await {
                if !circuit_breaker.can_execute().await {
                    return Err(ProactiveError::CircuitBreakerOpen {
                        service: service.to_string(),
                        last_failure: Utc::now(),
                        failure_count: attempt + 1,
                    });
                }
            }

            match timeout(self.config.recovery_timeout, operation()).await {
                Ok(Ok(result)) => {
                    // Success - record for circuit breaker
                    if let Some(circuit_breaker) = self.get_circuit_breaker(service).await {
                        circuit_breaker.record_success().await;
                    }
                    return Ok(result);
                }
                Ok(Err(error)) => {
                    // Operation failed
                    if let Some(circuit_breaker) = self.get_circuit_breaker(service).await {
                        circuit_breaker.record_failure().await;
                    }

                    // Check if retryable
                    if !error.is_retryable() || attempt == retry_strategy.max_attempts - 1 {
                        self.add_to_dead_letter_queue(operation_name, error.clone())
                            .await;
                        return Err(error);
                    }

                    // Calculate delay and wait
                    let delay = retry_strategy.calculate_delay(attempt + 1);
                    warn!(
                        "Operation {} failed (attempt {}), retrying in {:?}: {}",
                        operation_name,
                        attempt + 1,
                        delay,
                        error
                    );

                    sleep(delay).await;
                }
                Err(_) => {
                    // Timeout
                    let timeout_error = ProactiveError::Timeout {
                        operation: operation_name.to_string(),
                        timeout: self.config.recovery_timeout,
                        elapsed: self.config.recovery_timeout,
                    };

                    if attempt == retry_strategy.max_attempts - 1 {
                        self.add_to_dead_letter_queue(operation_name, timeout_error.clone())
                            .await;
                        return Err(timeout_error);
                    }

                    let delay = retry_strategy.calculate_delay(attempt + 1);
                    sleep(delay).await;
                }
            }
        }

        // All retries exhausted
        let exhausted_error = ProactiveError::Permanent {
            message: format!(
                "Max retries ({}) exhausted for operation {}",
                retry_strategy.max_attempts, operation_name
            ),
            error_code: "MAX_RETRIES_EXHAUSTED".to_string(),
        };

        self.add_to_dead_letter_queue(operation_name, exhausted_error.clone())
            .await;
        Err(exhausted_error)
    }

    /// Get or create circuit breaker for service
    async fn get_circuit_breaker(&self, service: &str) -> Option<Arc<CircuitBreaker>> {
        let circuit_breakers = self.circuit_breakers.read().await;
        if let Some(cb) = circuit_breakers.get(service) {
            return Some(cb.clone());
        }
        drop(circuit_breakers);

        // Create new circuit breaker
        let config = self
            .config
            .circuit_breaker_configs
            .get(service)
            .or_else(|| self.config.circuit_breaker_configs.get("default"))
            .cloned()?;

        let circuit_breaker = Arc::new(CircuitBreaker::new(config));
        let mut circuit_breakers = self.circuit_breakers.write().await;
        circuit_breakers.insert(service.to_string(), circuit_breaker.clone());

        Some(circuit_breaker)
    }

    /// Determine appropriate recovery strategy for error
    async fn determine_recovery_strategy(
        &self,
        error: &ProactiveError,
        _operation_name: &str,
    ) -> Option<RecoveryStrategy> {
        let classification = error.classify();

        match classification {
            ErrorClassification::Transient => self
                .config
                .retry_strategies
                .get(&classification)
                .map(|retry_strategy| RecoveryStrategy::Retry {
                    delay: retry_strategy.initial_delay,
                    attempt: 1,
                    max_attempts: retry_strategy.max_attempts,
                }),
            ErrorClassification::RateLimit => {
                error.retry_delay().map(|delay| RecoveryStrategy::Retry {
                    delay,
                    attempt: 1,
                    max_attempts: 3,
                })
            }
            ErrorClassification::ResourceExhaustion => {
                error
                    .retry_delay()
                    .map(|delay| RecoveryStrategy::ResourceThrottling {
                        backoff_duration: delay,
                        resource: "system".to_string(),
                    })
            }
            ErrorClassification::CircuitBreaker => Some(RecoveryStrategy::CircuitBreakerWait {
                service: "default".to_string(),
                wait_duration: Duration::from_secs(60),
            }),
            ErrorClassification::ExternalService => Some(RecoveryStrategy::ServiceDegradation {
                alternative_strategy: "fallback_mechanism".to_string(),
            }),
            _ => None,
        }
    }

    /// Execute recovery strategy
    async fn execute_recovery_strategy<F, Fut, T>(
        &self,
        strategy: RecoveryStrategy,
        operation: F,
        operation_name: &str,
        service: &str,
    ) -> Result<T, ProactiveError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, ProactiveError>>,
    {
        match strategy {
            RecoveryStrategy::Retry {
                delay,
                attempt,
                max_attempts,
            } => {
                sleep(delay).await;

                // Retry the operation
                match timeout(self.config.recovery_timeout, operation()).await {
                    Ok(Ok(result)) => Ok(result),
                    Ok(Err(error)) => {
                        if attempt < max_attempts && error.is_retryable() {
                            // Retry with incremented attempt
                            let new_strategy = RecoveryStrategy::Retry {
                                delay: delay.mul_f64(2.0),
                                attempt: attempt + 1,
                                max_attempts,
                            };
                            Box::pin(self.execute_recovery_strategy(
                                new_strategy,
                                operation,
                                operation_name,
                                service,
                            ))
                            .await
                        } else {
                            Err(error)
                        }
                    }
                    Err(_) => Err(ProactiveError::Timeout {
                        operation: operation_name.to_string(),
                        timeout: self.config.recovery_timeout,
                        elapsed: self.config.recovery_timeout,
                    }),
                }
            }
            RecoveryStrategy::CircuitBreakerWait { wait_duration, .. } => {
                sleep(wait_duration).await;
                match timeout(self.config.recovery_timeout, operation()).await {
                    Ok(result) => result,
                    Err(_) => Err(ProactiveError::Timeout {
                        operation: operation_name.to_string(),
                        timeout: self.config.recovery_timeout,
                        elapsed: self.config.recovery_timeout,
                    }),
                }
            }
            RecoveryStrategy::ResourceThrottling {
                backoff_duration, ..
            } => {
                sleep(backoff_duration).await;
                match timeout(self.config.recovery_timeout, operation()).await {
                    Ok(result) => result,
                    Err(_) => Err(ProactiveError::Timeout {
                        operation: operation_name.to_string(),
                        timeout: self.config.recovery_timeout,
                        elapsed: self.config.recovery_timeout,
                    }),
                }
            }
            RecoveryStrategy::ServiceDegradation { .. } => {
                // Implement degraded service operation
                match timeout(self.config.recovery_timeout, operation()).await {
                    Ok(result) => result,
                    Err(_) => Err(ProactiveError::Timeout {
                        operation: operation_name.to_string(),
                        timeout: self.config.recovery_timeout,
                        elapsed: self.config.recovery_timeout,
                    }),
                }
            }
            RecoveryStrategy::ManualIntervention { escalation_reason } => {
                error!(
                    "Manual intervention required for {}: {}",
                    operation_name, escalation_reason
                );
                Err(ProactiveError::Permanent {
                    message: format!("Manual intervention required: {escalation_reason}"),
                    error_code: "MANUAL_INTERVENTION_REQUIRED".to_string(),
                })
            }
        }
    }

    /// Add failed operation to dead letter queue
    async fn add_to_dead_letter_queue(&self, operation: &str, error: ProactiveError) {
        let entry = DeadLetterEntry {
            id: Uuid::new_v4().to_string(),
            original_operation: operation.to_string(),
            final_error: error,
            total_attempts: 0,
            first_failure: Utc::now(),
            final_failure: Utc::now(),
            recovery_attempts: Vec::new(),
            escalated: false,
        };

        let mut dlq = self.dead_letter_queue.lock().await;

        // Maintain size limit
        if dlq.len() >= self.config.dead_letter_queue_size {
            dlq.pop_front();
        }

        dlq.push_back(entry);

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.dead_letter_entries += 1;
    }

    /// Update error metrics
    async fn update_error_metrics(&self, classification: &ErrorClassification) {
        let mut metrics = self.metrics.write().await;
        metrics.total_errors += 1;
        *metrics
            .errors_by_classification
            .entry(classification.clone())
            .or_insert(0) += 1;
        metrics.last_updated = Utc::now();
    }

    /// Get current error metrics
    pub async fn get_metrics(&self) -> ErrorMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Get dead letter queue entries
    pub async fn get_dead_letter_entries(&self) -> Vec<DeadLetterEntry> {
        let dlq = self.dead_letter_queue.lock().await;
        dlq.iter().cloned().collect()
    }

    /// Start monitoring background task
    async fn start_monitoring(&self) {
        let running = self.running.clone();
        let metrics = self.metrics.clone();
        let escalation_thresholds = self.config.escalation_thresholds.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));

            while *running.read().await {
                interval.tick().await;

                let current_metrics = metrics.read().await.clone();

                // Check escalation thresholds
                if current_metrics.dead_letter_entries
                    > escalation_thresholds.max_dead_letter_entries as u64
                {
                    error!(
                        "Dead letter queue threshold exceeded: {} entries",
                        current_metrics.dead_letter_entries
                    );
                }

                if current_metrics.total_errors > 0 {
                    let error_rate = current_metrics
                        .errors_by_classification
                        .values()
                        .sum::<u64>() as f64
                        / current_metrics.total_errors as f64;

                    if error_rate > escalation_thresholds.max_error_rate {
                        error!("Error rate threshold exceeded: {:.2}%", error_rate * 100.0);
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_error_classification() {
        let transient_error = ProactiveError::Transient {
            message: "Temporary failure".to_string(),
            attempt: 1,
            max_attempts: 3,
            retry_after: Some(Duration::from_secs(1)),
        };

        assert_eq!(transient_error.classify(), ErrorClassification::Transient);
        assert!(transient_error.is_retryable());
        assert_eq!(transient_error.retry_delay(), Some(Duration::from_secs(1)));

        let permanent_error = ProactiveError::Permanent {
            message: "Invalid configuration".to_string(),
            error_code: "CONFIG_ERROR".to_string(),
        };

        assert_eq!(permanent_error.classify(), ErrorClassification::Permanent);
        assert!(!permanent_error.is_retryable());
        assert_eq!(permanent_error.retry_delay(), None);
    }

    #[tokio::test]
    async fn test_retry_strategy() {
        let strategy = RetryStrategy::default();

        assert_eq!(strategy.calculate_delay(0), strategy.initial_delay);

        let delay1 = strategy.calculate_delay(1);
        let delay2 = strategy.calculate_delay(2);

        // Should be exponential backoff
        assert!(delay2 > delay1);
        assert!(delay2 <= strategy.max_delay);
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout: Duration::from_millis(100),
            half_open_max_calls: 1,
        };

        let cb = CircuitBreaker::new(config);

        // Initially closed
        assert!(cb.can_execute().await);
        assert_eq!(cb.get_state().await, CircuitState::Closed);

        // Record failures to trip circuit
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);

        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Open);
        assert!(!cb.can_execute().await);

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should now be half-open
        assert!(cb.can_execute().await);
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

        // Record success to close circuit
        cb.record_success().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_error_handler_retry() {
        let config = ErrorHandlerConfig::default();
        let handler = ErrorHandler::new(config);
        handler.start().await.unwrap();

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let operation = || {
            let count = call_count_clone.fetch_add(1, Ordering::SeqCst);
            async move {
                if count < 2 {
                    Err(ProactiveError::Transient {
                        message: "Temporary failure".to_string(),
                        attempt: count + 1,
                        max_attempts: 3,
                        retry_after: Some(Duration::from_millis(10)),
                    })
                } else {
                    Ok("Success".to_string())
                }
            }
        };

        let result = handler
            .execute_with_retry(operation, "test_operation", None)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
        assert_eq!(call_count.load(Ordering::SeqCst), 3);

        handler.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_dead_letter_queue() {
        let config = ErrorHandlerConfig {
            dead_letter_queue_size: 2,
            ..ErrorHandlerConfig::default()
        };
        let handler = ErrorHandler::new(config);

        let error1 = ProactiveError::Permanent {
            message: "Error 1".to_string(),
            error_code: "ERR1".to_string(),
        };

        let error2 = ProactiveError::Permanent {
            message: "Error 2".to_string(),
            error_code: "ERR2".to_string(),
        };

        let error3 = ProactiveError::Permanent {
            message: "Error 3".to_string(),
            error_code: "ERR3".to_string(),
        };

        handler.add_to_dead_letter_queue("op1", error1).await;
        handler.add_to_dead_letter_queue("op2", error2).await;
        handler.add_to_dead_letter_queue("op3", error3).await;

        let entries = handler.get_dead_letter_entries().await;
        assert_eq!(entries.len(), 2); // Size limit enforced

        // Should contain the most recent entries
        assert_eq!(entries[0].original_operation, "op2");
        assert_eq!(entries[1].original_operation, "op3");
    }

    #[tokio::test]
    async fn test_error_metrics() {
        let config = ErrorHandlerConfig::default();
        let handler = ErrorHandler::new(config);

        handler
            .update_error_metrics(&ErrorClassification::Transient)
            .await;
        handler
            .update_error_metrics(&ErrorClassification::Permanent)
            .await;
        handler
            .update_error_metrics(&ErrorClassification::Transient)
            .await;

        let metrics = handler.get_metrics().await;
        assert_eq!(metrics.total_errors, 3);
        assert_eq!(
            metrics
                .errors_by_classification
                .get(&ErrorClassification::Transient),
            Some(&2)
        );
        assert_eq!(
            metrics
                .errors_by_classification
                .get(&ErrorClassification::Permanent),
            Some(&1)
        );
    }
}
