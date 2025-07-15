//! Smoke tests to verify basic functionality
//!
//! Simple tests to ensure the test infrastructure is working correctly.

use fortitude_types::{
    AudienceContext, AudienceLevel, ClassificationError, ClassificationResult, ClassifiedRequest,
    DomainContext, ResearchType, TechnicalDomain, UrgencyLevel,
};

#[test]
fn test_research_type_display() {
    assert_eq!(ResearchType::Learning.to_string(), "Learning");
    assert_eq!(ResearchType::Implementation.to_string(), "Implementation");
    assert_eq!(ResearchType::Troubleshooting.to_string(), "Troubleshooting");
    assert_eq!(ResearchType::Decision.to_string(), "Decision");
    assert_eq!(ResearchType::Validation.to_string(), "Validation");
}

#[test]
fn test_audience_level_display() {
    assert_eq!(AudienceLevel::Beginner.display_name(), "Beginner");
    assert_eq!(AudienceLevel::Intermediate.display_name(), "Intermediate");
    assert_eq!(AudienceLevel::Advanced.display_name(), "Advanced");
}

#[test]
fn test_technical_domain_display() {
    assert_eq!(TechnicalDomain::Rust.display_name(), "Rust");
    assert_eq!(TechnicalDomain::Web.display_name(), "Web");
    assert_eq!(TechnicalDomain::Python.display_name(), "Python");
    assert_eq!(TechnicalDomain::DevOps.display_name(), "DevOps");
    assert_eq!(TechnicalDomain::AI.display_name(), "AI");
    assert_eq!(TechnicalDomain::Database.display_name(), "Database");
    assert_eq!(TechnicalDomain::Architecture.display_name(), "Architecture");
    assert_eq!(TechnicalDomain::General.display_name(), "General");
}

#[test]
fn test_urgency_level_display() {
    assert_eq!(UrgencyLevel::Immediate.display_name(), "Immediate");
    assert_eq!(UrgencyLevel::Planned.display_name(), "Planned");
    assert_eq!(UrgencyLevel::Exploratory.display_name(), "Exploratory");
}

#[test]
fn test_classification_result_creation() {
    let result = ClassificationResult::new(
        ResearchType::Learning,
        0.85,
        vec!["learn".to_string(), "rust".to_string()],
        1,
        vec![],
    );

    assert_eq!(result.research_type, ResearchType::Learning);
    assert_eq!(result.confidence, 0.85);
    assert_eq!(result.matched_keywords, vec!["learn", "rust"]);
    assert_eq!(result.rule_priority, 1);
}

#[test]
fn test_classified_request_creation() {
    let audience = AudienceContext {
        level: "intermediate".to_string(),
        domain: "web".to_string(),
        format: "detailed".to_string(),
    };

    let domain = DomainContext {
        technology: "rust".to_string(),
        project_type: "web".to_string(),
        frameworks: vec!["axum".to_string()],
        tags: vec!["async".to_string()],
    };

    let request = ClassifiedRequest::new(
        "How to build a web server in Rust?".to_string(),
        ResearchType::Implementation,
        audience.clone(),
        domain.clone(),
        0.9,
        vec!["rust".to_string(), "web".to_string()],
    );

    assert_eq!(request.original_query, "How to build a web server in Rust?");
    assert_eq!(request.research_type, ResearchType::Implementation);
    assert_eq!(request.audience_context, audience);
    assert_eq!(request.domain_context, domain);
    assert_eq!(request.confidence, 0.9);
    assert_eq!(request.matched_keywords, vec!["rust", "web"]);
}

#[test]
fn test_defaults() {
    let audience = AudienceContext::default();
    assert_eq!(audience.level, "intermediate");
    assert_eq!(audience.domain, "general");
    assert_eq!(audience.format, "detailed");

    let domain = DomainContext::default();
    assert_eq!(domain.technology, "general");
    assert_eq!(domain.project_type, "general");
    assert!(domain.frameworks.is_empty());
    assert!(domain.tags.is_empty());

    let beginner_default = AudienceLevel::default();
    assert_eq!(beginner_default, AudienceLevel::Intermediate);

    let domain_default = TechnicalDomain::default();
    assert_eq!(domain_default, TechnicalDomain::General);

    let urgency_default = UrgencyLevel::default();
    assert_eq!(urgency_default, UrgencyLevel::Planned);
}

#[test]
fn test_error_handling() {
    let error = ClassificationError::InvalidInput("Test error".to_string());
    assert!(error.to_string().contains("Test error"));

    let error = ClassificationError::ProcessingTimeout;
    assert!(error.to_string().contains("timeout"));
}

#[test]
fn test_enum_serialization() {
    // Test that enums can be serialized/deserialized
    let research_type = ResearchType::Implementation;
    let serialized = serde_json::to_string(&research_type).unwrap();
    let deserialized: ResearchType = serde_json::from_str(&serialized).unwrap();
    assert_eq!(research_type, deserialized);

    let audience = AudienceLevel::Advanced;
    let serialized = serde_json::to_string(&audience).unwrap();
    let deserialized: AudienceLevel = serde_json::from_str(&serialized).unwrap();
    assert_eq!(audience, deserialized);

    let domain = TechnicalDomain::Rust;
    let serialized = serde_json::to_string(&domain).unwrap();
    let deserialized: TechnicalDomain = serde_json::from_str(&serialized).unwrap();
    assert_eq!(domain, deserialized);

    let urgency = UrgencyLevel::Immediate;
    let serialized = serde_json::to_string(&urgency).unwrap();
    let deserialized: UrgencyLevel = serde_json::from_str(&serialized).unwrap();
    assert_eq!(urgency, deserialized);
}

#[test]
fn test_classification_types_coverage() {
    // Test that all variants are covered
    let research_types = vec![
        ResearchType::Learning,
        ResearchType::Implementation,
        ResearchType::Troubleshooting,
        ResearchType::Decision,
        ResearchType::Validation,
    ];

    for rt in research_types {
        assert!(!rt.to_string().is_empty());
    }

    let audience_levels = vec![
        AudienceLevel::Beginner,
        AudienceLevel::Intermediate,
        AudienceLevel::Advanced,
    ];

    for al in audience_levels {
        assert!(!al.display_name().is_empty());
    }

    let technical_domains = vec![
        TechnicalDomain::Rust,
        TechnicalDomain::Web,
        TechnicalDomain::Python,
        TechnicalDomain::DevOps,
        TechnicalDomain::AI,
        TechnicalDomain::Database,
        TechnicalDomain::Architecture,
        TechnicalDomain::General,
    ];

    for td in technical_domains {
        assert!(!td.display_name().is_empty());
    }

    let urgency_levels = vec![
        UrgencyLevel::Immediate,
        UrgencyLevel::Planned,
        UrgencyLevel::Exploratory,
    ];

    for ul in urgency_levels {
        assert!(!ul.display_name().is_empty());
    }
}

#[test]
fn test_basic_functionality() {
    // This test verifies that basic types and structures work
    println!("Testing basic functionality...");

    let query = "How to implement async functions in Rust?";
    let research_type = ResearchType::Implementation;
    let audience = AudienceLevel::Intermediate;
    let domain = TechnicalDomain::Rust;
    let urgency = UrgencyLevel::Planned;

    println!("Query: {}", query);
    println!("Research Type: {}", research_type);
    println!("Audience: {}", audience.display_name());
    println!("Domain: {}", domain.display_name());
    println!("Urgency: {}", urgency.display_name());

    // Test that we can create contexts
    let audience_context = AudienceContext {
        level: audience.display_name().to_lowercase(),
        domain: domain.display_name().to_lowercase(),
        format: "detailed".to_string(),
    };

    let domain_context = DomainContext {
        technology: domain.display_name().to_lowercase(),
        project_type: "general".to_string(),
        frameworks: vec!["tokio".to_string()],
        tags: vec!["async".to_string()],
    };

    // Test that we can create a classified request
    let request = ClassifiedRequest::new(
        query.to_string(),
        research_type.clone(),
        audience_context,
        domain_context,
        0.8,
        vec!["rust".to_string(), "async".to_string()],
    );

    assert_eq!(request.original_query, query);
    assert_eq!(request.research_type, research_type);
    assert_eq!(request.confidence, 0.8);

    println!("✓ Basic functionality test passed");
}
