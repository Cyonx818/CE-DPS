// ABOUTME: Pattern tracking for MCP server tool calls
//! # MCP Pattern Tracking
//!
//! This module provides pattern tracking capabilities for MCP server tool calls,
//! capturing CLI-like interaction patterns for learning and behavioral analysis.

use crate::auth::Claims;
use chrono::Utc;
use fortitude_types::CliInteractionPattern;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, instrument};

/// Pattern tracking configuration for MCP server
#[derive(Debug, Clone)]
pub struct McpPatternTrackingConfig {
    /// Whether to track patterns
    pub enabled: bool,

    /// Buffer size for pattern queue
    pub buffer_size: usize,

    /// Minimum execution time to track (ms)
    pub min_execution_time_ms: u64,

    /// Track only authenticated requests
    pub track_authenticated_only: bool,

    /// Track tool arguments patterns
    pub track_arguments: bool,
}

impl Default for McpPatternTrackingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_size: 1000,
            min_execution_time_ms: 10,
            track_authenticated_only: false,
            track_arguments: true,
        }
    }
}

/// MCP pattern tracker service
#[derive(Debug, Clone)]
pub struct McpPatternTracker {
    /// Configuration
    config: Arc<McpPatternTrackingConfig>,

    /// Channel sender for patterns
    pattern_sender: mpsc::UnboundedSender<CliInteractionPattern>,
}

impl McpPatternTracker {
    /// Create a new MCP pattern tracker
    pub fn new(
        config: McpPatternTrackingConfig,
    ) -> (Self, mpsc::UnboundedReceiver<CliInteractionPattern>) {
        let (sender, receiver) = mpsc::unbounded_channel();

        let tracker = Self {
            config: Arc::new(config),
            pattern_sender: sender,
        };

        (tracker, receiver)
    }

    /// Track an MCP tool call pattern
    #[instrument(skip(self, pattern))]
    pub async fn track_pattern(&self, pattern: CliInteractionPattern) {
        if !self.config.enabled {
            return;
        }

        if pattern.avg_execution_time_ms < self.config.min_execution_time_ms {
            debug!(
                "Skipping pattern with execution time below threshold: {}ms",
                pattern.avg_execution_time_ms
            );
            return;
        }

        if let Err(e) = self.pattern_sender.send(pattern) {
            error!("Failed to send pattern to tracking queue: {}", e);
        }
    }

    /// Track a tool call execution
    #[instrument(skip(self))]
    pub async fn track_tool_call(
        &self,
        tool_name: &str,
        arguments: Option<&serde_json::Map<String, serde_json::Value>>,
        claims: Option<&Claims>,
        execution_time: std::time::Duration,
        success: bool,
        exit_code: i32,
    ) {
        if !self.config.enabled {
            return;
        }

        // Get user identifier
        let user_identifier = if let Some(claims) = claims {
            claims.sub.clone()
        } else {
            "mcp_client".to_string()
        };

        // Skip tracking if configured to only track authenticated requests and user is anonymous
        if self.config.track_authenticated_only && claims.is_none() {
            return;
        }

        // Extract arguments
        let mut args = Vec::new();
        if self.config.track_arguments {
            if let Some(arguments) = arguments {
                for (key, value) in arguments {
                    // Create argument pattern like "key:type" to avoid storing sensitive data
                    let arg_pattern = format!("{}:{}", key, value_type_name(value));
                    args.push(arg_pattern);
                }
            }
        }

        // Build exit code patterns
        let mut exit_codes = HashMap::new();
        exit_codes.insert(exit_code, 1);

        let pattern = CliInteractionPattern {
            command: tool_name.to_string(),
            arguments: args,
            frequency: 1,
            success_rate: if success { 1.0 } else { 0.0 },
            avg_execution_time_ms: execution_time.as_millis() as u64,
            user_identifier,
            timestamp: Utc::now(),
            exit_codes,
        };

        self.track_pattern(pattern).await;
    }
}

/// Get the type name of a JSON value for pattern tracking
fn value_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

/// Pattern aggregation service for MCP tool calls
#[derive(Debug)]
pub struct McpPatternAggregationService {
    /// Configuration
    #[allow(dead_code)]
    config: Arc<McpPatternTrackingConfig>,

    /// Pattern receiver
    pattern_receiver: mpsc::UnboundedReceiver<CliInteractionPattern>,

    /// Aggregated patterns by tool
    aggregated_patterns: HashMap<String, AggregatedMcpPattern>,
}

/// Aggregated pattern data for MCP tools
#[derive(Debug)]
struct AggregatedMcpPattern {
    /// Tool name
    command: String,

    /// Argument patterns by frequency
    argument_patterns: HashMap<String, u32>,

    /// Total frequency
    frequency: u32,

    /// Execution times for analysis
    execution_times: Vec<u64>,

    /// Success count
    success_count: u32,

    /// Exit code patterns
    exit_code_patterns: HashMap<i32, u32>,

    /// User identifiers
    user_identifiers: HashMap<String, u32>,

    /// First seen timestamp
    #[allow(dead_code)]
    first_seen: chrono::DateTime<Utc>,

    /// Last seen timestamp
    last_seen: chrono::DateTime<Utc>,
}

impl McpPatternAggregationService {
    /// Create new aggregation service
    pub fn new(
        config: McpPatternTrackingConfig,
        pattern_receiver: mpsc::UnboundedReceiver<CliInteractionPattern>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            pattern_receiver,
            aggregated_patterns: HashMap::new(),
        }
    }

    /// Run the aggregation service
    #[instrument(skip(self))]
    pub async fn run(&mut self) {
        debug!("Starting MCP pattern aggregation service");

        while let Some(pattern) = self.pattern_receiver.recv().await {
            self.aggregate_pattern(pattern).await;
        }

        debug!("MCP pattern aggregation service stopped");
    }

    /// Aggregate a single pattern
    async fn aggregate_pattern(&mut self, pattern: CliInteractionPattern) {
        let key = pattern.command.clone();

        match self.aggregated_patterns.get_mut(&key) {
            Some(aggregated) => {
                // Update existing pattern
                aggregated.frequency += pattern.frequency;
                aggregated
                    .execution_times
                    .push(pattern.avg_execution_time_ms);

                if pattern.success_rate > 0.0 {
                    aggregated.success_count += 1;
                }

                // Merge exit code patterns
                for (exit_code, count) in pattern.exit_codes {
                    *aggregated.exit_code_patterns.entry(exit_code).or_insert(0) += count;
                }

                // Track user
                *aggregated
                    .user_identifiers
                    .entry(pattern.user_identifier)
                    .or_insert(0) += 1;

                // Track argument patterns
                for arg in pattern.arguments {
                    *aggregated.argument_patterns.entry(arg).or_insert(0) += 1;
                }

                aggregated.last_seen = pattern.timestamp;
            }
            None => {
                // Create new aggregated pattern
                let mut user_identifiers = HashMap::new();
                user_identifiers.insert(pattern.user_identifier.clone(), 1);

                let mut argument_patterns = HashMap::new();
                for arg in &pattern.arguments {
                    argument_patterns.insert(arg.clone(), 1);
                }

                let aggregated = AggregatedMcpPattern {
                    command: pattern.command.clone(),
                    argument_patterns,
                    frequency: pattern.frequency,
                    execution_times: vec![pattern.avg_execution_time_ms],
                    success_count: if pattern.success_rate > 0.0 { 1 } else { 0 },
                    exit_code_patterns: pattern.exit_codes.clone(),
                    user_identifiers,
                    first_seen: pattern.timestamp,
                    last_seen: pattern.timestamp,
                };

                self.aggregated_patterns.insert(key, aggregated);
            }
        }
    }

    /// Get aggregated patterns for analysis
    pub fn get_aggregated_patterns(&self) -> Vec<CliInteractionPattern> {
        self.aggregated_patterns
            .values()
            .map(|agg| self.convert_to_cli_pattern(agg))
            .collect()
    }

    /// Convert aggregated pattern back to CLI interaction pattern
    fn convert_to_cli_pattern(&self, agg: &AggregatedMcpPattern) -> CliInteractionPattern {
        // Calculate average execution time
        let avg_execution_time_ms = if agg.execution_times.is_empty() {
            0
        } else {
            agg.execution_times.iter().sum::<u64>() / agg.execution_times.len() as u64
        };

        let success_rate = if agg.frequency > 0 {
            agg.success_count as f64 / agg.frequency as f64
        } else {
            0.0
        };

        // Get most common user identifier
        let user_identifier = agg
            .user_identifiers
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(user, _)| user.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Get common argument patterns (at least 25% frequency)
        let arguments: Vec<String> = agg
            .argument_patterns
            .iter()
            .filter(|(_, count)| **count > agg.frequency / 4)
            .map(|(arg, _)| arg.clone())
            .collect();

        CliInteractionPattern {
            command: agg.command.clone(),
            arguments,
            frequency: agg.frequency,
            success_rate,
            avg_execution_time_ms,
            user_identifier,
            timestamp: agg.last_seen,
            exit_codes: agg.exit_code_patterns.clone(),
        }
    }

    /// Clear aggregated patterns (for periodic flush)
    pub fn clear_patterns(&mut self) {
        self.aggregated_patterns.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mcp_pattern_tracker_creation() {
        let config = McpPatternTrackingConfig::default();
        let (tracker, _receiver) = McpPatternTracker::new(config.clone());

        assert!(tracker.config.enabled);
        assert_eq!(tracker.config.buffer_size, 1000);
    }

    #[test]
    fn test_value_type_name() {
        assert_eq!(value_type_name(&json!(null)), "null");
        assert_eq!(value_type_name(&json!(true)), "bool");
        assert_eq!(value_type_name(&json!(42)), "number");
        assert_eq!(value_type_name(&json!("hello")), "string");
        assert_eq!(value_type_name(&json!([])), "array");
        assert_eq!(value_type_name(&json!({})), "object");
    }

    #[tokio::test]
    async fn test_tool_call_tracking() {
        let config = McpPatternTrackingConfig::default();
        let (tracker, mut receiver) = McpPatternTracker::new(config);

        let arguments = json!({
            "query": "test query",
            "type": "research"
        });

        let arguments_map = arguments.as_object().unwrap();

        tracker
            .track_tool_call(
                "research_query",
                Some(arguments_map),
                None,
                std::time::Duration::from_millis(150),
                true,
                0,
            )
            .await;

        // Check that pattern was sent
        let pattern = receiver.recv().await.unwrap();
        assert_eq!(pattern.command, "research_query");
        assert_eq!(pattern.frequency, 1);
        assert_eq!(pattern.success_rate, 1.0);
        assert_eq!(pattern.avg_execution_time_ms, 150);
        assert!(pattern.arguments.contains(&"query:string".to_string()));
        assert!(pattern.arguments.contains(&"type:string".to_string()));
    }

    #[tokio::test]
    async fn test_pattern_aggregation() {
        let config = McpPatternTrackingConfig::default();
        let (_sender, receiver) = mpsc::unbounded_channel();
        let mut service = McpPatternAggregationService::new(config, receiver);

        let pattern = CliInteractionPattern {
            command: "research_query".to_string(),
            arguments: vec!["query:string".to_string(), "type:string".to_string()],
            frequency: 1,
            success_rate: 1.0,
            avg_execution_time_ms: 150,
            user_identifier: "user123".to_string(),
            timestamp: Utc::now(),
            exit_codes: std::iter::once((0, 1)).collect(),
        };

        service.aggregate_pattern(pattern).await;

        let aggregated = service.get_aggregated_patterns();
        assert_eq!(aggregated.len(), 1);
        assert_eq!(aggregated[0].command, "research_query");
        assert_eq!(aggregated[0].frequency, 1);
        assert_eq!(aggregated[0].success_rate, 1.0);
    }
}
