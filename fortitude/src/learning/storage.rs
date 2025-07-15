// ABOUTME: Learning data persistence with vector database integration
//! Learning Data Storage Layer
//!
//! This module provides persistent storage for learning system data including
//! user feedback, usage patterns, and derived insights. It integrates with
//! the existing vector database infrastructure for efficient storage and retrieval.

use crate::learning::{
    LearningData, LearningError, LearningResult, LearningStorageConfig, PatternData, UsagePattern,
    UserFeedback,
};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use fortitude_core::vector::storage::{
    BatchError, BatchResult, DocumentMetadata, SearchConfig, VectorDocument, VectorStorageService,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

/// Trait defining the interface for learning data storage operations
#[async_trait]
pub trait LearningStorageService: Send + Sync {
    /// Store user feedback
    async fn store_feedback(&self, feedback: &UserFeedback) -> LearningResult<UserFeedback>;

    /// Retrieve feedback by ID
    async fn get_feedback(&self, id: &str) -> LearningResult<Option<UserFeedback>>;

    /// Get feedback for a specific content ID
    async fn get_feedback_for_content(&self, content_id: &str)
        -> LearningResult<Vec<UserFeedback>>;

    /// Store pattern data
    async fn store_pattern(&self, pattern: &PatternData) -> LearningResult<PatternData>;

    /// Get patterns by type
    async fn get_patterns_by_type(&self, pattern_type: &str) -> LearningResult<Vec<PatternData>>;

    /// Store learning insights
    async fn store_learning_data(&self, data: &LearningData) -> LearningResult<LearningData>;

    /// Get recent learning data
    async fn get_recent_learning_data(&self, limit: usize) -> LearningResult<Vec<LearningData>>;

    /// Store usage pattern
    async fn store_usage_pattern(&self, pattern: &UsagePattern) -> LearningResult<UsagePattern>;

    /// Get top patterns by frequency
    async fn get_top_patterns(
        &self,
        pattern_type: &str,
        limit: usize,
    ) -> LearningResult<Vec<UsagePattern>>;

    /// Get trending patterns
    async fn get_trending_patterns(
        &self,
        pattern_type: &str,
        days: u32,
    ) -> LearningResult<Vec<UsagePattern>>;

    /// Get average feedback score for content
    async fn get_average_feedback_score(&self, content_id: &str) -> LearningResult<Option<f64>>;

    /// Get feedback trend analysis
    async fn get_feedback_trend(
        &self,
        content_id: &str,
        days: u32,
    ) -> LearningResult<FeedbackTrend>;

    /// Get recent feedback for content
    async fn get_recent_feedback(
        &self,
        content_id: &str,
        limit: usize,
    ) -> LearningResult<Vec<UserFeedback>>;

    /// Cleanup old data based on retention policy
    async fn cleanup_old_data(&self, retention_days: u32) -> LearningResult<CleanupResult>;

    /// Initialize the storage layer
    async fn initialize(&self) -> LearningResult<()>;
}

/// Enhanced learning storage service with vector database integration
#[async_trait]
pub trait EnhancedLearningStorageService: LearningStorageService {
    /// Find similar learning insights using vector similarity
    async fn find_similar_insights(
        &self,
        query: &str,
        similarity_threshold: f64,
        max_results: usize,
    ) -> LearningResult<Vec<SimilarityLearningResult>>;

    /// Get embedding for stored feedback
    async fn get_feedback_embedding(&self, feedback_id: &str) -> LearningResult<Option<Vec<f32>>>;

    /// Get embedding cache statistics
    async fn get_embedding_cache_stats(&self) -> LearningResult<EmbeddingCacheStats>;

    /// Store multiple learning data entries in batch
    async fn store_learning_data_batch(
        &self,
        data_batch: &[LearningData],
    ) -> LearningResult<BatchResult<LearningData>>;

    /// Retrieve multiple learning data entries by IDs
    async fn retrieve_learning_data_batch(
        &self,
        ids: &[String],
    ) -> LearningResult<BatchResult<LearningData>>;

    /// Update multiple learning data entries in batch
    async fn update_learning_data_batch(
        &self,
        data_batch: &[LearningData],
    ) -> LearningResult<BatchResult<LearningData>>;

    /// Find similar usage patterns using vector embeddings
    async fn find_similar_usage_patterns(
        &self,
        query: &str,
        similarity_threshold: f64,
        max_results: usize,
    ) -> LearningResult<Vec<SimilarityUsagePattern>>;

    /// Cluster usage patterns by similarity
    async fn cluster_usage_patterns_by_similarity(
        &self,
        pattern_type: &str,
        similarity_threshold: f64,
        max_clusters: usize,
    ) -> LearningResult<Vec<Vec<UsagePattern>>>;

    /// Get specific learning data by ID
    async fn get_learning_data(&self, id: &str) -> LearningResult<Option<LearningData>>;

    /// Cleanup expired learning data
    async fn cleanup_expired_learning_data(&self) -> LearningResult<CleanupResult>;
}

/// Vector database-backed learning storage implementation
pub struct VectorLearningStorage {
    /// Vector storage service for learning data
    vector_storage: Arc<dyn VectorStorageService>,

    /// Configuration for learning storage
    #[allow(dead_code)] // TODO: Will be used for storage configuration
    config: LearningStorageConfig,

    /// Collection names for different data types
    #[allow(dead_code)] // TODO: Will be used for feedback data collection
    feedback_collection: String,
    #[allow(dead_code)] // TODO: Will be used for pattern data collection
    pattern_collection: String,
    learning_collection: String,
    usage_collection: String,
}

impl VectorLearningStorage {
    /// Create a new vector learning storage instance
    pub fn new(
        vector_storage: Arc<dyn VectorStorageService>,
        config: LearningStorageConfig,
    ) -> Self {
        let base_name = config.collection_name.clone();

        Self {
            vector_storage,
            config,
            feedback_collection: format!("{base_name}_feedback"),
            pattern_collection: format!("{base_name}_patterns"),
            learning_collection: format!("{base_name}_insights"),
            usage_collection: format!("{base_name}_usage"),
        }
    }

    /// Convert user feedback to vector document
    fn feedback_to_document(
        &self,
        feedback: &UserFeedback,
    ) -> LearningResult<(String, DocumentMetadata)> {
        let content = self.serialize_feedback_content(feedback)?;

        let mut custom_fields = HashMap::new();
        custom_fields.insert("data_type".to_string(), serde_json::json!("feedback"));
        custom_fields.insert("user_id".to_string(), serde_json::json!(feedback.user_id));
        custom_fields.insert(
            "content_id".to_string(),
            serde_json::json!(feedback.content_id),
        );
        custom_fields.insert(
            "feedback_type".to_string(),
            serde_json::json!(feedback.feedback_type),
        );

        if let Some(score) = feedback.score {
            custom_fields.insert("score".to_string(), serde_json::json!(score));
        }

        custom_fields.insert(
            "timestamp".to_string(),
            serde_json::json!(feedback.timestamp.to_rfc3339()),
        );

        // Include metadata
        for (key, value) in &feedback.metadata {
            custom_fields.insert(format!("meta_{key}"), value.clone());
        }

        let metadata = DocumentMetadata {
            research_type: None,
            content_type: "learning_feedback".to_string(),
            quality_score: feedback.score,
            source: Some("user_feedback".to_string()),
            tags: vec![
                "learning".to_string(),
                "feedback".to_string(),
                feedback.feedback_type.clone(),
            ],
            custom_fields,
        };

        Ok((content, metadata))
    }

    /// Convert pattern data to vector document
    fn pattern_to_document(
        &self,
        pattern: &PatternData,
    ) -> LearningResult<(String, DocumentMetadata)> {
        let content = self.serialize_pattern_content(pattern)?;

        let mut custom_fields = HashMap::new();
        custom_fields.insert("data_type".to_string(), serde_json::json!("pattern"));
        custom_fields.insert(
            "pattern_type".to_string(),
            serde_json::json!(pattern.pattern_type),
        );
        custom_fields.insert(
            "frequency".to_string(),
            serde_json::json!(pattern.frequency),
        );
        custom_fields.insert(
            "success_rate".to_string(),
            serde_json::json!(pattern.success_rate),
        );
        custom_fields.insert(
            "first_seen".to_string(),
            serde_json::json!(pattern.first_seen.to_rfc3339()),
        );
        custom_fields.insert(
            "last_seen".to_string(),
            serde_json::json!(pattern.last_seen.to_rfc3339()),
        );

        // Include context
        for (key, value) in &pattern.context {
            custom_fields.insert(format!("ctx_{key}"), value.clone());
        }

        let metadata = DocumentMetadata {
            research_type: None,
            content_type: "learning_pattern".to_string(),
            quality_score: Some(pattern.success_rate),
            source: Some("pattern_analysis".to_string()),
            tags: vec![
                "learning".to_string(),
                "pattern".to_string(),
                pattern.pattern_type.clone(),
            ],
            custom_fields,
        };

        Ok((content, metadata))
    }

    /// Convert learning data to vector document
    fn learning_to_document(
        &self,
        learning: &LearningData,
    ) -> LearningResult<(String, DocumentMetadata)> {
        let content = self.serialize_learning_content(learning)?;

        let mut custom_fields = HashMap::new();
        custom_fields.insert("data_type".to_string(), serde_json::json!("learning"));
        custom_fields.insert(
            "learning_type".to_string(),
            serde_json::json!(learning.learning_type),
        );
        custom_fields.insert(
            "source_data_id".to_string(),
            serde_json::json!(learning.source_data_id),
        );
        custom_fields.insert(
            "confidence_score".to_string(),
            serde_json::json!(learning.confidence_score),
        );
        custom_fields.insert(
            "created_at".to_string(),
            serde_json::json!(learning.created_at.to_rfc3339()),
        );

        if let Some(expires_at) = learning.expires_at {
            custom_fields.insert(
                "expires_at".to_string(),
                serde_json::json!(expires_at.to_rfc3339()),
            );
        }

        // Include metadata
        for (key, value) in &learning.metadata {
            custom_fields.insert(format!("meta_{key}"), value.clone());
        }

        let metadata = DocumentMetadata {
            research_type: None,
            content_type: "learning_insight".to_string(),
            quality_score: Some(learning.confidence_score),
            source: Some("learning_analysis".to_string()),
            tags: vec![
                "learning".to_string(),
                "insight".to_string(),
                learning.learning_type.clone(),
            ],
            custom_fields,
        };

        Ok((content, metadata))
    }

    /// Convert usage pattern to vector document
    fn usage_to_document(
        &self,
        usage: &UsagePattern,
    ) -> LearningResult<(String, DocumentMetadata)> {
        let content = self.serialize_usage_content(usage)?;

        let mut custom_fields = HashMap::new();
        custom_fields.insert("data_type".to_string(), serde_json::json!("usage"));
        custom_fields.insert(
            "pattern_type".to_string(),
            serde_json::json!(usage.pattern_type),
        );
        custom_fields.insert("data".to_string(), serde_json::json!(usage.data));
        custom_fields.insert("frequency".to_string(), serde_json::json!(usage.frequency));
        custom_fields.insert(
            "last_used".to_string(),
            serde_json::json!(usage.last_used.to_rfc3339()),
        );

        // Include context
        for (key, value) in &usage.context {
            custom_fields.insert(format!("ctx_{key}"), value.clone());
        }

        let metadata = DocumentMetadata {
            research_type: None,
            content_type: "usage_pattern".to_string(),
            quality_score: None,
            source: Some("usage_tracking".to_string()),
            tags: vec![
                "learning".to_string(),
                "usage".to_string(),
                usage.pattern_type.clone(),
            ],
            custom_fields,
        };

        Ok((content, metadata))
    }

    /// Serialize feedback for vector storage
    fn serialize_feedback_content(&self, feedback: &UserFeedback) -> LearningResult<String> {
        let mut content_parts = vec![
            format!("Feedback Type: {}", feedback.feedback_type),
            format!("User: {}", feedback.user_id),
            format!("Content: {}", feedback.content_id),
        ];

        if let Some(score) = feedback.score {
            content_parts.push(format!("Score: {score:.2}"));
        }

        if let Some(text) = &feedback.text_feedback {
            content_parts.push(format!("Text: {text}"));
        }

        Ok(content_parts.join("\n"))
    }

    /// Serialize pattern for vector storage
    fn serialize_pattern_content(&self, pattern: &PatternData) -> LearningResult<String> {
        Ok(format!(
            "Pattern Type: {}\nFrequency: {}\nSuccess Rate: {:.2}\nContext: {}",
            pattern.pattern_type,
            pattern.frequency,
            pattern.success_rate,
            serde_json::to_string(&pattern.context)
                .map_err(|e| LearningError::SerializationError(e.to_string()))?
        ))
    }

    /// Serialize learning data for vector storage
    fn serialize_learning_content(&self, learning: &LearningData) -> LearningResult<String> {
        let insights_text = learning.insights.join("; ");
        Ok(format!(
            "Learning Type: {}\nInsights: {}\nConfidence: {:.2}\nSource: {}",
            learning.learning_type,
            insights_text,
            learning.confidence_score,
            learning.source_data_id
        ))
    }

    /// Serialize usage pattern for vector storage
    fn serialize_usage_content(&self, usage: &UsagePattern) -> LearningResult<String> {
        Ok(format!(
            "Pattern Type: {}\nData: {}\nFrequency: {}\nLast Used: {}",
            usage.pattern_type,
            usage.data,
            usage.frequency,
            usage.last_used.format("%Y-%m-%d %H:%M:%S")
        ))
    }

    /// Convert vector document back to user feedback
    fn document_to_feedback(&self, doc: &VectorDocument) -> LearningResult<UserFeedback> {
        let custom_fields = &doc.metadata.custom_fields;

        let id = doc.id.clone();
        let user_id = custom_fields
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LearningError::InvalidFeedback("Missing user_id".to_string()))?
            .to_string();

        let content_id = custom_fields
            .get("content_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LearningError::InvalidFeedback("Missing content_id".to_string()))?
            .to_string();

        let feedback_type = custom_fields
            .get("feedback_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LearningError::InvalidFeedback("Missing feedback_type".to_string()))?
            .to_string();

        let score = custom_fields.get("score").and_then(|v| v.as_f64());

        let timestamp = custom_fields
            .get("timestamp")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or_else(|| LearningError::InvalidFeedback("Invalid timestamp".to_string()))?;

        // Extract text feedback from content if present
        let text_feedback = if doc.content.contains("Text: ") {
            doc.content
                .lines()
                .find(|line| line.starts_with("Text: "))
                .map(|line| line.strip_prefix("Text: ").unwrap_or("").to_string())
        } else {
            None
        };

        // Extract metadata
        let mut metadata = HashMap::new();
        for (key, value) in custom_fields {
            if key.starts_with("meta_") {
                let meta_key = key.strip_prefix("meta_").unwrap().to_string();
                metadata.insert(meta_key, value.clone());
            }
        }

        Ok(UserFeedback {
            id,
            user_id,
            content_id,
            feedback_type,
            score,
            text_feedback,
            timestamp,
            metadata,
        })
    }

    /// Convert vector document back to pattern data
    fn document_to_pattern(&self, doc: &VectorDocument) -> LearningResult<PatternData> {
        let custom_fields = &doc.metadata.custom_fields;

        let id = doc.id.clone();
        let pattern_type = custom_fields
            .get("pattern_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LearningError::PatternAnalysisError("Missing pattern_type".to_string()))?
            .to_string();

        let frequency = custom_fields
            .get("frequency")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| LearningError::PatternAnalysisError("Missing frequency".to_string()))?
            as u32;

        let success_rate = custom_fields
            .get("success_rate")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| {
                LearningError::PatternAnalysisError("Missing success_rate".to_string())
            })?;

        let first_seen = custom_fields
            .get("first_seen")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or_else(|| LearningError::PatternAnalysisError("Invalid first_seen".to_string()))?;

        let last_seen = custom_fields
            .get("last_seen")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or_else(|| LearningError::PatternAnalysisError("Invalid last_seen".to_string()))?;

        // Extract context
        let mut context = HashMap::new();
        for (key, value) in custom_fields {
            if key.starts_with("ctx_") {
                let ctx_key = key.strip_prefix("ctx_").unwrap().to_string();
                context.insert(ctx_key, value.clone());
            }
        }

        Ok(PatternData {
            id,
            pattern_type,
            frequency,
            success_rate,
            context,
            first_seen,
            last_seen,
        })
    }

    /// Convert vector document back to learning data
    fn document_to_learning(&self, doc: &VectorDocument) -> LearningResult<LearningData> {
        let custom_fields = &doc.metadata.custom_fields;

        let id = doc.id.clone();
        let learning_type = custom_fields
            .get("learning_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LearningError::InvalidOperation("Missing learning_type".to_string()))?
            .to_string();

        let source_data_id = custom_fields
            .get("source_data_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LearningError::InvalidOperation("Missing source_data_id".to_string()))?
            .to_string();

        let confidence_score = custom_fields
            .get("confidence_score")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| {
                LearningError::InvalidOperation("Missing confidence_score".to_string())
            })?;

        let created_at = custom_fields
            .get("created_at")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or_else(|| LearningError::InvalidOperation("Invalid created_at".to_string()))?;

        let expires_at = custom_fields
            .get("expires_at")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        // Extract insights from content
        let insights = if doc.content.contains("Insights: ") {
            doc.content
                .lines()
                .find(|line| line.starts_with("Insights: "))
                .map(|line| line.strip_prefix("Insights: ").unwrap_or(""))
                .map(|insights_str| insights_str.split("; ").map(|s| s.to_string()).collect())
                .unwrap_or_default()
        } else {
            vec![]
        };

        // Extract metadata
        let mut metadata = HashMap::new();
        for (key, value) in custom_fields {
            if key.starts_with("meta_") {
                let meta_key = key.strip_prefix("meta_").unwrap().to_string();
                metadata.insert(meta_key, value.clone());
            }
        }

        Ok(LearningData {
            id,
            learning_type,
            source_data_id,
            insights,
            confidence_score,
            created_at,
            expires_at,
            metadata,
        })
    }

    /// Convert vector document back to usage pattern
    fn document_to_usage(&self, doc: &VectorDocument) -> LearningResult<UsagePattern> {
        let custom_fields = &doc.metadata.custom_fields;

        let id = doc.id.clone();
        let pattern_type = custom_fields
            .get("pattern_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LearningError::PatternAnalysisError("Missing pattern_type".to_string()))?
            .to_string();

        let data = custom_fields
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LearningError::PatternAnalysisError("Missing data".to_string()))?
            .to_string();

        let frequency = custom_fields
            .get("frequency")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| LearningError::PatternAnalysisError("Missing frequency".to_string()))?
            as u32;

        let last_used = custom_fields
            .get("last_used")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or_else(|| LearningError::PatternAnalysisError("Invalid last_used".to_string()))?;

        // Extract context
        let mut context = HashMap::new();
        for (key, value) in custom_fields {
            if key.starts_with("ctx_") {
                let ctx_key = key.strip_prefix("ctx_").unwrap().to_string();
                context.insert(ctx_key, value.clone());
            }
        }

        Ok(UsagePattern {
            id,
            pattern_type,
            data,
            frequency,
            last_used,
            context,
        })
    }
}

#[async_trait]
impl LearningStorageService for VectorLearningStorage {
    #[instrument(skip(self, feedback))]
    async fn store_feedback(&self, feedback: &UserFeedback) -> LearningResult<UserFeedback> {
        debug!("Storing feedback: {}", feedback.id);

        let (content, metadata) = self.feedback_to_document(feedback)?;

        match self.vector_storage.store_document(&content, metadata).await {
            Ok(_doc) => {
                info!("Feedback stored successfully: {}", feedback.id);
                Ok(feedback.clone())
            }
            Err(e) => {
                error!("Failed to store feedback: {}", e);
                Err(LearningError::StorageError(e.to_string()))
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_feedback(&self, id: &str) -> LearningResult<Option<UserFeedback>> {
        debug!("Retrieving feedback: {}", id);

        match self.vector_storage.retrieve_by_id(id).await {
            Ok(Some(doc)) => {
                let feedback = self.document_to_feedback(&doc)?;
                Ok(Some(feedback))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to retrieve feedback: {}", e);
                Err(LearningError::StorageError(e.to_string()))
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_feedback_for_content(
        &self,
        content_id: &str,
    ) -> LearningResult<Vec<UserFeedback>> {
        debug!("Retrieving feedback for content: {}", content_id);

        let search_config = SearchConfig {
            limit: 100,
            threshold: None,
            collection: None,
            filters: vec![],
        };

        match self
            .vector_storage
            .retrieve_similar(&format!("Content: {content_id}"), search_config)
            .await
        {
            Ok(results) => {
                let mut feedback_list = Vec::new();
                for result in results {
                    if let Ok(feedback) = self.document_to_feedback(&result.document) {
                        if feedback.content_id == content_id {
                            feedback_list.push(feedback);
                        }
                    }
                }
                Ok(feedback_list)
            }
            Err(e) => {
                error!("Failed to retrieve feedback for content: {}", e);
                Err(LearningError::StorageError(e.to_string()))
            }
        }
    }

    #[instrument(skip(self, pattern))]
    async fn store_pattern(&self, pattern: &PatternData) -> LearningResult<PatternData> {
        debug!("Storing pattern: {}", pattern.id);

        let (content, metadata) = self.pattern_to_document(pattern)?;

        match self.vector_storage.store_document(&content, metadata).await {
            Ok(_doc) => {
                info!("Pattern stored successfully: {}", pattern.id);
                Ok(pattern.clone())
            }
            Err(e) => {
                error!("Failed to store pattern: {}", e);
                Err(LearningError::StorageError(e.to_string()))
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_patterns_by_type(&self, pattern_type: &str) -> LearningResult<Vec<PatternData>> {
        debug!("Retrieving patterns by type: {}", pattern_type);

        let search_config = SearchConfig {
            limit: 100,
            threshold: None,
            collection: None,
            filters: vec![],
        };

        match self
            .vector_storage
            .retrieve_similar(&format!("Pattern Type: {pattern_type}"), search_config)
            .await
        {
            Ok(results) => {
                let mut patterns = Vec::new();
                for result in results {
                    if let Ok(pattern) = self.document_to_pattern(&result.document) {
                        if pattern.pattern_type == pattern_type {
                            patterns.push(pattern);
                        }
                    }
                }
                Ok(patterns)
            }
            Err(e) => {
                error!("Failed to retrieve patterns: {}", e);
                Err(LearningError::StorageError(e.to_string()))
            }
        }
    }

    #[instrument(skip(self, data))]
    async fn store_learning_data(&self, data: &LearningData) -> LearningResult<LearningData> {
        debug!("Storing learning data: {}", data.id);

        let (content, metadata) = self.learning_to_document(data)?;

        match self.vector_storage.store_document(&content, metadata).await {
            Ok(_doc) => {
                info!("Learning data stored successfully: {}", data.id);
                Ok(data.clone())
            }
            Err(e) => {
                error!("Failed to store learning data: {}", e);
                Err(LearningError::StorageError(e.to_string()))
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_recent_learning_data(&self, limit: usize) -> LearningResult<Vec<LearningData>> {
        debug!("Retrieving recent learning data, limit: {}", limit);

        let search_config = SearchConfig {
            limit,
            threshold: None,
            collection: None,
            filters: vec![],
        };

        match self
            .vector_storage
            .retrieve_similar("Learning Type:", search_config)
            .await
        {
            Ok(results) => {
                let mut learning_data = Vec::new();
                for result in results {
                    if let Ok(data) = self.document_to_learning(&result.document) {
                        learning_data.push(data);
                    }
                }
                // Sort by creation time, most recent first
                learning_data.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                Ok(learning_data)
            }
            Err(e) => {
                error!("Failed to retrieve learning data: {}", e);
                Err(LearningError::StorageError(e.to_string()))
            }
        }
    }

    #[instrument(skip(self, pattern))]
    async fn store_usage_pattern(&self, pattern: &UsagePattern) -> LearningResult<UsagePattern> {
        debug!("Storing usage pattern: {}", pattern.id);

        let (content, metadata) = self.usage_to_document(pattern)?;

        match self.vector_storage.store_document(&content, metadata).await {
            Ok(_doc) => {
                info!("Usage pattern stored successfully: {}", pattern.id);
                Ok(pattern.clone())
            }
            Err(e) => {
                error!("Failed to store usage pattern: {}", e);
                Err(LearningError::StorageError(e.to_string()))
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_top_patterns(
        &self,
        pattern_type: &str,
        limit: usize,
    ) -> LearningResult<Vec<UsagePattern>> {
        debug!(
            "Retrieving top patterns for type: {}, limit: {}",
            pattern_type, limit
        );

        let search_config = SearchConfig {
            limit: limit * 2, // Get more to filter and sort
            threshold: None,
            collection: None,
            filters: vec![],
        };

        match self
            .vector_storage
            .retrieve_similar(&format!("Pattern Type: {pattern_type}"), search_config)
            .await
        {
            Ok(results) => {
                let mut patterns = Vec::new();
                for result in results {
                    if let Ok(pattern) = self.document_to_usage(&result.document) {
                        if pattern.pattern_type == pattern_type {
                            patterns.push(pattern);
                        }
                    }
                }
                // Sort by frequency, highest first
                patterns.sort_by(|a, b| b.frequency.cmp(&a.frequency));
                patterns.truncate(limit);
                Ok(patterns)
            }
            Err(e) => {
                error!("Failed to retrieve top patterns: {}", e);
                Err(LearningError::StorageError(e.to_string()))
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_trending_patterns(
        &self,
        pattern_type: &str,
        days: u32,
    ) -> LearningResult<Vec<UsagePattern>> {
        debug!(
            "Retrieving trending patterns for type: {}, days: {}",
            pattern_type, days
        );

        let cutoff_date = Utc::now() - Duration::days(days as i64);

        let search_config = SearchConfig {
            limit: 100,
            threshold: None,
            collection: None,
            filters: vec![],
        };

        match self
            .vector_storage
            .retrieve_similar(&format!("Pattern Type: {pattern_type}"), search_config)
            .await
        {
            Ok(results) => {
                let mut patterns = Vec::new();
                for result in results {
                    if let Ok(pattern) = self.document_to_usage(&result.document) {
                        if pattern.pattern_type == pattern_type && pattern.last_used >= cutoff_date
                        {
                            patterns.push(pattern);
                        }
                    }
                }
                // Sort by recency and frequency combined
                patterns.sort_by(|a, b| {
                    let a_score = a.frequency as f64
                        * (1.0 + (Utc::now() - a.last_used).num_hours() as f64 / 24.0);
                    let b_score = b.frequency as f64
                        * (1.0 + (Utc::now() - b.last_used).num_hours() as f64 / 24.0);
                    b_score
                        .partial_cmp(&a_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                Ok(patterns)
            }
            Err(e) => {
                error!("Failed to retrieve trending patterns: {}", e);
                Err(LearningError::StorageError(e.to_string()))
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_average_feedback_score(&self, content_id: &str) -> LearningResult<Option<f64>> {
        debug!(
            "Calculating average feedback score for content: {}",
            content_id
        );

        let feedback_list = self.get_feedback_for_content(content_id).await?;
        let scores: Vec<f64> = feedback_list.iter().filter_map(|f| f.score).collect();

        if scores.is_empty() {
            Ok(None)
        } else {
            let average = scores.iter().sum::<f64>() / scores.len() as f64;
            Ok(Some(average))
        }
    }

    #[instrument(skip(self))]
    async fn get_feedback_trend(
        &self,
        content_id: &str,
        days: u32,
    ) -> LearningResult<FeedbackTrend> {
        debug!(
            "Analyzing feedback trend for content: {}, days: {}",
            content_id, days
        );

        let cutoff_date = Utc::now() - Duration::days(days as i64);
        let feedback_list = self.get_feedback_for_content(content_id).await?;

        let recent_feedback: Vec<&UserFeedback> = feedback_list
            .iter()
            .filter(|f| f.timestamp >= cutoff_date)
            .collect();

        let total_feedback = recent_feedback.len();
        let scores: Vec<f64> = recent_feedback.iter().filter_map(|f| f.score).collect();

        let average_score = if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        // Calculate trend direction
        let trend_direction = if scores.len() >= 4 {
            let mid = scores.len() / 2;
            let recent_avg = scores[mid..].iter().sum::<f64>() / (scores.len() - mid) as f64;
            let older_avg = scores[..mid].iter().sum::<f64>() / mid as f64;
            recent_avg - older_avg
        } else {
            0.0
        };

        Ok(FeedbackTrend {
            content_id: content_id.to_string(),
            total_feedback,
            average_score,
            trend_direction,
        })
    }

    #[instrument(skip(self))]
    async fn get_recent_feedback(
        &self,
        content_id: &str,
        limit: usize,
    ) -> LearningResult<Vec<UserFeedback>> {
        debug!(
            "Retrieving recent feedback for content: {}, limit: {}",
            content_id, limit
        );

        let mut feedback_list = self.get_feedback_for_content(content_id).await?;
        feedback_list.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        feedback_list.truncate(limit);
        Ok(feedback_list)
    }

    #[instrument(skip(self))]
    async fn cleanup_old_data(&self, retention_days: u32) -> LearningResult<CleanupResult> {
        debug!("Cleaning up data older than {} days", retention_days);

        // Note: This is a simplified implementation
        // In a real system, you would implement proper cleanup logic
        // based on the vector database's capabilities

        let _cutoff_date = Utc::now() - Duration::days(retention_days as i64);

        warn!("Cleanup operation not fully implemented - this is a placeholder");

        Ok(CleanupResult {
            deleted_feedback: 0,
            deleted_patterns: 0,
            deleted_learning_data: 0,
            deleted_usage_patterns: 0,
            cleanup_date: Utc::now(),
        })
    }

    #[instrument(skip(self))]
    async fn initialize(&self) -> LearningResult<()> {
        info!("Initializing learning storage");

        match self.vector_storage.initialize().await {
            Ok(_) => {
                info!("Learning storage initialized successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to initialize learning storage: {}", e);
                Err(LearningError::StorageError(e.to_string()))
            }
        }
    }
}

/// Feedback trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackTrend {
    /// Content ID analyzed
    pub content_id: String,

    /// Total feedback entries
    pub total_feedback: usize,

    /// Average score
    pub average_score: f64,

    /// Trend direction (-1.0 to 1.0)
    pub trend_direction: f64,
}

/// Result of cleanup operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    /// Number of feedback entries deleted
    pub deleted_feedback: usize,

    /// Number of pattern entries deleted
    pub deleted_patterns: usize,

    /// Number of learning data entries deleted
    pub deleted_learning_data: usize,

    /// Number of usage patterns deleted
    pub deleted_usage_patterns: usize,

    /// When cleanup was performed
    pub cleanup_date: DateTime<Utc>,
}

/// Learning data with similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityLearningResult {
    /// The learning data
    pub learning_data: LearningData,
    /// Similarity score (0.0-1.0)
    pub similarity_score: f64,
}

/// Usage pattern with similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityUsagePattern {
    /// The usage pattern
    pub usage_pattern: UsagePattern,
    /// Similarity score (0.0-1.0)
    pub similarity_score: f64,
}

/// Embedding cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingCacheStats {
    /// Cache hit rate (0.0-1.0)
    pub hit_rate: f64,
    /// Total requests processed
    pub total_requests: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Average embedding generation time in milliseconds
    pub average_generation_time_ms: f64,
}

#[async_trait]
impl EnhancedLearningStorageService for VectorLearningStorage {
    #[instrument(skip(self, query))]
    async fn find_similar_insights(
        &self,
        query: &str,
        similarity_threshold: f64,
        max_results: usize,
    ) -> LearningResult<Vec<SimilarityLearningResult>> {
        debug!("Finding similar insights for query: {}", query);

        let search_config = SearchConfig {
            limit: max_results * 2, // Get more to filter by threshold
            threshold: Some(similarity_threshold),
            collection: Some(self.learning_collection.clone()),
            filters: vec![],
        };

        match self
            .vector_storage
            .retrieve_similar(query, search_config)
            .await
        {
            Ok(results) => {
                let mut similarity_results = Vec::new();

                for result in results {
                    if let Ok(learning_data) = self.document_to_learning(&result.document) {
                        if result.score >= similarity_threshold {
                            similarity_results.push(SimilarityLearningResult {
                                learning_data,
                                similarity_score: result.score,
                            });
                        }
                    }
                }

                // Sort by similarity score descending
                similarity_results.sort_by(|a, b| {
                    b.similarity_score
                        .partial_cmp(&a.similarity_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                similarity_results.truncate(max_results);

                info!("Found {} similar insights", similarity_results.len());
                Ok(similarity_results)
            }
            Err(e) => {
                error!("Failed to find similar insights: {}", e);
                Err(LearningError::StorageError(e.to_string()))
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_feedback_embedding(&self, feedback_id: &str) -> LearningResult<Option<Vec<f32>>> {
        debug!("Retrieving embedding for feedback: {}", feedback_id);

        match self.vector_storage.retrieve_by_id(feedback_id).await {
            Ok(Some(document)) => Ok(Some(document.embedding)),
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to retrieve feedback embedding: {}", e);
                Err(LearningError::StorageError(e.to_string()))
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_embedding_cache_stats(&self) -> LearningResult<EmbeddingCacheStats> {
        debug!("Retrieving embedding cache statistics");

        match self.vector_storage.get_stats().await {
            Ok(stats) => Ok(EmbeddingCacheStats {
                hit_rate: stats.embedding_cache_hit_rate,
                total_requests: stats.total_documents + stats.total_searches,
                cache_hits: (stats.embedding_cache_hit_rate
                    * (stats.total_documents + stats.total_searches) as f64)
                    as u64,
                cache_misses: ((1.0 - stats.embedding_cache_hit_rate)
                    * (stats.total_documents + stats.total_searches) as f64)
                    as u64,
                average_generation_time_ms: stats.avg_embedding_time_ms,
            }),
            Err(e) => {
                error!("Failed to retrieve cache stats: {}", e);
                Err(LearningError::StorageError(e.to_string()))
            }
        }
    }

    #[instrument(skip(self, data_batch))]
    async fn store_learning_data_batch(
        &self,
        data_batch: &[LearningData],
    ) -> LearningResult<BatchResult<LearningData>> {
        debug!("Storing learning data batch of {} items", data_batch.len());

        let mut documents = Vec::new();
        let mut learning_data_map = HashMap::new();

        for data in data_batch {
            let (content, metadata) = self.learning_to_document(data)?;
            documents.push((content, metadata));
            learning_data_map.insert(data.id.clone(), data.clone());
        }

        match self.vector_storage.store_documents(documents).await {
            Ok(batch_result) => {
                let mut successful = Vec::new();
                let mut failed = Vec::new();

                for success_doc in batch_result.successful {
                    if let Some(learning_data) = learning_data_map.get(&success_doc.id) {
                        successful.push(learning_data.clone());
                    }
                }

                for failure in batch_result.failed {
                    failed.push(BatchError {
                        index: failure.index,
                        document_id: failure.document_id,
                        error: failure.error,
                    });
                }

                info!(
                    "Batch stored: {} successful, {} failed",
                    successful.len(),
                    failed.len()
                );
                Ok(BatchResult {
                    successful,
                    failed,
                    total_attempted: data_batch.len(),
                })
            }
            Err(e) => {
                error!("Failed to store learning data batch: {}", e);
                Err(LearningError::StorageError(e.to_string()))
            }
        }
    }

    #[instrument(skip(self, ids))]
    async fn retrieve_learning_data_batch(
        &self,
        ids: &[String],
    ) -> LearningResult<BatchResult<LearningData>> {
        debug!("Retrieving learning data batch of {} items", ids.len());

        match self.vector_storage.retrieve_batch(ids.to_vec()).await {
            Ok(batch_result) => {
                let mut successful = Vec::new();
                let mut failed = Vec::new();

                for success_doc in batch_result.successful {
                    match self.document_to_learning(&success_doc) {
                        Ok(learning_data) => successful.push(learning_data),
                        Err(e) => failed.push(BatchError {
                            index: 0, // Index not available from document
                            document_id: Some(success_doc.id),
                            error: e.to_string(),
                        }),
                    }
                }

                for failure in batch_result.failed {
                    failed.push(BatchError {
                        index: failure.index,
                        document_id: failure.document_id,
                        error: failure.error,
                    });
                }

                info!(
                    "Batch retrieved: {} successful, {} failed",
                    successful.len(),
                    failed.len()
                );
                Ok(BatchResult {
                    successful,
                    failed,
                    total_attempted: ids.len(),
                })
            }
            Err(e) => {
                error!("Failed to retrieve learning data batch: {}", e);
                Err(LearningError::StorageError(e.to_string()))
            }
        }
    }

    #[instrument(skip(self, data_batch))]
    async fn update_learning_data_batch(
        &self,
        data_batch: &[LearningData],
    ) -> LearningResult<BatchResult<LearningData>> {
        debug!("Updating learning data batch of {} items", data_batch.len());

        // For updates, we need to store each individually to maintain IDs
        let mut successful = Vec::new();
        let mut failed = Vec::new();

        for (index, data) in data_batch.iter().enumerate() {
            match self.store_learning_data(data).await {
                Ok(updated_data) => successful.push(updated_data),
                Err(e) => failed.push(BatchError {
                    index,
                    document_id: Some(data.id.clone()),
                    error: e.to_string(),
                }),
            }
        }

        info!(
            "Batch updated: {} successful, {} failed",
            successful.len(),
            failed.len()
        );
        Ok(BatchResult {
            successful,
            failed,
            total_attempted: data_batch.len(),
        })
    }

    #[instrument(skip(self, query))]
    async fn find_similar_usage_patterns(
        &self,
        query: &str,
        similarity_threshold: f64,
        max_results: usize,
    ) -> LearningResult<Vec<SimilarityUsagePattern>> {
        debug!("Finding similar usage patterns for query: {}", query);

        let search_config = SearchConfig {
            limit: max_results * 2, // Get more to filter by threshold
            threshold: Some(similarity_threshold),
            collection: Some(self.usage_collection.clone()),
            filters: vec![],
        };

        match self
            .vector_storage
            .retrieve_similar(query, search_config)
            .await
        {
            Ok(results) => {
                let mut similarity_results = Vec::new();

                for result in results {
                    if let Ok(usage_pattern) = self.document_to_usage(&result.document) {
                        if result.score >= similarity_threshold {
                            similarity_results.push(SimilarityUsagePattern {
                                usage_pattern,
                                similarity_score: result.score,
                            });
                        }
                    }
                }

                // Sort by similarity score descending
                similarity_results.sort_by(|a, b| {
                    b.similarity_score
                        .partial_cmp(&a.similarity_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                similarity_results.truncate(max_results);

                info!("Found {} similar usage patterns", similarity_results.len());
                Ok(similarity_results)
            }
            Err(e) => {
                error!("Failed to find similar usage patterns: {}", e);
                Err(LearningError::StorageError(e.to_string()))
            }
        }
    }

    #[instrument(skip(self))]
    async fn cluster_usage_patterns_by_similarity(
        &self,
        pattern_type: &str,
        similarity_threshold: f64,
        max_clusters: usize,
    ) -> LearningResult<Vec<Vec<UsagePattern>>> {
        debug!(
            "Clustering usage patterns by similarity for type: {}",
            pattern_type
        );

        // First get all patterns of the specified type
        let all_patterns = self.get_top_patterns(pattern_type, 1000).await?; // Get a large number

        if all_patterns.is_empty() {
            return Ok(vec![]);
        }

        let mut clusters: Vec<Vec<UsagePattern>> = Vec::new();
        let mut processed: std::collections::HashSet<String> = std::collections::HashSet::new();

        for base_pattern in &all_patterns {
            if processed.contains(&base_pattern.id) {
                continue;
            }

            // Find similar patterns to this base pattern
            let similar_patterns = self
                .find_similar_usage_patterns(
                    &base_pattern.data,
                    similarity_threshold,
                    50, // Reasonable limit for cluster size
                )
                .await?;

            let mut cluster = vec![base_pattern.clone()];
            processed.insert(base_pattern.id.clone());

            for similar in similar_patterns {
                if !processed.contains(&similar.usage_pattern.id)
                    && similar.usage_pattern.pattern_type == pattern_type
                {
                    cluster.push(similar.usage_pattern);
                    processed.insert(cluster.last().unwrap().id.clone());
                }
            }

            if cluster.len() > 1 {
                clusters.push(cluster);
            }

            if clusters.len() >= max_clusters {
                break;
            }
        }

        info!("Created {} pattern clusters", clusters.len());
        Ok(clusters)
    }

    #[instrument(skip(self))]
    async fn get_learning_data(&self, id: &str) -> LearningResult<Option<LearningData>> {
        debug!("Retrieving learning data: {}", id);

        match self.vector_storage.retrieve_by_id(id).await {
            Ok(Some(document)) => {
                let learning_data = self.document_to_learning(&document)?;
                Ok(Some(learning_data))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to retrieve learning data: {}", e);
                Err(LearningError::StorageError(e.to_string()))
            }
        }
    }

    #[instrument(skip(self))]
    async fn cleanup_expired_learning_data(&self) -> LearningResult<CleanupResult> {
        debug!("Cleaning up expired learning data");

        let current_time = Utc::now();
        let recent_data = self.get_recent_learning_data(10000).await?; // Get large set for cleanup

        let mut expired_ids = Vec::new();

        for data in recent_data {
            if let Some(expires_at) = data.expires_at {
                if expires_at < current_time {
                    expired_ids.push(data.id);
                }
            }
        }

        let mut deleted_count = 0;
        if !expired_ids.is_empty() {
            match self.vector_storage.delete_batch(expired_ids.clone()).await {
                Ok(batch_result) => {
                    deleted_count = batch_result.successful.len();
                    info!("Deleted {} expired learning data entries", deleted_count);
                }
                Err(e) => {
                    error!("Failed to delete expired learning data: {}", e);
                    return Err(LearningError::StorageError(e.to_string()));
                }
            }
        }

        Ok(CleanupResult {
            deleted_feedback: 0,
            deleted_patterns: 0,
            deleted_learning_data: deleted_count,
            deleted_usage_patterns: 0,
            cleanup_date: current_time,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_trend_calculation() {
        let trend = FeedbackTrend {
            content_id: "test_content".to_string(),
            total_feedback: 10,
            average_score: 0.85,
            trend_direction: 0.05,
        };

        assert_eq!(trend.content_id, "test_content");
        assert_eq!(trend.total_feedback, 10);
        assert_eq!(trend.average_score, 0.85);
        assert_eq!(trend.trend_direction, 0.05);
    }

    #[test]
    fn test_cleanup_result_structure() {
        let cleanup = CleanupResult {
            deleted_feedback: 5,
            deleted_patterns: 3,
            deleted_learning_data: 2,
            deleted_usage_patterns: 8,
            cleanup_date: Utc::now(),
        };

        assert_eq!(cleanup.deleted_feedback, 5);
        assert_eq!(cleanup.deleted_patterns, 3);
        assert_eq!(cleanup.deleted_learning_data, 2);
        assert_eq!(cleanup.deleted_usage_patterns, 8);
    }
}
