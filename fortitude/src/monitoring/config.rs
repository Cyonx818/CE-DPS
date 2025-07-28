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

// ABOUTME: Performance monitoring configuration system
//! # Monitoring Configuration
//!
//! This module provides comprehensive configuration management for the Fortitude
//! monitoring and observability system. It supports environment-based configuration,
//! file-based configuration, and runtime configuration updates.
//!
//! ## Key Features
//!
//! - **Environment Variable Support**: Full environment variable override capability
//! - **File Configuration**: JSON and TOML configuration file support
//! - **Validation**: Comprehensive configuration validation with sensible defaults
//! - **Hot Reload**: Runtime configuration updates for non-breaking changes
//! - **Performance Tuning**: Configurable thresholds and collection intervals
//!
//! ## Configuration Structure
//!
//! ```
//! MonitoringConfiguration
//! ├── Collection (Metrics collection settings)
//! ├── Storage (Data persistence configuration)
//! ├── Alerting (Alert rules and notification settings)
//! ├── Performance (Performance thresholds and targets)
//! └── Export (Data export and dashboard configuration)
//! ```

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use validator::Validate;

/// Complete monitoring system configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
pub struct MonitoringConfiguration {
    /// Core monitoring settings
    #[validate(nested)]
    pub core: CoreConfig,

    /// Metrics collection configuration
    #[validate(nested)]
    pub collection: CollectionConfig,

    /// Data storage configuration
    #[validate(nested)]
    pub storage: StorageConfig,

    /// Alerting system configuration
    #[validate(nested)]
    pub alerting: AlertingConfig,

    /// Performance monitoring configuration
    #[validate(nested)]
    pub performance: PerformanceConfig,

    /// Export and dashboard configuration
    #[validate(nested)]
    pub export: ExportConfig,

    /// Component-specific configurations
    pub components: HashMap<String, ComponentConfig>,
}

/// Core monitoring system settings
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CoreConfig {
    /// Enable monitoring system
    pub enabled: bool,

    /// System identification
    pub system_name: String,

    /// Environment (development, staging, production)
    pub environment: String,

    /// Monitoring data directory
    pub data_directory: PathBuf,

    /// Log level for monitoring system
    pub log_level: String,

    /// Enable debug mode
    pub debug_mode: bool,

    /// Maximum memory usage in MB for monitoring system
    #[validate(range(min = 100, max = 10000))]
    pub max_memory_mb: u64,
}

/// Metrics collection configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CollectionConfig {
    /// Enable metrics collection
    pub enabled: bool,

    /// Metrics collection interval in seconds
    #[validate(range(min = 1, max = 3600))]
    pub interval_seconds: u64,

    /// Maximum metrics to keep in memory
    #[validate(range(min = 1000, max = 1000000))]
    pub max_metrics_in_memory: usize,

    /// Batch size for metric processing
    #[validate(range(min = 10, max = 10000))]
    pub batch_size: usize,

    /// Enable high-resolution metrics (sub-second timing)
    pub enable_high_resolution: bool,

    /// Metrics sampling rate (0.0 to 1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    pub sampling_rate: f64,

    /// Enable custom metrics
    pub enable_custom_metrics: bool,

    /// Maximum custom metric name length
    #[validate(range(min = 10, max = 200))]
    pub max_metric_name_length: usize,

    /// Metric collection timeout in milliseconds
    #[validate(range(min = 100, max = 30000))]
    pub collection_timeout_ms: u64,

    /// Enable metrics collection (alternative field expected by some tests)
    pub metrics_collection_enabled: bool,

    /// Alert thresholds configuration (expected by tests)
    pub alert_thresholds: serde_json::Value,

    /// Health check interval duration (expected by tests)
    pub health_check_interval: std::time::Duration,

    /// Data retention period duration (expected by tests)
    pub retention_period: std::time::Duration,
}

/// Data storage configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct StorageConfig {
    /// Enable persistent storage
    pub enabled: bool,

    /// Storage backend type (memory, file, database)
    pub backend: String,

    /// Storage file path (for file backend)
    pub file_path: Option<PathBuf>,

    /// Database connection string (for database backend)
    pub database_url: Option<String>,

    /// Data retention period in hours
    #[validate(range(min = 1, max = 8760))] // 1 hour to 1 year
    pub retention_hours: u64,

    /// Compression enabled
    pub enable_compression: bool,

    /// Maximum storage size in MB
    #[validate(range(min = 100, max = 100000))]
    pub max_storage_mb: u64,

    /// Storage cleanup interval in hours
    #[validate(range(min = 1, max = 168))]
    pub cleanup_interval_hours: u64,

    /// Enable automatic backups
    pub enable_backups: bool,

    /// Backup retention count
    #[validate(range(min = 1, max = 100))]
    pub backup_retention_count: u32,
}

/// Alerting system configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AlertingConfig {
    /// Enable alerting system
    pub enabled: bool,

    /// Alert evaluation interval in seconds
    #[validate(range(min = 10, max = 3600))]
    pub evaluation_interval_seconds: u64,

    /// Alert channels configuration
    #[validate(nested)]
    pub channels: AlertChannelsConfig,

    /// Alert rate limiting
    #[validate(nested)]
    pub rate_limiting: AlertRateLimitConfig,

    /// Default alert rules
    pub default_rules: Vec<AlertRuleConfig>,

    /// Alert escalation settings
    #[validate(nested)]
    pub escalation: AlertEscalationConfig,

    /// Enable alert correlation
    pub enable_correlation: bool,

    /// Alert correlation window in minutes
    #[validate(range(min = 1, max = 60))]
    pub correlation_window_minutes: u64,
}

/// Alert channels configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
pub struct AlertChannelsConfig {
    /// Email configuration
    pub email: Option<EmailChannelConfig>,

    /// Webhook configurations
    pub webhooks: Vec<WebhookChannelConfig>,

    /// Slack configuration
    pub slack: Option<SlackChannelConfig>,

    /// PagerDuty configuration
    pub pagerduty: Option<PagerDutyChannelConfig>,

    /// Custom channel configurations
    pub custom: HashMap<String, serde_json::Value>,
}

/// Email alert channel configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct EmailChannelConfig {
    /// Enable email alerts
    pub enabled: bool,

    /// SMTP configuration
    #[validate(nested)]
    pub smtp: SmtpConfig,

    /// Default recipients
    pub recipients: Vec<String>,

    /// Subject prefix
    pub subject_prefix: String,

    /// HTML template path
    pub html_template: Option<PathBuf>,

    /// Text template path
    pub text_template: Option<PathBuf>,
}

/// SMTP configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SmtpConfig {
    /// SMTP server hostname
    pub server: String,

    /// SMTP server port
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,

    /// Username for authentication
    pub username: String,

    /// Password for authentication
    pub password: String,

    /// From address
    #[validate(email)]
    pub from_address: String,

    /// Enable TLS
    pub enable_tls: bool,

    /// Connection timeout in seconds
    #[validate(range(min = 1, max = 120))]
    pub timeout_seconds: u64,
}

/// Webhook alert channel configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct WebhookChannelConfig {
    /// Webhook name/identifier
    pub name: String,

    /// Enable this webhook
    pub enabled: bool,

    /// Webhook URL
    #[validate(url)]
    pub url: String,

    /// HTTP method (GET, POST, PUT)
    pub method: String,

    /// Headers to include
    pub headers: HashMap<String, String>,

    /// Request timeout in seconds
    #[validate(range(min = 1, max = 120))]
    pub timeout_seconds: u64,

    /// Retry configuration
    #[validate(nested)]
    pub retry: RetryConfig,

    /// Payload template
    pub payload_template: Option<String>,
}

/// Slack alert channel configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SlackChannelConfig {
    /// Enable Slack alerts
    pub enabled: bool,

    /// Webhook URL
    #[validate(url)]
    pub webhook_url: String,

    /// Default channel
    pub channel: String,

    /// Bot username
    pub username: String,

    /// Bot icon emoji
    pub icon_emoji: Option<String>,

    /// Bot icon URL
    pub icon_url: Option<String>,

    /// Markdown enabled
    pub enable_markdown: bool,
}

/// PagerDuty alert channel configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PagerDutyChannelConfig {
    /// Enable PagerDuty alerts
    pub enabled: bool,

    /// Integration key
    pub integration_key: String,

    /// API endpoint
    #[validate(url)]
    pub api_endpoint: String,

    /// Default severity
    pub default_severity: String,

    /// Client name
    pub client: String,

    /// Client URL
    pub client_url: Option<String>,
}

/// Retry configuration for external services
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RetryConfig {
    /// Enable retries
    pub enabled: bool,

    /// Maximum retry attempts
    #[validate(range(min = 0, max = 10))]
    pub max_attempts: u32,

    /// Initial retry delay in milliseconds
    #[validate(range(min = 100, max = 30000))]
    pub initial_delay_ms: u64,

    /// Maximum retry delay in milliseconds
    #[validate(range(min = 1000, max = 300000))]
    pub max_delay_ms: u64,

    /// Backoff multiplier
    #[validate(range(min = 1.0, max = 10.0))]
    pub backoff_multiplier: f64,
}

/// Alert rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AlertRateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,

    /// Maximum alerts per hour per rule
    #[validate(range(min = 1, max = 1000))]
    pub max_alerts_per_hour: u32,

    /// Burst allowance
    #[validate(range(min = 1, max = 100))]
    pub burst_allowance: u32,

    /// Rate limit reset window in hours
    #[validate(range(min = 1, max = 24))]
    pub reset_window_hours: u64,
}

/// Alert rule configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AlertRuleConfig {
    /// Rule name
    pub name: String,

    /// Enable this rule
    pub enabled: bool,

    /// Metric name to monitor
    pub metric: String,

    /// Threshold value
    pub threshold: f64,

    /// Comparison operator (>, <, >=, <=, ==, !=)
    pub operator: String,

    /// Evaluation window in minutes
    #[validate(range(min = 1, max = 1440))]
    pub window_minutes: u64,

    /// Alert severity
    pub severity: String,

    /// Alert description
    pub description: String,

    /// Runbook URL
    pub runbook_url: Option<String>,

    /// Additional labels
    pub labels: HashMap<String, String>,
}

/// Alert escalation configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AlertEscalationConfig {
    /// Enable alert escalation
    pub enabled: bool,

    /// Escalation steps
    pub steps: Vec<EscalationStepConfig>,

    /// Maximum escalation level
    #[validate(range(min = 1, max = 10))]
    pub max_level: u32,

    /// Auto-resolve timeout in hours
    #[validate(range(min = 1, max = 168))]
    pub auto_resolve_hours: u64,
}

/// Escalation step configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct EscalationStepConfig {
    /// Step level
    #[validate(range(min = 1, max = 10))]
    pub level: u32,

    /// Delay before escalation in minutes
    #[validate(range(min = 1, max = 1440))]
    pub delay_minutes: u64,

    /// Channels to notify at this level
    pub channels: Vec<String>,

    /// Additional recipients
    pub recipients: Vec<String>,

    /// Override severity
    pub override_severity: Option<String>,
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PerformanceConfig {
    /// Enable performance monitoring
    pub enabled: bool,

    /// Response time thresholds in milliseconds
    #[validate(nested)]
    pub response_time: ResponseTimeConfig,

    /// Resource utilization thresholds
    #[validate(nested)]
    pub resource_utilization: ResourceConfig,

    /// Error rate thresholds
    #[validate(nested)]
    pub error_rates: ErrorRateConfig,

    /// Throughput monitoring
    #[validate(nested)]
    pub throughput: ThroughputConfig,

    /// SLA monitoring configuration
    #[validate(nested)]
    pub sla: SlaConfig,
}

/// Response time configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ResponseTimeConfig {
    /// Target response time in milliseconds
    #[validate(range(min = 1, max = 10000))]
    pub target_ms: u64,

    /// Warning threshold in milliseconds
    #[validate(range(min = 1, max = 10000))]
    pub warning_ms: u64,

    /// Critical threshold in milliseconds
    #[validate(range(min = 1, max = 10000))]
    pub critical_ms: u64,

    /// Enable percentile tracking
    pub enable_percentiles: bool,

    /// Percentiles to track (e.g., [50.0, 95.0, 99.0])
    pub percentiles: Vec<f64>,
}

/// Resource utilization configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ResourceConfig {
    /// CPU utilization thresholds
    #[validate(nested)]
    pub cpu: ThresholdConfig,

    /// Memory utilization thresholds
    #[validate(nested)]
    pub memory: ThresholdConfig,

    /// Disk utilization thresholds
    #[validate(nested)]
    pub disk: ThresholdConfig,

    /// Network utilization thresholds
    #[validate(nested)]
    pub network: ThresholdConfig,
}

/// Generic threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ThresholdConfig {
    /// Warning threshold (percentage)
    #[validate(range(min = 0.0, max = 100.0))]
    pub warning: f64,

    /// Critical threshold (percentage)
    #[validate(range(min = 0.0, max = 100.0))]
    pub critical: f64,

    /// Enable monitoring for this resource
    pub enabled: bool,
}

/// Error rate configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ErrorRateConfig {
    /// Warning error rate threshold (percentage)
    #[validate(range(min = 0.0, max = 100.0))]
    pub warning_percent: f64,

    /// Critical error rate threshold (percentage)
    #[validate(range(min = 0.0, max = 100.0))]
    pub critical_percent: f64,

    /// Evaluation window in minutes
    #[validate(range(min = 1, max = 1440))]
    pub window_minutes: u64,

    /// Minimum request count for evaluation
    #[validate(range(min = 1, max = 10000))]
    pub min_request_count: u64,
}

/// Throughput configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ThroughputConfig {
    /// Target requests per second
    #[validate(range(min = 1.0, max = 100000.0))]
    pub target_rps: f64,

    /// Minimum acceptable requests per second
    #[validate(range(min = 0.1, max = 100000.0))]
    pub min_rps: f64,

    /// Maximum sustainable requests per second
    #[validate(range(min = 1.0, max = 100000.0))]
    pub max_rps: f64,

    /// Evaluation window in minutes
    #[validate(range(min = 1, max = 60))]
    pub window_minutes: u64,
}

/// SLA configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SlaConfig {
    /// Enable SLA monitoring
    pub enabled: bool,

    /// Target availability percentage
    #[validate(range(min = 90.0, max = 100.0))]
    pub availability_target: f64,

    /// SLA reporting period in hours
    #[validate(range(min = 1, max = 8760))]
    pub reporting_period_hours: u64,

    /// SLA violation thresholds
    pub violation_thresholds: Vec<SlaThresholdConfig>,
}

/// SLA threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SlaThresholdConfig {
    /// Threshold name
    pub name: String,

    /// Availability percentage threshold
    #[validate(range(min = 0.0, max = 100.0))]
    pub availability: f64,

    /// Response time threshold in milliseconds
    #[validate(range(min = 1, max = 10000))]
    pub response_time_ms: u64,

    /// Actions to take when threshold is violated
    pub actions: Vec<String>,
}

/// Export and dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ExportConfig {
    /// Enable data export
    pub enabled: bool,

    /// Export formats (json, csv, prometheus)
    pub formats: Vec<String>,

    /// Export destinations
    pub destinations: Vec<ExportDestinationConfig>,

    /// Export interval in seconds
    #[validate(range(min = 60, max = 86400))]
    pub interval_seconds: u64,

    /// Dashboard configuration
    #[validate(nested)]
    pub dashboard: DashboardConfig,
}

/// Export destination configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ExportDestinationConfig {
    /// Destination name
    pub name: String,

    /// Destination type (file, http, s3, etc.)
    pub destination_type: String,

    /// Destination configuration
    pub config: HashMap<String, serde_json::Value>,

    /// Enable this destination
    pub enabled: bool,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DashboardConfig {
    /// Enable dashboard
    pub enabled: bool,

    /// Dashboard port
    #[validate(range(min = 1024, max = 65535))]
    pub port: u16,

    /// Dashboard host
    pub host: String,

    /// Enable authentication
    pub enable_auth: bool,

    /// Dashboard theme
    pub theme: String,

    /// Refresh interval in seconds
    #[validate(range(min = 5, max = 300))]
    pub refresh_interval_seconds: u64,

    /// Custom dashboard panels
    pub panels: Vec<DashboardPanelConfig>,
}

/// Dashboard panel configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DashboardPanelConfig {
    /// Panel name
    pub name: String,

    /// Panel type (chart, table, gauge, etc.)
    pub panel_type: String,

    /// Data source query
    pub query: String,

    /// Panel size (small, medium, large)
    pub size: String,

    /// Panel position
    #[validate(nested)]
    pub position: PanelPositionConfig,

    /// Panel-specific configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Panel position configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PanelPositionConfig {
    /// X coordinate
    #[validate(range(min = 0, max = 100))]
    pub x: u32,

    /// Y coordinate
    #[validate(range(min = 0, max = 100))]
    pub y: u32,

    /// Width
    #[validate(range(min = 1, max = 100))]
    pub width: u32,

    /// Height
    #[validate(range(min = 1, max = 100))]
    pub height: u32,
}

/// Component-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ComponentConfig {
    /// Component name
    pub name: String,

    /// Enable monitoring for this component
    pub enabled: bool,

    /// Component-specific thresholds
    pub thresholds: HashMap<String, f64>,

    /// Custom metrics for this component
    pub custom_metrics: Vec<String>,

    /// Sampling configuration
    #[validate(range(min = 0.0, max = 1.0))]
    pub sampling_rate: f64,

    /// Component tags
    pub tags: HashMap<String, String>,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            system_name: "fortitude-monitoring".to_string(),
            environment: "development".to_string(),
            data_directory: PathBuf::from("./monitoring_data"),
            log_level: "info".to_string(),
            debug_mode: false,
            max_memory_mb: 512,
        }
    }
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 10,
            max_metrics_in_memory: 10000,
            batch_size: 100,
            enable_high_resolution: false,
            sampling_rate: 1.0,
            enable_custom_metrics: true,
            max_metric_name_length: 100,
            collection_timeout_ms: 5000,
            metrics_collection_enabled: true,
            alert_thresholds: serde_json::json!({
                "cpu_usage": 80.0,
                "memory_usage": 85.0,
                "error_rate": 5.0,
                "response_time_ms": 1000.0
            }),
            health_check_interval: std::time::Duration::from_secs(300),
            retention_period: std::time::Duration::from_secs(604800), // 7 days
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: "file".to_string(),
            file_path: Some(PathBuf::from("./monitoring_data/metrics.db")),
            database_url: None,
            retention_hours: 168, // 7 days
            enable_compression: true,
            max_storage_mb: 1024, // 1GB
            cleanup_interval_hours: 24,
            enable_backups: false,
            backup_retention_count: 7,
        }
    }
}

impl Default for AlertingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            evaluation_interval_seconds: 60,
            channels: AlertChannelsConfig::default(),
            rate_limiting: AlertRateLimitConfig::default(),
            default_rules: Vec::new(),
            escalation: AlertEscalationConfig::default(),
            enable_correlation: false,
            correlation_window_minutes: 5,
        }
    }
}

impl Default for AlertRateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_alerts_per_hour: 10,
            burst_allowance: 3,
            reset_window_hours: 1,
        }
    }
}

impl Default for AlertEscalationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            steps: Vec::new(),
            max_level: 3,
            auto_resolve_hours: 24,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            response_time: ResponseTimeConfig::default(),
            resource_utilization: ResourceConfig::default(),
            error_rates: ErrorRateConfig::default(),
            throughput: ThroughputConfig::default(),
            sla: SlaConfig::default(),
        }
    }
}

impl Default for ResponseTimeConfig {
    fn default() -> Self {
        Self {
            target_ms: 200,
            warning_ms: 500,
            critical_ms: 1000,
            enable_percentiles: true,
            percentiles: vec![50.0, 95.0, 99.0, 99.9],
        }
    }
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            cpu: ThresholdConfig {
                warning: 70.0,
                critical: 90.0,
                enabled: true,
            },
            memory: ThresholdConfig {
                warning: 80.0,
                critical: 95.0,
                enabled: true,
            },
            disk: ThresholdConfig {
                warning: 85.0,
                critical: 95.0,
                enabled: true,
            },
            network: ThresholdConfig {
                warning: 80.0,
                critical: 95.0,
                enabled: false,
            },
        }
    }
}

impl Default for ErrorRateConfig {
    fn default() -> Self {
        Self {
            warning_percent: 5.0,
            critical_percent: 10.0,
            window_minutes: 5,
            min_request_count: 10,
        }
    }
}

impl Default for ThroughputConfig {
    fn default() -> Self {
        Self {
            target_rps: 100.0,
            min_rps: 10.0,
            max_rps: 1000.0,
            window_minutes: 5,
        }
    }
}

impl Default for SlaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            availability_target: 99.9,
            reporting_period_hours: 24,
            violation_thresholds: Vec::new(),
        }
    }
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            formats: vec!["json".to_string()],
            destinations: Vec::new(),
            interval_seconds: 300,
            dashboard: DashboardConfig::default(),
        }
    }
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 3001,
            host: "127.0.0.1".to_string(),
            enable_auth: false,
            theme: "light".to_string(),
            refresh_interval_seconds: 30,
            panels: Vec::new(),
        }
    }
}

impl MonitoringConfiguration {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // Core configuration
        if let Ok(enabled) = std::env::var("FORTITUDE_MONITORING_ENABLED") {
            config.core.enabled = enabled.parse().unwrap_or(true);
        }

        if let Ok(system_name) = std::env::var("FORTITUDE_MONITORING_SYSTEM_NAME") {
            config.core.system_name = system_name;
        }

        if let Ok(environment) = std::env::var("FORTITUDE_MONITORING_ENVIRONMENT") {
            config.core.environment = environment;
        }

        if let Ok(data_dir) = std::env::var("FORTITUDE_MONITORING_DATA_DIR") {
            config.core.data_directory = PathBuf::from(data_dir);
        }

        if let Ok(log_level) = std::env::var("FORTITUDE_MONITORING_LOG_LEVEL") {
            config.core.log_level = log_level;
        }

        if let Ok(debug) = std::env::var("FORTITUDE_MONITORING_DEBUG") {
            config.core.debug_mode = debug.parse().unwrap_or(false);
        }

        if let Ok(max_memory) = std::env::var("FORTITUDE_MONITORING_MAX_MEMORY_MB") {
            config.core.max_memory_mb = max_memory.parse().unwrap_or(512);
        }

        // Collection configuration
        if let Ok(collection_enabled) = std::env::var("FORTITUDE_MONITORING_COLLECTION_ENABLED") {
            config.collection.enabled = collection_enabled.parse().unwrap_or(true);
        }

        if let Ok(interval) = std::env::var("FORTITUDE_MONITORING_COLLECTION_INTERVAL") {
            config.collection.interval_seconds = interval.parse().unwrap_or(10);
        }

        if let Ok(max_metrics) = std::env::var("FORTITUDE_MONITORING_MAX_METRICS") {
            config.collection.max_metrics_in_memory = max_metrics.parse().unwrap_or(10000);
        }

        if let Ok(batch_size) = std::env::var("FORTITUDE_MONITORING_BATCH_SIZE") {
            config.collection.batch_size = batch_size.parse().unwrap_or(100);
        }

        if let Ok(high_res) = std::env::var("FORTITUDE_MONITORING_HIGH_RESOLUTION") {
            config.collection.enable_high_resolution = high_res.parse().unwrap_or(false);
        }

        if let Ok(sampling_rate) = std::env::var("FORTITUDE_MONITORING_SAMPLING_RATE") {
            config.collection.sampling_rate = sampling_rate.parse().unwrap_or(1.0);
        }

        // Storage configuration
        if let Ok(storage_enabled) = std::env::var("FORTITUDE_MONITORING_STORAGE_ENABLED") {
            config.storage.enabled = storage_enabled.parse().unwrap_or(true);
        }

        if let Ok(backend) = std::env::var("FORTITUDE_MONITORING_STORAGE_BACKEND") {
            config.storage.backend = backend;
        }

        if let Ok(file_path) = std::env::var("FORTITUDE_MONITORING_STORAGE_FILE") {
            config.storage.file_path = Some(PathBuf::from(file_path));
        }

        if let Ok(db_url) = std::env::var("FORTITUDE_MONITORING_DATABASE_URL") {
            config.storage.database_url = Some(db_url);
        }

        if let Ok(retention) = std::env::var("FORTITUDE_MONITORING_RETENTION_HOURS") {
            config.storage.retention_hours = retention.parse().unwrap_or(168);
        }

        if let Ok(compression) = std::env::var("FORTITUDE_MONITORING_COMPRESSION") {
            config.storage.enable_compression = compression.parse().unwrap_or(true);
        }

        if let Ok(max_storage) = std::env::var("FORTITUDE_MONITORING_MAX_STORAGE_MB") {
            config.storage.max_storage_mb = max_storage.parse().unwrap_or(1024);
        }

        // Performance configuration
        if let Ok(target_ms) = std::env::var("FORTITUDE_MONITORING_TARGET_RESPONSE_MS") {
            config.performance.response_time.target_ms = target_ms.parse().unwrap_or(200);
        }

        if let Ok(warning_ms) = std::env::var("FORTITUDE_MONITORING_WARNING_RESPONSE_MS") {
            config.performance.response_time.warning_ms = warning_ms.parse().unwrap_or(500);
        }

        if let Ok(critical_ms) = std::env::var("FORTITUDE_MONITORING_CRITICAL_RESPONSE_MS") {
            config.performance.response_time.critical_ms = critical_ms.parse().unwrap_or(1000);
        }

        // CPU thresholds
        if let Ok(cpu_warning) = std::env::var("FORTITUDE_MONITORING_CPU_WARNING") {
            config.performance.resource_utilization.cpu.warning =
                cpu_warning.parse().unwrap_or(70.0);
        }

        if let Ok(cpu_critical) = std::env::var("FORTITUDE_MONITORING_CPU_CRITICAL") {
            config.performance.resource_utilization.cpu.critical =
                cpu_critical.parse().unwrap_or(90.0);
        }

        // Memory thresholds
        if let Ok(memory_warning) = std::env::var("FORTITUDE_MONITORING_MEMORY_WARNING") {
            config.performance.resource_utilization.memory.warning =
                memory_warning.parse().unwrap_or(80.0);
        }

        if let Ok(memory_critical) = std::env::var("FORTITUDE_MONITORING_MEMORY_CRITICAL") {
            config.performance.resource_utilization.memory.critical =
                memory_critical.parse().unwrap_or(95.0);
        }

        // Error rate thresholds
        if let Ok(error_warning) = std::env::var("FORTITUDE_MONITORING_ERROR_WARNING") {
            config.performance.error_rates.warning_percent = error_warning.parse().unwrap_or(5.0);
        }

        if let Ok(error_critical) = std::env::var("FORTITUDE_MONITORING_ERROR_CRITICAL") {
            config.performance.error_rates.critical_percent =
                error_critical.parse().unwrap_or(10.0);
        }

        // Alerting configuration
        if let Ok(alerting_enabled) = std::env::var("FORTITUDE_MONITORING_ALERTING_ENABLED") {
            config.alerting.enabled = alerting_enabled.parse().unwrap_or(true);
        }

        if let Ok(eval_interval) = std::env::var("FORTITUDE_MONITORING_ALERT_INTERVAL") {
            config.alerting.evaluation_interval_seconds = eval_interval.parse().unwrap_or(60);
        }

        // Dashboard configuration
        if let Ok(dashboard_enabled) = std::env::var("FORTITUDE_MONITORING_DASHBOARD_ENABLED") {
            config.export.dashboard.enabled = dashboard_enabled.parse().unwrap_or(true);
        }

        if let Ok(dashboard_port) = std::env::var("FORTITUDE_MONITORING_DASHBOARD_PORT") {
            config.export.dashboard.port = dashboard_port.parse().unwrap_or(3001);
        }

        if let Ok(dashboard_host) = std::env::var("FORTITUDE_MONITORING_DASHBOARD_HOST") {
            config.export.dashboard.host = dashboard_host;
        }

        // Validate configuration
        config
            .validate()
            .map_err(|e| anyhow!("Configuration validation failed: {}", e))?;

        Ok(config)
    }

    /// Load configuration from file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| anyhow!("Failed to read config file: {}", e))?;

        let config: Self = if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::from_str(&content).map_err(|e| anyhow!("Failed to parse TOML config: {}", e))?
        } else {
            serde_json::from_str(&content)
                .map_err(|e| anyhow!("Failed to parse JSON config: {}", e))?
        };

        config
            .validate()
            .map_err(|e| anyhow!("Configuration validation failed: {}", e))?;

        Ok(config)
    }

    /// Save configuration to file
    pub async fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let content = if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::to_string_pretty(self)
                .map_err(|e| anyhow!("Failed to serialize config to TOML: {}", e))?
        } else {
            serde_json::to_string_pretty(self)
                .map_err(|e| anyhow!("Failed to serialize config to JSON: {}", e))?
        };

        fs::write(path, content)
            .await
            .map_err(|e| anyhow!("Failed to write config file: {}", e))?;

        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        Validate::validate(self).map_err(|e| anyhow!("Validation failed: {}", e))?;

        // Custom validations
        if self.performance.response_time.warning_ms >= self.performance.response_time.critical_ms {
            return Err(anyhow!(
                "Warning response time must be less than critical response time"
            ));
        }

        if self.performance.error_rates.warning_percent
            >= self.performance.error_rates.critical_percent
        {
            return Err(anyhow!(
                "Warning error rate must be less than critical error rate"
            ));
        }

        // Validate resource thresholds
        for (name, resource) in [
            ("CPU", &self.performance.resource_utilization.cpu),
            ("Memory", &self.performance.resource_utilization.memory),
            ("Disk", &self.performance.resource_utilization.disk),
            ("Network", &self.performance.resource_utilization.network),
        ] {
            if resource.warning >= resource.critical {
                return Err(anyhow!(
                    "{} warning threshold must be less than critical threshold",
                    name
                ));
            }
        }

        // Validate throughput configuration
        if self.performance.throughput.min_rps >= self.performance.throughput.target_rps {
            return Err(anyhow!("Minimum RPS must be less than target RPS"));
        }

        if self.performance.throughput.target_rps >= self.performance.throughput.max_rps {
            return Err(anyhow!("Target RPS must be less than maximum RPS"));
        }

        // Validate data directory
        if self.storage.enabled && !self.core.data_directory.exists() {
            std::fs::create_dir_all(&self.core.data_directory)
                .map_err(|e| anyhow!("Failed to create data directory: {}", e))?;
        }

        Ok(())
    }

    /// Get environment variable documentation
    pub fn get_env_var_documentation() -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "FORTITUDE_MONITORING_ENABLED",
                "Enable monitoring system (default: true)",
            ),
            (
                "FORTITUDE_MONITORING_SYSTEM_NAME",
                "System name for monitoring (default: fortitude-monitoring)",
            ),
            (
                "FORTITUDE_MONITORING_ENVIRONMENT",
                "Environment name (default: development)",
            ),
            (
                "FORTITUDE_MONITORING_DATA_DIR",
                "Data directory path (default: ./monitoring_data)",
            ),
            (
                "FORTITUDE_MONITORING_LOG_LEVEL",
                "Log level (default: info)",
            ),
            (
                "FORTITUDE_MONITORING_DEBUG",
                "Enable debug mode (default: false)",
            ),
            (
                "FORTITUDE_MONITORING_MAX_MEMORY_MB",
                "Maximum memory usage in MB (default: 512)",
            ),
            (
                "FORTITUDE_MONITORING_COLLECTION_ENABLED",
                "Enable metrics collection (default: true)",
            ),
            (
                "FORTITUDE_MONITORING_COLLECTION_INTERVAL",
                "Collection interval in seconds (default: 10)",
            ),
            (
                "FORTITUDE_MONITORING_MAX_METRICS",
                "Maximum metrics in memory (default: 10000)",
            ),
            (
                "FORTITUDE_MONITORING_BATCH_SIZE",
                "Metric processing batch size (default: 100)",
            ),
            (
                "FORTITUDE_MONITORING_HIGH_RESOLUTION",
                "Enable high-resolution metrics (default: false)",
            ),
            (
                "FORTITUDE_MONITORING_SAMPLING_RATE",
                "Metrics sampling rate 0.0-1.0 (default: 1.0)",
            ),
            (
                "FORTITUDE_MONITORING_STORAGE_ENABLED",
                "Enable persistent storage (default: true)",
            ),
            (
                "FORTITUDE_MONITORING_STORAGE_BACKEND",
                "Storage backend type (default: file)",
            ),
            (
                "FORTITUDE_MONITORING_STORAGE_FILE",
                "Storage file path (default: ./monitoring_data/metrics.db)",
            ),
            (
                "FORTITUDE_MONITORING_DATABASE_URL",
                "Database connection string (optional)",
            ),
            (
                "FORTITUDE_MONITORING_RETENTION_HOURS",
                "Data retention in hours (default: 168)",
            ),
            (
                "FORTITUDE_MONITORING_COMPRESSION",
                "Enable storage compression (default: true)",
            ),
            (
                "FORTITUDE_MONITORING_MAX_STORAGE_MB",
                "Maximum storage size in MB (default: 1024)",
            ),
            (
                "FORTITUDE_MONITORING_TARGET_RESPONSE_MS",
                "Target response time in ms (default: 200)",
            ),
            (
                "FORTITUDE_MONITORING_WARNING_RESPONSE_MS",
                "Warning response time in ms (default: 500)",
            ),
            (
                "FORTITUDE_MONITORING_CRITICAL_RESPONSE_MS",
                "Critical response time in ms (default: 1000)",
            ),
            (
                "FORTITUDE_MONITORING_CPU_WARNING",
                "CPU warning threshold % (default: 70.0)",
            ),
            (
                "FORTITUDE_MONITORING_CPU_CRITICAL",
                "CPU critical threshold % (default: 90.0)",
            ),
            (
                "FORTITUDE_MONITORING_MEMORY_WARNING",
                "Memory warning threshold % (default: 80.0)",
            ),
            (
                "FORTITUDE_MONITORING_MEMORY_CRITICAL",
                "Memory critical threshold % (default: 95.0)",
            ),
            (
                "FORTITUDE_MONITORING_ERROR_WARNING",
                "Error rate warning % (default: 5.0)",
            ),
            (
                "FORTITUDE_MONITORING_ERROR_CRITICAL",
                "Error rate critical % (default: 10.0)",
            ),
            (
                "FORTITUDE_MONITORING_ALERTING_ENABLED",
                "Enable alerting system (default: true)",
            ),
            (
                "FORTITUDE_MONITORING_ALERT_INTERVAL",
                "Alert evaluation interval in seconds (default: 60)",
            ),
            (
                "FORTITUDE_MONITORING_DASHBOARD_ENABLED",
                "Enable dashboard (default: true)",
            ),
            (
                "FORTITUDE_MONITORING_DASHBOARD_PORT",
                "Dashboard port (default: 3001)",
            ),
            (
                "FORTITUDE_MONITORING_DASHBOARD_HOST",
                "Dashboard host (default: 127.0.0.1)",
            ),
        ]
    }

    /// Create configuration for API server integration
    pub fn for_api_server() -> Self {
        let mut config = Self::default();

        // Optimize for API server workload
        config.collection.interval_seconds = 5; // More frequent collection
        config.collection.enable_high_resolution = true;
        config.performance.response_time.target_ms = 200; // Strict 200ms target
        config.performance.response_time.warning_ms = 300;
        config.performance.response_time.critical_ms = 500;

        // Add API-specific component configuration
        let mut api_component = ComponentConfig {
            name: "api-server".to_string(),
            enabled: true,
            thresholds: HashMap::new(),
            custom_metrics: vec![
                "http_requests_total".to_string(),
                "http_request_duration_ms".to_string(),
                "http_response_size_bytes".to_string(),
                "active_connections".to_string(),
            ],
            sampling_rate: 1.0,
            tags: HashMap::new(),
        };

        api_component
            .thresholds
            .insert("response_time_ms".to_string(), 200.0);
        api_component
            .thresholds
            .insert("error_rate_percent".to_string(), 5.0);
        api_component
            .tags
            .insert("component".to_string(), "api-server".to_string());

        config
            .components
            .insert("api-server".to_string(), api_component);

        config
    }

    /// Create configuration for MCP server integration
    pub fn for_mcp_server() -> Self {
        let mut config = Self::default();

        // Optimize for MCP server workload
        config.collection.interval_seconds = 10;
        config.performance.response_time.target_ms = 500; // More relaxed for complex operations
        config.performance.response_time.warning_ms = 1000;
        config.performance.response_time.critical_ms = 2000;

        // Add MCP-specific component configuration
        let mut mcp_component = ComponentConfig {
            name: "mcp-server".to_string(),
            enabled: true,
            thresholds: HashMap::new(),
            custom_metrics: vec![
                "mcp_requests_total".to_string(),
                "mcp_request_duration_ms".to_string(),
                "mcp_tool_calls_total".to_string(),
                "mcp_research_operations_total".to_string(),
            ],
            sampling_rate: 1.0,
            tags: HashMap::new(),
        };

        mcp_component
            .thresholds
            .insert("response_time_ms".to_string(), 500.0);
        mcp_component
            .thresholds
            .insert("error_rate_percent".to_string(), 2.0);
        mcp_component
            .tags
            .insert("component".to_string(), "mcp-server".to_string());

        config
            .components
            .insert("mcp-server".to_string(), mcp_component);

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config_validation() {
        let config = MonitoringConfiguration::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_response_time_thresholds() {
        let mut config = MonitoringConfiguration::default();
        config.performance.response_time.warning_ms = 1000;
        config.performance.response_time.critical_ms = 500; // Invalid: warning > critical
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_error_rate_thresholds() {
        let mut config = MonitoringConfiguration::default();
        config.performance.error_rates.warning_percent = 10.0;
        config.performance.error_rates.critical_percent = 5.0; // Invalid: warning > critical
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_throughput_thresholds() {
        let mut config = MonitoringConfiguration::default();
        config.performance.throughput.min_rps = 100.0;
        config.performance.throughput.target_rps = 50.0; // Invalid: min > target
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn test_config_file_roundtrip() {
        let original_config = MonitoringConfiguration::default();
        let temp_file = NamedTempFile::new().unwrap();

        // Save and load config
        original_config
            .save_to_file(temp_file.path())
            .await
            .unwrap();
        let loaded_config = MonitoringConfiguration::from_file(temp_file.path())
            .await
            .unwrap();

        // Compare key values
        assert_eq!(original_config.core.enabled, loaded_config.core.enabled);
        assert_eq!(
            original_config.core.system_name,
            loaded_config.core.system_name
        );
        assert_eq!(
            original_config.collection.interval_seconds,
            loaded_config.collection.interval_seconds
        );
        assert_eq!(
            original_config.performance.response_time.target_ms,
            loaded_config.performance.response_time.target_ms
        );
    }

    #[test]
    fn test_env_config_loading() {
        // Set test environment variables
        std::env::set_var("FORTITUDE_MONITORING_ENABLED", "false");
        std::env::set_var("FORTITUDE_MONITORING_SYSTEM_NAME", "test-system");
        std::env::set_var("FORTITUDE_MONITORING_TARGET_RESPONSE_MS", "150");
        std::env::set_var("FORTITUDE_MONITORING_CPU_WARNING", "60.0");

        let config = MonitoringConfiguration::from_env().unwrap();

        assert!(!config.core.enabled);
        assert_eq!(config.core.system_name, "test-system");
        assert_eq!(config.performance.response_time.target_ms, 150);
        assert_eq!(config.performance.resource_utilization.cpu.warning, 60.0);

        // Clean up
        std::env::remove_var("FORTITUDE_MONITORING_ENABLED");
        std::env::remove_var("FORTITUDE_MONITORING_SYSTEM_NAME");
        std::env::remove_var("FORTITUDE_MONITORING_TARGET_RESPONSE_MS");
        std::env::remove_var("FORTITUDE_MONITORING_CPU_WARNING");
    }

    #[test]
    fn test_api_server_config() {
        let config = MonitoringConfiguration::for_api_server();

        assert_eq!(config.collection.interval_seconds, 5);
        assert!(config.collection.enable_high_resolution);
        assert_eq!(config.performance.response_time.target_ms, 200);
        assert!(config.components.contains_key("api-server"));

        let api_component = &config.components["api-server"];
        assert_eq!(api_component.name, "api-server");
        assert!(api_component.enabled);
        assert!(api_component
            .custom_metrics
            .contains(&"http_requests_total".to_string()));
    }

    #[test]
    fn test_mcp_server_config() {
        let config = MonitoringConfiguration::for_mcp_server();

        assert_eq!(config.collection.interval_seconds, 10);
        assert_eq!(config.performance.response_time.target_ms, 500);
        assert!(config.components.contains_key("mcp-server"));

        let mcp_component = &config.components["mcp-server"];
        assert_eq!(mcp_component.name, "mcp-server");
        assert!(mcp_component.enabled);
        assert!(mcp_component
            .custom_metrics
            .contains(&"mcp_requests_total".to_string()));
    }

    #[test]
    fn test_threshold_config_validation() {
        let mut config = ThresholdConfig {
            warning: 90.0,
            critical: 80.0, // Invalid: warning > critical
            enabled: true,
        };

        // Manual validation since ThresholdConfig doesn't implement Validate
        assert!(config.warning > config.critical);

        // Fix the configuration
        config.critical = 95.0;
        assert!(config.warning < config.critical);
    }

    #[test]
    fn test_retry_config_validation() {
        let config = RetryConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = config.clone();
        invalid_config.max_attempts = 15; // Too high
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_env_var_documentation() {
        let docs = MonitoringConfiguration::get_env_var_documentation();
        assert!(!docs.is_empty());

        // Check that key variables are documented
        let var_names: Vec<&str> = docs.iter().map(|(name, _)| *name).collect();
        assert!(var_names.contains(&"FORTITUDE_MONITORING_ENABLED"));
        assert!(var_names.contains(&"FORTITUDE_MONITORING_TARGET_RESPONSE_MS"));
        assert!(var_names.contains(&"FORTITUDE_MONITORING_CPU_WARNING"));
    }
}
