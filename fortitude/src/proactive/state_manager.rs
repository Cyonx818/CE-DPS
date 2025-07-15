// ABOUTME: Centralized task state management system with comprehensive lifecycle tracking
//! This module provides a centralized state management system for research tasks that enhances
//! the existing state management with comprehensive lifecycle tracking, persistence, recovery,
//! and monitoring capabilities. Features include:
//! - Enhanced state transition validation with metadata tracking
//! - Comprehensive state history and audit trails
//! - Persistent state storage with atomic operations
//! - Recovery mechanisms for system failures and stale tasks
//! - Real-time state monitoring and event emission
//! - Integration with existing task queue, executor, and scheduler

use crate::proactive::{ResearchTask, TaskPriority, TaskState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::fs;
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Errors that can occur during state management operations
#[derive(Error, Debug)]
pub enum StateManagerError {
    #[error("Invalid state transition from {from:?} to {to:?}: {reason}")]
    InvalidStateTransition {
        from: TaskState,
        to: TaskState,
        reason: String,
    },

    #[error("Task not found: {task_id}")]
    TaskNotFound { task_id: String },

    #[error("State persistence error: {0}")]
    Persistence(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Concurrent modification detected for task: {task_id}")]
    ConcurrentModification { task_id: String },

    #[error("State lock timeout for task: {task_id}")]
    LockTimeout { task_id: String },

    #[error("State manager not running")]
    NotRunning,

    #[error("Recovery operation failed: {0}")]
    RecoveryFailed(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

/// State transition metadata with comprehensive tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransitionMetadata {
    /// Reason for the state change
    pub reason: String,
    /// Actor that initiated the change (e.g., "scheduler", "executor", "user")
    pub actor: String,
    /// Additional context data
    pub additional_data: HashMap<String, String>,
    /// Previous state for validation
    pub previous_state: Option<TaskState>,
    /// Transition validation rules applied
    pub validation_rules: Vec<String>,
}

/// Complete state change entry with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChangeEntry {
    /// Unique identifier for this state change
    pub id: String,
    /// Task identifier
    pub task_id: String,
    /// Previous state
    pub from_state: TaskState,
    /// New state
    pub to_state: TaskState,
    /// Timestamp of the change
    pub timestamp: DateTime<Utc>,
    /// Transition metadata
    pub metadata: StateTransitionMetadata,
    /// Duration in previous state
    pub duration_in_previous_state: Option<Duration>,
    /// Validation results
    pub validation_passed: bool,
    /// Error message if validation failed
    pub validation_error: Option<String>,
}

/// Task lifecycle tracking with complete history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskLifecycle {
    /// Task identifier
    pub task_id: String,
    /// Task creation timestamp
    pub created_at: DateTime<Utc>,
    /// Current state
    pub current_state: TaskState,
    /// All state transitions
    pub transitions: Vec<StateChangeEntry>,
    /// Total time in each state
    pub state_durations: HashMap<TaskState, Duration>,
    /// Lifecycle metadata
    pub metadata: HashMap<String, String>,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

/// State manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateManagerConfig {
    /// File for persisting state data
    pub persistence_file: PathBuf,
    /// Interval for automatic persistence
    pub persistence_interval: Duration,
    /// Maximum state history entries to keep per task
    pub max_history_entries: usize,
    /// Enable real-time state monitoring
    pub enable_monitoring: bool,
    /// State validation rules
    pub validation_rules: StateValidationRules,
    /// Recovery configuration
    pub recovery_config: StateRecoveryConfig,
    /// Monitoring configuration
    pub monitoring_config: StateMonitoringConfig,
}

impl Default for StateManagerConfig {
    fn default() -> Self {
        Self {
            persistence_file: PathBuf::from("state_manager.json"),
            persistence_interval: Duration::from_secs(10),
            max_history_entries: 100,
            enable_monitoring: true,
            validation_rules: StateValidationRules::default(),
            recovery_config: StateRecoveryConfig::default(),
            monitoring_config: StateMonitoringConfig::default(),
        }
    }
}

/// State validation rules configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateValidationRules {
    /// Strict validation mode (reject invalid transitions)
    pub strict_mode: bool,
    /// Custom transition rules
    pub custom_rules: HashMap<String, String>,
    /// Timeout for tasks in executing state
    pub executing_timeout: Duration,
    /// Require metadata for certain transitions
    pub require_metadata: Vec<String>,
}

impl Default for StateValidationRules {
    fn default() -> Self {
        Self {
            strict_mode: true,
            custom_rules: HashMap::new(),
            executing_timeout: Duration::from_secs(600), // 10 minutes
            require_metadata: vec!["manual_transition".to_string()],
        }
    }
}

/// Recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateRecoveryConfig {
    /// Enable automatic recovery on startup
    pub enable_auto_recovery: bool,
    /// Maximum age for tasks to be considered stale
    pub stale_task_threshold: Duration,
    /// Recovery strategy for stale executing tasks
    pub stale_executing_strategy: StaleTaskStrategy,
    /// Recovery strategy for orphaned tasks
    pub orphaned_task_strategy: OrphanedTaskStrategy,
}

impl Default for StateRecoveryConfig {
    fn default() -> Self {
        Self {
            enable_auto_recovery: true,
            stale_task_threshold: Duration::from_secs(3600), // 1 hour
            stale_executing_strategy: StaleTaskStrategy::ResetToPending,
            orphaned_task_strategy: OrphanedTaskStrategy::RetryWithBackoff,
        }
    }
}

/// Strategy for handling stale executing tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StaleTaskStrategy {
    ResetToPending,
    MarkAsFailed,
    Ignore,
}

/// Strategy for handling orphaned tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrphanedTaskStrategy {
    RetryWithBackoff,
    MarkAsFailed,
    RequeueWithLowPriority,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMonitoringConfig {
    /// Enable event broadcasting
    pub enable_events: bool,
    /// Monitoring interval
    pub monitoring_interval: Duration,
    /// Metrics collection configuration
    pub metrics_config: MetricsConfig,
}

impl Default for StateMonitoringConfig {
    fn default() -> Self {
        Self {
            enable_events: true,
            monitoring_interval: Duration::from_secs(5),
            metrics_config: MetricsConfig::default(),
        }
    }
}

/// Metrics collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Track state transition latencies
    pub track_transition_latencies: bool,
    /// Track state distribution
    pub track_state_distribution: bool,
    /// Track error rates
    pub track_error_rates: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            track_transition_latencies: true,
            track_state_distribution: true,
            track_error_rates: true,
        }
    }
}

/// State management events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateEvent {
    StateTransition {
        task_id: String,
        from_state: TaskState,
        to_state: TaskState,
        timestamp: DateTime<Utc>,
        metadata: StateTransitionMetadata,
    },
    TaskCreated {
        task_id: String,
        timestamp: DateTime<Utc>,
        priority: TaskPriority,
    },
    TaskRecovered {
        task_id: String,
        previous_state: TaskState,
        recovered_state: TaskState,
        timestamp: DateTime<Utc>,
    },
    ValidationFailed {
        task_id: String,
        attempted_transition: (TaskState, TaskState),
        reason: String,
        timestamp: DateTime<Utc>,
    },
}

/// State manager metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateManagerMetrics {
    /// Total state transitions processed
    pub total_transitions: u64,
    /// Successful transitions
    pub successful_transitions: u64,
    /// Failed transitions
    pub failed_transitions: u64,
    /// Recovery operations performed
    pub recovery_operations: u64,
    /// Tasks by current state
    pub tasks_by_state: HashMap<TaskState, u64>,
    /// Average transition latency
    pub average_transition_latency: Duration,
    /// Error rate
    pub error_rate: f64,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

impl Default for StateManagerMetrics {
    fn default() -> Self {
        Self {
            total_transitions: 0,
            successful_transitions: 0,
            failed_transitions: 0,
            recovery_operations: 0,
            tasks_by_state: HashMap::new(),
            average_transition_latency: Duration::from_millis(0),
            error_rate: 0.0,
            last_updated: Utc::now(),
        }
    }
}

/// Persistence format for state manager data
#[derive(Debug, Serialize, Deserialize)]
struct StateManagerPersistence {
    task_lifecycles: HashMap<String, TaskLifecycle>,
    metrics: StateManagerMetrics,
    version: u32,
    last_persistence: DateTime<Utc>,
}

/// Centralized state manager with comprehensive lifecycle tracking
#[derive(Debug)]
pub struct StateManager {
    /// Configuration
    config: StateManagerConfig,
    /// Task lifecycles storage
    task_lifecycles: Arc<RwLock<HashMap<String, TaskLifecycle>>>,
    /// State change locks for concurrent access
    state_locks: Arc<RwLock<HashMap<String, Arc<Mutex<()>>>>>,
    /// Event broadcaster
    event_sender: broadcast::Sender<StateEvent>,
    /// Metrics tracking
    metrics: Arc<RwLock<StateManagerMetrics>>,
    /// Running state
    running: Arc<RwLock<bool>>,
    /// Last persistence timestamp
    last_persistence: Arc<Mutex<Instant>>,
}

impl StateManager {
    /// Create a new state manager with the given configuration
    #[instrument(level = "debug", skip(config))]
    pub async fn new(config: StateManagerConfig) -> Result<Self, StateManagerError> {
        info!(
            "Initializing state manager with persistence file: {}",
            config.persistence_file.display()
        );

        // Validate configuration
        Self::validate_config(&config)?;

        let (event_sender, _) = broadcast::channel(1000);

        let state_manager = Self {
            config,
            task_lifecycles: Arc::new(RwLock::new(HashMap::new())),
            state_locks: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            metrics: Arc::new(RwLock::new(StateManagerMetrics::default())),
            running: Arc::new(RwLock::new(false)),
            last_persistence: Arc::new(Mutex::new(Instant::now())),
        };

        // Load existing state if file exists
        if state_manager.config.persistence_file.exists() {
            state_manager.load_state().await?;
        }

        debug!("State manager initialized successfully");
        Ok(state_manager)
    }

    /// Start the state manager
    #[instrument(level = "debug", skip(self))]
    pub async fn start(&self) -> Result<(), StateManagerError> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        info!("Starting state manager");

        // Start monitoring if enabled
        if self.config.enable_monitoring {
            self.start_monitoring().await;
        }

        // Start periodic persistence
        self.start_periodic_persistence().await;

        // Perform recovery if enabled
        if self.config.recovery_config.enable_auto_recovery {
            self.perform_recovery().await?;
        }

        info!("State manager started successfully");
        Ok(())
    }

    /// Stop the state manager
    #[instrument(level = "debug", skip(self))]
    pub async fn stop(&self) -> Result<(), StateManagerError> {
        let mut running = self.running.write().await;
        if !*running {
            return Ok(());
        }
        *running = false;
        drop(running);

        info!("Stopping state manager");

        // Final persistence
        self.persist_state().await?;

        info!("State manager stopped");
        Ok(())
    }

    /// Check if state manager is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Track task creation
    #[instrument(level = "debug", skip(self, task))]
    pub async fn track_task_creation(&self, task: &ResearchTask) -> Result<(), StateManagerError> {
        let lifecycle = TaskLifecycle {
            task_id: task.id.clone(),
            created_at: task.created_at,
            current_state: task.state.clone(),
            transitions: Vec::new(),
            state_durations: HashMap::new(),
            metadata: HashMap::new(),
            last_updated: Utc::now(),
        };

        let mut lifecycles = self.task_lifecycles.write().await;
        lifecycles.insert(task.id.clone(), lifecycle);

        // Emit event
        if self.config.monitoring_config.enable_events {
            let _ = self.event_sender.send(StateEvent::TaskCreated {
                task_id: task.id.clone(),
                timestamp: Utc::now(),
                priority: task.priority,
            });
        }

        debug!("Task creation tracked: {}", task.id);
        Ok(())
    }

    /// Transition task state with metadata and validation
    #[instrument(level = "debug", skip(self, metadata))]
    pub async fn transition_task(
        &self,
        task_id: &str,
        new_state: TaskState,
        actor: &str,
        metadata: Option<StateTransitionMetadata>,
    ) -> Result<(), StateManagerError> {
        if !*self.running.read().await {
            return Err(StateManagerError::NotRunning);
        }

        // Acquire task-specific lock to prevent concurrent modifications
        let lock = self.get_task_lock(task_id).await;
        let _guard = lock.lock().await;

        let start_time = Instant::now();

        // Get current lifecycle
        let mut lifecycles = self.task_lifecycles.write().await;
        let lifecycle =
            lifecycles
                .get_mut(task_id)
                .ok_or_else(|| StateManagerError::TaskNotFound {
                    task_id: task_id.to_string(),
                })?;

        let current_state = lifecycle.current_state.clone();

        // Validate state transition
        self.validate_state_transition(&current_state, &new_state, task_id)?;

        // Calculate duration in previous state
        let duration_in_previous_state = if let Some(last_transition) = lifecycle.transitions.last()
        {
            Some(
                Utc::now()
                    .signed_duration_since(last_transition.timestamp)
                    .to_std()
                    .unwrap_or(Duration::from_secs(0)),
            )
        } else {
            Some(
                Utc::now()
                    .signed_duration_since(lifecycle.created_at)
                    .to_std()
                    .unwrap_or(Duration::from_secs(0)),
            )
        };

        // Create state change entry
        let change_entry = StateChangeEntry {
            id: Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            from_state: current_state.clone(),
            to_state: new_state.clone(),
            timestamp: Utc::now(),
            metadata: metadata.unwrap_or_else(|| StateTransitionMetadata {
                reason: "Automatic transition".to_string(),
                actor: actor.to_string(),
                additional_data: HashMap::new(),
                previous_state: Some(current_state.clone()),
                validation_rules: vec!["basic_validation".to_string()],
            }),
            duration_in_previous_state,
            validation_passed: true,
            validation_error: None,
        };

        // Update lifecycle
        lifecycle.current_state = new_state.clone();
        lifecycle.transitions.push(change_entry.clone());
        lifecycle.last_updated = Utc::now();

        // Update state durations
        if let Some(duration) = duration_in_previous_state {
            *lifecycle
                .state_durations
                .entry(current_state.clone())
                .or_insert(Duration::from_secs(0)) += duration;
        }

        // Trim history if needed
        if lifecycle.transitions.len() > self.config.max_history_entries {
            lifecycle.transitions.remove(0);
        }

        drop(lifecycles);

        // Update metrics
        let transition_latency = start_time.elapsed();
        self.update_metrics_for_transition(&current_state, &new_state, transition_latency, true)
            .await;

        // Emit event
        if self.config.monitoring_config.enable_events {
            let _ = self.event_sender.send(StateEvent::StateTransition {
                task_id: task_id.to_string(),
                from_state: current_state.clone(),
                to_state: new_state.clone(),
                timestamp: change_entry.timestamp,
                metadata: change_entry.metadata.clone(),
            });
        }

        // Persist if needed
        self.persist_if_needed().await?;

        debug!("State transition completed: {} -> {:?}", task_id, new_state);
        Ok(())
    }

    /// Get task lifecycle information
    pub async fn get_task_lifecycle(
        &self,
        task_id: &str,
    ) -> Result<TaskLifecycle, StateManagerError> {
        let lifecycles = self.task_lifecycles.read().await;
        lifecycles
            .get(task_id)
            .cloned()
            .ok_or_else(|| StateManagerError::TaskNotFound {
                task_id: task_id.to_string(),
            })
    }

    /// Get task state history
    pub async fn get_task_state_history(
        &self,
        task_id: &str,
    ) -> Result<Vec<StateChangeEntry>, StateManagerError> {
        let lifecycle = self.get_task_lifecycle(task_id).await?;
        Ok(lifecycle.transitions)
    }

    /// Get tasks by current state
    pub async fn get_tasks_by_state(&self, state: TaskState) -> Vec<String> {
        let lifecycles = self.task_lifecycles.read().await;
        lifecycles
            .values()
            .filter(|lifecycle| lifecycle.current_state == state)
            .map(|lifecycle| lifecycle.task_id.clone())
            .collect()
    }

    /// Count tasks by state
    pub async fn count_tasks_by_state(&self, state: TaskState) -> u64 {
        let lifecycles = self.task_lifecycles.read().await;
        lifecycles
            .values()
            .filter(|lifecycle| lifecycle.current_state == state)
            .count() as u64
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> StateManagerMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Subscribe to state events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<StateEvent> {
        self.event_sender.subscribe()
    }

    /// Perform recovery operations
    #[instrument(level = "debug", skip(self))]
    pub async fn perform_recovery(&self) -> Result<u64, StateManagerError> {
        info!("Performing state recovery operations");

        let mut recovery_count = 0;
        let now = Utc::now();
        let stale_threshold =
            chrono::Duration::from_std(self.config.recovery_config.stale_task_threshold)
                .map_err(|e| StateManagerError::RecoveryFailed(e.to_string()))?;

        let mut lifecycles = self.task_lifecycles.write().await;

        for lifecycle in lifecycles.values_mut() {
            let last_update_age = now.signed_duration_since(lifecycle.last_updated);

            // Handle stale executing tasks
            if lifecycle.current_state == TaskState::Executing && last_update_age > stale_threshold
            {
                match self.config.recovery_config.stale_executing_strategy {
                    StaleTaskStrategy::ResetToPending => {
                        lifecycle.current_state = TaskState::Pending;
                        recovery_count += 1;

                        // Emit recovery event
                        if self.config.monitoring_config.enable_events {
                            let _ = self.event_sender.send(StateEvent::TaskRecovered {
                                task_id: lifecycle.task_id.clone(),
                                previous_state: TaskState::Executing,
                                recovered_state: TaskState::Pending,
                                timestamp: now,
                            });
                        }
                    }
                    StaleTaskStrategy::MarkAsFailed => {
                        lifecycle.current_state = TaskState::Failed;
                        recovery_count += 1;
                    }
                    StaleTaskStrategy::Ignore => {
                        // Do nothing
                    }
                }
            }
        }

        drop(lifecycles);

        // Update recovery metrics
        let mut metrics = self.metrics.write().await;
        metrics.recovery_operations += recovery_count;
        metrics.last_updated = now;

        info!("Recovery completed: {} tasks recovered", recovery_count);
        Ok(recovery_count)
    }

    /// Validate configuration
    fn validate_config(config: &StateManagerConfig) -> Result<(), StateManagerError> {
        if config.max_history_entries == 0 {
            return Err(StateManagerError::Configuration(
                "max_history_entries must be greater than 0".to_string(),
            ));
        }

        if config.persistence_interval.as_millis() == 0 {
            return Err(StateManagerError::Configuration(
                "persistence_interval must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate state transition
    fn validate_state_transition(
        &self,
        from_state: &TaskState,
        to_state: &TaskState,
        task_id: &str,
    ) -> Result<(), StateManagerError> {
        // Basic validation using existing method
        if !from_state.can_transition_to(to_state) {
            let reason = format!(
                "Invalid transition from {from_state:?} to {to_state:?} for task {task_id}"
            );

            if self.config.validation_rules.strict_mode {
                return Err(StateManagerError::InvalidStateTransition {
                    from: from_state.clone(),
                    to: to_state.clone(),
                    reason,
                });
            } else {
                warn!("Non-strict mode: allowing invalid transition: {}", reason);
            }
        }

        Ok(())
    }

    /// Get or create task-specific lock
    async fn get_task_lock(&self, task_id: &str) -> Arc<Mutex<()>> {
        let mut locks = self.state_locks.write().await;
        locks
            .entry(task_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    /// Update metrics for a state transition
    async fn update_metrics_for_transition(
        &self,
        from_state: &TaskState,
        to_state: &TaskState,
        latency: Duration,
        success: bool,
    ) {
        let mut metrics = self.metrics.write().await;

        metrics.total_transitions += 1;

        if success {
            metrics.successful_transitions += 1;
        } else {
            metrics.failed_transitions += 1;
        }

        // Update state counts
        *metrics.tasks_by_state.entry(to_state.clone()).or_insert(0) += 1;
        if let Some(count) = metrics.tasks_by_state.get_mut(from_state) {
            if *count > 0 {
                *count -= 1;
            }
        }

        // Update average latency
        if metrics.total_transitions > 0 {
            let total_latency = metrics
                .average_transition_latency
                .mul_f64(metrics.total_transitions as f64 - 1.0)
                + latency;
            metrics.average_transition_latency =
                total_latency.div_f64(metrics.total_transitions as f64);
        }

        // Update error rate
        metrics.error_rate = metrics.failed_transitions as f64 / metrics.total_transitions as f64;

        metrics.last_updated = Utc::now();
    }

    /// Start monitoring background task
    async fn start_monitoring(&self) {
        let running = self.running.clone();
        let metrics = self.metrics.clone();
        let task_lifecycles = self.task_lifecycles.clone();
        let monitoring_interval = self.config.monitoring_config.monitoring_interval;

        tokio::spawn(async move {
            let mut interval = interval(monitoring_interval);

            while *running.read().await {
                interval.tick().await;

                // Update state distribution metrics
                let lifecycles = task_lifecycles.read().await;
                let mut state_counts = HashMap::new();

                for lifecycle in lifecycles.values() {
                    *state_counts
                        .entry(lifecycle.current_state.clone())
                        .or_insert(0) += 1;
                }

                drop(lifecycles);

                // Update metrics
                let mut metrics_guard = metrics.write().await;
                metrics_guard.tasks_by_state = state_counts;
                metrics_guard.last_updated = Utc::now();
            }
        });
    }

    /// Start periodic persistence
    async fn start_periodic_persistence(&self) {
        let running = self.running.clone();
        let persistence_interval = self.config.persistence_interval;
        let state_manager = self.clone();

        tokio::spawn(async move {
            let mut interval = interval(persistence_interval);

            while *running.read().await {
                interval.tick().await;

                if let Err(e) = state_manager.persist_state().await {
                    error!("Periodic persistence failed: {}", e);
                }
            }
        });
    }

    /// Persist state to file
    #[instrument(level = "debug", skip(self))]
    async fn persist_state(&self) -> Result<(), StateManagerError> {
        let start = Instant::now();

        let task_lifecycles = {
            let lifecycles = self.task_lifecycles.read().await;
            lifecycles.clone()
        };

        let metrics = {
            let metrics = self.metrics.read().await;
            metrics.clone()
        };

        let persistence_data = StateManagerPersistence {
            task_lifecycles,
            metrics,
            version: 1,
            last_persistence: Utc::now(),
        };

        // Write to temporary file first, then rename for atomicity
        let temp_file = self.config.persistence_file.with_extension("tmp");
        let json_data = serde_json::to_string_pretty(&persistence_data)?;

        fs::write(&temp_file, json_data).await?;
        fs::rename(&temp_file, &self.config.persistence_file).await?;

        // Update last persistence time
        {
            let mut last_persistence = self.last_persistence.lock().await;
            *last_persistence = Instant::now();
        }

        let duration = start.elapsed();
        debug!("State persisted successfully in {:?}", duration);
        Ok(())
    }

    /// Load state from file
    #[instrument(level = "debug", skip(self))]
    async fn load_state(&self) -> Result<(), StateManagerError> {
        if !self.config.persistence_file.exists() {
            debug!("State file does not exist, starting with empty state");
            return Ok(());
        }

        let start = Instant::now();

        match fs::read_to_string(&self.config.persistence_file).await {
            Ok(content) => {
                match serde_json::from_str::<StateManagerPersistence>(&content) {
                    Ok(persistence_data) => {
                        // Restore task lifecycles
                        {
                            let mut lifecycles = self.task_lifecycles.write().await;
                            *lifecycles = persistence_data.task_lifecycles;
                        }

                        // Restore metrics
                        {
                            let mut metrics = self.metrics.write().await;
                            *metrics = persistence_data.metrics;
                        }

                        let duration = start.elapsed();
                        info!("State loaded successfully in {:?}", duration);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to parse state file, starting with empty state: {}",
                            e
                        );
                    }
                }
            }
            Err(e) => {
                warn!(
                    "Failed to read state file, starting with empty state: {}",
                    e
                );
            }
        }

        Ok(())
    }

    /// Persist state if interval has passed
    async fn persist_if_needed(&self) -> Result<(), StateManagerError> {
        let should_persist = {
            let last_persistence = self.last_persistence.lock().await;
            last_persistence.elapsed() >= self.config.persistence_interval
        };

        if should_persist {
            self.persist_state().await?;
        }

        Ok(())
    }
}

impl Clone for StateManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            task_lifecycles: self.task_lifecycles.clone(),
            state_locks: self.state_locks.clone(),
            event_sender: self.event_sender.clone(),
            metrics: self.metrics.clone(),
            running: self.running.clone(),
            last_persistence: self.last_persistence.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proactive::{DetectedGap, GapType};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_task() -> ResearchTask {
        let gap = DetectedGap {
            gap_type: GapType::TodoComment,
            file_path: PathBuf::from("test.rs"),
            line_number: 42,
            column_number: Some(10),
            context: "// TODO: Test implementation".to_string(),
            description: "Test TODO comment".to_string(),
            confidence: 0.9,
            priority: 7,
            metadata: HashMap::new(),
        };
        ResearchTask::from_gap(gap, TaskPriority::High)
    }

    #[tokio::test]
    async fn test_state_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = StateManagerConfig {
            persistence_file: temp_dir.path().join("test_state.json"),
            ..StateManagerConfig::default()
        };

        let state_manager = StateManager::new(config).await.unwrap();
        assert!(!state_manager.is_running().await);
    }

    #[tokio::test]
    async fn test_task_lifecycle_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let config = StateManagerConfig {
            persistence_file: temp_dir.path().join("test_state.json"),
            ..StateManagerConfig::default()
        };

        let state_manager = StateManager::new(config).await.unwrap();
        state_manager.start().await.unwrap();

        let task = create_test_task();
        let task_id = task.id.clone();

        // Track task creation
        state_manager.track_task_creation(&task).await.unwrap();

        // Verify lifecycle was created
        let lifecycle = state_manager.get_task_lifecycle(&task_id).await.unwrap();
        assert_eq!(lifecycle.task_id, task_id);
        assert_eq!(lifecycle.current_state, TaskState::Pending);
        assert_eq!(lifecycle.transitions.len(), 0);

        state_manager.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_state_transitions() {
        let temp_dir = TempDir::new().unwrap();
        let config = StateManagerConfig {
            persistence_file: temp_dir.path().join("test_state.json"),
            ..StateManagerConfig::default()
        };

        let state_manager = StateManager::new(config).await.unwrap();
        state_manager.start().await.unwrap();

        let task = create_test_task();
        let task_id = task.id.clone();

        // Track task creation
        state_manager.track_task_creation(&task).await.unwrap();

        // Transition to executing
        state_manager
            .transition_task(&task_id, TaskState::Executing, "test", None)
            .await
            .unwrap();

        // Verify transition
        let lifecycle = state_manager.get_task_lifecycle(&task_id).await.unwrap();
        assert_eq!(lifecycle.current_state, TaskState::Executing);
        assert_eq!(lifecycle.transitions.len(), 1);

        let transition = &lifecycle.transitions[0];
        assert_eq!(transition.from_state, TaskState::Pending);
        assert_eq!(transition.to_state, TaskState::Executing);

        state_manager.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_invalid_state_transition() {
        let temp_dir = TempDir::new().unwrap();
        let config = StateManagerConfig {
            persistence_file: temp_dir.path().join("test_state.json"),
            validation_rules: StateValidationRules {
                strict_mode: true,
                ..StateValidationRules::default()
            },
            ..StateManagerConfig::default()
        };

        let state_manager = StateManager::new(config).await.unwrap();
        state_manager.start().await.unwrap();

        let task = create_test_task();
        let task_id = task.id.clone();

        // Track task creation
        state_manager.track_task_creation(&task).await.unwrap();

        // Try invalid transition (Pending -> Completed)
        let result = state_manager
            .transition_task(&task_id, TaskState::Completed, "test", None)
            .await;
        assert!(result.is_err());

        if let Err(StateManagerError::InvalidStateTransition { from, to, .. }) = result {
            assert_eq!(from, TaskState::Pending);
            assert_eq!(to, TaskState::Completed);
        } else {
            panic!("Expected InvalidStateTransition error");
        }

        state_manager.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_state_persistence_and_recovery() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("test_state.json");

        let config = StateManagerConfig {
            persistence_file: state_file.clone(),
            ..StateManagerConfig::default()
        };

        let task_id = {
            let state_manager = StateManager::new(config.clone()).await.unwrap();
            state_manager.start().await.unwrap();

            let task = create_test_task();
            let task_id = task.id.clone();

            // Track task and transition
            state_manager.track_task_creation(&task).await.unwrap();
            state_manager
                .transition_task(&task_id, TaskState::Executing, "test", None)
                .await
                .unwrap();

            // Force persistence
            state_manager.persist_state().await.unwrap();
            state_manager.stop().await.unwrap();

            task_id
        };

        // Create new state manager and verify recovery
        {
            let state_manager = StateManager::new(config).await.unwrap();

            let lifecycle = state_manager.get_task_lifecycle(&task_id).await.unwrap();
            assert_eq!(lifecycle.current_state, TaskState::Executing);
            assert_eq!(lifecycle.transitions.len(), 1);
        }
    }

    #[tokio::test]
    async fn test_metrics_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let config = StateManagerConfig {
            persistence_file: temp_dir.path().join("test_state.json"),
            ..StateManagerConfig::default()
        };

        let state_manager = StateManager::new(config).await.unwrap();
        state_manager.start().await.unwrap();

        let task = create_test_task();
        let task_id = task.id.clone();

        // Track task and perform transitions
        state_manager.track_task_creation(&task).await.unwrap();
        state_manager
            .transition_task(&task_id, TaskState::Executing, "test", None)
            .await
            .unwrap();
        state_manager
            .transition_task(&task_id, TaskState::Completed, "test", None)
            .await
            .unwrap();

        // Check metrics
        let metrics = state_manager.get_metrics().await;
        assert_eq!(metrics.total_transitions, 2);
        assert_eq!(metrics.successful_transitions, 2);
        assert_eq!(metrics.failed_transitions, 0);
        assert!(metrics.average_transition_latency > Duration::from_nanos(0));

        state_manager.stop().await.unwrap();
    }
}
