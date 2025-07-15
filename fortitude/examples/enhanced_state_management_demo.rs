// ABOUTME: Demonstration of enhanced task state management system
//! This example demonstrates the comprehensive task state management system including:
//! - State manager initialization and configuration
//! - Integration with background scheduler, task executor, and research scheduler
//! - State transition tracking with metadata
//! - State persistence and recovery
//! - Real-time state monitoring and event handling

use fortitude::proactive::{
    BackgroundScheduler, BackgroundSchedulerConfig, DetectedGap, GapType, ResearchScheduler,
    ResearchSchedulerConfig, ResearchTask, StateEvent, StateManager, StateManagerConfig,
    StateTransitionMetadata, TaskExecutor, TaskExecutorConfig, TaskPriority, TaskState,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting enhanced state management demonstration");

    // Create temporary directory for persistence
    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path();

    // Initialize state manager
    let state_config = StateManagerConfig {
        persistence_file: base_path.join("state_manager.json"),
        persistence_interval: Duration::from_secs(2),
        max_history_entries: 50,
        enable_monitoring: true,
        ..StateManagerConfig::default()
    };

    let state_manager = Arc::new(StateManager::new(state_config).await?);
    state_manager.start().await?;
    info!("State manager initialized and started");

    // Initialize background scheduler
    let scheduler_config = BackgroundSchedulerConfig {
        queue_file: base_path.join("task_queue.json"),
        max_queue_size: 100,
        persistence_interval: Duration::from_secs(2),
        max_concurrent_tasks: 3,
        default_timeout: Duration::from_secs(30),
    };

    let scheduler = Arc::new(BackgroundScheduler::new(scheduler_config).await?);
    scheduler
        .configure_state_manager(state_manager.clone())
        .await?;
    info!("Background scheduler initialized with state manager");

    // Initialize task executor
    let executor_config = TaskExecutorConfig {
        max_concurrent_tasks: 3,
        api_calls_per_minute: 30,
        max_cpu_percent: 50.0,
        max_memory_percent: 80.0,
        task_timeout: Duration::from_secs(30),
        ..TaskExecutorConfig::default()
    };

    let executor = Arc::new(TaskExecutor::new(executor_config));
    executor
        .configure_state_manager(state_manager.clone())
        .await?;
    info!("Task executor initialized with state manager");

    // Initialize research scheduler
    let research_config = ResearchSchedulerConfig {
        gap_analysis_interval: Duration::from_secs(10),
        max_concurrent_schedules: 2,
        enable_event_driven: false,
        enable_time_based: false,
        scheduler_persistence_file: base_path.join("research_scheduler.json"),
        ..ResearchSchedulerConfig::default()
    };

    let research_scheduler = Arc::new(ResearchScheduler::new(research_config).await?);
    research_scheduler
        .configure_queue(scheduler.clone())
        .await?;
    research_scheduler
        .configure_executor(executor.clone())
        .await?;
    research_scheduler
        .configure_state_manager(state_manager.clone())
        .await?;
    info!("Research scheduler initialized with all components");

    // Subscribe to state events for monitoring
    let mut event_receiver = state_manager.subscribe_to_events();

    // Spawn task to monitor state events
    let monitoring_handle = tokio::spawn(async move {
        info!("Starting state event monitoring");
        while let Ok(event) = event_receiver.recv().await {
            match event {
                StateEvent::StateTransition {
                    task_id,
                    from_state,
                    to_state,
                    timestamp,
                    metadata,
                } => {
                    info!(
                        "State transition: {} {} -> {} at {} (actor: {})",
                        task_id,
                        format!("{:?}", from_state),
                        format!("{:?}", to_state),
                        timestamp.format("%H:%M:%S"),
                        metadata.actor
                    );
                }
                StateEvent::TaskCreated {
                    task_id,
                    timestamp,
                    priority,
                } => {
                    info!(
                        "Task created: {} with priority {:?} at {}",
                        task_id,
                        priority,
                        timestamp.format("%H:%M:%S")
                    );
                }
                StateEvent::TaskRecovered {
                    task_id,
                    previous_state,
                    recovered_state,
                    timestamp,
                } => {
                    info!(
                        "Task recovered: {} from {:?} to {:?} at {}",
                        task_id,
                        previous_state,
                        recovered_state,
                        timestamp.format("%H:%M:%S")
                    );
                }
                StateEvent::ValidationFailed {
                    task_id,
                    attempted_transition,
                    reason,
                    timestamp,
                } => {
                    info!(
                        "Validation failed: {} attempted {:?} -> {:?}: {} at {}",
                        task_id,
                        attempted_transition.0,
                        attempted_transition.1,
                        reason,
                        timestamp.format("%H:%M:%S")
                    );
                }
            }
        }
    });

    // Create some test tasks
    let tasks = create_demo_tasks();
    info!("Created {} demo tasks", tasks.len());

    // Demonstrate task lifecycle with state management
    for (i, task) in tasks.into_iter().enumerate() {
        let task_id = task.id.clone();

        // Enqueue task (this will trigger state tracking)
        scheduler.enqueue(task).await?;
        info!("Enqueued task {}: {}", i + 1, task_id);

        // Small delay to see state transitions clearly
        sleep(Duration::from_millis(500)).await;

        // Demonstrate manual state transition with metadata
        let metadata = StateTransitionMetadata {
            reason: format!("Manual transition for demo task {}", i + 1),
            actor: "demo_controller".to_string(),
            additional_data: {
                let mut data = HashMap::new();
                data.insert("demo_step".to_string(), format!("step_{}", i + 1));
                data.insert("manual_trigger".to_string(), "true".to_string());
                data
            },
            previous_state: Some(TaskState::Pending),
            validation_rules: vec!["demo_validation".to_string()],
        };

        // Update task to executing state
        if let Ok(Some(mut task)) = scheduler.get_task_by_id(&task_id).await {
            task.state = TaskState::Executing;
            scheduler
                .update_task_with_metadata(task, Some(metadata))
                .await?;
        }

        sleep(Duration::from_millis(300)).await;

        // Complete the task
        if let Ok(Some(mut task)) = scheduler.get_task_by_id(&task_id).await {
            task.state = TaskState::Completed;
            let completion_metadata = StateTransitionMetadata {
                reason: "Demo task completed".to_string(),
                actor: "demo_controller".to_string(),
                additional_data: {
                    let mut data = HashMap::new();
                    data.insert(
                        "completion_time".to_string(),
                        chrono::Utc::now().to_rfc3339(),
                    );
                    data.insert("demo_success".to_string(), "true".to_string());
                    data
                },
                previous_state: Some(TaskState::Executing),
                validation_rules: vec!["completion_validation".to_string()],
            };
            scheduler
                .update_task_with_metadata(task, Some(completion_metadata))
                .await?;
        }

        sleep(Duration::from_millis(200)).await;
    }

    // Wait for events to process
    sleep(Duration::from_secs(1)).await;

    // Demonstrate state querying and reporting
    info!("\n=== State Management Reporting ===");

    // Get overall metrics
    let metrics = state_manager.get_metrics().await;
    info!("State Manager Metrics:");
    info!("  Total transitions: {}", metrics.total_transitions);
    info!(
        "  Successful transitions: {}",
        metrics.successful_transitions
    );
    info!("  Failed transitions: {}", metrics.failed_transitions);
    info!(
        "  Average transition latency: {:?}",
        metrics.average_transition_latency
    );
    info!("  Error rate: {:.2}%", metrics.error_rate * 100.0);

    // Show task counts by state
    info!("Tasks by state:");
    for (state, count) in &metrics.tasks_by_state {
        if *count > 0 {
            info!("  {:?}: {}", state, count);
        }
    }

    // Demonstrate state history retrieval
    info!("\n=== Task State Histories ===");
    let pending_tasks = state_manager.get_tasks_by_state(TaskState::Pending).await;
    let executing_tasks = state_manager.get_tasks_by_state(TaskState::Executing).await;
    let completed_tasks = state_manager.get_tasks_by_state(TaskState::Completed).await;

    info!("Current task states:");
    info!("  Pending: {}", pending_tasks.len());
    info!("  Executing: {}", executing_tasks.len());
    info!("  Completed: {}", completed_tasks.len());

    // Show detailed history for one completed task
    if let Some(task_id) = completed_tasks.first() {
        if let Ok(history) = state_manager.get_task_state_history(task_id).await {
            info!("\nDetailed history for task {}:", task_id);
            for (i, entry) in history.iter().enumerate() {
                info!(
                    "  {}. {} -> {} at {} ({})",
                    i + 1,
                    format!("{:?}", entry.from_state),
                    format!("{:?}", entry.to_state),
                    entry.timestamp.format("%H:%M:%S"),
                    entry.metadata.reason
                );
            }
        }

        if let Ok(lifecycle) = state_manager.get_task_lifecycle(task_id).await {
            info!("\nLifecycle summary for task {}:", task_id);
            info!("  Created: {}", lifecycle.created_at.format("%H:%M:%S"));
            info!("  Current state: {:?}", lifecycle.current_state);
            info!("  Total transitions: {}", lifecycle.transitions.len());
            info!(
                "  Last updated: {}",
                lifecycle.last_updated.format("%H:%M:%S")
            );

            // Show time spent in each state
            info!("  Time in states:");
            for (state, duration) in &lifecycle.state_durations {
                info!("    {:?}: {:?}", state, duration);
            }
        }
    }

    // Demonstrate recovery functionality
    info!("\n=== Recovery Demonstration ===");
    let recovery_count = state_manager.perform_recovery().await?;
    info!(
        "Recovery operation completed: {} tasks recovered",
        recovery_count
    );

    // Demonstrate persistence
    info!("\n=== Persistence Demonstration ===");
    state_manager.stop().await?;
    info!("State manager stopped and state persisted");

    // Create new state manager and verify recovery
    let new_state_manager = Arc::new(
        StateManager::new(StateManagerConfig {
            persistence_file: base_path.join("state_manager.json"),
            ..StateManagerConfig::default()
        })
        .await?,
    );

    let recovered_metrics = new_state_manager.get_metrics().await;
    info!("Recovered state manager metrics:");
    info!(
        "  Total transitions: {}",
        recovered_metrics.total_transitions
    );
    info!("  Tasks by state: {:?}", recovered_metrics.tasks_by_state);

    // Stop monitoring
    monitoring_handle.abort();

    info!("Enhanced state management demonstration completed successfully!");
    Ok(())
}

fn create_demo_tasks() -> Vec<ResearchTask> {
    vec![
        create_task(
            "Documentation gap in module A",
            GapType::MissingDocumentation,
        ),
        create_task("TODO: Implement error handling", GapType::TodoComment),
        create_task(
            "Undocumented Rust async patterns",
            GapType::UndocumentedTechnology,
        ),
        create_task(
            "Missing API documentation for endpoints",
            GapType::ApiDocumentationGap,
        ),
        create_task(
            "Configuration options not documented",
            GapType::ConfigurationGap,
        ),
    ]
}

fn create_task(description: &str, gap_type: GapType) -> ResearchTask {
    let gap = DetectedGap {
        gap_type,
        file_path: PathBuf::from("demo.rs"),
        line_number: 42,
        column_number: Some(10),
        context: format!("Demo context: {}", description),
        description: description.to_string(),
        confidence: 0.8,
        priority: 7,
        metadata: HashMap::new(),
    };
    ResearchTask::from_gap(gap, TaskPriority::Medium)
}
