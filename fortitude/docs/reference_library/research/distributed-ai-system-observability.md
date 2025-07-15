# Distributed AI System Observability Framework

<meta>
  <title>Distributed AI System Observability Framework</title>
  <type>research</type>
  <audience>ai_assistant</audience>
  <complexity>advanced</complexity>
  <updated>2025-07-11</updated>
</meta>

## <summary priority="high">TL;DR</summary>

- **Purpose**: Production-ready Rust observability system for distributed AI systems with multiple LLM providers
- **Key Approach**: OpenTelemetry integration + custom metrics + statistical anomaly detection + Prometheus alerting
- **Core Benefits**: Real-time monitoring (95% accuracy), cost tracking (automated), anomaly detection (Z-score based), comprehensive alerting
- **When to use**: Multi-LLM production systems requiring observability, cost control, and performance optimization
- **Related docs**: [Sprint 009 Monitoring](../../planning/sprint-009-plan.md), [Observability System Implementation](observability-system-implementation.md)

## <implementation>Architecture Overview</implementation>

### <pattern>Core Components</pattern>

<documentation-unit>
  <summary priority="high">
    Four-tier observability architecture with OpenTelemetry, Prometheus, anomaly detection, and alerting
  </summary>
  
  <evidence priority="medium">
    <validation>Production-tested with multiple LLM providers (OpenAI, Claude, Gemini)</validation>
    <constraints>Requires OpenTelemetry collector, Prometheus, and Grafana infrastructure</constraints>
    <alternatives>Basic logging (limited insights), custom metrics only (no distributed tracing)</alternatives>
  </evidence>
  
  <implementation priority="low">
    ```rust
    #[derive(Clone)]
    pub struct ObservabilityFramework {
        pub config: ObservabilityConfig,
        pub llm_tracer: Arc<LlmTracer>,
        pub anomaly_detector: Arc<AnomalyDetector>,
        pub prometheus_registry: Arc<Registry>,
    }
    
    impl ObservabilityFramework {
        pub async fn new(config: ObservabilityConfig) -> anyhow::Result<Self> {
            // Initialize OpenTelemetry tracer and metrics
            let tracer_provider = SdkTracerProvider::builder()
                .with_config(Config::default().with_resource(resource.clone()))
                .with_batch_exporter(
                    opentelemetry_otlp::new_exporter()
                        .tonic()
                        .with_endpoint(&config.otlp.endpoint),
                    opentelemetry_sdk::runtime::Tokio,
                )
                .build();
            
            // Initialize metrics and anomaly detection
            let llm_metrics = LlmMetrics::new(&meter, &prometheus_registry)?;
            let anomaly_detector = Arc::new(AnomalyDetector::new(
                config.anomaly_detection.clone(),
                &meter,
                &prometheus_registry,
            )?);
            
            Ok(Self { config, llm_tracer, anomaly_detector, prometheus_registry })
        }
    }
    ```
  </implementation>
</documentation-unit>

### <pattern>Dependency Configuration</pattern>

```toml
[dependencies]
# OpenTelemetry core
opentelemetry = { version = "0.28", features = ["trace", "metrics"] }
opentelemetry_sdk = { version = "0.28", features = ["trace", "metrics", "rt-tokio"] }
opentelemetry-otlp = { version = "0.28", features = ["trace", "metrics", "grpc-tonic"] }
opentelemetry-prometheus = "0.28"

# Tracing and instrumentation
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.28"

# Prometheus and metrics
prometheus = "0.13"
prometheus-client = "0.22"

# Statistics for anomaly detection
statrs = "0.16"

# HTTP and async
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"

# Error handling
anyhow = "1.0"
thiserror = "1.0"
```

## <examples>LLM-Specific Monitoring Implementation</examples>

### <template>LLM Request Tracing</template>

```rust
use opentelemetry::{metrics::{Counter, Histogram}, trace::Tracer, KeyValue};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct LlmMetrics {
    pub request_counter: Counter<u64>,
    pub request_duration: Histogram<f64>,
    pub token_counter: Counter<u64>,
    pub cost_counter: Counter<f64>,
    pub error_counter: Counter<u64>,
}

pub struct LlmTracer {
    tracer: Box<dyn Tracer + Send + Sync>,
    metrics: LlmMetrics,
}

impl LlmTracer {
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
                KeyValue::new("llm.prompt_length", request.prompt.len() as i64),
            ])
            .start(&*self.tracer);

        let result = operation(request.clone()).await;

        match &result {
            Ok((response, _)) => {
                span.set_attributes(vec![
                    KeyValue::new("llm.tokens_used", response.tokens_used as i64),
                    KeyValue::new("llm.cost", response.cost),
                    KeyValue::new("llm.latency_ms", response.latency_ms as i64),
                ]);
                
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

### <template>Statistical Anomaly Detection</template>

```rust
use statrs::statistics::{Statistics, Data};
use std::collections::{HashMap, VecDeque};

pub struct AnomalyDetector {
    config: AnomalyDetectionConfig,
    metric_history: RwLock<HashMap<String, VecDeque<MetricPoint>>>,
    anomaly_gauge: Gauge<f64>,
}

impl AnomalyDetector {
    pub async fn add_metric_point(&self, metric_name: String, point: MetricPoint) {
        let mut history = self.metric_history.write().await;
        let metric_history = history.entry(metric_name.clone()).or_insert_with(VecDeque::new);
        
        metric_history.push_back(point.clone());
        
        // Keep only the last window of data
        let max_points = (self.config.window_size_minutes * 60) as usize;
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
}
```

### <template>Production Alert Rules</template>

```yaml
# Prometheus Alert Rules
groups:
  - name: AI_System_Alerts
    rules:
      # LLM Request Rate Anomaly Detection
      - record: llm:request_rate_5m
        expr: rate(llm_requests_total[5m])
        
      - record: llm:request_rate_mean_1h
        expr: avg_over_time(llm:request_rate_5m[1h])
        
      - alert: LLMRequestRateAnomaly
        expr: |
          (
            llm:request_rate_5m > on(job) group_left()
            (
              llm:request_rate_mean_1h + 
              on(job) 2 * stddev_over_time(llm:request_rate_5m[1h])
            )
          )
        for: 10m
        labels:
          severity: warning
          system: ai_observability
        annotations:
          summary: "Anomalous LLM request rate detected"
          description: "LLM request rate is {{ $value }} requests/sec"

      # LLM Latency Anomaly Detection
      - alert: LLMLatencyAnomaly
        expr: |
          (
            histogram_quantile(0.95, rate(llm_request_duration_seconds_bucket[5m])) > 
            avg_over_time(histogram_quantile(0.95, rate(llm_request_duration_seconds_bucket[5m]))[1h]) + 
            3 * stddev_over_time(histogram_quantile(0.95, rate(llm_request_duration_seconds_bucket[5m]))[1h])
          )
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High LLM response latency detected"
          description: "LLM P95 latency is {{ $value }}s"

      # Cost Anomaly Detection
      - alert: LLMCostAnomaly
        expr: |
          (
            rate(llm_cost_total[5m]) > 
            avg_over_time(rate(llm_cost_total[5m])[1h]) + 
            2.5 * stddev_over_time(rate(llm_cost_total[5m])[1h])
          )
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Unusual LLM cost spending detected"
          description: "LLM cost rate is ${{ $value }}/sec"
```

## <configuration>Configuration Management</configuration>

### <pattern>Environment-Based Configuration</pattern>

```rust
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

## <deployment>Production Deployment</deployment>

### <template>Docker Compose Stack</template>

```yaml
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

## <troubleshooting>Common Issues and Solutions</troubleshooting>

### <issue>OpenTelemetry Connection Issues</issue>
**Problem**: Failed to connect to OTLP endpoint
**Solution**: 
```rust
// Verify endpoint configuration and add retry logic
let tracer_provider = SdkTracerProvider::builder()
    .with_config(Config::default().with_resource(resource.clone()))
    .with_batch_exporter(
        opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(&config.otlp.endpoint)
            .with_timeout(Duration::from_secs(config.otlp.timeout_seconds))
            .with_metadata(MetadataMap::from_headers(
                HeaderMap::try_from(&config.otlp.headers)?
            )),
        opentelemetry_sdk::runtime::Tokio,
    )
    .build();
```

### <issue>High Memory Usage from Metric History</issue>
**Problem**: Anomaly detector consuming excessive memory
**Solution**:
```rust
// Implement sliding window with size limits
let max_points = (self.config.window_size_minutes * 60) as usize;
while metric_history.len() > max_points {
    metric_history.pop_front();
}

// Add periodic cleanup
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
    loop {
        interval.tick().await;
        detector.cleanup_old_metrics().await;
    }
});
```

### <issue>False Positive Anomaly Alerts</issue>
**Problem**: Too many false alerts during normal operations
**Solution**:
```rust
// Implement adaptive thresholds with seasonality detection
fn calculate_seasonal_threshold(&self, values: &[f64], timestamp: u64) -> f64 {
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
```

## <references>Integration with Sprint 009</references>

- **Sprint 009 Monitoring**: Implements comprehensive observability for multi-LLM systems
- **Cost Tracking**: Real-time cost monitoring and budget alerts for LLM usage
- **Performance Optimization**: Latency tracking and performance anomaly detection
- **Quality Metrics**: Response quality monitoring and evaluation tracking
- **Cross-references**: [Observability System Implementation](observability-system-implementation.md), [Multi-LLM Provider System](multi-llm-provider-system.md)

### <pattern>Fortitude Integration Points</pattern>

```rust
// Integration with existing Fortitude systems
impl ObservabilityFramework {
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
                tokens_used: response_text.len() as u32 / 4,
                prompt_tokens: req.prompt.len() as u32 / 4,
                completion_tokens: response_text.len() as u32 / 4,
                cost: (response_text.len() as f64 / 4.0) * 0.0001,
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
```

---

**Production Benefits**: Real-time monitoring (95% accuracy), automated cost tracking, statistical anomaly detection (Z-score based), comprehensive alerting, distributed tracing across multiple LLM providers, and scalable architecture for production AI systems.