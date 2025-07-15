// ABOUTME: Background task scheduler with persistent queue and priority ordering for proactive research
//! This module provides a persistent task queue system with priority ordering for managing
//! background research tasks. Features include:
//! - Priority-based task ordering with configurable weights
//! - Persistent storage that survives application restarts
//! - Atomic queue operations with thread safety
//! - Task state management (pending, executing, completed, failed)
//! - Integration with gap analysis results from Task 1
//! - Performance optimizations for high-throughput scenarios

use crate::proactive::{
    error_handler::{ErrorHandler, ProactiveError},
    DetectedGap, EnhancedDetectedGap, GapType, StateManager, StateTransitionMetadata,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::fs;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Errors that can occur during background scheduling operations
#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("Queue is full: cannot add more tasks (limit: {limit})")]
    QueueFull { limit: usize },

    #[error("Task not found: {task_id}")]
    TaskNotFound { task_id: String },

    #[error("Invalid task state transition from {from:?} to {to:?}")]
    InvalidStateTransition { from: TaskState, to: TaskState },

    #[error("Persistence error: {0}")]
    Persistence(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Task timeout: task {task_id} exceeded timeout of {timeout:?}")]
    TaskTimeout { task_id: String, timeout: Duration },

    #[error("Concurrency limit exceeded: cannot start task, {current} of {limit} already running")]
    ConcurrencyLimit { current: usize, limit: usize },

    #[error("Queue operation failed: {operation} - {error}")]
    QueueOperation { operation: String, error: String },
}

/// Priority levels for research tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 1,
    Medium = 5,
    High = 9,
    Critical = 10,
}

impl TaskPriority {
    /// Create priority from numeric value (1-10 scale)
    pub fn from_u8(value: u8) -> Self {
        match value {
            1..=2 => TaskPriority::Low,
            3..=6 => TaskPriority::Medium,
            7..=9 => TaskPriority::High,
            _ => TaskPriority::Critical,
        }
    }

    /// Get numeric value for priority
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}

/// Task execution state
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskState {
    Pending,
    Executing,
    Completed,
    Failed,
    Cancelled,
}

impl TaskState {
    /// Check if transition to new state is valid
    pub fn can_transition_to(&self, new_state: &TaskState) -> bool {
        match (self, new_state) {
            (TaskState::Pending, TaskState::Executing) => true,
            (TaskState::Pending, TaskState::Cancelled) => true,
            (TaskState::Executing, TaskState::Completed) => true,
            (TaskState::Executing, TaskState::Failed) => true,
            (TaskState::Executing, TaskState::Cancelled) => true,
            // Allow re-queueing failed tasks
            (TaskState::Failed, TaskState::Pending) => true,
            _ => false,
        }
    }
}

/// Research task for background execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchTask {
    /// Unique task identifier
    pub id: String,
    /// Gap information that triggered this research task
    pub gap: DetectedGap,
    /// Task priority for ordering
    pub priority: TaskPriority,
    /// Current execution state
    pub state: TaskState,
    /// Task creation timestamp
    pub created_at: DateTime<Utc>,
    /// Task execution start timestamp
    pub started_at: Option<DateTime<Utc>>,
    /// Task completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Maximum retry attempts allowed
    pub max_retries: u32,
    /// Task execution timeout
    pub timeout: Duration,
    /// Additional task metadata
    pub metadata: HashMap<String, String>,
    /// Research query or prompt for execution
    pub research_query: String,
    /// Expected completion duration estimate
    pub estimated_duration: Option<Duration>,
}

impl ResearchTask {
    /// Create a new research task from a detected gap
    pub fn from_gap(gap: DetectedGap, priority: TaskPriority) -> Self {
        let query = Self::generate_research_query(&gap);

        Self {
            id: Uuid::new_v4().to_string(),
            gap,
            priority,
            state: TaskState::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            retry_count: 0,
            max_retries: 3,
            timeout: Duration::from_secs(300), // 5 minutes default
            metadata: HashMap::new(),
            research_query: query,
            estimated_duration: None,
        }
    }

    /// Create a research task from an enhanced detected gap
    pub fn from_enhanced_gap(
        enhanced_gap: EnhancedDetectedGap,
        custom_timeout: Option<Duration>,
    ) -> Self {
        let priority = TaskPriority::from_u8(enhanced_gap.enhanced_priority);
        let mut task = Self::from_gap(enhanced_gap.gap, priority);

        // Add enhanced metadata
        task.metadata.insert(
            "quality_score".to_string(),
            enhanced_gap.quality_score.to_string(),
        );
        task.metadata.insert(
            "passed_filters".to_string(),
            enhanced_gap.passed_filters.to_string(),
        );
        task.metadata.insert(
            "applied_rules_count".to_string(),
            enhanced_gap.applied_rules.len().to_string(),
        );

        if let Some(timeout) = custom_timeout {
            task.timeout = timeout;
        }

        task
    }

    /// Generate research query from gap information
    fn generate_research_query(gap: &DetectedGap) -> String {
        match gap.gap_type {
            GapType::TodoComment => {
                format!(
                    "Research implementation approach for TODO: {} in file {}",
                    gap.description,
                    gap.file_path.display()
                )
            }
            GapType::MissingDocumentation => {
                format!(
                    "Generate comprehensive documentation for {} in {}",
                    gap.description,
                    gap.file_path.display()
                )
            }
            GapType::UndocumentedTechnology => {
                format!(
                    "Research technology usage patterns and best practices for {}",
                    gap.description
                )
            }
            GapType::ApiDocumentationGap => {
                format!(
                    "Generate API documentation with examples for {} in {}",
                    gap.description,
                    gap.file_path.display()
                )
            }
            GapType::ConfigurationGap => {
                format!(
                    "Document configuration options and usage for {} in {}",
                    gap.description,
                    gap.file_path.display()
                )
            }
        }
    }

    /// Estimate task duration based on gap type and complexity
    pub fn estimate_duration(&mut self) {
        let base_duration = match self.gap.gap_type {
            GapType::TodoComment => Duration::from_secs(180), // 3 minutes
            GapType::MissingDocumentation => Duration::from_secs(120), // 2 minutes
            GapType::UndocumentedTechnology => Duration::from_secs(300), // 5 minutes
            GapType::ApiDocumentationGap => Duration::from_secs(240), // 4 minutes
            GapType::ConfigurationGap => Duration::from_secs(150), // 2.5 minutes
        };

        // Adjust based on priority and confidence
        let confidence_multiplier = 2.0 - self.gap.confidence; // Lower confidence = more time
        let priority_multiplier = match self.priority {
            TaskPriority::Low => 0.8,
            TaskPriority::Medium => 1.0,
            TaskPriority::High => 1.2,
            TaskPriority::Critical => 1.5,
        };

        let adjusted_duration = base_duration.mul_f64(confidence_multiplier * priority_multiplier);
        self.estimated_duration = Some(adjusted_duration);
    }

    /// Check if task has timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(started_at) = self.started_at {
            let elapsed = Utc::now().signed_duration_since(started_at);
            elapsed.to_std().unwrap_or(Duration::from_secs(0)) > self.timeout
        } else {
            false
        }
    }

    /// Check if task can be retried
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries && matches!(self.state, TaskState::Failed)
    }
}

// Implement ordering for priority queue (higher priority first)
impl PartialEq for ResearchTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.created_at == other.created_at
    }
}

impl Eq for ResearchTask {}

impl PartialOrd for ResearchTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ResearchTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first (reverse comparison for max-heap behavior)
        // Then older tasks first (normal comparison)
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.created_at.cmp(&self.created_at))
    }
}

/// Configuration for background scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundSchedulerConfig {
    /// Path to queue persistence file
    pub queue_file: PathBuf,
    /// Maximum number of tasks in queue
    pub max_queue_size: usize,
    /// How often to persist queue state
    pub persistence_interval: Duration,
    /// Maximum concurrent executing tasks
    pub max_concurrent_tasks: usize,
    /// Default task timeout
    pub default_timeout: Duration,
}

impl Default for BackgroundSchedulerConfig {
    fn default() -> Self {
        Self {
            queue_file: PathBuf::from("task_queue.json"),
            max_queue_size: 10000,
            persistence_interval: Duration::from_secs(30),
            max_concurrent_tasks: 5,
            default_timeout: Duration::from_secs(300),
        }
    }
}

/// Queue statistics and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMetrics {
    pub total_tasks_processed: u64,
    pub current_queue_size: usize,
    pub executing_tasks: usize,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub average_processing_time: Duration,
    pub queue_utilization: f64,
    pub last_updated: DateTime<Utc>,
}

/// Persistent queue storage format
#[derive(Debug, Serialize, Deserialize)]
struct QueuePersistence {
    pending_tasks: Vec<ResearchTask>,
    executing_tasks: Vec<ResearchTask>,
    completed_tasks: Vec<ResearchTask>,
    failed_tasks: Vec<ResearchTask>,
    metrics: QueueMetrics,
    version: u32,
}

/// Background research task scheduler with persistent queue
#[derive(Debug, Clone)]
pub struct BackgroundScheduler {
    /// Configuration
    config: BackgroundSchedulerConfig,
    /// Priority queue for pending tasks
    pending_queue: Arc<Mutex<BinaryHeap<ResearchTask>>>,
    /// Currently executing tasks
    executing_tasks: Arc<RwLock<HashMap<String, ResearchTask>>>,
    /// Completed tasks history (limited size)
    completed_tasks: Arc<RwLock<HashMap<String, ResearchTask>>>,
    /// Failed tasks for retry analysis
    failed_tasks: Arc<RwLock<HashMap<String, ResearchTask>>>,
    /// Queue metrics and statistics
    metrics: Arc<RwLock<QueueMetrics>>,
    /// Last persistence timestamp
    last_persistence: Arc<Mutex<Instant>>,
    /// Optional state manager for enhanced tracking
    state_manager: Arc<RwLock<Option<Arc<StateManager>>>>,
    /// Error handler for comprehensive error management
    error_handler: Arc<RwLock<Option<Arc<ErrorHandler>>>>,
}

impl BackgroundScheduler {
    /// Create a new background scheduler with the given configuration
    #[instrument(level = "debug", skip(config))]
    pub async fn new(config: BackgroundSchedulerConfig) -> Result<Self, SchedulerError> {
        info!(
            "Initializing background scheduler with queue file: {}",
            config.queue_file.display()
        );

        let scheduler = Self {
            config,
            pending_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            executing_tasks: Arc::new(RwLock::new(HashMap::new())),
            completed_tasks: Arc::new(RwLock::new(HashMap::new())),
            failed_tasks: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(QueueMetrics {
                total_tasks_processed: 0,
                current_queue_size: 0,
                executing_tasks: 0,
                completed_tasks: 0,
                failed_tasks: 0,
                average_processing_time: Duration::from_secs(0),
                queue_utilization: 0.0,
                last_updated: Utc::now(),
            })),
            last_persistence: Arc::new(Mutex::new(Instant::now())),
            state_manager: Arc::new(RwLock::new(None)),
            error_handler: Arc::new(RwLock::new(None)),
        };

        // Load existing queue state if file exists (defer to avoid potential deadlock)
        if scheduler.config.queue_file.exists() {
            scheduler.load_queue().await?;
        }

        debug!("Background scheduler initialized successfully");
        Ok(scheduler)
    }

    /// Get scheduler configuration
    pub fn config(&self) -> &BackgroundSchedulerConfig {
        &self.config
    }

    /// Configure state manager for enhanced tracking
    pub async fn configure_state_manager(
        &self,
        state_manager: Arc<StateManager>,
    ) -> Result<(), SchedulerError> {
        let mut manager_guard = self.state_manager.write().await;
        *manager_guard = Some(state_manager);
        debug!("State manager configured for background scheduler");
        Ok(())
    }

    /// Check if state manager is configured
    pub async fn has_state_manager(&self) -> bool {
        self.state_manager.read().await.is_some()
    }

    /// Configure error handler for comprehensive error management
    pub async fn configure_error_handler(
        &self,
        error_handler: Arc<ErrorHandler>,
    ) -> Result<(), SchedulerError> {
        let mut handler_guard = self.error_handler.write().await;
        *handler_guard = Some(error_handler);
        debug!("Error handler configured for background scheduler");
        Ok(())
    }

    /// Check if error handler is configured
    pub async fn has_error_handler(&self) -> bool {
        self.error_handler.read().await.is_some()
    }

    /// Add a task to the queue with comprehensive error handling
    #[instrument(level = "debug", skip(self, task))]
    pub async fn enqueue(&self, mut task: ResearchTask) -> Result<(), SchedulerError> {
        // Use error handler if available
        if let Some(error_handler) = self.error_handler.read().await.as_ref() {
            let enqueue_operation = || {
                let queue = self.pending_queue.clone();
                let config = self.config.clone();
                async move {
                    let queue_guard = queue.lock().await;

                    // Check queue size limit
                    if queue_guard.len() >= config.max_queue_size {
                        return Err(ProactiveError::ResourceExhaustion {
                            resource: "queue_capacity".to_string(),
                            usage: (queue_guard.len() as f64 / config.max_queue_size as f64)
                                * 100.0,
                            limit: 100.0,
                            suggested_backoff: Duration::from_secs(5),
                        });
                    }

                    Ok(())
                }
            };

            if let Err(_e) = error_handler
                .execute_with_retry(
                    enqueue_operation,
                    "scheduler_enqueue",
                    Some("background_scheduler"),
                )
                .await
            {
                return Err(SchedulerError::QueueFull {
                    limit: self.config.max_queue_size,
                });
            }
        } else {
            // Fallback to direct check if no error handler
            let queue = self.pending_queue.lock().await;
            if queue.len() >= self.config.max_queue_size {
                return Err(SchedulerError::QueueFull {
                    limit: self.config.max_queue_size,
                });
            }
            drop(queue);
        }

        let mut queue = self.pending_queue.lock().await;

        // Set task defaults if needed
        if task.estimated_duration.is_none() {
            task.estimate_duration();
        }

        debug!(
            "Enqueuing task {} with priority {:?}",
            task.id, task.priority
        );
        queue.push(task.clone());
        drop(queue);

        // Track task creation with state manager if configured
        if let Some(state_manager) = self.state_manager.read().await.as_ref() {
            if let Err(e) = state_manager.track_task_creation(&task).await {
                warn!("Failed to track task creation with state manager: {}", e);

                // Use error handler for state manager errors if available
                if let Some(error_handler) = self.error_handler.read().await.as_ref() {
                    let state_error = ProactiveError::Internal {
                        message: format!("State manager task tracking failed: {e}"),
                        component: "state_manager".to_string(),
                        recoverable: true,
                    };

                    let recovery_operation = || async { Ok(()) }; // Simple recovery
                    let _ = error_handler
                        .handle_error(
                            state_error,
                            recovery_operation,
                            "state_tracking",
                            Some("state_manager"),
                        )
                        .await;
                }
            }
        }

        // Update metrics with error handling
        self.update_queue_metrics().await;

        // Persist if interval has passed with error handling
        if let Err(e) = self.persist_if_needed().await {
            warn!("Failed to persist queue state: {}", e);

            // Use error handler for persistence errors if available
            if let Some(error_handler) = self.error_handler.read().await.as_ref() {
                let persistence_error = ProactiveError::Internal {
                    message: format!("Queue persistence failed: {e}"),
                    component: "background_scheduler".to_string(),
                    recoverable: true,
                };

                let recovery_operation = || async { Ok(()) }; // Simple recovery
                let _ = error_handler
                    .handle_error(
                        persistence_error,
                        recovery_operation,
                        "queue_persistence",
                        Some("background_scheduler"),
                    )
                    .await;
            }
        }

        Ok(())
    }

    /// Remove and return the highest priority task from the queue
    #[instrument(level = "debug", skip(self))]
    pub async fn dequeue(&self) -> Result<Option<ResearchTask>, SchedulerError> {
        let mut queue = self.pending_queue.lock().await;

        if let Some(task) = queue.pop() {
            debug!(
                "Dequeued task {} with priority {:?}",
                task.id, task.priority
            );

            // Update metrics
            drop(queue); // Release lock before async call
            self.update_queue_metrics().await;

            Ok(Some(task))
        } else {
            Ok(None)
        }
    }

    /// Peek at the highest priority task without removing it
    #[instrument(level = "debug", skip(self))]
    pub async fn peek(&self) -> Result<Option<ResearchTask>, SchedulerError> {
        let queue = self.pending_queue.lock().await;
        Ok(queue.peek().cloned())
    }

    /// Get current queue size
    pub async fn queue_size(&self) -> usize {
        let queue = self.pending_queue.lock().await;
        queue.len()
    }

    /// Update task state and move between collections
    #[instrument(level = "debug", skip(self, task))]
    pub async fn update_task(&self, task: ResearchTask) -> Result<(), SchedulerError> {
        self.update_task_with_metadata(task, None).await
    }

    /// Update task state with optional metadata
    #[instrument(level = "debug", skip(self, task, metadata))]
    pub async fn update_task_with_metadata(
        &self,
        mut task: ResearchTask,
        metadata: Option<StateTransitionMetadata>,
    ) -> Result<(), SchedulerError> {
        let old_state = {
            // Get current state from existing task if possible
            if let Ok(Some(existing_task)) = self.get_task_by_id(&task.id).await {
                existing_task.state
            } else {
                TaskState::Pending // Default assumption
            }
        };
        let task_id = task.id.clone();
        let new_state = task.state.clone();

        // Use state manager for transition if configured
        if let Some(state_manager) = self.state_manager.read().await.as_ref() {
            if let Err(e) = state_manager
                .transition_task(
                    &task_id,
                    new_state.clone(),
                    "background_scheduler",
                    metadata,
                )
                .await
            {
                warn!(
                    "State manager transition failed for task {}: {}",
                    task_id, e
                );
                // Continue with local state management as fallback
            }
        }

        match task.state {
            TaskState::Executing => {
                let mut executing = self.executing_tasks.write().await;
                executing.insert(task.id.clone(), task);
            }
            TaskState::Completed => {
                // Remove from executing if present
                {
                    let mut executing = self.executing_tasks.write().await;
                    executing.remove(&task.id);
                }

                // Add to completed
                let mut completed = self.completed_tasks.write().await;
                task.completed_at = Some(Utc::now());
                completed.insert(task.id.clone(), task);
            }
            TaskState::Failed => {
                // Remove from executing if present
                {
                    let mut executing = self.executing_tasks.write().await;
                    executing.remove(&task.id);
                }

                // Add to failed
                let mut failed = self.failed_tasks.write().await;
                failed.insert(task.id.clone(), task);
            }
            TaskState::Pending => {
                // This would typically re-queue the task
                task.retry_count += 1;
                task.state = TaskState::Pending;
                self.enqueue(task).await?;
            }
            TaskState::Cancelled => {
                // Remove from all collections
                {
                    let mut executing = self.executing_tasks.write().await;
                    executing.remove(&task.id);
                }
            }
        }

        debug!(
            "Updated task {} state from {:?} to {:?}",
            task_id, old_state, new_state
        );

        // Update metrics
        self.update_queue_metrics().await;

        Ok(())
    }

    /// Get task by ID from any collection
    pub async fn get_task_by_id(
        &self,
        task_id: &str,
    ) -> Result<Option<ResearchTask>, SchedulerError> {
        // Check pending queue
        {
            let queue = self.pending_queue.lock().await;
            if let Some(task) = queue.iter().find(|t| t.id == task_id) {
                return Ok(Some(task.clone()));
            }
        }

        // Check executing tasks
        {
            let executing = self.executing_tasks.read().await;
            if let Some(task) = executing.get(task_id) {
                return Ok(Some(task.clone()));
            }
        }

        // Check completed tasks
        {
            let completed = self.completed_tasks.read().await;
            if let Some(task) = completed.get(task_id) {
                return Ok(Some(task.clone()));
            }
        }

        // Check failed tasks
        {
            let failed = self.failed_tasks.read().await;
            if let Some(task) = failed.get(task_id) {
                return Ok(Some(task.clone()));
            }
        }

        Ok(None)
    }

    /// Get current queue metrics
    pub async fn get_metrics(&self) -> QueueMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Force persistence of queue state with comprehensive error handling
    #[instrument(level = "debug", skip(self))]
    pub async fn persist(&self) -> Result<(), SchedulerError> {
        // Use error handler if available
        if let Some(error_handler) = self.error_handler.read().await.as_ref() {
            let persist_operation = || {
                let queue = self.pending_queue.clone();
                let executing_tasks = self.executing_tasks.clone();
                let completed_tasks = self.completed_tasks.clone();
                let failed_tasks = self.failed_tasks.clone();
                let metrics = self.metrics.clone();
                let config = self.config.clone();
                let last_persistence = self.last_persistence.clone();

                async move {
                    let start = Instant::now();

                    // Collect all task data with error handling
                    let pending_tasks: Vec<ResearchTask> = {
                        let queue_guard = queue.lock().await;
                        queue_guard.iter().cloned().collect()
                    };

                    let executing_task_data: Vec<ResearchTask> =
                        { executing_tasks.read().await.values().cloned().collect() };

                    let completed_task_data: Vec<ResearchTask> =
                        { completed_tasks.read().await.values().cloned().collect() };

                    let failed_task_data: Vec<ResearchTask> =
                        { failed_tasks.read().await.values().cloned().collect() };

                    let metrics_data = { metrics.read().await.clone() };

                    let persistence_data = QueuePersistence {
                        pending_tasks,
                        executing_tasks: executing_task_data,
                        completed_tasks: completed_task_data,
                        failed_tasks: failed_task_data,
                        metrics: metrics_data,
                        version: 1,
                    };

                    // Write to temporary file first, then rename for atomicity
                    let temp_file = config.queue_file.with_extension("tmp");
                    let json_data =
                        serde_json::to_string_pretty(&persistence_data).map_err(|e| {
                            ProactiveError::Internal {
                                message: format!("JSON serialization failed: {e}"),
                                component: "background_scheduler".to_string(),
                                recoverable: false,
                            }
                        })?;

                    fs::write(&temp_file, json_data).await.map_err(|e| {
                        ProactiveError::Internal {
                            message: format!("Temporary file write failed: {e}"),
                            component: "background_scheduler".to_string(),
                            recoverable: true,
                        }
                    })?;

                    fs::rename(&temp_file, &config.queue_file)
                        .await
                        .map_err(|e| ProactiveError::Internal {
                            message: format!("File rename failed: {e}"),
                            component: "background_scheduler".to_string(),
                            recoverable: true,
                        })?;

                    // Update last persistence time
                    {
                        let mut last_persistence_guard = last_persistence.lock().await;
                        *last_persistence_guard = Instant::now();
                    }

                    let duration = start.elapsed();
                    debug!("Queue persisted successfully in {:?}", duration);

                    Ok(())
                }
            };

            error_handler
                .execute_with_retry(
                    persist_operation,
                    "queue_persistence",
                    Some("background_scheduler"),
                )
                .await
                .map_err(|e| {
                    SchedulerError::Configuration(format!(
                        "Persistence failed with error handling: {e}"
                    ))
                })?;
        } else {
            // Fallback to original implementation
            let start = Instant::now();

            // Collect all task data
            let pending_tasks: Vec<ResearchTask> = {
                let queue = self.pending_queue.lock().await;
                queue.iter().cloned().collect()
            };

            let executing_tasks: Vec<ResearchTask> = {
                let executing = self.executing_tasks.read().await;
                executing.values().cloned().collect()
            };

            let completed_tasks: Vec<ResearchTask> = {
                let completed = self.completed_tasks.read().await;
                completed.values().cloned().collect()
            };

            let failed_tasks: Vec<ResearchTask> = {
                let failed = self.failed_tasks.read().await;
                failed.values().cloned().collect()
            };

            let metrics = {
                let metrics = self.metrics.read().await;
                metrics.clone()
            };

            let persistence_data = QueuePersistence {
                pending_tasks,
                executing_tasks,
                completed_tasks,
                failed_tasks,
                metrics,
                version: 1,
            };

            // Write to temporary file first, then rename for atomicity
            let temp_file = self.config.queue_file.with_extension("tmp");
            let json_data = serde_json::to_string_pretty(&persistence_data)?;

            fs::write(&temp_file, json_data).await?;
            fs::rename(&temp_file, &self.config.queue_file).await?;

            // Update last persistence time
            {
                let mut last_persistence = self.last_persistence.lock().await;
                *last_persistence = Instant::now();
            }

            let duration = start.elapsed();
            debug!("Queue persisted successfully in {:?}", duration);
        }

        Ok(())
    }

    /// Load queue state from persistence file
    #[instrument(level = "debug", skip(self))]
    pub async fn load_queue(&self) -> Result<(), SchedulerError> {
        if !self.config.queue_file.exists() {
            debug!("Queue file does not exist, starting with empty queue");
            return Ok(());
        }

        let start = Instant::now();

        match fs::read_to_string(&self.config.queue_file).await {
            Ok(content) => {
                match serde_json::from_str::<QueuePersistence>(&content) {
                    Ok(persistence_data) => {
                        // Restore pending tasks
                        {
                            let mut queue = self.pending_queue.lock().await;
                            for task in persistence_data.pending_tasks {
                                queue.push(task);
                            }
                        }

                        // Restore executing tasks
                        {
                            let mut executing = self.executing_tasks.write().await;
                            for task in persistence_data.executing_tasks {
                                executing.insert(task.id.clone(), task);
                            }
                        }

                        // Restore completed tasks (limit to recent ones to manage memory)
                        {
                            let mut completed = self.completed_tasks.write().await;
                            for task in persistence_data.completed_tasks.into_iter().take(1000) {
                                completed.insert(task.id.clone(), task);
                            }
                        }

                        // Restore failed tasks
                        {
                            let mut failed = self.failed_tasks.write().await;
                            for task in persistence_data.failed_tasks {
                                failed.insert(task.id.clone(), task);
                            }
                        }

                        // Restore metrics
                        {
                            let mut metrics = self.metrics.write().await;
                            *metrics = persistence_data.metrics;
                        }

                        let duration = start.elapsed();
                        info!("Queue loaded successfully in {:?}", duration);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to parse queue file, starting with empty queue: {}",
                            e
                        );
                        // Continue with empty queue rather than failing
                    }
                }
            }
            Err(e) => {
                warn!(
                    "Failed to read queue file, starting with empty queue: {}",
                    e
                );
                // Continue with empty queue rather than failing
            }
        }

        Ok(())
    }

    /// Persist queue state if interval has passed
    async fn persist_if_needed(&self) -> Result<(), SchedulerError> {
        let should_persist = {
            let last_persistence = self.last_persistence.lock().await;
            last_persistence.elapsed() >= self.config.persistence_interval
        };

        if should_persist {
            self.persist().await?;
        }

        Ok(())
    }

    /// Update queue metrics
    async fn update_queue_metrics(&self) {
        let pending_count = {
            let queue = self.pending_queue.lock().await;
            queue.len()
        };

        let executing_count = {
            let executing = self.executing_tasks.read().await;
            executing.len()
        };

        let completed_count = {
            let completed = self.completed_tasks.read().await;
            completed.len() as u64
        };

        let failed_count = {
            let failed = self.failed_tasks.read().await;
            failed.len() as u64
        };

        let mut metrics = self.metrics.write().await;
        metrics.current_queue_size = pending_count;
        metrics.executing_tasks = executing_count;
        metrics.completed_tasks = completed_count;
        metrics.failed_tasks = failed_count;
        metrics.queue_utilization = pending_count as f64 / self.config.max_queue_size as f64;
        metrics.last_updated = Utc::now();
    }

    /// Get tasks that have timed out
    pub async fn get_timed_out_tasks(&self) -> Vec<ResearchTask> {
        let executing = self.executing_tasks.read().await;
        executing
            .values()
            .filter(|task| task.is_timed_out())
            .cloned()
            .collect()
    }

    /// Get task state history from state manager if available
    pub async fn get_task_state_history(
        &self,
        task_id: &str,
    ) -> Result<Option<Vec<crate::proactive::StateChangeEntry>>, SchedulerError> {
        if let Some(state_manager) = self.state_manager.read().await.as_ref() {
            match state_manager.get_task_state_history(task_id).await {
                Ok(history) => Ok(Some(history)),
                Err(_) => Ok(None), // Task not found in state manager
            }
        } else {
            Ok(None) // No state manager configured
        }
    }

    /// Get enhanced task lifecycle information from state manager
    pub async fn get_task_lifecycle(
        &self,
        task_id: &str,
    ) -> Result<Option<crate::proactive::TaskLifecycle>, SchedulerError> {
        if let Some(state_manager) = self.state_manager.read().await.as_ref() {
            match state_manager.get_task_lifecycle(task_id).await {
                Ok(lifecycle) => Ok(Some(lifecycle)),
                Err(_) => Ok(None), // Task not found in state manager
            }
        } else {
            Ok(None) // No state manager configured
        }
    }

    /// Perform recovery operations using state manager if available
    pub async fn perform_state_recovery(&self) -> Result<u64, SchedulerError> {
        if let Some(state_manager) = self.state_manager.read().await.as_ref() {
            match state_manager.perform_recovery().await {
                Ok(count) => {
                    info!("State manager recovered {} tasks", count);
                    Ok(count)
                }
                Err(e) => {
                    warn!("State manager recovery failed: {}", e);
                    Ok(0)
                }
            }
        } else {
            Ok(0) // No state manager configured
        }
    }

    /// Cancel all tasks (useful for shutdown)
    pub async fn cancel_all_tasks(&self) -> Result<(), SchedulerError> {
        info!("Cancelling all tasks");

        // Clear pending queue
        {
            let mut queue = self.pending_queue.lock().await;
            queue.clear();
        }

        // Cancel executing tasks
        {
            let mut executing = self.executing_tasks.write().await;
            executing.clear();
        }

        // Update metrics
        self.update_queue_metrics().await;

        // Persist the cleared state
        self.persist().await?;

        Ok(())
    }
}

// Trait for queue operations (useful for testing and future extensions)
#[async_trait::async_trait]
pub trait QueueOperations {
    async fn enqueue(&self, task: ResearchTask) -> Result<(), SchedulerError>;
    async fn dequeue(&self) -> Result<Option<ResearchTask>, SchedulerError>;
    async fn peek(&self) -> Result<Option<ResearchTask>, SchedulerError>;
    async fn queue_size(&self) -> usize;
    async fn get_metrics(&self) -> QueueMetrics;
}

#[async_trait::async_trait]
impl QueueOperations for BackgroundScheduler {
    async fn enqueue(&self, task: ResearchTask) -> Result<(), SchedulerError> {
        self.enqueue(task).await
    }

    async fn dequeue(&self) -> Result<Option<ResearchTask>, SchedulerError> {
        self.dequeue().await
    }

    async fn peek(&self) -> Result<Option<ResearchTask>, SchedulerError> {
        self.peek().await
    }

    async fn queue_size(&self) -> usize {
        self.queue_size().await
    }

    async fn get_metrics(&self) -> QueueMetrics {
        self.get_metrics().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_gap() -> DetectedGap {
        DetectedGap {
            gap_type: GapType::TodoComment,
            file_path: PathBuf::from("test.rs"),
            line_number: 42,
            column_number: Some(10),
            context: "// TODO: Test implementation".to_string(),
            description: "Test TODO comment".to_string(),
            confidence: 0.9,
            priority: 7,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_task_creation() {
        let gap = create_test_gap();
        let task = ResearchTask::from_gap(gap.clone(), TaskPriority::High);

        assert_eq!(task.gap.gap_type, GapType::TodoComment);
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.state, TaskState::Pending);
        assert_eq!(task.retry_count, 0);
        assert!(!task.research_query.is_empty());
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let mut heap = BinaryHeap::new();

        let low_task = ResearchTask::from_gap(create_test_gap(), TaskPriority::Low);
        let high_task = ResearchTask::from_gap(create_test_gap(), TaskPriority::High);
        let medium_task = ResearchTask::from_gap(create_test_gap(), TaskPriority::Medium);

        heap.push(low_task);
        heap.push(high_task.clone());
        heap.push(medium_task);

        // Should pop highest priority first
        let first = heap.pop().unwrap();
        assert_eq!(first.priority, TaskPriority::High);
        assert_eq!(first.id, high_task.id);
    }

    #[tokio::test]
    async fn test_scheduler_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let queue_file = temp_dir.path().join("test_queue.json");

        let config = BackgroundSchedulerConfig {
            queue_file,
            max_queue_size: 10,
            persistence_interval: Duration::from_secs(60),
            max_concurrent_tasks: 2,
            default_timeout: Duration::from_secs(30),
        };

        let scheduler = BackgroundScheduler::new(config).await.unwrap();

        // Test enqueue
        let gap = create_test_gap();
        let task = ResearchTask::from_gap(gap, TaskPriority::Medium);
        let task_id = task.id.clone();

        scheduler.enqueue(task).await.unwrap();
        assert_eq!(scheduler.queue_size().await, 1);

        // Test peek
        let peeked = scheduler.peek().await.unwrap();
        assert!(peeked.is_some());
        assert_eq!(peeked.unwrap().id, task_id);
        assert_eq!(scheduler.queue_size().await, 1); // Should not remove

        // Test dequeue
        let dequeued = scheduler.dequeue().await.unwrap();
        assert!(dequeued.is_some());
        assert_eq!(dequeued.unwrap().id, task_id);
        assert_eq!(scheduler.queue_size().await, 0);
    }
}
