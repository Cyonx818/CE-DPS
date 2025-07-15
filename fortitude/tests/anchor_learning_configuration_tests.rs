// ABOUTME: Anchor tests for critical learning system configuration functionality
//! These tests ensure that critical configuration functionality continues to work
//! correctly even as the system evolves. They test the most important configuration
//! behaviors that must not break:
//! - Environment variable parsing and override behavior
//! - Configuration validation for security and correctness
//! - Production vs development configuration differences
//! - File-based configuration with environment overrides
//! - Configuration change notification system

use fortitude::learning::{
    AlertConfig, ConfigWatcher, EnhancedLearningConfig, HealthCheckConfig, LearningConfig,
    LearningConfigManager, LearningError, LearningMonitoringConfig, LearningResult,
    MonitoringThresholds,
};
use serde_json;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

/// Test configuration watcher for anchor tests
struct AnchorConfigWatcher {
    change_count: Arc<Mutex<u32>>,
    last_old_config: Arc<Mutex<Option<EnhancedLearningConfig>>>,
    last_new_config: Arc<Mutex<Option<EnhancedLearningConfig>>>,
}

impl AnchorConfigWatcher {
    fn new() -> Self {
        Self {
            change_count: Arc::new(Mutex::new(0)),
            last_old_config: Arc::new(Mutex::new(None)),
            last_new_config: Arc::new(Mutex::new(None)),
        }
    }

    fn get_change_count(&self) -> u32 {
        *self.change_count.lock().unwrap()
    }

    fn get_last_configs(
        &self,
    ) -> (
        Option<EnhancedLearningConfig>,
        Option<EnhancedLearningConfig>,
    ) {
        let old = self.last_old_config.lock().unwrap().clone();
        let new = self.last_new_config.lock().unwrap().clone();
        (old, new)
    }
}

#[async_trait::async_trait]
impl ConfigWatcher for AnchorConfigWatcher {
    async fn on_config_changed(
        &mut self,
        old_config: &EnhancedLearningConfig,
        new_config: &EnhancedLearningConfig,
    ) {
        let mut count = self.change_count.lock().unwrap();
        *count += 1;

        let mut old = self.last_old_config.lock().unwrap();
        *old = Some(old_config.clone());

        let mut new = self.last_new_config.lock().unwrap();
        *new = Some(new_config.clone());
    }
}

#[cfg(test)]
mod anchor_tests {
    use super::*;

    /// ANCHOR: Environment variable parsing must work consistently
    /// This test ensures that environment variables are correctly parsed and applied
    /// to configuration settings, with proper type conversion and error handling.
    #[tokio::test]
    async fn anchor_environment_variable_parsing() {
        // Test boolean parsing
        env::set_var("LEARNING_ENABLE_FEEDBACK", "true");
        env::set_var("LEARNING_ENABLE_PATTERN_RECOGNITION", "false");
        env::set_var("LEARNING_ENABLE_OPTIMIZATION", "1");

        // Test numeric parsing
        env::set_var("LEARNING_ADAPTATION_THRESHOLD", "0.85");
        env::set_var("LEARNING_MAX_DATA_AGE_DAYS", "120");
        env::set_var("LEARNING_MIN_FEEDBACK_THRESHOLD", "15");
        env::set_var("LEARNING_PATTERN_FREQUENCY_THRESHOLD", "8");
        env::set_var("LEARNING_RATE", "0.15");

        // Test string parsing
        env::set_var(
            "LEARNING_STORAGE_COLLECTION_NAME",
            "test_learning_collection",
        );
        env::set_var("LEARNING_ENVIRONMENT", "test_environment");

        // Test monitoring settings
        env::set_var("LEARNING_METRICS_ENABLED", "true");
        env::set_var("LEARNING_METRICS_INTERVAL", "45");
        env::set_var("LEARNING_HEALTH_CHECK_INTERVAL", "90");
        env::set_var("LEARNING_ALERT_ERROR_THRESHOLD", "0.02");

        let manager = LearningConfigManager::from_environment().await.unwrap();
        let config = manager.config();
        let enhanced_config = manager.enhanced_config();

        // Verify boolean values
        assert!(config.enable_feedback_learning);
        assert!(!config.enable_pattern_recognition);
        assert!(config.enable_optimization);

        // Verify numeric values
        assert_eq!(config.adaptation_threshold, 0.85);
        assert_eq!(config.max_data_age_days, 120);
        assert_eq!(config.min_feedback_threshold, 15);
        assert_eq!(config.pattern_frequency_threshold, 8);
        assert_eq!(config.learning_rate, 0.15);

        // Verify string values
        assert_eq!(config.storage.collection_name, "test_learning_collection");
        assert_eq!(enhanced_config.environment, "test_environment");

        // Verify monitoring settings
        assert!(enhanced_config.monitoring.metrics_enabled);
        assert_eq!(
            enhanced_config
                .monitoring
                .metrics_collection_interval_seconds,
            45
        );
        assert_eq!(enhanced_config.health_checks.check_interval_seconds, 90);
        assert_eq!(enhanced_config.alerts.error_rate_threshold, 0.02);

        // Clean up environment variables
        env::remove_var("LEARNING_ENABLE_FEEDBACK");
        env::remove_var("LEARNING_ENABLE_PATTERN_RECOGNITION");
        env::remove_var("LEARNING_ENABLE_OPTIMIZATION");
        env::remove_var("LEARNING_ADAPTATION_THRESHOLD");
        env::remove_var("LEARNING_MAX_DATA_AGE_DAYS");
        env::remove_var("LEARNING_MIN_FEEDBACK_THRESHOLD");
        env::remove_var("LEARNING_PATTERN_FREQUENCY_THRESHOLD");
        env::remove_var("LEARNING_RATE");
        env::remove_var("LEARNING_STORAGE_COLLECTION_NAME");
        env::remove_var("LEARNING_ENVIRONMENT");
        env::remove_var("LEARNING_METRICS_ENABLED");
        env::remove_var("LEARNING_METRICS_INTERVAL");
        env::remove_var("LEARNING_HEALTH_CHECK_INTERVAL");
        env::remove_var("LEARNING_ALERT_ERROR_THRESHOLD");
    }

    /// ANCHOR: Configuration validation must catch invalid values
    /// This test ensures that configuration validation properly catches and rejects
    /// invalid configuration values that could cause system instability or security issues.
    #[tokio::test]
    async fn anchor_configuration_validation_enforcement() {
        let mut config = EnhancedLearningConfig::production_defaults();

        // Valid configuration should pass
        assert!(
            config.validate().is_ok(),
            "Valid production config should pass validation"
        );

        // Test adaptation threshold bounds
        config.learning.adaptation_threshold = -0.1;
        assert!(
            config.validate().is_err(),
            "Negative adaptation threshold should be rejected"
        );

        config.learning.adaptation_threshold = 1.1;
        assert!(
            config.validate().is_err(),
            "Adaptation threshold > 1.0 should be rejected"
        );

        config.learning.adaptation_threshold = 0.8; // Reset to valid value

        // Test learning rate bounds
        config.learning.learning_rate = -0.1;
        assert!(
            config.validate().is_err(),
            "Negative learning rate should be rejected"
        );

        config.learning.learning_rate = 1.1;
        assert!(
            config.validate().is_err(),
            "Learning rate > 1.0 should be rejected"
        );

        config.learning.learning_rate = 0.1; // Reset to valid value

        // Test zero values that should be rejected
        config.learning.max_data_age_days = 0;
        assert!(
            config.validate().is_err(),
            "Zero max data age should be rejected"
        );

        config.learning.max_data_age_days = 90; // Reset

        config.learning.storage.batch_size = 0;
        assert!(
            config.validate().is_err(),
            "Zero batch size should be rejected"
        );

        config.learning.storage.batch_size = 100; // Reset

        config.learning.storage.retention_days = 0;
        assert!(
            config.validate().is_err(),
            "Zero retention days should be rejected"
        );

        config.learning.storage.retention_days = 365; // Reset

        // Test monitoring configuration validation
        config.monitoring.metrics_collection_interval_seconds = 0;
        assert!(
            config.validate().is_err(),
            "Zero metrics interval should be rejected"
        );

        config.monitoring.metrics_collection_interval_seconds = 60; // Reset

        // Test health check configuration validation
        config.health_checks.check_interval_seconds = 0;
        assert!(
            config.validate().is_err(),
            "Zero health check interval should be rejected"
        );

        config.health_checks.check_interval_seconds = 30; // Reset

        config.health_checks.timeout_seconds = 0;
        assert!(
            config.validate().is_err(),
            "Zero timeout should be rejected"
        );

        config.health_checks.timeout_seconds = 10; // Reset

        // Test alert configuration validation
        config.alerts.error_rate_threshold = -0.1;
        assert!(
            config.validate().is_err(),
            "Negative error rate threshold should be rejected"
        );

        config.alerts.error_rate_threshold = 1.1;
        assert!(
            config.validate().is_err(),
            "Error rate threshold > 1.0 should be rejected"
        );

        config.alerts.error_rate_threshold = 0.05; // Reset

        config.alerts.response_time_threshold_ms = 0;
        assert!(
            config.validate().is_err(),
            "Zero response time threshold should be rejected"
        );

        config.alerts.response_time_threshold_ms = 5000; // Reset

        // Test monitoring thresholds validation
        config
            .monitoring
            .alert_thresholds
            .min_pattern_recognition_accuracy = -0.1;
        assert!(
            config.validate().is_err(),
            "Negative pattern recognition accuracy should be rejected"
        );

        config
            .monitoring
            .alert_thresholds
            .min_pattern_recognition_accuracy = 1.1;
        assert!(
            config.validate().is_err(),
            "Pattern recognition accuracy > 1.0 should be rejected"
        );

        config
            .monitoring
            .alert_thresholds
            .min_pattern_recognition_accuracy = 0.85; // Reset

        config.monitoring.alert_thresholds.max_cpu_usage_percent = -1.0;
        assert!(
            config.validate().is_err(),
            "Negative CPU usage should be rejected"
        );

        config.monitoring.alert_thresholds.max_cpu_usage_percent = 101.0;
        assert!(
            config.validate().is_err(),
            "CPU usage > 100% should be rejected"
        );

        config.monitoring.alert_thresholds.max_cpu_usage_percent = 80.0; // Reset

        // Final validation should pass
        assert!(
            config.validate().is_ok(),
            "Reset config should pass validation"
        );
    }

    /// ANCHOR: Production vs Development configuration differences must be maintained
    /// This test ensures that production and development configurations have appropriate
    /// differences in security, performance, and safety settings.
    #[tokio::test]
    async fn anchor_production_vs_development_configuration_differences() {
        let prod_config = EnhancedLearningConfig::production_defaults();
        let dev_config = EnhancedLearningConfig::development_defaults();
        let test_config = EnhancedLearningConfig::testing_defaults();

        // Production should be more conservative/secure
        assert!(
            prod_config.learning.adaptation_threshold >= dev_config.learning.adaptation_threshold,
            "Production should have higher adaptation threshold than development"
        );

        assert!(
            prod_config.learning.adaptation_threshold >= test_config.learning.adaptation_threshold,
            "Production should have higher adaptation threshold than testing"
        );

        // Production should have optimization enabled, others disabled for safety
        assert!(
            prod_config.learning.enable_optimization,
            "Production should have optimization enabled"
        );
        assert!(
            !dev_config.learning.enable_optimization,
            "Development should have optimization disabled for safety"
        );
        assert!(
            !test_config.learning.enable_optimization,
            "Testing should have optimization disabled for predictability"
        );

        // Production should require more feedback before making adaptations
        assert!(
            prod_config.learning.min_feedback_threshold
                >= dev_config.learning.min_feedback_threshold,
            "Production should require more feedback than development"
        );
        assert!(
            prod_config.learning.min_feedback_threshold
                >= test_config.learning.min_feedback_threshold,
            "Production should require more feedback than testing"
        );

        // Production should have stricter monitoring thresholds
        assert!(
            prod_config
                .monitoring
                .alert_thresholds
                .min_pattern_recognition_accuracy
                >= dev_config
                    .monitoring
                    .alert_thresholds
                    .min_pattern_recognition_accuracy,
            "Production should have stricter pattern recognition accuracy requirements"
        );

        // Production should have alerts enabled, development and testing disabled
        assert!(
            prod_config.alerts.enabled,
            "Production should have alerts enabled"
        );
        assert!(
            !dev_config.alerts.enabled,
            "Development should have alerts disabled to reduce noise"
        );
        assert!(
            !test_config.alerts.enabled,
            "Testing should have alerts disabled"
        );

        // Production should have stricter error rate thresholds
        assert!(
            prod_config.alerts.error_rate_threshold <= dev_config.alerts.error_rate_threshold,
            "Production should have stricter error rate threshold"
        );

        // Testing should have the fastest collection intervals for quick feedback
        assert!(
            test_config.monitoring.metrics_collection_interval_seconds
                <= dev_config.monitoring.metrics_collection_interval_seconds,
            "Testing should have faster metrics collection than development"
        );
        assert!(
            test_config.health_checks.check_interval_seconds
                <= dev_config.health_checks.check_interval_seconds,
            "Testing should have faster health checks than development"
        );

        // Environment identifiers should be correct
        assert_eq!(prod_config.environment, "production");
        assert_eq!(dev_config.environment, "development");
        assert_eq!(test_config.environment, "testing");

        // All configurations should be valid
        assert!(
            prod_config.validate().is_ok(),
            "Production config should be valid"
        );
        assert!(
            dev_config.validate().is_ok(),
            "Development config should be valid"
        );
        assert!(
            test_config.validate().is_ok(),
            "Testing config should be valid"
        );
    }

    /// ANCHOR: File configuration with environment overrides must work correctly
    /// This test ensures that file-based configuration correctly applies environment
    /// variable overrides without losing the base configuration values.
    #[tokio::test]
    async fn anchor_file_config_with_environment_overrides() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("learning_config.json");

        // Create a base configuration file with known values
        let mut base_config = EnhancedLearningConfig::development_defaults();
        base_config.learning.adaptation_threshold = 0.7;
        base_config.learning.max_data_age_days = 60;
        base_config.learning.storage.collection_name = "base_collection".to_string();
        base_config.monitoring.metrics_collection_interval_seconds = 120;
        base_config.health_checks.check_interval_seconds = 60;
        base_config.alerts.error_rate_threshold = 0.1;

        base_config
            .save_to_file(config_path.to_str().unwrap())
            .await
            .unwrap();

        // Set environment variables to override specific values
        env::set_var("LEARNING_ADAPTATION_THRESHOLD", "0.9");
        env::set_var("LEARNING_STORAGE_COLLECTION_NAME", "override_collection");
        env::set_var("LEARNING_METRICS_INTERVAL", "30");
        env::set_var("LEARNING_ALERT_ERROR_THRESHOLD", "0.03");

        // Load configuration with environment overrides
        let manager = LearningConfigManager::from_file_with_env(config_path.to_str().unwrap())
            .await
            .unwrap();
        let config = manager.config();
        let enhanced_config = manager.enhanced_config();

        // Verify environment overrides were applied
        assert_eq!(
            config.adaptation_threshold, 0.9,
            "Adaptation threshold should be overridden"
        );
        assert_eq!(
            config.storage.collection_name, "override_collection",
            "Collection name should be overridden"
        );
        assert_eq!(
            enhanced_config.alerts.error_rate_threshold, 0.03,
            "Error rate threshold should be overridden"
        );

        // Verify non-overridden values remain from file
        assert_eq!(
            config.max_data_age_days, 60,
            "Non-overridden values should remain from file"
        );
        assert_eq!(
            enhanced_config.health_checks.check_interval_seconds, 60,
            "Non-overridden health check interval should remain"
        );

        // Verify other default values are preserved
        assert!(
            config.enable_feedback_learning,
            "Default boolean values should be preserved"
        );
        assert_eq!(
            enhanced_config.environment, "development",
            "Environment should remain from base config"
        );

        // Configuration should be valid
        assert!(
            enhanced_config.validate().is_ok(),
            "Overridden config should be valid"
        );

        // Clean up environment variables
        env::remove_var("LEARNING_ADAPTATION_THRESHOLD");
        env::remove_var("LEARNING_STORAGE_COLLECTION_NAME");
        env::remove_var("LEARNING_METRICS_INTERVAL");
        env::remove_var("LEARNING_ALERT_ERROR_THRESHOLD");
    }

    /// ANCHOR: Configuration change notification system must work reliably
    /// This test ensures that configuration watchers are properly notified when
    /// configuration changes occur, with the correct old and new configuration values.
    #[tokio::test]
    async fn anchor_configuration_change_notification_system() {
        let initial_config = LearningConfig::default();
        let mut manager = LearningConfigManager::new(initial_config.clone());

        // Add a configuration watcher
        let watcher = AnchorConfigWatcher::new();
        let change_count = watcher.change_count.clone();
        let last_configs = (
            watcher.last_old_config.clone(),
            watcher.last_new_config.clone(),
        );

        manager.add_watcher(Box::new(watcher));

        // Verify no changes initially
        assert_eq!(
            *change_count.lock().unwrap(),
            0,
            "Should start with no changes"
        );

        // Make first configuration change
        let mut updated_config = initial_config.clone();
        updated_config.adaptation_threshold = 0.85;
        updated_config.enable_optimization = true;

        manager.update_config(updated_config.clone()).await.unwrap();

        // Verify watcher was notified
        assert_eq!(
            *change_count.lock().unwrap(),
            1,
            "Should have one change notification"
        );

        // Verify the change notification contained correct values
        let (old_config_opt, new_config_opt) = {
            let old = last_configs.0.lock().unwrap().clone();
            let new = last_configs.1.lock().unwrap().clone();
            (old, new)
        };

        assert!(
            old_config_opt.is_some(),
            "Old config should be provided to watcher"
        );
        assert!(
            new_config_opt.is_some(),
            "New config should be provided to watcher"
        );

        let old_config = old_config_opt.unwrap();
        let new_config = new_config_opt.unwrap();

        // Verify old config has original values
        assert_eq!(
            old_config.learning.adaptation_threshold,
            initial_config.adaptation_threshold
        );
        assert_eq!(
            old_config.learning.enable_optimization,
            initial_config.enable_optimization
        );

        // Verify new config has updated values
        assert_eq!(new_config.learning.adaptation_threshold, 0.85);
        assert_eq!(new_config.learning.enable_optimization, true);

        // Make second configuration change
        let mut second_update = updated_config.clone();
        second_update.learning_rate = 0.2;
        second_update.max_data_age_days = 120;

        manager.update_config(second_update.clone()).await.unwrap();

        // Verify second notification
        assert_eq!(
            *change_count.lock().unwrap(),
            2,
            "Should have two change notifications"
        );

        // Verify current configuration is correct
        let current_config = manager.config();
        assert_eq!(current_config.adaptation_threshold, 0.85);
        assert_eq!(current_config.enable_optimization, true);
        assert_eq!(current_config.learning_rate, 0.2);
        assert_eq!(current_config.max_data_age_days, 120);
    }

    /// ANCHOR: Environment override behavior must be consistent and predictable
    /// This test ensures that environment-specific configuration overrides work
    /// consistently and produce predictable results across different environments.
    #[tokio::test]
    async fn anchor_environment_override_behavior_consistency() {
        let base_config = LearningConfig::default();

        // Test production overrides
        let mut prod_manager = LearningConfigManager::new(base_config.clone());
        prod_manager
            .apply_environment_overrides("production")
            .unwrap();
        let prod_config = prod_manager.config();
        let prod_enhanced = prod_manager.enhanced_config();

        // Production overrides should be applied consistently
        assert_eq!(
            prod_config.adaptation_threshold, 0.8,
            "Production adaptation threshold should be 0.8"
        );
        assert!(
            prod_config.enable_optimization,
            "Production should enable optimization"
        );
        assert_eq!(
            prod_config.min_feedback_threshold, 10,
            "Production should require 10 feedback entries"
        );
        assert!(
            prod_enhanced.monitoring.metrics_enabled,
            "Production should enable metrics"
        );
        assert!(
            prod_enhanced.health_checks.enabled,
            "Production should enable health checks"
        );
        assert!(
            prod_enhanced.alerts.enabled,
            "Production should enable alerts"
        );
        assert_eq!(
            prod_enhanced.environment, "production",
            "Environment should be set to production"
        );

        // Test development overrides
        let mut dev_manager = LearningConfigManager::new(base_config.clone());
        dev_manager
            .apply_environment_overrides("development")
            .unwrap();
        let dev_config = dev_manager.config();
        let dev_enhanced = dev_manager.enhanced_config();

        // Development overrides should be applied consistently
        assert_eq!(
            dev_config.adaptation_threshold, 0.6,
            "Development adaptation threshold should be 0.6"
        );
        assert!(
            !dev_config.enable_optimization,
            "Development should disable optimization"
        );
        assert_eq!(
            dev_config.min_feedback_threshold, 3,
            "Development should require 3 feedback entries"
        );
        assert!(
            dev_enhanced.monitoring.metrics_enabled,
            "Development should enable metrics"
        );
        assert!(
            dev_enhanced.health_checks.enabled,
            "Development should enable health checks"
        );
        assert!(
            !dev_enhanced.alerts.enabled,
            "Development should disable alerts"
        );
        assert_eq!(
            dev_enhanced.environment, "development",
            "Environment should be set to development"
        );

        // Test testing overrides
        let mut test_manager = LearningConfigManager::new(base_config.clone());
        test_manager.apply_environment_overrides("testing").unwrap();
        let test_config = test_manager.config();
        let test_enhanced = test_manager.enhanced_config();

        // Testing overrides should be applied consistently
        assert_eq!(
            test_config.adaptation_threshold, 0.5,
            "Testing adaptation threshold should be 0.5"
        );
        assert!(
            !test_config.enable_optimization,
            "Testing should disable optimization"
        );
        assert_eq!(
            test_config.min_feedback_threshold, 1,
            "Testing should require 1 feedback entry"
        );
        assert!(
            test_enhanced.monitoring.metrics_collection_interval_seconds <= 30,
            "Testing should have fast metrics collection"
        );
        assert!(
            test_enhanced.health_checks.check_interval_seconds <= 15,
            "Testing should have fast health checks"
        );
        assert!(
            !test_enhanced.alerts.enabled,
            "Testing should disable alerts"
        );
        assert_eq!(
            test_enhanced.environment, "testing",
            "Environment should be set to testing"
        );

        // Test unknown environment (should not crash, may warn)
        let mut unknown_manager = LearningConfigManager::new(base_config.clone());
        unknown_manager
            .apply_environment_overrides("unknown_env")
            .unwrap();
        let unknown_enhanced = unknown_manager.enhanced_config();

        assert_eq!(
            unknown_enhanced.environment, "unknown_env",
            "Environment should be set even for unknown environments"
        );
        // Should still have a valid configuration
        assert!(
            unknown_enhanced.validate().is_ok(),
            "Configuration should remain valid for unknown environments"
        );
    }

    /// ANCHOR: Configuration serialization must preserve all data accurately
    /// This test ensures that configuration serialization and deserialization
    /// preserves all configuration data without loss or corruption.
    #[tokio::test]
    async fn anchor_configuration_serialization_data_integrity() {
        // Create a comprehensive configuration with various data types
        let mut original_config = EnhancedLearningConfig::production_defaults();

        // Modify various fields to test serialization
        original_config.learning.adaptation_threshold = 0.123456789;
        original_config.learning.learning_rate = 0.987654321;
        original_config.learning.max_data_age_days = 42;
        original_config.learning.min_feedback_threshold = 123;
        original_config.learning.pattern_frequency_threshold = 456;
        original_config.learning.enable_feedback_learning = false;
        original_config.learning.enable_pattern_recognition = true;
        original_config.learning.enable_optimization = false;

        original_config.learning.storage.collection_name =
            "test_collection_with_special_chars_123!@#".to_string();
        original_config.learning.storage.enable_embeddings = false;
        original_config.learning.storage.batch_size = 789;
        original_config.learning.storage.retention_days = 999;

        original_config.learning.adaptation.enabled_algorithms = vec![
            "algorithm_1".to_string(),
            "algorithm_2".to_string(),
            "algorithm_with_underscores".to_string(),
        ];
        original_config.learning.adaptation.update_frequency_hours = 48;
        original_config.learning.adaptation.auto_apply_adaptations = true;

        // Add some algorithm settings
        let mut algorithm_settings = HashMap::new();
        algorithm_settings.insert(
            "param1".to_string(),
            serde_json::Value::Number(serde_json::Number::from(42)),
        );
        algorithm_settings.insert(
            "param2".to_string(),
            serde_json::Value::String("test_value".to_string()),
        );
        algorithm_settings.insert("param3".to_string(), serde_json::Value::Bool(true));
        algorithm_settings.insert(
            "param4".to_string(),
            serde_json::Value::Array(vec![
                serde_json::Value::String("item1".to_string()),
                serde_json::Value::String("item2".to_string()),
            ]),
        );
        original_config.learning.adaptation.algorithm_settings = algorithm_settings;

        original_config.monitoring.metrics_enabled = false;
        original_config
            .monitoring
            .metrics_collection_interval_seconds = 12345;
        original_config.monitoring.performance_tracking_enabled = false;
        original_config.monitoring.adaptation_tracking_enabled = true;
        original_config.monitoring.storage_metrics_enabled = false;

        original_config
            .monitoring
            .alert_thresholds
            .max_adaptation_time_ms = 98765;
        original_config
            .monitoring
            .alert_thresholds
            .max_storage_response_time_ms = 54321;
        original_config
            .monitoring
            .alert_thresholds
            .min_pattern_recognition_accuracy = 0.876543210;
        original_config
            .monitoring
            .alert_thresholds
            .max_memory_usage_mb = 2048;
        original_config
            .monitoring
            .alert_thresholds
            .max_cpu_usage_percent = 95.5;

        original_config.environment = "custom_test_environment".to_string();

        original_config.health_checks.enabled = false;
        original_config.health_checks.check_interval_seconds = 13579;
        original_config.health_checks.storage_health_check = false;
        original_config.health_checks.adaptation_health_check = true;
        original_config
            .health_checks
            .pattern_recognition_health_check = false;
        original_config.health_checks.timeout_seconds = 24680;

        original_config.alerts.enabled = false;
        original_config.alerts.error_rate_threshold = 0.0123456789;
        original_config.alerts.response_time_threshold_ms = 97531;
        original_config.alerts.storage_error_threshold = 86420;
        original_config.alerts.adaptation_failure_threshold = 13579;
        original_config.alerts.notification_channels = vec![
            "channel1".to_string(),
            "channel_with_underscores".to_string(),
            "channel-with-dashes".to_string(),
            "Channel With Spaces".to_string(),
        ];

        // Validate original configuration
        assert!(
            original_config.validate().is_ok(),
            "Original configuration should be valid"
        );

        // Serialize to JSON
        let json_string = serde_json::to_string_pretty(&original_config).unwrap();
        assert!(
            !json_string.is_empty(),
            "Serialized JSON should not be empty"
        );

        // Deserialize from JSON
        let deserialized_config: EnhancedLearningConfig =
            serde_json::from_str(&json_string).unwrap();

        // Validate deserialized configuration
        assert!(
            deserialized_config.validate().is_ok(),
            "Deserialized configuration should be valid"
        );

        // Verify all fields are preserved exactly
        assert_eq!(
            original_config.learning.adaptation_threshold,
            deserialized_config.learning.adaptation_threshold
        );
        assert_eq!(
            original_config.learning.learning_rate,
            deserialized_config.learning.learning_rate
        );
        assert_eq!(
            original_config.learning.max_data_age_days,
            deserialized_config.learning.max_data_age_days
        );
        assert_eq!(
            original_config.learning.min_feedback_threshold,
            deserialized_config.learning.min_feedback_threshold
        );
        assert_eq!(
            original_config.learning.pattern_frequency_threshold,
            deserialized_config.learning.pattern_frequency_threshold
        );
        assert_eq!(
            original_config.learning.enable_feedback_learning,
            deserialized_config.learning.enable_feedback_learning
        );
        assert_eq!(
            original_config.learning.enable_pattern_recognition,
            deserialized_config.learning.enable_pattern_recognition
        );
        assert_eq!(
            original_config.learning.enable_optimization,
            deserialized_config.learning.enable_optimization
        );

        assert_eq!(
            original_config.learning.storage.collection_name,
            deserialized_config.learning.storage.collection_name
        );
        assert_eq!(
            original_config.learning.storage.enable_embeddings,
            deserialized_config.learning.storage.enable_embeddings
        );
        assert_eq!(
            original_config.learning.storage.batch_size,
            deserialized_config.learning.storage.batch_size
        );
        assert_eq!(
            original_config.learning.storage.retention_days,
            deserialized_config.learning.storage.retention_days
        );

        assert_eq!(
            original_config.learning.adaptation.enabled_algorithms,
            deserialized_config.learning.adaptation.enabled_algorithms
        );
        assert_eq!(
            original_config.learning.adaptation.update_frequency_hours,
            deserialized_config
                .learning
                .adaptation
                .update_frequency_hours
        );
        assert_eq!(
            original_config.learning.adaptation.auto_apply_adaptations,
            deserialized_config
                .learning
                .adaptation
                .auto_apply_adaptations
        );

        // Verify complex algorithm settings preservation
        assert_eq!(
            original_config.learning.adaptation.algorithm_settings.len(),
            deserialized_config
                .learning
                .adaptation
                .algorithm_settings
                .len()
        );
        for (key, value) in &original_config.learning.adaptation.algorithm_settings {
            assert!(deserialized_config
                .learning
                .adaptation
                .algorithm_settings
                .contains_key(key));
            assert_eq!(
                value,
                deserialized_config
                    .learning
                    .adaptation
                    .algorithm_settings
                    .get(key)
                    .unwrap()
            );
        }

        assert_eq!(
            original_config.monitoring.metrics_enabled,
            deserialized_config.monitoring.metrics_enabled
        );
        assert_eq!(
            original_config
                .monitoring
                .metrics_collection_interval_seconds,
            deserialized_config
                .monitoring
                .metrics_collection_interval_seconds
        );
        assert_eq!(
            original_config.monitoring.performance_tracking_enabled,
            deserialized_config.monitoring.performance_tracking_enabled
        );
        assert_eq!(
            original_config.monitoring.adaptation_tracking_enabled,
            deserialized_config.monitoring.adaptation_tracking_enabled
        );
        assert_eq!(
            original_config.monitoring.storage_metrics_enabled,
            deserialized_config.monitoring.storage_metrics_enabled
        );

        assert_eq!(
            original_config
                .monitoring
                .alert_thresholds
                .max_adaptation_time_ms,
            deserialized_config
                .monitoring
                .alert_thresholds
                .max_adaptation_time_ms
        );
        assert_eq!(
            original_config
                .monitoring
                .alert_thresholds
                .max_storage_response_time_ms,
            deserialized_config
                .monitoring
                .alert_thresholds
                .max_storage_response_time_ms
        );
        assert_eq!(
            original_config
                .monitoring
                .alert_thresholds
                .min_pattern_recognition_accuracy,
            deserialized_config
                .monitoring
                .alert_thresholds
                .min_pattern_recognition_accuracy
        );
        assert_eq!(
            original_config
                .monitoring
                .alert_thresholds
                .max_memory_usage_mb,
            deserialized_config
                .monitoring
                .alert_thresholds
                .max_memory_usage_mb
        );
        assert_eq!(
            original_config
                .monitoring
                .alert_thresholds
                .max_cpu_usage_percent,
            deserialized_config
                .monitoring
                .alert_thresholds
                .max_cpu_usage_percent
        );

        assert_eq!(original_config.environment, deserialized_config.environment);

        assert_eq!(
            original_config.health_checks.enabled,
            deserialized_config.health_checks.enabled
        );
        assert_eq!(
            original_config.health_checks.check_interval_seconds,
            deserialized_config.health_checks.check_interval_seconds
        );
        assert_eq!(
            original_config.health_checks.storage_health_check,
            deserialized_config.health_checks.storage_health_check
        );
        assert_eq!(
            original_config.health_checks.adaptation_health_check,
            deserialized_config.health_checks.adaptation_health_check
        );
        assert_eq!(
            original_config
                .health_checks
                .pattern_recognition_health_check,
            deserialized_config
                .health_checks
                .pattern_recognition_health_check
        );
        assert_eq!(
            original_config.health_checks.timeout_seconds,
            deserialized_config.health_checks.timeout_seconds
        );

        assert_eq!(
            original_config.alerts.enabled,
            deserialized_config.alerts.enabled
        );
        assert_eq!(
            original_config.alerts.error_rate_threshold,
            deserialized_config.alerts.error_rate_threshold
        );
        assert_eq!(
            original_config.alerts.response_time_threshold_ms,
            deserialized_config.alerts.response_time_threshold_ms
        );
        assert_eq!(
            original_config.alerts.storage_error_threshold,
            deserialized_config.alerts.storage_error_threshold
        );
        assert_eq!(
            original_config.alerts.adaptation_failure_threshold,
            deserialized_config.alerts.adaptation_failure_threshold
        );
        assert_eq!(
            original_config.alerts.notification_channels,
            deserialized_config.alerts.notification_channels
        );

        // Test round-trip again to ensure stability
        let second_json = serde_json::to_string_pretty(&deserialized_config).unwrap();
        let second_deserialized: EnhancedLearningConfig =
            serde_json::from_str(&second_json).unwrap();
        assert!(
            second_deserialized.validate().is_ok(),
            "Second deserialization should be valid"
        );

        // Key fields should still match after second round-trip
        assert_eq!(
            deserialized_config.learning.adaptation_threshold,
            second_deserialized.learning.adaptation_threshold
        );
        assert_eq!(
            deserialized_config.environment,
            second_deserialized.environment
        );
        assert_eq!(
            deserialized_config.alerts.notification_channels,
            second_deserialized.alerts.notification_channels
        );
    }
}
