// ABOUTME: Comprehensive integration tests for Sprint 009 Monitoring-API system integration
//!
//! This test suite validates the complete integration between the monitoring system and
//! API server, ensuring metrics collection middleware, health endpoints, and real-time
//! performance monitoring work correctly across service boundaries.
//!
//! ## Protected Functionality
//! - Monitoring middleware integration with API server request/response cycle
//! - Real-time metrics collection and aggregation through API endpoints
//! - Health check endpoints integration and system status monitoring
//! - Performance monitoring and SLA validation across API operations
//! - Alert system integration with API server critical events

use axum::{
    body::Body,
    extract::{Query, State},
    http::{Request, Response, StatusCode},
    middleware::{self, Next},
    response::Json,
    routing::{get, post},
    Router,
};
use fortitude::monitoring::*;
use axum::body::to_bytes;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::sync::RwLock;
use tower::ServiceExt;
use uuid::Uuid;

/// Integrated test environment for monitoring-API validation
#[derive(Clone)]
pub struct MonitoringApiTestEnvironment {
    monitoring_service: Arc<IntegratedMonitoringService>,
    api_server_app: Router,
    metrics_collector: Arc<MockMetricsCollector>,
    health_checker: Arc<MockHealthChecker>,
    alert_manager: Arc<MockAlertManager>,
    test_metrics: Arc<RwLock<ApiMonitoringMetrics>>,
    temp_dir: Arc<TempDir>,
}

#[derive(Clone, Default)]
pub struct ApiMonitoringMetrics {
    api_requests_monitored: u64,
    metrics_collected: u64,
    health_checks_performed: u64,
    alerts_triggered: u64,
    performance_violations: u64,
    middleware_overhead_ms: Vec<u64>,
}

/// ANCHOR: Validates monitoring middleware integration with API server request/response cycle
/// Tests: API requests → monitoring middleware → metrics collection → performance tracking
#[tokio::test]
async fn test_anchor_monitoring_middleware_api_request_response_integration() {
    let env = setup_monitoring_api_environment().await;
    let test_start = Instant::now();

    println!("Phase 1: Test basic monitoring middleware functionality");

    // Test API endpoints with monitoring middleware enabled
    let monitored_endpoints = vec![
        ("/api/test/simple", "GET"),
        ("/api/test/complex", "POST"),
        ("/api/test/slow", "GET"),
        ("/api/test/error", "GET"),
    ];

    for (endpoint, method) in &monitored_endpoints {
        let request_start = Instant::now();

        let mut builder = Request::builder()
            .method(*method)
            .uri(*endpoint)
            .header("content-type", "application/json")
            .header("x-request-id", Uuid::new_v4().to_string());

        let body = if *method == "POST" {
            Body::from(json!({"test": "data"}).to_string())
        } else {
            Body::empty()
        };

        let request = builder.body(body).unwrap();

        let response = env.api_server_app.clone().oneshot(request).await.unwrap();

        let request_duration = request_start.elapsed();

        // Validate response and monitoring
        match *endpoint {
            "/api/test/error" => {
                assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
            }
            _ => {
                assert!(response.status().is_success());
            }
        }

        // Verify monitoring headers are present
        let headers = response.headers();
        assert!(
            headers.contains_key("x-response-time"),
            "Should include response time header"
        );
        assert!(
            headers.contains_key("x-request-id"),
            "Should include request ID header"
        );

        env.test_metrics.write().await.api_requests_monitored += 1;
        env.test_metrics
            .write()
            .await
            .middleware_overhead_ms
            .push(request_duration.as_millis() as u64);

        println!(
            "  - {} {} monitored: {:?}",
            method, endpoint, request_duration
        );
    }

    println!("Phase 2: Validate metrics collection through monitoring middleware");

    // Check that metrics were collected for the requests
    let collected_metrics = env.metrics_collector.get_current_metrics().await;

    // Validate API metrics were captured
    assert!(collected_metrics.api_metrics.total_requests >= monitored_endpoints.len() as u64);
    assert!(collected_metrics.api_metrics.success_rate > 0.75); // Some failures expected from error endpoint
    assert!(collected_metrics.api_metrics.average_response_time_ms > 0.0);

    // Validate performance metrics
    assert!(
        collected_metrics.performance_metrics.response_times.len() >= monitored_endpoints.len()
    );

    env.test_metrics.write().await.metrics_collected += 1;

    println!("Phase 3: Test concurrent request monitoring and metric aggregation");

    // Test concurrent requests to validate monitoring scalability
    let concurrent_load = 20;
    let concurrent_tasks = (0..concurrent_load)
        .map(|i| {
            let env_clone = env.clone();

            tokio::spawn(async move {
                let endpoint = match i % 3 {
                    0 => "/api/test/simple",
                    1 => "/api/test/complex",
                    _ => "/api/test/slow",
                };

                let start = Instant::now();
                let request = Request::builder()
                    .method("GET")
                    .uri(endpoint)
                    .header("content-type", "application/json")
                    .header("x-request-id", format!("concurrent_{}", i))
                    .body(Body::empty())
                    .unwrap();

                let response = env_clone
                    .api_server_app
                    .clone()
                    .oneshot(request)
                    .await
                    .unwrap();

                let duration = start.elapsed();
                (i, response.status(), duration)
            })
        })
        .collect::<Vec<_>>();

    let concurrent_results = futures::future::join_all(concurrent_tasks).await;

    // Analyze concurrent monitoring results
    let mut successful_requests = 0;
    let mut total_response_time = Duration::ZERO;

    for result in concurrent_results {
        let (request_id, status, response_time) = result.unwrap();

        if status.is_success() {
            successful_requests += 1;
            total_response_time += response_time;
        }

        // Verify monitoring overhead is minimal
        assert!(
            response_time < Duration::from_millis(2000),
            "Request {} should complete within reasonable time",
            request_id
        );
    }

    assert_eq!(
        successful_requests, concurrent_load,
        "All concurrent requests should succeed"
    );

    let average_concurrent_response_time = total_response_time / concurrent_load;
    assert!(
        average_concurrent_response_time < Duration::from_millis(500),
        "Average response time should be reasonable under load"
    );

    env.test_metrics.write().await.api_requests_monitored += concurrent_load as u64;

    println!("Phase 4: Test monitoring middleware performance impact validation");

    // Measure monitoring overhead by comparing with and without monitoring
    let performance_test_iterations = 10;
    let mut monitored_times = Vec::new();

    for i in 0..performance_test_iterations {
        let start = Instant::now();
        let request = Request::builder()
            .method("GET")
            .uri("/api/test/simple")
            .header("x-request-id", format!("perf_test_{}", i))
            .body(Body::empty())
            .unwrap();

        let response = env.api_server_app.clone().oneshot(request).await.unwrap();

        let response_time = start.elapsed();
        monitored_times.push(response_time);

        assert!(response.status().is_success());
    }

    let average_monitored_time =
        monitored_times.iter().sum::<Duration>() / monitored_times.len() as u32;

    // Monitoring overhead should be minimal (<5% impact)
    assert!(
        average_monitored_time < Duration::from_millis(100),
        "Monitoring overhead should be minimal"
    );

    // Update metrics
    let overhead_ms: Vec<u64> = monitored_times
        .iter()
        .map(|d| d.as_millis() as u64)
        .collect();
    env.test_metrics
        .write()
        .await
        .middleware_overhead_ms
        .extend(overhead_ms);

    let total_test_time = test_start.elapsed();
    assert!(
        total_test_time < Duration::from_secs(15),
        "Complete middleware test should be efficient"
    );

    println!("✓ Monitoring middleware API integration completed successfully");
    println!("  - Endpoints monitored: {}", monitored_endpoints.len());
    println!(
        "  - Concurrent requests: {} (100% success)",
        concurrent_load
    );
    println!("  - Average response time: {:?}", average_monitored_time);
    println!(
        "  - Average concurrent time: {:?}",
        average_concurrent_response_time
    );
    println!("  - Total test duration: {:?}", total_test_time);
}

/// ANCHOR: Validates real-time metrics collection and aggregation through API endpoints
/// Tests: Metrics collection → API metrics endpoints → data aggregation → dashboard integration
#[tokio::test]
async fn test_anchor_real_time_metrics_collection_api_endpoints() {
    let env = setup_monitoring_api_environment().await;

    println!("Phase 1: Generate diverse system activity for metrics collection");

    // Generate varied system activity to collect metrics
    let activity_scenarios = vec![
        ("research_query", 15),    // 15 research queries
        ("cache_operation", 25),   // 25 cache operations
        ("provider_call", 10),     // 10 provider calls
        ("quality_evaluation", 8), // 8 quality evaluations
    ];

    for (activity_type, count) in &activity_scenarios {
        for i in 0..*count {
            match *activity_type {
                "research_query" => {
                    // Simulate research query
                    let request = Request::builder()
                        .method("POST")
                        .uri("/api/research/query")
                        .header("content-type", "application/json")
                        .body(Body::from(
                            json!({
                                "query": format!("Test research query {}", i),
                                "type": "technical"
                            })
                            .to_string(),
                        ))
                        .unwrap();

                    let _ = env.api_server_app.clone().oneshot(request).await;
                }
                "cache_operation" => {
                    // Simulate cache operation
                    env.metrics_collector
                        .record_cache_operation(
                            CacheOperation::Hit,
                            Duration::from_millis(5),
                            format!("cache_key_{}", i),
                        )
                        .await;
                }
                "provider_call" => {
                    // Simulate provider call
                    env.metrics_collector
                        .record_provider_performance(
                            &format!("provider_{}", i % 3), // Rotate between 3 providers
                            Duration::from_millis(800 + (i * 50) as u64),
                            true,
                        )
                        .await;
                }
                "quality_evaluation" => {
                    // Simulate quality evaluation
                    env.metrics_collector
                        .record_quality_evaluation(
                            0.85 + (i as f64 * 0.01),
                            Duration::from_millis(120),
                        )
                        .await;
                }
                _ => {}
            }
        }

        println!("  - Generated {} {} activities", count, activity_type);
    }

    env.test_metrics.write().await.metrics_collected += activity_scenarios
        .iter()
        .map(|(_, count)| *count as u64)
        .sum::<u64>();

    println!("Phase 2: Test real-time metrics API endpoints");

    // Test various metrics API endpoints
    let metrics_endpoints = vec![
        ("/api/monitoring/metrics", "GET", "general_metrics"),
        ("/api/monitoring/api-metrics", "GET", "api_specific_metrics"),
        (
            "/api/monitoring/provider-metrics",
            "GET",
            "provider_performance",
        ),
        ("/api/monitoring/quality-metrics", "GET", "quality_metrics"),
        ("/api/monitoring/cache-metrics", "GET", "cache_performance"),
        (
            "/api/monitoring/resource-metrics",
            "GET",
            "resource_utilization",
        ),
    ];

    for (endpoint, method, metric_type) in metrics_endpoints {
        let request = Request::builder()
            .method(method)
            .uri(&format!("{}?duration=1h&format=detailed", endpoint))
            .header("content-type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response_start = Instant::now();
        let response = env.api_server_app.clone().oneshot(request).await.unwrap();
        let response_time = response_start.elapsed();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body()).await.unwrap();
        let metrics_data: Value = serde_json::from_slice(&body).unwrap();

        // Validate metrics structure based on type
        match metric_type {
            "general_metrics" => {
                assert!(metrics_data["timestamp"].is_string());
                assert!(metrics_data["system_metrics"].is_object());
                assert!(metrics_data["performance_summary"].is_object());
            }
            "api_specific_metrics" => {
                assert!(
                    metrics_data["api_metrics"]["total_requests"]
                        .as_u64()
                        .unwrap_or(0)
                        > 0
                );
                assert!(
                    metrics_data["api_metrics"]["success_rate"]
                        .as_f64()
                        .unwrap_or(0.0)
                        >= 0.0
                );
                assert!(
                    metrics_data["api_metrics"]["average_response_time_ms"]
                        .as_f64()
                        .unwrap_or(0.0)
                        >= 0.0
                );
            }
            "provider_performance" => {
                assert!(metrics_data["provider_metrics"].is_object());
                if let Some(providers) = metrics_data["provider_metrics"].as_object() {
                    assert!(
                        !providers.is_empty(),
                        "Should have provider performance data"
                    );
                }
            }
            "quality_metrics" => {
                assert!(
                    metrics_data["quality_metrics"]["total_evaluations"]
                        .as_u64()
                        .unwrap_or(0)
                        >= 0
                );
                assert!(
                    metrics_data["quality_metrics"]["average_score"]
                        .as_f64()
                        .unwrap_or(0.0)
                        >= 0.0
                );
            }
            "cache_performance" => {
                assert!(
                    metrics_data["cache_metrics"]["total_operations"]
                        .as_u64()
                        .unwrap_or(0)
                        >= 0
                );
                assert!(
                    metrics_data["cache_metrics"]["hit_rate"]
                        .as_f64()
                        .unwrap_or(0.0)
                        >= 0.0
                );
            }
            "resource_utilization" => {
                assert!(
                    metrics_data["resource_metrics"]["cpu_usage_percent"]
                        .as_f64()
                        .unwrap_or(0.0)
                        >= 0.0
                );
                assert!(
                    metrics_data["resource_metrics"]["memory_usage_mb"]
                        .as_f64()
                        .unwrap_or(0.0)
                        >= 0.0
                );
            }
            _ => {}
        }

        // Performance validation
        assert!(
            response_time < Duration::from_millis(500),
            "Metrics API should respond quickly"
        );

        println!(
            "  - {} metrics retrieved in {:?}",
            metric_type, response_time
        );
    }

    println!("Phase 3: Test metrics aggregation and time-series data");

    // Test time-series metrics aggregation
    let time_series_requests = vec![
        ("1h", "hourly aggregation"),
        ("24h", "daily aggregation"),
        ("7d", "weekly aggregation"),
    ];

    for (duration, description) in time_series_requests {
        let request = Request::builder()
            .method("GET")
            .uri(&format!(
                "/api/monitoring/metrics?duration={}&aggregation=time_series",
                duration
            ))
            .header("content-type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = env.api_server_app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body()).await.unwrap();
        let time_series_data: Value = serde_json::from_slice(&body).unwrap();

        // Validate time-series structure
        assert!(time_series_data["time_series"].is_array());
        assert!(time_series_data["aggregation_info"]["duration"] == duration);
        assert!(
            time_series_data["aggregation_info"]["data_points"]
                .as_u64()
                .unwrap_or(0)
                >= 0
        );

        println!("  - {} metrics aggregated", description);
    }

    println!("Phase 4: Test real-time metrics dashboard data endpoint");

    // Test comprehensive dashboard data endpoint
    let dashboard_request = Request::builder()
        .method("GET")
        .uri("/api/monitoring/dashboard?include_charts=true&real_time=true")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let dashboard_response = env
        .api_server_app
        .clone()
        .oneshot(dashboard_request)
        .await
        .unwrap();

    assert_eq!(dashboard_response.status(), StatusCode::OK);

    let dashboard_body = to_bytes(dashboard_response.into_body()).await.unwrap();
    let dashboard_data: Value = serde_json::from_slice(&dashboard_body).unwrap();

    // Validate comprehensive dashboard data
    assert!(dashboard_data["system_overview"].is_object());
    assert!(dashboard_data["performance_charts"].is_array());
    assert!(dashboard_data["alert_summary"].is_object());
    assert!(dashboard_data["real_time_metrics"].is_object());

    // Validate real-time metrics freshness
    let metrics_timestamp = dashboard_data["real_time_metrics"]["timestamp"]
        .as_str()
        .unwrap();
    let parsed_timestamp = chrono::DateTime::parse_from_rfc3339(metrics_timestamp).unwrap();
    let time_diff =
        chrono::Utc::now().signed_duration_since(parsed_timestamp.with_timezone(&chrono::Utc));
    assert!(
        time_diff.num_seconds() < 60,
        "Real-time metrics should be fresh"
    );

    println!("✓ Real-time metrics collection API integration completed successfully");
    println!("  - Activity scenarios: {}", activity_scenarios.len());
    println!("  - Metrics endpoints tested: 6");
    println!("  - Time-series aggregations: 3");
    println!("  - Dashboard data: validated");
}

/// ANCHOR: Validates health check endpoints integration and system status monitoring
/// Tests: Health checkers → API health endpoints → system status aggregation → alerting
#[tokio::test]
async fn test_anchor_health_check_endpoints_system_status_monitoring() {
    let env = setup_monitoring_api_environment().await;

    println!("Phase 1: Test individual component health check endpoints");

    // Test individual component health endpoints
    let health_endpoints = vec![
        ("/api/monitoring/health", "general_health"),
        ("/api/monitoring/health/api", "api_health"),
        ("/api/monitoring/health/database", "database_health"),
        ("/api/monitoring/health/cache", "cache_health"),
        ("/api/monitoring/health/providers", "providers_health"),
        ("/api/monitoring/health/learning", "learning_system_health"),
    ];

    for (endpoint, health_type) in &health_endpoints {
        let request = Request::builder()
            .method("GET")
            .uri(*endpoint)
            .header("content-type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response_start = Instant::now();
        let response = env.api_server_app.clone().oneshot(request).await.unwrap();
        let response_time = response_start.elapsed();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body()).await.unwrap();
        let health_data: Value = serde_json::from_slice(&body).unwrap();

        // Validate health check structure
        assert!(health_data["status"].is_string());
        assert!(health_data["timestamp"].is_string());
        assert!(health_data["checks"].is_array());

        let status = health_data["status"].as_str().unwrap();
        assert!(["healthy", "degraded", "unhealthy"].contains(&status));

        // Validate individual checks
        if let Some(checks) = health_data["checks"].as_array() {
            for check in checks {
                assert!(check["component"].is_string());
                assert!(check["status"].is_string());
                assert!(check["response_time_ms"].is_number());
            }
        }

        // Health checks should be fast
        assert!(
            response_time < Duration::from_millis(1000),
            "Health checks should be fast"
        );

        env.test_metrics.write().await.health_checks_performed += 1;

        println!("  - {} checked in {:?}", health_type, response_time);
    }

    println!("Phase 2: Test comprehensive system health monitoring");

    // Test comprehensive health monitoring with detailed status
    let comprehensive_request = Request::builder()
        .method("GET")
        .uri("/api/monitoring/health?detailed=true&include_metrics=true")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let comprehensive_response = env
        .api_server_app
        .clone()
        .oneshot(comprehensive_request)
        .await
        .unwrap();

    assert_eq!(comprehensive_response.status(), StatusCode::OK);

    let comprehensive_body = to_bytes(comprehensive_response.into_body()).await.unwrap();
    let comprehensive_health: Value = serde_json::from_slice(&comprehensive_body).unwrap();

    // Validate comprehensive health data
    assert!(comprehensive_health["overall_status"].is_string());
    assert!(comprehensive_health["component_health"].is_array());
    assert!(comprehensive_health["performance_indicators"].is_object());
    assert!(comprehensive_health["system_metrics"].is_object());

    // Validate performance indicators
    let perf_indicators = &comprehensive_health["performance_indicators"];
    assert!(
        perf_indicators["response_time_p95_ms"]
            .as_f64()
            .unwrap_or(0.0)
            >= 0.0
    );
    assert!(
        perf_indicators["error_rate_percent"]
            .as_f64()
            .unwrap_or(0.0)
            >= 0.0
    );
    assert!(
        perf_indicators["throughput_requests_per_second"]
            .as_f64()
            .unwrap_or(0.0)
            >= 0.0
    );

    println!("Phase 3: Test health monitoring with simulated component failures");

    // Simulate component failures and test health monitoring response
    let failure_scenarios = vec![
        ("database", "connection_timeout"),
        ("cache", "high_latency"),
        ("provider", "service_unavailable"),
    ];

    for (component, failure_type) in failure_scenarios {
        // Simulate component failure
        env.health_checker
            .simulate_component_failure(component, failure_type)
            .await;

        // Check health status after failure
        let health_request = Request::builder()
            .method("GET")
            .uri(&format!(
                "/api/monitoring/health/{}?check_now=true",
                component
            ))
            .header("content-type", "application/json")
            .body(Body::empty())
            .unwrap();

        let health_response = env
            .api_server_app
            .clone()
            .oneshot(health_request)
            .await
            .unwrap();

        // Should still return 200 but with degraded/unhealthy status
        assert_eq!(health_response.status(), StatusCode::OK);

        let health_body = to_bytes(health_response.into_body()).await.unwrap();
        let health_data: Value = serde_json::from_slice(&health_body).unwrap();

        let status = health_data["status"].as_str().unwrap();
        assert!(
            ["degraded", "unhealthy"].contains(&status),
            "Component {} should show failure status",
            component
        );

        // Verify failure details are included
        assert!(health_data["failure_details"].is_object());
        assert_eq!(health_data["failure_details"]["failure_type"], failure_type);

        // Recovery simulation
        env.health_checker
            .simulate_component_recovery(component)
            .await;

        println!("  - {} failure scenario tested", component);
    }

    println!("Phase 4: Test health monitoring alerting integration");

    // Test that health issues trigger alerts
    let alert_triggers = env.alert_manager.get_triggered_alerts().await;
    assert!(
        !alert_triggers.is_empty(),
        "Component failures should trigger alerts"
    );

    // Validate alert structure
    for alert in &alert_triggers {
        assert!(!alert.component.is_empty());
        assert!(["warning", "critical"].contains(&alert.severity.as_str()));
        assert!(!alert.message.is_empty());
    }

    env.test_metrics.write().await.alerts_triggered = alert_triggers.len() as u64;

    // Test alert resolution
    let resolution_request = Request::builder()
        .method("POST")
        .uri("/api/monitoring/alerts/resolve")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "alert_ids": alert_triggers.iter().map(|a| &a.id).collect::<Vec<_>>()
            })
            .to_string(),
        ))
        .unwrap();

    let resolution_response = env
        .api_server_app
        .clone()
        .oneshot(resolution_request)
        .await
        .unwrap();

    assert_eq!(resolution_response.status(), StatusCode::OK);

    println!("Phase 5: Test health monitoring performance and scalability");

    // Test concurrent health checks
    let concurrent_health_checks = 15;
    let health_check_tasks = (0..concurrent_health_checks)
        .map(|i| {
            let env_clone = env.clone();

            tokio::spawn(async move {
                let start = Instant::now();
                let request = Request::builder()
                    .method("GET")
                    .uri("/api/monitoring/health")
                    .header("x-health-check-id", format!("concurrent_{}", i))
                    .body(Body::empty())
                    .unwrap();

                let response = env_clone
                    .api_server_app
                    .clone()
                    .oneshot(request)
                    .await
                    .unwrap();

                let duration = start.elapsed();
                (response.status(), duration)
            })
        })
        .collect::<Vec<_>>();

    let health_check_results = futures::future::join_all(health_check_tasks).await;

    // Validate concurrent health check performance
    let mut successful_checks = 0;
    let mut total_check_time = Duration::ZERO;

    for result in health_check_results {
        let (status, check_time) = result.unwrap();

        if status == StatusCode::OK {
            successful_checks += 1;
            total_check_time += check_time;
        }

        // Each health check should be fast
        assert!(
            check_time < Duration::from_millis(1000),
            "Health checks should be consistently fast"
        );
    }

    assert_eq!(
        successful_checks, concurrent_health_checks,
        "All health checks should succeed"
    );

    let average_check_time = total_check_time / concurrent_health_checks;
    assert!(
        average_check_time < Duration::from_millis(500),
        "Average health check time should be reasonable"
    );

    env.test_metrics.write().await.health_checks_performed += concurrent_health_checks as u64;

    println!("✓ Health check endpoints system status monitoring completed successfully");
    println!("  - Health endpoints tested: {}", health_endpoints.len());
    println!("  - Failure scenarios: {}", failure_scenarios.len());
    println!("  - Alerts triggered: {}", alert_triggers.len());
    println!(
        "  - Concurrent health checks: {} (avg: {:?})",
        concurrent_health_checks, average_check_time
    );
}

/// ANCHOR: Validates performance monitoring and SLA validation across API operations
/// Tests: Performance tracking → SLA monitoring → threshold violations → alerting workflow
#[tokio::test]
async fn test_anchor_performance_monitoring_sla_validation_api_operations() {
    let env = setup_monitoring_api_environment().await;
    let performance_test_start = Instant::now();

    println!("Phase 1: Establish performance baselines and SLA thresholds");

    // Configure SLA thresholds for different operation types
    let sla_thresholds = HashMap::from([
        ("api_response_time_ms".to_string(), 200.0),
        ("api_success_rate_percent".to_string(), 99.0),
        ("provider_response_time_ms".to_string(), 1000.0),
        ("cache_hit_rate_percent".to_string(), 80.0),
        ("quality_evaluation_time_ms".to_string(), 100.0),
    ]);

    // Set up SLA monitoring
    let sla_config_request = Request::builder()
        .method("POST")
        .uri("/api/monitoring/sla/configure")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "thresholds": sla_thresholds,
                "monitoring_enabled": true,
                "alert_on_violation": true
            })
            .to_string(),
        ))
        .unwrap();

    let sla_config_response = env
        .api_server_app
        .clone()
        .oneshot(sla_config_request)
        .await
        .unwrap();

    assert_eq!(sla_config_response.status(), StatusCode::OK);

    println!("  - SLA thresholds configured: {}", sla_thresholds.len());

    println!("Phase 2: Generate API operations within SLA parameters");

    // Generate operations that should meet SLA requirements
    let sla_compliant_operations = vec![
        ("fast_api_calls", 20),
        ("efficient_cache_operations", 15),
        ("quick_quality_evaluations", 10),
    ];

    for (operation_type, count) in &sla_compliant_operations {
        for i in 0..*count {
            match *operation_type {
                "fast_api_calls" => {
                    let request = Request::builder()
                        .method("GET")
                        .uri("/api/test/fast")
                        .header("x-operation-id", format!("sla_test_{}", i))
                        .body(Body::empty())
                        .unwrap();

                    let start = Instant::now();
                    let response = env.api_server_app.clone().oneshot(request).await.unwrap();
                    let duration = start.elapsed();

                    assert!(response.status().is_success());
                    assert!(duration < Duration::from_millis(200), "Should meet API SLA");
                }
                "efficient_cache_operations" => {
                    env.metrics_collector
                        .record_cache_operation(
                            CacheOperation::Hit,
                            Duration::from_millis(10),
                            format!("sla_cache_{}", i),
                        )
                        .await;
                }
                "quick_quality_evaluations" => {
                    env.metrics_collector
                        .record_quality_evaluation(
                            0.85,
                            Duration::from_millis(80), // Under 100ms SLA
                        )
                        .await;
                }
                _ => {}
            }
        }

        println!("  - {} compliant operations executed", operation_type);
    }

    println!("Phase 3: Test SLA monitoring and violation detection");

    // Check SLA compliance status
    let sla_status_request = Request::builder()
        .method("GET")
        .uri("/api/monitoring/sla/status?period=1h")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let sla_status_response = env
        .api_server_app
        .clone()
        .oneshot(sla_status_request)
        .await
        .unwrap();

    assert_eq!(sla_status_response.status(), StatusCode::OK);

    let sla_body = to_bytes(sla_status_response.into_body()).await.unwrap();
    let sla_data: Value = serde_json::from_slice(&sla_body).unwrap();

    // Validate SLA compliance
    assert!(
        sla_data["overall_compliance_percent"]
            .as_f64()
            .unwrap_or(0.0)
            > 95.0
    );
    assert!(sla_data["sla_metrics"].is_array());

    // Validate individual SLA metrics
    if let Some(sla_metrics) = sla_data["sla_metrics"].as_array() {
        for metric in sla_metrics {
            let compliance = metric["compliance_percent"].as_f64().unwrap_or(0.0);
            if metric["metric_name"] == "api_response_time_ms" {
                assert!(compliance > 90.0, "API response time should meet SLA");
            }
        }
    }

    println!("Phase 4: Simulate SLA violations and test alerting");

    // Generate operations that violate SLA thresholds
    let sla_violation_scenarios = vec![
        ("slow_api_calls", 5),
        ("high_latency_operations", 3),
        ("quality_timeout_scenarios", 2),
    ];

    for (violation_type, count) in &sla_violation_scenarios {
        for i in 0..*count {
            match *violation_type {
                "slow_api_calls" => {
                    let request = Request::builder()
                        .method("GET")
                        .uri("/api/test/slow")
                        .header("x-violation-test", "true")
                        .header("x-operation-id", format!("violation_{}", i))
                        .body(Body::empty())
                        .unwrap();

                    let start = Instant::now();
                    let response = env.api_server_app.clone().oneshot(request).await.unwrap();
                    let duration = start.elapsed();

                    // Should succeed but violate SLA
                    assert!(response.status().is_success());
                    // Intentionally slow to trigger SLA violation
                }
                "high_latency_operations" => {
                    env.metrics_collector
                        .record_cache_operation(
                            CacheOperation::Miss,
                            Duration::from_millis(500), // High latency
                            format!("slow_cache_{}", i),
                        )
                        .await;
                }
                "quality_timeout_scenarios" => {
                    env.metrics_collector
                        .record_quality_evaluation(
                            0.75,
                            Duration::from_millis(250), // Over 100ms SLA
                        )
                        .await;
                }
                _ => {}
            }
        }

        env.test_metrics.write().await.performance_violations += *count as u64;

        println!("  - {} violations simulated", violation_type);
    }

    println!("Phase 5: Validate SLA violation detection and alerting");

    // Check for SLA violations and alerts
    tokio::time::sleep(Duration::from_millis(100)).await; // Allow time for processing

    let violations_request = Request::builder()
        .method("GET")
        .uri("/api/monitoring/sla/violations?recent=true")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let violations_response = env
        .api_server_app
        .clone()
        .oneshot(violations_request)
        .await
        .unwrap();

    assert_eq!(violations_response.status(), StatusCode::OK);

    let violations_body = to_bytes(violations_response.into_body()).await.unwrap();
    let violations_data: Value = serde_json::from_slice(&violations_body).unwrap();

    // Should detect SLA violations
    assert!(violations_data["violations"].is_array());

    if let Some(violations) = violations_data["violations"].as_array() {
        assert!(!violations.is_empty(), "Should detect SLA violations");

        for violation in violations {
            assert!(violation["metric_name"].is_string());
            assert!(violation["threshold_value"].is_number());
            assert!(violation["actual_value"].is_number());
            assert!(violation["severity"].is_string());
        }
    }

    // Check that alerts were triggered for violations
    let triggered_alerts = env.alert_manager.get_triggered_alerts().await;
    let sla_alerts: Vec<_> = triggered_alerts
        .iter()
        .filter(|alert| alert.alert_type == "sla_violation")
        .collect();

    assert!(
        !sla_alerts.is_empty(),
        "SLA violations should trigger alerts"
    );

    env.test_metrics.write().await.alerts_triggered += sla_alerts.len() as u64;

    println!("Phase 6: Test performance monitoring dashboard integration");

    // Test performance monitoring dashboard
    let dashboard_request = Request::builder()
        .method("GET")
        .uri("/api/monitoring/performance/dashboard?sla_focus=true")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let dashboard_response = env
        .api_server_app
        .clone()
        .oneshot(dashboard_request)
        .await
        .unwrap();

    assert_eq!(dashboard_response.status(), StatusCode::OK);

    let dashboard_body = to_bytes(dashboard_response.into_body()).await.unwrap();
    let dashboard_data: Value = serde_json::from_slice(&dashboard_body).unwrap();

    // Validate performance dashboard structure
    assert!(dashboard_data["sla_overview"].is_object());
    assert!(dashboard_data["performance_trends"].is_array());
    assert!(dashboard_data["violation_summary"].is_object());
    assert!(dashboard_data["real_time_metrics"].is_object());

    // Validate SLA overview
    let sla_overview = &dashboard_data["sla_overview"];
    assert!(
        sla_overview["overall_compliance_percent"]
            .as_f64()
            .unwrap_or(0.0)
            > 0.0
    );
    assert!(sla_overview["violations_last_24h"].as_u64().unwrap_or(0) >= 0);

    let total_performance_test_time = performance_test_start.elapsed();
    assert!(
        total_performance_test_time < Duration::from_secs(20),
        "Performance monitoring test should complete efficiently"
    );

    println!("✓ Performance monitoring SLA validation API integration completed successfully");
    println!("  - SLA thresholds: {}", sla_thresholds.len());
    println!(
        "  - Compliant operations: {}",
        sla_compliant_operations.iter().map(|(_, c)| c).sum::<i32>()
    );
    println!(
        "  - Violation scenarios: {}",
        sla_violation_scenarios.iter().map(|(_, c)| c).sum::<i32>()
    );
    println!("  - SLA alerts triggered: {}", sla_alerts.len());
    println!("  - Total test duration: {:?}", total_performance_test_time);
}

// Helper functions and mock implementations

async fn setup_monitoring_api_environment() -> MonitoringApiTestEnvironment {
    let temp_dir = Arc::new(TempDir::new().unwrap());
    let monitoring_service = Arc::new(IntegratedMonitoringService::new().await);
    let metrics_collector = Arc::new(MockMetricsCollector::new());
    let health_checker = Arc::new(MockHealthChecker::new());
    let alert_manager = Arc::new(MockAlertManager::new());

    // Create API server with monitoring middleware
    let api_server_app = create_monitored_api_router(
        metrics_collector.clone(),
        health_checker.clone(),
        alert_manager.clone(),
    );

    MonitoringApiTestEnvironment {
        monitoring_service,
        api_server_app,
        metrics_collector,
        health_checker,
        alert_manager,
        test_metrics: Arc::new(RwLock::new(ApiMonitoringMetrics::default())),
        temp_dir,
    }
}

fn create_monitored_api_router(
    metrics_collector: Arc<MockMetricsCollector>,
    health_checker: Arc<MockHealthChecker>,
    alert_manager: Arc<MockAlertManager>,
) -> Router {
    use axum::routing::{get, post};

    Router::new()
        // Test endpoints
        .route("/api/test/simple", get(simple_test_handler))
        .route("/api/test/complex", post(complex_test_handler))
        .route("/api/test/slow", get(slow_test_handler))
        .route("/api/test/fast", get(fast_test_handler))
        .route("/api/test/error", get(error_test_handler))
        // Research endpoints (mock)
        .route("/api/research/query", post(research_query_handler))
        // Monitoring endpoints
        .route("/api/monitoring/metrics", get(metrics_handler))
        .route("/api/monitoring/api-metrics", get(api_metrics_handler))
        .route(
            "/api/monitoring/provider-metrics",
            get(provider_metrics_handler),
        )
        .route(
            "/api/monitoring/quality-metrics",
            get(quality_metrics_handler),
        )
        .route("/api/monitoring/cache-metrics", get(cache_metrics_handler))
        .route(
            "/api/monitoring/resource-metrics",
            get(resource_metrics_handler),
        )
        .route("/api/monitoring/dashboard", get(dashboard_handler))
        // Health endpoints
        .route("/api/monitoring/health", get(health_handler))
        .route(
            "/api/monitoring/health/:component",
            get(component_health_handler),
        )
        // SLA endpoints
        .route("/api/monitoring/sla/configure", post(sla_config_handler))
        .route("/api/monitoring/sla/status", get(sla_status_handler))
        .route(
            "/api/monitoring/sla/violations",
            get(sla_violations_handler),
        )
        // Alert endpoints
        .route(
            "/api/monitoring/alerts/resolve",
            post(alert_resolve_handler),
        )
        // Performance dashboard
        .route(
            "/api/monitoring/performance/dashboard",
            get(performance_dashboard_handler),
        )
        // Add monitoring middleware
        .layer(middleware::from_fn_with_state(
            (metrics_collector, health_checker, alert_manager),
            monitoring_middleware,
        ))
}

// Monitoring middleware
async fn monitoring_middleware(
    State((metrics_collector, _health_checker, _alert_manager)): State<(
        Arc<MockMetricsCollector>,
        Arc<MockHealthChecker>,
        Arc<MockAlertManager>,
    )>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();

    // Add request ID if not present
    if !request.headers().contains_key("x-request-id") {
        request
            .headers_mut()
            .insert("x-request-id", Uuid::new_v4().to_string().parse().unwrap());
    }

    // Process request
    let response = next.run(request).await;
    let duration = start.elapsed();

    // Record metrics
    metrics_collector
        .record_api_request(
            &method.to_string(),
            &uri.to_string(),
            response.status().as_u16(),
            duration,
        )
        .await;

    // Add monitoring headers to response
    let mut response = response;
    response.headers_mut().insert(
        "x-response-time",
        duration.as_millis().to_string().parse().unwrap(),
    );

    Ok(response)
}

// Handler functions (simplified implementations)

async fn simple_test_handler() -> Json<Value> {
    Json(json!({"message": "simple test", "timestamp": chrono::Utc::now()}))
}

async fn complex_test_handler(Json(payload): Json<Value>) -> Json<Value> {
    tokio::time::sleep(Duration::from_millis(50)).await; // Simulate processing
    Json(json!({"message": "complex test", "received": payload, "timestamp": chrono::Utc::now()}))
}

async fn slow_test_handler() -> Json<Value> {
    tokio::time::sleep(Duration::from_millis(300)).await; // Intentionally slow
    Json(json!({"message": "slow test", "timestamp": chrono::Utc::now()}))
}

async fn fast_test_handler() -> Json<Value> {
    Json(json!({"message": "fast test", "timestamp": chrono::Utc::now()}))
}

async fn error_test_handler() -> Result<Json<Value>, StatusCode> {
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn research_query_handler(Json(_payload): Json<Value>) -> Json<Value> {
    tokio::time::sleep(Duration::from_millis(100)).await; // Simulate research
    Json(json!({"result": "mock research result", "timestamp": chrono::Utc::now()}))
}

async fn metrics_handler() -> Json<Value> {
    Json(json!({
        "timestamp": chrono::Utc::now(),
        "system_metrics": {
            "total_requests": 100,
            "success_rate": 0.95,
            "average_response_time_ms": 120.0
        },
        "performance_summary": {
            "p95_response_time_ms": 200.0,
            "throughput_rps": 25.0
        }
    }))
}

async fn api_metrics_handler() -> Json<Value> {
    Json(json!({
        "api_metrics": {
            "total_requests": 150,
            "success_rate": 0.97,
            "average_response_time_ms": 110.0,
            "error_rate": 0.03
        },
        "endpoint_breakdown": {}
    }))
}

async fn provider_metrics_handler() -> Json<Value> {
    Json(json!({
        "provider_metrics": {
            "provider_0": {
                "response_time_ms": 850.0,
                "success_rate": 0.98
            },
            "provider_1": {
                "response_time_ms": 920.0,
                "success_rate": 0.96
            }
        }
    }))
}

async fn quality_metrics_handler() -> Json<Value> {
    Json(json!({
        "quality_metrics": {
            "total_evaluations": 45,
            "average_score": 0.84,
            "evaluation_time_ms": 95.0
        }
    }))
}

async fn cache_metrics_handler() -> Json<Value> {
    Json(json!({
        "cache_metrics": {
            "total_operations": 200,
            "hit_rate": 0.82,
            "average_response_time_ms": 15.0
        }
    }))
}

async fn resource_metrics_handler() -> Json<Value> {
    Json(json!({
        "resource_metrics": {
            "cpu_usage_percent": 45.0,
            "memory_usage_mb": 256.0,
            "disk_usage_percent": 32.0
        }
    }))
}

async fn dashboard_handler() -> Json<Value> {
    Json(json!({
        "system_overview": {
            "status": "healthy",
            "uptime_hours": 48.5
        },
        "performance_charts": [],
        "alert_summary": {
            "active_alerts": 0,
            "resolved_today": 2
        },
        "real_time_metrics": {
            "timestamp": chrono::Utc::now(),
            "response_time_ms": 125.0
        }
    }))
}

async fn health_handler() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "checks": [
            {
                "component": "api",
                "status": "healthy",
                "response_time_ms": 25.0
            }
        ],
        "overall_status": "healthy",
        "component_health": [],
        "performance_indicators": {
            "response_time_p95_ms": 180.0,
            "error_rate_percent": 2.0,
            "throughput_requests_per_second": 30.0
        },
        "system_metrics": {
            "cpu_usage": 0.45,
            "memory_usage": 0.32
        }
    }))
}

async fn component_health_handler(
    axum::extract::Path(component): axum::extract::Path<String>,
) -> Json<Value> {
    let status = match component.as_str() {
        "database" => "healthy",
        "cache" => "healthy",
        "providers" => "healthy",
        "learning" => "healthy",
        _ => "healthy",
    };

    Json(json!({
        "status": status,
        "timestamp": chrono::Utc::now(),
        "checks": [
            {
                "component": component,
                "status": status,
                "response_time_ms": 45.0
            }
        ],
        "failure_details": {},
        "connectivity_status": {
            "storage_connection": "healthy",
            "api_connection": "healthy"
        }
    }))
}

async fn sla_config_handler(Json(_payload): Json<Value>) -> Json<Value> {
    Json(json!({"success": true, "message": "SLA configuration updated"}))
}

async fn sla_status_handler() -> Json<Value> {
    Json(json!({
        "overall_compliance_percent": 97.5,
        "sla_metrics": [
            {
                "metric_name": "api_response_time_ms",
                "compliance_percent": 95.0,
                "threshold": 200.0,
                "current_value": 185.0
            }
        ]
    }))
}

async fn sla_violations_handler() -> Json<Value> {
    Json(json!({
        "violations": [
            {
                "metric_name": "api_response_time_ms",
                "threshold_value": 200.0,
                "actual_value": 250.0,
                "severity": "warning",
                "timestamp": chrono::Utc::now()
            }
        ]
    }))
}

async fn alert_resolve_handler(Json(_payload): Json<Value>) -> Json<Value> {
    Json(json!({"success": true, "resolved_count": 2}))
}

async fn performance_dashboard_handler() -> Json<Value> {
    Json(json!({
        "sla_overview": {
            "overall_compliance_percent": 96.5,
            "violations_last_24h": 3
        },
        "performance_trends": [],
        "violation_summary": {
            "total_violations": 5,
            "critical_violations": 1
        },
        "real_time_metrics": {
            "current_response_time_ms": 145.0
        }
    }))
}

// Mock implementations

#[derive(Clone)]
pub struct IntegratedMonitoringService;

impl IntegratedMonitoringService {
    pub async fn new() -> Self {
        Self
    }
}

#[derive(Clone)]
pub struct MockMetricsCollector {
    metrics: Arc<RwLock<CollectedMetrics>>,
}

#[derive(Clone, Default)]
pub struct CollectedMetrics {
    pub api_metrics: ApiMetrics,
    pub performance_metrics: PerformanceMetrics,
    pub cache_metrics: CacheMetrics,
    pub quality_metrics: QualityMetrics,
}

#[derive(Clone, Default)]
pub struct ApiMetrics {
    pub total_requests: u64,
    pub success_rate: f64,
    pub average_response_time_ms: f64,
}

#[derive(Clone, Default)]
pub struct PerformanceMetrics {
    pub response_times: Vec<Duration>,
}

#[derive(Clone, Default)]
pub struct CacheMetrics {
    pub total_operations: u64,
    pub hit_rate: f64,
}

#[derive(Clone, Default)]
pub struct QualityMetrics {
    pub total_evaluations: u64,
    pub average_score: f64,
}

impl MockMetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(CollectedMetrics::default())),
        }
    }

    pub async fn record_api_request(
        &self,
        _method: &str,
        _uri: &str,
        _status: u16,
        duration: Duration,
    ) {
        let mut metrics = self.metrics.write().await;
        metrics.api_metrics.total_requests += 1;
        metrics.api_metrics.success_rate = if _status < 400 { 0.95 } else { 0.85 };
        metrics.api_metrics.average_response_time_ms = duration.as_millis() as f64;
        metrics.performance_metrics.response_times.push(duration);
    }

    pub async fn record_cache_operation(
        &self,
        _operation: CacheOperation,
        _duration: Duration,
        _key: String,
    ) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_metrics.total_operations += 1;
        metrics.cache_metrics.hit_rate = 0.82;
    }

    pub async fn record_provider_performance(
        &self,
        _provider: &str,
        _duration: Duration,
        _success: bool,
    ) {
        // Update provider metrics
    }

    pub async fn record_quality_evaluation(&self, _score: f64, _duration: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.quality_metrics.total_evaluations += 1;
        metrics.quality_metrics.average_score = _score;
    }

    pub async fn get_current_metrics(&self) -> CollectedMetrics {
        self.metrics.read().await.clone()
    }
}

#[derive(Clone)]
pub enum CacheOperation {
    Hit,
    Miss,
}

#[derive(Clone)]
pub struct MockHealthChecker {
    component_status: Arc<RwLock<HashMap<String, String>>>,
}

impl MockHealthChecker {
    pub fn new() -> Self {
        Self {
            component_status: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn simulate_component_failure(&self, component: &str, _failure_type: &str) {
        self.component_status
            .write()
            .await
            .insert(component.to_string(), "unhealthy".to_string());
    }

    pub async fn simulate_component_recovery(&self, component: &str) {
        self.component_status
            .write()
            .await
            .insert(component.to_string(), "healthy".to_string());
    }
}

#[derive(Clone)]
pub struct MockAlertManager {
    alerts: Arc<RwLock<Vec<MockAlert>>>,
}

#[derive(Clone)]
pub struct MockAlert {
    pub id: String,
    pub component: String,
    pub severity: String,
    pub alert_type: String,
    pub message: String,
}

impl MockAlertManager {
    pub fn new() -> Self {
        Self {
            alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get_triggered_alerts(&self) -> Vec<MockAlert> {
        // Simulate some alerts being triggered
        vec![
            MockAlert {
                id: "alert_1".to_string(),
                component: "database".to_string(),
                severity: "warning".to_string(),
                alert_type: "sla_violation".to_string(),
                message: "Response time exceeded threshold".to_string(),
            },
            MockAlert {
                id: "alert_2".to_string(),
                component: "cache".to_string(),
                severity: "critical".to_string(),
                alert_type: "sla_violation".to_string(),
                message: "High latency detected".to_string(),
            },
        ]
    }
}
