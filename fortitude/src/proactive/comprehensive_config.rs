// ABOUTME: Comprehensive proactive configuration management system
//! # Comprehensive Proactive Configuration Management
//!
//! This module provides a complete configuration management system for proactive research settings,
//! extending the existing gap detection configuration with additional capabilities for background
//! research, notifications, performance monitoring, user preferences, and workspace-specific settings.
//!
//! ## Key Features:
//! - Multi-source configuration loading (files, environment variables, CLI args)
//! - Hot-reload capabilities for runtime configuration updates
//! - Configuration validation with comprehensive error messages
//! - Export/import functionality for sharing settings
//! - Configuration versioning and migration support
//! - Integration with CLI, API, and MCP configuration interfaces
//! - Environment-specific configuration profiles (dev, prod, staging)
//! - Workspace-specific configuration overrides

use chrono::{DateTime, Utc};
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::fs;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

// Re-export existing gap detection configuration
pub use super::config::{ConfigurationError as GapConfigurationError, GapDetectionConfig};

/// Comprehensive errors that can occur during configuration management
#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("Invalid configuration value: {field} - {message}")]
    InvalidValue { field: String, message: String },

    #[error("Invalid threshold value: {field} must be between {min} and {max}, got {value}")]
    InvalidThreshold {
        field: String,
        min: f64,
        max: f64,
        value: f64,
    },

    #[error("Invalid pattern: {context} - {pattern}: {error}")]
    InvalidPattern {
        context: String,
        pattern: String,
        error: String,
    },

    #[error("Configuration preset not found: {preset}")]
    PresetNotFound { preset: String },

    #[error("Unsupported configuration version: {version}")]
    UnsupportedVersion { version: String },

    #[error("Conflicting settings: {message}")]
    ConflictingSettings { message: String },

    #[error("Missing required configuration: {field}")]
    MissingRequired { field: String },

    #[error("File operation failed: {operation} - {path}: {error}")]
    FileOperation {
        operation: String,
        path: String,
        error: String,
    },

    #[error("Validation failed: {0}")]
    Validation(#[from] ValidationErrors),

    #[error("Serialization error: {format} - {error}")]
    Serialization { format: String, error: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Watch error: {0}")]
    Watch(#[from] notify::Error),

    #[error("Gap configuration error: {0}")]
    GapConfig(#[from] GapConfigurationError),
}

/// Comprehensive proactive configuration with all subsystems
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ProactiveConfig {
    /// Configuration version for migration support
    #[serde(default = "current_version")]
    pub version: String,

    /// Configuration metadata
    #[serde(default)]
    pub metadata: ConfigMetadata,

    /// Gap analysis configuration (extends existing gap detection)
    #[validate(nested)]
    pub gap_analysis: Option<GapAnalysisConfig>,

    /// Background research configuration
    #[validate(nested)]
    pub background_research: Option<BackgroundResearchConfig>,

    /// Notification system configuration
    #[validate(nested)]
    pub notifications: Option<NotificationConfig>,

    /// Performance and resource management configuration
    #[validate(nested)]
    pub performance: Option<PerformanceConfig>,

    /// User preferences and personalization
    #[validate(nested)]
    pub user_preferences: Option<UserPreferenceConfig>,

    /// Workspace-specific configuration
    #[validate(nested)]
    pub workspace: Option<WorkspaceConfig>,

    /// Environment-specific overrides
    #[serde(default)]
    pub environment_overrides: HashMap<String, serde_json::Value>,
}

/// Configuration metadata for tracking and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    /// Unique identifier for this configuration
    pub id: Uuid,

    /// Human-readable name for the configuration
    pub name: String,

    /// Description of the configuration purpose
    pub description: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modification timestamp
    pub modified_at: DateTime<Utc>,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Author information
    pub author: String,

    /// Environment this configuration targets
    pub target_environment: String,
}

/// Enhanced gap analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GapAnalysisConfig {
    /// Scan interval in seconds
    #[validate(range(min = 1, max = 86400))]
    pub scan_intervals_seconds: u64,

    /// File patterns to monitor
    #[validate(length(min = 1))]
    pub file_patterns: Vec<String>,

    /// Detection rules for different gap types
    #[validate(length(min = 1))]
    pub detection_rules: Vec<String>,

    /// Confidence threshold for gap detection
    #[validate(range(min = 0.0, max = 1.0))]
    pub confidence_threshold: f64,

    /// Enable semantic analysis for gap validation
    pub enable_semantic_analysis: bool,

    /// Maximum number of files to scan per interval
    #[validate(range(min = 1, max = 100000))]
    pub max_files_per_scan: u64,

    /// Priority boost for certain file types
    pub priority_file_types: HashMap<String, f64>,

    /// Custom gap detection rules
    pub custom_rules: Vec<CustomGapRule>,

    /// Integration with existing gap detection config
    pub gap_detection: Option<GapDetectionConfig>,
}

/// Custom gap detection rule
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CustomGapRule {
    /// Rule name
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    /// Rule description
    pub description: String,

    /// Pattern to match
    #[validate(length(min = 1))]
    pub pattern: String,

    /// Rule priority (1-10)
    #[validate(range(min = 1, max = 10))]
    pub priority: u8,

    /// Whether rule is enabled
    pub enabled: bool,

    /// Confidence boost for this rule
    #[validate(range(min = 0.0, max = 2.0))]
    pub confidence_boost: f64,
}

/// Background research configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct BackgroundResearchConfig {
    /// Maximum concurrent research tasks
    #[validate(range(min = 1, max = 20))]
    pub max_concurrent_tasks: u32,

    /// Rate limiting: requests per minute
    #[validate(range(min = 1, max = 1000))]
    pub rate_limit_requests_per_minute: u32,

    /// Enable automatic scheduling
    pub scheduling_enabled: bool,

    /// Keywords that boost research priority
    pub priority_keywords: Vec<String>,

    /// Research timeout in seconds
    #[validate(range(min = 30, max = 3600))]
    pub research_timeout_seconds: u64,

    /// Enable auto-prioritization based on context
    pub auto_prioritization_enabled: bool,

    /// Research quality thresholds
    #[validate(nested)]
    pub quality_thresholds: ResearchQualityConfig,

    /// Integration settings
    #[validate(nested)]
    pub integration: ResearchIntegrationConfig,
}

/// Research quality configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ResearchQualityConfig {
    /// Minimum confidence for research results
    #[validate(range(min = 0.0, max = 1.0))]
    pub min_confidence: f64,

    /// Minimum research depth (number of sources)
    #[validate(range(min = 1, max = 20))]
    pub min_sources: u32,

    /// Enable quality validation
    pub enable_validation: bool,

    /// Retry failed research attempts
    pub retry_failed_research: bool,

    /// Maximum retry attempts
    #[validate(range(min = 1, max = 5))]
    pub max_retry_attempts: u32,
}

/// Research integration configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ResearchIntegrationConfig {
    /// Enable Claude API integration
    pub enable_claude_integration: bool,

    /// Enable vector database integration
    pub enable_vector_integration: bool,

    /// Enable external API integrations
    pub enable_external_apis: bool,

    /// API rate limiting configuration
    pub api_rate_limits: HashMap<String, u32>,

    /// Integration timeouts
    pub integration_timeouts: HashMap<String, u64>,
}

/// Comprehensive notification configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NotificationConfig {
    /// Available notification channels
    #[validate(length(min = 1))]
    pub channels: Vec<String>,

    /// Delivery preferences by channel
    pub delivery_preferences: HashMap<String, bool>,

    /// Delivery rules and conditions
    #[validate(length(min = 1))]
    pub delivery_rules: Vec<NotificationRule>,

    /// Rate limiting for notifications
    #[validate(nested)]
    pub rate_limiting: NotificationRateLimiting,

    /// Template configuration
    pub templates: HashMap<String, NotificationTemplate>,

    /// Channel-specific settings
    pub channel_settings: HashMap<String, serde_json::Value>,
}

/// Notification delivery rule
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NotificationRule {
    /// Rule name
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    /// Condition for triggering notification
    pub condition: String,

    /// Channels to use for this rule
    pub channels: Vec<String>,

    /// Priority level (1-10)
    #[validate(range(min = 1, max = 10))]
    pub priority: u8,

    /// Whether rule is enabled
    pub enabled: bool,

    /// Throttling settings
    pub throttling: Option<NotificationThrottling>,
}

/// Notification rate limiting
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NotificationRateLimiting {
    /// Maximum notifications per hour
    #[validate(range(min = 1, max = 1000))]
    pub max_per_hour: u32,

    /// Maximum notifications per day
    #[validate(range(min = 1, max = 10000))]
    pub max_per_day: u32,

    /// Enable burst protection
    pub enable_burst_protection: bool,

    /// Burst threshold
    #[validate(range(min = 1, max = 100))]
    pub burst_threshold: u32,
}

/// Notification throttling configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NotificationThrottling {
    /// Minimum time between notifications (seconds)
    #[validate(range(min = 1, max = 86400))]
    pub min_interval_seconds: u64,

    /// Enable deduplication
    pub enable_deduplication: bool,

    /// Deduplication window (seconds)
    #[validate(range(min = 60, max = 3600))]
    pub deduplication_window_seconds: u64,
}

/// Notification template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTemplate {
    /// Template subject
    pub subject: String,

    /// Template body
    pub body: String,

    /// Template format (text, html, markdown)
    pub format: String,

    /// Template variables
    pub variables: Vec<String>,
}

/// Performance and resource management configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PerformanceConfig {
    /// Resource limits
    #[validate(nested)]
    pub resource_limits: ResourceLimitsConfig,

    /// Enable performance monitoring
    pub monitoring_enabled: bool,

    /// Alert thresholds
    pub alert_thresholds: HashMap<String, f64>,

    /// Caching configuration
    #[validate(nested)]
    pub caching: CachingConfig,

    /// Optimization settings
    #[validate(nested)]
    pub optimization: OptimizationConfig,
}

/// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ResourceLimitsConfig {
    /// Maximum memory usage in MB
    #[validate(range(min = 100, max = 32768))]
    pub max_memory_mb: u64,

    /// Maximum CPU usage percentage
    #[validate(range(min = 10, max = 100))]
    pub max_cpu_percent: u8,

    /// Maximum disk usage in MB
    #[validate(range(min = 100, max = 102400))]
    pub max_disk_mb: u64,

    /// Maximum network bandwidth in Mbps
    #[validate(range(min = 1, max = 1000))]
    pub max_network_mbps: u32,
}

/// Caching configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CachingConfig {
    /// Enable caching
    pub enabled: bool,

    /// Cache size limit in MB
    #[validate(range(min = 10, max = 10240))]
    pub size_limit_mb: u64,

    /// Cache TTL in seconds
    #[validate(range(min = 60, max = 86400))]
    pub ttl_seconds: u64,

    /// Cache eviction policy
    pub eviction_policy: String,

    /// Cache compression
    pub enable_compression: bool,
}

/// Optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct OptimizationConfig {
    /// Enable automatic optimization
    pub auto_optimization_enabled: bool,

    /// Optimization strategies
    pub strategies: Vec<String>,

    /// Optimization interval in seconds
    #[validate(range(min = 300, max = 86400))]
    pub optimization_interval_seconds: u64,

    /// Performance targets
    pub performance_targets: HashMap<String, f64>,
}

/// User preferences and personalization
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UserPreferenceConfig {
    /// Preferred research domains
    #[validate(length(min = 1))]
    pub research_domains: Vec<String>,

    /// Notification frequency in hours
    #[validate(range(min = 1, max = 168))]
    pub notification_frequency_hours: u64,

    /// Preferred output formats
    #[validate(length(min = 1))]
    pub preferred_formats: Vec<String>,

    /// Language preferences
    pub language_preferences: Vec<String>,

    /// Personalization settings
    pub personalization: PersonalizationConfig,

    /// Interface preferences
    pub interface_preferences: InterfacePreferences,
}

/// Personalization configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PersonalizationConfig {
    /// Enable adaptive learning
    pub enable_adaptive_learning: bool,

    /// Learning rate
    #[validate(range(min = 0.0, max = 1.0))]
    pub learning_rate: f64,

    /// User behavior tracking
    pub enable_behavior_tracking: bool,

    /// Preference learning
    pub enable_preference_learning: bool,

    /// Feedback integration
    pub enable_feedback_integration: bool,
}

/// Interface preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfacePreferences {
    /// Preferred interface theme
    pub theme: String,

    /// Display preferences
    pub display_preferences: HashMap<String, serde_json::Value>,

    /// Accessibility settings
    pub accessibility: HashMap<String, bool>,

    /// Custom shortcuts
    pub shortcuts: HashMap<String, String>,
}

/// Workspace-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct WorkspaceConfig {
    /// Project paths to monitor
    #[validate(length(min = 1))]
    pub project_paths: Vec<PathBuf>,

    /// Patterns to exclude from monitoring
    pub exclude_patterns: Vec<String>,

    /// Enable auto-discovery of project structure
    pub auto_discovery_enabled: bool,

    /// Workspace-specific overrides
    pub workspace_overrides: HashMap<String, serde_json::Value>,

    /// Project type detection
    pub project_type_detection: ProjectTypeConfig,

    /// Integration settings
    pub integrations: WorkspaceIntegrationConfig,
}

/// Project type detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTypeConfig {
    /// Enable automatic project type detection
    pub auto_detection_enabled: bool,

    /// Project type indicators
    pub type_indicators: HashMap<String, Vec<String>>,

    /// Custom project types
    pub custom_types: Vec<CustomProjectType>,
}

/// Custom project type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomProjectType {
    /// Type name
    pub name: String,

    /// Detection patterns
    pub patterns: Vec<String>,

    /// Configuration overrides for this type
    pub config_overrides: serde_json::Value,
}

/// Workspace integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceIntegrationConfig {
    /// Enable Git integration
    pub enable_git_integration: bool,

    /// Enable IDE integration
    pub enable_ide_integration: bool,

    /// Enable build system integration
    pub enable_build_integration: bool,

    /// Integration-specific settings
    pub integration_settings: HashMap<String, serde_json::Value>,
}

/// Configuration manager for handling all configuration operations
pub struct ProactiveConfigManager {
    /// Current configuration
    current_config: Arc<RwLock<ProactiveConfig>>,

    /// File watcher for hot reload
    _watcher: Option<notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>>,

    /// Hot reload callback
    #[allow(dead_code)] // TODO: Will be used for configuration hot reload notifications
    reload_callback: Option<Box<dyn Fn(ProactiveConfig) + Send + Sync>>,

    /// Performance metrics
    performance_metrics: Arc<RwLock<HashMap<String, serde_json::Value>>>,

    /// Performance alerts
    performance_alerts: Arc<RwLock<Vec<PerformanceAlert>>>,

    /// Enable performance monitoring
    performance_monitoring_enabled: bool,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    /// Alert type
    pub alert_type: String,

    /// Alert message
    pub message: String,

    /// Alert timestamp
    pub timestamp: DateTime<Utc>,

    /// Alert severity
    pub severity: String,

    /// Alert metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

// Helper functions
fn current_version() -> String {
    "2.0".to_string()
}

impl Default for ConfigMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: "Default Proactive Configuration".to_string(),
            description: "Default configuration for proactive research system".to_string(),
            created_at: now,
            modified_at: now,
            tags: vec!["default".to_string()],
            author: "system".to_string(),
            target_environment: "development".to_string(),
        }
    }
}

impl Default for ProactiveConfig {
    fn default() -> Self {
        Self {
            version: current_version(),
            metadata: ConfigMetadata::default(),
            gap_analysis: Some(GapAnalysisConfig::default()),
            background_research: Some(BackgroundResearchConfig::default()),
            notifications: Some(NotificationConfig::default()),
            performance: Some(PerformanceConfig::default()),
            user_preferences: Some(UserPreferenceConfig::default()),
            workspace: Some(WorkspaceConfig::default()),
            environment_overrides: HashMap::new(),
        }
    }
}

impl Default for GapAnalysisConfig {
    fn default() -> Self {
        Self {
            scan_intervals_seconds: 300,
            file_patterns: vec!["*.rs".to_string(), "*.md".to_string(), "*.toml".to_string()],
            detection_rules: vec![
                "todo".to_string(),
                "fixme".to_string(),
                "hack".to_string(),
                "bug".to_string(),
            ],
            confidence_threshold: 0.7,
            enable_semantic_analysis: true,
            max_files_per_scan: 1000,
            priority_file_types: {
                let mut map = HashMap::new();
                map.insert("rs".to_string(), 1.2);
                map.insert("md".to_string(), 1.0);
                map.insert("toml".to_string(), 0.8);
                map
            },
            custom_rules: Vec::new(),
            gap_detection: Some(GapDetectionConfig::default()),
        }
    }
}

impl Default for BackgroundResearchConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 3,
            rate_limit_requests_per_minute: 50,
            scheduling_enabled: true,
            priority_keywords: vec![
                "urgent".to_string(),
                "critical".to_string(),
                "security".to_string(),
                "performance".to_string(),
            ],
            research_timeout_seconds: 300,
            auto_prioritization_enabled: true,
            quality_thresholds: ResearchQualityConfig::default(),
            integration: ResearchIntegrationConfig::default(),
        }
    }
}

impl Default for ResearchQualityConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.6,
            min_sources: 2,
            enable_validation: true,
            retry_failed_research: true,
            max_retry_attempts: 3,
        }
    }
}

impl Default for ResearchIntegrationConfig {
    fn default() -> Self {
        Self {
            enable_claude_integration: true,
            enable_vector_integration: true,
            enable_external_apis: false,
            api_rate_limits: {
                let mut map = HashMap::new();
                map.insert("claude".to_string(), 50);
                map.insert("vector".to_string(), 100);
                map
            },
            integration_timeouts: {
                let mut map = HashMap::new();
                map.insert("claude".to_string(), 30);
                map.insert("vector".to_string(), 10);
                map
            },
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            channels: vec![
                "email".to_string(),
                "desktop".to_string(),
                "slack".to_string(),
            ],
            delivery_preferences: {
                let mut map = HashMap::new();
                map.insert("email".to_string(), true);
                map.insert("desktop".to_string(), true);
                map.insert("slack".to_string(), false);
                map
            },
            delivery_rules: vec![NotificationRule::default()],
            rate_limiting: NotificationRateLimiting::default(),
            templates: HashMap::new(),
            channel_settings: HashMap::new(),
        }
    }
}

impl Default for NotificationRule {
    fn default() -> Self {
        Self {
            name: "Default Rule".to_string(),
            condition: "gap_detected".to_string(),
            channels: vec!["desktop".to_string()],
            priority: 5,
            enabled: true,
            throttling: Some(NotificationThrottling::default()),
        }
    }
}

impl Default for NotificationRateLimiting {
    fn default() -> Self {
        Self {
            max_per_hour: 10,
            max_per_day: 50,
            enable_burst_protection: true,
            burst_threshold: 5,
        }
    }
}

impl Default for NotificationThrottling {
    fn default() -> Self {
        Self {
            min_interval_seconds: 300,
            enable_deduplication: true,
            deduplication_window_seconds: 900,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            resource_limits: ResourceLimitsConfig::default(),
            monitoring_enabled: true,
            alert_thresholds: {
                let mut map = HashMap::new();
                map.insert("cpu_usage".to_string(), 80.0);
                map.insert("memory_usage".to_string(), 80.0);
                map.insert("disk_usage".to_string(), 90.0);
                map.insert("response_time_ms".to_string(), 1000.0);
                map
            },
            caching: CachingConfig::default(),
            optimization: OptimizationConfig::default(),
        }
    }
}

impl Default for ResourceLimitsConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 2048,
            max_cpu_percent: 80,
            max_disk_mb: 10240,
            max_network_mbps: 100,
        }
    }
}

impl Default for CachingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            size_limit_mb: 512,
            ttl_seconds: 3600,
            eviction_policy: "lru".to_string(),
            enable_compression: true,
        }
    }
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            auto_optimization_enabled: true,
            strategies: vec![
                "memory_optimization".to_string(),
                "cpu_optimization".to_string(),
                "cache_optimization".to_string(),
            ],
            optimization_interval_seconds: 3600,
            performance_targets: {
                let mut map = HashMap::new();
                map.insert("response_time_ms".to_string(), 500.0);
                map.insert("cpu_usage_percent".to_string(), 60.0);
                map.insert("memory_usage_percent".to_string(), 70.0);
                map
            },
        }
    }
}

impl Default for UserPreferenceConfig {
    fn default() -> Self {
        Self {
            research_domains: vec![
                "software_development".to_string(),
                "documentation".to_string(),
                "best_practices".to_string(),
            ],
            notification_frequency_hours: 4,
            preferred_formats: vec!["markdown".to_string(), "json".to_string()],
            language_preferences: vec!["en".to_string()],
            personalization: PersonalizationConfig::default(),
            interface_preferences: InterfacePreferences::default(),
        }
    }
}

impl Default for PersonalizationConfig {
    fn default() -> Self {
        Self {
            enable_adaptive_learning: true,
            learning_rate: 0.1,
            enable_behavior_tracking: false,
            enable_preference_learning: true,
            enable_feedback_integration: true,
        }
    }
}

impl Default for InterfacePreferences {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            display_preferences: HashMap::new(),
            accessibility: HashMap::new(),
            shortcuts: HashMap::new(),
        }
    }
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            project_paths: vec![PathBuf::from(".")],
            exclude_patterns: vec![
                "target/**".to_string(),
                "node_modules/**".to_string(),
                ".git/**".to_string(),
                "*.tmp".to_string(),
            ],
            auto_discovery_enabled: true,
            workspace_overrides: HashMap::new(),
            project_type_detection: ProjectTypeConfig::default(),
            integrations: WorkspaceIntegrationConfig::default(),
        }
    }
}

impl Default for ProjectTypeConfig {
    fn default() -> Self {
        Self {
            auto_detection_enabled: true,
            type_indicators: {
                let mut map = HashMap::new();
                map.insert(
                    "rust".to_string(),
                    vec!["Cargo.toml".to_string(), "src/main.rs".to_string()],
                );
                map.insert(
                    "javascript".to_string(),
                    vec!["package.json".to_string(), "src/index.js".to_string()],
                );
                map.insert(
                    "python".to_string(),
                    vec![
                        "requirements.txt".to_string(),
                        "setup.py".to_string(),
                        "pyproject.toml".to_string(),
                    ],
                );
                map
            },
            custom_types: Vec::new(),
        }
    }
}

impl Default for WorkspaceIntegrationConfig {
    fn default() -> Self {
        Self {
            enable_git_integration: true,
            enable_ide_integration: true,
            enable_build_integration: true,
            integration_settings: HashMap::new(),
        }
    }
}

impl ProactiveConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self {
            current_config: Arc::new(RwLock::new(ProactiveConfig::default())),
            _watcher: None,
            reload_callback: None,
            performance_metrics: Arc::new(RwLock::new(HashMap::new())),
            performance_alerts: Arc::new(RwLock::new(Vec::new())),
            performance_monitoring_enabled: false,
        }
    }

    /// Create a new configuration manager with performance monitoring
    pub fn with_performance_monitoring(enabled: bool) -> Self {
        Self {
            current_config: Arc::new(RwLock::new(ProactiveConfig::default())),
            _watcher: None,
            reload_callback: None,
            performance_metrics: Arc::new(RwLock::new(HashMap::new())),
            performance_alerts: Arc::new(RwLock::new(Vec::new())),
            performance_monitoring_enabled: enabled,
        }
    }

    /// Get the current configuration
    pub async fn get_current_config(&self) -> ProactiveConfig {
        self.current_config.read().await.clone()
    }

    /// Load configuration from multiple sources with precedence
    pub async fn load_from_sources(
        &self,
        config_file: Option<&Path>,
        load_from_env: bool,
        cli_args: Option<HashMap<String, String>>,
    ) -> Result<ProactiveConfig, ConfigurationError> {
        let start_time = std::time::Instant::now();

        // Start with default configuration
        let mut config = ProactiveConfig::default();

        // Load from file if provided
        if let Some(file_path) = config_file {
            let file_config = Self::load_from_file_internal(file_path).await?;
            config = self.merge_configs(config, file_config)?;
        }

        // Load from environment variables if enabled
        if load_from_env {
            let env_config = Self::load_from_env_internal().await?;
            config = self.merge_configs(config, env_config)?;
        }

        // Apply CLI arguments if provided
        if let Some(args) = cli_args {
            let cli_config = Self::parse_cli_args(args)?;
            config = self.merge_configs(config, cli_config)?;
        }

        // Validate the final configuration
        config.validate()?;

        // Update current configuration
        {
            let mut current = self.current_config.write().await;
            *current = config.clone();
        }

        // Record performance metrics
        if self.performance_monitoring_enabled {
            self.record_performance_metric(
                "config_load_time_ms",
                serde_json::Value::Number((start_time.elapsed().as_millis() as u64).into()),
            )
            .await;
        }

        Ok(config)
    }

    /// Load configuration from file
    pub async fn load_from_file(&self, path: &Path) -> Result<ProactiveConfig, ConfigurationError> {
        Self::load_from_file_internal(path).await
    }

    /// Internal method to load configuration from file
    async fn load_from_file_internal(path: &Path) -> Result<ProactiveConfig, ConfigurationError> {
        let content =
            fs::read_to_string(path)
                .await
                .map_err(|e| ConfigurationError::FileOperation {
                    operation: "read".to_string(),
                    path: path.display().to_string(),
                    error: e.to_string(),
                })?;

        let config = match path.extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::from_str(&content)?,
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content)?,
            Some("toml") => {
                toml::from_str(&content).map_err(|e| ConfigurationError::Serialization {
                    format: "toml".to_string(),
                    error: e.to_string(),
                })?
            }
            _ => {
                // Try to detect format by content
                if content.trim_start().starts_with('{') {
                    serde_json::from_str(&content)?
                } else if content.contains("---") || content.contains(':') {
                    serde_yaml::from_str(&content)?
                } else {
                    toml::from_str(&content).map_err(|e| ConfigurationError::Serialization {
                        format: "toml".to_string(),
                        error: e.to_string(),
                    })?
                }
            }
        };

        Ok(config)
    }

    /// Load configuration from environment variables
    async fn load_from_env_internal() -> Result<ProactiveConfig, ConfigurationError> {
        let mut config = ProactiveConfig::default();

        // Load gap analysis settings
        if let Ok(scan_interval) = std::env::var("FORTITUDE_PROACTIVE_GAP_SCAN_INTERVAL") {
            if let Some(ref mut gap_config) = config.gap_analysis {
                gap_config.scan_intervals_seconds =
                    scan_interval
                        .parse()
                        .map_err(|_| ConfigurationError::InvalidValue {
                            field: "scan_intervals_seconds".to_string(),
                            message: format!("Invalid scan interval: {scan_interval}"),
                        })?;
            }
        }

        if let Ok(confidence) = std::env::var("FORTITUDE_PROACTIVE_CONFIDENCE_THRESHOLD") {
            if let Some(ref mut gap_config) = config.gap_analysis {
                gap_config.confidence_threshold =
                    confidence
                        .parse()
                        .map_err(|_| ConfigurationError::InvalidValue {
                            field: "confidence_threshold".to_string(),
                            message: format!("Invalid confidence threshold: {confidence}"),
                        })?;
            }
        }

        // Load background research settings
        if let Ok(max_tasks) = std::env::var("FORTITUDE_PROACTIVE_MAX_CONCURRENT_TASKS") {
            if let Some(ref mut research_config) = config.background_research {
                research_config.max_concurrent_tasks =
                    max_tasks
                        .parse()
                        .map_err(|_| ConfigurationError::InvalidValue {
                            field: "max_concurrent_tasks".to_string(),
                            message: format!("Invalid max concurrent tasks: {max_tasks}"),
                        })?;
            }
        }

        // Load notification settings
        if let Ok(channels) = std::env::var("FORTITUDE_PROACTIVE_NOTIFICATION_CHANNELS") {
            if let Some(ref mut notification_config) = config.notifications {
                notification_config.channels =
                    channels.split(',').map(|s| s.trim().to_string()).collect();
            }
        }

        // Load performance settings
        if let Ok(max_memory) = std::env::var("FORTITUDE_PROACTIVE_MAX_MEMORY_MB") {
            if let Some(ref mut perf_config) = config.performance {
                perf_config.resource_limits.max_memory_mb =
                    max_memory
                        .parse()
                        .map_err(|_| ConfigurationError::InvalidValue {
                            field: "max_memory_mb".to_string(),
                            message: format!("Invalid max memory: {max_memory}"),
                        })?;
            }
        }

        Ok(config)
    }

    /// Parse CLI arguments into configuration
    fn parse_cli_args(
        args: HashMap<String, String>,
    ) -> Result<ProactiveConfig, ConfigurationError> {
        let mut config = ProactiveConfig::default();

        for (key, value) in args {
            match key.as_str() {
                "scan-interval" => {
                    if let Some(ref mut gap_config) = config.gap_analysis {
                        gap_config.scan_intervals_seconds =
                            value
                                .parse()
                                .map_err(|_| ConfigurationError::InvalidValue {
                                    field: "scan_intervals_seconds".to_string(),
                                    message: format!("Invalid scan interval: {value}"),
                                })?;
                    }
                }
                "confidence-threshold" => {
                    if let Some(ref mut gap_config) = config.gap_analysis {
                        gap_config.confidence_threshold =
                            value
                                .parse()
                                .map_err(|_| ConfigurationError::InvalidValue {
                                    field: "confidence_threshold".to_string(),
                                    message: format!("Invalid confidence threshold: {value}"),
                                })?;
                    }
                }
                "max-concurrent-tasks" => {
                    if let Some(ref mut research_config) = config.background_research {
                        research_config.max_concurrent_tasks =
                            value
                                .parse()
                                .map_err(|_| ConfigurationError::InvalidValue {
                                    field: "max_concurrent_tasks".to_string(),
                                    message: format!("Invalid max concurrent tasks: {value}"),
                                })?;
                    }
                }
                _ => {
                    // Ignore unknown CLI arguments
                }
            }
        }

        Ok(config)
    }

    /// Merge two configurations with precedence
    fn merge_configs(
        &self,
        base: ProactiveConfig,
        override_config: ProactiveConfig,
    ) -> Result<ProactiveConfig, ConfigurationError> {
        // For this implementation, we'll do a simple field-by-field merge
        // In a production system, you might want more sophisticated merging logic

        let mut result = base;

        // Merge gap analysis configuration
        if override_config.gap_analysis.is_some() {
            result.gap_analysis = override_config.gap_analysis;
        }

        // Merge background research configuration
        if override_config.background_research.is_some() {
            result.background_research = override_config.background_research;
        }

        // Merge notification configuration
        if override_config.notifications.is_some() {
            result.notifications = override_config.notifications;
        }

        // Merge performance configuration
        if override_config.performance.is_some() {
            result.performance = override_config.performance;
        }

        // Merge user preferences
        if override_config.user_preferences.is_some() {
            result.user_preferences = override_config.user_preferences;
        }

        // Merge workspace configuration
        if override_config.workspace.is_some() {
            result.workspace = override_config.workspace;
        }

        // Merge environment overrides
        for (key, value) in override_config.environment_overrides {
            result.environment_overrides.insert(key, value);
        }

        Ok(result)
    }

    /// Enable hot reload functionality
    pub async fn enable_hot_reload<F>(
        &mut self,
        config_file: &Path,
        callback: F,
    ) -> Result<(), ConfigurationError>
    where
        F: Fn(ProactiveConfig) + Send + Sync + 'static,
    {
        let (tx, mut rx) = mpsc::channel(100);
        let config_path = config_file.to_path_buf();
        let config_path_for_watcher = config_path.clone();
        let current_config = self.current_config.clone();

        // Set up file watcher
        let mut debouncer = new_debouncer(
            Duration::from_millis(250),
            move |res: DebounceEventResult| match res {
                Ok(events) => {
                    for event in events {
                        if event.path == config_path_for_watcher {
                            let _ = tx.try_send(event);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("File watch error: {e:?}");
                }
            },
        )?;

        debouncer
            .watcher()
            .watch(config_file, RecursiveMode::NonRecursive)?;

        // Store the watcher to keep it alive
        self._watcher = Some(debouncer);

        // Spawn task to handle file changes
        tokio::spawn(async move {
            while let Some(_event) = rx.recv().await {
                // Reload configuration
                match Self::load_from_file_internal(&config_path).await {
                    Ok(new_config) => {
                        // Validate new configuration
                        if new_config.validate().is_ok() {
                            // Update current configuration
                            {
                                let mut current = current_config.write().await;
                                *current = new_config.clone();
                            }

                            // Call the callback
                            callback(new_config);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to reload configuration: {e:?}");
                    }
                }
            }
        });

        Ok(())
    }

    /// Load configuration with migration support
    pub async fn load_with_migration(
        &self,
        path: &Path,
    ) -> Result<ProactiveConfig, ConfigurationError> {
        let mut config = Self::load_from_file_internal(path).await?;

        // Check version and migrate if necessary
        match config.version.as_str() {
            "1.0" => {
                config = self.migrate_from_v1(config).await?;
            }
            "2.0" => {
                // Current version, no migration needed
            }
            version => {
                return Err(ConfigurationError::UnsupportedVersion {
                    version: version.to_string(),
                });
            }
        }

        Ok(config)
    }

    /// Migrate configuration from version 1.0 to 2.0
    async fn migrate_from_v1(
        &self,
        mut config: ProactiveConfig,
    ) -> Result<ProactiveConfig, ConfigurationError> {
        // Update version
        config.version = "2.0".to_string();

        // Migrate any structural changes here
        // For this example, we'll assume the migration is straightforward

        // Update metadata
        config.metadata.modified_at = Utc::now();

        Ok(config)
    }

    /// Load configuration for specific environment
    pub async fn load_for_environment(&self) -> Result<ProactiveConfig, ConfigurationError> {
        let environment =
            std::env::var("FORTITUDE_ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        let mut config = ProactiveConfig::default();

        // Apply environment-specific settings
        match environment.as_str() {
            "development" => {
                if let Some(ref mut gap_config) = config.gap_analysis {
                    gap_config.scan_intervals_seconds = 60; // More frequent scans
                }
                if let Some(ref mut perf_config) = config.performance {
                    perf_config.resource_limits.max_cpu_percent = 90; // Allow higher CPU usage
                    perf_config.monitoring_enabled = false; // Less monitoring overhead
                }
            }
            "staging" => {
                if let Some(ref mut gap_config) = config.gap_analysis {
                    gap_config.scan_intervals_seconds = 300; // Moderate scan frequency
                }
                if let Some(ref mut perf_config) = config.performance {
                    perf_config.monitoring_enabled = true; // Enable monitoring
                }
            }
            "production" => {
                if let Some(ref mut gap_config) = config.gap_analysis {
                    gap_config.scan_intervals_seconds = 600; // Less frequent scans
                }
                if let Some(ref mut research_config) = config.background_research {
                    research_config.rate_limit_requests_per_minute = 30; // Conservative rate limiting
                }
                if let Some(ref mut perf_config) = config.performance {
                    perf_config.monitoring_enabled = true; // Full monitoring
                }
            }
            _ => {
                // Unknown environment, use defaults
            }
        }

        config.metadata.target_environment = environment;

        Ok(config)
    }

    /// Load workspace-specific configuration
    pub async fn load_for_workspace(
        &self,
        base_config: &ProactiveConfig,
        workspace_path: &Path,
    ) -> Result<ProactiveConfig, ConfigurationError> {
        let mut config = base_config.clone();

        // Look for workspace-specific configuration file
        let workspace_config_file = workspace_path.join(".fortitude_config.json");
        if workspace_config_file.exists() {
            let workspace_overrides = Self::load_from_file_internal(&workspace_config_file).await?;
            config = self.merge_configs(config, workspace_overrides)?;
        }

        Ok(config)
    }

    /// Record performance metric
    async fn record_performance_metric(&self, metric_name: &str, value: serde_json::Value) {
        if self.performance_monitoring_enabled {
            let mut metrics = self.performance_metrics.write().await;
            metrics.insert(metric_name.to_string(), value);
        }
    }

    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> HashMap<String, serde_json::Value> {
        self.performance_metrics.read().await.clone()
    }

    /// Get performance alerts
    pub async fn get_performance_alerts(&self) -> Vec<PerformanceAlert> {
        self.performance_alerts.read().await.clone()
    }
}

impl ProactiveConfig {
    /// Validate configuration with custom business rules
    pub fn validate_with_custom_rules(&self) -> Result<(), ConfigurationError> {
        // First run standard validation
        self.validate()?;

        // Check for conflicting settings
        if let (Some(gap_config), Some(research_config), Some(perf_config)) = (
            &self.gap_analysis,
            &self.background_research,
            &self.performance,
        ) {
            // Check if scan frequency and concurrency are compatible with resource limits
            if gap_config.scan_intervals_seconds < 60
                && research_config.max_concurrent_tasks > 5
                && perf_config.resource_limits.max_cpu_percent < 70
            {
                return Err(ConfigurationError::ConflictingSettings {
                    message: "High scan frequency and concurrency require higher CPU limits"
                        .to_string(),
                });
            }

            // Check memory compatibility
            let estimated_memory_per_task = 100u64; // MB per task estimate
            let estimated_total_memory =
                research_config.max_concurrent_tasks as u64 * estimated_memory_per_task;
            if estimated_total_memory > perf_config.resource_limits.max_memory_mb / 2 {
                return Err(ConfigurationError::ConflictingSettings {
                    message: format!(
                        "Concurrent tasks ({}) may exceed memory limits (estimated {} MB needed, {} MB available)",
                        research_config.max_concurrent_tasks,
                        estimated_total_memory,
                        perf_config.resource_limits.max_memory_mb
                    ),
                });
            }
        }

        Ok(())
    }

    /// Export configuration to file with specified format
    pub async fn export_to_file(
        &self,
        path: &Path,
        format: &str,
    ) -> Result<(), ConfigurationError> {
        let content = match format {
            "json" => serde_json::to_string_pretty(self)?,
            "yaml" => serde_yaml::to_string(self)?,
            "toml" => {
                toml::to_string_pretty(self).map_err(|e| ConfigurationError::Serialization {
                    format: "toml".to_string(),
                    error: e.to_string(),
                })?
            }
            _ => {
                return Err(ConfigurationError::InvalidValue {
                    field: "format".to_string(),
                    message: format!("Unsupported format: {format}"),
                });
            }
        };

        fs::write(path, content)
            .await
            .map_err(|e| ConfigurationError::FileOperation {
                operation: "write".to_string(),
                path: path.display().to_string(),
                error: e.to_string(),
            })?;

        Ok(())
    }

    /// Import configuration from file
    pub async fn import_from_file(path: &Path) -> Result<ProactiveConfig, ConfigurationError> {
        ProactiveConfigManager::load_from_file_internal(path).await
    }

    /// Create configuration preset
    pub fn preset(name: &str) -> Result<ProactiveConfig, ConfigurationError> {
        match name {
            "development" => Ok(Self::development_preset()),
            "production" => Ok(Self::production_preset()),
            "research" => Ok(Self::research_preset()),
            "minimal" => Ok(Self::minimal_preset()),
            _ => Err(ConfigurationError::PresetNotFound {
                preset: name.to_string(),
            }),
        }
    }

    /// Development preset configuration
    fn development_preset() -> Self {
        let mut config = Self::default();

        // Frequent scans for quick feedback
        if let Some(ref mut gap_config) = config.gap_analysis {
            gap_config.scan_intervals_seconds = 60;
            gap_config.enable_semantic_analysis = true;
        }

        // Enable desktop notifications for immediate feedback
        if let Some(ref mut notification_config) = config.notifications {
            notification_config
                .delivery_preferences
                .insert("desktop".to_string(), true);
            notification_config
                .delivery_preferences
                .insert("email".to_string(), false);
        }

        // Less monitoring overhead for development
        if let Some(ref mut perf_config) = config.performance {
            perf_config.monitoring_enabled = false;
            perf_config.resource_limits.max_cpu_percent = 90;
        }

        config.metadata.name = "Development Configuration".to_string();
        config.metadata.target_environment = "development".to_string();

        config
    }

    /// Production preset configuration
    fn production_preset() -> Self {
        let mut config = Self::default();

        // Conservative scan frequency for production
        if let Some(ref mut gap_config) = config.gap_analysis {
            gap_config.scan_intervals_seconds = 600;
            gap_config.confidence_threshold = 0.8;
        }

        // Conservative rate limiting
        if let Some(ref mut research_config) = config.background_research {
            research_config.rate_limit_requests_per_minute = 30;
            research_config.max_concurrent_tasks = 2;
        }

        // Full monitoring enabled
        if let Some(ref mut perf_config) = config.performance {
            perf_config.monitoring_enabled = true;
            perf_config.resource_limits.max_cpu_percent = 70;
        }

        config.metadata.name = "Production Configuration".to_string();
        config.metadata.target_environment = "production".to_string();

        config
    }

    /// Research-focused preset configuration
    fn research_preset() -> Self {
        let mut config = Self::default();

        // Enhanced analysis capabilities
        if let Some(ref mut gap_config) = config.gap_analysis {
            gap_config.enable_semantic_analysis = true;
            gap_config.confidence_threshold = 0.6; // Lower threshold for more research
        }

        // Higher concurrency for research
        if let Some(ref mut research_config) = config.background_research {
            research_config.max_concurrent_tasks = 5;
            research_config.auto_prioritization_enabled = true;
        }

        // More resources allocated
        if let Some(ref mut perf_config) = config.performance {
            perf_config.resource_limits.max_memory_mb = 4096;
            perf_config.resource_limits.max_cpu_percent = 85;
        }

        config.metadata.name = "Research Configuration".to_string();
        config.metadata.target_environment = "research".to_string();

        config
    }

    /// Minimal preset configuration
    fn minimal_preset() -> Self {
        let mut config = Self::default();

        // Minimal analysis
        if let Some(ref mut gap_config) = config.gap_analysis {
            gap_config.enable_semantic_analysis = false;
            gap_config.scan_intervals_seconds = 600;
        }

        // Single-threaded research
        if let Some(ref mut research_config) = config.background_research {
            research_config.max_concurrent_tasks = 1;
            research_config.rate_limit_requests_per_minute = 10;
        }

        // Minimal notifications
        if let Some(ref mut notification_config) = config.notifications {
            notification_config.channels = vec!["desktop".to_string()];
        }

        // Minimal monitoring
        if let Some(ref mut perf_config) = config.performance {
            perf_config.monitoring_enabled = false;
            perf_config.resource_limits.max_memory_mb = 512;
        }

        config.metadata.name = "Minimal Configuration".to_string();
        config.metadata.target_environment = "minimal".to_string();

        config
    }
}

impl Default for ProactiveConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
