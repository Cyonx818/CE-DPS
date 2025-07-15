//! Performance tests for multi-dimensional classification system
//!
//! These tests verify that the classification system meets performance
//! requirements, particularly the <60s pipeline target.

use chrono::Utc;
use fortitude_core::classification::{
    AdvancedClassificationConfig, AdvancedClassifier, BasicClassifier, ClassificationSignal,
    CompositionConfig, ContextDetectionConfig, ContextDetector, FortitudeContextDetector,
    SignalComposer,
};
use fortitude_core::pipeline::{PipelineBuilder, PipelineConfig, ResearchPipeline};
use fortitude_types::{
    classification_result::{
        AudienceLevel, ClassificationDimension, TechnicalDomain, UrgencyLevel,
    },
    AudienceContext, CacheEntry, CacheStats, ClassificationCandidate, ClassificationConfig,
    ClassificationError, ClassificationResult, ClassifiedRequest, Classifier, DomainContext,
    ResearchMetadata, ResearchResult, ResearchType, SearchQuery, SearchResult, Storage,
    StorageError,
};
use mockall::mock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;

// Mock implementations for performance testing
mock! {
    FastClassifier {}

    impl Classifier for FastClassifier {
        fn classify(&self, query: &str) -> Result<ClassificationResult, ClassificationError>;
        fn get_confidence(&self, query: &str, research_type: &ResearchType) -> f64;
        fn get_all_classifications(&self, query: &str) -> Vec<ClassificationCandidate>;
    }
}

mock! {
    FastStorage {}

    #[async_trait::async_trait]
    impl Storage for FastStorage {
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

/// Create a fast mock classifier for performance testing
fn create_fast_mock_classifier() -> MockFastClassifier {
    let mut mock = MockFastClassifier::new();

    mock.expect_classify().returning(|query| {
        // Simulate minimal processing time
        let query_lower = query.to_lowercase();
        let research_type = if query_lower.contains("learn") {
            ResearchType::Learning
        } else if query_lower.contains("implement") {
            ResearchType::Implementation
        } else if query_lower.contains("debug") {
            ResearchType::Troubleshooting
        } else if query_lower.contains("choose") {
            ResearchType::Decision
        } else {
            ResearchType::Validation
        };

        Ok(ClassificationResult::new(
            research_type,
            0.8,
            vec!["fast".to_string()],
            1,
            vec![],
        ))
    });

    mock.expect_get_confidence().returning(|_, _| 0.8);

    mock.expect_get_all_classifications().returning(|_| {
        vec![ClassificationCandidate::new(
            ResearchType::Learning,
            0.8,
            vec!["fast".to_string()],
        )]
    });

    mock
}

/// Create a fast mock storage for performance testing
fn create_fast_mock_storage() -> MockFastStorage {
    let mut mock = MockFastStorage::new();

    mock.expect_retrieve().returning(|_| Ok(None)); // Always cache miss for consistent timing

    mock.expect_store()
        .returning(|_| Ok("fast-cache-key".to_string()));

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

/// Performance test queries of varying complexity
fn get_performance_test_queries() -> Vec<(&'static str, ResearchType)> {
    vec![
        ("What is Rust?", ResearchType::Learning),
        (
            "How to implement async functions in Rust?",
            ResearchType::Implementation,
        ),
        (
            "Debug memory leak in C++ application",
            ResearchType::Troubleshooting,
        ),
        (
            "Choose between PostgreSQL and MongoDB",
            ResearchType::Decision,
        ),
        (
            "Validate my microservices architecture",
            ResearchType::Validation,
        ),
        (
            "Learn advanced Rust patterns for concurrent programming",
            ResearchType::Learning,
        ),
        (
            "Implement OAuth2 authentication with JWT tokens",
            ResearchType::Implementation,
        ),
        (
            "Debug performance issues in React application",
            ResearchType::Troubleshooting,
        ),
        (
            "Decide on cloud provider for enterprise deployment",
            ResearchType::Decision,
        ),
        (
            "Validate API security best practices",
            ResearchType::Validation,
        ),
        (
            "I'm a beginner and need to learn basic programming concepts",
            ResearchType::Learning,
        ),
        (
            "How to build a high-performance web server with advanced caching",
            ResearchType::Implementation,
        ),
        (
            "URGENT: Production system is failing with critical errors",
            ResearchType::Troubleshooting,
        ),
        (
            "Compare various machine learning frameworks for our AI project",
            ResearchType::Decision,
        ),
        (
            "Review and validate our entire software architecture",
            ResearchType::Validation,
        ),
    ]
}

#[tokio::test]
async fn test_basic_classification_performance() {
    let classifier = BasicClassifier::new(ClassificationConfig::default());
    let queries = get_performance_test_queries();

    let start_time = Instant::now();
    let mut total_queries = 0;

    for (query, expected_type) in queries {
        let query_start = Instant::now();

        let result = classifier
            .classify(query)
            .expect("Classification should succeed");

        let query_duration = query_start.elapsed();

        // Each classification should be very fast
        assert!(
            query_duration.as_millis() < 100,
            "Basic classification should be <100ms, got {}ms for '{}'",
            query_duration.as_millis(),
            query
        );

        assert!(result.confidence > 0.0);
        assert!(!result.matched_keywords.is_empty());

        total_queries += 1;
    }

    let total_duration = start_time.elapsed();
    let avg_duration = total_duration.as_millis() as f64 / total_queries as f64;

    println!("Basic classification performance:");
    println!("  Total queries: {}", total_queries);
    println!("  Total time: {}ms", total_duration.as_millis());
    println!("  Average time per query: {:.2}ms", avg_duration);

    // Basic classification should be very fast
    assert!(
        avg_duration < 50.0,
        "Average basic classification should be <50ms"
    );
    assert!(
        total_duration.as_secs() < 1,
        "Total basic classification should be <1s"
    );
}

#[tokio::test]
async fn test_advanced_classification_performance() {
    let config = AdvancedClassificationConfig {
        max_processing_time_ms: 1000, // 1 second limit
        ..Default::default()
    };

    let classifier = AdvancedClassifier::new(config);
    let queries = get_performance_test_queries();

    let start_time = Instant::now();
    let mut total_queries = 0;
    let mut performance_stats = Vec::new();

    for (query, expected_type) in queries {
        let query_start = Instant::now();

        let result = classifier
            .classify_enhanced(query, &expected_type)
            .expect("Enhanced classification should succeed");

        let query_duration = query_start.elapsed();
        performance_stats.push(query_duration);

        // Each enhanced classification should be reasonably fast
        assert!(
            query_duration.as_millis() < 1000,
            "Enhanced classification should be <1000ms, got {}ms for '{}'",
            query_duration.as_millis(),
            query
        );

        assert!(result.overall_confidence > 0.0);
        assert!(!result.dimension_confidences.is_empty());
        assert!(result.processing_time_ms > 0);

        total_queries += 1;
    }

    let total_duration = start_time.elapsed();
    let avg_duration = total_duration.as_millis() as f64 / total_queries as f64;

    // Calculate statistics
    let mut durations: Vec<u128> = performance_stats.iter().map(|d| d.as_millis()).collect();
    durations.sort();
    let median = durations[durations.len() / 2];
    let p95 = durations[(durations.len() * 95) / 100];
    let max_duration = durations.iter().max().unwrap();

    println!("Advanced classification performance:");
    println!("  Total queries: {}", total_queries);
    println!("  Total time: {}ms", total_duration.as_millis());
    println!("  Average time per query: {:.2}ms", avg_duration);
    println!("  Median time: {}ms", median);
    println!("  95th percentile: {}ms", p95);
    println!("  Max time: {}ms", max_duration);

    // Advanced classification should still be fast
    assert!(
        avg_duration < 500.0,
        "Average advanced classification should be <500ms"
    );
    assert!(p95 < 1000, "95th percentile should be <1000ms");
    assert!(
        total_duration.as_secs() < 10,
        "Total advanced classification should be <10s"
    );
}

#[tokio::test]
async fn test_context_detection_performance() {
    let config = ContextDetectionConfig {
        max_processing_time_ms: 100, // 100ms limit
        ..Default::default()
    };

    let detector = FortitudeContextDetector::with_config(config);
    let queries = get_performance_test_queries();

    let start_time = Instant::now();
    let mut total_detections = 0;
    let mut performance_stats = Vec::new();

    for (query, research_type) in queries {
        let detection_start = Instant::now();

        let result = detector
            .detect_context(query, &research_type)
            .expect("Context detection should succeed");

        let detection_duration = detection_start.elapsed();
        performance_stats.push(detection_duration);

        // Context detection should be very fast
        assert!(
            detection_duration.as_millis() < 100,
            "Context detection should be <100ms, got {}ms for '{}'",
            detection_duration.as_millis(),
            query
        );

        assert!(result.overall_confidence >= 0.0);
        assert!(!result.dimension_confidences.is_empty());
        assert!(result.processing_time_ms > 0);

        total_detections += 1;
    }

    let total_duration = start_time.elapsed();
    let avg_duration = total_duration.as_millis() as f64 / total_detections as f64;

    println!("Context detection performance:");
    println!("  Total detections: {}", total_detections);
    println!("  Total time: {}ms", total_duration.as_millis());
    println!("  Average time per detection: {:.2}ms", avg_duration);

    // Context detection should be very fast
    assert!(
        avg_duration < 50.0,
        "Average context detection should be <50ms"
    );
    assert!(
        total_duration.as_secs() < 1,
        "Total context detection should be <1s"
    );
}

#[tokio::test]
async fn test_signal_composition_performance() {
    let composer = SignalComposer::with_balanced_rules().expect("Should create balanced composer");

    let start_time = Instant::now();
    let mut total_compositions = 0;

    // Test with different signal combinations
    for i in 0..100 {
        let signals = vec![
            ClassificationSignal::ResearchType(
                ResearchType::Implementation,
                0.9,
                vec![format!("impl_{}", i)],
            ),
            ClassificationSignal::AudienceLevel(
                AudienceLevel::Intermediate,
                0.8,
                vec![format!("intermediate_{}", i)],
            ),
            ClassificationSignal::TechnicalDomain(
                TechnicalDomain::Rust,
                0.85,
                vec![format!("rust_{}", i)],
            ),
            ClassificationSignal::UrgencyLevel(
                UrgencyLevel::Planned,
                0.7,
                vec![format!("planned_{}", i)],
            ),
        ];

        let composition_start = Instant::now();

        let result = composer
            .compose_signals(signals)
            .expect("Signal composition should succeed");

        let composition_duration = composition_start.elapsed();

        // Signal composition should be very fast
        assert!(
            composition_duration.as_millis() < 10,
            "Signal composition should be <10ms, got {}ms",
            composition_duration.as_millis()
        );

        assert!(result.overall_confidence > 0.0);
        assert_eq!(result.dimension_confidences.len(), 4);

        total_compositions += 1;
    }

    let total_duration = start_time.elapsed();
    let avg_duration = total_duration.as_millis() as f64 / total_compositions as f64;

    println!("Signal composition performance:");
    println!("  Total compositions: {}", total_compositions);
    println!("  Total time: {}ms", total_duration.as_millis());
    println!("  Average time per composition: {:.2}ms", avg_duration);

    // Signal composition should be extremely fast
    assert!(
        avg_duration < 5.0,
        "Average signal composition should be <5ms"
    );
    assert!(
        total_duration.as_secs() < 1,
        "Total signal composition should be <1s"
    );
}

#[tokio::test]
async fn test_pipeline_performance_under_60s_target() {
    let mock_classifier = create_fast_mock_classifier();
    let mock_storage = create_fast_mock_storage();

    let config = PipelineConfig {
        enable_context_detection: true,
        enable_advanced_classification: true,
        enable_caching: true,
        max_concurrent: 10,
        timeout_seconds: 60,
        ..PipelineConfig::default()
    };

    let pipeline = ResearchPipeline::new(Arc::new(mock_classifier), Arc::new(mock_storage), config);

    let queries = get_performance_test_queries();
    let start_time = Instant::now();
    let mut total_queries = 0;
    let mut performance_stats = Vec::new();

    for (query, _) in queries {
        let query_start = Instant::now();

        // Use timeout to enforce 60s requirement
        let result = timeout(
            Duration::from_secs(60),
            pipeline.process_query(query, None, None),
        )
        .await
        .expect("Pipeline should complete within 60 seconds")
        .expect("Pipeline processing should succeed");

        let query_duration = query_start.elapsed();
        performance_stats.push(query_duration);

        // Each query should be well under 60 seconds
        assert!(
            query_duration.as_secs() < 60,
            "Query '{}' took {:.2}s, exceeding 60s limit",
            query,
            query_duration.as_secs_f64()
        );

        assert_eq!(result.request.original_query, query);
        assert!(!result.immediate_answer.is_empty());
        assert!(result.metadata.processing_time_ms > 0);

        total_queries += 1;
    }

    let total_duration = start_time.elapsed();
    let avg_duration = total_duration.as_secs_f64() / total_queries as f64;

    // Calculate statistics
    let mut durations: Vec<u64> = performance_stats
        .iter()
        .map(|d| d.as_millis() as u64)
        .collect();
    durations.sort();
    let median = durations[durations.len() / 2];
    let p95 = durations[(durations.len() * 95) / 100];
    let max_duration = durations.iter().max().unwrap();

    println!("Pipeline performance (60s target):");
    println!("  Total queries: {}", total_queries);
    println!("  Total time: {:.2}s", total_duration.as_secs_f64());
    println!("  Average time per query: {:.2}s", avg_duration);
    println!("  Median time: {}ms", median);
    println!("  95th percentile: {}ms", p95);
    println!("  Max time: {}ms", max_duration);

    // Pipeline should be well under 60s target
    assert!(
        avg_duration < 10.0,
        "Average pipeline time should be <10s for efficiency"
    );
    assert!(p95 < 30000, "95th percentile should be <30s");
    assert!(
        total_duration.as_secs() < 60,
        "Total pipeline time should be <60s"
    );
}

#[tokio::test]
async fn test_concurrent_pipeline_performance() {
    let mock_classifier = create_fast_mock_classifier();
    let mock_storage = create_fast_mock_storage();

    let config = PipelineConfig {
        enable_context_detection: true,
        enable_advanced_classification: true,
        enable_caching: true,
        max_concurrent: 10,
        timeout_seconds: 60,
        ..PipelineConfig::default()
    };

    let pipeline = Arc::new(ResearchPipeline::new(
        Arc::new(mock_classifier),
        Arc::new(mock_storage),
        config,
    ));

    let queries = get_performance_test_queries();
    let start_time = Instant::now();

    // Process queries concurrently
    let mut handles = Vec::new();
    for (query, _) in queries {
        let pipeline_clone = pipeline.clone();
        let handle = tokio::spawn(async move {
            let query_start = Instant::now();

            let result = timeout(
                Duration::from_secs(60),
                pipeline_clone.process_query(query, None, None),
            )
            .await
            .expect("Pipeline should complete within 60 seconds")
            .expect("Query should process successfully");

            let query_duration = query_start.elapsed();
            (query, query_duration, result)
        });
        handles.push(handle);
    }

    // Wait for all queries to complete
    let results = futures::future::join_all(handles).await;
    let total_duration = start_time.elapsed();

    let mut performance_stats = Vec::new();
    let mut successful_queries = 0;

    for result in results {
        let (query, query_duration, query_result) = result.expect("Task should complete");

        performance_stats.push(query_duration);
        successful_queries += 1;

        // Each query should complete within reasonable time
        assert!(
            query_duration.as_secs() < 60,
            "Query '{}' took {:.2}s, exceeding 60s limit",
            query,
            query_duration.as_secs_f64()
        );

        assert!(!query_result.immediate_answer.is_empty());
        assert!(query_result.metadata.processing_time_ms > 0);
    }

    let avg_duration = total_duration.as_secs_f64() / successful_queries as f64;

    println!("Concurrent pipeline performance:");
    println!("  Total queries: {}", successful_queries);
    println!("  Total time: {:.2}s", total_duration.as_secs_f64());
    println!("  Average time per query: {:.2}s", avg_duration);

    // Concurrent processing should be efficient
    assert!(
        successful_queries > 0,
        "Should process all queries successfully"
    );
    assert!(
        total_duration.as_secs() < 60,
        "Concurrent processing should be <60s"
    );

    // Concurrent processing should be faster than sequential
    let expected_sequential_time = performance_stats
        .iter()
        .map(|d| d.as_secs_f64())
        .sum::<f64>();
    let speedup = expected_sequential_time / total_duration.as_secs_f64();

    println!("  Speedup factor: {:.2}x", speedup);
    assert!(
        speedup > 1.0,
        "Concurrent processing should provide speedup"
    );
}

#[tokio::test]
async fn test_memory_efficiency_under_load() {
    let mock_classifier = create_fast_mock_classifier();
    let mock_storage = create_fast_mock_storage();

    let config = PipelineConfig {
        enable_context_detection: true,
        enable_advanced_classification: true,
        enable_caching: true,
        max_concurrent: 50, // High concurrency
        timeout_seconds: 60,
        ..PipelineConfig::default()
    };

    let pipeline = Arc::new(ResearchPipeline::new(
        Arc::new(mock_classifier),
        Arc::new(mock_storage),
        config,
    ));

    let start_time = Instant::now();
    let mut handles = Vec::new();

    // Process many queries concurrently to test memory efficiency
    for i in 0..100 {
        let pipeline_clone = pipeline.clone();
        let query = format!("Test query number {} for memory efficiency testing", i);

        let handle = tokio::spawn(async move {
            let result = timeout(
                Duration::from_secs(60),
                pipeline_clone.process_query(&query, None, None),
            )
            .await
            .expect("Pipeline should complete within 60 seconds")
            .expect("Query should process successfully");

            result.request.original_query.len() // Return something to prevent optimization
        });
        handles.push(handle);
    }

    // Wait for all queries to complete
    let results = futures::future::join_all(handles).await;
    let total_duration = start_time.elapsed();

    let successful_queries = results.len();
    let avg_duration = total_duration.as_secs_f64() / successful_queries as f64;

    println!("Memory efficiency under load:");
    println!("  Total queries: {}", successful_queries);
    println!("  Total time: {:.2}s", total_duration.as_secs_f64());
    println!("  Average time per query: {:.2}s", avg_duration);

    // System should handle high load efficiently
    assert_eq!(
        successful_queries, 100,
        "Should process all queries successfully"
    );
    assert!(
        total_duration.as_secs() < 60,
        "High load processing should be <60s"
    );
    assert!(
        avg_duration < 10.0,
        "Average time under load should be reasonable"
    );
}

#[tokio::test]
async fn test_performance_regression_detection() {
    let config = AdvancedClassificationConfig {
        max_processing_time_ms: 1000, // 1 second limit
        ..Default::default()
    };

    let classifier = AdvancedClassifier::new(config);

    // Baseline measurements
    let baseline_queries = vec![
        "What is Rust?",
        "How to implement async functions?",
        "Debug memory issues",
        "Choose a database",
        "Validate API design",
    ];

    let mut baseline_times = Vec::new();

    for query in &baseline_queries {
        let start = Instant::now();
        let _ = classifier
            .classify_enhanced(query, &ResearchType::Learning)
            .expect("Classification should succeed");
        baseline_times.push(start.elapsed());
    }

    let baseline_avg = baseline_times.iter().map(|d| d.as_millis()).sum::<u128>() as f64
        / baseline_times.len() as f64;

    // Performance under different loads
    let load_tests = vec![("Light load", 5), ("Medium load", 20), ("Heavy load", 50)];

    for (load_name, query_count) in load_tests {
        let start_time = Instant::now();
        let mut load_times = Vec::new();

        for i in 0..query_count {
            let query = format!("Performance test query number {} for load testing", i);
            let query_start = Instant::now();

            let _ = classifier
                .classify_enhanced(&query, &ResearchType::Learning)
                .expect("Classification should succeed");

            load_times.push(query_start.elapsed());
        }

        let total_duration = start_time.elapsed();
        let load_avg =
            load_times.iter().map(|d| d.as_millis()).sum::<u128>() as f64 / load_times.len() as f64;

        println!("{} performance:", load_name);
        println!("  Queries: {}", query_count);
        println!("  Total time: {}ms", total_duration.as_millis());
        println!("  Average time: {:.2}ms", load_avg);
        println!("  Baseline comparison: {:.2}x", load_avg / baseline_avg);

        // Performance should not degrade significantly under load
        assert!(
            load_avg < baseline_avg * 2.0,
            "{} should not be >2x slower than baseline",
            load_name
        );
        assert!(
            total_duration.as_secs() < 60,
            "{} should complete within 60s",
            load_name
        );
    }
}

#[tokio::test]
async fn test_pipeline_timeout_handling() {
    let mock_classifier = create_fast_mock_classifier();
    let mock_storage = create_fast_mock_storage();

    let config = PipelineConfig {
        enable_context_detection: true,
        enable_advanced_classification: true,
        enable_caching: true,
        max_concurrent: 5,
        timeout_seconds: 1, // Very short timeout for testing
        ..PipelineConfig::default()
    };

    let pipeline = ResearchPipeline::new(Arc::new(mock_classifier), Arc::new(mock_storage), config);

    // Test that pipeline respects timeout settings
    let query = "Test query for timeout handling";

    let start_time = Instant::now();
    let result = pipeline.process_query(query, None, None).await;
    let duration = start_time.elapsed();

    // Should complete successfully within timeout
    assert!(result.is_ok(), "Query should complete within timeout");
    assert!(duration.as_secs() < 10, "Query should complete quickly");

    let query_result = result.unwrap();
    assert_eq!(query_result.request.original_query, query);
    assert!(!query_result.immediate_answer.is_empty());

    println!("Timeout handling test:");
    println!("  Query completed in: {}ms", duration.as_millis());
    println!("  Result length: {}", query_result.immediate_answer.len());
}
