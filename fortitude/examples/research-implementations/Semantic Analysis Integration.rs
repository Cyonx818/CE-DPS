use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use serde::{Deserialize, Serialize};
use tokenizers::Tokenizer;

// Custom error types for semantic analysis
#[derive(Debug)]
pub enum SemanticError {
    TokenizationError(String),
    ModelError(String),
    InvalidInput(String),
    ConfigurationError(String),
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SemanticError::TokenizationError(msg) => write!(f, "Tokenization error: {}", msg),
            SemanticError::ModelError(msg) => write!(f, "Model error: {}", msg),
            SemanticError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            SemanticError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl Error for SemanticError {}

// Multi-dimensional classification results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub audience_level: AudienceLevel,
    pub domain_context: DomainContext,
    pub confidence_scores: HashMap<String, f32>,
    pub semantic_features: Vec<SemanticFeature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudienceLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainContext {
    Technical,
    Business,
    Academic,
    Creative,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFeature {
    pub feature_type: String,
    pub value: f32,
    pub importance: f32,
}

// Core semantic analyzer structure
pub struct SemanticAnalyzer {
    tokenizer: Option<Tokenizer>,
    keyword_weights: HashMap<String, f32>,
    domain_patterns: HashMap<DomainContext, Vec<String>>,
    audience_indicators: HashMap<AudienceLevel, Vec<String>>,
    config: AnalyzerConfig,
}

#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    pub min_confidence_threshold: f32,
    pub max_tokens: usize,
    pub enable_context_window: bool,
    pub context_window_size: usize,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            min_confidence_threshold: 0.6,
            max_tokens: 512,
            enable_context_window: true,
            context_window_size: 50,
        }
    }
}

impl SemanticAnalyzer {
    pub fn new(config: AnalyzerConfig) -> Result<Self, SemanticError> {
        let mut analyzer = Self {
            tokenizer: None,
            keyword_weights: HashMap::new(),
            domain_patterns: HashMap::new(),
            audience_indicators: HashMap::new(),
            config,
        };
        
        analyzer.initialize_patterns()?;
        Ok(analyzer)
    }

    pub fn with_tokenizer(mut self, tokenizer_path: &str) -> Result<Self, SemanticError> {
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| SemanticError::ConfigurationError(
                format!("Failed to load tokenizer: {}", e)
            ))?;
        
        self.tokenizer = Some(tokenizer);
        Ok(self)
    }

    fn initialize_patterns(&mut self) -> Result<(), SemanticError> {
        // Initialize domain context patterns
        self.domain_patterns.insert(
            DomainContext::Technical,
            vec![
                "algorithm".to_string(), "implementation".to_string(), "optimization".to_string(),
                "architecture".to_string(), "performance".to_string(), "scalability".to_string(),
                "api".to_string(), "framework".to_string(), "library".to_string(),
            ]
        );

        self.domain_patterns.insert(
            DomainContext::Business,
            vec![
                "strategy".to_string(), "revenue".to_string(), "market".to_string(),
                "customer".to_string(), "roi".to_string(), "growth".to_string(),
                "stakeholder".to_string(), "budget".to_string(), "analytics".to_string(),
            ]
        );

        self.domain_patterns.insert(
            DomainContext::Academic,
            vec![
                "research".to_string(), "study".to_string(), "analysis".to_string(),
                "methodology".to_string(), "hypothesis".to_string(), "peer-reviewed".to_string(),
                "literature".to_string(), "experiment".to_string(), "theory".to_string(),
            ]
        );

        self.domain_patterns.insert(
            DomainContext::Creative,
            vec![
                "design".to_string(), "artistic".to_string(), "creative".to_string(),
                "aesthetic".to_string(), "visual".to_string(), "narrative".to_string(),
                "storytelling".to_string(), "inspiration".to_string(), "imagination".to_string(),
            ]
        );

        // Initialize audience level indicators
        self.audience_indicators.insert(
            AudienceLevel::Beginner,
            vec![
                "basic".to_string(), "introduction".to_string(), "simple".to_string(),
                "tutorial".to_string(), "getting started".to_string(), "fundamentals".to_string(),
                "explained".to_string(), "guide".to_string(), "beginner".to_string(),
            ]
        );

        self.audience_indicators.insert(
            AudienceLevel::Intermediate,
            vec![
                "intermediate".to_string(), "practical".to_string(), "hands-on".to_string(),
                "examples".to_string(), "application".to_string(), "usage".to_string(),
                "implementation".to_string(), "techniques".to_string(),
            ]
        );

        self.audience_indicators.insert(
            AudienceLevel::Advanced,
            vec![
                "advanced".to_string(), "optimization".to_string(), "performance".to_string(),
                "architecture".to_string(), "scalable".to_string(), "enterprise".to_string(),
                "complex".to_string(), "sophisticated".to_string(),
            ]
        );

        self.audience_indicators.insert(
            AudienceLevel::Expert,
            vec![
                "research".to_string(), "novel".to_string(), "cutting-edge".to_string(),
                "theoretical".to_string(), "experimental".to_string(), "breakthrough".to_string(),
                "innovative".to_string(), "state-of-the-art".to_string(),
            ]
        );

        Ok(())
    }

    pub async fn analyze(&self, text: &str) -> Result<ClassificationResult, SemanticError> {
        if text.trim().is_empty() {
            return Err(SemanticError::InvalidInput("Empty input text".to_string()));
        }

        let preprocessed_text = self.preprocess_text(text)?;
        let tokens = self.tokenize(&preprocessed_text)?;
        let semantic_features = self.extract_semantic_features(&tokens, text)?;
        
        let audience_level = self.classify_audience_level(&tokens, &semantic_features)?;
        let domain_context = self.classify_domain_context(&tokens, &semantic_features)?;
        let confidence_scores = self.calculate_confidence_scores(&tokens, &semantic_features)?;

        Ok(ClassificationResult {
            audience_level,
            domain_context,
            confidence_scores,
            semantic_features,
        })
    }

    fn preprocess_text(&self, text: &str) -> Result<String, SemanticError> {
        let cleaned = text
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
            .collect::<String>();

        if cleaned.len() > self.config.max_tokens * 4 {
            return Err(SemanticError::InvalidInput(
                "Input text exceeds maximum token limit".to_string()
            ));
        }

        Ok(cleaned)
    }

    fn tokenize(&self, text: &str) -> Result<Vec<String>, SemanticError> {
        match &self.tokenizer {
            Some(tokenizer) => {
                let encoding = tokenizer.encode(text, false)
                    .map_err(|e| SemanticError::TokenizationError(e.to_string()))?;
                
                let tokens = encoding.get_tokens()
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                
                Ok(tokens)
            }
            None => {
                // Fallback to simple whitespace tokenization
                Ok(text.split_whitespace()
                    .map(|s| s.to_string())
                    .collect())
            }
        }
    }

    fn extract_semantic_features(&self, tokens: &[String], original_text: &str) -> Result<Vec<SemanticFeature>, SemanticError> {
        let mut features = Vec::new();

        // Text length feature
        features.push(SemanticFeature {
            feature_type: "text_length".to_string(),
            value: original_text.len() as f32 / 1000.0,
            importance: 0.3,
        });

        // Token count feature
        features.push(SemanticFeature {
            feature_type: "token_count".to_string(),
            value: tokens.len() as f32 / 100.0,
            importance: 0.4,
        });

        // Average word length
        let avg_word_length = if !tokens.is_empty() {
            tokens.iter().map(|t| t.len()).sum::<usize>() as f32 / tokens.len() as f32
        } else {
            0.0
        };

        features.push(SemanticFeature {
            feature_type: "avg_word_length".to_string(),
            value: avg_word_length / 10.0,
            importance: 0.5,
        });

        // Technical term density
        let technical_terms = ["algorithm", "implementation", "optimization", "architecture"];
        let technical_count = tokens.iter()
            .filter(|token| technical_terms.iter().any(|&term| token.contains(term)))
            .count();

        features.push(SemanticFeature {
            feature_type: "technical_density".to_string(),
            value: if !tokens.is_empty() { 
                technical_count as f32 / tokens.len() as f32 
            } else { 
                0.0 
            },
            importance: 0.8,
        });

        // Question indicator
        let question_indicators = original_text.matches('?').count();
        features.push(SemanticFeature {
            feature_type: "question_density".to_string(),
            value: question_indicators as f32 / (original_text.len() as f32 / 100.0),
            importance: 0.6,
        });

        Ok(features)
    }

    fn classify_audience_level(&self, tokens: &[String], features: &[SemanticFeature]) -> Result<AudienceLevel, SemanticError> {
        let mut scores = HashMap::new();

        for (level, indicators) in &self.audience_indicators {
            let mut score = 0.0f32;
            
            for token in tokens {
                for indicator in indicators {
                    if token.contains(indicator) {
                        score += 1.0;
                    }
                }
            }

            // Apply semantic feature weighting
            let technical_density = features.iter()
                .find(|f| f.feature_type == "technical_density")
                .map(|f| f.value)
                .unwrap_or(0.0);

            let adjusted_score = match level {
                AudienceLevel::Beginner => score * (1.0 - technical_density),
                AudienceLevel::Intermediate => score * (0.8 + technical_density * 0.4),
                AudienceLevel::Advanced => score * (0.6 + technical_density * 0.6),
                AudienceLevel::Expert => score * (0.4 + technical_density * 0.8),
            };

            scores.insert(level.clone(), adjusted_score);
        }

        let best_level = scores.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(level, _)| level.clone())
            .unwrap_or(AudienceLevel::General);

        Ok(best_level)
    }

    fn classify_domain_context(&self, tokens: &[String], _features: &[SemanticFeature]) -> Result<DomainContext, SemanticError> {
        let mut scores = HashMap::new();

        for (domain, patterns) in &self.domain_patterns {
            let mut score = 0.0f32;
            
            for token in tokens {
                for pattern in patterns {
                    if token.contains(pattern) {
                        score += 1.0;
                    }
                }
            }

            scores.insert(domain.clone(), score);
        }

        let best_domain = scores.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(domain, _)| domain.clone())
            .unwrap_or(DomainContext::General);

        Ok(best_domain)
    }

    fn calculate_confidence_scores(&self, tokens: &[String], features: &[SemanticFeature]) -> Result<HashMap<String, f32>, SemanticError> {
        let mut confidence_scores = HashMap::new();

        // Base confidence from token count
        let token_confidence = if tokens.len() > 10 { 0.8 } else { 0.5 };
        confidence_scores.insert("token_count".to_string(), token_confidence);

        // Feature-based confidence
        let feature_confidence = features.iter()
            .map(|f| f.importance * f.value.min(1.0))
            .sum::<f32>() / features.len() as f32;
        
        confidence_scores.insert("feature_analysis".to_string(), feature_confidence);

        // Overall confidence
        let overall_confidence = (token_confidence + feature_confidence) / 2.0;
        confidence_scores.insert("overall".to_string(), overall_confidence);

        Ok(confidence_scores)
    }

    pub fn batch_analyze(&self, texts: Vec<&str>) -> Result<Vec<ClassificationResult>, SemanticError> {
        let mut results = Vec::new();
        
        for text in texts {
            match futures::executor::block_on(self.analyze(text)) {
                Ok(result) => results.push(result),
                Err(e) => return Err(e),
            }
        }

        Ok(results)
    }

    pub fn update_domain_patterns(&mut self, domain: DomainContext, patterns: Vec<String>) {
        self.domain_patterns.insert(domain, patterns);
    }

    pub fn add_keyword_weight(&mut self, keyword: String, weight: f32) {
        self.keyword_weights.insert(keyword, weight);
    }
}

// Integration pipeline for extending existing keyword classification
pub struct ClassificationPipeline {
    semantic_analyzer: SemanticAnalyzer,
    keyword_classifier: KeywordClassifier,
}

pub struct KeywordClassifier {
    keywords: HashMap<String, Vec<String>>,
}

impl KeywordClassifier {
    pub fn new() -> Self {
        let mut keywords = HashMap::new();
        
        keywords.insert("programming".to_string(), vec![
            "code".to_string(), "function".to_string(), "variable".to_string(),
            "loop".to_string(), "condition".to_string(),
        ]);

        Self { keywords }
    }

    pub fn classify(&self, text: &str) -> Vec<String> {
        let mut matches = Vec::new();
        let text_lower = text.to_lowercase();

        for (category, terms) in &self.keywords {
            for term in terms {
                if text_lower.contains(term) {
                    matches.push(category.clone());
                    break;
                }
            }
        }

        matches
    }
}

impl ClassificationPipeline {
    pub fn new(config: AnalyzerConfig) -> Result<Self, SemanticError> {
        let semantic_analyzer = SemanticAnalyzer::new(config)?;
        let keyword_classifier = KeywordClassifier::new();

        Ok(Self {
            semantic_analyzer,
            keyword_classifier,
        })
    }

    pub async fn classify_with_context(&self, text: &str) -> Result<EnhancedClassificationResult, SemanticError> {
        let keyword_results = self.keyword_classifier.classify(text);
        let semantic_results = self.semantic_analyzer.analyze(text).await?;

        Ok(EnhancedClassificationResult {
            keyword_matches: keyword_results,
            semantic_analysis: semantic_results,
            integration_confidence: self.calculate_integration_confidence(text)?,
        })
    }

    fn calculate_integration_confidence(&self, text: &str) -> Result<f32, SemanticError> {
        let keyword_count = self.keyword_classifier.classify(text).len() as f32;
        let text_length_factor = (text.len() as f32 / 100.0).min(1.0);
        
        Ok((keyword_count * 0.3 + text_length_factor * 0.7).min(1.0))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedClassificationResult {
    pub keyword_matches: Vec<String>,
    pub semantic_analysis: ClassificationResult,
    pub integration_confidence: f32,
}

// Example usage and testing
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_semantic_analysis() -> Result<(), SemanticError> {
        let config = AnalyzerConfig::default();
        let analyzer = SemanticAnalyzer::new(config)?;

        let test_text = "I need help implementing a complex machine learning algorithm 
                         for advanced users with deep technical expertise in optimization";

        let result = analyzer.analyze(test_text).await?;

        println!("Classification result: {:?}", result);
        assert!(matches!(result.audience_level, AudienceLevel::Advanced | AudienceLevel::Expert));
        assert!(matches!(result.domain_context, DomainContext::Technical));

        Ok(())
    }

    #[tokio::test]
    async fn test_integration_pipeline() -> Result<(), SemanticError> {
        let config = AnalyzerConfig::default();
        let pipeline = ClassificationPipeline::new(config)?;

        let test_text = "How do I write a simple function in Python for beginners?";
        let result = pipeline.classify_with_context(test_text).await?;

        println!("Enhanced result: {:?}", result);
        assert!(!result.keyword_matches.is_empty());
        assert!(result.integration_confidence > 0.0);

        Ok(())
    }
}

// Example integration with async runtime
pub async fn example_usage() -> Result<(), Box<dyn Error>> {
    let config = AnalyzerConfig {
        min_confidence_threshold: 0.7,
        max_tokens: 256,
        enable_context_window: true,
        context_window_size: 30,
    };

    let pipeline = ClassificationPipeline::new(config)?;

    let queries = vec![
        "How to implement advanced neural networks for research purposes?",
        "Simple tutorial on variables for programming beginners",
        "Market analysis for our Q4 business strategy presentation",
        "Creative writing techniques for narrative storytelling",
    ];

    for query in queries {
        println!("Analyzing: {}", query);
        let result = pipeline.classify_with_context(query).await?;
        
        println!("  Audience: {:?}", result.semantic_analysis.audience_level);
        println!("  Domain: {:?}", result.semantic_analysis.domain_context);
        println!("  Keywords: {:?}", result.keyword_matches);
        println!("  Confidence: {:.2}", result.integration_confidence);
        println!();
    }

    Ok(())
}