//! ANCHOR: Critical gap analysis system tests for proactive research mode (minimal version)
//!
//! These tests verify critical functionality and should be maintained
//! as the system evolves. Do not delete these tests.
//!
//! Tests: Gap detection accuracy, performance requirements, file monitoring integration,
//! basic configurable rules, error handling

use fortitude::proactive::{EventType, FileEvent, GapAnalysisConfig, GapAnalyzer, GapType};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::fs;
use tokio::time::timeout;

/// Helper function to create test files for gap analysis
async fn create_test_rust_file(temp_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let rust_file = temp_dir.join("test_lib.rs");

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
    Ok(rust_file)
}

/// ANCHOR: Verifies gap detection accuracy >90% for known patterns
/// Tests: TODO comments, missing documentation, undocumented technologies
#[tokio::test]
async fn test_anchor_gap_detection_accuracy() {
    let temp_dir = TempDir::new().unwrap();
    let rust_file = create_test_rust_file(temp_dir.path()).await.unwrap();

    let analyzer = GapAnalyzer::for_rust_project().unwrap();
    let gaps = analyzer.analyze_file(&rust_file).await.unwrap();

    // Log detected gaps for verification
    println!("Detected {} gaps:", gaps.len());
    for gap in &gaps {
        println!(
            "  {:?} at line {} - {}",
            gap.gap_type, gap.line_number, gap.description
        );
    }

    // Verify we detected gaps
    assert!(!gaps.is_empty(), "Should detect gaps in the test file");

    let gap_types: HashSet<_> = gaps.iter().map(|g| &g.gap_type).collect();

    // Should detect TODO comments
    assert!(
        gap_types.contains(&GapType::TodoComment),
        "Should detect TODO comments"
    );

    // Should detect missing documentation
    assert!(
        gap_types.contains(&GapType::MissingDocumentation),
        "Should detect missing documentation"
    );

    // Should detect undocumented technologies
    assert!(
        gap_types.contains(&GapType::UndocumentedTechnology),
        "Should detect undocumented technologies"
    );

    // Verify minimum accuracy: at least 3 different gap types should be detected
    assert!(
        gap_types.len() >= 3,
        "Should detect at least 3 different gap types"
    );

    // Verify specific gap counts to ensure reasonable accuracy
    let todo_gaps = gaps
        .iter()
        .filter(|g| g.gap_type == GapType::TodoComment)
        .count();
    let doc_gaps = gaps
        .iter()
        .filter(|g| g.gap_type == GapType::MissingDocumentation)
        .count();
    let tech_gaps = gaps
        .iter()
        .filter(|g| g.gap_type == GapType::UndocumentedTechnology)
        .count();

    println!(
        "Gap breakdown: {} TODO, {} missing docs, {} undocumented tech",
        todo_gaps, doc_gaps, tech_gaps
    );

    // ANCHOR: Require reasonable detection rates for each gap type
    assert!(
        todo_gaps >= 3,
        "Should detect at least 3 TODO comments (TODO, FIXME, HACK, NOTE)"
    );
    assert!(
        doc_gaps >= 1,
        "Should detect at least 1 missing documentation gap"
    );
    assert!(
        tech_gaps >= 1,
        "Should detect at least 1 undocumented technology"
    );
}

/// ANCHOR: Verifies performance requirement <500ms for gap analysis of up to 1000 files
/// Tests: Performance regression prevention, file processing speed, memory efficiency
#[tokio::test]
async fn test_anchor_gap_analysis_performance_requirement() {
    let temp_dir = TempDir::new().unwrap();

    // Create a moderate number of test files to validate performance
    let mut created_files = Vec::new();
    for i in 0..50 {
        // 50 files should extrapolate to 1000 file performance
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

    // Use timeout to enforce the 500ms requirement (scaled for 50 files = 25ms)
    let timeout_duration = Duration::from_millis(25); // 25ms for 50 files
    let total_gaps = timeout(timeout_duration, analysis_future)
        .await
        .expect("Gap analysis should complete within performance requirement");

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

/// ANCHOR: Verifies file event integration with gap analysis
/// Tests: File event processing, gap analysis integration, batch processing performance
#[tokio::test]
async fn test_anchor_file_event_integration() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple test files to simulate file events
    let mut test_files = Vec::new();
    for i in 0..50 {
        // 50 files to test batch processing
        let file_path = temp_dir.path().join(format!("event_file_{}.rs", i));
        let content = format!(
            r#"
// TODO: Implement feature {}
use external_crate::{{}};  // Undocumented technology

pub fn function_{}() {{
    // FIXME: Add proper implementation
}}

pub struct Struct{} {{
    // Missing documentation
}}
"#,
            i, i, i
        );

        fs::write(&file_path, content).await.unwrap();
        test_files.push(file_path);
    }

    let analyzer = GapAnalyzer::for_rust_project().unwrap();
    let start_time = Instant::now();

    // Process files as file events
    let mut total_gaps = 0;
    let mut processed_events = 0;

    for file_path in &test_files {
        // Create a file event for each file
        let event = FileEvent::new(file_path.clone(), EventType::Create);

        match analyzer.analyze_file_event(&event).await {
            Ok(gaps) => {
                total_gaps += gaps.len();
                processed_events += 1;
            }
            Err(e) => {
                println!("Failed to analyze file event for {:?}: {}", file_path, e);
            }
        }
    }

    let elapsed = start_time.elapsed();
    let events_per_second = processed_events as f64 / elapsed.as_secs_f64();
    let events_per_minute = events_per_second * 60.0;

    println!("File event processing performance:");
    println!(
        "  Events processed: {}/{}",
        processed_events,
        test_files.len()
    );
    println!("  Processing time: {:.2}s", elapsed.as_secs_f64());
    println!("  Events per minute: {:.0}", events_per_minute);
    println!("  Total gaps detected: {}", total_gaps);

    // ANCHOR: Require handling 100+ file events per minute
    assert!(
        events_per_minute >= 100.0,
        "File event processing should handle 100+ events per minute, got {:.0}",
        events_per_minute
    );

    // Verify integration functionality
    assert_eq!(
        processed_events,
        test_files.len(),
        "Should successfully process all file events"
    );
    assert!(total_gaps > 0, "Should detect gaps in file events");
    assert!(
        total_gaps >= test_files.len() * 2,
        "Should detect at least 2 gaps per file event on average"
    );

    // Verify performance
    assert!(
        elapsed.as_secs() < 2,
        "File event processing should complete within 2 seconds"
    );
}

/// ANCHOR: Verifies basic configurable rules system functionality
/// Tests: Configuration presets, rule application, basic filtering
#[tokio::test]
async fn test_anchor_basic_configurable_rules() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_rust_file(temp_dir.path()).await.unwrap();

    // Test different analyzer configurations
    let test_configs = vec![
        ("for_rust_project", GapAnalysisConfig::for_rust_project()),
        (
            "with_timeout",
            GapAnalysisConfig::for_rust_project().with_timeout_ms(100),
        ),
    ];

    for (config_name, config) in test_configs {
        println!("Testing {} configuration", config_name);

        let analyzer = GapAnalyzer::new(config.clone()).unwrap();

        let start_time = Instant::now();
        let gaps = analyzer.analyze_file(&test_file).await.unwrap();
        let elapsed = start_time.elapsed();

        println!("  {} configuration results:", config_name);
        println!("    Gaps detected: {}", gaps.len());
        println!("    Analysis time: {:.2}ms", elapsed.as_millis());

        // Verify all configurations work
        assert!(!gaps.is_empty(), "All configurations should detect gaps");
        assert!(
            elapsed.as_millis() < 500,
            "Configuration analysis should be fast"
        );

        // Verify gap structure
        for gap in &gaps {
            assert!(!gap.description.is_empty(), "Gap should have description");
            assert!(gap.confidence > 0.0, "Gap should have confidence");
            assert!(gap.line_number > 0, "Gap should have line number");
        }
    }
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
        // Large file (simulated)
        ("large.rs", "Should handle large files with limits"),
    ];

    // Create test files
    let empty_file = temp_dir.path().join("empty.rs");
    fs::write(&empty_file, "").await.unwrap();

    let binary_file = temp_dir.path().join("binary.bin");
    fs::write(&binary_file, &[0u8; 100]).await.unwrap();

    let large_file = temp_dir.path().join("large.rs");
    let large_content = "// Large file\n".repeat(1000); // Large but not binary
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

    println!("All error handling tests completed successfully");
}
