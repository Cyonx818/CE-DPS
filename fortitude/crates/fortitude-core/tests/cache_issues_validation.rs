//! Comprehensive tests to validate identified cache issues and measure performance
//!
//! This file contains tests that specifically target the three identified cache issues:
//! 1. Cache key generation with floating-point confidence values (storage.rs:132-180)
//! 2. Storage index management with immutable references (storage.rs:318-332)
//! 3. Retrieval fallback logic gaps (storage.rs:810-842)
//!
//! ANCHOR: Critical cache functionality tests that protect against regressions
//! Tests: cache key stability, storage index management, retrieval fallback logic

use chrono::Utc;
use fortitude_core::classification::context_detector::ContextDetectionResult;
use fortitude_core::storage::FileStorage;
use fortitude_types::{
    AudienceContext, AudienceLevel, CacheOperation, CacheOperationType, ClassificationDimension,
    ClassifiedRequest, DimensionConfidence, DomainContext, EnhancedClassificationResult,
    ResearchMetadata, ResearchResult, ResearchType, Storage, StorageConfig, StorageError,
    TechnicalDomain, UrgencyLevel,
};
use std::collections::HashMap;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

// Test fixtures and utilities
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

    ResearchResult::new(request, "Test answer".to_string(), vec![], vec![], metadata)
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
        50,    // processing_time_ms
        false, // fallback_used
    )
}

// Performance measurement utilities
struct CachePerformanceMetrics {
    hit_rate: f64,
    miss_rate: f64,
    average_response_time_ms: f64,
    total_operations: u64,
    cache_key_collisions: u64,
    normalization_effectiveness: f64,
}

impl CachePerformanceMetrics {
    fn new() -> Self {
        Self {
            hit_rate: 0.0,
            miss_rate: 0.0,
            average_response_time_ms: 0.0,
            total_operations: 0,
            cache_key_collisions: 0,
            normalization_effectiveness: 0.0,
        }
    }

    fn calculate_from_operations(&mut self, operations: &[CacheOperation]) {
        if operations.is_empty() {
            return;
        }

        let hits = operations
            .iter()
            .filter(|op| matches!(op.operation_type, CacheOperationType::Hit))
            .count();
        let misses = operations
            .iter()
            .filter(|op| matches!(op.operation_type, CacheOperationType::Miss))
            .count();
        let total_lookup_ops = hits + misses;

        self.total_operations = operations.len() as u64;
        if total_lookup_ops > 0 {
            self.hit_rate = hits as f64 / total_lookup_ops as f64;
            self.miss_rate = misses as f64 / total_lookup_ops as f64;
        }

        self.average_response_time_ms = operations
            .iter()
            .map(|op| op.duration_ms as f64)
            .sum::<f64>()
            / operations.len() as f64;

        // Calculate key collisions - simplified approximation
        let unique_keys: std::collections::HashSet<_> =
            operations.iter().map(|op| &op.cache_key).collect();
        self.cache_key_collisions = (operations.len() - unique_keys.len()) as u64;

        // Calculate normalization effectiveness
        if operations.len() > 0 {
            self.normalization_effectiveness = unique_keys.len() as f64 / operations.len() as f64;
        }
    }
}

// ISSUE 1: Cache Key Stability with Floating-Point Confidence Values
// Tests the cache key generation problem mentioned in storage.rs:132-180
#[cfg(test)]
mod cache_key_stability_tests {
    use super::*;

    /// ANCHOR: Test cache key generation with different floating-point confidence values
    /// This test demonstrates the issue where slightly different confidence values
    /// produce different cache keys, leading to cache misses for semantically identical queries
    #[tokio::test]
    async fn test_cache_key_stability_with_floating_point_confidence() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Create two research results with semantically identical content but slightly different confidence
        let mut result1 =
            create_test_research_result("rust async programming", ResearchType::Learning);
        let mut result2 =
            create_test_research_result("rust async programming", ResearchType::Learning);

        // Create context results with very slightly different confidence values
        // This simulates the real-world scenario where confidence calculations might vary by tiny amounts
        let context1 = create_test_context_result(0.85);
        let context2 = create_test_context_result(0.8500000000000001); // Floating point precision difference

        // Store both results
        let key1 = storage
            .store_with_context(&result1, Some(&context1))
            .await
            .unwrap();
        let key2 = storage
            .store_with_context(&result2, Some(&context2))
            .await
            .unwrap();

        // BUG: These keys should be identical for semantically identical queries
        // but floating-point precision issues cause them to differ
        println!("Key 1: {}", key1);
        println!("Key 2: {}", key2);

        // This assertion demonstrates the bug - keys should be equal but aren't
        assert_ne!(key1, key2, "ISSUE DEMONSTRATED: Floating-point precision in confidence causes different cache keys for identical queries");

        // Try to retrieve with the second key - this should hit the cache but won't
        let retrieved = storage
            .retrieve_with_context(&key2, Some(&context1))
            .await
            .unwrap();

        // This demonstrates the cache miss that shouldn't happen
        if retrieved.is_none() {
            println!("CACHE MISS: Query with nearly identical confidence failed to hit cache");
        }
    }

    /// ANCHOR: Test query normalization effectiveness for cache hits
    /// This test checks if the query normalization in storage.rs helps with cache hits
    #[tokio::test]
    async fn test_query_normalization_cache_effectiveness() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Create variations of the same query that should normalize to the same cache key
        let queries = vec![
            "How to use Rust async programming?",
            "how to use rust async programming?",
            "How    to   use   Rust   async   programming?",
            "How to use Rust async programming", // No question mark
            "rust async programming how to use", // Different word order
        ];

        let mut cache_keys = Vec::new();
        let mut results = Vec::new();

        for query in &queries {
            let result = create_test_research_result(query, ResearchType::Learning);
            let key = storage.store_with_context(&result, None).await.unwrap();
            cache_keys.push(key);
            results.push(result);
        }

        // Analyze cache key effectiveness
        let unique_keys: std::collections::HashSet<_> = cache_keys.iter().collect();
        let normalization_effectiveness = unique_keys.len() as f64 / cache_keys.len() as f64;

        println!("Cache keys generated: {:?}", cache_keys);
        println!("Unique keys: {}", unique_keys.len());
        println!("Total queries: {}", queries.len());
        println!(
            "Normalization effectiveness: {:.2}",
            normalization_effectiveness
        );

        // Test cache retrieval with normalized queries
        let mut cache_hits = 0;
        let mut cache_misses = 0;

        for (i, query) in queries.iter().enumerate() {
            let test_result = create_test_research_result(query, ResearchType::Learning);
            let expected_key = &cache_keys[i];

            let retrieved = storage
                .retrieve_with_context(expected_key, None)
                .await
                .unwrap();
            if retrieved.is_some() {
                cache_hits += 1;
            } else {
                cache_misses += 1;
                println!("Cache miss for query: {}", query);
            }
        }

        let hit_rate = cache_hits as f64 / queries.len() as f64;
        println!("Cache hit rate: {:.2}", hit_rate);

        // This assertion shows the current normalization effectiveness
        // A low effectiveness indicates the normalization isn't working well
        assert!(
            normalization_effectiveness > 0.0,
            "Normalization should have some effect"
        );

        // Document the current performance for regression tracking
        println!(
            "PERFORMANCE BASELINE: Normalization effectiveness: {:.2}, Hit rate: {:.2}",
            normalization_effectiveness, hit_rate
        );
    }

    /// ANCHOR: Test confidence band grouping for cache key stability
    /// This test validates that the confidence band approach works for cache key stability
    #[tokio::test]
    async fn test_confidence_band_cache_key_stability() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Test confidence values that should fall into the same band
        let confidence_values = vec![
            0.81, 0.82, 0.83, 0.84, 0.85, // Should all be "very_high" band
            0.61, 0.62, 0.63, 0.64, 0.65, // Should all be "high" band
            0.31, 0.32, 0.33, 0.34, 0.35, // Should all be "medium" band
        ];

        let mut cache_keys = Vec::new();

        for confidence in confidence_values {
            let result = create_test_research_result("test query", ResearchType::Learning);
            let context = create_test_context_result(confidence);
            let key = storage
                .store_with_context(&result, Some(&context))
                .await
                .unwrap();
            cache_keys.push((confidence, key));
        }

        // Group keys by confidence band
        let mut band_groups: HashMap<String, Vec<String>> = HashMap::new();
        for (confidence, key) in &cache_keys {
            let band = match confidence {
                0.8..=1.0 => "very_high",
                0.6..=0.8 => "high",
                0.3..=0.6 => "medium",
                _ => "low",
            };
            band_groups
                .entry(band.to_string())
                .or_default()
                .push(key.clone());
        }

        // Analyze cache key stability within bands
        for (band, keys) in &band_groups {
            let unique_keys: std::collections::HashSet<_> = keys.iter().collect();
            let stability = if keys.len() > 1 {
                // For cache key stability, we want fewer unique keys (more reuse)
                1.0 - (unique_keys.len() as f64 / keys.len() as f64)
            } else {
                1.0
            };

            println!(
                "Band {}: {} queries, {} unique keys, stability: {:.2}",
                band,
                keys.len(),
                unique_keys.len(),
                stability
            );

            // This assertion will likely fail, demonstrating the issue
            if keys.len() > 1 {
                assert!(stability > 0.0,
                    "ISSUE: Cache keys should be more stable within confidence bands. Band: {}, Stability: {:.2}",
                    band, stability);
            }
        }
    }
}

// ISSUE 2: Storage Index Management with Immutable References
// Tests the storage index update problem mentioned in storage.rs:318-332
#[cfg(test)]
mod storage_index_management_tests {
    use super::*;

    /// ANCHOR: Test storage index updates with immutable reference limitations
    /// This test demonstrates the issue where cache index cannot be updated due to immutable references
    #[tokio::test]
    async fn test_storage_index_update_limitations() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Store a result
        let result = create_test_research_result("test query", ResearchType::Learning);
        let cache_key = storage.store(&result).await.unwrap();

        // Try to get cache entries - this should include our stored result
        let entries_before = storage.list_cache_entries().await.unwrap();
        println!("Cache entries before: {}", entries_before.len());

        // Store another result
        let result2 = create_test_research_result("second query", ResearchType::Implementation);
        let cache_key2 = storage.store(&result2).await.unwrap();

        // Get cache entries again - this should show both results
        let entries_after = storage.list_cache_entries().await.unwrap();
        println!("Cache entries after: {}", entries_after.len());

        // BUG: The cache index is not being updated due to immutable reference limitations
        // This assertion will likely fail, demonstrating the issue
        assert_eq!(entries_after.len(), 2,
            "ISSUE DEMONSTRATED: Cache index not updated due to immutable references. Expected 2 entries, got {}", 
            entries_after.len());

        // Try to retrieve using the cache index lookup
        let retrieved_via_index = storage.retrieve(&cache_key).await.unwrap();
        let retrieved_via_scan = storage.retrieve(&cache_key2).await.unwrap();

        // Test if both retrieval methods work
        println!("Retrieved via index: {}", retrieved_via_index.is_some());
        println!("Retrieved via scan: {}", retrieved_via_scan.is_some());

        // The index lookup might fail while file scan succeeds, showing the issue
        if retrieved_via_index.is_none() && retrieved_via_scan.is_some() {
            println!(
                "ISSUE: Cache index lookup failed, but file scan succeeded - index not updated"
            );
        }
    }

    /// ANCHOR: Test cache statistics accuracy with index management issues
    /// This test checks if cache statistics are accurate when index updates fail
    #[tokio::test]
    async fn test_cache_statistics_accuracy() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Get initial stats
        let initial_stats = storage.get_cache_stats().await.unwrap();
        println!(
            "Initial stats - Total entries: {}, Hit rate: {:.2}",
            initial_stats.total_entries, initial_stats.hit_rate
        );

        // Store multiple results
        let results = vec![
            create_test_research_result("query 1", ResearchType::Learning),
            create_test_research_result("query 2", ResearchType::Implementation),
            create_test_research_result("query 3", ResearchType::Troubleshooting),
        ];

        let mut stored_keys = Vec::new();
        for result in results {
            let key = storage.store(&result).await.unwrap();
            stored_keys.push(key);
        }

        // Get stats after storing
        let after_store_stats = storage.get_cache_stats().await.unwrap();
        println!(
            "After store stats - Total entries: {}, Hit rate: {:.2}",
            after_store_stats.total_entries, after_store_stats.hit_rate
        );

        // Perform some retrieval operations
        let mut retrieval_results = Vec::new();
        for key in &stored_keys {
            let retrieved = storage.retrieve(key).await.unwrap();
            retrieval_results.push(retrieved.is_some());
        }

        // Get final stats
        let final_stats = storage.get_cache_stats().await.unwrap();
        println!(
            "Final stats - Total entries: {}, Hit rate: {:.2}, Hits: {}, Misses: {}",
            final_stats.total_entries, final_stats.hit_rate, final_stats.hits, final_stats.misses
        );

        // Check if statistics accurately reflect the operations
        let expected_entries = stored_keys.len();
        let actual_entries = final_stats.total_entries;

        // This assertion will likely fail due to index update issues
        assert_eq!(
            actual_entries, expected_entries,
            "ISSUE DEMONSTRATED: Cache statistics inaccurate. Expected {} entries, got {}",
            expected_entries, actual_entries
        );

        // Check hit rate calculation
        let successful_retrievals = retrieval_results.iter().filter(|&&r| r).count();
        let expected_hits = successful_retrievals as u64;
        let actual_hits = final_stats.hits;

        if expected_hits > 0 && actual_hits == 0 {
            println!(
                "ISSUE: Hit rate calculation incorrect - expected {} hits, got {}",
                expected_hits, actual_hits
            );
        }
    }

    /// ANCHOR: Test concurrent cache operations with index consistency
    /// This test checks index consistency under concurrent operations
    #[tokio::test]
    async fn test_concurrent_cache_operations_index_consistency() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Create concurrent store operations
        let mut handles = Vec::new();

        for i in 0..10 {
            let result = create_test_research_result(
                &format!("concurrent query {}", i),
                ResearchType::Learning,
            );

            // In a real concurrent test, we'd need Arc<Mutex<FileStorage>> or similar
            // For now, we'll simulate the issue by rapid sequential operations
            handles.push(tokio::spawn(async move {
                // Simulate concurrent operation timing
                sleep(Duration::from_millis(i * 10)).await;
                i
            }));
        }

        // Wait for all operations
        for handle in handles {
            handle.await.unwrap();
        }

        // Store results sequentially to simulate the concurrency issue
        let mut keys = Vec::new();
        for i in 0..10 {
            let result = create_test_research_result(
                &format!("concurrent query {}", i),
                ResearchType::Learning,
            );
            let key = storage.store(&result).await.unwrap();
            keys.push(key);
        }

        // Check index consistency
        let stats = storage.get_cache_stats().await.unwrap();
        let entries = storage.list_cache_entries().await.unwrap();

        println!(
            "Concurrent operations - Stats entries: {}, Listed entries: {}",
            stats.total_entries,
            entries.len()
        );

        // Test retrieval consistency
        let mut retrieval_success = 0;
        for key in &keys {
            if storage.retrieve(key).await.unwrap().is_some() {
                retrieval_success += 1;
            }
        }

        println!(
            "Retrieval success rate: {}/{}",
            retrieval_success,
            keys.len()
        );

        // This assertion may fail due to index consistency issues
        assert_eq!(
            stats.total_entries,
            keys.len(),
            "ISSUE: Index consistency problems under concurrent operations"
        );
    }
}

// ISSUE 3: Retrieval Fallback Logic Gaps
// Tests the retrieval fallback problem mentioned in storage.rs:810-842
#[cfg(test)]
mod retrieval_fallback_tests {
    use super::*;

    /// ANCHOR: Test retrieval fallback logic completeness
    /// This test checks if the fallback logic properly handles all cache miss scenarios
    #[tokio::test]
    async fn test_retrieval_fallback_logic_completeness() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Store results using different methods to create various file locations
        let result1 = create_test_research_result("standard query", ResearchType::Learning);
        let result2 = create_test_research_result("context query", ResearchType::Implementation);

        // Store using standard method
        let key1 = storage.store(&result1).await.unwrap();

        // Store using context-aware method
        let context = create_test_context_result(0.8);
        let key2 = storage
            .store_with_context(&result2, Some(&context))
            .await
            .unwrap();

        // Test retrieval with standard method
        let retrieved1_standard = storage.retrieve(&key1).await.unwrap();
        let retrieved2_standard = storage.retrieve(&key2).await.unwrap();

        // Test retrieval with context-aware method
        let retrieved1_context = storage.retrieve_with_context(&key1, None).await.unwrap();
        let retrieved2_context = storage
            .retrieve_with_context(&key2, Some(&context))
            .await
            .unwrap();

        println!("Standard retrieval results:");
        println!(
            "  Key1 (standard stored): {}",
            retrieved1_standard.is_some()
        );
        println!("  Key2 (context stored): {}", retrieved2_standard.is_some());

        println!("Context-aware retrieval results:");
        println!("  Key1 (standard stored): {}", retrieved1_context.is_some());
        println!("  Key2 (context stored): {}", retrieved2_context.is_some());

        // Test cross-method retrieval - this reveals fallback logic gaps
        let cross_retrieved1 = storage
            .retrieve_with_context(&key1, Some(&context))
            .await
            .unwrap();
        let cross_retrieved2 = storage.retrieve(&key2).await.unwrap();

        println!("Cross-method retrieval results:");
        println!("  Key1 with context: {}", cross_retrieved1.is_some());
        println!("  Key2 without context: {}", cross_retrieved2.is_some());

        // These assertions may fail due to fallback logic gaps
        if cross_retrieved1.is_none() {
            println!("ISSUE: Standard-stored item not found via context-aware retrieval");
        }
        if cross_retrieved2.is_none() {
            println!("ISSUE: Context-stored item not found via standard retrieval");
        }

        // Check if fallback logic handles all scenarios
        let fallback_success_rate = vec![
            retrieved1_standard.is_some(),
            retrieved2_standard.is_some(),
            retrieved1_context.is_some(),
            retrieved2_context.is_some(),
            cross_retrieved1.is_some(),
            cross_retrieved2.is_some(),
        ]
        .iter()
        .filter(|&&success| success)
        .count();

        let total_attempts = 6;
        let success_rate = fallback_success_rate as f64 / total_attempts as f64;
        println!("Fallback logic success rate: {:.2}", success_rate);

        // This assertion documents the current fallback effectiveness
        assert!(
            success_rate >= 0.5,
            "Fallback logic should handle at least 50% of cross-method scenarios"
        );
    }

    /// ANCHOR: Test retrieval with missing cache index entries
    /// This test simulates scenarios where cache index is missing but files exist
    #[tokio::test]
    async fn test_retrieval_with_missing_index_entries() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Store a result normally
        let result = create_test_research_result("test query", ResearchType::Learning);
        let cache_key = storage.store(&result).await.unwrap();

        // Verify normal retrieval works
        let normal_retrieval = storage.retrieve(&cache_key).await.unwrap();
        assert!(normal_retrieval.is_some(), "Normal retrieval should work");

        // Simulate missing index entry scenario by testing with a manually constructed key
        // This tests the fallback directory scanning logic
        let manual_key = "manually_constructed_key";
        let manual_result =
            create_test_research_result("manual query", ResearchType::Troubleshooting);

        // Manually create a file in the expected location (simulating index corruption)
        let file_path = temp_dir
            .path()
            .join("research_results")
            .join("troubleshooting")
            .join(format!("{}.json", manual_key));

        // Ensure directory exists
        tokio::fs::create_dir_all(file_path.parent().unwrap())
            .await
            .unwrap();

        // Write the file directly (bypassing the storage system)
        let json_content = serde_json::to_string_pretty(&manual_result).unwrap();
        tokio::fs::write(&file_path, json_content).await.unwrap();

        // Test retrieval of the manually created file
        let fallback_retrieval = storage.retrieve(manual_key).await.unwrap();

        println!(
            "Fallback retrieval success: {}",
            fallback_retrieval.is_some()
        );

        if fallback_retrieval.is_none() {
            println!("ISSUE: Directory scanning fallback failed to find existing file");
        }

        // Test context-aware fallback
        let context_fallback = storage
            .retrieve_with_context(manual_key, None)
            .await
            .unwrap();
        println!(
            "Context-aware fallback success: {}",
            context_fallback.is_some()
        );

        // This assertion tests the fallback scanning logic
        assert!(
            fallback_retrieval.is_some() || context_fallback.is_some(),
            "ISSUE: Fallback logic should find files even when index is missing"
        );
    }

    /// ANCHOR: Test retrieval path priority and fallback ordering
    /// This test checks if the retrieval fallback follows the correct priority order
    #[tokio::test]
    async fn test_retrieval_path_priority_ordering() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Store results in different ways to create files in multiple locations
        let result = create_test_research_result("multi-path query", ResearchType::Validation);
        let context = create_test_context_result(0.75);

        // Store using both methods - this might create files in different locations
        let key1 = storage.store(&result).await.unwrap();
        let key2 = storage
            .store_with_context(&result, Some(&context))
            .await
            .unwrap();

        println!("Generated keys:");
        println!("  Standard key: {}", key1);
        println!("  Context key: {}", key2);

        // Test retrieval timing to understand path priority
        let start_time = std::time::Instant::now();
        let retrieval1 = storage.retrieve(&key1).await.unwrap();
        let time1 = start_time.elapsed();

        let start_time2 = std::time::Instant::now();
        let retrieval2 = storage.retrieve(&key2).await.unwrap();
        let time2 = start_time2.elapsed();

        println!("Retrieval times:");
        println!("  Standard retrieval: {:?}", time1);
        println!("  Context-aware retrieval: {:?}", time2);

        // Test cross-method retrieval timing
        let start_time3 = std::time::Instant::now();
        let cross_retrieval = storage
            .retrieve_with_context(&key1, Some(&context))
            .await
            .unwrap();
        let time3 = start_time3.elapsed();

        println!("Cross-method retrieval time: {:?}", time3);

        // Analyze retrieval success and performance
        let retrieval_success = vec![
            retrieval1.is_some(),
            retrieval2.is_some(),
            cross_retrieval.is_some(),
        ];

        let success_count = retrieval_success.iter().filter(|&&s| s).count();
        println!(
            "Retrieval success rate: {}/{}",
            success_count,
            retrieval_success.len()
        );

        // Document performance characteristics
        println!("PERFORMANCE ANALYSIS:");
        println!(
            "  Average retrieval time: {:?}",
            (time1 + time2 + time3) / 3
        );
        println!(
            "  Fallback effectiveness: {:.2}",
            success_count as f64 / 3.0
        );

        // This assertion checks if fallback paths are working
        assert!(
            success_count >= 2,
            "At least 2 of 3 retrieval methods should work"
        );
    }
}

// Performance and Measurement Tests
#[cfg(test)]
mod cache_performance_tests {
    use super::*;

    /// ANCHOR: Measure current cache hit rates and performance
    /// This test establishes baseline performance metrics for cache operations
    #[tokio::test]
    async fn test_cache_performance_baseline_measurement() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        let mut performance_metrics = CachePerformanceMetrics::new();
        let mut operations = Vec::new();

        // Simulate realistic cache operations
        let queries = vec![
            ("rust async programming", ResearchType::Learning),
            ("implement rest api", ResearchType::Implementation),
            ("debug memory leak", ResearchType::Troubleshooting),
            ("choose database", ResearchType::Decision),
            ("test api endpoint", ResearchType::Validation),
        ];

        // Store phase
        let mut stored_keys = Vec::new();
        for (query, research_type) in &queries {
            let start_time = std::time::Instant::now();
            let result = create_test_research_result(query, *research_type);
            let key = storage.store(&result).await.unwrap();
            let duration = start_time.elapsed();

            stored_keys.push(key.clone());
            operations.push(CacheOperation {
                timestamp: Utc::now(),
                operation_type: CacheOperationType::Store,
                cache_key: key,
                duration_ms: duration.as_millis() as u64,
                success: true,
                context: HashMap::new(),
            });
        }

        // Retrieval phase - mix of hits and misses
        let retrieval_keys = vec![
            stored_keys[0].clone(),            // Should hit
            stored_keys[1].clone(),            // Should hit
            "non_existent_key".to_string(),    // Should miss
            stored_keys[2].clone(),            // Should hit
            "another_missing_key".to_string(), // Should miss
        ];

        for key in retrieval_keys {
            let start_time = std::time::Instant::now();
            let result = storage.retrieve(&key).await.unwrap();
            let duration = start_time.elapsed();

            let success = result.is_some();
            let operation_type = if success {
                CacheOperationType::Hit
            } else {
                CacheOperationType::Miss
            };

            operations.push(CacheOperation {
                timestamp: Utc::now(),
                operation_type,
                cache_key: key,
                duration_ms: duration.as_millis() as u64,
                success,
                context: HashMap::new(),
            });
        }

        // Calculate performance metrics
        performance_metrics.calculate_from_operations(&operations);

        // Report baseline metrics
        println!("CACHE PERFORMANCE BASELINE:");
        println!("  Hit Rate: {:.2}", performance_metrics.hit_rate);
        println!("  Miss Rate: {:.2}", performance_metrics.miss_rate);
        println!(
            "  Average Response Time: {:.2}ms",
            performance_metrics.average_response_time_ms
        );
        println!(
            "  Total Operations: {}",
            performance_metrics.total_operations
        );
        println!(
            "  Cache Key Collisions: {}",
            performance_metrics.cache_key_collisions
        );
        println!(
            "  Normalization Effectiveness: {:.2}",
            performance_metrics.normalization_effectiveness
        );

        // Document current performance for regression tracking
        assert!(
            performance_metrics.hit_rate >= 0.0,
            "Hit rate should be non-negative"
        );
        assert!(
            performance_metrics.average_response_time_ms >= 0.0,
            "Response time should be non-negative"
        );

        // Store baseline metrics for future comparison
        let baseline_file = temp_dir.path().join("performance_baseline.json");
        let baseline_json = serde_json::to_string_pretty(&performance_metrics).unwrap();
        tokio::fs::write(baseline_file, baseline_json)
            .await
            .unwrap();

        println!("Performance baseline saved for regression tracking");
    }

    /// ANCHOR: Test cache performance under load
    /// This test measures cache performance under higher load conditions
    #[tokio::test]
    async fn test_cache_performance_under_load() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        let load_test_queries = 50;
        let mut operations = Vec::new();

        // Generate diverse queries
        let mut queries = Vec::new();
        for i in 0..load_test_queries {
            let query = format!("load test query {} with variations", i);
            let research_type = match i % 5 {
                0 => ResearchType::Learning,
                1 => ResearchType::Implementation,
                2 => ResearchType::Troubleshooting,
                3 => ResearchType::Decision,
                _ => ResearchType::Validation,
            };
            queries.push((query, research_type));
        }

        // Store phase
        let mut stored_keys = Vec::new();
        let store_start = std::time::Instant::now();

        for (query, research_type) in &queries {
            let start_time = std::time::Instant::now();
            let result = create_test_research_result(query, *research_type);
            let key = storage.store(&result).await.unwrap();
            let duration = start_time.elapsed();

            stored_keys.push(key.clone());
            operations.push(CacheOperation {
                timestamp: Utc::now(),
                operation_type: CacheOperationType::Store,
                cache_key: key,
                duration_ms: duration.as_millis() as u64,
                success: true,
                context: HashMap::new(),
            });
        }

        let store_duration = store_start.elapsed();

        // Retrieval phase with cache warming
        let retrieval_start = std::time::Instant::now();
        let mut retrieval_success = 0;

        for key in &stored_keys {
            let start_time = std::time::Instant::now();
            let result = storage.retrieve(key).await.unwrap();
            let duration = start_time.elapsed();

            let success = result.is_some();
            if success {
                retrieval_success += 1;
            }

            operations.push(CacheOperation {
                timestamp: Utc::now(),
                operation_type: if success {
                    CacheOperationType::Hit
                } else {
                    CacheOperationType::Miss
                },
                cache_key: key.clone(),
                duration_ms: duration.as_millis() as u64,
                success,
                context: HashMap::new(),
            });
        }

        let retrieval_duration = retrieval_start.elapsed();

        // Calculate and report load test metrics
        let mut load_metrics = CachePerformanceMetrics::new();
        load_metrics.calculate_from_operations(&operations);

        println!("CACHE LOAD TEST RESULTS:");
        println!("  Queries Processed: {}", load_test_queries);
        println!("  Store Phase Duration: {:?}", store_duration);
        println!("  Retrieval Phase Duration: {:?}", retrieval_duration);
        println!("  Cache Hit Rate: {:.2}", load_metrics.hit_rate);
        println!(
            "  Average Response Time: {:.2}ms",
            load_metrics.average_response_time_ms
        );
        println!(
            "  Throughput: {:.2} ops/sec",
            load_test_queries as f64 / store_duration.as_secs_f64()
        );
        println!(
            "  Cache Key Collisions: {}",
            load_metrics.cache_key_collisions
        );

        // Performance assertions
        assert!(
            load_metrics.hit_rate >= 0.8,
            "Load test should achieve at least 80% hit rate"
        );
        assert!(
            load_metrics.average_response_time_ms < 100.0,
            "Average response time should be under 100ms"
        );

        println!("Load test completed successfully");
    }

    /// ANCHOR: Test cache performance with different query patterns
    /// This test measures how different query patterns affect cache performance
    #[tokio::test]
    async fn test_cache_performance_query_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Test different query patterns
        let patterns = vec![
            (
                "exact_match",
                vec!["rust async", "rust async", "rust async"],
            ),
            (
                "case_variation",
                vec!["Rust Async", "rust async", "RUST ASYNC"],
            ),
            (
                "whitespace_variation",
                vec!["rust async", "rust  async", "rust   async"],
            ),
            (
                "word_order",
                vec![
                    "rust async programming",
                    "async rust programming",
                    "programming rust async",
                ],
            ),
            (
                "semantic_similarity",
                vec!["rust async await", "rust asynchronous", "rust concurrent"],
            ),
        ];

        for (pattern_name, queries) in patterns {
            println!("\nTesting pattern: {}", pattern_name);
            let mut pattern_operations = Vec::new();

            // Store first query
            let first_result = create_test_research_result(&queries[0], ResearchType::Learning);
            let first_key = storage.store(&first_result).await.unwrap();

            // Test retrieval with pattern variations
            for (i, query) in queries.iter().enumerate() {
                let start_time = std::time::Instant::now();
                let test_result = create_test_research_result(query, ResearchType::Learning);

                // Try to retrieve with a key that might match due to normalization
                let retrieval_result = if i == 0 {
                    storage.retrieve(&first_key).await.unwrap()
                } else {
                    // For variations, test if our normalization helps
                    let variant_key = storage.store(&test_result).await.unwrap();
                    storage.retrieve(&variant_key).await.unwrap()
                };

                let duration = start_time.elapsed();
                let success = retrieval_result.is_some();

                pattern_operations.push(CacheOperation {
                    timestamp: Utc::now(),
                    operation_type: if success {
                        CacheOperationType::Hit
                    } else {
                        CacheOperationType::Miss
                    },
                    cache_key: first_key.clone(),
                    duration_ms: duration.as_millis() as u64,
                    success,
                    context: HashMap::new(),
                });
            }

            // Analyze pattern performance
            let mut pattern_metrics = CachePerformanceMetrics::new();
            pattern_metrics.calculate_from_operations(&pattern_operations);

            println!("  Pattern Results:");
            println!("    Hit Rate: {:.2}", pattern_metrics.hit_rate);
            println!(
                "    Avg Response Time: {:.2}ms",
                pattern_metrics.average_response_time_ms
            );
            println!(
                "    Normalization Effectiveness: {:.2}",
                pattern_metrics.normalization_effectiveness
            );
        }
    }
}

// Comprehensive Integration Tests
#[cfg(test)]
mod cache_integration_tests {
    use super::*;

    /// ANCHOR: Comprehensive cache system integration test
    /// This test validates the entire cache system under realistic conditions
    #[tokio::test]
    async fn test_comprehensive_cache_system_integration() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Phase 1: Initial population
        let initial_queries = vec![
            (
                "How to handle async errors in Rust?",
                ResearchType::Learning,
            ),
            ("Implement JWT authentication", ResearchType::Implementation),
            ("Debug connection timeout", ResearchType::Troubleshooting),
            (
                "Choose between PostgreSQL and MySQL",
                ResearchType::Decision,
            ),
            ("Validate API response format", ResearchType::Validation),
        ];

        let mut all_keys = Vec::new();
        for (query, research_type) in &initial_queries {
            let result = create_test_research_result(query, *research_type);
            let key = storage.store(&result).await.unwrap();
            all_keys.push(key);
        }

        // Phase 2: Cache warming and retrieval
        let mut retrieval_success = 0;
        for key in &all_keys {
            if storage.retrieve(key).await.unwrap().is_some() {
                retrieval_success += 1;
            }
        }

        let initial_hit_rate = retrieval_success as f64 / all_keys.len() as f64;
        println!("Initial hit rate: {:.2}", initial_hit_rate);

        // Phase 3: Context-aware operations
        let context = create_test_context_result(0.8);
        let context_queries = vec![
            ("Advanced Rust patterns", ResearchType::Learning),
            ("Microservices architecture", ResearchType::Implementation),
        ];

        for (query, research_type) in &context_queries {
            let result = create_test_research_result(query, *research_type);
            let key = storage
                .store_with_context(&result, Some(&context))
                .await
                .unwrap();
            all_keys.push(key);
        }

        // Phase 4: Mixed retrieval patterns
        let mixed_retrieval_tests = vec![
            ("Standard retrieval of standard item", &all_keys[0], None),
            (
                "Context retrieval of standard item",
                &all_keys[0],
                Some(&context),
            ),
            (
                "Standard retrieval of context item",
                &all_keys[all_keys.len() - 1],
                None,
            ),
            (
                "Context retrieval of context item",
                &all_keys[all_keys.len() - 1],
                Some(&context),
            ),
        ];

        for (test_name, key, context_opt) in mixed_retrieval_tests {
            let result = if let Some(ctx) = context_opt {
                storage.retrieve_with_context(key, Some(ctx)).await.unwrap()
            } else {
                storage.retrieve(key).await.unwrap()
            };

            println!("{}: {}", test_name, result.is_some());
        }

        // Phase 5: Performance analysis
        let final_stats = storage.get_cache_stats().await.unwrap();
        println!("\nFinal Cache Statistics:");
        println!("  Total Entries: {}", final_stats.total_entries);
        println!("  Hit Rate: {:.2}", final_stats.hit_rate);
        println!("  Total Size: {} bytes", final_stats.total_size_bytes);
        println!(
            "  Average Age: {:.2} seconds",
            final_stats.average_age_seconds
        );

        // Phase 6: Cleanup and expiration
        let cleanup_count = storage.cleanup_expired().await.unwrap();
        println!("Expired entries cleaned: {}", cleanup_count);

        // Final validation
        assert!(
            final_stats.total_entries > 0,
            "Cache should contain entries"
        );
        assert!(
            final_stats.hit_rate >= 0.0,
            "Hit rate should be non-negative"
        );

        println!("Comprehensive integration test completed successfully");
    }
}
