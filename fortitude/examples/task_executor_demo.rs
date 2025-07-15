//! Task Executor Demo
//!
//! Demonstrates the background task executor with concurrency limits and rate limiting.
//! This example shows how tasks are queued, executed with resource monitoring,
//! and tracked through completion.

use fortitude::proactive::{
    BackgroundScheduler, BackgroundSchedulerConfig, DetectedGap, GapType, QueueOperations,
    ResearchTask, TaskExecutor, TaskExecutorConfig, TaskPriority,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::{interval, timeout};
use tracing::{error, info, warn};

/// Create a test gap for demonstration purposes
fn create_demo_gap(
    gap_type: GapType,
    priority: u8,
    file_path: &str,
    description: &str,
) -> DetectedGap {
    DetectedGap {
        gap_type,
        file_path: PathBuf::from(file_path),
        line_number: 42,
        column_number: Some(10),
        context: format!("// TODO: {}", description),
        description: description.to_string(),
        confidence: 0.85,
        priority,
        metadata: HashMap::new(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting Task Executor Demo");

    // Create temporary directory for queue persistence
    let temp_dir = TempDir::new()?;
    let queue_file = temp_dir.path().join("demo_task_queue.json");

    // Create scheduler configuration
    let scheduler_config = BackgroundSchedulerConfig {
        queue_file,
        max_queue_size: 50,
        persistence_interval: Duration::from_secs(2),
        max_concurrent_tasks: 5,
        default_timeout: Duration::from_secs(60),
    };

    // Create and initialize scheduler
    let scheduler = Arc::new(BackgroundScheduler::new(scheduler_config).await?);
    info!("Background scheduler initialized");

    // Create executor configuration with performance constraints
    let executor_config = TaskExecutorConfig {
        max_concurrent_tasks: 5,
        api_calls_per_minute: 30,
        max_cpu_percent: 20.0, // Target <20% CPU usage
        max_memory_percent: 80.0,
        resource_check_interval: Duration::from_secs(2),
        task_timeout: Duration::from_secs(30),
        max_retries: 3,
        retry_initial_delay: Duration::from_millis(500),
        retry_max_delay: Duration::from_secs(10),
        retry_multiplier: 2.0,
        progress_report_interval: Duration::from_secs(5),
    };

    let executor = Arc::new(TaskExecutor::new(executor_config));
    info!("Task executor initialized with concurrency limit: {}", 5);

    // Create demonstration tasks
    let demo_tasks = vec![
        (
            GapType::TodoComment,
            8,
            "api/auth.rs",
            "Implement OAuth2 authentication",
        ),
        (
            GapType::MissingDocumentation,
            7,
            "lib.rs",
            "Add comprehensive API documentation",
        ),
        (
            GapType::UndocumentedTechnology,
            6,
            "database/migrations.rs",
            "Document migration patterns",
        ),
        (
            GapType::ApiDocumentationGap,
            7,
            "handlers/user.rs",
            "Add REST API documentation",
        ),
        (
            GapType::ConfigurationGap,
            5,
            "config.yaml",
            "Document configuration options",
        ),
        (
            GapType::TodoComment,
            9,
            "security/validation.rs",
            "Add input validation",
        ),
        (
            GapType::MissingDocumentation,
            6,
            "models/user.rs",
            "Document data model",
        ),
        (
            GapType::TodoComment,
            8,
            "tests/integration.rs",
            "Add comprehensive tests",
        ),
    ];

    // Create and enqueue tasks
    info!(
        "Creating and enqueueing {} demonstration tasks",
        demo_tasks.len()
    );
    for (gap_type, priority, file_path, description) in demo_tasks {
        let gap = create_demo_gap(gap_type, priority, file_path, description);
        let task_priority = TaskPriority::from_u8(priority);
        let task = ResearchTask::from_gap(gap, task_priority);

        scheduler.enqueue(task.clone()).await?;
        info!(
            "Enqueued task: {} (priority: {:?})",
            task.research_query, task_priority
        );
    }

    let initial_queue_size = scheduler.queue_size().await;
    info!("Queue initialized with {} tasks", initial_queue_size);

    // Start monitoring and execution
    let scheduler_clone = scheduler.clone();
    let executor_clone = executor.clone();

    // Spawn task execution loop
    tokio::spawn(async move {
        let mut completed_tasks = 0;
        let expected_tasks = initial_queue_size;

        while completed_tasks < expected_tasks {
            // Check for available tasks
            match scheduler_clone.dequeue().await {
                Ok(Some(task)) => {
                    let task_id = task.id.clone();
                    let research_query = task.research_query.clone();

                    info!("Starting execution of task: {}", research_query);

                    // Execute task
                    match executor_clone.execute_task(task).await {
                        Ok(()) => {
                            completed_tasks += 1;
                            info!(
                                "Task completed successfully: {} ({}/{})",
                                research_query, completed_tasks, expected_tasks
                            );
                        }
                        Err(e) => {
                            error!("Task failed: {} - Error: {}", task_id, e);
                            completed_tasks += 1; // Count as processed even if failed
                        }
                    }
                }
                Ok(None) => {
                    // No tasks available, wait a bit
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Err(e) => {
                    error!("Failed to dequeue task: {}", e);
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                }
            }
        }

        info!("All tasks completed!");
    });

    // Monitor metrics and progress
    let mut metrics_interval = interval(Duration::from_secs(3));
    let mut iteration = 0;
    let max_iterations = 30; // Run for up to 90 seconds

    while iteration < max_iterations {
        metrics_interval.tick().await;
        iteration += 1;

        // Get current metrics
        let metrics = executor.get_metrics().await;
        let queue_size = scheduler.queue_size().await;
        let executing_tasks = executor.get_executing_tasks().await;
        let resource_usage = executor.get_resource_usage().await;

        info!("=== Executor Status (iteration {}) ===", iteration);
        info!("Queue size: {}", queue_size);
        info!("Currently executing: {}", executing_tasks.len());
        info!("Total processed: {}", metrics.total_tasks_executed);
        info!("Successful: {}", metrics.successful_tasks);
        info!("Failed: {}", metrics.failed_tasks);
        info!("Rate limit hits: {}", metrics.rate_limit_hits);
        info!(
            "Resource throttling events: {}",
            metrics.resource_throttling_events
        );

        if let Some(usage) = resource_usage.last() {
            info!(
                "Current resource usage - CPU: {:.1}%, Memory: {:.1}%",
                usage.cpu_percent, usage.memory_percent
            );
        }

        // Show executing task details
        if !executing_tasks.is_empty() {
            info!("Currently executing tasks:");
            for task in &executing_tasks {
                info!(
                    "  - {}: {} ({:.1}% complete)",
                    task.task_id, task.stage, task.progress_percent
                );
            }
        }

        // Check if all tasks are complete
        if queue_size == 0 && executing_tasks.is_empty() && metrics.total_tasks_executed > 0 {
            info!("All tasks completed! Breaking from monitoring loop.");
            break;
        }
    }

    // Final summary
    let final_metrics = executor.get_metrics().await;
    info!("=== Final Summary ===");
    info!(
        "Total tasks executed: {}",
        final_metrics.total_tasks_executed
    );
    info!("Successful tasks: {}", final_metrics.successful_tasks);
    info!("Failed tasks: {}", final_metrics.failed_tasks);
    info!(
        "Peak concurrency achieved: {}",
        final_metrics.peak_concurrency
    );
    info!("Average CPU usage: {:.1}%", final_metrics.cpu_usage_average);
    info!(
        "Average memory usage: {:.1}%",
        final_metrics.memory_usage_average
    );
    info!("Rate limit hits: {}", final_metrics.rate_limit_hits);
    info!(
        "Resource throttling events: {}",
        final_metrics.resource_throttling_events
    );

    // Validate performance requirements
    if final_metrics.cpu_usage_average <= 20.0 {
        info!(
            "✓ Performance requirement met: CPU usage {:.1}% <= 20%",
            final_metrics.cpu_usage_average
        );
    } else {
        warn!(
            "✗ Performance requirement NOT met: CPU usage {:.1}% > 20%",
            final_metrics.cpu_usage_average
        );
    }

    if final_metrics.total_tasks_executed >= 5 {
        info!(
            "✓ Concurrency requirement met: {} tasks executed",
            final_metrics.total_tasks_executed
        );
    } else {
        warn!(
            "✗ Concurrency requirement NOT met: only {} tasks executed",
            final_metrics.total_tasks_executed
        );
    }

    info!("Task Executor Demo completed successfully!");
    Ok(())
}
