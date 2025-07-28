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

// ABOUTME: Pattern recognition types for API and CLI interaction tracking
//! # Pattern Recognition Types
//!
//! This module defines the core types for tracking and analyzing API and CLI
//! interaction patterns for learning and behavioral analysis.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// API interaction pattern data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiInteractionPattern {
    /// API endpoint path
    pub endpoint: String,

    /// HTTP method
    pub method: String,

    /// Request frequency per time window
    pub frequency: u32,

    /// Response time patterns (average, percentiles)
    pub response_times: ResponseTimePattern,

    /// Success rate (2xx responses)
    pub success_rate: f64,

    /// Error patterns and frequencies
    pub error_patterns: HashMap<String, u32>,

    /// User ID or session identifier
    pub user_identifier: String,

    /// Timestamp of interaction
    pub timestamp: DateTime<Utc>,

    /// Request parameters patterns
    pub parameter_patterns: Vec<String>,
}

/// CLI interaction pattern data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliInteractionPattern {
    /// Command executed
    pub command: String,

    /// Arguments used
    pub arguments: Vec<String>,

    /// Execution frequency
    pub frequency: u32,

    /// Success rate
    pub success_rate: f64,

    /// Average execution time
    pub avg_execution_time_ms: u64,

    /// User identifier
    pub user_identifier: String,

    /// Timestamp of interaction
    pub timestamp: DateTime<Utc>,

    /// Exit code patterns
    pub exit_codes: HashMap<i32, u32>,
}

/// Response time pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimePattern {
    /// Average response time in milliseconds
    pub average_ms: f64,

    /// 50th percentile response time
    pub p50_ms: f64,

    /// 95th percentile response time
    pub p95_ms: f64,

    /// 99th percentile response time
    pub p99_ms: f64,

    /// Maximum response time observed
    pub max_ms: f64,
}

/// Request frequency analysis for time-based patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestFrequencyAnalysis {
    /// Requests per hour breakdown
    pub hourly_patterns: HashMap<u32, u32>, // hour (0-23) -> count

    /// Requests per day of week
    pub daily_patterns: HashMap<u32, u32>, // day (0-6, 0=Sunday) -> count

    /// Peak usage hours
    pub peak_hours: Vec<u32>,

    /// Busiest day of week
    pub busiest_day: u32,

    /// Total requests in analysis period
    pub total_requests: u32,

    /// Analysis time window in days
    pub analysis_window_days: u32,
}

/// Usage trends analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageTrendsAnalysis {
    /// Overall trend direction (-1.0 to 1.0)
    pub trend_direction: f64,

    /// Weekly growth rate
    pub weekly_growth_rate: f64,

    /// Monthly growth rate
    pub monthly_growth_rate: f64,

    /// Seasonal patterns detected
    pub seasonal_patterns: Vec<SeasonalPattern>,

    /// Anomaly detection results
    pub anomalies: Vec<UsageAnomaly>,
}

/// Seasonal usage pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalPattern {
    /// Pattern type (daily, weekly, monthly)
    pub pattern_type: String,

    /// Pattern strength (0.0-1.0)
    pub strength: f64,

    /// Description of the pattern
    pub description: String,
}

/// Usage anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAnomaly {
    /// Timestamp of anomaly
    pub timestamp: DateTime<Utc>,

    /// Anomaly type (spike, drop, pattern_break)
    pub anomaly_type: String,

    /// Severity score (0.0-1.0)
    pub severity: f64,

    /// Description of the anomaly
    pub description: String,

    /// Affected patterns or metrics
    pub affected_metrics: Vec<String>,
}
