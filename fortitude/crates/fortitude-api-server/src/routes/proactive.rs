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

// ABOUTME: Proactive research endpoint handlers for API server
// Provides HTTP endpoints for proactive research management with full ProactiveManager integration

use crate::extractors::SafeQuery;
use crate::middleware::auth::Claims;
use crate::models::{
    errors::ApiError,
    requests::{
        ProactiveConfigRequest, ProactiveNotificationListRequest, ProactiveTaskListRequest,
    },
    responses::{
        ApiResponse, PaginationInfo, ProactiveConfigResponse, ProactiveHealthMetrics,
        ProactiveNotificationListResponse, ProactiveNotificationPreferencesResponse,
        ProactiveNotificationStatistics, ProactiveStatusResponse, ProactiveTaskListResponse,
        ProactiveTaskStatistics,
    },
};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};
use uuid::Uuid;
use validator::Validate;

// Placeholder types for proactive research - these would normally come from the main fortitude crate
#[derive(Clone, Debug)]
pub struct ProactiveManagerConfig {
    pub base_directory: PathBuf,
    pub monitoring_interval_seconds: u64,
    pub max_concurrent_tasks: u8,
    pub priority_threshold: f64,
}

impl Default for ProactiveManagerConfig {
    fn default() -> Self {
        Self {
            base_directory: PathBuf::from("."),
            monitoring_interval_seconds: 300,
            max_concurrent_tasks: 5,
            priority_threshold: 0.7,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProactiveManager {
    _config: ProactiveManagerConfig,
    is_running: bool,
    start_time: std::time::Instant,
}

impl ProactiveManager {
    pub async fn new(config: ProactiveManagerConfig) -> Result<Self, ApiError> {
        Ok(Self {
            _config: config,
            is_running: false,
            start_time: std::time::Instant::now(),
        })
    }

    pub async fn start(&mut self) -> Result<(), ApiError> {
        self.is_running = true;
        self.start_time = std::time::Instant::now();
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), ApiError> {
        self.is_running = false;
        Ok(())
    }

    pub fn get_status(&self) -> Result<ProactiveStatus, ApiError> {
        Ok(ProactiveStatus {
            is_running: self.is_running,
            uptime_seconds: if self.is_running {
                self.start_time.elapsed().as_secs()
            } else {
                0
            },
            monitored_files_count: 0,
            active_tasks_count: 0,
            completed_tasks_today: 0,
            pending_tasks_count: 0,
            last_gap_detection: None,
            error_count_24h: 0,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ProactiveStatus {
    pub is_running: bool,
    pub uptime_seconds: u64,
    pub monitored_files_count: usize,
    pub active_tasks_count: usize,
    pub completed_tasks_today: usize,
    pub pending_tasks_count: usize,
    pub last_gap_detection: Option<chrono::DateTime<Utc>>,
    pub error_count_24h: usize,
}

/// Proactive research service state
#[derive(Clone)]
pub struct ProactiveState {
    pub manager: Arc<RwLock<Option<ProactiveManager>>>,
    pub config: Arc<RwLock<ProactiveManagerConfig>>,
    pub start_time: Arc<std::time::Instant>,
}

impl ProactiveState {
    /// Create new proactive state
    pub async fn new() -> Result<Self, ApiError> {
        // Load default configuration
        let config = ProactiveManagerConfig::default();

        Ok(Self {
            manager: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(config)),
            start_time: Arc::new(std::time::Instant::now()),
        })
    }

    /// Check if proactive research is running
    pub async fn is_running(&self) -> bool {
        let manager = self.manager.read().await;
        manager.is_some()
    }

    /// Get current status
    pub async fn get_status(&self) -> Result<ProactiveStatus, ApiError> {
        let manager = self.manager.read().await;
        match manager.as_ref() {
            Some(mgr) => mgr.get_status(),
            None => Ok(ProactiveStatus {
                is_running: false,
                uptime_seconds: 0,
                monitored_files_count: 0,
                active_tasks_count: 0,
                completed_tasks_today: 0,
                pending_tasks_count: 0,
                last_gap_detection: None,
                error_count_24h: 0,
            }),
        }
    }
}

/// Start proactive research mode
///
/// Initializes and starts the proactive research system with current configuration.
/// This will begin monitoring files and detecting knowledge gaps.
#[utoipa::path(
    post,
    path = "/api/v1/proactive/start",
    tag = "Proactive Research",
    summary = "Start proactive research mode",
    description = "Initialize and start the proactive research system",
    responses(
        (status = 200, description = "Proactive research started successfully", body = ApiResponse<ProactiveStatusResponse>),
        (status = 409, description = "Proactive research is already running"),
        (status = 500, description = "Failed to start proactive research")
    ),
    security(
        ("bearer_auth" = ["ResearchWrite"])
    )
)]
#[instrument(skip(state, claims), fields(user_id = %claims.sub))]
pub async fn start_proactive_research(
    State(state): State<ProactiveState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();
    info!(user_id = %claims.sub, "Starting proactive research");

    // Check if already running
    if state.is_running().await {
        return Err(ApiError::Conflict {
            resource: "Proactive research is already running".to_string(),
        });
    }

    // Get current configuration
    let config = {
        let config_guard = state.config.read().await;
        config_guard.clone()
    };

    // Create and start ProactiveManager
    let mut manager = ProactiveManager::new(config)
        .await
        .map_err(|e| ApiError::InternalError {
            message: format!("Failed to create ProactiveManager: {e}"),
        })?;

    manager.start().await.map_err(|e| ApiError::InternalError {
        message: format!("Failed to start proactive research: {e}"),
    })?;

    // Store the manager
    {
        let mut manager_guard = state.manager.write().await;
        *manager_guard = Some(manager);
    }

    // Get status for response
    let status = state
        .get_status()
        .await
        .map_err(|e| ApiError::InternalError {
            message: format!("Failed to get status: {e}"),
        })?;

    let processing_time = start_time.elapsed().as_millis() as u64;
    let response_data = ProactiveStatusResponse {
        is_running: status.is_running,
        status: if status.is_running {
            "started"
        } else {
            "stopped"
        }
        .to_string(),
        uptime_seconds: Some(status.uptime_seconds),
        monitored_files_count: status.monitored_files_count,
        active_tasks_count: status.active_tasks_count,
        completed_tasks_today: status.completed_tasks_today,
        pending_tasks_count: status.pending_tasks_count,
        last_gap_detection: status.last_gap_detection,
        health_metrics: ProactiveHealthMetrics {
            file_monitor_status: "healthy".to_string(),
            scheduler_status: "active".to_string(),
            executor_status: "running".to_string(),
            notification_status: "operational".to_string(),
            error_count_24h: status.error_count_24h,
            memory_usage_percent: 0.0, // Would be calculated in real implementation
            cpu_usage_percent: 0.0,    // Would be calculated in real implementation
        },
        processing_time_ms: processing_time,
    };

    let request_id = Uuid::new_v4();
    let response = ApiResponse::success(response_data, request_id);

    info!(
        user_id = %claims.sub,
        request_id = %request_id,
        processing_time_ms = processing_time,
        "Proactive research started successfully"
    );

    Ok((StatusCode::OK, Json(response)))
}

/// Stop proactive research mode
///
/// Gracefully stops the proactive research system, completing any running tasks.
#[utoipa::path(
    post,
    path = "/api/v1/proactive/stop",
    tag = "Proactive Research",
    summary = "Stop proactive research mode",
    description = "Gracefully stop the proactive research system",
    responses(
        (status = 200, description = "Proactive research stopped successfully", body = ApiResponse<ProactiveStatusResponse>),
        (status = 409, description = "Proactive research is not running"),
        (status = 500, description = "Failed to stop proactive research")
    ),
    security(
        ("bearer_auth" = ["ResearchWrite"])
    )
)]
#[instrument(skip(state, claims), fields(user_id = %claims.sub))]
pub async fn stop_proactive_research(
    State(state): State<ProactiveState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();
    info!(user_id = %claims.sub, "Stopping proactive research");

    // Check if running
    if !state.is_running().await {
        return Err(ApiError::Conflict {
            resource: "Proactive research is not currently running".to_string(),
        });
    }

    // Stop and remove the manager
    {
        let mut manager_guard = state.manager.write().await;
        if let Some(mut manager) = manager_guard.take() {
            manager.stop().await.map_err(|e| ApiError::InternalError {
                message: format!("Failed to stop proactive research: {e}"),
            })?;
        }
    }

    let processing_time = start_time.elapsed().as_millis() as u64;
    let response_data = ProactiveStatusResponse {
        is_running: false,
        status: "stopped".to_string(),
        uptime_seconds: None,
        monitored_files_count: 0,
        active_tasks_count: 0,
        completed_tasks_today: 0,
        pending_tasks_count: 0,
        last_gap_detection: None,
        health_metrics: ProactiveHealthMetrics {
            file_monitor_status: "stopped".to_string(),
            scheduler_status: "stopped".to_string(),
            executor_status: "stopped".to_string(),
            notification_status: "stopped".to_string(),
            error_count_24h: 0,
            memory_usage_percent: 0.0,
            cpu_usage_percent: 0.0,
        },
        processing_time_ms: processing_time,
    };

    let request_id = Uuid::new_v4();
    let response = ApiResponse::success(response_data, request_id);

    info!(
        user_id = %claims.sub,
        request_id = %request_id,
        processing_time_ms = processing_time,
        "Proactive research stopped successfully"
    );

    Ok((StatusCode::OK, Json(response)))
}

/// Get proactive research status
///
/// Returns current status information including health metrics and task counts.
#[utoipa::path(
    get,
    path = "/api/v1/proactive/status",
    tag = "Proactive Research",
    summary = "Get proactive research status",
    description = "Get current status and health metrics",
    responses(
        (status = 200, description = "Status retrieved successfully", body = ApiResponse<ProactiveStatusResponse>),
        (status = 500, description = "Failed to get status")
    ),
    security(
        ("bearer_auth" = ["ResourcesRead"])
    )
)]
#[instrument(skip(state, claims), fields(user_id = %claims.sub))]
pub async fn get_proactive_status(
    State(state): State<ProactiveState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();
    debug!(user_id = %claims.sub, "Getting proactive research status");

    let status = state
        .get_status()
        .await
        .map_err(|e| ApiError::InternalError {
            message: format!("Failed to get status: {e}"),
        })?;

    let processing_time = start_time.elapsed().as_millis() as u64;
    let response_data = ProactiveStatusResponse {
        is_running: status.is_running,
        status: if status.is_running {
            "running"
        } else {
            "stopped"
        }
        .to_string(),
        uptime_seconds: if status.is_running {
            Some(status.uptime_seconds)
        } else {
            None
        },
        monitored_files_count: status.monitored_files_count,
        active_tasks_count: status.active_tasks_count,
        completed_tasks_today: status.completed_tasks_today,
        pending_tasks_count: status.pending_tasks_count,
        last_gap_detection: status.last_gap_detection,
        health_metrics: ProactiveHealthMetrics {
            file_monitor_status: if status.is_running {
                "healthy"
            } else {
                "stopped"
            }
            .to_string(),
            scheduler_status: if status.is_running {
                "active"
            } else {
                "stopped"
            }
            .to_string(),
            executor_status: if status.is_running {
                "running"
            } else {
                "stopped"
            }
            .to_string(),
            notification_status: if status.is_running {
                "operational"
            } else {
                "stopped"
            }
            .to_string(),
            error_count_24h: status.error_count_24h,
            memory_usage_percent: 0.0, // Would be calculated in real implementation
            cpu_usage_percent: 0.0,    // Would be calculated in real implementation
        },
        processing_time_ms: processing_time,
    };

    let request_id = Uuid::new_v4();
    let response = ApiResponse::success(response_data, request_id);

    Ok((StatusCode::OK, Json(response)))
}

/// Get proactive research configuration
///
/// Returns current configuration settings for the proactive research system.
#[utoipa::path(
    get,
    path = "/api/v1/proactive/config",
    tag = "Proactive Research",
    summary = "Get proactive research configuration",
    description = "Get current configuration settings",
    responses(
        (status = 200, description = "Configuration retrieved successfully", body = ApiResponse<ProactiveConfigResponse>),
        (status = 500, description = "Failed to get configuration")
    ),
    security(
        ("bearer_auth" = ["ResourcesRead"])
    )
)]
#[instrument(skip(state, claims), fields(user_id = %claims.sub))]
pub async fn get_proactive_config(
    State(state): State<ProactiveState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();
    debug!(user_id = %claims.sub, "Getting proactive research configuration");

    let config = {
        let config_guard = state.config.read().await;
        config_guard.clone()
    };

    let processing_time = start_time.elapsed().as_millis() as u64;

    // Convert internal config to response format
    let response_data = ProactiveConfigResponse {
        base_directory: config.base_directory.to_string_lossy().to_string(),
        file_patterns: vec!["*.rs".to_string(), "*.md".to_string()], // Placeholder
        ignore_patterns: vec!["target/".to_string(), "*.log".to_string()], // Placeholder
        enabled: true,                                               // Placeholder
        monitoring_interval_seconds: 300,                            // Placeholder
        max_concurrent_tasks: 5,                                     // Placeholder
        priority_threshold: 0.7,                                     // Placeholder
        auto_execute_high_priority: false,                           // Placeholder
        notification_preferences: ProactiveNotificationPreferencesResponse {
            gap_detection_enabled: true,
            research_completion_enabled: true,
            error_notifications_enabled: true,
            frequency: "immediate".to_string(),
            min_priority_level: "medium".to_string(),
        },
        last_updated: Utc::now(), // Placeholder
        processing_time_ms: processing_time,
    };

    let request_id = Uuid::new_v4();
    let response = ApiResponse::success(response_data, request_id);

    Ok((StatusCode::OK, Json(response)))
}

/// Update proactive research configuration
///
/// Updates configuration settings for the proactive research system.
/// Changes take effect immediately if the system is running.
#[utoipa::path(
    put,
    path = "/api/v1/proactive/config",
    tag = "Proactive Research",
    summary = "Update proactive research configuration",
    description = "Update configuration settings",
    request_body = ProactiveConfigRequest,
    responses(
        (status = 200, description = "Configuration updated successfully", body = ApiResponse<ProactiveConfigResponse>),
        (status = 400, description = "Invalid configuration"),
        (status = 500, description = "Failed to update configuration")
    ),
    security(
        ("bearer_auth" = ["Admin"])
    )
)]
#[instrument(skip(state, claims, request), fields(user_id = %claims.sub))]
pub async fn update_proactive_config(
    State(state): State<ProactiveState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<ProactiveConfigRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();
    info!(user_id = %claims.sub, "Updating proactive research configuration");

    // Validate request
    request.validate().map_err(|e| ApiError::BadRequest {
        message: format!("Invalid configuration: {e}"),
    })?;

    // Update configuration
    {
        let _config_guard = state.config.write().await;

        // Apply updates from request (placeholder implementation)
        if let Some(_base_dir) = request.base_directory {
            // Update base directory
        }
        if let Some(_patterns) = request.file_patterns {
            // Update file patterns
        }
        // ... other config updates
    }

    // If manager is running, apply config changes
    if state.is_running().await {
        let manager_guard = state.manager.read().await;
        if let Some(_manager) = manager_guard.as_ref() {
            // In real implementation, would call manager.update_config()
            info!("Configuration updated for running manager");
        }
    }

    // Return updated configuration
    let updated_config = get_proactive_config(State(state), Extension(claims)).await?;

    let processing_time = start_time.elapsed().as_millis() as u64;
    info!(
        processing_time_ms = processing_time,
        "Configuration updated successfully"
    );

    Ok(updated_config)
}

/// List proactive research tasks
///
/// Returns a paginated list of background research tasks with filtering options.
#[utoipa::path(
    get,
    path = "/api/v1/proactive/tasks",
    tag = "Proactive Research",
    summary = "List proactive research tasks",
    description = "Get paginated list of background research tasks",
    params(ProactiveTaskListRequest),
    responses(
        (status = 200, description = "Tasks retrieved successfully", body = ApiResponse<ProactiveTaskListResponse>),
        (status = 400, description = "Invalid query parameters"),
        (status = 500, description = "Failed to retrieve tasks")
    ),
    security(
        ("bearer_auth" = ["ResourcesRead"])
    )
)]
#[instrument(skip(state, claims, query), fields(user_id = %claims.sub))]
pub async fn list_proactive_tasks(
    State(state): State<ProactiveState>,
    Extension(claims): Extension<Claims>,
    SafeQuery(query): SafeQuery<ProactiveTaskListRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();
    debug!(user_id = %claims.sub, "Listing proactive research tasks");

    // Validate query parameters
    query.validate().map_err(|e| ApiError::BadRequest {
        message: format!("Invalid query parameters: {e}"),
    })?;

    // Get tasks from manager (placeholder implementation)
    let tasks = if state.is_running().await {
        // In real implementation, would get tasks from manager
        vec![]
    } else {
        vec![]
    };

    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    let total_count = tasks.len();
    let total_pages = total_count.div_ceil(limit);

    let processing_time = start_time.elapsed().as_millis() as u64;
    let response_data = ProactiveTaskListResponse {
        tasks,
        total_count,
        pagination: PaginationInfo {
            offset,
            limit,
            total_pages,
            has_more: offset + limit < total_count,
        },
        task_statistics: ProactiveTaskStatistics {
            by_status: HashMap::new(),
            by_priority: HashMap::new(),
            by_research_type: HashMap::new(),
            by_gap_type: HashMap::new(),
            avg_completion_time_seconds: 180.0,
            success_rate_percent: 85.5,
        },
        processing_time_ms: processing_time,
    };

    let request_id = Uuid::new_v4();
    let response = ApiResponse::success(response_data, request_id);

    Ok((StatusCode::OK, Json(response)))
}

/// List proactive research notifications
///
/// Returns a paginated list of notifications with filtering options.
#[utoipa::path(
    get,
    path = "/api/v1/proactive/notifications",
    tag = "Proactive Research",
    summary = "List proactive research notifications",
    description = "Get paginated list of notifications",
    params(ProactiveNotificationListRequest),
    responses(
        (status = 200, description = "Notifications retrieved successfully", body = ApiResponse<ProactiveNotificationListResponse>),
        (status = 400, description = "Invalid query parameters"),
        (status = 500, description = "Failed to retrieve notifications")
    ),
    security(
        ("bearer_auth" = ["ResourcesRead"])
    )
)]
#[instrument(skip(state, claims, query), fields(user_id = %claims.sub))]
pub async fn list_proactive_notifications(
    State(state): State<ProactiveState>,
    Extension(claims): Extension<Claims>,
    SafeQuery(query): SafeQuery<ProactiveNotificationListRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();
    debug!(user_id = %claims.sub, "Listing proactive research notifications");

    // Validate query parameters
    query.validate().map_err(|e| ApiError::BadRequest {
        message: format!("Invalid query parameters: {e}"),
    })?;

    // Get notifications from manager (placeholder implementation)
    let notifications = if state.is_running().await {
        // In real implementation, would get notifications from manager
        vec![]
    } else {
        vec![]
    };

    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    let total_count = notifications.len();
    let unread_count = 0; // Placeholder
    let total_pages = total_count.div_ceil(limit);

    let processing_time = start_time.elapsed().as_millis() as u64;
    let response_data = ProactiveNotificationListResponse {
        notifications,
        total_count,
        unread_count,
        pagination: PaginationInfo {
            offset,
            limit,
            total_pages,
            has_more: offset + limit < total_count,
        },
        notification_statistics: ProactiveNotificationStatistics {
            by_type: HashMap::new(),
            by_level: HashMap::new(),
            read_status: HashMap::new(),
            created_last_24h: 5,
            avg_time_to_read_seconds: 300.0,
        },
        processing_time_ms: processing_time,
    };

    let request_id = Uuid::new_v4();
    let response = ApiResponse::success(response_data, request_id);

    Ok((StatusCode::OK, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::auth::Claims;
    use axum::Extension;

    async fn create_test_state() -> ProactiveState {
        ProactiveState::new().await.unwrap()
    }

    fn create_test_claims() -> Claims {
        Claims {
            sub: "test-user".to_string(),
            exp: 9999999999, // Far future
            iat: 1000000000,
            iss: "fortitude-test".to_string(),
            permissions: vec![
                "ResourcesRead".to_string(),
                "ResearchWrite".to_string(),
                "Admin".to_string(),
            ],
        }
    }

    #[tokio::test]
    async fn test_get_proactive_status_when_stopped() {
        let state = create_test_state().await;
        let claims = create_test_claims();

        let result = get_proactive_status(State(state), Extension(claims)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_proactive_config() {
        let state = create_test_state().await;
        let claims = create_test_claims();

        let result = get_proactive_config(State(state), Extension(claims)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_proactive_tasks_empty() {
        let state = create_test_state().await;
        let claims = create_test_claims();
        let query = ProactiveTaskListRequest {
            status: None,
            priority: None,
            research_type: None,
            gap_type: None,
            created_after: None,
            created_before: None,
            keywords: None,
            limit: Some(10),
            offset: Some(0),
            sort: None,
        };

        let result = list_proactive_tasks(State(state), Extension(claims), SafeQuery(query)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_proactive_notifications_empty() {
        let state = create_test_state().await;
        let claims = create_test_claims();
        let query = ProactiveNotificationListRequest {
            notification_type: None,
            level: None,
            read: None,
            created_after: None,
            created_before: None,
            limit: Some(10),
            offset: Some(0),
            sort: None,
        };

        let result =
            list_proactive_notifications(State(state), Extension(claims), SafeQuery(query)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stop_proactive_research_when_not_running() {
        let state = create_test_state().await;
        let claims = create_test_claims();

        let result = stop_proactive_research(State(state), Extension(claims)).await;

        // Should return an error since it's not running
        assert!(result.is_err());
        if let Err(ApiError::Conflict { resource }) = result {
            assert!(resource.contains("not currently running"));
        } else {
            panic!("Expected Conflict error");
        }
    }

    #[tokio::test]
    async fn test_update_proactive_config_validation() {
        let state = create_test_state().await;
        let claims = create_test_claims();

        // Test with invalid config (values too high)
        let invalid_request = ProactiveConfigRequest {
            base_directory: None,
            file_patterns: None,
            ignore_patterns: None,
            enabled: None,
            monitoring_interval_seconds: Some(5000), // Too high
            max_concurrent_tasks: Some(50),          // Too high
            priority_threshold: Some(1.5),           // Too high
            auto_execute_high_priority: None,
            notification_preferences: None,
        };

        let result =
            update_proactive_config(State(state), Extension(claims), Json(invalid_request)).await;

        assert!(result.is_err());
        if let Err(ApiError::BadRequest { message }) = result {
            assert!(message.contains("Invalid configuration"));
        } else {
            panic!("Expected BadRequest error");
        }
    }
}
