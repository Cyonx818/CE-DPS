// Copyright 2025 CE-DPS Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// ABOUTME: MCP server monitoring integration
//! # MCP Server Monitoring Integration
//!
//! This module integrates the Fortitude monitoring system with the MCP server,
//! providing comprehensive performance tracking, health monitoring, and
//! observability for Model Context Protocol operations.
//!
//! ## Features
//!
//! - **Tool Call Metrics**: Timing and success rate tracking for tool calls
//! - **Resource Access Monitoring**: Resource read/write operation tracking
//! - **Authentication Metrics**: Login attempts and token usage tracking
//! - **Error Rate Monitoring**: Automatic error detection and categorization
//! - **Performance Profiling**: Detailed performance analysis for MCP operations
//! - **Custom Metrics**: Support for MCP-specific metrics and indicators
//!
//! ## Integration
//!
//! The monitoring service integrates with the core monitoring system to provide:
//! - Real-time MCP operation dashboards
//! - Automated alerting on performance issues
//! - Tool usage analytics and trends
//! - Resource access patterns analysis

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

// Note: Using local re-implementations for now since the monitoring module
// would need to be restructured to be shared across crates
// In a full implementation, these would be in a shared crate

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for monitoring operations
pub type MonitoringResult<T> = Result<T, MonitoringError>;

/// Errors that can occur in the monitoring system
#[derive(Debug, Error, Clone)]
pub enum MonitoringError {
    #[error("Metrics collection failed: {0}")]
    MetricsError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Critical,
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub component_name: String,
    pub status: HealthStatus,
    pub message: String,
    pub last_check_time: DateTime<Utc>,
    pub checks: HashMap<String, String>,
}

/// Component performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetrics {
    pub component_name: String,
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub average_duration: Duration,
    pub p95_duration: Duration,
    pub operations_per_second: f64,
    pub last_operation_time: Option<DateTime<Utc>>,
    pub custom_metrics: HashMap<String, f64>,
}

/// Basic monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfiguration {
    pub target_response_ms: u64,
    pub warning_response_ms: u64,
    pub critical_response_ms: u64,
    pub max_metrics_in_memory: usize,
}

impl MonitoringConfiguration {
    pub fn for_mcp_server() -> Self {
        Self {
            target_response_ms: 500,
            warning_response_ms: 1000,
            critical_response_ms: 2000,
            max_metrics_in_memory: 10000,
        }
    }
}

/// Core monitoring trait for system components
#[async_trait::async_trait]
pub trait Monitorable: Send + Sync {
    fn component_name(&self) -> &str;

    async fn record_operation_metrics(
        &self,
        operation_name: &str,
        duration: Duration,
        success: bool,
        metadata: Option<HashMap<String, String>>,
    ) -> MonitoringResult<()>;

    async fn get_health_status(&self) -> MonitoringResult<ComponentHealth>;

    async fn get_performance_metrics(&self) -> MonitoringResult<ComponentMetrics>;
}

/// MCP server monitoring service
#[derive(Debug, Clone)]
pub struct McpMonitoringService {
    /// Monitoring configuration
    config: MonitoringConfiguration,

    /// MCP operation metrics
    metrics: Arc<RwLock<McpMetrics>>,

    /// Component health status
    health: Arc<RwLock<ComponentHealth>>,

    /// Custom metrics storage
    custom_metrics: Arc<RwLock<HashMap<String, f64>>>,
}

/// MCP-specific metrics
#[derive(Debug, Clone)]
pub struct McpMetrics {
    /// Total tool calls processed
    pub total_tool_calls: u64,

    /// Successful tool calls
    pub successful_tool_calls: u64,

    /// Failed tool calls
    pub failed_tool_calls: u64,

    /// Total resource reads
    pub total_resource_reads: u64,

    /// Successful resource reads
    pub successful_resource_reads: u64,

    /// Failed resource reads
    pub failed_resource_reads: u64,

    /// Authentication attempts
    pub auth_attempts: u64,

    /// Successful authentications
    pub successful_auth: u64,

    /// Failed authentications
    pub failed_auth: u64,

    /// Tool call duration samples
    pub tool_call_durations: Vec<Duration>,

    /// Resource read duration samples
    pub resource_read_durations: Vec<Duration>,

    /// Tool-specific metrics
    pub tool_metrics: HashMap<String, ToolMetrics>,

    /// Resource-specific metrics
    pub resource_metrics: HashMap<String, ResourceMetrics>,

    /// Active sessions
    pub active_sessions: u64,

    /// Peak concurrent sessions
    pub peak_sessions: u64,

    /// Operations per second
    pub operations_per_second: f64,

    /// Last update timestamp
    pub last_update: Instant,
}

/// Metrics for specific tools
#[derive(Debug, Clone)]
pub struct ToolMetrics {
    /// Tool name
    pub name: String,

    /// Total calls to this tool
    pub total_calls: u64,

    /// Successful calls
    pub successful_calls: u64,

    /// Failed calls
    pub failed_calls: u64,

    /// Average execution time
    pub avg_execution_time: Duration,

    /// 95th percentile execution time
    pub p95_execution_time: Duration,

    /// 99th percentile execution time
    pub p99_execution_time: Duration,

    /// Execution time samples
    pub execution_times: Vec<Duration>,

    /// Arguments size statistics
    pub avg_args_size: u64,
    pub total_args_size: u64,

    /// Response size statistics
    pub avg_response_size: u64,
    pub total_response_size: u64,
}

/// Metrics for specific resources
#[derive(Debug, Clone)]
pub struct ResourceMetrics {
    /// Resource URI pattern
    pub uri_pattern: String,

    /// Total reads for this resource type
    pub total_reads: u64,

    /// Successful reads
    pub successful_reads: u64,

    /// Failed reads
    pub failed_reads: u64,

    /// Average read time
    pub avg_read_time: Duration,

    /// Read time samples
    pub read_times: Vec<Duration>,

    /// Resource size statistics
    pub avg_resource_size: u64,
    pub total_bytes_read: u64,
}

/// MCP operation context for monitoring
#[derive(Debug, Clone)]
pub struct McpOperationContext {
    /// Operation start time
    pub start_time: Instant,

    /// Operation type (tool_call, resource_read, etc.)
    pub operation_type: String,

    /// Operation identifier (tool name, resource URI, etc.)
    pub operation_id: String,

    /// Session ID
    pub session_id: Option<String>,

    /// Client ID
    pub client_id: Option<String>,

    /// Custom labels
    pub labels: HashMap<String, String>,
}

impl McpMonitoringService {
    /// Create new MCP monitoring service
    pub fn new(config: MonitoringConfiguration) -> Self {
        let health = ComponentHealth {
            component_name: "mcp-server".to_string(),
            status: HealthStatus::Healthy,
            message: "MCP server operational".to_string(),
            last_check_time: chrono::Utc::now(),
            checks: HashMap::new(),
        };

        Self {
            config,
            metrics: Arc::new(RwLock::new(McpMetrics::new())),
            health: Arc::new(RwLock::new(health)),
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create monitoring service with MCP server optimized configuration
    pub fn for_mcp_server() -> Self {
        let config = MonitoringConfiguration::for_mcp_server();
        Self::new(config)
    }

    /// Record tool call metrics
    pub async fn record_tool_call(
        &self,
        tool_name: &str,
        duration: Duration,
        success: bool,
        args_size: Option<u64>,
        response_size: Option<u64>,
    ) -> MonitoringResult<()> {
        let mut metrics = self.metrics.write().await;

        // Update total tool call count
        metrics.total_tool_calls += 1;

        // Update success/failure counts
        if success {
            metrics.successful_tool_calls += 1;
        } else {
            metrics.failed_tool_calls += 1;
        }

        // Add duration sample
        metrics.tool_call_durations.push(duration);

        // Limit samples to prevent memory growth
        if metrics.tool_call_durations.len() > self.config.max_metrics_in_memory {
            metrics.tool_call_durations.remove(0);
        }

        // Update tool-specific metrics
        let tool_metrics = metrics
            .tool_metrics
            .entry(tool_name.to_string())
            .or_insert_with(|| ToolMetrics::new(tool_name.to_string()));

        tool_metrics.record_call(success, duration, args_size, response_size);

        // Update operations per second
        let now = Instant::now();
        let time_diff = now.duration_since(metrics.last_update).as_secs_f64();
        if time_diff > 0.0 {
            metrics.operations_per_second = 1.0 / time_diff;
        }
        metrics.last_update = now;

        // Check performance thresholds
        self.check_performance_thresholds(&metrics, duration, success)
            .await?;

        debug!(
            "Recorded tool call: {} -> {} in {:?}",
            tool_name,
            if success { "success" } else { "failure" },
            duration
        );

        Ok(())
    }

    /// Record resource read metrics
    pub async fn record_resource_read(
        &self,
        resource_uri: &str,
        duration: Duration,
        success: bool,
        resource_size: Option<u64>,
    ) -> MonitoringResult<()> {
        let mut metrics = self.metrics.write().await;

        // Update total resource read count
        metrics.total_resource_reads += 1;

        // Update success/failure counts
        if success {
            metrics.successful_resource_reads += 1;
        } else {
            metrics.failed_resource_reads += 1;
        }

        // Add duration sample
        metrics.resource_read_durations.push(duration);

        // Limit samples
        if metrics.resource_read_durations.len() > self.config.max_metrics_in_memory {
            metrics.resource_read_durations.remove(0);
        }

        // Extract resource type pattern from URI
        let resource_pattern = extract_resource_pattern(resource_uri);

        // Update resource-specific metrics
        let resource_metrics = metrics
            .resource_metrics
            .entry(resource_pattern.clone())
            .or_insert_with(|| ResourceMetrics::new(resource_pattern));

        resource_metrics.record_read(success, duration, resource_size);

        debug!(
            "Recorded resource read: {} -> {} in {:?}",
            resource_uri,
            if success { "success" } else { "failure" },
            duration
        );

        Ok(())
    }

    /// Record authentication attempt
    pub async fn record_auth_attempt(
        &self,
        success: bool,
        client_id: Option<&str>,
    ) -> MonitoringResult<()> {
        let mut metrics = self.metrics.write().await;

        metrics.auth_attempts += 1;

        if success {
            metrics.successful_auth += 1;
        } else {
            metrics.failed_auth += 1;
        }

        // Check for authentication abuse
        if metrics.auth_attempts > 0 {
            let auth_failure_rate =
                metrics.failed_auth as f64 / metrics.auth_attempts as f64 * 100.0;
            if auth_failure_rate > 50.0 && metrics.auth_attempts > 10 {
                warn!(
                    "High authentication failure rate detected: {:.2}% (client: {:?})",
                    auth_failure_rate, client_id
                );

                // Update health status
                let mut health = self.health.write().await;
                health.status = HealthStatus::Degraded;
                health.message =
                    format!("High authentication failure rate: {auth_failure_rate:.2}%");
            }
        }

        debug!(
            "Recorded auth attempt: {} (client: {:?})",
            if success { "success" } else { "failure" },
            client_id
        );

        Ok(())
    }

    /// Check performance thresholds and update health status
    async fn check_performance_thresholds(
        &self,
        metrics: &McpMetrics,
        duration: Duration,
        _success: bool,
    ) -> MonitoringResult<()> {
        let mut health = self.health.write().await;
        let mut health_issues = Vec::new();

        // Check response time thresholds
        let response_time_ms = duration.as_millis() as u64;
        let target_ms = self.config.target_response_ms;
        let critical_ms = self.config.critical_response_ms;

        if response_time_ms > critical_ms {
            health_issues.push(format!(
                "Response time {response_time_ms}ms exceeds critical threshold {critical_ms}ms"
            ));
            health.status = HealthStatus::Critical;
        } else if response_time_ms > target_ms && health.status == HealthStatus::Healthy {
            health.status = HealthStatus::Degraded;
        }

        // Check error rate thresholds
        let total_operations = metrics.total_tool_calls + metrics.total_resource_reads;
        if total_operations >= 10 {
            let total_errors = metrics.failed_tool_calls + metrics.failed_resource_reads;
            let error_rate = total_errors as f64 / total_operations as f64 * 100.0;

            let critical_error_rate = 2.0;
            let warning_error_rate = 1.0;

            if error_rate > critical_error_rate {
                health_issues.push(format!(
                    "Error rate {error_rate:.2}% exceeds critical threshold {critical_error_rate:.2}%"
                ));
                health.status = HealthStatus::Critical;
            } else if error_rate > warning_error_rate && health.status == HealthStatus::Healthy {
                health.status = HealthStatus::Degraded;
            }
        }

        // Update health message
        if health_issues.is_empty() {
            health.message = "MCP server operating within normal parameters".to_string();
            if health.status != HealthStatus::Critical {
                health.status = HealthStatus::Healthy;
            }
        } else {
            health.message = health_issues.join("; ");
        }

        health.last_check_time = chrono::Utc::now();

        Ok(())
    }

    /// Record custom metric
    pub async fn record_custom_metric(&self, name: &str, value: f64) -> MonitoringResult<()> {
        let mut custom_metrics = self.custom_metrics.write().await;
        custom_metrics.insert(name.to_string(), value);
        Ok(())
    }

    /// Get current MCP metrics
    pub async fn get_mcp_metrics(&self) -> McpMetrics {
        self.metrics.read().await.clone()
    }

    /// Get performance summary
    pub async fn get_performance_summary(&self) -> MonitoringResult<McpPerformanceSummary> {
        let metrics = self.metrics.read().await;
        let health = self.health.read().await;

        // Calculate tool call percentiles
        let mut tool_durations: Vec<u64> = metrics
            .tool_call_durations
            .iter()
            .map(|d| d.as_millis() as u64)
            .collect();
        tool_durations.sort_unstable();

        let tool_p50 = percentile(&tool_durations, 50.0);
        let tool_p95 = percentile(&tool_durations, 95.0);
        let tool_p99 = percentile(&tool_durations, 99.0);

        let avg_tool_duration = if !tool_durations.is_empty() {
            tool_durations.iter().sum::<u64>() / tool_durations.len() as u64
        } else {
            0
        };

        // Calculate resource read percentiles
        let mut resource_durations: Vec<u64> = metrics
            .resource_read_durations
            .iter()
            .map(|d| d.as_millis() as u64)
            .collect();
        resource_durations.sort_unstable();

        let resource_avg = if !resource_durations.is_empty() {
            resource_durations.iter().sum::<u64>() / resource_durations.len() as u64
        } else {
            0
        };

        let total_operations = metrics.total_tool_calls + metrics.total_resource_reads;
        let total_errors = metrics.failed_tool_calls + metrics.failed_resource_reads;
        let total_successful = metrics.successful_tool_calls + metrics.successful_resource_reads;

        let error_rate = if total_operations > 0 {
            total_errors as f64 / total_operations as f64 * 100.0
        } else {
            0.0
        };

        let success_rate = if total_operations > 0 {
            total_successful as f64 / total_operations as f64 * 100.0
        } else {
            100.0
        };

        let auth_success_rate = if metrics.auth_attempts > 0 {
            metrics.successful_auth as f64 / metrics.auth_attempts as f64 * 100.0
        } else {
            100.0
        };

        Ok(McpPerformanceSummary {
            total_tool_calls: metrics.total_tool_calls,
            total_resource_reads: metrics.total_resource_reads,
            total_operations,
            error_rate,
            success_rate,
            avg_tool_call_time_ms: avg_tool_duration,
            avg_resource_read_time_ms: resource_avg,
            tool_p50_ms: tool_p50,
            tool_p95_ms: tool_p95,
            tool_p99_ms: tool_p99,
            operations_per_second: metrics.operations_per_second,
            active_sessions: metrics.active_sessions,
            peak_sessions: metrics.peak_sessions,
            auth_success_rate,
            health_status: health.status.clone(),
            health_message: health.message.clone(),
            tool_count: metrics.tool_metrics.len(),
            resource_pattern_count: metrics.resource_metrics.len(),
        })
    }

    /// Update session count
    pub async fn update_session_count(&self, active: u64) -> MonitoringResult<()> {
        let mut metrics = self.metrics.write().await;
        metrics.active_sessions = active;
        if active > metrics.peak_sessions {
            metrics.peak_sessions = active;
        }
        Ok(())
    }
}

/// Performance summary for MCP operations
#[derive(Debug, Clone, serde::Serialize)]
pub struct McpPerformanceSummary {
    pub total_tool_calls: u64,
    pub total_resource_reads: u64,
    pub total_operations: u64,
    pub error_rate: f64,
    pub success_rate: f64,
    pub avg_tool_call_time_ms: u64,
    pub avg_resource_read_time_ms: u64,
    pub tool_p50_ms: u64,
    pub tool_p95_ms: u64,
    pub tool_p99_ms: u64,
    pub operations_per_second: f64,
    pub active_sessions: u64,
    pub peak_sessions: u64,
    pub auth_success_rate: f64,
    pub health_status: HealthStatus,
    pub health_message: String,
    pub tool_count: usize,
    pub resource_pattern_count: usize,
}

impl McpMetrics {
    fn new() -> Self {
        Self {
            total_tool_calls: 0,
            successful_tool_calls: 0,
            failed_tool_calls: 0,
            total_resource_reads: 0,
            successful_resource_reads: 0,
            failed_resource_reads: 0,
            auth_attempts: 0,
            successful_auth: 0,
            failed_auth: 0,
            tool_call_durations: Vec::new(),
            resource_read_durations: Vec::new(),
            tool_metrics: HashMap::new(),
            resource_metrics: HashMap::new(),
            active_sessions: 0,
            peak_sessions: 0,
            operations_per_second: 0.0,
            last_update: Instant::now(),
        }
    }
}

impl ToolMetrics {
    fn new(name: String) -> Self {
        Self {
            name,
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            avg_execution_time: Duration::default(),
            p95_execution_time: Duration::default(),
            p99_execution_time: Duration::default(),
            execution_times: Vec::new(),
            avg_args_size: 0,
            total_args_size: 0,
            avg_response_size: 0,
            total_response_size: 0,
        }
    }

    fn record_call(
        &mut self,
        success: bool,
        duration: Duration,
        args_size: Option<u64>,
        response_size: Option<u64>,
    ) {
        self.total_calls += 1;

        if success {
            self.successful_calls += 1;
        } else {
            self.failed_calls += 1;
        }

        self.execution_times.push(duration);

        // Limit samples
        if self.execution_times.len() > 1000 {
            self.execution_times.remove(0);
        }

        // Update averages
        let total_ms: u64 = self
            .execution_times
            .iter()
            .map(|d| d.as_millis() as u64)
            .sum();
        self.avg_execution_time =
            Duration::from_millis(total_ms / self.execution_times.len() as u64);

        // Calculate percentiles
        let mut sorted_times: Vec<u64> = self
            .execution_times
            .iter()
            .map(|d| d.as_millis() as u64)
            .collect();
        sorted_times.sort_unstable();
        self.p95_execution_time = Duration::from_millis(percentile(&sorted_times, 95.0));
        self.p99_execution_time = Duration::from_millis(percentile(&sorted_times, 99.0));

        // Update size statistics
        if let Some(size) = args_size {
            self.total_args_size += size;
            self.avg_args_size = self.total_args_size / self.total_calls;
        }

        if let Some(size) = response_size {
            self.total_response_size += size;
            self.avg_response_size = self.total_response_size / self.total_calls;
        }
    }
}

impl ResourceMetrics {
    fn new(uri_pattern: String) -> Self {
        Self {
            uri_pattern,
            total_reads: 0,
            successful_reads: 0,
            failed_reads: 0,
            avg_read_time: Duration::default(),
            read_times: Vec::new(),
            avg_resource_size: 0,
            total_bytes_read: 0,
        }
    }

    fn record_read(&mut self, success: bool, duration: Duration, resource_size: Option<u64>) {
        self.total_reads += 1;

        if success {
            self.successful_reads += 1;
        } else {
            self.failed_reads += 1;
        }

        self.read_times.push(duration);

        // Limit samples
        if self.read_times.len() > 1000 {
            self.read_times.remove(0);
        }

        // Update average read time
        let total_ms: u64 = self.read_times.iter().map(|d| d.as_millis() as u64).sum();
        self.avg_read_time = Duration::from_millis(total_ms / self.read_times.len() as u64);

        // Update size statistics
        if let Some(size) = resource_size {
            self.total_bytes_read += size;
            self.avg_resource_size = self.total_bytes_read / self.total_reads;
        }
    }
}

/// Extract resource pattern from URI for grouping
fn extract_resource_pattern(uri: &str) -> String {
    // Group similar resources together by extracting patterns
    if uri.contains("/config/") {
        "config".to_string()
    } else if uri.contains("/docs/") {
        "documentation".to_string()
    } else if uri.contains("/cache/") {
        "cache".to_string()
    } else if uri.contains("/research/") {
        "research".to_string()
    } else if uri.contains("/reference/") {
        "reference".to_string()
    } else {
        "other".to_string()
    }
}

/// Calculate percentile from sorted array
fn percentile(sorted_data: &[u64], percentile: f64) -> u64 {
    if sorted_data.is_empty() {
        return 0;
    }

    let index = (percentile / 100.0 * (sorted_data.len() - 1) as f64) as usize;
    sorted_data.get(index).copied().unwrap_or(0)
}

/// Implement Monitorable trait for MCP monitoring service
#[async_trait::async_trait]
impl Monitorable for McpMonitoringService {
    fn component_name(&self) -> &str {
        "mcp-server"
    }

    async fn record_operation_metrics(
        &self,
        operation_name: &str,
        duration: Duration,
        success: bool,
        metadata: Option<HashMap<String, String>>,
    ) -> MonitoringResult<()> {
        // Record custom operation metric
        self.record_custom_metric(
            &format!("{operation_name}_duration_ms"),
            duration.as_millis() as f64,
        )
        .await?;

        // Record success/failure
        self.record_custom_metric(
            &format!("{operation_name}_success"),
            if success { 1.0 } else { 0.0 },
        )
        .await?;

        // Record metadata as metrics if provided
        if let Some(metadata) = metadata {
            for (key, value) in metadata {
                if let Ok(numeric_value) = value.parse::<f64>() {
                    self.record_custom_metric(&key, numeric_value).await?;
                }
            }
        }

        Ok(())
    }

    async fn get_health_status(&self) -> MonitoringResult<ComponentHealth> {
        Ok(self.health.read().await.clone())
    }

    async fn get_performance_metrics(&self) -> MonitoringResult<ComponentMetrics> {
        let metrics = self.metrics.read().await;
        let custom_metrics = self.custom_metrics.read().await;

        // Calculate average duration across all operations
        let mut all_durations: Vec<&Duration> = Vec::new();
        all_durations.extend(&metrics.tool_call_durations);
        all_durations.extend(&metrics.resource_read_durations);

        let avg_duration = if !all_durations.is_empty() {
            let total_ms: u128 = all_durations.iter().map(|d| d.as_millis()).sum();
            Duration::from_millis((total_ms / all_durations.len() as u128) as u64)
        } else {
            Duration::default()
        };

        // Calculate 95th percentile
        let mut durations: Vec<u64> = all_durations.iter().map(|d| d.as_millis() as u64).collect();
        durations.sort_unstable();
        let p95_duration = Duration::from_millis(percentile(&durations, 95.0));

        let total_operations = metrics.total_tool_calls + metrics.total_resource_reads;
        let successful_operations =
            metrics.successful_tool_calls + metrics.successful_resource_reads;
        let failed_operations = metrics.failed_tool_calls + metrics.failed_resource_reads;

        // Calculate operations per second
        let ops_per_second = if metrics.last_update.elapsed().as_secs() > 0 {
            total_operations as f64 / metrics.last_update.elapsed().as_secs() as f64
        } else {
            0.0
        };

        Ok(ComponentMetrics {
            component_name: "mcp-server".to_string(),
            total_operations,
            successful_operations,
            failed_operations,
            average_duration: avg_duration,
            p95_duration,
            operations_per_second: ops_per_second,
            last_operation_time: Some(chrono::Utc::now()),
            custom_metrics: custom_metrics.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_monitoring_service_creation() {
        let service = McpMonitoringService::for_mcp_server();
        assert_eq!(service.component_name(), "mcp-server");

        let health = service.get_health_status().await.unwrap();
        assert_eq!(health.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_tool_call_recording() {
        let service = McpMonitoringService::for_mcp_server();

        // Record a successful tool call
        service
            .record_tool_call(
                "research_analyze",
                Duration::from_millis(250),
                true,
                Some(1024),
                Some(2048),
            )
            .await
            .unwrap();

        let metrics = service.get_mcp_metrics().await;
        assert_eq!(metrics.total_tool_calls, 1);
        assert_eq!(metrics.successful_tool_calls, 1);
        assert_eq!(metrics.failed_tool_calls, 0);

        // Check tool-specific metrics
        assert!(metrics.tool_metrics.contains_key("research_analyze"));
        let tool_metrics = &metrics.tool_metrics["research_analyze"];
        assert_eq!(tool_metrics.total_calls, 1);
        assert_eq!(tool_metrics.successful_calls, 1);
    }

    #[tokio::test]
    async fn test_resource_read_recording() {
        let service = McpMonitoringService::for_mcp_server();

        // Record a successful resource read
        service
            .record_resource_read(
                "/config/server.toml",
                Duration::from_millis(50),
                true,
                Some(4096),
            )
            .await
            .unwrap();

        let metrics = service.get_mcp_metrics().await;
        assert_eq!(metrics.total_resource_reads, 1);
        assert_eq!(metrics.successful_resource_reads, 1);
        assert_eq!(metrics.failed_resource_reads, 0);

        // Check resource-specific metrics
        assert!(metrics.resource_metrics.contains_key("config"));
    }

    #[tokio::test]
    async fn test_auth_attempt_recording() {
        let service = McpMonitoringService::for_mcp_server();

        // Record successful auth
        service
            .record_auth_attempt(true, Some("client_123"))
            .await
            .unwrap();

        // Record failed auth
        service
            .record_auth_attempt(false, Some("client_456"))
            .await
            .unwrap();

        let metrics = service.get_mcp_metrics().await;
        assert_eq!(metrics.auth_attempts, 2);
        assert_eq!(metrics.successful_auth, 1);
        assert_eq!(metrics.failed_auth, 1);
    }

    #[tokio::test]
    async fn test_performance_summary() {
        let service = McpMonitoringService::for_mcp_server();

        // Record several operations
        for i in 0..5 {
            service
                .record_tool_call(
                    "test_tool",
                    Duration::from_millis(100 + i * 50),
                    true,
                    Some(1024),
                    Some(2048),
                )
                .await
                .unwrap();
        }

        for i in 0..3 {
            service
                .record_resource_read(
                    "/docs/test.md",
                    Duration::from_millis(50 + i * 25),
                    true,
                    Some(512),
                )
                .await
                .unwrap();
        }

        let summary = service.get_performance_summary().await.unwrap();
        assert_eq!(summary.total_tool_calls, 5);
        assert_eq!(summary.total_resource_reads, 3);
        assert_eq!(summary.total_operations, 8);
        assert_eq!(summary.success_rate, 100.0);
        assert_eq!(summary.error_rate, 0.0);
        assert!(summary.avg_tool_call_time_ms > 0);
        assert!(summary.avg_resource_read_time_ms > 0);
    }

    #[test]
    fn test_resource_pattern_extraction() {
        assert_eq!(extract_resource_pattern("/config/server.toml"), "config");
        assert_eq!(extract_resource_pattern("/docs/api.md"), "documentation");
        assert_eq!(extract_resource_pattern("/cache/key123"), "cache");
        assert_eq!(extract_resource_pattern("/research/result456"), "research");
        assert_eq!(extract_resource_pattern("/reference/lib789"), "reference");
        assert_eq!(extract_resource_pattern("/unknown/path"), "other");
    }

    #[tokio::test]
    async fn test_session_tracking() {
        let service = McpMonitoringService::for_mcp_server();

        // Update session count
        service.update_session_count(10).await.unwrap();
        service.update_session_count(25).await.unwrap();
        service.update_session_count(15).await.unwrap();

        let metrics = service.get_mcp_metrics().await;
        assert_eq!(metrics.active_sessions, 15);
        assert_eq!(metrics.peak_sessions, 25);
    }

    #[tokio::test]
    async fn test_custom_metrics() {
        let service = McpMonitoringService::for_mcp_server();

        // Record custom metrics
        service
            .record_custom_metric("tool_cache_hit_rate", 0.92)
            .await
            .unwrap();
        service
            .record_custom_metric("resource_cache_size_mb", 128.0)
            .await
            .unwrap();

        let performance_metrics = service.get_performance_metrics().await.unwrap();
        assert_eq!(
            performance_metrics
                .custom_metrics
                .get("tool_cache_hit_rate"),
            Some(&0.92)
        );
        assert_eq!(
            performance_metrics
                .custom_metrics
                .get("resource_cache_size_mb"),
            Some(&128.0)
        );
    }
}
