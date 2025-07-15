// ABOUTME: Integrated gap analyzer combining standard gap detection with semantic analysis
//! This module provides an integrated analyzer that combines the gap detection from Task 1.2
//! with the semantic analysis capabilities from Task 1.3, creating a complete pipeline
//! for enhanced knowledge gap detection and analysis.

use crate::proactive::{
    DetectedGap, FileEvent, GapAnalysisConfig, GapAnalysisError, GapAnalyzer,
    SemanticAnalysisConfig, SemanticAnalysisError, SemanticGapAnalysis, SemanticGapAnalyzer,
};
use fortitude_core::vector::SemanticSearchOperations;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use tokio::time::Duration;
use tracing::{debug, info, instrument, warn};

/// Errors that can occur during integrated analysis
#[derive(Error, Debug)]
pub enum IntegratedAnalysisError {
    #[error("Gap analysis error: {0}")]
    GapAnalysis(#[from] GapAnalysisError),

    #[error("Semantic analysis error: {0}")]
    SemanticAnalysis(#[from] SemanticAnalysisError),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error(
        "Performance threshold exceeded: total analysis took {duration:?}, limit is {limit:?}"
    )]
    PerformanceThreshold { duration: Duration, limit: Duration },
}

/// Configuration for the integrated analyzer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegratedAnalysisConfig {
    /// Gap analysis configuration
    pub gap_analysis: GapAnalysisConfig,
    /// Semantic analysis configuration
    pub semantic_analysis: SemanticAnalysisConfig,
    /// Maximum total analysis time (milliseconds)
    pub max_total_time_ms: u64,
    /// Enable semantic analysis (can be disabled for performance)
    pub enable_semantic_analysis: bool,
    /// Minimum gaps required to enable semantic analysis
    pub min_gaps_for_semantic: usize,
}

impl Default for IntegratedAnalysisConfig {
    fn default() -> Self {
        Self {
            gap_analysis: GapAnalysisConfig::for_rust_project(),
            semantic_analysis: SemanticAnalysisConfig::default(),
            max_total_time_ms: 500, // 500ms total requirement
            enable_semantic_analysis: true,
            min_gaps_for_semantic: 1,
        }
    }
}

impl IntegratedAnalysisConfig {
    /// Create performance-optimized configuration
    pub fn for_performance() -> Self {
        Self {
            gap_analysis: GapAnalysisConfig::for_rust_project().with_timeout_ms(200),
            semantic_analysis: SemanticAnalysisConfig::for_performance(),
            max_total_time_ms: 300,
            enable_semantic_analysis: true,
            min_gaps_for_semantic: 2, // Only run semantic analysis if at least 2 gaps
        }
    }

    /// Create accuracy-optimized configuration
    pub fn for_accuracy() -> Self {
        Self {
            gap_analysis: GapAnalysisConfig::for_rust_project(),
            semantic_analysis: SemanticAnalysisConfig::for_accuracy(),
            max_total_time_ms: 1000,
            enable_semantic_analysis: true,
            min_gaps_for_semantic: 1, // Always run semantic analysis
        }
    }
}

/// Result of integrated gap analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegratedAnalysisResult {
    /// Standard gap detection results
    pub detected_gaps: Vec<DetectedGap>,
    /// Semantic analysis results (if enabled and performed)
    pub semantic_analysis: Option<Vec<SemanticGapAnalysis>>,
    /// Analysis performance metrics
    pub performance_metrics: AnalysisPerformanceMetrics,
    /// Whether semantic analysis was performed
    pub semantic_analysis_performed: bool,
}

/// Performance metrics for the integrated analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPerformanceMetrics {
    /// Total analysis time (milliseconds)
    pub total_time_ms: f64,
    /// Gap detection time (milliseconds)
    pub gap_detection_time_ms: f64,
    /// Semantic analysis time (milliseconds)
    pub semantic_analysis_time_ms: f64,
    /// Number of gaps detected
    pub gaps_detected: usize,
    /// Number of gaps with semantic enhancement
    pub gaps_semantically_enhanced: usize,
    /// Total vector queries performed
    pub total_vector_queries: usize,
}

/// Integrated analyzer that combines gap detection with semantic analysis
pub struct IntegratedGapAnalyzer {
    /// Standard gap analyzer
    gap_analyzer: GapAnalyzer,
    /// Semantic gap analyzer (optional)
    semantic_analyzer: Option<SemanticGapAnalyzer>,
    /// Configuration
    config: IntegratedAnalysisConfig,
}

impl IntegratedGapAnalyzer {
    /// Create a new integrated analyzer with semantic capabilities
    pub fn new(
        config: IntegratedAnalysisConfig,
        search_service: Option<Arc<dyn SemanticSearchOperations>>,
    ) -> Result<Self, IntegratedAnalysisError> {
        let gap_analyzer = GapAnalyzer::new(config.gap_analysis.clone())?;

        let semantic_analyzer = if config.enable_semantic_analysis {
            if let Some(service) = search_service {
                Some(SemanticGapAnalyzer::new(
                    service,
                    config.semantic_analysis.clone(),
                ))
            } else {
                warn!("Semantic analysis enabled but no search service provided");
                None
            }
        } else {
            None
        };

        Ok(Self {
            gap_analyzer,
            semantic_analyzer,
            config,
        })
    }

    /// Create analyzer with only gap detection (no semantic analysis)
    pub fn gap_detection_only(config: GapAnalysisConfig) -> Result<Self, IntegratedAnalysisError> {
        let integrated_config = IntegratedAnalysisConfig {
            gap_analysis: config,
            enable_semantic_analysis: false,
            ..Default::default()
        };

        Self::new(integrated_config, None)
    }

    /// Analyze a file with integrated gap detection and semantic analysis
    #[instrument(skip(self, file_path))]
    pub async fn analyze_file(
        &self,
        file_path: &Path,
    ) -> Result<IntegratedAnalysisResult, IntegratedAnalysisError> {
        let total_start = Instant::now();

        // Step 1: Standard gap detection
        let gap_start = Instant::now();
        let detected_gaps = self.gap_analyzer.analyze_file(file_path).await?;
        let gap_detection_time = gap_start.elapsed().as_millis() as f64;

        debug!(
            "Gap detection completed: {} gaps found in {:.1}ms",
            detected_gaps.len(),
            gap_detection_time
        );

        // Step 2: Semantic analysis (if enabled and conditions met)
        let (semantic_analysis, semantic_time, vector_queries) =
            if self.should_perform_semantic_analysis(&detected_gaps) {
                let semantic_start = Instant::now();

                match &self.semantic_analyzer {
                    Some(analyzer) => {
                        match analyzer
                            .analyze_gaps_semantically(detected_gaps.clone())
                            .await
                        {
                            Ok(results) => {
                                let time = semantic_start.elapsed().as_millis() as f64;
                                let queries = results
                                    .iter()
                                    .map(|r| r.metadata.vector_queries_count)
                                    .sum();
                                (Some(results), time, queries)
                            }
                            Err(e) => {
                                warn!("Semantic analysis failed: {}", e);
                                (None, 0.0, 0)
                            }
                        }
                    }
                    None => (None, 0.0, 0),
                }
            } else {
                (None, 0.0, 0)
            };

        let total_time = total_start.elapsed().as_millis() as f64;

        // Check performance requirements
        let total_limit = Duration::from_millis(self.config.max_total_time_ms);
        if total_start.elapsed() > total_limit {
            warn!(
                "Integrated analysis exceeded time limit: {:.1}ms > {}ms",
                total_time, self.config.max_total_time_ms
            );
        }

        let semantic_analysis_performed = semantic_analysis.is_some();
        let gaps_semantically_enhanced = semantic_analysis
            .as_ref()
            .map(|analysis| analysis.len())
            .unwrap_or(0);

        let gaps_detected_count = detected_gaps.len();

        let result = IntegratedAnalysisResult {
            detected_gaps,
            semantic_analysis,
            performance_metrics: AnalysisPerformanceMetrics {
                total_time_ms: total_time,
                gap_detection_time_ms: gap_detection_time,
                semantic_analysis_time_ms: semantic_time,
                gaps_detected: gaps_detected_count,
                gaps_semantically_enhanced,
                total_vector_queries: vector_queries,
            },
            semantic_analysis_performed,
        };

        info!(
            "Integrated analysis completed: {} gaps, semantic={}, total_time={:.1}ms",
            result.detected_gaps.len(),
            semantic_analysis_performed,
            total_time
        );

        Ok(result)
    }

    /// Analyze a file event with integrated analysis
    #[instrument(skip(self, event))]
    pub async fn analyze_file_event(
        &self,
        event: &FileEvent,
    ) -> Result<IntegratedAnalysisResult, IntegratedAnalysisError> {
        if !event.should_trigger_analysis {
            return Ok(IntegratedAnalysisResult {
                detected_gaps: Vec::new(),
                semantic_analysis: None,
                performance_metrics: AnalysisPerformanceMetrics {
                    total_time_ms: 0.0,
                    gap_detection_time_ms: 0.0,
                    semantic_analysis_time_ms: 0.0,
                    gaps_detected: 0,
                    gaps_semantically_enhanced: 0,
                    total_vector_queries: 0,
                },
                semantic_analysis_performed: false,
            });
        }

        self.analyze_file(&event.path).await
    }

    /// Determine if semantic analysis should be performed
    fn should_perform_semantic_analysis(&self, gaps: &[DetectedGap]) -> bool {
        self.config.enable_semantic_analysis
            && self.semantic_analyzer.is_some()
            && gaps.len() >= self.config.min_gaps_for_semantic
    }

    /// Get the enhanced gaps (with semantic analysis if available)
    pub fn get_enhanced_gaps(&self, result: &IntegratedAnalysisResult) -> Vec<EnhancedGap> {
        if let Some(semantic_analysis) = &result.semantic_analysis {
            // Return semantic analysis results as enhanced gaps
            semantic_analysis
                .iter()
                .map(|analysis| EnhancedGap {
                    gap: analysis.gap.clone(),
                    is_validated: Some(analysis.is_validated),
                    validation_confidence: Some(analysis.validation_confidence),
                    enhanced_priority: Some(analysis.enhanced_priority),
                    related_documents_count: analysis.related_documents.len(),
                    has_semantic_enhancement: true,
                })
                .collect()
        } else {
            // Return standard gaps without semantic enhancement
            result
                .detected_gaps
                .iter()
                .map(|gap| EnhancedGap {
                    gap: gap.clone(),
                    is_validated: None,
                    validation_confidence: None,
                    enhanced_priority: None,
                    related_documents_count: 0,
                    has_semantic_enhancement: false,
                })
                .collect()
        }
    }
}

/// Enhanced gap with optional semantic analysis information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedGap {
    /// The detected gap
    pub gap: DetectedGap,
    /// Whether the gap is validated (from semantic analysis)
    pub is_validated: Option<bool>,
    /// Validation confidence (from semantic analysis)
    pub validation_confidence: Option<f64>,
    /// Enhanced priority (from semantic analysis)
    pub enhanced_priority: Option<u8>,
    /// Number of related documents found
    pub related_documents_count: usize,
    /// Whether this gap has semantic enhancement
    pub has_semantic_enhancement: bool,
}

impl EnhancedGap {
    /// Get the effective priority (enhanced if available, otherwise original)
    pub fn effective_priority(&self) -> u8 {
        self.enhanced_priority.unwrap_or(self.gap.priority)
    }

    /// Check if the gap should be acted upon (validated or no semantic analysis)
    pub fn should_act_upon(&self) -> bool {
        self.is_validated.unwrap_or(true) // Default to true if no semantic analysis
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proactive::{EventType, FileEvent, GapType};
    use std::path::PathBuf;
    use tempfile::TempDir;
    use tokio::fs;

    #[test]
    fn test_integrated_config_default() {
        let config = IntegratedAnalysisConfig::default();
        assert!(config.enable_semantic_analysis);
        assert_eq!(config.max_total_time_ms, 500);
        assert_eq!(config.min_gaps_for_semantic, 1);
    }

    #[test]
    fn test_integrated_config_performance() {
        let config = IntegratedAnalysisConfig::for_performance();
        assert_eq!(config.max_total_time_ms, 300);
        assert_eq!(config.min_gaps_for_semantic, 2);
    }

    #[test]
    fn test_integrated_config_accuracy() {
        let config = IntegratedAnalysisConfig::for_accuracy();
        assert_eq!(config.max_total_time_ms, 1000);
        assert_eq!(config.min_gaps_for_semantic, 1);
    }

    #[tokio::test]
    async fn test_gap_detection_only_analyzer() {
        // FAILING TEST: Should work without semantic analysis
        let gap_config = GapAnalysisConfig::for_rust_project();
        let analyzer = IntegratedGapAnalyzer::gap_detection_only(gap_config).unwrap();

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let content = "// TODO: Test comment\npub fn test() {}";
        fs::write(&test_file, content).await.unwrap();

        let result = analyzer.analyze_file(&test_file).await.unwrap();

        assert!(!result.detected_gaps.is_empty());
        assert!(result.semantic_analysis.is_none());
        assert!(!result.semantic_analysis_performed);
        assert_eq!(result.performance_metrics.semantic_analysis_time_ms, 0.0);
    }

    #[tokio::test]
    async fn test_enhanced_gap_effective_priority() {
        // FAILING TEST: Should return enhanced priority when available
        let original_gap = DetectedGap::new(
            GapType::TodoComment,
            PathBuf::from("test.rs"),
            1,
            "// TODO: test".to_string(),
            "test".to_string(),
            0.9,
        );

        // Gap without enhancement
        let basic_gap = EnhancedGap {
            gap: original_gap.clone(),
            is_validated: None,
            validation_confidence: None,
            enhanced_priority: None,
            related_documents_count: 0,
            has_semantic_enhancement: false,
        };

        assert_eq!(basic_gap.effective_priority(), original_gap.priority);
        assert!(basic_gap.should_act_upon());

        // Gap with enhancement
        let enhanced_gap = EnhancedGap {
            gap: original_gap.clone(),
            is_validated: Some(true),
            validation_confidence: Some(0.9),
            enhanced_priority: Some(9),
            related_documents_count: 2,
            has_semantic_enhancement: true,
        };

        assert_eq!(enhanced_gap.effective_priority(), 9);
        assert!(enhanced_gap.should_act_upon());

        // Gap invalidated by semantic analysis
        let invalidated_gap = EnhancedGap {
            gap: original_gap,
            is_validated: Some(false),
            validation_confidence: Some(0.3),
            enhanced_priority: Some(5),
            related_documents_count: 1,
            has_semantic_enhancement: true,
        };

        assert!(!invalidated_gap.should_act_upon());
    }

    #[tokio::test]
    async fn test_should_perform_semantic_analysis() {
        // FAILING TEST: Should correctly determine when to perform semantic analysis
        let config = IntegratedAnalysisConfig {
            enable_semantic_analysis: true,
            min_gaps_for_semantic: 2,
            ..Default::default()
        };

        let analyzer =
            IntegratedGapAnalyzer::gap_detection_only(config.gap_analysis.clone()).unwrap();

        // Test with insufficient gaps
        let few_gaps = vec![DetectedGap::new(
            GapType::TodoComment,
            PathBuf::from("test.rs"),
            1,
            "// TODO: one".to_string(),
            "one".to_string(),
            0.9,
        )];

        assert!(!analyzer.should_perform_semantic_analysis(&few_gaps));

        // Test with sufficient gaps
        let many_gaps = vec![
            DetectedGap::new(
                GapType::TodoComment,
                PathBuf::from("test.rs"),
                1,
                "// TODO: one".to_string(),
                "one".to_string(),
                0.9,
            ),
            DetectedGap::new(
                GapType::MissingDocumentation,
                PathBuf::from("test.rs"),
                2,
                "pub fn test()".to_string(),
                "missing docs".to_string(),
                0.8,
            ),
        ];

        // Would return true if semantic analyzer was available
        // Current analyzer has none, so returns false
        assert!(!analyzer.should_perform_semantic_analysis(&many_gaps));
    }

    #[tokio::test]
    async fn test_file_event_analysis() {
        // FAILING TEST: Should handle file events correctly
        let analyzer =
            IntegratedGapAnalyzer::gap_detection_only(GapAnalysisConfig::for_rust_project())
                .unwrap();

        // Event that should not trigger analysis
        let no_analysis_event = FileEvent::new(PathBuf::from("test.txt"), EventType::Create);
        // Manually set should_trigger_analysis to false
        let mut no_analysis_event = no_analysis_event;
        no_analysis_event.should_trigger_analysis = false;

        let result = analyzer
            .analyze_file_event(&no_analysis_event)
            .await
            .unwrap();
        assert!(result.detected_gaps.is_empty());
        assert_eq!(result.performance_metrics.total_time_ms, 0.0);

        // Event that should trigger analysis
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        let content = "// TODO: Test analysis\npub fn test() {}";
        fs::write(&test_file, content).await.unwrap();

        let analysis_event = FileEvent::new(test_file, EventType::Write);
        let result = analyzer.analyze_file_event(&analysis_event).await.unwrap();

        assert!(!result.detected_gaps.is_empty());
        assert!(result.performance_metrics.total_time_ms >= 0.0); // Allow 0.0 for very fast analysis
    }

    #[test]
    fn test_performance_metrics() {
        let metrics = AnalysisPerformanceMetrics {
            total_time_ms: 150.0,
            gap_detection_time_ms: 100.0,
            semantic_analysis_time_ms: 50.0,
            gaps_detected: 5,
            gaps_semantically_enhanced: 5,
            total_vector_queries: 10,
        };

        assert_eq!(metrics.total_time_ms, 150.0);
        assert_eq!(metrics.gaps_detected, 5);
        assert_eq!(metrics.total_vector_queries, 10);
    }
}
