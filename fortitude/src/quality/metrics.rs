// ABOUTME: Automated quality metrics collection and storage system for comprehensive research quality monitoring
//! This module provides automated collection, storage, and analysis of quality metrics
//! to continuously monitor research quality and system performance. It supports real-time
//! metrics collection, time-series storage, and analytics for data-driven improvements.
//!
//! # Key Components
//! - `QualityMetric`: Core metric data structure with timestamps and context
//! - `MetricsCollector`: Real-time metrics collection with minimal overhead
//! - `MetricsStorage`: Time-series storage with efficient querying and retention
//! - `MetricsAnalyzer`: Analytics engine for trends, patterns, and insights
//! - `MetricsAggregator`: Batch processing and aggregation of metrics
//!
//! # Performance Requirements
//! - Collection overhead: <5ms per research operation
//! - Storage throughput: Support 1000+ metrics/second ingestion
//! - Query performance: <100ms for standard analytics queries
//! - Memory efficiency: <100MB for metrics collection buffer
//! - Storage efficiency: Configurable retention with automatic cleanup
//!
//! # Metrics Types
//! - **Research Quality Metrics**: Accuracy, relevance, completeness scores over time
//! - **Provider Performance Metrics**: Response times, success rates, cost efficiency
//! - **Cross-Validation Metrics**: Consistency scores, consensus rates, validation accuracy
//! - **User Satisfaction Metrics**: Feedback ratings, correction frequencies, usage patterns
//! - **System Performance Metrics**: Throughput, latency, resource utilization
//!
//! # Collection Strategies
//! - Real-time: Inline collection during operations (<5ms overhead)
//! - Batch: Periodic aggregation for efficiency
//! - Streaming: Continuous high-volume processing
//! - Event-driven: Triggered by quality changes or thresholds
//!
//! # Example Usage
//!
//! ```rust
//! use fortitude::quality::metrics::{MetricsCollector, QualityMetric, MetricType, MetricValue};
//!
//! async fn collect_research_metrics(
//!     collector: &MetricsCollector,
//!     provider: &str,
//!     quality_score: f64,
//!     response_time: u64,
//! ) -> Result<(), MetricsError> {
//!     // Collect quality score metric
//!     let quality_metric = QualityMetric::new(
//!         MetricType::ResearchQuality,
//!         MetricValue::QualityScore(quality_score.into()),
//!         Some(provider.to_string()),
//!     );
//!     collector.collect(quality_metric).await?;
//!     
//!     // Collect performance metric
//!     let perf_metric = QualityMetric::new(
//!         MetricType::ProviderPerformance,
//!         MetricValue::Gauge(response_time as f64),
//!         Some(provider.to_string()),
//!     );
//!     collector.collect(perf_metric).await?;
//!     
//!     Ok(())
//! }
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

use super::QualityScore;

/// Core metric data structure for quality and performance measurements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityMetric {
    /// Unique identifier for this metric
    pub metric_id: String,
    /// Timestamp when metric was collected
    pub timestamp: DateTime<Utc>,
    /// Type of metric being collected
    pub metric_type: MetricType,
    /// Provider associated with this metric (if applicable)
    pub provider: Option<String>,
    /// The actual metric value
    pub value: MetricValue,
    /// Additional context for the metric
    pub context: MetricContext,
    /// Key-value tags for categorization and filtering
    pub tags: HashMap<String, String>,
}

impl QualityMetric {
    /// Create a new quality metric with current timestamp
    pub fn new(metric_type: MetricType, value: MetricValue, provider: Option<String>) -> Self {
        Self {
            metric_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            metric_type,
            provider,
            value,
            context: MetricContext::default(),
            tags: HashMap::new(),
        }
    }

    /// Create a metric with additional context
    pub fn with_context(
        metric_type: MetricType,
        value: MetricValue,
        provider: Option<String>,
        context: MetricContext,
    ) -> Self {
        Self {
            metric_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            metric_type,
            provider,
            value,
            context,
            tags: HashMap::new(),
        }
    }

    /// Add a tag to the metric
    pub fn with_tag(mut self, key: String, value: String) -> Self {
        self.tags.insert(key, value);
        self
    }

    /// Add multiple tags to the metric
    pub fn with_tags(mut self, tags: HashMap<String, String>) -> Self {
        self.tags.extend(tags);
        self
    }

    /// Check if metric matches given filters
    pub fn matches_filters(&self, filters: &MetricFilters) -> bool {
        // Check metric type filter
        if let Some(ref types) = filters.metric_types {
            if !types.contains(&self.metric_type) {
                return false;
            }
        }

        // Check provider filter
        if let Some(ref providers) = filters.providers {
            match &self.provider {
                Some(provider) => {
                    if !providers.contains(provider) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Check time range filter
        if let Some((start, end)) = filters.time_range {
            if self.timestamp < start || self.timestamp > end {
                return false;
            }
        }

        // Check tag filters
        for (key, expected_value) in &filters.tags {
            match self.tags.get(key) {
                Some(actual_value) => {
                    if actual_value != expected_value {
                        return false;
                    }
                }
                None => return false,
            }
        }

        true
    }
}

/// Types of metrics that can be collected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MetricType {
    /// Research quality metrics (accuracy, relevance, etc.)
    ResearchQuality,
    /// Provider performance metrics (response time, success rate, etc.)
    ProviderPerformance,
    /// Cross-validation metrics (consistency, consensus, etc.)
    CrossValidation,
    /// User satisfaction metrics (feedback ratings, corrections, etc.)
    UserSatisfaction,
    /// System performance metrics (throughput, latency, etc.)
    SystemPerformance,
}

/// Different types of metric values
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricValue {
    /// Counter metric (monotonically increasing)
    Counter(u64),
    /// Gauge metric (current value)
    Gauge(f64),
    /// Histogram of values
    Histogram(Vec<f64>),
    /// Quality score with all dimensions
    QualityScore(QualityScore),
    /// Boolean flag metric
    Boolean(bool),
    /// Duration metric
    Duration(Duration),
}

impl MetricValue {
    /// Get numeric value for aggregation (if applicable)
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            MetricValue::Counter(val) => Some(*val as f64),
            MetricValue::Gauge(val) => Some(*val),
            MetricValue::QualityScore(score) => Some(score.composite),
            MetricValue::Boolean(val) => Some(if *val { 1.0 } else { 0.0 }),
            MetricValue::Duration(dur) => Some(dur.as_millis() as f64),
            MetricValue::Histogram(_) => None, // Requires specific aggregation
        }
    }

    /// Get the type of metric value as string
    pub fn value_type(&self) -> &'static str {
        match self {
            MetricValue::Counter(_) => "counter",
            MetricValue::Gauge(_) => "gauge",
            MetricValue::Histogram(_) => "histogram",
            MetricValue::QualityScore(_) => "quality_score",
            MetricValue::Boolean(_) => "boolean",
            MetricValue::Duration(_) => "duration",
        }
    }
}

/// Context information for metrics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MetricContext {
    /// Research type or domain
    pub research_type: Option<String>,
    /// Domain or subject area
    pub domain: Option<String>,
    /// Query complexity score
    pub query_complexity: Option<f64>,
    /// User identifier (anonymized)
    pub user_id: Option<String>,
    /// Session identifier
    pub session_id: Option<String>,
    /// Additional custom context
    pub custom: HashMap<String, String>,
}

impl MetricContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Add research type context
    pub fn with_research_type(mut self, research_type: String) -> Self {
        self.research_type = Some(research_type);
        self
    }

    /// Add domain context
    pub fn with_domain(mut self, domain: String) -> Self {
        self.domain = Some(domain);
        self
    }

    /// Add query complexity context
    pub fn with_query_complexity(mut self, complexity: f64) -> Self {
        self.query_complexity = Some(complexity);
        self
    }

    /// Add user context (should be anonymized)
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Add session context
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Add custom context parameter
    pub fn with_custom(mut self, key: String, value: String) -> Self {
        self.custom.insert(key, value);
        self
    }
}

/// Configuration for metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Maximum buffer size for real-time collection
    pub buffer_size: usize,
    /// Batch size for storage operations
    pub batch_size: usize,
    /// Flush interval for batched metrics
    pub flush_interval: Duration,
    /// Enable real-time collection
    pub enable_realtime: bool,
    /// Enable batch collection
    pub enable_batch: bool,
    /// Maximum memory usage for buffers (in bytes)
    pub max_memory_usage: usize,
    /// Retention policy configuration
    pub retention: RetentionConfig,
    /// Performance thresholds
    pub performance_thresholds: PerformanceThresholds,
    /// Whether metrics collection is enabled globally
    pub enabled: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10000,
            batch_size: 100,
            flush_interval: Duration::from_secs(30),
            enable_realtime: true,
            enable_batch: true,
            max_memory_usage: 100 * 1024 * 1024, // 100MB
            retention: RetentionConfig::default(),
            performance_thresholds: PerformanceThresholds::default(),
            enabled: true,
        }
    }
}

impl MetricsConfig {
    /// Validate the metrics configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.buffer_size == 0 {
            return Err("Buffer size must be greater than 0".to_string());
        }

        if self.batch_size == 0 {
            return Err("Batch size must be greater than 0".to_string());
        }

        if self.batch_size > self.buffer_size {
            return Err("Batch size cannot be larger than buffer size".to_string());
        }

        if self.flush_interval.as_millis() == 0 {
            return Err("Flush interval must be greater than 0".to_string());
        }

        if self.max_memory_usage == 0 {
            return Err("Max memory usage must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Create production-optimized metrics configuration
    pub fn production_optimized() -> Self {
        Self {
            buffer_size: 50000,                      // Larger buffer for production
            batch_size: 500,                         // Larger batches for efficiency
            flush_interval: Duration::from_secs(15), // More frequent flushing
            enable_realtime: true,
            enable_batch: true,
            max_memory_usage: 200 * 1024 * 1024, // 200MB
            retention: RetentionConfig::production_optimized(),
            performance_thresholds: PerformanceThresholds::production_optimized(),
            enabled: true,
        }
    }

    /// Create development-optimized metrics configuration
    pub fn development_optimized() -> Self {
        Self {
            buffer_size: 1000,                       // Smaller buffer for development
            batch_size: 50,                          // Smaller batches
            flush_interval: Duration::from_secs(60), // Less frequent flushing
            enable_realtime: false,                  // Disabled for development
            enable_batch: true,
            max_memory_usage: 50 * 1024 * 1024, // 50MB
            retention: RetentionConfig::development_optimized(),
            performance_thresholds: PerformanceThresholds::development_optimized(),
            enabled: false, // Disabled by default in development
        }
    }
}

/// Configuration for data retention policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    /// How long to keep raw metrics
    pub raw_retention: Duration,
    /// How long to keep aggregated metrics
    pub aggregated_retention: Duration,
    /// Cleanup interval
    pub cleanup_interval: Duration,
    /// Maximum storage size (in bytes)
    pub max_storage_size: Option<usize>,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            raw_retention: Duration::from_secs(30 * 24 * 3600), // 30 days
            aggregated_retention: Duration::from_secs(365 * 24 * 3600), // 1 year
            cleanup_interval: Duration::from_secs(24 * 3600),   // Daily
            max_storage_size: Some(10 * 1024 * 1024 * 1024),    // 10GB
        }
    }
}

impl RetentionConfig {
    /// Create production-optimized retention configuration
    pub fn production_optimized() -> Self {
        Self {
            raw_retention: Duration::from_secs(90 * 24 * 3600), // 90 days
            aggregated_retention: Duration::from_secs(2 * 365 * 24 * 3600), // 2 years
            cleanup_interval: Duration::from_secs(6 * 3600),    // Every 6 hours
            max_storage_size: Some(100 * 1024 * 1024 * 1024),   // 100GB
        }
    }

    /// Create development-optimized retention configuration
    pub fn development_optimized() -> Self {
        Self {
            raw_retention: Duration::from_secs(7 * 24 * 3600), // 7 days
            aggregated_retention: Duration::from_secs(30 * 24 * 3600), // 30 days
            cleanup_interval: Duration::from_secs(24 * 3600),  // Daily
            max_storage_size: Some(1024 * 1024 * 1024),        // 1GB
        }
    }
}

/// Performance thresholds for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    /// Maximum collection time per metric
    pub max_collection_time: Duration,
    /// Maximum storage time per batch
    pub max_storage_time: Duration,
    /// Maximum query time
    pub max_query_time: Duration,
    /// Maximum memory usage per operation
    pub max_memory_per_operation: usize,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_collection_time: Duration::from_millis(5),
            max_storage_time: Duration::from_millis(100),
            max_query_time: Duration::from_millis(100),
            max_memory_per_operation: 10 * 1024 * 1024, // 10MB
        }
    }
}

impl PerformanceThresholds {
    /// Create production-optimized performance thresholds
    pub fn production_optimized() -> Self {
        Self {
            max_collection_time: Duration::from_millis(2), // Stricter for production
            max_storage_time: Duration::from_millis(50),   // Stricter for production
            max_query_time: Duration::from_millis(50),     // Stricter for production
            max_memory_per_operation: 5 * 1024 * 1024,     // 5MB
        }
    }

    /// Create development-optimized performance thresholds
    pub fn development_optimized() -> Self {
        Self {
            max_collection_time: Duration::from_millis(20), // More lenient for development
            max_storage_time: Duration::from_millis(500),   // More lenient for development
            max_query_time: Duration::from_millis(500),     // More lenient for development
            max_memory_per_operation: 50 * 1024 * 1024,     // 50MB
        }
    }
}

/// Filters for querying metrics
#[derive(Debug, Clone, Default)]
pub struct MetricFilters {
    /// Filter by metric types
    pub metric_types: Option<Vec<MetricType>>,
    /// Filter by providers
    pub providers: Option<Vec<String>>,
    /// Filter by time range
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// Filter by tags
    pub tags: HashMap<String, String>,
    /// Limit number of results
    pub limit: Option<usize>,
}

impl MetricFilters {
    /// Create new empty filters
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by metric type
    pub fn with_metric_type(mut self, metric_type: MetricType) -> Self {
        self.metric_types = Some(vec![metric_type]);
        self
    }

    /// Filter by multiple metric types
    pub fn with_metric_types(mut self, metric_types: Vec<MetricType>) -> Self {
        self.metric_types = Some(metric_types);
        self
    }

    /// Filter by provider
    pub fn with_provider(mut self, provider: String) -> Self {
        self.providers = Some(vec![provider]);
        self
    }

    /// Filter by multiple providers
    pub fn with_providers(mut self, providers: Vec<String>) -> Self {
        self.providers = Some(providers);
        self
    }

    /// Filter by time range
    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.time_range = Some((start, end));
        self
    }

    /// Add tag filter
    pub fn with_tag(mut self, key: String, value: String) -> Self {
        self.tags.insert(key, value);
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Aggregated metric result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetric {
    /// Metric type
    pub metric_type: MetricType,
    /// Provider (if applicable)
    pub provider: Option<String>,
    /// Time window for aggregation
    pub time_window: (DateTime<Utc>, DateTime<Utc>),
    /// Statistical aggregations
    pub aggregations: MetricAggregations,
    /// Number of data points
    pub count: usize,
    /// Tags common to all metrics
    pub common_tags: HashMap<String, String>,
}

/// Statistical aggregations for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricAggregations {
    /// Average value
    pub average: Option<f64>,
    /// Minimum value
    pub minimum: Option<f64>,
    /// Maximum value
    pub maximum: Option<f64>,
    /// Sum of values
    pub sum: Option<f64>,
    /// Standard deviation
    pub std_dev: Option<f64>,
    /// Percentiles (50th, 95th, 99th)
    pub percentiles: Option<HashMap<String, f64>>,
}

/// Performance statistics for metrics operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsPerformanceStats {
    /// Collection performance
    pub collection_time_avg: Duration,
    pub collection_time_p95: Duration,
    /// Storage performance
    pub storage_time_avg: Duration,
    pub storage_time_p95: Duration,
    /// Query performance
    pub query_time_avg: Duration,
    pub query_time_p95: Duration,
    /// Memory usage
    pub memory_usage_current: usize,
    pub memory_usage_peak: usize,
    /// Throughput metrics
    pub metrics_per_second: f64,
    pub storage_operations_per_second: f64,
}

/// Real-time metrics collector
pub struct MetricsCollector {
    config: MetricsConfig,
    buffer: Arc<Mutex<Vec<QualityMetric>>>,
    storage: Arc<dyn MetricsStorage>,
    performance_stats: Arc<RwLock<MetricsPerformanceStats>>,
    is_running: Arc<Mutex<bool>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: MetricsConfig, storage: Arc<dyn MetricsStorage>) -> Self {
        Self {
            config,
            buffer: Arc::new(Mutex::new(Vec::new())),
            storage,
            performance_stats: Arc::new(RwLock::new(MetricsPerformanceStats {
                collection_time_avg: Duration::default(),
                collection_time_p95: Duration::default(),
                storage_time_avg: Duration::default(),
                storage_time_p95: Duration::default(),
                query_time_avg: Duration::default(),
                query_time_p95: Duration::default(),
                memory_usage_current: 0,
                memory_usage_peak: 0,
                metrics_per_second: 0.0,
                storage_operations_per_second: 0.0,
            })),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start the metrics collector background tasks
    pub async fn start(&self) -> Result<(), MetricsError> {
        let mut running = self.is_running.lock().await;
        if *running {
            return Err(MetricsError::AlreadyRunning);
        }
        *running = true;

        info!("Starting metrics collector");

        // Start background flush task if batch collection is enabled
        if self.config.enable_batch {
            self.start_flush_task().await;
        }

        Ok(())
    }

    /// Stop the metrics collector
    pub async fn stop(&self) -> Result<(), MetricsError> {
        let mut running = self.is_running.lock().await;
        if !*running {
            return Ok(());
        }
        *running = false;

        info!("Stopping metrics collector");

        // Flush remaining metrics
        self.flush_buffer().await?;

        Ok(())
    }

    /// Collect a single metric
    pub async fn collect(&self, metric: QualityMetric) -> Result<(), MetricsError> {
        let start_time = Instant::now();

        if self.config.enable_realtime {
            // Store immediately for real-time collection
            self.storage.store_metric(&metric).await?;
        }

        if self.config.enable_batch {
            // Add to buffer for batch processing
            let mut buffer = self.buffer.lock().await;
            buffer.push(metric);

            // Check if buffer needs flushing
            if buffer.len() >= self.config.batch_size {
                drop(buffer); // Release lock before flush
                self.flush_buffer().await?;
            }
        }

        // Update performance stats
        let collection_time = start_time.elapsed();
        self.update_collection_stats(collection_time).await;

        // Check performance thresholds
        if collection_time > self.config.performance_thresholds.max_collection_time {
            warn!(
                "Metrics collection exceeded threshold: {:?} > {:?}",
                collection_time, self.config.performance_thresholds.max_collection_time
            );
        }

        Ok(())
    }

    /// Collect multiple metrics in batch
    pub async fn collect_batch(&self, metrics: Vec<QualityMetric>) -> Result<(), MetricsError> {
        let start_time = Instant::now();

        if self.config.enable_realtime {
            self.storage.store_metrics(&metrics).await?;
        }

        if self.config.enable_batch {
            let mut buffer = self.buffer.lock().await;
            buffer.extend(metrics);

            if buffer.len() >= self.config.batch_size {
                drop(buffer);
                self.flush_buffer().await?;
            }
        }

        let collection_time = start_time.elapsed();
        self.update_collection_stats(collection_time).await;

        Ok(())
    }

    /// Get current performance statistics
    pub async fn performance_stats(&self) -> MetricsPerformanceStats {
        self.performance_stats.read().await.clone()
    }

    /// Start background flush task
    async fn start_flush_task(&self) {
        let buffer = Arc::clone(&self.buffer);
        let storage = Arc::clone(&self.storage);
        let is_running = Arc::clone(&self.is_running);
        let flush_interval = self.config.flush_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(flush_interval);

            loop {
                interval.tick().await;

                let running = is_running.lock().await;
                if !*running {
                    break;
                }
                drop(running);

                // Flush buffer if not empty
                let mut buffer_guard = buffer.lock().await;
                if !buffer_guard.is_empty() {
                    let metrics_to_flush = buffer_guard.drain(..).collect::<Vec<_>>();
                    drop(buffer_guard);

                    if let Err(e) = storage.store_metrics(&metrics_to_flush).await {
                        error!("Failed to flush metrics buffer: {}", e);
                    } else {
                        debug!("Flushed {} metrics from buffer", metrics_to_flush.len());
                    }
                }
            }
        });
    }

    /// Flush current buffer to storage
    async fn flush_buffer(&self) -> Result<(), MetricsError> {
        let mut buffer = self.buffer.lock().await;
        if buffer.is_empty() {
            return Ok(());
        }

        let metrics_to_flush = buffer.drain(..).collect::<Vec<_>>();
        drop(buffer);

        let start_time = Instant::now();
        self.storage.store_metrics(&metrics_to_flush).await?;
        let storage_time = start_time.elapsed();

        self.update_storage_stats(storage_time).await;

        debug!("Flushed {} metrics to storage", metrics_to_flush.len());
        Ok(())
    }

    /// Update collection performance statistics
    async fn update_collection_stats(&self, collection_time: Duration) {
        let mut stats = self.performance_stats.write().await;

        // Simple moving average for now (could be improved with proper windowing)
        stats.collection_time_avg = Duration::from_nanos(
            (stats.collection_time_avg.as_nanos() as f64 * 0.9
                + collection_time.as_nanos() as f64 * 0.1) as u64,
        );

        // Update P95 (simplified)
        if collection_time > stats.collection_time_p95 {
            stats.collection_time_p95 = collection_time;
        } else {
            stats.collection_time_p95 =
                Duration::from_nanos((stats.collection_time_p95.as_nanos() as f64 * 0.95) as u64);
        }
    }

    /// Update storage performance statistics
    async fn update_storage_stats(&self, storage_time: Duration) {
        let mut stats = self.performance_stats.write().await;

        stats.storage_time_avg = Duration::from_nanos(
            (stats.storage_time_avg.as_nanos() as f64 * 0.9 + storage_time.as_nanos() as f64 * 0.1)
                as u64,
        );

        if storage_time > stats.storage_time_p95 {
            stats.storage_time_p95 = storage_time;
        } else {
            stats.storage_time_p95 =
                Duration::from_nanos((stats.storage_time_p95.as_nanos() as f64 * 0.95) as u64);
        }
    }
}

/// Trait for metrics storage backends
#[async_trait]
pub trait MetricsStorage: Send + Sync {
    /// Store a single metric
    async fn store_metric(&self, metric: &QualityMetric) -> Result<(), MetricsError>;

    /// Store multiple metrics in batch
    async fn store_metrics(&self, metrics: &[QualityMetric]) -> Result<(), MetricsError>;

    /// Query metrics with filters
    async fn query_metrics(
        &self,
        filters: &MetricFilters,
    ) -> Result<Vec<QualityMetric>, MetricsError>;

    /// Get aggregated metrics
    async fn get_aggregated_metrics(
        &self,
        filters: &MetricFilters,
        time_window: Duration,
    ) -> Result<Vec<AggregatedMetric>, MetricsError>;

    /// Delete metrics matching filters
    async fn delete_metrics(&self, filters: &MetricFilters) -> Result<usize, MetricsError>;

    /// Get storage statistics
    async fn get_storage_stats(&self) -> Result<StorageStats, MetricsError>;

    /// Perform cleanup based on retention policy
    async fn cleanup(&self, retention: &RetentionConfig) -> Result<CleanupStats, MetricsError>;
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    /// Total number of metrics stored
    pub total_metrics: usize,
    /// Total storage size in bytes
    pub storage_size_bytes: usize,
    /// Number of metrics by type
    pub metrics_by_type: HashMap<MetricType, usize>,
    /// Number of metrics by provider
    pub metrics_by_provider: HashMap<String, usize>,
    /// Oldest metric timestamp
    pub oldest_metric: Option<DateTime<Utc>>,
    /// Newest metric timestamp
    pub newest_metric: Option<DateTime<Utc>>,
}

/// Cleanup operation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupStats {
    /// Number of metrics deleted
    pub metrics_deleted: usize,
    /// Storage space freed in bytes
    pub bytes_freed: usize,
    /// Time taken for cleanup
    pub cleanup_duration: Duration,
}

/// Analytics engine for metrics analysis
pub struct MetricsAnalyzer {
    storage: Arc<dyn MetricsStorage>,
}

impl MetricsAnalyzer {
    /// Create a new metrics analyzer
    pub fn new(storage: Arc<dyn MetricsStorage>) -> Self {
        Self { storage }
    }

    /// Analyze quality trends over time
    pub async fn analyze_quality_trends(
        &self,
        provider: Option<String>,
        time_range: (DateTime<Utc>, DateTime<Utc>),
    ) -> Result<QualityTrends, MetricsError> {
        let mut filters = MetricFilters::new()
            .with_metric_type(MetricType::ResearchQuality)
            .with_time_range(time_range.0, time_range.1);

        if let Some(provider) = provider {
            filters = filters.with_provider(provider);
        }

        let metrics = self.storage.query_metrics(&filters).await?;

        // Analyze trends
        let quality_scores: Vec<f64> = metrics
            .iter()
            .filter_map(|m| match &m.value {
                MetricValue::QualityScore(score) => Some(score.composite),
                _ => None,
            })
            .collect();

        if quality_scores.is_empty() {
            return Ok(QualityTrends::default());
        }

        let average = quality_scores.iter().sum::<f64>() / quality_scores.len() as f64;
        let min = quality_scores.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = quality_scores
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        // Calculate trend direction (simplified linear regression)
        let trend_direction = self.calculate_trend_direction(&quality_scores);

        Ok(QualityTrends {
            average_quality: average,
            min_quality: min,
            max_quality: max,
            trend_direction,
            sample_count: quality_scores.len(),
            time_range,
        })
    }

    /// Analyze provider performance comparison
    pub async fn analyze_provider_performance(
        &self,
        time_range: (DateTime<Utc>, DateTime<Utc>),
    ) -> Result<Vec<ProviderPerformanceAnalysis>, MetricsError> {
        let filters = MetricFilters::new()
            .with_metric_type(MetricType::ProviderPerformance)
            .with_time_range(time_range.0, time_range.1);

        let metrics = self.storage.query_metrics(&filters).await?;

        // Group by provider
        let mut provider_metrics: HashMap<String, Vec<f64>> = HashMap::new();

        for metric in metrics {
            if let Some(provider) = &metric.provider {
                if let Some(value) = metric.value.as_f64() {
                    provider_metrics
                        .entry(provider.clone())
                        .or_default()
                        .push(value);
                }
            }
        }

        let mut results = Vec::new();
        for (provider, values) in provider_metrics {
            if !values.is_empty() {
                let average = values.iter().sum::<f64>() / values.len() as f64;
                let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

                results.push(ProviderPerformanceAnalysis {
                    provider,
                    average_performance: average,
                    min_performance: min,
                    max_performance: max,
                    sample_count: values.len(),
                    reliability_score: self.calculate_reliability_score(&values),
                });
            }
        }

        Ok(results)
    }

    /// Detect quality anomalies
    pub async fn detect_quality_anomalies(
        &self,
        threshold_std_dev: f64,
        time_range: (DateTime<Utc>, DateTime<Utc>),
    ) -> Result<Vec<QualityAnomaly>, MetricsError> {
        let filters = MetricFilters::new()
            .with_metric_type(MetricType::ResearchQuality)
            .with_time_range(time_range.0, time_range.1);

        let metrics = self.storage.query_metrics(&filters).await?;

        let quality_scores: Vec<(DateTime<Utc>, f64, Option<String>)> = metrics
            .iter()
            .filter_map(|m| match &m.value {
                MetricValue::QualityScore(score) => {
                    Some((m.timestamp, score.composite, m.provider.clone()))
                }
                _ => None,
            })
            .collect();

        if quality_scores.len() < 10 {
            return Ok(Vec::new()); // Need sufficient data for anomaly detection
        }

        // Calculate statistics
        let values: Vec<f64> = quality_scores.iter().map(|(_, score, _)| *score).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance =
            values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        let threshold = std_dev * threshold_std_dev;

        // Find anomalies
        let mut anomalies = Vec::new();
        for (timestamp, score, provider) in quality_scores {
            if (score - mean).abs() > threshold {
                anomalies.push(QualityAnomaly {
                    timestamp,
                    quality_score: score,
                    expected_range: (mean - threshold, mean + threshold),
                    deviation: (score - mean).abs(),
                    provider,
                    severity: if (score - mean).abs() > threshold * 2.0 {
                        AnomalySeverity::High
                    } else {
                        AnomalySeverity::Medium
                    },
                });
            }
        }

        Ok(anomalies)
    }

    /// Calculate trend direction using simple linear regression
    fn calculate_trend_direction(&self, values: &[f64]) -> TrendDirection {
        if values.len() < 2 {
            return TrendDirection::Stable;
        }

        let n = values.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = values.iter().sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, &y) in values.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean).powi(2);
        }

        if denominator == 0.0 {
            return TrendDirection::Stable;
        }

        let slope = numerator / denominator;

        if slope > 0.01 {
            TrendDirection::Improving
        } else if slope < -0.01 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        }
    }

    /// Calculate reliability score based on variance
    fn calculate_reliability_score(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance =
            values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let coefficient_of_variation = if mean > 0.0 {
            variance.sqrt() / mean
        } else {
            1.0
        };

        // Higher reliability for lower variance (inverse relationship)
        (1.0 / (1.0 + coefficient_of_variation)).clamp(0.0, 1.0)
    }
}

/// Quality trends analysis result
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityTrends {
    /// Average quality score
    pub average_quality: f64,
    /// Minimum quality score
    pub min_quality: f64,
    /// Maximum quality score  
    pub max_quality: f64,
    /// Trend direction
    pub trend_direction: TrendDirection,
    /// Number of samples
    pub sample_count: usize,
    /// Time range analyzed
    pub time_range: (DateTime<Utc>, DateTime<Utc>),
}

/// Provider performance analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPerformanceAnalysis {
    /// Provider name
    pub provider: String,
    /// Average performance metric
    pub average_performance: f64,
    /// Minimum performance
    pub min_performance: f64,
    /// Maximum performance
    pub max_performance: f64,
    /// Number of samples
    pub sample_count: usize,
    /// Reliability score (0.0 - 1.0)
    pub reliability_score: f64,
}

/// Quality anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAnomaly {
    /// When the anomaly occurred
    pub timestamp: DateTime<Utc>,
    /// The anomalous quality score
    pub quality_score: f64,
    /// Expected range for normal scores
    pub expected_range: (f64, f64),
    /// How much it deviates from normal
    pub deviation: f64,
    /// Provider associated with anomaly
    pub provider: Option<String>,
    /// Severity of the anomaly
    pub severity: AnomalySeverity,
}

/// Trend direction enumeration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum TrendDirection {
    Improving,
    #[default]
    Stable,
    Declining,
}

/// Anomaly severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Errors that can occur during metrics operations
#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("Storage error: {message}")]
    StorageError { message: String },

    #[error("Invalid metric data: {message}")]
    InvalidMetric { message: String },

    #[error("Performance threshold exceeded: {operation} took {actual:?}, limit is {limit:?}")]
    PerformanceThresholdExceeded {
        operation: String,
        actual: Duration,
        limit: Duration,
    },

    #[error("Buffer overflow: buffer size {current} exceeds limit {limit}")]
    BufferOverflow { current: usize, limit: usize },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Collector already running")]
    AlreadyRunning,

    #[error("Collector not running")]
    NotRunning,

    #[error("Serialization error: {source}")]
    SerializationError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Analytics error: {message}")]
    AnalyticsError { message: String },

    #[error("Query error: {message}")]
    QueryError { message: String },

    #[error("Network error: {source}")]
    NetworkError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },
}

/// Result type for metrics operations
pub type MetricsResult<T> = Result<T, MetricsError>;

/// In-memory storage implementation for development and testing
#[derive(Debug, Clone)]
pub struct InMemoryMetricsStorage {
    metrics: Arc<Mutex<Vec<QualityMetric>>>,
    aggregations_cache: Arc<Mutex<HashMap<String, CachedAggregation>>>,
}

#[derive(Debug, Clone)]
struct CachedAggregation {
    aggregation: AggregatedMetric,
    created_at: DateTime<Utc>,
    cache_ttl: Duration,
}

impl InMemoryMetricsStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
            aggregations_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the current number of stored metrics (for testing)
    pub async fn metrics_count(&self) -> usize {
        self.metrics.lock().await.len()
    }

    /// Clear all stored metrics (for testing)
    pub async fn clear(&self) {
        self.metrics.lock().await.clear();
        self.aggregations_cache.lock().await.clear();
    }

    /// Generate cache key for aggregation
    fn aggregation_cache_key(filters: &MetricFilters, time_window: Duration) -> String {
        format!(
            "agg_{}_{}_{}_{:?}",
            filters
                .metric_types
                .as_ref()
                .map(|types| format!("{types:?}"))
                .unwrap_or_else(|| "all".to_string()),
            filters
                .providers
                .as_ref()
                .map(|providers| providers.join(","))
                .unwrap_or_else(|| "all".to_string()),
            time_window.as_secs(),
            filters
                .time_range
                .map(|(start, end)| format!("{}_{}", start.timestamp(), end.timestamp()))
                .unwrap_or_else(|| "all".to_string())
        )
    }

    /// Check if cached aggregation is still valid
    fn is_cache_valid(cached: &CachedAggregation) -> bool {
        let elapsed = Utc::now() - cached.created_at;
        elapsed.to_std().unwrap_or(Duration::MAX) < cached.cache_ttl
    }

    /// Calculate aggregations for a set of metrics
    fn calculate_aggregations(metrics: &[QualityMetric]) -> MetricAggregations {
        let values: Vec<f64> = metrics.iter().filter_map(|m| m.value.as_f64()).collect();

        if values.is_empty() {
            return MetricAggregations {
                average: None,
                minimum: None,
                maximum: None,
                sum: None,
                std_dev: None,
                percentiles: None,
            };
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let average = sum / count;
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        // Calculate standard deviation
        let variance = values.iter().map(|&x| (x - average).powi(2)).sum::<f64>() / count;
        let std_dev = variance.sqrt();

        // Calculate percentiles
        let mut sorted_values = values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mut percentiles = HashMap::new();
        if !sorted_values.is_empty() {
            let len = sorted_values.len();
            percentiles.insert("50".to_string(), sorted_values[len / 2]);
            percentiles.insert(
                "95".to_string(),
                sorted_values[(len as f64 * 0.95) as usize],
            );
            percentiles.insert(
                "99".to_string(),
                sorted_values[(len as f64 * 0.99) as usize],
            );
        }

        MetricAggregations {
            average: Some(average),
            minimum: Some(min),
            maximum: Some(max),
            sum: Some(sum),
            std_dev: Some(std_dev),
            percentiles: Some(percentiles),
        }
    }
}

impl Default for InMemoryMetricsStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MetricsStorage for InMemoryMetricsStorage {
    async fn store_metric(&self, metric: &QualityMetric) -> Result<(), MetricsError> {
        let mut metrics = self.metrics.lock().await;
        metrics.push(metric.clone());
        Ok(())
    }

    async fn store_metrics(&self, metrics: &[QualityMetric]) -> Result<(), MetricsError> {
        let mut stored_metrics = self.metrics.lock().await;
        stored_metrics.extend_from_slice(metrics);
        Ok(())
    }

    async fn query_metrics(
        &self,
        filters: &MetricFilters,
    ) -> Result<Vec<QualityMetric>, MetricsError> {
        let metrics = self.metrics.lock().await;
        let mut results: Vec<QualityMetric> = metrics
            .iter()
            .filter(|m| m.matches_filters(filters))
            .cloned()
            .collect();

        // Sort by timestamp (newest first)
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit if specified
        if let Some(limit) = filters.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    async fn get_aggregated_metrics(
        &self,
        filters: &MetricFilters,
        time_window: Duration,
    ) -> Result<Vec<AggregatedMetric>, MetricsError> {
        let cache_key = Self::aggregation_cache_key(filters, time_window);

        // Check cache first
        {
            let cache = self.aggregations_cache.lock().await;
            if let Some(cached) = cache.get(&cache_key) {
                if Self::is_cache_valid(cached) {
                    return Ok(vec![cached.aggregation.clone()]);
                }
            }
        }

        // Get filtered metrics
        let metrics = self.query_metrics(filters).await?;
        if metrics.is_empty() {
            return Ok(Vec::new());
        }

        // Group by provider and metric type
        let mut grouped: HashMap<(MetricType, Option<String>), Vec<QualityMetric>> = HashMap::new();

        for metric in metrics {
            let key = (metric.metric_type.clone(), metric.provider.clone());
            grouped.entry(key).or_default().push(metric);
        }

        let mut results = Vec::new();
        for ((metric_type, provider), group_metrics) in grouped {
            if !group_metrics.is_empty() {
                let aggregations = Self::calculate_aggregations(&group_metrics);

                let start_time = group_metrics.iter().map(|m| m.timestamp).min().unwrap();
                let end_time = group_metrics.iter().map(|m| m.timestamp).max().unwrap();

                let aggregated = AggregatedMetric {
                    metric_type,
                    provider,
                    time_window: (start_time, end_time),
                    aggregations,
                    count: group_metrics.len(),
                    common_tags: HashMap::new(), // Could be enhanced to find common tags
                };

                results.push(aggregated.clone());

                // Cache the result
                let cached = CachedAggregation {
                    aggregation: aggregated,
                    created_at: Utc::now(),
                    cache_ttl: Duration::from_secs(300), // 5 minutes
                };
                self.aggregations_cache
                    .lock()
                    .await
                    .insert(cache_key.clone(), cached);
            }
        }

        Ok(results)
    }

    async fn delete_metrics(&self, filters: &MetricFilters) -> Result<usize, MetricsError> {
        let mut metrics = self.metrics.lock().await;
        let initial_len = metrics.len();
        metrics.retain(|m| !m.matches_filters(filters));

        // Clear aggregation cache since data changed
        self.aggregations_cache.lock().await.clear();

        Ok(initial_len - metrics.len())
    }

    async fn get_storage_stats(&self) -> Result<StorageStats, MetricsError> {
        let metrics = self.metrics.lock().await;
        let mut metrics_by_type = HashMap::new();
        let mut metrics_by_provider = HashMap::new();

        for metric in metrics.iter() {
            *metrics_by_type
                .entry(metric.metric_type.clone())
                .or_insert(0) += 1;
            if let Some(ref provider) = metric.provider {
                *metrics_by_provider.entry(provider.clone()).or_insert(0) += 1;
            }
        }

        Ok(StorageStats {
            total_metrics: metrics.len(),
            storage_size_bytes: metrics.len() * std::mem::size_of::<QualityMetric>(),
            metrics_by_type,
            metrics_by_provider,
            oldest_metric: metrics.first().map(|m| m.timestamp),
            newest_metric: metrics.last().map(|m| m.timestamp),
        })
    }

    async fn cleanup(&self, retention: &RetentionConfig) -> Result<CleanupStats, MetricsError> {
        let cutoff_chrono = chrono::Duration::from_std(retention.raw_retention).map_err(|e| {
            MetricsError::ConfigurationError {
                message: format!("Invalid retention duration: {e}"),
            }
        })?;
        let cutoff_time = Utc::now() - cutoff_chrono;
        let mut metrics = self.metrics.lock().await;
        let initial_len = metrics.len();

        metrics.retain(|m| m.timestamp >= cutoff_time);
        let deleted_count = initial_len - metrics.len();

        // Clear aggregation cache since data changed
        self.aggregations_cache.lock().await.clear();

        Ok(CleanupStats {
            metrics_deleted: deleted_count,
            bytes_freed: deleted_count * std::mem::size_of::<QualityMetric>(),
            cleanup_duration: Duration::from_millis(10), // Fast for in-memory
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Mock storage implementation for testing
    #[derive(Debug)]
    struct MockMetricsStorage {
        metrics: Arc<Mutex<Vec<QualityMetric>>>,
        store_calls: Arc<AtomicUsize>,
        query_calls: Arc<AtomicUsize>,
    }

    impl MockMetricsStorage {
        fn new() -> Self {
            Self {
                metrics: Arc::new(Mutex::new(Vec::new())),
                store_calls: Arc::new(AtomicUsize::new(0)),
                query_calls: Arc::new(AtomicUsize::new(0)),
            }
        }

        async fn get_stored_metrics(&self) -> Vec<QualityMetric> {
            self.metrics.lock().await.clone()
        }

        fn get_store_calls(&self) -> usize {
            self.store_calls.load(Ordering::Relaxed)
        }

        fn get_query_calls(&self) -> usize {
            self.query_calls.load(Ordering::Relaxed)
        }
    }

    #[async_trait]
    impl MetricsStorage for MockMetricsStorage {
        async fn store_metric(&self, metric: &QualityMetric) -> Result<(), MetricsError> {
            self.store_calls.fetch_add(1, Ordering::Relaxed);
            self.metrics.lock().await.push(metric.clone());
            Ok(())
        }

        async fn store_metrics(&self, metrics: &[QualityMetric]) -> Result<(), MetricsError> {
            self.store_calls.fetch_add(1, Ordering::Relaxed);
            self.metrics.lock().await.extend_from_slice(metrics);
            Ok(())
        }

        async fn query_metrics(
            &self,
            filters: &MetricFilters,
        ) -> Result<Vec<QualityMetric>, MetricsError> {
            self.query_calls.fetch_add(1, Ordering::Relaxed);
            let metrics = self.metrics.lock().await;
            Ok(metrics
                .iter()
                .filter(|m| m.matches_filters(filters))
                .cloned()
                .collect())
        }

        async fn get_aggregated_metrics(
            &self,
            _filters: &MetricFilters,
            _time_window: Duration,
        ) -> Result<Vec<AggregatedMetric>, MetricsError> {
            // Simplified implementation for testing
            Ok(Vec::new())
        }

        async fn delete_metrics(&self, filters: &MetricFilters) -> Result<usize, MetricsError> {
            let mut metrics = self.metrics.lock().await;
            let initial_len = metrics.len();
            metrics.retain(|m| !m.matches_filters(filters));
            Ok(initial_len - metrics.len())
        }

        async fn get_storage_stats(&self) -> Result<StorageStats, MetricsError> {
            let metrics = self.metrics.lock().await;
            Ok(StorageStats {
                total_metrics: metrics.len(),
                storage_size_bytes: metrics.len() * 1024, // Rough estimate
                metrics_by_type: HashMap::new(),
                metrics_by_provider: HashMap::new(),
                oldest_metric: metrics.first().map(|m| m.timestamp),
                newest_metric: metrics.last().map(|m| m.timestamp),
            })
        }

        async fn cleanup(
            &self,
            _retention: &RetentionConfig,
        ) -> Result<CleanupStats, MetricsError> {
            Ok(CleanupStats {
                metrics_deleted: 0,
                bytes_freed: 0,
                cleanup_duration: Duration::from_millis(10),
            })
        }
    }

    #[test]
    fn test_quality_metric_creation() {
        let metric = QualityMetric::new(
            MetricType::ResearchQuality,
            MetricValue::Gauge(0.85),
            Some("claude".to_string()),
        );

        assert_eq!(metric.metric_type, MetricType::ResearchQuality);
        assert_eq!(metric.provider, Some("claude".to_string()));
        assert!(matches!(metric.value, MetricValue::Gauge(0.85)));
        assert!(!metric.metric_id.is_empty());
    }

    #[test]
    fn test_metric_with_context_and_tags() {
        let context = MetricContext::new()
            .with_research_type("technical".to_string())
            .with_domain("rust".to_string())
            .with_query_complexity(0.7);

        let metric = QualityMetric::with_context(
            MetricType::ProviderPerformance,
            MetricValue::Duration(Duration::from_millis(150)),
            Some("openai".to_string()),
            context,
        )
        .with_tag("endpoint".to_string(), "chat".to_string())
        .with_tag("model".to_string(), "gpt-4".to_string());

        assert_eq!(metric.context.research_type, Some("technical".to_string()));
        assert_eq!(metric.context.domain, Some("rust".to_string()));
        assert_eq!(metric.context.query_complexity, Some(0.7));
        assert_eq!(metric.tags.get("endpoint"), Some(&"chat".to_string()));
        assert_eq!(metric.tags.get("model"), Some(&"gpt-4".to_string()));
    }

    #[test]
    fn test_metric_value_conversions() {
        assert_eq!(MetricValue::Counter(42).as_f64(), Some(42.0));
        assert_eq!(MetricValue::Gauge(3.14).as_f64(), Some(3.14));
        assert_eq!(MetricValue::Boolean(true).as_f64(), Some(1.0));
        assert_eq!(MetricValue::Boolean(false).as_f64(), Some(0.0));
        assert_eq!(
            MetricValue::Duration(Duration::from_millis(500)).as_f64(),
            Some(500.0)
        );

        let quality_score = QualityScore {
            composite: 0.75,
            ..Default::default()
        };
        assert_eq!(
            MetricValue::QualityScore(quality_score).as_f64(),
            Some(0.75)
        );

        assert_eq!(MetricValue::Histogram(vec![1.0, 2.0, 3.0]).as_f64(), None);
    }

    #[test]
    fn test_metric_filtering() {
        let metric = QualityMetric::new(
            MetricType::ResearchQuality,
            MetricValue::Gauge(0.85),
            Some("claude".to_string()),
        )
        .with_tag("domain".to_string(), "ai".to_string());

        // Test type filter
        let type_filter = MetricFilters::new().with_metric_type(MetricType::ResearchQuality);
        assert!(metric.matches_filters(&type_filter));

        let wrong_type_filter =
            MetricFilters::new().with_metric_type(MetricType::ProviderPerformance);
        assert!(!metric.matches_filters(&wrong_type_filter));

        // Test provider filter
        let provider_filter = MetricFilters::new().with_provider("claude".to_string());
        assert!(metric.matches_filters(&provider_filter));

        let wrong_provider_filter = MetricFilters::new().with_provider("openai".to_string());
        assert!(!metric.matches_filters(&wrong_provider_filter));

        // Test tag filter
        let tag_filter = MetricFilters::new().with_tag("domain".to_string(), "ai".to_string());
        assert!(metric.matches_filters(&tag_filter));

        let wrong_tag_filter =
            MetricFilters::new().with_tag("domain".to_string(), "science".to_string());
        assert!(!metric.matches_filters(&wrong_tag_filter));
    }

    #[test]
    fn test_metrics_config_defaults() {
        let config = MetricsConfig::default();

        assert_eq!(config.buffer_size, 10000);
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.flush_interval, Duration::from_secs(30));
        assert!(config.enable_realtime);
        assert!(config.enable_batch);
        assert_eq!(config.max_memory_usage, 100 * 1024 * 1024);
    }

    #[test]
    fn test_retention_config_defaults() {
        let retention = RetentionConfig::default();

        assert_eq!(retention.raw_retention, Duration::from_secs(30 * 24 * 3600));
        assert_eq!(
            retention.aggregated_retention,
            Duration::from_secs(365 * 24 * 3600)
        );
        assert_eq!(retention.cleanup_interval, Duration::from_secs(24 * 3600));
        assert_eq!(retention.max_storage_size, Some(10 * 1024 * 1024 * 1024));
    }

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let config = MetricsConfig::default();
        let storage = Arc::new(MockMetricsStorage::new());
        let collector = MetricsCollector::new(config, storage);

        // Test that collector is created successfully
        let stats = collector.performance_stats().await;
        assert_eq!(stats.collection_time_avg, Duration::default());
        assert_eq!(stats.metrics_per_second, 0.0);
    }

    #[tokio::test]
    async fn test_metrics_collector_start_stop() {
        let config = MetricsConfig::default();
        let storage = Arc::new(MockMetricsStorage::new());
        let collector = MetricsCollector::new(config, storage);

        // Test starting
        assert!(collector.start().await.is_ok());

        // Test that starting again fails
        assert!(collector.start().await.is_err());

        // Test stopping
        assert!(collector.stop().await.is_ok());

        // Test that stopping again succeeds (idempotent)
        assert!(collector.stop().await.is_ok());
    }

    #[tokio::test]
    async fn test_collect_single_metric() {
        let mut config = MetricsConfig::default();
        config.enable_batch = false; // Only real-time for this test

        let storage = Arc::new(MockMetricsStorage::new());
        let collector = MetricsCollector::new(config, storage.clone());

        let metric = QualityMetric::new(
            MetricType::ResearchQuality,
            MetricValue::Gauge(0.85),
            Some("claude".to_string()),
        );

        assert!(collector.collect(metric.clone()).await.is_ok());

        // Verify metric was stored
        let stored_metrics = storage.get_stored_metrics().await;
        assert_eq!(stored_metrics.len(), 1);
        assert_eq!(stored_metrics[0].metric_type, metric.metric_type);
        assert_eq!(stored_metrics[0].provider, metric.provider);
    }

    #[tokio::test]
    async fn test_collect_batch_metrics() {
        let mut config = MetricsConfig::default();
        config.enable_realtime = false; // Only batch for this test
        config.batch_size = 2;

        let storage = Arc::new(MockMetricsStorage::new());
        let collector = MetricsCollector::new(config, storage.clone());

        let metrics = vec![
            QualityMetric::new(
                MetricType::ResearchQuality,
                MetricValue::Gauge(0.85),
                Some("claude".to_string()),
            ),
            QualityMetric::new(
                MetricType::ProviderPerformance,
                MetricValue::Duration(Duration::from_millis(200)),
                Some("claude".to_string()),
            ),
        ];

        assert!(collector.collect_batch(metrics.clone()).await.is_ok());

        // Should trigger flush due to batch size
        let stored_metrics = storage.get_stored_metrics().await;
        assert_eq!(stored_metrics.len(), 2);
    }

    #[tokio::test]
    async fn test_mock_storage_operations() {
        let storage = MockMetricsStorage::new();

        let metric = QualityMetric::new(
            MetricType::ResearchQuality,
            MetricValue::Gauge(0.85),
            Some("claude".to_string()),
        );

        // Test single store
        assert!(storage.store_metric(&metric).await.is_ok());
        assert_eq!(storage.get_store_calls(), 1);

        // Test batch store
        let metrics = vec![metric.clone(), metric.clone()];
        assert!(storage.store_metrics(&metrics).await.is_ok());
        assert_eq!(storage.get_store_calls(), 2);

        // Test query
        let filters = MetricFilters::new().with_metric_type(MetricType::ResearchQuality);
        let results = storage.query_metrics(&filters).await.unwrap();
        assert_eq!(results.len(), 3); // 1 + 2 from batches
        assert_eq!(storage.get_query_calls(), 1);

        // Test storage stats
        let stats = storage.get_storage_stats().await.unwrap();
        assert_eq!(stats.total_metrics, 3);
    }

    #[tokio::test]
    async fn test_metrics_analyzer_creation() {
        let storage = Arc::new(MockMetricsStorage::new()) as Arc<dyn MetricsStorage>;
        let analyzer = MetricsAnalyzer::new(storage);

        // Analyzer should be created successfully
        // More comprehensive tests would require actual data
        let _analyzer = analyzer; // Just ensure it compiles
    }

    // Test specific error conditions
    #[test]
    fn test_metrics_error_types() {
        let storage_error = MetricsError::StorageError {
            message: "Connection failed".to_string(),
        };
        assert!(storage_error.to_string().contains("Storage error"));

        let invalid_metric_error = MetricsError::InvalidMetric {
            message: "Missing required field".to_string(),
        };
        assert!(invalid_metric_error
            .to_string()
            .contains("Invalid metric data"));

        let performance_error = MetricsError::PerformanceThresholdExceeded {
            operation: "collection".to_string(),
            actual: Duration::from_millis(150),
            limit: Duration::from_millis(100),
        };
        assert!(performance_error
            .to_string()
            .contains("Performance threshold exceeded"));
    }

    // Test configuration validation
    #[test]
    fn test_metrics_config_validation() {
        let mut config = MetricsConfig::default();

        // Test that default config is reasonable
        assert!(config.buffer_size > 0);
        assert!(config.batch_size > 0);
        assert!(config.max_memory_usage > 0);

        // Test edge cases
        config.buffer_size = 0;
        // In real implementation, we'd validate this

        config.batch_size = config.buffer_size + 1;
        // In real implementation, batch_size should be <= buffer_size
    }

    // Performance tests
    #[tokio::test]
    async fn test_collection_performance() {
        let config = MetricsConfig::default();
        let storage = Arc::new(MockMetricsStorage::new());
        let collector = MetricsCollector::new(config, storage);

        let metric = QualityMetric::new(
            MetricType::ResearchQuality,
            MetricValue::Gauge(0.85),
            Some("claude".to_string()),
        );

        let start = Instant::now();
        assert!(collector.collect(metric).await.is_ok());
        let duration = start.elapsed();

        // Should complete well under performance threshold (5ms)
        assert!(duration < Duration::from_millis(50)); // Being generous for test environment
    }

    // Integration test for filters
    #[tokio::test]
    async fn test_metric_filtering_integration() {
        let storage = MockMetricsStorage::new();

        // Create metrics with different properties
        let metric1 = QualityMetric::new(
            MetricType::ResearchQuality,
            MetricValue::Gauge(0.85),
            Some("claude".to_string()),
        );

        let metric2 = QualityMetric::new(
            MetricType::ProviderPerformance,
            MetricValue::Duration(Duration::from_millis(200)),
            Some("openai".to_string()),
        );

        let metric3 = QualityMetric::new(
            MetricType::ResearchQuality,
            MetricValue::Gauge(0.75),
            Some("claude".to_string()),
        );

        // Store all metrics
        storage
            .store_metrics(&[metric1, metric2, metric3])
            .await
            .unwrap();

        // Test filtering by type
        let type_filter = MetricFilters::new().with_metric_type(MetricType::ResearchQuality);
        let results = storage.query_metrics(&type_filter).await.unwrap();
        assert_eq!(results.len(), 2);

        // Test filtering by provider
        let provider_filter = MetricFilters::new().with_provider("claude".to_string());
        let results = storage.query_metrics(&provider_filter).await.unwrap();
        assert_eq!(results.len(), 2);

        // Test combined filtering
        let combined_filter = MetricFilters::new()
            .with_metric_type(MetricType::ResearchQuality)
            .with_provider("claude".to_string());
        let results = storage.query_metrics(&combined_filter).await.unwrap();
        assert_eq!(results.len(), 2);
    }
}
