# Dependency Management Automation Patterns

<meta>
  <title>Dependency Management Automation Patterns</title>
  <type>pattern</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-11</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Automate dependency validation to prevent compilation errors and missing imports
- **Key Approach**: Proactive validation + workspace management = zero unresolved dependency errors
- **Core Benefits**: Early detection, automated resolution suggestions, workspace consistency
- **When to use**: API changes, new imports, crate additions, cross-component development
- **Related docs**: [Development Validation Commands](../quick-reference/development-validation-commands.md), [API Compatibility Testing](./api-compatibility-testing.md)

## <implementation>Dependency Management Framework</implementation>

### <pattern>Core Automation Pattern</pattern>

Automated dependency management prevents import resolution failures through systematic validation:

```rust
// Example: Dependency validation automation
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use cargo_metadata::{MetadataCommand, Package};

pub struct DependencyValidator {
    workspace_root: PathBuf,
    metadata: cargo_metadata::Metadata,
    dependency_map: HashMap<String, Vec<String>>,
}

impl DependencyValidator {
    pub fn new(workspace_root: PathBuf) -> Result<Self, DependencyError> {
        let metadata = MetadataCommand::new()
            .manifest_path(workspace_root.join("Cargo.toml"))
            .exec()?;
            
        let dependency_map = Self::build_dependency_map(&metadata);
        
        Ok(Self {
            workspace_root,
            metadata,
            dependency_map,
        })
    }
    
    /// Validate import statements against declared dependencies
    pub fn validate_imports(&self, file_path: &Path, imports: &[ImportStatement]) -> ValidationResult {
        let mut issues = Vec::new();
        
        for import in imports {
            match self.resolve_import_dependency(import) {
                ImportResolution::Missing(crate_name) => {
                    issues.push(DependencyIssue::MissingDependency {
                        import: import.clone(),
                        suggested_crate: crate_name,
                        file_path: file_path.to_path_buf(),
                    });
                }
                ImportResolution::VersionConflict(conflict) => {
                    issues.push(DependencyIssue::VersionConflict(conflict));
                }
                ImportResolution::Valid => {}
            }
        }
        
        ValidationResult { issues }
    }
    
    /// Suggest dependency additions for missing imports
    pub fn suggest_dependency_additions(&self, missing_imports: &[ImportStatement]) -> Vec<DependencySuggestion> {
        missing_imports.iter()
            .filter_map(|import| self.suggest_crate_for_import(import))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ImportStatement {
    pub crate_name: String,
    pub module_path: Vec<String>,
    pub items: Vec<String>,
    pub line_number: usize,
}

#[derive(Debug)]
pub enum ImportResolution {
    Valid,
    Missing(String),
    VersionConflict(VersionConflict),
}

#[derive(Debug)]
pub struct DependencySuggestion {
    pub crate_name: String,
    pub version: String,
    pub justification: String,
    pub cargo_toml_addition: String,
}
```

### <pattern>Automated Validation Pipeline</pattern>

```rust
use tokio::fs;
use tokio::process::Command;

pub struct DependencyPipeline {
    validator: DependencyValidator,
    workspace_config: WorkspaceConfig,
}

impl DependencyPipeline {
    /// Execute full dependency validation pipeline
    pub async fn validate_workspace(&self) -> Result<PipelineResult, PipelineError> {
        let mut results = PipelineResult::new();
        
        // Stage 1: Validate existing dependencies
        results.extend(self.validate_existing_dependencies().await?);
        
        // Stage 2: Check for missing imports
        results.extend(self.scan_for_missing_imports().await?);
        
        // Stage 3: Validate workspace consistency
        results.extend(self.validate_workspace_consistency().await?);
        
        // Stage 4: Check for dependency conflicts
        results.extend(self.check_version_conflicts().await?);
        
        // Stage 5: Generate resolution suggestions
        if !results.issues.is_empty() {
            results.suggestions = self.generate_resolution_suggestions(&results.issues).await?;
        }
        
        Ok(results)
    }
    
    /// Automatically resolve simple dependency issues
    pub async fn auto_resolve(&self, issues: &[DependencyIssue]) -> Result<ResolutionResult, ResolutionError> {
        let mut resolved = Vec::new();
        let mut failed = Vec::new();
        
        for issue in issues {
            match self.attempt_auto_resolution(issue).await {
                Ok(resolution) => resolved.push(resolution),
                Err(e) => failed.push((issue.clone(), e)),
            }
        }
        
        Ok(ResolutionResult { resolved, failed })
    }
    
    async fn attempt_auto_resolution(&self, issue: &DependencyIssue) -> Result<Resolution, ResolutionError> {
        match issue {
            DependencyIssue::MissingDependency { suggested_crate, .. } => {
                self.add_workspace_dependency(suggested_crate).await
            }
            DependencyIssue::VersionConflict(conflict) => {
                self.resolve_version_conflict(conflict).await
            }
            DependencyIssue::UnusedDependency(dep) => {
                self.remove_dependency(dep).await
            }
        }
    }
}

#[derive(Debug)]
pub struct PipelineResult {
    pub issues: Vec<DependencyIssue>,
    pub suggestions: Vec<DependencySuggestion>,
    pub workspace_health: WorkspaceHealth,
}

#[derive(Debug)]
pub enum DependencyIssue {
    MissingDependency {
        import: ImportStatement,
        suggested_crate: String,
        file_path: PathBuf,
    },
    VersionConflict(VersionConflict),
    UnusedDependency(String),
    WorkspaceInconsistency {
        crate_name: String,
        versions: Vec<String>,
    },
}
```

## <automation>Validation Command Integration</automation>

### <integration>Development Workflow Integration</integration>

**API Change Detection**:
```rust
/// Trigger automatic validation when core types change
pub struct ApiChangeDetector {
    core_types_paths: Vec<PathBuf>,
    last_modification: std::time::SystemTime,
}

impl ApiChangeDetector {
    pub async fn monitor_api_changes(&mut self) -> Result<(), MonitorError> {
        let mut watcher = notify::watcher(Duration::from_secs(1))?;
        
        for path in &self.core_types_paths {
            watcher.watch(path, RecursiveMode::Recursive)?;
        }
        
        while let Ok(event) = watcher.recv() {
            match event {
                DebouncedEvent::Write(path) if self.is_core_type_file(&path) => {
                    self.trigger_dependency_validation().await?;
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    async fn trigger_dependency_validation(&self) -> Result<(), ValidationError> {
        // Run automated cargo check
        let output = Command::new("cargo")
            .args(&["check", "--all-targets", "--all-features"])
            .output()
            .await?;
            
        if !output.status.success() {
            // Parse compilation errors and suggest dependency fixes
            let errors = self.parse_compilation_errors(&output.stderr)?;
            let suggestions = self.generate_dependency_suggestions(&errors).await?;
            
            // Report to user with suggested fixes
            self.report_dependency_issues(&suggestions).await?;
        }
        
        Ok(())
    }
}
```

### <automation>Import Validation Automation</automation>

```rust
/// Scan source files for import statements and validate against dependencies
pub struct ImportScanner {
    ignore_patterns: Vec<glob::Pattern>,
}

impl ImportScanner {
    pub async fn scan_workspace_imports(&self) -> Result<ImportAnalysis, ScanError> {
        let mut analysis = ImportAnalysis::new();
        
        // Scan all Rust source files
        let rust_files = self.find_rust_files().await?;
        
        for file_path in rust_files {
            let imports = self.extract_imports_from_file(&file_path).await?;
            analysis.add_file_imports(file_path, imports);
        }
        
        // Validate against workspace dependencies
        let validator = DependencyValidator::new(self.workspace_root.clone())?;
        
        for (file_path, imports) in &analysis.file_imports {
            let validation = validator.validate_imports(file_path, imports);
            analysis.add_validation_result(file_path.clone(), validation);
        }
        
        Ok(analysis)
    }
    
    async fn extract_imports_from_file(&self, file_path: &Path) -> Result<Vec<ImportStatement>, ExtractError> {
        let content = fs::read_to_string(file_path).await?;
        let mut imports = Vec::new();
        
        for (line_num, line) in content.lines().enumerate() {
            if let Some(import) = self.parse_import_statement(line, line_num + 1)? {
                imports.push(import);
            }
        }
        
        Ok(imports)
    }
    
    fn parse_import_statement(&self, line: &str, line_number: usize) -> Result<Option<ImportStatement>, ParseError> {
        let trimmed = line.trim();
        
        if trimmed.starts_with("use ") && trimmed.ends_with(';') {
            // Parse: use crate_name::module::item;
            let use_part = &trimmed[4..trimmed.len()-1]; // Remove "use " and ";"
            
            let parts: Vec<&str> = use_part.split("::").collect();
            if parts.len() >= 2 {
                return Ok(Some(ImportStatement {
                    crate_name: parts[0].to_string(),
                    module_path: parts[1..parts.len()-1].iter().map(|s| s.to_string()).collect(),
                    items: vec![parts.last().unwrap().to_string()],
                    line_number,
                }));
            }
        }
        
        Ok(None)
    }
}

#[derive(Debug)]
pub struct ImportAnalysis {
    pub file_imports: HashMap<PathBuf, Vec<ImportStatement>>,
    pub validation_results: HashMap<PathBuf, ValidationResult>,
    pub missing_dependencies: Vec<DependencyIssue>,
    pub suggestions: Vec<DependencySuggestion>,
}
```

## <commands>Automated Validation Commands</commands>

### <script>Dependency Health Check Script</script>

```bash
#!/bin/bash
# dependency-health-check.sh

set -e

echo "🔍 Dependency Health Check"
echo "========================="

# Check for missing dependencies
echo "📦 Scanning for missing dependencies..."
cargo tree --duplicates

# Check for unused dependencies  
echo "🧹 Checking for unused dependencies..."
if command -v cargo-machete &> /dev/null; then
    cargo machete
else
    echo "⚠️  Install cargo-machete for unused dependency detection: cargo install cargo-machete"
fi

# Validate import statements
echo "📥 Validating import statements..."
cargo check --all-targets --all-features

# Check workspace consistency
echo "🏗️  Checking workspace consistency..."
cargo metadata --format-version 1 | jq '.workspace_members' | wc -l
echo "Workspace members verified"

# Security audit
echo "🔒 Running security audit..."
if command -v cargo-audit &> /dev/null; then
    cargo audit
else
    echo "⚠️  Install cargo-audit for security checks: cargo install cargo-audit"
fi

echo "✅ Dependency health check complete"
```

### <automation>Git Hook Integration</automation>

```bash
#!/bin/bash
# .git/hooks/pre-commit-dependency-check

# Dependency validation in pre-commit hook
echo "🔍 Pre-commit dependency validation..."

# Check if any Cargo.toml files changed
if git diff --cached --name-only | grep -q "Cargo.toml"; then
    echo "📦 Cargo.toml modified, validating dependencies..."
    
    # Validate workspace after dependency changes
    cargo check --all-targets --all-features
    
    if [ $? -ne 0 ]; then
        echo "❌ Dependency validation failed"
        echo "💡 Run: cargo check --all-targets --all-features"
        exit 1
    fi
fi

# Check for new use statements
if git diff --cached --name-only | grep -q "\.rs$"; then
    echo "🦀 Rust files modified, checking imports..."
    
    # Extract new use statements from staged changes
    NEW_IMPORTS=$(git diff --cached | grep "^+.*use " | wc -l)
    
    if [ "$NEW_IMPORTS" -gt 0 ]; then
        echo "📥 Found $NEW_IMPORTS new import statements, validating..."
        cargo check --tests
        
        if [ $? -ne 0 ]; then
            echo "❌ Import validation failed"
            echo "💡 Check that all imported crates are declared in Cargo.toml"
            exit 1
        fi
    fi
fi

echo "✅ Dependency validation passed"
```

## <integration>Workspace Configuration Patterns</integration>

### <pattern>Workspace Dependency Management</pattern>

```toml
# Cargo.toml - Workspace-level dependency management
[workspace]
members = [
    "crates/fortitude-types",
    "crates/fortitude-core", 
    "crates/fortitude-api-server",
    "crates/fortitude-mcp-server",
]

# Workspace dependencies for consistency
[workspace.dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
clap = { version = "4.0", features = ["derive"] }
thiserror = "1.0"
tracing = "0.1"

# External dependencies with version consistency
reqwest = { version = "0.11", features = ["json"] }
axum = { version = "0.8", features = ["macros", "tokio"] }
tower = { version = "0.5", features = ["util", "timeout"] }

# Development dependencies
[workspace.dev-dependencies]
tokio-test = "0.4"
tempfile = "3.0"
criterion = "0.5"
proptest = "1.0"
```

### <pattern>Crate-Level Dependency Inheritance</pattern>

```toml
# crates/fortitude-core/Cargo.toml
[package]
name = "fortitude-core"
version = "0.1.0"
edition = "2021"

[dependencies]
# Inherit from workspace for consistency
fortitude-types = { path = "../fortitude-types" }
tokio = { workspace = true }
serde = { workspace = true }
reqwest = { workspace = true }

# Crate-specific dependencies
qdrant-client = "1.0"
uuid = { version = "1.0", features = ["v4"] }

[dev-dependencies]
tokio-test = { workspace = true }
tempfile = { workspace = true }
```

## <troubleshooting>Common Dependency Issues and Solutions</troubleshooting>

### <issue>Missing Dependency Errors</issue>
**Problem**: `cargo check` fails with "unresolved import" errors
**Detection**: Import statements reference crates not declared in Cargo.toml
**Solution**:
```bash
# 1. Identify missing crates
cargo check 2>&1 | grep "unresolved import"

# 2. Add to Cargo.toml workspace dependencies
# 3. Inherit in specific crate Cargo.toml

# 4. Verify resolution
cargo check --all-targets
```

### <issue>Version Conflicts</issue>
**Problem**: Multiple versions of same dependency in dependency tree
**Detection**: `cargo tree --duplicates` shows conflicts
**Solution**:
```bash
# 1. Identify conflicting versions
cargo tree --duplicates

# 2. Update workspace dependencies for consistency
# Edit [workspace.dependencies] in root Cargo.toml

# 3. Update all crates to use workspace = true
# 4. Verify resolution
cargo tree --duplicates
```

### <issue>Unused Dependencies</issue>
**Problem**: Dependencies declared but not used in code
**Detection**: `cargo machete` identifies unused dependencies
**Solution**:
```bash
# 1. Install cargo-machete if needed
cargo install cargo-machete

# 2. Identify unused dependencies
cargo machete

# 3. Remove unused dependencies from Cargo.toml
# 4. Verify build still works
cargo check --all-targets
```

## <validation>Dependency Health Metrics</validation>

### <metrics>Dependency Quality Indicators</metrics>

```rust
pub struct DependencyHealthReport {
    pub total_dependencies: usize,
    pub direct_dependencies: usize,
    pub transitive_dependencies: usize,
    pub duplicate_dependencies: usize,
    pub unused_dependencies: usize,
    pub outdated_dependencies: usize,
    pub security_vulnerabilities: usize,
    pub health_score: f64,
}

impl DependencyHealthReport {
    pub fn calculate_health_score(&mut self) {
        let base_score = 100.0;
        let mut penalties = 0.0;
        
        // Penalty for duplicates (high impact)
        penalties += (self.duplicate_dependencies as f64) * 10.0;
        
        // Penalty for unused dependencies (medium impact)
        penalties += (self.unused_dependencies as f64) * 5.0;
        
        // Penalty for security vulnerabilities (critical impact)
        penalties += (self.security_vulnerabilities as f64) * 20.0;
        
        // Penalty for outdated dependencies (low impact)
        penalties += (self.outdated_dependencies as f64) * 2.0;
        
        self.health_score = (base_score - penalties).max(0.0);
    }
}
```

## <references>See Also</references>

- [Development Validation Commands](../quick-reference/development-validation-commands.md)
- [API Compatibility Testing](./api-compatibility-testing.md)
- [Development Process](../../DEVELOPMENT_PROCESS.md)
- [Cargo Book - Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)
- [Cargo Book - Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)

---

**Success Metrics**: Zero unresolved import errors, 100% workspace dependency consistency, automated dependency health monitoring, proactive issue detection and resolution.