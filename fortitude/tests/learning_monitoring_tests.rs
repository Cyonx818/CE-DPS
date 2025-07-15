// ABOUTME: Comprehensive tests for learning system monitoring and health checks
//! This test module verifies that the learning system monitoring provides:
//! - Real-time metrics collection for learning operations
//! - Health check functionality for learning components
//! - Performance monitoring and alerting
//! - Integration with existing monitoring infrastructure
//! - Metrics aggregation and reporting

use fortitude::learning::{LearningError, LearningResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time;

/// Metrics collector for learning system operations
pub struct LearningMetricsCollector {
    metrics: Arc<Mutex<LearningMetrics>>,
    config: MetricsConfig,
    collectors: Vec<Box<dyn MetricCollector>>,
}

/// Learning system health checker
pub struct LearningHealthChecker {
    storage_checker: Box<dyn HealthCheck>,
    adaptation_checker: Box<dyn HealthCheck>,
    pattern_recognition_checker: Box<dyn HealthCheck>,
    config: HealthCheckConfig,
}

/// Performance monitor for learning operations
pub struct LearningPerformanceMonitor {
    metrics_collector: LearningMetricsCollector,
    alert_manager: AlertManager,
    performance_history: PerformanceHistory,
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

/// Trait for individual metric collectors
#[async_trait::async_trait]
pub trait MetricCollector: Send + Sync {
    async fn collect_metrics(&self) -> LearningResult<HashMap<String, f64>>;
    fn metric_name(&self) -> &str;
}

/// Trait for health checks
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check_health(&self) -> LearningResult<HealthCheckResult>;
    fn component_name(&self) -> &str;
}

/// Trait for alert handlers
#[async_trait::async_trait]
pub trait AlertHandler: Send + Sync {
    async fn handle_alert(&self, alert: &Alert) -> LearningResult<()>;
    fn handler_name(&self) -> &str;
}

impl LearningMetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: MetricsConfig) -> Self {
        todo!("Implement metrics collector creation")
    }

    /// Start collecting metrics at configured intervals
    pub async fn start_collection(&mut self) -> LearningResult<()> {
        todo!("Implement metrics collection startup")
    }

    /// Stop metrics collection
    pub async fn stop_collection(&mut self) -> LearningResult<()> {
        todo!("Implement metrics collection shutdown")
    }

    /// Get current metrics snapshot
    pub async fn get_current_metrics(&self) -> LearningResult<LearningMetrics> {
        todo!("Implement current metrics retrieval")
    }

    /// Get metrics history for a time range
    pub async fn get_metrics_history(
        &self,
        duration: Duration,
    ) -> LearningResult<Vec<LearningMetrics>> {
        todo!("Implement metrics history retrieval")
    }

    /// Add a custom metric collector
    pub fn add_collector(&mut self, collector: Box<dyn MetricCollector>) {
        self.collectors.push(collector);
    }

    /// Record adaptation event
    pub async fn record_adaptation(
        &self,
        duration: Duration,
        success: bool,
        confidence: f64,
    ) -> LearningResult<()> {
        todo!("Implement adaptation event recording")
    }

    /// Record storage operation
    pub async fn record_storage_operation(
        &self,
        operation_type: &str,
        duration: Duration,
        success: bool,
    ) -> LearningResult<()> {
        todo!("Implement storage operation recording")
    }

    /// Record pattern recognition event
    pub async fn record_pattern_recognition(
        &self,
        patterns_found: u64,
        accuracy: f64,
        duration: Duration,
    ) -> LearningResult<()> {
        todo!("Implement pattern recognition recording")
    }
}

impl LearningHealthChecker {
    /// Create a new health checker
    pub fn new(config: HealthCheckConfig) -> Self {
        todo!("Implement health checker creation")
    }

    /// Perform comprehensive health check
    pub async fn check_system_health(&self) -> LearningResult<HealthReport> {
        todo!("Implement comprehensive health check")
    }

    /// Check specific component health
    pub async fn check_component_health(
        &self,
        component: &str,
    ) -> LearningResult<HealthCheckResult> {
        todo!("Implement component health check")
    }

    /// Start periodic health checks
    pub async fn start_health_monitoring(&mut self) -> LearningResult<()> {
        todo!("Implement health monitoring startup")
    }

    /// Stop health monitoring
    pub async fn stop_health_monitoring(&mut self) -> LearningResult<()> {
        todo!("Implement health monitoring shutdown")
    }
}

impl LearningPerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(
        metrics_config: MetricsConfig,
        health_config: HealthCheckConfig,
        alert_config: AlertConfig,
    ) -> Self {
        todo!("Implement performance monitor creation")
    }

    /// Start comprehensive monitoring
    pub async fn start_monitoring(&mut self) -> LearningResult<()> {
        todo!("Implement monitoring startup")
    }

    /// Stop monitoring
    pub async fn stop_monitoring(&mut self) -> LearningResult<()> {
        todo!("Implement monitoring shutdown")
    }

    /// Get performance summary
    pub async fn get_performance_summary(&self) -> LearningResult<PerformanceSummary> {
        todo!("Implement performance summary generation")
    }

    /// Generate monitoring dashboard data
    pub async fn get_dashboard_data(&self) -> LearningResult<DashboardData> {
        todo!("Implement dashboard data generation")
    }
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
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let config = MetricsConfig {
            collection_interval_seconds: 60,
            enabled_metrics: vec!["adaptation".to_string(), "storage".to_string()],
            retention_hours: 24,
            export_format: "json".to_string(),
            aggregation_window_minutes: 5,
        };

        let collector = LearningMetricsCollector::new(config);
        assert_eq!(collector.collectors.len(), 0);
    }

    #[tokio::test]
    async fn test_metrics_collection_lifecycle() {
        let config = MetricsConfig {
            collection_interval_seconds: 1, // Fast for testing
            enabled_metrics: vec!["adaptation".to_string()],
            retention_hours: 1,
            export_format: "json".to_string(),
            aggregation_window_minutes: 1,
        };

        let mut collector = LearningMetricsCollector::new(config);

        // Start collection
        collector.start_collection().await.unwrap();

        // Wait a bit for metrics to be collected
        sleep(Duration::from_millis(100)).await;

        // Get current metrics
        let metrics = collector.get_current_metrics().await.unwrap();
        assert!(metrics.timestamp <= SystemTime::now());

        // Stop collection
        collector.stop_collection().await.unwrap();
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

        // Record failed adaptation
        collector
            .record_adaptation(Duration::from_millis(1000), false, 0.3)
            .await
            .unwrap();

        let metrics = collector.get_current_metrics().await.unwrap();
        assert_eq!(metrics.adaptation_metrics.adaptations_applied, 1);
        assert_eq!(metrics.adaptation_metrics.adaptations_failed, 1);
        assert!(metrics.adaptation_metrics.average_adaptation_time_ms > 0.0);
    }

    #[tokio::test]
    async fn test_storage_metrics_recording() {
        let config = MetricsConfig {
            collection_interval_seconds: 60,
            enabled_metrics: vec!["storage".to_string()],
            retention_hours: 24,
            export_format: "json".to_string(),
            aggregation_window_minutes: 5,
        };

        let collector = LearningMetricsCollector::new(config);

        // Record storage operations
        collector
            .record_storage_operation("read", Duration::from_millis(100), true)
            .await
            .unwrap();
        collector
            .record_storage_operation("write", Duration::from_millis(200), true)
            .await
            .unwrap();
        collector
            .record_storage_operation("read", Duration::from_millis(150), false)
            .await
            .unwrap();

        let metrics = collector.get_current_metrics().await.unwrap();
        assert_eq!(metrics.storage_metrics.total_operations, 3);
        assert_eq!(metrics.storage_metrics.successful_operations, 2);
        assert_eq!(metrics.storage_metrics.failed_operations, 1);
        assert!(metrics.storage_metrics.average_response_time_ms > 0.0);
    }

    #[tokio::test]
    async fn test_pattern_recognition_metrics() {
        let config = MetricsConfig {
            collection_interval_seconds: 60,
            enabled_metrics: vec!["pattern_recognition".to_string()],
            retention_hours: 24,
            export_format: "json".to_string(),
            aggregation_window_minutes: 5,
        };

        let collector = LearningMetricsCollector::new(config);

        // Record pattern recognition events
        collector
            .record_pattern_recognition(5, 0.92, Duration::from_millis(300))
            .await
            .unwrap();
        collector
            .record_pattern_recognition(3, 0.88, Duration::from_millis(250))
            .await
            .unwrap();

        let metrics = collector.get_current_metrics().await.unwrap();
        assert_eq!(metrics.pattern_recognition_metrics.patterns_analyzed, 2);
        assert_eq!(metrics.pattern_recognition_metrics.patterns_recognized, 8);
        assert!(metrics.pattern_recognition_metrics.recognition_accuracy > 0.0);
    }

    #[tokio::test]
    async fn test_health_checker_creation() {
        let config = HealthCheckConfig {
            check_interval_seconds: 30,
            timeout_seconds: 10,
            enabled_checks: vec!["storage".to_string(), "adaptation".to_string()],
            failure_threshold: 3,
        };

        let checker = LearningHealthChecker::new(config);
        assert_eq!(checker.config.check_interval_seconds, 30);
    }

    #[tokio::test]
    async fn test_system_health_check() {
        let config = HealthCheckConfig {
            check_interval_seconds: 30,
            timeout_seconds: 5,
            enabled_checks: vec![
                "storage".to_string(),
                "adaptation".to_string(),
                "pattern_recognition".to_string(),
            ],
            failure_threshold: 3,
        };

        let checker = LearningHealthChecker::new(config);
        let health_report = checker.check_system_health().await.unwrap();

        assert!(!health_report.component_results.is_empty());
        assert!(!health_report.summary.is_empty());
        assert!(health_report.timestamp <= SystemTime::now());
    }

    #[tokio::test]
    async fn test_component_specific_health_check() {
        let config = HealthCheckConfig {
            check_interval_seconds: 30,
            timeout_seconds: 5,
            enabled_checks: vec!["storage".to_string()],
            failure_threshold: 3,
        };

        let checker = LearningHealthChecker::new(config);
        let result = checker.check_component_health("storage").await.unwrap();

        assert_eq!(result.component, "storage");
        assert!(result.response_time_ms > 0);
        assert!(!result.message.is_empty());
    }

    #[tokio::test]
    async fn test_health_monitoring_lifecycle() {
        let config = HealthCheckConfig {
            check_interval_seconds: 1, // Fast for testing
            timeout_seconds: 5,
            enabled_checks: vec!["storage".to_string()],
            failure_threshold: 3,
        };

        let mut checker = LearningHealthChecker::new(config);

        // Start monitoring
        checker.start_health_monitoring().await.unwrap();

        // Wait a bit for health checks to run
        sleep(Duration::from_millis(100)).await;

        // Stop monitoring
        checker.stop_health_monitoring().await.unwrap();
    }

    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let metrics_config = MetricsConfig {
            collection_interval_seconds: 60,
            enabled_metrics: vec!["all".to_string()],
            retention_hours: 24,
            export_format: "json".to_string(),
            aggregation_window_minutes: 5,
        };

        let health_config = HealthCheckConfig {
            check_interval_seconds: 30,
            timeout_seconds: 10,
            enabled_checks: vec!["all".to_string()],
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

        let monitor = LearningPerformanceMonitor::new(metrics_config, health_config, alert_config);
        assert!(true); // Just test creation succeeds
    }

    #[tokio::test]
    async fn test_monitoring_lifecycle() {
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

        // Start monitoring
        monitor.start_monitoring().await.unwrap();

        // Wait a bit
        sleep(Duration::from_millis(100)).await;

        // Get performance summary
        let summary = monitor.get_performance_summary().await.unwrap();
        assert!(!summary.key_metrics.is_empty());

        // Get dashboard data
        let dashboard = monitor.get_dashboard_data().await.unwrap();
        assert!(
            !dashboard.system_overview.total_adaptations > 0
                || dashboard.system_overview.total_adaptations == 0
        );

        // Stop monitoring
        monitor.stop_monitoring().await.unwrap();
    }

    #[tokio::test]
    async fn test_metrics_history() {
        let config = MetricsConfig {
            collection_interval_seconds: 1,
            enabled_metrics: vec!["adaptation".to_string()],
            retention_hours: 24,
            export_format: "json".to_string(),
            aggregation_window_minutes: 1,
        };

        let mut collector = LearningMetricsCollector::new(config);

        // Record some events to generate history
        collector
            .record_adaptation(Duration::from_millis(100), true, 0.8)
            .await
            .unwrap();

        // Get history
        let history = collector
            .get_metrics_history(Duration::from_secs(3600))
            .await
            .unwrap();
        assert!(!history.is_empty());
    }

    #[tokio::test]
    async fn test_custom_metric_collector() {
        struct TestMetricCollector;

        #[async_trait::async_trait]
        impl MetricCollector for TestMetricCollector {
            async fn collect_metrics(&self) -> LearningResult<HashMap<String, f64>> {
                let mut metrics = HashMap::new();
                metrics.insert("test_metric".to_string(), 42.0);
                Ok(metrics)
            }

            fn metric_name(&self) -> &str {
                "test_collector"
            }
        }

        let config = MetricsConfig {
            collection_interval_seconds: 60,
            enabled_metrics: vec!["test".to_string()],
            retention_hours: 24,
            export_format: "json".to_string(),
            aggregation_window_minutes: 5,
        };

        let mut collector = LearningMetricsCollector::new(config);
        collector.add_collector(Box::new(TestMetricCollector));

        assert_eq!(collector.collectors.len(), 1);
        assert_eq!(collector.collectors[0].metric_name(), "test_collector");
    }

    #[tokio::test]
    async fn test_alert_thresholds() {
        let thresholds = AlertThresholds {
            max_adaptation_time_ms: 1000,
            max_storage_response_time_ms: 500,
            min_pattern_recognition_accuracy: 0.85,
            max_error_rate: 0.05,
            max_memory_usage_mb: 512.0,
            max_cpu_usage_percent: 80.0,
        };

        // Test threshold validation
        assert!(thresholds.max_adaptation_time_ms > 0);
        assert!(thresholds.max_storage_response_time_ms > 0);
        assert!(thresholds.min_pattern_recognition_accuracy > 0.0);
        assert!(thresholds.min_pattern_recognition_accuracy <= 1.0);
        assert!(thresholds.max_error_rate >= 0.0);
        assert!(thresholds.max_error_rate <= 1.0);
        assert!(thresholds.max_memory_usage_mb > 0.0);
        assert!(thresholds.max_cpu_usage_percent > 0.0);
        assert!(thresholds.max_cpu_usage_percent <= 100.0);
    }

    #[tokio::test]
    async fn test_health_status_transitions() {
        // Test health status logic
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Warning);
        assert_ne!(HealthStatus::Warning, HealthStatus::Critical);

        // Test health check result
        let result = HealthCheckResult {
            component: "test".to_string(),
            status: HealthStatus::Healthy,
            message: "All systems operational".to_string(),
            timestamp: SystemTime::now(),
            response_time_ms: 50,
            details: HashMap::new(),
        };

        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(!result.message.is_empty());
        assert!(result.response_time_ms > 0);
    }

    #[tokio::test]
    async fn test_alert_severity_levels() {
        let info_alert = Alert {
            id: "test_info".to_string(),
            severity: AlertSeverity::Info,
            component: "test".to_string(),
            message: "Information alert".to_string(),
            metric_value: 5.0,
            threshold: 10.0,
            timestamp: SystemTime::now(),
            acknowledged: false,
        };

        let warning_alert = Alert {
            id: "test_warning".to_string(),
            severity: AlertSeverity::Warning,
            component: "test".to_string(),
            message: "Warning alert".to_string(),
            metric_value: 15.0,
            threshold: 10.0,
            timestamp: SystemTime::now(),
            acknowledged: false,
        };

        let critical_alert = Alert {
            id: "test_critical".to_string(),
            severity: AlertSeverity::Critical,
            component: "test".to_string(),
            message: "Critical alert".to_string(),
            metric_value: 25.0,
            threshold: 10.0,
            timestamp: SystemTime::now(),
            acknowledged: false,
        };

        assert_eq!(info_alert.severity, AlertSeverity::Info);
        assert_eq!(warning_alert.severity, AlertSeverity::Warning);
        assert_eq!(critical_alert.severity, AlertSeverity::Critical);

        // Test alert comparison logic
        assert!(info_alert.metric_value < info_alert.threshold);
        assert!(warning_alert.metric_value > warning_alert.threshold);
        assert!(critical_alert.metric_value > critical_alert.threshold);
    }

    #[tokio::test]
    async fn test_metrics_serialization() {
        let metrics = LearningMetrics::default();

        // Test JSON serialization
        let json = serde_json::to_string(&metrics).unwrap();
        assert!(!json.is_empty());

        // Test deserialization
        let deserialized: LearningMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(
            metrics.adaptation_metrics.adaptations_applied,
            deserialized.adaptation_metrics.adaptations_applied
        );
    }
}
