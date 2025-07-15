// ABOUTME: Integration tests for research task scheduling system with configurable intervals
//! This test module verifies the complete scheduling pipeline from gap analysis to task execution,
//! including event-driven and time-based scheduling modes, priority-based intervals, and
//! resource-aware scheduling logic.

use fortitude::proactive::{
    BackgroundScheduler, BackgroundSchedulerConfig, DetectedGap, EventType, FileEvent, GapType,
    QueueOperations, ResearchTask, TaskExecutor, TaskExecutorConfig, TaskPriority, TaskState,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

// TODO: These tests will initially fail - implement scheduler.rs to make them pass

#[tokio::test]
async fn test_research_scheduler_creation() {
    // FAILING TEST: Create research scheduler with default configuration
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_scheduler_config(temp_dir.path());

    let result = ResearchScheduler::new(config).await;
    assert!(result.is_ok());

    let scheduler = result.unwrap();
    assert!(!scheduler.is_running().await);
    assert!(scheduler.get_scheduled_intervals().await.is_empty());
}

#[tokio::test]
async fn test_event_driven_scheduling() {
    // FAILING TEST: Test event-driven scheduling triggered by file changes
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_scheduler_config(temp_dir.path());

    let scheduler = ResearchScheduler::new(config).await.unwrap();
    let file_event = create_test_file_event();

    // Should trigger gap analysis and queue research tasks
    let result = scheduler.handle_file_event(file_event).await;
    assert!(result.is_ok());

    // Verify gap analysis was triggered
    let metrics = scheduler.get_metrics().await;
    assert!(metrics.event_driven_triggers > 0);
}

#[tokio::test]
async fn test_time_based_scheduling() {
    // FAILING TEST: Test time-based scheduling with cron-like intervals
    let temp_dir = TempDir::new().unwrap();
    let mut config = create_test_scheduler_config(temp_dir.path());

    // Configure time-based scheduling every 5 seconds for high priority gaps
    config
        .time_based_intervals
        .insert(TaskPriority::High, Duration::from_secs(5));

    let scheduler = ResearchScheduler::new(config).await.unwrap();
    scheduler.start().await.unwrap();

    // Wait for time-based trigger
    sleep(Duration::from_secs(6)).await;

    let metrics = scheduler.get_metrics().await;
    assert!(metrics.time_based_triggers > 0);

    scheduler.stop().await.unwrap();
}

#[tokio::test]
async fn test_priority_based_intervals() {
    // FAILING TEST: Test different scheduling intervals for different priorities
    let temp_dir = TempDir::new().unwrap();
    let mut config = create_test_scheduler_config(temp_dir.path());

    // Configure different intervals for different priorities
    config
        .time_based_intervals
        .insert(TaskPriority::Critical, Duration::from_secs(1));
    config
        .time_based_intervals
        .insert(TaskPriority::High, Duration::from_secs(5));
    config
        .time_based_intervals
        .insert(TaskPriority::Medium, Duration::from_secs(30));
    config
        .time_based_intervals
        .insert(TaskPriority::Low, Duration::from_secs(300));

    let scheduler = ResearchScheduler::new(config).await.unwrap();
    let intervals = scheduler.get_scheduled_intervals().await;

    assert_eq!(intervals.len(), 4);
    assert_eq!(
        intervals.get(&TaskPriority::Critical),
        Some(&Duration::from_secs(1))
    );
    assert_eq!(
        intervals.get(&TaskPriority::Low),
        Some(&Duration::from_secs(300))
    );
}

#[tokio::test]
async fn test_pipeline_orchestration() {
    // FAILING TEST: Test complete pipeline from gap analysis to research execution
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_scheduler_config(temp_dir.path());

    let scheduler = ResearchScheduler::new(config).await.unwrap();

    // Create test gaps to trigger the pipeline
    let gaps = vec![create_test_gap()];

    // Should orchestrate: gap analysis → task queuing → background execution
    let result = scheduler.process_detected_gaps(gaps).await;
    assert!(result.is_ok());

    // Verify pipeline coordination
    let metrics = scheduler.get_metrics().await;
    assert!(metrics.gaps_processed > 0);
    assert!(metrics.tasks_queued > 0);
}

#[tokio::test]
async fn test_resource_aware_scheduling() {
    // FAILING TEST: Test scheduling adjusts based on system resource usage
    let temp_dir = TempDir::new().unwrap();
    let mut config = create_test_scheduler_config(temp_dir.path());

    // Configure resource limits
    config.resource_limits.max_cpu_percent = 15.0;
    config.resource_limits.max_memory_percent = 70.0;

    let scheduler = ResearchScheduler::new(config).await.unwrap();

    // Simulate high resource usage
    let high_usage = ResourceUsage {
        cpu_percent: 25.0, // Above limit
        memory_percent: 50.0,
        timestamp: chrono::Utc::now(),
        memory_mb: 1000.0,
        network_in_kb: 100.0,
        network_out_kb: 50.0,
    };

    // Should throttle scheduling under high resource usage
    let should_schedule = scheduler.should_schedule_now(high_usage).await;
    assert!(!should_schedule);
}

#[tokio::test]
async fn test_scheduler_integration_with_queue_and_executor() {
    // FAILING TEST: Test integration with existing queue and executor
    let temp_dir = TempDir::new().unwrap();
    let scheduler_config = create_test_scheduler_config(temp_dir.path());

    let queue_config = BackgroundSchedulerConfig {
        queue_file: temp_dir.path().join("test_queue.json"),
        max_queue_size: 100,
        persistence_interval: Duration::from_secs(30),
        max_concurrent_tasks: 3,
        default_timeout: Duration::from_secs(30),
    };

    let executor_config = TaskExecutorConfig::default();

    let queue = BackgroundScheduler::new(queue_config).await.unwrap();
    let executor = TaskExecutor::new(executor_config);
    let scheduler = ResearchScheduler::new(scheduler_config).await.unwrap();

    // Configure scheduler with queue and executor
    scheduler.configure_queue(Arc::new(queue)).await.unwrap();
    scheduler
        .configure_executor(Arc::new(executor))
        .await
        .unwrap();

    scheduler.start().await.unwrap();

    // Should integrate seamlessly
    assert!(scheduler.is_running().await);
    assert!(scheduler.has_queue_configured().await);
    assert!(scheduler.has_executor_configured().await);

    scheduler.stop().await.unwrap();
}

#[tokio::test]
async fn test_configurable_scheduling_intervals() {
    // FAILING TEST: Test configurable intervals for different gap types
    let temp_dir = TempDir::new().unwrap();
    let mut config = create_test_scheduler_config(temp_dir.path());

    // Configure different intervals for different gap types
    config
        .gap_type_intervals
        .insert(GapType::TodoComment, Duration::from_secs(60));
    config
        .gap_type_intervals
        .insert(GapType::MissingDocumentation, Duration::from_secs(300));
    config
        .gap_type_intervals
        .insert(GapType::UndocumentedTechnology, Duration::from_secs(30));

    let scheduler = ResearchScheduler::new(config).await.unwrap();
    let intervals = scheduler.get_gap_type_intervals().await;

    assert_eq!(intervals.len(), 3);
    assert_eq!(
        intervals.get(&GapType::TodoComment),
        Some(&Duration::from_secs(60))
    );
    assert_eq!(
        intervals.get(&GapType::UndocumentedTechnology),
        Some(&Duration::from_secs(30))
    );
}

#[tokio::test]
async fn test_scheduler_error_handling() {
    // FAILING TEST: Test comprehensive error handling for scheduling operations
    let temp_dir = TempDir::new().unwrap();
    let mut config = create_test_scheduler_config(temp_dir.path());

    // Configure invalid settings to trigger errors
    config.max_concurrent_schedules = 0; // Invalid

    let result = ResearchScheduler::new(config).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        ResearchSchedulerError::Configuration(msg) => {
            assert!(msg.contains("max_concurrent_schedules"));
        }
        _ => panic!("Expected Configuration error"),
    }
}

#[tokio::test]
async fn test_scheduler_metrics_and_monitoring() {
    // FAILING TEST: Test scheduler metrics collection and monitoring
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_scheduler_config(temp_dir.path());

    let scheduler = ResearchScheduler::new(config).await.unwrap();
    scheduler.start().await.unwrap();

    // Trigger some scheduling activity
    let file_event = create_test_file_event();
    scheduler.handle_file_event(file_event).await.unwrap();

    let metrics = scheduler.get_metrics().await;
    assert!(metrics.total_scheduling_cycles > 0);
    assert!(metrics.last_updated > chrono::Utc::now() - chrono::Duration::seconds(10));

    scheduler.stop().await.unwrap();
}

// Helper functions
fn create_test_scheduler_config(temp_path: &std::path::Path) -> ResearchSchedulerConfig {
    ResearchSchedulerConfig {
        gap_analysis_interval: Duration::from_secs(30),
        time_based_intervals: HashMap::new(),
        gap_type_intervals: HashMap::new(),
        max_concurrent_schedules: 5,
        resource_limits: ResourceLimits {
            max_cpu_percent: 20.0,
            max_memory_percent: 80.0,
        },
        enable_event_driven: true,
        enable_time_based: true,
        scheduler_persistence_file: temp_path.join("scheduler_state.json"),
    }
}

fn create_test_file_event() -> FileEvent {
    use std::time::SystemTime;

    FileEvent {
        path: PathBuf::from("test.rs"),
        event_type: EventType::Write,
        timestamp: SystemTime::now(),
        should_trigger_analysis: true,
        priority: 6,
    }
}

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

// Import types that will need to be implemented
use fortitude::proactive::{
    ResearchScheduler, ResearchSchedulerConfig, ResearchSchedulerError, ResourceLimits,
    ResourceUsage,
};
