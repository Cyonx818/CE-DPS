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

// ABOUTME: Basic keyword-based classification engine for research type detection
use fortitude_types::{
    AudienceContext, ClassificationCandidate, ClassificationConfig, ClassificationError,
    ClassificationResult, ClassificationRule, ClassifiedRequest, Classifier, DomainContext,
    ResearchType,
};
use tracing::{debug, info, warn};

/// Basic keyword-based classifier for research type detection
pub struct BasicClassifier {
    rules: Vec<ClassificationRule>,
    config: ClassificationConfig,
}

impl BasicClassifier {
    /// Create a new basic classifier with default rules
    pub fn new(config: ClassificationConfig) -> Self {
        let rules = Self::create_default_rules();
        Self { rules, config }
    }

    /// Create a new classifier with custom rules
    pub fn with_rules(rules: Vec<ClassificationRule>, config: ClassificationConfig) -> Self {
        Self { rules, config }
    }

    /// Create default classification rules for all research types
    fn create_default_rules() -> Vec<ClassificationRule> {
        vec![
            // Decision rules
            ClassificationRule::new(
                ResearchType::Decision,
                [
                    ("choose", 1.0),
                    ("select", 0.9),
                    ("decide", 1.0),
                    ("decision", 1.0),
                    ("alternative", 0.8),
                    ("option", 0.7),
                    ("compare", 0.8),
                    ("versus", 0.7),
                    ("vs", 0.7),
                    ("best", 0.6),
                    ("recommend", 0.8),
                    ("should I", 0.9),
                    ("which", 0.7),
                    ("what is better", 0.9),
                    ("pros and cons", 0.9),
                    ("trade-offs", 0.8),
                    ("evaluate", 0.7),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
                0.6,
                1,
            ),
            // Implementation rules
            ClassificationRule::new(
                ResearchType::Implementation,
                [
                    ("implement", 1.0),
                    ("build", 0.9),
                    ("create", 0.8),
                    ("develop", 0.9),
                    ("code", 0.8),
                    ("write", 0.7),
                    ("make", 0.6),
                    ("construct", 0.8),
                    ("setup", 0.7),
                    ("configure", 0.7),
                    ("how to", 0.6),
                    ("step by step", 0.8),
                    ("tutorial", 0.7),
                    ("guide", 0.6),
                    ("example", 0.7),
                    ("sample", 0.6),
                    ("template", 0.7),
                    ("boilerplate", 0.8),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
                0.6,
                1,
            ),
            // Troubleshooting rules
            ClassificationRule::new(
                ResearchType::Troubleshooting,
                [
                    ("error", 1.0),
                    ("bug", 1.0),
                    ("problem", 0.9),
                    ("issue", 0.8),
                    ("fix", 1.0),
                    ("debug", 1.0),
                    ("solve", 0.9),
                    ("troubleshoot", 1.0),
                    ("broken", 0.9),
                    ("failed", 0.8),
                    ("failing", 0.8),
                    ("not working", 0.9),
                    ("doesn't work", 0.9),
                    ("won't", 0.7),
                    ("can't", 0.7),
                    ("unable", 0.7),
                    ("crash", 0.9),
                    ("exception", 0.9),
                    ("panic", 0.9),
                    ("segfault", 0.9),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
                0.6,
                2, // Higher priority than general implementation
            ),
            // Learning rules
            ClassificationRule::new(
                ResearchType::Learning,
                [
                    ("what is", 1.0),
                    ("what are", 1.0),
                    ("explain", 0.9),
                    ("understand", 0.9),
                    ("learn", 1.0),
                    ("definition", 0.9),
                    ("concept", 0.8),
                    ("theory", 0.8),
                    ("principle", 0.8),
                    ("overview", 0.7),
                    ("introduction", 0.8),
                    ("basics", 0.8),
                    ("fundamentals", 0.8),
                    ("background", 0.7),
                    ("history", 0.6),
                    ("why", 0.6),
                    ("purpose", 0.7),
                    ("meaning", 0.7),
                    ("documentation", 0.6),
                    ("reference", 0.6),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
                0.6,
                1,
            ),
            // Validation rules
            ClassificationRule::new(
                ResearchType::Validation,
                [
                    ("test", 1.0),
                    ("testing", 1.0),
                    ("verify", 0.9),
                    ("validate", 1.0),
                    ("check", 0.8),
                    ("ensure", 0.7),
                    ("confirm", 0.8),
                    ("prove", 0.7),
                    ("quality", 0.7),
                    ("performance", 0.7),
                    ("benchmark", 0.8),
                    ("measure", 0.7),
                    ("assess", 0.7),
                    ("evaluate", 0.7),
                    ("review", 0.6),
                    ("audit", 0.8),
                    ("correct", 0.6),
                    ("accurate", 0.6),
                    ("reliable", 0.6),
                    ("stable", 0.6),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
                0.6,
                1,
            ),
        ]
    }

    /// Classify a query into a research request with full context
    pub fn classify_request(
        &self,
        query: &str,
        audience_context: Option<AudienceContext>,
        domain_context: Option<DomainContext>,
    ) -> Result<ClassifiedRequest, ClassificationError> {
        let classification_result = self.classify(query)?;

        let request = ClassifiedRequest::new(
            query.to_string(),
            classification_result.research_type,
            audience_context.unwrap_or_default(),
            domain_context.unwrap_or_default(),
            classification_result.confidence,
            classification_result.matched_keywords,
        );

        Ok(request)
    }

    /// Get the best matching rule for a query
    #[allow(dead_code)]
    fn get_best_rule(&self, query: &str) -> Option<(&ClassificationRule, f64, Vec<String>)> {
        let mut best_rule = None;
        let mut best_score = 0.0;
        let mut best_keywords = Vec::new();

        for rule in &self.rules {
            let confidence = rule.calculate_confidence(query);
            if confidence > best_score && confidence >= rule.min_confidence {
                best_score = confidence;
                best_rule = Some(rule);
                best_keywords = rule.get_matched_keywords(query);
            }
        }

        best_rule.map(|rule| (rule, best_score, best_keywords))
    }

    /// Generate all classification candidates for a query
    fn generate_candidates(&self, query: &str) -> Vec<ClassificationCandidate> {
        let mut candidates = Vec::new();

        for rule in &self.rules {
            let confidence = rule.calculate_confidence(query);
            if confidence > 0.0 {
                let matched_keywords = rule.get_matched_keywords(query);
                candidates.push(ClassificationCandidate::new(
                    rule.research_type.clone(),
                    confidence,
                    matched_keywords,
                    rule.priority,
                ));
            }
        }

        // Sort by confidence (descending) then by priority (descending)
        candidates.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.rule_priority.cmp(&a.rule_priority))
        });

        // Limit to max candidates
        candidates.truncate(self.config.max_candidates);

        candidates
    }
}

impl Classifier for BasicClassifier {
    fn classify(&self, query: &str) -> Result<ClassificationResult, ClassificationError> {
        debug!("Classifying query: '{}'", query);

        if query.trim().is_empty() {
            return Err(ClassificationError::InvalidInput(
                "Query cannot be empty".to_string(),
            ));
        }

        let candidates = self.generate_candidates(query);

        if candidates.is_empty() {
            info!("No classification rules matched for query: '{}'", query);
            // Return fallback type with low confidence
            let fallback_result = ClassificationResult::new(
                self.config.fallback_type.clone(),
                0.0,
                vec![],
                0,
                vec![],
            );
            return Ok(fallback_result);
        }

        // Take the best candidate
        let best_candidate = &candidates[0];

        // Check if it meets the threshold
        if best_candidate.confidence < self.config.default_threshold {
            warn!(
                "Best classification confidence ({:.2}) below threshold ({:.2}) for query: '{}'",
                best_candidate.confidence, self.config.default_threshold, query
            );
            return Err(ClassificationError::LowConfidence {
                actual: best_candidate.confidence,
                threshold: self.config.default_threshold,
            });
        }

        info!(
            "Classified query '{}' as {} with confidence {:.2}",
            query, best_candidate.research_type, best_candidate.confidence
        );

        Ok(ClassificationResult::new(
            best_candidate.research_type.clone(),
            best_candidate.confidence,
            best_candidate.matched_keywords.clone(),
            best_candidate.rule_priority,
            candidates,
        ))
    }

    fn get_confidence(&self, query: &str, research_type: &ResearchType) -> f64 {
        for rule in &self.rules {
            if rule.research_type == *research_type {
                return rule.calculate_confidence(query);
            }
        }
        0.0
    }

    fn get_all_classifications(&self, query: &str) -> Vec<ClassificationCandidate> {
        self.generate_candidates(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_classifier() -> BasicClassifier {
        let config = ClassificationConfig {
            default_threshold: 0.1, // Lower threshold for testing
            ..Default::default()
        };
        BasicClassifier::new(config)
    }

    #[test]
    fn test_decision_classification() {
        let classifier = create_test_classifier();

        let result = classifier
            .classify("Should I choose React or Vue?")
            .unwrap();
        assert_eq!(result.research_type, ResearchType::Decision);
        assert!(result.confidence > 0.1);
        assert!(result.matched_keywords.contains(&"choose".to_string()));
    }

    #[test]
    fn test_implementation_classification() {
        let classifier = create_test_classifier();

        let result = classifier
            .classify("How to implement async functions in Rust?")
            .unwrap();
        assert_eq!(result.research_type, ResearchType::Implementation);
        assert!(result.confidence > 0.1);
        assert!(result.matched_keywords.contains(&"implement".to_string()));
    }

    #[test]
    fn test_troubleshooting_classification() {
        let classifier = create_test_classifier();

        let result = classifier
            .classify("Getting an error when running cargo build and need to fix it")
            .unwrap();
        assert_eq!(result.research_type, ResearchType::Troubleshooting);
        assert!(result.confidence > 0.1);
        assert!(result.matched_keywords.contains(&"error".to_string()));
    }

    #[test]
    fn test_learning_classification() {
        let classifier = create_test_classifier();

        let result = classifier
            .classify("What is the definition of async programming?")
            .unwrap();
        assert_eq!(result.research_type, ResearchType::Learning);
        assert!(result.confidence > 0.1);
        assert!(result.matched_keywords.contains(&"what is".to_string()));
    }

    #[test]
    fn test_validation_classification() {
        let classifier = create_test_classifier();

        let result = classifier
            .classify("How to test and validate my Rust application?")
            .unwrap();
        assert_eq!(result.research_type, ResearchType::Validation);
        assert!(result.confidence > 0.1);
        assert!(result.matched_keywords.contains(&"test".to_string()));
    }

    #[test]
    fn test_empty_query() {
        let classifier = create_test_classifier();

        let result = classifier.classify("");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ClassificationError::InvalidInput(_)
        ));
    }

    #[test]
    fn test_no_match_fallback() {
        let classifier = create_test_classifier();

        let result = classifier.classify("xyz abc def random words").unwrap();
        assert_eq!(result.research_type, ResearchType::Learning); // Default fallback
        assert_eq!(result.confidence, 0.0);
        assert!(result.matched_keywords.is_empty());
    }

    #[test]
    fn test_confidence_threshold() {
        let config = ClassificationConfig {
            default_threshold: 0.9, // Very high threshold
            ..Default::default()
        };

        let classifier = BasicClassifier::new(config);

        // This should fail the threshold check
        let result = classifier.classify("maybe choose something");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ClassificationError::LowConfidence { .. }
        ));
    }

    #[test]
    fn test_get_confidence() {
        let classifier = create_test_classifier();

        let confidence =
            classifier.get_confidence("How to implement", &ResearchType::Implementation);
        assert!(confidence > 0.0);

        let confidence = classifier.get_confidence("random text", &ResearchType::Implementation);
        assert_eq!(confidence, 0.0);
    }

    #[test]
    fn test_get_all_classifications() {
        let classifier = create_test_classifier();

        let candidates = classifier.get_all_classifications("How to implement and test");
        assert!(candidates.len() >= 2); // Should match both implementation and validation

        // Should be sorted by confidence
        for i in 1..candidates.len() {
            assert!(candidates[i - 1].confidence >= candidates[i].confidence);
        }
    }

    #[test]
    fn test_classify_request() {
        let classifier = create_test_classifier();

        let request = classifier
            .classify_request("How to debug Rust code?", None, None)
            .unwrap();

        assert_eq!(request.research_type, ResearchType::Troubleshooting);
        assert!(request.confidence > 0.1);
        assert!(!request.matched_keywords.is_empty());
        assert_eq!(request.original_query, "How to debug Rust code?");
    }

    #[test]
    fn test_classify_request_with_context() {
        let classifier = create_test_classifier();

        let audience_context = AudienceContext {
            level: "beginner".to_string(),
            domain: "rust".to_string(),
            format: "markdown".to_string(),
        };

        let domain_context = DomainContext {
            technology: "rust".to_string(),
            project_type: "cli".to_string(),
            frameworks: vec!["tokio".to_string()],
            tags: vec!["async".to_string()],
        };

        let request = classifier
            .classify_request(
                "How to implement async functions?",
                Some(audience_context.clone()),
                Some(domain_context.clone()),
            )
            .unwrap();

        assert_eq!(request.research_type, ResearchType::Implementation);
        assert_eq!(request.audience_context, audience_context);
        assert_eq!(request.domain_context, domain_context);
    }
}
