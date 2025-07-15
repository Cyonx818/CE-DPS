// ABOUTME: Middleware for tracking API interaction patterns for learning system
//! # Pattern Tracking Middleware
//!
//! This middleware captures API interaction patterns in real-time to feed into
//! the learning system for usage pattern analysis and behavioral insights.

use crate::middleware::auth::Claims;
use axum::{
    extract::{Extension, Request},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use fortitude_types::{ApiInteractionPattern, ResponseTimePattern};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{debug, error, instrument};

/// Pattern tracking configuration
#[derive(Debug, Clone)]
pub struct PatternTrackingConfig {
    /// Whether to track patterns
    pub enabled: bool,

    /// Buffer size for pattern queue
    pub buffer_size: usize,

    /// Minimum response time to track (ms)
    pub min_response_time_ms: u64,

    /// Track only authenticated requests
    pub track_authenticated_only: bool,
}

impl Default for PatternTrackingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_size: 1000,
            min_response_time_ms: 10,
            track_authenticated_only: false,
        }
    }
}

/// Pattern tracker service
#[derive(Debug, Clone)]
pub struct PatternTracker {
    /// Configuration
    config: Arc<PatternTrackingConfig>,

    /// Channel sender for patterns
    pattern_sender: mpsc::UnboundedSender<ApiInteractionPattern>,
}

impl PatternTracker {
    /// Create a new pattern tracker
    pub fn new(
        config: PatternTrackingConfig,
    ) -> (Self, mpsc::UnboundedReceiver<ApiInteractionPattern>) {
        let (sender, receiver) = mpsc::unbounded_channel();

        let tracker = Self {
            config: Arc::new(config),
            pattern_sender: sender,
        };

        (tracker, receiver)
    }

    /// Track an API interaction pattern
    #[instrument(skip(self, pattern))]
    pub async fn track_pattern(&self, pattern: ApiInteractionPattern) {
        if !self.config.enabled {
            return;
        }

        if pattern.response_times.average_ms < self.config.min_response_time_ms as f64 {
            debug!(
                "Skipping pattern with response time below threshold: {}ms",
                pattern.response_times.average_ms
            );
            return;
        }

        if let Err(e) = self.pattern_sender.send(pattern) {
            error!("Failed to send pattern to tracking queue: {}", e);
        }
    }
}

/// Middleware for tracking API interaction patterns
#[instrument(skip(req, next, tracker))]
pub async fn pattern_tracking_middleware(
    Extension(tracker): Extension<PatternTracker>,
    claims: Option<Extension<Claims>>,
    req: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();

    // Extract request information
    let method = req.method().to_string();
    let uri = req.uri().path().to_string();
    let headers = req.headers().clone();

    // Get user identifier
    let user_identifier = if let Some(Extension(claims)) = claims.as_ref() {
        claims.sub.clone()
    } else {
        // Use IP address or anonymous identifier
        extract_client_ip(&headers).unwrap_or_else(|| "anonymous".to_string())
    };

    // Skip tracking if configured to only track authenticated requests and user is anonymous
    if tracker.config.track_authenticated_only && user_identifier == "anonymous" {
        return next.run(req).await;
    }

    // Process request
    let response = next.run(req).await;

    // Calculate response time
    let response_time_ms = start_time.elapsed().as_millis() as f64;

    // Extract response information
    let status_code = response.status();
    let success_rate = if status_code.is_success() { 1.0 } else { 0.0 };

    // Build error patterns
    let mut error_patterns = HashMap::new();
    if !status_code.is_success() {
        let error_code = status_code.as_u16().to_string();
        error_patterns.insert(error_code, 1);
    }

    // Create API interaction pattern
    let pattern = ApiInteractionPattern {
        endpoint: uri,
        method,
        frequency: 1,
        response_times: ResponseTimePattern {
            average_ms: response_time_ms,
            p50_ms: response_time_ms,
            p95_ms: response_time_ms,
            p99_ms: response_time_ms,
            max_ms: response_time_ms,
        },
        success_rate,
        error_patterns,
        user_identifier,
        timestamp: Utc::now(),
        parameter_patterns: extract_parameter_patterns(&headers),
    };

    // Track the pattern asynchronously
    tracker.track_pattern(pattern).await;

    response
}

/// Extract client IP address from headers
fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    // Try X-Forwarded-For first
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take the first IP in the chain
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return Some(first_ip.trim().to_string());
            }
        }
    }

    // Try X-Real-IP
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return Some(ip_str.to_string());
        }
    }

    None
}

/// Extract parameter patterns from request headers
fn extract_parameter_patterns(headers: &HeaderMap) -> Vec<String> {
    let mut patterns = Vec::new();

    // Extract content type
    if let Some(content_type) = headers.get("content-type") {
        if let Ok(ct_str) = content_type.to_str() {
            patterns.push(format!("content-type:{ct_str}"));
        }
    }

    // Extract user agent for API client identification
    if let Some(user_agent) = headers.get("user-agent") {
        if let Ok(ua_str) = user_agent.to_str() {
            // Extract just the primary client identifier
            let client = ua_str.split('/').next().unwrap_or("unknown");
            patterns.push(format!("client:{client}"));
        }
    }

    // Extract accept header patterns
    if let Some(accept) = headers.get("accept") {
        if let Ok(accept_str) = accept.to_str() {
            patterns.push(format!("accept:{accept_str}"));
        }
    }

    patterns
}

/// Pattern aggregation service for batching and processing patterns
#[derive(Debug)]
pub struct PatternAggregationService {
    /// Configuration
    #[allow(dead_code)]
    config: Arc<PatternTrackingConfig>,

    /// Pattern receiver
    pattern_receiver: mpsc::UnboundedReceiver<ApiInteractionPattern>,

    /// Aggregated patterns by endpoint
    aggregated_patterns: HashMap<String, AggregatedPattern>,
}

/// Aggregated pattern data for efficiency
#[derive(Debug)]
struct AggregatedPattern {
    /// Endpoint path
    endpoint: String,

    /// HTTP method
    method: String,

    /// Total frequency
    frequency: u32,

    /// Response times for percentile calculation
    response_times: Vec<f64>,

    /// Success count
    success_count: u32,

    /// Error patterns
    error_patterns: HashMap<String, u32>,

    /// User identifiers
    user_identifiers: HashMap<String, u32>,

    /// Parameter patterns
    parameter_patterns: HashMap<String, u32>,

    /// First seen timestamp
    #[allow(dead_code)]
    first_seen: chrono::DateTime<Utc>,

    /// Last seen timestamp
    last_seen: chrono::DateTime<Utc>,
}

impl PatternAggregationService {
    /// Create new aggregation service
    pub fn new(
        config: PatternTrackingConfig,
        pattern_receiver: mpsc::UnboundedReceiver<ApiInteractionPattern>,
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
        debug!("Starting pattern aggregation service");

        while let Some(pattern) = self.pattern_receiver.recv().await {
            self.aggregate_pattern(pattern).await;
        }

        debug!("Pattern aggregation service stopped");
    }

    /// Aggregate a single pattern
    async fn aggregate_pattern(&mut self, pattern: ApiInteractionPattern) {
        let key = format!("{}:{}", pattern.method, pattern.endpoint);

        match self.aggregated_patterns.get_mut(&key) {
            Some(aggregated) => {
                // Update existing pattern
                aggregated.frequency += pattern.frequency;
                aggregated
                    .response_times
                    .push(pattern.response_times.average_ms);

                if pattern.success_rate > 0.0 {
                    aggregated.success_count += 1;
                }

                // Merge error patterns
                for (error_type, count) in pattern.error_patterns {
                    *aggregated.error_patterns.entry(error_type).or_insert(0) += count;
                }

                // Track user
                *aggregated
                    .user_identifiers
                    .entry(pattern.user_identifier)
                    .or_insert(0) += 1;

                // Track parameters
                for param in pattern.parameter_patterns {
                    *aggregated.parameter_patterns.entry(param).or_insert(0) += 1;
                }

                aggregated.last_seen = pattern.timestamp;
            }
            None => {
                // Create new aggregated pattern
                let mut user_identifiers = HashMap::new();
                user_identifiers.insert(pattern.user_identifier.clone(), 1);

                let mut parameter_patterns = HashMap::new();
                for param in &pattern.parameter_patterns {
                    parameter_patterns.insert(param.clone(), 1);
                }

                let aggregated = AggregatedPattern {
                    endpoint: pattern.endpoint.clone(),
                    method: pattern.method.clone(),
                    frequency: pattern.frequency,
                    response_times: vec![pattern.response_times.average_ms],
                    success_count: if pattern.success_rate > 0.0 { 1 } else { 0 },
                    error_patterns: pattern.error_patterns.clone(),
                    user_identifiers,
                    parameter_patterns,
                    first_seen: pattern.timestamp,
                    last_seen: pattern.timestamp,
                };

                self.aggregated_patterns.insert(key, aggregated);
            }
        }
    }

    /// Get aggregated patterns for analysis
    pub fn get_aggregated_patterns(&self) -> Vec<ApiInteractionPattern> {
        self.aggregated_patterns
            .values()
            .map(|agg| self.convert_to_api_pattern(agg))
            .collect()
    }

    /// Convert aggregated pattern back to API interaction pattern
    fn convert_to_api_pattern(&self, agg: &AggregatedPattern) -> ApiInteractionPattern {
        // Calculate response time percentiles
        let mut sorted_times = agg.response_times.clone();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let len = sorted_times.len();
        let average_ms = sorted_times.iter().sum::<f64>() / len as f64;
        let p50_ms = sorted_times[len / 2];
        let p95_ms = sorted_times[(len as f64 * 0.95) as usize];
        let p99_ms = sorted_times[(len as f64 * 0.99) as usize];
        let max_ms = sorted_times[len - 1];

        let success_rate = agg.success_count as f64 / agg.frequency as f64;

        // Get most common user identifier
        let user_identifier = agg
            .user_identifiers
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(user, _)| user.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Get common parameter patterns
        let parameter_patterns: Vec<String> = agg
            .parameter_patterns
            .iter()
            .filter(|(_, count)| **count > agg.frequency / 4) // At least 25% frequency
            .map(|(param, _)| param.clone())
            .collect();

        ApiInteractionPattern {
            endpoint: agg.endpoint.clone(),
            method: agg.method.clone(),
            frequency: agg.frequency,
            response_times: ResponseTimePattern {
                average_ms,
                p50_ms,
                p95_ms,
                p99_ms,
                max_ms,
            },
            success_rate,
            error_patterns: agg.error_patterns.clone(),
            user_identifier,
            timestamp: agg.last_seen,
            parameter_patterns,
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
    // Test imports available for future use\n    // use axum::http::{Method, Uri};

    #[test]
    fn test_pattern_tracker_creation() {
        let config = PatternTrackingConfig::default();
        let (tracker, _receiver) = PatternTracker::new(config.clone());

        assert!(tracker.config.enabled);
        assert_eq!(tracker.config.buffer_size, 1000);
    }

    #[test]
    fn test_extract_client_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "192.168.1.1, 10.0.0.1".parse().unwrap());

        let ip = extract_client_ip(&headers);
        assert_eq!(ip, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_extract_parameter_patterns() {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/json".parse().unwrap());
        headers.insert("user-agent", "curl/7.68.0".parse().unwrap());
        headers.insert("accept", "application/json".parse().unwrap());

        let patterns = extract_parameter_patterns(&headers);

        assert!(patterns.contains(&"content-type:application/json".to_string()));
        assert!(patterns.contains(&"client:curl".to_string()));
        assert!(patterns.contains(&"accept:application/json".to_string()));
    }

    #[tokio::test]
    async fn test_pattern_aggregation() {
        let config = PatternTrackingConfig::default();
        let (_sender, receiver) = mpsc::unbounded_channel();
        let mut service = PatternAggregationService::new(config, receiver);

        let pattern = ApiInteractionPattern {
            endpoint: "/api/v1/research".to_string(),
            method: "POST".to_string(),
            frequency: 1,
            response_times: ResponseTimePattern {
                average_ms: 150.0,
                p50_ms: 150.0,
                p95_ms: 150.0,
                p99_ms: 150.0,
                max_ms: 150.0,
            },
            success_rate: 1.0,
            error_patterns: HashMap::new(),
            user_identifier: "user123".to_string(),
            timestamp: Utc::now(),
            parameter_patterns: vec!["content-type:application/json".to_string()],
        };

        service.aggregate_pattern(pattern).await;

        let aggregated = service.get_aggregated_patterns();
        assert_eq!(aggregated.len(), 1);
        assert_eq!(aggregated[0].endpoint, "/api/v1/research");
        assert_eq!(aggregated[0].method, "POST");
        assert_eq!(aggregated[0].frequency, 1);
    }
}
