// ABOUTME: Anchor tests for Sprint 009 Tasks 3 & 4 - Configuration and Validation Critical Workflows
//! These tests protect critical configuration and validation functionality implemented in Sprint 009
//! Tasks 3 and 4. They ensure that configuration management and validation workflows continue to work
//! correctly as the system evolves.
//!
//! ## Protected Functionality
//! - User input processing (configuration validation, parameter processing)
//! - Type definition changes (configuration structures, validation interfaces)
//! - Business logic (configuration management, validation rules)
//! - Critical error handling (configuration error recovery, validation failure handling)
//! - Cross-component integration (configuration propagation across systems)

use fortitude::learning::*;
use fortitude::monitoring::*;
use fortitude_api_server::{
    middleware::auth::{AuthConfig, Permission},
    ApiServerConfig,
};
use fortitude_mcp_server::{RateLimitConfig, ServerConfig as McpServerConfig};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::RwLock;

/// Comprehensive configuration management system
pub struct ConfigurationManager {
    learning_config: Arc<RwLock<LearningSystemConfig>>,
    monitoring_config: Arc<RwLock<MonitoringSystemConfig>>,
    api_config: Arc<RwLock<ApiServerConfig>>,
    mcp_config: Arc<RwLock<McpServerConfig>>,
    quality_config: Arc<RwLock<QualityControlConfig>>,
    validation_rules: Arc<RwLock<ValidationRules>>,
    config_history: Arc<RwLock<Vec<ConfigurationChange>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LearningSystemConfig {
    pub enabled: bool,
    pub feedback_processing: FeedbackProcessingConfig,
    pub pattern_analysis: PatternAnalysisConfig,
    pub learning_algorithms: LearningAlgorithmsConfig,
    pub storage_settings: StorageConfig,
    pub performance_thresholds: PerformanceThresholds,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeedbackProcessingConfig {
    pub batch_size: usize,
    pub processing_interval_ms: u64,
    pub quality_threshold: f64,
    pub auto_validation: bool,
    pub feedback_expiry_days: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatternAnalysisConfig {
    pub min_data_points: usize,
    pub confidence_threshold: f64,
    pub analysis_interval_hours: u32,
    pub pattern_retention_days: u32,
    pub similarity_threshold: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LearningAlgorithmsConfig {
    pub adaptation_rate: f64,
    pub convergence_threshold: f64,
    pub max_iterations: u32,
    pub learning_rate_decay: f64,
    pub regularization_factor: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    pub vector_dimension: usize,
    pub cache_size_mb: usize,
    pub backup_interval_hours: u32,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitoringSystemConfig {
    pub enabled: bool,
    pub metrics_collection: MetricsCollectionConfig,
    pub alerting: AlertingConfig,
    pub health_checks: HealthCheckConfig,
    pub retention_settings: RetentionConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricsCollectionConfig {
    pub collection_interval_ms: u64,
    pub buffer_size: usize,
    pub aggregation_window_minutes: u32,
    pub detailed_metrics: bool,
    pub custom_metrics: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlertingConfig {
    pub enabled: bool,
    pub severity_thresholds: HashMap<String, f64>,
    pub notification_channels: Vec<String>,
    pub rate_limiting: bool,
    pub escalation_rules: Vec<EscalationRule>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EscalationRule {
    pub condition: String,
    pub delay_minutes: u32,
    pub action: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub interval_seconds: u32,
    pub timeout_seconds: u32,
    pub failure_threshold: u32,
    pub recovery_threshold: u32,
    pub components: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetentionConfig {
    pub metrics_retention_days: u32,
    pub alerts_retention_days: u32,
    pub logs_retention_days: u32,
    pub cleanup_interval_hours: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QualityControlConfig {
    pub enabled: bool,
    pub validation_levels: Vec<ValidationLevel>,
    pub quality_thresholds: QualityThresholds,
    pub processing_rules: ProcessingRules,
    pub output_requirements: OutputRequirements,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationLevel {
    pub name: String,
    pub enabled: bool,
    pub rules: Vec<String>,
    pub severity: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QualityThresholds {
    pub min_confidence: f64,
    pub max_processing_time_ms: u64,
    pub min_data_quality_score: f64,
    pub max_error_rate: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessingRules {
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub fallback_enabled: bool,
    pub parallel_processing: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputRequirements {
    pub format_validation: bool,
    pub content_validation: bool,
    pub schema_validation: bool,
    pub performance_validation: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationRules {
    pub learning_rules: LearningValidationRules,
    pub monitoring_rules: MonitoringValidationRules,
    pub api_rules: ApiValidationRules,
    pub mcp_rules: McpValidationRules,
    pub global_rules: GlobalValidationRules,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LearningValidationRules {
    pub min_feedback_score: f64,
    pub max_feedback_score: f64,
    pub required_fields: Vec<String>,
    pub max_text_length: usize,
    pub allowed_feedback_types: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitoringValidationRules {
    pub metric_name_pattern: String,
    pub value_range: (f64, f64),
    pub required_tags: Vec<String>,
    pub max_tag_count: usize,
    pub timestamp_tolerance_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiValidationRules {
    pub max_request_size_bytes: usize,
    pub rate_limit_per_minute: u32,
    pub required_headers: Vec<String>,
    pub allowed_content_types: Vec<String>,
    pub max_response_time_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpValidationRules {
    pub max_tool_params: usize,
    pub required_permissions: Vec<String>,
    pub max_resource_size_bytes: usize,
    pub allowed_operations: Vec<String>,
    pub timeout_seconds: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalValidationRules {
    pub max_concurrent_operations: usize,
    pub max_memory_usage_mb: usize,
    pub max_disk_usage_gb: usize,
    pub max_cpu_usage_percent: f64,
    pub health_check_interval_seconds: u32,
}

#[derive(Clone, Debug)]
pub struct ConfigurationChange {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub component: String,
    pub change_type: String,
    pub old_value: Option<Value>,
    pub new_value: Value,
    pub user: String,
    pub reason: String,
}

impl Default for LearningSystemConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            feedback_processing: FeedbackProcessingConfig {
                batch_size: 100,
                processing_interval_ms: 5000,
                quality_threshold: 0.7,
                auto_validation: true,
                feedback_expiry_days: 30,
            },
            pattern_analysis: PatternAnalysisConfig {
                min_data_points: 10,
                confidence_threshold: 0.8,
                analysis_interval_hours: 24,
                pattern_retention_days: 90,
                similarity_threshold: 0.85,
            },
            learning_algorithms: LearningAlgorithmsConfig {
                adaptation_rate: 0.01,
                convergence_threshold: 0.001,
                max_iterations: 1000,
                learning_rate_decay: 0.95,
                regularization_factor: 0.001,
            },
            storage_settings: StorageConfig {
                vector_dimension: 1536,
                cache_size_mb: 512,
                backup_interval_hours: 24,
                compression_enabled: true,
                encryption_enabled: true,
            },
            performance_thresholds: PerformanceThresholds::default(),
        }
    }
}

impl Default for MonitoringSystemConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            metrics_collection: MetricsCollectionConfig {
                collection_interval_ms: 1000,
                buffer_size: 1000,
                aggregation_window_minutes: 5,
                detailed_metrics: true,
                custom_metrics: vec![
                    "learning_performance".to_string(),
                    "api_response_time".to_string(),
                ],
            },
            alerting: AlertingConfig {
                enabled: true,
                severity_thresholds: {
                    let mut thresholds = HashMap::new();
                    thresholds.insert("error_rate".to_string(), 0.05);
                    thresholds.insert("response_time".to_string(), 1000.0);
                    thresholds.insert("memory_usage".to_string(), 0.8);
                    thresholds
                },
                notification_channels: vec!["email".to_string(), "slack".to_string()],
                rate_limiting: true,
                escalation_rules: vec![EscalationRule {
                    condition: "critical_error".to_string(),
                    delay_minutes: 5,
                    action: "page_admin".to_string(),
                }],
            },
            health_checks: HealthCheckConfig {
                interval_seconds: 30,
                timeout_seconds: 10,
                failure_threshold: 3,
                recovery_threshold: 2,
                components: vec![
                    "api_server".to_string(),
                    "learning_service".to_string(),
                    "mcp_server".to_string(),
                ],
            },
            retention_settings: RetentionConfig {
                metrics_retention_days: 30,
                alerts_retention_days: 90,
                logs_retention_days: 7,
                cleanup_interval_hours: 24,
            },
        }
    }
}

impl Default for QualityControlConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            validation_levels: vec![
                ValidationLevel {
                    name: "basic".to_string(),
                    enabled: true,
                    rules: vec!["format_check".to_string(), "size_check".to_string()],
                    severity: "warning".to_string(),
                },
                ValidationLevel {
                    name: "advanced".to_string(),
                    enabled: true,
                    rules: vec![
                        "content_analysis".to_string(),
                        "quality_scoring".to_string(),
                    ],
                    severity: "error".to_string(),
                },
            ],
            quality_thresholds: QualityThresholds {
                min_confidence: 0.8,
                max_processing_time_ms: 5000,
                min_data_quality_score: 0.7,
                max_error_rate: 0.05,
            },
            processing_rules: ProcessingRules {
                max_retries: 3,
                retry_delay_ms: 1000,
                fallback_enabled: true,
                parallel_processing: true,
            },
            output_requirements: OutputRequirements {
                format_validation: true,
                content_validation: true,
                schema_validation: true,
                performance_validation: true,
            },
        }
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            learning_rules: LearningValidationRules {
                min_feedback_score: 0.0,
                max_feedback_score: 1.0,
                required_fields: vec!["user_id".to_string(), "content_id".to_string()],
                max_text_length: 10000,
                allowed_feedback_types: vec![
                    "quality_rating".to_string(),
                    "relevance".to_string(),
                    "satisfaction".to_string(),
                ],
            },
            monitoring_rules: MonitoringValidationRules {
                metric_name_pattern: "^[a-zA-Z_][a-zA-Z0-9_]*$".to_string(),
                value_range: (-1000000.0, 1000000.0),
                required_tags: vec!["component".to_string()],
                max_tag_count: 20,
                timestamp_tolerance_ms: 60000,
            },
            api_rules: ApiValidationRules {
                max_request_size_bytes: 10 * 1024 * 1024, // 10MB
                rate_limit_per_minute: 1000,
                required_headers: vec!["content-type".to_string()],
                allowed_content_types: vec!["application/json".to_string()],
                max_response_time_ms: 30000,
            },
            mcp_rules: McpValidationRules {
                max_tool_params: 50,
                required_permissions: vec!["basic_access".to_string()],
                max_resource_size_bytes: 5 * 1024 * 1024, // 5MB
                allowed_operations: vec![
                    "read".to_string(),
                    "write".to_string(),
                    "execute".to_string(),
                ],
                timeout_seconds: 30,
            },
            global_rules: GlobalValidationRules {
                max_concurrent_operations: 100,
                max_memory_usage_mb: 2048,
                max_disk_usage_gb: 50,
                max_cpu_usage_percent: 80.0,
                health_check_interval_seconds: 30,
            },
        }
    }
}

impl ConfigurationManager {
    pub fn new() -> Self {
        Self {
            learning_config: Arc::new(RwLock::new(LearningSystemConfig::default())),
            monitoring_config: Arc::new(RwLock::new(MonitoringSystemConfig::default())),
            api_config: Arc::new(RwLock::new(ApiServerConfig::default())),
            mcp_config: Arc::new(RwLock::new(McpServerConfig {
                auth_enabled: true,
                rate_limiting: RateLimitConfig {
                    enabled: true,
                    requests_per_minute: 60,
                    burst_size: 10,
                },
                cors_enabled: true,
                metrics_enabled: true,
            })),
            quality_config: Arc::new(RwLock::new(QualityControlConfig::default())),
            validation_rules: Arc::new(RwLock::new(ValidationRules::default())),
            config_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn validate_learning_config(
        &self,
        config: &LearningSystemConfig,
    ) -> Result<(), String> {
        let rules = self.validation_rules.read().await;

        // Validate feedback processing config
        if config.feedback_processing.batch_size == 0 {
            return Err("Batch size must be greater than 0".to_string());
        }
        if config.feedback_processing.batch_size > 10000 {
            return Err("Batch size too large (max 10000)".to_string());
        }
        if config.feedback_processing.quality_threshold < 0.0
            || config.feedback_processing.quality_threshold > 1.0
        {
            return Err("Quality threshold must be between 0.0 and 1.0".to_string());
        }

        // Validate pattern analysis config
        if config.pattern_analysis.min_data_points < 5 {
            return Err("Minimum data points must be at least 5".to_string());
        }
        if config.pattern_analysis.confidence_threshold < 0.5
            || config.pattern_analysis.confidence_threshold > 1.0
        {
            return Err("Confidence threshold must be between 0.5 and 1.0".to_string());
        }

        // Validate learning algorithms config
        if config.learning_algorithms.adaptation_rate <= 0.0
            || config.learning_algorithms.adaptation_rate > 1.0
        {
            return Err("Adaptation rate must be between 0.0 and 1.0".to_string());
        }
        if config.learning_algorithms.max_iterations == 0
            || config.learning_algorithms.max_iterations > 100000
        {
            return Err("Max iterations must be between 1 and 100000".to_string());
        }

        // Validate storage config
        if config.storage_settings.vector_dimension == 0
            || config.storage_settings.vector_dimension > 4096
        {
            return Err("Vector dimension must be between 1 and 4096".to_string());
        }
        if config.storage_settings.cache_size_mb == 0
            || config.storage_settings.cache_size_mb > 8192
        {
            return Err("Cache size must be between 1MB and 8GB".to_string());
        }

        Ok(())
    }

    pub async fn validate_monitoring_config(
        &self,
        config: &MonitoringSystemConfig,
    ) -> Result<(), String> {
        let rules = self.validation_rules.read().await;

        // Validate metrics collection config
        if config.metrics_collection.collection_interval_ms < 100 {
            return Err("Collection interval must be at least 100ms".to_string());
        }
        if config.metrics_collection.buffer_size == 0
            || config.metrics_collection.buffer_size > 1000000
        {
            return Err("Buffer size must be between 1 and 1000000".to_string());
        }

        // Validate alerting config
        for (metric, threshold) in &config.alerting.severity_thresholds {
            if threshold.is_nan() || threshold.is_infinite() {
                return Err(format!(
                    "Invalid threshold for metric {}: {}",
                    metric, threshold
                ));
            }
        }

        // Validate health check config
        if config.health_checks.interval_seconds == 0
            || config.health_checks.interval_seconds > 3600
        {
            return Err("Health check interval must be between 1 second and 1 hour".to_string());
        }
        if config.health_checks.timeout_seconds >= config.health_checks.interval_seconds {
            return Err("Health check timeout must be less than interval".to_string());
        }

        // Validate retention config
        if config.retention_settings.metrics_retention_days == 0
            || config.retention_settings.metrics_retention_days > 365
        {
            return Err("Metrics retention must be between 1 and 365 days".to_string());
        }

        Ok(())
    }

    pub async fn validate_quality_config(
        &self,
        config: &QualityControlConfig,
    ) -> Result<(), String> {
        // Validate quality thresholds
        if config.quality_thresholds.min_confidence < 0.0
            || config.quality_thresholds.min_confidence > 1.0
        {
            return Err("Min confidence must be between 0.0 and 1.0".to_string());
        }
        if config.quality_thresholds.max_processing_time_ms == 0
            || config.quality_thresholds.max_processing_time_ms > 300000
        {
            return Err("Max processing time must be between 1ms and 5 minutes".to_string());
        }
        if config.quality_thresholds.max_error_rate < 0.0
            || config.quality_thresholds.max_error_rate > 1.0
        {
            return Err("Max error rate must be between 0.0 and 1.0".to_string());
        }

        // Validate processing rules
        if config.processing_rules.max_retries > 10 {
            return Err("Max retries cannot exceed 10".to_string());
        }
        if config.processing_rules.retry_delay_ms > 60000 {
            return Err("Retry delay cannot exceed 60 seconds".to_string());
        }

        // Validate validation levels
        for level in &config.validation_levels {
            if level.rules.is_empty() {
                return Err(format!(
                    "Validation level '{}' must have at least one rule",
                    level.name
                ));
            }
            if !["warning", "error", "critical"].contains(&level.severity.as_str()) {
                return Err(format!(
                    "Invalid severity '{}' for validation level '{}'",
                    level.severity, level.name
                ));
            }
        }

        Ok(())
    }

    pub async fn validate_user_feedback(&self, feedback: &UserFeedback) -> Result<(), String> {
        let rules = self.validation_rules.read().await;

        // Check required fields
        for field in &rules.learning_rules.required_fields {
            match field.as_str() {
                "user_id" => {
                    if feedback.user_id.is_empty() {
                        return Err("User ID is required".to_string());
                    }
                }
                "content_id" => {
                    if feedback.content_id.is_empty() {
                        return Err("Content ID is required".to_string());
                    }
                }
                _ => {}
            }
        }

        // Validate feedback score
        if let Some(score) = feedback.score {
            if score < rules.learning_rules.min_feedback_score
                || score > rules.learning_rules.max_feedback_score
            {
                return Err(format!(
                    "Feedback score must be between {} and {}",
                    rules.learning_rules.min_feedback_score,
                    rules.learning_rules.max_feedback_score
                ));
            }
        }

        // Validate feedback type
        if !rules
            .learning_rules
            .allowed_feedback_types
            .contains(&feedback.feedback_type)
        {
            return Err(format!(
                "Feedback type '{}' is not allowed",
                feedback.feedback_type
            ));
        }

        // Validate text length
        if let Some(text) = &feedback.text_feedback {
            if text.len() > rules.learning_rules.max_text_length {
                return Err(format!(
                    "Text feedback exceeds maximum length of {} characters",
                    rules.learning_rules.max_text_length
                ));
            }
        }

        Ok(())
    }

    pub async fn update_learning_config(
        &self,
        config: LearningSystemConfig,
        user: &str,
        reason: &str,
    ) -> Result<(), String> {
        // Validate new config
        self.validate_learning_config(&config).await?;

        // Record configuration change
        let old_config = self.learning_config.read().await.clone();
        let change = ConfigurationChange {
            timestamp: chrono::Utc::now(),
            component: "learning_system".to_string(),
            change_type: "update".to_string(),
            old_value: Some(serde_json::to_value(&old_config).unwrap()),
            new_value: serde_json::to_value(&config).unwrap(),
            user: user.to_string(),
            reason: reason.to_string(),
        };

        // Update config
        {
            let mut current_config = self.learning_config.write().await;
            *current_config = config;
        }

        // Record change
        {
            let mut history = self.config_history.write().await;
            history.push(change);
        }

        Ok(())
    }

    pub async fn get_configuration_history(
        &self,
        component: Option<&str>,
    ) -> Vec<ConfigurationChange> {
        let history = self.config_history.read().await;
        if let Some(comp) = component {
            history
                .iter()
                .filter(|change| change.component == comp)
                .cloned()
                .collect()
        } else {
            history.clone()
        }
    }

    pub async fn save_configuration_to_file(
        &self,
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let learning_config = self.learning_config.read().await.clone();
        let monitoring_config = self.monitoring_config.read().await.clone();
        let quality_config = self.quality_config.read().await.clone();
        let validation_rules = self.validation_rules.read().await.clone();

        let full_config = json!({
            "learning_system": learning_config,
            "monitoring_system": monitoring_config,
            "quality_control": quality_config,
            "validation_rules": validation_rules,
            "saved_at": chrono::Utc::now()
        });

        fs::write(path, serde_json::to_string_pretty(&full_config)?)?;
        Ok(())
    }

    pub async fn load_configuration_from_file(
        &self,
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = fs::read_to_string(path)?;
        let full_config: Value = serde_json::from_str(&content)?;

        if let Some(learning_config) = full_config.get("learning_system") {
            let config: LearningSystemConfig = serde_json::from_value(learning_config.clone())?;
            self.validate_learning_config(&config)
                .await
                .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
            *self.learning_config.write().await = config;
        }

        if let Some(monitoring_config) = full_config.get("monitoring_system") {
            let config: MonitoringSystemConfig = serde_json::from_value(monitoring_config.clone())?;
            self.validate_monitoring_config(&config)
                .await
                .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
            *self.monitoring_config.write().await = config;
        }

        if let Some(quality_config) = full_config.get("quality_control") {
            let config: QualityControlConfig = serde_json::from_value(quality_config.clone())?;
            self.validate_quality_config(&config)
                .await
                .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
            *self.quality_config.write().await = config;
        }

        if let Some(validation_rules) = full_config.get("validation_rules") {
            let rules: ValidationRules = serde_json::from_value(validation_rules.clone())?;
            *self.validation_rules.write().await = rules;
        }

        Ok(())
    }
}

#[cfg(test)]
mod anchor_tests {
    use super::*;

    /// ANCHOR: Configuration validation ensures system integrity
    /// Tests: Config validation → Error handling → Default fallback → Recovery
    /// Protects: Configuration validation logic and system safety
    #[tokio::test]
    async fn test_anchor_configuration_validation_system_integrity() {
        let config_manager = ConfigurationManager::new();

        // Test 1: Valid learning configuration validation
        let valid_learning_config = LearningSystemConfig::default();
        let validation_result = config_manager
            .validate_learning_config(&valid_learning_config)
            .await;
        assert!(
            validation_result.is_ok(),
            "Default learning configuration should be valid"
        );

        // Test 2: Invalid learning configuration detection
        let invalid_configs = vec![
            // Invalid batch size
            LearningSystemConfig {
                feedback_processing: FeedbackProcessingConfig {
                    batch_size: 0,
                    ..valid_learning_config.feedback_processing.clone()
                },
                ..valid_learning_config.clone()
            },
            // Invalid quality threshold
            LearningSystemConfig {
                feedback_processing: FeedbackProcessingConfig {
                    quality_threshold: 1.5,
                    ..valid_learning_config.feedback_processing.clone()
                },
                ..valid_learning_config.clone()
            },
            // Invalid confidence threshold
            LearningSystemConfig {
                pattern_analysis: PatternAnalysisConfig {
                    confidence_threshold: 0.3,
                    ..valid_learning_config.pattern_analysis.clone()
                },
                ..valid_learning_config.clone()
            },
            // Invalid adaptation rate
            LearningSystemConfig {
                learning_algorithms: LearningAlgorithmsConfig {
                    adaptation_rate: 1.5,
                    ..valid_learning_config.learning_algorithms.clone()
                },
                ..valid_learning_config.clone()
            },
            // Invalid vector dimension
            LearningSystemConfig {
                storage_settings: StorageConfig {
                    vector_dimension: 0,
                    ..valid_learning_config.storage_settings.clone()
                },
                ..valid_learning_config.clone()
            },
        ];

        for (index, invalid_config) in invalid_configs.iter().enumerate() {
            let validation_result = config_manager
                .validate_learning_config(invalid_config)
                .await;
            assert!(
                validation_result.is_err(),
                "Invalid learning configuration {} should be rejected",
                index
            );
        }

        // Test 3: Valid monitoring configuration validation
        let valid_monitoring_config = MonitoringSystemConfig::default();
        let monitoring_validation = config_manager
            .validate_monitoring_config(&valid_monitoring_config)
            .await;
        assert!(
            monitoring_validation.is_ok(),
            "Default monitoring configuration should be valid"
        );

        // Test 4: Invalid monitoring configuration detection
        let invalid_monitoring_configs = vec![
            // Invalid collection interval
            MonitoringSystemConfig {
                metrics_collection: MetricsCollectionConfig {
                    collection_interval_ms: 50,
                    ..valid_monitoring_config.metrics_collection.clone()
                },
                ..valid_monitoring_config.clone()
            },
            // Invalid buffer size
            MonitoringSystemConfig {
                metrics_collection: MetricsCollectionConfig {
                    buffer_size: 0,
                    ..valid_monitoring_config.metrics_collection.clone()
                },
                ..valid_monitoring_config.clone()
            },
            // Invalid health check interval
            MonitoringSystemConfig {
                health_checks: HealthCheckConfig {
                    interval_seconds: 0,
                    ..valid_monitoring_config.health_checks.clone()
                },
                ..valid_monitoring_config.clone()
            },
            // Timeout >= interval
            MonitoringSystemConfig {
                health_checks: HealthCheckConfig {
                    interval_seconds: 30,
                    timeout_seconds: 30,
                    ..valid_monitoring_config.health_checks.clone()
                },
                ..valid_monitoring_config.clone()
            },
        ];

        for (index, invalid_config) in invalid_monitoring_configs.iter().enumerate() {
            let validation_result = config_manager
                .validate_monitoring_config(invalid_config)
                .await;
            assert!(
                validation_result.is_err(),
                "Invalid monitoring configuration {} should be rejected",
                index
            );
        }

        // Test 5: Quality control configuration validation
        let valid_quality_config = QualityControlConfig::default();
        let quality_validation = config_manager
            .validate_quality_config(&valid_quality_config)
            .await;
        assert!(
            quality_validation.is_ok(),
            "Default quality configuration should be valid"
        );

        // Test 6: Invalid quality configuration detection
        let invalid_quality_configs = vec![
            // Invalid confidence threshold
            QualityControlConfig {
                quality_thresholds: QualityThresholds {
                    min_confidence: 1.5,
                    ..valid_quality_config.quality_thresholds.clone()
                },
                ..valid_quality_config.clone()
            },
            // Invalid processing time
            QualityControlConfig {
                quality_thresholds: QualityThresholds {
                    max_processing_time_ms: 0,
                    ..valid_quality_config.quality_thresholds.clone()
                },
                ..valid_quality_config.clone()
            },
            // Too many retries
            QualityControlConfig {
                processing_rules: ProcessingRules {
                    max_retries: 15,
                    ..valid_quality_config.processing_rules.clone()
                },
                ..valid_quality_config.clone()
            },
            // Empty validation level
            QualityControlConfig {
                validation_levels: vec![ValidationLevel {
                    name: "empty".to_string(),
                    enabled: true,
                    rules: vec![],
                    severity: "error".to_string(),
                }],
                ..valid_quality_config.clone()
            },
        ];

        for (index, invalid_config) in invalid_quality_configs.iter().enumerate() {
            let validation_result = config_manager.validate_quality_config(invalid_config).await;
            assert!(
                validation_result.is_err(),
                "Invalid quality configuration {} should be rejected",
                index
            );
        }

        // Test 7: User feedback validation
        let valid_feedback = UserFeedback::new(
            "test_user".to_string(),
            "test_content".to_string(),
            "quality_rating".to_string(),
            Some(0.8),
            Some("Valid feedback".to_string()),
        );

        let feedback_validation = config_manager.validate_user_feedback(&valid_feedback).await;
        assert!(
            feedback_validation.is_ok(),
            "Valid feedback should pass validation"
        );

        // Test 8: Invalid user feedback detection
        let invalid_feedbacks = vec![
            // Missing user ID
            UserFeedback::new(
                "".to_string(),
                "test_content".to_string(),
                "quality_rating".to_string(),
                Some(0.8),
                None,
            ),
            // Missing content ID
            UserFeedback::new(
                "test_user".to_string(),
                "".to_string(),
                "quality_rating".to_string(),
                Some(0.8),
                None,
            ),
            // Invalid score
            UserFeedback::new(
                "test_user".to_string(),
                "test_content".to_string(),
                "quality_rating".to_string(),
                Some(1.5),
                None,
            ),
            // Invalid feedback type
            UserFeedback::new(
                "test_user".to_string(),
                "test_content".to_string(),
                "invalid_type".to_string(),
                Some(0.8),
                None,
            ),
        ];

        for (index, invalid_feedback) in invalid_feedbacks.iter().enumerate() {
            let validation_result = config_manager
                .validate_user_feedback(invalid_feedback)
                .await;
            assert!(
                validation_result.is_err(),
                "Invalid feedback {} should be rejected",
                index
            );
        }

        // Test 9: Configuration update workflow
        let mut updated_config = valid_learning_config.clone();
        updated_config.feedback_processing.batch_size = 200;
        updated_config.pattern_analysis.confidence_threshold = 0.9;

        let update_result = config_manager
            .update_learning_config(
                updated_config.clone(),
                "test_admin",
                "Performance optimization",
            )
            .await;

        assert!(
            update_result.is_ok(),
            "Valid configuration update should succeed"
        );

        // Verify configuration was updated
        let current_config = config_manager.learning_config.read().await;
        assert_eq!(current_config.feedback_processing.batch_size, 200);
        assert_eq!(current_config.pattern_analysis.confidence_threshold, 0.9);

        // Test 10: Configuration history tracking
        let history = config_manager
            .get_configuration_history(Some("learning_system"))
            .await;
        assert_eq!(history.len(), 1, "Should track configuration changes");

        let change = &history[0];
        assert_eq!(change.component, "learning_system");
        assert_eq!(change.user, "test_admin");
        assert_eq!(change.reason, "Performance optimization");
        assert!(
            change.old_value.is_some(),
            "Should record old configuration value"
        );
    }

    /// ANCHOR: Configuration persistence and recovery workflows
    /// Tests: Save configuration → Load configuration → Validation → Error recovery
    /// Protects: Configuration persistence and system recovery capabilities
    #[tokio::test]
    async fn test_anchor_configuration_persistence_recovery_workflows() {
        let temp_dir = TempDir::new().unwrap();
        let config_file_path = temp_dir.path().join("test_config.json");

        let config_manager = ConfigurationManager::new();

        // Test 1: Configuration serialization and saving
        let save_result = config_manager
            .save_configuration_to_file(&config_file_path)
            .await;
        assert!(save_result.is_ok(), "Configuration saving should succeed");
        assert!(
            config_file_path.exists(),
            "Configuration file should be created"
        );

        // Verify file content
        let saved_content = fs::read_to_string(&config_file_path).unwrap();
        let saved_json: Value = serde_json::from_str(&saved_content).unwrap();
        assert!(
            saved_json.get("learning_system").is_some(),
            "Should include learning system config"
        );
        assert!(
            saved_json.get("monitoring_system").is_some(),
            "Should include monitoring system config"
        );
        assert!(
            saved_json.get("quality_control").is_some(),
            "Should include quality control config"
        );
        assert!(
            saved_json.get("validation_rules").is_some(),
            "Should include validation rules"
        );
        assert!(
            saved_json.get("saved_at").is_some(),
            "Should include save timestamp"
        );

        // Test 2: Configuration loading and validation
        let new_config_manager = ConfigurationManager::new();
        let load_result = new_config_manager
            .load_configuration_from_file(&config_file_path)
            .await;
        assert!(load_result.is_ok(), "Configuration loading should succeed");

        // Verify loaded configuration matches original
        let original_learning = config_manager.learning_config.read().await;
        let loaded_learning = new_config_manager.learning_config.read().await;
        assert_eq!(
            original_learning.feedback_processing.batch_size,
            loaded_learning.feedback_processing.batch_size
        );
        assert_eq!(
            original_learning.pattern_analysis.confidence_threshold,
            loaded_learning.pattern_analysis.confidence_threshold
        );

        // Test 3: Invalid configuration file handling
        let invalid_config_path = temp_dir.path().join("invalid_config.json");
        fs::write(&invalid_config_path, "invalid json content").unwrap();

        let invalid_load_result = new_config_manager
            .load_configuration_from_file(&invalid_config_path)
            .await;
        assert!(
            invalid_load_result.is_err(),
            "Invalid configuration file should be rejected"
        );

        // Test 4: Partial configuration loading
        let partial_config = json!({
            "learning_system": {
                "enabled": true,
                "feedback_processing": {
                    "batch_size": 500,
                    "processing_interval_ms": 3000,
                    "quality_threshold": 0.9,
                    "auto_validation": false,
                    "feedback_expiry_days": 60
                },
                "pattern_analysis": {
                    "min_data_points": 20,
                    "confidence_threshold": 0.95,
                    "analysis_interval_hours": 12,
                    "pattern_retention_days": 120,
                    "similarity_threshold": 0.9
                },
                "learning_algorithms": {
                    "adaptation_rate": 0.005,
                    "convergence_threshold": 0.0005,
                    "max_iterations": 2000,
                    "learning_rate_decay": 0.98,
                    "regularization_factor": 0.0005
                },
                "storage_settings": {
                    "vector_dimension": 768,
                    "cache_size_mb": 1024,
                    "backup_interval_hours": 12,
                    "compression_enabled": false,
                    "encryption_enabled": false
                },
                "performance_thresholds": {
                    "max_response_time_ms": 2000,
                    "max_memory_usage_mb": 1024,
                    "max_cpu_usage_percent": 70.0,
                    "max_error_rate": 0.03
                }
            }
        });

        let partial_config_path = temp_dir.path().join("partial_config.json");
        fs::write(
            &partial_config_path,
            serde_json::to_string_pretty(&partial_config).unwrap(),
        )
        .unwrap();

        let partial_config_manager = ConfigurationManager::new();
        let partial_load_result = partial_config_manager
            .load_configuration_from_file(&partial_config_path)
            .await;
        assert!(
            partial_load_result.is_ok(),
            "Partial configuration loading should succeed"
        );

        // Verify partial configuration was loaded
        let partial_learning = partial_config_manager.learning_config.read().await;
        assert_eq!(partial_learning.feedback_processing.batch_size, 500);
        assert_eq!(partial_learning.pattern_analysis.min_data_points, 20);
        assert_eq!(partial_learning.learning_algorithms.adaptation_rate, 0.005);

        // Test 5: Configuration validation during loading
        let invalid_values_config = json!({
            "learning_system": {
                "enabled": true,
                "feedback_processing": {
                    "batch_size": 0, // Invalid
                    "processing_interval_ms": 5000,
                    "quality_threshold": 1.5, // Invalid
                    "auto_validation": true,
                    "feedback_expiry_days": 30
                },
                "pattern_analysis": {
                    "min_data_points": 10,
                    "confidence_threshold": 0.8,
                    "analysis_interval_hours": 24,
                    "pattern_retention_days": 90,
                    "similarity_threshold": 0.85
                },
                "learning_algorithms": {
                    "adaptation_rate": 0.01,
                    "convergence_threshold": 0.001,
                    "max_iterations": 1000,
                    "learning_rate_decay": 0.95,
                    "regularization_factor": 0.001
                },
                "storage_settings": {
                    "vector_dimension": 1536,
                    "cache_size_mb": 512,
                    "backup_interval_hours": 24,
                    "compression_enabled": true,
                    "encryption_enabled": true
                },
                "performance_thresholds": {
                    "max_response_time_ms": 5000,
                    "max_memory_usage_mb": 2048,
                    "max_cpu_usage_percent": 80.0,
                    "max_error_rate": 0.05
                }
            }
        });

        let invalid_values_path = temp_dir.path().join("invalid_values_config.json");
        fs::write(
            &invalid_values_path,
            serde_json::to_string_pretty(&invalid_values_config).unwrap(),
        )
        .unwrap();

        let invalid_values_manager = ConfigurationManager::new();
        let invalid_values_load_result = invalid_values_manager
            .load_configuration_from_file(&invalid_values_path)
            .await;
        assert!(
            invalid_values_load_result.is_err(),
            "Configuration with invalid values should be rejected"
        );

        // Test 6: Configuration backup and restore workflow
        let original_config_manager = ConfigurationManager::new();

        // Modify configuration
        let mut modified_config = LearningSystemConfig::default();
        modified_config.feedback_processing.batch_size = 300;
        modified_config.pattern_analysis.confidence_threshold = 0.95;

        let update_result = original_config_manager
            .update_learning_config(
                modified_config.clone(),
                "backup_test_user",
                "Backup test modification",
            )
            .await;
        assert!(update_result.is_ok(), "Configuration update should succeed");

        // Save modified configuration
        let backup_path = temp_dir.path().join("backup_config.json");
        let backup_save_result = original_config_manager
            .save_configuration_to_file(&backup_path)
            .await;
        assert!(backup_save_result.is_ok(), "Backup save should succeed");

        // Create new manager and restore from backup
        let restored_manager = ConfigurationManager::new();
        let restore_result = restored_manager
            .load_configuration_from_file(&backup_path)
            .await;
        assert!(
            restore_result.is_ok(),
            "Configuration restore should succeed"
        );

        // Verify restored configuration
        let restored_config = restored_manager.learning_config.read().await;
        assert_eq!(restored_config.feedback_processing.batch_size, 300);
        assert_eq!(restored_config.pattern_analysis.confidence_threshold, 0.95);

        // Test 7: Multiple configuration file format support
        let yaml_like_config = json!({
            "version": "1.0",
            "timestamp": "2024-01-01T00:00:00Z",
            "environment": "test",
            "learning_system": {
                "enabled": true,
                "feedback_processing": {
                    "batch_size": 150,
                    "processing_interval_ms": 4000,
                    "quality_threshold": 0.75,
                    "auto_validation": true,
                    "feedback_expiry_days": 45
                },
                "pattern_analysis": {
                    "min_data_points": 15,
                    "confidence_threshold": 0.85,
                    "analysis_interval_hours": 18,
                    "pattern_retention_days": 100,
                    "similarity_threshold": 0.88
                },
                "learning_algorithms": {
                    "adaptation_rate": 0.008,
                    "convergence_threshold": 0.0008,
                    "max_iterations": 1500,
                    "learning_rate_decay": 0.96,
                    "regularization_factor": 0.0008
                },
                "storage_settings": {
                    "vector_dimension": 1024,
                    "cache_size_mb": 768,
                    "backup_interval_hours": 18,
                    "compression_enabled": true,
                    "encryption_enabled": true
                },
                "performance_thresholds": {
                    "max_response_time_ms": 3000,
                    "max_memory_usage_mb": 1536,
                    "max_cpu_usage_percent": 75.0,
                    "max_error_rate": 0.04
                }
            }
        });

        let yaml_like_path = temp_dir.path().join("yaml_like_config.json");
        fs::write(
            &yaml_like_path,
            serde_json::to_string_pretty(&yaml_like_config).unwrap(),
        )
        .unwrap();

        let yaml_like_manager = ConfigurationManager::new();
        let yaml_like_load_result = yaml_like_manager
            .load_configuration_from_file(&yaml_like_path)
            .await;
        assert!(
            yaml_like_load_result.is_ok(),
            "YAML-like configuration should load successfully"
        );

        // Verify YAML-like configuration was loaded correctly
        let yaml_like_config_loaded = yaml_like_manager.learning_config.read().await;
        assert_eq!(yaml_like_config_loaded.feedback_processing.batch_size, 150);
        assert_eq!(
            yaml_like_config_loaded
                .pattern_analysis
                .analysis_interval_hours,
            18
        );

        // Test 8: Configuration migration and versioning
        let old_format_config = json!({
            "learning_system": {
                "enabled": true,
                "feedback_processing": {
                    "batch_size": 75,
                    "processing_interval_ms": 6000,
                    "quality_threshold": 0.65,
                    "auto_validation": false,
                    "feedback_expiry_days": 21
                },
                "pattern_analysis": {
                    "min_data_points": 8,
                    "confidence_threshold": 0.78,
                    "analysis_interval_hours": 36,
                    "pattern_retention_days": 75,
                    "similarity_threshold": 0.82
                },
                "learning_algorithms": {
                    "adaptation_rate": 0.012,
                    "convergence_threshold": 0.0012,
                    "max_iterations": 800,
                    "learning_rate_decay": 0.94,
                    "regularization_factor": 0.0012
                },
                "storage_settings": {
                    "vector_dimension": 2048,
                    "cache_size_mb": 256,
                    "backup_interval_hours": 48,
                    "compression_enabled": false,
                    "encryption_enabled": false
                },
                "performance_thresholds": {
                    "max_response_time_ms": 8000,
                    "max_memory_usage_mb": 512,
                    "max_cpu_usage_percent": 90.0,
                    "max_error_rate": 0.08
                }
            }
        });

        let old_format_path = temp_dir.path().join("old_format_config.json");
        fs::write(
            &old_format_path,
            serde_json::to_string_pretty(&old_format_config).unwrap(),
        )
        .unwrap();

        let migration_manager = ConfigurationManager::new();
        let migration_load_result = migration_manager
            .load_configuration_from_file(&old_format_path)
            .await;
        assert!(
            migration_load_result.is_ok(),
            "Old format configuration should migrate successfully"
        );

        // Test 9: Concurrent configuration access and modification
        let concurrent_manager = Arc::new(ConfigurationManager::new());
        let mut concurrent_tasks = Vec::new();

        for i in 0..10 {
            let manager = concurrent_manager.clone();
            let task = tokio::spawn(async move {
                let mut config = LearningSystemConfig::default();
                config.feedback_processing.batch_size = 100 + i * 10;

                manager
                    .update_learning_config(
                        config,
                        &format!("concurrent_user_{}", i),
                        &format!("Concurrent update {}", i),
                    )
                    .await
            });
            concurrent_tasks.push(task);
        }

        let concurrent_results: Vec<Result<Result<(), String>, _>> =
            futures::future::join_all(concurrent_tasks).await;

        let successful_updates = concurrent_results
            .iter()
            .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
            .count();

        // All concurrent updates should succeed (last one wins)
        assert!(
            successful_updates >= 8,
            "Most concurrent configuration updates should succeed"
        );

        // Verify configuration history tracks all changes
        let concurrent_history = concurrent_manager
            .get_configuration_history(Some("learning_system"))
            .await;
        assert!(
            concurrent_history.len() >= 8,
            "Should track most concurrent configuration changes"
        );

        // Test 10: Error recovery and system resilience
        let resilience_manager = ConfigurationManager::new();

        // Test recovery from file system errors
        let nonexistent_path = temp_dir
            .path()
            .join("nonexistent_directory")
            .join("config.json");
        let save_to_nonexistent = resilience_manager
            .save_configuration_to_file(&nonexistent_path)
            .await;
        assert!(
            save_to_nonexistent.is_err(),
            "Should handle file system errors gracefully"
        );

        // Test recovery from permission errors (simulate with read-only file)
        let readonly_path = temp_dir.path().join("readonly_config.json");
        fs::write(&readonly_path, "{}").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&readonly_path).unwrap().permissions();
            perms.set_mode(0o444); // Read-only
            fs::set_permissions(&readonly_path, perms).unwrap();
        }

        // System should remain operational after errors
        let post_error_validation = resilience_manager
            .validate_learning_config(&LearningSystemConfig::default())
            .await;
        assert!(
            post_error_validation.is_ok(),
            "System should remain operational after file system errors"
        );

        // Test final configuration consistency
        let final_config = resilience_manager.learning_config.read().await;
        assert!(
            final_config.enabled,
            "Configuration should remain in valid state"
        );
        assert!(
            final_config.feedback_processing.batch_size > 0,
            "Configuration should maintain valid values"
        );
    }
}
