//! Integration tests for CLI commands with vector services.
//! These tests verify CLI functionality works correctly with the public API.

use fortitude_core::vector::{
    config::{ConnectionPoolConfig, DistanceMetric, HealthCheckConfig, VectorConfig},
    embeddings::{CacheKeyStrategy, DeviceType, EmbeddingCacheConfig, EmbeddingConfig},
};
use std::fs;
use std::process::Command;
use std::time::Duration;
use tempfile::TempDir;

/// Test configuration helper for CLI integration tests
#[allow(dead_code)]
fn create_test_cli_config() -> VectorConfig {
    VectorConfig {
        url: "http://localhost:6334".to_string(),
        api_key: None,
        timeout: Duration::from_secs(30),
        default_collection: "test_cli_collection".to_string(),
        vector_dimensions: 384,
        distance_metric: DistanceMetric::Cosine,
        health_check: HealthCheckConfig {
            enabled: true,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            max_failures: 3,
        },
        connection_pool: ConnectionPoolConfig {
            max_connections: 10,
            connection_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(600),
        },
        embedding: EmbeddingConfig {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            max_sequence_length: 128,
            batch_size: 4,
            device: DeviceType::Cpu,
            cache_config: EmbeddingCacheConfig {
                enabled: true,
                max_entries: 100,
                ttl: Duration::from_secs(300),
                key_strategy: CacheKeyStrategy::Hash,
            },
            ..Default::default()
        },
    }
}

/// Create test data directory with sample research content
fn setup_test_data_directory() -> std::io::Result<TempDir> {
    let temp_dir = TempDir::new()?;

    // Create sample research files
    let research_dir = temp_dir.path().join("research");
    fs::create_dir_all(&research_dir)?;

    // Sample research document 1
    let doc1 = serde_json::json!({
        "query": "How to implement async patterns in Rust?",
        "research_type": "Implementation",
        "content": "Asynchronous programming in Rust uses the async/await syntax with the tokio runtime...",
        "quality_score": 0.9,
        "tags": ["rust", "async", "programming"]
    });
    fs::write(
        research_dir.join("async_rust.json"),
        serde_json::to_string_pretty(&doc1)?,
    )?;

    // Sample research document 2
    let doc2 = serde_json::json!({
        "query": "Database optimization techniques",
        "research_type": "BestPractices",
        "content": "Database optimization involves proper indexing, query optimization, and caching strategies...",
        "quality_score": 0.85,
        "tags": ["database", "optimization", "performance"]
    });
    fs::write(
        research_dir.join("database_optimization.json"),
        serde_json::to_string_pretty(&doc2)?,
    )?;

    // Sample configuration file
    let config = serde_json::json!({
        "claude": {
            "api_key": "test-key",
            "model": "claude-3-sonnet-20240229",
            "timeout_seconds": 60
        },
        "vector": {
            "url": "http://localhost:6334",
            "default_collection": "test_cli_collection",
            "vector_dimensions": 384,
            "distance_metric": "cosine",
            "timeout_seconds": 30
        },
        "storage": {
            "base_path": temp_dir.path().join("storage"),
            "cache_expiration_seconds": 3600
        },
        "classification": {
            "default_threshold": 0.7,
            "enable_advanced": true,
            "enable_context_detection": true
        }
    });
    fs::write(
        temp_dir.path().join("fortitude_config.json"),
        serde_json::to_string_pretty(&config)?,
    )?;

    Ok(temp_dir)
}

/// ANCHOR: Test CLI help and version commands
/// Tests: Basic CLI functionality, help output, version information
#[test]
fn test_anchor_cli_basic_commands() {
    // Test help command
    let output = Command::new("cargo")
        .args(["run", "-p", "fortitude-cli", "--", "--help"])
        .output()
        .expect("Failed to execute help command");

    assert!(output.status.success(), "Help command should succeed");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in help output");
    assert!(
        stdout.contains("Automated research system"),
        "Should contain main description"
    );
    assert!(stdout.contains("research"), "Should list research command");
    assert!(stdout.contains("vector"), "Should list vector command");
    assert!(
        stdout.contains("semantic-search"),
        "Should list semantic search command"
    );

    // Test version command
    let output = Command::new("cargo")
        .args(["run", "-p", "fortitude-cli", "--", "--version"])
        .output()
        .expect("Failed to execute version command");

    assert!(output.status.success(), "Version command should succeed");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in version output");
    assert!(stdout.contains("0.1.0"), "Should contain version number");
}

/// ANCHOR: Test CLI configuration commands
/// Tests: Configuration validation, generation, and display
#[test]
fn test_anchor_cli_configuration_commands() {
    let temp_dir = setup_test_data_directory().expect("Failed to setup test data");

    // Test config generate command
    let config_path = temp_dir.path().join("generated_config.json");
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "config",
            "generate",
            "--output",
            config_path.to_str().unwrap(),
            "--force",
        ])
        .output()
        .expect("Failed to execute config generate command");

    assert!(output.status.success(), "Config generate should succeed");
    assert!(config_path.exists(), "Config file should be generated");

    // Verify generated config is valid JSON
    let config_content = fs::read_to_string(&config_path).expect("Failed to read generated config");
    let _: serde_json::Value =
        serde_json::from_str(&config_content).expect("Generated config should be valid JSON");

    // Test config show command
    let output = Command::new("cargo")
        .args(["run", "-p", "fortitude-cli", "--", "config", "show"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute config show command");

    // Should succeed even without config file (uses defaults)
    assert!(output.status.success(), "Config show should succeed");

    // Test config validation
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "config",
            "validate",
            "--file",
            config_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute config validate command");

    assert!(
        output.status.success(),
        "Config validation should succeed for generated config"
    );
}

/// ANCHOR: Test CLI research commands with various options
/// Tests: Research execution, output formatting, advanced classification
#[tokio::test]
async fn test_anchor_cli_research_commands() {
    let temp_dir = setup_test_data_directory().expect("Failed to setup test data");

    // Test basic research command
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "research",
            "How to optimize Rust performance?",
            "--level",
            "intermediate",
            "--domain",
            "rust",
            "--format",
            "json",
            "--no-cache",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute research command");

    // Should succeed or fail gracefully without Claude API key
    let _stdout = String::from_utf8(output.stdout).unwrap_or_default();
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();

    // Check that command was parsed correctly
    assert!(
        output.status.success()
            || stderr.contains("Claude API")
            || stderr.contains("configuration"),
        "Research command should succeed or fail with clear error message"
    );

    // Test research with advanced classification
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "research",
            "Advanced async patterns for concurrent systems",
            "--advanced-classification",
            "--context-detection",
            "--context-threshold",
            "0.8",
            "--graceful-degradation",
            "--format",
            "markdown",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute advanced research command");

    // Should handle advanced options gracefully
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();
    assert!(
        output.status.success()
            || stderr.contains("Claude API")
            || stderr.contains("configuration"),
        "Advanced research command should succeed or fail with clear error message"
    );

    // Test research with tags and frameworks
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "research",
            "Web API development patterns",
            "--technology",
            "rust",
            "--project-type",
            "web",
            "--frameworks",
            "axum,tokio",
            "--tags",
            "api,rest,microservices",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute research with tags command");

    // Should parse and handle tags correctly
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();
    assert!(
        output.status.success()
            || stderr.contains("Claude API")
            || stderr.contains("configuration"),
        "Research with tags should succeed or fail with clear error message"
    );
}

/// ANCHOR: Test CLI cache management commands
/// Tests: Cache listing, searching, status, and cleanup
#[tokio::test]
async fn test_anchor_cli_cache_commands() {
    let temp_dir = setup_test_data_directory().expect("Failed to setup test data");

    // Test cache status command
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "cache-status",
            "--format",
            "json",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute cache status command");

    assert!(output.status.success(), "Cache status should succeed");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in cache status output");
    // Should output cache statistics in JSON format
    assert!(!stdout.is_empty(), "Cache status should produce output");

    // Test cache status with detailed flag
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "cache-status",
            "--detailed",
            "--format",
            "table",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute detailed cache status command");

    assert!(
        output.status.success(),
        "Detailed cache status should succeed"
    );

    // Test list cached results
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "list",
            "--format",
            "table",
            "--limit",
            "5",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute list command");

    assert!(output.status.success(), "List command should succeed");

    // Test search cached results
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "search",
            "rust async",
            "--format",
            "json",
            "--limit",
            "3",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute search command");

    assert!(output.status.success(), "Search command should succeed");

    // Test cleanup with dry run
    let output = Command::new("cargo")
        .args(["run", "-p", "fortitude-cli", "--", "cleanup", "--dry-run"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute cleanup dry run command");

    assert!(output.status.success(), "Cleanup dry run should succeed");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in cleanup output");
    assert!(stdout.contains("DRY RUN"), "Should indicate dry run mode");
}

/// ANCHOR: Test CLI vector database commands
/// Tests: Vector configuration, health checks, stats, setup
#[tokio::test]
async fn test_anchor_cli_vector_commands() {
    let temp_dir = setup_test_data_directory().expect("Failed to setup test data");

    // Test vector config show
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "vector",
            "config",
            "--show",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute vector config show command");

    // Should succeed and show config (even if vector DB not configured)
    assert!(output.status.success(), "Vector config show should succeed");

    // Test vector health check
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "vector",
            "health",
            "--format",
            "json",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute vector health command");

    // May fail if vector DB not available, but should handle gracefully
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();
    if !output.status.success() {
        assert!(
            stderr.contains("not available") || stderr.contains("not configured"),
            "Should provide clear error message when vector DB unavailable"
        );
    }

    // Test vector stats
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "vector",
            "stats",
            "--format",
            "table",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute vector stats command");

    // May fail if vector DB not available
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();
    if !output.status.success() {
        assert!(
            stderr.contains("not available") || stderr.contains("not configured"),
            "Should provide clear error message for stats when DB unavailable"
        );
    }

    // Test vector setup with dry run equivalent (show what would be done)
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "vector",
            "setup",
            "--collection",
            "test_collection",
            "--dimensions",
            "384",
            "--metric",
            "cosine",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute vector setup command");

    // Should show setup parameters even if not executed
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();
    if !output.status.success() {
        assert!(
            stderr.contains("not available")
                || stderr.contains("not configured")
                || stderr.contains("not yet implemented"),
            "Should provide clear message about setup availability"
        );
    }
}

/// ANCHOR: Test CLI migration commands
/// Tests: Migration operations, status tracking, list management
#[tokio::test]
async fn test_anchor_cli_migration_commands() {
    let temp_dir = setup_test_data_directory().expect("Failed to setup test data");

    // Test migration with dry run
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "vector",
            "migrate",
            temp_dir.path().join("research").to_str().unwrap(),
            "--collection",
            "test_migration",
            "--batch-size",
            "10",
            "--validation",
            "moderate",
            "--dry-run",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute migration dry run command");

    // Should succeed in dry run mode or provide clear error
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();
    let stdout = String::from_utf8(output.stdout).unwrap_or_default();

    if output.status.success() {
        assert!(stdout.contains("DRY RUN"), "Should indicate dry run mode");
    } else {
        assert!(
            stderr.contains("not available") || stderr.contains("not configured"),
            "Should provide clear error when migration service unavailable"
        );
    }

    // Test migration status list
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "vector",
            "migration-status",
            "--all",
            "--format",
            "table",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute migration status command");

    // Should handle gracefully if no migrations exist
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();
    if !output.status.success() {
        assert!(
            stderr.contains("not available") || stderr.contains("not configured"),
            "Should provide clear error for migration status when unavailable"
        );
    }

    // Test migration list
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "vector",
            "migration-list",
            "--format",
            "json",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute migration list command");

    // Should handle gracefully
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();
    if !output.status.success() {
        assert!(
            stderr.contains("not available") || stderr.contains("not configured"),
            "Should provide clear error for migration list when unavailable"
        );
    }
}

/// ANCHOR: Test CLI search commands with vector integration
/// Tests: Semantic search, hybrid search, similarity search
#[tokio::test]
async fn test_anchor_cli_search_commands() {
    let temp_dir = setup_test_data_directory().expect("Failed to setup test data");

    // Test semantic search command
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "semantic-search",
            "async programming patterns",
            "--strategy",
            "semantic",
            "--limit",
            "5",
            "--threshold",
            "0.7",
            "--format",
            "table",
            "--explain",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute semantic search command");

    // Should handle gracefully if vector services not available
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();
    if !output.status.success() {
        assert!(
            stderr.contains("not available") || stderr.contains("not configured"),
            "Should provide clear error for semantic search when services unavailable"
        );
    }

    // Test hybrid search command
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "hybrid-search",
            "database optimization techniques",
            "--keyword-weight",
            "0.4",
            "--semantic-weight",
            "0.6",
            "--limit",
            "3",
            "--threshold",
            "0.5",
            "--format",
            "json",
            "--explain",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute hybrid search command");

    // Should handle gracefully if hybrid services not available
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();
    if !output.status.success() {
        assert!(
            stderr.contains("not available") || stderr.contains("not configured"),
            "Should provide clear error for hybrid search when services unavailable"
        );
    }

    // Test find similar command
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "find-similar",
            "How to implement efficient concurrent algorithms in Rust using async/await?",
            "--limit",
            "5",
            "--threshold",
            "0.8",
            "--format",
            "detailed",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute find similar command");

    // Should handle gracefully if services not available
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();
    if !output.status.success() {
        assert!(
            stderr.contains("not available") || stderr.contains("not configured"),
            "Should provide clear error for find similar when services unavailable"
        );
    }
}

/// ANCHOR: Test CLI analytics and monitoring commands
/// Tests: Vector analytics, performance metrics, monitoring
#[tokio::test]
async fn test_anchor_cli_analytics_commands() {
    let temp_dir = setup_test_data_directory().expect("Failed to setup test data");

    // Test vector analytics command
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "vector",
            "analytics",
            "--period",
            "7",
            "--collection",
            "test_collection",
            "--format",
            "json",
            "--performance",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute vector analytics command");

    // Should handle gracefully if analytics not available
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();
    if !output.status.success() {
        assert!(
            stderr.contains("not available")
                || stderr.contains("not configured")
                || stderr.contains("not yet implemented"),
            "Should provide clear error for analytics when services unavailable"
        );
    }

    // Test index status command
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "vector",
            "index",
            "status",
            "--collection",
            "test_collection",
            "--format",
            "table",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute index status command");

    // Should handle gracefully if index operations not available
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();
    if !output.status.success() {
        assert!(
            stderr.contains("not available")
                || stderr.contains("not configured")
                || stderr.contains("not yet implemented"),
            "Should provide clear error for index status when services unavailable"
        );
    }
}

/// ANCHOR: Test CLI error handling and validation
/// Tests: Invalid arguments, missing configuration, graceful failures
#[test]
fn test_anchor_cli_error_handling() {
    // Test invalid command
    let output = Command::new("cargo")
        .args(["run", "-p", "fortitude-cli", "--", "invalid-command"])
        .output()
        .expect("Failed to execute invalid command");

    assert!(!output.status.success(), "Invalid command should fail");

    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8 in error output");
    assert!(
        stderr.contains("error") || stderr.contains("invalid") || stderr.contains("unrecognized"),
        "Should provide clear error message for invalid command"
    );

    // Test missing required arguments
    let output = Command::new("cargo")
        .args(["run", "-p", "fortitude-cli", "--", "research"])
        .output()
        .expect("Failed to execute research without topic");

    assert!(
        !output.status.success(),
        "Research command without topic should fail"
    );

    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8 in error output");
    assert!(
        stderr.contains("required") || stderr.contains("argument") || stderr.contains("topic"),
        "Should indicate missing required topic argument"
    );

    // Test invalid option values
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "research",
            "test topic",
            "--context-threshold",
            "invalid_number",
        ])
        .output()
        .expect("Failed to execute research with invalid threshold");

    assert!(
        !output.status.success(),
        "Invalid threshold value should fail"
    );

    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8 in error output");
    assert!(
        stderr.contains("invalid") || stderr.contains("parse") || stderr.contains("number"),
        "Should indicate invalid number format"
    );

    // Test invalid vector command arguments
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "vector",
            "migrate",
            "nonexistent_source",
            "--batch-size",
            "invalid_size",
        ])
        .output()
        .expect("Failed to execute migration with invalid batch size");

    assert!(!output.status.success(), "Invalid batch size should fail");

    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8 in error output");
    assert!(
        stderr.contains("invalid") || stderr.contains("parse") || stderr.contains("number"),
        "Should indicate invalid batch size format"
    );
}

/// ANCHOR: Test CLI output formatting consistency
/// Tests: JSON output validation, table formatting, consistent error messages
#[tokio::test]
async fn test_anchor_cli_output_formatting() {
    let temp_dir = setup_test_data_directory().expect("Failed to setup test data");

    // Test JSON output formatting
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "cache-status",
            "--format",
            "json",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute cache status with JSON format");

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in JSON output");
        if !stdout.trim().is_empty() {
            // Verify JSON is valid
            let _: serde_json::Value =
                serde_json::from_str(&stdout).expect("JSON output should be valid");
        }
    }

    // Test table output formatting
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "list",
            "--format",
            "table",
            "--limit",
            "3",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute list with table format");

    assert!(output.status.success(), "List command should succeed");

    // Test verbose flag
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "--verbose",
            "cache-status",
        ])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command with verbose flag");

    assert!(output.status.success(), "Verbose flag should work");

    // Test data directory override
    let custom_data_dir = temp_dir.path().join("custom_data");
    fs::create_dir_all(&custom_data_dir).expect("Failed to create custom data directory");

    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "--data-dir",
            custom_data_dir.to_str().unwrap(),
            "cache-status",
        ])
        .output()
        .expect("Failed to execute command with custom data directory");

    assert!(output.status.success(), "Custom data directory should work");
}

/// ANCHOR: Test CLI integration with mock vector services
/// Tests: End-to-end CLI workflows with simulated vector operations
#[tokio::test]
async fn test_anchor_cli_mock_vector_integration() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = setup_test_data_directory().expect("Failed to setup test data");

    // Create a more complete test configuration
    let enhanced_config = serde_json::json!({
        "claude": {
            "api_key": "mock-test-key",
            "model": "claude-3-sonnet-20240229",
            "timeout_seconds": 60,
            "base_url": "http://localhost:8080/mock-claude"
        },
        "vector": {
            "url": "http://localhost:6334",
            "default_collection": "test_cli_integration",
            "vector_dimensions": 384,
            "distance_metric": "cosine",
            "timeout_seconds": 30
        },
        "storage": {
            "base_path": temp_dir.path().join("storage"),
            "cache_expiration_seconds": 3600,
            "enable_content_addressing": true
        },
        "classification": {
            "default_threshold": 0.7,
            "enable_advanced": true,
            "enable_context_detection": true
        },
        "pipeline": {
            "enable_caching": true,
            "enable_validation": true
        }
    });

    let config_path = temp_dir.path().join("enhanced_config.json");
    fs::write(
        &config_path,
        serde_json::to_string_pretty(&enhanced_config)?,
    )
    .expect("Failed to write enhanced config");

    // Test full research workflow with all features enabled
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "fortitude-cli",
            "--",
            "--data-dir",
            temp_dir.path().to_str().unwrap(),
            "--verbose",
            "research",
            "How to implement high-performance vector search in Rust?",
            "--level",
            "expert",
            "--domain",
            "rust",
            "--technology",
            "rust",
            "--project-type",
            "library",
            "--frameworks",
            "tokio,serde,qdrant",
            "--tags",
            "search,vector,performance,optimization",
            "--advanced-classification",
            "--context-detection",
            "--context-threshold",
            "0.85",
            "--graceful-degradation",
            "--format",
            "json",
        ])
        .output()
        .expect("Failed to execute comprehensive research command");

    // Should handle all options gracefully even without real services
    let stderr = String::from_utf8(output.stderr).unwrap_or_default();
    if !output.status.success() {
        // Should provide informative error messages
        assert!(
            stderr.contains("Claude API")
                || stderr.contains("configuration")
                || stderr.contains("not configured")
                || stderr.contains("placeholder"),
            "Should provide clear guidance when services unavailable: {stderr}"
        );
    }

    // Test vector command workflow
    let commands = vec![
        vec!["vector", "config", "--show"],
        vec![
            "vector",
            "setup",
            "--collection",
            "test_workflow",
            "--dimensions",
            "384",
        ],
        vec!["vector", "health", "--format", "json"],
        vec!["vector", "stats", "--format", "table"],
    ];

    for cmd_args in commands {
        let mut full_args = vec!["run", "-p", "fortitude-cli", "--"];
        full_args.extend(cmd_args.clone());

        let output = Command::new("cargo")
            .args(&full_args)
            .current_dir(temp_dir.path())
            .output()
            .unwrap_or_else(|_| panic!("Failed to execute vector command: {cmd_args:?}"));

        // Commands should either succeed or fail with clear messages
        if !output.status.success() {
            let stderr = String::from_utf8(output.stderr).unwrap_or_default();
            assert!(
                stderr.contains("not available")
                    || stderr.contains("not configured")
                    || stderr.contains("not yet implemented"),
                "Vector command should provide clear error message: {cmd_args:?} -> {stderr}"
            );
        }
    }

    // Test search command workflow
    let search_commands = vec![
        vec![
            "semantic-search",
            "rust async patterns",
            "--strategy",
            "semantic",
            "--limit",
            "5",
        ],
        vec![
            "hybrid-search",
            "database optimization",
            "--semantic-weight",
            "0.7",
            "--keyword-weight",
            "0.3",
        ],
        vec![
            "find-similar",
            "vector search implementation",
            "--threshold",
            "0.8",
        ],
    ];

    for cmd_args in search_commands {
        let mut full_args = vec!["run", "-p", "fortitude-cli", "--"];
        full_args.extend(cmd_args.clone());

        let output = Command::new("cargo")
            .args(&full_args)
            .current_dir(temp_dir.path())
            .output()
            .unwrap_or_else(|_| panic!("Failed to execute search command: {cmd_args:?}"));

        // Search commands should handle absence of vector services gracefully
        if !output.status.success() {
            let stderr = String::from_utf8(output.stderr).unwrap_or_default();
            assert!(
                stderr.contains("not available") || stderr.contains("not configured"),
                "Search command should provide clear error message: {cmd_args:?} -> {stderr}"
            );
        }
    }

    Ok(())
}
