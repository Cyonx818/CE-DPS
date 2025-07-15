//! Anchor Tests for Prompt Optimization Adaptation Algorithms
//!
//! ANCHOR: These tests protect critical prompt optimization functionality from regression.
//! They validate core adaptation algorithms that improve research prompts based on feedback.

use chrono::Utc;
use fortitude::learning::{
    AdaptationAlgorithmFactory, AdaptationConfig, FeedbackData, UsagePattern,
};
use std::collections::HashMap;

/// ANCHOR: Verifies PromptOptimizer provides high-priority recommendations for poor performance.
/// Tests: Critical optimization logic for underperforming prompts
#[tokio::test]
async fn test_anchor_prompt_optimizer_urgent_optimization() {
    let config = AdaptationConfig::default();
    let optimizer = AdaptationAlgorithmFactory::create_algorithm("prompt_optimizer", config)
        .expect("Should create prompt optimizer");

    // Test urgent optimization scenario - poor score with declining trend
    let urgent_feedback = FeedbackData {
        content_id: "failing_prompt".to_string(),
        average_score: 0.4,  // Well below acceptable threshold
        feedback_count: 20,  // Sufficient data
        recent_trend: -0.08, // Declining quality
    };

    let result = optimizer.analyze_feedback(&urgent_feedback).await.unwrap();

    // ANCHOR: Critical assertions for urgent optimization
    assert_eq!(
        result.priority, "high",
        "Urgent optimization should be high priority"
    );
    assert!(
        result.confidence_score > 0.5,
        "Should have reasonable confidence with sufficient data"
    );
    assert!(
        !result.recommendations.is_empty(),
        "Should provide optimization recommendations"
    );

    // Should contain critical optimization guidance
    let has_critical_recommendation = result
        .recommendations
        .iter()
        .any(|r| r.contains("Critical") || r.contains("immediate"));
    assert!(
        has_critical_recommendation,
        "Should identify critical optimization need"
    );
}

/// ANCHOR: Verifies QueryOptimizer identifies successful patterns for replication.
/// Tests: Pattern analysis and success identification for query optimization
#[tokio::test]
async fn test_anchor_query_optimizer_success_pattern_identification() {
    let config = AdaptationConfig::default();
    let optimizer = AdaptationAlgorithmFactory::create_algorithm("query_optimizer", config)
        .expect("Should create query optimizer");

    // Test with clear successful and failed patterns
    let patterns = vec![
        UsagePattern {
            id: "success_1".to_string(),
            pattern_type: "successful_query".to_string(),
            data: "comprehensive implementation analysis with examples".to_string(),
            frequency: 15, // High frequency
            last_used: Utc::now(),
            context: HashMap::new(),
        },
        UsagePattern {
            id: "success_2".to_string(),
            pattern_type: "successful_query".to_string(),
            data: "step-by-step technical documentation".to_string(),
            frequency: 12,
            last_used: Utc::now(),
            context: HashMap::new(),
        },
        UsagePattern {
            id: "fail_1".to_string(),
            pattern_type: "failed_query".to_string(),
            data: "vague request".to_string(),
            frequency: 3,
            last_used: Utc::now(),
            context: HashMap::new(),
        },
    ];

    let result = optimizer.analyze_patterns(&patterns).await.unwrap();

    // ANCHOR: Critical assertions for pattern identification
    assert!(
        result.confidence_score > 0.7,
        "Should have high confidence with clear patterns"
    );
    assert!(
        !result.insights.is_empty(),
        "Should provide pattern insights"
    );
    assert!(
        !result.recommendations.is_empty(),
        "Should provide optimization recommendations"
    );

    // Should identify successful patterns
    let identifies_success = result
        .insights
        .iter()
        .any(|i| i.contains("successful") && i.contains("2"));
    assert!(
        identifies_success,
        "Should identify multiple successful patterns"
    );

    // Should recommend leveraging successful patterns
    let recommends_replication = result
        .recommendations
        .iter()
        .any(|r| r.to_lowercase().contains("incorporate") || r.to_lowercase().contains("template"));
    assert!(
        recommends_replication,
        "Should recommend replicating successful patterns"
    );
}

/// ANCHOR: Verifies TemplateAdaptor maintains excellent performance templates.
/// Tests: High-performance template preservation and enhancement logic
#[tokio::test]
async fn test_anchor_template_adaptor_excellence_preservation() {
    let config = AdaptationConfig::default();
    let adaptor = AdaptationAlgorithmFactory::create_algorithm("template_adaptor", config)
        .expect("Should create template adaptor");

    // Test with excellent performance metrics
    let excellent_feedback = FeedbackData {
        content_id: "excellent_template".to_string(),
        average_score: 0.96, // Excellent performance
        feedback_count: 40,  // Strong data volume
        recent_trend: 0.02,  // Stable/slightly improving
    };

    let result = adaptor.analyze_feedback(&excellent_feedback).await.unwrap();

    // ANCHOR: Critical assertions for excellence preservation
    assert_eq!(
        result.priority, "low",
        "Excellent templates should be low priority for changes"
    );
    assert!(
        result.confidence_score > 0.85,
        "Should have high confidence with excellent data"
    );
    assert!(
        !result.recommendations.is_empty(),
        "Should provide preservation guidance"
    );

    // Should recommend maintaining excellence
    let maintains_excellence = result
        .recommendations
        .iter()
        .any(|r| r.contains("maintain") || r.contains("preserve") || r.contains("enhance"));
    assert!(
        maintains_excellence,
        "Should recommend maintaining excellent performance"
    );

    // Should promote as best practice
    let promotes_standard = result
        .recommendations
        .iter()
        .any(|r| r.contains("best practice") || r.contains("reference standard"));
    assert!(
        promotes_standard,
        "Should promote excellent templates as standards"
    );
}

/// ANCHOR: Verifies algorithm factory provides all expected optimization algorithms.
/// Tests: Complete algorithm availability for prompt optimization workflows
#[test]
fn test_anchor_optimization_algorithm_availability() {
    let algorithms = AdaptationAlgorithmFactory::available_algorithms();

    // ANCHOR: Critical assertions for algorithm availability
    assert!(
        algorithms.contains(&"prompt_optimizer"),
        "Must provide prompt optimizer"
    );
    assert!(
        algorithms.contains(&"query_optimizer"),
        "Must provide query optimizer"
    );
    assert!(
        algorithms.contains(&"template_adaptor"),
        "Must provide template adaptor"
    );
    assert!(
        algorithms.len() >= 5,
        "Must provide comprehensive algorithm suite"
    );

    // Verify each optimization algorithm can be created
    let config = AdaptationConfig::default();

    let prompt_optimizer =
        AdaptationAlgorithmFactory::create_algorithm("prompt_optimizer", config.clone());
    assert!(
        prompt_optimizer.is_ok(),
        "Prompt optimizer must be creatable"
    );

    let query_optimizer =
        AdaptationAlgorithmFactory::create_algorithm("query_optimizer", config.clone());
    assert!(query_optimizer.is_ok(), "Query optimizer must be creatable");

    let template_adaptor = AdaptationAlgorithmFactory::create_algorithm("template_adaptor", config);
    assert!(
        template_adaptor.is_ok(),
        "Template adaptor must be creatable"
    );
}

/// ANCHOR: Verifies coordinated optimization across multiple algorithms.
/// Tests: Integration and consistency across optimization algorithms
#[tokio::test]
async fn test_anchor_coordinated_prompt_optimization() {
    let config = AdaptationConfig::default();

    // Create all optimization algorithms
    let prompt_optimizer =
        AdaptationAlgorithmFactory::create_algorithm("prompt_optimizer", config.clone())
            .expect("Should create prompt optimizer");
    let query_optimizer =
        AdaptationAlgorithmFactory::create_algorithm("query_optimizer", config.clone())
            .expect("Should create query optimizer");
    let template_adaptor = AdaptationAlgorithmFactory::create_algorithm("template_adaptor", config)
        .expect("Should create template adaptor");

    // Test with moderate performance requiring optimization
    let moderate_feedback = FeedbackData {
        content_id: "moderate_performance".to_string(),
        average_score: 0.72, // Decent but improvable
        feedback_count: 18,
        recent_trend: -0.02, // Slight decline
    };

    // Analyze with all algorithms
    let prompt_result = prompt_optimizer
        .analyze_feedback(&moderate_feedback)
        .await
        .unwrap();
    let query_result = query_optimizer
        .analyze_feedback(&moderate_feedback)
        .await
        .unwrap();
    let template_result = template_adaptor
        .analyze_feedback(&moderate_feedback)
        .await
        .unwrap();

    // ANCHOR: Critical assertions for coordinated optimization
    assert!(
        !prompt_result.recommendations.is_empty(),
        "Prompt optimizer should provide guidance"
    );
    assert!(
        !query_result.recommendations.is_empty(),
        "Query optimizer should provide guidance"
    );
    assert!(
        !template_result.recommendations.is_empty(),
        "Template adaptor should provide guidance"
    );

    // All should have reasonable confidence for this data quality
    assert!(
        prompt_result.confidence_score > 0.5,
        "Prompt optimizer should have reasonable confidence"
    );
    assert!(
        query_result.confidence_score > 0.5,
        "Query optimizer should have reasonable confidence"
    );
    assert!(
        template_result.confidence_score > 0.5,
        "Template adaptor should have reasonable confidence"
    );

    // Priority should be appropriate for moderate performance
    assert!(
        prompt_result.priority == "medium" || prompt_result.priority == "high",
        "Prompt optimizer should prioritize improvement for declining moderate performance"
    );
    assert!(
        template_result.priority == "medium" || template_result.priority == "high",
        "Template adaptor should prioritize improvement for declining moderate performance"
    );
}
