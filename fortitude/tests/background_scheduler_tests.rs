//! Background scheduler tests for task queue system
//!
//! Tests persistent task queue system with priority ordering for background research tasks.
//! Verifies queue operations, persistence, state management, and integration with gap analysis.

use chrono::Utc;
use fortitude::proactive::{
    BackgroundScheduler, BackgroundSchedulerConfig, DetectedGap, EnhancedDetectedGap, GapType,
    PriorityBreakdown, QueueOperations, ResearchTask, SchedulerError, TaskPriority, TaskState,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::fs;

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

/// Test queue persistence across restarts
#[tokio::test]
async fn test_queue_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let queue_file = temp_dir.path().join("task_queue.json");

    // This test should fail initially as we haven't implemented the types yet
    // Creating a scope to test persistence across "restarts"
    let task_id = {
        // First "session" - create scheduler and add tasks
        let config = BackgroundSchedulerConfig {
            queue_file: queue_file.clone(),
            max_queue_size: 1000,
            persistence_interval: Duration::from_secs(1),
            max_concurrent_tasks: 5,
            default_timeout: Duration::from_secs(30),
        };

        let mut scheduler = BackgroundScheduler::new(config).await.unwrap();

        // Create test research task from gap analysis
        let gap = create_test_gap(GapType::TodoComment, 8, "src/main.rs");
        let task = ResearchTask::from_gap(gap, TaskPriority::High);
        let task_id = task.id.clone();

        // Add task to queue
        scheduler.enqueue(task).await.unwrap();

        // Force persistence
        scheduler.persist().await.unwrap();

        task_id
    };

    // Second "session" - load scheduler and verify task exists
    {
        let config = BackgroundSchedulerConfig {
            queue_file: queue_file.clone(),
            max_queue_size: 1000,
            persistence_interval: Duration::from_secs(1),
            max_concurrent_tasks: 5,
            default_timeout: Duration::from_secs(30),
        };

        let scheduler = BackgroundScheduler::new(config).await.unwrap();

        // Task should exist after restart
        assert_eq!(scheduler.queue_size().await, 1);

        let task = scheduler.peek().await.unwrap();
        assert!(task.is_some());
        assert_eq!(task.unwrap().id, task_id);
    }
}

/// Test priority ordering - higher priority tasks come first
#[tokio::test]
async fn test_priority_ordering() {
    let temp_dir = TempDir::new().unwrap();
    let queue_file = temp_dir.path().join("priority_queue.json");

    let config = BackgroundSchedulerConfig {
        queue_file,
        max_queue_size: 1000,
        persistence_interval: Duration::from_secs(10),
        max_concurrent_tasks: 5,
        default_timeout: Duration::from_secs(30),
    };

    let mut scheduler = BackgroundScheduler::new(config).await.unwrap();

    // Add tasks with different priorities
    let low_gap = create_test_gap(GapType::ConfigurationGap, 3, "config.toml");
    let medium_gap = create_test_gap(GapType::MissingDocumentation, 5, "lib.rs");
    let high_gap = create_test_gap(GapType::TodoComment, 9, "main.rs");

    let low_task = ResearchTask::from_gap(low_gap, TaskPriority::Low);
    let medium_task = ResearchTask::from_gap(medium_gap, TaskPriority::Medium);
    let high_task = ResearchTask::from_gap(high_gap, TaskPriority::High);

    // Add in random order
    scheduler.enqueue(medium_task.clone()).await.unwrap();
    scheduler.enqueue(low_task.clone()).await.unwrap();
    scheduler.enqueue(high_task.clone()).await.unwrap();

    // Should dequeue in priority order (High -> Medium -> Low)
    let first = scheduler.dequeue().await.unwrap().unwrap();
    assert_eq!(first.priority, TaskPriority::High);

    let second = scheduler.dequeue().await.unwrap().unwrap();
    assert_eq!(second.priority, TaskPriority::Medium);

    let third = scheduler.dequeue().await.unwrap().unwrap();
    assert_eq!(third.priority, TaskPriority::Low);
}

/// Test task state management and transitions
#[tokio::test]
async fn test_task_state_management() {
    let temp_dir = TempDir::new().unwrap();
    let queue_file = temp_dir.path().join("state_queue.json");

    let config = BackgroundSchedulerConfig {
        queue_file,
        max_queue_size: 1000,
        persistence_interval: Duration::from_secs(10),
        max_concurrent_tasks: 5,
        default_timeout: Duration::from_secs(30),
    };

    let mut scheduler = BackgroundScheduler::new(config).await.unwrap();

    let gap = create_test_gap(GapType::TodoComment, 7, "test.rs");
    let mut task = ResearchTask::from_gap(gap, TaskPriority::Medium);
    let task_id = task.id.clone();

    // Initial state should be Pending
    assert_eq!(task.state, TaskState::Pending);

    scheduler.enqueue(task).await.unwrap();

    // Start executing task
    let mut executing_task = scheduler.dequeue().await.unwrap().unwrap();
    executing_task.state = TaskState::Executing;
    executing_task.started_at = Some(Utc::now());

    scheduler.update_task(executing_task.clone()).await.unwrap();

    // Complete task
    executing_task.state = TaskState::Completed;
    executing_task.completed_at = Some(Utc::now());

    scheduler.update_task(executing_task).await.unwrap();

    // Verify task no longer in pending queue
    assert_eq!(scheduler.queue_size().await, 0);

    // Should be able to get completed task from history
    let completed_task = scheduler.get_task_by_id(&task_id).await.unwrap();
    assert!(completed_task.is_some());
    assert_eq!(completed_task.unwrap().state, TaskState::Completed);
}

/// Test concurrent queue operations (atomic operations)
#[tokio::test]
async fn test_concurrent_operations() {
    let temp_dir = TempDir::new().unwrap();
    let queue_file = temp_dir.path().join("concurrent_queue.json");

    let config = BackgroundSchedulerConfig {
        queue_file,
        max_queue_size: 1000,
        persistence_interval: Duration::from_secs(10),
        max_concurrent_tasks: 5,
        default_timeout: Duration::from_secs(30),
    };

    let scheduler = Arc::new(BackgroundScheduler::new(config).await.unwrap());

    // Spawn multiple tasks adding to queue concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let scheduler_clone = scheduler.clone();
        let handle = tokio::spawn(async move {
            let gap = create_test_gap(GapType::TodoComment, 5, &format!("file_{}.rs", i));
            let task = ResearchTask::from_gap(gap, TaskPriority::Medium);
            scheduler_clone.enqueue(task).await.unwrap();
        });
        handles.push(handle);
    }

    // Wait for all enqueue operations to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Should have exactly 10 tasks
    assert_eq!(scheduler.queue_size().await, 10);

    // Spawn multiple tasks dequeuing concurrently
    let mut handles = vec![];

    for _ in 0..5 {
        let scheduler_clone = scheduler.clone();
        let handle = tokio::spawn(async move { scheduler_clone.dequeue().await.unwrap() });
        handles.push(handle);
    }

    // Wait for all dequeue operations
    let mut dequeued_count = 0;
    for handle in handles {
        if handle.await.unwrap().is_some() {
            dequeued_count += 1;
        }
    }

    // Should have dequeued exactly 5 tasks
    assert_eq!(dequeued_count, 5);
    assert_eq!(scheduler.queue_size().await, 5);
}

/// Test queue performance requirements
#[tokio::test]
async fn test_queue_performance() {
    let temp_dir = TempDir::new().unwrap();
    let queue_file = temp_dir.path().join("perf_queue.json");

    let config = BackgroundSchedulerConfig {
        queue_file,
        max_queue_size: 10000,
        persistence_interval: Duration::from_secs(60),
        max_concurrent_tasks: 5,
        default_timeout: Duration::from_secs(30),
    };

    let mut scheduler = BackgroundScheduler::new(config).await.unwrap();

    // Test enqueue performance: should be <10ms per operation
    let start = Instant::now();

    for i in 0..100 {
        let gap = create_test_gap(GapType::TodoComment, 5, &format!("perf_{}.rs", i));
        let task = ResearchTask::from_gap(gap, TaskPriority::Medium);
        scheduler.enqueue(task).await.unwrap();
    }

    let enqueue_duration = start.elapsed();
    let avg_enqueue_time = enqueue_duration / 100;

    // Should average <10ms per enqueue operation
    assert!(
        avg_enqueue_time < Duration::from_millis(10),
        "Enqueue took {:?} on average, expected <10ms",
        avg_enqueue_time
    );

    // Test dequeue performance
    let start = Instant::now();

    for _ in 0..100 {
        scheduler.dequeue().await.unwrap();
    }

    let dequeue_duration = start.elapsed();
    let avg_dequeue_time = dequeue_duration / 100;

    // Should average <10ms per dequeue operation
    assert!(
        avg_dequeue_time < Duration::from_millis(10),
        "Dequeue took {:?} on average, expected <10ms",
        avg_dequeue_time
    );
}

/// Test persistence performance requirements
#[tokio::test]
async fn test_persistence_performance() {
    let temp_dir = TempDir::new().unwrap();
    let queue_file = temp_dir.path().join("persist_perf_queue.json");

    let config = BackgroundSchedulerConfig {
        queue_file,
        max_queue_size: 10000,
        persistence_interval: Duration::from_secs(60),
        max_concurrent_tasks: 5,
        default_timeout: Duration::from_secs(30),
    };

    let mut scheduler = BackgroundScheduler::new(config).await.unwrap();

    // Add 1000 tasks
    for i in 0..1000 {
        let gap = create_test_gap(GapType::TodoComment, 5, &format!("persist_{}.rs", i));
        let task = ResearchTask::from_gap(gap, TaskPriority::Medium);
        scheduler.enqueue(task).await.unwrap();
    }

    // Test persistence performance: should be <50ms
    let start = Instant::now();
    scheduler.persist().await.unwrap();
    let persist_duration = start.elapsed();

    assert!(
        persist_duration < Duration::from_millis(50),
        "Persistence took {:?}, expected <50ms",
        persist_duration
    );

    // Test load performance: should be <50ms
    let start = Instant::now();
    let new_scheduler = BackgroundScheduler::new(scheduler.config().clone())
        .await
        .unwrap();
    let load_duration = start.elapsed();

    assert!(
        load_duration < Duration::from_millis(50),
        "Load took {:?}, expected <50ms",
        load_duration
    );

    // Verify all tasks were loaded
    assert_eq!(new_scheduler.queue_size().await, 1000);
}

/// Test integration with gap analysis results
#[tokio::test]
async fn test_gap_analysis_integration() {
    let temp_dir = TempDir::new().unwrap();
    let queue_file = temp_dir.path().join("integration_queue.json");

    let config = BackgroundSchedulerConfig {
        queue_file,
        max_queue_size: 1000,
        persistence_interval: Duration::from_secs(10),
        max_concurrent_tasks: 5,
        default_timeout: Duration::from_secs(30),
    };

    let mut scheduler = BackgroundScheduler::new(config).await.unwrap();

    // Create enhanced detected gap from configurable analyzer
    let base_gap = create_test_gap(GapType::TodoComment, 7, "integration.rs");
    let enhanced_gap = EnhancedDetectedGap {
        gap: base_gap,
        enhanced_priority: 9,
        quality_score: 0.85,
        passed_filters: true,
        applied_rules: vec![],
        priority_breakdown: PriorityBreakdown::default(),
    };

    // Should be able to create research task from enhanced gap
    let task = ResearchTask::from_enhanced_gap(enhanced_gap, None);
    assert_eq!(task.priority, TaskPriority::High); // Should map from enhanced_priority 9
    assert!(task.metadata.contains_key("quality_score"));

    scheduler.enqueue(task).await.unwrap();

    // Task should be accessible and properly prioritized
    let queued_task = scheduler.peek().await.unwrap();
    assert!(queued_task.is_some());
    assert_eq!(queued_task.unwrap().priority, TaskPriority::High);
}

/// Test queue size limits and memory management
#[tokio::test]
async fn test_queue_limits() {
    let temp_dir = TempDir::new().unwrap();
    let queue_file = temp_dir.path().join("limits_queue.json");

    let config = BackgroundSchedulerConfig {
        queue_file,
        max_queue_size: 5, // Small limit for testing
        persistence_interval: Duration::from_secs(10),
        max_concurrent_tasks: 2,
        default_timeout: Duration::from_secs(30),
    };

    let mut scheduler = BackgroundScheduler::new(config).await.unwrap();

    // Add tasks up to limit
    for i in 0..5 {
        let gap = create_test_gap(GapType::TodoComment, 5, &format!("limit_{}.rs", i));
        let task = ResearchTask::from_gap(gap, TaskPriority::Medium);
        scheduler.enqueue(task).await.unwrap();
    }

    assert_eq!(scheduler.queue_size().await, 5);

    // Adding one more should fail or handle gracefully
    let gap = create_test_gap(GapType::TodoComment, 5, "overflow.rs");
    let task = ResearchTask::from_gap(gap, TaskPriority::Medium);

    let result = scheduler.enqueue(task).await;
    // Should either fail with QueueFull error or handle gracefully
    match result {
        Err(SchedulerError::QueueFull { .. }) => {
            // Expected behavior
        }
        Ok(()) => {
            // If it handles gracefully, queue size should still be 5
            assert_eq!(scheduler.queue_size().await, 5);
        }
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

/// Test error handling and recovery
#[tokio::test]
async fn test_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let queue_file = temp_dir.path().join("error_queue.json");

    let config = BackgroundSchedulerConfig {
        queue_file: queue_file.clone(),
        max_queue_size: 1000,
        persistence_interval: Duration::from_secs(10),
        max_concurrent_tasks: 5,
        default_timeout: Duration::from_secs(30),
    };

    // Test with corrupted queue file
    fs::write(&queue_file, "invalid json content")
        .await
        .unwrap();

    // Should handle corrupted file gracefully and start with empty queue
    let scheduler = BackgroundScheduler::new(config).await.unwrap();
    assert_eq!(scheduler.queue_size().await, 0);

    // Should be able to add tasks normally after recovery
    let gap = create_test_gap(GapType::TodoComment, 5, "recovery.rs");
    let task = ResearchTask::from_gap(gap, TaskPriority::Medium);
    scheduler.enqueue(task).await.unwrap();

    assert_eq!(scheduler.queue_size().await, 1);
}
