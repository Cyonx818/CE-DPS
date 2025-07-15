// ABOUTME: Comprehensive anchor tests for performance monitoring and observability
//! # Performance Monitoring Anchor Tests
//!
//! This test suite provides comprehensive validation of the performance monitoring
//! and observability system across all components of Fortitude. These tests ensure
//! that the monitoring system meets its performance requirements and provides
//! accurate metrics collection, alerting, and health monitoring.
//!
//! ## Test Coverage
//!
//! - **Configuration System**: Validation of monitoring configuration
//! - **API Server Integration**: HTTP request monitoring and metrics
//! - **MCP Server Integration**: Tool call and resource access monitoring
//! - **Performance Thresholds**: Response time and error rate validation
//! - **Health Monitoring**: Component health status tracking
//! - **Custom Metrics**: Application-specific metric collection
//! - **Real-time Performance**: <200ms response time validation
//!
//! ## Test Categories
//!
//! - **Anchor Tests**: Immutable behavior validation
//! - **Integration Tests**: Cross-component monitoring validation
//! - **Performance Tests**: Response time and throughput validation
//! - **Load Tests**: High-volume monitoring validation

use std::time::{Duration, Instant};
use tokio::time;

use fortitude::monitoring::{
    ComponentHealth, ComponentMetrics, HealthStatus, Monitorable, MonitoringConfiguration,
    MonitoringError, PerformanceThresholds,
};

/// Test monitoring configuration creation and validation
#[tokio::test]
async fn anchor_test_monitoring_configuration_defaults() {
    // Create default configuration
    let config = MonitoringConfiguration::default();

    // Validate core configuration
    assert!(config.core.enabled);
    assert_eq!(config.core.system_name, "fortitude-monitoring");
    assert_eq!(config.core.environment, "development");
    assert_eq!(config.core.max_memory_mb, 512);

    // Validate collection configuration
    assert!(config.collection.enabled);
    assert_eq!(config.collection.interval_seconds, 10);
    assert_eq!(config.collection.max_metrics_in_memory, 10000);
    assert_eq!(config.collection.batch_size, 100);
    assert_eq!(config.collection.sampling_rate, 1.0);

    // Validate performance configuration
    assert!(config.performance.enabled);
    assert_eq!(config.performance.response_time.target_ms, 200);
    assert_eq!(config.performance.response_time.warning_ms, 500);
    assert_eq!(config.performance.response_time.critical_ms, 1000);

    // Validate error rate thresholds
    assert_eq!(config.performance.error_rates.warning_percent, 5.0);
    assert_eq!(config.performance.error_rates.critical_percent, 10.0);
    assert_eq!(config.performance.error_rates.min_request_count, 10);

    // Validate resource utilization thresholds
    assert_eq!(config.performance.resource_utilization.cpu.warning, 70.0);
    assert_eq!(config.performance.resource_utilization.cpu.critical, 90.0);
    assert_eq!(config.performance.resource_utilization.memory.warning, 80.0);
    assert_eq!(
        config.performance.resource_utilization.memory.critical,
        95.0
    );

    // Validate configuration
    assert!(config.validate().is_ok());
}

/// Test API server optimized configuration
#[tokio::test]
async fn anchor_test_api_server_monitoring_configuration() {
    let config = MonitoringConfiguration::for_api_server();

    // API server should have more frequent collection
    assert_eq!(config.collection.interval_seconds, 5);
    assert!(config.collection.enable_high_resolution);

    // Stricter response time targets for API server
    assert_eq!(config.performance.response_time.target_ms, 200);
    assert_eq!(config.performance.response_time.warning_ms, 300);
    assert_eq!(config.performance.response_time.critical_ms, 500);

    // Should have API server component configuration
    assert!(config.components.contains_key("api-server"));
    let api_component = &config.components["api-server"];
    assert_eq!(api_component.name, "api-server");
    assert!(api_component.enabled);
    assert!(api_component
        .custom_metrics
        .contains(&"http_requests_total".to_string()));
    assert!(api_component
        .custom_metrics
        .contains(&"http_request_duration_ms".to_string()));

    assert!(config.validate().is_ok());
}

/// Test MCP server optimized configuration
#[tokio::test]
async fn anchor_test_mcp_server_monitoring_configuration() {
    let config = MonitoringConfiguration::for_mcp_server();

    // MCP server should have standard collection interval
    assert_eq!(config.collection.interval_seconds, 10);

    // More relaxed response time targets for complex operations
    assert_eq!(config.performance.response_time.target_ms, 500);
    assert_eq!(config.performance.response_time.warning_ms, 1000);
    assert_eq!(config.performance.response_time.critical_ms, 2000);

    // Should have MCP server component configuration
    assert!(config.components.contains_key("mcp-server"));
    let mcp_component = &config.components["mcp-server"];
    assert_eq!(mcp_component.name, "mcp-server");
    assert!(mcp_component.enabled);
    assert!(mcp_component
        .custom_metrics
        .contains(&"mcp_requests_total".to_string()));
    assert!(mcp_component
        .custom_metrics
        .contains(&"mcp_tool_calls_total".to_string()));

    assert!(config.validate().is_ok());
}

/// Test configuration validation with invalid values
#[tokio::test]
async fn anchor_test_monitoring_configuration_validation() {
    let mut config = MonitoringConfiguration::default();

    // Test invalid response time thresholds (warning >= critical)
    config.performance.response_time.warning_ms = 1000;
    config.performance.response_time.critical_ms = 500;
    assert!(config.validate().is_err());

    // Fix response times
    config.performance.response_time.warning_ms = 500;
    config.performance.response_time.critical_ms = 1000;

    // Test invalid error rate thresholds (warning >= critical)
    config.performance.error_rates.warning_percent = 15.0;
    config.performance.error_rates.critical_percent = 10.0;
    assert!(config.validate().is_err());

    // Fix error rates
    config.performance.error_rates.warning_percent = 5.0;
    config.performance.error_rates.critical_percent = 10.0;

    // Test invalid throughput thresholds
    config.performance.throughput.min_rps = 200.0;
    config.performance.throughput.target_rps = 100.0; // min > target
    assert!(config.validate().is_err());

    // Fix throughput
    config.performance.throughput.min_rps = 50.0;
    config.performance.throughput.target_rps = 100.0;
    config.performance.throughput.max_rps = 200.0;

    // Should now be valid
    assert!(config.validate().is_ok());
}

/// Test monitoring system performance requirements
#[tokio::test]
async fn anchor_test_monitoring_system_performance() {
    let start = Instant::now();

    // Create monitoring configuration
    let config = MonitoringConfiguration::default();
    let config_time = start.elapsed();

    // Configuration creation should be very fast
    assert!(
        config_time < Duration::from_millis(10),
        "Configuration creation took {:?}, expected <10ms",
        config_time
    );

    // Validation should be fast
    let validation_start = Instant::now();
    assert!(config.validate().is_ok());
    let validation_time = validation_start.elapsed();

    assert!(
        validation_time < Duration::from_millis(5),
        "Configuration validation took {:?}, expected <5ms",
        validation_time
    );
}

/// Test environment variable configuration loading
#[tokio::test]
async fn anchor_test_monitoring_env_configuration() {
    // Set test environment variables
    std::env::set_var("FORTITUDE_MONITORING_ENABLED", "true");
    std::env::set_var("FORTITUDE_MONITORING_TARGET_RESPONSE_MS", "150");
    std::env::set_var("FORTITUDE_MONITORING_CPU_WARNING", "60.0");
    std::env::set_var("FORTITUDE_MONITORING_MEMORY_CRITICAL", "90.0");

    let config = MonitoringConfiguration::from_env().expect("Failed to load config from env");

    assert!(config.core.enabled);
    assert_eq!(config.performance.response_time.target_ms, 150);
    assert_eq!(config.performance.resource_utilization.cpu.warning, 60.0);
    assert_eq!(
        config.performance.resource_utilization.memory.critical,
        90.0
    );

    // Clean up
    std::env::remove_var("FORTITUDE_MONITORING_ENABLED");
    std::env::remove_var("FORTITUDE_MONITORING_TARGET_RESPONSE_MS");
    std::env::remove_var("FORTITUDE_MONITORING_CPU_WARNING");
    std::env::remove_var("FORTITUDE_MONITORING_MEMORY_CRITICAL");
}

/// Test API monitoring service creation and basic functionality
#[cfg(feature = "api-server")]
#[tokio::test]
async fn anchor_test_api_monitoring_service() {
    use axum::http::{Method, StatusCode};
    use fortitude_api_server::middleware::monitoring::{ApiMonitoringService, PerformanceSummary};

    let service = ApiMonitoringService::for_api_server();

    // Verify component name
    assert_eq!(service.component_name(), "api-server");

    // Check initial health status
    let health = service
        .get_health_status()
        .await
        .expect("Failed to get health status");
    assert_eq!(health.component_name, "api-server");
    assert_eq!(health.status, HealthStatus::Healthy);

    // Record some test requests
    let requests = vec![
        ("GET", "/api/v1/health", StatusCode::OK, 150),
        ("POST", "/api/v1/research", StatusCode::OK, 200),
        ("GET", "/api/v1/classify", StatusCode::OK, 180),
        ("DELETE", "/api/v1/cache/test", StatusCode::NOT_FOUND, 50),
        (
            "POST",
            "/api/v1/research",
            StatusCode::INTERNAL_SERVER_ERROR,
            100,
        ),
    ];

    for (method, path, status, duration_ms) in requests {
        let method = method.parse::<Method>().unwrap();
        service
            .record_request(
                &method,
                path,
                status,
                Duration::from_millis(duration_ms),
                Some(1024),
            )
            .await
            .expect("Failed to record request");
    }

    // Get performance summary
    let summary = service
        .get_performance_summary()
        .await
        .expect("Failed to get performance summary");

    // Validate metrics
    assert_eq!(summary.total_requests, 5);
    assert_eq!(summary.success_rate, 60.0); // 3 out of 5 successful
    assert_eq!(summary.error_rate, 40.0); // 2 out of 5 failed
    assert!(summary.avg_response_time_ms > 0);
    assert!(summary.p95_response_time_ms >= summary.avg_response_time_ms);

    // Test component metrics interface
    let metrics = service
        .get_performance_metrics()
        .await
        .expect("Failed to get component metrics");
    assert_eq!(metrics.component_name, "api-server");
    assert_eq!(metrics.total_operations, 5);
    assert_eq!(metrics.successful_operations, 3);
    assert_eq!(metrics.failed_operations, 2);
}

/// Test MCP monitoring service creation and basic functionality
#[cfg(feature = "mcp-server")]
#[tokio::test]
async fn anchor_test_mcp_monitoring_service() {
    use fortitude_mcp_server::monitoring::{McpMonitoringService, McpPerformanceSummary};

    let service = McpMonitoringService::for_mcp_server();

    // Verify component name
    assert_eq!(service.component_name(), "mcp-server");

    // Check initial health status
    let health = service
        .get_health_status()
        .await
        .expect("Failed to get health status");
    assert_eq!(health.component_name, "mcp-server");
    assert_eq!(health.status, HealthStatus::Healthy);

    // Record some test tool calls
    let tool_calls = vec![
        ("research_analyze", true, 250),
        ("classification_classify", true, 150),
        ("proactive_gap_analyze", true, 300),
        ("research_analyze", false, 100),
        ("quality_validate", true, 200),
    ];

    for (tool_name, success, duration_ms) in tool_calls {
        service
            .record_tool_call(
                tool_name,
                Duration::from_millis(duration_ms),
                success,
                Some(1024),
                Some(2048),
            )
            .await
            .expect("Failed to record tool call");
    }

    // Record some resource reads
    let resource_reads = vec![
        ("/config/server.toml", true, 50),
        ("/docs/api.md", true, 75),
        ("/cache/test_key", false, 25),
    ];

    for (resource_uri, success, duration_ms) in resource_reads {
        service
            .record_resource_read(
                resource_uri,
                Duration::from_millis(duration_ms),
                success,
                Some(4096),
            )
            .await
            .expect("Failed to record resource read");
    }

    // Get performance summary
    let summary = service
        .get_performance_summary()
        .await
        .expect("Failed to get performance summary");

    // Validate metrics
    assert_eq!(summary.total_tool_calls, 5);
    assert_eq!(summary.total_resource_reads, 3);
    assert_eq!(summary.total_operations, 8);
    assert_eq!(summary.success_rate, 75.0); // 6 out of 8 successful
    assert_eq!(summary.error_rate, 25.0); // 2 out of 8 failed
    assert!(summary.avg_tool_call_time_ms > 0);
    assert!(summary.avg_resource_read_time_ms > 0);

    // Test authentication recording
    service
        .record_auth_attempt(true, Some("client_123"))
        .await
        .expect("Failed to record auth");
    service
        .record_auth_attempt(false, Some("client_456"))
        .await
        .expect("Failed to record auth");

    let updated_summary = service
        .get_performance_summary()
        .await
        .expect("Failed to get updated summary");
    assert_eq!(updated_summary.auth_success_rate, 50.0); // 1 out of 2 successful
}

/// Test performance threshold monitoring and health status updates
#[tokio::test]
async fn anchor_test_performance_threshold_monitoring() {
    #[cfg(feature = "api-server")]
    {
        use axum::http::{Method, StatusCode};
        use fortitude_api_server::middleware::monitoring::ApiMonitoringService;

        let service = ApiMonitoringService::for_api_server();

        // Record a request that exceeds the critical threshold (500ms for API server)
        service
            .record_request(
                &Method::GET,
                "/api/v1/slow-endpoint",
                StatusCode::OK,
                Duration::from_millis(600), // Exceeds critical threshold
                Some(1024),
            )
            .await
            .expect("Failed to record slow request");

        // Health status should be degraded or critical
        let health = service
            .get_health_status()
            .await
            .expect("Failed to get health status");
        assert!(
            health.status == HealthStatus::Degraded || health.status == HealthStatus::Critical,
            "Expected degraded or critical health status, got {:?}",
            health.status
        );
    }
}

/// Test response time validation for <200ms target
#[tokio::test]
async fn anchor_test_response_time_validation() {
    let config = MonitoringConfiguration::for_api_server();

    // API server should have 200ms target
    assert_eq!(config.performance.response_time.target_ms, 200);

    // Simulate monitoring a fast operation
    let start = Instant::now();

    // Perform a lightweight operation (configuration validation)
    let validation_result = config.validate();

    let duration = start.elapsed();

    // Should complete successfully
    assert!(validation_result.is_ok());

    // Should meet the 200ms target with significant margin
    assert!(
        duration < Duration::from_millis(200),
        "Operation took {:?}, exceeds 200ms target",
        duration
    );

    // In fact, should be much faster for this lightweight operation
    assert!(
        duration < Duration::from_millis(50),
        "Configuration validation took {:?}, expected <50ms",
        duration
    );
}

/// Test concurrent monitoring operations performance
#[tokio::test]
async fn anchor_test_concurrent_monitoring_performance() {
    #[cfg(feature = "api-server")]
    {
        use axum::http::{Method, StatusCode};
        use fortitude_api_server::middleware::monitoring::ApiMonitoringService;
        use std::sync::Arc;

        let service = Arc::new(ApiMonitoringService::for_api_server());
        let num_concurrent = 100;
        let num_requests_per_task = 10;

        let start = Instant::now();

        // Spawn concurrent tasks that record metrics
        let mut handles = Vec::new();
        for task_id in 0..num_concurrent {
            let service = service.clone();
            let handle = tokio::spawn(async move {
                for req_id in 0..num_requests_per_task {
                    let path = format!("/api/v1/test/{}/{}", task_id, req_id);
                    service
                        .record_request(
                            &Method::GET,
                            &path,
                            StatusCode::OK,
                            Duration::from_millis(100),
                            Some(512),
                        )
                        .await
                        .expect("Failed to record request");
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.expect("Task failed");
        }

        let duration = start.elapsed();
        let total_requests = num_concurrent * num_requests_per_task;

        // All requests should complete within reasonable time
        assert!(
            duration < Duration::from_secs(10),
            "Concurrent monitoring took {:?} for {} requests",
            duration,
            total_requests
        );

        // Verify all requests were recorded
        let summary = service
            .get_performance_summary()
            .await
            .expect("Failed to get summary");
        assert_eq!(summary.total_requests, total_requests as u64);
        assert_eq!(summary.success_rate, 100.0);

        // Calculate throughput
        let requests_per_second = total_requests as f64 / duration.as_secs_f64();
        println!(
            "Monitoring throughput: {:.2} requests/second",
            requests_per_second
        );

        // Should handle at least 100 requests per second
        assert!(
            requests_per_second > 100.0,
            "Monitoring throughput {:.2} req/s is below minimum 100 req/s",
            requests_per_second
        );
    }
}

/// Test monitoring memory usage and limits
#[tokio::test]
async fn anchor_test_monitoring_memory_limits() {
    #[cfg(feature = "api-server")]
    {
        use axum::http::{Method, StatusCode};
        use fortitude_api_server::middleware::monitoring::ApiMonitoringService;

        let service = ApiMonitoringService::for_api_server();

        // Record many requests to test memory limit enforcement
        let max_samples = 1000; // Less than the configured limit

        for i in 0..max_samples * 2 {
            service
                .record_request(
                    &Method::GET,
                    &format!("/test/{}", i),
                    StatusCode::OK,
                    Duration::from_millis(100 + (i % 100) as u64),
                    Some(1024),
                )
                .await
                .expect("Failed to record request");
        }

        let metrics = service.get_api_metrics().await;

        // Should not exceed the configured memory limit
        assert!(
            metrics.duration_samples.len() <= max_samples,
            "Duration samples {} exceeds limit {}",
            metrics.duration_samples.len(),
            max_samples
        );

        // Should still have collected recent samples
        assert!(
            !metrics.duration_samples.is_empty(),
            "Duration samples should not be empty"
        );

        // Total request count should still be accurate
        assert_eq!(metrics.total_requests, max_samples as u64 * 2);
    }
}

/// Test monitoring error handling and resilience
#[tokio::test]
async fn anchor_test_monitoring_error_resilience() {
    #[cfg(feature = "api-server")]
    {
        use axum::http::{Method, StatusCode};
        use fortitude_api_server::middleware::monitoring::ApiMonitoringService;

        let service = ApiMonitoringService::for_api_server();

        // Record various error conditions
        let test_cases = vec![
            // Normal successful request
            (StatusCode::OK, true),
            // Client errors (should not affect health critically)
            (StatusCode::BAD_REQUEST, false),
            (StatusCode::NOT_FOUND, false),
            (StatusCode::UNAUTHORIZED, false),
            // Server errors (more concerning)
            (StatusCode::INTERNAL_SERVER_ERROR, false),
            (StatusCode::SERVICE_UNAVAILABLE, false),
        ];

        for (status_code, _expected_success) in test_cases {
            let result = service
                .record_request(
                    &Method::GET,
                    "/test/endpoint",
                    status_code,
                    Duration::from_millis(150),
                    Some(512),
                )
                .await;

            // Recording should never fail even with error status codes
            assert!(
                result.is_ok(),
                "Failed to record request with status {}",
                status_code
            );
        }

        // Service should still be operational
        let health = service
            .get_health_status()
            .await
            .expect("Failed to get health status");
        assert!(
            health.status != HealthStatus::Critical,
            "Health status should not be critical from error codes alone"
        );

        let summary = service
            .get_performance_summary()
            .await
            .expect("Failed to get summary");
        assert_eq!(summary.total_requests, 6);
        assert!(
            summary.error_rate > 0.0,
            "Error rate should be greater than 0"
        );
    }
}

/// Test custom metrics functionality
#[tokio::test]
async fn anchor_test_custom_metrics() {
    #[cfg(feature = "api-server")]
    {
        use fortitude_api_server::middleware::monitoring::ApiMonitoringService;

        let service = ApiMonitoringService::for_api_server();

        // Record various custom metrics
        let custom_metrics = vec![
            ("cache_hit_rate", 0.85),
            ("active_connections", 42.0),
            ("memory_usage_mb", 256.0),
            ("cpu_usage_percent", 15.5),
            ("disk_usage_percent", 45.2),
        ];

        for (metric_name, value) in &custom_metrics {
            service
                .record_custom_metric(metric_name, *value)
                .await
                .expect("Failed to record custom metric");
        }

        // Verify custom metrics are retrievable
        let component_metrics = service
            .get_performance_metrics()
            .await
            .expect("Failed to get component metrics");

        for (metric_name, expected_value) in custom_metrics {
            let actual_value = component_metrics.custom_metrics.get(&metric_name);
            assert_eq!(
                actual_value,
                Some(&expected_value),
                "Custom metric {} not found or incorrect value",
                metric_name
            );
        }
    }
}

/// Comprehensive integration test across monitoring system
#[tokio::test]
async fn anchor_test_monitoring_integration() {
    // Test that monitoring configuration, API monitoring, and MCP monitoring
    // all work together correctly

    let api_config = MonitoringConfiguration::for_api_server();
    let mcp_config = MonitoringConfiguration::for_mcp_server();

    // Both configs should be valid
    assert!(api_config.validate().is_ok());
    assert!(mcp_config.validate().is_ok());

    // API config should be optimized for lower latency
    assert!(
        api_config.performance.response_time.target_ms
            <= mcp_config.performance.response_time.target_ms
    );

    // Both should have appropriate component configurations
    assert!(api_config.components.contains_key("api-server"));
    assert!(mcp_config.components.contains_key("mcp-server"));

    // Performance requirements should be consistent
    assert!(api_config.performance.error_rates.critical_percent > 0.0);
    assert!(mcp_config.performance.error_rates.critical_percent > 0.0);
}

/// Test monitoring system startup and shutdown performance
#[tokio::test]
async fn anchor_test_monitoring_startup_shutdown() {
    // Test that monitoring system components can be created and cleaned up quickly

    let startup_start = Instant::now();

    // Create monitoring configurations
    let api_config = MonitoringConfiguration::for_api_server();
    let mcp_config = MonitoringConfiguration::for_mcp_server();

    // Validate configurations
    assert!(api_config.validate().is_ok());
    assert!(mcp_config.validate().is_ok());

    #[cfg(feature = "api-server")]
    {
        // Create API monitoring service
        let _api_service =
            fortitude_api_server::middleware::monitoring::ApiMonitoringService::for_api_server();
    }

    #[cfg(feature = "mcp-server")]
    {
        // Create MCP monitoring service
        let _mcp_service = fortitude_mcp_server::monitoring::McpMonitoringService::for_mcp_server();
    }

    let startup_duration = startup_start.elapsed();

    // Startup should be very fast
    assert!(
        startup_duration < Duration::from_millis(100),
        "Monitoring system startup took {:?}, expected <100ms",
        startup_duration
    );

    // Cleanup happens automatically via Drop
    // Test that cleanup is also fast by just measuring scope exit
    let cleanup_start = Instant::now();
    drop(api_config);
    drop(mcp_config);
    let cleanup_duration = cleanup_start.elapsed();

    assert!(
        cleanup_duration < Duration::from_millis(10),
        "Monitoring system cleanup took {:?}, expected <10ms",
        cleanup_duration
    );
}

/// Test performance under high frequency metric collection
#[tokio::test]
async fn anchor_test_high_frequency_metrics() {
    #[cfg(feature = "api-server")]
    {
        use axum::http::{Method, StatusCode};
        use fortitude_api_server::middleware::monitoring::ApiMonitoringService;

        let service = ApiMonitoringService::for_api_server();
        let start = Instant::now();
        let test_duration = Duration::from_secs(1);
        let mut request_count = 0;

        // Record metrics as fast as possible for 1 second
        while start.elapsed() < test_duration {
            service
                .record_request(
                    &Method::GET,
                    "/high-frequency-test",
                    StatusCode::OK,
                    Duration::from_millis(10),
                    Some(256),
                )
                .await
                .expect("Failed to record request");

            request_count += 1;

            // Small yield to prevent blocking
            if request_count % 100 == 0 {
                tokio::task::yield_now().await;
            }
        }

        let actual_duration = start.elapsed();
        let rate = request_count as f64 / actual_duration.as_secs_f64();

        println!(
            "High frequency test: {} requests in {:?} ({:.0} req/s)",
            request_count, actual_duration, rate
        );

        // Should handle at least 1000 requests per second
        assert!(
            rate > 1000.0,
            "High frequency monitoring rate {:.0} req/s below minimum 1000 req/s",
            rate
        );

        // Verify metrics are still accurate
        let summary = service
            .get_performance_summary()
            .await
            .expect("Failed to get summary");
        assert_eq!(summary.total_requests, request_count);
        assert_eq!(summary.success_rate, 100.0);
    }
}

/// Final validation test - ensure monitoring meets all requirements
#[tokio::test]
async fn anchor_test_monitoring_requirements_validation() {
    // Requirement 1: Configuration system exists and works
    let config = MonitoringConfiguration::default();
    assert!(config.validate().is_ok());

    // Requirement 2: API server integration exists
    #[cfg(feature = "api-server")]
    {
        let _api_service =
            fortitude_api_server::middleware::monitoring::ApiMonitoringService::for_api_server();
    }

    // Requirement 3: MCP server integration exists
    #[cfg(feature = "mcp-server")]
    {
        let _mcp_service = fortitude_mcp_server::monitoring::McpMonitoringService::for_mcp_server();
    }

    // Requirement 4: <200ms response time target for API server
    let api_config = MonitoringConfiguration::for_api_server();
    assert_eq!(api_config.performance.response_time.target_ms, 200);

    // Requirement 5: Performance thresholds are configurable
    assert!(
        api_config.performance.response_time.warning_ms
            > api_config.performance.response_time.target_ms
    );
    assert!(
        api_config.performance.response_time.critical_ms
            > api_config.performance.response_time.warning_ms
    );

    // Requirement 6: Health monitoring is available
    let health_check_enabled = api_config.core.enabled && api_config.performance.enabled;
    assert!(health_check_enabled);

    // Requirement 7: Custom metrics are supported
    let supports_custom_metrics = api_config.collection.enable_custom_metrics;
    assert!(supports_custom_metrics);

    // Requirement 8: Environment configuration is supported
    let env_docs = MonitoringConfiguration::get_env_var_documentation();
    assert!(!env_docs.is_empty());
    assert!(env_docs.iter().any(|(var, _)| var.contains("RESPONSE_MS")));

    println!("✓ All monitoring system requirements validated successfully");
}
