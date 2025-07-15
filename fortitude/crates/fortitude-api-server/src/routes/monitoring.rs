// ABOUTME: Monitoring dashboard endpoints for system observability and performance tracking

use crate::models::responses::{
    MonitoringAlertResponse, MonitoringAlertSeverity, MonitoringAlertsResponse,
    MonitoringApiMetricsResponse, MonitoringCacheMetricsResponse,
    MonitoringComponentHealthResponse, MonitoringCurrentMetricsResponse,
    MonitoringDashboardResponse, MonitoringDataPoint, MonitoringHealthResponse,
    MonitoringHealthStatus, MonitoringHealthStatusResponse, MonitoringLearningMetricsResponse,
    MonitoringMetricsResponse, MonitoringPerformanceSummaryResponse,
    MonitoringProviderMetricsResponse, MonitoringQualityMetricsResponse,
    MonitoringResourceMetricsResponse, MonitoringSystemOverviewResponse, PaginationInfo,
};
use crate::monitoring_types::{
    AlertManager, AlertSeverity, HealthChecker, HealthStatus, MetricsCollector,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, instrument};

/// State for monitoring endpoints
#[derive(Debug, Clone)]
pub struct MonitoringState {
    /// Metrics collector instance
    pub metrics_collector: Arc<MetricsCollector>,
    /// Health checker instance  
    pub health_checker: Arc<HealthChecker>,
    /// Alert manager instance
    pub alert_manager: Arc<AlertManager>,
}

impl MonitoringState {
    /// Create new monitoring state
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!("Initializing monitoring state for API endpoints");

        let metrics_collector = Arc::new(MetricsCollector::new());
        let health_checker = Arc::new(HealthChecker::new());
        let alert_manager = Arc::new(AlertManager::new());

        Ok(Self {
            metrics_collector,
            health_checker,
            alert_manager,
        })
    }
}

/// Query parameters for monitoring endpoints
#[derive(Debug, Deserialize)]
pub struct MonitoringQuery {
    /// Time range for metrics (in hours)
    #[serde(default = "default_time_range")]
    pub time_range_hours: u32,
    /// Include detailed graphs
    #[serde(default)]
    pub include_graphs: bool,
    /// Limit for results
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Offset for pagination
    #[serde(default)]
    pub offset: usize,
}

fn default_time_range() -> u32 {
    24
}
fn default_limit() -> usize {
    50
}

/// Get comprehensive monitoring dashboard data
#[utoipa::path(
    get,
    path = "/api/v1/monitoring/dashboard",
    responses(
        (status = 200, description = "Monitoring dashboard data", body = MonitoringDashboardResponse),
        (status = 500, description = "Internal server error"),
    ),
    params(
        ("time_range_hours" = Option<u32>, Query, description = "Time range for metrics in hours (default: 24)"),
        ("include_graphs" = Option<bool>, Query, description = "Include detailed performance graphs"),
    ),
    tag = "Monitoring"
)]
#[instrument(skip(state))]
pub async fn get_monitoring_dashboard(
    State(state): State<Arc<MonitoringState>>,
    Query(query): Query<MonitoringQuery>,
) -> Result<Json<MonitoringDashboardResponse>, StatusCode> {
    let start_time = std::time::Instant::now();

    info!(
        "Fetching monitoring dashboard data with time_range: {} hours",
        query.time_range_hours
    );

    match get_dashboard_data(&state, &query).await {
        Ok(dashboard_data) => {
            let processing_time = start_time.elapsed();
            info!(
                "Successfully generated monitoring dashboard data in {:?}",
                processing_time
            );

            Ok(Json(dashboard_data))
        }
        Err(e) => {
            error!("Failed to generate monitoring dashboard data: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get current system metrics
#[utoipa::path(
    get,
    path = "/api/v1/monitoring/metrics",
    responses(
        (status = 200, description = "Current system metrics", body = MonitoringMetricsResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Monitoring"
)]
#[instrument(skip(state))]
pub async fn get_monitoring_metrics(
    State(state): State<Arc<MonitoringState>>,
) -> Result<Json<MonitoringMetricsResponse>, StatusCode> {
    let start_time = std::time::Instant::now();

    info!("Fetching current monitoring metrics");

    match get_current_metrics(&state).await {
        Ok(metrics) => {
            let processing_time = start_time.elapsed();
            info!(
                "Successfully fetched monitoring metrics in {:?}",
                processing_time
            );

            let response_data = MonitoringMetricsResponse {
                metrics,
                total_count: 1, // TODO: Calculate actual total count of metrics
                processing_time_ms: processing_time.as_millis() as u64,
            };

            Ok(Json(response_data))
        }
        Err(e) => {
            error!("Failed to fetch monitoring metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get system health status
#[utoipa::path(
    get,
    path = "/api/v1/monitoring/health",
    responses(
        (status = 200, description = "System health status", body = MonitoringHealthResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Monitoring"
)]
#[instrument(skip(state))]
pub async fn get_monitoring_health(
    State(state): State<Arc<MonitoringState>>,
) -> Result<Json<MonitoringHealthResponse>, StatusCode> {
    let start_time = std::time::Instant::now();

    info!("Fetching monitoring health status");

    match get_health_status(&state).await {
        Ok(health) => {
            let processing_time = start_time.elapsed();
            info!(
                "Successfully fetched health status in {:?}",
                processing_time
            );

            let response_data = MonitoringHealthResponse {
                health: health.clone(),
                overall_status: format!("{:?}", health.overall_status), // TODO: Convert enum to string properly
                processing_time_ms: processing_time.as_millis() as u64,
            };

            Ok(Json(response_data))
        }
        Err(e) => {
            error!("Failed to fetch health status: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get monitoring alerts
#[utoipa::path(
    get,
    path = "/api/v1/monitoring/alerts",
    responses(
        (status = 200, description = "Monitoring alerts list", body = MonitoringAlertsResponse),
        (status = 500, description = "Internal server error"),
    ),
    params(
        ("limit" = Option<usize>, Query, description = "Maximum number of alerts to return"),
        ("offset" = Option<usize>, Query, description = "Offset for pagination"),
    ),
    tag = "Monitoring"
)]
#[instrument(skip(state))]
pub async fn get_monitoring_alerts(
    State(state): State<Arc<MonitoringState>>,
    Query(query): Query<MonitoringQuery>,
) -> Result<Json<MonitoringAlertsResponse>, StatusCode> {
    let start_time = std::time::Instant::now();

    info!(
        "Fetching monitoring alerts with limit: {}, offset: {}",
        query.limit, query.offset
    );

    match get_alerts(&state, &query).await {
        Ok(alerts_data) => {
            let processing_time = start_time.elapsed();
            info!(
                "Successfully fetched monitoring alerts in {:?}",
                processing_time
            );

            Ok(Json(alerts_data))
        }
        Err(e) => {
            error!("Failed to fetch monitoring alerts: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get monitoring performance summary
#[utoipa::path(
    get,
    path = "/api/v1/monitoring/performance",
    responses(
        (status = 200, description = "Performance summary", body = MonitoringPerformanceSummaryResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Monitoring"
)]
#[instrument(skip(state))]
pub async fn get_monitoring_performance_summary(
    State(state): State<Arc<MonitoringState>>,
) -> Result<Json<MonitoringPerformanceSummaryResponse>, StatusCode> {
    let start_time = std::time::Instant::now();

    info!("Fetching monitoring performance summary");

    match get_performance_summary(&state).await {
        Ok(summary) => {
            let processing_time = start_time.elapsed();
            info!(
                "Successfully fetched performance summary in {:?}",
                processing_time
            );

            Ok(Json(summary))
        }
        Err(e) => {
            error!("Failed to fetch performance summary: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Helper functions for data aggregation

async fn get_dashboard_data(
    state: &MonitoringState,
    query: &MonitoringQuery,
) -> Result<MonitoringDashboardResponse, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = std::time::Instant::now();

    // Get current metrics
    let current_metrics = get_current_metrics(state).await?;

    // Get health status
    let health_status = get_health_status(state).await?;

    // Get active alerts (limited)
    let alerts_query = MonitoringQuery {
        limit: 10,
        offset: 0,
        time_range_hours: query.time_range_hours,
        include_graphs: false,
    };
    let alerts_response = get_alerts(state, &alerts_query).await?;
    let active_alerts = alerts_response.alerts;

    // Generate performance graphs if requested
    let performance_graphs = if query.include_graphs {
        generate_performance_graphs(state, query.time_range_hours).await?
    } else {
        HashMap::new()
    };

    // Calculate system overview
    let system_overview = calculate_system_overview(state, &active_alerts).await?;

    // Determine overall status
    let overall_status = match health_status.overall_status {
        MonitoringHealthStatus::Healthy => "healthy".to_string(),
        MonitoringHealthStatus::Warning => "warning".to_string(),
        MonitoringHealthStatus::Critical => "critical".to_string(),
        MonitoringHealthStatus::Unknown => "unknown".to_string(),
    };

    let processing_time = start_time.elapsed();

    Ok(MonitoringDashboardResponse {
        overall_status,
        current_metrics,
        health_status,
        active_alerts,
        performance_graphs,
        system_overview,
        processing_time_ms: processing_time.as_millis() as u64,
    })
}

async fn get_current_metrics(
    state: &MonitoringState,
) -> Result<MonitoringCurrentMetricsResponse, Box<dyn std::error::Error + Send + Sync>> {
    // Get API metrics
    let api_metrics_data = state
        .metrics_collector
        .get_api_metrics()
        .await
        .map_err(|e| format!("Failed to get API metrics: {e}"))?;

    let api_metrics = MonitoringApiMetricsResponse {
        total_requests: api_metrics_data.total_requests,
        successful_requests: api_metrics_data.successful_requests,
        failed_requests: api_metrics_data.failed_requests,
        average_response_time_ms: api_metrics_data.average_response_time.as_millis() as f64,
        p95_response_time_ms: api_metrics_data.p95_response_time.as_millis() as f64,
        error_rate: api_metrics_data.error_rate(),
        requests_by_method: api_metrics_data.requests_by_method,
        requests_by_path: api_metrics_data.requests_by_path,
        last_request_time: api_metrics_data.last_request_time,
    };

    // Get provider metrics (simulated - would be real provider data)
    let mut provider_metrics = HashMap::new();
    provider_metrics.insert(
        "claude".to_string(),
        MonitoringProviderMetricsResponse {
            provider_name: "claude".to_string(),
            request_count: 245,
            success_rate: 0.98,
            average_latency_ms: 850.5,
            error_count: 5,
            last_success_time: Utc::now(),
        },
    );

    // Get quality metrics
    let quality_metrics_data = state
        .metrics_collector
        .get_quality_metrics()
        .await
        .map_err(|e| format!("Failed to get quality metrics: {e}"))?;

    let quality_metrics = MonitoringQualityMetricsResponse {
        total_evaluations: quality_metrics_data.total_evaluations,
        average_processing_time_ms: quality_metrics_data.average_processing_time.as_millis() as f64,
        total_tokens_processed: quality_metrics_data.total_tokens_processed,
        evaluations_by_type: quality_metrics_data.evaluations_by_type,
    };

    // Get cache metrics (simulated - would be real cache data)
    let mut cache_metrics = HashMap::new();
    cache_metrics.insert(
        "main_cache".to_string(),
        MonitoringCacheMetricsResponse {
            cache_name: "main_cache".to_string(),
            total_operations: 1250,
            hit_count: 1000,
            miss_count: 250,
            write_count: 180,
            eviction_count: 25,
            hit_rate: 0.8,
            average_hit_time_ms: 2.5,
            average_miss_time_ms: 45.8,
        },
    );

    // Get learning metrics
    let learning_metrics_data = state
        .metrics_collector
        .get_learning_metrics()
        .await
        .map_err(|e| format!("Failed to get learning metrics: {e}"))?;

    let learning_metrics = MonitoringLearningMetricsResponse {
        feedback_processed: learning_metrics_data.feedback_processed,
        patterns_recognized: learning_metrics_data.patterns_recognized,
        adaptations_applied: learning_metrics_data.adaptations_applied,
        learning_accuracy: learning_metrics_data.learning_accuracy,
        processing_time_ms: learning_metrics_data.processing_time.as_millis() as f64,
    };

    // Get resource metrics
    let resource_metrics_data = state
        .metrics_collector
        .get_resource_metrics()
        .await
        .map_err(|e| format!("Failed to get resource metrics: {e}"))?;

    let resource_metrics = MonitoringResourceMetricsResponse {
        cpu_usage_percent: resource_metrics_data.cpu_usage_percent,
        memory_usage_mb: resource_metrics_data.memory_usage_bytes as f64 / (1024.0 * 1024.0),
        network_bytes_sent: resource_metrics_data.network_bytes_sent,
        network_bytes_received: resource_metrics_data.network_bytes_received,
        disk_io_bytes: resource_metrics_data.disk_io_bytes,
        timestamp: resource_metrics_data.timestamp,
    };

    Ok(MonitoringCurrentMetricsResponse {
        api_metrics,
        provider_metrics,
        quality_metrics,
        cache_metrics,
        learning_metrics,
        resource_metrics,
        timestamp: Utc::now(),
    })
}

async fn get_health_status(
    state: &MonitoringState,
) -> Result<MonitoringHealthStatusResponse, Box<dyn std::error::Error + Send + Sync>> {
    // Get health check results
    let health_report = state
        .health_checker
        .get_health_report()
        .await
        .map_err(|e| format!("Failed to get health report: {e}"))?;

    let overall_status = match health_report.overall_health {
        HealthStatus::Healthy => MonitoringHealthStatus::Healthy,
        HealthStatus::Degraded => MonitoringHealthStatus::Warning,
        HealthStatus::Unhealthy => MonitoringHealthStatus::Critical,
        HealthStatus::Unknown => MonitoringHealthStatus::Unknown,
    };

    let component_results = health_report
        .component_health
        .into_iter()
        .map(|component| {
            let status = match component.status {
                HealthStatus::Healthy => MonitoringHealthStatus::Healthy,
                HealthStatus::Degraded => MonitoringHealthStatus::Warning,
                HealthStatus::Unhealthy => MonitoringHealthStatus::Critical,
                HealthStatus::Unknown => MonitoringHealthStatus::Unknown,
            };

            MonitoringComponentHealthResponse {
                component: component.component_name,
                status,
                message: component.message,
                timestamp: component.last_check_time,
                response_time_ms: 0, // Would be calculated from actual check times
                details: component.checks,
            }
        })
        .collect();

    Ok(MonitoringHealthStatusResponse {
        overall_status,
        component_results,
        summary: health_report.summary,
        timestamp: health_report.timestamp,
    })
}

async fn get_alerts(
    state: &MonitoringState,
    query: &MonitoringQuery,
) -> Result<MonitoringAlertsResponse, Box<dyn std::error::Error + Send + Sync>> {
    // Get alerts from alert manager
    let alerts_data = state
        .alert_manager
        .get_alerts()
        .await
        .map_err(|e| format!("Failed to get alerts: {e}"))?;

    let alerts: Vec<MonitoringAlertResponse> = alerts_data
        .into_iter()
        .map(|alert| {
            let severity = match alert.severity {
                AlertSeverity::Info => MonitoringAlertSeverity::Info,
                AlertSeverity::Warning => MonitoringAlertSeverity::Warning,
                AlertSeverity::Critical => MonitoringAlertSeverity::Critical,
            };

            MonitoringAlertResponse {
                id: alert.id,
                severity,
                component: alert.component,
                message: alert.message,
                metric_value: alert.metric_value.unwrap_or(0.0),
                threshold: alert.threshold.unwrap_or(0.0),
                timestamp: alert.timestamp,
                acknowledged: alert.acknowledged,
            }
        })
        .collect();

    // Apply pagination
    let total_count = alerts.len();
    let end_index = std::cmp::min(query.offset + query.limit, alerts.len());
    let paginated_alerts = if query.offset < alerts.len() {
        alerts[query.offset..end_index].to_vec()
    } else {
        vec![]
    };

    let unacknowledged_count = alerts.iter().filter(|a| !a.acknowledged).count();

    let pagination = PaginationInfo {
        offset: query.offset,
        limit: query.limit,
        total_pages: total_count.div_ceil(query.limit),
        has_more: query.offset + query.limit < total_count,
    };

    Ok(MonitoringAlertsResponse {
        alerts: paginated_alerts,
        total_count,
        unacknowledged_count,
        pagination,
        processing_time_ms: 0, // Would be calculated
    })
}

async fn get_performance_summary(
    state: &MonitoringState,
) -> Result<MonitoringPerformanceSummaryResponse, Box<dyn std::error::Error + Send + Sync>> {
    // Get current metrics for summary
    let metrics = get_current_metrics(state).await?;
    let health = get_health_status(state).await?;
    let alerts_query = MonitoringQuery {
        limit: 5,
        offset: 0,
        time_range_hours: 24,
        include_graphs: false,
    };
    let alerts_response = get_alerts(state, &alerts_query).await?;

    // Build key metrics
    let mut key_metrics = HashMap::new();
    key_metrics.insert(
        "response_time_ms".to_string(),
        metrics.api_metrics.average_response_time_ms,
    );
    key_metrics.insert("error_rate".to_string(), metrics.api_metrics.error_rate);
    key_metrics.insert(
        "cpu_usage_percent".to_string(),
        metrics.resource_metrics.cpu_usage_percent,
    );
    key_metrics.insert(
        "memory_usage_mb".to_string(),
        metrics.resource_metrics.memory_usage_mb,
    );

    // Build performance trends (simulated)
    let mut performance_trends = HashMap::new();
    performance_trends.insert(
        "response_time".to_string(),
        vec![180.0, 170.0, 190.0, 175.0, 165.0],
    );
    performance_trends.insert("error_rate".to_string(), vec![0.02, 0.03, 0.02, 0.01, 0.02]);

    // Generate recommendations
    let mut recommendations = Vec::new();
    if metrics.api_metrics.error_rate > 0.05 {
        recommendations.push("Consider investigating high error rate".to_string());
    }
    if metrics.resource_metrics.cpu_usage_percent > 80.0 {
        recommendations.push("CPU usage is high, consider scaling resources".to_string());
    }
    if metrics
        .cache_metrics
        .get("main_cache")
        .is_some_and(|c| c.hit_rate < 0.8)
    {
        recommendations
            .push("Cache hit rate is below optimal, review caching strategy".to_string());
    }

    Ok(MonitoringPerformanceSummaryResponse {
        overall_health: health.overall_status,
        key_metrics,
        active_alerts: alerts_response.alerts,
        performance_trends,
        recommendations,
        processing_time_ms: 0, // Would be calculated
    })
}

async fn generate_performance_graphs(
    _state: &MonitoringState,
    time_range_hours: u32,
) -> Result<HashMap<String, Vec<MonitoringDataPoint>>, Box<dyn std::error::Error + Send + Sync>> {
    let mut graphs = HashMap::new();

    // Generate sample time series data for the specified time range
    let now = Utc::now();
    let interval_minutes = if time_range_hours <= 1 {
        1
    } else if time_range_hours <= 24 {
        5
    } else {
        60
    };
    let data_points = (time_range_hours * 60) / interval_minutes;

    // Response time graph
    let mut response_time_data = Vec::new();
    for i in 0..data_points {
        let timestamp =
            now - chrono::Duration::minutes((data_points - i) as i64 * interval_minutes as i64);
        let value = 150.0 + (i as f64 * 0.5) + (i as f64 * 0.1).sin() * 20.0; // Simulated data
        response_time_data.push(MonitoringDataPoint { timestamp, value });
    }
    graphs.insert("response_time_ms".to_string(), response_time_data);

    // Error rate graph
    let mut error_rate_data = Vec::new();
    for i in 0..data_points {
        let timestamp =
            now - chrono::Duration::minutes((data_points - i) as i64 * interval_minutes as i64);
        let value = 0.02 + (i as f64 * 0.001).sin() * 0.01; // Simulated data
        error_rate_data.push(MonitoringDataPoint { timestamp, value });
    }
    graphs.insert("error_rate".to_string(), error_rate_data);

    // CPU usage graph
    let mut cpu_usage_data = Vec::new();
    for i in 0..data_points {
        let timestamp =
            now - chrono::Duration::minutes((data_points - i) as i64 * interval_minutes as i64);
        let value = 45.0 + (i as f64 * 0.02).cos() * 15.0; // Simulated data
        cpu_usage_data.push(MonitoringDataPoint { timestamp, value });
    }
    graphs.insert("cpu_usage_percent".to_string(), cpu_usage_data);

    Ok(graphs)
}

async fn calculate_system_overview(
    state: &MonitoringState,
    active_alerts: &[MonitoringAlertResponse],
) -> Result<MonitoringSystemOverviewResponse, Box<dyn std::error::Error + Send + Sync>> {
    let metrics = get_current_metrics(state).await?;

    // Calculate system-wide statistics
    let total_operations = metrics.api_metrics.total_requests;
    let success_rate = if total_operations > 0 {
        metrics.api_metrics.successful_requests as f64 / total_operations as f64
    } else {
        1.0
    };

    let average_response_time_ms = metrics.api_metrics.average_response_time_ms;
    let uptime_seconds = 86400; // Simulated 24 hours uptime

    let resource_utilization = (metrics.resource_metrics.cpu_usage_percent
        + (metrics.resource_metrics.memory_usage_mb / 1024.0 * 100.0))
        / 2.0;

    let active_alerts_count = active_alerts.len();
    let threshold_violations_count = active_alerts
        .iter()
        .filter(|a| a.severity == MonitoringAlertSeverity::Critical)
        .count();

    Ok(MonitoringSystemOverviewResponse {
        total_operations,
        success_rate,
        average_response_time_ms,
        uptime_seconds,
        resource_utilization,
        active_alerts_count,
        threshold_violations_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitoring_state_creation() {
        let state = MonitoringState::new().await;
        assert!(state.is_ok());
    }

    #[test]
    fn test_query_defaults() {
        let query: MonitoringQuery = serde_qs::from_str("").unwrap();
        assert_eq!(query.time_range_hours, 24);
        assert!(!query.include_graphs);
        assert_eq!(query.limit, 50);
        assert_eq!(query.offset, 0);
    }

    #[test]
    fn test_query_parsing() {
        let query: MonitoringQuery =
            serde_qs::from_str("time_range_hours=12&include_graphs=true&limit=100").unwrap();
        assert_eq!(query.time_range_hours, 12);
        assert!(query.include_graphs);
        assert_eq!(query.limit, 100);
    }
}
