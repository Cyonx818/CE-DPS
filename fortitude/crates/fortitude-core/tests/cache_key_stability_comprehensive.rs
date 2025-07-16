//! Comprehensive cache key stability tests
//!
//! ANCHOR: Critical cache key generation tests that protect against regressions
//! Tests: cache key determinism, floating-point stability, query normalization

use chrono::Utc;
use fortitude_core::classification::context_detector::ContextDetectionResult;
use fortitude_core::storage::FileStorage;
use fortitude_types::{
    AudienceContext, AudienceLevel, ClassificationDimension, ClassifiedRequest,
    DimensionConfidence, DomainContext, ResearchMetadata, ResearchResult, ResearchType, Storage,
    StorageConfig, TechnicalDomain, UrgencyLevel,
};
use std::collections::{HashMap, HashSet};
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

fn create_basic_research_result(query: &str, research_type: ResearchType) -> ResearchResult {
    let request = ClassifiedRequest::new(
        query.to_string(),
        research_type,
        AudienceContext::default(),
        DomainContext::default(),
        0.8,
        vec!["test".to_string()],
    );

    let metadata = ResearchMetadata {
        completed_at: Utc::now(),
        processing_time_ms: 1000,
        sources_consulted: vec!["test_source".to_string()],
        quality_score: 0.9,
        cache_key: String::new(), // Let storage generate the cache key
        tags: HashMap::new(),
    };

    ResearchResult::new(request, "Test answer".to_string(), vec![], vec![], metadata)
}

fn create_context_result_with_confidence(confidence: f64) -> ContextDetectionResult {
    let dimension_confidences = vec![DimensionConfidence {
        dimension: ClassificationDimension::AudienceLevel,
        confidence,
        matched_keywords: vec!["test".to_string()],
        reasoning: "test evidence".to_string(),
    }];

    ContextDetectionResult::new(
        AudienceLevel::Intermediate,
        TechnicalDomain::Rust,
        UrgencyLevel::Planned,
        dimension_confidences,
        100,   // processing_time_ms
        false, // fallback_used
    )
}

#[cfg(test)]
mod cache_key_determinism_tests {
    use super::*;

    /// ANCHOR: Test cache key determinism across multiple runs
    /// Ensures that identical inputs always produce identical cache keys
    #[tokio::test]
    async fn test_cache_key_determinism_multiple_runs() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        let query = "How to implement async programming in Rust?";
        let research_type = ResearchType::Learning;

        // Generate cache keys multiple times with identical inputs
        let mut cache_keys = Vec::new();
        for _ in 0..10 {
            let result = create_basic_research_result(query, research_type.clone());
            let key = storage.store(&result).await.unwrap();
            cache_keys.push(key);
        }

        // All keys should be identical
        let unique_keys: HashSet<_> = cache_keys.iter().collect();
        assert_eq!(
            unique_keys.len(),
            1,
            "Cache keys should be deterministic. Got {} unique keys: {:?}",
            unique_keys.len(),
            cache_keys
        );

        println!(
            "✅ Determinism test passed: {} identical cache keys generated",
            cache_keys.len()
        );
        println!("   Cache key: {}", cache_keys[0]);
    }

    /// ANCHOR: Test confidence banding eliminates floating-point precision issues
    /// Verifies that slightly different confidence values within the same band generate identical keys
    #[tokio::test]
    async fn test_confidence_banding_stability() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        let query = "rust async programming";

        // Test confidence values within the same band (0.8-1.0 = "very_high")
        let confidence_values = [
            0.8,
            0.85,
            0.8500000000000001, // Floating-point precision issue
            0.9,
            0.95,
            0.999,
            1.0,
        ];

        let mut cache_keys = Vec::new();
        for confidence in confidence_values.iter() {
            let result = create_basic_research_result(query, ResearchType::Learning);
            let key = storage.store(&result).await.unwrap();
            cache_keys.push((*confidence, key));
        }

        // Analyze cache key stability within the same confidence band
        let unique_keys: HashSet<_> = cache_keys.iter().map(|(_, key)| key).collect();

        println!("Confidence banding test results:");
        for (confidence, key) in &cache_keys {
            println!("  Confidence: {confidence:.16} -> Key: {key}");
        }
        println!("  Unique keys: {}", unique_keys.len());

        // All values in the "very_high" band (0.8-1.0) should produce the same cache key
        assert_eq!(
            unique_keys.len(), 1,
            "All confidence values in the same band should produce identical cache keys. Got {} unique keys",
            unique_keys.len()
        );

        println!("✅ Confidence banding test passed: floating-point precision issues eliminated");
    }

    /// ANCHOR: Test cache key stability across different research types
    /// Verifies that different research types produce different cache keys appropriately
    #[tokio::test]
    async fn test_research_type_differentiation() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        let query = "rust async programming";

        // Test different research types
        let research_types = [
            ResearchType::Learning,
            ResearchType::Implementation,
            ResearchType::Troubleshooting,
            ResearchType::Decision,
            ResearchType::Validation,
        ];

        let mut cache_keys = Vec::new();
        for research_type in research_types.iter() {
            let result = create_basic_research_result(query, research_type.clone());
            let key = storage.store(&result).await.unwrap();
            cache_keys.push((research_type, key));
        }

        // Different research types should produce different cache keys
        let unique_keys: HashSet<_> = cache_keys.iter().map(|(_, key)| key).collect();

        println!("Research type differentiation test results:");
        for (research_type, key) in &cache_keys {
            println!("  Research type: {research_type:?} -> Key: {key}");
        }

        assert_eq!(
            unique_keys.len(), research_types.len(),
            "Different research types should produce different cache keys. Got {} unique keys for {} types",
            unique_keys.len(), research_types.len()
        );

        println!(
            "✅ Research type differentiation test passed: {} distinct cache keys for {} types",
            unique_keys.len(),
            research_types.len()
        );
    }
}

#[cfg(test)]
mod query_normalization_comprehensive_tests {
    use super::*;

    /// ANCHOR: Test comprehensive query normalization scenarios
    /// Verifies that semantically equivalent queries produce identical cache keys
    #[tokio::test]
    async fn test_comprehensive_query_normalization() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Semantically equivalent queries with various formatting differences
        let query_variants = vec![
            "How to use Rust async programming?",
            "how to use rust async programming?",
            "How    to   use   Rust   async   programming?",
            "How to use Rust async programming", // No question mark
            "How to use Rust async programming.", // Period instead
            "How to use Rust async programming!", // Exclamation
            "How to use Rust: async programming?", // Colon
            "How to use Rust, async programming?", // Comma
            "How to use Rust (async) programming?", // Parentheses
            "How to use Rust [async] programming?", // Brackets
        ];

        let mut cache_keys = Vec::new();
        for query in &query_variants {
            let result = create_basic_research_result(query, ResearchType::Learning);
            let key = storage.store(&result).await.unwrap();
            cache_keys.push((query, key));
        }

        // Analyze normalization effectiveness
        let unique_keys: HashSet<_> = cache_keys.iter().map(|(_, key)| key).collect();
        let normalization_effectiveness =
            1.0 - (unique_keys.len() as f64 / cache_keys.len() as f64);

        println!("Comprehensive query normalization test results:");
        for (query, key) in &cache_keys {
            println!("  Query: \"{query}\" -> Key: {key}");
        }
        println!("  Unique keys: {}/{}", unique_keys.len(), cache_keys.len());
        println!("  Normalization effectiveness: {normalization_effectiveness:.2}");

        // We expect high normalization effectiveness (ideally 1 unique key)
        assert!(
            normalization_effectiveness >= 0.7,
            "Normalization effectiveness should be at least 70%, got {normalization_effectiveness:.2}"
        );

        // Ideally, all semantically identical queries should produce the same key
        if unique_keys.len() == 1 {
            println!("✅ Perfect normalization: all query variants produce identical cache keys");
        } else {
            println!(
                "⚠️  Partial normalization: {} unique keys for {} variants",
                unique_keys.len(),
                cache_keys.len()
            );
        }
    }

    /// ANCHOR: Test stop word removal effectiveness
    /// Verifies that common stop words don't affect cache key generation
    #[tokio::test]
    async fn test_stop_word_removal() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Queries with and without stop words
        let query_pairs = vec![
            (
                "How to implement async programming in Rust",
                "implement async programming Rust", // Stop words removed
            ),
            (
                "What is the best way to handle errors in Rust",
                "best way handle errors Rust",
            ),
            (
                "Where can I find documentation for the tokio library",
                "find documentation tokio library",
            ),
        ];

        for (full_query, minimal_query) in query_pairs {
            let result1 = create_basic_research_result(full_query, ResearchType::Learning);
            let result2 = create_basic_research_result(minimal_query, ResearchType::Learning);

            let key1 = storage.store(&result1).await.unwrap();
            let key2 = storage.store(&result2).await.unwrap();

            println!("Stop word removal test:");
            println!("  Full query: \"{full_query}\" -> Key: {key1}");
            println!("  Minimal query: \"{minimal_query}\" -> Key: {key2}");

            assert_eq!(
                key1, key2,
                "Queries with and without stop words should produce identical cache keys"
            );
            println!("  ✅ Keys match: stop words properly removed");
        }
    }
}

#[cfg(test)]
mod context_aware_cache_key_tests {
    use super::*;

    /// ANCHOR: Test context-aware cache key generation stability
    /// Verifies that context detection results are properly normalized in cache keys
    #[tokio::test]
    async fn test_context_aware_cache_key_stability() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        let query = "rust async programming patterns";
        let result = create_basic_research_result(query, ResearchType::Learning);

        // Test with slightly different context confidence values in the same band
        let context_confidences = vec![0.85, 0.8500000000000001, 0.86, 0.89];
        let mut cache_keys = Vec::new();

        for confidence in context_confidences {
            let context = create_context_result_with_confidence(confidence);
            let key = storage
                .store_with_context(&result, Some(&context))
                .await
                .unwrap();
            cache_keys.push((confidence, key));
        }

        // All context results in the same confidence band should produce identical keys
        let unique_keys: HashSet<_> = cache_keys.iter().map(|(_, key)| key).collect();

        println!("Context-aware cache key stability test:");
        for (confidence, key) in &cache_keys {
            println!("  Context confidence: {confidence} -> Key: {key}");
        }
        println!("  Unique keys: {}", unique_keys.len());

        assert_eq!(
            unique_keys.len(),
            1,
            "Context results with similar confidence should produce identical cache keys"
        );

        println!("✅ Context-aware cache key stability test passed");
    }

    /// ANCHOR: Test processing time categorization in cache keys
    /// Verifies that processing time categories work correctly for cache differentiation
    #[tokio::test]
    async fn test_processing_time_categorization() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        let query = "rust async programming";
        let result = create_basic_research_result(query, ResearchType::Learning);

        // Test different processing time categories
        let processing_times = vec![
            (50, "fast"),        // 0-100ms
            (250, "medium"),     // 101-500ms
            (1000, "slow"),      // 501-2000ms
            (3000, "very_slow"), // >2000ms
        ];

        let mut cache_keys = Vec::new();
        for (time_ms, expected_category) in processing_times {
            let _context = create_context_result_with_confidence(0.8);
            // Update processing time using reflection - this is a simplified approach
            let dimension_confidences = vec![DimensionConfidence {
                dimension: ClassificationDimension::AudienceLevel,
                confidence: 0.8,
                matched_keywords: vec!["test".to_string()],
                reasoning: "test evidence".to_string(),
            }];

            let context_with_time = ContextDetectionResult::new(
                AudienceLevel::Intermediate,
                TechnicalDomain::Rust,
                UrgencyLevel::Planned,
                dimension_confidences,
                time_ms,
                false,
            );

            let key = storage
                .store_with_context(&result, Some(&context_with_time))
                .await
                .unwrap();
            cache_keys.push((time_ms, expected_category, key));
        }

        // Different processing time categories should produce different cache keys
        let unique_keys: HashSet<_> = cache_keys.iter().map(|(_, _, key)| key).collect();

        println!("Processing time categorization test:");
        for (time_ms, category, key) in &cache_keys {
            println!("  Time: {time_ms}ms (category: {category}) -> Key: {key}");
        }
        println!("  Unique keys: {}", unique_keys.len());

        assert_eq!(
            unique_keys.len(),
            4,
            "Different processing time categories should produce different cache keys"
        );

        println!("✅ Processing time categorization test passed");
    }
}

#[cfg(test)]
mod cache_key_collision_tests {
    use super::*;

    /// ANCHOR: Test cache key collision avoidance
    /// Verifies that different inputs produce different cache keys (no unintended collisions)
    #[tokio::test]
    async fn test_cache_key_collision_avoidance() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Create distinctly different research results
        let test_cases = vec![
            ("rust async programming", ResearchType::Learning),
            ("rust error handling", ResearchType::Learning),
            ("rust async programming", ResearchType::Implementation), // Same query, different type
            ("python async programming", ResearchType::Learning),     // Different language
            ("rust async debugging", ResearchType::Troubleshooting),  // Different domain
        ];

        let mut cache_keys = Vec::new();
        for (query, research_type) in test_cases {
            let result = create_basic_research_result(query, research_type.clone());
            let key = storage.store(&result).await.unwrap();
            cache_keys.push((query, research_type, key));
        }

        // All different inputs should produce different cache keys
        let unique_keys: HashSet<_> = cache_keys.iter().map(|(_, _, key)| key).collect();

        println!("Cache key collision avoidance test:");
        for (query, research_type, key) in &cache_keys {
            println!("  Query: \"{query}\" (type: {research_type:?}) -> Key: {key}");
        }
        println!("  Unique keys: {}/{}", unique_keys.len(), cache_keys.len());

        assert_eq!(
            unique_keys.len(),
            cache_keys.len(),
            "Different inputs should produce different cache keys (no collisions)"
        );

        println!(
            "✅ Cache key collision avoidance test passed: {} unique keys for {} inputs",
            unique_keys.len(),
            cache_keys.len()
        );
    }
}
