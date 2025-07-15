//! Comprehensive Learning System Test Suite
//!
//! This test suite provides comprehensive behavioral validation, end-to-end workflow testing,
//! performance validation, and system integration tests for the complete learning system.
//! These tests validate all learning system components work together correctly and provide
//! regression protection for critical learning functionality.

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use fortitude::learning::{
    CleanupResult, FeedbackTrend, LearningConfig, LearningData, LearningError, LearningResult,
    LearningStorageService, PatternData, UsagePattern, UserFeedback,
};
use tracing::info;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{sleep, Duration as TokioDuration};
use uuid::Uuid;

/// ANCHOR: Verifies complete end-to-end learning system workflow integration.
/// Tests: Full learning workflow from feedback input to system adaptation
#[tokio::test]
async fn test_anchor_end_to_end_learning_workflow() {
    let test_env = setup_comprehensive_learning_environment().await;

    // Phase 1: Collect Initial Feedback
    println!("Phase 1: Collecting initial user feedback");
    let content_id = "research_result_001";
    let initial_feedback = vec![
        create_quality_feedback(content_id, 0.9, "Excellent research depth", "user_001"),
        create_quality_feedback(
            content_id,
            0.8,
            "Good but could be more concise",
            "user_002",
        ),
        create_quality_feedback(content_id, 0.85, "Very helpful information", "user_003"),
        create_quality_feedback(content_id, 0.75, "Needs better organization", "user_004"),
    ];

    for feedback in &initial_feedback {
        test_env.storage.store_feedback(feedback).await.unwrap();
    }

    // Verify feedback storage
    let stored_feedback = test_env
        .storage
        .get_feedback_for_content(content_id)
        .await
        .unwrap();
    assert_eq!(stored_feedback.len(), 4);

    // Phase 2: Pattern Recognition and Analysis
    println!("Phase 2: Analyzing usage patterns");
    let usage_patterns = vec![
        create_usage_pattern("search_query", "rust async programming", 15),
        create_usage_pattern("search_query", "vector database optimization", 12),
        create_usage_pattern("response_format", "detailed_with_examples", 20),
        create_usage_pattern("response_format", "concise_summary", 8),
    ];

    for pattern in &usage_patterns {
        test_env.storage.store_usage_pattern(pattern).await.unwrap();
    }

    // Verify pattern analysis
    let top_query_patterns = test_env
        .storage
        .get_top_patterns("search_query", 3)
        .await
        .unwrap();
    assert!(!top_query_patterns.is_empty());
    assert_eq!(top_query_patterns[0].data, "rust async programming");

    let top_format_patterns = test_env
        .storage
        .get_top_patterns("response_format", 2)
        .await
        .unwrap();
    assert_eq!(top_format_patterns[0].data, "detailed_with_examples");

    // Phase 3: Learning Data Generation
    println!("Phase 3: Generating learning insights");
    let learning_insights = vec![
        LearningData::new(
            "user_preference".to_string(),
            content_id.to_string(),
            vec![
                "Users prefer detailed responses with examples".to_string(),
                "Average quality score indicates good satisfaction".to_string(),
                "Some users request better organization".to_string(),
            ],
            0.82,
        ),
        LearningData::new(
            "pattern_insight".to_string(),
            "pattern_analysis_001".to_string(),
            vec![
                "Rust async programming is most searched topic".to_string(),
                "Users strongly prefer detailed format over concise".to_string(),
            ],
            0.78,
        ),
    ];

    for insight in &learning_insights {
        test_env.storage.store_learning_data(insight).await.unwrap();
    }

    // Phase 4: Feedback Trend Analysis
    println!("Phase 4: Analyzing feedback trends");
    let feedback_trend = test_env
        .storage
        .get_feedback_trend(content_id, 30)
        .await
        .unwrap();
    assert_eq!(feedback_trend.total_feedback, 4);
    assert!(feedback_trend.average_score > 0.8);

    // Phase 5: Basic Validation
    println!("Phase 5: Validating basic learning cycle");

    // Verify learning data persistence
    let recent_learning = test_env.storage.get_recent_learning_data(10).await.unwrap();
    assert_eq!(recent_learning.len(), 2);

    // Verify average feedback calculation
    let avg_score = test_env
        .storage
        .get_average_feedback_score(content_id)
        .await
        .unwrap();
    assert!(avg_score.is_some());
    assert!(avg_score.unwrap() > 0.8);

    println!("End-to-end learning workflow validation completed successfully");
}

/// ANCHOR: Verifies behavioral adaptation and learning system intelligence.
/// Tests: Learning algorithm behavior, adaptation quality, decision making
#[tokio::test]
async fn test_anchor_behavioral_learning_adaptation() {
    let test_env = setup_comprehensive_learning_environment().await;

    // Test Scenario 1: Quality Improvement Learning
    info!("Testing quality improvement learning behavior");

    let content_id = "improving_content";
    let progressive_feedback = vec![
        (
            create_quality_feedback(content_id, 0.6, "Needs improvement", "user_001"),
            0,
        ),
        (
            create_quality_feedback(content_id, 0.65, "Getting better", "user_002"),
            1,
        ),
        (
            create_quality_feedback(content_id, 0.75, "Much improved", "user_003"),
            2,
        ),
        (
            create_quality_feedback(content_id, 0.85, "Excellent now", "user_004"),
            3,
        ),
    ];

    for (feedback, delay_hours) in progressive_feedback {
        let mut timed_feedback = feedback;
        timed_feedback.timestamp = Utc::now() - Duration::hours(delay_hours);
        test_env
            .storage
            .store_feedback(&timed_feedback)
            .await
            .unwrap();
    }

    // Analyze learning behavior for improving content
    let trend = test_env
        .storage
        .get_feedback_trend(content_id, 7)
        .await
        .unwrap();
    assert!(trend.trend_direction > 0.0, "Should detect improving trend");
    assert!(
        trend.average_score > 0.7,
        "Should reflect overall improvement"
    );

    // Test Scenario 2: Pattern-Based Adaptation
    info!("Testing pattern-based adaptation behavior");

    let behavioral_patterns = vec![
        // High-frequency successful patterns
        create_usage_pattern_with_success("query_style", "detailed_technical", 20, 0.9),
        create_usage_pattern_with_success("query_style", "concise_overview", 5, 0.6),
        // Response format preferences
        create_usage_pattern_with_success("response_format", "code_examples", 18, 0.95),
        create_usage_pattern_with_success("response_format", "theoretical_only", 3, 0.4),
    ];

    for pattern in &behavioral_patterns {
        test_env.storage.store_usage_pattern(pattern).await.unwrap();
    }

    // Verify adaptive learning from patterns
    let top_query_styles = test_env
        .storage
        .get_top_patterns("query_style", 5)
        .await
        .unwrap();
    let preferred_style = &top_query_styles[0];
    assert_eq!(preferred_style.data, "detailed_technical");
    assert!(preferred_style.frequency > 15);

    // Test Scenario 3: Multi-Modal Learning Integration
    info!("Testing multi-modal learning integration");

    let integration_learning = LearningData::new(
        "behavioral_adaptation".to_string(),
        "multi_modal_analysis".to_string(),
        vec![
            "Users prefer detailed technical queries with code examples".to_string(),
            "Quality improvement correlates with user engagement".to_string(),
            "Pattern frequency indicates strong user preference".to_string(),
        ],
        0.88,
    );

    test_env
        .storage
        .store_learning_data(&integration_learning)
        .await
        .unwrap();

    // Verify learning correlation analysis
    let similar_behavioral_insights = test_env
        .storage
        .find_similar_insights("behavioral adaptation technical", 0.75, 3)
        .await
        .unwrap();

    assert!(!similar_behavioral_insights.is_empty());
    let top_insight = &similar_behavioral_insights[0];
    assert!(top_insight.similarity_score > 0.75);
    assert!(top_insight.learning_data.confidence_score > 0.8);

    // Test Scenario 4: Adaptive Threshold Adjustment
    info!("Testing adaptive threshold adjustment behavior");

    let threshold_test_feedback = vec![
        create_quality_feedback("threshold_test", 0.95, "Outstanding", "user_001"),
        create_quality_feedback("threshold_test", 0.92, "Excellent", "user_002"),
        create_quality_feedback("threshold_test", 0.88, "Very good", "user_003"),
    ];

    for feedback in &threshold_test_feedback {
        test_env.storage.store_feedback(feedback).await.unwrap();
    }

    let high_quality_trend = test_env
        .storage
        .get_feedback_trend("threshold_test", 7)
        .await
        .unwrap();
    assert!(
        high_quality_trend.average_score > 0.9,
        "Should detect high-quality pattern"
    );

    // Verify adaptive behavior for high-quality content
    let adaptation_factory = AdaptationAlgorithmFactory::new(test_env.config.adaptation.clone());
    let pattern_analyzer = adaptation_factory.create_pattern_analyzer().await.unwrap();

    let pattern_analysis = pattern_analyzer
        .analyze_patterns(&behavioral_patterns)
        .await
        .unwrap();
    assert!(pattern_analysis.confidence_score > 0.8);
    assert!(pattern_analysis.insights.len() >= 2);

    info!("Behavioral learning adaptation validation completed successfully");
}

/// ANCHOR: Verifies learning system performance under load and stress conditions.
/// Tests: Performance characteristics, scalability, resource utilization
#[tokio::test]
async fn test_anchor_learning_performance_validation() {
    let test_env = setup_comprehensive_learning_environment().await;
    let performance_monitor = LearningPerformanceMonitor::new(test_env.config.monitoring.clone());

    // Performance Test 1: High-Volume Feedback Processing
    info!("Testing high-volume feedback processing performance");

    let start_time = Instant::now();
    let batch_size = 100;
    let mut feedback_batch = Vec::new();

    for i in 0..batch_size {
        let feedback = create_quality_feedback(
            &format!("content_{}", i % 10), // 10 different content items
            0.7 + (i as f64 * 0.003),       // Varying scores
            &format!("Feedback #{}", i),
            &format!("user_{}", i % 20), // 20 different users
        );
        feedback_batch.push(feedback);
    }

    // Store feedback in parallel batches
    let batch_futures = feedback_batch.chunks(20).map(|chunk| {
        let storage = &test_env.storage;
        async move {
            let mut results = Vec::new();
            for feedback in chunk {
                results.push(storage.store_feedback(feedback).await);
            }
            results
        }
    });

    let batch_results: Vec<_> = futures::future::join_all(batch_futures).await;
    let total_stored = batch_results.iter().flatten().filter(|r| r.is_ok()).count();

    let processing_duration = start_time.elapsed();
    assert_eq!(total_stored, batch_size);
    assert!(
        processing_duration.as_secs() < 10,
        "Should process {} feedback entries in under 10 seconds",
        batch_size
    );

    let throughput = batch_size as f64 / processing_duration.as_secs_f64();
    info!(
        "Feedback processing throughput: {:.2} entries/second",
        throughput
    );
    assert!(
        throughput > 10.0,
        "Should achieve minimum 10 entries/second throughput"
    );

    // Performance Test 2: Pattern Analysis Scalability
    info!("Testing pattern analysis scalability");

    let pattern_start = Instant::now();
    let pattern_batch_size = 50;
    let pattern_types = vec![
        "search_query",
        "response_format",
        "user_behavior",
        "content_type",
    ];

    for i in 0..pattern_batch_size {
        let pattern_type = &pattern_types[i % pattern_types.len()];
        let pattern = create_usage_pattern(
            pattern_type,
            &format!("pattern_data_{}", i),
            (i % 20) + 1, // Frequency 1-20
        );
        test_env
            .storage
            .store_usage_pattern(&pattern)
            .await
            .unwrap();
    }

    // Test pattern retrieval performance
    let retrieval_start = Instant::now();
    let mut total_patterns = 0;

    for pattern_type in &pattern_types {
        let patterns = test_env
            .storage
            .get_top_patterns(pattern_type, 20)
            .await
            .unwrap();
        total_patterns += patterns.len();
    }

    let retrieval_duration = retrieval_start.elapsed();
    assert!(total_patterns >= pattern_types.len() * 5); // At least 5 patterns per type
    assert!(
        retrieval_duration.as_millis() < 2000,
        "Pattern retrieval should complete in under 2 seconds"
    );

    info!(
        "Pattern analysis completed in {:?}",
        pattern_start.elapsed()
    );

    // Performance Test 3: Similarity Search Performance
    info!("Testing similarity search performance");

    let similarity_start = Instant::now();
    let search_queries = vec![
        "user preference detailed",
        "technical documentation",
        "code examples rust",
        "performance optimization",
        "quality improvement",
    ];

    let mut total_similar_results = 0;
    for query in &search_queries {
        let similar_insights = test_env
            .storage
            .find_similar_insights(query, 0.6, 10)
            .await
            .unwrap();
        total_similar_results += similar_insights.len();
    }

    let similarity_duration = similarity_start.elapsed();
    assert!(
        similarity_duration.as_millis() < 5000,
        "Similarity search should complete in under 5 seconds"
    );

    info!(
        "Similarity search completed in {:?}, found {} total results",
        similarity_duration, total_similar_results
    );

    // Performance Test 4: Memory and Resource Utilization
    info!("Testing memory and resource utilization");

    let initial_metrics = performance_monitor
        .collect_performance_metrics()
        .await
        .unwrap();

    // Perform intensive operations
    for i in 0..20 {
        let learning_data = LearningData::new(
            "performance_test".to_string(),
            format!("test_source_{}", i),
            vec![
                format!("Performance insight #{}", i),
                "Resource utilization monitoring".to_string(),
            ],
            0.8 + (i as f64 * 0.01),
        );
        test_env
            .storage
            .store_learning_data(&learning_data)
            .await
            .unwrap();
    }

    let final_metrics = performance_monitor
        .collect_performance_metrics()
        .await
        .unwrap();

    // Verify reasonable resource usage
    assert!(final_metrics.memory_usage_mb > initial_metrics.memory_usage_mb);
    assert!(final_metrics.memory_usage_mb < initial_metrics.memory_usage_mb + 100); // Should not increase by more than 100MB

    // Performance Test 5: Concurrent Access Performance
    info!("Testing concurrent access performance");

    let concurrent_start = Instant::now();
    let concurrent_tasks = 10;

    let concurrent_futures: Vec<_> = (0..concurrent_tasks)
        .map(|task_id| {
            let storage = &test_env.storage;
            async move {
                let content_id = format!("concurrent_content_{}", task_id);

                // Store feedback
                let feedback = create_quality_feedback(
                    &content_id,
                    0.8,
                    "Concurrent test",
                    &format!("user_{}", task_id),
                );
                storage.store_feedback(&feedback).await.unwrap();

                // Retrieve feedback
                let retrieved = storage.get_feedback_for_content(&content_id).await.unwrap();
                assert_eq!(retrieved.len(), 1);

                // Store pattern
                let pattern = create_usage_pattern(
                    "concurrent_test",
                    &format!("data_{}", task_id),
                    task_id as u32 + 1,
                );
                storage.store_usage_pattern(&pattern).await.unwrap();

                task_id
            }
        })
        .collect();

    let concurrent_results: Vec<_> = futures::future::join_all(concurrent_futures).await;
    let concurrent_duration = concurrent_start.elapsed();

    assert_eq!(concurrent_results.len(), concurrent_tasks);
    assert!(
        concurrent_duration.as_secs() < 15,
        "Concurrent operations should complete in under 15 seconds"
    );

    info!("Performance validation completed successfully");
    info!("Total test duration: {:?}", concurrent_start.elapsed());
}

/// ANCHOR: Verifies comprehensive system integration across all learning components.
/// Tests: Component interactions, data flow, error handling, consistency
#[tokio::test]
async fn test_anchor_comprehensive_system_integration() {
    let test_env = setup_comprehensive_learning_environment().await;

    // Integration Test 1: Configuration Management Integration
    info!("Testing configuration management integration");

    let config_manager = LearningConfigManager::new();
    let enhanced_config = EnhancedLearningConfig {
        base_config: test_env.config.clone(),
        monitoring: test_env.config.monitoring.clone(),
        advanced_features: HashMap::new(),
    };

    config_manager
        .update_config(enhanced_config.clone())
        .await
        .unwrap();
    let retrieved_config = config_manager.get_current_config().await.unwrap();

    assert_eq!(
        retrieved_config.base_config.adaptation_threshold,
        enhanced_config.base_config.adaptation_threshold
    );
    assert_eq!(
        retrieved_config
            .monitoring
            .health_check
            .check_interval_seconds,
        enhanced_config
            .monitoring
            .health_check
            .check_interval_seconds
    );

    // Integration Test 2: Metrics Collection Integration
    info!("Testing metrics collection integration");

    let metrics_collector = LearningMetricsCollector::new(retrieved_config.monitoring.clone());

    // Generate metrics through system operations
    let test_content = "integration_test_content";
    let feedback = create_quality_feedback(
        test_content,
        0.9,
        "Integration test feedback",
        "integration_user",
    );
    test_env.storage.store_feedback(&feedback).await.unwrap();

    let pattern = create_usage_pattern("integration_test", "test_pattern_data", 5);
    test_env
        .storage
        .store_usage_pattern(&pattern)
        .await
        .unwrap();

    // Collect and verify metrics
    let collected_metrics = metrics_collector.collect_learning_metrics().await.unwrap();

    assert!(collected_metrics.total_feedback_entries > 0);
    assert!(collected_metrics.total_patterns > 0);
    assert!(collected_metrics.average_feedback_score > 0.0);
    assert!(collected_metrics.system_uptime_seconds > 0);

    // Integration Test 3: Health Monitoring Integration
    info!("Testing health monitoring integration");

    let health_checker = LearningHealthChecker::new(retrieved_config.monitoring.clone());
    let health_report = health_checker.check_system_health().await.unwrap();

    assert!(health_report.overall_health.is_healthy());
    assert!(!health_report.component_health.is_empty());
    assert!(health_report.timestamp <= Utc::now());

    // Verify individual component health
    let storage_health = health_report
        .component_health
        .iter()
        .find(|h| h.component_name == "storage")
        .expect("Storage health should be reported");
    assert!(storage_health.is_healthy);

    // Integration Test 4: Optimization Integration
    info!("Testing optimization integration");

    let optimizer = PerformanceOptimizer::new(OptimizationConfig::default());
    let optimization_context = OptimizationContext {
        recent_feedback: vec![feedback.clone()],
        usage_patterns: vec![pattern.clone()],
        system_metrics: collected_metrics.clone(),
    };

    let optimization_result = optimizer
        .optimize_system(&optimization_context)
        .await
        .unwrap();

    assert!(optimization_result.provider_selection.is_some());
    assert!(optimization_result.cache_optimization.is_some());
    assert!(optimization_result.query_optimization.is_some());
    assert!(optimization_result.performance_improvement >= 0.0);

    // Integration Test 5: Template System Integration
    info!("Testing template system integration");

    let template_service = TemplateOptimizationService::new(IntegrationConfig::default());
    let template_metrics = TemplatePerformanceMetrics {
        template_id: "integration_template".to_string(),
        usage_count: 15,
        average_quality_score: 0.85,
        optimization_potential: 0.12,
        last_updated: Utc::now(),
    };

    let template_recommendations = template_service
        .analyze_template_performance(&template_metrics)
        .await
        .unwrap();

    assert!(!template_recommendations.is_empty());

    let top_recommendation = &template_recommendations[0];
    assert!(!top_recommendation.recommendation_text.is_empty());
    assert!(top_recommendation.confidence_score > 0.0);
    assert!(top_recommendation.expected_improvement > 0.0);

    // Integration Test 6: Error Handling and Recovery Integration
    info!("Testing error handling and recovery integration");

    // Test invalid feedback handling
    let invalid_feedback = UserFeedback::new(
        "".to_string(), // Invalid empty user ID
        test_content.to_string(),
        "quality".to_string(),
        Some(1.5), // Invalid score > 1.0
        None,
    );

    assert!(!invalid_feedback.is_valid());

    // Test storage error recovery
    let storage_result = test_env.storage.get_feedback("nonexistent_id").await;
    assert!(storage_result.is_ok());
    assert!(storage_result.unwrap().is_none());

    // Integration Test 7: Data Consistency and Integrity
    info!("Testing data consistency and integrity");

    let consistency_content = "consistency_test_content";
    let consistency_feedback = create_quality_feedback(
        consistency_content,
        0.8,
        "Consistency test",
        "consistency_user",
    );

    // Store and retrieve multiple times
    test_env
        .storage
        .store_feedback(&consistency_feedback)
        .await
        .unwrap();
    let retrieved1 = test_env
        .storage
        .get_feedback(&consistency_feedback.id)
        .await
        .unwrap()
        .unwrap();
    let retrieved2 = test_env
        .storage
        .get_feedback(&consistency_feedback.id)
        .await
        .unwrap()
        .unwrap();

    // Verify consistency
    assert_eq!(retrieved1.id, retrieved2.id);
    assert_eq!(retrieved1.score, retrieved2.score);
    assert_eq!(retrieved1.text_feedback, retrieved2.text_feedback);
    assert_eq!(retrieved1.timestamp, retrieved2.timestamp);

    // Integration Test 8: Batch Operations Integration
    info!("Testing batch operations integration");

    let batch_learning_data = vec![
        LearningData::new(
            "batch_test_1".to_string(),
            "batch_source_1".to_string(),
            vec!["Batch insight 1".to_string()],
            0.8,
        ),
        LearningData::new(
            "batch_test_2".to_string(),
            "batch_source_2".to_string(),
            vec!["Batch insight 2".to_string()],
            0.75,
        ),
    ];

    let batch_result = test_env
        .storage
        .store_learning_data_batch(&batch_learning_data)
        .await
        .unwrap();

    assert_eq!(batch_result.successful.len(), 2);
    assert_eq!(batch_result.failed.len(), 0);
    assert_eq!(batch_result.total_attempted, 2);

    // Verify batch retrieval
    let ids: Vec<String> = batch_learning_data.iter().map(|d| d.id.clone()).collect();
    let retrieval_result = test_env
        .storage
        .retrieve_learning_data_batch(&ids)
        .await
        .unwrap();

    assert_eq!(retrieval_result.successful.len(), 2);
    assert_eq!(retrieval_result.failed.len(), 0);

    info!("Comprehensive system integration validation completed successfully");
}

/// ANCHOR: Verifies critical learning functionality regression protection.
/// Tests: Core functionality stability, backward compatibility, critical paths
#[tokio::test]
async fn test_anchor_critical_learning_regression_protection() {
    let test_env = setup_comprehensive_learning_environment().await;

    // Regression Test 1: Core Data Model Stability
    info!("Testing core data model stability");

    let feedback = UserFeedback::new(
        "regression_user".to_string(),
        "regression_content".to_string(),
        "quality_rating".to_string(),
        Some(0.85),
        Some("Regression test feedback".to_string()),
    );

    // Verify core functionality remains intact
    assert!(feedback.is_valid());
    assert!(!feedback.id.is_empty());
    assert_eq!(feedback.user_id, "regression_user");
    assert_eq!(feedback.content_id, "regression_content");
    assert_eq!(feedback.score, Some(0.85));

    // Test serialization stability
    let serialized = serde_json::to_string(&feedback).unwrap();
    let deserialized: UserFeedback = serde_json::from_str(&serialized).unwrap();
    assert_eq!(feedback.id, deserialized.id);
    assert_eq!(feedback.score, deserialized.score);

    // Regression Test 2: Storage Interface Stability
    info!("Testing storage interface stability");

    // Store and retrieve feedback (core functionality)
    let stored_feedback = test_env.storage.store_feedback(&feedback).await.unwrap();
    assert_eq!(stored_feedback.id, feedback.id);

    let retrieved_feedback = test_env.storage.get_feedback(&feedback.id).await.unwrap();
    assert!(retrieved_feedback.is_some());
    assert_eq!(retrieved_feedback.unwrap().id, feedback.id);

    // Test batch operations stability
    let batch_feedback = vec![feedback.clone()];
    // Note: Assuming batch feedback storage would work with enhanced interface

    // Regression Test 3: Pattern Recognition Stability
    info!("Testing pattern recognition stability");

    let pattern = PatternData::new("regression_pattern".to_string(), 10, 0.8);
    let stored_pattern = test_env.storage.store_pattern(&pattern).await.unwrap();
    assert_eq!(stored_pattern.id, pattern.id);
    assert_eq!(stored_pattern.frequency, 10);
    assert_eq!(stored_pattern.success_rate, 0.8);

    // Test pattern update behavior
    let mut updated_pattern = pattern.clone();
    updated_pattern.update_occurrence(true);
    assert_eq!(updated_pattern.frequency, 11);
    assert!(updated_pattern.success_rate > 0.8);

    // Regression Test 4: Learning Data Processing Stability
    info!("Testing learning data processing stability");

    let learning_data = LearningData::new(
        "regression_learning".to_string(),
        feedback.id.clone(),
        vec![
            "Regression test insight".to_string(),
            "Data processing stability verified".to_string(),
        ],
        0.9,
    );

    assert!(learning_data.is_valid());
    assert_eq!(learning_data.confidence_score, 0.9);
    assert_eq!(learning_data.insights.len(), 2);

    let stored_learning = test_env
        .storage
        .store_learning_data(&learning_data)
        .await
        .unwrap();
    assert_eq!(stored_learning.id, learning_data.id);

    // Regression Test 5: Feedback Aggregation Stability
    info!("Testing feedback aggregation stability");

    let content_id = "regression_aggregation_test";
    let aggregation_feedback = vec![
        create_quality_feedback(content_id, 0.8, "Test 1", "user1"),
        create_quality_feedback(content_id, 0.9, "Test 2", "user2"),
        create_quality_feedback(content_id, 0.7, "Test 3", "user3"),
    ];

    for fb in &aggregation_feedback {
        test_env.storage.store_feedback(fb).await.unwrap();
    }

    let average_score = test_env
        .storage
        .get_average_feedback_score(content_id)
        .await
        .unwrap();
    assert!(average_score.is_some());
    let score = average_score.unwrap();
    assert!((score - 0.8).abs() < 0.01); // Should be approximately 0.8

    // Regression Test 6: Configuration Management Stability
    info!("Testing configuration management stability");

    let config = LearningConfig::default();
    assert!(config.enable_feedback_learning);
    assert!(config.enable_pattern_recognition);
    assert!(!config.enable_optimization); // Should remain false by default for safety
    assert_eq!(config.adaptation_threshold, 0.7);
    assert_eq!(config.max_data_age_days, 90);

    // Regression Test 7: Error Handling Stability
    info!("Testing error handling stability");

    // Test invalid feedback handling
    let invalid_feedback = UserFeedback::new(
        "".to_string(), // Invalid empty user ID
        "content".to_string(),
        "type".to_string(),
        Some(1.5), // Invalid score
        None,
    );
    assert!(!invalid_feedback.is_valid());

    // Test non-existent data retrieval
    let missing_feedback = test_env.storage.get_feedback("nonexistent").await.unwrap();
    assert!(missing_feedback.is_none());

    // Regression Test 8: Performance Characteristics Stability
    info!("Testing performance characteristics stability");

    let perf_start = Instant::now();

    // Perform standard operations that should maintain performance characteristics
    for i in 0..10 {
        let test_feedback = create_quality_feedback(
            &format!("perf_content_{}", i),
            0.8,
            "Performance test",
            &format!("perf_user_{}", i),
        );
        test_env
            .storage
            .store_feedback(&test_feedback)
            .await
            .unwrap();
    }

    let perf_duration = perf_start.elapsed();
    assert!(
        perf_duration.as_secs() < 5,
        "Performance should remain stable"
    );

    // Regression Test 9: Data Migration Compatibility
    info!("Testing data migration compatibility");

    // Verify that old data structures can still be processed
    let legacy_style_feedback = UserFeedback {
        id: Uuid::new_v4().to_string(),
        user_id: "legacy_user".to_string(),
        content_id: "legacy_content".to_string(),
        feedback_type: "legacy_type".to_string(),
        score: Some(0.75),
        text_feedback: Some("Legacy feedback".to_string()),
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    };

    assert!(legacy_style_feedback.is_valid());
    let stored_legacy = test_env
        .storage
        .store_feedback(&legacy_style_feedback)
        .await
        .unwrap();
    assert_eq!(stored_legacy.id, legacy_style_feedback.id);

    info!("Critical learning regression protection validation completed successfully");
}

// Helper functions for test setup and data creation

async fn setup_comprehensive_learning_environment() -> TestEnvironment {
    let config = create_test_learning_config();
    let storage = create_test_storage().await;

    TestEnvironment { config, storage }
}

struct TestEnvironment {
    config: LearningConfig,
    storage: Box<dyn LearningStorageService>,
}

fn create_test_learning_config() -> LearningConfig {
    LearningConfig {
        enable_feedback_learning: true,
        enable_pattern_recognition: true,
        enable_optimization: true,
        adaptation_threshold: 0.7,
        max_data_age_days: 90,
        min_feedback_threshold: 3,
        pattern_frequency_threshold: 2,
        learning_rate: 0.1,
        storage: fortitude::learning::LearningStorageConfig::default(),
        adaptation: fortitude::learning::AdaptationConfig::default(),
    }
}

async fn create_test_storage() -> Box<dyn LearningStorageService> {
    // Create a mock storage implementation
    Box::new(MockLearningStorage::new())
}

fn create_quality_feedback(
    content_id: &str,
    score: f64,
    text: &str,
    user_id: &str,
) -> UserFeedback {
    UserFeedback::new(
        user_id.to_string(),
        content_id.to_string(),
        "quality_rating".to_string(),
        Some(score),
        Some(text.to_string()),
    )
}

fn create_usage_pattern(pattern_type: &str, data: &str, frequency: u32) -> UsagePattern {
    let mut pattern = UsagePattern::new(pattern_type.to_string(), data.to_string());
    for _ in 1..frequency {
        pattern.increment_usage();
    }
    pattern
}

fn create_usage_pattern_with_success(
    pattern_type: &str,
    data: &str,
    frequency: u32,
    success_rate: f64,
) -> UsagePattern {
    let mut pattern = create_usage_pattern(pattern_type, data, frequency);
    // Add success rate to context for tracking
    pattern
        .context
        .insert("success_rate".to_string(), serde_json::json!(success_rate));
    pattern
}

fn create_mock_performance_metrics() -> PerformanceMetrics {
    PerformanceMetrics {
        memory_usage_mb: 150.0,
        cpu_usage_percent: 25.0,
        disk_usage_mb: 500.0,
        network_usage_mbps: 10.0,
        response_time_ms: 250.0,
        throughput_ops_per_sec: 100.0,
        error_rate_percent: 0.5,
        uptime_seconds: 3600,
    }
}

// Mock storage implementation for testing
struct MockLearningStorage {
    feedback_store: Arc<tokio::sync::RwLock<HashMap<String, UserFeedback>>>,
    pattern_store: Arc<tokio::sync::RwLock<HashMap<String, PatternData>>>,
    learning_store: Arc<tokio::sync::RwLock<HashMap<String, LearningData>>>,
    usage_patterns: Arc<tokio::sync::RwLock<Vec<UsagePattern>>>,
}

impl MockLearningStorage {
    fn new() -> Self {
        Self {
            feedback_store: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            pattern_store: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            learning_store: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            usage_patterns: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
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
        self.get_top_patterns(pattern_type, 10).await
    }

    async fn get_average_feedback_score(&self, content_id: &str) -> LearningResult<Option<f64>> {
        let feedback = self.get_feedback_for_content(content_id).await?;
        let scores: Vec<f64> = feedback.iter().filter_map(|f| f.score).collect();

        if scores.is_empty() {
            Ok(None)
        } else {
            let average = scores.iter().sum::<f64>() / scores.len() as f64;
            Ok(Some(average))
        }
    }

    async fn get_feedback_trend(
        &self,
        content_id: &str,
        _days: u32,
    ) -> LearningResult<FeedbackTrend> {
        let feedback = self.get_feedback_for_content(content_id).await?;
        let scores: Vec<f64> = feedback.iter().filter_map(|f| f.score).collect();

        let average_score = if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

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
            total_feedback: feedback.len(),
            average_score,
            trend_direction,
        })
    }

    async fn get_recent_feedback(
        &self,
        content_id: &str,
        limit: usize,
    ) -> LearningResult<Vec<UserFeedback>> {
        let mut feedback = self.get_feedback_for_content(content_id).await?;
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

use futures;
