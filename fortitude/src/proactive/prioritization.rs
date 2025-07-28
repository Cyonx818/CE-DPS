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

// ABOUTME: Intelligent prioritization algorithms for knowledge gap research urgency assessment
//! This module provides sophisticated prioritization algorithms that intelligently rank research tasks
//! based on multiple urgency factors. The system evaluates research urgency using:
//! - Gap type urgency assessment (TODO vs API docs vs missing tech documentation)
//! - Recency weighting (recently discovered or changing gaps get higher priority)
//! - Impact potential scoring (based on code visibility, API usage, team impact)
//! - User context awareness (development phase, project deadlines, team preferences)
//! - Multi-dimensional scoring with configurable weights for different prioritization strategies
//!
//! Performance Requirements:
//! - Priority scoring <100ms for up to 50 identified gaps
//! - Algorithm scalability for 100+ gaps without degradation
//! - Memory-efficient scoring with minimal allocations
//! - Integration overhead <10ms with existing systems

use crate::proactive::context_aware_scorer::{
    ContextAwarePriorityBreakdown, ContextAwarePriorityScorer, ContextAwareScoringConfig,
};
use crate::proactive::impact_assessor::{
    ImpactAssessmentConfig, ImpactAssessmentError, ImpactAssessmentResult, ImpactAssessor,
};
use crate::proactive::{DetectedGap, GapType, TaskPriority};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

/// Errors that can occur during prioritization operations
#[derive(Error, Debug)]
pub enum PrioritizationError {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Invalid priority score: {score}, must be between 0.0 and 10.0")]
    InvalidPriorityScore { score: f64 },

    #[error("Gap analysis failed: {0}")]
    GapAnalysis(String),

    #[error("Performance threshold exceeded: scoring took {duration:?}, limit is {limit:?}")]
    PerformanceThreshold { duration: Duration, limit: Duration },

    #[error("Batch size too large: {size}, maximum allowed is {max}")]
    BatchSizeTooLarge { size: usize, max: usize },

    #[error("Context analysis failed: {0}")]
    ContextAnalysis(String),

    #[error("Weight validation failed: sum of weights {sum:.3} must equal 1.0")]
    WeightValidation { sum: f64 },

    #[error("Context-aware scoring error: {0}")]
    ContextAwareScoring(String),

    #[error("Impact assessment error: {0}")]
    ImpactAssessment(#[from] ImpactAssessmentError),
}

/// Development context for priority assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentContext {
    /// Current development phase (prototyping, development, testing, production)
    pub phase: DevelopmentPhase,
    /// Project deadlines affecting priority
    pub has_urgent_deadlines: bool,
    /// Team size affecting impact assessment
    pub team_size: usize,
    /// Whether this is a public API project
    pub is_public_api: bool,
    /// Performance criticality of the project
    pub performance_critical: bool,
    /// User-defined priority boosts for specific gap types
    pub custom_boosts: HashMap<GapType, f64>,
}

impl Default for DevelopmentContext {
    fn default() -> Self {
        Self {
            phase: DevelopmentPhase::Development,
            has_urgent_deadlines: false,
            team_size: 3,
            is_public_api: false,
            performance_critical: false,
            custom_boosts: HashMap::new(),
        }
    }
}

/// Development phase affecting priority scoring
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevelopmentPhase {
    Prototyping,
    Development,
    Testing,
    PreProduction,
    Production,
}

impl DevelopmentPhase {
    /// Get urgency multiplier for this phase
    pub fn urgency_multiplier(&self) -> f64 {
        match self {
            DevelopmentPhase::Prototyping => 0.7, // Lower urgency during prototyping
            DevelopmentPhase::Development => 1.0, // Normal urgency
            DevelopmentPhase::Testing => 1.3,     // Higher urgency during testing
            DevelopmentPhase::PreProduction => 1.5, // High urgency before production
            DevelopmentPhase::Production => 2.0,  // Critical urgency in production
        }
    }
}

/// Configuration for prioritization algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizationConfig {
    /// Weight for gap type urgency in final score (0.0-1.0)
    pub gap_type_weight: f64,
    /// Weight for recency in final score (0.0-1.0)
    pub recency_weight: f64,
    /// Weight for impact assessment in final score (0.0-1.0)
    pub impact_weight: f64,
    /// Weight for context factors in final score (0.0-1.0)
    pub context_weight: f64,
    /// Maximum time allowed for priority scoring (milliseconds)
    pub max_scoring_time_ms: u64,
    /// Maximum batch size for simultaneous scoring
    pub max_batch_size: usize,
    /// Minimum confidence threshold for scoring
    pub min_confidence_threshold: f64,
    /// Recency decay factor (how quickly recency importance fades)
    pub recency_decay_hours: f64,
    /// Enable caching of priority scores
    pub enable_score_caching: bool,
    /// Cache TTL for priority scores (seconds)
    pub score_cache_ttl_secs: u64,
    /// Enable context-aware priority scoring using classification
    pub enable_context_aware_scoring: bool,
    /// Configuration for context-aware scoring
    pub context_aware_config: Option<ContextAwareScoringConfig>,
    /// Enable impact assessment based on code usage patterns
    pub enable_impact_assessment: bool,
    /// Configuration for impact assessment
    pub impact_assessment_config: Option<ImpactAssessmentConfig>,
}

impl Default for PrioritizationConfig {
    fn default() -> Self {
        Self {
            gap_type_weight: 0.4,
            recency_weight: 0.2,
            impact_weight: 0.3,
            context_weight: 0.1,
            max_scoring_time_ms: 100,
            max_batch_size: 50,
            min_confidence_threshold: 0.5,
            recency_decay_hours: 24.0,
            enable_score_caching: true,
            score_cache_ttl_secs: 300,           // 5 minutes
            enable_context_aware_scoring: false, // Disabled by default for backward compatibility
            context_aware_config: None,
            enable_impact_assessment: false, // Disabled by default for backward compatibility
            impact_assessment_config: None,
        }
    }
}

impl PrioritizationConfig {
    /// Validate configuration weights sum to 1.0
    pub fn validate(&self) -> Result<(), PrioritizationError> {
        let weight_sum =
            self.gap_type_weight + self.recency_weight + self.impact_weight + self.context_weight;

        if (weight_sum - 1.0).abs() > 0.001 {
            return Err(PrioritizationError::WeightValidation { sum: weight_sum });
        }

        if self.max_batch_size == 0 {
            return Err(PrioritizationError::Configuration(
                "max_batch_size must be greater than 0".to_string(),
            ));
        }

        if self.max_scoring_time_ms == 0 {
            return Err(PrioritizationError::Configuration(
                "max_scoring_time_ms must be greater than 0".to_string(),
            ));
        }

        if !(0.0..=1.0).contains(&self.min_confidence_threshold) {
            return Err(PrioritizationError::Configuration(
                "min_confidence_threshold must be between 0.0 and 1.0".to_string(),
            ));
        }

        Ok(())
    }

    /// Create configuration optimized for performance
    pub fn for_performance() -> Self {
        Self {
            max_scoring_time_ms: 50,
            max_batch_size: 25,
            enable_score_caching: true,
            score_cache_ttl_secs: 600, // 10 minutes
            ..Default::default()
        }
    }

    /// Create configuration optimized for accuracy
    pub fn for_accuracy() -> Self {
        Self {
            max_scoring_time_ms: 200,
            max_batch_size: 100,
            min_confidence_threshold: 0.3, // Lower threshold for more comprehensive scoring
            ..Default::default()
        }
    }

    /// Create configuration with context-aware scoring enabled
    pub fn with_context_aware_scoring() -> Self {
        Self {
            enable_context_aware_scoring: true,
            context_aware_config: Some(ContextAwareScoringConfig::default()),
            max_scoring_time_ms: 150, // Slightly higher to account for classification overhead
            ..Default::default()
        }
    }

    /// Create configuration with context-aware scoring and custom config
    pub fn with_custom_context_aware(context_config: ContextAwareScoringConfig) -> Self {
        Self {
            enable_context_aware_scoring: true,
            context_aware_config: Some(context_config),
            max_scoring_time_ms: 150,
            ..Default::default()
        }
    }

    /// Create configuration with impact assessment enabled
    pub fn with_impact_assessment() -> Self {
        Self {
            enable_impact_assessment: true,
            impact_assessment_config: Some(ImpactAssessmentConfig::default()),
            max_scoring_time_ms: 200, // Higher to account for impact analysis overhead
            ..Default::default()
        }
    }

    /// Create configuration with impact assessment and custom config
    pub fn with_custom_impact_assessment(impact_config: ImpactAssessmentConfig) -> Self {
        Self {
            enable_impact_assessment: true,
            impact_assessment_config: Some(impact_config),
            max_scoring_time_ms: 200,
            ..Default::default()
        }
    }

    /// Create configuration with both context-aware scoring and impact assessment
    pub fn with_full_enhancement() -> Self {
        Self {
            enable_context_aware_scoring: true,
            context_aware_config: Some(ContextAwareScoringConfig::default()),
            enable_impact_assessment: true,
            impact_assessment_config: Some(ImpactAssessmentConfig::default()),
            max_scoring_time_ms: 250, // Higher for both enhancements
            ..Default::default()
        }
    }
}

/// Priority score breakdown for analysis and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityScoreBreakdown {
    /// Final calculated priority score (0.0-10.0)
    pub final_score: f64,
    /// Gap type urgency component score
    pub gap_type_score: f64,
    /// Recency component score
    pub recency_score: f64,
    /// Impact assessment component score
    pub impact_score: f64,
    /// Context factors component score
    pub context_score: f64,
    /// Final priority level assigned
    pub priority_level: TaskPriority,
    /// Confidence in the priority assessment
    pub confidence: f64,
    /// Calculation timestamp
    pub calculated_at: DateTime<Utc>,
    /// Detailed impact assessment result (if enabled)
    pub impact_assessment: Option<ImpactAssessmentResult>,
}

/// Cached priority score entry
#[derive(Debug, Clone)]
struct CachedScore {
    breakdown: PriorityScoreBreakdown,
    expires_at: DateTime<Utc>,
}

/// Prioritization metrics for monitoring and optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizationMetrics {
    pub total_scores_calculated: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub batch_operations: u64,
    pub average_scoring_time: Duration,
    pub performance_violations: u64,
    pub last_updated: DateTime<Utc>,
}

impl Default for PrioritizationMetrics {
    fn default() -> Self {
        Self {
            total_scores_calculated: 0,
            cache_hits: 0,
            cache_misses: 0,
            batch_operations: 0,
            average_scoring_time: Duration::from_millis(0),
            performance_violations: 0,
            last_updated: Utc::now(),
        }
    }
}

/// Priority scorer for research gap urgency assessment
pub struct PriorityScorer {
    config: PrioritizationConfig,
    context: DevelopmentContext,
    score_cache: Arc<RwLock<HashMap<String, CachedScore>>>,
    metrics: Arc<RwLock<PrioritizationMetrics>>,
    context_aware_scorer: Option<ContextAwarePriorityScorer>,
    impact_assessor: Option<ImpactAssessor>,
}

impl PriorityScorer {
    /// Create a new priority scorer with the given configuration
    #[instrument(level = "debug", skip(config, context))]
    pub async fn new(
        config: PrioritizationConfig,
        context: DevelopmentContext,
    ) -> Result<Self, PrioritizationError> {
        config.validate()?;

        info!("Initializing priority scorer with weights: gap_type={}, recency={}, impact={}, context={}",
              config.gap_type_weight, config.recency_weight, config.impact_weight, config.context_weight);

        // Initialize context-aware scorer if enabled
        let context_aware_scorer = if config.enable_context_aware_scoring {
            if let Some(context_config) = &config.context_aware_config {
                match ContextAwarePriorityScorer::new(context_config.clone()).await {
                    Ok(scorer) => {
                        info!("Context-aware priority scoring enabled");
                        Some(scorer)
                    }
                    Err(e) => {
                        warn!(
                            "Failed to initialize context-aware scorer: {}, continuing without it",
                            e
                        );
                        None
                    }
                }
            } else {
                warn!(
                    "Context-aware scoring enabled but no configuration provided, using defaults"
                );
                match ContextAwarePriorityScorer::with_defaults().await {
                    Ok(scorer) => Some(scorer),
                    Err(e) => {
                        warn!("Failed to initialize default context-aware scorer: {}", e);
                        None
                    }
                }
            }
        } else {
            None
        };

        // Initialize impact assessor if enabled
        let impact_assessor = if config.enable_impact_assessment {
            if let Some(impact_config) = &config.impact_assessment_config {
                match ImpactAssessor::new(impact_config.clone()).await {
                    Ok(assessor) => {
                        info!("Impact assessment enabled");
                        Some(assessor)
                    }
                    Err(e) => {
                        warn!(
                            "Failed to initialize impact assessor: {}, continuing without it",
                            e
                        );
                        None
                    }
                }
            } else {
                warn!("Impact assessment enabled but no configuration provided, using defaults");
                match ImpactAssessor::with_defaults().await {
                    Ok(assessor) => Some(assessor),
                    Err(e) => {
                        warn!("Failed to initialize default impact assessor: {}", e);
                        None
                    }
                }
            }
        } else {
            None
        };

        Ok(Self {
            config,
            context,
            score_cache: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(PrioritizationMetrics::default())),
            context_aware_scorer,
            impact_assessor,
        })
    }

    /// Create a priority scorer with default configuration
    pub async fn with_defaults() -> Result<Self, PrioritizationError> {
        Self::new(
            PrioritizationConfig::default(),
            DevelopmentContext::default(),
        )
        .await
    }

    /// Score priority for a single gap
    #[instrument(level = "debug", skip(self, gap))]
    pub async fn score_gap_priority(
        &self,
        gap: &DetectedGap,
    ) -> Result<PriorityScoreBreakdown, PrioritizationError> {
        let start_time = Instant::now();

        // Check cache first if enabled
        if self.config.enable_score_caching {
            let cache_key = self.generate_cache_key(gap);
            if let Some(cached) = self.get_cached_score(&cache_key).await {
                self.increment_cache_hit().await;
                return Ok(cached.breakdown);
            }
            self.increment_cache_miss().await;
        }

        // Calculate priority score components
        let gap_type_score = self.calculate_gap_type_urgency(gap);
        let recency_score = self.calculate_recency_score(gap);
        let (impact_score, impact_assessment) = self.calculate_impact_score(gap).await?;
        let context_score = self.calculate_context_score(gap);

        // Combine scores using configured weights
        let final_score = (gap_type_score * self.config.gap_type_weight)
            + (recency_score * self.config.recency_weight)
            + (impact_score * self.config.impact_weight)
            + (context_score * self.config.context_weight);

        // Validate final score
        if !(0.0..=10.0).contains(&final_score) {
            return Err(PrioritizationError::InvalidPriorityScore { score: final_score });
        }

        let priority_level = self.score_to_priority_level(final_score);
        let confidence = self.calculate_confidence(gap, final_score);

        let breakdown = PriorityScoreBreakdown {
            final_score,
            gap_type_score,
            recency_score,
            impact_score,
            context_score,
            priority_level,
            confidence,
            calculated_at: Utc::now(),
            impact_assessment,
        };

        // Cache the result if enabled
        if self.config.enable_score_caching {
            let cache_key = self.generate_cache_key(gap);
            self.cache_score(cache_key, breakdown.clone()).await;
        }

        // Check performance
        let duration = start_time.elapsed();
        let limit = Duration::from_millis(self.config.max_scoring_time_ms);
        if duration > limit {
            warn!(
                "Priority scoring exceeded time limit: {:?} > {:?}",
                duration, limit
            );
            self.increment_performance_violation().await;
        }

        self.update_metrics(duration).await;

        debug!(
            "Calculated priority for gap {} with score {:.2} ({})",
            gap.gap_type, final_score, priority_level
        );

        Ok(breakdown)
    }

    /// Score priority for a single gap with context-aware enhancements (if enabled)
    #[instrument(level = "debug", skip(self, gap))]
    pub async fn score_gap_priority_enhanced(
        &self,
        gap: &DetectedGap,
    ) -> Result<ContextAwarePriorityBreakdown, PrioritizationError> {
        // First get the base priority breakdown
        let base_breakdown = self.score_gap_priority(gap).await?;

        // Apply context-aware enhancements if available
        if let Some(context_scorer) = &self.context_aware_scorer {
            match context_scorer
                .enhance_priority_with_context(base_breakdown.clone(), gap)
                .await
            {
                Ok(enhanced_breakdown) => Ok(enhanced_breakdown),
                Err(e) => {
                    warn!(
                        "Context-aware enhancement failed: {}, returning base score",
                        e
                    );
                    // Return a ContextAwarePriorityBreakdown with base data only
                    let final_score = base_breakdown.final_score;
                    Ok(ContextAwarePriorityBreakdown {
                        base_breakdown,
                        extracted_context: None,
                        domain_adjustment: 1.0,
                        audience_adjustment: 1.0,
                        urgency_adjustment: 1.0,
                        confidence_weighting: 0.0,
                        context_enhanced_score: final_score,
                        context_processing_time: Duration::from_millis(0),
                        classification_available: false,
                        used_graceful_degradation: true,
                    })
                }
            }
        } else {
            // No context-aware scoring available, return base breakdown wrapped
            let final_score = base_breakdown.final_score;
            Ok(ContextAwarePriorityBreakdown {
                base_breakdown,
                extracted_context: None,
                domain_adjustment: 1.0,
                audience_adjustment: 1.0,
                urgency_adjustment: 1.0,
                confidence_weighting: 0.0,
                context_enhanced_score: final_score,
                context_processing_time: Duration::from_millis(0),
                classification_available: false,
                used_graceful_degradation: false,
            })
        }
    }

    /// Score priority for multiple gaps with context-aware enhancements in batch
    #[instrument(level = "debug", skip(self, gaps))]
    pub async fn score_gaps_batch_enhanced(
        &self,
        gaps: &[DetectedGap],
    ) -> Result<Vec<ContextAwarePriorityBreakdown>, PrioritizationError> {
        let start_time = Instant::now();

        if gaps.len() > self.config.max_batch_size {
            return Err(PrioritizationError::BatchSizeTooLarge {
                size: gaps.len(),
                max: self.config.max_batch_size,
            });
        }

        info!(
            "Enhanced batch scoring {} gaps for priority assessment",
            gaps.len()
        );

        let mut results = Vec::with_capacity(gaps.len());

        for gap in gaps {
            match self.score_gap_priority_enhanced(gap).await {
                Ok(breakdown) => results.push(breakdown),
                Err(e) => {
                    warn!("Failed to score gap {:?}: {}", gap.gap_type, e);
                    // Continue with other gaps, but note the error
                    continue;
                }
            }
        }

        // Check performance for batch operation
        let duration = start_time.elapsed();
        let per_gap_limit = Duration::from_millis(self.config.max_scoring_time_ms);
        let batch_limit = per_gap_limit * gaps.len() as u32;

        if duration > batch_limit {
            warn!(
                "Enhanced batch priority scoring exceeded time limit: {:?} > {:?}",
                duration, batch_limit
            );
            self.increment_performance_violation().await;
        }

        self.increment_batch_operation().await;

        info!(
            "Completed enhanced batch scoring of {} gaps in {:?}",
            results.len(),
            duration
        );

        Ok(results)
    }

    /// Check if context-aware scoring is enabled and available
    pub fn is_context_aware_enabled(&self) -> bool {
        self.context_aware_scorer.is_some()
    }

    /// Score priority for multiple gaps in batch for improved performance
    #[instrument(level = "debug", skip(self, gaps))]
    pub async fn score_gaps_batch(
        &self,
        gaps: &[DetectedGap],
    ) -> Result<Vec<PriorityScoreBreakdown>, PrioritizationError> {
        let start_time = Instant::now();

        if gaps.len() > self.config.max_batch_size {
            return Err(PrioritizationError::BatchSizeTooLarge {
                size: gaps.len(),
                max: self.config.max_batch_size,
            });
        }

        info!("Batch scoring {} gaps for priority assessment", gaps.len());

        let mut results = Vec::with_capacity(gaps.len());

        for gap in gaps {
            match self.score_gap_priority(gap).await {
                Ok(breakdown) => results.push(breakdown),
                Err(e) => {
                    warn!("Failed to score gap {:?}: {}", gap.gap_type, e);
                    // Continue with other gaps, but note the error
                    continue;
                }
            }
        }

        // Check performance for batch operation
        let duration = start_time.elapsed();
        let per_gap_limit = Duration::from_millis(self.config.max_scoring_time_ms);
        let batch_limit = per_gap_limit * gaps.len() as u32;

        if duration > batch_limit {
            warn!(
                "Batch priority scoring exceeded time limit: {:?} > {:?}",
                duration, batch_limit
            );
            self.increment_performance_violation().await;
        }

        self.increment_batch_operation().await;

        info!(
            "Completed batch scoring of {} gaps in {:?}",
            results.len(),
            duration
        );

        Ok(results)
    }

    /// Calculate gap type urgency score (0.0-10.0)
    fn calculate_gap_type_urgency(&self, gap: &DetectedGap) -> f64 {
        let base_score = match gap.gap_type {
            GapType::TodoComment => 7.0,         // High urgency - actionable items
            GapType::ApiDocumentationGap => 9.0, // Critical - affects users
            GapType::UndocumentedTechnology => 8.0, // High - affects maintainability
            GapType::MissingDocumentation => 6.0, // Medium - affects team understanding
            GapType::ConfigurationGap => 5.0,    // Lower - affects deployment
        };

        // Apply custom boosts if configured
        let boost = self
            .context
            .custom_boosts
            .get(&gap.gap_type)
            .copied()
            .unwrap_or(1.0);

        // Apply confidence weighting
        let confidence_adjusted = base_score * gap.confidence;

        (confidence_adjusted * boost).min(10.0)
    }

    /// Calculate recency score based on gap discovery time (0.0-10.0)
    fn calculate_recency_score(&self, gap: &DetectedGap) -> f64 {
        // For now, we'll use a simple model based on gap priority
        // In a real implementation, this would use gap discovery/modification timestamps

        // Assume higher priority gaps were discovered more recently
        let recency_factor = match gap.priority {
            9..=10 => 1.0, // Very recent
            7..=8 => 0.8,  // Recent
            5..=6 => 0.6,  // Somewhat recent
            3..=4 => 0.4,  // Older
            _ => 0.2,      // Old
        };

        // Apply development phase urgency
        let phase_multiplier = self.context.phase.urgency_multiplier();

        (recency_factor * 10.0 * phase_multiplier).min(10.0)
    }

    /// Calculate impact score based on gap characteristics and context (0.0-10.0)
    async fn calculate_impact_score(
        &self,
        gap: &DetectedGap,
    ) -> Result<(f64, Option<ImpactAssessmentResult>), PrioritizationError> {
        // Use impact assessor if available for sophisticated analysis
        if let Some(impact_assessor) = &self.impact_assessor {
            match impact_assessor.assess_gap_impact(gap).await {
                Ok(assessment) => {
                    return Ok((assessment.final_impact_score, Some(assessment)));
                }
                Err(e) => {
                    warn!(
                        "Impact assessment failed: {}, falling back to basic calculation",
                        e
                    );
                    // Fall through to basic calculation
                }
            }
        }

        // Basic impact calculation (fallback or when impact assessor is disabled)
        let mut impact_score: f64 = 5.0; // Base impact

        // API visibility impact
        if self.context.is_public_api {
            match gap.gap_type {
                GapType::ApiDocumentationGap => impact_score += 3.0,
                GapType::MissingDocumentation => impact_score += 2.0,
                _ => impact_score += 1.0,
            }
        }

        // Team size impact
        let team_impact_multiplier = match self.context.team_size {
            1 => 0.7,      // Solo developer
            2..=5 => 1.0,  // Small team
            6..=15 => 1.3, // Medium team
            _ => 1.5,      // Large team
        };
        impact_score *= team_impact_multiplier;

        // Performance criticality impact
        if self.context.performance_critical {
            match gap.gap_type {
                GapType::UndocumentedTechnology => impact_score += 2.0,
                GapType::TodoComment => impact_score += 1.5,
                _ => impact_score += 1.0,
            }
        }

        // Deadline urgency impact
        if self.context.has_urgent_deadlines {
            impact_score *= 1.5;
        }

        Ok((impact_score.min(10.0), None))
    }

    /// Calculate context factors score (0.0-10.0)
    fn calculate_context_score(&self, gap: &DetectedGap) -> f64 {
        let mut context_score: f64 = 5.0; // Base context score

        // Development phase affects context importance
        context_score *= self.context.phase.urgency_multiplier();

        // File path context - core files are more important
        let file_path_str = gap.file_path.to_string_lossy().to_lowercase();
        if file_path_str.contains("src/lib.rs") || file_path_str.contains("main.rs") {
            context_score += 2.0;
        } else if file_path_str.contains("src/") {
            context_score += 1.0;
        } else if file_path_str.contains("tests/") {
            context_score += 0.5;
        }

        // Gap metadata context
        if let Some(item_type) = gap.metadata.get("item_type") {
            match item_type.as_str() {
                "function" => context_score += 1.0,
                "struct" => context_score += 1.5,
                "module" => context_score += 2.0,
                _ => {}
            }
        }

        context_score.min(10.0)
    }

    /// Convert numerical score to priority level
    fn score_to_priority_level(&self, score: f64) -> TaskPriority {
        match score {
            s if s >= 8.5 => TaskPriority::Critical,
            s if s >= 7.0 => TaskPriority::High,
            s if s >= 4.0 => TaskPriority::Medium,
            _ => TaskPriority::Low,
        }
    }

    /// Calculate confidence in the priority assessment
    fn calculate_confidence(&self, gap: &DetectedGap, final_score: f64) -> f64 {
        let mut confidence = gap.confidence; // Start with gap detection confidence

        // Adjust based on available metadata
        if gap.metadata.is_empty() {
            confidence *= 0.8; // Lower confidence with less context
        }

        // Adjust based on score extremes (very high or very low scores may be less reliable)
        if !(2.0..=9.0).contains(&final_score) {
            confidence *= 0.9;
        }

        // Development phase affects confidence
        confidence *= match self.context.phase {
            DevelopmentPhase::Production => 1.0, // High confidence in production
            DevelopmentPhase::PreProduction => 0.95,
            DevelopmentPhase::Testing => 0.9,
            DevelopmentPhase::Development => 0.85,
            DevelopmentPhase::Prototyping => 0.8, // Lower confidence in prototyping
        };

        confidence.clamp(0.0, 1.0)
    }

    /// Generate cache key for gap
    fn generate_cache_key(&self, gap: &DetectedGap) -> String {
        format!(
            "{}:{}:{}:{}",
            gap.file_path.to_string_lossy(),
            gap.line_number,
            gap.gap_type,
            gap.confidence
        )
    }

    /// Get cached score if available and not expired
    async fn get_cached_score(&self, cache_key: &str) -> Option<CachedScore> {
        let cache = self.score_cache.read().await;
        if let Some(cached) = cache.get(cache_key) {
            if Utc::now() < cached.expires_at {
                return Some(cached.clone());
            }
        }
        None
    }

    /// Cache priority score
    async fn cache_score(&self, cache_key: String, breakdown: PriorityScoreBreakdown) {
        let expires_at =
            Utc::now() + ChronoDuration::seconds(self.config.score_cache_ttl_secs as i64);
        let cached = CachedScore {
            breakdown,
            expires_at,
        };

        let mut cache = self.score_cache.write().await;
        cache.insert(cache_key, cached);
    }

    /// Update scoring metrics
    async fn update_metrics(&self, duration: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.total_scores_calculated += 1;

        // Update rolling average
        let total = metrics.total_scores_calculated;
        let previous_avg = metrics.average_scoring_time.as_nanos() as f64;
        let new_avg =
            (previous_avg * (total - 1) as f64 + duration.as_nanos() as f64) / total as f64;
        metrics.average_scoring_time = Duration::from_nanos(new_avg as u64);

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

    /// Increment batch operation counter
    async fn increment_batch_operation(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.batch_operations += 1;
    }

    /// Increment performance violation counter
    async fn increment_performance_violation(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.performance_violations += 1;
    }

    /// Get current prioritization metrics
    pub async fn get_metrics(&self) -> PrioritizationMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Clear priority score cache
    pub async fn clear_cache(&self) {
        let mut cache = self.score_cache.write().await;
        cache.clear();
        info!("Priority score cache cleared");
    }

    /// Update development context
    pub async fn update_context(&mut self, context: DevelopmentContext) {
        self.context = context;
        // Clear cache since context change affects scoring
        self.clear_cache().await;
        info!("Development context updated, cache cleared");
    }

    /// Get current configuration
    pub fn get_config(&self) -> &PrioritizationConfig {
        &self.config
    }

    /// Get current development context
    pub fn get_context(&self) -> &DevelopmentContext {
        &self.context
    }

    /// Get context-aware scorer configuration if available
    pub fn get_context_aware_config(&self) -> Option<&ContextAwareScoringConfig> {
        self.context_aware_scorer
            .as_ref()
            .map(|scorer| scorer.get_config())
    }

    /// Check if impact assessment is enabled and available
    pub fn is_impact_assessment_enabled(&self) -> bool {
        self.impact_assessor.is_some()
    }

    /// Get impact assessor configuration if available
    pub fn get_impact_assessor_config(&self) -> Option<&ImpactAssessmentConfig> {
        self.impact_assessor
            .as_ref()
            .map(|assessor| assessor.get_config())
    }
}

// Helper trait implementations for cleaner string conversion
impl std::fmt::Display for GapType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GapType::TodoComment => write!(f, "todo_comment"),
            GapType::MissingDocumentation => write!(f, "missing_documentation"),
            GapType::UndocumentedTechnology => write!(f, "undocumented_technology"),
            GapType::ApiDocumentationGap => write!(f, "api_documentation_gap"),
            GapType::ConfigurationGap => write!(f, "configuration_gap"),
        }
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskPriority::Low => write!(f, "low"),
            TaskPriority::Medium => write!(f, "medium"),
            TaskPriority::High => write!(f, "high"),
            TaskPriority::Critical => write!(f, "critical"),
        }
    }
}

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
    async fn test_priority_scorer_creation() {
        // FAILING TEST: Priority scorer should be creatable with valid configuration
        let config = PrioritizationConfig::default();
        let context = DevelopmentContext::default();
        let result = PriorityScorer::new(config, context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_priority_scorer_with_defaults() {
        // FAILING TEST: Default priority scorer creation should work
        let result = PriorityScorer::with_defaults().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_gap_type_urgency_calculation() {
        // FAILING TEST: Gap type urgency should be calculated correctly for all types
        let scorer = PriorityScorer::with_defaults().await.unwrap();

        let mut gap = create_test_gap();

        // Test TodoComment urgency
        gap.gap_type = GapType::TodoComment;
        let score = scorer.calculate_gap_type_urgency(&gap);
        assert!((6.0..=8.0).contains(&score)); // Should be high priority

        // Test ApiDocumentationGap urgency
        gap.gap_type = GapType::ApiDocumentationGap;
        let score = scorer.calculate_gap_type_urgency(&gap);
        assert!(score >= 8.0); // Should be highest priority

        // Test UndocumentedTechnology urgency
        gap.gap_type = GapType::UndocumentedTechnology;
        let score = scorer.calculate_gap_type_urgency(&gap);
        assert!((7.0..=9.0).contains(&score)); // Should be high priority
    }

    #[tokio::test]
    async fn test_single_gap_priority_scoring() {
        // FAILING TEST: Single gap priority scoring should work correctly
        let scorer = PriorityScorer::with_defaults().await.unwrap();
        let gap = create_test_gap();

        let result = scorer.score_gap_priority(&gap).await;
        assert!(result.is_ok());

        let breakdown = result.unwrap();
        assert!(breakdown.final_score >= 0.0 && breakdown.final_score <= 10.0);
        assert!(breakdown.confidence >= 0.0 && breakdown.confidence <= 1.0);
        assert!(breakdown.gap_type_score >= 0.0 && breakdown.gap_type_score <= 10.0);
        assert!(breakdown.recency_score >= 0.0 && breakdown.recency_score <= 10.0);
        assert!(breakdown.impact_score >= 0.0 && breakdown.impact_score <= 10.0);
        assert!(breakdown.context_score >= 0.0 && breakdown.context_score <= 10.0);
    }

    #[tokio::test]
    async fn test_batch_gap_priority_scoring() {
        // FAILING TEST: Batch gap priority scoring should handle multiple gaps efficiently
        let scorer = PriorityScorer::with_defaults().await.unwrap();

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
        let result = scorer.score_gaps_batch(&gaps).await;
        let duration = start_time.elapsed();

        assert!(result.is_ok());
        let breakdowns = result.unwrap();
        assert_eq!(breakdowns.len(), gaps.len());

        // Check performance requirement: <100ms for up to 50 gaps
        assert!(duration < Duration::from_millis(100));

        // All scores should be valid
        for breakdown in &breakdowns {
            assert!(breakdown.final_score >= 0.0 && breakdown.final_score <= 10.0);
            assert!(breakdown.confidence >= 0.0 && breakdown.confidence <= 1.0);
        }
    }

    #[tokio::test]
    async fn test_development_context_impact() {
        // FAILING TEST: Development context should affect priority scoring
        let context = DevelopmentContext {
            phase: DevelopmentPhase::Production,
            is_public_api: true,
            has_urgent_deadlines: true,
            ..Default::default()
        };

        let config = PrioritizationConfig::default();
        let scorer = PriorityScorer::new(config, context).await.unwrap();

        let mut gap = create_test_gap();
        gap.gap_type = GapType::ApiDocumentationGap;

        let breakdown = scorer.score_gap_priority(&gap).await.unwrap();

        // Production + public API + urgent deadlines should result in high priority
        assert!(breakdown.final_score >= 7.0);
        // Allow for either High or Critical priority given the strong context
        assert!(matches!(
            breakdown.priority_level,
            TaskPriority::High | TaskPriority::Critical
        ));
    }

    #[tokio::test]
    async fn test_recency_scoring() {
        // FAILING TEST: Recency scoring should favor recently discovered gaps
        let scorer = PriorityScorer::with_defaults().await.unwrap();

        let mut high_priority_gap = create_test_gap();
        high_priority_gap.priority = 9; // Recent

        let mut low_priority_gap = create_test_gap();
        low_priority_gap.priority = 3; // Older

        let high_recency = scorer.calculate_recency_score(&high_priority_gap);
        let low_recency = scorer.calculate_recency_score(&low_priority_gap);

        assert!(high_recency > low_recency);
        assert!(high_recency >= 6.0); // Should be reasonably high for recent gaps
        assert!(low_recency <= 5.0); // Should be lower for older gaps
    }

    #[tokio::test]
    async fn test_priority_level_assignment() {
        // FAILING TEST: Score to priority level conversion should work correctly
        let scorer = PriorityScorer::with_defaults().await.unwrap();

        assert_eq!(scorer.score_to_priority_level(9.5), TaskPriority::Critical);
        assert_eq!(scorer.score_to_priority_level(8.0), TaskPriority::High);
        assert_eq!(scorer.score_to_priority_level(5.5), TaskPriority::Medium);
        assert_eq!(scorer.score_to_priority_level(2.0), TaskPriority::Low);
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        // FAILING TEST: Configuration validation should catch invalid weights
        let config = PrioritizationConfig {
            gap_type_weight: 0.5,
            recency_weight: 0.5,
            impact_weight: 0.3, // Sum > 1.0
            context_weight: 0.3,
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());

        if let Err(PrioritizationError::WeightValidation { sum }) = result {
            assert!(sum > 1.0);
        } else {
            panic!("Expected WeightValidation error");
        }
    }

    #[tokio::test]
    async fn test_performance_requirements() {
        // FAILING TEST: Priority scoring should meet performance requirements
        let config = PrioritizationConfig::for_performance();
        let context = DevelopmentContext::default();
        let scorer = PriorityScorer::new(config, context).await.unwrap();

        // Test single gap scoring performance
        let gap = create_test_gap();
        let start_time = Instant::now();
        let _result = scorer.score_gap_priority(&gap).await.unwrap();
        let duration = start_time.elapsed();

        assert!(duration < Duration::from_millis(50)); // Performance config limit

        // Test batch scoring performance with maximum batch size
        let gaps: Vec<_> = (0..25)
            .map(|i| {
                let mut gap = create_test_gap();
                gap.line_number = i;
                gap
            })
            .collect();

        let start_time = Instant::now();
        let result = scorer.score_gaps_batch(&gaps).await;
        let duration = start_time.elapsed();

        assert!(result.is_ok());
        assert!(duration < Duration::from_millis(100)); // Original requirement
    }

    #[tokio::test]
    async fn test_score_caching() {
        // FAILING TEST: Score caching should improve performance for repeated scoring
        let config = PrioritizationConfig {
            enable_score_caching: true,
            score_cache_ttl_secs: 60,
            ..Default::default()
        };

        let context = DevelopmentContext::default();
        let scorer = PriorityScorer::new(config, context).await.unwrap();

        let gap = create_test_gap();

        // First scoring should be cache miss
        let _result1 = scorer.score_gap_priority(&gap).await.unwrap();
        let metrics1 = scorer.get_metrics().await;
        assert!(metrics1.cache_misses > 0);

        // Second scoring should be cache hit
        let _result2 = scorer.score_gap_priority(&gap).await.unwrap();
        let metrics2 = scorer.get_metrics().await;
        assert!(metrics2.cache_hits > 0);
        assert_eq!(metrics2.cache_hits, 1);
    }

    #[tokio::test]
    async fn test_custom_priority_boosts() {
        // FAILING TEST: Custom priority boosts should affect gap type urgency
        let mut context = DevelopmentContext::default();
        context.custom_boosts.insert(GapType::TodoComment, 1.5); // 50% boost

        let config = PrioritizationConfig::default();
        let scorer = PriorityScorer::new(config, context).await.unwrap();

        let gap = create_test_gap(); // TodoComment
        let score_with_boost = scorer.calculate_gap_type_urgency(&gap);

        // Create scorer without boost
        let scorer_no_boost = PriorityScorer::with_defaults().await.unwrap();
        let score_without_boost = scorer_no_boost.calculate_gap_type_urgency(&gap);

        assert!(score_with_boost > score_without_boost);
        assert!((score_with_boost / score_without_boost - 1.5).abs() < 0.1); // Approximately 50% boost
    }

    #[tokio::test]
    async fn test_metrics_tracking() {
        // FAILING TEST: Metrics should track scoring operations correctly
        let scorer = PriorityScorer::with_defaults().await.unwrap();
        let gap = create_test_gap();

        let initial_metrics = scorer.get_metrics().await;
        assert_eq!(initial_metrics.total_scores_calculated, 0);

        // Score a gap
        let _result = scorer.score_gap_priority(&gap).await.unwrap();

        let updated_metrics = scorer.get_metrics().await;
        assert_eq!(updated_metrics.total_scores_calculated, 1);
        assert!(updated_metrics.average_scoring_time > Duration::from_nanos(0));
    }

    #[tokio::test]
    async fn test_context_aware_configuration() {
        // FAILING TEST: Context-aware configuration should work correctly
        let config = PrioritizationConfig::with_context_aware_scoring();
        assert!(config.enable_context_aware_scoring);
        assert!(config.context_aware_config.is_some());
        assert_eq!(config.max_scoring_time_ms, 150); // Adjusted for context-aware overhead

        let context = DevelopmentContext::default();
        let scorer = PriorityScorer::new(config, context).await.unwrap();
        assert!(scorer.is_context_aware_enabled());
        assert!(scorer.get_context_aware_config().is_some());
    }

    #[tokio::test]
    async fn test_context_aware_disabled_by_default() {
        // FAILING TEST: Context-aware scoring should be disabled by default
        let scorer = PriorityScorer::with_defaults().await.unwrap();
        assert!(!scorer.is_context_aware_enabled());
        assert!(scorer.get_context_aware_config().is_none());
    }

    #[tokio::test]
    async fn test_enhanced_priority_scoring_without_context_aware() {
        // FAILING TEST: Enhanced scoring should work even when context-aware is disabled
        let scorer = PriorityScorer::with_defaults().await.unwrap();
        let gap = create_test_gap();

        let result = scorer.score_gap_priority_enhanced(&gap).await;
        assert!(result.is_ok());

        let breakdown = result.unwrap();
        assert!(!breakdown.classification_available);
        assert!(!breakdown.used_graceful_degradation);
        assert_eq!(breakdown.domain_adjustment, 1.0);
        assert_eq!(breakdown.audience_adjustment, 1.0);
        assert_eq!(breakdown.urgency_adjustment, 1.0);
        assert_eq!(breakdown.confidence_weighting, 0.0);
        assert!(breakdown.extracted_context.is_none());

        // Enhanced score should equal base score when no context-aware scoring
        assert_eq!(
            breakdown.context_enhanced_score,
            breakdown.base_breakdown.final_score
        );
    }

    #[tokio::test]
    async fn test_enhanced_priority_scoring_with_context_aware() {
        // FAILING TEST: Enhanced scoring should use context-aware when enabled
        let config = PrioritizationConfig::with_context_aware_scoring();
        let context = DevelopmentContext::default();
        let scorer = PriorityScorer::new(config, context).await.unwrap();

        let gap = create_test_gap();
        let result = scorer.score_gap_priority_enhanced(&gap).await;

        // Should succeed but may use graceful degradation
        assert!(result.is_ok());

        let breakdown = result.unwrap();
        assert!(breakdown.context_enhanced_score >= 0.0);
        assert!(breakdown.context_enhanced_score <= 10.0);
        // Processing time should be minimal
        assert!(breakdown.context_processing_time < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_enhanced_batch_scoring() {
        // FAILING TEST: Enhanced batch scoring should work correctly
        let scorer = PriorityScorer::with_defaults().await.unwrap();

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
        let result = scorer.score_gaps_batch_enhanced(&gaps).await;
        let duration = start_time.elapsed();

        assert!(result.is_ok());
        let breakdowns = result.unwrap();
        assert_eq!(breakdowns.len(), gaps.len());

        // Check performance requirement
        assert!(duration < Duration::from_millis(200)); // Reasonable limit for enhanced scoring

        // All enhanced scores should be valid
        for breakdown in &breakdowns {
            assert!(breakdown.context_enhanced_score >= 0.0);
            assert!(breakdown.context_enhanced_score <= 10.0);
            assert!(breakdown.base_breakdown.final_score >= 0.0);
            assert!(breakdown.base_breakdown.final_score <= 10.0);
        }
    }

    #[tokio::test]
    async fn test_custom_context_aware_configuration() {
        // FAILING TEST: Custom context-aware configuration should be applied correctly
        use crate::proactive::context_aware_scorer::ContextAwareScoringConfig;

        let mut context_config = ContextAwareScoringConfig::default();
        context_config.domain_weights.rust_multiplier = 1.5;
        context_config.max_context_overhead_ms = 30;

        let config = PrioritizationConfig::with_custom_context_aware(context_config.clone());
        assert!(config.enable_context_aware_scoring);
        assert!(config.context_aware_config.is_some());
        assert_eq!(
            config
                .context_aware_config
                .as_ref()
                .unwrap()
                .max_context_overhead_ms,
            30
        );

        let context = DevelopmentContext::default();
        let scorer = PriorityScorer::new(config, context).await.unwrap();
        assert!(scorer.is_context_aware_enabled());

        let scorer_config = scorer.get_context_aware_config().unwrap();
        assert_eq!(scorer_config.domain_weights.rust_multiplier, 1.5);
        assert_eq!(scorer_config.max_context_overhead_ms, 30);
    }

    #[tokio::test]
    async fn test_impact_assessment_disabled_by_default() {
        // FAILING TEST: Impact assessment should be disabled by default
        let scorer = PriorityScorer::with_defaults().await.unwrap();
        assert!(!scorer.is_impact_assessment_enabled());
        assert!(scorer.get_impact_assessor_config().is_none());
    }

    #[tokio::test]
    async fn test_impact_assessment_configuration() {
        // FAILING TEST: Impact assessment configuration should work correctly
        let config = PrioritizationConfig::with_impact_assessment();
        assert!(config.enable_impact_assessment);
        assert!(config.impact_assessment_config.is_some());
        assert_eq!(config.max_scoring_time_ms, 200); // Adjusted for impact assessment overhead

        let context = DevelopmentContext::default();
        let scorer = PriorityScorer::new(config, context).await.unwrap();
        assert!(scorer.is_impact_assessment_enabled());
        assert!(scorer.get_impact_assessor_config().is_some());
    }

    #[tokio::test]
    async fn test_custom_impact_assessment_configuration() {
        // FAILING TEST: Custom impact assessment configuration should be applied correctly
        use crate::proactive::impact_assessor::ImpactAssessmentConfig;

        let impact_config = ImpactAssessmentConfig {
            usage_frequency_weight: 0.4,
            dependency_impact_weight: 0.3,
            api_visibility_weight: 0.2,
            development_activity_weight: 0.07,
            team_impact_weight: 0.03,
            max_analysis_time_ms: 150,
            ..Default::default()
        };

        let config = PrioritizationConfig::with_custom_impact_assessment(impact_config.clone());
        assert!(config.enable_impact_assessment);
        assert!(config.impact_assessment_config.is_some());
        assert_eq!(
            config
                .impact_assessment_config
                .as_ref()
                .unwrap()
                .max_analysis_time_ms,
            150
        );

        let context = DevelopmentContext::default();
        let scorer = PriorityScorer::new(config, context).await.unwrap();
        assert!(scorer.is_impact_assessment_enabled());

        let scorer_config = scorer.get_impact_assessor_config().unwrap();
        assert_eq!(scorer_config.usage_frequency_weight, 0.4);
        assert_eq!(scorer_config.dependency_impact_weight, 0.3);
        assert_eq!(scorer_config.max_analysis_time_ms, 150);
    }

    #[tokio::test]
    async fn test_full_enhancement_configuration() {
        // FAILING TEST: Full enhancement configuration should enable both features
        let config = PrioritizationConfig::with_full_enhancement();
        assert!(config.enable_context_aware_scoring);
        assert!(config.enable_impact_assessment);
        assert!(config.context_aware_config.is_some());
        assert!(config.impact_assessment_config.is_some());
        assert_eq!(config.max_scoring_time_ms, 250); // Higher for both enhancements

        let context = DevelopmentContext::default();
        let scorer = PriorityScorer::new(config, context).await.unwrap();
        assert!(scorer.is_context_aware_enabled());
        assert!(scorer.is_impact_assessment_enabled());
    }

    #[tokio::test]
    async fn test_priority_scoring_with_impact_assessment() {
        // FAILING TEST: Priority scoring should work with impact assessment enabled
        let config = PrioritizationConfig::with_impact_assessment();
        let context = DevelopmentContext::default();
        let scorer = PriorityScorer::new(config, context).await.unwrap();

        let gap = create_test_gap();
        let result = scorer.score_gap_priority(&gap).await;
        assert!(result.is_ok());

        let breakdown = result.unwrap();
        assert!(breakdown.final_score >= 0.0 && breakdown.final_score <= 10.0);
        assert!(breakdown.impact_assessment.is_some());

        let impact_assessment = breakdown.impact_assessment.unwrap();
        assert!(impact_assessment.final_impact_score >= 0.0);
        assert!(impact_assessment.overall_confidence >= 0.0);
        assert!(!impact_assessment.primary_impact_factor().is_empty());
    }

    #[tokio::test]
    async fn test_impact_assessment_fallback() {
        // FAILING TEST: Priority scoring should fallback gracefully when impact assessment fails
        let config = PrioritizationConfig::with_impact_assessment();
        let context = DevelopmentContext::default();
        let scorer = PriorityScorer::new(config, context).await.unwrap();

        // Create a gap that might cause impact assessment issues
        let mut gap = create_test_gap();
        gap.file_path = PathBuf::from(""); // Empty path might cause issues

        let result = scorer.score_gap_priority(&gap).await;
        assert!(result.is_ok()); // Should still work with fallback

        let breakdown = result.unwrap();
        assert!(breakdown.final_score >= 0.0 && breakdown.final_score <= 10.0);
        // Impact assessment might be None due to fallback
    }

    #[tokio::test]
    async fn test_impact_assessment_batch_scoring() {
        // FAILING TEST: Batch scoring should work with impact assessment
        let config = PrioritizationConfig::with_impact_assessment();
        let context = DevelopmentContext::default();
        let scorer = PriorityScorer::new(config, context).await.unwrap();

        let mut gaps = Vec::new();
        for i in 0..5 {
            let mut gap = create_test_gap();
            gap.line_number = 10 + i;
            gap.gap_type = match i % 3 {
                0 => GapType::TodoComment,
                1 => GapType::ApiDocumentationGap,
                _ => GapType::UndocumentedTechnology,
            };
            gaps.push(gap);
        }

        let start_time = Instant::now();
        let result = scorer.score_gaps_batch(&gaps).await;
        let duration = start_time.elapsed();

        assert!(result.is_ok());
        let breakdowns = result.unwrap();
        assert_eq!(breakdowns.len(), gaps.len());

        // Check performance with impact assessment
        assert!(duration < Duration::from_millis(1000)); // Should still be reasonable

        // Check that some gaps have impact assessments
        let with_impact = breakdowns
            .iter()
            .filter(|b| b.impact_assessment.is_some())
            .count();
        assert!(with_impact > 0); // At least some should have impact assessment
    }
}
