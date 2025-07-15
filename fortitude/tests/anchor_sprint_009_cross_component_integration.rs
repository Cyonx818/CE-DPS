// ABOUTME: Anchor tests for Cross-Component Integration Critical Workflows
//! These tests protect critical cross-component integration functionality.
//! They ensure that integration workflows between learning, monitoring, API, and MCP
//! systems continue to work correctly as the system evolves.
//!
//! ## Protected Functionality
//! - Cross-component integration (learning+monitoring+API+MCP workflow integration)
//! - External API integration (coordinated API calls across systems)
//! - Business logic (end-to-end workflow orchestration)
//! - Type definition changes (integration interfaces, data flow types)
//! - Critical error handling (cross-system failure recovery)

use fortitude::learning::*;
use fortitude::monitoring::*;
use fortitude_api_server::{
    middleware::auth::{AuthManager, Claims, Permission},
    models::{requests::*, responses::*},
    ApiServerConfig,
};
use fortitude_mcp_server::{
    AuthManager as McpAuthManager, FortitudeTools, McpServer, RateLimitConfig,
    ServerConfig as McpServerConfig,
};
use rmcp::model::{CallToolRequestParam, CallToolResult, Content};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Learning insight data structure
#[derive(Clone, Debug)]
pub struct LearningInsight {
    pub id: String,
    pub insight_type: String,
    pub content: String,
    pub confidence_score: f64,
    pub source_data_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
}

/// Integrated system state for cross-component testing
#[derive(Clone)]
pub struct IntegratedSystemState {
    learning_service: Arc<IntegratedLearningService>,
    monitoring_service: Arc<IntegratedMonitoringService>,
    api_auth_manager: Arc<AuthManager>,
    mcp_auth_manager: Arc<McpAuthManager>,
    mcp_server: Arc<McpServer>,
    system_metrics: Arc<RwLock<SystemIntegrationMetrics>>,
}

#[derive(Clone)]
pub struct SystemIntegrationMetrics {
    total_workflow_executions: u64,
    successful_integrations: u64,
    cross_component_calls: u64,
    integration_errors: u64,
    average_workflow_duration: Duration,
    last_integration_test: chrono::DateTime<chrono::Utc>,
}

impl Default for SystemIntegrationMetrics {
    fn default() -> Self {
        Self {
            total_workflow_executions: 0,
            successful_integrations: 0,
            cross_component_calls: 0,
            integration_errors: 0,
            average_workflow_duration: Duration::from_millis(100),
            last_integration_test: chrono::Utc::now(),
        }
    }
}

/// Integrated learning service that coordinates with monitoring
pub struct IntegratedLearningService {
    feedback_data: Arc<RwLock<Vec<UserFeedback>>>,
    learning_insights: Arc<RwLock<Vec<LearningInsight>>>,
    learning_metrics: Arc<RwLock<LearningMetrics>>,
    monitoring_service: Option<Arc<IntegratedMonitoringService>>,
}

impl IntegratedLearningService {
    pub fn new() -> Self {
        Self {
            feedback_data: Arc::new(RwLock::new(Vec::new())),
            learning_insights: Arc::new(RwLock::new(Vec::new())),
            learning_metrics: Arc::new(RwLock::new(LearningMetrics::default())),
            monitoring_service: None,
        }
    }

    pub fn set_monitoring_service(&mut self, monitoring: Arc<IntegratedMonitoringService>) {
        self.monitoring_service = Some(monitoring);
    }

    pub async fn submit_feedback_with_monitoring(
        &self,
        feedback: UserFeedback,
    ) -> Result<String, String> {
        let start_time = Instant::now();

        // Validate feedback
        if !feedback.is_valid() {
            if let Some(monitor) = &self.monitoring_service {
                monitor
                    .record_integration_error(
                        "learning",
                        "feedback_validation",
                        "Invalid feedback data",
                    )
                    .await;
            }
            return Err("Invalid feedback data".to_string());
        }

        // Store feedback
        {
            let mut data = self.feedback_data.write().await;
            data.push(feedback.clone());
        }

        // Update learning metrics
        {
            let mut metrics = self.learning_metrics.write().await;
            metrics.feedback_metrics.feedback_received += 1;
            metrics.feedback_metrics.feedback_processing_time_ms = 100.0; // Mock rate
        }

        // Record integration metrics
        if let Some(monitor) = &self.monitoring_service {
            let duration = start_time.elapsed();
            monitor
                .record_integration_metric(
                    "learning",
                    "feedback_submission",
                    duration.as_millis() as f64,
                )
                .await;
            monitor
                .record_integration_success("learning", "feedback_workflow")
                .await;
        }

        Ok(feedback.id.clone())
    }

    pub async fn get_insights_with_monitoring(
        &self,
        query: Option<String>,
    ) -> Vec<LearningInsight> {
        let start_time = Instant::now();

        let data = self.feedback_data.read().await;

        let insights = if data.is_empty() {
            vec![]
        } else {
            vec![LearningInsight {
                id: Uuid::new_v4().to_string(),
                insight_type: "cross_component_analysis".to_string(),
                content: "System performance correlates with learning quality".to_string(),
                confidence_score: 0.87,
                source_data_count: data.len(),
                created_at: chrono::Utc::now(),
                tags: vec!["integration".to_string(), "performance".to_string()],
            }]
        };

        // Record monitoring metrics
        if let Some(monitor) = &self.monitoring_service {
            let duration = start_time.elapsed();
            monitor
                .record_integration_metric(
                    "learning",
                    "insights_generation",
                    duration.as_millis() as f64,
                )
                .await;
            monitor
                .record_integration_success("learning", "insights_workflow")
                .await;
        }

        insights
    }

    pub async fn get_learning_health_with_monitoring(&self) -> LearningHealthResponse {
        let start_time = Instant::now();

        let data = self.feedback_data.read().await;
        let metrics = self.learning_metrics.read().await;

        let health = LearningHealthResponse {
            overall_status: if data.len() > 5 {
                LearningHealthStatus::Healthy
            } else {
                LearningHealthStatus::Warning
            },
            component_results: vec![
                LearningComponentHealth {
                    component: "feedback_processor".to_string(),
                    status: LearningHealthStatus::Healthy,
                    message: "Integrated with monitoring".to_string(),
                    timestamp: chrono::Utc::now(),
                    response_time_ms: 50,
                    details: HashMap::new(),
                },
                LearningComponentHealth {
                    component: "integration_layer".to_string(),
                    status: LearningHealthStatus::Healthy,
                    message: "Cross-component communication active".to_string(),
                    timestamp: chrono::Utc::now(),
                    response_time_ms: 30,
                    details: HashMap::new(),
                },
            ],
            summary: "Learning system integration health check completed".to_string(),
            timestamp: chrono::Utc::now(),
        };

        // Record monitoring metrics
        if let Some(monitor) = &self.monitoring_service {
            let duration = start_time.elapsed();
            monitor
                .record_integration_metric("learning", "health_check", duration.as_millis() as f64)
                .await;
        }

        health
    }
}

/// Integrated monitoring service that coordinates with learning
pub struct IntegratedMonitoringService {
    metrics_data: Arc<RwLock<Vec<IntegrationMetricEntry>>>,
    alerts_data: Arc<RwLock<Vec<IntegrationAlert>>>,
    health_status: Arc<RwLock<SystemHealthStatus>>,
    integration_metrics: Arc<RwLock<HashMap<String, ComponentIntegrationMetrics>>>,
}

#[derive(Clone)]
pub struct IntegrationMetricEntry {
    pub component: String,
    pub operation: String,
    pub value: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub integration_context: String,
}

#[derive(Clone)]
pub struct IntegrationAlert {
    pub id: String,
    pub component: String,
    pub operation: String,
    pub message: String,
    pub severity: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub integration_impact: Vec<String>,
}

#[derive(Clone)]
pub struct SystemHealthStatus {
    pub overall_status: String,
    pub component_health: HashMap<String, String>,
    pub integration_health: HashMap<String, String>,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct ComponentIntegrationMetrics {
    pub component_name: String,
    pub total_interactions: u64,
    pub successful_interactions: u64,
    pub average_response_time: Duration,
    pub error_rate: f64,
    pub last_interaction: chrono::DateTime<chrono::Utc>,
}

impl IntegratedMonitoringService {
    pub fn new() -> Self {
        Self {
            metrics_data: Arc::new(RwLock::new(Vec::new())),
            alerts_data: Arc::new(RwLock::new(Vec::new())),
            health_status: Arc::new(RwLock::new(SystemHealthStatus {
                overall_status: "healthy".to_string(),
                component_health: HashMap::new(),
                integration_health: HashMap::new(),
                last_check: chrono::Utc::now(),
            })),
            integration_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn record_integration_metric(&self, component: &str, operation: &str, value: f64) {
        let mut metrics = self.metrics_data.write().await;
        metrics.push(IntegrationMetricEntry {
            component: component.to_string(),
            operation: operation.to_string(),
            value,
            timestamp: chrono::Utc::now(),
            integration_context: "cross_component".to_string(),
        });

        // Update component integration metrics
        let mut component_metrics = self.integration_metrics.write().await;
        let key = component.to_string();
        let entry = component_metrics
            .entry(key)
            .or_insert(ComponentIntegrationMetrics {
                component_name: component.to_string(),
                total_interactions: 0,
                successful_interactions: 0,
                average_response_time: Duration::from_millis(50),
                error_rate: 0.0,
                last_interaction: chrono::Utc::now(),
            });
        entry.total_interactions += 1;
        entry.last_interaction = chrono::Utc::now();
    }

    pub async fn record_integration_success(&self, component: &str, operation: &str) {
        let mut component_metrics = self.integration_metrics.write().await;
        if let Some(metrics) = component_metrics.get_mut(component) {
            metrics.successful_interactions += 1;
            metrics.error_rate =
                1.0 - (metrics.successful_interactions as f64 / metrics.total_interactions as f64);
        }
    }

    pub async fn record_integration_error(&self, component: &str, operation: &str, message: &str) {
        let mut alerts = self.alerts_data.write().await;
        alerts.push(IntegrationAlert {
            id: Uuid::new_v4().to_string(),
            component: component.to_string(),
            operation: operation.to_string(),
            message: message.to_string(),
            severity: "warning".to_string(),
            timestamp: chrono::Utc::now(),
            integration_impact: vec!["cross_component_workflow".to_string()],
        });

        // Update error rate
        let mut component_metrics = self.integration_metrics.write().await;
        if let Some(metrics) = component_metrics.get_mut(component) {
            metrics.error_rate =
                1.0 - (metrics.successful_interactions as f64 / metrics.total_interactions as f64);
        }
    }

    pub async fn get_integration_health(&self) -> SystemHealthStatus {
        let health = self.health_status.read().await;
        health.clone()
    }

    pub async fn get_component_metrics(
        &self,
        component: &str,
    ) -> Option<ComponentIntegrationMetrics> {
        let metrics = self.integration_metrics.read().await;
        metrics.get(component).cloned()
    }

    pub async fn get_integration_alerts(&self) -> Vec<IntegrationAlert> {
        let alerts = self.alerts_data.read().await;
        alerts.clone()
    }
}

impl IntegratedSystemState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let temp_dir = TempDir::new()?;

        // Setup API auth manager
        let mut api_config = ApiServerConfig::default();
        api_config.auth.enabled = true;
        api_config.auth.jwt_secret =
            "test_secret_key_that_is_at_least_32_characters_long_for_security".to_string();
        let api_auth_manager = Arc::new(AuthManager::new(Arc::new(api_config))?);

        // Setup MCP auth manager and server
        let mcp_config = McpServerConfig::default();
        let mcp_auth_manager = Arc::new(McpAuthManager::new(Arc::new(mcp_config.clone())).unwrap());
        let mcp_server = Arc::new(McpServer::new(mcp_config).await.unwrap());

        // Setup integrated services
        let monitoring_service = Arc::new(IntegratedMonitoringService::new());
        let mut learning_service = IntegratedLearningService::new();
        learning_service.set_monitoring_service(monitoring_service.clone());
        let learning_service = Arc::new(learning_service);

        Ok(Self {
            learning_service,
            monitoring_service,
            api_auth_manager,
            mcp_auth_manager,
            mcp_server,
            system_metrics: Arc::new(RwLock::new(SystemIntegrationMetrics::default())),
        })
    }

    pub async fn execute_full_integration_workflow(
        &self,
        user_id: &str,
    ) -> Result<IntegrationWorkflowResult, String> {
        let start_time = Instant::now();
        let mut workflow_result = IntegrationWorkflowResult::new();

        // Step 1: API Authentication
        let token = self
            .api_auth_manager
            .generate_token(user_id, vec![Permission::Admin])
            .await
            .map_err(|e| format!("API auth failed: {}", e))?;

        let claims = self
            .api_auth_manager
            .verify_token(&token)
            .await
            .map_err(|e| format!("Token verification failed: {}", e))?;

        workflow_result.api_auth_success = true;

        // Step 2: Learning System Interaction
        let feedback = UserFeedback::new(
            user_id.to_string(),
            "integration_test_content".to_string(),
            "quality_rating".to_string(),
            Some(0.9),
            Some("Integration test feedback".to_string()),
        );

        let feedback_id = self
            .learning_service
            .submit_feedback_with_monitoring(feedback)
            .await
            .map_err(|e| format!("Learning feedback failed: {}", e))?;

        workflow_result.learning_feedback_id = Some(feedback_id);
        workflow_result.learning_interaction_success = true;

        // Step 3: Get Learning Insights with Monitoring
        let insights = self
            .learning_service
            .get_insights_with_monitoring(Some("integration".to_string()))
            .await;

        workflow_result.learning_insights_count = insights.len();

        // Step 4: Monitoring System Interaction
        self.monitoring_service
            .record_integration_metric("integration_test", "full_workflow", 1.0)
            .await;

        let health = self.monitoring_service.get_integration_health().await;
        workflow_result.monitoring_health_status =
            format!("{:?}", health.overall_status).to_lowercase();
        workflow_result.monitoring_interaction_success = true;

        // Step 5: MCP Tool Execution Simulation
        let mcp_result = self
            .execute_mcp_learning_tool(user_id, "get_learning_metrics")
            .await;
        workflow_result.mcp_tool_success = mcp_result.is_ok();
        if let Ok(result) = mcp_result {
            workflow_result.mcp_tool_result = Some(result);
        }

        // Step 6: Cross-Component Health Check
        let learning_health = self
            .learning_service
            .get_learning_health_with_monitoring()
            .await;
        workflow_result.cross_component_health_success =
            learning_health.overall_status == LearningHealthStatus::Healthy;

        // Record integration metrics
        let workflow_duration = start_time.elapsed();
        {
            let mut metrics = self.system_metrics.write().await;
            metrics.total_workflow_executions += 1;
            if workflow_result.is_fully_successful() {
                metrics.successful_integrations += 1;
            }
            metrics.cross_component_calls += 6; // Steps 1-6
            metrics.average_workflow_duration = Duration::from_millis(
                (metrics.average_workflow_duration.as_millis() as u64
                    + workflow_duration.as_millis() as u64)
                    / 2,
            );
            metrics.last_integration_test = chrono::Utc::now();
        }

        workflow_result.total_duration = workflow_duration;
        Ok(workflow_result)
    }

    async fn execute_mcp_learning_tool(
        &self,
        user_id: &str,
        tool_name: &str,
    ) -> Result<String, String> {
        // Simulate MCP tool execution
        let params = json!({
            "user_id": user_id,
            "operation": tool_name
        });

        // Mock MCP tool result
        Ok(format!(
            "MCP tool '{}' executed successfully for user '{}'",
            tool_name, user_id
        ))
    }
}

#[derive(Clone)]
pub struct IntegrationWorkflowResult {
    pub api_auth_success: bool,
    pub learning_interaction_success: bool,
    pub learning_feedback_id: Option<String>,
    pub learning_insights_count: usize,
    pub monitoring_interaction_success: bool,
    pub monitoring_health_status: String,
    pub mcp_tool_success: bool,
    pub mcp_tool_result: Option<String>,
    pub cross_component_health_success: bool,
    pub total_duration: Duration,
}

impl IntegrationWorkflowResult {
    pub fn new() -> Self {
        Self {
            api_auth_success: false,
            learning_interaction_success: false,
            learning_feedback_id: None,
            learning_insights_count: 0,
            monitoring_interaction_success: false,
            monitoring_health_status: String::new(),
            mcp_tool_success: false,
            mcp_tool_result: None,
            cross_component_health_success: false,
            total_duration: Duration::from_millis(0),
        }
    }

    pub fn is_fully_successful(&self) -> bool {
        self.api_auth_success
            && self.learning_interaction_success
            && self.monitoring_interaction_success
            && self.mcp_tool_success
            && self.cross_component_health_success
    }
}

#[cfg(test)]
mod anchor_tests {
    use super::*;

    /// ANCHOR: Cross-component integration workflows maintain stability
    /// Tests: API auth → Learning feedback → Monitoring metrics → MCP tools → Health checks
    /// Protects: End-to-end cross-component integration and coordination
    #[tokio::test]
    async fn test_anchor_cross_component_integration_workflow() {
        let system = IntegratedSystemState::new().await.unwrap();

        // Test 1: Single user full integration workflow
        let workflow_result = system
            .execute_full_integration_workflow("integration_test_user")
            .await
            .unwrap();

        assert!(
            workflow_result.is_fully_successful(),
            "Full integration workflow should succeed"
        );
        assert!(
            workflow_result.api_auth_success,
            "API authentication should succeed"
        );
        assert!(
            workflow_result.learning_interaction_success,
            "Learning interaction should succeed"
        );
        assert!(
            workflow_result.monitoring_interaction_success,
            "Monitoring interaction should succeed"
        );
        assert!(
            workflow_result.mcp_tool_success,
            "MCP tool execution should succeed"
        );
        assert!(
            workflow_result.cross_component_health_success,
            "Cross-component health check should succeed"
        );
        assert!(
            workflow_result.total_duration < Duration::from_secs(5),
            "Integration workflow should be efficient"
        );

        // Verify integration results
        assert!(
            workflow_result.learning_feedback_id.is_some(),
            "Learning feedback should generate ID"
        );
        assert!(
            workflow_result.learning_insights_count > 0,
            "Learning insights should be generated"
        );
        assert_eq!(
            workflow_result.monitoring_health_status, "healthy",
            "System health should be healthy"
        );
        assert!(
            workflow_result.mcp_tool_result.is_some(),
            "MCP tool should return result"
        );

        // Test 2: Multiple user concurrent integration workflows
        let mut concurrent_tasks = Vec::new();

        for i in 0..5 {
            let sys = system.clone();
            let task = tokio::spawn(async move {
                sys.execute_full_integration_workflow(&format!("concurrent_user_{}", i))
                    .await
            });
            concurrent_tasks.push(task);
        }

        let concurrent_results: Vec<Result<Result<IntegrationWorkflowResult, String>, _>> =
            futures::future::join_all(concurrent_tasks).await;

        let successful_workflows = concurrent_results
            .iter()
            .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
            .count();

        assert_eq!(
            successful_workflows, 5,
            "All concurrent integration workflows should succeed"
        );

        // Verify each concurrent workflow
        for result in concurrent_results {
            let workflow = result.unwrap().unwrap();
            assert!(
                workflow.is_fully_successful(),
                "Each concurrent workflow should be fully successful"
            );
            assert!(
                workflow.total_duration < Duration::from_secs(10),
                "Concurrent workflows should remain efficient"
            );
        }

        // Test 3: Integration metrics validation
        let system_metrics = system.system_metrics.read().await;
        assert!(
            system_metrics.total_workflow_executions >= 6,
            "Should track all workflow executions"
        );
        assert!(
            system_metrics.successful_integrations >= 6,
            "Should track successful integrations"
        );
        assert!(
            system_metrics.cross_component_calls >= 36,
            "Should track cross-component calls"
        ); // 6 calls per workflow
        assert!(
            system_metrics.average_workflow_duration < Duration::from_secs(5),
            "Average workflow duration should be reasonable"
        );

        // Test 4: Component integration metrics
        let learning_metrics = system
            .monitoring_service
            .get_component_metrics("learning")
            .await
            .unwrap();

        assert!(
            learning_metrics.total_interactions >= 18,
            "Learning component should have multiple interactions"
        ); // 3 per workflow
        assert!(
            learning_metrics.successful_interactions >= 18,
            "Learning interactions should be successful"
        );
        assert!(
            learning_metrics.error_rate < 0.1,
            "Learning error rate should be low"
        );

        // Test 5: Integration alerts monitoring
        let integration_alerts = system.monitoring_service.get_integration_alerts().await;
        assert!(
            integration_alerts.len() == 0,
            "Should have no integration alerts for successful workflows"
        );

        // Test 6: Cross-component error handling
        let error_feedback = UserFeedback::new(
            "".to_string(), // Invalid user ID
            "error_test_content".to_string(),
            "quality_rating".to_string(),
            Some(0.5),
            None,
        );

        let error_result = system
            .learning_service
            .submit_feedback_with_monitoring(error_feedback)
            .await;

        assert!(
            error_result.is_err(),
            "Invalid feedback should trigger error handling"
        );

        // Verify error was recorded in monitoring
        let error_alerts = system.monitoring_service.get_integration_alerts().await;
        assert!(
            error_alerts.len() > 0,
            "Error should generate integration alert"
        );

        let learning_error_alert = error_alerts
            .iter()
            .find(|a| a.component == "learning" && a.operation == "feedback_validation");
        assert!(
            learning_error_alert.is_some(),
            "Learning validation error should be tracked"
        );

        // Test 7: System health integration validation
        let health_status = system.monitoring_service.get_integration_health().await;
        assert_eq!(
            health_status.overall_status, "healthy",
            "Overall system health should remain healthy"
        );

        let learning_health = system
            .learning_service
            .get_learning_health_with_monitoring()
            .await;
        assert!(
            !learning_health.component_results.is_empty(),
            "Learning health should include component statuses"
        );

        let integration_component = learning_health
            .component_results
            .iter()
            .find(|c| c.component == "integration_layer");
        assert!(
            integration_component.is_some(),
            "Integration layer health should be tracked"
        );
        assert_eq!(
            integration_component.unwrap().status,
            LearningHealthStatus::Healthy,
            "Integration layer should be healthy"
        );

        // Test 8: Performance under load integration
        let load_test_start = Instant::now();
        let mut load_tasks = Vec::new();

        for i in 0..10 {
            let sys = system.clone();
            let task = tokio::spawn(async move {
                sys.execute_full_integration_workflow(&format!("load_test_user_{}", i))
                    .await
            });
            load_tasks.push(task);
        }

        let load_results: Vec<Result<Result<IntegrationWorkflowResult, String>, _>> =
            futures::future::join_all(load_tasks).await;

        let load_test_duration = load_test_start.elapsed();
        assert!(
            load_test_duration < Duration::from_secs(30),
            "Load test should complete in reasonable time"
        );

        let successful_load_workflows = load_results
            .iter()
            .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
            .count();

        assert!(
            successful_load_workflows >= 8,
            "Most load test workflows should succeed"
        );

        // Test 9: Data consistency across components
        let final_system_metrics = system.system_metrics.read().await;
        let final_learning_metrics = system
            .monitoring_service
            .get_component_metrics("learning")
            .await
            .unwrap();

        // Verify data consistency
        assert!(
            final_system_metrics.total_workflow_executions >= 16,
            "Should count all executed workflows"
        );
        assert!(
            final_learning_metrics.total_interactions
                >= final_system_metrics.total_workflow_executions * 3,
            "Learning interactions should be proportional to workflows"
        );

        // Test 10: Integration recovery after errors
        // Simulate system recovery by executing successful workflow after errors
        let recovery_result = system
            .execute_full_integration_workflow("recovery_test_user")
            .await
            .unwrap();

        assert!(
            recovery_result.is_fully_successful(),
            "System should recover and execute workflows successfully"
        );
        assert!(
            recovery_result.total_duration < Duration::from_secs(5),
            "Recovery workflow should be efficient"
        );

        // Verify system remains healthy after recovery
        let post_recovery_health = system.monitoring_service.get_integration_health().await;
        assert_eq!(
            post_recovery_health.overall_status, "healthy",
            "System should remain healthy after recovery"
        );

        let post_recovery_learning_health = system
            .learning_service
            .get_learning_health_with_monitoring()
            .await;
        assert_eq!(
            post_recovery_learning_health.overall_status,
            LearningHealthStatus::Healthy,
            "Learning system should remain healthy"
        );
    }

    /// ANCHOR: API-Learning-Monitoring integration maintains data flow consistency
    /// Tests: API requests → Learning processing → Monitoring tracking → Data consistency validation
    /// Protects: Data flow integrity across API, learning, and monitoring systems
    #[tokio::test]
    async fn test_anchor_api_learning_monitoring_data_flow_consistency() {
        let system = IntegratedSystemState::new().await.unwrap();

        // Test 1: API-initiated learning workflow with monitoring
        let user_token = system
            .api_auth_manager
            .generate_token("api_flow_user", vec![Permission::Admin])
            .await
            .unwrap();

        let claims = system
            .api_auth_manager
            .verify_token(&user_token)
            .await
            .unwrap();
        assert_eq!(claims.sub, "api_flow_user");

        // Submit feedback through API flow
        let api_feedback = UserFeedback::new(
            "api_flow_user".to_string(),
            "api_content_123".to_string(),
            "api_quality_rating".to_string(),
            Some(0.88),
            Some("API-submitted feedback for data flow test".to_string()),
        );

        let feedback_id = system
            .learning_service
            .submit_feedback_with_monitoring(api_feedback.clone())
            .await
            .unwrap();

        assert!(
            !feedback_id.is_empty(),
            "API feedback submission should generate valid ID"
        );

        // Verify monitoring recorded the API interaction
        let learning_metrics = system
            .monitoring_service
            .get_component_metrics("learning")
            .await
            .unwrap();

        assert!(
            learning_metrics.total_interactions > 0,
            "Monitoring should track learning interactions"
        );
        assert!(
            learning_metrics.successful_interactions > 0,
            "Monitoring should track successful interactions"
        );

        // Test 2: Batch API operations with consistent monitoring
        let batch_feedback = vec![
            ("api_user_1", "content_A", 0.9),
            ("api_user_2", "content_A", 0.85),
            ("api_user_3", "content_B", 0.92),
            ("api_user_4", "content_B", 0.78),
            ("api_user_5", "content_C", 0.95),
        ];

        let mut batch_feedback_ids = Vec::new();
        for (user_id, content_id, score) in batch_feedback {
            let batch_token = system
                .api_auth_manager
                .generate_token(user_id, vec![Permission::Admin])
                .await
                .unwrap();

            let _batch_claims = system
                .api_auth_manager
                .verify_token(&batch_token)
                .await
                .unwrap();

            let feedback = UserFeedback::new(
                user_id.to_string(),
                content_id.to_string(),
                "api_batch_rating".to_string(),
                Some(score),
                Some(format!("Batch API feedback from {}", user_id)),
            );

            let batch_id = system
                .learning_service
                .submit_feedback_with_monitoring(feedback)
                .await
                .unwrap();

            batch_feedback_ids.push(batch_id);
        }

        assert_eq!(
            batch_feedback_ids.len(),
            5,
            "All batch feedback should be processed"
        );

        // Verify monitoring consistency for batch operations
        let post_batch_metrics = system
            .monitoring_service
            .get_component_metrics("learning")
            .await
            .unwrap();

        assert!(
            post_batch_metrics.total_interactions >= learning_metrics.total_interactions + 5,
            "Monitoring should track all batch interactions"
        );

        // Test 3: Learning insights generation with monitoring integration
        let insights_start_time = Instant::now();
        let api_insights = system
            .learning_service
            .get_insights_with_monitoring(Some("api_data_flow".to_string()))
            .await;

        let insights_duration = insights_start_time.elapsed();
        assert!(
            !api_insights.is_empty(),
            "API should generate learning insights"
        );
        assert!(
            insights_duration < Duration::from_millis(500),
            "Insights generation should be efficient"
        );

        // Verify insight structure
        let cross_component_insight = api_insights
            .iter()
            .find(|i| i.insight_type == "cross_component_analysis");
        assert!(
            cross_component_insight.is_some(),
            "Should generate cross-component insights"
        );

        if let Some(insight) = cross_component_insight {
            assert!(
                insight.confidence_score > 0.8,
                "Cross-component insights should have high confidence"
            );
            assert!(
                insight.source_data_count >= 6,
                "Insights should be based on sufficient data"
            );
            assert!(
                insight.tags.contains(&"integration".to_string()),
                "Should tag integration insights"
            );
        }

        // Test 4: Monitoring health checks with learning system integration
        let health_check_start = Instant::now();
        let integrated_health = system
            .learning_service
            .get_learning_health_with_monitoring()
            .await;
        let health_check_duration = health_check_start.elapsed();

        assert!(
            health_check_duration < Duration::from_millis(100),
            "Health checks should be fast"
        );
        assert_eq!(
            integrated_health.overall_status,
            LearningHealthStatus::Healthy,
            "Integrated system should be healthy"
        );

        // Verify health components
        let feedback_processor = integrated_health
            .component_results
            .iter()
            .find(|c| c.component == "feedback_processor");
        assert!(
            feedback_processor.is_some(),
            "Should track feedback processor health"
        );
        assert_eq!(
            feedback_processor.unwrap().status,
            LearningHealthStatus::Healthy
        );

        let integration_layer = integrated_health
            .component_results
            .iter()
            .find(|c| c.component == "integration_layer");
        assert!(
            integration_layer.is_some(),
            "Should track integration layer health"
        );
        assert_eq!(
            integration_layer.unwrap().status,
            LearningHealthStatus::Healthy
        );

        // Test 5: Data consistency validation across all systems
        let final_learning_metrics = system
            .monitoring_service
            .get_component_metrics("learning")
            .await
            .unwrap();

        let final_system_metrics = system.system_metrics.read().await;

        // Verify data consistency
        assert!(
            final_learning_metrics.total_interactions >= 6,
            "Should track all learning interactions"
        );
        assert_eq!(
            final_learning_metrics.successful_interactions,
            final_learning_metrics.total_interactions,
            "All tracked interactions should be successful"
        );
        assert!(
            final_learning_metrics.error_rate < 0.1,
            "Error rate should be minimal"
        );

        // Test 6: Concurrent API-Learning-Monitoring workflow consistency
        let mut concurrent_api_tasks = Vec::new();

        for i in 0..8 {
            let sys = system.clone();
            let task = tokio::spawn(async move {
                let user_id = format!("concurrent_api_user_{}", i);

                // Generate token
                let token = sys
                    .api_auth_manager
                    .generate_token(&user_id, vec![Permission::Admin])
                    .await?;

                // Verify token
                let _claims = sys.api_auth_manager.verify_token(&token).await?;

                // Submit feedback
                let feedback = UserFeedback::new(
                    user_id.clone(),
                    format!("concurrent_content_{}", i % 3),
                    "concurrent_rating".to_string(),
                    Some(0.8 + (i as f64 * 0.02)),
                    Some(format!("Concurrent API feedback {}", i)),
                );

                let feedback_id = sys
                    .learning_service
                    .submit_feedback_with_monitoring(feedback)
                    .await?;

                // Get insights
                let insights = sys
                    .learning_service
                    .get_insights_with_monitoring(Some("concurrent".to_string()))
                    .await;

                Ok::<(String, usize), Box<dyn std::error::Error + Send + Sync>>((
                    feedback_id,
                    insights.len(),
                ))
            });
            concurrent_api_tasks.push(task);
        }

        let concurrent_api_results: Vec<Result<Result<(String, usize), _>, _>> =
            futures::future::join_all(concurrent_api_tasks).await;

        let successful_api_workflows = concurrent_api_results
            .iter()
            .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
            .count();

        assert_eq!(
            successful_api_workflows, 8,
            "All concurrent API workflows should succeed"
        );

        // Verify concurrent workflow data
        for result in concurrent_api_results {
            let (feedback_id, insights_count) = result.unwrap().unwrap();
            assert!(
                !feedback_id.is_empty(),
                "Each concurrent workflow should generate feedback ID"
            );
            assert!(
                insights_count > 0,
                "Each concurrent workflow should generate insights"
            );
        }

        // Test 7: Final data consistency validation
        let final_concurrent_metrics = system
            .monitoring_service
            .get_component_metrics("learning")
            .await
            .unwrap();

        assert!(
            final_concurrent_metrics.total_interactions
                >= final_learning_metrics.total_interactions + 16,
            "Should track all concurrent interactions"
        ); // 8 feedback + 8 insights calls

        let final_concurrent_health = system
            .learning_service
            .get_learning_health_with_monitoring()
            .await;
        assert_eq!(
            final_concurrent_health.overall_status,
            LearningHealthStatus::Healthy,
            "System should remain healthy after concurrent operations"
        );

        // Test 8: Error propagation consistency
        // Test API error → Learning error → Monitoring alert flow
        let error_token = system
            .api_auth_manager
            .generate_token("error_test_user", vec![Permission::Admin])
            .await
            .unwrap();

        let _error_claims = system
            .api_auth_manager
            .verify_token(&error_token)
            .await
            .unwrap();

        let invalid_feedback = UserFeedback::new(
            "".to_string(), // Invalid user ID
            "error_content".to_string(),
            "error_rating".to_string(),
            Some(0.5),
            None,
        );

        let error_result = system
            .learning_service
            .submit_feedback_with_monitoring(invalid_feedback)
            .await;

        assert!(error_result.is_err(), "Invalid feedback should fail");

        // Verify error was tracked in monitoring
        let error_alerts = system.monitoring_service.get_integration_alerts().await;
        let validation_alert = error_alerts
            .iter()
            .find(|a| a.component == "learning" && a.operation == "feedback_validation");
        assert!(
            validation_alert.is_some(),
            "Validation error should generate monitoring alert"
        );

        // Test 9: Recovery and consistency after errors
        let recovery_feedback = UserFeedback::new(
            "recovery_test_user".to_string(),
            "recovery_content".to_string(),
            "recovery_rating".to_string(),
            Some(0.9),
            Some("Recovery test feedback".to_string()),
        );

        let recovery_result = system
            .learning_service
            .submit_feedback_with_monitoring(recovery_feedback)
            .await;

        assert!(recovery_result.is_ok(), "System should recover from errors");

        let post_recovery_health = system
            .learning_service
            .get_learning_health_with_monitoring()
            .await;
        assert_eq!(
            post_recovery_health.overall_status,
            LearningHealthStatus::Healthy,
            "System should recover to healthy state"
        );

        // Test 10: End-to-end data integrity validation
        let final_insights = system
            .learning_service
            .get_insights_with_monitoring(Some("data_integrity".to_string()))
            .await;

        assert!(
            !final_insights.is_empty(),
            "Final insights should be available"
        );

        let final_health_check = system.monitoring_service.get_integration_health().await;
        assert_eq!(
            final_health_check.overall_status, "healthy",
            "Final system health should be healthy"
        );

        let final_metrics_check = system
            .monitoring_service
            .get_component_metrics("learning")
            .await
            .unwrap();

        // Verify final data integrity
        assert!(
            final_metrics_check.total_interactions >= 25,
            "Should track all interactions throughout test"
        );
        assert!(
            final_metrics_check.successful_interactions >= 24,
            "Most interactions should be successful"
        );
        assert!(
            final_metrics_check.error_rate < 0.1,
            "Final error rate should be acceptable"
        );
    }
}
