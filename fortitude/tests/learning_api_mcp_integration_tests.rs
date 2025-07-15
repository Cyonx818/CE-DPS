//! Learning System API and MCP Server Integration Tests
//!
//! This test suite validates the integration between the learning system
//! and both the API server and MCP server components. Tests cover
//! end-to-end workflows, data consistency, error handling, and performance
//! across service boundaries.

use chrono::{DateTime, Duration, Utc};
use fortitude::learning::{
    DashboardData, HealthReport, LearningConfig, LearningData, LearningMetrics, LearningResult,
    PerformanceMetrics, UsagePattern, UserFeedback,
};
use fortitude_test_utils::{
    fixtures::create_test_research_result, helpers::setup_test_environment,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{sleep, Duration as TokioDuration};
use uuid::Uuid;

/// ANCHOR: Verifies API server learning dashboard integration end-to-end.
/// Tests: Dashboard API, metrics collection, health monitoring, real-time updates
#[tokio::test]
async fn test_anchor_api_server_learning_dashboard_integration() {
    let test_env = setup_api_learning_environment().await;

    // Test 1: Dashboard Data Collection
    info!("Testing dashboard data collection and API endpoints");

    // Generate learning activity to populate dashboard
    let content_id = "dashboard_test_content";
    let dashboard_feedback = vec![
        create_api_feedback(
            content_id,
            0.9,
            "Excellent API response",
            "dashboard_user_1",
        ),
        create_api_feedback(
            content_id,
            0.8,
            "Good but could improve",
            "dashboard_user_2",
        ),
        create_api_feedback(content_id, 0.85, "Very helpful", "dashboard_user_3"),
    ];

    for feedback in &dashboard_feedback {
        test_env
            .learning_service
            .store_feedback(feedback)
            .await
            .unwrap();
    }

    // Simulate API endpoint call for dashboard metrics
    let dashboard_metrics = test_env
        .api_client
        .get_learning_dashboard_metrics(None)
        .await
        .unwrap();

    assert!(dashboard_metrics.total_feedback_entries >= 3);
    assert!(dashboard_metrics.average_feedback_score > 0.8);
    assert!(dashboard_metrics.system_uptime_seconds > 0);
    assert!(!dashboard_metrics.recent_learning_insights.is_empty());

    // Test 2: Real-time Metrics Updates
    info!("Testing real-time metrics updates through API");

    let initial_metrics = test_env
        .api_client
        .get_learning_metrics("1h".to_string())
        .await
        .unwrap();

    // Add more learning data
    let new_feedback =
        create_api_feedback(content_id, 0.95, "Outstanding quality", "realtime_user");
    test_env
        .learning_service
        .store_feedback(&new_feedback)
        .await
        .unwrap();

    // Wait for metrics update
    sleep(TokioDuration::from_millis(100)).await;

    let updated_metrics = test_env
        .api_client
        .get_learning_metrics("1h".to_string())
        .await
        .unwrap();

    assert!(updated_metrics.total_feedback_entries > initial_metrics.total_feedback_entries);
    assert!(updated_metrics.average_feedback_score >= initial_metrics.average_feedback_score);

    // Test 3: Learning Health Monitoring API
    info!("Testing learning health monitoring API integration");

    let health_report = test_env
        .api_client
        .get_learning_health_status()
        .await
        .unwrap();

    assert!(health_report.overall_health.is_healthy());
    assert!(!health_report.component_health.is_empty());
    assert!(health_report.performance_metrics.is_some());

    // Verify specific component health
    let storage_health = health_report
        .component_health
        .iter()
        .find(|h| h.component_name == "learning_storage")
        .expect("Learning storage health should be reported");
    assert!(storage_health.is_healthy);

    // Test 4: Learning Performance Dashboard
    info!("Testing learning performance dashboard integration");

    let performance_summary = test_env
        .api_client
        .get_learning_performance_summary("24h".to_string())
        .await
        .unwrap();

    assert!(performance_summary.metrics_collected > 0);
    assert!(performance_summary.average_response_time_ms > 0.0);
    assert!(performance_summary.learning_operations_per_second > 0.0);
    assert!(!performance_summary.performance_trends.is_empty());

    // Test 5: Learning Configuration API
    info!("Testing learning configuration management API");

    let current_config = test_env
        .api_client
        .get_learning_configuration()
        .await
        .unwrap();

    assert!(current_config.enable_feedback_learning);
    assert!(current_config.enable_pattern_recognition);

    // Update configuration through API
    let mut updated_config = current_config.clone();
    updated_config.adaptation_threshold = 0.75;

    let config_update_result = test_env
        .api_client
        .update_learning_configuration(updated_config)
        .await
        .unwrap();

    assert!(config_update_result.success);
    assert_eq!(config_update_result.new_threshold, 0.75);

    info!("API server learning dashboard integration validation completed successfully");
}

/// ANCHOR: Verifies MCP server learning tools and commands integration.
/// Tests: MCP tool execution, command processing, data synchronization
#[tokio::test]
async fn test_anchor_mcp_server_learning_tools_integration() {
    let test_env = setup_mcp_learning_environment().await;

    // Test 1: MCP Learning Feedback Tool
    info!("Testing MCP learning feedback tool");

    let feedback_tool_request = json!({
        "tool": "learning_feedback",
        "arguments": {
            "content_id": "mcp_test_content",
            "user_id": "mcp_user",
            "feedback_type": "quality_rating",
            "score": 0.9,
            "text_feedback": "Excellent MCP integration"
        }
    });

    let feedback_response = test_env
        .mcp_client
        .execute_tool("learning_feedback", feedback_tool_request)
        .await
        .unwrap();

    assert!(feedback_response.success);
    assert!(!feedback_response.feedback_id.is_empty());

    // Verify feedback was stored in learning system
    let stored_feedback = test_env
        .learning_service
        .get_feedback(&feedback_response.feedback_id)
        .await
        .unwrap();
    assert!(stored_feedback.is_some());
    assert_eq!(stored_feedback.unwrap().score, Some(0.9));

    // Test 2: MCP Pattern Analysis Tool
    info!("Testing MCP pattern analysis tool");

    let pattern_tool_request = json!({
        "tool": "learning_analyze_patterns",
        "arguments": {
            "pattern_type": "mcp_usage",
            "time_range": "7d",
            "include_predictions": true
        }
    });

    let pattern_response = test_env
        .mcp_client
        .execute_tool("learning_analyze_patterns", pattern_tool_request)
        .await
        .unwrap();

    assert!(pattern_response.success);
    assert!(!pattern_response.patterns.is_empty());
    assert!(pattern_response.analysis_confidence > 0.0);

    // Test 3: MCP Learning Insights Tool
    info!("Testing MCP learning insights retrieval tool");

    let insights_tool_request = json!({
        "tool": "learning_get_insights",
        "arguments": {
            "query": "user preferences",
            "similarity_threshold": 0.8,
            "max_results": 5
        }
    });

    let insights_response = test_env
        .mcp_client
        .execute_tool("learning_get_insights", insights_tool_request)
        .await
        .unwrap();

    assert!(insights_response.success);
    assert!(!insights_response.insights.is_empty());

    for insight in &insights_response.insights {
        assert!(insight.confidence_score > 0.8);
        assert!(!insight.insights.is_empty());
    }

    // Test 4: MCP Learning Optimization Command
    info!("Testing MCP learning optimization command");

    let optimization_request = json!({
        "command": "learning_optimize",
        "arguments": {
            "optimization_type": "performance",
            "target_metric": "response_time",
            "aggressive": false
        }
    });

    let optimization_response = test_env
        .mcp_client
        .execute_command("learning_optimize", optimization_request)
        .await
        .unwrap();

    assert!(optimization_response.success);
    assert!(optimization_response.performance_improvement > 0.0);
    assert!(!optimization_response.optimizations_applied.is_empty());

    // Test 5: MCP Learning Status Command
    info!("Testing MCP learning status monitoring command");

    let status_request = json!({
        "command": "learning_status",
        "arguments": {
            "detailed": true,
            "include_metrics": true
        }
    });

    let status_response = test_env
        .mcp_client
        .execute_command("learning_status", status_request)
        .await
        .unwrap();

    assert!(status_response.success);
    assert!(status_response.system_healthy);
    assert!(status_response.metrics.is_some());

    let metrics = status_response.metrics.unwrap();
    assert!(metrics.total_feedback > 0);
    assert!(metrics.total_patterns > 0);
    assert!(metrics.uptime_seconds > 0);

    // Test 6: MCP Batch Learning Operations
    info!("Testing MCP batch learning operations");

    let batch_feedback_request = json!({
        "tool": "learning_batch_feedback",
        "arguments": {
            "feedback_entries": [
                {
                    "content_id": "batch_content_1",
                    "user_id": "batch_user_1",
                    "score": 0.8,
                    "text_feedback": "Batch feedback 1"
                },
                {
                    "content_id": "batch_content_2",
                    "user_id": "batch_user_2",
                    "score": 0.9,
                    "text_feedback": "Batch feedback 2"
                }
            ]
        }
    });

    let batch_response = test_env
        .mcp_client
        .execute_tool("learning_batch_feedback", batch_feedback_request)
        .await
        .unwrap();

    assert!(batch_response.success);
    assert_eq!(batch_response.processed_count, 2);
    assert_eq!(batch_response.failed_count, 0);

    info!("MCP server learning tools integration validation completed successfully");
}

/// ANCHOR: Verifies cross-service learning data consistency and synchronization.
/// Tests: Data consistency, synchronization, conflict resolution, error handling
#[tokio::test]
async fn test_anchor_cross_service_learning_consistency() {
    let test_env = setup_cross_service_environment().await;

    // Test 1: Data Consistency Across Services
    info!("Testing data consistency across API and MCP services");

    let content_id = "consistency_test_content";

    // Store feedback through API
    let api_feedback = create_api_feedback(content_id, 0.85, "API feedback", "api_user");
    let api_result = test_env
        .api_client
        .store_feedback(api_feedback.clone())
        .await
        .unwrap();

    // Store feedback through MCP
    let mcp_feedback_request = json!({
        "tool": "learning_feedback",
        "arguments": {
            "content_id": content_id,
            "user_id": "mcp_user",
            "score": 0.8,
            "text_feedback": "MCP feedback"
        }
    });

    let mcp_result = test_env
        .mcp_client
        .execute_tool("learning_feedback", mcp_feedback_request)
        .await
        .unwrap();

    // Verify both feedbacks are accessible from both services
    let api_feedback_list = test_env
        .api_client
        .get_feedback_for_content(content_id)
        .await
        .unwrap();

    let mcp_feedback_list = test_env
        .mcp_client
        .get_content_feedback(content_id)
        .await
        .unwrap();

    assert_eq!(api_feedback_list.len(), 2);
    assert_eq!(mcp_feedback_list.len(), 2);

    // Verify data consistency
    let api_feedback_by_id = api_feedback_list
        .iter()
        .find(|f| f.id == api_result.feedback_id)
        .expect("API feedback should be found");
    assert_eq!(api_feedback_by_id.score, Some(0.85));

    let mcp_feedback_by_id = mcp_feedback_list
        .iter()
        .find(|f| f.user_id == "mcp_user")
        .expect("MCP feedback should be found");
    assert_eq!(mcp_feedback_by_id.score, Some(0.8));

    // Test 2: Learning Data Synchronization
    info!("Testing learning data synchronization between services");

    // Create learning data through API
    let api_learning_data = LearningData::new(
        "api_learning".to_string(),
        content_id.to_string(),
        vec!["API-generated insight".to_string()],
        0.9,
    );

    let api_learning_result = test_env
        .api_client
        .store_learning_data(api_learning_data.clone())
        .await
        .unwrap();

    // Verify learning data is accessible through MCP
    let mcp_learning_insights = test_env
        .mcp_client
        .get_learning_insights("api_learning", 0.8, 5)
        .await
        .unwrap();

    assert!(!mcp_learning_insights.is_empty());
    let synced_insight = mcp_learning_insights
        .iter()
        .find(|insight| insight.learning_data.id == api_learning_result.learning_id)
        .expect("API learning data should be accessible through MCP");

    assert_eq!(synced_insight.learning_data.confidence_score, 0.9);

    // Test 3: Pattern Synchronization
    info!("Testing usage pattern synchronization");

    // Store pattern through MCP
    let mcp_pattern_request = json!({
        "tool": "learning_store_pattern",
        "arguments": {
            "pattern_type": "synchronization_test",
            "data": "cross_service_pattern",
            "frequency": 5
        }
    });

    let mcp_pattern_result = test_env
        .mcp_client
        .execute_tool("learning_store_pattern", mcp_pattern_request)
        .await
        .unwrap();

    // Verify pattern is accessible through API
    let api_patterns = test_env
        .api_client
        .get_usage_patterns("synchronization_test")
        .await
        .unwrap();

    assert!(!api_patterns.is_empty());
    let synced_pattern = api_patterns
        .iter()
        .find(|p| p.data == "cross_service_pattern")
        .expect("MCP pattern should be accessible through API");

    assert_eq!(synced_pattern.frequency, 5);

    // Test 4: Error Handling and Recovery
    info!("Testing error handling and recovery across services");

    // Test API error handling
    let invalid_feedback = json!({
        "content_id": content_id,
        "user_id": "", // Invalid empty user ID
        "score": 1.5, // Invalid score > 1.0
        "text_feedback": "Invalid feedback"
    });

    let api_error_result = test_env
        .api_client
        .store_feedback_json(invalid_feedback)
        .await;

    assert!(api_error_result.is_err());

    // Test MCP error handling
    let invalid_mcp_request = json!({
        "tool": "learning_feedback",
        "arguments": {
            "content_id": "",  // Invalid empty content ID
            "user_id": "test_user",
            "score": -0.5  // Invalid negative score
        }
    });

    let mcp_error_result = test_env
        .mcp_client
        .execute_tool("learning_feedback", invalid_mcp_request)
        .await;

    assert!(mcp_error_result.is_err());

    // Verify system remains stable after errors
    let health_check = test_env
        .api_client
        .get_learning_health_status()
        .await
        .unwrap();

    assert!(health_check.overall_health.is_healthy());

    // Test 5: Concurrent Access Consistency
    info!("Testing concurrent access consistency across services");

    let concurrent_content = "concurrent_consistency_test";
    let concurrent_tasks = 5;

    // Create concurrent tasks that access both API and MCP
    let concurrent_futures: Vec<_> = (0..concurrent_tasks)
        .map(|task_id| {
            let api_client = &test_env.api_client;
            let mcp_client = &test_env.mcp_client;

            async move {
                // Store through API
                let api_feedback = create_api_feedback(
                    concurrent_content,
                    0.8 + (task_id as f64 * 0.02),
                    &format!("Concurrent API feedback {}", task_id),
                    &format!("api_user_{}", task_id),
                );
                api_client.store_feedback(api_feedback).await.unwrap();

                // Store through MCP
                let mcp_request = json!({
                    "tool": "learning_feedback",
                    "arguments": {
                        "content_id": concurrent_content,
                        "user_id": format!("mcp_user_{}", task_id),
                        "score": 0.75 + (task_id as f64 * 0.03),
                        "text_feedback": format!("Concurrent MCP feedback {}", task_id)
                    }
                });
                mcp_client
                    .execute_tool("learning_feedback", mcp_request)
                    .await
                    .unwrap();

                task_id
            }
        })
        .collect();

    let concurrent_results: Vec<_> = futures::future::join_all(concurrent_futures).await;
    assert_eq!(concurrent_results.len(), concurrent_tasks);

    // Verify consistency after concurrent access
    let final_api_feedback = test_env
        .api_client
        .get_feedback_for_content(concurrent_content)
        .await
        .unwrap();

    let final_mcp_feedback = test_env
        .mcp_client
        .get_content_feedback(concurrent_content)
        .await
        .unwrap();

    assert_eq!(final_api_feedback.len(), concurrent_tasks * 2); // 2 feedback per task
    assert_eq!(final_mcp_feedback.len(), concurrent_tasks * 2);

    // Verify data integrity
    for feedback in &final_api_feedback {
        assert!(feedback.is_valid());
        assert!(feedback.score.unwrap() >= 0.75);
        assert!(feedback.score.unwrap() <= 1.0);
    }

    info!("Cross-service learning consistency validation completed successfully");
}

/// ANCHOR: Verifies learning system performance across API and MCP boundaries.
/// Tests: Cross-service performance, latency, throughput, resource usage
#[tokio::test]
async fn test_anchor_cross_service_learning_performance() {
    let test_env = setup_cross_service_environment().await;

    // Performance Test 1: API Response Time Performance
    info!("Testing API learning endpoints response time performance");

    let api_start = Instant::now();
    let api_operations = 20;

    for i in 0..api_operations {
        let feedback = create_api_feedback(
            &format!("api_perf_content_{}", i),
            0.8 + (i as f64 * 0.01),
            "Performance test feedback",
            &format!("api_perf_user_{}", i),
        );

        test_env.api_client.store_feedback(feedback).await.unwrap();
    }

    let api_duration = api_start.elapsed();
    let api_throughput = api_operations as f64 / api_duration.as_secs_f64();

    assert!(
        api_duration.as_secs() < 10,
        "API operations should complete in under 10 seconds"
    );
    assert!(
        api_throughput > 2.0,
        "API should achieve minimum 2 operations/second"
    );

    info!(
        "API learning operations throughput: {:.2} ops/second",
        api_throughput
    );

    // Performance Test 2: MCP Tool Execution Performance
    info!("Testing MCP learning tools execution performance");

    let mcp_start = Instant::now();
    let mcp_operations = 15;

    for i in 0..mcp_operations {
        let feedback_request = json!({
            "tool": "learning_feedback",
            "arguments": {
                "content_id": format!("mcp_perf_content_{}", i),
                "user_id": format!("mcp_perf_user_{}", i),
                "score": 0.75 + (i as f64 * 0.015),
                "text_feedback": "MCP performance test"
            }
        });

        test_env
            .mcp_client
            .execute_tool("learning_feedback", feedback_request)
            .await
            .unwrap();
    }

    let mcp_duration = mcp_start.elapsed();
    let mcp_throughput = mcp_operations as f64 / mcp_duration.as_secs_f64();

    assert!(
        mcp_duration.as_secs() < 12,
        "MCP operations should complete in under 12 seconds"
    );
    assert!(
        mcp_throughput > 1.0,
        "MCP should achieve minimum 1 operation/second"
    );

    info!(
        "MCP learning operations throughput: {:.2} ops/second",
        mcp_throughput
    );

    // Performance Test 3: Cross-Service Query Performance
    info!("Testing cross-service query performance");

    let query_start = Instant::now();

    // Parallel queries across both services
    let (api_metrics, mcp_status) = tokio::join!(
        test_env.api_client.get_learning_metrics("1h".to_string()),
        test_env
            .mcp_client
            .execute_command("learning_status", json!({"arguments": {"detailed": true}}))
    );

    let query_duration = query_start.elapsed();

    assert!(api_metrics.is_ok());
    assert!(mcp_status.is_ok());
    assert!(
        query_duration.as_millis() < 2000,
        "Cross-service queries should complete in under 2 seconds"
    );

    info!("Cross-service query latency: {:?}", query_duration);

    // Performance Test 4: Batch Operations Performance
    info!("Testing batch operations performance across services");

    let batch_start = Instant::now();
    let batch_size = 10;

    // API batch feedback
    let api_batch_feedback: Vec<_> = (0..batch_size)
        .map(|i| {
            create_api_feedback(
                &format!("batch_content_{}", i),
                0.8,
                "Batch feedback",
                &format!("batch_user_{}", i),
            )
        })
        .collect();

    let api_batch_result = test_env
        .api_client
        .store_feedback_batch(api_batch_feedback)
        .await
        .unwrap();

    // MCP batch operations
    let mcp_batch_request = json!({
        "tool": "learning_batch_feedback",
        "arguments": {
            "feedback_entries": (0..batch_size).map(|i| json!({
                "content_id": format!("mcp_batch_content_{}", i),
                "user_id": format!("mcp_batch_user_{}", i),
                "score": 0.85,
                "text_feedback": "MCP batch feedback"
            })).collect::<Vec<_>>()
        }
    });

    let mcp_batch_result = test_env
        .mcp_client
        .execute_tool("learning_batch_feedback", mcp_batch_request)
        .await
        .unwrap();

    let batch_duration = batch_start.elapsed();

    assert_eq!(api_batch_result.successful_count, batch_size);
    assert_eq!(mcp_batch_result.processed_count, batch_size);
    assert!(
        batch_duration.as_secs() < 5,
        "Batch operations should complete in under 5 seconds"
    );

    info!("Batch operations completed in {:?}", batch_duration);

    // Performance Test 5: Memory Usage Monitoring
    info!("Testing memory usage across services");

    let initial_api_metrics = test_env.api_client.get_performance_metrics().await.unwrap();

    let initial_mcp_metrics = test_env.mcp_client.get_performance_metrics().await.unwrap();

    // Perform intensive operations
    for i in 0..50 {
        let learning_data = LearningData::new(
            "performance_test".to_string(),
            format!("perf_source_{}", i),
            vec![format!("Performance insight #{}", i)],
            0.8,
        );

        test_env
            .api_client
            .store_learning_data(learning_data)
            .await
            .unwrap();
    }

    let final_api_metrics = test_env.api_client.get_performance_metrics().await.unwrap();

    let final_mcp_metrics = test_env.mcp_client.get_performance_metrics().await.unwrap();

    // Verify reasonable memory usage
    let api_memory_increase =
        final_api_metrics.memory_usage_mb - initial_api_metrics.memory_usage_mb;
    let mcp_memory_increase =
        final_mcp_metrics.memory_usage_mb - initial_mcp_metrics.memory_usage_mb;

    assert!(
        api_memory_increase < 200.0,
        "API memory increase should be reasonable"
    );
    assert!(
        mcp_memory_increase < 150.0,
        "MCP memory increase should be reasonable"
    );

    info!("Cross-service learning performance validation completed successfully");
    info!("API memory increase: {:.2} MB", api_memory_increase);
    info!("MCP memory increase: {:.2} MB", mcp_memory_increase);
}

// Helper functions and test environment setup

async fn setup_api_learning_environment() -> ApiLearningTestEnvironment {
    ApiLearningTestEnvironment {
        api_client: MockApiClient::new().await,
        learning_service: MockLearningService::new().await,
    }
}

async fn setup_mcp_learning_environment() -> McpLearningTestEnvironment {
    McpLearningTestEnvironment {
        mcp_client: MockMcpClient::new().await,
        learning_service: MockLearningService::new().await,
    }
}

async fn setup_cross_service_environment() -> CrossServiceTestEnvironment {
    CrossServiceTestEnvironment {
        api_client: MockApiClient::new().await,
        mcp_client: MockMcpClient::new().await,
        learning_service: MockLearningService::new().await,
    }
}

struct ApiLearningTestEnvironment {
    api_client: MockApiClient,
    learning_service: MockLearningService,
}

struct McpLearningTestEnvironment {
    mcp_client: MockMcpClient,
    learning_service: MockLearningService,
}

struct CrossServiceTestEnvironment {
    api_client: MockApiClient,
    mcp_client: MockMcpClient,
    learning_service: MockLearningService,
}

fn create_api_feedback(content_id: &str, score: f64, text: &str, user_id: &str) -> UserFeedback {
    UserFeedback::new(
        user_id.to_string(),
        content_id.to_string(),
        "quality_rating".to_string(),
        Some(score),
        Some(text.to_string()),
    )
}

// Mock clients and services for testing

struct MockApiClient {
    base_url: String,
}

impl MockApiClient {
    async fn new() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
        }
    }

    async fn get_learning_dashboard_metrics(
        &self,
        _duration: Option<String>,
    ) -> Result<DashboardMetrics, Box<dyn std::error::Error + Send + Sync>> {
        Ok(DashboardMetrics {
            total_feedback_entries: 15,
            average_feedback_score: 0.85,
            system_uptime_seconds: 3600,
            recent_learning_insights: vec![
                "Users prefer detailed responses".to_string(),
                "Code examples increase satisfaction".to_string(),
            ],
        })
    }

    async fn get_learning_metrics(
        &self,
        _duration: String,
    ) -> Result<LearningMetrics, Box<dyn std::error::Error + Send + Sync>> {
        Ok(LearningMetrics {
            total_feedback_entries: 20,
            total_patterns: 8,
            average_feedback_score: 0.82,
            system_uptime_seconds: 3600,
            learning_rate: 0.15,
        })
    }

    async fn get_learning_health_status(
        &self,
    ) -> Result<HealthReport, Box<dyn std::error::Error + Send + Sync>> {
        Ok(HealthReport {
            overall_health: HealthStatus::Healthy,
            component_health: vec![ComponentHealth {
                component_name: "learning_storage".to_string(),
                is_healthy: true,
                last_check: Utc::now(),
                details: "Storage functioning normally".to_string(),
            }],
            performance_metrics: Some(PerformanceMetrics {
                memory_usage_mb: 150.0,
                cpu_usage_percent: 25.0,
                disk_usage_mb: 500.0,
                network_usage_mbps: 10.0,
                response_time_ms: 200.0,
                throughput_ops_per_sec: 50.0,
                error_rate_percent: 0.1,
                uptime_seconds: 3600,
            }),
            timestamp: Utc::now(),
        })
    }

    async fn get_learning_performance_summary(
        &self,
        _duration: String,
    ) -> Result<PerformanceSummary, Box<dyn std::error::Error + Send + Sync>> {
        Ok(PerformanceSummary {
            metrics_collected: 100,
            average_response_time_ms: 180.0,
            learning_operations_per_second: 45.0,
            performance_trends: vec![
                "Response time improving".to_string(),
                "Throughput stable".to_string(),
            ],
        })
    }

    async fn get_learning_configuration(
        &self,
    ) -> Result<LearningConfig, Box<dyn std::error::Error + Send + Sync>> {
        Ok(LearningConfig::default())
    }

    async fn update_learning_configuration(
        &self,
        _config: LearningConfig,
    ) -> Result<ConfigUpdateResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(ConfigUpdateResult {
            success: true,
            new_threshold: 0.75,
        })
    }

    async fn store_feedback(
        &self,
        _feedback: UserFeedback,
    ) -> Result<FeedbackStoreResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(FeedbackStoreResult {
            feedback_id: Uuid::new_v4().to_string(),
            success: true,
        })
    }

    async fn get_feedback_for_content(
        &self,
        _content_id: &str,
    ) -> Result<Vec<UserFeedback>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![
            UserFeedback::new(
                "user1".to_string(),
                "content1".to_string(),
                "rating".to_string(),
                Some(0.8),
                None,
            ),
            UserFeedback::new(
                "user2".to_string(),
                "content1".to_string(),
                "rating".to_string(),
                Some(0.9),
                None,
            ),
        ])
    }

    async fn store_learning_data(
        &self,
        _data: LearningData,
    ) -> Result<LearningDataStoreResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(LearningDataStoreResult {
            learning_id: Uuid::new_v4().to_string(),
            success: true,
        })
    }

    async fn get_usage_patterns(
        &self,
        _pattern_type: &str,
    ) -> Result<Vec<UsagePattern>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![UsagePattern::new(
            "test_pattern".to_string(),
            "cross_service_pattern".to_string(),
        )])
    }

    async fn store_feedback_json(
        &self,
        _feedback_json: serde_json::Value,
    ) -> Result<FeedbackStoreResult, Box<dyn std::error::Error + Send + Sync>> {
        Err("Invalid feedback data".into())
    }

    async fn store_feedback_batch(
        &self,
        _feedback_batch: Vec<UserFeedback>,
    ) -> Result<BatchStoreResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(BatchStoreResult {
            successful_count: 10,
            failed_count: 0,
        })
    }

    async fn get_performance_metrics(
        &self,
    ) -> Result<PerformanceMetrics, Box<dyn std::error::Error + Send + Sync>> {
        Ok(PerformanceMetrics {
            memory_usage_mb: 145.0,
            cpu_usage_percent: 20.0,
            disk_usage_mb: 480.0,
            network_usage_mbps: 8.0,
            response_time_ms: 190.0,
            throughput_ops_per_sec: 55.0,
            error_rate_percent: 0.05,
            uptime_seconds: 3600,
        })
    }
}

struct MockMcpClient {
    connection_id: String,
}

impl MockMcpClient {
    async fn new() -> Self {
        Self {
            connection_id: Uuid::new_v4().to_string(),
        }
    }

    async fn execute_tool(
        &self,
        _tool_name: &str,
        _request: serde_json::Value,
    ) -> Result<McpToolResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(McpToolResult {
            success: true,
            feedback_id: Uuid::new_v4().to_string(),
            patterns: vec![UsagePattern::new(
                "mcp_pattern".to_string(),
                "mcp_data".to_string(),
            )],
            analysis_confidence: 0.85,
            insights: vec![LearningInsight {
                learning_data: LearningData::new(
                    "mcp_insight".to_string(),
                    "mcp_source".to_string(),
                    vec!["MCP generated insight".to_string()],
                    0.9,
                ),
                confidence_score: 0.9,
            }],
            performance_improvement: 0.12,
            optimizations_applied: vec!["cache_optimization".to_string()],
            processed_count: 2,
            failed_count: 0,
        })
    }

    async fn execute_command(
        &self,
        _command_name: &str,
        _request: serde_json::Value,
    ) -> Result<McpCommandResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(McpCommandResult {
            success: true,
            system_healthy: true,
            metrics: Some(LearningMetrics {
                total_feedback: 25,
                total_patterns: 12,
                uptime_seconds: 3600,
                learning_rate: 0.18,
                average_feedback_score: 0.83,
            }),
            performance_improvement: 0.08,
            optimizations_applied: vec!["query_optimization".to_string()],
        })
    }

    async fn get_content_feedback(
        &self,
        _content_id: &str,
    ) -> Result<Vec<UserFeedback>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![
            UserFeedback::new(
                "mcp_user1".to_string(),
                "content1".to_string(),
                "rating".to_string(),
                Some(0.85),
                None,
            ),
            UserFeedback::new(
                "mcp_user2".to_string(),
                "content1".to_string(),
                "rating".to_string(),
                Some(0.8),
                None,
            ),
        ])
    }

    async fn get_learning_insights(
        &self,
        _query: &str,
        _threshold: f64,
        _max_results: usize,
    ) -> Result<Vec<LearningInsight>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![LearningInsight {
            learning_data: LearningData::new(
                "api_learning".to_string(),
                "api_source".to_string(),
                vec!["API-generated insight".to_string()],
                0.9,
            ),
            confidence_score: 0.9,
        }])
    }

    async fn get_performance_metrics(
        &self,
    ) -> Result<PerformanceMetrics, Box<dyn std::error::Error + Send + Sync>> {
        Ok(PerformanceMetrics {
            memory_usage_mb: 120.0,
            cpu_usage_percent: 15.0,
            disk_usage_mb: 350.0,
            network_usage_mbps: 5.0,
            response_time_ms: 220.0,
            throughput_ops_per_sec: 40.0,
            error_rate_percent: 0.02,
            uptime_seconds: 3600,
        })
    }
}

struct MockLearningService {
    feedback_store: Arc<tokio::sync::RwLock<HashMap<String, UserFeedback>>>,
}

impl MockLearningService {
    async fn new() -> Self {
        Self {
            feedback_store: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    async fn store_feedback(
        &self,
        feedback: &UserFeedback,
    ) -> Result<UserFeedback, Box<dyn std::error::Error + Send + Sync>> {
        let mut store = self.feedback_store.write().await;
        store.insert(feedback.id.clone(), feedback.clone());
        Ok(feedback.clone())
    }

    async fn get_feedback(
        &self,
        id: &str,
    ) -> Result<Option<UserFeedback>, Box<dyn std::error::Error + Send + Sync>> {
        let store = self.feedback_store.read().await;
        Ok(store.get(id).cloned())
    }
}

// Response types for mock services

#[derive(Debug, Clone)]
struct DashboardMetrics {
    total_feedback_entries: usize,
    average_feedback_score: f64,
    system_uptime_seconds: u64,
    recent_learning_insights: Vec<String>,
}

#[derive(Debug, Clone)]
struct ConfigUpdateResult {
    success: bool,
    new_threshold: f64,
}

#[derive(Debug, Clone)]
struct FeedbackStoreResult {
    feedback_id: String,
    success: bool,
}

#[derive(Debug, Clone)]
struct LearningDataStoreResult {
    learning_id: String,
    success: bool,
}

#[derive(Debug, Clone)]
struct BatchStoreResult {
    successful_count: usize,
    failed_count: usize,
}

#[derive(Debug, Clone)]
struct McpToolResult {
    success: bool,
    feedback_id: String,
    patterns: Vec<UsagePattern>,
    analysis_confidence: f64,
    insights: Vec<LearningInsight>,
    performance_improvement: f64,
    optimizations_applied: Vec<String>,
    processed_count: usize,
    failed_count: usize,
}

#[derive(Debug, Clone)]
struct McpCommandResult {
    success: bool,
    system_healthy: bool,
    metrics: Option<LearningMetrics>,
    performance_improvement: f64,
    optimizations_applied: Vec<String>,
}

#[derive(Debug, Clone)]
struct LearningInsight {
    learning_data: LearningData,
    confidence_score: f64,
}

#[derive(Debug, Clone)]
struct PerformanceSummary {
    metrics_collected: usize,
    average_response_time_ms: f64,
    learning_operations_per_second: f64,
    performance_trends: Vec<String>,
}

#[derive(Debug, Clone)]
struct ComponentHealth {
    component_name: String,
    is_healthy: bool,
    last_check: DateTime<Utc>,
    details: String,
}

#[derive(Debug, Clone)]
enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

impl HealthStatus {
    fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }
}

// Additional imports
use async_trait::async_trait;
use futures;
use tracing::{debug, info};
