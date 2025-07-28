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

// ABOUTME: Vector database module for semantic search capabilities
//! This module provides vector database integration using Qdrant for semantic
//! search capabilities, including client management, configuration, and error handling.

pub mod cache;
pub mod client;
pub mod config;
pub mod connection_pool;
pub mod embeddings;
pub mod error;
pub mod hybrid;
pub mod migration;
pub mod optimized_config;
pub mod optimized_embeddings;
pub mod performance;
pub mod regression_detection;
pub mod search;
pub mod storage;
pub mod utils;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod storage_tests;

pub use cache::*;
pub use client::*;
pub use config::*;
pub use embeddings::*;
pub use error::*;
pub use storage::*;
pub use utils::*;

// Re-export search module selectively to avoid conflicts
pub use search::{
    BatchSearchRequest, BatchSearchResult, BatchSearchStats,
    FilterOperation as SemanticFilterOperation, MatchMetadata, SearchAnalytics,
    SearchExecutionStats, SearchExplanation, SearchFilter as SemanticSearchFilter, SearchOptions,
    SearchQueryMetadata, SearchResult, SearchResultSet, SemanticSearchConfig,
    SemanticSearchOperations, SemanticSearchService, SuggestionRequest,
};

// Re-export hybrid search functionality
pub use hybrid::{
    FusionMethod, HybridMatchMetadata, HybridSearchAnalytics, HybridSearchConfig,
    HybridSearchExplanation, HybridSearchOperations, HybridSearchRequest, HybridSearchResult,
    HybridSearchResultSet, HybridSearchService, HybridSearchStats, KeywordSearchResult,
    KeywordSearcher, PerformanceMetrics, QueryAnalysis, QueryType, SearchSources, SearchStrategy,
    TermMatch,
};

// Re-export migration functionality
pub use migration::{
    BatchStatistics, DataConverter, FailedItem, MigrationConfig, MigrationError, MigrationProgress,
    MigrationResult, MigrationService, MigrationSource, MigrationState, MigrationStatistics,
    MigrationStatus, MigrationSummary, RollbackResult, ValidationError, ValidationLevel,
    ValidationResult, ValidationSeverity, ValidationStatistics,
};
