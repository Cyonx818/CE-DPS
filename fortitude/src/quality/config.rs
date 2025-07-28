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

// ABOUTME: Centralized configuration management for all quality control systems
//! This module provides comprehensive configuration management for the entire
//! quality control ecosystem, including scoring, cross-validation, feedback,
//! metrics collection, and optimization systems.
//!
//! # Key Components
//! - `QualityControlConfig`: Master configuration for all quality systems
//! - `QualitySubsystemConfigs`: Individual subsystem configurations
//! - `QualityConfigManager`: Runtime configuration management
//! - `ConfigValidation`: Configuration validation and defaults
//!
//! # Integration
//! This configuration system is designed to integrate with all Task 2 components:
//! - Quality Scoring (Task 2.1)
//! - Cross-Validation (Task 2.2)
//! - User Feedback Learning (Task 2.3)
//! - Metrics Collection (Task 2.4)
//! - Provider Selection Optimization (Task 2.5)
//!
//! # Example Usage
//!
//! ```rust
//! use fortitude::quality::config::QualityControlConfig;
//!
//! // Load configuration from file
//! let config = QualityControlConfig::from_file("quality.yaml").await?;
//!
//! // Get scoring configuration
//! let scoring_config = config.scoring();
//!
//! // Get cross-validation configuration
//! let validation_config = config.cross_validation();
//!
//! // Apply runtime updates
//! config.update_quality_threshold(0.95).await?;
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use thiserror::Error;
use tokio::fs;

use crate::quality::{
    cross_validation::CrossValidationConfig,
    feedback::{FeedbackCollectionConfig, QualityLearningConfig},
    metrics::MetricsConfig,
    optimization::OptimizationConfig,
    scoring::ScorerConfig,
    QualityWeights,
};

/// Master configuration for entire quality control system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityControlConfig {
    /// Global quality control settings
    pub global: GlobalQualityConfig,
    /// Quality scoring configuration
    pub scoring: ScorerConfig,
    /// Cross-validation configuration
    pub cross_validation: CrossValidationConfig,
    /// User feedback system configuration
    pub feedback: FeedbackSystemConfig,
    /// Metrics collection configuration
    pub metrics: MetricsConfig,
    /// Provider selection optimization configuration
    pub optimization: OptimizationConfig,
    /// Environment-specific overrides
    pub environment_overrides: HashMap<String, EnvironmentConfig>,
}

/// Global quality control settings that apply across all subsystems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalQualityConfig {
    /// Overall quality target (e.g., 0.95 for >95% accuracy)
    pub quality_target: f64,
    /// Default quality weights for all operations
    pub default_weights: QualityWeights,
    /// Enable/disable quality control system-wide
    pub enabled: bool,
    /// Strict mode enforces higher quality standards
    pub strict_mode: bool,
    /// Default timeout for quality operations
    pub operation_timeout: Duration,
    /// Maximum concurrent quality operations
    pub max_concurrent_operations: usize,
    /// Enable debug logging for quality operations
    pub debug_logging: bool,
    /// Quality control version for compatibility
    pub version: String,
}

/// Combined feedback and learning system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackSystemConfig {
    /// Feedback collection configuration
    pub collection: FeedbackCollectionConfig,
    /// Learning algorithm configuration
    pub learning: QualityLearningConfig,
    /// Privacy settings for feedback data
    pub privacy: FeedbackPrivacyConfig,
    /// A/B testing configuration
    pub ab_testing: ABTestingConfig,
}

/// Privacy configuration for feedback handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackPrivacyConfig {
    /// Enable anonymization of user feedback
    pub anonymize_users: bool,
    /// Data retention period for feedback
    pub retention_days: u32,
    /// Enable encryption of stored feedback
    pub encrypt_storage: bool,
    /// Hash user identifiers
    pub hash_identifiers: bool,
    /// Compliance settings
    pub compliance: ComplianceConfig,
}

/// A/B testing configuration for quality experiments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestingConfig {
    /// Enable A/B testing system
    pub enabled: bool,
    /// Default test duration in days
    pub default_duration_days: u32,
    /// Minimum sample size for valid tests
    pub min_sample_size: usize,
    /// Statistical significance threshold
    pub significance_threshold: f64,
    /// Maximum concurrent A/B tests
    pub max_concurrent_tests: usize,
}

/// Compliance configuration for regulatory requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// GDPR compliance mode
    pub gdpr_enabled: bool,
    /// CCPA compliance mode
    pub ccpa_enabled: bool,
    /// Data processing agreement references
    pub dpa_references: Vec<String>,
    /// Audit logging requirements
    pub audit_logging: bool,
}

/// Environment-specific configuration overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Quality target override for this environment
    pub quality_target_override: Option<f64>,
    /// Enable/disable features per environment
    pub feature_flags: HashMap<String, bool>,
    /// Resource limits for this environment
    pub resource_limits: ResourceLimits,
    /// Monitoring configuration overrides
    pub monitoring_overrides: MonitoringConfig,
}

/// Resource limits for quality control operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: usize,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: f64,
    /// Maximum disk usage in MB
    pub max_disk_mb: usize,
    /// Maximum network bandwidth in Mbps
    pub max_network_mbps: f64,
}

/// Monitoring configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable metrics collection
    pub metrics_enabled: bool,
    /// Metrics collection interval
    pub collection_interval: Duration,
    /// Enable distributed tracing
    pub tracing_enabled: bool,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Quality score threshold for alerts
    pub quality_threshold: f64,
    /// Response time threshold in milliseconds
    pub response_time_ms: u64,
    /// Error rate threshold (0.0 to 1.0)
    pub error_rate_threshold: f64,
    /// Memory usage threshold in MB
    pub memory_threshold_mb: usize,
}

impl QualityControlConfig {
    /// Create a new configuration with production defaults
    pub fn production_defaults() -> Self {
        Self {
            global: GlobalQualityConfig::production_defaults(),
            scoring: ScorerConfig::production_optimized(),
            cross_validation: CrossValidationConfig::production_defaults(),
            feedback: FeedbackSystemConfig::production_defaults(),
            metrics: MetricsConfig::production_optimized(),
            optimization: OptimizationConfig::production_optimized(),
            environment_overrides: HashMap::new(),
        }
    }

    /// Create a new configuration with development defaults
    pub fn development_defaults() -> Self {
        Self {
            global: GlobalQualityConfig::development_defaults(),
            scoring: ScorerConfig::development_optimized(),
            cross_validation: CrossValidationConfig::development_defaults(),
            feedback: FeedbackSystemConfig::development_defaults(),
            metrics: MetricsConfig::development_optimized(),
            optimization: OptimizationConfig::development_optimized(),
            environment_overrides: HashMap::new(),
        }
    }

    /// Load configuration from YAML file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> ConfigResult<Self> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| ConfigError::FileError {
                message: e.to_string(),
            })?;

        let config: Self = serde_yaml::from_str(&content).map_err(|e| ConfigError::ParseError {
            message: e.to_string(),
        })?;

        config.validate()?;
        Ok(config)
    }

    /// Save configuration to YAML file
    pub async fn save_to_file<P: AsRef<Path>>(&self, path: P) -> ConfigResult<()> {
        self.validate()?;

        let yaml = serde_yaml::to_string(self).map_err(|e| ConfigError::SerializationError {
            message: e.to_string(),
        })?;

        fs::write(path, yaml)
            .await
            .map_err(|e| ConfigError::FileError {
                message: e.to_string(),
            })?;

        Ok(())
    }

    /// Validate the configuration for consistency and correctness
    pub fn validate(&self) -> ConfigResult<()> {
        // Validate global settings
        if self.global.quality_target < 0.0 || self.global.quality_target > 1.0 {
            return Err(ConfigError::ValidationError {
                message: "Quality target must be between 0.0 and 1.0".to_string(),
            });
        }

        if !self.global.default_weights.is_valid() {
            return Err(ConfigError::ValidationError {
                message: "Default quality weights must sum to 1.0".to_string(),
            });
        }

        if self.global.max_concurrent_operations == 0 {
            return Err(ConfigError::ValidationError {
                message: "Max concurrent operations must be greater than 0".to_string(),
            });
        }

        // Validate scoring configuration
        self.scoring
            .validate()
            .map_err(|e| ConfigError::ValidationError {
                message: format!("Scoring config: {e}"),
            })?;

        // Validate cross-validation configuration
        self.cross_validation
            .validate()
            .map_err(|e| ConfigError::ValidationError {
                message: format!("Cross-validation config: {e}"),
            })?;

        // Validate metrics configuration
        self.metrics
            .validate()
            .map_err(|e| ConfigError::ValidationError {
                message: format!("Metrics config: {e}"),
            })?;

        // Validate optimization configuration
        self.optimization
            .validate()
            .map_err(|e| ConfigError::ValidationError {
                message: format!("Optimization config: {e}"),
            })?;

        Ok(())
    }

    /// Get configuration for specific environment
    pub fn for_environment(&self, env: &str) -> Self {
        let mut config = self.clone();

        if let Some(env_override) = self.environment_overrides.get(env) {
            if let Some(quality_override) = env_override.quality_target_override {
                config.global.quality_target = quality_override;
            }

            // Apply feature flags
            for (feature, enabled) in &env_override.feature_flags {
                match feature.as_str() {
                    "strict_mode" => config.global.strict_mode = *enabled,
                    "debug_logging" => config.global.debug_logging = *enabled,
                    "cross_validation" => config.cross_validation.enabled = *enabled,
                    "feedback_collection" => config.feedback.collection.enabled = *enabled,
                    "metrics_collection" => config.metrics.enabled = *enabled,
                    _ => {} // Unknown feature flags are ignored
                }
            }
        }

        config
    }

    /// Update quality target at runtime
    pub fn update_quality_target(&mut self, target: f64) -> ConfigResult<()> {
        if !(0.0..=1.0).contains(&target) {
            return Err(ConfigError::ValidationError {
                message: "Quality target must be between 0.0 and 1.0".to_string(),
            });
        }

        self.global.quality_target = target;
        Ok(())
    }

    /// Update quality weights at runtime
    pub fn update_quality_weights(&mut self, weights: QualityWeights) -> ConfigResult<()> {
        if !weights.is_valid() {
            return Err(ConfigError::ValidationError {
                message: "Quality weights must sum to 1.0".to_string(),
            });
        }

        self.global.default_weights = weights;
        Ok(())
    }

    /// Enable or disable strict mode
    pub fn set_strict_mode(&mut self, enabled: bool) {
        self.global.strict_mode = enabled;

        // Update related configurations for strict mode
        if enabled {
            self.cross_validation.agreement_threshold = 0.9;
            self.optimization.selection_criteria.quality_priority = 0.95;
            self.feedback.learning.conservative_updates = true;
        }
    }

    /// Get effective configuration combining global and subsystem settings
    pub fn effective_config(&self) -> EffectiveConfig {
        EffectiveConfig {
            quality_target: self.global.quality_target,
            quality_weights: self.global.default_weights.clone(),
            timeout: self.global.operation_timeout,
            max_concurrent: self.global.max_concurrent_operations,
            strict_mode: self.global.strict_mode,
            cross_validation_enabled: self.cross_validation.enabled,
            feedback_enabled: self.feedback.collection.enabled,
            metrics_enabled: self.metrics.enabled,
            optimization_enabled: self.optimization.enabled,
        }
    }
}

/// Effective configuration combining all settings for runtime use
#[derive(Debug, Clone)]
pub struct EffectiveConfig {
    pub quality_target: f64,
    pub quality_weights: QualityWeights,
    pub timeout: Duration,
    pub max_concurrent: usize,
    pub strict_mode: bool,
    pub cross_validation_enabled: bool,
    pub feedback_enabled: bool,
    pub metrics_enabled: bool,
    pub optimization_enabled: bool,
}

impl GlobalQualityConfig {
    pub fn production_defaults() -> Self {
        Self {
            quality_target: 0.95,
            default_weights: QualityWeights::research_optimized(),
            enabled: true,
            strict_mode: true,
            operation_timeout: Duration::from_secs(30),
            max_concurrent_operations: 100,
            debug_logging: false,
            version: "1.0.0".to_string(),
        }
    }

    pub fn development_defaults() -> Self {
        Self {
            quality_target: 0.85,
            default_weights: QualityWeights::new(),
            enabled: true,
            strict_mode: false,
            operation_timeout: Duration::from_secs(60),
            max_concurrent_operations: 10,
            debug_logging: true,
            version: "1.0.0".to_string(),
        }
    }
}

impl FeedbackSystemConfig {
    pub fn production_defaults() -> Self {
        Self {
            collection: FeedbackCollectionConfig::production_optimized(),
            learning: QualityLearningConfig::production_optimized(),
            privacy: FeedbackPrivacyConfig::production_defaults(),
            ab_testing: ABTestingConfig::production_defaults(),
        }
    }

    pub fn development_defaults() -> Self {
        Self {
            collection: FeedbackCollectionConfig::development_optimized(),
            learning: QualityLearningConfig::development_optimized(),
            privacy: FeedbackPrivacyConfig::development_defaults(),
            ab_testing: ABTestingConfig::development_defaults(),
        }
    }
}

impl FeedbackPrivacyConfig {
    pub fn production_defaults() -> Self {
        Self {
            anonymize_users: true,
            retention_days: 365,
            encrypt_storage: true,
            hash_identifiers: true,
            compliance: ComplianceConfig::production_defaults(),
        }
    }

    pub fn development_defaults() -> Self {
        Self {
            anonymize_users: false,
            retention_days: 30,
            encrypt_storage: false,
            hash_identifiers: false,
            compliance: ComplianceConfig::development_defaults(),
        }
    }
}

impl ABTestingConfig {
    pub fn production_defaults() -> Self {
        Self {
            enabled: true,
            default_duration_days: 14,
            min_sample_size: 1000,
            significance_threshold: 0.05,
            max_concurrent_tests: 5,
        }
    }

    pub fn development_defaults() -> Self {
        Self {
            enabled: true,
            default_duration_days: 7,
            min_sample_size: 100,
            significance_threshold: 0.1,
            max_concurrent_tests: 2,
        }
    }
}

impl ComplianceConfig {
    pub fn production_defaults() -> Self {
        Self {
            gdpr_enabled: true,
            ccpa_enabled: true,
            dpa_references: vec!["DPA-2024-001".to_string()],
            audit_logging: true,
        }
    }

    pub fn development_defaults() -> Self {
        Self {
            gdpr_enabled: false,
            ccpa_enabled: false,
            dpa_references: vec![],
            audit_logging: false,
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 1024, // 1GB
            max_cpu_percent: 80.0,
            max_disk_mb: 5120, // 5GB
            max_network_mbps: 100.0,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            collection_interval: Duration::from_secs(60),
            tracing_enabled: true,
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            quality_threshold: 0.85,
            response_time_ms: 5000,
            error_rate_threshold: 0.05,
            memory_threshold_mb: 512,
        }
    }
}

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("File operation failed: {message}")]
    FileError { message: String },

    #[error("Configuration parsing failed: {message}")]
    ParseError { message: String },

    #[error("Configuration validation failed: {message}")]
    ValidationError { message: String },

    #[error("Serialization failed: {message}")]
    SerializationError { message: String },

    #[error("Environment configuration not found: {environment}")]
    EnvironmentNotFound { environment: String },

    #[error("Invalid configuration value: {field} = {value}")]
    InvalidValue { field: String, value: String },

    #[error("Configuration compatibility error: {message}")]
    CompatibilityError { message: String },
}

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Configuration manager for runtime configuration management
pub struct QualityConfigManager {
    config: QualityControlConfig,
    watchers: Vec<Box<dyn ConfigWatcher>>,
}

impl QualityConfigManager {
    /// Create a new configuration manager
    pub fn new(config: QualityControlConfig) -> Self {
        Self {
            config,
            watchers: Vec::new(),
        }
    }

    /// Load configuration from file and create manager
    pub async fn from_file<P: AsRef<Path>>(path: P) -> ConfigResult<Self> {
        let config = QualityControlConfig::from_file(path).await?;
        Ok(Self::new(config))
    }

    /// Get current configuration
    pub fn config(&self) -> &QualityControlConfig {
        &self.config
    }

    /// Update configuration and notify watchers
    pub async fn update_config(&mut self, new_config: QualityControlConfig) -> ConfigResult<()> {
        new_config.validate()?;

        let old_config = self.config.clone();
        self.config = new_config;

        // Notify all watchers of configuration change
        for watcher in &mut self.watchers {
            watcher.on_config_changed(&old_config, &self.config).await;
        }

        Ok(())
    }

    /// Add a configuration watcher
    pub fn add_watcher(&mut self, watcher: Box<dyn ConfigWatcher>) {
        self.watchers.push(watcher);
    }

    /// Reload configuration from file
    pub async fn reload_from_file<P: AsRef<Path>>(&mut self, path: P) -> ConfigResult<()> {
        let new_config = QualityControlConfig::from_file(path).await?;
        self.update_config(new_config).await
    }

    /// Get configuration for specific environment
    pub fn config_for_environment(&self, env: &str) -> QualityControlConfig {
        self.config.for_environment(env)
    }
}

/// Trait for configuration change notifications
#[async_trait::async_trait]
pub trait ConfigWatcher: Send + Sync {
    /// Called when configuration changes
    async fn on_config_changed(
        &mut self,
        old_config: &QualityControlConfig,
        new_config: &QualityControlConfig,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_production_config_creation() {
        let config = QualityControlConfig::production_defaults();
        assert!(config.validate().is_ok());
        assert_eq!(config.global.quality_target, 0.95);
        assert!(config.global.strict_mode);
        assert!(config.global.enabled);
    }

    #[test]
    fn test_development_config_creation() {
        let config = QualityControlConfig::development_defaults();
        assert!(config.validate().is_ok());
        assert_eq!(config.global.quality_target, 0.85);
        assert!(!config.global.strict_mode);
        assert!(config.global.debug_logging);
    }

    #[test]
    fn test_config_validation() {
        let mut config = QualityControlConfig::production_defaults();

        // Valid configuration should pass
        assert!(config.validate().is_ok());

        // Invalid quality target should fail
        config.global.quality_target = 1.5;
        assert!(config.validate().is_err());

        // Invalid max concurrent operations should fail
        config.global.quality_target = 0.95;
        config.global.max_concurrent_operations = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_quality_target_update() {
        let mut config = QualityControlConfig::production_defaults();

        // Valid update should succeed
        assert!(config.update_quality_target(0.98).is_ok());
        assert_eq!(config.global.quality_target, 0.98);

        // Invalid update should fail
        assert!(config.update_quality_target(1.5).is_err());
        assert_eq!(config.global.quality_target, 0.98); // Should remain unchanged
    }

    #[test]
    fn test_quality_weights_update() {
        let mut config = QualityControlConfig::production_defaults();
        let weights = QualityWeights::fact_checking_optimized();

        assert!(config.update_quality_weights(weights.clone()).is_ok());
        assert_eq!(config.global.default_weights, weights);
    }

    #[test]
    fn test_strict_mode_effects() {
        let mut config = QualityControlConfig::development_defaults();

        // Enable strict mode
        config.set_strict_mode(true);
        assert!(config.global.strict_mode);
        assert_eq!(config.cross_validation.agreement_threshold, 0.9);
        assert_eq!(
            config.optimization.selection_criteria.quality_priority,
            0.95
        );
    }

    #[test]
    fn test_environment_overrides() {
        let mut config = QualityControlConfig::production_defaults();

        // Add environment override
        let mut env_override = EnvironmentConfig {
            quality_target_override: Some(0.98),
            feature_flags: HashMap::new(),
            resource_limits: ResourceLimits::default(),
            monitoring_overrides: MonitoringConfig::default(),
        };
        env_override
            .feature_flags
            .insert("debug_logging".to_string(), true);

        config
            .environment_overrides
            .insert("staging".to_string(), env_override);

        // Get environment-specific config
        let staging_config = config.for_environment("staging");
        assert_eq!(staging_config.global.quality_target, 0.98);
        assert!(staging_config.global.debug_logging);
    }

    #[test]
    fn test_effective_config() {
        let config = QualityControlConfig::production_defaults();
        let effective = config.effective_config();

        assert_eq!(effective.quality_target, 0.95);
        assert!(effective.strict_mode);
        assert!(effective.cross_validation_enabled);
        assert!(effective.feedback_enabled);
        assert!(effective.metrics_enabled);
    }

    #[tokio::test]
    async fn test_config_file_operations() {
        let config = QualityControlConfig::production_defaults();

        // Create temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();

        // Save configuration
        assert!(config.save_to_file(temp_path).await.is_ok());

        // Load configuration
        let loaded_config = QualityControlConfig::from_file(temp_path).await.unwrap();
        assert_eq!(
            loaded_config.global.quality_target,
            config.global.quality_target
        );
        assert_eq!(loaded_config.global.strict_mode, config.global.strict_mode);
    }

    #[tokio::test]
    async fn test_config_manager() {
        let initial_config = QualityControlConfig::development_defaults();
        let mut manager = QualityConfigManager::new(initial_config.clone());

        assert_eq!(manager.config().global.quality_target, 0.85);

        // Update configuration
        let new_config = QualityControlConfig::production_defaults();
        assert!(manager.update_config(new_config).await.is_ok());
        assert_eq!(manager.config().global.quality_target, 0.95);
    }

    #[test]
    fn test_privacy_config_defaults() {
        let prod_privacy = FeedbackPrivacyConfig::production_defaults();
        assert!(prod_privacy.anonymize_users);
        assert!(prod_privacy.encrypt_storage);
        assert_eq!(prod_privacy.retention_days, 365);

        let dev_privacy = FeedbackPrivacyConfig::development_defaults();
        assert!(!dev_privacy.anonymize_users);
        assert!(!dev_privacy.encrypt_storage);
        assert_eq!(dev_privacy.retention_days, 30);
    }

    #[test]
    fn test_ab_testing_config() {
        let prod_ab = ABTestingConfig::production_defaults();
        assert!(prod_ab.enabled);
        assert_eq!(prod_ab.min_sample_size, 1000);
        assert_eq!(prod_ab.significance_threshold, 0.05);

        let dev_ab = ABTestingConfig::development_defaults();
        assert_eq!(dev_ab.min_sample_size, 100);
        assert_eq!(dev_ab.significance_threshold, 0.1);
    }
}
