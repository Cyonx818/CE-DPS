//! Tests for monitoring metrics collection system
//!
//! These tests verify the core performance metrics collection functionality
//! following the test-driven development approach defined in AI_RULES.md

use fortitude::monitoring::*;
use std::time::Duration;

#[tokio::test]
async fn test_api_request_metrics_collection() {
    let metrics_collector = MetricsCollector::new();

    // Record API request metrics
    let start_time = std::time::Instant::now();
    tokio::time::sleep(Duration::from_millis(50)).await; // Simulate processing
    let duration = start_time.elapsed();

    metrics_collector
        .record_api_request("GET", "/api/research", 200, duration)
        .await;

    // Verify metrics were recorded
    let api_metrics = metrics_collector.get_api_metrics().await.unwrap();
    assert_eq!(api_metrics.total_requests, 1);
    assert_eq!(api_metrics.successful_requests, 1);
    assert!(api_metrics.average_response_time >= Duration::from_millis(50));
    assert!(api_metrics.average_response_time < Duration::from_millis(200));
}

#[tokio::test]
async fn test_provider_performance_metrics_collection() {
    let metrics_collector = MetricsCollector::new();

    // Record provider performance metrics
    let provider_metrics = ProviderPerformanceMetrics {
        provider_name: "claude".to_string(),
        request_count: 1,
        success_rate: 1.0,
        average_latency: Duration::from_millis(150),
        error_count: 0,
        last_success_time: chrono::Utc::now(),
    };

    metrics_collector
        .record_provider_metrics(&provider_metrics)
        .await;

    // Verify provider metrics were recorded
    let recorded_metrics = metrics_collector
        .get_provider_metrics("claude")
        .await
        .unwrap();
    assert_eq!(recorded_metrics.provider_name, "claude");
    assert_eq!(recorded_metrics.request_count, 1);
    assert_eq!(recorded_metrics.success_rate, 1.0);
    assert_eq!(recorded_metrics.average_latency, Duration::from_millis(150));
}

#[tokio::test]
async fn test_quality_processing_overhead_metrics() {
    let metrics_collector = MetricsCollector::new();

    // Record quality processing metrics
    let start_time = std::time::Instant::now();
    tokio::time::sleep(Duration::from_millis(75)).await; // Simulate quality processing
    let processing_time = start_time.elapsed();

    metrics_collector
        .record_quality_processing("accuracy_check", processing_time, 512)
        .await;

    // Verify quality metrics were recorded
    let quality_metrics = metrics_collector.get_quality_metrics().await.unwrap();
    assert_eq!(quality_metrics.total_evaluations, 1);
    assert!(quality_metrics.average_processing_time >= Duration::from_millis(75));
    assert!(quality_metrics.average_processing_time < Duration::from_millis(200));
    assert_eq!(quality_metrics.total_tokens_processed, 512);
}

#[tokio::test]
async fn test_cache_hit_rates_and_performance() {
    let metrics_collector = MetricsCollector::new();

    // Record cache operations
    metrics_collector
        .record_cache_operation(
            "vector_embeddings",
            CacheOperation::Hit,
            Duration::from_micros(500),
        )
        .await;
    metrics_collector
        .record_cache_operation(
            "vector_embeddings",
            CacheOperation::Miss,
            Duration::from_millis(10),
        )
        .await;
    metrics_collector
        .record_cache_operation(
            "vector_embeddings",
            CacheOperation::Hit,
            Duration::from_micros(750),
        )
        .await;

    // Verify cache metrics
    let cache_metrics = metrics_collector
        .get_cache_metrics("vector_embeddings")
        .await
        .unwrap();
    assert_eq!(cache_metrics.total_operations, 3);
    assert_eq!(cache_metrics.hit_count, 2);
    assert_eq!(cache_metrics.miss_count, 1);
    assert!((cache_metrics.hit_rate - 0.6667).abs() < 0.01);
    assert!(cache_metrics.average_hit_time < Duration::from_millis(1));
    assert!(cache_metrics.average_miss_time > Duration::from_millis(5));
}

#[tokio::test]
async fn test_learning_system_metrics_collection() {
    let metrics_collector = MetricsCollector::new();

    // Record learning system metrics
    let learning_metrics = LearningSystemMetrics {
        feedback_processed: 15,
        patterns_recognized: 3,
        adaptations_applied: 1,
        learning_accuracy: 0.87,
        processing_time: Duration::from_millis(200),
    };

    metrics_collector
        .record_learning_metrics(&learning_metrics)
        .await;

    // Verify learning metrics were recorded
    let recorded_metrics = metrics_collector.get_learning_metrics().await.unwrap();
    assert_eq!(recorded_metrics.feedback_processed, 15);
    assert_eq!(recorded_metrics.patterns_recognized, 3);
    assert_eq!(recorded_metrics.adaptations_applied, 1);
    assert_eq!(recorded_metrics.learning_accuracy, 0.87);
    assert_eq!(recorded_metrics.processing_time, Duration::from_millis(200));
}

#[tokio::test]
async fn test_resource_utilization_metrics_collection() {
    let metrics_collector = MetricsCollector::new();

    // Record resource utilization
    let resource_metrics = ResourceUtilizationMetrics {
        cpu_usage_percent: 25.5,
        memory_usage_bytes: 512 * 1024 * 1024,   // 512MB
        network_bytes_sent: 1024 * 1024,         // 1MB
        network_bytes_received: 2 * 1024 * 1024, // 2MB
        disk_io_bytes: 10 * 1024 * 1024,         // 10MB
        timestamp: chrono::Utc::now(),
    };

    metrics_collector
        .record_resource_metrics(&resource_metrics)
        .await;

    // Verify resource metrics were recorded
    let recorded_metrics = metrics_collector.get_resource_metrics().await.unwrap();
    assert_eq!(recorded_metrics.cpu_usage_percent, 25.5);
    assert_eq!(recorded_metrics.memory_usage_bytes, 512 * 1024 * 1024);
    assert_eq!(recorded_metrics.network_bytes_sent, 1024 * 1024);
    assert_eq!(recorded_metrics.network_bytes_received, 2 * 1024 * 1024);
    assert_eq!(recorded_metrics.disk_io_bytes, 10 * 1024 * 1024);
}

#[tokio::test]
async fn test_metrics_aggregation_and_reporting() {
    let metrics_collector = MetricsCollector::new();

    // Record multiple metrics over time
    for i in 0..5 {
        let duration = Duration::from_millis(100 + i * 10);
        metrics_collector
            .record_api_request("GET", "/api/research", 200, duration)
            .await;
    }

    // Get aggregated metrics
    let report = metrics_collector
        .generate_performance_report()
        .await
        .unwrap();

    // Verify aggregation
    assert_eq!(report.total_requests, 5);
    assert!(report.average_response_time >= Duration::from_millis(100));
    assert!(report.average_response_time <= Duration::from_millis(140));
    assert!(report.p95_response_time >= Duration::from_millis(130));
    assert_eq!(report.error_rate, 0.0);
}

#[tokio::test]
async fn test_performance_threshold_monitoring() {
    let mut metrics_collector = MetricsCollector::new();

    // Set performance thresholds
    let thresholds = PerformanceThresholds {
        max_response_time: Duration::from_millis(200),
        max_error_rate: 0.05,
        min_cache_hit_rate: 0.8,
        max_cpu_usage: 80.0,
        max_memory_usage: 1024 * 1024 * 1024, // 1GB
    };

    metrics_collector.set_thresholds(thresholds).await;

    // Record metrics that exceed thresholds
    metrics_collector
        .record_api_request("POST", "/api/classify", 500, Duration::from_millis(300))
        .await;

    // Check for threshold violations
    let violations = metrics_collector
        .check_threshold_violations()
        .await
        .unwrap();
    assert!(!violations.is_empty());

    let response_time_violation = violations
        .iter()
        .find(|v| v.metric_type == "response_time")
        .expect("Should have response time violation");

    assert_eq!(response_time_violation.threshold_value, 200.0);
    assert_eq!(response_time_violation.actual_value, 300.0);
    assert_eq!(response_time_violation.severity, ViolationSeverity::High);
}
