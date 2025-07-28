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

// ABOUTME: Main context detection engine for audience level, technical domain, and urgency classification
use crate::classification::rules::{AudienceRules, DomainRules, UrgencyRules};
use fortitude_types::{
    classification_result::{
        AudienceLevel, ClassificationDimension, DimensionConfidence, TechnicalDomain, UrgencyLevel,
    },
    ClassificationError, ResearchType,
};
use std::time::Instant;
use tracing::{debug, info, warn};

/// Trait for context detection in different dimensions
pub trait ContextDetector {
    /// Detect audience level with confidence scoring
    fn detect_audience_level(
        &self,
        query: &str,
    ) -> Result<(AudienceLevel, f64, Vec<String>), ClassificationError>;

    /// Detect technical domain with confidence scoring
    fn detect_technical_domain(
        &self,
        query: &str,
    ) -> Result<(TechnicalDomain, f64, Vec<String>), ClassificationError>;

    /// Detect urgency level with confidence scoring
    fn detect_urgency_level(
        &self,
        query: &str,
    ) -> Result<(UrgencyLevel, f64, Vec<String>), ClassificationError>;

    /// Perform comprehensive context detection across all dimensions
    fn detect_context(
        &self,
        query: &str,
        research_type: &ResearchType,
    ) -> Result<ContextDetectionResult, ClassificationError>;
}

/// Configuration for context detection
#[derive(Debug, Clone)]
pub struct ContextDetectionConfig {
    /// Minimum confidence threshold for each dimension
    pub confidence_threshold: f64,
    /// Whether to use fallback mechanisms for low confidence
    pub enable_fallback: bool,
    /// Maximum processing time in milliseconds
    pub max_processing_time_ms: u64,
    /// Enable debug logging
    pub debug_logging: bool,
}

impl Default for ContextDetectionConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.6,
            enable_fallback: true,
            max_processing_time_ms: 50,
            debug_logging: false,
        }
    }
}

/// Result of context detection across all dimensions
#[derive(Debug, Clone)]
pub struct ContextDetectionResult {
    /// Detected audience level
    pub audience_level: AudienceLevel,
    /// Detected technical domain
    pub technical_domain: TechnicalDomain,
    /// Detected urgency level
    pub urgency_level: UrgencyLevel,
    /// Confidence scores for each dimension
    pub dimension_confidences: Vec<DimensionConfidence>,
    /// Overall confidence score
    pub overall_confidence: f64,
    /// Total processing time in milliseconds
    pub processing_time_ms: u64,
    /// Whether any fallback was used
    pub fallback_used: bool,
}

impl ContextDetectionResult {
    /// Create a new context detection result
    pub fn new(
        audience_level: AudienceLevel,
        technical_domain: TechnicalDomain,
        urgency_level: UrgencyLevel,
        dimension_confidences: Vec<DimensionConfidence>,
        processing_time_ms: u64,
        fallback_used: bool,
    ) -> Self {
        // Calculate overall confidence as the average of all dimension confidences
        let overall_confidence = if dimension_confidences.is_empty() {
            0.0
        } else {
            dimension_confidences
                .iter()
                .map(|dc| dc.confidence)
                .sum::<f64>()
                / dimension_confidences.len() as f64
        };

        Self {
            audience_level,
            technical_domain,
            urgency_level,
            dimension_confidences,
            overall_confidence,
            processing_time_ms,
            fallback_used,
        }
    }

    /// Get confidence for a specific dimension
    pub fn get_dimension_confidence(&self, dimension: &ClassificationDimension) -> Option<f64> {
        self.dimension_confidences
            .iter()
            .find(|dc| dc.dimension == *dimension)
            .map(|dc| dc.confidence)
    }

    /// Check if all dimensions meet the confidence threshold
    pub fn meets_confidence_threshold(&self, threshold: f64) -> bool {
        self.dimension_confidences
            .iter()
            .all(|dc| dc.confidence >= threshold)
    }

    /// Get the least confident dimension
    pub fn least_confident_dimension(&self) -> Option<&DimensionConfidence> {
        self.dimension_confidences.iter().min_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

/// Main context detection engine
pub struct FortitudeContextDetector {
    /// Configuration for context detection
    config: ContextDetectionConfig,
    /// Audience level detection rules
    audience_rules: AudienceRules,
    /// Technical domain detection rules
    domain_rules: DomainRules,
    /// Urgency level detection rules
    urgency_rules: UrgencyRules,
}

impl FortitudeContextDetector {
    /// Create a new context detector with default configuration
    pub fn new() -> Self {
        Self {
            config: ContextDetectionConfig::default(),
            audience_rules: AudienceRules::new(),
            domain_rules: DomainRules::new(),
            urgency_rules: UrgencyRules::new(),
        }
    }

    /// Create a new context detector with custom configuration
    pub fn with_config(config: ContextDetectionConfig) -> Self {
        Self {
            config,
            audience_rules: AudienceRules::new(),
            domain_rules: DomainRules::new(),
            urgency_rules: UrgencyRules::new(),
        }
    }

    /// Apply fallback mechanism for low confidence results
    fn apply_fallback(
        &self,
        audience_result: Option<(AudienceLevel, f64, Vec<String>)>,
        domain_result: Option<(TechnicalDomain, f64, Vec<String>)>,
        urgency_result: Option<(UrgencyLevel, f64, Vec<String>)>,
        query: &str,
        research_type: &ResearchType,
    ) -> (AudienceLevel, TechnicalDomain, UrgencyLevel, bool) {
        if self.config.debug_logging {
            debug!("Applying fallback mechanisms for query: '{}'", query);
        }

        let mut fallback_used = false;

        // Fallback for audience level
        let audience_level = if let Some((level, confidence, _)) = audience_result {
            if confidence >= self.config.confidence_threshold {
                level
            } else {
                fallback_used = true;
                // Use research type to infer audience level
                match research_type {
                    ResearchType::Learning => AudienceLevel::Beginner,
                    ResearchType::Troubleshooting => AudienceLevel::Intermediate,
                    ResearchType::Implementation => AudienceLevel::Intermediate,
                    ResearchType::Decision => AudienceLevel::Advanced,
                    ResearchType::Validation => AudienceLevel::Advanced,
                }
            }
        } else {
            fallback_used = true;
            AudienceLevel::default()
        };

        // Fallback for technical domain
        let technical_domain = if let Some((domain, confidence, _)) = domain_result {
            if confidence >= self.config.confidence_threshold {
                domain
            } else {
                fallback_used = true;
                TechnicalDomain::default()
            }
        } else {
            fallback_used = true;
            TechnicalDomain::default()
        };

        // Fallback for urgency level
        let urgency_level = if let Some((level, confidence, _)) = urgency_result {
            if confidence >= self.config.confidence_threshold {
                level
            } else {
                fallback_used = true;
                // Use research type to infer urgency
                match research_type {
                    ResearchType::Troubleshooting => UrgencyLevel::Immediate,
                    ResearchType::Implementation => UrgencyLevel::Planned,
                    ResearchType::Decision => UrgencyLevel::Planned,
                    ResearchType::Learning => UrgencyLevel::Exploratory,
                    ResearchType::Validation => UrgencyLevel::Planned,
                }
            }
        } else {
            fallback_used = true;
            UrgencyLevel::default()
        };

        (
            audience_level,
            technical_domain,
            urgency_level,
            fallback_used,
        )
    }
}

impl Default for FortitudeContextDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextDetector for FortitudeContextDetector {
    fn detect_audience_level(
        &self,
        query: &str,
    ) -> Result<(AudienceLevel, f64, Vec<String>), ClassificationError> {
        if query.trim().is_empty() {
            return Err(ClassificationError::InvalidInput(
                "Query cannot be empty".to_string(),
            ));
        }

        self.audience_rules.detect_audience_level(query)
    }

    fn detect_technical_domain(
        &self,
        query: &str,
    ) -> Result<(TechnicalDomain, f64, Vec<String>), ClassificationError> {
        if query.trim().is_empty() {
            return Err(ClassificationError::InvalidInput(
                "Query cannot be empty".to_string(),
            ));
        }

        self.domain_rules.detect_technical_domain(query)
    }

    fn detect_urgency_level(
        &self,
        query: &str,
    ) -> Result<(UrgencyLevel, f64, Vec<String>), ClassificationError> {
        if query.trim().is_empty() {
            return Err(ClassificationError::InvalidInput(
                "Query cannot be empty".to_string(),
            ));
        }

        self.urgency_rules.detect_urgency_level(query)
    }

    fn detect_context(
        &self,
        query: &str,
        research_type: &ResearchType,
    ) -> Result<ContextDetectionResult, ClassificationError> {
        let start_time = Instant::now();

        if self.config.debug_logging {
            debug!("Starting context detection for query: '{}'", query);
        }

        if query.trim().is_empty() {
            return Err(ClassificationError::InvalidInput(
                "Query cannot be empty".to_string(),
            ));
        }

        // Perform parallel detection across all dimensions
        let audience_result = self.detect_audience_level(query).ok();
        let domain_result = self.detect_technical_domain(query).ok();
        let urgency_result = self.detect_urgency_level(query).ok();

        // Apply fallback mechanisms if enabled
        let (audience_level, technical_domain, urgency_level, fallback_used) =
            if self.config.enable_fallback {
                self.apply_fallback(
                    audience_result.clone(),
                    domain_result.clone(),
                    urgency_result.clone(),
                    query,
                    research_type,
                )
            } else {
                (
                    audience_result
                        .as_ref()
                        .map(|(level, _, _)| level.clone())
                        .unwrap_or_default(),
                    domain_result
                        .as_ref()
                        .map(|(domain, _, _)| domain.clone())
                        .unwrap_or_default(),
                    urgency_result
                        .as_ref()
                        .map(|(level, _, _)| level.clone())
                        .unwrap_or_default(),
                    false,
                )
            };

        // Build dimension confidences
        let mut dimension_confidences = Vec::new();

        if let Some((_, confidence, keywords)) = audience_result {
            dimension_confidences.push(DimensionConfidence::new(
                ClassificationDimension::AudienceLevel,
                confidence,
                keywords,
                format!("Detected {} audience level", audience_level.display_name()),
            ));
        }

        if let Some((_, confidence, keywords)) = domain_result {
            dimension_confidences.push(DimensionConfidence::new(
                ClassificationDimension::TechnicalDomain,
                confidence,
                keywords,
                format!(
                    "Detected {} technical domain",
                    technical_domain.display_name()
                ),
            ));
        }

        if let Some((_, confidence, keywords)) = urgency_result {
            dimension_confidences.push(DimensionConfidence::new(
                ClassificationDimension::Urgency,
                confidence,
                keywords,
                format!("Detected {} urgency level", urgency_level.display_name()),
            ));
        }

        let processing_time = start_time.elapsed().as_millis() as u64;

        // Check processing time limit
        if processing_time > self.config.max_processing_time_ms {
            warn!(
                "Context detection took {}ms, exceeding limit of {}ms",
                processing_time, self.config.max_processing_time_ms
            );
        }

        let result = ContextDetectionResult::new(
            audience_level,
            technical_domain,
            urgency_level,
            dimension_confidences,
            processing_time,
            fallback_used,
        );

        if self.config.debug_logging {
            info!(
                "Context detection completed in {}ms: audience={}, domain={}, urgency={}, confidence={:.2}",
                processing_time,
                result.audience_level.display_name(),
                result.technical_domain.display_name(),
                result.urgency_level.display_name(),
                result.overall_confidence
            );
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_detector() -> FortitudeContextDetector {
        let config = ContextDetectionConfig {
            confidence_threshold: 0.5, // Higher threshold to trigger fallback
            enable_fallback: true,
            max_processing_time_ms: 100,
            debug_logging: false,
        };
        FortitudeContextDetector::with_config(config)
    }

    #[test]
    fn test_context_detector_creation() {
        let detector = FortitudeContextDetector::new();
        assert_eq!(detector.config.confidence_threshold, 0.6);
        assert!(detector.config.enable_fallback);

        let detector = FortitudeContextDetector::default();
        assert_eq!(detector.config.confidence_threshold, 0.6);
    }

    #[test]
    fn test_context_detection_config() {
        let config = ContextDetectionConfig {
            confidence_threshold: 0.8,
            enable_fallback: false,
            max_processing_time_ms: 200,
            debug_logging: true,
        };

        let detector = FortitudeContextDetector::with_config(config.clone());
        assert_eq!(detector.config.confidence_threshold, 0.8);
        assert!(!detector.config.enable_fallback);
        assert_eq!(detector.config.max_processing_time_ms, 200);
        assert!(detector.config.debug_logging);
    }

    #[test]
    fn test_context_detection_result_creation() {
        let dimension_confidences = vec![
            DimensionConfidence::new(
                ClassificationDimension::AudienceLevel,
                0.8,
                vec!["beginner".to_string()],
                "Beginner level detected".to_string(),
            ),
            DimensionConfidence::new(
                ClassificationDimension::TechnicalDomain,
                0.9,
                vec!["rust".to_string()],
                "Rust domain detected".to_string(),
            ),
        ];

        let result = ContextDetectionResult::new(
            AudienceLevel::Beginner,
            TechnicalDomain::Rust,
            UrgencyLevel::Planned,
            dimension_confidences,
            25,
            false,
        );

        assert_eq!(result.audience_level, AudienceLevel::Beginner);
        assert_eq!(result.technical_domain, TechnicalDomain::Rust);
        assert_eq!(result.urgency_level, UrgencyLevel::Planned);
        assert!((result.overall_confidence - 0.85).abs() < 0.01); // Average of 0.8 and 0.9
        assert_eq!(result.processing_time_ms, 25);
        assert!(!result.fallback_used);
    }

    #[test]
    fn test_context_detection_result_accessors() {
        let dimension_confidences = vec![
            DimensionConfidence::new(
                ClassificationDimension::AudienceLevel,
                0.6,
                vec!["intermediate".to_string()],
                "Intermediate level detected".to_string(),
            ),
            DimensionConfidence::new(
                ClassificationDimension::TechnicalDomain,
                0.9,
                vec!["web".to_string()],
                "Web domain detected".to_string(),
            ),
        ];

        let result = ContextDetectionResult::new(
            AudienceLevel::Intermediate,
            TechnicalDomain::Web,
            UrgencyLevel::Immediate,
            dimension_confidences,
            30,
            true,
        );

        assert_eq!(
            result.get_dimension_confidence(&ClassificationDimension::AudienceLevel),
            Some(0.6)
        );
        assert_eq!(
            result.get_dimension_confidence(&ClassificationDimension::TechnicalDomain),
            Some(0.9)
        );
        assert_eq!(
            result.get_dimension_confidence(&ClassificationDimension::Urgency),
            None
        );

        assert!(result.meets_confidence_threshold(0.5));
        assert!(!result.meets_confidence_threshold(0.8));

        let least_confident = result.least_confident_dimension().unwrap();
        assert_eq!(
            least_confident.dimension,
            ClassificationDimension::AudienceLevel
        );
        assert_eq!(least_confident.confidence, 0.6);
    }

    #[test]
    fn test_empty_query_handling() {
        let detector = create_test_detector();

        let result = detector.detect_audience_level("");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ClassificationError::InvalidInput(_)
        ));

        let result = detector.detect_technical_domain("");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ClassificationError::InvalidInput(_)
        ));

        let result = detector.detect_urgency_level("");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ClassificationError::InvalidInput(_)
        ));

        let result = detector.detect_context("", &ResearchType::Learning);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ClassificationError::InvalidInput(_)
        ));
    }

    #[test]
    fn test_fallback_mechanisms() {
        let detector = create_test_detector();

        // Test with a query that should trigger fallback
        let result = detector.detect_context("zzz111", &ResearchType::Learning);
        assert!(result.is_ok());

        let context = result.unwrap();
        // Should use fallback values - check that fallback was used
        assert!(context.fallback_used);
        assert_eq!(context.audience_level, AudienceLevel::Beginner); // Learning -> Beginner
        assert_eq!(context.technical_domain, TechnicalDomain::General); // Default
        assert_eq!(context.urgency_level, UrgencyLevel::Exploratory); // Learning -> Exploratory
    }

    #[test]
    fn test_research_type_based_fallback() {
        let detector = create_test_detector();

        // Test different research types with fallback
        let troubleshooting_result =
            detector.detect_context("zzz111", &ResearchType::Troubleshooting);
        assert!(troubleshooting_result.is_ok());
        let context = troubleshooting_result.unwrap();
        assert!(context.fallback_used);
        assert_eq!(context.audience_level, AudienceLevel::Intermediate);
        assert_eq!(context.urgency_level, UrgencyLevel::Immediate);

        let decision_result = detector.detect_context("zzz111", &ResearchType::Decision);
        assert!(decision_result.is_ok());
        let context = decision_result.unwrap();
        assert!(context.fallback_used);
        assert_eq!(context.audience_level, AudienceLevel::Advanced);
        assert_eq!(context.urgency_level, UrgencyLevel::Planned);
    }
}
