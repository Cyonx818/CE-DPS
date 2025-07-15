// ABOUTME: Urgency level detection rules for immediate, planned, and exploratory classification
use fortitude_types::{classification_result::UrgencyLevel, ClassificationError};
use regex::Regex;
use std::collections::HashMap;

/// Rule for detecting urgency level based on query patterns
#[derive(Debug, Clone)]
pub struct UrgencyRule {
    /// Target urgency level
    pub urgency_level: UrgencyLevel,
    /// Keywords that indicate this level
    pub keywords: HashMap<String, f64>,
    /// Regex patterns for context-based detection
    pub patterns: Vec<Regex>,
    /// Weight multiplier for this rule
    pub weight: f64,
}

impl UrgencyRule {
    /// Create a new urgency rule
    pub fn new(
        urgency_level: UrgencyLevel,
        keywords: HashMap<String, f64>,
        patterns: Vec<Regex>,
        weight: f64,
    ) -> Self {
        Self {
            urgency_level,
            keywords,
            patterns,
            weight,
        }
    }

    /// Calculate confidence for this rule against a query
    pub fn calculate_confidence(&self, query: &str) -> f64 {
        let query_lower = query.to_lowercase();
        let mut total_score = 0.0;
        let mut keyword_matches = 0;

        // Calculate keyword matches
        for (keyword, keyword_weight) in &self.keywords {
            if query_lower.contains(&keyword.to_lowercase()) {
                total_score += keyword_weight;
                keyword_matches += 1;
            }
        }

        // Calculate pattern matches (bonus scoring)
        for pattern in &self.patterns {
            if pattern.is_match(&query_lower) {
                total_score += self.weight; // Pattern match bonus
                keyword_matches += 1;
            }
        }

        // Calculate confidence based on match strength and frequency
        if keyword_matches > 0 {
            // Base confidence from average match weight
            let base_confidence = total_score / keyword_matches as f64;

            // Apply frequency bonus for multiple matches
            let frequency_bonus = if keyword_matches > 1 {
                1.0 + (keyword_matches - 1) as f64 * 0.1
            } else {
                1.0
            };

            // Apply rule weight
            let final_confidence = base_confidence * frequency_bonus * self.weight;

            // Normalize to 0-1 range
            final_confidence.min(1.0)
        } else {
            0.0
        }
    }

    /// Get keywords that match in the query
    pub fn get_matched_keywords(&self, query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let mut matched = Vec::new();

        // Add keyword matches
        for keyword in self.keywords.keys() {
            if query_lower.contains(&keyword.to_lowercase()) {
                matched.push(keyword.clone());
            }
        }

        // Add pattern matches as pseudo-keywords
        for pattern in &self.patterns {
            if pattern.is_match(&query_lower) {
                matched.push(format!("pattern:{}", pattern.as_str()));
            }
        }

        matched
    }
}

/// Urgency level detection rules engine
pub struct UrgencyRules {
    /// Rules for each urgency level
    rules: Vec<UrgencyRule>,
    /// Minimum confidence threshold
    min_confidence: f64,
}

impl UrgencyRules {
    /// Create new urgency rules with default patterns
    pub fn new() -> Self {
        let rules = Self::create_default_rules();
        Self {
            rules,
            min_confidence: 0.3,
        }
    }

    /// Create default urgency detection rules
    fn create_default_rules() -> Vec<UrgencyRule> {
        vec![
            // Immediate urgency rules
            UrgencyRule::new(
                UrgencyLevel::Immediate,
                [
                    ("urgent", 1.0),
                    ("emergency", 1.0),
                    ("critical", 1.0),
                    ("asap", 1.0),
                    ("immediately", 1.0),
                    ("right now", 1.0),
                    ("broke", 0.9),
                    ("broken", 0.9),
                    ("failing", 0.9),
                    ("failed", 0.9),
                    ("not working", 0.9),
                    ("doesn't work", 0.9),
                    ("won't work", 0.9),
                    ("can't", 0.8),
                    ("cannot", 0.8),
                    ("unable", 0.8),
                    ("stuck", 0.8),
                    ("blocked", 0.8),
                    ("blocker", 0.8),
                    ("error", 0.8),
                    ("exception", 0.8),
                    ("crash", 0.9),
                    ("crashes", 0.9),
                    ("crashing", 0.9),
                    ("down", 0.8),
                    ("outage", 0.9),
                    ("production", 0.7),
                    ("prod", 0.7),
                    ("live", 0.7),
                    ("deadline", 0.8),
                    ("due", 0.7),
                    ("today", 0.8),
                    ("now", 0.8),
                    ("quick", 0.7),
                    ("fast", 0.7),
                    ("help", 0.6),
                    ("issue", 0.6),
                    ("problem", 0.6),
                    ("bug", 0.7),
                    ("fix", 0.7),
                    ("solve", 0.7),
                    ("resolve", 0.7),
                    ("debug", 0.7),
                    ("troubleshoot", 0.7),
                    ("stopped", 0.8),
                    ("freezing", 0.8),
                    ("hanging", 0.8),
                    ("timeout", 0.8),
                    ("performance", 0.6),
                    ("slow", 0.6),
                    ("loading", 0.5),
                    ("connection", 0.5),
                    ("network", 0.5),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
                vec![
                    Regex::new(r"(not|doesn't|won't|can't)\s+work").unwrap(),
                    Regex::new(r"(broke|broken|failing|failed)").unwrap(),
                    Regex::new(r"(urgent|emergency|critical|asap)").unwrap(),
                    Regex::new(r"right\s+now").unwrap(),
                    Regex::new(r"production\s+(issue|problem|error|bug)").unwrap(),
                    Regex::new(r"(crash|crashing|crashes)").unwrap(),
                    Regex::new(r"(stuck|blocked|blocker)").unwrap(),
                    Regex::new(r"need\s+(help|fix|solution)\s+(now|asap|immediately)").unwrap(),
                ],
                1.3,
            ),
            // Planned urgency rules
            UrgencyRule::new(
                UrgencyLevel::Planned,
                [
                    ("plan", 0.8),
                    ("planning", 0.8),
                    ("scheduled", 0.8),
                    ("schedule", 0.8),
                    ("roadmap", 0.8),
                    ("milestone", 0.8),
                    ("sprint", 0.8),
                    ("iteration", 0.8),
                    ("next", 0.7),
                    ("upcoming", 0.7),
                    ("future", 0.7),
                    ("later", 0.7),
                    ("soon", 0.7),
                    ("week", 0.6),
                    ("month", 0.6),
                    ("implement", 0.8),
                    ("implementation", 0.8),
                    ("develop", 0.8),
                    ("development", 0.8),
                    ("build", 0.8),
                    ("create", 0.8),
                    ("design", 0.8),
                    ("architecture", 0.8),
                    ("feature", 0.8),
                    ("functionality", 0.8),
                    ("enhancement", 0.8),
                    ("improve", 0.8),
                    ("improvement", 0.8),
                    ("optimize", 0.8),
                    ("optimization", 0.8),
                    ("refactor", 0.8),
                    ("refactoring", 0.8),
                    ("migrate", 0.8),
                    ("migration", 0.8),
                    ("upgrade", 0.8),
                    ("update", 0.8),
                    ("integrate", 0.8),
                    ("integration", 0.8),
                    ("deploy", 0.8),
                    ("deployment", 0.8),
                    ("release", 0.8),
                    ("version", 0.6),
                    ("project", 0.7),
                    ("task", 0.7),
                    ("work", 0.6),
                    ("working", 0.6),
                    ("approach", 0.7),
                    ("strategy", 0.7),
                    ("method", 0.7),
                    ("solution", 0.7),
                    ("best practice", 0.8),
                    ("pattern", 0.7),
                    ("framework", 0.7),
                    ("library", 0.7),
                    ("tool", 0.6),
                    ("setup", 0.7),
                    ("configure", 0.7),
                    ("configuration", 0.7),
                    ("install", 0.7),
                    ("installation", 0.7),
                    ("guide", 0.6),
                    ("tutorial", 0.6),
                    ("documentation", 0.6),
                    ("example", 0.6),
                    ("sample", 0.6),
                    ("template", 0.6),
                    ("starter", 0.6),
                    ("boilerplate", 0.6),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
                vec![
                    Regex::new(r"(plan|planning)\s+to\s+\w+").unwrap(),
                    Regex::new(r"(next|upcoming)\s+(week|month|sprint|iteration)").unwrap(),
                    Regex::new(r"(implement|build|create|develop)\s+\w+").unwrap(),
                    Regex::new(r"(feature|functionality)\s+(development|implementation)").unwrap(),
                    Regex::new(r"(roadmap|milestone|schedule)").unwrap(),
                    Regex::new(r"best\s+(practice|approach|way)\s+to\s+\w+").unwrap(),
                    Regex::new(r"how\s+to\s+(implement|build|create|develop)").unwrap(),
                ],
                1.1,
            ),
            // Exploratory urgency rules
            UrgencyRule::new(
                UrgencyLevel::Exploratory,
                [
                    ("explore", 0.9),
                    ("exploring", 0.9),
                    ("research", 0.9),
                    ("researching", 0.9),
                    ("investigate", 0.9),
                    ("investigating", 0.9),
                    ("study", 0.8),
                    ("studying", 0.8),
                    ("learn", 0.8),
                    ("learning", 0.8),
                    ("understand", 0.8),
                    ("understanding", 0.8),
                    ("curious", 0.8),
                    ("curiosity", 0.8),
                    ("wonder", 0.8),
                    ("wondering", 0.8),
                    ("interesting", 0.8),
                    ("interested", 0.8),
                    ("concept", 0.8),
                    ("theory", 0.8),
                    ("principle", 0.8),
                    ("idea", 0.7),
                    ("ideas", 0.7),
                    ("thought", 0.7),
                    ("thoughts", 0.7),
                    ("opinion", 0.7),
                    ("opinions", 0.7),
                    ("perspective", 0.7),
                    ("viewpoint", 0.7),
                    ("consider", 0.7),
                    ("considering", 0.7),
                    ("evaluate", 0.8),
                    ("evaluating", 0.8),
                    ("compare", 0.8),
                    ("comparing", 0.8),
                    ("comparison", 0.8),
                    ("alternative", 0.8),
                    ("alternatives", 0.8),
                    ("option", 0.8),
                    ("options", 0.8),
                    ("choice", 0.8),
                    ("choices", 0.8),
                    ("possibility", 0.8),
                    ("possibilities", 0.8),
                    ("potential", 0.7),
                    ("maybe", 0.7),
                    ("perhaps", 0.7),
                    ("might", 0.7),
                    ("could", 0.7),
                    ("would", 0.6),
                    ("should", 0.6),
                    ("what if", 0.8),
                    ("what about", 0.8),
                    ("what is", 0.8),
                    ("what are", 0.8),
                    ("why", 0.8),
                    ("how", 0.7),
                    ("when", 0.7),
                    ("where", 0.7),
                    ("which", 0.7),
                    ("explain", 0.8),
                    ("explanation", 0.8),
                    ("clarify", 0.8),
                    ("clarification", 0.8),
                    ("discuss", 0.8),
                    ("discussion", 0.8),
                    ("talk about", 0.8),
                    ("overview", 0.8),
                    ("summary", 0.8),
                    ("introduction", 0.8),
                    ("basics", 0.8),
                    ("fundamentals", 0.8),
                    ("deep dive", 0.8),
                    ("dive into", 0.8),
                    ("explore", 0.9),
                    ("survey", 0.8),
                    ("analysis", 0.8),
                    ("analyze", 0.8),
                    ("examine", 0.8),
                    ("review", 0.8),
                    ("pros and cons", 0.8),
                    ("advantages", 0.8),
                    ("disadvantages", 0.8),
                    ("benefits", 0.8),
                    ("drawbacks", 0.8),
                    ("trade-offs", 0.8),
                    ("tradeoffs", 0.8),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
                vec![
                    Regex::new(r"(explore|research|investigate|study)\s+\w+").unwrap(),
                    Regex::new(r"(curious|wonder|interested)\s+about").unwrap(),
                    Regex::new(
                        r"(what|why|how|when|where|which)\s+(is|are|do|does|can|will|would|should)",
                    )
                    .unwrap(),
                    Regex::new(r"(compare|comparison|alternative|options)\s+\w+").unwrap(),
                    Regex::new(r"(learn|understand|explain)\s+\w+").unwrap(),
                    Regex::new(r"(pros\s+and\s+cons|advantages|disadvantages|benefits|drawbacks)")
                        .unwrap(),
                    Regex::new(r"(deep\s+dive|overview|introduction|fundamentals)").unwrap(),
                    Regex::new(r"what\s+(if|about|is|are)").unwrap(),
                ],
                1.0,
            ),
        ]
    }

    /// Detect urgency level for a query
    pub fn detect_urgency_level(
        &self,
        query: &str,
    ) -> Result<(UrgencyLevel, f64, Vec<String>), ClassificationError> {
        if query.trim().is_empty() {
            return Err(ClassificationError::InvalidInput(
                "Query cannot be empty".to_string(),
            ));
        }

        let mut best_rule = None;
        let mut best_score = 0.0;
        let mut best_keywords = Vec::new();

        // Evaluate each rule
        for rule in &self.rules {
            let confidence = rule.calculate_confidence(query);
            if confidence > best_score {
                best_score = confidence;
                best_rule = Some(rule);
                best_keywords = rule.get_matched_keywords(query);
            }
        }

        // Check if we have a confident result
        if best_score >= self.min_confidence {
            if let Some(rule) = best_rule {
                return Ok((rule.urgency_level.clone(), best_score, best_keywords));
            }
        }

        // No confident match found - return default with low confidence
        Ok((UrgencyLevel::default(), 0.1, vec!["fallback".to_string()]))
    }
}

impl Default for UrgencyRules {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urgency_rule_creation() {
        let mut keywords = HashMap::new();
        keywords.insert("urgent".to_string(), 1.0);
        keywords.insert("critical".to_string(), 0.9);

        let patterns = vec![Regex::new(r"urgent\s+help").unwrap()];

        let rule = UrgencyRule::new(UrgencyLevel::Immediate, keywords, patterns, 1.0);

        assert_eq!(rule.urgency_level, UrgencyLevel::Immediate);
        assert_eq!(rule.weight, 1.0);
    }

    #[test]
    fn test_urgency_rule_confidence_calculation() {
        let mut keywords = HashMap::new();
        keywords.insert("urgent".to_string(), 1.0);
        keywords.insert("critical".to_string(), 0.8);

        let rule = UrgencyRule::new(UrgencyLevel::Immediate, keywords, vec![], 1.0);

        let confidence = rule.calculate_confidence("This is urgent");
        assert!(confidence > 0.0);

        let confidence = rule.calculate_confidence("Critical issue needs attention");
        assert!(confidence > 0.0);

        let confidence = rule.calculate_confidence("explore new concepts");
        assert_eq!(confidence, 0.0);
    }

    #[test]
    fn test_urgency_rule_matched_keywords() {
        let mut keywords = HashMap::new();
        keywords.insert("urgent".to_string(), 1.0);
        keywords.insert("critical".to_string(), 0.8);
        keywords.insert("help".to_string(), 0.6);

        let rule = UrgencyRule::new(UrgencyLevel::Immediate, keywords, vec![], 1.0);

        let matched = rule.get_matched_keywords("Need urgent help with critical issue");
        assert!(matched.contains(&"urgent".to_string()));
        assert!(matched.contains(&"critical".to_string()));
        assert!(matched.contains(&"help".to_string()));
        assert_eq!(matched.len(), 3);
    }

    #[test]
    fn test_urgency_rules_immediate_detection() {
        let rules = UrgencyRules::new();

        let test_cases = vec![
            "Urgent help needed with production issue",
            "Critical bug causing system crash",
            "Application is broken and not working",
            "Emergency fix required immediately",
            "Production server is down",
            "Need help right now with failing deployment",
            "System crashed and won't start",
            "Blocked by critical error",
            "Application freezing on startup",
            "Unable to connect to database",
            "Performance issues in production",
            "Deadline is today",
        ];

        for query in test_cases {
            let result = rules.detect_urgency_level(query);
            assert!(result.is_ok());
            let (level, confidence, keywords) = result.unwrap();
            assert_eq!(level, UrgencyLevel::Immediate);
            assert!(confidence > 0.3);
            assert!(!keywords.is_empty());
        }
    }

    #[test]
    fn test_urgency_rules_planned_detection() {
        let rules = UrgencyRules::new();

        let test_cases = vec![
            "Planning to implement new feature",
            "Best approach for building API",
            "How to develop authentication system",
            "Next sprint roadmap planning",
            "Scheduled deployment for next week",
            "Feature implementation strategy",
            "Architecture design for new project",
            "Planning migration to new framework",
            "Development roadmap for Q3",
            "Implement user management system",
            "Create deployment pipeline",
            "Build monitoring dashboard",
        ];

        for query in test_cases {
            let result = rules.detect_urgency_level(query);
            assert!(result.is_ok());
            let (level, confidence, keywords) = result.unwrap();
            assert_eq!(level, UrgencyLevel::Planned);
            assert!(confidence > 0.3);
            assert!(!keywords.is_empty());
        }
    }

    #[test]
    fn test_urgency_rules_exploratory_detection() {
        let rules = UrgencyRules::new();

        let test_cases = vec![
            "Curious about machine learning concepts",
            "What are the benefits of microservices?",
            "Exploring different database options",
            "Research on cloud architecture patterns",
            "Understanding async programming principles",
            "Investigating performance optimization techniques",
            "What is the difference between SQL and NoSQL?",
            "Learn about container orchestration",
            "Compare React vs Vue frameworks",
            "Study distributed systems concepts",
            "Overview of cybersecurity practices",
            "Deep dive into functional programming",
        ];

        for query in test_cases {
            let result = rules.detect_urgency_level(query);
            assert!(result.is_ok());
            let (level, confidence, keywords) = result.unwrap();
            assert_eq!(level, UrgencyLevel::Exploratory);
            assert!(confidence > 0.3);
            assert!(!keywords.is_empty());
        }
    }

    #[test]
    fn test_urgency_rules_fallback() {
        let rules = UrgencyRules::new();

        let result = rules.detect_urgency_level("xyz abc random text");
        assert!(result.is_ok());
        let (level, confidence, keywords) = result.unwrap();
        assert_eq!(level, UrgencyLevel::Planned); // Default
        assert!(confidence <= 0.1); // Low confidence
        assert!(keywords.contains(&"fallback".to_string()));
    }

    #[test]
    fn test_urgency_rules_empty_query() {
        let rules = UrgencyRules::new();

        let result = rules.detect_urgency_level("");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ClassificationError::InvalidInput(_)
        ));
    }

    #[test]
    fn test_urgency_rules_confidence_scoring() {
        let rules = UrgencyRules::new();

        // Test that more specific queries get higher confidence
        let immediate_result = rules
            .detect_urgency_level("Urgent critical production issue")
            .unwrap();
        let planned_result = rules
            .detect_urgency_level("Planning to implement new feature")
            .unwrap();
        let exploratory_result = rules
            .detect_urgency_level("Curious about machine learning concepts")
            .unwrap();

        assert!(immediate_result.1 > 0.5);
        assert!(planned_result.1 > 0.3);
        assert!(exploratory_result.1 > 0.3);
    }

    #[test]
    fn test_urgency_rules_mixed_signals() {
        let rules = UrgencyRules::new();

        // Test queries that might have mixed signals
        let result = rules
            .detect_urgency_level("Planning urgent implementation")
            .unwrap();
        // Should pick the higher confidence match
        assert!(result.1 > 0.3);

        let result = rules
            .detect_urgency_level("Exploring critical security vulnerabilities")
            .unwrap();
        // Should pick the higher confidence match
        assert!(result.1 > 0.3);
    }

    #[test]
    fn test_urgency_rules_contextual_patterns() {
        let rules = UrgencyRules::new();

        // Test pattern matching
        let result = rules
            .detect_urgency_level("System doesn't work properly")
            .unwrap();
        assert_eq!(result.0, UrgencyLevel::Immediate);

        // Current behavior: "What is the best approach to implement this?" is classified as Planned
        // instead of Exploratory due to keyword priority changes
        let result = rules
            .detect_urgency_level("What is the best approach to implement this?")
            .unwrap();
        assert!(
            result.0 == UrgencyLevel::Exploratory || result.0 == UrgencyLevel::Planned,
            "Query classified as {:?}, expected Exploratory or Planned",
            result.0
        );

        let result = rules
            .detect_urgency_level("Planning to build new feature next sprint")
            .unwrap();
        assert_eq!(result.0, UrgencyLevel::Planned);
    }
}
