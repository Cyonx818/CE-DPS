//! ANCHOR: Critical multi-dimensional classification system tests
//!
//! These tests verify critical functionality and should be maintained
//! as the system evolves. Do not delete these tests.
//!
//! Tests: Multi-dimensional classification accuracy, context detection,
//! pipeline integration, fallback mechanisms, performance requirements

use chrono::Utc;
use fortitude_core::classification::{
    advanced_classifier::{AdvancedClassificationConfig, AdvancedClassifier},
    context_detector::{ContextDetectionConfig, ContextDetector, FortitudeContextDetector},
    scoring::{ClassificationSignal, CompositionConfig, SignalComposer},
};
use fortitude_core::pipeline::{PipelineBuilder, PipelineConfig, ResearchPipeline};
use fortitude_types::{
    classification_result::{
        AudienceLevel, ClassificationDimension, DimensionConfidence, EnhancedClassificationResult,
        TechnicalDomain, UrgencyLevel,
    },
    AudienceContext, CacheAnalytics, CacheEntry, CacheOperation, CachePerformanceMonitor,
    CacheStats, CacheWarmingStats, ClassificationCandidate, ClassificationError,
    ClassificationResult, ClassifiedRequest, Classifier, DomainContext, HitRateTrend,
    ResearchMetadata, ResearchResult, ResearchType, SearchQuery, SearchResult, Storage,
    StorageError,
};
use mockall::mock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{timeout, Duration};

// Mock implementations for testing
mock! {
    TestClassifier {}

    impl Classifier for TestClassifier {
        fn classify(&self, query: &str) -> Result<ClassificationResult, ClassificationError>;
        fn get_confidence(&self, query: &str, research_type: &ResearchType) -> f64;
        fn get_all_classifications(&self, query: &str) -> Vec<ClassificationCandidate>;
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

/// Helper function to create a test advanced classifier
fn create_test_advanced_classifier() -> AdvancedClassifier {
    let config = AdvancedClassificationConfig {
        basic_config: fortitude_types::ClassificationConfig {
            default_threshold: 0.1, // Lower threshold for testing
            ..Default::default()
        },
        composition_config: CompositionConfig {
            confidence_threshold: 0.1,
            ..Default::default()
        },
        max_processing_time_ms: 5000, // 5 seconds for tests
        ..Default::default()
    };

    AdvancedClassifier::new(config)
}

/// Helper function to create a test context detector
fn create_test_context_detector() -> FortitudeContextDetector {
    let config = ContextDetectionConfig {
        confidence_threshold: 0.5,
        enable_fallback: true,
        max_processing_time_ms: 100,
        debug_logging: false,
    };
    FortitudeContextDetector::with_config(config)
}

/// ANCHOR: Verifies multi-dimensional classification accuracy >80%
/// Tests: Research type classification, audience detection, domain detection, urgency assessment
#[tokio::test]
async fn test_anchor_multi_dimensional_classification_accuracy() {
    let classifier = create_test_advanced_classifier();

    // Test cases with expected results
    let test_cases = vec![
        (
            "I'm new to Rust and need to learn the basics of ownership",
            ResearchType::Learning,
            AudienceLevel::Beginner,
            TechnicalDomain::Rust,
            UrgencyLevel::Exploratory,
        ),
        (
            "How to implement async functions in Rust for web development?",
            ResearchType::Implementation,
            AudienceLevel::Intermediate,
            TechnicalDomain::Rust,
            UrgencyLevel::Planned,
        ),
        (
            "URGENT: My production server is down, need immediate fix",
            ResearchType::Troubleshooting,
            AudienceLevel::Intermediate,
            TechnicalDomain::General,
            UrgencyLevel::Immediate,
        ),
        (
            "Which database should I choose for my web application?",
            ResearchType::Decision,
            AudienceLevel::Intermediate,
            TechnicalDomain::Web,
            UrgencyLevel::Planned,
        ),
        (
            "How to validate my API design follows best practices?",
            ResearchType::Validation,
            AudienceLevel::Advanced,
            TechnicalDomain::Web,
            UrgencyLevel::Planned,
        ),
    ];

    let mut correct_predictions = 0;
    let total_predictions = test_cases.len() * 4; // 4 dimensions per test case

    for (query, expected_research_type, expected_audience, expected_domain, expected_urgency) in
        test_cases
    {
        let result = classifier
            .classify_enhanced(query, &expected_research_type)
            .expect("Classification should succeed");

        // Check research type accuracy
        if result.research_type == expected_research_type {
            correct_predictions += 1;
        }

        // Check audience level accuracy
        if result.audience_level == expected_audience {
            correct_predictions += 1;
        }

        // Check technical domain accuracy
        if result.technical_domain == expected_domain {
            correct_predictions += 1;
        }

        // Check urgency level accuracy
        if result.urgency_level == expected_urgency {
            correct_predictions += 1;
        }

        // Verify confidence scores are present
        assert!(
            result.overall_confidence > 0.0,
            "Overall confidence should be > 0"
        );
        assert!(
            !result.dimension_confidences.is_empty(),
            "Should have dimension confidences"
        );

        // Verify processing time is reasonable
        assert!(
            result.metadata.processing_time_ms < 1000,
            "Processing should be < 1s"
        );
    }

    let accuracy = (correct_predictions as f64) / (total_predictions as f64);
    println!(
        "Multi-dimensional classification accuracy: {:.2}%",
        accuracy * 100.0
    );

    // ANCHOR: Require >80% accuracy for multi-dimensional classification
    assert!(
        accuracy > 0.8,
        "Multi-dimensional classification accuracy must be >80%, got {:.2}%",
        accuracy * 100.0
    );
}

/// ANCHOR: Verifies context detection accuracy for audience and domain identification
/// Tests: Context detection reliability, fallback mechanisms, confidence scoring
#[tokio::test]
async fn test_anchor_context_detection_accuracy() {
    let detector = create_test_context_detector();

    // Test cases with expected context detection results
    let test_cases = vec![
        (
            "I'm a beginner and need help with basic Rust concepts",
            ResearchType::Learning,
            AudienceLevel::Beginner,
            TechnicalDomain::Rust,
        ),
        (
            "How to optimize my React component performance?",
            ResearchType::Implementation,
            AudienceLevel::Intermediate,
            TechnicalDomain::Web,
        ),
        (
            "Advanced Kubernetes cluster configuration best practices",
            ResearchType::Decision,
            AudienceLevel::Advanced,
            TechnicalDomain::DevOps,
        ),
        (
            "Need immediate help with Python deployment issues",
            ResearchType::Troubleshooting,
            AudienceLevel::Intermediate,
            TechnicalDomain::Python,
        ),
        (
            "Validate my AI model architecture for production",
            ResearchType::Validation,
            AudienceLevel::Advanced,
            TechnicalDomain::AI,
        ),
    ];

    let mut correct_audience_detections = 0;
    let mut correct_domain_detections = 0;
    let total_tests = test_cases.len();

    for (query, research_type, expected_audience, expected_domain) in test_cases {
        let result = detector
            .detect_context(query, &research_type)
            .expect("Context detection should succeed");

        // Check audience detection
        if result.audience_level == expected_audience {
            correct_audience_detections += 1;
        }

        // Check domain detection
        if result.technical_domain == expected_domain {
            correct_domain_detections += 1;
        }

        // Verify confidence scores
        assert!(
            result.overall_confidence > 0.0,
            "Overall confidence should be > 0"
        );
        assert!(
            !result.dimension_confidences.is_empty(),
            "Should have dimension confidences"
        );

        // Verify processing time
        assert!(
            result.processing_time_ms < 100,
            "Context detection should be < 100ms"
        );
    }

    let audience_accuracy = (correct_audience_detections as f64) / (total_tests as f64);
    let domain_accuracy = (correct_domain_detections as f64) / (total_tests as f64);

    println!(
        "Audience detection accuracy: {:.2}%",
        audience_accuracy * 100.0
    );
    println!("Domain detection accuracy: {:.2}%", domain_accuracy * 100.0);

    // ANCHOR: Require >80% accuracy for context detection
    assert!(
        audience_accuracy > 0.8,
        "Audience detection accuracy must be >80%, got {:.2}%",
        audience_accuracy * 100.0
    );
    assert!(
        domain_accuracy > 0.8,
        "Domain detection accuracy must be >80%, got {:.2}%",
        domain_accuracy * 100.0
    );
}

/// ANCHOR: Verifies context-aware pipeline processing with cache integration
/// Tests: Pipeline integration, context-aware caching, cache key generation
#[tokio::test]
async fn test_anchor_context_aware_pipeline_processing() {
    let mut mock_classifier = MockTestClassifier::new();
    let mut mock_storage = MockTestStorage::new();

    // Setup classifier mock
    mock_classifier.expect_classify().times(1).returning(|_| {
        Ok(ClassificationResult::new(
            ResearchType::Implementation,
            0.9,
            vec!["async".to_string(), "rust".to_string()],
            1,
            vec![],
        ))
    });

    // Setup storage mock for cache miss, then store
    mock_storage
        .expect_retrieve()
        .times(1)
        .returning(|_| Ok(None));

    mock_storage
        .expect_store()
        .times(1)
        .returning(|_| Ok("context-aware-cache-key".to_string()));

    let config = PipelineConfig {
        enable_context_detection: true,
        enable_advanced_classification: true,
        advanced_classification_config: Some(AdvancedClassificationConfig::default()),
        ..PipelineConfig::default()
    };

    let pipeline = ResearchPipeline::new(Arc::new(mock_classifier), Arc::new(mock_storage), config);

    let result = pipeline
        .process_query(
            "How to implement async functions in Rust for web development?",
            Some(AudienceContext {
                level: "intermediate".to_string(),
                domain: "web".to_string(),
                format: "detailed".to_string(),
            }),
            Some(DomainContext {
                technology: "rust".to_string(),
                project_type: "web".to_string(),
                frameworks: vec!["tokio".to_string()],
                tags: vec!["async".to_string()],
            }),
        )
        .await
        .expect("Pipeline processing should succeed");

    // Verify result structure
    assert_eq!(result.request.research_type, ResearchType::Implementation);
    assert_eq!(
        result.request.original_query,
        "How to implement async functions in Rust for web development?"
    );
    assert!(!result.immediate_answer.is_empty());
    assert!(result.metadata.processing_time_ms > 0);

    // Verify cache key is context-aware
    assert!(!result.metadata.cache_key.is_empty());
    assert_ne!(result.metadata.cache_key, "simple-cache-key");
}

/// ANCHOR: Verifies performance meets <60s pipeline target
/// Tests: Performance regression prevention, pipeline speed, memory efficiency
#[tokio::test]
async fn test_anchor_performance_pipeline_target() {
    let mut mock_classifier = MockTestClassifier::new();
    let mut mock_storage = MockTestStorage::new();

    // Setup fast-responding mocks
    mock_classifier.expect_classify().returning(|_| {
        Ok(ClassificationResult::new(
            ResearchType::Learning,
            0.8,
            vec!["learn".to_string()],
            1,
            vec![],
        ))
    });

    mock_storage.expect_retrieve().returning(|_| Ok(None));

    mock_storage
        .expect_store()
        .returning(|_| Ok("perf-test-key".to_string()));

    let config = PipelineConfig {
        enable_context_detection: true,
        enable_advanced_classification: true,
        advanced_classification_config: Some(AdvancedClassificationConfig::default()),
        ..PipelineConfig::default()
    };

    let pipeline = ResearchPipeline::new(Arc::new(mock_classifier), Arc::new(mock_storage), config);

    // Test multiple queries to ensure consistent performance
    let test_queries = vec![
        "What is machine learning?",
        "How to deploy a web application?",
        "Debug memory leaks in C++",
        "Compare database technologies",
        "Validate API security measures",
    ];

    let start_time = Instant::now();

    for query in test_queries {
        let query_start = Instant::now();

        // Use timeout to enforce performance requirement
        let result = timeout(
            Duration::from_secs(60),
            pipeline.process_query(query, None, None),
        )
        .await
        .expect("Pipeline should complete within 60 seconds")
        .expect("Pipeline processing should succeed");

        let query_duration = query_start.elapsed();

        // ANCHOR: Each query should complete within 60 seconds
        assert!(
            query_duration.as_secs() < 60,
            "Query '{}' took {:.2}s, exceeding 60s limit",
            query,
            query_duration.as_secs_f64()
        );

        // Verify result quality
        assert_eq!(result.request.original_query, query);
        assert!(result.metadata.processing_time_ms > 0);
        assert!(!result.immediate_answer.is_empty());
    }

    let total_duration = start_time.elapsed();
    let average_duration = total_duration.as_secs_f64() / 5.0;

    println!("Average query processing time: {:.2}s", average_duration);

    // ANCHOR: Average processing time should be well under 60s
    assert!(
        average_duration < 10.0,
        "Average processing time should be < 10s for efficiency, got {:.2}s",
        average_duration
    );
}

/// ANCHOR: Verifies fallback mechanisms when context detection fails
/// Tests: Graceful degradation, fallback to basic classification, error handling
#[tokio::test]
async fn test_anchor_fallback_mechanisms() {
    let config = AdvancedClassificationConfig {
        enable_context_detection: true,
        enable_graceful_degradation: true,
        max_processing_time_ms: 5000,
        ..Default::default()
    };

    let classifier = AdvancedClassifier::new(config);

    // Test with ambiguous/unclear queries that might trigger fallback
    let ambiguous_queries = vec![
        "xyz123",           // Random text
        "",                 // Empty (should error)
        "a",                // Single character
        "help me",          // Too vague
        "urgent something", // Unclear context
    ];

    for query in ambiguous_queries {
        if query.is_empty() {
            // Empty query should error
            let result = classifier.classify_enhanced(query, &ResearchType::Learning);
            assert!(result.is_err(), "Empty query should fail");
            continue;
        }

        let result = classifier.classify_enhanced(query, &ResearchType::Learning);

        // Should succeed due to graceful degradation
        assert!(
            result.is_ok(),
            "Query '{}' should succeed with fallback",
            query
        );

        let enhanced_result = result.unwrap();

        // Verify fallback behavior
        assert!(
            enhanced_result.overall_confidence >= 0.0,
            "Should have some confidence"
        );
        assert!(
            !enhanced_result.dimension_confidences.is_empty(),
            "Should have dimension confidences"
        );

        // Verify metadata indicates fallback was used
        assert!(enhanced_result
            .metadata
            .tags
            .contains_key("context_detection"));

        // Processing time should be reasonable even with fallback
        assert!(
            enhanced_result.metadata.processing_time_ms < 5000,
            "Processing should be < 5s even with fallback"
        );
    }
}

/// ANCHOR: Verifies signal composition accuracy and weighting
/// Tests: Signal composition, weighted rule application, conflict resolution
#[tokio::test]
async fn test_anchor_signal_composition_accuracy() {
    let composer = SignalComposer::with_balanced_rules().expect("Should create balanced composer");

    // Test signal composition with multiple signals
    let test_signals = vec![
        ClassificationSignal::ResearchType(
            ResearchType::Implementation,
            0.9,
            vec!["implement".to_string(), "build".to_string()],
        ),
        ClassificationSignal::AudienceLevel(
            AudienceLevel::Intermediate,
            0.8,
            vec!["intermediate".to_string()],
        ),
        ClassificationSignal::TechnicalDomain(
            TechnicalDomain::Rust,
            0.85,
            vec!["rust".to_string()],
        ),
        ClassificationSignal::UrgencyLevel(UrgencyLevel::Planned, 0.7, vec!["planned".to_string()]),
    ];

    let result = composer
        .compose_signals(test_signals)
        .expect("Signal composition should succeed");

    // Verify composition results
    assert_eq!(result.research_type, ResearchType::Implementation);
    assert_eq!(result.audience_level, AudienceLevel::Intermediate);
    assert_eq!(result.technical_domain, TechnicalDomain::Rust);
    assert_eq!(result.urgency_level, UrgencyLevel::Planned);

    // Verify confidence scoring
    assert!(
        result.overall_confidence > 0.0,
        "Overall confidence should be > 0"
    );
    assert_eq!(
        result.dimension_confidences.len(),
        4,
        "Should have 4 dimension confidences"
    );

    // Verify all keywords are preserved
    assert!(
        !result.matched_keywords.is_empty(),
        "Should have matched keywords"
    );
    assert!(result.matched_keywords.contains(&"implement".to_string()));
    assert!(result.matched_keywords.contains(&"rust".to_string()));

    // Verify processing time
    assert!(result.processing_time_ms > 0, "Should have processing time");
    assert!(
        result.processing_time_ms < 100,
        "Signal composition should be fast"
    );
}

/// ANCHOR: Verifies backward compatibility with existing CLI interface
/// Tests: API compatibility, existing workflows, configuration options
#[tokio::test]
async fn test_anchor_backward_compatibility() {
    let classifier = create_test_advanced_classifier();

    // Test basic Classifier trait compatibility
    let basic_result = classifier
        .classify("How to debug Rust code?")
        .expect("Basic classification should work");

    assert_eq!(basic_result.research_type, ResearchType::Troubleshooting);
    assert!(basic_result.confidence > 0.0);
    assert!(!basic_result.matched_keywords.is_empty());

    // Test get_confidence method
    let confidence = classifier.get_confidence("implement", &ResearchType::Implementation);
    assert!(
        confidence > 0.0,
        "Should return confidence for implementation query"
    );

    // Test get_all_classifications method
    let candidates = classifier.get_all_classifications("test and validate");
    assert!(
        !candidates.is_empty(),
        "Should return classification candidates"
    );

    // Verify that enhanced features don't break basic functionality
    let enhanced_result = classifier
        .classify_enhanced("Basic Rust question", &ResearchType::Learning)
        .expect("Enhanced classification should work");

    // Basic fields should still be accessible
    assert_eq!(enhanced_result.research_type, ResearchType::Learning);
    assert!(enhanced_result.overall_confidence > 0.0);
    assert!(!enhanced_result.matched_keywords.is_empty());

    // Enhanced fields should be available
    assert!(enhanced_result.dimension_confidences.len() > 0);
    assert!(enhanced_result.metadata.processing_time_ms > 0);
}

/// ANCHOR: Verifies cache hit rate improvement with context awareness
/// Tests: Cache efficiency, context-aware keys, performance improvement
#[tokio::test]
async fn test_anchor_cache_efficiency_improvement() {
    let mut mock_classifier = MockTestClassifier::new();
    let mut mock_storage = MockTestStorage::new();

    // Setup classifier mock
    mock_classifier
        .expect_classify()
        .times(2) // Two different queries
        .returning(|query| {
            if query.contains("async") {
                Ok(ClassificationResult::new(
                    ResearchType::Implementation,
                    0.9,
                    vec!["async".to_string()],
                    1,
                    vec![],
                ))
            } else {
                Ok(ClassificationResult::new(
                    ResearchType::Learning,
                    0.8,
                    vec!["learn".to_string()],
                    1,
                    vec![],
                ))
            }
        });

    // Setup storage mock for cache behavior
    let mut call_count = 0;
    mock_storage
        .expect_retrieve()
        .times(4) // Two queries, each called twice
        .returning(move |_| {
            call_count += 1;
            if call_count <= 2 {
                Ok(None) // First calls are cache misses
            } else {
                // Create a cached result for subsequent calls
                let cached_request = ClassifiedRequest::new(
                    "cached query".to_string(),
                    ResearchType::Implementation,
                    AudienceContext::default(),
                    DomainContext::default(),
                    0.9,
                    vec!["cached".to_string()],
                );

                let cached_metadata = ResearchMetadata {
                    completed_at: Utc::now(),
                    processing_time_ms: 50,
                    sources_consulted: vec!["cache".to_string()],
                    quality_score: 0.9,
                    cache_key: "cached-key".to_string(),
                    tags: HashMap::new(),
                };

                let cached_result = ResearchResult::new(
                    cached_request,
                    "Cached response".to_string(),
                    vec![],
                    vec![],
                    cached_metadata,
                );

                Ok(Some(cached_result))
            }
        });

    mock_storage
        .expect_store()
        .times(2)
        .returning(|_| Ok("stored-key".to_string()));

    let config = PipelineConfig {
        enable_context_detection: true,
        enable_advanced_classification: true,
        enable_caching: true,
        ..PipelineConfig::default()
    };

    let pipeline = ResearchPipeline::new(Arc::new(mock_classifier), Arc::new(mock_storage), config);

    // Test queries with different contexts
    let query1 = "How to implement async functions in Rust?";
    let query2 = "What is Rust ownership?";

    // First calls - should be cache misses
    let start_time = Instant::now();
    let result1_first = pipeline
        .process_query(query1, None, None)
        .await
        .expect("First query should succeed");
    let first_call_time = start_time.elapsed();

    let result2_first = pipeline
        .process_query(query2, None, None)
        .await
        .expect("Second query should succeed");

    // Second calls - should be cache hits (faster)
    let start_time = Instant::now();
    let result1_second = pipeline
        .process_query(query1, None, None)
        .await
        .expect("First query repeat should succeed");
    let second_call_time = start_time.elapsed();

    let result2_second = pipeline
        .process_query(query2, None, None)
        .await
        .expect("Second query repeat should succeed");

    // Verify cache hit behavior
    assert_eq!(result1_second.immediate_answer, "Cached response");
    assert_eq!(result2_second.immediate_answer, "Cached response");

    // Cache hits should be faster
    assert!(
        second_call_time < first_call_time,
        "Cache hit should be faster than cache miss"
    );

    // Verify context-aware cache keys are different for different contexts
    assert_ne!(
        result1_first.metadata.cache_key,
        result2_first.metadata.cache_key
    );
}
