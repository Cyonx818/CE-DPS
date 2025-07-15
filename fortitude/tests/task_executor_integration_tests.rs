//! Integration tests for task executor and background scheduler
//!
//! Tests the complete background task execution pipeline including:
//! - Queue integration with task executor
//! - Concurrent task execution
//! - Rate limiting and resource monitoring
//! - Progress tracking and metrics
//! - Error handling and retry logic

use fortitude::proactive::{
    BackgroundScheduler, BackgroundSchedulerConfig, DetectedGap, GapType, ResearchTask,
    TaskExecutor, TaskExecutorConfig, TaskExecutorError, TaskPriority,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

/// Helper to create test detected gap for task creation
fn create_test_gap(gap_type: GapType, priority: u8, file_path: &str) -> DetectedGap {
    DetectedGap {
        gap_type,
        file_path: PathBuf::from(file_path),
        line_number: 42,
        column_number: Some(10),
        context: "// TODO: Implement this feature".to_string(),
        description: "Missing implementation for critical feature".to_string(),
        confidence: 0.9,
        priority,
        metadata: HashMap::new(),
    }
}

/// Test basic task executor and scheduler integration
#[tokio::test]
async fn test_executor_scheduler_integration() {
    let temp_dir = TempDir::new().unwrap();
    let queue_file = temp_dir.path().join("task_queue.json");

    // Create scheduler
    let scheduler_config = BackgroundSchedulerConfig {
        queue_file,
        max_queue_size: 100,
        persistence_interval: Duration::from_secs(1),
        max_concurrent_tasks: 3,
        default_timeout: Duration::from_secs(30),
    };

    let scheduler = Arc::new(BackgroundScheduler::new(scheduler_config).await.unwrap());

    // Create executor
    let mut executor_config = TaskExecutorConfig::default();
    executor_config.max_concurrent_tasks = 3;
    executor_config.task_timeout = Duration::from_secs(10);
    let executor = TaskExecutor::new(executor_config);

    // Create and enqueue some test tasks
    let gap1 = create_test_gap(GapType::TodoComment, 8, "test1.rs");
    let task1 = ResearchTask::from_gap(gap1, TaskPriority::High);
    let task1_id = task1.id.clone();

    let gap2 = create_test_gap(GapType::MissingDocumentation, 6, "test2.rs");
    let task2 = ResearchTask::from_gap(gap2, TaskPriority::Medium);
    let task2_id = task2.id.clone();

    scheduler.enqueue(task1).await.unwrap();
    scheduler.enqueue(task2).await.unwrap();

    // Verify tasks are queued
    assert_eq!(scheduler.queue_size().await, 2);

    // Start executor manually (not using the automatic loop)
    // and execute tasks one by one
    let initial_metrics = executor.get_metrics().await;
    assert_eq!(initial_metrics.total_tasks_executed, 0);

    // Execute first task
    if let Some(task) = scheduler.dequeue().await.unwrap() {
        assert_eq!(task.id, task1_id);
        let result = executor.execute_task(task).await;
        assert!(result.is_ok());
    }

    // Execute second task
    if let Some(task) = scheduler.dequeue().await.unwrap() {
        assert_eq!(task.id, task2_id);
        let result = executor.execute_task(task).await;
        assert!(result.is_ok());
    }

    // Verify queue is empty
    assert_eq!(scheduler.queue_size().await, 0);

    // Verify metrics updated
    let final_metrics = executor.get_metrics().await;
    assert_eq!(final_metrics.total_tasks_executed, 2);
    assert_eq!(final_metrics.successful_tasks, 2);
    assert_eq!(final_metrics.failed_tasks, 0);
}

/// Test concurrent task execution
#[tokio::test]
async fn test_concurrent_task_execution() {
    let temp_dir = TempDir::new().unwrap();
    let queue_file = temp_dir.path().join("task_queue.json");

    let scheduler_config = BackgroundSchedulerConfig {
        queue_file,
        max_queue_size: 100,
        persistence_interval: Duration::from_secs(1),
        max_concurrent_tasks: 3,
        default_timeout: Duration::from_secs(30),
    };

    let scheduler = Arc::new(BackgroundScheduler::new(scheduler_config).await.unwrap());

    let mut executor_config = TaskExecutorConfig::default();
    executor_config.max_concurrent_tasks = 3;
    let executor = Arc::new(TaskExecutor::new(executor_config));

    // Create multiple tasks
    let mut tasks = Vec::new();
    for i in 0..5 {
        let gap = create_test_gap(GapType::TodoComment, 7, &format!("test{}.rs", i));
        let task = ResearchTask::from_gap(gap, TaskPriority::Medium);
        tasks.push(task);
    }

    // Execute tasks concurrently
    let mut handles = Vec::new();
    for task in tasks {
        let executor_clone = executor.clone();
        let handle = tokio::spawn(async move { executor_clone.execute_task(task).await });
        handles.push(handle);
    }

    // Wait for all tasks to complete with timeout
    let results: Vec<Result<Result<(), TaskExecutorError>, _>> =
        timeout(Duration::from_secs(10), futures::future::join_all(handles))
            .await
            .expect("Tasks should complete within timeout");

    // Verify all tasks completed successfully
    for result in results {
        let task_result = result.expect("Task handle should complete");
        assert!(task_result.is_ok(), "Task should execute successfully");
    }

    let final_metrics = executor.get_metrics().await;
    assert_eq!(final_metrics.total_tasks_executed, 5);
    assert_eq!(final_metrics.successful_tasks, 5);
}

/// Test rate limiting functionality
#[tokio::test]
async fn test_rate_limiting() {
    let mut executor_config = TaskExecutorConfig::default();
    executor_config.api_calls_per_minute = 2; // Very low limit
    executor_config.max_concurrent_tasks = 5; // Higher than rate limit
    let executor = TaskExecutor::new(executor_config);

    // Create multiple tasks
    let mut tasks = Vec::new();
    for i in 0..4 {
        let gap = create_test_gap(GapType::TodoComment, 7, &format!("test{}.rs", i));
        let task = ResearchTask::from_gap(gap, TaskPriority::High);
        tasks.push(task);
    }

    // Execute tasks rapidly - some should hit rate limit
    let mut results = Vec::new();
    for task in tasks {
        let result = executor.execute_task(task).await;
        results.push(result);
    }

    // Count successful vs rate limited
    let successful = results.iter().filter(|r| r.is_ok()).count();
    let rate_limited = results
        .iter()
        .filter(|r| matches!(r, Err(TaskExecutorError::RateLimitExceeded { .. })))
        .count();

    // Should have some rate limited tasks due to low limit
    assert!(rate_limited > 0, "Expected some tasks to be rate limited");
    assert!(successful > 0, "Expected some tasks to succeed");

    let metrics = executor.get_metrics().await;
    assert!(metrics.rate_limit_hits > 0);
}

/// Test resource constraint checking
#[tokio::test]
async fn test_resource_constraints() {
    let mut executor_config = TaskExecutorConfig::default();
    executor_config.max_cpu_percent = 5.0; // Very low limit to trigger constraint
    let executor = TaskExecutor::new(executor_config);

    let gap = create_test_gap(GapType::TodoComment, 7, "test.rs");
    let task = ResearchTask::from_gap(gap, TaskPriority::High);

    let result = executor.execute_task(task).await;

    // Should fail due to resource constraints (mock returns 10% CPU)
    assert!(result.is_err());
    match result {
        Err(TaskExecutorError::ResourceExhaustion { resource, .. }) => {
            assert_eq!(resource, "CPU");
        }
        _ => panic!("Expected ResourceExhaustion error"),
    }

    let metrics = executor.get_metrics().await;
    assert!(metrics.resource_throttling_events > 0);
}

/// Test progress tracking during task execution
#[tokio::test]
async fn test_progress_tracking() {
    let executor_config = TaskExecutorConfig::default();
    let executor = Arc::new(TaskExecutor::new(executor_config));

    let gap = create_test_gap(GapType::MissingDocumentation, 7, "test.rs");
    let task = ResearchTask::from_gap(gap, TaskPriority::High);
    let task_id = task.id.clone();

    // Start task execution in background
    let executor_clone = executor.clone();
    let handle = tokio::spawn(async move { executor_clone.execute_task(task).await });

    // Monitor progress during execution
    let mut progress_updates = Vec::new();
    for _ in 0..10 {
        let executing_tasks = executor.get_executing_tasks().await;
        if let Some(progress) = executing_tasks.iter().find(|p| p.task_id == task_id) {
            progress_updates.push((progress.stage.clone(), progress.progress_percent));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Wait for task to complete
    let result = handle
        .await
        .expect("Task should complete")
        .expect("Task should succeed");

    // Verify we captured progress updates
    assert!(
        progress_updates.len() > 0,
        "Should have captured progress updates"
    );

    // Verify progression through stages
    let has_executing = progress_updates
        .iter()
        .any(|(stage, _)| stage == "executing");
    let has_processing = progress_updates
        .iter()
        .any(|(stage, _)| stage == "processing");
    let has_completing = progress_updates
        .iter()
        .any(|(stage, _)| stage == "completing");

    assert!(
        has_executing || has_processing || has_completing,
        "Should have captured at least one stage"
    );

    // Verify final state - no executing tasks
    assert_eq!(executor.get_executing_tasks().await.len(), 0);
}

/// Test executor metrics accuracy
#[tokio::test]
async fn test_executor_metrics() {
    let executor_config = TaskExecutorConfig::default();
    let executor = TaskExecutor::new(executor_config);

    let initial_metrics = executor.get_metrics().await;
    assert_eq!(initial_metrics.total_tasks_executed, 0);
    assert_eq!(initial_metrics.successful_tasks, 0);
    assert_eq!(initial_metrics.failed_tasks, 0);

    // Execute successful task
    let gap = create_test_gap(GapType::TodoComment, 7, "test.rs");
    let task = ResearchTask::from_gap(gap, TaskPriority::High);
    let result = executor.execute_task(task).await;
    assert!(result.is_ok());

    let metrics_after_success = executor.get_metrics().await;
    assert_eq!(metrics_after_success.total_tasks_executed, 1);
    assert_eq!(metrics_after_success.successful_tasks, 1);
    assert_eq!(metrics_after_success.failed_tasks, 0);

    // Force a failure by triggering resource constraint
    let gap2 = create_test_gap(GapType::TodoComment, 7, "test2.rs");
    let task2 = ResearchTask::from_gap(gap2, TaskPriority::High);

    // Simulate high resource usage by setting very low limit temporarily
    // (This will fail in check_resource_constraints due to mock returning 10% CPU)
    let mut modified_config = TaskExecutorConfig::default();
    modified_config.max_cpu_percent = 5.0;
    let executor2 = TaskExecutor::new(modified_config);

    let result2 = executor2.execute_task(task2).await;
    assert!(result2.is_err());

    let metrics2 = executor2.get_metrics().await;
    assert_eq!(metrics2.total_tasks_executed, 0); // Task failed before execution
    assert_eq!(metrics2.resource_throttling_events, 1);
}
