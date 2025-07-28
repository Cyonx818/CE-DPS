// Copyright 2025 CE-DPS Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// ABOUTME: Gap detection algorithms for automated knowledge gap analysis in proactive research mode
//! This module provides algorithms for detecting various types of knowledge gaps in codebases:
//! - TODO/FIXME/HACK comments and their context
//! - Missing documentation (functions without docstrings, undocumented APIs)
//! - Undocumented technologies (imports, dependencies not in docs)
//! - API documentation gaps (public APIs without examples or proper descriptions)
//! - Configuration gaps (config options not documented)

use crate::proactive::FileEvent;
use regex::{Regex, RegexSet};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tokio::fs;
use tokio::time::{Duration, Instant};
use tracing::{debug, error, warn};

/// Errors that can occur during gap analysis
#[derive(Error, Debug)]
pub enum GapAnalysisError {
    #[error("Failed to read file {path}: {error}")]
    FileRead { path: String, error: std::io::Error },

    #[error("Failed to analyze file {path}: {error}")]
    AnalysisFailed { path: String, error: String },

    #[error("Regex compilation failed: {0}")]
    RegexCompilation(#[from] regex::Error),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Performance threshold exceeded: analysis took {duration:?}, limit is {limit:?}")]
    PerformanceThreshold { duration: Duration, limit: Duration },

    #[error("File too large: {size} bytes, limit is {limit} bytes")]
    FileTooLarge { size: u64, limit: u64 },

    #[error("Unsupported file type: {extension}")]
    UnsupportedFileType { extension: String },
}

/// Type of knowledge gap detected
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GapType {
    /// TODO/FIXME/HACK comments
    TodoComment,
    /// Missing function/struct documentation
    MissingDocumentation,
    /// Undocumented technology/dependency
    UndocumentedTechnology,
    /// Public API without proper documentation
    ApiDocumentationGap,
    /// Configuration option not documented
    ConfigurationGap,
}

impl GapType {
    /// Get the priority score for this gap type (1-10, higher is more urgent)
    pub fn priority(&self) -> u8 {
        match self {
            GapType::TodoComment => 7,
            GapType::MissingDocumentation => 6,
            GapType::UndocumentedTechnology => 8,
            GapType::ApiDocumentationGap => 9,
            GapType::ConfigurationGap => 5,
        }
    }
}

/// A detected knowledge gap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedGap {
    /// Type of gap detected
    pub gap_type: GapType,
    /// File where the gap was found
    pub file_path: PathBuf,
    /// Line number where the gap was detected (1-based)
    pub line_number: usize,
    /// Column number where the gap was detected (1-based, optional)
    pub column_number: Option<usize>,
    /// Context around the gap (the actual content)
    pub context: String,
    /// Extracted information about what needs to be done
    pub description: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Priority score (1-10, higher is more urgent)
    pub priority: u8,
    /// Additional metadata specific to the gap type
    pub metadata: HashMap<String, String>,
}

impl DetectedGap {
    /// Create a new detected gap
    pub fn new(
        gap_type: GapType,
        file_path: PathBuf,
        line_number: usize,
        context: String,
        description: String,
        confidence: f64,
    ) -> Self {
        let priority = gap_type.priority();
        Self {
            gap_type,
            file_path,
            line_number,
            column_number: None,
            context,
            description,
            confidence,
            priority,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the gap
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Set the column number
    pub fn with_column(mut self, column: usize) -> Self {
        self.column_number = Some(column);
        self
    }
}

/// Configuration for gap analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapAnalysisConfig {
    /// Maximum file size to analyze (in bytes)
    pub max_file_size_bytes: u64,
    /// Performance timeout for analysis (in milliseconds)
    pub analysis_timeout_ms: u64,
    /// Minimum confidence threshold for reporting gaps
    pub min_confidence_threshold: f64,
    /// File extensions to analyze
    pub supported_extensions: HashSet<String>,
    /// Whether to enable TODO comment detection
    pub enable_todo_detection: bool,
    /// Whether to enable missing documentation detection
    pub enable_docs_detection: bool,
    /// Whether to enable undocumented technology detection
    pub enable_tech_detection: bool,
    /// Whether to enable API documentation gap detection
    pub enable_api_detection: bool,
    /// Whether to enable configuration gap detection
    pub enable_config_detection: bool,
    /// Custom TODO patterns (regex)
    pub custom_todo_patterns: Vec<String>,
    /// Custom documentation patterns
    pub custom_doc_patterns: Vec<String>,
}

impl Default for GapAnalysisConfig {
    fn default() -> Self {
        Self {
            max_file_size_bytes: 50 * 1024 * 1024, // 50MB
            analysis_timeout_ms: 500,
            min_confidence_threshold: 0.6,
            supported_extensions: [
                "rs", "md", "yaml", "yml", "toml", "json", "py", "js", "ts", "go", "java", "cpp",
                "c", "h",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            enable_todo_detection: true,
            enable_docs_detection: true,
            enable_tech_detection: true,
            enable_api_detection: true,
            enable_config_detection: true,
            custom_todo_patterns: Vec::new(),
            custom_doc_patterns: Vec::new(),
        }
    }
}

impl GapAnalysisConfig {
    /// Create configuration optimized for Rust projects
    pub fn for_rust_project() -> Self {
        Self {
            supported_extensions: ["rs", "md", "toml", "yaml", "yml", "json"]
                .into_iter()
                .map(String::from)
                .collect(),
            ..Default::default()
        }
    }

    /// Builder method to set file size limit
    pub fn with_max_file_size_mb(mut self, size_mb: u64) -> Self {
        self.max_file_size_bytes = size_mb * 1024 * 1024;
        self
    }

    /// Builder method to set analysis timeout
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.analysis_timeout_ms = timeout_ms;
        self
    }

    /// Builder method to set confidence threshold
    pub fn with_confidence_threshold(mut self, threshold: f64) -> Self {
        self.min_confidence_threshold = threshold;
        self
    }

    /// Check if a file should be analyzed based on extension
    pub fn should_analyze_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                return self.supported_extensions.contains(ext_str);
            }
        }
        false
    }
}

/// Compiled regex patterns for efficient gap detection
#[derive(Debug)]
struct CompiledPatterns {
    todo_patterns: RegexSet,
    todo_extractors: Vec<Regex>,
    rust_function_patterns: Regex,
    rust_struct_patterns: Regex,
    #[allow(dead_code)] // TODO: Will be used for implementation gap analysis
    rust_impl_patterns: Regex,
    rust_use_patterns: Regex,
    #[allow(dead_code)] // TODO: Will be used for documentation structure analysis
    markdown_header_patterns: Regex,
    config_key_patterns: Regex,
}

impl CompiledPatterns {
    fn new(config: &GapAnalysisConfig) -> Result<Self, GapAnalysisError> {
        // TODO comment patterns
        let mut todo_pattern_strings = vec![
            r"(?i)//\s*TODO:?\s*(.+)",
            r"(?i)//\s*FIXME:?\s*(.+)",
            r"(?i)//\s*HACK:?\s*(.+)",
            r"(?i)//\s*BUG:?\s*(.+)",
            r"(?i)//\s*NOTE:?\s*(.+)",
            r"(?i)#\s*TODO:?\s*(.+)",
            r"(?i)/\*\s*TODO:?\s*(.+?)\s*\*/",
        ];
        for pattern in &config.custom_todo_patterns {
            todo_pattern_strings.push(pattern.as_str());
        }

        let todo_patterns = RegexSet::new(&todo_pattern_strings)?;
        let todo_extractors: Result<Vec<_>, _> =
            todo_pattern_strings.into_iter().map(Regex::new).collect();
        let todo_extractors = todo_extractors?;

        Ok(Self {
            todo_patterns,
            todo_extractors,
            rust_function_patterns: Regex::new(
                r"(?m)^(\s*)pub\s+(async\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(",
            )?,
            rust_struct_patterns: Regex::new(r"(?m)^(\s*)pub\s+struct\s+([a-zA-Z_][a-zA-Z0-9_]*)")?,
            rust_impl_patterns: Regex::new(r"(?m)^(\s*)impl\s+.*?\s*\{")?,
            rust_use_patterns: Regex::new(
                r"(?m)^use\s+([a-zA-Z_][a-zA-Z0-9_]*(?:::[a-zA-Z_][a-zA-Z0-9_]*)*)",
            )?,
            markdown_header_patterns: Regex::new(r"(?m)^(#{1,6})\s+(.+)")?,
            config_key_patterns: Regex::new(r"(?m)^([a-zA-Z_][a-zA-Z0-9_]*)\s*=")?,
        })
    }
}

/// Main gap analyzer
pub struct GapAnalyzer {
    config: GapAnalysisConfig,
    patterns: Arc<CompiledPatterns>,
}

impl GapAnalyzer {
    /// Create a new gap analyzer with the given configuration
    pub fn new(config: GapAnalysisConfig) -> Result<Self, GapAnalysisError> {
        let patterns = Arc::new(CompiledPatterns::new(&config)?);
        Ok(Self { config, patterns })
    }

    /// Create a gap analyzer with default configuration for Rust projects
    pub fn for_rust_project() -> Result<Self, GapAnalysisError> {
        Self::new(GapAnalysisConfig::for_rust_project())
    }

    /// Analyze a file for knowledge gaps
    pub async fn analyze_file(
        &self,
        file_path: &Path,
    ) -> Result<Vec<DetectedGap>, GapAnalysisError> {
        let start_time = Instant::now();

        // Check if file should be analyzed
        if !self.config.should_analyze_file(file_path) {
            return Ok(Vec::new());
        }

        // Check file size
        let metadata = fs::metadata(file_path)
            .await
            .map_err(|e| GapAnalysisError::FileRead {
                path: file_path.to_string_lossy().to_string(),
                error: e,
            })?;

        if metadata.len() > self.config.max_file_size_bytes {
            return Err(GapAnalysisError::FileTooLarge {
                size: metadata.len(),
                limit: self.config.max_file_size_bytes,
            });
        }

        // Read file content
        let content =
            fs::read_to_string(file_path)
                .await
                .map_err(|e| GapAnalysisError::FileRead {
                    path: file_path.to_string_lossy().to_string(),
                    error: e,
                })?;

        // Analyze content
        let mut gaps = Vec::new();

        if self.config.enable_todo_detection {
            gaps.extend(self.detect_todo_comments(&content, file_path));
        }

        if self.config.enable_docs_detection {
            gaps.extend(self.detect_missing_documentation(&content, file_path)?);
        }

        if self.config.enable_tech_detection {
            gaps.extend(self.detect_undocumented_technologies(&content, file_path)?);
        }

        if self.config.enable_api_detection {
            gaps.extend(self.detect_api_documentation_gaps(&content, file_path)?);
        }

        if self.config.enable_config_detection {
            gaps.extend(self.detect_configuration_gaps(&content, file_path)?);
        }

        // Filter by confidence threshold
        let pre_filter_count = gaps.len();
        gaps.retain(|gap| gap.confidence >= self.config.min_confidence_threshold);
        if gaps.len() != pre_filter_count {
            debug!(
                "Filtered {} gaps below confidence threshold {}",
                pre_filter_count - gaps.len(),
                self.config.min_confidence_threshold
            );
        }

        // Check performance
        let duration = start_time.elapsed();
        let timeout = Duration::from_millis(self.config.analysis_timeout_ms);
        if duration > timeout {
            warn!(
                "Gap analysis exceeded timeout for {}: {:?} > {:?}",
                file_path.display(),
                duration,
                timeout
            );
        }

        debug!(
            "Analyzed {} in {:?}, found {} gaps",
            file_path.display(),
            duration,
            gaps.len()
        );

        Ok(gaps)
    }

    /// Analyze a file event for knowledge gaps
    pub async fn analyze_file_event(
        &self,
        event: &FileEvent,
    ) -> Result<Vec<DetectedGap>, GapAnalysisError> {
        // Only analyze events that should trigger analysis
        if !event.should_trigger_analysis {
            return Ok(Vec::new());
        }

        self.analyze_file(&event.path).await
    }

    /// Detect TODO/FIXME/HACK comments
    fn detect_todo_comments(&self, content: &str, file_path: &Path) -> Vec<DetectedGap> {
        let mut gaps = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            if self.patterns.todo_patterns.is_match(line) {
                // Find which pattern matched and extract the content
                for extractor in &self.patterns.todo_extractors {
                    if let Some(captures) = extractor.captures(line) {
                        if let Some(todo_content) = captures.get(1) {
                            let gap = DetectedGap::new(
                                GapType::TodoComment,
                                file_path.to_path_buf(),
                                line_idx + 1,
                                line.trim().to_string(),
                                todo_content.as_str().trim().to_string(),
                                0.9, // High confidence for regex matches
                            );
                            gaps.push(gap);
                            break; // Only report first match per line
                        }
                    }
                }
            }
        }

        gaps
    }

    /// Detect missing documentation for functions and structs
    fn detect_missing_documentation(
        &self,
        content: &str,
        file_path: &Path,
    ) -> Result<Vec<DetectedGap>, GapAnalysisError> {
        let mut gaps = Vec::new();

        // Only analyze Rust files for now
        if file_path.extension().is_none_or(|ext| ext != "rs") {
            return Ok(gaps);
        }

        let lines: Vec<&str> = content.lines().collect();

        // Check public functions
        for captures in self.patterns.rust_function_patterns.captures_iter(content) {
            if let (Some(full_match), Some(fn_name)) = (captures.get(0), captures.get(3)) {
                let line_start = content[..captures.get(0).unwrap().start()]
                    .matches('\n')
                    .count();

                // Check if there's documentation above this function
                let has_doc = self.has_documentation_above(&lines, line_start);

                if !has_doc {
                    let gap = DetectedGap::new(
                        GapType::MissingDocumentation,
                        file_path.to_path_buf(),
                        line_start + 1,
                        full_match.as_str().trim().to_string(),
                        format!("Public function '{}' lacks documentation", fn_name.as_str()),
                        0.8,
                    )
                    .with_metadata("function_name", fn_name.as_str())
                    .with_metadata("item_type", "function");

                    gaps.push(gap);
                }
            }
        }

        // Check public structs
        for captures in self.patterns.rust_struct_patterns.captures_iter(content) {
            if let (Some(full_match), Some(struct_name)) = (captures.get(0), captures.get(2)) {
                let line_start = content[..captures.get(0).unwrap().start()]
                    .matches('\n')
                    .count();

                let has_doc = self.has_documentation_above(&lines, line_start);

                if !has_doc {
                    let gap = DetectedGap::new(
                        GapType::MissingDocumentation,
                        file_path.to_path_buf(),
                        line_start + 1,
                        full_match.as_str().trim().to_string(),
                        format!(
                            "Public struct '{}' lacks documentation",
                            struct_name.as_str()
                        ),
                        0.8,
                    )
                    .with_metadata("struct_name", struct_name.as_str())
                    .with_metadata("item_type", "struct");

                    gaps.push(gap);
                }
            }
        }

        Ok(gaps)
    }

    /// Check if there's documentation above a given line
    fn has_documentation_above(&self, lines: &[&str], line_idx: usize) -> bool {
        if line_idx == 0 {
            return false;
        }

        // Look up to 3 lines above for documentation
        let start_idx = line_idx.saturating_sub(3);

        for line in lines.iter().take(line_idx).skip(start_idx) {
            let line = line.trim();
            if line.starts_with("///") || line.starts_with("//!") {
                return true;
            }
            if line.starts_with("/**") || line.contains("*/") {
                return true;
            }
            if line.starts_with("#[doc") {
                return true;
            }
        }

        false
    }

    /// Detect undocumented technologies/dependencies
    fn detect_undocumented_technologies(
        &self,
        content: &str,
        file_path: &Path,
    ) -> Result<Vec<DetectedGap>, GapAnalysisError> {
        let mut gaps = Vec::new();

        // Only analyze Rust files for now
        if file_path.extension().is_none_or(|ext| ext != "rs") {
            return Ok(gaps);
        }

        // Extract use statements
        for captures in self.patterns.rust_use_patterns.captures_iter(content) {
            if let Some(crate_name) = captures.get(1) {
                let line_start = content[..captures.get(0).unwrap().start()]
                    .matches('\n')
                    .count();

                let crate_str = crate_name.as_str();
                let root_crate = crate_str.split("::").next().unwrap_or(crate_str);

                // Skip standard library and local crates
                if !self.is_external_crate(root_crate) {
                    continue;
                }

                let gap = DetectedGap::new(
                    GapType::UndocumentedTechnology,
                    file_path.to_path_buf(),
                    line_start + 1,
                    captures.get(0).unwrap().as_str().trim().to_string(),
                    format!("External crate '{root_crate}' may need documentation"),
                    0.7,
                )
                .with_metadata("crate_name", root_crate)
                .with_metadata("full_path", crate_str);

                gaps.push(gap);
            }
        }

        Ok(gaps)
    }

    /// Check if a crate name is likely an external dependency
    fn is_external_crate(&self, crate_name: &str) -> bool {
        // Skip standard library crates and common internal patterns
        !matches!(
            crate_name,
            "std" | "core" | "alloc" | "proc_macro" | "test" | "super" | "self" | "crate"
        ) && !crate_name.starts_with("crate::")
            && !crate_name.starts_with("super::")
            && !crate_name.starts_with("self::")
    }

    /// Detect API documentation gaps
    fn detect_api_documentation_gaps(
        &self,
        content: &str,
        file_path: &Path,
    ) -> Result<Vec<DetectedGap>, GapAnalysisError> {
        let mut gaps = Vec::new();

        // Only analyze Rust files for now
        if file_path.extension().is_none_or(|ext| ext != "rs") {
            return Ok(gaps);
        }

        // This is a simplified implementation - in a real system, you'd want more sophisticated analysis
        // For now, just flag public items without examples in their documentation

        let lines: Vec<&str> = content.lines().collect();

        for captures in self.patterns.rust_function_patterns.captures_iter(content) {
            if let Some(fn_name) = captures.get(3) {
                let line_start = content[..captures.get(0).unwrap().start()]
                    .matches('\n')
                    .count();

                if self.has_documentation_above(&lines, line_start) {
                    // Check if the documentation includes examples
                    if !self.has_examples_in_documentation(&lines, line_start) {
                        let gap = DetectedGap::new(
                            GapType::ApiDocumentationGap,
                            file_path.to_path_buf(),
                            line_start + 1,
                            captures.get(0).unwrap().as_str().trim().to_string(),
                            format!(
                                "Public function '{}' documentation lacks examples",
                                fn_name.as_str()
                            ),
                            0.6,
                        )
                        .with_metadata("function_name", fn_name.as_str())
                        .with_metadata("missing_element", "examples");

                        gaps.push(gap);
                    }
                }
            }
        }

        Ok(gaps)
    }

    /// Check if documentation includes examples
    fn has_examples_in_documentation(&self, lines: &[&str], line_idx: usize) -> bool {
        if line_idx == 0 {
            return false;
        }

        // Look up to 10 lines above for documentation with examples
        let start_idx = line_idx.saturating_sub(10);

        for line in lines.iter().take(line_idx).skip(start_idx) {
            let line = line.trim();
            if line.contains("```") || line.contains("Example") || line.contains("example") {
                return true;
            }
        }

        false
    }

    /// Detect configuration gaps
    fn detect_configuration_gaps(
        &self,
        content: &str,
        file_path: &Path,
    ) -> Result<Vec<DetectedGap>, GapAnalysisError> {
        let mut gaps = Vec::new();

        // Only analyze configuration files
        let is_config_file = file_path.extension().is_some_and(|ext| {
            matches!(
                ext.to_str(),
                Some("toml") | Some("yaml") | Some("yml") | Some("json")
            )
        });

        if !is_config_file {
            return Ok(gaps);
        }

        // For TOML files, look for configuration keys that might need documentation
        if file_path.extension().is_some_and(|ext| ext == "toml") {
            for captures in self.patterns.config_key_patterns.captures_iter(content) {
                if let Some(key_name) = captures.get(1) {
                    let line_start = content[..captures.get(0).unwrap().start()]
                        .matches('\n')
                        .count();

                    let gap = DetectedGap::new(
                        GapType::ConfigurationGap,
                        file_path.to_path_buf(),
                        line_start + 1,
                        captures.get(0).unwrap().as_str().trim().to_string(),
                        format!(
                            "Configuration key '{}' may need documentation",
                            key_name.as_str()
                        ),
                        0.5, // Lower confidence as this is heuristic
                    )
                    .with_metadata("config_key", key_name.as_str());

                    gaps.push(gap);
                }
            }
        }

        Ok(gaps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_gap_type_priority() {
        assert_eq!(GapType::TodoComment.priority(), 7);
        assert_eq!(GapType::ApiDocumentationGap.priority(), 9);
        assert_eq!(GapType::UndocumentedTechnology.priority(), 8);
    }

    #[test]
    fn test_config_should_analyze_file() {
        let config = GapAnalysisConfig::for_rust_project();

        assert!(config.should_analyze_file(&PathBuf::from("test.rs")));
        assert!(config.should_analyze_file(&PathBuf::from("README.md")));
        assert!(config.should_analyze_file(&PathBuf::from("Cargo.toml")));
        assert!(!config.should_analyze_file(&PathBuf::from("test.exe")));
        assert!(!config.should_analyze_file(&PathBuf::from("no_extension")));
    }

    #[tokio::test]
    async fn test_gap_analyzer_creation() {
        // FAILING TEST: GapAnalyzer should be creatable with valid configuration
        let config = GapAnalysisConfig::for_rust_project();
        let result = GapAnalyzer::new(config);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_detect_todo_comments() {
        // FAILING TEST: Should detect TODO comments in Rust code
        let analyzer = GapAnalyzer::for_rust_project().unwrap();
        let content = r#"
fn main() {
    // TODO: Implement error handling
    println!("Hello, world!");
    // FIXME: This is broken
    let x = 42;
}
"#;
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let gaps = analyzer.detect_todo_comments(content, &file_path);
        assert_eq!(gaps.len(), 2);
        assert_eq!(gaps[0].gap_type, GapType::TodoComment);
        assert_eq!(gaps[0].line_number, 3);
        assert!(gaps[0].description.contains("Implement error handling"));
    }

    #[tokio::test]
    async fn test_detect_missing_documentation() {
        // FAILING TEST: Should detect missing documentation for public functions
        let analyzer = GapAnalyzer::for_rust_project().unwrap();
        let content = r#"
pub fn undocumented_function() {
    println!("No docs");
}

/// This function has documentation
pub fn documented_function() {
    println!("Has docs");
}
"#;
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let gaps = analyzer
            .detect_missing_documentation(content, &file_path)
            .unwrap();
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].gap_type, GapType::MissingDocumentation);
        assert!(gaps[0].description.contains("undocumented_function"));
    }

    #[tokio::test]
    async fn test_detect_undocumented_technologies() {
        // FAILING TEST: Should detect external crates that might need documentation
        let analyzer = GapAnalyzer::for_rust_project().unwrap();
        let content = r#"
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::fs;
use super::module;
"#;
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let gaps = analyzer
            .detect_undocumented_technologies(content, &file_path)
            .unwrap();
        // Should detect serde and tokio but not std or super
        assert!(gaps.len() >= 2);
        let crate_names: Vec<_> = gaps
            .iter()
            .filter_map(|g| g.metadata.get("crate_name"))
            .collect();
        assert!(crate_names.contains(&&"serde".to_string()));
        assert!(crate_names.contains(&&"tokio".to_string()));
    }

    #[tokio::test]
    async fn test_analyze_file_performance() {
        // FAILING TEST: File analysis should complete within performance requirements
        let config = GapAnalysisConfig::for_rust_project().with_timeout_ms(100);
        let analyzer = GapAnalyzer::new(config).unwrap();

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        // Create a reasonably sized test file
        let content = "fn test() {}\n".repeat(1000);
        fs::write(&file_path, content).await.unwrap();

        let start = Instant::now();
        let result = analyzer.analyze_file(&file_path).await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration < Duration::from_millis(500)); // Should be faster than timeout
    }

    #[tokio::test]
    async fn test_file_event_integration() {
        // FAILING TEST: Should analyze file events and detect gaps
        use crate::proactive::{EventType, FileEvent};

        let analyzer = GapAnalyzer::for_rust_project().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let content = "// TODO: Add tests\npub fn untested_function() {}";
        fs::write(&file_path, content).await.unwrap();

        let event = FileEvent::new(file_path.clone(), EventType::Create);
        let gaps = analyzer.analyze_file_event(&event).await.unwrap();

        assert!(!gaps.is_empty());
        // Should detect both TODO and missing documentation
        let gap_types: HashSet<_> = gaps.iter().map(|g| &g.gap_type).collect();
        assert!(gap_types.contains(&GapType::TodoComment));
        assert!(gap_types.contains(&GapType::MissingDocumentation));
    }
}
