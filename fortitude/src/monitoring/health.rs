// ABOUTME: Health check system for monitoring component status
//! # Health Check Module
//!
//! This module provides comprehensive health monitoring for all system components.
//! It enables proactive detection of issues and provides detailed status information
//! for system operations and maintenance.
//!
//! ## Key Features
//!
//! - **Component health monitoring**: Individual component status tracking
//! - **Automated health checks**: Periodic status verification
//! - **Health aggregation**: System-wide health status calculation
//! - **Detailed reporting**: Rich health information for debugging

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use super::{MonitoringError, MonitoringResult};

/// Core health checker for system components
pub struct HealthChecker {
    /// Registered health checks by component name
    health_checks: Arc<RwLock<HashMap<String, Box<dyn HealthCheck>>>>,

    /// Latest health report cache
    latest_report: Arc<RwLock<Option<HealthReport>>>,

    /// Health check configuration
    config: HealthConfig,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(config: HealthConfig) -> Self {
        Self {
            health_checks: Arc::new(RwLock::new(HashMap::new())),
            latest_report: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// Register a health check for a component
    pub async fn register_health_check(
        &self,
        component_name: String,
        health_check: Box<dyn HealthCheck>,
    ) -> MonitoringResult<()> {
        let mut checks = self.health_checks.write().await;
        checks.insert(component_name, health_check);
        Ok(())
    }

    /// Run all health checks and generate a report
    pub async fn check_health(&self) -> MonitoringResult<HealthReport> {
        let checks = self.health_checks.read().await;
        let mut component_healths = Vec::new();
        let mut overall_status = HealthStatus::Healthy;

        for (component_name, health_check) in checks.iter() {
            match health_check.check_health().await {
                Ok(health) => {
                    // Update overall status based on component health
                    match health.status {
                        HealthStatus::Critical => overall_status = HealthStatus::Critical,
                        HealthStatus::Degraded if overall_status != HealthStatus::Critical => {
                            overall_status = HealthStatus::Degraded;
                        }
                        HealthStatus::Warning if overall_status == HealthStatus::Healthy => {
                            overall_status = HealthStatus::Warning;
                        }
                        _ => {}
                    }
                    component_healths.push(health);
                }
                Err(e) => {
                    // If health check fails, mark as critical
                    overall_status = HealthStatus::Critical;
                    component_healths.push(ComponentHealth {
                        component_name: component_name.clone(),
                        status: HealthStatus::Critical,
                        message: format!("Health check failed: {e}"),
                        last_check_time: Utc::now(),
                        checks: HashMap::new(),
                    });
                }
            }
        }

        let report = HealthReport {
            overall_status,
            components: component_healths,
            check_time: Utc::now(),
            summary: self.generate_summary(&overall_status),
        };

        // Cache the latest report
        let mut latest = self.latest_report.write().await;
        *latest = Some(report.clone());

        Ok(report)
    }

    /// Get the latest cached health report
    pub async fn get_latest_health_report(&self) -> MonitoringResult<Option<HealthReport>> {
        let report = self.latest_report.read().await;
        Ok(report.clone())
    }

    /// Get health status for a specific component
    pub async fn get_component_health(
        &self,
        component_name: &str,
    ) -> MonitoringResult<Option<ComponentHealth>> {
        let checks = self.health_checks.read().await;
        if let Some(health_check) = checks.get(component_name) {
            Ok(Some(health_check.check_health().await?))
        } else {
            Ok(None)
        }
    }

    /// Start periodic health checking
    pub async fn start_periodic_checks(&self) -> MonitoringResult<()> {
        // In a full implementation, this would spawn a background task
        // that runs health checks on the configured interval

        // For now, just validate the configuration
        if self.config.check_interval.as_secs() == 0 {
            return Err(MonitoringError::ConfigurationError(
                "Health check interval cannot be zero".to_string(),
            ));
        }

        // TODO: Implement background health checking task
        Ok(())
    }

    /// Generate a human-readable summary of health status
    fn generate_summary(&self, status: &HealthStatus) -> String {
        match status {
            HealthStatus::Healthy => "All systems operational".to_string(),
            HealthStatus::Warning => "Some components have warnings".to_string(),
            HealthStatus::Degraded => "System performance is degraded".to_string(),
            HealthStatus::Critical => "Critical issues detected".to_string(),
        }
    }
}

/// Configuration for health checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Enable periodic health checks
    pub enable_periodic_checks: bool,

    /// Interval between health checks
    pub check_interval: Duration,

    /// Timeout for individual health checks
    pub check_timeout: Duration,

    /// Maximum number of health check failures before marking as critical
    pub max_failures: u32,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            enable_periodic_checks: true,
            check_interval: Duration::from_secs(30),
            check_timeout: Duration::from_secs(5),
            max_failures: 3,
        }
    }
}

/// Health status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// All systems functioning normally
    Healthy,

    /// Non-critical issues present
    Warning,

    /// Performance or functionality degraded
    Degraded,

    /// Critical issues requiring immediate attention
    Critical,
}

impl HealthStatus {
    /// Check if the status is healthy (no issues)
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    /// Check if the status indicates problems
    pub fn has_issues(&self) -> bool {
        !self.is_healthy()
    }
}

/// Overall health report for the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Overall system health status
    pub overall_status: HealthStatus,

    /// Health status of individual components
    pub components: Vec<ComponentHealth>,

    /// When the health check was performed
    pub check_time: DateTime<Utc>,

    /// Human-readable summary
    pub summary: String,
}

impl HealthReport {
    /// Get components with issues
    pub fn components_with_issues(&self) -> Vec<&ComponentHealth> {
        self.components
            .iter()
            .filter(|c| c.status.has_issues())
            .collect()
    }

    /// Get critical components
    pub fn critical_components(&self) -> Vec<&ComponentHealth> {
        self.components
            .iter()
            .filter(|c| matches!(c.status, HealthStatus::Critical))
            .collect()
    }
}

/// Health status for an individual component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Name of the component
    pub component_name: String,

    /// Current health status
    pub status: HealthStatus,

    /// Human-readable status message
    pub message: String,

    /// When the health check was last performed
    pub last_check_time: DateTime<Utc>,

    /// Detailed check results
    pub checks: HashMap<String, CheckResult>,
}

impl ComponentHealth {
    /// Create a healthy component status
    pub fn healthy(component_name: String, message: String) -> Self {
        Self {
            component_name,
            status: HealthStatus::Healthy,
            message,
            last_check_time: Utc::now(),
            checks: HashMap::new(),
        }
    }

    /// Create a warning component status
    pub fn warning(component_name: String, message: String) -> Self {
        Self {
            component_name,
            status: HealthStatus::Warning,
            message,
            last_check_time: Utc::now(),
            checks: HashMap::new(),
        }
    }

    /// Create a degraded component status
    pub fn degraded(component_name: String, message: String) -> Self {
        Self {
            component_name,
            status: HealthStatus::Degraded,
            message,
            last_check_time: Utc::now(),
            checks: HashMap::new(),
        }
    }

    /// Create a critical component status
    pub fn critical(component_name: String, message: String) -> Self {
        Self {
            component_name,
            status: HealthStatus::Critical,
            message,
            last_check_time: Utc::now(),
            checks: HashMap::new(),
        }
    }

    /// Add a check result to the component health
    pub fn with_check(mut self, check_name: String, result: CheckResult) -> Self {
        self.checks.insert(check_name, result);
        self
    }
}

/// Result of an individual health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Whether the check passed
    pub passed: bool,

    /// Check result message
    pub message: String,

    /// Check execution time
    pub duration: Duration,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl CheckResult {
    /// Create a passing check result
    pub fn pass(message: String, duration: Duration) -> Self {
        Self {
            passed: true,
            message,
            duration,
            metadata: HashMap::new(),
        }
    }

    /// Create a failing check result
    pub fn fail(message: String, duration: Duration) -> Self {
        Self {
            passed: false,
            message,
            duration,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the check result
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Trait for implementing health checks
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform the health check and return component health status
    async fn check_health(&self) -> MonitoringResult<ComponentHealth>;

    /// Get the component name this health check is for
    fn component_name(&self) -> &str;

    /// Get the timeout for this health check
    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }
}

/// Basic health check implementation for testing
#[derive(Debug)]
pub struct BasicHealthCheck {
    component_name: String,
    status_provider: fn() -> HealthStatus,
}

impl BasicHealthCheck {
    /// Create a new basic health check
    pub fn new(component_name: String, status_provider: fn() -> HealthStatus) -> Self {
        Self {
            component_name,
            status_provider,
        }
    }
}

#[async_trait]
impl HealthCheck for BasicHealthCheck {
    async fn check_health(&self) -> MonitoringResult<ComponentHealth> {
        let start_time = std::time::Instant::now();
        let status = (self.status_provider)();
        let duration = start_time.elapsed();

        let message = match status {
            HealthStatus::Healthy => "Component is healthy".to_string(),
            HealthStatus::Warning => "Component has warnings".to_string(),
            HealthStatus::Degraded => "Component is degraded".to_string(),
            HealthStatus::Critical => "Component is critical".to_string(),
        };

        let check_result = CheckResult::pass("Status check completed".to_string(), duration);

        Ok(ComponentHealth {
            component_name: self.component_name.clone(),
            status,
            message,
            last_check_time: Utc::now(),
            checks: {
                let mut checks = HashMap::new();
                checks.insert("status_check".to_string(), check_result);
                checks
            },
        })
    }

    fn component_name(&self) -> &str {
        &self.component_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_checker_creation() {
        let config = HealthConfig::default();
        let checker = HealthChecker::new(config);

        let report = checker.check_health().await.unwrap();
        assert_eq!(report.overall_status, HealthStatus::Healthy);
        assert!(report.components.is_empty());
    }

    #[tokio::test]
    async fn test_register_and_check_health() {
        let config = HealthConfig::default();
        let checker = HealthChecker::new(config);

        let health_check = Box::new(BasicHealthCheck::new("test_component".to_string(), || {
            HealthStatus::Healthy
        }));

        checker
            .register_health_check("test_component".to_string(), health_check)
            .await
            .unwrap();

        let report = checker.check_health().await.unwrap();
        assert_eq!(report.overall_status, HealthStatus::Healthy);
        assert_eq!(report.components.len(), 1);
        assert_eq!(report.components[0].component_name, "test_component");
        assert_eq!(report.components[0].status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_status_aggregation() {
        let config = HealthConfig::default();
        let checker = HealthChecker::new(config);

        // Register healthy component
        let healthy_check = Box::new(BasicHealthCheck::new(
            "healthy_component".to_string(),
            || HealthStatus::Healthy,
        ));
        checker
            .register_health_check("healthy_component".to_string(), healthy_check)
            .await
            .unwrap();

        // Register warning component
        let warning_check = Box::new(BasicHealthCheck::new(
            "warning_component".to_string(),
            || HealthStatus::Warning,
        ));
        checker
            .register_health_check("warning_component".to_string(), warning_check)
            .await
            .unwrap();

        let report = checker.check_health().await.unwrap();
        assert_eq!(report.overall_status, HealthStatus::Warning);
        assert_eq!(report.components.len(), 2);
    }

    #[tokio::test]
    async fn test_critical_status_aggregation() {
        let config = HealthConfig::default();
        let checker = HealthChecker::new(config);

        // Register healthy component
        let healthy_check = Box::new(BasicHealthCheck::new(
            "healthy_component".to_string(),
            || HealthStatus::Healthy,
        ));
        checker
            .register_health_check("healthy_component".to_string(), healthy_check)
            .await
            .unwrap();

        // Register critical component
        let critical_check = Box::new(BasicHealthCheck::new(
            "critical_component".to_string(),
            || HealthStatus::Critical,
        ));
        checker
            .register_health_check("critical_component".to_string(), critical_check)
            .await
            .unwrap();

        let report = checker.check_health().await.unwrap();
        assert_eq!(report.overall_status, HealthStatus::Critical);

        let critical_components = report.critical_components();
        assert_eq!(critical_components.len(), 1);
        assert_eq!(critical_components[0].component_name, "critical_component");
    }

    #[tokio::test]
    async fn test_component_health_creation() {
        let health = ComponentHealth::healthy("test".to_string(), "All good".to_string());
        assert_eq!(health.component_name, "test");
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.message, "All good");
        assert!(health.status.is_healthy());
        assert!(!health.status.has_issues());

        let warning_health = ComponentHealth::warning("test".to_string(), "Warning".to_string());
        assert_eq!(warning_health.status, HealthStatus::Warning);
        assert!(!warning_health.status.is_healthy());
        assert!(warning_health.status.has_issues());
    }

    #[tokio::test]
    async fn test_basic_health_check() {
        let health_check =
            BasicHealthCheck::new("test_component".to_string(), || HealthStatus::Healthy);

        assert_eq!(health_check.component_name(), "test_component");
        assert_eq!(health_check.timeout(), Duration::from_secs(5));

        let health = health_check.check_health().await.unwrap();
        assert_eq!(health.component_name, "test_component");
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(!health.checks.is_empty());
    }

    #[test]
    fn test_check_result_creation() {
        let pass_result = CheckResult::pass("All good".to_string(), Duration::from_millis(100));
        assert!(pass_result.passed);
        assert_eq!(pass_result.message, "All good");
        assert_eq!(pass_result.duration, Duration::from_millis(100));

        let fail_result = CheckResult::fail("Failed".to_string(), Duration::from_millis(200));
        assert!(!fail_result.passed);
        assert_eq!(fail_result.message, "Failed");

        let with_metadata = fail_result.with_metadata("error_code".to_string(), "500".to_string());
        assert_eq!(
            with_metadata.metadata.get("error_code"),
            Some(&"500".to_string())
        );
    }

    #[test]
    fn test_health_config_default() {
        let config = HealthConfig::default();

        assert!(config.enable_periodic_checks);
        assert_eq!(config.check_interval, Duration::from_secs(30));
        assert_eq!(config.check_timeout, Duration::from_secs(5));
        assert_eq!(config.max_failures, 3);
    }

    #[test]
    fn test_health_report_filtering() {
        let healthy_component = ComponentHealth::healthy("healthy".to_string(), "OK".to_string());
        let warning_component =
            ComponentHealth::warning("warning".to_string(), "Warning".to_string());
        let critical_component =
            ComponentHealth::critical("critical".to_string(), "Critical".to_string());

        let report = HealthReport {
            overall_status: HealthStatus::Critical,
            components: vec![healthy_component, warning_component, critical_component],
            check_time: Utc::now(),
            summary: "Mixed health".to_string(),
        };

        let issues = report.components_with_issues();
        assert_eq!(issues.len(), 2); // warning and critical

        let critical = report.critical_components();
        assert_eq!(critical.len(), 1);
        assert_eq!(critical[0].component_name, "critical");
    }
}
