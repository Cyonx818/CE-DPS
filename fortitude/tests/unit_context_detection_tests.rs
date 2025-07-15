//! Unit tests for context detection accuracy
//!
//! These tests verify the accuracy and reliability of context detection
//! across different dimensions: audience level, technical domain, and urgency.

use fortitude_core::classification::{
    context_detector::{
        ContextDetectionConfig, ContextDetectionResult, ContextDetector, FortitudeContextDetector,
    },
    rules::{AudienceRules, DomainRules, UrgencyRules},
};
use fortitude_types::{
    classification_result::{
        AudienceLevel, ClassificationDimension, DimensionConfidence, TechnicalDomain, UrgencyLevel,
    },
    ClassificationError, ResearchType,
};
use std::collections::HashMap;

/// Helper function to create a test context detector with custom configuration
fn create_test_context_detector(config: ContextDetectionConfig) -> FortitudeContextDetector {
    FortitudeContextDetector::with_config(config)
}

/// Helper function to create a default test context detector
fn create_default_test_detector() -> FortitudeContextDetector {
    let config = ContextDetectionConfig {
        confidence_threshold: 0.5,
        enable_fallback: true,
        max_processing_time_ms: 1000,
        debug_logging: false,
    };
    create_test_context_detector(config)
}

/// Test data for audience level detection
fn get_audience_test_cases() -> Vec<(&'static str, AudienceLevel, f64)> {
    vec![
        // Beginner indicators
        (
            "I'm new to programming and need help",
            AudienceLevel::Beginner,
            0.8,
        ),
        ("I'm a beginner learning Rust", AudienceLevel::Beginner, 0.9),
        (
            "Just started coding, need basic help",
            AudienceLevel::Beginner,
            0.8,
        ),
        (
            "I don't understand the fundamentals",
            AudienceLevel::Beginner,
            0.7,
        ),
        (
            "Can you explain this in simple terms?",
            AudienceLevel::Beginner,
            0.6,
        ),
        ("What is the basics of...", AudienceLevel::Beginner, 0.7),
        (
            "I'm learning programming for the first time",
            AudienceLevel::Beginner,
            0.9,
        ),
        // Intermediate indicators
        (
            "I have some experience with programming",
            AudienceLevel::Intermediate,
            0.7,
        ),
        (
            "I know the basics but need help with...",
            AudienceLevel::Intermediate,
            0.8,
        ),
        (
            "I'm familiar with C++ but new to Rust",
            AudienceLevel::Intermediate,
            0.8,
        ),
        (
            "I understand the concept but need implementation help",
            AudienceLevel::Intermediate,
            0.7,
        ),
        (
            "I've been programming for a few years",
            AudienceLevel::Intermediate,
            0.8,
        ),
        (
            "I'm comfortable with basic concepts",
            AudienceLevel::Intermediate,
            0.7,
        ),
        // Advanced indicators
        (
            "I need help with advanced optimization techniques",
            AudienceLevel::Advanced,
            0.8,
        ),
        (
            "Looking for expert-level architecture advice",
            AudienceLevel::Advanced,
            0.9,
        ),
        (
            "I'm implementing a complex distributed system",
            AudienceLevel::Advanced,
            0.8,
        ),
        (
            "Need guidance on enterprise-level solutions",
            AudienceLevel::Advanced,
            0.8,
        ),
        (
            "I'm experienced in systems programming",
            AudienceLevel::Advanced,
            0.9,
        ),
        (
            "Looking for best practices in production environments",
            AudienceLevel::Advanced,
            0.7,
        ),
        (
            "I need advanced performance tuning advice",
            AudienceLevel::Advanced,
            0.8,
        ),
        // Ambiguous cases (should have lower confidence)
        (
            "I need help with something",
            AudienceLevel::Intermediate,
            0.3,
        ),
        ("Can you help me?", AudienceLevel::Intermediate, 0.2),
        ("I have a question", AudienceLevel::Intermediate, 0.2),
    ]
}

/// Test data for technical domain detection
fn get_domain_test_cases() -> Vec<(&'static str, TechnicalDomain, f64)> {
    vec![
        // Rust domain
        (
            "How to implement async functions in Rust?",
            TechnicalDomain::Rust,
            0.9,
        ),
        (
            "Rust ownership and borrowing concepts",
            TechnicalDomain::Rust,
            0.9,
        ),
        (
            "Cargo package management in Rust",
            TechnicalDomain::Rust,
            0.8,
        ),
        ("Rust memory safety features", TechnicalDomain::Rust, 0.8),
        ("tokio async runtime in Rust", TechnicalDomain::Rust, 0.9),
        // Web domain
        ("How to build a REST API?", TechnicalDomain::Web, 0.8),
        ("React component optimization", TechnicalDomain::Web, 0.9),
        ("HTML5 and CSS3 best practices", TechnicalDomain::Web, 0.9),
        ("Node.js server development", TechnicalDomain::Web, 0.8),
        ("Frontend JavaScript frameworks", TechnicalDomain::Web, 0.8),
        ("HTTP protocol and web standards", TechnicalDomain::Web, 0.7),
        // DevOps domain
        (
            "Docker containerization strategies",
            TechnicalDomain::DevOps,
            0.9,
        ),
        (
            "Kubernetes cluster management",
            TechnicalDomain::DevOps,
            0.9,
        ),
        ("CI/CD pipeline optimization", TechnicalDomain::DevOps, 0.8),
        ("AWS cloud infrastructure", TechnicalDomain::DevOps, 0.8),
        (
            "Terraform infrastructure as code",
            TechnicalDomain::DevOps,
            0.8,
        ),
        (
            "Monitoring and logging solutions",
            TechnicalDomain::DevOps,
            0.7,
        ),
        // Python domain
        (
            "Python data science libraries",
            TechnicalDomain::Python,
            0.8,
        ),
        ("Django web framework", TechnicalDomain::Python, 0.8),
        (
            "Python machine learning with scikit-learn",
            TechnicalDomain::Python,
            0.9,
        ),
        ("FastAPI development", TechnicalDomain::Python, 0.8),
        ("Python async programming", TechnicalDomain::Python, 0.7),
        // AI/ML domain
        ("Machine learning model training", TechnicalDomain::AI, 0.8),
        ("Neural network architectures", TechnicalDomain::AI, 0.9),
        ("Deep learning with TensorFlow", TechnicalDomain::AI, 0.9),
        ("Natural language processing", TechnicalDomain::AI, 0.8),
        ("Computer vision algorithms", TechnicalDomain::AI, 0.8),
        // Database domain
        (
            "PostgreSQL query optimization",
            TechnicalDomain::Database,
            0.9,
        ),
        ("MongoDB document database", TechnicalDomain::Database, 0.8),
        ("SQL database design", TechnicalDomain::Database, 0.8),
        ("Redis caching strategies", TechnicalDomain::Database, 0.8),
        // Architecture domain
        (
            "Microservices architecture patterns",
            TechnicalDomain::Architecture,
            0.9,
        ),
        (
            "System design principles",
            TechnicalDomain::Architecture,
            0.8,
        ),
        (
            "Distributed systems architecture",
            TechnicalDomain::Architecture,
            0.9,
        ),
        (
            "Event-driven architecture",
            TechnicalDomain::Architecture,
            0.8,
        ),
        // General cases
        ("How to debug a program?", TechnicalDomain::General, 0.6),
        (
            "Best practices for code review",
            TechnicalDomain::General,
            0.7,
        ),
        ("Version control with Git", TechnicalDomain::General, 0.6),
        // Ambiguous cases
        ("I need help with something", TechnicalDomain::General, 0.3),
        ("Can you assist me?", TechnicalDomain::General, 0.2),
    ]
}

/// Test data for urgency level detection
fn get_urgency_test_cases() -> Vec<(&'static str, UrgencyLevel, f64)> {
    vec![
        // Immediate urgency
        (
            "URGENT: Production server is down",
            UrgencyLevel::Immediate,
            0.9,
        ),
        (
            "CRITICAL: System failure in production",
            UrgencyLevel::Immediate,
            0.9,
        ),
        (
            "Emergency: Need immediate help",
            UrgencyLevel::Immediate,
            0.8,
        ),
        ("ASAP: Client demo failing", UrgencyLevel::Immediate, 0.8),
        ("Immediate assistance needed", UrgencyLevel::Immediate, 0.8),
        (
            "Production issue needs fixing now",
            UrgencyLevel::Immediate,
            0.8,
        ),
        ("Critical bug affecting users", UrgencyLevel::Immediate, 0.7),
        // Planned urgency
        (
            "I need to implement this feature next week",
            UrgencyLevel::Planned,
            0.8,
        ),
        ("Working on a project deadline", UrgencyLevel::Planned, 0.7),
        ("Need to deliver this by Friday", UrgencyLevel::Planned, 0.8),
        (
            "Planning to implement this soon",
            UrgencyLevel::Planned,
            0.7,
        ),
        ("I have a deadline coming up", UrgencyLevel::Planned, 0.7),
        ("Need to complete this task", UrgencyLevel::Planned, 0.6),
        // Exploratory urgency
        (
            "I'm curious about this technology",
            UrgencyLevel::Exploratory,
            0.8,
        ),
        (
            "Just exploring different options",
            UrgencyLevel::Exploratory,
            0.8,
        ),
        (
            "Learning for future projects",
            UrgencyLevel::Exploratory,
            0.7,
        ),
        (
            "Interested in understanding how this works",
            UrgencyLevel::Exploratory,
            0.7,
        ),
        (
            "I want to learn more about...",
            UrgencyLevel::Exploratory,
            0.8,
        ),
        (
            "Just reading up on this topic",
            UrgencyLevel::Exploratory,
            0.7,
        ),
        ("Exploring new technologies", UrgencyLevel::Exploratory, 0.8),
        // Ambiguous cases
        ("I need help with something", UrgencyLevel::Planned, 0.3),
        ("Can you help me?", UrgencyLevel::Planned, 0.2),
        ("I have a question", UrgencyLevel::Planned, 0.3),
    ]
}

#[test]
fn test_audience_level_detection_accuracy() {
    let rules = AudienceRules::new();
    let test_cases = get_audience_test_cases();
    let mut correct_predictions = 0;
    let total_predictions = test_cases.len();

    for (query, expected_level, expected_min_confidence) in test_cases {
        let result = rules
            .detect_audience_level(query)
            .expect("Audience detection should succeed");

        let (detected_level, confidence, keywords) = result;

        // Check if prediction is correct
        if detected_level == expected_level {
            correct_predictions += 1;
        }

        // Check confidence is reasonable
        assert!(
            confidence >= 0.0 && confidence <= 1.0,
            "Confidence should be between 0 and 1, got {}",
            confidence
        );

        // For clear cases, confidence should meet minimum threshold
        if expected_min_confidence > 0.5 {
            assert!(
                confidence >= expected_min_confidence * 0.8,
                "Confidence for '{}' should be >= {:.2}, got {:.2}",
                query,
                expected_min_confidence * 0.8,
                confidence
            );
        }

        // Should have matched some keywords
        assert!(
            !keywords.is_empty(),
            "Should have matched keywords for '{}'",
            query
        );

        println!(
            "Query: '{}' -> Detected: {:?}, Expected: {:?}, Confidence: {:.2}",
            query, detected_level, expected_level, confidence
        );
    }

    let accuracy = (correct_predictions as f64) / (total_predictions as f64);
    println!(
        "Audience level detection accuracy: {:.2}%",
        accuracy * 100.0
    );

    // Should achieve >80% accuracy
    assert!(
        accuracy > 0.8,
        "Audience detection accuracy should be >80%, got {:.2}%",
        accuracy * 100.0
    );
}

#[test]
fn test_technical_domain_detection_accuracy() {
    let rules = DomainRules::new();
    let test_cases = get_domain_test_cases();
    let mut correct_predictions = 0;
    let total_predictions = test_cases.len();

    for (query, expected_domain, expected_min_confidence) in test_cases {
        let result = rules
            .detect_technical_domain(query)
            .expect("Domain detection should succeed");

        let (detected_domain, confidence, keywords) = result;

        // Check if prediction is correct
        if detected_domain == expected_domain {
            correct_predictions += 1;
        }

        // Check confidence is reasonable
        assert!(
            confidence >= 0.0 && confidence <= 1.0,
            "Confidence should be between 0 and 1, got {}",
            confidence
        );

        // For clear cases, confidence should meet minimum threshold
        if expected_min_confidence > 0.5 {
            assert!(
                confidence >= expected_min_confidence * 0.8,
                "Confidence for '{}' should be >= {:.2}, got {:.2}",
                query,
                expected_min_confidence * 0.8,
                confidence
            );
        }

        // Should have matched some keywords
        assert!(
            !keywords.is_empty(),
            "Should have matched keywords for '{}'",
            query
        );

        println!(
            "Query: '{}' -> Detected: {:?}, Expected: {:?}, Confidence: {:.2}",
            query, detected_domain, expected_domain, confidence
        );
    }

    let accuracy = (correct_predictions as f64) / (total_predictions as f64);
    println!(
        "Technical domain detection accuracy: {:.2}%",
        accuracy * 100.0
    );

    // Should achieve >80% accuracy
    assert!(
        accuracy > 0.8,
        "Domain detection accuracy should be >80%, got {:.2}%",
        accuracy * 100.0
    );
}

#[test]
fn test_urgency_level_detection_accuracy() {
    let rules = UrgencyRules::new();
    let test_cases = get_urgency_test_cases();
    let mut correct_predictions = 0;
    let total_predictions = test_cases.len();

    for (query, expected_urgency, expected_min_confidence) in test_cases {
        let result = rules
            .detect_urgency_level(query)
            .expect("Urgency detection should succeed");

        let (detected_urgency, confidence, keywords) = result;

        // Check if prediction is correct
        if detected_urgency == expected_urgency {
            correct_predictions += 1;
        }

        // Check confidence is reasonable
        assert!(
            confidence >= 0.0 && confidence <= 1.0,
            "Confidence should be between 0 and 1, got {}",
            confidence
        );

        // For clear cases, confidence should meet minimum threshold
        if expected_min_confidence > 0.5 {
            assert!(
                confidence >= expected_min_confidence * 0.8,
                "Confidence for '{}' should be >= {:.2}, got {:.2}",
                query,
                expected_min_confidence * 0.8,
                confidence
            );
        }

        // Should have matched some keywords
        assert!(
            !keywords.is_empty(),
            "Should have matched keywords for '{}'",
            query
        );

        println!(
            "Query: '{}' -> Detected: {:?}, Expected: {:?}, Confidence: {:.2}",
            query, detected_urgency, expected_urgency, confidence
        );
    }

    let accuracy = (correct_predictions as f64) / (total_predictions as f64);
    println!("Urgency level detection accuracy: {:.2}%", accuracy * 100.0);

    // Should achieve >80% accuracy
    assert!(
        accuracy > 0.8,
        "Urgency detection accuracy should be >80%, got {:.2}%",
        accuracy * 100.0
    );
}

#[test]
fn test_comprehensive_context_detection() {
    let detector = create_default_test_detector();

    // Test comprehensive context detection with realistic scenarios
    let test_scenarios = vec![
        (
            "I'm new to Rust and need help implementing async functions for web development",
            ResearchType::Implementation,
            AudienceLevel::Beginner,
            TechnicalDomain::Rust,
            UrgencyLevel::Exploratory,
        ),
        (
            "URGENT: Production React app has performance issues, need expert help",
            ResearchType::Troubleshooting,
            AudienceLevel::Intermediate,
            TechnicalDomain::Web,
            UrgencyLevel::Immediate,
        ),
        (
            "I'm exploring advanced microservices patterns for enterprise architecture",
            ResearchType::Learning,
            AudienceLevel::Advanced,
            TechnicalDomain::Architecture,
            UrgencyLevel::Exploratory,
        ),
        (
            "Need to choose between PostgreSQL and MongoDB for my project deadline",
            ResearchType::Decision,
            AudienceLevel::Intermediate,
            TechnicalDomain::Database,
            UrgencyLevel::Planned,
        ),
        (
            "I want to validate my Python machine learning pipeline architecture",
            ResearchType::Validation,
            AudienceLevel::Advanced,
            TechnicalDomain::AI,
            UrgencyLevel::Planned,
        ),
    ];

    for (query, research_type, expected_audience, expected_domain, expected_urgency) in
        test_scenarios
    {
        let result = detector
            .detect_context(query, &research_type)
            .expect("Context detection should succeed");

        println!("Query: '{}'", query);
        println!(
            "  Expected: audience={:?}, domain={:?}, urgency={:?}",
            expected_audience, expected_domain, expected_urgency
        );
        println!(
            "  Detected: audience={:?}, domain={:?}, urgency={:?}",
            result.audience_level, result.technical_domain, result.urgency_level
        );
        println!(
            "  Confidence: {:.2}, Fallback: {}, Time: {}ms",
            result.overall_confidence, result.fallback_used, result.processing_time_ms
        );
        println!();

        // Verify basic result structure
        assert!(result.overall_confidence >= 0.0 && result.overall_confidence <= 1.0);
        assert!(!result.dimension_confidences.is_empty());
        assert!(result.processing_time_ms > 0);
        assert!(result.processing_time_ms < 1000); // Should be fast

        // For clear cases, check if detection is reasonably accurate
        // We allow some flexibility since context detection can be subjective
        let audience_correct = result.audience_level == expected_audience;
        let domain_correct = result.technical_domain == expected_domain;
        let urgency_correct = result.urgency_level == expected_urgency;

        let accuracy_score =
            (audience_correct as u32 + domain_correct as u32 + urgency_correct as u32) as f64 / 3.0;
        assert!(
            accuracy_score >= 0.5,
            "Context detection should be at least 50% accurate, got {:.2}%",
            accuracy_score * 100.0
        );
    }
}

#[test]
fn test_context_detection_confidence_scoring() {
    let detector = create_default_test_detector();

    // Test queries with different confidence levels
    let confidence_tests = vec![
        (
            "I'm a complete beginner learning Rust async programming",
            0.8,
        ), // High confidence
        ("I need help with programming", 0.4), // Medium confidence
        ("Something is wrong", 0.2),           // Low confidence
        ("URGENT: Critical production bug in React app", 0.9), // High confidence
        ("I'm exploring different options", 0.6), // Medium confidence
        ("Help me", 0.1),                      // Very low confidence
    ];

    for (query, expected_min_confidence) in confidence_tests {
        let result = detector
            .detect_context(query, &ResearchType::Learning)
            .expect("Context detection should succeed");

        println!(
            "Query: '{}' -> Confidence: {:.2} (expected >= {:.2})",
            query, result.overall_confidence, expected_min_confidence
        );

        // Confidence should be reasonable for the query clarity
        if expected_min_confidence > 0.5 {
            assert!(
                result.overall_confidence >= expected_min_confidence * 0.8,
                "High clarity query should have high confidence"
            );
        }

        // All queries should have some confidence
        assert!(
            result.overall_confidence > 0.0,
            "Should have some confidence"
        );

        // Check dimension confidences are consistent
        let dimension_avg = result
            .dimension_confidences
            .iter()
            .map(|dc| dc.confidence)
            .sum::<f64>()
            / result.dimension_confidences.len() as f64;

        assert!(
            (result.overall_confidence - dimension_avg).abs() < 0.1,
            "Overall confidence should be consistent with dimension confidences"
        );
    }
}

#[test]
fn test_context_detection_fallback_mechanisms() {
    let detector = create_default_test_detector();

    // Test with different research types to verify fallback logic
    let fallback_tests = vec![
        (
            "vague query",
            ResearchType::Learning,
            AudienceLevel::Beginner,
            UrgencyLevel::Exploratory,
        ),
        (
            "unclear request",
            ResearchType::Troubleshooting,
            AudienceLevel::Intermediate,
            UrgencyLevel::Immediate,
        ),
        (
            "ambiguous question",
            ResearchType::Decision,
            AudienceLevel::Advanced,
            UrgencyLevel::Planned,
        ),
        (
            "random text xyz",
            ResearchType::Implementation,
            AudienceLevel::Intermediate,
            UrgencyLevel::Planned,
        ),
        (
            "???",
            ResearchType::Validation,
            AudienceLevel::Advanced,
            UrgencyLevel::Planned,
        ),
    ];

    for (query, research_type, expected_audience, expected_urgency) in fallback_tests {
        let result = detector
            .detect_context(query, &research_type)
            .expect("Context detection should succeed even with fallback");

        println!("Query: '{}' ({})", query, research_type);
        println!(
            "  Detected: audience={:?}, domain={:?}, urgency={:?}",
            result.audience_level, result.technical_domain, result.urgency_level
        );
        println!(
            "  Fallback used: {}, Confidence: {:.2}",
            result.fallback_used, result.overall_confidence
        );

        // Fallback should be used for vague queries
        assert!(
            result.fallback_used,
            "Fallback should be used for vague queries"
        );

        // Fallback should use research type to infer context
        assert_eq!(
            result.audience_level, expected_audience,
            "Fallback should infer audience from research type"
        );
        assert_eq!(
            result.urgency_level, expected_urgency,
            "Fallback should infer urgency from research type"
        );

        // Should default to General domain for unclear queries
        assert_eq!(
            result.technical_domain,
            TechnicalDomain::General,
            "Fallback should default to General domain"
        );

        // Fallback should have lower confidence
        assert!(
            result.overall_confidence < 0.5,
            "Fallback should have lower confidence"
        );
    }
}

#[test]
fn test_context_detection_edge_cases() {
    let detector = create_default_test_detector();

    // Test edge cases that might cause issues
    let edge_cases = vec![
        "",                        // Empty string
        "   ",                     // Whitespace only
        "a",                       // Single character
        "123",                     // Numbers only
        "!@#$%",                   // Special characters only
        "A".repeat(1000),          // Very long string
        "Mixed CASE and numb3rs!", // Mixed case and numbers
    ];

    for query in edge_cases {
        if query.trim().is_empty() {
            // Empty queries should fail
            let result = detector.detect_context(query, &ResearchType::Learning);
            assert!(result.is_err(), "Empty query should fail");
            continue;
        }

        let result = detector.detect_context(query, &ResearchType::Learning);

        // Should handle edge cases gracefully
        assert!(result.is_ok(), "Should handle edge case: '{}'", query);

        let context = result.unwrap();

        // Should have fallback results
        assert!(context.fallback_used, "Should use fallback for edge cases");
        assert!(
            context.overall_confidence >= 0.0,
            "Should have some confidence"
        );
        assert!(
            !context.dimension_confidences.is_empty(),
            "Should have dimension confidences"
        );

        println!(
            "Edge case: '{}' -> Confidence: {:.2}, Fallback: {}",
            query, context.overall_confidence, context.fallback_used
        );
    }
}

#[test]
fn test_context_detection_performance() {
    let detector = create_default_test_detector();

    // Test performance with various query lengths and complexities
    let performance_queries = vec![
        "Short query",
        "Medium length query about programming concepts",
        "Very long and detailed query about implementing advanced asynchronous programming patterns in Rust for building high-performance web applications with complex business logic",
        "Query with many technical terms: async, await, tokio, futures, streams, channels, actors, microservices, distributed systems, load balancing, caching, monitoring, logging, tracing, metrics",
    ];

    for query in performance_queries {
        let start_time = std::time::Instant::now();

        let result = detector
            .detect_context(query, &ResearchType::Implementation)
            .expect("Context detection should succeed");

        let duration = start_time.elapsed();

        println!(
            "Query length: {} chars, Time: {}ms, Confidence: {:.2}",
            query.len(),
            duration.as_millis(),
            result.overall_confidence
        );

        // Should be fast regardless of query length
        assert!(
            duration.as_millis() < 100,
            "Context detection should be <100ms, got {}ms",
            duration.as_millis()
        );

        // Should return valid results
        assert!(result.processing_time_ms > 0);
        assert!(result.processing_time_ms < 100);
        assert!(!result.dimension_confidences.is_empty());
    }
}

#[test]
fn test_context_detection_consistency() {
    let detector = create_default_test_detector();

    // Test consistency across multiple runs
    let test_query = "I'm learning Rust and need help with async programming";
    let mut results = Vec::new();

    for _ in 0..10 {
        let result = detector
            .detect_context(test_query, &ResearchType::Learning)
            .expect("Context detection should succeed");
        results.push(result);
    }

    // Results should be consistent
    let first_result = &results[0];

    for (i, result) in results.iter().enumerate().skip(1) {
        assert_eq!(
            result.audience_level, first_result.audience_level,
            "Run {} should have same audience level as first run",
            i
        );
        assert_eq!(
            result.technical_domain, first_result.technical_domain,
            "Run {} should have same technical domain as first run",
            i
        );
        assert_eq!(
            result.urgency_level, first_result.urgency_level,
            "Run {} should have same urgency level as first run",
            i
        );

        // Confidence should be very similar (within 10%)
        let confidence_diff = (result.overall_confidence - first_result.overall_confidence).abs();
        assert!(
            confidence_diff < 0.1,
            "Run {} confidence should be within 10% of first run",
            i
        );
    }

    println!(
        "Consistency test passed: {} runs with same results",
        results.len()
    );
}

#[test]
fn test_context_detection_dimension_confidence() {
    let detector = create_default_test_detector();

    // Test that dimension confidences are properly calculated
    let test_query = "I'm an expert in Rust and need urgent help with production issues";

    let result = detector
        .detect_context(test_query, &ResearchType::Troubleshooting)
        .expect("Context detection should succeed");

    println!("Query: '{}'", test_query);
    println!("Overall confidence: {:.2}", result.overall_confidence);
    println!("Dimension confidences:");

    for dc in &result.dimension_confidences {
        println!(
            "  {:?}: {:.2} (keywords: {:?})",
            dc.dimension, dc.confidence, dc.matched_keywords
        );

        // Each dimension should have reasonable confidence
        assert!(
            dc.confidence >= 0.0 && dc.confidence <= 1.0,
            "Dimension confidence should be between 0 and 1"
        );

        // Should have matched keywords
        assert!(
            !dc.matched_keywords.is_empty(),
            "Dimension should have matched keywords"
        );

        // Should have explanation
        assert!(
            !dc.explanation.is_empty(),
            "Dimension should have explanation"
        );
    }

    // Should have confidences for multiple dimensions
    assert!(
        result.dimension_confidences.len() >= 2,
        "Should have confidences for multiple dimensions"
    );

    // Overall confidence should be reasonable average
    let avg_confidence = result
        .dimension_confidences
        .iter()
        .map(|dc| dc.confidence)
        .sum::<f64>()
        / result.dimension_confidences.len() as f64;

    assert!(
        (result.overall_confidence - avg_confidence).abs() < 0.1,
        "Overall confidence should be close to average of dimension confidences"
    );

    // Test dimension confidence accessor
    let audience_confidence =
        result.get_dimension_confidence(&ClassificationDimension::AudienceLevel);
    assert!(
        audience_confidence.is_some(),
        "Should have audience confidence"
    );

    let domain_confidence =
        result.get_dimension_confidence(&ClassificationDimension::TechnicalDomain);
    assert!(domain_confidence.is_some(), "Should have domain confidence");

    let urgency_confidence = result.get_dimension_confidence(&ClassificationDimension::Urgency);
    assert!(
        urgency_confidence.is_some(),
        "Should have urgency confidence"
    );
}

#[test]
fn test_context_detection_configuration_impact() {
    // Test with different configurations
    let configs = vec![
        (
            "High threshold",
            ContextDetectionConfig {
                confidence_threshold: 0.9,
                enable_fallback: false,
                max_processing_time_ms: 100,
                debug_logging: false,
            },
        ),
        (
            "Low threshold",
            ContextDetectionConfig {
                confidence_threshold: 0.1,
                enable_fallback: true,
                max_processing_time_ms: 100,
                debug_logging: false,
            },
        ),
        (
            "No fallback",
            ContextDetectionConfig {
                confidence_threshold: 0.5,
                enable_fallback: false,
                max_processing_time_ms: 100,
                debug_logging: false,
            },
        ),
    ];

    let test_query = "I need some help with programming";

    for (config_name, config) in configs {
        let detector = create_test_context_detector(config.clone());

        let result = detector.detect_context(test_query, &ResearchType::Learning);

        println!("Config: {}", config_name);
        println!(
            "  Threshold: {:.2}, Fallback: {}",
            config.confidence_threshold, config.enable_fallback
        );

        if config.enable_fallback {
            assert!(result.is_ok(), "Should succeed with fallback enabled");
            let context = result.unwrap();
            println!(
                "  Result: confidence={:.2}, fallback={}",
                context.overall_confidence, context.fallback_used
            );
        } else {
            println!("  Result: {:?}", result.is_ok());
        }

        println!();
    }
}
