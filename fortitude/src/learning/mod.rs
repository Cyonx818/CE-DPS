// ABOUTME: Learning system interfaces and core data models for real-time learning
//! # Learning System Module
//!
//! This module provides the core data structures and interfaces for the real-time
//! learning system. It enables the system to adapt based on user feedback and
//! usage patterns to improve research quality over time.
//!
//! ## Core Components
//!
//! - **Data Models**: Core structures for feedback, patterns, and learning insights
//! - **Storage Layer**: Persistence and retrieval of learning data with vector database integration
//! - **Adaptation Algorithms**: Interfaces for system improvement based on learning data
//! - **Pattern Recognition**: Analysis of usage patterns and trends
//! - **Configuration**: Settings and parameters for learning system behavior
//!
//! ## Architecture
//!
//! The learning system follows a modular architecture:
//! ```
//! Learning System
//! ├── Data Models (UserFeedback, PatternData, LearningData)
//! ├── Storage Layer (Vector DB + metadata storage)
//! ├── Pattern Recognition (Usage analysis)
//! ├── Adaptation Algorithms (System improvement)
//! └── Configuration (Learning parameters)
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub mod adaptation;
pub mod config;
pub mod monitoring;
pub mod optimization;
pub mod pattern_recognition;
pub mod storage;
pub mod template_integration;

// Re-export key types for easier access
pub use adaptation::AdaptationAlgorithmFactory;
pub use config::{
    AlertConfig, ConfigWatcher, EnhancedLearningConfig, HealthCheckConfig, LearningConfigManager,
    LearningMonitoringConfig, MonitoringThresholds,
};
pub use monitoring::{
    Alert, AlertHandler, AlertSeverity, DashboardData, HealthCheck, HealthReport, HealthStatus,
    LearningHealthChecker, LearningMetrics, LearningMetricsCollector, LearningPerformanceMonitor,
    MetricCollector, PerformanceSummary,
};
pub use optimization::{
    CacheOptimizationResult, ComprehensiveOptimizationResult, OptimizationConfig,
    OptimizationContext, PerformanceMetrics, PerformanceOptimizer, ProviderSelectionResult,
    QueryPerformanceResult,
};
pub use storage::{
    CleanupResult, EmbeddingCacheStats, EnhancedLearningStorageService, FeedbackTrend,
    LearningStorageService, SimilarityLearningResult, SimilarityUsagePattern,
    VectorLearningStorage,
};
pub use template_integration::{
    IntegrationConfig, TemplateOptimizationRecommendation, TemplateOptimizationService,
    TemplatePerformanceMetrics,
};

/// Result type for learning system operations
pub type LearningResult<T> = Result<T, LearningError>;

/// Errors that can occur in the learning system
#[derive(Debug, Error, Clone)]
pub enum LearningError {
    #[error("Storage operation failed: {0}")]
    StorageError(String),

    #[error("Invalid feedback data: {0}")]
    InvalidFeedback(String),

    #[error("Pattern analysis failed: {0}")]
    PatternAnalysisError(String),

    #[error("Adaptation algorithm error: {0}")]
    AdaptationError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Database connection error: {0}")]
    DatabaseError(String),

    #[error("Learning data not found: {0}")]
    NotFound(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

impl From<serde_json::Error> for LearningError {
    fn from(error: serde_json::Error) -> Self {
        LearningError::SerializationError(error.to_string())
    }
}

/// Configuration for the learning system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    /// Enable feedback-based learning
    pub enable_feedback_learning: bool,

    /// Enable pattern recognition and analysis
    pub enable_pattern_recognition: bool,

    /// Enable automated system optimization
    pub enable_optimization: bool,

    /// Minimum confidence threshold for adaptations
    pub adaptation_threshold: f64,

    /// Maximum age of learning data in days
    pub max_data_age_days: u32,

    /// Minimum number of feedback entries for analysis
    pub min_feedback_threshold: usize,

    /// Pattern frequency threshold for significance
    pub pattern_frequency_threshold: u32,

    /// Learning rate for adaptation algorithms
    pub learning_rate: f64,

    /// Storage configuration
    pub storage: LearningStorageConfig,

    /// Adaptation algorithm settings
    pub adaptation: AdaptationConfig,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            enable_feedback_learning: true,
            enable_pattern_recognition: true,
            enable_optimization: false, // Disabled by default for safety
            adaptation_threshold: 0.7,
            max_data_age_days: 90,
            min_feedback_threshold: 5,
            pattern_frequency_threshold: 3,
            learning_rate: 0.1,
            storage: LearningStorageConfig::default(),
            adaptation: AdaptationConfig::default(),
        }
    }
}

/// Storage configuration for learning data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStorageConfig {
    /// Vector database collection for learning data
    pub collection_name: String,

    /// Enable vector embeddings for learning data
    pub enable_embeddings: bool,

    /// Batch size for bulk operations
    pub batch_size: usize,

    /// Retention period for old data in days
    pub retention_days: u32,
}

impl Default for LearningStorageConfig {
    fn default() -> Self {
        Self {
            collection_name: "learning_data".to_string(),
            enable_embeddings: true,
            batch_size: 100,
            retention_days: 365,
        }
    }
}

/// Configuration for adaptation algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationConfig {
    /// Algorithms to enable
    pub enabled_algorithms: Vec<String>,

    /// Algorithm-specific settings
    pub algorithm_settings: HashMap<String, serde_json::Value>,

    /// Update frequency in hours
    pub update_frequency_hours: u32,

    /// Enable automatic application of adaptations
    pub auto_apply_adaptations: bool,
}

impl Default for AdaptationConfig {
    fn default() -> Self {
        Self {
            enabled_algorithms: vec![
                "feedback_analyzer".to_string(),
                "pattern_matcher".to_string(),
            ],
            algorithm_settings: HashMap::new(),
            update_frequency_hours: 24,
            auto_apply_adaptations: false,
        }
    }
}

/// User feedback data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    /// Unique feedback identifier
    pub id: String,

    /// User who provided the feedback
    pub user_id: String,

    /// Content being rated (research result, response, etc.)
    pub content_id: String,

    /// Type of feedback (quality_rating, relevance, satisfaction, etc.)
    pub feedback_type: String,

    /// Numerical score (0.0-1.0 scale)
    pub score: Option<f64>,

    /// Optional text feedback
    pub text_feedback: Option<String>,

    /// When the feedback was provided
    pub timestamp: DateTime<Utc>,

    /// Additional context and metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl UserFeedback {
    /// Create a new feedback entry
    pub fn new(
        user_id: String,
        content_id: String,
        feedback_type: String,
        score: Option<f64>,
        text_feedback: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            content_id,
            feedback_type,
            score,
            text_feedback,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Validate the feedback data
    pub fn is_valid(&self) -> bool {
        !self.user_id.is_empty()
            && !self.content_id.is_empty()
            && !self.feedback_type.is_empty()
            && self.score.is_none_or(|s| (0.0..=1.0).contains(&s))
    }

    /// Add metadata to the feedback
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Pattern data for usage analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternData {
    /// Unique pattern identifier
    pub id: String,

    /// Type of pattern (query_type, search_behavior, preference, etc.)
    pub pattern_type: String,

    /// How often this pattern occurs
    pub frequency: u32,

    /// Success rate for this pattern (0.0-1.0)
    pub success_rate: f64,

    /// Context information for the pattern
    pub context: HashMap<String, serde_json::Value>,

    /// When this pattern was first observed
    pub first_seen: DateTime<Utc>,

    /// When this pattern was last observed
    pub last_seen: DateTime<Utc>,
}

impl PatternData {
    /// Create a new pattern entry
    pub fn new(pattern_type: String, frequency: u32, success_rate: f64) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            pattern_type,
            frequency,
            success_rate,
            context: HashMap::new(),
            first_seen: now,
            last_seen: now,
        }
    }

    /// Update the pattern with new occurrence
    pub fn update_occurrence(&mut self, success: bool) {
        self.frequency += 1;
        self.last_seen = Utc::now();

        // Update success rate using running average
        let old_successes = (self.success_rate * (self.frequency - 1) as f64).round() as u32;
        let new_successes = if success {
            old_successes + 1
        } else {
            old_successes
        };
        self.success_rate = new_successes as f64 / self.frequency as f64;
    }

    /// Check if pattern is significant based on frequency threshold
    pub fn is_significant(&self, threshold: u32) -> bool {
        self.frequency >= threshold
    }
}

/// Learning insights and data derived from analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningData {
    /// Unique learning data identifier
    pub id: String,

    /// Type of learning (user_preference, system_optimization, pattern_insight)
    pub learning_type: String,

    /// Source data that generated this learning
    pub source_data_id: String,

    /// Key insights derived from analysis
    pub insights: Vec<String>,

    /// Confidence score for these insights (0.0-1.0)
    pub confidence_score: f64,

    /// When this learning was generated
    pub created_at: DateTime<Utc>,

    /// Optional expiration time for the learning
    pub expires_at: Option<DateTime<Utc>>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl LearningData {
    /// Create new learning data
    pub fn new(
        learning_type: String,
        source_data_id: String,
        insights: Vec<String>,
        confidence_score: f64,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            learning_type,
            source_data_id,
            insights,
            confidence_score,
            created_at: Utc::now(),
            expires_at: None,
            metadata: HashMap::new(),
        }
    }

    /// Check if the learning data is still valid (not expired)
    pub fn is_valid(&self) -> bool {
        self.expires_at.is_none_or(|exp| Utc::now() < exp)
    }

    /// Set expiration time
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
}

/// Usage pattern for behavioral analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePattern {
    /// Unique pattern identifier
    pub id: String,

    /// Type of usage pattern
    pub pattern_type: String,

    /// Pattern data/content
    pub data: String,

    /// Frequency of this pattern
    pub frequency: u32,

    /// Last time this pattern was used
    pub last_used: DateTime<Utc>,

    /// Context for the pattern
    pub context: HashMap<String, serde_json::Value>,
}

impl UsagePattern {
    /// Create a new usage pattern
    pub fn new(pattern_type: String, data: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            pattern_type,
            data,
            frequency: 1,
            last_used: Utc::now(),
            context: HashMap::new(),
        }
    }

    /// Increment usage of this pattern
    pub fn increment_usage(&mut self) {
        self.frequency += 1;
        self.last_used = Utc::now();
    }
}

/// Aggregated feedback data for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackData {
    /// Content ID this feedback relates to
    pub content_id: String,

    /// Average score across all feedback
    pub average_score: f64,

    /// Total number of feedback entries
    pub feedback_count: usize,

    /// Recent trend direction (-1.0 to 1.0, negative means declining)
    pub recent_trend: f64,
}

impl FeedbackData {
    /// Create feedback data from individual feedback entries
    pub fn from_feedback(content_id: String, feedback: &[UserFeedback]) -> Self {
        let scores: Vec<f64> = feedback.iter().filter_map(|f| f.score).collect();

        let average_score = if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        // Simple trend calculation based on recent vs. older feedback
        let recent_trend = if scores.len() >= 4 {
            let mid = scores.len() / 2;
            let recent_avg = scores[mid..].iter().sum::<f64>() / (scores.len() - mid) as f64;
            let older_avg = scores[..mid].iter().sum::<f64>() / mid as f64;
            recent_avg - older_avg
        } else {
            0.0
        };

        Self {
            content_id,
            average_score,
            feedback_count: feedback.len(),
            recent_trend,
        }
    }
}

/// Trait for adaptation algorithms that learn from data
#[async_trait::async_trait]
pub trait AdaptationAlgorithm: Send + Sync {
    /// Analyze feedback data and provide adaptation recommendations
    async fn analyze_feedback(&self, feedback: &FeedbackData) -> LearningResult<AdaptationResult>;

    /// Analyze usage patterns and provide insights
    async fn analyze_patterns(
        &self,
        patterns: &[UsagePattern],
    ) -> LearningResult<PatternAnalysisResult>;

    /// Get algorithm configuration
    fn get_config(&self) -> &AdaptationConfig;
}

/// Result of adaptation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationResult {
    /// Recommended adaptations
    pub recommendations: Vec<String>,

    /// Confidence score for recommendations
    pub confidence_score: f64,

    /// Priority level for applying adaptations
    pub priority: String,
}

/// Result of pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAnalysisResult {
    /// Insights derived from pattern analysis
    pub insights: Vec<String>,

    /// Confidence score for insights
    pub confidence_score: f64,

    /// Recommendations based on patterns
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_feedback_creation() {
        let feedback = UserFeedback::new(
            "user123".to_string(),
            "content456".to_string(),
            "quality_rating".to_string(),
            Some(0.85),
            Some("Great response".to_string()),
        );

        assert!(!feedback.id.is_empty());
        assert_eq!(feedback.user_id, "user123");
        assert_eq!(feedback.content_id, "content456");
        assert_eq!(feedback.score, Some(0.85));
        assert!(feedback.is_valid());
    }

    #[test]
    fn test_pattern_data_update() {
        let mut pattern = PatternData::new("query_type".to_string(), 1, 1.0);
        assert_eq!(pattern.frequency, 1);
        assert_eq!(pattern.success_rate, 1.0);

        pattern.update_occurrence(false);
        assert_eq!(pattern.frequency, 2);
        assert_eq!(pattern.success_rate, 0.5);

        pattern.update_occurrence(true);
        assert_eq!(pattern.frequency, 3);
        assert!((pattern.success_rate - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_learning_data_validity() {
        let learning = LearningData::new(
            "user_preference".to_string(),
            "source123".to_string(),
            vec!["User prefers detailed responses".to_string()],
            0.8,
        );

        assert!(learning.is_valid());

        let expired_learning = learning.with_expiration(Utc::now() - chrono::Duration::hours(1));
        assert!(!expired_learning.is_valid());
    }

    #[test]
    fn test_usage_pattern_increment() {
        let mut pattern = UsagePattern::new("search_style".to_string(), "detailed".to_string());
        assert_eq!(pattern.frequency, 1);

        pattern.increment_usage();
        assert_eq!(pattern.frequency, 2);
    }

    #[test]
    fn test_feedback_data_aggregation() {
        let feedback_entries = vec![
            UserFeedback::new(
                "user1".to_string(),
                "content1".to_string(),
                "rating".to_string(),
                Some(0.8),
                None,
            ),
            UserFeedback::new(
                "user2".to_string(),
                "content1".to_string(),
                "rating".to_string(),
                Some(0.9),
                None,
            ),
            UserFeedback::new(
                "user3".to_string(),
                "content1".to_string(),
                "rating".to_string(),
                Some(0.7),
                None,
            ),
        ];

        let feedback_data = FeedbackData::from_feedback("content1".to_string(), &feedback_entries);

        assert_eq!(feedback_data.content_id, "content1");
        assert_eq!(feedback_data.feedback_count, 3);
        assert!((feedback_data.average_score - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_learning_config_defaults() {
        let config = LearningConfig::default();

        assert!(config.enable_feedback_learning);
        assert!(config.enable_pattern_recognition);
        assert!(!config.enable_optimization); // Should be false by default
        assert_eq!(config.adaptation_threshold, 0.7);
        assert_eq!(config.max_data_age_days, 90);
    }

    #[test]
    fn test_feedback_validation() {
        let valid_feedback = UserFeedback::new(
            "user".to_string(),
            "content".to_string(),
            "rating".to_string(),
            Some(0.5),
            None,
        );
        assert!(valid_feedback.is_valid());

        let invalid_score_feedback = UserFeedback::new(
            "user".to_string(),
            "content".to_string(),
            "rating".to_string(),
            Some(1.5), // Invalid score > 1.0
            None,
        );
        assert!(!invalid_score_feedback.is_valid());

        let empty_user_feedback = UserFeedback::new(
            "".to_string(), // Empty user ID
            "content".to_string(),
            "rating".to_string(),
            Some(0.5),
            None,
        );
        assert!(!empty_user_feedback.is_valid());
    }
}
