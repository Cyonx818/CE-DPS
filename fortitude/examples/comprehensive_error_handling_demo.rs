// ABOUTME: Comprehensive demonstration of error handling and retry mechanisms across proactive components
//! This example demonstrates the complete error handling system integrated across all proactive
//! research components, showing how failures are detected, classified, and recovered from automatically.

use fortitude::proactive::{
    error_handler::{CircuitBreakerConfig, ErrorHandler, ErrorHandlerConfig, ProactiveError},
    BackgroundScheduler, BackgroundSchedulerConfig, DetectedGap, GapType, ResearchTask,
    StateManager, StateManagerConfig, TaskExecutor, TaskExecutorConfig, TaskPriority, TaskState,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting comprehensive error handling demonstration");

    // Set up temporary directory for the demo
    let temp_dir = TempDir::new()?;

    // Configure all components with error-handling friendly settings
    let scheduler_config = BackgroundSchedulerConfig {
        queue_file: temp_dir.path().join("demo_queue.json"),
        max_queue_size: 10,
        persistence_interval: Duration::from_secs(2),
        max_concurrent_tasks: 3,
        default_timeout: Duration::from_secs(30),
    };

    let executor_config = TaskExecutorConfig {
        max_concurrent_tasks: 2,
        api_calls_per_minute: 30,
        max_cpu_percent: 80.0,
        max_memory_percent: 80.0,
        resource_check_interval: Duration::from_secs(1),
        task_timeout: Duration::from_secs(10),
        max_retries: 3,
        retry_initial_delay: Duration::from_millis(500),
        retry_max_delay: Duration::from_secs(10),
        retry_multiplier: 2.0,
        progress_report_interval: Duration::from_secs(2),
    };

    let state_config = StateManagerConfig {
        persistence_file: temp_dir.path().join("demo_state.json"),
        persistence_interval: Duration::from_secs(1),
        max_history_entries: 50,
        enable_monitoring: true,
        ..StateManagerConfig::default()
    };

    let mut error_handler_config = ErrorHandlerConfig::default();
    error_handler_config.circuit_breaker_configs.insert(
        "demo_service".to_string(),
        CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout: Duration::from_secs(5),
            half_open_max_calls: 3,
        },
    );

    // Create and start all components
    info!("Initializing components...");

    let background_scheduler = Arc::new(BackgroundScheduler::new(scheduler_config).await?);
    let task_executor = Arc::new(TaskExecutor::new(executor_config));
    let state_manager = Arc::new(StateManager::new(state_config).await?);
    let error_handler = Arc::new(ErrorHandler::new(error_handler_config));

    // Start all components
    state_manager.start().await?;
    error_handler.start().await?;

    // Configure error handling integration
    background_scheduler
        .configure_state_manager(state_manager.clone())
        .await?;
    background_scheduler
        .configure_error_handler(error_handler.clone())
        .await?;
    task_executor
        .configure_state_manager(state_manager.clone())
        .await?;

    info!("All components started successfully");

    // Demonstrate error handling scenarios

    // Scenario 1: Queue overflow handling
    info!("\n=== Scenario 1: Queue Overflow Handling ===");
    let mut tasks = Vec::new();

    // Create tasks to fill the queue beyond capacity
    for i in 0..15 {
        let gap = DetectedGap {
            gap_type: GapType::TodoComment,
            file_path: PathBuf::from(format!("demo_file_{}.rs", i)),
            line_number: 42 + i,
            column_number: Some(10),
            context: format!("// TODO: Implement feature {}", i),
            description: format!("Demo TODO comment {}", i),
            confidence: 0.8 + (i as f64 * 0.01),
            priority: 5 + (i % 3) as u8,
            metadata: HashMap::new(),
        };

        let task = ResearchTask::from_gap(gap, TaskPriority::Medium);
        tasks.push(task);
    }

    let mut successful_enqueues = 0;
    let mut failed_enqueues = 0;

    for task in tasks {
        match background_scheduler.enqueue(task).await {
            Ok(()) => {
                successful_enqueues += 1;
                info!("Task enqueued successfully");
            }
            Err(e) => {
                failed_enqueues += 1;
                warn!("Task enqueue failed (expected): {}", e);
            }
        }
    }

    info!(
        "Queue overflow test: {} successful, {} failed (expected)",
        successful_enqueues, failed_enqueues
    );

    // Scenario 2: State manager error handling with invalid transitions
    info!("\n=== Scenario 2: State Manager Error Handling ===");

    // Create a task and try invalid state transitions
    let demo_gap = DetectedGap {
        gap_type: GapType::MissingDocumentation,
        file_path: PathBuf::from("invalid_transition_demo.rs"),
        line_number: 100,
        column_number: Some(5),
        context: "fn undocumented_function() {}".to_string(),
        description: "Undocumented function".to_string(),
        confidence: 0.9,
        priority: 8,
        metadata: HashMap::new(),
    };

    let demo_task = ResearchTask::from_gap(demo_gap, TaskPriority::High);
    let demo_task_id = demo_task.id.clone();

    // Track task creation
    state_manager.track_task_creation(&demo_task).await?;
    info!("Task created and tracked: {}", demo_task_id);

    // Valid transition: Pending -> Executing
    match state_manager
        .transition_task(&demo_task_id, TaskState::Executing, "demo", None)
        .await
    {
        Ok(()) => info!("Valid transition succeeded: Pending -> Executing"),
        Err(e) => error!("Valid transition failed: {}", e),
    }

    // Invalid transition: Executing -> Pending (should fail in strict mode)
    match state_manager
        .transition_task(&demo_task_id, TaskState::Pending, "demo", None)
        .await
    {
        Ok(()) => warn!("Invalid transition unexpectedly succeeded"),
        Err(e) => info!("Invalid transition correctly failed: {}", e),
    }

    // Valid transition: Executing -> Completed
    match state_manager
        .transition_task(&demo_task_id, TaskState::Completed, "demo", None)
        .await
    {
        Ok(()) => info!("Valid transition succeeded: Executing -> Completed"),
        Err(e) => error!("Valid transition failed: {}", e),
    }

    // Scenario 3: Error handler retry mechanism
    info!("\n=== Scenario 3: Error Handler Retry Mechanism ===");

    let mut attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let attempt_count_clone = attempt_count.clone();

    // Simulate an operation that fails twice then succeeds
    let retry_operation = || {
        let count = attempt_count_clone.clone();
        async move {
            let current_attempt = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            info!("Retry operation attempt: {}", current_attempt + 1);

            if current_attempt < 2 {
                Err(ProactiveError::Transient {
                    message: format!("Simulated failure on attempt {}", current_attempt + 1),
                    attempt: current_attempt + 1,
                    max_attempts: 3,
                    retry_after: Some(Duration::from_millis(100)),
                })
            } else {
                Ok(format!("Success after {} attempts", current_attempt + 1))
            }
        }
    };

    match error_handler
        .execute_with_retry(
            retry_operation,
            "demo_retry_operation",
            Some("demo_service"),
        )
        .await
    {
        Ok(result) => info!("Retry operation succeeded: {}", result),
        Err(e) => error!("Retry operation failed: {}", e),
    }

    // Scenario 4: Circuit breaker demonstration
    info!("\n=== Scenario 4: Circuit Breaker Demonstration ===");

    // Simulate multiple failures to trip the circuit breaker
    let failure_operation = || async {
        Err(ProactiveError::ExternalService {
            service: "demo_external_api".to_string(),
            message: "Service unavailable".to_string(),
            status_code: Some(503),
            service_status: fortitude::proactive::error_handler::ServiceStatus::Unavailable,
        })
    };

    // First failure
    let result1: Result<String, ProactiveError> = error_handler
        .execute_with_retry(
            failure_operation,
            "failing_operation_1",
            Some("demo_service"),
        )
        .await;
    info!("First failure result: {:?}", result1.is_err());

    // Second failure - should trip circuit breaker
    let result2: Result<String, ProactiveError> = error_handler
        .execute_with_retry(
            failure_operation,
            "failing_operation_2",
            Some("demo_service"),
        )
        .await;
    info!("Second failure result: {:?}", result2.is_err());

    // Third attempt - should be blocked by circuit breaker
    let result3: Result<String, ProactiveError> = error_handler
        .execute_with_retry(
            failure_operation,
            "failing_operation_3",
            Some("demo_service"),
        )
        .await;
    match result3 {
        Err(ProactiveError::CircuitBreakerOpen { service, .. }) => {
            info!(
                "Circuit breaker correctly blocked request for service: {}",
                service
            );
        }
        _ => warn!("Circuit breaker did not activate as expected"),
    }

    // Wait for circuit breaker timeout
    info!("Waiting for circuit breaker timeout...");
    sleep(Duration::from_secs(6)).await;

    // Try again - should be in half-open state
    let success_operation = || async { Ok("Service recovered".to_string()) };
    let result4 = error_handler
        .execute_with_retry(
            success_operation,
            "recovery_operation",
            Some("demo_service"),
        )
        .await;
    match result4 {
        Ok(msg) => info!("Circuit breaker recovery succeeded: {}", msg),
        Err(e) => error!("Circuit breaker recovery failed: {}", e),
    }

    // Scenario 5: Task execution with comprehensive error handling
    info!("\n=== Scenario 5: Task Execution with Error Handling ===");

    // Dequeue tasks and attempt execution
    let mut execution_results = Vec::new();

    for _ in 0..3 {
        if let Ok(Some(task)) = background_scheduler.dequeue().await {
            let task_id = task.id.clone();
            info!("Executing task: {}", task_id);

            match task_executor.execute_task(task).await {
                Ok(()) => {
                    info!("Task {} executed successfully", task_id);
                    execution_results.push(("success", task_id));
                }
                Err(e) => {
                    warn!("Task {} execution failed: {}", task_id, e);
                    execution_results.push(("failed", task_id));
                }
            }
        } else {
            info!("No more tasks to execute");
            break;
        }
    }

    // Display final metrics and statistics
    info!("\n=== Final Metrics and Statistics ===");

    let error_metrics = error_handler.get_metrics().await;
    info!("Error Handler Metrics:");
    info!("  Total errors: {}", error_metrics.total_errors);
    info!(
        "  Dead letter entries: {}",
        error_metrics.dead_letter_entries
    );
    info!(
        "  Error classifications: {:?}",
        error_metrics.errors_by_classification
    );

    let state_metrics = state_manager.get_metrics().await;
    info!("State Manager Metrics:");
    info!("  Total transitions: {}", state_metrics.total_transitions);
    info!(
        "  Successful transitions: {}",
        state_metrics.successful_transitions
    );
    info!("  Failed transitions: {}", state_metrics.failed_transitions);
    info!("  Tasks by state: {:?}", state_metrics.tasks_by_state);

    let queue_metrics = background_scheduler.get_metrics().await;
    info!("Background Scheduler Metrics:");
    info!("  Current queue size: {}", queue_metrics.current_queue_size);
    info!(
        "  Total tasks processed: {}",
        queue_metrics.total_tasks_processed
    );
    info!("  Completed tasks: {}", queue_metrics.completed_tasks);
    info!("  Failed tasks: {}", queue_metrics.failed_tasks);

    let executor_metrics = task_executor.get_metrics().await;
    info!("Task Executor Metrics:");
    info!(
        "  Total tasks executed: {}",
        executor_metrics.total_tasks_executed
    );
    info!("  Successful tasks: {}", executor_metrics.successful_tasks);
    info!("  Failed tasks: {}", executor_metrics.failed_tasks);
    info!("  Rate limit hits: {}", executor_metrics.rate_limit_hits);

    // Display dead letter queue entries
    let dlq_entries = error_handler.get_dead_letter_entries().await;
    if !dlq_entries.is_empty() {
        info!("Dead Letter Queue Entries: {}", dlq_entries.len());
        for entry in dlq_entries.iter().take(3) {
            info!(
                "  Operation: {}, Error: {}",
                entry.original_operation, entry.final_error
            );
        }
    }

    // Cleanup
    info!("\n=== Cleanup ===");
    error_handler.stop().await?;
    state_manager.stop().await?;

    info!("Comprehensive error handling demonstration completed successfully!");
    info!("This demo showed:");
    info!("  ✓ Queue overflow protection with error handling");
    info!("  ✓ State transition validation and error recovery");
    info!("  ✓ Retry mechanisms with exponential backoff");
    info!("  ✓ Circuit breaker pattern for external service failures");
    info!("  ✓ Comprehensive error monitoring and metrics");
    info!("  ✓ Dead letter queue for permanently failed operations");

    Ok(())
}
