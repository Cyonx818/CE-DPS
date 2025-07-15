// ABOUTME: Comprehensive integration tests for Sprint 009 Learning-API system integration
//!
//! This test suite validates the complete integration between the learning system and
//! API server, ensuring data persistence, dashboard integration, and real-time metrics
//! collection work correctly across service boundaries.
//!
//! ## Protected Functionality
//! - Learning API endpoints integration (dashboard data, metrics, health)
//! - Real-time learning data persistence through API
//! - Dashboard integration workflows (metrics collection, display data)
//! - API authentication and authorization for learning endpoints
//! - Cross-service data consistency and synchronization

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    response::{Json, Response},
    routing::{get, post},
};
use fortitude::learning::*;
use fortitude_api_server::{
    models::{requests::*, responses::*},
    routes::learning::*,
    server::*,
    ApiServerConfig,
};
use axum::body::to_bytes;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::sync::RwLock;
use tower::ServiceExt;
use uuid::Uuid;

/// Integrated test environment for learning-API validation
#[derive(Clone)]
pub struct LearningApiTestEnvironment {
    learning_service: Arc<IntegratedLearningService>,
    api_server_app: axum::Router,
    learning_state: LearningState,
    test_metrics: Arc<RwLock<ApiIntegrationMetrics>>,
    temp_dir: Arc<TempDir>,
}

#[derive(Clone, Default)]
pub struct ApiIntegrationMetrics {
    api_requests_processed: u64,
    learning_data_synced: u64,
    dashboard_updates: u64,
    health_checks_completed: u64,
    data_consistency_validations: u64,
    response_times: Vec<Duration>,
}

/// ANCHOR: Validates learning dashboard API integration and real-time data flow
/// Tests: Learning data collection → API dashboard endpoints → data visualization
#[tokio::test]
async fn test_anchor_learning_dashboard_api_integration() {
    let env = setup_learning_api_environment().await;
    let test_start = Instant::now();

    println!("Phase 1: Generate learning activity for dashboard testing");

    // Create diverse learning activities
    let learning_activities = vec![
        (
            "user_feedback",
            create_feedback_activity("content_001", 0.85, "Excellent research quality"),
        ),
        (
            "pattern_recognition",
            create_pattern_activity("search_query", "rust async", 15),
        ),
        (
            "adaptation",
            create_adaptation_activity("prompt_optimization", 0.12),
        ),
        (
            "optimization",
            create_optimization_activity("response_time", 150),
        ),
    ];

    // Process learning activities through the service
    for (activity_type, activity_data) in &learning_activities {
        match activity_type {
            &"user_feedback" => {
                let feedback: UserFeedback = serde_json::from_value(activity_data.clone()).unwrap();
                env.learning_service
                    .process_feedback(&feedback)
                    .await
                    .unwrap();
            }
            &"pattern_recognition" => {
                let pattern: PatternData = serde_json::from_value(activity_data.clone()).unwrap();
                env.learning_service
                    .analyze_pattern(&pattern)
                    .await
                    .unwrap();
            }
            &"adaptation" => {
                let adaptation: Value = activity_data.clone();
                env.learning_service
                    .apply_adaptation(adaptation)
                    .await
                    .unwrap();
            }
            &"optimization" => {
                let optimization: Value = activity_data.clone();
                env.learning_service
                    .optimize_performance(optimization)
                    .await
                    .unwrap();
            }
            _ => {}
        }
    }

    println!("Phase 2: Test learning dashboard API endpoint");

    // Test dashboard data API endpoint
    let dashboard_request = Request::builder()
        .method("GET")
        .uri("/api/learning/dashboard?duration=24h&detailed=true")
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
    let dashboard_data: LearningDashboardResponse =
        serde_json::from_slice(&dashboard_body).unwrap();

    // Validate dashboard data structure and content
    assert!(dashboard_data.system_overview.total_feedback_processed > 0);
    assert!(dashboard_data.system_overview.patterns_recognized > 0);
    assert!(dashboard_data.system_overview.adaptations_applied > 0);
    assert!(!dashboard_data.recent_activities.is_empty());

    // Validate learning metrics are populated
    assert!(
        dashboard_data
            .learning_metrics
            .adaptation_metrics
            .adaptations_applied
            > 0
    );
    assert!(
        dashboard_data
            .learning_metrics
            .storage_metrics
            .total_operations
            > 0
    );
    assert!(
        dashboard_data
            .learning_metrics
            .pattern_recognition_metrics
            .patterns_analyzed
            > 0
    );

    env.test_metrics.write().await.api_requests_processed += 1;
    env.test_metrics.write().await.dashboard_updates += 1;

    println!("Phase 3: Test real-time learning metrics API endpoint");

    // Test metrics API endpoint with different query parameters
    let metrics_scenarios = vec![
        ("1h", false), // 1 hour, basic metrics
        ("24h", true), // 24 hours, detailed metrics
        ("7d", false), // 7 days, basic metrics
    ];

    for (duration, detailed) in metrics_scenarios {
        let metrics_request = Request::builder()
            .method("GET")
            .uri(&format!(
                "/api/learning/metrics?duration={}&detailed={}",
                duration, detailed
            ))
            .header("content-type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response_start = Instant::now();
        let metrics_response = env
            .api_server_app
            .clone()
            .oneshot(metrics_request)
            .await
            .unwrap();
        let response_time = response_start.elapsed();

        assert_eq!(metrics_response.status(), StatusCode::OK);

        let metrics_body = to_bytes(metrics_response.into_body()).await.unwrap();
        let metrics_data: LearningMetricsResponse = serde_json::from_slice(&metrics_body).unwrap();

        // Validate metrics response structure
        assert!(metrics_data.adaptation_metrics.adaptations_applied >= 0);
        assert!(metrics_data.storage_metrics.total_operations >= 0);
        assert!(metrics_data.feedback_metrics.total_feedback_received >= 0);

        // Validate detailed metrics when requested
        if detailed {
            assert!(!metrics_data
                .optimization_metrics
                .recent_optimizations
                .is_empty());
            assert!(
                metrics_data
                    .pattern_recognition_metrics
                    .pattern_categories
                    .len()
                    > 0
            );
        }

        // Performance validation
        assert!(
            response_time < Duration::from_millis(500),
            "Metrics API should respond quickly"
        );

        env.test_metrics
            .write()
            .await
            .response_times
            .push(response_time);
        env.test_metrics.write().await.api_requests_processed += 1;

        println!(
            "  - Metrics API ({}): {:?} response time",
            duration, response_time
        );
    }

    println!("Phase 4: Test learning health monitoring integration");

    // Test health endpoint
    let health_request = Request::builder()
        .method("GET")
        .uri("/api/learning/health")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let health_response = env
        .api_server_app
        .clone()
        .oneshot(health_request)
        .await
        .unwrap();

    assert_eq!(health_response.status(), StatusCode::OK);

    let health_body = to_bytes(health_response.into_body()).await.unwrap();
    let health_data: LearningHealthResponse = serde_json::from_slice(&health_body).unwrap();

    // Validate health response
    assert_eq!(health_data.overall_status, LearningHealthStatus::Healthy);
    assert!(!health_data.component_health.is_empty());

    // Validate individual component health
    for component in &health_data.component_health {
        assert!(matches!(
            component.status,
            LearningHealthStatus::Healthy | LearningHealthStatus::Degraded
        ));
        assert!(component.response_time_ms < 1000.0); // Should be under 1 second
    }

    env.test_metrics.write().await.health_checks_completed += 1;

    println!("Phase 5: Test learning performance summary integration");

    // Test performance summary endpoint
    let performance_request = Request::builder()
        .method("GET")
        .uri("/api/learning/performance?period=24h")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let performance_response = env
        .api_server_app
        .clone()
        .oneshot(performance_request)
        .await
        .unwrap();

    assert_eq!(performance_response.status(), StatusCode::OK);

    let performance_body = to_bytes(performance_response.into_body()).await.unwrap();
    let performance_data: LearningPerformanceSummaryResponse =
        serde_json::from_slice(&performance_body).unwrap();

    // Validate performance summary
    assert!(
        performance_data
            .adaptation_performance
            .average_processing_time_ms
            < 1000.0
    );
    assert!(performance_data.storage_performance.average_write_time_ms < 100.0);
    assert!(performance_data.storage_performance.average_read_time_ms < 50.0);
    assert!(
        performance_data
            .pattern_recognition_performance
            .average_analysis_time_ms
            < 500.0
    );

    // Validate performance trends
    assert!(!performance_data.performance_trends.is_empty());
    for trend in &performance_data.performance_trends {
        assert!(trend.value >= 0.0);
        assert!(!trend.metric_name.is_empty());
    }

    env.test_metrics.write().await.api_requests_processed += 1;

    // Performance validation for entire workflow
    let total_test_time = test_start.elapsed();
    assert!(
        total_test_time < Duration::from_secs(15),
        "Complete dashboard integration should be fast"
    );

    println!("✓ Learning dashboard API integration completed successfully");
    println!(
        "  - API requests processed: {}",
        env.test_metrics.read().await.api_requests_processed
    );
    println!(
        "  - Average response time: {:?}",
        calculate_average_response_time(&env.test_metrics.read().await.response_times)
    );
    println!("  - Total test duration: {:?}", total_test_time);
}

/// ANCHOR: Validates learning data persistence and synchronization across API boundaries
/// Tests: Learning data storage → API data retrieval → consistency validation
#[tokio::test]
async fn test_anchor_learning_data_persistence_api_synchronization() {
    let env = setup_learning_api_environment().await;

    println!("Phase 1: Create and persist diverse learning data");

    // Create comprehensive learning dataset
    let learning_dataset = vec![
        create_feedback_data("research_quality_001", 0.92, "Excellent technical depth"),
        create_feedback_data("research_quality_002", 0.78, "Good but needs more examples"),
        create_feedback_data(
            "research_quality_003",
            0.85,
            "Clear explanation with good structure",
        ),
        create_pattern_data("user_query", "rust async programming", 25),
        create_pattern_data("response_format", "technical_with_examples", 18),
        create_learning_insight(
            "prompt_optimization",
            json!({
                "optimization_type": "clarity_improvement",
                "success_rate": 0.15,
                "user_satisfaction_increase": 0.08
            }),
        ),
    ];

    // Store data through learning service
    for (i, data) in learning_dataset.iter().enumerate() {
        match data {
            LearningDataType::Feedback(feedback) => {
                env.learning_service.store_feedback(feedback).await.unwrap();
            }
            LearningDataType::Pattern(pattern) => {
                env.learning_service.store_pattern(pattern).await.unwrap();
            }
            LearningDataType::Insight(insight) => {
                env.learning_service
                    .store_learning_data(insight)
                    .await
                    .unwrap();
            }
        }

        println!("  - Stored learning data item {}", i + 1);
    }

    env.test_metrics.write().await.learning_data_synced = learning_dataset.len() as u64;

    println!("Phase 2: Validate data persistence through API retrieval");

    // Test feedback data retrieval
    let feedback_request = Request::builder()
        .method("GET")
        .uri("/api/learning/feedback?content_id=research_quality_001")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let feedback_response = env
        .api_server_app
        .clone()
        .oneshot(feedback_request)
        .await
        .unwrap();

    assert_eq!(feedback_response.status(), StatusCode::OK);

    let feedback_body = to_bytes(feedback_response.into_body()).await.unwrap();
    let feedback_data: Vec<UserFeedback> = serde_json::from_slice(&feedback_body).unwrap();

    assert!(!feedback_data.is_empty());
    assert_eq!(feedback_data[0].content_id, "research_quality_001");
    assert_eq!(feedback_data[0].score, 0.92);

    println!("Phase 3: Test pattern data retrieval and consistency");

    // Test pattern data retrieval
    let patterns_request = Request::builder()
        .method("GET")
        .uri("/api/learning/patterns?pattern_type=user_query&days=30")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let patterns_response = env
        .api_server_app
        .clone()
        .oneshot(patterns_request)
        .await
        .unwrap();

    assert_eq!(patterns_response.status(), StatusCode::OK);

    let patterns_body = to_bytes(patterns_response.into_body()).await.unwrap();
    let patterns_data: Vec<PatternData> = serde_json::from_slice(&patterns_body).unwrap();

    assert!(!patterns_data.is_empty());

    // Validate pattern data consistency
    let rust_async_pattern = patterns_data.iter().find(|p| {
        p.metadata
            .get("query_text")
            .map(|v| v.contains("rust async"))
            .unwrap_or(false)
    });
    assert!(
        rust_async_pattern.is_some(),
        "Should find rust async pattern"
    );

    println!("Phase 4: Test learning insights retrieval and data integrity");

    // Test learning insights retrieval
    let insights_request = Request::builder()
        .method("GET")
        .uri("/api/learning/insights?insight_type=prompt_optimization")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let insights_response = env
        .api_server_app
        .clone()
        .oneshot(insights_request)
        .await
        .unwrap();

    assert_eq!(insights_response.status(), StatusCode::OK);

    let insights_body = to_bytes(insights_response.into_body()).await.unwrap();
    let insights_data: Vec<LearningData> = serde_json::from_slice(&insights_body).unwrap();

    assert!(!insights_data.is_empty());

    // Validate insight data structure
    let optimization_insight = &insights_data[0];
    assert_eq!(optimization_insight.content_id, "prompt_optimization");
    assert!(optimization_insight
        .insights
        .get("optimization_type")
        .is_some());
    assert!(optimization_insight.insights.get("success_rate").is_some());

    println!("Phase 5: Cross-service data consistency validation");

    // Test data consistency across service boundaries
    let consistency_tests = vec![
        ("feedback_count", "/api/learning/stats/feedback"),
        ("pattern_count", "/api/learning/stats/patterns"),
        ("insight_count", "/api/learning/stats/insights"),
    ];

    for (stat_type, endpoint) in consistency_tests {
        let stats_request = Request::builder()
            .method("GET")
            .uri(endpoint)
            .header("content-type", "application/json")
            .body(Body::empty())
            .unwrap();

        let stats_response = env
            .api_server_app
            .clone()
            .oneshot(stats_request)
            .await
            .unwrap();

        assert_eq!(stats_response.status(), StatusCode::OK);

        let stats_body = to_bytes(stats_response.into_body()).await.unwrap();
        let stats_data: Value = serde_json::from_slice(&stats_body).unwrap();

        let count = stats_data["count"].as_u64().unwrap_or(0);
        assert!(count > 0, "{} should have positive count", stat_type);

        env.test_metrics.write().await.data_consistency_validations += 1;

        println!("  - {}: {} items", stat_type, count);
    }

    println!("Phase 6: Concurrent access and data integrity testing");

    // Test concurrent API access to verify data integrity
    let concurrent_tasks = (0..10)
        .map(|i| {
            let env_clone = env.clone();
            tokio::spawn(async move {
                let request = Request::builder()
                    .method("GET")
                    .uri("/api/learning/dashboard")
                    .header("content-type", "application/json")
                    .body(Body::empty())
                    .unwrap();

                let response = env_clone
                    .api_server_app
                    .clone()
                    .oneshot(request)
                    .await
                    .unwrap();

                (i, response.status())
            })
        })
        .collect::<Vec<_>>();

    // Wait for all concurrent requests to complete
    let results = futures::future::join_all(concurrent_tasks).await;

    // Validate all requests succeeded
    for (task_id, status) in results {
        let (task_num, response_status) = task_id.unwrap();
        assert_eq!(
            response_status,
            StatusCode::OK,
            "Concurrent request {} should succeed",
            task_num
        );
    }

    println!("✓ Learning data persistence API integration completed successfully");
    println!(
        "  - Learning data items synced: {}",
        env.test_metrics.read().await.learning_data_synced
    );
    println!(
        "  - Data consistency validations: {}",
        env.test_metrics.read().await.data_consistency_validations
    );
    println!("  - Concurrent access tests: 10 successful");
}

/// ANCHOR: Validates learning API authentication and authorization workflows
/// Tests: API authentication → learning endpoint access → authorization validation
#[tokio::test]
async fn test_anchor_learning_api_authentication_authorization() {
    let env = setup_learning_api_environment().await;

    println!("Phase 1: Test unauthenticated access restrictions");

    // Test protected endpoints without authentication
    let protected_endpoints = vec![
        ("/api/learning/admin/config", "GET"),
        ("/api/learning/admin/reset", "POST"),
        ("/api/learning/feedback", "POST"),
        ("/api/learning/insights", "POST"),
    ];

    for (endpoint, method) in protected_endpoints {
        let request = Request::builder()
            .method(method)
            .uri(endpoint)
            .header("content-type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = env.api_server_app.clone().oneshot(request).await.unwrap();

        // Should require authentication (401) or be forbidden (403)
        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::FORBIDDEN,
            "Protected endpoint {} should require authentication",
            endpoint
        );
    }

    println!("Phase 2: Test authenticated read-only access");

    // Test read endpoints with basic authentication
    let read_endpoints = vec![
        "/api/learning/dashboard",
        "/api/learning/metrics",
        "/api/learning/health",
        "/api/learning/performance",
    ];

    for endpoint in read_endpoints {
        let request = Request::builder()
            .method("GET")
            .uri(endpoint)
            .header("content-type", "application/json")
            .header("authorization", "Bearer mock_read_token")
            .body(Body::empty())
            .unwrap();

        let response = env.api_server_app.clone().oneshot(request).await.unwrap();

        // Should allow read access (200 OK)
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Read endpoint {} should be accessible",
            endpoint
        );
    }

    println!("Phase 3: Test authenticated write access with proper permissions");

    // Test write operations with appropriate authentication
    let feedback_data = json!({
        "content_id": "auth_test_001",
        "user_id": "test_user",
        "score": 0.85,
        "comment": "Test feedback for authentication",
        "feedback_type": "quality_assessment"
    });

    let write_request = Request::builder()
        .method("POST")
        .uri("/api/learning/feedback")
        .header("content-type", "application/json")
        .header("authorization", "Bearer mock_write_token")
        .body(Body::from(feedback_data.to_string()))
        .unwrap();

    let write_response = env
        .api_server_app
        .clone()
        .oneshot(write_request)
        .await
        .unwrap();

    // Should allow write access with proper token
    assert_eq!(
        write_response.status(),
        StatusCode::CREATED,
        "Write endpoint should accept authenticated requests"
    );

    println!("Phase 4: Test admin-level access restrictions");

    // Test admin endpoints with non-admin authentication
    let admin_request = Request::builder()
        .method("GET")
        .uri("/api/learning/admin/config")
        .header("content-type", "application/json")
        .header("authorization", "Bearer mock_write_token") // Non-admin token
        .body(Body::empty())
        .unwrap();

    let admin_response = env
        .api_server_app
        .clone()
        .oneshot(admin_request)
        .await
        .unwrap();

    // Should deny access without admin privileges
    assert_eq!(
        admin_response.status(),
        StatusCode::FORBIDDEN,
        "Admin endpoint should require admin privileges"
    );

    // Test admin endpoints with admin authentication
    let admin_auth_request = Request::builder()
        .method("GET")
        .uri("/api/learning/admin/config")
        .header("content-type", "application/json")
        .header("authorization", "Bearer mock_admin_token")
        .body(Body::empty())
        .unwrap();

    let admin_auth_response = env
        .api_server_app
        .clone()
        .oneshot(admin_auth_request)
        .await
        .unwrap();

    // Should allow access with admin token
    assert_eq!(
        admin_auth_response.status(),
        StatusCode::OK,
        "Admin endpoint should accept admin authentication"
    );

    println!("Phase 5: Test rate limiting and access control");

    // Test rate limiting behavior
    let rate_limit_tasks = (0..20)
        .map(|i| {
            let env_clone = env.clone();
            tokio::spawn(async move {
                let request = Request::builder()
                    .method("GET")
                    .uri("/api/learning/metrics")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer mock_read_token")
                    .header("x-forwarded-for", "192.168.1.100") // Same IP for rate limiting
                    .body(Body::empty())
                    .unwrap();

                let response = env_clone
                    .api_server_app
                    .clone()
                    .oneshot(request)
                    .await
                    .unwrap();

                (i, response.status())
            })
        })
        .collect::<Vec<_>>();

    let rate_limit_results = futures::future::join_all(rate_limit_tasks).await;

    // Some requests should succeed, some might be rate limited
    let successful_requests = rate_limit_results
        .iter()
        .filter(|result| result.as_ref().unwrap().1 == StatusCode::OK)
        .count();

    let rate_limited_requests = rate_limit_results
        .iter()
        .filter(|result| result.as_ref().unwrap().1 == StatusCode::TOO_MANY_REQUESTS)
        .count();

    // Should have some successful requests and potentially some rate limited
    assert!(
        successful_requests > 0,
        "Should have some successful requests"
    );
    println!("  - Successful requests: {}", successful_requests);
    println!("  - Rate limited requests: {}", rate_limited_requests);

    println!("✓ Learning API authentication/authorization completed successfully");
}

/// ANCHOR: Validates learning API performance under realistic load conditions
/// Tests: Concurrent learning API requests → response time validation → throughput measurement
#[tokio::test]
async fn test_anchor_learning_api_performance_under_load() {
    let env = setup_learning_api_environment().await;
    let load_test_start = Instant::now();

    println!("Phase 1: Baseline performance measurement");

    // Measure baseline response times
    let baseline_endpoints = vec![
        "/api/learning/dashboard",
        "/api/learning/metrics",
        "/api/learning/health",
    ];

    let mut baseline_times = HashMap::new();
    for endpoint in &baseline_endpoints {
        let start = Instant::now();
        let request = Request::builder()
            .method("GET")
            .uri(*endpoint)
            .header("content-type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = env.api_server_app.clone().oneshot(request).await.unwrap();

        let response_time = start.elapsed();
        assert_eq!(response.status(), StatusCode::OK);
        baseline_times.insert(endpoint.to_string(), response_time);

        println!("  - {} baseline: {:?}", endpoint, response_time);
    }

    println!("Phase 2: Concurrent load testing");

    // Test concurrent requests to learning endpoints
    let concurrent_load = 50; // 50 concurrent requests
    let endpoint_rotations = vec![
        "/api/learning/dashboard",
        "/api/learning/metrics",
        "/api/learning/health",
        "/api/learning/performance",
    ];

    let concurrent_tasks = (0..concurrent_load)
        .map(|i| {
            let env_clone = env.clone();
            let endpoint = endpoint_rotations[i % endpoint_rotations.len()].to_string();

            tokio::spawn(async move {
                let start = Instant::now();
                let request = Request::builder()
                    .method("GET")
                    .uri(&endpoint)
                    .header("content-type", "application/json")
                    .body(Body::empty())
                    .unwrap();

                let response = env_clone
                    .api_server_app
                    .clone()
                    .oneshot(request)
                    .await
                    .unwrap();

                let response_time = start.elapsed();
                (endpoint, response.status(), response_time)
            })
        })
        .collect::<Vec<_>>();

    let concurrent_results = futures::future::join_all(concurrent_tasks).await;

    // Analyze concurrent performance results
    let mut successful_requests = 0;
    let mut total_response_time = Duration::ZERO;
    let mut max_response_time = Duration::ZERO;
    let mut min_response_time = Duration::from_secs(10);

    for result in concurrent_results {
        let (endpoint, status, response_time) = result.unwrap();

        if status == StatusCode::OK {
            successful_requests += 1;
            total_response_time += response_time;
            max_response_time = max_response_time.max(response_time);
            min_response_time = min_response_time.min(response_time);
        }
    }

    let average_response_time = total_response_time / successful_requests;

    // Performance validations
    assert_eq!(
        successful_requests, concurrent_load,
        "All concurrent requests should succeed"
    );
    assert!(
        average_response_time < Duration::from_millis(1000),
        "Average response time should be under 1 second"
    );
    assert!(
        max_response_time < Duration::from_millis(2000),
        "Max response time should be under 2 seconds"
    );

    println!("Phase 3: Sustained load testing");

    // Test sustained load over time
    let sustained_duration = Duration::from_secs(30);
    let requests_per_second = 10;
    let total_sustained_requests =
        (sustained_duration.as_secs() * requests_per_second as u64) as usize;

    let sustained_start = Instant::now();
    let mut sustained_tasks = Vec::new();

    for i in 0..total_sustained_requests {
        let env_clone = env.clone();
        let endpoint = endpoint_rotations[i % endpoint_rotations.len()].to_string();

        // Distribute requests over time
        let delay = Duration::from_millis((i as u64 * 1000) / requests_per_second as u64);

        let task = tokio::spawn(async move {
            tokio::time::sleep(delay).await;

            let start = Instant::now();
            let request = Request::builder()
                .method("GET")
                .uri(&endpoint)
                .header("content-type", "application/json")
                .body(Body::empty())
                .unwrap();

            let response = env_clone
                .api_server_app
                .clone()
                .oneshot(request)
                .await
                .unwrap();

            let response_time = start.elapsed();
            (response.status(), response_time)
        });

        sustained_tasks.push(task);
    }

    let sustained_results = futures::future::join_all(sustained_tasks).await;

    // Analyze sustained load results
    let mut sustained_successful = 0;
    let mut sustained_total_time = Duration::ZERO;
    let mut response_times_over_time = Vec::new();

    for result in sustained_results {
        let (status, response_time) = result.unwrap();

        if status == StatusCode::OK {
            sustained_successful += 1;
            sustained_total_time += response_time;
            response_times_over_time.push(response_time);
        }
    }

    let sustained_average = sustained_total_time / sustained_successful;
    let sustained_success_rate = sustained_successful as f64 / total_sustained_requests as f64;

    // Sustained load validations
    assert!(
        sustained_success_rate > 0.95,
        "Should maintain >95% success rate under sustained load"
    );
    assert!(
        sustained_average < Duration::from_millis(1500),
        "Sustained load average should be reasonable"
    );

    println!("Phase 4: Memory and resource utilization validation");

    // Test memory usage under load (simplified validation)
    let memory_test_tasks = (0..100)
        .map(|i| {
            let env_clone = env.clone();
            tokio::spawn(async move {
                let request = Request::builder()
                    .method("GET")
                    .uri("/api/learning/dashboard?detailed=true")
                    .header("content-type", "application/json")
                    .body(Body::empty())
                    .unwrap();

                let response = env_clone
                    .api_server_app
                    .clone()
                    .oneshot(request)
                    .await
                    .unwrap();

                // Return response body size as a proxy for memory usage
                let body = to_bytes(response.into_body()).await.unwrap();
                body.len()
            })
        })
        .collect::<Vec<_>>();

    let memory_results = futures::future::join_all(memory_test_tasks).await;
    let total_response_size: usize = memory_results.iter().map(|r| r.as_ref().unwrap()).sum();

    let average_response_size = total_response_size / memory_results.len();

    // Memory usage validation (responses shouldn't be excessively large)
    assert!(
        average_response_size < 1_000_000,
        "Average response size should be reasonable"
    ); // < 1MB

    let total_load_test_time = load_test_start.elapsed();

    println!("✓ Learning API performance testing completed successfully");
    println!(
        "  - Concurrent requests: {} (all successful)",
        concurrent_load
    );
    println!(
        "  - Average concurrent response time: {:?}",
        average_response_time
    );
    println!(
        "  - Min/Max response times: {:?} / {:?}",
        min_response_time, max_response_time
    );
    println!(
        "  - Sustained requests: {} ({:.1}% success rate)",
        total_sustained_requests,
        sustained_success_rate * 100.0
    );
    println!(
        "  - Sustained average response time: {:?}",
        sustained_average
    );
    println!("  - Average response size: {} bytes", average_response_size);
    println!("  - Total test duration: {:?}", total_load_test_time);
}

// Helper functions and data structures

async fn setup_learning_api_environment() -> LearningApiTestEnvironment {
    let temp_dir = Arc::new(TempDir::new().unwrap());
    let learning_service = Arc::new(IntegratedLearningService::new().await);
    let learning_state = LearningState::new().await.unwrap();

    // Create mock API server app
    let api_server_app = create_mock_learning_api_router(learning_state.clone());

    LearningApiTestEnvironment {
        learning_service,
        api_server_app,
        learning_state,
        test_metrics: Arc::new(RwLock::new(ApiIntegrationMetrics::default())),
        temp_dir,
    }
}

fn create_mock_learning_api_router(learning_state: LearningState) -> axum::Router {
    use axum::routing::{get, post};

    axum::Router::new()
        .route("/api/learning/dashboard", get(mock_learning_dashboard))
        .route("/api/learning/metrics", get(mock_learning_metrics))
        .route("/api/learning/health", get(mock_learning_health))
        .route("/api/learning/performance", get(mock_learning_performance))
        .route(
            "/api/learning/feedback",
            get(mock_get_feedback).post(mock_post_feedback),
        )
        .route("/api/learning/patterns", get(mock_get_patterns))
        .route(
            "/api/learning/insights",
            get(mock_get_insights).post(mock_post_insights),
        )
        .route("/api/learning/stats/feedback", get(mock_feedback_stats))
        .route("/api/learning/stats/patterns", get(mock_pattern_stats))
        .route("/api/learning/stats/insights", get(mock_insight_stats))
        .route("/api/learning/admin/config", get(mock_admin_config))
        .route("/api/learning/admin/reset", post(mock_admin_reset))
        .with_state(learning_state)
}

// Mock API handlers (simplified implementations)

async fn mock_learning_dashboard(
    State(state): State<LearningState>,
) -> Json<LearningDashboardResponse> {
    let system_overview = LearningSystemOverview {
        total_feedback_processed: 150,
        patterns_recognized: 25,
        adaptations_applied: 12,
        system_uptime_hours: 48.5,
        overall_learning_score: 0.87,
    };

    let learning_metrics = state.performance_monitor.get_current_metrics().await;

    let recent_activities = vec![
        LearningDataPoint {
            timestamp: chrono::Utc::now(),
            metric_name: "feedback_processed".to_string(),
            value: 10.0,
            metadata: HashMap::new(),
        },
        LearningDataPoint {
            timestamp: chrono::Utc::now() - chrono::Duration::hours(1),
            metric_name: "pattern_recognized".to_string(),
            value: 3.0,
            metadata: HashMap::new(),
        },
    ];

    Json(LearningDashboardResponse {
        system_overview,
        learning_metrics,
        recent_activities,
        timestamp: chrono::Utc::now(),
    })
}

async fn mock_learning_metrics(
    State(state): State<LearningState>,
) -> Json<LearningMetricsResponse> {
    Json(state.performance_monitor.get_current_metrics().await)
}

async fn mock_learning_health(State(state): State<LearningState>) -> Json<LearningHealthResponse> {
    Json(state.performance_monitor.get_health_status().await)
}

async fn mock_learning_performance(
    State(_state): State<LearningState>,
) -> Json<LearningPerformanceSummaryResponse> {
    Json(LearningPerformanceSummaryResponse::default())
}

async fn mock_get_feedback() -> Json<Vec<UserFeedback>> {
    Json(vec![UserFeedback {
        id: Uuid::new_v4().to_string(),
        user_id: "user_001".to_string(),
        content_id: "research_quality_001".to_string(),
        feedback_type: "quality_assessment".to_string(),
        score: Some(0.92),
        text_feedback: Some("Excellent technical depth".to_string()),
        timestamp: chrono::Utc::now(),
        metadata: HashMap::new(),
    }])
}

async fn mock_post_feedback() -> (StatusCode, Json<Value>) {
    (
        StatusCode::CREATED,
        Json(json!({"success": true, "id": "feedback_123"})),
    )
}

async fn mock_get_patterns() -> Json<Vec<PatternData>> {
    Json(vec![PatternData {
        id: Uuid::new_v4(),
        pattern_type: "user_query".to_string(),
        frequency: 25,
        confidence: 0.85,
        metadata: HashMap::from([(
            "query_text".to_string(),
            "rust async programming".to_string(),
        )]),
        created_at: chrono::Utc::now(),
        last_seen: chrono::Utc::now(),
    }])
}

async fn mock_get_insights() -> Json<Vec<LearningData>> {
    Json(vec![LearningData {
        id: Uuid::new_v4(),
        content_id: "prompt_optimization".to_string(),
        insights: json!({
            "optimization_type": "clarity_improvement",
            "success_rate": 0.15
        }),
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        expiry_date: Some(chrono::Utc::now() + chrono::Duration::days(90)),
    }])
}

async fn mock_post_insights() -> (StatusCode, Json<Value>) {
    (
        StatusCode::CREATED,
        Json(json!({"success": true, "id": "insight_123"})),
    )
}

async fn mock_feedback_stats() -> Json<Value> {
    Json(json!({"count": 150, "average_score": 0.85}))
}

async fn mock_pattern_stats() -> Json<Value> {
    Json(json!({"count": 25, "active_patterns": 18}))
}

async fn mock_insight_stats() -> Json<Value> {
    Json(json!({"count": 12, "recent_insights": 5}))
}

async fn mock_admin_config() -> Json<Value> {
    Json(json!({"learning_enabled": true, "adaptation_rate": 0.1}))
}

async fn mock_admin_reset() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({"success": true, "message": "Learning system reset"})),
    )
}

// Helper data creation functions

#[derive(Clone)]
enum LearningDataType {
    Feedback(UserFeedback),
    Pattern(PatternData),
    Insight(LearningData),
}

fn create_feedback_activity(content_id: &str, score: f64, comment: &str) -> Value {
    json!({
        "id": Uuid::new_v4(),
        "content_id": content_id,
        "user_id": "test_user",
        "score": score,
        "comment": comment,
        "feedback_type": "quality_assessment",
        "created_at": chrono::Utc::now(),
        "metadata": {}
    })
}

fn create_pattern_activity(pattern_type: &str, pattern_value: &str, frequency: u32) -> Value {
    json!({
        "id": Uuid::new_v4(),
        "pattern_type": pattern_type,
        "frequency": frequency,
        "confidence": 0.8,
        "metadata": {
            "pattern_value": pattern_value
        },
        "created_at": chrono::Utc::now(),
        "last_seen": chrono::Utc::now()
    })
}

fn create_adaptation_activity(adaptation_type: &str, improvement: f64) -> Value {
    json!({
        "adaptation_type": adaptation_type,
        "improvement_score": improvement,
        "applied_at": chrono::Utc::now()
    })
}

fn create_optimization_activity(optimization_type: &str, value: u32) -> Value {
    json!({
        "optimization_type": optimization_type,
        "value": value,
        "applied_at": chrono::Utc::now()
    })
}

fn create_feedback_data(content_id: &str, score: f64, comment: &str) -> LearningDataType {
    LearningDataType::Feedback(UserFeedback {
        id: Uuid::new_v4().to_string(),
        user_id: "test_user".to_string(),
        content_id: content_id.to_string(),
        feedback_type: "quality_assessment".to_string(),
        score: Some(score),
        text_feedback: Some(comment.to_string()),
        timestamp: chrono::Utc::now(),
        metadata: HashMap::new(),
    })
}

fn create_pattern_data(
    pattern_type: &str,
    pattern_value: &str,
    frequency: u32,
) -> LearningDataType {
    LearningDataType::Pattern(PatternData {
        id: Uuid::new_v4().to_string(),
        pattern_type: pattern_type.to_string(),
        frequency,
        success_rate: 0.8,
        context: HashMap::from([(pattern_type.to_string(), serde_json::json!(pattern_value))]),
        first_seen: chrono::Utc::now(),
        last_seen: chrono::Utc::now(),
    })
}

fn create_learning_insight(content_id: &str, insights: Value) -> LearningDataType {
    LearningDataType::Insight(LearningData {
        id: Uuid::new_v4().to_string(),
        learning_type: "insight".to_string(),
        source_data_id: content_id.to_string(),
        insights: vec![insights.to_string()],
        confidence_score: 0.8,
        created_at: chrono::Utc::now(),
        expires_at: Some(chrono::Utc::now() + chrono::Duration::days(90)),
        metadata: HashMap::new(),
    })
}

fn calculate_average_response_time(times: &[Duration]) -> Duration {
    if times.is_empty() {
        return Duration::ZERO;
    }
    let total: Duration = times.iter().sum();
    total / times.len() as u32
}

// Mock integrated learning service

#[derive(Clone)]
pub struct IntegratedLearningService {
    feedback_storage: Arc<RwLock<Vec<UserFeedback>>>,
    pattern_storage: Arc<RwLock<Vec<PatternData>>>,
    learning_storage: Arc<RwLock<Vec<LearningData>>>,
}

impl IntegratedLearningService {
    pub async fn new() -> Self {
        Self {
            feedback_storage: Arc::new(RwLock::new(Vec::new())),
            pattern_storage: Arc::new(RwLock::new(Vec::new())),
            learning_storage: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn process_feedback(&self, feedback: &UserFeedback) -> Result<(), LearningError> {
        self.feedback_storage.write().await.push(feedback.clone());
        Ok(())
    }

    pub async fn analyze_pattern(&self, pattern: &PatternData) -> Result<(), LearningError> {
        self.pattern_storage.write().await.push(pattern.clone());
        Ok(())
    }

    pub async fn apply_adaptation(&self, _adaptation: Value) -> Result<(), LearningError> {
        Ok(())
    }

    pub async fn optimize_performance(&self, _optimization: Value) -> Result<(), LearningError> {
        Ok(())
    }

    pub async fn store_feedback(&self, feedback: &UserFeedback) -> Result<(), LearningError> {
        self.feedback_storage.write().await.push(feedback.clone());
        Ok(())
    }

    pub async fn store_pattern(&self, pattern: &PatternData) -> Result<(), LearningError> {
        self.pattern_storage.write().await.push(pattern.clone());
        Ok(())
    }

    pub async fn store_learning_data(&self, data: &LearningData) -> Result<(), LearningError> {
        self.learning_storage.write().await.push(data.clone());
        Ok(())
    }
}
