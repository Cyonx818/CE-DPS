// ABOUTME: Embedding generation service for converting text to vectors
//! This module provides text-to-vector conversion capabilities using local
//! embedding models with caching, batch processing, and error handling.

use crate::vector::error::{VectorError, VectorResult};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

// Mock types for demonstration - replace with actual candle types in production
#[derive(Debug, Clone)]
pub struct MockDevice;

#[derive(Debug)]
pub struct MockBertModel;

#[derive(Debug)]
pub struct MockTokenizer;

/// Configuration for embedding generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Model name or path for embedding generation
    pub model_name: String,
    /// Maximum sequence length for tokenization
    pub max_sequence_length: usize,
    /// Batch size for processing multiple texts
    pub batch_size: usize,
    /// Device to use for inference (cpu/cuda)
    pub device: DeviceType,
    /// Cache configuration
    pub cache_config: EmbeddingCacheConfig,
    /// Model download configuration
    pub download_config: ModelDownloadConfig,
    /// Text preprocessing options
    pub preprocessing: PreprocessingConfig,
}

/// Device type for model inference
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum DeviceType {
    #[default]
    Cpu,
    Cuda,
    Metal,
}

impl From<DeviceType> for MockDevice {
    fn from(_device_type: DeviceType) -> Self {
        // Mock implementation - replace with actual Device conversion in production
        MockDevice
    }
}

/// Cache configuration for embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingCacheConfig {
    /// Enable embedding caching
    pub enabled: bool,
    /// Maximum number of cached embeddings
    pub max_entries: usize,
    /// TTL for cached embeddings
    pub ttl: Duration,
    /// Cache key strategy
    pub key_strategy: CacheKeyStrategy,
}

/// Strategy for generating cache keys
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum CacheKeyStrategy {
    /// Use SHA256 hash of the text
    #[default]
    Hash,
    /// Use text length + hash suffix
    LengthHash,
    /// Use first N characters + hash
    PrefixHash(usize),
}

impl Default for EmbeddingCacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 10000,
            ttl: Duration::from_secs(3600), // 1 hour
            key_strategy: CacheKeyStrategy::default(),
        }
    }
}

/// Model download configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDownloadConfig {
    /// Cache directory for downloaded models
    pub cache_dir: Option<PathBuf>,
    /// Enable offline mode (use only local models)
    pub offline: bool,
    /// Download timeout
    pub timeout: Duration,
}

impl Default for ModelDownloadConfig {
    fn default() -> Self {
        Self {
            cache_dir: None,
            offline: false,
            timeout: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Text preprocessing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessingConfig {
    /// Enable lowercase normalization
    pub lowercase: bool,
    /// Enable whitespace normalization
    pub normalize_whitespace: bool,
    /// Remove special characters
    pub remove_special_chars: bool,
    /// Maximum text length before truncation
    pub max_text_length: usize,
}

impl Default for PreprocessingConfig {
    fn default() -> Self {
        Self {
            lowercase: true,
            normalize_whitespace: true,
            remove_special_chars: false,
            max_text_length: 8192, // 8K characters
        }
    }
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            max_sequence_length: 512,
            batch_size: 32,
            device: DeviceType::default(),
            cache_config: EmbeddingCacheConfig::default(),
            download_config: ModelDownloadConfig::default(),
            preprocessing: PreprocessingConfig::default(),
        }
    }
}

/// Cached embedding entry
#[derive(Debug, Clone)]
struct CachedEmbedding {
    /// The embedding vector
    embedding: Vec<f32>,
    /// Timestamp when cached
    cached_at: Instant,
    /// Access count for LRU eviction
    access_count: u64,
}

/// Statistics about embedding operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingStats {
    /// Total embeddings generated
    pub total_generated: u64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Average generation time in milliseconds
    pub avg_generation_time_ms: f64,
    /// Current cache size
    pub cache_size: usize,
}

/// Trait for embedding generation services
#[async_trait]
pub trait EmbeddingGenerator: Send + Sync {
    /// Generate embedding for a single text
    async fn generate_embedding(&self, text: &str) -> VectorResult<Vec<f32>>;

    /// Generate embeddings for multiple texts in batch
    async fn generate_embeddings(&self, texts: &[String]) -> VectorResult<Vec<Vec<f32>>>;

    /// Get embedding statistics
    async fn get_stats(&self) -> EmbeddingStats;

    /// Clear the embedding cache
    async fn clear_cache(&self) -> VectorResult<()>;

    /// Get the embedding dimension
    fn embedding_dimension(&self) -> usize;
}

/// Local embedding service using mock implementation (replace with Candle framework in production)
pub struct LocalEmbeddingService {
    config: EmbeddingConfig,
    model: Arc<RwLock<Option<MockBertModel>>>,
    tokenizer: Arc<RwLock<Option<MockTokenizer>>>,
    #[allow(dead_code)] // TODO: Use device for embedding model placement
    device: MockDevice,
    cache: Arc<DashMap<String, CachedEmbedding>>,
    stats: Arc<RwLock<EmbeddingStats>>,
}

impl Clone for LocalEmbeddingService {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            model: self.model.clone(),
            tokenizer: self.tokenizer.clone(),
            device: self.device.clone(),
            cache: self.cache.clone(),
            stats: self.stats.clone(),
        }
    }
}

impl LocalEmbeddingService {
    /// Create a new local embedding service
    pub fn new(config: EmbeddingConfig) -> Self {
        let device: MockDevice = config.device.clone().into();

        Self {
            config,
            model: Arc::new(RwLock::new(None)),
            tokenizer: Arc::new(RwLock::new(None)),
            device,
            cache: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(EmbeddingStats {
                total_generated: 0,
                cache_hit_rate: 0.0,
                avg_generation_time_ms: 0.0,
                cache_size: 0,
            })),
        }
    }

    /// Initialize the model and tokenizer
    #[instrument(skip(self))]
    pub async fn initialize(&self) -> VectorResult<()> {
        info!(
            "Initializing embedding service with model: {}",
            self.config.model_name
        );

        // Download/load tokenizer
        let tokenizer = self.load_tokenizer().await?;

        // Download/load model
        let model = self.load_model().await?;

        // Update the shared state
        *self.tokenizer.write().await = Some(tokenizer);
        *self.model.write().await = Some(model);

        info!("Embedding service initialized successfully");
        Ok(())
    }

    /// Load the tokenizer (mock implementation)
    async fn load_tokenizer(&self) -> VectorResult<MockTokenizer> {
        debug!("Loading tokenizer for model: {}", self.config.model_name);

        // Mock implementation - replace with actual tokenizer loading in production
        // Simulate some async work
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(MockTokenizer)
    }

    /// Load the model (mock implementation)
    async fn load_model(&self) -> VectorResult<MockBertModel> {
        debug!("Loading model: {}", self.config.model_name);

        // Mock implementation - replace with actual model loading in production
        // Simulate some async work and model loading time
        tokio::time::sleep(Duration::from_millis(500)).await;

        Ok(MockBertModel)
    }

    /// Preprocess text according to configuration
    fn preprocess_text(&self, text: &str) -> String {
        let mut processed = text.to_string();

        if self.config.preprocessing.normalize_whitespace {
            processed = processed.split_whitespace().collect::<Vec<_>>().join(" ");
        }

        if self.config.preprocessing.lowercase {
            processed = processed.to_lowercase();
        }

        if self.config.preprocessing.remove_special_chars {
            processed = processed
                .chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                .collect();
        }

        if processed.len() > self.config.preprocessing.max_text_length {
            processed.truncate(self.config.preprocessing.max_text_length);
        }

        processed
    }

    /// Generate cache key for text
    fn generate_cache_key(&self, text: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        match &self.config.cache_config.key_strategy {
            CacheKeyStrategy::Hash => {
                let mut hasher = DefaultHasher::new();
                text.hash(&mut hasher);
                format!("emb_{:x}", hasher.finish())
            }
            CacheKeyStrategy::LengthHash => {
                let mut hasher = DefaultHasher::new();
                text.hash(&mut hasher);
                format!("emb_{}_{:x}", text.len(), hasher.finish())
            }
            CacheKeyStrategy::PrefixHash(n) => {
                let mut hasher = DefaultHasher::new();
                text.hash(&mut hasher);
                let prefix = text.chars().take(*n).collect::<String>();
                format!("emb_{}__{:x}", prefix, hasher.finish())
            }
        }
    }

    /// Check if cached embedding is still valid
    fn is_cache_valid(&self, cached: &CachedEmbedding) -> bool {
        cached.cached_at.elapsed() < self.config.cache_config.ttl
    }

    /// Clean expired cache entries
    async fn cleanup_cache(&self) {
        if !self.config.cache_config.enabled {
            return;
        }

        let mut expired_keys = Vec::new();

        // First, remove expired entries
        for entry in self.cache.iter() {
            if !self.is_cache_valid(entry.value()) {
                expired_keys.push(entry.key().clone());
            }
        }

        for key in expired_keys {
            self.cache.remove(&key);
        }

        // If still over capacity, implement LRU eviction
        while self.cache.len() >= self.config.cache_config.max_entries {
            // Find the least recently used entry (oldest cached_at)
            let mut oldest_key: Option<String> = None;
            let mut oldest_time = Instant::now();

            for entry in self.cache.iter() {
                if entry.value().cached_at < oldest_time {
                    oldest_time = entry.value().cached_at;
                    oldest_key = Some(entry.key().clone());
                }
            }

            if let Some(key) = oldest_key {
                self.cache.remove(&key);
            } else {
                break; // Safety break in case of unexpected state
            }
        }

        // Update cache size in stats
        let mut stats = self.stats.write().await;
        stats.cache_size = self.cache.len();
    }

    /// Generate embedding using the model (mock implementation)
    async fn generate_embedding_internal(&self, text: &str) -> VectorResult<Vec<f32>> {
        let start_time = Instant::now();

        // Get model and tokenizer
        let model_guard = self.model.read().await;
        let tokenizer_guard = self.tokenizer.read().await;

        let _model = model_guard
            .as_ref()
            .ok_or_else(|| VectorError::EmbeddingError("Model not initialized".to_string()))?;

        let _tokenizer = tokenizer_guard
            .as_ref()
            .ok_or_else(|| VectorError::EmbeddingError("Tokenizer not initialized".to_string()))?;

        // Mock implementation - replace with actual inference in production
        // Generate a deterministic but pseudo-random embedding based on text hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        // Generate 384-dimensional embedding (matching all-MiniLM-L6-v2)
        let mut embedding = Vec::with_capacity(384);
        let mut rng_state = hash;

        for _ in 0..384 {
            // Simple linear congruential generator for deterministic pseudo-randomness
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            let normalized = (rng_state % 1000) as f32 / 1000.0 - 0.5; // Range [-0.5, 0.5]
            embedding.push(normalized);
        }

        // Normalize the vector to unit length (common for embeddings)
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in &mut embedding {
                *value /= norm;
            }
        }

        // Simulate some processing time
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Update stats
        let generation_time = start_time.elapsed().as_millis() as f64;
        let mut stats = self.stats.write().await;
        stats.total_generated += 1;

        // Update rolling average
        let n = stats.total_generated as f64;
        stats.avg_generation_time_ms =
            (stats.avg_generation_time_ms * (n - 1.0) + generation_time) / n;

        Ok(embedding)
    }
}

#[async_trait]
impl EmbeddingGenerator for LocalEmbeddingService {
    #[instrument(skip(self, text))]
    async fn generate_embedding(&self, text: &str) -> VectorResult<Vec<f32>> {
        let processed_text = self.preprocess_text(text);

        // Check cache first
        if self.config.cache_config.enabled {
            let cache_key = self.generate_cache_key(&processed_text);

            if let Some(cached) = self.cache.get(&cache_key) {
                if self.is_cache_valid(&cached) {
                    debug!("Cache hit for embedding");

                    // Update access count
                    drop(cached);
                    if let Some(mut entry) = self.cache.get_mut(&cache_key) {
                        entry.access_count += 1;
                    }

                    // Update cache hit rate
                    let mut stats = self.stats.write().await;
                    let total_requests = stats.total_generated + 1;
                    let cache_hits = (stats.cache_hit_rate * stats.total_generated as f64) + 1.0;
                    stats.cache_hit_rate = cache_hits / total_requests as f64;

                    return Ok(self.cache.get(&cache_key).unwrap().embedding.clone());
                }
            }
        }

        // Generate new embedding
        let embedding = self.generate_embedding_internal(&processed_text).await?;

        // Cache the result
        if self.config.cache_config.enabled {
            let cache_key = self.generate_cache_key(&processed_text);

            // Check cache size limit
            if self.cache.len() >= self.config.cache_config.max_entries {
                self.cleanup_cache().await;
            }

            self.cache.insert(
                cache_key,
                CachedEmbedding {
                    embedding: embedding.clone(),
                    cached_at: Instant::now(),
                    access_count: 1,
                },
            );
        }

        Ok(embedding)
    }

    async fn generate_embeddings(&self, texts: &[String]) -> VectorResult<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());

        // Process in batches
        for chunk in texts.chunks(self.config.batch_size) {
            let mut batch_results = Vec::with_capacity(chunk.len());

            for text in chunk {
                let embedding = self.generate_embedding(text).await?;
                batch_results.push(embedding);
            }

            results.extend(batch_results);
        }

        Ok(results)
    }

    async fn get_stats(&self) -> EmbeddingStats {
        let mut stats = self.stats.read().await.clone();
        stats.cache_size = self.cache.len();
        stats
    }

    async fn clear_cache(&self) -> VectorResult<()> {
        self.cache.clear();
        let mut stats = self.stats.write().await;
        stats.cache_size = 0;
        stats.cache_hit_rate = 0.0;
        info!("Embedding cache cleared");
        Ok(())
    }

    fn embedding_dimension(&self) -> usize {
        // This should match the model's output dimension
        // For all-MiniLM-L6-v2, it's 384
        384
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    // Include comprehensive tests from separate file
    include!("embedding_tests.rs");

    #[test]
    fn test_embedding_config_default() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.model_name, "sentence-transformers/all-MiniLM-L6-v2");
        assert_eq!(config.max_sequence_length, 512);
        assert_eq!(config.batch_size, 32);
        assert!(config.cache_config.enabled);
    }

    #[test]
    fn test_preprocessing_config() {
        let config = PreprocessingConfig {
            lowercase: true,
            normalize_whitespace: true,
            remove_special_chars: false,
            max_text_length: 100,
        };

        let service = LocalEmbeddingService::new(EmbeddingConfig {
            preprocessing: config,
            ..Default::default()
        });

        let processed = service.preprocess_text("  Hello   WORLD!  ");
        assert_eq!(processed, "hello world!");
    }

    #[test]
    fn test_cache_key_generation() {
        let config = EmbeddingConfig::default();
        let service = LocalEmbeddingService::new(config);

        let key1 = service.generate_cache_key("hello world");
        let key2 = service.generate_cache_key("hello world");
        let key3 = service.generate_cache_key("different text");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[tokio::test]
    async fn test_embedding_stats() {
        let config = EmbeddingConfig::default();
        let service = LocalEmbeddingService::new(config);

        let stats = service.get_stats().await;
        assert_eq!(stats.total_generated, 0);
        assert_eq!(stats.cache_hit_rate, 0.0);
        assert_eq!(stats.cache_size, 0);
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let mut config = EmbeddingConfig::default();
        config.cache_config.ttl = Duration::from_millis(1); // Very short TTL

        let service = LocalEmbeddingService::new(config);

        // Add an entry to cache manually
        service.cache.insert(
            "test_key".to_string(),
            CachedEmbedding {
                embedding: vec![1.0, 2.0, 3.0],
                cached_at: Instant::now() - Duration::from_millis(10), // Already expired
                access_count: 1,
            },
        );

        assert_eq!(service.cache.len(), 1);

        service.cleanup_cache().await;

        assert_eq!(service.cache.len(), 0);
    }

    #[test]
    fn test_device_type_default() {
        let device = DeviceType::default();
        assert!(matches!(device, DeviceType::Cpu));
    }

    #[test]
    fn test_device_type_conversion() {
        let cpu_device = DeviceType::Cpu;
        let cuda_device = DeviceType::Cuda;
        let metal_device = DeviceType::Metal;

        // Test that conversions work (mock implementation)
        let _: MockDevice = cpu_device.into();
        let _: MockDevice = cuda_device.into();
        let _: MockDevice = metal_device.into();
    }

    #[test]
    fn test_cache_key_strategy_default() {
        let strategy = CacheKeyStrategy::default();
        assert!(matches!(strategy, CacheKeyStrategy::Hash));
    }

    #[test]
    fn test_cache_key_strategy_prefix_hash() {
        let config = EmbeddingConfig {
            cache_config: EmbeddingCacheConfig {
                key_strategy: CacheKeyStrategy::PrefixHash(5),
                ..Default::default()
            },
            ..Default::default()
        };

        let service = LocalEmbeddingService::new(config);
        let key1 = service.generate_cache_key("hello world test");
        let key2 = service.generate_cache_key("hello world different");

        // Should both start with "emb_hello"
        assert!(key1.starts_with("emb_hello"));
        assert!(key2.starts_with("emb_hello"));
    }

    #[test]
    fn test_cache_key_strategy_length_hash() {
        let config = EmbeddingConfig {
            cache_config: EmbeddingCacheConfig {
                key_strategy: CacheKeyStrategy::LengthHash,
                ..Default::default()
            },
            ..Default::default()
        };

        let service = LocalEmbeddingService::new(config);
        let key1 = service.generate_cache_key("hello");
        let key2 = service.generate_cache_key("world");

        // Should include length in key
        assert!(key1.contains("5")); // "hello" has 5 characters
        assert!(key2.contains("5")); // "world" has 5 characters
    }

    #[test]
    fn test_text_preprocessing_lowercase() {
        let config = EmbeddingConfig {
            preprocessing: PreprocessingConfig {
                lowercase: true,
                normalize_whitespace: false,
                remove_special_chars: false,
                max_text_length: 1000,
            },
            ..Default::default()
        };

        let service = LocalEmbeddingService::new(config);
        let processed = service.preprocess_text("HELLO WORLD");
        assert_eq!(processed, "hello world");
    }

    #[test]
    fn test_text_preprocessing_whitespace_normalization() {
        let config = EmbeddingConfig {
            preprocessing: PreprocessingConfig {
                lowercase: false,
                normalize_whitespace: true,
                remove_special_chars: false,
                max_text_length: 1000,
            },
            ..Default::default()
        };

        let service = LocalEmbeddingService::new(config);
        let processed = service.preprocess_text("hello    world   test");
        assert_eq!(processed, "hello world test");
    }

    #[test]
    fn test_text_preprocessing_remove_special_chars() {
        let config = EmbeddingConfig {
            preprocessing: PreprocessingConfig {
                lowercase: false,
                normalize_whitespace: false,
                remove_special_chars: true,
                max_text_length: 1000,
            },
            ..Default::default()
        };

        let service = LocalEmbeddingService::new(config);
        let processed = service.preprocess_text("hello!@# world$%^");
        assert_eq!(processed, "hello world");
    }

    #[test]
    fn test_text_preprocessing_max_length_truncation() {
        let config = EmbeddingConfig {
            preprocessing: PreprocessingConfig {
                lowercase: false,
                normalize_whitespace: false,
                remove_special_chars: false,
                max_text_length: 10,
            },
            ..Default::default()
        };

        let service = LocalEmbeddingService::new(config);
        let processed =
            service.preprocess_text("this is a very long text that should be truncated");
        assert_eq!(processed.len(), 10);
        assert_eq!(processed, "this is a ");
    }

    #[test]
    fn test_text_preprocessing_combined() {
        let config = EmbeddingConfig {
            preprocessing: PreprocessingConfig {
                lowercase: true,
                normalize_whitespace: true,
                remove_special_chars: true,
                max_text_length: 20,
            },
            ..Default::default()
        };

        let service = LocalEmbeddingService::new(config);
        let processed = service.preprocess_text("  HELLO!!!   WORLD@@@   TEST$$$  ");
        assert_eq!(processed, "hello world test");
        assert!(processed.len() <= 20);
    }

    #[test]
    fn test_embedding_cache_config_default() {
        let cache_config = EmbeddingCacheConfig::default();

        assert!(cache_config.enabled);
        assert_eq!(cache_config.max_entries, 10000);
        assert_eq!(cache_config.ttl, Duration::from_secs(3600));
        assert!(matches!(cache_config.key_strategy, CacheKeyStrategy::Hash));
    }

    #[test]
    fn test_model_download_config_default() {
        let download_config = ModelDownloadConfig::default();

        assert!(download_config.cache_dir.is_none());
        assert!(!download_config.offline);
        assert_eq!(download_config.timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_preprocessing_config_default() {
        let preprocessing_config = PreprocessingConfig::default();

        assert!(preprocessing_config.lowercase);
        assert!(preprocessing_config.normalize_whitespace);
        assert!(!preprocessing_config.remove_special_chars);
        assert_eq!(preprocessing_config.max_text_length, 8192);
    }

    #[tokio::test]
    async fn test_embedding_service_initialization() {
        let config = EmbeddingConfig::default();
        let service = LocalEmbeddingService::new(config);

        // Test that initialization completes without error
        let result = service.initialize().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_embedding_dimension_consistency() {
        let config = EmbeddingConfig::default();
        let service = LocalEmbeddingService::new(config);

        assert_eq!(service.embedding_dimension(), 384);
    }

    #[tokio::test]
    async fn test_generate_embedding_consistency() {
        let config = EmbeddingConfig::default();
        let service = LocalEmbeddingService::new(config);

        // Initialize service
        service.initialize().await.unwrap();

        let text = "test embedding generation";
        let embedding1 = service.generate_embedding(text).await.unwrap();
        let embedding2 = service.generate_embedding(text).await.unwrap();

        // Should be deterministic and identical for same input
        assert_eq!(embedding1, embedding2);
        assert_eq!(embedding1.len(), 384);
    }

    #[tokio::test]
    async fn test_generate_embedding_different_texts() {
        let config = EmbeddingConfig::default();
        let service = LocalEmbeddingService::new(config);

        service.initialize().await.unwrap();

        let embedding1 = service.generate_embedding("hello world").await.unwrap();
        let embedding2 = service.generate_embedding("goodbye world").await.unwrap();

        // Different texts should produce different embeddings
        assert_ne!(embedding1, embedding2);
        assert_eq!(embedding1.len(), 384);
        assert_eq!(embedding2.len(), 384);
    }

    #[tokio::test]
    async fn test_cache_hit_functionality() {
        let config = EmbeddingConfig::default();
        let service = LocalEmbeddingService::new(config);

        service.initialize().await.unwrap();

        let text = "cached embedding test";

        // First call should generate and cache
        let _embedding1 = service.generate_embedding(text).await.unwrap();

        // Second call should hit cache
        let _embedding2 = service.generate_embedding(text).await.unwrap();

        let stats = service.get_stats().await;
        assert!(stats.cache_hit_rate > 0.0);
    }

    #[tokio::test]
    async fn test_batch_embedding_generation() {
        let config = EmbeddingConfig::default();
        let service = LocalEmbeddingService::new(config);

        service.initialize().await.unwrap();

        let texts = vec![
            "first text".to_string(),
            "second text".to_string(),
            "third text".to_string(),
        ];

        let embeddings = service.generate_embeddings(&texts).await.unwrap();

        assert_eq!(embeddings.len(), 3);
        for embedding in embeddings {
            assert_eq!(embedding.len(), 384);
        }
    }

    #[tokio::test]
    async fn test_embedding_stats_tracking() {
        let config = EmbeddingConfig::default();
        let service = LocalEmbeddingService::new(config);

        service.initialize().await.unwrap();

        let initial_stats = service.get_stats().await;
        assert_eq!(initial_stats.total_generated, 0);

        let _embedding = service.generate_embedding("test").await.unwrap();

        let updated_stats = service.get_stats().await;
        assert_eq!(updated_stats.total_generated, 1);
        assert!(updated_stats.avg_generation_time_ms > 0.0);
    }

    #[tokio::test]
    async fn test_clear_cache_functionality() {
        let config = EmbeddingConfig::default();
        let service = LocalEmbeddingService::new(config);

        service.initialize().await.unwrap();

        // Generate some embeddings to populate cache
        let _embedding1 = service.generate_embedding("test1").await.unwrap();
        let _embedding2 = service.generate_embedding("test2").await.unwrap();

        let stats_before = service.get_stats().await;
        assert!(stats_before.cache_size > 0);

        // Clear cache
        service.clear_cache().await.unwrap();

        let stats_after = service.get_stats().await;
        assert_eq!(stats_after.cache_size, 0);
        assert_eq!(stats_after.cache_hit_rate, 0.0);
    }

    #[test]
    fn test_cached_embedding_structure() {
        let embedding = CachedEmbedding {
            embedding: vec![0.1, 0.2, 0.3],
            cached_at: Instant::now(),
            access_count: 5,
        };

        assert_eq!(embedding.embedding.len(), 3);
        assert_eq!(embedding.access_count, 5);
    }

    #[test]
    fn test_embedding_stats_structure() {
        let stats = EmbeddingStats {
            total_generated: 100,
            cache_hit_rate: 0.75,
            avg_generation_time_ms: 25.5,
            cache_size: 50,
        };

        assert_eq!(stats.total_generated, 100);
        assert_eq!(stats.cache_hit_rate, 0.75);
        assert_eq!(stats.avg_generation_time_ms, 25.5);
        assert_eq!(stats.cache_size, 50);
    }

    #[tokio::test]
    async fn test_cache_ttl_expiration() {
        let mut config = EmbeddingConfig::default();
        config.cache_config.ttl = Duration::from_millis(50);

        let service = LocalEmbeddingService::new(config);
        service.initialize().await.unwrap();

        // Generate embedding to cache it
        let text = "ttl test";
        let _embedding = service.generate_embedding(text).await.unwrap();

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(100)).await;

        // This should trigger cache cleanup and regenerate
        let _embedding2 = service.generate_embedding(text).await.unwrap();

        // Cache should be cleaned up
        let stats = service.get_stats().await;
        // Due to timing, we can't guarantee exact cache size, but it should be small
        assert!(stats.cache_size <= 1);
    }

    #[tokio::test]
    async fn test_cache_max_entries_limit() {
        let mut config = EmbeddingConfig::default();
        config.cache_config.max_entries = 2; // Very small cache

        let service = LocalEmbeddingService::new(config);
        service.initialize().await.unwrap();

        // Generate more embeddings than cache limit
        let _embedding1 = service.generate_embedding("text1").await.unwrap();
        let _embedding2 = service.generate_embedding("text2").await.unwrap();
        let _embedding3 = service.generate_embedding("text3").await.unwrap();

        let stats = service.get_stats().await;
        // Cache should not exceed max_entries
        assert!(stats.cache_size <= 2);
    }

    #[test]
    fn test_is_cache_valid_functionality() {
        let config = EmbeddingConfig {
            cache_config: EmbeddingCacheConfig {
                ttl: Duration::from_secs(60),
                ..Default::default()
            },
            ..Default::default()
        };

        let service = LocalEmbeddingService::new(config);

        // Fresh cache entry should be valid
        let fresh_entry = CachedEmbedding {
            embedding: vec![0.1, 0.2],
            cached_at: Instant::now(),
            access_count: 1,
        };
        assert!(service.is_cache_valid(&fresh_entry));

        // Old cache entry should be invalid
        let old_entry = CachedEmbedding {
            embedding: vec![0.1, 0.2],
            cached_at: Instant::now() - Duration::from_secs(120),
            access_count: 1,
        };
        assert!(!service.is_cache_valid(&old_entry));
    }

    #[test]
    fn test_embedding_config_serialization() {
        let config = EmbeddingConfig::default();

        let serialized = serde_json::to_string(&config).expect("Failed to serialize config");
        let deserialized: EmbeddingConfig =
            serde_json::from_str(&serialized).expect("Failed to deserialize config");

        assert_eq!(config.model_name, deserialized.model_name);
        assert_eq!(config.max_sequence_length, deserialized.max_sequence_length);
        assert_eq!(config.batch_size, deserialized.batch_size);
    }

    #[test]
    fn test_embedding_stats_serialization() {
        let stats = EmbeddingStats {
            total_generated: 42,
            cache_hit_rate: 0.85,
            avg_generation_time_ms: 15.7,
            cache_size: 100,
        };

        let serialized = serde_json::to_string(&stats).expect("Failed to serialize stats");
        let deserialized: EmbeddingStats =
            serde_json::from_str(&serialized).expect("Failed to deserialize stats");

        assert_eq!(stats.total_generated, deserialized.total_generated);
        assert_eq!(stats.cache_hit_rate, deserialized.cache_hit_rate);
        assert_eq!(
            stats.avg_generation_time_ms,
            deserialized.avg_generation_time_ms
        );
        assert_eq!(stats.cache_size, deserialized.cache_size);
    }
}
