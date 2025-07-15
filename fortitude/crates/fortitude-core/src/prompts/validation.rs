// ABOUTME: Template validation system for progressive disclosure and quality assurance
use fortitude_types::research::{ResearchResult, ResearchType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during validation
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Completion criteria not met: {0}")]
    CompletionCriteriaMissing(String),
    #[error("Format validation failed: {0}")]
    FormatValidationFailed(String),
    #[error("Quality threshold not met: expected {expected}, got {actual}")]
    QualityThresholdNotMet { expected: f64, actual: f64 },
    #[error("Semantic markup validation failed: {0}")]
    SemanticMarkupInvalid(String),
    #[error("Progressive disclosure structure invalid: {0}")]
    ProgressiveDisclosureInvalid(String),
}

/// Completion criteria for different research types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionCriteria {
    /// Required sections that must be present
    pub required_sections: Vec<String>,
    /// Minimum word count for each section
    pub min_word_counts: HashMap<String, usize>,
    /// Maximum word count for each section
    pub max_word_counts: HashMap<String, usize>,
    /// Required semantic markup tags
    pub required_tags: Vec<String>,
    /// Quality threshold (0.0-1.0)
    pub quality_threshold: f64,
    /// Progressive disclosure layers that must be present
    pub required_layers: Vec<String>,
}

impl CompletionCriteria {
    /// Create completion criteria for Decision research
    pub fn for_decision() -> Self {
        let mut min_word_counts = HashMap::new();
        min_word_counts.insert("problem_statement".to_string(), 20);
        min_word_counts.insert("options".to_string(), 30);
        min_word_counts.insert("recommendation".to_string(), 50);

        let mut max_word_counts = HashMap::new();
        max_word_counts.insert("problem_statement".to_string(), 200);
        max_word_counts.insert("options".to_string(), 300);
        max_word_counts.insert("recommendation".to_string(), 400);

        Self {
            required_sections: vec![
                "problem_statement".to_string(),
                "options".to_string(),
                "recommendation".to_string(),
            ],
            min_word_counts,
            max_word_counts,
            required_tags: vec![
                "summary".to_string(),
                "evidence".to_string(),
                "implementation".to_string(),
            ],
            quality_threshold: 0.7,
            required_layers: vec![
                "immediate_answer".to_string(),
                "supporting_evidence".to_string(),
                "implementation_details".to_string(),
            ],
        }
    }

    /// Create completion criteria for Implementation research
    pub fn for_implementation() -> Self {
        let mut min_word_counts = HashMap::new();
        min_word_counts.insert("overview".to_string(), 30);
        min_word_counts.insert("steps".to_string(), 100);
        min_word_counts.insert("examples".to_string(), 50);

        let mut max_word_counts = HashMap::new();
        max_word_counts.insert("overview".to_string(), 300);
        max_word_counts.insert("steps".to_string(), 800);
        max_word_counts.insert("examples".to_string(), 500);

        Self {
            required_sections: vec![
                "overview".to_string(),
                "steps".to_string(),
                "examples".to_string(),
            ],
            min_word_counts,
            max_word_counts,
            required_tags: vec!["summary".to_string(), "implementation".to_string()],
            quality_threshold: 0.85,
            required_layers: vec![
                "immediate_answer".to_string(),
                "supporting_evidence".to_string(),
                "implementation_details".to_string(),
            ],
        }
    }

    /// Create completion criteria for Troubleshooting research
    pub fn for_troubleshooting() -> Self {
        let mut min_word_counts = HashMap::new();
        min_word_counts.insert("problem_description".to_string(), 20);
        min_word_counts.insert("diagnostic_steps".to_string(), 80);
        min_word_counts.insert("solutions".to_string(), 60);

        let mut max_word_counts = HashMap::new();
        max_word_counts.insert("problem_description".to_string(), 200);
        max_word_counts.insert("diagnostic_steps".to_string(), 600);
        max_word_counts.insert("solutions".to_string(), 400);

        Self {
            required_sections: vec![
                "problem_description".to_string(),
                "diagnostic_steps".to_string(),
                "solutions".to_string(),
            ],
            min_word_counts,
            max_word_counts,
            required_tags: vec![
                "summary".to_string(),
                "evidence".to_string(),
                "implementation".to_string(),
            ],
            quality_threshold: 0.82,
            required_layers: vec![
                "immediate_answer".to_string(),
                "supporting_evidence".to_string(),
                "implementation_details".to_string(),
            ],
        }
    }

    /// Create completion criteria for Learning research
    pub fn for_learning() -> Self {
        let mut min_word_counts = HashMap::new();
        min_word_counts.insert("concept_explanation".to_string(), 50);
        min_word_counts.insert("examples".to_string(), 40);
        min_word_counts.insert("applications".to_string(), 30);

        let mut max_word_counts = HashMap::new();
        max_word_counts.insert("concept_explanation".to_string(), 400);
        max_word_counts.insert("examples".to_string(), 300);
        max_word_counts.insert("applications".to_string(), 200);

        Self {
            required_sections: vec![
                "concept_explanation".to_string(),
                "examples".to_string(),
                "applications".to_string(),
            ],
            min_word_counts,
            max_word_counts,
            required_tags: vec!["summary".to_string(), "evidence".to_string()],
            quality_threshold: 0.75,
            required_layers: vec![
                "immediate_answer".to_string(),
                "supporting_evidence".to_string(),
            ],
        }
    }

    /// Create completion criteria for Validation research
    pub fn for_validation() -> Self {
        let mut min_word_counts = HashMap::new();
        min_word_counts.insert("approach_analysis".to_string(), 40);
        min_word_counts.insert("best_practices".to_string(), 60);
        min_word_counts.insert("trade_offs".to_string(), 50);

        let mut max_word_counts = HashMap::new();
        max_word_counts.insert("approach_analysis".to_string(), 300);
        max_word_counts.insert("best_practices".to_string(), 400);
        max_word_counts.insert("trade_offs".to_string(), 300);

        Self {
            required_sections: vec![
                "approach_analysis".to_string(),
                "best_practices".to_string(),
                "trade_offs".to_string(),
            ],
            min_word_counts,
            max_word_counts,
            required_tags: vec![
                "summary".to_string(),
                "evidence".to_string(),
                "implementation".to_string(),
            ],
            quality_threshold: 0.8,
            required_layers: vec![
                "immediate_answer".to_string(),
                "supporting_evidence".to_string(),
                "implementation_details".to_string(),
            ],
        }
    }

    /// Get completion criteria for a specific research type
    pub fn for_research_type(research_type: &ResearchType) -> Self {
        match research_type {
            ResearchType::Decision => Self::for_decision(),
            ResearchType::Implementation => Self::for_implementation(),
            ResearchType::Troubleshooting => Self::for_troubleshooting(),
            ResearchType::Learning => Self::for_learning(),
            ResearchType::Validation => Self::for_validation(),
        }
    }
}

/// Quality validator for research results
pub struct QualityValidator {
    /// Completion criteria by research type
    criteria_map: HashMap<ResearchType, CompletionCriteria>,
}

impl QualityValidator {
    /// Create a new quality validator
    pub fn new() -> Self {
        let mut criteria_map = HashMap::new();

        for research_type in ResearchType::all() {
            criteria_map.insert(
                research_type.clone(),
                CompletionCriteria::for_research_type(&research_type),
            );
        }

        Self { criteria_map }
    }

    /// Validate a research result against completion criteria
    pub fn validate(&self, result: &ResearchResult) -> Result<ValidationReport, ValidationError> {
        let criteria = self
            .criteria_map
            .get(result.research_type())
            .ok_or_else(|| {
                ValidationError::CompletionCriteriaMissing(format!(
                    "No criteria found for research type: {:?}",
                    result.research_type()
                ))
            })?;

        let mut report = ValidationReport::new(result.research_type().clone());

        // Validate progressive disclosure structure
        self.validate_progressive_disclosure(result, criteria, &mut report)?;

        // Validate semantic markup
        self.validate_semantic_markup(result, criteria, &mut report)?;

        // Validate content quality
        self.validate_content_quality(result, criteria, &mut report)?;

        // Calculate overall score from component scores
        report.calculate_overall_score();

        // Check overall quality threshold
        if report.overall_score < criteria.quality_threshold {
            return Err(ValidationError::QualityThresholdNotMet {
                expected: criteria.quality_threshold,
                actual: report.overall_score,
            });
        }

        Ok(report)
    }

    /// Validate progressive disclosure structure
    fn validate_progressive_disclosure(
        &self,
        result: &ResearchResult,
        criteria: &CompletionCriteria,
        report: &mut ValidationReport,
    ) -> Result<(), ValidationError> {
        // Check that all required layers are present
        let has_immediate_answer = !result.immediate_answer.is_empty();
        let has_supporting_evidence = !result.supporting_evidence.is_empty();
        let has_implementation_details = !result.implementation_details.is_empty();

        if !has_immediate_answer {
            report.add_issue("Missing immediate answer layer".to_string());
        }

        if criteria
            .required_layers
            .contains(&"supporting_evidence".to_string())
            && !has_supporting_evidence
        {
            report.add_issue("Missing supporting evidence layer".to_string());
        }

        if criteria
            .required_layers
            .contains(&"implementation_details".to_string())
            && !has_implementation_details
        {
            report.add_issue("Missing implementation details layer".to_string());
        }

        // Validate layer structure
        if has_immediate_answer {
            let word_count = result.immediate_answer.split_whitespace().count();
            if word_count < 10 {
                report.add_issue("Immediate answer too short".to_string());
            }
            if word_count > 200 {
                report.add_issue("Immediate answer too long".to_string());
            }
        }

        report.progressive_disclosure_score = if report.issues.is_empty() { 1.0 } else { 0.5 };
        Ok(())
    }

    /// Validate semantic markup
    fn validate_semantic_markup(
        &self,
        result: &ResearchResult,
        criteria: &CompletionCriteria,
        report: &mut ValidationReport,
    ) -> Result<(), ValidationError> {
        let evidence_content: Vec<&str> = result
            .supporting_evidence
            .iter()
            .map(|e| e.content.as_str())
            .collect();
        let content = format!(
            "{}\n{}",
            result.immediate_answer,
            evidence_content.join("\n")
        );

        // Check for required semantic tags
        let required_tags = &criteria.required_tags;
        let mut found_tags = 0;

        for tag in required_tags {
            if content.contains(&format!("<{tag}>")) || content.contains(&format!("<{tag} ")) {
                found_tags += 1;
            }
        }

        report.semantic_markup_score = if required_tags.is_empty() {
            1.0
        } else {
            found_tags as f64 / required_tags.len() as f64
        };

        if report.semantic_markup_score < 0.5 {
            report.add_issue("Insufficient semantic markup".to_string());
        }

        Ok(())
    }

    /// Validate content quality
    fn validate_content_quality(
        &self,
        result: &ResearchResult,
        criteria: &CompletionCriteria,
        report: &mut ValidationReport,
    ) -> Result<(), ValidationError> {
        // Use existing quality score from metadata
        let quality_score = result.metadata.quality_score;

        // Additional quality checks
        let has_meaningful_content =
            !result.immediate_answer.trim().is_empty() && result.immediate_answer.len() > 50;

        let has_relevant_evidence = !result.supporting_evidence.is_empty()
            && result.supporting_evidence.iter().any(|e| e.relevance > 0.5);

        if !has_meaningful_content {
            report.add_issue("Insufficient meaningful content".to_string());
            report.content_quality_score = 0.3;
        } else if !has_relevant_evidence
            && criteria
                .required_layers
                .contains(&"supporting_evidence".to_string())
        {
            report.add_issue("Low relevance supporting evidence".to_string());
            report.content_quality_score = 0.6;
        } else {
            report.content_quality_score = quality_score;
        }

        Ok(())
    }

    /// Get completion criteria for a research type
    pub fn get_criteria(&self, research_type: &ResearchType) -> Option<&CompletionCriteria> {
        self.criteria_map.get(research_type)
    }
}

impl Default for QualityValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation report with scores and issues
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Research type being validated
    pub research_type: ResearchType,
    /// Overall quality score (0.0-1.0)
    pub overall_score: f64,
    /// Progressive disclosure structure score
    pub progressive_disclosure_score: f64,
    /// Semantic markup score
    pub semantic_markup_score: f64,
    /// Content quality score
    pub content_quality_score: f64,
    /// List of validation issues
    pub issues: Vec<String>,
    /// Validation timestamp
    pub validated_at: chrono::DateTime<chrono::Utc>,
}

impl ValidationReport {
    /// Create a new validation report
    pub fn new(research_type: ResearchType) -> Self {
        Self {
            research_type,
            overall_score: 0.0,
            progressive_disclosure_score: 0.0,
            semantic_markup_score: 0.0,
            content_quality_score: 0.0,
            issues: Vec::new(),
            validated_at: chrono::Utc::now(),
        }
    }

    /// Add a validation issue
    pub fn add_issue(&mut self, issue: String) {
        self.issues.push(issue);
    }

    /// Calculate overall score from component scores
    pub fn calculate_overall_score(&mut self) {
        self.overall_score = self.progressive_disclosure_score * 0.3
            + self.semantic_markup_score * 0.2
            + self.content_quality_score * 0.5;
    }

    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.overall_score >= 0.7 && self.issues.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use fortitude_types::research::*;

    fn create_test_research_result(research_type: ResearchType) -> ResearchResult {
        let request = ClassifiedRequest::new(
            "Test query".to_string(),
            research_type,
            AudienceContext::default(),
            DomainContext::default(),
            0.8,
            vec!["test".to_string()],
        );

        let metadata = ResearchMetadata {
            completed_at: Utc::now(),
            processing_time_ms: 1000,
            sources_consulted: vec!["test".to_string()],
            quality_score: 0.85,
            cache_key: "test-key".to_string(),
            tags: HashMap::new(),
        };

        ResearchResult::new(
            request,
            "This is a comprehensive answer to the research question with sufficient detail."
                .to_string(),
            vec![Evidence {
                source: "test".to_string(),
                content: "Supporting evidence content".to_string(),
                relevance: 0.8,
                evidence_type: "documentation".to_string(),
            }],
            vec![Detail {
                category: "implementation".to_string(),
                content: "Implementation details".to_string(),
                priority: "high".to_string(),
                prerequisites: vec![],
            }],
            metadata,
        )
    }

    #[test]
    fn test_completion_criteria_for_decision() {
        let criteria = CompletionCriteria::for_decision();
        assert!(criteria
            .required_sections
            .contains(&"problem_statement".to_string()));
        assert!(criteria.required_sections.contains(&"options".to_string()));
        assert!(criteria
            .required_sections
            .contains(&"recommendation".to_string()));
        assert_eq!(criteria.quality_threshold, 0.7);
    }

    #[test]
    fn test_completion_criteria_for_implementation() {
        let criteria = CompletionCriteria::for_implementation();
        assert!(criteria.required_sections.contains(&"overview".to_string()));
        assert!(criteria.required_sections.contains(&"steps".to_string()));
        assert!(criteria.required_sections.contains(&"examples".to_string()));
        assert_eq!(criteria.quality_threshold, 0.85);
    }

    #[test]
    fn test_quality_validator_creation() {
        let validator = QualityValidator::new();
        assert!(validator.get_criteria(&ResearchType::Decision).is_some());
        assert!(validator
            .get_criteria(&ResearchType::Implementation)
            .is_some());
        assert!(validator
            .get_criteria(&ResearchType::Troubleshooting)
            .is_some());
        assert!(validator.get_criteria(&ResearchType::Learning).is_some());
        assert!(validator.get_criteria(&ResearchType::Validation).is_some());
    }

    #[test]
    fn test_validation_report_creation() {
        let mut report = ValidationReport::new(ResearchType::Decision);
        assert_eq!(report.research_type, ResearchType::Decision);
        assert_eq!(report.overall_score, 0.0);
        assert!(report.issues.is_empty());

        report.add_issue("Test issue".to_string());
        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.issues[0], "Test issue");
    }

    #[test]
    fn test_overall_score_calculation() {
        let mut report = ValidationReport::new(ResearchType::Decision);
        report.progressive_disclosure_score = 0.8;
        report.semantic_markup_score = 0.6;
        report.content_quality_score = 0.9;

        report.calculate_overall_score();

        // Expected: 0.8 * 0.3 + 0.6 * 0.2 + 0.9 * 0.5 = 0.24 + 0.12 + 0.45 = 0.81
        assert!((report.overall_score - 0.81).abs() < 0.01);
    }

    #[test]
    fn test_validate_good_result() {
        let validator = QualityValidator::new();
        let result = create_test_research_result(ResearchType::Decision);

        let validation_result = validator.validate(&result);
        assert!(validation_result.is_ok());

        let report = validation_result.unwrap();
        assert_eq!(report.research_type, ResearchType::Decision);
        assert!(report.content_quality_score > 0.0);
    }

    #[test]
    fn test_validate_empty_result() {
        let validator = QualityValidator::new();
        let request = ClassifiedRequest::new(
            "Test query".to_string(),
            ResearchType::Decision,
            AudienceContext::default(),
            DomainContext::default(),
            0.8,
            vec!["test".to_string()],
        );

        let metadata = ResearchMetadata {
            completed_at: Utc::now(),
            processing_time_ms: 1000,
            sources_consulted: vec![],
            quality_score: 0.2,
            cache_key: "test-key".to_string(),
            tags: HashMap::new(),
        };

        let result = ResearchResult::new(
            request,
            "".to_string(), // Empty answer
            vec![],
            vec![],
            metadata,
        );

        let validation_result = validator.validate(&result);
        assert!(validation_result.is_err());
    }
}
