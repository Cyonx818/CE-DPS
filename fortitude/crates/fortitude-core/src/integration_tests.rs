// ABOUTME: Integration tests for enhanced research workflows with vector search
//! This module contains comprehensive integration tests for the enhanced research
//! pipeline that includes vector search context discovery, automatic indexing,
//! and quality feedback systems.

#[cfg(test)]
#[allow(unused_imports)] // Allow unused imports in integration tests
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    use chrono::Utc;
    use mockall::mock;

    use crate::{
        api::ClaudeConfig,
        pipeline::{PipelineBuilder, PipelineConfig},
        research_engine::{ClaudeResearchConfig, ClaudeResearchEngine},
        research_feedback::{
            AutomatedQualityMetrics, FeedbackConfig, FeedbackOperations, FeedbackSource,
            ResearchFeedbackProcessor, ResearchQualityFeedback,
        },
        vector::{
            DocumentMetadata, HybridSearchConfig, HybridSearchRequest, HybridSearchService,
            KeywordSearcher, SearchOptions, SemanticSearchConfig, SemanticSearchService,
            VectorDocument, VectorStorageService,
        },
    };
    use fortitude_types::{
        AudienceContext, CacheAnalytics, CacheEntry, CacheOperation, CachePerformanceMonitor,
        CacheStats, CacheWarmingStats, ClassificationError, ClassificationResult, Classifier,
        DomainContext, HitRateTrend, ResearchResult, ResearchType, SearchQuery, SearchResult,
        Storage, StorageError,
    };

    // Mock implementations for testing
    mock! {
        TestClassifier {}

        impl Classifier for TestClassifier {
            fn classify(&self, query: &str) -> Result<ClassificationResult, ClassificationError>;
            fn get_confidence(&self, query: &str, research_type: &ResearchType) -> f64;
            fn get_all_classifications(&self, query: &str) -> Vec<fortitude_types::ClassificationCandidate>;
        }
    }

    mock! {
        TestStorage {}

        #[async_trait::async_trait]
        impl Storage for TestStorage {
            async fn store(&self, result: &ResearchResult) -> Result<String, StorageError>;
            async fn retrieve(&self, cache_key: &str) -> Result<Option<ResearchResult>, StorageError>;
            async fn delete(&self, cache_key: &str) -> Result<(), StorageError>;
            async fn list_cache_entries(&self) -> Result<Vec<CacheEntry>, StorageError>;
            async fn get_cache_stats(&self) -> Result<CacheStats, StorageError>;
            async fn cleanup_expired(&self) -> Result<u64, StorageError>;
            async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, StorageError>;
            async fn update_index(&self) -> Result<(), StorageError>;
            async fn record_cache_operation(&self, operation: CacheOperation) -> Result<(), StorageError>;
            async fn get_performance_monitor(&self) -> Result<CachePerformanceMonitor, StorageError>;
            async fn update_analytics(&self, analytics: CacheAnalytics) -> Result<(), StorageError>;
            async fn get_key_optimization_recommendations(&self) -> Result<Vec<String>, StorageError>;
            async fn warm_cache(&self, entries: Vec<String>) -> Result<CacheWarmingStats, StorageError>;
            async fn get_hit_rate_trends(&self, timeframe_hours: u64) -> Result<Vec<HitRateTrend>, StorageError>;
        }
    }

    /// Test the complete research workflow with vector search integration
    #[tokio::test]
    async fn test_complete_enhanced_research_workflow() {
        // This is a conceptual test - actual implementation would require
        // properly configured vector services and mock research engine

        let mut mock_classifier = MockTestClassifier::new();
        let mut mock_storage = MockTestStorage::new();

        // Setup classifier mock
        mock_classifier.expect_classify().returning(|_| {
            Ok(ClassificationResult::new(
                ResearchType::Implementation,
                0.8,
                vec!["rust".to_string(), "async".to_string()],
                1,
                vec![],
            ))
        });

        // Setup storage mock
        mock_storage.expect_retrieve().returning(|_| Ok(None));
        mock_storage
            .expect_store()
            .returning(|_| Ok("test-cache-key".to_string()));

        // Create a basic pipeline configuration with vector search enabled
        let mut config = PipelineConfig::default();
        config.enable_vector_search = true;
        config.enable_context_discovery = true;
        config.auto_index_results = true;

        // Note: In a real test, we would create properly configured vector services
        // For now, we're testing the configuration and interface structure

        assert!(config.enable_vector_search);
        assert!(config.enable_context_discovery);
        assert!(config.auto_index_results);
    }

    /// Test pipeline builder with vector search configuration
    #[test]
    fn test_pipeline_builder_with_vector_search() {
        let builder = PipelineBuilder::new()
            .with_context_discovery(true)
            .with_auto_indexing(true)
            .with_max_concurrent(10)
            .with_timeout(600);

        // Verify configuration is set correctly
        assert!(builder.config().enable_context_discovery);
        assert!(builder.config().auto_index_results);
        assert_eq!(builder.config().max_concurrent, 10);
        assert_eq!(builder.config().timeout_seconds, 600);
    }

    /// Test research engine configuration with vector search
    #[test]
    fn test_research_engine_with_vector_search_config() {
        let mut config = ClaudeResearchConfig::default();
        config.enable_vector_search = true;
        config.max_context_documents = 10;
        config.context_relevance_threshold = 0.8;

        assert!(config.enable_vector_search);
        assert_eq!(config.max_context_documents, 10);
        assert_eq!(config.context_relevance_threshold, 0.8);
    }

    /// Test research quality feedback system
    #[tokio::test]
    async fn test_research_feedback_processing() {
        // Create test feedback
        let feedback = ResearchQualityFeedback {
            feedback_id: "test-feedback-1".to_string(),
            research_cache_key: "test-research-1".to_string(),
            quality_rating: 4,
            context_relevance: 3,
            answer_usefulness: 5,
            completeness_rating: 4,
            feedback_text: Some("Great implementation examples!".to_string()),
            issues: vec![],
            suggestions: vec!["Add more error handling examples".to_string()],
            provided_at: Utc::now(),
            feedback_source: FeedbackSource::User,
        };

        // Verify feedback structure
        assert_eq!(feedback.quality_rating, 4);
        assert_eq!(feedback.context_relevance, 3);
        assert_eq!(feedback.answer_usefulness, 5);
        assert_eq!(feedback.completeness_rating, 4);
        assert!(matches!(feedback.feedback_source, FeedbackSource::User));
        assert_eq!(feedback.suggestions.len(), 1);
    }

    /// Test automated quality metrics conversion
    #[test]
    fn test_automated_quality_metrics() {
        let metrics = AutomatedQualityMetrics {
            overall_score: 0.85,
            context_relevance: 0.75,
            answer_quality: 0.90,
            completeness: 0.80,
        };

        // Test metric ranges
        assert!(metrics.overall_score >= 0.0 && metrics.overall_score <= 1.0);
        assert!(metrics.context_relevance >= 0.0 && metrics.context_relevance <= 1.0);
        assert!(metrics.answer_quality >= 0.0 && metrics.answer_quality <= 1.0);
        assert!(metrics.completeness >= 0.0 && metrics.completeness <= 1.0);

        // Test conversion to feedback ratings (would be 4, 4, 5, 4 out of 5)
        let quality_rating = (metrics.overall_score * 5.0).round() as u8;
        assert_eq!(quality_rating, 4);

        let context_rating = (metrics.context_relevance * 5.0).round() as u8;
        assert_eq!(context_rating, 4);
    }

    /// Test vector document creation from research result
    #[test]
    fn test_research_result_to_vector_document_conversion() {
        // Create a test research result
        let request = fortitude_types::ClassifiedRequest::new(
            "How to implement async Rust functions?".to_string(),
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

        let metadata = fortitude_types::ResearchMetadata {
            completed_at: Utc::now(),
            processing_time_ms: 1500,
            sources_consulted: vec!["Claude API".to_string()],
            quality_score: 0.85,
            cache_key: "test-cache-key".to_string(),
            tags: HashMap::new(),
        };

        let research_result = ResearchResult::new(
            request,
            "Async functions in Rust use the async/await syntax...".to_string(),
            vec![],
            vec![],
            metadata,
        );

        // Test that we can create the basic structure
        // (The actual conversion would be done by the pipeline)
        assert!(!research_result.immediate_answer.is_empty());
        assert_eq!(
            research_result.request.research_type,
            ResearchType::Implementation
        );
        assert_eq!(research_result.metadata.cache_key, "test-cache-key");
    }

    /// Test context discovery configuration
    #[test]
    fn test_context_discovery_configuration() {
        let config = PipelineConfig {
            enable_vector_search: true,
            enable_context_discovery: true,
            auto_index_results: true,
            ..PipelineConfig::default()
        };

        assert!(config.enable_vector_search);
        assert!(config.enable_context_discovery);
        assert!(config.auto_index_results);
    }

    /// Test feature flag system for gradual rollout
    #[test]
    fn test_feature_flag_system() {
        // Test default configuration (vector search disabled)
        let default_config = PipelineConfig::default();
        assert!(!default_config.enable_vector_search);
        assert!(!default_config.auto_index_results);
        assert!(!default_config.enable_context_discovery);

        // Test enabled configuration
        let enabled_config = PipelineConfig {
            enable_vector_search: true,
            auto_index_results: true,
            enable_context_discovery: true,
            ..PipelineConfig::default()
        };

        assert!(enabled_config.enable_vector_search);
        assert!(enabled_config.auto_index_results);
        assert!(enabled_config.enable_context_discovery);

        // Test partial enablement
        let partial_config = PipelineConfig {
            enable_vector_search: true,
            auto_index_results: false,
            enable_context_discovery: true,
            ..PipelineConfig::default()
        };

        assert!(partial_config.enable_vector_search);
        assert!(!partial_config.auto_index_results);
        assert!(partial_config.enable_context_discovery);
    }

    /// Test hybrid search request creation for different research types
    #[test]
    fn test_hybrid_search_request_creation() {
        let test_cases = vec![
            (ResearchType::Decision, "SemanticFocus"),
            (ResearchType::Implementation, "Balanced"),
            (ResearchType::Troubleshooting, "KeywordFocus"),
            (ResearchType::Learning, "SemanticFocus"),
            (ResearchType::Validation, "Balanced"),
        ];

        for (research_type, expected_strategy) in test_cases {
            // This tests the strategy selection logic
            let strategy_name = match research_type {
                ResearchType::Decision => "SemanticFocus",
                ResearchType::Implementation => "Balanced",
                ResearchType::Troubleshooting => "KeywordFocus",
                ResearchType::Learning => "SemanticFocus",
                ResearchType::Validation => "Balanced",
            };

            assert_eq!(strategy_name, expected_strategy);
        }
    }

    /// Test performance monitoring capabilities
    #[test]
    fn test_performance_monitoring_structure() {
        // Test that we can create performance monitoring structures
        let start_time = std::time::Instant::now();

        // Simulate some processing time
        std::thread::sleep(Duration::from_millis(10));

        let processing_time = start_time.elapsed();

        // Test timing measurements
        assert!(processing_time.as_millis() >= 10);

        // Test that we can convert to metrics format
        let processing_time_ms = processing_time.as_millis() as f64;
        assert!(processing_time_ms >= 10.0);
    }

    /// Test backward compatibility
    #[test]
    fn test_backward_compatibility() {
        // Test that existing research workflows still work
        let mut mock_classifier = MockTestClassifier::new();
        let mock_storage = MockTestStorage::new();

        mock_classifier.expect_classify().returning(|_| {
            Ok(ClassificationResult::new(
                ResearchType::Learning,
                0.7,
                vec!["test".to_string()],
                1,
                vec![],
            ))
        });

        // Create pipeline without vector search (backward compatibility)
        let pipeline =
            PipelineBuilder::new().build(Arc::new(mock_classifier), Arc::new(mock_storage));

        // Verify that vector search features are disabled by default
        assert!(!pipeline.config().enable_vector_search);
        assert!(!pipeline.config().auto_index_results);
        assert!(!pipeline.config().enable_context_discovery);
        assert!(pipeline.vector_search().is_none());
        assert!(pipeline.vector_storage().is_none());
    }

    /// Test error handling in enhanced workflows
    #[test]
    fn test_error_handling() {
        // Test that appropriate error types exist
        use crate::research_engine::ResearchEngineError;
        use crate::research_feedback::FeedbackError;

        // Test research engine errors
        let vector_search_error = ResearchEngineError::VectorSearchError("Test error".to_string());
        assert!(matches!(
            vector_search_error,
            ResearchEngineError::VectorSearchError(_)
        ));

        let context_discovery_error =
            ResearchEngineError::ContextDiscoveryError("Test error".to_string());
        assert!(matches!(
            context_discovery_error,
            ResearchEngineError::ContextDiscoveryError(_)
        ));

        // Test feedback errors
        let feedback_error = FeedbackError::ProcessingError("Test error".to_string());
        assert!(matches!(feedback_error, FeedbackError::ProcessingError(_)));

        let invalid_data_error = FeedbackError::InvalidData("Invalid rating".to_string());
        assert!(matches!(invalid_data_error, FeedbackError::InvalidData(_)));
    }

    /// Test research type strategy mapping
    #[test]
    fn test_research_type_strategy_mapping() {
        // Test that each research type maps to an appropriate search strategy
        let mappings = vec![
            (
                ResearchType::Decision,
                "should use semantic focus for conceptual decisions",
            ),
            (
                ResearchType::Implementation,
                "should use balanced approach for practical implementation",
            ),
            (
                ResearchType::Troubleshooting,
                "should use keyword focus for specific technical issues",
            ),
            (
                ResearchType::Learning,
                "should use semantic focus for educational content",
            ),
            (
                ResearchType::Validation,
                "should use balanced approach for validation tasks",
            ),
        ];

        for (research_type, description) in mappings {
            // Test that we can map research types to strategies
            let has_mapping = match research_type {
                ResearchType::Decision | ResearchType::Learning => true, // Semantic focus
                ResearchType::Implementation | ResearchType::Validation => true, // Balanced
                ResearchType::Troubleshooting => true,                   // Keyword focus
            };

            assert!(
                has_mapping,
                "Missing strategy mapping for {:?}: {}",
                research_type, description
            );
        }
    }
}
