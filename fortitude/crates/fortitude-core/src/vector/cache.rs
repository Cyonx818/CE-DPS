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

// ABOUTME: Advanced multi-level caching system for vector operations
//! This module provides comprehensive caching strategies for embeddings, search results,
//! and vector operations to achieve <200ms response times and optimize throughput.

use crate::vector::error::VectorResult;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

/// Multi-level cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// L1 cache (in-memory, fastest)
    pub l1_config: L1CacheConfig,
    /// L2 cache (persistent, larger capacity)
    pub l2_config: L2CacheConfig,
    /// Cache eviction policy
    pub eviction_policy: EvictionPolicy,
    /// Cache statistics collection
    pub enable_stats: bool,
    /// Cache warming strategies
    pub warming_config: CacheWarmingConfig,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            l1_config: L1CacheConfig::default(),
            l2_config: L2CacheConfig::default(),
            eviction_policy: EvictionPolicy::LruWithSize,
            enable_stats: true,
            warming_config: CacheWarmingConfig::default(),
        }
    }
}

/// L1 cache configuration (in-memory, fast access)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L1CacheConfig {
    /// Enable L1 cache
    pub enabled: bool,
    /// Maximum number of entries
    pub max_entries: usize,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Entry TTL
    pub ttl: Duration,
    /// Preemptive refresh threshold (refresh when TTL reaches this fraction)
    pub refresh_threshold: f64,
}

impl Default for L1CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 10000,
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            ttl: Duration::from_secs(3600),      // 1 hour
            refresh_threshold: 0.8,
        }
    }
}

/// L2 cache configuration (persistent, larger capacity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2CacheConfig {
    /// Enable L2 cache
    pub enabled: bool,
    /// Maximum number of entries
    pub max_entries: usize,
    /// Maximum disk usage in bytes
    pub max_disk_bytes: usize,
    /// Entry TTL
    pub ttl: Duration,
    /// Cache directory path
    pub cache_dir: Option<std::path::PathBuf>,
}

impl Default for L2CacheConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for simplicity
            max_entries: 100000,
            max_disk_bytes: 1024 * 1024 * 1024, // 1GB
            ttl: Duration::from_secs(86400),    // 24 hours
            cache_dir: None,
        }
    }
}

/// Cache eviction policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// Least Recently Used
    Lru,
    /// Least Recently Used with size consideration
    LruWithSize,
    /// Least Frequently Used
    Lfu,
    /// Time-based expiration only
    TimeOnly,
    /// Adaptive Replacement Cache
    Arc,
}

/// Cache warming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheWarmingConfig {
    /// Enable cache warming
    pub enabled: bool,
    /// Pre-compute popular embeddings
    pub precompute_embeddings: bool,
    /// Pre-cache frequent searches
    pub precache_searches: bool,
    /// Popular queries for warming
    pub popular_queries: Vec<String>,
}

impl Default for CacheWarmingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            precompute_embeddings: true,
            precache_searches: true,
            popular_queries: vec![
                "research methodology".to_string(),
                "implementation patterns".to_string(),
                "best practices".to_string(),
                "troubleshooting guide".to_string(),
                "performance optimization".to_string(),
            ],
        }
    }
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    /// The cached value
    pub value: T,
    /// When the entry was created
    pub created_at: Instant,
    /// When the entry was last accessed
    pub last_accessed: Instant,
    /// Number of times accessed
    pub access_count: u64,
    /// Entry size in bytes (estimated)
    pub size_bytes: usize,
    /// Entry TTL
    pub ttl: Duration,
}

impl<T> CacheEntry<T> {
    pub fn new(value: T, ttl: Duration, size_bytes: usize) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            size_bytes,
            ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    pub fn should_refresh(&self, refresh_threshold: f64) -> bool {
        let elapsed_ratio = self.created_at.elapsed().as_secs_f64() / self.ttl.as_secs_f64();
        elapsed_ratio >= refresh_threshold
    }

    pub fn access(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total cache requests
    pub total_requests: u64,
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Hit rate (0.0-1.0)
    pub hit_rate: f64,
    /// Current L1 cache size (entries)
    pub l1_entries: usize,
    /// Current L1 memory usage (bytes)
    pub l1_memory_bytes: usize,
    /// Current L2 cache size (entries)
    pub l2_entries: usize,
    /// Current L2 disk usage (bytes)
    pub l2_disk_bytes: usize,
    /// Average response time for hits (ms)
    pub avg_hit_time_ms: f64,
    /// Average response time for misses (ms)
    pub avg_miss_time_ms: f64,
    /// Cache efficiency score (0.0-1.0)
    pub efficiency_score: f64,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            hits: 0,
            misses: 0,
            hit_rate: 0.0,
            l1_entries: 0,
            l1_memory_bytes: 0,
            l2_entries: 0,
            l2_disk_bytes: 0,
            avg_hit_time_ms: 0.0,
            avg_miss_time_ms: 0.0,
            efficiency_score: 0.0,
        }
    }
}

/// Advanced cache key with automatic optimization
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// Key namespace
    pub namespace: String,
    /// Primary key component
    pub key: String,
    /// Key version for cache invalidation
    pub version: u32,
    /// Key hash for fast comparison
    pub hash: u64,
}

impl CacheKey {
    pub fn new(namespace: &str, key: &str, version: u32) -> Self {
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        namespace.hash(&mut hasher);
        key.hash(&mut hasher);
        version.hash(&mut hasher);
        let hash = hasher.finish();

        Self {
            namespace: namespace.to_string(),
            key: key.to_string(),
            version,
            hash,
        }
    }

    pub fn embedding_key(text: &str) -> Self {
        Self::new("embedding", text, 1)
    }

    pub fn search_key(query: &str, options_hash: u64) -> Self {
        let key = format!("{query}:{options_hash}");
        Self::new("search", &key, 1)
    }

    pub fn vector_key(id: &str) -> Self {
        Self::new("vector", id, 1)
    }
}

/// Multi-level cache implementation
pub struct MultiLevelCache<T> {
    /// L1 cache (in-memory)
    l1_cache: Arc<DashMap<CacheKey, CacheEntry<T>>>,
    /// L2 cache (persistent) - simplified as in-memory for now
    l2_cache: Arc<DashMap<CacheKey, CacheEntry<T>>>,
    /// Cache configuration
    config: CacheConfig,
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
    /// Current memory usage
    current_memory: Arc<RwLock<usize>>,
}

impl<T: Clone + Send + Sync> MultiLevelCache<T> {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            l1_cache: Arc::new(DashMap::new()),
            l2_cache: Arc::new(DashMap::new()),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
            current_memory: Arc::new(RwLock::new(0)),
        }
    }

    /// Get value from cache with automatic L1/L2 promotion
    #[instrument(skip(self))]
    pub async fn get(&self, key: &CacheKey) -> Option<T> {
        let start_time = Instant::now();
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;

        // Try L1 cache first
        if let Some(mut entry) = self.l1_cache.get_mut(key) {
            if !entry.is_expired() {
                entry.access();
                stats.hits += 1;
                stats.hit_rate = stats.hits as f64 / stats.total_requests as f64;

                let response_time = start_time.elapsed().as_millis() as f64;
                stats.avg_hit_time_ms = (stats.avg_hit_time_ms * (stats.hits - 1) as f64
                    + response_time)
                    / stats.hits as f64;

                debug!("L1 cache hit for key: {:?}", key);
                return Some(entry.value.clone());
            } else {
                // Remove expired entry
                drop(entry);
                self.l1_cache.remove(key);
            }
        }

        // Try L2 cache
        if self.config.l2_config.enabled {
            if let Some(mut entry) = self.l2_cache.get_mut(key) {
                if !entry.is_expired() {
                    entry.access();
                    let value = entry.value.clone();

                    // Promote to L1 cache
                    self.set_l1(key.clone(), value.clone(), entry.size_bytes)
                        .await;

                    stats.hits += 1;
                    stats.hit_rate = stats.hits as f64 / stats.total_requests as f64;

                    let response_time = start_time.elapsed().as_millis() as f64;
                    stats.avg_hit_time_ms = (stats.avg_hit_time_ms * (stats.hits - 1) as f64
                        + response_time)
                        / stats.hits as f64;

                    debug!("L2 cache hit (promoted to L1) for key: {:?}", key);
                    return Some(value);
                } else {
                    // Remove expired entry
                    drop(entry);
                    self.l2_cache.remove(key);
                }
            }
        }

        // Cache miss
        stats.misses += 1;
        stats.hit_rate = stats.hits as f64 / stats.total_requests as f64;

        let response_time = start_time.elapsed().as_millis() as f64;
        stats.avg_miss_time_ms = (stats.avg_miss_time_ms * (stats.misses - 1) as f64
            + response_time)
            / stats.misses as f64;

        debug!("Cache miss for key: {:?}", key);
        None
    }

    /// Set value in cache with automatic level management
    #[instrument(skip(self, value))]
    pub async fn set(&self, key: CacheKey, value: T, size_bytes: usize) -> VectorResult<()> {
        // Always try to set in L1 first
        self.set_l1(key.clone(), value.clone(), size_bytes).await;

        // Set in L2 if enabled and item is large enough
        if self.config.l2_config.enabled && size_bytes > 1024 {
            self.set_l2(key, value, size_bytes).await;
        }

        Ok(())
    }

    /// Set value in L1 cache
    async fn set_l1(&self, key: CacheKey, value: T, size_bytes: usize) {
        // Check memory limits
        let current_memory = *self.current_memory.read().await;
        if current_memory + size_bytes > self.config.l1_config.max_memory_bytes {
            self.evict_l1_memory().await;
        }

        // Check entry limits
        if self.l1_cache.len() >= self.config.l1_config.max_entries {
            self.evict_l1_entries().await;
        }

        let entry = CacheEntry::new(value, self.config.l1_config.ttl, size_bytes);
        self.l1_cache.insert(key, entry);

        // Update memory usage
        let mut memory = self.current_memory.write().await;
        *memory += size_bytes;
    }

    /// Set value in L2 cache
    async fn set_l2(&self, key: CacheKey, value: T, size_bytes: usize) {
        // Check entry limits
        if self.l2_cache.len() >= self.config.l2_config.max_entries {
            self.evict_l2_entries().await;
        }

        let entry = CacheEntry::new(value, self.config.l2_config.ttl, size_bytes);
        self.l2_cache.insert(key, entry);
    }

    /// Evict entries from L1 cache based on memory pressure
    async fn evict_l1_memory(&self) {
        let target_memory = self.config.l1_config.max_memory_bytes * 8 / 10; // 80% of max
        let mut current_memory = *self.current_memory.read().await;

        match &self.config.eviction_policy {
            EvictionPolicy::Lru | EvictionPolicy::LruWithSize => {
                let mut entries: Vec<_> = self
                    .l1_cache
                    .iter()
                    .map(|entry| {
                        (
                            entry.key().clone(),
                            entry.value().last_accessed,
                            entry.value().size_bytes,
                        )
                    })
                    .collect();

                // Sort by last accessed time (oldest first)
                entries.sort_by_key(|(_, last_accessed, _)| *last_accessed);

                for (key, _, size_bytes) in entries {
                    if current_memory <= target_memory {
                        break;
                    }

                    if self.l1_cache.remove(&key).is_some() {
                        current_memory -= size_bytes;
                    }
                }
            }
            EvictionPolicy::Lfu => {
                let mut entries: Vec<_> = self
                    .l1_cache
                    .iter()
                    .map(|entry| {
                        (
                            entry.key().clone(),
                            entry.value().access_count,
                            entry.value().size_bytes,
                        )
                    })
                    .collect();

                // Sort by access count (least frequent first)
                entries.sort_by_key(|(_, access_count, _)| *access_count);

                for (key, _, size_bytes) in entries {
                    if current_memory <= target_memory {
                        break;
                    }

                    if self.l1_cache.remove(&key).is_some() {
                        current_memory -= size_bytes;
                    }
                }
            }
            _ => {
                // TimeOnly - remove expired entries
                self.cleanup_expired().await;
            }
        }

        // Update memory usage
        let mut memory = self.current_memory.write().await;
        *memory = current_memory;
    }

    /// Evict entries from L1 cache based on entry count
    async fn evict_l1_entries(&self) {
        let target_entries = self.config.l1_config.max_entries * 8 / 10; // 80% of max
        let current_entries = self.l1_cache.len();

        if current_entries <= target_entries {
            return;
        }

        let entries_to_remove = current_entries - target_entries;

        let mut entries: Vec<_> = self
            .l1_cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().last_accessed))
            .collect();

        // Sort by last accessed time (oldest first)
        entries.sort_by_key(|(_, last_accessed)| *last_accessed);

        let mut memory_freed = 0usize;
        for (key, _) in entries.into_iter().take(entries_to_remove) {
            if let Some((_, entry)) = self.l1_cache.remove(&key) {
                memory_freed += entry.size_bytes;
            }
        }

        // Update memory usage
        let mut memory = self.current_memory.write().await;
        *memory = memory.saturating_sub(memory_freed);
    }

    /// Evict entries from L2 cache
    async fn evict_l2_entries(&self) {
        let target_entries = self.config.l2_config.max_entries * 8 / 10; // 80% of max
        let current_entries = self.l2_cache.len();

        if current_entries <= target_entries {
            return;
        }

        let entries_to_remove = current_entries - target_entries;

        let mut entries: Vec<_> = self
            .l2_cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().last_accessed))
            .collect();

        // Sort by last accessed time (oldest first)
        entries.sort_by_key(|(_, last_accessed)| *last_accessed);

        for (key, _) in entries.into_iter().take(entries_to_remove) {
            self.l2_cache.remove(&key);
        }
    }

    /// Clean up expired entries
    #[instrument(skip(self))]
    pub async fn cleanup_expired(&self) {
        let mut expired_keys = Vec::new();
        let mut memory_freed = 0usize;

        // Check L1 cache
        for entry in self.l1_cache.iter() {
            if entry.value().is_expired() {
                expired_keys.push((entry.key().clone(), entry.value().size_bytes));
            }
        }

        let expired_count = expired_keys.len();
        for (key, size_bytes) in expired_keys {
            if self.l1_cache.remove(&key).is_some() {
                memory_freed += size_bytes;
            }
        }

        // Update memory usage
        let mut memory = self.current_memory.write().await;
        *memory = memory.saturating_sub(memory_freed);

        // Check L2 cache
        if self.config.l2_config.enabled {
            let mut l2_expired_keys = Vec::new();

            for entry in self.l2_cache.iter() {
                if entry.value().is_expired() {
                    l2_expired_keys.push(entry.key().clone());
                }
            }

            for key in l2_expired_keys {
                self.l2_cache.remove(&key);
            }
        }

        if expired_count > 0 {
            info!(
                "Cleaned up {} expired L1 entries, freed {} bytes",
                expired_count, memory_freed
            );
        }
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let mut stats = self.stats.read().await.clone();
        stats.l1_entries = self.l1_cache.len();
        stats.l1_memory_bytes = *self.current_memory.read().await;
        stats.l2_entries = self.l2_cache.len();

        // Calculate efficiency score
        if stats.total_requests > 0 {
            let hit_rate_score = stats.hit_rate;
            let memory_efficiency = if stats.l1_memory_bytes > 0 {
                1.0 - (stats.l1_memory_bytes as f64 / self.config.l1_config.max_memory_bytes as f64)
            } else {
                1.0
            };
            stats.efficiency_score = (hit_rate_score + memory_efficiency) / 2.0;
        }

        stats
    }

    /// Clear all cache entries
    pub async fn clear(&self) -> VectorResult<()> {
        self.l1_cache.clear();
        self.l2_cache.clear();

        let mut memory = self.current_memory.write().await;
        *memory = 0;

        let mut stats = self.stats.write().await;
        *stats = CacheStats::default();

        info!("Cache cleared");
        Ok(())
    }

    /// Warm up cache with popular items
    #[instrument(skip(self, warm_fn))]
    pub async fn warm_cache<F, Fut>(&self, warm_fn: F) -> VectorResult<()>
    where
        F: Fn(String) -> Fut,
        Fut: std::future::Future<Output = VectorResult<Option<(T, usize)>>>,
    {
        if !self.config.warming_config.enabled {
            return Ok(());
        }

        info!(
            "Starting cache warming with {} popular queries",
            self.config.warming_config.popular_queries.len()
        );

        for query in &self.config.warming_config.popular_queries {
            if let Ok(Some((value, size))) = warm_fn(query.clone()).await {
                let key = CacheKey::new("warm", query, 1);
                self.set(key, value, size).await?;
            }
        }

        info!("Cache warming completed");
        Ok(())
    }

    /// Check if cache needs preemptive refresh
    pub async fn needs_refresh(&self, key: &CacheKey) -> bool {
        if let Some(entry) = self.l1_cache.get(key) {
            entry.should_refresh(self.config.l1_config.refresh_threshold)
        } else {
            false
        }
    }
}

/// Cache manager for coordinating different cache types
pub struct CacheManager {
    /// Embedding cache
    pub embedding_cache: MultiLevelCache<Vec<f32>>,
    /// Search result cache
    pub search_cache: MultiLevelCache<Vec<serde_json::Value>>,
    /// Vector document cache
    pub vector_cache: MultiLevelCache<serde_json::Value>,
    /// Cache configuration
    #[allow(dead_code)] // TODO: Use config for cache management settings
    config: CacheConfig,
}

impl CacheManager {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            embedding_cache: MultiLevelCache::new(config.clone()),
            search_cache: MultiLevelCache::new(config.clone()),
            vector_cache: MultiLevelCache::new(config.clone()),
            config,
        }
    }

    /// Get aggregated cache statistics
    pub async fn get_aggregated_stats(&self) -> HashMap<String, CacheStats> {
        let mut stats = HashMap::new();
        stats.insert(
            "embedding".to_string(),
            self.embedding_cache.get_stats().await,
        );
        stats.insert("search".to_string(), self.search_cache.get_stats().await);
        stats.insert("vector".to_string(), self.vector_cache.get_stats().await);
        stats
    }

    /// Clear all caches
    pub async fn clear_all(&self) -> VectorResult<()> {
        self.embedding_cache.clear().await?;
        self.search_cache.clear().await?;
        self.vector_cache.clear().await?;
        Ok(())
    }

    /// Cleanup all caches
    pub async fn cleanup_all(&self) {
        self.embedding_cache.cleanup_expired().await;
        self.search_cache.cleanup_expired().await;
        self.vector_cache.cleanup_expired().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[test]
    fn test_cache_key_creation() {
        let key1 = CacheKey::embedding_key("test text");
        let key2 = CacheKey::embedding_key("test text");
        let key3 = CacheKey::embedding_key("different text");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
        assert_eq!(key1.namespace, "embedding");
    }

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let config = CacheConfig::default();
        let cache: MultiLevelCache<String> = MultiLevelCache::new(config);

        let key = CacheKey::new("test", "key1", 1);
        let value = "test_value".to_string();

        // Test set and get
        cache.set(key.clone(), value.clone(), 100).await.unwrap();
        let retrieved = cache.get(&key).await;

        assert_eq!(retrieved, Some(value));

        // Test stats
        let stats = cache.get_stats().await;
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.hit_rate, 1.0);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let mut config = CacheConfig::default();
        config.l1_config.ttl = Duration::from_millis(10);

        let cache: MultiLevelCache<String> = MultiLevelCache::new(config);
        let key = CacheKey::new("test", "expire_key", 1);
        let value = "expire_value".to_string();

        cache.set(key.clone(), value, 100).await.unwrap();

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(20)).await;

        let retrieved = cache.get(&key).await;
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_cache_manager() {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config);

        let embedding_key = CacheKey::embedding_key("test embedding");
        let embedding = vec![0.1, 0.2, 0.3];

        manager
            .embedding_cache
            .set(embedding_key.clone(), embedding.clone(), 100)
            .await
            .unwrap();

        let retrieved = manager.embedding_cache.get(&embedding_key).await;
        assert_eq!(retrieved, Some(embedding));

        let stats = manager.get_aggregated_stats().await;
        assert!(stats.contains_key("embedding"));
        assert!(stats.contains_key("search"));
        assert!(stats.contains_key("vector"));
    }
}
