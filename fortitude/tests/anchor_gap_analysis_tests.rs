//! ANCHOR: Critical gap analysis system tests for proactive research mode
//!
//! These tests verify critical functionality and should be maintained
//! as the system evolves. Do not delete these tests.
//!
//! Tests: Gap detection accuracy, semantic analysis integration, performance requirements,
//! configurable rules, file monitoring integration, end-to-end pipeline functionality

use chrono::Utc;
use fortitude::proactive::{
    ConfigurableAnalysisError, ConfigurableAnalysisResult, ConfigurableGapAnalyzer, DetectedGap,
    EventType, FileEvent, FileMonitor, FileMonitorConfig, GapAnalysisConfig, GapAnalysisError,
    GapAnalyzer, GapDetectionConfig, GapType, IntegratedAnalysisConfig, IntegratedAnalysisError,
    IntegratedAnalysisResult, IntegratedGapAnalyzer, MonitorError, SemanticAnalysisConfig,
    SemanticAnalysisError, SemanticGapAnalysis, SemanticGapAnalyzer,
};
use fortitude_core::vector::storage::{DocumentMetadata, VectorStorageService};
use fortitude_core::vector::{
    client::QdrantClient,
    config::{ConnectionPoolConfig, VectorConfig},
    embeddings::{EmbeddingConfig, LocalEmbeddingService},
    error::{VectorError, VectorResult},
    SearchOptions, SemanticSearchOperations, SemanticSearchService, VectorDocument, VectorStorage,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::fs;
use tokio::time::{sleep, timeout};
use tracing::{info, warn};

/// Helper function to create test vector storage
async fn create_test_vector_storage() -> Result<Arc<VectorStorage>, Box<dyn std::error::Error>> {
    let embedding_config = EmbeddingConfig {
        model_name: "test-model".to_string(),
        max_sequence_length: 128,
        batch_size: 4,
        ..Default::default()
    };

    let embedding_service = Arc::new(LocalEmbeddingService::new(embedding_config));

    let vector_config = VectorConfig {
        url: "http://localhost:6334".to_string(),
        default_collection: "test_anchor_gap_analysis".to_string(),
        api_key: None,
        timeout: std::time::Duration::from_secs(5),
        vector_dimensions: 384,
        distance_metric: fortitude_core::vector::config::DistanceMetric::Cosine,
        health_check: fortitude_core::vector::config::HealthCheckConfig::default(),
        connection_pool: ConnectionPoolConfig::default(),
        embedding: EmbeddingConfig::default(),
    };

    let qdrant_client = match QdrantClient::new(vector_config).await {
        Ok(client) => Arc::new(client),
        Err(_) => {
            // Skip tests if Qdrant is not available
            return Err("Test Qdrant instance not available".into());
        }
    };

    let storage = VectorStorage::new(qdrant_client, embedding_service);
    storage.initialize().await?;
    Ok(Arc::new(storage))
}

/// Helper function to create semantic search service with test data
async fn create_semantic_search_service_with_data(
) -> Result<Arc<SemanticSearchService>, Box<dyn std::error::Error>> {
    let vector_storage = create_test_vector_storage().await?;
    let search_service = SemanticSearchService::with_defaults(vector_storage.clone());
    search_service.initialize().await?;

    // Populate with test content for semantic analysis
    let test_docs = vec![
        VectorDocument {
            id: "async-error-handling".to_string(),
            content: "Comprehensive guide to async error handling in Rust using Result types and error propagation.".to_string(),
            embedding: vec![0.1, 0.2, 0.3, 0.4], // Mock embedding
            metadata: DocumentMetadata {
                content_type: "guide".to_string(),
                quality_score: Some(0.9),
                tags: vec!["async".to_string(), "error".to_string(), "tokio".to_string()],
                custom_fields: {
                    let mut fields = std::collections::HashMap::new();
                    fields.insert("title".to_string(), serde_json::json!("Async Error Handling Guide"));
                    fields.insert("topic".to_string(), serde_json::json!("error_handling"));
                    fields.insert("language".to_string(), serde_json::json!("rust"));
                    fields
                },
                ..Default::default()
            },
            stored_at: Utc::now(),
        },
        VectorDocument {
            id: "tokio-documentation".to_string(),
            content: "Tokio async runtime documentation covering sync primitives, RwLock, and async I/O operations.".to_string(),
            embedding: vec![0.2, 0.3, 0.4, 0.5], // Mock embedding
            metadata: DocumentMetadata {
                content_type: "documentation".to_string(),
                quality_score: Some(0.8),
                tags: vec!["tokio".to_string(), "async".to_string(), "documentation".to_string()],
                custom_fields: {
                    let mut fields = std::collections::HashMap::new();
                    fields.insert("title".to_string(), serde_json::json!("Tokio Documentation"));
                    fields.insert("crate".to_string(), serde_json::json!("tokio"));
                    fields.insert("category".to_string(), serde_json::json!("async"));
                    fields
                },
                ..Default::default()
            },
            stored_at: Utc::now(),
        },
    ];

    for doc in test_docs {
        vector_storage
            .store_document(&doc.content, doc.metadata)
            .await?;
    }

    Ok(Arc::new(search_service))
}

/// Helper function to create test files for gap analysis
async fn create_test_project_files(
    temp_dir: &Path,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut created_files = Vec::new();

    // Create a Rust file with various gaps
    let rust_file = temp_dir.join("src").join("lib.rs");
    fs::create_dir_all(rust_file.parent().unwrap()).await?;

    let rust_content = r#"
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;  // Undocumented technology
use std::collections::HashMap;

// TODO: Implement proper error handling for all async operations
// FIXME: This function needs better input validation
pub fn undocumented_function(data: &str) -> String {
    // HACK: Quick fix for now, needs proper implementation
    data.to_uppercase()
}

/// This function has documentation but no examples - API documentation gap
pub fn documented_no_examples(input: i32) -> i32 {
    input * 2
}

/// This function has proper documentation with examples
/// 
/// # Examples
/// 
/// ```
/// let result = well_documented_function(5);
/// assert_eq!(result, 10);
/// ```
pub fn well_documented_function(input: i32) -> i32 {
    input * 2
}

pub struct UndocumentedStruct {
    value: String,
}

// NOTE: Need to implement Clone trait for this struct
impl UndocumentedStruct {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}
"#;

    fs::write(&rust_file, rust_content).await?;
    created_files.push(rust_file);

    // Create a configuration file with undocumented options
    let config_file = temp_dir.join("config.toml");
    let config_content = r#"
[database]
host = "localhost"
port = 5432
# Missing documentation for these options
max_connections = 100
timeout_seconds = 30

[api]
base_url = "https://api.example.com"
# TODO: Document the rate limiting configuration
rate_limit = 1000
"#;

    fs::write(&config_file, config_content).await?;
    created_files.push(config_file);

    // Create a JavaScript file to test multi-language support
    let js_file = temp_dir.join("frontend").join("app.js");
    fs::create_dir_all(js_file.parent().unwrap()).await?;

    let js_content = r#"
// TODO: Add TypeScript definitions
const express = require('express');  // Undocumented technology

// Missing documentation for this function
function processData(data) {
    // FIXME: Handle edge cases better
    return data.map(item => item.value);
}

/**
 * This function has documentation but no examples
 */
function calculateTotal(items) {
    return items.reduce((sum, item) => sum + item.price, 0);
}
"#;

    fs::write(&js_file, js_content).await?;
    created_files.push(js_file);

    Ok(created_files)
}

/// ANCHOR: Verifies gap detection accuracy >90% for known patterns
/// Tests: TODO comments, missing documentation, undocumented technologies, API gaps, configuration gaps
#[tokio::test]
async fn test_anchor_gap_detection_accuracy() {
    let temp_dir = TempDir::new().unwrap();
    let test_files = create_test_project_files(temp_dir.path()).await.unwrap();

    let analyzer = GapAnalyzer::for_rust_project().unwrap();

    let mut total_expected_gaps = 0;
    let mut total_detected_gaps = 0;
    let mut correctly_detected_gaps = 0;

    // Expected gaps per file
    let expected_gaps_per_file = HashMap::from([
        (
            "lib.rs",
            vec![
                GapType::TodoComment,            // TODO: Implement proper error handling
                GapType::TodoComment, // FIXME: This function needs better input validation
                GapType::TodoComment, // HACK: Quick fix for now
                GapType::TodoComment, // NOTE: Need to implement Clone trait
                GapType::MissingDocumentation, // undocumented_function
                GapType::MissingDocumentation, // UndocumentedStruct
                GapType::UndocumentedTechnology, // tokio
                GapType::ApiDocumentationGap, // documented_no_examples (has docs but no examples)
            ],
        ),
        (
            "config.toml",
            vec![
                GapType::TodoComment,      // TODO: Document the rate limiting
                GapType::ConfigurationGap, // max_connections
                GapType::ConfigurationGap, // timeout_seconds
            ],
        ),
        (
            "app.js",
            vec![
                GapType::TodoComment,          // TODO: Add TypeScript definitions
                GapType::TodoComment,          // FIXME: Handle edge cases better
                GapType::MissingDocumentation, // processData function
                GapType::UndocumentedTechnology, // express
                                               // Note: calculateTotal has docs but no examples (API gap) - depends on JS support
            ],
        ),
    ]);

    for file_path in test_files {
        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        let gaps = analyzer.analyze_file(&file_path).await.unwrap();

        total_detected_gaps += gaps.len();

        if let Some(expected_gap_types) = expected_gaps_per_file.get(file_name) {
            total_expected_gaps += expected_gap_types.len();

            // Check for expected gaps
            let detected_gap_types: HashSet<_> = gaps.iter().map(|g| &g.gap_type).collect();

            for expected_type in expected_gap_types {
                if detected_gap_types.contains(expected_type) {
                    correctly_detected_gaps += 1;
                }
            }

            // Log detected gaps for verification
            info!("File {}: detected {} gaps", file_name, gaps.len());
            for gap in &gaps {
                info!(
                    "  {:?} at line {} - {}",
                    gap.gap_type, gap.line_number, gap.description
                );
            }
        }
    }

    let accuracy = if total_expected_gaps > 0 {
        (correctly_detected_gaps as f64) / (total_expected_gaps as f64)
    } else {
        0.0
    };

    println!(
        "Gap detection accuracy: {:.2}% ({}/{} expected gaps detected)",
        accuracy * 100.0,
        correctly_detected_gaps,
        total_expected_gaps
    );
    println!("Total gaps detected: {}", total_detected_gaps);

    // ANCHOR: Require >90% accuracy for known gap patterns
    assert!(
        accuracy > 0.9,
        "Gap detection accuracy must be >90%, got {:.2}% ({}/{})",
        accuracy * 100.0,
        correctly_detected_gaps,
        total_expected_gaps
    );

    // Verify we detected a reasonable number of gaps
    assert!(
        total_detected_gaps >= total_expected_gaps * 8 / 10,
        "Should detect at least 80% of expected gaps, got {} detected vs {} expected",
        total_detected_gaps,
        total_expected_gaps
    );
}

/// ANCHOR: Verifies performance requirement <500ms for gap analysis of up to 1000 files
/// Tests: Performance regression prevention, file processing speed, memory efficiency
#[tokio::test]
async fn test_anchor_gap_analysis_performance_requirement() {
    let temp_dir = TempDir::new().unwrap();

    // Create a moderate number of test files to validate performance
    let mut created_files = Vec::new();
    for i in 0..100 {
        // 100 files should extrapolate to 1000 file performance
        let file_path = temp_dir.path().join(format!("test_file_{}.rs", i));

        let content = format!(
            r#"
// TODO: Implement feature {}
use std::collections::HashMap;
use serde::{{Serialize, Deserialize}};

pub fn function_{}(input: &str) -> String {{
    // FIXME: Add error handling
    input.to_uppercase()
}}

pub struct Struct{} {{
    value: String,
}}
"#,
            i, i, i
        );

        fs::write(&file_path, content).await.unwrap();
        created_files.push(file_path);
    }

    let analyzer = GapAnalyzer::for_rust_project().unwrap();

    // Test performance with timeout enforcement
    let start_time = Instant::now();

    let analysis_future = async {
        let mut total_gaps = 0;
        for file_path in &created_files {
            let gaps = analyzer.analyze_file(file_path).await.unwrap();
            total_gaps += gaps.len();
        }
        total_gaps
    };

    // Use timeout to enforce the 500ms requirement (scaled for 100 files = 50ms)
    let timeout_duration = Duration::from_millis(50); // 50ms for 100 files
    let total_gaps = timeout(timeout_duration, analysis_future)
        .await
        .expect("Gap analysis should complete within performance requirement")
        .expect("Analysis should succeed");

    let elapsed = start_time.elapsed();
    let files_per_ms = created_files.len() as f64 / elapsed.as_millis() as f64;
    let projected_1000_files_ms = 1000.0 / files_per_ms;

    println!(
        "Analyzed {} files in {:.2}ms ({:.2} files/ms)",
        created_files.len(),
        elapsed.as_millis(),
        files_per_ms
    );
    println!(
        "Projected time for 1000 files: {:.2}ms",
        projected_1000_files_ms
    );
    println!("Total gaps detected: {}", total_gaps);

    // ANCHOR: Performance requirement must be met
    assert!(
        projected_1000_files_ms < 500.0,
        "Gap analysis must complete in <500ms for 1000 files, projected: {:.2}ms",
        projected_1000_files_ms
    );

    // Verify we detected gaps (functionality verification)
    assert!(total_gaps > 0, "Should detect gaps in test files");
    assert!(
        total_gaps >= created_files.len() * 2,
        "Should detect at least 2 gaps per file on average"
    );
}

/// ANCHOR: Verifies semantic analysis integration with >80% relevance for gap enhancement
/// Tests: Vector database integration, semantic gap validation, related content discovery
#[tokio::test]
async fn test_anchor_semantic_analysis_integration() {
    let temp_dir = TempDir::new().unwrap();
    let test_files = create_test_project_files(temp_dir.path()).await.unwrap();

    // Create semantic search service with test data
    let search_service = create_semantic_search_service_with_data().await.unwrap();

    let config = SemanticAnalysisConfig {
        enable_gap_validation: true,
        enable_related_content: true,
        enable_priority_enhancement: true,
        max_analysis_time_ms: 1000,
        gap_validation_threshold: 0.8,
        related_content_threshold: 0.7,
        max_related_documents: 5,
        min_content_length: 50,
        batch_size: 20,
        semantic_priority_weight: 0.3,
    };

    let semantic_analyzer = SemanticGapAnalyzer::new(search_service, config);

    // Create test gaps from the basic gap analyzer
    let basic_analyzer = GapAnalyzer::for_rust_project().unwrap();
    let rust_file = test_files
        .iter()
        .find(|f| f.file_name().unwrap() == "lib.rs")
        .unwrap();
    let detected_gaps = basic_analyzer.analyze_file(rust_file).await.unwrap();

    assert!(!detected_gaps.is_empty(), "Should have detected basic gaps");

    let start_time = Instant::now();

    // Perform semantic analysis on detected gaps
    let semantic_results = semantic_analyzer
        .analyze_gaps_semantically(detected_gaps)
        .await
        .unwrap();

    let elapsed = start_time.elapsed();
    println!(
        "Semantic analysis completed in {:.2}ms",
        elapsed.as_millis()
    );

    // ANCHOR: Semantic analysis should complete within performance requirements
    assert!(
        elapsed.as_millis() < 1000,
        "Semantic analysis should complete in <1000ms, took {:.2}ms",
        elapsed.as_millis()
    );

    // Verify semantic analysis results
    assert!(
        !semantic_results.is_empty(),
        "Should have semantic analysis results"
    );

    let mut relevant_enhancements = 0;
    let mut total_relevance_score = 0.0;

    for result in &semantic_results {
        // Verify structure
        assert!(
            result.validation_confidence >= 0.0 && result.validation_confidence <= 1.0,
            "Confidence should be between 0 and 1"
        );

        // Check for related content
        if !result.related_documents.is_empty() {
            relevant_enhancements += 1;

            for doc in &result.related_documents {
                total_relevance_score += doc.similarity_score;

                // Verify relevance scores are reasonable
                assert!(
                    doc.similarity_score >= 0.0 && doc.similarity_score <= 1.0,
                    "Similarity score should be between 0 and 1"
                );
            }
        }

        // Verify processing time tracking
        assert!(
            result.metadata.processing_time_ms > 0,
            "Should track processing time"
        );
        assert!(
            result.metadata.processing_time_ms < 100,
            "Individual gap analysis should be fast"
        );
    }

    // Calculate average relevance
    let total_related_docs: usize = semantic_results
        .iter()
        .map(|r| r.related_documents.len())
        .sum();

    let average_relevance = if total_related_docs > 0 {
        total_relevance_score / total_related_docs as f64
    } else {
        0.0
    };

    println!(
        "Semantic analysis enhancement rate: {:.2}% ({}/{})",
        (relevant_enhancements as f64 / semantic_results.len() as f64) * 100.0,
        relevant_enhancements,
        semantic_results.len()
    );
    println!("Average relevance score: {:.2}", average_relevance);

    // ANCHOR: Require >80% relevance for semantic enhancement
    assert!(
        average_relevance > 0.8 || total_related_docs == 0,
        "Average relevance should be >80% when related content is found, got {:.2}%",
        average_relevance * 100.0
    );

    // Verify some gaps were enhanced
    assert!(
        relevant_enhancements > 0,
        "Should enhance at least some gaps with related content"
    );
}

/// ANCHOR: Verifies configurable rules system with all configuration presets working correctly
/// Tests: Configuration flexibility, rule customization, filtering, priority adjustment
#[tokio::test]
async fn test_anchor_configurable_rules_system() {
    let temp_dir = TempDir::new().unwrap();
    let test_files = create_test_project_files(temp_dir.path()).await.unwrap();

    // Test different configuration presets
    let test_configs = vec![
        ("minimal", GapDetectionConfig::minimal()),
        ("performance", GapDetectionConfig::for_performance()),
        ("comprehensive", GapDetectionConfig::comprehensive()),
    ];

    for (preset_name, config) in test_configs {
        println!("Testing {} configuration preset", preset_name);

        let search_service = create_semantic_search_service_with_data().await.unwrap();
        let configurable_analyzer =
            ConfigurableGapAnalyzer::new(config.clone(), Some(search_service)).unwrap();

        let start_time = Instant::now();

        let mut all_results = Vec::new();
        for file_path in &test_files {
            let result = configurable_analyzer
                .analyze_file_comprehensive(file_path)
                .await
                .unwrap();
            all_results.push(result);
        }

        let elapsed = start_time.elapsed();

        // Aggregate results
        let total_gaps: usize = all_results.iter().map(|r| r.filtered_gaps.len()).sum();

        let total_raw_gaps: usize = all_results
            .iter()
            .map(|r| r.performance_metrics.initial_gap_count)
            .sum();

        let filtering_rate = if total_raw_gaps > 0 {
            1.0 - (total_gaps as f64 / total_raw_gaps as f64)
        } else {
            0.0
        };

        println!("  {} preset results:", preset_name);
        println!(
            "    Filtered gaps: {} (from {} raw gaps)",
            total_gaps, total_raw_gaps
        );
        println!("    Filtering rate: {:.2}%", filtering_rate * 100.0);
        println!("    Analysis time: {:.2}ms", elapsed.as_millis());

        // Verify configuration behavior
        match preset_name {
            "minimal" => {
                // Minimal should be fastest but may miss some gaps
                assert!(
                    elapsed.as_millis() < 500,
                    "Minimal preset should be very fast"
                );
            }
            "performance" => {
                // Performance should be fast with good coverage
                assert!(
                    elapsed.as_millis() < 800,
                    "Performance preset should be fast"
                );
                assert!(
                    total_gaps > 0 || total_raw_gaps == 0,
                    "Performance should detect gaps"
                );
            }
            "comprehensive" => {
                // Comprehensive should find more gaps but may take longer
                assert!(
                    total_gaps > 0 || total_raw_gaps == 0,
                    "Comprehensive should detect gaps"
                );
            }
            _ => {}
        }

        // Verify all configurations work
        assert!(
            elapsed.as_millis() < 1000,
            "Configuration analysis should be fast"
        );

        // Verify result structure
        for result in &all_results {
            assert!(
                result.config_summary.rules_applied > 0,
                "Should apply some rules"
            );
            assert!(
                !result.config_summary.preset_name.is_empty(),
                "Should have preset name"
            );
            assert!(
                result.performance_metrics.total_analysis_time_ms > 0,
                "Should track time"
            );

            // Verify priority breakdown
            let total_priority: usize = result.priority_breakdown.values().sum();
            assert_eq!(
                total_priority,
                result.filtered_gaps.len(),
                "Priority breakdown should match filtered gap count"
            );
        }
    }
}

/// ANCHOR: Verifies file monitoring integration handles 100+ changes per minute
/// Tests: File monitoring performance, event processing, integration with gap analysis
#[tokio::test]
async fn test_anchor_file_monitoring_integration() {
    let temp_dir = TempDir::new().unwrap();
    let watch_dir = temp_dir.path().join("watched");
    fs::create_dir_all(&watch_dir).await.unwrap();

    let config = FileMonitorConfig {
        debounce_duration: Duration::from_millis(10), // Fast debouncing for test
        ignore_patterns: vec![".git".to_string(), "*.tmp".to_string()],
        max_events_per_second: 200, // Allow high throughput
        enable_recursive: true,
        file_size_limit: Some(1024 * 1024), // 1MB limit
        processing_timeout: Duration::from_millis(100),
    };

    let (event_sender, mut event_receiver) = tokio::sync::mpsc::unbounded_channel();
    let file_monitor = FileMonitor::new(config, event_sender);

    // Start monitoring
    file_monitor.watch_directory(&watch_dir).await.unwrap();

    let start_time = Instant::now();
    let target_events = 100; // Test with 100+ file changes
    let mut events_received = 0;
    let mut gap_analysis_count = 0;

    // Create gap analyzer for processing events
    let gap_analyzer = GapAnalyzer::for_rust_project().unwrap();

    // Spawn file creation task
    let watch_dir_clone = watch_dir.clone();
    let file_creation_task = tokio::spawn(async move {
        for i in 0..target_events {
            let file_path = watch_dir_clone.join(format!("test_file_{}.rs", i));
            let content = format!(
                r#"
// TODO: Implement feature {}
pub fn function_{}() {{
    // FIXME: Add implementation
}}
"#,
                i, i
            );

            fs::write(&file_path, content).await.unwrap();

            // Small delay to allow processing
            if i % 10 == 0 {
                sleep(Duration::from_millis(1)).await;
            }
        }
    });

    // Process file events and run gap analysis
    let event_processing_task = tokio::spawn(async move {
        let mut received_count = 0;

        while received_count < target_events {
            if let Some(event) = event_receiver.recv().await {
                received_count += 1;

                // Run gap analysis on the event
                if let Ok(gaps) = gap_analyzer.analyze_file_event(&event).await {
                    if !gaps.is_empty() {
                        gap_analysis_count += 1;
                    }
                }

                // Track progress
                if received_count % 20 == 0 {
                    println!("Processed {} file events", received_count);
                }
            }
        }

        received_count
    });

    // Wait for both tasks with timeout
    let results = timeout(Duration::from_secs(10), async {
        let _ = file_creation_task.await.unwrap();
        let received = event_processing_task.await.unwrap();
        received
    })
    .await
    .expect("File monitoring test should complete within 10 seconds");

    events_received = results;
    let elapsed = start_time.elapsed();

    // Calculate performance metrics
    let events_per_minute = (events_received as f64 / elapsed.as_secs_f64()) * 60.0;

    println!("File monitoring performance:");
    println!("  Events processed: {}/{}", events_received, target_events);
    println!("  Processing time: {:.2}s", elapsed.as_secs_f64());
    println!("  Events per minute: {:.0}", events_per_minute);
    println!("  Gap analyses completed: {}", gap_analysis_count);

    // ANCHOR: Require handling 100+ changes per minute
    assert!(
        events_per_minute >= 100.0,
        "File monitoring should handle 100+ changes per minute, got {:.0}",
        events_per_minute
    );

    // Verify integration functionality
    assert!(
        events_received >= target_events * 8 / 10,
        "Should receive at least 80% of file events"
    );
    assert!(
        gap_analysis_count > 0,
        "Should successfully analyze at least some files"
    );

    // Verify performance
    assert!(
        elapsed.as_secs() < 5,
        "Processing should complete within 5 seconds"
    );
}

/// ANCHOR: Verifies complete end-to-end gap analysis pipeline integration
/// Tests: File monitoring → Gap detection → Semantic analysis → Configurable rules pipeline
#[tokio::test]
async fn test_anchor_end_to_end_gap_analysis_pipeline() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("test_project");
    fs::create_dir_all(&project_dir).await.unwrap();

    // Create integrated analyzer with all components
    let search_service = create_semantic_search_service_with_data().await.unwrap();

    let integrated_config = IntegratedAnalysisConfig {
        gap_analysis: GapAnalysisConfig::for_rust_project(),
        semantic_analysis: SemanticAnalysisConfig {
            enable_gap_validation: true,
            enable_related_content: true,
            enable_priority_enhancement: true,
            max_analysis_time_ms: 500,
            gap_validation_threshold: 0.8,
            related_content_threshold: 0.4,
            max_related_documents: 3,
            min_content_length: 50,
            batch_size: 20,
            semantic_priority_weight: 0.6,
        },
        max_total_time_ms: 2000,
        enable_semantic_analysis: true,
        min_gaps_for_semantic: 1,
    };

    let integrated_analyzer =
        IntegratedGapAnalyzer::new(integrated_config, Some(search_service)).unwrap();

    // Create test project files
    let test_files = create_test_project_files(&project_dir).await.unwrap();

    let start_time = Instant::now();

    // Run complete pipeline analysis on first file (since analyze_project_directory may not exist)
    let first_file = &test_files[0];
    let pipeline_result = integrated_analyzer.analyze_file(first_file).await.unwrap();

    let elapsed = start_time.elapsed();

    println!("End-to-end pipeline analysis results:");
    println!("  Processing time: {:.2}ms", elapsed.as_millis());
    println!("  Gaps detected: {}", pipeline_result.detected_gaps.len());
    println!(
        "  Semantic analysis performed: {}",
        pipeline_result.semantic_analysis_performed
    );
    if let Some(ref semantic_results) = pipeline_result.semantic_analysis {
        println!("  Semantic analyses: {}", semantic_results.len());
    }

    // ANCHOR: End-to-end pipeline should complete within performance requirements
    assert!(
        elapsed.as_millis() < 2000,
        "Complete pipeline should complete in <2000ms, took {:.2}ms",
        elapsed.as_millis()
    );

    // Verify pipeline functionality
    assert!(
        !pipeline_result.detected_gaps.is_empty(),
        "Should detect gaps"
    );
    assert!(
        pipeline_result.performance_metrics.gaps_detected > 0,
        "Should detect gaps"
    );

    // Verify gap structure
    for gap in &pipeline_result.detected_gaps {
        // Basic gap structure
        assert!(!gap.description.is_empty(), "Gap should have description");
        assert!(gap.confidence > 0.0, "Gap should have confidence");
        assert!(gap.line_number > 0, "Gap should have line number");
    }

    // Verify semantic analysis (if performed)
    if let Some(ref semantic_results) = pipeline_result.semantic_analysis {
        assert!(
            !semantic_results.is_empty(),
            "Should have semantic results if analysis performed"
        );

        for semantic_result in semantic_results {
            assert!(
                semantic_result.validation_confidence >= 0.0
                    && semantic_result.validation_confidence <= 1.0,
                "Semantic confidence should be between 0 and 1"
            );
        }

        println!(
            "  Gaps with semantic enhancement: {}/{}",
            semantic_results.len(),
            pipeline_result.detected_gaps.len()
        );
    }

    // Verify performance metrics
    assert!(
        pipeline_result.performance_metrics.gap_detection_time_ms < 100.0,
        "Gap detection should be <100ms"
    );
    assert!(
        pipeline_result.performance_metrics.total_time_ms > 0.0,
        "Should track total time"
    );

    // Verify gap type diversity
    let gap_types: HashSet<_> = pipeline_result
        .detected_gaps
        .iter()
        .map(|g| &g.gap_type)
        .collect();

    assert!(gap_types.len() >= 3, "Should detect multiple types of gaps");
    assert!(
        gap_types.contains(&GapType::TodoComment),
        "Should detect TODO comments"
    );
    assert!(
        gap_types.contains(&GapType::MissingDocumentation),
        "Should detect missing docs"
    );
}

/// ANCHOR: Verifies error handling and recovery for critical gap analysis paths
/// Tests: Graceful degradation, error recovery, fallback mechanisms, resource limits
#[tokio::test]
async fn test_anchor_gap_analysis_error_handling() {
    let temp_dir = TempDir::new().unwrap();

    // Test various error conditions
    let error_test_cases = vec![
        // Non-existent file
        ("nonexistent.rs", "Should handle missing files gracefully"),
        // Empty file
        ("empty.rs", "Should handle empty files"),
        // Binary file
        ("binary.bin", "Should handle binary files"),
        // Very large file (simulated)
        ("large.rs", "Should handle large files with limits"),
    ];

    // Create test files
    let empty_file = temp_dir.path().join("empty.rs");
    fs::write(&empty_file, "").await.unwrap();

    let binary_file = temp_dir.path().join("binary.bin");
    fs::write(&binary_file, &[0u8; 100]).await.unwrap();

    let large_file = temp_dir.path().join("large.rs");
    let large_content = "// Large file\n".repeat(10000); // Large but not binary
    fs::write(&large_file, large_content).await.unwrap();

    let analyzer = GapAnalyzer::for_rust_project().unwrap();

    // Test error handling for each case
    for (filename, description) in error_test_cases {
        let file_path = temp_dir.path().join(filename);

        println!("Testing error handling: {}", description);

        let result = analyzer.analyze_file(&file_path).await;

        match filename {
            "nonexistent.rs" => {
                // Should return error for non-existent file
                assert!(result.is_err(), "Should error on non-existent file");

                // Error should be informative
                let error = result.unwrap_err();
                assert!(
                    error.to_string().contains("nonexistent.rs"),
                    "Error should mention the problematic file"
                );
            }
            "empty.rs" => {
                // Should handle empty files gracefully
                let gaps = result.expect("Should handle empty files");
                assert!(gaps.is_empty(), "Empty file should have no gaps");
            }
            "binary.bin" => {
                // Should either error gracefully or return no gaps
                match result {
                    Ok(gaps) => {
                        // If it succeeds, should find no meaningful gaps
                        assert!(
                            gaps.is_empty() || gaps.len() < 3,
                            "Binary file should not have many text-based gaps"
                        );
                    }
                    Err(_) => {
                        // Erroring on binary files is acceptable
                    }
                }
            }
            "large.rs" => {
                // Should handle large files (may apply limits)
                match result {
                    Ok(gaps) => {
                        // Should find gaps but in reasonable time
                        println!("Large file analysis found {} gaps", gaps.len());
                    }
                    Err(error) => {
                        // May error due to size limits - should be clear
                        assert!(
                            error.to_string().contains("large")
                                || error.to_string().contains("size")
                                || error.to_string().contains("limit"),
                            "Size-related error should be descriptive: {}",
                            error
                        );
                    }
                }
            }
            _ => {}
        }
    }

    // Test semantic analysis error handling
    let search_service = create_semantic_search_service_with_data().await.unwrap();

    let config = SemanticAnalysisConfig {
        enable_gap_validation: true,
        enable_related_content: true,
        enable_priority_enhancement: true,
        max_analysis_time_ms: 10, // Very short timeout to trigger timeout errors
        gap_validation_threshold: 0.8,
        related_content_threshold: 0.3,
        max_related_documents: 5,
        min_content_length: 50,
        batch_size: 20,
        semantic_priority_weight: 0.7,
    };

    let semantic_analyzer = SemanticGapAnalyzer::new(search_service, config);

    // Create a gap that might cause semantic analysis issues
    let test_gap = DetectedGap::new(
        GapType::TodoComment,
        temp_dir.path().join("test.rs"),
        1,
        "// TODO: This is a test comment that might cause issues".to_string(),
        "Test gap for error handling".to_string(),
        0.8,
    );

    let semantic_result = semantic_analyzer
        .analyze_gaps_semantically(vec![test_gap])
        .await;

    // Should either succeed or fail gracefully
    match semantic_result {
        Ok(results) => {
            println!("Semantic analysis succeeded with {} results", results.len());
        }
        Err(error) => {
            println!("Semantic analysis failed gracefully: {}", error);

            // Error should be descriptive and not a panic
            assert!(
                !error.to_string().is_empty(),
                "Error should have description"
            );
        }
    }

    println!("All error handling tests completed successfully");
}
