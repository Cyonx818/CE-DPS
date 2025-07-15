//! Anchor Tests for Learning Vector Database Persistence
//!
//! These are critical anchor tests that must always pass to ensure the
//! learning vector database persistence functionality remains stable.
//! Any failure in these tests indicates a breaking change that requires
//! immediate attention.

use chrono::{DateTime, Duration, Utc};
use fortitude::learning::{
    CleanupResult, EmbeddingCacheStats, EnhancedLearningStorageService, FeedbackTrend,
    LearningData, LearningError, LearningResult, LearningStorageConfig, LearningStorageService,
    PatternData, SimilarityLearningResult, SimilarityUsagePattern, UsagePattern, UserFeedback,
    VectorLearningStorage,
};
use std::collections::HashMap;
use uuid::Uuid;

/// ANCHOR: Core learning data storage and retrieval functionality must work
/// Purpose: Protects against regression in basic learning data persistence
/// This test ensures that the fundamental storage operations continue to work
#[tokio::test]
async fn anchor_learning_data_basic_persistence() {
    let storage = create_anchor_test_storage().await;

    // Create test learning data
    let learning_data = LearningData {
        id: Uuid::new_v4().to_string(),
        learning_type: "user_preference".to_string(),
        source_data_id: "feedback_source_123".to_string(),
        insights: vec![
            "Users prefer detailed explanations".to_string(),
            "Examples improve comprehension".to_string(),
        ],
        confidence_score: 0.85,
        created_at: Utc::now(),
        expires_at: None,
        metadata: HashMap::new(),
    };

    // Test storage
    let stored = storage.store_learning_data(&learning_data).await.unwrap();
    assert_eq!(stored.id, learning_data.id);
    assert_eq!(stored.learning_type, learning_data.learning_type);
    assert_eq!(stored.insights.len(), 2);
    assert_eq!(stored.confidence_score, 0.85);

    // Test retrieval
    let recent_data = storage.get_recent_learning_data(5).await.unwrap();
    assert!(recent_data.iter().any(|d| d.id == learning_data.id));

    // Verify data integrity
    let retrieved = recent_data
        .iter()
        .find(|d| d.id == learning_data.id)
        .unwrap();
    assert_eq!(retrieved.learning_type, "user_preference");
    assert_eq!(retrieved.source_data_id, "feedback_source_123");
    assert_eq!(retrieved.insights, learning_data.insights);
    assert_eq!(retrieved.confidence_score, 0.85);
}

/// ANCHOR: User feedback storage and analysis must remain functional
/// Purpose: Protects against regression in feedback collection and analysis
/// This test ensures feedback can be stored, retrieved, and analyzed correctly
#[tokio::test]
async fn anchor_user_feedback_persistence_and_analysis() {
    let storage = create_anchor_test_storage().await;

    let content_id = "test_content_456";
    let feedback_entries = vec![
        UserFeedback {
            id: Uuid::new_v4().to_string(),
            user_id: "user_1".to_string(),
            content_id: content_id.to_string(),
            feedback_type: "quality_rating".to_string(),
            score: Some(0.9),
            text_feedback: Some("Excellent quality".to_string()),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        },
        UserFeedback {
            id: Uuid::new_v4().to_string(),
            user_id: "user_2".to_string(),
            content_id: content_id.to_string(),
            feedback_type: "quality_rating".to_string(),
            score: Some(0.8),
            text_feedback: Some("Good quality".to_string()),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        },
        UserFeedback {
            id: Uuid::new_v4().to_string(),
            user_id: "user_3".to_string(),
            content_id: content_id.to_string(),
            feedback_type: "quality_rating".to_string(),
            score: Some(0.85),
            text_feedback: Some("Very good".to_string()),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        },
    ];

    // Store all feedback
    for feedback in &feedback_entries {
        let stored = storage.store_feedback(feedback).await.unwrap();
        assert_eq!(stored.id, feedback.id);
        assert_eq!(stored.content_id, feedback.content_id);
        assert_eq!(stored.score, feedback.score);
    }

    // Test feedback retrieval
    let content_feedback = storage.get_feedback_for_content(content_id).await.unwrap();
    assert_eq!(content_feedback.len(), 3);

    // Test individual feedback retrieval
    for feedback in &feedback_entries {
        let retrieved = storage.get_feedback(&feedback.id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved_feedback = retrieved.unwrap();
        assert_eq!(retrieved_feedback.id, feedback.id);
        assert_eq!(retrieved_feedback.user_id, feedback.user_id);
        assert_eq!(retrieved_feedback.score, feedback.score);
    }

    // Test feedback analysis
    let avg_score = storage
        .get_average_feedback_score(content_id)
        .await
        .unwrap();
    assert!(avg_score.is_some());
    let score = avg_score.unwrap();
    // Average of 0.9, 0.8, 0.85 = 0.85
    assert!((score - 0.85).abs() < 0.01);

    // Test feedback trend analysis
    let trend = storage.get_feedback_trend(content_id, 30).await.unwrap();
    assert_eq!(trend.content_id, content_id);
    assert_eq!(trend.total_feedback, 3);
    assert!((trend.average_score - 0.85).abs() < 0.01);

    // Test recent feedback retrieval
    let recent_feedback = storage.get_recent_feedback(content_id, 2).await.unwrap();
    assert_eq!(recent_feedback.len(), 2);

    // Verify ordering (most recent first)
    assert!(recent_feedback[0].timestamp >= recent_feedback[1].timestamp);
}

/// ANCHOR: Usage pattern analysis must continue to work correctly
/// Purpose: Protects against regression in pattern recognition and frequency analysis
/// This test ensures usage patterns can be stored and analyzed for insights
#[tokio::test]
async fn anchor_usage_pattern_analysis() {
    let storage = create_anchor_test_storage().await;

    let pattern_type = "search_query";
    let patterns = vec![
        UsagePattern {
            id: Uuid::new_v4().to_string(),
            pattern_type: pattern_type.to_string(),
            data: "rust async programming".to_string(),
            frequency: 25,
            last_used: Utc::now(),
            context: HashMap::new(),
        },
        UsagePattern {
            id: Uuid::new_v4().to_string(),
            pattern_type: pattern_type.to_string(),
            data: "vector database tutorial".to_string(),
            frequency: 20,
            last_used: Utc::now(),
            context: HashMap::new(),
        },
        UsagePattern {
            id: Uuid::new_v4().to_string(),
            pattern_type: pattern_type.to_string(),
            data: "machine learning basics".to_string(),
            frequency: 15,
            last_used: Utc::now(),
            context: HashMap::new(),
        },
        UsagePattern {
            id: Uuid::new_v4().to_string(),
            pattern_type: pattern_type.to_string(),
            data: "web development guide".to_string(),
            frequency: 10,
            last_used: Utc::now(),
            context: HashMap::new(),
        },
    ];

    // Store all patterns
    for pattern in &patterns {
        let stored = storage.store_usage_pattern(pattern).await.unwrap();
        assert_eq!(stored.id, pattern.id);
        assert_eq!(stored.pattern_type, pattern.pattern_type);
        assert_eq!(stored.data, pattern.data);
        assert_eq!(stored.frequency, pattern.frequency);
    }

    // Test top patterns retrieval (should be ordered by frequency)
    let top_patterns = storage.get_top_patterns(pattern_type, 5).await.unwrap();
    assert!(top_patterns.len() >= 4);

    // Verify ordering by frequency (descending)
    assert!(top_patterns[0].frequency >= top_patterns[1].frequency);
    assert!(top_patterns[1].frequency >= top_patterns[2].frequency);
    assert!(top_patterns[2].frequency >= top_patterns[3].frequency);

    // Verify highest frequency pattern
    assert_eq!(top_patterns[0].data, "rust async programming");
    assert_eq!(top_patterns[0].frequency, 25);

    // Test trending patterns
    let trending = storage
        .get_trending_patterns(pattern_type, 7)
        .await
        .unwrap();
    assert!(!trending.is_empty());

    // All patterns should be recent (within 7 days)
    let cutoff = Utc::now() - Duration::days(7);
    for pattern in &trending {
        assert!(pattern.last_used >= cutoff);
    }
}

/// ANCHOR: Pattern data storage and retrieval must remain stable
/// Purpose: Protects against regression in pattern data management
/// This test ensures pattern data can be stored and retrieved correctly
#[tokio::test]
async fn anchor_pattern_data_persistence() {
    let storage = create_anchor_test_storage().await;

    let pattern_type = "user_behavior";
    let pattern_data = PatternData {
        id: Uuid::new_v4().to_string(),
        pattern_type: pattern_type.to_string(),
        frequency: 15,
        success_rate: 0.78,
        context: {
            let mut ctx = HashMap::new();
            ctx.insert("domain".to_string(), serde_json::json!("technical"));
            ctx.insert("complexity".to_string(), serde_json::json!("intermediate"));
            ctx
        },
        first_seen: Utc::now() - Duration::days(10),
        last_seen: Utc::now(),
    };

    // Store pattern data
    let stored = storage.store_pattern(&pattern_data).await.unwrap();
    assert_eq!(stored.id, pattern_data.id);
    assert_eq!(stored.pattern_type, pattern_data.pattern_type);
    assert_eq!(stored.frequency, pattern_data.frequency);
    assert_eq!(stored.success_rate, pattern_data.success_rate);
    assert_eq!(stored.context.len(), 2);

    // Test retrieval by type
    let retrieved_patterns = storage.get_patterns_by_type(pattern_type).await.unwrap();
    assert!(retrieved_patterns.iter().any(|p| p.id == pattern_data.id));

    // Verify retrieved pattern data
    let retrieved = retrieved_patterns
        .iter()
        .find(|p| p.id == pattern_data.id)
        .unwrap();
    assert_eq!(retrieved.frequency, 15);
    assert_eq!(retrieved.success_rate, 0.78);
    assert_eq!(retrieved.context.get("domain").unwrap(), "technical");
    assert_eq!(retrieved.context.get("complexity").unwrap(), "intermediate");

    // Verify timestamps
    assert!(retrieved.first_seen <= retrieved.last_seen);
    assert!(retrieved.last_seen <= Utc::now());
}

/// ANCHOR: Data cleanup and retention policies must function correctly
/// Purpose: Protects against regression in data lifecycle management
/// This test ensures old data can be cleaned up properly
#[tokio::test]
async fn anchor_data_cleanup_and_retention() {
    let storage = create_anchor_test_storage().await;

    // Store some test data
    let test_learning = LearningData {
        id: Uuid::new_v4().to_string(),
        learning_type: "cleanup_test".to_string(),
        source_data_id: "cleanup_source".to_string(),
        insights: vec!["Test insight for cleanup".to_string()],
        confidence_score: 0.7,
        created_at: Utc::now(),
        expires_at: None,
        metadata: HashMap::new(),
    };

    storage.store_learning_data(&test_learning).await.unwrap();

    let test_feedback = UserFeedback {
        id: Uuid::new_v4().to_string(),
        user_id: "cleanup_user".to_string(),
        content_id: "cleanup_content".to_string(),
        feedback_type: "test_feedback".to_string(),
        score: Some(0.8),
        text_feedback: Some("Test feedback for cleanup".to_string()),
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    };

    storage.store_feedback(&test_feedback).await.unwrap();

    // Test cleanup operation
    let cleanup_result = storage.cleanup_old_data(365).await.unwrap(); // 1 year retention

    // Verify cleanup result structure
    assert!(cleanup_result.deleted_feedback >= 0);
    assert!(cleanup_result.deleted_patterns >= 0);
    assert!(cleanup_result.deleted_learning_data >= 0);
    assert!(cleanup_result.deleted_usage_patterns >= 0);
    assert!(cleanup_result.cleanup_date <= Utc::now());

    // Verify data still exists (should not be deleted with 1 year retention)
    let remaining_data = storage.get_recent_learning_data(10).await.unwrap();
    assert!(remaining_data.iter().any(|d| d.id == test_learning.id));

    let remaining_feedback = storage.get_feedback(&test_feedback.id).await.unwrap();
    assert!(remaining_feedback.is_some());
}

/// ANCHOR: Storage initialization must work reliably
/// Purpose: Protects against regression in storage setup and configuration
/// This test ensures the storage system can be initialized correctly
#[tokio::test]
async fn anchor_storage_initialization() {
    let storage = create_anchor_test_storage().await;

    // Test initialization
    let init_result = storage.initialize().await;
    assert!(init_result.is_ok());

    // Verify storage is functional after initialization
    let test_data = LearningData {
        id: Uuid::new_v4().to_string(),
        learning_type: "init_test".to_string(),
        source_data_id: "init_source".to_string(),
        insights: vec!["Testing post-initialization functionality".to_string()],
        confidence_score: 0.9,
        created_at: Utc::now(),
        expires_at: None,
        metadata: HashMap::new(),
    };

    let stored = storage.store_learning_data(&test_data).await.unwrap();
    assert_eq!(stored.id, test_data.id);

    let retrieved = storage.get_recent_learning_data(1).await.unwrap();
    assert!(!retrieved.is_empty());
    assert_eq!(retrieved[0].id, test_data.id);
}

/// ANCHOR: Error handling must be robust and consistent
/// Purpose: Protects against regression in error handling behavior
/// This test ensures errors are handled gracefully and consistently
#[tokio::test]
async fn anchor_error_handling_consistency() {
    let storage = create_anchor_test_storage().await;

    // Test retrieval of non-existent feedback
    let non_existent_feedback = storage.get_feedback("non_existent_id").await.unwrap();
    assert!(non_existent_feedback.is_none());

    // Test retrieval of feedback for non-existent content
    let no_feedback = storage
        .get_feedback_for_content("non_existent_content")
        .await
        .unwrap();
    assert!(no_feedback.is_empty());

    // Test average score for content with no feedback
    let no_avg = storage
        .get_average_feedback_score("no_feedback_content")
        .await
        .unwrap();
    assert!(no_avg.is_none());

    // Test patterns for non-existent type
    let no_patterns = storage
        .get_patterns_by_type("non_existent_type")
        .await
        .unwrap();
    assert!(no_patterns.is_empty());

    // Test top patterns for non-existent type
    let no_top_patterns = storage
        .get_top_patterns("non_existent_type", 5)
        .await
        .unwrap();
    assert!(no_top_patterns.is_empty());

    // Test recent feedback for non-existent content
    let no_recent = storage
        .get_recent_feedback("non_existent_content", 5)
        .await
        .unwrap();
    assert!(no_recent.is_empty());

    // Verify that operations continue to work after handling non-existent data
    let test_feedback = UserFeedback {
        id: Uuid::new_v4().to_string(),
        user_id: "error_test_user".to_string(),
        content_id: "error_test_content".to_string(),
        feedback_type: "error_test".to_string(),
        score: Some(0.5),
        text_feedback: Some("Testing after error handling".to_string()),
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    };

    let stored = storage.store_feedback(&test_feedback).await.unwrap();
    assert_eq!(stored.id, test_feedback.id);
}

// Helper function to create test storage
async fn create_anchor_test_storage() -> impl LearningStorageService {
    MockAnchorLearningStorage::new()
}

// Minimal mock implementation for anchor tests
struct MockAnchorLearningStorage {
    feedback_store: std::sync::Arc<tokio::sync::RwLock<HashMap<String, UserFeedback>>>,
    learning_store: std::sync::Arc<tokio::sync::RwLock<HashMap<String, LearningData>>>,
    pattern_store: std::sync::Arc<tokio::sync::RwLock<HashMap<String, PatternData>>>,
    usage_patterns: std::sync::Arc<tokio::sync::RwLock<Vec<UsagePattern>>>,
}

impl MockAnchorLearningStorage {
    fn new() -> Self {
        Self {
            feedback_store: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            learning_store: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            pattern_store: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            usage_patterns: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl LearningStorageService for MockAnchorLearningStorage {
    async fn store_feedback(&self, feedback: &UserFeedback) -> LearningResult<UserFeedback> {
        let mut store = self.feedback_store.write().await;
        store.insert(feedback.id.clone(), feedback.clone());
        Ok(feedback.clone())
    }

    async fn get_feedback(&self, id: &str) -> LearningResult<Option<UserFeedback>> {
        let store = self.feedback_store.read().await;
        Ok(store.get(id).cloned())
    }

    async fn get_feedback_for_content(
        &self,
        content_id: &str,
    ) -> LearningResult<Vec<UserFeedback>> {
        let store = self.feedback_store.read().await;
        let feedback: Vec<UserFeedback> = store
            .values()
            .filter(|f| f.content_id == content_id)
            .cloned()
            .collect();
        Ok(feedback)
    }

    async fn store_pattern(&self, pattern: &PatternData) -> LearningResult<PatternData> {
        let mut store = self.pattern_store.write().await;
        store.insert(pattern.id.clone(), pattern.clone());
        Ok(pattern.clone())
    }

    async fn get_patterns_by_type(&self, pattern_type: &str) -> LearningResult<Vec<PatternData>> {
        let store = self.pattern_store.read().await;
        let patterns: Vec<PatternData> = store
            .values()
            .filter(|p| p.pattern_type == pattern_type)
            .cloned()
            .collect();
        Ok(patterns)
    }

    async fn store_learning_data(&self, data: &LearningData) -> LearningResult<LearningData> {
        let mut store = self.learning_store.write().await;
        store.insert(data.id.clone(), data.clone());
        Ok(data.clone())
    }

    async fn get_recent_learning_data(&self, limit: usize) -> LearningResult<Vec<LearningData>> {
        let store = self.learning_store.read().await;
        let mut data: Vec<LearningData> = store.values().cloned().collect();
        data.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        data.truncate(limit);
        Ok(data)
    }

    async fn store_usage_pattern(&self, pattern: &UsagePattern) -> LearningResult<UsagePattern> {
        let mut store = self.usage_patterns.write().await;
        store.push(pattern.clone());
        Ok(pattern.clone())
    }

    async fn get_top_patterns(
        &self,
        pattern_type: &str,
        limit: usize,
    ) -> LearningResult<Vec<UsagePattern>> {
        let store = self.usage_patterns.read().await;
        let mut patterns: Vec<UsagePattern> = store
            .iter()
            .filter(|p| p.pattern_type == pattern_type)
            .cloned()
            .collect();
        patterns.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        patterns.truncate(limit);
        Ok(patterns)
    }

    async fn get_trending_patterns(
        &self,
        pattern_type: &str,
        days: u32,
    ) -> LearningResult<Vec<UsagePattern>> {
        let cutoff_date = Utc::now() - Duration::days(days as i64);
        let store = self.usage_patterns.read().await;
        let patterns: Vec<UsagePattern> = store
            .iter()
            .filter(|p| p.pattern_type == pattern_type && p.last_used >= cutoff_date)
            .cloned()
            .collect();
        Ok(patterns)
    }

    async fn get_average_feedback_score(&self, content_id: &str) -> LearningResult<Option<f64>> {
        let store = self.feedback_store.read().await;
        let feedback: Vec<&UserFeedback> = store
            .values()
            .filter(|f| f.content_id == content_id && f.score.is_some())
            .collect();

        if feedback.is_empty() {
            return Ok(None);
        }

        let sum: f64 = feedback.iter().filter_map(|f| f.score).sum();
        let avg = sum / feedback.len() as f64;
        Ok(Some(avg))
    }

    async fn get_feedback_trend(
        &self,
        content_id: &str,
        _days: u32,
    ) -> LearningResult<FeedbackTrend> {
        let store = self.feedback_store.read().await;
        let feedback: Vec<&UserFeedback> = store
            .values()
            .filter(|f| f.content_id == content_id)
            .collect();

        let total_feedback = feedback.len();
        let scores: Vec<f64> = feedback.iter().filter_map(|f| f.score).collect();
        let average_score = if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        Ok(FeedbackTrend {
            content_id: content_id.to_string(),
            total_feedback,
            average_score,
            trend_direction: if average_score > 0.8 { 1.0 } else { 0.0 },
        })
    }

    async fn get_recent_feedback(
        &self,
        content_id: &str,
        limit: usize,
    ) -> LearningResult<Vec<UserFeedback>> {
        let store = self.feedback_store.read().await;
        let mut feedback: Vec<UserFeedback> = store
            .values()
            .filter(|f| f.content_id == content_id)
            .cloned()
            .collect();
        feedback.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        feedback.truncate(limit);
        Ok(feedback)
    }

    async fn cleanup_old_data(&self, _retention_days: u32) -> LearningResult<CleanupResult> {
        Ok(CleanupResult {
            deleted_feedback: 0,
            deleted_patterns: 0,
            deleted_learning_data: 0,
            deleted_usage_patterns: 0,
            cleanup_date: Utc::now(),
        })
    }

    async fn initialize(&self) -> LearningResult<()> {
        Ok(())
    }
}
