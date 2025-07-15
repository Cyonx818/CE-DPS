// ABOUTME: Integration tests for learning dashboard API endpoints
//! Tests for learning metrics dashboard endpoints that integrate with the learning system

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use fortitude_api_server::{config::ApiServerConfig, server::ApiServer};
use serde_json::Value;
use std::time::Duration;
use tower::ServiceExt;

// Test helper to create a test API server
async fn create_test_server() -> Router {
    let mut config = ApiServerConfig::default();
    // Disable pattern tracking for tests to avoid middleware dependency issues
    config
        .features
        .insert("pattern_tracking".to_string(), false);
    // Disable authentication for tests
    config.auth.enabled = false;
    let server = ApiServer::new(config)
        .await
        .expect("Failed to create test server");
    server.app
}

// Test helper to send request and get response
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
async fn test_get_learning_dashboard_data_endpoint_exists() {
    let app = create_test_server().await;
    let (status, _) = send_request(&app, "GET", "/api/v1/learning/dashboard").await;

    // Should not return 404 (endpoint should exist)
    assert_ne!(
        status,
        StatusCode::NOT_FOUND,
        "Learning dashboard endpoint should exist"
    );
}

#[tokio::test]
async fn test_get_learning_metrics_endpoint_exists() {
    let app = create_test_server().await;
    let (status, _) = send_request(&app, "GET", "/api/v1/learning/metrics").await;

    // Should not return 404 (endpoint should exist)
    assert_ne!(
        status,
        StatusCode::NOT_FOUND,
        "Learning metrics endpoint should exist"
    );
}

#[tokio::test]
async fn test_get_learning_health_endpoint_exists() {
    let app = create_test_server().await;
    let (status, _) = send_request(&app, "GET", "/api/v1/learning/health").await;

    // Should not return 404 (endpoint should exist)
    assert_ne!(
        status,
        StatusCode::NOT_FOUND,
        "Learning health endpoint should exist"
    );
}

#[tokio::test]
async fn test_get_learning_performance_summary_endpoint_exists() {
    let app = create_test_server().await;
    let (status, _) = send_request(&app, "GET", "/api/v1/learning/performance").await;

    // Should not return 404 (endpoint should exist)
    assert_ne!(
        status,
        StatusCode::NOT_FOUND,
        "Learning performance endpoint should exist"
    );
}

#[tokio::test]
async fn test_learning_dashboard_data_response_structure() {
    let app = create_test_server().await;
    let (status, json) = send_request(&app, "GET", "/api/v1/learning/dashboard").await;

    // Should return success
    assert_eq!(
        status,
        StatusCode::OK,
        "Dashboard data endpoint should return 200"
    );

    // Should have expected dashboard structure
    assert!(
        json.get("current_metrics").is_some(),
        "Should have current_metrics field"
    );
    assert!(
        json.get("health_status").is_some(),
        "Should have health_status field"
    );
    assert!(json.get("alerts").is_some(), "Should have alerts field");
    assert!(
        json.get("performance_graphs").is_some(),
        "Should have performance_graphs field"
    );
    assert!(
        json.get("system_overview").is_some(),
        "Should have system_overview field"
    );
}

#[tokio::test]
async fn test_learning_metrics_response_structure() {
    let app = create_test_server().await;
    let (status, json) = send_request(&app, "GET", "/api/v1/learning/metrics").await;

    // Should return success
    assert_eq!(status, StatusCode::OK, "Metrics endpoint should return 200");

    // Should have expected metrics structure
    assert!(
        json.get("adaptation_metrics").is_some(),
        "Should have adaptation_metrics field"
    );
    assert!(
        json.get("storage_metrics").is_some(),
        "Should have storage_metrics field"
    );
    assert!(
        json.get("pattern_recognition_metrics").is_some(),
        "Should have pattern_recognition_metrics field"
    );
    assert!(
        json.get("feedback_metrics").is_some(),
        "Should have feedback_metrics field"
    );
    assert!(
        json.get("optimization_metrics").is_some(),
        "Should have optimization_metrics field"
    );
    assert!(
        json.get("system_metrics").is_some(),
        "Should have system_metrics field"
    );
    assert!(
        json.get("timestamp").is_some(),
        "Should have timestamp field"
    );
}

#[tokio::test]
async fn test_learning_health_response_structure() {
    let app = create_test_server().await;
    let (status, json) = send_request(&app, "GET", "/api/v1/learning/health").await;

    // Should return success
    assert_eq!(status, StatusCode::OK, "Health endpoint should return 200");

    // Should have expected health structure
    assert!(
        json.get("overall_status").is_some(),
        "Should have overall_status field"
    );
    assert!(
        json.get("component_results").is_some(),
        "Should have component_results field"
    );
    assert!(json.get("summary").is_some(), "Should have summary field");
    assert!(
        json.get("timestamp").is_some(),
        "Should have timestamp field"
    );
}

#[tokio::test]
async fn test_learning_performance_summary_response_structure() {
    let app = create_test_server().await;
    let (status, json) = send_request(&app, "GET", "/api/v1/learning/performance").await;

    // Should return success
    assert_eq!(
        status,
        StatusCode::OK,
        "Performance endpoint should return 200"
    );

    // Should have expected performance summary structure
    assert!(
        json.get("overall_health").is_some(),
        "Should have overall_health field"
    );
    assert!(
        json.get("key_metrics").is_some(),
        "Should have key_metrics field"
    );
    assert!(
        json.get("active_alerts").is_some(),
        "Should have active_alerts field"
    );
    assert!(
        json.get("performance_trends").is_some(),
        "Should have performance_trends field"
    );
    assert!(
        json.get("recommendations").is_some(),
        "Should have recommendations field"
    );
}

#[tokio::test]
async fn test_learning_metrics_with_query_parameters() {
    let app = create_test_server().await;
    let (status, _) = send_request(&app, "GET", "/api/v1/learning/metrics?duration=1h").await;

    // Should return success with query parameters
    assert_eq!(
        status,
        StatusCode::OK,
        "Metrics endpoint should handle query parameters"
    );
}

#[tokio::test]
async fn test_learning_dashboard_real_time_updates() {
    let app = create_test_server().await;

    // Get initial dashboard data
    let (status1, json1) = send_request(&app, "GET", "/api/v1/learning/dashboard").await;
    assert_eq!(status1, StatusCode::OK);

    // Small delay to simulate time passing
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Get updated dashboard data
    let (status2, json2) = send_request(&app, "GET", "/api/v1/learning/dashboard").await;
    assert_eq!(status2, StatusCode::OK);

    // Both requests should succeed (testing real-time capability)
    assert!(json1.get("current_metrics").is_some());
    assert!(json2.get("current_metrics").is_some());
}

#[tokio::test]
async fn test_learning_endpoints_error_handling() {
    let app = create_test_server().await;

    // Test invalid endpoint path
    let (status, _json) = send_request(&app, "GET", "/api/v1/learning/invalid").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Test invalid method
    let (status, _) = send_request(&app, "POST", "/api/v1/learning/metrics").await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn test_learning_dashboard_concurrent_requests() {
    let app = create_test_server().await;

    // Send multiple concurrent requests
    let requests = (0..3)
        .map(|_| {
            let app_clone = app.clone();
            tokio::spawn(async move {
                send_request(&app_clone, "GET", "/api/v1/learning/dashboard").await
            })
        })
        .collect::<Vec<_>>();

    let results = futures::future::join_all(requests).await;

    // All requests should succeed
    for result in results {
        let (status, json) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert!(json.get("current_metrics").is_some());
    }
}
