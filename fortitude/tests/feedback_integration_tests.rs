// Integration tests for user feedback integration system
// These tests define the expected behavior of the feedback system before implementation

use chrono::Utc;
use fortitude::quality::feedback::{
    ABTestConfig, AlgorithmVariant, FeedbackAnalytics, FeedbackCollectionConfig, FeedbackCollector,
    FeedbackContext, FeedbackIntegrationSystem, FeedbackPrivacyConfig, FeedbackStorage,
    FeedbackType, LearningEngine, ProviderPreferenceLearning, QualityLearningConfig, UserFeedback,
};
use fortitude::quality::{QualityScore, QualityWeights};
use std::time::Duration;

#[tokio::test]
async fn test_feedback_collection_framework_creation() {
    // Test creating feedback collector with different mechanisms
    let config = FeedbackCollectionConfig::default();
    let collector = FeedbackCollector::new(config).await;

    assert!(collector.is_ok());
    let collector = collector.unwrap();

    // Should support all feedback types
    assert!(collector.supports_feedback_type(&FeedbackType::QualityRating));
    assert!(collector.supports_feedback_type(&FeedbackType::AccuracyCorrection));
    assert!(collector.supports_feedback_type(&FeedbackType::RelevanceFeedback));
    assert!(collector.supports_feedback_type(&FeedbackType::ProviderPreference));
    assert!(collector.supports_feedback_type(&FeedbackType::FeatureRequest));
    assert!(collector.supports_feedback_type(&FeedbackType::BugReport));
}

#[tokio::test]
async fn test_quality_rating_feedback() {
    let collector = create_test_feedback_collector().await;

    let feedback = UserFeedback {
        feedback_id: "test_rating_1".to_string(),
        user_id: Some("user123".to_string()),
        query: "What is Rust programming?".to_string(),
        provider: "claude".to_string(),
        feedback_type: FeedbackType::QualityRating,
        rating: Some(4), // 4/5 stars
        correction: None,
        relevance_score: None,
        comments: Some("Good explanation but could be more detailed".to_string()),
        timestamp: Utc::now(),
        context: FeedbackContext {
            research_type: "programming_language".to_string(),
            domain: Some("technology".to_string()),
            audience: Some("beginner".to_string()),
            original_quality_score: create_test_quality_score(),
            provider_response_time: Duration::from_millis(150),
        },
    };

    let result = collector.collect_feedback(feedback).await;
    assert!(result.is_ok());

    // Should process feedback in <50ms
    let start = std::time::Instant::now();
    let _ = collector.collect_feedback(create_test_feedback()).await;
    let duration = start.elapsed();
    assert!(duration < Duration::from_millis(50));
}

#[tokio::test]
async fn test_accuracy_correction_feedback() {
    let collector = create_test_feedback_collector().await;

    let feedback = UserFeedback {
        feedback_id: "correction_1".to_string(),
        user_id: Some("expert_user".to_string()),
        query: "When was Rust first released?".to_string(),
        provider: "openai".to_string(),
        feedback_type: FeedbackType::AccuracyCorrection,
        rating: None,
        correction: Some("Rust was first released in 2010, not 2015 as stated".to_string()),
        relevance_score: None,
        comments: Some("The response contained incorrect release date information".to_string()),
        timestamp: Utc::now(),
        context: FeedbackContext {
            research_type: "factual_query".to_string(),
            domain: Some("technology".to_string()),
            audience: Some("expert".to_string()),
            original_quality_score: create_test_quality_score(),
            provider_response_time: Duration::from_millis(200),
        },
    };

    let result = collector.collect_feedback(feedback.clone()).await;
    assert!(result.is_ok());

    // Should validate correction before accepting
    let validation_result = collector.validate_correction(&feedback).await;
    assert!(validation_result.is_ok());
}

#[tokio::test]
async fn test_provider_preference_learning() {
    let mut learning_config = QualityLearningConfig::default();
    learning_config.enable_provider_learning = true;
    learning_config.learning_rate = 0.1;

    let learning_engine = LearningEngine::new(learning_config).await.unwrap();
    let provider_learner = ProviderPreferenceLearning::new().await.unwrap();

    // Simulate user consistently rating one provider higher
    let high_quality_feedback = vec![
        create_provider_feedback("claude", 5, "Excellent detailed response"),
        create_provider_feedback("claude", 4, "Very good explanation"),
        create_provider_feedback("claude", 5, "Perfect answer"),
    ];

    let low_quality_feedback = vec![
        create_provider_feedback("openai", 2, "Too brief"),
        create_provider_feedback("openai", 3, "Adequate but not comprehensive"),
        create_provider_feedback("openai", 2, "Missing key details"),
    ];

    // Learn from feedback patterns
    for feedback in high_quality_feedback
        .into_iter()
        .chain(low_quality_feedback)
    {
        let result = learning_engine.process_feedback(feedback).await;
        assert!(result.is_ok());
    }

    // Provider preferences should be learned
    let preferences = provider_learner
        .get_user_preferences("user123")
        .await
        .unwrap();
    assert!(preferences.preferred_providers.contains_key("claude"));
    assert!(preferences.preferred_providers["claude"] > preferences.preferred_providers["openai"]);
}

#[tokio::test]
async fn test_quality_score_adaptation() {
    let learning_config = QualityLearningConfig {
        enable_quality_adaptation: true,
        adaptation_threshold: 0.1,
        min_feedback_count: 5,
        learning_rate: 0.05,
        ..Default::default()
    };

    let learning_engine = LearningEngine::new(learning_config).await.unwrap();

    // User consistently rates responses with high accuracy as better
    let accuracy_feedback = vec![
        create_dimension_feedback("accuracy", 5, 0.95),
        create_dimension_feedback("accuracy", 5, 0.92),
        create_dimension_feedback("accuracy", 4, 0.88),
        create_dimension_feedback("accuracy", 5, 0.96),
        create_dimension_feedback("accuracy", 4, 0.89),
    ];

    for feedback in accuracy_feedback {
        let result = learning_engine.adapt_quality_weights(feedback).await;
        assert!(result.is_ok());
    }

    // Quality weights should be adapted
    let adapted_weights = learning_engine
        .get_adapted_weights("user123")
        .await
        .unwrap();
    let original_weights = QualityWeights::research_optimized();

    // Accuracy weight should be increased based on user feedback patterns
    assert!(adapted_weights.accuracy > original_weights.accuracy);
}

#[tokio::test]
async fn test_real_time_feedback_collection() {
    let collector = create_test_feedback_collector().await;

    // Test real-time feedback during research session
    let session_id = "research_session_123";
    let start_result = collector.start_feedback_session(session_id).await;
    assert!(start_result.is_ok());

    // Collect feedback during active session
    let real_time_feedback = UserFeedback {
        feedback_id: "realtime_1".to_string(),
        user_id: Some("active_user".to_string()),
        query: "Explain async Rust".to_string(),
        provider: "claude".to_string(),
        feedback_type: FeedbackType::RelevanceFeedback,
        rating: None,
        correction: None,
        relevance_score: Some(0.85),
        comments: None,
        timestamp: Utc::now(),
        context: create_test_feedback_context(),
    };

    let result = collector
        .collect_realtime_feedback(session_id, real_time_feedback)
        .await;
    assert!(result.is_ok());

    // Should update learning in real-time (<5 seconds)
    let start = std::time::Instant::now();
    let _ = collector.apply_realtime_learning(session_id).await;
    let duration = start.elapsed();
    assert!(duration < Duration::from_secs(5));

    let end_result = collector.end_feedback_session(session_id).await;
    assert!(end_result.is_ok());
}

#[tokio::test]
async fn test_batch_feedback_processing() {
    let collector = create_test_feedback_collector().await;

    // Create batch of feedback entries
    let batch_feedback = vec![
        create_test_feedback(),
        create_provider_feedback("claude", 4, "Good response"),
        create_provider_feedback("openai", 3, "Adequate response"),
        create_correction_feedback("Fix: Rust was released in 2010"),
        create_relevance_feedback(0.78),
    ];

    // Note: process_feedback_batch is on FeedbackIntegrationSystem, not FeedbackCollector
    let system = FeedbackIntegrationSystem::new().await.unwrap();
    let batch_result = system.process_feedback_batch(batch_feedback).await;
    assert!(batch_result.is_ok());

    let processing_metrics = batch_result.unwrap();
    assert!(processing_metrics.total_processed > 0);
    assert!(processing_metrics.successful_count > 0);
    assert!(processing_metrics.processing_time < Duration::from_secs(10));
}

#[tokio::test]
async fn test_feedback_validation_and_filtering() {
    let collector = create_test_feedback_collector().await;

    // Valid feedback should be accepted
    let valid_feedback = create_test_feedback();
    let result = collector.validate_feedback(&valid_feedback).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_valid);

    // Invalid feedback should be filtered out
    let invalid_feedback = UserFeedback {
        feedback_id: "".to_string(), // Empty ID
        user_id: None,
        query: "".to_string(), // Empty query
        provider: "unknown_provider".to_string(),
        feedback_type: FeedbackType::QualityRating,
        rating: Some(6), // Invalid rating (should be 1-5)
        correction: None,
        relevance_score: Some(1.5), // Invalid relevance score (should be 0.0-1.0)
        comments: None,
        timestamp: Utc::now(),
        context: create_test_feedback_context(),
    };

    let result = collector.validate_feedback(&invalid_feedback).await;
    assert!(result.is_ok());
    assert!(!result.unwrap().is_valid);
}

#[tokio::test]
async fn test_privacy_preserving_feedback() {
    let privacy_config = FeedbackPrivacyConfig {
        anonymize_user_data: true,
        retain_feedback_duration: chrono::Duration::days(90),
        encrypt_sensitive_data: true,
        allow_data_export: true,
        require_consent: true,
    };

    let collector = FeedbackCollector::with_privacy_config(privacy_config)
        .await
        .unwrap();

    // Anonymous feedback should be accepted
    let anonymous_feedback = UserFeedback {
        feedback_id: "anon_1".to_string(),
        user_id: None, // Anonymous
        query: "Test query".to_string(),
        provider: "claude".to_string(),
        feedback_type: FeedbackType::QualityRating,
        rating: Some(4),
        correction: None,
        relevance_score: None,
        comments: None,
        timestamp: Utc::now(),
        context: create_test_feedback_context(),
    };

    let result = collector.collect_feedback(anonymous_feedback).await;
    assert!(result.is_ok());

    // Authenticated feedback should be properly anonymized
    let authenticated_feedback = UserFeedback {
        feedback_id: "auth_1".to_string(),
        user_id: Some("sensitive_user_123".to_string()),
        query: "Sensitive query with personal data".to_string(),
        provider: "claude".to_string(),
        feedback_type: FeedbackType::AccuracyCorrection,
        rating: None,
        correction: Some("Correction with user context".to_string()),
        relevance_score: None,
        comments: Some("Personal comment about experience".to_string()),
        timestamp: Utc::now(),
        context: create_test_feedback_context(),
    };

    let result = collector.collect_feedback(authenticated_feedback).await;
    assert!(result.is_ok());

    // Should be able to export user data
    let export_result = collector.export_user_data("sensitive_user_123").await;
    assert!(export_result.is_ok());
}

#[tokio::test]
async fn test_feedback_storage_and_querying() {
    let storage = FeedbackStorage::new().await.unwrap();

    // Store multiple feedback entries
    let feedback_entries = vec![
        create_test_feedback(),
        create_provider_feedback("claude", 5, "Excellent"),
        create_provider_feedback("openai", 3, "Average"),
        create_correction_feedback("Fix needed"),
    ];

    for feedback in &feedback_entries {
        let result = storage.store_feedback(feedback.clone()).await;
        assert!(result.is_ok());
    }

    // Query feedback by provider
    let claude_feedback = storage.query_by_provider("claude").await.unwrap();
    assert!(claude_feedback.len() >= 1);
    assert!(claude_feedback.iter().all(|f| f.provider == "claude"));

    // Query feedback by type
    let rating_feedback = storage
        .query_by_feedback_type(FeedbackType::QualityRating)
        .await
        .unwrap();
    assert!(rating_feedback.len() >= 1);
    assert!(rating_feedback
        .iter()
        .all(|f| f.feedback_type == FeedbackType::QualityRating));

    // Query feedback by time range
    let yesterday = Utc::now() - chrono::Duration::days(1);
    let tomorrow = Utc::now() + chrono::Duration::days(1);
    let recent_feedback = storage
        .query_by_time_range(yesterday, tomorrow)
        .await
        .unwrap();
    assert!(recent_feedback.len() >= feedback_entries.len());
}

#[tokio::test]
async fn test_feedback_analytics_and_trends() {
    let analytics = FeedbackAnalytics::new().await.unwrap();
    let storage = create_test_feedback_storage().await;

    // Generate test data for analytics
    for i in 0..20 {
        let feedback = create_provider_feedback(
            if i % 2 == 0 { "claude" } else { "openai" },
            (i % 5) + 1, // Ratings 1-5
            &format!("Test feedback {}", i),
        );
        let _ = storage.store_feedback(feedback).await;
    }

    // Analyze provider performance trends
    let provider_trends = analytics.analyze_provider_trends(&storage).await.unwrap();
    assert!(provider_trends.contains_key("claude"));
    assert!(provider_trends.contains_key("openai"));

    // Analyze feedback patterns
    let patterns = analytics.analyze_feedback_patterns(&storage).await.unwrap();
    assert!(patterns.total_feedback_count > 0);
    assert!(patterns.average_rating > 0.0);
    assert!(!patterns.common_issues.is_empty());

    // Quality improvement metrics
    let improvement_metrics = analytics
        .calculate_quality_improvement(&storage)
        .await
        .unwrap();
    assert!(improvement_metrics.baseline_accuracy >= 0.0);
    assert!(improvement_metrics.current_accuracy >= 0.0);
    assert!(improvement_metrics.improvement_percentage >= -100.0); // Can be negative for new systems
}

#[tokio::test]
async fn test_a_b_testing_support() {
    let learning_engine = LearningEngine::new(QualityLearningConfig::default())
        .await
        .unwrap();

    // Create two algorithm variants for A/B testing
    let variant_a = AlgorithmVariant {
        name: "enhanced_accuracy_weights".to_string(),
        weights: QualityWeights {
            accuracy: 0.35, // Increased accuracy weight
            relevance: 0.25,
            completeness: 0.15,
            clarity: 0.10,
            credibility: 0.10,
            timeliness: 0.03,
            specificity: 0.02,
        },
        description: "Enhanced accuracy weighting".to_string(),
    };

    let variant_b = QualityWeights::research_optimized(); // Control group

    // Start A/B test
    let test_config = ABTestConfig {
        test_name: "accuracy_weight_experiment".to_string(),
        variant_a_weight: 0.5,
        variant_b_weight: 0.5,
        min_sample_size: 100,
        confidence_level: 0.95,
        duration: chrono::Duration::days(7),
    };

    let ab_test = learning_engine
        .start_ab_test(test_config, variant_a, variant_b)
        .await;
    assert!(ab_test.is_ok());

    // Simulate feedback for both variants
    for i in 0..50 {
        let feedback = create_test_feedback();
        let result = learning_engine
            .process_ab_test_feedback("accuracy_weight_experiment", feedback)
            .await;
        assert!(result.is_ok());
    }

    // Analyze A/B test results
    let test_results = learning_engine
        .analyze_ab_test("accuracy_weight_experiment")
        .await;
    assert!(test_results.is_ok());

    let results = test_results.unwrap();
    assert!(results.statistical_significance >= 0.0);
    assert!(results.sample_size_a > 0);
    assert!(results.sample_size_b > 0);
}

#[tokio::test]
async fn test_quality_improvement_measurement() {
    let integration_system = FeedbackIntegrationSystem::new().await.unwrap();

    // Establish baseline quality
    let baseline_queries = vec![
        "What is machine learning?",
        "Explain quantum computing",
        "How does Rust memory management work?",
        "What are the benefits of async programming?",
    ];

    let baseline_accuracy = integration_system
        .measure_baseline_accuracy(&baseline_queries)
        .await
        .unwrap();
    assert!(baseline_accuracy >= 0.0 && baseline_accuracy <= 1.0);

    // Simulate learning from feedback over time
    for iteration in 0..10 {
        let feedback_batch = generate_test_feedback_batch(iteration);
        let _ = integration_system
            .process_feedback_batch(feedback_batch)
            .await;
        let _ = integration_system.apply_learning_updates().await;
    }

    // Measure improved accuracy
    let improved_accuracy = integration_system
        .measure_current_accuracy(&baseline_queries)
        .await
        .unwrap();
    assert!(improved_accuracy >= 0.0 && improved_accuracy <= 1.0);

    // Calculate improvement
    let improvement = integration_system
        .calculate_accuracy_improvement()
        .await
        .unwrap();
    assert!(improvement.baseline_accuracy == baseline_accuracy);
    assert!(improvement.current_accuracy == improved_accuracy);
    assert!(improvement.improvement_percentage >= -100.0); // Can be negative initially

    // Target: >95% accuracy after sufficient learning
    // Note: This would require substantial feedback data in real usage
    // For testing, we just verify the measurement system works
}

#[tokio::test]
async fn test_feedback_system_performance_requirements() {
    let integration_system = FeedbackIntegrationSystem::new().await.unwrap();

    // Test feedback collection latency (<50ms)
    let feedback = create_test_feedback();
    let start = std::time::Instant::now();
    let result = integration_system.collect_feedback(feedback).await;
    let latency = start.elapsed();

    assert!(result.is_ok());
    assert!(latency < Duration::from_millis(50));

    // Test learning update latency (<5 seconds)
    let start = std::time::Instant::now();
    let result = integration_system.apply_learning_updates().await;
    let update_latency = start.elapsed();

    assert!(result.is_ok());
    assert!(update_latency < Duration::from_secs(5));

    // Test storage efficiency (support millions of entries)
    let large_batch_size = 1000; // Simulating subset of millions
    let large_batch = (0..large_batch_size)
        .map(|i| create_indexed_feedback(i))
        .collect();

    let start = std::time::Instant::now();
    let result = integration_system.process_feedback_batch(large_batch).await;
    let batch_time = start.elapsed();

    assert!(result.is_ok());
    // Should process 1000 entries quickly (extrapolate to millions)
    assert!(batch_time < Duration::from_secs(10));

    // Test query performance (<100ms)
    let start = std::time::Instant::now();
    let analytics_result = integration_system.generate_analytics_report().await;
    let query_time = start.elapsed();

    assert!(analytics_result.is_ok());
    assert!(query_time < Duration::from_millis(100));
}

// Helper functions for tests

async fn create_test_feedback_collector() -> FeedbackCollector {
    let config = FeedbackCollectionConfig::default();
    FeedbackCollector::new(config).await.unwrap()
}

async fn create_test_feedback_storage() -> FeedbackStorage {
    FeedbackStorage::new().await.unwrap()
}

fn create_test_feedback() -> UserFeedback {
    UserFeedback {
        feedback_id: "test_1".to_string(),
        user_id: Some("user123".to_string()),
        query: "Test query".to_string(),
        provider: "claude".to_string(),
        feedback_type: FeedbackType::QualityRating,
        rating: Some(4),
        correction: None,
        relevance_score: None,
        comments: Some("Good response".to_string()),
        timestamp: Utc::now(),
        context: create_test_feedback_context(),
    }
}

fn create_test_feedback_context() -> FeedbackContext {
    FeedbackContext {
        research_type: "general".to_string(),
        domain: Some("technology".to_string()),
        audience: Some("general".to_string()),
        original_quality_score: create_test_quality_score(),
        provider_response_time: Duration::from_millis(100),
    }
}

fn create_test_quality_score() -> QualityScore {
    QualityScore {
        relevance: 0.8,
        accuracy: 0.85,
        completeness: 0.7,
        clarity: 0.9,
        credibility: 0.75,
        timeliness: 0.6,
        specificity: 0.8,
        composite: 0.78,
        confidence: 0.85,
    }
}

fn create_provider_feedback(provider: &str, rating: u8, comments: &str) -> UserFeedback {
    UserFeedback {
        feedback_id: format!("provider_{}_{}", provider, rating),
        user_id: Some("user123".to_string()),
        query: "Test query".to_string(),
        provider: provider.to_string(),
        feedback_type: FeedbackType::QualityRating,
        rating: Some(rating),
        correction: None,
        relevance_score: None,
        comments: Some(comments.to_string()),
        timestamp: Utc::now(),
        context: create_test_feedback_context(),
    }
}

fn create_correction_feedback(correction: &str) -> UserFeedback {
    UserFeedback {
        feedback_id: "correction_test".to_string(),
        user_id: Some("expert_user".to_string()),
        query: "Test query".to_string(),
        provider: "claude".to_string(),
        feedback_type: FeedbackType::AccuracyCorrection,
        rating: None,
        correction: Some(correction.to_string()),
        relevance_score: None,
        comments: None,
        timestamp: Utc::now(),
        context: create_test_feedback_context(),
    }
}

fn create_relevance_feedback(relevance_score: f64) -> UserFeedback {
    UserFeedback {
        feedback_id: "relevance_test".to_string(),
        user_id: Some("user123".to_string()),
        query: "Test query".to_string(),
        provider: "openai".to_string(),
        feedback_type: FeedbackType::RelevanceFeedback,
        rating: None,
        correction: None,
        relevance_score: Some(relevance_score),
        comments: None,
        timestamp: Utc::now(),
        context: create_test_feedback_context(),
    }
}

fn create_dimension_feedback(dimension: &str, rating: u8, score: f64) -> UserFeedback {
    let mut context = create_test_feedback_context();
    match dimension {
        "accuracy" => context.original_quality_score.accuracy = score,
        "relevance" => context.original_quality_score.relevance = score,
        "clarity" => context.original_quality_score.clarity = score,
        _ => {}
    }

    UserFeedback {
        feedback_id: format!("dimension_{}_{}", dimension, rating),
        user_id: Some("user123".to_string()),
        query: "Test query".to_string(),
        provider: "claude".to_string(),
        feedback_type: FeedbackType::QualityRating,
        rating: Some(rating),
        correction: None,
        relevance_score: None,
        comments: Some(format!("Testing {} dimension", dimension)),
        timestamp: Utc::now(),
        context,
    }
}

fn create_indexed_feedback(index: usize) -> UserFeedback {
    UserFeedback {
        feedback_id: format!("indexed_{}", index),
        user_id: Some(format!("user_{}", index % 100)), // 100 different users
        query: format!("Test query {}", index),
        provider: if index % 2 == 0 { "claude" } else { "openai" }.to_string(),
        feedback_type: if index % 3 == 0 {
            FeedbackType::QualityRating
        } else {
            FeedbackType::RelevanceFeedback
        },
        rating: Some((index % 5 + 1) as u8),
        correction: None,
        relevance_score: Some((index as f64 % 100.0) / 100.0),
        comments: Some(format!("Batch feedback {}", index)),
        timestamp: Utc::now(),
        context: create_test_feedback_context(),
    }
}

fn generate_test_feedback_batch(iteration: usize) -> Vec<UserFeedback> {
    let batch_size = 10;
    (0..batch_size)
        .map(|i| create_indexed_feedback(iteration * batch_size + i))
        .collect()
}

// Additional helper functions and test utilities
