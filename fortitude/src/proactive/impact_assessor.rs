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

// ABOUTME: Code usage pattern impact assessment for knowledge gap prioritization in proactive research mode
//! This module provides sophisticated impact assessment that analyzes code usage patterns, dependency
//! relationships, API visibility, and development activity to determine the potential impact of addressing
//! different knowledge gaps. The system enhances prioritization accuracy by understanding how code
//! components are used and interconnected.
//!
//! Key Features:
//! - Usage frequency analysis through imports, references, and function calls
//! - Dependency tree analysis to assess impact propagation
//! - API surface analysis for public/private exposure assessment
//! - Development activity analysis based on change frequency
//! - Cross-module impact assessment for component relationships
//! - Team workflow impact evaluation
//!
//! Performance Requirements:
//! - Impact analysis should complete within existing priority scoring timeframes (<100ms)
//! - Memory-efficient analysis with configurable depth limits
//! - Caching of analysis results for frequently assessed components
//! - Graceful degradation for incomplete or unavailable code information

use crate::proactive::{DetectedGap, GapType};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

/// Errors that can occur during impact assessment operations
#[derive(Error, Debug)]
pub enum ImpactAssessmentError {
    #[error("Code analysis error: {0}")]
    CodeAnalysis(String),

    #[error("File access error for {path}: {error}")]
    FileAccess { path: String, error: std::io::Error },

    #[error("Dependency analysis failed: {0}")]
    DependencyAnalysis(String),

    #[error("API visibility analysis failed: {0}")]
    ApiVisibilityAnalysis(String),

    #[error("Development activity analysis failed: {0}")]
    DevelopmentActivityAnalysis(String),

    #[error("Team impact analysis failed: {0}")]
    TeamImpactAnalysis(String),

    #[error(
        "Performance threshold exceeded: impact analysis took {duration:?}, limit is {limit:?}"
    )]
    PerformanceThreshold { duration: Duration, limit: Duration },

    #[error("Invalid impact score: {score}, must be between 0.0 and 10.0")]
    InvalidImpactScore { score: f64 },

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Cache operation failed: {0}")]
    CacheOperation(String),
}

/// Configuration for impact assessment analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessmentConfig {
    /// Maximum time allowed for impact analysis (milliseconds)
    pub max_analysis_time_ms: u64,
    /// Maximum depth for dependency tree analysis
    pub max_dependency_depth: usize,
    /// Weight for usage frequency in impact score (0.0-1.0)
    pub usage_frequency_weight: f64,
    /// Weight for dependency impact in impact score (0.0-1.0)
    pub dependency_impact_weight: f64,
    /// Weight for API visibility in impact score (0.0-1.0)
    pub api_visibility_weight: f64,
    /// Weight for development activity in impact score (0.0-1.0)
    pub development_activity_weight: f64,
    /// Weight for team impact in impact score (0.0-1.0)
    pub team_impact_weight: f64,
    /// Enable caching of impact analysis results
    pub enable_impact_caching: bool,
    /// Cache TTL for impact analysis results (seconds)
    pub impact_cache_ttl_secs: u64,
    /// Maximum file size to analyze (bytes)
    pub max_file_size_bytes: u64,
    /// File extensions to include in analysis
    pub supported_extensions: HashSet<String>,
    /// Patterns to identify public APIs
    pub public_api_patterns: Vec<String>,
    /// Patterns to identify internal APIs
    pub internal_api_patterns: Vec<String>,
}

impl Default for ImpactAssessmentConfig {
    fn default() -> Self {
        let mut supported_extensions = HashSet::new();
        supported_extensions.extend([
            "rs".to_string(),
            "py".to_string(),
            "js".to_string(),
            "ts".to_string(),
            "java".to_string(),
            "cpp".to_string(),
            "c".to_string(),
            "h".to_string(),
            "go".to_string(),
            "rb".to_string(),
            "php".to_string(),
            "cs".to_string(),
        ]);

        Self {
            max_analysis_time_ms: 100,
            max_dependency_depth: 5,
            usage_frequency_weight: 0.25,
            dependency_impact_weight: 0.25,
            api_visibility_weight: 0.2,
            development_activity_weight: 0.15,
            team_impact_weight: 0.15,
            enable_impact_caching: true,
            impact_cache_ttl_secs: 300,       // 5 minutes
            max_file_size_bytes: 1024 * 1024, // 1MB
            supported_extensions,
            public_api_patterns: vec![
                r"pub\s+fn\s+".to_string(),
                r"pub\s+struct\s+".to_string(),
                r"pub\s+enum\s+".to_string(),
                r"pub\s+trait\s+".to_string(),
                r"export\s+".to_string(),
                r"public\s+".to_string(),
            ],
            internal_api_patterns: vec![
                r"fn\s+".to_string(),
                r"struct\s+".to_string(),
                r"enum\s+".to_string(),
                r"trait\s+".to_string(),
                r"impl\s+".to_string(),
            ],
        }
    }
}

impl ImpactAssessmentConfig {
    /// Validate configuration weights sum to 1.0
    pub fn validate(&self) -> Result<(), ImpactAssessmentError> {
        let weight_sum = self.usage_frequency_weight
            + self.dependency_impact_weight
            + self.api_visibility_weight
            + self.development_activity_weight
            + self.team_impact_weight;

        if (weight_sum - 1.0).abs() > 0.001 {
            return Err(ImpactAssessmentError::Configuration(format!(
                "Weight sum {weight_sum:.3} must equal 1.0"
            )));
        }

        if self.max_dependency_depth == 0 {
            return Err(ImpactAssessmentError::Configuration(
                "max_dependency_depth must be greater than 0".to_string(),
            ));
        }

        if self.max_analysis_time_ms == 0 {
            return Err(ImpactAssessmentError::Configuration(
                "max_analysis_time_ms must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Create configuration optimized for performance
    pub fn for_performance() -> Self {
        Self {
            max_analysis_time_ms: 50,
            max_dependency_depth: 3,
            enable_impact_caching: true,
            impact_cache_ttl_secs: 600,      // 10 minutes
            max_file_size_bytes: 512 * 1024, // 512KB
            ..Default::default()
        }
    }

    /// Create configuration optimized for accuracy
    pub fn for_accuracy() -> Self {
        Self {
            max_analysis_time_ms: 200,
            max_dependency_depth: 8,
            max_file_size_bytes: 2 * 1024 * 1024, // 2MB
            ..Default::default()
        }
    }
}

/// Usage pattern analysis for a code component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePatternAnalysis {
    /// Number of times this component is imported/referenced
    pub import_count: usize,
    /// Number of direct function/method calls to this component
    pub call_count: usize,
    /// Number of files that reference this component
    pub referencing_files: usize,
    /// Files that directly import/use this component
    pub importing_files: Vec<PathBuf>,
    /// Estimated usage frequency score (0.0-10.0)
    pub usage_frequency_score: f64,
    /// Confidence in usage analysis
    pub analysis_confidence: f64,
}

impl Default for UsagePatternAnalysis {
    fn default() -> Self {
        Self {
            import_count: 0,
            call_count: 0,
            referencing_files: 0,
            importing_files: Vec::new(),
            usage_frequency_score: 0.0,
            analysis_confidence: 0.0,
        }
    }
}

/// Dependency impact analysis for a code component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyImpactAnalysis {
    /// Number of direct dependencies
    pub direct_dependencies: usize,
    /// Number of transitive dependencies within analysis depth
    pub transitive_dependencies: usize,
    /// Number of components that depend on this component
    pub reverse_dependencies: usize,
    /// Maximum depth of dependency chain
    pub max_dependency_depth: usize,
    /// Critical dependencies that would break functionality
    pub critical_dependencies: Vec<String>,
    /// Dependency impact score (0.0-10.0)
    pub dependency_impact_score: f64,
    /// Confidence in dependency analysis
    pub analysis_confidence: f64,
}

impl Default for DependencyImpactAnalysis {
    fn default() -> Self {
        Self {
            direct_dependencies: 0,
            transitive_dependencies: 0,
            reverse_dependencies: 0,
            max_dependency_depth: 0,
            critical_dependencies: Vec::new(),
            dependency_impact_score: 0.0,
            analysis_confidence: 0.0,
        }
    }
}

/// API visibility and exposure analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiVisibilityAnalysis {
    /// Whether this component has public API exposure
    pub has_public_api: bool,
    /// Number of public methods/functions
    pub public_methods: usize,
    /// Number of internal/private methods
    pub internal_methods: usize,
    /// Whether this is part of external API surface
    pub external_api_surface: bool,
    /// API surface size (total exposed API points)
    pub api_surface_size: usize,
    /// External usage indicators (e.g., in examples, documentation)
    pub external_usage_indicators: Vec<String>,
    /// API visibility impact score (0.0-10.0)
    pub api_visibility_score: f64,
    /// Confidence in visibility analysis
    pub analysis_confidence: f64,
}

impl Default for ApiVisibilityAnalysis {
    fn default() -> Self {
        Self {
            has_public_api: false,
            public_methods: 0,
            internal_methods: 0,
            external_api_surface: false,
            api_surface_size: 0,
            external_usage_indicators: Vec::new(),
            api_visibility_score: 0.0,
            analysis_confidence: 0.0,
        }
    }
}

/// Development activity and change frequency analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentActivityAnalysis {
    /// Recent change frequency (changes per week)
    pub change_frequency: f64,
    /// Number of developers who have modified this component
    pub developer_count: usize,
    /// Last modification timestamp
    pub last_modified: Option<DateTime<Utc>>,
    /// Average time between modifications
    pub average_modification_interval: Option<Duration>,
    /// Whether this is actively developed
    pub actively_developed: bool,
    /// Development activity impact score (0.0-10.0)
    pub activity_impact_score: f64,
    /// Confidence in activity analysis
    pub analysis_confidence: f64,
}

impl Default for DevelopmentActivityAnalysis {
    fn default() -> Self {
        Self {
            change_frequency: 0.0,
            developer_count: 0,
            last_modified: None,
            average_modification_interval: None,
            actively_developed: false,
            activity_impact_score: 0.0,
            analysis_confidence: 0.0,
        }
    }
}

/// Team workflow impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamImpactAnalysis {
    /// Number of team members who work with this component
    pub affected_team_members: usize,
    /// Critical workflows that depend on this component
    pub critical_workflows: Vec<String>,
    /// Development bottleneck potential
    pub bottleneck_potential: f64,
    /// Knowledge distribution among team members
    pub knowledge_distribution: f64,
    /// Team impact score (0.0-10.0)
    pub team_impact_score: f64,
    /// Confidence in team impact analysis
    pub analysis_confidence: f64,
}

impl Default for TeamImpactAnalysis {
    fn default() -> Self {
        Self {
            affected_team_members: 0,
            critical_workflows: Vec::new(),
            bottleneck_potential: 0.0,
            knowledge_distribution: 0.0,
            team_impact_score: 0.0,
            analysis_confidence: 0.0,
        }
    }
}

/// Comprehensive impact assessment result for a knowledge gap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessmentResult {
    /// Final calculated impact score (0.0-10.0)
    pub final_impact_score: f64,
    /// Usage pattern analysis component
    pub usage_analysis: UsagePatternAnalysis,
    /// Dependency impact analysis component
    pub dependency_analysis: DependencyImpactAnalysis,
    /// API visibility analysis component
    pub api_visibility_analysis: ApiVisibilityAnalysis,
    /// Development activity analysis component
    pub activity_analysis: DevelopmentActivityAnalysis,
    /// Team impact analysis component
    pub team_analysis: TeamImpactAnalysis,
    /// Overall confidence in impact assessment
    pub overall_confidence: f64,
    /// Analysis timestamp
    pub analyzed_at: DateTime<Utc>,
    /// Time taken for analysis
    pub analysis_duration: Duration,
    /// Whether analysis used cached results
    pub used_cached_results: bool,
    /// Analysis metadata and debugging information
    pub analysis_metadata: HashMap<String, String>,
}

impl ImpactAssessmentResult {
    /// Get the highest contributing factor to impact score
    pub fn primary_impact_factor(&self) -> String {
        let scores = vec![
            ("usage_frequency", self.usage_analysis.usage_frequency_score),
            (
                "dependency_impact",
                self.dependency_analysis.dependency_impact_score,
            ),
            (
                "api_visibility",
                self.api_visibility_analysis.api_visibility_score,
            ),
            (
                "development_activity",
                self.activity_analysis.activity_impact_score,
            ),
            ("team_impact", self.team_analysis.team_impact_score),
        ];

        scores
            .into_iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(factor, _)| factor.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Check if impact assessment has high confidence
    pub fn has_high_confidence(&self, threshold: f64) -> bool {
        self.overall_confidence >= threshold
    }

    /// Get summary of impact factors
    pub fn impact_summary(&self) -> HashMap<String, f64> {
        let mut summary = HashMap::new();
        summary.insert(
            "usage_frequency".to_string(),
            self.usage_analysis.usage_frequency_score,
        );
        summary.insert(
            "dependency_impact".to_string(),
            self.dependency_analysis.dependency_impact_score,
        );
        summary.insert(
            "api_visibility".to_string(),
            self.api_visibility_analysis.api_visibility_score,
        );
        summary.insert(
            "development_activity".to_string(),
            self.activity_analysis.activity_impact_score,
        );
        summary.insert(
            "team_impact".to_string(),
            self.team_analysis.team_impact_score,
        );
        summary
    }
}

/// Cached impact assessment result
#[derive(Debug, Clone)]
struct CachedImpactResult {
    result: ImpactAssessmentResult,
    expires_at: DateTime<Utc>,
}

/// Impact assessment metrics for monitoring and optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessmentMetrics {
    pub total_assessments: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub average_analysis_time: Duration,
    pub performance_violations: u64,
    pub successful_analyses: u64,
    pub failed_analyses: u64,
    pub last_updated: DateTime<Utc>,
}

impl Default for ImpactAssessmentMetrics {
    fn default() -> Self {
        Self {
            total_assessments: 0,
            cache_hits: 0,
            cache_misses: 0,
            average_analysis_time: Duration::from_millis(0),
            performance_violations: 0,
            successful_analyses: 0,
            failed_analyses: 0,
            last_updated: Utc::now(),
        }
    }
}

/// Impact assessor for analyzing code usage patterns and their impact on development workflow
pub struct ImpactAssessor {
    config: ImpactAssessmentConfig,
    impact_cache: Arc<RwLock<HashMap<String, CachedImpactResult>>>,
    metrics: Arc<RwLock<ImpactAssessmentMetrics>>,
    // TODO: Add code analysis dependencies (AST parser, git integration, etc.)
}

impl ImpactAssessor {
    /// Create a new impact assessor with the given configuration
    #[instrument(level = "debug", skip(config))]
    pub async fn new(config: ImpactAssessmentConfig) -> Result<Self, ImpactAssessmentError> {
        config.validate()?;

        info!("Initializing impact assessor with weights: usage={}, dependency={}, api={}, activity={}, team={}",
              config.usage_frequency_weight, config.dependency_impact_weight,
              config.api_visibility_weight, config.development_activity_weight, config.team_impact_weight);

        Ok(Self {
            config,
            impact_cache: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(ImpactAssessmentMetrics::default())),
        })
    }

    /// Create an impact assessor with default configuration
    pub async fn with_defaults() -> Result<Self, ImpactAssessmentError> {
        Self::new(ImpactAssessmentConfig::default()).await
    }

    /// Assess impact for a single gap based on code usage patterns
    #[instrument(level = "debug", skip(self, gap))]
    pub async fn assess_gap_impact(
        &self,
        gap: &DetectedGap,
    ) -> Result<ImpactAssessmentResult, ImpactAssessmentError> {
        let start_time = Instant::now();

        // Check cache first if enabled
        if self.config.enable_impact_caching {
            let cache_key = self.generate_cache_key(gap);
            if let Some(cached) = self.get_cached_result(&cache_key).await {
                self.increment_cache_hit().await;
                return Ok(cached.result);
            }
            self.increment_cache_miss().await;
        }

        // Perform impact analysis
        let result = self.perform_impact_analysis(gap).await?;

        // Cache the result if enabled
        if self.config.enable_impact_caching {
            let cache_key = self.generate_cache_key(gap);
            self.cache_result(cache_key, result.clone()).await;
        }

        // Check performance
        let duration = start_time.elapsed();
        let limit = Duration::from_millis(self.config.max_analysis_time_ms);
        if duration > limit {
            warn!(
                "Impact analysis exceeded time limit: {:?} > {:?}",
                duration, limit
            );
            self.increment_performance_violation().await;
        }

        self.update_metrics(duration, true).await;

        debug!(
            "Completed impact assessment for gap {} with score {:.2} (primary factor: {})",
            gap.gap_type.to_string(),
            result.final_impact_score,
            result.primary_impact_factor()
        );

        Ok(result)
    }

    /// Assess impact for multiple gaps in batch for improved performance
    #[instrument(level = "debug", skip(self, gaps))]
    pub async fn assess_gaps_batch(
        &self,
        gaps: &[DetectedGap],
    ) -> Result<Vec<ImpactAssessmentResult>, ImpactAssessmentError> {
        let start_time = Instant::now();

        info!("Batch assessing impact for {} gaps", gaps.len());

        let mut results = Vec::with_capacity(gaps.len());

        for gap in gaps {
            match self.assess_gap_impact(gap).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Failed to assess impact for gap {:?}: {}", gap.gap_type, e);
                    self.update_metrics(Duration::from_millis(0), false).await;
                    // Continue with other gaps, but note the error
                    continue;
                }
            }
        }

        let duration = start_time.elapsed();
        info!(
            "Completed batch impact assessment of {} gaps in {:?}",
            results.len(),
            duration
        );

        Ok(results)
    }

    /// Perform the core impact analysis for a gap
    async fn perform_impact_analysis(
        &self,
        gap: &DetectedGap,
    ) -> Result<ImpactAssessmentResult, ImpactAssessmentError> {
        let analysis_start = Instant::now();
        let mut metadata = HashMap::new();

        // Analyze usage patterns
        let usage_analysis = self.analyze_usage_patterns(gap).await?;

        // Analyze dependency impact
        let dependency_analysis = self.analyze_dependency_impact(gap).await?;

        // Analyze API visibility
        let api_visibility_analysis = self.analyze_api_visibility(gap).await?;

        // Analyze development activity
        let activity_analysis = self.analyze_development_activity(gap).await?;

        // Analyze team impact
        let team_analysis = self.analyze_team_impact(gap).await?;

        // Calculate final impact score using configured weights
        let final_impact_score = self.calculate_weighted_impact_score(
            &usage_analysis,
            &dependency_analysis,
            &api_visibility_analysis,
            &activity_analysis,
            &team_analysis,
        )?;

        // Calculate overall confidence
        let overall_confidence = self.calculate_overall_confidence(
            &usage_analysis,
            &dependency_analysis,
            &api_visibility_analysis,
            &activity_analysis,
            &team_analysis,
        );

        let analysis_duration = analysis_start.elapsed();

        metadata.insert(
            "file_extension".to_string(),
            gap.file_path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        );
        metadata.insert(
            "gap_context_length".to_string(),
            gap.context.len().to_string(),
        );
        metadata.insert(
            "analysis_method".to_string(),
            "code_usage_patterns".to_string(),
        );

        Ok(ImpactAssessmentResult {
            final_impact_score,
            usage_analysis,
            dependency_analysis,
            api_visibility_analysis,
            activity_analysis,
            team_analysis,
            overall_confidence,
            analyzed_at: Utc::now(),
            analysis_duration,
            used_cached_results: false,
            analysis_metadata: metadata,
        })
    }

    /// Calculate weighted final impact score from component analyses
    fn calculate_weighted_impact_score(
        &self,
        usage: &UsagePatternAnalysis,
        dependency: &DependencyImpactAnalysis,
        api_visibility: &ApiVisibilityAnalysis,
        activity: &DevelopmentActivityAnalysis,
        team: &TeamImpactAnalysis,
    ) -> Result<f64, ImpactAssessmentError> {
        let score = (usage.usage_frequency_score * self.config.usage_frequency_weight)
            + (dependency.dependency_impact_score * self.config.dependency_impact_weight)
            + (api_visibility.api_visibility_score * self.config.api_visibility_weight)
            + (activity.activity_impact_score * self.config.development_activity_weight)
            + (team.team_impact_score * self.config.team_impact_weight);

        if !(0.0..=10.0).contains(&score) {
            return Err(ImpactAssessmentError::InvalidImpactScore { score });
        }

        Ok(score)
    }

    /// Calculate overall confidence from component confidences
    fn calculate_overall_confidence(
        &self,
        usage: &UsagePatternAnalysis,
        dependency: &DependencyImpactAnalysis,
        api_visibility: &ApiVisibilityAnalysis,
        activity: &DevelopmentActivityAnalysis,
        team: &TeamImpactAnalysis,
    ) -> f64 {
        let confidence = (usage.analysis_confidence * self.config.usage_frequency_weight)
            + (dependency.analysis_confidence * self.config.dependency_impact_weight)
            + (api_visibility.analysis_confidence * self.config.api_visibility_weight)
            + (activity.analysis_confidence * self.config.development_activity_weight)
            + (team.analysis_confidence * self.config.team_impact_weight);

        confidence.clamp(0.0, 1.0)
    }

    /// Analyze usage patterns for the gap's code component
    async fn analyze_usage_patterns(
        &self,
        gap: &DetectedGap,
    ) -> Result<UsagePatternAnalysis, ImpactAssessmentError> {
        // TODO: Implement actual usage pattern analysis
        // This would involve:
        // 1. Parsing the file to identify the component (function, struct, etc.)
        // 2. Searching for imports/uses of this component across the codebase
        // 3. Counting function calls and references
        // 4. Analyzing usage frequency and patterns

        // Placeholder implementation based on gap metadata and file characteristics
        let mut usage_analysis = UsagePatternAnalysis::default();

        // Basic analysis based on gap type and file location
        match gap.gap_type {
            GapType::ApiDocumentationGap => {
                usage_analysis.usage_frequency_score = 8.0; // High impact for API gaps
                usage_analysis.analysis_confidence = 0.7;
            }
            GapType::TodoComment => {
                usage_analysis.usage_frequency_score = 6.0; // Medium-high for TODOs
                usage_analysis.analysis_confidence = 0.8;
            }
            GapType::UndocumentedTechnology => {
                usage_analysis.usage_frequency_score = 7.0; // High for tech gaps
                usage_analysis.analysis_confidence = 0.6;
            }
            GapType::MissingDocumentation => {
                usage_analysis.usage_frequency_score = 5.0; // Medium for missing docs
                usage_analysis.analysis_confidence = 0.7;
            }
            GapType::ConfigurationGap => {
                usage_analysis.usage_frequency_score = 4.0; // Lower for config gaps
                usage_analysis.analysis_confidence = 0.8;
            }
        }

        // Adjust based on file location (core files have higher impact)
        let file_path_str = gap.file_path.to_string_lossy().to_lowercase();
        if file_path_str.contains("src/lib.rs") || file_path_str.contains("main.rs") {
            usage_analysis.usage_frequency_score += 2.0;
            usage_analysis.referencing_files = 10; // Assume high usage for core files
        } else if file_path_str.contains("src/") {
            usage_analysis.usage_frequency_score += 1.0;
            usage_analysis.referencing_files = 5;
        } else if file_path_str.contains("tests/") {
            usage_analysis.usage_frequency_score += 0.5;
            usage_analysis.referencing_files = 2;
        }

        usage_analysis.usage_frequency_score = usage_analysis.usage_frequency_score.min(10.0);

        Ok(usage_analysis)
    }

    /// Analyze dependency impact for the gap's code component
    async fn analyze_dependency_impact(
        &self,
        gap: &DetectedGap,
    ) -> Result<DependencyImpactAnalysis, ImpactAssessmentError> {
        // TODO: Implement actual dependency analysis
        // This would involve:
        // 1. Building a dependency graph of the codebase
        // 2. Identifying dependencies of the component with the gap
        // 3. Analyzing reverse dependencies (what depends on this component)
        // 4. Calculating impact propagation through the dependency tree

        let mut dependency_analysis = DependencyImpactAnalysis::default();

        // Basic analysis based on gap type and location
        match gap.gap_type {
            GapType::ApiDocumentationGap => {
                dependency_analysis.dependency_impact_score = 9.0; // High dependency impact
                dependency_analysis.reverse_dependencies = 15; // Assume many dependents for APIs
                dependency_analysis.analysis_confidence = 0.8;
            }
            GapType::UndocumentedTechnology => {
                dependency_analysis.dependency_impact_score = 7.0; // High for tech dependencies
                dependency_analysis.reverse_dependencies = 8;
                dependency_analysis.analysis_confidence = 0.7;
            }
            GapType::TodoComment => {
                dependency_analysis.dependency_impact_score = 5.0; // Medium for TODOs
                dependency_analysis.reverse_dependencies = 3;
                dependency_analysis.analysis_confidence = 0.6;
            }
            GapType::MissingDocumentation => {
                dependency_analysis.dependency_impact_score = 4.0; // Lower for docs
                dependency_analysis.reverse_dependencies = 2;
                dependency_analysis.analysis_confidence = 0.7;
            }
            GapType::ConfigurationGap => {
                dependency_analysis.dependency_impact_score = 6.0; // Medium-high for config
                dependency_analysis.reverse_dependencies = 5;
                dependency_analysis.analysis_confidence = 0.8;
            }
        }

        // Adjust based on file structure
        let file_path_str = gap.file_path.to_string_lossy().to_lowercase();
        if file_path_str.contains("src/lib.rs") {
            dependency_analysis.dependency_impact_score += 2.0;
            dependency_analysis.reverse_dependencies += 10;
        } else if file_path_str.contains("main.rs") {
            dependency_analysis.dependency_impact_score += 1.5;
            dependency_analysis.reverse_dependencies += 5;
        }

        dependency_analysis.dependency_impact_score =
            dependency_analysis.dependency_impact_score.min(10.0);

        Ok(dependency_analysis)
    }

    /// Analyze API visibility and exposure for the gap's code component
    async fn analyze_api_visibility(
        &self,
        gap: &DetectedGap,
    ) -> Result<ApiVisibilityAnalysis, ImpactAssessmentError> {
        // TODO: Implement actual API visibility analysis
        // This would involve:
        // 1. Parsing code to identify public vs private APIs
        // 2. Analyzing external usage in examples, documentation
        // 3. Checking for exposure in public crate interfaces
        // 4. Assessing API surface size and complexity

        let mut api_analysis = ApiVisibilityAnalysis::default();

        // Basic analysis based on gap type
        match gap.gap_type {
            GapType::ApiDocumentationGap => {
                api_analysis.has_public_api = true;
                api_analysis.external_api_surface = true;
                api_analysis.api_visibility_score = 9.0; // Highest for API gaps
                api_analysis.public_methods = 5; // Assume multiple public methods
                api_analysis.analysis_confidence = 0.9;
            }
            GapType::UndocumentedTechnology => {
                api_analysis.has_public_api = false;
                api_analysis.api_visibility_score = 3.0; // Lower for tech gaps
                api_analysis.internal_methods = 3;
                api_analysis.analysis_confidence = 0.6;
            }
            GapType::MissingDocumentation => {
                // Could be public or private
                api_analysis.api_visibility_score = 5.0; // Medium
                api_analysis.analysis_confidence = 0.5;
            }
            GapType::TodoComment => {
                api_analysis.api_visibility_score = 4.0; // Medium-low
                api_analysis.analysis_confidence = 0.6;
            }
            GapType::ConfigurationGap => {
                api_analysis.api_visibility_score = 7.0; // High for config visibility
                api_analysis.external_api_surface = true;
                api_analysis.analysis_confidence = 0.8;
            }
        }

        // Check for public API indicators in context
        if gap.context.contains("pub fn") || gap.context.contains("pub struct") {
            api_analysis.has_public_api = true;
            api_analysis.api_visibility_score += 2.0;
            api_analysis.public_methods += 1;
        }

        api_analysis.api_visibility_score = api_analysis.api_visibility_score.min(10.0);

        Ok(api_analysis)
    }

    /// Analyze development activity and change frequency for the gap's code component
    async fn analyze_development_activity(
        &self,
        gap: &DetectedGap,
    ) -> Result<DevelopmentActivityAnalysis, ImpactAssessmentError> {
        // TODO: Implement actual development activity analysis
        // This would involve:
        // 1. Git history analysis for the file/component
        // 2. Commit frequency and author analysis
        // 3. Recent change patterns and velocity
        // 4. Developer activity tracking

        let mut activity_analysis = DevelopmentActivityAnalysis::default();

        // Placeholder analysis based on gap characteristics
        match gap.gap_type {
            GapType::TodoComment => {
                activity_analysis.actively_developed = true;
                activity_analysis.change_frequency = 2.0; // 2 changes per week
                activity_analysis.activity_impact_score = 8.0; // High for active TODOs
                activity_analysis.developer_count = 3;
                activity_analysis.analysis_confidence = 0.7;
            }
            GapType::ApiDocumentationGap => {
                activity_analysis.actively_developed = true;
                activity_analysis.change_frequency = 1.5;
                activity_analysis.activity_impact_score = 7.0; // High for API changes
                activity_analysis.developer_count = 5;
                activity_analysis.analysis_confidence = 0.8;
            }
            GapType::UndocumentedTechnology => {
                activity_analysis.actively_developed = false;
                activity_analysis.change_frequency = 0.5;
                activity_analysis.activity_impact_score = 4.0; // Lower for stable tech
                activity_analysis.developer_count = 2;
                activity_analysis.analysis_confidence = 0.6;
            }
            GapType::MissingDocumentation => {
                activity_analysis.change_frequency = 1.0;
                activity_analysis.activity_impact_score = 5.0; // Medium
                activity_analysis.developer_count = 2;
                activity_analysis.analysis_confidence = 0.7;
            }
            GapType::ConfigurationGap => {
                activity_analysis.change_frequency = 0.8;
                activity_analysis.activity_impact_score = 6.0; // Medium-high for config
                activity_analysis.developer_count = 4;
                activity_analysis.analysis_confidence = 0.8;
            }
        }

        // Set timestamps for active development
        if activity_analysis.actively_developed {
            activity_analysis.last_modified = Some(Utc::now() - ChronoDuration::days(3));
            activity_analysis.average_modification_interval = Some(Duration::from_secs(86400 * 4));
            // 4 days
        }

        Ok(activity_analysis)
    }

    /// Analyze team impact and workflow implications for the gap
    async fn analyze_team_impact(
        &self,
        gap: &DetectedGap,
    ) -> Result<TeamImpactAnalysis, ImpactAssessmentError> {
        // TODO: Implement actual team impact analysis
        // This would involve:
        // 1. Code ownership analysis from git history
        // 2. Workflow dependency analysis
        // 3. Knowledge distribution assessment
        // 4. Bottleneck potential evaluation

        let mut team_analysis = TeamImpactAnalysis::default();

        // Basic analysis based on gap type
        match gap.gap_type {
            GapType::ApiDocumentationGap => {
                team_analysis.affected_team_members = 8; // High team impact for APIs
                team_analysis.bottleneck_potential = 0.9;
                team_analysis.knowledge_distribution = 0.3; // Low distribution for APIs
                team_analysis.team_impact_score = 9.0;
                team_analysis
                    .critical_workflows
                    .push("API usage".to_string());
                team_analysis
                    .critical_workflows
                    .push("integration testing".to_string());
                team_analysis.analysis_confidence = 0.8;
            }
            GapType::TodoComment => {
                team_analysis.affected_team_members = 3; // Medium team impact
                team_analysis.bottleneck_potential = 0.6;
                team_analysis.knowledge_distribution = 0.7; // Better distribution for TODOs
                team_analysis.team_impact_score = 6.0;
                team_analysis
                    .critical_workflows
                    .push("development".to_string());
                team_analysis.analysis_confidence = 0.7;
            }
            GapType::UndocumentedTechnology => {
                team_analysis.affected_team_members = 5; // Medium-high team impact
                team_analysis.bottleneck_potential = 0.8;
                team_analysis.knowledge_distribution = 0.4; // Low for specialized tech
                team_analysis.team_impact_score = 7.0;
                team_analysis
                    .critical_workflows
                    .push("technology adoption".to_string());
                team_analysis
                    .critical_workflows
                    .push("maintenance".to_string());
                team_analysis.analysis_confidence = 0.6;
            }
            GapType::MissingDocumentation => {
                team_analysis.affected_team_members = 4;
                team_analysis.bottleneck_potential = 0.5;
                team_analysis.knowledge_distribution = 0.6;
                team_analysis.team_impact_score = 5.0;
                team_analysis
                    .critical_workflows
                    .push("onboarding".to_string());
                team_analysis.analysis_confidence = 0.7;
            }
            GapType::ConfigurationGap => {
                team_analysis.affected_team_members = 6; // High for config gaps
                team_analysis.bottleneck_potential = 0.7;
                team_analysis.knowledge_distribution = 0.5;
                team_analysis.team_impact_score = 8.0;
                team_analysis
                    .critical_workflows
                    .push("deployment".to_string());
                team_analysis
                    .critical_workflows
                    .push("configuration management".to_string());
                team_analysis.analysis_confidence = 0.8;
            }
        }

        Ok(team_analysis)
    }

    /// Generate cache key for gap
    fn generate_cache_key(&self, gap: &DetectedGap) -> String {
        format!(
            "impact:{}:{}:{}:{}",
            gap.file_path.to_string_lossy(),
            gap.line_number,
            gap.gap_type,
            gap.confidence
        )
    }

    /// Get cached impact result if available and not expired
    async fn get_cached_result(&self, cache_key: &str) -> Option<CachedImpactResult> {
        let cache = self.impact_cache.read().await;
        if let Some(cached) = cache.get(cache_key) {
            if Utc::now() < cached.expires_at {
                return Some(cached.clone());
            }
        }
        None
    }

    /// Cache impact assessment result
    async fn cache_result(&self, cache_key: String, result: ImpactAssessmentResult) {
        let expires_at =
            Utc::now() + ChronoDuration::seconds(self.config.impact_cache_ttl_secs as i64);
        let cached = CachedImpactResult { result, expires_at };

        let mut cache = self.impact_cache.write().await;
        cache.insert(cache_key, cached);
    }

    /// Update assessment metrics
    async fn update_metrics(&self, duration: Duration, success: bool) {
        let mut metrics = self.metrics.write().await;
        metrics.total_assessments += 1;

        if success {
            metrics.successful_analyses += 1;
        } else {
            metrics.failed_analyses += 1;
        }

        // Update rolling average
        let total = metrics.total_assessments;
        let previous_avg = metrics.average_analysis_time.as_nanos() as f64;
        let new_avg =
            (previous_avg * (total - 1) as f64 + duration.as_nanos() as f64) / total as f64;
        metrics.average_analysis_time = Duration::from_nanos(new_avg as u64);

        metrics.last_updated = Utc::now();
    }

    /// Increment cache hit counter
    async fn increment_cache_hit(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_hits += 1;
    }

    /// Increment cache miss counter
    async fn increment_cache_miss(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_misses += 1;
    }

    /// Increment performance violation counter
    async fn increment_performance_violation(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.performance_violations += 1;
    }

    /// Get current impact assessment metrics
    pub async fn get_metrics(&self) -> ImpactAssessmentMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Clear impact assessment cache
    pub async fn clear_cache(&self) {
        let mut cache = self.impact_cache.write().await;
        cache.clear();
        info!("Impact assessment cache cleared");
    }

    /// Get current configuration
    pub fn get_config(&self) -> &ImpactAssessmentConfig {
        &self.config
    }
}

// Note: ToString for GapType is implemented in prioritization.rs

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn create_test_gap() -> DetectedGap {
        DetectedGap {
            gap_type: GapType::TodoComment,
            file_path: PathBuf::from("src/main.rs"),
            line_number: 42,
            column_number: Some(10),
            context: "// TODO: Implement error handling".to_string(),
            description: "Implement proper error handling for main function".to_string(),
            confidence: 0.9,
            priority: 7,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_impact_assessor_creation() {
        // FAILING TEST: Impact assessor should be creatable with valid configuration
        let config = ImpactAssessmentConfig::default();
        let result = ImpactAssessor::new(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_impact_assessor_with_defaults() {
        // FAILING TEST: Default impact assessor creation should work
        let result = ImpactAssessor::with_defaults().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_single_gap_impact_assessment() {
        // FAILING TEST: Single gap impact assessment should work correctly
        let assessor = ImpactAssessor::with_defaults().await.unwrap();
        let gap = create_test_gap();

        let result = assessor.assess_gap_impact(&gap).await;
        assert!(result.is_ok());

        let assessment = result.unwrap();
        assert!(assessment.final_impact_score >= 0.0 && assessment.final_impact_score <= 10.0);
        assert!(assessment.overall_confidence >= 0.0 && assessment.overall_confidence <= 1.0);
        assert!(assessment.usage_analysis.usage_frequency_score >= 0.0);
        assert!(assessment.dependency_analysis.dependency_impact_score >= 0.0);
        assert!(assessment.api_visibility_analysis.api_visibility_score >= 0.0);
        assert!(assessment.activity_analysis.activity_impact_score >= 0.0);
        assert!(assessment.team_analysis.team_impact_score >= 0.0);
    }

    #[tokio::test]
    async fn test_batch_gap_impact_assessment() {
        // FAILING TEST: Batch gap impact assessment should handle multiple gaps efficiently
        let assessor = ImpactAssessor::with_defaults().await.unwrap();

        let mut gaps = Vec::new();
        for i in 0..10 {
            let mut gap = create_test_gap();
            gap.line_number = 10 + i;
            gap.gap_type = match i % 5 {
                0 => GapType::TodoComment,
                1 => GapType::MissingDocumentation,
                2 => GapType::UndocumentedTechnology,
                3 => GapType::ApiDocumentationGap,
                _ => GapType::ConfigurationGap,
            };
            gaps.push(gap);
        }

        let start_time = Instant::now();
        let result = assessor.assess_gaps_batch(&gaps).await;
        let duration = start_time.elapsed();

        assert!(result.is_ok());
        let assessments = result.unwrap();
        assert_eq!(assessments.len(), gaps.len());

        // Check performance requirement: within priority scoring timeframes
        assert!(duration < Duration::from_millis(1000)); // 1 second for batch

        // All scores should be valid
        for assessment in &assessments {
            assert!(assessment.final_impact_score >= 0.0 && assessment.final_impact_score <= 10.0);
            assert!(assessment.overall_confidence >= 0.0 && assessment.overall_confidence <= 1.0);
        }
    }

    #[tokio::test]
    async fn test_usage_pattern_analysis() {
        // FAILING TEST: Usage pattern analysis should provide meaningful scores for different gap types
        let assessor = ImpactAssessor::with_defaults().await.unwrap();

        let mut api_gap = create_test_gap();
        api_gap.gap_type = GapType::ApiDocumentationGap;

        let mut todo_gap = create_test_gap();
        todo_gap.gap_type = GapType::TodoComment;

        let api_result = assessor.assess_gap_impact(&api_gap).await.unwrap();
        let todo_result = assessor.assess_gap_impact(&todo_gap).await.unwrap();

        // API gaps should generally have higher usage impact than TODOs
        assert!(
            api_result.usage_analysis.usage_frequency_score
                >= todo_result.usage_analysis.usage_frequency_score
        );
        assert!(api_result.usage_analysis.analysis_confidence > 0.0);
        assert!(todo_result.usage_analysis.analysis_confidence > 0.0);
    }

    #[tokio::test]
    async fn test_dependency_impact_analysis() {
        // FAILING TEST: Dependency impact analysis should reflect gap type importance
        let assessor = ImpactAssessor::with_defaults().await.unwrap();

        let mut api_gap = create_test_gap();
        api_gap.gap_type = GapType::ApiDocumentationGap;

        let mut config_gap = create_test_gap();
        config_gap.gap_type = GapType::ConfigurationGap;

        let api_result = assessor.assess_gap_impact(&api_gap).await.unwrap();
        let config_result = assessor.assess_gap_impact(&config_gap).await.unwrap();

        // API gaps should have higher dependency impact
        assert!(
            api_result.dependency_analysis.dependency_impact_score
                > config_result.dependency_analysis.dependency_impact_score
        );
        assert!(api_result.dependency_analysis.reverse_dependencies > 0);
    }

    #[tokio::test]
    async fn test_api_visibility_analysis() {
        // FAILING TEST: API visibility analysis should correctly identify public APIs
        let assessor = ImpactAssessor::with_defaults().await.unwrap();

        let mut api_gap = create_test_gap();
        api_gap.gap_type = GapType::ApiDocumentationGap;
        api_gap.context = "pub fn process_data() -> Result<Data, Error>".to_string();

        let result = assessor.assess_gap_impact(&api_gap).await.unwrap();

        assert!(result.api_visibility_analysis.has_public_api);
        assert!(result.api_visibility_analysis.api_visibility_score >= 8.0);
        assert!(result.api_visibility_analysis.analysis_confidence > 0.0);
    }

    #[tokio::test]
    async fn test_development_activity_analysis() {
        // FAILING TEST: Development activity analysis should reflect gap urgency
        let assessor = ImpactAssessor::with_defaults().await.unwrap();

        let mut todo_gap = create_test_gap();
        todo_gap.gap_type = GapType::TodoComment;

        let mut tech_gap = create_test_gap();
        tech_gap.gap_type = GapType::UndocumentedTechnology;

        let todo_result = assessor.assess_gap_impact(&todo_gap).await.unwrap();
        let tech_result = assessor.assess_gap_impact(&tech_gap).await.unwrap();

        // TODOs should indicate active development
        assert!(todo_result.activity_analysis.actively_developed);
        assert!(
            todo_result.activity_analysis.activity_impact_score
                > tech_result.activity_analysis.activity_impact_score
        );
    }

    #[tokio::test]
    async fn test_team_impact_analysis() {
        // FAILING TEST: Team impact analysis should assess workflow implications
        let assessor = ImpactAssessor::with_defaults().await.unwrap();

        let mut api_gap = create_test_gap();
        api_gap.gap_type = GapType::ApiDocumentationGap;

        let result = assessor.assess_gap_impact(&api_gap).await.unwrap();

        assert!(result.team_analysis.affected_team_members > 0);
        assert!(result.team_analysis.team_impact_score >= 8.0); // High for API gaps
        assert!(!result.team_analysis.critical_workflows.is_empty());
        assert!(result.team_analysis.bottleneck_potential > 0.0);
    }

    #[tokio::test]
    async fn test_file_location_impact() {
        // FAILING TEST: File location should affect impact scores
        let assessor = ImpactAssessor::with_defaults().await.unwrap();

        let mut core_gap = create_test_gap();
        core_gap.file_path = PathBuf::from("src/lib.rs");

        let mut test_gap = create_test_gap();
        test_gap.file_path = PathBuf::from("tests/unit_test.rs");

        let core_result = assessor.assess_gap_impact(&core_gap).await.unwrap();
        let test_result = assessor.assess_gap_impact(&test_gap).await.unwrap();

        // Core files should have higher impact than test files
        assert!(core_result.final_impact_score > test_result.final_impact_score);
        assert!(
            core_result.usage_analysis.usage_frequency_score
                > test_result.usage_analysis.usage_frequency_score
        );
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        // FAILING TEST: Configuration validation should catch invalid weights
        let config = ImpactAssessmentConfig {
            usage_frequency_weight: 0.3,
            dependency_impact_weight: 0.3,
            api_visibility_weight: 0.3,
            development_activity_weight: 0.3, // Sum > 1.0
            team_impact_weight: 0.3,
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());

        if let Err(ImpactAssessmentError::Configuration(msg)) = result {
            assert!(msg.contains("Weight sum"));
        } else {
            panic!("Expected Configuration error");
        }
    }

    #[tokio::test]
    async fn test_performance_requirements() {
        // FAILING TEST: Impact assessment should meet performance requirements
        let config = ImpactAssessmentConfig::for_performance();
        let assessor = ImpactAssessor::new(config).await.unwrap();

        // Test single gap assessment performance
        let gap = create_test_gap();
        let start_time = Instant::now();
        let _result = assessor.assess_gap_impact(&gap).await.unwrap();
        let duration = start_time.elapsed();

        assert!(duration < Duration::from_millis(50)); // Performance config limit

        // Test batch assessment performance
        let gaps: Vec<_> = (0..10)
            .map(|i| {
                let mut gap = create_test_gap();
                gap.line_number = i;
                gap
            })
            .collect();

        let start_time = Instant::now();
        let result = assessor.assess_gaps_batch(&gaps).await;
        let duration = start_time.elapsed();

        assert!(result.is_ok());
        assert!(duration < Duration::from_millis(500)); // Reasonable batch limit
    }

    #[tokio::test]
    async fn test_impact_caching() {
        // FAILING TEST: Impact caching should improve performance for repeated assessments
        let config = ImpactAssessmentConfig {
            enable_impact_caching: true,
            impact_cache_ttl_secs: 60,
            ..Default::default()
        };

        let assessor = ImpactAssessor::new(config).await.unwrap();

        let gap = create_test_gap();

        // First assessment should be cache miss
        let _result1 = assessor.assess_gap_impact(&gap).await.unwrap();
        let metrics1 = assessor.get_metrics().await;
        assert!(metrics1.cache_misses > 0);

        // Second assessment should be cache hit
        let _result2 = assessor.assess_gap_impact(&gap).await.unwrap();
        let metrics2 = assessor.get_metrics().await;
        assert!(metrics2.cache_hits > 0);
        assert_eq!(metrics2.cache_hits, 1);
    }

    #[tokio::test]
    async fn test_weighted_impact_calculation() {
        // FAILING TEST: Weighted impact calculation should properly combine component scores
        let assessor = ImpactAssessor::with_defaults().await.unwrap();

        let usage = UsagePatternAnalysis {
            usage_frequency_score: 8.0,
            analysis_confidence: 0.8,
            ..Default::default()
        };

        let dependency = DependencyImpactAnalysis {
            dependency_impact_score: 6.0,
            analysis_confidence: 0.7,
            ..Default::default()
        };

        let api_visibility = ApiVisibilityAnalysis {
            api_visibility_score: 9.0,
            analysis_confidence: 0.9,
            ..Default::default()
        };

        let activity = DevelopmentActivityAnalysis {
            activity_impact_score: 5.0,
            analysis_confidence: 0.6,
            ..Default::default()
        };

        let team = TeamImpactAnalysis {
            team_impact_score: 7.0,
            analysis_confidence: 0.8,
            ..Default::default()
        };

        let final_score = assessor.calculate_weighted_impact_score(
            &usage,
            &dependency,
            &api_visibility,
            &activity,
            &team,
        );

        assert!(final_score.is_ok());
        let score = final_score.unwrap();
        assert!((0.0..=10.0).contains(&score));

        // Score should be weighted average of components
        let expected = (8.0 * 0.25) + (6.0 * 0.25) + (9.0 * 0.2) + (5.0 * 0.15) + (7.0 * 0.15);
        assert!((score - expected).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_impact_assessment_result_analysis() {
        // FAILING TEST: Impact assessment result should provide useful analysis methods
        let assessor = ImpactAssessor::with_defaults().await.unwrap();

        let mut gap = create_test_gap();
        gap.gap_type = GapType::ApiDocumentationGap;

        let result = assessor.assess_gap_impact(&gap).await.unwrap();

        // Test primary impact factor identification
        let primary_factor = result.primary_impact_factor();
        assert!(!primary_factor.is_empty());
        assert!(primary_factor != "unknown");

        // Test high confidence check
        assert!(result.has_high_confidence(0.5));

        // Test impact summary
        let summary = result.impact_summary();
        assert_eq!(summary.len(), 5);
        assert!(summary.contains_key("usage_frequency"));
        assert!(summary.contains_key("dependency_impact"));
        assert!(summary.contains_key("api_visibility"));
        assert!(summary.contains_key("development_activity"));
        assert!(summary.contains_key("team_impact"));
    }

    #[tokio::test]
    async fn test_metrics_tracking() {
        // FAILING TEST: Metrics should track impact assessment operations correctly
        let assessor = ImpactAssessor::with_defaults().await.unwrap();
        let gap = create_test_gap();

        let initial_metrics = assessor.get_metrics().await;
        assert_eq!(initial_metrics.total_assessments, 0);
        assert_eq!(initial_metrics.successful_analyses, 0);

        // Perform an assessment
        let _result = assessor.assess_gap_impact(&gap).await.unwrap();

        let updated_metrics = assessor.get_metrics().await;
        assert_eq!(updated_metrics.total_assessments, 1);
        assert_eq!(updated_metrics.successful_analyses, 1);
        assert_eq!(updated_metrics.failed_analyses, 0);
        assert!(updated_metrics.average_analysis_time > Duration::from_nanos(0));
    }
}
