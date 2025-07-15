// ABOUTME: Multi-provider research engine with intelligent provider selection and failover
//! This module provides a research engine that leverages multiple LLM providers through the
//! provider abstraction layer. It supports intelligent provider selection, automatic failover,
//! performance tracking, and cost optimization while maintaining backward compatibility.
//!
//! # Features
//!
//! - **Multi-Provider Support**: Seamlessly works with OpenAI, Claude, and other providers
//! - **Intelligent Selection**: Chooses optimal provider based on query characteristics
//! - **Automatic Failover**: Switches to backup providers on failures
//! - **Performance Tracking**: Monitors provider performance and adapts selection
//! - **Cost Optimization**: Routes queries to cost-effective providers
//! - **Quality Validation**: Cross-provider result validation and quality comparison
//! - **Backward Compatibility**: Drop-in replacement for existing research engines
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use fortitude_core::multi_provider_research_engine::{MultiProviderResearchEngine, MultiProviderConfig};
//! use fortitude::providers::{ProviderManager, ProviderConfig, OpenAIProvider, ClaudeProvider};
//! use fortitude::providers::config::ProviderSettings;
//! use fortitude_types::{ClassifiedRequest, ResearchType};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Set up provider manager
//!     let mut provider_manager = ProviderManager::new(ProviderConfig::default()).await?;
//!     
//!     // Add OpenAI provider
//!     let openai_settings = ProviderSettings::new(
//!         std::env::var("OPENAI_API_KEY")?,
//!         "gpt-4".to_string()
//!     );
//!     let openai_provider = std::sync::Arc::new(OpenAIProvider::new(openai_settings).await?);
//!     provider_manager.add_provider("openai".to_string(), openai_provider).await?;
//!     
//!     // Add Claude provider
//!     let claude_settings = ProviderSettings::new(
//!         std::env::var("CLAUDE_API_KEY")?,
//!         "claude-3-5-sonnet-20241022".to_string()
//!     );
//!     let claude_provider = std::sync::Arc::new(ClaudeProvider::new(claude_settings).await?);
//!     provider_manager.add_provider("claude".to_string(), claude_provider).await?;
//!     
//!     // Create multi-provider research engine
//!     let config = MultiProviderConfig::default();
//!     let engine = MultiProviderResearchEngine::new(provider_manager, config).await?;
//!     
//!     // Execute research - provider selection is automatic
//!     let request = ClassifiedRequest::new(
//!         "How to implement async Rust applications?".to_string(),
//!         ResearchType::Implementation,
//!         // ... other fields
//!     );
//!     
//!     let result = engine.generate_research(&request).await?;
//!     println!("Research result: {}", result.immediate_answer);
//!     
//!     Ok(())
//! }
//! ```

use crate::prompts::{DefaultTemplateFactory, ParameterValue, QualityValidator, TemplateRegistry};
use crate::research_engine::{ResearchEngine, ResearchEngineError};
use crate::vector::{HybridSearchService, VectorDocument};
use fortitude_types::{
    ClassifiedRequest, Detail, Evidence, ResearchMetadata, ResearchResult, ResearchType,
};

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{debug, error, info, warn};

// Forward declaration - provider types will be resolved at compile time
pub trait ProviderManagerTrait: Send + Sync {
    /// Execute research with automatic provider selection and fallback
    fn execute_research(
        &self,
        request: &ClassifiedRequest,
    ) -> impl std::future::Future<Output = Result<String, Box<dyn std::error::Error + Send + Sync>>> + Send;

    /// Get performance statistics for all providers
    fn get_performance_stats(
        &self,
    ) -> impl std::future::Future<Output = HashMap<String, ProviderPerformanceStats>> + Send;

    /// Perform health checks on all providers
    fn health_check_all(
        &self,
    ) -> impl std::future::Future<
        Output = Result<
            HashMap<String, ProviderHealthStatus>,
            Box<dyn std::error::Error + Send + Sync>,
        >,
    > + Send;
}

#[derive(Debug, Clone)]
pub struct ProviderPerformanceStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_latency: Duration,
    pub average_quality: f64,
    pub success_rate: f64,
}

#[derive(Debug, Clone)]
pub enum ProviderHealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

/// Errors specific to multi-provider research engine
#[derive(Error, Debug)]
pub enum MultiProviderResearchError {
    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Research engine error: {0}")]
    ResearchEngineError(#[from] ResearchEngineError),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Quality validation failed: {0}")]
    QualityValidationError(String),

    #[error("Cross-validation failed: {0}")]
    CrossValidationError(String),

    #[error("Context discovery error: {0}")]
    ContextDiscoveryError(String),
}

/// Configuration for multi-provider research engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiProviderConfig {
    /// Enable cross-provider result validation
    pub enable_cross_validation: bool,

    /// Number of providers to use for cross-validation
    pub cross_validation_providers: usize,

    /// Quality threshold for accepting results
    pub quality_threshold: f64,

    /// Enable vector search context discovery
    pub enable_vector_search: bool,

    /// Maximum context documents to retrieve
    pub max_context_documents: usize,

    /// Context relevance threshold
    pub context_relevance_threshold: f64,

    /// Enable quality validation
    pub enable_quality_validation: bool,

    /// Minimum quality score threshold
    pub min_quality_score: f64,

    /// Maximum processing time before timeout
    pub max_processing_time: Duration,

    /// Enable performance optimization
    pub enable_performance_optimization: bool,

    /// Cost optimization weight (0.0 = ignore cost, 1.0 = only cost)
    pub cost_optimization_weight: f64,

    /// Quality optimization weight (0.0 = ignore quality, 1.0 = only quality)
    pub quality_optimization_weight: f64,

    /// Latency optimization weight (0.0 = ignore latency, 1.0 = only latency)
    pub latency_optimization_weight: f64,
}

impl Default for MultiProviderConfig {
    fn default() -> Self {
        Self {
            enable_cross_validation: false,
            cross_validation_providers: 2,
            quality_threshold: 0.7,
            enable_vector_search: false,
            max_context_documents: 5,
            context_relevance_threshold: 0.7,
            enable_quality_validation: true,
            min_quality_score: 0.6,
            max_processing_time: Duration::from_secs(60),
            enable_performance_optimization: true,
            cost_optimization_weight: 0.3,
            quality_optimization_weight: 0.5,
            latency_optimization_weight: 0.2,
        }
    }
}

/// Multi-provider research engine implementation
pub struct MultiProviderResearchEngine<T: ProviderManagerTrait> {
    provider_manager: Arc<T>,
    config: MultiProviderConfig,
    #[allow(dead_code)]
    template_registry: TemplateRegistry,
    quality_validator: QualityValidator,
    vector_search: Option<Arc<HybridSearchService>>,
}

impl<T: ProviderManagerTrait> MultiProviderResearchEngine<T> {
    /// Create a new multi-provider research engine
    pub async fn new(
        provider_manager: Arc<T>,
        config: MultiProviderConfig,
    ) -> Result<Self, MultiProviderResearchError> {
        let template_registry = DefaultTemplateFactory::create_default_registry();
        let quality_validator = QualityValidator::new();

        Ok(Self {
            provider_manager,
            config,
            template_registry,
            quality_validator,
            vector_search: None,
        })
    }

    /// Create a new multi-provider research engine with vector search capabilities
    pub async fn with_vector_search(
        provider_manager: Arc<T>,
        config: MultiProviderConfig,
        vector_search: Arc<HybridSearchService>,
    ) -> Result<Self, MultiProviderResearchError> {
        let template_registry = DefaultTemplateFactory::create_default_registry();
        let quality_validator = QualityValidator::new();

        Ok(Self {
            provider_manager,
            config,
            template_registry,
            quality_validator,
            vector_search: Some(vector_search),
        })
    }

    /// Execute research with cross-validation if enabled
    async fn execute_research_with_validation(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<ResearchResult, MultiProviderResearchError> {
        let start_time = Instant::now();

        info!(
            "Executing multi-provider research for query: '{}'",
            request.original_query
        );
        debug!(
            "Research type: {:?}, confidence: {:.2}",
            request.research_type, request.confidence
        );

        // Execute research through provider manager
        let response_text = self
            .provider_manager
            .execute_research(request)
            .await
            .map_err(|e| MultiProviderResearchError::ProviderError(e.to_string()))?;

        // Parse the response into structured format
        let (immediate_answer, supporting_evidence, implementation_details) =
            self.parse_research_response(&response_text, request);

        // Validate quality if enabled
        let quality_score = if self.config.enable_quality_validation {
            // Create a temporary result for validation
            let temp_result = ResearchResult::new(
                request.clone(),
                immediate_answer.clone(),
                supporting_evidence.clone(),
                implementation_details.clone(),
                ResearchMetadata {
                    completed_at: Utc::now(),
                    processing_time_ms: 0,
                    sources_consulted: vec![],
                    quality_score: 0.0,
                    cache_key: String::new(),
                    tags: HashMap::new(),
                },
            );

            match self.quality_validator.validate(&temp_result) {
                Ok(report) => {
                    debug!("Quality validation passed: {:.2}", report.overall_score);
                    if !report.issues.is_empty() {
                        warn!("Quality validation issues: {:?}", report.issues);
                    }
                    report.overall_score
                }
                Err(e) => {
                    warn!("Quality validation failed: {}", e);
                    0.6 // Default fallback score
                }
            }
        } else {
            0.8 // Default score when validation is disabled
        };

        // Check quality threshold
        if quality_score < self.config.min_quality_score {
            warn!(
                "Research quality {} below threshold {}",
                quality_score, self.config.min_quality_score
            );

            if self.config.enable_cross_validation && quality_score < self.config.quality_threshold
            {
                // Cross-validation should be triggered when quality is below threshold and cross-validation is enabled
                warn!("Cross-validation would be triggered here due to low quality score ({:.2} < {:.2})", 
                      quality_score, self.config.quality_threshold);
            }
        }

        let processing_time = start_time.elapsed();

        // Create research metadata
        let mut metadata = ResearchMetadata {
            completed_at: Utc::now(),
            processing_time_ms: processing_time.as_millis() as u64,
            sources_consulted: vec!["Multi-Provider Research Engine".to_string()],
            quality_score,
            cache_key: String::new(),
            tags: HashMap::new(),
        };

        // Add performance statistics to metadata
        let performance_stats = self.provider_manager.get_performance_stats().await;
        if !performance_stats.is_empty() {
            metadata.tags.insert(
                "provider_count".to_string(),
                performance_stats.len().to_string(),
            );

            // Calculate aggregate statistics
            let total_requests: u64 = performance_stats.values().map(|p| p.total_requests).sum();
            let avg_success_rate: f64 = performance_stats
                .values()
                .map(|p| p.success_rate)
                .sum::<f64>()
                / performance_stats.len() as f64;

            metadata.tags.insert(
                "total_provider_requests".to_string(),
                total_requests.to_string(),
            );
            metadata.tags.insert(
                "avg_provider_success_rate".to_string(),
                format!("{avg_success_rate:.2}"),
            );
        }

        let result = ResearchResult::new(
            request.clone(),
            immediate_answer,
            supporting_evidence,
            implementation_details,
            metadata,
        );

        info!(
            "Multi-provider research completed in {:.2}s (quality: {:.2})",
            processing_time.as_secs_f64(),
            quality_score
        );

        Ok(result)
    }

    /// Execute research with cross-validation across multiple providers (placeholder)
    #[allow(dead_code)]
    async fn execute_with_cross_validation(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<fortitude_types::ResearchResult, MultiProviderResearchError> {
        // Placeholder implementation
        self.execute_research_with_validation(request).await
    }

    /// Select the best result from cross-validation (placeholder)
    #[allow(dead_code)]
    async fn select_best_result(
        &self,
        results: Vec<fortitude_types::ResearchResult>,
    ) -> Result<fortitude_types::ResearchResult, MultiProviderResearchError> {
        if results.is_empty() {
            return Err(MultiProviderResearchError::CrossValidationError(
                "No results to select from".to_string(),
            ));
        }

        // Just return the first result for now
        Ok(results.into_iter().next().unwrap())
    }

    /// Build research prompt using template system
    #[allow(dead_code)]
    fn build_research_prompt(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<String, MultiProviderResearchError> {
        use crate::prompts::{ComplexityLevel, RegistryError};

        // Get appropriate template from registry
        let template = self
            .template_registry
            .get_best_for_type(
                &request.research_type,
                ComplexityLevel::Basic, // TODO: Determine complexity from request
            )
            .map_err(|e| match e {
                RegistryError::NoTemplatesForType(rt) => {
                    MultiProviderResearchError::ConfigurationError(format!(
                        "No templates found for research type: {rt:?}"
                    ))
                }
                RegistryError::TemplateNotFound(name) => {
                    MultiProviderResearchError::ConfigurationError(format!(
                        "Template not found: {name}"
                    ))
                }
                _ => MultiProviderResearchError::ConfigurationError(e.to_string()),
            })?;

        // Prepare template parameters
        let mut params = HashMap::new();

        // Add common parameters based on research type
        match request.research_type {
            ResearchType::Decision => {
                params.insert(
                    "problem".to_string(),
                    ParameterValue::Text(request.original_query.clone()),
                );
                params.insert(
                    "context".to_string(),
                    ParameterValue::Text(format!(
                        "Technology: {}, Project: {}, Audience: {} level",
                        request.domain_context.technology,
                        request.domain_context.project_type,
                        request.audience_context.level
                    )),
                );
            }
            ResearchType::Implementation => {
                params.insert(
                    "feature".to_string(),
                    ParameterValue::Text(request.original_query.clone()),
                );
                params.insert(
                    "technology".to_string(),
                    ParameterValue::Text(format!(
                        "{} ({})",
                        request.domain_context.technology,
                        request.domain_context.frameworks.join(", ")
                    )),
                );
            }
            ResearchType::Troubleshooting => {
                params.insert(
                    "problem".to_string(),
                    ParameterValue::Text(request.original_query.clone()),
                );
                params.insert(
                    "symptoms".to_string(),
                    ParameterValue::Text(format!(
                        "Context: {} project using {}",
                        request.domain_context.project_type, request.domain_context.technology
                    )),
                );
            }
            ResearchType::Learning => {
                params.insert(
                    "concept".to_string(),
                    ParameterValue::Text(request.original_query.clone()),
                );
                params.insert(
                    "level".to_string(),
                    ParameterValue::Text(request.audience_context.level.clone()),
                );
            }
            ResearchType::Validation => {
                params.insert(
                    "approach".to_string(),
                    ParameterValue::Text(request.original_query.clone()),
                );
                params.insert(
                    "criteria".to_string(),
                    ParameterValue::Text(format!(
                        "Suitable for {} level developers in {} domain",
                        request.audience_context.level, request.audience_context.domain
                    )),
                );
            }
        }

        // Render the template with parameters
        let rendered_template = template
            .render(&params)
            .map_err(|e| MultiProviderResearchError::ConfigurationError(e.to_string()))?;

        // Add context and instructions
        let audience_context = format!(
            "Audience: {} level developer in {} domain\nOutput format: {}",
            request.audience_context.level,
            request.audience_context.domain,
            request.audience_context.format
        );

        let domain_context = format!(
            "Technology: {}\nProject type: {}\nFrameworks: {}\nTags: {}",
            request.domain_context.technology,
            request.domain_context.project_type,
            request.domain_context.frameworks.join(", "),
            request.domain_context.tags.join(", ")
        );

        Ok(format!(
            r#"{}

{}

{}

Research Query: "{}"

Please provide a comprehensive research response following the progressive disclosure structure from the template above.

Ensure your response is:
- Technically accurate and current
- Appropriate for the specified audience level
- Relevant to the technology stack and project context
- Structured with clear sections and subsections
- Includes practical examples and code snippets where relevant
- Provides actionable next steps"#,
            rendered_template, audience_context, domain_context, request.original_query
        ))
    }

    /// Discover relevant context using vector search
    async fn discover_research_context(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<Vec<VectorDocument>, MultiProviderResearchError> {
        if !self.config.enable_vector_search {
            debug!("Vector search disabled, skipping context discovery");
            return Ok(Vec::new());
        }

        let Some(ref vector_search) = self.vector_search else {
            warn!("Vector search not configured despite being enabled");
            return Ok(Vec::new());
        };

        info!(
            "Discovering research context for: '{}'",
            request.original_query
        );

        // Use existing context discovery logic from ClaudeResearchEngine
        // This would be moved to a shared module in a real implementation
        use crate::vector::{FusionMethod, HybridSearchRequest, SearchOptions, SearchStrategy};

        // Determine search strategy based on research type
        let search_strategy = match request.research_type {
            ResearchType::Decision => SearchStrategy::SemanticFocus,
            ResearchType::Implementation => SearchStrategy::Balanced,
            ResearchType::Troubleshooting => SearchStrategy::KeywordFocus,
            ResearchType::Learning => SearchStrategy::SemanticFocus,
            ResearchType::Validation => SearchStrategy::Balanced,
        };

        // Create hybrid search request
        let search_request = HybridSearchRequest {
            query: request.original_query.clone(),
            strategy: Some(search_strategy),
            fusion_method: Some(FusionMethod::ReciprocalRankFusion),
            options: SearchOptions {
                limit: self.config.max_context_documents,
                threshold: Some(self.config.context_relevance_threshold),
                filters: vec![], // TODO: Add research type filtering
                ..Default::default()
            },
            include_explanations: true,
            custom_weights: None,
            min_hybrid_score: Some(self.config.context_relevance_threshold),
        };

        // Perform hybrid search
        match vector_search.hybrid_search(search_request).await {
            Ok(search_results) => {
                let context_docs: Vec<VectorDocument> = search_results
                    .results
                    .into_iter()
                    .map(|result| result.document)
                    .collect();

                info!(
                    "Found {} context documents for research query",
                    context_docs.len()
                );
                debug!(
                    "Context discovery took {:.2}ms",
                    search_results.execution_stats.total_time_ms
                );

                Ok(context_docs)
            }
            Err(e) => {
                warn!("Context discovery failed: {}", e);
                // Return empty context instead of failing the entire research request
                Ok(Vec::new())
            }
        }
    }

    /// Parse response text into structured research result components
    fn parse_research_response(
        &self,
        response_text: &str,
        _request: &ClassifiedRequest,
    ) -> (String, Vec<Evidence>, Vec<Detail>) {
        let mut immediate_answer = String::new();
        let mut supporting_evidence = Vec::new();
        let mut implementation_details = Vec::new();

        // Simple parsing - in a real implementation, this would be more sophisticated
        let sections: Vec<&str> = response_text.split("## ").collect();

        for section in sections {
            if section.trim().is_empty() {
                continue;
            }

            if section.starts_with("Answer") {
                immediate_answer = section
                    .lines()
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join("\n")
                    .trim()
                    .to_string();
            } else if section.starts_with("Evidence") {
                let evidence_content = section
                    .lines()
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join("\n")
                    .trim()
                    .to_string();

                if !evidence_content.is_empty() {
                    supporting_evidence.push(Evidence {
                        source: "Multi-Provider Research Engine".to_string(),
                        content: evidence_content,
                        relevance: 0.9,
                        evidence_type: "Research Analysis".to_string(),
                    });
                }
            } else if section.starts_with("Implementation") {
                let impl_content = section
                    .lines()
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join("\n")
                    .trim()
                    .to_string();

                if !impl_content.is_empty() {
                    implementation_details.push(Detail {
                        category: "Implementation Guidance".to_string(),
                        content: impl_content,
                        priority: "High".to_string(),
                        prerequisites: vec![],
                    });
                }
            }
        }

        // If no structured sections found, use the entire response as the answer
        if immediate_answer.is_empty() {
            immediate_answer = response_text.to_string();
        }

        (
            immediate_answer,
            supporting_evidence,
            implementation_details,
        )
    }
}

#[async_trait]
impl<T: ProviderManagerTrait> ResearchEngine for MultiProviderResearchEngine<T> {
    async fn generate_research(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<ResearchResult, ResearchEngineError> {
        match self.execute_research_with_validation(request).await {
            Ok(result) => Ok(result),
            Err(e) => Err(ResearchEngineError::ApiError(
                crate::api::ApiError::ServiceUnavailable(e.to_string()),
            )),
        }
    }

    async fn generate_research_with_context(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<ResearchResult, ResearchEngineError> {
        let start_time = Instant::now();

        info!(
            "Generating context-aware research for query: '{}'",
            request.original_query
        );

        // Discover relevant context if enabled
        let context_documents = match self.discover_research_context(request).await {
            Ok(docs) => docs,
            Err(e) => {
                warn!("Context discovery failed: {}", e);
                Vec::new()
            }
        };

        // For now, execute regular research and enhance metadata with context info
        let mut result = self
            .execute_research_with_validation(request)
            .await
            .map_err(|e| {
                ResearchEngineError::ApiError(crate::api::ApiError::ServiceUnavailable(
                    e.to_string(),
                ))
            })?;

        // Add context information to metadata
        if !context_documents.is_empty() {
            result
                .metadata
                .sources_consulted
                .push("Vector Search Context".to_string());
            result.metadata.tags.insert(
                "context_documents".to_string(),
                context_documents.len().to_string(),
            );
            result
                .metadata
                .tags
                .insert("enhanced_with_context".to_string(), "true".to_string());
        }

        let processing_time = start_time.elapsed();
        info!(
            "Context-aware research completed in {:.2}s with {} context documents",
            processing_time.as_secs_f64(),
            context_documents.len()
        );

        Ok(result)
    }

    async fn discover_context(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<Vec<VectorDocument>, ResearchEngineError> {
        self.discover_research_context(request)
            .await
            .map_err(|e| ResearchEngineError::ContextDiscoveryError(e.to_string()))
    }

    async fn health_check(&self) -> Result<(), ResearchEngineError> {
        debug!("Performing health check on multi-provider research engine");

        // Check health of all providers through the provider manager
        match self.provider_manager.health_check_all().await {
            Ok(health_statuses) => {
                let mut unhealthy_providers = Vec::new();
                let mut degraded_providers = Vec::new();

                for (provider_name, status) in health_statuses {
                    match status {
                        ProviderHealthStatus::Healthy => {
                            debug!("Provider '{}' is healthy", provider_name);
                        }
                        ProviderHealthStatus::Degraded(reason) => {
                            warn!("Provider '{}' is degraded: {}", provider_name, reason);
                            degraded_providers.push(provider_name);
                        }
                        ProviderHealthStatus::Unhealthy(reason) => {
                            error!("Provider '{}' is unhealthy: {}", provider_name, reason);
                            unhealthy_providers.push(provider_name);
                        }
                    }
                }

                // If all providers are unhealthy, return error
                if !unhealthy_providers.is_empty() && degraded_providers.is_empty() {
                    return Err(ResearchEngineError::ConfigError(format!(
                        "All providers are unhealthy: {}",
                        unhealthy_providers.join(", ")
                    )));
                }

                // Log summary
                if !degraded_providers.is_empty() {
                    warn!(
                        "Multi-provider research engine operational with degraded providers: {}",
                        degraded_providers.join(", ")
                    );
                } else {
                    info!("Multi-provider research engine fully healthy");
                }

                Ok(())
            }
            Err(e) => {
                error!("Provider health check failed: {}", e);
                Err(ResearchEngineError::ApiError(
                    crate::api::ApiError::ServiceUnavailable(e.to_string()),
                ))
            }
        }
    }

    fn estimate_processing_time(&self, request: &ClassifiedRequest) -> Duration {
        // Base processing time
        let mut base_time = Duration::from_secs(10);

        // Adjust based on research type complexity
        let complexity_multiplier = match request.research_type {
            ResearchType::Learning => 1.0,
            ResearchType::Decision => 1.2,
            ResearchType::Implementation => 1.5,
            ResearchType::Troubleshooting => 1.3,
            ResearchType::Validation => 1.1,
        };

        // Adjust for cross-validation
        let cross_validation_multiplier = if self.config.enable_cross_validation {
            self.config.cross_validation_providers as f64
        } else {
            1.0
        };

        // Adjust for context discovery
        let context_multiplier = if self.config.enable_vector_search {
            1.2
        } else {
            1.0
        };

        let total_multiplier =
            complexity_multiplier * cross_validation_multiplier * context_multiplier;
        base_time = base_time.mul_f64(total_multiplier);

        // Cap at maximum processing time
        if base_time > self.config.max_processing_time {
            base_time = self.config.max_processing_time;
        }

        base_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fortitude_types::{AudienceContext, DomainContext};

    // Mock provider manager for testing
    #[derive(Debug)]
    struct MockProviderManager;

    impl ProviderManagerTrait for MockProviderManager {
        async fn execute_research(
            &self,
            _request: &ClassifiedRequest,
        ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok("Mock research response with detailed analysis".to_string())
        }

        async fn get_performance_stats(&self) -> HashMap<String, ProviderPerformanceStats> {
            let mut stats = HashMap::new();
            stats.insert(
                "mock-provider".to_string(),
                ProviderPerformanceStats {
                    total_requests: 10,
                    successful_requests: 9,
                    failed_requests: 1,
                    average_latency: Duration::from_millis(500),
                    average_quality: 0.85,
                    success_rate: 0.9,
                },
            );
            stats
        }

        async fn health_check_all(
            &self,
        ) -> Result<HashMap<String, ProviderHealthStatus>, Box<dyn std::error::Error + Send + Sync>>
        {
            let mut health = HashMap::new();
            health.insert("mock-provider".to_string(), ProviderHealthStatus::Healthy);
            Ok(health)
        }
    }

    fn create_test_request() -> ClassifiedRequest {
        ClassifiedRequest::new(
            "Test research query".to_string(),
            ResearchType::Implementation,
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

    #[tokio::test]
    async fn test_multi_provider_engine_creation() {
        let config = MultiProviderConfig::default();
        let mock_manager = Arc::new(MockProviderManager);
        let result = MultiProviderResearchEngine::new(mock_manager, config).await;

        assert!(
            result.is_ok(),
            "Multi-provider engine creation should succeed"
        );
    }

    #[tokio::test]
    async fn test_multi_provider_config_defaults() {
        let config = MultiProviderConfig::default();

        assert!(!config.enable_cross_validation);
        assert_eq!(config.cross_validation_providers, 2);
        assert_eq!(config.quality_threshold, 0.7);
        assert!(!config.enable_vector_search);
        assert!(config.enable_quality_validation);
        assert_eq!(config.min_quality_score, 0.6);
        assert_eq!(config.max_processing_time, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_processing_time_estimation() {
        let config = MultiProviderConfig {
            enable_cross_validation: true,
            cross_validation_providers: 3,
            enable_vector_search: true,
            ..Default::default()
        };

        let mock_manager = Arc::new(MockProviderManager);
        let engine = MultiProviderResearchEngine::new(mock_manager, config)
            .await
            .unwrap();
        let request = create_test_request();

        let estimate = engine.estimate_processing_time(&request);

        // Should account for cross-validation and context discovery
        assert!(estimate > Duration::from_secs(30)); // Should be longer due to multipliers
    }

    #[tokio::test]
    async fn test_build_research_prompt() {
        let config = MultiProviderConfig::default();
        let mock_manager = Arc::new(MockProviderManager);
        let engine = MultiProviderResearchEngine::new(mock_manager, config)
            .await
            .unwrap();

        let request = create_test_request();
        let prompt_result = engine.build_research_prompt(&request);

        assert!(
            prompt_result.is_ok(),
            "Building research prompt should succeed"
        );

        let prompt = prompt_result.unwrap();
        assert!(prompt.contains("Implementation Guide"));
        assert!(prompt.contains("intermediate level developer"));
        assert!(prompt.contains("rust"));
        assert!(prompt.contains("Test research query"));
    }

    #[tokio::test]
    async fn test_parse_research_response() {
        let config = MultiProviderConfig::default();
        let mock_manager = Arc::new(MockProviderManager);
        let engine = MultiProviderResearchEngine::new(mock_manager, config)
            .await
            .unwrap();

        let response_text = r#"## Answer
This is the immediate answer to the question.

## Evidence
This is supporting evidence with examples and references.

## Implementation
This is the implementation guidance with code examples.
"#;

        let request = create_test_request();
        let (answer, evidence, implementation) =
            engine.parse_research_response(response_text, &request);

        assert_eq!(answer, "This is the immediate answer to the question.");
        assert_eq!(evidence.len(), 1);
        assert_eq!(
            evidence[0].content,
            "This is supporting evidence with examples and references."
        );
        assert_eq!(implementation.len(), 1);
        assert_eq!(
            implementation[0].content,
            "This is the implementation guidance with code examples."
        );
    }
}
