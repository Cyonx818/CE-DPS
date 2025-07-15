// ABOUTME: Integration layer between learning system adaptation algorithms and template/research systems
//! # Template Integration Module
//!
//! This module provides the integration layer between the learning system's adaptation
//! algorithms and the existing template registry and research systems. It enables
//! prompt optimization algorithms to analyze template performance and apply improvements.
//!
//! ## Key Features
//!
//! - **Template Performance Analysis**: Track template usage and quality metrics
//! - **Adaptive Template Management**: Apply optimization recommendations to templates
//! - **Research System Integration**: Connect learning insights to research workflows
//! - **Quality Feedback Integration**: Bridge quality scoring with template optimization

use crate::learning::{
    AdaptationAlgorithm, AdaptationAlgorithmFactory, AdaptationConfig, AdaptationResult,
    FeedbackData, LearningError, LearningResult, PatternAnalysisResult, UsagePattern,
};
use crate::quality::QualityWeights;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

/// Template performance metrics for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatePerformanceMetrics {
    /// Template identifier
    pub template_id: String,

    /// Template name
    pub template_name: String,

    /// Research type this template serves
    pub research_type: String,

    /// Usage frequency over time period
    pub usage_count: u32,

    /// Average quality score
    pub average_quality: f64,

    /// Quality trend (positive = improving)
    pub quality_trend: f64,

    /// Success rate (successful completions / total uses)
    pub success_rate: f64,

    /// Average response time in milliseconds
    pub avg_response_time_ms: u64,

    /// User satisfaction rating (0.0-1.0)
    pub user_satisfaction: f64,

    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Template optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateOptimizationRecommendation {
    /// Template being optimized
    pub template_id: String,

    /// Optimization priority level
    pub priority: String,

    /// Confidence in the recommendation
    pub confidence: f64,

    /// Specific optimization actions
    pub actions: Vec<OptimizationAction>,

    /// Expected improvement metrics
    pub expected_improvement: ExpectedImprovement,

    /// Generated timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Specific optimization action to take
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationAction {
    /// Type of action
    pub action_type: String,

    /// Description of the action
    pub description: String,

    /// Parameters for the action
    pub parameters: HashMap<String, serde_json::Value>,

    /// Implementation priority
    pub priority: u32,
}

/// Expected improvement from optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedImprovement {
    /// Expected quality improvement
    pub quality_improvement: f64,

    /// Expected performance improvement
    pub performance_improvement: f64,

    /// Expected user satisfaction improvement
    pub satisfaction_improvement: f64,

    /// Confidence in expectations
    pub confidence: f64,
}

/// Integration service for template optimization
pub struct TemplateOptimizationService {
    /// Configuration for the service
    config: IntegrationConfig,

    /// Adaptation algorithms for different optimization aspects
    algorithms: HashMap<String, Box<dyn AdaptationAlgorithm>>,

    /// Template performance cache
    performance_cache: Arc<RwLock<HashMap<String, TemplatePerformanceMetrics>>>,

    /// Quality weights for evaluation
    #[allow(dead_code)] // TODO: Will be used for template quality evaluation
    quality_weights: QualityWeights,
}

/// Configuration for template integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    /// Enable real-time optimization
    pub enable_realtime_optimization: bool,

    /// Enable performance tracking
    pub enable_performance_tracking: bool,

    /// Minimum usage threshold for optimization
    pub min_usage_threshold: u32,

    /// Quality improvement threshold
    pub quality_improvement_threshold: f64,

    /// Performance cache TTL in seconds
    pub cache_ttl_seconds: u64,

    /// Optimization algorithms to enable
    pub enabled_algorithms: Vec<String>,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            enable_realtime_optimization: true,
            enable_performance_tracking: true,
            min_usage_threshold: 10,
            quality_improvement_threshold: 0.1,
            cache_ttl_seconds: 3600, // 1 hour
            enabled_algorithms: vec![
                "prompt_optimizer".to_string(),
                "template_adaptor".to_string(),
                "query_optimizer".to_string(),
            ],
        }
    }
}

impl TemplateOptimizationService {
    /// Create a new template optimization service
    pub async fn new(config: IntegrationConfig) -> LearningResult<Self> {
        let mut algorithms = HashMap::new();
        let adaptation_config = AdaptationConfig::default();

        // Initialize enabled algorithms
        for algorithm_name in &config.enabled_algorithms {
            let algorithm = AdaptationAlgorithmFactory::create_algorithm(
                algorithm_name,
                adaptation_config.clone(),
            )
            .map_err(|e| {
                LearningError::ConfigurationError(format!(
                    "Failed to create algorithm {algorithm_name}: {e}"
                ))
            })?;

            algorithms.insert(algorithm_name.clone(), algorithm);
        }

        Ok(Self {
            config,
            algorithms,
            performance_cache: Arc::new(RwLock::new(HashMap::new())),
            quality_weights: QualityWeights::default(),
        })
    }

    /// Track template performance metrics
    #[instrument(skip(self, metrics))]
    pub async fn update_template_metrics(
        &self,
        metrics: TemplatePerformanceMetrics,
    ) -> LearningResult<()> {
        if !self.config.enable_performance_tracking {
            return Ok(());
        }

        debug!("Updating metrics for template: {}", metrics.template_id);

        let metrics_clone = metrics.clone();

        let mut cache = self.performance_cache.write().await;
        cache.insert(metrics.template_id.clone(), metrics);

        // Trigger optimization if thresholds are met
        if self.config.enable_realtime_optimization {
            self.maybe_trigger_optimization(&metrics_clone).await?;
        }

        Ok(())
    }

    /// Analyze template performance and generate optimization recommendations
    #[instrument(skip(self))]
    pub async fn analyze_template_performance(
        &self,
        template_id: &str,
    ) -> LearningResult<Vec<TemplateOptimizationRecommendation>> {
        let metrics = {
            let cache = self.performance_cache.read().await;
            cache.get(template_id).cloned()
        };

        let Some(metrics) = metrics else {
            return Err(LearningError::NotFound(format!(
                "No metrics found for template: {template_id}"
            )));
        };

        if metrics.usage_count < self.config.min_usage_threshold {
            debug!(
                "Template {} has insufficient usage ({} < {})",
                template_id, metrics.usage_count, self.config.min_usage_threshold
            );
            return Ok(vec![]);
        }

        let mut recommendations = Vec::new();

        // Create feedback data for analysis
        let feedback_data = FeedbackData {
            content_id: template_id.to_string(),
            average_score: metrics.average_quality,
            feedback_count: metrics.usage_count as usize,
            recent_trend: metrics.quality_trend,
        };

        // Run analysis with each algorithm
        for (algorithm_name, algorithm) in &self.algorithms {
            debug!("Running analysis with algorithm: {}", algorithm_name);

            match algorithm.analyze_feedback(&feedback_data).await {
                Ok(result) => {
                    let recommendation = self
                        .create_recommendation_from_result(
                            template_id,
                            algorithm_name,
                            &result,
                            &metrics,
                        )
                        .await?;

                    if recommendation.confidence > 0.5 {
                        recommendations.push(recommendation);
                    }
                }
                Err(e) => {
                    warn!(
                        "Algorithm {} failed for template {}: {}",
                        algorithm_name, template_id, e
                    );
                }
            }
        }

        // Sort by priority and confidence
        recommendations.sort_by(|a, b| {
            let priority_cmp = self
                .priority_value(&a.priority)
                .cmp(&self.priority_value(&b.priority));
            if priority_cmp == std::cmp::Ordering::Equal {
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            } else {
                priority_cmp
            }
        });

        info!(
            "Generated {} recommendations for template {}",
            recommendations.len(),
            template_id
        );

        Ok(recommendations)
    }

    /// Analyze usage patterns for template optimization
    #[instrument(skip(self, patterns))]
    pub async fn analyze_usage_patterns(
        &self,
        patterns: &[UsagePattern],
    ) -> LearningResult<HashMap<String, PatternAnalysisResult>> {
        let mut results = HashMap::new();

        for (algorithm_name, algorithm) in &self.algorithms {
            debug!("Analyzing patterns with algorithm: {}", algorithm_name);

            match algorithm.analyze_patterns(patterns).await {
                Ok(result) => {
                    results.insert(algorithm_name.clone(), result);
                }
                Err(e) => {
                    warn!(
                        "Pattern analysis failed for algorithm {}: {}",
                        algorithm_name, e
                    );
                }
            }
        }

        Ok(results)
    }

    /// Get performance metrics for a template
    pub async fn get_template_metrics(
        &self,
        template_id: &str,
    ) -> Option<TemplatePerformanceMetrics> {
        let cache = self.performance_cache.read().await;
        cache.get(template_id).cloned()
    }

    /// Get all tracked templates
    pub async fn get_all_template_ids(&self) -> Vec<String> {
        let cache = self.performance_cache.read().await;
        cache.keys().cloned().collect()
    }

    /// Apply optimization recommendations and return affected template IDs
    pub async fn apply_recommendations(
        &self,
        recommendations: &[TemplateOptimizationRecommendation],
    ) -> LearningResult<Vec<String>> {
        // Returns the list of template IDs that would be modified by the recommendations
        // Future implementation will integrate with the template registry to apply changes
        Ok(recommendations
            .iter()
            .map(|r| r.template_id.clone())
            .collect())
    }

    // Private helper methods

    async fn maybe_trigger_optimization(
        &self,
        metrics: &TemplatePerformanceMetrics,
    ) -> LearningResult<()> {
        // Check if optimization should be triggered based on metrics
        let should_optimize = metrics.quality_trend < -self.config.quality_improvement_threshold
            || metrics.average_quality < 0.7
            || metrics.success_rate < 0.8;

        if should_optimize {
            debug!(
                "Triggering optimization for template: {}",
                metrics.template_id
            );
            // In a full implementation, this would trigger optimization workflow
        }

        Ok(())
    }

    async fn create_recommendation_from_result(
        &self,
        template_id: &str,
        algorithm_name: &str,
        result: &AdaptationResult,
        metrics: &TemplatePerformanceMetrics,
    ) -> LearningResult<TemplateOptimizationRecommendation> {
        let actions = result
            .recommendations
            .iter()
            .enumerate()
            .map(|(idx, rec)| OptimizationAction {
                action_type: algorithm_name.to_string(),
                description: rec.clone(),
                parameters: HashMap::new(),
                priority: idx as u32,
            })
            .collect();

        let expected_improvement = ExpectedImprovement {
            quality_improvement: self.estimate_quality_improvement(result, metrics),
            performance_improvement: self.estimate_performance_improvement(result, metrics),
            satisfaction_improvement: self.estimate_satisfaction_improvement(result, metrics),
            confidence: result.confidence_score,
        };

        Ok(TemplateOptimizationRecommendation {
            template_id: template_id.to_string(),
            priority: result.priority.clone(),
            confidence: result.confidence_score,
            actions,
            expected_improvement,
            created_at: chrono::Utc::now(),
        })
    }

    fn priority_value(&self, priority: &str) -> i32 {
        match priority {
            "high" => 3,
            "medium" => 2,
            "low" => 1,
            _ => 0,
        }
    }

    fn estimate_quality_improvement(
        &self,
        _result: &AdaptationResult,
        metrics: &TemplatePerformanceMetrics,
    ) -> f64 {
        // Estimate based on current quality and recommendation confidence
        let improvement_potential = 1.0 - metrics.average_quality;
        improvement_potential * 0.3 // Conservative estimate
    }

    fn estimate_performance_improvement(
        &self,
        _result: &AdaptationResult,
        _metrics: &TemplatePerformanceMetrics,
    ) -> f64 {
        0.1 // Placeholder estimate
    }

    fn estimate_satisfaction_improvement(
        &self,
        _result: &AdaptationResult,
        metrics: &TemplatePerformanceMetrics,
    ) -> f64 {
        let improvement_potential = 1.0 - metrics.user_satisfaction;
        improvement_potential * 0.2 // Conservative estimate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_template_optimization_service_creation() {
        let config = IntegrationConfig::default();
        let service = TemplateOptimizationService::new(config).await.unwrap();

        // Should have created the enabled algorithms
        assert!(!service.algorithms.is_empty());
        assert!(service.algorithms.contains_key("prompt_optimizer"));
        assert!(service.algorithms.contains_key("template_adaptor"));
        assert!(service.algorithms.contains_key("query_optimizer"));
    }

    #[tokio::test]
    async fn test_template_metrics_update() {
        let config = IntegrationConfig::default();
        let service = TemplateOptimizationService::new(config).await.unwrap();

        let metrics = TemplatePerformanceMetrics {
            template_id: "test_template".to_string(),
            template_name: "Test Template".to_string(),
            research_type: "decision".to_string(),
            usage_count: 50,
            average_quality: 0.85,
            quality_trend: 0.05,
            success_rate: 0.92,
            avg_response_time_ms: 1500,
            user_satisfaction: 0.88,
            last_updated: Utc::now(),
        };

        service
            .update_template_metrics(metrics.clone())
            .await
            .unwrap();

        let retrieved = service.get_template_metrics("test_template").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().template_id, "test_template");
    }

    #[tokio::test]
    async fn test_template_performance_analysis() {
        let config = IntegrationConfig::default();
        let service = TemplateOptimizationService::new(config).await.unwrap();

        // Add metrics for a template that needs optimization
        let metrics = TemplatePerformanceMetrics {
            template_id: "low_quality_template".to_string(),
            template_name: "Low Quality Template".to_string(),
            research_type: "implementation".to_string(),
            usage_count: 25,      // Above threshold
            average_quality: 0.6, // Low quality
            quality_trend: -0.1,  // Declining
            success_rate: 0.7,
            avg_response_time_ms: 2500,
            user_satisfaction: 0.6,
            last_updated: Utc::now(),
        };

        service.update_template_metrics(metrics).await.unwrap();

        let recommendations = service
            .analyze_template_performance("low_quality_template")
            .await
            .unwrap();

        // Should generate recommendations for poor performing template
        assert!(!recommendations.is_empty());

        // Check that recommendations have high priority
        for rec in &recommendations {
            assert!(rec.priority == "high" || rec.priority == "medium");
            assert!(rec.confidence > 0.5);
        }
    }

    #[tokio::test]
    async fn test_insufficient_usage_threshold() {
        let config = IntegrationConfig::default();
        let service = TemplateOptimizationService::new(config).await.unwrap();

        // Add metrics for a template with low usage
        let metrics = TemplatePerformanceMetrics {
            template_id: "low_usage_template".to_string(),
            template_name: "Low Usage Template".to_string(),
            research_type: "learning".to_string(),
            usage_count: 5, // Below threshold
            average_quality: 0.9,
            quality_trend: 0.0,
            success_rate: 0.95,
            avg_response_time_ms: 1000,
            user_satisfaction: 0.9,
            last_updated: Utc::now(),
        };

        service.update_template_metrics(metrics).await.unwrap();

        let recommendations = service
            .analyze_template_performance("low_usage_template")
            .await
            .unwrap();

        // Should not generate recommendations for low usage templates
        assert!(recommendations.is_empty());
    }

    #[test]
    fn test_integration_config_defaults() {
        let config = IntegrationConfig::default();

        assert!(config.enable_realtime_optimization);
        assert!(config.enable_performance_tracking);
        assert_eq!(config.min_usage_threshold, 10);
        assert_eq!(config.quality_improvement_threshold, 0.1);
        assert_eq!(config.cache_ttl_seconds, 3600);
        assert_eq!(config.enabled_algorithms.len(), 3);
    }
}
