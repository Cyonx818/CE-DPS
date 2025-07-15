// ABOUTME: Performance metrics collection for comprehensive system monitoring
//! # Metrics Collection Module
//!
//! This module provides comprehensive performance metrics collection for all
//! system components including API operations, provider performance, quality
//! processing, cache operations, learning system, and resource utilization.
//!
//! ## Key Features
//!
//! - **Low-overhead collection**: <5% performance impact
//! - **Real-time aggregation**: Metrics available within 1 second
//! - **Threshold monitoring**: Automated violation detection
//! - **Multi-dimensional tracking**: Request times, error rates, resource usage
//!
//! ## Metrics Categories
//!
//! - API request/response metrics
//! - Provider performance metrics  
//! - Quality processing overhead
//! - Cache hit rates and performance
//! - Learning system metrics
//! - Resource utilization metrics

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use super::{MonitoringError, MonitoringResult};

/// Core metrics collector for performance data
#[derive(Debug)]
pub struct MetricsCollector {
    /// API request metrics storage
    api_metrics: Arc<RwLock<ApiMetrics>>,

    /// Provider performance metrics by provider name
    provider_metrics: Arc<RwLock<HashMap<String, ProviderPerformanceMetrics>>>,

    /// Quality processing metrics
    quality_metrics: Arc<RwLock<QualityMetrics>>,

    /// Cache metrics by cache name
    cache_metrics: Arc<RwLock<HashMap<String, CacheMetrics>>>,

    /// Learning system metrics
    learning_metrics: Arc<RwLock<LearningSystemMetrics>>,

    /// Resource utilization metrics
    resource_metrics: Arc<RwLock<ResourceUtilizationMetrics>>,

    /// Performance thresholds for violation detection
    thresholds: Arc<RwLock<Option<PerformanceThresholds>>>,

    /// Historical metrics for trend analysis
    #[allow(dead_code)] // TODO: Will be used for historical trend analysis
    historical_metrics: Arc<RwLock<Vec<MetricsSnapshot>>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            api_metrics: Arc::new(RwLock::new(ApiMetrics::new())),
            provider_metrics: Arc::new(RwLock::new(HashMap::new())),
            quality_metrics: Arc::new(RwLock::new(QualityMetrics::new())),
            cache_metrics: Arc::new(RwLock::new(HashMap::new())),
            learning_metrics: Arc::new(RwLock::new(LearningSystemMetrics::new())),
            resource_metrics: Arc::new(RwLock::new(ResourceUtilizationMetrics::new())),
            thresholds: Arc::new(RwLock::new(None)),
            historical_metrics: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record API request metrics
    pub async fn record_api_request(
        &self,
        method: &str,
        path: &str,
        status_code: u16,
        duration: Duration,
    ) {
        let mut metrics = self.api_metrics.write().await;
        metrics.record_request(method, path, status_code, duration);
    }

    /// Record provider performance metrics
    pub async fn record_provider_metrics(&self, metrics: &ProviderPerformanceMetrics) {
        let mut provider_metrics = self.provider_metrics.write().await;
        provider_metrics.insert(metrics.provider_name.clone(), metrics.clone());
    }

    /// Record quality processing metrics
    pub async fn record_quality_processing(
        &self,
        operation_type: &str,
        processing_time: Duration,
        tokens_processed: usize,
    ) {
        let mut metrics = self.quality_metrics.write().await;
        metrics.record_evaluation(operation_type, processing_time, tokens_processed);
    }

    /// Record cache operation metrics
    pub async fn record_cache_operation(
        &self,
        cache_name: &str,
        operation: CacheOperation,
        duration: Duration,
    ) {
        let mut cache_metrics = self.cache_metrics.write().await;
        let metrics = cache_metrics
            .entry(cache_name.to_string())
            .or_insert_with(|| CacheMetrics::new(cache_name.to_string()));
        metrics.record_operation(operation, duration);
    }

    /// Record learning system metrics
    pub async fn record_learning_metrics(&self, metrics: &LearningSystemMetrics) {
        let mut learning_metrics = self.learning_metrics.write().await;
        *learning_metrics = metrics.clone();
    }

    /// Record resource utilization metrics
    pub async fn record_resource_metrics(&self, metrics: &ResourceUtilizationMetrics) {
        let mut resource_metrics = self.resource_metrics.write().await;
        *resource_metrics = metrics.clone();
    }

    /// Get API metrics
    pub async fn get_api_metrics(&self) -> MonitoringResult<ApiMetrics> {
        let metrics = self.api_metrics.read().await;
        Ok(metrics.clone())
    }

    /// Get provider metrics for specific provider
    pub async fn get_provider_metrics(
        &self,
        provider_name: &str,
    ) -> MonitoringResult<ProviderPerformanceMetrics> {
        let metrics = self.provider_metrics.read().await;
        metrics.get(provider_name).cloned().ok_or_else(|| {
            MonitoringError::MetricsError(format!("Provider metrics not found: {provider_name}"))
        })
    }

    /// Get quality processing metrics
    pub async fn get_quality_metrics(&self) -> MonitoringResult<QualityMetrics> {
        let metrics = self.quality_metrics.read().await;
        Ok(metrics.clone())
    }

    /// Get cache metrics for specific cache
    pub async fn get_cache_metrics(&self, cache_name: &str) -> MonitoringResult<CacheMetrics> {
        let metrics = self.cache_metrics.read().await;
        metrics.get(cache_name).cloned().ok_or_else(|| {
            MonitoringError::MetricsError(format!("Cache metrics not found: {cache_name}"))
        })
    }

    /// Get learning system metrics
    pub async fn get_learning_metrics(&self) -> MonitoringResult<LearningSystemMetrics> {
        let metrics = self.learning_metrics.read().await;
        Ok(metrics.clone())
    }

    /// Get resource utilization metrics
    pub async fn get_resource_metrics(&self) -> MonitoringResult<ResourceUtilizationMetrics> {
        let metrics = self.resource_metrics.read().await;
        Ok(metrics.clone())
    }

    /// Set performance thresholds
    pub async fn set_thresholds(&mut self, thresholds: PerformanceThresholds) {
        let mut threshold_guard = self.thresholds.write().await;
        *threshold_guard = Some(thresholds);
    }

    /// Check for threshold violations
    pub async fn check_threshold_violations(&self) -> MonitoringResult<Vec<ThresholdViolation>> {
        let thresholds_guard = self.thresholds.read().await;
        let thresholds = match thresholds_guard.as_ref() {
            Some(t) => t,
            None => return Ok(Vec::new()),
        };

        let mut violations = Vec::new();

        // Check API response time violations
        let api_metrics = self.api_metrics.read().await;
        if api_metrics.average_response_time > thresholds.max_response_time {
            violations.push(ThresholdViolation {
                metric_type: "response_time".to_string(),
                threshold_value: thresholds.max_response_time.as_millis() as f64,
                actual_value: api_metrics.average_response_time.as_millis() as f64,
                severity: ViolationSeverity::High,
                timestamp: Utc::now(),
            });
        }

        // Check error rate violations
        if api_metrics.error_rate() > thresholds.max_error_rate {
            violations.push(ThresholdViolation {
                metric_type: "error_rate".to_string(),
                threshold_value: thresholds.max_error_rate,
                actual_value: api_metrics.error_rate(),
                severity: ViolationSeverity::Critical,
                timestamp: Utc::now(),
            });
        }

        // Check cache hit rate violations
        let cache_metrics = self.cache_metrics.read().await;
        for (cache_name, metrics) in cache_metrics.iter() {
            if metrics.hit_rate < thresholds.min_cache_hit_rate {
                violations.push(ThresholdViolation {
                    metric_type: format!("cache_hit_rate_{cache_name}"),
                    threshold_value: thresholds.min_cache_hit_rate,
                    actual_value: metrics.hit_rate,
                    severity: ViolationSeverity::Medium,
                    timestamp: Utc::now(),
                });
            }
        }

        // Check resource utilization violations
        let resource_metrics = self.resource_metrics.read().await;
        if resource_metrics.cpu_usage_percent > thresholds.max_cpu_usage {
            violations.push(ThresholdViolation {
                metric_type: "cpu_usage".to_string(),
                threshold_value: thresholds.max_cpu_usage,
                actual_value: resource_metrics.cpu_usage_percent,
                severity: ViolationSeverity::High,
                timestamp: Utc::now(),
            });
        }

        if resource_metrics.memory_usage_bytes > thresholds.max_memory_usage {
            violations.push(ThresholdViolation {
                metric_type: "memory_usage".to_string(),
                threshold_value: thresholds.max_memory_usage as f64,
                actual_value: resource_metrics.memory_usage_bytes as f64,
                severity: ViolationSeverity::High,
                timestamp: Utc::now(),
            });
        }

        Ok(violations)
    }

    /// Generate comprehensive performance report
    pub async fn generate_performance_report(&self) -> MonitoringResult<PerformanceReport> {
        let api_metrics = self.api_metrics.read().await;
        let provider_metrics = self.provider_metrics.read().await;
        let quality_metrics = self.quality_metrics.read().await;
        let cache_metrics = self.cache_metrics.read().await;
        let learning_metrics = self.learning_metrics.read().await;
        let resource_metrics = self.resource_metrics.read().await;

        let violations = self.check_threshold_violations().await?;

        Ok(PerformanceReport {
            timestamp: Utc::now(),
            total_requests: api_metrics.total_requests,
            successful_requests: api_metrics.successful_requests,
            failed_requests: api_metrics.failed_requests,
            average_response_time: api_metrics.average_response_time,
            p95_response_time: api_metrics.p95_response_time,
            error_rate: api_metrics.error_rate(),
            providers: provider_metrics.clone(),
            quality_processing: quality_metrics.clone(),
            cache_performance: cache_metrics.clone(),
            learning_system: learning_metrics.clone(),
            resource_utilization: resource_metrics.clone(),
            threshold_violations: violations,
        })
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// API request and response metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: Duration,
    pub p95_response_time: Duration,
    pub requests_by_method: HashMap<String, u64>,
    pub requests_by_path: HashMap<String, u64>,
    pub response_times: Vec<Duration>,
    pub last_request_time: Option<DateTime<Utc>>,
}

impl ApiMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: Duration::default(),
            p95_response_time: Duration::default(),
            requests_by_method: HashMap::new(),
            requests_by_path: HashMap::new(),
            response_times: Vec::new(),
            last_request_time: None,
        }
    }

    pub fn record_request(
        &mut self,
        method: &str,
        path: &str,
        status_code: u16,
        duration: Duration,
    ) {
        self.total_requests += 1;

        if status_code < 400 {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }

        // Update method and path counters
        *self
            .requests_by_method
            .entry(method.to_string())
            .or_insert(0) += 1;
        *self.requests_by_path.entry(path.to_string()).or_insert(0) += 1;

        // Record response time
        self.response_times.push(duration);

        // Keep only last 1000 response times for memory efficiency
        if self.response_times.len() > 1000 {
            self.response_times.remove(0);
        }

        // Update average response time
        let total_time: Duration = self.response_times.iter().sum();
        self.average_response_time = total_time / self.response_times.len() as u32;

        // Calculate 95th percentile
        if !self.response_times.is_empty() {
            let mut sorted_times = self.response_times.clone();
            sorted_times.sort();
            let p95_index = (sorted_times.len() as f64 * 0.95) as usize;
            self.p95_response_time = sorted_times
                .get(p95_index.min(sorted_times.len() - 1))
                .copied()
                .unwrap_or_default();
        }

        self.last_request_time = Some(Utc::now());
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.failed_requests as f64 / self.total_requests as f64
        }
    }
}

impl Default for ApiMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Provider-specific performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPerformanceMetrics {
    pub provider_name: String,
    pub request_count: u64,
    pub success_rate: f64,
    pub average_latency: Duration,
    pub error_count: u64,
    pub last_success_time: DateTime<Utc>,
}

impl ProviderPerformanceMetrics {
    pub fn new(provider_name: String) -> Self {
        Self {
            provider_name,
            request_count: 0,
            success_rate: 1.0,
            average_latency: Duration::default(),
            error_count: 0,
            last_success_time: Utc::now(),
        }
    }
}

/// Quality processing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub total_evaluations: u64,
    pub average_processing_time: Duration,
    pub total_tokens_processed: u64,
    pub evaluations_by_type: HashMap<String, u64>,
    pub processing_times: Vec<Duration>,
}

impl QualityMetrics {
    pub fn new() -> Self {
        Self {
            total_evaluations: 0,
            average_processing_time: Duration::default(),
            total_tokens_processed: 0,
            evaluations_by_type: HashMap::new(),
            processing_times: Vec::new(),
        }
    }

    pub fn record_evaluation(
        &mut self,
        operation_type: &str,
        processing_time: Duration,
        tokens: usize,
    ) {
        self.total_evaluations += 1;
        self.total_tokens_processed += tokens as u64;

        *self
            .evaluations_by_type
            .entry(operation_type.to_string())
            .or_insert(0) += 1;

        self.processing_times.push(processing_time);

        // Keep only last 1000 processing times
        if self.processing_times.len() > 1000 {
            self.processing_times.remove(0);
        }

        // Update average processing time
        let total_time: Duration = self.processing_times.iter().sum();
        self.average_processing_time = total_time / self.processing_times.len() as u32;
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheOperation {
    Hit,
    Miss,
    Write,
    Eviction,
}

/// Cache performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub cache_name: String,
    pub total_operations: u64,
    pub hit_count: u64,
    pub miss_count: u64,
    pub write_count: u64,
    pub eviction_count: u64,
    pub hit_rate: f64,
    pub average_hit_time: Duration,
    pub average_miss_time: Duration,
    pub hit_times: Vec<Duration>,
    pub miss_times: Vec<Duration>,
}

impl CacheMetrics {
    pub fn new(cache_name: String) -> Self {
        Self {
            cache_name,
            total_operations: 0,
            hit_count: 0,
            miss_count: 0,
            write_count: 0,
            eviction_count: 0,
            hit_rate: 0.0,
            average_hit_time: Duration::default(),
            average_miss_time: Duration::default(),
            hit_times: Vec::new(),
            miss_times: Vec::new(),
        }
    }

    pub fn record_operation(&mut self, operation: CacheOperation, duration: Duration) {
        self.total_operations += 1;

        match operation {
            CacheOperation::Hit => {
                self.hit_count += 1;
                self.hit_times.push(duration);

                // Keep only last 500 hit times
                if self.hit_times.len() > 500 {
                    self.hit_times.remove(0);
                }

                if !self.hit_times.is_empty() {
                    let total_time: Duration = self.hit_times.iter().sum();
                    self.average_hit_time = total_time / self.hit_times.len() as u32;
                }
            }
            CacheOperation::Miss => {
                self.miss_count += 1;
                self.miss_times.push(duration);

                // Keep only last 500 miss times
                if self.miss_times.len() > 500 {
                    self.miss_times.remove(0);
                }

                if !self.miss_times.is_empty() {
                    let total_time: Duration = self.miss_times.iter().sum();
                    self.average_miss_time = total_time / self.miss_times.len() as u32;
                }
            }
            CacheOperation::Write => {
                self.write_count += 1;
            }
            CacheOperation::Eviction => {
                self.eviction_count += 1;
            }
        }

        // Update hit rate
        if self.hit_count + self.miss_count > 0 {
            self.hit_rate = self.hit_count as f64 / (self.hit_count + self.miss_count) as f64;
        }
    }
}

/// Learning system performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSystemMetrics {
    pub feedback_processed: u64,
    pub patterns_recognized: u64,
    pub adaptations_applied: u64,
    pub learning_accuracy: f64,
    pub processing_time: Duration,
}

impl LearningSystemMetrics {
    pub fn new() -> Self {
        Self {
            feedback_processed: 0,
            patterns_recognized: 0,
            adaptations_applied: 0,
            learning_accuracy: 0.0,
            processing_time: Duration::default(),
        }
    }
}

impl Default for LearningSystemMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilizationMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub network_bytes_sent: u64,
    pub network_bytes_received: u64,
    pub disk_io_bytes: u64,
    pub timestamp: DateTime<Utc>,
}

impl ResourceUtilizationMetrics {
    pub fn new() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_bytes: 0,
            network_bytes_sent: 0,
            network_bytes_received: 0,
            disk_io_bytes: 0,
            timestamp: Utc::now(),
        }
    }
}

impl Default for ResourceUtilizationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance thresholds for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub max_response_time: Duration,
    pub max_error_rate: f64,
    pub min_cache_hit_rate: f64,
    pub max_cpu_usage: f64,
    pub max_memory_usage: u64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_response_time: Duration::from_millis(200),
            max_error_rate: 0.05,                 // 5%
            min_cache_hit_rate: 0.8,              // 80%
            max_cpu_usage: 80.0,                  // 80%
            max_memory_usage: 1024 * 1024 * 1024, // 1GB
        }
    }
}

/// Threshold violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdViolation {
    pub metric_type: String,
    pub threshold_value: f64,
    pub actual_value: f64,
    pub severity: ViolationSeverity,
    pub timestamp: DateTime<Utc>,
}

/// Severity levels for threshold violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Comprehensive performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub timestamp: DateTime<Utc>,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: Duration,
    pub p95_response_time: Duration,
    pub error_rate: f64,
    pub providers: HashMap<String, ProviderPerformanceMetrics>,
    pub quality_processing: QualityMetrics,
    pub cache_performance: HashMap<String, CacheMetrics>,
    pub learning_system: LearningSystemMetrics,
    pub resource_utilization: ResourceUtilizationMetrics,
    pub threshold_violations: Vec<ThresholdViolation>,
}

/// Snapshot of metrics at a specific point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: DateTime<Utc>,
    pub api_metrics: ApiMetrics,
    pub provider_metrics: HashMap<String, ProviderPerformanceMetrics>,
    pub quality_metrics: QualityMetrics,
    pub cache_metrics: HashMap<String, CacheMetrics>,
    pub learning_metrics: LearningSystemMetrics,
    pub resource_metrics: ResourceUtilizationMetrics,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();

        let api_metrics = collector.get_api_metrics().await.unwrap();
        assert_eq!(api_metrics.total_requests, 0);
        assert_eq!(api_metrics.successful_requests, 0);
        assert_eq!(api_metrics.failed_requests, 0);
    }

    #[tokio::test]
    async fn test_api_metrics_recording() {
        let collector = MetricsCollector::new();

        collector
            .record_api_request("GET", "/test", 200, Duration::from_millis(100))
            .await;
        collector
            .record_api_request("POST", "/test", 201, Duration::from_millis(150))
            .await;
        collector
            .record_api_request("GET", "/test", 500, Duration::from_millis(200))
            .await;

        let api_metrics = collector.get_api_metrics().await.unwrap();
        assert_eq!(api_metrics.total_requests, 3);
        assert_eq!(api_metrics.successful_requests, 2);
        assert_eq!(api_metrics.failed_requests, 1);
        assert!((api_metrics.error_rate() - 0.3333).abs() < 0.01);
        assert!(api_metrics.average_response_time >= Duration::from_millis(100));
        assert!(api_metrics.average_response_time <= Duration::from_millis(200));
    }

    #[tokio::test]
    async fn test_cache_metrics_recording() {
        let collector = MetricsCollector::new();

        collector
            .record_cache_operation(
                "test_cache",
                CacheOperation::Hit,
                Duration::from_micros(100),
            )
            .await;
        collector
            .record_cache_operation(
                "test_cache",
                CacheOperation::Hit,
                Duration::from_micros(120),
            )
            .await;
        collector
            .record_cache_operation(
                "test_cache",
                CacheOperation::Miss,
                Duration::from_millis(10),
            )
            .await;

        let cache_metrics = collector.get_cache_metrics("test_cache").await.unwrap();
        assert_eq!(cache_metrics.total_operations, 3);
        assert_eq!(cache_metrics.hit_count, 2);
        assert_eq!(cache_metrics.miss_count, 1);
        assert!((cache_metrics.hit_rate - 0.6667).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_threshold_violations() {
        let mut collector = MetricsCollector::new();

        let thresholds = PerformanceThresholds {
            max_response_time: Duration::from_millis(100),
            max_error_rate: 0.1,
            min_cache_hit_rate: 0.8,
            max_cpu_usage: 70.0,
            max_memory_usage: 512 * 1024 * 1024, // 512MB
        };

        collector.set_thresholds(thresholds).await;

        // Record metrics that violate thresholds
        collector
            .record_api_request("GET", "/test", 500, Duration::from_millis(200))
            .await;

        let violations = collector.check_threshold_violations().await.unwrap();

        // Should have violations for response time and error rate
        assert!(!violations.is_empty());

        let response_time_violation = violations
            .iter()
            .find(|v| v.metric_type == "response_time")
            .expect("Should have response time violation");

        assert_eq!(response_time_violation.threshold_value, 100.0);
        assert_eq!(response_time_violation.actual_value, 200.0);
    }

    #[tokio::test]
    async fn test_performance_report_generation() {
        let collector = MetricsCollector::new();

        // Record some sample metrics
        collector
            .record_api_request("GET", "/test", 200, Duration::from_millis(100))
            .await;

        let provider_metrics = ProviderPerformanceMetrics {
            provider_name: "test_provider".to_string(),
            request_count: 5,
            success_rate: 0.8,
            average_latency: Duration::from_millis(120),
            error_count: 1,
            last_success_time: Utc::now(),
        };
        collector.record_provider_metrics(&provider_metrics).await;

        let report = collector.generate_performance_report().await.unwrap();

        assert_eq!(report.total_requests, 1);
        assert_eq!(report.successful_requests, 1);
        assert_eq!(report.failed_requests, 0);
        assert!(report.providers.contains_key("test_provider"));
    }

    #[test]
    fn test_api_metrics_error_rate_calculation() {
        let mut metrics = ApiMetrics::new();

        // Test with no requests
        assert_eq!(metrics.error_rate(), 0.0);

        // Record successful request
        metrics.record_request("GET", "/test", 200, Duration::from_millis(100));
        assert_eq!(metrics.error_rate(), 0.0);

        // Record failed request
        metrics.record_request("GET", "/test", 500, Duration::from_millis(100));
        assert!((metrics.error_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_cache_metrics_hit_rate_calculation() {
        let mut metrics = CacheMetrics::new("test_cache".to_string());

        // Test with no operations
        assert_eq!(metrics.hit_rate, 0.0);

        // Record hit
        metrics.record_operation(CacheOperation::Hit, Duration::from_micros(100));
        assert_eq!(metrics.hit_rate, 1.0);

        // Record miss
        metrics.record_operation(CacheOperation::Miss, Duration::from_millis(10));
        assert!((metrics.hit_rate - 0.5).abs() < 0.01);

        // Record another hit
        metrics.record_operation(CacheOperation::Hit, Duration::from_micros(150));
        assert!((metrics.hit_rate - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_performance_thresholds_default() {
        let thresholds = PerformanceThresholds::default();

        assert_eq!(thresholds.max_response_time, Duration::from_millis(200));
        assert_eq!(thresholds.max_error_rate, 0.05);
        assert_eq!(thresholds.min_cache_hit_rate, 0.8);
        assert_eq!(thresholds.max_cpu_usage, 80.0);
        assert_eq!(thresholds.max_memory_usage, 1024 * 1024 * 1024);
    }
}
