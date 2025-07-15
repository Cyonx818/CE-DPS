# CLI Testing Patterns

<meta>
  <title>CLI Testing Patterns</title>
  <type>pattern</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-08</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Comprehensive testing strategies for Rust CLI applications with async operations
- **Key Approach**: Multi-layered testing with unit, integration, and performance tests
- **Core Benefits**: >90% test coverage, reliable CI/CD integration, production-ready patterns
- **When to use**: CLI applications with subcommands, file I/O, async operations, complex workflows
- **Related docs**: [Async Patterns](async-patterns.md), [Error Handling](error-handling.md)

## <implementation>Testing Architecture</implementation>

### <pattern>Test Organization Structure</pattern>
```rust
// tests/
// ├── integration/
// │   ├── cli_workflows.rs      # End-to-end CLI testing
// │   ├── file_operations.rs    # File I/O integration tests
// │   └── async_workflows.rs    # Async operation integration
// ├── unit/
// │   ├── argument_parsing.rs   # Clap argument validation
// │   ├── core_logic.rs         # Business logic units
// │   └── error_handling.rs     # Error condition testing
// ├── mocks/
// │   ├── filesystem.rs         # File system mocking
// │   └── storage.rs            # Storage provider mocks
// └── common/
//     └── test_helpers.rs       # Shared test utilities
```

### <pattern>CLI Structure for Testability</pattern>
```rust
// src/lib.rs - Testable CLI architecture
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "fortitude")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    #[arg(short, long)]
    pub config: Option<PathBuf>,
    
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Research {
        #[arg(short, long)]
        query: String,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    List {
        #[arg(short, long)]
        filter: Option<String>,
    },
    CacheStatus,
}

// Dependency injection traits for testing
#[async_trait::async_trait]
pub trait StorageProvider: Send + Sync {
    async fn store_result(&self, data: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn retrieve_cached(&self, key: &str) -> Result<Option<String>, Box<dyn std::error::Error>>;
}
```

## <examples>Testing Examples</examples>

### <template>Unit Tests - Argument Parsing</template>
```rust
// tests/unit/argument_parsing.rs
use fortitude::{Cli, Commands};
use clap::Parser;

#[test]
fn test_research_command_parsing() {
    let args = vec!["fortitude", "research", "--query", "test query", "--output", "results.json"];
    let cli = Cli::try_parse_from(args).unwrap();
    
    match cli.command {
        Commands::Research { query, output } => {
            assert_eq!(query, "test query");
            assert_eq!(output, Some("results.json".into()));
        }
        _ => panic!("Expected Research command"),
    }
}

#[test]
fn test_invalid_arguments() {
    let args = vec!["fortitude", "research"]; // Missing required --query
    let result = Cli::try_parse_from(args);
    assert!(result.is_err());
}

#[test]
fn test_global_flags() {
    let args = vec!["fortitude", "--verbose", "--config", "custom.toml", "list"];
    let cli = Cli::try_parse_from(args).unwrap();
    
    assert!(cli.verbose);
    assert_eq!(cli.config, Some("custom.toml".into()));
}
```

### <template>Integration Tests - CLI Workflows</template>
```rust
// tests/integration/cli_workflows.rs
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_research_command_output() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = assert_fs::TempDir::new()?;
    let output_file = temp_dir.child("results.json");
    
    let mut cmd = Command::cargo_bin("fortitude")?;
    cmd.arg("research")
        .arg("--query")
        .arg("machine learning")
        .arg("--output")
        .arg(output_file.path());
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Research completed"));
    
    output_file.assert(predicate::path::exists());
    
    let content = output_file.read_to_string()?;
    assert!(content.contains("machine learning"));
    
    Ok(())
}

#[test]
fn test_cache_status_subcommand() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("fortitude")?;
    cmd.arg("cache-status");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Cache entries:").or(
            predicate::str::contains("Cache is empty")
        ));
    
    Ok(())
}
```

### <template>Async Testing Patterns</template>
```rust
// tests/async/tokio_integration.rs
use tokio::time::{sleep, Duration, timeout};
use serial_test::serial;

#[tokio::test]
async fn test_concurrent_research_operations() {
    let tasks = (0..5).map(|i| {
        tokio::spawn(async move {
            simulate_research_task(format!("query_{}", i)).await
        })
    });
    
    let results: Vec<_> = futures::future::join_all(tasks).await;
    
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}

#[tokio::test]
async fn test_timeout_handling() {
    let result = timeout(
        Duration::from_millis(100),
        simulate_slow_operation()
    ).await;
    
    assert!(result.is_err()); // Should timeout
}

#[tokio::test(flavor = "multi_thread")]
async fn test_multi_threaded_operations() {
    let handle1 = tokio::spawn(async { process_large_dataset().await });
    let handle2 = tokio::spawn(async { perform_classification().await });
    
    let (result1, result2) = tokio::try_join!(handle1, handle2).unwrap();
    assert!(result1.is_ok());
    assert!(result2.is_ok());
}
```

### <template>Mock Testing with Dependency Injection</template>
```rust
// tests/unit/core_logic.rs
use mockall::{predicate::*, mock};

mock! {
    Storage {}
    
    #[async_trait::async_trait]
    impl StorageProvider for Storage {
        async fn store_result(&self, data: &str) -> Result<(), Box<dyn std::error::Error>>;
        async fn retrieve_cached(&self, key: &str) -> Result<Option<String>, Box<dyn std::error::Error>>;
    }
}

#[tokio::test]
async fn test_research_with_cache_hit() {
    let mut mock_storage = MockStorage::new();
    mock_storage
        .expect_retrieve_cached()
        .with(eq("test_query"))
        .returning(|_| Ok(Some("cached_result".to_string())));
    
    let result = perform_research("test_query", &mock_storage).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "cached_result");
}

async fn perform_research(query: &str, storage: &dyn StorageProvider) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(cached) = storage.retrieve_cached(query).await? {
        return Ok(cached);
    }
    
    let result = format!("fresh_result_for_{}", query);
    storage.store_result(&result).await?;
    Ok(result)
}
```

### <template>File System Mocking</template>
```rust
// tests/mocks/filesystem.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct MockFileSystem {
    files: Arc<Mutex<HashMap<String, String>>>,
}

impl FileSystem for MockFileSystem {
    fn read_to_string(&self, path: &std::path::Path) -> std::io::Result<String> {
        let files = self.files.lock().unwrap();
        files.get(&path.to_string_lossy().to_string())
            .cloned()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found in mock"
            ))
    }
    
    fn write(&self, path: &std::path::Path, contents: &str) -> std::io::Result<()> {
        let mut files = self.files.lock().unwrap();
        files.insert(path.to_string_lossy().to_string(), contents.to_string());
        Ok(())
    }
    
    fn exists(&self, path: &std::path::Path) -> bool {
        let files = self.files.lock().unwrap();
        files.contains_key(&path.to_string_lossy().to_string())
    }
}
```

## <troubleshooting>Testing Issues</troubleshooting>

### <issue>Flaky Async Tests</issue>
**Problem**: Tests failing intermittently due to timing issues
**Solution**: 
- Use `tokio-test` for deterministic async testing
- Implement proper timeout handling with `tokio::time::timeout`
- Use `serial_test` for tests that can't run concurrently

### <issue>CLI Integration Test Failures</issue>
**Problem**: Tests failing in CI but passing locally
**Solution**: 
- Use `assert_fs::TempDir` for isolated test environments
- Ensure tests clean up after themselves
- Use proper environment variable isolation

### <issue>Mock Setup Complexity</issue>
**Problem**: Complex mock setup obscuring test intent
**Solution**: 
- Create test helper functions for common mock scenarios
- Use builder pattern for complex mock configurations
- Implement trait-based dependency injection

## <references>See Also</references>
- [Async Patterns](async-patterns.md)
- [Error Handling Patterns](error-handling.md)
- [Performance Testing](../research/performance-testing.md)