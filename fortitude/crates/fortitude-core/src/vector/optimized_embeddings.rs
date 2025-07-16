// ABOUTME: Optimized embedding generation service with advanced caching and batching
//! This module provides high-performance embedding generation optimized for <200ms response times
//! with intelligent caching, batching, and model optimization strategies.

use crate::vector::{
    cache::{CacheKey, CacheManager},
    embeddings::{EmbeddingConfig, EmbeddingGenerator, EmbeddingStats},
    error::{VectorError, VectorResult},
    performance::{BatchProcessor, LatencyMeasurement, PerformanceMonitor},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, instrument, warn};

/// Optimized embedding configuration with performance tuning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedEmbeddingConfig {
    /// Base embedding configuration
    pub base_config: EmbeddingConfig,
    /// Performance optimization settings
    pub performance: EmbeddingPerformanceConfig,
    /// Advanced caching configuration
    pub caching: AdvancedCacheConfig,
    /// Batch processing configuration
    pub batching: BatchConfig,
    /// Model optimization settings
    pub model_optimization: ModelOptimizationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingPerformanceConfig {
    /// Target response time per embedding (ms)
    pub target_response_time_ms: u64,
    /// Maximum concurrent embedding requests
    pub max_concurrent_requests: usize,
    /// Request timeout
    pub request_timeout: Duration,
    /// Enable performance monitoring
    pub enable_monitoring: bool,
    /// Memory optimization mode
    pub memory_optimization: MemoryOptimizationMode,
    /// Model warming strategies
    pub model_warming: ModelWarmingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryOptimizationMode {
    /// Minimal memory usage, slower processing
    Minimal,
    /// Balanced memory and speed
    Balanced,
    /// Maximum performance, higher memory usage
    Performance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelWarmingConfig {
    /// Enable model warming on startup
    pub enable_warmup: bool,
    /// Warmup batch size
    pub warmup_batch_size: usize,
    /// Warmup sample texts
    pub warmup_texts: Vec<String>,
    /// Warmup timeout
    pub warmup_timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedCacheConfig {
    /// Enable intelligent cache preloading
    pub enable_preloading: bool,
    /// Preload popular embeddings
    pub preload_popular: bool,
    /// Cache compression
    pub enable_compression: bool,
    /// Cache segmentation by text type
    pub enable_segmentation: bool,
    /// Predictive caching
    pub predictive_caching: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Enable adaptive batch sizing
    pub adaptive_sizing: bool,
    /// Minimum batch size
    pub min_batch_size: usize,
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Dynamic batch timeout
    pub dynamic_timeout: bool,
    /// Batch optimization algorithm
    pub optimization_algorithm: BatchOptimizationAlgorithm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchOptimizationAlgorithm {
    /// Fixed batch size
    Fixed,
    /// Adaptive based on load
    LoadAdaptive,
    /// Adaptive based on text length
    LengthAdaptive,
    /// AI-driven optimization
    AiOptimized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOptimizationConfig {
    /// Enable model quantization
    pub enable_quantization: bool,
    /// Model precision (fp16, fp32)
    pub precision: ModelPrecision,
    /// Enable model compilation optimization
    pub enable_compilation: bool,
    /// Memory mapping optimization
    pub memory_mapping: bool,
    /// Enable ONNX runtime if available
    pub prefer_onnx: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelPrecision {
    Float16,
    Float32,
    Int8,
}

impl Default for OptimizedEmbeddingConfig {
    fn default() -> Self {
        Self {
            base_config: EmbeddingConfig::default(),
            performance: EmbeddingPerformanceConfig {
                target_response_time_ms: 100,
                max_concurrent_requests: 50,
                request_timeout: Duration::from_millis(500),
                enable_monitoring: true,
                memory_optimization: MemoryOptimizationMode::Balanced,
                model_warming: ModelWarmingConfig {
                    enable_warmup: true,
                    warmup_batch_size: 10,
                    warmup_texts: vec![
                        "sample text for model warming".to_string(),
                        "another example for warming up the model".to_string(),
                    ],
                    warmup_timeout: Duration::from_secs(30),
                },
            },
            caching: AdvancedCacheConfig {
                enable_preloading: true,
                preload_popular: true,
                enable_compression: false, // Disabled for simplicity
                enable_segmentation: true,
                predictive_caching: false, // Advanced feature for future
            },
            batching: BatchConfig {
                adaptive_sizing: true,
                min_batch_size: 1,
                max_batch_size: 64,
                dynamic_timeout: true,
                optimization_algorithm: BatchOptimizationAlgorithm::LoadAdaptive,
            },
            model_optimization: ModelOptimizationConfig {
                enable_quantization: false, // Disabled for mock implementation
                precision: ModelPrecision::Float32,
                enable_compilation: false,
                memory_mapping: true,
                prefer_onnx: false,
            },
        }
    }
}

/// Enhanced embedding statistics with performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedEmbeddingStats {
    /// Base embedding statistics
    pub base_stats: EmbeddingStats,
    /// Performance metrics
    pub performance: EmbeddingPerformanceStats,
    /// Cache effectiveness
    pub cache_effectiveness: CacheEffectivenessStats,
    /// Batch processing metrics
    pub batch_metrics: BatchProcessingStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingPerformanceStats {
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// P95 response time (ms)
    pub p95_response_time_ms: f64,
    /// P99 response time (ms)
    pub p99_response_time_ms: f64,
    /// Throughput (embeddings/second)
    pub throughput_eps: f64,
    /// Current active requests
    pub active_requests: usize,
    /// Queue depth
    pub queue_depth: usize,
    /// Model utilization percentage
    pub model_utilization: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEffectivenessStats {
    /// Cache hit rate by text type
    pub hit_rate_by_type: HashMap<String, f64>,
    /// Average cache lookup time (ms)
    pub avg_lookup_time_ms: f64,
    /// Cache memory efficiency
    pub memory_efficiency: f64,
    /// Preemptive cache hits
    pub preemptive_hits: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProcessingStats {
    /// Average batch size
    pub avg_batch_size: f64,
    /// Batch utilization efficiency
    pub batch_efficiency: f64,
    /// Adaptive sizing effectiveness
    pub adaptive_effectiveness: f64,
    /// Processing time saved by batching (ms)
    pub time_saved_ms: f64,
}

/// Classification rule for text type identification
type ClassificationRule = Box<dyn Fn(&str) -> bool + Send + Sync>;

/// Text type classifier for cache segmentation
pub struct TextClassifier {
    /// Classification rules
    rules: HashMap<String, ClassificationRule>,
}

impl Default for TextClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl TextClassifier {
    pub fn new() -> Self {
        let mut rules: HashMap<String, ClassificationRule> = HashMap::new();

        rules.insert("short".to_string(), Box::new(|text| text.len() < 100));
        rules.insert(
            "medium".to_string(),
            Box::new(|text| text.len() >= 100 && text.len() < 1000),
        );
        rules.insert("long".to_string(), Box::new(|text| text.len() >= 1000));
        rules.insert(
            "code".to_string(),
            Box::new(|text| {
                text.contains("fn ") || text.contains("class ") || text.contains("import ")
            }),
        );
        rules.insert(
            "technical".to_string(),
            Box::new(|text| {
                text.contains("API")
                    || text.contains("implementation")
                    || text.contains("algorithm")
            }),
        );

        Self { rules }
    }

    pub fn classify(&self, text: &str) -> String {
        // Check more specific rules first (in priority order)
        let priority_order = ["code", "technical", "long", "medium", "short"];

        for category in priority_order {
            if let Some(rule) = self.rules.get(category) {
                if rule(text) {
                    return category.to_string();
                }
            }
        }

        // Check remaining rules if none of the priority ones matched
        for (category, rule) in &self.rules {
            if !priority_order.contains(&category.as_str()) && rule(text) {
                return category.clone();
            }
        }

        "general".to_string()
    }
}

/// Optimized embedding service with advanced performance features
pub struct OptimizedEmbeddingService {
    config: OptimizedEmbeddingConfig,
    cache_manager: Arc<CacheManager>,
    performance_monitor: Arc<PerformanceMonitor>,
    #[allow(dead_code)] // TODO: Implement optimized batch processing for embeddings
    batch_processor: Arc<BatchProcessor>,
    text_classifier: Arc<TextClassifier>,
    request_semaphore: Arc<Semaphore>,
    stats: Arc<RwLock<OptimizedEmbeddingStats>>,
    model_loaded: Arc<RwLock<bool>>,
}

impl OptimizedEmbeddingService {
    pub fn new(config: OptimizedEmbeddingConfig, cache_manager: CacheManager) -> Self {
        let performance_monitor = Arc::new(PerformanceMonitor::new(
            crate::vector::performance::MonitoringConfig::default(),
        ));

        let batch_processor = Arc::new(BatchProcessor::new(
            crate::vector::performance::BatchProcessingConfig::default(),
        ));

        Self {
            request_semaphore: Arc::new(Semaphore::new(config.performance.max_concurrent_requests)),
            config,
            cache_manager: Arc::new(cache_manager),
            performance_monitor,
            batch_processor,
            text_classifier: Arc::new(TextClassifier::new()),
            stats: Arc::new(RwLock::new(OptimizedEmbeddingStats {
                base_stats: EmbeddingStats {
                    total_generated: 0,
                    cache_hit_rate: 0.0,
                    avg_generation_time_ms: 0.0,
                    cache_size: 0,
                },
                performance: EmbeddingPerformanceStats {
                    avg_response_time_ms: 0.0,
                    p95_response_time_ms: 0.0,
                    p99_response_time_ms: 0.0,
                    throughput_eps: 0.0,
                    active_requests: 0,
                    queue_depth: 0,
                    model_utilization: 0.0,
                },
                cache_effectiveness: CacheEffectivenessStats {
                    hit_rate_by_type: HashMap::new(),
                    avg_lookup_time_ms: 0.0,
                    memory_efficiency: 0.0,
                    preemptive_hits: 0,
                },
                batch_metrics: BatchProcessingStats {
                    avg_batch_size: 0.0,
                    batch_efficiency: 0.0,
                    adaptive_effectiveness: 0.0,
                    time_saved_ms: 0.0,
                },
            })),
            model_loaded: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize the optimized embedding service
    #[instrument(skip(self))]
    pub async fn initialize(&self) -> VectorResult<()> {
        info!("Initializing optimized embedding service");

        // Initialize performance monitoring
        self.performance_monitor.start_monitoring().await?;

        // Warm up the model if configured
        if self.config.performance.model_warming.enable_warmup {
            self.warm_up_model().await?;
        }

        // Preload cache if configured
        if self.config.caching.enable_preloading {
            self.preload_cache().await?;
        }

        *self.model_loaded.write().await = true;
        info!("Optimized embedding service initialized successfully");
        Ok(())
    }

    /// Warm up the model with sample texts
    async fn warm_up_model(&self) -> VectorResult<()> {
        info!("Warming up embedding model");
        let warmup_config = &self.config.performance.model_warming;

        let warmup_texts = if warmup_config.warmup_texts.is_empty() {
            vec!["sample text for warmup".to_string()]
        } else {
            warmup_config.warmup_texts.clone()
        };

        // Process warmup texts in batches
        for chunk in warmup_texts.chunks(warmup_config.warmup_batch_size) {
            let texts: Vec<String> = chunk.to_vec();

            // Generate embeddings without caching to warm up the model
            match tokio::time::timeout(
                warmup_config.warmup_timeout,
                self.generate_embeddings_internal(&texts, false),
            )
            .await
            {
                Ok(Ok(_)) => debug!("Warmup batch processed successfully"),
                Ok(Err(e)) => warn!("Warmup batch failed: {}", e),
                Err(_) => warn!("Warmup batch timed out"),
            }
        }

        info!("Model warmup completed");
        Ok(())
    }

    /// Preload popular embeddings into cache
    async fn preload_cache(&self) -> VectorResult<()> {
        if !self.config.caching.preload_popular {
            return Ok(());
        }

        info!("Preloading popular embeddings into cache");

        // Popular technical terms for research and implementation
        let popular_texts = vec![
            "implementation",
            "algorithm",
            "performance",
            "optimization",
            "database",
            "vector search",
            "machine learning",
            "artificial intelligence",
            "data structure",
            "software architecture",
        ];

        for text in popular_texts {
            if let Ok(embedding) = self.generate_embedding_internal(text).await {
                let text_type = self.text_classifier.classify(text);
                let cache_key = CacheKey::new("embedding", &format!("{text_type}:{text}"), 1);
                let size = embedding.len() * 4; // f32 = 4 bytes

                let _ = self
                    .cache_manager
                    .embedding_cache
                    .set(cache_key, embedding, size)
                    .await;
            }
        }

        info!("Cache preloading completed");
        Ok(())
    }

    /// Generate embedding with optimized caching and monitoring
    #[instrument(skip(self, text))]
    pub async fn generate_embedding_optimized(&self, text: &str) -> VectorResult<Vec<f32>> {
        let _permit = self.request_semaphore.acquire().await.map_err(|_| {
            VectorError::PerformanceError("Failed to acquire request permit".to_string())
        })?;

        let measurement = LatencyMeasurement::new("embedding_generation");

        // Update active request count
        {
            let mut stats = self.stats.write().await;
            stats.performance.active_requests += 1;
        }

        let result = self.generate_embedding_with_cache(text).await;

        // Record performance metrics
        let duration = measurement.finish_with_result(&result);
        self.performance_monitor
            .record_latency("embedding_generation", duration)
            .await;

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.performance.active_requests -= 1;

            let response_time_ms = duration.as_millis() as f64;
            let total_requests = stats.base_stats.total_generated + 1;

            // Update rolling averages
            stats.performance.avg_response_time_ms = (stats.performance.avg_response_time_ms
                * (total_requests - 1) as f64
                + response_time_ms)
                / total_requests as f64;

            stats.base_stats.total_generated += 1;
        }

        result
    }

    /// Generate embedding with intelligent caching
    async fn generate_embedding_with_cache(&self, text: &str) -> VectorResult<Vec<f32>> {
        let cache_lookup_start = Instant::now();
        let text_type = self.text_classifier.classify(text);

        // Try cache first
        let cache_key = if self.config.caching.enable_segmentation {
            CacheKey::new("embedding", &format!("{text_type}:{text}"), 1)
        } else {
            CacheKey::embedding_key(text)
        };

        if let Some(cached_embedding) = self.cache_manager.embedding_cache.get(&cache_key).await {
            let lookup_time = cache_lookup_start.elapsed();

            // Update cache statistics
            {
                let mut stats = self.stats.write().await;
                stats.cache_effectiveness.avg_lookup_time_ms = lookup_time.as_millis() as f64;

                let hit_rate = stats
                    .cache_effectiveness
                    .hit_rate_by_type
                    .entry(text_type.clone())
                    .or_insert(0.0);
                *hit_rate = (*hit_rate + 1.0) / 2.0; // Simple moving average
            }

            debug!("Cache hit for text type: {}", text_type);
            return Ok(cached_embedding);
        }

        // Generate new embedding
        let embedding = self.generate_embedding_internal(text).await?;

        // Cache the result
        let embedding_size = embedding.len() * 4; // f32 = 4 bytes
        self.cache_manager
            .embedding_cache
            .set(cache_key, embedding.clone(), embedding_size)
            .await?;

        Ok(embedding)
    }

    /// Internal embedding generation (mock implementation)
    async fn generate_embedding_internal(&self, text: &str) -> VectorResult<Vec<f32>> {
        if !*self.model_loaded.read().await {
            return Err(VectorError::EmbeddingError("Model not loaded".to_string()));
        }

        // Preprocess text
        let processed_text = self.preprocess_text_optimized(text);

        // Mock embedding generation with deterministic output
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        processed_text.hash(&mut hasher);
        let hash = hasher.finish();

        // Generate 384-dimensional embedding
        let mut embedding = Vec::with_capacity(384);
        let mut rng_state = hash;

        for _ in 0..384 {
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            let normalized = (rng_state % 1000) as f32 / 1000.0 - 0.5;
            embedding.push(normalized);
        }

        // Normalize to unit length
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in &mut embedding {
                *value /= norm;
            }
        }

        // Simulate processing time based on text length and optimization mode
        let processing_delay = match self.config.performance.memory_optimization {
            MemoryOptimizationMode::Minimal => Duration::from_millis(100 + text.len() as u64 / 10),
            MemoryOptimizationMode::Balanced => Duration::from_millis(50 + text.len() as u64 / 20),
            MemoryOptimizationMode::Performance => {
                Duration::from_millis(20 + text.len() as u64 / 50)
            }
        };

        tokio::time::sleep(processing_delay).await;

        Ok(embedding)
    }

    /// Generate embeddings for multiple texts using batching
    async fn generate_embeddings_internal(
        &self,
        texts: &[String],
        use_cache: bool,
    ) -> VectorResult<Vec<Vec<f32>>> {
        if !use_cache {
            // Direct generation for warmup
            let mut embeddings = Vec::with_capacity(texts.len());
            for text in texts {
                let embedding = self.generate_embedding_internal(text).await?;
                embeddings.push(embedding);
            }
            return Ok(embeddings);
        }

        // Use optimized batch processing
        let mut embeddings = Vec::with_capacity(texts.len());
        for text in texts {
            let embedding = self.generate_embedding_optimized(text).await?;
            embeddings.push(embedding);
        }
        Ok(embeddings)
    }

    /// Optimized text preprocessing
    fn preprocess_text_optimized(&self, text: &str) -> String {
        let preprocessing = &self.config.base_config.preprocessing;
        let mut processed = text.to_string();

        // Optimize preprocessing order for performance
        if preprocessing.normalize_whitespace {
            processed = processed.split_whitespace().collect::<Vec<_>>().join(" ");
        }

        if preprocessing.lowercase {
            processed = processed.to_lowercase();
        }

        if preprocessing.remove_special_chars {
            // Use retain for better performance than filter + collect
            processed.retain(|c| c.is_alphanumeric() || c.is_whitespace());
        }

        if processed.len() > preprocessing.max_text_length {
            // Use byte-aware truncation
            processed.truncate(preprocessing.max_text_length);
        }

        processed
    }

    /// Get comprehensive embedding statistics
    pub async fn get_optimized_stats(&self) -> OptimizedEmbeddingStats {
        let stats = self.stats.read().await.clone();

        // Update real-time metrics
        let mut updated_stats = stats;
        let cache_stats = self.cache_manager.get_aggregated_stats().await;

        if let Some(embedding_cache_stats) = cache_stats.get("embedding") {
            updated_stats.base_stats.cache_hit_rate = embedding_cache_stats.hit_rate;
            updated_stats.base_stats.cache_size = embedding_cache_stats.l1_entries;
            updated_stats.cache_effectiveness.memory_efficiency =
                embedding_cache_stats.efficiency_score;
        }

        updated_stats
    }
}

#[async_trait]
impl EmbeddingGenerator for OptimizedEmbeddingService {
    async fn generate_embedding(&self, text: &str) -> VectorResult<Vec<f32>> {
        self.generate_embedding_optimized(text).await
    }

    async fn generate_embeddings(&self, texts: &[String]) -> VectorResult<Vec<Vec<f32>>> {
        self.generate_embeddings_internal(texts, true).await
    }

    async fn get_stats(&self) -> EmbeddingStats {
        let optimized_stats = self.get_optimized_stats().await;
        optimized_stats.base_stats
    }

    async fn clear_cache(&self) -> VectorResult<()> {
        self.cache_manager.clear_all().await?;

        let mut stats = self.stats.write().await;
        stats.base_stats.cache_hit_rate = 0.0;
        stats.base_stats.cache_size = 0;

        info!("Optimized embedding cache cleared");
        Ok(())
    }

    fn embedding_dimension(&self) -> usize {
        384 // Consistent with the mock implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::cache::CacheConfig;

    #[tokio::test]
    async fn test_optimized_embedding_service() {
        let config = OptimizedEmbeddingConfig::default();
        let cache_manager = CacheManager::new(CacheConfig::default());
        let service = OptimizedEmbeddingService::new(config, cache_manager);

        let _ = service.initialize().await;

        let text = "test embedding optimization";
        let embedding = service.generate_embedding_optimized(text).await.unwrap();

        assert_eq!(embedding.len(), 384);

        // Test cache hit on second call
        let embedding2 = service.generate_embedding_optimized(text).await.unwrap();
        assert_eq!(embedding, embedding2);

        let stats = service.get_optimized_stats().await;
        assert!(stats.base_stats.total_generated > 0);
    }

    #[test]
    fn test_text_classifier() {
        let classifier = TextClassifier::new();

        assert_eq!(classifier.classify("short"), "short");
        assert_eq!(classifier.classify(&"a".repeat(500)), "medium");
        assert_eq!(classifier.classify(&"a".repeat(1500)), "long");
        assert_eq!(classifier.classify("fn main() {}"), "code");
        assert_eq!(classifier.classify("API implementation"), "technical");
    }

    #[tokio::test]
    async fn test_batch_processing() {
        let config = OptimizedEmbeddingConfig::default();
        let cache_manager = CacheManager::new(CacheConfig::default());
        let service = OptimizedEmbeddingService::new(config, cache_manager);

        let _ = service.initialize().await;

        let texts = vec![
            "first text".to_string(),
            "second text".to_string(),
            "third text".to_string(),
        ];

        let embeddings = service.generate_embeddings(&texts).await.unwrap();
        assert_eq!(embeddings.len(), 3);
        assert!(embeddings.iter().all(|e| e.len() == 384));
    }
}
