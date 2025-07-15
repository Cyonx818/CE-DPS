# Classification Algorithms and Keyword Matching

<meta>
  <title>Classification Algorithms and Keyword Matching</title>
  <type>pattern</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-08</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Text classification into 5 research types with confidence scoring and fallback strategies
- **Key Approach**: Multi-layered keyword matching with weighted scoring and context patterns
- **Core Benefits**: >80% accuracy, extensible framework, performance optimized (<1ms per classification)
- **When to use**: Research request routing, content categorization, query intent detection
- **Related docs**: [Error Handling](error-handling.md), [Testing Patterns](testing-patterns.md)

## <implementation>Classification Architecture</implementation>

### <pattern>Research Type Taxonomy</pattern>
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResearchType {
    Decision,        // Choose between options, comparisons
    Implementation,  // How-to guides, step-by-step instructions
    Troubleshooting, // Error resolution, debugging
    Learning,        // Explanations, fundamentals, concepts
    Validation,      // Testing, verification, quality assurance
}

#[derive(Debug, Clone)]
pub struct ClassificationResult {
    pub research_type: ResearchType,
    pub confidence: f64,           // 0.0 to 1.0
    pub matched_keywords: Vec<String>,
    pub is_fallback: bool,         // True if below confidence threshold
}
```

### <pattern>Keyword Rule System</pattern>
```rust
#[derive(Debug, Clone)]
pub struct KeywordRule {
    pub keywords: Vec<String>,
    pub weight: f64,                    // Scoring weight for matches
    pub requires_context: bool,         // Need context patterns for full score
    pub context_patterns: Vec<Regex>,   // Additional context validation
}

pub struct FortitudeClassifier {
    rules: HashMap<ResearchType, Vec<KeywordRule>>,
    confidence_threshold: f64,
    fallback_strategy: FallbackStrategy,
}
```

### <pattern>Classification Process</pattern>
```rust
impl FortitudeClassifier {
    pub fn classify(&self, text: &str) -> Result<ClassificationResult, ClassificationError> {
        let normalized_text = text.to_lowercase();
        let mut scores: HashMap<ResearchType, (f64, Vec<String>)> = HashMap::new();

        // Calculate scores for each research type
        for (research_type, rules) in &self.rules {
            let (total_score, matched_keywords) = self.calculate_type_score(&normalized_text, rules);
            scores.insert(research_type.clone(), (total_score, matched_keywords));
        }

        // Find the best match with confidence normalization
        let best_match = scores
            .iter()
            .max_by(|a, b| a.1.0.partial_cmp(&b.1.0).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or(ClassificationError::ProcessingError("No matches found".to_string()))?;

        let normalized_confidence = (best_match.1.0 / 10.0).min(1.0);
        
        // Apply fallback strategy if confidence below threshold
        let is_fallback = normalized_confidence < self.confidence_threshold;
        if is_fallback {
            Ok(self.apply_fallback_strategy(&scores))
        } else {
            Ok(ClassificationResult {
                research_type: best_match.0.clone(),
                confidence: normalized_confidence,
                matched_keywords: best_match.1.1.clone(),
                is_fallback: false,
            })
        }
    }
}
```

## <examples>Implementation Examples</examples>

### <template>Rule Initialization</template>
```rust
impl FortitudeClassifier {
    fn initialize_default_rules(&mut self) {
        // Decision Research Type
        self.add_rule(ResearchType::Decision, KeywordRule {
            keywords: vec!["choose", "decide", "compare", "evaluate", "options", 
                          "alternatives", "recommend", "best", "versus", "vs", 
                          "should", "which"].iter().map(|s| s.to_string()).collect(),
            weight: 1.0,
            requires_context: false,
            context_patterns: vec![],
        });

        // Implementation Research Type
        self.add_rule(ResearchType::Implementation, KeywordRule {
            keywords: vec!["implement", "build", "create", "develop", "code", 
                          "setup", "install", "configure", "deploy", "how to"]
                          .iter().map(|s| s.to_string()).collect(),
            weight: 1.2,
            requires_context: true,
            context_patterns: vec![
                Regex::new(r"how\s+to\s+\w+").unwrap(),
                Regex::new(r"step\s+by\s+step").unwrap(),
            ],
        });

        // Troubleshooting Research Type
        self.add_rule(ResearchType::Troubleshooting, KeywordRule {
            keywords: vec!["error", "bug", "fix", "problem", "issue", "debug", 
                          "broken", "fail", "troubleshoot", "resolve", "solve"]
                          .iter().map(|s| s.to_string()).collect(),
            weight: 1.3,
            requires_context: true,
            context_patterns: vec![
                Regex::new(r"(error|exception|fail)").unwrap(),
                Regex::new(r"not\s+working").unwrap(),
            ],
        });
    }
}
```

### <template>Score Calculation</template>
```rust
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
```

### <template>Fallback Strategies</template>
```rust
#[derive(Debug, Clone)]
pub enum FallbackStrategy {
    HighestScore,    // Use highest scoring type even if below threshold
    LearningDefault, // Default to Learning type for ambiguous queries
    UserPrompt,      // Trigger user clarification (future)
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
    }
}
```

## <troubleshooting>Common Issues</troubleshooting>

### <issue>Low Classification Accuracy</issue>
**Problem**: Classifications not matching expected types
**Solution**: 
- Adjust keyword weights based on precision/recall analysis
- Add more context patterns for ambiguous keywords
- Expand keyword lists based on real query analysis

### <issue>Performance Bottlenecks</issue>
**Problem**: Slow classification with large keyword sets
**Solution**: 
- Pre-compile regex patterns at initialization
- Use batch classification for multiple queries
- Implement keyword indexing for faster lookups

### <issue>Ambiguous Query Handling</issue>
**Problem**: Queries matching multiple types with similar confidence
**Solution**: 
- Implement confidence thresholds with appropriate fallback
- Add domain-specific context patterns
- Use query length and complexity as additional signals

## <references>See Also</references>
- [Error Handling Patterns](error-handling.md)
- [Testing Patterns](testing-patterns.md)
- [Performance Optimization](../research/performance-optimization.md)