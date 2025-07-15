# Monitoring and Observability Architecture

<meta>
  <title>Monitoring and Observability Architecture</title>
  <type>architecture</type>
  <audience>ai_assistant</audience>
  <complexity>high</complexity>
  <updated>2025-07-12</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Comprehensive observability system achieving <200ms response times with <5% monitoring overhead
- **Key Architecture**: Metrics collection + distributed tracing + health monitoring + alerting = 360-degree observability
- **Core Benefits**: Real-time performance insights, automated anomaly detection, proactive issue resolution
- **When to use**: Production deployments requiring high availability and performance visibility
- **Related docs**: [Multi-LLM Architecture](multi-llm-architecture.md), [Quality Control Design](quality-control-design.md)

## <context>System Overview</context>

The Monitoring and Observability Architecture provides comprehensive performance tracking, health monitoring, and alerting capabilities for all Fortitude components. It enables real-time visibility into system performance, automated anomaly detection, and proactive issue resolution.

### <architecture>Core Design Principles</architecture>

```rust
// Core monitoring traits and interfaces
#[async_trait]
pub trait Monitorable: Send + Sync {
    fn component_name(&self) -> &str;
    
    async fn record_operation_metrics(
        &self,
        operation_name: &str,
        duration: Duration,
        success: bool,
        metadata: Option<HashMap<String, String>>,
    ) -> MonitoringResult<()>;
    
    async fn get_health_status(&self) -> MonitoringResult<ComponentHealth>;
    async fn get_performance_metrics(&self) -> MonitoringResult<ComponentMetrics>;
}

// Monitoring system orchestration
#[async_trait]
pub trait MonitoringSystem: Send + Sync {
    async fn initialize(&mut self, config: MonitoringConfig) -> MonitoringResult<()>;
    async fn register_component(&mut self, component: Box<dyn Monitorable>) -> MonitoringResult<()>;
    async fn get_system_performance_report(&self) -> MonitoringResult<SystemPerformanceReport>;
    async fn check_performance_thresholds(&self) -> MonitoringResult<Vec<ThresholdViolation>>;
}
```

## <implementation>Architecture Components</implementation>

### **1. Metrics Collection System**

```rust
pub struct MetricsCollector {
    storage: Arc<dyn MetricsStorage>,
    aggregators: Vec<Box<dyn MetricsAggregator>>,
    exporters: Vec<Box<dyn MetricsExporter>>,
    collection_config: CollectionConfig,
}

impl MetricsCollector {
    pub async fn collect_api_metrics(
        &self,
        request_info: &RequestInfo,
        response_info: &ResponseInfo,
        duration: Duration,
    ) -> MonitoringResult<()> {
        // Create comprehensive API metric
        let metric = ApiMetric {
            timestamp: Utc::now(),
            endpoint: request_info.endpoint.clone(),
            method: request_info.method.clone(),
            status_code: response_info.status_code,
            duration,
            request_size: request_info.body_size,
            response_size: response_info.body_size,
            user_agent: request_info.user_agent.clone(),
            client_ip: request_info.client_ip.clone(),
            provider_used: response_info.provider_used.clone(),
            quality_score: response_info.quality_score,
            cache_hit: response_info.cache_hit,
        };
        
        // Store metric
        self.storage.store_api_metric(&metric).await?;
        
        // Real-time aggregation
        for aggregator in &self.aggregators {
            aggregator.aggregate_api_metric(&metric).await?;
        }
        
        // Export to external systems
        for exporter in &self.exporters {
            exporter.export_api_metric(&metric).await?;
        }
        
        Ok(())
    }
    
    pub async fn collect_provider_metrics(
        &self,
        provider_name: &str,
        operation: &str,
        result: &ProviderOperationResult,
    ) -> MonitoringResult<()> {
        let metric = ProviderMetric {
            timestamp: Utc::now(),
            provider_name: provider_name.to_string(),
            operation: operation.to_string(),
            success: result.success,
            duration: result.duration,
            tokens_used: result.tokens_used,
            cost_usd: result.cost_usd,
            quality_score: result.quality_score,
            error_type: result.error_type.clone(),
            rate_limited: result.rate_limited,
        };
        
        self.storage.store_provider_metric(&metric).await?;
        self.aggregate_and_export_provider_metric(&metric).await?;
        
        Ok(())
    }
    
    pub async fn collect_quality_metrics(
        &self,
        evaluation: &QualityEvaluation,
    ) -> MonitoringResult<()> {
        let metric = QualityMetric {
            timestamp: evaluation.timestamp,
            provider: evaluation.provider.clone(),
            overall_score: evaluation.score.composite,
            relevance_score: evaluation.score.relevance,
            accuracy_score: evaluation.score.accuracy,
            completeness_score: evaluation.score.completeness,
            clarity_score: evaluation.score.clarity,
            credibility_score: evaluation.score.credibility,
            timeliness_score: evaluation.score.timeliness,
            specificity_score: evaluation.score.specificity,
            confidence: evaluation.score.confidence,
            evaluation_time: evaluation.metrics.evaluation_time,
            tokens_processed: evaluation.metrics.tokens_processed,
        };
        
        self.storage.store_quality_metric(&metric).await?;
        self.aggregate_and_export_quality_metric(&metric).await?;
        
        Ok(())
    }
}
```

### **2. Distributed Tracing System**

```rust
pub struct TracingService {
    tracer: opentelemetry::global::Tracer,
    span_processor: Arc<dyn SpanProcessor>,
    context_propagator: Arc<dyn ContextPropagator>,
    sampling_config: SamplingConfig,
}

impl TracingService {
    pub fn start_research_trace(&self, query: &str) -> TraceContext {
        let trace_id = TraceId::new();
        let span = self.tracer
            .span_builder("research_query")
            .with_trace_id(trace_id)
            .with_attributes(vec![
                KeyValue::new("query.text", query.to_string()),
                KeyValue::new("query.length", query.len() as i64),
                KeyValue::new("timestamp", Utc::now().to_rfc3339()),
            ])
            .start(&self.tracer);
        
        TraceContext {
            trace_id,
            span_id: span.span_context().span_id(),
            span: Arc::new(Mutex::new(span)),
            baggage: HashMap::new(),
        }
    }
    
    pub async fn trace_provider_call(
        &self,
        parent_context: &TraceContext,
        provider_name: &str,
        operation: &str,
    ) -> ProviderSpan {
        let span = self.tracer
            .span_builder(format!("provider.{}.{}", provider_name, operation))
            .with_parent_context(&parent_context.to_opentelemetry_context())
            .with_attributes(vec![
                KeyValue::new("provider.name", provider_name.to_string()),
                KeyValue::new("provider.operation", operation.to_string()),
                KeyValue::new("span.kind", "client"),
            ])
            .start(&self.tracer);
        
        ProviderSpan {
            span: Arc::new(Mutex::new(span)),
            start_time: Utc::now(),
            provider_name: provider_name.to_string(),
            operation: operation.to_string(),
        }
    }
    
    pub async fn trace_quality_evaluation(
        &self,
        parent_context: &TraceContext,
        scorer_name: &str,
    ) -> QualitySpan {
        let span = self.tracer
            .span_builder(format!("quality.evaluation.{}", scorer_name))
            .with_parent_context(&parent_context.to_opentelemetry_context())
            .with_attributes(vec![
                KeyValue::new("quality.scorer", scorer_name.to_string()),
                KeyValue::new("evaluation.type", "comprehensive"),
            ])
            .start(&self.tracer);
        
        QualitySpan {
            span: Arc::new(Mutex::new(span)),
            start_time: Utc::now(),
            scorer_name: scorer_name.to_string(),
        }
    }
    
    pub fn inject_trace_context(&self, context: &TraceContext, headers: &mut HeaderMap) {
        self.context_propagator.inject_context(context, headers);
    }
    
    pub fn extract_trace_context(&self, headers: &HeaderMap) -> Option<TraceContext> {
        self.context_propagator.extract_context(headers)
    }
}
```

### **3. Health Monitoring System**

```rust
pub struct HealthChecker {
    components: HashMap<String, Box<dyn HealthCheckable>>,
    check_scheduler: Arc<HealthCheckScheduler>,
    status_storage: Arc<dyn HealthStatusStorage>,
    alert_manager: Arc<AlertManager>,
    config: HealthCheckConfig,
}

impl HealthChecker {
    pub async fn register_component<T: HealthCheckable + 'static>(
        &mut self,
        name: String,
        component: T,
    ) -> MonitoringResult<()> {
        self.components.insert(name.clone(), Box::new(component));
        
        // Schedule regular health checks
        self.check_scheduler
            .schedule_health_check(name, self.config.check_interval)
            .await?;
        
        Ok(())
    }
    
    pub async fn check_system_health(&self) -> HealthReport {
        let mut component_healths = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;
        
        // Check each registered component
        for (name, component) in &self.components {
            match component.check_health().await {
                Ok(health) => {
                    match &health.status {
                        HealthStatus::Unhealthy(_) => {
                            overall_status = HealthStatus::Unhealthy(
                                format!("Component {} is unhealthy", name)
                            );
                        }
                        HealthStatus::Degraded(_) if matches!(overall_status, HealthStatus::Healthy) => {
                            overall_status = HealthStatus::Degraded(
                                format!("Component {} is degraded", name)
                            );
                        }
                        _ => {}
                    }
                    component_healths.insert(name.clone(), health);
                }
                Err(error) => {
                    let unhealthy_status = ComponentHealth {
                        component_name: name.clone(),
                        status: HealthStatus::Unhealthy(format!("Health check failed: {}", error)),
                        message: error.to_string(),
                        last_check_time: Utc::now(),
                        checks: HashMap::new(),
                    };
                    component_healths.insert(name.clone(), unhealthy_status);
                    overall_status = HealthStatus::Unhealthy(format!("Component {} health check failed", name));
                }
            }
        }
        
        // Store system health status
        let system_health = SystemHealth {
            overall_status: overall_status.clone(),
            component_healths: component_healths.clone(),
            last_check_time: Utc::now(),
            system_uptime: self.calculate_system_uptime().await,
        };
        
        let _ = self.status_storage.store_system_health(&system_health).await;
        
        // Generate alerts for health issues
        if !matches!(overall_status, HealthStatus::Healthy) {
            self.generate_health_alert(&overall_status).await;
        }
        
        HealthReport {
            component_name: "system".to_string(),
            status: overall_status,
            message: "System-wide health check complete".to_string(),
            last_check_time: Utc::now(),
            checks: component_healths,
        }
    }
    
    async fn generate_health_alert(&self, status: &HealthStatus) {
        let alert = Alert {
            id: uuid::Uuid::new_v4().to_string(),
            severity: match status {
                HealthStatus::Unhealthy(_) => AlertSeverity::Critical,
                HealthStatus::Degraded(_) => AlertSeverity::Warning,
                HealthStatus::Healthy => return, // No alert needed
            },
            title: "System Health Alert".to_string(),
            message: format!("System health status: {:?}", status),
            timestamp: Utc::now(),
            source: "health_checker".to_string(),
            metadata: HashMap::new(),
        };
        
        if let Err(error) = self.alert_manager.send_alert(alert).await {
            log::error!("Failed to send health alert: {}", error);
        }
    }
}
```

### **4. Alerting System**

```rust
pub struct AlertManager {
    channels: Vec<Box<dyn AlertChannel>>,
    rules: Vec<AlertRule>,
    rate_limiter: Arc<AlertRateLimiter>,
    alert_storage: Arc<dyn AlertStorage>,
    config: AlertingConfig,
}

impl AlertManager {
    pub async fn send_alert(&self, alert: Alert) -> MonitoringResult<()> {
        // Check rate limiting
        if !self.rate_limiter.allow_alert(&alert).await? {
            log::warn!("Alert rate limited: {}", alert.id);
            return Ok(());
        }
        
        // Store alert
        self.alert_storage.store_alert(&alert).await?;
        
        // Apply alert rules for filtering and routing
        let processed_alert = self.apply_alert_rules(alert).await?;
        
        // Send through configured channels
        for channel in &self.channels {
            if channel.should_handle_alert(&processed_alert) {
                if let Err(error) = channel.send_alert(&processed_alert).await {
                    log::error!("Failed to send alert through channel {}: {}", 
                              channel.name(), error);
                }
            }
        }
        
        Ok(())
    }
    
    pub async fn create_performance_alert(
        &self,
        component: &str,
        metric: &str,
        current_value: f64,
        threshold: f64,
        severity: AlertSeverity,
    ) -> Alert {
        Alert {
            id: uuid::Uuid::new_v4().to_string(),
            severity,
            title: format!("Performance Alert: {} {}", component, metric),
            message: format!(
                "Component '{}' metric '{}' value {:.3} exceeds threshold {:.3}",
                component, metric, current_value, threshold
            ),
            timestamp: Utc::now(),
            source: "performance_monitor".to_string(),
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("component".to_string(), component.to_string());
                metadata.insert("metric".to_string(), metric.to_string());
                metadata.insert("current_value".to_string(), current_value.to_string());
                metadata.insert("threshold".to_string(), threshold.to_string());
                metadata
            },
        }
    }
    
    async fn apply_alert_rules(&self, mut alert: Alert) -> MonitoringResult<Alert> {
        for rule in &self.rules {
            if rule.matches(&alert) {
                alert = rule.transform(alert)?;
            }
        }
        Ok(alert)
    }
}

// Email alert channel implementation
pub struct EmailAlertChannel {
    smtp_client: SmtpClient,
    config: EmailConfig,
}

#[async_trait]
impl AlertChannel for EmailAlertChannel {
    fn name(&self) -> &str {
        "email"
    }
    
    fn should_handle_alert(&self, alert: &Alert) -> bool {
        // Send emails for Critical and Warning alerts only
        matches!(alert.severity, AlertSeverity::Critical | AlertSeverity::Warning)
    }
    
    async fn send_alert(&self, alert: &Alert) -> AlertResult<()> {
        let email = Email::builder()
            .to(self.config.to_addresses.clone())
            .from(self.config.from_address.clone())
            .subject(format!("[FORTITUDE] {}", alert.title))
            .body(self.format_alert_email(alert))
            .build()?;
        
        self.smtp_client.send(email).await?;
        Ok(())
    }
}

// Webhook alert channel implementation
pub struct WebhookAlertChannel {
    http_client: reqwest::Client,
    webhook_url: String,
    config: WebhookConfig,
}

#[async_trait]
impl AlertChannel for WebhookAlertChannel {
    fn name(&self) -> &str {
        "webhook"
    }
    
    fn should_handle_alert(&self, alert: &Alert) -> bool {
        true // Handle all alerts
    }
    
    async fn send_alert(&self, alert: &Alert) -> AlertResult<()> {
        let webhook_payload = WebhookPayload {
            alert_id: alert.id.clone(),
            severity: alert.severity.clone(),
            title: alert.title.clone(),
            message: alert.message.clone(),
            timestamp: alert.timestamp,
            source: alert.source.clone(),
            metadata: alert.metadata.clone(),
        };
        
        let response = self.http_client
            .post(&self.webhook_url)
            .json(&webhook_payload)
            .timeout(Duration::from_secs(self.config.timeout_seconds))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(AlertError::WebhookError(format!(
                "Webhook returned status: {}", response.status()
            )));
        }
        
        Ok(())
    }
}
```

### **5. Performance Threshold Monitoring**

```rust
pub struct PerformanceThresholdMonitor {
    thresholds: PerformanceThresholds,
    metrics_collector: Arc<MetricsCollector>,
    alert_manager: Arc<AlertManager>,
    violation_storage: Arc<dyn ViolationStorage>,
}

impl PerformanceThresholdMonitor {
    pub async fn check_all_thresholds(&self) -> MonitoringResult<Vec<ThresholdViolation>> {
        let mut violations = Vec::new();
        
        // Check API response time thresholds
        violations.extend(self.check_api_response_thresholds().await?);
        
        // Check provider performance thresholds
        violations.extend(self.check_provider_thresholds().await?);
        
        // Check quality score thresholds
        violations.extend(self.check_quality_thresholds().await?);
        
        // Check resource utilization thresholds
        violations.extend(self.check_resource_thresholds().await?);
        
        // Check cache performance thresholds
        violations.extend(self.check_cache_thresholds().await?);
        
        // Store violations and generate alerts
        for violation in &violations {
            self.violation_storage.store_violation(violation).await?;
            self.generate_threshold_alert(violation).await?;
        }
        
        Ok(violations)
    }
    
    async fn check_api_response_thresholds(&self) -> MonitoringResult<Vec<ThresholdViolation>> {
        let mut violations = Vec::new();
        let recent_metrics = self.metrics_collector
            .get_recent_api_metrics(Duration::from_secs(300)) // Last 5 minutes
            .await?;
        
        if recent_metrics.is_empty() {
            return Ok(violations);
        }
        
        // Calculate average response time
        let avg_response_time = recent_metrics
            .iter()
            .map(|m| m.duration.as_millis() as f64)
            .sum::<f64>() / recent_metrics.len() as f64;
        
        // Check against threshold
        if avg_response_time > self.thresholds.api_response_time_ms as f64 {
            violations.push(ThresholdViolation {
                id: uuid::Uuid::new_v4().to_string(),
                threshold_name: "api_response_time".to_string(),
                component: "api_server".to_string(),
                metric_name: "average_response_time_ms".to_string(),
                current_value: avg_response_time,
                threshold_value: self.thresholds.api_response_time_ms as f64,
                severity: self.calculate_violation_severity(
                    avg_response_time,
                    self.thresholds.api_response_time_ms as f64,
                ),
                timestamp: Utc::now(),
                duration: self.calculate_violation_duration(&recent_metrics),
            });
        }
        
        // Check error rate
        let error_count = recent_metrics
            .iter()
            .filter(|m| m.status_code >= 400)
            .count();
        let error_rate = error_count as f64 / recent_metrics.len() as f64;
        
        if error_rate > self.thresholds.max_error_rate {
            violations.push(ThresholdViolation {
                id: uuid::Uuid::new_v4().to_string(),
                threshold_name: "api_error_rate".to_string(),
                component: "api_server".to_string(),
                metric_name: "error_rate".to_string(),
                current_value: error_rate,
                threshold_value: self.thresholds.max_error_rate,
                severity: ViolationSeverity::Critical,
                timestamp: Utc::now(),
                duration: Duration::from_secs(300),
            });
        }
        
        Ok(violations)
    }
    
    async fn generate_threshold_alert(&self, violation: &ThresholdViolation) -> MonitoringResult<()> {
        let alert_severity = match violation.severity {
            ViolationSeverity::Critical => AlertSeverity::Critical,
            ViolationSeverity::Warning => AlertSeverity::Warning,
            ViolationSeverity::Info => AlertSeverity::Info,
        };
        
        let alert = self.alert_manager.create_performance_alert(
            &violation.component,
            &violation.metric_name,
            violation.current_value,
            violation.threshold_value,
            alert_severity,
        ).await;
        
        self.alert_manager.send_alert(alert).await?;
        Ok(())
    }
}
```

## <integration>System Integration</integration>

### **Monitoring Middleware Integration**

```rust
// API monitoring middleware
pub struct MonitoringMiddleware {
    metrics_collector: Arc<MetricsCollector>,
    tracing_service: Arc<TracingService>,
    health_checker: Arc<HealthChecker>,
}

impl<S, B> Transform<S, ServiceRequest> for MonitoringMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MonitoringMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MonitoringMiddlewareService {
            service,
            metrics_collector: Arc::clone(&self.metrics_collector),
            tracing_service: Arc::clone(&self.tracing_service),
            health_checker: Arc::clone(&self.health_checker),
        }))
    }
}

impl<S, B> Service<ServiceRequest> for MonitoringMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;
    
    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }
    
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = Utc::now();
        let request_info = RequestInfo::from_service_request(&req);
        
        // Start distributed trace
        let trace_context = self.tracing_service.start_request_trace(&req);
        
        let metrics_collector = Arc::clone(&self.metrics_collector);
        let tracing_service = Arc::clone(&self.tracing_service);
        
        let fut = self.service.call(req);
        
        Box::pin(async move {
            let result = fut.await;
            let end_time = Utc::now();
            let duration = end_time.signed_duration_since(start_time).to_std().unwrap_or_default();
            
            match &result {
                Ok(response) => {
                    let response_info = ResponseInfo::from_service_response(response);
                    
                    // Collect metrics
                    let _ = metrics_collector.collect_api_metrics(
                        &request_info,
                        &response_info,
                        duration,
                    ).await;
                    
                    // Complete trace
                    tracing_service.complete_request_trace(trace_context, &response_info).await;
                }
                Err(error) => {
                    // Collect error metrics
                    let _ = metrics_collector.collect_api_error(
                        &request_info,
                        error,
                        duration,
                    ).await;
                    
                    // Complete trace with error
                    tracing_service.complete_request_trace_with_error(trace_context, error).await;
                }
            }
            
            result
        })
    }
}
```

### **Provider Monitoring Integration**

```rust
// Provider wrapper with monitoring
pub struct MonitoredProvider<T: Provider> {
    inner: T,
    metrics_collector: Arc<MetricsCollector>,
    tracing_service: Arc<TracingService>,
    health_tracker: Arc<ProviderHealthTracker>,
}

impl<T: Provider> Provider for MonitoredProvider<T> {
    async fn research_query(&self, query: String) -> ProviderResult<String> {
        let start_time = Utc::now();
        let provider_name = self.inner.metadata().name();
        
        // Start provider trace
        let trace_span = self.tracing_service
            .start_provider_trace(provider_name, "research_query")
            .await;
        
        // Execute query with monitoring
        let result = self.inner.research_query(query).await;
        let end_time = Utc::now();
        let duration = end_time.signed_duration_since(start_time).to_std().unwrap_or_default();
        
        // Collect metrics
        let operation_result = ProviderOperationResult {
            success: result.is_ok(),
            duration,
            tokens_used: self.estimate_tokens_used(&result),
            cost_usd: self.estimate_cost(&result),
            quality_score: None, // Will be filled by quality evaluation
            error_type: result.as_ref().err().map(|e| e.to_string()),
            rate_limited: matches!(result, Err(ProviderError::RateLimitExceeded { .. })),
        };
        
        let _ = self.metrics_collector.collect_provider_metrics(
            provider_name,
            "research_query",
            &operation_result,
        ).await;
        
        // Update health tracking
        self.health_tracker.record_operation_result(
            provider_name,
            &operation_result,
        ).await;
        
        // Complete trace
        self.tracing_service.complete_provider_trace(
            trace_span,
            &operation_result,
        ).await;
        
        result
    }
    
    fn metadata(&self) -> ProviderMetadata {
        self.inner.metadata()
    }
    
    async fn health_check(&self) -> ProviderResult<HealthStatus> {
        let start_time = Utc::now();
        let result = self.inner.health_check().await;
        let duration = Utc::now().signed_duration_since(start_time).to_std().unwrap_or_default();
        
        // Record health check metrics
        let _ = self.metrics_collector.collect_health_check_metrics(
            self.inner.metadata().name(),
            &result,
            duration,
        ).await;
        
        result
    }
}
```

## <configuration>Configuration Management</configuration>

### **Monitoring Configuration**

```yaml
monitoring:
  core:
    enable_metrics: true
    enable_tracing: true
    enable_health_checks: true
    enable_alerts: true
    
  collection:
    metrics_interval_seconds: 10
    health_check_interval_seconds: 30
    trace_sampling_rate: 0.1  # 10% sampling
    max_metrics_in_memory: 10000
    metrics_retention_hours: 24
    
  storage:
    metrics_storage_type: "prometheus"
    trace_storage_type: "jaeger"
    health_storage_type: "postgresql"
    
    prometheus:
      endpoint: "http://prometheus:9090"
      push_gateway: "http://pushgateway:9091"
      
    jaeger:
      endpoint: "http://jaeger:14268/api/traces"
      sampling_endpoint: "http://jaeger:5778/sampling"
      
    postgresql:
      connection_string: "${POSTGRES_URL}"
      health_table: "component_health"
      
  alerting:
    enable_email: true
    enable_webhooks: true
    enable_slack: false
    
    rate_limit_per_hour: 50
    
    email:
      smtp_server: "smtp.company.com"
      smtp_port: 587
      username: "${SMTP_USERNAME}"
      password: "${SMTP_PASSWORD}"
      from_address: "fortitude-alerts@company.com"
      to_addresses:
        - "devops@company.com"
        - "sre@company.com"
        
    webhooks:
      - url: "https://alerts.company.com/webhook/fortitude"
        timeout_seconds: 10
        retry_attempts: 3
        
  performance:
    thresholds:
      api_response_time_ms: 200
      provider_response_time_ms: 5000
      quality_evaluation_time_ms: 100
      cache_hit_ratio: 0.8
      max_error_rate: 0.05
      max_memory_usage_mb: 2048
      max_cpu_usage_percent: 80
      
  dashboard:
    enable_web_dashboard: true
    dashboard_port: 8081
    update_interval_seconds: 5
    enable_real_time_updates: true
    
  export:
    enable_prometheus_export: true
    enable_grafana_integration: true
    enable_custom_exports: false
    
    prometheus_port: 9090
    prometheus_path: "/metrics"
    
    grafana:
      url: "http://grafana:3000"
      api_key: "${GRAFANA_API_KEY}"
      dashboard_folder: "Fortitude"
```

### **Component-Specific Configuration**

```yaml
component_monitoring:
  api_server:
    enable_request_tracing: true
    enable_response_time_tracking: true
    enable_error_rate_monitoring: true
    track_user_agents: true
    track_client_ips: false  # Privacy consideration
    
  providers:
    openai:
      track_token_usage: true
      track_cost: true
      track_quality_scores: true
      health_check_endpoint: "/health"
      
    claude:
      track_token_usage: true
      track_cost: true
      track_quality_scores: true
      health_check_interval_seconds: 60
      
    gemini:
      track_token_usage: true
      track_cost: false  # Cost tracking not available
      track_quality_scores: true
      
  quality_system:
    track_evaluation_time: true
    track_dimension_scores: true
    track_confidence_levels: true
    enable_quality_trend_analysis: true
    
  learning_system:
    track_feedback_volume: true
    track_pattern_recognition_accuracy: true
    track_adaptation_success_rate: true
    enable_learning_progress_monitoring: true
    
  cache_system:
    track_hit_ratio: true
    track_eviction_rate: true
    track_memory_usage: true
    enable_cache_performance_analysis: true
```

## <monitoring>Real-time Monitoring Dashboard</monitoring>

### **Dashboard Components**

```rust
pub struct MonitoringDashboard {
    metrics_collector: Arc<MetricsCollector>,
    health_checker: Arc<HealthChecker>,
    alert_manager: Arc<AlertManager>,
    websocket_server: Arc<WebSocketServer>,
}

impl MonitoringDashboard {
    pub async fn start_dashboard_server(&self, port: u16) -> MonitoringResult<()> {
        let app = App::new()
            .route("/api/metrics/system", web::get().to(self.get_system_metrics()))
            .route("/api/metrics/providers", web::get().to(self.get_provider_metrics()))
            .route("/api/health", web::get().to(self.get_system_health()))
            .route("/api/alerts", web::get().to(self.get_active_alerts()))
            .route("/ws/live-metrics", web::get().to(self.websocket_handler()));
        
        HttpServer::new(move || app.clone())
            .bind(("0.0.0.0", port))?
            .run()
            .await?;
        
        Ok(())
    }
    
    async fn get_system_metrics(&self) -> Result<Json<SystemMetricsResponse>, ApiError> {
        let metrics = self.metrics_collector
            .get_system_performance_summary(Duration::from_secs(3600)) // Last hour
            .await?;
        
        Ok(Json(SystemMetricsResponse {
            timestamp: Utc::now(),
            api_performance: metrics.api_performance,
            provider_performance: metrics.provider_performance,
            quality_metrics: metrics.quality_metrics,
            resource_utilization: metrics.resource_utilization,
            cache_performance: metrics.cache_performance,
        }))
    }
    
    async fn get_provider_metrics(&self) -> Result<Json<ProviderMetricsResponse>, ApiError> {
        let provider_metrics = self.metrics_collector
            .get_provider_performance_comparison(Duration::from_secs(3600))
            .await?;
        
        Ok(Json(ProviderMetricsResponse {
            timestamp: Utc::now(),
            providers: provider_metrics,
        }))
    }
    
    async fn websocket_handler(&self, req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
        let (response, session, msg_stream) = actix_ws::handle(&req, stream)?;
        
        // Start real-time metrics streaming
        let metrics_collector = Arc::clone(&self.metrics_collector);
        actix_web::rt::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                let live_metrics = metrics_collector
                    .get_live_metrics()
                    .await
                    .unwrap_or_default();
                
                let message = serde_json::to_string(&live_metrics).unwrap();
                
                if session.text(message).await.is_err() {
                    break;
                }
            }
        });
        
        Ok(response)
    }
}
```

### **Grafana Dashboard Integration**

```json
{
  "dashboard": {
    "title": "Fortitude System Overview",
    "panels": [
      {
        "title": "API Response Times",
        "type": "graph",
        "targets": [
          {
            "expr": "avg(fortitude_api_response_time_seconds)",
            "legendFormat": "Average Response Time"
          }
        ]
      },
      {
        "title": "Provider Performance",
        "type": "table",
        "targets": [
          {
            "expr": "fortitude_provider_success_rate by (provider)",
            "legendFormat": "{{provider}} Success Rate"
          }
        ]
      },
      {
        "title": "Quality Score Distribution",
        "type": "histogram",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, fortitude_quality_score_bucket)",
            "legendFormat": "95th Percentile Quality Score"
          }
        ]
      },
      {
        "title": "System Health Status",
        "type": "stat",
        "targets": [
          {
            "expr": "fortitude_component_health_status",
            "legendFormat": "Component Health"
          }
        ]
      }
    ]
  }
}
```

## <deployment>Deployment Configuration</deployment>

### **Container Deployment with Monitoring**

```dockerfile
# Monitoring-enabled Fortitude deployment
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release --features "monitoring,tracing,alerting"

FROM debian:bookworm-slim

# Install monitoring dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/fortitude /usr/local/bin/
COPY --from=builder /app/config/monitoring/ /etc/fortitude/monitoring/

# Monitoring ports
EXPOSE 8080 8081 9090

# Health check endpoint
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8081/api/health || exit 1

CMD ["fortitude", "--config", "/etc/fortitude/monitoring/production.yaml"]
```

### **Kubernetes Monitoring Stack**

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: fortitude-monitoring-config
data:
  monitoring.yaml: |
    monitoring:
      enable_metrics: true
      enable_tracing: true
      prometheus_endpoint: "http://prometheus-server:9090"
      jaeger_endpoint: "http://jaeger-collector:14268"
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: fortitude-with-monitoring
spec:
  replicas: 3
  selector:
    matchLabels:
      app: fortitude
  template:
    metadata:
      labels:
        app: fortitude
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
        prometheus.io/path: "/metrics"
    spec:
      containers:
      - name: fortitude
        image: fortitude:monitoring-latest
        ports:
        - containerPort: 8080
          name: api
        - containerPort: 8081
          name: dashboard
        - containerPort: 9090
          name: metrics
        env:
        - name: FORTITUDE_MONITORING_CONFIG
          value: "/etc/fortitude/monitoring.yaml"
        - name: JAEGER_AGENT_HOST
          value: "jaeger-agent"
        - name: PROMETHEUS_GATEWAY
          value: "prometheus-pushgateway:9091"
        volumeMounts:
        - name: monitoring-config
          mountPath: /etc/fortitude
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /api/health
            port: 8081
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /api/health
            port: 8081
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: monitoring-config
        configMap:
          name: fortitude-monitoring-config
---
apiVersion: v1
kind: Service
metadata:
  name: fortitude-monitoring-service
  labels:
    app: fortitude
spec:
  ports:
  - port: 8080
    targetPort: 8080
    name: api
  - port: 8081
    targetPort: 8081
    name: dashboard
  - port: 9090
    targetPort: 9090
    name: metrics
  selector:
    app: fortitude
```

## <troubleshooting>Common Issues and Solutions</troubleshooting>

### **Monitoring Performance Issues**

<troubleshooting-guide>

**Issue**: High monitoring overhead impacting performance
```rust
// Solution: Implement adaptive sampling and efficient aggregation
pub struct AdaptiveMonitoringCollector {
    sampling_rate: Arc<AtomicU64>,
    performance_tracker: Arc<PerformanceTracker>,
}

impl AdaptiveMonitoringCollector {
    pub async fn collect_metric_with_adaptive_sampling(&self, metric: &Metric) -> MonitoringResult<()> {
        // Check current system load
        let current_load = self.performance_tracker.get_current_load().await;
        
        // Adjust sampling rate based on load
        let sampling_rate = if current_load > 0.8 {
            0.1 // Reduce sampling under high load
        } else if current_load > 0.5 {
            0.5 // Normal sampling
        } else {
            1.0 // Full sampling under low load
        };
        
        // Sample based on calculated rate
        if fastrand::f64() < sampling_rate {
            self.store_metric(metric).await?;
        }
        
        Ok(())
    }
}
```

**Issue**: Alert fatigue from too many notifications
```rust
// Solution: Implement intelligent alert grouping and rate limiting
pub struct IntelligentAlertManager {
    alert_groups: Arc<Mutex<HashMap<String, AlertGroup>>>,
    rate_limiter: Arc<AlertRateLimiter>,
}

impl IntelligentAlertManager {
    pub async fn send_alert_with_grouping(&self, alert: Alert) -> MonitoringResult<()> {
        let group_key = self.calculate_group_key(&alert);
        
        {
            let mut groups = self.alert_groups.lock().await;
            if let Some(group) = groups.get_mut(&group_key) {
                group.add_alert(alert);
                
                // Send grouped alert if threshold reached
                if group.should_send_grouped_alert() {
                    let grouped_alert = group.create_grouped_alert();
                    self.send_immediate_alert(grouped_alert).await?;
                    group.reset();
                }
            } else {
                // First alert in group
                let mut group = AlertGroup::new(group_key.clone());
                group.add_alert(alert.clone());
                groups.insert(group_key, group);
                
                // Send immediate alert for critical issues
                if matches!(alert.severity, AlertSeverity::Critical) {
                    self.send_immediate_alert(alert).await?;
                }
            }
        }
        
        Ok(())
    }
}
```

**Issue**: Distributed tracing causing memory leaks
```rust
// Solution: Implement proper span lifecycle management
pub struct ManagedTracingService {
    span_registry: Arc<RwLock<HashMap<TraceId, SpanInfo>>>,
    cleanup_scheduler: Arc<CleanupScheduler>,
}

impl ManagedTracingService {
    pub async fn start_managed_span(&self, operation: &str) -> ManagedSpan {
        let span_info = SpanInfo {
            start_time: Utc::now(),
            operation: operation.to_string(),
            ttl: Duration::from_secs(300), // 5 minute TTL
        };
        
        let trace_id = TraceId::new();
        
        {
            let mut registry = self.span_registry.write().await;
            registry.insert(trace_id, span_info);
        }
        
        // Schedule cleanup
        self.cleanup_scheduler.schedule_span_cleanup(trace_id, Duration::from_secs(300)).await;
        
        ManagedSpan::new(trace_id, self.span_registry.clone())
    }
    
    pub async fn cleanup_expired_spans(&self) -> usize {
        let now = Utc::now();
        let mut registry = self.span_registry.write().await;
        let initial_count = registry.len();
        
        registry.retain(|_, span_info| {
            now.signed_duration_since(span_info.start_time) < span_info.ttl.into()
        });
        
        initial_count - registry.len()
    }
}
```

</troubleshooting-guide>

## <capabilities>Advanced Monitoring Capabilities</capabilities>

### **Implemented Features**

1. **Machine Learning Anomaly Detection**: AI-powered detection of unusual patterns
2. **Predictive Alerting**: Forecasts potential issues before they occur
3. **Automated Remediation**: Self-healing capabilities for common issues
4. **Multi-Cloud Monitoring**: Unified monitoring across cloud providers
5. **Edge Monitoring**: Performance monitoring for edge deployments

### **System Integration**

- **Core Monitoring**: Comprehensive metrics, tracing, and alerting system
- **Performance Analysis**: Advanced performance monitoring and dashboard visualization
- **Intelligence Layer**: ML-powered anomaly detection and predictive alerting
- **Automation**: Automated remediation and self-healing capabilities

## <references>See Also</references>

- [Multi-LLM Architecture](multi-llm-architecture.md) - Provider monitoring integration
- [Quality Control Design](quality-control-design.md) - Quality metrics integration
- [Learning System Design](learning-system-design.md) - Learning metrics integration
- [Monitoring Configuration Guide](../user-guides/learning-and-monitoring-configuration.md) - Setup guide
- [Performance Tuning](../performance/tuning-guide.md) - Optimization guide
- [Troubleshooting Guide](../troubleshooting/monitoring-issues.md) - Issue resolution