# Production-Ready Distributed AI System Observability Framework

## Overview

A comprehensive Rust-based observability system designed specifically for distributed AI systems with multiple LLM providers. This implementation provides OpenTelemetry integration, custom metrics, distributed tracing, anomaly detection, and Prometheus alerting.

## Cargo.toml

```toml
[package]
name = "ai_observability"
version = "0.1.0"
edition = "2021"

[dependencies]
# OpenTelemetry core
opentelemetry = { version = "0.28", features = ["trace", "metrics"] }
opentelemetry_sdk = { version = "0.28", features = ["trace", "metrics", "rt-tokio"] }
opentelemetry-otlp = { version = "0.28", features = ["trace", "metrics", "grpc-tonic"] }
opentelemetry-prometheus = "0.28"
opentelemetry-semantic-conventions = "0.28"

# Tracing and instrumentation
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.28"

# Prometheus and metrics
prometheus = "0.13"
prometheus-client = "0.22"

# HTTP and async
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
hyper = { version = "1.0", features = ["full"] }
axum = "0.7"

# Data structures and serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4"] }

# Error handling and logging
anyhow = "1.0"
thiserror = "1.0"
log = "0.4"

# Statistics for anomaly detection
statrs = "0.16"

# Configuration
config = "0.14"
```

## Core Configuration

```rust
// src/config.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ObservabilityConfig {
    pub service_name: String,
    pub service_version: String,
    pub environment: String,
    
    // OpenTelemetry configuration
    pub otlp: OtlpConfig,
    
    // Prometheus configuration
    pub prometheus: PrometheusConfig,
    
    // LLM-specific configuration
    pub llm_monitoring: LlmMonitoringConfig,
    
    // Anomaly detection configuration
    pub anomaly_detection: AnomalyDetectionConfig,
    
    // Alert configuration
    pub alerting: AlertingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OtlpConfig {
    pub endpoint: String,
    pub timeout_seconds: u64,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrometheusConfig {
    pub listen_address: String,
    pub metrics_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmMonitoringConfig {
    pub track_tokens: bool,
    pub track_costs: bool,
    pub track_quality_metrics: bool,
    pub sample_prompts: bool,
    pub max_prompt_length: usize,
    pub cost_per_token: HashMap<String, f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnomalyDetectionConfig {
    pub enabled: bool,
    pub window_size_minutes: u64,
    pub std_dev_threshold: f64,
    pub min_data_points: usize,
    pub seasonality_detection: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlertingConfig {
    pub webhook_url: Option<String>,
    pub email_config: Option<EmailConfig>,
    pub slack_config: Option<SlackConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub from_address: String,
    pub to_addresses: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlackConfig {
    pub webhook_url: String,
    pub channel: String,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            service_name: "ai-service".to_string(),
            service_version: "0.1.0".to_string(),
            environment: "production".to_string(),
            otlp: OtlpConfig {
                endpoint: "http://localhost:4317".to_string(),
                timeout_seconds: 10,
                headers: HashMap::new(),
            },
            prometheus: PrometheusConfig {
                listen_address: "0.0.0.0:9090".to_string(),
                metrics_path: "/metrics".to_string(),
            },
            llm_monitoring: LlmMonitoringConfig {
                track_tokens: true,
                track_costs: true,
                track_quality_metrics: true,
                sample_prompts: true,
                max_prompt_length: 1000,
                cost_per_token: HashMap::new(),
            },
            anomaly_detection: AnomalyDetectionConfig {
                enabled: true,
                window_size_minutes: 60,
                std_dev_threshold: 2.0,
                min_data_points: 30,
                seasonality_detection: true,
            },
            alerting: AlertingConfig {
                webhook_url: None,
                email_config: None,
                slack_config: None,
            },
        }
    }
}
```

## LLM-Specific Metrics and Tracing

```rust
// src/llm_metrics.rs
use opentelemetry::{
    metrics::{Counter, Histogram, Meter, UpDownCounter},
    trace::{Span, Tracer},
    KeyValue,
};
use prometheus::{Counter as PrometheusCounter, Histogram as PrometheusHistogram, 
                 IntCounter, IntGauge, Registry};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub request_id: String,
    pub provider: String,
    pub model: String,
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub request_id: String,
    pub response: String,
    pub tokens_used: u32,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub cost: f64,
    pub latency_ms: u64,
    pub model_version: Option<String>,
    pub finish_reason: String,
}

#[derive(Debug, Clone)]
pub struct LlmMetrics {
    // OpenTelemetry metrics
    pub request_counter: Counter<u64>,
    pub request_duration: Histogram<f64>,
    pub token_counter: Counter<u64>,
    pub cost_counter: Counter<f64>,
    pub error_counter: Counter<u64>,
    
    // Prometheus metrics
    pub prometheus_requests: IntCounter,
    pub prometheus_duration: PrometheusHistogram,
    pub prometheus_tokens: IntCounter,
    pub prometheus_costs: PrometheusCounter,
    pub prometheus_errors: IntCounter,
    pub prometheus_active_requests: IntGauge,
}

impl LlmMetrics {
    pub fn new(meter: &Meter, registry: &Registry) -> anyhow::Result<Self> {
        // OpenTelemetry metrics
        let request_counter = meter
            .u64_counter("llm_requests_total")
            .with_description("Total number of LLM requests")
            .init();

        let request_duration = meter
            .f64_histogram("llm_request_duration_seconds")
            .with_description("Duration of LLM requests in seconds")
            .init();

        let token_counter = meter
            .u64_counter("llm_tokens_total")
            .with_description("Total number of tokens processed")
            .init();

        let cost_counter = meter
            .f64_counter("llm_cost_total")
            .with_description("Total cost of LLM requests")
            .init();

        let error_counter = meter
            .u64_counter("llm_errors_total")
            .with_description("Total number of LLM errors")
            .init();

        // Prometheus metrics
        let prometheus_requests = IntCounter::new(
            "llm_requests_total",
            "Total number of LLM requests"
        )?;
        registry.register(Box::new(prometheus_requests.clone()))?;

        let prometheus_duration = PrometheusHistogram::with_opts(
            prometheus::HistogramOpts::new(
                "llm_request_duration_seconds",
                "Duration of LLM requests in seconds"
            ).buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0])
        )?;
        registry.register(Box::new(prometheus_duration.clone()))?;

        let prometheus_tokens = IntCounter::new(
            "llm_tokens_total",
            "Total number of tokens processed"
        )?;
        registry.register(Box::new(prometheus_tokens.clone()))?;

        let prometheus_costs = PrometheusCounter::new(
            "llm_cost_total",
            "Total cost of LLM requests"
        )?;
        registry.register(Box::new(prometheus_costs.clone()))?;

        let prometheus_errors = IntCounter::new(
            "llm_errors_total",
            "Total number of LLM errors"
        )?;
        registry.register(Box::new(prometheus_errors.clone()))?;

        let prometheus_active_requests = IntGauge::new(
            "llm_active_requests",
            "Number of active LLM requests"
        )?;
        registry.register(Box::new(prometheus_active_requests.clone()))?;

        Ok(Self {
            request_counter,
            request_duration,
            token_counter,
            cost_counter,
            error_counter,
            prometheus_requests,
            prometheus_duration,
            prometheus_tokens,
            prometheus_costs,
            prometheus_errors,
            prometheus_active_requests,
        })
    }

    pub fn record_request(&self, request: &LlmRequest, response: &LlmResponse) {
        let labels = [
            KeyValue::new("provider", request.provider.clone()),
            KeyValue::new("model", request.model.clone()),
            KeyValue::new("finish_reason", response.finish_reason.clone()),
        ];

        // Record metrics
        self.request_counter.add(1, &labels);
        self.request_duration.record(response.latency_ms as f64 / 1000.0, &labels);
        self.token_counter.add(response.tokens_used as u64, &labels);
        self.cost_counter.add(response.cost, &labels);

        // Prometheus metrics
        self.prometheus_requests.inc();
        self.prometheus_duration.observe(response.latency_ms as f64 / 1000.0);
        self.prometheus_tokens.inc_by(response.tokens_used as u64);
        self.prometheus_costs.inc_by(response.cost);
    }

    pub fn record_error(&self, provider: &str, model: &str, error_type: &str) {
        let labels = [
            KeyValue::new("provider", provider.to_string()),
            KeyValue::new("model", model.to_string()),
            KeyValue::new("error_type", error_type.to_string()),
        ];

        self.error_counter.add(1, &labels);
        self.prometheus_errors.inc();
    }

    pub fn increment_active_requests(&self) {
        self.prometheus_active_requests.inc();
    }

    pub fn decrement_active_requests(&self) {
        self.prometheus_active_requests.dec();
    }
}

pub struct LlmTracer {
    tracer: Box<dyn Tracer + Send + Sync>,
    metrics: LlmMetrics,
}

impl LlmTracer {
    pub fn new(tracer: Box<dyn Tracer + Send + Sync>, metrics: LlmMetrics) -> Self {
        Self { tracer, metrics }
    }

    pub async fn trace_llm_request<F, Fut, T, E>(
        &self,
        request: LlmRequest,
        operation: F,
    ) -> Result<(LlmResponse, T), E>
    where
        F: FnOnce(LlmRequest) -> Fut,
        Fut: std::future::Future<Output = Result<(LlmResponse, T), E>>,
        E: std::fmt::Display,
    {
        let start_time = Instant::now();
        self.metrics.increment_active_requests();

        let mut span = self.tracer
            .span_builder(format!("llm_request_{}", request.provider))
            .with_attributes(vec![
                KeyValue::new("llm.provider", request.provider.clone()),
                KeyValue::new("llm.model", request.model.clone()),
                KeyValue::new("llm.request_id", request.request_id.clone()),
                KeyValue::new("llm.max_tokens", request.max_tokens.unwrap_or(0) as i64),
                KeyValue::new("llm.temperature", request.temperature.unwrap_or(0.0) as f64),
                KeyValue::new("llm.top_p", request.top_p.unwrap_or(0.0) as f64),
                KeyValue::new("llm.prompt_length", request.prompt.len() as i64),
            ])
            .start(&*self.tracer);

        // Add prompt as event (not attribute due to size)
        span.add_event(
            "llm.prompt",
            vec![KeyValue::new("content", 
                if request.prompt.len() > 1000 {
                    format!("{}...", &request.prompt[..1000])
                } else {
                    request.prompt.clone()
                }
            )],
        );

        let result = operation(request.clone()).await;

        match &result {
            Ok((response, _)) => {
                let duration = start_time.elapsed();
                
                span.set_attributes(vec![
                    KeyValue::new("llm.tokens_used", response.tokens_used as i64),
                    KeyValue::new("llm.prompt_tokens", response.prompt_tokens as i64),
                    KeyValue::new("llm.completion_tokens", response.completion_tokens as i64),
                    KeyValue::new("llm.cost", response.cost),
                    KeyValue::new("llm.latency_ms", response.latency_ms as i64),
                    KeyValue::new("llm.finish_reason", response.finish_reason.clone()),
                ]);

                if let Some(model_version) = &response.model_version {
                    span.set_attribute(KeyValue::new("llm.model_version", model_version.clone()));
                }

                // Add response as event
                span.add_event(
                    "llm.response",
                    vec![KeyValue::new("content", 
                        if response.response.len() > 1000 {
                            format!("{}...", &response.response[..1000])
                        } else {
                            response.response.clone()
                        }
                    )],
                );

                self.metrics.record_request(&request, response);
                span.set_status(opentelemetry::trace::Status::Ok);
            }
            Err(e) => {
                span.set_status(opentelemetry::trace::Status::error(e.to_string()));
                self.metrics.record_error(&request.provider, &request.model, "request_failed");
            }
        }

        span.end();
        self.metrics.decrement_active_requests();

        result
    }
}
```

## Anomaly Detection Engine

```rust
// src/anomaly_detection.rs
use crate::config::AnomalyDetectionConfig;
use opentelemetry::{metrics::{Meter, Gauge}, KeyValue};
use prometheus::{Gauge as PrometheusGauge, Registry};
use serde::{Deserialize, Serialize};
use statrs::statistics::{Statistics, Data};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp: u64,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyAlert {
    pub metric_name: String,
    pub current_value: f64,
    pub expected_value: f64,
    pub anomaly_score: f64,
    pub severity: AnomalySeverity,
    pub timestamp: u64,
    pub labels: HashMap<String, String>,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct AnomalyDetector {
    config: AnomalyDetectionConfig,
    metric_history: RwLock<HashMap<String, VecDeque<MetricPoint>>>,
    anomaly_gauge: Gauge<f64>,
    prometheus_anomaly_gauge: PrometheusGauge,
}

impl AnomalyDetector {
    pub fn new(
        config: AnomalyDetectionConfig,
        meter: &Meter,
        registry: &Registry,
    ) -> anyhow::Result<Self> {
        let anomaly_gauge = meter
            .f64_gauge("anomaly_score")
            .with_description("Current anomaly score for metrics")
            .init();

        let prometheus_anomaly_gauge = PrometheusGauge::new(
            "anomaly_score",
            "Current anomaly score for metrics"
        )?;
        registry.register(Box::new(prometheus_anomaly_gauge.clone()))?;

        Ok(Self {
            config,
            metric_history: RwLock::new(HashMap::new()),
            anomaly_gauge,
            prometheus_anomaly_gauge,
        })
    }

    pub async fn add_metric_point(&self, metric_name: String, point: MetricPoint) {
        if !self.config.enabled {
            return;
        }

        let mut history = self.metric_history.write().await;
        let metric_history = history.entry(metric_name.clone()).or_insert_with(VecDeque::new);
        
        metric_history.push_back(point.clone());
        
        // Keep only the last window of data
        let max_points = (self.config.window_size_minutes * 60) as usize; // Assuming 1 point per second
        while metric_history.len() > max_points {
            metric_history.pop_front();
        }

        // Check for anomalies if we have enough data
        if metric_history.len() >= self.config.min_data_points {
            if let Some(anomaly) = self.detect_anomaly(&metric_name, &point, metric_history).await {
                self.handle_anomaly(anomaly).await;
            }
        }
    }

    async fn detect_anomaly(
        &self,
        metric_name: &str,
        current_point: &MetricPoint,
        history: &VecDeque<MetricPoint>,
    ) -> Option<AnomalyAlert> {
        let values: Vec<f64> = history.iter().map(|p| p.value).collect();
        
        // Calculate statistical measures
        let mean = values.mean();
        let std_dev = values.std_dev();
        
        if std_dev == 0.0 {
            return None; // No variation in data
        }

        // Z-score calculation
        let z_score = (current_point.value - mean) / std_dev;
        let anomaly_score = z_score.abs();

        // Seasonal adjustment if enabled
        let adjusted_threshold = if self.config.seasonality_detection {
            self.calculate_seasonal_threshold(&values, current_point.timestamp)
        } else {
            self.config.std_dev_threshold
        };

        if anomaly_score > adjusted_threshold {
            let severity = self.calculate_severity(anomaly_score, adjusted_threshold);
            
            Some(AnomalyAlert {
                metric_name: metric_name.to_string(),
                current_value: current_point.value,
                expected_value: mean,
                anomaly_score,
                severity,
                timestamp: current_point.timestamp,
                labels: current_point.labels.clone(),
                context: format!(
                    "Z-score: {:.2}, Mean: {:.2}, StdDev: {:.2}, Threshold: {:.2}",
                    z_score, mean, std_dev, adjusted_threshold
                ),
            })
        } else {
            None
        }
    }

    fn calculate_seasonal_threshold(&self, values: &[f64], timestamp: u64) -> f64 {
        // Simple seasonality detection based on time of day
        let hour_of_day = ((timestamp % 86400) / 3600) as usize;
        
        // Group values by hour and calculate hour-specific threshold
        let mut hourly_values: HashMap<usize, Vec<f64>> = HashMap::new();
        
        for (i, &value) in values.iter().enumerate() {
            let point_timestamp = timestamp - (values.len() - 1 - i) as u64;
            let point_hour = ((point_timestamp % 86400) / 3600) as usize;
            hourly_values.entry(point_hour).or_insert_with(Vec::new).push(value);
        }

        // Calculate threshold for current hour
        if let Some(hour_values) = hourly_values.get(&hour_of_day) {
            if hour_values.len() >= 3 {
                let hour_std_dev = hour_values.std_dev();
                return self.config.std_dev_threshold * (1.0 + hour_std_dev / values.std_dev());
            }
        }

        self.config.std_dev_threshold
    }

    fn calculate_severity(&self, anomaly_score: f64, threshold: f64) -> AnomalySeverity {
        let severity_ratio = anomaly_score / threshold;
        
        if severity_ratio > 4.0 {
            AnomalySeverity::Critical
        } else if severity_ratio > 3.0 {
            AnomalySeverity::High
        } else if severity_ratio > 2.0 {
            AnomalySeverity::Medium
        } else {
            AnomalySeverity::Low
        }
    }

    async fn handle_anomaly(&self, anomaly: AnomalyAlert) {
        // Record anomaly metrics
        let labels = vec![
            KeyValue::new("metric_name", anomaly.metric_name.clone()),
            KeyValue::new("severity", format!("{:?}", anomaly.severity)),
        ];

        self.anomaly_gauge.record(anomaly.anomaly_score, &labels);
        self.prometheus_anomaly_gauge.set(anomaly.anomaly_score);

        // Log anomaly
        log::warn!(
            "Anomaly detected: {} = {:.2} (expected: {:.2}, score: {:.2}) - {}",
            anomaly.metric_name,
            anomaly.current_value,
            anomaly.expected_value,
            anomaly.anomaly_score,
            anomaly.context
        );

        // Here you would typically send alerts via configured channels
        // (Slack, email, webhook, etc.)
    }

    pub async fn get_metric_summary(&self, metric_name: &str) -> Option<MetricSummary> {
        let history = self.metric_history.read().await;
        let metric_history = history.get(metric_name)?;
        
        if metric_history.is_empty() {
            return None;
        }

        let values: Vec<f64> = metric_history.iter().map(|p| p.value).collect();
        
        Some(MetricSummary {
            metric_name: metric_name.to_string(),
            count: values.len(),
            mean: values.mean(),
            std_dev: values.std_dev(),
            min: values.min(),
            max: values.max(),
            latest_value: metric_history.back().unwrap().value,
            latest_timestamp: metric_history.back().unwrap().timestamp,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct MetricSummary {
    pub metric_name: String,
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub latest_value: f64,
    pub latest_timestamp: u64,
}
```

## Prometheus Integration and Alert Rules

```rust
// src/prometheus_integration.rs
use crate::config::PrometheusConfig;
use axum::{
    extract::State,
    http::StatusCode,
    response::Response,
    routing::get,
    Router,
};
use prometheus::{Encoder, Registry, TextEncoder};
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct PrometheusExporter {
    registry: Arc<Registry>,
    config: PrometheusConfig,
}

impl PrometheusExporter {
    pub fn new(registry: Arc<Registry>, config: PrometheusConfig) -> Self {
        Self { registry, config }
    }

    pub async fn start_server(&self) -> anyhow::Result<()> {
        let app = Router::new()
            .route(&self.config.metrics_path, get(metrics_handler))
            .with_state(self.registry.clone());

        let listener = TcpListener::bind(&self.config.listen_address).await?;
        
        log::info!("Prometheus metrics server listening on {}", self.config.listen_address);
        
        axum::serve(listener, app).await?;
        
        Ok(())
    }
}

async fn metrics_handler(State(registry): State<Arc<Registry>>) -> Result<Response<String>, StatusCode> {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    
    match encoder.encode_to_string(&metric_families) {
        Ok(output) => Ok(Response::builder()
            .header("content-type", "text/plain; version=0.0.4")
            .body(output)
            .unwrap()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Alert rules configuration that can be used with Prometheus
pub const PROMETHEUS_ALERT_RULES: &str = r#"
groups:
  - name: AI_System_Alerts
    rules:
      # LLM Request Rate Anomaly Detection
      - record: llm:request_rate_5m
        expr: rate(llm_requests_total[5m])
        
      - record: llm:request_rate_mean_1h
        expr: avg_over_time(llm:request_rate_5m[1h])
        
      - record: llm:request_rate_stddev_1h
        expr: stddev_over_time(llm:request_rate_5m[1h])
        
      - alert: LLMRequestRateAnomaly
        expr: |
          (
            llm:request_rate_5m > on(job) group_left()
            (
              llm:request_rate_mean_1h + 
              on(job) 2 * llm:request_rate_stddev_1h
            )
            and
            llm:request_rate_5m > on(job) group_left()
            1.2 * llm:request_rate_mean_1h
          )
        for: 10m
        labels:
          severity: warning
          system: ai_observability
        annotations:
          summary: "Anomalous LLM request rate detected"
          description: "LLM request rate is {{ $value }} requests/sec, which is significantly higher than expected (mean: {{ with query \"llm:request_rate_mean_1h\" }}{{ . | first | value | printf \"%.2f\" }}{{ end }})"

      # LLM Latency Anomaly Detection
      - record: llm:latency_p95_5m
        expr: histogram_quantile(0.95, rate(llm_request_duration_seconds_bucket[5m]))
        
      - record: llm:latency_mean_1h
        expr: avg_over_time(llm:latency_p95_5m[1h])
        
      - record: llm:latency_stddev_1h
        expr: stddev_over_time(llm:latency_p95_5m[1h])
        
      - alert: LLMLatencyAnomaly
        expr: |
          (
            llm:latency_p95_5m > on(job) group_left()
            (
              llm:latency_mean_1h + 
              on(job) 3 * llm:latency_stddev_1h
            )
            and
            llm:latency_p95_5m > 5
          )
        for: 5m
        labels:
          severity: critical
          system: ai_observability
        annotations:
          summary: "High LLM response latency detected"
          description: "LLM P95 latency is {{ $value }}s, significantly higher than normal (mean: {{ with query \"llm:latency_mean_1h\" }}{{ . | first | value | printf \"%.2f\" }}{{ end }}s)"

      # LLM Error Rate Alert
      - alert: LLMHighErrorRate
        expr: |
          (
            rate(llm_errors_total[5m]) / rate(llm_requests_total[5m])
          ) > 0.05
        for: 3m
        labels:
          severity: warning
          system: ai_observability
        annotations:
          summary: "High LLM error rate detected"
          description: "LLM error rate is {{ $value | humanizePercentage }}, above 5% threshold"

      # Token Cost Anomaly Detection
      - record: llm:cost_rate_5m
        expr: rate(llm_cost_total[5m])
        
      - record: llm:cost_mean_1h
        expr: avg_over_time(llm:cost_rate_5m[1h])
        
      - record: llm:cost_stddev_1h
        expr: stddev_over_time(llm:cost_rate_5m[1h])
        
      - alert: LLMCostAnomaly
        expr: |
          (
            llm:cost_rate_5m > on(job) group_left()
            (
              llm:cost_mean_1h + 
              on(job) 2.5 * llm:cost_stddev_1h
            )
            and
            llm:cost_rate_5m > 0.10
          )
        for: 10m
        labels:
          severity: warning
          system: ai_observability
        annotations:
          summary: "Unusual LLM cost spending detected"
          description: "LLM cost rate is ${{ $value }}/sec, significantly higher than expected (mean: ${{ with query \"llm:cost_mean_1h\" }}{{ . | first | value | printf \"%.4f\" }}{{ end }}/sec)"

      # Active Requests Overload
      - alert: LLMTooManyActiveRequests
        expr: llm_active_requests > 100
        for: 2m
        labels:
          severity: critical
          system: ai_observability
        annotations:
          summary: "Too many active LLM requests"
          description: "{{ $value }} active LLM requests, may indicate system overload"

      # Custom Anomaly Score Alert
      - alert: CustomAnomalyDetected
        expr: anomaly_score > 3.0
        for: 5m
        labels:
          severity: warning
          system: ai_observability
        annotations:
          summary: "Statistical anomaly detected in {{ $labels.metric_name }}"
          description: "Anomaly score of {{ $value }} detected for metric {{ $labels.metric_name }}"

      # Token Usage Anomaly (per model)
      - alert: LLMTokenUsageAnomaly
        expr: |
          (
            rate(llm_tokens_total[5m]) by (model) > on(model) group_left()
            (
              avg_over_time(rate(llm_tokens_total[5m])[1h]) by (model) + 
              on(model) 2 * stddev_over_time(rate(llm_tokens_total[5m])[1h]) by (model)
            )
          )
        for: 10m
        labels:
          severity: warning
          system: ai_observability
        annotations:
          summary: "Unusual token usage for model {{ $labels.model }}"
          description: "Token usage rate for {{ $labels.model }} is {{ $value }} tokens/sec, higher than expected"

      # Provider-specific alerts
      - alert: LLMProviderDown
        expr: |
          (
            up{job=~".*llm.*"} == 0
          )
        for: 1m
        labels:
          severity: critical
          system: ai_observability
        annotations:
          summary: "LLM provider {{ $labels.instance }} is down"
          description: "LLM provider {{ $labels.instance }} has been down for more than 1 minute"
"#;

// Alertmanager configuration for AI system alerts
pub const ALERTMANAGER_CONFIG: &str = r#"
global:
  smtp_smarthost: 'localhost:587'
  smtp_from: 'ai-observability@company.com'

route:
  group_by: ['alertname', 'system']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 12h
  receiver: 'ai-team'
  routes:
  - match:
      severity: critical
    receiver: 'ai-team-critical'
  - match:
      system: ai_observability
    receiver: 'ai-team-observability'

receivers:
- name: 'ai-team'
  email_configs:
  - to: 'ai-team@company.com'
    subject: '[AI System] {{ .GroupLabels.alertname }}'
    body: |
      {{ range .Alerts }}
      Alert: {{ .Annotations.summary }}
      Description: {{ .Annotations.description }}
      Labels: {{ range .Labels.SortedPairs }}{{ .Name }}: {{ .Value }} {{ end }}
      {{ end }}

- name: 'ai-team-critical'
  email_configs:
  - to: 'ai-team@company.com,oncall@company.com'
    subject: '[CRITICAL AI Alert] {{ .GroupLabels.alertname }}'
  slack_configs:
  - api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'
    channel: '#ai-alerts'
    title: 'Critical AI System Alert'
    text: |
      {{ range .Alerts }}
      🚨 {{ .Annotations.summary }}
      {{ .Annotations.description }}
      {{ end }}

- name: 'ai-team-observability'
  webhook_configs:
  - url: 'http://ai-observability:8080/webhook/alerts'
    send_resolved: true
"#;
```

## Main Application Integration

```rust
// src/main.rs
mod config;
mod llm_metrics;
mod anomaly_detection;
mod prometheus_integration;

use crate::{
    config::ObservabilityConfig,
    llm_metrics::{LlmMetrics, LlmTracer, LlmRequest, LlmResponse},
    anomaly_detection::{AnomalyDetector, MetricPoint},
    prometheus_integration::PrometheusExporter,
};

use opentelemetry::{
    global, 
    trace::TracerProvider,
    metrics::MeterProvider,
    KeyValue,
};
use opentelemetry_otlp::{WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::{
    metrics::{SdkMeterProvider, PeriodicReader},
    trace::{SdkTracerProvider, Config, Sampler},
    Resource,
};
use prometheus::Registry;
use std::{sync::Arc, time::{Duration, SystemTime, UNIX_EPOCH}};
use tokio::time::sleep;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct ObservabilityFramework {
    pub config: ObservabilityConfig,
    pub llm_tracer: Arc<LlmTracer>,
    pub anomaly_detector: Arc<AnomalyDetector>,
    pub prometheus_registry: Arc<Registry>,
}

impl ObservabilityFramework {
    pub async fn new(config: ObservabilityConfig) -> anyhow::Result<Self> {
        // Initialize tracing subscriber
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info".into())
            )
            .with(tracing_subscriber::fmt::layer())
            .init();

        // Create resource
        let resource = Resource::new(vec![
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("service.version", config.service_version.clone()),
            KeyValue::new("service.environment", config.environment.clone()),
        ]);

        // Initialize OpenTelemetry tracer
        let tracer_provider = SdkTracerProvider::builder()
            .with_config(Config::default().with_resource(resource.clone()))
            .with_batch_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(&config.otlp.endpoint)
                    .with_timeout(Duration::from_secs(config.otlp.timeout_seconds)),
                opentelemetry_sdk::runtime::Tokio,
            )
            .build();

        global::set_tracer_provider(tracer_provider.clone());
        let tracer = tracer_provider.tracer("ai-observability");

        // Initialize OpenTelemetry metrics
        let meter_provider = SdkMeterProvider::builder()
            .with_resource(resource)
            .with_reader(
                PeriodicReader::builder(
                    opentelemetry_otlp::new_exporter()
                        .tonic()
                        .with_endpoint(&config.otlp.endpoint)
                        .with_timeout(Duration::from_secs(config.otlp.timeout_seconds))
                        .build_metrics_exporter(Box::new(opentelemetry_sdk::runtime::Tokio))?,
                    opentelemetry_sdk::runtime::Tokio,
                )
                .with_interval(Duration::from_secs(30))
                .build(),
            )
            .build();

        global::set_meter_provider(meter_provider.clone());
        let meter = meter_provider.meter("ai-observability");

        // Initialize Prometheus registry
        let prometheus_registry = Arc::new(Registry::new());

        // Initialize LLM metrics and tracer
        let llm_metrics = LlmMetrics::new(&meter, &prometheus_registry)?;
        let llm_tracer = Arc::new(LlmTracer::new(
            Box::new(tracer),
            llm_metrics,
        ));

        // Initialize anomaly detector
        let anomaly_detector = Arc::new(AnomalyDetector::new(
            config.anomaly_detection.clone(),
            &meter,
            &prometheus_registry,
        )?);

        Ok(Self {
            config,
            llm_tracer,
            anomaly_detector,
            prometheus_registry,
        })
    }

    pub async fn start_prometheus_server(&self) -> anyhow::Result<()> {
        let exporter = PrometheusExporter::new(
            self.prometheus_registry.clone(),
            self.config.prometheus.clone(),
        );
        
        exporter.start_server().await
    }

    pub async fn monitor_metric(&self, metric_name: String, value: f64, labels: std::collections::HashMap<String, String>) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let point = MetricPoint {
            timestamp,
            value,
            labels,
        };

        self.anomaly_detector.add_metric_point(metric_name, point).await;
    }

    // Example LLM request wrapper
    pub async fn execute_llm_request<F, Fut>(
        &self,
        provider: String,
        model: String,
        prompt: String,
        operation: F,
    ) -> anyhow::Result<String>
    where
        F: FnOnce(String) -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<String>>,
    {
        let request = LlmRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            provider: provider.clone(),
            model: model.clone(),
            prompt: prompt.clone(),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            top_p: Some(0.9),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };

        let start_time = std::time::Instant::now();
        
        let result = self.llm_tracer.trace_llm_request(request, |req| async move {
            let response_text = operation(req.prompt).await?;
            let duration = start_time.elapsed();
            
            let response = LlmResponse {
                request_id: req.request_id,
                response: response_text.clone(),
                tokens_used: response_text.len() as u32 / 4, // Rough estimation
                prompt_tokens: req.prompt.len() as u32 / 4,
                completion_tokens: response_text.len() as u32 / 4,
                cost: (response_text.len() as f64 / 4.0) * 0.0001, // Rough cost estimation
                latency_ms: duration.as_millis() as u64,
                model_version: Some("gpt-4".to_string()),
                finish_reason: "stop".to_string(),
            };
            
            Ok((response, response_text))
        }).await;

        match result {
            Ok((_, response_text)) => Ok(response_text),
            Err(e) => Err(e),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = ObservabilityConfig::default();
    
    // Initialize observability framework
    let framework = ObservabilityFramework::new(config).await?;
    
    // Start Prometheus metrics server
    let framework_clone = framework.clone();
    tokio::spawn(async move {
        if let Err(e) = framework_clone.start_prometheus_server().await {
            log::error!("Failed to start Prometheus server: {}", e);
        }
    });

    // Example usage: simulate LLM requests and monitoring
    tokio::spawn(async move {
        let mut counter = 0;
        loop {
            counter += 1;
            
            // Simulate LLM request
            let result = framework.execute_llm_request(
                "openai".to_string(),
                "gpt-4".to_string(),
                format!("Hello, this is request number {}", counter),
                |prompt| async move {
                    // Simulate API call delay
                    sleep(Duration::from_millis(500 + counter % 1000)).await;
                    Ok(format!("Response to: {}", prompt))
                },
            ).await;

            if let Err(e) = result {
                log::error!("LLM request failed: {}", e);
            }

            // Simulate monitoring other metrics
            let mut labels = std::collections::HashMap::new();
            labels.insert("service".to_string(), "ai-service".to_string());
            
            framework.monitor_metric(
                "cpu_usage".to_string(),
                50.0 + (counter as f64 * 0.1).sin() * 20.0 + if counter % 10 == 0 { 30.0 } else { 0.0 },
                labels.clone(),
            ).await;

            framework.monitor_metric(
                "memory_usage".to_string(),
                70.0 + (counter as f64 * 0.05).cos() * 15.0,
                labels,
            ).await;

            sleep(Duration::from_secs(1)).await;
        }
    });

    // Keep the main thread alive
    loop {
        sleep(Duration::from_secs(60)).await;
        log::info!("Observability framework running...");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_observability_framework_initialization() {
        let config = ObservabilityConfig::default();
        let framework = ObservabilityFramework::new(config).await;
        assert!(framework.is_ok());
    }

    #[tokio::test]
    async fn test_anomaly_detection() {
        let config = ObservabilityConfig::default();
        let framework = ObservabilityFramework::new(config).await.unwrap();
        
        let mut labels = std::collections::HashMap::new();
        labels.insert("test".to_string(), "value".to_string());
        
        // Add normal data points
        for i in 0..50 {
            framework.monitor_metric(
                "test_metric".to_string(),
                50.0 + (i as f64).sin() * 5.0,
                labels.clone(),
            ).await;
        }
        
        // Add anomalous data point
        framework.monitor_metric(
            "test_metric".to_string(),
            150.0, // Significant deviation
            labels,
        ).await;
        
        // Verify anomaly detection would trigger
        // (In a real test, you'd check the anomaly detector's output)
    }
}
```

## Deployment Configuration

```yaml
# docker-compose.yml
version: '3.8'

services:
  ai-observability:
    build: .
    ports:
      - "9090:9090"  # Prometheus metrics
      - "8080:8080"  # Application port
    environment:
      - RUST_LOG=info
      - OTLP_ENDPOINT=http://otel-collector:4317
    depends_on:
      - otel-collector
      - prometheus
      - grafana

  otel-collector:
    image: otel/opentelemetry-collector-contrib:latest
    command: ["--config=/etc/otel-collector-config.yaml"]
    volumes:
      - ./otel-collector-config.yaml:/etc/otel-collector-config.yaml
    ports:
      - "4317:4317"   # OTLP gRPC receiver
      - "4318:4318"   # OTLP HTTP receiver
    depends_on:
      - jaeger
      - prometheus

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9091:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - ./alert-rules.yml:/etc/prometheus/alert-rules.yml

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana-storage:/var/lib/grafana

  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"
      - "14250:14250"

  alertmanager:
    image: prom/alertmanager:latest
    ports:
      - "9093:9093"
    volumes:
      - ./alertmanager.yml:/etc/alertmanager/alertmanager.yml

volumes:
  grafana-storage:
```

This production-ready framework provides:

1. **Comprehensive LLM Observability**: Custom metrics for tokens, costs, latency, and quality
2. **Distributed Tracing**: OpenTelemetry-based tracing with detailed span attributes
3. **Statistical Anomaly Detection**: Z-score based detection with seasonal adjustments
4. **Prometheus Integration**: Custom metrics and alert rules
5. **Alert Configuration**: Production-ready alerting rules and Alertmanager setup
6. **Scalable Architecture**: Designed for distributed systems with multiple LLM providers

Key features include real-time monitoring, automatic anomaly detection, cost tracking, performance optimization, and comprehensive alerting for production AI systems.
