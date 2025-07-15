// ABOUTME: Tests for the file system monitoring functionality in proactive research mode
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::fs;
use tokio::time::timeout;

use fortitude::proactive::{FileEvent, FileMonitor, FileMonitorConfig, MonitorError};

#[tokio::test]
async fn test_file_monitor_creation() {
    // FAILING TEST: FileMonitor should be creatable with valid configuration
    let temp_dir = TempDir::new().unwrap();
    let config = FileMonitorConfig::for_rust_project()
        .with_debounce_ms(100)
        .with_max_queue_size(1000);

    let paths = vec![temp_dir.path().to_path_buf()];
    let result = FileMonitor::new(paths, config).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_file_monitor_detects_file_creation() {
    // FAILING TEST: FileMonitor should detect when new files are created
    let temp_dir = TempDir::new().unwrap();
    let config = FileMonitorConfig::for_rust_project().with_debounce_ms(50);

    let paths = vec![temp_dir.path().to_path_buf()];
    let mut monitor = FileMonitor::new(paths, config).await.unwrap();

    // Create a new file
    let file_path = temp_dir.path().join("test.rs");
    fs::write(&file_path, "// Test content").await.unwrap();

    // Should receive a file event within reasonable time
    let event = timeout(Duration::from_millis(500), monitor.next_event())
        .await
        .expect("Should receive event within timeout")
        .expect("Should receive valid event");

    assert_eq!(event.path, file_path);
    assert!(matches!(
        event.event_type,
        fortitude::proactive::EventType::Create
    ));
}

#[tokio::test]
async fn test_file_monitor_detects_file_modification() {
    // FAILING TEST: FileMonitor should detect when files are modified
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    fs::write(&file_path, "// Initial content").await.unwrap();

    let config = FileMonitorConfig::for_rust_project().with_debounce_ms(50);

    let paths = vec![temp_dir.path().to_path_buf()];
    let mut monitor = FileMonitor::new(paths, config).await.unwrap();

    // Wait a moment for initial setup
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Modify the file
    fs::write(&file_path, "// Modified content").await.unwrap();

    // Should receive a modification event
    let event = timeout(Duration::from_millis(500), monitor.next_event())
        .await
        .expect("Should receive event within timeout")
        .expect("Should receive valid event");

    assert_eq!(event.path, file_path);
    assert!(matches!(
        event.event_type,
        fortitude::proactive::EventType::Write
    ));
}

#[tokio::test]
async fn test_file_monitor_filters_excluded_patterns() {
    // FAILING TEST: FileMonitor should exclude files matching exclude patterns
    let temp_dir = TempDir::new().unwrap();
    let config = FileMonitorConfig::for_rust_project()
        .with_debounce_ms(50)
        .with_exclude_patterns(vec!["*.tmp".to_string(), "target/*".to_string()]);

    let paths = vec![temp_dir.path().to_path_buf()];
    let mut monitor = FileMonitor::new(paths, config).await.unwrap();

    // Create files that should be filtered out
    let tmp_file = temp_dir.path().join("test.tmp");
    fs::write(&tmp_file, "temp content").await.unwrap();

    let target_dir = temp_dir.path().join("target");
    fs::create_dir(&target_dir).await.unwrap();
    let target_file = target_dir.join("build.out");
    fs::write(&target_file, "build output").await.unwrap();

    // Create a file that should NOT be filtered
    let rs_file = temp_dir.path().join("test.rs");
    fs::write(&rs_file, "// Valid Rust file").await.unwrap();

    // Should only receive event for the .rs file
    let event = timeout(Duration::from_millis(500), monitor.next_event())
        .await
        .expect("Should receive event within timeout")
        .expect("Should receive valid event");

    assert_eq!(event.path, rs_file);
}

#[tokio::test]
async fn test_file_monitor_handles_high_frequency_changes() {
    // FAILING TEST: FileMonitor should handle rapid file changes without degradation
    let temp_dir = TempDir::new().unwrap();
    let config = FileMonitorConfig::for_rust_project()
        .with_debounce_ms(100)
        .with_max_queue_size(1000);

    let paths = vec![temp_dir.path().to_path_buf()];
    let mut monitor = FileMonitor::new(paths, config).await.unwrap();

    // Create many files rapidly
    let mut file_paths = Vec::new();
    for i in 0..50 {
        let file_path = temp_dir.path().join(format!("test_{}.rs", i));
        file_paths.push(file_path.clone());
        fs::write(&file_path, format!("// Test file {}", i))
            .await
            .unwrap();
    }

    // Should receive events for files (may be debounced)
    let mut received_events = 0;
    let start_time = std::time::Instant::now();

    while received_events < 10 && start_time.elapsed() < Duration::from_secs(2) {
        if let Ok(Some(_event)) = timeout(Duration::from_millis(100), monitor.next_event()).await {
            received_events += 1;
        }
    }

    assert!(
        received_events > 0,
        "Should receive at least some events from rapid file creation"
    );
    assert!(
        start_time.elapsed() < Duration::from_secs(1),
        "Should handle events within reasonable time"
    );
}

#[tokio::test]
async fn test_file_monitor_debouncing() {
    // FAILING TEST: FileMonitor should debounce rapid changes to the same file
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");

    let config = FileMonitorConfig::for_rust_project().with_debounce_ms(200); // Longer debounce for this test

    let paths = vec![temp_dir.path().to_path_buf()];
    let mut monitor = FileMonitor::new(paths, config).await.unwrap();

    // Rapidly modify the same file multiple times
    for i in 0..5 {
        fs::write(&file_path, format!("// Content {}", i))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await; // Rapid changes
    }

    // Should receive a debounced event (not 5 separate events)
    let event = timeout(Duration::from_millis(500), monitor.next_event())
        .await
        .expect("Should receive debounced event")
        .expect("Should receive valid event");

    assert_eq!(event.path, file_path);

    // Check that we don't get immediate additional events (they were debounced)
    // Wait a bit longer since the debounce window is 200ms
    let no_additional_event = timeout(Duration::from_millis(300), monitor.next_event()).await;
    assert!(
        no_additional_event.is_err(),
        "Should not receive additional events due to debouncing"
    );
}

#[tokio::test]
async fn test_file_monitor_shutdown() {
    // FAILING TEST: FileMonitor should shut down gracefully
    let temp_dir = TempDir::new().unwrap();
    let config = FileMonitorConfig::for_rust_project();

    let paths = vec![temp_dir.path().to_path_buf()];
    let monitor = FileMonitor::new(paths, config).await.unwrap();

    // Should shut down without hanging
    let shutdown_result = timeout(Duration::from_millis(500), monitor.shutdown()).await;
    assert!(
        shutdown_result.is_ok(),
        "Shutdown should complete within timeout"
    );
}

#[tokio::test]
async fn test_file_monitor_error_handling() {
    // FAILING TEST: FileMonitor should handle invalid paths gracefully
    let invalid_path = PathBuf::from("/this/path/does/not/exist");
    let config = FileMonitorConfig::for_rust_project();

    let paths = vec![invalid_path];
    let result = FileMonitor::new(paths, config).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        MonitorError::WatchPath { .. }
    ));
}

#[tokio::test]
async fn test_file_monitor_performance_requirements() {
    // FAILING TEST: FileMonitor should meet performance requirements from Sprint 008
    let temp_dir = TempDir::new().unwrap();
    let config = FileMonitorConfig::for_rust_project().with_debounce_ms(50);

    let paths = vec![temp_dir.path().to_path_buf()];
    let mut monitor = FileMonitor::new(paths, config).await.unwrap();

    let start_time = std::time::Instant::now();

    // Create 100 files to simulate the performance requirement
    for i in 0..100 {
        let file_path = temp_dir.path().join(format!("perf_test_{}.rs", i));
        fs::write(&file_path, format!("// Performance test file {}", i))
            .await
            .unwrap();

        // Simulate real-world timing - not all at once
        if i % 10 == 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    // Process events without degradation
    let mut processed_events = 0;
    let processing_start = std::time::Instant::now();

    while processed_events < 50 && processing_start.elapsed() < Duration::from_secs(5) {
        if let Ok(Some(_event)) = timeout(Duration::from_millis(50), monitor.next_event()).await {
            processed_events += 1;
        }
    }

    let total_time = start_time.elapsed();

    // Performance requirements from Sprint 008:
    // - Handle 100+ file changes per minute without degradation
    // - Gap analysis should complete <500ms for projects up to 1000 files
    assert!(
        total_time < Duration::from_secs(30),
        "Should handle 100 files within 30 seconds"
    );
    assert!(
        processed_events > 10,
        "Should process reasonable number of events"
    );
}

mod config_tests {
    use super::*;

    #[test]
    fn test_file_monitor_config_for_rust_project() {
        // FAILING TEST: Should create valid default config for Rust projects
        let config = FileMonitorConfig::for_rust_project();

        assert!(config.include_patterns.contains(&"*.rs".to_string()));
        assert!(config.include_patterns.contains(&"Cargo.toml".to_string()));
        assert!(config.include_patterns.contains(&"*.md".to_string()));

        assert!(config.exclude_patterns.contains(&"target/*".to_string()));
        assert!(config.exclude_patterns.contains(&".git/*".to_string()));

        assert!(config.exclude_dirs.contains("target"));
        assert!(config.exclude_dirs.contains(".git"));

        assert_eq!(config.debounce_ms, 300);
        assert_eq!(config.max_file_size_mb, 50);
    }

    #[test]
    fn test_file_monitor_config_builder() {
        // FAILING TEST: Should support builder pattern for configuration
        let config = FileMonitorConfig::default()
            .with_debounce_ms(500)
            .with_max_queue_size(2000)
            .with_include_patterns(vec!["*.py".to_string()])
            .with_exclude_patterns(vec!["__pycache__/*".to_string()]);

        assert_eq!(config.debounce_ms, 500);
        assert_eq!(config.max_queue_size, 2000);
        assert!(config.include_patterns.contains(&"*.py".to_string()));
        assert!(config
            .exclude_patterns
            .contains(&"__pycache__/*".to_string()));
    }
}
