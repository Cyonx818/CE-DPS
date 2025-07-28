// Copyright 2025 CE-DPS Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// ABOUTME: Testing infrastructure and mock channels for comprehensive notification system testing
//! This module provides comprehensive testing infrastructure for the notification system
//! including mock channels, test harnesses, and stress testing capabilities. Features include:
//! - Mock notification channels with configurable behavior patterns
//! - Test harness for simulating various notification scenarios
//! - Stress testing for high-volume notification scenarios
//! - Integration testing utilities for cross-component validation
//! - Performance benchmarking and load testing tools

use crate::proactive::{
    NotificationChannel, NotificationType, Notification, NotificationSystem,
    NotificationSystemConfig, NotificationSystemError, DeliveryStatus
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{Mutex, RwLock, broadcast};
use tracing::{debug, error, info, warn, instrument};
use uuid::Uuid;

/// Errors that can occur in the testing harness
#[derive(Error, Debug, Clone)]
pub enum TestHarnessError {
    #[error("Mock channel error: {0}")]
    MockChannelError(String),

    #[error("Test scenario failed: {0}")]
    TestScenarioFailed(String),

    #[error("Performance test failed: expected {expected}, got {actual}")]
    PerformanceTestFailed { expected: String, actual: String },

    #[error("Stress test failed: {reason}")]
    StressTestFailed { reason: String },

    #[error("Integration test failed: {component} - {reason}")]
    IntegrationTestFailed { component: String, reason: String },

    #[error("Mock configuration error: {0}")]
    MockConfigurationError(String),
}

/// Mock behavior patterns for testing different failure scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MockBehavior {
    /// Always succeed
    AlwaysSucceed,
    /// Always fail with specified error
    AlwaysFail { error_message: String },
    /// Succeed after specified delay
    DelayedSuccess { delay_ms: u64 },
    /// Intermittent failures with specified rate (0.0-1.0)
    IntermittentFailure { failure_rate: f64 },
    /// Rate limited behavior with specified limit
    RateLimited { max_per_second: u32 },
    /// Custom pattern with success/failure sequence
    CustomPattern { pattern: Vec<bool> },
}

impl Default for MockBehavior {
    fn default() -> Self {
        MockBehavior::AlwaysSucceed
    }
}

/// Mock notification channel for testing
#[derive(Debug, Clone)]
pub struct MockNotificationChannel {
    channel_id: String,
    channel_type: String,
    behavior: MockBehavior,
    delivery_log: Arc<RwLock<Vec<MockDeliveryRecord>>>,
    current_pattern_index: Arc<RwLock<usize>>,
    rate_limiter: Arc<RwLock<Option<MockRateLimiter>>>,
}

/// Record of a mock delivery attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockDeliveryRecord {
    pub delivery_id: String,
    pub notification_id: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub duration: Duration,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Simple rate limiter for mock channels
#[derive(Debug)]
struct MockRateLimiter {
    max_per_second: u32,
    tokens: u32,
    last_refill: Instant,
}

impl MockRateLimiter {
    fn new(max_per_second: u32) -> Self {
        Self {
            max_per_second,
            tokens: max_per_second,
            last_refill: Instant::now(),
        }
    }

    fn can_proceed(&mut self) -> bool {
        self.refill_tokens();
        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    fn refill_tokens(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let tokens_to_add = (elapsed.as_secs_f64() * self.max_per_second as f64) as u32;

        if tokens_to_add > 0 {
            self.tokens = (self.tokens + tokens_to_add).min(self.max_per_second);
            self.last_refill = now;
        }
    }
}

impl MockNotificationChannel {
    /// Create a new mock notification channel
    pub fn new(channel_id: String, channel_type: String) -> Self {
        Self {
            channel_id,
            channel_type,
            behavior: MockBehavior::default(),
            delivery_log: Arc::new(RwLock::new(Vec::new())),
            current_pattern_index: Arc::new(RwLock::new(0)),
            rate_limiter: Arc::new(RwLock::new(None)),
        }
    }

    /// Configure the mock behavior
    pub async fn set_behavior(&mut self, behavior: MockBehavior) {
        self.behavior = behavior.clone();

        // Setup rate limiter if needed
        if let MockBehavior::RateLimited { max_per_second } = behavior {
            let mut rate_limiter = self.rate_limiter.write().await;
            *rate_limiter = Some(MockRateLimiter::new(max_per_second));
        }
    }

    /// Simulate delivery to this mock channel
    #[instrument(level = "debug", skip(self, notification))]
    pub async fn deliver(&self, notification: &Notification) -> Result<(), NotificationSystemError> {
        let start_time = Instant::now();
        let delivery_id = Uuid::new_v4().to_string();

        debug!("Mock delivery attempt to channel: {} for notification: {}",
               self.channel_id, notification.id);

        // Apply behavior pattern
        let result = self.apply_behavior_pattern().await;
        let duration = start_time.elapsed();

        // Record delivery attempt
        let record = MockDeliveryRecord {
            delivery_id,
            notification_id: notification.id.clone(),
            timestamp: Utc::now(),
            success: result.is_ok(),
            duration,
            error_message: result.as_ref().err().map(|e| e.to_string()),
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("channel_type".to_string(), self.channel_type.clone());
                metadata.insert("behavior".to_string(), format!("{:?}", self.behavior));
                metadata
            },
        };

        {
            let mut log = self.delivery_log.write().await;
            log.push(record);
        }

        result
    }

    async fn apply_behavior_pattern(&self) -> Result<(), NotificationSystemError> {
        match &self.behavior {
            MockBehavior::AlwaysSucceed => Ok(()),

            MockBehavior::AlwaysFail { error_message } => {
                Err(NotificationSystemError::ChannelDeliveryFailed {
                    channel: self.channel_id.clone(),
                    message: error_message.clone(),
                    retry_count: 0,
                })
            }

            MockBehavior::DelayedSuccess { delay_ms } => {
                tokio::time::sleep(Duration::from_millis(*delay_ms)).await;
                Ok(())
            }

            MockBehavior::IntermittentFailure { failure_rate } => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                if rng.gen::<f64>() < *failure_rate {
                    Err(NotificationSystemError::ChannelDeliveryFailed {
                        channel: self.channel_id.clone(),
                        message: "Intermittent failure simulation".to_string(),
                        retry_count: 0,
                    })
                } else {
                    Ok(())
                }
            }

            MockBehavior::RateLimited { .. } => {
                let mut rate_limiter_guard = self.rate_limiter.write().await;
                if let Some(ref mut limiter) = rate_limiter_guard.as_mut() {
                    if limiter.can_proceed() {
                        Ok(())
                    } else {
                        Err(NotificationSystemError::RateLimitExceeded {
                            channel: self.channel_id.clone(),
                            current: 0, // Simplified
                            limit: limiter.max_per_second,
                        })
                    }
                } else {
                    Ok(())
                }
            }

            MockBehavior::CustomPattern { pattern } => {
                let mut index_guard = self.current_pattern_index.write().await;
                let index = *index_guard % pattern.len();
                *index_guard += 1;

                if pattern[index] {
                    Ok(())
                } else {
                    Err(NotificationSystemError::ChannelDeliveryFailed {
                        channel: self.channel_id.clone(),
                        message: format!("Custom pattern failure at index {}", index),
                        retry_count: 0,
                    })
                }
            }
        }
    }

    /// Get delivery statistics for this mock channel
    pub async fn get_delivery_stats(&self) -> MockChannelStats {
        let log = self.delivery_log.read().await;
        let total_deliveries = log.len();
        let successful_deliveries = log.iter().filter(|r| r.success).count();
        let failed_deliveries = total_deliveries - successful_deliveries;

        let total_duration: Duration = log.iter().map(|r| r.duration).sum();
        let average_duration = if total_deliveries > 0 {
            total_duration / total_deliveries as u32
        } else {
            Duration::from_nanos(0)
        };

        MockChannelStats {
            channel_id: self.channel_id.clone(),
            channel_type: self.channel_type.clone(),
            total_deliveries,
            successful_deliveries,
            failed_deliveries,
            average_duration,
            total_duration,
            success_rate: if total_deliveries > 0 {
                successful_deliveries as f64 / total_deliveries as f64
            } else {
                0.0
            },
        }
    }

    /// Get full delivery log
    pub async fn get_delivery_log(&self) -> Vec<MockDeliveryRecord> {
        let log = self.delivery_log.read().await;
        log.clone()
    }

    /// Clear delivery log
    pub async fn clear_delivery_log(&self) {
        let mut log = self.delivery_log.write().await;
        log.clear();
    }
}

/// Statistics for a mock channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockChannelStats {
    pub channel_id: String,
    pub channel_type: String,
    pub total_deliveries: usize,
    pub successful_deliveries: usize,
    pub failed_deliveries: usize,
    pub average_duration: Duration,
    pub total_duration: Duration,
    pub success_rate: f64,
}

/// Test scenario configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenario {
    pub name: String,
    pub description: String,
    pub notification_count: usize,
    pub notification_types: Vec<NotificationType>,
    pub channels: Vec<String>,
    pub duration_seconds: u64,
    pub expected_success_rate: f64,
    pub expected_max_latency_ms: u64,
    pub concurrent_senders: usize,
}

impl Default for TestScenario {
    fn default() -> Self {
        Self {
            name: "Basic Test".to_string(),
            description: "Basic notification delivery test".to_string(),
            notification_count: 10,
            notification_types: vec![NotificationType::Info],
            channels: vec!["mock_cli".to_string()],
            duration_seconds: 10,
            expected_success_rate: 1.0,
            expected_max_latency_ms: 1000,
            concurrent_senders: 1,
        }
    }
}

/// Test results for a scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenarioResults {
    pub scenario_name: String,
    pub total_notifications_sent: usize,
    pub successful_deliveries: usize,
    pub failed_deliveries: usize,
    pub actual_success_rate: f64,
    pub average_latency_ms: u64,
    pub max_latency_ms: u64,
    pub min_latency_ms: u64,
    pub total_duration: Duration,
    pub throughput_per_second: f64,
    pub channel_stats: HashMap<String, MockChannelStats>,
    pub test_passed: bool,
    pub failure_reasons: Vec<String>,
}

/// Comprehensive notification testing harness
pub struct NotificationTestingHarness {
    mock_channels: Arc<RwLock<HashMap<String, MockNotificationChannel>>>,
    notification_system: Option<NotificationSystem>,
    test_events: broadcast::Sender<TestEvent>,
    running_tests: Arc<RwLock<HashMap<String, TestScenario>>>,
}

/// Test events for monitoring
#[derive(Debug, Clone)]
pub enum TestEvent {
    ScenarioStarted { scenario_name: String, timestamp: DateTime<Utc> },
    ScenarioCompleted { scenario_name: String, results: TestScenarioResults },
    NotificationSent { notification_id: String, scenario: String },
    DeliveryCompleted { notification_id: String, success: bool, duration: Duration },
    TestError { scenario: String, error: String },
}

impl NotificationTestingHarness {
    /// Create a new testing harness
    pub fn new() -> Self {
        let (test_events, _) = broadcast::channel(1000);

        Self {
            mock_channels: Arc::new(RwLock::new(HashMap::new())),
            notification_system: None,
            test_events,
            running_tests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a mock channel to the harness
    pub async fn add_mock_channel(&self, channel: MockNotificationChannel) {
        let mut channels = self.mock_channels.write().await;
        channels.insert(channel.channel_id.clone(), channel);
    }

    /// Configure the notification system for testing
    pub async fn configure_notification_system(&mut self, system: NotificationSystem) {
        self.notification_system = Some(system);
    }

    /// Run a test scenario
    #[instrument(level = "info", skip(self, scenario))]
    pub async fn run_scenario(&self, scenario: TestScenario) -> Result<TestScenarioResults, TestHarnessError> {
        info!("Starting test scenario: {}", scenario.name);

        // Emit scenario started event
        let _ = self.test_events.send(TestEvent::ScenarioStarted {
            scenario_name: scenario.name.clone(),
            timestamp: Utc::now(),
        });

        // Track running test
        {
            let mut running_tests = self.running_tests.write().await;
            running_tests.insert(scenario.name.clone(), scenario.clone());
        }

        let start_time = Instant::now();
        let mut notifications_sent = 0;
        let mut successful_deliveries = 0;
        let mut failed_deliveries = 0;
        let mut latencies = Vec::new();
        let mut failure_reasons = Vec::new();

        if let Some(ref notification_system) = self.notification_system {
            // Clear mock channel logs
            self.clear_all_mock_logs().await;

            // Send notifications according to scenario
            if scenario.concurrent_senders == 1 {
                // Sequential sending
                for i in 0..scenario.notification_count {
                    let notification_type = scenario.notification_types[i % scenario.notification_types.len()].clone();
                    let channels = self.get_mock_channels_for_scenario(&scenario).await;

                    let notification = Notification::new(
                        notification_type,
                        format!("Test Notification {}", i),
                        format!("Scenario: {} - Message {}", scenario.name, i),
                        channels,
                    );

                    let notification_id = notification.id.clone();
                    let send_start = Instant::now();

                    match notification_system.send(notification).await {
                        Ok(_) => {
                            notifications_sent += 1;
                            successful_deliveries += 1;
                            let latency = send_start.elapsed();
                            latencies.push(latency);

                            // Emit notification sent event
                            let _ = self.test_events.send(TestEvent::NotificationSent {
                                notification_id: notification_id.clone(),
                                scenario: scenario.name.clone(),
                            });
                        }
                        Err(e) => {
                            failed_deliveries += 1;
                            failure_reasons.push(format!("Send failed: {}", e));

                            let _ = self.test_events.send(TestEvent::TestError {
                                scenario: scenario.name.clone(),
                                error: e.to_string(),
                            });
                        }
                    }
                }
            } else {
                // Concurrent sending
                let mut tasks = Vec::new();
                let notifications_per_sender = scenario.notification_count / scenario.concurrent_senders;

                for sender_id in 0..scenario.concurrent_senders {
                    let system = notification_system.clone();
                    let scenario_clone = scenario.clone();
                    let test_events = self.test_events.clone();

                    let task = tokio::spawn(async move {
                        let mut sender_results = Vec::new();

                        for i in 0..notifications_per_sender {
                            let notification_type = scenario_clone.notification_types[i % scenario_clone.notification_types.len()].clone();
                            // Note: In a full implementation, we'd need to get channels differently for concurrent scenarios
                            let channels = vec![NotificationChannel::CLI]; // Simplified

                            let notification = Notification::new(
                                notification_type,
                                format!("Concurrent Test {}-{}", sender_id, i),
                                format!("Scenario: {} - Sender {} Message {}", scenario_clone.name, sender_id, i),
                                channels,
                            );

                            let notification_id = notification.id.clone();
                            let send_start = Instant::now();

                            match system.send(notification).await {
                                Ok(_) => {
                                    let latency = send_start.elapsed();
                                    sender_results.push((true, latency, None));

                                    let _ = test_events.send(TestEvent::NotificationSent {
                                        notification_id,
                                        scenario: scenario_clone.name.clone(),
                                    });
                                }
                                Err(e) => {
                                    sender_results.push((false, send_start.elapsed(), Some(e.to_string())));
                                }
                            }
                        }

                        sender_results
                    });

                    tasks.push(task);
                }

                // Collect results from all concurrent senders
                for task in tasks {
                    match task.await {
                        Ok(sender_results) => {
                            for (success, latency, error) in sender_results {
                                notifications_sent += 1;
                                if success {
                                    successful_deliveries += 1;
                                } else {
                                    failed_deliveries += 1;
                                    if let Some(error) = error {
                                        failure_reasons.push(error);
                                    }
                                }
                                latencies.push(latency);
                            }
                        }
                        Err(e) => {
                            failure_reasons.push(format!("Concurrent task failed: {}", e));
                        }
                    }
                }
            }

            // Wait a bit for async deliveries to complete
            tokio::time::sleep(Duration::from_millis(100)).await;
        } else {
            return Err(TestHarnessError::MockConfigurationError(
                "No notification system configured".to_string()
            ));
        }

        let total_duration = start_time.elapsed();

        // Calculate metrics
        let actual_success_rate = if notifications_sent > 0 {
            successful_deliveries as f64 / notifications_sent as f64
        } else {
            0.0
        };

        let (average_latency_ms, max_latency_ms, min_latency_ms) = if !latencies.is_empty() {
            let total_latency: Duration = latencies.iter().sum();
            let avg = total_latency / latencies.len() as u32;
            let max = latencies.iter().max().unwrap_or(&Duration::from_nanos(0));
            let min = latencies.iter().min().unwrap_or(&Duration::from_nanos(0));
            (avg.as_millis() as u64, max.as_millis() as u64, min.as_millis() as u64)
        } else {
            (0, 0, 0)
        };

        let throughput_per_second = if total_duration.as_secs_f64() > 0.0 {
            notifications_sent as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        // Collect channel statistics
        let channel_stats = self.collect_channel_stats().await;

        // Determine if test passed
        let mut test_passed = true;
        if actual_success_rate < scenario.expected_success_rate {
            test_passed = false;
            failure_reasons.push(format!(
                "Success rate {} below expected {}",
                actual_success_rate, scenario.expected_success_rate
            ));
        }

        if max_latency_ms > scenario.expected_max_latency_ms {
            test_passed = false;
            failure_reasons.push(format!(
                "Max latency {}ms above expected {}ms",
                max_latency_ms, scenario.expected_max_latency_ms
            ));
        }

        let results = TestScenarioResults {
            scenario_name: scenario.name.clone(),
            total_notifications_sent: notifications_sent,
            successful_deliveries,
            failed_deliveries,
            actual_success_rate,
            average_latency_ms,
            max_latency_ms,
            min_latency_ms,
            total_duration,
            throughput_per_second,
            channel_stats,
            test_passed,
            failure_reasons,
        };

        // Emit scenario completed event
        let _ = self.test_events.send(TestEvent::ScenarioCompleted {
            scenario_name: scenario.name.clone(),
            results: results.clone(),
        });

        // Remove from running tests
        {
            let mut running_tests = self.running_tests.write().await;
            running_tests.remove(&scenario.name);
        }

        info!("Completed test scenario: {} - Success: {}", scenario.name, test_passed);
        Ok(results)
    }

    /// Subscribe to test events
    pub fn subscribe_to_test_events(&self) -> broadcast::Receiver<TestEvent> {
        self.test_events.subscribe()
    }

    /// Run stress test with high notification volume
    pub async fn run_stress_test(
        &self,
        notifications_per_second: u32,
        duration_seconds: u64,
    ) -> Result<TestScenarioResults, TestHarnessError> {
        let scenario = TestScenario {
            name: "Stress Test".to_string(),
            description: format!("Stress test: {} notifications/sec for {}s",
                               notifications_per_second, duration_seconds),
            notification_count: (notifications_per_second * duration_seconds as u32) as usize,
            notification_types: vec![NotificationType::Info, NotificationType::Warning],
            channels: vec!["mock_cli".to_string()],
            duration_seconds,
            expected_success_rate: 0.95, // Allow some failures under stress
            expected_max_latency_ms: 5000, // More lenient latency under stress
            concurrent_senders: 5, // Use multiple concurrent senders
        };

        self.run_scenario(scenario).await
    }

    async fn get_mock_channels_for_scenario(&self, scenario: &TestScenario) -> Vec<NotificationChannel> {
        // For now, return CLI channel - in a full implementation, this would map scenario channels to real channels
        vec![NotificationChannel::CLI]
    }

    async fn clear_all_mock_logs(&self) {
        let channels = self.mock_channels.read().await;
        for channel in channels.values() {
            channel.clear_delivery_log().await;
        }
    }

    async fn collect_channel_stats(&self) -> HashMap<String, MockChannelStats> {
        let mut stats = HashMap::new();
        let channels = self.mock_channels.read().await;

        for (channel_id, channel) in channels.iter() {
            let channel_stats = channel.get_delivery_stats().await;
            stats.insert(channel_id.clone(), channel_stats);
        }

        stats
    }
}

impl Default for NotificationTestingHarness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_channel_creation() {
        let channel = MockNotificationChannel::new("test_cli".to_string(), "CLI".to_string());
        assert_eq!(channel.channel_id, "test_cli");
        assert_eq!(channel.channel_type, "CLI");
    }

    #[tokio::test]
    async fn test_mock_channel_always_succeed() {
        let mut channel = MockNotificationChannel::new("test".to_string(), "CLI".to_string());
        channel.set_behavior(MockBehavior::AlwaysSucceed).await;

        let notification = Notification::new(
            NotificationType::Info,
            "Test".to_string(),
            "Test message".to_string(),
            vec![NotificationChannel::CLI],
        );

        let result = channel.deliver(&notification).await;
        assert!(result.is_ok());

        let stats = channel.get_delivery_stats().await;
        assert_eq!(stats.total_deliveries, 1);
        assert_eq!(stats.successful_deliveries, 1);
        assert_eq!(stats.success_rate, 1.0);
    }

    #[tokio::test]
    async fn test_mock_channel_always_fail() {
        let mut channel = MockNotificationChannel::new("test".to_string(), "CLI".to_string());
        channel.set_behavior(MockBehavior::AlwaysFail {
            error_message: "Test failure".to_string(),
        }).await;

        let notification = Notification::new(
            NotificationType::Info,
            "Test".to_string(),
            "Test message".to_string(),
            vec![NotificationChannel::CLI],
        );

        let result = channel.deliver(&notification).await;
        assert!(result.is_err());

        let stats = channel.get_delivery_stats().await;
        assert_eq!(stats.total_deliveries, 1);
        assert_eq!(stats.failed_deliveries, 1);
        assert_eq!(stats.success_rate, 0.0);
    }

    #[tokio::test]
    async fn test_testing_harness_creation() {
        let harness = NotificationTestingHarness::new();
        assert!(harness.mock_channels.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_scenario_configuration() {
        let scenario = TestScenario {
            name: "Test Scenario".to_string(),
            description: "Test scenario description".to_string(),
            notification_count: 5,
            notification_types: vec![NotificationType::Info, NotificationType::Warning],
            channels: vec!["mock_cli".to_string()],
            duration_seconds: 10,
            expected_success_rate: 1.0,
            expected_max_latency_ms: 500,
            concurrent_senders: 1,
        };

        assert_eq!(scenario.name, "Test Scenario");
        assert_eq!(scenario.notification_count, 5);
        assert_eq!(scenario.notification_types.len(), 2);
    }
}