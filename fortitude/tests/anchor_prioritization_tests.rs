//! ANCHOR: Critical prioritization system tests for knowledge gap research prioritization
//!
//! These tests verify critical functionality and should be maintained
//! as the system evolves. Do not delete these tests.
//!
//! Tests: Priority scoring algorithms, performance requirements, configuration validation,
//! development context impact, and error handling

use fortitude::proactive::{
    DetectedGap, DevelopmentContext, DevelopmentPhase, GapType, PrioritizationConfig,
    PriorityScorer, TaskPriority,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use validator::Validate;

// Test data helper functions
fn create_test_gap(gap_type: GapType, line: usize) -> DetectedGap {
    let gap_type_str = gap_type.to_string();
    DetectedGap {
        gap_type,
        file_path: PathBuf::from(format!("src/test_{}.rs", line)),
        line_number: line,
        column_number: Some(10),
        context: format!("Test context for line {}", line),
        description: format!("Test gap description for {}", gap_type_str),
        confidence: 0.85,
        priority: 7,
        metadata: HashMap::new(),
    }
}

/// ANCHOR: Verifies priority scoring algorithms meet <100ms performance requirement for up to 50 gaps
/// Tests: Performance requirements, batch processing efficiency, scaling behavior
#[tokio::test]
async fn test_anchor_priority_scoring_performance_requirements() {
    let config = PrioritizationConfig::for_performance();
    let context = DevelopmentContext::default();
    let scorer = PriorityScorer::new(config.clone(), context)
        .await
        .expect("Priority scorer creation should succeed");

    // Test single gap performance requirement
    let gap = create_test_gap(GapType::TodoComment, 1);
    let start_time = Instant::now();

    let result = scorer.score_gap_priority(&gap).await;
    let single_duration = start_time.elapsed();

    assert!(result.is_ok(), "Single gap scoring should succeed");

    // ANCHOR: Single gap scoring must be <50ms (performance config)
    assert!(
        single_duration < Duration::from_millis(50),
        "Single gap scoring took {:?}, exceeding 50ms performance limit",
        single_duration
    );

    // Test batch performance with maximum allowed gaps (25 for performance config)
    let gaps: Vec<_> = (0..25)
        .map(|i| {
            create_test_gap(
                match i % 5 {
                    0 => GapType::TodoComment,
                    1 => GapType::ApiDocumentationGap,
                    2 => GapType::UndocumentedTechnology,
                    3 => GapType::MissingDocumentation,
                    _ => GapType::ConfigurationGap,
                },
                i,
            )
        })
        .collect();

    let start_time = Instant::now();
    let result = scorer.score_gaps_batch(&gaps).await;
    let batch_duration = start_time.elapsed();

    assert!(result.is_ok(), "Batch scoring should succeed");
    let breakdowns = result.unwrap();
    assert_eq!(breakdowns.len(), 25, "Should score all 25 gaps");

    // ANCHOR: Batch scoring for 25 gaps must be <100ms (performance requirement)
    assert!(
        batch_duration < Duration::from_millis(100),
        "Batch scoring of 25 gaps took {:?}, exceeding 100ms requirement",
        batch_duration
    );

    // Verify all scores are valid
    for (i, breakdown) in breakdowns.iter().enumerate() {
        assert!(
            breakdown.final_score >= 0.0 && breakdown.final_score <= 10.0,
            "Gap {} has invalid score: {}",
            i,
            breakdown.final_score
        );
        assert!(
            breakdown.confidence >= 0.0 && breakdown.confidence <= 1.0,
            "Gap {} has invalid confidence: {}",
            i,
            breakdown.confidence
        );
    }

    // Check performance metrics
    let metrics = scorer.get_metrics().await;
    assert_eq!(metrics.total_scores_calculated, 26); // 1 single + 25 batch
    assert!(
        metrics.average_scoring_time < Duration::from_millis(50),
        "Average scoring time {} exceeded performance target",
        metrics.average_scoring_time.as_millis()
    );
}

/// ANCHOR: Verifies prioritization algorithm accuracy and consistency
/// Tests: Gap type urgency scoring, priority level assignment, score validation
#[tokio::test]
async fn test_anchor_prioritization_algorithm_accuracy() {
    let scorer = PriorityScorer::with_defaults()
        .await
        .expect("Default scorer creation should succeed");

    // Test gap type urgency scoring accuracy
    let gap_type_tests = vec![
        (GapType::ApiDocumentationGap, 7.0, TaskPriority::Medium), // Critical for users
        (GapType::UndocumentedTechnology, 6.5, TaskPriority::Medium), // High maintainability impact
        (GapType::TodoComment, 5.5, TaskPriority::Medium),         // Actionable items
        (GapType::MissingDocumentation, 4.5, TaskPriority::Medium), // Team understanding
        (GapType::ConfigurationGap, 3.5, TaskPriority::Medium),    // Deployment impact
    ];

    for (gap_type, expected_min_score, expected_min_priority) in gap_type_tests {
        let gap = create_test_gap(gap_type.clone(), 1);
        let breakdown = scorer
            .score_gap_priority(&gap)
            .await
            .expect("Gap scoring should succeed");

        // ANCHOR: Gap type urgency must reflect business impact hierarchy
        assert!(
            breakdown.gap_type_score >= expected_min_score,
            "Gap type {} scored {:.2}, expected >= {:.2}",
            gap_type.to_string(),
            breakdown.gap_type_score,
            expected_min_score
        );

        // Priority level should match expected minimum
        let priority_value = match breakdown.priority_level {
            TaskPriority::Low => 1,
            TaskPriority::Medium => 2,
            TaskPriority::High => 3,
            TaskPriority::Critical => 4,
        };
        let expected_value = match expected_min_priority {
            TaskPriority::Low => 1,
            TaskPriority::Medium => 2,
            TaskPriority::High => 3,
            TaskPriority::Critical => 4,
        };

        assert!(
            priority_value >= expected_value,
            "Gap type {} assigned {:?}, expected >= {:?}",
            gap_type.to_string(),
            breakdown.priority_level,
            expected_min_priority
        );

        // Score components should be reasonable
        assert!(breakdown.recency_score >= 0.0 && breakdown.recency_score <= 10.0);
        assert!(breakdown.impact_score >= 0.0 && breakdown.impact_score <= 10.0);
        assert!(breakdown.context_score >= 0.0 && breakdown.context_score <= 10.0);

        // Final score should be within valid range
        assert!(breakdown.final_score >= 0.0 && breakdown.final_score <= 10.0);
        assert!(breakdown.confidence >= 0.0 && breakdown.confidence <= 1.0);
    }
}

/// ANCHOR: Verifies development context impact on priority scoring
/// Tests: Development phase multipliers, API visibility impact, team size scaling
#[tokio::test]
async fn test_anchor_development_context_impact() {
    // Test production context with public API
    let mut production_context = DevelopmentContext::default();
    production_context.phase = DevelopmentPhase::Production;
    production_context.is_public_api = true;
    production_context.has_urgent_deadlines = true;
    production_context.team_size = 10;
    production_context.performance_critical = true;

    let config = PrioritizationConfig::default();
    let production_scorer = PriorityScorer::new(config.clone(), production_context)
        .await
        .expect("Production scorer creation should succeed");

    // Test prototyping context
    let mut prototype_context = DevelopmentContext::default();
    prototype_context.phase = DevelopmentPhase::Prototyping;
    prototype_context.is_public_api = false;
    prototype_context.has_urgent_deadlines = false;
    prototype_context.team_size = 1;

    let prototype_scorer = PriorityScorer::new(config, prototype_context)
        .await
        .expect("Prototype scorer creation should succeed");

    // API documentation gap should have much higher priority in production
    let api_gap = create_test_gap(GapType::ApiDocumentationGap, 1);

    let production_score = production_scorer
        .score_gap_priority(&api_gap)
        .await
        .expect("Production scoring should succeed");
    let prototype_score = prototype_scorer
        .score_gap_priority(&api_gap)
        .await
        .expect("Prototype scoring should succeed");

    // ANCHOR: Production context must significantly increase API gap priority
    assert!(
        production_score.final_score > prototype_score.final_score,
        "Production score {:.2} should exceed prototype score {:.2}",
        production_score.final_score,
        prototype_score.final_score
    );

    // Production should result in High or Critical priority
    assert!(
        matches!(
            production_score.priority_level,
            TaskPriority::High | TaskPriority::Critical
        ),
        "Production API gap should be High or Critical priority, got {:?}",
        production_score.priority_level
    );

    // Test that production context results in higher scores overall
    let tech_gap = create_test_gap(GapType::UndocumentedTechnology, 2);
    let production_tech_score = production_scorer
        .score_gap_priority(&tech_gap)
        .await
        .expect("Production tech scoring should succeed");
    let prototype_tech_score = prototype_scorer
        .score_gap_priority(&tech_gap)
        .await
        .expect("Prototype tech scoring should succeed");

    // Production phase should generally increase priority for technical gaps
    assert!(
        production_tech_score.final_score >= prototype_tech_score.final_score,
        "Production tech score {:.2} should be >= prototype tech score {:.2}",
        production_tech_score.final_score,
        prototype_tech_score.final_score
    );
}

/// ANCHOR: Verifies prioritization configuration validation and customization
/// Tests: Configuration validation, weight checks, custom settings
#[tokio::test]
async fn test_anchor_prioritization_configuration() {
    // Test valid configuration
    let valid_config = PrioritizationConfig::default();
    assert!(
        valid_config.validate().is_ok(),
        "Default configuration should be valid"
    );

    // Test invalid configuration with weights that don't sum to 1.0
    let mut invalid_config = PrioritizationConfig::default();
    invalid_config.gap_type_weight = 0.6;
    invalid_config.recency_weight = 0.4;
    invalid_config.impact_weight = 0.3; // Sum > 1.0
    invalid_config.context_weight = 0.1;

    let validation_result = invalid_config.validate();
    assert!(
        validation_result.is_err(),
        "Invalid weight configuration should be rejected"
    );

    // Test performance configuration
    let perf_config = PrioritizationConfig::for_performance();
    assert!(
        perf_config.validate().is_ok(),
        "Performance config should be valid"
    );
    assert!(
        perf_config.max_scoring_time_ms <= 100,
        "Performance config should have strict time limits"
    );
    assert!(
        perf_config.max_batch_size <= 50,
        "Performance config should have reasonable batch limits"
    );

    // Test accuracy configuration
    let accuracy_config = PrioritizationConfig::for_accuracy();
    assert!(
        accuracy_config.validate().is_ok(),
        "Accuracy config should be valid"
    );
    assert!(
        accuracy_config.max_batch_size >= 50,
        "Accuracy config should allow larger batches"
    );

    // Test configuration application
    let context = DevelopmentContext::default();
    let scorer = PriorityScorer::new(perf_config, context)
        .await
        .expect("Scorer creation with performance config should succeed");

    let gap = create_test_gap(GapType::TodoComment, 1);
    let result = scorer
        .score_gap_priority(&gap)
        .await
        .expect("Scoring with performance config should succeed");

    // Verify result quality
    assert!(
        result.final_score >= 0.0 && result.final_score <= 10.0,
        "Score should be in valid range"
    );
    assert!(
        result.confidence >= 0.0 && result.confidence <= 1.0,
        "Confidence should be in valid range"
    );
    assert!(matches!(
        result.priority_level,
        TaskPriority::Low | TaskPriority::Medium | TaskPriority::High | TaskPriority::Critical
    ));
}

/// ANCHOR: Verifies error handling and resilience
/// Tests: Invalid input handling, configuration errors, graceful failure modes
#[tokio::test]
async fn test_anchor_error_handling_resilience() {
    // Test batch size limits
    let valid_config = PrioritizationConfig::for_performance(); // Max 25 batch size
    let context = DevelopmentContext::default();
    let scorer = PriorityScorer::new(valid_config, context)
        .await
        .expect("Valid scorer creation should succeed");

    let oversized_batch: Vec<_> = (0..50)
        .map(|i| create_test_gap(GapType::TodoComment, i))
        .collect();

    let result = scorer.score_gaps_batch(&oversized_batch).await;
    assert!(result.is_err(), "Oversized batch should be rejected");

    // Test invalid gap data handling
    let invalid_gap = DetectedGap {
        gap_type: GapType::TodoComment,
        file_path: PathBuf::from("test.rs"),
        line_number: 1,
        column_number: Some(1),
        context: "test".to_string(),
        description: "test".to_string(),
        confidence: 0.0, // Edge case confidence
        priority: 15,    // Invalid priority
        metadata: HashMap::new(),
    };

    // Should handle gracefully and produce valid scores
    let result = scorer.score_gap_priority(&invalid_gap).await;
    assert!(
        result.is_ok(),
        "Invalid gap data should be handled gracefully"
    );

    let breakdown = result.unwrap();
    assert!(
        breakdown.final_score >= 0.0 && breakdown.final_score <= 10.0,
        "Should produce valid score despite invalid input"
    );
    assert!(
        breakdown.confidence >= 0.0 && breakdown.confidence <= 1.0,
        "Should produce valid confidence despite invalid input"
    );
}

/// ANCHOR: Verifies cache efficiency and correctness
/// Tests: Cache hit/miss behavior, performance improvement
#[tokio::test]
async fn test_anchor_caching_efficiency() {
    let mut config = PrioritizationConfig::default();
    config.enable_score_caching = true;
    config.score_cache_ttl_secs = 60; // 1 minute TTL

    let context = DevelopmentContext::default();
    let scorer = PriorityScorer::new(config, context)
        .await
        .expect("Caching scorer creation should succeed");

    let gap = create_test_gap(GapType::TodoComment, 1);

    // First scoring should be cache miss
    let start_time = Instant::now();
    let result1 = scorer
        .score_gap_priority(&gap)
        .await
        .expect("First scoring should succeed");
    let first_duration = start_time.elapsed();

    let metrics1 = scorer.get_metrics().await;
    assert!(metrics1.cache_misses > 0, "Should record cache miss");
    assert_eq!(metrics1.cache_hits, 0, "Should have no cache hits yet");

    // Second scoring should be cache hit
    let start_time = Instant::now();
    let result2 = scorer
        .score_gap_priority(&gap)
        .await
        .expect("Second scoring should succeed");
    let second_duration = start_time.elapsed();

    let metrics2 = scorer.get_metrics().await;
    assert!(metrics2.cache_hits > 0, "Should record cache hit");

    // ANCHOR: Cache hit must be significantly faster than cache miss
    assert!(
        second_duration < first_duration,
        "Cache hit ({:?}) should be faster than cache miss ({:?})",
        second_duration,
        first_duration
    );

    // Results should be identical
    assert!(
        (result1.final_score - result2.final_score).abs() < 0.001,
        "Cache hit result should match original"
    );
    assert_eq!(
        result1.priority_level, result2.priority_level,
        "Priority level should be consistent from cache"
    );

    // Test cache clearing
    scorer.clear_cache().await;

    // Next scoring should be cache miss again
    let _result3 = scorer
        .score_gap_priority(&gap)
        .await
        .expect("Post-clear scoring should succeed");

    let metrics3 = scorer.get_metrics().await;
    assert!(
        metrics3.cache_misses > metrics2.cache_misses,
        "Should record additional cache miss after clear"
    );
}

/// ANCHOR: Verifies batch processing efficiency and scalability
/// Tests: Large batch handling, memory efficiency, performance scaling
#[tokio::test]
async fn test_anchor_batch_processing_efficiency() {
    let config = PrioritizationConfig::for_accuracy(); // Higher batch size limit
    let context = DevelopmentContext::default();
    let scorer = PriorityScorer::new(config, context)
        .await
        .expect("Accuracy scorer creation should succeed");

    // Test with maximum batch size
    let large_batch: Vec<_> = (0..100)
        .map(|i| {
            create_test_gap(
                match i % 5 {
                    0 => GapType::TodoComment,
                    1 => GapType::ApiDocumentationGap,
                    2 => GapType::UndocumentedTechnology,
                    3 => GapType::MissingDocumentation,
                    _ => GapType::ConfigurationGap,
                },
                i,
            )
        })
        .collect();

    // ANCHOR: Large batch processing must maintain efficiency
    let start_time = Instant::now();
    let result = scorer.score_gaps_batch(&large_batch).await;
    let duration = start_time.elapsed();

    assert!(result.is_ok(), "Large batch processing should succeed");
    let results = result.unwrap();
    assert_eq!(results.len(), 100, "Should process all 100 gaps");

    // Performance should scale reasonably
    let per_gap_time = duration.as_millis() / 100;
    assert!(
        per_gap_time < 2, // Very efficient per-gap processing
        "Large batch took {}ms per gap, should be <2ms for efficiency",
        per_gap_time
    );

    // Memory usage should be reasonable (can't test directly, but verify no crashes)
    assert!(
        duration < Duration::from_millis(200),
        "Large batch took {:?}, should complete quickly",
        duration
    );

    // Test metrics efficiency
    let metrics = scorer.get_metrics().await;
    assert_eq!(
        metrics.batch_operations, 1,
        "Should count one batch operation"
    );
    assert_eq!(
        metrics.total_scores_calculated, 100,
        "Should count all gap scores"
    );
}
