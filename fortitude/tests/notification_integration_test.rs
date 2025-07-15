//! Integration test showing notification system working with other proactive components.

use fortitude::proactive::{
    Notification, NotificationChannel, NotificationSystem, NotificationSystemConfig,
    NotificationType,
};
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_notification_system_integration() {
    // Create notification system
    let config = NotificationSystemConfig::default();
    let notification_system = NotificationSystem::new(config);

    // Start the system
    notification_system
        .start()
        .await
        .expect("Failed to start notification system");

    // Simulate integration with gap analyzer component
    notification_system
        .info(
            "Gap Analysis Started".to_string(),
            "Beginning analysis of documentation gaps".to_string(),
        )
        .await
        .expect("Failed to send gap analysis start notification");

    // Simulate progress notifications
    for progress in [25, 50, 75, 100] {
        notification_system
            .progress(
                "Gap Analysis Progress".to_string(),
                format!("Analyzing files and dependencies"),
                progress,
                100,
            )
            .await
            .expect("Failed to send progress notification");
    }

    // Simulate completion notification
    notification_system
        .success(
            "Gap Analysis Complete".to_string(),
            "Found 5 documentation gaps, 3 high priority".to_string(),
        )
        .await
        .expect("Failed to send completion notification");

    // Simulate integration with task executor
    notification_system
        .info(
            "Task Executor Started".to_string(),
            "Beginning background research task execution".to_string(),
        )
        .await
        .expect("Failed to send task executor notification");

    // Simulate warning about resource usage
    notification_system
        .warning(
            "High CPU Usage".to_string(),
            "CPU usage at 85%, consider throttling task execution".to_string(),
        )
        .await
        .expect("Failed to send warning notification");

    // Simulate error in external API call
    notification_system
        .error(
            "API Request Failed".to_string(),
            "Failed to retrieve research data from external API: timeout after 30s".to_string(),
        )
        .await
        .expect("Failed to send error notification");

    // Verify metrics show all notifications (before audit notification)
    let metrics = notification_system.get_metrics().await;
    assert_eq!(
        metrics.total_notifications, 9,
        "Expected 9 total notifications"
    );
    assert!(metrics.notifications_by_type.contains_key("INFO"));
    assert!(metrics.notifications_by_type.contains_key("SUCCESS"));
    assert!(metrics.notifications_by_type.contains_key("WARN"));
    assert!(metrics.notifications_by_type.contains_key("ERROR"));

    // Test file-based notifications for audit trail
    let log_file = NamedTempFile::new().expect("Failed to create temp log file");
    let log_path = log_file.path().to_path_buf();

    let audit_notification = Notification::new(
        NotificationType::Info,
        "Audit Log Entry".to_string(),
        "System audit: notification system integration test completed".to_string(),
        vec![
            NotificationChannel::CLI,
            NotificationChannel::File {
                path: log_path.clone(),
            },
        ],
    );

    notification_system
        .send(audit_notification)
        .await
        .expect("Failed to send audit notification");

    // Wait for file write
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify audit log was written
    let log_content = tokio::fs::read_to_string(&log_path)
        .await
        .expect("Failed to read audit log");
    assert!(log_content.contains("Audit Log Entry"));
    assert!(log_content.contains("notification system integration test completed"));

    notification_system
        .stop()
        .await
        .expect("Failed to stop notification system");
}
