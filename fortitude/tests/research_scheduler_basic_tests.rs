// ABOUTME: Basic tests for research task scheduling system functionality
//! These tests verify the core scheduling functionality works correctly
//! including configuration, lifecycle management, and basic scheduling operations.

use fortitude::proactive::{
    GapType, ResearchScheduler, ResearchSchedulerConfig, ResourceLimits, ResourceUsage,
    TaskPriority,
};
use std::collections::HashMap;
use std::time::Duration;
use tempfile::TempDir;

fn create_test_config(temp_path: &std::path::Path) -> ResearchSchedulerConfig {
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

#[tokio::test]
async fn test_scheduler_creation_and_configuration() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(temp_dir.path());

    let scheduler = ResearchScheduler::new(config).await;
    assert!(scheduler.is_ok());

    let scheduler = scheduler.unwrap();
    assert!(!scheduler.is_running().await);
    assert!(scheduler.get_scheduled_intervals().await.is_empty());
}

#[tokio::test]
async fn test_scheduler_lifecycle() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(temp_dir.path());

    let scheduler = ResearchScheduler::new(config).await.unwrap();

    // Should start successfully
    assert!(!scheduler.is_running().await);
    let result = scheduler.start().await;
    assert!(result.is_ok());
    assert!(scheduler.is_running().await);

    // Should stop successfully
    let result = scheduler.stop().await;
    assert!(result.is_ok());
    assert!(!scheduler.is_running().await);
}

#[tokio::test]
async fn test_configurable_intervals() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = create_test_config(temp_dir.path());

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
async fn test_gap_type_intervals() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = create_test_config(temp_dir.path());

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
async fn test_resource_aware_scheduling() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = create_test_config(temp_dir.path());

    // Configure resource limits
    config.resource_limits.max_cpu_percent = 15.0;
    config.resource_limits.max_memory_percent = 70.0;

    let scheduler = ResearchScheduler::new(config).await.unwrap();

    // Low resource usage should allow scheduling
    let low_usage = ResourceUsage {
        cpu_percent: 10.0,
        memory_percent: 50.0,
        memory_mb: 500.0,
        network_in_kb: 100.0,
        network_out_kb: 50.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(scheduler.should_schedule_now(low_usage).await);

    // High resource usage should prevent scheduling
    let high_usage = ResourceUsage {
        cpu_percent: 25.0, // Above limit of 15%
        memory_percent: 50.0,
        memory_mb: 1000.0,
        network_in_kb: 100.0,
        network_out_kb: 50.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(!scheduler.should_schedule_now(high_usage).await);
}

#[tokio::test]
async fn test_scheduler_metrics() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(temp_dir.path());

    let scheduler = ResearchScheduler::new(config).await.unwrap();
    let metrics = scheduler.get_metrics().await;

    // Initial metrics should be zero
    assert_eq!(metrics.total_scheduling_cycles, 0);
    assert_eq!(metrics.event_driven_triggers, 0);
    assert_eq!(metrics.time_based_triggers, 0);
    assert_eq!(metrics.gaps_processed, 0);
    assert_eq!(metrics.tasks_queued, 0);
    assert_eq!(metrics.scheduling_errors, 0);
}

#[tokio::test]
async fn test_configuration_validation() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = create_test_config(temp_dir.path());

    // Invalid configuration should fail
    config.max_concurrent_schedules = 0;

    let result = ResearchScheduler::new(config).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_component_configuration() {
    let temp_dir = TempDir::new().unwrap();
    let config = create_test_config(temp_dir.path());

    let scheduler = ResearchScheduler::new(config).await.unwrap();

    // Initially no components configured
    assert!(!scheduler.has_queue_configured().await);
    assert!(!scheduler.has_executor_configured().await);
}

#[tokio::test]
async fn test_default_configuration_validity() {
    let temp_dir = TempDir::new().unwrap();
    let mut config = ResearchSchedulerConfig::default();
    config.scheduler_persistence_file = temp_dir.path().join("scheduler_state.json");

    // Default configuration should be valid
    let result = ResearchScheduler::new(config).await;
    assert!(result.is_ok());

    let scheduler = result.unwrap();

    // Should have default intervals configured
    let intervals = scheduler.get_scheduled_intervals().await;
    assert!(!intervals.is_empty());
    assert!(intervals.contains_key(&TaskPriority::Critical));
    assert!(intervals.contains_key(&TaskPriority::High));
    assert!(intervals.contains_key(&TaskPriority::Medium));
    assert!(intervals.contains_key(&TaskPriority::Low));

    let gap_intervals = scheduler.get_gap_type_intervals().await;
    assert!(!gap_intervals.is_empty());
    assert!(gap_intervals.contains_key(&GapType::TodoComment));
    assert!(gap_intervals.contains_key(&GapType::ApiDocumentationGap));
}
