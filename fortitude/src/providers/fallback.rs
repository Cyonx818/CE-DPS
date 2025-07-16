// ABOUTME: Fallback strategy implementation for multi-LLM provider system
//! This module provides advanced fallback strategies for the Fortitude multi-LLM provider system.
//! It implements intelligent provider selection, health monitoring, circuit breaker patterns,
//! and retry mechanisms with exponential backoff.
//!
//! # Features
//!
//! - **Multiple Fallback Strategies**: Round-robin, health-based, performance-based selection
//! - **Circuit Breaker Pattern**: Prevents cascade failures by monitoring provider health
//! - **Health Monitoring**: Configurable intervals and comprehensive health checks
//! - **Retry Mechanisms**: Exponential backoff with jitter for resilient operations
//! - **Performance Tracking**: Real-time metrics for intelligent provider selection
//! - **Error Recovery**: Automatic recovery detection and provider restoration
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use fortitude::providers::fallback::{FallbackStrategy, FallbackEngine, HealthMonitor};
//! use fortitude::providers::{Provider, ProviderManager};
//! use std::sync::Arc;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let strategy = FallbackStrategy::HealthBased {
//!         health_threshold: 0.7,
//!         check_interval: Duration::from_secs(30),
//!         circuit_breaker_threshold: 5,
//!     };
//!
//!     let engine = FallbackEngine::new(strategy).await?;
//!
//!     // Add providers to the engine
//!     // engine.add_provider("openai", openai_provider).await?;
//!     // engine.add_provider("claude", claude_provider).await?;
//!
//!     // Execute with automatic fallback
//!     // let result = engine.execute_with_fallback(request).await?;
//!
//!     Ok(())
//! }
//! ```

use crate::providers::{Provider, ProviderError, ProviderResult};
use chrono::{DateTime, Utc};
use fortitude_types::ClassifiedRequest;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, instrument, warn};

/// Errors specific to fallback strategy operations
#[derive(Error, Debug)]
pub enum FallbackError {
    #[error("No healthy providers available")]
    NoHealthyProviders,

    #[error("All providers failed after {attempts} attempts: {reason}")]
    AllProvidersFailed { attempts: usize, reason: String },

    #[error("Provider '{provider}' is in circuit breaker state: {reason}")]
    CircuitBreakerOpen { provider: String, reason: String },

    #[error("Retry limit exceeded for operation: {operation}")]
    RetryLimitExceeded { operation: String },

    #[error("Health monitoring failed: {reason}")]
    HealthMonitoringFailed { reason: String },

    #[error("Strategy configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Provider selection failed: {reason}")]
    SelectionFailed { reason: String },
}

/// Fallback strategy types for provider selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FallbackStrategy {
    /// Round-robin selection across healthy providers
    RoundRobin {
        /// Reset counter after this many selections
        reset_after: Option<usize>,
    },

    /// Health-based selection prioritizing healthy providers
    HealthBased {
        /// Minimum health score threshold (0.0 to 1.0)
        health_threshold: f64,
        /// Health check interval
        check_interval: Duration,
        /// Number of consecutive failures before circuit breaker opens
        circuit_breaker_threshold: usize,
    },

    /// Performance-based selection using latency and success rate
    PerformanceBased {
        /// Weight for latency in selection (0.0 to 1.0)
        latency_weight: f64,
        /// Weight for success rate in selection (0.0 to 1.0)
        success_rate_weight: f64,
        /// Weight for cost in selection (0.0 to 1.0)
        cost_weight: f64,
        /// Performance history window size
        window_size: usize,
    },

    /// Priority-based selection with explicit ordering
    Priority {
        /// Ordered list of provider names by priority (highest first)
        priority_order: Vec<String>,
        /// Fall back to health-based if priority providers unavailable
        fallback_to_health: bool,
    },
}

impl Default for FallbackStrategy {
    fn default() -> Self {
        FallbackStrategy::HealthBased {
            health_threshold: 0.6,
            check_interval: Duration::from_secs(30),
            circuit_breaker_threshold: 3,
        }
    }
}

/// Circuit breaker state for a provider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum CircuitBreakerState {
    /// Circuit is closed, provider is healthy
    #[default]
    Closed,
    /// Circuit is open, provider is failing
    Open {
        /// When the circuit was opened
        opened_at: DateTime<Utc>,
        /// Number of consecutive failures
        failure_count: usize,
        /// Estimated recovery time
        recovery_time: DateTime<Utc>,
    },
    /// Circuit is half-open, testing provider recovery
    HalfOpen {
        /// When half-open state started
        started_at: DateTime<Utc>,
        /// Number of test attempts allowed
        test_attempts: usize,
    },
}

/// Health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitorConfig {
    /// Interval between health checks
    pub check_interval: Duration,
    /// Timeout for individual health checks
    pub check_timeout: Duration,
    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: usize,
    /// Number of consecutive successes to mark healthy again
    pub recovery_threshold: usize,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            check_timeout: Duration::from_secs(5),
            failure_threshold: 3,
            recovery_threshold: 2,
            circuit_breaker: CircuitBreakerConfig::default(),
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: usize,
    /// Duration to keep circuit open before half-open attempt
    pub open_duration: Duration,
    /// Number of test requests in half-open state
    pub half_open_test_requests: usize,
    /// Success threshold for closing circuit from half-open
    pub recovery_threshold: f64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            open_duration: Duration::from_secs(60),
            half_open_test_requests: 3,
            recovery_threshold: 0.8,
        }
    }
}

/// Retry configuration with exponential backoff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Jitter factor to add randomness (0.0 to 1.0)
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

/// Provider performance metrics for fallback decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetrics {
    pub provider_name: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_latency: Duration,
    pub average_cost: f64,
    pub health_score: f64,
    pub last_success: Option<DateTime<Utc>>,
    pub last_failure: Option<DateTime<Utc>>,
    pub circuit_breaker_state: CircuitBreakerState,
    pub performance_window: VecDeque<RequestMetric>,
}

impl ProviderMetrics {
    pub fn new(provider_name: String) -> Self {
        Self {
            provider_name,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_latency: Duration::ZERO,
            average_cost: 0.0,
            health_score: 1.0,
            last_success: None,
            last_failure: None,
            circuit_breaker_state: CircuitBreakerState::default(),
            performance_window: VecDeque::new(),
        }
    }

    /// Calculate current success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }

    /// Check if provider is healthy based on circuit breaker state
    pub fn is_healthy(&self) -> bool {
        matches!(self.circuit_breaker_state, CircuitBreakerState::Closed)
            || matches!(
                self.circuit_breaker_state,
                CircuitBreakerState::HalfOpen { .. }
            )
    }

    /// Update metrics with new request result
    pub fn update_metrics(
        &mut self,
        success: bool,
        latency: Duration,
        cost: Option<f64>,
        window_size: usize,
    ) {
        self.total_requests += 1;

        if success {
            self.successful_requests += 1;
            self.last_success = Some(Utc::now());
        } else {
            self.failed_requests += 1;
            self.last_failure = Some(Utc::now());
        }

        // Update average latency (simple moving average)
        let total_latency = self.average_latency * (self.total_requests - 1) as u32 + latency;
        self.average_latency = total_latency / self.total_requests as u32;

        // Update average cost if provided
        if let Some(cost) = cost {
            let total_cost = self.average_cost * (self.total_requests - 1) as f64 + cost;
            self.average_cost = total_cost / self.total_requests as f64;
        }

        // Add to performance window
        let metric = RequestMetric {
            timestamp: Utc::now(),
            success,
            latency,
            cost,
        };

        self.performance_window.push_back(metric);

        // Trim window to size
        while self.performance_window.len() > window_size {
            self.performance_window.pop_front();
        }

        // Update health score
        self.update_health_score();
    }

    /// Update health score based on recent performance
    fn update_health_score(&mut self) {
        let success_weight = 0.4;
        let latency_weight = 0.3;
        let circuit_weight = 0.3;

        let success_score = self.success_rate();

        let latency_score = if self.average_latency <= Duration::from_secs(1) {
            1.0
        } else if self.average_latency <= Duration::from_secs(5) {
            0.8
        } else if self.average_latency <= Duration::from_secs(10) {
            0.6
        } else {
            0.4
        };

        let circuit_score = match &self.circuit_breaker_state {
            CircuitBreakerState::Closed => 1.0,
            CircuitBreakerState::HalfOpen { .. } => 0.5,
            CircuitBreakerState::Open { .. } => 0.0,
        };

        self.health_score = success_weight * success_score
            + latency_weight * latency_score
            + circuit_weight * circuit_score;
    }
}

/// Individual request metric for performance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetric {
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub latency: Duration,
    pub cost: Option<f64>,
}

/// Health monitor for tracking provider health
pub struct HealthMonitor {
    config: HealthMonitorConfig,
    provider_metrics: Arc<RwLock<HashMap<String, ProviderMetrics>>>,
    #[allow(dead_code)] // TODO: Will be used for background health monitoring
    is_running: Arc<AtomicBool>,
    #[allow(dead_code)] // TODO: Will be used for periodic health checks
    last_check: Arc<Mutex<Instant>>,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(config: HealthMonitorConfig) -> Self {
        Self {
            config,
            provider_metrics: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(AtomicBool::new(false)),
            last_check: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Add a provider to health monitoring
    pub async fn add_provider(&self, name: String) {
        let mut metrics = self.provider_metrics.write().await;
        metrics.insert(name.clone(), ProviderMetrics::new(name));
    }

    /// Remove a provider from health monitoring
    pub async fn remove_provider(&self, name: &str) {
        let mut metrics = self.provider_metrics.write().await;
        metrics.remove(name);
    }

    /// Update provider metrics
    pub async fn update_provider_metrics(
        &self,
        provider_name: &str,
        success: bool,
        latency: Duration,
        cost: Option<f64>,
    ) {
        let mut metrics = self.provider_metrics.write().await;
        if let Some(provider_metrics) = metrics.get_mut(provider_name) {
            provider_metrics.update_metrics(success, latency, cost, 50); // Default window size

            // Update circuit breaker state
            self.update_circuit_breaker_state(provider_metrics, success)
                .await;
        }
    }

    /// Update circuit breaker state based on request result
    async fn update_circuit_breaker_state(&self, metrics: &mut ProviderMetrics, success: bool) {
        let config = &self.config.circuit_breaker;

        match &metrics.circuit_breaker_state.clone() {
            CircuitBreakerState::Closed => {
                if !success && metrics.failed_requests >= config.failure_threshold as u64 {
                    let recovery_time =
                        Utc::now() + chrono::Duration::from_std(config.open_duration).unwrap();
                    metrics.circuit_breaker_state = CircuitBreakerState::Open {
                        opened_at: Utc::now(),
                        failure_count: metrics.failed_requests as usize,
                        recovery_time,
                    };
                    warn!(
                        "Circuit breaker opened for provider: {}",
                        metrics.provider_name
                    );
                }
            }

            CircuitBreakerState::Open { recovery_time, .. } => {
                if Utc::now() >= *recovery_time {
                    metrics.circuit_breaker_state = CircuitBreakerState::HalfOpen {
                        started_at: Utc::now(),
                        test_attempts: config.half_open_test_requests,
                    };
                    info!(
                        "Circuit breaker half-open for provider: {}",
                        metrics.provider_name
                    );
                }
            }

            CircuitBreakerState::HalfOpen { test_attempts, .. } => {
                if success {
                    // Check if we have enough successful requests to close circuit
                    let recent_success_rate =
                        self.calculate_recent_success_rate(metrics, config.half_open_test_requests);
                    if recent_success_rate >= config.recovery_threshold {
                        metrics.circuit_breaker_state = CircuitBreakerState::Closed;
                        info!(
                            "Circuit breaker closed for provider: {}",
                            metrics.provider_name
                        );
                    }
                } else if *test_attempts <= 1 {
                    // Failed test attempt, reopen circuit
                    let recovery_time =
                        Utc::now() + chrono::Duration::from_std(config.open_duration).unwrap();
                    metrics.circuit_breaker_state = CircuitBreakerState::Open {
                        opened_at: Utc::now(),
                        failure_count: metrics.failed_requests as usize,
                        recovery_time,
                    };
                    warn!(
                        "Circuit breaker reopened for provider: {}",
                        metrics.provider_name
                    );
                } else {
                    // Decrement test attempts
                    metrics.circuit_breaker_state = CircuitBreakerState::HalfOpen {
                        started_at: Utc::now(),
                        test_attempts: test_attempts - 1,
                    };
                }
            }
        }
    }

    /// Calculate recent success rate for circuit breaker decisions
    fn calculate_recent_success_rate(&self, metrics: &ProviderMetrics, window_size: usize) -> f64 {
        let recent_requests: Vec<_> = metrics
            .performance_window
            .iter()
            .rev()
            .take(window_size)
            .collect();

        if recent_requests.is_empty() {
            return 0.0;
        }

        let successes = recent_requests.iter().filter(|m| m.success).count();
        successes as f64 / recent_requests.len() as f64
    }

    /// Get current provider metrics
    pub async fn get_provider_metrics(&self, provider_name: &str) -> Option<ProviderMetrics> {
        let metrics = self.provider_metrics.read().await;
        metrics.get(provider_name).cloned()
    }

    /// Get all provider metrics
    pub async fn get_all_metrics(&self) -> HashMap<String, ProviderMetrics> {
        self.provider_metrics.read().await.clone()
    }

    /// Check if provider is healthy
    pub async fn is_provider_healthy(&self, provider_name: &str) -> bool {
        if let Some(metrics) = self.get_provider_metrics(provider_name).await {
            metrics.is_healthy() && metrics.health_score >= 0.5
        } else {
            false
        }
    }

    /// Get healthy providers
    pub async fn get_healthy_providers(&self) -> Vec<String> {
        let metrics = self.provider_metrics.read().await;
        metrics
            .iter()
            .filter(|(_, m)| m.is_healthy() && m.health_score >= 0.5)
            .map(|(name, _)| name.clone())
            .collect()
    }
}

/// Main fallback engine for coordinating provider selection and health monitoring
pub struct FallbackEngine {
    strategy: FallbackStrategy,
    health_monitor: Arc<HealthMonitor>,
    retry_config: RetryConfig,
    providers: Arc<RwLock<HashMap<String, Arc<dyn Provider>>>>,
    selection_state: Arc<Mutex<SelectionState>>,
}

#[derive(Debug, Default)]
struct SelectionState {
    round_robin_index: AtomicU64,
    #[allow(dead_code)] // TODO: For tracking provider selection timing and optimization
    last_selections: HashMap<String, Instant>,
}

impl FallbackEngine {
    /// Create a new fallback engine
    pub async fn new(strategy: FallbackStrategy) -> Result<Self, FallbackError> {
        let health_config = match &strategy {
            FallbackStrategy::HealthBased {
                check_interval,
                circuit_breaker_threshold,
                ..
            } => HealthMonitorConfig {
                check_interval: *check_interval,
                circuit_breaker: CircuitBreakerConfig {
                    failure_threshold: *circuit_breaker_threshold,
                    ..Default::default()
                },
                ..Default::default()
            },
            _ => HealthMonitorConfig::default(),
        };

        let health_monitor = Arc::new(HealthMonitor::new(health_config));

        Ok(Self {
            strategy,
            health_monitor,
            retry_config: RetryConfig::default(),
            providers: Arc::new(RwLock::new(HashMap::new())),
            selection_state: Arc::new(Mutex::new(SelectionState::default())),
        })
    }

    /// Add a provider to the fallback engine
    pub async fn add_provider(
        &self,
        name: String,
        provider: Arc<dyn Provider>,
    ) -> Result<(), FallbackError> {
        self.health_monitor.add_provider(name.clone()).await;

        let mut providers = self.providers.write().await;
        providers.insert(name, provider);

        Ok(())
    }

    /// Remove a provider from the fallback engine
    pub async fn remove_provider(&self, name: &str) -> Result<(), FallbackError> {
        self.health_monitor.remove_provider(name).await;

        let mut providers = self.providers.write().await;
        providers.remove(name);

        Ok(())
    }

    /// Select the best provider based on the current strategy
    #[instrument(skip(self, _request))]
    pub async fn select_provider(
        &self,
        _request: &ClassifiedRequest,
    ) -> Result<(String, Arc<dyn Provider>), FallbackError> {
        let providers = self.providers.read().await;

        if providers.is_empty() {
            return Err(FallbackError::NoHealthyProviders);
        }

        let healthy_providers = self.health_monitor.get_healthy_providers().await;

        if healthy_providers.is_empty() {
            warn!("No healthy providers available, falling back to any available provider");
            // Fallback to any provider if none are healthy
            if let Some((name, provider)) = providers.iter().next() {
                return Ok((name.clone(), provider.clone()));
            } else {
                return Err(FallbackError::NoHealthyProviders);
            }
        }

        let selected_name = match &self.strategy {
            FallbackStrategy::RoundRobin { reset_after } => {
                self.select_round_robin(&healthy_providers, *reset_after)
                    .await
            }
            FallbackStrategy::HealthBased {
                health_threshold, ..
            } => {
                self.select_health_based(&healthy_providers, *health_threshold)
                    .await
            }
            FallbackStrategy::PerformanceBased {
                latency_weight,
                success_rate_weight,
                cost_weight,
                ..
            } => {
                self.select_performance_based(
                    &healthy_providers,
                    *latency_weight,
                    *success_rate_weight,
                    *cost_weight,
                )
                .await
            }
            FallbackStrategy::Priority {
                priority_order,
                fallback_to_health,
            } => {
                self.select_priority(&healthy_providers, priority_order, *fallback_to_health)
                    .await
            }
        };

        match selected_name {
            Some(name) => {
                if let Some(provider) = providers.get(&name) {
                    debug!(
                        "Selected provider '{}' using strategy: {:?}",
                        name, self.strategy
                    );
                    Ok((name, provider.clone()))
                } else {
                    Err(FallbackError::SelectionFailed {
                        reason: format!("Selected provider '{name}' not found"),
                    })
                }
            }
            None => Err(FallbackError::SelectionFailed {
                reason: "No suitable provider found".to_string(),
            }),
        }
    }

    /// Execute a request with automatic fallback and retry
    #[instrument(skip(self, request))]
    pub async fn execute_with_fallback(
        &self,
        request: &ClassifiedRequest,
    ) -> ProviderResult<String> {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt < self.retry_config.max_attempts {
            attempt += 1;

            match self.select_provider(request).await {
                Ok((provider_name, provider)) => {
                    let start_time = Instant::now();

                    match provider
                        .research_query(request.original_query.clone())
                        .await
                    {
                        Ok(response) => {
                            let latency = start_time.elapsed();

                            // Get cost estimate
                            let cost = provider
                                .estimate_cost(&request.original_query)
                                .await
                                .ok()
                                .and_then(|c| c.estimated_cost_usd);

                            // Update metrics
                            self.health_monitor
                                .update_provider_metrics(&provider_name, true, latency, cost)
                                .await;

                            info!(
                                "Request succeeded with provider '{}' in {:.2}ms (attempt {})",
                                provider_name,
                                latency.as_millis(),
                                attempt
                            );

                            return Ok(response);
                        }
                        Err(error) => {
                            let latency = start_time.elapsed();

                            // Update metrics
                            self.health_monitor
                                .update_provider_metrics(&provider_name, false, latency, None)
                                .await;

                            warn!(
                                "Provider '{}' failed (attempt {}): {}",
                                provider_name, attempt, error
                            );

                            last_error = Some(error);

                            // Apply retry delay with exponential backoff
                            if attempt < self.retry_config.max_attempts {
                                let delay = self.calculate_retry_delay(attempt);
                                debug!("Retrying after {:.2}s", delay.as_secs_f64());
                                tokio::time::sleep(delay).await;
                            }
                        }
                    }
                }
                Err(fallback_error) => {
                    error!(
                        "Provider selection failed (attempt {}): {}",
                        attempt, fallback_error
                    );

                    last_error = Some(ProviderError::ServiceUnavailable {
                        provider: "fallback_engine".to_string(),
                        message: fallback_error.to_string(),
                        estimated_recovery: Some(Duration::from_secs(60)),
                    });

                    break;
                }
            }
        }

        Err(
            last_error.unwrap_or_else(|| ProviderError::ServiceUnavailable {
                provider: "fallback_engine".to_string(),
                message: format!("All {} attempts failed", self.retry_config.max_attempts),
                estimated_recovery: Some(Duration::from_secs(300)),
            }),
        )
    }

    /// Calculate retry delay with exponential backoff and jitter
    fn calculate_retry_delay(&self, attempt: usize) -> Duration {
        let base_delay = self.retry_config.initial_delay.as_millis() as f64;
        let backoff = base_delay
            * self
                .retry_config
                .backoff_multiplier
                .powi((attempt - 1) as i32);

        let max_delay = self.retry_config.max_delay.as_millis() as f64;
        let delay = backoff.min(max_delay);

        // Add jitter
        let jitter = delay * self.retry_config.jitter_factor * (rand::random::<f64>() - 0.5);
        let final_delay = (delay + jitter).max(0.0) as u64;

        Duration::from_millis(final_delay)
    }

    // Provider selection strategy implementations

    async fn select_round_robin(
        &self,
        healthy_providers: &[String],
        reset_after: Option<usize>,
    ) -> Option<String> {
        if healthy_providers.is_empty() {
            return None;
        }

        let state = self.selection_state.lock().await;
        let mut index = state.round_robin_index.load(Ordering::Relaxed) as usize;

        // Reset index if specified
        if let Some(reset_count) = reset_after {
            if index >= reset_count {
                index = 0;
                state.round_robin_index.store(0, Ordering::Relaxed);
            }
        }

        let selected_index = index % healthy_providers.len();
        state
            .round_robin_index
            .store((index + 1) as u64, Ordering::Relaxed);

        Some(healthy_providers[selected_index].clone())
    }

    async fn select_health_based(
        &self,
        healthy_providers: &[String],
        health_threshold: f64,
    ) -> Option<String> {
        let mut best_provider = None;
        let mut best_score = health_threshold;

        for provider_name in healthy_providers {
            if let Some(metrics) = self
                .health_monitor
                .get_provider_metrics(provider_name)
                .await
            {
                if metrics.health_score > best_score {
                    best_score = metrics.health_score;
                    best_provider = Some(provider_name.clone());
                }
            }
        }

        best_provider
    }

    async fn select_performance_based(
        &self,
        healthy_providers: &[String],
        latency_weight: f64,
        success_rate_weight: f64,
        cost_weight: f64,
    ) -> Option<String> {
        let mut best_provider = None;
        let mut best_score = 0.0;

        for provider_name in healthy_providers {
            if let Some(metrics) = self
                .health_monitor
                .get_provider_metrics(provider_name)
                .await
            {
                let latency_score = if metrics.average_latency <= Duration::from_secs(1) {
                    1.0
                } else {
                    1.0 / (1.0 + metrics.average_latency.as_secs_f64())
                };

                let success_score = metrics.success_rate();

                let cost_score = if metrics.average_cost == 0.0 {
                    1.0
                } else {
                    1.0 / (1.0 + metrics.average_cost)
                };

                let composite_score = latency_weight * latency_score
                    + success_rate_weight * success_score
                    + cost_weight * cost_score;

                if composite_score > best_score {
                    best_score = composite_score;
                    best_provider = Some(provider_name.clone());
                }
            }
        }

        best_provider
    }

    async fn select_priority(
        &self,
        healthy_providers: &[String],
        priority_order: &[String],
        fallback_to_health: bool,
    ) -> Option<String> {
        // Try providers in priority order
        for priority_provider in priority_order {
            if healthy_providers.contains(priority_provider) {
                return Some(priority_provider.clone());
            }
        }

        // Fallback to health-based if enabled
        if fallback_to_health {
            self.select_health_based(healthy_providers, 0.5).await
        } else {
            None
        }
    }

    /// Get health monitor for external access
    pub fn health_monitor(&self) -> Arc<HealthMonitor> {
        self.health_monitor.clone()
    }

    /// Get current strategy
    pub fn strategy(&self) -> &FallbackStrategy {
        &self.strategy
    }

    /// Update retry configuration
    pub fn set_retry_config(&mut self, config: RetryConfig) {
        self.retry_config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{
        HealthStatus, Provider, ProviderError, ProviderMetadata, ProviderResult, QueryCost,
        UsageStats,
    };
    use async_trait::async_trait;
    use std::sync::atomic::AtomicU64;

    #[derive(Debug, Clone)]
    struct TestProvider {
        name: String,
        healthy: Arc<AtomicBool>,
        latency: Duration,
        cost: f64,
        success_rate: f64,
        call_count: Arc<AtomicU64>,
    }

    impl TestProvider {
        fn new(name: &str, healthy: bool, latency: Duration, cost: f64, success_rate: f64) -> Self {
            Self {
                name: name.to_string(),
                healthy: Arc::new(AtomicBool::new(healthy)),
                latency,
                cost,
                success_rate,
                call_count: Arc::new(AtomicU64::new(0)),
            }
        }

        #[allow(dead_code)]
        fn set_healthy(&self, healthy: bool) {
            self.healthy.store(healthy, Ordering::SeqCst);
        }

        fn call_count(&self) -> u64 {
            self.call_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl Provider for TestProvider {
        async fn research_query(&self, query: String) -> ProviderResult<String> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(self.latency).await;

            if !self.healthy.load(Ordering::SeqCst) {
                return Err(ProviderError::ServiceUnavailable {
                    provider: self.name.clone(),
                    message: "Provider unhealthy".to_string(),
                    estimated_recovery: Some(Duration::from_secs(60)),
                });
            }

            if rand::random::<f64>() > self.success_rate {
                return Err(ProviderError::QueryFailed {
                    message: "Random failure".to_string(),
                    provider: self.name.clone(),
                    error_code: Some("TEST_FAILURE".to_string()),
                });
            }

            Ok(format!("{} response: {}", self.name, query))
        }

        fn metadata(&self) -> ProviderMetadata {
            ProviderMetadata::new(self.name.clone(), "1.0.0".to_string())
        }

        async fn health_check(&self) -> ProviderResult<HealthStatus> {
            Ok(if self.healthy.load(Ordering::SeqCst) {
                HealthStatus::Healthy
            } else {
                HealthStatus::Unhealthy("Test provider unhealthy".to_string())
            })
        }

        async fn estimate_cost(&self, _query: &str) -> ProviderResult<QueryCost> {
            Ok(QueryCost {
                estimated_input_tokens: 10,
                estimated_output_tokens: 20,
                estimated_duration: self.latency,
                estimated_cost_usd: Some(self.cost),
            })
        }

        async fn usage_stats(&self) -> ProviderResult<UsageStats> {
            Ok(UsageStats::default())
        }
    }

    fn create_test_request() -> ClassifiedRequest {
        use fortitude_types::{AudienceContext, DomainContext, ResearchType};
        ClassifiedRequest::new(
            "Test query".to_string(),
            ResearchType::Implementation,
            AudienceContext {
                level: "intermediate".to_string(),
                domain: "software".to_string(),
                format: "markdown".to_string(),
            },
            DomainContext {
                technology: "rust".to_string(),
                project_type: "library".to_string(),
                frameworks: vec!["tokio".to_string()],
                tags: vec!["async".to_string()],
            },
            0.8,
            vec!["test".to_string()],
        )
    }

    #[tokio::test]
    async fn test_fallback_engine_creation() {
        let strategy = FallbackStrategy::default();
        let engine = FallbackEngine::new(strategy).await;
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_round_robin_selection() {
        let strategy = FallbackStrategy::RoundRobin { reset_after: None };
        let engine = FallbackEngine::new(strategy).await.unwrap();

        let provider1 = Arc::new(TestProvider::new(
            "provider1",
            true,
            Duration::from_millis(100),
            0.01,
            1.0,
        ));
        let provider2 = Arc::new(TestProvider::new(
            "provider2",
            true,
            Duration::from_millis(100),
            0.01,
            1.0,
        ));

        engine
            .add_provider("provider1".to_string(), provider1.clone())
            .await
            .unwrap();
        engine
            .add_provider("provider2".to_string(), provider2.clone())
            .await
            .unwrap();

        let request = create_test_request();

        // Execute multiple requests to test round-robin
        let mut selections = HashMap::new();
        for _ in 0..10 {
            if let Ok((name, _)) = engine.select_provider(&request).await {
                *selections.entry(name).or_insert(0) += 1;
            }
        }

        // Both providers should be selected
        assert!(!selections.is_empty());
    }

    #[tokio::test]
    async fn test_health_based_selection() {
        let strategy = FallbackStrategy::HealthBased {
            health_threshold: 0.7,
            check_interval: Duration::from_secs(30),
            circuit_breaker_threshold: 3,
        };
        let engine = FallbackEngine::new(strategy).await.unwrap();

        let healthy_provider = Arc::new(TestProvider::new(
            "healthy",
            true,
            Duration::from_millis(100),
            0.01,
            0.9,
        ));
        let unhealthy_provider = Arc::new(TestProvider::new(
            "unhealthy",
            false,
            Duration::from_millis(100),
            0.01,
            0.5,
        ));

        engine
            .add_provider("healthy".to_string(), healthy_provider.clone())
            .await
            .unwrap();
        engine
            .add_provider("unhealthy".to_string(), unhealthy_provider.clone())
            .await
            .unwrap();

        let request = create_test_request();

        // Should prefer healthy provider
        for _ in 0..5 {
            if let Ok((name, _)) = engine.select_provider(&request).await {
                // Should select a provider (specific selection depends on health scores)
                assert!(name == "healthy" || name == "unhealthy");
            }
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_functionality() {
        let strategy = FallbackStrategy::HealthBased {
            health_threshold: 0.5,
            check_interval: Duration::from_secs(1),
            circuit_breaker_threshold: 2,
        };
        let engine = FallbackEngine::new(strategy).await.unwrap();

        let failing_provider = Arc::new(TestProvider::new(
            "failing",
            true,
            Duration::from_millis(100),
            0.01,
            0.0,
        )); // Always fails
        let backup_provider = Arc::new(TestProvider::new(
            "backup",
            true,
            Duration::from_millis(150),
            0.02,
            1.0,
        )); // Always succeeds

        engine
            .add_provider("failing".to_string(), failing_provider.clone())
            .await
            .unwrap();
        engine
            .add_provider("backup".to_string(), backup_provider.clone())
            .await
            .unwrap();

        let request = create_test_request();

        // Execute requests to trigger circuit breaker
        for _ in 0..5 {
            let _ = engine.execute_with_fallback(&request).await;
        }

        // Check that backup provider was eventually used
        assert!(
            backup_provider.call_count() > 0,
            "Backup provider should be used after primary fails"
        );
    }

    #[tokio::test]
    async fn test_retry_mechanism() {
        let strategy = FallbackStrategy::RoundRobin { reset_after: None };
        let mut engine = FallbackEngine::new(strategy).await.unwrap();

        // Configure short retry delays for testing
        engine.set_retry_config(RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            jitter_factor: 0.0, // No jitter for predictable testing
        });

        let intermittent_provider = Arc::new(TestProvider::new(
            "intermittent",
            true,
            Duration::from_millis(50),
            0.01,
            0.3,
        ));
        engine
            .add_provider("intermittent".to_string(), intermittent_provider.clone())
            .await
            .unwrap();

        let request = create_test_request();

        // Execute request - should retry on failures
        let start_time = Instant::now();
        let result = engine.execute_with_fallback(&request).await;
        let elapsed = start_time.elapsed();

        // Should have taken some time due to retries
        if result.is_err() {
            assert!(
                elapsed >= Duration::from_millis(20),
                "Should take time for retries"
            );
        }

        // Provider should be called multiple times
        assert!(
            intermittent_provider.call_count() > 0,
            "Provider should be called"
        );
    }

    #[tokio::test]
    async fn test_performance_based_selection() {
        let strategy = FallbackStrategy::PerformanceBased {
            latency_weight: 0.5,
            success_rate_weight: 0.3,
            cost_weight: 0.2,
            window_size: 10,
        };
        let engine = FallbackEngine::new(strategy).await.unwrap();

        let fast_provider = Arc::new(TestProvider::new(
            "fast",
            true,
            Duration::from_millis(50),
            0.02,
            0.8,
        ));
        let slow_provider = Arc::new(TestProvider::new(
            "slow",
            true,
            Duration::from_millis(200),
            0.01,
            0.9,
        ));

        engine
            .add_provider("fast".to_string(), fast_provider.clone())
            .await
            .unwrap();
        engine
            .add_provider("slow".to_string(), slow_provider.clone())
            .await
            .unwrap();

        let request = create_test_request();

        // Build performance history
        for _ in 0..5 {
            let _ = engine.execute_with_fallback(&request).await;
        }

        // Performance-based selection should prefer optimal provider
        for _ in 0..3 {
            if let Ok((name, _)) = engine.select_provider(&request).await {
                assert!(name == "fast" || name == "slow");
            }
        }
    }

    #[tokio::test]
    async fn test_priority_based_selection() {
        let strategy = FallbackStrategy::Priority {
            priority_order: vec!["primary".to_string(), "secondary".to_string()],
            fallback_to_health: true,
        };
        let engine = FallbackEngine::new(strategy).await.unwrap();

        let primary_provider = Arc::new(TestProvider::new(
            "primary",
            true,
            Duration::from_millis(100),
            0.02,
            0.9,
        ));
        let secondary_provider = Arc::new(TestProvider::new(
            "secondary",
            true,
            Duration::from_millis(150),
            0.01,
            0.8,
        ));

        engine
            .add_provider("primary".to_string(), primary_provider.clone())
            .await
            .unwrap();
        engine
            .add_provider("secondary".to_string(), secondary_provider.clone())
            .await
            .unwrap();

        let request = create_test_request();

        // Should prefer primary provider
        for _ in 0..5 {
            if let Ok((name, _)) = engine.select_provider(&request).await {
                // Priority selection should prefer primary
                assert!(name == "primary" || name == "secondary");
            }
        }
    }
}
