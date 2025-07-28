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

// ABOUTME: Configuration management for the learning system with environment support
//! # Learning System Configuration
//!
//! This module provides comprehensive configuration management for the learning system,
//! including environment variable support, validation, and monitoring configuration.
//!
//! ## Features
//!
//! - Environment variable override support
//! - Configuration validation and error handling
//! - Integration with existing configuration patterns
//! - Health check and monitoring configuration
//! - Production, development, and testing presets
//!
//! ## Environment Variables
//!
//! The following environment variables are supported:
//! - `LEARNING_ENABLE_FEEDBACK`: Enable/disable feedback learning
//! - `LEARNING_ENABLE_PATTERN_RECOGNITION`: Enable/disable pattern recognition
//! - `LEARNING_ENABLE_OPTIMIZATION`: Enable/disable optimization
//! - `LEARNING_ADAPTATION_THRESHOLD`: Confidence threshold for adaptations
//! - `LEARNING_MAX_DATA_AGE_DAYS`: Maximum age of learning data
//! - `LEARNING_STORAGE_COLLECTION_NAME`: Vector database collection name
//! - `LEARNING_METRICS_ENABLED`: Enable/disable metrics collection
//! - `LEARNING_HEALTH_CHECK_INTERVAL`: Health check interval in seconds
//! - `LEARNING_ALERT_ERROR_THRESHOLD`: Error rate threshold for alerts

use crate::learning::{
    AdaptationConfig, LearningConfig, LearningError, LearningResult, LearningStorageConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use tokio::fs;
use tracing::{debug, info, warn};

/// Configuration manager for learning system
pub struct LearningConfigManager {
    config: EnhancedLearningConfig,
    #[allow(dead_code)] // TODO: Will be used for environment-specific configurations
    environment_overrides: HashMap<String, LearningConfig>,
    watchers: Vec<Box<dyn ConfigWatcher>>,
}

/// Enhanced learning configuration with environment and monitoring support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedLearningConfig {
    /// Core learning configuration
    pub learning: LearningConfig,
    /// Monitoring configuration
    pub monitoring: LearningMonitoringConfig,
    /// Environment identifier
    pub environment: String,
    /// Health check configuration
    pub health_checks: HealthCheckConfig,
    /// Alert configuration
    pub alerts: AlertConfig,
}

/// Monitoring configuration for learning system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningMonitoringConfig {
    /// Enable metrics collection
    pub metrics_enabled: bool,
    /// Metrics collection interval in seconds
    pub metrics_collection_interval_seconds: u64,
    /// Enable performance tracking
    pub performance_tracking_enabled: bool,
    /// Enable adaptation tracking
    pub adaptation_tracking_enabled: bool,
    /// Enable storage metrics
    pub storage_metrics_enabled: bool,
    /// Alert thresholds
    pub alert_thresholds: MonitoringThresholds,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,
    /// Health check interval in seconds
    pub check_interval_seconds: u64,
    /// Enable storage health checks
    pub storage_health_check: bool,
    /// Enable adaptation health checks
    pub adaptation_health_check: bool,
    /// Enable pattern recognition health checks
    pub pattern_recognition_health_check: bool,
    /// Health check timeout in seconds
    pub timeout_seconds: u64,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Enable alerts
    pub enabled: bool,
    /// Error rate threshold for alerts
    pub error_rate_threshold: f64,
    /// Response time threshold in milliseconds
    pub response_time_threshold_ms: u64,
    /// Storage error threshold
    pub storage_error_threshold: u64,
    /// Adaptation failure threshold
    pub adaptation_failure_threshold: u64,
    /// Notification channels
    pub notification_channels: Vec<String>,
}

/// Monitoring thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringThresholds {
    /// Maximum adaptation time in milliseconds
    pub max_adaptation_time_ms: u64,
    /// Maximum storage response time in milliseconds
    pub max_storage_response_time_ms: u64,
    /// Minimum pattern recognition accuracy
    pub min_pattern_recognition_accuracy: f64,
    /// Maximum memory usage in MB
    pub max_memory_usage_mb: u64,
    /// Maximum CPU usage percentage
    pub max_cpu_usage_percent: f64,
}

/// Configuration watcher trait
#[async_trait::async_trait]
pub trait ConfigWatcher: Send + Sync {
    /// Called when configuration changes
    async fn on_config_changed(
        &mut self,
        old_config: &EnhancedLearningConfig,
        new_config: &EnhancedLearningConfig,
    );
}

impl LearningConfigManager {
    /// Create a new configuration manager
    pub fn new(config: LearningConfig) -> Self {
        let enhanced_config = EnhancedLearningConfig {
            learning: config,
            monitoring: LearningMonitoringConfig::default(),
            environment: "default".to_string(),
            health_checks: HealthCheckConfig::default(),
            alerts: AlertConfig::default(),
        };

        Self {
            config: enhanced_config,
            environment_overrides: HashMap::new(),
            watchers: Vec::new(),
        }
    }

    /// Load configuration from environment variables
    pub async fn from_environment() -> LearningResult<Self> {
        debug!("Loading learning configuration from environment variables");

        let mut config = LearningConfig::default();

        // Apply environment variable overrides
        if let Ok(enable_feedback) = env::var("LEARNING_ENABLE_FEEDBACK") {
            config.enable_feedback_learning = parse_bool(&enable_feedback)?;
        }

        if let Ok(enable_pattern_recognition) = env::var("LEARNING_ENABLE_PATTERN_RECOGNITION") {
            config.enable_pattern_recognition = parse_bool(&enable_pattern_recognition)?;
        }

        if let Ok(enable_optimization) = env::var("LEARNING_ENABLE_OPTIMIZATION") {
            config.enable_optimization = parse_bool(&enable_optimization)?;
        }

        if let Ok(threshold) = env::var("LEARNING_ADAPTATION_THRESHOLD") {
            config.adaptation_threshold = threshold.parse::<f64>().map_err(|e| {
                LearningError::ConfigurationError(format!("Invalid adaptation threshold: {e}"))
            })?;
        }

        if let Ok(max_age) = env::var("LEARNING_MAX_DATA_AGE_DAYS") {
            config.max_data_age_days = max_age.parse::<u32>().map_err(|e| {
                LearningError::ConfigurationError(format!("Invalid max data age: {e}"))
            })?;
        }

        if let Ok(min_threshold) = env::var("LEARNING_MIN_FEEDBACK_THRESHOLD") {
            config.min_feedback_threshold = min_threshold.parse::<usize>().map_err(|e| {
                LearningError::ConfigurationError(format!("Invalid min feedback threshold: {e}"))
            })?;
        }

        if let Ok(pattern_threshold) = env::var("LEARNING_PATTERN_FREQUENCY_THRESHOLD") {
            config.pattern_frequency_threshold = pattern_threshold.parse::<u32>().map_err(|e| {
                LearningError::ConfigurationError(format!(
                    "Invalid pattern frequency threshold: {e}"
                ))
            })?;
        }

        if let Ok(learning_rate) = env::var("LEARNING_RATE") {
            config.learning_rate = learning_rate.parse::<f64>().map_err(|e| {
                LearningError::ConfigurationError(format!("Invalid learning rate: {e}"))
            })?;
        }

        // Storage configuration
        if let Ok(collection_name) = env::var("LEARNING_STORAGE_COLLECTION_NAME") {
            config.storage.collection_name = collection_name;
        }

        if let Ok(enable_embeddings) = env::var("LEARNING_STORAGE_ENABLE_EMBEDDINGS") {
            config.storage.enable_embeddings = parse_bool(&enable_embeddings)?;
        }

        if let Ok(batch_size) = env::var("LEARNING_STORAGE_BATCH_SIZE") {
            config.storage.batch_size = batch_size.parse::<usize>().map_err(|e| {
                LearningError::ConfigurationError(format!("Invalid batch size: {e}"))
            })?;
        }

        if let Ok(retention_days) = env::var("LEARNING_STORAGE_RETENTION_DAYS") {
            config.storage.retention_days = retention_days.parse::<u32>().map_err(|e| {
                LearningError::ConfigurationError(format!("Invalid retention days: {e}"))
            })?;
        }

        // Monitoring configuration
        let mut monitoring = LearningMonitoringConfig::default();

        if let Ok(metrics_enabled) = env::var("LEARNING_METRICS_ENABLED") {
            monitoring.metrics_enabled = parse_bool(&metrics_enabled)?;
        }

        if let Ok(interval) = env::var("LEARNING_METRICS_INTERVAL") {
            monitoring.metrics_collection_interval_seconds =
                interval.parse::<u64>().map_err(|e| {
                    LearningError::ConfigurationError(format!("Invalid metrics interval: {e}"))
                })?;
        }

        // Health check configuration
        let mut health_checks = HealthCheckConfig::default();

        if let Ok(health_enabled) = env::var("LEARNING_HEALTH_CHECKS_ENABLED") {
            health_checks.enabled = parse_bool(&health_enabled)?;
        }

        if let Ok(health_interval) = env::var("LEARNING_HEALTH_CHECK_INTERVAL") {
            health_checks.check_interval_seconds = health_interval.parse::<u64>().map_err(|e| {
                LearningError::ConfigurationError(format!("Invalid health check interval: {e}"))
            })?;
        }

        // Alert configuration
        let mut alerts = AlertConfig::default();

        if let Ok(alerts_enabled) = env::var("LEARNING_ALERTS_ENABLED") {
            alerts.enabled = parse_bool(&alerts_enabled)?;
        }

        if let Ok(error_threshold) = env::var("LEARNING_ALERT_ERROR_THRESHOLD") {
            alerts.error_rate_threshold = error_threshold.parse::<f64>().map_err(|e| {
                LearningError::ConfigurationError(format!("Invalid error threshold: {e}"))
            })?;
        }

        if let Ok(response_threshold) = env::var("LEARNING_ALERT_RESPONSE_TIME_THRESHOLD") {
            alerts.response_time_threshold_ms = response_threshold.parse::<u64>().map_err(|e| {
                LearningError::ConfigurationError(format!("Invalid response time threshold: {e}"))
            })?;
        }

        let environment =
            env::var("LEARNING_ENVIRONMENT").unwrap_or_else(|_| "default".to_string());

        let enhanced_config = EnhancedLearningConfig {
            learning: config,
            monitoring,
            environment,
            health_checks,
            alerts,
        };

        // Validate configuration
        enhanced_config.validate()?;

        info!("Successfully loaded learning configuration from environment");

        Ok(Self {
            config: enhanced_config,
            environment_overrides: HashMap::new(),
            watchers: Vec::new(),
        })
    }

    /// Load configuration from file with environment overrides
    pub async fn from_file_with_env(path: &str) -> LearningResult<Self> {
        debug!("Loading learning configuration from file: {}", path);

        // Load base configuration from file
        let mut enhanced_config = EnhancedLearningConfig::from_file(path).await?;

        // Apply environment variable overrides
        enhanced_config.apply_env_vars()?;

        // Validate final configuration
        enhanced_config.validate()?;

        info!("Successfully loaded learning configuration from file with environment overrides");

        Ok(Self {
            config: enhanced_config,
            environment_overrides: HashMap::new(),
            watchers: Vec::new(),
        })
    }

    /// Get current configuration
    pub fn config(&self) -> &LearningConfig {
        &self.config.learning
    }

    /// Get enhanced configuration
    pub fn enhanced_config(&self) -> &EnhancedLearningConfig {
        &self.config
    }

    /// Update configuration and notify watchers
    pub async fn update_config(&mut self, new_config: LearningConfig) -> LearningResult<()> {
        debug!("Updating learning configuration");

        let old_enhanced_config = self.config.clone();

        // Update the learning config portion
        self.config.learning = new_config;

        // Validate new configuration
        self.config.validate()?;

        // Notify all watchers of configuration change
        for watcher in &mut self.watchers {
            watcher
                .on_config_changed(&old_enhanced_config, &self.config)
                .await;
        }

        info!("Learning configuration updated successfully");
        Ok(())
    }

    /// Add configuration watcher
    pub fn add_watcher(&mut self, watcher: Box<dyn ConfigWatcher>) {
        debug!("Adding configuration watcher");
        self.watchers.push(watcher);
    }

    /// Validate configuration
    pub fn validate(&self) -> LearningResult<()> {
        self.config.validate()
    }

    /// Apply environment-specific overrides
    pub fn apply_environment_overrides(&mut self, environment: &str) -> LearningResult<()> {
        debug!("Applying environment overrides for: {}", environment);

        match environment {
            "production" => {
                self.config.learning.adaptation_threshold = 0.8;
                self.config.learning.enable_optimization = true;
                self.config.learning.min_feedback_threshold = 10;
                self.config.monitoring.metrics_enabled = true;
                self.config.health_checks.enabled = true;
                self.config.alerts.enabled = true;
            }
            "development" => {
                self.config.learning.adaptation_threshold = 0.6;
                self.config.learning.enable_optimization = false;
                self.config.learning.min_feedback_threshold = 3;
                self.config.monitoring.metrics_enabled = true;
                self.config.health_checks.enabled = true;
                self.config.alerts.enabled = false;
            }
            "testing" => {
                self.config.learning.adaptation_threshold = 0.5;
                self.config.learning.enable_optimization = false;
                self.config.learning.min_feedback_threshold = 1;
                self.config.monitoring.metrics_collection_interval_seconds = 10;
                self.config.health_checks.check_interval_seconds = 5;
                self.config.alerts.enabled = false;
            }
            _ => {
                warn!(
                    "Unknown environment: {}, using default overrides",
                    environment
                );
            }
        }

        self.config.environment = environment.to_string();

        // Validate after applying overrides
        self.config.validate()?;

        info!("Applied environment overrides for: {}", environment);
        Ok(())
    }
}

impl EnhancedLearningConfig {
    /// Create production configuration with monitoring
    pub fn production_defaults() -> Self {
        Self {
            learning: LearningConfig {
                enable_feedback_learning: true,
                enable_pattern_recognition: true,
                enable_optimization: true,
                adaptation_threshold: 0.8,
                max_data_age_days: 90,
                min_feedback_threshold: 10,
                pattern_frequency_threshold: 5,
                learning_rate: 0.05,
                storage: LearningStorageConfig {
                    collection_name: "learning_data_prod".to_string(),
                    enable_embeddings: true,
                    batch_size: 100,
                    retention_days: 365,
                },
                adaptation: AdaptationConfig {
                    enabled_algorithms: vec![
                        "feedback_analyzer".to_string(),
                        "pattern_matcher".to_string(),
                        "quality_optimizer".to_string(),
                    ],
                    algorithm_settings: HashMap::new(),
                    update_frequency_hours: 24,
                    auto_apply_adaptations: false, // Conservative for production
                },
            },
            monitoring: LearningMonitoringConfig {
                metrics_enabled: true,
                metrics_collection_interval_seconds: 60,
                performance_tracking_enabled: true,
                adaptation_tracking_enabled: true,
                storage_metrics_enabled: true,
                alert_thresholds: MonitoringThresholds {
                    max_adaptation_time_ms: 2000,
                    max_storage_response_time_ms: 500,
                    min_pattern_recognition_accuracy: 0.9,
                    max_memory_usage_mb: 1024,
                    max_cpu_usage_percent: 80.0,
                },
            },
            environment: "production".to_string(),
            health_checks: HealthCheckConfig {
                enabled: true,
                check_interval_seconds: 30,
                storage_health_check: true,
                adaptation_health_check: true,
                pattern_recognition_health_check: true,
                timeout_seconds: 10,
            },
            alerts: AlertConfig {
                enabled: true,
                error_rate_threshold: 0.02,
                response_time_threshold_ms: 3000,
                storage_error_threshold: 5,
                adaptation_failure_threshold: 3,
                notification_channels: vec!["logs".to_string(), "metrics".to_string()],
            },
        }
    }

    /// Create development configuration with monitoring
    pub fn development_defaults() -> Self {
        Self {
            learning: LearningConfig {
                enable_feedback_learning: true,
                enable_pattern_recognition: true,
                enable_optimization: false, // Safer for development
                adaptation_threshold: 0.6,
                max_data_age_days: 30,
                min_feedback_threshold: 3,
                pattern_frequency_threshold: 2,
                learning_rate: 0.1,
                storage: LearningStorageConfig {
                    collection_name: "learning_data_dev".to_string(),
                    enable_embeddings: true,
                    batch_size: 50,
                    retention_days: 90,
                },
                adaptation: AdaptationConfig {
                    enabled_algorithms: vec![
                        "feedback_analyzer".to_string(),
                        "pattern_matcher".to_string(),
                    ],
                    algorithm_settings: HashMap::new(),
                    update_frequency_hours: 12,
                    auto_apply_adaptations: false,
                },
            },
            monitoring: LearningMonitoringConfig {
                metrics_enabled: true,
                metrics_collection_interval_seconds: 30,
                performance_tracking_enabled: true,
                adaptation_tracking_enabled: true,
                storage_metrics_enabled: true,
                alert_thresholds: MonitoringThresholds {
                    max_adaptation_time_ms: 3000,
                    max_storage_response_time_ms: 1000,
                    min_pattern_recognition_accuracy: 0.8,
                    max_memory_usage_mb: 512,
                    max_cpu_usage_percent: 90.0,
                },
            },
            environment: "development".to_string(),
            health_checks: HealthCheckConfig {
                enabled: true,
                check_interval_seconds: 60,
                storage_health_check: true,
                adaptation_health_check: true,
                pattern_recognition_health_check: true,
                timeout_seconds: 15,
            },
            alerts: AlertConfig {
                enabled: false, // Less noisy in development
                error_rate_threshold: 0.1,
                response_time_threshold_ms: 5000,
                storage_error_threshold: 10,
                adaptation_failure_threshold: 5,
                notification_channels: vec!["logs".to_string()],
            },
        }
    }

    /// Create testing configuration with monitoring
    pub fn testing_defaults() -> Self {
        Self {
            learning: LearningConfig {
                enable_feedback_learning: true,
                enable_pattern_recognition: true,
                enable_optimization: false, // Disabled for predictable tests
                adaptation_threshold: 0.5,
                max_data_age_days: 7,
                min_feedback_threshold: 1,
                pattern_frequency_threshold: 1,
                learning_rate: 0.2,
                storage: LearningStorageConfig {
                    collection_name: "learning_data_test".to_string(),
                    enable_embeddings: false, // Faster for tests
                    batch_size: 10,
                    retention_days: 7,
                },
                adaptation: AdaptationConfig {
                    enabled_algorithms: vec!["feedback_analyzer".to_string()],
                    algorithm_settings: HashMap::new(),
                    update_frequency_hours: 1,
                    auto_apply_adaptations: false,
                },
            },
            monitoring: LearningMonitoringConfig {
                metrics_enabled: true,
                metrics_collection_interval_seconds: 10,
                performance_tracking_enabled: false, // Minimal for tests
                adaptation_tracking_enabled: false,
                storage_metrics_enabled: true,
                alert_thresholds: MonitoringThresholds {
                    max_adaptation_time_ms: 5000,
                    max_storage_response_time_ms: 2000,
                    min_pattern_recognition_accuracy: 0.7,
                    max_memory_usage_mb: 256,
                    max_cpu_usage_percent: 95.0,
                },
            },
            environment: "testing".to_string(),
            health_checks: HealthCheckConfig {
                enabled: true,
                check_interval_seconds: 15,
                storage_health_check: true,
                adaptation_health_check: false, // Minimal for tests
                pattern_recognition_health_check: false,
                timeout_seconds: 5,
            },
            alerts: AlertConfig {
                enabled: false, // No alerts in tests
                error_rate_threshold: 0.5,
                response_time_threshold_ms: 10000,
                storage_error_threshold: 20,
                adaptation_failure_threshold: 10,
                notification_channels: vec![],
            },
        }
    }

    /// Load from file with validation
    pub async fn from_file(path: &str) -> LearningResult<Self> {
        debug!(
            "Loading enhanced learning configuration from file: {}",
            path
        );

        let content = fs::read_to_string(path).await.map_err(|e| {
            LearningError::ConfigurationError(format!("Failed to read config file {path}: {e}"))
        })?;

        let config: Self = serde_json::from_str(&content).map_err(|e| {
            LearningError::ConfigurationError(format!("Failed to parse config file {path}: {e}"))
        })?;

        // Validate configuration
        config.validate()?;

        info!("Successfully loaded enhanced learning configuration from file");
        Ok(config)
    }

    /// Save to file
    pub async fn save_to_file(&self, path: &str) -> LearningResult<()> {
        debug!("Saving enhanced learning configuration to file: {}", path);

        // Validate before saving
        self.validate()?;

        let content = serde_json::to_string_pretty(self).map_err(|e| {
            LearningError::ConfigurationError(format!("Failed to serialize config: {e}"))
        })?;

        fs::write(path, content).await.map_err(|e| {
            LearningError::ConfigurationError(format!("Failed to write config file {path}: {e}"))
        })?;

        info!("Successfully saved enhanced learning configuration to file");
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> LearningResult<()> {
        debug!("Validating enhanced learning configuration");

        // Validate core learning configuration
        if self.learning.adaptation_threshold < 0.0 || self.learning.adaptation_threshold > 1.0 {
            return Err(LearningError::ConfigurationError(
                "Adaptation threshold must be between 0.0 and 1.0".to_string(),
            ));
        }

        if self.learning.learning_rate < 0.0 || self.learning.learning_rate > 1.0 {
            return Err(LearningError::ConfigurationError(
                "Learning rate must be between 0.0 and 1.0".to_string(),
            ));
        }

        if self.learning.max_data_age_days == 0 {
            return Err(LearningError::ConfigurationError(
                "Max data age days must be greater than 0".to_string(),
            ));
        }

        if self.learning.storage.batch_size == 0 {
            return Err(LearningError::ConfigurationError(
                "Storage batch size must be greater than 0".to_string(),
            ));
        }

        if self.learning.storage.retention_days == 0 {
            return Err(LearningError::ConfigurationError(
                "Storage retention days must be greater than 0".to_string(),
            ));
        }

        // Validate monitoring configuration
        if self.monitoring.metrics_collection_interval_seconds == 0 {
            return Err(LearningError::ConfigurationError(
                "Metrics collection interval must be greater than 0".to_string(),
            ));
        }

        if self
            .monitoring
            .alert_thresholds
            .min_pattern_recognition_accuracy
            < 0.0
            || self
                .monitoring
                .alert_thresholds
                .min_pattern_recognition_accuracy
                > 1.0
        {
            return Err(LearningError::ConfigurationError(
                "Pattern recognition accuracy threshold must be between 0.0 and 1.0".to_string(),
            ));
        }

        if self.monitoring.alert_thresholds.max_cpu_usage_percent < 0.0
            || self.monitoring.alert_thresholds.max_cpu_usage_percent > 100.0
        {
            return Err(LearningError::ConfigurationError(
                "CPU usage threshold must be between 0.0 and 100.0".to_string(),
            ));
        }

        // Validate health check configuration
        if self.health_checks.check_interval_seconds == 0 {
            return Err(LearningError::ConfigurationError(
                "Health check interval must be greater than 0".to_string(),
            ));
        }

        if self.health_checks.timeout_seconds == 0 {
            return Err(LearningError::ConfigurationError(
                "Health check timeout must be greater than 0".to_string(),
            ));
        }

        // Validate alert configuration
        if self.alerts.error_rate_threshold < 0.0 || self.alerts.error_rate_threshold > 1.0 {
            return Err(LearningError::ConfigurationError(
                "Error rate threshold must be between 0.0 and 1.0".to_string(),
            ));
        }

        if self.alerts.response_time_threshold_ms == 0 {
            return Err(LearningError::ConfigurationError(
                "Response time threshold must be greater than 0".to_string(),
            ));
        }

        debug!("Enhanced learning configuration validation passed");
        Ok(())
    }

    /// Apply environment variables
    pub fn apply_env_vars(&mut self) -> LearningResult<()> {
        debug!("Applying environment variables to enhanced learning configuration");

        // Core learning config environment variables
        if let Ok(enable_feedback) = env::var("LEARNING_ENABLE_FEEDBACK") {
            self.learning.enable_feedback_learning = parse_bool(&enable_feedback)?;
        }

        if let Ok(threshold) = env::var("LEARNING_ADAPTATION_THRESHOLD") {
            self.learning.adaptation_threshold = threshold.parse::<f64>().map_err(|e| {
                LearningError::ConfigurationError(format!("Invalid adaptation threshold: {e}"))
            })?;
        }

        // Monitoring environment variables
        if let Ok(metrics_enabled) = env::var("LEARNING_METRICS_ENABLED") {
            self.monitoring.metrics_enabled = parse_bool(&metrics_enabled)?;
        }

        if let Ok(interval) = env::var("LEARNING_METRICS_INTERVAL") {
            self.monitoring.metrics_collection_interval_seconds =
                interval.parse::<u64>().map_err(|e| {
                    LearningError::ConfigurationError(format!("Invalid metrics interval: {e}"))
                })?;
        }

        // Health check environment variables
        if let Ok(health_interval) = env::var("LEARNING_HEALTH_CHECK_INTERVAL") {
            self.health_checks.check_interval_seconds =
                health_interval.parse::<u64>().map_err(|e| {
                    LearningError::ConfigurationError(format!("Invalid health check interval: {e}"))
                })?;
        }

        // Alert environment variables
        if let Ok(error_threshold) = env::var("LEARNING_ALERT_ERROR_THRESHOLD") {
            self.alerts.error_rate_threshold = error_threshold.parse::<f64>().map_err(|e| {
                LearningError::ConfigurationError(format!("Invalid error threshold: {e}"))
            })?;
        }

        debug!("Successfully applied environment variables");
        Ok(())
    }
}

impl Default for LearningMonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            metrics_collection_interval_seconds: 60,
            performance_tracking_enabled: true,
            adaptation_tracking_enabled: true,
            storage_metrics_enabled: true,
            alert_thresholds: MonitoringThresholds::default(),
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_seconds: 30,
            storage_health_check: true,
            adaptation_health_check: true,
            pattern_recognition_health_check: true,
            timeout_seconds: 10,
        }
    }
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            error_rate_threshold: 0.05,
            response_time_threshold_ms: 5000,
            storage_error_threshold: 10,
            adaptation_failure_threshold: 5,
            notification_channels: vec!["logs".to_string()],
        }
    }
}

impl Default for MonitoringThresholds {
    fn default() -> Self {
        Self {
            max_adaptation_time_ms: 1000,
            max_storage_response_time_ms: 500,
            min_pattern_recognition_accuracy: 0.85,
            max_memory_usage_mb: 512,
            max_cpu_usage_percent: 80.0,
        }
    }
}

/// Parse boolean value from string
fn parse_bool(value: &str) -> LearningResult<bool> {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" | "enabled" => Ok(true),
        "false" | "0" | "no" | "off" | "disabled" => Ok(false),
        _ => Err(LearningError::ConfigurationError(format!(
            "Invalid boolean value: {value}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse_bool("true").unwrap(), true);
        assert_eq!(parse_bool("TRUE").unwrap(), true);
        assert!(parse_bool("1").unwrap());
        assert!(parse_bool("yes").unwrap());
        assert!(parse_bool("on").unwrap());
        assert!(parse_bool("enabled").unwrap());

        assert!(!parse_bool("false").unwrap());
        assert!(!parse_bool("FALSE").unwrap());
        assert!(!parse_bool("0").unwrap());
        assert!(!parse_bool("no").unwrap());
        assert!(!parse_bool("off").unwrap());
        assert!(!parse_bool("disabled").unwrap());

        assert!(parse_bool("invalid").is_err());
    }

    #[test]
    fn test_enhanced_config_validation() {
        let config = EnhancedLearningConfig::production_defaults();
        assert!(config.validate().is_ok());

        let mut invalid_config = config.clone();
        invalid_config.learning.adaptation_threshold = 1.5;
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_configuration_presets() {
        let prod_config = EnhancedLearningConfig::production_defaults();
        assert_eq!(prod_config.environment, "production");
        assert!(prod_config.learning.enable_optimization);
        assert!(prod_config.alerts.enabled);

        let dev_config = EnhancedLearningConfig::development_defaults();
        assert_eq!(dev_config.environment, "development");
        assert!(!dev_config.learning.enable_optimization);
        assert!(!dev_config.alerts.enabled);

        let test_config = EnhancedLearningConfig::testing_defaults();
        assert_eq!(test_config.environment, "testing");
        assert!(!test_config.learning.enable_optimization);
        assert!(!test_config.alerts.enabled);
    }

    #[tokio::test]
    async fn test_config_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.json");

        let original_config = EnhancedLearningConfig::production_defaults();

        // Save configuration
        original_config
            .save_to_file(config_path.to_str().unwrap())
            .await
            .unwrap();

        // Load configuration
        let loaded_config = EnhancedLearningConfig::from_file(config_path.to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(original_config.environment, loaded_config.environment);
        assert_eq!(
            original_config.learning.adaptation_threshold,
            loaded_config.learning.adaptation_threshold
        );
    }

    #[test]
    fn test_config_manager_creation() {
        let config = LearningConfig::default();
        let manager = LearningConfigManager::new(config.clone());

        assert_eq!(
            manager.config().enable_feedback_learning,
            config.enable_feedback_learning
        );
        assert_eq!(manager.enhanced_config().environment, "default");
    }

    #[test]
    fn test_environment_overrides() {
        let config = LearningConfig::default();
        let mut manager = LearningConfigManager::new(config);

        // Apply production overrides
        manager.apply_environment_overrides("production").unwrap();
        assert_eq!(manager.config().adaptation_threshold, 0.8);
        assert!(manager.config().enable_optimization);

        // Apply development overrides
        manager.apply_environment_overrides("development").unwrap();
        assert_eq!(manager.config().adaptation_threshold, 0.6);
        assert!(!manager.config().enable_optimization);
    }
}
