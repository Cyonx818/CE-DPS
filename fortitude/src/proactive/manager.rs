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

// ABOUTME: Proactive research manager for CLI integration and component coordination
//! This module provides a high-level manager for coordinating all proactive research components.
//! It serves as the main interface between the CLI commands and the underlying proactive system,
//! handling initialization, state management, configuration, and graceful shutdown.

use crate::proactive::{
    BackgroundScheduler, BackgroundSchedulerConfig, ErrorHandler, ErrorHandlerConfig,
    ExecutorMetrics, FileMonitor, FileMonitorConfig, GapAnalysisConfig, GapAnalyzer,
    ImpactAssessmentConfig, ImpactAssessor, NotificationMetrics, NotificationSystem,
    NotificationSystemConfig, PrioritizationConfig, PriorityScorer, ProgressPerformanceMetrics,
    ProgressTracker, ProgressTrackerConfig, ResearchCompletionConfig, ResearchCompletionNotifier,
    ResearchScheduler, ResearchSchedulerConfig, SchedulerMetrics, StateManager, StateManagerConfig,
    TaskExecutor, TaskExecutorConfig, UserPreferenceManager,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, instrument};

/// Errors that can occur during proactive manager operations
#[derive(Error, Debug)]
pub enum ProactiveManagerError {
    #[error("Manager not initialized")]
    NotInitialized,

    #[error("Manager already running")]
    AlreadyRunning,

    #[error("Manager not running")]
    NotRunning,

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Component initialization failed: {component} - {error}")]
    ComponentInitialization { component: String, error: String },

    #[error("State persistence error: {0}")]
    StatePersistence(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Timeout during operation: {operation}")]
    Timeout { operation: String },

    #[error("Invalid configuration value: {key} = {value}")]
    InvalidConfigValue { key: String, value: String },
}

/// Configuration for the proactive manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveManagerConfig {
    /// File monitor configuration
    pub file_monitor: FileMonitorConfig,

    /// Gap analysis configuration
    pub gap_analysis: GapAnalysisConfig,

    /// Background scheduler configuration
    pub scheduler: BackgroundSchedulerConfig,

    /// Task executor configuration
    pub executor: TaskExecutorConfig,

    /// Research scheduler configuration
    pub research_scheduler: ResearchSchedulerConfig,

    /// State manager configuration
    pub state_manager: StateManagerConfig,

    /// Notification system configuration
    pub notification_system: NotificationSystemConfig,

    /// Progress tracker configuration
    pub progress_tracker: ProgressTrackerConfig,

    /// Research completion notifier configuration
    pub completion_notifier: ResearchCompletionConfig,

    /// Error handler configuration
    pub error_handler: ErrorHandlerConfig,

    /// Prioritization configuration
    pub prioritization: PrioritizationConfig,

    /// Impact assessment configuration
    pub impact_assessment: ImpactAssessmentConfig,

    /// Base directory for monitoring
    pub base_directory: PathBuf,

    /// Configuration persistence path
    pub config_path: Option<PathBuf>,

    /// Enable automatic state persistence
    pub auto_persist: bool,

    /// State persistence interval
    pub persist_interval: Duration,
}

impl Default for ProactiveManagerConfig {
    fn default() -> Self {
        Self {
            file_monitor: FileMonitorConfig::default(),
            gap_analysis: GapAnalysisConfig::default(),
            scheduler: BackgroundSchedulerConfig::default(),
            executor: TaskExecutorConfig::default(),
            research_scheduler: ResearchSchedulerConfig::default(),
            state_manager: StateManagerConfig::default(),
            notification_system: NotificationSystemConfig::default(),
            progress_tracker: ProgressTrackerConfig::default(),
            completion_notifier: ResearchCompletionConfig::default(),
            error_handler: ErrorHandlerConfig::default(),
            prioritization: PrioritizationConfig::default(),
            impact_assessment: ImpactAssessmentConfig::default(),
            base_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            config_path: None,
            auto_persist: true,
            persist_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Current status of the proactive research system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveStatus {
    /// Whether the system is currently running
    pub is_running: bool,

    /// System startup time
    pub started_at: Option<DateTime<Utc>>,

    /// Current system uptime
    pub uptime: Option<Duration>,

    /// Number of active tasks
    pub active_tasks: usize,

    /// Number of completed tasks
    pub completed_tasks: usize,

    /// Number of failed tasks
    pub failed_tasks: usize,

    /// Number of detected gaps
    pub detected_gaps: usize,

    /// Last gap analysis time
    pub last_gap_analysis: Option<DateTime<Utc>>,

    /// Scheduler metrics
    pub scheduler_metrics: Option<SchedulerMetrics>,

    /// Executor metrics
    pub executor_metrics: Option<ExecutorMetrics>,

    /// Notification metrics
    pub notification_metrics: Option<NotificationMetrics>,

    /// Progress metrics
    pub progress_metrics: Option<ProgressPerformanceMetrics>,

    /// Recent activity (last 10 events)
    pub recent_activity: Vec<ProactiveEvent>,

    /// Configuration summary
    pub config_summary: ConfigSummary,
}

/// Event in the proactive research system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveEvent {
    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Event type
    pub event_type: ProactiveEventType,

    /// Event description
    pub description: String,

    /// Associated task ID (if applicable)
    pub task_id: Option<String>,

    /// Associated gap ID (if applicable)
    pub gap_id: Option<String>,
}

/// Types of proactive research events
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Configuration summary for status display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSummary {
    pub gap_interval_minutes: u64,
    pub max_concurrent_tasks: usize,
    pub file_watch_debounce_seconds: u64,
    pub auto_persist_enabled: bool,
    pub notification_channels: Vec<String>,
}

/// Main proactive research manager
pub struct ProactiveManager {
    /// Manager configuration
    config: ProactiveManagerConfig,

    /// Whether the manager is currently running
    is_running: Arc<RwLock<bool>>,

    /// System startup time
    started_at: Arc<RwLock<Option<DateTime<Utc>>>>,

    /// File monitor component
    #[allow(dead_code)]
    // TODO: Modular component - will be activated in proactive research features
    file_monitor: Option<Arc<FileMonitor>>,

    /// Gap analyzer component
    #[allow(dead_code)] // TODO: Modular component - will be activated in gap analysis features
    gap_analyzer: Option<Arc<GapAnalyzer>>,

    /// Background scheduler component
    #[allow(dead_code)]
    // TODO: Modular component - will be activated in scheduled research features
    background_scheduler: Option<Arc<BackgroundScheduler>>,

    /// Task executor component
    task_executor: Option<Arc<TaskExecutor>>,

    /// Research scheduler component
    research_scheduler: Option<Arc<ResearchScheduler>>,

    /// State manager component
    #[allow(dead_code)]
    // TODO: Modular component - will be activated in state persistence features
    state_manager: Option<Arc<StateManager>>,

    /// Notification system component
    notification_system: Option<Arc<NotificationSystem>>,

    /// Progress tracker component
    progress_tracker: Option<Arc<ProgressTracker>>,

    /// Research completion notifier component
    #[allow(dead_code)] // TODO: Modular component - will be activated in notification features
    completion_notifier: Option<Arc<ResearchCompletionNotifier>>,

    /// Error handler component
    #[allow(dead_code)]
    // TODO: Modular component - will be activated in advanced error handling
    error_handler: Option<Arc<ErrorHandler>>,

    /// Priority scorer component
    #[allow(dead_code)]
    // TODO: Modular component - will be activated in priority-based research
    priority_scorer: Option<Arc<PriorityScorer>>,

    /// User preference manager component
    #[allow(dead_code)]
    // TODO: Modular component - will be activated in personalization features
    user_preferences: Option<Arc<UserPreferenceManager>>,

    /// Impact assessor component
    #[allow(dead_code)]
    // TODO: Modular component - will be activated in impact analysis features
    impact_assessor: Option<Arc<ImpactAssessor>>,

    /// Event history for status reporting
    event_history: Arc<RwLock<Vec<ProactiveEvent>>>,

    /// Shutdown signal sender
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl ProactiveManager {
    /// Create a new proactive manager with the given configuration
    pub fn new(config: ProactiveManagerConfig) -> Self {
        Self {
            config,
            is_running: Arc::new(RwLock::new(false)),
            started_at: Arc::new(RwLock::new(None)),
            file_monitor: None,
            gap_analyzer: None,
            background_scheduler: None,
            task_executor: None,
            research_scheduler: None,
            state_manager: None,
            notification_system: None,
            progress_tracker: None,
            completion_notifier: None,
            error_handler: None,
            priority_scorer: None,
            user_preferences: None,
            impact_assessor: None,
            event_history: Arc::new(RwLock::new(Vec::new())),
            shutdown_tx: None,
        }
    }

    /// Create a new proactive manager with default configuration
    pub fn with_defaults() -> Self {
        Self::new(ProactiveManagerConfig::default())
    }

    /// Start the proactive research system
    #[instrument(skip(self))]
    pub async fn start(&mut self) -> Result<(), ProactiveManagerError> {
        {
            let is_running = self.is_running.read().await;
            if *is_running {
                return Err(ProactiveManagerError::AlreadyRunning);
            }
        }

        info!("Starting proactive research manager");

        // Initialize all components
        self.initialize_components().await?;

        // Start components in dependency order
        self.start_components().await?;

        // Mark as running
        {
            let mut is_running = self.is_running.write().await;
            *is_running = true;
        }
        {
            let mut started_at = self.started_at.write().await;
            *started_at = Some(Utc::now());
        }

        // Log event
        self.log_event(
            ProactiveEventType::SystemStarted,
            "Proactive research system started".to_string(),
            None,
            None,
        )
        .await;

        info!("Proactive research manager started successfully");
        Ok(())
    }

    /// Stop the proactive research system
    #[instrument(skip(self))]
    pub async fn stop(
        &mut self,
        force: bool,
        timeout: Duration,
    ) -> Result<(), ProactiveManagerError> {
        {
            let is_running = self.is_running.read().await;
            if !*is_running {
                return Err(ProactiveManagerError::NotRunning);
            }
        }

        info!(
            "Stopping proactive research manager (force: {}, timeout: {:?})",
            force, timeout
        );

        // Send shutdown signal
        if let Some(shutdown_tx) = &self.shutdown_tx {
            let _ = shutdown_tx.send(());
        }

        if force {
            // Force stop all components immediately
            self.force_stop_components().await?;
        } else {
            // Graceful stop with timeout
            tokio::time::timeout(timeout, self.graceful_stop_components())
                .await
                .map_err(|_| ProactiveManagerError::Timeout {
                    operation: "graceful_stop".to_string(),
                })??;
        }

        // Mark as stopped
        {
            let mut is_running = self.is_running.write().await;
            *is_running = false;
        }
        {
            let mut started_at = self.started_at.write().await;
            *started_at = None;
        }

        // Log event
        self.log_event(
            ProactiveEventType::SystemStopped,
            "Proactive research system stopped".to_string(),
            None,
            None,
        )
        .await;

        info!("Proactive research manager stopped successfully");
        Ok(())
    }

    /// Get the current status of the proactive research system
    #[instrument(skip(self))]
    pub async fn get_status(
        &self,
        detailed: bool,
        metrics: bool,
        recent_minutes: Option<u64>,
    ) -> Result<ProactiveStatus, ProactiveManagerError> {
        let is_running = *self.is_running.read().await;
        let started_at = *self.started_at.read().await;

        let uptime = started_at.map(|start| {
            let now = Utc::now();
            (now - start).to_std().unwrap_or(Duration::ZERO)
        });

        // Get metrics from components if requested
        let scheduler_metrics = if metrics && self.research_scheduler.is_some() {
            // TODO: Implement metrics collection
            None
        } else {
            None
        };

        let executor_metrics = if metrics && self.task_executor.is_some() {
            // TODO: Implement metrics collection
            None
        } else {
            None
        };

        let notification_metrics = if metrics && self.notification_system.is_some() {
            // TODO: Implement metrics collection
            None
        } else {
            None
        };

        let progress_metrics = if metrics && self.progress_tracker.is_some() {
            // TODO: Implement metrics collection
            None
        } else {
            None
        };

        // Get recent activity
        let event_history = self.event_history.read().await;
        let recent_activity = if let Some(minutes) = recent_minutes {
            let cutoff = Utc::now() - chrono::Duration::minutes(minutes as i64);
            event_history
                .iter()
                .filter(|event| event.timestamp >= cutoff)
                .cloned()
                .collect()
        } else {
            event_history.iter().rev().take(10).cloned().collect()
        };

        // Create configuration summary
        let config_summary = ConfigSummary {
            gap_interval_minutes: 30, // TODO: Get from actual config
            max_concurrent_tasks: self.config.executor.max_concurrent_tasks,
            file_watch_debounce_seconds: 5, // TODO: Get from actual config
            auto_persist_enabled: self.config.auto_persist,
            notification_channels: vec!["console".to_string()], // TODO: Get from actual config
        };

        Ok(ProactiveStatus {
            is_running,
            started_at,
            uptime,
            active_tasks: 0,         // TODO: Get from state manager
            completed_tasks: 0,      // TODO: Get from state manager
            failed_tasks: 0,         // TODO: Get from state manager
            detected_gaps: 0,        // TODO: Get from gap analyzer
            last_gap_analysis: None, // TODO: Get from gap analyzer
            scheduler_metrics,
            executor_metrics,
            notification_metrics,
            progress_metrics,
            recent_activity,
            config_summary,
        })
    }

    /// Update configuration setting
    #[instrument(skip(self))]
    pub async fn set_config(
        &mut self,
        key: &str,
        value: &str,
    ) -> Result<(), ProactiveManagerError> {
        // Validate and set configuration values
        match key {
            "gap_interval" => {
                let minutes: u64 =
                    value
                        .parse()
                        .map_err(|_| ProactiveManagerError::InvalidConfigValue {
                            key: key.to_string(),
                            value: value.to_string(),
                        })?;
                if minutes == 0 || minutes > 1440 {
                    // 0 to 24 hours
                    return Err(ProactiveManagerError::InvalidConfigValue {
                        key: key.to_string(),
                        value: value.to_string(),
                    });
                }
                // TODO: Update actual configuration
                info!("Set gap_interval to {} minutes", minutes);
            }
            "max_tasks" => {
                let tasks: usize =
                    value
                        .parse()
                        .map_err(|_| ProactiveManagerError::InvalidConfigValue {
                            key: key.to_string(),
                            value: value.to_string(),
                        })?;
                if tasks == 0 || tasks > 50 {
                    return Err(ProactiveManagerError::InvalidConfigValue {
                        key: key.to_string(),
                        value: value.to_string(),
                    });
                }
                self.config.executor.max_concurrent_tasks = tasks;
                info!("Set max_tasks to {}", tasks);
            }
            "debounce" => {
                let seconds: u64 =
                    value
                        .parse()
                        .map_err(|_| ProactiveManagerError::InvalidConfigValue {
                            key: key.to_string(),
                            value: value.to_string(),
                        })?;
                if seconds > 300 {
                    // Max 5 minutes
                    return Err(ProactiveManagerError::InvalidConfigValue {
                        key: key.to_string(),
                        value: value.to_string(),
                    });
                }
                // TODO: Update actual configuration
                info!("Set debounce to {} seconds", seconds);
            }
            "auto_persist" => {
                let enabled: bool =
                    value
                        .parse()
                        .map_err(|_| ProactiveManagerError::InvalidConfigValue {
                            key: key.to_string(),
                            value: value.to_string(),
                        })?;
                self.config.auto_persist = enabled;
                info!("Set auto_persist to {}", enabled);
            }
            _ => {
                return Err(ProactiveManagerError::InvalidConfigValue {
                    key: key.to_string(),
                    value: value.to_string(),
                });
            }
        }

        // Log configuration change
        self.log_event(
            ProactiveEventType::ConfigurationChanged,
            format!("Configuration changed: {key} = {value}"),
            None,
            None,
        )
        .await;

        // Persist configuration if enabled
        if self.config.auto_persist {
            self.persist_config().await?;
        }

        Ok(())
    }

    /// Get configuration value
    pub async fn get_config(&self, key: &str) -> Result<String, ProactiveManagerError> {
        match key {
            "gap_interval" => Ok("30".to_string()), // TODO: Get from actual config
            "max_tasks" => Ok(self.config.executor.max_concurrent_tasks.to_string()),
            "debounce" => Ok("5".to_string()), // TODO: Get from actual config
            "auto_persist" => Ok(self.config.auto_persist.to_string()),
            _ => Err(ProactiveManagerError::InvalidConfigValue {
                key: key.to_string(),
                value: "unknown key".to_string(),
            }),
        }
    }

    /// List all configuration values
    pub async fn list_config(&self) -> Result<HashMap<String, String>, ProactiveManagerError> {
        let mut config = HashMap::new();
        config.insert("gap_interval".to_string(), "30".to_string()); // TODO: Get from actual config
        config.insert(
            "max_tasks".to_string(),
            self.config.executor.max_concurrent_tasks.to_string(),
        );
        config.insert("debounce".to_string(), "5".to_string()); // TODO: Get from actual config
        config.insert(
            "auto_persist".to_string(),
            self.config.auto_persist.to_string(),
        );
        Ok(config)
    }

    /// Reset configuration to defaults
    pub async fn reset_config(&mut self) -> Result<(), ProactiveManagerError> {
        self.config = ProactiveManagerConfig::default();

        // Log configuration reset
        self.log_event(
            ProactiveEventType::ConfigurationChanged,
            "Configuration reset to defaults".to_string(),
            None,
            None,
        )
        .await;

        // Persist configuration if enabled
        if self.config.auto_persist {
            self.persist_config().await?;
        }

        info!("Configuration reset to defaults");
        Ok(())
    }

    /// Load configuration from file
    pub async fn load_config(&mut self, path: &PathBuf) -> Result<(), ProactiveManagerError> {
        let content = tokio::fs::read_to_string(path).await?;
        self.config = serde_json::from_str(&content)?;

        info!("Configuration loaded from {:?}", path);
        Ok(())
    }

    /// Persist configuration to file
    async fn persist_config(&self) -> Result<(), ProactiveManagerError> {
        if let Some(config_path) = &self.config.config_path {
            let content = serde_json::to_string_pretty(&self.config)?;
            tokio::fs::write(config_path, content).await?;
            debug!("Configuration persisted to {:?}", config_path);
        }
        Ok(())
    }

    /// Initialize all components
    async fn initialize_components(&mut self) -> Result<(), ProactiveManagerError> {
        // TODO: Initialize all components with proper error handling
        // This is a placeholder implementation
        info!("Initializing proactive research components");

        // Create shutdown channel
        let (shutdown_tx, _) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        Ok(())
    }

    /// Start all components in dependency order
    async fn start_components(&mut self) -> Result<(), ProactiveManagerError> {
        // TODO: Start components in proper dependency order
        info!("Starting proactive research components");
        Ok(())
    }

    /// Gracefully stop all components
    async fn graceful_stop_components(&mut self) -> Result<(), ProactiveManagerError> {
        // TODO: Implement graceful component shutdown
        info!("Gracefully stopping proactive research components");
        Ok(())
    }

    /// Force stop all components
    async fn force_stop_components(&mut self) -> Result<(), ProactiveManagerError> {
        // TODO: Implement force component shutdown
        info!("Force stopping proactive research components");
        Ok(())
    }

    /// Log an event to the event history
    async fn log_event(
        &self,
        event_type: ProactiveEventType,
        description: String,
        task_id: Option<String>,
        gap_id: Option<String>,
    ) {
        let event = ProactiveEvent {
            timestamp: Utc::now(),
            event_type,
            description,
            task_id,
            gap_id,
        };

        let mut history = self.event_history.write().await;
        history.push(event);

        // Keep only last 1000 events
        if history.len() > 1000 {
            history.remove(0);
        }
    }
}
