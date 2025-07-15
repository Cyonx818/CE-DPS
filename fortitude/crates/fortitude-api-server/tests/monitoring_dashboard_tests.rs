// ABOUTME: Comprehensive tests for monitoring dashboard endpoints following TDD approach

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use fortitude_api_server::{
    config::ApiServerConfig,
    models::responses::{
        ApiResponse, MonitoringAlertSeverity, MonitoringAlertsResponse,
        MonitoringCurrentMetricsResponse, MonitoringDashboardResponse, MonitoringHealthResponse,
        MonitoringHealthStatus, MonitoringPerformanceSummaryResponse,
    },
    routes::monitoring::MonitoringState,
    server::ApiServer,
};
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

/// Helper function to create test API server with monitoring endpoints
async fn create_test_server() -> ApiServer {
    let mut config = ApiServerConfig::default();
    config.auth.enabled = false; // Disable auth for testing
    config
        .features
        .insert("pattern_tracking".to_string(), false); // Disable pattern tracking for testing

    ApiServer::new(config)
        .await
        .expect("Failed to create test server")
}

/// Helper function to make HTTP requests to the test server
async fn make_request(server: &ApiServer, method: Method, path: &str) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .body(Body::empty())
        .unwrap();

    let response = server.app.clone().oneshot(request).await.unwrap();
    let status = response.status();

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({}));

    (status, body)
}

#[tokio::test]
async fn test_monitoring_dashboard_endpoint_success() {
    // TDD: Test that monitoring dashboard endpoint exists and returns 200
    let server = create_test_server().await;

    let (status, body) = make_request(&server, Method::GET, "/api/v1/monitoring/dashboard").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.get("overall_status").is_some());
    assert!(body.get("current_metrics").is_some());
    assert!(body.get("health_status").is_some());
    assert!(body.get("active_alerts").is_some());
    assert!(body.get("system_overview").is_some());
    assert!(body.get("processing_time_ms").is_some());
}

#[tokio::test]
async fn test_monitoring_dashboard_with_query_parameters() {
    // TDD: Test dashboard endpoint with time range and graph parameters
    let server = create_test_server().await;

    let (status, body) = make_request(
        &server,
        Method::GET,
        "/api/v1/monitoring/dashboard?time_range_hours=12&include_graphs=true",
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    let data = &body;
    // When include_graphs=true, performance_graphs should be populated
    let graphs = data.get("performance_graphs").unwrap();
    assert!(graphs.is_object());
    // Should have at least response_time, error_rate, and cpu_usage graphs
    assert!(graphs.get("response_time_ms").is_some());
    assert!(graphs.get("error_rate").is_some());
    assert!(graphs.get("cpu_usage_percent").is_some());
}

#[tokio::test]
async fn test_monitoring_metrics_endpoint() {
    // TDD: Test that metrics endpoint returns current system metrics
    let server = create_test_server().await;

    let (status, body) = make_request(&server, Method::GET, "/api/v1/monitoring/metrics").await;

    assert_eq!(status, StatusCode::OK);

    let data = &body;
    let metrics = data;

    // Verify all required metrics categories are present
    assert!(metrics.get("api_metrics").is_some());
    assert!(metrics.get("provider_metrics").is_some());
    assert!(metrics.get("quality_metrics").is_some());
    assert!(metrics.get("cache_metrics").is_some());
    assert!(metrics.get("learning_metrics").is_some());
    assert!(metrics.get("resource_metrics").is_some());
    assert!(metrics.get("timestamp").is_some());

    // Verify API metrics structure
    let api_metrics = metrics.get("api_metrics").unwrap();
    assert!(api_metrics.get("total_requests").is_some());
    assert!(api_metrics.get("successful_requests").is_some());
    assert!(api_metrics.get("failed_requests").is_some());
    assert!(api_metrics.get("average_response_time_ms").is_some());
    assert!(api_metrics.get("error_rate").is_some());
}

#[tokio::test]
async fn test_monitoring_health_endpoint() {
    // TDD: Test that health endpoint returns system health status
    let server = create_test_server().await;

    let (status, body) = make_request(&server, Method::GET, "/api/v1/monitoring/health").await;

    assert_eq!(status, StatusCode::OK);

    let data = &body;
    let health = data.get("health").unwrap();

    // Verify health response structure
    assert!(health.get("overall_status").is_some());
    assert!(health.get("component_results").is_some());
    assert!(health.get("summary").is_some());
    assert!(health.get("timestamp").is_some());

    // Verify overall_status is a valid health status
    let overall_status = health.get("overall_status").unwrap().as_str().unwrap();
    assert!(["Healthy", "Warning", "Critical", "Unknown"].contains(&overall_status));

    // Verify component_results is an array
    let components = health.get("component_results").unwrap();
    assert!(components.is_array());
}

#[tokio::test]
async fn test_monitoring_alerts_endpoint() {
    // TDD: Test that alerts endpoint returns monitoring alerts with pagination
    let server = create_test_server().await;

    let (status, body) = make_request(&server, Method::GET, "/api/v1/monitoring/alerts").await;

    assert_eq!(status, StatusCode::OK);

    let data = &body;

    // Verify alerts response structure
    assert!(data.get("alerts").is_some());
    assert!(data.get("total_count").is_some());
    assert!(data.get("unacknowledged_count").is_some());
    assert!(data.get("pagination").is_some());

    // Verify alerts is an array
    let alerts = data.get("alerts").unwrap();
    assert!(alerts.is_array());

    // Verify pagination structure
    let pagination = data.get("pagination").unwrap();
    assert!(pagination.get("offset").is_some());
    assert!(pagination.get("limit").is_some());
    assert!(pagination.get("total_pages").is_some());
    assert!(pagination.get("has_more").is_some());
}

#[tokio::test]
async fn test_monitoring_alerts_with_pagination() {
    // TDD: Test alerts endpoint with pagination parameters
    let server = create_test_server().await;

    let (status, body) = make_request(
        &server,
        Method::GET,
        "/api/v1/monitoring/alerts?limit=10&offset=5",
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    let data = &body;
    let pagination = data.get("pagination").unwrap();

    // Verify pagination parameters were applied
    assert_eq!(pagination.get("limit").unwrap().as_u64().unwrap(), 10);
    assert_eq!(pagination.get("offset").unwrap().as_u64().unwrap(), 5);
}

#[tokio::test]
async fn test_monitoring_performance_summary_endpoint() {
    // TDD: Test that performance summary endpoint returns comprehensive summary
    let server = create_test_server().await;

    let (status, body) = make_request(&server, Method::GET, "/api/v1/monitoring/performance").await;

    assert_eq!(status, StatusCode::OK);

    let data = &body;

    // Verify performance summary structure
    assert!(data.get("overall_health").is_some());
    assert!(data.get("key_metrics").is_some());
    assert!(data.get("active_alerts").is_some());
    assert!(data.get("performance_trends").is_some());
    assert!(data.get("recommendations").is_some());

    // Verify key_metrics is an object with expected metrics
    let key_metrics = data.get("key_metrics").unwrap();
    assert!(key_metrics.is_object());
    assert!(key_metrics.get("response_time_ms").is_some());
    assert!(key_metrics.get("error_rate").is_some());
    assert!(key_metrics.get("cpu_usage_percent").is_some());
    assert!(key_metrics.get("memory_usage_mb").is_some());

    // Verify performance_trends is an object
    let trends = data.get("performance_trends").unwrap();
    assert!(trends.is_object());

    // Verify recommendations is an array
    let recommendations = data.get("recommendations").unwrap();
    assert!(recommendations.is_array());
}

#[tokio::test]
async fn test_monitoring_endpoints_response_format() {
    // TDD: Test that all monitoring endpoints follow the ApiResponse format
    let server = create_test_server().await;

    let endpoints = [
        "/api/v1/monitoring/dashboard",
        "/api/v1/monitoring/metrics",
        "/api/v1/monitoring/health",
        "/api/v1/monitoring/alerts",
        "/api/v1/monitoring/performance",
    ];

    for endpoint in endpoints {
        let (status, body) = make_request(&server, Method::GET, endpoint).await;

        assert_eq!(status, StatusCode::OK, "Endpoint {} failed", endpoint);

        // Verify ApiResponse structure
        assert!(
            body.get("data").is_some(),
            "Missing 'data' field in {}",
            endpoint
        );
        assert!(
            body.get("request_id").is_some(),
            "Missing 'request_id' field in {}",
            endpoint
        );
        assert!(
            body.get("timestamp").is_some(),
            "Missing 'timestamp' field in {}",
            endpoint
        );
        assert!(
            body.get("success").is_some(),
            "Missing 'success' field in {}",
            endpoint
        );
        assert!(
            body.get("success").unwrap().as_bool().unwrap(),
            "Success should be true for {}",
            endpoint
        );
    }
}

#[tokio::test]
async fn test_monitoring_dashboard_data_integrity() {
    // TDD: Test that dashboard data is consistent and valid
    let server = create_test_server().await;

    let (status, body) = make_request(&server, Method::GET, "/api/v1/monitoring/dashboard").await;

    assert_eq!(status, StatusCode::OK);

    let data = &body;
    let metrics = data.get("current_metrics").unwrap();
    let overview = data.get("system_overview").unwrap();

    // Verify data consistency between metrics and overview
    let api_metrics = metrics.get("api_metrics").unwrap();
    let total_operations = overview.get("total_operations").unwrap().as_u64().unwrap();
    let api_total_requests = api_metrics.get("total_requests").unwrap().as_u64().unwrap();

    assert_eq!(
        total_operations, api_total_requests,
        "Total operations should match API total requests"
    );

    // Verify success rate calculation
    let success_rate = overview.get("success_rate").unwrap().as_f64().unwrap();
    let successful_requests = api_metrics
        .get("successful_requests")
        .unwrap()
        .as_u64()
        .unwrap();

    if total_operations > 0 {
        let expected_success_rate = successful_requests as f64 / total_operations as f64;
        assert!(
            (success_rate - expected_success_rate).abs() < 0.001,
            "Success rate calculation incorrect"
        );
    }

    // Verify health status consistency
    let health_status = data.get("health_status").unwrap();
    let overall_status = data.get("overall_status").unwrap().as_str().unwrap();
    let health_overall_status = health_status
        .get("overall_status")
        .unwrap()
        .as_str()
        .unwrap();

    // Map health status to overall status string
    let expected_status = match health_overall_status {
        "Healthy" => "healthy",
        "Warning" => "warning",
        "Critical" => "critical",
        "Unknown" => "unknown",
        _ => panic!("Invalid health status: {}", health_overall_status),
    };

    assert_eq!(
        overall_status, expected_status,
        "Overall status should match health status"
    );
}

#[tokio::test]
async fn test_monitoring_error_handling() {
    // TDD: Test that monitoring endpoints handle errors gracefully
    let server = create_test_server().await;

    // Test with invalid query parameters
    let (status, _) = make_request(
        &server,
        Method::GET,
        "/api/v1/monitoring/alerts?limit=invalid",
    )
    .await;

    // Should handle invalid parameters gracefully (either 400 or fallback to defaults)
    assert!(status == StatusCode::BAD_REQUEST || status == StatusCode::OK);
}

#[tokio::test]
async fn test_monitoring_performance_metrics() {
    // TDD: Test that monitoring endpoints themselves are performant
    let server = create_test_server().await;

    let start_time = std::time::Instant::now();
    let (status, body) = make_request(&server, Method::GET, "/api/v1/monitoring/dashboard").await;
    let elapsed = start_time.elapsed();

    assert_eq!(status, StatusCode::OK);

    // Verify response time is reasonable (less than 1 second for dashboard)
    assert!(
        elapsed.as_secs() < 1,
        "Dashboard endpoint took too long: {:?}",
        elapsed
    );

    // Verify processing_time_ms is reported
    let data = &body;
    let processing_time = data.get("processing_time_ms").unwrap().as_u64().unwrap();
    assert!(
        processing_time > 0,
        "Processing time should be greater than 0"
    );
    assert!(
        processing_time < 1000,
        "Processing time should be less than 1000ms, got {}",
        processing_time
    );
}

#[tokio::test]
async fn test_monitoring_state_initialization() {
    // TDD: Test that MonitoringState can be initialized successfully
    let state = MonitoringState::new().await;
    assert!(
        state.is_ok(),
        "MonitoringState initialization should succeed"
    );
}

// Integration tests
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_monitoring_workflow() {
        // TDD: Test a complete monitoring workflow
        let server = create_test_server().await;

        // 1. Get dashboard data
        let (status, dashboard_body) =
            make_request(&server, Method::GET, "/api/v1/monitoring/dashboard").await;
        assert_eq!(status, StatusCode::OK);

        // 2. Get detailed metrics
        let (status, metrics_body) =
            make_request(&server, Method::GET, "/api/v1/monitoring/metrics").await;
        assert_eq!(status, StatusCode::OK);

        // 3. Get health status
        let (status, health_body) =
            make_request(&server, Method::GET, "/api/v1/monitoring/health").await;
        assert_eq!(status, StatusCode::OK);

        // 4. Get alerts
        let (status, alerts_body) =
            make_request(&server, Method::GET, "/api/v1/monitoring/alerts").await;
        assert_eq!(status, StatusCode::OK);

        // 5. Get performance summary
        let (status, perf_body) =
            make_request(&server, Method::GET, "/api/v1/monitoring/performance").await;
        assert_eq!(status, StatusCode::OK);

        // Verify all responses have consistent timestamps (within reasonable window)
        let dashboard_ts = dashboard_body["timestamp"].as_str().unwrap();
        let metrics_ts = metrics_body["timestamp"].as_str().unwrap();
        let health_ts = health_body["timestamp"].as_str().unwrap();
        let alerts_ts = alerts_body["timestamp"].as_str().unwrap();
        let perf_ts = perf_body["timestamp"].as_str().unwrap();

        // All timestamps should be valid ISO 8601 format
        assert!(chrono::DateTime::parse_from_rfc3339(dashboard_ts).is_ok());
        assert!(chrono::DateTime::parse_from_rfc3339(metrics_ts).is_ok());
        assert!(chrono::DateTime::parse_from_rfc3339(health_ts).is_ok());
        assert!(chrono::DateTime::parse_from_rfc3339(alerts_ts).is_ok());
        assert!(chrono::DateTime::parse_from_rfc3339(perf_ts).is_ok());
    }

    #[tokio::test]
    async fn test_monitoring_data_aggregation() {
        // TDD: Test that monitoring data aggregation works correctly
        let server = create_test_server().await;

        // Get dashboard and individual metric endpoints
        let (_, dashboard_body) =
            make_request(&server, Method::GET, "/api/v1/monitoring/dashboard").await;
        let (_, metrics_body) =
            make_request(&server, Method::GET, "/api/v1/monitoring/metrics").await;
        let (_, health_body) =
            make_request(&server, Method::GET, "/api/v1/monitoring/health").await;

        // Verify dashboard aggregates data consistently
        let dashboard_data = dashboard_body.get("data").unwrap();
        let dashboard_metrics = dashboard_data.get("current_metrics").unwrap();
        let dashboard_health = dashboard_data.get("health_status").unwrap();

        let individual_metrics = metrics_body.get("data").unwrap().get("metrics").unwrap();
        let individual_health = health_body.get("data").unwrap().get("health").unwrap();

        // Compare key fields for consistency
        assert_eq!(
            dashboard_metrics
                .get("api_metrics")
                .unwrap()
                .get("total_requests"),
            individual_metrics
                .get("api_metrics")
                .unwrap()
                .get("total_requests")
        );

        assert_eq!(
            dashboard_health.get("overall_status"),
            individual_health.get("overall_status")
        );
    }
}

// Anchor tests for critical functionality
mod anchor_tests {
    use super::*;

    #[tokio::test]
    async fn anchor_test_monitoring_dashboard_availability() {
        // ANCHOR: Critical test - monitoring dashboard must always be available
        let server = create_test_server().await;

        let (status, body) =
            make_request(&server, Method::GET, "/api/v1/monitoring/dashboard").await;

        assert_eq!(
            status,
            StatusCode::OK,
            "Monitoring dashboard must be available"
        );
        assert!(body.get("data").is_some(), "Dashboard must return data");
        assert!(
            body.get("success").unwrap().as_bool().unwrap(),
            "Dashboard response must indicate success"
        );
    }

    #[tokio::test]
    async fn anchor_test_monitoring_health_status() {
        // ANCHOR: Critical test - health status must be reportable
        let server = create_test_server().await;

        let (status, body) = make_request(&server, Method::GET, "/api/v1/monitoring/health").await;

        assert_eq!(status, StatusCode::OK, "Health endpoint must be available");

        let health = body.get("data").unwrap().get("health").unwrap();
        let overall_status = health.get("overall_status").unwrap().as_str().unwrap();

        assert!(
            ["Healthy", "Warning", "Critical", "Unknown"].contains(&overall_status),
            "Health status must be valid: {}",
            overall_status
        );
    }

    #[tokio::test]
    async fn anchor_test_monitoring_response_format() {
        // ANCHOR: Critical test - all monitoring endpoints must follow ApiResponse format
        let server = create_test_server().await;
        let endpoints = [
            "/api/v1/monitoring/dashboard",
            "/api/v1/monitoring/metrics",
            "/api/v1/monitoring/health",
            "/api/v1/monitoring/alerts",
            "/api/v1/monitoring/performance",
        ];

        for endpoint in endpoints {
            let (status, body) = make_request(&server, Method::GET, endpoint).await;

            assert_eq!(
                status,
                StatusCode::OK,
                "Endpoint {} must be available",
                endpoint
            );

            // Must have ApiResponse structure
            assert!(
                body.get("data").is_some(),
                "Must have 'data' field for {}",
                endpoint
            );
            assert!(
                body.get("request_id").is_some(),
                "Must have 'request_id' field for {}",
                endpoint
            );
            assert!(
                body.get("timestamp").is_some(),
                "Must have 'timestamp' field for {}",
                endpoint
            );
            assert!(
                body.get("success").is_some(),
                "Must have 'success' field for {}",
                endpoint
            );

            // Request ID must be valid UUID
            let request_id = body.get("request_id").unwrap().as_str().unwrap();
            assert!(
                uuid::Uuid::parse_str(request_id).is_ok(),
                "Request ID must be valid UUID for {}",
                endpoint
            );
        }
    }
}
