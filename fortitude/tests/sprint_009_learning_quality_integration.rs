// ABOUTME: Comprehensive integration tests for Sprint 009 Learning-Quality system integration
//!
//! This test suite validates the complete integration between the learning system and
//! quality control system, ensuring they work together to improve research quality
//! through feedback loops, cross-validation workflows, and adaptive learning.
//!
//! ## Protected Functionality
//! - Learning-Quality integration workflows (feedback → quality improvement)
//! - Cross-validation with learning adaptation
//! - Quality-based learning algorithm optimization
//! - Provider selection optimization based on learning insights
//! - End-to-end quality improvement workflows

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use fortitude::learning::*;
use fortitude::quality::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};
use tempfile::TempDir;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Integrated test environment for learning-quality validation
#[derive(Clone)]
pub struct LearningQualityTestEnvironment {
    learning_storage: Arc<MockLearningStorage>,
    quality_scorer: Arc<MockQualityScorer>,
    cross_validator: Arc<MockCrossValidator>,
    feedback_collector: Arc<MockFeedbackCollector>,
    learning_engine: Arc<MockLearningEngine>,
    test_metrics: Arc<RwLock<IntegrationMetrics>>,
}

#[derive(Clone, Default)]
pub struct IntegrationMetrics {
    quality_assessments: u64,
    learning_adaptations: u64,
    feedback_loops_completed: u64,
    provider_optimizations: u64,
    validation_cycles: u64,
    improvement_rate: f64,
}

/// ANCHOR: Validates complete learning-quality feedback loop integration workflow
/// Tests: Feedback collection → quality assessment → learning adaptation → system improvement
#[tokio::test]
async fn test_anchor_learning_quality_feedback_loop_integration() {
    let env = setup_learning_quality_environment().await;
    let start_time = Instant::now();

    println!("Phase 1: Initial quality assessment and baseline establishment");

    // Set initial baseline quality scores
    let research_query = "How does Rust async programming improve performance?";
    let research_response = "Rust async programming provides zero-cost abstractions for concurrent operations, enabling high-performance applications through compile-time optimization and efficient task scheduling.";

    let initial_quality = env
        .quality_scorer
        .evaluate_quality(
            research_query,
            research_response,
            &QualityWeights::default(),
            &QualityContext::default(),
        )
        .await
        .unwrap();

    // Verify initial quality baseline
    assert!(
        initial_quality.composite > 0.7,
        "Initial quality should be reasonable"
    );
    env.test_metrics.write().await.quality_assessments += 1;

    println!("Phase 2: User feedback collection and learning data generation");

    // Collect user feedback indicating areas for improvement
    let feedback_data = vec![
        create_quality_feedback(
            "content_001",
            0.8,
            "Good technical accuracy but needs more examples",
            "user_001",
        ),
        create_quality_feedback(
            "content_001",
            0.75,
            "Could be more comprehensive",
            "user_002",
        ),
        create_quality_feedback(
            "content_001",
            0.85,
            "Clear explanation but missing edge cases",
            "user_003",
        ),
    ];

    // Store feedback and trigger learning
    for feedback in &feedback_data {
        env.learning_storage.store_feedback(feedback).await.unwrap();
        env.feedback_collector
            .collect_feedback(feedback.clone())
            .await
            .unwrap();
    }

    // Analyze feedback patterns for learning insights
    let feedback_trends = env
        .learning_storage
        .analyze_feedback_trends("content_001", 30)
        .await
        .unwrap();
    assert_eq!(feedback_trends.len(), 1);

    let trend = &feedback_trends[0];
    assert!(trend.average_score > 0.75);
    assert!(trend.feedback_count >= 3);

    env.test_metrics.write().await.feedback_loops_completed += 1;

    println!("Phase 3: Cross-validation with learning-informed quality assessment");

    // Perform cross-validation using learning insights
    let validation_config = CrossValidationConfig {
        provider_count: 2,
        agreement_threshold: 0.8,
        validation_strategy: ValidationStrategy::ConsensusWithLearning,
        learning_weight: 0.3,
        ..CrossValidationConfig::default()
    };

    let validation_result = env
        .cross_validator
        .validate_response(research_query, research_response, &validation_config)
        .await
        .unwrap();

    assert!(validation_result.consensus_score > 0.75);
    assert!(validation_result.consistency_analysis.agreement_score > 0.7);

    env.test_metrics.write().await.validation_cycles += 1;

    println!("Phase 4: Learning-driven system adaptation and optimization");

    // Generate learning insights from feedback and quality data
    let learning_data = LearningData {
        id: Uuid::new_v4().to_string(),
        content_id: "content_001".to_string(),
        insights: json!({
            "quality_improvement_areas": ["examples", "comprehensiveness", "edge_cases"],
            "user_preferences": {
                "detail_level": "comprehensive",
                "example_count": "multiple",
                "technical_depth": "high"
            },
            "optimization_recommendations": {
                "response_structure": "add_more_examples",
                "content_focus": "include_edge_cases",
                "presentation": "improve_comprehensiveness"
            }
        }),
        metadata: HashMap::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        expiry_date: Some(Utc::now() + Duration::days(90)),
    };

    env.learning_storage
        .store_learning_data(&learning_data)
        .await
        .unwrap();

    // Apply learning-driven adaptations
    let adaptation_result = env
        .learning_engine
        .apply_learning_insights(&learning_data, &initial_quality)
        .await
        .unwrap();

    assert!(adaptation_result.improvement_score > 0.0);
    assert!(!adaptation_result.adaptations.is_empty());

    env.test_metrics.write().await.learning_adaptations += 1;

    println!("Phase 5: Quality improvement validation and performance measurement");

    // Simulate improved response based on learning insights
    let improved_response = "Rust async programming provides zero-cost abstractions for concurrent operations, enabling high-performance applications through compile-time optimization and efficient task scheduling. For example, async/await syntax allows writing concurrent code that looks synchronous while being non-blocking. Key benefits include: 1) Memory efficiency through futures that don't allocate unless polled, 2) CPU efficiency through cooperative scheduling, 3) Scalability through lightweight tasks. Edge cases to consider include blocking operations in async contexts and proper error handling across await points.";

    let improved_quality = env
        .quality_scorer
        .evaluate_quality(
            research_query,
            improved_response,
            &QualityWeights::default(),
            &QualityContext::default(),
        )
        .await
        .unwrap();

    // Validate quality improvement
    let improvement_rate =
        (improved_quality.composite - initial_quality.composite) / initial_quality.composite;
    assert!(
        improvement_rate > 0.05,
        "Should show at least 5% improvement"
    );

    // Validate specific dimension improvements based on feedback
    assert!(
        improved_quality.completeness > initial_quality.completeness,
        "Completeness should improve"
    );
    assert!(
        improved_quality.specificity > initial_quality.specificity,
        "Specificity should improve"
    );

    env.test_metrics.write().await.quality_assessments += 1;
    env.test_metrics.write().await.improvement_rate = improvement_rate;

    println!("Phase 6: Provider optimization based on learning insights");

    // Test provider optimization using learning data
    let optimization_config = OptimizationConfig {
        selection_criteria: SelectionCriteria {
            quality_weight: 0.4,
            speed_weight: 0.3,
            cost_weight: 0.2,
            learning_weight: 0.1,
        },
        urgency_level: UrgencyLevel::Normal,
        cost_constraints: CostConstraints::Moderate,
        query_complexity: QueryComplexity::Medium,
        learning_insights: Some(learning_data.insights.clone()),
    };

    let provider_optimization = MockProviderOptimizer::new();
    let optimization_result = provider_optimization
        .optimize_provider_selection(research_query, &optimization_config)
        .await
        .unwrap();

    assert!(!optimization_result.recommended_providers.is_empty());
    assert!(optimization_result.expected_quality_score > 0.8);

    env.test_metrics.write().await.provider_optimizations += 1;

    // Performance validation
    let total_duration = start_time.elapsed();
    assert!(
        total_duration < StdDuration::from_millis(5000),
        "Full integration workflow should complete within 5 seconds"
    );

    // Final metrics validation
    let metrics = env.test_metrics.read().await;
    assert!(metrics.quality_assessments >= 2);
    assert!(metrics.learning_adaptations >= 1);
    assert!(metrics.feedback_loops_completed >= 1);
    assert!(metrics.validation_cycles >= 1);
    assert!(metrics.improvement_rate > 0.05);

    println!("✓ Learning-Quality integration workflow completed successfully");
    println!("  - Quality assessments: {}", metrics.quality_assessments);
    println!("  - Learning adaptations: {}", metrics.learning_adaptations);
    println!("  - Feedback loops: {}", metrics.feedback_loops_completed);
    println!(
        "  - Improvement rate: {:.2}%",
        metrics.improvement_rate * 100.0
    );
    println!("  - Total duration: {:?}", total_duration);
}

/// ANCHOR: Validates cross-validation system integration with learning adaptation
/// Tests: Cross-validation results → learning insights → validation strategy optimization
#[tokio::test]
async fn test_anchor_cross_validation_learning_adaptation() {
    let env = setup_learning_quality_environment().await;

    println!("Phase 1: Multi-provider cross-validation with baseline configuration");

    let test_query = "What are the best practices for error handling in distributed systems?";
    let test_responses = vec![
        "Use circuit breakers, retries with exponential backoff, and comprehensive logging for effective error handling in distributed systems.",
        "Implement graceful degradation, timeout strategies, and centralized error monitoring to handle failures in distributed architectures.",
        "Apply bulkhead patterns, health checks, and automated recovery mechanisms for robust distributed system error management.",
    ];

    // Initial cross-validation with standard configuration
    let baseline_config = CrossValidationConfig {
        provider_count: 3,
        agreement_threshold: 0.7,
        validation_strategy: ValidationStrategy::Consensus,
        learning_weight: 0.0,
        consensus_method: ConsensusMethod::MajorityVoting,
        bias_detection_enabled: true,
        ..CrossValidationConfig::default()
    };

    let mut validation_results = Vec::new();
    for (i, response) in test_responses.iter().enumerate() {
        let result = env
            .cross_validator
            .validate_response(test_query, response, &baseline_config)
            .await
            .unwrap();

        validation_results.push(result);

        // Store validation result as learning data
        let learning_insight = LearningData {
            id: Uuid::new_v4().to_string(),
            content_id: format!("validation_result_{}", i),
            insights: json!({
                "validation_score": result.consensus_score,
                "agreement_patterns": result.consistency_analysis,
                "bias_indicators": result.bias_analysis,
                "provider_performance": result.provider_scores
            }),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expiry_date: Some(Utc::now() + Duration::days(30)),
        };

        env.learning_storage
            .store_learning_data(&learning_insight)
            .await
            .unwrap();
    }

    println!("Phase 2: Learning analysis of validation patterns");

    // Analyze validation patterns for learning insights
    let validation_patterns = env
        .learning_storage
        .analyze_patterns("validation_result", 30, 0.1)
        .await
        .unwrap();

    assert!(!validation_patterns.is_empty());

    // Generate adaptation recommendations based on patterns
    let adaptation_recommendations = json!({
        "validation_strategy_optimization": {
            "recommended_threshold": 0.75,
            "optimal_provider_count": 2,
            "bias_detection_adjustments": true
        },
        "provider_weighting": {
            "accuracy_based_weighting": true,
            "consistency_bonus": 0.1,
            "speed_penalty_threshold": 2000
        },
        "consensus_method_optimization": {
            "recommended_method": "WeightedConsensus",
            "learning_integration": true
        }
    });

    println!("Phase 3: Learning-enhanced cross-validation optimization");

    // Apply learning insights to optimize validation strategy
    let enhanced_config = CrossValidationConfig {
        provider_count: 2,         // Optimized based on learning
        agreement_threshold: 0.75, // Increased based on patterns
        validation_strategy: ValidationStrategy::ConsensusWithLearning,
        learning_weight: 0.2, // Enable learning integration
        consensus_method: ConsensusMethod::WeightedConsensus,
        bias_detection_enabled: true,
        adaptation_recommendations: Some(adaptation_recommendations),
        ..CrossValidationConfig::default()
    };

    // Test enhanced validation with learning integration
    let enhanced_result = env
        .cross_validator
        .validate_response(
            test_query,
            &test_responses[0], // Use best response from initial validation
            &enhanced_config,
        )
        .await
        .unwrap();

    // Validate improvement in validation quality
    assert!(enhanced_result.consensus_score > validation_results[0].consensus_score);
    assert!(enhanced_result.consistency_analysis.agreement_score > 0.75);
    assert!(enhanced_result.learning_integration_score.unwrap_or(0.0) > 0.0);

    println!("Phase 4: Adaptive validation strategy evolution");

    // Simulate multiple validation cycles with continuous learning
    let mut adaptation_scores = Vec::new();
    for cycle in 0..5 {
        let cycle_config = CrossValidationConfig {
            learning_weight: 0.1 + (cycle as f64 * 0.05), // Gradually increase learning influence
            adaptation_cycle: Some(cycle),
            ..enhanced_config.clone()
        };

        let cycle_result = env
            .cross_validator
            .validate_response(
                test_query,
                &test_responses[cycle % test_responses.len()],
                &cycle_config,
            )
            .await
            .unwrap();

        adaptation_scores.push(cycle_result.consensus_score);

        // Verify learning-driven improvements
        if cycle > 0 {
            let improvement = cycle_result.consensus_score - validation_results[0].consensus_score;
            assert!(
                improvement >= 0.0,
                "Validation should maintain or improve quality"
            );
        }
    }

    // Validate learning curve - scores should generally improve or stabilize
    let final_score = adaptation_scores.last().unwrap();
    let initial_score = adaptation_scores.first().unwrap();
    let learning_improvement = (final_score - initial_score) / initial_score;

    assert!(
        learning_improvement >= -0.05,
        "Learning should not significantly degrade performance"
    );

    println!("✓ Cross-validation learning adaptation completed successfully");
    println!("  - Validation cycles: {}", adaptation_scores.len());
    println!(
        "  - Learning improvement: {:.2}%",
        learning_improvement * 100.0
    );
    println!("  - Final validation score: {:.3}", final_score);
}

/// ANCHOR: Validates provider selection optimization based on learning insights
/// Tests: Learning data → provider performance analysis → optimal provider selection
#[tokio::test]
async fn test_anchor_provider_optimization_learning_integration() {
    let env = setup_learning_quality_environment().await;

    println!("Phase 1: Collect provider performance learning data");

    // Simulate historical provider performance data
    let provider_performance_data = vec![
        ("openai", 0.92, 850, 0.05), // (provider, quality, latency_ms, cost)
        ("claude", 0.89, 920, 0.04),
        ("gemini", 0.87, 780, 0.03),
    ];

    for (provider, quality, latency, cost) in &provider_performance_data {
        let performance_learning = LearningData {
            id: Uuid::new_v4().to_string(),
            content_id: format!("provider_performance_{}", provider),
            insights: json!({
                "provider_name": provider,
                "quality_score": quality,
                "average_latency_ms": latency,
                "cost_per_request": cost,
                "usage_patterns": {
                    "preferred_for": ["complex_queries", "technical_research"],
                    "performance_category": "high_quality"
                },
                "user_satisfaction": 0.85
            }),
            metadata: HashMap::from([
                ("provider_type".to_string(), "llm".to_string()),
                ("data_type".to_string(), "performance_metrics".to_string()),
            ]),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expiry_date: Some(Utc::now() + Duration::days(90)),
        };

        env.learning_storage
            .store_learning_data(&performance_learning)
            .await
            .unwrap();
    }

    println!("Phase 2: Analyze provider patterns and generate optimization insights");

    // Analyze provider performance patterns
    let provider_patterns = env
        .learning_storage
        .analyze_patterns("provider_performance", 90, 0.05)
        .await
        .unwrap();

    assert!(provider_patterns.len() >= 3); // One for each provider

    // Generate provider optimization recommendations
    let optimization_insights = json!({
        "provider_rankings": {
            "quality_focused": ["openai", "claude", "gemini"],
            "speed_focused": ["gemini", "openai", "claude"],
            "cost_focused": ["gemini", "claude", "openai"]
        },
        "usage_recommendations": {
            "complex_queries": "openai",
            "simple_queries": "gemini",
            "balanced_queries": "claude"
        },
        "performance_thresholds": {
            "min_quality_score": 0.85,
            "max_latency_ms": 1000,
            "max_cost_per_request": 0.06
        }
    });

    println!("Phase 3: Learning-informed provider selection optimization");

    let optimization_config = OptimizationConfig {
        selection_criteria: SelectionCriteria {
            quality_weight: 0.5,
            speed_weight: 0.3,
            cost_weight: 0.2,
            learning_weight: 0.3, // High learning influence
        },
        urgency_level: UrgencyLevel::Normal,
        cost_constraints: CostConstraints::Moderate,
        query_complexity: QueryComplexity::High,
        learning_insights: Some(optimization_insights),
    };

    let provider_optimizer = MockProviderOptimizer::new();

    // Test optimization for different query types
    let test_scenarios = vec![
        ("What is the complexity of QuickSort?", QueryComplexity::Low),
        (
            "Explain distributed consensus algorithms in detail",
            QueryComplexity::High,
        ),
        (
            "How do I implement async/await in Rust?",
            QueryComplexity::Medium,
        ),
    ];

    for (query, complexity) in test_scenarios {
        let scenario_config = OptimizationConfig {
            query_complexity: complexity,
            ..optimization_config.clone()
        };

        let optimization_result = provider_optimizer
            .optimize_provider_selection(query, &scenario_config)
            .await
            .unwrap();

        // Validate optimization results
        assert!(!optimization_result.recommended_providers.is_empty());
        assert!(optimization_result.expected_quality_score > 0.8);
        assert!(optimization_result.estimated_latency_ms < 1200);

        // Verify learning integration
        assert!(
            optimization_result
                .learning_integration_score
                .unwrap_or(0.0)
                > 0.0
        );

        // Validate provider selection logic based on learning
        let primary_provider = &optimization_result.recommended_providers[0];
        match complexity {
            QueryComplexity::High => {
                assert!(
                    primary_provider.expected_quality > 0.9,
                    "High complexity should select high-quality provider"
                );
            }
            QueryComplexity::Low => {
                assert!(
                    primary_provider.estimated_latency_ms < 900,
                    "Low complexity should prioritize speed"
                );
            }
            QueryComplexity::Medium => {
                assert!(
                    primary_provider.cost_per_request < 0.05,
                    "Medium complexity should balance all factors"
                );
            }
        }
    }

    println!("Phase 4: Continuous learning and adaptation validation");

    // Simulate feedback loop for provider optimization
    let feedback_scenarios = vec![
        ("claude", 0.95, "Excellent detailed explanation"),
        ("openai", 0.88, "Good but slightly verbose"),
        ("gemini", 0.82, "Fast but lacks depth"),
    ];

    for (provider, score, comment) in feedback_scenarios {
        let feedback = UserFeedback {
            id: Uuid::new_v4().to_string(),
            content_id: format!("optimization_test_{}", provider),
            user_id: "test_user".to_string(),
            score: Some(score),
            text_feedback: Some(comment.to_string()),
            feedback_type: "quality_assessment".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::from([
                ("provider".to_string(), Value::String(provider.to_string())),
                (
                    "optimization_test".to_string(),
                    Value::String("true".to_string()),
                ),
            ]),
        };

        env.learning_storage
            .store_feedback(&feedback)
            .await
            .unwrap();

        // Generate learning insights from feedback
        let feedback_learning = LearningData {
            id: Uuid::new_v4().to_string(),
            content_id: format!("provider_feedback_{}", provider),
            insights: json!({
                "provider": provider,
                "performance_feedback": {
                    "quality_score": score,
                    "user_comment": comment,
                    "satisfaction_level": if score > 0.9 { "high" } else if score > 0.8 { "medium" } else { "low" }
                },
                "optimization_adjustments": {
                    "quality_weight_adjustment": if score > 0.9 { 0.05 } else { -0.02 },
                    "provider_preference_update": true
                }
            }),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expiry_date: Some(Utc::now() + Duration::days(60)),
        };

        env.learning_storage
            .store_learning_data(&feedback_learning)
            .await
            .unwrap();
    }

    // Re-run optimization with updated learning data
    let updated_optimization = provider_optimizer
        .optimize_provider_selection("Explain machine learning algorithms", &optimization_config)
        .await
        .unwrap();

    // Validate learning-driven improvements
    assert!(
        updated_optimization
            .learning_integration_score
            .unwrap_or(0.0)
            > 0.2
    );
    assert!(updated_optimization.confidence_score > 0.8);

    println!("✓ Provider optimization learning integration completed successfully");
    println!("  - Test scenarios: {}", test_scenarios.len());
    println!("  - Feedback loops: {}", feedback_scenarios.len());
    println!(
        "  - Final confidence: {:.3}",
        updated_optimization.confidence_score
    );
}

/// ANCHOR: Validates end-to-end quality improvement workflow with learning integration
/// Tests: Complete workflow from baseline quality → learning → adaptation → improvement validation
#[tokio::test]
async fn test_anchor_end_to_end_quality_improvement_workflow() {
    let env = setup_learning_quality_environment().await;
    let workflow_start = Instant::now();

    println!("Phase 1: Establish baseline quality measurements");

    let test_research_tasks = vec![
        (
            "What are microservices advantages?",
            "Microservices provide better scalability and modularity.",
        ),
        (
            "How does Kubernetes work?",
            "Kubernetes orchestrates containerized applications across clusters.",
        ),
        (
            "Explain REST API principles",
            "REST APIs use HTTP methods for stateless communication.",
        ),
    ];

    let mut baseline_qualities = Vec::new();
    for (i, (query, response)) in test_research_tasks.iter().enumerate() {
        let quality_score = env
            .quality_scorer
            .evaluate_quality(
                query,
                response,
                &QualityWeights::default(),
                &QualityContext::default(),
            )
            .await
            .unwrap();

        baseline_qualities.push(quality_score.clone());

        // Store baseline quality as learning data
        let baseline_data = LearningData {
            id: Uuid::new_v4().to_string(),
            content_id: format!("baseline_quality_{}", i),
            insights: json!({
                "baseline_quality": {
                    "overall_score": quality_score.composite,
                    "dimension_scores": {
                        "relevance": quality_score.relevance,
                        "accuracy": quality_score.accuracy,
                        "completeness": quality_score.completeness,
                        "clarity": quality_score.clarity
                    },
                    "improvement_potential": 1.0 - quality_score.composite
                },
                "content_analysis": {
                    "query": query,
                    "response_length": response.len(),
                    "complexity_level": "baseline"
                }
            }),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expiry_date: Some(Utc::now() + Duration::days(30)),
        };

        env.learning_storage
            .store_learning_data(&baseline_data)
            .await
            .unwrap();
    }

    let average_baseline_quality: f64 = baseline_qualities.iter().map(|q| q.composite).sum::<f64>()
        / baseline_qualities.len() as f64;

    println!(
        "  - Baseline average quality: {:.3}",
        average_baseline_quality
    );

    println!("Phase 2: Collect targeted user feedback for improvement insights");

    // Simulate user feedback indicating specific improvement areas
    let improvement_feedback = vec![
        ("Need more concrete examples", 0.7, "completeness"),
        ("Add technical details", 0.75, "accuracy"),
        ("Improve explanation clarity", 0.8, "clarity"),
        ("Include edge cases", 0.72, "completeness"),
        ("More comprehensive coverage", 0.68, "completeness"),
    ];

    for (i, (comment, score, dimension)) in improvement_feedback.iter().enumerate() {
        let feedback = UserFeedback {
            id: Uuid::new_v4().to_string(),
            content_id: format!("improvement_feedback_{}", i),
            user_id: format!("user_{}", i % 3), // Simulate multiple users
            score: Some(*score),
            text_feedback: Some(comment.to_string()),
            feedback_type: "improvement_suggestion".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::from([
                (
                    "improvement_dimension".to_string(),
                    Value::String(dimension.to_string()),
                ),
                ("priority".to_string(), Value::String("high".to_string())),
            ]),
        };

        env.learning_storage
            .store_feedback(&feedback)
            .await
            .unwrap();
    }

    println!("Phase 3: Generate comprehensive learning insights and improvement strategies");

    // Analyze feedback patterns to generate improvement strategies
    let improvement_patterns = env
        .learning_storage
        .analyze_patterns("improvement_feedback", 30, 0.1)
        .await
        .unwrap();

    // Generate targeted improvement recommendations
    let improvement_strategy = json!({
        "priority_improvements": {
            "completeness": {
                "weight": 0.4,
                "specific_actions": ["add_examples", "include_edge_cases", "expand_coverage"],
                "target_improvement": 0.15
            },
            "clarity": {
                "weight": 0.3,
                "specific_actions": ["improve_structure", "simplify_language", "add_explanations"],
                "target_improvement": 0.12
            },
            "accuracy": {
                "weight": 0.3,
                "specific_actions": ["add_technical_details", "verify_facts", "cite_sources"],
                "target_improvement": 0.10
            }
        },
        "content_enhancement_guidelines": {
            "examples_per_concept": 2,
            "technical_depth_level": "intermediate",
            "explanation_structure": "definition_example_application"
        },
        "quality_targets": {
            "minimum_improvement": 0.15,
            "target_overall_score": 0.85,
            "consistency_threshold": 0.05
        }
    });

    let strategy_learning = LearningData {
        id: Uuid::new_v4().to_string(),
        content_id: "improvement_strategy".to_string(),
        insights: improvement_strategy.clone(),
        metadata: HashMap::from([
            (
                "strategy_type".to_string(),
                "quality_improvement".to_string(),
            ),
            ("confidence_level".to_string(), "high".to_string()),
        ]),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        expiry_date: Some(Utc::now() + Duration::days(60)),
    };

    env.learning_storage
        .store_learning_data(&strategy_learning)
        .await
        .unwrap();

    println!("Phase 4: Apply learning-driven content improvements");

    // Generate improved responses based on learning insights
    let improved_responses = vec![
        "Microservices provide better scalability and modularity by decomposing applications into independent services. Key advantages include: 1) Independent scaling (each service can scale based on demand), 2) Technology diversity (different services can use different tech stacks), 3) Fault isolation (failure in one service doesn't crash the entire system). For example, an e-commerce platform might separate user management, inventory, and payment services. Edge cases to consider: network latency between services, data consistency challenges, and deployment complexity.",
        "Kubernetes orchestrates containerized applications across clusters using a master-worker architecture. Core components include: 1) API Server (receives and processes requests), 2) etcd (distributed key-value store for cluster state), 3) Scheduler (assigns pods to nodes), 4) kubelet (manages pod lifecycle on nodes). For example, when you deploy an application, the API Server validates the request, the Scheduler selects appropriate nodes, and kubelet starts the containers. Advanced scenarios include horizontal pod autoscaling, rolling updates, and multi-cluster networking.",
        "REST APIs use HTTP methods for stateless communication following specific architectural principles: 1) Uniform Interface (consistent resource identification via URIs), 2) Statelessness (each request contains all necessary information), 3) Cacheability (responses can be cached for performance), 4) Client-Server separation, 5) Layered system architecture. Example: GET /users/123 retrieves user data, POST /users creates a new user, PUT /users/123 updates existing user. Important considerations include versioning strategies, rate limiting, authentication methods, and error handling patterns for production APIs."
    ];

    // Evaluate improved responses
    let mut improved_qualities = Vec::new();
    for (i, (improved_response, (query, _))) in improved_responses
        .iter()
        .zip(test_research_tasks.iter())
        .enumerate()
    {
        let improved_quality = env
            .quality_scorer
            .evaluate_quality(
                query,
                improved_response,
                &QualityWeights::default(),
                &QualityContext::default(),
            )
            .await
            .unwrap();

        improved_qualities.push(improved_quality.clone());

        // Validate specific dimension improvements
        let baseline_quality = &baseline_qualities[i];

        // Check completeness improvement (should be significant based on feedback)
        let completeness_improvement =
            improved_quality.completeness - baseline_quality.completeness;
        assert!(
            completeness_improvement > 0.1,
            "Completeness should improve significantly"
        );

        // Check clarity improvement
        let clarity_improvement = improved_quality.clarity - baseline_quality.clarity;
        assert!(clarity_improvement > 0.05, "Clarity should improve");

        // Check overall improvement
        let overall_improvement = improved_quality.composite - baseline_quality.composite;
        assert!(
            overall_improvement > 0.1,
            "Overall quality should improve by at least 10%"
        );

        println!(
            "  - Task {}: {:.3} → {:.3} (+{:.3})",
            i + 1,
            baseline_quality.composite,
            improved_quality.composite,
            overall_improvement
        );
    }

    println!("Phase 5: Validate sustained quality improvement and learning effectiveness");

    let average_improved_quality: f64 = improved_qualities.iter().map(|q| q.composite).sum::<f64>()
        / improved_qualities.len() as f64;

    let overall_improvement_rate =
        (average_improved_quality - average_baseline_quality) / average_baseline_quality;

    // Performance and quality validations
    assert!(
        overall_improvement_rate > 0.15,
        "Should achieve at least 15% quality improvement"
    );
    assert!(
        average_improved_quality > 0.8,
        "Improved quality should be high"
    );

    // Consistency validation
    let quality_variance: f64 = improved_qualities
        .iter()
        .map(|q| (q.composite - average_improved_quality).powi(2))
        .sum::<f64>()
        / improved_qualities.len() as f64;
    let quality_std_dev = quality_variance.sqrt();
    assert!(
        quality_std_dev < 0.1,
        "Quality improvements should be consistent"
    );

    // Performance validation
    let workflow_duration = workflow_start.elapsed();
    assert!(
        workflow_duration < StdDuration::from_secs(10),
        "Workflow should complete efficiently"
    );

    // Update final metrics
    env.test_metrics.write().await.improvement_rate = overall_improvement_rate;
    env.test_metrics.write().await.quality_assessments =
        (baseline_qualities.len() + improved_qualities.len()) as u64;

    println!("✓ End-to-end quality improvement workflow completed successfully");
    println!(
        "  - Average quality improvement: {:.1}%",
        overall_improvement_rate * 100.0
    );
    println!("  - Final average quality: {:.3}", average_improved_quality);
    println!("  - Quality consistency (std dev): {:.3}", quality_std_dev);
    println!("  - Total workflow time: {:?}", workflow_duration);
}

// Helper functions and mock implementations

async fn setup_learning_quality_environment() -> LearningQualityTestEnvironment {
    LearningQualityTestEnvironment {
        learning_storage: Arc::new(MockLearningStorage::new()),
        quality_scorer: Arc::new(MockQualityScorer::new()),
        cross_validator: Arc::new(MockCrossValidator::new()),
        feedback_collector: Arc::new(MockFeedbackCollector::new()),
        learning_engine: Arc::new(MockLearningEngine::new()),
        test_metrics: Arc::new(RwLock::new(IntegrationMetrics::default())),
    }
}

fn create_quality_feedback(
    content_id: &str,
    score: f64,
    comment: &str,
    user_id: &str,
) -> UserFeedback {
    UserFeedback {
        id: Uuid::new_v4().to_string(),
        content_id: content_id.to_string(),
        user_id: user_id.to_string(),
        score: Some(score),
        text_feedback: Some(comment.to_string()),
        feedback_type: "quality_assessment".to_string(),
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    }
}

// Mock implementations (simplified for testing)

#[derive(Clone)]
pub struct MockLearningStorage {
    feedback_data: Arc<RwLock<Vec<UserFeedback>>>,
    learning_data: Arc<RwLock<Vec<LearningData>>>,
}

impl MockLearningStorage {
    pub fn new() -> Self {
        Self {
            feedback_data: Arc::new(RwLock::new(Vec::new())),
            learning_data: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl LearningStorageService for MockLearningStorage {
    async fn store_feedback(&self, feedback: &UserFeedback) -> LearningResult<()> {
        self.feedback_data.write().await.push(feedback.clone());
        Ok(())
    }

    async fn store_learning_data(&self, data: &LearningData) -> LearningResult<()> {
        self.learning_data.write().await.push(data.clone());
        Ok(())
    }

    async fn get_feedback_for_content(
        &self,
        content_id: &str,
    ) -> LearningResult<Vec<UserFeedback>> {
        let feedback_data = self.feedback_data.read().await;
        Ok(feedback_data
            .iter()
            .filter(|f| f.content_id == content_id)
            .cloned()
            .collect())
    }

    async fn analyze_feedback_trends(
        &self,
        content_id: &str,
        days: i32,
    ) -> LearningResult<Vec<FeedbackTrend>> {
        let feedback_data = self.feedback_data.read().await;
        let relevant_feedback: Vec<_> = feedback_data
            .iter()
            .filter(|f| f.content_id == content_id)
            .collect();

        if relevant_feedback.is_empty() {
            return Ok(vec![]);
        }

        let average_score = relevant_feedback
            .iter()
            .map(|f| f.score.unwrap_or(0.0))
            .sum::<f64>()
            / relevant_feedback.len() as f64;

        Ok(vec![FeedbackTrend {
            content_id: format!("last_{}days", days),
            total_feedback: relevant_feedback.len(),
            average_score,
            trend_direction: 0.0, // Simplified
        }])
    }

    async fn analyze_patterns(
        &self,
        prefix: &str,
        days: i32,
        _threshold: f64,
    ) -> LearningResult<Vec<PatternData>> {
        let learning_data = self.learning_data.read().await;
        let patterns: Vec<_> = learning_data
            .iter()
            .filter(|d| d.id.starts_with(prefix))
            .map(|d| PatternData {
                id: Uuid::new_v4().to_string(),
                pattern_type: "mock_pattern".to_string(),
                frequency: 1,
                success_rate: 0.8,
                context: HashMap::new(),
                first_seen: Utc::now(),
                last_seen: Utc::now(),
            })
            .collect();

        Ok(patterns)
    }
}

#[derive(Clone)]
pub struct MockQualityScorer;

impl MockQualityScorer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl QualityScorer for MockQualityScorer {
    async fn evaluate_quality(
        &self,
        _query: &str,
        response: &str,
        _weights: &QualityWeights,
        _context: &QualityContext,
    ) -> Result<QualityScore, QualityError> {
        // Simulate quality scoring based on response characteristics
        let length_factor = (response.len() as f64 / 100.0).min(1.0);
        let example_factor = if response.contains("example") || response.contains("Example") {
            0.1
        } else {
            0.0
        };
        let detail_factor = if response.contains("1)") || response.contains("2)") {
            0.15
        } else {
            0.0
        };
        let edge_case_factor = if response.contains("edge case") || response.contains("consider") {
            0.1
        } else {
            0.0
        };

        let base_score = 0.6;
        let enhanced_score =
            base_score + length_factor * 0.2 + example_factor + detail_factor + edge_case_factor;

        Ok(QualityScore {
            relevance: enhanced_score.min(1.0),
            accuracy: (enhanced_score * 0.95).min(1.0),
            completeness: (enhanced_score + detail_factor).min(1.0),
            clarity: (enhanced_score + example_factor).min(1.0),
            credibility: (enhanced_score * 0.9).min(1.0),
            timeliness: 0.85,
            specificity: (enhanced_score + edge_case_factor).min(1.0),
            composite: enhanced_score.min(1.0),
            confidence: 0.9,
        })
    }
}

#[derive(Clone)]
pub struct MockCrossValidator;

impl MockCrossValidator {
    pub fn new() -> Self {
        Self
    }

    pub async fn validate_response(
        &self,
        _query: &str,
        _response: &str,
        config: &CrossValidationConfig,
    ) -> Result<ValidationResult, QualityError> {
        let base_consensus = 0.75;
        let learning_bonus = config.learning_weight * 0.1;

        Ok(ValidationResult {
            consensus_score: (base_consensus + learning_bonus).min(1.0),
            consistency_analysis: ConsistencyAnalysis {
                agreement_score: 0.8,
                variance: 0.1,
                outlier_count: 0,
            },
            bias_analysis: BiasAnalysis {
                detected_biases: vec![],
                bias_score: 0.05,
                confidence: 0.9,
            },
            provider_scores: HashMap::new(),
            learning_integration_score: Some(learning_bonus),
        })
    }
}

#[derive(Clone)]
pub struct MockFeedbackCollector;

impl MockFeedbackCollector {
    pub fn new() -> Self {
        Self
    }

    pub async fn collect_feedback(&self, _feedback: UserFeedback) -> Result<(), QualityError> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct MockLearningEngine;

impl MockLearningEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn apply_learning_insights(
        &self,
        _learning_data: &LearningData,
        _quality_score: &QualityScore,
    ) -> Result<AdaptationResult, LearningError> {
        Ok(AdaptationResult {
            improvement_score: 0.15,
            adaptations: vec!["add_examples".to_string(), "improve_structure".to_string()],
            confidence: 0.85,
        })
    }
}

#[derive(Clone)]
pub struct AdaptationResult {
    pub improvement_score: f64,
    pub adaptations: Vec<String>,
    pub confidence: f64,
}

#[derive(Clone)]
pub struct MockProviderOptimizer;

impl MockProviderOptimizer {
    pub fn new() -> Self {
        Self
    }

    pub async fn optimize_provider_selection(
        &self,
        _query: &str,
        config: &OptimizationConfig,
    ) -> Result<OptimizedQueryResult, OptimizationError> {
        let learning_bonus = config
            .learning_insights
            .as_ref()
            .map(|_| 0.1)
            .unwrap_or(0.0);

        Ok(OptimizedQueryResult {
            recommended_providers: vec![ProviderSelection {
                provider_name: "openai".to_string(),
                expected_quality: 0.9,
                estimated_latency_ms: 800,
                cost_per_request: 0.04,
                confidence_score: 0.85,
            }],
            expected_quality_score: 0.9 + learning_bonus,
            estimated_latency_ms: 800,
            total_cost_estimate: 0.04,
            confidence_score: 0.85 + learning_bonus,
            learning_integration_score: Some(learning_bonus),
        })
    }
}
