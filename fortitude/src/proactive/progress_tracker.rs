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

// ABOUTME: Enhanced progress tracking system for background research tasks with detailed step monitoring
//! This module provides comprehensive progress tracking for research tasks with features including:
//! - Real-time progress updates during task execution with detailed step tracking
//! - Progress event emission and notification integration
//! - Performance metrics tracking (time per step, completion rates)
//! - Progress persistence across application restarts
//! - External APIs for progress monitoring
//! - Integration with existing task executor and state manager

use crate::proactive::{NotificationChannel, NotificationSystem, StateManager, TaskProgress};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

/// Errors that can occur during progress tracking operations
#[derive(Error, Debug)]
pub enum ProgressTrackerError {
    #[error("Progress tracker not initialized")]
    NotInitialized,

    #[error("Task not found: {task_id}")]
    TaskNotFound { task_id: String },

    #[error("Invalid progress update: {reason}")]
    InvalidProgressUpdate { reason: String },

    #[error("Notification system error: {0}")]
    NotificationError(String),

    #[error("Persistence error: {0}")]
    PersistenceError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("State manager error: {0}")]
    StateManagerError(String),
}

/// Detailed progress step information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressStep {
    pub step_id: String,
    pub task_id: String,
    pub step_name: String,
    pub description: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress_percent: f64,
    pub step_metadata: HashMap<String, String>,
    pub substeps: Vec<ProgressStep>,
    pub error_info: Option<String>,
}

impl ProgressStep {
    pub fn new(
        task_id: String,
        step_name: String,
        description: String,
        progress_percent: f64,
    ) -> Self {
        Self {
            step_id: Uuid::new_v4().to_string(),
            task_id,
            step_name,
            description,
            started_at: Utc::now(),
            completed_at: None,
            progress_percent,
            step_metadata: HashMap::new(),
            substeps: Vec::new(),
            error_info: None,
        }
    }

    pub fn complete(&mut self) {
        self.completed_at = Some(Utc::now());
        self.progress_percent = 100.0;
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.step_metadata = metadata;
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.error_info = Some(error);
        self
    }

    pub fn duration(&self) -> Option<Duration> {
        if let Some(completed) = self.completed_at {
            Some(
                completed
                    .signed_duration_since(self.started_at)
                    .to_std()
                    .unwrap_or(Duration::from_secs(0)),
            )
        } else {
            Some(
                Utc::now()
                    .signed_duration_since(self.started_at)
                    .to_std()
                    .unwrap_or(Duration::from_secs(0)),
            )
        }
    }
}

/// Enhanced task progress with detailed step tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedTaskProgress {
    pub task_id: String,
    pub current_stage: String,
    pub overall_progress_percent: f64,
    pub started_at: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub steps: Vec<ProgressStep>,
    pub current_step_index: Option<usize>,
    pub performance_metrics: ProgressPerformanceMetrics,
    pub metadata: HashMap<String, String>,
}

impl EnhancedTaskProgress {
    pub fn new(task_id: String) -> Self {
        Self {
            task_id,
            current_stage: "initializing".to_string(),
            overall_progress_percent: 0.0,
            started_at: Utc::now(),
            last_update: Utc::now(),
            estimated_completion: None,
            steps: Vec::new(),
            current_step_index: None,
            performance_metrics: ProgressPerformanceMetrics::default(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_step(&mut self, step: ProgressStep) {
        self.steps.push(step);
        self.update_overall_progress();
        self.last_update = Utc::now();
    }

    pub fn complete_current_step(&mut self) {
        if let Some(index) = self.current_step_index {
            if let Some(step) = self.steps.get_mut(index) {
                step.complete();
                self.update_overall_progress();
                self.last_update = Utc::now();
            }
        }
    }

    pub fn start_step(&mut self, step_name: &str) -> Option<usize> {
        // Find step by name and mark as current
        for (index, step) in self.steps.iter().enumerate() {
            if step.step_name == step_name {
                self.current_step_index = Some(index);
                self.update_overall_progress();
                self.last_update = Utc::now();
                return Some(index);
            }
        }
        None
    }

    fn update_overall_progress(&mut self) {
        if self.steps.is_empty() {
            return;
        }

        let total_progress: f64 = self.steps.iter().map(|step| step.progress_percent).sum();

        self.overall_progress_percent = total_progress / self.steps.len() as f64;

        // Update performance metrics
        self.performance_metrics.update_metrics(&self.steps);
    }

    pub fn estimate_completion(&mut self) {
        if let Some(avg_step_duration) = self.performance_metrics.average_step_duration {
            let remaining_steps = self
                .steps
                .iter()
                .filter(|step| step.completed_at.is_none())
                .count();

            if remaining_steps > 0 {
                let estimated_remaining_time = avg_step_duration * remaining_steps as u32;
                self.estimated_completion = Some(
                    Utc::now()
                        + chrono::Duration::from_std(estimated_remaining_time)
                            .unwrap_or(chrono::Duration::zero()),
                );
            }
        }
    }
}

/// Performance metrics for progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressPerformanceMetrics {
    pub total_steps: u32,
    pub completed_steps: u32,
    pub failed_steps: u32,
    pub average_step_duration: Option<Duration>,
    pub fastest_step_duration: Option<Duration>,
    pub slowest_step_duration: Option<Duration>,
    pub throughput_steps_per_minute: f64,
    pub last_updated: DateTime<Utc>,
}

impl Default for ProgressPerformanceMetrics {
    fn default() -> Self {
        Self {
            total_steps: 0,
            completed_steps: 0,
            failed_steps: 0,
            average_step_duration: None,
            fastest_step_duration: None,
            slowest_step_duration: None,
            throughput_steps_per_minute: 0.0,
            last_updated: Utc::now(),
        }
    }
}

impl ProgressPerformanceMetrics {
    pub fn update_metrics(&mut self, steps: &[ProgressStep]) {
        self.total_steps = steps.len() as u32;
        self.completed_steps = steps.iter().filter(|s| s.completed_at.is_some()).count() as u32;
        self.failed_steps = steps.iter().filter(|s| s.error_info.is_some()).count() as u32;

        let completed_steps: Vec<_> = steps.iter().filter(|s| s.completed_at.is_some()).collect();

        if !completed_steps.is_empty() {
            let durations: Vec<Duration> = completed_steps
                .iter()
                .filter_map(|s| s.duration())
                .collect();

            if !durations.is_empty() {
                let total_duration: Duration = durations.iter().sum();
                self.average_step_duration = Some(total_duration / durations.len() as u32);

                self.fastest_step_duration = durations.iter().min().copied();
                self.slowest_step_duration = durations.iter().max().copied();

                // Calculate throughput
                if let Some(first_step) = completed_steps.first() {
                    if let Some(last_step) = completed_steps.last() {
                        if let Some(last_completed) = last_step.completed_at {
                            let total_time =
                                last_completed.signed_duration_since(first_step.started_at);
                            let total_minutes: i64 = total_time.num_minutes();
                            if total_minutes > 0 {
                                self.throughput_steps_per_minute =
                                    self.completed_steps as f64 / total_minutes as f64;
                            }
                        }
                    }
                }
            }
        }

        self.last_updated = Utc::now();
    }
}

/// Progress events for real-time monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressEvent {
    TaskStarted {
        task_id: String,
        timestamp: DateTime<Utc>,
    },
    StepStarted {
        task_id: String,
        step_id: String,
        step_name: String,
        timestamp: DateTime<Utc>,
    },
    StepProgress {
        task_id: String,
        step_id: String,
        progress_percent: f64,
        timestamp: DateTime<Utc>,
    },
    StepCompleted {
        task_id: String,
        step_id: String,
        duration: Duration,
        timestamp: DateTime<Utc>,
    },
    StepFailed {
        task_id: String,
        step_id: String,
        error: String,
        timestamp: DateTime<Utc>,
    },
    TaskCompleted {
        task_id: String,
        total_duration: Duration,
        timestamp: DateTime<Utc>,
    },
    TaskFailed {
        task_id: String,
        error: String,
        timestamp: DateTime<Utc>,
    },
}

/// Configuration for the progress tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressTrackerConfig {
    /// Enable real-time progress notifications
    pub enable_notifications: bool,
    /// Notification channels for progress updates
    pub notification_channels: Vec<NotificationChannel>,
    /// Interval for progress update notifications
    pub notification_interval: Duration,
    /// Enable progress persistence
    pub enable_persistence: bool,
    /// Persistence interval
    pub persistence_interval: Duration,
    /// Maximum number of progress entries to keep per task
    pub max_progress_history: usize,
    /// Enable performance metrics tracking
    pub enable_metrics: bool,
}

impl Default for ProgressTrackerConfig {
    fn default() -> Self {
        Self {
            enable_notifications: true,
            notification_channels: vec![NotificationChannel::CLI],
            notification_interval: Duration::from_secs(5),
            enable_persistence: true,
            persistence_interval: Duration::from_secs(10),
            max_progress_history: 100,
            enable_metrics: true,
        }
    }
}

/// Enhanced progress tracker for background research tasks
pub struct ProgressTracker {
    config: ProgressTrackerConfig,
    /// Active task progress tracking
    active_tasks: Arc<RwLock<HashMap<String, EnhancedTaskProgress>>>,
    /// Progress event broadcaster
    progress_events: broadcast::Sender<ProgressEvent>,
    /// Notification system integration
    notification_system: Option<Arc<NotificationSystem>>,
    /// State manager integration
    state_manager: Option<Arc<StateManager>>,
    /// Running state
    running: Arc<RwLock<bool>>,
    /// Background tasks
    background_tasks: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

impl ProgressTracker {
    /// Create a new progress tracker with the given configuration
    pub fn new(config: ProgressTrackerConfig) -> Self {
        let (progress_events, _) = broadcast::channel(1000);

        Self {
            config,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            progress_events,
            notification_system: None,
            state_manager: None,
            running: Arc::new(RwLock::new(false)),
            background_tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Start the progress tracker
    #[instrument(level = "debug", skip(self))]
    pub async fn start(&self) -> Result<(), ProgressTrackerError> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        info!("Starting enhanced progress tracker");

        // Start background monitoring tasks
        if self.config.enable_notifications {
            self.start_notification_worker().await;
        }

        if self.config.enable_persistence {
            self.start_persistence_worker().await;
        }

        Ok(())
    }

    /// Stop the progress tracker
    pub async fn stop(&self) -> Result<(), ProgressTrackerError> {
        let mut running = self.running.write().await;
        if !*running {
            return Ok(());
        }
        *running = false;
        drop(running);

        info!("Stopping progress tracker");

        // Cancel background tasks
        let mut background_tasks = self.background_tasks.lock().await;
        for handle in background_tasks.drain(..) {
            handle.abort();
        }

        Ok(())
    }

    /// Configure notification system integration
    pub async fn set_notification_system(&mut self, notification_system: Arc<NotificationSystem>) {
        self.notification_system = Some(notification_system);
    }

    /// Configure state manager integration
    pub async fn set_state_manager(&mut self, state_manager: Arc<StateManager>) {
        self.state_manager = Some(state_manager);
    }

    /// Start tracking progress for a task
    #[instrument(level = "debug", skip(self))]
    pub async fn start_task(&self, task_id: String) -> Result<(), ProgressTrackerError> {
        if !*self.running.read().await {
            return Err(ProgressTrackerError::NotInitialized);
        }

        let progress = EnhancedTaskProgress::new(task_id.clone());

        {
            let mut active_tasks = self.active_tasks.write().await;
            active_tasks.insert(task_id.clone(), progress);
        }

        // Emit task started event
        let event = ProgressEvent::TaskStarted {
            task_id: task_id.clone(),
            timestamp: Utc::now(),
        };
        let _ = self.progress_events.send(event);

        debug!("Started progress tracking for task: {}", task_id);
        Ok(())
    }

    /// Add a progress step for a task
    #[instrument(level = "debug", skip(self))]
    pub async fn add_step(
        &self,
        task_id: &str,
        step_name: String,
        description: String,
        progress_percent: f64,
    ) -> Result<String, ProgressTrackerError> {
        let mut active_tasks = self.active_tasks.write().await;
        let progress =
            active_tasks
                .get_mut(task_id)
                .ok_or_else(|| ProgressTrackerError::TaskNotFound {
                    task_id: task_id.to_string(),
                })?;

        let step = ProgressStep::new(
            task_id.to_string(),
            step_name.clone(),
            description,
            progress_percent,
        );
        let step_id = step.step_id.clone();

        progress.add_step(step);

        // Emit step started event
        let event = ProgressEvent::StepStarted {
            task_id: task_id.to_string(),
            step_id: step_id.clone(),
            step_name,
            timestamp: Utc::now(),
        };
        let _ = self.progress_events.send(event);

        debug!("Added progress step {} for task: {}", step_id, task_id);
        Ok(step_id)
    }

    /// Update progress for a specific step
    #[instrument(level = "debug", skip(self))]
    pub async fn update_step_progress(
        &self,
        task_id: &str,
        step_id: &str,
        progress_percent: f64,
    ) -> Result<(), ProgressTrackerError> {
        let mut active_tasks = self.active_tasks.write().await;
        let progress =
            active_tasks
                .get_mut(task_id)
                .ok_or_else(|| ProgressTrackerError::TaskNotFound {
                    task_id: task_id.to_string(),
                })?;

        // Find and update the step
        if let Some(step) = progress.steps.iter_mut().find(|s| s.step_id == step_id) {
            step.progress_percent = progress_percent;
            step.step_metadata
                .insert("last_update".to_string(), Utc::now().to_rfc3339());
        } else {
            return Err(ProgressTrackerError::InvalidProgressUpdate {
                reason: format!("Step {step_id} not found in task {task_id}"),
            });
        }

        progress.update_overall_progress();

        // Emit step progress event
        let event = ProgressEvent::StepProgress {
            task_id: task_id.to_string(),
            step_id: step_id.to_string(),
            progress_percent,
            timestamp: Utc::now(),
        };
        let _ = self.progress_events.send(event);

        debug!(
            "Updated step {} progress to {}% for task: {}",
            step_id, progress_percent, task_id
        );
        Ok(())
    }

    /// Complete a step
    #[instrument(level = "debug", skip(self))]
    pub async fn complete_step(
        &self,
        task_id: &str,
        step_id: &str,
    ) -> Result<(), ProgressTrackerError> {
        let mut active_tasks = self.active_tasks.write().await;
        let progress =
            active_tasks
                .get_mut(task_id)
                .ok_or_else(|| ProgressTrackerError::TaskNotFound {
                    task_id: task_id.to_string(),
                })?;

        // Find and complete the step
        if let Some(step) = progress.steps.iter_mut().find(|s| s.step_id == step_id) {
            let duration = step.duration().unwrap_or(Duration::from_secs(0));
            step.complete();

            // Emit step completed event
            let event = ProgressEvent::StepCompleted {
                task_id: task_id.to_string(),
                step_id: step_id.to_string(),
                duration,
                timestamp: Utc::now(),
            };
            let _ = self.progress_events.send(event);
        } else {
            return Err(ProgressTrackerError::InvalidProgressUpdate {
                reason: format!("Step {step_id} not found in task {task_id}"),
            });
        }

        progress.update_overall_progress();
        progress.estimate_completion();

        debug!("Completed step {} for task: {}", step_id, task_id);
        Ok(())
    }

    /// Complete task tracking
    #[instrument(level = "debug", skip(self))]
    pub async fn complete_task(&self, task_id: &str) -> Result<(), ProgressTrackerError> {
        let total_duration = {
            let mut active_tasks = self.active_tasks.write().await;
            if let Some(progress) = active_tasks.remove(task_id) {
                progress
                    .started_at
                    .signed_duration_since(Utc::now())
                    .to_std()
                    .unwrap_or(Duration::from_secs(0))
            } else {
                return Err(ProgressTrackerError::TaskNotFound {
                    task_id: task_id.to_string(),
                });
            }
        };

        // Emit task completed event
        let event = ProgressEvent::TaskCompleted {
            task_id: task_id.to_string(),
            total_duration,
            timestamp: Utc::now(),
        };
        let _ = self.progress_events.send(event);

        debug!("Completed progress tracking for task: {}", task_id);
        Ok(())
    }

    /// Get current progress for a task
    pub async fn get_task_progress(
        &self,
        task_id: &str,
    ) -> Result<EnhancedTaskProgress, ProgressTrackerError> {
        let active_tasks = self.active_tasks.read().await;
        active_tasks
            .get(task_id)
            .cloned()
            .ok_or_else(|| ProgressTrackerError::TaskNotFound {
                task_id: task_id.to_string(),
            })
    }

    /// Get all active task progress
    pub async fn get_all_active_progress(&self) -> HashMap<String, EnhancedTaskProgress> {
        let active_tasks = self.active_tasks.read().await;
        active_tasks.clone()
    }

    /// Subscribe to progress events
    pub fn subscribe_to_progress_events(&self) -> broadcast::Receiver<ProgressEvent> {
        self.progress_events.subscribe()
    }

    /// Convert enhanced progress to basic TaskProgress for compatibility
    pub async fn to_task_progress(
        &self,
        task_id: &str,
    ) -> Result<TaskProgress, ProgressTrackerError> {
        let enhanced = self.get_task_progress(task_id).await?;

        Ok(TaskProgress {
            task_id: enhanced.task_id,
            stage: enhanced.current_stage,
            progress_percent: enhanced.overall_progress_percent,
            started_at: enhanced.started_at,
            last_update: enhanced.last_update,
            estimated_completion: enhanced.estimated_completion,
            metadata: enhanced.metadata,
        })
    }

    /// Start notification worker
    async fn start_notification_worker(&self) {
        if self.notification_system.is_none() {
            return;
        }

        let running = self.running.clone();
        let mut progress_receiver = self.progress_events.subscribe();
        let notification_system = self.notification_system.clone();
        let notification_interval = self.config.notification_interval;

        let handle = tokio::spawn(async move {
            let mut interval = interval(notification_interval);

            while *running.read().await {
                tokio::select! {
                    _ = interval.tick() => {
                        // Periodic progress notifications
                        if let Some(ref notifier) = notification_system {
                            let _ = notifier.info(
                                "Progress Update".to_string(),
                                "Background research tasks are progressing".to_string(),
                            ).await;
                        }
                    }
                    event_result = progress_receiver.recv() => {
                        if let Ok(event) = event_result {
                            // Handle specific progress events
                            if let Some(ref notifier) = notification_system {
                                match event {
                                    ProgressEvent::TaskCompleted { task_id, .. } => {
                                        let _ = notifier.success(
                                            "Task Completed".to_string(),
                                            format!("Research task {task_id} completed successfully"),
                                        ).await;
                                    }
                                    ProgressEvent::TaskFailed { task_id, error, .. } => {
                                        let _ = notifier.error(
                                            "Task Failed".to_string(),
                                            format!("Research task {task_id} failed: {error}"),
                                        ).await;
                                    }
                                    _ => {} // Handle other events as needed
                                }
                            }
                        }
                    }
                }
            }
        });

        let mut background_tasks = self.background_tasks.lock().await;
        background_tasks.push(handle);
    }

    /// Start persistence worker
    async fn start_persistence_worker(&self) {
        let running = self.running.clone();
        let active_tasks = self.active_tasks.clone();
        let persistence_interval = self.config.persistence_interval;

        let handle = tokio::spawn(async move {
            let mut interval = interval(persistence_interval);

            while *running.read().await {
                interval.tick().await;

                // Persist progress data
                let tasks = {
                    let active_tasks = active_tasks.read().await;
                    active_tasks.clone()
                };

                // TODO: Implement actual persistence logic
                debug!("Persisting progress for {} active tasks", tasks.len());
            }
        });

        let mut background_tasks = self.background_tasks.lock().await;
        background_tasks.push(handle);
    }
}

// Implement Clone for ProgressTracker (needed for tests)
impl Clone for ProgressTracker {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            active_tasks: self.active_tasks.clone(),
            progress_events: self.progress_events.clone(),
            notification_system: self.notification_system.clone(),
            state_manager: self.state_manager.clone(),
            running: self.running.clone(),
            background_tasks: self.background_tasks.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_progress_tracker_creation() {
        let config = ProgressTrackerConfig::default();
        let tracker = ProgressTracker::new(config);

        assert!(!*tracker.running.read().await);
    }

    #[tokio::test]
    async fn test_task_progress_tracking() {
        let config = ProgressTrackerConfig::default();
        let tracker = ProgressTracker::new(config);
        tracker.start().await.unwrap();

        let task_id = "test_task_123".to_string();

        // Start tracking
        tracker.start_task(task_id.clone()).await.unwrap();

        // Add steps
        let step1_id = tracker
            .add_step(
                &task_id,
                "step1".to_string(),
                "First step".to_string(),
                25.0,
            )
            .await
            .unwrap();

        let _step2_id = tracker
            .add_step(
                &task_id,
                "step2".to_string(),
                "Second step".to_string(),
                75.0,
            )
            .await
            .unwrap();

        // Update progress
        tracker
            .update_step_progress(&task_id, &step1_id, 50.0)
            .await
            .unwrap();
        tracker.complete_step(&task_id, &step1_id).await.unwrap();

        // Get progress
        let progress = tracker.get_task_progress(&task_id).await.unwrap();
        assert_eq!(progress.steps.len(), 2);
        assert!(progress.steps[0].completed_at.is_some());

        // Complete task
        tracker.complete_task(&task_id).await.unwrap();

        // Verify task is no longer active
        assert!(tracker.get_task_progress(&task_id).await.is_err());

        tracker.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_progress_events() {
        let config = ProgressTrackerConfig::default();
        let tracker = ProgressTracker::new(config);
        tracker.start().await.unwrap();

        let mut event_receiver = tracker.subscribe_to_progress_events();
        let task_id = "test_task_events".to_string();

        // Start tracking in background
        let tracker_clone = tracker.clone();
        let task_id_clone = task_id.clone();
        tokio::spawn(async move {
            tracker_clone
                .start_task(task_id_clone.clone())
                .await
                .unwrap();
            let step_id = tracker_clone
                .add_step(
                    &task_id_clone,
                    "test_step".to_string(),
                    "Test step".to_string(),
                    50.0,
                )
                .await
                .unwrap();
            tracker_clone
                .complete_step(&task_id_clone, &step_id)
                .await
                .unwrap();
            tracker_clone.complete_task(&task_id_clone).await.unwrap();
        });

        // Collect events
        let mut events = Vec::new();
        for _ in 0..5 {
            if let Ok(Ok(event)) =
                tokio::time::timeout(Duration::from_millis(100), event_receiver.recv()).await
            {
                events.push(event);
            }
        }

        // Verify events were received
        assert!(!events.is_empty());

        // Check for task started event
        let has_task_started = events
            .iter()
            .any(|e| matches!(e, ProgressEvent::TaskStarted { task_id: id, .. } if id == &task_id));
        assert!(has_task_started);

        tracker.stop().await.unwrap();
    }
}
