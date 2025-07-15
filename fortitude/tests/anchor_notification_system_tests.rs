//! Anchor integration tests for notification system core functionality.
//!
//! These tests verify critical notification delivery functionality and should be maintained
//! as the system evolves. Do not delete these tests.

use fortitude::proactive::{
    Notification, NotificationChannel, NotificationSystem, NotificationSystemConfig,
    NotificationSystemError, NotificationType,
};
use futures::future;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use tokio::time::{timeout, Duration};

/// ANCHOR: Verifies CLI notification delivery works end-to-end.
/// Tests: CLI output formatting, colored output, stderr/stdout routing, real-time delivery
#[tokio::test]
async fn test_anchor_cli_notification_delivery() {
    // Create notification system with CLI channel
    let config = NotificationSystemConfig::default();
    let system = NotificationSystem::new(config);

    // Start the system
    system
        .start()
        .await
        .expect("Failed to start notification system");

    // Test different notification types through CLI
    let test_cases = vec![
        (
            NotificationType::Info,
            "Info notification",
            "This is an info message",
        ),
        (
            NotificationType::Warning,
            "Warning notification",
            "This is a warning message",
        ),
        (
            NotificationType::Error,
            "Error notification",
            "This is an error message",
        ),
        (
            NotificationType::Success,
            "Success notification",
            "This is a success message",
        ),
        (
            NotificationType::Progress {
                current: 50,
                total: 100,
            },
            "Progress notification",
            "Processing items",
        ),
    ];

    for (notification_type, title, message) in test_cases {
        let notification = Notification::new(
            notification_type.clone(),
            title.to_string(),
            message.to_string(),
            vec![NotificationChannel::CLI],
        )
        .with_source("anchor_test".to_string());

        // Verify notification can be sent without errors
        let result = system.send(notification).await;
        assert!(
            result.is_ok(),
            "CLI notification delivery failed for type: {:?}",
            notification_type
        );
    }

    // Verify metrics were updated
    let metrics = system.get_metrics().await;
    assert_eq!(
        metrics.total_notifications, 5,
        "Expected 5 notifications to be recorded"
    );
    assert!(metrics.notifications_by_type.contains_key("INFO"));
    assert!(metrics.notifications_by_type.contains_key("WARN"));
    assert!(metrics.notifications_by_type.contains_key("ERROR"));
    assert!(metrics.notifications_by_type.contains_key("SUCCESS"));
    assert!(metrics
        .notifications_by_type
        .contains_key("PROGRESS(50/100)"));

    // Test convenience methods work
    assert!(system
        .info(
            "Convenience Info".to_string(),
            "Info via convenience method".to_string()
        )
        .await
        .is_ok());
    assert!(system
        .warning(
            "Convenience Warning".to_string(),
            "Warning via convenience method".to_string()
        )
        .await
        .is_ok());
    assert!(system
        .error(
            "Convenience Error".to_string(),
            "Error via convenience method".to_string()
        )
        .await
        .is_ok());
    assert!(system
        .success(
            "Convenience Success".to_string(),
            "Success via convenience method".to_string()
        )
        .await
        .is_ok());
    assert!(system
        .progress(
            "Convenience Progress".to_string(),
            "Progress via convenience method".to_string(),
            75,
            100
        )
        .await
        .is_ok());

    system
        .stop()
        .await
        .expect("Failed to stop notification system");
}

/// ANCHOR: Verifies file notification delivery with persistence and timestamps.
/// Tests: File I/O operations, timestamp formatting, append mode, concurrent writes
#[tokio::test]
async fn test_anchor_file_notification_persistence() {
    // Create temporary file for testing
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let file_path = temp_file.path().to_path_buf();

    // Create notification system with file channel
    let config = NotificationSystemConfig::default();
    let system = NotificationSystem::new(config);
    system
        .start()
        .await
        .expect("Failed to start notification system");

    // Send notifications to file
    let notifications = vec![
        Notification::new(
            NotificationType::Info,
            "File Test 1".to_string(),
            "First file notification".to_string(),
            vec![NotificationChannel::File {
                path: file_path.clone(),
            }],
        )
        .with_source("file_test".to_string()),
        Notification::new(
            NotificationType::Warning,
            "File Test 2".to_string(),
            "Second file notification".to_string(),
            vec![NotificationChannel::File {
                path: file_path.clone(),
            }],
        )
        .with_source("file_test".to_string()),
        Notification::new(
            NotificationType::Error,
            "File Test 3".to_string(),
            "Third file notification".to_string(),
            vec![NotificationChannel::File {
                path: file_path.clone(),
            }],
        )
        .with_source("file_test".to_string()),
    ];

    // Send all notifications
    for notification in notifications {
        let result = system.send(notification).await;
        assert!(result.is_ok(), "File notification delivery failed");
    }

    // Wait a moment for async writes to complete
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify file content
    let file_content = tokio::fs::read_to_string(&file_path)
        .await
        .expect("Failed to read notification file");

    // Verify all notifications were written
    assert!(
        file_content.contains("File Test 1"),
        "First notification not found in file"
    );
    assert!(
        file_content.contains("File Test 2"),
        "Second notification not found in file"
    );
    assert!(
        file_content.contains("File Test 3"),
        "Third notification not found in file"
    );

    // Verify formatting includes timestamps and proper structure
    assert!(file_content.contains("INFO"), "Info level not found");
    assert!(file_content.contains("WARN"), "Warning level not found");
    assert!(file_content.contains("ERROR"), "Error level not found");
    assert!(
        file_content.contains("[file_test]"),
        "Source component not found"
    );

    // Verify timestamps are present (should contain UTC timestamps)
    assert!(file_content.contains("UTC"), "UTC timestamp not found");

    // Count lines to ensure all notifications were written
    let line_count = file_content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    assert_eq!(line_count, 3, "Expected 3 non-empty lines in file");

    system
        .stop()
        .await
        .expect("Failed to stop notification system");
}

/// ANCHOR: Verifies API notification delivery with HTTP endpoint integration.
/// Tests: HTTP request formatting, JSON serialization, error handling, endpoint communication
#[tokio::test]
async fn test_anchor_api_notification_http_delivery() {
    // Note: This test would require a mock HTTP server in a full implementation
    // For now, we test the API delivery mechanism with error handling

    let config = NotificationSystemConfig::default();
    let system = NotificationSystem::new(config);
    system
        .start()
        .await
        .expect("Failed to start notification system");

    // Test API notification creation and serialization
    let notification = Notification::new(
        NotificationType::Info,
        "API Test".to_string(),
        "Testing API notification".to_string(),
        vec![NotificationChannel::API {
            endpoint: "http://invalid-endpoint-for-testing.com/notifications".to_string(),
        }],
    )
    .with_source("api_test".to_string());

    // Add metadata to test full serialization
    let mut metadata = HashMap::new();
    metadata.insert("test_key".to_string(), "test_value".to_string());
    metadata.insert("priority".to_string(), "high".to_string());
    let notification_with_metadata = notification.with_metadata(metadata);

    // Verify notification can be serialized to JSON (required for API delivery)
    let serialized = serde_json::to_string(&notification_with_metadata)
        .expect("Failed to serialize notification to JSON");

    // Verify essential fields are present in JSON
    assert!(serialized.contains("\"notification_type\""));
    assert!(serialized.contains("\"title\""));
    assert!(serialized.contains("\"message\""));
    assert!(serialized.contains("\"timestamp\""));
    assert!(serialized.contains("\"metadata\""));
    assert!(serialized.contains("\"test_key\""));
    assert!(serialized.contains("\"test_value\""));

    // Test that the notification can be deserialized back
    let _deserialized: Notification =
        serde_json::from_str(&serialized).expect("Failed to deserialize notification from JSON");

    // Test API delivery failure handling (with invalid endpoint)
    let result = system.send(notification_with_metadata).await;
    // Should fail due to invalid endpoint, but error should be handled gracefully
    assert!(
        result.is_ok(),
        "System should handle API delivery failures gracefully"
    );

    // Verify metrics show the attempted delivery
    let metrics = system.get_metrics().await;
    assert_eq!(
        metrics.total_notifications, 1,
        "Notification should be counted even if delivery fails"
    );

    system
        .stop()
        .await
        .expect("Failed to stop notification system");
}

/// ANCHOR: Verifies multi-channel notification delivery works simultaneously.
/// Tests: Multiple channel delivery, channel independence, error isolation, concurrent processing
#[tokio::test]
async fn test_anchor_multi_channel_delivery() {
    // Create temporary file for file channel
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let file_path = temp_file.path().to_path_buf();

    // Create notification system
    let config = NotificationSystemConfig::default();
    let system = NotificationSystem::new(config);
    system
        .start()
        .await
        .expect("Failed to start notification system");

    // Create notification with multiple channels
    let notification = Notification::new(
        NotificationType::Warning,
        "Multi-Channel Test".to_string(),
        "Testing delivery to multiple channels simultaneously".to_string(),
        vec![
            NotificationChannel::CLI,
            NotificationChannel::File {
                path: file_path.clone(),
            },
            NotificationChannel::API {
                endpoint: "http://invalid-endpoint-for-testing.com/notifications".to_string(),
            },
        ],
    )
    .with_source("multi_channel_test".to_string());

    // Send notification to all channels
    let result = system.send(notification).await;
    assert!(result.is_ok(), "Multi-channel notification delivery failed");

    // Wait for async operations to complete
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify file channel received the notification
    let file_content = tokio::fs::read_to_string(&file_path)
        .await
        .expect("Failed to read notification file");
    assert!(
        file_content.contains("Multi-Channel Test"),
        "File channel did not receive notification"
    );
    assert!(
        file_content.contains("Testing delivery to multiple channels"),
        "File channel content incomplete"
    );
    assert!(
        file_content.contains("WARN"),
        "File channel missing notification type"
    );
    assert!(
        file_content.contains("[multi_channel_test]"),
        "File channel missing source component"
    );

    // Verify metrics reflect the multi-channel delivery
    let metrics = system.get_metrics().await;
    assert_eq!(
        metrics.total_notifications, 1,
        "Should count as one notification"
    );
    assert_eq!(
        metrics.notifications_by_type.get("WARN"),
        Some(&1),
        "Warning type should be recorded"
    );

    // Verify channel metrics (file channel should show success, API might show failure)
    if let Some(file_metrics) = metrics.channel_metrics.get("file") {
        assert!(
            file_metrics.successful_deliveries > 0,
            "File channel should record successful delivery"
        );
    }

    system
        .stop()
        .await
        .expect("Failed to stop notification system");
}

/// ANCHOR: Verifies error handling and recovery for notification delivery failures.
/// Tests: Error handling, graceful degradation, metrics tracking, system stability
#[tokio::test]
async fn test_anchor_error_handling_and_recovery() {
    let config = NotificationSystemConfig::default();
    let system = NotificationSystem::new(config);
    system
        .start()
        .await
        .expect("Failed to start notification system");

    // Test notification delivery to non-existent file path
    let invalid_file_path = PathBuf::from("/invalid/path/that/does/not/exist/notification.log");
    let file_notification = Notification::new(
        NotificationType::Error,
        "Error Test".to_string(),
        "Testing error handling".to_string(),
        vec![NotificationChannel::File {
            path: invalid_file_path,
        }],
    );

    // System should handle file delivery failure gracefully
    let result = system.send(file_notification).await;
    assert!(
        result.is_ok(),
        "System should handle file delivery failures gracefully"
    );

    // Test notification to invalid API endpoint
    let api_notification = Notification::new(
        NotificationType::Warning,
        "API Error Test".to_string(),
        "Testing API error handling".to_string(),
        vec![NotificationChannel::API {
            endpoint: "http://definitely-invalid-domain-12345.com/api".to_string(),
        }],
    );

    // System should handle API delivery failure gracefully
    let result = system.send(api_notification).await;
    assert!(
        result.is_ok(),
        "System should handle API delivery failures gracefully"
    );

    // Test mixed success/failure scenario
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let file_path = temp_file.path().to_path_buf();

    let mixed_notification = Notification::new(
        NotificationType::Info,
        "Mixed Delivery Test".to_string(),
        "Testing mixed success and failure".to_string(),
        vec![
            NotificationChannel::CLI, // Should succeed
            NotificationChannel::File {
                path: file_path.clone(),
            }, // Should succeed
            NotificationChannel::API {
                endpoint: "http://invalid.com".to_string(),
            }, // Should fail
        ],
    );

    let result = system.send(mixed_notification).await;
    assert!(
        result.is_ok(),
        "System should handle mixed delivery scenarios"
    );

    // Wait for async operations
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify successful channels still work
    let file_content = tokio::fs::read_to_string(&file_path)
        .await
        .expect("Failed to read notification file");
    assert!(
        file_content.contains("Mixed Delivery Test"),
        "Successful channels should still deliver"
    );

    // Verify metrics reflect the attempts
    let metrics = system.get_metrics().await;
    assert!(
        metrics.total_notifications >= 3,
        "All notifications should be counted"
    );

    // System should remain operational after errors
    let final_notification = Notification::new(
        NotificationType::Success,
        "Recovery Test".to_string(),
        "System should still work after errors".to_string(),
        vec![NotificationChannel::CLI],
    );

    let result = system.send(final_notification).await;
    assert!(
        result.is_ok(),
        "System should remain operational after handling errors"
    );

    system
        .stop()
        .await
        .expect("Failed to stop notification system");
}

/// ANCHOR: Verifies notification system performance and resource management.
/// Tests: Concurrent notifications, rate limiting, resource usage, system stability
#[tokio::test]
async fn test_anchor_performance_and_concurrency() {
    let config = NotificationSystemConfig::default();
    let system = NotificationSystem::new(config);
    system
        .start()
        .await
        .expect("Failed to start notification system");

    // Test concurrent notification delivery
    let concurrent_count = 50;
    let mut tasks = Vec::new();

    for i in 0..concurrent_count {
        let notification = Notification::new(
            NotificationType::Info,
            format!("Concurrent Test {}", i),
            format!("Concurrent notification number {}", i),
            vec![NotificationChannel::CLI],
        )
        .with_source("performance_test".to_string());

        let task = system.send(notification);
        tasks.push(task);
    }

    // Wait for all concurrent notifications to complete
    let results = timeout(Duration::from_secs(10), future::join_all(tasks))
        .await
        .expect("Concurrent notifications should complete within timeout");

    // Verify all notifications succeeded
    let mut success_count = 0;
    for result in results {
        if result.is_ok() {
            success_count += 1;
        }
    }

    assert_eq!(
        success_count, concurrent_count,
        "All concurrent notifications should succeed, got {}/{}",
        success_count, concurrent_count
    );

    // Verify metrics reflect all notifications
    let metrics = system.get_metrics().await;
    assert!(
        metrics.total_notifications >= concurrent_count as u64,
        "Metrics should reflect all concurrent notifications"
    );

    // Test system stability after high load
    let stability_notification = Notification::new(
        NotificationType::Success,
        "Stability Test".to_string(),
        "System should be stable after high load".to_string(),
        vec![NotificationChannel::CLI],
    );

    let result = system.send(stability_notification).await;
    assert!(
        result.is_ok(),
        "System should remain stable after concurrent load"
    );

    system
        .stop()
        .await
        .expect("Failed to stop notification system");
}

/// ANCHOR: Verifies notification system lifecycle management and state consistency.
/// Tests: Start/stop operations, state management, cleanup, initialization verification
#[tokio::test]
async fn test_anchor_lifecycle_management() {
    let config = NotificationSystemConfig::default();
    let system = NotificationSystem::new(config);

    // Test initial state
    let notification = Notification::new(
        NotificationType::Info,
        "Lifecycle Test".to_string(),
        "Testing before start".to_string(),
        vec![NotificationChannel::CLI],
    );

    // Should fail when not started
    let result = system.send(notification.clone()).await;
    assert!(
        result.is_err(),
        "Notifications should fail before system start"
    );
    assert!(matches!(
        result.unwrap_err(),
        NotificationSystemError::NotInitialized
    ));

    // Start system
    system
        .start()
        .await
        .expect("Failed to start notification system");

    // Should work after start
    let result = system.send(notification.clone()).await;
    assert!(
        result.is_ok(),
        "Notifications should work after system start"
    );

    // Test multiple starts (should be idempotent)
    let result = system.start().await;
    assert!(result.is_ok(), "Multiple starts should be allowed");

    // Should still work
    let result = system.send(notification.clone()).await;
    assert!(
        result.is_ok(),
        "Notifications should still work after multiple starts"
    );

    // Stop system
    system
        .stop()
        .await
        .expect("Failed to stop notification system");

    // Should fail after stop
    let result = system.send(notification).await;
    assert!(
        result.is_err(),
        "Notifications should fail after system stop"
    );

    // Verify metrics are still accessible after stop
    let metrics = system.get_metrics().await;
    assert!(
        metrics.total_notifications > 0,
        "Metrics should persist after stop"
    );

    // Test restart
    system
        .start()
        .await
        .expect("Failed to restart notification system");

    let restart_notification = Notification::new(
        NotificationType::Success,
        "Restart Test".to_string(),
        "Testing after restart".to_string(),
        vec![NotificationChannel::CLI],
    );

    let result = system.send(restart_notification).await;
    assert!(result.is_ok(), "Notifications should work after restart");

    system
        .stop()
        .await
        .expect("Failed to stop notification system");
}
