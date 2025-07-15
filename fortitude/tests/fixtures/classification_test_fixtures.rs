//! Test fixtures for multi-dimensional classification scenarios
//!
//! This module provides comprehensive test data and fixtures for testing
//! the multi-dimensional classification system across various scenarios.

use chrono::Utc;
use fortitude_types::{
    classification_result::{
        AudienceLevel, ClassificationDimension, ClassificationMetadata, DimensionConfidence,
        EnhancedClassificationResult, TechnicalDomain, UrgencyLevel,
    },
    AudienceContext, ClassificationCandidate, ClassificationResult, ClassifiedRequest,
    DomainContext, ResearchMetadata, ResearchResult, ResearchType,
};
use std::collections::HashMap;

/// Comprehensive test scenario for multi-dimensional classification
#[derive(Debug, Clone)]
pub struct ClassificationTestScenario {
    pub name: String,
    pub query: String,
    pub research_type: ResearchType,
    pub expected_audience: AudienceLevel,
    pub expected_domain: TechnicalDomain,
    pub expected_urgency: UrgencyLevel,
    pub expected_confidence_range: (f64, f64),
    pub expected_keywords: Vec<String>,
    pub description: String,
    pub tags: Vec<String>,
}

impl ClassificationTestScenario {
    pub fn new(
        name: String,
        query: String,
        research_type: ResearchType,
        expected_audience: AudienceLevel,
        expected_domain: TechnicalDomain,
        expected_urgency: UrgencyLevel,
        expected_confidence_range: (f64, f64),
        expected_keywords: Vec<String>,
        description: String,
        tags: Vec<String>,
    ) -> Self {
        Self {
            name,
            query,
            research_type,
            expected_audience,
            expected_domain,
            expected_urgency,
            expected_confidence_range,
            expected_keywords,
            description,
            tags,
        }
    }
}

/// Test fixture for audience context scenarios
#[derive(Debug, Clone)]
pub struct AudienceTestFixture {
    pub query: String,
    pub expected_level: AudienceLevel,
    pub confidence_range: (f64, f64),
    pub keywords: Vec<String>,
    pub indicators: Vec<String>,
}

/// Test fixture for domain context scenarios
#[derive(Debug, Clone)]
pub struct DomainTestFixture {
    pub query: String,
    pub expected_domain: TechnicalDomain,
    pub confidence_range: (f64, f64),
    pub keywords: Vec<String>,
    pub technologies: Vec<String>,
}

/// Test fixture for urgency context scenarios
#[derive(Debug, Clone)]
pub struct UrgencyTestFixture {
    pub query: String,
    pub expected_urgency: UrgencyLevel,
    pub confidence_range: (f64, f64),
    pub keywords: Vec<String>,
    pub indicators: Vec<String>,
}

/// Get comprehensive test scenarios for multi-dimensional classification
pub fn get_comprehensive_test_scenarios() -> Vec<ClassificationTestScenario> {
    vec![
        // Beginner Learning Scenarios
        ClassificationTestScenario::new(
            "Beginner Rust Learning".to_string(),
            "I'm new to Rust and want to learn the basics of ownership and borrowing".to_string(),
            ResearchType::Learning,
            AudienceLevel::Beginner,
            TechnicalDomain::Rust,
            UrgencyLevel::Exploratory,
            (0.7, 0.9),
            vec!["new".to_string(), "learn".to_string(), "basics".to_string(), "rust".to_string(), "ownership".to_string()],
            "Classic beginner learning scenario with clear indicators".to_string(),
            vec!["beginner".to_string(), "learning".to_string(), "rust".to_string()],
        ),
        
        ClassificationTestScenario::new(
            "Beginner Web Development".to_string(),
            "I'm just starting web development and need to understand HTML, CSS, and JavaScript basics".to_string(),
            ResearchType::Learning,
            AudienceLevel::Beginner,
            TechnicalDomain::Web,
            UrgencyLevel::Exploratory,
            (0.8, 0.9),
            vec!["starting".to_string(), "web".to_string(), "html".to_string(), "css".to_string(), "javascript".to_string()],
            "Beginner web development with multiple technologies".to_string(),
            vec!["beginner".to_string(), "web".to_string(), "frontend".to_string()],
        ),
        
        // Intermediate Implementation Scenarios
        ClassificationTestScenario::new(
            "Intermediate Rust Implementation".to_string(),
            "I have some Rust experience and need to implement async functions for my web server".to_string(),
            ResearchType::Implementation,
            AudienceLevel::Intermediate,
            TechnicalDomain::Rust,
            UrgencyLevel::Planned,
            (0.7, 0.9),
            vec!["experience".to_string(), "implement".to_string(), "async".to_string(), "rust".to_string(), "server".to_string()],
            "Intermediate implementation with specific technical requirements".to_string(),
            vec!["intermediate".to_string(), "implementation".to_string(), "async".to_string()],
        ),
        
        ClassificationTestScenario::new(
            "Intermediate Python API".to_string(),
            "I know Python basics and need to build a REST API with FastAPI and database integration".to_string(),
            ResearchType::Implementation,
            AudienceLevel::Intermediate,
            TechnicalDomain::Python,
            UrgencyLevel::Planned,
            (0.7, 0.9),
            vec!["python".to_string(), "build".to_string(), "api".to_string(), "fastapi".to_string(), "database".to_string()],
            "Python API implementation with framework specifics".to_string(),
            vec!["intermediate".to_string(), "python".to_string(), "api".to_string()],
        ),
        
        // Advanced Decision Scenarios
        ClassificationTestScenario::new(
            "Advanced Architecture Decision".to_string(),
            "I'm architecting an enterprise microservices system and need to choose between event sourcing and CQRS patterns".to_string(),
            ResearchType::Decision,
            AudienceLevel::Advanced,
            TechnicalDomain::Architecture,
            UrgencyLevel::Planned,
            (0.8, 0.9),
            vec!["architecting".to_string(), "enterprise".to_string(), "microservices".to_string(), "event".to_string(), "cqrs".to_string()],
            "Advanced architectural pattern decision".to_string(),
            vec!["advanced".to_string(), "architecture".to_string(), "patterns".to_string()],
        ),
        
        ClassificationTestScenario::new(
            "Advanced Database Decision".to_string(),
            "For our high-scale distributed system, I need to decide between Cassandra, MongoDB, and PostgreSQL".to_string(),
            ResearchType::Decision,
            AudienceLevel::Advanced,
            TechnicalDomain::Database,
            UrgencyLevel::Planned,
            (0.7, 0.9),
            vec!["high-scale".to_string(), "distributed".to_string(), "cassandra".to_string(), "mongodb".to_string(), "postgresql".to_string()],
            "Database technology decision for distributed systems".to_string(),
            vec!["advanced".to_string(), "database".to_string(), "distributed".to_string()],
        ),
        
        // Urgent Troubleshooting Scenarios
        ClassificationTestScenario::new(
            "Urgent Production Issue".to_string(),
            "URGENT: Our production React application is crashing and users can't access critical features".to_string(),
            ResearchType::Troubleshooting,
            AudienceLevel::Intermediate,
            TechnicalDomain::Web,
            UrgencyLevel::Immediate,
            (0.8, 0.9),
            vec!["urgent".to_string(), "production".to_string(), "react".to_string(), "crashing".to_string(), "critical".to_string()],
            "Critical production issue requiring immediate attention".to_string(),
            vec!["urgent".to_string(), "production".to_string(), "react".to_string()],
        ),
        
        ClassificationTestScenario::new(
            "Critical System Failure".to_string(),
            "CRITICAL: Our Kubernetes cluster is down and all services are unavailable - need immediate help".to_string(),
            ResearchType::Troubleshooting,
            AudienceLevel::Advanced,
            TechnicalDomain::DevOps,
            UrgencyLevel::Immediate,
            (0.9, 1.0),
            vec!["critical".to_string(), "kubernetes".to_string(), "cluster".to_string(), "down".to_string(), "services".to_string()],
            "Critical infrastructure failure".to_string(),
            vec!["critical".to_string(), "kubernetes".to_string(), "infrastructure".to_string()],
        ),
        
        // Validation Scenarios
        ClassificationTestScenario::new(
            "Advanced Security Validation".to_string(),
            "I need expert review of my OAuth2 implementation and security architecture for compliance".to_string(),
            ResearchType::Validation,
            AudienceLevel::Advanced,
            TechnicalDomain::Security,
            UrgencyLevel::Planned,
            (0.7, 0.9),
            vec!["expert".to_string(), "review".to_string(), "oauth2".to_string(), "security".to_string(), "compliance".to_string()],
            "Security architecture validation for compliance".to_string(),
            vec!["advanced".to_string(), "security".to_string(), "validation".to_string()],
        ),
        
        ClassificationTestScenario::new(
            "Code Quality Validation".to_string(),
            "I want to validate my Python machine learning pipeline follows best practices and is production-ready".to_string(),
            ResearchType::Validation,
            AudienceLevel::Intermediate,
            TechnicalDomain::AI,
            UrgencyLevel::Planned,
            (0.7, 0.9),
            vec!["validate".to_string(), "python".to_string(), "machine".to_string(), "learning".to_string(), "production".to_string()],
            "ML pipeline validation for production readiness".to_string(),
            vec!["validation".to_string(), "ml".to_string(), "production".to_string()],
        ),
        
        // Complex Multi-Domain Scenarios
        ClassificationTestScenario::new(
            "Full-Stack Implementation".to_string(),
            "I'm building a full-stack application with Rust backend, React frontend, and PostgreSQL database".to_string(),
            ResearchType::Implementation,
            AudienceLevel::Intermediate,
            TechnicalDomain::Web, // Primary domain
            UrgencyLevel::Planned,
            (0.6, 0.8),
            vec!["full-stack".to_string(), "rust".to_string(), "react".to_string(), "postgresql".to_string(), "backend".to_string()],
            "Multi-technology full-stack implementation".to_string(),
            vec!["full-stack".to_string(), "multi-tech".to_string(), "implementation".to_string()],
        ),
        
        ClassificationTestScenario::new(
            "DevOps AI Integration".to_string(),
            "I need to implement ML model deployment pipeline with Docker, Kubernetes, and monitoring".to_string(),
            ResearchType::Implementation,
            AudienceLevel::Advanced,
            TechnicalDomain::DevOps, // Primary domain
            UrgencyLevel::Planned,
            (0.7, 0.9),
            vec!["ml".to_string(), "deployment".to_string(), "docker".to_string(), "kubernetes".to_string(), "monitoring".to_string()],
            "ML deployment with DevOps practices".to_string(),
            vec!["devops".to_string(), "ml".to_string(), "deployment".to_string()],
        ),
        
        // Edge Cases and Ambiguous Scenarios
        ClassificationTestScenario::new(
            "Ambiguous Request".to_string(),
            "I need help with something related to programming".to_string(),
            ResearchType::Learning,
            AudienceLevel::Intermediate, // Fallback
            TechnicalDomain::General,
            UrgencyLevel::Exploratory,
            (0.1, 0.3),
            vec!["help".to_string(), "programming".to_string()],
            "Ambiguous request that should trigger fallback".to_string(),
            vec!["ambiguous".to_string(), "fallback".to_string()],
        ),
        
        ClassificationTestScenario::new(
            "Mixed Signals".to_string(),
            "I'm an expert beginner who needs urgent help learning advanced basics of simple complexity".to_string(),
            ResearchType::Learning,
            AudienceLevel::Beginner, // Should pick strongest signal
            TechnicalDomain::General,
            UrgencyLevel::Immediate, // Urgent signal
            (0.3, 0.6),
            vec!["expert".to_string(), "beginner".to_string(), "urgent".to_string(), "learning".to_string(), "advanced".to_string()],
            "Conflicting signals that test disambiguation".to_string(),
            vec!["conflicting".to_string(), "disambiguation".to_string()],
        ),
    ]
}

/// Get test fixtures for audience level detection
pub fn get_audience_test_fixtures() -> Vec<AudienceTestFixture> {
    vec![
        // Beginner fixtures
        AudienceTestFixture {
            query: "I'm completely new to programming and don't know where to start".to_string(),
            expected_level: AudienceLevel::Beginner,
            confidence_range: (0.8, 0.9),
            keywords: vec![
                "new".to_string(),
                "programming".to_string(),
                "start".to_string(),
            ],
            indicators: vec!["completely new".to_string(), "don't know".to_string()],
        },
        AudienceTestFixture {
            query: "I'm a beginner learning Python and need help with basic syntax".to_string(),
            expected_level: AudienceLevel::Beginner,
            confidence_range: (0.9, 1.0),
            keywords: vec![
                "beginner".to_string(),
                "learning".to_string(),
                "basic".to_string(),
            ],
            indicators: vec!["beginner".to_string(), "basic syntax".to_string()],
        },
        AudienceTestFixture {
            query: "Just started coding last week and everything is confusing".to_string(),
            expected_level: AudienceLevel::Beginner,
            confidence_range: (0.8, 0.9),
            keywords: vec![
                "started".to_string(),
                "coding".to_string(),
                "confusing".to_string(),
            ],
            indicators: vec!["just started".to_string(), "last week".to_string()],
        },
        // Intermediate fixtures
        AudienceTestFixture {
            query: "I have some programming experience but I'm stuck on this problem".to_string(),
            expected_level: AudienceLevel::Intermediate,
            confidence_range: (0.7, 0.8),
            keywords: vec![
                "experience".to_string(),
                "programming".to_string(),
                "stuck".to_string(),
            ],
            indicators: vec!["some experience".to_string(), "stuck on".to_string()],
        },
        AudienceTestFixture {
            query: "I know the basics of Java but I'm new to Spring framework".to_string(),
            expected_level: AudienceLevel::Intermediate,
            confidence_range: (0.8, 0.9),
            keywords: vec![
                "know".to_string(),
                "basics".to_string(),
                "java".to_string(),
                "spring".to_string(),
            ],
            indicators: vec!["know the basics".to_string(), "new to".to_string()],
        },
        AudienceTestFixture {
            query: "I've been coding for 2 years but this concept is new to me".to_string(),
            expected_level: AudienceLevel::Intermediate,
            confidence_range: (0.8, 0.9),
            keywords: vec![
                "coding".to_string(),
                "years".to_string(),
                "concept".to_string(),
            ],
            indicators: vec!["2 years".to_string(), "new to me".to_string()],
        },
        // Advanced fixtures
        AudienceTestFixture {
            query: "I'm an experienced architect looking for advanced optimization techniques"
                .to_string(),
            expected_level: AudienceLevel::Advanced,
            confidence_range: (0.9, 1.0),
            keywords: vec![
                "experienced".to_string(),
                "architect".to_string(),
                "advanced".to_string(),
                "optimization".to_string(),
            ],
            indicators: vec![
                "experienced architect".to_string(),
                "advanced optimization".to_string(),
            ],
        },
        AudienceTestFixture {
            query: "I need expert-level guidance on distributed systems design patterns"
                .to_string(),
            expected_level: AudienceLevel::Advanced,
            confidence_range: (0.8, 0.9),
            keywords: vec![
                "expert".to_string(),
                "guidance".to_string(),
                "distributed".to_string(),
                "patterns".to_string(),
            ],
            indicators: vec![
                "expert-level".to_string(),
                "distributed systems".to_string(),
            ],
        },
        AudienceTestFixture {
            query: "I'm a senior developer with 10+ years experience in enterprise systems"
                .to_string(),
            expected_level: AudienceLevel::Advanced,
            confidence_range: (0.9, 1.0),
            keywords: vec![
                "senior".to_string(),
                "developer".to_string(),
                "years".to_string(),
                "enterprise".to_string(),
            ],
            indicators: vec!["senior developer".to_string(), "10+ years".to_string()],
        },
    ]
}

/// Get test fixtures for technical domain detection
pub fn get_domain_test_fixtures() -> Vec<DomainTestFixture> {
    vec![
        // Rust domain
        DomainTestFixture {
            query: "How to implement async/await in Rust with tokio runtime?".to_string(),
            expected_domain: TechnicalDomain::Rust,
            confidence_range: (0.9, 1.0),
            keywords: vec!["rust".to_string(), "async".to_string(), "tokio".to_string()],
            technologies: vec!["rust".to_string(), "tokio".to_string(), "async".to_string()],
        },
        DomainTestFixture {
            query: "Rust ownership, borrowing, and lifetimes explained".to_string(),
            expected_domain: TechnicalDomain::Rust,
            confidence_range: (0.9, 1.0),
            keywords: vec![
                "rust".to_string(),
                "ownership".to_string(),
                "borrowing".to_string(),
                "lifetimes".to_string(),
            ],
            technologies: vec!["rust".to_string()],
        },
        // Web domain
        DomainTestFixture {
            query: "React hooks and state management with Redux toolkit".to_string(),
            expected_domain: TechnicalDomain::Web,
            confidence_range: (0.8, 0.9),
            keywords: vec![
                "react".to_string(),
                "hooks".to_string(),
                "redux".to_string(),
            ],
            technologies: vec!["react".to_string(), "redux".to_string()],
        },
        DomainTestFixture {
            query: "Building responsive web design with HTML5, CSS3, and JavaScript".to_string(),
            expected_domain: TechnicalDomain::Web,
            confidence_range: (0.9, 1.0),
            keywords: vec![
                "responsive".to_string(),
                "html5".to_string(),
                "css3".to_string(),
                "javascript".to_string(),
            ],
            technologies: vec![
                "html5".to_string(),
                "css3".to_string(),
                "javascript".to_string(),
            ],
        },
        // Python domain
        DomainTestFixture {
            query: "Python data science with pandas, numpy, and scikit-learn".to_string(),
            expected_domain: TechnicalDomain::Python,
            confidence_range: (0.8, 0.9),
            keywords: vec![
                "python".to_string(),
                "pandas".to_string(),
                "numpy".to_string(),
                "scikit-learn".to_string(),
            ],
            technologies: vec![
                "python".to_string(),
                "pandas".to_string(),
                "numpy".to_string(),
                "scikit-learn".to_string(),
            ],
        },
        DomainTestFixture {
            query: "FastAPI development with Pydantic models and SQLAlchemy ORM".to_string(),
            expected_domain: TechnicalDomain::Python,
            confidence_range: (0.8, 0.9),
            keywords: vec![
                "fastapi".to_string(),
                "pydantic".to_string(),
                "sqlalchemy".to_string(),
            ],
            technologies: vec![
                "fastapi".to_string(),
                "pydantic".to_string(),
                "sqlalchemy".to_string(),
            ],
        },
        // DevOps domain
        DomainTestFixture {
            query: "Kubernetes deployment with Helm charts and monitoring".to_string(),
            expected_domain: TechnicalDomain::DevOps,
            confidence_range: (0.9, 1.0),
            keywords: vec![
                "kubernetes".to_string(),
                "deployment".to_string(),
                "helm".to_string(),
                "monitoring".to_string(),
            ],
            technologies: vec!["kubernetes".to_string(), "helm".to_string()],
        },
        DomainTestFixture {
            query: "Docker containerization with CI/CD pipeline automation".to_string(),
            expected_domain: TechnicalDomain::DevOps,
            confidence_range: (0.8, 0.9),
            keywords: vec![
                "docker".to_string(),
                "containerization".to_string(),
                "ci/cd".to_string(),
                "pipeline".to_string(),
            ],
            technologies: vec!["docker".to_string(), "ci/cd".to_string()],
        },
        // AI/ML domain
        DomainTestFixture {
            query: "Machine learning model training with TensorFlow and PyTorch".to_string(),
            expected_domain: TechnicalDomain::AI,
            confidence_range: (0.9, 1.0),
            keywords: vec![
                "machine".to_string(),
                "learning".to_string(),
                "tensorflow".to_string(),
                "pytorch".to_string(),
            ],
            technologies: vec!["tensorflow".to_string(), "pytorch".to_string()],
        },
        DomainTestFixture {
            query: "Deep learning neural networks for computer vision".to_string(),
            expected_domain: TechnicalDomain::AI,
            confidence_range: (0.8, 0.9),
            keywords: vec![
                "deep".to_string(),
                "learning".to_string(),
                "neural".to_string(),
                "vision".to_string(),
            ],
            technologies: vec!["deep learning".to_string(), "neural networks".to_string()],
        },
        // Database domain
        DomainTestFixture {
            query: "PostgreSQL query optimization and indexing strategies".to_string(),
            expected_domain: TechnicalDomain::Database,
            confidence_range: (0.9, 1.0),
            keywords: vec![
                "postgresql".to_string(),
                "query".to_string(),
                "optimization".to_string(),
                "indexing".to_string(),
            ],
            technologies: vec!["postgresql".to_string()],
        },
        DomainTestFixture {
            query: "MongoDB aggregation pipeline and document modeling".to_string(),
            expected_domain: TechnicalDomain::Database,
            confidence_range: (0.8, 0.9),
            keywords: vec![
                "mongodb".to_string(),
                "aggregation".to_string(),
                "pipeline".to_string(),
                "document".to_string(),
            ],
            technologies: vec!["mongodb".to_string()],
        },
    ]
}

/// Get test fixtures for urgency level detection
pub fn get_urgency_test_fixtures() -> Vec<UrgencyTestFixture> {
    vec![
        // Immediate urgency
        UrgencyTestFixture {
            query: "URGENT: Production system is down and users can't access the service"
                .to_string(),
            expected_urgency: UrgencyLevel::Immediate,
            confidence_range: (0.9, 1.0),
            keywords: vec![
                "urgent".to_string(),
                "production".to_string(),
                "down".to_string(),
            ],
            indicators: vec![
                "URGENT".to_string(),
                "production system".to_string(),
                "down".to_string(),
            ],
        },
        UrgencyTestFixture {
            query: "CRITICAL: Security breach detected, need immediate response".to_string(),
            expected_urgency: UrgencyLevel::Immediate,
            confidence_range: (0.9, 1.0),
            keywords: vec![
                "critical".to_string(),
                "security".to_string(),
                "breach".to_string(),
                "immediate".to_string(),
            ],
            indicators: vec![
                "CRITICAL".to_string(),
                "security breach".to_string(),
                "immediate".to_string(),
            ],
        },
        UrgencyTestFixture {
            query: "Emergency: Database corruption causing data loss".to_string(),
            expected_urgency: UrgencyLevel::Immediate,
            confidence_range: (0.8, 0.9),
            keywords: vec![
                "emergency".to_string(),
                "database".to_string(),
                "corruption".to_string(),
                "data".to_string(),
            ],
            indicators: vec!["Emergency".to_string(), "data loss".to_string()],
        },
        // Planned urgency
        UrgencyTestFixture {
            query: "I need to implement this feature for next week's release".to_string(),
            expected_urgency: UrgencyLevel::Planned,
            confidence_range: (0.7, 0.8),
            keywords: vec![
                "implement".to_string(),
                "feature".to_string(),
                "next".to_string(),
                "week".to_string(),
                "release".to_string(),
            ],
            indicators: vec!["next week".to_string(), "release".to_string()],
        },
        UrgencyTestFixture {
            query: "Working on a project with a deadline in two months".to_string(),
            expected_urgency: UrgencyLevel::Planned,
            confidence_range: (0.7, 0.8),
            keywords: vec![
                "working".to_string(),
                "project".to_string(),
                "deadline".to_string(),
                "months".to_string(),
            ],
            indicators: vec!["deadline".to_string(), "two months".to_string()],
        },
        UrgencyTestFixture {
            query: "I need to complete this task by the end of the quarter".to_string(),
            expected_urgency: UrgencyLevel::Planned,
            confidence_range: (0.7, 0.8),
            keywords: vec![
                "complete".to_string(),
                "task".to_string(),
                "end".to_string(),
                "quarter".to_string(),
            ],
            indicators: vec!["end of quarter".to_string(), "complete".to_string()],
        },
        // Exploratory urgency
        UrgencyTestFixture {
            query: "I'm curious about this new technology and want to learn more".to_string(),
            expected_urgency: UrgencyLevel::Exploratory,
            confidence_range: (0.8, 0.9),
            keywords: vec![
                "curious".to_string(),
                "technology".to_string(),
                "learn".to_string(),
            ],
            indicators: vec!["curious".to_string(), "want to learn".to_string()],
        },
        UrgencyTestFixture {
            query: "Just exploring different database options for future projects".to_string(),
            expected_urgency: UrgencyLevel::Exploratory,
            confidence_range: (0.7, 0.8),
            keywords: vec![
                "exploring".to_string(),
                "database".to_string(),
                "options".to_string(),
                "future".to_string(),
            ],
            indicators: vec!["exploring".to_string(), "future projects".to_string()],
        },
        UrgencyTestFixture {
            query: "Interested in understanding how machine learning works".to_string(),
            expected_urgency: UrgencyLevel::Exploratory,
            confidence_range: (0.8, 0.9),
            keywords: vec![
                "interested".to_string(),
                "understanding".to_string(),
                "machine".to_string(),
                "learning".to_string(),
            ],
            indicators: vec!["interested".to_string(), "understanding".to_string()],
        },
    ]
}

/// Create a test classified request with realistic data
pub fn create_test_classified_request(scenario: &ClassificationTestScenario) -> ClassifiedRequest {
    let audience_context = AudienceContext {
        level: scenario.expected_audience.display_name().to_lowercase(),
        domain: scenario.expected_domain.display_name().to_lowercase(),
        format: "detailed".to_string(),
    };

    let domain_context = DomainContext {
        technology: scenario.expected_domain.display_name().to_lowercase(),
        project_type: "general".to_string(),
        frameworks: vec![],
        tags: scenario.tags.clone(),
    };

    ClassifiedRequest::new(
        scenario.query.clone(),
        scenario.research_type.clone(),
        audience_context,
        domain_context,
        scenario.expected_confidence_range.1, // Use upper bound
        scenario.expected_keywords.clone(),
    )
}

/// Create a test enhanced classification result
pub fn create_test_enhanced_result(
    scenario: &ClassificationTestScenario,
) -> EnhancedClassificationResult {
    let dimension_confidences = vec![
        DimensionConfidence::new(
            ClassificationDimension::ResearchType,
            scenario.expected_confidence_range.1,
            scenario.expected_keywords.clone(),
            format!("Detected {} research type", scenario.research_type),
        ),
        DimensionConfidence::new(
            ClassificationDimension::AudienceLevel,
            scenario.expected_confidence_range.1 * 0.9,
            vec![scenario.expected_audience.display_name().to_lowercase()],
            format!(
                "Detected {} audience level",
                scenario.expected_audience.display_name()
            ),
        ),
        DimensionConfidence::new(
            ClassificationDimension::TechnicalDomain,
            scenario.expected_confidence_range.1 * 0.8,
            vec![scenario.expected_domain.display_name().to_lowercase()],
            format!(
                "Detected {} technical domain",
                scenario.expected_domain.display_name()
            ),
        ),
        DimensionConfidence::new(
            ClassificationDimension::Urgency,
            scenario.expected_confidence_range.1 * 0.7,
            vec![scenario.expected_urgency.display_name().to_lowercase()],
            format!(
                "Detected {} urgency level",
                scenario.expected_urgency.display_name()
            ),
        ),
    ];

    let mut tags = HashMap::new();
    tags.insert("test_scenario".to_string(), scenario.name.clone());
    tags.insert("test_description".to_string(), scenario.description.clone());

    let metadata = ClassificationMetadata {
        processing_time_ms: 50,
        algorithm: "test_fixture".to_string(),
        classifier_version: "1.0.0".to_string(),
        fallback_used: false,
        tags,
    };

    EnhancedClassificationResult::new(
        scenario.research_type.clone(),
        scenario.expected_confidence_range.1,
        scenario.expected_audience.clone(),
        scenario.expected_domain.clone(),
        scenario.expected_urgency.clone(),
        dimension_confidences,
        scenario.expected_keywords.clone(),
        1,
        vec![],
        metadata,
    )
}

/// Create a test research result from a scenario
pub fn create_test_research_result(scenario: &ClassificationTestScenario) -> ResearchResult {
    let request = create_test_classified_request(scenario);

    let immediate_answer = match scenario.research_type {
        ResearchType::Learning => format!("Learning material for: {}", scenario.query),
        ResearchType::Implementation => format!("Implementation guide for: {}", scenario.query),
        ResearchType::Troubleshooting => format!("Troubleshooting steps for: {}", scenario.query),
        ResearchType::Decision => format!("Decision guidance for: {}", scenario.query),
        ResearchType::Validation => format!("Validation approach for: {}", scenario.query),
    };

    let mut tags = HashMap::new();
    tags.insert("test_scenario".to_string(), scenario.name.clone());
    tags.insert(
        "audience".to_string(),
        scenario.expected_audience.display_name().to_string(),
    );
    tags.insert(
        "domain".to_string(),
        scenario.expected_domain.display_name().to_string(),
    );
    tags.insert(
        "urgency".to_string(),
        scenario.expected_urgency.display_name().to_string(),
    );

    let metadata = ResearchMetadata {
        completed_at: Utc::now(),
        processing_time_ms: 150,
        sources_consulted: vec!["test_fixture".to_string()],
        quality_score: scenario.expected_confidence_range.1,
        cache_key: format!("test_cache_{}", scenario.name),
        tags,
    };

    ResearchResult::new(
        request,
        immediate_answer,
        vec![], // No supporting evidence in fixtures
        vec![], // No implementation details in fixtures
        metadata,
    )
}

/// Get test scenarios filtered by specific criteria
pub fn get_scenarios_by_research_type(
    research_type: ResearchType,
) -> Vec<ClassificationTestScenario> {
    get_comprehensive_test_scenarios()
        .into_iter()
        .filter(|scenario| scenario.research_type == research_type)
        .collect()
}

/// Get test scenarios filtered by audience level
pub fn get_scenarios_by_audience(audience: AudienceLevel) -> Vec<ClassificationTestScenario> {
    get_comprehensive_test_scenarios()
        .into_iter()
        .filter(|scenario| scenario.expected_audience == audience)
        .collect()
}

/// Get test scenarios filtered by domain
pub fn get_scenarios_by_domain(domain: TechnicalDomain) -> Vec<ClassificationTestScenario> {
    get_comprehensive_test_scenarios()
        .into_iter()
        .filter(|scenario| scenario.expected_domain == domain)
        .collect()
}

/// Get test scenarios filtered by urgency
pub fn get_scenarios_by_urgency(urgency: UrgencyLevel) -> Vec<ClassificationTestScenario> {
    get_comprehensive_test_scenarios()
        .into_iter()
        .filter(|scenario| scenario.expected_urgency == urgency)
        .collect()
}

/// Get test scenarios filtered by tags
pub fn get_scenarios_by_tag(tag: &str) -> Vec<ClassificationTestScenario> {
    get_comprehensive_test_scenarios()
        .into_iter()
        .filter(|scenario| scenario.tags.contains(&tag.to_string()))
        .collect()
}

/// Get edge case scenarios for testing robustness
pub fn get_edge_case_scenarios() -> Vec<ClassificationTestScenario> {
    get_scenarios_by_tag("ambiguous")
        .into_iter()
        .chain(get_scenarios_by_tag("conflicting"))
        .chain(get_scenarios_by_tag("fallback"))
        .collect()
}

/// Get high-confidence scenarios for accuracy testing
pub fn get_high_confidence_scenarios() -> Vec<ClassificationTestScenario> {
    get_comprehensive_test_scenarios()
        .into_iter()
        .filter(|scenario| scenario.expected_confidence_range.0 > 0.7)
        .collect()
}

/// Get scenarios for specific performance testing
pub fn get_performance_test_scenarios() -> Vec<ClassificationTestScenario> {
    // Include a variety of scenarios for performance testing
    let mut scenarios = Vec::new();

    // Add some from each category
    scenarios.extend(
        get_scenarios_by_research_type(ResearchType::Learning)
            .into_iter()
            .take(2),
    );
    scenarios.extend(
        get_scenarios_by_research_type(ResearchType::Implementation)
            .into_iter()
            .take(2),
    );
    scenarios.extend(
        get_scenarios_by_research_type(ResearchType::Troubleshooting)
            .into_iter()
            .take(2),
    );
    scenarios.extend(
        get_scenarios_by_research_type(ResearchType::Decision)
            .into_iter()
            .take(2),
    );
    scenarios.extend(
        get_scenarios_by_research_type(ResearchType::Validation)
            .into_iter()
            .take(2),
    );

    scenarios
}
