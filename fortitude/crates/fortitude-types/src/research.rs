// ABOUTME: Research domain types for the Fortitude research system
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Core research types supported by the Fortitude system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ResearchType {
    /// Research to support decision-making between alternatives
    Decision,
    /// Research to guide implementation of specific features/solutions
    Implementation,
    /// Research to resolve specific problems or bugs
    Troubleshooting,
    /// Research to understand concepts, technologies, or patterns
    Learning,
    /// Research to validate approaches, test quality, or verify assumptions
    Validation,
}

impl ResearchType {
    /// Get all research types as a vec
    pub fn all() -> Vec<Self> {
        vec![
            Self::Decision,
            Self::Implementation,
            Self::Troubleshooting,
            Self::Learning,
            Self::Validation,
        ]
    }

    /// Get display name for the research type
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Decision => "Decision",
            Self::Implementation => "Implementation",
            Self::Troubleshooting => "Troubleshooting",
            Self::Learning => "Learning",
            Self::Validation => "Validation",
        }
    }

    /// Get description for the research type
    pub fn description(&self) -> &'static str {
        match self {
            Self::Decision => "Research to support decision-making between alternatives",
            Self::Implementation => {
                "Research to guide implementation of specific features/solutions"
            }
            Self::Troubleshooting => "Research to resolve specific problems or bugs",
            Self::Learning => "Research to understand concepts, technologies, or patterns",
            Self::Validation => {
                "Research to validate approaches, test quality, or verify assumptions"
            }
        }
    }
}

impl std::fmt::Display for ResearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for ResearchType {
    type Err = crate::error::ResearchError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "decision" => Ok(Self::Decision),
            "implementation" => Ok(Self::Implementation),
            "troubleshooting" => Ok(Self::Troubleshooting),
            "learning" => Ok(Self::Learning),
            "validation" => Ok(Self::Validation),
            _ => Err(crate::error::ResearchError::InvalidType(s.to_string())),
        }
    }
}

/// Context about the intended audience for research results
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudienceContext {
    /// Technical level (beginner, intermediate, advanced)
    pub level: String,
    /// Domain expertise (rust, web, devops, etc.)
    pub domain: String,
    /// Output format preference (markdown, json, plain)
    pub format: String,
}

impl Default for AudienceContext {
    fn default() -> Self {
        Self {
            level: "intermediate".to_string(),
            domain: "general".to_string(),
            format: "markdown".to_string(),
        }
    }
}

/// Domain-specific context for research
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainContext {
    /// Technology stack (rust, javascript, python, etc.)
    pub technology: String,
    /// Project type (web, cli, library, etc.)
    pub project_type: String,
    /// Specific frameworks or libraries
    pub frameworks: Vec<String>,
    /// Additional context tags
    pub tags: Vec<String>,
}

impl Default for DomainContext {
    fn default() -> Self {
        Self {
            technology: "rust".to_string(),
            project_type: "library".to_string(),
            frameworks: vec![],
            tags: vec![],
        }
    }
}

/// Research request with classification and context
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassifiedRequest {
    /// Unique identifier for the request
    pub id: Uuid,
    /// Original query text
    pub original_query: String,
    /// Classified research type
    pub research_type: ResearchType,
    /// Audience context
    pub audience_context: AudienceContext,
    /// Domain context
    pub domain_context: DomainContext,
    /// Classification confidence score (0.0-1.0)
    pub confidence: f64,
    /// Keywords that influenced classification
    pub matched_keywords: Vec<String>,
    /// Timestamp when request was created
    pub created_at: DateTime<Utc>,
    /// Enhanced classification result (optional for backward compatibility)
    pub enhanced_classification:
        Option<Box<crate::classification_result::EnhancedClassificationResult>>,
}

impl ClassifiedRequest {
    /// Create a new classified request
    pub fn new(
        query: String,
        research_type: ResearchType,
        audience_context: AudienceContext,
        domain_context: DomainContext,
        confidence: f64,
        matched_keywords: Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            original_query: query,
            research_type,
            audience_context,
            domain_context,
            confidence,
            matched_keywords,
            created_at: Utc::now(),
            enhanced_classification: None,
        }
    }

    /// Create a new classified request with enhanced classification
    pub fn new_enhanced(
        query: String,
        research_type: ResearchType,
        audience_context: AudienceContext,
        domain_context: DomainContext,
        confidence: f64,
        matched_keywords: Vec<String>,
        enhanced_classification: crate::classification_result::EnhancedClassificationResult,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            original_query: query,
            research_type,
            audience_context,
            domain_context,
            confidence,
            matched_keywords,
            created_at: Utc::now(),
            enhanced_classification: Some(Box::new(enhanced_classification)),
        }
    }

    /// Check if this request has enhanced classification data
    pub fn has_enhanced_classification(&self) -> bool {
        self.enhanced_classification.is_some()
    }

    /// Get the enhanced classification result
    pub fn get_enhanced_classification(
        &self,
    ) -> Option<&crate::classification_result::EnhancedClassificationResult> {
        self.enhanced_classification.as_ref().map(|ec| ec.as_ref())
    }
}

/// Evidence supporting a research result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    /// Source of the evidence
    pub source: String,
    /// Evidence content
    pub content: String,
    /// Relevance score (0.0-1.0)
    pub relevance: f64,
    /// Evidence type (documentation, example, reference)
    pub evidence_type: String,
}

/// Implementation detail for research results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Detail {
    /// Detail category (code, config, setup, etc.)
    pub category: String,
    /// Detail content
    pub content: String,
    /// Priority level (low, medium, high)
    pub priority: String,
    /// Prerequisites for this detail
    pub prerequisites: Vec<String>,
}

/// Metadata about research results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchMetadata {
    /// Research completion timestamp
    pub completed_at: DateTime<Utc>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Sources consulted
    pub sources_consulted: Vec<String>,
    /// Quality score (0.0-1.0)
    pub quality_score: f64,
    /// Cache key for storage
    pub cache_key: String,
    /// Additional metadata tags
    pub tags: HashMap<String, String>,
}

/// Complete research result with progressive disclosure structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchResult {
    /// Request that generated this result
    pub request: ClassifiedRequest,
    /// Immediate answer (first layer)
    pub immediate_answer: String,
    /// Supporting evidence (second layer)
    pub supporting_evidence: Vec<Evidence>,
    /// Implementation details (third layer)
    pub implementation_details: Vec<Detail>,
    /// Research metadata
    pub metadata: ResearchMetadata,
}

impl ResearchResult {
    /// Create a new research result
    pub fn new(
        request: ClassifiedRequest,
        immediate_answer: String,
        supporting_evidence: Vec<Evidence>,
        implementation_details: Vec<Detail>,
        metadata: ResearchMetadata,
    ) -> Self {
        Self {
            request,
            immediate_answer,
            supporting_evidence,
            implementation_details,
            metadata,
        }
    }

    /// Get the research type from the request
    pub fn research_type(&self) -> &ResearchType {
        &self.request.research_type
    }

    /// Get the original query from the request
    pub fn original_query(&self) -> &str {
        &self.request.original_query
    }

    /// Get the cache key from metadata
    pub fn cache_key(&self) -> &str {
        &self.metadata.cache_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_research_type_display() {
        assert_eq!(ResearchType::Decision.display_name(), "Decision");
        assert_eq!(
            ResearchType::Implementation.display_name(),
            "Implementation"
        );
        assert_eq!(
            ResearchType::Troubleshooting.display_name(),
            "Troubleshooting"
        );
        assert_eq!(ResearchType::Learning.display_name(), "Learning");
        assert_eq!(ResearchType::Validation.display_name(), "Validation");
    }

    #[test]
    fn test_research_type_from_str() {
        assert_eq!(
            "decision".parse::<ResearchType>().unwrap(),
            ResearchType::Decision
        );
        assert_eq!(
            "IMPLEMENTATION".parse::<ResearchType>().unwrap(),
            ResearchType::Implementation
        );
        assert!("invalid".parse::<ResearchType>().is_err());
    }

    #[test]
    fn test_research_type_all() {
        let all_types = ResearchType::all();
        assert_eq!(all_types.len(), 5);
        assert!(all_types.contains(&ResearchType::Decision));
        assert!(all_types.contains(&ResearchType::Implementation));
        assert!(all_types.contains(&ResearchType::Troubleshooting));
        assert!(all_types.contains(&ResearchType::Learning));
        assert!(all_types.contains(&ResearchType::Validation));
    }

    #[test]
    fn test_classified_request_creation() {
        let request = ClassifiedRequest::new(
            "How to implement async in Rust?".to_string(),
            ResearchType::Implementation,
            AudienceContext::default(),
            DomainContext::default(),
            0.85,
            vec!["implement".to_string(), "async".to_string()],
        );

        assert_eq!(request.original_query, "How to implement async in Rust?");
        assert_eq!(request.research_type, ResearchType::Implementation);
        assert_eq!(request.confidence, 0.85);
        assert_eq!(request.matched_keywords.len(), 2);
    }

    #[test]
    fn test_research_result_accessors() {
        let request = ClassifiedRequest::new(
            "Test query".to_string(),
            ResearchType::Learning,
            AudienceContext::default(),
            DomainContext::default(),
            0.9,
            vec![],
        );

        let metadata = ResearchMetadata {
            completed_at: Utc::now(),
            processing_time_ms: 1000,
            sources_consulted: vec![],
            quality_score: 0.8,
            cache_key: "test-key".to_string(),
            tags: HashMap::new(),
        };

        let result =
            ResearchResult::new(request, "Test answer".to_string(), vec![], vec![], metadata);

        assert_eq!(result.research_type(), &ResearchType::Learning);
        assert_eq!(result.original_query(), "Test query");
        assert_eq!(result.cache_key(), "test-key");
    }
}
