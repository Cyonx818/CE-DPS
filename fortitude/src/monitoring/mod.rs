// ABOUTME: Performance monitoring and observability system for Fortitude
//! # Monitoring Module
//!
//! This module provides comprehensive performance monitoring and observability
//! for all components of the Fortitude system. It enables real-time metrics
//! collection, performance tracking, and alerting for critical issues.
//!
//! ## Core Components
//!
//! - **Metrics Collection**: Performance data gathering for all system components
//! - **Distributed Tracing**: Request flow tracking across system boundaries
//! - **Health Checks**: System health monitoring and status reporting
//! - **Alerting**: Automated notifications for critical performance issues
//!
//! ## Performance Requirements
//!
//! - Low overhead: <5% performance impact on monitored operations
//! - Real-time collection: Metrics available within 1 second
//! - High throughput: Support >10,000 metrics per second
//! - Response time target: <200ms for all system operations
//!
//! ## Architecture
//!
//! ```
//! Monitoring System
//! ├── Metrics (Performance data collection)
//! ├── Tracing (Distributed request tracking)
//! ├── Health (System status monitoring)
//! └── Alerts (Critical issue notifications)
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

pub mod alerts;
pub mod config;
pub mod health;
pub mod metrics;
pub mod tracing;

// Re-export key types for easier access
pub use metrics::{
    ApiMetrics, CacheMetrics, CacheOperation, LearningSystemMetrics, MetricsCollector,
    PerformanceReport, PerformanceThresholds, ProviderPerformanceMetrics, QualityMetrics,
    ResourceUtilizationMetrics, ThresholdViolation, ViolationSeverity,
};

pub use tracing::{Span, SpanBuilder, SpanId, TraceContext, TraceId, TracingService};

pub use health::{ComponentHealth, HealthChecker, HealthReport, HealthStatus};

pub use alerts::{Alert, AlertChannel, AlertManager, AlertRule, AlertSeverity};

pub use config::{
    AlertingConfig, CollectionConfig, ComponentConfig, CoreConfig, DashboardConfig,
    ErrorRateConfig, ExportConfig, MonitoringConfiguration, PerformanceConfig, ResourceConfig,
    ResponseTimeConfig, SlaConfig, StorageConfig, ThresholdConfig, ThroughputConfig,
};

/// Result type for monitoring operations
pub type MonitoringResult<T> = Result<T, MonitoringError>;

/// Errors that can occur in the monitoring system
#[derive(Debug, Error, Clone)]
pub enum MonitoringError {
    #[error("Metrics collection failed: {0}")]
    MetricsError(String),

    #[error("Tracing operation failed: {0}")]
    TracingError(String),

    #[error("Health check failed: {0}")]
    HealthCheckError(String),

    #[error("Alert processing failed: {0}")]
    AlertError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Storage operation failed: {0}")]
    StorageError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

impl From<serde_json::Error> for MonitoringError {
    fn from(error: serde_json::Error) -> Self {
        MonitoringError::SerializationError(error.to_string())
    }
}

/// Configuration for the monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable metrics collection
    pub enable_metrics: bool,

    /// Enable distributed tracing
    pub enable_tracing: bool,

    /// Enable health checks
    pub enable_health_checks: bool,

    /// Enable alerting system
    pub enable_alerts: bool,

    /// Metrics collection interval in seconds
    pub metrics_interval_seconds: u64,

    /// Health check interval in seconds
    pub health_check_interval_seconds: u64,

    /// Maximum number of metrics to store in memory
    pub max_metrics_in_memory: usize,

    /// Metrics retention period in hours
    pub metrics_retention_hours: u64,

    /// Performance thresholds configuration
    pub performance_thresholds: PerformanceThresholds,

    /// Alert configuration
    pub alert_config: AlertConfig,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            enable_tracing: true,
            enable_health_checks: true,
            enable_alerts: true,
            metrics_interval_seconds: 10,
            health_check_interval_seconds: 30,
            max_metrics_in_memory: 10000,
            metrics_retention_hours: 24,
            performance_thresholds: PerformanceThresholds::default(),
            alert_config: AlertConfig::default(),
        }
    }
}

/// Alert system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Enable email alerts
    pub enable_email: bool,

    /// Enable webhook alerts
    pub enable_webhooks: bool,

    /// Email configuration
    pub email_config: Option<EmailConfig>,

    /// Webhook URLs for alerts
    pub webhook_urls: Vec<String>,

    /// Alert rate limiting (max alerts per hour)
    pub rate_limit_per_hour: usize,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enable_email: false,
            enable_webhooks: false,
            email_config: None,
            webhook_urls: Vec::new(),
            rate_limit_per_hour: 10,
        }
    }
}

/// Email configuration for alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub to_addresses: Vec<String>,
}

/// Core monitoring trait for system components
#[async_trait]
pub trait Monitorable: Send + Sync {
    /// Get the component name for monitoring
    fn component_name(&self) -> &str;

    /// Record performance metrics for an operation
    async fn record_operation_metrics(
        &self,
        operation_name: &str,
        duration: Duration,
        success: bool,
        metadata: Option<HashMap<String, String>>,
    ) -> MonitoringResult<()>;

    /// Get current health status
    async fn get_health_status(&self) -> MonitoringResult<ComponentHealth>;

    /// Get performance metrics for this component
    async fn get_performance_metrics(&self) -> MonitoringResult<ComponentMetrics>;
}

/// Performance metrics for a specific component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetrics {
    /// Component name
    pub component_name: String,

    /// Total operations performed
    pub total_operations: u64,

    /// Successful operations count
    pub successful_operations: u64,

    /// Failed operations count
    pub failed_operations: u64,

    /// Average operation duration
    pub average_duration: Duration,

    /// 95th percentile duration
    pub p95_duration: Duration,

    /// Operations per second
    pub operations_per_second: f64,

    /// Last operation timestamp
    pub last_operation_time: Option<DateTime<Utc>>,

    /// Component-specific metrics
    pub custom_metrics: HashMap<String, f64>,
}

impl ComponentMetrics {
    /// Create new empty component metrics
    pub fn new(component_name: String) -> Self {
        Self {
            component_name,
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            average_duration: Duration::default(),
            p95_duration: Duration::default(),
            operations_per_second: 0.0,
            last_operation_time: None,
            custom_metrics: HashMap::new(),
        }
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            1.0
        } else {
            self.successful_operations as f64 / self.total_operations as f64
        }
    }

    /// Calculate error rate
    pub fn error_rate(&self) -> f64 {
        1.0 - self.success_rate()
    }
}

/// System-wide monitoring interface
#[async_trait]
pub trait MonitoringSystem: Send + Sync {
    /// Initialize the monitoring system
    async fn initialize(&mut self, config: MonitoringConfig) -> MonitoringResult<()>;

    /// Register a component for monitoring
    async fn register_component(&mut self, component: Box<dyn Monitorable>)
        -> MonitoringResult<()>;

    /// Get system-wide performance report
    async fn get_system_performance_report(&self) -> MonitoringResult<SystemPerformanceReport>;

    /// Check all performance thresholds
    async fn check_performance_thresholds(&self) -> MonitoringResult<Vec<ThresholdViolation>>;

    /// Start monitoring background tasks
    async fn start_monitoring(&mut self) -> MonitoringResult<()>;

    /// Stop monitoring and cleanup resources
    async fn stop_monitoring(&mut self) -> MonitoringResult<()>;
}

/// System-wide performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPerformanceReport {
    /// Report generation timestamp
    pub timestamp: DateTime<Utc>,

    /// Overall system health status
    pub overall_health: HealthStatus,

    /// Component-level metrics
    pub component_metrics: Vec<ComponentMetrics>,

    /// System-wide statistics
    pub system_stats: SystemStats,

    /// Active alerts
    pub active_alerts: Vec<Alert>,

    /// Performance threshold violations
    pub threshold_violations: Vec<ThresholdViolation>,
}

/// System-wide statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    /// Total requests processed
    pub total_requests: u64,

    /// Average system response time
    pub average_response_time: Duration,

    /// System error rate
    pub error_rate: f64,

    /// Total system uptime
    pub uptime: Duration,

    /// Current resource utilization
    pub resource_utilization: ResourceUtilizationMetrics,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitoring_config_default() {
        let config = MonitoringConfig::default();

        assert!(config.enable_metrics);
        assert!(config.enable_tracing);
        assert!(config.enable_health_checks);
        assert!(config.enable_alerts);
        assert_eq!(config.metrics_interval_seconds, 10);
        assert_eq!(config.health_check_interval_seconds, 30);
    }

    #[test]
    fn test_component_metrics_creation() {
        let metrics = ComponentMetrics::new("test_component".to_string());

        assert_eq!(metrics.component_name, "test_component");
        assert_eq!(metrics.total_operations, 0);
        assert_eq!(metrics.success_rate(), 1.0);
        assert_eq!(metrics.error_rate(), 0.0);
    }

    #[test]
    fn test_component_metrics_success_rate() {
        let mut metrics = ComponentMetrics::new("test".to_string());

        metrics.total_operations = 10;
        metrics.successful_operations = 8;
        metrics.failed_operations = 2;

        assert!((metrics.success_rate() - 0.8).abs() < 0.01);
        assert!((metrics.error_rate() - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_alert_config_default() {
        let config = AlertConfig::default();

        assert!(!config.enable_email);
        assert!(!config.enable_webhooks);
        assert!(config.email_config.is_none());
        assert!(config.webhook_urls.is_empty());
        assert_eq!(config.rate_limit_per_hour, 10);
    }

    // Mock implementation for testing Monitorable trait
    struct MockComponent {
        name: String,
    }

    #[async_trait]
    impl Monitorable for MockComponent {
        fn component_name(&self) -> &str {
            &self.name
        }

        async fn record_operation_metrics(
            &self,
            _operation_name: &str,
            _duration: Duration,
            _success: bool,
            _metadata: Option<HashMap<String, String>>,
        ) -> MonitoringResult<()> {
            Ok(())
        }

        async fn get_health_status(&self) -> MonitoringResult<ComponentHealth> {
            Ok(ComponentHealth {
                component_name: self.name.clone(),
                status: HealthStatus::Healthy,
                message: "All systems operational".to_string(),
                last_check_time: Utc::now(),
                checks: HashMap::new(),
            })
        }

        async fn get_performance_metrics(&self) -> MonitoringResult<ComponentMetrics> {
            Ok(ComponentMetrics::new(self.name.clone()))
        }
    }

    #[tokio::test]
    async fn test_monitorable_trait_implementation() {
        let component = MockComponent {
            name: "test_component".to_string(),
        };

        assert_eq!(component.component_name(), "test_component");

        let result = component
            .record_operation_metrics("test_operation", Duration::from_millis(100), true, None)
            .await;
        assert!(result.is_ok());

        let health = component.get_health_status().await.unwrap();
        assert_eq!(health.component_name, "test_component");
        assert_eq!(health.status, HealthStatus::Healthy);

        let metrics = component.get_performance_metrics().await.unwrap();
        assert_eq!(metrics.component_name, "test_component");
    }
}
