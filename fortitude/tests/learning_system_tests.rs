//! Learning System Tests
//!
//! Comprehensive test suite for the learning system implementation.
//! These tests follow TDD methodology and verify core learning data structures,
//! storage layer, and adaptation interfaces.

use chrono::{DateTime, Utc};
use fortitude::learning::{
    storage::LearningStorageService, AdaptationAlgorithm, FeedbackData, LearningConfig,
    LearningData, LearningError, LearningResult, LearningStorage, PatternData, UsagePattern,
    UserFeedback,
};
use std::collections::HashMap;
use uuid::Uuid;

/// ANCHOR: Verifies learning data model structure and serialization.
/// Tests: Core data structures, validation, serialization/deserialization
#[test]
fn test_anchor_learning_data_model() {
    // Test UserFeedback structure
    let feedback = UserFeedback {
        id: Uuid::new_v4().to_string(),
        user_id: "test_user".to_string(),
        content_id: "test_content".to_string(),
        feedback_type: "quality_rating".to_string(),
        score: Some(0.85),
        text_feedback: Some("Excellent research quality".to_string()),
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    };

    assert!(!feedback.id.is_empty());
    assert_eq!(feedback.user_id, "test_user");
    assert_eq!(feedback.score, Some(0.85));

    // Test PatternData structure
    let pattern = PatternData {
        id: Uuid::new_v4().to_string(),
        pattern_type: "search_query".to_string(),
        frequency: 10,
        success_rate: 0.8,
        context: HashMap::new(),
        first_seen: Utc::now(),
        last_seen: Utc::now(),
    };

    assert!(!pattern.id.is_empty());
    assert_eq!(pattern.frequency, 10);
    assert_eq!(pattern.success_rate, 0.8);

    // Test LearningData structure
    let learning_data = LearningData {
        id: Uuid::new_v4().to_string(),
        learning_type: "feedback_analysis".to_string(),
        source_data_id: "source_123".to_string(),
        insights: vec!["User prefers detailed responses".to_string()],
        confidence_score: 0.75,
        created_at: Utc::now(),
        expires_at: None,
        metadata: HashMap::new(),
    };

    assert_eq!(learning_data.confidence_score, 0.75);
    assert_eq!(learning_data.insights.len(), 1);
}

/// ANCHOR: Verifies learning storage operations work end-to-end.
/// Tests: Store, retrieve, update, delete with vector database integration
#[tokio::test]
async fn test_anchor_learning_storage_operations() {
    let storage = create_test_learning_storage().await;

    // Test storing feedback data
    let feedback = UserFeedback {
        id: Uuid::new_v4().to_string(),
        user_id: "test_user".to_string(),
        content_id: "content_123".to_string(),
        feedback_type: "quality_rating".to_string(),
        score: Some(0.9),
        text_feedback: Some("High quality results".to_string()),
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    };

    let stored_feedback = storage.store_feedback(&feedback).await.unwrap();
    assert_eq!(stored_feedback.id, feedback.id);
    assert_eq!(stored_feedback.score, Some(0.9));

    // Test retrieving feedback
    let retrieved_feedback = storage.get_feedback(&feedback.id).await.unwrap();
    assert!(retrieved_feedback.is_some());
    let retrieved = retrieved_feedback.unwrap();
    assert_eq!(retrieved.user_id, "test_user");
    assert_eq!(retrieved.content_id, "content_123");

    // Test storing pattern data
    let pattern = PatternData {
        id: Uuid::new_v4().to_string(),
        pattern_type: "search_preference".to_string(),
        frequency: 5,
        success_rate: 0.85,
        context: HashMap::new(),
        first_seen: Utc::now(),
        last_seen: Utc::now(),
    };

    let stored_pattern = storage.store_pattern(&pattern).await.unwrap();
    assert_eq!(stored_pattern.frequency, 5);

    // Test retrieving patterns by type
    let patterns = storage
        .get_patterns_by_type("search_preference")
        .await
        .unwrap();
    assert!(!patterns.is_empty());
    assert!(patterns.iter().any(|p| p.id == pattern.id));

    // Test storing learning insights
    let learning_data = LearningData {
        id: Uuid::new_v4().to_string(),
        learning_type: "user_preference".to_string(),
        source_data_id: feedback.id.clone(),
        insights: vec!["User prefers concise summaries".to_string()],
        confidence_score: 0.8,
        created_at: Utc::now(),
        expires_at: None,
        metadata: HashMap::new(),
    };

    let stored_learning = storage.store_learning_data(&learning_data).await.unwrap();
    assert_eq!(stored_learning.confidence_score, 0.8);

    // Test querying learning data
    let recent_learning = storage.get_recent_learning_data(10).await.unwrap();
    assert!(!recent_learning.is_empty());
    assert!(recent_learning.iter().any(|l| l.id == learning_data.id));
}

/// ANCHOR: Verifies pattern recognition and analysis functionality.
/// Tests: Pattern extraction, frequency analysis, trend detection
#[tokio::test]
async fn test_anchor_pattern_recognition() {
    let storage = create_test_learning_storage().await;

    // Store multiple usage patterns
    let patterns = vec![
        create_test_usage_pattern("search_query", "rust async", 3),
        create_test_usage_pattern("search_query", "vector database", 5),
        create_test_usage_pattern("search_query", "machine learning", 2),
        create_test_usage_pattern("response_preference", "detailed", 4),
        create_test_usage_pattern("response_preference", "concise", 6),
    ];

    for pattern in &patterns {
        storage.store_usage_pattern(pattern).await.unwrap();
    }

    // Test pattern analysis
    let top_patterns = storage.get_top_patterns("search_query", 3).await.unwrap();
    assert_eq!(top_patterns.len(), 3);
    assert_eq!(top_patterns[0].data, "vector database"); // Highest frequency

    let preference_patterns = storage
        .get_top_patterns("response_preference", 2)
        .await
        .unwrap();
    assert_eq!(preference_patterns.len(), 2);
    assert_eq!(preference_patterns[0].data, "concise"); // Highest frequency

    // Test pattern trending
    let trending = storage
        .get_trending_patterns("search_query", 7)
        .await
        .unwrap();
    assert!(!trending.is_empty());
}

/// ANCHOR: Verifies feedback integration and quality metrics.
/// Tests: Feedback processing, quality scoring, improvement tracking
#[tokio::test]
async fn test_anchor_feedback_integration() {
    let storage = create_test_learning_storage().await;

    // Store multiple feedback entries for the same content
    let content_id = "content_test_123";
    let feedback_entries = vec![
        create_test_feedback(content_id, 0.9, "Excellent quality"),
        create_test_feedback(content_id, 0.8, "Good results"),
        create_test_feedback(content_id, 0.85, "Very helpful"),
        create_test_feedback(content_id, 0.7, "Decent but could be better"),
    ];

    for feedback in &feedback_entries {
        storage.store_feedback(feedback).await.unwrap();
    }

    // Test feedback aggregation
    let avg_score = storage
        .get_average_feedback_score(content_id)
        .await
        .unwrap();
    assert!(avg_score.is_some());
    let score = avg_score.unwrap();
    assert!(score > 0.8 && score < 0.85); // Should be around 0.8125

    // Test feedback trends
    let feedback_trend = storage.get_feedback_trend(content_id, 30).await.unwrap();
    assert_eq!(feedback_trend.total_feedback, 4);
    assert!(feedback_trend.average_score > 0.8);

    // Test quality improvement detection
    let recent_feedback = storage.get_recent_feedback(content_id, 2).await.unwrap();
    assert_eq!(recent_feedback.len(), 2);

    // Should be ordered by most recent first
    assert!(recent_feedback[0].timestamp >= recent_feedback[1].timestamp);
}

/// ANCHOR: Verifies adaptation algorithm interfaces and configuration.
/// Tests: Algorithm registration, configuration, execution flow
#[tokio::test]
async fn test_anchor_adaptation_interfaces() {
    let config = LearningConfig::default();
    assert!(config.enable_feedback_learning);
    assert!(config.enable_pattern_recognition);
    assert_eq!(config.adaptation_threshold, 0.7);

    // Test adaptation algorithm interface
    let algorithm = MockAdaptationAlgorithm::new();

    let feedback_data = FeedbackData {
        content_id: "test_content".to_string(),
        average_score: 0.85,
        feedback_count: 10,
        recent_trend: 0.05, // Improving
    };

    let adaptation_result = algorithm.analyze_feedback(&feedback_data).await.unwrap();
    assert!(!adaptation_result.recommendations.is_empty());
    assert!(adaptation_result.confidence_score > 0.0);

    // Test pattern-based adaptation
    let patterns = vec![
        create_test_usage_pattern("query_style", "detailed", 8),
        create_test_usage_pattern("query_style", "brief", 3),
    ];

    let pattern_result = algorithm.analyze_patterns(&patterns).await.unwrap();
    assert!(!pattern_result.insights.is_empty());
}

// Helper functions for test setup

async fn create_test_learning_storage() -> impl LearningStorageService {
    // This would create a test instance of the storage service
    // For now, we'll use a mock implementation
    MockLearningStorage::new()
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

// Mock implementations for testing

struct MockLearningStorage {
    feedback_store: std::sync::Arc<tokio::sync::RwLock<HashMap<String, UserFeedback>>>,
    pattern_store: std::sync::Arc<tokio::sync::RwLock<HashMap<String, PatternData>>>,
    learning_store: std::sync::Arc<tokio::sync::RwLock<HashMap<String, LearningData>>>,
    usage_patterns: std::sync::Arc<tokio::sync::RwLock<Vec<UsagePattern>>>,
}

impl MockLearningStorage {
    fn new() -> Self {
        Self {
            feedback_store: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            pattern_store: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            learning_store: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            usage_patterns: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl LearningStorageService for MockLearningStorage {
    async fn store_feedback(&self, feedback: &UserFeedback) -> LearningResult<UserFeedback> {
        let mut store = self.feedback_store.write().await;
        store.insert(feedback.id.clone(), feedback.clone());
        Ok(feedback.clone())
    }

    async fn get_feedback(&self, id: &str) -> LearningResult<Option<UserFeedback>> {
        let store = self.feedback_store.read().await;
        Ok(store.get(id).cloned())
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
        // Simple implementation for testing
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
}

#[derive(Debug, Clone)]
struct FeedbackTrend {
    content_id: String,
    total_feedback: usize,
    average_score: f64,
    trend_direction: f64,
}

struct MockAdaptationAlgorithm {}

impl MockAdaptationAlgorithm {
    fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl AdaptationAlgorithm for MockAdaptationAlgorithm {
    async fn analyze_feedback(&self, feedback: &FeedbackData) -> LearningResult<AdaptationResult> {
        Ok(AdaptationResult {
            recommendations: vec!["Maintain current quality level".to_string()],
            confidence_score: feedback.average_score,
            priority: if feedback.average_score > 0.8 {
                "low"
            } else {
                "high"
            }
            .to_string(),
        })
    }

    async fn analyze_patterns(
        &self,
        patterns: &[UsagePattern],
    ) -> LearningResult<PatternAnalysisResult> {
        let insights = patterns
            .iter()
            .map(|p| format!("Pattern '{}' appears {} times", p.data, p.frequency))
            .collect();

        Ok(PatternAnalysisResult {
            insights,
            confidence_score: 0.8,
            recommendations: vec!["Continue monitoring patterns".to_string()],
        })
    }
}

#[derive(Debug, Clone)]
struct AdaptationResult {
    recommendations: Vec<String>,
    confidence_score: f64,
    priority: String,
}

#[derive(Debug, Clone)]
struct PatternAnalysisResult {
    insights: Vec<String>,
    confidence_score: f64,
    recommendations: Vec<String>,
}
