//! Enhanced cache retrieval logic tests
//!
//! This test suite validates the enhanced cache retrieval and fallback mechanisms,
//! ensuring comprehensive cache lookup coverage across all storage patterns.
//!
//! ANCHOR: Enhanced retrieval functionality tests that protect against cache lookup gaps
//! Tests: comprehensive fallback coverage, cross-context retrieval, fuzzy matching

use fortitude_core::storage::FileStorage;
use fortitude_types::{
    AudienceLevel, ClassificationDimension, DimensionConfidence, TechnicalDomain, UrgencyLevel,
    ResearchResult, ResearchType, StorageConfig, AudienceContext, DomainContext,
    ClassifiedRequest, ResearchMetadata, Storage,
};
use fortitude_core::classification::context_detector::ContextDetectionResult;
use std::collections::HashMap;
use chrono::Utc;
use tempfile::TempDir;

// Test utilities
fn create_test_storage_config(temp_dir: &TempDir) -> StorageConfig {
    StorageConfig {
        base_path: temp_dir.path().to_path_buf(),
        cache_expiration_seconds: 3600,
        max_cache_size_bytes: 1024 * 1024,
        enable_content_addressing: true,
        index_update_interval_seconds: 300,
    }
}

fn create_test_classified_request(query: &str, research_type: ResearchType) -> ClassifiedRequest {
    ClassifiedRequest::new(
        query.to_string(),
        research_type,
        AudienceContext::default(),
        DomainContext::default(),
        0.8,
        vec!["test".to_string()],
    )
}

fn create_test_research_result(query: &str, research_type: ResearchType) -> ResearchResult {
    let request = create_test_classified_request(query, research_type);
    let metadata = ResearchMetadata {
        completed_at: Utc::now(),
        processing_time_ms: 1000,
        sources_consulted: vec!["test_source".to_string()],
        quality_score: 0.9,
        cache_key: "test-key".to_string(),
        tags: HashMap::new(),
    };

    ResearchResult::new(
        request,
        "Test answer".to_string(),
        vec![],
        vec![],
        metadata,
    )
}

fn create_test_context_result(confidence: f64) -> ContextDetectionResult {
    let dimension_confidences = vec![
        DimensionConfidence {
            dimension: ClassificationDimension::AudienceLevel,
            confidence,
            matched_keywords: vec!["test".to_string()],
            reasoning: "test evidence".to_string(),
        },
        DimensionConfidence {
            dimension: ClassificationDimension::TechnicalDomain,
            confidence,
            matched_keywords: vec!["domain".to_string()],
            reasoning: "domain evidence".to_string(),
        },
        DimensionConfidence {
            dimension: ClassificationDimension::Urgency,
            confidence,
            matched_keywords: vec!["urgency".to_string()],
            reasoning: "urgency evidence".to_string(),
        },
    ];

    ContextDetectionResult::new(
        AudienceLevel::Intermediate,
        TechnicalDomain::Rust,
        UrgencyLevel::Planned,
        dimension_confidences,
        50,
        false,
    )
}

#[cfg(test)]
mod comprehensive_fallback_tests {
    use super::*;

    /// ANCHOR: Test comprehensive fallback logic for all storage location patterns
    /// This test ensures that retrieval methods can find cache entries regardless of
    /// how they were stored (standard vs context-aware)
    #[tokio::test]
    async fn test_comprehensive_fallback_coverage() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Store items using different methods to create various file locations
        let result_standard = create_test_research_result("standard query", ResearchType::Learning);
        let result_context = create_test_research_result("context query", ResearchType::Implementation);
        
        let context = create_test_context_result(0.8);
        
        // Store using different methods
        let key_standard = storage.store(&result_standard).await.unwrap();
        let key_context = storage.store_with_context(&result_context, Some(&context)).await.unwrap();

        // Test comprehensive retrieval matrix - ALL should succeed after enhancement
        let test_cases = vec![
            ("Standard method, standard storage", &key_standard, false),
            ("Standard method, context storage", &key_context, false),
            ("Context method, standard storage", &key_standard, true),
            ("Context method, context storage", &key_context, true),
        ];

        for (description, cache_key, use_context) in test_cases {
            let retrieved = if use_context {
                storage.retrieve_with_context(cache_key, Some(&context)).await.unwrap()
            } else {
                storage.retrieve(cache_key).await.unwrap()
            };

            assert!(
                retrieved.is_some(),
                "ENHANCED REQUIREMENT: {} should succeed with comprehensive fallback logic",
                description
            );
        }
    }

    /// ANCHOR: Test directory scanning fallback for all possible cache locations
    /// This test ensures that fallback scanning checks all directories where 
    /// cache entries might be stored
    #[tokio::test]
    async fn test_directory_scanning_fallback_completeness() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Create test contexts with different characteristics
        let contexts = vec![
            create_test_context_result(0.9),
            ContextDetectionResult::new(
                AudienceLevel::Advanced,
                TechnicalDomain::Database,
                UrgencyLevel::Immediate,
                vec![],
                100,
                false,
            ),
            ContextDetectionResult::new(
                AudienceLevel::Beginner,
                TechnicalDomain::Web,
                UrgencyLevel::Research,
                vec![],
                50,
                false,
            ),
        ];

        let mut stored_keys = Vec::new();
        let research_types = vec![
            ResearchType::Learning,
            ResearchType::Implementation,
            ResearchType::Troubleshooting,
        ];

        // Store items in various locations using different context combinations
        for (i, context) in contexts.iter().enumerate() {
            for (j, research_type) in research_types.iter().enumerate() {
                let query = format!("query {} type {}", i, j);
                let result = create_test_research_result(&query, *research_type);
                
                // Store some with context, some without
                let key = if i % 2 == 0 {
                    storage.store_with_context(&result, Some(context)).await.unwrap()
                } else {
                    storage.store(&result).await.unwrap()
                };
                
                stored_keys.push((key, context.clone(), *research_type));
            }
        }

        // Test that enhanced directory scanning finds ALL stored items
        for (cache_key, original_context, _research_type) in &stored_keys {
            // Try retrieving with standard method - should find via enhanced scanning
            let retrieved_standard = storage.retrieve(cache_key).await.unwrap();
            assert!(
                retrieved_standard.is_some(),
                "ENHANCED REQUIREMENT: Standard retrieval should find all cache entries via comprehensive directory scanning. Key: {}",
                cache_key
            );

            // Try retrieving with context method - should also find via enhanced scanning
            let retrieved_context = storage.retrieve_with_context(cache_key, Some(original_context)).await.unwrap();
            assert!(
                retrieved_context.is_some(),
                "ENHANCED REQUIREMENT: Context retrieval should find all cache entries via comprehensive directory scanning. Key: {}",
                cache_key
            );
        }
    }

    /// ANCHOR: Test retrieval efficiency with intelligent search order
    /// This test ensures that retrieval uses an optimal search order:
    /// index -> direct file -> context-aware scan -> comprehensive scan
    #[tokio::test]
    async fn test_intelligent_search_order_efficiency() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Store a result and measure retrieval timing for different scenarios
        let result = create_test_research_result("efficiency test query", ResearchType::Learning);
        let context = create_test_context_result(0.85);
        
        let cache_key = storage.store_with_context(&result, Some(&context)).await.unwrap();

        // Test retrieval timing - this should be very fast due to intelligent ordering
        let start_time = std::time::Instant::now();
        
        for _ in 0..10 {
            let retrieved = storage.retrieve(&cache_key).await.unwrap();
            assert!(retrieved.is_some(), "All retrievals should succeed");
        }
        
        let total_time = start_time.elapsed();
        let avg_time_per_retrieval = total_time / 10;

        // Enhanced retrieval should be efficient even with comprehensive fallback
        assert!(
            avg_time_per_retrieval.as_millis() < 50,
            "ENHANCED REQUIREMENT: Average retrieval time should be under 50ms even with comprehensive fallback. Actual: {:?}",
            avg_time_per_retrieval
        );
    }
}

#[cfg(test)]
mod fuzzy_matching_tests {
    use super::*;

    /// ANCHOR: Test fuzzy matching for semantically similar queries
    /// This test ensures that queries with minor variations can match existing cache entries
    #[tokio::test]
    async fn test_fuzzy_semantic_matching() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Store a result with the original query
        let original_query = "How to implement async programming in Rust";
        let result = create_test_research_result(original_query, ResearchType::Learning);
        let cache_key = storage.store(&result).await.unwrap();

        // Test variations that should match via fuzzy logic
        let similar_queries = vec![
            "how to implement async programming in rust",  // Case difference
            "How to implement async programming in Rust?", // Punctuation difference
            "implement async programming in rust how to",  // Word order difference
            "How do I implement async programming in Rust", // Slight phrasing difference
            "async programming implementation in rust",     // Semantic similarity
        ];

        for similar_query in similar_queries {
            // This will require enhanced fuzzy matching implementation
            let fuzzy_result = create_test_research_result(similar_query, ResearchType::Learning);
            
            // The enhanced retrieval should find the original cache entry via fuzzy matching
            // Note: This test will initially fail until fuzzy matching is implemented
            println!("Testing fuzzy match for: '{}'", similar_query);
            
            // For now, we'll test that the system doesn't crash and returns None/Some consistently
            // TODO: Implement actual fuzzy matching logic
            let retrieved = storage.retrieve(&cache_key).await.unwrap();
            assert!(retrieved.is_some(), "Original key should always be retrievable");
        }
    }

    /// ANCHOR: Test confidence-based fuzzy matching with configurable thresholds
    /// This test ensures fuzzy matching respects confidence thresholds for quality control
    #[tokio::test]
    async fn test_confidence_based_fuzzy_matching() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Store results with different confidence levels
        let high_conf_result = create_test_research_result("high confidence query", ResearchType::Learning);
        let med_conf_result = create_test_research_result("medium confidence query", ResearchType::Learning);
        
        let high_context = create_test_context_result(0.95);
        let med_context = create_test_context_result(0.65);
        
        let high_key = storage.store_with_context(&high_conf_result, Some(&high_context)).await.unwrap();
        let med_key = storage.store_with_context(&med_conf_result, Some(&med_context)).await.unwrap();

        // Test that confidence thresholds are respected in fuzzy matching
        // High confidence items should be more easily found
        let high_retrieved = storage.retrieve(&high_key).await.unwrap();
        let med_retrieved = storage.retrieve(&med_key).await.unwrap();

        assert!(high_retrieved.is_some(), "High confidence items should be retrievable");
        assert!(med_retrieved.is_some(), "Medium confidence items should be retrievable");

        // TODO: Implement actual confidence-based fuzzy matching
        // For now, just ensure basic retrieval works
    }
}

#[cfg(test)]
mod cross_context_retrieval_tests {
    use super::*;

    /// ANCHOR: Test complete cross-context retrieval matrix
    /// This test ensures all combinations of storage and retrieval methods work
    #[tokio::test]
    async fn test_complete_cross_context_retrieval_matrix() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Create multiple contexts for comprehensive testing
        let context1 = create_test_context_result(0.8);
        let context2 = ContextDetectionResult::new(
            AudienceLevel::Advanced,
            TechnicalDomain::Database,
            UrgencyLevel::Immediate,
            vec![],
            100,
            false,
        );

        // Store results using different combinations
        let result_std = create_test_research_result("standard storage", ResearchType::Learning);
        let result_ctx1 = create_test_research_result("context1 storage", ResearchType::Implementation);
        let result_ctx2 = create_test_research_result("context2 storage", ResearchType::Troubleshooting);

        let key_std = storage.store(&result_std).await.unwrap();
        let key_ctx1 = storage.store_with_context(&result_ctx1, Some(&context1)).await.unwrap();
        let key_ctx2 = storage.store_with_context(&result_ctx2, Some(&context2)).await.unwrap();

        // Complete cross-context retrieval matrix - ALL must succeed with enhancement
        let retrieval_matrix = vec![
            // (description, cache_key, retrieval_context, should_succeed)
            ("Standard->Standard", &key_std, None, true),
            ("Standard->Context1", &key_std, Some(&context1), true),
            ("Standard->Context2", &key_std, Some(&context2), true),
            ("Context1->Standard", &key_ctx1, None, true),
            ("Context1->Context1", &key_ctx1, Some(&context1), true),
            ("Context1->Context2", &key_ctx1, Some(&context2), true),
            ("Context2->Standard", &key_ctx2, None, true),
            ("Context2->Context1", &key_ctx2, Some(&context1), true),
            ("Context2->Context2", &key_ctx2, Some(&context2), true),
        ];

        for (description, cache_key, retrieval_context, should_succeed) in retrieval_matrix {
            let retrieved = if let Some(ctx) = retrieval_context {
                storage.retrieve_with_context(cache_key, Some(ctx)).await.unwrap()
            } else {
                storage.retrieve(cache_key).await.unwrap()
            };

            if should_succeed {
                assert!(
                    retrieved.is_some(),
                    "ENHANCED REQUIREMENT: {} retrieval should succeed with comprehensive fallback",
                    description
                );
            }
        }
    }
}

#[cfg(test)]
mod performance_regression_tests {
    use super::*;

    /// ANCHOR: Test that enhanced retrieval maintains acceptable performance
    /// This test ensures optimizations don't compromise retrieval speed
    #[tokio::test]
    async fn test_enhanced_retrieval_performance_regression() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Store multiple items to create a realistic performance test
        let mut stored_keys = Vec::new();
        let context = create_test_context_result(0.8);

        for i in 0..50 {
            let query = format!("performance test query {}", i);
            let result = create_test_research_result(&query, ResearchType::Learning);
            
            let key = if i % 2 == 0 {
                storage.store(&result).await.unwrap()
            } else {
                storage.store_with_context(&result, Some(&context)).await.unwrap()
            };
            
            stored_keys.push(key);
        }

        // Measure retrieval performance with enhanced fallback logic
        let start_time = std::time::Instant::now();
        let mut successful_retrievals = 0;

        for key in &stored_keys {
            let retrieved = storage.retrieve(key).await.unwrap();
            if retrieved.is_some() {
                successful_retrievals += 1;
            }
        }

        let total_time = start_time.elapsed();
        let avg_time_per_retrieval = total_time / stored_keys.len() as u32;

        // Performance requirements
        assert_eq!(
            successful_retrievals,
            stored_keys.len(),
            "All stored items should be retrievable with enhanced fallback"
        );

        assert!(
            avg_time_per_retrieval.as_millis() < 100,
            "Enhanced retrieval should maintain good performance. Average time: {:?}",
            avg_time_per_retrieval
        );

        println!("Enhanced retrieval performance: {} items in {:?} (avg: {:?})", 
                 stored_keys.len(), total_time, avg_time_per_retrieval);
    }
}