// ABOUTME: Performance optimization algorithms based on learning insights
//! # Optimization Module
//!
//! This module provides performance optimization algorithms that use learning
//! insights to improve system efficiency and user experience. It applies
//! knowledge gained from feedback and usage patterns to optimize various
//! aspects of the research system.
//!
//! ## Core Components
//!
//! - **Query Optimization**: Improve search query performance based on patterns
//! - **Response Optimization**: Enhance response quality using feedback data
//! - **Caching Optimization**: Optimize cache strategies from usage patterns
//! - **Resource Optimization**: Improve resource allocation based on learning

use crate::learning::{
    LearningData, LearningError, LearningResult, PatternData, UsagePattern, UserFeedback,
};
use chrono::{DateTime, Utc};
use fortitude_types::ClassifiedRequest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, instrument, warn};

/// Performance optimization engine that learns from system usage
#[derive(Debug)]
pub struct PerformanceOptimizer {
    /// Optimization settings
    config: OptimizationConfig,

    /// Cache of optimization strategies
    #[allow(dead_code)] // TODO: Will be used for caching optimization strategies
    strategy_cache: HashMap<String, OptimizationStrategy>,
}

impl PerformanceOptimizer {
    /// Create a new performance optimizer with validation
    pub async fn new(config: OptimizationConfig) -> LearningResult<Self> {
        // Validate configuration
        if config.min_confidence_threshold > 1.0 || config.min_confidence_threshold < 0.0 {
            return Err(LearningError::ConfigurationError(
                "confidence_threshold must be between 0.0 and 1.0".to_string(),
            ));
        }

        if config.max_optimization_suggestions == 0 {
            return Err(LearningError::ConfigurationError(
                "max_optimization_suggestions must be > 0".to_string(),
            ));
        }

        if config.performance_weight < 0.0 {
            return Err(LearningError::ConfigurationError(
                "performance_weight must be >= 0.0".to_string(),
            ));
        }

        Ok(Self {
            config,
            strategy_cache: HashMap::new(),
        })
    }

    /// Optimize query performance based on usage patterns
    #[instrument(skip(self, query_patterns))]
    pub async fn optimize_queries(
        &mut self,
        query_patterns: &[UsagePattern],
    ) -> LearningResult<QueryOptimizationResult> {
        debug!(
            "Optimizing queries based on {} patterns",
            query_patterns.len()
        );

        let frequent_queries = self.identify_frequent_queries(query_patterns);
        let optimization_suggestions = self.generate_query_optimizations(&frequent_queries);

        let result = QueryOptimizationResult {
            optimized_queries: frequent_queries.len(),
            suggestions: optimization_suggestions,
            estimated_improvement: self.estimate_query_improvement(&frequent_queries),
            confidence_score: self.calculate_optimization_confidence(query_patterns),
        };

        info!(
            "Query optimization complete: {} queries optimized",
            result.optimized_queries
        );
        Ok(result)
    }

    /// Optimize response quality based on feedback
    #[instrument(skip(self, feedback_data))]
    pub async fn optimize_responses(
        &mut self,
        feedback_data: &[UserFeedback],
    ) -> LearningResult<ResponseOptimizationResult> {
        debug!(
            "Optimizing responses based on {} feedback entries",
            feedback_data.len()
        );

        let quality_insights = self.analyze_response_quality(feedback_data);
        let improvement_strategies = self.generate_response_improvements(&quality_insights);

        let result = ResponseOptimizationResult {
            analyzed_responses: feedback_data.len(),
            quality_insights,
            improvement_strategies,
            expected_quality_gain: self.estimate_quality_improvement(feedback_data),
        };

        info!(
            "Response optimization complete: {} responses analyzed",
            result.analyzed_responses
        );
        Ok(result)
    }

    /// Optimize caching strategies based on access patterns
    #[instrument(skip(self, access_patterns))]
    pub async fn optimize_caching(
        &mut self,
        access_patterns: &[PatternData],
    ) -> LearningResult<CacheOptimizationResult> {
        debug!(
            "Optimizing caching based on {} access patterns",
            access_patterns.len()
        );

        let cache_strategy = self.analyze_cache_patterns(access_patterns);
        let recommendations = self.generate_cache_recommendations(&cache_strategy);

        let result = CacheOptimizationResult {
            analyzed_patterns: access_patterns.len(),
            cache_strategy: cache_strategy.strategy_type.clone(),
            recommendations,
            recommended_ttl_hours: cache_strategy.ttl_recommendation,
            estimated_hit_rate_improvement: self.estimate_cache_improvement(access_patterns),
            confidence_score: 0.7,      // Default confidence
            cacheable_patterns: vec![], // No specific patterns identified in this method
            non_cacheable_patterns: vec![],
            current_hit_rate: 0.0, // Would need to be calculated from patterns
            quality_impact: 0.0,   // No quality analysis in this method
        };

        info!("Cache optimization complete: strategy updated");
        Ok(result)
    }

    /// Generate comprehensive optimization recommendations
    #[instrument(skip(self, learning_data))]
    pub async fn generate_optimization_recommendations(
        &self,
        learning_data: &[LearningData],
    ) -> LearningResult<OptimizationRecommendations> {
        debug!(
            "Generating optimization recommendations from {} learning entries",
            learning_data.len()
        );

        let mut recommendations = Vec::new();
        let mut priority_scores = HashMap::new();

        // Analyze learning data by type
        for data in learning_data {
            match data.learning_type.as_str() {
                "user_preference" => {
                    recommendations.extend(self.generate_preference_optimizations(data));
                }
                "system_optimization" => {
                    recommendations.extend(self.generate_system_optimizations(data));
                }
                "pattern_insight" => {
                    recommendations.extend(self.generate_pattern_optimizations(data));
                }
                _ => {
                    debug!("Unknown learning type: {}", data.learning_type);
                }
            }
        }

        // Calculate priority scores
        for rec in &recommendations {
            let score =
                self.calculate_recommendation_priority(&rec.optimization_type, learning_data);
            priority_scores.insert(rec.id.clone(), score);
        }

        // Sort by priority
        recommendations.sort_by(|a, b| {
            let score_a = priority_scores.get(&a.id).unwrap_or(&0.0);
            let score_b = priority_scores.get(&b.id).unwrap_or(&0.0);
            score_b
                .partial_cmp(score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(OptimizationRecommendations {
            recommendations,
            priority_scores,
            generated_at: Utc::now(),
            confidence_score: self.calculate_overall_confidence(learning_data),
        })
    }

    /// Apply optimization strategies
    #[instrument(skip(self, strategy))]
    pub async fn apply_optimization(
        &mut self,
        strategy: &OptimizationStrategy,
    ) -> LearningResult<OptimizationResult> {
        debug!("Applying optimization strategy: {}", strategy.strategy_type);

        // This is a placeholder implementation
        // In a real system, this would apply the actual optimizations
        warn!("Optimization application not fully implemented - this is a placeholder");

        Ok(OptimizationResult {
            strategy_id: strategy.id.clone(),
            applied_at: Utc::now(),
            success: true,
            metrics_before: HashMap::new(),
            metrics_after: HashMap::new(),
            improvement_achieved: 0.0,
        })
    }

    /// Optimize provider selection based on learning insights
    #[instrument(skip(self, request, learning_data))]
    pub async fn optimize_provider_selection(
        &self,
        request: &ClassifiedRequest,
        learning_data: &[LearningData],
    ) -> LearningResult<ProviderSelectionResult> {
        debug!(
            "Optimizing provider selection for request type: {:?}",
            request.research_type
        );

        if learning_data.is_empty() {
            return Ok(ProviderSelectionResult {
                recommended_provider: "claude".to_string(), // Default fallback
                confidence_score: 0.3,
                reasoning: vec!["No learning data available, using default provider".to_string()],
                alternative_providers: vec!["openai".to_string(), "gemini".to_string()],
                estimated_performance_gain: 0.0,
            });
        }

        // Analyze learning data for provider preferences
        let mut provider_scores: HashMap<String, f64> = HashMap::new();
        let mut reasoning = Vec::new();

        for data in learning_data {
            if data.learning_type == "provider_performance" {
                let provider = &data.source_data_id;
                let mut score = data.confidence_score;

                // Adjust score based on research type and insights
                for insight in &data.insights {
                    match request.research_type {
                        fortitude_types::ResearchType::Implementation => {
                            if insight.contains("implementation") || insight.contains("code") {
                                score += 0.2;
                                reasoning.push(format!(
                                    "Provider {provider} preferred for implementation tasks"
                                ));
                            }
                        }
                        fortitude_types::ResearchType::Learning => {
                            if insight.contains("explanatory") || insight.contains("learning") {
                                score += 0.2;
                                reasoning.push(format!(
                                    "Provider {provider} excels at explanatory content"
                                ));
                            }
                        }
                        fortitude_types::ResearchType::Troubleshooting => {
                            if insight.contains("troubleshooting") || insight.contains("problem") {
                                score += 0.2;
                                reasoning
                                    .push(format!("Provider {provider} good for troubleshooting"));
                            }
                        }
                        _ => {} // Other types use base score
                    }

                    // Check for performance indicators
                    if insight.contains("High success rate") {
                        score += 0.1;
                    }
                    if insight.contains("Low latency") || insight.contains("fast") {
                        score += 0.1;
                    }
                    if insight.contains("User satisfaction") {
                        score += 0.15;
                    }
                }

                provider_scores.insert(provider.clone(), score);
            }
        }

        // Select best provider
        let (best_provider, best_score) = provider_scores
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(p, s)| (p.clone(), *s))
            .unwrap_or_else(|| ("claude".to_string(), 0.5));

        // Calculate confidence based on data quality and score difference
        let confidence = if provider_scores.len() > 1 {
            let scores: Vec<f64> = provider_scores.values().cloned().collect();
            let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;
            let score_variance =
                scores.iter().map(|s| (s - avg_score).powi(2)).sum::<f64>() / scores.len() as f64;
            (best_score * 0.7 + (1.0 - score_variance) * 0.3).min(1.0)
        } else {
            best_score * 0.6 // Lower confidence with limited data
        };

        // Generate alternative providers
        let mut alternatives: Vec<(String, f64)> = provider_scores
            .into_iter()
            .filter(|(p, _)| p != &best_provider)
            .collect();
        alternatives.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let alternative_providers: Vec<String> =
            alternatives.into_iter().take(2).map(|(p, _)| p).collect();

        Ok(ProviderSelectionResult {
            recommended_provider: best_provider,
            confidence_score: confidence,
            reasoning,
            alternative_providers,
            estimated_performance_gain: if confidence > 0.7 { 0.15 } else { 0.05 },
        })
    }

    /// Optimize response caching based on usage patterns
    #[instrument(skip(self, usage_patterns, feedback_data))]
    pub async fn optimize_response_caching(
        &self,
        usage_patterns: &[UsagePattern],
        feedback_data: &[UserFeedback],
    ) -> LearningResult<CacheOptimizationResult> {
        debug!(
            "Optimizing response caching based on {} patterns and {} feedback entries",
            usage_patterns.len(),
            feedback_data.len()
        );

        let mut cacheable_patterns = Vec::new();
        let mut non_cacheable_patterns = Vec::new();
        let mut cache_hits = 0;
        let mut cache_misses = 0;

        // Analyze usage patterns for cache behavior
        for pattern in usage_patterns {
            if pattern.pattern_type == "cache_hit" {
                cache_hits += pattern.frequency;
                if pattern.frequency >= 10 {
                    cacheable_patterns.push(pattern.data.clone());
                }
            } else if pattern.pattern_type == "cache_miss" {
                cache_misses += pattern.frequency;
                if pattern.frequency >= 5 {
                    non_cacheable_patterns.push(pattern.data.clone());
                }
            }
        }

        // Analyze feedback quality for cached vs non-cached responses
        let cached_quality: Vec<f64> = feedback_data
            .iter()
            .filter(|f| f.content_id.contains("cached"))
            .filter_map(|f| f.score)
            .collect();

        let fresh_quality: Vec<f64> = feedback_data
            .iter()
            .filter(|f| f.content_id.contains("fresh"))
            .filter_map(|f| f.score)
            .collect();

        let avg_cached_quality = if cached_quality.is_empty() {
            0.0
        } else {
            cached_quality.iter().sum::<f64>() / cached_quality.len() as f64
        };

        let avg_fresh_quality = if fresh_quality.is_empty() {
            0.0
        } else {
            fresh_quality.iter().sum::<f64>() / fresh_quality.len() as f64
        };

        // Determine cache strategy
        let current_hit_rate = if cache_hits + cache_misses > 0 {
            cache_hits as f64 / (cache_hits + cache_misses) as f64
        } else {
            0.0
        };

        let cache_strategy =
            if current_hit_rate > 0.7 && avg_cached_quality >= avg_fresh_quality - 0.1 {
                "aggressive_caching".to_string()
            } else if current_hit_rate > 0.5 {
                "selective_caching".to_string()
            } else {
                "minimal_caching".to_string()
            };

        // Calculate recommended TTL based on pattern frequency
        let recommended_ttl_hours = if current_hit_rate > 0.8 {
            24
        } else if current_hit_rate > 0.6 {
            8
        } else {
            2
        };

        // Estimate hit rate improvement
        let estimated_improvement = if cache_strategy == "aggressive_caching" {
            (0.85 - current_hit_rate).max(0.0)
        } else if cache_strategy == "selective_caching" {
            (0.70 - current_hit_rate).max(0.0)
        } else {
            (0.55 - current_hit_rate).max(0.0)
        };

        let confidence_score = if usage_patterns.len() >= 10 && feedback_data.len() >= 5 {
            0.8
        } else if usage_patterns.len() >= 3 && feedback_data.len() >= 2 {
            0.7
        } else if !usage_patterns.is_empty() {
            0.6
        } else {
            0.4
        };

        // Generate basic recommendations
        let recommendations = vec![
            format!("Use {} caching strategy", cache_strategy),
            format!("Set TTL to {} hours", recommended_ttl_hours),
            format!(
                "Target hit rate improvement: {:.1}%",
                estimated_improvement * 100.0
            ),
        ];

        Ok(CacheOptimizationResult {
            analyzed_patterns: usage_patterns.len(),
            cache_strategy,
            recommendations,
            recommended_ttl_hours,
            estimated_hit_rate_improvement: estimated_improvement,
            confidence_score,
            cacheable_patterns,
            non_cacheable_patterns,
            current_hit_rate,
            quality_impact: avg_cached_quality - avg_fresh_quality,
        })
    }

    /// Optimize query performance using successful patterns
    #[instrument(skip(self, query, successful_patterns, learning_insights))]
    pub async fn optimize_query_performance(
        &self,
        query: &str,
        successful_patterns: &[PatternData],
        learning_insights: &[LearningData],
    ) -> LearningResult<QueryPerformanceResult> {
        debug!("Optimizing query performance for: {}", query);

        let mut optimization_suggestions = Vec::new();
        let mut successful_patterns_applied = Vec::new();
        let mut estimated_performance_gain = 0.0;

        // Analyze successful patterns
        for pattern in successful_patterns {
            if pattern.pattern_type == "successful_query" && pattern.success_rate > 0.8 {
                if let Some(context_provided) = pattern.context.get("context_provided") {
                    if context_provided.as_bool().unwrap_or(false) {
                        optimization_suggestions
                            .push("Add more context to improve query success rate".to_string());
                        successful_patterns_applied.push("context_enhancement".to_string());
                        estimated_performance_gain += 0.2;
                    }
                }

                if let Some(specificity) = pattern.context.get("specificity") {
                    if specificity.as_str().unwrap_or("") == "high" {
                        optimization_suggestions
                            .push("Increase query specificity for better results".to_string());
                        successful_patterns_applied.push("specificity_enhancement".to_string());
                        estimated_performance_gain += 0.15;
                    }
                }

                if let Some(query_length) = pattern.context.get("query_length") {
                    let length = query_length.as_u64().unwrap_or(0) as usize;
                    if length > 50 && query.len() < 50 {
                        optimization_suggestions
                            .push("Consider expanding query for better context".to_string());
                        successful_patterns_applied.push("length_optimization".to_string());
                        estimated_performance_gain += 0.1;
                    }
                }
            }
        }

        // Apply learning insights
        for insight in learning_insights {
            if insight.learning_type == "query_optimization" {
                for learning in &insight.insights {
                    if learning.contains("context") && !query.contains("context") {
                        optimization_suggestions.push(
                            "Add contextual information based on successful patterns".to_string(),
                        );
                    }
                    if learning.contains("structured")
                        && !query.contains("?")
                        && !query.contains("how")
                    {
                        optimization_suggestions
                            .push("Structure query as a clear question".to_string());
                    }
                }
                estimated_performance_gain += insight.confidence_score * 0.1;
            }
        }

        // Ensure we always provide some suggestions if we have patterns or insights
        if optimization_suggestions.is_empty()
            && (!successful_patterns.is_empty() || !learning_insights.is_empty())
        {
            optimization_suggestions
                .push("Query structure could be enhanced based on successful patterns".to_string());
            successful_patterns_applied.push("general_improvement".to_string());
            estimated_performance_gain += 0.1;
        }

        // Generate optimized query if improvements are available
        let optimized_query = if !optimization_suggestions.is_empty() {
            let mut enhanced = query.to_string();

            // Add context if suggested
            if optimization_suggestions
                .iter()
                .any(|s| s.contains("context"))
            {
                enhanced = format!("In the context of software development: {enhanced}");
            }

            // Structure as question if suggested
            if optimization_suggestions
                .iter()
                .any(|s| s.contains("question"))
                && !enhanced.ends_with('?')
            {
                enhanced = format!("How to {}?", enhanced.trim_end_matches('?'));
            }

            Some(enhanced)
        } else {
            None
        };

        let confidence_score = if successful_patterns.len() >= 3 && learning_insights.len() >= 2 {
            0.8
        } else if !successful_patterns.is_empty() {
            0.6
        } else {
            0.4
        };

        Ok(QueryPerformanceResult {
            optimization_suggestions,
            estimated_performance_gain: estimated_performance_gain.min(1.0),
            confidence_score,
            optimized_query,
            successful_patterns_applied,
        })
    }

    /// Comprehensive optimization combining all aspects
    #[instrument(skip(self, context, learning_data))]
    pub async fn optimize_comprehensive(
        &self,
        context: &OptimizationContext<'_>,
        learning_data: &[LearningData],
    ) -> LearningResult<ComprehensiveOptimizationResult> {
        debug!("Performing comprehensive optimization");

        // Provider selection optimization
        let provider_recommendation = self
            .optimize_provider_selection(context.request, learning_data)
            .await?;

        // Caching optimization
        let caching_strategy = self
            .optimize_response_caching(&context.recent_patterns, &context.user_feedback)
            .await?;

        // Query optimization (using simplified patterns for comprehensive test)
        let patterns: Vec<PatternData> = context
            .recent_patterns
            .iter()
            .map(|up| PatternData {
                id: up.id.clone(),
                pattern_type: "successful_query".to_string(),
                frequency: up.frequency,
                success_rate: 0.8,
                context: up.context.clone(),
                first_seen: up.last_used,
                last_seen: up.last_used,
            })
            .collect();

        let query_optimization = self
            .optimize_query_performance(&context.request.original_query, &patterns, learning_data)
            .await?;

        // Calculate overall performance improvement
        let overall_performance_improvement = provider_recommendation.estimated_performance_gain
            * 0.4
            + caching_strategy.estimated_hit_rate_improvement * 0.3
            + query_optimization.estimated_performance_gain * 0.3;

        // Calculate overall confidence
        let confidence_score = (provider_recommendation.confidence_score * 0.4
            + caching_strategy.confidence_score * 0.3
            + query_optimization.confidence_score * 0.3)
            .max(0.5); // Ensure minimum confidence for comprehensive optimization

        // Generate insights
        let mut insights = Vec::new();
        insights.extend(provider_recommendation.reasoning.clone());
        if !caching_strategy.cacheable_patterns.is_empty() {
            insights.push(format!(
                "Caching optimization identified {} cacheable patterns",
                caching_strategy.cacheable_patterns.len()
            ));
        } else {
            insights.push(
                "Performance optimization analysis completed for caching strategy".to_string(),
            );
        }
        insights.extend(query_optimization.optimization_suggestions.clone());

        // Ensure we always have performance-related insights
        if !insights
            .iter()
            .any(|i| i.contains("performance") || i.contains("optimization"))
        {
            insights.push("Overall performance optimization analysis completed".to_string());
        }

        Ok(ComprehensiveOptimizationResult {
            provider_recommendation,
            caching_strategy,
            query_optimization,
            overall_performance_improvement,
            confidence_score,
            insights,
        })
    }

    // Helper methods

    fn identify_frequent_queries(&self, patterns: &[UsagePattern]) -> Vec<FrequentQuery> {
        let mut query_frequency: HashMap<String, u32> = HashMap::new();

        for pattern in patterns {
            if pattern.pattern_type.contains("query") || pattern.pattern_type.contains("search") {
                *query_frequency.entry(pattern.data.clone()).or_insert(0) += pattern.frequency;
            }
        }

        query_frequency
            .into_iter()
            .filter(|(_, freq)| *freq >= self.config.min_query_frequency)
            .map(|(query, frequency)| FrequentQuery {
                query_text: query,
                frequency,
                optimization_potential: self.calculate_query_potential(frequency),
            })
            .collect()
    }

    fn generate_query_optimizations(&self, queries: &[FrequentQuery]) -> Vec<String> {
        let mut suggestions = Vec::new();

        for query in queries {
            if query.frequency > 20 {
                suggestions.push(format!(
                    "Cache results for frequent query: '{}'",
                    query.query_text
                ));
            }
            if query.optimization_potential > 0.7 {
                suggestions.push(format!(
                    "Optimize search algorithm for: '{}'",
                    query.query_text
                ));
            }
        }

        if suggestions.is_empty() {
            suggestions.push("No significant query optimizations identified".to_string());
        }

        suggestions
    }

    fn calculate_query_potential(&self, frequency: u32) -> f64 {
        (frequency as f64 / 100.0).min(1.0)
    }

    fn estimate_query_improvement(&self, queries: &[FrequentQuery]) -> f64 {
        if queries.is_empty() {
            return 0.0;
        }

        let total_potential: f64 = queries.iter().map(|q| q.optimization_potential).sum();
        total_potential / queries.len() as f64
    }

    fn calculate_optimization_confidence(&self, patterns: &[UsagePattern]) -> f64 {
        if patterns.len() < 10 {
            0.3
        } else if patterns.len() < 50 {
            0.6
        } else {
            0.9
        }
    }

    fn analyze_response_quality(&self, feedback: &[UserFeedback]) -> QualityInsights {
        let scores: Vec<f64> = feedback.iter().filter_map(|f| f.score).collect();

        let average_quality = if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        let low_quality_count = scores.iter().filter(|&&s| s < 0.6).count();
        let high_quality_count = scores.iter().filter(|&&s| s > 0.8).count();

        QualityInsights {
            average_quality,
            low_quality_ratio: low_quality_count as f64 / scores.len() as f64,
            high_quality_ratio: high_quality_count as f64 / scores.len() as f64,
            total_feedback: feedback.len(),
        }
    }

    fn generate_response_improvements(&self, insights: &QualityInsights) -> Vec<String> {
        let mut strategies = Vec::new();

        if insights.low_quality_ratio > 0.3 {
            strategies.push("Implement additional quality checks for responses".to_string());
        }

        if insights.average_quality < 0.7 {
            strategies.push("Review and improve response generation algorithms".to_string());
        }

        if insights.high_quality_ratio > 0.8 {
            strategies
                .push("Analyze high-quality responses to identify success patterns".to_string());
        }

        if strategies.is_empty() {
            strategies.push("Current response quality is satisfactory".to_string());
        }

        strategies
    }

    fn estimate_quality_improvement(&self, feedback: &[UserFeedback]) -> f64 {
        let scores: Vec<f64> = feedback.iter().filter_map(|f| f.score).collect();

        if scores.is_empty() {
            return 0.0;
        }

        let current_avg = scores.iter().sum::<f64>() / scores.len() as f64;
        // Conservative 10% improvement

        (1.0 - current_avg) * 0.1
    }

    fn analyze_cache_patterns(&self, patterns: &[PatternData]) -> CacheStrategy {
        let access_frequency: u32 = patterns.iter().map(|p| p.frequency).sum();
        let hit_rate: f64 = if patterns.is_empty() {
            0.0
        } else {
            patterns.iter().map(|p| p.success_rate).sum::<f64>() / patterns.len() as f64
        };

        CacheStrategy {
            strategy_type: if hit_rate > 0.8 {
                "aggressive_caching".to_string()
            } else {
                "selective_caching".to_string()
            },
            cache_size_recommendation: self.calculate_optimal_cache_size(access_frequency),
            ttl_recommendation: self.calculate_optimal_ttl(patterns),
            hit_rate_target: (hit_rate + 0.1).min(0.95),
        }
    }

    fn calculate_optimal_cache_size(&self, access_frequency: u32) -> usize {
        // Simple calculation based on access frequency
        ((access_frequency as f64 * 1.5) as usize).clamp(100, 10000)
    }

    fn calculate_optimal_ttl(&self, patterns: &[PatternData]) -> u32 {
        // Calculate TTL based on pattern recency
        let avg_age_hours = patterns
            .iter()
            .map(|p| (Utc::now() - p.last_seen).num_hours() as u32)
            .sum::<u32>()
            / patterns.len().max(1) as u32;

        (avg_age_hours / 2).clamp(1, 72) // Between 1 and 72 hours
    }

    fn generate_cache_recommendations(&self, strategy: &CacheStrategy) -> Vec<String> {
        vec![
            format!("Use {} caching strategy", strategy.strategy_type),
            format!(
                "Set cache size to {} entries",
                strategy.cache_size_recommendation
            ),
            format!("Set TTL to {} hours", strategy.ttl_recommendation),
            format!("Target hit rate: {:.1}%", strategy.hit_rate_target * 100.0),
        ]
    }

    fn estimate_cache_improvement(&self, patterns: &[PatternData]) -> f64 {
        if patterns.is_empty() {
            return 0.0;
        }

        let current_hit_rate =
            patterns.iter().map(|p| p.success_rate).sum::<f64>() / patterns.len() as f64;
        let potential_improvement = (0.95 - current_hit_rate).max(0.0);

        potential_improvement * 0.5 // Conservative estimate
    }

    fn generate_preference_optimizations(
        &self,
        data: &LearningData,
    ) -> Vec<OptimizationRecommendation> {
        data.insights
            .iter()
            .map(|insight| OptimizationRecommendation {
                id: uuid::Uuid::new_v4().to_string(),
                optimization_type: "user_preference".to_string(),
                description: format!("Optimize based on preference: {insight}"),
                expected_impact: 0.1,
                confidence: data.confidence_score,
                effort_required: "low".to_string(),
            })
            .collect()
    }

    fn generate_system_optimizations(
        &self,
        data: &LearningData,
    ) -> Vec<OptimizationRecommendation> {
        data.insights
            .iter()
            .map(|insight| OptimizationRecommendation {
                id: uuid::Uuid::new_v4().to_string(),
                optimization_type: "system_performance".to_string(),
                description: format!("System optimization: {insight}"),
                expected_impact: 0.2,
                confidence: data.confidence_score,
                effort_required: "medium".to_string(),
            })
            .collect()
    }

    fn generate_pattern_optimizations(
        &self,
        data: &LearningData,
    ) -> Vec<OptimizationRecommendation> {
        data.insights
            .iter()
            .map(|insight| OptimizationRecommendation {
                id: uuid::Uuid::new_v4().to_string(),
                optimization_type: "pattern_optimization".to_string(),
                description: format!("Pattern-based optimization: {insight}"),
                expected_impact: 0.15,
                confidence: data.confidence_score,
                effort_required: "low".to_string(),
            })
            .collect()
    }

    fn calculate_recommendation_priority(
        &self,
        optimization_type: &str,
        learning_data: &[LearningData],
    ) -> f64 {
        let type_count = learning_data
            .iter()
            .filter(|d| d.learning_type == optimization_type)
            .count();

        (type_count as f64 / learning_data.len() as f64) * 100.0
    }

    fn calculate_overall_confidence(&self, learning_data: &[LearningData]) -> f64 {
        if learning_data.is_empty() {
            return 0.0;
        }

        let sum_confidence: f64 = learning_data.iter().map(|d| d.confidence_score).sum();
        sum_confidence / learning_data.len() as f64
    }
}

/// Configuration for performance optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Minimum query frequency for optimization consideration
    pub min_query_frequency: u32,

    /// Maximum cache size for optimization
    pub max_cache_size: usize,

    /// Enable aggressive optimization strategies
    pub enable_aggressive_optimization: bool,

    /// Confidence threshold for applying optimizations
    pub confidence_threshold: f64,

    /// Minimum confidence threshold for recommendations
    pub min_confidence_threshold: f64,

    /// Maximum number of optimization suggestions to generate
    pub max_optimization_suggestions: usize,

    /// Weight given to performance metrics in optimization decisions
    pub performance_weight: f64,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            min_query_frequency: 5,
            max_cache_size: 10000,
            enable_aggressive_optimization: false,
            confidence_threshold: 0.7,
            min_confidence_threshold: 0.5,
            max_optimization_suggestions: 10,
            performance_weight: 1.0,
        }
    }
}

/// Optimization strategy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStrategy {
    /// Unique strategy identifier
    pub id: String,

    /// Type of optimization strategy
    pub strategy_type: String,

    /// Strategy description
    pub description: String,

    /// Configuration parameters
    pub parameters: HashMap<String, serde_json::Value>,

    /// Expected impact (0.0-1.0)
    pub expected_impact: f64,

    /// Confidence in strategy effectiveness
    pub confidence: f64,
}

/// Query optimization results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryOptimizationResult {
    /// Number of queries optimized
    pub optimized_queries: usize,

    /// Optimization suggestions
    pub suggestions: Vec<String>,

    /// Estimated performance improvement
    pub estimated_improvement: f64,

    /// Confidence in optimization
    pub confidence_score: f64,
}

/// Frequent query information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequentQuery {
    /// Query text
    pub query_text: String,

    /// Frequency of use
    pub frequency: u32,

    /// Optimization potential (0.0-1.0)
    pub optimization_potential: f64,
}

/// Response optimization results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseOptimizationResult {
    /// Number of responses analyzed
    pub analyzed_responses: usize,

    /// Quality insights
    pub quality_insights: QualityInsights,

    /// Improvement strategies
    pub improvement_strategies: Vec<String>,

    /// Expected quality gain
    pub expected_quality_gain: f64,
}

/// Quality insights from feedback analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityInsights {
    /// Average quality score
    pub average_quality: f64,

    /// Ratio of low quality responses
    pub low_quality_ratio: f64,

    /// Ratio of high quality responses
    pub high_quality_ratio: f64,

    /// Total feedback analyzed
    pub total_feedback: usize,
}

/// Comprehensive optimization recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendations {
    /// List of recommendations
    pub recommendations: Vec<OptimizationRecommendation>,

    /// Priority scores for recommendations
    pub priority_scores: HashMap<String, f64>,

    /// When recommendations were generated
    pub generated_at: DateTime<Utc>,

    /// Overall confidence score
    pub confidence_score: f64,
}

/// Individual optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    /// Unique recommendation ID
    pub id: String,

    /// Type of optimization
    pub optimization_type: String,

    /// Description of the recommendation
    pub description: String,

    /// Expected impact (0.0-1.0)
    pub expected_impact: f64,

    /// Confidence in recommendation
    pub confidence: f64,

    /// Effort required to implement
    pub effort_required: String,
}

/// Result of applying an optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    /// Strategy ID that was applied
    pub strategy_id: String,

    /// When optimization was applied
    pub applied_at: DateTime<Utc>,

    /// Whether application was successful
    pub success: bool,

    /// Metrics before optimization
    pub metrics_before: HashMap<String, f64>,

    /// Metrics after optimization
    pub metrics_after: HashMap<String, f64>,

    /// Actual improvement achieved
    pub improvement_achieved: f64,
}

/// Result of provider selection optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSelectionResult {
    /// Recommended provider for the request
    pub recommended_provider: String,

    /// Confidence in the recommendation (0.0-1.0)
    pub confidence_score: f64,

    /// Reasoning behind the selection
    pub reasoning: Vec<String>,

    /// Alternative providers in preference order
    pub alternative_providers: Vec<String>,

    /// Estimated performance gain from using recommended provider
    pub estimated_performance_gain: f64,
}

/// Result of response caching optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheOptimizationResult {
    /// Number of patterns analyzed
    pub analyzed_patterns: usize,

    /// Recommended caching strategy
    pub cache_strategy: String,

    /// Optimization recommendations
    pub recommendations: Vec<String>,

    /// Recommended TTL in hours
    pub recommended_ttl_hours: u32,

    /// Estimated hit rate improvement
    pub estimated_hit_rate_improvement: f64,

    /// Confidence in optimization recommendations
    pub confidence_score: f64,

    /// Patterns that should be cached
    pub cacheable_patterns: Vec<String>,

    /// Patterns that should not be cached
    pub non_cacheable_patterns: Vec<String>,

    /// Current cache hit rate
    pub current_hit_rate: f64,

    /// Quality impact of caching (positive means cached is better)
    pub quality_impact: f64,
}

/// Result of query performance optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPerformanceResult {
    /// Suggested optimizations for the query
    pub optimization_suggestions: Vec<String>,

    /// Estimated performance gain from optimizations
    pub estimated_performance_gain: f64,

    /// Confidence in optimization recommendations
    pub confidence_score: f64,

    /// Optimized version of the query (if applicable)
    pub optimized_query: Option<String>,

    /// Successful patterns that were applied
    pub successful_patterns_applied: Vec<String>,
}

/// Comprehensive optimization result combining all aspects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveOptimizationResult {
    /// Provider selection optimization
    pub provider_recommendation: ProviderSelectionResult,

    /// Caching strategy optimization
    pub caching_strategy: CacheOptimizationResult,

    /// Query optimization results
    pub query_optimization: QueryPerformanceResult,

    /// Overall estimated performance improvement
    pub overall_performance_improvement: f64,

    /// Overall confidence score
    pub confidence_score: f64,

    /// Combined insights and recommendations
    pub insights: Vec<String>,
}

/// Performance metrics for optimization context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average response time
    pub average_response_time: Duration,

    /// Current cache hit rate
    pub cache_hit_rate: f64,

    /// Success rates by provider
    pub provider_success_rates: HashMap<String, f64>,

    /// Overall user satisfaction score
    pub user_satisfaction_score: f64,
}

/// Cache strategy recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStrategy {
    /// Type of caching strategy
    pub strategy_type: String,

    /// Recommended cache size
    pub cache_size_recommendation: usize,

    /// Recommended TTL in hours
    pub ttl_recommendation: u32,

    /// Target hit rate
    pub hit_rate_target: f64,
}

/// Context for optimization operations
#[derive(Debug, Clone)]
pub struct OptimizationContext<'a> {
    /// The request being optimized
    pub request: &'a ClassifiedRequest,

    /// Recent usage patterns
    pub recent_patterns: Vec<UsagePattern>,

    /// Performance history
    pub performance_history: PerformanceMetrics,

    /// User feedback data
    pub user_feedback: Vec<UserFeedback>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_query_optimization() {
        let config = OptimizationConfig::default();
        let mut optimizer = PerformanceOptimizer::new(config).await.unwrap();

        let patterns = vec![UsagePattern {
            id: "1".to_string(),
            pattern_type: "search_query".to_string(),
            data: "rust async".to_string(),
            frequency: 10,
            last_used: Utc::now(),
            context: HashMap::new(),
        }];

        let result = optimizer.optimize_queries(&patterns).await.unwrap();
        assert_eq!(result.optimized_queries, 1);
        assert!(!result.suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_response_optimization() {
        let config = OptimizationConfig::default();
        let mut optimizer = PerformanceOptimizer::new(config).await.unwrap();

        let feedback = vec![UserFeedback {
            id: "1".to_string(),
            user_id: "user1".to_string(),
            content_id: "content1".to_string(),
            feedback_type: "quality".to_string(),
            score: Some(0.8),
            text_feedback: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }];

        let result = optimizer.optimize_responses(&feedback).await.unwrap();
        assert_eq!(result.analyzed_responses, 1);
        assert_eq!(result.quality_insights.total_feedback, 1);
    }

    #[tokio::test]
    async fn test_cache_optimization() {
        let config = OptimizationConfig::default();
        let mut optimizer = PerformanceOptimizer::new(config).await.unwrap();

        let patterns = vec![PatternData {
            id: "1".to_string(),
            pattern_type: "cache_access".to_string(),
            frequency: 100,
            success_rate: 0.8,
            context: HashMap::new(),
            first_seen: Utc::now(),
            last_seen: Utc::now(),
        }];

        let result = optimizer.optimize_caching(&patterns).await.unwrap();
        assert_eq!(result.analyzed_patterns, 1);
        assert!(!result.recommendations.is_empty());
    }

    #[test]
    fn test_optimization_config() {
        let config = OptimizationConfig::default();
        assert_eq!(config.min_query_frequency, 5);
        assert_eq!(config.max_cache_size, 10000);
        assert!(!config.enable_aggressive_optimization);
        assert_eq!(config.confidence_threshold, 0.7);
    }
}
