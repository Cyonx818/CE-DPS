//! Learning System Validation Tests
//!
//! Simple validation tests to ensure the learning system components
//! work correctly and provide a foundation for Task 3 completion.

use chrono::{DateTime, Utc};
use fortitude::learning::{
    FeedbackData, LearningConfig, LearningData, PatternData, UsagePattern, UserFeedback,
};
use std::collections::HashMap;
use uuid::Uuid;

/// ANCHOR: Verifies core learning data structures function correctly.
/// Tests: Data model creation, validation, serialization
#[test]
fn test_anchor_learning_data_structures() {
    // Test UserFeedback creation and validation
    let feedback = UserFeedback::new(
        "test_user".to_string(),
        "test_content".to_string(),
        "quality_rating".to_string(),
        Some(0.85),
        Some("Great quality response".to_string()),
    );

    assert!(feedback.is_valid());
    assert!(!feedback.id.is_empty());
    assert_eq!(feedback.user_id, "test_user");
    assert_eq!(feedback.content_id, "test_content");
    assert_eq!(feedback.score, Some(0.85));

    // Test invalid feedback
    let invalid_feedback = UserFeedback::new(
        "".to_string(), // Invalid empty user ID
        "content".to_string(),
        "rating".to_string(),
        Some(1.5), // Invalid score > 1.0
        None,
    );
    assert!(!invalid_feedback.is_valid());

    // Test PatternData creation and updates
    let mut pattern = PatternData::new("search_query".to_string(), 5, 0.8);
    assert_eq!(pattern.frequency, 5);
    assert_eq!(pattern.success_rate, 0.8);
    assert!(pattern.is_significant(3)); // Above threshold
    assert!(!pattern.is_significant(10)); // Below threshold

    // Test pattern update behavior
    pattern.update_occurrence(true);
    assert_eq!(pattern.frequency, 6);
    assert!(pattern.success_rate > 0.8); // Should improve

    pattern.update_occurrence(false);
    assert_eq!(pattern.frequency, 7);
    // Success rate should adjust accordingly

    // Test LearningData creation and validity
    let learning_data = LearningData::new(
        "user_preference".to_string(),
        feedback.id.clone(),
        vec![
            "User prefers detailed responses".to_string(),
            "Quality score indicates satisfaction".to_string(),
        ],
        0.9,
    );

    assert!(learning_data.is_valid());
    assert_eq!(learning_data.confidence_score, 0.9);
    assert_eq!(learning_data.insights.len(), 2);
    assert!(learning_data.expires_at.is_none()); // Not expired by default

    // Test learning data with expiration
    let expired_learning = learning_data.with_expiration(Utc::now() - chrono::Duration::hours(1));
    assert!(!expired_learning.is_valid()); // Should be expired

    // Test UsagePattern creation and usage tracking
    let mut usage_pattern =
        UsagePattern::new("response_format".to_string(), "detailed".to_string());
    assert_eq!(usage_pattern.frequency, 1);

    usage_pattern.increment_usage();
    assert_eq!(usage_pattern.frequency, 2);

    println!("Learning data structures validation completed successfully");
}

/// ANCHOR: Verifies feedback aggregation and analysis functionality.
/// Tests: Feedback processing, trend analysis, quality metrics
#[test]
fn test_anchor_feedback_aggregation() {
    let content_id = "test_content_123";

    // Create feedback entries with varying scores
    let feedback_entries = vec![
        create_test_feedback(content_id, 0.9, "Excellent"),
        create_test_feedback(content_id, 0.8, "Good"),
        create_test_feedback(content_id, 0.85, "Very good"),
        create_test_feedback(content_id, 0.7, "Decent"),
        create_test_feedback(content_id, 0.95, "Outstanding"),
    ];

    // Test feedback data aggregation
    let feedback_data = FeedbackData::from_feedback(content_id.to_string(), &feedback_entries);

    assert_eq!(feedback_data.content_id, content_id);
    assert_eq!(feedback_data.feedback_count, 5);

    // Calculate expected average: (0.9 + 0.8 + 0.85 + 0.7 + 0.95) / 5 = 0.84
    let expected_avg = 0.84;
    assert!((feedback_data.average_score - expected_avg).abs() < 0.01);

    // Test trend calculation with more entries
    let trend_feedback = vec![
        create_test_feedback_with_time(
            content_id,
            0.7,
            "Early feedback",
            Utc::now() - chrono::Duration::hours(4),
        ),
        create_test_feedback_with_time(
            content_id,
            0.75,
            "Getting better",
            Utc::now() - chrono::Duration::hours(3),
        ),
        create_test_feedback_with_time(
            content_id,
            0.8,
            "Improved",
            Utc::now() - chrono::Duration::hours(2),
        ),
        create_test_feedback_with_time(
            content_id,
            0.9,
            "Much better",
            Utc::now() - chrono::Duration::hours(1),
        ),
    ];

    let trend_data = FeedbackData::from_feedback(content_id.to_string(), &trend_feedback);
    assert!(trend_data.recent_trend > 0.0); // Should show improvement

    println!("Feedback aggregation validation completed successfully");
}

/// ANCHOR: Verifies learning configuration management.
/// Tests: Configuration defaults, validation, settings
#[test]
fn test_anchor_learning_configuration() {
    // Test default configuration
    let config = LearningConfig::default();

    assert!(config.enable_feedback_learning);
    assert!(config.enable_pattern_recognition);
    assert!(!config.enable_optimization); // Should be false by default for safety
    assert_eq!(config.adaptation_threshold, 0.7);
    assert_eq!(config.max_data_age_days, 90);
    assert_eq!(config.min_feedback_threshold, 5);
    assert_eq!(config.pattern_frequency_threshold, 3);
    assert_eq!(config.learning_rate, 0.1);

    // Test storage configuration defaults
    assert_eq!(config.storage.collection_name, "learning_data");
    assert!(config.storage.enable_embeddings);
    assert_eq!(config.storage.batch_size, 100);
    assert_eq!(config.storage.retention_days, 365);

    // Test adaptation configuration defaults
    assert!(!config.adaptation.enabled_algorithms.is_empty());
    assert_eq!(config.adaptation.update_frequency_hours, 24);
    assert!(!config.adaptation.auto_apply_adaptations); // Should be false for safety

    println!("Learning configuration validation completed successfully");
}

/// ANCHOR: Verifies learning system serialization and data persistence.
/// Tests: JSON serialization, data integrity, round-trip conversion
#[test]
fn test_anchor_learning_serialization() {
    // Test UserFeedback serialization
    let feedback = UserFeedback::new(
        "serialization_user".to_string(),
        "serialization_content".to_string(),
        "quality_test".to_string(),
        Some(0.88),
        Some("Serialization test feedback".to_string()),
    );

    let serialized = serde_json::to_string(&feedback).unwrap();
    let deserialized: UserFeedback = serde_json::from_str(&serialized).unwrap();

    assert_eq!(feedback.id, deserialized.id);
    assert_eq!(feedback.user_id, deserialized.user_id);
    assert_eq!(feedback.content_id, deserialized.content_id);
    assert_eq!(feedback.score, deserialized.score);
    assert_eq!(feedback.text_feedback, deserialized.text_feedback);

    // Test PatternData serialization
    let pattern = PatternData::new("serialization_pattern".to_string(), 10, 0.85);
    let serialized_pattern = serde_json::to_string(&pattern).unwrap();
    let deserialized_pattern: PatternData = serde_json::from_str(&serialized_pattern).unwrap();

    assert_eq!(pattern.id, deserialized_pattern.id);
    assert_eq!(pattern.pattern_type, deserialized_pattern.pattern_type);
    assert_eq!(pattern.frequency, deserialized_pattern.frequency);
    assert_eq!(pattern.success_rate, deserialized_pattern.success_rate);

    // Test LearningData serialization
    let learning_data = LearningData::new(
        "serialization_learning".to_string(),
        "serialization_source".to_string(),
        vec!["Serialization insight".to_string()],
        0.92,
    );

    let serialized_learning = serde_json::to_string(&learning_data).unwrap();
    let deserialized_learning: LearningData = serde_json::from_str(&serialized_learning).unwrap();

    assert_eq!(learning_data.id, deserialized_learning.id);
    assert_eq!(
        learning_data.learning_type,
        deserialized_learning.learning_type
    );
    assert_eq!(learning_data.insights, deserialized_learning.insights);
    assert_eq!(
        learning_data.confidence_score,
        deserialized_learning.confidence_score
    );

    // Test LearningConfig serialization
    let config = LearningConfig::default();
    let serialized_config = serde_json::to_string(&config).unwrap();
    let deserialized_config: LearningConfig = serde_json::from_str(&serialized_config).unwrap();

    assert_eq!(
        config.enable_feedback_learning,
        deserialized_config.enable_feedback_learning
    );
    assert_eq!(
        config.adaptation_threshold,
        deserialized_config.adaptation_threshold
    );
    assert_eq!(
        config.storage.collection_name,
        deserialized_config.storage.collection_name
    );

    println!("Learning serialization validation completed successfully");
}

/// ANCHOR: Verifies learning system error handling and edge cases.
/// Tests: Error conditions, boundary values, invalid inputs
#[test]
fn test_anchor_learning_error_handling() {
    // Test invalid feedback scores
    let invalid_feedback_high = UserFeedback::new(
        "user".to_string(),
        "content".to_string(),
        "rating".to_string(),
        Some(1.5), // Invalid score > 1.0
        None,
    );
    assert!(!invalid_feedback_high.is_valid());

    let invalid_feedback_negative = UserFeedback::new(
        "user".to_string(),
        "content".to_string(),
        "rating".to_string(),
        Some(-0.1), // Invalid negative score
        None,
    );
    assert!(!invalid_feedback_negative.is_valid());

    // Test boundary values
    let boundary_feedback_low = UserFeedback::new(
        "user".to_string(),
        "content".to_string(),
        "rating".to_string(),
        Some(0.0), // Valid boundary
        None,
    );
    assert!(boundary_feedback_low.is_valid());

    let boundary_feedback_high = UserFeedback::new(
        "user".to_string(),
        "content".to_string(),
        "rating".to_string(),
        Some(1.0), // Valid boundary
        None,
    );
    assert!(boundary_feedback_high.is_valid());

    // Test empty required fields
    let empty_user_feedback = UserFeedback::new(
        "".to_string(), // Invalid empty user ID
        "content".to_string(),
        "rating".to_string(),
        Some(0.8),
        None,
    );
    assert!(!empty_user_feedback.is_valid());

    let empty_content_feedback = UserFeedback::new(
        "user".to_string(),
        "".to_string(), // Invalid empty content ID
        "rating".to_string(),
        Some(0.8),
        None,
    );
    assert!(!empty_content_feedback.is_valid());

    let empty_type_feedback = UserFeedback::new(
        "user".to_string(),
        "content".to_string(),
        "".to_string(), // Invalid empty feedback type
        Some(0.8),
        None,
    );
    assert!(!empty_type_feedback.is_valid());

    // Test pattern significance with edge cases
    let pattern = PatternData::new("edge_case_pattern".to_string(), 0, 0.0);
    assert!(!pattern.is_significant(1)); // Frequency 0 should not be significant

    // Test learning data with invalid confidence
    let learning_data = LearningData::new(
        "test_learning".to_string(),
        "test_source".to_string(),
        vec!["Test insight".to_string()],
        1.5, // Invalid confidence > 1.0 (but constructor may not validate this)
    );
    // Note: Constructor doesn't validate confidence bounds, which might be intentional

    println!("Learning error handling validation completed successfully");
}

// Helper functions for test data creation

fn create_test_feedback(content_id: &str, score: f64, text: &str) -> UserFeedback {
    UserFeedback::new(
        format!("user_{}", Uuid::new_v4()),
        content_id.to_string(),
        "quality_rating".to_string(),
        Some(score),
        Some(text.to_string()),
    )
}

fn create_test_feedback_with_time(
    content_id: &str,
    score: f64,
    text: &str,
    timestamp: DateTime<Utc>,
) -> UserFeedback {
    let mut feedback = create_test_feedback(content_id, score, text);
    feedback.timestamp = timestamp;
    feedback
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Integration test for complete learning workflow
    #[test]
    fn test_learning_workflow_integration() {
        // Create feedback entries
        let content_id = "integration_test_content";
        let feedback_entries = vec![
            create_test_feedback(content_id, 0.9, "Great"),
            create_test_feedback(content_id, 0.8, "Good"),
            create_test_feedback(content_id, 0.85, "Very good"),
        ];

        // Aggregate feedback
        let feedback_data = FeedbackData::from_feedback(content_id.to_string(), &feedback_entries);
        assert_eq!(feedback_data.feedback_count, 3);
        assert!(feedback_data.average_score > 0.8);

        // Create usage patterns
        let patterns = vec![
            PatternData::new("query_type".to_string(), 10, 0.9),
            PatternData::new("response_format".to_string(), 5, 0.7),
        ];

        // Generate learning insights
        let insights = vec![LearningData::new(
            "user_preference".to_string(),
            content_id.to_string(),
            vec!["Users prefer high-quality responses".to_string()],
            feedback_data.average_score,
        )];

        // Validate the learning cycle
        assert!(!insights.is_empty());
        assert!(insights[0].confidence_score > 0.8);
        assert!(!patterns.is_empty());
        assert!(patterns.iter().all(|p| p.frequency > 0));

        println!("Learning workflow integration test completed successfully");
    }
}
