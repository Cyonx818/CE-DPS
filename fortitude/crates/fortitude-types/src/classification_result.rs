// ABOUTME: Enhanced classification result types for multi-dimensional classification
use crate::{classification::ClassificationCandidate, research::ResearchType};
use serde::{Deserialize, Serialize};

/// Multi-dimensional classification dimensions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ClassificationDimension {
    /// Research type classification (Decision, Implementation, etc.)
    ResearchType,
    /// Audience level classification (Beginner, Intermediate, Advanced)
    AudienceLevel,
    /// Technical domain classification (Rust, Web, DevOps, etc.)
    TechnicalDomain,
    /// Urgency classification (Immediate, Planned, Exploratory)
    Urgency,
}

impl ClassificationDimension {
    /// Get all classification dimensions
    pub fn all() -> Vec<Self> {
        vec![
            Self::ResearchType,
            Self::AudienceLevel,
            Self::TechnicalDomain,
            Self::Urgency,
        ]
    }

    /// Get display name for the dimension
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ResearchType => "Research Type",
            Self::AudienceLevel => "Audience Level",
            Self::TechnicalDomain => "Technical Domain",
            Self::Urgency => "Urgency",
        }
    }
}

impl std::fmt::Display for ClassificationDimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Audience level classifications
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AudienceLevel {
    /// Beginner level - introductory explanations needed
    Beginner,
    /// Intermediate level - some background assumed
    Intermediate,
    /// Advanced level - technical depth expected
    Advanced,
}

impl AudienceLevel {
    /// Get all audience levels
    pub fn all() -> Vec<Self> {
        vec![Self::Beginner, Self::Intermediate, Self::Advanced]
    }

    /// Get display name for the level
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Beginner => "Beginner",
            Self::Intermediate => "Intermediate",
            Self::Advanced => "Advanced",
        }
    }
}

impl std::fmt::Display for AudienceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl Default for AudienceLevel {
    fn default() -> Self {
        Self::Intermediate
    }
}

/// Technical domain classifications
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TechnicalDomain {
    /// Rust programming language and ecosystem
    Rust,
    /// Python programming language and ecosystem
    Python,
    /// Web development (frontend, backend, full-stack)
    Web,
    /// DevOps, infrastructure, and deployment
    DevOps,
    /// Artificial intelligence and machine learning
    AI,
    /// Database design and management
    Database,
    /// System programming and performance
    Systems,
    /// Security and cryptography
    Security,
    /// Software architecture and design patterns
    Architecture,
    /// General programming concepts
    General,
}

impl TechnicalDomain {
    /// Get all technical domains
    pub fn all() -> Vec<Self> {
        vec![
            Self::Rust,
            Self::Python,
            Self::Web,
            Self::DevOps,
            Self::AI,
            Self::Database,
            Self::Systems,
            Self::Security,
            Self::Architecture,
            Self::General,
        ]
    }

    /// Get display name for the domain
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::Python => "Python",
            Self::Web => "Web Development",
            Self::DevOps => "DevOps",
            Self::AI => "AI/ML",
            Self::Database => "Database",
            Self::Systems => "Systems",
            Self::Security => "Security",
            Self::Architecture => "Architecture",
            Self::General => "General",
        }
    }
}

impl std::fmt::Display for TechnicalDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl Default for TechnicalDomain {
    fn default() -> Self {
        Self::General
    }
}

/// Urgency level classifications
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum UrgencyLevel {
    /// Immediate - blocking current work
    Immediate,
    /// Planned - scheduled for near-term implementation
    Planned,
    /// Exploratory - research for future consideration
    Exploratory,
}

impl UrgencyLevel {
    /// Get all urgency levels
    pub fn all() -> Vec<Self> {
        vec![Self::Immediate, Self::Planned, Self::Exploratory]
    }

    /// Get display name for the level
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Immediate => "Immediate",
            Self::Planned => "Planned",
            Self::Exploratory => "Exploratory",
        }
    }
}

impl std::fmt::Display for UrgencyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl Default for UrgencyLevel {
    fn default() -> Self {
        Self::Planned
    }
}

/// Classification confidence score for a specific dimension
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionConfidence {
    /// The classification dimension
    pub dimension: ClassificationDimension,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Matched keywords for this dimension
    pub matched_keywords: Vec<String>,
    /// Reason for this classification
    pub reasoning: String,
}

impl DimensionConfidence {
    /// Create a new dimension confidence
    pub fn new(
        dimension: ClassificationDimension,
        confidence: f64,
        matched_keywords: Vec<String>,
        reasoning: String,
    ) -> Self {
        Self {
            dimension,
            confidence,
            matched_keywords,
            reasoning,
        }
    }

    /// Check if confidence meets a threshold
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.confidence >= threshold
    }
}

/// Enhanced classification result with multi-dimensional analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnhancedClassificationResult {
    /// Primary research type classification
    pub research_type: ResearchType,
    /// Overall confidence score (0.0-1.0)
    pub overall_confidence: f64,
    /// Detected audience level
    pub audience_level: AudienceLevel,
    /// Detected technical domain
    pub technical_domain: TechnicalDomain,
    /// Detected urgency level
    pub urgency_level: UrgencyLevel,
    /// Confidence scores for each dimension
    pub dimension_confidences: Vec<DimensionConfidence>,
    /// All keywords that influenced the classification
    pub matched_keywords: Vec<String>,
    /// Rule priority that was applied
    pub rule_priority: u32,
    /// All candidate results considered
    pub candidates: Vec<ClassificationCandidate>,
    /// Classification metadata
    pub metadata: ClassificationMetadata,
}

impl EnhancedClassificationResult {
    /// Create a new enhanced classification result
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        research_type: ResearchType,
        overall_confidence: f64,
        audience_level: AudienceLevel,
        technical_domain: TechnicalDomain,
        urgency_level: UrgencyLevel,
        dimension_confidences: Vec<DimensionConfidence>,
        matched_keywords: Vec<String>,
        rule_priority: u32,
        candidates: Vec<ClassificationCandidate>,
        metadata: ClassificationMetadata,
    ) -> Self {
        Self {
            research_type,
            overall_confidence,
            audience_level,
            technical_domain,
            urgency_level,
            dimension_confidences,
            matched_keywords,
            rule_priority,
            candidates,
            metadata,
        }
    }

    /// Check if overall confidence meets threshold
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.overall_confidence >= threshold
    }

    /// Get confidence for a specific dimension
    pub fn get_dimension_confidence(&self, dimension: &ClassificationDimension) -> Option<f64> {
        self.dimension_confidences
            .iter()
            .find(|dc| dc.dimension == *dimension)
            .map(|dc| dc.confidence)
    }

    /// Get matched keywords for a specific dimension
    pub fn get_dimension_keywords(&self, dimension: &ClassificationDimension) -> Vec<String> {
        self.dimension_confidences
            .iter()
            .find(|dc| dc.dimension == *dimension)
            .map(|dc| dc.matched_keywords.clone())
            .unwrap_or_default()
    }

    /// Check if any dimension meets the confidence threshold
    pub fn has_confident_dimension(&self, threshold: f64) -> bool {
        self.dimension_confidences
            .iter()
            .any(|dc| dc.confidence >= threshold)
    }

    /// Get the most confident dimension
    pub fn most_confident_dimension(&self) -> Option<&DimensionConfidence> {
        self.dimension_confidences.iter().max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

/// Metadata for classification operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassificationMetadata {
    /// Time taken for classification in milliseconds
    pub processing_time_ms: u64,
    /// Classification algorithm used
    pub algorithm: String,
    /// Version of the classifier
    pub classifier_version: String,
    /// Whether fallback was used
    pub fallback_used: bool,
    /// Additional context tags
    pub tags: std::collections::HashMap<String, String>,
}

impl Default for ClassificationMetadata {
    fn default() -> Self {
        Self {
            processing_time_ms: 0,
            algorithm: "advanced".to_string(),
            classifier_version: "1.0.0".to_string(),
            fallback_used: false,
            tags: std::collections::HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_dimension_display() {
        assert_eq!(
            ClassificationDimension::ResearchType.display_name(),
            "Research Type"
        );
        assert_eq!(
            ClassificationDimension::AudienceLevel.display_name(),
            "Audience Level"
        );
        assert_eq!(
            ClassificationDimension::TechnicalDomain.display_name(),
            "Technical Domain"
        );
        assert_eq!(ClassificationDimension::Urgency.display_name(), "Urgency");
    }

    #[test]
    fn test_audience_level_all() {
        let levels = AudienceLevel::all();
        assert_eq!(levels.len(), 3);
        assert!(levels.contains(&AudienceLevel::Beginner));
        assert!(levels.contains(&AudienceLevel::Intermediate));
        assert!(levels.contains(&AudienceLevel::Advanced));
    }

    #[test]
    fn test_technical_domain_all() {
        let domains = TechnicalDomain::all();
        assert_eq!(domains.len(), 8);
        assert!(domains.contains(&TechnicalDomain::Rust));
        assert!(domains.contains(&TechnicalDomain::Web));
        assert!(domains.contains(&TechnicalDomain::General));
    }

    #[test]
    fn test_urgency_level_all() {
        let levels = UrgencyLevel::all();
        assert_eq!(levels.len(), 3);
        assert!(levels.contains(&UrgencyLevel::Immediate));
        assert!(levels.contains(&UrgencyLevel::Planned));
        assert!(levels.contains(&UrgencyLevel::Exploratory));
    }

    #[test]
    fn test_dimension_confidence() {
        let confidence = DimensionConfidence::new(
            ClassificationDimension::AudienceLevel,
            0.85,
            vec!["beginner".to_string()],
            "Detected beginner-level language".to_string(),
        );

        assert_eq!(confidence.dimension, ClassificationDimension::AudienceLevel);
        assert_eq!(confidence.confidence, 0.85);
        assert!(confidence.meets_threshold(0.8));
        assert!(!confidence.meets_threshold(0.9));
    }

    #[test]
    fn test_enhanced_classification_result() {
        let dimension_confidences = vec![
            DimensionConfidence::new(
                ClassificationDimension::ResearchType,
                0.9,
                vec!["implement".to_string()],
                "Strong implementation keywords".to_string(),
            ),
            DimensionConfidence::new(
                ClassificationDimension::TechnicalDomain,
                0.8,
                vec!["rust".to_string()],
                "Rust-specific terminology".to_string(),
            ),
        ];

        let result = EnhancedClassificationResult::new(
            ResearchType::Implementation,
            0.85,
            AudienceLevel::Intermediate,
            TechnicalDomain::Rust,
            UrgencyLevel::Planned,
            dimension_confidences,
            vec!["implement".to_string(), "rust".to_string()],
            1,
            vec![],
            ClassificationMetadata::default(),
        );

        assert_eq!(result.research_type, ResearchType::Implementation);
        assert_eq!(result.overall_confidence, 0.85);
        assert_eq!(result.technical_domain, TechnicalDomain::Rust);
        assert!(result.meets_threshold(0.8));
        assert!(!result.meets_threshold(0.9));

        // Test dimension confidence access
        assert_eq!(
            result.get_dimension_confidence(&ClassificationDimension::ResearchType),
            Some(0.9)
        );
        assert_eq!(
            result.get_dimension_confidence(&ClassificationDimension::AudienceLevel),
            None
        );

        // Test most confident dimension
        let most_confident = result.most_confident_dimension().unwrap();
        assert_eq!(
            most_confident.dimension,
            ClassificationDimension::ResearchType
        );
        assert_eq!(most_confident.confidence, 0.9);
    }

    #[test]
    fn test_classification_metadata_default() {
        let metadata = ClassificationMetadata::default();
        assert_eq!(metadata.algorithm, "advanced");
        assert_eq!(metadata.classifier_version, "1.0.0");
        assert!(!metadata.fallback_used);
        assert!(metadata.tags.is_empty());
    }
}
