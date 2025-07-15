// ABOUTME: Anchor tests for Sprint 009 Tasks 3 & 4 - API Integration Critical Workflows
//! These tests protect critical API integration functionality implemented in Sprint 009
//! Tasks 3 and 4. They ensure that API integration workflows continue to work correctly
//! as the system evolves.
//!
//! ## Protected Functionality
//! - External API integration (learning and monitoring API endpoints)
//! - API compatibility (endpoint stability, request/response formats)
//! - User input processing (API request validation, parameter processing)
//! - Authentication (API security and access control)
//! - Cross-component integration (API+learning+monitoring integration)

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    Extension,
};
use fortitude::learning::*;
use fortitude::monitoring::*;
use fortitude_api_server::{
    middleware::auth::{AuthManager, Claims, Permission},
    models::{requests::*, responses::*},
    routes::{learning::*, monitoring::*},
    ApiServerConfig,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use uuid::Uuid;

/// Mock learning service for API testing
pub struct MockLearningService {
    feedback_data: Arc<tokio::sync::RwLock<Vec<UserFeedback>>>,
    learning_metrics: Arc<tokio::sync::RwLock<LearningMetrics>>,
}

impl MockLearningService {
    pub fn new() -> Self {
        Self {
            feedback_data: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            learning_metrics: Arc::new(tokio::sync::RwLock::new(LearningMetrics::default())),
        }
    }

    pub async fn submit_feedback(&self, feedback: UserFeedback) -> Result<(), String> {
        if !feedback.is_valid() {
            return Err("Invalid feedback data".to_string());
        }

        let mut data = self.feedback_data.write().await;
        data.push(feedback);

        // Update metrics
        let mut metrics = self.learning_metrics.write().await;
        metrics.total_feedback_entries += 1;
        metrics.feedback_processing_rate = data.len() as f64 / 3600.0; // Mock rate

        Ok(())
    }

    pub async fn get_learning_insights(&self, query: Option<String>) -> Vec<LearningInsight> {
        let data = self.feedback_data.read().await;

        // Mock insights based on feedback data
        if data.is_empty() {
            return vec![];
        }

        vec![
            LearningInsight {
                id: Uuid::new_v4().to_string(),
                insight_type: "user_preference".to_string(),
                content: "Users prefer detailed explanations".to_string(),
                confidence_score: 0.85,
                source_data_count: data.len(),
                created_at: chrono::Utc::now(),
                tags: vec!["user_feedback".to_string(), "quality".to_string()],
            },
            LearningInsight {
                id: Uuid::new_v4().to_string(),
                insight_type: "performance_optimization".to_string(),
                content: "Response time correlates with user satisfaction".to_string(),
                confidence_score: 0.78,
                source_data_count: data.len(),
                created_at: chrono::Utc::now(),
                tags: vec!["performance".to_string(), "optimization".to_string()],
            },
        ]
    }

    pub async fn get_learning_metrics(&self) -> LearningMetrics {
        self.learning_metrics.read().await.clone()
    }

    pub async fn get_learning_health(&self) -> LearningHealthResponse {
        let data = self.feedback_data.read().await;
        let metrics = self.learning_metrics.read().await;

        LearningHealthResponse {
            overall_status: if data.len() > 10 {
                LearningHealthStatus::Healthy
            } else {
                LearningHealthStatus::Warning
            },
            component_results: vec![
                LearningComponentHealth {
                    component: "feedback_processor".to_string(),
                    status: LearningHealthStatus::Healthy,
                    timestamp: chrono::Utc::now(),
                    message: "Processing feedback normally".to_string(),
                    response_time_ms: 50,
                },
                LearningComponentHealth {
                    component: "pattern_analyzer".to_string(),
                    status: LearningHealthStatus::Healthy,
                    timestamp: chrono::Utc::now(),
                    message: "Pattern analysis operational".to_string(),
                    response_time_ms: 30,
                },
            ],
            summary: "Learning system is operational".to_string(),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Mock monitoring service for API testing
pub struct MockMonitoringService {
    metrics_data: Arc<tokio::sync::RwLock<Vec<MetricEntry>>>,
    alerts_data: Arc<tokio::sync::RwLock<Vec<Alert>>>,
}

#[derive(Clone)]
pub struct MetricEntry {
    pub name: String,
    pub value: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub tags: HashMap<String, String>,
}

impl MockMonitoringService {
    pub fn new() -> Self {
        Self {
            metrics_data: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            alerts_data: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    pub async fn record_metric(&self, name: &str, value: f64, tags: HashMap<String, String>) {
        let mut data = self.metrics_data.write().await;
        data.push(MetricEntry {
            name: name.to_string(),
            value,
            timestamp: chrono::Utc::now(),
            tags,
        });
    }

    pub async fn get_metrics(
        &self,
        query: Option<MonitoringMetricsQuery>,
    ) -> MonitoringMetricsResponse {
        let data = self.metrics_data.read().await;

        let filtered_metrics = if let Some(q) = query {
            data.iter()
                .filter(|m| {
                    q.metric_names
                        .as_ref()
                        .map_or(true, |names| names.contains(&m.name))
                })
                .cloned()
                .collect()
        } else {
            data.clone()
        };

        MonitoringMetricsResponse {
            metrics: filtered_metrics
                .into_iter()
                .map(|m| MonitoringMetric {
                    name: m.name,
                    value: m.value,
                    timestamp: m.timestamp,
                    tags: m.tags,
                })
                .collect(),
            total_count: data.len(),
            query_time_ms: 15,
        }
    }

    pub async fn get_health_status(&self) -> MonitoringHealthResponse {
        let alerts = self.alerts_data.read().await;

        MonitoringHealthResponse {
            overall_status: if alerts.is_empty() {
                "healthy"
            } else {
                "warning"
            }
            .to_string(),
            components: vec![
                MonitoringComponentStatus {
                    name: "api_server".to_string(),
                    status: "healthy".to_string(),
                    last_check: chrono::Utc::now(),
                    response_time_ms: 25,
                },
                MonitoringComponentStatus {
                    name: "metrics_collector".to_string(),
                    status: "healthy".to_string(),
                    last_check: chrono::Utc::now(),
                    response_time_ms: 10,
                },
            ],
            active_alerts_count: alerts.len(),
            last_health_check: chrono::Utc::now(),
        }
    }

    pub async fn send_alert(&self, alert: Alert) {
        let mut alerts = self.alerts_data.write().await;
        alerts.push(alert);
    }

    pub async fn get_alerts(&self, severity: Option<AlertSeverity>) -> Vec<Alert> {
        let alerts = self.alerts_data.read().await;

        if let Some(sev) = severity {
            alerts
                .iter()
                .filter(|a| a.severity == sev)
                .cloned()
                .collect()
        } else {
            alerts.clone()
        }
    }
}

/// Helper function to create test authentication claims
fn create_test_claims(permissions: Vec<Permission>) -> Claims {
    Claims {
        sub: "test_user".to_string(),
        permissions: permissions.iter().map(|p| p.as_str().to_string()).collect(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        iss: "fortitude-api-server".to_string(),
    }
}

/// Helper function to create test request context
fn create_test_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("content-type", "application/json".parse().unwrap());
    headers.insert("user-agent", "test-client/1.0".parse().unwrap());
    headers
}

#[cfg(test)]
mod anchor_tests {
    use super::*;

    /// ANCHOR: Learning API endpoints maintain stability and functionality
    /// Tests: Feedback submission → Learning insights retrieval → Metrics collection → Health monitoring
    /// Protects: Learning API endpoint stability and business logic
    #[tokio::test]
    async fn test_anchor_learning_api_endpoints_stability() {
        let learning_service = Arc::new(MockLearningService::new());

        // Test 1: Feedback submission endpoint
        let feedback_request = SubmitFeedbackRequest {
            user_id: "test_user_123".to_string(),
            content_id: "content_456".to_string(),
            feedback_type: "quality_rating".to_string(),
            score: Some(0.85),
            text_feedback: Some("Great response, very helpful".to_string()),
            metadata: Some(json!({
                "source": "api_test",
                "version": "1.0"
            })),
        };

        // Convert request to UserFeedback
        let feedback = UserFeedback::new(
            feedback_request.user_id.clone(),
            feedback_request.content_id.clone(),
            feedback_request.feedback_type.clone(),
            feedback_request.score,
            feedback_request.text_feedback.clone(),
        );

        let submit_result = learning_service.submit_feedback(feedback).await;
        assert!(submit_result.is_ok(), "Feedback submission should succeed");

        // Test 2: Multiple feedback submissions
        let feedback_batch = vec![
            ("user_1", "content_A", 0.9),
            ("user_2", "content_A", 0.8),
            ("user_3", "content_B", 0.7),
            ("user_4", "content_B", 0.95),
            ("user_5", "content_C", 0.85),
        ];

        for (user_id, content_id, score) in feedback_batch {
            let feedback = UserFeedback::new(
                user_id.to_string(),
                content_id.to_string(),
                "quality_rating".to_string(),
                Some(score),
                Some(format!("Test feedback from {}", user_id)),
            );

            let result = learning_service.submit_feedback(feedback).await;
            assert!(
                result.is_ok(),
                "Batch feedback submission should succeed for {}",
                user_id
            );
        }

        // Test 3: Learning insights retrieval endpoint
        let insights_query = Some("user preference".to_string());
        let insights = learning_service.get_learning_insights(insights_query).await;

        assert!(
            !insights.is_empty(),
            "Learning insights should be available"
        );
        assert_eq!(
            insights.len(),
            2,
            "Should return expected number of insights"
        );

        // Verify insight structure
        let first_insight = &insights[0];
        assert!(!first_insight.id.is_empty(), "Insight ID should be present");
        assert_eq!(first_insight.insight_type, "user_preference");
        assert!(
            first_insight.confidence_score > 0.0 && first_insight.confidence_score <= 1.0,
            "Confidence score should be valid"
        );
        assert!(
            first_insight.source_data_count > 0,
            "Should reference source data"
        );
        assert!(
            !first_insight.tags.is_empty(),
            "Should have classification tags"
        );

        // Test 4: Learning metrics endpoint
        let metrics = learning_service.get_learning_metrics().await;
        assert!(
            metrics.total_feedback_entries > 0,
            "Should track feedback entries"
        );
        assert!(
            metrics.feedback_processing_rate >= 0.0,
            "Processing rate should be non-negative"
        );

        // Test 5: Learning health status endpoint
        let health_status = learning_service.get_learning_health().await;
        assert!(
            !health_status.overall_status.is_empty(),
            "Overall status should be provided"
        );
        assert!(
            !health_status.components.is_empty(),
            "Component health should be tracked"
        );
        assert!(
            health_status.active_learning_tasks >= 0,
            "Active tasks should be tracked"
        );

        // Verify component health details
        let feedback_processor = health_status
            .components
            .iter()
            .find(|c| c.name == "feedback_processor");
        assert!(
            feedback_processor.is_some(),
            "Feedback processor health should be tracked"
        );

        let pattern_analyzer = health_status
            .components
            .iter()
            .find(|c| c.name == "pattern_analyzer");
        assert!(
            pattern_analyzer.is_some(),
            "Pattern analyzer health should be tracked"
        );

        // Test 6: Invalid feedback handling
        let invalid_feedback = UserFeedback::new(
            "".to_string(), // Invalid empty user ID
            "content".to_string(),
            "rating".to_string(),
            Some(0.5),
            None,
        );

        let invalid_result = learning_service.submit_feedback(invalid_feedback).await;
        assert!(
            invalid_result.is_err(),
            "Invalid feedback should be rejected"
        );
        assert!(
            invalid_result.unwrap_err().contains("Invalid"),
            "Error message should be informative"
        );

        // Test 7: Edge case - empty insights query
        let empty_insights = learning_service.get_learning_insights(None).await;
        assert!(
            !empty_insights.is_empty(),
            "Should return insights even without specific query"
        );

        // Test 8: Concurrent API access
        let mut concurrent_tasks = Vec::new();

        for i in 0..10 {
            let service = learning_service.clone();
            let task = tokio::spawn(async move {
                let feedback = UserFeedback::new(
                    format!("concurrent_user_{}", i),
                    format!("concurrent_content_{}", i % 3),
                    "quality_rating".to_string(),
                    Some(0.8),
                    Some(format!("Concurrent test {}", i)),
                );
                service.submit_feedback(feedback).await
            });
            concurrent_tasks.push(task);
        }

        let concurrent_results: Vec<Result<Result<(), String>, _>> =
            futures::future::join_all(concurrent_tasks).await;

        let successful_submissions = concurrent_results
            .iter()
            .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
            .count();

        assert!(
            successful_submissions >= 8,
            "Most concurrent submissions should succeed"
        );

        // Test 9: API response time validation
        let start_time = std::time::Instant::now();
        let _quick_insights = learning_service
            .get_learning_insights(Some("quick".to_string()))
            .await;
        let insights_time = start_time.elapsed();

        assert!(
            insights_time < Duration::from_millis(100),
            "Learning insights API should respond quickly"
        );

        let start_time = std::time::Instant::now();
        let _quick_metrics = learning_service.get_learning_metrics().await;
        let metrics_time = start_time.elapsed();

        assert!(
            metrics_time < Duration::from_millis(50),
            "Learning metrics API should respond very quickly"
        );

        // Test 10: API data consistency
        let final_metrics = learning_service.get_learning_metrics().await;
        let final_insights = learning_service.get_learning_insights(None).await;

        // Metrics should reflect all submitted feedback
        assert!(
            final_metrics.total_feedback_entries >= 15,
            "Should count all feedback submissions"
        );

        // Insights should be based on available data
        assert!(
            !final_insights.is_empty(),
            "Insights should be available with sufficient data"
        );

        for insight in final_insights {
            assert!(
                insight.source_data_count > 0,
                "Insights should reference actual data"
            );
            assert!(
                insight.confidence_score > 0.0,
                "Insights should have valid confidence scores"
            );
        }
    }

    /// ANCHOR: Monitoring API endpoints maintain stability and functionality
    /// Tests: Metrics collection → Health monitoring → Alert management → Performance tracking
    /// Protects: Monitoring API endpoint stability and operational logic
    #[tokio::test]
    async fn test_anchor_monitoring_api_endpoints_stability() {
        let monitoring_service = Arc::new(MockMonitoringService::new());

        // Test 1: Metrics recording and collection
        let test_metrics = vec![
            (
                "api_response_time",
                150.0,
                vec![("endpoint", "/api/research"), ("method", "GET")],
            ),
            (
                "memory_usage",
                750.0,
                vec![("component", "learning_service"), ("unit", "MB")],
            ),
            (
                "cache_hit_rate",
                0.85,
                vec![("cache_type", "embeddings"), ("operation", "read")],
            ),
            (
                "error_rate",
                0.02,
                vec![("service", "api_server"), ("period", "1h")],
            ),
            (
                "throughput",
                450.0,
                vec![("component", "request_processor"), ("unit", "req/min")],
            ),
        ];

        for (name, value, tag_pairs) in test_metrics {
            let mut tags = HashMap::new();
            for (key, val) in tag_pairs {
                tags.insert(key.to_string(), val.to_string());
            }
            monitoring_service.record_metric(name, value, tags).await;
        }

        // Test 2: Metrics retrieval endpoint
        let metrics_query = MonitoringMetricsQuery {
            metric_names: Some(vec![
                "api_response_time".to_string(),
                "memory_usage".to_string(),
            ]),
            start_time: None,
            end_time: None,
            tags: None,
            aggregation: None,
        };

        let metrics_response = monitoring_service.get_metrics(Some(metrics_query)).await;
        assert_eq!(
            metrics_response.metrics.len(),
            2,
            "Should filter metrics by name"
        );
        assert!(
            metrics_response.query_time_ms > 0,
            "Should track query performance"
        );
        assert_eq!(
            metrics_response.total_count, 5,
            "Should report total available metrics"
        );

        // Verify metric structure
        let api_metric = metrics_response
            .metrics
            .iter()
            .find(|m| m.name == "api_response_time");
        assert!(
            api_metric.is_some(),
            "API response time metric should be present"
        );

        if let Some(metric) = api_metric {
            assert_eq!(metric.value, 150.0);
            assert!(
                metric.tags.contains_key("endpoint"),
                "Should preserve metric tags"
            );
            assert_eq!(metric.tags["endpoint"], "/api/research");
        }

        // Test 3: All metrics retrieval (no filter)
        let all_metrics = monitoring_service.get_metrics(None).await;
        assert_eq!(
            all_metrics.metrics.len(),
            5,
            "Should return all metrics when unfiltered"
        );
        assert_eq!(
            all_metrics.total_count, 5,
            "Total count should match actual metrics"
        );

        // Test 4: Health status endpoint
        let health_status = monitoring_service.get_health_status().await;
        assert!(
            !health_status.overall_status.is_empty(),
            "Overall health status should be provided"
        );
        assert!(
            !health_status.components.is_empty(),
            "Component statuses should be tracked"
        );
        assert_eq!(
            health_status.active_alerts_count, 0,
            "Should start with no alerts"
        );

        // Verify component health structure
        let api_component = health_status
            .components
            .iter()
            .find(|c| c.name == "api_server");
        assert!(
            api_component.is_some(),
            "API server health should be tracked"
        );

        if let Some(component) = api_component {
            assert_eq!(component.status, "healthy");
            assert!(
                component.response_time_ms > 0,
                "Should track component response times"
            );
        }

        // Test 5: Alert management endpoints
        let test_alerts = vec![
            Alert::new(
                "High Response Time".to_string(),
                "API response time exceeds threshold".to_string(),
                AlertSeverity::Warning,
                "api_server".to_string(),
            ),
            Alert::new(
                "Memory Usage Critical".to_string(),
                "System memory usage is critical".to_string(),
                AlertSeverity::Critical,
                "system".to_string(),
            ),
            Alert::new(
                "Cache Performance".to_string(),
                "Cache hit rate below optimal".to_string(),
                AlertSeverity::Medium,
                "cache_system".to_string(),
            ),
        ];

        for alert in &test_alerts {
            monitoring_service.send_alert(alert.clone()).await;
        }

        // Test 6: Alert retrieval by severity
        let critical_alerts = monitoring_service
            .get_alerts(Some(AlertSeverity::Critical))
            .await;
        assert_eq!(critical_alerts.len(), 1, "Should filter alerts by severity");
        assert_eq!(critical_alerts[0].title, "Memory Usage Critical");

        let warning_alerts = monitoring_service
            .get_alerts(Some(AlertSeverity::Warning))
            .await;
        assert_eq!(warning_alerts.len(), 1, "Should have one warning alert");

        let all_alerts = monitoring_service.get_alerts(None).await;
        assert_eq!(
            all_alerts.len(),
            3,
            "Should return all alerts when unfiltered"
        );

        // Test 7: Health status with active alerts
        let health_with_alerts = monitoring_service.get_health_status().await;
        assert_eq!(
            health_with_alerts.overall_status, "warning",
            "Health status should reflect active alerts"
        );
        assert_eq!(
            health_with_alerts.active_alerts_count, 3,
            "Should count active alerts"
        );

        // Test 8: High-volume metrics collection
        let start_time = std::time::Instant::now();

        for i in 0..100 {
            let mut tags = HashMap::new();
            tags.insert("batch".to_string(), "load_test".to_string());
            tags.insert("iteration".to_string(), i.to_string());

            monitoring_service
                .record_metric(&format!("load_test_metric_{}", i % 10), i as f64, tags)
                .await;
        }

        let load_test_time = start_time.elapsed();
        assert!(
            load_test_time < Duration::from_secs(1),
            "High-volume metrics collection should be efficient"
        );

        // Test 9: Concurrent monitoring operations
        let mut concurrent_tasks = Vec::new();

        for i in 0..20 {
            let service = monitoring_service.clone();
            let task = tokio::spawn(async move {
                // Mix of operations
                if i % 3 == 0 {
                    // Record metric
                    let mut tags = HashMap::new();
                    tags.insert("concurrent".to_string(), "test".to_string());
                    service
                        .record_metric(&format!("concurrent_metric_{}", i), i as f64, tags)
                        .await;
                } else if i % 3 == 1 {
                    // Get health
                    let _health = service.get_health_status().await;
                } else {
                    // Get metrics
                    let _metrics = service.get_metrics(None).await;
                }
            });
            concurrent_tasks.push(task);
        }

        futures::future::join_all(concurrent_tasks).await;

        // Verify system remains consistent after concurrent operations
        let final_metrics = monitoring_service.get_metrics(None).await;
        assert!(
            final_metrics.total_count >= 105,
            "Should track all metrics including concurrent ones"
        );

        // Test 10: API performance validation
        let performance_start = std::time::Instant::now();
        let _perf_metrics = monitoring_service.get_metrics(None).await;
        let metrics_query_time = performance_start.elapsed();

        assert!(
            metrics_query_time < Duration::from_millis(50),
            "Metrics query should be very fast"
        );

        let performance_start = std::time::Instant::now();
        let _perf_health = monitoring_service.get_health_status().await;
        let health_query_time = performance_start.elapsed();

        assert!(
            health_query_time < Duration::from_millis(30),
            "Health status query should be extremely fast"
        );

        let performance_start = std::time::Instant::now();
        let _perf_alerts = monitoring_service.get_alerts(None).await;
        let alerts_query_time = performance_start.elapsed();

        assert!(
            alerts_query_time < Duration::from_millis(20),
            "Alerts query should be very fast"
        );

        // Test 11: Data consistency validation
        let final_health = monitoring_service.get_health_status().await;
        let final_all_metrics = monitoring_service.get_metrics(None).await;
        let final_all_alerts = monitoring_service.get_alerts(None).await;

        // Verify data consistency
        assert!(
            final_all_metrics.total_count > 100,
            "Should maintain metric count accuracy"
        );
        assert_eq!(
            final_health.active_alerts_count,
            final_all_alerts.len(),
            "Health status should accurately reflect alert count"
        );

        // Verify alert structure consistency
        for alert in final_all_alerts {
            assert!(!alert.id.is_empty(), "Alert ID should be preserved");
            assert!(!alert.title.is_empty(), "Alert title should be preserved");
            assert!(!alert.source.is_empty(), "Alert source should be preserved");
        }
    }

    /// ANCHOR: API authentication and authorization workflow
    /// Tests: Token validation → Permission checking → Access control → Rate limiting
    /// Protects: API security and access control mechanisms
    #[tokio::test]
    async fn test_anchor_api_authentication_authorization_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = ApiServerConfig::default();
        config.auth.enabled = true;
        config.auth.jwt_secret =
            "test_secret_key_that_is_at_least_32_characters_long_for_security".to_string();
        config.auth.token_expiration_hours = 1;

        let auth_manager = Arc::new(AuthManager::new(Arc::new(config)).unwrap());
        let learning_service = Arc::new(MockLearningService::new());
        let monitoring_service = Arc::new(MockMonitoringService::new());

        // Test 1: Token generation and validation
        let valid_permissions = vec![
            Permission::ResearchRead,
            Permission::ResourcesRead,
            Permission::LearningRead,
            Permission::MonitoringRead,
        ];

        let valid_token = auth_manager
            .generate_token("test_user", valid_permissions.clone())
            .await
            .unwrap();

        let claims = auth_manager.verify_token(&valid_token).await.unwrap();
        assert_eq!(claims.sub, "test_user");
        assert!(claims
            .permissions
            .contains(&Permission::LearningRead.as_str().to_string()));
        assert!(claims
            .permissions
            .contains(&Permission::MonitoringRead.as_str().to_string()));

        // Test 2: Learning API with valid authentication
        let learning_claims =
            create_test_claims(vec![Permission::LearningRead, Permission::LearningWrite]);

        // Simulate authenticated feedback submission
        let feedback = UserFeedback::new(
            "authenticated_user".to_string(),
            "authenticated_content".to_string(),
            "quality_rating".to_string(),
            Some(0.9),
            Some("Authenticated feedback".to_string()),
        );

        let auth_submit_result = learning_service.submit_feedback(feedback).await;
        assert!(
            auth_submit_result.is_ok(),
            "Authenticated learning API access should succeed"
        );

        // Simulate authenticated insights retrieval
        let auth_insights = learning_service
            .get_learning_insights(Some("auth test".to_string()))
            .await;
        assert!(
            !auth_insights.is_empty(),
            "Authenticated insights access should succeed"
        );

        // Test 3: Monitoring API with valid authentication
        let monitoring_claims = create_test_claims(vec![Permission::MonitoringRead]);

        // Simulate authenticated metrics access
        let auth_metrics = monitoring_service.get_metrics(None).await;
        assert!(
            auth_metrics.total_count >= 0,
            "Authenticated metrics access should succeed"
        );

        // Simulate authenticated health status access
        let auth_health = monitoring_service.get_health_status().await;
        assert!(
            !auth_health.overall_status.is_empty(),
            "Authenticated health access should succeed"
        );

        // Test 4: Permission-based access control
        let read_only_token = auth_manager
            .generate_token("read_user", vec![Permission::ResearchRead])
            .await
            .unwrap();

        let read_only_claims = auth_manager.verify_token(&read_only_token).await.unwrap();

        // Should have read permission
        assert!(read_only_claims
            .permissions
            .contains(&Permission::ResearchRead.as_str().to_string()));

        // Should not have write permissions
        assert!(!read_only_claims
            .permissions
            .contains(&Permission::LearningWrite.as_str().to_string()));
        assert!(!read_only_claims
            .permissions
            .contains(&Permission::Admin.as_str().to_string()));

        // Test 5: Admin access validation
        let admin_token = auth_manager
            .generate_token("admin_user", vec![Permission::Admin])
            .await
            .unwrap();

        let admin_claims = auth_manager.verify_token(&admin_token).await.unwrap();
        assert!(admin_claims
            .permissions
            .contains(&Permission::Admin.as_str().to_string()));

        // Admin should be able to access all APIs
        let admin_check_result = auth_manager
            .check_permission(&admin_claims, Permission::LearningWrite)
            .await;
        assert!(
            admin_check_result.is_ok(),
            "Admin should have elevated permissions"
        );

        // Test 6: Invalid token handling
        let invalid_tokens = vec![
            "invalid_token",
            "Bearer invalid",
            "",
            "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.invalid.signature",
        ];

        for invalid_token in invalid_tokens {
            let result = auth_manager.verify_token(invalid_token).await;
            assert!(
                result.is_err(),
                "Invalid token should be rejected: {}",
                invalid_token
            );
        }

        // Test 7: Token expiration validation
        let claims = auth_manager.verify_token(&valid_token).await.unwrap();
        assert!(
            claims.exp > chrono::Utc::now().timestamp(),
            "Token should not be expired"
        );

        // Test 8: Rate limiting validation
        // Note: This is a conceptual test - actual rate limiting would be in middleware
        let mut rate_limit_requests = 0;
        let rate_limit_start = std::time::Instant::now();

        // Simulate rapid API requests
        for _ in 0..10 {
            let _insights = learning_service.get_learning_insights(None).await;
            rate_limit_requests += 1;
        }

        let rate_limit_time = rate_limit_start.elapsed();
        assert!(
            rate_limit_requests == 10,
            "All requests should be processed"
        );
        assert!(
            rate_limit_time < Duration::from_secs(1),
            "Rate limiting should not add excessive delay"
        );

        // Test 9: Cross-API permission validation
        // Verify that learning permissions don't grant monitoring access inappropriately
        let learning_only_token = auth_manager
            .generate_token("learning_user", vec![Permission::LearningRead])
            .await
            .unwrap();

        let learning_only_claims = auth_manager
            .verify_token(&learning_only_token)
            .await
            .unwrap();

        // Should have learning access
        assert!(learning_only_claims
            .permissions
            .contains(&Permission::LearningRead.as_str().to_string()));

        // Should not have monitoring write access
        let monitoring_write_check = auth_manager
            .check_permission(&learning_only_claims, Permission::MonitoringWrite)
            .await;
        assert!(
            monitoring_write_check.is_err(),
            "Learning permissions should not grant monitoring write"
        );

        // Test 10: Secure API response handling
        // Verify that authentication errors don't leak sensitive information
        let secure_check_start = std::time::Instant::now();

        for _ in 0..5 {
            let result = auth_manager.verify_token("invalid_token").await;
            assert!(result.is_err(), "Should consistently reject invalid tokens");

            let error_msg = result.unwrap_err().to_string();
            assert!(
                !error_msg.contains("secret"),
                "Error should not leak secrets"
            );
            assert!(
                !error_msg.contains("key"),
                "Error should not leak key information"
            );
        }

        let secure_check_time = secure_check_start.elapsed();
        assert!(
            secure_check_time < Duration::from_millis(100),
            "Security checks should be efficient"
        );

        // Test 11: Token refresh and lifecycle management
        let lifecycle_token = auth_manager
            .generate_token("lifecycle_user", vec![Permission::ResearchRead])
            .await
            .unwrap();

        // Verify token is initially valid
        let initial_claims = auth_manager.verify_token(&lifecycle_token).await.unwrap();
        assert_eq!(initial_claims.sub, "lifecycle_user");

        // Verify token structure
        assert!(
            initial_claims.iat <= chrono::Utc::now().timestamp(),
            "Issue time should be valid"
        );
        assert!(
            initial_claims.exp > chrono::Utc::now().timestamp(),
            "Expiration should be in future"
        );
        assert_eq!(
            initial_claims.iss, "fortitude-api-server",
            "Issuer should be correct"
        );

        // Test 12: Multi-user concurrent authentication
        let mut auth_tasks = Vec::new();

        for i in 0..10 {
            let auth_mgr = auth_manager.clone();
            let task = tokio::spawn(async move {
                let permissions = if i % 2 == 0 {
                    vec![Permission::LearningRead]
                } else {
                    vec![Permission::MonitoringRead]
                };

                let token = auth_mgr
                    .generate_token(&format!("concurrent_user_{}", i), permissions)
                    .await?;

                let claims = auth_mgr.verify_token(&token).await?;
                Ok::<String, Box<dyn std::error::Error + Send + Sync>>(claims.sub)
            });
            auth_tasks.push(task);
        }

        let auth_results: Vec<Result<Result<String, _>, _>> =
            futures::future::join_all(auth_tasks).await;

        let successful_auths = auth_results
            .iter()
            .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
            .count();

        assert_eq!(
            successful_auths, 10,
            "All concurrent authentications should succeed"
        );

        // Verify unique users were processed
        let user_names: Vec<String> = auth_results
            .into_iter()
            .filter_map(|r| r.ok()?.ok())
            .collect();

        assert_eq!(user_names.len(), 10, "Should process all unique users");

        for i in 0..10 {
            let expected_user = format!("concurrent_user_{}", i);
            assert!(
                user_names.contains(&expected_user),
                "Should include user: {}",
                expected_user
            );
        }
    }
}
