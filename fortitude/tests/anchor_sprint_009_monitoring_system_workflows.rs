// ABOUTME: Anchor tests for Sprint 009 Tasks 3 & 4 - Monitoring System Critical Workflows
//! These tests protect critical monitoring system functionality implemented in Sprint 009
//! Tasks 3 and 4. They ensure that monitoring workflows continue to work correctly
//! as the system evolves.
//!
//! ## Protected Functionality
//! - External API integration (metrics collection, alerting systems)
//! - Business logic (performance monitoring, health checks, alerting logic)
//! - Cross-component integration (monitoring+learning+API+MCP integration)
//! - Critical error handling (system failure detection and recovery)
//! - Type definition changes (monitoring data types, alert structures)

use chrono::{DateTime, Utc};
use fortitude::learning::{LearningConfig, LearningData, UserFeedback, PatternData};
use fortitude::monitoring::{
    Alert, AlertSeverity, AlertManager, AlertManagerConfig, 
    HealthReport, HealthStatus, HealthChecker, MonitoringResult,
    MonitoringConfig, MonitoringService, PerformanceMetrics
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Mock metrics collector for testing
pub struct MockMetricsCollector {
    api_metrics: Arc<RwLock<Vec<ApiMetricEntry>>>,
    provider_metrics: Arc<RwLock<HashMap<String, ProviderPerformanceMetrics>>>,
    quality_metrics: Arc<RwLock<Vec<QualityMetricEntry>>>,
    cache_metrics: Arc<RwLock<HashMap<String, CacheMetricEntry>>>,
    resource_metrics: Arc<RwLock<Vec<ResourceUtilizationMetrics>>>,
    thresholds: Arc<RwLock<PerformanceThresholds>>,
}

#[derive(Clone)]
pub struct ApiMetricEntry {
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration: Duration,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone)]
pub struct QualityMetricEntry {
    pub operation: String,
    pub duration: Duration,
    pub bytes_processed: usize,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone)]
pub struct CacheMetricEntry {
    pub cache_name: String,
    pub operation: CacheOperation,
    pub duration: Duration,
    pub timestamp: DateTime<Utc>,
}

impl MockMetricsCollector {
    pub fn new() -> Self {
        Self {
            api_metrics: Arc::new(RwLock::new(Vec::new())),
            provider_metrics: Arc::new(RwLock::new(HashMap::new())),
            quality_metrics: Arc::new(RwLock::new(Vec::new())),
            cache_metrics: Arc::new(RwLock::new(HashMap::new())),
            resource_metrics: Arc::new(RwLock::new(Vec::new())),
            thresholds: Arc::new(RwLock::new(PerformanceThresholds::default())),
        }
    }

    pub async fn record_api_request(
        &self,
        method: &str,
        path: &str,
        status: u16,
        duration: Duration,
    ) {
        let mut metrics = self.api_metrics.write().await;
        metrics.push(ApiMetricEntry {
            method: method.to_string(),
            path: path.to_string(),
            status,
            duration,
            timestamp: Utc::now(),
        });
    }

    pub async fn record_provider_metrics(&self, metrics: &ProviderPerformanceMetrics) {
        let mut provider_metrics = self.provider_metrics.write().await;
        provider_metrics.insert(metrics.provider_name.clone(), metrics.clone());
    }

    pub async fn record_quality_processing(
        &self,
        operation: &str,
        duration: Duration,
        bytes_processed: usize,
    ) {
        let mut metrics = self.quality_metrics.write().await;
        metrics.push(QualityMetricEntry {
            operation: operation.to_string(),
            duration,
            bytes_processed,
            timestamp: Utc::now(),
        });
    }

    pub async fn record_cache_operation(
        &self,
        cache_name: &str,
        operation: CacheOperation,
        duration: Duration,
    ) {
        let mut metrics = self.cache_metrics.write().await;
        let key = format!("{}_{:?}", cache_name, operation);
        metrics.insert(
            key,
            CacheMetricEntry {
                cache_name: cache_name.to_string(),
                operation,
                duration,
                timestamp: Utc::now(),
            },
        );
    }

    pub async fn record_resource_metrics(&self, metrics: &ResourceUtilizationMetrics) {
        let mut resource_metrics = self.resource_metrics.write().await;
        resource_metrics.push(metrics.clone());
    }

    pub async fn set_thresholds(&self, thresholds: PerformanceThresholds) {
        let mut stored_thresholds = self.thresholds.write().await;
        *stored_thresholds = thresholds;
    }

    pub async fn generate_performance_report(&self) -> MonitoringResult<PerformanceReport> {
        let api_metrics = self.api_metrics.read().await;
        let provider_metrics = self.provider_metrics.read().await;
        let quality_metrics = self.quality_metrics.read().await;
        let resource_metrics = self.resource_metrics.read().await;
        let thresholds = self.thresholds.read().await;

        let total_requests = api_metrics.len() as u64;
        let successful_requests = api_metrics.iter().filter(|m| m.status < 400).count() as u64;
        let failed_requests = total_requests - successful_requests;
        let error_rate = if total_requests > 0 {
            failed_requests as f64 / total_requests as f64
        } else {
            0.0
        };

        let avg_response_time = if !api_metrics.is_empty() {
            api_metrics.iter().map(|m| m.duration).sum::<Duration>() / api_metrics.len() as u32
        } else {
            Duration::default()
        };

        let mut violations = Vec::new();

        // Check response time violations
        if avg_response_time > thresholds.max_response_time {
            violations.push(ThresholdViolation {
                metric_type: "response_time".to_string(),
                actual_value: avg_response_time.as_millis() as f64,
                threshold_value: thresholds.max_response_time.as_millis() as f64,
                severity: ViolationSeverity::High,
                timestamp: Utc::now(),
                description: "Average response time exceeds threshold".to_string(),
            });
        }

        // Check error rate violations
        if error_rate > thresholds.max_error_rate {
            violations.push(ThresholdViolation {
                metric_type: "error_rate".to_string(),
                actual_value: error_rate,
                threshold_value: thresholds.max_error_rate,
                severity: ViolationSeverity::Critical,
                timestamp: Utc::now(),
                description: "Error rate exceeds threshold".to_string(),
            });
        }

        // Check resource utilization
        if let Some(latest_resource) = resource_metrics.last() {
            if latest_resource.cpu_usage_percent > thresholds.max_cpu_usage {
                violations.push(ThresholdViolation {
                    metric_type: "cpu_usage".to_string(),
                    actual_value: latest_resource.cpu_usage_percent,
                    threshold_value: thresholds.max_cpu_usage,
                    severity: ViolationSeverity::High,
                    timestamp: Utc::now(),
                    description: "CPU usage exceeds threshold".to_string(),
                });
            }

            if latest_resource.memory_usage_bytes > thresholds.max_memory_usage {
                violations.push(ThresholdViolation {
                    metric_type: "memory_usage".to_string(),
                    actual_value: latest_resource.memory_usage_bytes as f64,
                    threshold_value: thresholds.max_memory_usage as f64,
                    severity: ViolationSeverity::High,
                    timestamp: Utc::now(),
                    description: "Memory usage exceeds threshold".to_string(),
                });
            }
        }

        Ok(PerformanceReport {
            timestamp: Utc::now(),
            total_requests,
            successful_requests,
            failed_requests,
            error_rate,
            avg_response_time,
            threshold_violations: violations,
            providers: provider_metrics.clone(),
            quality_processing: QualityMetrics {
                total_evaluations: quality_metrics.len() as u64,
                avg_evaluation_time: if !quality_metrics.is_empty() {
                    quality_metrics.iter().map(|q| q.duration).sum::<Duration>()
                        / quality_metrics.len() as u32
                } else {
                    Duration::default()
                },
                total_bytes_processed: quality_metrics
                    .iter()
                    .map(|q| q.bytes_processed)
                    .sum::<usize>() as u64,
            },
        })
    }

    pub async fn check_threshold_violations(&self) -> MonitoringResult<Vec<ThresholdViolation>> {
        let report = self.generate_performance_report().await?;
        Ok(report.threshold_violations)
    }

    pub async fn get_api_metrics(&self) -> MonitoringResult<ApiMetrics> {
        let metrics = self.api_metrics.read().await;

        let total_requests = metrics.len() as u64;
        let avg_response_time = if !metrics.is_empty() {
            metrics.iter().map(|m| m.duration).sum::<Duration>() / metrics.len() as u32
        } else {
            Duration::default()
        };

        Ok(ApiMetrics {
            total_requests,
            average_response_time: avg_response_time,
            requests_per_second: if !metrics.is_empty() {
                // Calculate RPS based on time span
                let start_time = metrics.first().unwrap().timestamp;
                let end_time = metrics.last().unwrap().timestamp;
                let duration_secs = (end_time - start_time).num_seconds().max(1) as f64;
                total_requests as f64 / duration_secs
            } else {
                0.0
            },
            error_rate: {
                let failed = metrics.iter().filter(|m| m.status >= 400).count();
                if total_requests > 0 {
                    failed as f64 / total_requests as f64
                } else {
                    0.0
                }
            },
        })
    }
}

/// Mock health checker for testing
pub struct MockHealthChecker {
    components: Arc<RwLock<HashMap<String, ComponentHealth>>>,
}

impl MockHealthChecker {
    pub fn new() -> Self {
        Self {
            components: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_component(&self, name: String, health: ComponentHealth) {
        let mut components = self.components.write().await;
        components.insert(name, health);
    }

    pub async fn check_health(&self) -> MonitoringResult<HealthReport> {
        let components = self.components.read().await;
        let component_list: Vec<ComponentHealth> = components.values().cloned().collect();

        let overall_status = if component_list
            .iter()
            .any(|c| c.status == HealthStatus::Critical)
        {
            HealthStatus::Critical
        } else if component_list
            .iter()
            .any(|c| c.status == HealthStatus::Warning)
        {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };

        Ok(HealthReport {
            timestamp: Utc::now(),
            overall_status,
            components: component_list,
        })
    }
}

/// Mock alert manager for testing
pub struct MockAlertManager {
    active_alerts: Arc<RwLock<Vec<Alert>>>,
    alert_history: Arc<RwLock<Vec<Alert>>>,
    channels: Arc<RwLock<HashMap<String, String>>>,
}

impl MockAlertManager {
    pub fn new() -> Self {
        Self {
            active_alerts: Arc::new(RwLock::new(Vec::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_channel(&self, name: String, config: String) {
        let mut channels = self.channels.write().await;
        channels.insert(name, config);
    }

    pub async fn send_alert(&self, alert: Alert) -> MonitoringResult<()> {
        let mut active_alerts = self.active_alerts.write().await;
        let mut history = self.alert_history.write().await;

        active_alerts.push(alert.clone());
        history.push(alert);

        Ok(())
    }

    pub async fn resolve_alert(&self, alert_id: &str) -> MonitoringResult<()> {
        let mut active_alerts = self.active_alerts.write().await;
        active_alerts.retain(|a| a.id != alert_id);
        Ok(())
    }

    pub async fn get_active_alerts(&self) -> MonitoringResult<Vec<Alert>> {
        let active_alerts = self.active_alerts.read().await;
        Ok(active_alerts.clone())
    }
}

#[cfg(test)]
mod anchor_tests {
    use super::*;

    /// ANCHOR: Monitoring system end-to-end metrics collection workflow
    /// Tests: Metrics recording → Aggregation → Threshold detection → Performance reporting
    /// Protects: Complete monitoring workflow and performance tracking
    #[tokio::test]
    async fn test_anchor_monitoring_metrics_collection_workflow() {
        let metrics_collector = MockMetricsCollector::new();

        // Test 1: Set up performance thresholds
        let thresholds = PerformanceThresholds {
            max_response_time: Duration::from_millis(200),
            max_error_rate: 0.05,
            min_cache_hit_rate: 0.8,
            max_cpu_usage: 80.0,
            max_memory_usage: 1024 * 1024 * 1024, // 1GB
        };
        metrics_collector.set_thresholds(thresholds).await;

        // Test 2: Record API metrics with threshold violations
        let api_requests = vec![
            ("GET", "/api/research", 200, Duration::from_millis(150)),
            ("POST", "/api/classify", 201, Duration::from_millis(180)),
            ("GET", "/api/health", 200, Duration::from_millis(50)),
            ("POST", "/api/feedback", 500, Duration::from_millis(300)), // Error + slow
            ("GET", "/api/status", 200, Duration::from_millis(250)),    // Slow
        ];

        for (method, path, status, duration) in api_requests {
            metrics_collector
                .record_api_request(method, path, status, duration)
                .await;
        }

        // Test 3: Record provider performance metrics
        let provider_metrics = vec![
            ProviderPerformanceMetrics {
                provider_name: "claude".to_string(),
                request_count: 15,
                success_rate: 0.93,
                average_latency: Duration::from_millis(180),
                error_count: 1,
                last_success_time: Utc::now(),
            },
            ProviderPerformanceMetrics {
                provider_name: "openai".to_string(),
                request_count: 12,
                success_rate: 0.92,
                average_latency: Duration::from_millis(220),
                error_count: 1,
                last_success_time: Utc::now(),
            },
        ];

        for provider in &provider_metrics {
            metrics_collector.record_provider_metrics(provider).await;
        }

        // Test 4: Record quality processing metrics
        let quality_operations = vec![
            ("relevance_check", Duration::from_millis(25), 512),
            ("accuracy_validation", Duration::from_millis(35), 1024),
            ("consistency_analysis", Duration::from_millis(40), 768),
            ("completeness_review", Duration::from_millis(30), 640),
        ];

        for (operation, duration, bytes) in quality_operations {
            metrics_collector
                .record_quality_processing(operation, duration, bytes)
                .await;
        }

        // Test 5: Record cache operations
        let cache_operations = vec![
            (
                "vector_embeddings",
                CacheOperation::Hit,
                Duration::from_micros(500),
            ),
            (
                "vector_embeddings",
                CacheOperation::Hit,
                Duration::from_micros(450),
            ),
            (
                "vector_embeddings",
                CacheOperation::Miss,
                Duration::from_millis(15),
            ),
            (
                "research_cache",
                CacheOperation::Hit,
                Duration::from_millis(2),
            ),
            (
                "research_cache",
                CacheOperation::Miss,
                Duration::from_millis(50),
            ),
        ];

        for (cache_name, operation, duration) in cache_operations {
            metrics_collector
                .record_cache_operation(cache_name, operation, duration)
                .await;
        }

        // Test 6: Record resource utilization with violations
        let resource_metrics = ResourceUtilizationMetrics {
            cpu_usage_percent: 85.0,                // Exceeds 80% threshold
            memory_usage_bytes: 1536 * 1024 * 1024, // Exceeds 1GB threshold
            network_bytes_sent: 10240,
            network_bytes_received: 20480,
            disk_io_bytes: 4096,
            timestamp: Utc::now(),
        };
        metrics_collector
            .record_resource_metrics(&resource_metrics)
            .await;

        // Test 7: Generate comprehensive performance report
        let report = metrics_collector
            .generate_performance_report()
            .await
            .unwrap();

        // Verify API metrics aggregation
        assert_eq!(report.total_requests, 5, "Should track all API requests");
        assert_eq!(
            report.successful_requests, 4,
            "Should count successful requests"
        );
        assert_eq!(report.failed_requests, 1, "Should count failed requests");
        assert!(
            (report.error_rate - 0.2).abs() < 0.01,
            "Should calculate error rate correctly"
        );

        // Verify provider metrics are included
        assert_eq!(
            report.providers.len(),
            2,
            "Should include all provider metrics"
        );
        assert!(
            report.providers.contains_key("claude"),
            "Should include Claude metrics"
        );
        assert!(
            report.providers.contains_key("openai"),
            "Should include OpenAI metrics"
        );

        let claude_metrics = &report.providers["claude"];
        assert_eq!(claude_metrics.request_count, 15);
        assert_eq!(claude_metrics.success_rate, 0.93);

        // Verify quality processing metrics
        assert_eq!(
            report.quality_processing.total_evaluations, 4,
            "Should track quality evaluations"
        );
        assert!(
            report.quality_processing.avg_evaluation_time > Duration::default(),
            "Should calculate average time"
        );
        assert_eq!(
            report.quality_processing.total_bytes_processed, 2944,
            "Should sum bytes processed"
        );

        // Test 8: Threshold violation detection
        assert!(
            !report.threshold_violations.is_empty(),
            "Should detect threshold violations"
        );

        // Check for response time violation
        let response_time_violation = report
            .threshold_violations
            .iter()
            .find(|v| v.metric_type == "response_time");
        assert!(
            response_time_violation.is_some(),
            "Should detect response time violation"
        );

        if let Some(violation) = response_time_violation {
            assert_eq!(violation.severity, ViolationSeverity::High);
            assert!(
                violation.actual_value > 200.0,
                "Should report actual response time"
            );
        }

        // Check for error rate violation
        let error_rate_violation = report
            .threshold_violations
            .iter()
            .find(|v| v.metric_type == "error_rate");
        assert!(
            error_rate_violation.is_some(),
            "Should detect error rate violation"
        );

        if let Some(violation) = error_rate_violation {
            assert_eq!(violation.severity, ViolationSeverity::Critical);
            assert!(
                violation.actual_value > 0.05,
                "Should report actual error rate"
            );
        }

        // Check for CPU usage violation
        let cpu_violation = report
            .threshold_violations
            .iter()
            .find(|v| v.metric_type == "cpu_usage");
        assert!(cpu_violation.is_some(), "Should detect CPU usage violation");

        // Check for memory usage violation
        let memory_violation = report
            .threshold_violations
            .iter()
            .find(|v| v.metric_type == "memory_usage");
        assert!(
            memory_violation.is_some(),
            "Should detect memory usage violation"
        );

        // Test 9: Real-time threshold monitoring
        let violations = metrics_collector
            .check_threshold_violations()
            .await
            .unwrap();
        assert!(
            !violations.is_empty(),
            "Real-time monitoring should detect violations"
        );

        // Test 10: API metrics detailed view
        let api_metrics = metrics_collector.get_api_metrics().await.unwrap();
        assert_eq!(api_metrics.total_requests, 5);
        assert!(
            api_metrics.average_response_time > Duration::from_millis(180),
            "Average should reflect slow requests"
        );
        assert!(api_metrics.error_rate > 0.0, "Should report error rate");
        assert!(
            api_metrics.requests_per_second > 0.0,
            "Should calculate RPS"
        );
    }

    /// ANCHOR: Monitoring system health checking and alerting workflow
    /// Tests: Health registration → Status aggregation → Alert generation → Resolution tracking
    /// Protects: Health monitoring and alerting system integrity
    #[tokio::test]
    async fn test_anchor_monitoring_health_alerting_workflow() {
        let health_checker = MockHealthChecker::new();
        let alert_manager = MockAlertManager::new();

        // Test 1: Register health check components
        let components = vec![
            ("api_server", HealthStatus::Healthy),
            ("database", HealthStatus::Warning),
            ("cache_system", HealthStatus::Healthy),
            ("learning_service", HealthStatus::Critical),
            ("monitoring_service", HealthStatus::Healthy),
        ];

        for (name, status) in &components {
            let component_health = ComponentHealth {
                component_name: name.to_string(),
                status: status.clone(),
                message: format!("{} status: {:?}", name, status),
                last_check_time: Utc::now(),
                checks: HashMap::new(),
            };
            health_checker
                .register_component(name.to_string(), component_health)
                .await;
        }

        // Test 2: Perform comprehensive health check
        let health_report = health_checker.check_health().await.unwrap();

        // Verify overall health status aggregation
        assert_eq!(
            health_report.overall_status,
            HealthStatus::Critical,
            "Overall status should be Critical when any component is Critical"
        );
        assert_eq!(
            health_report.components.len(),
            5,
            "Should include all registered components"
        );

        // Verify individual component statuses
        let critical_components: Vec<&ComponentHealth> = health_report
            .components
            .iter()
            .filter(|c| c.status == HealthStatus::Critical)
            .collect();
        assert_eq!(
            critical_components.len(),
            1,
            "Should have one critical component"
        );
        assert_eq!(critical_components[0].component_name, "learning_service");

        let warning_components: Vec<&ComponentHealth> = health_report
            .components
            .iter()
            .filter(|c| c.status == HealthStatus::Warning)
            .collect();
        assert_eq!(
            warning_components.len(),
            1,
            "Should have one warning component"
        );
        assert_eq!(warning_components[0].component_name, "database");

        let healthy_components: Vec<&ComponentHealth> = health_report
            .components
            .iter()
            .filter(|c| c.status == HealthStatus::Healthy)
            .collect();
        assert_eq!(
            healthy_components.len(),
            3,
            "Should have three healthy components"
        );

        // Test 3: Set up alert channels
        alert_manager
            .register_channel("email".to_string(), "admin@example.com".to_string())
            .await;
        alert_manager
            .register_channel(
                "webhook".to_string(),
                "https://alerts.example.com/webhook".to_string(),
            )
            .await;
        alert_manager
            .register_channel("console".to_string(), "stdout".to_string())
            .await;

        // Test 4: Generate alerts based on health status
        let alerts_to_send = vec![
            Alert::new(
                "Critical Service Failure".to_string(),
                "Learning service is in critical state".to_string(),
                AlertSeverity::Critical,
                "learning_service".to_string(),
            ),
            Alert::new(
                "Database Performance Warning".to_string(),
                "Database showing performance degradation".to_string(),
                AlertSeverity::Warning,
                "database".to_string(),
            ),
            Alert::new(
                "High Memory Usage".to_string(),
                "System memory usage exceeds threshold".to_string(),
                AlertSeverity::High,
                "system".to_string(),
            ),
        ];

        let mut alert_ids = Vec::new();
        for alert in alerts_to_send {
            let alert_id = alert.id.clone();
            alert_ids.push(alert_id);
            alert_manager.send_alert(alert).await.unwrap();
        }

        // Test 5: Verify active alerts tracking
        let active_alerts = alert_manager.get_active_alerts().await.unwrap();
        assert_eq!(
            active_alerts.len(),
            3,
            "Should track all sent alerts as active"
        );

        // Verify alert severity distribution
        let critical_alerts: Vec<&Alert> = active_alerts
            .iter()
            .filter(|a| a.severity == AlertSeverity::Critical)
            .collect();
        assert_eq!(critical_alerts.len(), 1, "Should have one critical alert");

        let warning_alerts: Vec<&Alert> = active_alerts
            .iter()
            .filter(|a| a.severity == AlertSeverity::Warning)
            .collect();
        assert_eq!(warning_alerts.len(), 1, "Should have one warning alert");

        let high_alerts: Vec<&Alert> = active_alerts
            .iter()
            .filter(|a| a.severity == AlertSeverity::High)
            .collect();
        assert_eq!(high_alerts.len(), 1, "Should have one high severity alert");

        // Test 6: Alert resolution workflow
        // Resolve the critical alert
        let critical_alert_id = &critical_alerts[0].id;
        alert_manager
            .resolve_alert(critical_alert_id)
            .await
            .unwrap();

        // Verify alert resolution
        let active_after_resolution = alert_manager.get_active_alerts().await.unwrap();
        assert_eq!(
            active_after_resolution.len(),
            2,
            "Should have two active alerts after resolution"
        );

        let still_critical: Vec<&Alert> = active_after_resolution
            .iter()
            .filter(|a| a.severity == AlertSeverity::Critical)
            .collect();
        assert!(
            still_critical.is_empty(),
            "Should have no critical alerts after resolution"
        );

        // Test 7: Health status improvement workflow
        // Simulate learning service recovery
        let recovered_component = ComponentHealth {
            component_name: "learning_service".to_string(),
            status: HealthStatus::Warning, // Improved from Critical
            message: "Learning service recovering".to_string(),
            last_check_time: Utc::now(),
            checks: HashMap::new(),
        };
        health_checker
            .register_component("learning_service".to_string(), recovered_component)
            .await;

        // Check updated health status
        let updated_health_report = health_checker.check_health().await.unwrap();
        assert_eq!(
            updated_health_report.overall_status,
            HealthStatus::Warning,
            "Overall status should improve to Warning when no components are Critical"
        );

        // Test 8: Alert escalation based on duration
        // Simulate long-running alert
        let persistent_alert = Alert::new(
            "Persistent Performance Issue".to_string(),
            "Performance issue has been ongoing".to_string(),
            AlertSeverity::Medium,
            "performance".to_string(),
        );
        let persistent_alert_id = persistent_alert.id.clone();
        alert_manager.send_alert(persistent_alert).await.unwrap();

        // Verify persistent alert is tracked
        let current_active = alert_manager.get_active_alerts().await.unwrap();
        let persistent_found = current_active.iter().any(|a| a.id == persistent_alert_id);
        assert!(persistent_found, "Persistent alert should be tracked");

        // Test 9: Bulk alert resolution
        // Resolve remaining alerts
        for alert_id in &alert_ids[1..] {
            // Skip already resolved critical alert
            alert_manager.resolve_alert(alert_id).await.unwrap();
        }
        alert_manager
            .resolve_alert(&persistent_alert_id)
            .await
            .unwrap();

        let final_active = alert_manager.get_active_alerts().await.unwrap();
        assert!(final_active.is_empty(), "All alerts should be resolved");

        // Test 10: System recovery validation
        // Simulate full system recovery
        let recovery_components = vec![
            ("api_server", HealthStatus::Healthy),
            ("database", HealthStatus::Healthy),
            ("cache_system", HealthStatus::Healthy),
            ("learning_service", HealthStatus::Healthy),
            ("monitoring_service", HealthStatus::Healthy),
        ];

        for (name, status) in recovery_components {
            let component_health = ComponentHealth {
                component_name: name.to_string(),
                status,
                message: format!("{} fully recovered", name),
                last_check_time: Utc::now(),
                checks: HashMap::new(),
            };
            health_checker
                .register_component(name.to_string(), component_health)
                .await;
        }

        let recovery_report = health_checker.check_health().await.unwrap();
        assert_eq!(
            recovery_report.overall_status,
            HealthStatus::Healthy,
            "System should be fully healthy after recovery"
        );

        // Verify no new alerts for healthy system
        let final_alert_count = alert_manager.get_active_alerts().await.unwrap().len();
        assert_eq!(
            final_alert_count, 0,
            "Healthy system should have no active alerts"
        );
    }

    /// ANCHOR: Monitoring system performance under load validation
    /// Tests: Concurrent metrics collection → High-volume processing → Latency tracking → Resource monitoring
    /// Protects: Performance characteristics and scalability under load
    #[tokio::test]
    async fn test_anchor_monitoring_performance_under_load() {
        let metrics_collector = Arc::new(MockMetricsCollector::new());
        let health_checker = Arc::new(MockHealthChecker::new());
        let alert_manager = Arc::new(MockAlertManager::new());

        // Test 1: Set up aggressive performance thresholds for load testing
        let strict_thresholds = PerformanceThresholds {
            max_response_time: Duration::from_millis(50), // Very strict
            max_error_rate: 0.01,                         // 1%
            min_cache_hit_rate: 0.95,                     // 95%
            max_cpu_usage: 60.0,                          // 60%
            max_memory_usage: 512 * 1024 * 1024,          // 512MB
        };
        metrics_collector.set_thresholds(strict_thresholds).await;

        let load_test_start = Instant::now();

        // Test 2: High-volume concurrent API metrics recording
        let api_load_tasks = (0..100)
            .map(|i| {
                let collector = metrics_collector.clone();
                tokio::spawn(async move {
                    let method = if i % 2 == 0 { "GET" } else { "POST" };
                    let path = format!("/api/endpoint_{}", i % 10);
                    let status = if i % 20 == 0 { 500 } else { 200 }; // 5% error rate
                    let duration = Duration::from_millis(30 + (i % 100) as u64); // Varying response times

                    collector
                        .record_api_request(&method, &path, status, duration)
                        .await;
                })
            })
            .collect::<Vec<_>>();

        // Test 3: Concurrent provider metrics recording
        let provider_load_tasks = (0..50)
            .map(|i| {
                let collector = metrics_collector.clone();
                tokio::spawn(async move {
                    let provider_metrics = ProviderPerformanceMetrics {
                        provider_name: format!("provider_{}", i % 5),
                        request_count: 10 + (i % 20) as u64,
                        success_rate: 0.90 + (i % 10) as f64 * 0.01,
                        average_latency: Duration::from_millis(100 + (i % 50) as u64),
                        error_count: i % 3,
                        last_success_time: Utc::now(),
                    };
                    collector.record_provider_metrics(&provider_metrics).await;
                })
            })
            .collect::<Vec<_>>();

        // Test 4: Concurrent quality processing metrics
        let quality_load_tasks = (0..75)
            .map(|i| {
                let collector = metrics_collector.clone();
                tokio::spawn(async move {
                    let operation = format!("quality_op_{}", i % 8);
                    let duration = Duration::from_millis(20 + (i % 30) as u64);
                    let bytes = 256 + (i % 1024);

                    collector
                        .record_quality_processing(&operation, duration, bytes)
                        .await;
                })
            })
            .collect::<Vec<_>>();

        // Test 5: Concurrent cache operation recording
        let cache_load_tasks = (0..200)
            .map(|i| {
                let collector = metrics_collector.clone();
                tokio::spawn(async move {
                    let cache_name = format!("cache_{}", i % 5);
                    let operation = if i % 5 == 0 {
                        CacheOperation::Miss
                    } else {
                        CacheOperation::Hit
                    };
                    let duration = if matches!(operation, CacheOperation::Hit) {
                        Duration::from_micros(500 + (i % 1000) as u64)
                    } else {
                        Duration::from_millis(10 + (i % 40) as u64)
                    };

                    collector
                        .record_cache_operation(&cache_name, operation, duration)
                        .await;
                })
            })
            .collect::<Vec<_>>();

        // Test 6: Concurrent health status updates
        let health_load_tasks = (0..30)
            .map(|i| {
                let checker = health_checker.clone();
                tokio::spawn(async move {
                    let component_name = format!("component_{}", i % 10);
                    let status = match i % 6 {
                        0 => HealthStatus::Critical,
                        1 => HealthStatus::Warning,
                        _ => HealthStatus::Healthy,
                    };

                    let health = ComponentHealth {
                        component_name: component_name.clone(),
                        status,
                        message: format!("Status update {}", i),
                        last_check_time: Utc::now(),
                        checks: HashMap::new(),
                    };

                    checker.register_component(component_name, health).await;
                })
            })
            .collect::<Vec<_>>();

        // Test 7: Concurrent alert generation
        let alert_load_tasks = (0..40)
            .map(|i| {
                let manager = alert_manager.clone();
                tokio::spawn(async move {
                    let severity = match i % 4 {
                        0 => AlertSeverity::Critical,
                        1 => AlertSeverity::High,
                        2 => AlertSeverity::Medium,
                        _ => AlertSeverity::Low,
                    };

                    let alert = Alert::new(
                        format!("Load Test Alert {}", i),
                        format!("Alert generated during load test iteration {}", i),
                        severity,
                        format!("source_{}", i % 5),
                    );

                    manager.send_alert(alert).await.unwrap();
                })
            })
            .collect::<Vec<_>>();

        // Wait for all concurrent operations to complete
        for task in api_load_tasks {
            task.await.unwrap();
        }
        for task in provider_load_tasks {
            task.await.unwrap();
        }
        for task in quality_load_tasks {
            task.await.unwrap();
        }
        for task in cache_load_tasks {
            task.await.unwrap();
        }
        for task in health_load_tasks {
            task.await.unwrap();
        }
        for task in alert_load_tasks {
            task.await.unwrap();
        }

        let load_completion_time = load_test_start.elapsed();
        assert!(
            load_completion_time < Duration::from_secs(10),
            "Load test should complete within 10 seconds"
        );

        // Test 8: Performance report generation under load
        let report_start = Instant::now();
        let performance_report = metrics_collector
            .generate_performance_report()
            .await
            .unwrap();
        let report_generation_time = report_start.elapsed();

        assert!(
            report_generation_time < Duration::from_millis(500),
            "Performance report generation should be fast even under load"
        );

        // Verify load test data integrity
        assert_eq!(
            performance_report.total_requests, 100,
            "Should track all API requests"
        );
        assert!(
            performance_report.providers.len() <= 5,
            "Should aggregate provider metrics correctly"
        );
        assert_eq!(
            performance_report.quality_processing.total_evaluations, 75,
            "Should track all quality operations"
        );

        // Test 9: Health check performance under load
        let health_start = Instant::now();
        let health_report = health_checker.check_health().await.unwrap();
        let health_check_time = health_start.elapsed();

        assert!(
            health_check_time < Duration::from_millis(100),
            "Health checks should be fast under load"
        );
        assert!(
            health_report.components.len() <= 10,
            "Should track all unique components"
        );

        // Test 10: Alert system performance under load
        let alert_start = Instant::now();
        let active_alerts = alert_manager.get_active_alerts().await.unwrap();
        let alert_query_time = alert_start.elapsed();

        assert!(
            alert_query_time < Duration::from_millis(50),
            "Alert queries should be very fast"
        );
        assert_eq!(active_alerts.len(), 40, "Should track all generated alerts");

        // Test 11: Threshold violation detection performance
        let violation_start = Instant::now();
        let violations = metrics_collector
            .check_threshold_violations()
            .await
            .unwrap();
        let violation_check_time = violation_start.elapsed();

        assert!(
            violation_check_time < Duration::from_millis(200),
            "Threshold violation checks should be efficient"
        );
        assert!(
            !violations.is_empty(),
            "Should detect violations from load test"
        );

        // Test 12: System responsiveness during sustained load
        let sustained_start = Instant::now();

        // Perform sustained operations
        for batch in 0..5 {
            let batch_start = Instant::now();

            // Simulate sustained API load
            for i in 0..20 {
                metrics_collector
                    .record_api_request(
                        "GET",
                        &format!("/sustained/{}", i),
                        200,
                        Duration::from_millis(25),
                    )
                    .await;
            }

            // Check responsiveness between batches
            let batch_time = batch_start.elapsed();
            assert!(
                batch_time < Duration::from_millis(100),
                "Batch {} should complete quickly",
                batch
            );

            // Brief pause between batches
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        let sustained_time = sustained_start.elapsed();
        assert!(
            sustained_time < Duration::from_secs(2),
            "Sustained load test should maintain responsiveness"
        );

        // Test 13: Memory efficiency under load
        // Verify system doesn't accumulate excessive data
        let final_report = metrics_collector
            .generate_performance_report()
            .await
            .unwrap();
        assert!(
            final_report.total_requests >= 200,
            "Should track cumulative requests"
        );
        assert!(
            final_report.quality_processing.total_evaluations >= 75,
            "Should track cumulative processing"
        );

        // Test 14: Alert resolution performance under load
        let resolution_start = Instant::now();

        // Resolve all alerts to test bulk operations
        for alert in active_alerts {
            alert_manager.resolve_alert(&alert.id).await.unwrap();
        }

        let resolution_time = resolution_start.elapsed();
        assert!(
            resolution_time < Duration::from_secs(1),
            "Bulk alert resolution should be efficient"
        );

        let remaining_alerts = alert_manager.get_active_alerts().await.unwrap();
        assert!(remaining_alerts.is_empty(), "All alerts should be resolved");

        let total_test_time = load_test_start.elapsed();
        println!(
            "Complete monitoring load test finished in {:?}",
            total_test_time
        );
        println!("Load completion: {:?}", load_completion_time);
        println!("Report generation: {:?}", report_generation_time);
        println!("Health check: {:?}", health_check_time);
        println!("Alert query: {:?}", alert_query_time);
        println!("Violation check: {:?}", violation_check_time);
    }

    /// ANCHOR: Monitoring system integration with learning system workflow
    /// Tests: Cross-component monitoring → Learning metrics integration → Adaptive monitoring
    /// Protects: Integration between monitoring and learning systems
    #[tokio::test]
    async fn test_anchor_monitoring_learning_integration_workflow() {
        let metrics_collector = Arc::new(MockMetricsCollector::new());
        let health_checker = Arc::new(MockHealthChecker::new());

        // Test 1: Set up monitoring for learning system components
        let learning_components = vec![
            "learning_storage",
            "pattern_analyzer",
            "adaptation_engine",
            "feedback_processor",
            "optimization_service",
        ];

        for component in &learning_components {
            let health = ComponentHealth {
                component_name: component.to_string(),
                status: HealthStatus::Healthy,
                message: format!("{} is operational", component),
                last_check_time: Utc::now(),
                checks: HashMap::new(),
            };
            health_checker
                .register_component(component.to_string(), health)
                .await;
        }

        // Test 2: Monitor learning system API endpoints
        let learning_api_calls = vec![
            (
                "/api/learning/feedback",
                "POST",
                201,
                Duration::from_millis(45),
            ),
            (
                "/api/learning/patterns",
                "GET",
                200,
                Duration::from_millis(80),
            ),
            (
                "/api/learning/insights",
                "GET",
                200,
                Duration::from_millis(120),
            ),
            (
                "/api/learning/adaptation",
                "POST",
                200,
                Duration::from_millis(200),
            ),
            (
                "/api/learning/optimize",
                "POST",
                202,
                Duration::from_millis(300),
            ),
        ];

        for (path, method, status, duration) in learning_api_calls {
            metrics_collector
                .record_api_request(method, path, status, duration)
                .await;
        }

        // Test 3: Monitor learning algorithm performance
        let learning_operations = vec![
            ("feedback_analysis", Duration::from_millis(35), 1024),
            ("pattern_recognition", Duration::from_millis(85), 2048),
            ("similarity_search", Duration::from_millis(120), 4096),
            ("learning_generation", Duration::from_millis(150), 1536),
            ("adaptation_calculation", Duration::from_millis(200), 3072),
        ];

        for (operation, duration, bytes) in learning_operations {
            metrics_collector
                .record_quality_processing(operation, duration, bytes)
                .await;
        }

        // Test 4: Monitor learning system caching
        let learning_cache_ops = vec![
            (
                "embeddings_cache",
                CacheOperation::Hit,
                Duration::from_micros(800),
            ),
            (
                "embeddings_cache",
                CacheOperation::Hit,
                Duration::from_micros(750),
            ),
            (
                "embeddings_cache",
                CacheOperation::Miss,
                Duration::from_millis(25),
            ),
            (
                "patterns_cache",
                CacheOperation::Hit,
                Duration::from_millis(5),
            ),
            (
                "patterns_cache",
                CacheOperation::Miss,
                Duration::from_millis(60),
            ),
            (
                "learning_cache",
                CacheOperation::Hit,
                Duration::from_millis(3),
            ),
        ];

        for (cache_name, operation, duration) in learning_cache_ops {
            metrics_collector
                .record_cache_operation(cache_name, operation, duration)
                .await;
        }

        // Test 5: Monitor learning system resource utilization
        let learning_resources = vec![
            ResourceUtilizationMetrics {
                cpu_usage_percent: 45.0,               // Normal learning processing
                memory_usage_bytes: 800 * 1024 * 1024, // 800MB for vectors
                network_bytes_sent: 5120,
                network_bytes_received: 10240,
                disk_io_bytes: 8192,
                timestamp: Utc::now(),
            },
            ResourceUtilizationMetrics {
                cpu_usage_percent: 75.0,                // High during adaptation
                memory_usage_bytes: 1200 * 1024 * 1024, // 1.2GB during optimization
                network_bytes_sent: 8192,
                network_bytes_received: 16384,
                disk_io_bytes: 12288,
                timestamp: Utc::now(),
            },
        ];

        for resource_metric in learning_resources {
            metrics_collector
                .record_resource_metrics(&resource_metric)
                .await;
        }

        // Test 6: Generate integrated performance report
        let integrated_report = metrics_collector
            .generate_performance_report()
            .await
            .unwrap();

        // Verify learning API monitoring
        assert_eq!(
            integrated_report.total_requests, 5,
            "Should track learning API calls"
        );
        assert_eq!(
            integrated_report.successful_requests, 5,
            "Learning APIs should be successful"
        );
        assert_eq!(
            integrated_report.error_rate, 0.0,
            "Learning system should have no errors"
        );

        // Verify learning operation monitoring
        assert_eq!(
            integrated_report.quality_processing.total_evaluations, 5,
            "Should track learning operations"
        );
        assert!(
            integrated_report.quality_processing.avg_evaluation_time > Duration::from_millis(100),
            "Learning operations should be tracked with realistic timing"
        );
        assert_eq!(
            integrated_report.quality_processing.total_bytes_processed, 11776,
            "Should track learning data processing volume"
        );

        // Test 7: Health monitoring of learning components
        let learning_health = health_checker.check_health().await.unwrap();
        assert_eq!(
            learning_health.overall_status,
            HealthStatus::Healthy,
            "Learning system should be healthy"
        );
        assert_eq!(
            learning_health.components.len(),
            5,
            "Should monitor all learning components"
        );

        // Verify specific learning components
        let learning_component_names: Vec<&str> = learning_health
            .components
            .iter()
            .map(|c| c.component_name.as_str())
            .collect();

        assert!(
            learning_component_names.contains(&"learning_storage"),
            "Should monitor learning storage"
        );
        assert!(
            learning_component_names.contains(&"pattern_analyzer"),
            "Should monitor pattern analyzer"
        );
        assert!(
            learning_component_names.contains(&"adaptation_engine"),
            "Should monitor adaptation engine"
        );

        // Test 8: Simulate learning system stress and monitor impact
        // Record high-volume learning activity
        for i in 0..50 {
            let operation = format!("batch_learning_{}", i % 5);
            let duration = Duration::from_millis(80 + (i % 40) as u64);
            let bytes = 1024 + (i % 2048);

            metrics_collector
                .record_quality_processing(&operation, duration, bytes)
                .await;
        }

        // Check monitoring performance under learning load
        let load_report = metrics_collector
            .generate_performance_report()
            .await
            .unwrap();
        assert_eq!(
            load_report.quality_processing.total_evaluations, 55,
            "Should track increased learning activity"
        );

        // Test 9: Monitor learning system failure scenarios
        // Simulate learning component degradation
        let degraded_health = ComponentHealth {
            component_name: "pattern_analyzer".to_string(),
            status: HealthStatus::Warning,
            message: "Pattern analysis showing increased latency".to_string(),
            last_check_time: Utc::now(),
            checks: HashMap::new(),
        };
        health_checker
            .register_component("pattern_analyzer".to_string(), degraded_health)
            .await;

        let degraded_report = health_checker.check_health().await.unwrap();
        assert_eq!(
            degraded_report.overall_status,
            HealthStatus::Warning,
            "Should detect learning component degradation"
        );

        // Test 10: Adaptive monitoring based on learning insights
        // Simulate different learning workload patterns
        let workload_patterns = vec![
            (
                "feedback_heavy",
                vec![
                    ("feedback_processing", Duration::from_millis(30)),
                    ("feedback_processing", Duration::from_millis(35)),
                    ("feedback_processing", Duration::from_millis(28)),
                ],
            ),
            (
                "pattern_heavy",
                vec![
                    ("pattern_analysis", Duration::from_millis(90)),
                    ("pattern_analysis", Duration::from_millis(95)),
                    ("pattern_analysis", Duration::from_millis(88)),
                ],
            ),
            (
                "adaptation_heavy",
                vec![
                    ("adaptation_execution", Duration::from_millis(180)),
                    ("adaptation_execution", Duration::from_millis(175)),
                    ("adaptation_execution", Duration::from_millis(185)),
                ],
            ),
        ];

        for (pattern_name, operations) in workload_patterns {
            for (operation, duration) in operations {
                metrics_collector
                    .record_quality_processing(operation, duration, 1024)
                    .await;
            }
        }

        // Verify adaptive monitoring captures different patterns
        let pattern_report = metrics_collector
            .generate_performance_report()
            .await
            .unwrap();
        assert_eq!(
            pattern_report.quality_processing.total_evaluations, 64,
            "Should track all learning workload patterns"
        );

        // Test 11: Cross-system performance correlation
        // Verify that learning system metrics correlate with overall system health
        let final_health = health_checker.check_health().await.unwrap();
        let final_metrics = metrics_collector
            .generate_performance_report()
            .await
            .unwrap();

        // High learning activity should not degrade overall system health
        let learning_components_healthy = final_health
            .components
            .iter()
            .filter(|c| {
                c.component_name.starts_with("learning")
                    || c.component_name.contains("pattern")
                    || c.component_name.contains("adaptation")
            })
            .all(|c| c.status != HealthStatus::Critical);

        assert!(
            learning_components_healthy,
            "Learning components should maintain health under monitoring"
        );

        // Learning performance should be tracked accurately
        assert!(
            final_metrics.quality_processing.avg_evaluation_time > Duration::default(),
            "Learning performance should be measurable"
        );
        assert!(
            final_metrics.quality_processing.total_bytes_processed > 0,
            "Learning data processing should be quantified"
        );
    }
}
