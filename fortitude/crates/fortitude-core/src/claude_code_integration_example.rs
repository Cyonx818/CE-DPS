// Copyright 2025 CE-DPS Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// ABOUTME: Example showing how to integrate ClaudeCodeProvider with Fortitude research engine
// This demonstrates the configuration and usage pattern for Claude Code as a research fallback

use crate::claude_code_provider::{ClaudeCodeProvider, ClaudeCodeProviderConfig};
use crate::multi_provider_research_engine::{MultiProviderConfig, MultiProviderResearchEngine};
use crate::research_engine::ResearchEngine;
use fortitude_types::{AudienceContext, ClassifiedRequest, DomainContext, ResearchType};

use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

/// Example configuration for Claude Code integration
pub struct ClaudeCodeIntegrationConfig {
    pub enable_claude_code_fallback: bool,
    pub claude_code_timeout: Duration,
    pub max_web_search_results: usize,
    pub development_mode: bool,
}

impl Default for ClaudeCodeIntegrationConfig {
    fn default() -> Self {
        Self {
            enable_claude_code_fallback: true,
            claude_code_timeout: Duration::from_secs(120),
            max_web_search_results: 5,
            development_mode: true, // Enable for development, disable for production
        }
    }
}

/// Example of how to create a research engine with Claude Code fallback
pub async fn create_research_engine_with_claude_code_fallback(
    config: ClaudeCodeIntegrationConfig,
) -> Result<impl ResearchEngine, Box<dyn std::error::Error + Send + Sync>> {
    info!("Creating research engine with Claude Code fallback");

    if !config.enable_claude_code_fallback {
        warn!("Claude Code fallback is disabled");
        return Err("Claude Code fallback is required for this example".into());
    }

    // Create Claude Code provider configuration
    let claude_code_config = ClaudeCodeProviderConfig {
        provider_name: "claude-code-websearch-fallback".to_string(),
        enable_performance_tracking: true,
        max_processing_time: config.claude_code_timeout,
        max_web_results: config.max_web_search_results,
        enable_structured_parsing: true,
    };

    // Create Claude Code provider
    let claude_code_provider = ClaudeCodeProvider::new(claude_code_config);

    // Create multi-provider configuration
    let multi_provider_config = MultiProviderConfig {
        enable_cross_validation: false, // Disable for single provider
        quality_threshold: 0.6,
        enable_vector_search: false, // Disable for development
        enable_quality_validation: true,
        min_quality_score: 0.6,
        max_processing_time: config.claude_code_timeout,
        enable_performance_optimization: true,
        cost_optimization_weight: 0.1, // Low weight since we're using existing Claude subscription
        quality_optimization_weight: 0.8, // High weight for quality
        latency_optimization_weight: 0.1, // Low weight for latency
        ..Default::default()
    };

    // Create multi-provider research engine with Claude Code provider
    let research_engine = MultiProviderResearchEngine::new(
        Arc::new(claude_code_provider),
        multi_provider_config,
    )
    .await?;

    info!("Research engine with Claude Code fallback created successfully");
    Ok(research_engine)
}

/// Example of how to use the research engine for Fortitude development
pub async fn example_fortitude_research_usage() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create the research engine
    let research_engine = create_research_engine_with_claude_code_fallback(
        ClaudeCodeIntegrationConfig::default(),
    )
    .await?;

    // Example 1: Research a Phase 1 roadmap item
    let storage_research = ClassifiedRequest::new(
        "How to implement stable cache index management with proper write lock handling in Rust?".to_string(),
        ResearchType::Implementation,
        AudienceContext {
            level: "advanced".to_string(),
            domain: "rust".to_string(),
            format: "markdown".to_string(),
        },
        DomainContext {
            technology: "rust".to_string(),
            project_type: "library".to_string(),
            frameworks: vec!["tokio".to_string(), "serde".to_string()],
            tags: vec!["cache".to_string(), "concurrency".to_string()],
        },
        0.9,
        vec!["cache".to_string(), "index".to_string(), "lock".to_string()],
    );

    info!("Executing storage research query");
    let storage_result = research_engine.generate_research(&storage_research).await?;
    
    println!("Storage Research Result:");
    println!("Query: {}", storage_result.original_query());
    println!("Quality Score: {:.2}", storage_result.metadata.quality_score);
    println!("Processing Time: {}ms", storage_result.metadata.processing_time_ms);
    println!("Answer: {}", storage_result.immediate_answer);

    // Example 2: Research a Phase 2 ML integration item
    let ml_research = ClassifiedRequest::new(
        "How to implement ReciprocalRankFusion for hybrid search result combination?".to_string(),
        ResearchType::Implementation,
        AudienceContext {
            level: "intermediate".to_string(),
            domain: "machine-learning".to_string(),
            format: "markdown".to_string(),
        },
        DomainContext {
            technology: "rust".to_string(),
            project_type: "library".to_string(),
            frameworks: vec!["candle".to_string(), "qdrant".to_string()],
            tags: vec!["ml".to_string(), "search".to_string(), "fusion".to_string()],
        },
        0.8,
        vec!["reciprocal".to_string(), "rank".to_string(), "fusion".to_string()],
    );

    info!("Executing ML research query");
    let ml_result = research_engine.generate_research(&ml_research).await?;
    
    println!("\nML Research Result:");
    println!("Query: {}", ml_result.original_query());
    println!("Quality Score: {:.2}", ml_result.metadata.quality_score);
    println!("Processing Time: {}ms", ml_result.metadata.processing_time_ms);
    println!("Answer: {}", ml_result.immediate_answer);

    // Example 3: Research a troubleshooting query
    let troubleshooting_research = ClassifiedRequest::new(
        "Why is my Rust async function causing high memory usage and how can I optimize it?".to_string(),
        ResearchType::Troubleshooting,
        AudienceContext {
            level: "intermediate".to_string(),
            domain: "rust".to_string(),
            format: "markdown".to_string(),
        },
        DomainContext {
            technology: "rust".to_string(),
            project_type: "service".to_string(),
            frameworks: vec!["tokio".to_string()],
            tags: vec!["async".to_string(), "memory".to_string(), "performance".to_string()],
        },
        0.85,
        vec!["async".to_string(), "memory".to_string(), "optimize".to_string()],
    );

    info!("Executing troubleshooting research query");
    let troubleshooting_result = research_engine.generate_research(&troubleshooting_research).await?;
    
    println!("\nTroubleshooting Research Result:");
    println!("Query: {}", troubleshooting_result.original_query());
    println!("Quality Score: {:.2}", troubleshooting_result.metadata.quality_score);
    println!("Processing Time: {}ms", troubleshooting_result.metadata.processing_time_ms);
    println!("Answer: {}", troubleshooting_result.immediate_answer);

    // Note: Provider performance stats are available through the provider manager
    // but not directly exposed through the ResearchEngine trait
    info!("Research examples completed successfully");

    Ok(())
}

/// Example of how to use Claude Code provider for specific roadmap research
pub async fn research_roadmap_item(
    query: &str,
    research_type: ResearchType,
    technology: &str,
    frameworks: Vec<String>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let research_engine = create_research_engine_with_claude_code_fallback(
        ClaudeCodeIntegrationConfig::default(),
    )
    .await?;

    let request = ClassifiedRequest::new(
        query.to_string(),
        research_type,
        AudienceContext {
            level: "advanced".to_string(),
            domain: technology.to_string(),
            format: "markdown".to_string(),
        },
        DomainContext {
            technology: technology.to_string(),
            project_type: "library".to_string(),
            frameworks,
            tags: vec!["fortitude".to_string(), "development".to_string()],
        },
        0.9,
        vec!["implementation".to_string(), "roadmap".to_string()],
    );

    let result = research_engine.generate_research(&request).await?;
    Ok(result.immediate_answer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_claude_code_integration_creation() {
        let config = ClaudeCodeIntegrationConfig::default();
        let result = create_research_engine_with_claude_code_fallback(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_roadmap_research() {
        let result = research_roadmap_item(
            "How to implement circuit breaker patterns in Rust async applications?",
            ResearchType::Implementation,
            "rust",
            vec!["tokio".to_string()],
        )
        .await;
        
        assert!(result.is_ok());
        let answer = result.unwrap();
        assert!(answer.contains("circuit breaker"));
        assert!(!answer.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_research_types() {
        let research_engine = create_research_engine_with_claude_code_fallback(
            ClaudeCodeIntegrationConfig::default(),
        )
        .await
        .unwrap();

        // Test different research types
        let research_types = vec![
            (ResearchType::Implementation, "How to implement JWT authentication?"),
            (ResearchType::Decision, "Should I use PostgreSQL or MongoDB for this project?"),
            (ResearchType::Learning, "What are the key concepts of async programming in Rust?"),
            (ResearchType::Troubleshooting, "Why is my Rust application consuming too much memory?"),
            (ResearchType::Validation, "Is this error handling approach correct for production?"),
        ];

        for (research_type, query) in research_types {
            let request = ClassifiedRequest::new(
                query.to_string(),
                research_type,
                AudienceContext::default(),
                DomainContext::default(),
                0.8,
                vec![],
            );

            let result = research_engine.generate_research(&request).await;
            assert!(result.is_ok(), "Failed for research type: {:?}", research_type);
            
            let research_result = result.unwrap();
            assert!(!research_result.immediate_answer.is_empty());
            assert!(research_result.metadata.quality_score > 0.0);
        }
    }
}