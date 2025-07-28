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

// ABOUTME: Configurable gap analyzer that integrates configuration system with existing detection
//! This module provides a configurable wrapper around the existing gap analyzer that uses
//! the comprehensive configuration system to control detection behavior, rules, and thresholds.

use crate::proactive::{
    ConfigurationError, DetectedGap, FileEvent, GapAnalysisConfig, GapAnalysisError, GapAnalyzer,
    GapDetectionConfig, SemanticAnalysisConfig, SemanticAnalysisError, SemanticGapAnalysis,
    SemanticGapAnalyzer,
};
use fortitude_core::vector::SemanticSearchOperations;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use tracing::{debug, info, instrument};

/// Errors that can occur during configurable gap analysis
#[derive(Error, Debug)]
pub enum ConfigurableAnalysisError {
    #[error("Gap analysis error: {0}")]
    GapAnalysis(#[from] GapAnalysisError),

    #[error("Semantic analysis error: {0}")]
    SemanticAnalysis(#[from] SemanticAnalysisError),

    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigurationError),

    #[error("Rule evaluation failed: {rule} - {error}")]
    RuleEvaluation { rule: String, error: String },

    #[error("Priority calculation failed: {0}")]
    PriorityCalculation(String),

    #[error("Filtering failed: {0}")]
    Filtering(String),
}

/// Result of configurable gap analysis with enhanced metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurableAnalysisResult {
    /// Detected gaps after filtering and rule application
    pub filtered_gaps: Vec<EnhancedDetectedGap>,
    /// Semantic analysis results (if performed)
    pub semantic_analysis: Option<Vec<SemanticGapAnalysis>>,
    /// Configuration used for analysis
    pub config_summary: ConfigSummary,
    /// Analysis performance metrics
    pub performance_metrics: ConfigurablePerformanceMetrics,
    /// Rule application statistics
    pub rule_statistics: RuleStatistics,
}

/// Enhanced detected gap with configurable priority and filtering metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedDetectedGap {
    /// Original detected gap
    pub gap: DetectedGap,
    /// Enhanced priority calculated using configuration rules
    pub enhanced_priority: u8,
    /// Quality score based on filtering configuration
    pub quality_score: f64,
    /// Whether gap passed all filtering rules
    pub passed_filters: bool,
    /// Applied rule names and their effects
    pub applied_rules: Vec<AppliedRule>,
    /// Priority calculation breakdown
    pub priority_breakdown: PriorityBreakdown,
}

/// Information about a rule that was applied to a gap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedRule {
    /// Rule name
    pub rule_name: String,
    /// Rule type (filter, priority, validation)
    pub rule_type: String,
    /// Rule effect (passed, failed, priority_boost, etc.)
    pub effect: String,
    /// Additional details about rule application
    pub details: HashMap<String, String>,
}

/// Priority calculation breakdown for transparency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityBreakdown {
    /// Base priority from gap type
    pub base_priority: u8,
    /// Priority adjustments applied
    pub adjustments: Vec<PriorityAdjustment>,
    /// Final calculated priority
    pub final_priority: u8,
}

impl Default for PriorityBreakdown {
    fn default() -> Self {
        Self {
            base_priority: 5,
            adjustments: Vec::new(),
            final_priority: 5,
        }
    }
}

/// Individual priority adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityAdjustment {
    /// Source of adjustment (rule name, boost type)
    pub source: String,
    /// Adjustment value (can be negative)
    pub adjustment: f64,
    /// Reason for adjustment
    pub reason: String,
}

/// Summary of configuration used for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSummary {
    /// Configuration preset name (if applicable)
    pub preset_name: Option<String>,
    /// Key configuration settings
    pub key_settings: HashMap<String, String>,
    /// Enabled rule types
    pub enabled_rules: Vec<String>,
    /// Performance limits applied
    pub performance_limits: HashMap<String, u64>,
}

/// Performance metrics for configurable analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurablePerformanceMetrics {
    /// Total analysis time (milliseconds)
    pub total_time_ms: f64,
    /// Gap detection time (milliseconds)
    pub gap_detection_time_ms: f64,
    /// Semantic analysis time (milliseconds)
    pub semantic_analysis_time_ms: Option<f64>,
    /// Filtering and rule application time (milliseconds)
    pub filtering_time_ms: f64,
    /// Priority calculation time (milliseconds)
    pub priority_calculation_time_ms: f64,
    /// Number of gaps before filtering
    pub gaps_before_filtering: usize,
    /// Number of gaps after filtering
    pub gaps_after_filtering: usize,
    /// Configuration loading time (milliseconds)
    pub config_loading_time_ms: f64,
}

/// Statistics about rule application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleStatistics {
    /// Number of rules evaluated
    pub rules_evaluated: usize,
    /// Number of rules that matched
    pub rules_matched: usize,
    /// Rule application results by type
    pub results_by_type: HashMap<String, usize>,
    /// Rule performance metrics
    pub rule_performance: Vec<RulePerformanceMetric>,
    /// Number of gaps processed
    pub gaps_processed: usize,
    /// Number of gaps that passed filtering
    pub gaps_passed: usize,
}

/// Performance metric for individual rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulePerformanceMetric {
    /// Rule name
    pub rule_name: String,
    /// Rule type
    pub rule_type: String,
    /// Execution time (milliseconds)
    pub execution_time_ms: f64,
    /// Number of gaps processed
    pub gaps_processed: usize,
    /// Number of matches/effects
    pub matches: usize,
}

/// Configurable gap analyzer that wraps existing analyzers with configuration
pub struct ConfigurableGapAnalyzer {
    /// Configuration for analysis behavior
    config: GapDetectionConfig,
    /// Underlying gap analyzer
    gap_analyzer: GapAnalyzer,
    /// Semantic analyzer (optional)
    semantic_analyzer: Option<SemanticGapAnalyzer>,
    /// Compiled regex patterns for rules
    compiled_patterns: CompiledConfigPatterns,
}

/// Compiled patterns from configuration for efficient matching
#[derive(Debug)]
struct CompiledConfigPatterns {
    /// TODO custom patterns
    #[allow(dead_code)] // TODO: Will be used for custom TODO pattern detection
    todo_patterns: Vec<Regex>,
    /// Documentation patterns
    #[allow(dead_code)] // TODO: Will be used for documentation gap analysis
    doc_patterns: Vec<Regex>,
    /// Technology patterns
    #[allow(dead_code)] // TODO: Will be used for technology stack analysis
    tech_patterns: Vec<Regex>,
    /// Exclusion patterns
    exclusion_patterns: Vec<Regex>,
    /// Validation patterns
    #[allow(dead_code)] // TODO: Will be used for validation rule checking
    validation_patterns: Vec<Regex>,
    /// Priority rule patterns
    priority_rule_patterns: Vec<(String, Regex)>,
}

impl ConfigurableGapAnalyzer {
    /// Create a new configurable gap analyzer
    pub fn new(
        config: GapDetectionConfig,
        semantic_search: Option<Arc<dyn SemanticSearchOperations>>,
    ) -> Result<Self, ConfigurableAnalysisError> {
        // Validate configuration first
        config.validate()?;

        // Create underlying gap analyzer from configuration
        let gap_config = Self::convert_to_gap_analysis_config(&config);
        let gap_analyzer = GapAnalyzer::new(gap_config)?;

        // Create semantic analyzer if enabled and search service provided
        let semantic_analyzer = if config.semantic_config.enabled && semantic_search.is_some() {
            let semantic_config = Self::convert_to_semantic_analysis_config(&config);
            Some(SemanticGapAnalyzer::new(
                semantic_search.unwrap(),
                semantic_config,
            ))
        } else {
            None
        };

        // Compile patterns from configuration
        let compiled_patterns = Self::compile_patterns(&config)?;

        Ok(Self {
            config,
            gap_analyzer,
            semantic_analyzer,
            compiled_patterns,
        })
    }

    /// Create analyzer with preset configuration
    pub fn with_preset(
        preset_name: &str,
        semantic_search: Option<Arc<dyn SemanticSearchOperations>>,
    ) -> Result<Self, ConfigurableAnalysisError> {
        let config = GapDetectionConfig::preset(preset_name)?;
        Self::new(config, semantic_search)
    }

    /// Get reference to the configuration
    pub fn config(&self) -> &GapDetectionConfig {
        &self.config
    }

    /// Analyze a file with full configurable pipeline
    #[instrument(skip(self, file_path))]
    pub async fn analyze_file_comprehensive(
        &self,
        file_path: &Path,
    ) -> Result<ConfigurableAnalysisResult, ConfigurableAnalysisError> {
        let start_time = Instant::now();
        let mut rule_stats = RuleStatistics::new();

        // Step 1: Gap Detection
        let gap_detection_start = Instant::now();
        let raw_gaps = self.gap_analyzer.analyze_file(file_path).await?;
        let gap_detection_time = gap_detection_start.elapsed();
        let initial_gap_count = raw_gaps.len();

        debug!(
            "Detected {} raw gaps in {:?}",
            initial_gap_count, gap_detection_time
        );

        // Step 2: Semantic Analysis (if enabled)
        let semantic_start = Instant::now();
        let semantic_analysis = if let Some(ref semantic_analyzer) = self.semantic_analyzer {
            Some(
                semantic_analyzer
                    .analyze_gaps_semantically(raw_gaps.clone())
                    .await?,
            )
        } else {
            None
        };
        let semantic_time = semantic_start.elapsed();

        // Step 3: Apply Configuration Rules and Filtering
        let filtering_start = Instant::now();
        let enhanced_gaps = self
            .apply_rules_and_filtering(raw_gaps, &mut rule_stats)
            .await?;
        let filtering_time = filtering_start.elapsed();

        // Step 4: Calculate Enhanced Priorities
        let priority_start = Instant::now();
        let final_gaps = self
            .calculate_enhanced_priorities(enhanced_gaps, &semantic_analysis, &mut rule_stats)
            .await?;
        let priority_time = priority_start.elapsed();

        let total_time = start_time.elapsed();

        // Build result
        let result = ConfigurableAnalysisResult {
            filtered_gaps: final_gaps.clone(),
            semantic_analysis: semantic_analysis.clone(),
            config_summary: self.create_config_summary(),
            performance_metrics: ConfigurablePerformanceMetrics {
                total_time_ms: total_time.as_millis() as f64,
                gap_detection_time_ms: gap_detection_time.as_millis() as f64,
                semantic_analysis_time_ms: semantic_analysis
                    .as_ref()
                    .map(|_| semantic_time.as_millis() as f64),
                filtering_time_ms: filtering_time.as_millis() as f64,
                priority_calculation_time_ms: priority_time.as_millis() as f64,
                gaps_before_filtering: initial_gap_count,
                gaps_after_filtering: final_gaps.len(),
                config_loading_time_ms: 0.0, // Set during construction
            },
            rule_statistics: rule_stats,
        };

        info!(
            "Configurable analysis completed for {}: {} gaps -> {} gaps in {:?}",
            file_path.display(),
            initial_gap_count,
            final_gaps.len(),
            total_time
        );

        Ok(result)
    }

    /// Analyze a file event with configuration
    pub async fn analyze_file_event(
        &self,
        event: &FileEvent,
    ) -> Result<ConfigurableAnalysisResult, ConfigurableAnalysisError> {
        if !event.should_trigger_analysis {
            return Ok(ConfigurableAnalysisResult::empty());
        }

        self.analyze_file_comprehensive(&event.path).await
    }

    /// Apply configuration rules and filtering to detected gaps
    async fn apply_rules_and_filtering(
        &self,
        gaps: Vec<DetectedGap>,
        rule_stats: &mut RuleStatistics,
    ) -> Result<Vec<EnhancedDetectedGap>, ConfigurableAnalysisError> {
        let initial_count = gaps.len();
        let mut enhanced_gaps = Vec::new();

        for gap in gaps {
            let gap_start = Instant::now();
            let mut applied_rules = Vec::new();
            let mut quality_score = 1.0;
            let mut passed_filters = true;

            // Apply filtering rules
            if self.config.filtering_config.enabled {
                // Quality threshold filtering
                let quality_result = self.evaluate_quality_thresholds(&gap);
                quality_score = quality_result.score;
                passed_filters = quality_result.passed;
                applied_rules.push(AppliedRule {
                    rule_name: "quality_thresholds".to_string(),
                    rule_type: "filter".to_string(),
                    effect: if quality_result.passed {
                        "passed".to_string()
                    } else {
                        "failed".to_string()
                    },
                    details: quality_result.details,
                });

                // Exclusion rules
                if passed_filters {
                    let exclusion_result = self.evaluate_exclusion_rules(&gap);
                    passed_filters = exclusion_result.passed;
                    applied_rules.push(AppliedRule {
                        rule_name: "exclusion_rules".to_string(),
                        rule_type: "filter".to_string(),
                        effect: if exclusion_result.passed {
                            "passed".to_string()
                        } else {
                            "excluded".to_string()
                        },
                        details: exclusion_result.details,
                    });
                }

                // Validation rules
                if passed_filters {
                    let validation_result = self.evaluate_validation_rules(&gap);
                    passed_filters = validation_result.passed;
                    applied_rules.push(AppliedRule {
                        rule_name: "validation_rules".to_string(),
                        rule_type: "validation".to_string(),
                        effect: if validation_result.passed {
                            "passed".to_string()
                        } else {
                            "failed".to_string()
                        },
                        details: validation_result.details,
                    });
                }
            }

            let enhanced_gap = EnhancedDetectedGap {
                gap,
                enhanced_priority: 0, // Will be calculated later
                quality_score,
                passed_filters,
                applied_rules,
                priority_breakdown: PriorityBreakdown {
                    base_priority: 0,
                    adjustments: Vec::new(),
                    final_priority: 0,
                },
            };

            enhanced_gaps.push(enhanced_gap);

            // Update rule statistics
            let processing_time = gap_start.elapsed();
            rule_stats.add_rule_performance(RulePerformanceMetric {
                rule_name: "filtering_pipeline".to_string(),
                rule_type: "filter".to_string(),
                execution_time_ms: processing_time.as_millis() as f64,
                gaps_processed: 1,
                matches: if passed_filters { 1 } else { 0 },
            });
        }

        // Filter out gaps that didn't pass filters
        let filtered_gaps: Vec<_> = enhanced_gaps
            .into_iter()
            .filter(|gap| gap.passed_filters)
            .collect();

        rule_stats.gaps_processed = initial_count;
        rule_stats.gaps_passed = filtered_gaps.len();

        Ok(filtered_gaps)
    }

    /// Calculate enhanced priorities using configuration rules
    async fn calculate_enhanced_priorities(
        &self,
        mut gaps: Vec<EnhancedDetectedGap>,
        semantic_analysis: &Option<Vec<SemanticGapAnalysis>>,
        rule_stats: &mut RuleStatistics,
    ) -> Result<Vec<EnhancedDetectedGap>, ConfigurableAnalysisError> {
        for gap in &mut gaps {
            let priority_start = Instant::now();

            // Get base priority from configuration
            let gap_type_str = format!("{:?}", gap.gap.gap_type);
            let base_priority = self
                .config
                .priority_config
                .base_priorities
                .get(&gap_type_str)
                .copied()
                .unwrap_or(5);

            let mut adjustments = Vec::new();
            let mut current_priority = base_priority as f64;

            // Apply priority boosts from configuration
            let boosts = &self.config.priority_config.priority_boosts;

            // High confidence boost
            if gap.gap.confidence >= 0.9 {
                let boost = boosts.high_confidence_boost;
                current_priority += boost;
                adjustments.push(PriorityAdjustment {
                    source: "high_confidence_boost".to_string(),
                    adjustment: boost,
                    reason: format!("High confidence: {:.2}", gap.gap.confidence),
                });
            }

            // Low confidence penalty
            if gap.gap.confidence < 0.5 {
                let penalty = boosts.low_confidence_penalty;
                current_priority -= penalty;
                adjustments.push(PriorityAdjustment {
                    source: "low_confidence_penalty".to_string(),
                    adjustment: -penalty,
                    reason: format!("Low confidence: {:.2}", gap.gap.confidence),
                });
            }

            // Urgent keyword boost
            if self.has_urgent_keywords(&gap.gap) {
                let boost = boosts.urgent_keyword_boost;
                current_priority += boost;
                adjustments.push(PriorityAdjustment {
                    source: "urgent_keyword_boost".to_string(),
                    adjustment: boost,
                    reason: "Contains urgent keywords".to_string(),
                });
            }

            // Apply semantic analysis priority adjustments
            if let Some(ref semantic_results) = semantic_analysis {
                if let Some(semantic_gap) = semantic_results.iter().find(|sg| {
                    sg.gap.file_path == gap.gap.file_path
                        && sg.gap.line_number == gap.gap.line_number
                }) {
                    let semantic_weight = self.config.semantic_config.semantic_priority_weight;
                    let semantic_adjustment = (semantic_gap.enhanced_priority as f64
                        - base_priority as f64)
                        * semantic_weight;
                    current_priority += semantic_adjustment;
                    adjustments.push(PriorityAdjustment {
                        source: "semantic_analysis".to_string(),
                        adjustment: semantic_adjustment,
                        reason: format!(
                            "Semantic enhanced priority: {}",
                            semantic_gap.enhanced_priority
                        ),
                    });
                }
            }

            // Apply custom priority rules
            for custom_rule in &self.config.priority_config.custom_priority_rules {
                if self.evaluate_custom_priority_rule(custom_rule, &gap.gap)? {
                    let adjustment = custom_rule.priority_adjustment as f64;
                    current_priority += adjustment;
                    adjustments.push(PriorityAdjustment {
                        source: format!("custom_rule_{}", custom_rule.name),
                        adjustment,
                        reason: custom_rule.description.clone(),
                    });
                }
            }

            // Clamp to configured bounds
            let final_priority = current_priority
                .max(self.config.priority_config.min_priority as f64)
                .min(self.config.priority_config.max_priority as f64)
                as u8;

            gap.enhanced_priority = final_priority;
            gap.priority_breakdown = PriorityBreakdown {
                base_priority,
                adjustments,
                final_priority,
            };

            // Update performance metrics
            let processing_time = priority_start.elapsed();
            rule_stats.add_rule_performance(RulePerformanceMetric {
                rule_name: "priority_calculation".to_string(),
                rule_type: "priority".to_string(),
                execution_time_ms: processing_time.as_millis() as f64,
                gaps_processed: 1,
                matches: 1,
            });
        }

        // Sort by enhanced priority (highest first)
        gaps.sort_by(|a, b| b.enhanced_priority.cmp(&a.enhanced_priority));

        // Apply max gaps per file limit
        if gaps.len() > self.config.filtering_config.max_gaps_per_file {
            gaps.truncate(self.config.filtering_config.max_gaps_per_file);
        }

        Ok(gaps)
    }

    /// Convert GapDetectionConfig to GapAnalysisConfig
    fn convert_to_gap_analysis_config(config: &GapDetectionConfig) -> GapAnalysisConfig {
        let mut supported_extensions = HashSet::new();
        for ext in &config.detection_settings.supported_extensions {
            supported_extensions.insert(ext.clone());
        }

        GapAnalysisConfig {
            max_file_size_bytes: config.detection_settings.max_file_size_bytes,
            analysis_timeout_ms: config.detection_settings.analysis_timeout_ms,
            min_confidence_threshold: config.detection_settings.min_confidence_threshold,
            supported_extensions,
            enable_todo_detection: config.detection_rules.todo_rules.enabled,
            enable_docs_detection: config.detection_rules.documentation_rules.enabled,
            enable_tech_detection: config.detection_rules.technology_rules.enabled,
            enable_api_detection: config.detection_rules.api_rules.enabled,
            enable_config_detection: config.detection_rules.config_rules.enabled,
            custom_todo_patterns: config.detection_rules.todo_rules.custom_patterns.clone(),
            custom_doc_patterns: config
                .detection_rules
                .documentation_rules
                .valid_doc_patterns
                .clone(),
        }
    }

    /// Convert GapDetectionConfig to SemanticAnalysisConfig
    fn convert_to_semantic_analysis_config(config: &GapDetectionConfig) -> SemanticAnalysisConfig {
        SemanticAnalysisConfig {
            enable_gap_validation: config.semantic_config.enabled,
            enable_related_content: config.semantic_config.enabled,
            enable_priority_enhancement: config.semantic_config.enabled,
            max_analysis_time_ms: config.semantic_config.max_analysis_time_ms,
            gap_validation_threshold: config.semantic_config.gap_validation_threshold,
            related_content_threshold: config.semantic_config.related_content_threshold,
            max_related_documents: config.semantic_config.max_related_documents,
            min_content_length: config.semantic_config.min_content_length,
            batch_size: config.semantic_config.batch_size,
            semantic_priority_weight: config.semantic_config.semantic_priority_weight,
        }
    }

    /// Compile patterns from configuration
    fn compile_patterns(
        config: &GapDetectionConfig,
    ) -> Result<CompiledConfigPatterns, ConfigurableAnalysisError> {
        // Compile TODO patterns
        let mut todo_patterns = Vec::new();
        for pattern in &config.detection_rules.todo_rules.custom_patterns {
            todo_patterns.push(Regex::new(pattern).map_err(|e| {
                ConfigurableAnalysisError::Configuration(ConfigurationError::InvalidRegex {
                    context: "TODO patterns".to_string(),
                    pattern: pattern.clone(),
                    error: e.to_string(),
                })
            })?);
        }

        // Compile documentation patterns
        let mut doc_patterns = Vec::new();
        for pattern in &config
            .detection_rules
            .documentation_rules
            .valid_doc_patterns
        {
            doc_patterns.push(Regex::new(pattern).map_err(|e| {
                ConfigurableAnalysisError::Configuration(ConfigurationError::InvalidRegex {
                    context: "Documentation patterns".to_string(),
                    pattern: pattern.clone(),
                    error: e.to_string(),
                })
            })?);
        }

        // Compile technology patterns
        let mut tech_patterns = Vec::new();
        for pattern in &config.detection_rules.technology_rules.technology_patterns {
            tech_patterns.push(Regex::new(pattern).map_err(|e| {
                ConfigurableAnalysisError::Configuration(ConfigurationError::InvalidRegex {
                    context: "Technology patterns".to_string(),
                    pattern: pattern.clone(),
                    error: e.to_string(),
                })
            })?);
        }

        // Compile exclusion patterns
        let mut exclusion_patterns = Vec::new();
        for pattern in &config.filtering_config.exclusion_rules.exclude_patterns {
            exclusion_patterns.push(Regex::new(pattern).map_err(|e| {
                ConfigurableAnalysisError::Configuration(ConfigurationError::InvalidRegex {
                    context: "Exclusion patterns".to_string(),
                    pattern: pattern.clone(),
                    error: e.to_string(),
                })
            })?);
        }

        // Compile validation patterns
        let mut validation_patterns = Vec::new();
        for pattern in &config
            .filtering_config
            .validation_rules
            .custom_validation_patterns
        {
            validation_patterns.push(Regex::new(pattern).map_err(|e| {
                ConfigurableAnalysisError::Configuration(ConfigurationError::InvalidRegex {
                    context: "Validation patterns".to_string(),
                    pattern: pattern.clone(),
                    error: e.to_string(),
                })
            })?);
        }

        // Compile priority rule patterns
        let mut priority_rule_patterns = Vec::new();
        for rule in &config.priority_config.custom_priority_rules {
            if let Some(ref pattern) = rule.content_pattern {
                let compiled = Regex::new(pattern).map_err(|e| {
                    ConfigurableAnalysisError::Configuration(ConfigurationError::InvalidRegex {
                        context: format!("Priority rule {}", rule.name),
                        pattern: pattern.clone(),
                        error: e.to_string(),
                    })
                })?;
                priority_rule_patterns.push((rule.name.clone(), compiled));
            }
        }

        Ok(CompiledConfigPatterns {
            todo_patterns,
            doc_patterns,
            tech_patterns,
            exclusion_patterns,
            validation_patterns,
            priority_rule_patterns,
        })
    }

    /// Create configuration summary for results
    fn create_config_summary(&self) -> ConfigSummary {
        let mut key_settings = HashMap::new();
        key_settings.insert(
            "min_confidence_threshold".to_string(),
            self.config
                .detection_settings
                .min_confidence_threshold
                .to_string(),
        );
        key_settings.insert(
            "semantic_enabled".to_string(),
            self.config.semantic_config.enabled.to_string(),
        );
        key_settings.insert(
            "max_gaps_per_file".to_string(),
            self.config.filtering_config.max_gaps_per_file.to_string(),
        );

        let mut enabled_rules = Vec::new();
        if self.config.detection_rules.todo_rules.enabled {
            enabled_rules.push("todo_detection".to_string());
        }
        if self.config.detection_rules.documentation_rules.enabled {
            enabled_rules.push("documentation_detection".to_string());
        }
        if self.config.detection_rules.technology_rules.enabled {
            enabled_rules.push("technology_detection".to_string());
        }
        if self.config.filtering_config.enabled {
            enabled_rules.push("filtering".to_string());
        }

        let mut performance_limits = HashMap::new();
        performance_limits.insert(
            "max_total_time_ms".to_string(),
            self.config.performance_config.max_total_time_ms,
        );
        performance_limits.insert(
            "max_file_size_bytes".to_string(),
            self.config.detection_settings.max_file_size_bytes,
        );

        ConfigSummary {
            preset_name: None, // Could be tracked if created from preset
            key_settings,
            enabled_rules,
            performance_limits,
        }
    }

    /// Evaluate quality thresholds for a gap
    fn evaluate_quality_thresholds(&self, gap: &DetectedGap) -> RuleEvaluationResult {
        let thresholds = &self.config.filtering_config.quality_thresholds;
        let mut details = HashMap::new();
        let mut passed = true;

        // Check content length
        if gap.context.len() < thresholds.min_content_length {
            passed = false;
            details.insert("content_length_check".to_string(), "failed".to_string());
        } else if gap.context.len() > thresholds.max_content_length {
            passed = false;
            details.insert("content_length_check".to_string(), "too_long".to_string());
        } else {
            details.insert("content_length_check".to_string(), "passed".to_string());
        }

        // Check description length
        if gap.description.len() < thresholds.min_description_length {
            passed = false;
            details.insert("description_length_check".to_string(), "failed".to_string());
        } else {
            details.insert("description_length_check".to_string(), "passed".to_string());
        }

        // Calculate quality score based on various factors
        let score = gap.confidence * 0.6 + (gap.context.len() as f64 / 1000.0).min(1.0) * 0.4;

        if score < thresholds.min_quality_score {
            passed = false;
            details.insert("quality_score_check".to_string(), "failed".to_string());
        } else {
            details.insert("quality_score_check".to_string(), "passed".to_string());
        }

        details.insert("quality_score".to_string(), score.to_string());

        RuleEvaluationResult {
            passed,
            score,
            details,
        }
    }

    /// Evaluate exclusion rules for a gap
    fn evaluate_exclusion_rules(&self, gap: &DetectedGap) -> RuleEvaluationResult {
        let exclusion_rules = &self.config.filtering_config.exclusion_rules;
        let mut details = HashMap::new();
        let mut passed = true;

        // Check exclusion patterns
        for (i, pattern) in self.compiled_patterns.exclusion_patterns.iter().enumerate() {
            if pattern.is_match(&gap.context) || pattern.is_match(&gap.description) {
                passed = false;
                details.insert(format!("exclusion_pattern_{i}"), "matched".to_string());
                break;
            }
        }

        // Check non-actionable keywords
        for keyword in &exclusion_rules.non_actionable_keywords {
            if gap
                .description
                .to_lowercase()
                .contains(&keyword.to_lowercase())
            {
                passed = false;
                details.insert("non_actionable_keyword".to_string(), keyword.clone());
                break;
            }
        }

        // Check gap type exclusions
        let gap_type_str = format!("{:?}", gap.gap_type);
        if exclusion_rules.exclude_gap_types.contains(&gap_type_str) {
            passed = false;
            details.insert("gap_type_excluded".to_string(), gap_type_str);
        }

        if passed {
            details.insert("exclusion_check".to_string(), "passed".to_string());
        }

        RuleEvaluationResult {
            passed,
            score: if passed { 1.0 } else { 0.0 },
            details,
        }
    }

    /// Evaluate validation rules for a gap
    fn evaluate_validation_rules(&self, gap: &DetectedGap) -> RuleEvaluationResult {
        let validation_rules = &self.config.filtering_config.validation_rules;
        let mut details = HashMap::new();
        let mut passed = true;

        // Check minimum context length
        if gap.context.len() < validation_rules.min_context_length {
            passed = false;
            details.insert("min_context_length".to_string(), "failed".to_string());
        } else {
            details.insert("min_context_length".to_string(), "passed".to_string());
        }

        // Check valid line numbers
        if validation_rules.require_valid_line_numbers && gap.line_number == 0 {
            passed = false;
            details.insert("valid_line_numbers".to_string(), "failed".to_string());
        } else {
            details.insert("valid_line_numbers".to_string(), "passed".to_string());
        }

        // Check file existence
        if validation_rules.validate_file_existence && !gap.file_path.exists() {
            passed = false;
            details.insert("file_existence".to_string(), "failed".to_string());
        } else {
            details.insert("file_existence".to_string(), "passed".to_string());
        }

        RuleEvaluationResult {
            passed,
            score: if passed { 1.0 } else { 0.0 },
            details,
        }
    }

    /// Check if gap contains urgent keywords
    fn has_urgent_keywords(&self, gap: &DetectedGap) -> bool {
        let urgent_keywords = &self.config.detection_rules.todo_rules.urgent_keywords;
        let text = format!("{} {}", gap.context, gap.description).to_lowercase();

        urgent_keywords
            .iter()
            .any(|keyword| text.contains(&keyword.to_lowercase()))
    }

    /// Evaluate custom priority rule
    fn evaluate_custom_priority_rule(
        &self,
        rule: &crate::proactive::CustomPriorityRule,
        gap: &DetectedGap,
    ) -> Result<bool, ConfigurableAnalysisError> {
        let mut matches = true;

        // Check file pattern
        if let Some(ref file_pattern) = rule.file_pattern {
            let file_regex = Regex::new(file_pattern).map_err(|e| {
                ConfigurableAnalysisError::RuleEvaluation {
                    rule: rule.name.clone(),
                    error: format!("Invalid file pattern: {e}"),
                }
            })?;

            if !file_regex.is_match(&gap.file_path.to_string_lossy()) {
                matches = false;
            }
        }

        // Check gap type pattern
        if let Some(ref gap_type_pattern) = rule.gap_type_pattern {
            let gap_type_str = format!("{:?}", gap.gap_type);
            if !gap_type_str.contains(gap_type_pattern) {
                matches = false;
            }
        }

        // Check content pattern
        if let Some(ref _content_pattern) = rule.content_pattern {
            // Find compiled pattern
            if let Some((_, regex)) = self
                .compiled_patterns
                .priority_rule_patterns
                .iter()
                .find(|(name, _)| name == &rule.name)
            {
                let combined_text = format!("{} {}", gap.context, gap.description);
                if !regex.is_match(&combined_text) {
                    matches = false;
                }
            }
        }

        Ok(matches)
    }
}

/// Result of rule evaluation
#[derive(Debug)]
struct RuleEvaluationResult {
    passed: bool,
    score: f64,
    details: HashMap<String, String>,
}

impl RuleStatistics {
    fn new() -> Self {
        Self {
            rules_evaluated: 0,
            rules_matched: 0,
            results_by_type: HashMap::new(),
            rule_performance: Vec::new(),
            gaps_processed: 0,
            gaps_passed: 0,
        }
    }

    fn add_rule_performance(&mut self, metric: RulePerformanceMetric) {
        self.rules_evaluated += 1;
        if metric.matches > 0 {
            self.rules_matched += 1;
        }

        *self
            .results_by_type
            .entry(metric.rule_type.clone())
            .or_insert(0) += metric.matches;
        self.rule_performance.push(metric);
    }
}

impl ConfigurableAnalysisResult {
    fn empty() -> Self {
        Self {
            filtered_gaps: Vec::new(),
            semantic_analysis: None,
            config_summary: ConfigSummary {
                preset_name: None,
                key_settings: HashMap::new(),
                enabled_rules: Vec::new(),
                performance_limits: HashMap::new(),
            },
            performance_metrics: ConfigurablePerformanceMetrics {
                total_time_ms: 0.0,
                gap_detection_time_ms: 0.0,
                semantic_analysis_time_ms: None,
                filtering_time_ms: 0.0,
                priority_calculation_time_ms: 0.0,
                gaps_before_filtering: 0,
                gaps_after_filtering: 0,
                config_loading_time_ms: 0.0,
            },
            rule_statistics: RuleStatistics::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proactive::GapType;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_configurable_analyzer_creation() {
        let config = GapDetectionConfig::for_rust_project();
        let analyzer = ConfigurableGapAnalyzer::new(config, None).unwrap();

        // Should have disabled semantic analysis since no search service provided
        assert!(analyzer.semantic_analyzer.is_none());
    }

    #[tokio::test]
    async fn test_configurable_analyzer_with_preset() {
        let analyzer = ConfigurableGapAnalyzer::with_preset("performance", None).unwrap();

        // Should be performance optimized
        assert!(analyzer.config.performance_config.max_total_time_ms < 500);
    }

    #[tokio::test]
    async fn test_quality_threshold_evaluation() {
        let mut config = GapDetectionConfig::default();
        // Lower the quality threshold for this test
        config.filtering_config.quality_thresholds.min_quality_score = 0.4;
        let analyzer = ConfigurableGapAnalyzer::new(config, None).unwrap();

        let gap = DetectedGap::new(
            GapType::TodoComment,
            PathBuf::from("test.rs"),
            10,
            "This is a reasonable length context for testing".to_string(),
            "Valid description".to_string(),
            0.8,
        );

        let result = analyzer.evaluate_quality_thresholds(&gap);
        assert!(
            result.passed,
            "Quality evaluation should pass. Details: {:?}",
            result.details
        );
        assert!(result.score > 0.4);
    }

    #[tokio::test]
    async fn test_exclusion_rules_evaluation() {
        let mut config = GapDetectionConfig::default();
        config
            .filtering_config
            .exclusion_rules
            .non_actionable_keywords
            .push("maybe".to_string());

        let analyzer = ConfigurableGapAnalyzer::new(config, None).unwrap();

        let gap = DetectedGap::new(
            GapType::TodoComment,
            PathBuf::from("test.rs"),
            10,
            "// TODO: Maybe do this someday".to_string(),
            "Maybe implement this feature".to_string(),
            0.8,
        );

        let result = analyzer.evaluate_exclusion_rules(&gap);
        assert!(!result.passed); // Should be excluded due to "maybe" keyword
    }

    #[tokio::test]
    async fn test_urgent_keyword_detection() {
        let config = GapDetectionConfig::default();
        let analyzer = ConfigurableGapAnalyzer::new(config, None).unwrap();

        let urgent_gap = DetectedGap::new(
            GapType::TodoComment,
            PathBuf::from("test.rs"),
            10,
            "// FIXME: URGENT security issue".to_string(),
            "Fix security vulnerability".to_string(),
            0.9,
        );

        assert!(analyzer.has_urgent_keywords(&urgent_gap));

        let normal_gap = DetectedGap::new(
            GapType::TodoComment,
            PathBuf::from("test.rs"),
            10,
            "// TODO: Add documentation".to_string(),
            "Add docs".to_string(),
            0.8,
        );

        assert!(!analyzer.has_urgent_keywords(&normal_gap));
    }

    #[tokio::test]
    async fn test_config_conversion() {
        let config = GapDetectionConfig::for_rust_project();
        let gap_config = ConfigurableGapAnalyzer::convert_to_gap_analysis_config(&config);

        assert_eq!(
            gap_config.min_confidence_threshold,
            config.detection_settings.min_confidence_threshold
        );
        assert_eq!(
            gap_config.enable_todo_detection,
            config.detection_rules.todo_rules.enabled
        );
    }

    #[tokio::test]
    async fn test_empty_result_creation() {
        let result = ConfigurableAnalysisResult::empty();

        assert!(result.filtered_gaps.is_empty());
        assert!(result.semantic_analysis.is_none());
        assert_eq!(result.performance_metrics.gaps_before_filtering, 0);
    }
}
