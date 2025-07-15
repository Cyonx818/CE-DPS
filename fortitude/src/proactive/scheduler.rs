// ABOUTME: Research task scheduling system with configurable intervals for proactive research mode
//! This module provides a comprehensive scheduling system that orchestrates the complete
//! pipeline from gap analysis to research execution. Features include:
//! - Event-driven scheduling triggered by file system changes
//! - Time-based scheduling with cron-like expressions and configurable intervals
//! - Priority-based scheduling with different frequencies for different gap priority levels
//! - Pipeline orchestration coordinating gap analysis → task queuing → background execution
//! - Resource-aware scheduling that adjusts based on system load and availability
//! - Integration with all previous task components (file monitor, gap analysis, queue, executor)

use crate::proactive::{
    BackgroundScheduler, DetectedGap, EventType, FileEvent, GapType, PriorityScorer, ResearchTask,
    ResourceUsage, StateManager, TaskExecutor, TaskPriority,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::{interval, sleep};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Errors that can occur during research scheduling operations
#[derive(Error, Debug)]
pub enum ResearchSchedulerError {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Scheduler not running")]
    NotRunning,

    #[error("Scheduler already running")]
    AlreadyRunning,

    #[error("Queue not configured")]
    QueueNotConfigured,

    #[error("Executor not configured")]
    ExecutorNotConfigured,

    #[error("Gap analysis failed: {0}")]
    GapAnalysisFailed(String),

    #[error("Task queuing failed: {0}")]
    TaskQueuingFailed(String),

    #[error("Scheduling operation failed: {0}")]
    SchedulingFailed(String),

    #[error("Resource exhaustion: {resource} usage {current:.1}% exceeds limit {limit:.1}%")]
    ResourceExhaustion {
        resource: String,
        current: f64,
        limit: f64,
    },

    #[error("Persistence error: {0}")]
    Persistence(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Job scheduler error: {0}")]
    JobScheduler(String),

    #[error("Channel communication error: {0}")]
    Channel(String),
}

/// Resource usage limits for scheduling decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_cpu_percent: f64,
    pub max_memory_percent: f64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_percent: 20.0,
            max_memory_percent: 80.0,
        }
    }
}

/// Configuration for research task scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchSchedulerConfig {
    /// Interval for gap analysis scans
    pub gap_analysis_interval: Duration,
    /// Time-based scheduling intervals for different priorities
    pub time_based_intervals: HashMap<TaskPriority, Duration>,
    /// Scheduling intervals for different gap types
    pub gap_type_intervals: HashMap<GapType, Duration>,
    /// Maximum concurrent scheduling operations
    pub max_concurrent_schedules: usize,
    /// Resource usage limits for scheduling decisions
    pub resource_limits: ResourceLimits,
    /// Enable event-driven scheduling
    pub enable_event_driven: bool,
    /// Enable time-based scheduling
    pub enable_time_based: bool,
    /// File for persisting scheduler state
    pub scheduler_persistence_file: PathBuf,
}

impl Default for ResearchSchedulerConfig {
    fn default() -> Self {
        let mut time_based_intervals = HashMap::new();
        time_based_intervals.insert(TaskPriority::Critical, Duration::from_secs(30));
        time_based_intervals.insert(TaskPriority::High, Duration::from_secs(300));
        time_based_intervals.insert(TaskPriority::Medium, Duration::from_secs(1800));
        time_based_intervals.insert(TaskPriority::Low, Duration::from_secs(3600));

        let mut gap_type_intervals = HashMap::new();
        gap_type_intervals.insert(GapType::TodoComment, Duration::from_secs(120));
        gap_type_intervals.insert(GapType::ApiDocumentationGap, Duration::from_secs(60));
        gap_type_intervals.insert(GapType::UndocumentedTechnology, Duration::from_secs(90));
        gap_type_intervals.insert(GapType::MissingDocumentation, Duration::from_secs(300));
        gap_type_intervals.insert(GapType::ConfigurationGap, Duration::from_secs(600));

        Self {
            gap_analysis_interval: Duration::from_secs(300), // 5 minutes
            time_based_intervals,
            gap_type_intervals,
            max_concurrent_schedules: 5,
            resource_limits: ResourceLimits::default(),
            enable_event_driven: true,
            enable_time_based: true,
            scheduler_persistence_file: PathBuf::from("scheduler_state.json"),
        }
    }
}

/// Scheduling metrics and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerMetrics {
    pub total_scheduling_cycles: u64,
    pub event_driven_triggers: u64,
    pub time_based_triggers: u64,
    pub gaps_processed: u64,
    pub tasks_queued: u64,
    pub scheduling_errors: u64,
    pub resource_throttling_events: u64,
    pub average_scheduling_time: Duration,
    pub last_updated: DateTime<Utc>,
}

impl Default for SchedulerMetrics {
    fn default() -> Self {
        Self {
            total_scheduling_cycles: 0,
            event_driven_triggers: 0,
            time_based_triggers: 0,
            gaps_processed: 0,
            tasks_queued: 0,
            scheduling_errors: 0,
            resource_throttling_events: 0,
            average_scheduling_time: Duration::from_millis(0),
            last_updated: Utc::now(),
        }
    }
}

/// Scheduled job information
#[derive(Debug, Clone)]
pub struct ScheduledJob {
    pub id: Uuid,
    pub job_type: ScheduledJobType,
    pub priority: TaskPriority,
    pub gap_type: Option<GapType>,
    pub interval: Duration,
    pub next_run: DateTime<Utc>,
    pub enabled: bool,
}

/// Type of scheduled job
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduledJobType {
    TimeBasedAnalysis,
    PriorityBasedAnalysis,
    GapTypeBasedAnalysis,
    ResourceMonitoring,
}

/// Research task scheduler with configurable intervals
// Note: Debug trait not derived due to JobScheduler, TaskExecutor, and PriorityScorer not implementing Debug
pub struct ResearchScheduler {
    config: ResearchSchedulerConfig,
    /// Job scheduler for time-based operations
    job_scheduler: Arc<Mutex<Option<JobScheduler>>>,
    /// Background task queue
    queue: Arc<RwLock<Option<Arc<BackgroundScheduler>>>>,
    /// Task executor
    executor: Arc<RwLock<Option<Arc<TaskExecutor>>>>,
    /// State manager for enhanced tracking
    state_manager: Arc<RwLock<Option<Arc<StateManager>>>>,
    /// Event channel for file system events
    file_event_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<FileEvent>>>>,
    /// Priority scorer for intelligent gap prioritization
    priority_scorer: Arc<RwLock<Option<Arc<PriorityScorer>>>>,
    /// Scheduling metrics
    metrics: Arc<RwLock<SchedulerMetrics>>,
    /// Running state
    running: Arc<RwLock<bool>>,
    /// Scheduled jobs tracking
    scheduled_jobs: Arc<RwLock<HashMap<Uuid, ScheduledJob>>>,
    /// Last scheduling operations timestamps
    #[allow(dead_code)] // TODO: Will be used for operation timing and optimization
    last_operations: Arc<RwLock<HashMap<String, Instant>>>,
}

impl ResearchScheduler {
    /// Create a new research scheduler with the given configuration
    #[instrument(level = "debug", skip(config))]
    pub async fn new(config: ResearchSchedulerConfig) -> Result<Self, ResearchSchedulerError> {
        info!(
            "Initializing research scheduler with gap analysis interval: {:?}",
            config.gap_analysis_interval
        );

        // Validate configuration
        Self::validate_config(&config)?;

        let scheduler = Self {
            config,
            job_scheduler: Arc::new(Mutex::new(None)),
            queue: Arc::new(RwLock::new(None)),
            executor: Arc::new(RwLock::new(None)),
            state_manager: Arc::new(RwLock::new(None)),
            file_event_receiver: Arc::new(Mutex::new(None)),
            priority_scorer: Arc::new(RwLock::new(None)),
            metrics: Arc::new(RwLock::new(SchedulerMetrics::default())),
            running: Arc::new(RwLock::new(false)),
            scheduled_jobs: Arc::new(RwLock::new(HashMap::new())),
            last_operations: Arc::new(RwLock::new(HashMap::new())),
        };

        debug!("Research scheduler initialized successfully");
        Ok(scheduler)
    }

    /// Start the scheduler
    #[instrument(level = "debug", skip(self))]
    pub async fn start(&self) -> Result<(), ResearchSchedulerError> {
        let mut running = self.running.write().await;
        if *running {
            return Err(ResearchSchedulerError::AlreadyRunning);
        }
        *running = true;
        drop(running);

        info!("Starting research scheduler");

        // Initialize job scheduler if time-based scheduling is enabled
        if self.config.enable_time_based {
            self.start_job_scheduler().await?;
        }

        // Start event-driven scheduling if enabled
        if self.config.enable_event_driven {
            self.start_event_driven_processing().await;
        }

        // Start resource monitoring
        self.start_resource_monitoring().await;

        info!("Research scheduler started successfully");
        Ok(())
    }

    /// Stop the scheduler
    #[instrument(level = "debug", skip(self))]
    pub async fn stop(&self) -> Result<(), ResearchSchedulerError> {
        let mut running = self.running.write().await;
        if !*running {
            return Ok(());
        }
        *running = false;
        drop(running);

        info!("Stopping research scheduler");

        // Stop job scheduler
        {
            let mut job_scheduler = self.job_scheduler.lock().await;
            if let Some(mut scheduler) = job_scheduler.take() {
                if let Err(e) = scheduler.shutdown().await {
                    warn!("Error shutting down job scheduler: {}", e);
                }
            }
        }

        // Clear scheduled jobs
        {
            let mut jobs = self.scheduled_jobs.write().await;
            jobs.clear();
        }

        info!("Research scheduler stopped");
        Ok(())
    }

    /// Check if scheduler is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Configure the task queue
    pub async fn configure_queue(
        &self,
        queue: Arc<BackgroundScheduler>,
    ) -> Result<(), ResearchSchedulerError> {
        let mut queue_guard = self.queue.write().await;
        *queue_guard = Some(queue);
        debug!("Task queue configured for scheduler");
        Ok(())
    }

    /// Configure the task executor
    pub async fn configure_executor(
        &self,
        executor: Arc<TaskExecutor>,
    ) -> Result<(), ResearchSchedulerError> {
        let mut executor_guard = self.executor.write().await;
        *executor_guard = Some(executor);
        debug!("Task executor configured for scheduler");
        Ok(())
    }

    /// Configure the state manager
    pub async fn configure_state_manager(
        &self,
        state_manager: Arc<StateManager>,
    ) -> Result<(), ResearchSchedulerError> {
        let mut manager_guard = self.state_manager.write().await;
        *manager_guard = Some(state_manager);
        debug!("State manager configured for scheduler");
        Ok(())
    }

    /// Configure file event channel
    pub async fn configure_file_events(
        &self,
        receiver: mpsc::UnboundedReceiver<FileEvent>,
    ) -> Result<(), ResearchSchedulerError> {
        let mut receiver_guard = self.file_event_receiver.lock().await;
        *receiver_guard = Some(receiver);
        debug!("File event channel configured for scheduler");
        Ok(())
    }

    /// Configure the priority scorer
    pub async fn configure_priority_scorer(
        &self,
        priority_scorer: Arc<PriorityScorer>,
    ) -> Result<(), ResearchSchedulerError> {
        let mut scorer_guard = self.priority_scorer.write().await;
        *scorer_guard = Some(priority_scorer);
        debug!("Priority scorer configured for scheduler");
        Ok(())
    }

    /// Check if queue is configured
    pub async fn has_queue_configured(&self) -> bool {
        self.queue.read().await.is_some()
    }

    /// Check if executor is configured
    pub async fn has_executor_configured(&self) -> bool {
        self.executor.read().await.is_some()
    }

    /// Check if state manager is configured
    pub async fn has_state_manager_configured(&self) -> bool {
        self.state_manager.read().await.is_some()
    }

    /// Check if priority scorer is configured
    pub async fn has_priority_scorer_configured(&self) -> bool {
        self.priority_scorer.read().await.is_some()
    }

    /// Handle file system events for event-driven scheduling
    #[instrument(level = "debug", skip(self, event))]
    pub async fn handle_file_event(&self, event: FileEvent) -> Result<(), ResearchSchedulerError> {
        if !*self.running.read().await {
            return Err(ResearchSchedulerError::NotRunning);
        }

        debug!(
            "Handling file event: {:?} for file {:?}",
            event.event_type, event.path
        );

        // Check if this event should trigger gap analysis
        if self.should_process_file_event(&event).await {
            // Trigger gap analysis for the affected file
            if let Err(e) = self.trigger_gap_analysis_for_file(&event.path).await {
                error!(
                    "Failed to trigger gap analysis for file {:?}: {}",
                    event.path, e
                );
                self.increment_error_count().await;
                return Err(e);
            }

            // Update metrics
            let mut metrics = self.metrics.write().await;
            metrics.event_driven_triggers += 1;
            metrics.total_scheduling_cycles += 1;
            metrics.last_updated = Utc::now();
        }

        Ok(())
    }

    /// Process detected gaps through the scheduling pipeline
    #[instrument(level = "debug", skip(self, gaps))]
    pub async fn process_detected_gaps(
        &self,
        gaps: Vec<DetectedGap>,
    ) -> Result<(), ResearchSchedulerError> {
        if !*self.running.read().await {
            return Err(ResearchSchedulerError::NotRunning);
        }

        debug!("Processing {} detected gaps", gaps.len());

        let queue = {
            let queue_guard = self.queue.read().await;
            queue_guard
                .clone()
                .ok_or(ResearchSchedulerError::QueueNotConfigured)?
        };

        let mut tasks_queued = 0;
        let gaps_count = gaps.len();

        for gap in gaps {
            // Determine task priority based on gap type and other factors
            let priority = self.calculate_task_priority(&gap).await;

            // Create research task
            let task = ResearchTask::from_gap(gap, priority);

            // Queue the task
            match queue.enqueue(task).await {
                Ok(()) => {
                    tasks_queued += 1;
                    debug!("Successfully queued research task for gap");
                }
                Err(e) => {
                    error!("Failed to queue research task: {}", e);
                    self.increment_error_count().await;
                    return Err(ResearchSchedulerError::TaskQueuingFailed(e.to_string()));
                }
            }
        }

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.gaps_processed += gaps_count as u64;
        metrics.tasks_queued += tasks_queued;
        metrics.last_updated = Utc::now();

        info!(
            "Successfully processed {} gaps and queued {} tasks",
            gaps_count, tasks_queued
        );
        Ok(())
    }

    /// Check if scheduling should proceed based on current resource usage
    pub async fn should_schedule_now(&self, current_usage: ResourceUsage) -> bool {
        if current_usage.cpu_percent > self.config.resource_limits.max_cpu_percent {
            warn!(
                "Throttling scheduling due to high CPU usage: {:.1}%",
                current_usage.cpu_percent
            );
            return false;
        }

        if current_usage.memory_percent > self.config.resource_limits.max_memory_percent {
            warn!(
                "Throttling scheduling due to high memory usage: {:.1}%",
                current_usage.memory_percent
            );
            return false;
        }

        true
    }

    /// Get current scheduling metrics
    pub async fn get_metrics(&self) -> SchedulerMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Get scheduled intervals for priorities
    pub async fn get_scheduled_intervals(&self) -> HashMap<TaskPriority, Duration> {
        self.config.time_based_intervals.clone()
    }

    /// Get gap type intervals
    pub async fn get_gap_type_intervals(&self) -> HashMap<GapType, Duration> {
        self.config.gap_type_intervals.clone()
    }

    /// Validate configuration
    fn validate_config(config: &ResearchSchedulerConfig) -> Result<(), ResearchSchedulerError> {
        if config.max_concurrent_schedules == 0 {
            return Err(ResearchSchedulerError::Configuration(
                "max_concurrent_schedules must be greater than 0".to_string(),
            ));
        }

        if config.gap_analysis_interval.as_secs() == 0 {
            return Err(ResearchSchedulerError::Configuration(
                "gap_analysis_interval must be greater than 0".to_string(),
            ));
        }

        if config.resource_limits.max_cpu_percent <= 0.0
            || config.resource_limits.max_cpu_percent > 100.0
        {
            return Err(ResearchSchedulerError::Configuration(
                "max_cpu_percent must be between 0 and 100".to_string(),
            ));
        }

        if config.resource_limits.max_memory_percent <= 0.0
            || config.resource_limits.max_memory_percent > 100.0
        {
            return Err(ResearchSchedulerError::Configuration(
                "max_memory_percent must be between 0 and 100".to_string(),
            ));
        }

        Ok(())
    }

    /// Start job scheduler for time-based operations
    async fn start_job_scheduler(&self) -> Result<(), ResearchSchedulerError> {
        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| ResearchSchedulerError::JobScheduler(e.to_string()))?;

        // Schedule jobs for each priority level
        for (priority, interval) in &self.config.time_based_intervals {
            let job_id = Uuid::new_v4();
            let cron_expr = Self::duration_to_cron_expression(*interval);

            let scheduled_job = ScheduledJob {
                id: job_id,
                job_type: ScheduledJobType::PriorityBasedAnalysis,
                priority: *priority,
                gap_type: None,
                interval: *interval,
                next_run: Utc::now() + chrono::Duration::from_std(*interval).unwrap(),
                enabled: true,
            };

            // Store job info
            {
                let mut jobs = self.scheduled_jobs.write().await;
                jobs.insert(job_id, scheduled_job);
            }

            // Create the actual cron job
            let priority_for_job = *priority;
            let running = self.running.clone();
            let metrics = self.metrics.clone();

            let job = Job::new_async(cron_expr.as_str(), move |_uuid, _l| {
                let running = running.clone();
                let metrics = metrics.clone();
                Box::pin(async move {
                    if *running.read().await {
                        debug!(
                            "Executing time-based gap analysis for priority {:?}",
                            priority_for_job
                        );

                        // Update metrics
                        let mut metrics_guard = metrics.write().await;
                        metrics_guard.time_based_triggers += 1;
                        metrics_guard.total_scheduling_cycles += 1;
                        metrics_guard.last_updated = Utc::now();
                    }
                })
            })
            .map_err(|e| ResearchSchedulerError::JobScheduler(e.to_string()))?;

            scheduler
                .add(job)
                .await
                .map_err(|e| ResearchSchedulerError::JobScheduler(e.to_string()))?;
        }

        scheduler
            .start()
            .await
            .map_err(|e| ResearchSchedulerError::JobScheduler(e.to_string()))?;

        let mut job_scheduler_guard = self.job_scheduler.lock().await;
        *job_scheduler_guard = Some(scheduler);

        info!(
            "Time-based job scheduler started with {} jobs",
            self.config.time_based_intervals.len()
        );
        Ok(())
    }

    /// Convert duration to cron expression
    fn duration_to_cron_expression(duration: Duration) -> String {
        let seconds = duration.as_secs();

        if seconds < 60 {
            format!("*/{seconds} * * * * *")
        } else if seconds < 3600 {
            let minutes = seconds / 60;
            format!("0 */{minutes} * * * *")
        } else {
            let hours = seconds / 3600;
            format!("0 0 */{hours} * * *")
        }
    }

    /// Start event-driven processing
    async fn start_event_driven_processing(&self) {
        let running = self.running.clone();
        let file_event_receiver = self.file_event_receiver.clone();

        tokio::spawn(async move {
            while *running.read().await {
                let mut receiver_guard = file_event_receiver.lock().await;
                if let Some(receiver) = receiver_guard.as_mut() {
                    if let Some(event) = receiver.recv().await {
                        debug!("Received file event for processing: {:?}", event);
                        // Event processing would be handled here
                    }
                }
                drop(receiver_guard);

                // Small delay to prevent busy looping
                sleep(Duration::from_millis(100)).await;
            }
        });
    }

    /// Start resource monitoring
    async fn start_resource_monitoring(&self) {
        let running = self.running.clone();
        let metrics = self.metrics.clone();
        let resource_limits = self.config.resource_limits.clone();

        tokio::spawn(async move {
            let mut monitoring_interval = interval(Duration::from_secs(5));

            while *running.read().await {
                monitoring_interval.tick().await;

                // Get current resource usage (simplified for now)
                let current_usage = ResourceUsage {
                    cpu_percent: 10.0 + (rand::random::<f64>() * 10.0), // Mock: 10-20%
                    memory_percent: 30.0 + (rand::random::<f64>() * 20.0), // Mock: 30-50%
                    memory_mb: 500.0,
                    network_in_kb: 100.0,
                    network_out_kb: 50.0,
                    timestamp: Utc::now(),
                };

                // Check for resource throttling
                if current_usage.cpu_percent > resource_limits.max_cpu_percent
                    || current_usage.memory_percent > resource_limits.max_memory_percent
                {
                    let mut metrics_guard = metrics.write().await;
                    metrics_guard.resource_throttling_events += 1;
                    metrics_guard.last_updated = Utc::now();
                }
            }
        });
    }

    /// Check if a file event should trigger gap analysis
    async fn should_process_file_event(&self, event: &FileEvent) -> bool {
        // Only process certain file types and events
        match event.event_type {
            EventType::Write | EventType::Create => {
                // Check file extension
                if let Some(extension) = event.path.extension() {
                    matches!(
                        extension.to_str(),
                        Some("rs") | Some("md") | Some("toml") | Some("yaml") | Some("yml")
                    )
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Trigger gap analysis for a specific file
    async fn trigger_gap_analysis_for_file(
        &self,
        _file_path: &PathBuf,
    ) -> Result<(), ResearchSchedulerError> {
        // TODO: Integrate with actual gap analysis system
        // For now, just simulate the operation
        debug!("Triggering gap analysis for file: {:?}", _file_path);

        // Simulate some processing time
        sleep(Duration::from_millis(10)).await;

        Ok(())
    }

    /// Calculate task priority based on gap characteristics
    async fn calculate_task_priority(&self, gap: &DetectedGap) -> TaskPriority {
        // Use priority scorer if available for intelligent prioritization
        if let Some(scorer) = self.priority_scorer.read().await.as_ref() {
            match scorer.score_gap_priority(gap).await {
                Ok(breakdown) => {
                    debug!(
                        "Intelligent priority calculated for gap: {:.2} ({})",
                        breakdown.final_score,
                        breakdown.priority_level.to_string()
                    );
                    breakdown.priority_level
                }
                Err(e) => {
                    warn!(
                        "Priority scoring failed, falling back to gap priority: {}",
                        e
                    );
                    TaskPriority::from_u8(gap.priority)
                }
            }
        } else {
            // Fallback to simple priority mapping
            TaskPriority::from_u8(gap.priority)
        }
    }

    /// Increment error count in metrics
    async fn increment_error_count(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.scheduling_errors += 1;
        metrics.last_updated = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> ResearchSchedulerConfig {
        let temp_dir = TempDir::new().unwrap();
        ResearchSchedulerConfig {
            gap_analysis_interval: Duration::from_secs(30),
            time_based_intervals: HashMap::new(),
            gap_type_intervals: HashMap::new(),
            max_concurrent_schedules: 5,
            resource_limits: ResourceLimits::default(),
            enable_event_driven: true,
            enable_time_based: true,
            scheduler_persistence_file: temp_dir.path().join("scheduler_state.json"),
        }
    }

    #[tokio::test]
    async fn test_scheduler_creation() {
        let config = create_test_config();
        let scheduler = ResearchScheduler::new(config).await;
        assert!(scheduler.is_ok());

        let scheduler = scheduler.unwrap();
        assert!(!scheduler.is_running().await);
    }

    #[tokio::test]
    async fn test_scheduler_configuration_validation() {
        let mut config = create_test_config();
        config.max_concurrent_schedules = 0; // Invalid

        let result = ResearchScheduler::new(config).await;
        assert!(result.is_err());

        if let Err(ResearchSchedulerError::Configuration(msg)) = result {
            assert!(msg.contains("max_concurrent_schedules"));
        } else {
            panic!("Expected Configuration error");
        }
    }

    #[tokio::test]
    async fn test_duration_to_cron_conversion() {
        assert_eq!(
            ResearchScheduler::duration_to_cron_expression(Duration::from_secs(30)),
            "*/30 * * * * *"
        );
        assert_eq!(
            ResearchScheduler::duration_to_cron_expression(Duration::from_secs(120)),
            "0 */2 * * * *"
        );
        assert_eq!(
            ResearchScheduler::duration_to_cron_expression(Duration::from_secs(7200)),
            "0 0 */2 * * *"
        );
    }

    #[tokio::test]
    async fn test_resource_usage_checking() {
        let config = create_test_config();
        let scheduler = ResearchScheduler::new(config).await.unwrap();

        let low_usage = ResourceUsage {
            cpu_percent: 10.0,
            memory_percent: 50.0,
            memory_mb: 500.0,
            network_in_kb: 100.0,
            network_out_kb: 50.0,
            timestamp: Utc::now(),
        };

        assert!(scheduler.should_schedule_now(low_usage).await);

        let high_usage = ResourceUsage {
            cpu_percent: 25.0, // Above default limit of 20%
            memory_percent: 50.0,
            memory_mb: 500.0,
            network_in_kb: 100.0,
            network_out_kb: 50.0,
            timestamp: Utc::now(),
        };

        assert!(!scheduler.should_schedule_now(high_usage).await);
    }

    #[tokio::test]
    async fn test_scheduler_lifecycle() {
        let config = create_test_config();
        let scheduler = ResearchScheduler::new(config).await.unwrap();

        assert!(!scheduler.is_running().await);

        // Start should work
        let result = scheduler.start().await;
        assert!(result.is_ok());
        assert!(scheduler.is_running().await);

        // Stop should work
        let result = scheduler.stop().await;
        assert!(result.is_ok());
        assert!(!scheduler.is_running().await);
    }
}
