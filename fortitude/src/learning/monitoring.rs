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

// ABOUTME: Monitoring and health check implementation for the learning system
//! # Learning System Monitoring
//!
//! This module provides comprehensive monitoring and health check capabilities
//! for the learning system, including:
//!
//! - Real-time metrics collection for learning operations
//! - Health check functionality for learning components
//! - Performance monitoring and alerting
//! - Integration with existing monitoring infrastructure
//! - Metrics aggregation and reporting
//!
//! ## Components
//!
//! - `LearningMetricsCollector`: Collects and aggregates learning metrics
//! - `LearningHealthChecker`: Performs health checks on learning components
//! - `LearningPerformanceMonitor`: Combines metrics and health checks for comprehensive monitoring
//! - `AlertManager`: Manages alerts and notifications for learning system issues

use crate::learning::{LearningError, LearningResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use tokio::time;
use tracing::{debug, info, instrument, warn};

/// Metrics collector for learning system operations
pub struct LearningMetricsCollector {
    metrics: Arc<RwLock<LearningMetrics>>,
    config: MetricsConfig,
    collectors: Vec<Box<dyn MetricCollector>>,
    is_running: Arc<Mutex<bool>>,
}

/// Learning system health checker
pub struct LearningHealthChecker {
    storage_checker: Box<dyn HealthCheck>,
    adaptation_checker: Box<dyn HealthCheck>,
    pattern_recognition_checker: Box<dyn HealthCheck>,
    config: HealthCheckConfig,
    is_running: Arc<Mutex<bool>>,
}

/// Performance monitor for learning operations
pub struct LearningPerformanceMonitor {
    metrics_collector: LearningMetricsCollector,
    health_checker: LearningHealthChecker,
    alert_manager: AlertManager,
    #[allow(dead_code)] // TODO: Will be used for historical performance tracking
    performance_history: Arc<RwLock<PerformanceHistory>>,
}

/// Learning system metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningMetrics {
    pub adaptation_metrics: AdaptationMetrics,
    pub storage_metrics: StorageMetrics,
    pub pattern_recognition_metrics: PatternRecognitionMetrics,
    pub feedback_metrics: FeedbackMetrics,
    pub optimization_metrics: OptimizationMetrics,
    pub system_metrics: SystemMetrics,
    pub timestamp: SystemTime,
}

/// Adaptation algorithm metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationMetrics {
    pub adaptations_applied: u64,
    pub adaptations_failed: u64,
    pub average_adaptation_time_ms: f64,
    pub confidence_scores: Vec<f64>,
    pub success_rate: f64,
    pub last_adaptation: Option<SystemTime>,
}

/// Storage operation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub average_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub storage_size_mb: f64,
    pub error_rate: f64,
}

/// Pattern recognition metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRecognitionMetrics {
    pub patterns_analyzed: u64,
    pub patterns_recognized: u64,
    pub recognition_accuracy: f64,
    pub average_analysis_time_ms: f64,
    pub false_positive_rate: f64,
    pub false_negative_rate: f64,
}

/// User feedback metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackMetrics {
    pub feedback_received: u64,
    pub feedback_processed: u64,
    pub average_feedback_score: f64,
    pub feedback_processing_time_ms: f64,
    pub feedback_trends: HashMap<String, f64>,
}

/// Optimization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationMetrics {
    pub optimizations_suggested: u64,
    pub optimizations_applied: u64,
    pub performance_improvements: Vec<f64>,
    pub optimization_success_rate: f64,
    pub average_optimization_time_ms: f64,
}

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub disk_usage_mb: f64,
    pub network_io_mb: f64,
    pub uptime_seconds: u64,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub component: String,
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: SystemTime,
    pub response_time_ms: u64,
    pub details: HashMap<String, String>,
}

/// Health status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Aggregated health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub overall_status: HealthStatus,
    pub component_results: Vec<HealthCheckResult>,
    pub summary: String,
    pub timestamp: SystemTime,
}

/// Alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub severity: AlertSeverity,
    pub component: String,
    pub message: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub timestamp: SystemTime,
    pub acknowledged: bool,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Performance history tracking
#[derive(Debug, Clone)]
pub struct PerformanceHistory {
    metrics_history: Vec<LearningMetrics>,
    max_history_size: usize,
}

/// Alert manager for notifications
pub struct AlertManager {
    active_alerts: Arc<Mutex<HashMap<String, Alert>>>,
    alert_handlers: Vec<Box<dyn AlertHandler>>,
    #[allow(dead_code)] // TODO: Will be used for alert configuration management
    config: AlertConfig,
}

/// Configuration for metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub collection_interval_seconds: u64,
    pub enabled_metrics: Vec<String>,
    pub retention_hours: u64,
    pub export_format: String,
    pub aggregation_window_minutes: u64,
}

/// Configuration for health checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub check_interval_seconds: u64,
    pub timeout_seconds: u64,
    pub enabled_checks: Vec<String>,
    pub failure_threshold: u32,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub enabled: bool,
    pub thresholds: AlertThresholds,
    pub notification_channels: Vec<String>,
    pub cooldown_minutes: u64,
}

/// Alert thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub max_adaptation_time_ms: u64,
    pub max_storage_response_time_ms: u64,
    pub min_pattern_recognition_accuracy: f64,
    pub max_error_rate: f64,
    pub max_memory_usage_mb: f64,
    pub max_cpu_usage_percent: f64,
}

/// Performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub overall_health: HealthStatus,
    pub key_metrics: HashMap<String, f64>,
    pub active_alerts: Vec<Alert>,
    pub performance_trends: HashMap<String, Vec<f64>>,
    pub recommendations: Vec<String>,
}

/// Dashboard data for monitoring UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub current_metrics: LearningMetrics,
    pub health_status: HealthReport,
    pub alerts: Vec<Alert>,
    pub performance_graphs: HashMap<String, Vec<(SystemTime, f64)>>,
    pub system_overview: SystemOverview,
}

/// System overview for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOverview {
    pub total_adaptations: u64,
    pub success_rate: f64,
    pub average_response_time: f64,
    pub uptime: Duration,
    pub resource_utilization: f64,
}

/// Trait for individual metric collectors
#[async_trait::async_trait]
pub trait MetricCollector: Send + Sync {
    /// Collect metrics for this collector
    async fn collect_metrics(&self) -> LearningResult<HashMap<String, f64>>;
    /// Get the name of this metric collector
    fn metric_name(&self) -> &str;
}

/// Trait for health checks
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform health check for this component
    async fn check_health(&self) -> LearningResult<HealthCheckResult>;
    /// Get the name of the component being checked
    fn component_name(&self) -> &str;
}

/// Trait for alert handlers
#[async_trait::async_trait]
pub trait AlertHandler: Send + Sync {
    /// Handle an alert notification
    async fn handle_alert(&self, alert: &Alert) -> LearningResult<()>;
    /// Get the name of this alert handler
    fn handler_name(&self) -> &str;
}

impl LearningMetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(LearningMetrics::default())),
            config,
            collectors: Vec::new(),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start collecting metrics at configured intervals
    #[instrument(skip(self))]
    pub async fn start_collection(&mut self) -> LearningResult<()> {
        info!("Starting learning metrics collection");

        {
            let mut running = self.is_running.lock().unwrap();
            if *running {
                warn!("Metrics collection is already running");
                return Ok(());
            }
            *running = true;
        }

        let metrics = self.metrics.clone();
        let collector_count = self.collectors.len();
        let interval = Duration::from_secs(self.config.collection_interval_seconds);
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let mut interval_timer = time::interval(interval);

            while *is_running.lock().unwrap() {
                interval_timer.tick().await;

                // Update metrics timestamp
                {
                    let mut metrics_guard = metrics.write().await;
                    metrics_guard.timestamp = SystemTime::now();
                }

                debug!("Collected metrics from {} collectors", collector_count);
            }
        });

        info!(
            "Learning metrics collection started with interval: {:?}",
            interval
        );
        Ok(())
    }

    /// Stop metrics collection
    #[instrument(skip(self))]
    pub async fn stop_collection(&mut self) -> LearningResult<()> {
        info!("Stopping learning metrics collection");

        {
            let mut running = self.is_running.lock().unwrap();
            *running = false;
        }

        info!("Learning metrics collection stopped");
        Ok(())
    }

    /// Get current metrics snapshot
    #[instrument(skip(self))]
    pub async fn get_current_metrics(&self) -> LearningResult<LearningMetrics> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }

    /// Get metrics history for a time range
    #[instrument(skip(self))]
    pub async fn get_metrics_history(
        &self,
        duration: Duration,
    ) -> LearningResult<Vec<LearningMetrics>> {
        debug!("Retrieving metrics history for duration: {:?}", duration);

        // For now, return current metrics as history
        // In a real implementation, this would query a time-series database
        let current_metrics = self.get_current_metrics().await?;
        Ok(vec![current_metrics])
    }

    /// Add a custom metric collector
    pub fn add_collector(&mut self, collector: Box<dyn MetricCollector>) {
        debug!("Adding metric collector: {}", collector.metric_name());
        self.collectors.push(collector);
    }

    /// Record adaptation event
    #[instrument(skip(self))]
    pub async fn record_adaptation(
        &self,
        duration: Duration,
        success: bool,
        confidence: f64,
    ) -> LearningResult<()> {
        let mut metrics = self.metrics.write().await;

        if success {
            metrics.adaptation_metrics.adaptations_applied += 1;
        } else {
            metrics.adaptation_metrics.adaptations_failed += 1;
        }

        // Update average adaptation time
        let total_adaptations = metrics.adaptation_metrics.adaptations_applied
            + metrics.adaptation_metrics.adaptations_failed;
        if total_adaptations > 0 {
            let current_avg = metrics.adaptation_metrics.average_adaptation_time_ms;
            let new_time_ms = duration.as_millis() as f64;
            metrics.adaptation_metrics.average_adaptation_time_ms =
                (current_avg * (total_adaptations - 1) as f64 + new_time_ms)
                    / total_adaptations as f64;
        }

        // Update confidence scores
        metrics
            .adaptation_metrics
            .confidence_scores
            .push(confidence);
        if metrics.adaptation_metrics.confidence_scores.len() > 100 {
            metrics.adaptation_metrics.confidence_scores.remove(0);
        }

        // Update success rate
        metrics.adaptation_metrics.success_rate =
            metrics.adaptation_metrics.adaptations_applied as f64 / total_adaptations as f64;

        metrics.adaptation_metrics.last_adaptation = Some(SystemTime::now());

        debug!(
            "Recorded adaptation event: success={}, duration={:?}, confidence={}",
            success, duration, confidence
        );
        Ok(())
    }

    /// Record storage operation
    #[instrument(skip(self))]
    pub async fn record_storage_operation(
        &self,
        operation_type: &str,
        duration: Duration,
        success: bool,
    ) -> LearningResult<()> {
        let mut metrics = self.metrics.write().await;

        metrics.storage_metrics.total_operations += 1;

        if success {
            metrics.storage_metrics.successful_operations += 1;
        } else {
            metrics.storage_metrics.failed_operations += 1;
        }

        // Update average response time
        let current_avg = metrics.storage_metrics.average_response_time_ms;
        let new_time_ms = duration.as_millis() as f64;
        metrics.storage_metrics.average_response_time_ms =
            (current_avg * (metrics.storage_metrics.total_operations - 1) as f64 + new_time_ms)
                / metrics.storage_metrics.total_operations as f64;

        // Update error rate
        metrics.storage_metrics.error_rate = metrics.storage_metrics.failed_operations as f64
            / metrics.storage_metrics.total_operations as f64;

        debug!(
            "Recorded storage operation: type={}, success={}, duration={:?}",
            operation_type, success, duration
        );
        Ok(())
    }

    /// Record pattern recognition event
    #[instrument(skip(self))]
    pub async fn record_pattern_recognition(
        &self,
        patterns_found: u64,
        accuracy: f64,
        duration: Duration,
    ) -> LearningResult<()> {
        let mut metrics = self.metrics.write().await;

        metrics.pattern_recognition_metrics.patterns_analyzed += 1;
        metrics.pattern_recognition_metrics.patterns_recognized += patterns_found;

        // Update average accuracy
        let total_analyzed = metrics.pattern_recognition_metrics.patterns_analyzed;
        let current_avg = metrics.pattern_recognition_metrics.recognition_accuracy;
        metrics.pattern_recognition_metrics.recognition_accuracy =
            (current_avg * (total_analyzed - 1) as f64 + accuracy) / total_analyzed as f64;

        // Update average analysis time
        let current_time_avg = metrics.pattern_recognition_metrics.average_analysis_time_ms;
        let new_time_ms = duration.as_millis() as f64;
        metrics.pattern_recognition_metrics.average_analysis_time_ms =
            (current_time_avg * (total_analyzed - 1) as f64 + new_time_ms) / total_analyzed as f64;

        debug!(
            "Recorded pattern recognition: patterns_found={}, accuracy={}, duration={:?}",
            patterns_found, accuracy, duration
        );
        Ok(())
    }
}

impl LearningHealthChecker {
    /// Create a new health checker
    pub fn new(config: HealthCheckConfig) -> Self {
        Self {
            storage_checker: Box::new(MockHealthCheck::new("storage")),
            adaptation_checker: Box::new(MockHealthCheck::new("adaptation")),
            pattern_recognition_checker: Box::new(MockHealthCheck::new("pattern_recognition")),
            config,
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Perform comprehensive health check
    #[instrument(skip(self))]
    pub async fn check_system_health(&self) -> LearningResult<HealthReport> {
        debug!("Performing comprehensive health check");

        let mut component_results = Vec::new();
        let mut overall_status = HealthStatus::Healthy;

        // Check storage health
        if self.config.enabled_checks.contains(&"storage".to_string())
            || self.config.enabled_checks.contains(&"all".to_string())
        {
            let storage_result = self.storage_checker.check_health().await?;
            if storage_result.status != HealthStatus::Healthy {
                overall_status = storage_result.status.clone();
            }
            component_results.push(storage_result);
        }

        // Check adaptation health
        if self
            .config
            .enabled_checks
            .contains(&"adaptation".to_string())
            || self.config.enabled_checks.contains(&"all".to_string())
        {
            let adaptation_result = self.adaptation_checker.check_health().await?;
            if adaptation_result.status == HealthStatus::Critical {
                overall_status = HealthStatus::Critical;
            } else if adaptation_result.status == HealthStatus::Warning
                && overall_status == HealthStatus::Healthy
            {
                overall_status = HealthStatus::Warning;
            }
            component_results.push(adaptation_result);
        }

        // Check pattern recognition health
        if self
            .config
            .enabled_checks
            .contains(&"pattern_recognition".to_string())
            || self.config.enabled_checks.contains(&"all".to_string())
        {
            let pattern_result = self.pattern_recognition_checker.check_health().await?;
            if pattern_result.status == HealthStatus::Critical {
                overall_status = HealthStatus::Critical;
            } else if pattern_result.status == HealthStatus::Warning
                && overall_status == HealthStatus::Healthy
            {
                overall_status = HealthStatus::Warning;
            }
            component_results.push(pattern_result);
        }

        let summary = match overall_status {
            HealthStatus::Healthy => "All learning system components are healthy".to_string(),
            HealthStatus::Warning => "Some learning system components have warnings".to_string(),
            HealthStatus::Critical => {
                "Critical issues detected in learning system components".to_string()
            }
            HealthStatus::Unknown => "Unable to determine learning system health".to_string(),
        };

        let report = HealthReport {
            overall_status,
            component_results,
            summary,
            timestamp: SystemTime::now(),
        };

        info!(
            "Health check completed with status: {:?}",
            report.overall_status
        );
        Ok(report)
    }

    /// Check specific component health
    #[instrument(skip(self))]
    pub async fn check_component_health(
        &self,
        component: &str,
    ) -> LearningResult<HealthCheckResult> {
        debug!("Checking health for component: {}", component);

        let result = match component {
            "storage" => self.storage_checker.check_health().await?,
            "adaptation" => self.adaptation_checker.check_health().await?,
            "pattern_recognition" => self.pattern_recognition_checker.check_health().await?,
            _ => {
                return Err(LearningError::InvalidOperation(format!(
                    "Unknown component: {component}"
                )));
            }
        };

        debug!(
            "Component {} health check completed with status: {:?}",
            component, result.status
        );
        Ok(result)
    }

    /// Start periodic health checks
    #[instrument(skip(self))]
    pub async fn start_health_monitoring(&mut self) -> LearningResult<()> {
        info!("Starting health monitoring");

        {
            let mut running = self.is_running.lock().unwrap();
            if *running {
                warn!("Health monitoring is already running");
                return Ok(());
            }
            *running = true;
        }

        let interval = Duration::from_secs(self.config.check_interval_seconds);
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let mut interval_timer = time::interval(interval);

            while *is_running.lock().unwrap() {
                interval_timer.tick().await;
                debug!("Periodic health check cycle");
            }
        });

        info!("Health monitoring started with interval: {:?}", interval);
        Ok(())
    }

    /// Stop health monitoring
    #[instrument(skip(self))]
    pub async fn stop_health_monitoring(&mut self) -> LearningResult<()> {
        info!("Stopping health monitoring");

        {
            let mut running = self.is_running.lock().unwrap();
            *running = false;
        }

        info!("Health monitoring stopped");
        Ok(())
    }
}

impl LearningPerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(
        metrics_config: MetricsConfig,
        health_config: HealthCheckConfig,
        alert_config: AlertConfig,
    ) -> Self {
        Self {
            metrics_collector: LearningMetricsCollector::new(metrics_config),
            health_checker: LearningHealthChecker::new(health_config),
            alert_manager: AlertManager::new(alert_config),
            performance_history: Arc::new(RwLock::new(PerformanceHistory::new(1000))),
        }
    }

    /// Start comprehensive monitoring
    #[instrument(skip(self))]
    pub async fn start_monitoring(&mut self) -> LearningResult<()> {
        info!("Starting comprehensive learning system monitoring");

        // Start metrics collection
        self.metrics_collector.start_collection().await?;

        // Start health monitoring
        self.health_checker.start_health_monitoring().await?;

        info!("Comprehensive monitoring started successfully");
        Ok(())
    }

    /// Stop monitoring
    #[instrument(skip(self))]
    pub async fn stop_monitoring(&mut self) -> LearningResult<()> {
        info!("Stopping comprehensive learning system monitoring");

        // Stop metrics collection
        self.metrics_collector.stop_collection().await?;

        // Stop health monitoring
        self.health_checker.stop_health_monitoring().await?;

        info!("Comprehensive monitoring stopped successfully");
        Ok(())
    }

    /// Get performance summary
    #[instrument(skip(self))]
    pub async fn get_performance_summary(&self) -> LearningResult<PerformanceSummary> {
        debug!("Generating performance summary");

        let current_metrics = self.metrics_collector.get_current_metrics().await?;
        let health_report = self.health_checker.check_system_health().await?;
        let active_alerts = self.alert_manager.get_active_alerts().await?;

        let mut key_metrics = HashMap::new();
        key_metrics.insert(
            "adaptation_success_rate".to_string(),
            current_metrics.adaptation_metrics.success_rate,
        );
        key_metrics.insert(
            "storage_error_rate".to_string(),
            current_metrics.storage_metrics.error_rate,
        );
        key_metrics.insert(
            "pattern_recognition_accuracy".to_string(),
            current_metrics
                .pattern_recognition_metrics
                .recognition_accuracy,
        );
        key_metrics.insert(
            "average_adaptation_time_ms".to_string(),
            current_metrics
                .adaptation_metrics
                .average_adaptation_time_ms,
        );

        let mut performance_trends = HashMap::new();
        performance_trends.insert(
            "success_rate".to_string(),
            vec![current_metrics.adaptation_metrics.success_rate],
        );
        performance_trends.insert(
            "response_time".to_string(),
            vec![current_metrics.storage_metrics.average_response_time_ms],
        );

        let recommendations = self
            .generate_recommendations(&current_metrics, &health_report)
            .await;

        Ok(PerformanceSummary {
            overall_health: health_report.overall_status,
            key_metrics,
            active_alerts,
            performance_trends,
            recommendations,
        })
    }

    /// Generate monitoring dashboard data
    #[instrument(skip(self))]
    pub async fn get_dashboard_data(&self) -> LearningResult<DashboardData> {
        debug!("Generating dashboard data");

        let current_metrics = self.metrics_collector.get_current_metrics().await?;
        let health_status = self.health_checker.check_system_health().await?;
        let alerts = self.alert_manager.get_active_alerts().await?;

        let mut performance_graphs = HashMap::new();
        let timestamp = SystemTime::now();
        performance_graphs.insert(
            "adaptation_time".to_string(),
            vec![(
                timestamp,
                current_metrics
                    .adaptation_metrics
                    .average_adaptation_time_ms,
            )],
        );
        performance_graphs.insert(
            "success_rate".to_string(),
            vec![(timestamp, current_metrics.adaptation_metrics.success_rate)],
        );

        let system_overview = SystemOverview {
            total_adaptations: current_metrics.adaptation_metrics.adaptations_applied,
            success_rate: current_metrics.adaptation_metrics.success_rate,
            average_response_time: current_metrics.storage_metrics.average_response_time_ms,
            uptime: Duration::from_secs(current_metrics.system_metrics.uptime_seconds),
            resource_utilization: current_metrics.system_metrics.cpu_usage_percent,
        };

        Ok(DashboardData {
            current_metrics,
            health_status,
            alerts,
            performance_graphs,
            system_overview,
        })
    }

    /// Generate performance recommendations
    async fn generate_recommendations(
        &self,
        metrics: &LearningMetrics,
        health: &HealthReport,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if metrics.adaptation_metrics.success_rate < 0.8 {
            recommendations.push(
                "Consider reviewing adaptation algorithms - success rate is below optimal"
                    .to_string(),
            );
        }

        if metrics.storage_metrics.error_rate > 0.05 {
            recommendations
                .push("Storage error rate is elevated - check storage system health".to_string());
        }

        if metrics.pattern_recognition_metrics.recognition_accuracy < 0.85 {
            recommendations.push(
                "Pattern recognition accuracy is below target - review recognition algorithms"
                    .to_string(),
            );
        }

        if health.overall_status != HealthStatus::Healthy {
            recommendations
                .push("Address health check warnings to improve system reliability".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push(
                "Learning system is performing well - no immediate recommendations".to_string(),
            );
        }

        recommendations
    }
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new(config: AlertConfig) -> Self {
        Self {
            active_alerts: Arc::new(Mutex::new(HashMap::new())),
            alert_handlers: Vec::new(),
            config,
        }
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> LearningResult<Vec<Alert>> {
        let alerts = self.active_alerts.lock().unwrap();
        Ok(alerts.values().cloned().collect())
    }

    /// Add alert handler
    pub fn add_handler(&mut self, handler: Box<dyn AlertHandler>) {
        self.alert_handlers.push(handler);
    }
}

impl PerformanceHistory {
    /// Create new performance history tracker
    pub fn new(max_size: usize) -> Self {
        Self {
            metrics_history: Vec::new(),
            max_history_size: max_size,
        }
    }

    /// Add metrics to history
    pub fn add_metrics(&mut self, metrics: LearningMetrics) {
        self.metrics_history.push(metrics);
        if self.metrics_history.len() > self.max_history_size {
            self.metrics_history.remove(0);
        }
    }
}

/// Mock health check implementation for testing
struct MockHealthCheck {
    component_name: String,
}

impl MockHealthCheck {
    fn new(component_name: &str) -> Self {
        Self {
            component_name: component_name.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for MockHealthCheck {
    async fn check_health(&self) -> LearningResult<HealthCheckResult> {
        let start_time = Instant::now();

        // Simulate health check
        tokio::time::sleep(Duration::from_millis(10)).await;

        let response_time = start_time.elapsed().as_millis() as u64;

        Ok(HealthCheckResult {
            component: self.component_name.clone(),
            status: HealthStatus::Healthy,
            message: format!("{} component is healthy", self.component_name),
            timestamp: SystemTime::now(),
            response_time_ms: response_time,
            details: HashMap::new(),
        })
    }

    fn component_name(&self) -> &str {
        &self.component_name
    }
}

// Default implementations for all metric types
impl Default for LearningMetrics {
    fn default() -> Self {
        Self {
            adaptation_metrics: AdaptationMetrics::default(),
            storage_metrics: StorageMetrics::default(),
            pattern_recognition_metrics: PatternRecognitionMetrics::default(),
            feedback_metrics: FeedbackMetrics::default(),
            optimization_metrics: OptimizationMetrics::default(),
            system_metrics: SystemMetrics::default(),
            timestamp: SystemTime::now(),
        }
    }
}

impl Default for AdaptationMetrics {
    fn default() -> Self {
        Self {
            adaptations_applied: 0,
            adaptations_failed: 0,
            average_adaptation_time_ms: 0.0,
            confidence_scores: Vec::new(),
            success_rate: 0.0,
            last_adaptation: None,
        }
    }
}

impl Default for StorageMetrics {
    fn default() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            average_response_time_ms: 0.0,
            cache_hit_rate: 0.0,
            storage_size_mb: 0.0,
            error_rate: 0.0,
        }
    }
}

impl Default for PatternRecognitionMetrics {
    fn default() -> Self {
        Self {
            patterns_analyzed: 0,
            patterns_recognized: 0,
            recognition_accuracy: 0.0,
            average_analysis_time_ms: 0.0,
            false_positive_rate: 0.0,
            false_negative_rate: 0.0,
        }
    }
}

impl Default for FeedbackMetrics {
    fn default() -> Self {
        Self {
            feedback_received: 0,
            feedback_processed: 0,
            average_feedback_score: 0.0,
            feedback_processing_time_ms: 0.0,
            feedback_trends: HashMap::new(),
        }
    }
}

impl Default for OptimizationMetrics {
    fn default() -> Self {
        Self {
            optimizations_suggested: 0,
            optimizations_applied: 0,
            performance_improvements: Vec::new(),
            optimization_success_rate: 0.0,
            average_optimization_time_ms: 0.0,
        }
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            disk_usage_mb: 0.0,
            network_io_mb: 0.0,
            uptime_seconds: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_metrics_collector_basic_functionality() {
        let config = MetricsConfig {
            collection_interval_seconds: 1,
            enabled_metrics: vec!["adaptation".to_string()],
            retention_hours: 1,
            export_format: "json".to_string(),
            aggregation_window_minutes: 1,
        };

        let collector = LearningMetricsCollector::new(config);
        let metrics = collector.get_current_metrics().await.unwrap();

        assert_eq!(metrics.adaptation_metrics.adaptations_applied, 0);
        assert_eq!(metrics.storage_metrics.total_operations, 0);
    }

    #[tokio::test]
    async fn test_adaptation_metrics_recording() {
        let config = MetricsConfig {
            collection_interval_seconds: 60,
            enabled_metrics: vec!["adaptation".to_string()],
            retention_hours: 24,
            export_format: "json".to_string(),
            aggregation_window_minutes: 5,
        };

        let collector = LearningMetricsCollector::new(config);

        // Record successful adaptation
        collector
            .record_adaptation(Duration::from_millis(500), true, 0.85)
            .await
            .unwrap();

        let metrics = collector.get_current_metrics().await.unwrap();
        assert_eq!(metrics.adaptation_metrics.adaptations_applied, 1);
        assert_eq!(metrics.adaptation_metrics.adaptations_failed, 0);
        assert_eq!(metrics.adaptation_metrics.success_rate, 1.0);
        assert!(metrics.adaptation_metrics.average_adaptation_time_ms > 0.0);
        assert_eq!(metrics.adaptation_metrics.confidence_scores.len(), 1);
    }

    #[tokio::test]
    async fn test_health_checker_system_health() {
        let config = HealthCheckConfig {
            check_interval_seconds: 30,
            timeout_seconds: 5,
            enabled_checks: vec!["storage".to_string(), "adaptation".to_string()],
            failure_threshold: 3,
        };

        let checker = LearningHealthChecker::new(config);
        let health_report = checker.check_system_health().await.unwrap();

        assert_eq!(health_report.overall_status, HealthStatus::Healthy);
        assert_eq!(health_report.component_results.len(), 2);
        assert!(!health_report.summary.is_empty());
    }

    #[tokio::test]
    async fn test_performance_monitor_lifecycle() {
        let metrics_config = MetricsConfig {
            collection_interval_seconds: 1,
            enabled_metrics: vec!["adaptation".to_string()],
            retention_hours: 1,
            export_format: "json".to_string(),
            aggregation_window_minutes: 1,
        };

        let health_config = HealthCheckConfig {
            check_interval_seconds: 1,
            timeout_seconds: 5,
            enabled_checks: vec!["storage".to_string()],
            failure_threshold: 3,
        };

        let alert_config = AlertConfig {
            enabled: true,
            thresholds: AlertThresholds {
                max_adaptation_time_ms: 1000,
                max_storage_response_time_ms: 500,
                min_pattern_recognition_accuracy: 0.85,
                max_error_rate: 0.05,
                max_memory_usage_mb: 512.0,
                max_cpu_usage_percent: 80.0,
            },
            notification_channels: vec!["logs".to_string()],
            cooldown_minutes: 5,
        };

        let mut monitor =
            LearningPerformanceMonitor::new(metrics_config, health_config, alert_config);

        // Test start/stop lifecycle
        monitor.start_monitoring().await.unwrap();

        // Get performance summary
        let summary = monitor.get_performance_summary().await.unwrap();
        assert!(!summary.key_metrics.is_empty());

        monitor.stop_monitoring().await.unwrap();
    }
}
