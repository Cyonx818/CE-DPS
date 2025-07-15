# Rust Workspace Configuration Patterns

<meta>
  <title>Rust Workspace Configuration Patterns</title>
  <type>pattern</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-08</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Multi-crate workspace setup for AI pipeline with CLI, library, and future server components
- **Key Approach**: Flat workspace layout with centralized dependency management and shared types
- **Core Benefits**: Shared lockfile, consistent builds, cross-crate dependency resolution, comprehensive testing
- **When to use**: Projects with 10k-1M lines requiring multiple binary/library crates
- **Related docs**: [Error Handling](error-handling.md), [Testing Patterns](testing-patterns.md)

## <implementation>Workspace Architecture</implementation>

### <pattern>Flat Workspace Layout</pattern>
```rust
// Cargo.toml - Workspace root (virtual manifest)
[workspace]
resolver = "2"
members = [
    "crates/fortitude-core",
    "crates/fortitude-cli", 
    "crates/fortitude-types",
    "crates/fortitude-test-utils",
]

// Centralized dependency management
[workspace.dependencies]
tokio = { version = "1.28", features = ["full"] }
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
tracing = "0.1"

[workspace.dev-dependencies]
fortitude-test-utils = { path = "crates/fortitude-test-utils" }
tokio-test = "0.4"
tempfile = "3.8"

// Workspace-level standards
[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
```

### <pattern>Shared Types Architecture</pattern>
```rust
// crates/fortitude-types/src/lib.rs
//! Shared types and error definitions for the Fortitude AI knowledge pipeline.

pub mod errors;
pub mod pipeline;

pub use errors::{FortitudeError, FortitudeResult};
pub use pipeline::{PipelineConfig, KnowledgeNode, ProcessingStage};

/// Re-export commonly used types for convenience
pub mod prelude {
    pub use crate::{
        FortitudeError, FortitudeResult,
        PipelineConfig, KnowledgeNode, ProcessingStage,
    };
}
```

### <pattern>Cross-Crate Error Handling</pattern>
```rust
// crates/fortitude-types/src/errors.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FortitudeError {
    #[error("Pipeline configuration error: {message}")]
    ConfigError { message: String },

    #[error("Processing failed at stage '{stage}': {source}")]
    ProcessingError { 
        stage: String,
        #[source] 
        source: Box<dyn std::error::Error + Send + Sync>
    },

    #[error("I/O operation failed: {operation}")]
    IoError { 
        operation: String,
        #[from]
        source: std::io::Error 
    },
}

pub type FortitudeResult<T> = Result<T, FortitudeError>;

impl FortitudeError {
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::ConfigError { message: message.into() }
    }

    pub fn processing<S: Into<String>>(
        stage: S,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>
    ) -> Self {
        Self::ProcessingError {
            stage: stage.into(),
            source: source.into(),
        }
    }
}
```

## <examples>Implementation Examples</examples>

### <template>Core Library Structure</template>
```rust
// crates/fortitude-core/Cargo.toml
[package]
name = "fortitude-core"
version.workspace = true
authors.workspace = true
edition.workspace = true

[dependencies]
fortitude-types = { path = "../fortitude-types" }
tokio.workspace = true
serde.workspace = true
thiserror.workspace = true
```

### <template>CLI Application Structure</template>
```rust
// crates/fortitude-cli/src/main.rs
use fortitude_cli::{Cli, Commands, run_command};
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match run_command(cli.command).await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
```

### <template>Shared Test Utilities</template>
```rust
// crates/fortitude-test-utils/src/lib.rs
use fortitude_types::{KnowledgeNode, PipelineConfig};
use tempfile::TempDir;

pub fn create_test_node(id: &str) -> KnowledgeNode {
    KnowledgeNode {
        id: id.to_string(),
        node_type: "test".to_string(),
        content: format!("Test content for {}", id),
        metadata: HashMap::new(),
        connections: vec![],
    }
}

pub fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}
```

## <troubleshooting>Common Issues</troubleshooting>

### <issue>Circular Dependencies</issue>
**Problem**: Crates trying to depend on each other
**Solution**: Extract shared types into separate crate, use trait abstractions

### <issue>Version Conflicts</issue>
**Problem**: Different versions of same dependency across crates
**Solution**: Use workspace.dependencies to centralize version management

### <issue>Build Performance</issue>
**Problem**: Slow compilation times
**Solution**: Use shared target directory, parallel builds with `cargo build --workspace`

## <references>See Also</references>
- [Error Handling Patterns](error-handling.md)
- [Testing Patterns](testing-patterns.md)
- [Async Patterns](async-patterns.md)