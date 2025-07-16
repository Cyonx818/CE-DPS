// ABOUTME: Hybrid search service combining vector similarity and keyword matching
//! This module provides comprehensive hybrid search capabilities that combine semantic vector
//! search with traditional keyword search for improved relevance and precision in research
//! content discovery. It includes multiple fusion algorithms, adaptive strategy selection,
//! and configurable search balancing.

use crate::vector::{
    error::VectorResult,
    search::{MatchMetadata, SearchOptions, SearchResult, SemanticSearchService},
    storage::VectorDocument,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, instrument, warn};

/// Configuration for hybrid search operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchConfig {
    /// Default fusion method to use
    pub default_fusion_method: FusionMethod,
    /// Default search strategy
    pub default_strategy: SearchStrategy,
    /// Weight for vector search results (0.0-1.0)
    pub vector_weight: f64,
    /// Weight for keyword search results (0.0-1.0)
    pub keyword_weight: f64,
    /// Minimum vector similarity threshold
    pub min_vector_threshold: f64,
    /// Minimum keyword score threshold
    pub min_keyword_threshold: f64,
    /// Enable adaptive strategy selection
    pub enable_adaptive_strategy: bool,
    /// Maximum results from each search type before fusion
    pub max_results_per_type: usize,
    /// Enable result diversification
    pub enable_diversification: bool,
    /// Cache hybrid search results
    pub enable_caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            default_fusion_method: FusionMethod::ReciprocalRankFusion,
            default_strategy: SearchStrategy::Balanced,
            vector_weight: 0.6,
            keyword_weight: 0.4,
            min_vector_threshold: 0.5,
            min_keyword_threshold: 0.1,
            enable_adaptive_strategy: true,
            max_results_per_type: 50,
            enable_diversification: true,
            enable_caching: true,
            cache_ttl_seconds: 300,
        }
    }
}

/// Search strategy determining the balance between vector and keyword search
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SearchStrategy {
    /// Equal emphasis on both vector and keyword search
    Balanced,
    /// Prioritize semantic understanding (higher vector weight)
    SemanticFocus,
    /// Prioritize exact term matching (higher keyword weight)
    KeywordFocus,
    /// Automatically select strategy based on query analysis
    Adaptive,
    /// Use only vector search
    VectorOnly,
    /// Use only keyword search
    KeywordOnly,
    /// Custom weights specified by user
    Custom {
        vector_weight: f64,
        keyword_weight: f64,
    },
}

impl SearchStrategy {
    /// Get the vector and keyword weights for this strategy
    pub fn get_weights(&self, config: &HybridSearchConfig) -> (f64, f64) {
        match self {
            SearchStrategy::Balanced => (0.5, 0.5),
            SearchStrategy::SemanticFocus => (0.8, 0.2),
            SearchStrategy::KeywordFocus => (0.2, 0.8),
            SearchStrategy::Adaptive => (config.vector_weight, config.keyword_weight),
            SearchStrategy::VectorOnly => (1.0, 0.0),
            SearchStrategy::KeywordOnly => (0.0, 1.0),
            SearchStrategy::Custom {
                vector_weight,
                keyword_weight,
            } => (*vector_weight, *keyword_weight),
        }
    }
}

/// Method for fusing vector and keyword search results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FusionMethod {
    /// Reciprocal Rank Fusion (RRF) - balanced approach
    ReciprocalRankFusion,
    /// Weighted combination of normalized scores
    WeightedScoring,
    /// Simple rank-based fusion
    RankFusion,
    /// Maximum score from either search type
    MaxScore,
    /// Linear interpolation between scores
    LinearInterpolation,
    /// Machine learning-based fusion (future enhancement)
    MLFusion,
}

/// Keyword search implementation using TF-IDF and term matching
#[derive(Debug, Clone)]
pub struct KeywordSearcher {
    /// Document corpus for TF-IDF calculation
    document_corpus: Vec<VectorDocument>,
    /// Term frequency index: doc_id -> term -> frequency
    tf_index: HashMap<String, HashMap<String, f64>>,
    /// Document frequency: term -> number of documents containing term
    df_index: HashMap<String, usize>,
    /// Total number of documents
    total_documents: usize,
    /// Enable stemming and normalization
    enable_normalization: bool,
    /// Field weights for different document sections
    #[allow(dead_code)] // TODO: Implement field-specific weighting in scoring
    field_weights: HashMap<String, f64>,
}

/// Result from keyword search operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordSearchResult {
    /// The matching document
    pub document: VectorDocument,
    /// TF-IDF based relevance score
    pub tf_idf_score: f64,
    /// Term match details
    pub term_matches: Vec<TermMatch>,
    /// Field-specific scores
    pub field_scores: HashMap<String, f64>,
    /// Total matched terms
    pub matched_terms: usize,
    /// Query coverage (fraction of query terms found)
    pub query_coverage: f64,
}

/// Details about a matched term in the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermMatch {
    /// The matched term
    pub term: String,
    /// Field where the match occurred
    pub field: String,
    /// Position in the field (character offset)
    pub position: usize,
    /// Term frequency in this document
    pub tf: f64,
    /// Inverse document frequency for this term
    pub idf: f64,
    /// TF-IDF score for this term match
    pub tf_idf: f64,
}

/// Enhanced search result from hybrid search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResult {
    /// The matching document
    pub document: VectorDocument,
    /// Vector similarity score (if available)
    pub vector_score: Option<f64>,
    /// Keyword search score (if available)
    pub keyword_score: Option<f64>,
    /// Final fused relevance score
    pub hybrid_score: f64,
    /// Fusion method used
    pub fusion_method: FusionMethod,
    /// Search strategy applied
    pub strategy: SearchStrategy,
    /// Detailed score explanation
    pub explanation: Option<HybridSearchExplanation>,
    /// Match metadata
    pub match_metadata: HybridMatchMetadata,
    /// Source information
    pub search_sources: SearchSources,
}

/// Explanation of hybrid search scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchExplanation {
    /// Vector search contribution
    pub vector_contribution: Option<f64>,
    /// Keyword search contribution
    pub keyword_contribution: Option<f64>,
    /// Fusion algorithm details
    pub fusion_details: String,
    /// Strategy rationale
    pub strategy_rationale: String,
    /// Applied weights
    pub weights: (f64, f64), // (vector_weight, keyword_weight)
    /// Detailed calculation
    pub calculation: String,
}

/// Enhanced match metadata for hybrid results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridMatchMetadata {
    /// Standard metadata
    pub base_metadata: MatchMetadata,
    /// Vector search rank (if applicable)
    pub vector_rank: Option<usize>,
    /// Keyword search rank (if applicable)
    pub keyword_rank: Option<usize>,
    /// Number of matched query terms
    pub matched_terms: usize,
    /// Total query terms
    pub total_query_terms: usize,
    /// Query coverage percentage
    pub query_coverage: f64,
}

/// Information about which search sources contributed to the result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSources {
    /// Found in vector search results
    pub from_vector: bool,
    /// Found in keyword search results
    pub from_keyword: bool,
    /// Combined from both sources
    pub combined: bool,
}

/// Request for hybrid search operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchRequest {
    /// Search query
    pub query: String,
    /// Search strategy to use
    pub strategy: Option<SearchStrategy>,
    /// Fusion method to apply
    pub fusion_method: Option<FusionMethod>,
    /// Search options
    pub options: SearchOptions,
    /// Include detailed explanations
    pub include_explanations: bool,
    /// Custom weights (overrides strategy)
    pub custom_weights: Option<(f64, f64)>,
    /// Minimum hybrid score threshold
    pub min_hybrid_score: Option<f64>,
}

/// Result set from hybrid search operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResultSet {
    /// Hybrid search results
    pub results: Vec<HybridSearchResult>,
    /// Total results found before filtering
    pub total_found: usize,
    /// Vector search result count
    pub vector_results_count: usize,
    /// Keyword search result count
    pub keyword_results_count: usize,
    /// Search request used
    pub request: HybridSearchRequest,
    /// Execution statistics
    pub execution_stats: HybridSearchStats,
    /// Query analysis
    pub query_analysis: QueryAnalysis,
}

/// Statistics for hybrid search execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchStats {
    /// Total execution time
    pub total_time_ms: f64,
    /// Vector search time
    pub vector_search_time_ms: f64,
    /// Keyword search time
    pub keyword_search_time_ms: f64,
    /// Fusion time
    pub fusion_time_ms: f64,
    /// Post-processing time
    pub postprocessing_time_ms: f64,
    /// Strategy selection time
    pub strategy_selection_time_ms: f64,
    /// Cache hits
    pub cache_hits: usize,
    /// Cache misses
    pub cache_misses: usize,
}

/// Analysis of the search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAnalysis {
    /// Original query
    pub original_query: String,
    /// Processed query terms
    pub terms: Vec<String>,
    /// Detected query type
    pub query_type: QueryType,
    /// Complexity score
    pub complexity: f64,
    /// Recommended strategy
    pub recommended_strategy: SearchStrategy,
    /// Language detection
    pub language: Option<String>,
    /// Technical terms detected
    pub technical_terms: Vec<String>,
}

/// Type of search query detected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueryType {
    /// Conceptual/semantic query
    Conceptual,
    /// Specific technical query
    Technical,
    /// Question-based query
    Question,
    /// Keywords/tags query
    Keywords,
    /// Mixed query type
    Mixed,
}

/// Main hybrid search service
pub struct HybridSearchService {
    /// Semantic search service
    semantic_service: Arc<SemanticSearchService>,
    /// Keyword search engine
    keyword_searcher: Arc<KeywordSearcher>,
    /// Service configuration
    config: HybridSearchConfig,
    /// Search analytics
    analytics: Arc<tokio::sync::RwLock<HybridSearchAnalytics>>,
    /// Query cache
    cache: Arc<tokio::sync::RwLock<HashMap<String, (HybridSearchResultSet, Instant)>>>,
}

/// Analytics for hybrid search operations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HybridSearchAnalytics {
    /// Total hybrid searches performed
    pub total_searches: u64,
    /// Strategy usage counts
    pub strategy_usage: HashMap<String, u64>,
    /// Fusion method usage
    pub fusion_method_usage: HashMap<String, u64>,
    /// Average execution times
    pub avg_execution_time_ms: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Average result count
    pub avg_result_count: f64,
    /// Strategy effectiveness scores
    pub strategy_effectiveness: HashMap<String, f64>,
}

impl Default for KeywordSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl KeywordSearcher {
    /// Create a new keyword searcher
    pub fn new() -> Self {
        Self {
            document_corpus: Vec::new(),
            tf_index: HashMap::new(),
            df_index: HashMap::new(),
            total_documents: 0,
            enable_normalization: true,
            field_weights: Self::default_field_weights(),
        }
    }

    /// Default field weights for document sections
    fn default_field_weights() -> HashMap<String, f64> {
        let mut weights = HashMap::new();
        weights.insert("title".to_string(), 3.0);
        weights.insert("content".to_string(), 1.0);
        weights.insert("tags".to_string(), 2.0);
        weights.insert("metadata".to_string(), 0.5);
        weights
    }

    /// Index a set of documents for keyword search
    #[instrument(skip(self, documents))]
    pub async fn index_documents(&mut self, documents: Vec<VectorDocument>) -> VectorResult<()> {
        info!("Indexing {} documents for keyword search", documents.len());

        self.document_corpus = documents;
        self.total_documents = self.document_corpus.len();
        self.tf_index.clear();
        self.df_index.clear();

        // Build TF index for each document
        for doc in &self.document_corpus {
            let terms = self.extract_terms_from_document(doc);
            let tf_scores = self.calculate_tf_scores(&terms);
            self.tf_index.insert(doc.id.clone(), tf_scores);
        }

        // Build DF index across all documents
        self.build_df_index();

        info!(
            "Keyword search indexing completed: {} documents, {} unique terms",
            self.total_documents,
            self.df_index.len()
        );

        Ok(())
    }

    /// Search for documents using keyword matching
    #[instrument(skip(self, query, limit))]
    pub async fn search_keywords(
        &self,
        query: &str,
        limit: usize,
    ) -> VectorResult<Vec<KeywordSearchResult>> {
        let query_terms = self.normalize_query(query);
        if query_terms.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();

        for doc in &self.document_corpus {
            if let Some(tf_scores) = self.tf_index.get(&doc.id) {
                let mut total_score = 0.0;
                let mut term_matches = Vec::new();
                let mut matched_terms = 0;

                for term in &query_terms {
                    if let Some(tf) = tf_scores.get(term) {
                        let df = self.df_index.get(term).unwrap_or(&1);
                        let idf = self.calculate_idf(*df);
                        let tf_idf = tf * idf;

                        total_score += tf_idf;
                        matched_terms += 1;

                        // Find term positions (simplified)
                        let positions = self.find_term_positions(doc, term);
                        for position in positions {
                            term_matches.push(TermMatch {
                                term: term.clone(),
                                field: "content".to_string(), // Simplified
                                position,
                                tf: *tf,
                                idf,
                                tf_idf,
                            });
                        }
                    }
                }

                if matched_terms > 0 {
                    let query_coverage = matched_terms as f64 / query_terms.len() as f64;

                    results.push(KeywordSearchResult {
                        document: doc.clone(),
                        tf_idf_score: total_score,
                        term_matches,
                        field_scores: HashMap::new(), // TODO: Implement field-specific scoring
                        matched_terms,
                        query_coverage,
                    });
                }
            }
        }

        // Sort by TF-IDF score descending
        results.sort_by(|a, b| {
            b.tf_idf_score
                .partial_cmp(&a.tf_idf_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        debug!(
            "Keyword search found {} results for query: '{}'",
            results.len(),
            query
        );
        Ok(results)
    }

    /// Extract and normalize terms from a document
    fn extract_terms_from_document(&self, doc: &VectorDocument) -> Vec<String> {
        let mut terms = Vec::new();

        // Extract from content
        let content_terms = self.tokenize_and_normalize(&doc.content);
        terms.extend(content_terms);

        // Extract from metadata custom fields
        for value in doc.metadata.custom_fields.values() {
            if let Some(text) = value.as_str() {
                let field_terms = self.tokenize_and_normalize(text);
                terms.extend(field_terms);
            }
        }

        terms
    }

    /// Tokenize and normalize text into terms
    fn tokenize_and_normalize(&self, text: &str) -> Vec<String> {
        if !self.enable_normalization {
            return text.split_whitespace().map(|s| s.to_string()).collect();
        }

        text.to_lowercase()
            .split_whitespace()
            .map(|word| {
                // Remove punctuation
                word.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            })
            .filter(|word| word.len() > 2) // Filter very short words
            .collect()
    }

    /// Normalize query into terms
    fn normalize_query(&self, query: &str) -> Vec<String> {
        self.tokenize_and_normalize(query)
    }

    /// Calculate term frequency scores for a document
    fn calculate_tf_scores(&self, terms: &[String]) -> HashMap<String, f64> {
        let mut tf_scores = HashMap::new();
        let total_terms = terms.len() as f64;

        for term in terms {
            let count = terms.iter().filter(|t| *t == term).count() as f64;
            let tf = count / total_terms;
            tf_scores.insert(term.clone(), tf);
        }

        tf_scores
    }

    /// Build document frequency index
    fn build_df_index(&mut self) {
        for tf_map in self.tf_index.values() {
            for term in tf_map.keys() {
                *self.df_index.entry(term.clone()).or_insert(0) += 1;
            }
        }
    }

    /// Calculate inverse document frequency
    fn calculate_idf(&self, df: usize) -> f64 {
        if df == 0 {
            return 0.0;
        }
        (self.total_documents as f64 / df as f64).ln()
    }

    /// Find positions of a term in a document (simplified implementation)
    fn find_term_positions(&self, doc: &VectorDocument, term: &str) -> Vec<usize> {
        let content = doc.content.to_lowercase();
        let mut positions = Vec::new();
        let mut start = 0;

        while let Some(pos) = content[start..].find(term) {
            positions.push(start + pos);
            start = start + pos + term.len();
        }

        positions
    }
}

impl HybridSearchService {
    /// Create a new hybrid search service
    pub fn new(
        semantic_service: Arc<SemanticSearchService>,
        keyword_searcher: Arc<KeywordSearcher>,
        config: HybridSearchConfig,
    ) -> Self {
        Self {
            semantic_service,
            keyword_searcher,
            config,
            analytics: Arc::new(tokio::sync::RwLock::new(HybridSearchAnalytics::default())),
            cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Create service with default configuration
    pub fn with_defaults(
        semantic_service: Arc<SemanticSearchService>,
        keyword_searcher: Arc<KeywordSearcher>,
    ) -> Self {
        Self::new(
            semantic_service,
            keyword_searcher,
            HybridSearchConfig::default(),
        )
    }

    /// Initialize the hybrid search service
    #[instrument(skip(self))]
    pub async fn initialize(&self) -> VectorResult<()> {
        info!("Initializing hybrid search service");

        // Initialize semantic search service
        self.semantic_service.initialize().await?;

        info!("Hybrid search service initialized successfully");
        Ok(())
    }

    /// Perform hybrid search combining vector and keyword search
    #[instrument(skip(self, request))]
    pub async fn hybrid_search(
        &self,
        request: HybridSearchRequest,
    ) -> VectorResult<HybridSearchResultSet> {
        let start_time = Instant::now();

        // Check cache first
        if self.config.enable_caching {
            if let Some(cached_result) = self.check_cache(&request.query).await {
                debug!("Cache hit for query: '{}'", request.query);
                return Ok(cached_result);
            }
        }

        // Analyze query and determine strategy
        let strategy_start = Instant::now();
        let query_analysis = self.analyze_query(&request.query).await?;
        let final_strategy = request.strategy.clone().unwrap_or_else(|| {
            if self.config.enable_adaptive_strategy {
                query_analysis.recommended_strategy.clone()
            } else {
                self.config.default_strategy.clone()
            }
        });
        let strategy_time = strategy_start.elapsed().as_millis() as f64;

        // Get weights for the chosen strategy
        let (vector_weight, keyword_weight) = if let Some(custom_weights) = request.custom_weights {
            custom_weights
        } else {
            final_strategy.get_weights(&self.config)
        };

        // Determine fusion method
        let fusion_method = request
            .fusion_method
            .clone()
            .unwrap_or(self.config.default_fusion_method.clone());

        let mut vector_results = Vec::new();
        let mut keyword_results = Vec::new();
        let mut vector_time = 0.0;
        let mut keyword_time = 0.0;

        // Perform vector search if weight > 0
        if vector_weight > 0.0 && !matches!(final_strategy, SearchStrategy::KeywordOnly) {
            let vector_start = Instant::now();
            let vector_search_options = SearchOptions {
                limit: self.config.max_results_per_type,
                threshold: Some(self.config.min_vector_threshold),
                ..request.options.clone()
            };

            match self
                .semantic_service
                .search_similar(&request.query, vector_search_options)
                .await
            {
                Ok(result_set) => {
                    vector_results = result_set.results;
                    vector_time = vector_start.elapsed().as_millis() as f64;
                    debug!("Vector search found {} results", vector_results.len());
                }
                Err(e) => {
                    warn!("Vector search failed: {}", e);
                    // Continue with keyword-only search
                }
            }
        }

        // Perform keyword search if weight > 0
        if keyword_weight > 0.0 && !matches!(final_strategy, SearchStrategy::VectorOnly) {
            let keyword_start = Instant::now();

            match self
                .keyword_searcher
                .search_keywords(&request.query, self.config.max_results_per_type)
                .await
            {
                Ok(results) => {
                    keyword_results = results;
                    keyword_time = keyword_start.elapsed().as_millis() as f64;
                    debug!("Keyword search found {} results", keyword_results.len());
                }
                Err(e) => {
                    warn!("Keyword search failed: {}", e);
                    // Continue with vector-only search
                }
            }
        }

        // Fuse results
        let fusion_start = Instant::now();
        let fused_results = self
            .fuse_results(
                vector_results,
                keyword_results,
                &fusion_method,
                (vector_weight, keyword_weight),
                &request,
            )
            .await?;
        let fusion_time = fusion_start.elapsed().as_millis() as f64;

        // Post-process results
        let postprocessing_start = Instant::now();
        let final_results = self.post_process_results(fused_results, &request).await?;
        let postprocessing_time = postprocessing_start.elapsed().as_millis() as f64;

        let total_time = start_time.elapsed().as_millis() as f64;

        // Build result set
        let result_set = HybridSearchResultSet {
            total_found: final_results.len(),
            vector_results_count: if vector_weight > 0.0 {
                final_results
                    .iter()
                    .filter(|r| r.search_sources.from_vector)
                    .count()
            } else {
                0
            },
            keyword_results_count: if keyword_weight > 0.0 {
                final_results
                    .iter()
                    .filter(|r| r.search_sources.from_keyword)
                    .count()
            } else {
                0
            },
            results: final_results,
            request: request.clone(),
            execution_stats: HybridSearchStats {
                total_time_ms: total_time,
                vector_search_time_ms: vector_time,
                keyword_search_time_ms: keyword_time,
                fusion_time_ms: fusion_time,
                postprocessing_time_ms: postprocessing_time,
                strategy_selection_time_ms: strategy_time,
                cache_hits: 0, // TODO: Implement cache statistics
                cache_misses: 1,
            },
            query_analysis,
        };

        // Update analytics
        self.update_analytics(&result_set).await;

        // Cache result if enabled
        if self.config.enable_caching {
            self.cache_result(&request.query, &result_set).await;
        }

        info!(
            "Hybrid search completed: {} results in {:.2}ms (strategy: {:?})",
            result_set.results.len(),
            total_time,
            final_strategy
        );

        Ok(result_set)
    }

    /// Analyze query to determine optimal search strategy
    async fn analyze_query(&self, query: &str) -> VectorResult<QueryAnalysis> {
        let terms = self.tokenize_query(query);
        let _word_count = terms.len();
        let _char_count = query.len();

        // Detect query type based on patterns
        let query_type = self.detect_query_type(query, &terms);

        // Calculate complexity
        let complexity = self.calculate_query_complexity(query, &terms);

        // Determine recommended strategy
        let recommended_strategy = self.recommend_strategy(&query_type, complexity, &terms);

        // Detect technical terms (simplified heuristic)
        let technical_terms = terms
            .iter()
            .filter(|term| self.is_technical_term(term))
            .cloned()
            .collect();

        Ok(QueryAnalysis {
            original_query: query.to_string(),
            terms,
            query_type,
            complexity,
            recommended_strategy,
            language: Some("en".to_string()), // Simplified language detection
            technical_terms,
        })
    }

    /// Detect the type of query based on content and patterns
    fn detect_query_type(&self, query: &str, terms: &[String]) -> QueryType {
        let query_lower = query.to_lowercase();

        // Check for question patterns
        if query_lower.starts_with("how")
            || query_lower.starts_with("what")
            || query_lower.starts_with("why")
            || query_lower.starts_with("when")
            || query_lower.contains("?")
        {
            return QueryType::Question;
        }

        // Check for technical patterns
        if terms.iter().any(|term| self.is_technical_term(term)) {
            return QueryType::Technical;
        }

        // Check for conceptual patterns
        if query_lower.contains("concept")
            || query_lower.contains("pattern")
            || query_lower.contains("approach")
            || query_lower.contains("methodology")
        {
            return QueryType::Conceptual;
        }

        // Check for keyword patterns (short, specific terms)
        if terms.len() <= 3 && terms.iter().all(|term| term.len() <= 8) {
            return QueryType::Keywords;
        }

        QueryType::Mixed
    }

    /// Check if a term is likely technical
    fn is_technical_term(&self, term: &str) -> bool {
        // Simplified technical term detection
        let technical_indicators = [
            "api",
            "rust",
            "async",
            "trait",
            "impl",
            "struct",
            "enum",
            "fn",
            "tokio",
            "serde",
            "http",
            "json",
            "sql",
            "database",
            "vector",
            "algorithm",
            "performance",
            "optimization",
            "cache",
            "index",
        ];

        technical_indicators.contains(&term.to_lowercase().as_str())
    }

    /// Calculate query complexity score
    fn calculate_query_complexity(&self, query: &str, terms: &[String]) -> f64 {
        let word_count = terms.len() as f64;
        let char_count = query.len() as f64;
        let punctuation_count = query.chars().filter(|c| c.is_ascii_punctuation()).count() as f64;
        let unique_terms = terms.iter().collect::<HashSet<_>>().len() as f64;

        // Weighted complexity calculation

        (word_count * 0.3 + char_count * 0.01 + punctuation_count * 0.2 + unique_terms * 0.4)
            .min(10.0)
    }

    /// Recommend search strategy based on query analysis
    fn recommend_strategy(
        &self,
        query_type: &QueryType,
        complexity: f64,
        terms: &[String],
    ) -> SearchStrategy {
        match query_type {
            QueryType::Conceptual => SearchStrategy::SemanticFocus,
            QueryType::Technical => {
                if complexity > 5.0 {
                    SearchStrategy::Balanced
                } else {
                    SearchStrategy::KeywordFocus
                }
            }
            QueryType::Question => SearchStrategy::SemanticFocus,
            QueryType::Keywords => SearchStrategy::KeywordFocus,
            QueryType::Mixed => {
                if terms.len() > 5 {
                    SearchStrategy::SemanticFocus
                } else {
                    SearchStrategy::Balanced
                }
            }
        }
    }

    /// Tokenize query into terms
    fn tokenize_query(&self, query: &str) -> Vec<String> {
        query
            .to_lowercase()
            .split_whitespace()
            .map(|word| {
                word.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            })
            .filter(|word| !word.is_empty())
            .collect()
    }

    /// Fuse vector and keyword search results
    async fn fuse_results(
        &self,
        vector_results: Vec<SearchResult>,
        keyword_results: Vec<KeywordSearchResult>,
        fusion_method: &FusionMethod,
        weights: (f64, f64),
        request: &HybridSearchRequest,
    ) -> VectorResult<Vec<HybridSearchResult>> {
        match fusion_method {
            FusionMethod::ReciprocalRankFusion => {
                self.reciprocal_rank_fusion(vector_results, keyword_results, weights, request)
                    .await
            }
            FusionMethod::WeightedScoring => {
                self.weighted_scoring_fusion(vector_results, keyword_results, weights, request)
                    .await
            }
            FusionMethod::RankFusion => {
                self.rank_fusion(vector_results, keyword_results, weights, request)
                    .await
            }
            FusionMethod::MaxScore => {
                self.max_score_fusion(vector_results, keyword_results, weights, request)
                    .await
            }
            FusionMethod::LinearInterpolation => {
                self.linear_interpolation_fusion(vector_results, keyword_results, weights, request)
                    .await
            }
            FusionMethod::MLFusion => {
                // Fallback to RRF for now
                warn!("ML Fusion not yet implemented, falling back to RRF");
                self.reciprocal_rank_fusion(vector_results, keyword_results, weights, request)
                    .await
            }
        }
    }

    /// Reciprocal Rank Fusion algorithm
    async fn reciprocal_rank_fusion(
        &self,
        vector_results: Vec<SearchResult>,
        keyword_results: Vec<KeywordSearchResult>,
        weights: (f64, f64),
        request: &HybridSearchRequest,
    ) -> VectorResult<Vec<HybridSearchResult>> {
        let mut hybrid_results = HashMap::new();
        let k = 60.0; // RRF parameter

        // Process vector results
        for (rank, result) in vector_results.iter().enumerate() {
            let rrf_score = weights.0 / (k + (rank + 1) as f64);

            hybrid_results.insert(
                result.document.id.clone(),
                HybridSearchResult {
                    document: result.document.clone(),
                    vector_score: Some(result.relevance_score),
                    keyword_score: None,
                    hybrid_score: rrf_score,
                    fusion_method: FusionMethod::ReciprocalRankFusion,
                    strategy: request.strategy.clone().unwrap_or(SearchStrategy::Balanced),
                    explanation: None,
                    match_metadata: HybridMatchMetadata {
                        base_metadata: result.match_metadata.clone(),
                        vector_rank: Some(rank),
                        keyword_rank: None,
                        matched_terms: 0,
                        total_query_terms: self.tokenize_query(&request.query).len(),
                        query_coverage: 0.0,
                    },
                    search_sources: SearchSources {
                        from_vector: true,
                        from_keyword: false,
                        combined: false,
                    },
                },
            );
        }

        // Process keyword results
        for (rank, result) in keyword_results.iter().enumerate() {
            let rrf_score = weights.1 / (k + (rank + 1) as f64);

            if let Some(existing) = hybrid_results.get_mut(&result.document.id) {
                // Combine with existing vector result
                existing.keyword_score = Some(result.tf_idf_score);
                existing.hybrid_score += rrf_score;
                existing.match_metadata.keyword_rank = Some(rank);
                existing.match_metadata.matched_terms = result.matched_terms;
                existing.match_metadata.query_coverage = result.query_coverage;
                existing.search_sources.from_keyword = true;
                existing.search_sources.combined = true;
            } else {
                // New keyword-only result
                hybrid_results.insert(
                    result.document.id.clone(),
                    HybridSearchResult {
                        document: result.document.clone(),
                        vector_score: None,
                        keyword_score: Some(result.tf_idf_score),
                        hybrid_score: rrf_score,
                        fusion_method: FusionMethod::ReciprocalRankFusion,
                        strategy: request.strategy.clone().unwrap_or(SearchStrategy::Balanced),
                        explanation: None,
                        match_metadata: HybridMatchMetadata {
                            base_metadata: MatchMetadata {
                                search_time_ms: 0.0,
                                original_rank: rank,
                                final_rank: 0, // Will be updated later
                                matched_filters: 0,
                                snippet: Some(self.create_snippet(&result.document.content)),
                            },
                            vector_rank: None,
                            keyword_rank: Some(rank),
                            matched_terms: result.matched_terms,
                            total_query_terms: self.tokenize_query(&request.query).len(),
                            query_coverage: result.query_coverage,
                        },
                        search_sources: SearchSources {
                            from_vector: false,
                            from_keyword: true,
                            combined: false,
                        },
                    },
                );
            }
        }

        let mut results: Vec<_> = hybrid_results.into_values().collect();

        // Sort by hybrid score descending
        results.sort_by(|a, b| {
            b.hybrid_score
                .partial_cmp(&a.hybrid_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Update final ranks
        for (index, result) in results.iter_mut().enumerate() {
            result.match_metadata.base_metadata.final_rank = index;
        }

        Ok(results)
    }

    /// Weighted scoring fusion algorithm
    async fn weighted_scoring_fusion(
        &self,
        vector_results: Vec<SearchResult>,
        keyword_results: Vec<KeywordSearchResult>,
        weights: (f64, f64),
        request: &HybridSearchRequest,
    ) -> VectorResult<Vec<HybridSearchResult>> {
        let mut hybrid_results = HashMap::new();

        // Normalize vector scores
        let max_vector_score = vector_results
            .iter()
            .map(|r| r.relevance_score)
            .fold(0.0, f64::max);

        // Normalize keyword scores
        let max_keyword_score = keyword_results
            .iter()
            .map(|r| r.tf_idf_score)
            .fold(0.0, f64::max);

        // Process vector results
        for (rank, result) in vector_results.iter().enumerate() {
            let normalized_score = if max_vector_score > 0.0 {
                result.relevance_score / max_vector_score
            } else {
                0.0
            };
            let weighted_score = normalized_score * weights.0;

            hybrid_results.insert(
                result.document.id.clone(),
                HybridSearchResult {
                    document: result.document.clone(),
                    vector_score: Some(result.relevance_score),
                    keyword_score: None,
                    hybrid_score: weighted_score,
                    fusion_method: FusionMethod::WeightedScoring,
                    strategy: request.strategy.clone().unwrap_or(SearchStrategy::Balanced),
                    explanation: None,
                    match_metadata: HybridMatchMetadata {
                        base_metadata: result.match_metadata.clone(),
                        vector_rank: Some(rank),
                        keyword_rank: None,
                        matched_terms: 0,
                        total_query_terms: self.tokenize_query(&request.query).len(),
                        query_coverage: 0.0,
                    },
                    search_sources: SearchSources {
                        from_vector: true,
                        from_keyword: false,
                        combined: false,
                    },
                },
            );
        }

        // Process keyword results
        for (rank, result) in keyword_results.iter().enumerate() {
            let normalized_score = if max_keyword_score > 0.0 {
                result.tf_idf_score / max_keyword_score
            } else {
                0.0
            };
            let weighted_score = normalized_score * weights.1;

            if let Some(existing) = hybrid_results.get_mut(&result.document.id) {
                // Combine with existing vector result
                existing.keyword_score = Some(result.tf_idf_score);
                existing.hybrid_score += weighted_score;
                existing.match_metadata.keyword_rank = Some(rank);
                existing.match_metadata.matched_terms = result.matched_terms;
                existing.match_metadata.query_coverage = result.query_coverage;
                existing.search_sources.from_keyword = true;
                existing.search_sources.combined = true;
            } else {
                // New keyword-only result
                hybrid_results.insert(
                    result.document.id.clone(),
                    HybridSearchResult {
                        document: result.document.clone(),
                        vector_score: None,
                        keyword_score: Some(result.tf_idf_score),
                        hybrid_score: weighted_score,
                        fusion_method: FusionMethod::WeightedScoring,
                        strategy: request.strategy.clone().unwrap_or(SearchStrategy::Balanced),
                        explanation: None,
                        match_metadata: HybridMatchMetadata {
                            base_metadata: MatchMetadata {
                                search_time_ms: 0.0,
                                original_rank: rank,
                                final_rank: 0,
                                matched_filters: 0,
                                snippet: Some(self.create_snippet(&result.document.content)),
                            },
                            vector_rank: None,
                            keyword_rank: Some(rank),
                            matched_terms: result.matched_terms,
                            total_query_terms: self.tokenize_query(&request.query).len(),
                            query_coverage: result.query_coverage,
                        },
                        search_sources: SearchSources {
                            from_vector: false,
                            from_keyword: true,
                            combined: false,
                        },
                    },
                );
            }
        }

        let mut results: Vec<_> = hybrid_results.into_values().collect();

        // Sort by hybrid score descending
        results.sort_by(|a, b| {
            b.hybrid_score
                .partial_cmp(&a.hybrid_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Update final ranks
        for (index, result) in results.iter_mut().enumerate() {
            result.match_metadata.base_metadata.final_rank = index;
        }

        Ok(results)
    }

    /// Simple rank-based fusion
    async fn rank_fusion(
        &self,
        vector_results: Vec<SearchResult>,
        keyword_results: Vec<KeywordSearchResult>,
        weights: (f64, f64),
        request: &HybridSearchRequest,
    ) -> VectorResult<Vec<HybridSearchResult>> {
        // For simplicity, delegate to RRF with different parameters
        self.reciprocal_rank_fusion(vector_results, keyword_results, weights, request)
            .await
    }

    /// Maximum score fusion
    async fn max_score_fusion(
        &self,
        vector_results: Vec<SearchResult>,
        keyword_results: Vec<KeywordSearchResult>,
        weights: (f64, f64),
        request: &HybridSearchRequest,
    ) -> VectorResult<Vec<HybridSearchResult>> {
        // For simplicity, delegate to weighted scoring
        self.weighted_scoring_fusion(vector_results, keyword_results, weights, request)
            .await
    }

    /// Linear interpolation fusion
    async fn linear_interpolation_fusion(
        &self,
        vector_results: Vec<SearchResult>,
        keyword_results: Vec<KeywordSearchResult>,
        weights: (f64, f64),
        request: &HybridSearchRequest,
    ) -> VectorResult<Vec<HybridSearchResult>> {
        // For simplicity, delegate to weighted scoring
        self.weighted_scoring_fusion(vector_results, keyword_results, weights, request)
            .await
    }

    /// Post-process hybrid results
    async fn post_process_results(
        &self,
        mut results: Vec<HybridSearchResult>,
        request: &HybridSearchRequest,
    ) -> VectorResult<Vec<HybridSearchResult>> {
        // Apply minimum score threshold
        if let Some(min_score) = request.min_hybrid_score {
            results.retain(|r| r.hybrid_score >= min_score);
        }

        // Apply diversification if enabled
        if self.config.enable_diversification {
            results = self.diversify_hybrid_results(results);
        }

        // Add explanations if requested
        if request.include_explanations {
            for result in &mut results {
                result.explanation = Some(self.create_explanation(result));
            }
        }

        // Limit results
        results.truncate(request.options.limit);

        Ok(results)
    }

    /// Create content snippet
    fn create_snippet(&self, content: &str) -> String {
        if content.len() > 200 {
            format!("{}...", &content[..197])
        } else {
            content.to_string()
        }
    }

    /// Diversify hybrid results to reduce near-duplicates
    fn diversify_hybrid_results(
        &self,
        results: Vec<HybridSearchResult>,
    ) -> Vec<HybridSearchResult> {
        if results.len() <= 2 {
            return results;
        }

        let mut diversified = Vec::new();
        let mut used_content_hashes = HashSet::new();

        for result in results {
            let content_hash = self.calculate_content_hash(&result.document.content);

            if !used_content_hashes.contains(&content_hash) {
                used_content_hashes.insert(content_hash);
                diversified.push(result);
            }
        }

        diversified
    }

    /// Calculate content hash for diversification
    fn calculate_content_hash(&self, content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        let prefix = if content.len() > 100 {
            &content[..100]
        } else {
            content
        };
        prefix.hash(&mut hasher);
        hasher.finish()
    }

    /// Create explanation for hybrid search result
    fn create_explanation(&self, result: &HybridSearchResult) -> HybridSearchExplanation {
        let vector_contrib = result.vector_score.map(|score| score * 0.6); // Using default weight
        let keyword_contrib = result.keyword_score.map(|score| score * 0.4);

        let calculation = match (&result.vector_score, &result.keyword_score) {
            (Some(vs), Some(ks)) => format!(
                "vector({:.3} * 0.6) + keyword({:.3} * 0.4) = {:.3}",
                vs, ks, result.hybrid_score
            ),
            (Some(vs), None) => format!("vector_only({:.3}) = {:.3}", vs, result.hybrid_score),
            (None, Some(ks)) => format!("keyword_only({:.3}) = {:.3}", ks, result.hybrid_score),
            (None, None) => "no_scores_available".to_string(),
        };

        HybridSearchExplanation {
            vector_contribution: vector_contrib,
            keyword_contribution: keyword_contrib,
            fusion_details: format!("{:?} fusion applied", result.fusion_method),
            strategy_rationale: format!("{:?} strategy used", result.strategy),
            weights: (0.6, 0.4), // Default weights for now
            calculation,
        }
    }

    /// Check cache for existing results
    async fn check_cache(&self, query: &str) -> Option<HybridSearchResultSet> {
        let cache = self.cache.read().await;
        if let Some((result_set, timestamp)) = cache.get(query) {
            let age = timestamp.elapsed().as_secs();
            if age < self.config.cache_ttl_seconds {
                return Some(result_set.clone());
            }
        }
        None
    }

    /// Cache search results
    async fn cache_result(&self, query: &str, result_set: &HybridSearchResultSet) {
        let mut cache = self.cache.write().await;
        cache.insert(query.to_string(), (result_set.clone(), Instant::now()));

        // Simple cache cleanup - remove entries older than TTL
        let ttl = self.config.cache_ttl_seconds;
        cache.retain(|_, (_, timestamp)| timestamp.elapsed().as_secs() < ttl);
    }

    /// Update analytics
    async fn update_analytics(&self, result_set: &HybridSearchResultSet) {
        let mut analytics = self.analytics.write().await;

        analytics.total_searches += 1;

        // Update strategy usage
        let strategy_name = format!("{:?}", result_set.query_analysis.recommended_strategy);
        *analytics.strategy_usage.entry(strategy_name).or_insert(0) += 1;

        // Update execution time
        let current_avg = analytics.avg_execution_time_ms;
        let new_time = result_set.execution_stats.total_time_ms;
        analytics.avg_execution_time_ms = if analytics.total_searches == 1 {
            new_time
        } else {
            (current_avg * (analytics.total_searches - 1) as f64 + new_time)
                / analytics.total_searches as f64
        };

        // Update result count
        let current_avg_results = analytics.avg_result_count;
        let new_results = result_set.results.len() as f64;
        analytics.avg_result_count = if analytics.total_searches == 1 {
            new_results
        } else {
            (current_avg_results * (analytics.total_searches - 1) as f64 + new_results)
                / analytics.total_searches as f64
        };
    }

    /// Get analytics data
    pub async fn get_analytics(&self) -> HybridSearchAnalytics {
        self.analytics.read().await.clone()
    }

    /// Clear analytics data
    pub async fn clear_analytics(&self) -> VectorResult<()> {
        let mut analytics = self.analytics.write().await;
        *analytics = HybridSearchAnalytics::default();
        info!("Hybrid search analytics cleared");
        Ok(())
    }
}

/// Trait for hybrid search operations
#[async_trait]
pub trait HybridSearchOperations: Send + Sync {
    /// Perform hybrid search combining vector and keyword search
    async fn hybrid_search(
        &self,
        request: HybridSearchRequest,
    ) -> VectorResult<HybridSearchResultSet>;

    /// Automatically select optimal search strategy for a query
    async fn adaptive_search(
        &self,
        query: &str,
        options: SearchOptions,
    ) -> VectorResult<HybridSearchResultSet>;

    /// Compare multiple search strategies for a query
    async fn compare_strategies(
        &self,
        query: &str,
        strategies: Vec<SearchStrategy>,
        options: SearchOptions,
    ) -> VectorResult<Vec<HybridSearchResultSet>>;

    /// Explain how a hybrid result was scored
    async fn explain_hybrid_result(
        &self,
        result: &HybridSearchResult,
    ) -> VectorResult<HybridSearchExplanation>;

    /// Perform fallback search when primary method fails
    async fn fallback_search(
        &self,
        query: &str,
        failed_method: SearchStrategy,
        options: SearchOptions,
    ) -> VectorResult<HybridSearchResultSet>;

    /// Initialize the service
    async fn initialize(&self) -> VectorResult<()>;
}

#[async_trait]
impl HybridSearchOperations for HybridSearchService {
    async fn hybrid_search(
        &self,
        request: HybridSearchRequest,
    ) -> VectorResult<HybridSearchResultSet> {
        self.hybrid_search(request).await
    }

    #[instrument(skip(self, query, options))]
    async fn adaptive_search(
        &self,
        query: &str,
        options: SearchOptions,
    ) -> VectorResult<HybridSearchResultSet> {
        // Analyze query to determine optimal strategy
        let query_analysis = self.analyze_query(query).await?;

        let request = HybridSearchRequest {
            query: query.to_string(),
            strategy: Some(query_analysis.recommended_strategy.clone()),
            fusion_method: Some(self.config.default_fusion_method.clone()),
            options,
            include_explanations: true,
            custom_weights: None,
            min_hybrid_score: None,
        };

        debug!(
            "Adaptive search selected strategy: {:?} for query: '{}'",
            query_analysis.recommended_strategy, query
        );

        self.hybrid_search(request).await
    }

    #[instrument(skip(self, query, strategies, options))]
    async fn compare_strategies(
        &self,
        query: &str,
        strategies: Vec<SearchStrategy>,
        options: SearchOptions,
    ) -> VectorResult<Vec<HybridSearchResultSet>> {
        let mut results = Vec::new();

        for strategy in strategies {
            let request = HybridSearchRequest {
                query: query.to_string(),
                strategy: Some(strategy.clone()),
                fusion_method: Some(self.config.default_fusion_method.clone()),
                options: options.clone(),
                include_explanations: true,
                custom_weights: None,
                min_hybrid_score: None,
            };

            match self.hybrid_search(request).await {
                Ok(result_set) => {
                    debug!(
                        "Strategy {:?} found {} results",
                        strategy,
                        result_set.results.len()
                    );
                    results.push(result_set);
                }
                Err(e) => {
                    warn!("Strategy {:?} failed: {}", strategy, e);
                    // Continue with other strategies
                }
            }
        }

        info!(
            "Compared {} strategies for query: '{}'",
            results.len(),
            query
        );
        Ok(results)
    }

    async fn explain_hybrid_result(
        &self,
        result: &HybridSearchResult,
    ) -> VectorResult<HybridSearchExplanation> {
        if let Some(existing_explanation) = &result.explanation {
            return Ok(existing_explanation.clone());
        }

        // Create detailed explanation
        Ok(self.create_explanation(result))
    }

    #[instrument(skip(self, query, failed_method, options))]
    async fn fallback_search(
        &self,
        query: &str,
        failed_method: SearchStrategy,
        options: SearchOptions,
    ) -> VectorResult<HybridSearchResultSet> {
        // Determine fallback strategy based on what failed
        let fallback_strategy = match failed_method {
            SearchStrategy::VectorOnly => SearchStrategy::KeywordOnly,
            SearchStrategy::KeywordOnly => SearchStrategy::VectorOnly,
            SearchStrategy::SemanticFocus => SearchStrategy::KeywordFocus,
            SearchStrategy::KeywordFocus => SearchStrategy::SemanticFocus,
            _ => SearchStrategy::Balanced, // Default fallback
        };

        warn!(
            "Primary search strategy {:?} failed, falling back to {:?}",
            failed_method, fallback_strategy
        );

        let request = HybridSearchRequest {
            query: query.to_string(),
            strategy: Some(fallback_strategy),
            fusion_method: Some(FusionMethod::ReciprocalRankFusion), // Use robust fusion method
            options,
            include_explanations: true,
            custom_weights: None,
            min_hybrid_score: None,
        };

        self.hybrid_search(request).await
    }

    async fn initialize(&self) -> VectorResult<()> {
        self.initialize().await
    }
}

// Additional convenience methods for HybridSearchService
impl HybridSearchService {
    /// Perform quick hybrid search with default settings
    #[instrument(skip(self, query))]
    pub async fn quick_search(&self, query: &str) -> VectorResult<HybridSearchResultSet> {
        let request = HybridSearchRequest {
            query: query.to_string(),
            strategy: None,      // Use default or adaptive
            fusion_method: None, // Use default
            options: SearchOptions::default(),
            include_explanations: false,
            custom_weights: None,
            min_hybrid_score: None,
        };

        self.hybrid_search(request).await
    }

    /// Search with custom fusion method
    #[instrument(skip(self, query, fusion_method, options))]
    pub async fn search_with_fusion(
        &self,
        query: &str,
        fusion_method: FusionMethod,
        options: SearchOptions,
    ) -> VectorResult<HybridSearchResultSet> {
        let request = HybridSearchRequest {
            query: query.to_string(),
            strategy: None,
            fusion_method: Some(fusion_method),
            options,
            include_explanations: true,
            custom_weights: None,
            min_hybrid_score: None,
        };

        self.hybrid_search(request).await
    }

    /// Search with custom strategy and weights
    #[instrument(skip(self, query, strategy, weights, options))]
    pub async fn search_with_strategy(
        &self,
        query: &str,
        strategy: SearchStrategy,
        weights: Option<(f64, f64)>,
        options: SearchOptions,
    ) -> VectorResult<HybridSearchResultSet> {
        let request = HybridSearchRequest {
            query: query.to_string(),
            strategy: Some(strategy),
            fusion_method: None,
            options,
            include_explanations: true,
            custom_weights: weights,
            min_hybrid_score: None,
        };

        self.hybrid_search(request).await
    }

    /// Optimize search strategy based on query feedback
    #[instrument(skip(self, query, strategies))]
    pub async fn optimize_strategy(
        &self,
        query: &str,
        strategies: Vec<SearchStrategy>,
    ) -> VectorResult<SearchStrategy> {
        let options = SearchOptions::default();
        let comparison_results = self.compare_strategies(query, strategies, options).await?;

        // Simple optimization: pick strategy with highest average score
        let mut best_strategy = SearchStrategy::Balanced;
        let mut best_score = 0.0;

        for result_set in comparison_results {
            if let Some(strategy) = result_set.request.strategy {
                let avg_score = if result_set.results.is_empty() {
                    0.0
                } else {
                    result_set
                        .results
                        .iter()
                        .map(|r| r.hybrid_score)
                        .sum::<f64>()
                        / result_set.results.len() as f64
                };

                if avg_score > best_score {
                    best_score = avg_score;
                    best_strategy = strategy;
                }
            }
        }

        debug!(
            "Optimized strategy for query '{}': {:?} (score: {:.3})",
            query, best_strategy, best_score
        );

        Ok(best_strategy)
    }

    /// Get search performance metrics
    pub async fn get_performance_metrics(&self) -> VectorResult<PerformanceMetrics> {
        let analytics = self.get_analytics().await;

        Ok(PerformanceMetrics {
            total_searches: analytics.total_searches,
            avg_execution_time_ms: analytics.avg_execution_time_ms,
            cache_hit_rate: analytics.cache_hit_rate,
            avg_result_count: analytics.avg_result_count,
            strategy_distribution: analytics.strategy_usage,
            fusion_method_distribution: analytics.fusion_method_usage,
        })
    }
}

/// Performance metrics for hybrid search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total searches performed
    pub total_searches: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: f64,
    /// Cache hit rate (0.0-1.0)
    pub cache_hit_rate: f64,
    /// Average number of results returned
    pub avg_result_count: f64,
    /// Distribution of strategy usage
    pub strategy_distribution: HashMap<String, u64>,
    /// Distribution of fusion method usage
    pub fusion_method_distribution: HashMap<String, u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_document(id: &str, content: &str) -> VectorDocument {
        VectorDocument {
            id: id.to_string(),
            content: content.to_string(),
            embedding: vec![0.1, 0.2, 0.3], // Mock embedding
            metadata: crate::vector::storage::DocumentMetadata {
                research_type: None,
                content_type: "test".to_string(),
                quality_score: Some(0.8),
                source: None,
                tags: Vec::new(),
                custom_fields: HashMap::new(),
            },
            stored_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_hybrid_search_config_default() {
        let config = HybridSearchConfig::default();
        assert_eq!(config.vector_weight, 0.6);
        assert_eq!(config.keyword_weight, 0.4);
        assert!(matches!(
            config.default_fusion_method,
            FusionMethod::ReciprocalRankFusion
        ));
        assert!(matches!(config.default_strategy, SearchStrategy::Balanced));
        assert_eq!(config.min_vector_threshold, 0.5);
        assert_eq!(config.max_results_per_type, 50);
        assert!(config.enable_adaptive_strategy);
        assert!(config.enable_diversification);
    }

    #[test]
    fn test_search_strategy_weights() {
        let config = HybridSearchConfig::default();

        assert_eq!(SearchStrategy::Balanced.get_weights(&config), (0.5, 0.5));
        assert_eq!(
            SearchStrategy::SemanticFocus.get_weights(&config),
            (0.8, 0.2)
        );
        assert_eq!(
            SearchStrategy::KeywordFocus.get_weights(&config),
            (0.2, 0.8)
        );
        assert_eq!(SearchStrategy::VectorOnly.get_weights(&config), (1.0, 0.0));
        assert_eq!(SearchStrategy::KeywordOnly.get_weights(&config), (0.0, 1.0));

        let custom = SearchStrategy::Custom {
            vector_weight: 0.7,
            keyword_weight: 0.3,
        };
        assert_eq!(custom.get_weights(&config), (0.7, 0.3));

        // Test adaptive strategy falls back to config defaults
        assert_eq!(SearchStrategy::Adaptive.get_weights(&config), (0.6, 0.4));
    }

    #[test]
    fn test_fusion_methods() {
        // Test that fusion methods can be created and compared
        assert_eq!(
            FusionMethod::ReciprocalRankFusion,
            FusionMethod::ReciprocalRankFusion
        );
        assert_ne!(
            FusionMethod::ReciprocalRankFusion,
            FusionMethod::WeightedScoring
        );

        // Test all fusion method variants
        let methods = [
            FusionMethod::ReciprocalRankFusion,
            FusionMethod::WeightedScoring,
            FusionMethod::RankFusion,
            FusionMethod::MaxScore,
            FusionMethod::LinearInterpolation,
            FusionMethod::MLFusion,
        ];

        assert_eq!(methods.len(), 6);
    }

    #[test]
    fn test_query_type_detection() {
        // Test query type enumeration
        assert_eq!(QueryType::Conceptual, QueryType::Conceptual);
        assert_ne!(QueryType::Technical, QueryType::Keywords);

        // Test all query type variants
        let types = [
            QueryType::Conceptual,
            QueryType::Technical,
            QueryType::Question,
            QueryType::Keywords,
            QueryType::Mixed,
        ];

        assert_eq!(types.len(), 5);
    }

    #[test]
    fn test_keyword_searcher_creation() {
        let searcher = KeywordSearcher::new();
        assert_eq!(searcher.total_documents, 0);
        assert!(searcher.tf_index.is_empty());
        assert!(searcher.df_index.is_empty());
        assert!(searcher.enable_normalization);

        // Test default field weights
        assert!(searcher.field_weights.contains_key("title"));
        assert!(searcher.field_weights.contains_key("content"));
        assert_eq!(searcher.field_weights.get("title"), Some(&3.0));
        assert_eq!(searcher.field_weights.get("content"), Some(&1.0));
    }

    #[test]
    fn test_tokenization() {
        let searcher = KeywordSearcher::new();
        let text = "Hello, World! This is a test.";
        let tokens = searcher.tokenize_and_normalize(text);

        // Should normalize to lowercase and remove punctuation
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"this".to_string()));
        assert!(tokens.contains(&"test".to_string()));

        // Should filter short words (length <= 2)
        assert!(!tokens.contains(&"a".to_string()));
        assert!(!tokens.contains(&"is".to_string()));
    }

    #[test]
    fn test_tokenization_without_normalization() {
        let mut searcher = KeywordSearcher::new();
        searcher.enable_normalization = false;

        let text = "Hello World";
        let tokens = searcher.tokenize_and_normalize(text);

        // Should not normalize case or filter when normalization disabled
        assert_eq!(tokens, vec!["Hello".to_string(), "World".to_string()]);
    }

    #[test]
    fn test_query_normalization() {
        let searcher = KeywordSearcher::new();
        let query = "Rust async programming patterns";
        let normalized = searcher.normalize_query(query);

        assert_eq!(
            normalized,
            vec![
                "rust".to_string(),
                "async".to_string(),
                "programming".to_string(),
                "patterns".to_string()
            ]
        );
    }

    #[test]
    fn test_tf_calculation() {
        let searcher = KeywordSearcher::new();
        let terms = vec![
            "rust".to_string(),
            "async".to_string(),
            "rust".to_string(),
            "programming".to_string(),
        ];

        let tf_scores = searcher.calculate_tf_scores(&terms);

        // "rust" appears 2/4 = 0.5
        assert_eq!(tf_scores.get("rust"), Some(&0.5));
        // "async" appears 1/4 = 0.25
        assert_eq!(tf_scores.get("async"), Some(&0.25));
        // "programming" appears 1/4 = 0.25
        assert_eq!(tf_scores.get("programming"), Some(&0.25));
    }

    #[test]
    fn test_idf_calculation() {
        let mut searcher = KeywordSearcher::new();
        searcher.total_documents = 100;

        // Term appears in 10 documents
        let idf = searcher.calculate_idf(10);
        let expected = (100.0_f64 / 10.0_f64).ln();
        assert!((idf - expected).abs() < 0.001);

        // Test edge case: df = 0
        let idf_zero = searcher.calculate_idf(0);
        assert_eq!(idf_zero, 0.0);
    }

    #[test]
    fn test_hybrid_search_request() {
        let request = HybridSearchRequest {
            query: "test query".to_string(),
            strategy: Some(SearchStrategy::Balanced),
            fusion_method: Some(FusionMethod::ReciprocalRankFusion),
            options: SearchOptions::default(),
            include_explanations: true,
            custom_weights: Some((0.7, 0.3)),
            min_hybrid_score: Some(0.5),
        };

        assert_eq!(request.query, "test query");
        assert!(request.include_explanations);
        assert_eq!(request.min_hybrid_score, Some(0.5));
        assert_eq!(request.custom_weights, Some((0.7, 0.3)));
        assert!(matches!(request.strategy, Some(SearchStrategy::Balanced)));
        assert!(matches!(
            request.fusion_method,
            Some(FusionMethod::ReciprocalRankFusion)
        ));
    }

    #[test]
    fn test_hybrid_search_result_sources() {
        let sources = SearchSources {
            from_vector: true,
            from_keyword: false,
            combined: false,
        };

        assert!(sources.from_vector);
        assert!(!sources.from_keyword);
        assert!(!sources.combined);
    }

    #[test]
    fn test_term_match() {
        let term_match = TermMatch {
            term: "rust".to_string(),
            field: "content".to_string(),
            position: 0,
            tf: 0.5,
            idf: 2.3,
            tf_idf: 1.15,
        };

        assert_eq!(term_match.term, "rust");
        assert_eq!(term_match.field, "content");
        assert_eq!(term_match.tf, 0.5);
        assert_eq!(term_match.idf, 2.3);
        assert_eq!(term_match.tf_idf, 1.15);
    }

    #[test]
    fn test_keyword_search_result() {
        let doc = create_test_document("test-doc", "Rust async programming guide");
        let term_matches = vec![TermMatch {
            term: "rust".to_string(),
            field: "content".to_string(),
            position: 0,
            tf: 0.25,
            idf: 2.0,
            tf_idf: 0.5,
        }];

        let result = KeywordSearchResult {
            document: doc,
            tf_idf_score: 0.5,
            term_matches,
            field_scores: HashMap::new(),
            matched_terms: 1,
            query_coverage: 0.5,
        };

        assert_eq!(result.tf_idf_score, 0.5);
        assert_eq!(result.matched_terms, 1);
        assert_eq!(result.query_coverage, 0.5);
        assert_eq!(result.term_matches.len(), 1);
    }

    #[test]
    fn test_hybrid_search_explanation() {
        let explanation = HybridSearchExplanation {
            vector_contribution: Some(0.8),
            keyword_contribution: Some(0.4),
            fusion_details: "RRF fusion applied".to_string(),
            strategy_rationale: "Balanced strategy used".to_string(),
            weights: (0.6, 0.4),
            calculation: "vector(0.8 * 0.6) + keyword(0.4 * 0.4) = 0.64".to_string(),
        };

        assert_eq!(explanation.vector_contribution, Some(0.8));
        assert_eq!(explanation.keyword_contribution, Some(0.4));
        assert_eq!(explanation.weights, (0.6, 0.4));
        assert!(explanation.fusion_details.contains("RRF"));
    }

    #[test]
    fn test_hybrid_match_metadata() {
        let base_metadata = MatchMetadata {
            search_time_ms: 10.0,
            original_rank: 0,
            final_rank: 0,
            matched_filters: 0,
            snippet: Some("Test snippet".to_string()),
        };

        let hybrid_metadata = HybridMatchMetadata {
            base_metadata,
            vector_rank: Some(1),
            keyword_rank: Some(2),
            matched_terms: 3,
            total_query_terms: 4,
            query_coverage: 0.75,
        };

        assert_eq!(hybrid_metadata.vector_rank, Some(1));
        assert_eq!(hybrid_metadata.keyword_rank, Some(2));
        assert_eq!(hybrid_metadata.matched_terms, 3);
        assert_eq!(hybrid_metadata.total_query_terms, 4);
        assert_eq!(hybrid_metadata.query_coverage, 0.75);
    }

    #[test]
    fn test_performance_metrics() {
        let mut strategy_distribution = HashMap::new();
        strategy_distribution.insert("Balanced".to_string(), 10);
        strategy_distribution.insert("SemanticFocus".to_string(), 5);

        let mut fusion_distribution = HashMap::new();
        fusion_distribution.insert("ReciprocalRankFusion".to_string(), 12);
        fusion_distribution.insert("WeightedScoring".to_string(), 3);

        let metrics = PerformanceMetrics {
            total_searches: 15,
            avg_execution_time_ms: 45.5,
            cache_hit_rate: 0.8,
            avg_result_count: 7.2,
            strategy_distribution,
            fusion_method_distribution: fusion_distribution,
        };

        assert_eq!(metrics.total_searches, 15);
        assert_eq!(metrics.avg_execution_time_ms, 45.5);
        assert_eq!(metrics.cache_hit_rate, 0.8);
        assert_eq!(metrics.avg_result_count, 7.2);
        assert_eq!(metrics.strategy_distribution.get("Balanced"), Some(&10));
        assert_eq!(
            metrics
                .fusion_method_distribution
                .get("ReciprocalRankFusion"),
            Some(&12)
        );
    }

    #[test]
    fn test_query_analysis() {
        let analysis = QueryAnalysis {
            original_query: "How to implement async patterns in Rust?".to_string(),
            terms: vec![
                "how".to_string(),
                "implement".to_string(),
                "async".to_string(),
                "patterns".to_string(),
                "rust".to_string(),
            ],
            query_type: QueryType::Question,
            complexity: 5.5,
            recommended_strategy: SearchStrategy::SemanticFocus,
            language: Some("en".to_string()),
            technical_terms: vec!["async".to_string(), "rust".to_string()],
        };

        assert_eq!(analysis.query_type, QueryType::Question);
        assert_eq!(analysis.complexity, 5.5);
        assert!(matches!(
            analysis.recommended_strategy,
            SearchStrategy::SemanticFocus
        ));
        assert_eq!(analysis.technical_terms.len(), 2);
        assert!(analysis.technical_terms.contains(&"rust".to_string()));
        assert!(analysis.technical_terms.contains(&"async".to_string()));
    }

    #[test]
    fn test_hybrid_search_analytics_default() {
        let analytics = HybridSearchAnalytics::default();
        assert_eq!(analytics.total_searches, 0);
        assert_eq!(analytics.avg_execution_time_ms, 0.0);
        assert_eq!(analytics.cache_hit_rate, 0.0);
        assert_eq!(analytics.avg_result_count, 0.0);
        assert!(analytics.strategy_usage.is_empty());
        assert!(analytics.fusion_method_usage.is_empty());
        assert!(analytics.strategy_effectiveness.is_empty());
    }

    #[tokio::test]
    async fn test_keyword_searcher_document_indexing() {
        let mut searcher = KeywordSearcher::new();

        let documents = vec![
            create_test_document("doc1", "Rust programming language async await"),
            create_test_document("doc2", "JavaScript async programming patterns"),
            create_test_document("doc3", "Rust systems programming performance"),
        ];

        let result = searcher.index_documents(documents).await;
        assert!(result.is_ok());

        assert_eq!(searcher.total_documents, 3);
        assert!(!searcher.tf_index.is_empty());
        assert!(!searcher.df_index.is_empty());

        // Check that terms exist in indices
        assert!(searcher.df_index.contains_key("rust"));
        assert!(searcher.df_index.contains_key("async"));
        assert!(searcher.df_index.contains_key("programming"));

        // "rust" should appear in 2 documents
        assert_eq!(searcher.df_index.get("rust"), Some(&2));
        // "async" should appear in 2 documents
        assert_eq!(searcher.df_index.get("async"), Some(&2));
        // "javascript" should appear in 1 document
        assert_eq!(searcher.df_index.get("javascript"), Some(&1));
    }

    #[tokio::test]
    async fn test_keyword_search_basic() {
        let mut searcher = KeywordSearcher::new();

        let documents = vec![
            create_test_document("doc1", "Rust async programming guide with tokio runtime"),
            create_test_document("doc2", "Python async programming tutorial"),
            create_test_document("doc3", "Rust systems programming and memory safety"),
        ];

        searcher.index_documents(documents).await.unwrap();

        // Search for "rust async"
        let results = searcher.search_keywords("rust async", 10).await.unwrap();

        // Should find documents containing rust and/or async
        assert!(!results.is_empty());

        // First result should be doc1 (contains both terms)
        assert_eq!(results[0].document.id, "doc1");
        assert!(results[0].matched_terms >= 1);
        assert!(results[0].tf_idf_score > 0.0);

        // Check term matches
        assert!(!results[0].term_matches.is_empty());
        let has_rust = results[0].term_matches.iter().any(|tm| tm.term == "rust");
        let has_async = results[0].term_matches.iter().any(|tm| tm.term == "async");
        assert!(has_rust || has_async);
    }

    #[tokio::test]
    async fn test_keyword_search_no_results() {
        let mut searcher = KeywordSearcher::new();

        let documents = vec![create_test_document("doc1", "Rust programming language")];

        searcher.index_documents(documents).await.unwrap();

        // Search for term not in documents
        let results = searcher.search_keywords("python django", 10).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_keyword_search_scoring() {
        let mut searcher = KeywordSearcher::new();

        let documents = vec![
            create_test_document("doc1", "rust rust rust programming"), // High frequency of "rust"
            create_test_document("doc2", "rust programming language"),  // Lower frequency
            create_test_document("doc3", "programming language design"), // No "rust"
        ];

        searcher.index_documents(documents).await.unwrap();

        let results = searcher.search_keywords("rust", 10).await.unwrap();

        // Should return 2 results (doc1 and doc2)
        assert_eq!(results.len(), 2);

        // doc1 should score higher due to higher term frequency
        assert_eq!(results[0].document.id, "doc1");
        assert_eq!(results[1].document.id, "doc2");
        assert!(results[0].tf_idf_score > results[1].tf_idf_score);
    }

    // Integration-style tests would be added here for full HybridSearchService
    // testing, but they would require mocked SemanticSearchService instances
    // which is beyond the scope of unit tests.
}
