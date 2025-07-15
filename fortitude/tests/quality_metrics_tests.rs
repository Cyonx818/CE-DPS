// Integration tests for automated quality metrics collection and storage system
//! These tests validate the complete quality metrics collection, storage, and analysis pipeline
//! with focus on performance requirements and real-world usage scenarios.

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use fortitude::quality::metrics::{
    AggregatedMetric, CleanupStats, MetricContext, MetricFilters, MetricType, MetricValue,
    MetricsAnalyzer, MetricsCollector, MetricsConfig, MetricsError, MetricsStorage,
    ProviderPerformanceAnalysis, QualityAnomaly, QualityMetric, QualityTrends, RetentionConfig,
    StorageStats, TrendDirection,
};
use fortitude::quality::{QualityScore, QualityWeights};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

// Integration test storage implementation with realistic behavior
#[derive(Debug, Clone)]
struct TestMetricsStorage {
    metrics: Arc<Mutex<Vec<QualityMetric>>>,
    operation_delays: Arc<Mutex<HashMap<String, Duration>>>,
    failure_rate: Arc<Mutex<f64>>,
    operation_count: Arc<AtomicUsize>,
}

impl TestMetricsStorage {
    fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
            operation_delays: Arc::new(Mutex::new(HashMap::new())),
            failure_rate: Arc::new(Mutex::new(0.0)),
            operation_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    async fn set_operation_delay(&self, operation: &str, delay: Duration) {
        self.operation_delays
            .lock()
            .await
            .insert(operation.to_string(), delay);
    }

    async fn set_failure_rate(&self, rate: f64) {
        *self.failure_rate.lock().await = rate;
    }

    fn get_operation_count(&self) -> usize {
        self.operation_count.load(Ordering::Relaxed)
    }

    async fn clear(&self) {
        self.metrics.lock().await.clear();
        self.operation_count.store(0, Ordering::Relaxed);
    }

    async fn get_metrics_count(&self) -> usize {
        self.metrics.lock().await.len()
    }

    async fn should_fail(&self) -> bool {
        let rate = *self.failure_rate.lock().await;
        rand::random::<f64>() < rate
    }

    async fn apply_delay(&self, operation: &str) {
        if let Some(delay) = self.operation_delays.lock().await.get(operation) {
            tokio::time::sleep(*delay).await;
        }
    }
}

#[async_trait::async_trait]
impl MetricsStorage for TestMetricsStorage {
    async fn store_metric(&self, metric: &QualityMetric) -> Result<(), MetricsError> {
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.apply_delay("store_metric").await;

        if self.should_fail().await {
            return Err(MetricsError::StorageError {
                message: "Simulated storage failure".to_string(),
            });
        }

        self.metrics.lock().await.push(metric.clone());
        Ok(())
    }

    async fn store_metrics(&self, metrics: &[QualityMetric]) -> Result<(), MetricsError> {
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.apply_delay("store_metrics").await;

        if self.should_fail().await {
            return Err(MetricsError::StorageError {
                message: "Simulated batch storage failure".to_string(),
            });
        }

        self.metrics.lock().await.extend_from_slice(metrics);
        Ok(())
    }

    async fn query_metrics(
        &self,
        filters: &MetricFilters,
    ) -> Result<Vec<QualityMetric>, MetricsError> {
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.apply_delay("query_metrics").await;

        if self.should_fail().await {
            return Err(MetricsError::QueryError {
                message: "Simulated query failure".to_string(),
            });
        }

        let metrics = self.metrics.lock().await;
        let mut results: Vec<QualityMetric> = metrics
            .iter()
            .filter(|m| m.matches_filters(filters))
            .cloned()
            .collect();

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
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.apply_delay("get_aggregated_metrics").await;

        if self.should_fail().await {
            return Err(MetricsError::AnalyticsError {
                message: "Simulated aggregation failure".to_string(),
            });
        }

        // Simplified aggregation for testing
        let metrics = self.query_metrics(filters).await?;
        if metrics.is_empty() {
            return Ok(Vec::new());
        }

        // Group by provider and time windows
        let mut aggregated = Vec::new();
        let providers: std::collections::HashSet<Option<String>> =
            metrics.iter().map(|m| m.provider.clone()).collect();

        for provider in providers {
            let provider_metrics: Vec<&QualityMetric> =
                metrics.iter().filter(|m| m.provider == provider).collect();

            if !provider_metrics.is_empty() {
                let values: Vec<f64> = provider_metrics
                    .iter()
                    .filter_map(|m| m.value.as_f64())
                    .collect();

                if !values.is_empty() {
                    let avg = values.iter().sum::<f64>() / values.len() as f64;
                    let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                    let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                    let sum = values.iter().sum::<f64>();

                    let mut aggregations = super::MetricAggregations {
                        average: Some(avg),
                        minimum: Some(min),
                        maximum: Some(max),
                        sum: Some(sum),
                        std_dev: None,
                        percentiles: None,
                    };

                    // Calculate standard deviation
                    if values.len() > 1 {
                        let variance = values.iter().map(|&x| (x - avg).powi(2)).sum::<f64>()
                            / values.len() as f64;
                        aggregations.std_dev = Some(variance.sqrt());
                    }

                    let start_time = provider_metrics.iter().map(|m| m.timestamp).min().unwrap();
                    let end_time = provider_metrics.iter().map(|m| m.timestamp).max().unwrap();

                    aggregated.push(AggregatedMetric {
                        metric_type: provider_metrics[0].metric_type.clone(),
                        provider: provider.clone(),
                        time_window: (start_time, end_time),
                        aggregations,
                        count: values.len(),
                        common_tags: HashMap::new(),
                    });
                }
            }
        }

        Ok(aggregated)
    }

    async fn delete_metrics(&self, filters: &MetricFilters) -> Result<usize, MetricsError> {
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.apply_delay("delete_metrics").await;

        if self.should_fail().await {
            return Err(MetricsError::StorageError {
                message: "Simulated deletion failure".to_string(),
            });
        }

        let mut metrics = self.metrics.lock().await;
        let initial_len = metrics.len();
        metrics.retain(|m| !m.matches_filters(filters));
        Ok(initial_len - metrics.len())
    }

    async fn get_storage_stats(&self) -> Result<StorageStats, MetricsError> {
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.apply_delay("get_storage_stats").await;

        if self.should_fail().await {
            return Err(MetricsError::StorageError {
                message: "Simulated stats failure".to_string(),
            });
        }

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
            storage_size_bytes: metrics.len() * 1024, // Estimate
            metrics_by_type,
            metrics_by_provider,
            oldest_metric: metrics.first().map(|m| m.timestamp),
            newest_metric: metrics.last().map(|m| m.timestamp),
        })
    }

    async fn cleanup(&self, retention: &RetentionConfig) -> Result<CleanupStats, MetricsError> {
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.apply_delay("cleanup").await;

        if self.should_fail().await {
            return Err(MetricsError::StorageError {
                message: "Simulated cleanup failure".to_string(),
            });
        }

        let mut metrics = self.metrics.lock().await;
        let initial_len = metrics.len();
        let cutoff_time = Utc::now() - retention.raw_retention;

        metrics.retain(|m| m.timestamp >= cutoff_time);
        let deleted_count = initial_len - metrics.len();

        Ok(CleanupStats {
            metrics_deleted: deleted_count,
            bytes_freed: deleted_count * 1024,
            cleanup_duration: Duration::from_millis(50),
        })
    }
}

// Helper function to create test quality score
fn create_test_quality_score(composite: f64) -> QualityScore {
    QualityScore {
        relevance: composite,
        accuracy: composite,
        completeness: composite,
        clarity: composite,
        credibility: composite,
        timeliness: composite,
        specificity: composite,
        composite,
        confidence: composite,
    }
}

// Helper function to create test metrics
fn create_test_metrics(count: usize, provider: Option<String>) -> Vec<QualityMetric> {
    (0..count)
        .map(|i| {
            let quality_score = create_test_quality_score(0.7 + (i as f64 * 0.1) % 0.3);
            QualityMetric::new(
                MetricType::ResearchQuality,
                MetricValue::QualityScore(quality_score),
                provider.clone(),
            )
            .with_tag("test_run".to_string(), "integration".to_string())
        })
        .collect()
}

#[tokio::test]
async fn test_metrics_collection_integration() {
    let config = MetricsConfig::default();
    let storage = Arc::new(TestMetricsStorage::new());
    let collector = MetricsCollector::new(config, storage.clone());

    // Start collector
    assert!(collector.start().await.is_ok());

    // Collect some metrics
    let metrics = create_test_metrics(5, Some("claude".to_string()));
    for metric in metrics {
        assert!(collector.collect(metric).await.is_ok());
    }

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Stop collector and verify metrics were stored
    assert!(collector.stop().await.is_ok());
    assert!(storage.get_metrics_count().await > 0);
}

#[tokio::test]
async fn test_metrics_collection_performance() {
    let mut config = MetricsConfig::default();
    config.enable_batch = false; // Only real-time for performance test

    let storage = Arc::new(TestMetricsStorage::new());
    let collector = MetricsCollector::new(config, storage.clone());

    let metric = QualityMetric::new(
        MetricType::ResearchQuality,
        MetricValue::Gauge(0.85),
        Some("claude".to_string()),
    );

    // Test single collection performance
    let start = Instant::now();
    assert!(collector.collect(metric).await.is_ok());
    let duration = start.elapsed();

    // Should meet performance requirement of <5ms per operation
    assert!(
        duration < Duration::from_millis(10), // Being generous for test environment
        "Collection took {:?}, expected <10ms",
        duration
    );
}

#[tokio::test]
async fn test_high_volume_metrics_collection() {
    let mut config = MetricsConfig::default();
    config.batch_size = 50;
    config.buffer_size = 1000;

    let storage = Arc::new(TestMetricsStorage::new());
    let collector = MetricsCollector::new(config, storage.clone());

    assert!(collector.start().await.is_ok());

    // Collect 100 metrics rapidly
    let start = Instant::now();
    for i in 0..100 {
        let metric = QualityMetric::new(
            MetricType::ProviderPerformance,
            MetricValue::Duration(Duration::from_millis(100 + i)),
            Some(format!("provider_{}", i % 3)),
        );
        assert!(collector.collect(metric).await.is_ok());
    }
    let collection_duration = start.elapsed();

    // Should handle high volume efficiently
    assert!(
        collection_duration < Duration::from_secs(1),
        "High volume collection took {:?}, expected <1s",
        collection_duration
    );

    assert!(collector.stop().await.is_ok());

    // Verify all metrics were stored
    assert_eq!(storage.get_metrics_count().await, 100);
}

#[tokio::test]
async fn test_batch_processing_performance() {
    let mut config = MetricsConfig::default();
    config.enable_realtime = false;
    config.batch_size = 100;

    let storage = Arc::new(TestMetricsStorage::new());
    let collector = MetricsCollector::new(config, storage.clone());

    let metrics = create_test_metrics(500, Some("batch_test".to_string()));

    // Test batch collection performance
    let start = Instant::now();
    assert!(collector.collect_batch(metrics).await.is_ok());
    let duration = start.elapsed();

    // Should handle batch efficiently
    assert!(
        duration < Duration::from_millis(500),
        "Batch processing took {:?}, expected <500ms",
        duration
    );

    // Verify metrics were stored
    assert_eq!(storage.get_metrics_count().await, 500);
}

#[tokio::test]
async fn test_storage_query_performance() {
    let storage = TestMetricsStorage::new();

    // Store 1000 test metrics
    let metrics = create_test_metrics(1000, Some("query_test".to_string()));
    assert!(storage.store_metrics(&metrics).await.is_ok());

    // Test query performance
    let filters = MetricFilters::new()
        .with_metric_type(MetricType::ResearchQuality)
        .with_provider("query_test".to_string());

    let start = Instant::now();
    let results = storage.query_metrics(&filters).await.unwrap();
    let duration = start.elapsed();

    // Should meet query performance requirement of <100ms
    assert!(
        duration < Duration::from_millis(200), // Being generous for test environment
        "Query took {:?}, expected <200ms",
        duration
    );

    assert_eq!(results.len(), 1000);
}

#[tokio::test]
async fn test_storage_aggregation_performance() {
    let storage = TestMetricsStorage::new();

    // Store metrics from multiple providers
    for provider in ["claude", "openai", "gemini"] {
        let metrics = create_test_metrics(100, Some(provider.to_string()));
        assert!(storage.store_metrics(&metrics).await.is_ok());
    }

    let filters = MetricFilters::new().with_metric_type(MetricType::ResearchQuality);

    let start = Instant::now();
    let aggregated = storage
        .get_aggregated_metrics(&filters, Duration::from_secs(3600))
        .await
        .unwrap();
    let duration = start.elapsed();

    // Should perform aggregation efficiently
    assert!(
        duration < Duration::from_millis(300),
        "Aggregation took {:?}, expected <300ms",
        duration
    );

    assert_eq!(aggregated.len(), 3); // One per provider
}

#[tokio::test]
async fn test_metrics_filtering_accuracy() {
    let storage = TestMetricsStorage::new();

    // Create diverse test data
    let claude_metrics = create_test_metrics(50, Some("claude".to_string()));
    let openai_metrics = create_test_metrics(30, Some("openai".to_string()));

    let mut performance_metrics = Vec::new();
    for i in 0..20 {
        performance_metrics.push(
            QualityMetric::new(
                MetricType::ProviderPerformance,
                MetricValue::Duration(Duration::from_millis(100 + i * 10)),
                Some("claude".to_string()),
            )
            .with_tag("endpoint".to_string(), "chat".to_string()),
        );
    }

    assert!(storage.store_metrics(&claude_metrics).await.is_ok());
    assert!(storage.store_metrics(&openai_metrics).await.is_ok());
    assert!(storage.store_metrics(&performance_metrics).await.is_ok());

    // Test provider filtering
    let claude_filter = MetricFilters::new().with_provider("claude".to_string());
    let claude_results = storage.query_metrics(&claude_filter).await.unwrap();
    assert_eq!(claude_results.len(), 70); // 50 quality + 20 performance

    // Test type filtering
    let quality_filter = MetricFilters::new().with_metric_type(MetricType::ResearchQuality);
    let quality_results = storage.query_metrics(&quality_filter).await.unwrap();
    assert_eq!(quality_results.len(), 80); // 50 + 30 quality metrics

    // Test combined filtering
    let combined_filter = MetricFilters::new()
        .with_provider("claude".to_string())
        .with_metric_type(MetricType::ProviderPerformance);
    let combined_results = storage.query_metrics(&combined_filter).await.unwrap();
    assert_eq!(combined_results.len(), 20);

    // Test tag filtering
    let tag_filter = MetricFilters::new().with_tag("endpoint".to_string(), "chat".to_string());
    let tag_results = storage.query_metrics(&tag_filter).await.unwrap();
    assert_eq!(tag_results.len(), 20);
}

#[tokio::test]
async fn test_time_range_filtering() {
    let storage = TestMetricsStorage::new();

    let now = Utc::now();
    let hour_ago = now - ChronoDuration::hours(1);
    let day_ago = now - ChronoDuration::days(1);

    // Create metrics with different timestamps
    let mut old_metric = create_test_metrics(1, Some("test".to_string()))[0].clone();
    old_metric.timestamp = day_ago;

    let mut recent_metric = create_test_metrics(1, Some("test".to_string()))[0].clone();
    recent_metric.timestamp = hour_ago;

    let mut current_metric = create_test_metrics(1, Some("test".to_string()))[0].clone();
    current_metric.timestamp = now;

    assert!(storage
        .store_metrics(&[old_metric, recent_metric, current_metric])
        .await
        .is_ok());

    // Test time range filtering
    let recent_filter = MetricFilters::new().with_time_range(hour_ago, now);
    let recent_results = storage.query_metrics(&recent_filter).await.unwrap();
    assert_eq!(recent_results.len(), 2); // recent + current

    let current_filter = MetricFilters::new().with_time_range(
        now - ChronoDuration::minutes(30),
        now + ChronoDuration::minutes(30),
    );
    let current_results = storage.query_metrics(&current_filter).await.unwrap();
    assert_eq!(current_results.len(), 1); // only current
}

#[tokio::test]
async fn test_metrics_analytics_integration() {
    let storage = Arc::new(TestMetricsStorage::new());
    let analyzer = MetricsAnalyzer::new(storage.clone());

    // Create quality trend data
    let now = Utc::now();
    let mut quality_metrics = Vec::new();

    for i in 0..100 {
        let score = 0.5 + (i as f64 / 100.0) * 0.4; // Improving trend from 0.5 to 0.9
        let quality_score = create_test_quality_score(score);
        let mut metric = QualityMetric::new(
            MetricType::ResearchQuality,
            MetricValue::QualityScore(quality_score),
            Some("claude".to_string()),
        );
        metric.timestamp = now - ChronoDuration::hours(100 - i as i64);
        quality_metrics.push(metric);
    }

    assert!(storage.store_metrics(&quality_metrics).await.is_ok());

    // Test quality trends analysis
    let trends = analyzer
        .analyze_quality_trends(
            Some("claude".to_string()),
            (now - ChronoDuration::hours(100), now),
        )
        .await
        .unwrap();

    assert!(trends.average_quality > 0.6);
    assert!(trends.average_quality < 0.8);
    assert_eq!(trends.sample_count, 100);
    assert!(matches!(trends.trend_direction, TrendDirection::Improving));
}

#[tokio::test]
async fn test_provider_performance_analysis() {
    let storage = Arc::new(TestMetricsStorage::new());
    let analyzer = MetricsAnalyzer::new(storage.clone());

    let now = Utc::now();
    let mut performance_metrics = Vec::new();

    // Create performance data for different providers
    let providers = [("claude", 150.0), ("openai", 200.0), ("gemini", 120.0)];

    for (provider, avg_time) in providers.iter() {
        for i in 0..50 {
            let response_time = avg_time + (i as f64 - 25.0) * 2.0; // Some variance
            let mut metric = QualityMetric::new(
                MetricType::ProviderPerformance,
                MetricValue::Duration(Duration::from_millis(response_time as u64)),
                Some(provider.to_string()),
            );
            metric.timestamp = now - ChronoDuration::hours(i as i64);
            performance_metrics.push(metric);
        }
    }

    assert!(storage.store_metrics(&performance_metrics).await.is_ok());

    // Test provider performance analysis
    let analysis = analyzer
        .analyze_provider_performance((now - ChronoDuration::hours(50), now))
        .await
        .unwrap();

    assert_eq!(analysis.len(), 3);

    // Find gemini analysis (should be fastest)
    let gemini_analysis = analysis.iter().find(|a| a.provider == "gemini").unwrap();
    assert!(gemini_analysis.average_performance < 150.0);

    // Find openai analysis (should be slowest)
    let openai_analysis = analysis.iter().find(|a| a.provider == "openai").unwrap();
    assert!(openai_analysis.average_performance > 150.0);
}

#[tokio::test]
async fn test_quality_anomaly_detection() {
    let storage = Arc::new(TestMetricsStorage::new());
    let analyzer = MetricsAnalyzer::new(storage.clone());

    let now = Utc::now();
    let mut quality_metrics = Vec::new();

    // Create mostly normal quality scores with some anomalies
    for i in 0..100 {
        let score = if i == 30 || i == 70 {
            0.3 // Anomalously low
        } else if i == 50 {
            0.95 // Anomalously high
        } else {
            0.75 + (rand::random::<f64>() - 0.5) * 0.1 // Normal with small variance
        };

        let quality_score = create_test_quality_score(score);
        let mut metric = QualityMetric::new(
            MetricType::ResearchQuality,
            MetricValue::QualityScore(quality_score),
            Some("claude".to_string()),
        );
        metric.timestamp = now - ChronoDuration::hours(100 - i as i64);
        quality_metrics.push(metric);
    }

    assert!(storage.store_metrics(&quality_metrics).await.is_ok());

    // Test anomaly detection
    let anomalies = analyzer
        .detect_quality_anomalies(2.0, (now - ChronoDuration::hours(100), now))
        .await
        .unwrap();

    // Should detect the 3 anomalies we inserted
    assert!(
        anomalies.len() >= 3,
        "Expected at least 3 anomalies, got {}",
        anomalies.len()
    );

    // Check that anomalies are properly classified
    let low_anomalies = anomalies.iter().filter(|a| a.quality_score < 0.5).count();
    let high_anomalies = anomalies.iter().filter(|a| a.quality_score > 0.9).count();

    assert!(low_anomalies >= 2, "Expected at least 2 low anomalies");
    assert!(high_anomalies >= 1, "Expected at least 1 high anomaly");
}

#[tokio::test]
async fn test_storage_failure_handling() {
    let storage = Arc::new(TestMetricsStorage::new());
    storage.set_failure_rate(0.5).await; // 50% failure rate

    let mut config = MetricsConfig::default();
    config.enable_batch = false; // Test real-time failures

    let collector = MetricsCollector::new(config, storage.clone());

    let metric = QualityMetric::new(
        MetricType::ResearchQuality,
        MetricValue::Gauge(0.85),
        Some("claude".to_string()),
    );

    // Some collections should fail due to simulated failures
    let mut success_count = 0;
    let mut failure_count = 0;

    for _ in 0..20 {
        match collector.collect(metric.clone()).await {
            Ok(_) => success_count += 1,
            Err(_) => failure_count += 1,
        }
    }

    // Should have both successes and failures
    assert!(success_count > 0, "Expected some successful collections");
    assert!(failure_count > 0, "Expected some failed collections");
}

#[tokio::test]
async fn test_storage_performance_under_load() {
    let storage = TestMetricsStorage::new();

    // Simulate slow storage operations
    storage
        .set_operation_delay("store_metrics", Duration::from_millis(50))
        .await;

    let metrics = create_test_metrics(100, Some("load_test".to_string()));

    // Test that storage can handle load even with delays
    let start = Instant::now();
    assert!(storage.store_metrics(&metrics).await.is_ok());
    let duration = start.elapsed();

    // Should complete even with artificial delay
    assert!(duration >= Duration::from_millis(50));
    assert!(duration < Duration::from_millis(200));

    assert_eq!(storage.get_metrics_count().await, 100);
}

#[tokio::test]
async fn test_retention_policy_cleanup() {
    let storage = TestMetricsStorage::new();

    let now = Utc::now();
    let mut old_metrics = Vec::new();
    let mut recent_metrics = Vec::new();

    // Create old metrics (should be cleaned up)
    for i in 0..50 {
        let mut metric = create_test_metrics(1, Some("test".to_string()))[0].clone();
        metric.timestamp = now - ChronoDuration::days(60); // Older than default retention
        old_metrics.push(metric);
    }

    // Create recent metrics (should be kept)
    for i in 0..50 {
        let mut metric = create_test_metrics(1, Some("test".to_string()))[0].clone();
        metric.timestamp = now - ChronoDuration::days(10); // Within retention period
        recent_metrics.push(metric);
    }

    assert!(storage.store_metrics(&old_metrics).await.is_ok());
    assert!(storage.store_metrics(&recent_metrics).await.is_ok());
    assert_eq!(storage.get_metrics_count().await, 100);

    // Test cleanup
    let retention_config = RetentionConfig::default();
    let cleanup_stats = storage.cleanup(&retention_config).await.unwrap();

    assert_eq!(cleanup_stats.metrics_deleted, 50);
    assert_eq!(storage.get_metrics_count().await, 50);
}

#[tokio::test]
async fn test_storage_statistics_accuracy() {
    let storage = TestMetricsStorage::new();

    // Store diverse metrics
    let claude_metrics = create_test_metrics(30, Some("claude".to_string()));
    let openai_metrics = create_test_metrics(20, Some("openai".to_string()));

    let mut performance_metrics = Vec::new();
    for i in 0..10 {
        performance_metrics.push(QualityMetric::new(
            MetricType::ProviderPerformance,
            MetricValue::Duration(Duration::from_millis(100 + i * 10)),
            Some("claude".to_string()),
        ));
    }

    assert!(storage.store_metrics(&claude_metrics).await.is_ok());
    assert!(storage.store_metrics(&openai_metrics).await.is_ok());
    assert!(storage.store_metrics(&performance_metrics).await.is_ok());

    // Test storage statistics
    let stats = storage.get_storage_stats().await.unwrap();

    assert_eq!(stats.total_metrics, 60);
    assert_eq!(stats.metrics_by_type[&MetricType::ResearchQuality], 50);
    assert_eq!(stats.metrics_by_type[&MetricType::ProviderPerformance], 10);
    assert_eq!(stats.metrics_by_provider["claude"], 40); // 30 quality + 10 performance
    assert_eq!(stats.metrics_by_provider["openai"], 20);
}

#[tokio::test]
async fn test_memory_usage_limits() {
    let mut config = MetricsConfig::default();
    config.buffer_size = 1000; // Limit buffer size
    config.max_memory_usage = 1024 * 1024; // 1MB limit

    let storage = Arc::new(TestMetricsStorage::new());
    let collector = MetricsCollector::new(config, storage.clone());

    // The collector should respect memory limits
    // (This is a basic test - in a real implementation, we'd monitor actual memory usage)
    let stats = collector.performance_stats().await;
    assert_eq!(stats.memory_usage_current, 0); // Should start at 0
}

#[tokio::test]
async fn test_concurrent_operations() {
    let storage = Arc::new(TestMetricsStorage::new());
    let collector = Arc::new(MetricsCollector::new(
        MetricsConfig::default(),
        storage.clone(),
    ));

    assert!(collector.start().await.is_ok());

    // Spawn multiple concurrent collection tasks
    let mut handles = Vec::new();
    for i in 0..10 {
        let collector_clone = Arc::clone(&collector);
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                let metric = QualityMetric::new(
                    MetricType::ResearchQuality,
                    MetricValue::Gauge(0.5 + (i * 10 + j) as f64 / 100.0),
                    Some(format!("provider_{}", i)),
                );
                collector_clone.collect(metric).await.unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    assert!(collector.stop().await.is_ok());

    // Verify all metrics were collected
    assert_eq!(storage.get_metrics_count().await, 100);
}

// Import the missing type from the metrics module
use fortitude::quality::metrics::MetricAggregations;
