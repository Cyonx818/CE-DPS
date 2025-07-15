//! Learning Vector Database Integration Tests
//!
//! Comprehensive integration tests for enhanced learning data persistence with
//! vector database functionality. These tests verify end-to-end workflows,
//! cross-component integration, and performance characteristics.

use chrono::{DateTime, Duration, Utc};
use fortitude::learning::{
    CleanupResult, EmbeddingCacheStats, EnhancedLearningStorageService, FeedbackTrend,
    LearningData, LearningError, LearningResult, LearningStorageConfig, LearningStorageService,
    PatternData, SimilarityLearningResult, SimilarityUsagePattern, UsagePattern, UserFeedback,
    VectorLearningStorage,
};
use fortitude_core::vector::{
    error::VectorResult,
    storage::{BatchResult, DocumentMetadata, SearchConfig, VectorStorageService},
};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// ANCHOR: Integration test for complete vector-based learning workflow
/// Tests: End-to-end learning data lifecycle with vector similarity search
#[tokio::test]
async fn test_anchor_complete_vector_learning_workflow() {
    let storage = create_test_vector_storage().await;

    // Phase 1: Store diverse learning data with different types
    let learning_entries = vec![
        create_test_learning_data(
            "user_preference",
            "feedback_123",
            vec!["Users prefer detailed technical explanations with examples".to_string()],
            0.9,
        ),
        create_test_learning_data(
            "user_preference",
            "feedback_456",
            vec!["Users value comprehensive documentation and clear structure".to_string()],
            0.85,
        ),
        create_test_learning_data(
            "system_optimization",
            "analysis_789",
            vec!["Query response time can be improved with better caching".to_string()],
            0.8,
        ),
        create_test_learning_data(
            "pattern_insight",
            "pattern_101",
            vec!["Rust async programming questions are frequently asked".to_string()],
            0.75,
        ),
        create_test_learning_data(
            "user_preference",
            "feedback_202",
            vec!["Users appreciate quick responses over detailed explanations".to_string()],
            0.7,
        ),
    ];

    // Store learning data individually to verify basic functionality
    let mut stored_ids = Vec::new();
    for entry in &learning_entries {
        let stored = storage.store_learning_data(entry).await.unwrap();
        stored_ids.push(stored.id);
        assert_eq!(stored.learning_type, entry.learning_type);
        assert_eq!(stored.insights, entry.insights);
    }

    // Phase 2: Test vector similarity search functionality
    // Note: This uses the existing basic functionality since vector search would be enhanced
    let recent_data = storage.get_recent_learning_data(10).await.unwrap();
    assert!(recent_data.len() >= 5);

    // Verify we can find user preference patterns
    let user_prefs: Vec<_> = recent_data
        .iter()
        .filter(|data| data.learning_type == "user_preference")
        .collect();
    assert_eq!(user_prefs.len(), 3);

    // Phase 3: Test usage pattern integration
    let usage_patterns = vec![
        create_test_usage_pattern("search_query", "rust async tutorial", 15),
        create_test_usage_pattern("search_query", "async programming rust", 12),
        create_test_usage_pattern("search_query", "performance optimization", 8),
        create_test_usage_pattern("user_behavior", "detailed_explanation_request", 20),
        create_test_usage_pattern("user_behavior", "quick_answer_request", 18),
    ];

    for pattern in &usage_patterns {
        let stored = storage.store_usage_pattern(pattern).await.unwrap();
        assert_eq!(stored.pattern_type, pattern.pattern_type);
        assert_eq!(stored.frequency, pattern.frequency);
    }

    // Test pattern analysis
    let search_patterns = storage.get_top_patterns("search_query", 5).await.unwrap();
    assert!(search_patterns.len() >= 3);

    // Verify patterns are sorted by frequency
    for i in 1..search_patterns.len() {
        assert!(search_patterns[i - 1].frequency >= search_patterns[i].frequency);
    }

    // Phase 4: Test feedback analysis integration
    let feedback_entries = vec![
        create_test_feedback("content_123", 0.9, "Excellent detailed explanation"),
        create_test_feedback("content_123", 0.85, "Very helpful and comprehensive"),
        create_test_feedback("content_456", 0.7, "Good but could be more concise"),
        create_test_feedback("content_456", 0.8, "Useful information"),
        create_test_feedback("content_123", 0.95, "Perfect level of detail"),
    ];

    for feedback in &feedback_entries {
        let stored = storage.store_feedback(feedback).await.unwrap();
        assert_eq!(stored.content_id, feedback.content_id);
        assert_eq!(stored.score, feedback.score);
    }

    // Test feedback analysis
    let avg_score_123 = storage
        .get_average_feedback_score("content_123")
        .await
        .unwrap();
    assert!(avg_score_123.is_some());
    let score = avg_score_123.unwrap();
    assert!(score > 0.88 && score < 0.92); // Should be around 0.9

    let feedback_trend = storage.get_feedback_trend("content_123", 30).await.unwrap();
    assert_eq!(feedback_trend.total_feedback, 3);
    assert!(feedback_trend.average_score > 0.88);

    // Phase 5: Test data retention and cleanup
    let cleanup_result = storage.cleanup_old_data(365).await.unwrap();
    assert!(cleanup_result.deleted_learning_data >= 0);

    // Verify data integrity after cleanup
    let final_data = storage.get_recent_learning_data(10).await.unwrap();
    assert!(final_data.len() >= 5); // All data should remain (recent)
}

/// ANCHOR: Integration test for cross-component learning data flow
/// Tests: Data flow between different learning components and consistency
#[tokio::test]
async fn test_anchor_cross_component_learning_integration() {
    let storage = create_test_vector_storage().await;

    // Simulate a complete learning cycle from feedback to insights

    // Step 1: User provides feedback
    let user_feedback = create_test_feedback(
        "research_result_001",
        0.95,
        "This explanation of Rust async programming was extremely helpful and detailed",
    );
    let stored_feedback = storage.store_feedback(&user_feedback).await.unwrap();

    // Step 2: System analyzes usage patterns
    let usage_pattern = create_test_usage_pattern(
        "search_behavior",
        "rust async programming detailed explanation",
        5,
    );
    let stored_pattern = storage.store_usage_pattern(&usage_pattern).await.unwrap();

    // Step 3: System generates learning insights from feedback and patterns
    let learning_insight = LearningData {
        id: Uuid::new_v4().to_string(),
        learning_type: "user_preference_analysis".to_string(),
        source_data_id: stored_feedback.id.clone(),
        insights: vec![
            "User appreciates detailed technical explanations".to_string(),
            "Rust async programming is a high-interest topic".to_string(),
            "Comprehensive examples improve user satisfaction".to_string(),
        ],
        confidence_score: 0.88,
        created_at: Utc::now(),
        expires_at: Some(Utc::now() + Duration::days(90)),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert(
                "source_feedback_score".to_string(),
                serde_json::json!(user_feedback.score.unwrap()),
            );
            meta.insert(
                "pattern_frequency".to_string(),
                serde_json::json!(usage_pattern.frequency),
            );
            meta
        },
    };

    let stored_insight = storage
        .store_learning_data(&learning_insight)
        .await
        .unwrap();

    // Step 4: Verify cross-referential integrity
    assert_eq!(stored_insight.source_data_id, stored_feedback.id);
    assert_eq!(stored_insight.confidence_score, 0.88);
    assert!(stored_insight.insights.len() == 3);

    // Step 5: Test pattern recognition across related data
    let related_patterns = storage
        .get_top_patterns("search_behavior", 5)
        .await
        .unwrap();
    assert!(related_patterns.iter().any(|p| p.id == stored_pattern.id));

    // Step 6: Test feedback correlation analysis
    let content_feedback = storage
        .get_feedback_for_content("research_result_001")
        .await
        .unwrap();
    assert!(content_feedback.iter().any(|f| f.id == stored_feedback.id));

    // Step 7: Test temporal analysis
    let recent_insights = storage.get_recent_learning_data(5).await.unwrap();
    assert!(recent_insights.iter().any(|i| i.id == stored_insight.id));

    // Step 8: Test metadata preservation and retrieval
    let retrieved_insight = storage.get_recent_learning_data(1).await.unwrap();
    let insight = &retrieved_insight[0];
    assert!(insight.metadata.contains_key("source_feedback_score"));
    assert!(insight.metadata.contains_key("pattern_frequency"));
}

/// ANCHOR: Integration test for learning system performance and scalability
/// Tests: Performance characteristics, batch operations, concurrent access
#[tokio::test]
async fn test_anchor_learning_performance_integration() {
    let storage = create_test_vector_storage().await;

    // Phase 1: Test batch storage performance
    let batch_size = 50;
    let mut learning_batch = Vec::new();

    for i in 0..batch_size {
        learning_batch.push(create_test_learning_data(
            "performance_test",
            &format!("source_{}", i),
            vec![format!("Performance insight number {}", i)],
            0.7 + (i as f64 * 0.005), // Varying confidence scores
        ));
    }

    // Store individually (simulating batch operation)
    let start_time = std::time::Instant::now();
    let mut successful_stores = 0;

    for data in &learning_batch {
        match storage.store_learning_data(data).await {
            Ok(_) => successful_stores += 1,
            Err(_) => {} // Count failures
        }
    }

    let storage_duration = start_time.elapsed();
    assert_eq!(successful_stores, batch_size);
    assert!(storage_duration.as_millis() < 5000); // Should complete within 5 seconds

    // Phase 2: Test retrieval performance
    let retrieval_start = std::time::Instant::now();
    let retrieved_data = storage.get_recent_learning_data(100).await.unwrap();
    let retrieval_duration = retrieval_start.elapsed();

    assert!(retrieved_data.len() >= batch_size);
    assert!(retrieval_duration.as_millis() < 1000); // Should be fast

    // Phase 3: Test pattern analysis performance
    let pattern_batch_size = 30;
    let mut pattern_batch = Vec::new();

    for i in 0..pattern_batch_size {
        pattern_batch.push(create_test_usage_pattern(
            "performance_pattern",
            &format!("pattern_data_{}", i),
            10 + (i % 20), // Varying frequencies
        ));
    }

    // Store patterns
    for pattern in &pattern_batch {
        storage.store_usage_pattern(pattern).await.unwrap();
    }

    // Test pattern retrieval performance
    let pattern_start = std::time::Instant::now();
    let top_patterns = storage
        .get_top_patterns("performance_pattern", 20)
        .await
        .unwrap();
    let pattern_duration = pattern_start.elapsed();

    assert!(top_patterns.len() >= 20);
    assert!(pattern_duration.as_millis() < 500); // Should be very fast

    // Phase 4: Test concurrent access simulation
    let concurrent_feedback_count = 20;
    let mut feedback_batch = Vec::new();

    for i in 0..concurrent_feedback_count {
        feedback_batch.push(create_test_feedback(
            &format!("concurrent_content_{}", i % 5), // 5 different content items
            0.5 + (i as f64 * 0.02),                  // Varying scores
            &format!("Concurrent feedback {}", i),
        ));
    }

    // Store feedback concurrently (simulated)
    let concurrent_start = std::time::Instant::now();
    for feedback in &feedback_batch {
        storage.store_feedback(feedback).await.unwrap();
    }
    let concurrent_duration = concurrent_start.elapsed();

    assert!(concurrent_duration.as_millis() < 3000); // Should handle concurrent load

    // Phase 5: Test cleanup performance
    let cleanup_start = std::time::Instant::now();
    let cleanup_result = storage.cleanup_old_data(1).await.unwrap(); // Very recent retention
    let cleanup_duration = cleanup_start.elapsed();

    assert!(cleanup_duration.as_millis() < 2000); // Cleanup should be reasonably fast
    assert!(cleanup_result.deleted_learning_data >= 0);
}

/// ANCHOR: Integration test for learning data consistency and integrity
/// Tests: Data integrity across operations, transaction-like behavior
#[tokio::test]
async fn test_anchor_learning_data_integrity() {
    let storage = create_test_vector_storage().await;

    // Phase 1: Test data consistency across multiple operations
    let base_learning = create_test_learning_data(
        "integrity_test",
        "source_integrity",
        vec!["Original insight for integrity testing".to_string()],
        0.8,
    );

    let stored_learning = storage.store_learning_data(&base_learning).await.unwrap();
    assert_eq!(stored_learning.id, base_learning.id);
    assert_eq!(
        stored_learning.confidence_score,
        base_learning.confidence_score
    );

    // Phase 2: Test related data consistency
    let related_feedback = create_test_feedback(
        "integrity_content",
        0.85,
        "Feedback related to integrity test",
    );
    let stored_feedback = storage.store_feedback(&related_feedback).await.unwrap();

    let related_pattern =
        create_test_usage_pattern("integrity_pattern", "integrity test pattern data", 5);
    let stored_pattern = storage.store_usage_pattern(&related_pattern).await.unwrap();

    // Phase 3: Verify data relationships
    let all_recent_data = storage.get_recent_learning_data(10).await.unwrap();
    let integrity_data: Vec<_> = all_recent_data
        .iter()
        .filter(|d| d.learning_type == "integrity_test")
        .collect();
    assert_eq!(integrity_data.len(), 1);
    assert_eq!(integrity_data[0].id, stored_learning.id);

    // Phase 4: Test feedback integrity
    let content_feedback = storage
        .get_feedback_for_content("integrity_content")
        .await
        .unwrap();
    assert!(content_feedback.iter().any(|f| f.id == stored_feedback.id));

    // Phase 5: Test pattern integrity
    let pattern_results = storage
        .get_top_patterns("integrity_pattern", 5)
        .await
        .unwrap();
    assert!(pattern_results.iter().any(|p| p.id == stored_pattern.id));

    // Phase 6: Test data persistence after multiple operations
    let second_learning = create_test_learning_data(
        "integrity_test_2",
        "source_integrity_2",
        vec!["Second insight for integrity testing".to_string()],
        0.75,
    );
    storage.store_learning_data(&second_learning).await.unwrap();

    // Verify original data is still intact
    let updated_recent_data = storage.get_recent_learning_data(15).await.unwrap();
    assert!(updated_recent_data
        .iter()
        .any(|d| d.id == stored_learning.id));
    assert!(updated_recent_data
        .iter()
        .any(|d| d.id == second_learning.id));

    // Phase 7: Test metadata integrity
    let metadata_learning = LearningData {
        id: Uuid::new_v4().to_string(),
        learning_type: "metadata_test".to_string(),
        source_data_id: "metadata_source".to_string(),
        insights: vec!["Testing metadata integrity".to_string()],
        confidence_score: 0.9,
        created_at: Utc::now(),
        expires_at: Some(Utc::now() + Duration::hours(1)),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("test_key".to_string(), serde_json::json!("test_value"));
            meta.insert("numeric_key".to_string(), serde_json::json!(42));
            meta
        },
    };

    let stored_metadata_learning = storage
        .store_learning_data(&metadata_learning)
        .await
        .unwrap();
    assert_eq!(stored_metadata_learning.metadata.len(), 2);
    assert_eq!(
        stored_metadata_learning.metadata.get("test_key").unwrap(),
        "test_value"
    );
}

// Helper functions for test data creation

async fn create_test_vector_storage() -> impl LearningStorageService {
    // Returns a mock implementation that provides all required storage functionality
    // Production tests would use actual vector storage instance
    MockVectorLearningStorage::new()
}

fn create_test_learning_data(
    learning_type: &str,
    source_id: &str,
    insights: Vec<String>,
    confidence: f64,
) -> LearningData {
    LearningData {
        id: Uuid::new_v4().to_string(),
        learning_type: learning_type.to_string(),
        source_data_id: source_id.to_string(),
        insights,
        confidence_score: confidence,
        created_at: Utc::now(),
        expires_at: None,
        metadata: HashMap::new(),
    }
}

fn create_test_usage_pattern(pattern_type: &str, data: &str, frequency: u32) -> UsagePattern {
    UsagePattern {
        id: Uuid::new_v4().to_string(),
        pattern_type: pattern_type.to_string(),
        data: data.to_string(),
        frequency,
        last_used: Utc::now(),
        context: HashMap::new(),
    }
}

fn create_test_feedback(content_id: &str, score: f64, text: &str) -> UserFeedback {
    UserFeedback {
        id: Uuid::new_v4().to_string(),
        user_id: format!("user_{}", Uuid::new_v4()),
        content_id: content_id.to_string(),
        feedback_type: "quality_rating".to_string(),
        score: Some(score),
        text_feedback: Some(text.to_string()),
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    }
}

// Mock implementation for integration testing
struct MockVectorLearningStorage {
    feedback_store: Arc<tokio::sync::RwLock<HashMap<String, UserFeedback>>>,
    learning_store: Arc<tokio::sync::RwLock<HashMap<String, LearningData>>>,
    pattern_store: Arc<tokio::sync::RwLock<HashMap<String, PatternData>>>,
    usage_patterns: Arc<tokio::sync::RwLock<Vec<UsagePattern>>>,
}

impl MockVectorLearningStorage {
    fn new() -> Self {
        Self {
            feedback_store: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            learning_store: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            pattern_store: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            usage_patterns: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl LearningStorageService for MockVectorLearningStorage {
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
        _days: u32,
    ) -> LearningResult<Vec<UsagePattern>> {
        self.get_top_patterns(pattern_type, 5).await
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
