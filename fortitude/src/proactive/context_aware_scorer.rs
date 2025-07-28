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

// ABOUTME: Context-aware priority scoring using classification system for intelligent research prioritization
//! This module provides context-aware priority scoring that leverages the existing classification system
//! to provide intelligent priority adjustments based on research context, domain, and classification metadata.
//!
//! The system enhances the basic prioritization algorithms from Task 3.1 with classification-based context
//! understanding, domain-aware scoring, adaptive weighting, and multi-domain support for different research contexts.
//!
//! Performance Requirements:
//! - Context-aware scoring should add <20ms overhead to existing prioritization
//! - Classification integration should not degrade overall scoring performance
//! - Batch processing support for multiple gaps with classification context
//! - Memory-efficient context extraction and caching
//! - Graceful degradation when classification is unavailable

use crate::proactive::{DetectedGap, GapType, PriorityScoreBreakdown};
use chrono::{DateTime, Utc};
use fortitude_core::classification::{AdvancedClassificationConfig, AdvancedClassifier};
use fortitude_types::{
    classification_result::{
        AudienceLevel, ClassificationDimension, EnhancedClassificationResult, TechnicalDomain,
        UrgencyLevel,
    },
    research::ResearchType,
    ClassificationError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

/// Errors that can occur during context-aware priority scoring
#[derive(Error, Debug)]
pub enum ContextAwareScoringError {
    #[error("Classification error: {0}")]
    Classification(#[from] ClassificationError),

    #[error("Prioritization error: {0}")]
    Prioritization(String),

    #[error("Context extraction failed: {0}")]
    ContextExtraction(String),

    #[error("Domain mapping failed: {0}")]
    DomainMapping(String),

    #[error("Confidence weighting failed: confidence {confidence} outside valid range")]
    ConfidenceWeighting { confidence: f64 },

    #[error("Performance threshold exceeded: context-aware scoring took {duration:?}, limit is {limit:?}")]
    PerformanceThreshold { duration: Duration, limit: Duration },

    #[error("Classification unavailable: {reason}")]
    ClassificationUnavailable { reason: String },
}

/// Context extracted from classification results for priority scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedContext {
    /// Research type detected from classification
    pub research_type: ResearchType,
    /// Audience level (Beginner, Intermediate, Advanced)
    pub audience_level: AudienceLevel,
    /// Technical domain (Rust, Web, DevOps, etc.)
    pub technical_domain: TechnicalDomain,
    /// Urgency level (Immediate, Planned, Exploratory)
    pub urgency_level: UrgencyLevel,
    /// Overall classification confidence
    pub classification_confidence: f64,
    /// Confidence for each dimension
    pub dimension_confidences: HashMap<ClassificationDimension, f64>,
    /// Keywords that influenced classification
    pub classification_keywords: Vec<String>,
    /// Context extraction timestamp
    pub extracted_at: DateTime<Utc>,
}

impl ExtractedContext {
    /// Get confidence for a specific dimension
    pub fn get_dimension_confidence(&self, dimension: &ClassificationDimension) -> f64 {
        self.dimension_confidences
            .get(dimension)
            .copied()
            .unwrap_or(0.0)
    }

    /// Check if context has high confidence for any dimension
    pub fn has_high_confidence(&self, threshold: f64) -> bool {
        self.dimension_confidences
            .values()
            .any(|&conf| conf >= threshold)
    }

    /// Get the most confident dimension
    pub fn most_confident_dimension(&self) -> Option<(&ClassificationDimension, f64)> {
        self.dimension_confidences
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(dim, &conf)| (dim, conf))
    }
}

/// Domain-specific priority weights for different technical domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainPriorityWeights {
    /// Priority multiplier for Rust-specific research
    pub rust_multiplier: f64,
    /// Priority multiplier for Web development research
    pub web_multiplier: f64,
    /// Priority multiplier for DevOps research
    pub devops_multiplier: f64,
    /// Priority multiplier for AI/ML research
    pub ai_multiplier: f64,
    /// Priority multiplier for Database research
    pub database_multiplier: f64,
    /// Priority multiplier for Systems research
    pub systems_multiplier: f64,
    /// Priority multiplier for Security research
    pub security_multiplier: f64,
    /// Priority multiplier for General research
    pub general_multiplier: f64,
}

impl Default for DomainPriorityWeights {
    fn default() -> Self {
        Self {
            rust_multiplier: 1.2,     // Boost Rust research
            web_multiplier: 1.1,      // Slight boost for web
            devops_multiplier: 1.0,   // Neutral for DevOps
            ai_multiplier: 1.3,       // High boost for AI/ML
            database_multiplier: 1.0, // Neutral for database
            systems_multiplier: 1.1,  // Slight boost for systems
            security_multiplier: 1.4, // High boost for security
            general_multiplier: 0.9,  // Slight reduction for general
        }
    }
}

impl DomainPriorityWeights {
    /// Get priority multiplier for a technical domain
    pub fn get_multiplier(&self, domain: &TechnicalDomain) -> f64 {
        match domain {
            TechnicalDomain::Rust => self.rust_multiplier,
            TechnicalDomain::Python => self.rust_multiplier, // Use same multiplier as Rust for now
            TechnicalDomain::Web => self.web_multiplier,
            TechnicalDomain::DevOps => self.devops_multiplier,
            TechnicalDomain::AI => self.ai_multiplier,
            TechnicalDomain::Database => self.database_multiplier,
            TechnicalDomain::Systems => self.systems_multiplier,
            TechnicalDomain::Security => self.security_multiplier,
            TechnicalDomain::Architecture => self.systems_multiplier, // Use same multiplier as Systems for now
            TechnicalDomain::General => self.general_multiplier,
        }
    }
}

/// Audience-aware priority adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudiencePriorityAdjustments {
    /// Priority boost for beginner-focused research (more explanatory content needed)
    pub beginner_boost: f64,
    /// Priority adjustment for intermediate research
    pub intermediate_adjustment: f64,
    /// Priority adjustment for advanced research
    pub advanced_adjustment: f64,
}

impl Default for AudiencePriorityAdjustments {
    fn default() -> Self {
        Self {
            beginner_boost: 1.2,          // Higher priority for beginner content
            intermediate_adjustment: 1.0, // Neutral for intermediate
            advanced_adjustment: 0.9,     // Lower priority for advanced (assume existing knowledge)
        }
    }
}

impl AudiencePriorityAdjustments {
    /// Get priority adjustment for an audience level
    pub fn get_adjustment(&self, audience: &AudienceLevel) -> f64 {
        match audience {
            AudienceLevel::Beginner => self.beginner_boost,
            AudienceLevel::Intermediate => self.intermediate_adjustment,
            AudienceLevel::Advanced => self.advanced_adjustment,
        }
    }
}

/// Urgency-based priority scaling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrgencyPriorityScaling {
    /// Priority multiplier for immediate urgency
    pub immediate_multiplier: f64,
    /// Priority multiplier for planned urgency
    pub planned_multiplier: f64,
    /// Priority multiplier for exploratory urgency
    pub exploratory_multiplier: f64,
}

impl Default for UrgencyPriorityScaling {
    fn default() -> Self {
        Self {
            immediate_multiplier: 2.0,   // Double priority for immediate needs
            planned_multiplier: 1.0,     // Baseline for planned research
            exploratory_multiplier: 0.7, // Lower priority for exploratory
        }
    }
}

impl UrgencyPriorityScaling {
    /// Get priority multiplier for an urgency level
    pub fn get_multiplier(&self, urgency: &UrgencyLevel) -> f64 {
        match urgency {
            UrgencyLevel::Immediate => self.immediate_multiplier,
            UrgencyLevel::Planned => self.planned_multiplier,
            UrgencyLevel::Exploratory => self.exploratory_multiplier,
        }
    }
}

/// Configuration for context-aware priority scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAwareScoringConfig {
    /// Domain-specific priority weights
    pub domain_weights: DomainPriorityWeights,
    /// Audience-aware priority adjustments
    pub audience_adjustments: AudiencePriorityAdjustments,
    /// Urgency-based priority scaling
    pub urgency_scaling: UrgencyPriorityScaling,
    /// Minimum classification confidence threshold
    pub min_classification_confidence: f64,
    /// Weight for classification confidence in final scoring
    pub confidence_weight: f64,
    /// Maximum additional overhead time for context-aware scoring (ms)
    pub max_context_overhead_ms: u64,
    /// Enable graceful degradation when classification fails
    pub enable_graceful_degradation: bool,
    /// Cache classification results for performance
    pub enable_classification_cache: bool,
    /// Classification cache TTL in seconds
    pub classification_cache_ttl_secs: u64,
}

impl Default for ContextAwareScoringConfig {
    fn default() -> Self {
        Self {
            domain_weights: DomainPriorityWeights::default(),
            audience_adjustments: AudiencePriorityAdjustments::default(),
            urgency_scaling: UrgencyPriorityScaling::default(),
            min_classification_confidence: 0.3,
            confidence_weight: 0.2,
            max_context_overhead_ms: 20,
            enable_graceful_degradation: true,
            enable_classification_cache: true,
            classification_cache_ttl_secs: 300, // 5 minutes
        }
    }
}

impl ContextAwareScoringConfig {
    /// Validate configuration parameters
    pub fn validate(&self) -> Result<(), ContextAwareScoringError> {
        if !(0.0..=1.0).contains(&self.min_classification_confidence) {
            return Err(ContextAwareScoringError::ContextExtraction(format!(
                "min_classification_confidence must be between 0.0 and 1.0, got {}",
                self.min_classification_confidence
            )));
        }

        if !(0.0..=1.0).contains(&self.confidence_weight) {
            return Err(ContextAwareScoringError::ContextExtraction(format!(
                "confidence_weight must be between 0.0 and 1.0, got {}",
                self.confidence_weight
            )));
        }

        if self.max_context_overhead_ms == 0 {
            return Err(ContextAwareScoringError::ContextExtraction(
                "max_context_overhead_ms must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

/// Context-aware priority scoring breakdown with classification details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAwarePriorityBreakdown {
    /// Original priority breakdown from base prioritization
    pub base_breakdown: PriorityScoreBreakdown,
    /// Context extracted from classification
    pub extracted_context: Option<ExtractedContext>,
    /// Domain-aware score adjustment
    pub domain_adjustment: f64,
    /// Audience-aware score adjustment
    pub audience_adjustment: f64,
    /// Urgency-based score adjustment
    pub urgency_adjustment: f64,
    /// Classification confidence weighting
    pub confidence_weighting: f64,
    /// Final context-enhanced score
    pub context_enhanced_score: f64,
    /// Time taken for context extraction and scoring
    pub context_processing_time: Duration,
    /// Whether classification was available
    pub classification_available: bool,
    /// Whether graceful degradation was used
    pub used_graceful_degradation: bool,
}

/// Cached classification result for performance
#[derive(Debug, Clone)]
struct CachedClassification {
    result: EnhancedClassificationResult,
    expires_at: DateTime<Utc>,
}

/// Context-aware priority scorer that enhances basic prioritization with classification
pub struct ContextAwarePriorityScorer {
    /// Configuration for context-aware scoring
    config: ContextAwareScoringConfig,
    /// Advanced classifier for context extraction
    classifier: Arc<AdvancedClassifier>,
    /// Cache for classification results
    classification_cache: Arc<RwLock<HashMap<String, CachedClassification>>>,
}

impl ContextAwarePriorityScorer {
    /// Create a new context-aware priority scorer
    #[instrument(level = "debug", skip(config))]
    pub async fn new(config: ContextAwareScoringConfig) -> Result<Self, ContextAwareScoringError> {
        config.validate()?;

        info!("Initializing context-aware priority scorer with classification integration");

        // Create advanced classifier with research-focused configuration
        let classifier_config = AdvancedClassificationConfig {
            max_processing_time_ms: config.max_context_overhead_ms,
            enable_graceful_degradation: config.enable_graceful_degradation,
            ..Default::default()
        };

        let classifier = AdvancedClassifier::new(classifier_config);

        Ok(Self {
            config,
            classifier: Arc::new(classifier),
            classification_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create context-aware scorer with default configuration
    pub async fn with_defaults() -> Result<Self, ContextAwareScoringError> {
        Self::new(ContextAwareScoringConfig::default()).await
    }

    /// Enhance priority scoring with context-aware adjustments
    #[instrument(level = "debug", skip(self, base_breakdown, gap))]
    pub async fn enhance_priority_with_context(
        &self,
        base_breakdown: PriorityScoreBreakdown,
        gap: &DetectedGap,
    ) -> Result<ContextAwarePriorityBreakdown, ContextAwareScoringError> {
        let start_time = Instant::now();

        debug!(
            "Enhancing priority with context-aware scoring for gap: {:?}",
            gap.gap_type
        );

        // Extract context from classification if available
        let extracted_context = if self.should_use_classification() {
            match self.extract_context_from_gap(gap).await {
                Ok(context) => Some(context),
                Err(e) if self.config.enable_graceful_degradation => {
                    warn!(
                        "Context extraction failed, using graceful degradation: {}",
                        e
                    );
                    None
                }
                Err(e) => return Err(e),
            }
        } else {
            None
        };

        // Apply context-aware adjustments
        let (domain_adj, audience_adj, urgency_adj, confidence_weight) =
            self.calculate_context_adjustments(&extracted_context);

        // Calculate final context-enhanced score
        let context_enhanced_score = self.apply_context_enhancements(
            base_breakdown.final_score,
            domain_adj,
            audience_adj,
            urgency_adj,
            confidence_weight,
        );

        let processing_time = start_time.elapsed();

        // Check performance threshold
        let limit = Duration::from_millis(self.config.max_context_overhead_ms);
        if processing_time > limit {
            warn!(
                "Context-aware scoring exceeded time limit: {:?} > {:?}",
                processing_time, limit
            );
            return Err(ContextAwareScoringError::PerformanceThreshold {
                duration: processing_time,
                limit,
            });
        }

        let classification_available = extracted_context.is_some();
        let used_graceful_degradation =
            extracted_context.is_none() && self.config.enable_graceful_degradation;

        let breakdown = ContextAwarePriorityBreakdown {
            base_breakdown,
            extracted_context,
            domain_adjustment: domain_adj,
            audience_adjustment: audience_adj,
            urgency_adjustment: urgency_adj,
            confidence_weighting: confidence_weight,
            context_enhanced_score,
            context_processing_time: processing_time,
            classification_available,
            used_graceful_degradation,
        };

        debug!(
            "Context-aware scoring completed in {:?}: enhanced_score={:.2}",
            processing_time, context_enhanced_score
        );

        Ok(breakdown)
    }

    /// Extract context from a detected gap using classification
    async fn extract_context_from_gap(
        &self,
        gap: &DetectedGap,
    ) -> Result<ExtractedContext, ContextAwareScoringError> {
        // Create classification query from gap information
        let query = self.build_classification_query(gap);
        let cache_key = self.generate_classification_cache_key(&query);

        // Check cache first if enabled
        if self.config.enable_classification_cache {
            if let Some(cached) = self.get_cached_classification(&cache_key).await {
                return self.extract_context_from_classification(cached.result);
            }
        }

        // Perform classification
        let research_type = self.gap_type_to_research_type(&gap.gap_type);
        let classification_result = self.classifier.classify_enhanced(&query, &research_type)?;

        // Cache result if enabled
        if self.config.enable_classification_cache {
            self.cache_classification(cache_key, classification_result.clone())
                .await;
        }

        self.extract_context_from_classification(classification_result)
    }

    /// Build classification query from gap information
    fn build_classification_query(&self, gap: &DetectedGap) -> String {
        // Combine gap context, description, and metadata into a classification query
        let mut query_parts = Vec::new();

        // Add gap description
        query_parts.push(gap.description.clone());

        // Add context if available
        if !gap.context.trim().is_empty() {
            query_parts.push(gap.context.clone());
        }

        // Add metadata context
        if let Some(item_type) = gap.metadata.get("item_type") {
            query_parts.push(format!("item type: {item_type}"));
        }

        // Add file context
        let file_path_str = gap.file_path.to_string_lossy();
        if file_path_str.contains("src/") {
            query_parts.push("source code file".to_string());
        } else if file_path_str.contains("test") {
            query_parts.push("test file".to_string());
        } else if file_path_str.contains("doc") {
            query_parts.push("documentation file".to_string());
        }

        query_parts.join(" ")
    }

    /// Convert gap type to research type for classification
    fn gap_type_to_research_type(&self, gap_type: &GapType) -> ResearchType {
        match gap_type {
            GapType::TodoComment => ResearchType::Implementation,
            GapType::ApiDocumentationGap => ResearchType::Learning,
            GapType::UndocumentedTechnology => ResearchType::Learning,
            GapType::MissingDocumentation => ResearchType::Learning,
            GapType::ConfigurationGap => ResearchType::Implementation,
        }
    }

    /// Extract context from enhanced classification result
    fn extract_context_from_classification(
        &self,
        result: EnhancedClassificationResult,
    ) -> Result<ExtractedContext, ContextAwareScoringError> {
        // Build dimension confidences map
        let mut dimension_confidences = HashMap::new();
        for dim_conf in &result.dimension_confidences {
            dimension_confidences.insert(dim_conf.dimension.clone(), dim_conf.confidence);
        }

        Ok(ExtractedContext {
            research_type: result.research_type,
            audience_level: result.audience_level,
            technical_domain: result.technical_domain,
            urgency_level: result.urgency_level,
            classification_confidence: result.overall_confidence,
            dimension_confidences,
            classification_keywords: result.matched_keywords,
            extracted_at: Utc::now(),
        })
    }

    /// Calculate context-based adjustments
    fn calculate_context_adjustments(
        &self,
        context: &Option<ExtractedContext>,
    ) -> (f64, f64, f64, f64) {
        match context {
            Some(ctx) => {
                let domain_adj = self
                    .config
                    .domain_weights
                    .get_multiplier(&ctx.technical_domain);
                let audience_adj = self
                    .config
                    .audience_adjustments
                    .get_adjustment(&ctx.audience_level);
                let urgency_adj = self
                    .config
                    .urgency_scaling
                    .get_multiplier(&ctx.urgency_level);
                let confidence_weight =
                    ctx.classification_confidence * self.config.confidence_weight;

                (domain_adj, audience_adj, urgency_adj, confidence_weight)
            }
            None => (1.0, 1.0, 1.0, 0.0), // Neutral adjustments when no context
        }
    }

    /// Apply context enhancements to base score
    fn apply_context_enhancements(
        &self,
        base_score: f64,
        domain_adj: f64,
        audience_adj: f64,
        urgency_adj: f64,
        confidence_weight: f64,
    ) -> f64 {
        // Apply multiplicative adjustments
        let mut enhanced_score = base_score * domain_adj * audience_adj * urgency_adj;

        // Add confidence weighting bonus
        enhanced_score += confidence_weight * 10.0; // Scale confidence to 0-10 range

        // Ensure score stays within valid range
        enhanced_score.clamp(0.0, 10.0)
    }

    /// Check if classification should be used
    fn should_use_classification(&self) -> bool {
        // Always try to use classification unless explicitly disabled
        true
    }

    /// Generate cache key for classification result
    fn generate_classification_cache_key(&self, query: &str) -> String {
        // Simple hash of query for cache key
        format!(
            "classification_{}",
            query
                .chars()
                .fold(0u64, |acc, c| acc.wrapping_mul(31).wrapping_add(c as u64))
        )
    }

    /// Get cached classification result
    async fn get_cached_classification(&self, cache_key: &str) -> Option<CachedClassification> {
        let cache = self.classification_cache.read().await;
        cache
            .get(cache_key)
            .filter(|cached| Utc::now() < cached.expires_at)
            .cloned()
    }

    /// Cache classification result
    async fn cache_classification(&self, cache_key: String, result: EnhancedClassificationResult) {
        let expires_at = Utc::now()
            + chrono::Duration::seconds(self.config.classification_cache_ttl_secs as i64);
        let cached = CachedClassification { result, expires_at };

        let mut cache = self.classification_cache.write().await;
        cache.insert(cache_key, cached);
    }

    /// Clear classification cache
    pub async fn clear_classification_cache(&self) {
        let mut cache = self.classification_cache.write().await;
        cache.clear();
        info!("Classification cache cleared");
    }

    /// Get current configuration
    pub fn get_config(&self) -> &ContextAwareScoringConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proactive::TaskPriority;
    use std::path::PathBuf;

    fn create_test_gap() -> DetectedGap {
        DetectedGap {
            gap_type: GapType::TodoComment,
            file_path: PathBuf::from("src/main.rs"),
            line_number: 42,
            column_number: Some(10),
            context: "// TODO: Implement async error handling for web requests".to_string(),
            description: "Implement proper async error handling for HTTP requests".to_string(),
            confidence: 0.9,
            priority: 7,
            metadata: {
                let mut map = HashMap::new();
                map.insert("item_type".to_string(), "function".to_string());
                map
            },
        }
    }

    fn create_test_priority_breakdown() -> PriorityScoreBreakdown {
        PriorityScoreBreakdown {
            final_score: 7.5,
            gap_type_score: 7.0,
            recency_score: 8.0,
            impact_score: 7.0,
            context_score: 8.0,
            priority_level: TaskPriority::High,
            confidence: 0.8,
            calculated_at: Utc::now(),
            impact_assessment: None,
        }
    }

    #[tokio::test]
    async fn test_context_aware_scorer_creation() {
        // FAILING TEST: Context-aware scorer should be creatable with valid configuration
        let config = ContextAwareScoringConfig::default();
        let result = ContextAwarePriorityScorer::new(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_aware_scorer_with_defaults() {
        // FAILING TEST: Default context-aware scorer creation should work
        let result = ContextAwarePriorityScorer::with_defaults().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_domain_priority_weights() {
        // FAILING TEST: Domain priority weights should provide correct multipliers
        let weights = DomainPriorityWeights::default();

        assert_eq!(weights.get_multiplier(&TechnicalDomain::Rust), 1.2);
        assert_eq!(weights.get_multiplier(&TechnicalDomain::Security), 1.4);
        assert_eq!(weights.get_multiplier(&TechnicalDomain::General), 0.9);
        assert_eq!(weights.get_multiplier(&TechnicalDomain::AI), 1.3);
    }

    #[tokio::test]
    async fn test_audience_priority_adjustments() {
        // FAILING TEST: Audience priority adjustments should provide correct values
        let adjustments = AudiencePriorityAdjustments::default();

        assert_eq!(adjustments.get_adjustment(&AudienceLevel::Beginner), 1.2);
        assert_eq!(
            adjustments.get_adjustment(&AudienceLevel::Intermediate),
            1.0
        );
        assert_eq!(adjustments.get_adjustment(&AudienceLevel::Advanced), 0.9);
    }

    #[tokio::test]
    async fn test_urgency_priority_scaling() {
        // FAILING TEST: Urgency priority scaling should provide correct multipliers
        let scaling = UrgencyPriorityScaling::default();

        assert_eq!(scaling.get_multiplier(&UrgencyLevel::Immediate), 2.0);
        assert_eq!(scaling.get_multiplier(&UrgencyLevel::Planned), 1.0);
        assert_eq!(scaling.get_multiplier(&UrgencyLevel::Exploratory), 0.7);
    }

    #[tokio::test]
    async fn test_extracted_context_methods() {
        // FAILING TEST: ExtractedContext should provide dimension confidence access
        let mut dimension_confidences = HashMap::new();
        dimension_confidences.insert(ClassificationDimension::TechnicalDomain, 0.8);
        dimension_confidences.insert(ClassificationDimension::AudienceLevel, 0.6);

        let context = ExtractedContext {
            research_type: ResearchType::Implementation,
            audience_level: AudienceLevel::Intermediate,
            technical_domain: TechnicalDomain::Rust,
            urgency_level: UrgencyLevel::Planned,
            classification_confidence: 0.7,
            dimension_confidences,
            classification_keywords: vec!["rust".to_string(), "async".to_string()],
            extracted_at: Utc::now(),
        };

        assert_eq!(
            context.get_dimension_confidence(&ClassificationDimension::TechnicalDomain),
            0.8
        );
        assert_eq!(
            context.get_dimension_confidence(&ClassificationDimension::AudienceLevel),
            0.6
        );
        assert_eq!(
            context.get_dimension_confidence(&ClassificationDimension::Urgency),
            0.0
        );

        assert!(context.has_high_confidence(0.5));
        assert!(!context.has_high_confidence(0.9));

        let (dim, conf) = context.most_confident_dimension().unwrap();
        assert_eq!(*dim, ClassificationDimension::TechnicalDomain);
        assert_eq!(conf, 0.8);
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        // FAILING TEST: Configuration validation should catch invalid parameters
        // Test invalid confidence threshold
        let config = ContextAwareScoringConfig {
            min_classification_confidence: 1.5,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Test invalid confidence weight
        let config = ContextAwareScoringConfig {
            min_classification_confidence: 0.5,
            confidence_weight: -0.1,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Test invalid overhead time
        let config = ContextAwareScoringConfig {
            confidence_weight: 0.2,
            max_context_overhead_ms: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Test valid configuration
        let config = ContextAwareScoringConfig {
            max_context_overhead_ms: 20,
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn test_gap_type_to_research_type_conversion() {
        // FAILING TEST: Gap type should convert correctly to research type
        let scorer = ContextAwarePriorityScorer::with_defaults().await.unwrap();

        assert_eq!(
            scorer.gap_type_to_research_type(&GapType::TodoComment),
            ResearchType::Implementation
        );
        assert_eq!(
            scorer.gap_type_to_research_type(&GapType::ApiDocumentationGap),
            ResearchType::Learning
        );
        assert_eq!(
            scorer.gap_type_to_research_type(&GapType::UndocumentedTechnology),
            ResearchType::Learning
        );
        assert_eq!(
            scorer.gap_type_to_research_type(&GapType::MissingDocumentation),
            ResearchType::Learning
        );
        assert_eq!(
            scorer.gap_type_to_research_type(&GapType::ConfigurationGap),
            ResearchType::Implementation
        );
    }

    #[tokio::test]
    async fn test_classification_query_building() {
        // FAILING TEST: Classification query should be built correctly from gap
        let scorer = ContextAwarePriorityScorer::with_defaults().await.unwrap();
        let gap = create_test_gap();

        let query = scorer.build_classification_query(&gap);

        assert!(query.contains("Implement proper async error handling"));
        assert!(query.contains("TODO: Implement async error handling"));
        assert!(query.contains("item type: function"));
        assert!(query.contains("source code file"));
    }

    #[tokio::test]
    async fn test_context_adjustments_calculation() {
        // FAILING TEST: Context adjustments should be calculated correctly
        let scorer = ContextAwarePriorityScorer::with_defaults().await.unwrap();

        // Test with context
        let mut dimension_confidences = HashMap::new();
        dimension_confidences.insert(ClassificationDimension::TechnicalDomain, 0.8);

        let context = Some(ExtractedContext {
            research_type: ResearchType::Implementation,
            audience_level: AudienceLevel::Beginner,
            technical_domain: TechnicalDomain::Security,
            urgency_level: UrgencyLevel::Immediate,
            classification_confidence: 0.9,
            dimension_confidences,
            classification_keywords: vec!["security".to_string()],
            extracted_at: Utc::now(),
        });

        let (domain_adj, audience_adj, urgency_adj, confidence_weight) =
            scorer.calculate_context_adjustments(&context);

        assert_eq!(domain_adj, 1.4); // Security domain boost
        assert_eq!(audience_adj, 1.2); // Beginner boost
        assert_eq!(urgency_adj, 2.0); // Immediate urgency
        assert!((confidence_weight - 0.18).abs() < 0.01); // 0.9 * 0.2

        // Test without context
        let (domain_adj, audience_adj, urgency_adj, confidence_weight) =
            scorer.calculate_context_adjustments(&None);

        assert_eq!(domain_adj, 1.0);
        assert_eq!(audience_adj, 1.0);
        assert_eq!(urgency_adj, 1.0);
        assert_eq!(confidence_weight, 0.0);
    }

    #[tokio::test]
    async fn test_context_enhancements_application() {
        // FAILING TEST: Context enhancements should be applied correctly to base score
        let scorer = ContextAwarePriorityScorer::with_defaults().await.unwrap();

        let base_score = 7.0;
        let enhanced_score = scorer.apply_context_enhancements(
            base_score, 1.2,  // domain adjustment
            1.1,  // audience adjustment
            1.5,  // urgency adjustment
            0.15, // confidence weight
        );

        // Expected: 7.0 * 1.2 * 1.1 * 1.5 + 0.15 * 10 = 13.86 + 1.5 = 15.36, clamped to 10.0
        assert_eq!(enhanced_score, 10.0);

        // Test with lower multipliers
        let enhanced_score = scorer.apply_context_enhancements(
            base_score, 1.0,  // neutral domain
            0.9,  // slight audience reduction
            1.0,  // neutral urgency
            0.05, // low confidence
        );

        // Expected: 7.0 * 1.0 * 0.9 * 1.0 + 0.05 * 10 = 6.3 + 0.5 = 6.8
        assert!((enhanced_score - 6.8).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_enhance_priority_with_context_graceful_degradation() {
        // FAILING TEST: Enhancement should work with graceful degradation when classification fails
        let config = ContextAwareScoringConfig {
            enable_graceful_degradation: true,
            max_context_overhead_ms: 50, // Longer timeout for test
            ..Default::default()
        };

        let scorer = ContextAwarePriorityScorer::new(config).await.unwrap();
        let gap = create_test_gap();
        let base_breakdown = create_test_priority_breakdown();

        // This should succeed even if classification has issues
        let result = scorer
            .enhance_priority_with_context(base_breakdown, &gap)
            .await;
        assert!(result.is_ok());

        let breakdown = result.unwrap();
        assert!(breakdown.context_processing_time < Duration::from_millis(50));
        assert!(breakdown.context_enhanced_score >= 0.0);
        assert!(breakdown.context_enhanced_score <= 10.0);
    }

    #[tokio::test]
    async fn test_performance_requirements() {
        // FAILING TEST: Context-aware scoring should meet performance requirements
        let mut config = ContextAwareScoringConfig::default();
        config.max_context_overhead_ms = 20; // 20ms limit

        let scorer = ContextAwarePriorityScorer::new(config).await.unwrap();
        let gap = create_test_gap();
        let base_breakdown = create_test_priority_breakdown();

        let start_time = Instant::now();
        let result = scorer
            .enhance_priority_with_context(base_breakdown, &gap)
            .await;
        let duration = start_time.elapsed();

        // Should either succeed within time limit or fail with performance error
        match result {
            Ok(breakdown) => {
                assert!(breakdown.context_processing_time <= Duration::from_millis(20));
                assert!(duration <= Duration::from_millis(25)); // Allow small overhead
            }
            Err(ContextAwareScoringError::PerformanceThreshold {
                duration: err_duration,
                limit,
            }) => {
                assert!(err_duration > limit);
                assert_eq!(limit, Duration::from_millis(20));
            }
            Err(e) => panic!("Unexpected error: {e:?}"),
        }
    }

    #[tokio::test]
    async fn test_classification_cache_functionality() {
        // FAILING TEST: Classification cache should improve performance
        let mut config = ContextAwareScoringConfig::default();
        config.enable_classification_cache = true;
        config.classification_cache_ttl_secs = 60;

        let scorer = ContextAwarePriorityScorer::new(config).await.unwrap();
        let gap = create_test_gap();
        let base_breakdown = create_test_priority_breakdown();

        // First enhancement should populate cache
        let result1 = scorer
            .enhance_priority_with_context(base_breakdown.clone(), &gap)
            .await;
        assert!(result1.is_ok());

        // Second enhancement should use cache (but we can't easily verify this without exposing cache metrics)
        let result2 = scorer
            .enhance_priority_with_context(base_breakdown, &gap)
            .await;
        assert!(result2.is_ok());

        // Test cache clearing
        scorer.clear_classification_cache().await;
        // This should succeed without error
    }
}
