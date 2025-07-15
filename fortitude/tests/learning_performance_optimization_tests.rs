//! Performance Optimization Tests Based on Learning Insights
//!
//! Comprehensive test suite for performance optimization features that use
//! learning insights to improve system performance. These tests follow TDD
//! methodology and verify provider selection optimization, response caching
//! optimization, and query performance optimization.

use chrono::Utc;
use fortitude::learning::{
    CacheOptimizationResult, ComprehensiveOptimizationResult, LearningData, LearningError,
    OptimizationConfig, OptimizationContext, PatternData, PerformanceMetrics, PerformanceOptimizer,
    ProviderSelectionResult, QueryPerformanceResult, UsagePattern, UserFeedback,
};
use fortitude_types::{AudienceContext, ClassifiedRequest, DomainContext, ResearchType};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// ANCHOR: Verifies provider selection optimization based on learning insights.
/// Tests: Learning-based provider selection, performance history analysis, quality optimization
#[tokio::test]
async fn test_anchor_provider_selection_optimization() {
    let optimizer = create_test_optimizer().await;

    // Create learning data about provider performance
    let learning_data = vec![
        create_learning_data(
            "provider_performance",
            "openai",
            vec![
                "High success rate for implementation queries".to_string(),
                "Average latency 1.2s for code generation".to_string(),
                "User satisfaction 0.85 for technical content".to_string(),
            ],
            0.8,
        ),
        create_learning_data(
            "provider_performance",
            "claude",
            vec![
                "Excellent for explanatory content".to_string(),
                "Lower latency 0.8s for general queries".to_string(),
                "User satisfaction 0.9 for learning content".to_string(),
            ],
            0.9,
        ),
        create_learning_data(
            "provider_performance",
            "gemini",
            vec![
                "Good cost efficiency".to_string(),
                "Moderate quality for simple queries".to_string(),
                "User satisfaction 0.7 for basic tasks".to_string(),
            ],
            0.6,
        ),
    ];

    // Test provider selection optimization for implementation query
    let implementation_request = create_test_request(ResearchType::Implementation);
    let result = optimizer
        .optimize_provider_selection(&implementation_request, &learning_data)
        .await;

    assert!(result.is_ok());
    let optimization = result.unwrap();
    assert_eq!(optimization.recommended_provider, "openai"); // Should prefer OpenAI for implementation
    assert!(optimization.confidence_score > 0.7);
    assert!(!optimization.reasoning.is_empty());

    // Test provider selection optimization for learning query
    let learning_request = create_test_request(ResearchType::Learning);
    let result = optimizer
        .optimize_provider_selection(&learning_request, &learning_data)
        .await;

    assert!(result.is_ok());
    let optimization = result.unwrap();
    assert_eq!(optimization.recommended_provider, "claude"); // Should prefer Claude for learning
    assert!(optimization.confidence_score > 0.8);

    // Test fallback when no strong preference exists
    let decision_request = create_test_request(ResearchType::Decision);
    let result = optimizer
        .optimize_provider_selection(&decision_request, &learning_data)
        .await;

    assert!(result.is_ok());
    let optimization = result.unwrap();
    assert!(!optimization.recommended_provider.is_empty());
    assert!(optimization.confidence_score > 0.0);
}

/// ANCHOR: Verifies response caching optimization using usage patterns.
/// Tests: Cache strategy optimization, TTL adjustment, hit rate improvement
#[tokio::test]
async fn test_anchor_response_caching_optimization() {
    let optimizer = create_test_optimizer().await;

    // Create usage patterns showing cache behavior
    let usage_patterns = vec![
        create_usage_pattern("cache_hit", "rust async patterns", 15, 0.8),
        create_usage_pattern("cache_hit", "vector database setup", 12, 0.9),
        create_usage_pattern("cache_miss", "custom implementation", 8, 0.3),
        create_usage_pattern("cache_hit", "error handling patterns", 20, 0.85),
        create_usage_pattern("cache_miss", "specific troubleshooting", 5, 0.2),
    ];

    // Create feedback showing quality correlation with cache usage
    let feedback_data = vec![
        create_feedback("cached_response_1", 0.9, "Quick and accurate"),
        create_feedback("cached_response_2", 0.85, "Consistent quality"),
        create_feedback("fresh_response_1", 0.7, "Slower but relevant"),
        create_feedback("cached_response_3", 0.88, "Reliable results"),
    ];

    let result = optimizer
        .optimize_response_caching(&usage_patterns, &feedback_data)
        .await;

    assert!(result.is_ok());
    let optimization = result.unwrap();

    // Should recommend aggressive caching for high-hit patterns
    assert!(optimization.cache_strategy.contains("aggressive"));
    assert!(optimization.recommended_ttl_hours > 1);
    assert!(optimization.estimated_hit_rate_improvement > 0.0);
    assert!(optimization.confidence_score > 0.6);

    // Should identify patterns worth caching
    assert!(!optimization.cacheable_patterns.is_empty());
    assert!(optimization
        .cacheable_patterns
        .iter()
        .any(|p| p.contains("rust async")));
    assert!(optimization
        .cacheable_patterns
        .iter()
        .any(|p| p.contains("vector database")));

    // Should recommend against caching low-hit patterns
    assert!(!optimization.non_cacheable_patterns.is_empty());
    assert!(optimization
        .non_cacheable_patterns
        .iter()
        .any(|p| p.contains("custom implementation")));
}

/// ANCHOR: Verifies query performance optimization using successful patterns.
/// Tests: Query optimization, pattern recognition, performance prediction
#[tokio::test]
async fn test_anchor_query_performance_optimization() {
    let optimizer = create_test_optimizer().await;

    // Create patterns showing successful query structures
    let mut context1 = HashMap::new();
    context1.insert("query_length".to_string(), serde_json::Value::from(50));
    context1.insert("specificity".to_string(), serde_json::Value::from("high"));
    context1.insert(
        "context_provided".to_string(),
        serde_json::Value::from(true),
    );

    let mut context2 = HashMap::new();
    context2.insert("query_length".to_string(), serde_json::Value::from(75));
    context2.insert("specificity".to_string(), serde_json::Value::from("medium"));
    context2.insert(
        "context_provided".to_string(),
        serde_json::Value::from(true),
    );

    let mut context3 = HashMap::new();
    context3.insert("query_length".to_string(), serde_json::Value::from(15));
    context3.insert("specificity".to_string(), serde_json::Value::from("low"));
    context3.insert(
        "context_provided".to_string(),
        serde_json::Value::from(false),
    );

    let successful_patterns = vec![
        create_pattern_data("successful_query", 25, 0.9, context1),
        create_pattern_data("successful_query", 18, 0.85, context2),
        create_pattern_data("failed_query", 8, 0.3, context3),
    ];

    // Create learning insights about query optimization
    let learning_insights = vec![
        create_learning_data(
            "query_optimization",
            "pattern_analysis",
            vec![
                "Queries with context have 90% higher success rate".to_string(),
                "Optimal query length is 50-75 characters".to_string(),
                "High specificity correlates with better results".to_string(),
            ],
            0.85,
        ),
        create_learning_data(
            "query_optimization",
            "performance_analysis",
            vec![
                "Structured queries reduce processing time by 40%".to_string(),
                "Context-aware queries improve relevance by 60%".to_string(),
            ],
            0.8,
        ),
    ];

    let test_query = "How to implement async Rust?";
    let result = optimizer
        .optimize_query_performance(test_query, &successful_patterns, &learning_insights)
        .await;

    assert!(result.is_ok());
    let optimization = result.unwrap();

    // Should provide optimization suggestions
    assert!(!optimization.optimization_suggestions.is_empty());
    assert!(optimization
        .optimization_suggestions
        .iter()
        .any(|s| s.contains("context")));

    // Should predict performance improvement
    assert!(optimization.estimated_performance_gain > 0.0);
    assert!(optimization.confidence_score > 0.7);

    // Should provide optimized query if possible
    if let Some(optimized) = &optimization.optimized_query {
        assert!(optimized.len() >= test_query.len()); // Should be enhanced, not shortened
        assert!(optimized.contains("async"));
        assert!(optimized.contains("Rust"));
    }

    // Should identify successful patterns to apply
    assert!(!optimization.successful_patterns_applied.is_empty());
}

/// ANCHOR: Verifies integrated learning-based optimization workflow.
/// Tests: Multi-dimensional optimization, learning integration, performance tracking
#[tokio::test]
async fn test_anchor_integrated_learning_optimization() {
    let optimizer = create_test_optimizer().await;

    // Create comprehensive learning data
    let learning_data = vec![
        create_learning_data(
            "system_performance",
            "overall",
            vec![
                "Average response time improved 25% with caching".to_string(),
                "Provider selection based on query type improves satisfaction by 15%".to_string(),
                "Optimized queries reduce failure rate by 30%".to_string(),
            ],
            0.9,
        ),
        create_learning_data(
            "user_satisfaction",
            "performance",
            vec![
                "Users prefer faster responses over perfect accuracy".to_string(),
                "Cache hit rate above 70% maintains quality perception".to_string(),
            ],
            0.8,
        ),
    ];

    // Test comprehensive optimization
    let request = create_test_request(ResearchType::Implementation);
    let optimization_context = OptimizationContext {
        request: &request,
        recent_patterns: create_test_patterns(),
        performance_history: create_test_performance_metrics(),
        user_feedback: create_test_feedback_data(),
    };

    let result = optimizer
        .optimize_comprehensive(&optimization_context, &learning_data)
        .await;

    assert!(result.is_ok());
    let optimization = result.unwrap();

    // Should provide provider recommendation
    assert!(!optimization
        .provider_recommendation
        .recommended_provider
        .is_empty());
    assert!(optimization.provider_recommendation.confidence_score > 0.0);

    // Should provide caching strategy
    assert!(!optimization.caching_strategy.cache_strategy.is_empty());
    assert!(optimization.caching_strategy.recommended_ttl_hours > 0);

    // Should provide query optimization
    assert!(!optimization
        .query_optimization
        .optimization_suggestions
        .is_empty());

    // Should track overall performance improvement
    assert!(optimization.overall_performance_improvement > 0.0);
    assert!(optimization.confidence_score >= 0.5);

    // Should provide actionable insights
    assert!(!optimization.insights.is_empty());
    assert!(optimization
        .insights
        .iter()
        .any(|i| i.contains("performance") || i.contains("optimization")));
}

/// ANCHOR: Verifies optimization configuration and error handling.
/// Tests: Configuration validation, error recovery, optimization constraints
#[tokio::test]
async fn test_anchor_optimization_configuration_and_errors() {
    // Test with invalid configuration
    let invalid_config = OptimizationConfig {
        min_confidence_threshold: 1.5,   // Invalid - should be <= 1.0
        max_optimization_suggestions: 0, // Invalid - should be > 0
        enable_aggressive_optimization: true,
        performance_weight: -0.1, // Invalid - should be >= 0.0
        ..Default::default()
    };

    let result = PerformanceOptimizer::new(invalid_config).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        LearningError::ConfigurationError(msg) => {
            assert!(msg.contains("confidence_threshold") || msg.contains("performance_weight"));
        }
        _ => panic!("Expected ConfigurationError"),
    }

    // Test with valid configuration
    let valid_config = OptimizationConfig::default();
    let optimizer = PerformanceOptimizer::new(valid_config).await.unwrap();

    // Test error handling with insufficient data
    let empty_learning_data: Vec<LearningData> = vec![];
    let request = create_test_request(ResearchType::Implementation);

    let result = optimizer
        .optimize_provider_selection(&request, &empty_learning_data)
        .await;
    assert!(result.is_ok()); // Should handle gracefully with default recommendations
    let optimization = result.unwrap();
    assert!(optimization.confidence_score < 0.5); // Low confidence with no data

    // Test error handling with conflicting data
    let conflicting_data = vec![
        create_learning_data(
            "provider_performance",
            "test",
            vec!["High quality".to_string()],
            0.9,
        ),
        create_learning_data(
            "provider_performance",
            "test",
            vec!["Low quality".to_string()],
            0.2,
        ),
    ];

    let result = optimizer
        .optimize_provider_selection(&request, &conflicting_data)
        .await;
    assert!(result.is_ok()); // Should resolve conflicts gracefully
    let optimization = result.unwrap();
    assert!(optimization.confidence_score < 0.8); // Reduced confidence due to conflicts
}

// Helper functions and mock implementations

async fn create_test_optimizer() -> PerformanceOptimizer {
    let config = OptimizationConfig::default();
    PerformanceOptimizer::new(config).await.unwrap()
}

fn create_test_request(research_type: ResearchType) -> ClassifiedRequest {
    ClassifiedRequest::new(
        "Test query for optimization".to_string(),
        research_type,
        AudienceContext {
            level: "intermediate".to_string(),
            domain: "software".to_string(),
            format: "markdown".to_string(),
        },
        DomainContext {
            technology: "rust".to_string(),
            project_type: "library".to_string(),
            frameworks: vec!["tokio".to_string()],
            tags: vec!["async".to_string()],
        },
        0.8,
        vec!["test".to_string()],
    )
}

fn create_learning_data(
    learning_type: &str,
    source: &str,
    insights: Vec<String>,
    confidence: f64,
) -> LearningData {
    LearningData {
        id: Uuid::new_v4().to_string(),
        learning_type: learning_type.to_string(),
        source_data_id: source.to_string(),
        insights,
        confidence_score: confidence,
        created_at: Utc::now(),
        expires_at: None,
        metadata: HashMap::new(),
    }
}

fn create_usage_pattern(
    pattern_type: &str,
    data: &str,
    frequency: u32,
    success_rate: f64,
) -> UsagePattern {
    let mut context = HashMap::new();
    context.insert(
        "success_rate".to_string(),
        serde_json::Value::from(success_rate),
    );

    UsagePattern {
        id: Uuid::new_v4().to_string(),
        pattern_type: pattern_type.to_string(),
        data: data.to_string(),
        frequency,
        last_used: Utc::now(),
        context,
    }
}

fn create_pattern_data(
    pattern_type: &str,
    frequency: u32,
    success_rate: f64,
    context: HashMap<String, serde_json::Value>,
) -> PatternData {
    PatternData {
        id: Uuid::new_v4().to_string(),
        pattern_type: pattern_type.to_string(),
        frequency,
        success_rate,
        context,
        first_seen: Utc::now(),
        last_seen: Utc::now(),
    }
}

fn create_feedback(content_id: &str, score: f64, text: &str) -> UserFeedback {
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

fn create_test_patterns() -> Vec<UsagePattern> {
    vec![
        create_usage_pattern("query_pattern", "rust async", 10, 0.9),
        create_usage_pattern("response_preference", "detailed", 8, 0.85),
    ]
}

fn create_test_performance_metrics() -> PerformanceMetrics {
    let mut provider_success_rates = HashMap::new();
    provider_success_rates.insert("openai".to_string(), 0.85);
    provider_success_rates.insert("claude".to_string(), 0.9);

    PerformanceMetrics {
        average_response_time: Duration::from_millis(800),
        cache_hit_rate: 0.75,
        provider_success_rates,
        user_satisfaction_score: 0.8,
    }
}

fn create_test_feedback_data() -> Vec<UserFeedback> {
    vec![
        create_feedback("content1", 0.9, "Excellent"),
        create_feedback("content2", 0.8, "Good"),
        create_feedback("content3", 0.85, "Very good"),
    ]
}

// Additional structures needed for comprehensive testing
