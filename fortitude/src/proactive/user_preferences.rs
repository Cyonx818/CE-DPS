// ABOUTME: User preference integration for priority customization in proactive research mode
//! This module provides comprehensive user preference management that allows users to customize
//! priority scoring weights, domain preferences, gap type priorities, workflow modes, and other
//! personalization options for the proactive research system.
//!
//! The system supports:
//! - Preference profiles for different contexts and user workflows
//! - Customizable priority weights for gap types, domains, and context factors
//! - Workflow modes (development, review, documentation, learning)
//! - Expertise levels (beginner, intermediate, expert) with adaptive priority adjustments
//! - Personal filters for gap types, file patterns, and priority thresholds
//! - Persistent preference storage with JSON serialization
//! - Real-time preference updates without system restart
//!
//! Performance Requirements:
//! - Preference loading should be <10ms for typical operations
//! - Preference application should add <5ms to priority scoring
//! - Support multiple preference profiles without memory overhead
//! - Efficient preference storage and retrieval with caching

use crate::proactive::{
    context_aware_scorer::{
        AudiencePriorityAdjustments, ContextAwareScoringConfig, DomainPriorityWeights,
        UrgencyPriorityScaling,
    },
    notification_system::{
        Notification, NotificationChannel, NotificationSystem, NotificationType,
    },
    prioritization::{
        DevelopmentContext, PrioritizationConfig, PriorityScoreBreakdown, PriorityScorer,
    },
    DetectedGap, GapType,
};
use chrono::{DateTime, Datelike, NaiveTime, Utc, Weekday};
use fortitude_types::classification_result::{AudienceLevel, TechnicalDomain};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

/// Type alias for cached preference profiles with timestamps
type ProfileCache = Arc<RwLock<HashMap<String, (UserPreferenceProfile, DateTime<Utc>)>>>;

/// Errors that can occur during user preference operations
#[derive(Error, Debug)]
pub enum UserPreferenceError {
    #[error("Preference validation failed: {field} - {error}")]
    PreferenceValidation { field: String, error: String },

    #[error("Invalid weight value: {weight} for {field}, must be between {min} and {max}")]
    InvalidWeight {
        field: String,
        weight: f64,
        min: f64,
        max: f64,
    },

    #[error("Profile not found: {profile_name}")]
    ProfileNotFound { profile_name: String },

    #[error("Preference storage error: {0}")]
    Storage(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid workflow mode: {mode}")]
    InvalidWorkflowMode { mode: String },

    #[error("Invalid expertise level: {level}")]
    InvalidExpertiseLevel { level: String },

    #[error("Filter compilation error: {pattern} - {error}")]
    FilterCompilation { pattern: String, error: String },

    #[error("Performance requirement violation: operation took {duration:?}, limit is {limit:?}")]
    PerformanceViolation { duration: Duration, limit: Duration },

    #[error("Configuration conflict: {0}")]
    ConfigurationConflict(String),

    #[error("Invalid cron expression: {expression} - {error}")]
    InvalidCronExpression { expression: String, error: String },

    #[error("Invalid time range: {field} - {error}")]
    InvalidTimeRange { field: String, error: String },

    #[error("Notification system error: {0}")]
    NotificationSystemError(String),
}

/// User workflow modes that adapt priority scoring behavior
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowMode {
    /// Development mode - focus on implementation gaps and code quality
    Development,
    /// Review mode - focus on documentation and API gaps
    Review,
    /// Documentation mode - focus on missing documentation and examples
    Documentation,
    /// Learning mode - focus on undocumented technologies and educational content
    Learning,
    /// Maintenance mode - focus on technical debt and configuration gaps
    Maintenance,
    /// Custom mode with user-defined behavior
    Custom(String),
}

impl WorkflowMode {
    /// Get priority weight adjustments for this workflow mode
    pub fn get_gap_type_weights(&self) -> HashMap<GapType, f64> {
        let mut weights = HashMap::new();

        match self {
            WorkflowMode::Development => {
                weights.insert(GapType::TodoComment, 1.3);
                weights.insert(GapType::UndocumentedTechnology, 1.2);
                weights.insert(GapType::ConfigurationGap, 1.1);
                weights.insert(GapType::MissingDocumentation, 0.8);
                weights.insert(GapType::ApiDocumentationGap, 0.9);
            }
            WorkflowMode::Review => {
                weights.insert(GapType::ApiDocumentationGap, 1.4);
                weights.insert(GapType::MissingDocumentation, 1.3);
                weights.insert(GapType::TodoComment, 1.1);
                weights.insert(GapType::UndocumentedTechnology, 1.0);
                weights.insert(GapType::ConfigurationGap, 0.8);
            }
            WorkflowMode::Documentation => {
                weights.insert(GapType::MissingDocumentation, 1.5);
                weights.insert(GapType::ApiDocumentationGap, 1.4);
                weights.insert(GapType::UndocumentedTechnology, 1.2);
                weights.insert(GapType::TodoComment, 0.7);
                weights.insert(GapType::ConfigurationGap, 0.6);
            }
            WorkflowMode::Learning => {
                weights.insert(GapType::UndocumentedTechnology, 1.5);
                weights.insert(GapType::MissingDocumentation, 1.3);
                weights.insert(GapType::ApiDocumentationGap, 1.2);
                weights.insert(GapType::TodoComment, 0.8);
                weights.insert(GapType::ConfigurationGap, 0.7);
            }
            WorkflowMode::Maintenance => {
                weights.insert(GapType::ConfigurationGap, 1.4);
                weights.insert(GapType::TodoComment, 1.3);
                weights.insert(GapType::UndocumentedTechnology, 1.1);
                weights.insert(GapType::MissingDocumentation, 0.9);
                weights.insert(GapType::ApiDocumentationGap, 0.8);
            }
            WorkflowMode::Custom(_) => {
                // Return neutral weights for custom modes
                weights.insert(GapType::TodoComment, 1.0);
                weights.insert(GapType::MissingDocumentation, 1.0);
                weights.insert(GapType::UndocumentedTechnology, 1.0);
                weights.insert(GapType::ApiDocumentationGap, 1.0);
                weights.insert(GapType::ConfigurationGap, 1.0);
            }
        }

        weights
    }

    /// Get description of this workflow mode
    pub fn description(&self) -> &str {
        match self {
            WorkflowMode::Development => {
                "Focus on implementation tasks and code quality improvements"
            }
            WorkflowMode::Review => "Focus on code review and API documentation quality",
            WorkflowMode::Documentation => "Focus on comprehensive documentation coverage",
            WorkflowMode::Learning => "Focus on understanding new technologies and concepts",
            WorkflowMode::Maintenance => "Focus on technical debt and system maintenance",
            WorkflowMode::Custom(name) => name,
        }
    }
}

/// User expertise levels that influence priority adjustments
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpertiseLevel {
    /// Beginner level - boost basic documentation and examples
    Beginner,
    /// Intermediate level - balanced priority adjustments
    Intermediate,
    /// Expert level - focus on advanced topics and edge cases
    Expert,
}

impl ExpertiseLevel {
    /// Get audience level priority adjustments for this expertise level
    pub fn get_audience_adjustments(&self) -> HashMap<AudienceLevel, f64> {
        let mut adjustments = HashMap::new();

        match self {
            ExpertiseLevel::Beginner => {
                adjustments.insert(AudienceLevel::Beginner, 1.4);
                adjustments.insert(AudienceLevel::Intermediate, 1.1);
                adjustments.insert(AudienceLevel::Advanced, 0.7);
            }
            ExpertiseLevel::Intermediate => {
                adjustments.insert(AudienceLevel::Beginner, 1.1);
                adjustments.insert(AudienceLevel::Intermediate, 1.3);
                adjustments.insert(AudienceLevel::Advanced, 1.0);
            }
            ExpertiseLevel::Expert => {
                adjustments.insert(AudienceLevel::Beginner, 0.8);
                adjustments.insert(AudienceLevel::Intermediate, 1.0);
                adjustments.insert(AudienceLevel::Advanced, 1.4);
            }
        }

        adjustments
    }

    /// Get complexity preference multiplier for this expertise level
    pub fn complexity_preference(&self) -> f64 {
        match self {
            ExpertiseLevel::Beginner => 0.7,     // Prefer simpler content
            ExpertiseLevel::Intermediate => 1.0, // Balanced complexity
            ExpertiseLevel::Expert => 1.3,       // Prefer complex content
        }
    }
}

/// Personal filters for customizing gap detection and prioritization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalFilters {
    /// File path patterns to include (regex patterns)
    pub include_file_patterns: Vec<String>,
    /// File path patterns to exclude (regex patterns)
    pub exclude_file_patterns: Vec<String>,
    /// Gap types to prioritize
    pub priority_gap_types: HashSet<GapType>,
    /// Gap types to deprioritize
    pub depriority_gap_types: HashSet<GapType>,
    /// Minimum priority threshold (gaps below this are filtered out)
    pub min_priority_threshold: f64,
    /// Maximum number of gaps to process per file
    pub max_gaps_per_file: usize,
    /// Keywords that boost priority when found in gap content
    pub priority_keywords: Vec<String>,
    /// Keywords that reduce priority when found in gap content
    pub depriority_keywords: Vec<String>,
    /// Technical domains to prioritize
    pub priority_domains: HashSet<TechnicalDomain>,
    /// Technical domains to deprioritize
    pub depriority_domains: HashSet<TechnicalDomain>,
}

impl Default for PersonalFilters {
    fn default() -> Self {
        Self {
            include_file_patterns: vec![r"\.rs$".to_string(), r"\.md$".to_string()],
            exclude_file_patterns: vec![r"target/".to_string(), r"\.git/".to_string()],
            priority_gap_types: HashSet::new(),
            depriority_gap_types: HashSet::new(),
            min_priority_threshold: 3.0,
            max_gaps_per_file: 20,
            priority_keywords: vec![
                "urgent".to_string(),
                "important".to_string(),
                "critical".to_string(),
                "security".to_string(),
            ],
            depriority_keywords: vec![
                "maybe".to_string(),
                "eventually".to_string(),
                "someday".to_string(),
                "nice to have".to_string(),
            ],
            priority_domains: HashSet::new(),
            depriority_domains: HashSet::new(),
        }
    }
}

/// Notification frequency modes for controlling when notifications are delivered
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NotificationFrequency {
    /// Send notifications immediately when events occur
    #[default]
    Immediate,
    /// Batch notifications together for delivery
    Batched {
        /// Number of notifications to batch together
        batch_size: usize,
        /// Maximum time to wait before sending incomplete batch
        batch_timeout: Duration,
    },
    /// Send notifications on a schedule (cron expression)
    Scheduled {
        /// Cron expression for schedule (e.g., "0 9,17 * * MON-FRI")
        schedule: String,
    },
    /// Disable all notifications
    Disabled,
}

impl NotificationFrequency {
    /// Validate the frequency configuration
    pub fn validate(&self) -> Result<(), UserPreferenceError> {
        match self {
            NotificationFrequency::Batched {
                batch_size,
                batch_timeout,
            } => {
                if *batch_size == 0 {
                    return Err(UserPreferenceError::PreferenceValidation {
                        field: "batch_size".to_string(),
                        error: "Batch size must be greater than 0".to_string(),
                    });
                }
                if *batch_timeout == Duration::from_secs(0) {
                    return Err(UserPreferenceError::PreferenceValidation {
                        field: "batch_timeout".to_string(),
                        error: "Batch timeout must be greater than 0".to_string(),
                    });
                }
            }
            NotificationFrequency::Scheduled { schedule } => {
                // Basic cron validation - in a real implementation, use a cron parsing library
                if schedule.split_whitespace().count() != 5 {
                    return Err(UserPreferenceError::InvalidCronExpression {
                        expression: schedule.clone(),
                        error: "Cron expression must have 5 fields".to_string(),
                    });
                }
            }
            _ => {} // Immediate and Disabled are always valid
        }
        Ok(())
    }
}

/// Detail levels for notification content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NotificationDetailLevel {
    /// Minimal information - just the essential facts
    Brief,
    /// Standard information - balanced detail
    #[default]
    Standard,
    /// Detailed information - comprehensive but focused
    Detailed,
    /// Maximum information - everything available
    Comprehensive,
}

/// Time range specification for business hours and quiet periods
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time in 24-hour format
    pub start: NaiveTime,
    /// End time in 24-hour format
    pub end: NaiveTime,
}

impl TimeRange {
    /// Create a new time range
    pub fn new(start: NaiveTime, end: NaiveTime) -> Result<Self, UserPreferenceError> {
        if start >= end {
            return Err(UserPreferenceError::InvalidTimeRange {
                field: "time_range".to_string(),
                error: "Start time must be before end time".to_string(),
            });
        }
        Ok(Self { start, end })
    }

    /// Check if the given time falls within this range
    pub fn contains(&self, time: NaiveTime) -> bool {
        time >= self.start && time <= self.end
    }

    /// Check if this range spans midnight (end time is after start time next day)
    pub fn spans_midnight(&self) -> bool {
        self.end < self.start
    }
}

/// Quiet hours configuration for a notification channel
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuietHours {
    /// Start time for quiet period
    pub start: NaiveTime,
    /// End time for quiet period  
    pub end: NaiveTime,
    /// Timezone for the quiet hours (e.g., "UTC", "America/New_York")
    pub timezone: String,
}

impl QuietHours {
    /// Check if the current time (in the specified timezone) is within quiet hours
    pub fn is_quiet_time(&self, current_time: DateTime<Utc>) -> bool {
        // Currently operates in UTC timezone
        // Future implementation will support proper timezone conversion
        let time = current_time.time();

        if self.start <= self.end {
            // Same day range
            time >= self.start && time <= self.end
        } else {
            // Crosses midnight
            time >= self.start || time <= self.end
        }
    }
}

/// Business hours configuration for each day of the week
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessHours {
    pub monday: Option<TimeRange>,
    pub tuesday: Option<TimeRange>,
    pub wednesday: Option<TimeRange>,
    pub thursday: Option<TimeRange>,
    pub friday: Option<TimeRange>,
    pub saturday: Option<TimeRange>,
    pub sunday: Option<TimeRange>,
    /// Timezone for business hours
    pub timezone: String,
}

impl Default for BusinessHours {
    fn default() -> Self {
        let standard_hours = TimeRange::new(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .unwrap();

        Self {
            monday: Some(standard_hours.clone()),
            tuesday: Some(standard_hours.clone()),
            wednesday: Some(standard_hours.clone()),
            thursday: Some(standard_hours.clone()),
            friday: Some(standard_hours),
            saturday: None,
            sunday: None,
            timezone: "UTC".to_string(),
        }
    }
}

impl BusinessHours {
    /// Check if the current time is within business hours
    pub fn is_business_time(&self, current_time: DateTime<Utc>) -> bool {
        // Currently operates in UTC, future implementation will support timezone conversion
        let weekday = current_time.weekday();
        let time = current_time.time();

        let time_range = match weekday {
            Weekday::Mon => &self.monday,
            Weekday::Tue => &self.tuesday,
            Weekday::Wed => &self.wednesday,
            Weekday::Thu => &self.thursday,
            Weekday::Fri => &self.friday,
            Weekday::Sat => &self.saturday,
            Weekday::Sun => &self.sunday,
        };

        time_range
            .as_ref()
            .is_some_and(|range| range.contains(time))
    }
}

/// Settings for a specific notification type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTypeSettings {
    /// Whether this notification type is enabled
    pub enabled: bool,
    /// Specific channels for this notification type (overrides defaults)
    pub channels: Vec<NotificationChannel>,
    /// Detail level for this notification type
    pub detail_level: NotificationDetailLevel,
}

impl Default for NotificationTypeSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            channels: vec![],
            detail_level: NotificationDetailLevel::Standard,
        }
    }
}

/// Settings for a specific notification channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannelSettings {
    /// Whether this channel is enabled
    pub enabled: bool,
    /// Rate limit for this channel (notifications per minute)
    pub rate_limit: Option<u32>,
    /// Minimum detail level allowed on this channel
    pub min_detail_level: NotificationDetailLevel,
    /// Maximum detail level allowed on this channel
    pub max_detail_level: NotificationDetailLevel,
    /// Quiet hours for this specific channel
    pub quiet_hours: Option<QuietHours>,
}

impl Default for NotificationChannelSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            rate_limit: Some(60), // 60 notifications per minute
            min_detail_level: NotificationDetailLevel::Brief,
            max_detail_level: NotificationDetailLevel::Comprehensive,
            quiet_hours: None,
        }
    }
}

/// Contextual notification settings for different modes/environments
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextualNotificationSettings {
    /// Override frequency for this context
    pub frequency_override: Option<NotificationFrequency>,
    /// Override detail level for this context
    pub detail_level_override: Option<NotificationDetailLevel>,
    /// Type-specific overrides for this context
    pub type_overrides: HashMap<String, NotificationTypeSettings>, // Use String key for serialization
}

/// Priority override settings for critical notifications
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PriorityOverrideSettings {
    /// Always send this notification type regardless of other settings
    pub always_send: bool,
    /// Override quiet hours for this notification type
    pub override_quiet_hours: bool,
    /// Override frequency setting for this notification type
    pub override_frequency: Option<NotificationFrequency>,
    /// Override channels for this notification type
    pub override_channels: Option<Vec<NotificationChannel>>,
}

/// Comprehensive notification preferences for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    /// Master enable/disable for all notifications
    pub enable_notifications: bool,
    /// Default frequency for notifications
    pub frequency: NotificationFrequency,
    /// Default detail level for notifications
    pub default_detail_level: NotificationDetailLevel,
    /// Default channels for notifications
    pub default_channels: Vec<NotificationChannel>,
    /// Type-specific notification settings
    pub type_settings: HashMap<String, NotificationTypeSettings>, // Use String key for serialization
    /// Channel-specific settings
    pub channel_settings: HashMap<String, NotificationChannelSettings>,
    /// Business hours configuration
    pub business_hours: Option<BusinessHours>,
    /// Global quiet hours (applies to all channels unless overridden)
    pub global_quiet_hours: Option<QuietHours>,
    /// Whether to respect business hours for notifications
    pub respect_business_hours: bool,
    /// Whether to respect quiet hours for notifications
    pub respect_quiet_hours: bool,
    /// Context-specific notification overrides
    pub contextual_settings: HashMap<String, ContextualNotificationSettings>,
    /// Priority overrides for critical notifications
    pub priority_overrides: HashMap<String, PriorityOverrideSettings>, // Use String key for serialization
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            enable_notifications: true,
            frequency: NotificationFrequency::Immediate,
            default_detail_level: NotificationDetailLevel::Standard,
            default_channels: vec![NotificationChannel::CLI],
            type_settings: HashMap::new(),
            channel_settings: HashMap::new(),
            business_hours: None,
            global_quiet_hours: None,
            respect_business_hours: false,
            respect_quiet_hours: true,
            contextual_settings: HashMap::new(),
            priority_overrides: HashMap::new(),
        }
    }
}

impl NotificationPreferences {
    /// Validate notification preferences
    pub fn validate(&self) -> Result<(), UserPreferenceError> {
        // Validate frequency
        self.frequency.validate()?;

        // Validate contextual settings
        for (context, settings) in &self.contextual_settings {
            if let Some(ref freq) = settings.frequency_override {
                freq.validate()
                    .map_err(|e| UserPreferenceError::PreferenceValidation {
                        field: format!("contextual_settings.{context}.frequency_override"),
                        error: e.to_string(),
                    })?;
            }
        }

        // Validate priority overrides
        for (override_key, settings) in &self.priority_overrides {
            if let Some(ref freq) = settings.override_frequency {
                freq.validate()
                    .map_err(|e| UserPreferenceError::PreferenceValidation {
                        field: format!("priority_overrides.{override_key}.override_frequency"),
                        error: e.to_string(),
                    })?;
            }
        }

        // Validate business hours
        if let Some(ref business_hours) = self.business_hours {
            // Business hours validation
            // Currently validates timezone presence, future implementation will add comprehensive validation
            if business_hours.timezone.trim().is_empty() {
                return Err(UserPreferenceError::PreferenceValidation {
                    field: "business_hours.timezone".to_string(),
                    error: "Timezone cannot be empty".to_string(),
                });
            }
        }

        // Validate quiet hours
        if let Some(ref quiet_hours) = self.global_quiet_hours {
            if quiet_hours.timezone.trim().is_empty() {
                return Err(UserPreferenceError::PreferenceValidation {
                    field: "global_quiet_hours.timezone".to_string(),
                    error: "Timezone cannot be empty".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Get effective notification settings for a specific type and context
    pub fn get_effective_settings(
        &self,
        notification_type: &NotificationType,
        context: Option<&str>,
    ) -> EffectiveNotificationSettings {
        let type_key = notification_type_to_string(notification_type);

        // Start with defaults
        let mut settings = EffectiveNotificationSettings {
            enabled: self.enable_notifications,
            frequency: self.frequency.clone(),
            detail_level: self.default_detail_level.clone(),
            channels: self.default_channels.clone(),
            respect_quiet_hours: self.respect_quiet_hours,
            respect_business_hours: self.respect_business_hours,
        };

        // Apply type-specific settings
        if let Some(type_settings) = self.type_settings.get(&type_key) {
            settings.enabled = settings.enabled && type_settings.enabled;
            settings.detail_level = type_settings.detail_level.clone();
            if !type_settings.channels.is_empty() {
                settings.channels = type_settings.channels.clone();
            }
        }

        // Apply contextual overrides
        if let Some(context_name) = context {
            if let Some(context_settings) = self.contextual_settings.get(context_name) {
                if let Some(ref freq_override) = context_settings.frequency_override {
                    settings.frequency = freq_override.clone();
                }
                if let Some(ref detail_override) = context_settings.detail_level_override {
                    settings.detail_level = detail_override.clone();
                }
                if let Some(type_override) = context_settings.type_overrides.get(&type_key) {
                    settings.enabled = settings.enabled && type_override.enabled;
                    settings.detail_level = type_override.detail_level.clone();
                    if !type_override.channels.is_empty() {
                        settings.channels = type_override.channels.clone();
                    }
                }
            }
        }

        // Apply priority overrides
        if let Some(priority_override) = self.priority_overrides.get(&type_key) {
            if priority_override.always_send {
                settings.enabled = true;
            }
            if priority_override.override_quiet_hours {
                settings.respect_quiet_hours = false;
            }
            if let Some(ref freq_override) = priority_override.override_frequency {
                settings.frequency = freq_override.clone();
            }
            if let Some(ref channels_override) = priority_override.override_channels {
                settings.channels = channels_override.clone();
            }
        }

        settings
    }

    /// Check if notifications should be sent at the current time
    pub fn should_send_at_time(&self, current_time: DateTime<Utc>) -> bool {
        // Check quiet hours
        if self.respect_quiet_hours {
            if let Some(ref quiet_hours) = self.global_quiet_hours {
                if quiet_hours.is_quiet_time(current_time) {
                    return false;
                }
            }
        }

        // Check business hours
        if self.respect_business_hours {
            if let Some(ref business_hours) = self.business_hours {
                if !business_hours.is_business_time(current_time) {
                    return false;
                }
            }
        }

        true
    }
}

/// Effective notification settings after applying all overrides and context
#[derive(Debug, Clone)]
pub struct EffectiveNotificationSettings {
    pub enabled: bool,
    pub frequency: NotificationFrequency,
    pub detail_level: NotificationDetailLevel,
    pub channels: Vec<NotificationChannel>,
    pub respect_quiet_hours: bool,
    pub respect_business_hours: bool,
}

/// Helper function to convert NotificationType to string for HashMap keys
fn notification_type_to_string(notification_type: &NotificationType) -> String {
    match notification_type {
        NotificationType::Info => "info".to_string(),
        NotificationType::Warning => "warning".to_string(),
        NotificationType::Error => "error".to_string(),
        NotificationType::Success => "success".to_string(),
        NotificationType::Debug => "debug".to_string(),
        NotificationType::Progress { .. } => "progress".to_string(),
    }
}

/// Custom priority weights for different aspects of priority scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPriorityWeights {
    /// Weight for gap type urgency in final score (0.0-1.0)
    pub gap_type_weight: f64,
    /// Weight for recency in final score (0.0-1.0)
    pub recency_weight: f64,
    /// Weight for impact assessment in final score (0.0-1.0)
    pub impact_weight: f64,
    /// Weight for context factors in final score (0.0-1.0)
    pub context_weight: f64,
    /// Custom gap type priority multipliers
    pub gap_type_multipliers: HashMap<GapType, f64>,
    /// Domain-specific priority weights
    pub domain_weights: DomainPriorityWeights,
    /// Audience-aware priority adjustments
    pub audience_adjustments: AudiencePriorityAdjustments,
    /// Urgency-based priority scaling
    pub urgency_scaling: UrgencyPriorityScaling,
}

impl Default for CustomPriorityWeights {
    fn default() -> Self {
        Self {
            gap_type_weight: 0.4,
            recency_weight: 0.2,
            impact_weight: 0.3,
            context_weight: 0.1,
            gap_type_multipliers: HashMap::new(),
            domain_weights: DomainPriorityWeights::default(),
            audience_adjustments: AudiencePriorityAdjustments::default(),
            urgency_scaling: UrgencyPriorityScaling::default(),
        }
    }
}

impl CustomPriorityWeights {
    /// Validate that weights sum to 1.0
    pub fn validate(&self) -> Result<(), UserPreferenceError> {
        let weight_sum =
            self.gap_type_weight + self.recency_weight + self.impact_weight + self.context_weight;

        if (weight_sum - 1.0).abs() > 0.001 {
            return Err(UserPreferenceError::PreferenceValidation {
                field: "priority_weights".to_string(),
                error: format!("Weights must sum to 1.0, got {weight_sum:.3}"),
            });
        }

        // Validate individual weights
        let weights = [
            ("gap_type_weight", self.gap_type_weight),
            ("recency_weight", self.recency_weight),
            ("impact_weight", self.impact_weight),
            ("context_weight", self.context_weight),
        ];

        for (field, weight) in weights {
            if !(0.0..=1.0).contains(&weight) {
                return Err(UserPreferenceError::InvalidWeight {
                    field: field.to_string(),
                    weight,
                    min: 0.0,
                    max: 1.0,
                });
            }
        }

        Ok(())
    }
}

/// User preference profile containing all customization options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferenceProfile {
    /// Profile name for identification
    pub name: String,
    /// Profile description
    pub description: String,
    /// User's current workflow mode
    pub workflow_mode: WorkflowMode,
    /// User's expertise level
    pub expertise_level: ExpertiseLevel,
    /// Custom priority weights and multipliers
    pub priority_weights: CustomPriorityWeights,
    /// Personal filters for gap detection and prioritization
    pub personal_filters: PersonalFilters,
    /// Development context preferences
    pub development_context: DevelopmentContext,
    /// Context-aware scoring preferences
    pub context_aware_config: Option<ContextAwareScoringConfig>,
    /// Notification preferences for this profile
    pub notification_preferences: NotificationPreferences,
    /// Profile creation timestamp
    pub created_at: DateTime<Utc>,
    /// Profile last modified timestamp
    pub modified_at: DateTime<Utc>,
    /// Profile version for compatibility tracking
    pub version: String,
    /// Whether this profile is the active/default profile
    pub is_active: bool,
}

impl UserPreferenceProfile {
    /// Create a new user preference profile
    pub fn new(name: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            name,
            description,
            workflow_mode: WorkflowMode::Development,
            expertise_level: ExpertiseLevel::Intermediate,
            priority_weights: CustomPriorityWeights::default(),
            personal_filters: PersonalFilters::default(),
            development_context: DevelopmentContext::default(),
            context_aware_config: None,
            notification_preferences: NotificationPreferences::default(),
            created_at: now,
            modified_at: now,
            version: "1.0.0".to_string(),
            is_active: false,
        }
    }

    /// Update the profile's modified timestamp
    pub fn touch(&mut self) {
        self.modified_at = Utc::now();
    }

    /// Validate the profile configuration
    pub fn validate(&self) -> Result<(), UserPreferenceError> {
        // Validate priority weights
        self.priority_weights.validate()?;

        // Validate workflow mode and expertise level are compatible
        if let WorkflowMode::Custom(ref mode_name) = self.workflow_mode {
            if mode_name.trim().is_empty() {
                return Err(UserPreferenceError::PreferenceValidation {
                    field: "workflow_mode".to_string(),
                    error: "Custom workflow mode name cannot be empty".to_string(),
                });
            }
        }

        // Validate personal filters
        if self.personal_filters.min_priority_threshold < 0.0
            || self.personal_filters.min_priority_threshold > 10.0
        {
            return Err(UserPreferenceError::InvalidWeight {
                field: "min_priority_threshold".to_string(),
                weight: self.personal_filters.min_priority_threshold,
                min: 0.0,
                max: 10.0,
            });
        }

        if self.personal_filters.max_gaps_per_file == 0 {
            return Err(UserPreferenceError::PreferenceValidation {
                field: "max_gaps_per_file".to_string(),
                error: "Must be greater than 0".to_string(),
            });
        }

        // Validate context-aware config if present
        if let Some(ref context_config) = self.context_aware_config {
            context_config
                .validate()
                .map_err(|e| UserPreferenceError::PreferenceValidation {
                    field: "context_aware_config".to_string(),
                    error: e.to_string(),
                })?;
        }

        // Validate notification preferences
        self.notification_preferences.validate()?;

        Ok(())
    }

    /// Apply workflow mode adjustments to priority weights
    pub fn apply_workflow_adjustments(&mut self) {
        let workflow_weights = self.workflow_mode.get_gap_type_weights();

        for (gap_type, multiplier) in workflow_weights {
            self.priority_weights
                .gap_type_multipliers
                .insert(gap_type, multiplier);
        }

        self.touch();
    }

    /// Apply expertise level adjustments to audience preferences
    pub fn apply_expertise_adjustments(&mut self) {
        let expertise_adjustments = self.expertise_level.get_audience_adjustments();

        for (audience, adjustment) in expertise_adjustments {
            match audience {
                AudienceLevel::Beginner => {
                    self.priority_weights.audience_adjustments.beginner_boost = adjustment;
                }
                AudienceLevel::Intermediate => {
                    self.priority_weights
                        .audience_adjustments
                        .intermediate_adjustment = adjustment;
                }
                AudienceLevel::Advanced => {
                    self.priority_weights
                        .audience_adjustments
                        .advanced_adjustment = adjustment;
                }
            }
        }

        self.touch();
    }

    /// Create prioritization config from this profile
    pub fn to_prioritization_config(&self) -> PrioritizationConfig {
        PrioritizationConfig {
            gap_type_weight: self.priority_weights.gap_type_weight,
            recency_weight: self.priority_weights.recency_weight,
            impact_weight: self.priority_weights.impact_weight,
            context_weight: self.priority_weights.context_weight,
            enable_context_aware_scoring: self.context_aware_config.is_some(),
            context_aware_config: self.context_aware_config.clone(),
            ..Default::default()
        }
    }

    /// Create development context from this profile
    pub fn to_development_context(&self) -> DevelopmentContext {
        let mut context = self.development_context.clone();

        // Apply gap type multipliers as custom boosts
        for (gap_type, multiplier) in &self.priority_weights.gap_type_multipliers {
            context.custom_boosts.insert(gap_type.clone(), *multiplier);
        }

        context
    }
}

/// User preference manager for handling multiple profiles and persistence
pub struct UserPreferenceManager {
    /// Storage path for preference profiles
    storage_path: PathBuf,
    /// Loaded preference profiles
    profiles: Arc<RwLock<HashMap<String, UserPreferenceProfile>>>,
    /// Currently active profile name
    active_profile: Arc<RwLock<Option<String>>>,
    /// Profile load cache for performance
    profile_cache: ProfileCache,
    /// Cache TTL in seconds
    cache_ttl_secs: u64,
}

impl UserPreferenceManager {
    /// Create a new user preference manager
    #[instrument(level = "debug")]
    pub async fn new(storage_path: PathBuf) -> Result<Self, UserPreferenceError> {
        let manager = Self {
            storage_path,
            profiles: Arc::new(RwLock::new(HashMap::new())),
            active_profile: Arc::new(RwLock::new(None)),
            profile_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl_secs: 300, // 5 minutes
        };

        // Create storage directory if it doesn't exist
        if let Some(parent) = manager.storage_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Load existing profiles
        manager.load_all_profiles().await?;

        Ok(manager)
    }

    /// Create a new preference profile
    #[instrument(level = "debug", skip(self))]
    pub async fn create_profile(
        &self,
        name: String,
        description: String,
    ) -> Result<UserPreferenceProfile, UserPreferenceError> {
        let start_time = std::time::Instant::now();

        let profile = UserPreferenceProfile::new(name.clone(), description);
        profile.validate()?;

        // Check for existing profile with same name
        {
            let profiles = self.profiles.read().await;
            if profiles.contains_key(&name) {
                return Err(UserPreferenceError::ConfigurationConflict(format!(
                    "Profile '{name}' already exists"
                )));
            }
        }

        // Save profile to storage
        self.save_profile(&profile).await?;

        // Add to memory
        {
            let mut profiles = self.profiles.write().await;
            profiles.insert(name.clone(), profile.clone());
        }

        let duration = start_time.elapsed();
        if duration > Duration::from_millis(10) {
            warn!("Profile creation took {:?}, limit is 10ms", duration);
        }

        info!("Created user preference profile: {}", name);
        Ok(profile)
    }

    /// Update an existing preference profile
    #[instrument(level = "debug", skip(self, profile))]
    pub async fn update_profile(
        &self,
        mut profile: UserPreferenceProfile,
    ) -> Result<(), UserPreferenceError> {
        let start_time = std::time::Instant::now();

        profile.touch();
        profile.validate()?;

        // Save to storage
        self.save_profile(&profile).await?;

        // Update in memory
        {
            let mut profiles = self.profiles.write().await;
            profiles.insert(profile.name.clone(), profile.clone());
        }

        // Clear cache entry
        {
            let mut cache = self.profile_cache.write().await;
            cache.remove(&profile.name);
        }

        let duration = start_time.elapsed();
        if duration > Duration::from_millis(10) {
            warn!("Profile update took {:?}, limit is 10ms", duration);
        }

        info!("Updated user preference profile: {}", profile.name);
        Ok(())
    }

    /// Load a preference profile by name
    #[instrument(level = "debug", skip(self))]
    pub async fn load_profile(
        &self,
        name: &str,
    ) -> Result<UserPreferenceProfile, UserPreferenceError> {
        let start_time = std::time::Instant::now();

        // Check cache first
        {
            let cache = self.profile_cache.read().await;
            if let Some((profile, cached_at)) = cache.get(name) {
                let age = Utc::now().signed_duration_since(*cached_at);
                if age.num_seconds() < self.cache_ttl_secs as i64 {
                    let duration = start_time.elapsed();
                    debug!("Loaded profile '{}' from cache in {:?}", name, duration);
                    return Ok(profile.clone());
                }
            }
        }

        // Load from memory
        {
            let profiles = self.profiles.read().await;
            if let Some(profile) = profiles.get(name) {
                // Update cache
                {
                    let mut cache = self.profile_cache.write().await;
                    cache.insert(name.to_string(), (profile.clone(), Utc::now()));
                }

                let duration = start_time.elapsed();
                if duration > Duration::from_millis(10) {
                    warn!("Profile loading took {:?}, limit is 10ms", duration);
                }

                debug!("Loaded profile '{}' from memory in {:?}", name, duration);
                return Ok(profile.clone());
            }
        }

        Err(UserPreferenceError::ProfileNotFound {
            profile_name: name.to_string(),
        })
    }

    /// Set the active preference profile
    #[instrument(level = "debug", skip(self))]
    pub async fn set_active_profile(&self, name: &str) -> Result<(), UserPreferenceError> {
        // Verify profile exists
        self.load_profile(name).await?;

        // Update active profile
        {
            let mut active = self.active_profile.write().await;
            *active = Some(name.to_string());
        }

        // Mark profile as active and others as inactive
        {
            let mut profiles = self.profiles.write().await;
            for (profile_name, profile) in profiles.iter_mut() {
                profile.is_active = profile_name == name;
                if profile.is_active {
                    profile.touch();
                }
            }
        }

        info!("Set active preference profile: {}", name);
        Ok(())
    }

    /// Get the currently active preference profile
    #[instrument(level = "debug", skip(self))]
    pub async fn get_active_profile(
        &self,
    ) -> Result<Option<UserPreferenceProfile>, UserPreferenceError> {
        let active_name = {
            let active = self.active_profile.read().await;
            active.clone()
        };

        if let Some(name) = active_name {
            Ok(Some(self.load_profile(&name).await?))
        } else {
            Ok(None)
        }
    }

    /// List all available preference profiles
    #[instrument(level = "debug", skip(self))]
    pub async fn list_profiles(&self) -> Vec<String> {
        let profiles = self.profiles.read().await;
        profiles.keys().cloned().collect()
    }

    /// Delete a preference profile
    #[instrument(level = "debug", skip(self))]
    pub async fn delete_profile(&self, name: &str) -> Result<(), UserPreferenceError> {
        // Remove from memory
        let was_active = {
            let mut profiles = self.profiles.write().await;
            if let Some(profile) = profiles.remove(name) {
                profile.is_active
            } else {
                false
            }
        };

        // Clear from cache
        {
            let mut cache = self.profile_cache.write().await;
            cache.remove(name);
        }

        // Clear active profile if it was the deleted one
        if was_active {
            let mut active = self.active_profile.write().await;
            *active = None;
        }

        // Remove from storage
        let profile_path = self.get_profile_path(name);
        if profile_path.exists() {
            tokio::fs::remove_file(profile_path).await?;
        }

        info!("Deleted user preference profile: {}", name);
        Ok(())
    }

    /// Save a profile to persistent storage
    async fn save_profile(
        &self,
        profile: &UserPreferenceProfile,
    ) -> Result<(), UserPreferenceError> {
        // Ensure storage directory exists
        tokio::fs::create_dir_all(&self.storage_path).await?;

        let profile_path = self.get_profile_path(&profile.name);
        let contents = serde_json::to_string_pretty(profile)?;
        tokio::fs::write(profile_path, contents).await?;
        Ok(())
    }

    /// Load all profiles from storage
    async fn load_all_profiles(&self) -> Result<(), UserPreferenceError> {
        if !self.storage_path.exists() {
            return Ok(());
        }

        let mut dir = tokio::fs::read_dir(&self.storage_path).await?;
        let mut profiles = self.profiles.write().await;

        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match tokio::fs::read_to_string(&path).await {
                    Ok(contents) => {
                        match serde_json::from_str::<UserPreferenceProfile>(&contents) {
                            Ok(profile) => {
                                if profile.validate().is_ok() {
                                    profiles.insert(profile.name.clone(), profile);
                                } else {
                                    warn!("Skipping invalid profile: {:?}", path);
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse profile {:?}: {}", path, e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read profile {:?}: {}", path, e);
                    }
                }
            }
        }

        info!("Loaded {} preference profiles", profiles.len());
        Ok(())
    }

    /// Get the file path for a profile
    fn get_profile_path(&self, name: &str) -> PathBuf {
        self.storage_path.join(format!("{name}.json"))
    }

    /// Clear profile cache
    pub async fn clear_cache(&self) {
        let mut cache = self.profile_cache.write().await;
        cache.clear();
        info!("Cleared preference profile cache");
    }

    /// Create preset preference profiles
    pub async fn create_preset_profiles(&self) -> Result<(), UserPreferenceError> {
        // Create development preset
        let mut dev_profile = UserPreferenceProfile::new(
            "development".to_string(),
            "Development-focused workflow with emphasis on implementation tasks".to_string(),
        );
        dev_profile.workflow_mode = WorkflowMode::Development;
        dev_profile.expertise_level = ExpertiseLevel::Intermediate;
        dev_profile.apply_workflow_adjustments();
        dev_profile.apply_expertise_adjustments();

        // Save directly to avoid duplicate check
        self.save_profile(&dev_profile).await?;
        {
            let mut profiles = self.profiles.write().await;
            profiles.insert(dev_profile.name.clone(), dev_profile);
        }

        // Create documentation preset
        let mut doc_profile = UserPreferenceProfile::new(
            "documentation".to_string(),
            "Documentation-focused workflow with emphasis on missing docs".to_string(),
        );
        doc_profile.workflow_mode = WorkflowMode::Documentation;
        doc_profile.expertise_level = ExpertiseLevel::Intermediate;
        doc_profile.apply_workflow_adjustments();
        doc_profile.apply_expertise_adjustments();

        self.save_profile(&doc_profile).await?;
        {
            let mut profiles = self.profiles.write().await;
            profiles.insert(doc_profile.name.clone(), doc_profile);
        }

        // Create learning preset
        let mut learning_profile = UserPreferenceProfile::new(
            "learning".to_string(),
            "Learning-focused workflow with emphasis on understanding new technologies".to_string(),
        );
        learning_profile.workflow_mode = WorkflowMode::Learning;
        learning_profile.expertise_level = ExpertiseLevel::Beginner;
        learning_profile.apply_workflow_adjustments();
        learning_profile.apply_expertise_adjustments();

        self.save_profile(&learning_profile).await?;
        {
            let mut profiles = self.profiles.write().await;
            profiles.insert(learning_profile.name.clone(), learning_profile);
        }

        info!("Created preset preference profiles");
        Ok(())
    }
}

/// User-preference-aware priority scorer that combines base prioritization with user customizations
pub struct UserAwarePriorityScorer {
    /// Base priority scorer
    base_scorer: PriorityScorer,
    /// User preference manager
    preference_manager: Arc<UserPreferenceManager>,
    /// Current active profile (cached)
    active_profile: Arc<RwLock<Option<UserPreferenceProfile>>>,
}

impl UserAwarePriorityScorer {
    /// Create a new user-aware priority scorer
    #[instrument(level = "debug", skip(preference_manager))]
    pub async fn new(
        preference_manager: Arc<UserPreferenceManager>,
    ) -> Result<Self, UserPreferenceError> {
        // Get active profile or use defaults
        let active_profile = preference_manager.get_active_profile().await?;

        let (config, context) = if let Some(ref profile) = active_profile {
            (
                profile.to_prioritization_config(),
                profile.to_development_context(),
            )
        } else {
            (
                PrioritizationConfig::default(),
                DevelopmentContext::default(),
            )
        };

        let base_scorer = PriorityScorer::new(config, context)
            .await
            .map_err(|e| UserPreferenceError::ConfigurationConflict(e.to_string()))?;

        Ok(Self {
            base_scorer,
            preference_manager,
            active_profile: Arc::new(RwLock::new(active_profile)),
        })
    }

    /// Refresh the scorer with the current active profile
    #[instrument(level = "debug", skip(self))]
    pub async fn refresh_active_profile(&mut self) -> Result<(), UserPreferenceError> {
        let start_time = std::time::Instant::now();

        let active_profile = self.preference_manager.get_active_profile().await?;

        let (config, context) = if let Some(ref profile) = active_profile {
            (
                profile.to_prioritization_config(),
                profile.to_development_context(),
            )
        } else {
            (
                PrioritizationConfig::default(),
                DevelopmentContext::default(),
            )
        };

        // Create new base scorer with updated configuration
        self.base_scorer = PriorityScorer::new(config, context)
            .await
            .map_err(|e| UserPreferenceError::ConfigurationConflict(e.to_string()))?;

        // Update cached profile
        {
            let mut cached_profile = self.active_profile.write().await;
            *cached_profile = active_profile;
        }

        let duration = start_time.elapsed();
        if duration > Duration::from_millis(5) {
            warn!("Profile refresh took {:?}, target is <5ms", duration);
        }

        info!("Refreshed user-aware priority scorer with active profile");
        Ok(())
    }

    /// Score gap priority with user preference enhancements
    #[instrument(level = "debug", skip(self, gap))]
    pub async fn score_gap_priority_with_preferences(
        &self,
        gap: &DetectedGap,
    ) -> Result<UserAwarePriorityBreakdown, UserPreferenceError> {
        let start_time = std::time::Instant::now();

        // Get base priority score
        let base_breakdown = self
            .base_scorer
            .score_gap_priority(gap)
            .await
            .map_err(|e| UserPreferenceError::ConfigurationConflict(e.to_string()))?;

        // Apply user preference customizations
        let mut enhanced_breakdown = self
            .apply_user_preference_enhancements(base_breakdown, gap)
            .await?;

        let duration = start_time.elapsed();
        enhanced_breakdown.processing_time = duration;

        if duration > Duration::from_millis(5) {
            warn!(
                "User preference scoring took {:?}, target is <5ms",
                duration
            );
        }

        Ok(enhanced_breakdown)
    }

    /// Apply user preference enhancements to base priority score
    async fn apply_user_preference_enhancements(
        &self,
        base_breakdown: PriorityScoreBreakdown,
        gap: &DetectedGap,
    ) -> Result<UserAwarePriorityBreakdown, UserPreferenceError> {
        let active_profile = {
            let profile_lock = self.active_profile.read().await;
            profile_lock.clone()
        };

        let mut enhanced_score = base_breakdown.final_score;
        let mut preference_adjustments = HashMap::new();

        if let Some(ref profile) = active_profile {
            // Apply workflow mode adjustments
            let workflow_weights = profile.workflow_mode.get_gap_type_weights();
            if let Some(weight) = workflow_weights.get(&gap.gap_type) {
                enhanced_score *= weight;
                preference_adjustments.insert("workflow_mode".to_string(), *weight);
            }

            // Apply expertise level complexity preference
            let complexity_preference = profile.expertise_level.complexity_preference();
            enhanced_score *= complexity_preference;
            preference_adjustments
                .insert("expertise_complexity".to_string(), complexity_preference);

            // Apply personal filters - check if gap meets user thresholds
            if enhanced_score < profile.personal_filters.min_priority_threshold {
                enhanced_score = 0.0; // Filter out below threshold
                preference_adjustments.insert("threshold_filter".to_string(), 0.0);
            }

            // Apply priority/depriority keywords
            let content_lower = gap.context.to_lowercase() + &gap.description.to_lowercase();

            // Check for priority keywords
            for keyword in &profile.personal_filters.priority_keywords {
                if content_lower.contains(&keyword.to_lowercase()) {
                    enhanced_score *= 1.2;
                    preference_adjustments.insert(format!("priority_keyword_{keyword}"), 1.2);
                    break;
                }
            }

            // Check for depriority keywords
            for keyword in &profile.personal_filters.depriority_keywords {
                if content_lower.contains(&keyword.to_lowercase()) {
                    enhanced_score *= 0.8;
                    preference_adjustments.insert(format!("depriority_keyword_{keyword}"), 0.8);
                    break;
                }
            }

            // Apply gap type priority/depriority settings
            if profile
                .personal_filters
                .priority_gap_types
                .contains(&gap.gap_type)
            {
                enhanced_score *= 1.3;
                preference_adjustments.insert("priority_gap_type".to_string(), 1.3);
            } else if profile
                .personal_filters
                .depriority_gap_types
                .contains(&gap.gap_type)
            {
                enhanced_score *= 0.7;
                preference_adjustments.insert("depriority_gap_type".to_string(), 0.7);
            }
        }

        // Ensure score stays within valid range
        enhanced_score = enhanced_score.clamp(0.0, 10.0);

        Ok(UserAwarePriorityBreakdown {
            base_breakdown,
            enhanced_score,
            preference_adjustments,
            applied_profile: active_profile.map(|p| p.name),
            processing_time: Duration::from_millis(0), // Will be set by calling function
        })
    }

    /// Get the currently active profile name
    pub async fn get_active_profile_name(&self) -> Option<String> {
        let profile_lock = self.active_profile.read().await;
        profile_lock.as_ref().map(|p| p.name.clone())
    }

    /// Check if context-aware scoring is enabled in the current profile
    pub async fn is_context_aware_enabled(&self) -> bool {
        let profile_lock = self.active_profile.read().await;
        profile_lock
            .as_ref()
            .map(|p| p.context_aware_config.is_some())
            .unwrap_or(false)
    }
}

/// User-aware priority breakdown with preference-based enhancements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAwarePriorityBreakdown {
    /// Original priority breakdown from base scoring
    pub base_breakdown: PriorityScoreBreakdown,
    /// Enhanced score with user preference adjustments
    pub enhanced_score: f64,
    /// Applied preference adjustments with their values
    pub preference_adjustments: HashMap<String, f64>,
    /// Name of the applied preference profile
    pub applied_profile: Option<String>,
    /// Time taken for preference processing
    pub processing_time: Duration,
}

/// Preference-aware notification sender that filters and routes notifications based on user preferences
pub struct PreferenceAwareNotificationSender {
    notification_system: Arc<NotificationSystem>,
    preference_manager: Arc<UserPreferenceManager>,
    current_context: Arc<RwLock<Option<String>>>,
}

impl PreferenceAwareNotificationSender {
    /// Create a new preference-aware notification sender
    #[instrument(level = "debug", skip(notification_system, preference_manager))]
    pub async fn new(
        notification_system: Arc<NotificationSystem>,
        preference_manager: Arc<UserPreferenceManager>,
    ) -> Result<Self, UserPreferenceError> {
        Ok(Self {
            notification_system,
            preference_manager,
            current_context: Arc::new(RwLock::new(None)),
        })
    }

    /// Set the current context for contextual notification preferences
    pub async fn set_context(&self, context: Option<String>) {
        let mut current_context = self.current_context.write().await;
        *current_context = context;
    }

    /// Send a notification through the preference-aware system
    #[instrument(level = "debug", skip(self))]
    pub async fn send_notification(
        &self,
        notification_type: NotificationType,
        title: String,
        message: String,
    ) -> Result<(), UserPreferenceError> {
        // Get active profile
        let active_profile = self.preference_manager.get_active_profile().await?;

        if let Some(profile) = active_profile {
            // Get current context
            let context = {
                let context_lock = self.current_context.read().await;
                context_lock.clone()
            };

            // Get effective settings for this notification
            let effective_settings = profile
                .notification_preferences
                .get_effective_settings(&notification_type, context.as_deref());

            // Check if notification should be sent
            if !effective_settings.enabled {
                debug!(
                    "Notification filtered out by user preferences: {:?}",
                    notification_type
                );
                return Ok(());
            }

            // Check time-based restrictions
            let current_time = Utc::now();
            if !profile
                .notification_preferences
                .should_send_at_time(current_time)
            {
                debug!(
                    "Notification delayed due to time restrictions: {:?}",
                    notification_type
                );
                // Future implementation will queue notifications for later delivery
                return Ok(());
            }

            // Apply frequency controls
            match effective_settings.frequency {
                NotificationFrequency::Immediate => {
                    self.send_immediate_notification(
                        notification_type,
                        title,
                        message,
                        effective_settings,
                    )
                    .await?;
                }
                NotificationFrequency::Batched { .. } => {
                    self.add_to_batch(notification_type, title, message, effective_settings)
                        .await?;
                }
                NotificationFrequency::Scheduled { .. } => {
                    self.schedule_notification(
                        notification_type,
                        title,
                        message,
                        effective_settings,
                    )
                    .await?;
                }
                NotificationFrequency::Disabled => {
                    debug!("Notification frequency is disabled");
                    return Ok(());
                }
            }
        } else {
            // No active profile, use default behavior
            let notification = Notification::new(
                notification_type,
                title,
                message,
                vec![NotificationChannel::CLI],
            );

            self.notification_system
                .send(notification)
                .await
                .map_err(|e| UserPreferenceError::NotificationSystemError(e.to_string()))?;
        }

        Ok(())
    }

    /// Send notification immediately with preference formatting
    async fn send_immediate_notification(
        &self,
        notification_type: NotificationType,
        title: String,
        message: String,
        settings: EffectiveNotificationSettings,
    ) -> Result<(), UserPreferenceError> {
        // Format message according to detail level
        let formatted_message = self.format_message_for_detail_level(
            &message,
            &settings.detail_level,
            &notification_type,
        );

        let notification = Notification::new(
            notification_type,
            title,
            formatted_message,
            settings.channels,
        );

        self.notification_system
            .send(notification)
            .await
            .map_err(|e| UserPreferenceError::NotificationSystemError(e.to_string()))?;

        Ok(())
    }

    /// Add notification to batch queue (placeholder implementation)
    async fn add_to_batch(
        &self,
        _notification_type: NotificationType,
        _title: String,
        _message: String,
        _settings: EffectiveNotificationSettings,
    ) -> Result<(), UserPreferenceError> {
        // In a real implementation, this would add to a batch queue
        debug!("Adding notification to batch queue (not yet implemented)");
        Ok(())
    }

    /// Schedule notification for later delivery (placeholder implementation)
    async fn schedule_notification(
        &self,
        _notification_type: NotificationType,
        _title: String,
        _message: String,
        _settings: EffectiveNotificationSettings,
    ) -> Result<(), UserPreferenceError> {
        // In a real implementation, this would schedule the notification
        debug!("Scheduling notification for later delivery (not yet implemented)");
        Ok(())
    }

    /// Format message according to user's preferred detail level
    fn format_message_for_detail_level(
        &self,
        message: &str,
        detail_level: &NotificationDetailLevel,
        notification_type: &NotificationType,
    ) -> String {
        match detail_level {
            NotificationDetailLevel::Brief => {
                // Truncate to first line or 50 characters
                let brief = message.lines().next().unwrap_or(message);
                if brief.len() > 50 {
                    format!("{}...", &brief[..47])
                } else {
                    brief.to_string()
                }
            }
            NotificationDetailLevel::Standard => {
                // Use message as-is but limit to 200 characters
                if message.len() > 200 {
                    format!("{}...", &message[..197])
                } else {
                    message.to_string()
                }
            }
            NotificationDetailLevel::Detailed => {
                // Add type prefix and keep full message
                format!(
                    "[{}] {}",
                    notification_type_to_string(notification_type).to_uppercase(),
                    message
                )
            }
            NotificationDetailLevel::Comprehensive => {
                // Add type, timestamp, and full message
                let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
                format!(
                    "[{}] [{}] {}",
                    notification_type_to_string(notification_type).to_uppercase(),
                    timestamp,
                    message
                )
            }
        }
    }
}

/// Notification preference filter for checking whether notifications should be sent
pub struct NotificationPreferenceFilter {
    preferences: NotificationPreferences,
}

impl NotificationPreferenceFilter {
    /// Create a new notification preference filter
    pub fn new(preferences: &NotificationPreferences) -> Self {
        Self {
            preferences: preferences.clone(),
        }
    }

    /// Check if a notification should be sent based on preferences
    pub fn should_send_notification(&self, notification_type: &NotificationType) -> bool {
        if !self.preferences.enable_notifications {
            return false;
        }

        // Check if this notification type is enabled
        let type_key = notification_type_to_string(notification_type);
        if let Some(type_settings) = self.preferences.type_settings.get(&type_key) {
            return type_settings.enabled;
        }

        // Default to enabled if no specific setting
        true
    }

    /// Check if notifications should be sent at the current time
    pub fn should_send_at_current_time(&self) -> bool {
        self.preferences.should_send_at_time(Utc::now())
    }

    /// Get effective channels for a notification type
    pub fn get_effective_channels(
        &self,
        notification_type: &NotificationType,
    ) -> Vec<NotificationChannel> {
        let type_key = notification_type_to_string(notification_type);

        if let Some(type_settings) = self.preferences.type_settings.get(&type_key) {
            if !type_settings.channels.is_empty() {
                return type_settings.channels.clone();
            }
        }

        self.preferences.default_channels.clone()
    }

    /// Get effective detail level for a notification type
    pub fn get_effective_detail_level(
        &self,
        notification_type: &NotificationType,
    ) -> NotificationDetailLevel {
        let type_key = notification_type_to_string(notification_type);

        if let Some(type_settings) = self.preferences.type_settings.get(&type_key) {
            return type_settings.detail_level.clone();
        }

        self.preferences.default_detail_level.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proactive::DevelopmentPhase;
    use tempfile::TempDir;

    #[test]
    fn test_workflow_mode_gap_type_weights() {
        // FAILING TEST: Workflow modes should provide appropriate gap type weights
        let dev_weights = WorkflowMode::Development.get_gap_type_weights();
        assert!(dev_weights[&GapType::TodoComment] > 1.0);
        assert!(dev_weights[&GapType::MissingDocumentation] < 1.0);

        let doc_weights = WorkflowMode::Documentation.get_gap_type_weights();
        assert!(doc_weights[&GapType::MissingDocumentation] > 1.4);
        assert!(doc_weights[&GapType::TodoComment] < 1.0);

        let learning_weights = WorkflowMode::Learning.get_gap_type_weights();
        assert!(learning_weights[&GapType::UndocumentedTechnology] > 1.4);
    }

    #[test]
    fn test_expertise_level_audience_adjustments() {
        // FAILING TEST: Expertise levels should provide appropriate audience adjustments
        let beginner_adj = ExpertiseLevel::Beginner.get_audience_adjustments();
        assert!(beginner_adj[&AudienceLevel::Beginner] > 1.3);
        assert!(beginner_adj[&AudienceLevel::Advanced] < 1.0);

        let expert_adj = ExpertiseLevel::Expert.get_audience_adjustments();
        assert!(expert_adj[&AudienceLevel::Advanced] > 1.3);
        assert!(expert_adj[&AudienceLevel::Beginner] < 1.0);

        let intermediate_adj = ExpertiseLevel::Intermediate.get_audience_adjustments();
        assert!(intermediate_adj[&AudienceLevel::Intermediate] > 1.2);
    }

    #[test]
    fn test_custom_priority_weights_validation() {
        // FAILING TEST: Custom priority weights should validate correctly
        let mut weights = CustomPriorityWeights::default();
        assert!(weights.validate().is_ok());

        // Test invalid weight sum
        weights.gap_type_weight = 0.5;
        weights.recency_weight = 0.5;
        weights.impact_weight = 0.3; // Sum > 1.0
        weights.context_weight = 0.3;
        assert!(weights.validate().is_err());

        // Test individual weight out of range
        weights = CustomPriorityWeights::default();
        weights.gap_type_weight = 1.5; // > 1.0
        assert!(weights.validate().is_err());
    }

    #[test]
    fn test_user_preference_profile_creation() {
        // FAILING TEST: User preference profile should be created with valid defaults
        let profile = UserPreferenceProfile::new("test".to_string(), "Test profile".to_string());

        assert_eq!(profile.name, "test");
        assert_eq!(profile.description, "Test profile");
        assert_eq!(profile.workflow_mode, WorkflowMode::Development);
        assert_eq!(profile.expertise_level, ExpertiseLevel::Intermediate);
        assert!(!profile.is_active);
        assert_eq!(profile.version, "1.0.0");
        assert!(profile.validate().is_ok());
    }

    #[test]
    fn test_profile_workflow_adjustments() {
        // FAILING TEST: Profile should apply workflow adjustments correctly
        let mut profile =
            UserPreferenceProfile::new("test".to_string(), "Test profile".to_string());
        profile.workflow_mode = WorkflowMode::Documentation;

        let original_modified = profile.modified_at;
        std::thread::sleep(std::time::Duration::from_millis(1));

        profile.apply_workflow_adjustments();

        assert!(profile.modified_at > original_modified);
        assert!(profile
            .priority_weights
            .gap_type_multipliers
            .contains_key(&GapType::MissingDocumentation));
        assert!(
            profile.priority_weights.gap_type_multipliers[&GapType::MissingDocumentation] > 1.4
        );
    }

    #[test]
    fn test_profile_expertise_adjustments() {
        // FAILING TEST: Profile should apply expertise adjustments correctly
        let mut profile =
            UserPreferenceProfile::new("test".to_string(), "Test profile".to_string());
        profile.expertise_level = ExpertiseLevel::Expert;

        let original_modified = profile.modified_at;
        std::thread::sleep(std::time::Duration::from_millis(1));

        profile.apply_expertise_adjustments();

        assert!(profile.modified_at > original_modified);
        assert!(
            profile
                .priority_weights
                .audience_adjustments
                .advanced_adjustment
                > 1.3
        );
        assert!(profile.priority_weights.audience_adjustments.beginner_boost < 1.0);
    }

    #[test]
    fn test_profile_to_prioritization_config() {
        // FAILING TEST: Profile should convert to prioritization config correctly
        let mut profile =
            UserPreferenceProfile::new("test".to_string(), "Test profile".to_string());
        profile.priority_weights.gap_type_weight = 0.5;
        profile.priority_weights.recency_weight = 0.3;
        profile.priority_weights.impact_weight = 0.1;
        profile.priority_weights.context_weight = 0.1;

        let config = profile.to_prioritization_config();
        assert_eq!(config.gap_type_weight, 0.5);
        assert_eq!(config.recency_weight, 0.3);
        assert_eq!(config.impact_weight, 0.1);
        assert_eq!(config.context_weight, 0.1);
        assert!(!config.enable_context_aware_scoring);
    }

    #[test]
    fn test_profile_to_development_context() {
        // FAILING TEST: Profile should convert to development context correctly
        let mut profile =
            UserPreferenceProfile::new("test".to_string(), "Test profile".to_string());
        profile.development_context.phase = DevelopmentPhase::Production;
        profile.development_context.is_public_api = true;
        profile
            .priority_weights
            .gap_type_multipliers
            .insert(GapType::TodoComment, 1.5);

        let context = profile.to_development_context();
        assert_eq!(context.phase, DevelopmentPhase::Production);
        assert!(context.is_public_api);
        assert_eq!(context.custom_boosts[&GapType::TodoComment], 1.5);
    }

    #[tokio::test]
    async fn test_preference_manager_creation() {
        // FAILING TEST: Preference manager should be created successfully
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("preferences");

        let manager = UserPreferenceManager::new(storage_path).await;
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert!(manager.list_profiles().await.is_empty());
    }

    #[tokio::test]
    async fn test_create_and_load_profile() {
        // FAILING TEST: Profile creation and loading should work correctly
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("preferences");

        let manager = UserPreferenceManager::new(storage_path).await.unwrap();

        // Create profile
        let profile = manager
            .create_profile("test".to_string(), "Test profile".to_string())
            .await;
        assert!(profile.is_ok());

        // Load profile
        let loaded = manager.load_profile("test").await;
        assert!(loaded.is_ok());

        let loaded_profile = loaded.unwrap();
        assert_eq!(loaded_profile.name, "test");
        assert_eq!(loaded_profile.description, "Test profile");
    }

    #[tokio::test]
    async fn test_profile_performance_requirements() {
        // FAILING TEST: Profile operations should meet performance requirements
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("preferences");

        let manager = UserPreferenceManager::new(storage_path).await.unwrap();

        // Test profile creation performance
        let start = std::time::Instant::now();
        let _profile = manager
            .create_profile(
                "perf_test".to_string(),
                "Performance test profile".to_string(),
            )
            .await
            .unwrap();
        let creation_time = start.elapsed();
        assert!(creation_time < Duration::from_millis(10));

        // Test profile loading performance
        let start = std::time::Instant::now();
        let _loaded = manager.load_profile("perf_test").await.unwrap();
        let load_time = start.elapsed();
        assert!(load_time < Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_active_profile_management() {
        // FAILING TEST: Active profile management should work correctly
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("preferences");

        let manager = UserPreferenceManager::new(storage_path).await.unwrap();

        // No active profile initially
        let active = manager.get_active_profile().await.unwrap();
        assert!(active.is_none());

        // Create and set active profile
        let _profile = manager
            .create_profile("active_test".to_string(), "Active test profile".to_string())
            .await
            .unwrap();

        let result = manager.set_active_profile("active_test").await;
        assert!(result.is_ok());

        // Check active profile
        let active = manager.get_active_profile().await.unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().name, "active_test");
    }

    #[tokio::test]
    async fn test_profile_update() {
        // FAILING TEST: Profile updates should work correctly
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("preferences");

        let manager = UserPreferenceManager::new(storage_path).await.unwrap();

        // Create profile
        let mut profile = manager
            .create_profile("update_test".to_string(), "Update test profile".to_string())
            .await
            .unwrap();

        // Modify profile
        profile.workflow_mode = WorkflowMode::Learning;
        profile.expertise_level = ExpertiseLevel::Expert;

        // Update profile
        let result = manager.update_profile(profile).await;
        assert!(result.is_ok());

        // Load and verify changes
        let loaded = manager.load_profile("update_test").await.unwrap();
        assert_eq!(loaded.workflow_mode, WorkflowMode::Learning);
        assert_eq!(loaded.expertise_level, ExpertiseLevel::Expert);
    }

    #[tokio::test]
    async fn test_profile_deletion() {
        // FAILING TEST: Profile deletion should work correctly
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("preferences");

        let manager = UserPreferenceManager::new(storage_path).await.unwrap();

        // Create profile
        let _profile = manager
            .create_profile("delete_test".to_string(), "Delete test profile".to_string())
            .await
            .unwrap();

        // Verify profile exists
        assert!(manager.load_profile("delete_test").await.is_ok());

        // Delete profile
        let result = manager.delete_profile("delete_test").await;
        assert!(result.is_ok());

        // Verify profile is gone
        let load_result = manager.load_profile("delete_test").await;
        assert!(load_result.is_err());
        assert!(matches!(
            load_result,
            Err(UserPreferenceError::ProfileNotFound { .. })
        ));
    }

    #[tokio::test]
    async fn test_profile_persistence() {
        // FAILING TEST: Profile persistence should work across manager instances
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("preferences");

        // Create manager and profile
        {
            let manager = UserPreferenceManager::new(storage_path.clone())
                .await
                .unwrap();
            let _profile = manager
                .create_profile(
                    "persist_test".to_string(),
                    "Persistence test profile".to_string(),
                )
                .await
                .unwrap();
        }

        // Create new manager instance and verify profile exists
        {
            let manager = UserPreferenceManager::new(storage_path).await.unwrap();
            let profiles = manager.list_profiles().await;
            assert!(profiles.contains(&"persist_test".to_string()));

            let loaded = manager.load_profile("persist_test").await.unwrap();
            assert_eq!(loaded.name, "persist_test");
        }
    }

    #[tokio::test]
    async fn test_profile_cache() {
        // FAILING TEST: Profile caching should improve performance
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("preferences");

        let manager = UserPreferenceManager::new(storage_path).await.unwrap();

        // Create profile
        let _profile = manager
            .create_profile("cache_test".to_string(), "Cache test profile".to_string())
            .await
            .unwrap();

        // First load (should cache)
        let start = std::time::Instant::now();
        let _loaded1 = manager.load_profile("cache_test").await.unwrap();
        let first_load_time = start.elapsed();

        // Second load (should use cache)
        let start = std::time::Instant::now();
        let _loaded2 = manager.load_profile("cache_test").await.unwrap();
        let second_load_time = start.elapsed();

        // Cache should be faster (though this might be flaky in tests)
        assert!(second_load_time <= first_load_time);

        // Clear cache and verify
        manager.clear_cache().await;

        // Load after cache clear should work
        let _loaded3 = manager.load_profile("cache_test").await.unwrap();
    }

    #[tokio::test]
    async fn test_preset_profiles() {
        // FAILING TEST: Preset profiles should be created correctly
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("preferences");

        let manager = UserPreferenceManager::new(storage_path).await.unwrap();

        // Create preset profiles
        let result = manager.create_preset_profiles().await;
        assert!(result.is_ok());

        // Verify presets exist
        let profiles = manager.list_profiles().await;
        assert!(profiles.contains(&"development".to_string()));
        assert!(profiles.contains(&"documentation".to_string()));
        assert!(profiles.contains(&"learning".to_string()));

        // Verify preset configurations
        let dev_profile = manager.load_profile("development").await.unwrap();
        assert_eq!(dev_profile.workflow_mode, WorkflowMode::Development);
        assert_eq!(dev_profile.expertise_level, ExpertiseLevel::Intermediate);

        let doc_profile = manager.load_profile("documentation").await.unwrap();
        assert_eq!(doc_profile.workflow_mode, WorkflowMode::Documentation);

        let learning_profile = manager.load_profile("learning").await.unwrap();
        assert_eq!(learning_profile.workflow_mode, WorkflowMode::Learning);
        assert_eq!(learning_profile.expertise_level, ExpertiseLevel::Beginner);
    }

    #[tokio::test]
    async fn test_user_aware_priority_scorer_creation() {
        // FAILING TEST: User-aware priority scorer should be created successfully
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("preferences");

        let manager = Arc::new(UserPreferenceManager::new(storage_path).await.unwrap());
        let result = UserAwarePriorityScorer::new(manager).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_user_aware_priority_scoring_with_profile() {
        // FAILING TEST: User-aware scoring should apply profile preferences
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("preferences");

        let manager = Arc::new(UserPreferenceManager::new(storage_path).await.unwrap());

        // Create and set development profile
        manager.create_preset_profiles().await.unwrap();
        manager.set_active_profile("development").await.unwrap();

        let scorer = UserAwarePriorityScorer::new(manager.clone()).await.unwrap();

        // Create a test gap
        let gap = DetectedGap {
            gap_type: GapType::TodoComment,
            file_path: std::path::PathBuf::from("src/main.rs"),
            line_number: 42,
            column_number: Some(10),
            context: "// TODO: Implement important error handling".to_string(),
            description: "Implement proper error handling".to_string(),
            confidence: 0.9,
            priority: 7,
            metadata: std::collections::HashMap::new(),
        };

        let result = scorer.score_gap_priority_with_preferences(&gap).await;
        assert!(result.is_ok());

        let breakdown = result.unwrap();
        assert!(breakdown.enhanced_score > 0.0);
        assert!(!breakdown.preference_adjustments.is_empty());
        assert_eq!(breakdown.applied_profile, Some("development".to_string()));
    }

    #[tokio::test]
    async fn test_user_aware_priority_scoring_performance() {
        // FAILING TEST: User-aware scoring should meet performance requirements
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("preferences");

        let manager = Arc::new(UserPreferenceManager::new(storage_path).await.unwrap());
        let scorer = UserAwarePriorityScorer::new(manager).await.unwrap();

        let gap = DetectedGap {
            gap_type: GapType::MissingDocumentation,
            file_path: std::path::PathBuf::from("src/lib.rs"),
            line_number: 1,
            column_number: None,
            context: "pub struct ImportantStruct".to_string(),
            description: "Missing documentation for important struct".to_string(),
            confidence: 0.8,
            priority: 6,
            metadata: std::collections::HashMap::new(),
        };

        let start = std::time::Instant::now();
        let result = scorer.score_gap_priority_with_preferences(&gap).await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration < Duration::from_millis(5)); // Should be <5ms
    }

    #[tokio::test]
    async fn test_user_aware_priority_keyword_filtering() {
        // FAILING TEST: Keyword filtering should affect priority scores
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("preferences");

        let manager = Arc::new(UserPreferenceManager::new(storage_path).await.unwrap());

        // Create custom profile with priority keywords
        let mut profile = UserPreferenceProfile::new(
            "keyword_test".to_string(),
            "Test profile for keyword filtering".to_string(),
        );
        profile.personal_filters.priority_keywords =
            vec!["critical".to_string(), "urgent".to_string()];
        profile.personal_filters.depriority_keywords =
            vec!["maybe".to_string(), "someday".to_string()];

        manager.update_profile(profile).await.unwrap();
        manager.set_active_profile("keyword_test").await.unwrap();

        let scorer = UserAwarePriorityScorer::new(manager).await.unwrap();

        // Test gap with priority keyword
        let priority_gap = DetectedGap {
            gap_type: GapType::TodoComment,
            file_path: std::path::PathBuf::from("src/main.rs"),
            line_number: 42,
            column_number: None,
            context: "// TODO: Critical security fix needed".to_string(),
            description: "Fix critical security vulnerability".to_string(),
            confidence: 0.9,
            priority: 7,
            metadata: std::collections::HashMap::new(),
        };

        // Test gap with depriority keyword
        let depriority_gap = DetectedGap {
            gap_type: GapType::TodoComment,
            file_path: std::path::PathBuf::from("src/main.rs"),
            line_number: 43,
            column_number: None,
            context: "// TODO: Maybe improve this someday".to_string(),
            description: "Potential future improvement".to_string(),
            confidence: 0.9,
            priority: 7,
            metadata: std::collections::HashMap::new(),
        };

        let priority_result = scorer
            .score_gap_priority_with_preferences(&priority_gap)
            .await
            .unwrap();
        let depriority_result = scorer
            .score_gap_priority_with_preferences(&depriority_gap)
            .await
            .unwrap();

        // Priority keyword should boost score
        assert!(priority_result
            .preference_adjustments
            .contains_key("priority_keyword_critical"));

        // Depriority keyword should reduce score
        assert!(depriority_result
            .preference_adjustments
            .contains_key("depriority_keyword_maybe"));

        // Priority gap should have higher enhanced score than depriority gap
        assert!(priority_result.enhanced_score > depriority_result.enhanced_score);
    }

    #[tokio::test]
    async fn test_user_aware_scoring_threshold_filtering() {
        // FAILING TEST: Threshold filtering should filter out low-priority gaps
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("preferences");

        let manager = Arc::new(UserPreferenceManager::new(storage_path).await.unwrap());

        // Create profile with high threshold
        let mut profile = UserPreferenceProfile::new(
            "threshold_test".to_string(),
            "Test profile for threshold filtering".to_string(),
        );
        profile.personal_filters.min_priority_threshold = 8.0; // High threshold

        manager.update_profile(profile).await.unwrap();
        manager.set_active_profile("threshold_test").await.unwrap();

        let scorer = UserAwarePriorityScorer::new(manager).await.unwrap();

        // Create low-priority gap that should be filtered out
        let low_priority_gap = DetectedGap {
            gap_type: GapType::ConfigurationGap,
            file_path: std::path::PathBuf::from("config.toml"),
            line_number: 1,
            column_number: None,
            context: "missing_key = value".to_string(),
            description: "Minor configuration adjustment".to_string(),
            confidence: 0.5, // Low confidence
            priority: 3,     // Low priority
            metadata: std::collections::HashMap::new(),
        };

        let result = scorer
            .score_gap_priority_with_preferences(&low_priority_gap)
            .await
            .unwrap();

        // Should be filtered out (score set to 0.0)
        assert_eq!(result.enhanced_score, 0.0);
        assert!(result
            .preference_adjustments
            .contains_key("threshold_filter"));
    }

    #[test]
    fn test_personal_filters_defaults() {
        // FAILING TEST: Personal filters should have reasonable defaults
        let filters = PersonalFilters::default();
        assert!(!filters.include_file_patterns.is_empty());
        assert!(!filters.exclude_file_patterns.is_empty());
        assert!(filters.min_priority_threshold > 0.0);
        assert!(filters.max_gaps_per_file > 0);
        assert!(!filters.priority_keywords.is_empty());
        assert!(!filters.depriority_keywords.is_empty());
    }

    #[test]
    fn test_workflow_mode_descriptions() {
        // FAILING TEST: Workflow modes should have descriptive text
        assert!(!WorkflowMode::Development.description().is_empty());
        assert!(!WorkflowMode::Review.description().is_empty());
        assert!(!WorkflowMode::Documentation.description().is_empty());
        assert!(!WorkflowMode::Learning.description().is_empty());
        assert!(!WorkflowMode::Maintenance.description().is_empty());

        let custom = WorkflowMode::Custom("Custom workflow".to_string());
        assert_eq!(custom.description(), "Custom workflow");
    }

    #[test]
    fn test_expertise_level_complexity_preference() {
        // FAILING TEST: Expertise levels should have complexity preferences
        assert!(ExpertiseLevel::Beginner.complexity_preference() < 1.0);
        assert_eq!(ExpertiseLevel::Intermediate.complexity_preference(), 1.0);
        assert!(ExpertiseLevel::Expert.complexity_preference() > 1.0);
    }
}
