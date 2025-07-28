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

// ABOUTME: Test fixtures for multi-dimensional classification testing
use fortitude_types::{
    classification::{ClassificationCandidate, ClassificationConfig},
    classification_result::*,
    research::ResearchType,
};
use std::collections::HashMap;

/// Create a test enhanced classification result for implementation queries
pub fn create_implementation_classification() -> EnhancedClassificationResult {
    let dimension_confidences = vec![
        DimensionConfidence::new(
            ClassificationDimension::ResearchType,
            0.9,
            vec!["implement".to_string(), "build".to_string()],
            "Strong implementation keywords detected".to_string(),
        ),
        DimensionConfidence::new(
            ClassificationDimension::TechnicalDomain,
            0.85,
            vec!["rust".to_string(), "async".to_string()],
            "Rust-specific technical terms identified".to_string(),
        ),
        DimensionConfidence::new(
            ClassificationDimension::AudienceLevel,
            0.7,
            vec!["how to".to_string()],
            "Beginner-level language patterns".to_string(),
        ),
        DimensionConfidence::new(
            ClassificationDimension::Urgency,
            0.6,
            vec!["need".to_string()],
            "Moderate urgency indicators".to_string(),
        ),
    ];

    let candidates = vec![
        ClassificationCandidate::new(
            ResearchType::Implementation,
            0.9,
            vec!["implement".to_string(), "build".to_string()],
            1,
        ),
        ClassificationCandidate::new(ResearchType::Learning, 0.4, vec!["how to".to_string()], 1),
    ];

    let metadata = ClassificationMetadata {
        processing_time_ms: 15,
        algorithm: "advanced_multi_dimensional".to_string(),
        ..Default::default()
    };

    EnhancedClassificationResult::new(
        ResearchType::Implementation,
        0.88,
        AudienceLevel::Beginner,
        TechnicalDomain::Rust,
        UrgencyLevel::Planned,
        dimension_confidences,
        vec![
            "implement".to_string(),
            "build".to_string(),
            "rust".to_string(),
        ],
        1,
        candidates,
        metadata,
    )
}

/// Create a test enhanced classification result for troubleshooting queries
pub fn create_troubleshooting_classification() -> EnhancedClassificationResult {
    let dimension_confidences = vec![
        DimensionConfidence::new(
            ClassificationDimension::ResearchType,
            0.95,
            vec!["error".to_string(), "debug".to_string(), "fix".to_string()],
            "Clear troubleshooting indicators present".to_string(),
        ),
        DimensionConfidence::new(
            ClassificationDimension::TechnicalDomain,
            0.8,
            vec!["cargo".to_string(), "rust".to_string()],
            "Rust ecosystem terminology".to_string(),
        ),
        DimensionConfidence::new(
            ClassificationDimension::Urgency,
            0.9,
            vec!["blocking".to_string(), "urgent".to_string()],
            "High urgency language detected".to_string(),
        ),
        DimensionConfidence::new(
            ClassificationDimension::AudienceLevel,
            0.75,
            vec!["advanced".to_string(), "performance".to_string()],
            "Advanced technical terminology".to_string(),
        ),
    ];

    let candidates = vec![
        ClassificationCandidate::new(
            ResearchType::Troubleshooting,
            0.95,
            vec!["error".to_string(), "debug".to_string()],
            2,
        ),
        ClassificationCandidate::new(
            ResearchType::Implementation,
            0.3,
            vec!["fix".to_string()],
            1,
        ),
    ];

    let metadata = ClassificationMetadata {
        processing_time_ms: 12,
        algorithm: "advanced_multi_dimensional".to_string(),
        ..Default::default()
    };

    EnhancedClassificationResult::new(
        ResearchType::Troubleshooting,
        0.93,
        AudienceLevel::Advanced,
        TechnicalDomain::Rust,
        UrgencyLevel::Immediate,
        dimension_confidences,
        vec![
            "error".to_string(),
            "debug".to_string(),
            "cargo".to_string(),
        ],
        2,
        candidates,
        metadata,
    )
}

/// Create a test enhanced classification result for decision queries
pub fn create_decision_classification() -> EnhancedClassificationResult {
    let dimension_confidences = vec![
        DimensionConfidence::new(
            ClassificationDimension::ResearchType,
            0.85,
            vec![
                "choose".to_string(),
                "compare".to_string(),
                "decision".to_string(),
            ],
            "Decision-making keywords present".to_string(),
        ),
        DimensionConfidence::new(
            ClassificationDimension::TechnicalDomain,
            0.7,
            vec!["framework".to_string(), "library".to_string()],
            "General technical decision context".to_string(),
        ),
        DimensionConfidence::new(
            ClassificationDimension::AudienceLevel,
            0.8,
            vec!["pros and cons".to_string(), "evaluate".to_string()],
            "Intermediate analysis language".to_string(),
        ),
        DimensionConfidence::new(
            ClassificationDimension::Urgency,
            0.65,
            vec!["considering".to_string()],
            "Exploratory decision-making".to_string(),
        ),
    ];

    let candidates = vec![
        ClassificationCandidate::new(
            ResearchType::Decision,
            0.85,
            vec!["choose".to_string(), "compare".to_string()],
            1,
        ),
        ClassificationCandidate::new(
            ResearchType::Learning,
            0.5,
            vec!["pros and cons".to_string()],
            1,
        ),
    ];

    let metadata = ClassificationMetadata {
        processing_time_ms: 18,
        algorithm: "advanced_multi_dimensional".to_string(),
        ..Default::default()
    };

    EnhancedClassificationResult::new(
        ResearchType::Decision,
        0.82,
        AudienceLevel::Intermediate,
        TechnicalDomain::General,
        UrgencyLevel::Exploratory,
        dimension_confidences,
        vec![
            "choose".to_string(),
            "compare".to_string(),
            "framework".to_string(),
        ],
        1,
        candidates,
        metadata,
    )
}

/// Create a test enhanced classification result for learning queries
pub fn create_learning_classification() -> EnhancedClassificationResult {
    let dimension_confidences = vec![
        DimensionConfidence::new(
            ClassificationDimension::ResearchType,
            0.88,
            vec![
                "what is".to_string(),
                "explain".to_string(),
                "understand".to_string(),
            ],
            "Clear learning intent keywords".to_string(),
        ),
        DimensionConfidence::new(
            ClassificationDimension::TechnicalDomain,
            0.9,
            vec!["blockchain".to_string(), "consensus".to_string()],
            "Blockchain/crypto domain terminology".to_string(),
        ),
        DimensionConfidence::new(
            ClassificationDimension::AudienceLevel,
            0.85,
            vec!["beginner".to_string(), "introduction".to_string()],
            "Beginner-level learning request".to_string(),
        ),
        DimensionConfidence::new(
            ClassificationDimension::Urgency,
            0.5,
            vec!["eventually".to_string()],
            "Low urgency exploration".to_string(),
        ),
    ];

    let candidates = vec![
        ClassificationCandidate::new(
            ResearchType::Learning,
            0.88,
            vec!["what is".to_string(), "explain".to_string()],
            1,
        ),
        ClassificationCandidate::new(
            ResearchType::Implementation,
            0.2,
            vec!["understand".to_string()],
            1,
        ),
    ];

    let mut metadata = ClassificationMetadata {
        processing_time_ms: 14,
        algorithm: "advanced_multi_dimensional".to_string(),
        ..Default::default()
    };
    metadata
        .tags
        .insert("domain".to_string(), "blockchain".to_string());

    EnhancedClassificationResult::new(
        ResearchType::Learning,
        0.86,
        AudienceLevel::Beginner,
        TechnicalDomain::Security, // Blockchain falls under security domain
        UrgencyLevel::Exploratory,
        dimension_confidences,
        vec![
            "what is".to_string(),
            "blockchain".to_string(),
            "beginner".to_string(),
        ],
        1,
        candidates,
        metadata,
    )
}

/// Create a test enhanced classification result for validation queries
pub fn create_validation_classification() -> EnhancedClassificationResult {
    let dimension_confidences = vec![
        DimensionConfidence::new(
            ClassificationDimension::ResearchType,
            0.92,
            vec![
                "test".to_string(),
                "validate".to_string(),
                "verify".to_string(),
            ],
            "Strong validation keywords detected".to_string(),
        ),
        DimensionConfidence::new(
            ClassificationDimension::TechnicalDomain,
            0.88,
            vec![
                "api".to_string(),
                "performance".to_string(),
                "load".to_string(),
            ],
            "Web/API testing context".to_string(),
        ),
        DimensionConfidence::new(
            ClassificationDimension::AudienceLevel,
            0.8,
            vec!["best practices".to_string(), "comprehensive".to_string()],
            "Advanced testing methodology".to_string(),
        ),
        DimensionConfidence::new(
            ClassificationDimension::Urgency,
            0.75,
            vec!["production".to_string(), "deploy".to_string()],
            "Production deployment urgency".to_string(),
        ),
    ];

    let candidates = vec![
        ClassificationCandidate::new(
            ResearchType::Validation,
            0.92,
            vec!["test".to_string(), "validate".to_string()],
            1,
        ),
        ClassificationCandidate::new(
            ResearchType::Implementation,
            0.4,
            vec!["best practices".to_string()],
            1,
        ),
    ];

    let mut metadata = ClassificationMetadata {
        processing_time_ms: 16,
        algorithm: "advanced_multi_dimensional".to_string(),
        ..Default::default()
    };
    metadata
        .tags
        .insert("context".to_string(), "production".to_string());

    EnhancedClassificationResult::new(
        ResearchType::Validation,
        0.89,
        AudienceLevel::Advanced,
        TechnicalDomain::Web,
        UrgencyLevel::Planned,
        dimension_confidences,
        vec![
            "test".to_string(),
            "api".to_string(),
            "production".to_string(),
        ],
        1,
        candidates,
        metadata,
    )
}

/// Create test classification config for advanced classification
pub fn create_advanced_classification_config() -> ClassificationConfig {
    ClassificationConfig {
        default_threshold: 0.7,
        fallback_type: ResearchType::Learning,
        enable_fuzzy_matching: true,
        max_candidates: 5,
    }
}

/// Create test classification config with low threshold for testing
pub fn create_low_threshold_config() -> ClassificationConfig {
    ClassificationConfig {
        default_threshold: 0.1,
        fallback_type: ResearchType::Learning,
        enable_fuzzy_matching: false,
        max_candidates: 10,
    }
}

/// Test queries for multi-dimensional classification testing
pub struct TestQueries;

impl TestQueries {
    /// Get implementation test queries
    pub fn implementation() -> Vec<&'static str> {
        vec![
            "How to implement async functions in Rust?",
            "Build a REST API with authentication",
            "Create a CLI tool with clap",
            "Implement a concurrent file processor",
            "How to build a web scraper with tokio",
        ]
    }

    /// Get troubleshooting test queries
    pub fn troubleshooting() -> Vec<&'static str> {
        vec![
            "Getting compile error in my Rust code",
            "Debug performance issue with async code",
            "Fix memory leak in long-running service",
            "Cargo build fails with linking errors",
            "Runtime panic in production deployment",
        ]
    }

    /// Get decision test queries
    pub fn decision() -> Vec<&'static str> {
        vec![
            "Should I choose Tokio or async-std?",
            "Compare Actix vs Warp for web APIs",
            "Which database is best for my use case?",
            "Pros and cons of microservices vs monolith",
            "Evaluate Rust vs Go for backend services",
        ]
    }

    /// Get learning test queries
    pub fn learning() -> Vec<&'static str> {
        vec![
            "What is ownership in Rust?",
            "Explain the actor model pattern",
            "Introduction to blockchain consensus",
            "Understand async/await in depth",
            "What are the principles of clean architecture?",
        ]
    }

    /// Get validation test queries
    pub fn validation() -> Vec<&'static str> {
        vec![
            "How to test async Rust code effectively?",
            "Validate API performance under load",
            "Best practices for integration testing",
            "Verify security of authentication system",
            "Test concurrent data structures for correctness",
        ]
    }

    /// Get all test queries organized by type
    pub fn all_by_type() -> HashMap<ResearchType, Vec<&'static str>> {
        let mut queries = HashMap::new();
        queries.insert(ResearchType::Implementation, Self::implementation());
        queries.insert(ResearchType::Troubleshooting, Self::troubleshooting());
        queries.insert(ResearchType::Decision, Self::decision());
        queries.insert(ResearchType::Learning, Self::learning());
        queries.insert(ResearchType::Validation, Self::validation());
        queries
    }

    /// Get mixed complexity queries for testing edge cases
    pub fn mixed_complexity() -> Vec<&'static str> {
        vec![
            "How to debug and implement error handling?", // Troubleshooting + Implementation
            "What is the best way to test async code?",   // Learning + Validation
            "Should I fix this bug or redesign the system?", // Decision + Troubleshooting
            "Explain how to benchmark performance testing", // Learning + Validation
            "Choose and implement a caching strategy",    // Decision + Implementation
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_implementation_classification() {
        let result = create_implementation_classification();
        assert_eq!(result.research_type, ResearchType::Implementation);
        assert_eq!(result.technical_domain, TechnicalDomain::Rust);
        assert_eq!(result.audience_level, AudienceLevel::Beginner);
        assert!(result.overall_confidence > 0.8);
        assert_eq!(result.dimension_confidences.len(), 4);
    }

    #[test]
    fn test_create_troubleshooting_classification() {
        let result = create_troubleshooting_classification();
        assert_eq!(result.research_type, ResearchType::Troubleshooting);
        assert_eq!(result.urgency_level, UrgencyLevel::Immediate);
        assert_eq!(result.audience_level, AudienceLevel::Advanced);
        assert!(result.overall_confidence > 0.9);
    }

    #[test]
    fn test_test_queries_structure() {
        let all_queries = TestQueries::all_by_type();
        assert_eq!(all_queries.len(), 5);

        // Verify each type has queries
        for research_type in ResearchType::all() {
            assert!(all_queries.contains_key(&research_type));
            assert!(!all_queries[&research_type].is_empty());
        }
    }

    #[test]
    fn test_mixed_complexity_queries() {
        let queries = TestQueries::mixed_complexity();
        assert!(queries.len() >= 5);

        // These should be queries that could match multiple types
        for query in queries {
            assert!(!query.is_empty());
            assert!(query.len() > 20); // Should be substantial queries
        }
    }

    #[test]
    fn test_advanced_config() {
        let config = create_advanced_classification_config();
        assert_eq!(config.default_threshold, 0.7);
        assert!(config.enable_fuzzy_matching);
        assert_eq!(config.max_candidates, 5);
    }
}
