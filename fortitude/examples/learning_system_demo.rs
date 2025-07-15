//! Learning System Demonstration
//!
//! This example demonstrates the core functionality of the learning system
//! including data models, adaptation algorithms, and pattern recognition.

use chrono::Utc;
use fortitude::learning::{
    adaptation::{AdaptationAlgorithmFactory, FeedbackAnalyzer, PatternMatcher},
    optimization::{OptimizationConfig, PerformanceOptimizer},
    pattern_recognition::PatternRecognizer,
    AdaptationAlgorithm, AdaptationConfig, FeedbackData, LearningConfig, PatternData, UsagePattern,
    UserFeedback,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Learning System Demonstration");
    println!("================================");

    // 1. Demonstrate basic data models
    println!("\n1. Creating Learning Data Structures");

    let feedback = UserFeedback::new(
        "user123".to_string(),
        "content456".to_string(),
        "quality_rating".to_string(),
        Some(0.85),
        Some("Excellent response quality".to_string()),
    );

    println!(
        "✓ Created user feedback: {} (score: {:.2})",
        feedback.id,
        feedback.score.unwrap_or(0.0)
    );
    assert!(feedback.is_valid());

    let mut pattern = PatternData::new("search_query".to_string(), 1, 1.0);
    pattern.update_occurrence(true);
    pattern.update_occurrence(false);

    println!(
        "✓ Created pattern data: {} (frequency: {}, success rate: {:.2})",
        pattern.id, pattern.frequency, pattern.success_rate
    );

    // 2. Demonstrate adaptation algorithms
    println!("\n2. Testing Adaptation Algorithms");

    let config = AdaptationConfig::default();
    let feedback_analyzer = FeedbackAnalyzer::new(config.clone());

    let feedback_data = FeedbackData {
        content_id: "test_content".to_string(),
        average_score: 0.75,
        feedback_count: 10,
        recent_trend: 0.05,
    };

    let adaptation_result = feedback_analyzer.analyze_feedback(&feedback_data).await?;
    println!(
        "✓ Feedback analysis complete - {} recommendations, confidence: {:.2}",
        adaptation_result.recommendations.len(),
        adaptation_result.confidence_score
    );

    // Test pattern matcher
    let pattern_matcher = PatternMatcher::new(config.clone());
    let usage_patterns = vec![
        UsagePattern {
            id: "1".to_string(),
            pattern_type: "search".to_string(),
            data: "rust async".to_string(),
            frequency: 5,
            last_used: Utc::now(),
            context: HashMap::new(),
        },
        UsagePattern {
            id: "2".to_string(),
            pattern_type: "search".to_string(),
            data: "vector database".to_string(),
            frequency: 8,
            last_used: Utc::now(),
            context: HashMap::new(),
        },
    ];

    let pattern_result = pattern_matcher.analyze_patterns(&usage_patterns).await?;
    println!(
        "✓ Pattern analysis complete - {} insights, confidence: {:.2}",
        pattern_result.insights.len(),
        pattern_result.confidence_score
    );

    // 3. Demonstrate pattern recognition
    println!("\n3. Testing Pattern Recognition");

    let recognizer = PatternRecognizer::new();
    let detected_patterns = recognizer.detect_patterns(&usage_patterns).await?;
    println!(
        "✓ Detected {} significant patterns",
        detected_patterns.len()
    );

    let trend_analysis = recognizer.analyze_trends(&usage_patterns).await?;
    println!(
        "✓ Trend analysis: {} total patterns, {} recent, trend direction: {:.2}",
        trend_analysis.total_patterns,
        trend_analysis.recent_patterns,
        trend_analysis.trend_direction
    );

    // 4. Demonstrate performance optimization
    println!("\n4. Testing Performance Optimization");

    let opt_config = OptimizationConfig::default();
    let mut optimizer = PerformanceOptimizer::new(opt_config);

    let query_result = optimizer.optimize_queries(&usage_patterns).await?;
    println!(
        "✓ Query optimization: {} queries analyzed, estimated improvement: {:.2}%",
        query_result.optimized_queries,
        query_result.estimated_improvement * 100.0
    );

    let feedback_entries = vec![feedback];
    let response_result = optimizer.optimize_responses(&feedback_entries).await?;
    println!(
        "✓ Response optimization: {} responses analyzed, expected quality gain: {:.2}%",
        response_result.analyzed_responses,
        response_result.expected_quality_gain * 100.0
    );

    // 5. Demonstrate algorithm factory
    println!("\n5. Testing Algorithm Factory");

    let available_algorithms = AdaptationAlgorithmFactory::available_algorithms();
    println!("✓ Available algorithms: {:?}", available_algorithms);

    for algorithm_name in available_algorithms {
        let algorithm =
            AdaptationAlgorithmFactory::create_algorithm(algorithm_name, config.clone())?;
        println!("✓ Successfully created algorithm: {}", algorithm_name);
    }

    // 6. Demonstrate configuration
    println!("\n6. Testing Learning Configuration");

    let learning_config = LearningConfig::default();
    println!("✓ Learning config created:");
    println!(
        "  - Feedback learning: {}",
        learning_config.enable_feedback_learning
    );
    println!(
        "  - Pattern recognition: {}",
        learning_config.enable_pattern_recognition
    );
    println!("  - Optimization: {}", learning_config.enable_optimization);
    println!(
        "  - Adaptation threshold: {:.2}",
        learning_config.adaptation_threshold
    );
    println!(
        "  - Min feedback threshold: {}",
        learning_config.min_feedback_threshold
    );

    println!("\n🎉 Learning System Demonstration Complete!");
    println!("All core components are working correctly.");

    Ok(())
}
