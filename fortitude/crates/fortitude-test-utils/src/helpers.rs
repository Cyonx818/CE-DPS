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

// ABOUTME: Test helpers for the Fortitude research system
use fortitude_types::*;
use std::collections::HashMap;
use tempfile::TempDir;
use tokio::fs;
use uuid::Uuid;

/// Create a temporary directory for testing
pub fn create_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temporary directory")
}

/// Setup a temporary storage directory with sample data
pub async fn setup_temp_storage(temp_dir: &TempDir) -> StorageConfig {
    let base_path = temp_dir.path().to_path_buf();

    // Create required directories
    let dirs = [
        "research_results",
        "cache",
        "index",
        "research_results/decision",
        "research_results/implementation",
        "research_results/troubleshooting",
        "research_results/learning",
        "research_results/validation",
    ];

    for dir in dirs {
        fs::create_dir_all(base_path.join(dir))
            .await
            .expect("Failed to create test directory");
    }

    StorageConfig {
        base_path,
        cache_expiration_seconds: 3600,
        max_cache_size_bytes: 10 * 1024 * 1024, // 10MB
        enable_content_addressing: true,
        index_update_interval_seconds: 300,
    }
}

/// Create a test research result file
pub async fn create_test_result_file(
    storage_config: &StorageConfig,
    result: &ResearchResult,
) -> std::result::Result<String, Box<dyn std::error::Error>> {
    let cache_key = format!("{}", Uuid::new_v4());
    let file_path = storage_config
        .base_path
        .join("research_results")
        .join(result.request.research_type.to_string().to_lowercase())
        .join(format!("{cache_key}.json"));

    let json = serde_json::to_string_pretty(result)?;
    fs::write(&file_path, json).await?;

    Ok(cache_key)
}

/// Assert that two research results are equivalent
pub fn assert_research_results_equal(a: &ResearchResult, b: &ResearchResult) {
    assert_eq!(a.request.original_query, b.request.original_query);
    assert_eq!(a.request.research_type, b.request.research_type);
    assert_eq!(a.immediate_answer, b.immediate_answer);
    assert_eq!(a.supporting_evidence.len(), b.supporting_evidence.len());
    assert_eq!(
        a.implementation_details.len(),
        b.implementation_details.len()
    );
}

/// Assert that a classification result meets basic requirements
pub fn assert_classification_valid(result: &ClassificationResult) {
    assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
    assert!(!result.candidates.is_empty());

    // Verify candidates are sorted by confidence
    for i in 1..result.candidates.len() {
        assert!(result.candidates[i - 1].confidence >= result.candidates[i].confidence);
    }
}

/// Assert that a classified request is valid
pub fn assert_classified_request_valid(request: &ClassifiedRequest) {
    assert!(!request.original_query.is_empty());
    assert!(request.confidence >= 0.0 && request.confidence <= 1.0);
    assert!(request.created_at <= chrono::Utc::now());
}

/// Assert that a cache entry is valid
pub fn assert_cache_entry_valid(entry: &CacheEntry) {
    assert!(!entry.key.is_empty());
    assert!(!entry.original_query.is_empty());
    assert!(entry.size_bytes > 0);
    assert!(!entry.content_hash.is_empty());
    assert!(entry.created_at <= chrono::Utc::now());
    assert!(entry.expires_at > entry.created_at);
}

/// Create a mock classifier that returns predictable results
pub struct MockClassifier {
    pub default_result: ClassificationResult,
    pub query_results: HashMap<String, ClassificationResult>,
}

impl MockClassifier {
    pub fn new(default_result: ClassificationResult) -> Self {
        Self {
            default_result,
            query_results: HashMap::new(),
        }
    }

    pub fn add_query_result(&mut self, query: String, result: ClassificationResult) {
        self.query_results.insert(query, result);
    }
}

impl Classifier for MockClassifier {
    fn classify(
        &self,
        query: &str,
    ) -> std::result::Result<ClassificationResult, ClassificationError> {
        if query.trim().is_empty() {
            return Err(ClassificationError::InvalidInput("Empty query".to_string()));
        }

        Ok(self
            .query_results
            .get(query)
            .cloned()
            .unwrap_or_else(|| self.default_result.clone()))
    }

    fn get_confidence(&self, query: &str, research_type: &ResearchType) -> f64 {
        if let Some(result) = self.query_results.get(query) {
            if result.research_type == *research_type {
                return result.confidence;
            }
        }

        if self.default_result.research_type == *research_type {
            self.default_result.confidence
        } else {
            0.0
        }
    }

    fn get_all_classifications(&self, query: &str) -> Vec<ClassificationCandidate> {
        self.query_results
            .get(query)
            .map(|r| r.candidates.clone())
            .unwrap_or_else(|| self.default_result.candidates.clone())
    }
}

/// Create a mock storage that keeps everything in memory
pub struct MockStorage {
    pub results: HashMap<String, ResearchResult>,
    pub cache_entries: HashMap<String, CacheEntry>,
}

impl MockStorage {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
            cache_entries: HashMap::new(),
        }
    }

    pub fn add_result(&mut self, cache_key: String, result: ResearchResult) {
        // Create corresponding cache entry
        let cache_entry = CacheEntry::new(
            cache_key.clone(),
            format!("/mock/{cache_key}.json").into(),
            result.request.research_type.clone(),
            result.request.original_query.clone(),
            1024, // Mock size
            "mock-hash".to_string(),
            3600,
        );

        self.cache_entries.insert(cache_key.clone(), cache_entry);
        self.results.insert(cache_key, result);
    }
}

impl Default for MockStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Storage for MockStorage {
    async fn store(&self, _result: &ResearchResult) -> std::result::Result<String, StorageError> {
        let cache_key = format!("{}", Uuid::new_v4());
        // In a real mock, we would store the result
        // For now, just return the cache key
        Ok(cache_key)
    }

    async fn retrieve(
        &self,
        cache_key: &str,
    ) -> std::result::Result<Option<ResearchResult>, StorageError> {
        Ok(self.results.get(cache_key).cloned())
    }

    async fn delete(&self, _cache_key: &str) -> std::result::Result<(), StorageError> {
        // Mock implementation - would remove from storage
        Ok(())
    }

    async fn list_cache_entries(&self) -> std::result::Result<Vec<CacheEntry>, StorageError> {
        Ok(self.cache_entries.values().cloned().collect())
    }

    async fn get_cache_stats(&self) -> std::result::Result<CacheStats, StorageError> {
        let total_entries = self.cache_entries.len();
        let total_size_bytes = self.cache_entries.values().map(|e| e.size_bytes).sum();

        Ok(CacheStats {
            total_entries,
            expired_entries: 0,
            total_size_bytes,
            hit_rate: 0.8, // Mock hit rate
            hits: 100,
            misses: 25,
            average_age_seconds: 1800.0,
            by_research_type: HashMap::new(),
            analytics: fortitude_types::CacheAnalytics::default(),
            newest_entry: None,
        })
    }

    async fn cleanup_expired(&self) -> std::result::Result<u64, StorageError> {
        Ok(0) // Mock: no expired entries
    }

    async fn search(
        &self,
        _query: &SearchQuery,
    ) -> std::result::Result<Vec<SearchResult>, StorageError> {
        // Mock implementation - return empty results
        Ok(vec![])
    }

    async fn update_index(&self) -> std::result::Result<(), StorageError> {
        Ok(())
    }

    async fn record_cache_operation(
        &self,
        _operation: fortitude_types::CacheOperation,
    ) -> std::result::Result<(), StorageError> {
        Ok(())
    }

    async fn get_performance_monitor(
        &self,
    ) -> std::result::Result<fortitude_types::CachePerformanceMonitor, StorageError> {
        Ok(fortitude_types::CachePerformanceMonitor {
            target_hit_rate: 0.9,
            current_hit_rate: 0.8,
            status: fortitude_types::CachePerformanceStatus::Optimal,
            recent_operations: vec![],
            alerts: vec![],
        })
    }

    async fn update_analytics(
        &self,
        _analytics: fortitude_types::CacheAnalytics,
    ) -> std::result::Result<(), StorageError> {
        Ok(())
    }

    async fn get_key_optimization_recommendations(
        &self,
    ) -> std::result::Result<Vec<String>, StorageError> {
        Ok(vec!["test_recommendation".to_string()])
    }

    async fn warm_cache(
        &self,
        _entries: Vec<String>,
    ) -> std::result::Result<fortitude_types::CacheWarmingStats, StorageError> {
        Ok(fortitude_types::CacheWarmingStats::default())
    }

    async fn get_hit_rate_trends(
        &self,
        _timeframe_hours: u64,
    ) -> std::result::Result<Vec<fortitude_types::HitRateTrend>, StorageError> {
        Ok(vec![])
    }
}

/// Test utilities for async operations
pub mod async_utils {
    use tokio::time::{sleep, Duration};

    /// Wait for a condition to be true with timeout
    pub async fn wait_for_condition<F>(mut condition: F, timeout_ms: u64) -> bool
    where
        F: FnMut() -> bool,
    {
        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        while start.elapsed() < timeout {
            if condition() {
                return true;
            }
            sleep(Duration::from_millis(10)).await;
        }

        false
    }

    /// Create a future that resolves after a delay
    pub async fn delay_ms(ms: u64) {
        sleep(Duration::from_millis(ms)).await;
    }
}

/// Performance testing utilities
pub mod perf_utils {
    use std::time::Instant;

    /// Measure execution time of a function
    pub fn measure_time<F, R>(f: F) -> (R, std::time::Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    /// Measure execution time of an async function
    pub async fn measure_time_async<F, Fut, R>(f: F) -> (R, std::time::Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let start = Instant::now();
        let result = f().await;
        let duration = start.elapsed();
        (result, duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::*;

    #[test]
    fn test_create_temp_dir() {
        let temp_dir = create_temp_dir();
        assert!(temp_dir.path().exists());
        assert!(temp_dir.path().is_dir());
    }

    #[tokio::test]
    async fn test_setup_temp_storage() {
        let temp_dir = create_temp_dir();
        let config = setup_temp_storage(&temp_dir).await;

        assert_eq!(config.base_path, temp_dir.path());
        assert!(config.base_path.join("research_results").exists());
        assert!(config.base_path.join("cache").exists());
        assert!(config.base_path.join("index").exists());

        // Check research type directories
        for research_type in ResearchType::all() {
            let type_dir = config
                .base_path
                .join("research_results")
                .join(research_type.to_string().to_lowercase());
            assert!(type_dir.exists());
        }
    }

    #[tokio::test]
    async fn test_create_test_result_file() {
        let temp_dir = create_temp_dir();
        let config = setup_temp_storage(&temp_dir).await;
        let result = sample_research_result("test query", ResearchType::Learning);

        let cache_key = create_test_result_file(&config, &result).await.unwrap();

        assert!(!cache_key.is_empty());

        let file_path = config
            .base_path
            .join("research_results")
            .join("learning")
            .join(format!("{cache_key}.json"));

        assert!(file_path.exists());
    }

    #[test]
    fn test_assert_research_results_equal() {
        let result1 = sample_research_result("test query", ResearchType::Implementation);
        let result2 = sample_research_result("test query", ResearchType::Implementation);

        assert_research_results_equal(&result1, &result2);
    }

    #[test]
    fn test_assert_classification_valid() {
        let result = sample_classification_result(ResearchType::Decision, 0.8);
        assert_classification_valid(&result);
    }

    #[test]
    fn test_assert_classified_request_valid() {
        let request = sample_classified_request("test query", ResearchType::Validation);
        assert_classified_request_valid(&request);
    }

    #[test]
    fn test_assert_cache_entry_valid() {
        let entry = sample_cache_entry();
        assert_cache_entry_valid(&entry);
    }

    #[test]
    fn test_mock_classifier() {
        let default_result = sample_classification_result(ResearchType::Learning, 0.5);
        let mut classifier = MockClassifier::new(default_result);

        let custom_result = sample_classification_result(ResearchType::Decision, 0.9);
        classifier.add_query_result("custom query".to_string(), custom_result.clone());

        // Test default result
        let result = classifier.classify("unknown query").unwrap();
        assert_eq!(result.research_type, ResearchType::Learning);
        assert_eq!(result.confidence, 0.5);

        // Test custom result
        let result = classifier.classify("custom query").unwrap();
        assert_eq!(result.research_type, ResearchType::Decision);
        assert_eq!(result.confidence, 0.9);

        // Test empty query
        let result = classifier.classify("");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_storage() {
        let mut storage = MockStorage::new();
        let result = sample_research_result("test query", ResearchType::Implementation);

        storage.add_result("test-key".to_string(), result.clone());

        // Test retrieve
        let retrieved = storage.retrieve("test-key").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().request.original_query, "test query");

        // Test cache entries
        let entries = storage.list_cache_entries().await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "test-key");

        // Test stats
        let stats = storage.get_cache_stats().await.unwrap();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.hits, 100);
        assert_eq!(stats.misses, 25);
    }

    #[tokio::test]
    async fn test_async_utils() {
        use async_utils::*;

        let mut counter = 0;
        let condition = || {
            counter += 1;
            counter >= 3
        };

        let result = wait_for_condition(condition, 1000).await;
        assert!(result);
        assert!(counter >= 3);
    }

    #[tokio::test]
    async fn test_perf_utils() {
        use perf_utils::*;

        let (result, duration) = measure_time(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
        assert!(duration.as_millis() >= 10);

        let (result, duration) = measure_time_async(|| async {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            "async result"
        })
        .await;

        assert_eq!(result, "async result");
        assert!(duration.as_millis() >= 10);
    }
}
