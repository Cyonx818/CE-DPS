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

// ABOUTME: System adaptation algorithms based on learning feedback and patterns
//! # Adaptation Module
//!
//! This module provides algorithms and interfaces for system adaptation based on
//! learning data. It analyzes user feedback and usage patterns to recommend
//! improvements to the research system.
//!
//! ## Core Components
//!
//! - **Feedback Analysis**: Analyze user feedback to identify improvement areas
//! - **Pattern-Based Adaptation**: Use usage patterns to optimize system behavior
//! - **Confidence Scoring**: Evaluate reliability of adaptation recommendations
//! - **Recommendation Engine**: Generate actionable improvement suggestions

use crate::learning::{
    AdaptationAlgorithm, AdaptationConfig, AdaptationResult, FeedbackData, LearningError,
    LearningResult, PatternAnalysisResult, UsagePattern,
};
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use tracing::{debug, info, instrument};

/// Primary feedback analyzer that learns from user ratings and comments
pub struct FeedbackAnalyzer {
    /// Configuration for the analyzer
    config: AdaptationConfig,

    /// Minimum feedback threshold for reliable analysis
    min_feedback_threshold: usize,

    /// Confidence threshold for recommendations
    confidence_threshold: f64,
}

impl FeedbackAnalyzer {
    /// Create a new feedback analyzer
    pub fn new(config: AdaptationConfig) -> Self {
        Self {
            min_feedback_threshold: 5,
            confidence_threshold: 0.7,
            config,
        }
    }

    /// Analyze feedback trends to identify quality patterns
    fn analyze_feedback_trends(&self, feedback: &FeedbackData) -> LearningResult<Vec<String>> {
        let mut recommendations = Vec::new();

        // Analyze average score
        if feedback.average_score < 0.6 {
            recommendations.push("Content quality needs significant improvement".to_string());
        } else if feedback.average_score < 0.8 {
            recommendations.push("Content quality could be enhanced".to_string());
        } else if feedback.average_score > 0.9 {
            recommendations.push("Maintain current high quality standards".to_string());
        }

        // Analyze trend direction
        if feedback.recent_trend < -0.1 {
            recommendations
                .push("Quality appears to be declining - investigate recent changes".to_string());
        } else if feedback.recent_trend > 0.1 {
            recommendations.push("Quality is improving - continue current approach".to_string());
        }

        // Analyze feedback volume
        if feedback.feedback_count < self.min_feedback_threshold {
            recommendations
                .push("Insufficient feedback data - encourage more user feedback".to_string());
        }

        Ok(recommendations)
    }

    /// Calculate confidence score for feedback analysis
    fn calculate_feedback_confidence(&self, feedback: &FeedbackData) -> f64 {
        let volume_confidence = (feedback.feedback_count as f64 / 20.0).min(1.0);
        let consistency_confidence = 1.0 - feedback.recent_trend.abs().min(1.0);
        let score_confidence = if feedback.average_score > 0.0 {
            1.0
        } else {
            0.0
        };

        (volume_confidence + consistency_confidence + score_confidence) / 3.0
    }
}

#[async_trait]
impl AdaptationAlgorithm for FeedbackAnalyzer {
    #[instrument(skip(self, feedback))]
    async fn analyze_feedback(&self, feedback: &FeedbackData) -> LearningResult<AdaptationResult> {
        debug!("Analyzing feedback for content: {}", feedback.content_id);

        let recommendations = self.analyze_feedback_trends(feedback)?;
        let confidence_score = self.calculate_feedback_confidence(feedback);

        let priority = if confidence_score > self.confidence_threshold {
            if feedback.average_score < 0.7 {
                "high"
            } else if feedback.average_score < 0.85 {
                "medium"
            } else {
                "low"
            }
        } else {
            "low" // Low confidence means low priority
        };

        info!(
            "Feedback analysis complete - confidence: {:.2}, priority: {}",
            confidence_score, priority
        );

        Ok(AdaptationResult {
            recommendations,
            confidence_score,
            priority: priority.to_string(),
        })
    }

    #[instrument(skip(self, patterns))]
    async fn analyze_patterns(
        &self,
        patterns: &[UsagePattern],
    ) -> LearningResult<PatternAnalysisResult> {
        debug!("Analyzing {} usage patterns", patterns.len());

        // Basic pattern analysis - this is a skeleton implementation
        let insights = patterns
            .iter()
            .map(|p| format!("Pattern '{}' used {} times", p.data, p.frequency))
            .collect();

        let recommendations = vec![
            "Continue monitoring usage patterns".to_string(),
            "Consider optimizing frequently used patterns".to_string(),
        ];

        Ok(PatternAnalysisResult {
            insights,
            confidence_score: 0.7, // Placeholder confidence
            recommendations,
        })
    }

    fn get_config(&self) -> &AdaptationConfig {
        &self.config
    }
}

/// Pattern-based adaptation algorithm that learns from user behavior
pub struct PatternMatcher {
    /// Configuration for the matcher
    config: AdaptationConfig,

    /// Pattern frequency threshold for significance
    frequency_threshold: u32,
}

impl PatternMatcher {
    /// Create a new pattern matcher
    pub fn new(config: AdaptationConfig) -> Self {
        Self {
            frequency_threshold: 3,
            config,
        }
    }

    /// Identify significant patterns based on frequency and recency
    fn identify_significant_patterns<'a>(
        &self,
        patterns: &'a [UsagePattern],
    ) -> Vec<&'a UsagePattern> {
        patterns
            .iter()
            .filter(|p| p.frequency >= self.frequency_threshold)
            .collect()
    }

    /// Generate insights from pattern analysis
    fn generate_pattern_insights(&self, patterns: &[&UsagePattern]) -> Vec<String> {
        let mut insights = Vec::new();

        // Group patterns by type
        let mut pattern_groups: HashMap<String, Vec<&UsagePattern>> = HashMap::new();
        for pattern in patterns {
            pattern_groups
                .entry(pattern.pattern_type.clone())
                .or_default()
                .push(pattern);
        }

        // Analyze each group
        for (pattern_type, group_patterns) in pattern_groups {
            let total_frequency: u32 = group_patterns.iter().map(|p| p.frequency).sum();
            insights.push(format!(
                "Pattern type '{}' shows {} total occurrences across {} variants",
                pattern_type,
                total_frequency,
                group_patterns.len()
            ));

            // Find most frequent pattern in group
            if let Some(top_pattern) = group_patterns.iter().max_by_key(|p| p.frequency) {
                insights.push(format!(
                    "Most frequent '{}' pattern: '{}' ({} times)",
                    pattern_type, top_pattern.data, top_pattern.frequency
                ));
            }
        }

        insights
    }
}

#[async_trait]
impl AdaptationAlgorithm for PatternMatcher {
    #[instrument(skip(self, feedback))]
    async fn analyze_feedback(&self, feedback: &FeedbackData) -> LearningResult<AdaptationResult> {
        debug!(
            "Pattern matcher analyzing feedback for: {}",
            feedback.content_id
        );

        // Pattern matcher provides basic feedback analysis
        let recommendations =
            vec!["Pattern-based analysis suggests monitoring user behavior".to_string()];

        Ok(AdaptationResult {
            recommendations,
            confidence_score: 0.5, // Lower confidence for pattern-based feedback analysis
            priority: "low".to_string(),
        })
    }

    #[instrument(skip(self, patterns))]
    async fn analyze_patterns(
        &self,
        patterns: &[UsagePattern],
    ) -> LearningResult<PatternAnalysisResult> {
        debug!("Pattern matcher analyzing {} patterns", patterns.len());

        let significant_patterns = self.identify_significant_patterns(patterns);
        let insights = self.generate_pattern_insights(&significant_patterns);

        let mut recommendations = Vec::new();

        if significant_patterns.is_empty() {
            recommendations.push("No significant usage patterns detected yet".to_string());
        } else {
            recommendations.push("Consider optimizing for detected usage patterns".to_string());

            if significant_patterns.len() > 10 {
                recommendations
                    .push("Many patterns detected - consider pattern consolidation".to_string());
            }
        }

        let confidence_score = if significant_patterns.len() > 5 {
            0.8
        } else if significant_patterns.len() > 2 {
            0.6
        } else {
            0.4
        };

        Ok(PatternAnalysisResult {
            insights,
            confidence_score,
            recommendations,
        })
    }

    fn get_config(&self) -> &AdaptationConfig {
        &self.config
    }
}

/// Prompt optimization adaptation algorithm for improving research prompts
pub struct PromptOptimizer {
    /// Configuration for the optimizer
    config: AdaptationConfig,

    /// Confidence threshold for high-priority recommendations
    #[allow(dead_code)] // TODO: Will be used for adaptive prompt optimization
    confidence_threshold: f64,

    /// Quality threshold below which optimization is considered urgent
    urgent_optimization_threshold: f64,
}

impl PromptOptimizer {
    /// Create a new prompt optimizer
    pub fn new(config: AdaptationConfig) -> Self {
        Self {
            confidence_threshold: 0.75,
            urgent_optimization_threshold: 0.6,
            config,
        }
    }

    /// Analyze prompt performance and suggest optimizations
    fn analyze_prompt_performance(&self, feedback: &FeedbackData) -> LearningResult<Vec<String>> {
        let mut recommendations = Vec::new();

        // Analyze score-based optimizations
        if feedback.average_score < self.urgent_optimization_threshold {
            recommendations.push("Critical: Prompt requires immediate optimization - quality below acceptable threshold".to_string());
            recommendations.push("Consider restructuring prompt with clearer instructions and specific output format requirements".to_string());
            recommendations.push(
                "Add explicit quality criteria and examples to guide better responses".to_string(),
            );
        } else if feedback.average_score < 0.8 {
            recommendations.push(
                "Moderate optimization needed - enhance prompt clarity and specificity".to_string(),
            );
            recommendations.push(
                "Consider adding context examples or refining parameter substitutions".to_string(),
            );
        } else if feedback.average_score > 0.9 {
            recommendations.push("High-performing prompt - maintain current structure and consider as template for similar use cases".to_string());
        }

        // Analyze trend-based optimizations
        if feedback.recent_trend < -0.05 {
            recommendations.push("Quality declining - investigate recent prompt changes and revert problematic modifications".to_string());
            recommendations.push("Consider A/B testing alternative prompt variations to identify improvement strategies".to_string());
        } else if feedback.recent_trend > 0.05 {
            recommendations.push("Quality improving - analyze successful recent changes and apply patterns to other prompts".to_string());
        }

        // Volume-based recommendations
        if feedback.feedback_count < 10 {
            recommendations.push("Insufficient feedback data for reliable optimization - increase usage before making changes".to_string());
        } else if feedback.feedback_count > 50 {
            recommendations.push("Substantial feedback data available - consider detailed analysis for fine-tuning optimizations".to_string());
        }

        Ok(recommendations)
    }

    /// Calculate confidence for prompt optimization recommendations
    fn calculate_optimization_confidence(&self, feedback: &FeedbackData) -> f64 {
        let volume_confidence = (feedback.feedback_count as f64 / 30.0).min(1.0);
        let trend_confidence = if feedback.recent_trend.abs() > 0.02 {
            0.8
        } else {
            0.6
        };
        let score_confidence = if feedback.average_score > 0.0 {
            1.0
        } else {
            0.3
        };

        (volume_confidence + trend_confidence + score_confidence) / 3.0
    }
}

#[async_trait]
impl AdaptationAlgorithm for PromptOptimizer {
    #[instrument(skip(self, feedback))]
    async fn analyze_feedback(&self, feedback: &FeedbackData) -> LearningResult<AdaptationResult> {
        debug!(
            "Prompt optimizer analyzing feedback for: {}",
            feedback.content_id
        );

        let recommendations = self.analyze_prompt_performance(feedback)?;
        let confidence_score = self.calculate_optimization_confidence(feedback);

        let priority = if feedback.average_score <= self.urgent_optimization_threshold {
            "high"
        } else if feedback.average_score < 0.8 || feedback.recent_trend < -0.05 {
            "medium"
        } else {
            "low"
        };

        info!(
            "Prompt optimization analysis complete - score: {:.2}, confidence: {:.2}, priority: {}",
            feedback.average_score, confidence_score, priority
        );

        Ok(AdaptationResult {
            recommendations,
            confidence_score,
            priority: priority.to_string(),
        })
    }

    #[instrument(skip(self, patterns))]
    async fn analyze_patterns(
        &self,
        patterns: &[UsagePattern],
    ) -> LearningResult<PatternAnalysisResult> {
        debug!("Prompt optimizer analyzing {} patterns", patterns.len());

        let mut insights = Vec::new();
        let mut recommendations = Vec::new();

        // Group patterns by type and analyze
        let mut pattern_groups: HashMap<String, Vec<&UsagePattern>> = HashMap::new();
        for pattern in patterns {
            pattern_groups
                .entry(pattern.pattern_type.clone())
                .or_default()
                .push(pattern);
        }

        for (pattern_type, group_patterns) in pattern_groups {
            let total_frequency: u32 = group_patterns.iter().map(|p| p.frequency).sum();

            if pattern_type.contains("successful") {
                insights.push(format!(
                    "Successful prompt patterns: '{pattern_type}' type shows {total_frequency} total usages"
                ));
                recommendations.push(
                    "Incorporate successful pattern elements into underperforming prompts"
                        .to_string(),
                );
            } else if pattern_type.contains("failed") || pattern_type.contains("poor") {
                insights.push(format!(
                    "Problematic prompt patterns: '{pattern_type}' type shows {total_frequency} failures"
                ));
                recommendations.push(
                    "Identify and eliminate failed pattern elements from active prompts"
                        .to_string(),
                );
            }
        }

        if insights.is_empty() {
            insights.push("No significant prompt optimization patterns detected yet".to_string());
            recommendations.push(
                "Continue monitoring prompt performance to identify optimization opportunities"
                    .to_string(),
            );
        }

        let confidence_score = if patterns.len() > 10 { 0.8 } else { 0.5 };

        Ok(PatternAnalysisResult {
            insights,
            confidence_score,
            recommendations,
        })
    }

    fn get_config(&self) -> &AdaptationConfig {
        &self.config
    }
}

/// Query optimization adaptation algorithm for improving search queries
pub struct QueryOptimizer {
    /// Configuration for the optimizer
    config: AdaptationConfig,

    /// Minimum frequency threshold for pattern significance
    frequency_threshold: u32,

    /// Success rate threshold for considering patterns effective
    #[allow(dead_code)] // TODO: Will be used for query pattern optimization
    success_rate_threshold: f64,
}

impl QueryOptimizer {
    /// Create a new query optimizer
    pub fn new(config: AdaptationConfig) -> Self {
        Self {
            frequency_threshold: 5,
            success_rate_threshold: 0.7,
            config,
        }
    }

    /// Analyze query patterns to identify optimization opportunities
    fn analyze_query_patterns(
        &self,
        patterns: &[UsagePattern],
    ) -> LearningResult<(Vec<String>, Vec<String>)> {
        let mut insights = Vec::new();
        let mut recommendations = Vec::new();

        // Separate successful and unsuccessful query patterns
        let successful_patterns: Vec<&UsagePattern> = patterns
            .iter()
            .filter(|p| {
                p.pattern_type.contains("successful") && p.frequency >= self.frequency_threshold
            })
            .collect();

        let failed_patterns: Vec<&UsagePattern> = patterns
            .iter()
            .filter(|p| p.pattern_type.contains("failed") || p.pattern_type.contains("poor"))
            .collect();

        // Analyze successful patterns
        if !successful_patterns.is_empty() {
            insights.push(format!(
                "Identified {} high-frequency successful query patterns",
                successful_patterns.len()
            ));

            // Find common elements in successful queries
            let successful_keywords: HashSet<String> = successful_patterns
                .iter()
                .flat_map(|p| p.data.split_whitespace())
                .filter(|word| word.len() > 3) // Filter out short words
                .map(|word| word.to_lowercase())
                .collect();

            if successful_keywords.len() > 5 {
                insights.push(format!(
                    "Successful queries commonly include terms: {:?}",
                    successful_keywords.iter().take(5).collect::<Vec<_>>()
                ));
                recommendations.push(
                    "Incorporate high-performing keywords and phrases into new queries".to_string(),
                );
            }

            recommendations.push(
                "Template successful query structures for reuse in similar research contexts"
                    .to_string(),
            );
        }

        // Analyze failed patterns
        if !failed_patterns.is_empty() {
            insights.push(format!(
                "Identified {} failed query patterns to avoid",
                failed_patterns.len()
            ));

            recommendations.push(
                "Avoid query structures and terms that have shown poor performance".to_string(),
            );
            recommendations.push("Refine vague or overly broad query formulations".to_string());
        }

        // Pattern frequency analysis
        let high_frequency_patterns: Vec<&UsagePattern> = patterns
            .iter()
            .filter(|p| p.frequency >= self.frequency_threshold)
            .collect();

        if high_frequency_patterns.len() > 3 {
            insights.push(format!(
                "High-frequency patterns suggest {} common query types in use",
                high_frequency_patterns.len()
            ));
            recommendations
                .push("Optimize most commonly used query patterns for maximum impact".to_string());
        }

        Ok((insights, recommendations))
    }
}

#[async_trait]
impl AdaptationAlgorithm for QueryOptimizer {
    #[instrument(skip(self, feedback))]
    async fn analyze_feedback(&self, feedback: &FeedbackData) -> LearningResult<AdaptationResult> {
        debug!(
            "Query optimizer analyzing feedback for: {}",
            feedback.content_id
        );

        // Basic feedback analysis for query optimization
        let mut recommendations = Vec::new();

        if feedback.average_score < 0.8 {
            recommendations.push(
                "Query performance could be improved - consider reformulating search terms"
                    .to_string(),
            );
        }

        if feedback.recent_trend < -0.01 {
            recommendations.push(
                "Query effectiveness declining - review and update search strategy".to_string(),
            );
        }

        if feedback.feedback_count > 10 {
            recommendations
                .push("Monitor query patterns for optimization opportunities".to_string());
        }

        // Always provide at least basic guidance
        if recommendations.is_empty() {
            recommendations.push("Continue monitoring query performance and patterns".to_string());
        }

        let confidence_score = if feedback.feedback_count > 15 {
            0.7
        } else {
            0.6
        };

        Ok(AdaptationResult {
            recommendations,
            confidence_score,
            priority: "medium".to_string(),
        })
    }

    #[instrument(skip(self, patterns))]
    async fn analyze_patterns(
        &self,
        patterns: &[UsagePattern],
    ) -> LearningResult<PatternAnalysisResult> {
        debug!("Query optimizer analyzing {} patterns", patterns.len());

        let (insights, recommendations) = self.analyze_query_patterns(patterns)?;

        // Calculate confidence based on pattern quality and diversity
        let successful_patterns = patterns
            .iter()
            .filter(|p| p.pattern_type.contains("successful"))
            .count();
        let failed_patterns = patterns
            .iter()
            .filter(|p| p.pattern_type.contains("failed") || p.pattern_type.contains("poor"))
            .count();

        let confidence_score = if patterns.len() >= 10 {
            0.85
        } else if patterns.len() >= 5 {
            0.7
        } else if successful_patterns > 0 && failed_patterns > 0 {
            // High confidence when we have both success and failure examples
            0.75
        } else if successful_patterns >= 2 {
            // Good confidence with multiple success patterns
            0.7
        } else {
            0.5
        };

        Ok(PatternAnalysisResult {
            insights,
            confidence_score,
            recommendations,
        })
    }

    fn get_config(&self) -> &AdaptationConfig {
        &self.config
    }
}

/// Template adaptation algorithm for evolving prompt templates based on feedback
pub struct TemplateAdaptor {
    /// Configuration for the adaptor
    config: AdaptationConfig,

    /// Template performance threshold for adaptation decisions
    performance_threshold: f64,

    /// Confidence threshold for template modifications
    modification_confidence_threshold: f64,
}

impl TemplateAdaptor {
    /// Create a new template adaptor
    pub fn new(config: AdaptationConfig) -> Self {
        Self {
            performance_threshold: 0.75,
            modification_confidence_threshold: 0.8,
            config,
        }
    }

    /// Analyze template performance and suggest adaptations
    fn analyze_template_performance(&self, feedback: &FeedbackData) -> LearningResult<Vec<String>> {
        let mut recommendations = Vec::new();

        // High-performing template recommendations
        if feedback.average_score >= 0.9 {
            recommendations.push("Excellent template performance - maintain current structure and promote as best practice".to_string());
            recommendations.push(
                "Consider extracting successful elements for use in other templates".to_string(),
            );
            recommendations.push(
                "Document template patterns for knowledge base and training materials".to_string(),
            );
            recommendations.push(
                "Enhance documentation and consider this template as a reference standard"
                    .to_string(),
            );
        }
        // Good performing template recommendations
        else if feedback.average_score >= self.performance_threshold {
            recommendations.push(
                "Good template performance - minor enhancements may yield further improvements"
                    .to_string(),
            );
            recommendations
                .push("Test incremental modifications to optimize specific parameters".to_string());
        }
        // Underperforming template recommendations
        else if feedback.average_score < 0.6 {
            recommendations.push(
                "Template underperforming - consider significant restructuring or replacement"
                    .to_string(),
            );
            recommendations.push(
                "Analyze high-performing templates for structural patterns to incorporate"
                    .to_string(),
            );
            recommendations.push(
                "Review parameter definitions and validation rules for improvements".to_string(),
            );
        }
        // Moderately performing template recommendations
        else {
            recommendations.push(
                "Template showing moderate performance - targeted improvements recommended"
                    .to_string(),
            );
            recommendations
                .push("Focus on clarity of instructions and parameter specifications".to_string());
        }

        // Trend-based template adaptations
        if feedback.recent_trend > 0.1 {
            recommendations.push(
                "Template performance improving - continue current optimization direction"
                    .to_string(),
            );
            recommendations.push("Accelerate successful modification patterns".to_string());
        } else if feedback.recent_trend < -0.1 {
            recommendations.push(
                "Template performance declining - investigate recent changes and consider rollback"
                    .to_string(),
            );
            recommendations
                .push("Implement A/B testing to validate template modifications".to_string());
        }

        // Data volume considerations
        if feedback.feedback_count >= 30 {
            recommendations.push(
                "Substantial feedback data supports reliable template adaptation decisions"
                    .to_string(),
            );
        } else if feedback.feedback_count < 10 {
            recommendations.push(
                "Limited feedback data - proceed cautiously with template modifications"
                    .to_string(),
            );
        }

        Ok(recommendations)
    }

    /// Calculate confidence for template adaptation recommendations
    fn calculate_adaptation_confidence(&self, feedback: &FeedbackData) -> f64 {
        let volume_confidence = (feedback.feedback_count as f64 / 25.0).min(1.0);
        let stability_confidence = 1.0 - feedback.recent_trend.abs().min(0.5);
        let performance_confidence = if feedback.average_score > 0.5 {
            feedback.average_score
        } else {
            0.3
        };

        (volume_confidence + stability_confidence + performance_confidence) / 3.0
    }
}

#[async_trait]
impl AdaptationAlgorithm for TemplateAdaptor {
    #[instrument(skip(self, feedback))]
    async fn analyze_feedback(&self, feedback: &FeedbackData) -> LearningResult<AdaptationResult> {
        debug!(
            "Template adaptor analyzing feedback for: {}",
            feedback.content_id
        );

        let recommendations = self.analyze_template_performance(feedback)?;
        let confidence_score = self.calculate_adaptation_confidence(feedback);

        let priority = if feedback.average_score < 0.6
            && confidence_score > self.modification_confidence_threshold
        {
            "high"
        } else if feedback.average_score >= 0.9 {
            "low" // High-performing templates need low priority changes
        } else {
            "medium"
        };

        info!(
            "Template adaptation analysis complete - score: {:.2}, confidence: {:.2}, priority: {}",
            feedback.average_score, confidence_score, priority
        );

        Ok(AdaptationResult {
            recommendations,
            confidence_score,
            priority: priority.to_string(),
        })
    }

    #[instrument(skip(self, patterns))]
    async fn analyze_patterns(
        &self,
        patterns: &[UsagePattern],
    ) -> LearningResult<PatternAnalysisResult> {
        debug!("Template adaptor analyzing {} patterns", patterns.len());

        let mut insights = Vec::new();
        let mut recommendations = Vec::new();

        // Analyze template usage patterns
        let template_patterns: Vec<&UsagePattern> = patterns
            .iter()
            .filter(|p| p.pattern_type.contains("template") || p.pattern_type.contains("prompt"))
            .collect();

        if !template_patterns.is_empty() {
            insights.push(format!(
                "Template usage patterns: {} active template variations identified",
                template_patterns.len()
            ));

            // Find most frequently used templates
            if let Some(most_used) = template_patterns.iter().max_by_key(|p| p.frequency) {
                insights.push(format!(
                    "Most popular template pattern: '{}' used {} times",
                    most_used.data, most_used.frequency
                ));
                recommendations.push(
                    "Analyze high-usage templates for successful patterns to replicate".to_string(),
                );
            }

            // Template diversity analysis
            if template_patterns.len() > 10 {
                recommendations.push(
                    "High template diversity - consider consolidating similar patterns".to_string(),
                );
            } else if template_patterns.len() < 3 {
                recommendations.push(
                    "Limited template diversity - consider developing specialized variations"
                        .to_string(),
                );
            }
        } else {
            insights
                .push("No template-specific usage patterns detected in current data".to_string());
            recommendations.push(
                "Implement template usage tracking for better adaptation insights".to_string(),
            );
        }

        let confidence_score = if template_patterns.len() > 5 {
            0.75
        } else {
            0.5
        };

        Ok(PatternAnalysisResult {
            insights,
            confidence_score,
            recommendations,
        })
    }

    fn get_config(&self) -> &AdaptationConfig {
        &self.config
    }
}

/// Factory for creating adaptation algorithms
pub struct AdaptationAlgorithmFactory;

impl AdaptationAlgorithmFactory {
    /// Create an algorithm by name
    pub fn create_algorithm(
        algorithm_name: &str,
        config: AdaptationConfig,
    ) -> LearningResult<Box<dyn AdaptationAlgorithm>> {
        match algorithm_name {
            "feedback_analyzer" => Ok(Box::new(FeedbackAnalyzer::new(config))),
            "pattern_matcher" => Ok(Box::new(PatternMatcher::new(config))),
            "prompt_optimizer" => Ok(Box::new(PromptOptimizer::new(config))),
            "query_optimizer" => Ok(Box::new(QueryOptimizer::new(config))),
            "template_adaptor" => Ok(Box::new(TemplateAdaptor::new(config))),
            _ => Err(LearningError::ConfigurationError(format!(
                "Unknown algorithm: {algorithm_name}"
            ))),
        }
    }

    /// Get list of available algorithms
    pub fn available_algorithms() -> Vec<&'static str> {
        vec![
            "feedback_analyzer",
            "pattern_matcher",
            "prompt_optimizer",
            "query_optimizer",
            "template_adaptor",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning::UsagePattern;
    use chrono::Utc;

    #[tokio::test]
    async fn test_feedback_analyzer_basic() {
        let config = AdaptationConfig::default();
        let analyzer = FeedbackAnalyzer::new(config);

        let feedback_data = FeedbackData {
            content_id: "test_content".to_string(),
            average_score: 0.75,
            feedback_count: 10,
            recent_trend: 0.05,
        };

        let result = analyzer.analyze_feedback(&feedback_data).await.unwrap();

        assert!(!result.recommendations.is_empty());
        assert!(result.confidence_score > 0.0);
        assert!(!result.priority.is_empty());
    }

    #[tokio::test]
    async fn test_pattern_matcher_basic() {
        let config = AdaptationConfig::default();
        let matcher = PatternMatcher::new(config);

        let patterns = vec![
            UsagePattern {
                id: "1".to_string(),
                pattern_type: "search".to_string(),
                data: "rust async".to_string(),
                frequency: 5,
                last_used: Utc::now(),
                context: HashMap::new(),
            },
            UsagePattern {
                id: "2".to_string(),
                pattern_type: "search".to_string(),
                data: "vector database".to_string(),
                frequency: 3,
                last_used: Utc::now(),
                context: HashMap::new(),
            },
        ];

        let result = matcher.analyze_patterns(&patterns).await.unwrap();

        assert!(!result.insights.is_empty());
        assert!(result.confidence_score > 0.0);
        assert!(!result.recommendations.is_empty());
    }

    #[test]
    fn test_algorithm_factory() {
        let config = AdaptationConfig::default();

        let analyzer =
            AdaptationAlgorithmFactory::create_algorithm("feedback_analyzer", config.clone());
        assert!(analyzer.is_ok());

        let matcher =
            AdaptationAlgorithmFactory::create_algorithm("pattern_matcher", config.clone());
        assert!(matcher.is_ok());

        let unknown = AdaptationAlgorithmFactory::create_algorithm("unknown", config);
        assert!(unknown.is_err());
    }

    #[test]
    fn test_available_algorithms() {
        let algorithms = AdaptationAlgorithmFactory::available_algorithms();
        assert!(algorithms.contains(&"feedback_analyzer"));
        assert!(algorithms.contains(&"pattern_matcher"));
        assert!(algorithms.contains(&"prompt_optimizer")); // New algorithm
        assert!(algorithms.contains(&"query_optimizer")); // New algorithm
        assert!(algorithms.contains(&"template_adaptor")); // New algorithm
        assert_eq!(algorithms.len(), 5);
    }

    #[tokio::test]
    async fn test_prompt_optimizer_feedback_analysis() {
        let config = AdaptationConfig::default();
        let optimizer = AdaptationAlgorithmFactory::create_algorithm("prompt_optimizer", config)
            .expect("Should create prompt optimizer");

        let feedback_data = FeedbackData {
            content_id: "prompt_v1".to_string(),
            average_score: 0.6, // Low score indicates optimization needed
            feedback_count: 15,
            recent_trend: -0.05, // Declining quality
        };

        let result = optimizer.analyze_feedback(&feedback_data).await.unwrap();

        assert!(!result.recommendations.is_empty());
        assert!(result.confidence_score > 0.0);
        assert_eq!(result.priority, "high"); // Low scores should be high priority

        // Should contain prompt optimization recommendations
        let has_prompt_recommendation = result
            .recommendations
            .iter()
            .any(|r| r.contains("prompt") || r.contains("template"));
        assert!(has_prompt_recommendation);
    }

    #[tokio::test]
    async fn test_query_optimizer_pattern_analysis() {
        let config = AdaptationConfig::default();
        let optimizer = AdaptationAlgorithmFactory::create_algorithm("query_optimizer", config)
            .expect("Should create query optimizer");

        let patterns = vec![
            UsagePattern {
                id: "1".to_string(),
                pattern_type: "successful_query".to_string(),
                data: "detailed technical implementation guide".to_string(),
                frequency: 10,
                last_used: Utc::now(),
                context: HashMap::new(),
            },
            UsagePattern {
                id: "2".to_string(),
                pattern_type: "failed_query".to_string(),
                data: "vague implementation".to_string(),
                frequency: 2,
                last_used: Utc::now(),
                context: HashMap::new(),
            },
        ];

        let result = optimizer.analyze_patterns(&patterns).await.unwrap();

        assert!(!result.insights.is_empty());
        assert!(result.confidence_score > 0.0);
        assert!(!result.recommendations.is_empty());

        // Should identify successful query patterns
        let has_success_insight = result
            .insights
            .iter()
            .any(|i| i.contains("successful") || i.contains("detailed"));
        assert!(has_success_insight);
    }

    #[tokio::test]
    async fn test_template_adaptor_optimization() {
        let config = AdaptationConfig::default();
        let adaptor = AdaptationAlgorithmFactory::create_algorithm("template_adaptor", config)
            .expect("Should create template adaptor");

        // Test with high-quality feedback for template improvement
        let feedback_data = FeedbackData {
            content_id: "template_decision_v2".to_string(),
            average_score: 0.95, // High score
            feedback_count: 25,
            recent_trend: 0.1, // Improving trend
        };

        let result = adaptor.analyze_feedback(&feedback_data).await.unwrap();

        assert!(!result.recommendations.is_empty());
        assert!(result.confidence_score > 0.8); // High confidence for good data
        assert_eq!(result.priority, "low"); // Good performance = low priority

        // Should recommend maintaining or enhancing successful patterns
        let has_maintain_recommendation = result
            .recommendations
            .iter()
            .any(|r| r.contains("maintain") || r.contains("enhance"));
        assert!(has_maintain_recommendation);
    }

    #[tokio::test]
    async fn test_prompt_optimization_integration() {
        let config = AdaptationConfig::default();
        let prompt_optimizer =
            AdaptationAlgorithmFactory::create_algorithm("prompt_optimizer", config.clone())
                .expect("Should create prompt optimizer");
        let template_adaptor =
            AdaptationAlgorithmFactory::create_algorithm("template_adaptor", config)
                .expect("Should create template adaptor");

        // Test coordinated optimization across both algorithms
        let feedback_data = FeedbackData {
            content_id: "research_prompt_complex".to_string(),
            average_score: 0.7,
            feedback_count: 20,
            recent_trend: 0.0, // Stable but suboptimal
        };

        let prompt_result = prompt_optimizer
            .analyze_feedback(&feedback_data)
            .await
            .unwrap();
        let template_result = template_adaptor
            .analyze_feedback(&feedback_data)
            .await
            .unwrap();

        // Both should provide optimization recommendations
        assert!(!prompt_result.recommendations.is_empty());
        assert!(!template_result.recommendations.is_empty());

        // Results should have reasonable confidence
        assert!(prompt_result.confidence_score > 0.5);
        assert!(template_result.confidence_score > 0.5);
    }
}
