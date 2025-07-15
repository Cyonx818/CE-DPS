// ABOUTME: Comprehensive example demonstrating multi-provider research engine integration
//! This example shows how to set up and use the multi-provider research engine with
//! automatic provider selection, failover, and performance optimization.

use fortitude::providers::config::ProviderSettings;
use fortitude::providers::{
    ClaudeProvider, OpenAIProvider, ProviderConfig, ProviderManager, SelectionStrategy,
};
use fortitude_core::{
    MultiProviderConfig, MultiProviderResearchEngine, PipelineBuilder, PipelineConfig,
    ResearchEngine,
};
use fortitude_types::{
    AudienceContext, BasicClassifier, ClassifiedRequest, DomainContext, InMemoryStorage,
    ResearchType,
};

use std::sync::Arc;
use std::time::Duration;
use tokio;
use tracing::{error, info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting multi-provider research engine demo");

    // Step 1: Set up provider manager with multiple providers
    let provider_manager = setup_provider_manager().await?;

    // Step 2: Create multi-provider research engine
    let research_engine = create_multi_provider_engine(provider_manager).await?;

    // Step 3: Set up complete research pipeline
    let pipeline = setup_research_pipeline(research_engine).await?;

    // Step 4: Run demonstration scenarios
    run_demonstration_scenarios(&pipeline).await?;

    info!("Multi-provider research engine demo completed successfully");
    Ok(())
}

/// Set up provider manager with multiple LLM providers
async fn setup_provider_manager() -> Result<ProviderManager, Box<dyn std::error::Error>> {
    info!("Setting up multi-provider manager");

    // Configure provider manager for intelligent selection and failover
    let provider_config = ProviderConfig {
        selection_strategy: SelectionStrategy::Balanced,
        enable_failover: true,
        enable_cross_validation: false,
        max_failover_attempts: 3,
        provider_timeout: Duration::from_secs(30),
        health_check_interval: Duration::from_secs(300),
        enable_performance_tracking: true,
        performance_window_size: 100,
        cost_optimization_threshold: 0.1,
        min_quality_threshold: 0.6,
    };

    let mut provider_manager = ProviderManager::new(provider_config).await?;

    // Add OpenAI provider (if API key is available)
    if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
        if !openai_key.is_empty() && openai_key != "your-openai-api-key-here" {
            info!("Adding OpenAI provider");

            let openai_settings = ProviderSettings::new(openai_key, "gpt-4".to_string())
                .with_timeout(Duration::from_secs(30))
                .with_rate_limits(fortitude::providers::config::RateLimitConfig {
                    requests_per_minute: 60,
                    input_tokens_per_minute: 100_000,
                    output_tokens_per_minute: 20_000,
                    max_concurrent_requests: 5,
                });

            let openai_provider = Arc::new(OpenAIProvider::new(openai_settings).await?);
            provider_manager
                .add_provider("openai".to_string(), openai_provider)
                .await?;
            info!("OpenAI provider added successfully");
        } else {
            info!("OpenAI API key not configured, skipping OpenAI provider");
        }
    } else {
        info!("OPENAI_API_KEY environment variable not found, skipping OpenAI provider");
    }

    // Add Claude provider (if API key is available)
    if let Ok(claude_key) = std::env::var("CLAUDE_API_KEY") {
        if !claude_key.is_empty() && claude_key != "your-claude-api-key-here" {
            info!("Adding Claude provider");

            let claude_settings =
                ProviderSettings::new(claude_key, "claude-3-5-sonnet-20241022".to_string())
                    .with_timeout(Duration::from_secs(30))
                    .with_rate_limits(fortitude::providers::config::RateLimitConfig {
                        requests_per_minute: 50,
                        input_tokens_per_minute: 80_000,
                        output_tokens_per_minute: 16_000,
                        max_concurrent_requests: 3,
                    });

            let claude_provider = Arc::new(ClaudeProvider::new(claude_settings).await?);
            provider_manager
                .add_provider("claude".to_string(), claude_provider)
                .await?;
            info!("Claude provider added successfully");
        } else {
            info!("Claude API key not configured, skipping Claude provider");
        }
    } else {
        info!("CLAUDE_API_KEY environment variable not found, skipping Claude provider");
    }

    // Verify we have at least one provider
    let providers = provider_manager.list_providers().await;
    if providers.is_empty() {
        return Err("No providers configured. Please set OPENAI_API_KEY or CLAUDE_API_KEY environment variables.".into());
    }

    info!(
        "Provider manager configured with {} providers: {:?}",
        providers.len(),
        providers
    );
    Ok(provider_manager)
}

/// Create multi-provider research engine with intelligent selection
async fn create_multi_provider_engine(
    provider_manager: ProviderManager,
) -> Result<Arc<MultiProviderResearchEngine>, Box<dyn std::error::Error>> {
    info!("Creating multi-provider research engine");

    // Configure multi-provider research engine
    let config = MultiProviderConfig {
        enable_cross_validation: false, // Disable for demo to reduce API calls
        cross_validation_providers: 2,
        quality_threshold: 0.7,
        enable_vector_search: false, // Disable for demo simplicity
        max_context_documents: 5,
        context_relevance_threshold: 0.7,
        enable_quality_validation: true,
        min_quality_score: 0.6,
        max_processing_time: Duration::from_secs(60),
        enable_performance_optimization: true,
        cost_optimization_weight: 0.3,
        quality_optimization_weight: 0.5,
        latency_optimization_weight: 0.2,
    };

    let research_engine = MultiProviderResearchEngine::new(provider_manager, config).await?;

    info!("Multi-provider research engine created successfully");
    Ok(Arc::new(research_engine))
}

/// Set up complete research pipeline with multi-provider engine
async fn setup_research_pipeline(
    research_engine: Arc<MultiProviderResearchEngine>,
) -> Result<fortitude_core::ResearchPipeline, Box<dyn std::error::Error>> {
    info!("Setting up research pipeline");

    // Create basic classifier and storage for the pipeline
    let classifier = Arc::new(BasicClassifier::new());
    let storage = Arc::new(InMemoryStorage::new());

    // Configure pipeline for multi-provider research
    let pipeline_config = PipelineConfig {
        max_concurrent: 5,
        timeout_seconds: 300,
        enable_caching: true,
        default_audience: AudienceContext {
            level: "intermediate".to_string(),
            domain: "software_development".to_string(),
            format: "markdown".to_string(),
        },
        default_domain: DomainContext {
            technology: "rust".to_string(),
            project_type: "library".to_string(),
            frameworks: vec!["tokio".to_string(), "serde".to_string()],
            tags: vec!["async".to_string(), "performance".to_string()],
        },
        enable_context_detection: true,
        enable_advanced_classification: false,
        advanced_classification_config: None,
        enable_vector_search: false,
        auto_index_results: false,
        enable_context_discovery: false,
    };

    // Build pipeline with multi-provider research engine
    let pipeline = PipelineBuilder::new()
        .with_multi_provider_research_engine(research_engine)
        .with_max_concurrent(pipeline_config.max_concurrent)
        .with_timeout(pipeline_config.timeout_seconds)
        .with_caching(pipeline_config.enable_caching)
        .with_default_audience(pipeline_config.default_audience)
        .with_default_domain(pipeline_config.default_domain)
        .with_context_detection(pipeline_config.enable_context_detection)
        .build(classifier, storage);

    info!("Research pipeline configured successfully");
    Ok(pipeline)
}

/// Run various demonstration scenarios
async fn run_demonstration_scenarios(
    pipeline: &fortitude_core::ResearchPipeline,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running demonstration scenarios");

    // Scenario 1: Learning query
    info!("\n=== Scenario 1: Learning Query ===");
    let learning_result = pipeline
        .process_query(
            "What are the key principles of async programming in Rust?",
            None,
            None,
        )
        .await;

    match learning_result {
        Ok(result) => {
            info!("Learning query successful");
            info!("Research type: {:?}", result.request.research_type);
            info!("Quality score: {:.2}", result.metadata.quality_score);
            info!("Processing time: {}ms", result.metadata.processing_time_ms);
            info!("Sources: {:?}", result.metadata.sources_consulted);
            info!(
                "Answer preview: {}",
                result
                    .immediate_answer
                    .chars()
                    .take(150)
                    .collect::<String>()
                    + "..."
            );
        }
        Err(e) => {
            error!("Learning query failed: {}", e);
        }
    }

    // Scenario 2: Implementation query
    info!("\n=== Scenario 2: Implementation Query ===");
    let implementation_result = pipeline
        .process_query(
            "How do I implement a concurrent task queue using tokio in Rust?",
            Some(AudienceContext {
                level: "advanced".to_string(),
                domain: "systems_programming".to_string(),
                format: "markdown".to_string(),
            }),
            Some(DomainContext {
                technology: "rust".to_string(),
                project_type: "service".to_string(),
                frameworks: vec!["tokio".to_string(), "futures".to_string()],
                tags: vec!["concurrency".to_string(), "async".to_string()],
            }),
        )
        .await;

    match implementation_result {
        Ok(result) => {
            info!("Implementation query successful");
            info!("Research type: {:?}", result.request.research_type);
            info!("Quality score: {:.2}", result.metadata.quality_score);
            info!("Processing time: {}ms", result.metadata.processing_time_ms);
            info!("Sources: {:?}", result.metadata.sources_consulted);
            info!(
                "Answer preview: {}",
                result
                    .immediate_answer
                    .chars()
                    .take(150)
                    .collect::<String>()
                    + "..."
            );
        }
        Err(e) => {
            error!("Implementation query failed: {}", e);
        }
    }

    // Scenario 3: Troubleshooting query
    info!("\n=== Scenario 3: Troubleshooting Query ===");
    let troubleshooting_result = pipeline.process_query(
        "My Rust async function is not cancelling properly when the future is dropped. How do I fix this?",
        None,
        None,
    ).await;

    match troubleshooting_result {
        Ok(result) => {
            info!("Troubleshooting query successful");
            info!("Research type: {:?}", result.request.research_type);
            info!("Quality score: {:.2}", result.metadata.quality_score);
            info!("Processing time: {}ms", result.metadata.processing_time_ms);
            info!("Sources: {:?}", result.metadata.sources_consulted);
            info!(
                "Answer preview: {}",
                result
                    .immediate_answer
                    .chars()
                    .take(150)
                    .collect::<String>()
                    + "..."
            );
        }
        Err(e) => {
            error!("Troubleshooting query failed: {}", e);
        }
    }

    // Scenario 4: Decision query
    info!("\n=== Scenario 4: Decision Query ===");
    let decision_result = pipeline
        .process_query(
            "Should I use tokio or async-std for my new Rust web service?",
            Some(AudienceContext {
                level: "intermediate".to_string(),
                domain: "web_development".to_string(),
                format: "markdown".to_string(),
            }),
            None,
        )
        .await;

    match decision_result {
        Ok(result) => {
            info!("Decision query successful");
            info!("Research type: {:?}", result.request.research_type);
            info!("Quality score: {:.2}", result.metadata.quality_score);
            info!("Processing time: {}ms", result.metadata.processing_time_ms);
            info!("Sources: {:?}", result.metadata.sources_consulted);
            info!(
                "Answer preview: {}",
                result
                    .immediate_answer
                    .chars()
                    .take(150)
                    .collect::<String>()
                    + "..."
            );
        }
        Err(e) => {
            error!("Decision query failed: {}", e);
        }
    }

    // Scenario 5: Demonstrate caching - repeat a previous query
    info!("\n=== Scenario 5: Cache Demonstration ===");
    let cached_result = pipeline
        .process_query(
            "What are the key principles of async programming in Rust?", // Same as scenario 1
            None,
            None,
        )
        .await;

    match cached_result {
        Ok(result) => {
            info!("Cached query successful");
            info!(
                "Processing time: {}ms (should be faster due to caching)",
                result.metadata.processing_time_ms
            );
            if result
                .metadata
                .sources_consulted
                .contains(&"cache".to_string())
            {
                info!("Result served from cache!");
            } else {
                info!("Result generated fresh (cache miss or disabled)");
            }
        }
        Err(e) => {
            error!("Cached query failed: {}", e);
        }
    }

    info!("\nAll demonstration scenarios completed");
    Ok(())
}

/// Helper function to display provider performance statistics
#[allow(dead_code)]
async fn display_provider_stats(
    research_engine: &MultiProviderResearchEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("\n=== Provider Performance Statistics ===");

    // This would require exposing the provider manager from the research engine
    // In a real implementation, you might want to add methods to get performance stats

    info!("Provider statistics display not implemented in this demo");
    info!("In a production setup, you would see:");
    info!("- Request counts per provider");
    info!("- Success rates");
    info!("- Average response times");
    info!("- Cost metrics");
    info!("- Health status");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_demo_setup() {
        // Test that the demo setup functions don't panic
        // We can't test the full flow without API keys

        let provider_config = ProviderConfig::default();
        let provider_manager_result = ProviderManager::new(provider_config).await;
        assert!(provider_manager_result.is_ok());

        // Test multi-provider config creation
        let config = MultiProviderConfig::default();
        assert!(!config.enable_cross_validation);
        assert_eq!(config.quality_threshold, 0.7);
        assert!(config.enable_quality_validation);
    }

    #[test]
    fn test_pipeline_config() {
        let config = PipelineConfig::default();
        assert_eq!(config.max_concurrent, 5);
        assert_eq!(config.timeout_seconds, 300);
        assert!(config.enable_caching);
    }
}
