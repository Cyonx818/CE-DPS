// ABOUTME: Common test utilities for fortitude MCP server tests
// Provides shared mocks, fixtures, and test helpers for comprehensive testing
// Used by integration tests, performance tests, and anchor tests

use fortitude_mcp_server::{AuthManager, Permission, ServerConfig};
use rmcp::model::{CallToolRequestParam, ReadResourceRequestParam};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::RwLock;

/// Test configuration builder for easy test setup
pub struct TestConfigBuilder {
    config: ServerConfig,
}

impl TestConfigBuilder {
    pub fn new() -> Self {
        let mut config = ServerConfig::default();
        config.auth.jwt_secret =
            "test_secret_key_at_least_32_characters_long_for_testing".to_string();
        config.auth.enabled = true;
        config.auth.token_expiration_hours = 24;
        config.auth.rate_limit.max_requests_per_minute = 100;
        config.auth.rate_limit.window_seconds = 60;
        config.performance.cache_size = 100;
        config.performance.cache_ttl = 300;
        config.integration.classification_threshold = 0.1; // Lower for testing

        Self { config }
    }

    #[allow(dead_code)]
    pub fn with_auth_disabled(mut self) -> Self {
        self.config.auth.enabled = false;
        self
    }

    #[allow(dead_code)]
    pub fn with_rate_limit(mut self, max_requests: u32, window_seconds: u64) -> Self {
        self.config.auth.rate_limit.max_requests_per_minute = max_requests;
        self.config.auth.rate_limit.window_seconds = window_seconds;
        self
    }

    #[allow(dead_code)]
    pub fn with_performance_settings(mut self, cache_size: u32, cache_ttl: u64) -> Self {
        self.config.performance.cache_size = cache_size;
        self.config.performance.cache_ttl = cache_ttl;
        self
    }

    pub fn with_integration_threshold(mut self, threshold: f64) -> Self {
        self.config.integration.classification_threshold = threshold;
        self
    }

    #[allow(dead_code)]
    pub fn build(self) -> ServerConfig {
        self.config
    }

    pub fn build_arc(self) -> Arc<ServerConfig> {
        Arc::new(self.config)
    }
}

impl Default for TestConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Test environment setup with proper cleanup
pub struct TestEnvironment {
    #[allow(dead_code)]
    pub temp_dir: TempDir,
    pub config: Arc<ServerConfig>,
    pub auth_manager: Arc<AuthManager>,
    pub _storage_path: String,
}

impl TestEnvironment {
    pub async fn new() -> anyhow::Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let storage_path = temp_dir.path().join("test_storage");
        std::fs::create_dir_all(&storage_path)?;

        // Set environment variable for storage path
        std::env::set_var("FORTITUDE_STORAGE_PATH", storage_path.to_str().unwrap());

        let config = TestConfigBuilder::new()
            .with_integration_threshold(0.1) // Lower threshold for testing
            .build_arc();

        let auth_manager = Arc::new(AuthManager::new(config.clone())?);

        Ok(Self {
            temp_dir,
            config,
            auth_manager,
            _storage_path: storage_path.to_string_lossy().to_string(),
        })
    }

    #[allow(dead_code)]
    pub async fn with_auth_disabled() -> anyhow::Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let storage_path = temp_dir.path().join("test_storage");
        std::fs::create_dir_all(&storage_path)?;

        std::env::set_var("FORTITUDE_STORAGE_PATH", storage_path.to_str().unwrap());

        let config = TestConfigBuilder::new().with_auth_disabled().build_arc();

        let auth_manager = Arc::new(AuthManager::new(config.clone())?);

        Ok(Self {
            temp_dir,
            config,
            auth_manager,
            _storage_path: storage_path.to_string_lossy().to_string(),
        })
    }

    #[allow(dead_code)]
    pub async fn create_test_token(&self, permissions: Vec<Permission>) -> anyhow::Result<String> {
        self.auth_manager
            .generate_token("test_user", permissions)
            .await
    }

    #[allow(dead_code)]
    pub async fn create_admin_token(&self) -> anyhow::Result<String> {
        self.auth_manager.create_default_admin_token().await
    }
}

/// Mock MCP client for testing server interactions
pub struct MockMcpClient {
    #[allow(dead_code)]
    pub requests: Arc<RwLock<Vec<MockRequest>>>,
    #[allow(dead_code)]
    pub responses: Arc<RwLock<HashMap<String, Value>>>,
}

#[derive(Debug, Clone)]
pub struct MockRequest {
    #[allow(dead_code)]
    pub method: String,
    #[allow(dead_code)]
    pub params: Value,
    #[allow(dead_code)]
    pub timestamp: std::time::Instant,
}

impl MockMcpClient {
    pub fn new() -> Self {
        Self {
            requests: Arc::new(RwLock::new(Vec::new())),
            responses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[allow(dead_code)]
    pub async fn add_expected_response(&self, method: &str, response: Value) {
        let mut responses = self.responses.write().await;
        responses.insert(method.to_string(), response);
    }

    #[allow(dead_code)]
    pub async fn get_request_count(&self) -> usize {
        let requests = self.requests.read().await;
        requests.len()
    }

    #[allow(dead_code)]
    pub async fn get_requests(&self) -> Vec<MockRequest> {
        let requests = self.requests.read().await;
        requests.clone()
    }

    #[allow(dead_code)]
    pub async fn record_request(&self, method: &str, params: Value) {
        let mut requests = self.requests.write().await;
        requests.push(MockRequest {
            method: method.to_string(),
            params,
            timestamp: std::time::Instant::now(),
        });
    }

    #[allow(dead_code)]
    pub async fn clear_requests(&self) {
        let mut requests = self.requests.write().await;
        requests.clear();
    }
}

impl Default for MockMcpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Test data builders for consistent test inputs
pub struct TestDataBuilder;

impl TestDataBuilder {
    pub fn research_query_request(query: &str) -> CallToolRequestParam {
        let arguments = json!({
            "query": query,
            "query_type": "implementation",
            "audience": "intermediate",
            "domain": "rust"
        });

        CallToolRequestParam {
            name: "research_query".to_string().into(),
            arguments: Some(arguments.as_object().unwrap().clone()),
        }
    }

    #[allow(dead_code)]
    pub fn classify_query_request(query: &str) -> CallToolRequestParam {
        let arguments = json!({
            "query": query
        });

        CallToolRequestParam {
            name: "classify_query".to_string().into(),
            arguments: Some(arguments.as_object().unwrap().clone()),
        }
    }

    #[allow(dead_code)]
    pub fn detect_context_request(
        query: &str,
        research_type: Option<&str>,
    ) -> CallToolRequestParam {
        let mut arguments = json!({
            "query": query
        });

        if let Some(rt) = research_type {
            arguments["research_type"] = json!(rt);
        }

        CallToolRequestParam {
            name: "detect_context".to_string().into(),
            arguments: Some(arguments.as_object().unwrap().clone()),
        }
    }

    #[allow(dead_code)]
    pub fn resource_request(uri: &str) -> ReadResourceRequestParam {
        ReadResourceRequestParam {
            uri: uri.to_string(),
        }
    }

    pub fn invalid_tool_request(name: &str) -> CallToolRequestParam {
        CallToolRequestParam {
            name: name.to_string().into(),
            arguments: None,
        }
    }

    pub fn malformed_request(tool_name: &str) -> CallToolRequestParam {
        let arguments = json!({
            "invalid_param": "invalid_value"
        });

        CallToolRequestParam {
            name: tool_name.to_string().into(),
            arguments: Some(arguments.as_object().unwrap().clone()),
        }
    }

    pub fn empty_query_request() -> CallToolRequestParam {
        let arguments = json!({
            "query": ""
        });

        CallToolRequestParam {
            name: "research_query".to_string().into(),
            arguments: Some(arguments.as_object().unwrap().clone()),
        }
    }

    #[allow(dead_code)]
    pub fn oversized_query_request() -> CallToolRequestParam {
        let large_query = "a".repeat(2000); // Exceeds 1000 char limit
        let arguments = json!({
            "query": large_query
        });

        CallToolRequestParam {
            name: "research_query".to_string().into(),
            arguments: Some(arguments.as_object().unwrap().clone()),
        }
    }

    #[allow(dead_code)]
    pub fn injection_attempt_request() -> CallToolRequestParam {
        let malicious_query = "'; DROP TABLE users; --";
        let arguments = json!({
            "query": malicious_query
        });

        CallToolRequestParam {
            name: "research_query".to_string().into(),
            arguments: Some(arguments.as_object().unwrap().clone()),
        }
    }

    #[allow(dead_code)]
    pub fn xss_attempt_request() -> CallToolRequestParam {
        let malicious_query = "<script>alert('xss')</script>";
        let arguments = json!({
            "query": malicious_query
        });

        CallToolRequestParam {
            name: "research_query".to_string().into(),
            arguments: Some(arguments.as_object().unwrap().clone()),
        }
    }

    #[allow(dead_code)]
    pub fn path_traversal_resource_request() -> ReadResourceRequestParam {
        ReadResourceRequestParam {
            uri: "mcp://fortitude/docs/../../../etc/passwd".to_string(),
        }
    }
}

/// Performance test utilities
pub struct PerformanceTestHelper;

impl PerformanceTestHelper {
    #[allow(dead_code)]
    pub fn start_timer() -> std::time::Instant {
        std::time::Instant::now()
    }

    #[allow(dead_code)]
    pub fn measure_duration(start: std::time::Instant) -> std::time::Duration {
        start.elapsed()
    }

    #[allow(dead_code)]
    pub fn assert_latency_under_threshold(duration: std::time::Duration, threshold_ms: u64) {
        assert!(
            duration.as_millis() < threshold_ms as u128,
            "Latency {}ms exceeds threshold {}ms",
            duration.as_millis(),
            threshold_ms
        );
    }

    #[allow(dead_code)]
    pub async fn run_concurrent_requests<F, Fut, T>(
        request_count: usize,
        operation: F,
    ) -> Vec<anyhow::Result<T>>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = anyhow::Result<T>> + Send + 'static,
        T: Send + 'static,
    {
        let operation = Arc::new(operation);
        let mut handles = Vec::new();

        for _ in 0..request_count {
            let operation = operation.clone();
            let handle = tokio::spawn(async move { operation().await });
            handles.push(handle);
        }

        let results = futures::future::join_all(handles).await;
        results.into_iter().map(|r| r.unwrap()).collect()
    }

    #[allow(dead_code)]
    pub fn calculate_success_rate<T>(results: &[anyhow::Result<T>]) -> f64 {
        let successful = results.iter().filter(|r| r.is_ok()).count();
        successful as f64 / results.len() as f64
    }

    #[allow(dead_code)]
    pub fn calculate_average_latency(durations: &[std::time::Duration]) -> std::time::Duration {
        let total_ms: u128 = durations.iter().map(|d| d.as_millis()).sum();
        let avg_ms = total_ms / durations.len() as u128;
        std::time::Duration::from_millis(avg_ms as u64)
    }
}

/// Security test utilities
pub struct SecurityTestHelper;

impl SecurityTestHelper {
    #[allow(dead_code)]
    pub fn generate_malicious_inputs() -> Vec<String> {
        vec![
            "'; DROP TABLE users; --".to_string(),
            "<script>alert('xss')</script>".to_string(),
            "../../etc/passwd".to_string(),
            "~/.ssh/id_rsa".to_string(),
            "/etc/shadow".to_string(),
            "\\x00\\x01\\x02\\x03".to_string(),
            "SELECT * FROM information_schema.tables".to_string(),
            "<?php system($_GET['cmd']); ?>".to_string(),
            "javascript:alert('xss')".to_string(),
            "data:text/html,<script>alert('xss')</script>".to_string(),
        ]
    }

    #[allow(dead_code)]
    pub fn generate_oversized_inputs() -> Vec<String> {
        vec!["a".repeat(10000), "b".repeat(100000), "c".repeat(1000000)]
    }

    #[allow(dead_code)]
    pub fn generate_unicode_attacks() -> Vec<String> {
        vec![
            "＜script＞alert('xss')＜/script＞".to_string(),
            "\\u003cscript\\u003ealert('xss')\\u003c/script\\u003e".to_string(),
            "\\x3cscript\\x3ealert('xss')\\x3c/script\\x3e".to_string(),
        ]
    }

    #[allow(dead_code)]
    pub fn generate_path_traversal_attempts() -> Vec<String> {
        vec![
            "../../../etc/passwd".to_string(),
            "..\\..\\..\\windows\\system32\\config\\sam".to_string(),
            "....//....//....//etc//passwd".to_string(),
            "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd".to_string(),
            "..%252f..%252f..%252fetc%252fpasswd".to_string(),
            "..%c0%af..%c0%af..%c0%afetc%c0%afpasswd".to_string(),
        ]
    }
}

/// Test assertion helpers
#[allow(dead_code)]
pub struct TestAssertions;

impl TestAssertions {
    #[allow(dead_code)]
    pub fn assert_successful_response(result: &anyhow::Result<rmcp::model::CallToolResult>) {
        assert!(
            result.is_ok(),
            "Expected successful response, got error: {result:?}"
        );
        let response = result.as_ref().unwrap();
        assert_eq!(response.is_error, Some(false), "Response marked as error");
        assert!(!response.content.is_empty(), "Response content is empty");
    }

    #[allow(dead_code)]
    pub fn assert_error_response(result: &anyhow::Result<rmcp::model::CallToolResult>) {
        assert!(result.is_err(), "Expected error response, got success");
    }

    #[allow(dead_code)]
    pub fn assert_validation_error(result: &Result<rmcp::model::CallToolResult, rmcp::Error>) {
        assert!(result.is_err(), "Expected validation error");
        let error = result.as_ref().unwrap_err();
        assert!(
            error.to_string().contains("validation"),
            "Error should be validation-related: {error}"
        );
    }

    #[allow(dead_code)]
    pub fn assert_permission_error(result: &Result<rmcp::model::CallToolResult, rmcp::Error>) {
        assert!(result.is_err(), "Expected permission error");
        let error = result.as_ref().unwrap_err();
        assert!(
            error.to_string().contains("Permission"),
            "Error should be permission-related: {error}"
        );
    }

    #[allow(dead_code)]
    pub fn assert_rate_limit_error(result: &Result<rmcp::model::CallToolResult, rmcp::Error>) {
        assert!(result.is_err(), "Expected rate limit error");
        let error = result.as_ref().unwrap_err();
        assert!(
            error.to_string().contains("Rate limit"),
            "Error should be rate limit-related: {error}"
        );
    }

    #[allow(dead_code)]
    pub fn assert_resource_found(result: &anyhow::Result<rmcp::model::ReadResourceResult>) {
        assert!(
            result.is_ok(),
            "Expected resource to be found, got error: {result:?}"
        );
        let response = result.as_ref().unwrap();
        assert!(!response.contents.is_empty(), "Resource content is empty");
    }

    #[allow(dead_code)]
    pub fn assert_resource_not_found(result: &anyhow::Result<rmcp::model::ReadResourceResult>) {
        assert!(result.is_err(), "Expected resource not found error");
    }

    #[allow(dead_code)]
    pub fn assert_latency_acceptable(duration: std::time::Duration, threshold_ms: u64) {
        assert!(
            duration.as_millis() < threshold_ms as u128,
            "Latency {}ms exceeds acceptable threshold {}ms",
            duration.as_millis(),
            threshold_ms
        );
    }

    #[allow(dead_code)]
    pub fn assert_success_rate_acceptable(success_rate: f64, threshold: f64) {
        assert!(
            success_rate >= threshold,
            "Success rate {:.2}% below acceptable threshold {:.2}%",
            success_rate * 100.0,
            threshold * 100.0
        );
    }
}

/// Cleanup utilities for test resources
#[allow(dead_code)]
pub struct TestCleanup;

impl TestCleanup {
    #[allow(dead_code)]
    pub async fn cleanup_auth_manager(auth_manager: Arc<AuthManager>) {
        auth_manager.cleanup_expired_rate_limits().await;
    }

    #[allow(dead_code)]
    pub fn cleanup_env_vars() {
        std::env::remove_var("FORTITUDE_STORAGE_PATH");
        std::env::remove_var("MCP_SERVER_PORT");
        std::env::remove_var("MCP_SERVER_HOST");
        std::env::remove_var("MCP_AUTH_ENABLED");
        std::env::remove_var("MCP_JWT_SECRET");
    }
}

// Re-export commonly used types for test convenience
// Note: Exports only used when needed by specific tests
// Removed unused import: CallToolResult
