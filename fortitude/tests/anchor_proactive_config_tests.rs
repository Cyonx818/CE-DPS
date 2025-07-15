// ABOUTME: Comprehensive tests for proactive configuration management system
// ANCHOR: ProactiveConfigurationManagement
//! # Proactive Configuration Management Tests
//!
//! This test suite verifies the comprehensive configuration management system for proactive
//! research settings following TDD principles. Tests cover schema definition, validation,
//! loading from multiple sources, hot-reload capabilities, and integration with CLI/API/MCP.
//!
//! ## Key Test Areas:
//! - Configuration schema structure and validation
//! - Multi-source loading with proper precedence
//! - Hot-reload and runtime updates
//! - Export/import functionality
//! - Configuration versioning and migration
//! - Integration with existing fortitude systems

use fortitude::proactive::{
    BackgroundResearchConfig, ComprehensiveConfigError,
    ComprehensivePerformanceConfig as PerformanceConfig, NotificationConfig, ProactiveConfig,
    ProactiveConfigManager, ProactiveGapAnalysisConfig as GapAnalysisConfig, UserPreferenceConfig,
    WorkspaceConfig,
};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use tempfile::{NamedTempFile, TempDir};
use tokio::fs;
use validator::Validate;

#[tokio::test]
async fn test_proactive_config_schema_structure() {
    // ANCHOR: ProactiveConfigSchemaTest
    // Verify the complete configuration schema structure matches requirements

    let config = ProactiveConfig::default();

    // Verify main sections exist
    assert!(config.gap_analysis.is_some());
    assert!(config.background_research.is_some());
    assert!(config.notifications.is_some());
    assert!(config.performance.is_some());
    assert!(config.user_preferences.is_some());
    assert!(config.workspace.is_some());

    // Verify gap analysis configuration structure
    let gap_config = config.gap_analysis.unwrap();
    assert!(gap_config.scan_intervals_seconds > 0);
    assert!(!gap_config.file_patterns.is_empty());
    assert!(!gap_config.detection_rules.is_empty());
    assert!(gap_config.confidence_threshold >= 0.0 && gap_config.confidence_threshold <= 1.0);

    // Verify background research configuration structure
    let research_config = config.background_research.unwrap();
    assert!(research_config.max_concurrent_tasks > 0);
    assert!(research_config.rate_limit_requests_per_minute > 0);
    assert!(research_config.scheduling_enabled);
    assert!(!research_config.priority_keywords.is_empty());

    // Verify notification configuration structure
    let notification_config = config.notifications.unwrap();
    assert!(!notification_config.channels.is_empty());
    assert!(notification_config
        .delivery_preferences
        .contains_key("email"));
    assert!(notification_config
        .delivery_preferences
        .contains_key("desktop"));
    assert!(!notification_config.delivery_rules.is_empty());

    // Verify performance configuration structure
    let perf_config = config.performance.unwrap();
    assert!(perf_config.resource_limits.max_memory_mb > 0);
    assert!(perf_config.resource_limits.max_cpu_percent > 0);
    assert!(perf_config.monitoring_enabled);
    assert!(!perf_config.alert_thresholds.is_empty());

    // Verify user preferences configuration structure
    let user_config = config.user_preferences.unwrap();
    assert!(!user_config.research_domains.is_empty());
    assert!(user_config.notification_frequency_hours > 0);
    assert!(!user_config.preferred_formats.is_empty());

    // Verify workspace configuration structure
    let workspace_config = config.workspace.unwrap();
    assert!(!workspace_config.project_paths.is_empty());
    assert!(!workspace_config.exclude_patterns.is_empty());
    assert!(workspace_config.auto_discovery_enabled);
}

#[tokio::test]
async fn test_config_validation_comprehensive() {
    // ANCHOR: ConfigValidationTest
    // Test comprehensive validation with detailed error messages

    let mut config = ProactiveConfig::default();

    // Valid configuration should pass
    assert!(config.validate().is_ok());

    // Test invalid gap analysis configuration
    config.gap_analysis.as_mut().unwrap().scan_intervals_seconds = 0;
    let result = config.validate();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ComprehensiveConfigError::InvalidValue { .. }
    ));

    // Reset and test invalid confidence threshold
    config = ProactiveConfig::default();
    config.gap_analysis.as_mut().unwrap().confidence_threshold = 1.5;
    let result = config.validate();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ComprehensiveConfigError::InvalidThreshold { .. }
    ));

    // Test invalid background research configuration
    config = ProactiveConfig::default();
    config
        .background_research
        .as_mut()
        .unwrap()
        .max_concurrent_tasks = 0;
    let result = config.validate();
    assert!(result.is_err());

    // Test invalid rate limiting
    config = ProactiveConfig::default();
    config
        .background_research
        .as_mut()
        .unwrap()
        .rate_limit_requests_per_minute = 0;
    let result = config.validate();
    assert!(result.is_err());

    // Test invalid performance limits
    config = ProactiveConfig::default();
    config
        .performance
        .as_mut()
        .unwrap()
        .resource_limits
        .max_memory_mb = 0;
    let result = config.validate();
    assert!(result.is_err());

    // Test invalid notification configuration
    config = ProactiveConfig::default();
    config.notifications.as_mut().unwrap().channels.clear();
    let result = config.validate();
    assert!(result.is_err());

    // Test invalid user preferences
    config = ProactiveConfig::default();
    config
        .user_preferences
        .as_mut()
        .unwrap()
        .notification_frequency_hours = 0;
    let result = config.validate();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_config_loading_from_multiple_sources() {
    // ANCHOR: MultiSourceConfigLoadingTest
    // Test loading configuration from files, environment variables, and CLI args with precedence

    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("proactive_config.json");

    // Create test configuration file
    let file_config = json!({
        "gap_analysis": {
            "scan_intervals_seconds": 300,
            "file_patterns": ["*.rs", "*.md"],
            "confidence_threshold": 0.7
        },
        "background_research": {
            "max_concurrent_tasks": 2,
            "rate_limit_requests_per_minute": 30
        },
        "notifications": {
            "channels": ["email", "desktop"]
        }
    });

    fs::write(&config_file, file_config.to_string())
        .await
        .unwrap();

    // Set environment variables (should override file config)
    env::set_var("FORTITUDE_PROACTIVE_GAP_SCAN_INTERVAL", "600");
    env::set_var("FORTITUDE_PROACTIVE_MAX_CONCURRENT_TASKS", "4");
    env::set_var("FORTITUDE_PROACTIVE_CONFIDENCE_THRESHOLD", "0.8");

    // Load configuration with precedence: env vars > file > defaults
    let config_manager = ProactiveConfigManager::new();
    let config = config_manager
        .load_from_sources(
            Some(&config_file),
            true, // load_from_env
            None, // no CLI args for this test
        )
        .await
        .unwrap();

    // Verify environment variables take precedence
    assert_eq!(
        config.gap_analysis.as_ref().unwrap().scan_intervals_seconds,
        600
    );
    assert_eq!(
        config
            .background_research
            .as_ref()
            .unwrap()
            .max_concurrent_tasks,
        4
    );
    assert_eq!(
        config.gap_analysis.as_ref().unwrap().confidence_threshold,
        0.8
    );

    // Verify file values are used where env vars not set
    assert_eq!(
        config
            .background_research
            .as_ref()
            .unwrap()
            .rate_limit_requests_per_minute,
        30
    );

    // Clean up environment variables
    env::remove_var("FORTITUDE_PROACTIVE_GAP_SCAN_INTERVAL");
    env::remove_var("FORTITUDE_PROACTIVE_MAX_CONCURRENT_TASKS");
    env::remove_var("FORTITUDE_PROACTIVE_CONFIDENCE_THRESHOLD");
}

#[tokio::test]
async fn test_config_hot_reload_capabilities() {
    // ANCHOR: ConfigHotReloadTest
    // Test hot-reload and runtime configuration updates

    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("proactive_config.json");

    // Create initial configuration
    let initial_config = json!({
        "gap_analysis": {
            "scan_intervals_seconds": 300,
            "confidence_threshold": 0.7
        }
    });

    fs::write(&config_file, initial_config.to_string())
        .await
        .unwrap();

    // Create config manager with hot reload enabled
    let mut config_manager = ProactiveConfigManager::new();
    config_manager
        .enable_hot_reload(&config_file, |new_config| {
            // Callback for configuration changes
            println!("Configuration reloaded: {:?}", new_config);
        })
        .await
        .unwrap();

    let initial = config_manager.get_current_config().await;
    assert_eq!(
        initial
            .gap_analysis
            .as_ref()
            .unwrap()
            .scan_intervals_seconds,
        300
    );

    // Simulate configuration file change
    let updated_config = json!({
        "gap_analysis": {
            "scan_intervals_seconds": 600,
            "confidence_threshold": 0.8
        }
    });

    fs::write(&config_file, updated_config.to_string())
        .await
        .unwrap();

    // Give some time for file watcher to detect change
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify configuration was reloaded
    let reloaded = config_manager.get_current_config().await;
    assert_eq!(
        reloaded
            .gap_analysis
            .as_ref()
            .unwrap()
            .scan_intervals_seconds,
        600
    );
    assert_eq!(
        reloaded.gap_analysis.as_ref().unwrap().confidence_threshold,
        0.8
    );
}

#[tokio::test]
async fn test_config_export_import_functionality() {
    // ANCHOR: ConfigExportImportTest
    // Test configuration export/import for sharing settings

    let temp_dir = TempDir::new().unwrap();

    // Create a comprehensive configuration
    let original_config = ProactiveConfig {
        gap_analysis: Some(GapAnalysisConfig {
            scan_intervals_seconds: 300,
            file_patterns: vec!["*.rs".to_string(), "*.md".to_string()],
            detection_rules: vec!["todo".to_string(), "fixme".to_string()],
            confidence_threshold: 0.7,
            enable_semantic_analysis: true,
            max_files_per_scan: 1000,
            priority_file_types: HashMap::new(),
            custom_rules: Vec::new(),
            gap_detection: None,
        }),
        background_research: Some(BackgroundResearchConfig {
            max_concurrent_tasks: 3,
            rate_limit_requests_per_minute: 50,
            scheduling_enabled: true,
            priority_keywords: vec!["urgent".to_string(), "critical".to_string()],
            research_timeout_seconds: 300,
            auto_prioritization_enabled: true,
            quality_thresholds: Default::default(),
            integration: Default::default(),
        }),
        ..ProactiveConfig::default()
    };

    // Test JSON export/import
    let json_file = temp_dir.path().join("config_export.json");
    original_config
        .export_to_file(&json_file, "json")
        .await
        .unwrap();
    let imported_json = ProactiveConfig::import_from_file(&json_file).await.unwrap();
    assert_eq!(
        original_config
            .gap_analysis
            .as_ref()
            .unwrap()
            .scan_intervals_seconds,
        imported_json
            .gap_analysis
            .as_ref()
            .unwrap()
            .scan_intervals_seconds
    );

    // Test TOML export/import
    let toml_file = temp_dir.path().join("config_export.toml");
    original_config
        .export_to_file(&toml_file, "toml")
        .await
        .unwrap();
    let imported_toml = ProactiveConfig::import_from_file(&toml_file).await.unwrap();
    assert_eq!(
        original_config
            .background_research
            .as_ref()
            .unwrap()
            .max_concurrent_tasks,
        imported_toml
            .background_research
            .as_ref()
            .unwrap()
            .max_concurrent_tasks
    );

    // Test YAML export/import
    let yaml_file = temp_dir.path().join("config_export.yaml");
    original_config
        .export_to_file(&yaml_file, "yaml")
        .await
        .unwrap();
    let imported_yaml = ProactiveConfig::import_from_file(&yaml_file).await.unwrap();
    assert_eq!(
        original_config
            .gap_analysis
            .as_ref()
            .unwrap()
            .confidence_threshold,
        imported_yaml
            .gap_analysis
            .as_ref()
            .unwrap()
            .confidence_threshold
    );
}

#[tokio::test]
async fn test_config_versioning_and_migration() {
    // ANCHOR: ConfigVersioningTest
    // Test configuration versioning and migration support

    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("legacy_config.json");

    // Create legacy v1 configuration
    let legacy_config = json!({
        "version": "1.0",
        "gap_detection": {  // Old naming
            "interval": 300,
            "threshold": 0.7
        },
        "research": {  // Old naming
            "concurrent": 2
        }
    });

    fs::write(&config_file, legacy_config.to_string())
        .await
        .unwrap();

    // Load and migrate configuration
    let config_manager = ProactiveConfigManager::new();
    let migrated_config = config_manager
        .load_with_migration(&config_file)
        .await
        .unwrap();

    // Verify migration occurred correctly
    assert_eq!(migrated_config.version, "2.0".to_string());
    assert_eq!(
        migrated_config
            .gap_analysis
            .as_ref()
            .unwrap()
            .scan_intervals_seconds,
        300
    );
    assert_eq!(
        migrated_config
            .gap_analysis
            .as_ref()
            .unwrap()
            .confidence_threshold,
        0.7
    );
    assert_eq!(
        migrated_config
            .background_research
            .as_ref()
            .unwrap()
            .max_concurrent_tasks,
        2
    );

    // Test unsupported version error
    let unsupported_config = json!({
        "version": "0.5"
    });

    let unsupported_file = temp_dir.path().join("unsupported_config.json");
    fs::write(&unsupported_file, unsupported_config.to_string())
        .await
        .unwrap();

    let result = config_manager.load_with_migration(&unsupported_file).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ComprehensiveConfigError::UnsupportedVersion { .. }
    ));
}

#[tokio::test]
async fn test_config_templates_and_presets() {
    // ANCHOR: ConfigTemplatesTest
    // Test configuration templates for common use cases

    // Test development preset
    let dev_config = ProactiveConfig::preset("development").unwrap();
    assert!(
        dev_config
            .gap_analysis
            .as_ref()
            .unwrap()
            .scan_intervals_seconds
            <= 60
    ); // Frequent scans for dev
    assert!(dev_config
        .notifications
        .as_ref()
        .unwrap()
        .delivery_preferences
        .get("desktop")
        .unwrap_or(&false));
    assert!(!dev_config.performance.as_ref().unwrap().monitoring_enabled); // Less monitoring overhead

    // Test production preset
    let prod_config = ProactiveConfig::preset("production").unwrap();
    assert!(
        prod_config
            .gap_analysis
            .as_ref()
            .unwrap()
            .scan_intervals_seconds
            >= 300
    ); // Less frequent for prod
    assert!(prod_config.performance.as_ref().unwrap().monitoring_enabled); // Full monitoring
    assert!(
        prod_config
            .background_research
            .as_ref()
            .unwrap()
            .rate_limit_requests_per_minute
            <= 30
    ); // Conservative rate limiting

    // Test research-focused preset
    let research_config = ProactiveConfig::preset("research").unwrap();
    assert!(
        research_config
            .background_research
            .as_ref()
            .unwrap()
            .max_concurrent_tasks
            >= 5
    ); // More parallel research
    assert!(
        research_config
            .gap_analysis
            .as_ref()
            .unwrap()
            .enable_semantic_analysis
    ); // Enhanced analysis
    assert!(
        research_config
            .background_research
            .as_ref()
            .unwrap()
            .auto_prioritization_enabled
    );

    // Test minimal preset
    let minimal_config = ProactiveConfig::preset("minimal").unwrap();
    assert_eq!(
        minimal_config
            .background_research
            .as_ref()
            .unwrap()
            .max_concurrent_tasks,
        1
    );
    assert!(
        !minimal_config
            .gap_analysis
            .as_ref()
            .unwrap()
            .enable_semantic_analysis
    );
    assert_eq!(
        minimal_config
            .notifications
            .as_ref()
            .unwrap()
            .channels
            .len(),
        1
    );

    // Test unknown preset error
    let result = ProactiveConfig::preset("unknown");
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ComprehensiveConfigError::PresetNotFound { .. }
    ));
}

#[tokio::test]
async fn test_config_workspace_specific_settings() {
    // ANCHOR: WorkspaceConfigTest
    // Test workspace-specific configuration overrides

    let temp_dir = TempDir::new().unwrap();
    let workspace_dir = temp_dir.path().join("test_workspace");
    fs::create_dir_all(&workspace_dir).await.unwrap();

    // Create global configuration
    let global_config = ProactiveConfig {
        gap_analysis: Some(GapAnalysisConfig {
            scan_intervals_seconds: 300,
            confidence_threshold: 0.7,
            ..GapAnalysisConfig::default()
        }),
        ..ProactiveConfig::default()
    };

    // Create workspace-specific overrides
    let workspace_overrides = json!({
        "gap_analysis": {
            "scan_intervals_seconds": 60,  // More frequent for this workspace
            "confidence_threshold": 0.8     // Higher threshold
        },
        "background_research": {
            "max_concurrent_tasks": 1       // Limited resources
        }
    });

    let workspace_config_file = workspace_dir.join(".fortitude_config.json");
    fs::write(&workspace_config_file, workspace_overrides.to_string())
        .await
        .unwrap();

    // Load workspace-specific configuration
    let config_manager = ProactiveConfigManager::new();
    let workspace_config = config_manager
        .load_for_workspace(&global_config, &workspace_dir)
        .await
        .unwrap();

    // Verify workspace overrides are applied
    assert_eq!(
        workspace_config
            .gap_analysis
            .as_ref()
            .unwrap()
            .scan_intervals_seconds,
        60
    );
    assert_eq!(
        workspace_config
            .gap_analysis
            .as_ref()
            .unwrap()
            .confidence_threshold,
        0.8
    );
    assert_eq!(
        workspace_config
            .background_research
            .as_ref()
            .unwrap()
            .max_concurrent_tasks,
        1
    );

    // Verify non-overridden values remain from global config
    assert_eq!(
        workspace_config
            .gap_analysis
            .as_ref()
            .unwrap()
            .enable_semantic_analysis,
        global_config
            .gap_analysis
            .as_ref()
            .unwrap()
            .enable_semantic_analysis
    );
}

#[tokio::test]
async fn test_config_environment_specific_profiles() {
    // ANCHOR: EnvironmentProfilesTest
    // Test environment-specific configuration profiles

    let config_manager = ProactiveConfigManager::new();

    // Test development environment
    env::set_var("FORTITUDE_ENVIRONMENT", "development");
    let dev_config = config_manager.load_for_environment().await.unwrap();
    assert!(
        dev_config
            .gap_analysis
            .as_ref()
            .unwrap()
            .scan_intervals_seconds
            <= 120
    );
    assert!(
        dev_config
            .performance
            .as_ref()
            .unwrap()
            .resource_limits
            .max_cpu_percent
            >= 80
    );

    // Test staging environment
    env::set_var("FORTITUDE_ENVIRONMENT", "staging");
    let staging_config = config_manager.load_for_environment().await.unwrap();
    assert!(
        staging_config
            .gap_analysis
            .as_ref()
            .unwrap()
            .scan_intervals_seconds
            >= 300
    );
    assert!(
        staging_config
            .performance
            .as_ref()
            .unwrap()
            .monitoring_enabled
    );

    // Test production environment
    env::set_var("FORTITUDE_ENVIRONMENT", "production");
    let prod_config = config_manager.load_for_environment().await.unwrap();
    assert!(
        prod_config
            .gap_analysis
            .as_ref()
            .unwrap()
            .scan_intervals_seconds
            >= 600
    );
    assert!(
        prod_config
            .background_research
            .as_ref()
            .unwrap()
            .rate_limit_requests_per_minute
            <= 30
    );
    assert!(prod_config.performance.as_ref().unwrap().monitoring_enabled);

    // Clean up
    env::remove_var("FORTITUDE_ENVIRONMENT");
}

#[tokio::test]
async fn test_config_validation_with_custom_rules() {
    // ANCHOR: CustomValidationRulesTest
    // Test custom validation rules and detailed error reporting

    let mut config = ProactiveConfig::default();

    // Test conflicting settings validation
    config.gap_analysis.as_mut().unwrap().scan_intervals_seconds = 30; // Very frequent
    config
        .background_research
        .as_mut()
        .unwrap()
        .max_concurrent_tasks = 10; // High concurrency
    config
        .performance
        .as_mut()
        .unwrap()
        .resource_limits
        .max_cpu_percent = 50; // Low CPU limit

    let result = config.validate_with_custom_rules();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(
        error,
        ComprehensiveConfigError::ConflictingSettings { .. }
    ));

    // Test resource compatibility validation
    config = ProactiveConfig::default();
    config
        .background_research
        .as_mut()
        .unwrap()
        .max_concurrent_tasks = 20;
    config
        .performance
        .as_mut()
        .unwrap()
        .resource_limits
        .max_memory_mb = 100; // Too low for high concurrency

    let result = config.validate_with_custom_rules();
    assert!(result.is_err());

    // Test file pattern validation
    config = ProactiveConfig::default();
    config.gap_analysis.as_mut().unwrap().file_patterns = vec!["[invalid".to_string()]; // Invalid regex

    let result = config.validate();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ComprehensiveConfigError::InvalidPattern { .. }
    ));
}

#[tokio::test]
async fn test_config_performance_monitoring() {
    // ANCHOR: ConfigPerformanceMonitoringTest
    // Test configuration performance monitoring and alerting

    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("perf_config.json");

    let perf_config = json!({
        "performance": {
            "monitoring_enabled": true,
            "alert_thresholds": {
                "config_load_time_ms": 1000,
                "validation_time_ms": 500,
                "file_size_mb": 10
            },
            "metrics_collection_enabled": true
        }
    });

    fs::write(&config_file, perf_config.to_string())
        .await
        .unwrap();

    let config_manager = ProactiveConfigManager::with_performance_monitoring(true);

    // Load configuration and measure performance
    let start = std::time::Instant::now();
    let config = config_manager.load_from_file(&config_file).await.unwrap();
    let load_time = start.elapsed();

    // Verify performance metrics are collected
    let metrics = config_manager.get_performance_metrics().await;
    assert!(metrics.contains_key("config_load_time_ms"));
    assert!(metrics.contains_key("validation_time_ms"));
    assert!(metrics.contains_key("file_size_bytes"));

    // Test performance alerting
    if load_time.as_millis() > 1000 {
        let alerts = config_manager.get_performance_alerts().await;
        assert!(!alerts.is_empty());
        assert!(alerts
            .iter()
            .any(|alert| alert.alert_type == "slow_config_load"));
    }
}
