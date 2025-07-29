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

// ABOUTME: Provider manager for dynamic provider selection, fallback, and performance tracking
//! This module provides the ProviderManager that orchestrates multiple LLM providers for research queries.
//! It handles intelligent provider selection, automatic failover, performance tracking, and cost optimization.
//!
//! # Features
//!
//! - **Dynamic Provider Selection**: Chooses optimal provider based on query characteristics
//! - **Automatic Failover**: Seamlessly switches to backup providers on failures
//! - **Performance Tracking**: Monitors provider performance, latency, and success rates
//! - **Cost Optimization**: Routes queries to cost-effective providers when appropriate
//! - **Health Monitoring**: Tracks provider health and availability
//! - **Load Balancing**: Distributes load across multiple providers
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use fortitude::providers::manager::{ProviderManager, ProviderConfig, SelectionStrategy};
//! use fortitude::providers::{OpenAIProvider, ClaudeProvider};
//! use fortitude::providers::config::ProviderSettings;
//! use fortitude_types::{ClassifiedRequest, ResearchType};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut manager = ProviderManager::new(ProviderConfig::default()).await?;
//!
//!     // Add providers
//!     let openai_settings = ProviderSettings::new(
//!         std::env::var("OPENAI_API_KEY")?,
//!         "gpt-4".to_string()
//!     );
//!     let openai_provider = OpenAIProvider::new(openai_settings).await?;
//!     manager.add_provider("openai".to_string(), Box::new(openai_provider)).await?;
//!
//!     let claude_settings = ProviderSettings::new(
//!         std::env::var("CLAUDE_API_KEY")?,
//!         "claude-3-5-sonnet-20241022".to_string()
//!     );
//!     let claude_provider = ClaudeProvider::new(claude_settings).await?;
//!     manager.add_provider("claude".to_string(), Box::new(claude_provider)).await?;
//!
//!     // Execute research with intelligent provider selection
//!     let request = ClassifiedRequest::new(
//!         "How to implement async Rust?".to_string(),
//!         ResearchType::Implementation,
//!         // ... other fields
//!     );
//!
//!     let response = manager.execute_research(&request).await?;
//!     println!("Research result: {}", response.immediate_answer);
//!
//!     Ok(())
//! }
//! ```

use crate::providers::{HealthStatus, Provider, ProviderError, ProviderResult};
use chrono::Utc;
use fortitude_types::{
    AudienceContext, ClassifiedRequest, DomainContext, ResearchMetadata, ResearchResult,
    ResearchType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

/// Errors specific to provider management
#[derive(Error, Debug)]
pub enum ProviderManagerError {
    #[error("No providers configured")]
    NoProviders,

    #[error("Provider '{0}' not found")]
    ProviderNotFound(String),

    #[error("All providers failed: {0}")]
    AllProvidersFailed(String),

    #[error("Provider selection failed: {0}")]
    SelectionFailed(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Health check failed for provider '{provider}': {reason}")]
    HealthCheckFailed { provider: String, reason: String },
}

/// Provider selection strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum SelectionStrategy {
    /// Round-robin across all healthy providers
    RoundRobin,
    /// Choose provider with lowest average latency
    LowestLatency,
    /// Choose provider with highest success rate
    HighestSuccessRate,
    /// Choose most cost-effective provider
    CostOptimized,
    /// Choose provider based on research type characteristics
    ResearchTypeOptimized,
    /// Balanced approach considering latency, success rate, and cost
    #[default]
    Balanced,
}

/// Configuration for provider manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Default provider selection strategy
    pub selection_strategy: SelectionStrategy,

    /// Enable automatic failover on provider failures
    pub enable_failover: bool,

    /// Enable cross-provider result validation
    pub enable_cross_validation: bool,

    /// Maximum number of providers to try before giving up
    pub max_failover_attempts: usize,

    /// Timeout for individual provider requests
    pub provider_timeout: Duration,

    /// Interval for health checks
    pub health_check_interval: Duration,

    /// Enable performance tracking
    pub enable_performance_tracking: bool,

    /// Performance history window size
    pub performance_window_size: usize,

    /// Cost optimization threshold (prefer cheaper if quality difference < threshold)
    pub cost_optimization_threshold: f64,

    /// Minimum quality score threshold
    pub min_quality_threshold: f64,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            selection_strategy: SelectionStrategy::default(),
            enable_failover: true,
            enable_cross_validation: false,
            max_failover_attempts: 3,
            provider_timeout: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(300), // 5 minutes
            enable_performance_tracking: true,
            performance_window_size: 100,
            cost_optimization_threshold: 0.1, // 10% quality difference tolerance
            min_quality_threshold: 0.6,
        }
    }
}

/// Provider performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPerformance {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_latency: Duration,
    pub total_cost: f64,
    pub quality_scores: Vec<f64>,
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
    pub last_failure: Option<chrono::DateTime<chrono::Utc>>,
    pub consecutive_failures: u32,
    pub health_status: HealthStatus,
}

impl Default for ProviderPerformance {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_latency: Duration::ZERO,
            total_cost: 0.0,
            quality_scores: Vec::new(),
            last_success: None,
            last_failure: None,
            consecutive_failures: 0,
            health_status: HealthStatus::Healthy,
        }
    }
}

impl ProviderPerformance {
    /// Calculate average response time
    pub fn average_latency(&self) -> Duration {
        if self.total_requests == 0 {
            Duration::ZERO
        } else {
            self.total_latency / self.total_requests as u32
        }
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }

    /// Calculate average quality score
    pub fn average_quality(&self) -> f64 {
        if self.quality_scores.is_empty() {
            0.0
        } else {
            self.quality_scores.iter().sum::<f64>() / self.quality_scores.len() as f64
        }
    }

    /// Calculate average cost per request
    pub fn average_cost(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.total_cost / self.total_requests as f64
        }
    }

    /// Calculate provider health score (0.0 to 1.0)
    pub fn health_score(&self) -> f64 {
        let success_weight = 0.4;
        let latency_weight = 0.3;
        let quality_weight = 0.3;

        let success_score = self.success_rate();

        // Latency score: prefer sub-second responses
        let latency_score = if self.average_latency() <= Duration::from_secs(1) {
            1.0
        } else if self.average_latency() <= Duration::from_secs(5) {
            0.8
        } else if self.average_latency() <= Duration::from_secs(10) {
            0.6
        } else {
            0.4
        };

        let quality_score = self.average_quality();

        success_weight * success_score
            + latency_weight * latency_score
            + quality_weight * quality_score
    }

    /// Check if provider is considered healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.health_status, HealthStatus::Healthy)
            && self.consecutive_failures < 3
            && (self.total_requests == 0 || self.success_rate() > 0.5)
    }
}

/// Provider wrapper with performance tracking
struct ManagedProvider {
    provider: Arc<dyn Provider>,
    performance: Arc<Mutex<ProviderPerformance>>,
    last_health_check: Arc<Mutex<Instant>>,
}

impl ManagedProvider {
    fn new(provider: Arc<dyn Provider>) -> Self {
        Self {
            provider,
            performance: Arc::new(Mutex::new(ProviderPerformance::default())),
            last_health_check: Arc::new(Mutex::new(Instant::now())),
        }
    }

    async fn update_performance(
        &self,
        success: bool,
        latency: Duration,
        cost: Option<f64>,
        quality: Option<f64>,
    ) {
        let mut perf = self.performance.lock().await;
        perf.total_requests += 1;
        perf.total_latency += latency;

        if success {
            perf.successful_requests += 1;
            perf.consecutive_failures = 0;
            perf.last_success = Some(Utc::now());

            // Update health status on successful requests
            if !matches!(perf.health_status, HealthStatus::Healthy) {
                perf.health_status = HealthStatus::Healthy;
            }
        } else {
            perf.failed_requests += 1;
            perf.consecutive_failures += 1;
            perf.last_failure = Some(Utc::now());

            // Update health status on failures
            if perf.consecutive_failures >= 3 {
                perf.health_status =
                    HealthStatus::Unhealthy("Too many consecutive failures".to_string());
            }
        }

        if let Some(cost) = cost {
            perf.total_cost += cost;
        }

        if let Some(quality) = quality {
            perf.quality_scores.push(quality);
            // Keep only recent quality scores
            if perf.quality_scores.len() > 50 {
                perf.quality_scores.remove(0);
            }
        }
    }

    async fn get_performance(&self) -> ProviderPerformance {
        self.performance.lock().await.clone()
    }

    async fn needs_health_check(&self, interval: Duration) -> bool {
        let last_check = *self.last_health_check.lock().await;
        last_check.elapsed() >= interval
    }

    async fn update_health_status(&self, status: HealthStatus) {
        let mut perf = self.performance.lock().await;
        perf.health_status = status;
        *self.last_health_check.lock().await = Instant::now();
    }
}

/// Main provider manager responsible for coordinating multiple LLM providers
pub struct ProviderManager {
    providers: Arc<RwLock<HashMap<String, ManagedProvider>>>,
    config: ProviderConfig,
    selection_state: Arc<Mutex<SelectionState>>,
    performance_tracker: Arc<RwLock<HashMap<String, ProviderPerformance>>>,
}

#[derive(Debug, Default)]
struct SelectionState {
    round_robin_index: AtomicU64,
    #[allow(dead_code)] // TODO: For provider selection timing optimization
    last_selected: HashMap<String, Instant>,
}

impl ProviderManager {
    /// Create a new provider manager
    pub async fn new(config: ProviderConfig) -> Result<Self, ProviderManagerError> {
        Ok(Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            config,
            selection_state: Arc::new(Mutex::new(SelectionState::default())),
            performance_tracker: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Add a provider to the manager
    pub async fn add_provider(
        &self,
        name: String,
        provider: Arc<dyn Provider>,
    ) -> Result<(), ProviderManagerError> {
        info!("Adding provider: {}", name);

        // Initial health check
        let health_status =
            provider
                .health_check()
                .await
                .map_err(|e| ProviderManagerError::HealthCheckFailed {
                    provider: name.clone(),
                    reason: e.to_string(),
                })?;

        let managed_provider = ManagedProvider::new(provider);
        managed_provider.update_health_status(health_status).await;

        // Get performance before moving managed_provider
        let current_performance = managed_provider.get_performance().await;

        let mut providers = self.providers.write().await;
        providers.insert(name.clone(), managed_provider);

        let mut tracker = self.performance_tracker.write().await;
        tracker.insert(name, current_performance);

        info!("Provider added successfully");
        Ok(())
    }

    /// Remove a provider from the manager
    pub async fn remove_provider(&self, name: &str) -> Result<(), ProviderManagerError> {
        info!("Removing provider: {}", name);

        let mut providers = self.providers.write().await;
        providers
            .remove(name)
            .ok_or_else(|| ProviderManagerError::ProviderNotFound(name.to_string()))?;

        let mut tracker = self.performance_tracker.write().await;
        tracker.remove(name);

        info!("Provider removed successfully");
        Ok(())
    }

    /// Get list of available provider names
    pub async fn list_providers(&self) -> Vec<String> {
        let providers = self.providers.read().await;
        providers.keys().cloned().collect()
    }

    /// Select the optimal provider for a research request
    pub async fn select_provider(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<(String, Arc<dyn Provider>), ProviderManagerError> {
        let providers = self.providers.read().await;

        if providers.is_empty() {
            return Err(ProviderManagerError::NoProviders);
        }

        // Filter to healthy providers
        let healthy_providers: Vec<_> = {
            let mut healthy = Vec::new();
            for (name, managed_provider) in providers.iter() {
                let performance = managed_provider.get_performance().await;
                if performance.is_healthy() {
                    healthy.push((name.clone(), managed_provider.provider.clone(), performance));
                }
            }
            healthy
        };

        if healthy_providers.is_empty() {
            // Debug: Let's see what's wrong with health checks
            debug!("Investigating why no healthy providers found:");
            for (name, managed_provider) in providers.iter() {
                let performance = managed_provider.get_performance().await;
                debug!("Provider {}: health_status={:?}, is_healthy()={}, consecutive_failures={}, success_rate={}",
                    name, performance.health_status, performance.is_healthy(),
                    performance.consecutive_failures, performance.success_rate());
            }
            warn!("No healthy providers available, falling back to all providers");
            // If no healthy providers, try any available provider
            let (name, managed_provider) = providers
                .iter()
                .next()
                .ok_or(ProviderManagerError::NoProviders)?;
            return Ok((name.clone(), managed_provider.provider.clone()));
        }

        let selected = match &self.config.selection_strategy {
            SelectionStrategy::RoundRobin => self.select_round_robin(&healthy_providers).await,
            SelectionStrategy::LowestLatency => self.select_lowest_latency(&healthy_providers),
            SelectionStrategy::HighestSuccessRate => {
                self.select_highest_success_rate(&healthy_providers)
            }
            SelectionStrategy::CostOptimized => {
                self.select_cost_optimized(&healthy_providers, request)
                    .await
            }
            SelectionStrategy::ResearchTypeOptimized => {
                self.select_research_type_optimized(&healthy_providers, request)
            }
            SelectionStrategy::Balanced => self.select_balanced(&healthy_providers, request),
        };

        match selected {
            Some((name, provider, _)) => {
                debug!(
                    "Selected provider '{}' using strategy '{:?}'",
                    name, self.config.selection_strategy
                );
                Ok((name, provider))
            }
            None => {
                error!("Provider selection failed");
                Err(ProviderManagerError::SelectionFailed(
                    "No suitable provider found".to_string(),
                ))
            }
        }
    }

    /// Execute research with automatic failover
    pub async fn execute_research(
        &self,
        request: &ClassifiedRequest,
    ) -> ProviderResult<ResearchResult> {
        let start_time = Instant::now();
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.config.max_failover_attempts {
            attempts += 1;

            match self.select_provider(request).await {
                Ok((provider_name, provider)) => {
                    debug!(
                        "Attempting research with provider '{}' (attempt {})",
                        provider_name, attempts
                    );

                    let request_start = Instant::now();
                    match self.execute_with_timeout(&provider, request).await {
                        Ok(response) => {
                            let latency = request_start.elapsed();

                            // Estimate cost and quality (would be more sophisticated in real implementation)
                            let cost_estimate = provider
                                .estimate_cost(&request.original_query)
                                .await
                                .ok()
                                .and_then(|cost| cost.estimated_cost_usd);

                            // Update provider performance
                            let providers = self.providers.read().await;
                            if let Some(managed_provider) = providers.get(&provider_name) {
                                managed_provider
                                    .update_performance(
                                        true,
                                        latency,
                                        cost_estimate,
                                        Some(0.8), // Mock quality score
                                    )
                                    .await;
                            }

                            info!(
                                "Research completed successfully with provider '{}' in {:.2}s",
                                provider_name,
                                latency.as_secs_f64()
                            );

                            // Create research result (simplified for this implementation)
                            let metadata = ResearchMetadata {
                                completed_at: Utc::now(),
                                processing_time_ms: start_time.elapsed().as_millis() as u64,
                                sources_consulted: vec![
                                    provider_name,
                                    "Multi-Provider Research Engine".to_string(),
                                ],
                                quality_score: 0.8, // Mock quality score
                                cache_key: String::new(),
                                tags: HashMap::new(),
                            };

                            let result = ResearchResult::new(
                                request.clone(),
                                response,
                                vec![], // Mock evidence
                                vec![], // Mock details
                                metadata,
                            );

                            return Ok(result);
                        }
                        Err(error) => {
                            let latency = request_start.elapsed();
                            warn!("Provider '{}' failed: {}", provider_name, error);

                            // Update provider performance
                            let providers = self.providers.read().await;
                            if let Some(managed_provider) = providers.get(&provider_name) {
                                managed_provider
                                    .update_performance(false, latency, None, None)
                                    .await;
                            }

                            last_error = Some(error);

                            if !self.config.enable_failover {
                                break;
                            }
                        }
                    }
                }
                Err(manager_error) => {
                    error!("Provider selection failed: {}", manager_error);
                    last_error = Some(ProviderError::ServiceUnavailable {
                        provider: "manager".to_string(),
                        message: manager_error.to_string(),
                        estimated_recovery: Some(Duration::from_secs(60)),
                    });
                    break;
                }
            }
        }

        Err(last_error.unwrap_or(ProviderError::ServiceUnavailable {
            provider: "all".to_string(),
            message: "All failover attempts exhausted".to_string(),
            estimated_recovery: Some(Duration::from_secs(300)),
        }))
    }

    /// Execute provider request with timeout
    async fn execute_with_timeout(
        &self,
        provider: &Arc<dyn Provider>,
        request: &ClassifiedRequest,
    ) -> ProviderResult<String> {
        let timeout_duration = self.config.provider_timeout;

        match tokio::time::timeout(
            timeout_duration,
            provider.research_query(request.original_query.clone()),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => Err(ProviderError::Timeout {
                provider: provider.metadata().name().to_string(),
                duration: timeout_duration,
            }),
        }
    }

    /// Perform health checks on all providers
    pub async fn health_check_all(
        &self,
    ) -> Result<HashMap<String, HealthStatus>, ProviderManagerError> {
        let providers = self.providers.read().await;
        let mut results = HashMap::new();

        for (name, managed_provider) in providers.iter() {
            if managed_provider
                .needs_health_check(self.config.health_check_interval)
                .await
            {
                match managed_provider.provider.health_check().await {
                    Ok(status) => {
                        managed_provider.update_health_status(status.clone()).await;
                        results.insert(name.clone(), status);
                    }
                    Err(e) => {
                        let unhealthy_status = HealthStatus::Unhealthy(e.to_string());
                        managed_provider
                            .update_health_status(unhealthy_status.clone())
                            .await;
                        results.insert(name.clone(), unhealthy_status);
                    }
                }
            } else {
                let performance = managed_provider.get_performance().await;
                results.insert(name.clone(), performance.health_status);
            }
        }

        Ok(results)
    }

    /// Get performance statistics for all providers
    pub async fn get_performance_stats(&self) -> HashMap<String, ProviderPerformance> {
        let providers = self.providers.read().await;
        let mut stats = HashMap::new();

        for (name, managed_provider) in providers.iter() {
            let performance = managed_provider.get_performance().await;
            stats.insert(name.clone(), performance);
        }

        stats
    }

    /// Get all healthy providers for parallel execution
    pub async fn get_healthy_providers(&self) -> Vec<(String, Arc<dyn Provider>)> {
        let providers = self.providers.read().await;
        let mut healthy_providers = Vec::new();

        for (name, managed_provider) in providers.iter() {
            let performance = managed_provider.get_performance().await;
            if performance.is_healthy() {
                healthy_providers.push((name.clone(), managed_provider.provider.clone()));
            }
        }

        healthy_providers
    }

    // Provider selection strategies

    async fn select_round_robin(
        &self,
        providers: &[(String, Arc<dyn Provider>, ProviderPerformance)],
    ) -> Option<(String, Arc<dyn Provider>, ProviderPerformance)> {
        if providers.is_empty() {
            return None;
        }

        let state = self.selection_state.lock().await;
        let index =
            state.round_robin_index.fetch_add(1, Ordering::Relaxed) as usize % providers.len();
        Some(providers[index].clone())
    }

    fn select_lowest_latency(
        &self,
        providers: &[(String, Arc<dyn Provider>, ProviderPerformance)],
    ) -> Option<(String, Arc<dyn Provider>, ProviderPerformance)> {
        providers
            .iter()
            .min_by_key(|(_, _, perf)| perf.average_latency())
            .cloned()
    }

    fn select_highest_success_rate(
        &self,
        providers: &[(String, Arc<dyn Provider>, ProviderPerformance)],
    ) -> Option<(String, Arc<dyn Provider>, ProviderPerformance)> {
        providers
            .iter()
            .max_by(|(_, _, perf_a), (_, _, perf_b)| {
                perf_a
                    .success_rate()
                    .partial_cmp(&perf_b.success_rate())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    async fn select_cost_optimized(
        &self,
        providers: &[(String, Arc<dyn Provider>, ProviderPerformance)],
        request: &ClassifiedRequest,
    ) -> Option<(String, Arc<dyn Provider>, ProviderPerformance)> {
        let mut cost_candidates = Vec::new();

        for (name, provider, performance) in providers {
            if let Ok(cost) = provider.estimate_cost(&request.original_query).await {
                if let Some(cost_usd) = cost.estimated_cost_usd {
                    cost_candidates.push((
                        name.clone(),
                        provider.clone(),
                        performance.clone(),
                        cost_usd,
                    ));
                }
            }
        }

        cost_candidates.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));
        cost_candidates
            .first()
            .map(|(name, provider, perf, _)| (name.clone(), provider.clone(), perf.clone()))
    }

    fn select_research_type_optimized(
        &self,
        providers: &[(String, Arc<dyn Provider>, ProviderPerformance)],
        request: &ClassifiedRequest,
    ) -> Option<(String, Arc<dyn Provider>, ProviderPerformance)> {
        // Different research types prefer different characteristics
        match request.research_type {
            ResearchType::Implementation => {
                // Prefer providers with good code generation capabilities
                self.select_highest_success_rate(providers)
            }
            ResearchType::Learning => {
                // Prefer providers with good explanatory capabilities
                self.select_lowest_latency(providers)
            }
            ResearchType::Troubleshooting => {
                // Prefer providers with good problem-solving capabilities
                self.select_highest_success_rate(providers)
            }
            ResearchType::Decision => {
                // Prefer providers with balanced performance
                self.select_balanced(providers, request)
            }
            ResearchType::Validation => {
                // Prefer providers with high quality scores
                providers
                    .iter()
                    .max_by(|(_, _, perf_a), (_, _, perf_b)| {
                        perf_a
                            .average_quality()
                            .partial_cmp(&perf_b.average_quality())
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .cloned()
            }
        }
    }

    fn select_balanced(
        &self,
        providers: &[(String, Arc<dyn Provider>, ProviderPerformance)],
        _request: &ClassifiedRequest,
    ) -> Option<(String, Arc<dyn Provider>, ProviderPerformance)> {
        providers
            .iter()
            .max_by(|(_, _, perf_a), (_, _, perf_b)| {
                perf_a
                    .health_score()
                    .partial_cmp(&perf_b.health_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    /// Check if the manager overall is healthy (has at least one healthy provider)
    pub async fn is_healthy(&self) -> Result<bool, ProviderManagerError> {
        let providers = self.providers.read().await;

        if providers.is_empty() {
            return Ok(false);
        }

        // Check if any provider is healthy
        for (_, managed_provider) in providers.iter() {
            let performance = managed_provider.get_performance().await;
            if performance.is_healthy() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Simulate provider failure for testing purposes
    pub async fn simulate_provider_failure(
        &mut self,
        provider_name: &str,
    ) -> Result<(), ProviderManagerError> {
        let providers = self.providers.read().await;

        if let Some(managed_provider) = providers.get(provider_name) {
            managed_provider
                .update_health_status(HealthStatus::Unhealthy("Simulated failure".to_string()))
                .await;
            info!("Simulated failure for provider: {}", provider_name);
            Ok(())
        } else {
            Err(ProviderManagerError::ProviderNotFound(
                provider_name.to_string(),
            ))
        }
    }

    /// Execute a query using the provider manager
    pub async fn execute_query(&self, query: &str) -> Result<String, ProviderManagerError> {
        // Create a minimal classified request for the query
        let request = ClassifiedRequest::new(
            query.to_string(),
            ResearchType::Implementation,
            AudienceContext {
                level: "intermediate".to_string(),
                domain: "general".to_string(),
                format: "text".to_string(),
            },
            DomainContext {
                technology: "general".to_string(),
                project_type: "general".to_string(),
                frameworks: vec![],
                tags: vec![],
            },
            0.8,
            vec!["api-query".to_string()],
        );

        // Execute the research using existing infrastructure
        match self.execute_research(&request).await {
            Ok(result) => Ok(result.immediate_answer),
            Err(provider_error) => Err(ProviderManagerError::AllProvidersFailed(
                provider_error.to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{ProviderMetadata, QueryCost, UsageStats};
    use async_trait::async_trait;
    use fortitude_types::{AudienceContext, DomainContext};

    // Mock provider for testing
    #[derive(Debug, Clone)]
    struct TestProvider {
        name: String,
        healthy: bool,
        latency: Duration,
        cost_per_request: f64,
        success_rate: f64,
    }

    impl TestProvider {
        fn new(name: &str, healthy: bool, latency: Duration, cost: f64, success_rate: f64) -> Self {
            Self {
                name: name.to_string(),
                healthy,
                latency,
                cost_per_request: cost,
                success_rate,
            }
        }
    }

    #[async_trait]
    impl Provider for TestProvider {
        async fn research_query(&self, query: String) -> ProviderResult<String> {
            tokio::time::sleep(self.latency).await;

            if rand::random::<f64>() > self.success_rate {
                return Err(ProviderError::QueryFailed {
                    message: "Random failure".to_string(),
                    provider: self.name.clone(),
                    error_code: None,
                });
            }

            Ok(format!("{} response: {}", self.name, query))
        }

        fn metadata(&self) -> ProviderMetadata {
            ProviderMetadata::new(self.name.clone(), "1.0.0".to_string())
        }

        async fn health_check(&self) -> ProviderResult<HealthStatus> {
            Ok(if self.healthy {
                HealthStatus::Healthy
            } else {
                HealthStatus::Unhealthy("Test unhealthy".to_string())
            })
        }

        async fn estimate_cost(&self, _query: &str) -> ProviderResult<QueryCost> {
            Ok(QueryCost {
                estimated_input_tokens: 10,
                estimated_output_tokens: 20,
                estimated_duration: self.latency,
                estimated_cost_usd: Some(self.cost_per_request),
            })
        }

        async fn usage_stats(&self) -> ProviderResult<UsageStats> {
            Ok(UsageStats::default())
        }
    }

    fn create_test_request() -> ClassifiedRequest {
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
    async fn test_provider_manager_creation() {
        let config = ProviderConfig::default();
        let manager = ProviderManager::new(config).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_add_and_list_providers() {
        let config = ProviderConfig::default();
        let manager = ProviderManager::new(config).await.unwrap();

        let provider = Arc::new(TestProvider::new(
            "test-provider",
            true,
            Duration::from_millis(100),
            0.01,
            0.9,
        ));
        let result = manager.add_provider("test".to_string(), provider).await;
        assert!(result.is_ok());

        let providers = manager.list_providers().await;
        assert_eq!(providers.len(), 1);
        assert!(providers.contains(&"test".to_string()));
    }

    #[tokio::test]
    async fn test_provider_selection_strategies() {
        let config = ProviderConfig::default();
        let manager = ProviderManager::new(config).await.unwrap();

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

        manager
            .add_provider("fast".to_string(), fast_provider)
            .await
            .unwrap();
        manager
            .add_provider("slow".to_string(), slow_provider)
            .await
            .unwrap();

        let request = create_test_request();
        let (selected_name, _) = manager.select_provider(&request).await.unwrap();

        // Should select one of the providers
        assert!(selected_name == "fast" || selected_name == "slow");
    }

    #[tokio::test]
    async fn test_health_check_all() {
        let config = ProviderConfig::default();
        let manager = ProviderManager::new(config).await.unwrap();

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
            0.9,
        ));

        manager
            .add_provider("healthy".to_string(), healthy_provider)
            .await
            .unwrap();
        manager
            .add_provider("unhealthy".to_string(), unhealthy_provider)
            .await
            .unwrap();

        let health_results = manager.health_check_all().await.unwrap();

        assert_eq!(health_results.len(), 2);
        assert!(matches!(health_results["healthy"], HealthStatus::Healthy));
        assert!(matches!(
            health_results["unhealthy"],
            HealthStatus::Unhealthy(_)
        ));
    }

    #[tokio::test]
    async fn test_failover_mechanism() {
        let config = ProviderConfig {
            enable_failover: true,
            max_failover_attempts: 2,
            ..Default::default()
        };

        let manager = ProviderManager::new(config).await.unwrap();

        let failing_provider = Arc::new(TestProvider::new(
            "failing",
            true,
            Duration::from_millis(100),
            0.01,
            0.0,
        )); // Always fails
        let working_provider = Arc::new(TestProvider::new(
            "working",
            true,
            Duration::from_millis(100),
            0.01,
            1.0,
        )); // Always works

        manager
            .add_provider("failing".to_string(), failing_provider)
            .await
            .unwrap();
        manager
            .add_provider("working".to_string(), working_provider)
            .await
            .unwrap();

        let request = create_test_request();
        let result = manager.execute_research(&request).await;

        // Should eventually succeed with working provider
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_performance_tracking() {
        let config = ProviderConfig::default();
        let manager = ProviderManager::new(config).await.unwrap();

        let provider = Arc::new(TestProvider::new(
            "tracked",
            true,
            Duration::from_millis(100),
            0.01,
            0.8,
        ));
        manager
            .add_provider("tracked".to_string(), provider)
            .await
            .unwrap();

        let request = create_test_request();

        // Execute a few requests to build performance history
        for _ in 0..3 {
            let _ = manager.execute_research(&request).await;
        }

        let stats = manager.get_performance_stats().await;
        assert!(stats.contains_key("tracked"));

        let provider_stats = &stats["tracked"];
        assert!(provider_stats.total_requests > 0);
    }

    #[test]
    fn test_provider_health_check_fix() {
        // Test case 1: Brand new provider should be healthy
        let provider_perf = ProviderPerformance::default();
        assert!(provider_perf.is_healthy(), "New provider should be healthy");
        assert_eq!(provider_perf.total_requests, 0);
        assert_eq!(provider_perf.success_rate(), 0.0);
        assert_eq!(provider_perf.consecutive_failures, 0);
        assert!(matches!(provider_perf.health_status, HealthStatus::Healthy));

        // Test case 2: Provider with some successful requests should be healthy
        let provider_perf_with_success = ProviderPerformance {
            total_requests: 10,
            successful_requests: 8,
            failed_requests: 2,
            ..Default::default()
        };

        assert!(
            provider_perf_with_success.is_healthy(),
            "Provider with 80% success rate should be healthy"
        );
        assert_eq!(provider_perf_with_success.success_rate(), 0.8);

        // Test case 3: Provider with low success rate should not be healthy
        let provider_perf_low_success = ProviderPerformance {
            total_requests: 10,
            successful_requests: 3,
            failed_requests: 7,
            ..Default::default()
        };

        assert!(
            !provider_perf_low_success.is_healthy(),
            "Provider with 30% success rate should not be healthy"
        );
        assert_eq!(provider_perf_low_success.success_rate(), 0.3);

        // Test case 4: Provider with exactly 50% success rate should not be healthy
        let provider_perf_boundary = ProviderPerformance {
            total_requests: 10,
            successful_requests: 5,
            failed_requests: 5,
            ..Default::default()
        };

        assert!(
            !provider_perf_boundary.is_healthy(),
            "Provider with exactly 50% success rate should not be healthy"
        );
        assert_eq!(provider_perf_boundary.success_rate(), 0.5);

        // Test case 5: Provider with just above 50% success rate should be healthy
        let provider_perf_above_boundary = ProviderPerformance {
            total_requests: 1000,
            successful_requests: 501,
            failed_requests: 499,
            ..Default::default()
        };

        assert!(
            provider_perf_above_boundary.is_healthy(),
            "Provider with 50.1% success rate should be healthy"
        );
        assert_eq!(provider_perf_above_boundary.success_rate(), 0.501);
    }
}
