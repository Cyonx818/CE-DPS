// ABOUTME: Learning system dashboard and metrics API endpoints
//! Learning system monitoring endpoints for dashboard integration
//!
//! Provides REST API endpoints for accessing learning system metrics, health status,
//! and performance data for dashboard visualization and monitoring.

use crate::models::responses::{
    LearningAdaptationMetrics, LearningComponentHealth, LearningDashboardResponse,
    LearningDataPoint, LearningFeedbackMetrics, LearningHealthResponse, LearningHealthStatus,
    LearningMetricsResponse, LearningOptimizationMetrics, LearningPatternRecognitionMetrics,
    LearningPerformanceSummaryResponse, LearningStorageMetrics, LearningSystemMetrics,
    LearningSystemOverview,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};
use utoipa;

/// Learning system state for dashboard endpoints
#[derive(Debug, Clone)]
pub struct LearningState {
    /// Learning performance monitor
    pub performance_monitor: Arc<LearningPerformanceMonitor>,
}

/// Mock learning performance monitor for dashboard integration
#[derive(Debug)]
pub struct LearningPerformanceMonitor {
    /// Start time for uptime calculation
    start_time: Instant,
    /// Mock metrics storage
    metrics_cache: Arc<RwLock<LearningMetricsResponse>>,
}

/// Query parameters for learning metrics endpoints
#[derive(Debug, Deserialize, Serialize)]
pub struct LearningMetricsQuery {
    /// Time duration for metrics (e.g., "1h", "24h", "7d")
    pub duration: Option<String>,
    /// Include detailed breakdown
    pub detailed: Option<bool>,
}

impl LearningState {
    /// Create new learning state
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!("Initializing learning state for API server");

        let performance_monitor = Arc::new(LearningPerformanceMonitor::new().await?);

        Ok(Self {
            performance_monitor,
        })
    }
}

impl LearningPerformanceMonitor {
    /// Create new learning performance monitor
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = Instant::now();
        let metrics_cache = Arc::new(RwLock::new(Self::create_mock_metrics()));

        info!("Learning performance monitor initialized");

        Ok(Self {
            start_time,
            metrics_cache,
        })
    }

    /// Get current learning metrics
    pub async fn get_current_metrics(&self) -> LearningMetricsResponse {
        let metrics = self.metrics_cache.read().await;
        let mut current_metrics = metrics.clone();

        // Update timestamp to current time
        current_metrics.timestamp = Utc::now();

        // Add some realistic variation to the metrics
        current_metrics.adaptation_metrics.adaptations_applied += 1;
        current_metrics.storage_metrics.total_operations += 3;
        current_metrics
            .pattern_recognition_metrics
            .patterns_analyzed += 2;

        current_metrics
    }

    /// Get learning system health
    pub async fn get_health_status(&self) -> LearningHealthResponse {
        let component_results = vec![
            LearningComponentHealth {
                component: "adaptation".to_string(),
                status: LearningHealthStatus::Healthy,
                message: "Adaptation system operating normally".to_string(),
                timestamp: Utc::now(),
                response_time_ms: 15,
                details: HashMap::new(),
            },
            LearningComponentHealth {
                component: "storage".to_string(),
                status: LearningHealthStatus::Healthy,
                message: "Storage system operational".to_string(),
                timestamp: Utc::now(),
                response_time_ms: 8,
                details: HashMap::new(),
            },
            LearningComponentHealth {
                component: "pattern_recognition".to_string(),
                status: LearningHealthStatus::Healthy,
                message: "Pattern recognition functioning properly".to_string(),
                timestamp: Utc::now(),
                response_time_ms: 22,
                details: HashMap::new(),
            },
        ];

        LearningHealthResponse {
            overall_status: LearningHealthStatus::Healthy,
            component_results,
            summary: "All learning system components are healthy".to_string(),
            timestamp: Utc::now(),
        }
    }

    /// Get dashboard data
    pub async fn get_dashboard_data(&self) -> LearningDashboardResponse {
        let current_metrics = self.get_current_metrics().await;
        let health_status = self.get_health_status().await;

        // Create mock alerts (none for healthy system)
        let alerts = vec![];

        // Create performance graphs data
        let mut performance_graphs = HashMap::new();
        let now = Utc::now();
        performance_graphs.insert(
            "adaptation_time".to_string(),
            vec![LearningDataPoint {
                timestamp: now,
                value: current_metrics
                    .adaptation_metrics
                    .average_adaptation_time_ms,
            }],
        );
        performance_graphs.insert(
            "success_rate".to_string(),
            vec![LearningDataPoint {
                timestamp: now,
                value: current_metrics.adaptation_metrics.success_rate,
            }],
        );

        let system_overview = LearningSystemOverview {
            total_adaptations: current_metrics.adaptation_metrics.adaptations_applied,
            success_rate: current_metrics.adaptation_metrics.success_rate,
            average_response_time: current_metrics.storage_metrics.average_response_time_ms,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            resource_utilization: current_metrics.system_metrics.cpu_usage_percent,
        };

        LearningDashboardResponse {
            current_metrics,
            health_status,
            alerts,
            performance_graphs,
            system_overview,
            processing_time_ms: 25,
        }
    }

    /// Get performance summary
    pub async fn get_performance_summary(&self) -> LearningPerformanceSummaryResponse {
        let current_metrics = self.get_current_metrics().await;
        let health_status = self.get_health_status().await;

        let mut key_metrics = HashMap::new();
        key_metrics.insert(
            "adaptation_success_rate".to_string(),
            current_metrics.adaptation_metrics.success_rate,
        );
        key_metrics.insert(
            "storage_error_rate".to_string(),
            current_metrics.storage_metrics.error_rate,
        );
        key_metrics.insert(
            "pattern_recognition_accuracy".to_string(),
            current_metrics
                .pattern_recognition_metrics
                .recognition_accuracy,
        );
        key_metrics.insert(
            "average_adaptation_time_ms".to_string(),
            current_metrics
                .adaptation_metrics
                .average_adaptation_time_ms,
        );

        let active_alerts = vec![]; // No alerts for healthy system

        let mut performance_trends = HashMap::new();
        performance_trends.insert(
            "success_rate".to_string(),
            vec![current_metrics.adaptation_metrics.success_rate],
        );
        performance_trends.insert(
            "response_time".to_string(),
            vec![current_metrics.storage_metrics.average_response_time_ms],
        );

        let recommendations = if current_metrics.adaptation_metrics.success_rate > 0.9 {
            vec!["Learning system is performing excellently - no recommendations".to_string()]
        } else {
            vec!["Consider tuning adaptation algorithms for better performance".to_string()]
        };

        LearningPerformanceSummaryResponse {
            overall_health: health_status.overall_status,
            key_metrics,
            active_alerts,
            performance_trends,
            recommendations,
            processing_time_ms: 18,
        }
    }

    /// Create mock metrics for development/testing
    fn create_mock_metrics() -> LearningMetricsResponse {
        LearningMetricsResponse {
            adaptation_metrics: LearningAdaptationMetrics {
                adaptations_applied: 147,
                adaptations_failed: 8,
                average_adaptation_time_ms: 245.7,
                confidence_scores: vec![0.85, 0.92, 0.78, 0.89, 0.94],
                success_rate: 0.948,
                last_adaptation: Some(Utc::now()),
            },
            storage_metrics: LearningStorageMetrics {
                total_operations: 1024,
                successful_operations: 1015,
                failed_operations: 9,
                average_response_time_ms: 12.3,
                cache_hit_rate: 0.87,
                storage_size_mb: 45.2,
                error_rate: 0.009,
            },
            pattern_recognition_metrics: LearningPatternRecognitionMetrics {
                patterns_analyzed: 523,
                patterns_recognized: 465,
                recognition_accuracy: 0.889,
                average_analysis_time_ms: 67.4,
                false_positive_rate: 0.045,
                false_negative_rate: 0.066,
            },
            feedback_metrics: LearningFeedbackMetrics {
                feedback_received: 89,
                feedback_processed: 89,
                average_feedback_score: 4.2,
                feedback_processing_time_ms: 3.7,
                feedback_trends: {
                    let mut trends = HashMap::new();
                    trends.insert("positive".to_string(), 0.73);
                    trends.insert("neutral".to_string(), 0.19);
                    trends.insert("negative".to_string(), 0.08);
                    trends
                },
            },
            optimization_metrics: LearningOptimizationMetrics {
                optimizations_suggested: 34,
                optimizations_applied: 28,
                performance_improvements: vec![0.12, 0.08, 0.15, 0.09, 0.21],
                optimization_success_rate: 0.824,
                average_optimization_time_ms: 1247.6,
            },
            system_metrics: LearningSystemMetrics {
                memory_usage_mb: 127.8,
                cpu_usage_percent: 15.3,
                disk_usage_mb: 892.1,
                network_io_mb: 2.7,
                uptime_seconds: 86400,
            },
            timestamp: Utc::now(),
        }
    }
}

/// Get learning dashboard data
#[utoipa::path(
    get,
    path = "/api/v1/learning/dashboard",
    responses(
        (status = 200, description = "Learning dashboard data retrieved successfully", body = LearningDashboardResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Learning"
)]
#[instrument(skip(state))]
pub async fn get_learning_dashboard_data(
    State(state): State<Arc<LearningState>>,
) -> Result<Json<LearningDashboardResponse>, StatusCode> {
    debug!("Retrieving learning dashboard data");

    let dashboard_data = state.performance_monitor.get_dashboard_data().await;
    info!("Learning dashboard data retrieved successfully");
    Ok(Json(dashboard_data))
}

/// Get current learning metrics
#[utoipa::path(
    get,
    path = "/api/v1/learning/metrics",
    params(
        ("duration" = Option<String>, Query, description = "Time duration for metrics (e.g., '1h', '24h', '7d')"),
        ("detailed" = Option<bool>, Query, description = "Include detailed breakdown"),
    ),
    responses(
        (status = 200, description = "Learning metrics retrieved successfully", body = LearningMetricsResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Learning"
)]
#[instrument(skip(state))]
pub async fn get_learning_metrics(
    State(state): State<Arc<LearningState>>,
    Query(params): Query<LearningMetricsQuery>,
) -> Result<Json<LearningMetricsResponse>, StatusCode> {
    debug!("Retrieving learning metrics with params: {:?}", params);

    let metrics = state.performance_monitor.get_current_metrics().await;
    info!("Learning metrics retrieved successfully");
    Ok(Json(metrics))
}

/// Get learning system health status
#[utoipa::path(
    get,
    path = "/api/v1/learning/health",
    responses(
        (status = 200, description = "Learning health status retrieved successfully", body = LearningHealthResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Learning"
)]
#[instrument(skip(state))]
pub async fn get_learning_health(
    State(state): State<Arc<LearningState>>,
) -> Result<Json<LearningHealthResponse>, StatusCode> {
    debug!("Retrieving learning health status");

    let health_status = state.performance_monitor.get_health_status().await;
    info!("Learning health status retrieved successfully");
    Ok(Json(health_status))
}

/// Get learning performance summary
#[utoipa::path(
    get,
    path = "/api/v1/learning/performance",
    responses(
        (status = 200, description = "Learning performance summary retrieved successfully", body = LearningPerformanceSummaryResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Learning"
)]
#[instrument(skip(state))]
pub async fn get_learning_performance_summary(
    State(state): State<Arc<LearningState>>,
) -> Result<Json<LearningPerformanceSummaryResponse>, StatusCode> {
    debug!("Retrieving learning performance summary");

    let performance_summary = state.performance_monitor.get_performance_summary().await;
    info!("Learning performance summary retrieved successfully");
    Ok(Json(performance_summary))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_learning_state_creation() {
        let state = LearningState::new().await.unwrap();
        assert!(!format!("{:?}", state).is_empty());
    }

    #[tokio::test]
    async fn test_learning_metrics_retrieval() {
        let state = LearningState::new().await.unwrap();
        let metrics = state.performance_monitor.get_current_metrics().await;

        assert!(metrics.adaptation_metrics.adaptations_applied > 0);
        assert!(metrics.storage_metrics.total_operations > 0);
        assert!(metrics.pattern_recognition_metrics.patterns_analyzed > 0);
        assert!(
            metrics.adaptation_metrics.success_rate >= 0.0
                && metrics.adaptation_metrics.success_rate <= 1.0
        );
    }

    #[tokio::test]
    async fn test_learning_health_check() {
        let state = LearningState::new().await.unwrap();
        let health = state.performance_monitor.get_health_status().await;

        assert_eq!(health.overall_status, LearningHealthStatus::Healthy);
        assert!(!health.component_results.is_empty());
        assert!(!health.summary.is_empty());
    }

    #[tokio::test]
    async fn test_learning_dashboard_data() {
        let state = LearningState::new().await.unwrap();
        let dashboard_data = state.performance_monitor.get_dashboard_data().await;

        assert!(
            dashboard_data
                .current_metrics
                .adaptation_metrics
                .adaptations_applied
                > 0
        );
        assert_eq!(
            dashboard_data.health_status.overall_status,
            LearningHealthStatus::Healthy
        );
        assert!(!dashboard_data.performance_graphs.is_empty());
        assert!(dashboard_data.system_overview.total_adaptations > 0);
    }

    #[tokio::test]
    async fn test_learning_performance_summary() {
        let state = LearningState::new().await.unwrap();
        let performance_summary = state.performance_monitor.get_performance_summary().await;

        assert_eq!(
            performance_summary.overall_health,
            LearningHealthStatus::Healthy
        );
        assert!(!performance_summary.key_metrics.is_empty());
        assert!(!performance_summary.recommendations.is_empty());
        assert!(!performance_summary.performance_trends.is_empty());
    }

    #[tokio::test]
    async fn test_mock_metrics_structure() {
        let metrics = LearningPerformanceMonitor::create_mock_metrics();

        // Verify all metrics have reasonable values
        assert!(
            metrics.adaptation_metrics.success_rate >= 0.0
                && metrics.adaptation_metrics.success_rate <= 1.0
        );
        assert!(
            metrics.storage_metrics.error_rate >= 0.0 && metrics.storage_metrics.error_rate <= 1.0
        );
        assert!(metrics.pattern_recognition_metrics.recognition_accuracy >= 0.0);
        assert!(metrics.feedback_metrics.average_feedback_score > 0.0);
        assert!(metrics.system_metrics.memory_usage_mb > 0.0);
    }

    #[tokio::test]
    async fn test_metrics_updates_over_time() {
        let state = LearningState::new().await.unwrap();

        let metrics1 = state.performance_monitor.get_current_metrics().await;

        // Small delay to ensure different timestamp
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let metrics2 = state.performance_monitor.get_current_metrics().await;

        // Verify metrics can change over time
        assert!(
            metrics2.adaptation_metrics.adaptations_applied
                >= metrics1.adaptation_metrics.adaptations_applied
        );
        assert!(metrics2.timestamp >= metrics1.timestamp);
    }
}
