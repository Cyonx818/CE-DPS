// ABOUTME: Classification types for the Fortitude research system
use crate::research::ResearchType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Classification rule with keywords and weights
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassificationRule {
    /// Research type this rule targets
    pub research_type: ResearchType,
    /// Keywords with their weights
    pub keywords: HashMap<String, f64>,
    /// Minimum confidence threshold for this rule
    pub min_confidence: f64,
    /// Rule priority (higher = more important)
    pub priority: u32,
}

impl ClassificationRule {
    /// Create a new classification rule
    pub fn new(
        research_type: ResearchType,
        keywords: HashMap<String, f64>,
        min_confidence: f64,
        priority: u32,
    ) -> Self {
        Self {
            research_type,
            keywords,
            min_confidence,
            priority,
        }
    }

    /// Calculate confidence score for given text
    pub fn calculate_confidence(&self, text: &str) -> f64 {
        let text_lower = text.to_lowercase();
        let mut total_weight = 0.0;
        let mut matched_weight = 0.0;

        for (keyword, weight) in &self.keywords {
            total_weight += weight;
            if text_lower.contains(&keyword.to_lowercase()) {
                matched_weight += weight;
            }
        }

        if total_weight == 0.0 {
            0.0
        } else {
            matched_weight / total_weight
        }
    }

    /// Get keywords that match in the given text
    pub fn get_matched_keywords(&self, text: &str) -> Vec<String> {
        let text_lower = text.to_lowercase();
        self.keywords
            .keys()
            .filter(|keyword| text_lower.contains(&keyword.to_lowercase()))
            .cloned()
            .collect()
    }
}

/// Classification result with confidence and matched keywords
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassificationResult {
    /// Classified research type
    pub research_type: ResearchType,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Keywords that influenced the classification
    pub matched_keywords: Vec<String>,
    /// Rule that was applied
    pub rule_priority: u32,
    /// All candidate results considered
    pub candidates: Vec<ClassificationCandidate>,
}

impl ClassificationResult {
    /// Create a new classification result
    pub fn new(
        research_type: ResearchType,
        confidence: f64,
        matched_keywords: Vec<String>,
        rule_priority: u32,
        candidates: Vec<ClassificationCandidate>,
    ) -> Self {
        Self {
            research_type,
            confidence,
            matched_keywords,
            rule_priority,
            candidates,
        }
    }

    /// Check if confidence meets threshold
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.confidence >= threshold
    }
}

/// Candidate classification result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassificationCandidate {
    /// Candidate research type
    pub research_type: ResearchType,
    /// Confidence score for this candidate
    pub confidence: f64,
    /// Keywords that matched for this candidate
    pub matched_keywords: Vec<String>,
    /// Rule priority that generated this candidate
    pub rule_priority: u32,
}

impl ClassificationCandidate {
    /// Create a new classification candidate
    pub fn new(
        research_type: ResearchType,
        confidence: f64,
        matched_keywords: Vec<String>,
        rule_priority: u32,
    ) -> Self {
        Self {
            research_type,
            confidence,
            matched_keywords,
            rule_priority,
        }
    }
}

/// Configuration for the classification system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassificationConfig {
    /// Default confidence threshold
    pub default_threshold: f64,
    /// Fallback research type when no rules match
    pub fallback_type: ResearchType,
    /// Enable fuzzy matching
    pub enable_fuzzy_matching: bool,
    /// Maximum number of candidates to consider
    pub max_candidates: usize,
}

impl Default for ClassificationConfig {
    fn default() -> Self {
        Self {
            default_threshold: 0.6,
            fallback_type: ResearchType::Learning,
            enable_fuzzy_matching: false,
            max_candidates: 10,
        }
    }
}

/// Trait for classification systems
pub trait Classifier {
    /// Classify a research query
    fn classify(
        &self,
        query: &str,
    ) -> Result<ClassificationResult, crate::error::ClassificationError>;

    /// Get classification confidence without full classification
    fn get_confidence(&self, query: &str, research_type: &ResearchType) -> f64;

    /// Get all possible classifications with confidence scores
    fn get_all_classifications(&self, query: &str) -> Vec<ClassificationCandidate>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_rule_confidence() {
        let mut keywords = HashMap::new();
        keywords.insert("implement".to_string(), 1.0);
        keywords.insert("build".to_string(), 0.8);
        keywords.insert("create".to_string(), 0.6);

        let rule = ClassificationRule::new(ResearchType::Implementation, keywords, 0.5, 1);

        // Test exact match
        let confidence = rule.calculate_confidence("How to implement a feature?");
        assert!(confidence > 0.0);

        // Test multiple matches
        let confidence = rule.calculate_confidence("implement and build a solution");
        assert!(confidence > 0.5);

        // Test no match
        let confidence = rule.calculate_confidence("what is the definition?");
        assert_eq!(confidence, 0.0);
    }

    #[test]
    fn test_classification_rule_matched_keywords() {
        let mut keywords = HashMap::new();
        keywords.insert("debug".to_string(), 1.0);
        keywords.insert("fix".to_string(), 0.8);
        keywords.insert("error".to_string(), 0.9);

        let rule = ClassificationRule::new(ResearchType::Troubleshooting, keywords, 0.5, 1);

        let matched = rule.get_matched_keywords("How to debug and fix an error?");
        assert_eq!(matched.len(), 3);
        assert!(matched.contains(&"debug".to_string()));
        assert!(matched.contains(&"fix".to_string()));
        assert!(matched.contains(&"error".to_string()));
    }

    #[test]
    fn test_classification_result_threshold() {
        let result = ClassificationResult::new(
            ResearchType::Decision,
            0.75,
            vec!["choose".to_string()],
            1,
            vec![],
        );

        assert!(result.meets_threshold(0.6));
        assert!(!result.meets_threshold(0.8));
    }

    #[test]
    fn test_classification_config_default() {
        let config = ClassificationConfig::default();
        assert_eq!(config.default_threshold, 0.6);
        assert_eq!(config.fallback_type, ResearchType::Learning);
        assert!(!config.enable_fuzzy_matching);
        assert_eq!(config.max_candidates, 10);
    }

    #[test]
    fn test_classification_candidate_creation() {
        let candidate = ClassificationCandidate::new(
            ResearchType::Validation,
            0.8,
            vec!["test".to_string(), "verify".to_string()],
            2,
        );

        assert_eq!(candidate.research_type, ResearchType::Validation);
        assert_eq!(candidate.confidence, 0.8);
        assert_eq!(candidate.matched_keywords.len(), 2);
        assert_eq!(candidate.rule_priority, 2);
    }
}
