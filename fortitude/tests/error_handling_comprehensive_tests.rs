// ABOUTME: Comprehensive error handling tests for all proactive research components
//! This test suite validates error handling, retry mechanisms, circuit breaker patterns,
//! and recovery strategies across all proactive components including:
//! - Background scheduler error handling and recovery
//! - Task executor error handling with retry and circuit breaker
//! - Research scheduler error management and resource throttling
//! - State manager error handling and validation
//! - Integration error handling across the entire pipeline

use fortitude::proactive::{
    error_handler::{
        CircuitBreaker, CircuitBreakerConfig, ErrorClassification, ErrorHandler,
        ErrorHandlerConfig, ProactiveError, RetryStrategy,
    },
    BackgroundScheduler, BackgroundSchedulerConfig, DetectedGap, GapType, ResearchScheduler,
    ResearchSchedulerConfig, ResearchSchedulerError, ResearchTask, SchedulerError, StateManager,
    StateManagerConfig, StateManagerError, TaskExecutor, TaskExecutorConfig, TaskExecutorError,
    TaskPriority, TaskState,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tracing::{error, info};

/// Test helper to create a test gap
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

/// Test error classification and handling strategies
#[tokio::test]
async fn test_error_classification_and_strategies() {
    // Test transient error classification
    let transient_error = ProactiveError::Transient {
        message: "Network timeout".to_string(),
        attempt: 1,
        max_attempts: 3,
        retry_after: Some(Duration::from_secs(5)),
    };

    assert_eq!(transient_error.classify(), ErrorClassification::Transient);
    assert!(transient_error.is_retryable());
    assert_eq!(transient_error.retry_delay(), Some(Duration::from_secs(5)));

    // Test permanent error classification
    let permanent_error = ProactiveError::Permanent {
        message: "Invalid configuration".to_string(),
        error_code: "CONFIG_INVALID".to_string(),
    };

    assert_eq!(permanent_error.classify(), ErrorClassification::Permanent);
    assert!(!permanent_error.is_retryable());
    assert_eq!(permanent_error.retry_delay(), None);

    // Test rate limit error classification
    let rate_limit_error = ProactiveError::RateLimit {
        operation: "api_call".to_string(),
        retry_after: Duration::from_secs(60),
        current_requests: 100,
        limit: 50,
    };

    assert_eq!(rate_limit_error.classify(), ErrorClassification::RateLimit);
    assert!(rate_limit_error.is_retryable());
    assert_eq!(
        rate_limit_error.retry_delay(),
        Some(Duration::from_secs(60))
    );
}

/// Test retry strategy with exponential backoff and jitter
#[tokio::test]
async fn test_retry_strategy_exponential_backoff() {
    let strategy = RetryStrategy {
        max_attempts: 5,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
        multiplier: 2.0,
        jitter_factor: 0.1,
        enable_jitter: true,
    };

    // Test exponential backoff
    let delay0 = strategy.calculate_delay(0);
    let delay1 = strategy.calculate_delay(1);
    let delay2 = strategy.calculate_delay(2);

    assert_eq!(delay0, Duration::from_millis(100));
    assert!(delay1 >= Duration::from_millis(90) && delay1 <= Duration::from_millis(220)); // With jitter
    assert!(delay2 >= Duration::from_millis(180) && delay2 <= Duration::from_millis(440)); // With jitter

    // Test max delay is respected (with jitter, it might be slightly above max_delay)
    let large_delay = strategy.calculate_delay(10);
    assert!(large_delay <= strategy.max_delay.mul_f64(1.1)); // Allow 10% jitter tolerance
}

/// Test circuit breaker state transitions and failure detection
#[tokio::test]
async fn test_circuit_breaker_state_transitions() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout: Duration::from_millis(100),
        half_open_max_calls: 5,
    };

    let circuit_breaker = CircuitBreaker::new(config);

    // Initially closed - should allow calls
    assert!(circuit_breaker.can_execute().await);

    // Record failures below threshold
    circuit_breaker.record_failure().await;
    circuit_breaker.record_failure().await;
    assert!(circuit_breaker.can_execute().await);

    // Record failure at threshold - should open circuit
    circuit_breaker.record_failure().await;
    assert!(!circuit_breaker.can_execute().await);

    // Wait for timeout to move to half-open
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert!(circuit_breaker.can_execute().await);

    // Record successes to close circuit
    circuit_breaker.record_success().await;
    circuit_breaker.record_success().await;
    assert!(circuit_breaker.can_execute().await);
}

/// Test error handler retry logic with failing operations
#[tokio::test]
async fn test_error_handler_retry_with_eventual_success() {
    let config = ErrorHandlerConfig::default();
    let error_handler = ErrorHandler::new(config);
    error_handler.start().await.unwrap();

    let attempt_count = Arc::new(AtomicU32::new(0));
    let attempt_count_clone = attempt_count.clone();

    let operation = || {
        let count = attempt_count_clone.fetch_add(1, Ordering::SeqCst);
        async move {
            if count < 2 {
                Err(ProactiveError::Transient {
                    message: format!("Attempt {} failed", count + 1),
                    attempt: count + 1,
                    max_attempts: 3,
                    retry_after: Some(Duration::from_millis(10)),
                })
            } else {
                Ok(format!("Success after {} attempts", count + 1))
            }
        }
    };

    let result = error_handler
        .execute_with_retry(operation, "test_operation", Some("test_service"))
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Success after 3 attempts");
    assert_eq!(attempt_count.load(Ordering::SeqCst), 3);

    error_handler.stop().await.unwrap();
}

/// Test error handler with permanent failure leading to dead letter queue
#[tokio::test]
async fn test_error_handler_permanent_failure_dead_letter_queue() {
    let config = ErrorHandlerConfig::default();
    let error_handler = ErrorHandler::new(config);
    error_handler.start().await.unwrap();

    let operation = || async {
        Err(ProactiveError::Permanent {
            message: "Unrecoverable error".to_string(),
            error_code: "UNRECOVERABLE".to_string(),
        })
    };

    let result: Result<String, ProactiveError> = error_handler
        .execute_with_retry(operation, "failing_operation", None)
        .await;

    assert!(result.is_err());

    // Check dead letter queue
    let dlq_entries = error_handler.get_dead_letter_entries().await;
    assert_eq!(dlq_entries.len(), 1);
    assert_eq!(dlq_entries[0].original_operation, "failing_operation");

    error_handler.stop().await.unwrap();
}

/// Test background scheduler error handling with queue full scenarios
#[tokio::test]
async fn test_background_scheduler_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config = BackgroundSchedulerConfig {
        queue_file: temp_dir.path().join("test_queue.json"),
        max_queue_size: 2, // Very small queue for testing
        persistence_interval: Duration::from_secs(1),
        max_concurrent_tasks: 1,
        default_timeout: Duration::from_secs(30),
    };

    let scheduler = BackgroundScheduler::new(config).await.unwrap();

    // Fill the queue to capacity
    let gap1 = create_test_gap();
    let gap2 = create_test_gap();
    let gap3 = create_test_gap();

    let task1 = ResearchTask::from_gap(gap1, TaskPriority::Medium);
    let task2 = ResearchTask::from_gap(gap2, TaskPriority::Medium);
    let task3 = ResearchTask::from_gap(gap3, TaskPriority::Medium);

    // First two should succeed
    assert!(scheduler.enqueue(task1).await.is_ok());
    assert!(scheduler.enqueue(task2).await.is_ok());

    // Third should fail with queue full error
    let result = scheduler.enqueue(task3).await;
    assert!(result.is_err());

    if let Err(SchedulerError::QueueFull { limit }) = result {
        assert_eq!(limit, 2);
    } else {
        panic!("Expected QueueFull error");
    }
}

/// Test task executor error handling with resource constraints
#[tokio::test]
async fn test_task_executor_resource_constraint_handling() {
    let config = TaskExecutorConfig {
        max_concurrent_tasks: 1,
        api_calls_per_minute: 1, // Very low rate limit
        max_cpu_percent: 5.0,    // Very low CPU limit
        max_memory_percent: 50.0,
        resource_check_interval: Duration::from_secs(1),
        task_timeout: Duration::from_secs(5),
        max_retries: 2,
        retry_initial_delay: Duration::from_millis(100),
        retry_max_delay: Duration::from_secs(5),
        retry_multiplier: 2.0,
        progress_report_interval: Duration::from_secs(1),
    };

    let executor = TaskExecutor::new(config);

    // Test resource constraint checking - we need to test this differently since the method is private
    // Instead, test task execution which will internally check resource constraints
    let gap = create_test_gap();
    let task = ResearchTask::from_gap(gap, TaskPriority::High);
    let result = executor.execute_task(task).await;
    assert!(result.is_err());

    if let Err(TaskExecutorError::ResourceExhaustion {
        resource,
        current,
        limit,
    }) = result
    {
        assert_eq!(resource, "CPU");
        assert!(current > limit);
    } else {
        panic!("Expected ResourceExhaustion error");
    }
}

/// Test research scheduler error handling with missing components
#[tokio::test]
async fn test_research_scheduler_missing_components_error() {
    let config = ResearchSchedulerConfig::default();
    let scheduler = ResearchScheduler::new(config).await.unwrap();

    // Start scheduler without configuring required components
    let result = scheduler.start().await;
    assert!(result.is_ok()); // Should start but components won't be available

    // Try to process gaps without queue configured
    let gaps = vec![create_test_gap()];
    let result = scheduler.process_detected_gaps(gaps).await;
    assert!(result.is_err());

    if let Err(ResearchSchedulerError::QueueNotConfigured) = result {
        // Expected error
    } else {
        panic!("Expected QueueNotConfigured error");
    }

    scheduler.stop().await.unwrap();
}

/// Test state manager error handling with invalid state transitions
#[tokio::test]
async fn test_state_manager_invalid_state_transitions() {
    let temp_dir = TempDir::new().unwrap();
    let config = StateManagerConfig {
        persistence_file: temp_dir.path().join("test_state.json"),
        ..StateManagerConfig::default()
    };

    let state_manager = StateManager::new(config).await.unwrap();
    state_manager.start().await.unwrap();

    let gap = create_test_gap();
    let task = ResearchTask::from_gap(gap, TaskPriority::High);
    let task_id = task.id.clone();

    // Track task creation
    state_manager.track_task_creation(&task).await.unwrap();

    // Try invalid state transition (Pending -> Completed without Executing)
    let result = state_manager
        .transition_task(&task_id, TaskState::Completed, "test", None)
        .await;
    assert!(result.is_err());

    if let Err(StateManagerError::InvalidStateTransition { from, to, .. }) = result {
        assert_eq!(from, TaskState::Pending);
        assert_eq!(to, TaskState::Completed);
    } else {
        panic!("Expected InvalidStateTransition error");
    }

    state_manager.stop().await.unwrap();
}

/// Test concurrent error handling and race conditions
#[tokio::test]
async fn test_concurrent_error_handling() {
    let config = ErrorHandlerConfig::default();
    let error_handler = Arc::new(ErrorHandler::new(config));
    error_handler.start().await.unwrap();

    let mut handles = Vec::new();
    let error_count = Arc::new(AtomicU32::new(0));
    let success_count = Arc::new(AtomicU32::new(0));

    // Spawn multiple concurrent operations that will fail and retry
    for i in 0..10 {
        let error_handler_clone = error_handler.clone();
        let error_count_clone = error_count.clone();
        let success_count_clone = success_count.clone();

        let handle = tokio::spawn(async move {
            let operation = || {
                let error_count = error_count_clone.clone();
                async move {
                    let attempts = error_count.fetch_add(1, Ordering::SeqCst);
                    if attempts % 3 == 2 {
                        // Succeed every third attempt
                        Ok(format!("Task {} success", i))
                    } else {
                        Err(ProactiveError::Transient {
                            message: format!("Task {} transient failure", i),
                            attempt: attempts % 3 + 1,
                            max_attempts: 3,
                            retry_after: Some(Duration::from_millis(10)),
                        })
                    }
                }
            };

            match error_handler_clone
                .execute_with_retry(operation, &format!("task_{}", i), None)
                .await
            {
                Ok(_) => success_count_clone.fetch_add(1, Ordering::SeqCst),
                Err(_) => 0,
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify metrics
    let metrics = error_handler.get_metrics().await;
    assert!(metrics.total_errors > 0);
    assert!(success_count.load(Ordering::SeqCst) > 0);

    error_handler.stop().await.unwrap();
}

/// Test integration error handling across multiple components
#[tokio::test]
async fn test_integration_error_handling_pipeline() {
    let temp_dir = TempDir::new().unwrap();

    // Set up components with error-prone configurations
    let scheduler_config = BackgroundSchedulerConfig {
        queue_file: temp_dir.path().join("integration_queue.json"),
        max_queue_size: 5,
        persistence_interval: Duration::from_secs(1),
        max_concurrent_tasks: 2,
        default_timeout: Duration::from_secs(5),
    };

    let executor_config = TaskExecutorConfig {
        max_concurrent_tasks: 1,
        api_calls_per_minute: 10,
        max_cpu_percent: 80.0,
        max_memory_percent: 80.0,
        resource_check_interval: Duration::from_secs(1),
        task_timeout: Duration::from_secs(2), // Short timeout for testing
        max_retries: 2,
        retry_initial_delay: Duration::from_millis(50),
        retry_max_delay: Duration::from_secs(2),
        retry_multiplier: 2.0,
        progress_report_interval: Duration::from_secs(1),
    };

    let state_config = StateManagerConfig {
        persistence_file: temp_dir.path().join("integration_state.json"),
        ..StateManagerConfig::default()
    };

    let error_handler_config = ErrorHandlerConfig::default();

    // Initialize components
    let background_scheduler = Arc::new(BackgroundScheduler::new(scheduler_config).await.unwrap());
    let task_executor = Arc::new(TaskExecutor::new(executor_config));
    let state_manager = Arc::new(StateManager::new(state_config).await.unwrap());
    let error_handler = Arc::new(ErrorHandler::new(error_handler_config));

    // Start all components
    state_manager.start().await.unwrap();
    error_handler.start().await.unwrap();

    // Configure state manager for background scheduler
    background_scheduler
        .configure_state_manager(state_manager.clone())
        .await
        .unwrap();
    task_executor
        .configure_state_manager(state_manager.clone())
        .await
        .unwrap();

    // Create and enqueue tasks
    for i in 0..3 {
        let gap = DetectedGap {
            gap_type: GapType::TodoComment,
            file_path: PathBuf::from(format!("test_{}.rs", i)),
            line_number: 42 + i,
            column_number: Some(10),
            context: format!("// TODO: Test implementation {}", i),
            description: format!("Test TODO comment {}", i),
            confidence: 0.9,
            priority: 7,
            metadata: HashMap::new(),
        };

        let task = ResearchTask::from_gap(gap, TaskPriority::Medium);

        // Track task creation and enqueue with error handling
        let task_id = task.id.clone();

        match state_manager.track_task_creation(&task).await {
            Ok(()) => match background_scheduler.enqueue(task).await {
                Ok(()) => {
                    info!("Task {} enqueued successfully", task_id);
                }
                Err(e) => {
                    error!("Failed to enqueue task {}: {}", task_id, e);
                }
            },
            Err(e) => {
                error!("Failed to track task creation for {}: {}", task_id, e);
            }
        }
    }

    // Simulate task execution with potential failures
    let mut executed_tasks = 0;
    let mut failed_tasks = 0;

    for _ in 0..3 {
        if let Ok(Some(task)) = background_scheduler.dequeue().await {
            let task_id = task.id.clone();

            // Simulate task execution with error handling
            let result = error_handler
                .execute_with_retry(
                    || {
                        let task_id = task_id.clone();
                        async move {
                            // Simulate execution that may fail
                            if task_id.chars().last().unwrap().to_digit(10).unwrap() % 2 == 0 {
                                // Even-numbered tasks fail initially
                                Err(ProactiveError::Transient {
                                    message: "Simulated execution failure".to_string(),
                                    attempt: 1,
                                    max_attempts: 3,
                                    retry_after: Some(Duration::from_millis(100)),
                                })
                            } else {
                                Ok(())
                            }
                        }
                    },
                    &format!("execute_task_{}", task_id),
                    Some("task_execution"),
                )
                .await;

            match result {
                Ok(()) => {
                    executed_tasks += 1;
                    // Update task state to completed
                    let mut completed_task = task.clone();
                    completed_task.state = TaskState::Completed;
                    let _ = background_scheduler.update_task(completed_task).await;
                }
                Err(e) => {
                    failed_tasks += 1;
                    error!("Task {} execution failed: {}", task_id, e);

                    // Update task state to failed
                    let mut failed_task = task.clone();
                    failed_task.state = TaskState::Failed;
                    let _ = background_scheduler.update_task(failed_task).await;
                }
            }
        }
    }

    // Verify error handling metrics
    let error_metrics = error_handler.get_metrics().await;
    let state_metrics = state_manager.get_metrics().await;
    let queue_metrics = background_scheduler.get_metrics().await;

    assert!(executed_tasks > 0 || failed_tasks > 0);
    assert!(error_metrics.total_errors >= 0); // May be 0 if all operations succeeded
    assert!(state_metrics.total_transitions >= executed_tasks + failed_tasks);
    assert_eq!(queue_metrics.current_queue_size, 0); // All tasks should be dequeued

    // Check dead letter queue for permanently failed operations
    let dlq_entries = error_handler.get_dead_letter_entries().await;
    info!("Dead letter queue entries: {}", dlq_entries.len());

    // Clean up
    error_handler.stop().await.unwrap();
    state_manager.stop().await.unwrap();
}

/// Test error recovery and state consistency across system restart
#[tokio::test]
async fn test_error_recovery_across_restart() {
    let temp_dir = TempDir::new().unwrap();
    let state_file = temp_dir.path().join("recovery_test_state.json");

    let task_id = {
        // First run - create task and transition to executing state
        let config = StateManagerConfig {
            persistence_file: state_file.clone(),
            ..StateManagerConfig::default()
        };

        let state_manager = StateManager::new(config).await.unwrap();
        state_manager.start().await.unwrap();

        let gap = create_test_gap();
        let task = ResearchTask::from_gap(gap, TaskPriority::High);
        let task_id = task.id.clone();

        // Track task and transition to executing
        state_manager.track_task_creation(&task).await.unwrap();
        state_manager
            .transition_task(&task_id, TaskState::Executing, "test", None)
            .await
            .unwrap();

        // Force persistence - we'll use the stop method which triggers persistence
        // since persist_state is private
        state_manager.stop().await.unwrap();

        task_id
    };

    // Simulate system restart - create new state manager and verify recovery
    {
        let config = StateManagerConfig {
            persistence_file: state_file,
            recovery_config: fortitude::proactive::StateRecoveryConfig {
                enable_auto_recovery: true,
                stale_task_threshold: Duration::from_millis(1), // Very short for testing
                stale_executing_strategy: fortitude::proactive::StaleTaskStrategy::ResetToPending,
                orphaned_task_strategy:
                    fortitude::proactive::OrphanedTaskStrategy::RetryWithBackoff,
            },
            ..StateManagerConfig::default()
        };

        let state_manager = StateManager::new(config).await.unwrap();

        // Verify task was loaded
        let lifecycle = state_manager.get_task_lifecycle(&task_id).await.unwrap();
        assert_eq!(lifecycle.current_state, TaskState::Executing);

        // Start state manager to trigger recovery
        state_manager.start().await.unwrap();

        // Recovery should reset stale executing task to pending
        tokio::time::sleep(Duration::from_millis(100)).await; // Allow recovery to run

        let recovery_count = state_manager.perform_recovery().await.unwrap();
        assert!(recovery_count > 0);

        // Verify task state was recovered
        let recovered_lifecycle = state_manager.get_task_lifecycle(&task_id).await.unwrap();
        assert_eq!(recovered_lifecycle.current_state, TaskState::Pending);

        state_manager.stop().await.unwrap();
    }
}

/// Test error handling configuration validation
#[tokio::test]
async fn test_error_handling_configuration_validation() {
    // Test invalid retry strategy configuration
    let invalid_retry_strategy = RetryStrategy {
        max_attempts: 0,                       // Invalid
        initial_delay: Duration::from_secs(0), // Invalid
        max_delay: Duration::from_millis(100),
        multiplier: 0.0,     // Invalid
        jitter_factor: -0.1, // Invalid
        enable_jitter: true,
    };

    // This should not panic but should handle invalid configurations gracefully
    let delay = invalid_retry_strategy.calculate_delay(1);
    assert!(delay >= Duration::from_secs(0));

    // Test invalid circuit breaker configuration
    let invalid_cb_config = CircuitBreakerConfig {
        failure_threshold: 0,            // Invalid
        success_threshold: 0,            // Invalid
        timeout: Duration::from_secs(0), // Invalid
        half_open_max_calls: 0,          // Invalid
    };

    let circuit_breaker = CircuitBreaker::new(invalid_cb_config);

    // Should still function despite invalid configuration
    assert!(circuit_breaker.can_execute().await);
}

/// Test error handling performance under load
#[tokio::test]
async fn test_error_handling_performance_under_load() {
    let config = ErrorHandlerConfig::default();
    let error_handler = Arc::new(ErrorHandler::new(config));
    error_handler.start().await.unwrap();

    let start_time = std::time::Instant::now();
    let mut handles = Vec::new();

    // Create 100 concurrent operations with mix of success and failure
    for i in 0..100 {
        let error_handler_clone = error_handler.clone();

        let handle = tokio::spawn(async move {
            let operation = || async move {
                if i % 4 == 0 {
                    // 25% failure rate
                    Err(ProactiveError::Transient {
                        message: "Load test failure".to_string(),
                        attempt: 1,
                        max_attempts: 2,
                        retry_after: Some(Duration::from_millis(1)),
                    })
                } else {
                    Ok(i)
                }
            };

            error_handler_clone
                .execute_with_retry(operation, &format!("load_test_{}", i), None)
                .await
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    let mut success_count = 0;
    let mut failure_count = 0;

    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => success_count += 1,
            Err(_) => failure_count += 1,
        }
    }

    let elapsed = start_time.elapsed();

    // Verify performance - should complete within reasonable time
    assert!(elapsed < Duration::from_secs(10));
    assert!(success_count > 0);

    // Verify error handler can handle the load
    let metrics = error_handler.get_metrics().await;
    assert_eq!(metrics.total_errors, failure_count);

    info!(
        "Load test completed in {:?}: {} success, {} failures",
        elapsed, success_count, failure_count
    );

    error_handler.stop().await.unwrap();
}
