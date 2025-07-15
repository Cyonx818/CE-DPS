// ABOUTME: Comprehensive tests for learning system configuration management
//! This test module verifies that the learning system configuration provides:
//! - Environment variable support for configuration
//! - Configuration validation and error handling
//! - Integration with existing configuration patterns
//! - Health checks and monitoring setup
//! - Production and development configuration presets

use fortitude::learning::{
    AdaptationConfig, AlertConfig, ConfigWatcher, EnhancedLearningConfig, HealthCheckConfig,
    LearningConfig, LearningConfigManager, LearningError, LearningMonitoringConfig, LearningResult,
    LearningStorageConfig, MonitoringThresholds,
};
use serde_json;
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use tempfile::TempDir;
use tokio::fs;

/// Test configuration watcher for testing
struct TestConfigWatcher {
    notification_count: std::sync::Arc<std::sync::Mutex<u32>>,
}

impl TestConfigWatcher {
    fn new() -> Self {
        Self {
            notification_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
        }
    }

    fn get_notification_count(&self) -> u32 {
        *self.notification_count.lock().unwrap()
    }
}

#[async_trait::async_trait]
impl ConfigWatcher for TestConfigWatcher {
    async fn on_config_changed(
        &mut self,
        _old_config: &EnhancedLearningConfig,
        _new_config: &EnhancedLearningConfig,
    ) {
        let mut count = self.notification_count.lock().unwrap();
        *count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_learning_config_manager_creation() {
        let config = LearningConfig::default();
        let manager = LearningConfigManager::new(config.clone());

        assert_eq!(
            manager.config().enable_feedback_learning,
            config.enable_feedback_learning
        );
    }

    #[tokio::test]
    async fn test_environment_based_configuration() {
        // Set environment variables
        env::set_var("LEARNING_ENABLE_FEEDBACK", "false");
        env::set_var("LEARNING_ADAPTATION_THRESHOLD", "0.8");
        env::set_var("LEARNING_MAX_DATA_AGE_DAYS", "60");
        env::set_var("LEARNING_STORAGE_COLLECTION_NAME", "test_learning");

        let manager = LearningConfigManager::from_environment().await.unwrap();
        let config = manager.config();

        assert!(!config.enable_feedback_learning);
        assert_eq!(config.adaptation_threshold, 0.8);
        assert_eq!(config.max_data_age_days, 60);
        assert_eq!(config.storage.collection_name, "test_learning");

        // Clean up
        env::remove_var("LEARNING_ENABLE_FEEDBACK");
        env::remove_var("LEARNING_ADAPTATION_THRESHOLD");
        env::remove_var("LEARNING_MAX_DATA_AGE_DAYS");
        env::remove_var("LEARNING_STORAGE_COLLECTION_NAME");
    }

    #[tokio::test]
    async fn test_configuration_file_loading() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("learning_config.json");

        // Create test configuration
        let test_config = EnhancedLearningConfig::development_defaults();
        test_config
            .save_to_file(config_path.to_str().unwrap())
            .await
            .unwrap();

        // Load configuration
        let loaded_config = EnhancedLearningConfig::from_file(config_path.to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(loaded_config.environment, "development");
        assert!(loaded_config.monitoring.metrics_enabled);
        assert!(loaded_config.health_checks.enabled);
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        let mut config = EnhancedLearningConfig::production_defaults();

        // Valid configuration should pass
        assert!(config.validate().is_ok());

        // Invalid adaptation threshold should fail
        config.learning.adaptation_threshold = 1.5;
        assert!(config.validate().is_err());

        // Invalid monitoring interval should fail
        config.learning.adaptation_threshold = 0.7;
        config.monitoring.metrics_collection_interval_seconds = 0;
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn test_environment_specific_overrides() {
        let mut manager = LearningConfigManager::new(LearningConfig::default());

        // Apply production overrides
        manager.apply_environment_overrides("production").unwrap();
        let config = manager.config();

        // Production should have stricter settings
        assert!(config.adaptation_threshold >= 0.8);
        assert!(config.enable_optimization);

        // Apply development overrides
        manager.apply_environment_overrides("development").unwrap();
        let config = manager.config();

        // Development should have more relaxed settings
        assert!(config.adaptation_threshold <= 0.7);
    }

    #[tokio::test]
    async fn test_configuration_watchers() {
        let watcher = TestConfigWatcher::new();
        let count = watcher.notification_count.clone();

        let mut manager = LearningConfigManager::new(LearningConfig::default());
        manager.add_watcher(Box::new(watcher));

        // Update configuration should notify watchers
        let new_config = LearningConfig::default();
        manager.update_config(new_config).await.unwrap();

        let final_count = *count.lock().unwrap();
        assert_eq!(final_count, 1);
    }

    #[tokio::test]
    async fn test_production_configuration_defaults() {
        let config = EnhancedLearningConfig::production_defaults();

        // Production defaults should be secure and performant
        assert!(config.learning.enable_feedback_learning);
        assert!(config.learning.enable_pattern_recognition);
        assert!(config.learning.enable_optimization);
        assert!(config.learning.adaptation_threshold >= 0.8);
        assert!(config.monitoring.metrics_enabled);
        assert!(config.monitoring.performance_tracking_enabled);
        assert!(config.health_checks.enabled);
        assert!(config.alerts.enabled);
        assert_eq!(config.environment, "production");
    }

    #[tokio::test]
    async fn test_development_configuration_defaults() {
        let config = EnhancedLearningConfig::development_defaults();

        // Development defaults should be more permissive
        assert!(config.learning.enable_feedback_learning);
        assert!(config.learning.enable_pattern_recognition);
        assert!(!config.learning.enable_optimization); // Safer for development
        assert!(config.learning.adaptation_threshold <= 0.7);
        assert!(config.monitoring.metrics_enabled);
        assert_eq!(config.environment, "development");
    }

    #[tokio::test]
    async fn test_testing_configuration_defaults() {
        let config = EnhancedLearningConfig::testing_defaults();

        // Testing defaults should be fast and minimal
        assert!(!config.learning.enable_optimization);
        assert!(config.learning.adaptation_threshold <= 0.6);
        assert!(config.monitoring.metrics_collection_interval_seconds <= 30);
        assert!(config.health_checks.check_interval_seconds <= 15);
        assert_eq!(config.environment, "testing");
    }

    #[tokio::test]
    async fn test_monitoring_configuration() {
        let config = LearningMonitoringConfig::default();

        assert!(config.metrics_enabled);
        assert!(config.performance_tracking_enabled);
        assert!(config.adaptation_tracking_enabled);
        assert!(config.storage_metrics_enabled);
        assert!(config.metrics_collection_interval_seconds > 0);
        assert!(config.alert_thresholds.max_adaptation_time_ms > 0);
        assert!(config.alert_thresholds.max_storage_response_time_ms > 0);
        assert!(config.alert_thresholds.min_pattern_recognition_accuracy > 0.0);
    }

    #[tokio::test]
    async fn test_health_check_configuration() {
        let config = HealthCheckConfig::default();

        assert!(config.enabled);
        assert!(config.storage_health_check);
        assert!(config.adaptation_health_check);
        assert!(config.pattern_recognition_health_check);
        assert!(config.check_interval_seconds > 0);
        assert!(config.timeout_seconds > 0);
    }

    #[tokio::test]
    async fn test_alert_configuration() {
        let config = AlertConfig::default();

        assert!(config.enabled);
        assert!(config.error_rate_threshold > 0.0);
        assert!(config.error_rate_threshold < 1.0);
        assert!(config.response_time_threshold_ms > 0);
        assert!(config.storage_error_threshold > 0);
        assert!(config.adaptation_failure_threshold > 0);
        assert!(!config.notification_channels.is_empty());
    }

    #[tokio::test]
    async fn test_environment_variable_application() {
        let mut config = EnhancedLearningConfig::development_defaults();

        // Set environment variables
        env::set_var("LEARNING_METRICS_ENABLED", "false");
        env::set_var("LEARNING_HEALTH_CHECK_INTERVAL", "120");
        env::set_var("LEARNING_ALERT_ERROR_THRESHOLD", "0.1");

        config.apply_env_vars().unwrap();

        assert!(!config.monitoring.metrics_enabled);
        assert_eq!(config.health_checks.check_interval_seconds, 120);
        assert_eq!(config.alerts.error_rate_threshold, 0.1);

        // Clean up
        env::remove_var("LEARNING_METRICS_ENABLED");
        env::remove_var("LEARNING_HEALTH_CHECK_INTERVAL");
        env::remove_var("LEARNING_ALERT_ERROR_THRESHOLD");
    }

    #[tokio::test]
    async fn test_configuration_serialization() {
        let config = EnhancedLearningConfig::production_defaults();

        // Serialize and deserialize
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: EnhancedLearningConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.environment, deserialized.environment);
        assert_eq!(
            config.learning.adaptation_threshold,
            deserialized.learning.adaptation_threshold
        );
        assert_eq!(
            config.monitoring.metrics_enabled,
            deserialized.monitoring.metrics_enabled
        );
    }

    #[tokio::test]
    async fn test_configuration_validation_comprehensive() {
        let mut config = EnhancedLearningConfig::development_defaults();

        // Test various invalid configurations

        // Invalid adaptation threshold
        config.learning.adaptation_threshold = -0.1;
        assert!(config.validate().is_err());

        config.learning.adaptation_threshold = 1.1;
        assert!(config.validate().is_err());

        // Invalid learning rate
        config.learning.adaptation_threshold = 0.7;
        config.learning.learning_rate = -0.1;
        assert!(config.validate().is_err());

        config.learning.learning_rate = 1.1;
        assert!(config.validate().is_err());

        // Invalid monitoring intervals
        config.learning.learning_rate = 0.1;
        config.monitoring.metrics_collection_interval_seconds = 0;
        assert!(config.validate().is_err());

        // Invalid health check intervals
        config.monitoring.metrics_collection_interval_seconds = 60;
        config.health_checks.check_interval_seconds = 0;
        assert!(config.validate().is_err());

        // Invalid alert thresholds
        config.health_checks.check_interval_seconds = 30;
        config.alerts.error_rate_threshold = -0.1;
        assert!(config.validate().is_err());

        config.alerts.error_rate_threshold = 1.1;
        assert!(config.validate().is_err());

        // Valid configuration should pass
        config.alerts.error_rate_threshold = 0.05;
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn test_file_loading_with_environment_overrides() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        // Create base configuration file
        let base_config = EnhancedLearningConfig::development_defaults();
        base_config
            .save_to_file(config_path.to_str().unwrap())
            .await
            .unwrap();

        // Set environment variables to override
        env::set_var("LEARNING_ADAPTATION_THRESHOLD", "0.9");
        env::set_var("LEARNING_METRICS_INTERVAL", "30");

        let manager = LearningConfigManager::from_file_with_env(config_path.to_str().unwrap())
            .await
            .unwrap();
        let config = manager.config();

        // Should have environment overrides applied
        assert_eq!(config.adaptation_threshold, 0.9);

        // Clean up
        env::remove_var("LEARNING_ADAPTATION_THRESHOLD");
        env::remove_var("LEARNING_METRICS_INTERVAL");
    }
}
