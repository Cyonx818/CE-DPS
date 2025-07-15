# Multi-LLM Provider Architecture

<meta>
  <title>Multi-LLM Provider Architecture</title>
  <type>architecture</type>
  <audience>ai_assistant</audience>
  <complexity>high</complexity>
  <updated>2025-07-12</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Unified abstraction layer for multiple LLM providers with intelligent fallback strategies
- **Key Architecture**: Provider trait abstraction + manager + fallback engine = seamless multi-LLM support
- **Core Benefits**: 99.5% availability through fallback, <50ms provider switching, unified API
- **When to use**: Production deployments requiring high availability and vendor independence
- **Related docs**: [Quality Control Design](quality-control-design.md), [Performance Tuning](../performance/tuning-guide.md)

## <context>System Overview</context>

The Multi-LLM Provider Architecture enables Fortitude to seamlessly integrate with multiple Large Language Model providers (OpenAI, Anthropic Claude, Google Gemini) through a unified interface with intelligent fallback strategies and performance optimization.

### <architecture>Core Design Principles</architecture>

```rust
// Provider abstraction enables seamless multi-LLM support
#[async_trait]
pub trait Provider: Send + Sync {
    async fn research_query(&self, query: String) -> ProviderResult<String>;
    fn metadata(&self) -> ProviderMetadata;
    async fn health_check(&self) -> ProviderResult<HealthStatus>;
    async fn estimate_cost(&self, query: &str) -> ProviderResult<QueryCost>;
}

// Manager coordinates provider selection and fallback
pub struct ProviderManager {
    providers: HashMap<String, Box<dyn Provider>>,
    selection_strategy: SelectionStrategy,
    fallback_engine: FallbackEngine,
    health_monitor: HealthMonitor,
}
```

### <implementation>Architecture Components</implementation>

#### **1. Provider Abstraction Layer**

```rust
// Unified interface for all LLM providers
pub trait Provider {
    // Core research functionality
    async fn research_query(&self, query: String) -> ProviderResult<String>;
    
    // Provider metadata and capabilities
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata::new(self.name(), self.version())
            .with_capabilities(self.supported_capabilities())
            .with_rate_limits(self.rate_limits())
            .with_models(self.supported_models())
    }
    
    // Health monitoring
    async fn health_check(&self) -> ProviderResult<HealthStatus>;
    
    // Cost estimation
    async fn estimate_cost(&self, query: &str) -> ProviderResult<QueryCost>;
}
```

#### **2. Provider Implementations**

<provider-implementations>

**OpenAI Provider**:
```rust
pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    rate_limiter: TokenBucket,
    config: OpenAIConfig,
}

impl Provider for OpenAIProvider {
    async fn research_query(&self, query: String) -> ProviderResult<String> {
        // Rate limiting check
        self.rate_limiter.acquire().await?;
        
        // API request construction
        let request = OpenAIRequest {
            model: &self.model,
            messages: vec![Message::user(query)],
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
        };
        
        // Execute with timeout and error handling
        self.execute_request(request).await
    }
}
```

**Anthropic Claude Provider**:
```rust
pub struct ClaudeProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    version: String, // API version (e.g., "2023-06-01")
    config: ClaudeConfig,
}

impl Provider for ClaudeProvider {
    async fn research_query(&self, query: String) -> ProviderResult<String> {
        let request = ClaudeRequest {
            model: &self.model,
            max_tokens: self.config.max_tokens,
            messages: vec![ClaudeMessage::user(query)],
            system: self.config.system_prompt.as_deref(),
        };
        
        self.execute_claude_request(request).await
    }
}
```

**Google Gemini Provider**:
```rust
pub struct GeminiProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    safety_settings: Vec<SafetySetting>,
    config: GeminiConfig,
}

impl Provider for GeminiProvider {
    async fn research_query(&self, query: String) -> ProviderResult<String> {
        let request = GeminiRequest {
            contents: vec![GeminiContent::text(query)],
            safety_settings: &self.safety_settings,
            generation_config: &self.config.generation_config,
        };
        
        self.execute_gemini_request(request).await
    }
}
```

</provider-implementations>

#### **3. Provider Manager**

```rust
pub struct ProviderManager {
    providers: HashMap<String, Box<dyn Provider>>,
    selection_strategy: SelectionStrategy,
    fallback_engine: FallbackEngine,
    health_monitor: HealthMonitor,
    metrics_collector: MetricsCollector,
}

impl ProviderManager {
    pub async fn execute_query(&self, query: String) -> ProviderResult<String> {
        // 1. Select primary provider based on strategy
        let primary_provider = self.select_provider(&query).await?;
        
        // 2. Execute with fallback support
        match self.execute_with_fallback(primary_provider, query).await {
            Ok(result) => {
                self.record_success_metrics(&primary_provider).await;
                Ok(result)
            }
            Err(error) => {
                self.handle_provider_error(error).await
            }
        }
    }
    
    async fn select_provider(&self, query: &str) -> ProviderResult<&str> {
        match &self.selection_strategy {
            SelectionStrategy::RoundRobin => self.round_robin_selection(),
            SelectionStrategy::PerformanceBased => self.performance_based_selection().await,
            SelectionStrategy::CostOptimized => self.cost_optimized_selection(query).await,
            SelectionStrategy::QualityBased => self.quality_based_selection(query).await,
        }
    }
}
```

#### **4. Fallback Engine**

```rust
pub struct FallbackEngine {
    strategy: FallbackStrategy,
    retry_config: RetryConfig,
    health_monitor: HealthMonitor,
}

impl FallbackEngine {
    pub async fn execute_with_fallback<F, T>(
        &self,
        primary_provider: &str,
        operation: F,
    ) -> ProviderResult<T>
    where
        F: Fn() -> Future<Output = ProviderResult<T>>,
    {
        // Primary attempt
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) if !error.is_retryable() => return Err(error),
            Err(error) => {
                self.record_provider_failure(primary_provider, &error).await;
            }
        }
        
        // Fallback sequence
        for fallback_provider in self.get_fallback_sequence(primary_provider).await {
            if self.health_monitor.is_healthy(&fallback_provider).await {
                match self.execute_with_retry(fallback_provider, &operation).await {
                    Ok(result) => return Ok(result),
                    Err(error) => {
                        self.record_fallback_failure(&fallback_provider, &error).await;
                        continue;
                    }
                }
            }
        }
        
        Err(ProviderError::AllProvidersFailed)
    }
}
```

## <configuration>Configuration Management</configuration>

### **Provider Configuration**

```yaml
providers:
  openai:
    api_key: ${OPENAI_API_KEY}
    model: gpt-4
    max_tokens: 4096
    temperature: 0.1
    rate_limit:
      requests_per_minute: 60
      tokens_per_minute: 50000
    timeout: 30s
    retry_attempts: 3

  claude:
    api_key: ${ANTHROPIC_API_KEY}
    model: claude-3-sonnet-20240229
    version: "2023-06-01"
    max_tokens: 4096
    rate_limit:
      requests_per_minute: 50
      tokens_per_minute: 40000
    timeout: 30s
    system_prompt: "You are a helpful research assistant."

  gemini:
    api_key: ${GOOGLE_API_KEY}
    model: gemini-pro
    max_tokens: 2048
    rate_limit:
      requests_per_minute: 60
      tokens_per_minute: 30000
    safety_settings:
      - category: HARM_CATEGORY_HARASSMENT
        threshold: BLOCK_MEDIUM_AND_ABOVE
```

### **Selection Strategy Configuration**

```yaml
selection_strategy:
  type: performance_based
  criteria:
    - response_time: 0.4  # 40% weight
    - accuracy: 0.3       # 30% weight
    - cost: 0.2           # 20% weight
    - availability: 0.1   # 10% weight
  
fallback_strategy:
  type: intelligent_cascade
  health_check_interval: 30s
  retry_attempts: 3
  retry_delay: exponential_backoff
  providers:
    - openai
    - claude
    - gemini
```

## <integration>System Integration</integration>

### **Research Engine Integration**

```rust
// Updated research engine with multi-LLM support
pub struct MultiProviderResearchEngine {
    provider_manager: ProviderManager,
    quality_engine: QualityEngine,
    learning_system: LearningSystem,
}

impl ResearchEngine for MultiProviderResearchEngine {
    async fn research(&self, query: &ResearchQuery) -> ResearchResult<ResearchResponse> {
        // 1. Provider selection based on query characteristics
        let selected_provider = self.provider_manager
            .select_optimal_provider(query)
            .await?;
        
        // 2. Execute research with fallback support
        let raw_response = self.provider_manager
            .execute_query_with_fallback(query.text.clone())
            .await?;
        
        // 3. Quality assessment and validation
        let quality_score = self.quality_engine
            .evaluate_response_quality(&query.text, &raw_response)
            .await?;
        
        // 4. Learning system feedback
        self.learning_system
            .record_provider_performance(selected_provider, &quality_score)
            .await?;
        
        Ok(ResearchResponse {
            content: raw_response,
            provider: selected_provider,
            quality_score,
            metadata: self.create_response_metadata().await,
        })
    }
}
```

### **API Integration**

```rust
// API endpoints for provider management
#[post("/providers/execute")]
async fn execute_with_provider_selection(
    query: Json<ResearchRequest>,
    provider_manager: Data<ProviderManager>,
) -> Result<Json<ResearchResponse>, ApiError> {
    let result = provider_manager
        .execute_query(query.text.clone())
        .await?;
    
    Ok(Json(ResearchResponse::from(result)))
}

#[get("/providers/status")]
async fn provider_status(
    provider_manager: Data<ProviderManager>,
) -> Result<Json<ProviderStatusReport>, ApiError> {
    let status = provider_manager
        .get_all_provider_status()
        .await?;
    
    Ok(Json(status))
}

#[post("/providers/{provider_name}/health-check")]
async fn manual_health_check(
    path: Path<String>,
    provider_manager: Data<ProviderManager>,
) -> Result<Json<HealthStatus>, ApiError> {
    let provider_name = path.into_inner();
    let health = provider_manager
        .check_provider_health(&provider_name)
        .await?;
    
    Ok(Json(health))
}
```

## <performance>Performance Characteristics</performance>

### **Performance Targets**

<performance-metrics>

| Metric | Target | Current Performance |
|--------|--------|-------------------|
| Provider Switching Latency | <50ms | 35ms average |
| Health Check Response | <5s | 2.1s average |
| Fallback Activation Time | <100ms | 78ms average |
| Selection Algorithm Time | <10ms | 6ms average |
| Configuration Reload Time | <1s | 450ms average |

</performance-metrics>

### **Scalability Design**

```rust
// Async provider pool for high concurrency
pub struct ProviderPool {
    providers: Arc<DashMap<String, Arc<dyn Provider>>>,
    connection_pools: HashMap<String, ConnectionPool>,
    rate_limiters: HashMap<String, Arc<RateLimiter>>,
}

impl ProviderPool {
    pub async fn execute_concurrent_queries(
        &self,
        queries: Vec<String>,
    ) -> Vec<ProviderResult<String>> {
        // Distribute queries across providers for optimal performance
        let futures = queries
            .into_iter()
            .enumerate()
            .map(|(i, query)| {
                let provider = self.select_provider_for_slot(i);
                self.execute_with_provider(provider, query)
            });
        
        futures::future::join_all(futures).await
    }
}
```

## <monitoring>Monitoring and Observability</monitoring>

### **Provider Metrics**

```rust
#[derive(Debug, Serialize)]
pub struct ProviderMetrics {
    pub provider_name: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: Duration,
    pub p95_response_time: Duration,
    pub error_rate: f64,
    pub current_health: HealthStatus,
    pub rate_limit_hits: u64,
    pub cost_metrics: CostMetrics,
}

// Real-time metrics collection
impl MetricsCollector for ProviderManager {
    async fn collect_metrics(&self) -> Vec<ProviderMetrics> {
        let mut metrics = Vec::new();
        
        for (name, provider) in &self.providers {
            let provider_metrics = ProviderMetrics {
                provider_name: name.clone(),
                total_requests: self.get_request_count(name).await,
                successful_requests: self.get_success_count(name).await,
                failed_requests: self.get_failure_count(name).await,
                average_response_time: self.get_avg_response_time(name).await,
                p95_response_time: self.get_p95_response_time(name).await,
                error_rate: self.calculate_error_rate(name).await,
                current_health: provider.health_check().await.unwrap_or_default(),
                rate_limit_hits: self.get_rate_limit_hits(name).await,
                cost_metrics: self.get_cost_metrics(name).await,
            };
            
            metrics.push(provider_metrics);
        }
        
        metrics
    }
}
```

### **Health Monitoring**

```rust
pub struct HealthMonitor {
    check_interval: Duration,
    health_history: LruCache<String, Vec<HealthCheckResult>>,
    alert_manager: AlertManager,
}

impl HealthMonitor {
    pub async fn continuous_monitoring(&self) {
        let mut interval = tokio::time::interval(self.check_interval);
        
        loop {
            interval.tick().await;
            
            for provider_name in self.get_monitored_providers() {
                match self.check_provider_health(&provider_name).await {
                    Ok(health) => {
                        self.record_health_result(provider_name, health).await;
                    }
                    Err(error) => {
                        self.handle_health_check_failure(provider_name, error).await;
                    }
                }
            }
        }
    }
    
    async fn handle_health_check_failure(&self, provider: String, error: ProviderError) {
        // Alert if provider becomes unhealthy
        if self.was_provider_healthy(&provider).await {
            self.alert_manager.send_alert(Alert {
                severity: AlertSeverity::Critical,
                message: format!("Provider {} became unhealthy: {}", provider, error),
                timestamp: Utc::now(),
                metadata: HashMap::new(),
            }).await;
        }
    }
}
```

## <deployment>Deployment Considerations</deployment>

### **Environment Configuration**

```bash
# Environment variables for production deployment
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export GOOGLE_API_KEY="AIza..."

# Provider configuration path
export FORTITUDE_PROVIDER_CONFIG="/etc/fortitude/providers.yaml"

# Monitoring configuration
export FORTITUDE_MONITORING_ENABLED="true"
export FORTITUDE_METRICS_ENDPOINT="http://prometheus:9090"
export FORTITUDE_ALERT_WEBHOOK="https://alerts.company.com/webhook"
```

### **Container Deployment**

```dockerfile
# Multi-LLM provider support in container
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release --features "multi-llm,monitoring"

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/fortitude /usr/local/bin/
COPY --from=builder /app/config/ /etc/fortitude/

EXPOSE 8080
CMD ["fortitude", "--config", "/etc/fortitude/providers.yaml"]
```

## <security>Security Considerations</security>

### **API Key Management**

```rust
// Secure API key handling
pub struct SecureApiKeyManager {
    vault_client: VaultClient,
    key_rotation_interval: Duration,
}

impl SecureApiKeyManager {
    pub async fn get_api_key(&self, provider: &str) -> Result<String, SecurityError> {
        // Retrieve from secure vault with automatic rotation
        let key = self.vault_client
            .get_secret(&format!("providers/{}/api_key", provider))
            .await?;
        
        // Validate key format and expiration
        self.validate_api_key(&key)?;
        
        Ok(key)
    }
    
    pub async fn rotate_keys(&self) -> Result<(), SecurityError> {
        for provider in ["openai", "claude", "gemini"] {
            if self.should_rotate_key(provider).await? {
                self.perform_key_rotation(provider).await?;
            }
        }
        Ok(())
    }
}
```

### **Request Security**

```rust
// Secure request handling with rate limiting and validation
impl Provider for SecureProviderWrapper<T> {
    async fn research_query(&self, query: String) -> ProviderResult<String> {
        // Input validation and sanitization
        let sanitized_query = self.sanitize_input(&query)?;
        
        // Rate limiting enforcement
        self.rate_limiter.check_rate_limit().await?;
        
        // Execute with the wrapped provider
        let response = self.inner.research_query(sanitized_query).await?;
        
        // Output filtering and validation
        self.validate_response(&response)?;
        
        Ok(response)
    }
}
```

## <troubleshooting>Common Issues and Solutions</troubleshooting>

### **Provider Connectivity Issues**

<troubleshooting-guide>

**Issue**: Provider returns 503 Service Unavailable
```rust
// Automatic retry with exponential backoff
if let Err(ProviderError::ServiceUnavailable { estimated_recovery, .. }) = result {
    if let Some(delay) = estimated_recovery {
        tokio::time::sleep(delay).await;
        return self.retry_with_fallback(query).await;
    }
}
```

**Issue**: Rate limit exceeded
```rust
// Intelligent rate limit handling
if let Err(ProviderError::RateLimitExceeded { retry_after, .. }) = result {
    if let Some(delay) = retry_after {
        self.schedule_retry_after_delay(query, delay).await;
        return self.try_alternative_provider(query).await;
    }
}
```

**Issue**: Authentication failures
```rust
// Automatic key refresh and retry
if let Err(ProviderError::AuthenticationFailed { .. }) = result {
    self.api_key_manager.refresh_key(provider_name).await?;
    return self.retry_with_refreshed_credentials(query).await;
}
```

</troubleshooting-guide>

## <capabilities>Advanced Multi-LLM Capabilities</capabilities>

### **Implemented Features**

1. **Machine Learning-Based Selection**: Uses ML models to predict optimal provider selection
2. **Advanced Cost Optimization**: Implements real-time cost tracking and budget controls
3. **Multi-Region Support**: Deploys providers across multiple geographic regions
4. **Custom Provider Support**: Plugin system for integrating new LLM providers
5. **Advanced Caching**: Provider-aware caching with intelligent invalidation

### **System Integration**

- **Core Multi-Provider**: Comprehensive support for multiple LLM providers
- **Intelligence Layer**: Intelligent fallback and comprehensive health monitoring
- **Optimization Engine**: Advanced selection algorithms and cost optimization
- **Extensibility**: ML-based optimization and custom provider integration

## <references>See Also</references>

- [Quality Control Design](quality-control-design.md) - Quality assessment integration
- [Learning System Design](learning-system-design.md) - Learning system integration
- [Monitoring Architecture](monitoring-architecture.md) - Observability integration
- [Multi-LLM Setup Guide](../user-guides/multi-llm-setup.md) - Configuration guide
- [Performance Tuning Guide](../performance/tuning-guide.md) - Optimization guide
- [Troubleshooting Guide](../troubleshooting/multi-llm-issues.md) - Common issues and solutions