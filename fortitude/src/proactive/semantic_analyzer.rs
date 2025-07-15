// ABOUTME: Semantic analysis integration for knowledge gap detection using vector database
//! This module provides semantic analysis capabilities for the gap detection system.
//! It integrates with the vector database to validate gaps, find related content,
//! and enhance gap prioritization using semantic similarity analysis.

use crate::proactive::{DetectedGap, GapAnalysisError, GapType};
use fortitude_core::vector::{SearchOptions, SearchResult, SemanticSearchOperations};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use tokio::time::Duration;
use tracing::{debug, info, instrument, warn};

/// Errors that can occur during semantic analysis
#[derive(Error, Debug)]
pub enum SemanticAnalysisError {
    #[error("Vector database error: {0}")]
    VectorDatabase(#[from] fortitude_core::vector::VectorError),

    #[error("Gap analysis error: {0}")]
    GapAnalysis(#[from] GapAnalysisError),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Performance threshold exceeded: analysis took {duration:?}, limit is {limit:?}")]
    PerformanceThreshold { duration: Duration, limit: Duration },

    #[error("Semantic query construction failed: {0}")]
    QueryConstruction(String),

    #[error("Related content discovery failed for gap {gap_id}: {error}")]
    RelatedContentFailed { gap_id: String, error: String },
}

/// Configuration for semantic analysis integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalysisConfig {
    /// Enable semantic gap validation
    pub enable_gap_validation: bool,
    /// Enable related content discovery
    pub enable_related_content: bool,
    /// Enable semantic priority enhancement
    pub enable_priority_enhancement: bool,
    /// Maximum time allowed for semantic analysis (milliseconds)
    pub max_analysis_time_ms: u64,
    /// Similarity threshold for gap validation (0.0-1.0)
    pub gap_validation_threshold: f64,
    /// Similarity threshold for related content discovery (0.0-1.0)
    pub related_content_threshold: f64,
    /// Maximum number of related documents to consider per gap
    pub max_related_documents: usize,
    /// Minimum content length for semantic analysis
    pub min_content_length: usize,
    /// Batch size for vector queries
    pub batch_size: usize,
    /// Weight for semantic similarity in priority calculation
    pub semantic_priority_weight: f64,
}

impl Default for SemanticAnalysisConfig {
    fn default() -> Self {
        Self {
            enable_gap_validation: true,
            enable_related_content: true,
            enable_priority_enhancement: true,
            max_analysis_time_ms: 100, // 100ms overhead requirement
            gap_validation_threshold: 0.8,
            related_content_threshold: 0.7,
            max_related_documents: 10,
            min_content_length: 50,
            batch_size: 20,
            semantic_priority_weight: 0.3,
        }
    }
}

impl SemanticAnalysisConfig {
    /// Create configuration optimized for performance
    pub fn for_performance() -> Self {
        Self {
            max_analysis_time_ms: 50,
            max_related_documents: 5,
            batch_size: 10,
            gap_validation_threshold: 0.85,
            related_content_threshold: 0.75,
            ..Default::default()
        }
    }

    /// Create configuration optimized for accuracy
    pub fn for_accuracy() -> Self {
        Self {
            max_analysis_time_ms: 200,
            max_related_documents: 20,
            batch_size: 50,
            gap_validation_threshold: 0.7,
            related_content_threshold: 0.6,
            semantic_priority_weight: 0.5,
            ..Default::default()
        }
    }
}

/// Result of semantic analysis for a detected gap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticGapAnalysis {
    /// Original detected gap
    pub gap: DetectedGap,
    /// Whether the gap is validated (not already covered)
    pub is_validated: bool,
    /// Validation confidence score (0.0-1.0)
    pub validation_confidence: f64,
    /// Related documents found in the knowledge base
    pub related_documents: Vec<RelatedDocument>,
    /// Enhanced priority score with semantic considerations
    pub enhanced_priority: u8,
    /// Semantic analysis metadata
    pub metadata: SemanticAnalysisMetadata,
}

/// Information about a related document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedDocument {
    /// Document ID in the vector database
    pub document_id: String,
    /// Document content (truncated)
    pub content_preview: String,
    /// Similarity score to the gap (0.0-1.0)
    pub similarity_score: f64,
    /// Relationship type to the gap
    pub relationship_type: RelationshipType,
    /// Document metadata
    pub metadata: HashMap<String, String>,
}

/// Type of relationship between a gap and related content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Content partially covers the gap
    PartialCoverage,
    /// Content is closely related to the gap topic
    TopicalSimilarity,
    /// Content provides background context
    BackgroundContext,
    /// Content shows implementation patterns
    ImplementationPattern,
    /// Content is duplicated/similar gap
    DuplicateGap,
}

/// Metadata about the semantic analysis process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalysisMetadata {
    /// Time taken for semantic analysis (milliseconds)
    pub analysis_time_ms: f64,
    /// Number of vector queries performed
    pub vector_queries_count: usize,
    /// Query text used for semantic search
    pub search_query: String,
    /// Vector search results count
    pub search_results_count: usize,
    /// Semantic analysis features used
    pub features_used: Vec<String>,
}

/// Semantic gap analyzer that integrates with vector database
pub struct SemanticGapAnalyzer {
    /// Semantic search service for vector operations
    search_service: Arc<dyn SemanticSearchOperations>,
    /// Configuration for semantic analysis
    config: SemanticAnalysisConfig,
}

impl SemanticGapAnalyzer {
    /// Create a new semantic gap analyzer
    pub fn new(
        search_service: Arc<dyn SemanticSearchOperations>,
        config: SemanticAnalysisConfig,
    ) -> Self {
        Self {
            search_service,
            config,
        }
    }

    /// Create analyzer with default configuration
    pub fn with_defaults(search_service: Arc<dyn SemanticSearchOperations>) -> Self {
        Self::new(search_service, SemanticAnalysisConfig::default())
    }

    /// Analyze gaps with semantic enhancement
    #[instrument(skip(self, gaps))]
    pub async fn analyze_gaps_semantically(
        &self,
        gaps: Vec<DetectedGap>,
    ) -> Result<Vec<SemanticGapAnalysis>, SemanticAnalysisError> {
        let start_time = Instant::now();
        let mut semantic_analyses = Vec::new();

        // Process gaps in batches for better performance
        let batches: Vec<_> = gaps.chunks(self.config.batch_size).collect();
        let mut total_queries = 0;

        for batch in batches {
            for gap in batch {
                let analysis = self.analyze_single_gap(gap.clone()).await?;
                total_queries += analysis.metadata.vector_queries_count;
                semantic_analyses.push(analysis);
            }
        }

        let total_time = start_time.elapsed();
        let limit = Duration::from_millis(self.config.max_analysis_time_ms);

        if total_time > limit {
            warn!(
                "Semantic analysis exceeded time limit: {:?} > {:?} for {} gaps ({} queries)",
                total_time,
                limit,
                gaps.len(),
                total_queries
            );
        }

        info!(
            "Semantic analysis completed: {} gaps analyzed in {:?} with {} vector queries",
            gaps.len(),
            total_time,
            total_queries
        );

        Ok(semantic_analyses)
    }

    /// Analyze a single gap with semantic enhancement
    #[instrument(skip(self, gap))]
    async fn analyze_single_gap(
        &self,
        gap: DetectedGap,
    ) -> Result<SemanticGapAnalysis, SemanticAnalysisError> {
        let start_time = Instant::now();
        let mut query_count = 0;
        let mut features_used = Vec::new();

        // Construct semantic query from gap information
        let search_query = self.construct_semantic_query(&gap)?;

        // Initialize analysis result
        let mut analysis = SemanticGapAnalysis {
            gap: gap.clone(),
            is_validated: true, // Default to validated unless proven otherwise
            validation_confidence: 1.0,
            related_documents: Vec::new(),
            enhanced_priority: gap.priority,
            metadata: SemanticAnalysisMetadata {
                analysis_time_ms: 0.0,
                vector_queries_count: 0,
                search_query: search_query.clone(),
                search_results_count: 0,
                features_used: Vec::new(),
            },
        };

        // Perform gap validation if enabled
        if self.config.enable_gap_validation {
            let (is_valid, confidence) = self.validate_gap(&gap, &search_query).await?;
            analysis.is_validated = is_valid;
            analysis.validation_confidence = confidence;
            query_count += 1;
            features_used.push("gap_validation".to_string());
        }

        // Discover related content if enabled
        if self.config.enable_related_content {
            let related_docs = self.discover_related_content(&gap, &search_query).await?;
            analysis.related_documents = related_docs;
            query_count += 1;
            features_used.push("related_content".to_string());
        }

        // Enhance priority if enabled
        if self.config.enable_priority_enhancement {
            let enhanced_priority = self.enhance_priority(&gap, &analysis.related_documents)?;
            analysis.enhanced_priority = enhanced_priority;
            features_used.push("priority_enhancement".to_string());
        }

        // Update metadata
        analysis.metadata.analysis_time_ms = start_time.elapsed().as_millis() as f64;
        analysis.metadata.vector_queries_count = query_count;
        analysis.metadata.search_results_count = analysis.related_documents.len();
        analysis.metadata.features_used = features_used;

        debug!(
            "Semantic analysis completed for gap {:?}: validated={}, related_docs={}, enhanced_priority={}",
            gap.gap_type, analysis.is_validated, analysis.related_documents.len(), analysis.enhanced_priority
        );

        Ok(analysis)
    }

    /// Construct a semantic search query from gap information
    fn construct_semantic_query(&self, gap: &DetectedGap) -> Result<String, SemanticAnalysisError> {
        let mut query_parts = Vec::new();

        // Add gap description as primary query
        if !gap.description.is_empty() {
            query_parts.push(gap.description.clone());
        }

        // Add context information
        if !gap.context.is_empty() {
            query_parts.push(gap.context.clone());
        }

        // Add gap type specific keywords
        let type_keywords = match gap.gap_type {
            GapType::TodoComment => vec!["todo", "implementation", "task", "feature"],
            GapType::MissingDocumentation => vec!["documentation", "docs", "guide", "explanation"],
            GapType::UndocumentedTechnology => vec!["technology", "library", "dependency", "usage"],
            GapType::ApiDocumentationGap => vec!["api", "interface", "examples", "usage"],
            GapType::ConfigurationGap => vec!["configuration", "settings", "options", "setup"],
        };

        for keyword in type_keywords {
            query_parts.push(keyword.to_string());
        }

        // Add metadata as context
        for (key, value) in &gap.metadata {
            if !value.is_empty() {
                query_parts.push(format!("{key}: {value}"));
            }
        }

        let query = query_parts.join(" ");

        if query.len() < self.config.min_content_length {
            return Err(SemanticAnalysisError::QueryConstruction(format!(
                "Query too short: {} characters < {}",
                query.len(),
                self.config.min_content_length
            )));
        }

        Ok(query)
    }

    /// Validate whether a gap is actually missing or already covered
    async fn validate_gap(
        &self,
        _gap: &DetectedGap,
        search_query: &str,
    ) -> Result<(bool, f64), SemanticAnalysisError> {
        let search_options = SearchOptions {
            limit: 5,
            threshold: Some(self.config.gap_validation_threshold),
            include_explanations: false,
            diversify_results: true,
            ..SearchOptions::default()
        };

        let search_results = self
            .search_service
            .search_similar(search_query, search_options)
            .await?;

        // If we find highly similar content, the gap might already be covered
        let has_similar_content = search_results
            .results
            .iter()
            .any(|result| result.similarity_score >= self.config.gap_validation_threshold);

        if has_similar_content {
            // Gap might be already covered - calculate confidence based on similarity
            let max_similarity = search_results
                .results
                .iter()
                .map(|r| r.similarity_score)
                .fold(0.0, f64::max);

            let confidence = 1.0 - max_similarity; // Lower confidence when similar content exists
            Ok((false, confidence))
        } else {
            // Gap appears to be valid - high confidence
            Ok((true, 0.9))
        }
    }

    /// Discover related content for a gap
    async fn discover_related_content(
        &self,
        gap: &DetectedGap,
        search_query: &str,
    ) -> Result<Vec<RelatedDocument>, SemanticAnalysisError> {
        let search_options = SearchOptions {
            limit: self.config.max_related_documents,
            threshold: Some(self.config.related_content_threshold),
            include_explanations: false,
            diversify_results: true,
            ..SearchOptions::default()
        };

        let search_results = self
            .search_service
            .search_similar(search_query, search_options)
            .await?;

        let mut related_documents = Vec::new();

        for result in search_results.results {
            let relationship_type = self.determine_relationship_type(&result, gap);

            let related_doc = RelatedDocument {
                document_id: result.document.id.clone(),
                content_preview: self.create_content_preview(&result.document.content),
                similarity_score: result.similarity_score,
                relationship_type,
                metadata: result
                    .document
                    .metadata
                    .custom_fields
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect(),
            };

            related_documents.push(related_doc);
        }

        Ok(related_documents)
    }

    /// Determine the type of relationship between a search result and a gap
    fn determine_relationship_type(
        &self,
        result: &SearchResult,
        gap: &DetectedGap,
    ) -> RelationshipType {
        let similarity = result.similarity_score;
        let content = &result.document.content.to_lowercase();
        let gap_desc = gap.description.to_lowercase();

        // Very high similarity suggests duplicate or direct coverage
        if similarity >= 0.9 {
            return RelationshipType::DuplicateGap;
        }

        // High similarity with implementation keywords suggests patterns
        if similarity >= 0.8 && (content.contains("implementation") || content.contains("example"))
        {
            return RelationshipType::ImplementationPattern;
        }

        // Check for partial coverage keywords
        if similarity >= 0.75
            && (content.contains(&gap_desc)
                || gap_desc.contains("todo") && content.contains("todo"))
        {
            return RelationshipType::PartialCoverage;
        }

        // Medium similarity suggests topical relationship
        if similarity >= 0.7 {
            return RelationshipType::TopicalSimilarity;
        }

        // Lower similarity provides background context
        RelationshipType::BackgroundContext
    }

    /// Create a content preview from document content
    fn create_content_preview(&self, content: &str) -> String {
        const PREVIEW_LENGTH: usize = 200;

        if content.len() <= PREVIEW_LENGTH {
            content.to_string()
        } else {
            format!("{}...", &content[..PREVIEW_LENGTH])
        }
    }

    /// Enhance gap priority using semantic analysis results
    fn enhance_priority(
        &self,
        gap: &DetectedGap,
        related_documents: &[RelatedDocument],
    ) -> Result<u8, SemanticAnalysisError> {
        let base_priority = gap.priority as f64;
        let mut priority_adjustments = 0.0;

        // Increase priority if no related content exists (truly missing knowledge)
        if related_documents.is_empty() {
            priority_adjustments += 1.0;
        } else {
            // Analyze related documents for priority hints
            let max_similarity = related_documents
                .iter()
                .map(|doc| doc.similarity_score)
                .fold(0.0, f64::max);

            // Higher priority if only low-similarity content exists
            if max_similarity < 0.6 {
                priority_adjustments += 0.5;
            }

            // Check for implementation patterns that suggest immediate need
            let has_implementation_patterns = related_documents
                .iter()
                .any(|doc| doc.relationship_type == RelationshipType::ImplementationPattern);

            if has_implementation_patterns {
                priority_adjustments += 0.3;
            }
        }

        // Apply semantic weight
        let semantic_adjustment = priority_adjustments * self.config.semantic_priority_weight;
        let enhanced_priority = (base_priority + semantic_adjustment).clamp(1.0, 10.0);

        Ok(enhanced_priority as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proactive::DetectedGap;
    use mockall::mock;
    use std::path::PathBuf;

    // Mock the SearchOperations trait for testing
    mock! {
        SearchService {}

        #[async_trait::async_trait]
        impl SemanticSearchOperations for SearchService {
            async fn search_similar(
                &self,
                query: &str,
                options: SearchOptions,
            ) -> Result<fortitude_core::vector::SearchResultSet, fortitude_core::vector::VectorError>;

            async fn search_by_content(
                &self,
                content: &fortitude_core::vector::VectorDocument,
                options: SearchOptions,
            ) -> Result<fortitude_core::vector::SearchResultSet, fortitude_core::vector::VectorError>;

            async fn search_with_filters(
                &self,
                query: &str,
                filters: Vec<fortitude_core::vector::SemanticSearchFilter>,
                options: SearchOptions,
            ) -> Result<fortitude_core::vector::SearchResultSet, fortitude_core::vector::VectorError>;

            async fn suggest_related(
                &self,
                request: fortitude_core::vector::SuggestionRequest,
            ) -> Result<fortitude_core::vector::SearchResultSet, fortitude_core::vector::VectorError>;

            async fn search_batch(
                &self,
                request: fortitude_core::vector::BatchSearchRequest,
            ) -> Result<fortitude_core::vector::BatchSearchResult, fortitude_core::vector::VectorError>;

            async fn explain_search(
                &self,
                query: &str,
                result_id: &str,
            ) -> Result<Option<fortitude_core::vector::SearchExplanation>, fortitude_core::vector::VectorError>;

            async fn initialize(&self) -> Result<(), fortitude_core::vector::VectorError>;
        }
    }

    fn create_test_gap() -> DetectedGap {
        DetectedGap::new(
            GapType::TodoComment,
            PathBuf::from("test.rs"),
            10,
            "// TODO: Implement error handling".to_string(),
            "Implement error handling".to_string(),
            0.9,
        )
    }

    #[test]
    fn test_semantic_analysis_config_default() {
        let config = SemanticAnalysisConfig::default();
        assert!(config.enable_gap_validation);
        assert!(config.enable_related_content);
        assert!(config.enable_priority_enhancement);
        assert_eq!(config.max_analysis_time_ms, 100);
        assert_eq!(config.gap_validation_threshold, 0.8);
    }

    #[test]
    fn test_semantic_analysis_config_performance() {
        let config = SemanticAnalysisConfig::for_performance();
        assert_eq!(config.max_analysis_time_ms, 50);
        assert_eq!(config.max_related_documents, 5);
        assert_eq!(config.batch_size, 10);
    }

    #[test]
    fn test_semantic_analysis_config_accuracy() {
        let config = SemanticAnalysisConfig::for_accuracy();
        assert_eq!(config.max_analysis_time_ms, 200);
        assert_eq!(config.max_related_documents, 20);
        assert_eq!(config.semantic_priority_weight, 0.5);
    }

    #[test]
    fn test_construct_semantic_query() {
        let mut mock_search = MockSearchService::new();
        mock_search.expect_initialize().returning(|| Ok(()));

        let analyzer =
            SemanticGapAnalyzer::new(Arc::new(mock_search), SemanticAnalysisConfig::default());

        let gap = create_test_gap();
        let query = analyzer.construct_semantic_query(&gap).unwrap();

        assert!(query.contains("Implement error handling"));
        assert!(query.contains("todo"));
        assert!(query.contains("implementation"));
    }

    #[test]
    fn test_query_construction_too_short() {
        let mut mock_search = MockSearchService::new();
        mock_search.expect_initialize().returning(|| Ok(()));

        let config = SemanticAnalysisConfig {
            min_content_length: 1000,
            ..SemanticAnalysisConfig::default()
        };

        let analyzer = SemanticGapAnalyzer::new(Arc::new(mock_search), config);

        let gap = DetectedGap::new(
            GapType::TodoComment,
            PathBuf::from("test.rs"),
            1,
            "short".to_string(),
            "x".to_string(),
            0.9,
        );

        let result = analyzer.construct_semantic_query(&gap);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(SemanticAnalysisError::QueryConstruction(_))
        ));
    }

    #[test]
    fn test_determine_relationship_type() {
        let mut mock_search = MockSearchService::new();
        mock_search.expect_initialize().returning(|| Ok(()));

        let analyzer =
            SemanticGapAnalyzer::new(Arc::new(mock_search), SemanticAnalysisConfig::default());

        let gap = create_test_gap();

        // Create mock search result
        use chrono::Utc;
        use fortitude_core::vector::storage::DocumentMetadata;
        use fortitude_core::vector::{MatchMetadata, SearchResult, VectorDocument};

        let doc = VectorDocument {
            id: "test-doc".to_string(),
            content: "This is an implementation example".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata: DocumentMetadata::default(),
            stored_at: Utc::now(),
        };

        let result = SearchResult {
            document: doc,
            similarity_score: 0.85,
            relevance_score: 0.85,
            explanation: None,
            match_metadata: MatchMetadata {
                search_time_ms: 10.0,
                original_rank: 0,
                final_rank: 0,
                matched_filters: 0,
                snippet: None,
            },
        };

        let relationship = analyzer.determine_relationship_type(&result, &gap);
        assert_eq!(relationship, RelationshipType::ImplementationPattern);
    }

    #[test]
    fn test_create_content_preview() {
        let mut mock_search = MockSearchService::new();
        mock_search.expect_initialize().returning(|| Ok(()));

        let analyzer =
            SemanticGapAnalyzer::new(Arc::new(mock_search), SemanticAnalysisConfig::default());

        // Short content
        let short_content = "Short content";
        let preview = analyzer.create_content_preview(short_content);
        assert_eq!(preview, "Short content");

        // Long content
        let long_content = "a".repeat(300);
        let preview = analyzer.create_content_preview(&long_content);
        assert!(preview.len() <= 203); // 200 + "..."
        assert!(preview.ends_with("..."));
    }

    #[test]
    fn test_enhance_priority_no_related_docs() {
        let mut mock_search = MockSearchService::new();
        mock_search.expect_initialize().returning(|| Ok(()));

        let analyzer =
            SemanticGapAnalyzer::new(Arc::new(mock_search), SemanticAnalysisConfig::default());

        let gap = create_test_gap();
        let related_docs = Vec::new();

        let enhanced_priority = analyzer.enhance_priority(&gap, &related_docs).unwrap();

        // Should be higher than original due to no related content
        assert!(enhanced_priority >= gap.priority);
    }

    #[test]
    fn test_enhance_priority_with_implementation_patterns() {
        let mut mock_search = MockSearchService::new();
        mock_search.expect_initialize().returning(|| Ok(()));

        let analyzer =
            SemanticGapAnalyzer::new(Arc::new(mock_search), SemanticAnalysisConfig::default());

        let gap = create_test_gap();
        let related_docs = vec![RelatedDocument {
            document_id: "test-1".to_string(),
            content_preview: "Example implementation".to_string(),
            similarity_score: 0.8,
            relationship_type: RelationshipType::ImplementationPattern,
            metadata: HashMap::new(),
        }];

        let enhanced_priority = analyzer.enhance_priority(&gap, &related_docs).unwrap();

        // Should be enhanced due to implementation pattern
        assert!(enhanced_priority >= gap.priority);
    }

    #[test]
    fn test_relationship_type_serialization() {
        let types = vec![
            RelationshipType::PartialCoverage,
            RelationshipType::TopicalSimilarity,
            RelationshipType::BackgroundContext,
            RelationshipType::ImplementationPattern,
            RelationshipType::DuplicateGap,
        ];

        for rel_type in types {
            let serialized = serde_json::to_string(&rel_type).unwrap();
            let deserialized: RelationshipType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(rel_type, deserialized);
        }
    }

    #[test]
    fn test_semantic_analysis_metadata() {
        let metadata = SemanticAnalysisMetadata {
            analysis_time_ms: 45.5,
            vector_queries_count: 3,
            search_query: "test query".to_string(),
            search_results_count: 5,
            features_used: vec!["gap_validation".to_string(), "related_content".to_string()],
        };

        assert_eq!(metadata.analysis_time_ms, 45.5);
        assert_eq!(metadata.vector_queries_count, 3);
        assert_eq!(metadata.search_results_count, 5);
        assert_eq!(metadata.features_used.len(), 2);
    }
}
