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

// ABOUTME: Research engine that generates research results using Claude API
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::time::Instant;
use thiserror::Error;
use tracing::{debug, error, info, warn};

use crate::api::{ApiClient, ApiError, ClaudeClient, ClaudeConfig, ClaudeRequest, Message};
use crate::error_handling::PipelineError;
use crate::prompts::{DefaultTemplateFactory, ParameterValue, QualityValidator, TemplateRegistry};
use crate::vector::{
    FusionMethod, HybridSearchRequest, HybridSearchService, SearchOptions, SearchStrategy,
    VectorDocument,
};
use fortitude_types::{
    ClassifiedRequest, Detail, Evidence, ResearchMetadata, ResearchResult, ResearchType,
};

/// Errors that can occur during research generation
#[derive(Error, Debug)]
pub enum ResearchEngineError {
    #[error("API error: {0}")]
    ApiError(#[from] ApiError),

    #[error("Pipeline error: {0}")]
    PipelineError(#[from] PipelineError),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("Quality validation error: {0}")]
    QualityError(String),

    #[error("Timeout error: research took too long")]
    TimeoutError,

    #[error("Unknown research type: {0}")]
    UnknownResearchType(String),

    #[error("Vector search error: {0}")]
    VectorSearchError(String),

    #[error("Context discovery error: {0}")]
    ContextDiscoveryError(String),
}

impl ResearchEngineError {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::PipelineError(pipeline_error) => pipeline_error.is_retryable(),
            Self::ApiError(_) => true, // Most API errors are retryable
            Self::TimeoutError => true,
            Self::VectorSearchError(_) => true,
            Self::ContextDiscoveryError(_) => true,
            Self::ConfigError(_) => false,
            Self::TemplateError(_) => false,
            Self::QualityError(_) => false,
            Self::UnknownResearchType(_) => false,
        }
    }

    /// Convert to PipelineError for consistent error handling
    pub fn to_pipeline_error(&self) -> PipelineError {
        match self {
            Self::PipelineError(pipeline_error) => pipeline_error.clone(),
            Self::ApiError(api_error) => PipelineError::ExternalApi {
                api_name: "claude".to_string(),
                status_code: 500, // Default to server error
                message: api_error.to_string(),
                retry_after: None,
                correlation_id: uuid::Uuid::new_v4().to_string(),
            },
            Self::TimeoutError => PipelineError::Timeout {
                timeout_ms: 30000, // Default 30s timeout
                operation: "research_generation".to_string(),
                correlation_id: uuid::Uuid::new_v4().to_string(),
            },
            Self::VectorSearchError(msg) => PipelineError::Internal {
                message: format!("Vector search failed: {msg}"),
                correlation_id: uuid::Uuid::new_v4().to_string(),
                source: None,
            },
            _ => PipelineError::Internal {
                message: self.to_string(),
                correlation_id: uuid::Uuid::new_v4().to_string(),
                source: None,
            },
        }
    }
}

/// Research engine trait for generating research results
#[async_trait]
pub trait ResearchEngine: Send + Sync {
    /// Generate a research result for the given classified request
    async fn generate_research(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<ResearchResult, ResearchEngineError>;

    /// Generate research with context discovery using vector search
    async fn generate_research_with_context(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<ResearchResult, ResearchEngineError>;

    /// Discover relevant context for a research request
    async fn discover_context(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<Vec<VectorDocument>, ResearchEngineError>;

    /// Check if the research engine is available/healthy
    async fn health_check(&self) -> Result<(), ResearchEngineError>;

    /// Get estimated processing time for a request
    fn estimate_processing_time(&self, request: &ClassifiedRequest) -> std::time::Duration;
}

/// Claude-powered research engine implementation
pub struct ClaudeResearchEngine {
    client: ClaudeClient,
    config: ClaudeResearchConfig,
    template_registry: TemplateRegistry,
    quality_validator: QualityValidator,
    vector_search: Option<std::sync::Arc<HybridSearchService>>,
}

/// Configuration for Claude research engine
#[derive(Debug, Clone)]
pub struct ClaudeResearchConfig {
    /// Maximum tokens for research generation
    pub max_tokens: u32,

    /// Temperature for response generation
    pub temperature: f32,

    /// Maximum processing time before timeout
    pub max_processing_time: std::time::Duration,

    /// Enable quality validation
    pub enable_quality_validation: bool,

    /// Minimum quality score threshold
    pub min_quality_score: f64,

    /// System prompt template
    pub system_prompt: String,

    /// Enable vector search for context discovery
    pub enable_vector_search: bool,

    /// Maximum context documents to retrieve
    pub max_context_documents: usize,

    /// Context relevance threshold
    pub context_relevance_threshold: f64,
}

impl Default for ClaudeResearchConfig {
    fn default() -> Self {
        Self {
            max_tokens: 4000,
            temperature: 0.7,
            max_processing_time: std::time::Duration::from_secs(60),
            enable_quality_validation: true,
            min_quality_score: 0.6,
            system_prompt: Self::default_system_prompt(),
            enable_vector_search: false, // Default disabled for backward compatibility
            max_context_documents: 5,
            context_relevance_threshold: 0.7,
        }
    }
}

impl ClaudeResearchConfig {
    fn default_system_prompt() -> String {
        r#"You are a highly skilled research assistant specializing in AI-assisted software development. Your task is to generate comprehensive, accurate, and actionable research documentation.

Follow these guidelines:
1. Use progressive disclosure structure: Answer → Evidence → Implementation
2. Include semantic markup using XML-style tags for AI consumption
3. Provide specific, actionable information rather than generic advice
4. Include relevant code examples when appropriate
5. Cite specific sources and references when possible
6. Maintain professional, technical tone appropriate for software developers
7. Structure responses for maximum clarity and usability

Your responses should be comprehensive but concise, focusing on practical value for developers working on real projects."#.to_string()
    }
}

impl ClaudeResearchEngine {
    /// Create a new Claude research engine
    pub fn new(claude_config: ClaudeConfig) -> Result<Self, ResearchEngineError> {
        let client = ClaudeClient::new(claude_config)
            .map_err(|e| ResearchEngineError::ConfigError(e.to_string()))?;

        let config = ClaudeResearchConfig::default();
        let template_registry = DefaultTemplateFactory::create_default_registry();
        let quality_validator = QualityValidator::new();

        Ok(Self {
            client,
            config,
            template_registry,
            quality_validator,
            vector_search: None,
        })
    }

    /// Create a new Claude research engine with custom configuration
    pub fn with_config(
        claude_config: ClaudeConfig,
        research_config: ClaudeResearchConfig,
    ) -> Result<Self, ResearchEngineError> {
        let client = ClaudeClient::new(claude_config)
            .map_err(|e| ResearchEngineError::ConfigError(e.to_string()))?;

        let template_registry = DefaultTemplateFactory::create_default_registry();
        let quality_validator = QualityValidator::new();

        Ok(Self {
            client,
            config: research_config,
            template_registry,
            quality_validator,
            vector_search: None,
        })
    }

    /// Create a new Claude research engine with vector search capabilities
    pub fn with_vector_search(
        claude_config: ClaudeConfig,
        research_config: ClaudeResearchConfig,
        vector_search: std::sync::Arc<HybridSearchService>,
    ) -> Result<Self, ResearchEngineError> {
        let client = ClaudeClient::new(claude_config)
            .map_err(|e| ResearchEngineError::ConfigError(e.to_string()))?;

        let template_registry = DefaultTemplateFactory::create_default_registry();
        let quality_validator = QualityValidator::new();

        Ok(Self {
            client,
            config: research_config,
            template_registry,
            quality_validator,
            vector_search: Some(vector_search),
        })
    }

    /// Build the research prompt for a classified request using template system
    fn build_research_prompt(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<String, ResearchEngineError> {
        use crate::prompts::{ComplexityLevel, RegistryError};

        // Get appropriate template from registry
        let template = self
            .template_registry
            .get_best_for_type(
                &request.research_type,
                ComplexityLevel::Basic, // TODO: Determine complexity from request
            )
            .map_err(|e| match e {
                RegistryError::NoTemplatesForType(rt) => ResearchEngineError::TemplateError(
                    format!("No templates found for research type: {rt:?}"),
                ),
                RegistryError::TemplateNotFound(name) => {
                    ResearchEngineError::TemplateError(format!("Template not found: {name}"))
                }
                _ => ResearchEngineError::TemplateError(e.to_string()),
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
            .map_err(|e| ResearchEngineError::TemplateError(e.to_string()))?;

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

    /// Build the research prompt with context from vector search
    fn build_research_prompt_with_context(
        &self,
        request: &ClassifiedRequest,
        context_documents: &[VectorDocument],
    ) -> Result<String, ResearchEngineError> {
        // Start with the base prompt
        let base_prompt = self.build_research_prompt(request)?;

        if context_documents.is_empty() {
            return Ok(base_prompt);
        }

        // Build context section
        let mut context_section = String::new();
        context_section.push_str("\n\n## Relevant Context\n");
        context_section.push_str("The following information from previous research may be relevant to your response:\n\n");

        for (index, doc) in context_documents.iter().enumerate() {
            context_section.push_str(&format!(
                "### Context Document {} (Relevance: {:.2})\n",
                index + 1,
                doc.metadata.quality_score.unwrap_or(0.0)
            ));

            // Add a snippet of the document content
            let snippet = if doc.content.len() > 500 {
                format!("{}...", &doc.content[..497])
            } else {
                doc.content.clone()
            };
            context_section.push_str(&snippet);
            context_section.push_str("\n\n");
        }

        context_section.push_str("Please consider this context when generating your research response, but do not simply repeat it. Use it to provide more comprehensive and informed guidance.\n");

        // Combine base prompt with context
        Ok(format!("{base_prompt}{context_section}"))
    }

    /// Parse Claude response into structured research result
    fn parse_claude_response(
        &self,
        response_text: &str,
        _request: &ClassifiedRequest,
    ) -> Result<(String, Vec<Evidence>, Vec<Detail>), ResearchEngineError> {
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
                        source: "Claude API Research".to_string(),
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

        Ok((
            immediate_answer,
            supporting_evidence,
            implementation_details,
        ))
    }

    /// Validate the quality of generated research using the quality validator
    fn validate_research_quality(
        &self,
        result: &ResearchResult,
    ) -> Result<f64, ResearchEngineError> {
        match self.quality_validator.validate(result) {
            Ok(report) => {
                debug!(
                    "Research quality validation passed: {:.2} (progressive: {:.2}, markup: {:.2}, content: {:.2})",
                    report.overall_score,
                    report.progressive_disclosure_score,
                    report.semantic_markup_score,
                    report.content_quality_score
                );

                if !report.issues.is_empty() {
                    warn!("Quality validation issues: {:?}", report.issues);
                }

                Ok(report.overall_score)
            }
            Err(e) => {
                error!("Quality validation failed: {}", e);
                Err(ResearchEngineError::QualityError(e.to_string()))
            }
        }
    }
}

#[async_trait]
impl ResearchEngine for ClaudeResearchEngine {
    async fn generate_research(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<ResearchResult, ResearchEngineError> {
        let start_time = Instant::now();

        info!(
            "Generating research for query: '{}'",
            request.original_query
        );
        debug!(
            "Research type: {:?}, confidence: {:.2}",
            request.research_type, request.confidence
        );

        // Build the research prompt
        let user_prompt = self.build_research_prompt(request)?;

        // Create Claude API request
        let claude_request = ClaudeRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            max_tokens: self.config.max_tokens,
            messages: vec![Message {
                role: "user".to_string(),
                content: user_prompt,
            }],
            system: Some(self.config.system_prompt.clone()),
            temperature: Some(self.config.temperature),
            top_p: None,
            stop_sequences: None,
            stream: Some(false),
        };

        // Execute the API request
        let response = self.client.send_request(claude_request).await?;

        let processing_time = start_time.elapsed();

        // Check for timeout
        if processing_time > self.config.max_processing_time {
            return Err(ResearchEngineError::TimeoutError);
        }

        // Extract response text
        let response_text = response
            .content
            .first()
            .map(|content| content.text.clone())
            .unwrap_or_else(|| "No response content".to_string());

        // Parse response into structured format
        let (immediate_answer, supporting_evidence, implementation_details) =
            self.parse_claude_response(&response_text, request)?;

        // Create research metadata
        let metadata = ResearchMetadata {
            completed_at: Utc::now(),
            processing_time_ms: processing_time.as_millis() as u64,
            sources_consulted: vec![
                "Claude API".to_string(),
                "Anthropic AI Knowledge".to_string(),
            ],
            quality_score: 0.0,       // Will be set by validation
            cache_key: String::new(), // Will be set by pipeline
            tags: HashMap::new(),
        };

        // Create the research result
        let mut result = ResearchResult::new(
            request.clone(),
            immediate_answer,
            supporting_evidence,
            implementation_details,
            metadata,
        );

        // Validate quality if enabled
        if self.config.enable_quality_validation {
            let quality_score = self.validate_research_quality(&result)?;
            result.metadata.quality_score = quality_score;
        } else {
            result.metadata.quality_score = 0.8; // Default score
        }

        info!(
            "Generated research for '{}' in {:.2}s (quality: {:.2})",
            request.original_query,
            processing_time.as_secs_f64(),
            result.metadata.quality_score
        );

        Ok(result)
    }

    async fn generate_research_with_context(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<ResearchResult, ResearchEngineError> {
        // Use context-aware generation if vector search is enabled
        if self.config.enable_vector_search && self.vector_search.is_some() {
            self.generate_research_with_discovered_context(request)
                .await
        } else {
            // Fall back to regular generation
            self.generate_research(request).await
        }
    }

    async fn discover_context(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<Vec<VectorDocument>, ResearchEngineError> {
        self.discover_research_context(request).await
    }

    async fn health_check(&self) -> Result<(), ResearchEngineError> {
        debug!("Performing health check on Claude research engine");

        let health_status = self.client.health_check().await?;

        match health_status {
            crate::api::HealthStatus::Healthy => {
                debug!("Claude research engine is healthy");
                Ok(())
            }
            crate::api::HealthStatus::Degraded(reason) => {
                warn!("Claude research engine is degraded: {}", reason);
                Err(ResearchEngineError::ApiError(ApiError::ServiceUnavailable(
                    reason,
                )))
            }
            crate::api::HealthStatus::Unhealthy(reason) => {
                error!("Claude research engine is unhealthy: {}", reason);
                Err(ResearchEngineError::ApiError(ApiError::ServiceUnavailable(
                    reason,
                )))
            }
        }
    }

    fn estimate_processing_time(&self, request: &ClassifiedRequest) -> std::time::Duration {
        // Base processing time
        let mut base_time = std::time::Duration::from_secs(10);

        // Adjust based on research type complexity
        let complexity_multiplier = match request.research_type {
            ResearchType::Learning => 1.0,
            ResearchType::Decision => 1.2,
            ResearchType::Implementation => 1.5,
            ResearchType::Troubleshooting => 1.3,
            ResearchType::Validation => 1.1,
        };

        // Adjust based on query length
        let query_length_multiplier = if request.original_query.len() > 100 {
            1.2
        } else {
            1.0
        };

        // Adjust based on audience level
        let audience_multiplier = match request.audience_context.level.as_str() {
            "beginner" => 1.1,
            "intermediate" => 1.0,
            "advanced" => 1.2,
            _ => 1.0,
        };

        let total_multiplier =
            complexity_multiplier * query_length_multiplier * audience_multiplier;
        base_time = base_time.mul_f64(total_multiplier);

        // Cap at maximum processing time
        if base_time > self.config.max_processing_time {
            base_time = self.config.max_processing_time;
        }

        base_time
    }
}

impl ClaudeResearchEngine {
    /// Discover relevant context for a research request using vector search
    async fn discover_research_context(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<Vec<VectorDocument>, ResearchEngineError> {
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

    /// Generate research with enhanced context from vector search
    async fn generate_research_with_discovered_context(
        &self,
        request: &ClassifiedRequest,
    ) -> Result<ResearchResult, ResearchEngineError> {
        let start_time = Instant::now();

        info!(
            "Generating context-aware research for query: '{}'",
            request.original_query
        );

        // Discover relevant context
        let context_documents = self.discover_research_context(request).await?;

        // Build context-aware prompt
        let user_prompt = if context_documents.is_empty() {
            self.build_research_prompt(request)?
        } else {
            self.build_research_prompt_with_context(request, &context_documents)?
        };

        // Create Claude API request
        let claude_request = ClaudeRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            max_tokens: self.config.max_tokens,
            messages: vec![Message {
                role: "user".to_string(),
                content: user_prompt,
            }],
            system: Some(self.config.system_prompt.clone()),
            temperature: Some(self.config.temperature),
            top_p: None,
            stop_sequences: None,
            stream: Some(false),
        };

        // Execute the API request
        let response = self.client.send_request(claude_request).await?;

        let processing_time = start_time.elapsed();

        // Check for timeout
        if processing_time > self.config.max_processing_time {
            return Err(ResearchEngineError::TimeoutError);
        }

        // Extract response text
        let response_text = response
            .content
            .first()
            .map(|content| content.text.clone())
            .unwrap_or_else(|| "No response content".to_string());

        // Parse response into structured format
        let (immediate_answer, supporting_evidence, implementation_details) =
            self.parse_claude_response(&response_text, request)?;

        // Create enhanced research metadata
        let mut metadata = ResearchMetadata {
            completed_at: Utc::now(),
            processing_time_ms: processing_time.as_millis() as u64,
            sources_consulted: vec![
                "Claude API".to_string(),
                "Anthropic AI Knowledge".to_string(),
            ],
            quality_score: 0.0,       // Will be set by validation
            cache_key: String::new(), // Will be set by pipeline
            tags: HashMap::new(),
        };

        // Add context information to metadata
        if !context_documents.is_empty() {
            metadata
                .sources_consulted
                .push("Vector Search Context".to_string());
            metadata.tags.insert(
                "context_documents".to_string(),
                context_documents.len().to_string(),
            );
            metadata
                .tags
                .insert("enhanced_with_context".to_string(), "true".to_string());
        }

        // Create the research result
        let mut result = ResearchResult::new(
            request.clone(),
            immediate_answer,
            supporting_evidence,
            implementation_details,
            metadata,
        );

        // Validate quality if enabled
        if self.config.enable_quality_validation {
            let quality_score = self.validate_research_quality(&result)?;
            result.metadata.quality_score = quality_score;
        } else {
            result.metadata.quality_score = 0.8; // Default score
        }

        info!(
            "Generated context-aware research for '{}' in {:.2}s (quality: {:.2}, context: {})",
            request.original_query,
            processing_time.as_secs_f64(),
            result.metadata.quality_score,
            context_documents.len()
        );

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fortitude_types::{AudienceContext, DomainContext, ResearchType};

    #[test]
    fn test_research_engine_config() {
        let config = ClaudeResearchConfig::default();
        assert_eq!(config.max_tokens, 4000);
        assert_eq!(config.temperature, 0.7);
        assert!(config.enable_quality_validation);
        assert_eq!(config.min_quality_score, 0.6);
    }

    #[test]
    fn test_build_research_prompt() {
        let claude_config = ClaudeConfig::new("sk-test-key".to_string());
        let engine = ClaudeResearchEngine::new(claude_config).unwrap();

        let request = ClassifiedRequest::new(
            "How to implement async functions in Rust?".to_string(),
            ResearchType::Implementation,
            AudienceContext {
                level: "intermediate".to_string(),
                domain: "rust".to_string(),
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
        );

        let prompt = engine.build_research_prompt(&request).unwrap();

        assert!(prompt.contains("Implementation Guide"));
        assert!(prompt.contains("intermediate level developer"));
        assert!(prompt.contains("rust"));
        assert!(prompt.contains("tokio"));
        assert!(prompt.contains("How to implement async functions in Rust?"));
        assert!(prompt.contains("summary"));
        assert!(prompt.contains("evidence"));
        assert!(prompt.contains("implementation"));
    }

    #[test]
    fn test_parse_claude_response() {
        let claude_config = ClaudeConfig::new("sk-test-key".to_string());
        let engine = ClaudeResearchEngine::new(claude_config).unwrap();

        let response_text = r#"## Answer
This is the immediate answer to the question.

## Evidence
This is supporting evidence with examples and references.

## Implementation
This is the implementation guidance with code examples.
"#;

        let request = ClassifiedRequest::new(
            "test query".to_string(),
            ResearchType::Implementation,
            AudienceContext::default(),
            DomainContext::default(),
            0.8,
            vec![],
        );

        let (answer, evidence, implementation) = engine
            .parse_claude_response(response_text, &request)
            .unwrap();

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

    #[test]
    fn test_estimate_processing_time() {
        let claude_config = ClaudeConfig::new("sk-test-key".to_string());
        let engine = ClaudeResearchEngine::new(claude_config).unwrap();

        let request = ClassifiedRequest::new(
            "Simple question".to_string(),
            ResearchType::Learning,
            AudienceContext {
                level: "beginner".to_string(),
                domain: "rust".to_string(),
                format: "markdown".to_string(),
            },
            DomainContext::default(),
            0.8,
            vec![],
        );

        let estimate = engine.estimate_processing_time(&request);

        // Should be around 10 seconds * 1.0 (learning) * 1.0 (short query) * 1.1 (beginner) = ~11 seconds
        assert!(estimate >= std::time::Duration::from_secs(10));
        assert!(estimate <= std::time::Duration::from_secs(15));
    }
}
