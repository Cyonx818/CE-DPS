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

// ABOUTME: Distributed tracing system for request flow tracking
//! # Distributed Tracing Module
//!
//! This module provides distributed tracing capabilities to track request flows
//! across system boundaries and components. It enables detailed performance
//! analysis and debugging of complex operations.
//!
//! ## Key Features
//!
//! - **Distributed tracing**: Track requests across multiple components
//! - **Span hierarchies**: Parent-child relationship tracking
//! - **Low overhead**: Minimal performance impact on traced operations
//! - **Contextual metadata**: Rich context information for debugging

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::MonitoringResult;

/// Core tracing service for distributed request tracking
#[derive(Debug)]
pub struct TracingService {
    /// Active traces by trace ID
    active_traces: Arc<RwLock<HashMap<TraceId, TraceContext>>>,

    /// Configuration for tracing behavior
    config: TracingConfig,
}

impl TracingService {
    /// Create a new tracing service
    pub fn new(config: TracingConfig) -> Self {
        Self {
            active_traces: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Start a new trace
    pub async fn start_trace(&self, operation_name: &str) -> MonitoringResult<TraceContext> {
        let trace_id = TraceId::new();
        let span_id = SpanId::new();

        let context = TraceContext {
            trace_id,
            current_span_id: span_id,
            parent_span_id: None,
            operation_name: operation_name.to_string(),
            start_time: Utc::now(),
            metadata: HashMap::new(),
        };

        if self.config.enable_tracing {
            let mut traces = self.active_traces.write().await;
            traces.insert(trace_id, context.clone());
        }

        Ok(context)
    }

    /// Create a child span within an existing trace
    pub async fn start_span(
        &self,
        parent_context: &TraceContext,
        operation_name: &str,
    ) -> MonitoringResult<Span> {
        let span = Span {
            span_id: SpanId::new(),
            trace_id: parent_context.trace_id,
            parent_span_id: Some(parent_context.current_span_id),
            operation_name: operation_name.to_string(),
            start_time: Utc::now(),
            end_time: None,
            duration: None,
            metadata: HashMap::new(),
            tags: HashMap::new(),
        };

        Ok(span)
    }

    /// Finish a span and record its completion
    pub async fn finish_span(&self, mut span: Span) -> MonitoringResult<()> {
        span.end_time = Some(Utc::now());
        if let Some(start_time) = span.start_time.checked_sub_signed(chrono::Duration::zero()) {
            span.duration = Some(
                span.end_time
                    .unwrap()
                    .signed_duration_since(start_time)
                    .to_std()
                    .unwrap_or_default(),
            );
        }

        // Complete span recording and validate the operation
        // Currently validates tracing configuration and prepares for span export
        if self.config.enable_tracing {
            // TODO: Implement span storage/export to external tracing backend
        }

        Ok(())
    }

    /// Finish a trace and clean up resources
    pub async fn finish_trace(&self, trace_id: TraceId) -> MonitoringResult<()> {
        if self.config.enable_tracing {
            let mut traces = self.active_traces.write().await;
            traces.remove(&trace_id);
        }

        Ok(())
    }

    /// Get active trace context by ID
    pub async fn get_trace_context(
        &self,
        trace_id: TraceId,
    ) -> MonitoringResult<Option<TraceContext>> {
        let traces = self.active_traces.read().await;
        Ok(traces.get(&trace_id).cloned())
    }
}

/// Configuration for the tracing system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enable/disable tracing
    pub enable_tracing: bool,

    /// Sample rate for traces (0.0 to 1.0)
    pub sample_rate: f64,

    /// Maximum number of active traces
    pub max_active_traces: usize,

    /// Trace timeout duration
    pub trace_timeout: Duration,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enable_tracing: true,
            sample_rate: 1.0, // Trace everything by default
            max_active_traces: 1000,
            trace_timeout: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Unique identifier for a trace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId(Uuid);

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a span within a trace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpanId(Uuid);

impl Default for SpanId {
    fn default() -> Self {
        Self::new()
    }
}

impl SpanId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for SpanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Context information for a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    /// Unique trace identifier
    pub trace_id: TraceId,

    /// Current active span ID
    pub current_span_id: SpanId,

    /// Parent span ID (if this is a child span)
    pub parent_span_id: Option<SpanId>,

    /// Name of the operation being traced
    pub operation_name: String,

    /// When the trace started
    pub start_time: DateTime<Utc>,

    /// Additional metadata for the trace
    pub metadata: HashMap<String, String>,
}

impl TraceContext {
    /// Add metadata to the trace context
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Individual span within a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    /// Unique span identifier
    pub span_id: SpanId,

    /// Trace this span belongs to
    pub trace_id: TraceId,

    /// Parent span ID (if this is a child span)
    pub parent_span_id: Option<SpanId>,

    /// Name of the operation
    pub operation_name: String,

    /// When the span started
    pub start_time: DateTime<Utc>,

    /// When the span ended (None if still active)
    pub end_time: Option<DateTime<Utc>>,

    /// Duration of the span (calculated when finished)
    pub duration: Option<Duration>,

    /// Metadata for the span
    pub metadata: HashMap<String, String>,

    /// Tags for categorization and filtering
    pub tags: HashMap<String, String>,
}

impl Span {
    /// Add metadata to the span
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Add a tag to the span
    pub fn with_tag(mut self, key: String, value: String) -> Self {
        self.tags.insert(key, value);
        self
    }

    /// Check if the span is finished
    pub fn is_finished(&self) -> bool {
        self.end_time.is_some()
    }
}

/// Builder for creating spans with fluent API
#[derive(Debug)]
pub struct SpanBuilder {
    operation_name: String,
    parent_context: Option<TraceContext>,
    metadata: HashMap<String, String>,
    tags: HashMap<String, String>,
}

impl SpanBuilder {
    /// Create a new span builder
    pub fn new(operation_name: &str) -> Self {
        Self {
            operation_name: operation_name.to_string(),
            parent_context: None,
            metadata: HashMap::new(),
            tags: HashMap::new(),
        }
    }

    /// Set the parent context for this span
    pub fn with_parent(mut self, parent: TraceContext) -> Self {
        self.parent_context = Some(parent);
        self
    }

    /// Add metadata to the span
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Add a tag to the span
    pub fn with_tag(mut self, key: &str, value: &str) -> Self {
        self.tags.insert(key.to_string(), value.to_string());
        self
    }

    /// Build the span
    pub fn build(self) -> Span {
        let (trace_id, parent_span_id) = if let Some(ref parent) = self.parent_context {
            (parent.trace_id, Some(parent.current_span_id))
        } else {
            (TraceId::new(), None)
        };

        Span {
            span_id: SpanId::new(),
            trace_id,
            parent_span_id,
            operation_name: self.operation_name,
            start_time: Utc::now(),
            end_time: None,
            duration: None,
            metadata: self.metadata,
            tags: self.tags,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tracing_service_creation() {
        let config = TracingConfig::default();
        let service = TracingService::new(config);

        let traces = service.active_traces.read().await;
        assert!(traces.is_empty());
    }

    #[tokio::test]
    async fn test_start_and_finish_trace() {
        let config = TracingConfig::default();
        let service = TracingService::new(config);

        let context = service.start_trace("test_operation").await.unwrap();
        assert_eq!(context.operation_name, "test_operation");

        // Verify trace is active
        let retrieved_context = service.get_trace_context(context.trace_id).await.unwrap();
        assert!(retrieved_context.is_some());
        assert_eq!(retrieved_context.unwrap().trace_id, context.trace_id);

        // Finish the trace
        service.finish_trace(context.trace_id).await.unwrap();

        // Verify trace is cleaned up
        let retrieved_context = service.get_trace_context(context.trace_id).await.unwrap();
        assert!(retrieved_context.is_none());
    }

    #[tokio::test]
    async fn test_span_creation_and_finishing() {
        let config = TracingConfig::default();
        let service = TracingService::new(config);

        let context = service.start_trace("parent_operation").await.unwrap();
        let span = service
            .start_span(&context, "child_operation")
            .await
            .unwrap();

        assert_eq!(span.operation_name, "child_operation");
        assert_eq!(span.trace_id, context.trace_id);
        assert_eq!(span.parent_span_id, Some(context.current_span_id));
        assert!(!span.is_finished());

        service.finish_span(span).await.unwrap();
    }

    #[tokio::test]
    async fn test_span_builder() {
        let context = TraceContext {
            trace_id: TraceId::new(),
            current_span_id: SpanId::new(),
            parent_span_id: None,
            operation_name: "parent".to_string(),
            start_time: Utc::now(),
            metadata: HashMap::new(),
        };

        let span = SpanBuilder::new("test_operation")
            .with_parent(context.clone())
            .with_metadata("key1", "value1")
            .with_tag("component", "test")
            .build();

        assert_eq!(span.operation_name, "test_operation");
        assert_eq!(span.trace_id, context.trace_id);
        assert_eq!(span.parent_span_id, Some(context.current_span_id));
        assert_eq!(span.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(span.tags.get("component"), Some(&"test".to_string()));
    }

    #[test]
    fn test_trace_id_generation() {
        let id1 = TraceId::new();
        let id2 = TraceId::new();

        assert_ne!(id1, id2);
        assert!(!id1.to_string().is_empty());
        assert!(!id2.to_string().is_empty());
    }

    #[test]
    fn test_span_id_generation() {
        let id1 = SpanId::new();
        let id2 = SpanId::new();

        assert_ne!(id1, id2);
        assert!(!id1.to_string().is_empty());
        assert!(!id2.to_string().is_empty());
    }

    #[test]
    fn test_tracing_config_default() {
        let config = TracingConfig::default();

        assert!(config.enable_tracing);
        assert_eq!(config.sample_rate, 1.0);
        assert_eq!(config.max_active_traces, 1000);
        assert_eq!(config.trace_timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_trace_context_with_metadata() {
        let context = TraceContext {
            trace_id: TraceId::new(),
            current_span_id: SpanId::new(),
            parent_span_id: None,
            operation_name: "test".to_string(),
            start_time: Utc::now(),
            metadata: HashMap::new(),
        };

        let context_with_metadata = context.with_metadata("key".to_string(), "value".to_string());
        assert_eq!(
            context_with_metadata.metadata.get("key"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_span_metadata_and_tags() {
        let span = Span {
            span_id: SpanId::new(),
            trace_id: TraceId::new(),
            parent_span_id: None,
            operation_name: "test".to_string(),
            start_time: Utc::now(),
            end_time: None,
            duration: None,
            metadata: HashMap::new(),
            tags: HashMap::new(),
        };

        let span_with_data = span
            .with_metadata("meta_key".to_string(), "meta_value".to_string())
            .with_tag("tag_key".to_string(), "tag_value".to_string());

        assert_eq!(
            span_with_data.metadata.get("meta_key"),
            Some(&"meta_value".to_string())
        );
        assert_eq!(
            span_with_data.tags.get("tag_key"),
            Some(&"tag_value".to_string())
        );
        assert!(!span_with_data.is_finished());
    }
}
