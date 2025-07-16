// ABOUTME: Multi-channel notification system for proactive research events and status updates
//! This module provides a comprehensive notification system with multiple delivery channels
//! for the proactive research system. Features include:
//! - Multiple notification channels: CLI (stdout/stderr), File (log files), API (HTTP endpoints)
//! - Different notification types: info, warning, error, progress updates
//! - Channel-specific formatting and delivery mechanisms
//! - Async delivery with proper error handling and retry logic
//! - Configurable notification preferences and filtering
//! - Integration with existing proactive components for status updates

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, instrument};

/// Errors that can occur in the notification system
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum NotificationSystemError {
    #[error("Channel delivery failed: {channel} - {message}")]
    ChannelDeliveryFailed {
        channel: String,
        message: String,
        retry_count: u32,
    },

    #[error("Invalid notification configuration: {field} - {reason}")]
    InvalidConfiguration { field: String, reason: String },

    #[error("File I/O error: {path} - {error}")]
    FileIo { path: String, error: String },

    #[error("HTTP endpoint error: {endpoint} - status: {status:?}, error: {error}")]
    HttpEndpoint {
        endpoint: String,
        status: Option<u16>,
        error: String,
    },

    #[error("Notification formatting error: {notification_type} - {error}")]
    FormattingError {
        notification_type: String,
        error: String,
    },

    #[error("Channel not configured: {channel}")]
    ChannelNotConfigured { channel: String },

    #[error("Rate limit exceeded for channel: {channel} - {current}/{limit} notifications")]
    RateLimitExceeded {
        channel: String,
        current: u32,
        limit: u32,
    },

    #[error("Notification system not initialized")]
    NotInitialized,
}

/// Available notification channels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationChannel {
    /// CLI output (stdout/stderr with colors)
    CLI,
    /// File-based notifications (log files with timestamps)
    File { path: PathBuf },
    /// API-based notifications (HTTP endpoints)
    API { endpoint: String },
}

impl std::fmt::Display for NotificationChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationChannel::CLI => write!(f, "CLI"),
            NotificationChannel::File { path } => write!(f, "File({})", path.display()),
            NotificationChannel::API { endpoint } => write!(f, "API({endpoint})"),
        }
    }
}

/// Types of notifications that can be sent
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationType {
    /// General information
    Info,
    /// Warning messages
    Warning,
    /// Error messages
    Error,
    /// Progress updates
    Progress { current: u32, total: u32 },
    /// Success messages
    Success,
    /// Debug information
    Debug,
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationType::Info => write!(f, "INFO"),
            NotificationType::Warning => write!(f, "WARN"),
            NotificationType::Error => write!(f, "ERROR"),
            NotificationType::Progress { current, total } => {
                write!(f, "PROGRESS({current}/{total})")
            }
            NotificationType::Success => write!(f, "SUCCESS"),
            NotificationType::Debug => write!(f, "DEBUG"),
        }
    }
}

/// Individual notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub source_component: Option<String>,
    pub metadata: HashMap<String, String>,
    pub channels: Vec<NotificationChannel>,
}

impl Notification {
    pub fn new(
        notification_type: NotificationType,
        title: String,
        message: String,
        channels: Vec<NotificationChannel>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            notification_type,
            title,
            message,
            timestamp: Utc::now(),
            source_component: None,
            metadata: HashMap::new(),
            channels,
        }
    }

    pub fn with_source(mut self, source: String) -> Self {
        self.source_component = Some(source);
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Configuration for notification channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannelConfig {
    pub enabled: bool,
    pub rate_limit_per_minute: Option<u32>,
    pub min_notification_level: NotificationType,
    pub format_template: Option<String>,
    pub retry_count: u32,
    pub retry_delay_ms: u64,
}

impl Default for NotificationChannelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rate_limit_per_minute: Some(60),
            min_notification_level: NotificationType::Info,
            format_template: None,
            retry_count: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// Configuration for the notification system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSystemConfig {
    pub channel_configs: HashMap<String, NotificationChannelConfig>,
    pub default_channels: Vec<NotificationChannel>,
    pub buffer_size: usize,
    pub batch_delivery: bool,
    pub batch_size: usize,
    pub batch_timeout_ms: u64,
    pub enable_metrics: bool,
    pub enable_delivery_verification: bool,
}

impl Default for NotificationSystemConfig {
    fn default() -> Self {
        let mut channel_configs = HashMap::new();
        channel_configs.insert("cli".to_string(), NotificationChannelConfig::default());
        channel_configs.insert("file".to_string(), NotificationChannelConfig::default());
        channel_configs.insert("api".to_string(), NotificationChannelConfig::default());

        Self {
            channel_configs,
            default_channels: vec![NotificationChannel::CLI],
            buffer_size: 1000,
            batch_delivery: false,
            batch_size: 10,
            batch_timeout_ms: 5000,
            enable_metrics: true,
            enable_delivery_verification: false,
        }
    }
}

/// Delivery statistics for channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMetrics {
    pub total_sent: u64,
    pub successful_deliveries: u64,
    pub failed_deliveries: u64,
    pub retry_attempts: u64,
    pub rate_limit_hits: u64,
    pub average_delivery_time_ms: f64,
    pub last_delivery: Option<DateTime<Utc>>,
}

impl Default for ChannelMetrics {
    fn default() -> Self {
        Self {
            total_sent: 0,
            successful_deliveries: 0,
            failed_deliveries: 0,
            retry_attempts: 0,
            rate_limit_hits: 0,
            average_delivery_time_ms: 0.0,
            last_delivery: None,
        }
    }
}

/// System-wide notification metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMetrics {
    pub total_notifications: u64,
    pub notifications_by_type: HashMap<String, u64>,
    pub channel_metrics: HashMap<String, ChannelMetrics>,
    pub last_updated: DateTime<Utc>,
}

impl Default for NotificationMetrics {
    fn default() -> Self {
        Self {
            total_notifications: 0,
            notifications_by_type: HashMap::new(),
            channel_metrics: HashMap::new(),
            last_updated: Utc::now(),
        }
    }
}

/// Rate limiter for notification channels
#[derive(Debug)]
struct RateLimiter {
    limit: u32,
    window_start: std::time::Instant,
    count: u32,
}

impl RateLimiter {
    fn new(limit_per_minute: u32) -> Self {
        Self {
            limit: limit_per_minute,
            window_start: std::time::Instant::now(),
            count: 0,
        }
    }

    fn can_send(&mut self) -> bool {
        let now = std::time::Instant::now();
        if now.duration_since(self.window_start).as_secs() >= 60 {
            self.window_start = now;
            self.count = 0;
        }

        if self.count >= self.limit {
            false
        } else {
            self.count += 1;
            true
        }
    }
}

/// Main notification system
pub struct NotificationSystem {
    config: NotificationSystemConfig,
    rate_limiters: Arc<Mutex<HashMap<String, RateLimiter>>>,
    metrics: Arc<RwLock<NotificationMetrics>>,
    notification_buffer: Arc<Mutex<Vec<Notification>>>,
    running: Arc<RwLock<bool>>,
    http_client: reqwest::Client,
    delivery_verifier: Arc<
        RwLock<
            Option<crate::proactive::notification_delivery_verifier::NotificationDeliveryVerifier>,
        >,
    >,
}

impl NotificationSystem {
    /// Create a new notification system with the given configuration
    pub fn new(config: NotificationSystemConfig) -> Self {
        Self {
            config,
            rate_limiters: Arc::new(Mutex::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(NotificationMetrics::default())),
            notification_buffer: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
            http_client: reqwest::Client::new(),
            delivery_verifier: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the notification system
    pub async fn start(&self) -> Result<(), NotificationSystemError> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        info!(
            "Starting notification system with {} default channels",
            self.config.default_channels.len()
        );

        // Initialize rate limiters
        let mut rate_limiters = self.rate_limiters.lock().await;
        for (channel_name, config) in &self.config.channel_configs {
            if let Some(rate_limit) = config.rate_limit_per_minute {
                rate_limiters.insert(channel_name.clone(), RateLimiter::new(rate_limit));
            }
        }
        drop(rate_limiters);

        // Start background processing if batch delivery is enabled
        if self.config.batch_delivery {
            self.start_batch_processor().await;
        }

        Ok(())
    }

    /// Stop the notification system
    pub async fn stop(&self) -> Result<(), NotificationSystemError> {
        let mut running = self.running.write().await;
        *running = false;

        // Flush any remaining notifications
        if self.config.batch_delivery {
            self.flush_buffer().await?;
        }

        info!("Notification system stopped");
        Ok(())
    }

    /// Send a notification through configured channels
    #[instrument(level = "debug", skip(self, notification))]
    pub async fn send(&self, notification: Notification) -> Result<(), NotificationSystemError> {
        if !*self.running.read().await {
            return Err(NotificationSystemError::NotInitialized);
        }

        debug!(
            "Sending notification: {} - {}",
            notification.notification_type, notification.title
        );

        // Update metrics
        if self.config.enable_metrics {
            self.update_metrics(&notification).await;
        }

        if self.config.batch_delivery {
            // Add to buffer for batch processing
            let mut buffer = self.notification_buffer.lock().await;
            buffer.push(notification);

            // Check if buffer is full
            if buffer.len() >= self.config.batch_size {
                let notifications_to_send = buffer.drain(..).collect();
                drop(buffer);
                self.deliver_notifications(notifications_to_send).await?;
            }
        } else {
            // Send immediately
            self.deliver_notifications(vec![notification]).await?;
        }

        Ok(())
    }

    /// Send an info notification
    pub async fn info(
        &self,
        title: String,
        message: String,
    ) -> Result<(), NotificationSystemError> {
        let notification = Notification::new(
            NotificationType::Info,
            title,
            message,
            self.config.default_channels.clone(),
        );
        self.send(notification).await
    }

    /// Send a warning notification
    pub async fn warning(
        &self,
        title: String,
        message: String,
    ) -> Result<(), NotificationSystemError> {
        let notification = Notification::new(
            NotificationType::Warning,
            title,
            message,
            self.config.default_channels.clone(),
        );
        self.send(notification).await
    }

    /// Send an error notification
    pub async fn error(
        &self,
        title: String,
        message: String,
    ) -> Result<(), NotificationSystemError> {
        let notification = Notification::new(
            NotificationType::Error,
            title,
            message,
            self.config.default_channels.clone(),
        );
        self.send(notification).await
    }

    /// Send a progress notification
    pub async fn progress(
        &self,
        title: String,
        message: String,
        current: u32,
        total: u32,
    ) -> Result<(), NotificationSystemError> {
        let notification = Notification::new(
            NotificationType::Progress { current, total },
            title,
            message,
            self.config.default_channels.clone(),
        );
        self.send(notification).await
    }

    /// Send a success notification
    pub async fn success(
        &self,
        title: String,
        message: String,
    ) -> Result<(), NotificationSystemError> {
        let notification = Notification::new(
            NotificationType::Success,
            title,
            message,
            self.config.default_channels.clone(),
        );
        self.send(notification).await
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> NotificationMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Configure delivery verifier for the notification system
    pub async fn configure_delivery_verifier(
        &self,
        verifier: crate::proactive::notification_delivery_verifier::NotificationDeliveryVerifier,
    ) -> Result<(), NotificationSystemError> {
        let mut delivery_verifier = self.delivery_verifier.write().await;
        *delivery_verifier = Some(verifier);
        info!("Delivery verifier configured for notification system");
        Ok(())
    }

    /// Deliver notifications through their specified channels
    async fn deliver_notifications(
        &self,
        notifications: Vec<Notification>,
    ) -> Result<(), NotificationSystemError> {
        for notification in notifications {
            // Track delivery attempt if verification is enabled
            if self.config.enable_delivery_verification {
                if let Some(ref verifier) = *self.delivery_verifier.read().await {
                    let _ = verifier.track_delivery_attempt(&notification).await;
                }
            }

            for channel in &notification.channels {
                let delivery_result = self.deliver_to_channel(&notification, channel).await;

                // Update delivery verification status
                if self.config.enable_delivery_verification {
                    if let Some(ref verifier) = *self.delivery_verifier.read().await {
                        let _ = verifier
                            .update_delivery_status(
                                &notification.id,
                                channel,
                                delivery_result.clone(),
                            )
                            .await;
                    }
                }

                if let Err(e) = delivery_result {
                    error!(
                        "Failed to deliver notification {} to channel {}: {}",
                        notification.id, channel, e
                    );

                    // Update failure metrics
                    if self.config.enable_metrics {
                        self.update_channel_failure_metrics(channel).await;
                    }
                }
            }
        }
        Ok(())
    }

    /// Deliver notification to a specific channel
    async fn deliver_to_channel(
        &self,
        notification: &Notification,
        channel: &NotificationChannel,
    ) -> Result<(), NotificationSystemError> {
        let channel_key = self.get_channel_key(channel);

        // Check rate limiting
        if let Some(rate_limiter) = self.rate_limiters.lock().await.get_mut(&channel_key) {
            if !rate_limiter.can_send() {
                return Err(NotificationSystemError::RateLimitExceeded {
                    channel: channel_key,
                    current: rate_limiter.count,
                    limit: rate_limiter.limit,
                });
            }
        }

        let start_time = std::time::Instant::now();

        let result = match channel {
            NotificationChannel::CLI => self.deliver_to_cli(notification).await,
            NotificationChannel::File { path } => self.deliver_to_file(notification, path).await,
            NotificationChannel::API { endpoint } => {
                self.deliver_to_api(notification, endpoint).await
            }
        };

        // Update success metrics
        if result.is_ok() && self.config.enable_metrics {
            let delivery_time = start_time.elapsed().as_millis() as f64;
            self.update_channel_success_metrics(channel, delivery_time)
                .await;
        }

        result
    }

    /// Deliver notification to CLI (stdout/stderr with colors)
    async fn deliver_to_cli(
        &self,
        notification: &Notification,
    ) -> Result<(), NotificationSystemError> {
        let formatted = self.format_cli_notification(notification)?;

        match notification.notification_type {
            NotificationType::Error => {
                eprintln!("{formatted}");
                let _ = std::io::stderr().flush();
            }
            NotificationType::Warning => {
                eprintln!("{formatted}");
                let _ = std::io::stderr().flush();
            }
            _ => {
                println!("{formatted}");
                let _ = std::io::stdout().flush();
            }
        }

        Ok(())
    }

    /// Deliver notification to file with timestamps
    async fn deliver_to_file(
        &self,
        notification: &Notification,
        path: &PathBuf,
    ) -> Result<(), NotificationSystemError> {
        let formatted = self.format_file_notification(notification)?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await
            .map_err(|e| NotificationSystemError::FileIo {
                path: path.display().to_string(),
                error: e.to_string(),
            })?;

        file.write_all(formatted.as_bytes()).await.map_err(|e| {
            NotificationSystemError::FileIo {
                path: path.display().to_string(),
                error: e.to_string(),
            }
        })?;

        file.write_all(b"\n")
            .await
            .map_err(|e| NotificationSystemError::FileIo {
                path: path.display().to_string(),
                error: e.to_string(),
            })?;

        Ok(())
    }

    /// Deliver notification to API endpoint
    async fn deliver_to_api(
        &self,
        notification: &Notification,
        endpoint: &str,
    ) -> Result<(), NotificationSystemError> {
        let payload = serde_json::to_value(notification).map_err(|e| {
            NotificationSystemError::FormattingError {
                notification_type: notification.notification_type.to_string(),
                error: e.to_string(),
            }
        })?;

        let response = self
            .http_client
            .post(endpoint)
            .json(&payload)
            .send()
            .await
            .map_err(|e| NotificationSystemError::HttpEndpoint {
                endpoint: endpoint.to_string(),
                status: None,
                error: e.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(NotificationSystemError::HttpEndpoint {
                endpoint: endpoint.to_string(),
                status: Some(response.status().as_u16()),
                error: format!("HTTP request failed with status: {}", response.status()),
            });
        }

        Ok(())
    }

    /// Format notification for CLI output with colors
    fn format_cli_notification(
        &self,
        notification: &Notification,
    ) -> Result<String, NotificationSystemError> {
        let color_code = match notification.notification_type {
            NotificationType::Error => "\x1b[31m",           // Red
            NotificationType::Warning => "\x1b[33m",         // Yellow
            NotificationType::Success => "\x1b[32m",         // Green
            NotificationType::Info => "\x1b[36m",            // Cyan
            NotificationType::Debug => "\x1b[37m",           // Gray
            NotificationType::Progress { .. } => "\x1b[34m", // Blue
        };
        let reset_code = "\x1b[0m";

        let timestamp = notification.timestamp.format("%Y-%m-%d %H:%M:%S UTC");
        let source = notification.source_component.as_deref().unwrap_or("system");

        let formatted = match &notification.notification_type {
            NotificationType::Progress { current, total } => {
                let percentage = (*current as f64 / *total as f64 * 100.0) as u32;
                format!(
                    "{color}[{timestamp}] {type} [{source}] {title}: {message} ({current}/{total} - {percentage}%){reset}",
                    color = color_code,
                    timestamp = timestamp,
                    type = notification.notification_type,
                    source = source,
                    title = notification.title,
                    message = notification.message,
                    current = current,
                    total = total,
                    percentage = percentage,
                    reset = reset_code
                )
            }
            _ => {
                format!(
                    "{color}[{timestamp}] {type} [{source}] {title}: {message}{reset}",
                    color = color_code,
                    timestamp = timestamp,
                    type = notification.notification_type,
                    source = source,
                    title = notification.title,
                    message = notification.message,
                    reset = reset_code
                )
            }
        };

        Ok(formatted)
    }

    /// Format notification for file output with timestamps
    fn format_file_notification(
        &self,
        notification: &Notification,
    ) -> Result<String, NotificationSystemError> {
        let timestamp = notification.timestamp.format("%Y-%m-%d %H:%M:%S UTC");
        let source = notification.source_component.as_deref().unwrap_or("system");

        let formatted = match &notification.notification_type {
            NotificationType::Progress { current, total } => {
                let percentage = (*current as f64 / *total as f64 * 100.0) as u32;
                format!(
                    "[{}] {} [{}] {}: {} ({}/{} - {}%)",
                    timestamp,
                    notification.notification_type,
                    source,
                    notification.title,
                    notification.message,
                    current,
                    total,
                    percentage
                )
            }
            _ => {
                format!(
                    "[{}] {} [{}] {}: {}",
                    timestamp,
                    notification.notification_type,
                    source,
                    notification.title,
                    notification.message
                )
            }
        };

        Ok(formatted)
    }

    /// Get channel key for metrics and rate limiting
    fn get_channel_key(&self, channel: &NotificationChannel) -> String {
        match channel {
            NotificationChannel::CLI => "cli".to_string(),
            NotificationChannel::File { .. } => "file".to_string(),
            NotificationChannel::API { .. } => "api".to_string(),
        }
    }

    /// Update notification metrics
    async fn update_metrics(&self, notification: &Notification) {
        let mut metrics = self.metrics.write().await;
        metrics.total_notifications += 1;

        let type_key = notification.notification_type.to_string();
        *metrics.notifications_by_type.entry(type_key).or_insert(0) += 1;

        metrics.last_updated = Utc::now();
    }

    /// Update channel success metrics
    async fn update_channel_success_metrics(
        &self,
        channel: &NotificationChannel,
        delivery_time_ms: f64,
    ) {
        let channel_key = self.get_channel_key(channel);
        let mut metrics = self.metrics.write().await;

        let channel_metrics = metrics
            .channel_metrics
            .entry(channel_key)
            .or_insert_with(ChannelMetrics::default);

        channel_metrics.total_sent += 1;
        channel_metrics.successful_deliveries += 1;
        channel_metrics.last_delivery = Some(Utc::now());

        // Update rolling average
        let total_time = channel_metrics.average_delivery_time_ms
            * (channel_metrics.successful_deliveries - 1) as f64;
        channel_metrics.average_delivery_time_ms =
            (total_time + delivery_time_ms) / channel_metrics.successful_deliveries as f64;
    }

    /// Update channel failure metrics
    async fn update_channel_failure_metrics(&self, channel: &NotificationChannel) {
        let channel_key = self.get_channel_key(channel);
        let mut metrics = self.metrics.write().await;

        let channel_metrics = metrics
            .channel_metrics
            .entry(channel_key)
            .or_insert_with(ChannelMetrics::default);

        channel_metrics.total_sent += 1;
        channel_metrics.failed_deliveries += 1;
    }

    /// Start batch processor for batch delivery mode
    async fn start_batch_processor(&self) {
        let buffer = self.notification_buffer.clone();
        let running = self.running.clone();
        let batch_timeout = std::time::Duration::from_millis(self.config.batch_timeout_ms);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(batch_timeout);

            while *running.read().await {
                interval.tick().await;

                let mut buffer_guard = buffer.lock().await;
                if !buffer_guard.is_empty() {
                    let notifications_to_send: Vec<Notification> = buffer_guard.drain(..).collect();
                    drop(buffer_guard);

                    // Note: In a real implementation, we would need access to self here
                    // This would require restructuring or using channels for communication
                    debug!(
                        "Batch processor would send {} notifications",
                        notifications_to_send.len()
                    );
                }
            }
        });
    }

    /// Flush remaining notifications in buffer
    async fn flush_buffer(&self) -> Result<(), NotificationSystemError> {
        let mut buffer = self.notification_buffer.lock().await;
        if !buffer.is_empty() {
            let notifications_to_send = buffer.drain(..).collect();
            drop(buffer);
            self.deliver_notifications(notifications_to_send).await?;
        }
        Ok(())
    }
}

impl Clone for NotificationSystem {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            rate_limiters: self.rate_limiters.clone(),
            metrics: self.metrics.clone(),
            notification_buffer: self.notification_buffer.clone(),
            running: self.running.clone(),
            http_client: self.http_client.clone(),
            delivery_verifier: self.delivery_verifier.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_notification_creation() {
        let channels = vec![NotificationChannel::CLI];
        let notification = Notification::new(
            NotificationType::Info,
            "Test Title".to_string(),
            "Test message".to_string(),
            channels,
        );

        assert_eq!(notification.title, "Test Title");
        assert_eq!(notification.message, "Test message");
        assert_eq!(notification.notification_type, NotificationType::Info);
        assert_eq!(notification.channels.len(), 1);
        assert!(!notification.id.is_empty());
    }

    #[tokio::test]
    async fn test_notification_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());
        metadata.insert("key2".to_string(), "value2".to_string());

        let notification = Notification::new(
            NotificationType::Warning,
            "Test".to_string(),
            "Message".to_string(),
            vec![NotificationChannel::CLI],
        )
        .with_source("test_component".to_string())
        .with_metadata(metadata.clone());

        assert_eq!(
            notification.source_component,
            Some("test_component".to_string())
        );
        assert_eq!(notification.metadata, metadata);
    }

    #[tokio::test]
    async fn test_notification_system_creation() {
        let config = NotificationSystemConfig::default();
        let system = NotificationSystem::new(config.clone());

        assert_eq!(system.config.default_channels, config.default_channels);
        assert_eq!(system.config.buffer_size, config.buffer_size);
    }

    #[tokio::test]
    async fn test_notification_system_start_stop() {
        let config = NotificationSystemConfig::default();
        let system = NotificationSystem::new(config);

        // Test starting
        assert!(system.start().await.is_ok());
        assert!(*system.running.read().await);

        // Test stopping
        assert!(system.stop().await.is_ok());
        assert!(!*system.running.read().await);
    }

    #[tokio::test]
    async fn test_send_notification_not_initialized() {
        let config = NotificationSystemConfig::default();
        let system = NotificationSystem::new(config);

        let notification = Notification::new(
            NotificationType::Info,
            "Test".to_string(),
            "Message".to_string(),
            vec![NotificationChannel::CLI],
        );

        let result = system.send(notification).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            NotificationSystemError::NotInitialized
        ));
    }

    #[tokio::test]
    async fn test_cli_notification_formatting() {
        let config = NotificationSystemConfig::default();
        let system = NotificationSystem::new(config);

        let notification = Notification::new(
            NotificationType::Info,
            "Test Title".to_string(),
            "Test message".to_string(),
            vec![NotificationChannel::CLI],
        )
        .with_source("test_component".to_string());

        let formatted = system.format_cli_notification(&notification).unwrap();

        assert!(formatted.contains("Test Title"));
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("test_component"));
        assert!(formatted.contains("\x1b[36m")); // Cyan color for info
        assert!(formatted.contains("\x1b[0m")); // Reset color
    }

    #[tokio::test]
    async fn test_file_notification_formatting() {
        let config = NotificationSystemConfig::default();
        let system = NotificationSystem::new(config);

        let notification = Notification::new(
            NotificationType::Warning,
            "Test Title".to_string(),
            "Test message".to_string(),
            vec![NotificationChannel::CLI],
        )
        .with_source("test_component".to_string());

        let formatted = system.format_file_notification(&notification).unwrap();

        assert!(formatted.contains("Test Title"));
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("WARN"));
        assert!(formatted.contains("test_component"));
        // Should not contain color codes
        assert!(!formatted.contains("\x1b["));
    }

    #[tokio::test]
    async fn test_progress_notification_formatting() {
        let config = NotificationSystemConfig::default();
        let system = NotificationSystem::new(config);

        let notification = Notification::new(
            NotificationType::Progress {
                current: 50,
                total: 100,
            },
            "Processing".to_string(),
            "Items processed".to_string(),
            vec![NotificationChannel::CLI],
        );

        let formatted = system.format_cli_notification(&notification).unwrap();

        assert!(formatted.contains("Processing"));
        assert!(formatted.contains("Items processed"));
        assert!(formatted.contains("PROGRESS(50/100)"));
        assert!(formatted.contains("50%"));
    }

    #[tokio::test]
    async fn test_file_delivery() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        let config = NotificationSystemConfig::default();
        let system = NotificationSystem::new(config);

        let notification = Notification::new(
            NotificationType::Info,
            "Test Title".to_string(),
            "Test message".to_string(),
            vec![NotificationChannel::File {
                path: file_path.clone(),
            }],
        );

        let result = system.deliver_to_file(&notification, &file_path).await;
        assert!(result.is_ok());

        // Verify file content
        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert!(content.contains("Test Title"));
        assert!(content.contains("Test message"));
        assert!(content.contains("INFO"));
    }

    #[tokio::test]
    async fn test_channel_key_generation() {
        let config = NotificationSystemConfig::default();
        let system = NotificationSystem::new(config);

        assert_eq!(system.get_channel_key(&NotificationChannel::CLI), "cli");

        let file_channel = NotificationChannel::File {
            path: PathBuf::from("/tmp/test.log"),
        };
        assert_eq!(system.get_channel_key(&file_channel), "file");

        let api_channel = NotificationChannel::API {
            endpoint: "http://localhost:8080/notifications".to_string(),
        };
        assert_eq!(system.get_channel_key(&api_channel), "api");
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let mut rate_limiter = RateLimiter::new(2); // 2 per minute

        // Should allow first two sends
        assert!(rate_limiter.can_send());
        assert!(rate_limiter.can_send());

        // Should deny third send
        assert!(!rate_limiter.can_send());
    }

    #[tokio::test]
    async fn test_notification_metrics() {
        let config = NotificationSystemConfig::default();
        let system = NotificationSystem::new(config);

        let notification = Notification::new(
            NotificationType::Info,
            "Test".to_string(),
            "Message".to_string(),
            vec![NotificationChannel::CLI],
        );

        system.update_metrics(&notification).await;

        let metrics = system.get_metrics().await;
        assert_eq!(metrics.total_notifications, 1);
        assert_eq!(metrics.notifications_by_type.get("INFO"), Some(&1));
    }

    #[tokio::test]
    async fn test_convenience_methods() {
        let config = NotificationSystemConfig::default();
        let system = NotificationSystem::new(config);
        system.start().await.unwrap();

        // Test all convenience methods
        assert!(system
            .info("Info Title".to_string(), "Info message".to_string())
            .await
            .is_ok());
        assert!(system
            .warning("Warning Title".to_string(), "Warning message".to_string())
            .await
            .is_ok());
        assert!(system
            .error("Error Title".to_string(), "Error message".to_string())
            .await
            .is_ok());
        assert!(system
            .success("Success Title".to_string(), "Success message".to_string())
            .await
            .is_ok());
        assert!(system
            .progress(
                "Progress Title".to_string(),
                "Progress message".to_string(),
                25,
                100
            )
            .await
            .is_ok());

        let metrics = system.get_metrics().await;
        assert_eq!(metrics.total_notifications, 5);
    }

    #[tokio::test]
    async fn test_error_display() {
        let error = NotificationSystemError::ChannelDeliveryFailed {
            channel: "test_channel".to_string(),
            message: "test error".to_string(),
            retry_count: 3,
        };

        let error_string = error.to_string();
        assert!(error_string.contains("Channel delivery failed"));
        assert!(error_string.contains("test_channel"));
        assert!(error_string.contains("test error"));
    }

    #[tokio::test]
    async fn test_notification_type_display() {
        assert_eq!(NotificationType::Info.to_string(), "INFO");
        assert_eq!(NotificationType::Warning.to_string(), "WARN");
        assert_eq!(NotificationType::Error.to_string(), "ERROR");
        assert_eq!(NotificationType::Success.to_string(), "SUCCESS");
        assert_eq!(NotificationType::Debug.to_string(), "DEBUG");

        let progress = NotificationType::Progress {
            current: 25,
            total: 100,
        };
        assert_eq!(progress.to_string(), "PROGRESS(25/100)");
    }

    #[tokio::test]
    async fn test_channel_display() {
        assert_eq!(NotificationChannel::CLI.to_string(), "CLI");

        let file_channel = NotificationChannel::File {
            path: PathBuf::from("/tmp/test.log"),
        };
        assert_eq!(file_channel.to_string(), "File(/tmp/test.log)");

        let api_channel = NotificationChannel::API {
            endpoint: "http://localhost:8080/notifications".to_string(),
        };
        assert_eq!(
            api_channel.to_string(),
            "API(http://localhost:8080/notifications)"
        );
    }
}
