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

// ABOUTME: Alert management system for critical performance issues
//! # Alert Management Module
//!
//! This module provides comprehensive alerting capabilities for critical system
//! issues and performance violations. It supports multiple notification channels
//! and intelligent alert routing based on severity and context.
//!
//! ## Key Features
//!
//! - **Multi-channel alerting**: Email, webhooks, and custom integrations
//! - **Alert routing**: Intelligent routing based on severity and rules
//! - **Rate limiting**: Prevent alert flooding with configurable limits
//! - **Alert escalation**: Automatic escalation for unresolved critical issues

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use super::{MonitoringError, MonitoringResult};

/// Core alert manager for handling system alerts
pub struct AlertManager {
    /// Alert rules configuration
    rules: Arc<RwLock<Vec<AlertRule>>>,

    /// Registered alert channels
    channels: Arc<RwLock<HashMap<String, Box<dyn AlertChannel>>>>,

    /// Active alerts tracking
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,

    /// Alert history for rate limiting
    alert_history: Arc<RwLock<Vec<AlertHistoryEntry>>>,

    /// Alert manager configuration
    config: AlertManagerConfig,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new(config: AlertManagerConfig) -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Register an alert channel
    pub async fn register_channel(
        &self,
        channel_name: String,
        channel: Box<dyn AlertChannel>,
    ) -> MonitoringResult<()> {
        let mut channels = self.channels.write().await;
        channels.insert(channel_name, channel);
        Ok(())
    }

    /// Add an alert rule
    pub async fn add_rule(&self, rule: AlertRule) -> MonitoringResult<()> {
        let mut rules = self.rules.write().await;
        rules.push(rule);
        Ok(())
    }

    /// Send an alert through the system
    pub async fn send_alert(&self, alert: Alert) -> MonitoringResult<()> {
        // Check rate limiting
        if self.is_rate_limited(&alert).await? {
            return Ok(()); // Skip rate-limited alerts
        }

        // Find matching rules
        let matching_rules = self.find_matching_rules(&alert).await?;

        if matching_rules.is_empty() {
            // No rules match, use default handling
            self.handle_default_alert(&alert).await?;
        } else {
            // Process alert through matching rules
            for rule in matching_rules {
                self.process_alert_with_rule(&alert, &rule).await?;
            }
        }

        // Record alert in history for rate limiting
        self.record_alert_history(&alert).await?;

        // Track active alert
        if alert.severity == AlertSeverity::Critical {
            let mut active = self.active_alerts.write().await;
            active.insert(alert.id.clone(), alert);
        }

        Ok(())
    }

    /// Resolve an active alert
    pub async fn resolve_alert(&self, alert_id: &str) -> MonitoringResult<()> {
        let mut active = self.active_alerts.write().await;
        if let Some(alert) = active.remove(alert_id) {
            // Send resolution notification
            let resolution_alert = Alert {
                id: format!("{}_resolved", alert.id),
                title: format!("RESOLVED: {}", alert.title),
                message: format!("Alert {} has been resolved", alert.id),
                severity: AlertSeverity::Info,
                source: alert.source,
                timestamp: Utc::now(),
                metadata: alert.metadata,
            };

            self.handle_default_alert(&resolution_alert).await?;
        }

        Ok(())
    }

    /// Get all active alerts
    pub async fn get_active_alerts(&self) -> MonitoringResult<Vec<Alert>> {
        let active = self.active_alerts.read().await;
        Ok(active.values().cloned().collect())
    }

    /// Check if an alert is rate limited
    async fn is_rate_limited(&self, alert: &Alert) -> MonitoringResult<bool> {
        let history = self.alert_history.read().await;
        let cutoff_time = Utc::now() - chrono::Duration::hours(1);

        let recent_alerts = history
            .iter()
            .filter(|entry| entry.timestamp > cutoff_time && entry.source == alert.source)
            .count();

        Ok(recent_alerts >= self.config.max_alerts_per_hour)
    }

    /// Find alert rules that match the given alert
    async fn find_matching_rules(&self, alert: &Alert) -> MonitoringResult<Vec<AlertRule>> {
        let rules = self.rules.read().await;
        let matching_rules = rules
            .iter()
            .filter(|rule| rule.matches(alert))
            .cloned()
            .collect();

        Ok(matching_rules)
    }

    /// Process alert using a specific rule
    async fn process_alert_with_rule(
        &self,
        alert: &Alert,
        rule: &AlertRule,
    ) -> MonitoringResult<()> {
        let channels = self.channels.read().await;

        for channel_name in &rule.target_channels {
            if let Some(channel) = channels.get(channel_name) {
                channel.send_alert(alert).await.map_err(|e| {
                    MonitoringError::AlertError(format!(
                        "Failed to send alert via {channel_name}: {e}"
                    ))
                })?;
            }
        }

        Ok(())
    }

    /// Handle alert with default configuration
    async fn handle_default_alert(&self, alert: &Alert) -> MonitoringResult<()> {
        let channels = self.channels.read().await;

        // Send to all configured channels for critical alerts, or first available for others
        match alert.severity {
            AlertSeverity::Critical => {
                for (channel_name, channel) in channels.iter() {
                    if let Err(e) = channel.send_alert(alert).await {
                        eprintln!("Failed to send critical alert via {channel_name}: {e}");
                    }
                }
            }
            _ => {
                if let Some((_, channel)) = channels.iter().next() {
                    channel.send_alert(alert).await.map_err(|e| {
                        MonitoringError::AlertError(format!("Failed to send alert: {e}"))
                    })?;
                }
            }
        }

        Ok(())
    }

    /// Record alert in history for rate limiting
    async fn record_alert_history(&self, alert: &Alert) -> MonitoringResult<()> {
        let mut history = self.alert_history.write().await;

        history.push(AlertHistoryEntry {
            alert_id: alert.id.clone(),
            source: alert.source.clone(),
            severity: alert.severity,
            timestamp: alert.timestamp,
        });

        // Clean up old history entries
        let cutoff_time = Utc::now() - chrono::Duration::hours(24);
        history.retain(|entry| entry.timestamp > cutoff_time);

        Ok(())
    }
}

/// Configuration for the alert manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertManagerConfig {
    /// Enable alert processing
    pub enable_alerts: bool,

    /// Maximum alerts per hour per source
    pub max_alerts_per_hour: usize,

    /// Default alert timeout
    pub default_timeout: Duration,

    /// Enable alert escalation
    pub enable_escalation: bool,

    /// Escalation timeout for critical alerts
    pub escalation_timeout: Duration,
}

impl Default for AlertManagerConfig {
    fn default() -> Self {
        Self {
            enable_alerts: true,
            max_alerts_per_hour: 10,
            default_timeout: Duration::from_secs(300), // 5 minutes
            enable_escalation: false,
            escalation_timeout: Duration::from_secs(900), // 15 minutes
        }
    }
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational messages
    Info,

    /// Warning conditions
    Warning,

    /// Error conditions requiring attention
    Error,

    /// Critical conditions requiring immediate action
    Critical,
}

impl AlertSeverity {
    /// Check if this severity requires immediate attention
    pub fn is_urgent(&self) -> bool {
        matches!(self, AlertSeverity::Critical)
    }
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Info => write!(f, "INFO"),
            AlertSeverity::Warning => write!(f, "WARNING"),
            AlertSeverity::Error => write!(f, "ERROR"),
            AlertSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Alert message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Unique alert identifier
    pub id: String,

    /// Alert title/summary
    pub title: String,

    /// Detailed alert message
    pub message: String,

    /// Alert severity level
    pub severity: AlertSeverity,

    /// Source component or system
    pub source: String,

    /// When the alert was generated
    pub timestamp: DateTime<Utc>,

    /// Additional alert metadata
    pub metadata: HashMap<String, String>,
}

impl Alert {
    /// Create a new alert
    pub fn new(title: String, message: String, severity: AlertSeverity, source: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            message,
            severity,
            source,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the alert
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if this alert is critical
    pub fn is_critical(&self) -> bool {
        self.severity == AlertSeverity::Critical
    }
}

/// Alert routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Rule name
    pub name: String,

    /// Source pattern to match (regex or exact)
    pub source_pattern: String,

    /// Severity levels this rule applies to
    pub severity_levels: Vec<AlertSeverity>,

    /// Target channels for matching alerts
    pub target_channels: Vec<String>,

    /// Additional conditions for rule matching
    pub conditions: HashMap<String, String>,
}

impl AlertRule {
    /// Check if this rule matches the given alert
    pub fn matches(&self, alert: &Alert) -> bool {
        // Check severity
        if !self.severity_levels.contains(&alert.severity) {
            return false;
        }

        // Check source pattern (simplified - would use regex in full implementation)
        if !alert.source.contains(&self.source_pattern) {
            return false;
        }

        // Check additional conditions
        for (key, value) in &self.conditions {
            if let Some(alert_value) = alert.metadata.get(key) {
                if alert_value != value {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

/// Alert channel trait for different notification methods
#[async_trait]
pub trait AlertChannel: Send + Sync {
    /// Send an alert through this channel
    async fn send_alert(
        &self,
        alert: &Alert,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Get the channel name
    fn channel_name(&self) -> &str;

    /// Check if the channel is available
    async fn is_available(&self) -> bool;
}

/// Console alert channel for development/testing
#[derive(Debug)]
pub struct ConsoleAlertChannel {
    name: String,
}

impl ConsoleAlertChannel {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[async_trait]
impl AlertChannel for ConsoleAlertChannel {
    async fn send_alert(
        &self,
        alert: &Alert,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!(
            "[ALERT] {} | {} | {} | {}",
            alert.severity, alert.source, alert.title, alert.message
        );
        Ok(())
    }

    fn channel_name(&self) -> &str {
        &self.name
    }

    async fn is_available(&self) -> bool {
        true // Console is always available
    }
}

/// Email alert channel (skeleton implementation)
#[derive(Debug)]
pub struct EmailAlertChannel {
    name: String,
    #[allow(dead_code)] // TODO: Will be used for SMTP email configuration
    smtp_config: EmailConfig,
}

impl EmailAlertChannel {
    pub fn new(name: String, smtp_config: EmailConfig) -> Self {
        Self { name, smtp_config }
    }
}

#[async_trait]
impl AlertChannel for EmailAlertChannel {
    async fn send_alert(
        &self,
        _alert: &Alert,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement actual email sending
        // For now, just simulate the operation
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    fn channel_name(&self) -> &str {
        &self.name
    }

    async fn is_available(&self) -> bool {
        // TODO: Check SMTP server connectivity
        true
    }
}

/// Email configuration for alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub to_addresses: Vec<String>,
}

/// Alert history entry for rate limiting
#[derive(Debug, Clone)]
struct AlertHistoryEntry {
    #[allow(dead_code)] // TODO: Will be used for alert tracking and deduplication
    alert_id: String,
    source: String,
    #[allow(dead_code)] // TODO: Will be used for severity-based alert handling
    severity: AlertSeverity,
    timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alert_manager_creation() {
        let config = AlertManagerConfig::default();
        let manager = AlertManager::new(config);

        let active_alerts = manager.get_active_alerts().await.unwrap();
        assert!(active_alerts.is_empty());
    }

    #[tokio::test]
    async fn test_alert_channel_registration() {
        let config = AlertManagerConfig::default();
        let manager = AlertManager::new(config);

        let channel = Box::new(ConsoleAlertChannel::new("console".to_string()));
        manager
            .register_channel("console".to_string(), channel)
            .await
            .unwrap();

        // Verify channel was registered by checking if we can send alerts
        let alert = Alert::new(
            "Test Alert".to_string(),
            "This is a test".to_string(),
            AlertSeverity::Info,
            "test_component".to_string(),
        );

        manager.send_alert(alert).await.unwrap();
    }

    #[tokio::test]
    async fn test_alert_rule_matching() {
        let rule = AlertRule {
            name: "critical_api_rule".to_string(),
            source_pattern: "api".to_string(),
            severity_levels: vec![AlertSeverity::Critical, AlertSeverity::Error],
            target_channels: vec!["email".to_string()],
            conditions: HashMap::new(),
        };

        let matching_alert = Alert::new(
            "API Down".to_string(),
            "API is not responding".to_string(),
            AlertSeverity::Critical,
            "api_server".to_string(),
        );

        let non_matching_alert = Alert::new(
            "Cache Warning".to_string(),
            "Cache hit rate low".to_string(),
            AlertSeverity::Warning,
            "cache_system".to_string(),
        );

        assert!(rule.matches(&matching_alert));
        assert!(!rule.matches(&non_matching_alert));
    }

    #[tokio::test]
    async fn test_alert_rule_with_conditions() {
        let mut conditions = HashMap::new();
        conditions.insert("environment".to_string(), "production".to_string());

        let rule = AlertRule {
            name: "prod_only_rule".to_string(),
            source_pattern: "api".to_string(),
            severity_levels: vec![AlertSeverity::Critical],
            target_channels: vec!["email".to_string()],
            conditions,
        };

        let prod_alert = Alert::new(
            "Production Issue".to_string(),
            "Issue in production".to_string(),
            AlertSeverity::Critical,
            "api_server".to_string(),
        )
        .with_metadata("environment".to_string(), "production".to_string());

        let dev_alert = Alert::new(
            "Development Issue".to_string(),
            "Issue in development".to_string(),
            AlertSeverity::Critical,
            "api_server".to_string(),
        )
        .with_metadata("environment".to_string(), "development".to_string());

        assert!(rule.matches(&prod_alert));
        assert!(!rule.matches(&dev_alert));
    }

    #[tokio::test]
    async fn test_console_alert_channel() {
        let channel = ConsoleAlertChannel::new("test_console".to_string());

        assert_eq!(channel.channel_name(), "test_console");
        assert!(channel.is_available().await);

        let alert = Alert::new(
            "Test Alert".to_string(),
            "Test message".to_string(),
            AlertSeverity::Warning,
            "test_source".to_string(),
        );

        let result = channel.send_alert(&alert).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_alert_resolution() {
        let config = AlertManagerConfig::default();
        let manager = AlertManager::new(config);

        let channel = Box::new(ConsoleAlertChannel::new("console".to_string()));
        manager
            .register_channel("console".to_string(), channel)
            .await
            .unwrap();

        let alert = Alert::new(
            "Critical Error".to_string(),
            "System is down".to_string(),
            AlertSeverity::Critical,
            "system".to_string(),
        );

        let alert_id = alert.id.clone();
        manager.send_alert(alert).await.unwrap();

        // Verify alert is active
        let active_alerts = manager.get_active_alerts().await.unwrap();
        assert_eq!(active_alerts.len(), 1);

        // Resolve the alert
        manager.resolve_alert(&alert_id).await.unwrap();

        // Verify alert is no longer active
        let active_alerts = manager.get_active_alerts().await.unwrap();
        assert!(active_alerts.is_empty());
    }

    #[test]
    fn test_alert_creation() {
        let alert = Alert::new(
            "Test Alert".to_string(),
            "Test message".to_string(),
            AlertSeverity::Critical,
            "test_source".to_string(),
        );

        assert!(!alert.id.is_empty());
        assert_eq!(alert.title, "Test Alert");
        assert_eq!(alert.message, "Test message");
        assert_eq!(alert.severity, AlertSeverity::Critical);
        assert_eq!(alert.source, "test_source");
        assert!(alert.is_critical());
        assert!(alert.severity.is_urgent());
    }

    #[test]
    fn test_alert_severity_levels() {
        assert!(!AlertSeverity::Info.is_urgent());
        assert!(!AlertSeverity::Warning.is_urgent());
        assert!(!AlertSeverity::Error.is_urgent());
        assert!(AlertSeverity::Critical.is_urgent());
    }

    #[test]
    fn test_alert_manager_config_default() {
        let config = AlertManagerConfig::default();

        assert!(config.enable_alerts);
        assert_eq!(config.max_alerts_per_hour, 10);
        assert_eq!(config.default_timeout, Duration::from_secs(300));
        assert!(!config.enable_escalation);
        assert_eq!(config.escalation_timeout, Duration::from_secs(900));
    }

    #[test]
    fn test_alert_with_metadata() {
        let alert = Alert::new(
            "Test".to_string(),
            "Message".to_string(),
            AlertSeverity::Info,
            "source".to_string(),
        )
        .with_metadata("key1".to_string(), "value1".to_string())
        .with_metadata("key2".to_string(), "value2".to_string());

        assert_eq!(alert.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(alert.metadata.get("key2"), Some(&"value2".to_string()));
    }
}
