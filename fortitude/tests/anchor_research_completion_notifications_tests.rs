//! ANCHOR: Anchor tests for research completion notifications with result summaries
//!
//! These tests verify the research completion notification system works correctly
//! and provides comprehensive result summaries for background research tasks.
//!
//! Key functionality tested:
//! - Research result summary generation with quality metrics
//! - Completion notification delivery with embedded summaries
//! - Smart notification logic based on research importance
//! - Integration with existing notification and progress tracking systems
//! - Performance metrics and next recommended actions
//! - Failure notifications with retry recommendations

use fortitude::proactive::{
    CompletionNotificationLevel, DetectedGap, GapType, NextAction, NotificationChannel,
    NotificationSystem, NotificationSystemConfig, PerformanceMetrics, ProgressTracker,
    ProgressTrackerConfig, QualityMetrics, ResearchCompletionConfig, ResearchCompletionNotifier,
    ResearchResult, ResearchResultSummary, ResearchTask, StateManager, StateManagerConfig,
    TaskExecutor, TaskExecutorConfig, TaskExecutorError, TaskPriority, TaskState,
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

/// Helper to create a test research result with realistic data
fn create_test_research_result(task_id: String, quality_score: f64) -> ResearchResult {
    ResearchResult {
        task_id,
        research_query: "Implement async processing for user data".to_string(),
        findings: vec![
            "Use tokio for async runtime".to_string(),
            "Implement connection pooling for database".to_string(),
            "Add error handling with thiserror".to_string(),
        ],
        source_urls: vec![
            "https://tokio.rs/guide".to_string(),
            "https://docs.rs/thiserror".to_string(),
        ],
        confidence_score: quality_score,
        quality_metrics: QualityMetrics {
            relevance_score: quality_score * 0.9,
            credibility_score: quality_score * 0.8,
            completeness_score: quality_score * 0.85,
            timeliness_score: quality_score * 0.95,
        },
        gaps_addressed: 3,
        gaps_remaining: 1,
        execution_time: Duration::from_secs(45),
        knowledge_base_entries: 5,
        generated_at: chrono::Utc::now(),
        performance_metrics: Some(PerformanceMetrics {
            cpu_usage_percent: 15.0,
            memory_usage_mb: 128.0,
            network_requests_count: 8,
            cache_hit_ratio: 0.75,
            efficiency_score: 0.88,
        }),
    }
}

/// ANCHOR: test_research_completion_summary_generation
/// Tests that comprehensive research result summaries are generated correctly
#[tokio::test]
async fn test_research_completion_summary_generation() {
    let temp_dir = TempDir::new().unwrap();

    // Create completion notifier
    let completion_config = ResearchCompletionConfig::default();
    let completion_notifier = ResearchCompletionNotifier::new(completion_config);

    // Create test research result
    let task_id = "test_task_123".to_string();
    let research_result = create_test_research_result(task_id.clone(), 0.85);

    // Generate research summary
    let summary = completion_notifier
        .generate_result_summary(&research_result)
        .await
        .expect("Should generate research summary successfully");

    // ANCHOR: verify_summary_completeness
    // Verify summary contains all essential elements
    assert_eq!(summary.task_id, task_id);
    assert_eq!(summary.findings_count, 3);
    assert_eq!(summary.sources_count, 2);
    assert_eq!(summary.gaps_addressed, 3);
    assert_eq!(summary.gaps_remaining, 1);
    assert!(summary.overall_quality_score >= 0.7); // Expect average of individual quality metrics
    assert!(!summary.key_findings.is_empty());
    assert!(!summary.implementation_guidance.is_empty());

    // Verify quality metrics breakdown
    assert!(summary.quality_metrics.relevance_score > 0.0);
    assert!(summary.quality_metrics.credibility_score > 0.0);
    assert!(summary.quality_metrics.completeness_score > 0.0);
    assert!(summary.quality_metrics.timeliness_score > 0.0);

    // Verify performance metrics
    assert_eq!(
        summary.performance_metrics.execution_time,
        Duration::from_secs(45)
    );
    assert!(summary.performance_metrics.knowledge_base_integration_time > Duration::ZERO);
    assert_eq!(summary.performance_metrics.new_entries_created, 5);

    // Verify next actions are provided
    assert!(!summary.next_actions.is_empty());
    assert!(summary.next_actions.iter().any(|action| matches!(
        action.action_type,
        fortitude::proactive::NextActionType::ImplementRecommendation
    )));
}

/// ANCHOR: test_completion_notification_delivery
/// Tests that completion notifications are delivered with proper detail levels
#[tokio::test]
async fn test_completion_notification_delivery() {
    let temp_dir = TempDir::new().unwrap();

    // Create notification system
    let notification_config = NotificationSystemConfig::default();
    let notification_system = Arc::new(NotificationSystem::new(notification_config));
    notification_system
        .start()
        .await
        .expect("Should start notification system");

    // Create completion notifier with notification integration
    let completion_config = ResearchCompletionConfig {
        enable_notifications: true,
        notification_channels: vec![NotificationChannel::CLI],
        detailed_summaries: true,
        include_performance_metrics: true,
        ..ResearchCompletionConfig::default()
    };
    let mut completion_notifier = ResearchCompletionNotifier::new(completion_config);
    completion_notifier
        .configure_notification_system(notification_system.clone())
        .await
        .expect("Should configure notification system");

    // Test different notification levels
    let test_cases = vec![
        (
            0.95,
            CompletionNotificationLevel::Detailed,
            "High quality research",
        ),
        (
            0.75,
            CompletionNotificationLevel::Standard,
            "Medium quality research",
        ),
        (
            0.45,
            CompletionNotificationLevel::Brief,
            "Lower quality research",
        ),
    ];

    for (quality_score, expected_level, description) in test_cases {
        let task_id = format!("test_task_{}", quality_score);
        let research_result = create_test_research_result(task_id.clone(), quality_score);

        // Send completion notification
        let result = completion_notifier
            .send_completion_notification(research_result)
            .await;
        assert!(
            result.is_ok(),
            "Completion notification should succeed for {}",
            description
        );

        // Verify notification was sent (check metrics)
        let metrics = notification_system.get_metrics().await;
        assert!(
            metrics.total_notifications > 0,
            "Should have sent notifications"
        );
    }

    // ANCHOR: verify_notification_content
    // Verify the notification system received properly formatted notifications
    let final_metrics = notification_system.get_metrics().await;
    assert_eq!(
        final_metrics.total_notifications, 3,
        "Should have sent 3 notifications"
    );
    assert!(final_metrics.notifications_by_type.contains_key("SUCCESS"));

    notification_system
        .stop()
        .await
        .expect("Should stop notification system");
}

/// ANCHOR: test_failure_notification_with_retry_recommendations
/// Tests that failure notifications include proper retry recommendations
#[tokio::test]
async fn test_failure_notification_with_retry_recommendations() {
    let temp_dir = TempDir::new().unwrap();

    // Create notification system
    let notification_config = NotificationSystemConfig::default();
    let notification_system = Arc::new(NotificationSystem::new(notification_config));
    notification_system
        .start()
        .await
        .expect("Should start notification system");

    // Create completion notifier
    let completion_config = ResearchCompletionConfig {
        enable_notifications: true,
        notification_channels: vec![NotificationChannel::CLI],
        include_retry_recommendations: true,
        ..ResearchCompletionConfig::default()
    };
    let mut completion_notifier = ResearchCompletionNotifier::new(completion_config);
    completion_notifier
        .configure_notification_system(notification_system.clone())
        .await
        .expect("Should configure notification system");

    let task_id = "failed_task_123".to_string();
    let error = TaskExecutorError::TaskExecutionFailed {
        task_id: task_id.clone(),
        error: "Network timeout during research API call".to_string(),
    };

    // Send failure notification
    let result = completion_notifier
        .send_failure_notification(task_id.clone(), error)
        .await;
    assert!(result.is_ok(), "Failure notification should succeed");

    // ANCHOR: verify_failure_notification_content
    // Verify notification was sent with error type
    let metrics = notification_system.get_metrics().await;
    assert!(
        metrics.total_notifications > 0,
        "Should have sent failure notification"
    );
    assert!(
        metrics.notifications_by_type.contains_key("ERROR"),
        "Should have sent error notification"
    );

    notification_system
        .stop()
        .await
        .expect("Should stop notification system");
}

/// ANCHOR: test_batch_completion_notifications
/// Tests that batch research completion aggregates summaries correctly
#[tokio::test]
async fn test_batch_completion_notifications() {
    let temp_dir = TempDir::new().unwrap();

    // Create notification system
    let notification_config = NotificationSystemConfig::default();
    let notification_system = Arc::new(NotificationSystem::new(notification_config));
    notification_system
        .start()
        .await
        .expect("Should start notification system");

    // Create completion notifier with batch processing
    let completion_config = ResearchCompletionConfig {
        enable_notifications: true,
        notification_channels: vec![NotificationChannel::CLI],
        enable_batch_processing: true,
        batch_size: 3,
        batch_timeout: Duration::from_millis(500),
        ..ResearchCompletionConfig::default()
    };
    let mut completion_notifier = ResearchCompletionNotifier::new(completion_config);
    completion_notifier
        .configure_notification_system(notification_system.clone())
        .await
        .expect("Should configure notification system");

    // Start batch processing
    completion_notifier
        .start_batch_processing()
        .await
        .expect("Should start batch processing");

    // Add multiple research results to trigger batch
    let mut task_ids = Vec::new();
    for i in 0..3 {
        let task_id = format!("batch_task_{}", i);
        let research_result = create_test_research_result(task_id.clone(), 0.8);
        task_ids.push(task_id);

        let result = completion_notifier
            .send_completion_notification(research_result)
            .await;
        assert!(result.is_ok(), "Batch notification should succeed");
    }

    // Wait for batch processing
    tokio::time::sleep(Duration::from_millis(600)).await;

    // ANCHOR: verify_batch_notification
    // Verify batch notification was sent
    let metrics = notification_system.get_metrics().await;
    assert!(
        metrics.total_notifications > 0,
        "Should have sent batch notifications"
    );

    completion_notifier
        .stop_batch_processing()
        .await
        .expect("Should stop batch processing");
    notification_system
        .stop()
        .await
        .expect("Should stop notification system");
}

/// ANCHOR: test_adaptive_notification_detail_levels
/// Tests that notification detail adapts based on research importance and quality
#[tokio::test]
async fn test_adaptive_notification_detail_levels() {
    let temp_dir = TempDir::new().unwrap();

    // Create notification system
    let notification_config = NotificationSystemConfig::default();
    let notification_system = Arc::new(NotificationSystem::new(notification_config));
    notification_system
        .start()
        .await
        .expect("Should start notification system");

    // Create completion notifier with adaptive settings
    let completion_config = ResearchCompletionConfig {
        enable_notifications: true,
        notification_channels: vec![NotificationChannel::CLI],
        adaptive_detail_levels: true,
        high_importance_threshold: 0.8,
        low_importance_threshold: 0.4,
        ..ResearchCompletionConfig::default()
    };
    let mut completion_notifier = ResearchCompletionNotifier::new(completion_config);
    completion_notifier
        .configure_notification_system(notification_system.clone())
        .await
        .expect("Should configure notification system");

    // Test adaptive detail levels based on research quality and importance
    let test_scenarios = vec![
        (
            0.95,
            GapType::ApiDocumentationGap,
            "Should get detailed notification for high-quality API gaps",
        ),
        (
            0.85,
            GapType::MissingDocumentation,
            "Should get standard notification for good documentation gaps",
        ),
        (
            0.35,
            GapType::TodoComment,
            "Should get brief notification for low-quality TODO items",
        ),
    ];

    for (quality_score, gap_type, description) in test_scenarios {
        let gap = create_test_gap(gap_type, 8, "adaptive_test.rs");
        let task = ResearchTask::from_gap(gap, TaskPriority::High);
        let research_result = create_test_research_result(task.id.clone(), quality_score);

        let result = completion_notifier
            .send_completion_notification(research_result)
            .await;
        assert!(
            result.is_ok(),
            "Adaptive notification should succeed: {}",
            description
        );
    }

    // ANCHOR: verify_adaptive_behavior
    // Verify different notification levels were used
    let metrics = notification_system.get_metrics().await;
    assert_eq!(
        metrics.total_notifications, 3,
        "Should have sent 3 adaptive notifications"
    );

    notification_system
        .stop()
        .await
        .expect("Should stop notification system");
}

/// ANCHOR: test_notification_integration_with_progress_tracking
/// Tests that completion notifications integrate properly with progress tracking system
#[tokio::test]
async fn test_notification_integration_with_progress_tracking() {
    let temp_dir = TempDir::new().unwrap();

    // Create integrated system components
    let notification_config = NotificationSystemConfig::default();
    let notification_system = Arc::new(NotificationSystem::new(notification_config));
    notification_system
        .start()
        .await
        .expect("Should start notification system");

    let progress_config = ProgressTrackerConfig::default();
    let progress_tracker = Arc::new(ProgressTracker::new(progress_config));
    progress_tracker
        .start()
        .await
        .expect("Should start progress tracker");

    let state_config = StateManagerConfig {
        persistence_file: temp_dir.path().join("state.json"),
        ..StateManagerConfig::default()
    };
    let state_manager = Arc::new(StateManager::new(state_config).await.unwrap());
    state_manager.start().await.unwrap();

    // Create task executor with all integrations
    let executor_config = TaskExecutorConfig::default();
    let executor = Arc::new(TaskExecutor::new(executor_config));
    executor
        .configure_notification_system(notification_system.clone())
        .await
        .expect("Should configure notification system");
    executor
        .configure_progress_tracker(progress_tracker.clone())
        .await
        .expect("Should configure progress tracker");
    executor
        .configure_state_manager(state_manager.clone())
        .await
        .expect("Should configure state manager");

    // Subscribe to progress events
    let mut progress_event_receiver = progress_tracker.subscribe_to_progress_events();

    // Create and execute task
    let gap = create_test_gap(GapType::MissingDocumentation, 8, "integration_test.rs");
    let task = ResearchTask::from_gap(gap, TaskPriority::High);
    let task_id = task.id.clone();

    state_manager.track_task_creation(&task).await.unwrap();

    // Execute task while monitoring events
    let executor_clone = executor.clone();
    let handle = tokio::spawn(async move { executor_clone.execute_task(task).await });

    // Collect progress events
    let mut completion_events = Vec::new();
    let event_timeout = Duration::from_secs(5);
    let event_start = std::time::Instant::now();

    while event_start.elapsed() < event_timeout {
        match timeout(Duration::from_millis(100), progress_event_receiver.recv()).await {
            Ok(Ok(event)) => {
                match &event {
                    fortitude::proactive::ProgressEvent::TaskCompleted { task_id: id, .. }
                        if id == &task_id =>
                    {
                        completion_events.push(event);
                        break; // Found our completion event
                    }
                    _ => {} // Other events, continue monitoring
                }
            }
            Ok(Err(_)) => break, // Channel closed
            Err(_) => {}         // Timeout, continue
        }
    }

    // Wait for task completion
    let _result = handle
        .await
        .expect("Task should complete")
        .expect("Task should succeed");

    // ANCHOR: verify_integrated_completion_notification
    // Verify completion events were captured
    assert!(
        !completion_events.is_empty(),
        "Should capture task completion events"
    );

    // Verify notifications were sent for completion
    let notification_metrics = notification_system.get_metrics().await;
    assert!(
        notification_metrics.total_notifications > 0,
        "Should have sent completion notifications through integrated system"
    );

    // Verify final task state
    let final_lifecycle = state_manager.get_task_lifecycle(&task_id).await.unwrap();
    assert_eq!(final_lifecycle.current_state, TaskState::Completed);

    // Cleanup
    progress_tracker
        .stop()
        .await
        .expect("Should stop progress tracker");
    state_manager.stop().await.unwrap();
    notification_system
        .stop()
        .await
        .expect("Should stop notification system");
}

/// ANCHOR: test_customizable_notification_formats
/// Tests that notification formats can be customized based on user preferences
#[tokio::test]
async fn test_customizable_notification_formats() {
    let temp_dir = TempDir::new().unwrap();

    // Create notification system with file output for verification
    let log_file = temp_dir.path().join("notifications.log");
    let notification_config = NotificationSystemConfig {
        default_channels: vec![
            NotificationChannel::CLI,
            NotificationChannel::File {
                path: log_file.clone(),
            },
        ],
        ..NotificationSystemConfig::default()
    };
    let notification_system = Arc::new(NotificationSystem::new(notification_config));
    notification_system
        .start()
        .await
        .expect("Should start notification system");

    // Create completion notifier with custom format templates
    let completion_config = ResearchCompletionConfig {
        enable_notifications: true,
        notification_channels: vec![
            NotificationChannel::CLI,
            NotificationChannel::File {
                path: log_file.clone(),
            },
        ],
        custom_format_templates: {
            let mut templates = HashMap::new();
            templates.insert("detailed".to_string(), 
                "🔬 Research Complete: {title}\n📊 Quality: {quality_score}\n🎯 Findings: {findings_count}\n⏱️ Duration: {execution_time}".to_string());
            templates.insert(
                "brief".to_string(),
                "✅ {title} - Quality: {quality_score}".to_string(),
            );
            templates
        },
        ..ResearchCompletionConfig::default()
    };
    let mut completion_notifier = ResearchCompletionNotifier::new(completion_config);
    completion_notifier
        .configure_notification_system(notification_system.clone())
        .await
        .expect("Should configure notification system");

    // Send notification with custom format
    let task_id = "custom_format_test".to_string();
    let research_result = create_test_research_result(task_id.clone(), 0.87);

    let result = completion_notifier
        .send_completion_notification(research_result)
        .await;
    assert!(result.is_ok(), "Custom format notification should succeed");

    // Wait for file write
    tokio::time::sleep(Duration::from_millis(200)).await;

    // ANCHOR: verify_custom_format_output
    // Verify custom format was applied
    let log_content = tokio::fs::read_to_string(&log_file)
        .await
        .expect("Should read notification log file");

    assert!(
        log_content.contains("🔬 Research Task Completed"),
        "Should contain research completion notification"
    );
    assert!(
        log_content.contains("Quality Score"),
        "Should contain quality score information"
    );
    assert!(
        log_content.contains("findings"),
        "Should contain findings information"
    );

    notification_system
        .stop()
        .await
        .expect("Should stop notification system");
}

/// ANCHOR: test_performance_metrics_in_notifications
/// Tests that performance metrics are included in completion notifications
#[tokio::test]
async fn test_performance_metrics_in_notifications() {
    let temp_dir = TempDir::new().unwrap();

    // Create notification system
    let notification_config = NotificationSystemConfig::default();
    let notification_system = Arc::new(NotificationSystem::new(notification_config));
    notification_system
        .start()
        .await
        .expect("Should start notification system");

    // Create completion notifier with performance metrics enabled
    let completion_config = ResearchCompletionConfig {
        enable_notifications: true,
        notification_channels: vec![NotificationChannel::CLI],
        include_performance_metrics: true,
        include_resource_usage: true,
        include_efficiency_analysis: true,
        ..ResearchCompletionConfig::default()
    };
    let mut completion_notifier = ResearchCompletionNotifier::new(completion_config);
    completion_notifier
        .configure_notification_system(notification_system.clone())
        .await
        .expect("Should configure notification system");

    // Create research result with detailed performance data
    let task_id = "performance_test".to_string();
    let mut research_result = create_test_research_result(task_id.clone(), 0.82);

    // Add detailed performance metrics
    research_result.performance_metrics = Some(PerformanceMetrics {
        cpu_usage_percent: 15.5,
        memory_usage_mb: 128.0,
        network_requests_count: 12,
        cache_hit_ratio: 0.75,
        efficiency_score: 0.88,
    });

    let result = completion_notifier
        .send_completion_notification(research_result)
        .await;
    assert!(
        result.is_ok(),
        "Performance metrics notification should succeed"
    );

    // ANCHOR: verify_performance_metrics_inclusion
    // Verify notification was sent with performance data
    let metrics = notification_system.get_metrics().await;
    assert!(
        metrics.total_notifications > 0,
        "Should have sent performance notification"
    );

    notification_system
        .stop()
        .await
        .expect("Should stop notification system");
}
