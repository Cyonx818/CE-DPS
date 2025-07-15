// ANCHOR: notification_preferences_tests
// ABOUTME: Comprehensive test suite for user notification preferences system
//! This test suite validates the notification preferences system implementation
//! including frequency controls, type filtering, channel routing, detail levels,
//! time-based scheduling, and integration with the notification system.

use chrono::NaiveTime;
use fortitude::proactive::{
    BusinessHours, ContextualNotificationSettings, NotificationChannel,
    NotificationChannelSettings, NotificationDetailLevel, NotificationFrequency,
    NotificationPreferenceFilter, NotificationSystem, NotificationSystemConfig, NotificationType,
    NotificationTypeSettings, PreferenceAwareNotificationSender, PriorityOverrideSettings,
    QuietHours, TimeRange, UserPreferenceManager, UserPreferenceProfile,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

#[tokio::test]
async fn test_notification_frequency_preferences() {
    // ANCHOR_TEST: notification_frequency_controls
    // Tests that notification frequency preferences (immediate, batched, scheduled, disabled) work correctly
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("preferences");

    let manager = UserPreferenceManager::new(storage_path).await.unwrap();

    // Test immediate frequency
    let mut immediate_profile = manager
        .create_profile(
            "immediate_test".to_string(),
            "Test immediate frequency".to_string(),
        )
        .await
        .unwrap();

    immediate_profile.notification_preferences.frequency = NotificationFrequency::Immediate;
    let result = manager.update_profile(immediate_profile).await;
    assert!(
        result.is_ok(),
        "Should update profile with immediate frequency"
    );

    // Test batched frequency
    let mut batched_profile = manager
        .create_profile(
            "batched_test".to_string(),
            "Test batched frequency".to_string(),
        )
        .await
        .unwrap();

    batched_profile.notification_preferences.frequency = NotificationFrequency::Batched {
        batch_size: 5,
        batch_timeout: Duration::from_minutes(10),
    };
    let result = manager.update_profile(batched_profile).await;
    assert!(
        result.is_ok(),
        "Should update profile with batched frequency"
    );

    // Test scheduled frequency
    let mut scheduled_profile = manager
        .create_profile(
            "scheduled_test".to_string(),
            "Test scheduled frequency".to_string(),
        )
        .await
        .unwrap();

    scheduled_profile.notification_preferences.frequency = NotificationFrequency::Scheduled {
        schedule: "0 9 * * MON-FRI".to_string(),
    };
    let result = manager.update_profile(scheduled_profile).await;
    assert!(
        result.is_ok(),
        "Should update profile with scheduled frequency"
    );

    // Test disabled frequency
    let mut disabled_profile = manager
        .create_profile(
            "disabled_test".to_string(),
            "Test disabled frequency".to_string(),
        )
        .await
        .unwrap();

    disabled_profile.notification_preferences.frequency = NotificationFrequency::Disabled;
    let result = manager.update_profile(disabled_profile).await;
    assert!(
        result.is_ok(),
        "Should update profile with disabled frequency"
    );

    // Verify profiles were saved correctly
    let loaded_batched = manager.load_profile("batched_test").await.unwrap();
    match loaded_batched.notification_preferences.frequency {
        NotificationFrequency::Batched { batch_size, .. } => {
            assert_eq!(batch_size, 5, "Batch size should be preserved");
        }
        _ => panic!("Expected batched frequency"),
    }
}

#[tokio::test]
async fn test_notification_type_filtering() {
    // ANCHOR_TEST: notification_type_filtering
    // Tests that notification type filtering works correctly with per-type customization
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("preferences");

    let manager = UserPreferenceManager::new(storage_path).await.unwrap();

    let mut profile = manager
        .create_profile(
            "type_filtering_test".to_string(),
            "Test notification type filtering".to_string(),
        )
        .await
        .unwrap();

    // Configure type-specific preferences using string keys
    profile.notification_preferences.type_settings.insert(
        "progress".to_string(),
        NotificationTypeSettings {
            enabled: false,
            channels: vec![],
            detail_level: NotificationDetailLevel::Brief,
        },
    );

    profile.notification_preferences.type_settings.insert(
        "success".to_string(),
        NotificationTypeSettings {
            enabled: true,
            channels: vec![NotificationChannel::CLI],
            detail_level: NotificationDetailLevel::Detailed,
        },
    );

    let result = manager.update_profile(profile).await;
    assert!(
        result.is_ok(),
        "Should update profile with type-specific settings"
    );

    // Load and verify settings
    let loaded_profile = manager.load_profile("type_filtering_test").await.unwrap();

    // Progress notifications should be disabled
    let progress_settings = loaded_profile
        .notification_preferences
        .type_settings
        .get("progress")
        .unwrap();
    assert!(
        !progress_settings.enabled,
        "Progress notifications should be disabled"
    );

    // Success notifications should be enabled with detailed level
    let success_settings = loaded_profile
        .notification_preferences
        .type_settings
        .get("success")
        .unwrap();
    assert!(
        success_settings.enabled,
        "Success notifications should be enabled"
    );
    assert_eq!(
        success_settings.detail_level,
        NotificationDetailLevel::Detailed
    );
    assert_eq!(
        success_settings.channels.len(),
        1,
        "Should have 1 channel configured"
    );
}

#[tokio::test]
async fn test_notification_channel_preferences() {
    // ANCHOR_TEST: notification_channel_preferences
    // Tests channel preferences with per-type routing capabilities
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("preferences");

    let manager = UserPreferenceManager::new(storage_path).await.unwrap();

    let mut profile = manager
        .create_profile(
            "channel_routing_test".to_string(),
            "Test channel routing preferences".to_string(),
        )
        .await
        .unwrap();

    // Configure default channels
    profile.notification_preferences.default_channels = vec![
        NotificationChannel::CLI,
        NotificationChannel::File {
            path: PathBuf::from("logs/notifications.log"),
        },
    ];

    // Configure channel-specific settings
    profile.notification_preferences.channel_settings.insert(
        "cli".to_string(),
        NotificationChannelSettings {
            enabled: true,
            rate_limit: Some(60),
            min_detail_level: NotificationDetailLevel::Brief,
            max_detail_level: NotificationDetailLevel::Standard,
            quiet_hours: Some(QuietHours {
                start: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                timezone: "UTC".to_string(),
            }),
        },
    );

    let result = manager.update_profile(profile).await;
    assert!(
        result.is_ok(),
        "Should update profile with channel preferences"
    );

    // Verify channel settings
    let loaded_profile = manager.load_profile("channel_routing_test").await.unwrap();

    let cli_settings = loaded_profile
        .notification_preferences
        .channel_settings
        .get("cli")
        .unwrap();
    assert!(cli_settings.enabled);
    assert_eq!(cli_settings.rate_limit, Some(60));
    assert!(cli_settings.quiet_hours.is_some());
}

#[tokio::test]
async fn test_notification_detail_levels() {
    // ANCHOR_TEST: notification_detail_levels
    // Tests different notification detail levels (brief, standard, detailed, comprehensive)
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("preferences");

    let manager = UserPreferenceManager::new(storage_path).await.unwrap();

    // Test each detail level
    let detail_levels = vec![
        NotificationDetailLevel::Brief,
        NotificationDetailLevel::Standard,
        NotificationDetailLevel::Detailed,
        NotificationDetailLevel::Comprehensive,
    ];

    for (i, level) in detail_levels.iter().enumerate() {
        let mut profile = manager
            .create_profile(
                format!("detail_level_{}", i),
                format!("Profile with {:?} detail level", level),
            )
            .await
            .unwrap();

        profile.notification_preferences.default_detail_level = level.clone();

        let result = manager.update_profile(profile).await;
        assert!(
            result.is_ok(),
            "Should create profile with {:?} detail level",
            level
        );

        // Verify detail level is preserved
        let loaded_profile = manager
            .load_profile(&format!("detail_level_{}", i))
            .await
            .unwrap();
        assert_eq!(
            loaded_profile.notification_preferences.default_detail_level,
            *level
        );
    }
}

#[tokio::test]
async fn test_time_based_notification_scheduling() {
    // ANCHOR_TEST: time_based_scheduling
    // Tests time-based notification scheduling including quiet hours and business hours
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("preferences");

    let manager = UserPreferenceManager::new(storage_path).await.unwrap();

    let mut profile = manager
        .create_profile(
            "time_based_test".to_string(),
            "Test time-based notifications".to_string(),
        )
        .await
        .unwrap();

    // Configure business hours
    profile.notification_preferences.business_hours = Some(BusinessHours {
        monday: Some(TimeRange {
            start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        }),
        tuesday: Some(TimeRange {
            start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        }),
        wednesday: Some(TimeRange {
            start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        }),
        thursday: Some(TimeRange {
            start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        }),
        friday: Some(TimeRange {
            start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        }),
        saturday: None,
        sunday: None,
        timezone: "UTC".to_string(),
    });

    // Configure global quiet hours
    profile.notification_preferences.global_quiet_hours = Some(QuietHours {
        start: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
        end: NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
        timezone: "UTC".to_string(),
    });

    profile.notification_preferences.respect_business_hours = true;
    profile.notification_preferences.respect_quiet_hours = true;

    let result = manager.update_profile(profile).await;
    assert!(
        result.is_ok(),
        "Should update profile with time-based preferences"
    );

    // Verify time-based settings
    let loaded_profile = manager.load_profile("time_based_test").await.unwrap();

    assert!(loaded_profile
        .notification_preferences
        .business_hours
        .is_some());
    assert!(loaded_profile
        .notification_preferences
        .global_quiet_hours
        .is_some());
    assert!(
        loaded_profile
            .notification_preferences
            .respect_business_hours
    );
    assert!(loaded_profile.notification_preferences.respect_quiet_hours);

    let business_hours = loaded_profile
        .notification_preferences
        .business_hours
        .unwrap();
    assert!(business_hours.monday.is_some());
    assert!(business_hours.saturday.is_none());
}

#[tokio::test]
async fn test_notification_preference_validation() {
    // ANCHOR_TEST: preference_validation
    // Tests validation of notification preferences for correctness and consistency
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("preferences");

    let manager = UserPreferenceManager::new(storage_path).await.unwrap();

    // Test invalid batch settings - this should fail during validation
    let mut invalid_batch_profile = manager
        .create_profile(
            "invalid_batch_test".to_string(),
            "Test invalid batch settings".to_string(),
        )
        .await
        .unwrap();

    invalid_batch_profile.notification_preferences.frequency = NotificationFrequency::Batched {
        batch_size: 0, // Invalid - should be > 0
        batch_timeout: Duration::from_secs(10),
    };

    // This should fail validation
    let result = invalid_batch_profile.validate();
    assert!(
        result.is_err(),
        "Should reject profile with invalid batch size"
    );

    // Test valid settings should pass
    let mut valid_profile = manager
        .create_profile("valid_test".to_string(), "Test valid settings".to_string())
        .await
        .unwrap();

    valid_profile.notification_preferences.frequency = NotificationFrequency::Batched {
        batch_size: 5,
        batch_timeout: Duration::from_minutes(2),
    };

    let result = valid_profile.validate();
    assert!(result.is_ok(), "Should accept profile with valid settings");
}

#[tokio::test]
async fn test_notification_preference_persistence() {
    // ANCHOR_TEST: preference_persistence
    // Tests that notification preferences persist correctly across restarts
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("preferences");

    // Create preferences in first manager instance
    {
        let manager = UserPreferenceManager::new(storage_path.clone())
            .await
            .unwrap();

        let mut profile = manager
            .create_profile(
                "persistence_test".to_string(),
                "Test persistence".to_string(),
            )
            .await
            .unwrap();

        profile.notification_preferences.frequency = NotificationFrequency::Batched {
            batch_size: 10,
            batch_timeout: Duration::from_minutes(5),
        };

        profile.notification_preferences.type_settings.insert(
            "info".to_string(),
            NotificationTypeSettings {
                enabled: true,
                channels: vec![NotificationChannel::CLI],
                detail_level: NotificationDetailLevel::Detailed,
            },
        );

        manager.update_profile(profile).await.unwrap();
    }

    // Load preferences in second manager instance
    {
        let manager = UserPreferenceManager::new(storage_path).await.unwrap();

        let loaded_profile = manager.load_profile("persistence_test").await.unwrap();

        // Verify frequency settings persisted
        match loaded_profile.notification_preferences.frequency {
            NotificationFrequency::Batched {
                batch_size,
                batch_timeout,
            } => {
                assert_eq!(batch_size, 10);
                assert_eq!(batch_timeout, Duration::from_minutes(5));
            }
            _ => panic!("Expected batched frequency"),
        }

        // Verify type settings persisted
        let info_settings = loaded_profile
            .notification_preferences
            .type_settings
            .get("info")
            .unwrap();
        assert!(info_settings.enabled);
        assert_eq!(
            info_settings.detail_level,
            NotificationDetailLevel::Detailed
        );
    }
}

#[tokio::test]
async fn test_notification_system_integration() {
    // ANCHOR_TEST: notification_system_integration
    // Tests integration between notification preferences and the notification system
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("preferences");

    let manager = UserPreferenceManager::new(storage_path).await.unwrap();

    // Create profile with specific notification preferences
    let mut profile = manager
        .create_profile(
            "integration_test".to_string(),
            "Test notification system integration".to_string(),
        )
        .await
        .unwrap();

    profile.notification_preferences.frequency = NotificationFrequency::Immediate;
    profile.notification_preferences.type_settings.insert(
        "success".to_string(),
        NotificationTypeSettings {
            enabled: true,
            channels: vec![NotificationChannel::CLI],
            detail_level: NotificationDetailLevel::Standard,
        },
    );

    manager.update_profile(profile).await.unwrap();
    manager
        .set_active_profile("integration_test")
        .await
        .unwrap();

    // Create notification system with preference integration
    let notification_config = NotificationSystemConfig::default();
    let notification_system = NotificationSystem::new(notification_config);
    notification_system.start().await.unwrap();

    // Create preference-aware notification sender
    let preference_aware_sender =
        PreferenceAwareNotificationSender::new(Arc::new(notification_system), Arc::new(manager))
            .await
            .unwrap();

    // Send notification through preference-aware sender
    let result = preference_aware_sender
        .send_notification(
            NotificationType::Success,
            "Test Success".to_string(),
            "This is a test success notification".to_string(),
        )
        .await;

    assert!(
        result.is_ok(),
        "Should send notification through preference-aware sender"
    );
}

#[tokio::test]
async fn test_notification_preference_filter() {
    // ANCHOR_TEST: notification_preference_filter
    // Tests the notification preference filter functionality
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("preferences");

    let manager = UserPreferenceManager::new(storage_path).await.unwrap();

    let mut profile = manager
        .create_profile(
            "filter_test".to_string(),
            "Test notification filtering".to_string(),
        )
        .await
        .unwrap();

    // Disable debug notifications
    profile.notification_preferences.type_settings.insert(
        "debug".to_string(),
        NotificationTypeSettings {
            enabled: false,
            channels: vec![],
            detail_level: NotificationDetailLevel::Brief,
        },
    );

    // Enable info notifications
    profile.notification_preferences.type_settings.insert(
        "info".to_string(),
        NotificationTypeSettings {
            enabled: true,
            channels: vec![NotificationChannel::CLI],
            detail_level: NotificationDetailLevel::Standard,
        },
    );

    manager.update_profile(profile.clone()).await.unwrap();

    // Create filter
    let filter = NotificationPreferenceFilter::new(&profile.notification_preferences);

    // Test filtering
    assert!(
        !filter.should_send_notification(&NotificationType::Debug),
        "Debug notifications should be filtered out"
    );
    assert!(
        filter.should_send_notification(&NotificationType::Info),
        "Info notifications should be allowed"
    );

    // Test channels
    let info_channels = filter.get_effective_channels(&NotificationType::Info);
    assert_eq!(info_channels.len(), 1, "Info should have 1 channel");

    // Test detail levels
    let info_detail = filter.get_effective_detail_level(&NotificationType::Info);
    assert_eq!(info_detail, NotificationDetailLevel::Standard);
}

#[tokio::test]
async fn test_effective_notification_settings() {
    // ANCHOR_TEST: effective_settings
    // Tests the effective notification settings resolution with overrides
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("preferences");

    let manager = UserPreferenceManager::new(storage_path).await.unwrap();

    let mut profile = manager
        .create_profile(
            "effective_settings_test".to_string(),
            "Test effective settings resolution".to_string(),
        )
        .await
        .unwrap();

    // Configure base settings
    profile.notification_preferences.enable_notifications = true;
    profile.notification_preferences.default_detail_level = NotificationDetailLevel::Standard;

    // Configure type-specific override
    profile.notification_preferences.type_settings.insert(
        "error".to_string(),
        NotificationTypeSettings {
            enabled: true,
            channels: vec![NotificationChannel::CLI],
            detail_level: NotificationDetailLevel::Comprehensive,
        },
    );

    // Configure priority override for errors
    profile.notification_preferences.priority_overrides.insert(
        "error".to_string(),
        PriorityOverrideSettings {
            always_send: true,
            override_quiet_hours: true,
            override_frequency: Some(NotificationFrequency::Immediate),
            override_channels: None,
        },
    );

    manager.update_profile(profile.clone()).await.unwrap();

    // Test effective settings resolution
    let effective_settings = profile
        .notification_preferences
        .get_effective_settings(&NotificationType::Error, None);

    assert!(
        effective_settings.enabled,
        "Error notifications should be enabled"
    );
    assert_eq!(
        effective_settings.detail_level,
        NotificationDetailLevel::Comprehensive
    );
    assert!(
        !effective_settings.respect_quiet_hours,
        "Should override quiet hours"
    );

    // Test with context
    let mut contextual_settings = ContextualNotificationSettings::default();
    contextual_settings.detail_level_override = Some(NotificationDetailLevel::Brief);

    profile
        .notification_preferences
        .contextual_settings
        .insert("development".to_string(), contextual_settings);

    let effective_with_context = profile
        .notification_preferences
        .get_effective_settings(&NotificationType::Info, Some("development"));

    assert_eq!(
        effective_with_context.detail_level,
        NotificationDetailLevel::Brief
    );
}

// Utility trait for Duration creation
trait DurationExt {
    fn from_minutes(minutes: u64) -> Duration;
}

impl DurationExt for Duration {
    fn from_minutes(minutes: u64) -> Duration {
        Duration::from_secs(minutes * 60)
    }
}
