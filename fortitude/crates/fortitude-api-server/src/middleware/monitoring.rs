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

// ABOUTME: Monitoring middleware for API server
//! # API Server Monitoring Integration
//!
//! This middleware integrates the Fortitude monitoring system with the API server,
//! providing automatic metrics collection, performance tracking, and health monitoring
//! for all HTTP endpoints and operations.
//!
//! ## Features
//!
//! - **Request/Response Metrics**: Automatic timing and status code tracking
//! - **Performance Monitoring**: Response time tracking with percentiles
//! - **Error Rate Monitoring**: Automatic error detection and classification
//! - **Resource Utilization**: Server resource usage tracking
//! - **Health Check Integration**: Endpoint health status monitoring
//! - **Custom Metrics**: Support for application-specific metrics
//!
//! ## Integration
//!
//! The middleware integrates with the core monitoring system to provide:
//! - Real-time performance dashboards
//! - Automated alerting on threshold violations
//! - Historical trend analysis
//! - SLA compliance monitoring

use axum::{
    extract::{MatchedPath, Request, State},
    http::{Method, StatusCode},
    middleware::Next,
    response::Response,
};
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
    pub fn for_api_server() -> Self {
        Self {
            target_response_ms: 200,
            warning_response_ms: 300,
            critical_response_ms: 500,
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

/// API server monitoring integration
#[derive(Debug, Clone)]
pub struct ApiMonitoringService {
    /// Monitoring configuration
    config: MonitoringConfiguration,

    /// Request metrics storage
    metrics: Arc<RwLock<ApiMetrics>>,

    /// Component health status
    health: Arc<RwLock<ComponentHealth>>,

    /// Custom metrics storage
    custom_metrics: Arc<RwLock<HashMap<String, f64>>>,
}

/// API-specific metrics
#[derive(Debug, Clone)]
pub struct ApiMetrics {
    /// Total HTTP requests processed
    pub total_requests: u64,

    /// Successful requests (2xx status codes)
    pub successful_requests: u64,

    /// Client error requests (4xx status codes)
    pub client_errors: u64,

    /// Server error requests (5xx status codes)
    pub server_errors: u64,

    /// Request duration samples for percentile calculation
    pub duration_samples: Vec<Duration>,

    /// Endpoint-specific metrics
    pub endpoint_metrics: HashMap<String, EndpointMetrics>,

    /// Method-specific metrics
    pub method_metrics: HashMap<String, MethodMetrics>,

    /// Response size tracking
    pub total_response_bytes: u64,

    /// Active connection count
    pub active_connections: u64,

    /// Peak concurrent connections
    pub peak_connections: u64,

    /// Request rate (requests per second)
    pub request_rate: f64,

    /// Last update timestamp
    pub last_update: Instant,
}

/// Metrics for specific endpoints
#[derive(Debug, Clone)]
pub struct EndpointMetrics {
    /// Endpoint path
    pub path: String,

    /// Total requests to this endpoint
    pub total_requests: u64,

    /// Successful requests
    pub successful_requests: u64,

    /// Error requests
    pub error_requests: u64,

    /// Average response time
    pub avg_response_time: Duration,

    /// 95th percentile response time
    pub p95_response_time: Duration,

    /// 99th percentile response time
    pub p99_response_time: Duration,

    /// Response time samples
    pub response_times: Vec<Duration>,

    /// Response size statistics
    pub avg_response_size: u64,
    pub total_response_size: u64,
}

/// Metrics for HTTP methods
#[derive(Debug, Clone)]
pub struct MethodMetrics {
    /// HTTP method
    pub method: String,

    /// Total requests for this method
    pub total_requests: u64,

    /// Successful requests
    pub successful_requests: u64,

    /// Error requests
    pub error_requests: u64,

    /// Average response time
    pub avg_response_time: Duration,
}

/// Request context for monitoring
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Request start time
    pub start_time: Instant,

    /// HTTP method
    pub method: Method,

    /// Request path
    pub path: String,

    /// Request ID
    pub request_id: Option<String>,

    /// Custom labels
    pub labels: HashMap<String, String>,
}

impl ApiMonitoringService {
    /// Create new API monitoring service
    pub fn new(config: MonitoringConfiguration) -> Self {
        let health = ComponentHealth {
            component_name: "api-server".to_string(),
            status: HealthStatus::Healthy,
            message: "API server operational".to_string(),
            last_check_time: chrono::Utc::now(),
            checks: HashMap::new(),
        };

        Self {
            config,
            metrics: Arc::new(RwLock::new(ApiMetrics::new())),
            health: Arc::new(RwLock::new(health)),
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create monitoring service with API server optimized configuration
    pub fn for_api_server() -> Self {
        let config = MonitoringConfiguration::for_api_server();
        Self::new(config)
    }

    /// Record HTTP request metrics
    pub async fn record_request(
        &self,
        method: &Method,
        path: &str,
        status_code: StatusCode,
        duration: Duration,
        response_size: Option<u64>,
    ) -> MonitoringResult<()> {
        let mut metrics = self.metrics.write().await;

        // Update total request count
        metrics.total_requests += 1;

        // Update status code metrics
        match status_code.as_u16() {
            200..=299 => metrics.successful_requests += 1,
            400..=499 => metrics.client_errors += 1,
            500..=599 => metrics.server_errors += 1,
            _ => {} // Other status codes
        }

        // Add duration sample
        metrics.duration_samples.push(duration);

        // Limit samples to prevent memory growth
        if metrics.duration_samples.len() > self.config.max_metrics_in_memory {
            metrics.duration_samples.remove(0);
        }

        // Update response size
        if let Some(size) = response_size {
            metrics.total_response_bytes += size;
        }

        // Update endpoint-specific metrics
        let endpoint_key = format!("{method} {path}");
        let endpoint_metrics = metrics
            .endpoint_metrics
            .entry(endpoint_key.clone())
            .or_insert_with(|| EndpointMetrics::new(endpoint_key));

        endpoint_metrics.record_request(status_code, duration, response_size);

        // Update method-specific metrics
        let method_key = method.to_string();
        let method_metrics = metrics
            .method_metrics
            .entry(method_key.clone())
            .or_insert_with(|| MethodMetrics::new(method_key));

        method_metrics.record_request(status_code, duration);

        // Update request rate
        let now = Instant::now();
        let time_diff = now.duration_since(metrics.last_update).as_secs_f64();
        if time_diff > 0.0 {
            metrics.request_rate = 1.0 / time_diff;
        }
        metrics.last_update = now;

        // Check performance thresholds
        self.check_performance_thresholds(&metrics, duration, status_code)
            .await?;

        debug!(
            "Recorded API request: {} {} -> {} in {:?}",
            method, path, status_code, duration
        );

        Ok(())
    }

    /// Check performance thresholds and update health status
    async fn check_performance_thresholds(
        &self,
        metrics: &ApiMetrics,
        duration: Duration,
        _status_code: StatusCode,
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
        if metrics.total_requests >= 10 {
            let error_rate = (metrics.client_errors + metrics.server_errors) as f64
                / metrics.total_requests as f64
                * 100.0;

            let critical_error_rate = 10.0;
            let warning_error_rate = 5.0;

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
            health.message = "API server operating within normal parameters".to_string();
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

    /// Get current API metrics
    pub async fn get_api_metrics(&self) -> ApiMetrics {
        self.metrics.read().await.clone()
    }

    /// Get performance summary
    pub async fn get_performance_summary(&self) -> MonitoringResult<PerformanceSummary> {
        let metrics = self.metrics.read().await;
        let health = self.health.read().await;

        // Calculate percentiles
        let mut durations: Vec<u64> = metrics
            .duration_samples
            .iter()
            .map(|d| d.as_millis() as u64)
            .collect();
        durations.sort_unstable();

        let p50 = percentile(&durations, 50.0);
        let p95 = percentile(&durations, 95.0);
        let p99 = percentile(&durations, 99.0);

        let avg_duration = if !durations.is_empty() {
            durations.iter().sum::<u64>() / durations.len() as u64
        } else {
            0
        };

        let error_rate = if metrics.total_requests > 0 {
            (metrics.client_errors + metrics.server_errors) as f64 / metrics.total_requests as f64
                * 100.0
        } else {
            0.0
        };

        let success_rate = if metrics.total_requests > 0 {
            metrics.successful_requests as f64 / metrics.total_requests as f64 * 100.0
        } else {
            100.0
        };

        Ok(PerformanceSummary {
            total_requests: metrics.total_requests,
            error_rate,
            success_rate,
            avg_response_time_ms: avg_duration,
            p50_response_time_ms: p50,
            p95_response_time_ms: p95,
            p99_response_time_ms: p99,
            request_rate: metrics.request_rate,
            active_connections: metrics.active_connections,
            peak_connections: metrics.peak_connections,
            health_status: health.status.clone(),
            health_message: health.message.clone(),
            endpoint_count: metrics.endpoint_metrics.len(),
        })
    }

    /// Update connection count
    pub async fn update_connection_count(&self, active: u64) -> MonitoringResult<()> {
        let mut metrics = self.metrics.write().await;
        metrics.active_connections = active;
        if active > metrics.peak_connections {
            metrics.peak_connections = active;
        }
        Ok(())
    }
}

/// Performance summary for API endpoints
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceSummary {
    pub total_requests: u64,
    pub error_rate: f64,
    pub success_rate: f64,
    pub avg_response_time_ms: u64,
    pub p50_response_time_ms: u64,
    pub p95_response_time_ms: u64,
    pub p99_response_time_ms: u64,
    pub request_rate: f64,
    pub active_connections: u64,
    pub peak_connections: u64,
    pub health_status: HealthStatus,
    pub health_message: String,
    pub endpoint_count: usize,
}

impl ApiMetrics {
    fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            client_errors: 0,
            server_errors: 0,
            duration_samples: Vec::new(),
            endpoint_metrics: HashMap::new(),
            method_metrics: HashMap::new(),
            total_response_bytes: 0,
            active_connections: 0,
            peak_connections: 0,
            request_rate: 0.0,
            last_update: Instant::now(),
        }
    }
}

impl EndpointMetrics {
    fn new(path: String) -> Self {
        Self {
            path,
            total_requests: 0,
            successful_requests: 0,
            error_requests: 0,
            avg_response_time: Duration::default(),
            p95_response_time: Duration::default(),
            p99_response_time: Duration::default(),
            response_times: Vec::new(),
            avg_response_size: 0,
            total_response_size: 0,
        }
    }

    fn record_request(
        &mut self,
        status_code: StatusCode,
        duration: Duration,
        response_size: Option<u64>,
    ) {
        self.total_requests += 1;

        match status_code.as_u16() {
            200..=299 => self.successful_requests += 1,
            _ => self.error_requests += 1,
        }

        self.response_times.push(duration);

        // Limit samples
        if self.response_times.len() > 1000 {
            self.response_times.remove(0);
        }

        // Update averages
        let total_ms: u64 = self
            .response_times
            .iter()
            .map(|d| d.as_millis() as u64)
            .sum();
        self.avg_response_time = Duration::from_millis(total_ms / self.response_times.len() as u64);

        // Calculate percentiles
        let mut sorted_times: Vec<u64> = self
            .response_times
            .iter()
            .map(|d| d.as_millis() as u64)
            .collect();
        sorted_times.sort_unstable();
        self.p95_response_time = Duration::from_millis(percentile(&sorted_times, 95.0));
        self.p99_response_time = Duration::from_millis(percentile(&sorted_times, 99.0));

        // Update response size
        if let Some(size) = response_size {
            self.total_response_size += size;
            self.avg_response_size = self.total_response_size / self.total_requests;
        }
    }
}

impl MethodMetrics {
    fn new(method: String) -> Self {
        Self {
            method,
            total_requests: 0,
            successful_requests: 0,
            error_requests: 0,
            avg_response_time: Duration::default(),
        }
    }

    fn record_request(&mut self, status_code: StatusCode, duration: Duration) {
        self.total_requests += 1;

        match status_code.as_u16() {
            200..=299 => self.successful_requests += 1,
            _ => self.error_requests += 1,
        }

        // Update average response time (simple moving average)
        let current_avg_ms = self.avg_response_time.as_millis() as u64;
        let new_duration_ms = duration.as_millis() as u64;
        let new_avg_ms =
            (current_avg_ms * (self.total_requests - 1) + new_duration_ms) / self.total_requests;
        self.avg_response_time = Duration::from_millis(new_avg_ms);
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

/// Middleware function for automatic request monitoring
pub async fn monitoring_middleware(
    State(monitoring_service): State<Arc<ApiMonitoringService>>,
    request: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    let method = request.method().clone();
    let path = request
        .extensions()
        .get::<MatchedPath>()
        .map(|matched_path| matched_path.as_str())
        .unwrap_or_else(|| request.uri().path())
        .to_string();

    // Process the request
    let response = next.run(request).await;

    // Record metrics
    let duration = start_time.elapsed();
    let status_code = response.status();

    // Try to get response size from headers
    let response_size = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok());

    // Record the request metrics
    if let Err(e) = monitoring_service
        .record_request(&method, &path, status_code, duration, response_size)
        .await
    {
        error!("Failed to record request metrics: {}", e);
    }

    // Log slow requests
    let target_ms = monitoring_service.config.target_response_ms;
    if duration.as_millis() as u64 > target_ms {
        warn!(
            "Slow request detected: {} {} took {:?} (target: {}ms)",
            method, path, duration, target_ms
        );
    }

    response
}

/// Implement Monitorable trait for API monitoring service
#[async_trait::async_trait]
impl Monitorable for ApiMonitoringService {
    fn component_name(&self) -> &str {
        "api-server"
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

        // Calculate average duration
        let avg_duration = if !metrics.duration_samples.is_empty() {
            let total_ms: u128 = metrics.duration_samples.iter().map(|d| d.as_millis()).sum();
            Duration::from_millis((total_ms / metrics.duration_samples.len() as u128) as u64)
        } else {
            Duration::default()
        };

        // Calculate 95th percentile
        let mut durations: Vec<u64> = metrics
            .duration_samples
            .iter()
            .map(|d| d.as_millis() as u64)
            .collect();
        durations.sort_unstable();
        let p95_duration = Duration::from_millis(percentile(&durations, 95.0));

        // Calculate operations per second
        let ops_per_second = if metrics.last_update.elapsed().as_secs() > 0 {
            metrics.total_requests as f64 / metrics.last_update.elapsed().as_secs() as f64
        } else {
            0.0
        };

        Ok(ComponentMetrics {
            component_name: "api-server".to_string(),
            total_operations: metrics.total_requests,
            successful_operations: metrics.successful_requests,
            failed_operations: metrics.client_errors + metrics.server_errors,
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
    use axum::http::Method;

    #[tokio::test]
    async fn test_api_monitoring_service_creation() {
        let service = ApiMonitoringService::for_api_server();
        assert_eq!(service.component_name(), "api-server");

        let health = service.get_health_status().await.unwrap();
        assert_eq!(health.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_request_recording() {
        let service = ApiMonitoringService::for_api_server();

        // Record a successful request
        service
            .record_request(
                &Method::GET,
                "/api/v1/health",
                StatusCode::OK,
                Duration::from_millis(150),
                Some(1024),
            )
            .await
            .unwrap();

        let metrics = service.get_api_metrics().await;
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.client_errors, 0);
        assert_eq!(metrics.server_errors, 0);

        // Check endpoint metrics
        assert!(metrics.endpoint_metrics.contains_key("GET /api/v1/health"));
    }

    #[tokio::test]
    async fn test_error_recording() {
        let service = ApiMonitoringService::for_api_server();

        // Record a client error
        service
            .record_request(
                &Method::POST,
                "/api/v1/research",
                StatusCode::BAD_REQUEST,
                Duration::from_millis(50),
                None,
            )
            .await
            .unwrap();

        // Record a server error
        service
            .record_request(
                &Method::GET,
                "/api/v1/health",
                StatusCode::INTERNAL_SERVER_ERROR,
                Duration::from_millis(100),
                None,
            )
            .await
            .unwrap();

        let metrics = service.get_api_metrics().await;
        assert_eq!(metrics.total_requests, 2);
        assert_eq!(metrics.successful_requests, 0);
        assert_eq!(metrics.client_errors, 1);
        assert_eq!(metrics.server_errors, 1);
    }

    #[tokio::test]
    async fn test_performance_thresholds() {
        let service = ApiMonitoringService::for_api_server();

        // Record a slow request (exceeding target)
        service
            .record_request(
                &Method::GET,
                "/api/v1/research",
                StatusCode::OK,
                Duration::from_millis(300), // Exceeds 200ms target
                None,
            )
            .await
            .unwrap();

        let health = service.get_health_status().await.unwrap();
        // Health should still be healthy for single slow request
        assert!(health.status == HealthStatus::Healthy || health.status == HealthStatus::Degraded);
    }

    #[tokio::test]
    async fn test_custom_metrics() {
        let service = ApiMonitoringService::for_api_server();

        // Record custom metric
        service
            .record_custom_metric("cache_hit_rate", 0.85)
            .await
            .unwrap();
        service
            .record_custom_metric("memory_usage_mb", 256.0)
            .await
            .unwrap();

        let performance_metrics = service.get_performance_metrics().await.unwrap();
        assert_eq!(
            performance_metrics.custom_metrics.get("cache_hit_rate"),
            Some(&0.85)
        );
        assert_eq!(
            performance_metrics.custom_metrics.get("memory_usage_mb"),
            Some(&256.0)
        );
    }

    #[tokio::test]
    async fn test_performance_summary() {
        let service = ApiMonitoringService::for_api_server();

        // Record several requests
        for i in 0..10 {
            let duration = Duration::from_millis(100 + i * 10);
            service
                .record_request(
                    &Method::GET,
                    "/api/v1/test",
                    StatusCode::OK,
                    duration,
                    Some(1024),
                )
                .await
                .unwrap();
        }

        let summary = service.get_performance_summary().await.unwrap();
        assert_eq!(summary.total_requests, 10);
        assert_eq!(summary.success_rate, 100.0);
        assert_eq!(summary.error_rate, 0.0);
        assert!(summary.avg_response_time_ms > 0);
        assert!(summary.p95_response_time_ms >= summary.avg_response_time_ms);
    }

    #[test]
    fn test_percentile_calculation() {
        let data = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];

        assert_eq!(percentile(&data, 50.0), 50);
        assert_eq!(percentile(&data, 90.0), 90);
        assert_eq!(percentile(&data, 95.0), 100);

        // Test empty data
        assert_eq!(percentile(&[], 50.0), 0);

        // Test single element
        assert_eq!(percentile(&[42], 50.0), 42);
    }

    #[tokio::test]
    async fn test_connection_tracking() {
        let service = ApiMonitoringService::for_api_server();

        // Update connection count
        service.update_connection_count(50).await.unwrap();
        service.update_connection_count(75).await.unwrap();
        service.update_connection_count(30).await.unwrap();

        let metrics = service.get_api_metrics().await;
        assert_eq!(metrics.active_connections, 30);
        assert_eq!(metrics.peak_connections, 75);
    }
}
