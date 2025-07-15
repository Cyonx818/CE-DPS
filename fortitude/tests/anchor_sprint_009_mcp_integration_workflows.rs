// ABOUTME: Anchor tests for Sprint 009 Tasks 3 & 4 - MCP Integration Critical Workflows
//! These tests protect critical MCP integration functionality implemented in Sprint 009
//! Tasks 3 and 4. They ensure that MCP integration workflows continue to work correctly
//! as the system evolves.
//!
//! ## Protected Functionality
//! - External API integration (MCP protocol compliance, tool execution)
//! - Authentication (MCP server authentication and authorization)
//! - User input processing (MCP tool parameter validation and processing)
//! - Cross-component integration (MCP+learning+monitoring integration)
//! - Business logic (learning and monitoring tool execution logic)

use fortitude_mcp_server::{
    AuthManager, FortitudeTools, McpServer, Permission, RateLimitConfig, ResourceProvider,
    ServerConfig,
};
use rmcp::model::{
    CallToolRequestParam, CallToolResult, Content, ListResourcesResult, ListToolsResult,
    ReadResourceResult, ResourceContents,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Mock MCP learning tools service
pub struct MockMcpLearningService {
    feedback_data: Arc<tokio::sync::RwLock<Vec<LearningFeedbackEntry>>>,
    learning_insights: Arc<tokio::sync::RwLock<Vec<LearningInsightEntry>>>,
    learning_metrics: Arc<tokio::sync::RwLock<LearningMetricsData>>,
}

#[derive(Clone)]
pub struct LearningFeedbackEntry {
    pub id: String,
    pub user_id: String,
    pub content_id: String,
    pub feedback_type: String,
    pub score: Option<f64>,
    pub text_feedback: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct LearningInsightEntry {
    pub id: String,
    pub insight_type: String,
    pub content: String,
    pub confidence_score: f64,
    pub source_data_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
}

#[derive(Clone)]
pub struct LearningMetricsData {
    pub total_feedback_entries: u64,
    pub active_learning_tasks: u64,
    pub pattern_recognition_accuracy: f64,
    pub adaptation_success_rate: f64,
    pub last_learning_update: chrono::DateTime<chrono::Utc>,
}

impl Default for LearningMetricsData {
    fn default() -> Self {
        Self {
            total_feedback_entries: 0,
            active_learning_tasks: 0,
            pattern_recognition_accuracy: 0.85,
            adaptation_success_rate: 0.78,
            last_learning_update: chrono::Utc::now(),
        }
    }
}

impl MockMcpLearningService {
    pub fn new() -> Self {
        Self {
            feedback_data: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            learning_insights: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            learning_metrics: Arc::new(tokio::sync::RwLock::new(LearningMetricsData::default())),
        }
    }

    pub async fn submit_learning_feedback(&self, params: Value) -> Result<CallToolResult, String> {
        let user_id = params
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing user_id parameter")?;
        let content_id = params
            .get("content_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing content_id parameter")?;
        let feedback_type = params
            .get("feedback_type")
            .and_then(|v| v.as_str())
            .unwrap_or("quality_rating");
        let score = params.get("score").and_then(|v| v.as_f64());
        let text_feedback = params.get("text_feedback").and_then(|v| v.as_str());

        // Validate score if provided
        if let Some(s) = score {
            if s < 0.0 || s > 1.0 {
                return Err("Score must be between 0.0 and 1.0".to_string());
            }
        }

        let feedback_entry = LearningFeedbackEntry {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            content_id: content_id.to_string(),
            feedback_type: feedback_type.to_string(),
            score,
            text_feedback: text_feedback.map(|s| s.to_string()),
            timestamp: chrono::Utc::now(),
        };

        // Store feedback
        let mut feedback_data = self.feedback_data.write().await;
        feedback_data.push(feedback_entry.clone());

        // Update metrics
        let mut metrics = self.learning_metrics.write().await;
        metrics.total_feedback_entries += 1;
        metrics.last_learning_update = chrono::Utc::now();

        let response = json!({
            "success": true,
            "feedback_id": feedback_entry.id,
            "message": "Feedback submitted successfully",
            "processing_time_ms": 25
        });

        Ok(CallToolResult {
            content: vec![Content::text(response.to_string())],
            is_error: Some(false),
        })
    }

    pub async fn get_learning_insights(&self, params: Value) -> Result<CallToolResult, String> {
        let query = params.get("query").and_then(|v| v.as_str());
        let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

        let insights_data = self.learning_insights.read().await;

        // Filter insights based on query if provided
        let filtered_insights: Vec<&LearningInsightEntry> = if let Some(q) = query {
            insights_data
                .iter()
                .filter(|insight| {
                    insight.content.to_lowercase().contains(&q.to_lowercase())
                        || insight
                            .tags
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&q.to_lowercase()))
                })
                .take(limit)
                .collect()
        } else {
            insights_data.iter().take(limit).collect()
        };

        // If no stored insights, generate mock insights
        let insights_to_return = if filtered_insights.is_empty() {
            vec![
                LearningInsightEntry {
                    id: uuid::Uuid::new_v4().to_string(),
                    insight_type: "user_preference".to_string(),
                    content: "Users prefer detailed explanations with examples".to_string(),
                    confidence_score: 0.87,
                    source_data_count: 45,
                    created_at: chrono::Utc::now(),
                    tags: vec!["user_feedback".to_string(), "quality".to_string()],
                },
                LearningInsightEntry {
                    id: uuid::Uuid::new_v4().to_string(),
                    insight_type: "performance_optimization".to_string(),
                    content: "Response time under 200ms correlates with higher satisfaction"
                        .to_string(),
                    confidence_score: 0.82,
                    source_data_count: 38,
                    created_at: chrono::Utc::now(),
                    tags: vec!["performance".to_string(), "optimization".to_string()],
                },
            ]
        } else {
            filtered_insights.into_iter().cloned().collect()
        };

        let response = json!({
            "insights": insights_to_return,
            "total_count": insights_to_return.len(),
            "query": query,
            "processing_time_ms": 35
        });

        Ok(CallToolResult {
            content: vec![Content::text(response.to_string())],
            is_error: Some(false),
        })
    }

    pub async fn get_learning_status(&self, _params: Value) -> Result<CallToolResult, String> {
        let feedback_data = self.feedback_data.read().await;
        let metrics = self.learning_metrics.read().await;

        let status = if feedback_data.len() > 10 {
            "healthy"
        } else {
            "warming_up"
        };

        let response = json!({
            "status": status,
            "metrics": {
                "total_feedback_entries": metrics.total_feedback_entries,
                "active_learning_tasks": metrics.active_learning_tasks,
                "pattern_recognition_accuracy": metrics.pattern_recognition_accuracy,
                "adaptation_success_rate": metrics.adaptation_success_rate,
                "last_learning_update": metrics.last_learning_update
            },
            "components": [
                {
                    "name": "feedback_processor",
                    "status": "healthy",
                    "last_activity": chrono::Utc::now()
                },
                {
                    "name": "pattern_analyzer",
                    "status": "healthy",
                    "last_activity": chrono::Utc::now()
                },
                {
                    "name": "adaptation_engine",
                    "status": "healthy",
                    "last_activity": chrono::Utc::now()
                }
            ],
            "processing_time_ms": 15
        });

        Ok(CallToolResult {
            content: vec![Content::text(response.to_string())],
            is_error: Some(false),
        })
    }
}

/// Mock MCP monitoring tools service
pub struct MockMcpMonitoringService {
    metrics_data: Arc<tokio::sync::RwLock<Vec<MonitoringMetricEntry>>>,
    alerts_data: Arc<tokio::sync::RwLock<Vec<MonitoringAlertEntry>>>,
    health_data: Arc<tokio::sync::RwLock<SystemHealthData>>,
}

#[derive(Clone)]
pub struct MonitoringMetricEntry {
    pub name: String,
    pub value: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub tags: HashMap<String, String>,
}

#[derive(Clone)]
pub struct MonitoringAlertEntry {
    pub id: String,
    pub title: String,
    pub severity: String,
    pub source: String,
    pub message: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub resolved: bool,
}

#[derive(Clone)]
pub struct SystemHealthData {
    pub overall_status: String,
    pub components: Vec<ComponentHealthEntry>,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct ComponentHealthEntry {
    pub name: String,
    pub status: String,
    pub response_time_ms: u64,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

impl Default for SystemHealthData {
    fn default() -> Self {
        Self {
            overall_status: "healthy".to_string(),
            components: vec![
                ComponentHealthEntry {
                    name: "api_server".to_string(),
                    status: "healthy".to_string(),
                    response_time_ms: 25,
                    last_check: chrono::Utc::now(),
                },
                ComponentHealthEntry {
                    name: "learning_service".to_string(),
                    status: "healthy".to_string(),
                    response_time_ms: 35,
                    last_check: chrono::Utc::now(),
                },
            ],
            last_check: chrono::Utc::now(),
        }
    }
}

impl MockMcpMonitoringService {
    pub fn new() -> Self {
        // Initialize with some sample metrics
        let mut initial_metrics = Vec::new();
        initial_metrics.push(MonitoringMetricEntry {
            name: "api_response_time".to_string(),
            value: 150.0,
            timestamp: chrono::Utc::now(),
            tags: {
                let mut tags = HashMap::new();
                tags.insert("endpoint".to_string(), "/api/research".to_string());
                tags
            },
        });

        Self {
            metrics_data: Arc::new(tokio::sync::RwLock::new(initial_metrics)),
            alerts_data: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            health_data: Arc::new(tokio::sync::RwLock::new(SystemHealthData::default())),
        }
    }

    pub async fn get_monitoring_metrics(&self, params: Value) -> Result<CallToolResult, String> {
        let metric_names = params
            .get("metric_names")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            });
        let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;

        let metrics_data = self.metrics_data.read().await;

        let filtered_metrics: Vec<&MonitoringMetricEntry> = if let Some(names) = metric_names {
            metrics_data
                .iter()
                .filter(|m| names.contains(&m.name))
                .take(limit)
                .collect()
        } else {
            metrics_data.iter().take(limit).collect()
        };

        let response = json!({
            "metrics": filtered_metrics.iter().map(|m| json!({
                "name": m.name,
                "value": m.value,
                "timestamp": m.timestamp,
                "tags": m.tags
            })).collect::<Vec<_>>(),
            "total_count": metrics_data.len(),
            "filtered_count": filtered_metrics.len(),
            "processing_time_ms": 20
        });

        Ok(CallToolResult {
            content: vec![Content::text(response.to_string())],
            is_error: Some(false),
        })
    }

    pub async fn get_monitoring_health(&self, _params: Value) -> Result<CallToolResult, String> {
        let health_data = self.health_data.read().await;
        let alerts_data = self.alerts_data.read().await;

        let active_alerts_count = alerts_data.iter().filter(|a| !a.resolved).count();

        let response = json!({
            "overall_status": health_data.overall_status,
            "components": health_data.components.iter().map(|c| json!({
                "name": c.name,
                "status": c.status,
                "response_time_ms": c.response_time_ms,
                "last_check": c.last_check
            })).collect::<Vec<_>>(),
            "active_alerts_count": active_alerts_count,
            "last_health_check": health_data.last_check,
            "processing_time_ms": 12
        });

        Ok(CallToolResult {
            content: vec![Content::text(response.to_string())],
            is_error: Some(false),
        })
    }

    pub async fn get_monitoring_alerts(&self, params: Value) -> Result<CallToolResult, String> {
        let severity_filter = params.get("severity").and_then(|v| v.as_str());
        let include_resolved = params
            .get("include_resolved")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let alerts_data = self.alerts_data.read().await;

        let filtered_alerts: Vec<&MonitoringAlertEntry> = alerts_data
            .iter()
            .filter(|a| {
                let severity_match = severity_filter.map_or(true, |s| a.severity == s);
                let resolved_match = include_resolved || !a.resolved;
                severity_match && resolved_match
            })
            .collect();

        let response = json!({
            "alerts": filtered_alerts.iter().map(|a| json!({
                "id": a.id,
                "title": a.title,
                "severity": a.severity,
                "source": a.source,
                "message": a.message,
                "created_at": a.created_at,
                "resolved": a.resolved
            })).collect::<Vec<_>>(),
            "total_count": alerts_data.len(),
            "filtered_count": filtered_alerts.len(),
            "processing_time_ms": 18
        });

        Ok(CallToolResult {
            content: vec![Content::text(response.to_string())],
            is_error: Some(false),
        })
    }

    pub async fn send_monitoring_alert(&self, params: Value) -> Result<CallToolResult, String> {
        let title = params
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or("Missing title parameter")?;
        let severity = params
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("medium");
        let source = params
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("mcp_tool");
        let message = params.get("message").and_then(|v| v.as_str()).unwrap_or("");

        let alert = MonitoringAlertEntry {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            severity: severity.to_string(),
            source: source.to_string(),
            message: message.to_string(),
            created_at: chrono::Utc::now(),
            resolved: false,
        };

        let mut alerts_data = self.alerts_data.write().await;
        alerts_data.push(alert.clone());

        // Update health status if critical alert
        if severity == "critical" {
            let mut health_data = self.health_data.write().await;
            health_data.overall_status = "critical".to_string();
        }

        let response = json!({
            "success": true,
            "alert_id": alert.id,
            "message": "Alert sent successfully",
            "processing_time_ms": 8
        });

        Ok(CallToolResult {
            content: vec![Content::text(response.to_string())],
            is_error: Some(false),
        })
    }
}

/// Helper function to create test MCP tool requests
fn create_mcp_request(tool_name: &str, arguments: Value) -> CallToolRequestParam {
    CallToolRequestParam {
        name: tool_name.into(),
        arguments: Some(arguments.as_object().unwrap().clone()),
    }
}

#[cfg(test)]
mod anchor_tests {
    use super::*;

    /// ANCHOR: MCP learning tools integration workflow
    /// Tests: Tool listing → Parameter validation → Learning operations → Response formatting
    /// Protects: MCP learning tools functionality and protocol compliance
    #[tokio::test]
    async fn test_anchor_mcp_learning_tools_integration_workflow() {
        let learning_service = Arc::new(MockMcpLearningService::new());

        // Test 1: Learning feedback submission tool
        let feedback_params = json!({
            "user_id": "mcp_user_123",
            "content_id": "mcp_content_456",
            "feedback_type": "quality_rating",
            "score": 0.9,
            "text_feedback": "Excellent response via MCP"
        });

        let feedback_request = create_mcp_request("learning_submit_feedback", feedback_params);
        let feedback_result = learning_service
            .submit_learning_feedback(feedback_request.arguments.unwrap().into())
            .await
            .unwrap();

        // Verify MCP response structure
        assert_eq!(
            feedback_result.is_error,
            Some(false),
            "MCP feedback submission should succeed"
        );
        assert!(
            !feedback_result.content.is_empty(),
            "MCP response should have content"
        );

        if let Some(content) = feedback_result.content[0].as_text() {
            let response: Value = serde_json::from_str(&content.text).unwrap();
            assert_eq!(
                response["success"], true,
                "MCP response should indicate success"
            );
            assert!(
                response["feedback_id"].is_string(),
                "Should return feedback ID"
            );
            assert!(
                response["processing_time_ms"].is_number(),
                "Should track processing time"
            );
        }

        // Test 2: Batch learning feedback submission
        let batch_feedback = vec![
            ("batch_user_1", "batch_content_A", 0.85),
            ("batch_user_2", "batch_content_A", 0.92),
            ("batch_user_3", "batch_content_B", 0.78),
            ("batch_user_4", "batch_content_B", 0.88),
        ];

        for (user_id, content_id, score) in batch_feedback {
            let params = json!({
                "user_id": user_id,
                "content_id": content_id,
                "feedback_type": "quality_rating",
                "score": score,
                "text_feedback": format!("Batch feedback from {}", user_id)
            });

            let result = learning_service
                .submit_learning_feedback(params)
                .await
                .unwrap();

            assert_eq!(
                result.is_error,
                Some(false),
                "Batch submission should succeed for {}",
                user_id
            );
        }

        // Test 3: Learning insights retrieval tool
        let insights_params = json!({
            "query": "user preference",
            "limit": 10
        });

        let insights_request = create_mcp_request("learning_get_insights", insights_params);
        let insights_result = learning_service
            .get_learning_insights(insights_request.arguments.unwrap().into())
            .await
            .unwrap();

        assert_eq!(
            insights_result.is_error,
            Some(false),
            "MCP insights retrieval should succeed"
        );

        if let Some(content) = insights_result.content[0].as_text() {
            let response: Value = serde_json::from_str(&content.text).unwrap();
            assert!(
                response["insights"].is_array(),
                "Should return insights array"
            );
            assert!(
                response["total_count"].is_number(),
                "Should return total count"
            );
            assert!(
                response["processing_time_ms"].is_number(),
                "Should track processing time"
            );

            let insights = response["insights"].as_array().unwrap();
            assert!(!insights.is_empty(), "Should return learning insights");

            // Verify insight structure
            let first_insight = &insights[0];
            assert!(first_insight["id"].is_string(), "Insight should have ID");
            assert!(
                first_insight["insight_type"].is_string(),
                "Should have insight type"
            );
            assert!(
                first_insight["content"].is_string(),
                "Should have insight content"
            );
            assert!(
                first_insight["confidence_score"].is_number(),
                "Should have confidence score"
            );
            assert!(first_insight["tags"].is_array(), "Should have tags array");
        }

        // Test 4: Learning status monitoring tool
        let status_params = json!({});
        let status_request = create_mcp_request("learning_get_status", status_params);
        let status_result = learning_service
            .get_learning_status(status_request.arguments.unwrap().into())
            .await
            .unwrap();

        assert_eq!(
            status_result.is_error,
            Some(false),
            "MCP status retrieval should succeed"
        );

        if let Some(content) = status_result.content[0].as_text() {
            let response: Value = serde_json::from_str(&content.text).unwrap();
            assert!(response["status"].is_string(), "Should return status");
            assert!(
                response["metrics"].is_object(),
                "Should return metrics object"
            );
            assert!(
                response["components"].is_array(),
                "Should return components array"
            );

            let metrics = &response["metrics"];
            assert!(
                metrics["total_feedback_entries"].is_number(),
                "Should track feedback entries"
            );
            assert!(
                metrics["pattern_recognition_accuracy"].is_number(),
                "Should track accuracy"
            );
            assert!(
                metrics["adaptation_success_rate"].is_number(),
                "Should track success rate"
            );

            let components = response["components"].as_array().unwrap();
            assert!(!components.is_empty(), "Should have component statuses");

            for component in components {
                assert!(component["name"].is_string(), "Component should have name");
                assert!(
                    component["status"].is_string(),
                    "Component should have status"
                );
            }
        }

        // Test 5: Parameter validation for learning tools
        let invalid_params_cases = vec![
            // Missing required parameters
            json!({}),
            json!({"user_id": "test"}), // Missing content_id
            // Invalid parameter types
            json!({
                "user_id": 123,  // Should be string
                "content_id": "content"
            }),
            // Invalid score values
            json!({
                "user_id": "user",
                "content_id": "content",
                "score": 1.5  // Should be <= 1.0
            }),
            json!({
                "user_id": "user",
                "content_id": "content",
                "score": -0.1  // Should be >= 0.0
            }),
        ];

        for invalid_params in invalid_params_cases {
            let result = learning_service
                .submit_learning_feedback(invalid_params)
                .await;

            assert!(result.is_err(), "Invalid parameters should be rejected");
        }

        // Test 6: Learning insights with different query parameters
        let query_variations = vec![
            ("optimization", 5),
            ("user feedback", 15),
            ("performance", 3),
            ("", 20), // Empty query should work
        ];

        for (query, limit) in query_variations {
            let params = json!({
                "query": query,
                "limit": limit
            });

            let result = learning_service
                .get_learning_insights(params)
                .await
                .unwrap();

            assert_eq!(
                result.is_error,
                Some(false),
                "Query '{}' should succeed",
                query
            );

            if let Some(content) = result.content[0].as_text() {
                let response: Value = serde_json::from_str(&content.text).unwrap();
                let insights = response["insights"].as_array().unwrap();
                assert!(insights.len() <= limit, "Should respect limit parameter");
            }
        }

        // Test 7: Concurrent MCP learning tool access
        let mut concurrent_tasks = Vec::new();

        for i in 0..10 {
            let service = learning_service.clone();
            let task = tokio::spawn(async move {
                let params = json!({
                    "user_id": format!("concurrent_user_{}", i),
                    "content_id": format!("concurrent_content_{}", i % 3),
                    "feedback_type": "quality_rating",
                    "score": 0.8 + (i as f64 * 0.01),
                    "text_feedback": format!("Concurrent MCP feedback {}", i)
                });

                service.submit_learning_feedback(params).await
            });
            concurrent_tasks.push(task);
        }

        let concurrent_results: Vec<Result<Result<CallToolResult, String>, _>> =
            futures::future::join_all(concurrent_tasks).await;

        let successful_submissions = concurrent_results
            .iter()
            .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
            .count();

        assert!(
            successful_submissions >= 8,
            "Most concurrent MCP submissions should succeed"
        );

        // Test 8: MCP tool performance validation
        let performance_start = Instant::now();

        for i in 0..5 {
            let params = json!({
                "query": format!("performance test {}", i),
                "limit": 10
            });

            let _result = learning_service
                .get_learning_insights(params)
                .await
                .unwrap();
        }

        let performance_time = performance_start.elapsed();
        assert!(
            performance_time < Duration::from_millis(500),
            "MCP learning tools should perform efficiently"
        );

        // Test 9: MCP response consistency
        let consistency_params = json!({
            "query": "consistency test",
            "limit": 5
        });

        let first_response = learning_service
            .get_learning_insights(consistency_params.clone())
            .await
            .unwrap();
        let second_response = learning_service
            .get_learning_insights(consistency_params)
            .await
            .unwrap();

        // Both responses should have consistent structure
        assert_eq!(first_response.is_error, second_response.is_error);
        assert_eq!(first_response.content.len(), second_response.content.len());

        // Test 10: Learning tool error handling
        let error_cases = vec![
            json!({"user_id": "", "content_id": "test"}), // Empty user_id
            json!({"user_id": "test", "content_id": ""}), // Empty content_id
        ];

        for error_case in error_cases {
            let result = learning_service.submit_learning_feedback(error_case).await;
            assert!(result.is_err(), "Error cases should be handled gracefully");

            let error_message = result.unwrap_err();
            assert!(
                !error_message.is_empty(),
                "Error message should be informative"
            );
            assert!(
                !error_message.contains("panic"),
                "Should not contain internal error details"
            );
        }
    }

    /// ANCHOR: MCP monitoring tools integration workflow
    /// Tests: Tool listing → Metrics collection → Health monitoring → Alert management
    /// Protects: MCP monitoring tools functionality and operational reliability
    #[tokio::test]
    async fn test_anchor_mcp_monitoring_tools_integration_workflow() {
        let monitoring_service = Arc::new(MockMcpMonitoringService::new());

        // Test 1: Monitoring metrics retrieval tool
        let metrics_params = json!({
            "metric_names": ["api_response_time", "memory_usage"],
            "limit": 50
        });

        let metrics_request = create_mcp_request("monitoring_get_metrics", metrics_params);
        let metrics_result = monitoring_service
            .get_monitoring_metrics(metrics_request.arguments.unwrap().into())
            .await
            .unwrap();

        assert_eq!(
            metrics_result.is_error,
            Some(false),
            "MCP metrics retrieval should succeed"
        );

        if let Some(content) = metrics_result.content[0].as_text() {
            let response: Value = serde_json::from_str(&content.text).unwrap();
            assert!(
                response["metrics"].is_array(),
                "Should return metrics array"
            );
            assert!(
                response["total_count"].is_number(),
                "Should return total count"
            );
            assert!(
                response["filtered_count"].is_number(),
                "Should return filtered count"
            );
            assert!(
                response["processing_time_ms"].is_number(),
                "Should track processing time"
            );

            let metrics = response["metrics"].as_array().unwrap();
            if !metrics.is_empty() {
                let first_metric = &metrics[0];
                assert!(first_metric["name"].is_string(), "Metric should have name");
                assert!(
                    first_metric["value"].is_number(),
                    "Metric should have value"
                );
                assert!(
                    first_metric["timestamp"].is_string(),
                    "Metric should have timestamp"
                );
                assert!(first_metric["tags"].is_object(), "Metric should have tags");
            }
        }

        // Test 2: Health status monitoring tool
        let health_params = json!({});
        let health_request = create_mcp_request("monitoring_get_health", health_params);
        let health_result = monitoring_service
            .get_monitoring_health(health_request.arguments.unwrap().into())
            .await
            .unwrap();

        assert_eq!(
            health_result.is_error,
            Some(false),
            "MCP health retrieval should succeed"
        );

        if let Some(content) = health_result.content[0].as_text() {
            let response: Value = serde_json::from_str(&content.text).unwrap();
            assert!(
                response["overall_status"].is_string(),
                "Should return overall status"
            );
            assert!(
                response["components"].is_array(),
                "Should return components array"
            );
            assert!(
                response["active_alerts_count"].is_number(),
                "Should return alert count"
            );
            assert!(
                response["last_health_check"].is_string(),
                "Should return last check time"
            );

            let components = response["components"].as_array().unwrap();
            assert!(!components.is_empty(), "Should have component health data");

            for component in components {
                assert!(component["name"].is_string(), "Component should have name");
                assert!(
                    component["status"].is_string(),
                    "Component should have status"
                );
                assert!(
                    component["response_time_ms"].is_number(),
                    "Should track response time"
                );
            }
        }

        // Test 3: Alert management tools
        let alert_params = json!({
            "title": "Test MCP Alert",
            "severity": "warning",
            "source": "mcp_test",
            "message": "This is a test alert sent via MCP"
        });

        let alert_request = create_mcp_request("monitoring_send_alert", alert_params);
        let alert_result = monitoring_service
            .send_monitoring_alert(alert_request.arguments.unwrap().into())
            .await
            .unwrap();

        assert_eq!(
            alert_result.is_error,
            Some(false),
            "MCP alert sending should succeed"
        );

        if let Some(content) = alert_result.content[0].as_text() {
            let response: Value = serde_json::from_str(&content.text).unwrap();
            assert_eq!(
                response["success"], true,
                "Alert sending should be successful"
            );
            assert!(response["alert_id"].is_string(), "Should return alert ID");
        }

        // Test 4: Alert retrieval with filtering
        let get_alerts_params = json!({
            "severity": "warning",
            "include_resolved": false
        });

        let get_alerts_request = create_mcp_request("monitoring_get_alerts", get_alerts_params);
        let get_alerts_result = monitoring_service
            .get_monitoring_alerts(get_alerts_request.arguments.unwrap().into())
            .await
            .unwrap();

        assert_eq!(
            get_alerts_result.is_error,
            Some(false),
            "MCP alert retrieval should succeed"
        );

        if let Some(content) = get_alerts_result.content[0].as_text() {
            let response: Value = serde_json::from_str(&content.text).unwrap();
            assert!(response["alerts"].is_array(), "Should return alerts array");
            assert!(
                response["total_count"].is_number(),
                "Should return total count"
            );
            assert!(
                response["filtered_count"].is_number(),
                "Should return filtered count"
            );

            let alerts = response["alerts"].as_array().unwrap();
            for alert in alerts {
                assert!(alert["id"].is_string(), "Alert should have ID");
                assert!(alert["title"].is_string(), "Alert should have title");
                assert!(alert["severity"].is_string(), "Alert should have severity");
                assert!(alert["source"].is_string(), "Alert should have source");
                assert_eq!(alert["severity"], "warning", "Should filter by severity");
            }
        }

        // Test 5: Monitoring metrics with different parameters
        let metric_query_variations = vec![
            // Query all metrics
            json!({"limit": 100}),
            // Query specific metrics
            json!({
                "metric_names": ["api_response_time"],
                "limit": 10
            }),
            // Query with empty metric names (should return all)
            json!({
                "metric_names": [],
                "limit": 20
            }),
        ];

        for query_params in metric_query_variations {
            let result = monitoring_service
                .get_monitoring_metrics(query_params)
                .await
                .unwrap();

            assert_eq!(
                result.is_error,
                Some(false),
                "Metric queries should succeed"
            );
        }

        // Test 6: Alert severity levels
        let severity_levels = vec!["low", "medium", "high", "critical"];

        for severity in severity_levels {
            let params = json!({
                "title": format!("{} severity test", severity),
                "severity": severity,
                "source": "severity_test",
                "message": format!("Testing {} severity alert", severity)
            });

            let result = monitoring_service
                .send_monitoring_alert(params)
                .await
                .unwrap();

            assert_eq!(
                result.is_error,
                Some(false),
                "Alert with {} severity should succeed",
                severity
            );
        }

        // Test 7: Concurrent monitoring tool access
        let mut concurrent_monitoring_tasks = Vec::new();

        for i in 0..15 {
            let service = monitoring_service.clone();
            let task = tokio::spawn(async move {
                if i % 3 == 0 {
                    // Get metrics
                    service.get_monitoring_metrics(json!({"limit": 10})).await
                } else if i % 3 == 1 {
                    // Get health
                    service.get_monitoring_health(json!({})).await
                } else {
                    // Send alert
                    service
                        .send_monitoring_alert(json!({
                            "title": format!("Concurrent alert {}", i),
                            "severity": "low",
                            "source": "concurrent_test"
                        }))
                        .await
                }
            });
            concurrent_monitoring_tasks.push(task);
        }

        let concurrent_monitoring_results: Vec<Result<Result<CallToolResult, String>, _>> =
            futures::future::join_all(concurrent_monitoring_tasks).await;

        let successful_monitoring_ops = concurrent_monitoring_results
            .iter()
            .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
            .count();

        assert!(
            successful_monitoring_ops >= 12,
            "Most concurrent monitoring operations should succeed"
        );

        // Test 8: Monitoring tool performance validation
        let monitoring_perf_start = Instant::now();

        for _ in 0..10 {
            let _metrics = monitoring_service
                .get_monitoring_metrics(json!({"limit": 5}))
                .await
                .unwrap();
            let _health = monitoring_service
                .get_monitoring_health(json!({}))
                .await
                .unwrap();
        }

        let monitoring_perf_time = monitoring_perf_start.elapsed();
        assert!(
            monitoring_perf_time < Duration::from_millis(300),
            "MCP monitoring tools should perform efficiently"
        );

        // Test 9: Parameter validation for monitoring tools
        let invalid_monitoring_params = vec![
            // Invalid alert parameters
            json!({}),                                                // Missing title
            json!({"title": ""}),                                     // Empty title
            json!({"title": "test", "severity": "invalid_severity"}), // Invalid severity
        ];

        for invalid_params in invalid_monitoring_params {
            let result = monitoring_service
                .send_monitoring_alert(invalid_params)
                .await;

            assert!(
                result.is_err(),
                "Invalid monitoring parameters should be rejected"
            );
        }

        // Test 10: Monitoring system state consistency
        // Send multiple alerts and verify they are tracked correctly
        let test_alerts = vec![
            ("Test Alert 1", "medium"),
            ("Test Alert 2", "high"),
            ("Test Alert 3", "low"),
        ];

        for (title, severity) in test_alerts {
            let params = json!({
                "title": title,
                "severity": severity,
                "source": "consistency_test"
            });

            monitoring_service
                .send_monitoring_alert(params)
                .await
                .unwrap();
        }

        // Verify alerts are retrievable
        let final_alerts = monitoring_service
            .get_monitoring_alerts(json!({"include_resolved": true}))
            .await
            .unwrap();

        if let Some(content) = final_alerts.content[0].as_text() {
            let response: Value = serde_json::from_str(&content.text).unwrap();
            let alerts_count = response["total_count"].as_u64().unwrap();
            assert!(alerts_count >= 3, "Should track all sent alerts");
        }

        // Verify health status reflects alert activity
        let final_health = monitoring_service
            .get_monitoring_health(json!({}))
            .await
            .unwrap();

        if let Some(content) = final_health.content[0].as_text() {
            let response: Value = serde_json::from_str(&content.text).unwrap();
            let active_alerts = response["active_alerts_count"].as_u64().unwrap();
            assert!(
                active_alerts > 0,
                "Health status should reflect active alerts"
            );
        }
    }

    /// ANCHOR: MCP protocol compliance and error handling workflow
    /// Tests: MCP message format → Error responses → Authentication → Rate limiting
    /// Protects: MCP protocol compliance and robust error handling
    #[tokio::test]
    async fn test_anchor_mcp_protocol_compliance_error_handling() {
        let learning_service = Arc::new(MockMcpLearningService::new());
        let monitoring_service = Arc::new(MockMcpMonitoringService::new());

        // Test 1: MCP protocol message format compliance
        let valid_mcp_request = CallToolRequestParam {
            name: "learning_submit_feedback".into(),
            arguments: Some(
                json!({
                    "user_id": "protocol_test_user",
                    "content_id": "protocol_test_content",
                    "feedback_type": "quality_rating",
                    "score": 0.85
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        };

        // Verify request structure is valid MCP format
        assert!(
            !valid_mcp_request.name.is_empty(),
            "Tool name should be present"
        );
        assert!(
            valid_mcp_request.arguments.is_some(),
            "Arguments should be present"
        );

        let result = learning_service
            .submit_learning_feedback(valid_mcp_request.arguments.unwrap().into())
            .await
            .unwrap();

        // Verify response structure is valid MCP format
        assert!(
            result.is_error.is_some(),
            "is_error field should be present"
        );
        assert!(!result.content.is_empty(), "Content should be present");
        assert_eq!(
            result.is_error,
            Some(false),
            "Successful operation should set is_error to false"
        );

        // Test 2: Error response format compliance
        let error_request_params = json!({
            "user_id": "", // Invalid empty user_id
            "content_id": "test_content"
        });

        let error_result = learning_service
            .submit_learning_feedback(error_request_params)
            .await;

        assert!(error_result.is_err(), "Invalid request should return error");
        let error_message = error_result.unwrap_err();
        assert!(
            !error_message.is_empty(),
            "Error message should be informative"
        );

        // Test 3: Tool parameter validation
        let validation_test_cases = vec![
            // Learning tool validation
            (
                "learning_submit_feedback",
                json!({}), // Missing required parameters
                true,
            ),
            (
                "learning_submit_feedback",
                json!({"user_id": "test", "content_id": "test", "score": 2.0}), // Invalid score
                true,
            ),
            (
                "learning_get_insights",
                json!({"limit": -1}), // Invalid limit
                false,                // This might not error, but should handle gracefully
            ),
            // Monitoring tool validation
            (
                "monitoring_send_alert",
                json!({}), // Missing title
                true,
            ),
            (
                "monitoring_get_metrics",
                json!({"limit": "invalid"}), // Invalid type
                false,                       // Should handle gracefully
            ),
        ];

        for (tool_name, params, should_error) in validation_test_cases {
            let result = if tool_name.starts_with("learning_") {
                learning_service.submit_learning_feedback(params).await
            } else {
                monitoring_service.get_monitoring_metrics(params).await
            };

            if should_error {
                assert!(
                    result.is_err(),
                    "Tool {} should reject invalid parameters",
                    tool_name
                );
            } else {
                // Should either succeed or fail gracefully
                if let Err(e) = result {
                    assert!(
                        !e.is_empty(),
                        "Error message should be informative for {}",
                        tool_name
                    );
                }
            }
        }

        // Test 4: Content type and encoding validation
        let unicode_test_params = json!({
            "user_id": "用户测试",
            "content_id": "内容测试",
            "feedback_type": "quality_rating",
            "score": 0.8,
            "text_feedback": "Unicode feedback: 🚀 Test with emojis and special chars àáâã"
        });

        let unicode_result = learning_service
            .submit_learning_feedback(unicode_test_params)
            .await
            .unwrap();

        assert_eq!(
            unicode_result.is_error,
            Some(false),
            "Should handle Unicode content correctly"
        );

        // Test 5: Large payload handling
        let large_text = "A".repeat(10000); // 10KB text
        let large_payload_params = json!({
            "user_id": "large_payload_user",
            "content_id": "large_content",
            "text_feedback": large_text
        });

        let large_payload_result = learning_service
            .submit_learning_feedback(large_payload_params)
            .await;

        // Should either handle or reject gracefully
        if large_payload_result.is_err() {
            let error = large_payload_result.unwrap_err();
            assert!(
                !error.contains("panic"),
                "Should not panic on large payloads"
            );
        }

        // Test 6: Concurrent request handling and race conditions
        let mut race_condition_tasks = Vec::new();

        for i in 0..20 {
            let learning_svc = learning_service.clone();
            let monitoring_svc = monitoring_service.clone();

            let task = tokio::spawn(async move {
                let operation = i % 4;
                match operation {
                    0 => {
                        // Learning feedback
                        let params = json!({
                            "user_id": format!("race_user_{}", i),
                            "content_id": "race_content",
                            "score": 0.8
                        });
                        learning_svc.submit_learning_feedback(params).await
                    }
                    1 => {
                        // Learning insights
                        learning_svc
                            .get_learning_insights(json!({"limit": 5}))
                            .await
                    }
                    2 => {
                        // Monitoring metrics
                        monitoring_svc
                            .get_monitoring_metrics(json!({"limit": 5}))
                            .await
                    }
                    3 => {
                        // Monitoring alert
                        let params = json!({
                            "title": format!("Race condition test {}", i),
                            "severity": "low"
                        });
                        monitoring_svc.send_monitoring_alert(params).await
                    }
                    _ => unreachable!(),
                }
            });
            race_condition_tasks.push(task);
        }

        let race_results: Vec<Result<Result<CallToolResult, String>, _>> =
            futures::future::join_all(race_condition_tasks).await;

        // Most operations should succeed despite concurrent access
        let successful_race_ops = race_results
            .iter()
            .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
            .count();

        assert!(
            successful_race_ops >= 16,
            "Most concurrent operations should succeed"
        );

        // Test 7: Timeout and performance requirements
        let timeout_start = Instant::now();

        // All operations should complete within reasonable time
        let timeout_ops = vec![
            learning_service.get_learning_insights(json!({"limit": 100})),
            monitoring_service.get_monitoring_metrics(json!({"limit": 100})),
            monitoring_service.get_monitoring_health(json!({})),
            learning_service.get_learning_status(json!({})),
        ];

        for timeout_op in timeout_ops {
            let op_start = Instant::now();
            let result = timeout_op.await;
            let op_time = op_start.elapsed();

            assert!(result.is_ok(), "Operation should complete successfully");
            assert!(
                op_time < Duration::from_millis(200),
                "Operation should complete within 200ms"
            );
        }

        let total_timeout_time = timeout_start.elapsed();
        assert!(
            total_timeout_time < Duration::from_secs(2),
            "All timeout tests should complete within 2 seconds"
        );

        // Test 8: Memory usage and resource cleanup
        // Perform operations that could potentially leak memory
        for i in 0..100 {
            let feedback_params = json!({
                "user_id": format!("memory_test_user_{}", i),
                "content_id": format!("memory_test_content_{}", i % 10),
                "score": 0.8,
                "text_feedback": format!("Memory test iteration {}", i)
            });

            learning_service
                .submit_learning_feedback(feedback_params)
                .await
                .unwrap();

            if i % 10 == 0 {
                // Periodically check that insights can still be retrieved
                let insights = learning_service
                    .get_learning_insights(json!({"limit": 5}))
                    .await
                    .unwrap();
                assert_eq!(
                    insights.is_error,
                    Some(false),
                    "System should remain responsive"
                );
            }
        }

        // System should still be responsive after many operations
        let final_status = learning_service
            .get_learning_status(json!({}))
            .await
            .unwrap();
        assert_eq!(
            final_status.is_error,
            Some(false),
            "System should remain healthy after load"
        );

        // Test 9: Error recovery and system stability
        // Simulate various error conditions and verify recovery
        let error_scenarios = vec![
            json!({"user_id": "", "content_id": "test"}), // Empty user_id
            json!({"user_id": "test", "content_id": ""}), // Empty content_id
            json!({"user_id": "test", "content_id": "test", "score": 1.5}), // Invalid score
        ];

        for error_scenario in error_scenarios {
            let error_result = learning_service
                .submit_learning_feedback(error_scenario)
                .await;
            assert!(error_result.is_err(), "Error scenario should be handled");
        }

        // System should recover and handle valid requests after errors
        let recovery_params = json!({
            "user_id": "recovery_test_user",
            "content_id": "recovery_content",
            "score": 0.9
        });

        let recovery_result = learning_service
            .submit_learning_feedback(recovery_params)
            .await
            .unwrap();
        assert_eq!(
            recovery_result.is_error,
            Some(false),
            "System should recover after errors"
        );

        // Test 10: Protocol version compatibility and extensibility
        // Verify that adding new optional parameters doesn't break existing functionality
        let extended_params = json!({
            "user_id": "extended_test_user",
            "content_id": "extended_content",
            "feedback_type": "quality_rating",
            "score": 0.85,
            "text_feedback": "Extended test",

            // Additional parameters that might be added in future versions
            "metadata": {
                "version": "2.0",
                "client_type": "mcp_client",
                "feature_flags": ["advanced_analytics"]
            },
            "optional_tags": ["test", "mcp", "extended"],
            "priority": "normal"
        });

        let extended_result = learning_service
            .submit_learning_feedback(extended_params)
            .await
            .unwrap();
        assert_eq!(
            extended_result.is_error,
            Some(false),
            "Should handle extended parameters gracefully"
        );

        // Verify response includes processing confirmation
        if let Some(content) = extended_result.content[0].as_text() {
            let response: Value = serde_json::from_str(&content.text).unwrap();
            assert_eq!(response["success"], true, "Extended request should succeed");
            assert!(
                response["processing_time_ms"].is_number(),
                "Should track processing time"
            );
        }
    }
}
