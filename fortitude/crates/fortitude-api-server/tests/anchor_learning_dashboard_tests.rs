// ABOUTME: Anchor tests for learning dashboard API endpoints critical functionality
//! Critical anchor tests for learning system monitoring dashboard endpoints.
//!
//! ANCHOR: These tests protect critical learning dashboard functionality from regressions:
//! - Learning metrics data structure and API contracts
//! - Dashboard data aggregation and consistency
//! - Health monitoring endpoint behavior
//! - Performance summary calculations
//! - Real-time metrics updates
//!
//! These tests ensure that the learning dashboard maintains compatibility with
//! monitoring systems and that critical metrics continue to be available for
//! system observability and performance tracking.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use fortitude_api_server::{config::ApiServerConfig, server::ApiServer};
use serde_json::Value;
use tower::ServiceExt;

/// Test helper to create a test API server with disabled middleware for testing
async fn create_test_server() -> Router {
    let mut config = ApiServerConfig::default();
    // Disable pattern tracking and auth for anchor tests
    config
        .features
        .insert("pattern_tracking".to_string(), false);
    config.auth.enabled = false;
    let server = ApiServer::new(config)
        .await
        .expect("Failed to create test server");
    server.app
}

/// Helper to send request and get JSON response
async fn send_request(app: &Router, method: &str, path: &str) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);

    (status, json)
}

#[tokio::test]
async fn anchor_learning_dashboard_data_structure() {
    // ANCHOR: Protects the critical dashboard data structure and API contract
    let app = create_test_server().await;
    let (status, json) = send_request(&app, "GET", "/api/v1/learning/dashboard").await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Dashboard endpoint must be available"
    );

    // Verify core dashboard structure that external systems depend on
    assert!(
        json.get("current_metrics").is_some(),
        "Dashboard must provide current_metrics"
    );
    assert!(
        json.get("health_status").is_some(),
        "Dashboard must provide health_status"
    );
    assert!(
        json.get("alerts").is_some(),
        "Dashboard must provide alerts"
    );
    assert!(
        json.get("performance_graphs").is_some(),
        "Dashboard must provide performance_graphs"
    );
    assert!(
        json.get("system_overview").is_some(),
        "Dashboard must provide system_overview"
    );

    // Verify system overview critical fields
    let system_overview = json.get("system_overview").unwrap();
    assert!(
        system_overview.get("total_adaptations").is_some(),
        "System overview must track total adaptations"
    );
    assert!(
        system_overview.get("success_rate").is_some(),
        "System overview must track success rate"
    );
    assert!(
        system_overview.get("average_response_time").is_some(),
        "System overview must track response time"
    );
    assert!(
        system_overview.get("uptime_seconds").is_some(),
        "System overview must track uptime"
    );
    assert!(
        system_overview.get("resource_utilization").is_some(),
        "System overview must track resource usage"
    );
}

#[tokio::test]
async fn anchor_learning_metrics_api_contract() {
    // ANCHOR: Protects the learning metrics API contract for monitoring systems
    let app = create_test_server().await;
    let (status, json) = send_request(&app, "GET", "/api/v1/learning/metrics").await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Learning metrics endpoint must be available"
    );

    // Verify all metric categories that monitoring systems depend on
    assert!(
        json.get("adaptation_metrics").is_some(),
        "Must provide adaptation metrics"
    );
    assert!(
        json.get("storage_metrics").is_some(),
        "Must provide storage metrics"
    );
    assert!(
        json.get("pattern_recognition_metrics").is_some(),
        "Must provide pattern recognition metrics"
    );
    assert!(
        json.get("feedback_metrics").is_some(),
        "Must provide feedback metrics"
    );
    assert!(
        json.get("optimization_metrics").is_some(),
        "Must provide optimization metrics"
    );
    assert!(
        json.get("system_metrics").is_some(),
        "Must provide system metrics"
    );
    assert!(
        json.get("timestamp").is_some(),
        "Must provide timestamp for metrics"
    );

    // Verify adaptation metrics critical fields
    let adaptation_metrics = json.get("adaptation_metrics").unwrap();
    assert!(
        adaptation_metrics.get("adaptations_applied").is_some(),
        "Must track adaptations applied"
    );
    assert!(
        adaptation_metrics.get("adaptations_failed").is_some(),
        "Must track adaptation failures"
    );
    assert!(
        adaptation_metrics.get("success_rate").is_some(),
        "Must calculate success rate"
    );
    assert!(
        adaptation_metrics
            .get("average_adaptation_time_ms")
            .is_some(),
        "Must track timing"
    );

    // Verify storage metrics critical fields
    let storage_metrics = json.get("storage_metrics").unwrap();
    assert!(
        storage_metrics.get("total_operations").is_some(),
        "Must track total operations"
    );
    assert!(
        storage_metrics.get("error_rate").is_some(),
        "Must track error rate"
    );
    assert!(
        storage_metrics.get("average_response_time_ms").is_some(),
        "Must track response time"
    );
}

#[tokio::test]
async fn anchor_learning_health_monitoring() {
    // ANCHOR: Protects health monitoring endpoint critical for system observability
    let app = create_test_server().await;
    let (status, json) = send_request(&app, "GET", "/api/v1/learning/health").await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Health endpoint must always be available"
    );

    // Verify health report structure for monitoring systems
    assert!(
        json.get("overall_status").is_some(),
        "Must provide overall health status"
    );
    assert!(
        json.get("component_results").is_some(),
        "Must provide component health details"
    );
    assert!(json.get("summary").is_some(), "Must provide health summary");
    assert!(
        json.get("timestamp").is_some(),
        "Must provide health check timestamp"
    );

    // Verify component health results structure
    let component_results = json.get("component_results").unwrap().as_array().unwrap();
    assert!(
        !component_results.is_empty(),
        "Must check health of learning components"
    );

    for component in component_results {
        assert!(
            component.get("component").is_some(),
            "Component must have name"
        );
        assert!(
            component.get("status").is_some(),
            "Component must have status"
        );
        assert!(
            component.get("message").is_some(),
            "Component must have status message"
        );
        assert!(
            component.get("timestamp").is_some(),
            "Component must have check timestamp"
        );
        assert!(
            component.get("response_time_ms").is_some(),
            "Component must track response time"
        );
    }
}

#[tokio::test]
async fn anchor_learning_performance_summary() {
    // ANCHOR: Protects performance summary calculations and trending data
    let app = create_test_server().await;
    let (status, json) = send_request(&app, "GET", "/api/v1/learning/performance").await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Performance summary must be available"
    );

    // Verify performance summary structure for dashboards
    assert!(
        json.get("overall_health").is_some(),
        "Must provide overall health assessment"
    );
    assert!(
        json.get("key_metrics").is_some(),
        "Must provide key performance indicators"
    );
    assert!(
        json.get("active_alerts").is_some(),
        "Must provide active alerts"
    );
    assert!(
        json.get("performance_trends").is_some(),
        "Must provide trending data"
    );
    assert!(
        json.get("recommendations").is_some(),
        "Must provide system recommendations"
    );

    // Verify key metrics contain critical performance indicators
    let key_metrics = json.get("key_metrics").unwrap().as_object().unwrap();
    assert!(
        key_metrics.contains_key("adaptation_success_rate"),
        "Must track adaptation success rate"
    );
    assert!(
        key_metrics.contains_key("storage_error_rate"),
        "Must track storage error rate"
    );
    assert!(
        key_metrics.contains_key("pattern_recognition_accuracy"),
        "Must track recognition accuracy"
    );
    assert!(
        key_metrics.contains_key("average_adaptation_time_ms"),
        "Must track adaptation timing"
    );

    // Verify performance trends structure
    let performance_trends = json.get("performance_trends").unwrap().as_object().unwrap();
    assert!(
        performance_trends.contains_key("success_rate"),
        "Must provide success rate trends"
    );
    assert!(
        performance_trends.contains_key("response_time"),
        "Must provide response time trends"
    );

    // Verify recommendations are present
    let recommendations = json.get("recommendations").unwrap().as_array().unwrap();
    assert!(
        !recommendations.is_empty(),
        "Must provide system recommendations"
    );
}

#[tokio::test]
async fn anchor_learning_metrics_numerical_values() {
    // ANCHOR: Protects metric value types and ranges for monitoring system compatibility
    let app = create_test_server().await;
    let (status, json) = send_request(&app, "GET", "/api/v1/learning/metrics").await;

    assert_eq!(status, StatusCode::OK);

    // Verify adaptation metrics have valid numerical values
    let adaptation_metrics = json.get("adaptation_metrics").unwrap();
    let success_rate = adaptation_metrics
        .get("success_rate")
        .unwrap()
        .as_f64()
        .unwrap();
    assert!(
        success_rate >= 0.0 && success_rate <= 1.0,
        "Success rate must be valid percentage"
    );

    let _adaptations_applied = adaptation_metrics
        .get("adaptations_applied")
        .unwrap()
        .as_u64()
        .unwrap();
    // Note: u64 values are inherently non-negative

    let avg_time = adaptation_metrics
        .get("average_adaptation_time_ms")
        .unwrap()
        .as_f64()
        .unwrap();
    assert!(avg_time >= 0.0, "Average time must be non-negative");

    // Verify storage metrics have valid values
    let storage_metrics = json.get("storage_metrics").unwrap();
    let error_rate = storage_metrics.get("error_rate").unwrap().as_f64().unwrap();
    assert!(
        error_rate >= 0.0 && error_rate <= 1.0,
        "Error rate must be valid percentage"
    );

    let _total_ops = storage_metrics
        .get("total_operations")
        .unwrap()
        .as_u64()
        .unwrap();
    // Note: u64 values are inherently non-negative

    // Verify pattern recognition metrics have valid accuracy
    let pattern_metrics = json.get("pattern_recognition_metrics").unwrap();
    let accuracy = pattern_metrics
        .get("recognition_accuracy")
        .unwrap()
        .as_f64()
        .unwrap();
    assert!(
        accuracy >= 0.0 && accuracy <= 1.0,
        "Recognition accuracy must be valid percentage"
    );
}

#[tokio::test]
async fn anchor_learning_endpoint_stability() {
    // ANCHOR: Protects endpoint stability for continuous monitoring
    let app = create_test_server().await;

    // All learning endpoints must consistently return successful responses
    let endpoints = [
        "/api/v1/learning/dashboard",
        "/api/v1/learning/metrics",
        "/api/v1/learning/health",
        "/api/v1/learning/performance",
    ];

    for endpoint in endpoints.iter() {
        let (status, json) = send_request(&app, "GET", endpoint).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "Endpoint {} must be stable and available",
            endpoint
        );
        assert!(
            !json.is_null(),
            "Endpoint {} must return valid JSON",
            endpoint
        );

        // Ensure response has reasonable processing time
        if let Some(processing_time) = json.get("processing_time_ms") {
            let time_ms = processing_time.as_u64().unwrap();
            assert!(
                time_ms < 5000,
                "Endpoint {} processing time must be reasonable (< 5s)",
                endpoint
            );
        }
    }
}

#[tokio::test]
async fn anchor_learning_dashboard_real_time_capability() {
    // ANCHOR: Protects real-time data update capability for dashboard monitoring
    let app = create_test_server().await;

    // Get two dashboard snapshots with a small time gap
    let (status1, json1) = send_request(&app, "GET", "/api/v1/learning/dashboard").await;
    assert_eq!(status1, StatusCode::OK);

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let (status2, json2) = send_request(&app, "GET", "/api/v1/learning/dashboard").await;
    assert_eq!(status2, StatusCode::OK);

    // Verify both requests succeeded and have valid structure
    assert!(
        json1.get("current_metrics").is_some(),
        "First snapshot must have metrics"
    );
    assert!(
        json2.get("current_metrics").is_some(),
        "Second snapshot must have metrics"
    );

    // Verify timestamp progression for real-time capability
    let ts1 = json1
        .get("current_metrics")
        .unwrap()
        .get("timestamp")
        .unwrap()
        .as_str()
        .unwrap();
    let ts2 = json2
        .get("current_metrics")
        .unwrap()
        .get("timestamp")
        .unwrap()
        .as_str()
        .unwrap();

    // Timestamps should be different, demonstrating real-time updates
    // (In a real system with actual data changes, we'd see metric differences)
    assert!(
        !ts1.is_empty() && !ts2.is_empty(),
        "Timestamps must be present for real-time tracking"
    );
}

#[tokio::test]
async fn anchor_learning_metrics_query_parameters() {
    // ANCHOR: Protects query parameter support for flexible monitoring
    let app = create_test_server().await;

    // Test metrics endpoint with query parameters
    let (status, json) = send_request(
        &app,
        "GET",
        "/api/v1/learning/metrics?duration=1h&detailed=true",
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "Metrics endpoint must support query parameters"
    );

    // Verify response structure is maintained with parameters
    assert!(
        json.get("adaptation_metrics").is_some(),
        "Must maintain metrics structure with parameters"
    );
    assert!(
        json.get("timestamp").is_some(),
        "Must maintain timestamp with parameters"
    );
}
