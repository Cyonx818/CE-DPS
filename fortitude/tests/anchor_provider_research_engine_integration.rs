// ABOUTME: Anchor test for critical provider-research engine integration
//! This anchor test validates the complete integration of the multi-LLM provider system
//! with the research engine, ensuring that provider selection, fallback strategies,
//! and quality control work seamlessly together.

use fortitude::{
    providers::{
        config::ProviderSettings,
        manager::{ProviderConfig, ProviderManager, SelectionStrategy},
        mock::MockProvider,
    },
    research_engine_adapter::{ProviderManagerAdapter, ResearchEngineFactory},
};
use fortitude_core::{
    multi_provider_research_engine::MultiProviderConfig, research_engine::ResearchEngine,
};
use fortitude_types::{AudienceContext, ClassifiedRequest, DomainContext, ResearchType};

use std::sync::Arc;
use std::time::Duration;
use tracing_test::traced_test;

/// Create a test research request
fn create_test_request(query: &str, research_type: ResearchType) -> ClassifiedRequest {
    ClassifiedRequest::new(
        query.to_string(),
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
        vec!["async".to_string(), "rust".to_string()],
    )
}

/// Setup a provider manager with multiple mock providers
async fn setup_multi_provider_manager() -> Arc<ProviderManager> {
    let mut config = ProviderConfig::default();
    config.selection_strategy = SelectionStrategy::RoundRobin;
    config.enable_failover = true;
    config.max_failover_attempts = 3;

    let provider_manager = Arc::new(ProviderManager::new(config).await.unwrap());

    // Add OpenAI mock provider
    let openai_provider = Arc::new(MockProvider::new("openai")
        .with_response("## Answer\nOpenAI provides comprehensive async programming guidance for Rust.\n\n## Evidence\nRust's async ecosystem is built on the tokio runtime.\n\n## Implementation\nUse async/await syntax with proper error handling."));

    provider_manager
        .add_provider("openai".to_string(), openai_provider)
        .await
        .unwrap();

    // Add Claude mock provider
    let claude_provider = Arc::new(MockProvider::new("claude")
        .with_response("## Answer\nClaude offers detailed explanations for Rust async programming patterns.\n\n## Evidence\nAsync programming requires understanding of futures and executors.\n\n## Implementation\nImplement proper stream handling and concurrent task management."));

    provider_manager
        .add_provider("claude".to_string(), claude_provider)
        .await
        .unwrap();

    // Add Gemini mock provider
    let gemini_provider = Arc::new(MockProvider::new("gemini")
        .with_response("## Answer\nGemini provides structured guidance for Rust async development.\n\n## Evidence\nModern Rust applications leverage async for performance.\n\n## Implementation\nCombine async streams with proper resource management."));

    provider_manager
        .add_provider("gemini".to_string(), gemini_provider)
        .await
        .unwrap();

    provider_manager
}

#[traced_test]
#[tokio::test]
async fn test_anchor_multi_provider_research_engine_basic_functionality() {
    // Setup provider manager with multiple providers
    let provider_manager = setup_multi_provider_manager().await;

    // Create multi-provider research engine
    let engine_config = MultiProviderConfig {
        enable_quality_validation: true,
        min_quality_score: 0.5,
        max_processing_time: Duration::from_secs(30),
        ..Default::default()
    };

    let research_engine = ResearchEngineFactory::create_multi_provider_engine(
        provider_manager.clone(),
        engine_config,
    )
    .await
    .unwrap();

    // Test basic research generation
    let request = create_test_request(
        "How to implement async streams in Rust?",
        ResearchType::Implementation,
    );

    let result = research_engine.generate_research(&request).await;
    assert!(result.is_ok(), "Research generation should succeed");

    let research_result = result.unwrap();
    assert!(
        !research_result.immediate_answer.is_empty(),
        "Should have research answer"
    );
    assert!(
        research_result.metadata.quality_score > 0.0,
        "Should have quality score"
    );
    assert!(
        research_result.metadata.processing_time_ms > 0,
        "Should have processing time"
    );

    // Verify provider metadata is included
    assert!(research_result
        .metadata
        .sources_consulted
        .contains(&"Multi-Provider Research Engine".to_string()));

    println!("✓ Basic multi-provider research functionality works");
}

#[traced_test]
#[tokio::test]
async fn test_anchor_provider_failover_in_research_engine() {
    let mut config = ProviderConfig::default();
    config.enable_failover = true;
    config.max_failover_attempts = 3;

    let provider_manager = Arc::new(ProviderManager::new(config).await.unwrap());

    // Add a failing provider as primary
    let failing_provider = Arc::new(MockProvider::new("failing-provider").with_failure(true));
    provider_manager
        .add_provider("primary".to_string(), failing_provider)
        .await
        .unwrap();

    // Add a working backup provider
    let backup_provider = Arc::new(MockProvider::new("backup-provider")
        .with_response("## Answer\nBackup provider successfully handled the research request.\n\n## Evidence\nFailover mechanisms ensure reliability.\n\n## Implementation\nImplement proper error handling and retry logic."));
    provider_manager
        .add_provider("backup".to_string(), backup_provider)
        .await
        .unwrap();

    // Create research engine
    let engine_config = MultiProviderConfig::default();
    let research_engine =
        ResearchEngineFactory::create_multi_provider_engine(provider_manager, engine_config)
            .await
            .unwrap();

    // Execute research - should failover to backup provider
    let request = create_test_request("Test failover functionality", ResearchType::Troubleshooting);

    let result = research_engine.generate_research(&request).await;
    assert!(result.is_ok(), "Research should succeed with failover");

    let research_result = result.unwrap();
    assert!(
        research_result.immediate_answer.contains("Backup provider"),
        "Should use backup provider response"
    );

    println!("✓ Provider failover in research engine works correctly");
}

#[traced_test]
#[tokio::test]
async fn test_anchor_research_engine_health_monitoring() {
    let provider_manager = setup_multi_provider_manager().await;

    // Create research engine
    let engine_config = MultiProviderConfig::default();
    let research_engine = ResearchEngineFactory::create_multi_provider_engine(
        provider_manager.clone(),
        engine_config,
    )
    .await
    .unwrap();

    // Test health check functionality
    let health_result = research_engine.health_check().await;
    assert!(
        health_result.is_ok(),
        "Health check should pass with healthy providers"
    );

    // Verify adapter health check integration
    let adapter = ProviderManagerAdapter::new(provider_manager);
    let health_statuses = adapter.health_check_all().await;
    assert!(health_statuses.is_ok(), "Adapter health check should work");

    let statuses = health_statuses.unwrap();
    assert!(
        statuses.len() >= 3,
        "Should have health status for all providers"
    );
    assert!(statuses.contains_key("openai"), "Should have OpenAI status");
    assert!(statuses.contains_key("claude"), "Should have Claude status");
    assert!(statuses.contains_key("gemini"), "Should have Gemini status");

    println!("✓ Research engine health monitoring integration works");
}

#[traced_test]
#[tokio::test]
async fn test_anchor_research_with_context_integration() {
    let provider_manager = setup_multi_provider_manager().await;

    // Create research engine with vector search enabled
    let mut engine_config = MultiProviderConfig::default();
    engine_config.enable_vector_search = false; // Disabled for this test since we don't have vector search setup
    engine_config.enable_quality_validation = true;

    let research_engine =
        ResearchEngineFactory::create_multi_provider_engine(provider_manager, engine_config)
            .await
            .unwrap();

    // Test context-aware research generation
    let request = create_test_request(
        "Best practices for async error handling in Rust",
        ResearchType::Learning,
    );

    let result = research_engine
        .generate_research_with_context(&request)
        .await;
    assert!(result.is_ok(), "Context-aware research should succeed");

    let research_result = result.unwrap();
    assert!(
        !research_result.immediate_answer.is_empty(),
        "Should have research answer"
    );
    assert!(
        research_result.metadata.quality_score > 0.0,
        "Should have quality score"
    );

    // Verify processing time estimation
    let processing_estimate = research_engine.estimate_processing_time(&request);
    assert!(
        processing_estimate > Duration::ZERO,
        "Should provide processing time estimate"
    );
    assert!(
        processing_estimate <= Duration::from_secs(60),
        "Estimate should be reasonable"
    );

    println!("✓ Context-aware research integration works");
}

#[traced_test]
#[tokio::test]
async fn test_anchor_performance_statistics_integration() {
    let provider_manager = setup_multi_provider_manager().await;

    let adapter = ProviderManagerAdapter::new(provider_manager.clone());

    // Get initial performance stats
    let initial_stats = adapter.get_performance_stats().await;
    assert!(
        initial_stats.len() >= 3,
        "Should have stats for all providers"
    );

    // Execute some research to generate performance data
    let engine_config = MultiProviderConfig::default();
    let research_engine = ResearchEngineFactory::create_multi_provider_engine(
        provider_manager.clone(),
        engine_config,
    )
    .await
    .unwrap();

    let request = create_test_request("Performance statistics test query", ResearchType::Decision);

    // Execute multiple requests to build performance history
    for i in 0..3 {
        let test_request = create_test_request(
            &format!("Performance test query {}", i),
            ResearchType::Implementation,
        );

        let result = research_engine.generate_research(&test_request).await;
        assert!(result.is_ok(), "Research {} should succeed", i);
    }

    // Verify performance stats are updated
    let updated_stats = adapter.get_performance_stats().await;
    assert!(
        updated_stats.len() >= 3,
        "Should maintain stats for all providers"
    );

    // At least one provider should have been used
    let has_requests = updated_stats.values().any(|stats| stats.total_requests > 0);
    assert!(has_requests, "At least one provider should show usage");

    println!("✓ Performance statistics integration works");
}

#[traced_test]
#[tokio::test]
async fn test_anchor_research_quality_validation() {
    let provider_manager = setup_multi_provider_manager().await;

    // Configure strict quality validation
    let mut engine_config = MultiProviderConfig::default();
    engine_config.enable_quality_validation = true;
    engine_config.min_quality_score = 0.1; // Low threshold for testing

    let research_engine =
        ResearchEngineFactory::create_multi_provider_engine(provider_manager, engine_config)
            .await
            .unwrap();

    let request = create_test_request(
        "Quality validation test - how to optimize Rust performance?",
        ResearchType::Validation,
    );

    let result = research_engine.generate_research(&request).await;
    assert!(
        result.is_ok(),
        "Research with quality validation should succeed"
    );

    let research_result = result.unwrap();
    assert!(
        research_result.metadata.quality_score >= 0.1,
        "Quality score should meet minimum threshold"
    );
    assert!(
        !research_result.immediate_answer.is_empty(),
        "Should have validated research content"
    );

    // Verify quality validation metadata
    assert!(
        research_result.metadata.sources_consulted.len() > 0,
        "Should have source information"
    );

    println!("✓ Research quality validation integration works");
}

#[traced_test]
#[tokio::test]
async fn test_anchor_concurrent_multi_provider_research() {
    let provider_manager = setup_multi_provider_manager().await;

    let engine_config = MultiProviderConfig::default();
    let research_engine = Arc::new(
        ResearchEngineFactory::create_multi_provider_engine(provider_manager, engine_config)
            .await
            .unwrap(),
    );

    // Execute concurrent research requests
    let mut handles = Vec::new();

    for i in 0..5 {
        let engine_clone = Arc::clone(&research_engine);
        let handle = tokio::spawn(async move {
            let request = create_test_request(
                &format!("Concurrent research query {}", i),
                ResearchType::Learning,
            );

            let result = engine_clone.generate_research(&request).await;
            assert!(result.is_ok(), "Concurrent research {} should succeed", i);

            let research_result = result.unwrap();
            assert!(
                !research_result.immediate_answer.is_empty(),
                "Concurrent request {} should have answer",
                i
            );

            i
        });

        handles.push(handle);
    }

    // Wait for all concurrent requests to complete
    for handle in handles {
        let request_id = handle.await.unwrap();
        println!("✓ Concurrent request {} completed successfully", request_id);
    }

    println!("✓ Concurrent multi-provider research works correctly");
}

#[traced_test]
#[tokio::test]
async fn test_anchor_different_research_types_provider_selection() {
    let provider_manager = setup_multi_provider_manager().await;

    let engine_config = MultiProviderConfig::default();
    let research_engine =
        ResearchEngineFactory::create_multi_provider_engine(provider_manager, engine_config)
            .await
            .unwrap();

    // Test different research types
    let test_cases = vec![
        ("How to learn Rust programming?", ResearchType::Learning),
        (
            "Implement a web server in Rust",
            ResearchType::Implementation,
        ),
        (
            "Debug memory leak in Rust application",
            ResearchType::Troubleshooting,
        ),
        ("Choose between tokio and async-std", ResearchType::Decision),
        ("Validate async design patterns", ResearchType::Validation),
    ];

    for (query, research_type) in test_cases {
        let request = create_test_request(query, research_type);

        let result = research_engine.generate_research(&request).await;
        assert!(
            result.is_ok(),
            "Research for {:?} should succeed",
            research_type
        );

        let research_result = result.unwrap();
        assert!(
            !research_result.immediate_answer.is_empty(),
            "Should have answer for {:?}",
            research_type
        );
        assert!(
            research_result.metadata.quality_score > 0.0,
            "Should have quality score for {:?}",
            research_type
        );

        println!("✓ Research type {:?} handled successfully", research_type);
    }

    println!("✓ All research types work with provider selection");
}

#[traced_test]
#[tokio::test]
async fn test_anchor_research_engine_error_handling() {
    // Test with no providers configured
    let empty_config = ProviderConfig::default();
    let empty_provider_manager = Arc::new(ProviderManager::new(empty_config).await.unwrap());

    let engine_config = MultiProviderConfig::default();
    let research_engine =
        ResearchEngineFactory::create_multi_provider_engine(empty_provider_manager, engine_config)
            .await
            .unwrap();

    let request = create_test_request(
        "Test error handling with no providers",
        ResearchType::Implementation,
    );

    let result = research_engine.generate_research(&request).await;
    assert!(result.is_err(), "Research should fail with no providers");

    println!("✓ Error handling works correctly when no providers available");
}

#[traced_test]
#[tokio::test]
async fn test_anchor_context_discovery_integration() {
    let provider_manager = setup_multi_provider_manager().await;

    let engine_config = MultiProviderConfig {
        enable_vector_search: false, // Disabled since we don't have vector search in this test
        ..Default::default()
    };

    let research_engine =
        ResearchEngineFactory::create_multi_provider_engine(provider_manager, engine_config)
            .await
            .unwrap();

    let request = create_test_request("Context discovery test query", ResearchType::Learning);

    // Test context discovery
    let context_result = research_engine.discover_context(&request).await;
    assert!(
        context_result.is_ok(),
        "Context discovery should succeed even when disabled"
    );

    let context_docs = context_result.unwrap();
    assert!(
        context_docs.is_empty(),
        "Should return empty context when disabled"
    );

    println!("✓ Context discovery integration works correctly");
}
