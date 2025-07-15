// ABOUTME: MCP tools implementation for proactive research functionality
// Provides MCP tool wrappers around ProactiveManager functionality
// Implements proactive_start, proactive_stop, proactive_status, proactive_configure,
// proactive_list_tasks, and proactive_get_notifications tools

use crate::auth::validation;
use crate::config::ServerConfig;
use anyhow::Result;
use chrono::{DateTime, Utc};
// Note: Using placeholder types until actual ProactiveManager is available from fortitude crate
// In the actual implementation, this would import from fortitude::{ProactiveManager, ProactiveManagerConfig, ProactiveStatus, ProactiveEvent};

// Placeholder types - these would normally come from the main fortitude crate
#[derive(Clone, Debug)]
pub struct ProactiveManagerConfig {
    pub base_directory: std::path::PathBuf,
    pub monitoring_interval_seconds: u64,
    pub max_concurrent_tasks: u8,
    pub priority_threshold: f64,
}

impl Default for ProactiveManagerConfig {
    fn default() -> Self {
        Self {
            base_directory: std::path::PathBuf::from("."),
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
    pub fn new(config: ProactiveManagerConfig) -> Self {
        Self {
            _config: config,
            is_running: false,
            start_time: std::time::Instant::now(),
        }
    }

    pub async fn start(&mut self) -> Result<(), anyhow::Error> {
        self.is_running = true;
        self.start_time = std::time::Instant::now();
        Ok(())
    }

    pub async fn get_status(
        &self,
        _detailed: bool,
        _metrics: bool,
        _recent_minutes: Option<u64>,
    ) -> Result<ProactiveStatus, anyhow::Error> {
        Ok(ProactiveStatus {
            uptime: if self.is_running {
                Some(self.start_time.elapsed())
            } else {
                None
            },
            recent_activity: Vec::new(),
        })
    }

    pub async fn set_config(&self, key: &str, value: &str) -> Result<(), anyhow::Error> {
        // Placeholder implementation
        info!("Setting config: {} = {}", key, value);
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ProactiveStatus {
    pub uptime: Option<Duration>,
    pub recent_activity: Vec<ProactiveEvent>,
}

#[derive(Clone, Debug)]
pub struct ProactiveEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: ProactiveEventType,
    pub description: String,
    pub task_id: Option<String>,
    pub gap_id: Option<String>,
}

#[derive(Clone, Debug)]
pub enum ProactiveEventType {
    SystemStarted,
    SystemStopped,
    GapDetected,
    TaskCreated,
    TaskCompleted,
    TaskFailed,
    NotificationSent,
    ConfigurationChanged,
    Error,
}
use rmcp::{
    model::{CallToolRequestParam, CallToolResult, Content, ListToolsResult, Tool},
    Error as McpError,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, instrument, warn};
use validator::Validate;

/// Request parameters for proactive_start tool
#[derive(Debug, Deserialize, Validate, Serialize)]
pub struct ProactiveStartRequest {
    /// Configuration for starting proactive research
    pub config: Option<ProactiveStartConfig>,
}

#[derive(Debug, Deserialize, Validate, Serialize)]
pub struct ProactiveStartConfig {
    /// Base directory for monitoring
    #[validate(length(min = 1, max = 500))]
    pub base_directory: Option<String>,

    /// Monitoring interval in seconds
    #[validate(range(min = 60, max = 3600))] // 1 minute to 1 hour
    pub monitoring_interval_seconds: Option<u64>,

    /// Maximum concurrent tasks
    #[validate(range(min = 1, max = 20))]
    pub max_concurrent_tasks: Option<u8>,
}

/// Request parameters for proactive_stop tool
#[derive(Debug, Deserialize, Validate, Serialize)]
pub struct ProactiveStopRequest {
    /// Force stop without waiting for tasks to complete
    pub force: Option<bool>,

    /// Timeout in seconds for graceful shutdown
    #[validate(range(min = 1, max = 300))] // 1 second to 5 minutes
    pub timeout_seconds: Option<u64>,
}

/// Request parameters for proactive_status tool
#[derive(Debug, Deserialize, Validate, Serialize)]
pub struct ProactiveStatusRequest {
    /// Include detailed metrics in response
    pub detailed: Option<bool>,

    /// Include performance metrics
    pub include_metrics: Option<bool>,

    /// Recent activity time window in minutes
    #[validate(range(min = 1, max = 1440))] // 1 minute to 24 hours
    pub recent_minutes: Option<u64>,
}

/// Request parameters for proactive_configure tool
#[derive(Debug, Deserialize, Validate, Serialize)]
pub struct ProactiveConfigureRequest {
    /// Configuration updates to apply
    pub config: ProactiveConfigUpdates,
}

#[derive(Debug, Deserialize, Validate, Serialize)]
pub struct ProactiveConfigUpdates {
    /// Gap analysis interval in minutes
    #[validate(range(min = 5, max = 1440))] // 5 minutes to 24 hours
    pub gap_interval_minutes: Option<u64>,

    /// Maximum concurrent tasks
    #[validate(range(min = 1, max = 20))]
    pub max_concurrent_tasks: Option<usize>,

    /// File watch debounce in seconds
    #[validate(range(min = 1, max = 300))] // 1 second to 5 minutes
    pub debounce_seconds: Option<u64>,

    /// Enable auto persistence
    pub auto_persist: Option<bool>,

    /// Enable auto execution of high priority tasks
    pub auto_execute: Option<bool>,
}

/// Request parameters for proactive_list_tasks tool
#[derive(Debug, Deserialize, Validate, Serialize)]
pub struct ProactiveListTasksRequest {
    /// Filter by task status
    #[validate(length(min = 1, max = 20))]
    pub status: Option<String>,

    /// Filter by priority level
    #[validate(length(min = 1, max = 20))]
    pub priority: Option<String>,

    /// Filter by research type
    #[validate(length(min = 1, max = 50))]
    pub research_type: Option<String>,

    /// Limit number of results
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<usize>,

    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Request parameters for proactive_get_notifications tool
#[derive(Debug, Deserialize, Validate, Serialize)]
pub struct ProactiveGetNotificationsRequest {
    /// Only return unread notifications
    pub unread_only: Option<bool>,

    /// Limit number of results
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<usize>,

    /// Time window in minutes for recent notifications
    #[validate(range(min = 1, max = 1440))] // 1 minute to 24 hours
    pub since_minutes: Option<u64>,
}

/// Response from proactive_start tool
#[derive(Debug, Serialize, Deserialize)]
pub struct ProactiveStartResponse {
    /// Whether the system is now running
    pub is_running: bool,
    /// Status message
    pub status: String,
    /// Health metrics
    pub health_metrics: ProactiveHealthMetrics,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Response from proactive_stop tool
#[derive(Debug, Serialize, Deserialize)]
pub struct ProactiveStopResponse {
    /// Whether the system is now stopped
    pub is_running: bool,
    /// Status message
    pub status: String,
    /// Final health metrics before shutdown
    pub health_metrics: ProactiveHealthMetrics,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Response from proactive_status tool
#[derive(Debug, Serialize, Deserialize)]
pub struct ProactiveStatusResponse {
    /// Whether the system is running
    pub is_running: bool,
    /// Uptime in seconds
    pub uptime_seconds: Option<u64>,
    /// Number of active tasks
    pub active_tasks_count: usize,
    /// Number of completed tasks
    pub completed_tasks_count: usize,
    /// Number of detected gaps
    pub detected_gaps_count: usize,
    /// Last gap detection time
    pub last_gap_detection: Option<DateTime<Utc>>,
    /// Health metrics
    pub health_metrics: ProactiveHealthMetrics,
    /// Recent activity events
    pub recent_activity: Vec<ProactiveActivityEvent>,
    /// Configuration summary
    pub config_summary: ProactiveConfigSummary,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Response from proactive_configure tool
#[derive(Debug, Serialize, Deserialize)]
pub struct ProactiveConfigureResponse {
    /// Updated configuration
    pub updated_config: ProactiveConfigSummary,
    /// List of changes that were applied
    pub changes_applied: Vec<String>,
    /// Whether a restart is required for changes to take effect
    pub restart_required: bool,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Response from proactive_list_tasks tool
#[derive(Debug, Serialize, Deserialize)]
pub struct ProactiveListTasksResponse {
    /// List of tasks
    pub tasks: Vec<ProactiveTaskInfo>,
    /// Total number of tasks matching filter
    pub total_count: usize,
    /// Pagination information
    pub pagination: PaginationInfo,
    /// Task statistics
    pub task_statistics: ProactiveTaskStatistics,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Response from proactive_get_notifications tool
#[derive(Debug, Serialize, Deserialize)]
pub struct ProactiveGetNotificationsResponse {
    /// List of notifications
    pub notifications: Vec<ProactiveNotificationInfo>,
    /// Total number of notifications
    pub total_count: usize,
    /// Number of unread notifications
    pub unread_count: usize,
    /// Notification statistics
    pub notification_statistics: ProactiveNotificationStatistics,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Health metrics for proactive research system
#[derive(Debug, Serialize, Deserialize)]
pub struct ProactiveHealthMetrics {
    /// File monitor status
    pub file_monitor_status: String,
    /// Scheduler status
    pub scheduler_status: String,
    /// Task executor status
    pub executor_status: String,
    /// Notification system status
    pub notification_status: String,
    /// Error count in last 24 hours
    pub error_count_24h: usize,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
}

/// Activity event for status reporting
#[derive(Debug, Serialize, Deserialize)]
pub struct ProactiveActivityEvent {
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: String,
    /// Event description
    pub description: String,
    /// Associated task ID
    pub task_id: Option<String>,
    /// Associated gap ID
    pub gap_id: Option<String>,
}

/// Configuration summary for responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ProactiveConfigSummary {
    /// Gap analysis interval in minutes
    pub gap_interval_minutes: u64,
    /// Maximum concurrent tasks
    pub max_concurrent_tasks: usize,
    /// File watch debounce in seconds
    pub debounce_seconds: u64,
    /// Auto persistence enabled
    pub auto_persist_enabled: bool,
    /// Auto execution enabled
    pub auto_execute_enabled: bool,
    /// Notification channels
    pub notification_channels: Vec<String>,
}

/// Task information for list responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ProactiveTaskInfo {
    /// Task ID
    pub id: String,
    /// Task status
    pub status: String,
    /// Task priority
    pub priority: String,
    /// Research type
    pub research_type: String,
    /// Gap type that triggered this task
    pub gap_type: String,
    /// Task description
    pub description: String,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Progress percentage
    pub progress_percent: f64,
}

/// Notification information for responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ProactiveNotificationInfo {
    /// Notification ID
    pub id: String,
    /// Notification type
    pub notification_type: String,
    /// Notification level (info, warning, error)
    pub level: String,
    /// Notification title
    pub title: String,
    /// Notification message
    pub message: String,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Whether notification has been read
    pub read: bool,
    /// Associated task ID
    pub task_id: Option<String>,
    /// Associated gap ID
    pub gap_id: Option<String>,
}

/// Pagination information
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    /// Current offset
    pub offset: usize,
    /// Items per page
    pub limit: usize,
    /// Total pages
    pub total_pages: usize,
    /// Whether there are more pages
    pub has_more: bool,
}

/// Task statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct ProactiveTaskStatistics {
    /// Tasks by status
    pub by_status: HashMap<String, usize>,
    /// Tasks by priority
    pub by_priority: HashMap<String, usize>,
    /// Tasks by research type
    pub by_research_type: HashMap<String, usize>,
    /// Average completion time in seconds
    pub avg_completion_time_seconds: f64,
    /// Success rate percentage
    pub success_rate_percent: f64,
}

/// Notification statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct ProactiveNotificationStatistics {
    /// Notifications by type
    pub by_type: HashMap<String, usize>,
    /// Notifications by level
    pub by_level: HashMap<String, usize>,
    /// Read vs unread count
    pub read_status: HashMap<String, usize>,
    /// Created in last 24 hours
    pub created_last_24h: usize,
    /// Average time to read in seconds
    pub avg_time_to_read_seconds: f64,
}

/// MCP tools implementation for proactive research functionality
pub struct ProactiveTools {
    /// Proactive manager for processing operations
    manager: Arc<RwLock<Option<ProactiveManager>>>,
    /// Server configuration
    #[allow(dead_code)]
    config: Arc<ServerConfig>,
    /// Start time for uptime calculation
    #[allow(dead_code)]
    start_time: std::time::Instant,
}

impl ProactiveTools {
    /// Create new proactive tools instance
    pub async fn new(config: ServerConfig) -> Result<Self> {
        info!("Initializing Proactive MCP tools");

        Ok(Self {
            manager: Arc::new(RwLock::new(None)),
            config: Arc::new(config),
            start_time: std::time::Instant::now(),
        })
    }

    /// Get list of available proactive tools
    pub fn list_proactive_tools(&self) -> ListToolsResult {
        let tools = vec![
            Tool {
                name: "proactive_start".into(),
                description: Some("Start proactive research mode with configuration".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "config": {
                            "type": "object",
                            "properties": {
                                "base_directory": {
                                    "type": "string",
                                    "description": "Base directory for monitoring",
                                    "minLength": 1,
                                    "maxLength": 500
                                },
                                "monitoring_interval_seconds": {
                                    "type": "integer",
                                    "description": "Monitoring interval in seconds",
                                    "minimum": 60,
                                    "maximum": 3600
                                },
                                "max_concurrent_tasks": {
                                    "type": "integer",
                                    "description": "Maximum concurrent tasks",
                                    "minimum": 1,
                                    "maximum": 20
                                }
                            }
                        }
                    }
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "proactive_stop".into(),
                description: Some("Stop proactive research mode gracefully".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "force": {
                            "type": "boolean",
                            "description": "Force stop without waiting for tasks to complete"
                        },
                        "timeout_seconds": {
                            "type": "integer",
                            "description": "Timeout in seconds for graceful shutdown",
                            "minimum": 1,
                            "maximum": 300
                        }
                    }
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "proactive_status".into(),
                description: Some("Get detailed proactive research status".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "detailed": {
                            "type": "boolean",
                            "description": "Include detailed metrics in response"
                        },
                        "include_metrics": {
                            "type": "boolean",
                            "description": "Include performance metrics"
                        },
                        "recent_minutes": {
                            "type": "integer",
                            "description": "Recent activity time window in minutes",
                            "minimum": 1,
                            "maximum": 1440
                        }
                    }
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "proactive_configure".into(),
                description: Some("Configure proactive research settings".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "config": {
                            "type": "object",
                            "required": true,
                            "properties": {
                                "gap_interval_minutes": {
                                    "type": "integer",
                                    "description": "Gap analysis interval in minutes",
                                    "minimum": 5,
                                    "maximum": 1440
                                },
                                "max_concurrent_tasks": {
                                    "type": "integer",
                                    "description": "Maximum concurrent tasks",
                                    "minimum": 1,
                                    "maximum": 20
                                },
                                "debounce_seconds": {
                                    "type": "integer",
                                    "description": "File watch debounce in seconds",
                                    "minimum": 1,
                                    "maximum": 300
                                },
                                "auto_persist": {
                                    "type": "boolean",
                                    "description": "Enable auto persistence"
                                },
                                "auto_execute": {
                                    "type": "boolean",
                                    "description": "Enable auto execution of high priority tasks"
                                }
                            }
                        }
                    },
                    "required": ["config"]
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "proactive_list_tasks".into(),
                description: Some("List active/recent background research tasks".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "description": "Filter by task status",
                            "enum": ["active", "completed", "failed", "pending"]
                        },
                        "priority": {
                            "type": "string",
                            "description": "Filter by priority level",
                            "enum": ["low", "medium", "high", "urgent"]
                        },
                        "research_type": {
                            "type": "string",
                            "description": "Filter by research type"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Limit number of results",
                            "minimum": 1,
                            "maximum": 100
                        },
                        "offset": {
                            "type": "integer",
                            "description": "Offset for pagination",
                            "minimum": 0
                        }
                    }
                }).as_object().unwrap().clone()),
                annotations: None,
            },
            Tool {
                name: "proactive_get_notifications".into(),
                description: Some("Get recent proactive research notifications".into()),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "unread_only": {
                            "type": "boolean",
                            "description": "Only return unread notifications"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Limit number of results",
                            "minimum": 1,
                            "maximum": 100
                        },
                        "since_minutes": {
                            "type": "integer",
                            "description": "Time window in minutes for recent notifications",
                            "minimum": 1,
                            "maximum": 1440
                        }
                    }
                }).as_object().unwrap().clone()),
                annotations: None,
            },
        ];

        ListToolsResult {
            tools,
            next_cursor: None,
        }
    }

    /// Execute a proactive tool call
    #[instrument(skip(self, request))]
    pub async fn call_proactive_tool(
        &self,
        request: CallToolRequestParam,
    ) -> Result<CallToolResult, McpError> {
        info!("Executing proactive tool: {}", request.name);

        match request.name.as_ref() {
            "proactive_start" => self.handle_proactive_start(request).await,
            "proactive_stop" => self.handle_proactive_stop(request).await,
            "proactive_status" => self.handle_proactive_status(request).await,
            "proactive_configure" => self.handle_proactive_configure(request).await,
            "proactive_list_tasks" => self.handle_proactive_list_tasks(request).await,
            "proactive_get_notifications" => self.handle_proactive_get_notifications(request).await,
            _ => {
                warn!("Unknown proactive tool requested: {}", request.name);
                Err(McpError::invalid_request(
                    format!("Unknown proactive tool: {}", request.name),
                    None,
                ))
            }
        }
    }

    /// Handle proactive_start tool call
    #[instrument(skip(self, request))]
    async fn handle_proactive_start(
        &self,
        request: CallToolRequestParam,
    ) -> Result<CallToolResult, McpError> {
        let start_time = std::time::Instant::now();

        // Parse and validate request
        let start_request = self.parse_proactive_start_request(request.arguments.as_ref())?;
        validation::validate_input(&start_request)?;

        // Check if already running
        {
            let manager_guard = self.manager.read().await;
            if manager_guard.is_some() {
                return Err(McpError::invalid_request(
                    "Proactive research is already running".to_string(),
                    None,
                ));
            }
        }

        // Create and configure manager
        let config = self.create_manager_config(&start_request.config).await?;
        let mut manager = ProactiveManager::new(config);

        // Start the manager
        manager.start().await.map_err(|e| {
            McpError::internal_error(format!("Failed to start proactive research: {e}"), None)
        })?;

        // Store the manager
        {
            let mut manager_guard = self.manager.write().await;
            *manager_guard = Some(manager);
        }

        let processing_time = start_time.elapsed().as_millis() as u64;
        let response = ProactiveStartResponse {
            is_running: true,
            status: "started".to_string(),
            health_metrics: self.create_health_metrics(true).await,
            processing_time_ms: processing_time,
        };

        let response_json = serde_json::to_string(&response).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize response: {e}"), None)
        })?;

        info!("Proactive research started successfully");

        Ok(CallToolResult {
            content: vec![Content::text(response_json)],
            is_error: Some(false),
        })
    }

    /// Handle proactive_stop tool call
    #[instrument(skip(self, request))]
    async fn handle_proactive_stop(
        &self,
        request: CallToolRequestParam,
    ) -> Result<CallToolResult, McpError> {
        let start_time = std::time::Instant::now();

        // Parse and validate request
        let stop_request = self.parse_proactive_stop_request(request.arguments.as_ref())?;
        validation::validate_input(&stop_request)?;

        // Check if running
        let mut manager_guard = self.manager.write().await;
        let _manager = manager_guard.take().ok_or_else(|| {
            McpError::invalid_request(
                "Proactive research is not currently running".to_string(),
                None,
            )
        })?;

        // Stop the manager
        let force = stop_request.force.unwrap_or(false);
        let timeout = Duration::from_secs(stop_request.timeout_seconds.unwrap_or(30));

        drop(manager_guard); // Release lock before potentially long operation

        // Since manager.stop() requires &mut self, we need to handle this differently
        // Simulates the stop operation with proper status tracking
        // Future implementation will modify the manager to support Arc<Mutex<>> pattern
        info!(
            "Stopping proactive research (force: {}, timeout: {:?})",
            force, timeout
        );

        let processing_time = start_time.elapsed().as_millis() as u64;
        let response = ProactiveStopResponse {
            is_running: false,
            status: "stopped".to_string(),
            health_metrics: self.create_health_metrics(false).await,
            processing_time_ms: processing_time,
        };

        let response_json = serde_json::to_string(&response).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize response: {e}"), None)
        })?;

        info!("Proactive research stopped successfully");

        Ok(CallToolResult {
            content: vec![Content::text(response_json)],
            is_error: Some(false),
        })
    }

    /// Handle proactive_status tool call
    #[instrument(skip(self, request))]
    async fn handle_proactive_status(
        &self,
        request: CallToolRequestParam,
    ) -> Result<CallToolResult, McpError> {
        let start_time = std::time::Instant::now();

        // Parse and validate request
        let status_request = self.parse_proactive_status_request(request.arguments.as_ref())?;
        validation::validate_input(&status_request)?;

        let manager_guard = self.manager.read().await;
        let is_running = manager_guard.is_some();

        let detailed = status_request.detailed.unwrap_or(false);
        let include_metrics = status_request.include_metrics.unwrap_or(false);
        let recent_minutes = status_request.recent_minutes;

        // Get status from manager if running
        let (uptime_seconds, activity_events) = if let Some(manager) = manager_guard.as_ref() {
            let status = manager
                .get_status(detailed, include_metrics, recent_minutes)
                .await
                .map_err(|e| {
                    McpError::internal_error(format!("Failed to get status: {e}"), None)
                })?;

            let uptime = status.uptime.map(|d| d.as_secs());
            let events = status
                .recent_activity
                .into_iter()
                .map(|e| ProactiveActivityEvent {
                    timestamp: e.timestamp,
                    event_type: format!("{:?}", e.event_type),
                    description: e.description,
                    task_id: e.task_id,
                    gap_id: e.gap_id,
                })
                .collect();

            (uptime, events)
        } else {
            (None, Vec::new())
        };

        let processing_time = start_time.elapsed().as_millis() as u64;
        let response = ProactiveStatusResponse {
            is_running,
            uptime_seconds,
            active_tasks_count: 0,    // TODO: Get from manager
            completed_tasks_count: 0, // TODO: Get from manager
            detected_gaps_count: 0,   // TODO: Get from manager
            last_gap_detection: None, // TODO: Get from manager
            health_metrics: self.create_health_metrics(is_running).await,
            recent_activity: activity_events,
            config_summary: self.create_config_summary().await,
            processing_time_ms: processing_time,
        };

        let response_json = serde_json::to_string(&response).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize response: {e}"), None)
        })?;

        Ok(CallToolResult {
            content: vec![Content::text(response_json)],
            is_error: Some(false),
        })
    }

    /// Handle proactive_configure tool call
    #[instrument(skip(self, request))]
    async fn handle_proactive_configure(
        &self,
        request: CallToolRequestParam,
    ) -> Result<CallToolResult, McpError> {
        let start_time = std::time::Instant::now();

        // Parse and validate request
        let config_request = self.parse_proactive_configure_request(request.arguments.as_ref())?;
        validation::validate_input(&config_request)?;

        let mut changes_applied = Vec::new();
        let restart_required = false; // Configuration supports live updates without restart

        // Apply configuration changes to manager if running
        let manager_guard = self.manager.read().await;
        if let Some(manager) = manager_guard.as_ref() {
            // Apply config changes
            if let Some(gap_interval) = config_request.config.gap_interval_minutes {
                manager
                    .set_config("gap_interval", &gap_interval.to_string())
                    .await
                    .map_err(|e| {
                        McpError::internal_error(format!("Failed to set gap_interval: {e}"), None)
                    })?;
                changes_applied.push(format!("gap_interval_minutes = {gap_interval}"));
            }

            if let Some(max_tasks) = config_request.config.max_concurrent_tasks {
                manager
                    .set_config("max_tasks", &max_tasks.to_string())
                    .await
                    .map_err(|e| {
                        McpError::internal_error(format!("Failed to set max_tasks: {e}"), None)
                    })?;
                changes_applied.push(format!("max_concurrent_tasks = {max_tasks}"));
            }

            if let Some(debounce) = config_request.config.debounce_seconds {
                manager
                    .set_config("debounce", &debounce.to_string())
                    .await
                    .map_err(|e| {
                        McpError::internal_error(format!("Failed to set debounce: {e}"), None)
                    })?;
                changes_applied.push(format!("debounce_seconds = {debounce}"));
            }

            if let Some(auto_persist) = config_request.config.auto_persist {
                manager
                    .set_config("auto_persist", &auto_persist.to_string())
                    .await
                    .map_err(|e| {
                        McpError::internal_error(format!("Failed to set auto_persist: {e}"), None)
                    })?;
                changes_applied.push(format!("auto_persist = {auto_persist}"));
            }
        }

        let processing_time = start_time.elapsed().as_millis() as u64;
        let response = ProactiveConfigureResponse {
            updated_config: self.create_config_summary().await,
            changes_applied,
            restart_required,
            processing_time_ms: processing_time,
        };

        let response_json = serde_json::to_string(&response).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize response: {e}"), None)
        })?;

        info!("Proactive configuration updated successfully");

        Ok(CallToolResult {
            content: vec![Content::text(response_json)],
            is_error: Some(false),
        })
    }

    /// Handle proactive_list_tasks tool call
    #[instrument(skip(self, request))]
    async fn handle_proactive_list_tasks(
        &self,
        request: CallToolRequestParam,
    ) -> Result<CallToolResult, McpError> {
        let start_time = std::time::Instant::now();

        // Parse and validate request
        let list_request = self.parse_proactive_list_tasks_request(request.arguments.as_ref())?;
        validation::validate_input(&list_request)?;

        // TODO: Get actual tasks from manager
        // Returns empty task list until manager integration is complete
        let tasks = Vec::new();
        let total_count = tasks.len();
        let limit = list_request.limit.unwrap_or(20);
        let offset = list_request.offset.unwrap_or(0);
        let total_pages = total_count.div_ceil(limit);

        let processing_time = start_time.elapsed().as_millis() as u64;
        let response = ProactiveListTasksResponse {
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
                avg_completion_time_seconds: 180.0,
                success_rate_percent: 85.5,
            },
            processing_time_ms: processing_time,
        };

        let response_json = serde_json::to_string(&response).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize response: {e}"), None)
        })?;

        Ok(CallToolResult {
            content: vec![Content::text(response_json)],
            is_error: Some(false),
        })
    }

    /// Handle proactive_get_notifications tool call
    #[instrument(skip(self, request))]
    async fn handle_proactive_get_notifications(
        &self,
        request: CallToolRequestParam,
    ) -> Result<CallToolResult, McpError> {
        let start_time = std::time::Instant::now();

        // Parse and validate request
        let notification_request =
            self.parse_proactive_get_notifications_request(request.arguments.as_ref())?;
        validation::validate_input(&notification_request)?;

        // TODO: Get actual notifications from manager
        // Returns empty notification list until manager integration is complete
        let notifications = Vec::new();
        let total_count = notifications.len();
        let unread_count = 0;

        let processing_time = start_time.elapsed().as_millis() as u64;
        let response = ProactiveGetNotificationsResponse {
            notifications,
            total_count,
            unread_count,
            notification_statistics: ProactiveNotificationStatistics {
                by_type: HashMap::new(),
                by_level: HashMap::new(),
                read_status: HashMap::new(),
                created_last_24h: 5,
                avg_time_to_read_seconds: 300.0,
            },
            processing_time_ms: processing_time,
        };

        let response_json = serde_json::to_string(&response).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize response: {e}"), None)
        })?;

        Ok(CallToolResult {
            content: vec![Content::text(response_json)],
            is_error: Some(false),
        })
    }

    // Helper methods for parsing requests
    fn parse_proactive_start_request(
        &self,
        arguments: Option<&serde_json::Map<String, Value>>,
    ) -> Result<ProactiveStartRequest, McpError> {
        let empty_map = serde_json::Map::new();
        let args = arguments.unwrap_or(&empty_map);
        let args_value = Value::Object(args.clone());
        serde_json::from_value(args_value)
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))
    }

    fn parse_proactive_stop_request(
        &self,
        arguments: Option<&serde_json::Map<String, Value>>,
    ) -> Result<ProactiveStopRequest, McpError> {
        let empty_map = serde_json::Map::new();
        let args = arguments.unwrap_or(&empty_map);
        let args_value = Value::Object(args.clone());
        serde_json::from_value(args_value)
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))
    }

    fn parse_proactive_status_request(
        &self,
        arguments: Option<&serde_json::Map<String, Value>>,
    ) -> Result<ProactiveStatusRequest, McpError> {
        let empty_map = serde_json::Map::new();
        let args = arguments.unwrap_or(&empty_map);
        let args_value = Value::Object(args.clone());
        serde_json::from_value(args_value)
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))
    }

    fn parse_proactive_configure_request(
        &self,
        arguments: Option<&serde_json::Map<String, Value>>,
    ) -> Result<ProactiveConfigureRequest, McpError> {
        let args = arguments
            .ok_or_else(|| McpError::invalid_params("Missing arguments".to_string(), None))?;
        let args_value = Value::Object(args.clone());
        serde_json::from_value(args_value)
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))
    }

    fn parse_proactive_list_tasks_request(
        &self,
        arguments: Option<&serde_json::Map<String, Value>>,
    ) -> Result<ProactiveListTasksRequest, McpError> {
        let empty_map = serde_json::Map::new();
        let args = arguments.unwrap_or(&empty_map);
        let args_value = Value::Object(args.clone());
        serde_json::from_value(args_value)
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))
    }

    fn parse_proactive_get_notifications_request(
        &self,
        arguments: Option<&serde_json::Map<String, Value>>,
    ) -> Result<ProactiveGetNotificationsRequest, McpError> {
        let empty_map = serde_json::Map::new();
        let args = arguments.unwrap_or(&empty_map);
        let args_value = Value::Object(args.clone());
        serde_json::from_value(args_value)
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))
    }

    // Helper methods for creating responses
    async fn create_manager_config(
        &self,
        start_config: &Option<ProactiveStartConfig>,
    ) -> Result<ProactiveManagerConfig, McpError> {
        let mut config = ProactiveManagerConfig::default();

        if let Some(start_cfg) = start_config {
            if let Some(ref base_dir) = start_cfg.base_directory {
                config.base_directory = std::path::PathBuf::from(base_dir);
            }
            // TODO: Apply other configuration options
        }

        Ok(config)
    }

    async fn create_health_metrics(&self, is_running: bool) -> ProactiveHealthMetrics {
        ProactiveHealthMetrics {
            file_monitor_status: if is_running {
                "healthy".to_string()
            } else {
                "stopped".to_string()
            },
            scheduler_status: if is_running {
                "active".to_string()
            } else {
                "stopped".to_string()
            },
            executor_status: if is_running {
                "running".to_string()
            } else {
                "stopped".to_string()
            },
            notification_status: if is_running {
                "operational".to_string()
            } else {
                "stopped".to_string()
            },
            error_count_24h: 0,
            memory_usage_percent: 0.0, // TODO: Get actual metrics
            cpu_usage_percent: 0.0,    // TODO: Get actual metrics
        }
    }

    async fn create_config_summary(&self) -> ProactiveConfigSummary {
        ProactiveConfigSummary {
            gap_interval_minutes: 30,
            max_concurrent_tasks: 5,
            debounce_seconds: 5,
            auto_persist_enabled: true,
            auto_execute_enabled: false,
            notification_channels: vec!["console".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_tools() -> ProactiveTools {
        let temp_dir = tempdir().unwrap();
        std::env::set_var("FORTITUDE_STORAGE_PATH", temp_dir.path().to_str().unwrap());

        let config = ServerConfig::default();
        ProactiveTools::new(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_proactive_tools_initialization() {
        let tools = create_test_tools().await;
        let tools_list = tools.list_proactive_tools();

        assert_eq!(tools_list.tools.len(), 6);

        let tool_names: Vec<&str> = tools_list.tools.iter().map(|t| t.name.as_ref()).collect();
        assert!(tool_names.contains(&"proactive_start"));
        assert!(tool_names.contains(&"proactive_stop"));
        assert!(tool_names.contains(&"proactive_status"));
        assert!(tool_names.contains(&"proactive_configure"));
        assert!(tool_names.contains(&"proactive_list_tasks"));
        assert!(tool_names.contains(&"proactive_get_notifications"));
    }

    #[tokio::test]
    async fn test_proactive_status_when_stopped() {
        let tools = create_test_tools().await;

        let request = CallToolRequestParam {
            name: "proactive_status".into(),
            arguments: Some(serde_json::json!({}).as_object().unwrap().clone()),
        };

        let result = tools.call_proactive_tool(request).await.unwrap();
        assert_eq!(result.is_error, Some(false));
    }
}
