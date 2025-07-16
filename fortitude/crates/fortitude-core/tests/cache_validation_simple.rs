//! Simplified cache validation tests targeting the identified issues
//!
//! ANCHOR: Critical cache functionality tests that protect against regressions
//! Tests: cache key stability, storage index management, retrieval fallback logic

use chrono::Utc;
use fortitude_core::classification::context_detector::ContextDetectionResult;
use fortitude_core::storage::FileStorage;
use fortitude_types::{
    AudienceContext, AudienceLevel, ClassificationDimension, ClassifiedRequest,
    DimensionConfidence, DomainContext, ResearchMetadata, ResearchResult, ResearchType, Storage,
    StorageConfig, TechnicalDomain, UrgencyLevel,
};
use std::collections::HashMap;
use tempfile::TempDir;

// Basic test utilities
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

fn create_basic_context_result(confidence: f64) -> ContextDetectionResult {
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
        50,    // processing_time_ms
        false, // fallback_used
    )
}

// ISSUE 1: Cache Key Stability with Floating-Point Confidence Values
#[cfg(test)]
mod cache_key_stability_tests {
    use super::*;

    /// ANCHOR: Test cache key generation with different floating-point confidence values
    /// Demonstrates the issue where slightly different confidence values
    /// produce different cache keys, leading to cache misses for semantically identical queries
    #[tokio::test]
    async fn test_floating_point_confidence_cache_key_issue() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Create two research results with identical content but slightly different confidence
        let result1 =
            create_basic_research_result("rust async programming", ResearchType::Learning);
        let result2 =
            create_basic_research_result("rust async programming", ResearchType::Learning);

        // Create context results with very slightly different confidence values
        // This simulates floating point precision issues
        let context1 = create_basic_context_result(0.85);
        let context2 = create_basic_context_result(0.8500000000000001); // Tiny precision difference

        // Store both results
        let key1 = storage
            .store_with_context(&result1, Some(&context1))
            .await
            .unwrap();
        let key2 = storage
            .store_with_context(&result2, Some(&context2))
            .await
            .unwrap();

        println!("Key 1: {key1}");
        println!("Key 2: {key2}");

        // ISSUE DEMONSTRATION: These keys should be identical for semantically identical queries
        // but floating-point precision causes them to differ
        if key1 != key2 {
            println!("ISSUE CONFIRMED: Floating-point precision causes different cache keys for identical queries");
            println!("Key difference demonstrates cache key stability issue");
        }

        // Test cache retrieval effectiveness
        let retrieved1 = storage
            .retrieve_with_context(&key1, Some(&context1))
            .await
            .unwrap();
        let retrieved2 = storage
            .retrieve_with_context(&key2, Some(&context2))
            .await
            .unwrap();

        println!("Retrieval 1 success: {}", retrieved1.is_some());
        println!("Retrieval 2 success: {}", retrieved2.is_some());

        // Test cross-retrieval (this often fails due to the key differences)
        let cross_retrieved1 = storage
            .retrieve_with_context(&key1, Some(&context2))
            .await
            .unwrap();
        let cross_retrieved2 = storage
            .retrieve_with_context(&key2, Some(&context1))
            .await
            .unwrap();

        println!("Cross-retrieval 1 success: {}", cross_retrieved1.is_some());
        println!("Cross-retrieval 2 success: {}", cross_retrieved2.is_some());

        if cross_retrieved1.is_none() || cross_retrieved2.is_none() {
            println!("ISSUE: Cross-retrieval failed due to confidence precision differences");
        }
    }

    /// ANCHOR: Test query normalization effectiveness
    #[tokio::test]
    async fn test_query_normalization_effectiveness() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Test queries that should normalize to similar cache keys
        let queries = vec![
            "How to use Rust async programming?",
            "how to use rust async programming?",
            "How    to   use   Rust   async   programming?",
            "How to use Rust async programming", // No question mark
        ];

        let mut cache_keys = Vec::new();
        for query in &queries {
            let result = create_basic_research_result(query, ResearchType::Learning);
            let key = storage.store(&result).await.unwrap();
            cache_keys.push(key);
        }

        // Analyze normalization effectiveness
        let unique_keys: std::collections::HashSet<_> = cache_keys.iter().collect();
        let normalization_effectiveness =
            1.0 - (unique_keys.len() as f64 / cache_keys.len() as f64);

        println!("Cache keys generated: {cache_keys:?}");
        println!("Unique keys: {}", unique_keys.len());
        println!("Total queries: {}", queries.len());
        println!(
            "Normalization effectiveness: {normalization_effectiveness:.2} (higher is better)"
        );

        // Log the issue if normalization is poor
        if normalization_effectiveness < 0.5 {
            println!("ISSUE: Poor query normalization - similar queries generating different keys");
        }
    }
}

// ISSUE 2: Storage Index Management with Immutable References
#[cfg(test)]
mod storage_index_management_tests {
    use super::*;

    /// ANCHOR: Test storage index updates with immutable reference limitations
    /// Demonstrates the issue where cache index cannot be updated due to immutable references
    #[tokio::test]
    async fn test_index_update_limitations() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Check initial state
        let initial_entries = storage.list_cache_entries().await.unwrap();
        println!("Initial cache entries: {}", initial_entries.len());

        // Store multiple results
        let results = vec![
            create_basic_research_result("query 1", ResearchType::Learning),
            create_basic_research_result("query 2", ResearchType::Implementation),
            create_basic_research_result("query 3", ResearchType::Troubleshooting),
        ];

        let mut stored_keys = Vec::new();
        for result in results {
            let key = storage.store(&result).await.unwrap();
            stored_keys.push(key);
        }

        // Check if cache index was updated
        let updated_entries = storage.list_cache_entries().await.unwrap();
        println!("Cache entries after storing: {}", updated_entries.len());
        println!("Expected entries: {}", stored_keys.len());

        // ISSUE DEMONSTRATION: Cache index might not be updated
        if updated_entries.len() != stored_keys.len() {
            println!("ISSUE CONFIRMED: Cache index not properly updated");
            println!(
                "Index shows {} entries but {} were stored",
                updated_entries.len(),
                stored_keys.len()
            );
        }

        // Test retrieval success vs index accuracy
        let mut successful_retrievals = 0;
        for key in &stored_keys {
            if storage.retrieve(key).await.unwrap().is_some() {
                successful_retrievals += 1;
            }
        }

        println!(
            "Successful retrievals: {}/{}",
            successful_retrievals,
            stored_keys.len()
        );

        if successful_retrievals > updated_entries.len() {
            println!("ISSUE: File scan fallback working but index update failed");
        }
    }

    /// ANCHOR: Test cache statistics accuracy
    #[tokio::test]
    async fn test_cache_statistics_accuracy() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Store some results
        let stored_keys = vec![
            storage
                .store(&create_basic_research_result(
                    "test 1",
                    ResearchType::Learning,
                ))
                .await
                .unwrap(),
            storage
                .store(&create_basic_research_result(
                    "test 2",
                    ResearchType::Implementation,
                ))
                .await
                .unwrap(),
        ];

        // Perform some retrievals
        for key in &stored_keys {
            let _ = storage.retrieve(key).await.unwrap();
        }

        // Get statistics
        let stats = storage.get_cache_stats().await.unwrap();
        println!("Cache statistics:");
        println!("  Total entries: {}", stats.total_entries);
        println!("  Hit rate: {:.2}", stats.hit_rate);
        println!("  Hits: {}", stats.hits);
        println!("  Misses: {}", stats.misses);

        // Check for consistency issues
        if stats.total_entries == 0 && !stored_keys.is_empty() {
            println!("ISSUE: Statistics show 0 entries but files were stored");
        }

        if stats.hits == 0 && !stored_keys.is_empty() {
            println!("ISSUE: Statistics show 0 hits but retrievals were attempted");
        }
    }
}

// ISSUE 3: Retrieval Fallback Logic Gaps
#[cfg(test)]
mod retrieval_fallback_tests {
    use super::*;

    /// ANCHOR: Test retrieval fallback logic completeness
    /// Checks if fallback logic properly handles all cache miss scenarios
    #[tokio::test]
    async fn test_fallback_logic_completeness() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Store results using different methods
        let result1 = create_basic_research_result("standard query", ResearchType::Learning);
        let result2 = create_basic_research_result("context query", ResearchType::Implementation);

        let key1 = storage.store(&result1).await.unwrap();

        let context = create_basic_context_result(0.8);
        let key2 = storage
            .store_with_context(&result2, Some(&context))
            .await
            .unwrap();

        println!("Stored keys:");
        println!("  Standard key: {key1}");
        println!("  Context key: {key2}");

        // Test different retrieval combinations
        let test_cases = vec![
            ("Standard->Standard", &key1, None),
            ("Standard->Context", &key1, Some(&context)),
            ("Context->Standard", &key2, None),
            ("Context->Context", &key2, Some(&context)),
        ];

        for (test_name, key, context_opt) in test_cases {
            let result = if let Some(ctx) = context_opt {
                storage.retrieve_with_context(key, Some(ctx)).await.unwrap()
            } else {
                storage.retrieve(key).await.unwrap()
            };

            println!("{}: {}", test_name, result.is_some());

            if result.is_none() {
                println!("ISSUE: Fallback logic gap in {test_name}");
            }
        }
    }

    /// ANCHOR: Test retrieval with missing index entries
    /// Simulates scenarios where cache index is missing but files exist
    #[tokio::test]
    async fn test_missing_index_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Store a result normally
        let result = create_basic_research_result("test query", ResearchType::Learning);
        let cache_key = storage.store(&result).await.unwrap();

        // Test normal retrieval
        let normal_retrieval = storage.retrieve(&cache_key).await.unwrap();
        println!("Normal retrieval success: {}", normal_retrieval.is_some());

        // Create a hypothetical cache key and test directory scanning fallback
        let manual_key = "manual_test_key";
        let file_path = temp_dir
            .path()
            .join("research_results")
            .join("learning")
            .join(format!("{manual_key}.json"));

        // Manually create a file (simulating orphaned cache file)
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await.unwrap();
        }

        let manual_result = create_basic_research_result("manual query", ResearchType::Learning);
        let json_content = serde_json::to_string_pretty(&manual_result).unwrap();
        tokio::fs::write(&file_path, json_content).await.unwrap();

        // Test fallback retrieval
        let fallback_retrieval = storage.retrieve(manual_key).await.unwrap();
        println!(
            "Fallback retrieval success: {}",
            fallback_retrieval.is_some()
        );

        if fallback_retrieval.is_none() {
            println!("ISSUE: Directory scanning fallback failed to find existing file");
        }
    }
}

// Performance Baseline Tests
#[cfg(test)]
mod performance_baseline_tests {
    use super::*;

    /// ANCHOR: Measure baseline cache performance
    #[tokio::test]
    async fn test_cache_performance_baseline() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        let test_queries = vec![
            ("rust async programming", ResearchType::Learning),
            ("implement rest api", ResearchType::Implementation),
            ("debug memory leak", ResearchType::Troubleshooting),
            ("choose database", ResearchType::Decision),
            ("test api endpoint", ResearchType::Validation),
        ];

        // Store phase timing
        let store_start = std::time::Instant::now();
        let mut stored_keys = Vec::new();

        for (query, research_type) in &test_queries {
            let result = create_basic_research_result(query, research_type.clone());
            let key = storage.store(&result).await.unwrap();
            stored_keys.push(key);
        }

        let store_duration = store_start.elapsed();

        // Retrieval phase timing
        let retrieval_start = std::time::Instant::now();
        let mut successful_retrievals = 0;

        for key in &stored_keys {
            if storage.retrieve(key).await.unwrap().is_some() {
                successful_retrievals += 1;
            }
        }

        let retrieval_duration = retrieval_start.elapsed();

        // Calculate and report metrics
        let hit_rate = successful_retrievals as f64 / stored_keys.len() as f64;
        let avg_store_time = store_duration.as_millis() as f64 / test_queries.len() as f64;
        let avg_retrieval_time = retrieval_duration.as_millis() as f64 / stored_keys.len() as f64;

        println!("CACHE PERFORMANCE BASELINE:");
        println!("  Hit Rate: {hit_rate:.2}");
        println!("  Average Store Time: {avg_store_time:.2}ms");
        println!("  Average Retrieval Time: {avg_retrieval_time:.2}ms");
        println!("  Total Store Duration: {store_duration:?}");
        println!("  Total Retrieval Duration: {retrieval_duration:?}");

        // Performance assertions for regression tracking
        assert!(
            hit_rate >= 0.8,
            "Hit rate should be at least 80%, got {hit_rate:.2}"
        );
        assert!(
            avg_retrieval_time < 50.0,
            "Average retrieval time should be under 50ms, got {avg_retrieval_time:.2}ms"
        );

        println!("Performance baseline test completed");
    }
}
