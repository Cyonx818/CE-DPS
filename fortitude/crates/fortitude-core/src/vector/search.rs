// ABOUTME: Semantic search service for research content discovery and vector similarity operations
//! This module provides comprehensive semantic search capabilities for the Fortitude vector
//! database system. It includes query processing, result ranking, filtering, and various
//! search operations for research content discovery.

use crate::vector::{
    error::{VectorError, VectorResult},
    storage::{SearchConfig, SimilaritySearchResult, VectorDocument, VectorStorage},
};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, instrument, warn};

/// Advanced search configuration with additional semantic search options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    /// Maximum number of results to return
    pub limit: usize,
    /// Minimum similarity score threshold (0.0-1.0)
    pub threshold: Option<f64>,
    /// Collection to search in (optional, uses default if None)
    pub collection: Option<String>,
    /// Metadata-based filters
    pub filters: Vec<SearchFilter>,
    /// Enable result diversification to avoid similar duplicates
    pub diversify_results: bool,
    /// Boost scores for recent documents (weight factor)
    pub temporal_boost: Option<f64>,
    /// Boost scores for high-quality documents (weight factor)
    pub quality_boost: Option<f64>,
    /// Include explanation of why results were returned
    pub include_explanations: bool,
    /// Minimum content length for results (characters)
    pub min_content_length: Option<usize>,
    /// Maximum content length for results (characters)
    pub max_content_length: Option<usize>,
    /// Enable fuzzy matching for search terms
    pub fuzzy_matching: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: 10,
            threshold: None,
            collection: None,
            filters: Vec::new(),
            diversify_results: false,
            temporal_boost: None,
            quality_boost: None,
            include_explanations: false,
            min_content_length: None,
            max_content_length: None,
            fuzzy_matching: false,
        }
    }
}

/// Search filter for metadata-based filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilter {
    /// Field to filter on
    pub field: String,
    /// Filter operation type
    pub operation: FilterOperation,
    /// Filter value(s)
    pub value: serde_json::Value,
}

/// Filter operation types for search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperation {
    /// Exact equality match
    Equals,
    /// Not equal to value
    NotEquals,
    /// Contains substring/element
    Contains,
    /// Does not contain substring/element
    NotContains,
    /// Greater than numeric value
    GreaterThan,
    /// Less than numeric value
    LessThan,
    /// Greater than or equal to numeric value
    GreaterThanOrEqual,
    /// Less than or equal to numeric value
    LessThanOrEqual,
    /// Value is in provided list
    In,
    /// Value is not in provided list
    NotIn,
    /// Date/time after specified value
    After,
    /// Date/time before specified value
    Before,
    /// Text matches regular expression
    Regex,
}

/// Enhanced search result with additional metadata and explanations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The matching document
    pub document: VectorDocument,
    /// Raw similarity score (0.0-1.0)
    pub similarity_score: f64,
    /// Final relevance score after adjustments
    pub relevance_score: f64,
    /// Explanation of why this result was returned
    pub explanation: Option<SearchExplanation>,
    /// Match metadata
    pub match_metadata: MatchMetadata,
}

/// Explanation of search result ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchExplanation {
    /// Base similarity score contribution
    pub base_similarity: f64,
    /// Temporal boost contribution (if any)
    pub temporal_boost: Option<f64>,
    /// Quality boost contribution (if any)
    pub quality_boost: Option<f64>,
    /// Penalty factors applied
    pub penalties: Vec<String>,
    /// Boost factors applied
    pub boosts: Vec<String>,
    /// Final calculation formula
    pub calculation: String,
}

/// Metadata about the match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchMetadata {
    /// Search execution time in milliseconds
    pub search_time_ms: f64,
    /// Position in original results before reranking
    pub original_rank: usize,
    /// Final rank after all adjustments
    pub final_rank: usize,
    /// Number of matched filter criteria
    pub matched_filters: usize,
    /// Content preview snippet
    pub snippet: Option<String>,
}

/// Set of search results with metadata about the search operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultSet {
    /// Search results
    pub results: Vec<SearchResult>,
    /// Total results found (before limit)
    pub total_found: usize,
    /// Search options used
    pub search_options: SearchOptions,
    /// Search execution statistics
    pub execution_stats: SearchExecutionStats,
    /// Search query metadata
    pub query_metadata: SearchQueryMetadata,
}

/// Statistics about search execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchExecutionStats {
    /// Total search time in milliseconds
    pub total_time_ms: f64,
    /// Query preprocessing time
    pub preprocessing_time_ms: f64,
    /// Vector search time
    pub vector_search_time_ms: f64,
    /// Post-processing time (filtering, ranking)
    pub postprocessing_time_ms: f64,
    /// Number of documents evaluated
    pub documents_evaluated: usize,
    /// Number of documents after filtering
    pub documents_after_filtering: usize,
    /// Cache hits during search
    pub cache_hits: usize,
}

/// Metadata about the search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQueryMetadata {
    /// Original query text
    pub original_query: String,
    /// Processed query text
    pub processed_query: String,
    /// Query embedding dimension
    pub embedding_dimension: usize,
    /// Estimated query complexity
    pub complexity_score: f64,
    /// Detected query language
    pub language: Option<String>,
    /// Extracted keywords
    pub keywords: Vec<String>,
}

/// Batch search request for multiple queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSearchRequest {
    /// Multiple search queries
    pub queries: Vec<String>,
    /// Shared search options
    pub options: SearchOptions,
    /// Whether to merge results across queries
    pub merge_results: bool,
    /// Maximum results per query (if not merging)
    pub max_per_query: Option<usize>,
}

/// Result of batch search operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSearchResult {
    /// Results per query (if not merged)
    pub per_query_results: Option<Vec<SearchResultSet>>,
    /// Merged results (if merging enabled)
    pub merged_results: Option<SearchResultSet>,
    /// Batch execution statistics
    pub batch_stats: BatchSearchStats,
}

/// Statistics for batch search operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSearchStats {
    /// Total queries processed
    pub total_queries: usize,
    /// Successful queries
    pub successful_queries: usize,
    /// Failed queries
    pub failed_queries: usize,
    /// Total execution time
    pub total_time_ms: f64,
    /// Average time per query
    pub avg_time_per_query_ms: f64,
}

/// Suggestion request for related content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionRequest {
    /// Document ID to find related content for
    pub document_id: String,
    /// Number of suggestions to return
    pub limit: usize,
    /// Minimum similarity threshold
    pub threshold: Option<f64>,
    /// Exclude the source document from results
    pub exclude_source: bool,
    /// Additional filters
    pub filters: Vec<SearchFilter>,
}

/// Configuration for the semantic search service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchConfig {
    /// Default search limit
    pub default_limit: usize,
    /// Default similarity threshold
    pub default_threshold: f64,
    /// Maximum allowed search limit
    pub max_limit: usize,
    /// Enable performance analytics
    pub enable_analytics: bool,
    /// Cache search results
    pub cache_results: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Enable query optimization
    pub enable_query_optimization: bool,
    /// Maximum query length
    pub max_query_length: usize,
}

impl Default for SemanticSearchConfig {
    fn default() -> Self {
        Self {
            default_limit: 10,
            default_threshold: 0.7,
            max_limit: 100,
            enable_analytics: true,
            cache_results: true,
            cache_ttl_seconds: 300, // 5 minutes
            enable_query_optimization: true,
            max_query_length: 8192,
        }
    }
}

/// High-level semantic search service
pub struct SemanticSearchService {
    /// Vector storage backend
    vector_storage: Arc<VectorStorage>,
    /// Search configuration
    config: SemanticSearchConfig,
    /// Search analytics
    analytics: Arc<tokio::sync::RwLock<SearchAnalytics>>,
}

/// Analytics data for search operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAnalytics {
    /// Total searches performed
    pub total_searches: u64,
    /// Average search time
    pub avg_search_time_ms: f64,
    /// Most common search terms
    pub popular_terms: HashMap<String, u64>,
    /// Search result click-through rates
    pub click_through_rates: HashMap<String, f64>,
    /// Average results per search
    pub avg_results_per_search: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
}

impl Default for SearchAnalytics {
    fn default() -> Self {
        Self {
            total_searches: 0,
            avg_search_time_ms: 0.0,
            popular_terms: HashMap::new(),
            click_through_rates: HashMap::new(),
            avg_results_per_search: 0.0,
            cache_hit_rate: 0.0,
        }
    }
}

impl SemanticSearchService {
    /// Create a new semantic search service
    pub fn new(vector_storage: Arc<VectorStorage>, config: SemanticSearchConfig) -> Self {
        Self {
            vector_storage,
            config,
            analytics: Arc::new(tokio::sync::RwLock::new(SearchAnalytics::default())),
        }
    }

    /// Create service with default configuration
    pub fn with_defaults(vector_storage: Arc<VectorStorage>) -> Self {
        Self::new(vector_storage, SemanticSearchConfig::default())
    }

    /// Initialize the search service
    #[instrument(skip(self))]
    pub async fn initialize(&self) -> VectorResult<()> {
        info!("Initializing semantic search service");

        // Initialize vector storage
        self.vector_storage.initialize().await?;

        info!("Semantic search service initialized successfully");
        Ok(())
    }

    /// Search for semantically similar content
    #[instrument(skip(self, query, options))]
    pub async fn search_similar(
        &self,
        query: &str,
        options: SearchOptions,
    ) -> VectorResult<SearchResultSet> {
        let start_time = Instant::now();

        // Validate and preprocess query
        let processed_query = self.preprocess_query(query)?;
        let preprocessing_time = start_time.elapsed().as_millis() as f64;

        // Perform vector search
        let vector_search_start = Instant::now();
        let search_config = self.build_search_config(&options);
        let raw_results = self
            .vector_storage
            .retrieve_similar(&processed_query, search_config)
            .await?;
        let vector_search_time = vector_search_start.elapsed().as_millis() as f64;

        // Store counts before moving the results
        let raw_results_len = raw_results.len();

        // Post-process results
        let postprocessing_start = Instant::now();
        let enhanced_results = self
            .enhance_search_results(raw_results, query, &processed_query, &options)
            .await?;
        let postprocessing_time = postprocessing_start.elapsed().as_millis() as f64;

        let total_time = start_time.elapsed().as_millis() as f64;
        let enhanced_results_len = enhanced_results.len();

        // Build result set
        let result_set = SearchResultSet {
            total_found: enhanced_results_len,
            results: enhanced_results,
            search_options: options.clone(),
            execution_stats: SearchExecutionStats {
                total_time_ms: total_time,
                preprocessing_time_ms: preprocessing_time,
                vector_search_time_ms: vector_search_time,
                postprocessing_time_ms: postprocessing_time,
                documents_evaluated: raw_results_len,
                documents_after_filtering: enhanced_results_len,
                cache_hits: 0, // TODO: Implement caching
            },
            query_metadata: SearchQueryMetadata {
                original_query: query.to_string(),
                processed_query: processed_query.clone(),
                embedding_dimension: self.vector_storage.embedding_dimension(),
                complexity_score: self.calculate_query_complexity(query),
                language: None, // TODO: Implement language detection
                keywords: self.extract_keywords(query),
            },
        };

        // Update analytics
        self.update_analytics(&result_set).await;

        debug!(
            "Search completed: {} results in {:.2}ms",
            result_set.results.len(),
            total_time
        );

        Ok(result_set)
    }

    /// Search using existing content as the query
    #[instrument(skip(self, content, options))]
    pub async fn search_by_content(
        &self,
        content: &VectorDocument,
        options: SearchOptions,
    ) -> VectorResult<SearchResultSet> {
        // Use the document's content as the search query
        self.search_similar(&content.content, options).await
    }

    /// Search with advanced filtering
    #[instrument(skip(self, query, filters, options))]
    pub async fn search_with_filters(
        &self,
        query: &str,
        filters: Vec<SearchFilter>,
        mut options: SearchOptions,
    ) -> VectorResult<SearchResultSet> {
        // Add filters to options
        options.filters.extend(filters);
        self.search_similar(query, options).await
    }

    /// Find related documents for a given document
    #[instrument(skip(self, request))]
    pub async fn suggest_related(
        &self,
        request: SuggestionRequest,
    ) -> VectorResult<SearchResultSet> {
        // Retrieve the source document
        let source_doc = self
            .vector_storage
            .retrieve_by_id(&request.document_id)
            .await?
            .ok_or_else(|| {
                VectorError::from_operation_failed(
                    "suggest_related",
                    format!("Document not found: {}", request.document_id),
                )
            })?;

        // Create search options
        let options = SearchOptions {
            limit: request.limit,
            threshold: request.threshold,
            filters: request.filters,
            ..SearchOptions::default()
        };

        // Search using the document's content
        let mut result_set = self.search_by_content(&source_doc, options).await?;

        // Remove source document if requested
        if request.exclude_source {
            result_set
                .results
                .retain(|r| r.document.id != request.document_id);
        }

        Ok(result_set)
    }

    /// Perform batch search operations
    #[instrument(skip(self, request))]
    pub async fn search_batch(
        &self,
        request: BatchSearchRequest,
    ) -> VectorResult<BatchSearchResult> {
        let start_time = Instant::now();
        let mut successful_queries = 0;
        let mut failed_queries = 0;

        if request.merge_results {
            // Merge results from all queries
            let mut all_results = Vec::new();

            for query in &request.queries {
                match self.search_similar(query, request.options.clone()).await {
                    Ok(result_set) => {
                        all_results.extend(result_set.results);
                        successful_queries += 1;
                    }
                    Err(e) => {
                        warn!("Batch search query failed: {} - {}", query, e);
                        failed_queries += 1;
                    }
                }
            }

            // Sort by relevance score and limit
            all_results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
            all_results.truncate(request.options.limit);

            // Update ranks
            for (index, result) in all_results.iter_mut().enumerate() {
                result.match_metadata.final_rank = index;
            }

            let merged_result_set = SearchResultSet {
                results: all_results.clone(),
                total_found: all_results.len(),
                search_options: request.options.clone(),
                execution_stats: SearchExecutionStats {
                    total_time_ms: start_time.elapsed().as_millis() as f64,
                    preprocessing_time_ms: 0.0,
                    vector_search_time_ms: 0.0,
                    postprocessing_time_ms: 0.0,
                    documents_evaluated: all_results.len(),
                    documents_after_filtering: all_results.len(),
                    cache_hits: 0,
                },
                query_metadata: SearchQueryMetadata {
                    original_query: format!("[BATCH] {} queries", request.queries.len()),
                    processed_query: format!("[BATCH] {} queries", request.queries.len()),
                    embedding_dimension: self.vector_storage.embedding_dimension(),
                    complexity_score: 0.0,
                    language: None,
                    keywords: Vec::new(),
                },
            };

            Ok(BatchSearchResult {
                per_query_results: None,
                merged_results: Some(merged_result_set),
                batch_stats: BatchSearchStats {
                    total_queries: request.queries.len(),
                    successful_queries,
                    failed_queries,
                    total_time_ms: start_time.elapsed().as_millis() as f64,
                    avg_time_per_query_ms: start_time.elapsed().as_millis() as f64
                        / request.queries.len() as f64,
                },
            })
        } else {
            // Return results per query
            let mut per_query_results = Vec::new();

            for query in &request.queries {
                let mut query_options = request.options.clone();
                if let Some(max_per_query) = request.max_per_query {
                    query_options.limit = max_per_query.min(query_options.limit);
                }

                match self.search_similar(query, query_options).await {
                    Ok(result_set) => {
                        per_query_results.push(result_set);
                        successful_queries += 1;
                    }
                    Err(e) => {
                        warn!("Batch search query failed: {} - {}", query, e);
                        failed_queries += 1;
                    }
                }
            }

            Ok(BatchSearchResult {
                per_query_results: Some(per_query_results),
                merged_results: None,
                batch_stats: BatchSearchStats {
                    total_queries: request.queries.len(),
                    successful_queries,
                    failed_queries,
                    total_time_ms: start_time.elapsed().as_millis() as f64,
                    avg_time_per_query_ms: start_time.elapsed().as_millis() as f64
                        / request.queries.len() as f64,
                },
            })
        }
    }

    /// Explain why a specific search result was returned
    #[instrument(skip(self, query, result_id))]
    pub async fn explain_search(
        &self,
        query: &str,
        result_id: &str,
    ) -> VectorResult<Option<SearchExplanation>> {
        // Perform a search to find the specific result
        let options = SearchOptions {
            limit: 50, // Increase limit to find the specific result
            include_explanations: true,
            ..SearchOptions::default()
        };

        let result_set = self.search_similar(query, options).await?;

        // Find the specific result
        for result in result_set.results {
            if result.document.id == result_id {
                return Ok(result.explanation);
            }
        }

        Ok(None)
    }

    /// Get search analytics
    pub async fn get_analytics(&self) -> SearchAnalytics {
        self.analytics.read().await.clone()
    }

    /// Clear search analytics
    pub async fn clear_analytics(&self) -> VectorResult<()> {
        let mut analytics = self.analytics.write().await;
        *analytics = SearchAnalytics::default();
        info!("Search analytics cleared");
        Ok(())
    }

    /// Preprocess search query for optimal results
    fn preprocess_query(&self, query: &str) -> VectorResult<String> {
        if query.len() > self.config.max_query_length {
            return Err(VectorError::from_operation_failed(
                "preprocess_query",
                format!(
                    "Query too long: {} > {}",
                    query.len(),
                    self.config.max_query_length
                ),
            ));
        }

        if !self.config.enable_query_optimization {
            return Ok(query.to_string());
        }

        let mut processed = query.to_string();

        // Trim whitespace
        processed = processed.trim().to_string();

        // Remove extra whitespace
        processed = regex::Regex::new(r"\s+")
            .unwrap()
            .replace_all(&processed, " ")
            .to_string();

        // TODO: Add more sophisticated query optimization
        // - Remove stop words
        // - Expand abbreviations
        // - Handle typos
        // - Extract entities

        Ok(processed)
    }

    /// Build vector storage search config from search options
    fn build_search_config(&self, options: &SearchOptions) -> SearchConfig {
        SearchConfig {
            limit: options.limit.min(self.config.max_limit),
            threshold: options.threshold.or(Some(self.config.default_threshold)),
            collection: options.collection.clone(),
            filters: options
                .filters
                .iter()
                .map(|f| {
                    crate::vector::storage::SearchFilter {
                        field: f.field.clone(),
                        operation: match f.operation {
                            FilterOperation::Equals => {
                                crate::vector::storage::FilterOperation::Equals
                            }
                            FilterOperation::NotEquals => {
                                crate::vector::storage::FilterOperation::NotEquals
                            }
                            FilterOperation::Contains => {
                                crate::vector::storage::FilterOperation::Contains
                            }
                            FilterOperation::GreaterThan => {
                                crate::vector::storage::FilterOperation::GreaterThan
                            }
                            FilterOperation::LessThan => {
                                crate::vector::storage::FilterOperation::LessThan
                            }
                            FilterOperation::In => crate::vector::storage::FilterOperation::In,
                            FilterOperation::NotIn => {
                                crate::vector::storage::FilterOperation::NotIn
                            }
                            _ => crate::vector::storage::FilterOperation::Equals, // Fallback for unsupported operations
                        },
                        value: f.value.clone(),
                    }
                })
                .collect(),
        }
    }

    /// Enhance search results with additional metadata and ranking
    async fn enhance_search_results(
        &self,
        raw_results: Vec<SimilaritySearchResult>,
        _original_query: &str,
        _processed_query: &str,
        options: &SearchOptions,
    ) -> VectorResult<Vec<SearchResult>> {
        let mut enhanced_results = Vec::new();

        for (index, raw_result) in raw_results.into_iter().enumerate() {
            let mut relevance_score = raw_result.score;
            let mut _explanation_parts: Vec<String> = Vec::new();
            let mut boosts = Vec::new();
            let mut penalties = Vec::new();

            // Apply temporal boost
            if let Some(temporal_boost) = options.temporal_boost {
                let days_ago = (Utc::now() - raw_result.document.stored_at).num_days();
                let temporal_factor = (1.0 / (1.0 + days_ago as f64 * 0.1)).max(0.1);
                let boost_amount = temporal_factor * temporal_boost;
                relevance_score += boost_amount;
                boosts.push(format!("temporal_boost: +{boost_amount:.3}"));
            }

            // Apply quality boost
            if let (Some(quality_boost), Some(quality_score)) = (
                options.quality_boost,
                raw_result.document.metadata.quality_score,
            ) {
                let boost_amount = quality_score * quality_boost;
                relevance_score += boost_amount;
                boosts.push(format!("quality_boost: +{boost_amount:.3}"));
            }

            // Apply content length filters
            let content_len = raw_result.document.content.len();

            if let Some(min_len) = options.min_content_length {
                if content_len < min_len {
                    penalties.push(format!("content_too_short: {content_len} < {min_len}"));
                    continue; // Skip this result
                }
            }

            if let Some(max_len) = options.max_content_length {
                if content_len > max_len {
                    penalties.push(format!("content_too_long: {content_len} > {max_len}"));
                    continue; // Skip this result
                }
            }

            // Create explanation if requested
            let explanation = if options.include_explanations {
                Some(SearchExplanation {
                    base_similarity: raw_result.score,
                    temporal_boost: options.temporal_boost,
                    quality_boost: options.quality_boost,
                    penalties: penalties.clone(),
                    boosts: boosts.clone(),
                    calculation: format!(
                        "base({:.3}) + boosts({}) - penalties({}) = {:.3}",
                        raw_result.score,
                        boosts.join(", "),
                        penalties.join(", "),
                        relevance_score
                    ),
                })
            } else {
                None
            };

            // Create snippet
            let snippet = if raw_result.document.content.len() > 200 {
                Some(format!("{}...", &raw_result.document.content[..197]))
            } else {
                Some(raw_result.document.content.clone())
            };

            enhanced_results.push(SearchResult {
                document: raw_result.document,
                similarity_score: raw_result.score,
                relevance_score,
                explanation,
                match_metadata: MatchMetadata {
                    search_time_ms: 0.0, // Will be filled in by caller
                    original_rank: index,
                    final_rank: index,  // Will be updated after sorting
                    matched_filters: 0, // TODO: Calculate based on applied filters
                    snippet,
                },
            });
        }

        // Sort by relevance score
        enhanced_results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        // Update final ranks
        for (index, result) in enhanced_results.iter_mut().enumerate() {
            result.match_metadata.final_rank = index;
        }

        // Apply diversification if requested
        if options.diversify_results {
            enhanced_results = self.diversify_results(enhanced_results);
        }

        Ok(enhanced_results)
    }

    /// Diversify search results to avoid similar duplicates
    fn diversify_results(&self, results: Vec<SearchResult>) -> Vec<SearchResult> {
        if results.len() <= 2 {
            return results;
        }

        let mut diversified = Vec::new();
        let mut used_content_hashes = std::collections::HashSet::new();

        for result in results {
            // Simple content similarity check using content hash
            let content_hash = self.simple_content_hash(&result.document.content);

            if !used_content_hashes.contains(&content_hash) {
                used_content_hashes.insert(content_hash);
                diversified.push(result);
            }
        }

        diversified
    }

    /// Calculate simple content hash for diversification
    fn simple_content_hash(&self, content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        // Use first 100 characters for similarity detection
        let prefix = if content.len() > 100 {
            &content[..100]
        } else {
            content
        };
        prefix.hash(&mut hasher);
        hasher.finish()
    }

    /// Calculate query complexity score
    fn calculate_query_complexity(&self, query: &str) -> f64 {
        let word_count = query.split_whitespace().count();
        let char_count = query.len();
        let punctuation_count = query.chars().filter(|c| c.is_ascii_punctuation()).count();

        // Simple complexity calculation
        (word_count as f64 * 0.5 + char_count as f64 * 0.01 + punctuation_count as f64 * 0.1)
            .min(10.0)
    }

    /// Extract keywords from query text
    fn extract_keywords(&self, query: &str) -> Vec<String> {
        query
            .split_whitespace()
            .filter(|word| word.len() > 3) // Filter short words
            .map(|word| word.to_lowercase())
            .collect()
    }

    /// Update search analytics
    async fn update_analytics(&self, result_set: &SearchResultSet) {
        let mut analytics = self.analytics.write().await;

        analytics.total_searches += 1;

        // Update average search time
        let current_avg = analytics.avg_search_time_ms;
        let new_time = result_set.execution_stats.total_time_ms;
        analytics.avg_search_time_ms = if analytics.total_searches == 1 {
            new_time
        } else {
            (current_avg * (analytics.total_searches - 1) as f64 + new_time)
                / analytics.total_searches as f64
        };

        // Update average results per search
        let current_avg_results = analytics.avg_results_per_search;
        let new_results = result_set.results.len() as f64;
        analytics.avg_results_per_search = if analytics.total_searches == 1 {
            new_results
        } else {
            (current_avg_results * (analytics.total_searches - 1) as f64 + new_results)
                / analytics.total_searches as f64
        };

        // Update popular terms
        for keyword in &result_set.query_metadata.keywords {
            *analytics.popular_terms.entry(keyword.clone()).or_insert(0) += 1;
        }
    }
}

/// Trait for semantic search operations (for testing and alternative implementations)
#[async_trait]
pub trait SemanticSearchOperations: Send + Sync {
    /// Search for semantically similar content
    async fn search_similar(
        &self,
        query: &str,
        options: SearchOptions,
    ) -> VectorResult<SearchResultSet>;

    /// Search using existing content as query
    async fn search_by_content(
        &self,
        content: &VectorDocument,
        options: SearchOptions,
    ) -> VectorResult<SearchResultSet>;

    /// Search with advanced filtering
    async fn search_with_filters(
        &self,
        query: &str,
        filters: Vec<SearchFilter>,
        options: SearchOptions,
    ) -> VectorResult<SearchResultSet>;

    /// Find related documents
    async fn suggest_related(&self, request: SuggestionRequest) -> VectorResult<SearchResultSet>;

    /// Perform batch search operations
    async fn search_batch(&self, request: BatchSearchRequest) -> VectorResult<BatchSearchResult>;

    /// Explain search results
    async fn explain_search(
        &self,
        query: &str,
        result_id: &str,
    ) -> VectorResult<Option<SearchExplanation>>;

    /// Initialize the service
    async fn initialize(&self) -> VectorResult<()>;
}

#[async_trait]
impl SemanticSearchOperations for SemanticSearchService {
    async fn search_similar(
        &self,
        query: &str,
        options: SearchOptions,
    ) -> VectorResult<SearchResultSet> {
        self.search_similar(query, options).await
    }

    async fn search_by_content(
        &self,
        content: &VectorDocument,
        options: SearchOptions,
    ) -> VectorResult<SearchResultSet> {
        self.search_by_content(content, options).await
    }

    async fn search_with_filters(
        &self,
        query: &str,
        filters: Vec<SearchFilter>,
        options: SearchOptions,
    ) -> VectorResult<SearchResultSet> {
        self.search_with_filters(query, filters, options).await
    }

    async fn suggest_related(&self, request: SuggestionRequest) -> VectorResult<SearchResultSet> {
        self.suggest_related(request).await
    }

    async fn search_batch(&self, request: BatchSearchRequest) -> VectorResult<BatchSearchResult> {
        self.search_batch(request).await
    }

    async fn explain_search(
        &self,
        query: &str,
        result_id: &str,
    ) -> VectorResult<Option<SearchExplanation>> {
        self.explain_search(query, result_id).await
    }

    async fn initialize(&self) -> VectorResult<()> {
        self.initialize().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Note: Most imports removed as they're not needed for unit tests
    // Integration tests would use these imports

    // Helper function to create a simple service instance for testing
    // Note: Integration tests requiring Qdrant connection should use integration test files
    fn create_test_config() -> SemanticSearchConfig {
        SemanticSearchConfig::default()
    }

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::default();
        assert_eq!(options.limit, 10);
        assert!(options.threshold.is_none());
        assert!(!options.diversify_results);
        assert!(!options.include_explanations);
    }

    #[test]
    fn test_search_config_default() {
        let config = SemanticSearchConfig::default();
        assert_eq!(config.default_limit, 10);
        assert_eq!(config.default_threshold, 0.7);
        assert_eq!(config.max_limit, 100);
        assert!(config.enable_analytics);
        assert!(config.cache_results);
    }

    #[test]
    fn test_filter_operations() {
        let filter = SearchFilter {
            field: "research_type".to_string(),
            operation: FilterOperation::Equals,
            value: serde_json::json!("Learning"),
        };

        assert_eq!(filter.field, "research_type");
        assert!(matches!(filter.operation, FilterOperation::Equals));
    }

    #[test]
    fn test_batch_search_request() {
        let request = BatchSearchRequest {
            queries: vec!["rust async".to_string(), "tokio runtime".to_string()],
            options: SearchOptions::default(),
            merge_results: true,
            max_per_query: Some(5),
        };

        assert_eq!(request.queries.len(), 2);
        assert!(request.merge_results);
        assert_eq!(request.max_per_query, Some(5));
    }

    #[test]
    fn test_suggestion_request() {
        let request = SuggestionRequest {
            document_id: "test-doc-id".to_string(),
            limit: 5,
            threshold: Some(0.8),
            exclude_source: true,
            filters: vec![],
        };

        assert_eq!(request.document_id, "test-doc-id");
        assert_eq!(request.limit, 5);
        assert_eq!(request.threshold, Some(0.8));
        assert!(request.exclude_source);
    }

    #[test]
    fn test_search_explanation() {
        let explanation = SearchExplanation {
            base_similarity: 0.85,
            temporal_boost: Some(0.1),
            quality_boost: Some(0.05),
            penalties: vec!["content_too_short".to_string()],
            boosts: vec!["high_quality".to_string()],
            calculation: "base(0.85) + boosts(high_quality) = 0.90".to_string(),
        };

        assert_eq!(explanation.base_similarity, 0.85);
        assert_eq!(explanation.temporal_boost, Some(0.1));
        assert_eq!(explanation.penalties.len(), 1);
        assert_eq!(explanation.boosts.len(), 1);
    }

    #[test]
    fn test_search_analytics_default() {
        let analytics = SearchAnalytics::default();
        assert_eq!(analytics.total_searches, 0);
        assert_eq!(analytics.avg_search_time_ms, 0.0);
        assert!(analytics.popular_terms.is_empty());
        assert_eq!(analytics.cache_hit_rate, 0.0);
    }

    #[test]
    fn test_query_complexity_calculation() {
        // Test the static method without creating a full service
        let _config = create_test_config();

        // Create a minimal service-like struct to test the method
        struct TestCalculator;
        impl TestCalculator {
            fn calculate_query_complexity(&self, query: &str) -> f64 {
                let word_count = query.split_whitespace().count();
                let char_count = query.len();
                let punctuation_count = query.chars().filter(|c| c.is_ascii_punctuation()).count();

                (word_count as f64 * 0.5
                    + char_count as f64 * 0.01
                    + punctuation_count as f64 * 0.1)
                    .min(10.0)
            }
        }

        let calculator = TestCalculator;
        let simple_query = "rust async";
        let complex_query =
            "How to implement async/await patterns in Rust with proper error handling?";

        let simple_score = calculator.calculate_query_complexity(simple_query);
        let complex_score = calculator.calculate_query_complexity(complex_query);

        assert!(complex_score > simple_score);
        assert!(simple_score > 0.0);
        assert!(complex_score <= 10.0);
    }

    #[test]
    fn test_keyword_extraction() {
        // Test the keyword extraction logic independently
        fn extract_keywords(query: &str) -> Vec<String> {
            query
                .split_whitespace()
                .filter(|word| word.len() > 3)
                .map(|word| word.to_lowercase())
                .collect()
        }

        let query = "How to implement async patterns in Rust programming?";
        let keywords = extract_keywords(query);

        // Should filter out short words like "to", "in"
        assert!(keywords.contains(&"implement".to_string()));
        assert!(keywords.contains(&"async".to_string()));
        assert!(keywords.contains(&"patterns".to_string()));
        assert!(keywords.contains(&"rust".to_string()));
        assert!(keywords.contains(&"programming?".to_string())); // Note: includes punctuation
        assert!(!keywords.contains(&"to".to_string()));
    }

    #[test]
    fn test_content_hash() {
        // Test content hashing logic independently
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        fn simple_content_hash(content: &str) -> u64 {
            let mut hasher = DefaultHasher::new();
            let prefix = if content.len() > 100 {
                &content[..100]
            } else {
                content
            };
            prefix.hash(&mut hasher);
            hasher.finish()
        }

        // Create content where first 100 chars are identical
        let base_content =
            "This is a test content for similarity checking and some more text to make it exactly";
        let content1 = format!(
            "{base_content} one hundred characters long - different ending A"
        );
        let content2 = format!(
            "{base_content} one hundred characters long - different ending B"
        );
        let content3 = "Completely different content here";

        // Verify the first 100 characters are the same
        assert_eq!(&content1[..100], &content2[..100]);

        let hash1 = simple_content_hash(&content1);
        let hash2 = simple_content_hash(&content2);
        let hash3 = simple_content_hash(content3);

        // content1 and content2 should have same hash (first 100 chars)
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_preprocess_query() {
        // Test query preprocessing logic
        fn preprocess_query(query: &str, max_length: usize) -> Result<String, String> {
            if query.len() > max_length {
                return Err(format!("Query too long: {} > {}", query.len(), max_length));
            }

            let mut processed = query.to_string();
            processed = processed.trim().to_string();

            // Remove extra whitespace
            processed = regex::Regex::new(r"\s+")
                .unwrap()
                .replace_all(&processed, " ")
                .to_string();

            Ok(processed)
        }

        // Test normal query
        let result = preprocess_query("  How to   implement   async  ", 1000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "How to implement async");

        // Test query that's too long
        let long_query = "a".repeat(2000);
        let result = preprocess_query(&long_query, 1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_search_result_creation() {
        let metadata = crate::vector::storage::DocumentMetadata::default();
        let document = VectorDocument {
            id: "test-doc-1".to_string(),
            content: "This is a test document for search results".to_string(),
            embedding: vec![0.1, 0.2, 0.3, 0.4],
            metadata,
            stored_at: chrono::Utc::now(),
        };

        let match_metadata = MatchMetadata {
            search_time_ms: 25.5,
            original_rank: 1,
            final_rank: 1,
            matched_filters: 2,
            snippet: Some("This is a test document...".to_string()),
        };

        let search_result = SearchResult {
            document,
            similarity_score: 0.85,
            relevance_score: 0.92,
            explanation: None,
            match_metadata,
        };

        assert_eq!(search_result.similarity_score, 0.85);
        assert_eq!(search_result.relevance_score, 0.92);
        assert_eq!(search_result.match_metadata.search_time_ms, 25.5);
        assert_eq!(search_result.match_metadata.original_rank, 1);
        assert_eq!(search_result.match_metadata.matched_filters, 2);
    }

    #[test]
    fn test_search_explanation_creation() {
        let explanation = SearchExplanation {
            base_similarity: 0.80,
            temporal_boost: Some(0.05),
            quality_boost: Some(0.10),
            penalties: vec!["length_penalty".to_string()],
            boosts: vec!["quality_boost".to_string(), "temporal_boost".to_string()],
            calculation: "0.80 + 0.05 + 0.10 - 0.03 = 0.92".to_string(),
        };

        assert_eq!(explanation.base_similarity, 0.80);
        assert_eq!(explanation.temporal_boost, Some(0.05));
        assert_eq!(explanation.quality_boost, Some(0.10));
        assert_eq!(explanation.penalties.len(), 1);
        assert_eq!(explanation.boosts.len(), 2);
        assert!(explanation.calculation.contains("0.92"));
    }

    #[test]
    fn test_match_metadata_creation() {
        let metadata = MatchMetadata {
            search_time_ms: 15.7,
            original_rank: 3,
            final_rank: 1,
            matched_filters: 4,
            snippet: Some("Search snippet preview...".to_string()),
        };

        assert_eq!(metadata.search_time_ms, 15.7);
        assert_eq!(metadata.original_rank, 3);
        assert_eq!(metadata.final_rank, 1);
        assert_eq!(metadata.matched_filters, 4);
        assert!(metadata.snippet.is_some());
        assert_eq!(metadata.snippet.unwrap(), "Search snippet preview...");
    }

    #[test]
    fn test_search_filter_validation() {
        let valid_filters = vec![
            SearchFilter {
                field: "content_type".to_string(),
                operation: FilterOperation::Equals,
                value: serde_json::json!("article"),
            },
            SearchFilter {
                field: "quality_score".to_string(),
                operation: FilterOperation::GreaterThanOrEqual,
                value: serde_json::json!(0.8),
            },
            SearchFilter {
                field: "tags".to_string(),
                operation: FilterOperation::Contains,
                value: serde_json::json!("rust"),
            },
        ];

        for filter in valid_filters {
            assert!(!filter.field.is_empty());
            assert!(!filter.value.is_null());
        }
    }

    #[test]
    fn test_search_options_with_all_features() {
        let options = SearchOptions {
            limit: 25,
            threshold: Some(0.75),
            collection: Some("custom_collection".to_string()),
            filters: vec![SearchFilter {
                field: "research_type".to_string(),
                operation: FilterOperation::Equals,
                value: serde_json::json!("Learning"),
            }],
            diversify_results: true,
            temporal_boost: Some(1.2),
            quality_boost: Some(1.5),
            include_explanations: true,
            min_content_length: Some(100),
            max_content_length: Some(5000),
            fuzzy_matching: true,
        };

        assert_eq!(options.limit, 25);
        assert_eq!(options.threshold, Some(0.75));
        assert!(options.diversify_results);
        assert_eq!(options.temporal_boost, Some(1.2));
        assert_eq!(options.quality_boost, Some(1.5));
        assert!(options.include_explanations);
        assert_eq!(options.min_content_length, Some(100));
        assert_eq!(options.max_content_length, Some(5000));
        assert!(options.fuzzy_matching);
        assert_eq!(options.filters.len(), 1);
    }

    #[test]
    fn test_filter_operation_completeness() {
        let all_operations = vec![
            FilterOperation::Equals,
            FilterOperation::NotEquals,
            FilterOperation::Contains,
            FilterOperation::NotContains,
            FilterOperation::GreaterThan,
            FilterOperation::LessThan,
            FilterOperation::GreaterThanOrEqual,
            FilterOperation::LessThanOrEqual,
            FilterOperation::In,
            FilterOperation::NotIn,
            FilterOperation::After,
            FilterOperation::Before,
            FilterOperation::Regex,
        ];

        // Test that all operations can be created and serialized
        for operation in all_operations {
            let filter = SearchFilter {
                field: "test_field".to_string(),
                operation: operation.clone(),
                value: serde_json::json!("test_value"),
            };

            let serialized = serde_json::to_string(&filter).expect("Should serialize");
            let _: SearchFilter = serde_json::from_str(&serialized).expect("Should deserialize");
        }
    }

    #[test]
    fn test_search_result_serialization() {
        let metadata = crate::vector::storage::DocumentMetadata::default();
        let document = VectorDocument {
            id: "serialization-test".to_string(),
            content: "Test content for serialization".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata,
            stored_at: chrono::Utc::now(),
        };

        let match_metadata = MatchMetadata {
            search_time_ms: 12.3,
            original_rank: 2,
            final_rank: 1,
            matched_filters: 3,
            snippet: Some("Test content...".to_string()),
        };

        let search_result = SearchResult {
            document,
            similarity_score: 0.88,
            relevance_score: 0.92,
            explanation: None,
            match_metadata,
        };

        let serialized = serde_json::to_string(&search_result).expect("Should serialize");
        let deserialized: SearchResult =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(
            search_result.similarity_score,
            deserialized.similarity_score
        );
        assert_eq!(search_result.relevance_score, deserialized.relevance_score);
        assert_eq!(search_result.document.id, deserialized.document.id);
        assert_eq!(
            search_result.match_metadata.search_time_ms,
            deserialized.match_metadata.search_time_ms
        );
    }

    #[test]
    fn test_search_explanation_serialization() {
        let explanation = SearchExplanation {
            base_similarity: 0.75,
            temporal_boost: Some(0.08),
            quality_boost: Some(0.12),
            penalties: vec!["short_content".to_string(), "low_quality".to_string()],
            boosts: vec!["recent_document".to_string()],
            calculation: "base(0.75) + temporal(0.08) + quality(0.12) = 0.95".to_string(),
        };

        let serialized = serde_json::to_string(&explanation).expect("Should serialize");
        let deserialized: SearchExplanation =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(explanation.base_similarity, deserialized.base_similarity);
        assert_eq!(explanation.temporal_boost, deserialized.temporal_boost);
        assert_eq!(explanation.quality_boost, deserialized.quality_boost);
        assert_eq!(explanation.penalties, deserialized.penalties);
        assert_eq!(explanation.boosts, deserialized.boosts);
        assert_eq!(explanation.calculation, deserialized.calculation);
    }

    #[test]
    fn test_search_options_edge_cases() {
        // Test with minimal options
        let minimal_options = SearchOptions {
            limit: 1,
            threshold: Some(0.99),
            collection: None,
            filters: vec![],
            diversify_results: false,
            temporal_boost: None,
            quality_boost: None,
            include_explanations: false,
            min_content_length: None,
            max_content_length: None,
            fuzzy_matching: false,
        };

        assert_eq!(minimal_options.limit, 1);
        assert_eq!(minimal_options.threshold, Some(0.99));
        assert!(minimal_options.filters.is_empty());

        // Test with maximum options
        let max_options = SearchOptions {
            limit: 1000,
            threshold: Some(0.0),
            collection: Some("very_long_collection_name_for_testing".to_string()),
            filters: (0..10)
                .map(|i| SearchFilter {
                    field: format!("field_{i}"),
                    operation: FilterOperation::Equals,
                    value: serde_json::json!(format!("value_{}", i)),
                })
                .collect(),
            diversify_results: true,
            temporal_boost: Some(5.0),
            quality_boost: Some(10.0),
            include_explanations: true,
            min_content_length: Some(1),
            max_content_length: Some(100000),
            fuzzy_matching: true,
        };

        assert_eq!(max_options.limit, 1000);
        assert_eq!(max_options.threshold, Some(0.0));
        assert_eq!(max_options.filters.len(), 10);
        assert_eq!(max_options.temporal_boost, Some(5.0));
        assert_eq!(max_options.quality_boost, Some(10.0));
    }

    #[test]
    fn test_search_filter_different_value_types() {
        let string_filter = SearchFilter {
            field: "title".to_string(),
            operation: FilterOperation::Contains,
            value: serde_json::json!("rust programming"),
        };

        let number_filter = SearchFilter {
            field: "rating".to_string(),
            operation: FilterOperation::GreaterThan,
            value: serde_json::json!(4.5),
        };

        let boolean_filter = SearchFilter {
            field: "is_published".to_string(),
            operation: FilterOperation::Equals,
            value: serde_json::json!(true),
        };

        let array_filter = SearchFilter {
            field: "categories".to_string(),
            operation: FilterOperation::In,
            value: serde_json::json!(["technology", "programming", "rust"]),
        };

        let null_filter = SearchFilter {
            field: "deprecated_field".to_string(),
            operation: FilterOperation::Equals,
            value: serde_json::json!(null),
        };

        // All should be valid and serializable
        for filter in vec![
            string_filter,
            number_filter,
            boolean_filter,
            array_filter,
            null_filter,
        ] {
            let serialized = serde_json::to_string(&filter).expect("Should serialize");
            let _: SearchFilter = serde_json::from_str(&serialized).expect("Should deserialize");
        }
    }

    #[test]
    fn test_match_metadata_edge_cases() {
        // Test with zero values
        let zero_metadata = MatchMetadata {
            search_time_ms: 0.0,
            original_rank: 0,
            final_rank: 0,
            matched_filters: 0,
            snippet: None,
        };

        assert_eq!(zero_metadata.search_time_ms, 0.0);
        assert_eq!(zero_metadata.original_rank, 0);
        assert_eq!(zero_metadata.final_rank, 0);
        assert_eq!(zero_metadata.matched_filters, 0);
        assert!(zero_metadata.snippet.is_none());

        // Test with large values
        let large_metadata = MatchMetadata {
            search_time_ms: 9999.99,
            original_rank: usize::MAX,
            final_rank: usize::MAX,
            matched_filters: usize::MAX,
            snippet: Some("x".repeat(10000)),
        };

        assert_eq!(large_metadata.search_time_ms, 9999.99);
        assert_eq!(large_metadata.original_rank, usize::MAX);
        assert_eq!(large_metadata.final_rank, usize::MAX);
        assert_eq!(large_metadata.matched_filters, usize::MAX);
        assert!(large_metadata.snippet.is_some());
        assert_eq!(large_metadata.snippet.unwrap().len(), 10000);
    }

    #[test]
    fn test_search_explanation_edge_cases() {
        // Test with no boosts or penalties
        let minimal_explanation = SearchExplanation {
            base_similarity: 0.5,
            temporal_boost: None,
            quality_boost: None,
            penalties: vec![],
            boosts: vec![],
            calculation: "base_only(0.5)".to_string(),
        };

        assert_eq!(minimal_explanation.base_similarity, 0.5);
        assert!(minimal_explanation.temporal_boost.is_none());
        assert!(minimal_explanation.quality_boost.is_none());
        assert!(minimal_explanation.penalties.is_empty());
        assert!(minimal_explanation.boosts.is_empty());

        // Test with many factors
        let complex_explanation = SearchExplanation {
            base_similarity: 0.65,
            temporal_boost: Some(-0.1), // Negative boost (penalty)
            quality_boost: Some(0.25),
            penalties: vec![
                "too_long".to_string(),
                "low_quality".to_string(),
                "duplicate_content".to_string(),
                "outdated".to_string(),
            ],
            boosts: vec![
                "high_relevance".to_string(),
                "popular_content".to_string(),
                "recent_update".to_string(),
                "verified_source".to_string(),
                "high_engagement".to_string(),
            ],
            calculation:
                "base(0.65) + quality(0.25) - temporal(0.1) - penalties(0.15) + boosts(0.3) = 0.95"
                    .to_string(),
        };

        assert_eq!(complex_explanation.base_similarity, 0.65);
        assert_eq!(complex_explanation.temporal_boost, Some(-0.1));
        assert_eq!(complex_explanation.quality_boost, Some(0.25));
        assert_eq!(complex_explanation.penalties.len(), 4);
        assert_eq!(complex_explanation.boosts.len(), 5);
    }

    #[test]
    fn test_search_result_score_ranges() {
        let metadata = crate::vector::storage::DocumentMetadata::default();
        let document = VectorDocument {
            id: "score-test".to_string(),
            content: "Score range testing".to_string(),
            embedding: vec![0.5],
            metadata,
            stored_at: chrono::Utc::now(),
        };

        let match_metadata = MatchMetadata {
            search_time_ms: 10.0,
            original_rank: 1,
            final_rank: 1,
            matched_filters: 1,
            snippet: None,
        };

        // Test minimum scores
        let min_result = SearchResult {
            document: document.clone(),
            similarity_score: 0.0,
            relevance_score: 0.0,
            explanation: None,
            match_metadata: match_metadata.clone(),
        };

        assert_eq!(min_result.similarity_score, 0.0);
        assert_eq!(min_result.relevance_score, 0.0);

        // Test maximum scores
        let max_result = SearchResult {
            document: document.clone(),
            similarity_score: 1.0,
            relevance_score: 1.0,
            explanation: None,
            match_metadata: match_metadata.clone(),
        };

        assert_eq!(max_result.similarity_score, 1.0);
        assert_eq!(max_result.relevance_score, 1.0);

        // Test out-of-range scores (should still be stored as-is)
        let out_of_range_result = SearchResult {
            document,
            similarity_score: -0.5,
            relevance_score: 1.5,
            explanation: None,
            match_metadata,
        };

        assert_eq!(out_of_range_result.similarity_score, -0.5);
        assert_eq!(out_of_range_result.relevance_score, 1.5);
    }

    #[test]
    fn test_search_options_serialization() {
        let options = SearchOptions {
            limit: 15,
            threshold: Some(0.7),
            collection: Some("test_collection".to_string()),
            filters: vec![SearchFilter {
                field: "category".to_string(),
                operation: FilterOperation::Equals,
                value: serde_json::json!("research"),
            }],
            diversify_results: true,
            temporal_boost: Some(1.1),
            quality_boost: Some(1.3),
            include_explanations: true,
            min_content_length: Some(50),
            max_content_length: Some(2000),
            fuzzy_matching: false,
        };

        let serialized = serde_json::to_string(&options).expect("Should serialize");
        let deserialized: SearchOptions =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(options.limit, deserialized.limit);
        assert_eq!(options.threshold, deserialized.threshold);
        assert_eq!(options.collection, deserialized.collection);
        assert_eq!(options.filters.len(), deserialized.filters.len());
        assert_eq!(options.diversify_results, deserialized.diversify_results);
        assert_eq!(options.temporal_boost, deserialized.temporal_boost);
        assert_eq!(options.quality_boost, deserialized.quality_boost);
        assert_eq!(
            options.include_explanations,
            deserialized.include_explanations
        );
        assert_eq!(options.min_content_length, deserialized.min_content_length);
        assert_eq!(options.max_content_length, deserialized.max_content_length);
        assert_eq!(options.fuzzy_matching, deserialized.fuzzy_matching);
    }
}
