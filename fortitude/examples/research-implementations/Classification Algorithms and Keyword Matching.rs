// Fortitude AI Research Pipeline - Text Classification System
// Chain-of-thought Implementation: Requirements → Architecture → Dependencies → Error Handling → Core Logic → Tests → Optimization

use std::collections::HashMap;
use std::fmt;
use regex::Regex;
use serde::{Deserialize, Serialize};

// ===== REQUIREMENTS ANALYSIS =====
// 1. Classify into 5 research types with confidence scoring
// 2. Keyword-based approach with extensible patterns
// 3. Handle ambiguous inputs with fallback strategies
// 4. >80% accuracy target with performance optimization

// ===== ARCHITECTURE CHOICE =====
// Multi-layered classification system:
// - Keyword matching with weighted scoring
// - Pattern-based rules for context
// - Confidence thresholding with fallback logic
// - Extensible framework for new research types

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResearchType {
    Decision,
    Implementation,
    Troubleshooting,
    Learning,
    Validation,
}

impl fmt::Display for ResearchType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResearchType::Decision => write!(f, "Decision"),
            ResearchType::Implementation => write!(f, "Implementation"),
            ResearchType::Troubleshooting => write!(f, "Troubleshooting"),
            ResearchType::Learning => write!(f, "Learning"),
            ResearchType::Validation => write!(f, "Validation"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClassificationResult {
    pub research_type: ResearchType,
    pub confidence: f64,
    pub matched_keywords: Vec<String>,
    pub is_fallback: bool,
}

#[derive(Debug, Clone)]
pub struct KeywordRule {
    pub keywords: Vec<String>,
    pub weight: f64,
    pub requires_context: bool,
    pub context_patterns: Vec<Regex>,
}

// ===== ERROR HANDLING DESIGN =====
#[derive(Debug)]
pub enum ClassificationError {
    EmptyInput,
    InvalidConfiguration,
    ProcessingError(String),
}

impl fmt::Display for ClassificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClassificationError::EmptyInput => write!(f, "Input text is empty"),
            ClassificationError::InvalidConfiguration => write!(f, "Invalid classifier configuration"),
            ClassificationError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
        }
    }
}

impl std::error::Error for ClassificationError {}

// ===== CORE CLASSIFICATION LOGIC =====
pub struct FortitudeClassifier {
    rules: HashMap<ResearchType, Vec<KeywordRule>>,
    confidence_threshold: f64,
    fallback_strategy: FallbackStrategy,
}

#[derive(Debug, Clone)]
pub enum FallbackStrategy {
    HighestScore,
    LearningDefault,
    UserPrompt,
}

impl FortitudeClassifier {
    pub fn new() -> Self {
        let mut classifier = Self {
            rules: HashMap::new(),
            confidence_threshold: 0.6,
            fallback_strategy: FallbackStrategy::HighestScore,
        };
        classifier.initialize_default_rules();
        classifier
    }

    pub fn with_confidence_threshold(mut self, threshold: f64) -> Self {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    pub fn with_fallback_strategy(mut self, strategy: FallbackStrategy) -> Self {
        self.fallback_strategy = strategy;
        self
    }

    fn initialize_default_rules(&mut self) {
        // Decision Research Type
        self.add_rule(ResearchType::Decision, KeywordRule {
            keywords: vec!["choose", "decide", "compare", "evaluate", "options", "alternatives", 
                          "recommend", "best", "versus", "vs", "should", "which"].iter().map(|s| s.to_string()).collect(),
            weight: 1.0,
            requires_context: false,
            context_patterns: vec![],
        });

        // Implementation Research Type
        self.add_rule(ResearchType::Implementation, KeywordRule {
            keywords: vec!["implement", "build", "create", "develop", "code", "setup", "install",
                          "configure", "deploy", "architecture", "design", "how to"].iter().map(|s| s.to_string()).collect(),
            weight: 1.2,
            requires_context: true,
            context_patterns: vec![
                Regex::new(r"how\s+to\s+\w+").unwrap(),
                Regex::new(r"step\s+by\s+step").unwrap(),
            ],
        });

        // Troubleshooting Research Type
        self.add_rule(ResearchType::Troubleshooting, KeywordRule {
            keywords: vec!["error", "bug", "fix", "problem", "issue", "debug", "broken", "fail",
                          "troubleshoot", "resolve", "solve", "why", "not working"].iter().map(|s| s.to_string()).collect(),
            weight: 1.3,
            requires_context: true,
            context_patterns: vec![
                Regex::new(r"(error|exception|fail)").unwrap(),
                Regex::new(r"not\s+working").unwrap(),
            ],
        });

        // Learning Research Type
        self.add_rule(ResearchType::Learning, KeywordRule {
            keywords: vec!["learn", "understand", "explain", "what is", "tutorial", "guide", 
                          "introduction", "basics", "fundamentals", "concept", "theory"].iter().map(|s| s.to_string()).collect(),
            weight: 0.9,
            requires_context: false,
            context_patterns: vec![],
        });

        // Validation Research Type
        self.add_rule(ResearchType::Validation, KeywordRule {
            keywords: vec!["test", "verify", "validate", "check", "confirm", "ensure", "quality",
                          "benchmark", "performance", "accuracy", "correct"].iter().map(|s| s.to_string()).collect(),
            weight: 1.1,
            requires_context: true,
            context_patterns: vec![
                Regex::new(r"(test|verify|validate)\s+\w+").unwrap(),
            ],
        });
    }

    pub fn add_rule(&mut self, research_type: ResearchType, rule: KeywordRule) {
        self.rules.entry(research_type).or_insert_with(Vec::new).push(rule);
    }

    pub fn classify(&self, text: &str) -> Result<ClassificationResult, ClassificationError> {
        if text.trim().is_empty() {
            return Err(ClassificationError::EmptyInput);
        }

        let normalized_text = text.to_lowercase();
        let mut scores: HashMap<ResearchType, (f64, Vec<String>)> = HashMap::new();

        // Calculate scores for each research type
        for (research_type, rules) in &self.rules {
            let (total_score, matched_keywords) = self.calculate_type_score(&normalized_text, rules);
            scores.insert(research_type.clone(), (total_score, matched_keywords));
        }

        // Find the best match
        let best_match = scores
            .iter()
            .max_by(|a, b| a.1.0.partial_cmp(&b.1.0).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or(ClassificationError::ProcessingError("No matches found".to_string()))?;

        let (research_type, (confidence, matched_keywords)) = best_match;
        let normalized_confidence = (confidence / 10.0).min(1.0); // Normalize to 0-1 range

        let is_fallback = normalized_confidence < self.confidence_threshold;
        let final_result = if is_fallback {
            self.apply_fallback_strategy(&scores)
        } else {
            ClassificationResult {
                research_type: research_type.clone(),
                confidence: normalized_confidence,
                matched_keywords: matched_keywords.clone(),
                is_fallback: false,
            }
        };

        Ok(final_result)
    }

    fn calculate_type_score(&self, text: &str, rules: &[KeywordRule]) -> (f64, Vec<String>) {
        let mut total_score = 0.0;
        let mut matched_keywords = Vec::new();

        for rule in rules {
            let mut rule_score = 0.0;
            
            // Check keyword matches
            for keyword in &rule.keywords {
                if text.contains(keyword) {
                    rule_score += rule.weight;
                    matched_keywords.push(keyword.clone());
                }
            }

            // Apply context pattern bonuses
            if rule.requires_context && !rule.context_patterns.is_empty() {
                for pattern in &rule.context_patterns {
                    if pattern.is_match(text) {
                        rule_score *= 1.5; // Context bonus
                        break;
                    }
                }
            }

            total_score += rule_score;
        }

        (total_score, matched_keywords)
    }

    fn apply_fallback_strategy(&self, scores: &HashMap<ResearchType, (f64, Vec<String>)>) -> ClassificationResult {
        match self.fallback_strategy {
            FallbackStrategy::HighestScore => {
                let best = scores
                    .iter()
                    .max_by(|a, b| a.1.0.partial_cmp(&b.1.0).unwrap_or(std::cmp::Ordering::Equal))
                    .unwrap();
                
                ClassificationResult {
                    research_type: best.0.clone(),
                    confidence: (best.1.0 / 10.0).min(1.0),
                    matched_keywords: best.1.1.clone(),
                    is_fallback: true,
                }
            },
            FallbackStrategy::LearningDefault => {
                ClassificationResult {
                    research_type: ResearchType::Learning,
                    confidence: 0.3,
                    matched_keywords: vec!["fallback".to_string()],
                    is_fallback: true,
                }
            },
            FallbackStrategy::UserPrompt => {
                // In a real implementation, this would trigger user interaction
                ClassificationResult {
                    research_type: ResearchType::Learning,
                    confidence: 0.1,
                    matched_keywords: vec!["user_prompt_needed".to_string()],
                    is_fallback: true,
                }
            }
        }
    }

    // Performance optimization: batch classification
    pub fn classify_batch(&self, texts: &[String]) -> Vec<Result<ClassificationResult, ClassificationError>> {
        texts.iter().map(|text| self.classify(text)).collect()
    }
}

// ===== COMPREHENSIVE TESTS =====
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_classification() {
        let classifier = FortitudeClassifier::new();
        let result = classifier.classify("Which algorithm should I choose for sorting?").unwrap();
        assert_eq!(result.research_type, ResearchType::Decision);
        assert!(result.confidence > 0.5);
        assert!(!result.is_fallback);
    }

    #[test]
    fn test_implementation_classification() {
        let classifier = FortitudeClassifier::new();
        let result = classifier.classify("How to implement a binary tree in Rust step by step").unwrap();
        assert_eq!(result.research_type, ResearchType::Implementation);
        assert!(result.confidence > 0.6);
    }

    #[test]
    fn test_troubleshooting_classification() {
        let classifier = FortitudeClassifier::new();
        let result = classifier.classify("Getting error when compiling Rust code, need to fix").unwrap();
        assert_eq!(result.research_type, ResearchType::Troubleshooting);
        assert!(result.confidence > 0.7);
    }

    #[test]
    fn test_learning_classification() {
        let classifier = FortitudeClassifier::new();
        let result = classifier.classify("Explain the fundamentals of machine learning").unwrap();
        assert_eq!(result.research_type, ResearchType::Learning);
        assert!(result.confidence > 0.4);
    }

    #[test]
    fn test_validation_classification() {
        let classifier = FortitudeClassifier::new();
        let result = classifier.classify("Test the performance of my algorithm").unwrap();
        assert_eq!(result.research_type, ResearchType::Validation);
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_empty_input_error() {
        let classifier = FortitudeClassifier::new();
        let result = classifier.classify("");
        assert!(matches!(result, Err(ClassificationError::EmptyInput)));
    }

    #[test]
    fn test_fallback_strategy() {
        let classifier = FortitudeClassifier::new().with_confidence_threshold(0.9);
        let result = classifier.classify("random text").unwrap();
        assert!(result.is_fallback);
    }

    #[test]
    fn test_confidence_threshold_adjustment() {
        let classifier = FortitudeClassifier::new().with_confidence_threshold(0.8);
        let result = classifier.classify("maybe choose something").unwrap();
        // Should still classify but might trigger fallback based on threshold
        assert!(result.confidence >= 0.0);
    }

    #[test]
    fn test_batch_classification() {
        let classifier = FortitudeClassifier::new();
        let texts = vec![
            "How to implement sorting".to_string(),
            "Fix this error".to_string(),
            "Learn about AI".to_string(),
        ];
        let results = classifier.classify_batch(&texts);
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_keyword_matching_accuracy() {
        let classifier = FortitudeClassifier::new();
        let test_cases = vec![
            ("decide between options", ResearchType::Decision),
            ("build a web app", ResearchType::Implementation),
            ("debug this issue", ResearchType::Troubleshooting),
            ("understand concepts", ResearchType::Learning),
            ("validate results", ResearchType::Validation),
        ];

        let mut correct = 0;
        for (text, expected) in test_cases {
            if let Ok(result) = classifier.classify(text) {
                if result.research_type == expected {
                    correct += 1;
                }
            }
        }
        
        let accuracy = correct as f64 / 5.0;
        assert!(accuracy >= 0.8, "Accuracy requirement: >80%, got: {}%", accuracy * 100.0);
    }
}

// ===== PERFORMANCE BENCHMARKS =====
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_single_classification() {
        let classifier = FortitudeClassifier::new();
        let text = "How to implement a machine learning algorithm for classification";
        
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = classifier.classify(text).unwrap();
        }
        let duration = start.elapsed();
        
        println!("1000 classifications took: {:?}", duration);
        println!("Average per classification: {:?}", duration / 1000);
        
        // Performance target: < 1ms per classification
        assert!(duration.as_millis() < 1000, "Performance requirement not met");
    }

    #[test]
    fn benchmark_batch_classification() {
        let classifier = FortitudeClassifier::new();
        let texts: Vec<String> = (0..100)
            .map(|i| format!("Test classification text number {}", i))
            .collect();
        
        let start = Instant::now();
        let results = classifier.classify_batch(&texts);
        let duration = start.elapsed();
        
        println!("Batch classification of 100 texts took: {:?}", duration);
        assert_eq!(results.len(), 100);
        assert!(duration.as_millis() < 100, "Batch performance requirement not met");
    }
}

// ===== MEMORY OPTIMIZATION =====
impl Drop for FortitudeClassifier {
    fn drop(&mut self) {
        // Explicit cleanup for large rule sets
        self.rules.clear();
    }
}

// ===== USAGE EXAMPLE =====
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize classifier with custom settings
    let classifier = FortitudeClassifier::new()
        .with_confidence_threshold(0.7)
        .with_fallback_strategy(FallbackStrategy::HighestScore);

    // Example classifications
    let test_queries = vec![
        "Should I use PostgreSQL or MongoDB for my project?",
        "How to implement OAuth authentication in Rust",
        "Getting compilation errors in my Rust code",
        "Explain how neural networks work",
        "Test the accuracy of my model",
    ];

    for query in test_queries {
        match classifier.classify(&query) {
            Ok(result) => {
                println!("Query: {}", query);
                println!("Type: {}, Confidence: {:.2}, Fallback: {}", 
                        result.research_type, result.confidence, result.is_fallback);
                println!("Matched keywords: {:?}\n", result.matched_keywords);
            }
            Err(e) => {
                println!("Classification error: {}", e);
            }
        }
    }

    Ok(())
}

// Dependencies for Cargo.toml:
/*
[dependencies]
regex = "1.10"
serde = { version = "1.0", features = ["derive"] }
*/