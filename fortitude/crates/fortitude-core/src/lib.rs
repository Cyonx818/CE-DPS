// ABOUTME: Core business logic for the Fortitude research system
//! This crate contains the core business logic for the Fortitude research
//! system, including classification engines, storage systems, and pipeline
//! orchestration.

pub mod api;
pub mod classification;
pub mod claude_code_provider;
pub mod claude_code_research_engine;
pub mod claude_code_integration_example;
pub mod multi_provider_research_engine;
pub mod pipeline;
pub mod prompts;
pub mod research_engine;
pub mod research_feedback;
pub mod storage;
pub mod vector;

#[cfg(test)]
mod integration_tests;

// NOTE: Example disabled due to outdated API usage - needs updating
// pub mod enhanced_pipeline_example;

// Re-export specific types to avoid naming conflicts
pub use api::{ApiClient, ApiConfig, HealthStatus, RateLimitConfig, RequestCost, RetryConfig};
pub use classification::*;
pub use claude_code_provider::{ClaudeCodeProvider, ClaudeCodeProviderConfig};
pub use claude_code_research_engine::{ClaudeCodeResearchEngine, ClaudeCodeResearchEngineConfig};
pub use multi_provider_research_engine::{
    MultiProviderConfig, MultiProviderResearchEngine, MultiProviderResearchError,
};
pub use pipeline::*;
pub use prompts::*;
pub use research_engine::*;
pub use research_feedback::*;
pub use storage::*;
pub use vector::{
    BatchSearchRequest,
    BatchSearchResult,
    DocumentMetadata,
    FusionMethod,
    HybridMatchMetadata,
    HybridSearchAnalytics,
    HybridSearchConfig,
    HybridSearchExplanation,
    HybridSearchOperations,
    HybridSearchRequest,
    HybridSearchResult,
    HybridSearchResultSet,
    // Enhanced hybrid search exports
    HybridSearchService,
    HybridSearchStats,
    KeywordSearchResult,
    KeywordSearcher,
    PerformanceMetrics,
    QdrantClient,
    QueryAnalysis,
    QueryType,
    SearchExplanation,
    SearchOptions,
    SearchResult,
    SearchResultSet,
    SearchSources,
    SearchStrategy,
    SemanticFilterOperation,
    SemanticSearchConfig,
    SemanticSearchFilter,
    SemanticSearchOperations,
    SemanticSearchService,
    SuggestionRequest,
    TermMatch,
    VectorConfig,
    VectorDocument,
    VectorError,
    VectorOperation,
    VectorRequest,
    VectorResponse,
    VectorResult,
    // Vector storage exports
    VectorStorageService,
};
