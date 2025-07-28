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

//! Anchor tests for cache key generation stability
//!
//! ANCHOR: Critical cache key generation tests that protect against regressions
//! These tests must never be deleted and should be updated carefully to maintain
//! protection against cache key generation bugs and performance degradation.

use chrono::Utc;
use fortitude_core::classification::context_detector::ContextDetectionResult;
use fortitude_core::storage::FileStorage;
use fortitude_types::{
    AudienceContext, ClassifiedRequest, DomainContext, ResearchMetadata, ResearchResult,
    ResearchType, Storage, StorageConfig,
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

fn create_test_research_result(query: &str, research_type: ResearchType) -> ResearchResult {
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

#[cfg(test)]
mod anchor_tests {
    use super::*;

    /// ANCHOR: Cache key determinism regression protection
    /// This test protects against cache key generation becoming non-deterministic
    /// which would break cache effectiveness and cause performance degradation.
    #[tokio::test]
    async fn anchor_cache_key_determinism() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        let query = "How to implement async programming in Rust?";
        let research_type = ResearchType::Learning;

        // Generate cache keys multiple times - they must be identical
        let mut cache_keys = Vec::new();
        for _ in 0..5 {
            let result = create_test_research_result(query, research_type.clone());
            let key = storage.store(&result).await.unwrap();
            cache_keys.push(key);
        }

        // All keys must be identical for determinism
        let unique_keys: HashSet<_> = cache_keys.iter().collect();
        assert_eq!(
            unique_keys.len(),
            1,
            "REGRESSION: Cache keys are not deterministic. Expected 1 unique key, got {}: {:?}",
            unique_keys.len(),
            cache_keys
        );
    }

    /// ANCHOR: Query normalization effectiveness regression protection
    /// This test protects against query normalization becoming less effective,
    /// which would reduce cache hit rates and hurt performance.
    #[tokio::test]
    async fn anchor_query_normalization_effectiveness() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // These semantically identical queries should produce the same cache key
        let equivalent_queries = vec![
            "How to use Rust async programming?",
            "how to use rust async programming?",
            "How    to   use   Rust   async   programming?",
            "How to use Rust async programming", // No question mark
        ];

        let mut cache_keys = Vec::new();
        for query in &equivalent_queries {
            let result = create_test_research_result(query, ResearchType::Learning);
            let key = storage.store(&result).await.unwrap();
            cache_keys.push(key);
        }

        let unique_keys: HashSet<_> = cache_keys.iter().collect();
        let normalization_effectiveness =
            1.0 - (unique_keys.len() as f64 / cache_keys.len() as f64);

        // Normalization effectiveness must be at least 75%
        assert!(
            normalization_effectiveness >= 0.75,
            "REGRESSION: Query normalization effectiveness dropped below 75%. Got {:.2} with {} unique keys for {} queries",
            normalization_effectiveness, unique_keys.len(), cache_keys.len()
        );
    }

    /// ANCHOR: Cache key collision avoidance regression protection
    /// This test protects against cache key collisions where different queries
    /// produce the same cache key, which would cause incorrect cache hits.
    #[tokio::test]
    async fn anchor_cache_key_collision_avoidance() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // These different queries must produce different cache keys
        let different_queries = vec![
            ("rust async programming", ResearchType::Learning),
            ("rust error handling", ResearchType::Learning),
            ("rust async programming", ResearchType::Implementation), // Same query, different type
            ("python async programming", ResearchType::Learning),     // Different language
        ];

        let mut cache_keys = Vec::new();
        for (query, research_type) in different_queries {
            let result = create_test_research_result(query, research_type);
            let key = storage.store(&result).await.unwrap();
            cache_keys.push(key);
        }

        let unique_keys: HashSet<_> = cache_keys.iter().collect();

        // All different inputs must produce different cache keys (no collisions)
        assert_eq!(
            unique_keys.len(), cache_keys.len(),
            "REGRESSION: Cache key collision detected. Expected {} unique keys, got {} - this will cause incorrect cache hits",
            cache_keys.len(), unique_keys.len()
        );
    }

    /// ANCHOR: Context-aware retrieval fallback regression protection
    /// This test protects against the Context->Standard retrieval fallback breaking,
    /// which would cause cache misses for existing cache entries.
    #[tokio::test]
    async fn anchor_context_standard_retrieval_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        // Store with standard method
        let result1 = create_test_research_result("test query standard", ResearchType::Learning);
        let key1 = storage.store(&result1).await.unwrap();

        // Store with context method
        let result2 =
            create_test_research_result("test query context", ResearchType::Implementation);
        let context = ContextDetectionResult::new(
            fortitude_types::AudienceLevel::Intermediate,
            fortitude_types::TechnicalDomain::Rust,
            fortitude_types::UrgencyLevel::Planned,
            vec![],
            100,
            false,
        );
        let key2 = storage
            .store_with_context(&result2, Some(&context))
            .await
            .unwrap();

        // Test all retrieval combinations - all must succeed
        let test_cases = vec![
            ("Standard->Standard", &key1, None),
            ("Standard->Context", &key1, Some(&context)),
            ("Context->Standard", &key2, None), // This was the bug
            ("Context->Context", &key2, Some(&context)),
        ];

        for (test_name, key, context_opt) in test_cases {
            let result = if let Some(ctx) = context_opt {
                storage.retrieve_with_context(key, Some(ctx)).await.unwrap()
            } else {
                storage.retrieve(key).await.unwrap()
            };

            assert!(
                result.is_some(),
                "REGRESSION: {test_name} retrieval failed - this will cause cache misses for existing entries"
            );
        }
    }

    /// ANCHOR: Cache performance baseline regression protection
    /// This test protects against cache performance degrading significantly.
    #[tokio::test]
    async fn anchor_cache_performance_baseline() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_storage_config(&temp_dir);
        let storage = FileStorage::new(config).await.unwrap();

        let test_queries = vec![
            ("rust async programming", ResearchType::Learning),
            ("implement rest api", ResearchType::Implementation),
            ("debug memory leak", ResearchType::Troubleshooting),
        ];

        // Measure store performance
        let store_start = std::time::Instant::now();
        let mut stored_keys = Vec::new();

        for (query, research_type) in &test_queries {
            let result = create_test_research_result(query, research_type.clone());
            let key = storage.store(&result).await.unwrap();
            stored_keys.push(key);
        }

        let store_duration = store_start.elapsed();

        // Measure retrieval performance
        let retrieval_start = std::time::Instant::now();
        let mut successful_retrievals = 0;

        for key in &stored_keys {
            if storage.retrieve(key).await.unwrap().is_some() {
                successful_retrievals += 1;
            }
        }

        let retrieval_duration = retrieval_start.elapsed();

        // Performance assertions - these thresholds protect against major regressions
        let hit_rate = successful_retrievals as f64 / stored_keys.len() as f64;
        let avg_store_time = store_duration.as_millis() as f64 / test_queries.len() as f64;
        let avg_retrieval_time = retrieval_duration.as_millis() as f64 / stored_keys.len() as f64;

        assert!(
            hit_rate >= 0.9,
            "REGRESSION: Cache hit rate dropped below 90%. Got {hit_rate:.2} - this indicates cache effectiveness problems"
        );

        assert!(
            avg_store_time < 100.0,
            "REGRESSION: Average store time increased above 100ms. Got {avg_store_time:.2}ms - this indicates performance degradation"
        );

        assert!(
            avg_retrieval_time < 100.0,
            "REGRESSION: Average retrieval time increased above 100ms. Got {avg_retrieval_time:.2}ms - this indicates performance degradation"
        );
    }
}
