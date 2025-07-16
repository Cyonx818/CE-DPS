// ABOUTME: Research quality feedback system for improving vector search relevance
//! This module provides a feedback system for research quality assessment and
//! vector search relevance improvement based on research outcomes.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info, warn};

use crate::vector::HybridSearchService;
use fortitude_types::{ResearchResult, ResearchType};

/// Errors that can occur during feedback processing
#[derive(Error, Debug)]
pub enum FeedbackError {
    #[error("Feedback processing error: {0}")]
    ProcessingError(String),

    #[error("Vector search error: {0}")]
    VectorSearchError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Invalid feedback data: {0}")]
    InvalidData(String),
}

/// Quality assessment for a research result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchQualityFeedback {
    /// Unique feedback ID
    pub feedback_id: String,
    /// Associated research result cache key
    pub research_cache_key: String,
    /// User-provided quality rating (1-5)
    pub quality_rating: u8,
    /// Relevance of discovered context (1-5)
    pub context_relevance: u8,
    /// Usefulness of the research answer (1-5)
    pub answer_usefulness: u8,
    /// Whether the research was complete
    pub completeness_rating: u8,
    /// Free-form feedback text
    pub feedback_text: Option<String>,
    /// Specific issues identified
    pub issues: Vec<QualityIssue>,
    /// Suggestions for improvement
    pub suggestions: Vec<String>,
    /// Timestamp of feedback
    pub provided_at: DateTime<Utc>,
    /// Source of feedback (user, automated, etc.)
    pub feedback_source: FeedbackSource,
}

/// Specific quality issues identified in research
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    /// Type of issue
    pub issue_type: IssueType,
    /// Severity of the issue
    pub severity: IssueSeverity,
    /// Description of the issue
    pub description: String,
    /// Suggested fix
    pub suggested_fix: Option<String>,
}

/// Types of quality issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueType {
    /// Research answer was incomplete
    IncompleteAnswer,
    /// Context was not relevant
    IrrelevantContext,
    /// Answer contained inaccuracies
    InaccurateInformation,
    /// Research missed important aspects
    MissedKeyPoints,
    /// Examples were not helpful
    PoorExamples,
    /// Language was too complex/simple
    InappropriateLevel,
    /// Structure was confusing
    PoorStructure,
    /// Sources were unreliable
    UnreliableSources,
}

/// Severity levels for issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Low impact issue
    Low,
    /// Medium impact issue
    Medium,
    /// High impact issue
    High,
    /// Critical issue that renders research unusable
    Critical,
}

/// Source of the feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackSource {
    /// Direct user feedback
    User,
    /// Automated quality assessment
    Automated,
    /// Peer review
    PeerReview,
    /// System analysis
    SystemAnalysis,
}

/// Aggregated feedback analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackAnalytics {
    /// Total number of feedback entries
    pub total_feedback_count: u64,
    /// Average quality rating
    pub avg_quality_rating: f64,
    /// Average context relevance
    pub avg_context_relevance: f64,
    /// Average answer usefulness
    pub avg_answer_usefulness: f64,
    /// Average completeness rating
    pub avg_completeness_rating: f64,
    /// Issue frequency by type
    pub issue_frequency: HashMap<String, u64>,
    /// Research type performance
    pub research_type_performance: HashMap<ResearchType, f64>,
    /// Context discovery effectiveness
    pub context_discovery_effectiveness: f64,
    /// Most common suggestions
    pub common_suggestions: Vec<(String, u64)>,
}

/// Research feedback processor
pub struct ResearchFeedbackProcessor {
    /// Vector search service for relevance improvements
    #[allow(dead_code)] // TODO: Implement vector search integration for feedback relevance
    vector_search: Arc<HybridSearchService>,
    /// Vector storage for feedback indexing
    #[allow(dead_code)] // TODO: Implement vector storage for feedback indexing
    vector_storage: Arc<dyn crate::vector::VectorStorageService + Send + Sync>,
    /// Feedback storage
    feedback_storage: FeedbackStorage,
    /// Configuration
    config: FeedbackConfig,
}

/// Configuration for feedback processing
#[derive(Debug, Clone)]
pub struct FeedbackConfig {
    /// Minimum feedback count before making adjustments
    pub min_feedback_threshold: u64,
    /// Weight for user feedback vs automated assessment
    pub user_feedback_weight: f64,
    /// Enable automatic search relevance tuning
    pub enable_relevance_tuning: bool,
    /// Feedback aggregation window in days
    pub aggregation_window_days: u32,
}

impl Default for FeedbackConfig {
    fn default() -> Self {
        Self {
            min_feedback_threshold: 10,
            user_feedback_weight: 0.8,
            enable_relevance_tuning: true,
            aggregation_window_days: 30,
        }
    }
}

/// Storage for feedback data
pub struct FeedbackStorage {
    /// In-memory storage for demo purposes
    /// In production, this would be backed by a persistent database
    feedback_entries: tokio::sync::RwLock<Vec<ResearchQualityFeedback>>,
}

impl Default for FeedbackStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl FeedbackStorage {
    /// Create a new feedback storage
    pub fn new() -> Self {
        Self {
            feedback_entries: tokio::sync::RwLock::new(Vec::new()),
        }
    }

    /// Store a feedback entry
    pub async fn store_feedback(
        &self,
        feedback: ResearchQualityFeedback,
    ) -> Result<(), FeedbackError> {
        let mut entries = self.feedback_entries.write().await;
        entries.push(feedback);
        Ok(())
    }

    /// Get feedback for a research result
    pub async fn get_feedback_for_research(
        &self,
        cache_key: &str,
    ) -> Result<Vec<ResearchQualityFeedback>, FeedbackError> {
        let entries = self.feedback_entries.read().await;
        Ok(entries
            .iter()
            .filter(|f| f.research_cache_key == cache_key)
            .cloned()
            .collect())
    }

    /// Get all feedback entries
    pub async fn get_all_feedback(&self) -> Result<Vec<ResearchQualityFeedback>, FeedbackError> {
        let entries = self.feedback_entries.read().await;
        Ok(entries.clone())
    }

    /// Get feedback within a time window
    pub async fn get_feedback_in_window(
        &self,
        days: u32,
    ) -> Result<Vec<ResearchQualityFeedback>, FeedbackError> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        let entries = self.feedback_entries.read().await;
        Ok(entries
            .iter()
            .filter(|f| f.provided_at > cutoff)
            .cloned()
            .collect())
    }
}

impl ResearchFeedbackProcessor {
    /// Create a new feedback processor
    pub fn new(
        vector_search: Arc<HybridSearchService>,
        vector_storage: Arc<dyn crate::vector::VectorStorageService + Send + Sync>,
        config: FeedbackConfig,
    ) -> Self {
        Self {
            vector_search,
            vector_storage,
            feedback_storage: FeedbackStorage::new(),
            config,
        }
    }

    /// Process user feedback for a research result
    pub async fn process_feedback(
        &self,
        feedback: ResearchQualityFeedback,
    ) -> Result<(), FeedbackError> {
        info!(
            "Processing feedback for research: {}",
            feedback.research_cache_key
        );

        // Validate feedback
        self.validate_feedback(&feedback)?;

        // Store feedback
        self.feedback_storage
            .store_feedback(feedback.clone())
            .await?;

        // Update search relevance if enabled
        if self.config.enable_relevance_tuning {
            self.update_search_relevance(&feedback).await?;
        }

        // Log feedback analytics
        self.log_feedback_impact(&feedback).await;

        info!(
            "Feedback processed successfully for: {}",
            feedback.research_cache_key
        );
        Ok(())
    }

    /// Validate feedback data
    fn validate_feedback(&self, feedback: &ResearchQualityFeedback) -> Result<(), FeedbackError> {
        if feedback.quality_rating < 1 || feedback.quality_rating > 5 {
            return Err(FeedbackError::InvalidData(
                "Quality rating must be between 1 and 5".to_string(),
            ));
        }
        if feedback.context_relevance < 1 || feedback.context_relevance > 5 {
            return Err(FeedbackError::InvalidData(
                "Context relevance must be between 1 and 5".to_string(),
            ));
        }
        if feedback.answer_usefulness < 1 || feedback.answer_usefulness > 5 {
            return Err(FeedbackError::InvalidData(
                "Answer usefulness must be between 1 and 5".to_string(),
            ));
        }
        if feedback.completeness_rating < 1 || feedback.completeness_rating > 5 {
            return Err(FeedbackError::InvalidData(
                "Completeness rating must be between 1 and 5".to_string(),
            ));
        }
        Ok(())
    }

    /// Update search relevance based on feedback
    async fn update_search_relevance(
        &self,
        feedback: &ResearchQualityFeedback,
    ) -> Result<(), FeedbackError> {
        debug!("Updating search relevance based on feedback");

        // Get feedback analytics for this research
        let research_feedback = self
            .feedback_storage
            .get_feedback_for_research(&feedback.research_cache_key)
            .await?;

        if research_feedback.len() < self.config.min_feedback_threshold as usize {
            debug!("Insufficient feedback for relevance tuning");
            return Ok(());
        }

        // Calculate relevance adjustments
        let avg_context_relevance: f64 = research_feedback
            .iter()
            .map(|f| f.context_relevance as f64)
            .sum::<f64>()
            / research_feedback.len() as f64;

        // If context relevance is consistently low, we may need to adjust search strategies
        if avg_context_relevance < 2.5 {
            warn!(
                "Low context relevance detected for research: {}",
                feedback.research_cache_key
            );
            // TODO: Implement actual relevance tuning
            // This could involve adjusting search parameters, fusion weights, etc.
        }

        Ok(())
    }

    /// Log feedback impact for analytics
    async fn log_feedback_impact(&self, feedback: &ResearchQualityFeedback) {
        debug!(
            "Feedback impact - Quality: {}, Context: {}, Usefulness: {}, Completeness: {}",
            feedback.quality_rating,
            feedback.context_relevance,
            feedback.answer_usefulness,
            feedback.completeness_rating
        );

        if !feedback.issues.is_empty() {
            info!(
                "Issues identified in feedback: {} issues",
                feedback.issues.len()
            );
            for issue in &feedback.issues {
                debug!(
                    "Issue: {:?} - {} ({})",
                    issue.issue_type,
                    issue.description,
                    format!("{:?}", issue.severity)
                );
            }
        }
    }

    /// Generate feedback analytics
    pub async fn generate_analytics(&self) -> Result<FeedbackAnalytics, FeedbackError> {
        let all_feedback = self
            .feedback_storage
            .get_feedback_in_window(self.config.aggregation_window_days)
            .await?;

        if all_feedback.is_empty() {
            return Ok(FeedbackAnalytics {
                total_feedback_count: 0,
                avg_quality_rating: 0.0,
                avg_context_relevance: 0.0,
                avg_answer_usefulness: 0.0,
                avg_completeness_rating: 0.0,
                issue_frequency: HashMap::new(),
                research_type_performance: HashMap::new(),
                context_discovery_effectiveness: 0.0,
                common_suggestions: Vec::new(),
            });
        }

        let count = all_feedback.len() as f64;

        // Calculate averages
        let avg_quality_rating = all_feedback
            .iter()
            .map(|f| f.quality_rating as f64)
            .sum::<f64>()
            / count;

        let avg_context_relevance = all_feedback
            .iter()
            .map(|f| f.context_relevance as f64)
            .sum::<f64>()
            / count;

        let avg_answer_usefulness = all_feedback
            .iter()
            .map(|f| f.answer_usefulness as f64)
            .sum::<f64>()
            / count;

        let avg_completeness_rating = all_feedback
            .iter()
            .map(|f| f.completeness_rating as f64)
            .sum::<f64>()
            / count;

        // Calculate issue frequency
        let mut issue_frequency = HashMap::new();
        for feedback in &all_feedback {
            for issue in &feedback.issues {
                let issue_type_str = format!("{:?}", issue.issue_type);
                *issue_frequency.entry(issue_type_str).or_insert(0) += 1;
            }
        }

        // Calculate suggestion frequency
        let mut suggestion_frequency: HashMap<String, u64> = HashMap::new();
        for feedback in &all_feedback {
            for suggestion in &feedback.suggestions {
                *suggestion_frequency.entry(suggestion.clone()).or_insert(0) += 1;
            }
        }

        let mut common_suggestions: Vec<(String, u64)> = suggestion_frequency.into_iter().collect();
        common_suggestions.sort_by(|a, b| b.1.cmp(&a.1));
        common_suggestions.truncate(10); // Top 10 suggestions

        Ok(FeedbackAnalytics {
            total_feedback_count: all_feedback.len() as u64,
            avg_quality_rating,
            avg_context_relevance,
            avg_answer_usefulness,
            avg_completeness_rating,
            issue_frequency,
            research_type_performance: HashMap::new(), // TODO: Implement per-type analysis
            context_discovery_effectiveness: avg_context_relevance,
            common_suggestions,
        })
    }

    /// Provide research improvement recommendations
    pub async fn get_improvement_recommendations(&self) -> Result<Vec<String>, FeedbackError> {
        let analytics = self.generate_analytics().await?;
        let mut recommendations = Vec::new();

        // Quality-based recommendations
        if analytics.avg_quality_rating < 3.0 {
            recommendations.push("Consider improving research depth and accuracy".to_string());
        }

        if analytics.avg_context_relevance < 3.0 {
            recommendations.push(
                "Review vector search configuration for better context discovery".to_string(),
            );
        }

        if analytics.avg_answer_usefulness < 3.0 {
            recommendations
                .push("Focus on providing more actionable and practical guidance".to_string());
        }

        if analytics.avg_completeness_rating < 3.0 {
            recommendations
                .push("Ensure research covers all important aspects of the query".to_string());
        }

        // Issue-based recommendations
        for (issue_type, count) in &analytics.issue_frequency {
            if *count > analytics.total_feedback_count / 4 {
                match issue_type.as_str() {
                    "IrrelevantContext" => recommendations
                        .push("Adjust vector search relevance thresholds".to_string()),
                    "IncompleteAnswer" => {
                        recommendations.push("Extend research depth and coverage".to_string())
                    }
                    "PoorExamples" => recommendations
                        .push("Include more practical, real-world examples".to_string()),
                    "InappropriateLevel" => {
                        recommendations.push("Better tailor content to audience level".to_string())
                    }
                    _ => {}
                }
            }
        }

        if recommendations.is_empty() {
            recommendations.push(
                "Research quality is satisfactory. Continue monitoring feedback.".to_string(),
            );
        }

        Ok(recommendations)
    }

    /// Create feedback for automated quality assessment
    pub async fn create_automated_feedback(
        &self,
        research_result: &ResearchResult,
        quality_metrics: AutomatedQualityMetrics,
    ) -> Result<ResearchQualityFeedback, FeedbackError> {
        let feedback_id = format!(
            "auto_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );

        // Convert automated metrics to feedback ratings
        let quality_rating = (quality_metrics.overall_score * 5.0).round() as u8;
        let context_relevance = (quality_metrics.context_relevance * 5.0).round() as u8;
        let answer_usefulness = (quality_metrics.answer_quality * 5.0).round() as u8;
        let completeness_rating = (quality_metrics.completeness * 5.0).round() as u8;

        let mut issues = Vec::new();
        if quality_metrics.context_relevance < 0.6 {
            issues.push(QualityIssue {
                issue_type: IssueType::IrrelevantContext,
                severity: IssueSeverity::Medium,
                description: "Automated analysis detected low context relevance".to_string(),
                suggested_fix: Some("Adjust search relevance thresholds".to_string()),
            });
        }

        if quality_metrics.completeness < 0.7 {
            issues.push(QualityIssue {
                issue_type: IssueType::IncompleteAnswer,
                severity: IssueSeverity::Medium,
                description: "Research appears incomplete based on query complexity".to_string(),
                suggested_fix: Some("Expand research coverage".to_string()),
            });
        }

        Ok(ResearchQualityFeedback {
            feedback_id,
            research_cache_key: research_result.metadata.cache_key.clone(),
            quality_rating: quality_rating.clamp(1, 5),
            context_relevance: context_relevance.clamp(1, 5),
            answer_usefulness: answer_usefulness.clamp(1, 5),
            completeness_rating: completeness_rating.clamp(1, 5),
            feedback_text: Some(format!(
                "Automated assessment: {:.2}",
                quality_metrics.overall_score
            )),
            issues,
            suggestions: Vec::new(),
            provided_at: Utc::now(),
            feedback_source: FeedbackSource::Automated,
        })
    }
}

/// Automated quality metrics
#[derive(Debug, Clone)]
pub struct AutomatedQualityMetrics {
    /// Overall quality score (0.0-1.0)
    pub overall_score: f64,
    /// Context relevance score (0.0-1.0)
    pub context_relevance: f64,
    /// Answer quality score (0.0-1.0)
    pub answer_quality: f64,
    /// Completeness score (0.0-1.0)
    pub completeness: f64,
}

/// Trait for feedback operations
#[async_trait]
pub trait FeedbackOperations: Send + Sync {
    /// Process user feedback
    async fn process_feedback(
        &self,
        feedback: ResearchQualityFeedback,
    ) -> Result<(), FeedbackError>;

    /// Generate analytics
    async fn generate_analytics(&self) -> Result<FeedbackAnalytics, FeedbackError>;

    /// Get improvement recommendations
    async fn get_improvement_recommendations(&self) -> Result<Vec<String>, FeedbackError>;

    /// Create automated feedback
    async fn create_automated_feedback(
        &self,
        research_result: &ResearchResult,
        quality_metrics: AutomatedQualityMetrics,
    ) -> Result<ResearchQualityFeedback, FeedbackError>;
}

#[async_trait]
impl FeedbackOperations for ResearchFeedbackProcessor {
    async fn process_feedback(
        &self,
        feedback: ResearchQualityFeedback,
    ) -> Result<(), FeedbackError> {
        self.process_feedback(feedback).await
    }

    async fn generate_analytics(&self) -> Result<FeedbackAnalytics, FeedbackError> {
        self.generate_analytics().await
    }

    async fn get_improvement_recommendations(&self) -> Result<Vec<String>, FeedbackError> {
        self.get_improvement_recommendations().await
    }

    async fn create_automated_feedback(
        &self,
        research_result: &ResearchResult,
        quality_metrics: AutomatedQualityMetrics,
    ) -> Result<ResearchQualityFeedback, FeedbackError> {
        self.create_automated_feedback(research_result, quality_metrics)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_validation() {
        let processor = match create_test_processor() {
            Some(p) => p,
            None => {
                eprintln!("Skipping test - Qdrant server not available");
                return;
            }
        };

        let valid_feedback = ResearchQualityFeedback {
            feedback_id: "test-1".to_string(),
            research_cache_key: "test-cache-key".to_string(),
            quality_rating: 4,
            context_relevance: 3,
            answer_usefulness: 5,
            completeness_rating: 4,
            feedback_text: None,
            issues: Vec::new(),
            suggestions: Vec::new(),
            provided_at: Utc::now(),
            feedback_source: FeedbackSource::User,
        };

        assert!(processor.validate_feedback(&valid_feedback).is_ok());

        // Test invalid rating
        let mut invalid_feedback = valid_feedback.clone();
        invalid_feedback.quality_rating = 6;
        assert!(processor.validate_feedback(&invalid_feedback).is_err());
    }

    #[test]
    fn test_quality_issue_creation() {
        let issue = QualityIssue {
            issue_type: IssueType::IrrelevantContext,
            severity: IssueSeverity::High,
            description: "Context not related to query".to_string(),
            suggested_fix: Some("Adjust search parameters".to_string()),
        };

        assert!(matches!(issue.issue_type, IssueType::IrrelevantContext));
        assert!(matches!(issue.severity, IssueSeverity::High));
        assert_eq!(issue.description, "Context not related to query");
    }

    fn create_test_processor() -> Option<ResearchFeedbackProcessor> {
        // Create minimal test implementations
        use crate::vector::{
            EmbeddingConfig, HybridSearchService, KeywordSearcher, LocalEmbeddingService,
            QdrantClient, SemanticSearchService, VectorConfig, VectorStorage,
        };

        // Create test storage
        let embedding_config = EmbeddingConfig::default();
        let embedding_service = Arc::new(LocalEmbeddingService::new(embedding_config));
        let qdrant_config = VectorConfig::default();

        // Create a runtime for async operations in tests
        let rt = tokio::runtime::Runtime::new().unwrap();
        let qdrant_client = match rt.block_on(async { QdrantClient::new(qdrant_config).await }) {
            Ok(client) => client,
            Err(_) => {
                // Qdrant server not available, skip test
                eprintln!("Skipping test - Qdrant server not available");
                return None;
            }
        };

        let vector_storage = Arc::new(VectorStorage::new(
            Arc::new(qdrant_client),
            embedding_service.clone(),
        ));

        // Create hybrid search service with config
        let search_config = crate::vector::search::SemanticSearchConfig {
            default_limit: 10,
            default_threshold: 0.7,
            max_limit: 100,
            enable_analytics: true,
            cache_results: true,
            cache_ttl_seconds: 300,
            enable_query_optimization: true,
            max_query_length: 8192,
        };
        let semantic_service = Arc::new(SemanticSearchService::new(
            vector_storage.clone(),
            search_config,
        ));
        let keyword_searcher = Arc::new(KeywordSearcher::new());
        let hybrid_search = Arc::new(HybridSearchService::with_defaults(
            semantic_service,
            keyword_searcher,
        ));

        // Create test config
        let config = FeedbackConfig {
            min_feedback_threshold: 3,
            user_feedback_weight: 0.7,
            enable_relevance_tuning: true,
            aggregation_window_days: 30,
        };

        Some(ResearchFeedbackProcessor::new(
            hybrid_search,
            vector_storage,
            config,
        ))
    }
}
