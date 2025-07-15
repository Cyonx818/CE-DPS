//! ANCHOR: Monitoring system functionality that protects critical performance monitoring capabilities
//!
//! These tests verify core monitoring functionality and must be maintained
//! as the system evolves. Do not delete these tests.

use fortitude::monitoring::alerts::{AlertManagerConfig, ConsoleAlertChannel};
use fortitude::monitoring::health::{BasicHealthCheck, HealthConfig};
use fortitude::monitoring::tracing::TracingConfig;
use fortitude::monitoring::*;
use std::time::Duration;

/// ANCHOR: Verifies end-to-end performance metrics collection workflow
/// Tests: Metrics recording, aggregation, and threshold violation detection
#[tokio::test]
async fn test_anchor_performance_metrics_collection_workflow() {
    let mut metrics_collector = MetricsCollector::new();

    // Set up performance thresholds
    let thresholds = PerformanceThresholds {
        max_response_time: Duration::from_millis(200),
        max_error_rate: 0.05,
        min_cache_hit_rate: 0.8,
        max_cpu_usage: 80.0,
        max_memory_usage: 1024 * 1024 * 1024, // 1GB
    };
    metrics_collector.set_thresholds(thresholds).await;

    // Record multiple types of metrics - ensure average exceeds threshold
    metrics_collector
        .record_api_request("GET", "/api/research", 200, Duration::from_millis(250))
        .await;
    metrics_collector
        .record_api_request("POST", "/api/classify", 201, Duration::from_millis(250))
        .await;
    metrics_collector
        .record_api_request("GET", "/api/health", 500, Duration::from_millis(300))
        .await; // Both time and error violation

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
            CacheOperation::Hit,
            Duration::from_micros(600),
        )
        .await;
    metrics_collector
        .record_cache_operation(
            "vector_embeddings",
            CacheOperation::Miss,
            Duration::from_millis(10),
        )
        .await;

    let provider_metrics = ProviderPerformanceMetrics {
        provider_name: "claude".to_string(),
        request_count: 10,
        success_rate: 0.9,
        average_latency: Duration::from_millis(180),
        error_count: 1,
        last_success_time: chrono::Utc::now(),
    };
    metrics_collector
        .record_provider_metrics(&provider_metrics)
        .await;

    // Generate comprehensive performance report
    let report = metrics_collector
        .generate_performance_report()
        .await
        .unwrap();

    // Verify critical monitoring functionality
    assert_eq!(report.total_requests, 3);
    assert_eq!(report.successful_requests, 2);
    assert_eq!(report.failed_requests, 1);
    assert!((report.error_rate - 0.3333).abs() < 0.01);

    assert!(report.providers.contains_key("claude"));
    let claude_metrics = &report.providers["claude"];
    assert_eq!(claude_metrics.request_count, 10);
    assert_eq!(claude_metrics.success_rate, 0.9);

    // Verify threshold violations are detected
    assert!(!report.threshold_violations.is_empty());
    let response_time_violation = report
        .threshold_violations
        .iter()
        .find(|v| v.metric_type == "response_time")
        .expect("Should detect response time violation");
    assert_eq!(response_time_violation.severity, ViolationSeverity::High);

    let error_rate_violation = report
        .threshold_violations
        .iter()
        .find(|v| v.metric_type == "error_rate")
        .expect("Should detect error rate violation");
    assert_eq!(error_rate_violation.severity, ViolationSeverity::Critical);
}

/// ANCHOR: Verifies health checking system workflow
/// Tests: Component health registration, status aggregation, critical status handling
#[tokio::test]
async fn test_anchor_health_checking_system_workflow() {
    let config = HealthConfig::default();
    let health_checker = HealthChecker::new(config);

    // Register multiple components with different health states
    let healthy_check = Box::new(BasicHealthCheck::new("api_server".to_string(), || {
        HealthStatus::Healthy
    }));
    health_checker
        .register_health_check("api_server".to_string(), healthy_check)
        .await
        .unwrap();

    let warning_check = Box::new(BasicHealthCheck::new("cache_system".to_string(), || {
        HealthStatus::Warning
    }));
    health_checker
        .register_health_check("cache_system".to_string(), warning_check)
        .await
        .unwrap();

    let critical_check = Box::new(BasicHealthCheck::new("database".to_string(), || {
        HealthStatus::Critical
    }));
    health_checker
        .register_health_check("database".to_string(), critical_check)
        .await
        .unwrap();

    // Perform health check and verify aggregation
    let report = health_checker.check_health().await.unwrap();

    // Critical health check functionality
    assert_eq!(report.overall_status, HealthStatus::Critical);
    assert_eq!(report.components.len(), 3);

    // Verify component health states
    let api_health = report
        .components
        .iter()
        .find(|c| c.component_name == "api_server")
        .expect("Should have api_server health");
    assert_eq!(api_health.status, HealthStatus::Healthy);

    let cache_health = report
        .components
        .iter()
        .find(|c| c.component_name == "cache_system")
        .expect("Should have cache_system health");
    assert_eq!(cache_health.status, HealthStatus::Warning);

    let db_health = report
        .components
        .iter()
        .find(|c| c.component_name == "database")
        .expect("Should have database health");
    assert_eq!(db_health.status, HealthStatus::Critical);

    // Verify critical component filtering
    let critical_components = report.critical_components();
    assert_eq!(critical_components.len(), 1);
    assert_eq!(critical_components[0].component_name, "database");

    let components_with_issues = report.components_with_issues();
    assert_eq!(components_with_issues.len(), 2); // Warning and Critical
}

/// ANCHOR: Verifies alert management system workflow
/// Tests: Alert routing, channel delivery, rate limiting, resolution
#[tokio::test]
async fn test_anchor_alert_management_system_workflow() {
    let config = AlertManagerConfig::default();
    let alert_manager = AlertManager::new(config);

    // Register alert channels
    let console_channel = Box::new(ConsoleAlertChannel::new("console".to_string()));
    alert_manager
        .register_channel("console".to_string(), console_channel)
        .await
        .unwrap();

    // Add alert routing rules
    let critical_rule = AlertRule {
        name: "critical_alerts".to_string(),
        source_pattern: "system".to_string(),
        severity_levels: vec![AlertSeverity::Critical],
        target_channels: vec!["console".to_string()],
        conditions: std::collections::HashMap::new(),
    };
    alert_manager.add_rule(critical_rule).await.unwrap();

    // Send different types of alerts
    let critical_alert = Alert::new(
        "System Critical Error".to_string(),
        "Database connection lost".to_string(),
        AlertSeverity::Critical,
        "system".to_string(),
    );

    let warning_alert = Alert::new(
        "High Memory Usage".to_string(),
        "Memory usage at 85%".to_string(),
        AlertSeverity::Warning,
        "system".to_string(),
    );

    let critical_alert_id = critical_alert.id.clone();

    // Send alerts and verify processing
    alert_manager.send_alert(critical_alert).await.unwrap();
    alert_manager.send_alert(warning_alert).await.unwrap();

    // Verify active alerts tracking
    let active_alerts = alert_manager.get_active_alerts().await.unwrap();
    assert_eq!(active_alerts.len(), 1); // Only critical alerts are tracked as active
    assert_eq!(active_alerts[0].severity, AlertSeverity::Critical);

    // Test alert resolution
    alert_manager
        .resolve_alert(&critical_alert_id)
        .await
        .unwrap();

    let active_alerts_after_resolution = alert_manager.get_active_alerts().await.unwrap();
    assert!(active_alerts_after_resolution.is_empty());
}

/// ANCHOR: Verifies distributed tracing system workflow
/// Tests: Trace creation, span hierarchy, context propagation
#[tokio::test]
async fn test_anchor_distributed_tracing_system_workflow() {
    let config = TracingConfig::default();
    let tracing_service = TracingService::new(config);

    // Start a new trace
    let trace_context = tracing_service
        .start_trace("research_request")
        .await
        .unwrap();
    assert_eq!(trace_context.operation_name, "research_request");

    // Create child spans
    let provider_span = tracing_service
        .start_span(&trace_context, "provider_call")
        .await
        .unwrap();
    assert_eq!(provider_span.trace_id, trace_context.trace_id);
    assert_eq!(
        provider_span.parent_span_id,
        Some(trace_context.current_span_id)
    );

    let quality_span = tracing_service
        .start_span(&trace_context, "quality_check")
        .await
        .unwrap();
    assert_eq!(quality_span.trace_id, trace_context.trace_id);
    assert_eq!(
        quality_span.parent_span_id,
        Some(trace_context.current_span_id)
    );

    // Verify trace context retrieval
    let retrieved_context = tracing_service
        .get_trace_context(trace_context.trace_id)
        .await
        .unwrap();
    assert!(retrieved_context.is_some());
    assert_eq!(retrieved_context.unwrap().trace_id, trace_context.trace_id);

    // Finish spans and trace
    tracing_service.finish_span(provider_span).await.unwrap();
    tracing_service.finish_span(quality_span).await.unwrap();
    tracing_service
        .finish_trace(trace_context.trace_id)
        .await
        .unwrap();

    // Verify trace cleanup
    let context_after_finish = tracing_service
        .get_trace_context(trace_context.trace_id)
        .await
        .unwrap();
    assert!(context_after_finish.is_none());
}

/// ANCHOR: Verifies cross-component monitoring integration
/// Tests: Monitoring system coordination, unified reporting, system overview
#[tokio::test]
async fn test_anchor_cross_component_monitoring_integration() {
    // This test verifies that all monitoring components work together
    let mut metrics_collector = MetricsCollector::new();
    let health_checker = HealthChecker::new(HealthConfig::default());
    let alert_manager = AlertManager::new(AlertManagerConfig::default());
    let tracing_service = TracingService::new(TracingConfig::default());

    // Set up monitoring for a complete request flow
    let trace_context = tracing_service
        .start_trace("integration_test")
        .await
        .unwrap();

    // Record various metrics during the request
    metrics_collector
        .record_api_request("POST", "/api/research", 200, Duration::from_millis(150))
        .await;

    let provider_metrics = ProviderPerformanceMetrics {
        provider_name: "claude".to_string(),
        request_count: 1,
        success_rate: 1.0,
        average_latency: Duration::from_millis(120),
        error_count: 0,
        last_success_time: chrono::Utc::now(),
    };
    metrics_collector
        .record_provider_metrics(&provider_metrics)
        .await;

    metrics_collector
        .record_quality_processing("relevance_check", Duration::from_millis(30), 256)
        .await;

    // Register and check health of components
    let api_health_check = Box::new(BasicHealthCheck::new("api".to_string(), || {
        HealthStatus::Healthy
    }));
    health_checker
        .register_health_check("api".to_string(), api_health_check)
        .await
        .unwrap();

    // Generate reports from all systems
    let metrics_report = metrics_collector
        .generate_performance_report()
        .await
        .unwrap();
    let health_report = health_checker.check_health().await.unwrap();

    // Verify integrated monitoring data
    assert_eq!(metrics_report.total_requests, 1);
    assert_eq!(metrics_report.successful_requests, 1);
    assert_eq!(metrics_report.error_rate, 0.0);

    assert_eq!(health_report.overall_status, HealthStatus::Healthy);
    assert_eq!(health_report.components.len(), 1);

    assert!(metrics_report.providers.contains_key("claude"));
    assert!(metrics_report.quality_processing.total_evaluations > 0);

    // Clean up tracing
    tracing_service
        .finish_trace(trace_context.trace_id)
        .await
        .unwrap();
}

/// ANCHOR: Verifies performance threshold violation detection accuracy
/// Tests: Threshold configuration, violation detection, severity classification
#[tokio::test]
async fn test_anchor_performance_threshold_violation_detection() {
    let mut metrics_collector = MetricsCollector::new();

    // Configure strict performance thresholds
    let strict_thresholds = PerformanceThresholds {
        max_response_time: Duration::from_millis(100),
        max_error_rate: 0.01,                // 1%
        min_cache_hit_rate: 0.95,            // 95%
        max_cpu_usage: 50.0,                 // 50%
        max_memory_usage: 512 * 1024 * 1024, // 512MB
    };
    metrics_collector.set_thresholds(strict_thresholds).await;

    // Record metrics that violate each threshold type
    // Response time violation - ensure all requests exceed threshold for clear average violation
    metrics_collector
        .record_api_request("GET", "/slow", 200, Duration::from_millis(120))
        .await;
    metrics_collector
        .record_api_request("GET", "/slow", 200, Duration::from_millis(120))
        .await;
    metrics_collector
        .record_api_request("GET", "/slow", 200, Duration::from_millis(120))
        .await;

    // Error rate violation - use 500 status code but also long response times to not skew average
    for _ in 0..10 {
        metrics_collector
            .record_api_request("GET", "/error", 500, Duration::from_millis(110))
            .await;
    }

    // Cache hit rate violation
    for _ in 0..10 {
        metrics_collector
            .record_cache_operation("test_cache", CacheOperation::Miss, Duration::from_millis(5))
            .await;
    }

    // Resource utilization violations
    let resource_metrics = ResourceUtilizationMetrics {
        cpu_usage_percent: 75.0,                // Exceeds 50%
        memory_usage_bytes: 1024 * 1024 * 1024, // Exceeds 512MB
        network_bytes_sent: 1024,
        network_bytes_received: 2048,
        disk_io_bytes: 4096,
        timestamp: chrono::Utc::now(),
    };
    metrics_collector
        .record_resource_metrics(&resource_metrics)
        .await;

    // Check for threshold violations
    let violations = metrics_collector
        .check_threshold_violations()
        .await
        .unwrap();

    // Debug output for troubleshooting
    let api_metrics = metrics_collector.get_api_metrics().await.unwrap();
    println!(
        "Average response time: {:?}ms, Threshold: 100ms",
        api_metrics.average_response_time.as_millis()
    );
    println!("Violations found: {}", violations.len());
    for v in &violations {
        println!(
            "Violation: {} = {} (threshold {})",
            v.metric_type, v.actual_value, v.threshold_value
        );
    }

    // Verify all expected violations are detected
    assert!(!violations.is_empty());

    // Response time violation
    let response_time_violation = violations
        .iter()
        .find(|v| v.metric_type == "response_time")
        .expect("Should detect response time violation");
    assert_eq!(response_time_violation.severity, ViolationSeverity::High);
    assert_eq!(response_time_violation.threshold_value, 100.0);
    assert!(response_time_violation.actual_value > 100.0);

    // Error rate violation
    let error_rate_violation = violations
        .iter()
        .find(|v| v.metric_type == "error_rate")
        .expect("Should detect error rate violation");
    assert_eq!(error_rate_violation.severity, ViolationSeverity::Critical);
    assert!(error_rate_violation.actual_value > 0.01);

    // Cache hit rate violation
    let cache_violation = violations
        .iter()
        .find(|v| v.metric_type.starts_with("cache_hit_rate"))
        .expect("Should detect cache hit rate violation");
    assert_eq!(cache_violation.severity, ViolationSeverity::Medium);
    assert!(cache_violation.actual_value < 0.95);

    // CPU usage violation
    let cpu_violation = violations
        .iter()
        .find(|v| v.metric_type == "cpu_usage")
        .expect("Should detect CPU usage violation");
    assert_eq!(cpu_violation.severity, ViolationSeverity::High);
    assert_eq!(cpu_violation.actual_value, 75.0);

    // Memory usage violation
    let memory_violation = violations
        .iter()
        .find(|v| v.metric_type == "memory_usage")
        .expect("Should detect memory usage violation");
    assert_eq!(memory_violation.severity, ViolationSeverity::High);
    assert_eq!(memory_violation.actual_value, 1024.0 * 1024.0 * 1024.0);
}
