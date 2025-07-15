//! Integration tests for context-aware pipeline processing
//!
//! These tests verify the end-to-end functionality of the context-aware
//! research pipeline with multi-dimensional classification.

use chrono::Utc;
use fortitude_core::classification::{
    AdvancedClassificationConfig, AdvancedClassifier, BasicClassifier, ContextDetectionConfig,
    ContextDetector, FortitudeContextDetector,
};
use fortitude_core::pipeline::{PipelineBuilder, PipelineConfig, ResearchPipeline};
use fortitude_types::{
    classification_result::{
        AudienceLevel, ClassificationDimension, DimensionConfidence, EnhancedClassificationResult,
        TechnicalDomain, UrgencyLevel,
    },
    AudienceContext, CacheEntry, CacheStats, ClassificationCandidate, ClassificationConfig,
    ClassificationError, ClassificationResult, ClassifiedRequest, Classifier, DomainContext,
    ResearchMetadata, ResearchResult, ResearchType, SearchQuery, SearchResult, Storage,
    StorageError,
};
use mockall::mock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{sleep, Duration};

// Mock implementations for integration testing
mock! {
    IntegrationClassifier {}

    impl Classifier for IntegrationClassifier {
        fn classify(&self, query: &str) -> Result<ClassificationResult, ClassificationError>;
        fn get_confidence(&self, query: &str, research_type: &ResearchType) -> f64;
        fn get_all_classifications(&self, query: &str) -> Vec<ClassificationCandidate>;
    }
}

mock! {
    IntegrationStorage {}

    #[async_trait::async_trait]
    impl Storage for IntegrationStorage {
        async fn store(&self, result: &ResearchResult) -> Result<String, StorageError>;
        async fn retrieve(&self, cache_key: &str) -> Result<Option<ResearchResult>, StorageError>;
        async fn delete(&self, cache_key: &str) -> Result<(), StorageError>;
        async fn list_cache_entries(&self) -> Result<Vec<CacheEntry>, StorageError>;
        async fn get_cache_stats(&self) -> Result<CacheStats, StorageError>;
        async fn cleanup_expired(&self) -> Result<u64, StorageError>;
        async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, StorageError>;
        async fn update_index(&self) -> Result<(), StorageError>;
    }
}

/// Helper to create a realistic mock classifier
fn create_realistic_mock_classifier() -> MockIntegrationClassifier {
    let mut mock = MockIntegrationClassifier::new();

    mock.expect_classify().returning(|query| {
        let query_lower = query.to_lowercase();

        let (research_type, confidence, keywords) = if query_lower.contains("learn")
            || query_lower.contains("what is")
            || query_lower.contains("beginner")
        {
            (
                ResearchType::Learning,
                0.9,
                vec!["learn".to_string(), "basics".to_string()],
            )
        } else if query_lower.contains("implement")
            || query_lower.contains("how to")
            || query_lower.contains("build")
        {
            (
                ResearchType::Implementation,
                0.85,
                vec!["implement".to_string(), "build".to_string()],
            )
        } else if query_lower.contains("debug")
            || query_lower.contains("error")
            || query_lower.contains("fix")
            || query_lower.contains("urgent")
        {
            (
                ResearchType::Troubleshooting,
                0.8,
                vec!["debug".to_string(), "fix".to_string()],
            )
        } else if query_lower.contains("choose")
            || query_lower.contains("compare")
            || query_lower.contains("decide")
        {
            (
                ResearchType::Decision,
                0.75,
                vec!["compare".to_string(), "decide".to_string()],
            )
        } else if query_lower.contains("validate")
            || query_lower.contains("review")
            || query_lower.contains("check")
        {
            (
                ResearchType::Validation,
                0.7,
                vec!["validate".to_string(), "review".to_string()],
            )
        } else {
            (ResearchType::Learning, 0.5, vec!["general".to_string()])
        };

        Ok(ClassificationResult::new(
            research_type,
            confidence,
            keywords,
            1,
            vec![],
        ))
    });

    mock.expect_get_confidence()
        .returning(|query, research_type| {
            let query_lower = query.to_lowercase();
            match research_type {
                ResearchType::Learning if query_lower.contains("learn") => 0.9,
                ResearchType::Implementation if query_lower.contains("implement") => 0.85,
                ResearchType::Troubleshooting if query_lower.contains("debug") => 0.8,
                ResearchType::Decision if query_lower.contains("choose") => 0.75,
                ResearchType::Validation if query_lower.contains("validate") => 0.7,
                _ => 0.5,
            }
        });

    mock.expect_get_all_classifications().returning(|query| {
        let query_lower = query.to_lowercase();
        vec![
            ClassificationCandidate::new(ResearchType::Learning, 0.6, vec!["learn".to_string()]),
            ClassificationCandidate::new(
                ResearchType::Implementation,
                0.5,
                vec!["implement".to_string()],
            ),
            ClassificationCandidate::new(
                ResearchType::Troubleshooting,
                0.4,
                vec!["debug".to_string()],
            ),
        ]
    });

    mock
}

/// Helper to create a realistic mock storage
fn create_realistic_mock_storage() -> MockIntegrationStorage {
    let mut mock = MockIntegrationStorage::new();

    mock.expect_retrieve().returning(|_| Ok(None)); // Always cache miss for testing

    mock.expect_store().returning(|result| {
        let cache_key = format!("cache_{}", result.request.original_query.len());
        Ok(cache_key)
    });

    mock.expect_list_cache_entries().returning(|| Ok(vec![]));

    mock.expect_get_cache_stats().returning(|| {
        Ok(CacheStats {
            total_entries: 0,
            hit_rate: 0.0,
            miss_rate: 1.0,
            total_size_bytes: 0,
            oldest_entry: None,
            newest_entry: None,
        })
    });

    mock.expect_cleanup_expired().returning(|| Ok(0));

    mock.expect_search().returning(|_| Ok(vec![]));

    mock.expect_update_index().returning(|| Ok(()));

    mock
}

#[tokio::test]
async fn test_context_aware_pipeline_end_to_end() {
    let mock_classifier = create_realistic_mock_classifier();
    let mock_storage = create_realistic_mock_storage();

    let config = PipelineConfig {
        enable_context_detection: true,
        enable_advanced_classification: true,
        enable_caching: true,
        max_concurrent: 3,
        timeout_seconds: 30,
        ..PipelineConfig::default()
    };

    let pipeline = ResearchPipeline::new(Arc::new(mock_classifier), Arc::new(mock_storage), config);

    // Test with a complex query that should trigger context detection
    let query = "I'm a beginner learning Rust and need help implementing async functions for web development";
    let audience = AudienceContext {
        level: "beginner".to_string(),
        domain: "web".to_string(),
        format: "tutorial".to_string(),
    };
    let domain = DomainContext {
        technology: "rust".to_string(),
        project_type: "web".to_string(),
        frameworks: vec!["tokio".to_string(), "axum".to_string()],
        tags: vec!["async".to_string(), "web".to_string()],
    };

    let result = pipeline
        .process_query(query, Some(audience.clone()), Some(domain.clone()))
        .await
        .expect("Pipeline should process query successfully");

    // Verify the result structure
    assert_eq!(result.request.original_query, query);
    assert_eq!(result.request.research_type, ResearchType::Implementation);
    assert_eq!(result.request.audience_context, audience);
    assert_eq!(result.request.domain_context, domain);
    assert!(result.request.confidence > 0.0);
    assert!(!result.request.matched_keywords.is_empty());

    // Verify response content
    assert!(!result.immediate_answer.is_empty());
    assert!(
        result.immediate_answer.contains("async functions")
            || result.immediate_answer.contains("Implementation")
    );

    // Verify metadata
    assert!(result.metadata.processing_time_ms > 0);
    assert!(!result.metadata.cache_key.is_empty());
    assert!(result.metadata.quality_score > 0.0);

    // Verify context-aware cache key generation
    let cache_key = result.metadata.cache_key.clone();
    assert!(cache_key.len() > 8); // Should be a reasonable hash length
}

#[tokio::test]
async fn test_pipeline_with_different_contexts() {
    let mock_classifier = create_realistic_mock_classifier();
    let mock_storage = create_realistic_mock_storage();

    let pipeline = PipelineBuilder::new()
        .with_context_detection(true)
        .with_advanced_classification(true)
        .with_caching(true)
        .build(Arc::new(mock_classifier), Arc::new(mock_storage));

    // Test different context combinations
    let test_cases = vec![
        (
            "How to debug memory leaks in C++?",
            AudienceContext {
                level: "advanced".to_string(),
                domain: "systems".to_string(),
                format: "detailed".to_string(),
            },
            DomainContext {
                technology: "cpp".to_string(),
                project_type: "systems".to_string(),
                frameworks: vec!["gdb".to_string()],
                tags: vec!["memory".to_string(), "debugging".to_string()],
            },
            ResearchType::Troubleshooting,
        ),
        (
            "What database should I choose for my web app?",
            AudienceContext {
                level: "intermediate".to_string(),
                domain: "web".to_string(),
                format: "comparison".to_string(),
            },
            DomainContext {
                technology: "database".to_string(),
                project_type: "web".to_string(),
                frameworks: vec![],
                tags: vec!["database".to_string(), "web".to_string()],
            },
            ResearchType::Decision,
        ),
        (
            "Validate my REST API design",
            AudienceContext {
                level: "advanced".to_string(),
                domain: "api".to_string(),
                format: "checklist".to_string(),
            },
            DomainContext {
                technology: "api".to_string(),
                project_type: "web".to_string(),
                frameworks: vec!["rest".to_string()],
                tags: vec!["api".to_string(), "validation".to_string()],
            },
            ResearchType::Validation,
        ),
    ];

    for (query, audience, domain, expected_type) in test_cases {
        let result = pipeline
            .process_query(query, Some(audience.clone()), Some(domain.clone()))
            .await
            .expect("Pipeline should process query successfully");

        assert_eq!(result.request.original_query, query);
        assert_eq!(result.request.research_type, expected_type);
        assert_eq!(result.request.audience_context, audience);
        assert_eq!(result.request.domain_context, domain);
        assert!(!result.immediate_answer.is_empty());
        assert!(result.metadata.processing_time_ms > 0);

        // Each context should generate different cache keys
        assert!(!result.metadata.cache_key.is_empty());
        println!(
            "Query: '{}' -> Cache key: {}",
            query, result.metadata.cache_key
        );
    }
}

#[tokio::test]
async fn test_pipeline_context_detection_fallback() {
    let mock_classifier = create_realistic_mock_classifier();
    let mock_storage = create_realistic_mock_storage();

    let config = PipelineConfig {
        enable_context_detection: true,
        enable_advanced_classification: true,
        enable_caching: false, // Disable caching for this test
        ..PipelineConfig::default()
    };

    let pipeline = ResearchPipeline::new(Arc::new(mock_classifier), Arc::new(mock_storage), config);

    // Test with ambiguous queries that should trigger fallback
    let ambiguous_queries = vec![
        "help me with something",
        "quick question",
        "need assistance",
        "xyz abc 123",
    ];

    for query in ambiguous_queries {
        let result = pipeline
            .process_query(query, None, None)
            .await
            .expect("Pipeline should handle ambiguous queries gracefully");

        assert_eq!(result.request.original_query, query);
        assert!(result.request.confidence >= 0.0);
        assert!(!result.immediate_answer.is_empty());

        // Should use default contexts when none provided
        assert_eq!(result.request.audience_context, AudienceContext::default());
        assert_eq!(result.request.domain_context, DomainContext::default());
    }
}

#[tokio::test]
async fn test_pipeline_concurrent_processing() {
    let mock_classifier = create_realistic_mock_classifier();
    let mock_storage = create_realistic_mock_storage();

    let config = PipelineConfig {
        enable_context_detection: true,
        enable_advanced_classification: true,
        enable_caching: true,
        max_concurrent: 5,
        ..PipelineConfig::default()
    };

    let pipeline = Arc::new(ResearchPipeline::new(
        Arc::new(mock_classifier),
        Arc::new(mock_storage),
        config,
    ));

    // Test concurrent processing
    let queries = vec![
        "How to implement authentication in Rust?",
        "What is the best web framework for Python?",
        "Debug memory issues in C++",
        "Choose between SQL and NoSQL databases",
        "Validate my microservices architecture",
    ];

    let start_time = Instant::now();

    // Process queries concurrently
    let mut handles = Vec::new();
    for query in queries {
        let pipeline_clone = pipeline.clone();
        let handle =
            tokio::spawn(async move { pipeline_clone.process_query(query, None, None).await });
        handles.push(handle);
    }

    // Wait for all queries to complete
    let results = futures::future::join_all(handles).await;
    let total_time = start_time.elapsed();

    // Verify all queries completed successfully
    for (i, result) in results.into_iter().enumerate() {
        let query_result = result
            .expect("Task should complete")
            .expect("Query should process successfully");

        assert!(!query_result.immediate_answer.is_empty());
        assert!(query_result.metadata.processing_time_ms > 0);

        println!(
            "Query {}: processed in {}ms",
            i, query_result.metadata.processing_time_ms
        );
    }

    // Concurrent processing should be faster than sequential
    assert!(
        total_time.as_millis() < 5000,
        "Concurrent processing should be efficient"
    );
}

#[tokio::test]
async fn test_pipeline_context_aware_caching() {
    let mock_classifier = create_realistic_mock_classifier();
    let mut mock_storage = MockIntegrationStorage::new();

    // Track cache interactions
    let mut cache_store = HashMap::new();

    mock_storage.expect_retrieve().returning(move |key| {
        if let Some(result) = cache_store.get(key) {
            Ok(Some(result.clone()))
        } else {
            Ok(None)
        }
    });

    mock_storage.expect_store().returning(move |result| {
        let cache_key = format!("cache_{}", result.request.original_query.len());
        Ok(cache_key)
    });

    let config = PipelineConfig {
        enable_context_detection: true,
        enable_advanced_classification: true,
        enable_caching: true,
        ..PipelineConfig::default()
    };

    let pipeline = ResearchPipeline::new(Arc::new(mock_classifier), Arc::new(mock_storage), config);

    let query = "How to implement OAuth2 in Rust?";

    // Test with different contexts - should generate different cache keys
    let context1 = AudienceContext {
        level: "beginner".to_string(),
        domain: "web".to_string(),
        format: "tutorial".to_string(),
    };

    let context2 = AudienceContext {
        level: "advanced".to_string(),
        domain: "security".to_string(),
        format: "reference".to_string(),
    };

    let result1 = pipeline
        .process_query(query, Some(context1.clone()), None)
        .await
        .expect("First query should succeed");

    let result2 = pipeline
        .process_query(query, Some(context2.clone()), None)
        .await
        .expect("Second query should succeed");

    // Same query with different contexts should have different cache keys
    assert_ne!(result1.metadata.cache_key, result2.metadata.cache_key);

    // Verify contexts are preserved
    assert_eq!(result1.request.audience_context, context1);
    assert_eq!(result2.request.audience_context, context2);
}

#[tokio::test]
async fn test_pipeline_advanced_classification_integration() {
    let mock_classifier = create_realistic_mock_classifier();
    let mock_storage = create_realistic_mock_storage();

    let advanced_config = AdvancedClassificationConfig {
        enable_context_detection: true,
        enable_graceful_degradation: true,
        max_processing_time_ms: 1000,
        ..Default::default()
    };

    let config = PipelineConfig {
        enable_context_detection: true,
        enable_advanced_classification: true,
        advanced_classification_config: Some(advanced_config),
        enable_caching: true,
        ..PipelineConfig::default()
    };

    let pipeline = ResearchPipeline::new(Arc::new(mock_classifier), Arc::new(mock_storage), config);

    // Test queries that should trigger advanced classification
    let test_cases = vec![
        (
            "I'm new to Rust and need help with ownership concepts",
            AudienceLevel::Beginner,
            TechnicalDomain::Rust,
            UrgencyLevel::Exploratory,
        ),
        (
            "URGENT: Production server down, need immediate debugging help",
            AudienceLevel::Intermediate,
            TechnicalDomain::General,
            UrgencyLevel::Immediate,
        ),
        (
            "Advanced microservices architecture patterns for enterprise",
            AudienceLevel::Advanced,
            TechnicalDomain::Architecture,
            UrgencyLevel::Planned,
        ),
    ];

    for (query, expected_audience, expected_domain, expected_urgency) in test_cases {
        let result = pipeline
            .process_query(query, None, None)
            .await
            .expect("Pipeline should process query successfully");

        assert_eq!(result.request.original_query, query);
        assert!(!result.immediate_answer.is_empty());

        // Advanced classification should provide enhanced context
        assert!(result.metadata.processing_time_ms > 0);
        assert!(result.metadata.processing_time_ms < 1000); // Should be fast

        // Response should be contextually appropriate
        if expected_audience == AudienceLevel::Beginner {
            assert!(
                result.immediate_answer.contains("basics")
                    || result.immediate_answer.contains("learning")
                    || result.immediate_answer.contains("beginner")
            );
        }

        if expected_urgency == UrgencyLevel::Immediate {
            assert!(
                result.immediate_answer.contains("immediate")
                    || result.immediate_answer.contains("urgent")
                    || result.immediate_answer.contains("quickly")
            );
        }

        println!(
            "Query: '{}' -> Response length: {}",
            query,
            result.immediate_answer.len()
        );
    }
}

#[tokio::test]
async fn test_pipeline_error_handling_and_recovery() {
    let mut mock_classifier = MockIntegrationClassifier::new();
    let mock_storage = create_realistic_mock_storage();

    // Setup classifier to fail on specific queries
    mock_classifier.expect_classify().returning(|query| {
        if query.contains("FAIL") {
            Err(ClassificationError::InvalidInput(
                "Test failure".to_string(),
            ))
        } else {
            Ok(ClassificationResult::new(
                ResearchType::Learning,
                0.7,
                vec!["test".to_string()],
                1,
                vec![],
            ))
        }
    });

    mock_classifier
        .expect_get_confidence()
        .returning(|_, _| 0.5);

    mock_classifier
        .expect_get_all_classifications()
        .returning(|_| vec![]);

    let config = PipelineConfig {
        enable_context_detection: true,
        enable_advanced_classification: false, // Disable to test basic path
        enable_caching: true,
        ..PipelineConfig::default()
    };

    let pipeline = ResearchPipeline::new(Arc::new(mock_classifier), Arc::new(mock_storage), config);

    // Test error handling
    let error_result = pipeline.process_query("FAIL this query", None, None).await;
    assert!(error_result.is_err());

    // Test recovery with valid query
    let success_result = pipeline.process_query("Valid query", None, None).await;
    assert!(success_result.is_ok());

    let result = success_result.unwrap();
    assert_eq!(result.request.original_query, "Valid query");
    assert!(!result.immediate_answer.is_empty());
}

#[tokio::test]
async fn test_pipeline_builder_configurations() {
    let mock_classifier = create_realistic_mock_classifier();
    let mock_storage = create_realistic_mock_storage();

    // Test different builder configurations
    let configs = vec![
        (
            "Basic configuration",
            PipelineBuilder::new()
                .with_caching(true)
                .with_max_concurrent(3)
                .with_timeout(30),
            false,
            false,
        ),
        (
            "Context detection enabled",
            PipelineBuilder::new()
                .with_context_detection(true)
                .with_caching(true),
            true,
            false,
        ),
        (
            "Advanced classification enabled",
            PipelineBuilder::new()
                .with_advanced_classification(true)
                .with_caching(true),
            true,
            true,
        ),
        (
            "Full features enabled",
            PipelineBuilder::new()
                .with_context_detection(true)
                .with_advanced_classification(true)
                .with_caching(true)
                .with_max_concurrent(5)
                .with_timeout(60),
            true,
            true,
        ),
    ];

    for (name, builder, expected_context, expected_advanced) in configs {
        let pipeline = builder.build(
            Arc::new(create_realistic_mock_classifier()),
            Arc::new(create_realistic_mock_storage()),
        );

        assert_eq!(pipeline.config.enable_context_detection, expected_context);
        assert_eq!(
            pipeline.config.enable_advanced_classification,
            expected_advanced
        );
        assert!(pipeline.config.enable_caching);

        // Test that pipeline works with this configuration
        let result = pipeline
            .process_query("Test query for configuration", None, None)
            .await
            .expect("Pipeline should work with any valid configuration");

        assert!(!result.immediate_answer.is_empty());

        println!("Configuration '{}' works correctly", name);
    }
}

#[tokio::test]
async fn test_pipeline_context_stability_across_requests() {
    let mock_classifier = create_realistic_mock_classifier();
    let mock_storage = create_realistic_mock_storage();

    let pipeline = PipelineBuilder::new()
        .with_context_detection(true)
        .with_advanced_classification(true)
        .with_caching(true)
        .build(Arc::new(mock_classifier), Arc::new(mock_storage));

    let stable_context = AudienceContext {
        level: "intermediate".to_string(),
        domain: "web".to_string(),
        format: "detailed".to_string(),
    };

    let domain_context = DomainContext {
        technology: "rust".to_string(),
        project_type: "web".to_string(),
        frameworks: vec!["axum".to_string()],
        tags: vec!["async".to_string()],
    };

    // Process multiple queries with the same context
    let queries = vec![
        "How to handle errors in Rust?",
        "What is the best way to structure a Rust project?",
        "How to implement middleware in Axum?",
    ];

    let mut cache_keys = Vec::new();

    for query in queries {
        let result = pipeline
            .process_query(
                query,
                Some(stable_context.clone()),
                Some(domain_context.clone()),
            )
            .await
            .expect("Query should succeed");

        // Verify context is preserved
        assert_eq!(result.request.audience_context, stable_context);
        assert_eq!(result.request.domain_context, domain_context);

        // Cache keys should be different for different queries but consistent for same query
        cache_keys.push(result.metadata.cache_key);
    }

    // All cache keys should be different (different queries)
    let unique_keys: std::collections::HashSet<_> = cache_keys.iter().collect();
    assert_eq!(unique_keys.len(), cache_keys.len());

    // But processing the same query again should give the same cache key
    let repeat_result = pipeline
        .process_query(
            "How to handle errors in Rust?",
            Some(stable_context.clone()),
            Some(domain_context.clone()),
        )
        .await
        .expect("Repeat query should succeed");

    assert_eq!(repeat_result.metadata.cache_key, cache_keys[0]);
}
