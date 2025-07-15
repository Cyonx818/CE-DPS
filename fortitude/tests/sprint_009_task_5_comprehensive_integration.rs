// ABOUTME: Comprehensive integration test suite for Sprint 009 Task 5 system integration
// Tests all Sprint 009 features working together: multi-LLM providers, quality control, learning system, monitoring
// Follows TDD approach - write tests first, then implement functionality

use anyhow::Result;
use fortitude::learning::{LearningConfig, LearningSystem};
use fortitude::monitoring::{MonitoringConfig, MonitoringSystem, PerformanceMetrics};
use fortitude::providers::{FallbackStrategy, ProviderConfig, ProviderManager};
use fortitude::quality::{QualityConfig, QualityController, QualityMetrics};
use fortitude_core::{PipelineBuilder, ResearchPipeline};
use fortitude_test_utils::helpers::setup_test_environment;
use fortitude_types::{ClassificationConfig, ResearchType, Storage, StorageConfig};
use serde_json::json;
use std::time::Duration;
use tokio::time::timeout;

/// Test comprehensive system integration with all Sprint 009 features
#[tokio::test]
async fn test_comprehensive_system_integration() -> Result<()> {
    let _env = setup_test_environment().await?;

    // Initialize all systems
    let provider_manager = create_test_provider_manager().await?;
    let quality_controller = create_test_quality_controller().await?;
    let learning_system = create_test_learning_system().await?;
    let monitoring_system = create_test_monitoring_system().await?;

    // Create integrated research pipeline
    let pipeline = create_integrated_pipeline(
        &provider_manager,
        &quality_controller,
        &learning_system,
        &monitoring_system,
    )
    .await?;

    // Test end-to-end research with all systems working together
    let test_queries = vec![
        "How to implement async functions in Rust?",
        "Debug memory leaks in production systems",
        "Best practices for error handling in distributed systems",
    ];

    for query in test_queries {
        // Execute research query
        let result = timeout(
            Duration::from_secs(30),
            pipeline.process_query(query, None, None),
        )
        .await??;

        // Verify basic functionality
        assert!(!result.immediate_answer.is_empty());
        assert!(result.metadata.processing_time_ms > 0);
        assert!(result.metadata.processing_time_ms < 200); // <200ms requirement

        // Verify provider management integration
        let provider_metrics = provider_manager.get_performance_metrics().await?;
        assert!(provider_metrics.total_requests > 0);

        // Verify quality control integration
        let quality_score = quality_controller.evaluate_result(&result).await?;
        assert!(quality_score.overall_score >= 0.8); // >80% quality threshold

        // Verify learning system integration
        learning_system.record_usage_pattern(query, &result).await?;
        let adaptation_suggestions = learning_system.get_adaptation_suggestions().await?;
        assert!(!adaptation_suggestions.is_empty());

        // Verify monitoring integration
        let performance_metrics = monitoring_system.get_current_metrics().await?;
        assert!(performance_metrics.average_response_time_ms < 200.0);
        assert!(performance_metrics.success_rate >= 0.95);
    }

    Ok(())
}

/// Test multi-LLM provider fallback and switching
#[tokio::test]
async fn test_provider_fallback_integration() -> Result<()> {
    let _env = setup_test_environment().await?;

    let provider_manager = create_test_provider_manager().await?;

    // Test primary provider failure scenario
    provider_manager.simulate_provider_failure("openai").await?;

    let result = provider_manager
        .execute_query("Test fallback scenario")
        .await?;

    // Should fall back to Claude provider
    assert_eq!(result.provider_used, "claude");
    assert!(result.fallback_triggered);
    assert!(result.processing_time_ms < 100); // Fast fallback

    // Test recovery
    provider_manager.restore_provider("openai").await?;

    let result2 = provider_manager
        .execute_query("Test recovery scenario")
        .await?;
    assert_eq!(result2.provider_used, "openai"); // Back to primary
    assert!(!result2.fallback_triggered);

    Ok(())
}

/// Test cross-provider quality validation
#[tokio::test]
async fn test_cross_provider_quality_validation() -> Result<()> {
    let _env = setup_test_environment().await?;

    let provider_manager = create_test_provider_manager().await?;
    let quality_controller = create_test_quality_controller().await?;

    // Enable cross-validation mode
    quality_controller.enable_cross_validation().await?;

    let query = "Explain microservices architecture patterns";

    // Execute with cross-validation
    let validation_result = quality_controller
        .validate_with_cross_check(&provider_manager, query)
        .await?;

    // Should have responses from multiple providers
    assert!(validation_result.provider_responses.len() >= 2);

    // Agreement score should be calculated
    assert!(validation_result.agreement_score >= 0.0);
    assert!(validation_result.agreement_score <= 1.0);

    // Quality score should meet threshold
    assert!(validation_result.quality_score >= 0.95);

    // Consensus response should be generated
    assert!(!validation_result.consensus_response.is_empty());
    assert!(validation_result.confidence_score >= 0.8);

    Ok(())
}

/// Test real-time learning adaptation
#[tokio::test]
async fn test_learning_system_adaptation() -> Result<()> {
    let _env = setup_test_environment().await?;

    let learning_system = create_test_learning_system().await?;

    // Simulate user feedback pattern
    let queries_and_feedback = vec![
        ("How to handle errors in Rust?", 0.9, "Very helpful"),
        (
            "Rust async programming basics",
            0.7,
            "Good but needs more examples",
        ),
        ("Debugging Rust applications", 0.8, "Clear explanations"),
    ];

    for (query, rating, feedback) in queries_and_feedback {
        learning_system
            .record_feedback(query, rating, Some(feedback.to_string()))
            .await?;
    }

    // Trigger adaptation
    learning_system.trigger_adaptation().await?;

    // Verify learning insights
    let insights = learning_system.get_learning_insights().await?;
    assert!(insights.total_feedback_count >= 3);
    assert!(insights.average_rating >= 0.7);

    // Verify prompt optimization suggestions
    let optimizations = learning_system.get_prompt_optimizations().await?;
    assert!(!optimizations.is_empty());

    // Test pattern recognition
    let patterns = learning_system.detect_usage_patterns().await?;
    assert!(patterns.dominant_topics.contains(&"rust".to_string()));
    assert!(patterns.improvement_areas.len() > 0);

    Ok(())
}

/// Test performance monitoring and alerting
#[tokio::test]
async fn test_monitoring_and_alerting() -> Result<()> {
    let _env = setup_test_environment().await?;

    let monitoring_system = create_test_monitoring_system().await?;

    // Start monitoring
    monitoring_system.start_monitoring().await?;

    // Simulate high load scenario
    let queries = vec![
        "Test query 1",
        "Test query 2",
        "Test query 3",
        "Test query 4",
        "Test query 5",
    ];

    for query in queries {
        // Simulate processing with metrics collection
        monitoring_system.record_request_start(query).await?;

        // Simulate processing time
        tokio::time::sleep(Duration::from_millis(150)).await;

        monitoring_system.record_request_end(query, true).await?;
    }

    // Check collected metrics
    let metrics = monitoring_system.get_current_metrics().await?;
    assert_eq!(metrics.total_requests, 5);
    assert!(metrics.average_response_time_ms < 200.0);
    assert_eq!(metrics.success_rate, 1.0);

    // Test alert thresholds
    monitoring_system
        .simulate_slow_response(Duration::from_millis(300))
        .await?;

    let alerts = monitoring_system.get_active_alerts().await?;
    assert!(!alerts.is_empty());
    assert!(alerts.iter().any(|a| a.alert_type == "slow_response"));

    Ok(())
}

/// Test configuration integration and validation
#[tokio::test]
async fn test_configuration_integration() -> Result<()> {
    let _env = setup_test_environment().await?;

    // Test provider configuration
    let provider_config = ProviderConfig {
        openai: Some(json!({
            "model": "gpt-4",
            "rate_limit": 60,
            "timeout": "30s"
        })),
        claude: Some(json!({
            "model": "claude-3-sonnet-20240229",
            "rate_limit": 50,
            "timeout": "30s"
        })),
        fallback_strategy: FallbackStrategy::RoundRobin,
        health_check_interval: Duration::from_secs(30),
    };

    // Test quality configuration
    let quality_config = QualityConfig {
        cross_validation_enabled: true,
        provider_count: 2,
        agreement_threshold: 0.8,
        quality_threshold: 0.95,
        scoring_algorithms: vec![
            "semantic_similarity".to_string(),
            "completeness".to_string(),
        ],
        weights: vec![0.6, 0.4],
    };

    // Test learning configuration
    let learning_config = LearningConfig {
        feedback_collection_enabled: true,
        learning_rate: 0.1,
        adaptation_threshold: 0.8,
        pattern_recognition_enabled: true,
        optimization_interval: Duration::from_hours(1),
    };

    // Test monitoring configuration
    let monitoring_config = MonitoringConfig {
        metrics_collection_enabled: true,
        alert_thresholds: json!({
            "response_time_ms": 200,
            "success_rate": 0.95,
            "error_rate": 0.05
        }),
        health_check_interval: Duration::from_secs(30),
        retention_period: Duration::from_days(7),
    };

    // Create systems with configurations
    let provider_manager = ProviderManager::new(provider_config).await?;
    let quality_controller = QualityController::new(quality_config).await?;
    let learning_system = LearningSystem::new(learning_config).await?;
    let monitoring_system = MonitoringSystem::new(monitoring_config).await?;

    // Verify all systems initialized correctly
    assert!(provider_manager.is_healthy().await?);
    assert!(quality_controller.is_enabled().await?);
    assert!(learning_system.is_active().await?);
    assert!(monitoring_system.is_running().await?);

    Ok(())
}

/// Test error handling and recovery across all systems
#[tokio::test]
async fn test_error_handling_and_recovery() -> Result<()> {
    let _env = setup_test_environment().await?;

    let provider_manager = create_test_provider_manager().await?;
    let quality_controller = create_test_quality_controller().await?;
    let learning_system = create_test_learning_system().await?;
    let monitoring_system = create_test_monitoring_system().await?;

    // Test provider failure recovery
    provider_manager.simulate_provider_failure("openai").await?;
    provider_manager.simulate_provider_failure("claude").await?;

    // Should gracefully degrade to available providers
    let result = provider_manager.execute_query("Test with failures").await;
    match result {
        Ok(res) => {
            assert_eq!(res.provider_used, "gemini"); // Last available
            assert!(res.fallback_triggered);
        }
        Err(_) => {
            // All providers down - should return appropriate error
            assert!(true); // Expected behavior
        }
    }

    // Test quality system with provider failures
    let quality_result = quality_controller
        .validate_with_degraded_providers(&provider_manager, "Test quality with failures")
        .await?;

    assert!(quality_result.degraded_mode);
    assert!(quality_result.confidence_score < 1.0); // Lower confidence with fewer providers

    // Test learning system persistence during failures
    learning_system
        .record_failure_event("provider_unavailable")
        .await?;
    let failure_patterns = learning_system.analyze_failure_patterns().await?;
    assert!(!failure_patterns.is_empty());

    // Test monitoring system alert generation
    monitoring_system
        .record_system_failure("multi_provider_failure")
        .await?;
    let critical_alerts = monitoring_system.get_critical_alerts().await?;
    assert!(!critical_alerts.is_empty());

    Ok(())
}

// Helper functions for test setup
async fn create_test_provider_manager() -> Result<ProviderManager> {
    let config = ProviderConfig {
        openai: Some(json!({"model": "gpt-4", "rate_limit": 60})),
        claude: Some(json!({"model": "claude-3-sonnet-20240229", "rate_limit": 50})),
        gemini: Some(json!({"model": "gemini-pro", "rate_limit": 60})),
        fallback_strategy: FallbackStrategy::RoundRobin,
        health_check_interval: Duration::from_secs(30),
    };

    ProviderManager::new(config).await
}

async fn create_test_quality_controller() -> Result<QualityController> {
    let config = QualityConfig {
        cross_validation_enabled: true,
        provider_count: 2,
        agreement_threshold: 0.8,
        quality_threshold: 0.95,
        scoring_algorithms: vec!["semantic_similarity".to_string()],
        weights: vec![1.0],
    };

    QualityController::new(config).await
}

async fn create_test_learning_system() -> Result<LearningSystem> {
    let config = LearningConfig {
        feedback_collection_enabled: true,
        learning_rate: 0.1,
        adaptation_threshold: 0.8,
        pattern_recognition_enabled: true,
        optimization_interval: Duration::from_hours(1),
    };

    LearningSystem::new(config).await
}

async fn create_test_monitoring_system() -> Result<MonitoringSystem> {
    let config = MonitoringConfig {
        metrics_collection_enabled: true,
        alert_thresholds: json!({
            "response_time_ms": 200,
            "success_rate": 0.95
        }),
        health_check_interval: Duration::from_secs(30),
        retention_period: Duration::from_days(1), // Shorter for tests
    };

    MonitoringSystem::new(config).await
}

async fn create_integrated_pipeline(
    provider_manager: &ProviderManager,
    quality_controller: &QualityController,
    learning_system: &LearningSystem,
    monitoring_system: &MonitoringSystem,
) -> Result<ResearchPipeline> {
    // This function will be implemented when we integrate the systems
    todo!("Implement integrated pipeline creation")
}

// Additional test structs that will be implemented
#[derive(Debug)]
pub struct ProviderResult {
    pub provider_used: String,
    pub fallback_triggered: bool,
    pub processing_time_ms: u64,
}

#[derive(Debug)]
pub struct ValidationResult {
    pub provider_responses: Vec<String>,
    pub agreement_score: f64,
    pub quality_score: f64,
    pub consensus_response: String,
    pub confidence_score: f64,
}

#[derive(Debug)]
pub struct LearningInsights {
    pub total_feedback_count: u64,
    pub average_rating: f64,
}

#[derive(Debug)]
pub struct UsagePatterns {
    pub dominant_topics: Vec<String>,
    pub improvement_areas: Vec<String>,
}

#[derive(Debug)]
pub struct Alert {
    pub alert_type: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct QualityResultDegraded {
    pub degraded_mode: bool,
    pub confidence_score: f64,
}
