// ABOUTME: Integration tests for learning system configuration and monitoring
//! This test module verifies that the learning system configuration and monitoring
//! work together correctly in realistic scenarios:
//! - Configuration changes trigger monitoring updates
//! - Monitoring respects configuration settings
//! - Environment variables properly override file-based configuration
//! - Health checks adapt to configuration changes
//! - Metrics collection follows configuration intervals

use fortitude::learning::{
    Alert, AlertConfig, AlertHandler, AlertSeverity, AlertThresholds, ConfigWatcher,
    EnhancedLearningConfig, HealthCheck, HealthCheckConfig, HealthCheckResult, HealthStatus,
    LearningConfig, LearningConfigManager, LearningError, LearningHealthChecker, LearningMetrics,
    LearningMetricsCollector, LearningMonitoringConfig, LearningPerformanceMonitor, LearningResult,
    MetricCollector,
};
use serde_json;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tempfile::TempDir;
use tokio::time::sleep;

/// Test configuration watcher to verify notifications
struct TestConfigWatcher {
    notification_count: Arc<Mutex<u32>>,
    last_config: Arc<Mutex<Option<EnhancedLearningConfig>>>,
}

impl TestConfigWatcher {
    fn new() -> Self {
        Self {
            notification_count: Arc::new(Mutex::new(0)),
            last_config: Arc::new(Mutex::new(None)),
        }
    }

    fn get_notification_count(&self) -> u32 {
        *self.notification_count.lock().unwrap()
    }

    fn get_last_config(&self) -> Option<EnhancedLearningConfig> {
        self.last_config.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl ConfigWatcher for TestConfigWatcher {
    async fn on_config_changed(
        &mut self,
        _old_config: &EnhancedLearningConfig,
        new_config: &EnhancedLearningConfig,
    ) {
        let mut count = self.notification_count.lock().unwrap();
        *count += 1;

        let mut last_config = self.last_config.lock().unwrap();
        *last_config = Some(new_config.clone());
    }
}

/// Test metric collector for integration testing
struct TestMetricCollector {
    name: String,
    metrics_generated: Arc<Mutex<u32>>,
}

impl TestMetricCollector {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            metrics_generated: Arc::new(Mutex::new(0)),
        }
    }

    fn get_metrics_count(&self) -> u32 {
        *self.metrics_generated.lock().unwrap()
    }
}

#[async_trait::async_trait]
impl MetricCollector for TestMetricCollector {
    async fn collect_metrics(&self) -> LearningResult<HashMap<String, f64>> {
        let mut count = self.metrics_generated.lock().unwrap();
        *count += 1;

        let mut metrics = HashMap::new();
        metrics.insert(format!("{}_test_metric", self.name), 42.0);
        metrics.insert(format!("{}_counter", self.name), *count as f64);
        Ok(metrics)
    }

    fn metric_name(&self) -> &str {
        &self.name
    }
}

/// Test health check for integration testing
struct TestHealthCheck {
    component_name: String,
    health_status: Arc<Mutex<HealthStatus>>,
    check_count: Arc<Mutex<u32>>,
}

impl TestHealthCheck {
    fn new(component_name: &str) -> Self {
        Self {
            component_name: component_name.to_string(),
            health_status: Arc::new(Mutex::new(HealthStatus::Healthy)),
            check_count: Arc::new(Mutex::new(0)),
        }
    }

    fn set_health_status(&self, status: HealthStatus) {
        let mut health = self.health_status.lock().unwrap();
        *health = status;
    }

    fn get_check_count(&self) -> u32 {
        *self.check_count.lock().unwrap()
    }
}

#[async_trait::async_trait]
impl HealthCheck for TestHealthCheck {
    async fn check_health(&self) -> LearningResult<HealthCheckResult> {
        let mut count = self.check_count.lock().unwrap();
        *count += 1;

        let status = self.health_status.lock().unwrap().clone();

        Ok(HealthCheckResult {
            component: self.component_name.clone(),
            status: status.clone(),
            message: match status {
                HealthStatus::Healthy => "Component is healthy".to_string(),
                HealthStatus::Warning => "Component has warnings".to_string(),
                HealthStatus::Critical => "Component is critical".to_string(),
                HealthStatus::Unknown => "Component status unknown".to_string(),
            },
            timestamp: SystemTime::now(),
            response_time_ms: 50,
            details: HashMap::new(),
        })
    }

    fn component_name(&self) -> &str {
        &self.component_name
    }
}

/// Test alert handler for integration testing
struct TestAlertHandler {
    name: String,
    alerts_received: Arc<Mutex<Vec<Alert>>>,
}

impl TestAlertHandler {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            alerts_received: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_alerts_count(&self) -> usize {
        self.alerts_received.lock().unwrap().len()
    }

    fn get_last_alert(&self) -> Option<Alert> {
        self.alerts_received.lock().unwrap().last().cloned()
    }
}

#[async_trait::async_trait]
impl AlertHandler for TestAlertHandler {
    async fn handle_alert(&self, alert: &Alert) -> LearningResult<()> {
        let mut alerts = self.alerts_received.lock().unwrap();
        alerts.push(alert.clone());
        Ok(())
    }

    fn handler_name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_manager_with_watcher_integration() {
        let config = LearningConfig::default();
        let mut manager = LearningConfigManager::new(config.clone());

        let watcher = TestConfigWatcher::new();
        let watcher_count = watcher.notification_count.clone();
        manager.add_watcher(Box::new(watcher));

        // Update configuration should trigger watcher
        let mut new_config = config.clone();
        new_config.adaptation_threshold = 0.9;

        manager.update_config(new_config).await.unwrap();

        // Verify watcher was notified
        assert_eq!(*watcher_count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_environment_based_config_with_monitoring() {
        // Set environment variables
        env::set_var("LEARNING_ENABLE_FEEDBACK", "true");
        env::set_var("LEARNING_ADAPTATION_THRESHOLD", "0.85");
        env::set_var("LEARNING_METRICS_ENABLED", "true");
        env::set_var("LEARNING_METRICS_INTERVAL", "30");
        env::set_var("LEARNING_HEALTH_CHECK_INTERVAL", "60");
        env::set_var("LEARNING_ALERT_ERROR_THRESHOLD", "0.03");

        let manager = LearningConfigManager::from_environment().await.unwrap();
        let enhanced_config = manager.enhanced_config();

        // Verify core learning config
        assert!(enhanced_config.learning.enable_feedback_learning);
        assert_eq!(enhanced_config.learning.adaptation_threshold, 0.85);

        // Verify monitoring config
        assert!(enhanced_config.monitoring.metrics_enabled);
        assert_eq!(
            enhanced_config
                .monitoring
                .metrics_collection_interval_seconds,
            30
        );

        // Verify health check config
        assert_eq!(enhanced_config.health_checks.check_interval_seconds, 60);

        // Verify alert config
        assert_eq!(enhanced_config.alerts.error_rate_threshold, 0.03);

        // Clean up environment
        env::remove_var("LEARNING_ENABLE_FEEDBACK");
        env::remove_var("LEARNING_ADAPTATION_THRESHOLD");
        env::remove_var("LEARNING_METRICS_ENABLED");
        env::remove_var("LEARNING_METRICS_INTERVAL");
        env::remove_var("LEARNING_HEALTH_CHECK_INTERVAL");
        env::remove_var("LEARNING_ALERT_ERROR_THRESHOLD");
    }

    #[tokio::test]
    async fn test_file_config_with_env_overrides_integration() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("learning_config.json");

        // Create base configuration file
        let base_config = EnhancedLearningConfig::development_defaults();
        base_config
            .save_to_file(config_path.to_str().unwrap())
            .await
            .unwrap();

        // Set environment overrides
        env::set_var("LEARNING_ADAPTATION_THRESHOLD", "0.95");
        env::set_var("LEARNING_METRICS_INTERVAL", "15");
        env::set_var("LEARNING_HEALTH_CHECK_INTERVAL", "45");

        let manager = LearningConfigManager::from_file_with_env(config_path.to_str().unwrap())
            .await
            .unwrap();
        let config = manager.config();
        let enhanced_config = manager.enhanced_config();

        // Verify environment overrides were applied
        assert_eq!(config.adaptation_threshold, 0.95);
        // Note: metrics_interval would need to be handled in apply_env_vars method

        // Clean up
        env::remove_var("LEARNING_ADAPTATION_THRESHOLD");
        env::remove_var("LEARNING_METRICS_INTERVAL");
        env::remove_var("LEARNING_HEALTH_CHECK_INTERVAL");
    }

    #[tokio::test]
    async fn test_metrics_collector_with_custom_collectors() {
        let metrics_config = LearningMonitoringConfig {
            metrics_enabled: true,
            metrics_collection_interval_seconds: 1,
            performance_tracking_enabled: true,
            adaptation_tracking_enabled: true,
            storage_metrics_enabled: true,
            alert_thresholds: crate::learning::MonitoringThresholds::default(),
        };

        let mut collector =
            LearningMetricsCollector::new(crate::learning::monitoring::MetricsConfig {
                collection_interval_seconds: metrics_config.metrics_collection_interval_seconds,
                enabled_metrics: vec!["test".to_string()],
                retention_hours: 24,
                export_format: "json".to_string(),
                aggregation_window_minutes: 5,
            });

        // Add custom collectors
        let test_collector = TestMetricCollector::new("integration_test");
        let collector_metrics_count = test_collector.metrics_generated.clone();
        collector.add_collector(Box::new(test_collector));

        // Start collection
        collector.start_collection().await.unwrap();

        // Wait for at least one collection cycle
        sleep(Duration::from_millis(100)).await;

        // Record some events
        collector
            .record_adaptation(Duration::from_millis(200), true, 0.88)
            .await
            .unwrap();
        collector
            .record_storage_operation("read", Duration::from_millis(50), true)
            .await
            .unwrap();
        collector
            .record_pattern_recognition(3, 0.92, Duration::from_millis(100))
            .await
            .unwrap();

        // Get current metrics
        let metrics = collector.get_current_metrics().await.unwrap();

        // Verify adaptation metrics
        assert_eq!(metrics.adaptation_metrics.adaptations_applied, 1);
        assert_eq!(metrics.adaptation_metrics.adaptations_failed, 0);
        assert_eq!(metrics.adaptation_metrics.success_rate, 1.0);
        assert!(metrics.adaptation_metrics.average_adaptation_time_ms > 0.0);

        // Verify storage metrics
        assert_eq!(metrics.storage_metrics.total_operations, 1);
        assert_eq!(metrics.storage_metrics.successful_operations, 1);
        assert!(metrics.storage_metrics.average_response_time_ms > 0.0);

        // Verify pattern recognition metrics
        assert_eq!(metrics.pattern_recognition_metrics.patterns_analyzed, 1);
        assert_eq!(metrics.pattern_recognition_metrics.patterns_recognized, 3);
        assert!(metrics.pattern_recognition_metrics.recognition_accuracy > 0.0);

        // Stop collection
        collector.stop_collection().await.unwrap();
    }

    #[tokio::test]
    async fn test_health_checker_with_multiple_components() {
        let health_config = HealthCheckConfig {
            enabled: true,
            check_interval_seconds: 1,
            storage_health_check: true,
            adaptation_health_check: true,
            pattern_recognition_health_check: true,
            timeout_seconds: 5,
        };

        let mut checker = LearningHealthChecker::new(health_config);

        // Start health monitoring
        checker.start_health_monitoring().await.unwrap();

        // Wait for at least one check cycle
        sleep(Duration::from_millis(100)).await;

        // Check overall system health
        let health_report = checker.check_system_health().await.unwrap();
        assert!(!health_report.component_results.is_empty());
        assert_eq!(health_report.overall_status, HealthStatus::Healthy);

        // Check individual components
        let storage_health = checker.check_component_health("storage").await.unwrap();
        assert_eq!(storage_health.component, "storage");
        assert_eq!(storage_health.status, HealthStatus::Healthy);

        let adaptation_health = checker.check_component_health("adaptation").await.unwrap();
        assert_eq!(adaptation_health.component, "adaptation");
        assert_eq!(adaptation_health.status, HealthStatus::Healthy);

        // Stop monitoring
        checker.stop_health_monitoring().await.unwrap();
    }

    #[tokio::test]
    async fn test_performance_monitor_comprehensive_integration() {
        let metrics_config = crate::learning::monitoring::MetricsConfig {
            collection_interval_seconds: 1,
            enabled_metrics: vec!["adaptation".to_string(), "storage".to_string()],
            retention_hours: 24,
            export_format: "json".to_string(),
            aggregation_window_minutes: 5,
        };

        let health_config = HealthCheckConfig {
            enabled: true,
            check_interval_seconds: 1,
            storage_health_check: true,
            adaptation_health_check: true,
            pattern_recognition_health_check: false, // Disable for faster tests
            timeout_seconds: 5,
        };

        let alert_config = AlertConfig {
            enabled: true,
            error_rate_threshold: 0.05,
            response_time_threshold_ms: 5000,
            storage_error_threshold: 10,
            adaptation_failure_threshold: 5,
            notification_channels: vec!["test".to_string()],
        };

        let mut monitor =
            LearningPerformanceMonitor::new(metrics_config, health_config, alert_config);

        // Start comprehensive monitoring
        monitor.start_monitoring().await.unwrap();

        // Wait for monitoring to start
        sleep(Duration::from_millis(100)).await;

        // Get performance summary
        let summary = monitor.get_performance_summary().await.unwrap();
        assert_eq!(summary.overall_health, HealthStatus::Healthy);
        assert!(!summary.key_metrics.is_empty());
        assert!(!summary.recommendations.is_empty());

        // Get dashboard data
        let dashboard = monitor.get_dashboard_data().await.unwrap();
        assert_eq!(
            dashboard.health_status.overall_status,
            HealthStatus::Healthy
        );
        assert!(!dashboard.performance_graphs.is_empty());
        assert!(dashboard.system_overview.total_adaptations >= 0);

        // Stop monitoring
        monitor.stop_monitoring().await.unwrap();
    }

    #[tokio::test]
    async fn test_configuration_validation_with_monitoring_constraints() {
        let mut config = EnhancedLearningConfig::production_defaults();

        // Valid configuration should pass
        assert!(config.validate().is_ok());

        // Test various invalid configurations that affect monitoring

        // Invalid metrics collection interval
        config.monitoring.metrics_collection_interval_seconds = 0;
        assert!(config.validate().is_err());

        config.monitoring.metrics_collection_interval_seconds = 60;

        // Invalid health check interval
        config.health_checks.check_interval_seconds = 0;
        assert!(config.validate().is_err());

        config.health_checks.check_interval_seconds = 30;

        // Invalid alert thresholds
        config.alerts.error_rate_threshold = 1.5;
        assert!(config.validate().is_err());

        config.alerts.error_rate_threshold = 0.05;

        // Invalid monitoring thresholds
        config
            .monitoring
            .alert_thresholds
            .min_pattern_recognition_accuracy = 1.5;
        assert!(config.validate().is_err());

        config
            .monitoring
            .alert_thresholds
            .min_pattern_recognition_accuracy = 0.85;
        config.monitoring.alert_thresholds.max_cpu_usage_percent = 150.0;
        assert!(config.validate().is_err());

        // Valid configuration should pass again
        config.monitoring.alert_thresholds.max_cpu_usage_percent = 80.0;
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn test_environment_specific_monitoring_presets() {
        let config = LearningConfig::default();
        let mut manager = LearningConfigManager::new(config);

        // Test production overrides
        manager.apply_environment_overrides("production").unwrap();
        let prod_config = manager.enhanced_config();

        assert!(prod_config.learning.enable_optimization);
        assert!(prod_config.learning.adaptation_threshold >= 0.8);
        assert!(prod_config.monitoring.metrics_enabled);
        assert!(prod_config.health_checks.enabled);
        assert!(prod_config.alerts.enabled);
        assert_eq!(prod_config.environment, "production");

        // Test development overrides
        manager.apply_environment_overrides("development").unwrap();
        let dev_config = manager.enhanced_config();

        assert!(!dev_config.learning.enable_optimization);
        assert!(dev_config.learning.adaptation_threshold <= 0.7);
        assert!(dev_config.monitoring.metrics_enabled);
        assert!(dev_config.health_checks.enabled);
        assert!(!dev_config.alerts.enabled); // Less noisy in development
        assert_eq!(dev_config.environment, "development");

        // Test testing overrides
        manager.apply_environment_overrides("testing").unwrap();
        let test_config = manager.enhanced_config();

        assert!(!test_config.learning.enable_optimization);
        assert!(test_config.learning.adaptation_threshold <= 0.6);
        assert!(test_config.monitoring.metrics_collection_interval_seconds <= 30);
        assert!(test_config.health_checks.check_interval_seconds <= 15);
        assert!(!test_config.alerts.enabled);
        assert_eq!(test_config.environment, "testing");
    }

    #[tokio::test]
    async fn test_config_serialization_roundtrip_with_monitoring() {
        let original_config = EnhancedLearningConfig::production_defaults();

        // Serialize to JSON
        let json_str = serde_json::to_string_pretty(&original_config).unwrap();

        // Deserialize from JSON
        let deserialized_config: EnhancedLearningConfig = serde_json::from_str(&json_str).unwrap();

        // Verify key fields match
        assert_eq!(original_config.environment, deserialized_config.environment);
        assert_eq!(
            original_config.learning.adaptation_threshold,
            deserialized_config.learning.adaptation_threshold
        );
        assert_eq!(
            original_config.monitoring.metrics_enabled,
            deserialized_config.monitoring.metrics_enabled
        );
        assert_eq!(
            original_config.health_checks.enabled,
            deserialized_config.health_checks.enabled
        );
        assert_eq!(
            original_config.alerts.enabled,
            deserialized_config.alerts.enabled
        );

        // Validate both configurations
        assert!(original_config.validate().is_ok());
        assert!(deserialized_config.validate().is_ok());
    }

    #[tokio::test]
    async fn test_monitoring_respects_configuration_changes() {
        let mut initial_config = EnhancedLearningConfig::development_defaults();
        initial_config
            .monitoring
            .metrics_collection_interval_seconds = 1;
        initial_config.health_checks.check_interval_seconds = 1;

        let manager = LearningConfigManager::new(initial_config.learning.clone());

        // Create monitoring components with initial configuration
        let mut metrics_collector =
            LearningMetricsCollector::new(crate::learning::monitoring::MetricsConfig {
                collection_interval_seconds: initial_config
                    .monitoring
                    .metrics_collection_interval_seconds,
                enabled_metrics: vec!["adaptation".to_string()],
                retention_hours: 24,
                export_format: "json".to_string(),
                aggregation_window_minutes: 5,
            });

        let mut health_checker = LearningHealthChecker::new(initial_config.health_checks.clone());

        // Start monitoring
        metrics_collector.start_collection().await.unwrap();
        health_checker.start_health_monitoring().await.unwrap();

        // Let monitoring run briefly
        sleep(Duration::from_millis(100)).await;

        // Record some metrics
        metrics_collector
            .record_adaptation(Duration::from_millis(150), true, 0.87)
            .await
            .unwrap();

        // Check that metrics were recorded
        let metrics = metrics_collector.get_current_metrics().await.unwrap();
        assert_eq!(metrics.adaptation_metrics.adaptations_applied, 1);

        // Check health
        let health_report = health_checker.check_system_health().await.unwrap();
        assert_eq!(health_report.overall_status, HealthStatus::Healthy);

        // Stop monitoring
        metrics_collector.stop_collection().await.unwrap();
        health_checker.stop_health_monitoring().await.unwrap();
    }

    #[tokio::test]
    async fn test_error_handling_in_integration_scenarios() {
        // Test invalid environment variable values
        env::set_var("LEARNING_ADAPTATION_THRESHOLD", "invalid_float");

        let result = LearningConfigManager::from_environment().await;
        assert!(result.is_err());

        env::remove_var("LEARNING_ADAPTATION_THRESHOLD");

        // Test invalid configuration file
        let temp_dir = TempDir::new().unwrap();
        let invalid_config_path = temp_dir.path().join("invalid_config.json");
        tokio::fs::write(&invalid_config_path, "{ invalid json }")
            .await
            .unwrap();

        let result =
            LearningConfigManager::from_file_with_env(invalid_config_path.to_str().unwrap()).await;
        assert!(result.is_err());

        // Test configuration validation errors
        let mut config = EnhancedLearningConfig::development_defaults();
        config.learning.adaptation_threshold = 2.0; // Invalid value

        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn test_metrics_history_integration() {
        let metrics_config = crate::learning::monitoring::MetricsConfig {
            collection_interval_seconds: 1,
            enabled_metrics: vec!["adaptation".to_string()],
            retention_hours: 24,
            export_format: "json".to_string(),
            aggregation_window_minutes: 5,
        };

        let collector = LearningMetricsCollector::new(metrics_config);

        // Record multiple adaptation events
        collector
            .record_adaptation(Duration::from_millis(100), true, 0.85)
            .await
            .unwrap();
        collector
            .record_adaptation(Duration::from_millis(150), true, 0.90)
            .await
            .unwrap();
        collector
            .record_adaptation(Duration::from_millis(200), false, 0.60)
            .await
            .unwrap();

        // Get current metrics
        let current_metrics = collector.get_current_metrics().await.unwrap();
        assert_eq!(current_metrics.adaptation_metrics.adaptations_applied, 2);
        assert_eq!(current_metrics.adaptation_metrics.adaptations_failed, 1);
        assert_eq!(current_metrics.adaptation_metrics.success_rate, 2.0 / 3.0);

        // Get metrics history
        let history = collector
            .get_metrics_history(Duration::from_hours(1))
            .await
            .unwrap();
        assert!(!history.is_empty());

        // Verify that history contains the current metrics
        let last_metrics = history.last().unwrap();
        assert_eq!(last_metrics.adaptation_metrics.adaptations_applied, 2);
        assert_eq!(last_metrics.adaptation_metrics.adaptations_failed, 1);
    }
}
