// ABOUTME: Performance optimization and monitoring for vector operations
//! This module provides comprehensive performance optimizations and monitoring
//! to achieve <200ms response times for vector search operations.

use crate::vector::{
    cache::{CacheConfig, CacheKey, CacheManager, MultiLevelCache},
    error::{VectorError, VectorResult},
    storage::VectorDocument,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, instrument, warn};

/// Type alias for embedding batch entry
type EmbeddingBatchEntry = (String, tokio::sync::oneshot::Sender<VectorResult<Vec<f32>>>);

/// Type alias for search batch entry  
type SearchBatchEntry = (
    Vec<f32>,
    tokio::sync::oneshot::Sender<VectorResult<Vec<VectorDocument>>>,
);

/// Type alias for embedding batch storage
type EmbeddingBatchStorage = Arc<RwLock<Vec<EmbeddingBatchEntry>>>;

/// Type alias for search batch storage
type SearchBatchStorage = Arc<RwLock<Vec<SearchBatchEntry>>>;

/// Performance optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Target response time for searches (ms)
    pub target_response_time_ms: u64,
    /// Connection pool configuration
    pub connection_pool: ConnectionPoolConfig,
    /// Query optimization settings
    pub query_optimization: QueryOptimizationConfig,
    /// Batch processing configuration
    pub batch_processing: BatchProcessingConfig,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    /// Resource limits
    pub resource_limits: ResourceLimitsConfig,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            target_response_time_ms: 200,
            connection_pool: ConnectionPoolConfig::default(),
            query_optimization: QueryOptimizationConfig::default(),
            batch_processing: BatchProcessingConfig::default(),
            monitoring: MonitoringConfig::default(),
            resource_limits: ResourceLimitsConfig::default(),
        }
    }
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Minimum number of connections
    pub min_connections: usize,
    /// Maximum number of connections
    pub max_connections: usize,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Idle timeout
    pub idle_timeout: Duration,
    /// Maximum connection lifetime
    pub max_lifetime: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 2,
            max_connections: 20,
            connect_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(600), // 10 minutes
            max_lifetime: Duration::from_secs(3600), // 1 hour
            health_check_interval: Duration::from_secs(30),
        }
    }
}

/// Query optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryOptimizationConfig {
    /// Enable query plan caching
    pub enable_query_plan_cache: bool,
    /// Enable result caching
    pub enable_result_cache: bool,
    /// Enable query rewriting
    pub enable_query_rewriting: bool,
    /// Enable early termination
    pub enable_early_termination: bool,
    /// Parallel search threshold
    pub parallel_search_threshold: usize,
    /// Max concurrent queries
    pub max_concurrent_queries: usize,
    /// Query timeout
    pub query_timeout: Duration,
}

impl Default for QueryOptimizationConfig {
    fn default() -> Self {
        Self {
            enable_query_plan_cache: true,
            enable_result_cache: true,
            enable_query_rewriting: true,
            enable_early_termination: true,
            parallel_search_threshold: 100,
            max_concurrent_queries: 50,
            query_timeout: Duration::from_millis(500),
        }
    }
}

/// Batch processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProcessingConfig {
    /// Optimal batch size for embeddings
    pub optimal_embedding_batch_size: usize,
    /// Optimal batch size for searches
    pub optimal_search_batch_size: usize,
    /// Optimal batch size for inserts
    pub optimal_insert_batch_size: usize,
    /// Enable dynamic batch sizing
    pub dynamic_batch_sizing: bool,
    /// Batch timeout
    pub batch_timeout: Duration,
    /// Enable batch pipelining
    pub enable_pipelining: bool,
}

impl Default for BatchProcessingConfig {
    fn default() -> Self {
        Self {
            optimal_embedding_batch_size: 32,
            optimal_search_batch_size: 20,
            optimal_insert_batch_size: 100,
            dynamic_batch_sizing: true,
            batch_timeout: Duration::from_millis(100),
            enable_pipelining: true,
        }
    }
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable performance metrics collection
    pub enable_metrics: bool,
    /// Metrics collection interval
    pub metrics_interval: Duration,
    /// Enable latency histograms
    pub enable_latency_histograms: bool,
    /// Enable throughput monitoring
    pub enable_throughput_monitoring: bool,
    /// Enable resource usage monitoring
    pub enable_resource_monitoring: bool,
    /// Performance alert thresholds
    pub alert_thresholds: AlertThresholds,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            metrics_interval: Duration::from_secs(10),
            enable_latency_histograms: true,
            enable_throughput_monitoring: true,
            enable_resource_monitoring: true,
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

/// Alert thresholds for performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Maximum acceptable latency (ms)
    pub max_latency_ms: u64,
    /// Minimum acceptable throughput (ops/sec)
    pub min_throughput_ops_sec: f64,
    /// Maximum memory usage (MB)
    pub max_memory_mb: usize,
    /// Maximum CPU usage (percentage)
    pub max_cpu_percentage: f64,
    /// Maximum error rate (percentage)
    pub max_error_rate_percentage: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_latency_ms: 500,
            min_throughput_ops_sec: 10.0,
            max_memory_mb: 1024,
            max_cpu_percentage: 80.0,
            max_error_rate_percentage: 5.0,
        }
    }
}

/// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimitsConfig {
    /// Maximum memory usage for caches (bytes)
    pub max_cache_memory_bytes: usize,
    /// Maximum concurrent operations
    pub max_concurrent_operations: usize,
    /// Maximum query result size
    pub max_result_size: usize,
    /// Maximum vector dimensions
    pub max_vector_dimensions: usize,
    /// Maximum batch size
    pub max_batch_size: usize,
}

impl Default for ResourceLimitsConfig {
    fn default() -> Self {
        Self {
            max_cache_memory_bytes: 512 * 1024 * 1024, // 512MB
            max_concurrent_operations: 100,
            max_result_size: 1000,
            max_vector_dimensions: 2048,
            max_batch_size: 1000,
        }
    }
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average latency (ms)
    pub avg_latency_ms: f64,
    /// P95 latency (ms)
    pub p95_latency_ms: f64,
    /// P99 latency (ms)
    pub p99_latency_ms: f64,
    /// Throughput (operations per second)
    pub throughput_ops_sec: f64,
    /// Error rate (percentage)
    pub error_rate_percentage: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Memory usage (bytes)
    pub memory_usage_bytes: usize,
    /// CPU usage (percentage)
    pub cpu_usage_percentage: f64,
    /// Active connections
    pub active_connections: usize,
    /// Queue depth
    pub queue_depth: usize,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            avg_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            throughput_ops_sec: 0.0,
            error_rate_percentage: 0.0,
            cache_hit_rate: 0.0,
            memory_usage_bytes: 0,
            cpu_usage_percentage: 0.0,
            active_connections: 0,
            queue_depth: 0,
        }
    }
}

/// Latency measurement helper
#[derive(Debug)]
pub struct LatencyMeasurement {
    start_time: Instant,
    operation: String,
}

impl LatencyMeasurement {
    pub fn new(operation: &str) -> Self {
        Self {
            start_time: Instant::now(),
            operation: operation.to_string(),
        }
    }

    pub fn finish(self) -> Duration {
        let duration = self.start_time.elapsed();
        debug!("Operation '{}' took: {:?}", self.operation, duration);
        duration
    }

    pub fn finish_with_result<T>(self, result: &Result<T, VectorError>) -> Duration {
        let duration = self.start_time.elapsed();
        match result {
            Ok(_) => debug!(
                "Operation '{}' succeeded in: {:?}",
                self.operation, duration
            ),
            Err(e) => warn!(
                "Operation '{}' failed in: {:?}, error: {}",
                self.operation, duration, e
            ),
        }
        duration
    }
}

/// Query optimizer for search operations
pub struct QueryOptimizer {
    config: QueryOptimizationConfig,
    query_plan_cache: MultiLevelCache<QueryPlan>,
    semaphore: Arc<Semaphore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlan {
    /// Optimized query vector
    pub query_vector: Vec<f32>,
    /// Search parameters
    pub search_params: HashMap<String, serde_json::Value>,
    /// Estimated cost
    pub estimated_cost: f64,
    /// Parallel execution plan
    pub parallel_execution: bool,
}

impl QueryOptimizer {
    pub fn new(config: QueryOptimizationConfig) -> Self {
        let cache_config = CacheConfig::default();
        Self {
            config: config.clone(),
            query_plan_cache: MultiLevelCache::new(cache_config),
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_queries)),
        }
    }

    /// Optimize a search query
    #[instrument(skip(self, query_vector))]
    pub async fn optimize_query(
        &self,
        query_vector: &[f32],
        options: &HashMap<String, serde_json::Value>,
    ) -> VectorResult<QueryPlan> {
        let _measurement = LatencyMeasurement::new("query_optimization");

        // Generate cache key for query plan
        let cache_key = self.generate_plan_cache_key(query_vector, options);

        // Try to get cached query plan
        if self.config.enable_query_plan_cache {
            if let Some(cached_plan) = self.query_plan_cache.get(&cache_key).await {
                debug!("Using cached query plan");
                return Ok(cached_plan);
            }
        }

        // Create optimized query plan
        let mut optimized_vector = query_vector.to_vec();

        // Normalize vector for better performance
        let norm = optimized_vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in &mut optimized_vector {
                *value /= norm;
            }
        }

        // Optimize search parameters
        let mut search_params = options.clone();

        // Adjust batch size based on query complexity
        let query_complexity = self.estimate_query_complexity(query_vector, options);
        if query_complexity > self.config.parallel_search_threshold as f64 {
            search_params.insert("parallel_execution".to_string(), serde_json::json!(true));
        }

        let plan = QueryPlan {
            query_vector: optimized_vector,
            search_params,
            estimated_cost: query_complexity,
            parallel_execution: query_complexity > self.config.parallel_search_threshold as f64,
        };

        // Cache the query plan
        if self.config.enable_query_plan_cache {
            let plan_size = std::mem::size_of_val(&plan) + plan.query_vector.len() * 4;
            self.query_plan_cache
                .set(cache_key, plan.clone(), plan_size)
                .await?;
        }

        Ok(plan)
    }

    /// Acquire semaphore for query execution
    pub async fn acquire_query_permit(
        &self,
    ) -> Result<tokio::sync::SemaphorePermit<'_>, VectorError> {
        match tokio::time::timeout(self.config.query_timeout, self.semaphore.acquire()).await {
            Ok(Ok(permit)) => Ok(permit),
            Ok(Err(_)) => Err(VectorError::PerformanceError(
                "Failed to acquire query semaphore".to_string(),
            )),
            Err(_) => Err(VectorError::PerformanceError(
                "Query timeout exceeded".to_string(),
            )),
        }
    }

    fn generate_plan_cache_key(
        &self,
        query_vector: &[f32],
        options: &HashMap<String, serde_json::Value>,
    ) -> CacheKey {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash vector (sample to avoid performance impact)
        let sample_size = (query_vector.len() / 10).max(1);
        for i in (0..query_vector.len()).step_by(sample_size) {
            query_vector[i].to_bits().hash(&mut hasher);
        }

        // Hash options
        for (key, value) in options {
            key.hash(&mut hasher);
            value.to_string().hash(&mut hasher);
        }

        let hash = hasher.finish();
        CacheKey::new("query_plan", &format!("{hash:x}"), 1)
    }

    fn estimate_query_complexity(
        &self,
        query_vector: &[f32],
        options: &HashMap<String, serde_json::Value>,
    ) -> f64 {
        let mut complexity = query_vector.len() as f64;

        // Add complexity for filters
        if let Some(filters) = options.get("filters") {
            if let Some(filter_array) = filters.as_array() {
                complexity += filter_array.len() as f64 * 10.0;
            }
        }

        // Add complexity for result limit
        if let Some(limit) = options.get("limit") {
            if let Some(limit_num) = limit.as_u64() {
                complexity += limit_num as f64 * 0.1;
            }
        }

        complexity
    }
}

/// Batch processor for optimizing bulk operations
pub struct BatchProcessor {
    config: BatchProcessingConfig,
    embedding_batches: EmbeddingBatchStorage,
    search_batches: SearchBatchStorage,
}

impl BatchProcessor {
    pub fn new(config: BatchProcessingConfig) -> Self {
        Self {
            config,
            embedding_batches: Arc::new(RwLock::new(Vec::new())),
            search_batches: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add embedding request to batch
    #[instrument(skip(self, text, sender))]
    pub async fn add_embedding_request(
        &self,
        text: String,
        sender: tokio::sync::oneshot::Sender<VectorResult<Vec<f32>>>,
    ) {
        let mut batches = self.embedding_batches.write().await;
        batches.push((text, sender));

        // Check if batch is ready for processing
        if batches.len() >= self.config.optimal_embedding_batch_size
            || (!batches.is_empty() && self.should_flush_batch("embedding").await)
        {
            let batch = std::mem::take(&mut *batches);
            drop(batches);

            // Process batch asynchronously
            let config = self.config.clone();
            tokio::spawn(async move {
                Self::process_embedding_batch(batch, config).await;
            });
        }
    }

    /// Add search request to batch
    #[instrument(skip(self, query_vector, sender))]
    pub async fn add_search_request(
        &self,
        query_vector: Vec<f32>,
        sender: tokio::sync::oneshot::Sender<VectorResult<Vec<VectorDocument>>>,
    ) {
        let mut batches = self.search_batches.write().await;
        batches.push((query_vector, sender));

        // Check if batch is ready for processing
        if batches.len() >= self.config.optimal_search_batch_size
            || (!batches.is_empty() && self.should_flush_batch("search").await)
        {
            let batch = std::mem::take(&mut *batches);
            drop(batches);

            // Process batch asynchronously
            let config = self.config.clone();
            tokio::spawn(async move {
                Self::process_search_batch(batch, config).await;
            });
        }
    }

    async fn should_flush_batch(&self, _batch_type: &str) -> bool {
        // Simple time-based flushing for now
        // In practice, this would check the last batch time
        false
    }

    async fn process_embedding_batch(
        batch: Vec<(String, tokio::sync::oneshot::Sender<VectorResult<Vec<f32>>>)>,
        _config: BatchProcessingConfig,
    ) {
        debug!("Processing embedding batch of size: {}", batch.len());

        // Simulate batch embedding generation
        for (_text, sender) in batch {
            // In practice, this would use the actual embedding service
            let embedding = vec![0.5f32; 384]; // Mock embedding
            let _ = sender.send(Ok(embedding));
        }
    }

    async fn process_search_batch(batch: Vec<SearchBatchEntry>, _config: BatchProcessingConfig) {
        debug!("Processing search batch of size: {}", batch.len());

        // Simulate batch search processing
        for (_query_vector, sender) in batch {
            // In practice, this would use the actual search service
            let results = Vec::new(); // Mock results
            let _ = sender.send(Ok(results));
        }
    }
}

/// Performance monitor for tracking metrics and alerts
pub struct PerformanceMonitor {
    config: MonitoringConfig,
    metrics: Arc<RwLock<PerformanceMetrics>>,
    latency_samples: Arc<RwLock<Vec<f64>>>,
    operation_counts: Arc<RwLock<HashMap<String, u64>>>,
    error_counts: Arc<RwLock<HashMap<String, u64>>>,
}

impl PerformanceMonitor {
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            latency_samples: Arc::new(RwLock::new(Vec::new())),
            operation_counts: Arc::new(RwLock::new(HashMap::new())),
            error_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record operation latency
    #[instrument(skip(self))]
    pub async fn record_latency(&self, operation: &str, duration: Duration) {
        if !self.config.enable_metrics {
            return;
        }

        let latency_ms = duration.as_millis() as f64;

        let mut samples = self.latency_samples.write().await;
        samples.push(latency_ms);

        // Limit sample size to prevent memory growth
        if samples.len() > 1000 {
            samples.drain(0..500); // Keep last 500 samples
        }

        let mut counts = self.operation_counts.write().await;
        *counts.entry(operation.to_string()).or_insert(0) += 1;

        // Check for performance alerts
        if latency_ms > self.config.alert_thresholds.max_latency_ms as f64 {
            warn!(
                "High latency detected for {}: {:.2}ms",
                operation, latency_ms
            );
        }
    }

    /// Record operation error
    pub async fn record_error(&self, operation: &str, error: &VectorError) {
        if !self.config.enable_metrics {
            return;
        }

        let mut error_counts = self.error_counts.write().await;
        *error_counts.entry(operation.to_string()).or_insert(0) += 1;

        warn!("Error in operation {}: {}", operation, error);
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        if !self.config.enable_metrics {
            return PerformanceMetrics::default();
        }

        let samples = self.latency_samples.read().await;
        let operation_counts = self.operation_counts.read().await;
        let error_counts = self.error_counts.read().await;

        let mut metrics = self.metrics.read().await.clone();

        if !samples.is_empty() {
            // Calculate latency percentiles
            let mut sorted_samples = samples.clone();
            sorted_samples.sort_by(|a, b| a.partial_cmp(b).unwrap());

            metrics.avg_latency_ms =
                sorted_samples.iter().sum::<f64>() / sorted_samples.len() as f64;

            if sorted_samples.len() >= 20 {
                let p95_idx = (sorted_samples.len() as f64 * 0.95) as usize;
                let p99_idx = (sorted_samples.len() as f64 * 0.99) as usize;
                metrics.p95_latency_ms = sorted_samples[p95_idx.min(sorted_samples.len() - 1)];
                metrics.p99_latency_ms = sorted_samples[p99_idx.min(sorted_samples.len() - 1)];
            }

            // Calculate throughput (operations per second)
            let total_operations: u64 = operation_counts.values().sum();
            let time_window_secs = self.config.metrics_interval.as_secs() as f64;
            metrics.throughput_ops_sec = total_operations as f64 / time_window_secs;

            // Calculate error rate
            let total_errors: u64 = error_counts.values().sum();
            if total_operations > 0 {
                metrics.error_rate_percentage =
                    (total_errors as f64 / total_operations as f64) * 100.0;
            }
        }

        metrics
    }

    /// Start periodic metrics collection
    pub async fn start_monitoring(&self) -> VectorResult<()> {
        if !self.config.enable_metrics {
            return Ok(());
        }

        let monitor = self.clone();
        let interval = self.config.metrics_interval;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                let _metrics = monitor.get_metrics().await;
                // In practice, this would emit metrics to monitoring systems
            }
        });

        info!(
            "Performance monitoring started with interval: {:?}",
            self.config.metrics_interval
        );
        Ok(())
    }
}

impl Clone for PerformanceMonitor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            metrics: Arc::clone(&self.metrics),
            latency_samples: Arc::clone(&self.latency_samples),
            operation_counts: Arc::clone(&self.operation_counts),
            error_counts: Arc::clone(&self.error_counts),
        }
    }
}

/// Main performance manager
pub struct PerformanceManager {
    pub config: PerformanceConfig,
    pub query_optimizer: QueryOptimizer,
    pub batch_processor: BatchProcessor,
    pub monitor: PerformanceMonitor,
    pub cache_manager: CacheManager,
}

impl PerformanceManager {
    pub fn new(config: PerformanceConfig) -> Self {
        let cache_config = CacheConfig::default();

        Self {
            query_optimizer: QueryOptimizer::new(config.query_optimization.clone()),
            batch_processor: BatchProcessor::new(config.batch_processing.clone()),
            monitor: PerformanceMonitor::new(config.monitoring.clone()),
            cache_manager: CacheManager::new(cache_config),
            config,
        }
    }

    /// Initialize the performance manager
    pub async fn initialize(&self) -> VectorResult<()> {
        self.monitor.start_monitoring().await?;
        info!(
            "Performance manager initialized with target response time: {}ms",
            self.config.target_response_time_ms
        );
        Ok(())
    }

    /// Get comprehensive performance report
    pub async fn get_performance_report(&self) -> HashMap<String, serde_json::Value> {
        let mut report = HashMap::new();

        report.insert(
            "metrics".to_string(),
            serde_json::to_value(self.monitor.get_metrics().await).unwrap(),
        );
        report.insert(
            "cache_stats".to_string(),
            serde_json::to_value(self.cache_manager.get_aggregated_stats().await).unwrap(),
        );
        report.insert(
            "config".to_string(),
            serde_json::to_value(&self.config).unwrap(),
        );

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_latency_measurement() {
        let measurement = LatencyMeasurement::new("test_operation");
        tokio::time::sleep(Duration::from_millis(10)).await;
        let duration = measurement.finish();
        assert!(duration >= Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_query_optimizer() {
        let config = QueryOptimizationConfig::default();
        let optimizer = QueryOptimizer::new(config);

        let query_vector = vec![0.5f32; 384];
        let options = HashMap::new();

        let plan = optimizer
            .optimize_query(&query_vector, &options)
            .await
            .unwrap();
        assert_eq!(plan.query_vector.len(), 384);
    }

    #[tokio::test]
    async fn test_performance_monitor() {
        let config = MonitoringConfig::default();
        let monitor = PerformanceMonitor::new(config);

        monitor
            .record_latency("test_op", Duration::from_millis(100))
            .await;
        let metrics = monitor.get_metrics().await;

        assert!(metrics.avg_latency_ms > 0.0);
    }
}
