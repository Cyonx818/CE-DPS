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

// ABOUTME: Configurable gap detection rules and thresholds for proactive research mode
//! This module provides comprehensive configuration for gap detection rules, semantic analysis
//! thresholds, and system behavior tuning. It allows users to customize gap detection behavior,
//! validation criteria, and analysis parameters for different scenarios and performance requirements.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during configuration handling
#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("Invalid threshold value: {field} must be between {min} and {max}, got {value}")]
    InvalidThreshold {
        field: String,
        min: f64,
        max: f64,
        value: f64,
    },

    #[error("Invalid regex pattern in {context}: {pattern} - {error}")]
    InvalidRegex {
        context: String,
        pattern: String,
        error: String,
    },

    #[error("Invalid file extension: {extension}")]
    InvalidFileExtension { extension: String },

    #[error("Invalid priority value: {priority} must be between 1 and 10")]
    InvalidPriority { priority: u8 },

    #[error("Invalid performance limit: {field} must be greater than 0, got {value}")]
    InvalidPerformanceLimit { field: String, value: u64 },

    #[error("Rule validation failed: {rule_name} - {error}")]
    RuleValidation { rule_name: String, error: String },

    #[error("Configuration preset not found: {preset}")]
    PresetNotFound { preset: String },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Comprehensive gap detection configuration with rules and thresholds
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GapDetectionConfig {
    /// Basic detection settings
    pub detection_settings: DetectionSettings,
    /// Configurable detection rules
    pub detection_rules: DetectionRules,
    /// Semantic analysis configuration
    pub semantic_config: SemanticConfig,
    /// Performance tuning parameters
    pub performance_config: PerformanceConfig,
    /// Priority scoring configuration
    pub priority_config: PriorityConfig,
    /// Filtering and validation rules
    pub filtering_config: FilteringConfig,
}

/// Basic detection settings and thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionSettings {
    /// Global minimum confidence threshold (0.0-1.0)
    pub min_confidence_threshold: f64,
    /// Maximum file size to analyze (in bytes)
    pub max_file_size_bytes: u64,
    /// Analysis timeout per file (in milliseconds)
    pub analysis_timeout_ms: u64,
    /// File extensions to analyze
    pub supported_extensions: HashSet<String>,
    /// Directories to exclude from analysis
    pub excluded_directories: HashSet<PathBuf>,
    /// Files to exclude from analysis (patterns)
    pub excluded_file_patterns: Vec<String>,
}

/// Configurable detection rules for different gap types
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DetectionRules {
    /// TODO comment detection rules
    pub todo_rules: TodoDetectionRules,
    /// Documentation gap detection rules
    pub documentation_rules: DocumentationRules,
    /// Technology detection rules
    pub technology_rules: TechnologyRules,
    /// API documentation rules
    pub api_rules: ApiDocumentationRules,
    /// Configuration gap rules
    pub config_rules: ConfigurationRules,
}

/// Rules for detecting TODO comments and similar markers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoDetectionRules {
    /// Enable TODO comment detection
    pub enabled: bool,
    /// Custom regex patterns for TODO detection
    pub custom_patterns: Vec<String>,
    /// Keywords that indicate TODO-like comments
    pub todo_keywords: Vec<String>,
    /// Minimum confidence for TODO detection (0.0-1.0)
    pub min_confidence: f64,
    /// Priority boost for urgent TODO keywords
    pub urgent_keyword_boost: f64,
    /// Urgent keywords (FIXME, URGENT, etc.)
    pub urgent_keywords: Vec<String>,
}

/// Rules for detecting missing documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationRules {
    /// Enable documentation gap detection
    pub enabled: bool,
    /// Require documentation for public functions
    pub require_public_function_docs: bool,
    /// Require documentation for public structs/classes
    pub require_public_struct_docs: bool,
    /// Require documentation for public modules
    pub require_public_module_docs: bool,
    /// Minimum documentation length (characters)
    pub min_doc_length: usize,
    /// Documentation patterns to recognize as valid
    pub valid_doc_patterns: Vec<String>,
    /// Confidence threshold for documentation gaps (0.0-1.0)
    pub min_confidence: f64,
}

/// Rules for detecting undocumented technologies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnologyRules {
    /// Enable technology detection
    pub enabled: bool,
    /// Known internal/standard library patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Minimum confidence for technology detection (0.0-1.0)
    pub min_confidence: f64,
    /// Technology patterns to specifically look for
    pub technology_patterns: Vec<String>,
    /// Confidence boost for known external libraries
    pub external_library_boost: f64,
}

/// Rules for API documentation gap detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDocumentationRules {
    /// Enable API documentation gap detection
    pub enabled: bool,
    /// Require examples in API documentation
    pub require_examples: bool,
    /// Require parameter documentation
    pub require_parameter_docs: bool,
    /// Require return value documentation
    pub require_return_docs: bool,
    /// Example keywords to look for
    pub example_keywords: Vec<String>,
    /// Minimum confidence for API gaps (0.0-1.0)
    pub min_confidence: f64,
}

/// Rules for configuration gap detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationRules {
    /// Enable configuration gap detection
    pub enabled: bool,
    /// Configuration file patterns
    pub config_file_patterns: Vec<String>,
    /// Configuration key patterns
    pub config_key_patterns: Vec<String>,
    /// Minimum confidence for config gaps (0.0-1.0)
    pub min_confidence: f64,
    /// Require documentation for configuration keys
    pub require_key_documentation: bool,
}

/// Semantic analysis configuration and thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticConfig {
    /// Enable semantic analysis
    pub enabled: bool,
    /// Gap validation threshold (0.0-1.0)
    pub gap_validation_threshold: f64,
    /// Related content discovery threshold (0.0-1.0)
    pub related_content_threshold: f64,
    /// Maximum related documents to consider
    pub max_related_documents: usize,
    /// Minimum content length for semantic analysis
    pub min_content_length: usize,
    /// Batch size for vector queries
    pub batch_size: usize,
    /// Maximum semantic analysis time (milliseconds)
    pub max_analysis_time_ms: u64,
    /// Semantic similarity weight in priority calculation
    pub semantic_priority_weight: f64,
    /// Custom semantic search keywords for different gap types
    pub gap_type_keywords: HashMap<String, Vec<String>>,
}

/// Performance configuration and limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum total analysis time per file (milliseconds)
    pub max_total_time_ms: u64,
    /// Maximum concurrent file analyses
    pub max_concurrent_analyses: usize,
    /// Memory usage limit per analysis (bytes)
    pub max_memory_per_analysis_bytes: u64,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Performance alert threshold (milliseconds)
    pub performance_alert_threshold_ms: u64,
    /// Maximum regex compilation cache size
    pub max_regex_cache_size: usize,
}

/// Priority scoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityConfig {
    /// Base priority scores for each gap type
    pub base_priorities: HashMap<String, u8>,
    /// Priority boost factors
    pub priority_boosts: PriorityBoosts,
    /// Maximum priority score (ceiling)
    pub max_priority: u8,
    /// Minimum priority score (floor)
    pub min_priority: u8,
    /// Custom priority rules
    pub custom_priority_rules: Vec<CustomPriorityRule>,
}

/// Priority boost configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityBoosts {
    /// Boost for files modified recently
    pub recent_modification_boost: f64,
    /// Boost for files in critical paths
    pub critical_path_boost: f64,
    /// Boost for high-confidence gaps
    pub high_confidence_boost: f64,
    /// Boost for urgent keywords
    pub urgent_keyword_boost: f64,
    /// Penalty for low-confidence gaps
    pub low_confidence_penalty: f64,
}

/// Custom priority rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPriorityRule {
    /// Rule name for identification
    pub name: String,
    /// File path pattern (regex)
    pub file_pattern: Option<String>,
    /// Gap type pattern
    pub gap_type_pattern: Option<String>,
    /// Content pattern (regex)
    pub content_pattern: Option<String>,
    /// Priority adjustment (can be negative)
    pub priority_adjustment: i8,
    /// Rule description
    pub description: String,
}

/// Filtering and validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilteringConfig {
    /// Enable gap filtering
    pub enabled: bool,
    /// Quality thresholds for gap acceptance
    pub quality_thresholds: QualityThresholds,
    /// Exclusion rules
    pub exclusion_rules: ExclusionRules,
    /// Validation rules
    pub validation_rules: ValidationRules,
    /// Maximum gaps per file
    pub max_gaps_per_file: usize,
    /// Duplicate detection settings
    pub duplicate_detection: DuplicateDetectionConfig,
}

/// Quality thresholds for gap filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    /// Minimum overall quality score (0.0-1.0)
    pub min_quality_score: f64,
    /// Minimum content length for valid gaps
    pub min_content_length: usize,
    /// Maximum content length for valid gaps
    pub max_content_length: usize,
    /// Minimum description length
    pub min_description_length: usize,
}

/// Rules for excluding gaps from results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExclusionRules {
    /// Patterns to exclude from gap detection
    pub exclude_patterns: Vec<String>,
    /// File paths to exclude
    pub exclude_file_paths: Vec<PathBuf>,
    /// Gap types to exclude
    pub exclude_gap_types: Vec<String>,
    /// Keywords that mark gaps as not actionable
    pub non_actionable_keywords: Vec<String>,
}

/// Rules for validating detected gaps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    /// Require minimum context length
    pub min_context_length: usize,
    /// Require valid line numbers
    pub require_valid_line_numbers: bool,
    /// Validate file existence
    pub validate_file_existence: bool,
    /// Custom validation patterns
    pub custom_validation_patterns: Vec<String>,
}

/// Duplicate detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateDetectionConfig {
    /// Enable duplicate detection
    pub enabled: bool,
    /// Similarity threshold for duplicate detection (0.0-1.0)
    pub similarity_threshold: f64,
    /// Content similarity weight
    pub content_similarity_weight: f64,
    /// Location similarity weight
    pub location_similarity_weight: f64,
    /// Type similarity weight
    pub type_similarity_weight: f64,
}

impl Default for DetectionSettings {
    fn default() -> Self {
        Self {
            min_confidence_threshold: 0.6,
            max_file_size_bytes: 50 * 1024 * 1024, // 50MB
            analysis_timeout_ms: 500,
            supported_extensions: [
                "rs", "md", "yaml", "yml", "toml", "json", "py", "js", "ts", "go", "java", "cpp",
                "c", "h",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            excluded_directories: [
                PathBuf::from("target"),
                PathBuf::from("node_modules"),
                PathBuf::from(".git"),
                PathBuf::from("build"),
                PathBuf::from("dist"),
            ]
            .into_iter()
            .collect(),
            excluded_file_patterns: vec![
                r".*\.lock$".to_string(),
                r".*\.tmp$".to_string(),
                r".*\.backup$".to_string(),
            ],
        }
    }
}

impl Default for TodoDetectionRules {
    fn default() -> Self {
        Self {
            enabled: true,
            custom_patterns: Vec::new(),
            todo_keywords: vec![
                "TODO".to_string(),
                "FIXME".to_string(),
                "HACK".to_string(),
                "BUG".to_string(),
                "NOTE".to_string(),
                "REVIEW".to_string(),
            ],
            min_confidence: 0.8,
            urgent_keyword_boost: 0.3,
            urgent_keywords: vec![
                "URGENT".to_string(),
                "CRITICAL".to_string(),
                "SECURITY".to_string(),
                "FIXME".to_string(),
            ],
        }
    }
}

impl Default for DocumentationRules {
    fn default() -> Self {
        Self {
            enabled: true,
            require_public_function_docs: true,
            require_public_struct_docs: true,
            require_public_module_docs: false,
            min_doc_length: 20,
            valid_doc_patterns: vec![
                r"///.*".to_string(),
                r"//!.*".to_string(),
                r"/\*\*.*\*/".to_string(),
                r"#\[doc.*\]".to_string(),
            ],
            min_confidence: 0.7,
        }
    }
}

impl Default for TechnologyRules {
    fn default() -> Self {
        Self {
            enabled: true,
            exclude_patterns: vec![
                r"^std::.*".to_string(),
                r"^core::.*".to_string(),
                r"^alloc::.*".to_string(),
                r"^super::.*".to_string(),
                r"^self::.*".to_string(),
                r"^crate::.*".to_string(),
            ],
            min_confidence: 0.6,
            technology_patterns: vec![
                r"use\s+([a-zA-Z_][a-zA-Z0-9_]*)::".to_string(),
                r"import\s+([a-zA-Z_][a-zA-Z0-9_]*)".to_string(),
                r"#include\s*<([^>]+)>".to_string(),
            ],
            external_library_boost: 0.2,
        }
    }
}

impl Default for ApiDocumentationRules {
    fn default() -> Self {
        Self {
            enabled: true,
            require_examples: true,
            require_parameter_docs: false,
            require_return_docs: false,
            example_keywords: vec![
                "example".to_string(),
                "Example".to_string(),
                "```".to_string(),
                "sample".to_string(),
                "usage".to_string(),
            ],
            min_confidence: 0.6,
        }
    }
}

impl Default for ConfigurationRules {
    fn default() -> Self {
        Self {
            enabled: true,
            config_file_patterns: vec![
                r".*\.toml$".to_string(),
                r".*\.yaml$".to_string(),
                r".*\.yml$".to_string(),
                r".*\.json$".to_string(),
                r".*\.conf$".to_string(),
                r".*\.config$".to_string(),
            ],
            config_key_patterns: vec![
                r"^([a-zA-Z_][a-zA-Z0-9_]*)\s*=".to_string(),
                r"^(\s*)([a-zA-Z_][a-zA-Z0-9_]*):".to_string(),
            ],
            min_confidence: 0.5,
            require_key_documentation: false,
        }
    }
}

impl Default for SemanticConfig {
    fn default() -> Self {
        let mut gap_type_keywords = HashMap::new();
        gap_type_keywords.insert(
            "TodoComment".to_string(),
            vec![
                "todo".to_string(),
                "implementation".to_string(),
                "task".to_string(),
                "feature".to_string(),
            ],
        );
        gap_type_keywords.insert(
            "MissingDocumentation".to_string(),
            vec![
                "documentation".to_string(),
                "docs".to_string(),
                "guide".to_string(),
                "explanation".to_string(),
            ],
        );
        gap_type_keywords.insert(
            "UndocumentedTechnology".to_string(),
            vec![
                "technology".to_string(),
                "library".to_string(),
                "dependency".to_string(),
                "usage".to_string(),
            ],
        );
        gap_type_keywords.insert(
            "ApiDocumentationGap".to_string(),
            vec![
                "api".to_string(),
                "interface".to_string(),
                "examples".to_string(),
                "usage".to_string(),
            ],
        );
        gap_type_keywords.insert(
            "ConfigurationGap".to_string(),
            vec![
                "configuration".to_string(),
                "settings".to_string(),
                "options".to_string(),
                "setup".to_string(),
            ],
        );

        Self {
            enabled: true,
            gap_validation_threshold: 0.8,
            related_content_threshold: 0.7,
            max_related_documents: 10,
            min_content_length: 50,
            batch_size: 20,
            max_analysis_time_ms: 100,
            semantic_priority_weight: 0.3,
            gap_type_keywords,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_total_time_ms: 500,
            max_concurrent_analyses: 4,
            max_memory_per_analysis_bytes: 100 * 1024 * 1024, // 100MB
            enable_performance_monitoring: true,
            performance_alert_threshold_ms: 1000,
            max_regex_cache_size: 100,
        }
    }
}

impl Default for PriorityConfig {
    fn default() -> Self {
        let mut base_priorities = HashMap::new();
        base_priorities.insert("TodoComment".to_string(), 7);
        base_priorities.insert("MissingDocumentation".to_string(), 6);
        base_priorities.insert("UndocumentedTechnology".to_string(), 8);
        base_priorities.insert("ApiDocumentationGap".to_string(), 9);
        base_priorities.insert("ConfigurationGap".to_string(), 5);

        Self {
            base_priorities,
            priority_boosts: PriorityBoosts::default(),
            max_priority: 10,
            min_priority: 1,
            custom_priority_rules: Vec::new(),
        }
    }
}

impl Default for PriorityBoosts {
    fn default() -> Self {
        Self {
            recent_modification_boost: 0.2,
            critical_path_boost: 0.5,
            high_confidence_boost: 0.3,
            urgent_keyword_boost: 0.4,
            low_confidence_penalty: 0.2,
        }
    }
}

impl Default for FilteringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            quality_thresholds: QualityThresholds::default(),
            exclusion_rules: ExclusionRules::default(),
            validation_rules: ValidationRules::default(),
            max_gaps_per_file: 50,
            duplicate_detection: DuplicateDetectionConfig::default(),
        }
    }
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            min_quality_score: 0.6,
            min_content_length: 10,
            max_content_length: 5000,
            min_description_length: 5,
        }
    }
}

impl Default for ExclusionRules {
    fn default() -> Self {
        Self {
            exclude_patterns: vec![
                r"(?i)test.*only".to_string(),
                r"(?i)debug.*only".to_string(),
                r"(?i)temporary".to_string(),
            ],
            exclude_file_paths: vec![PathBuf::from("target/"), PathBuf::from("tests/fixtures/")],
            exclude_gap_types: Vec::new(),
            non_actionable_keywords: vec![
                "maybe".to_string(),
                "eventually".to_string(),
                "someday".to_string(),
                "nice to have".to_string(),
            ],
        }
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            min_context_length: 5,
            require_valid_line_numbers: true,
            validate_file_existence: true,
            custom_validation_patterns: Vec::new(),
        }
    }
}

impl Default for DuplicateDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            similarity_threshold: 0.8,
            content_similarity_weight: 0.6,
            location_similarity_weight: 0.3,
            type_similarity_weight: 0.1,
        }
    }
}

impl GapDetectionConfig {
    /// Create configuration for Rust projects with sensible defaults
    pub fn for_rust_project() -> Self {
        let mut config = Self::default();
        config.detection_settings.supported_extensions =
            ["rs", "md", "toml", "yaml", "yml", "json"]
                .into_iter()
                .map(String::from)
                .collect();
        config
    }

    /// Create performance-optimized configuration
    pub fn for_performance() -> Self {
        let mut config = Self::default();
        config.performance_config.max_total_time_ms = 300;
        config.semantic_config.max_analysis_time_ms = 50;
        config.semantic_config.batch_size = 10;
        config.semantic_config.max_related_documents = 5;
        config.filtering_config.max_gaps_per_file = 20;
        config
    }

    /// Create accuracy-optimized configuration
    pub fn for_accuracy() -> Self {
        let mut config = Self::default();
        config.performance_config.max_total_time_ms = 1000;
        config.semantic_config.max_analysis_time_ms = 200;
        config.semantic_config.gap_validation_threshold = 0.7;
        config.semantic_config.related_content_threshold = 0.6;
        config.detection_settings.min_confidence_threshold = 0.5;
        config.filtering_config.quality_thresholds.min_quality_score = 0.5;
        config
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigurationError> {
        // Validate thresholds
        self.validate_thresholds()?;

        // Validate regex patterns
        self.validate_regex_patterns()?;

        // Validate performance limits
        self.validate_performance_limits()?;

        // Validate priority configuration
        self.validate_priority_config()?;

        Ok(())
    }

    /// Validate all threshold values
    fn validate_thresholds(&self) -> Result<(), ConfigurationError> {
        let thresholds = [
            (
                "min_confidence_threshold",
                self.detection_settings.min_confidence_threshold,
            ),
            (
                "gap_validation_threshold",
                self.semantic_config.gap_validation_threshold,
            ),
            (
                "related_content_threshold",
                self.semantic_config.related_content_threshold,
            ),
            (
                "semantic_priority_weight",
                self.semantic_config.semantic_priority_weight,
            ),
            (
                "todo_min_confidence",
                self.detection_rules.todo_rules.min_confidence,
            ),
            (
                "documentation_min_confidence",
                self.detection_rules.documentation_rules.min_confidence,
            ),
            (
                "technology_min_confidence",
                self.detection_rules.technology_rules.min_confidence,
            ),
            (
                "api_min_confidence",
                self.detection_rules.api_rules.min_confidence,
            ),
            (
                "config_min_confidence",
                self.detection_rules.config_rules.min_confidence,
            ),
        ];

        for (field, value) in thresholds {
            if !(0.0..=1.0).contains(&value) {
                return Err(ConfigurationError::InvalidThreshold {
                    field: field.to_string(),
                    min: 0.0,
                    max: 1.0,
                    value,
                });
            }
        }

        Ok(())
    }

    /// Validate regex patterns in the configuration
    fn validate_regex_patterns(&self) -> Result<(), ConfigurationError> {
        // Validate TODO patterns
        for pattern in &self.detection_rules.todo_rules.custom_patterns {
            Regex::new(pattern).map_err(|e| ConfigurationError::InvalidRegex {
                context: "TODO custom patterns".to_string(),
                pattern: pattern.clone(),
                error: e.to_string(),
            })?;
        }

        // Validate documentation patterns
        for pattern in &self.detection_rules.documentation_rules.valid_doc_patterns {
            Regex::new(pattern).map_err(|e| ConfigurationError::InvalidRegex {
                context: "Documentation patterns".to_string(),
                pattern: pattern.clone(),
                error: e.to_string(),
            })?;
        }

        // Validate technology patterns
        for pattern in &self.detection_rules.technology_rules.technology_patterns {
            Regex::new(pattern).map_err(|e| ConfigurationError::InvalidRegex {
                context: "Technology patterns".to_string(),
                pattern: pattern.clone(),
                error: e.to_string(),
            })?;
        }

        // Validate exclusion patterns
        for pattern in &self.detection_settings.excluded_file_patterns {
            Regex::new(pattern).map_err(|e| ConfigurationError::InvalidRegex {
                context: "Excluded file patterns".to_string(),
                pattern: pattern.clone(),
                error: e.to_string(),
            })?;
        }

        Ok(())
    }

    /// Validate performance limits
    fn validate_performance_limits(&self) -> Result<(), ConfigurationError> {
        let limits = [
            (
                "max_file_size_bytes",
                self.detection_settings.max_file_size_bytes,
            ),
            (
                "analysis_timeout_ms",
                self.detection_settings.analysis_timeout_ms,
            ),
            (
                "max_total_time_ms",
                self.performance_config.max_total_time_ms,
            ),
            (
                "max_analysis_time_ms",
                self.semantic_config.max_analysis_time_ms,
            ),
            (
                "max_memory_per_analysis_bytes",
                self.performance_config.max_memory_per_analysis_bytes,
            ),
        ];

        for (field, value) in limits {
            if value == 0 {
                return Err(ConfigurationError::InvalidPerformanceLimit {
                    field: field.to_string(),
                    value,
                });
            }
        }

        Ok(())
    }

    /// Validate priority configuration
    fn validate_priority_config(&self) -> Result<(), ConfigurationError> {
        // Validate base priorities
        for priority in self.priority_config.base_priorities.values() {
            if !(1..=10).contains(priority) {
                return Err(ConfigurationError::InvalidPriority {
                    priority: *priority,
                });
            }
        }

        // Validate min/max priority bounds
        if !(1..=10).contains(&self.priority_config.min_priority) {
            return Err(ConfigurationError::InvalidPriority {
                priority: self.priority_config.min_priority,
            });
        }

        if !(1..=10).contains(&self.priority_config.max_priority) {
            return Err(ConfigurationError::InvalidPriority {
                priority: self.priority_config.max_priority,
            });
        }

        if self.priority_config.min_priority > self.priority_config.max_priority {
            return Err(ConfigurationError::RuleValidation {
                rule_name: "priority_bounds".to_string(),
                error: "min_priority cannot be greater than max_priority".to_string(),
            });
        }

        Ok(())
    }

    /// Save configuration to a JSON file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), ConfigurationError> {
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Load configuration from a JSON file
    pub fn load_from_file(path: &PathBuf) -> Result<Self, ConfigurationError> {
        let contents = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    /// Merge with another configuration (other takes precedence)
    pub fn merge_with(&mut self, other: GapDetectionConfig) -> Result<(), ConfigurationError> {
        // Merge detection settings
        self.detection_settings.min_confidence_threshold =
            other.detection_settings.min_confidence_threshold;
        self.detection_settings.max_file_size_bytes = other.detection_settings.max_file_size_bytes;
        self.detection_settings.analysis_timeout_ms = other.detection_settings.analysis_timeout_ms;
        self.detection_settings.supported_extensions =
            other.detection_settings.supported_extensions;
        self.detection_settings.excluded_directories =
            other.detection_settings.excluded_directories;
        self.detection_settings.excluded_file_patterns =
            other.detection_settings.excluded_file_patterns;

        // Merge detection rules
        self.detection_rules = other.detection_rules;

        // Merge semantic config
        self.semantic_config = other.semantic_config;

        // Merge performance config
        self.performance_config = other.performance_config;

        // Merge priority config
        self.priority_config = other.priority_config;

        // Merge filtering config
        self.filtering_config = other.filtering_config;

        // Validate merged configuration
        self.validate()?;

        Ok(())
    }

    /// Create a lightweight configuration preset by name
    pub fn preset(name: &str) -> Result<Self, ConfigurationError> {
        match name {
            "rust" => Ok(Self::for_rust_project()),
            "performance" => Ok(Self::for_performance()),
            "accuracy" => Ok(Self::for_accuracy()),
            "minimal" => Ok(Self::minimal()),
            "comprehensive" => Ok(Self::comprehensive()),
            _ => Err(ConfigurationError::PresetNotFound {
                preset: name.to_string(),
            }),
        }
    }

    /// Create minimal configuration for basic gap detection
    pub fn minimal() -> Self {
        let mut config = Self::default();
        config.semantic_config.enabled = false;
        config.detection_rules.technology_rules.enabled = false;
        config.detection_rules.api_rules.enabled = false;
        config.detection_rules.config_rules.enabled = false;
        config.filtering_config.duplicate_detection.enabled = false;
        config.performance_config.max_total_time_ms = 100;
        config
    }

    /// Create comprehensive configuration for thorough analysis
    pub fn comprehensive() -> Self {
        let mut config = Self::for_accuracy();
        config
            .detection_rules
            .documentation_rules
            .require_public_module_docs = true;
        config.detection_rules.api_rules.require_parameter_docs = true;
        config.detection_rules.api_rules.require_return_docs = true;
        config
            .detection_rules
            .config_rules
            .require_key_documentation = true;
        config.semantic_config.max_related_documents = 20;
        config.filtering_config.quality_thresholds.min_quality_score = 0.4;
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config_creation() {
        let config = GapDetectionConfig::default();
        assert!(config.detection_settings.min_confidence_threshold > 0.0);
        assert!(config.detection_settings.min_confidence_threshold <= 1.0);
        assert!(config.semantic_config.enabled);
        assert!(config.detection_rules.todo_rules.enabled);
    }

    #[test]
    fn test_config_validation_valid() {
        let config = GapDetectionConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_threshold() {
        let mut config = GapDetectionConfig::default();
        config.detection_settings.min_confidence_threshold = 1.5; // Invalid

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ConfigurationError::InvalidThreshold { .. })
        ));
    }

    #[test]
    fn test_config_validation_invalid_regex() {
        let mut config = GapDetectionConfig::default();
        config
            .detection_rules
            .todo_rules
            .custom_patterns
            .push("[invalid regex".to_string());

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ConfigurationError::InvalidRegex { .. })
        ));
    }

    #[test]
    fn test_config_validation_invalid_performance_limit() {
        let mut config = GapDetectionConfig::default();
        config.performance_config.max_total_time_ms = 0; // Invalid

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ConfigurationError::InvalidPerformanceLimit { .. })
        ));
    }

    #[test]
    fn test_config_validation_invalid_priority() {
        let mut config = GapDetectionConfig::default();
        config
            .priority_config
            .base_priorities
            .insert("TodoComment".to_string(), 15); // Invalid

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ConfigurationError::InvalidPriority { .. })
        ));
    }

    #[test]
    fn test_rust_project_config() {
        let config = GapDetectionConfig::for_rust_project();
        assert!(config
            .detection_settings
            .supported_extensions
            .contains("rs"));
        assert!(config
            .detection_settings
            .supported_extensions
            .contains("toml"));
        assert!(!config
            .detection_settings
            .supported_extensions
            .contains("py"));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_performance_config() {
        let config = GapDetectionConfig::for_performance();
        assert!(
            config.performance_config.max_total_time_ms
                < GapDetectionConfig::default()
                    .performance_config
                    .max_total_time_ms
        );
        assert!(
            config.semantic_config.max_related_documents
                < GapDetectionConfig::default()
                    .semantic_config
                    .max_related_documents
        );
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_accuracy_config() {
        let config = GapDetectionConfig::for_accuracy();
        assert!(
            config.performance_config.max_total_time_ms
                > GapDetectionConfig::default()
                    .performance_config
                    .max_total_time_ms
        );
        assert!(
            config.semantic_config.gap_validation_threshold
                < GapDetectionConfig::default()
                    .semantic_config
                    .gap_validation_threshold
        );
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_minimal_config() {
        let config = GapDetectionConfig::minimal();
        assert!(!config.semantic_config.enabled);
        assert!(!config.detection_rules.technology_rules.enabled);
        assert!(!config.detection_rules.api_rules.enabled);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_comprehensive_config() {
        let config = GapDetectionConfig::comprehensive();
        assert!(
            config
                .detection_rules
                .documentation_rules
                .require_public_module_docs
        );
        assert!(config.detection_rules.api_rules.require_parameter_docs);
        assert!(
            config
                .detection_rules
                .config_rules
                .require_key_documentation
        );
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_presets() {
        assert!(GapDetectionConfig::preset("rust").is_ok());
        assert!(GapDetectionConfig::preset("performance").is_ok());
        assert!(GapDetectionConfig::preset("accuracy").is_ok());
        assert!(GapDetectionConfig::preset("minimal").is_ok());
        assert!(GapDetectionConfig::preset("comprehensive").is_ok());

        let result = GapDetectionConfig::preset("nonexistent");
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ConfigurationError::PresetNotFound { .. })
        ));
    }

    #[test]
    fn test_save_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.json");

        let original_config = GapDetectionConfig::for_rust_project();
        original_config.save_to_file(&config_path).unwrap();

        let loaded_config = GapDetectionConfig::load_from_file(&config_path).unwrap();

        // Compare key fields
        assert_eq!(
            original_config.detection_settings.min_confidence_threshold,
            loaded_config.detection_settings.min_confidence_threshold
        );
        assert_eq!(
            original_config.detection_settings.supported_extensions,
            loaded_config.detection_settings.supported_extensions
        );
    }

    #[test]
    fn test_config_merge() {
        let mut base_config = GapDetectionConfig::default();
        let mut override_config = GapDetectionConfig::default();

        override_config.detection_settings.min_confidence_threshold = 0.9;
        override_config.semantic_config.enabled = false;

        base_config.merge_with(override_config).unwrap();

        assert_eq!(base_config.detection_settings.min_confidence_threshold, 0.9);
        assert!(!base_config.semantic_config.enabled);
    }

    #[test]
    fn test_config_serialization() {
        let config = GapDetectionConfig::for_rust_project();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: GapDetectionConfig = serde_json::from_str(&serialized).unwrap();

        assert!(deserialized.validate().is_ok());
        assert_eq!(
            config.detection_settings.min_confidence_threshold,
            deserialized.detection_settings.min_confidence_threshold
        );
    }

    #[test]
    fn test_priority_boost_configuration() {
        let config = GapDetectionConfig::default();
        assert!(config.priority_config.priority_boosts.urgent_keyword_boost > 0.0);
        assert!(config.priority_config.priority_boosts.high_confidence_boost > 0.0);
        assert!(
            config
                .priority_config
                .priority_boosts
                .low_confidence_penalty
                > 0.0
        );
    }

    #[test]
    fn test_filtering_configuration() {
        let config = GapDetectionConfig::default();
        assert!(config.filtering_config.enabled);
        assert!(config.filtering_config.duplicate_detection.enabled);
        assert!(config.filtering_config.quality_thresholds.min_quality_score > 0.0);
        assert!(config.filtering_config.max_gaps_per_file > 0);
    }

    #[test]
    fn test_semantic_config_keywords() {
        let config = GapDetectionConfig::default();
        assert!(config
            .semantic_config
            .gap_type_keywords
            .contains_key("TodoComment"));
        assert!(config
            .semantic_config
            .gap_type_keywords
            .contains_key("MissingDocumentation"));

        let todo_keywords = config
            .semantic_config
            .gap_type_keywords
            .get("TodoComment")
            .unwrap();
        assert!(todo_keywords.contains(&"todo".to_string()));
        assert!(todo_keywords.contains(&"implementation".to_string()));
    }

    #[test]
    fn test_detection_rules_defaults() {
        let rules = DetectionRules::default();
        assert!(rules.todo_rules.enabled);
        assert!(rules.documentation_rules.enabled);
        assert!(rules.technology_rules.enabled);
        assert!(rules.api_rules.enabled);
        assert!(rules.config_rules.enabled);

        assert!(!rules.todo_rules.todo_keywords.is_empty());
        assert!(!rules.documentation_rules.valid_doc_patterns.is_empty());
        assert!(!rules.technology_rules.exclude_patterns.is_empty());
    }

    #[test]
    fn test_performance_config_defaults() {
        let perf_config = PerformanceConfig::default();
        assert!(perf_config.enable_performance_monitoring);
        assert!(perf_config.max_total_time_ms > 0);
        assert!(perf_config.max_concurrent_analyses > 0);
        assert!(perf_config.max_memory_per_analysis_bytes > 0);
    }
}
