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

// ABOUTME: Quality scoring framework for evaluating LLM research outputs
//! This module provides comprehensive quality assessment algorithms to evaluate
//! research outputs with multi-dimensional scoring across relevance, accuracy,
//! completeness, clarity, credibility, timeliness, and specificity dimensions.
//!
//! # Key Components
//! - `QualityScore`: Core data structure for multi-dimensional quality assessment
//! - `QualityScorer`: Trait for implementing scoring algorithms
//! - `QualityWeights`: Configuration for dimension weighting
//! - `QualityContext`: Context information for enhanced scoring accuracy
//!
//! # Performance Requirements
//! - Real-time evaluation: <100ms per assessment
//! - Accuracy target: >95% correlation with human evaluators
//! - Scalability: Handle 1000+ evaluations per minute
//! - Memory efficient: <10MB per scoring session
//!
//! # Example Usage
//!
//! ```rust
//! use fortitude::quality::{QualityScorer, QualityScore, QualityWeights};
//!
//! async fn evaluate_research_output(
//!     scorer: impl QualityScorer,
//!     query: &str,
//!     response: &str,
//! ) -> Result<QualityScore, QualityError> {
//!     let weights = QualityWeights::default();
//!     scorer.evaluate_quality(query, response, &weights).await
//! }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

pub mod config;
pub mod cross_validation;
pub mod feedback;
pub mod metrics;
pub mod optimization;
pub mod optimization_minimal;
pub mod provider_integration;
pub mod scoring;

pub use config::{
    ABTestingConfig, AlertThresholds, ComplianceConfig, ConfigError, ConfigResult, ConfigWatcher,
    EffectiveConfig, EnvironmentConfig, FeedbackSystemConfig, GlobalQualityConfig,
    MonitoringConfig, QualityConfigManager, QualityControlConfig, ResourceLimits,
};
pub use cross_validation::{
    BiasAnalysis, ConsensusMethod, ConsistencyAnalysis, CrossValidationConfig,
    CrossValidationEngine, ValidationMetrics, ValidationResult, ValidationStrategy,
};
pub use feedback::{
    ABTestConfig, ABTestResults, AccuracyImprovementMetrics, AlgorithmVariant,
    BatchProcessingResult, CorrectionValidation, FeedbackAnalytics, FeedbackAnalyticsReport,
    FeedbackCollectionConfig, FeedbackCollector, FeedbackContext, FeedbackError,
    FeedbackIntegrationSystem, FeedbackPatterns, FeedbackPrivacyConfig, FeedbackResult,
    FeedbackStorage, FeedbackType, LearningEngine, ProviderPreferenceLearning, ProviderTrend,
    QualityImprovementMetrics, QualityLearningConfig, UserFeedback, UserProviderPreferences,
    ValidationResult as FeedbackValidationResult,
};
pub use metrics::{
    AggregatedMetric, AnomalySeverity, CleanupStats, InMemoryMetricsStorage, MetricAggregations,
    MetricContext, MetricFilters, MetricType, MetricValue, MetricsAnalyzer, MetricsCollector,
    MetricsConfig, MetricsError, MetricsPerformanceStats, MetricsResult, MetricsStorage,
    PerformanceThresholds, ProviderPerformanceAnalysis, QualityAnomaly, QualityMetric,
    QualityTrends, RetentionConfig, StorageStats, TrendDirection,
};
pub use optimization::{
    AdaptationConfig, CostConstraints, FinalEvaluation, OptimizationConfig, OptimizationError,
    OptimizationResult, OptimizedQueryResult, ProviderPerformance, ProviderSelection,
    ProviderSelectionStrategy, QualityOptimizationEngine, QueryComplexity, SelectionCriteria,
    UrgencyLevel,
};
pub use optimization_minimal::{
    MinimalOptimizationEngine, OptimizationConfig as MinimalOptimizationConfig,
    OptimizationError as MinimalOptimizationError,
    OptimizedQueryResult as MinimalOptimizedQueryResult, QueryComplexity as MinimalQueryComplexity,
    SelectionCriteria as MinimalSelectionCriteria, UrgencyLevel as MinimalUrgencyLevel,
};
pub use provider_integration::{
    ProviderQualityMetrics, QualityAwareProvider, QualityAwareProviderManager,
    QualityIntegrationConfig, QualitySelectionStrategy,
};
pub use scoring::{ComprehensiveQualityScorer, ScorerConfig};

/// Comprehensive quality score for research outputs across multiple dimensions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityScore {
    /// How well output addresses the research query (0.0 - 1.0)
    pub relevance: f64,
    /// Factual correctness and information validity (0.0 - 1.0)
    pub accuracy: f64,
    /// Thoroughness and depth of information (0.0 - 1.0)
    pub completeness: f64,
    /// Readability, structure, and coherence (0.0 - 1.0)
    pub clarity: f64,
    /// Source reliability and evidence quality (0.0 - 1.0)
    pub credibility: f64,
    /// Information recency and currency (0.0 - 1.0)
    pub timeliness: f64,
    /// Level of detail and precision (0.0 - 1.0)
    pub specificity: f64,
    /// Weighted composite score (0.0 - 1.0)
    pub composite: f64,
    /// Confidence in assessment accuracy (0.0 - 1.0)
    pub confidence: f64,
}

impl QualityScore {
    /// Create a new quality score with all dimensions set to zero
    pub fn new() -> Self {
        Self {
            relevance: 0.0,
            accuracy: 0.0,
            completeness: 0.0,
            clarity: 0.0,
            credibility: 0.0,
            timeliness: 0.0,
            specificity: 0.0,
            composite: 0.0,
            confidence: 0.0,
        }
    }

    /// Calculate composite score using provided weights
    pub fn calculate_composite(&mut self, weights: &QualityWeights) {
        self.composite = self.relevance * weights.relevance
            + self.accuracy * weights.accuracy
            + self.completeness * weights.completeness
            + self.clarity * weights.clarity
            + self.credibility * weights.credibility
            + self.timeliness * weights.timeliness
            + self.specificity * weights.specificity;
    }

    /// Validate that all dimension scores are within valid range [0.0, 1.0]
    pub fn is_valid(&self) -> bool {
        let scores = [
            self.relevance,
            self.accuracy,
            self.completeness,
            self.clarity,
            self.credibility,
            self.timeliness,
            self.specificity,
            self.composite,
            self.confidence,
        ];

        scores.iter().all(|&score| (0.0..=1.0).contains(&score))
    }

    /// Get the dimension with the lowest score
    pub fn lowest_dimension(&self) -> (&'static str, f64) {
        let dimensions = [
            ("relevance", self.relevance),
            ("accuracy", self.accuracy),
            ("completeness", self.completeness),
            ("clarity", self.clarity),
            ("credibility", self.credibility),
            ("timeliness", self.timeliness),
            ("specificity", self.specificity),
        ];

        dimensions
            .into_iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
    }

    /// Get the dimension with the highest score
    pub fn highest_dimension(&self) -> (&'static str, f64) {
        let dimensions = [
            ("relevance", self.relevance),
            ("accuracy", self.accuracy),
            ("completeness", self.completeness),
            ("clarity", self.clarity),
            ("credibility", self.credibility),
            ("timeliness", self.timeliness),
            ("specificity", self.specificity),
        ];

        dimensions
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
    }
}

impl Default for QualityScore {
    fn default() -> Self {
        Self::new()
    }
}

/// Weighting configuration for quality dimensions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityWeights {
    pub relevance: f64,
    pub accuracy: f64,
    pub completeness: f64,
    pub clarity: f64,
    pub credibility: f64,
    pub timeliness: f64,
    pub specificity: f64,
}

impl QualityWeights {
    /// Create new weights with all dimensions equally weighted
    pub fn new() -> Self {
        Self {
            relevance: 1.0 / 7.0,
            accuracy: 1.0 / 7.0,
            completeness: 1.0 / 7.0,
            clarity: 1.0 / 7.0,
            credibility: 1.0 / 7.0,
            timeliness: 1.0 / 7.0,
            specificity: 1.0 / 7.0,
        }
    }

    /// Create weights optimized for research tasks
    pub fn research_optimized() -> Self {
        Self {
            relevance: 0.25,
            accuracy: 0.25,
            completeness: 0.20,
            clarity: 0.10,
            credibility: 0.15,
            timeliness: 0.03,
            specificity: 0.02,
        }
    }

    /// Create weights optimized for fact-checking tasks
    pub fn fact_checking_optimized() -> Self {
        Self {
            relevance: 0.15,
            accuracy: 0.35,
            completeness: 0.10,
            clarity: 0.10,
            credibility: 0.25,
            timeliness: 0.03,
            specificity: 0.02,
        }
    }

    /// Validate that weights sum to approximately 1.0
    pub fn is_valid(&self) -> bool {
        let sum = self.relevance
            + self.accuracy
            + self.completeness
            + self.clarity
            + self.credibility
            + self.timeliness
            + self.specificity;

        (sum - 1.0).abs() < 0.001
    }

    /// Normalize weights to sum to 1.0
    pub fn normalize(&mut self) {
        let sum = self.relevance
            + self.accuracy
            + self.completeness
            + self.clarity
            + self.credibility
            + self.timeliness
            + self.specificity;

        if sum > 0.0 {
            self.relevance /= sum;
            self.accuracy /= sum;
            self.completeness /= sum;
            self.clarity /= sum;
            self.credibility /= sum;
            self.timeliness /= sum;
            self.specificity /= sum;
        }
    }
}

impl Default for QualityWeights {
    fn default() -> Self {
        Self::new()
    }
}

/// Context information for enhanced quality scoring accuracy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityContext {
    /// The research domain or topic area
    pub domain: Option<String>,
    /// Expected response length range
    pub expected_length: Option<(usize, usize)>,
    /// Required evidence level (low, medium, high)
    pub evidence_level: Option<String>,
    /// Target audience (expert, general, beginner)
    pub audience: Option<String>,
    /// Custom scoring parameters
    pub custom_params: HashMap<String, String>,
}

impl QualityContext {
    pub fn new() -> Self {
        Self {
            domain: None,
            expected_length: None,
            evidence_level: None,
            audience: None,
            custom_params: HashMap::new(),
        }
    }

    pub fn with_domain(mut self, domain: String) -> Self {
        self.domain = Some(domain);
        self
    }

    pub fn with_expected_length(mut self, min: usize, max: usize) -> Self {
        self.expected_length = Some((min, max));
        self
    }

    pub fn with_evidence_level(mut self, level: String) -> Self {
        self.evidence_level = Some(level);
        self
    }

    pub fn with_audience(mut self, audience: String) -> Self {
        self.audience = Some(audience);
        self
    }

    pub fn with_custom_param(mut self, key: String, value: String) -> Self {
        self.custom_params.insert(key, value);
        self
    }
}

impl Default for QualityContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance metrics for quality evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Time taken to perform quality evaluation
    pub evaluation_time: Duration,
    /// Number of tokens processed
    pub tokens_processed: usize,
    /// Memory usage during evaluation (in bytes)
    pub memory_usage: usize,
    /// Cache hit ratio for scoring operations
    pub cache_hit_ratio: f64,
}

impl QualityMetrics {
    pub fn new() -> Self {
        Self {
            evaluation_time: Duration::default(),
            tokens_processed: 0,
            memory_usage: 0,
            cache_hit_ratio: 0.0,
        }
    }

    /// Check if evaluation meets performance requirements
    pub fn meets_performance_requirements(&self) -> bool {
        self.evaluation_time < Duration::from_millis(100) && self.memory_usage < 10 * 1024 * 1024
        // 10MB
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Quality evaluation result combining score and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityEvaluation {
    pub score: QualityScore,
    pub metrics: QualityMetrics,
    pub context: QualityContext,
    pub provider: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl QualityEvaluation {
    pub fn new(
        score: QualityScore,
        metrics: QualityMetrics,
        context: QualityContext,
        provider: String,
    ) -> Self {
        Self {
            score,
            metrics,
            context,
            provider,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Errors that can occur during quality evaluation
#[derive(Error, Debug)]
pub enum QualityError {
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    #[error("Scoring algorithm failed: {algorithm} - {message}")]
    ScoringFailed { algorithm: String, message: String },

    #[error("Performance requirement not met: {requirement} - actual: {actual}")]
    PerformanceViolation { requirement: String, actual: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Feature extraction failed: {message}")]
    FeatureExtractionFailed { message: String },

    #[error("Model inference failed: {message}")]
    ModelInferenceFailed { message: String },

    #[error("Cache operation failed: {message}")]
    CacheFailed { message: String },

    #[error("Network error during evaluation: {source}")]
    NetworkError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Result type for quality operations
pub type QualityResult<T> = Result<T, QualityError>;

/// Core trait for quality scoring implementations
#[async_trait]
pub trait QualityScorer: Send + Sync {
    /// Evaluate the quality of a research response
    async fn evaluate_quality(
        &self,
        query: &str,
        response: &str,
        weights: &QualityWeights,
    ) -> QualityResult<QualityScore>;

    /// Evaluate quality with additional context
    async fn evaluate_quality_with_context(
        &self,
        query: &str,
        response: &str,
        weights: &QualityWeights,
        context: &QualityContext,
    ) -> QualityResult<QualityEvaluation>;

    /// Get scorer metadata and capabilities
    fn metadata(&self) -> ScorerMetadata;

    /// Validate inputs before scoring
    fn validate_inputs(&self, query: &str, response: &str) -> QualityResult<()> {
        if query.trim().is_empty() {
            return Err(QualityError::InvalidInput {
                message: "Query cannot be empty".to_string(),
            });
        }
        if response.trim().is_empty() {
            return Err(QualityError::InvalidInput {
                message: "Response cannot be empty".to_string(),
            });
        }
        Ok(())
    }

    /// Pre-process text for scoring
    async fn preprocess_text(&self, text: &str) -> QualityResult<String> {
        // Default implementation returns text as-is
        Ok(text.to_string())
    }

    /// Extract features for ML-based scoring
    async fn extract_features(
        &self,
        query: &str,
        response: &str,
        context: &QualityContext,
    ) -> QualityResult<FeatureVector>;
}

/// Metadata about a quality scorer implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorerMetadata {
    pub name: String,
    pub version: String,
    pub supported_dimensions: Vec<String>,
    pub performance_characteristics: PerformanceCharacteristics,
    pub accuracy_metrics: AccuracyMetrics,
}

/// Performance characteristics of a scorer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceCharacteristics {
    pub average_evaluation_time: Duration,
    pub max_tokens_per_evaluation: usize,
    pub memory_footprint: usize,
    pub supports_batch_evaluation: bool,
    pub supports_streaming: bool,
}

/// Accuracy metrics for a scorer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyMetrics {
    pub correlation_with_humans: f64,
    pub false_positive_rate: f64,
    pub false_negative_rate: f64,
    pub confidence_interval: (f64, f64),
}

/// Feature vector for ML-based scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    pub features: HashMap<String, f64>,
    pub metadata: HashMap<String, String>,
}

impl FeatureVector {
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_feature(&mut self, name: String, value: f64) {
        self.features.insert(name, value);
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_feature(&self, name: &str) -> Option<f64> {
        self.features.get(name).copied()
    }
}

impl Default for FeatureVector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_score_creation() {
        let score = QualityScore::new();
        assert_eq!(score.relevance, 0.0);
        assert_eq!(score.accuracy, 0.0);
        assert_eq!(score.composite, 0.0);
        assert!(score.is_valid());
    }

    #[test]
    fn test_quality_score_validation() {
        let mut score = QualityScore::new();
        assert!(score.is_valid());

        score.relevance = -0.1; // Invalid: below 0.0
        assert!(!score.is_valid());

        score.relevance = 1.1; // Invalid: above 1.0
        assert!(!score.is_valid());

        score.relevance = 0.5; // Valid
        assert!(score.is_valid());
    }

    #[test]
    fn test_quality_weights_validation() {
        let weights = QualityWeights::default();
        assert!(weights.is_valid());

        let mut invalid_weights = QualityWeights::new();
        invalid_weights.relevance = 0.5; // Sum will be > 1.0
        assert!(!invalid_weights.is_valid());
    }

    #[test]
    fn test_quality_weights_normalization() {
        let mut weights = QualityWeights {
            relevance: 2.0,
            accuracy: 2.0,
            completeness: 2.0,
            clarity: 2.0,
            credibility: 2.0,
            timeliness: 2.0,
            specificity: 2.0,
        };

        assert!(!weights.is_valid());
        weights.normalize();
        assert!(weights.is_valid());

        // Each weight should be approximately 1/7
        let expected = 1.0 / 7.0;
        assert!((weights.relevance - expected).abs() < 0.001);
    }

    #[test]
    fn test_composite_score_calculation() {
        let mut score = QualityScore {
            relevance: 0.8,
            accuracy: 0.9,
            completeness: 0.7,
            clarity: 0.6,
            credibility: 0.8,
            timeliness: 0.5,
            specificity: 0.4,
            composite: 0.0,
            confidence: 0.9,
        };

        let weights = QualityWeights::research_optimized();
        score.calculate_composite(&weights);

        // Verify composite score is calculated correctly
        let expected = 0.8 * 0.25
            + 0.9 * 0.25
            + 0.7 * 0.20
            + 0.6 * 0.10
            + 0.8 * 0.15
            + 0.5 * 0.03
            + 0.4 * 0.02;
        assert!((score.composite - expected).abs() < 0.001);
    }

    #[test]
    fn test_dimension_extremes() {
        let score = QualityScore {
            relevance: 0.9,
            accuracy: 0.8,
            completeness: 0.3, // Lowest
            clarity: 0.7,
            credibility: 0.6,
            timeliness: 0.95, // Highest
            specificity: 0.5,
            composite: 0.0,
            confidence: 0.8,
        };

        let (lowest_dim, lowest_val) = score.lowest_dimension();
        assert_eq!(lowest_dim, "completeness");
        assert_eq!(lowest_val, 0.3);

        let (highest_dim, highest_val) = score.highest_dimension();
        assert_eq!(highest_dim, "timeliness");
        assert_eq!(highest_val, 0.95);
    }

    #[test]
    fn test_quality_context_builder() {
        let context = QualityContext::new()
            .with_domain("machine learning".to_string())
            .with_expected_length(500, 2000)
            .with_evidence_level("high".to_string())
            .with_audience("expert".to_string())
            .with_custom_param("citation_style".to_string(), "apa".to_string());

        assert_eq!(context.domain, Some("machine learning".to_string()));
        assert_eq!(context.expected_length, Some((500, 2000)));
        assert_eq!(context.evidence_level, Some("high".to_string()));
        assert_eq!(context.audience, Some("expert".to_string()));
        assert_eq!(
            context.custom_params.get("citation_style"),
            Some(&"apa".to_string())
        );
    }

    #[test]
    fn test_quality_metrics_performance_check() {
        let mut metrics = QualityMetrics::new();
        metrics.evaluation_time = Duration::from_millis(50);
        metrics.memory_usage = 5 * 1024 * 1024; // 5MB
        assert!(metrics.meets_performance_requirements());

        metrics.evaluation_time = Duration::from_millis(150); // Too slow
        assert!(!metrics.meets_performance_requirements());

        metrics.evaluation_time = Duration::from_millis(50);
        metrics.memory_usage = 15 * 1024 * 1024; // Too much memory
        assert!(!metrics.meets_performance_requirements());
    }

    #[test]
    fn test_feature_vector_operations() {
        let mut features = FeatureVector::new();
        features.add_feature("word_count".to_string(), 150.0);
        features.add_feature("sentiment_score".to_string(), 0.7);
        features.add_metadata("language".to_string(), "english".to_string());

        assert_eq!(features.get_feature("word_count"), Some(150.0));
        assert_eq!(features.get_feature("sentiment_score"), Some(0.7));
        assert_eq!(features.get_feature("nonexistent"), None);
        assert_eq!(
            features.metadata.get("language"),
            Some(&"english".to_string())
        );
    }

    // Mock scorer for testing trait implementations
    struct MockScorer;

    #[async_trait]
    impl QualityScorer for MockScorer {
        async fn evaluate_quality(
            &self,
            query: &str,
            response: &str,
            weights: &QualityWeights,
        ) -> QualityResult<QualityScore> {
            self.validate_inputs(query, response)?;

            // Mock implementation returns fixed scores
            let mut score = QualityScore {
                relevance: 0.8,
                accuracy: 0.7,
                completeness: 0.6,
                clarity: 0.9,
                credibility: 0.5,
                timeliness: 0.8,
                specificity: 0.7,
                composite: 0.0,
                confidence: 0.85,
            };

            score.calculate_composite(weights);
            Ok(score)
        }

        async fn evaluate_quality_with_context(
            &self,
            query: &str,
            response: &str,
            weights: &QualityWeights,
            context: &QualityContext,
        ) -> QualityResult<QualityEvaluation> {
            let score = self.evaluate_quality(query, response, weights).await?;
            let metrics = QualityMetrics {
                evaluation_time: Duration::from_millis(45),
                tokens_processed: query.len() + response.len(),
                memory_usage: 2 * 1024 * 1024, // 2MB
                cache_hit_ratio: 0.8,
            };

            Ok(QualityEvaluation::new(
                score,
                metrics,
                context.clone(),
                "mock_scorer".to_string(),
            ))
        }

        fn metadata(&self) -> ScorerMetadata {
            ScorerMetadata {
                name: "MockScorer".to_string(),
                version: "1.0.0".to_string(),
                supported_dimensions: vec![
                    "relevance".to_string(),
                    "accuracy".to_string(),
                    "completeness".to_string(),
                    "clarity".to_string(),
                    "credibility".to_string(),
                    "timeliness".to_string(),
                    "specificity".to_string(),
                ],
                performance_characteristics: PerformanceCharacteristics {
                    average_evaluation_time: Duration::from_millis(50),
                    max_tokens_per_evaluation: 10000,
                    memory_footprint: 5 * 1024 * 1024,
                    supports_batch_evaluation: false,
                    supports_streaming: false,
                },
                accuracy_metrics: AccuracyMetrics {
                    correlation_with_humans: 0.92,
                    false_positive_rate: 0.05,
                    false_negative_rate: 0.03,
                    confidence_interval: (0.89, 0.95),
                },
            }
        }

        async fn extract_features(
            &self,
            query: &str,
            response: &str,
            _context: &QualityContext,
        ) -> QualityResult<FeatureVector> {
            let mut features = FeatureVector::new();
            features.add_feature("query_length".to_string(), query.len() as f64);
            features.add_feature("response_length".to_string(), response.len() as f64);
            features.add_metadata("scorer".to_string(), "mock".to_string());
            Ok(features)
        }
    }

    #[tokio::test]
    async fn test_quality_scorer_trait() {
        let scorer = MockScorer;
        let query = "What is machine learning?";
        let response = "Machine learning is a subset of artificial intelligence...";
        let weights = QualityWeights::research_optimized();

        let result = scorer.evaluate_quality(query, response, &weights).await;
        assert!(result.is_ok());

        let score = result.unwrap();
        assert!(score.is_valid());
        assert!(score.composite > 0.0);
    }

    #[tokio::test]
    async fn test_quality_scorer_with_context() {
        let scorer = MockScorer;
        let query = "Explain neural networks";
        let response =
            "Neural networks are computing systems inspired by biological neural networks...";
        let weights = QualityWeights::research_optimized();
        let context = QualityContext::new()
            .with_domain("machine learning".to_string())
            .with_audience("beginner".to_string());

        let result = scorer
            .evaluate_quality_with_context(query, response, &weights, &context)
            .await;
        assert!(result.is_ok());

        let evaluation = result.unwrap();
        assert!(evaluation.score.is_valid());
        assert!(evaluation.metrics.meets_performance_requirements());
        assert_eq!(evaluation.provider, "mock_scorer");
    }

    #[tokio::test]
    async fn test_quality_scorer_validation() {
        let scorer = MockScorer;
        let weights = QualityWeights::default();

        // Test empty query
        let result = scorer
            .evaluate_quality("", "valid response", &weights)
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            QualityError::InvalidInput { .. }
        ));

        // Test empty response
        let result = scorer.evaluate_quality("valid query", "", &weights).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            QualityError::InvalidInput { .. }
        ));
    }

    #[tokio::test]
    async fn test_feature_extraction() {
        let scorer = MockScorer;
        let query = "What is AI?";
        let response = "Artificial Intelligence is...";
        let context = QualityContext::new();

        let result = scorer.extract_features(query, response, &context).await;
        assert!(result.is_ok());

        let features = result.unwrap();
        assert_eq!(
            features.get_feature("query_length"),
            Some(query.len() as f64)
        );
        assert_eq!(
            features.get_feature("response_length"),
            Some(response.len() as f64)
        );
        assert_eq!(features.metadata.get("scorer"), Some(&"mock".to_string()));
    }

    #[test]
    fn test_scorer_metadata() {
        let scorer = MockScorer;
        let metadata = scorer.metadata();

        assert_eq!(metadata.name, "MockScorer");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.supported_dimensions.len(), 7);
        assert!(metadata.accuracy_metrics.correlation_with_humans > 0.9);
        assert!(
            metadata.performance_characteristics.average_evaluation_time
                < Duration::from_millis(100)
        );
    }
}
