# Multi-LLM Provider Setup and Configuration Guide

<meta>
  <title>Multi-LLM Provider Setup and Configuration Guide</title>
  <type>user_guide</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-12</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Complete setup guide for configuring multiple LLM providers with intelligent fallback strategies
- **Key Steps**: API key setup + provider configuration + fallback strategy + monitoring = production-ready multi-LLM system
- **Core Benefits**: 99.5% availability, vendor independence, cost optimization, automatic failover
- **Time to Setup**: 15-30 minutes for basic configuration, 1-2 hours for advanced optimization
- **Related docs**: [Multi-LLM Architecture](../architecture/multi-llm-architecture.md), [Deployment Guide](deployment-and-integration-guide.md)

## <context>Overview</context>

This guide walks you through setting up Fortitude's multi-LLM provider system, enabling seamless integration with OpenAI, Anthropic Claude, and Google Gemini with intelligent fallback strategies and cost optimization.

## <setup>Prerequisites</setup>

### **API Keys Required**

<credentials-setup>

**OpenAI API Key**:
1. Visit [OpenAI Platform](https://platform.openai.com/api-keys)
2. Create new API key with appropriate usage limits
3. Note: Requires billing setup for production use

**Anthropic Claude API Key**:
1. Visit [Anthropic Console](https://console.anthropic.com/)
2. Generate API key from the API Keys section
3. Note: May require waitlist approval

**Google Gemini API Key**:
1. Visit [Google AI Studio](https://makersuite.google.com/app/apikey)
2. Create new API key in your project
3. Enable the Generative AI API

</credentials-setup>

### **Environment Setup**

```bash
# Set environment variables for API keys
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export GOOGLE_API_KEY="AIza..."

# Optional: Set configuration file path
export FORTITUDE_PROVIDER_CONFIG="/path/to/providers.yaml"
```

### **System Requirements**

- **Memory**: Minimum 2GB RAM for multi-provider operation
- **Network**: Stable internet connection for API calls
- **Storage**: 100MB for configuration and cache storage
- **OS**: Linux, macOS, or Windows (Docker recommended)

## <configuration>Basic Configuration</configuration>

### **1. Provider Configuration File**

Create `providers.yaml` with basic multi-LLM setup:

```yaml
# Basic multi-LLM provider configuration
providers:
  openai:
    enabled: true
    api_key: ${OPENAI_API_KEY}
    model: "gpt-4"
    base_url: "https://api.openai.com/v1"
    
    # Basic settings
    max_tokens: 4096
    temperature: 0.1
    timeout_seconds: 30
    
    # Rate limiting
    rate_limit:
      requests_per_minute: 60
      tokens_per_minute: 50000
      max_concurrent: 5
    
    # Cost settings
    cost_per_1k_input_tokens: 0.03
    cost_per_1k_output_tokens: 0.06

  claude:
    enabled: true
    api_key: ${ANTHROPIC_API_KEY}
    model: "claude-3-sonnet-20240229"
    base_url: "https://api.anthropic.com"
    version: "2023-06-01"
    
    # Basic settings
    max_tokens: 4096
    timeout_seconds: 30
    
    # Rate limiting
    rate_limit:
      requests_per_minute: 50
      tokens_per_minute: 40000
      max_concurrent: 3
    
    # Cost settings
    cost_per_1k_input_tokens: 0.003
    cost_per_1k_output_tokens: 0.015
    
    # Claude-specific settings
    system_prompt: "You are a helpful research assistant providing accurate, well-sourced information."

  gemini:
    enabled: true
    api_key: ${GOOGLE_API_KEY}
    model: "gemini-pro"
    base_url: "https://generativelanguage.googleapis.com/v1"
    
    # Basic settings
    max_tokens: 2048
    timeout_seconds: 30
    
    # Rate limiting
    rate_limit:
      requests_per_minute: 60
      tokens_per_minute: 30000
      max_concurrent: 5
    
    # Safety settings
    safety_settings:
      - category: "HARM_CATEGORY_HARASSMENT"
        threshold: "BLOCK_MEDIUM_AND_ABOVE"
      - category: "HARM_CATEGORY_HATE_SPEECH"
        threshold: "BLOCK_MEDIUM_AND_ABOVE"
      - category: "HARM_CATEGORY_SEXUALLY_EXPLICIT"
        threshold: "BLOCK_MEDIUM_AND_ABOVE"
      - category: "HARM_CATEGORY_DANGEROUS_CONTENT"
        threshold: "BLOCK_MEDIUM_AND_ABOVE"

# Provider selection strategy
selection_strategy:
  type: "round_robin"  # Options: round_robin, performance_based, cost_optimized
  
# Fallback configuration
fallback:
  enabled: true
  strategy: "intelligent_cascade"
  max_retries: 3
  retry_delay: "exponential_backoff"
  health_check_interval: 30
  
  # Provider priority order for fallback
  provider_order:
    - "openai"
    - "claude"
    - "gemini"
```

### **2. Quick Start Commands**

```bash
# Test basic configuration
fortitude providers test

# Verify all providers are accessible
fortitude providers health-check

# Test multi-provider research query
fortitude research --query "What is quantum computing?" --providers all

# Check provider status
fortitude providers status
```

## <advanced-configuration>Advanced Configuration</advanced-configuration>

### **Performance-Based Provider Selection**

```yaml
selection_strategy:
  type: "performance_based"
  
  # Weighting criteria for provider selection
  criteria:
    response_time: 0.3      # 30% weight on speed
    quality_score: 0.4      # 40% weight on quality
    cost_efficiency: 0.2    # 20% weight on cost
    availability: 0.1       # 10% weight on uptime
  
  # Performance tracking window
  tracking_window_hours: 24
  
  # Minimum data points for reliable scoring
  min_samples: 10

# Provider-specific optimization
provider_optimization:
  openai:
    # Optimal use cases
    preferred_for:
      - "code_generation"
      - "creative_writing"
      - "complex_reasoning"
    
    # Performance characteristics
    expected_response_time_ms: 2000
    quality_consistency: 0.92
    
  claude:
    preferred_for:
      - "analytical_research"
      - "fact_checking"
      - "structured_analysis"
    
    expected_response_time_ms: 1800
    quality_consistency: 0.94
    
  gemini:
    preferred_for:
      - "current_events"
      - "factual_queries"
      - "multimodal_content"
    
    expected_response_time_ms: 1500
    quality_consistency: 0.87
```

### **Cost Optimization Configuration**

```yaml
cost_optimization:
  enabled: true
  
  # Budget controls
  daily_budget_usd: 50.0
  monthly_budget_usd: 1000.0
  
  # Cost tracking
  track_usage: true
  alert_thresholds:
    daily_spend_percent: 80    # Alert at 80% of daily budget
    monthly_spend_percent: 90  # Alert at 90% of monthly budget
  
  # Cost-aware provider selection
  cost_strategy:
    type: "budget_aware"
    
    # Prefer cheaper providers when quality difference is minimal
    quality_tolerance: 0.05  # Switch to cheaper if quality diff < 5%
    
    # Provider cost ranking (1 = cheapest)
    cost_ranking:
      claude: 1
      gemini: 2
      openai: 3

# Token usage optimization
token_optimization:
  enabled: true
  
  # Automatic prompt optimization
  optimize_prompts: true
  max_prompt_length: 2000
  
  # Response length management
  adaptive_max_tokens: true
  default_max_tokens: 1000
  
  # Context compression
  enable_context_compression: true
  compression_threshold: 3000  # Compress contexts longer than 3000 tokens
```

### **Advanced Fallback Strategies**

```yaml
fallback:
  enabled: true
  strategy: "adaptive_cascade"
  
  # Health monitoring configuration
  health_monitoring:
    check_interval_seconds: 30
    failure_threshold: 3          # Mark unhealthy after 3 failures
    recovery_threshold: 2         # Mark healthy after 2 successes
    circuit_breaker_timeout: 300  # 5 minute circuit breaker
  
  # Intelligent fallback rules
  fallback_rules:
    # Rate limit handling
    - trigger: "rate_limit_exceeded"
      action: "switch_provider"
      wait_time: "retry_after_header"  # Use provider's retry-after header
      
    # Timeout handling
    - trigger: "timeout"
      action: "retry_with_fallback"
      max_retries: 2
      timeout_escalation: [5, 10, 20]  # Increase timeout on retries
      
    # Quality threshold handling
    - trigger: "quality_below_threshold"
      threshold: 0.6
      action: "cross_validate"
      validation_providers: 2
      
    # Error handling
    - trigger: "authentication_error"
      action: "skip_provider"
      duration: 3600  # Skip for 1 hour
      
  # Provider-specific fallback configuration
  provider_fallback:
    openai:
      fallback_to: ["claude", "gemini"]
      skip_conditions:
        - "rate_limit_exceeded"
        - "quota_exceeded"
      
    claude:
      fallback_to: ["openai", "gemini"]
      skip_conditions:
        - "authentication_error"
        - "service_unavailable"
      
    gemini:
      fallback_to: ["openai", "claude"]
      skip_conditions:
        - "safety_filter_triggered"
        - "content_policy_violation"
```

## <testing>Testing and Validation</testing>

### **Provider Health Checks**

```bash
# Comprehensive health check
fortitude providers health-check --verbose

# Individual provider testing
fortitude providers test --provider openai
fortitude providers test --provider claude  
fortitude providers test --provider gemini

# Test with specific query
fortitude providers test --query "Explain machine learning" --timeout 10

# Performance benchmarking
fortitude providers benchmark --queries 10 --concurrent 3
```

### **Fallback Testing**

```bash
# Test fallback mechanisms
fortitude test fallback --disable-provider openai

# Simulate provider failures
fortitude test failure-scenarios --scenario rate_limit
fortitude test failure-scenarios --scenario timeout
fortitude test failure-scenarios --scenario authentication

# Load testing with fallback
fortitude test load --requests 100 --concurrent 10 --enable-fallback
```

### **Quality Validation**

```bash
# Test quality consistency across providers
fortitude test quality --query "Compare renewable energy sources" --providers all

# Cross-validation testing
fortitude test cross-validation --queries examples/test-queries.txt

# Performance vs quality analysis
fortitude test performance-quality --duration 300  # 5 minute test
```

## <monitoring>Monitoring and Observability</monitoring>

### **Basic Monitoring Setup**

```yaml
monitoring:
  enabled: true
  
  # Metrics collection
  metrics:
    provider_performance: true
    cost_tracking: true
    quality_scores: true
    fallback_events: true
  
  # Health check monitoring
  health_checks:
    interval_seconds: 60
    store_history: true
    retention_days: 30
  
  # Alert configuration
  alerts:
    enable_email: false
    enable_webhook: true
    webhook_url: "https://your-monitoring.com/webhook"
    
    # Alert conditions
    conditions:
      - name: "provider_unavailable"
        condition: "provider_health == 'unhealthy'"
        duration: "5m"
        severity: "critical"
        
      - name: "high_cost_usage"
        condition: "daily_cost_usd > daily_budget_usd * 0.8"
        duration: "1m"
        severity: "warning"
        
      - name: "quality_degradation"
        condition: "avg_quality_score < 0.7"
        duration: "10m"
        severity: "warning"
```

### **Dashboard Access**

```bash
# Start monitoring dashboard
fortitude dashboard --port 8081

# View real-time metrics
curl http://localhost:8081/api/metrics/providers

# Get system health status
curl http://localhost:8081/api/health

# Provider comparison
curl http://localhost:8081/api/providers/comparison
```

## <integration>API Integration</integration>

### **REST API Usage**

```bash
# Multi-provider research request
curl -X POST http://localhost:8080/api/research \
  -H "Content-Type: application/json" \
  -d '{
    "query": "Explain quantum computing applications",
    "provider_preference": "auto",
    "quality_threshold": 0.8,
    "max_cost_usd": 0.10
  }'

# Provider-specific request
curl -X POST http://localhost:8080/api/research \
  -H "Content-Type: application/json" \
  -d '{
    "query": "Current AI trends",
    "provider": "claude",
    "fallback_enabled": true
  }'

# Get provider status
curl http://localhost:8080/api/providers/status

# Health check endpoint
curl http://localhost:8080/api/health
```

### **Python Client Example**

```python
import asyncio
import aiohttp

class FortitudeClient:
    def __init__(self, base_url="http://localhost:8080"):
        self.base_url = base_url
    
    async def research_with_multi_llm(
        self,
        query: str,
        provider_preference: str = "auto",
        quality_threshold: float = 0.8,
        max_cost_usd: float = None
    ):
        async with aiohttp.ClientSession() as session:
            payload = {
                "query": query,
                "provider_preference": provider_preference,
                "quality_threshold": quality_threshold,
            }
            
            if max_cost_usd:
                payload["max_cost_usd"] = max_cost_usd
            
            async with session.post(
                f"{self.base_url}/api/research",
                json=payload
            ) as response:
                return await response.json()
    
    async def get_provider_status(self):
        async with aiohttp.ClientSession() as session:
            async with session.get(
                f"{self.base_url}/api/providers/status"
            ) as response:
                return await response.json()

# Usage example
async def main():
    client = FortitudeClient()
    
    # Multi-provider research
    result = await client.research_with_multi_llm(
        query="Explain the benefits of renewable energy",
        provider_preference="performance_optimized",
        quality_threshold=0.85,
        max_cost_usd=0.05
    )
    
    print(f"Provider used: {result['provider_used']}")
    print(f"Quality score: {result['quality_score']}")
    print(f"Response: {result['content']}")
    
    # Check provider status
    status = await client.get_provider_status()
    for provider, info in status['providers'].items():
        print(f"{provider}: {info['status']}")

if __name__ == "__main__":
    asyncio.run(main())
```

### **JavaScript Client Example**

```javascript
class FortitudeClient {
    constructor(baseUrl = 'http://localhost:8080') {
        this.baseUrl = baseUrl;
    }
    
    async researchWithMultiLLM(options) {
        const {
            query,
            providerPreference = 'auto',
            qualityThreshold = 0.8,
            maxCostUsd = null
        } = options;
        
        const payload = {
            query,
            provider_preference: providerPreference,
            quality_threshold: qualityThreshold,
        };
        
        if (maxCostUsd) {
            payload.max_cost_usd = maxCostUsd;
        }
        
        const response = await fetch(`${this.baseUrl}/api/research`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(payload),
        });
        
        return await response.json();
    }
    
    async getProviderStatus() {
        const response = await fetch(`${this.baseUrl}/api/providers/status`);
        return await response.json();
    }
}

// Usage example
async function main() {
    const client = new FortitudeClient();
    
    try {
        // Multi-provider research
        const result = await client.researchWithMultiLLM({
            query: 'What are the latest developments in artificial intelligence?',
            providerPreference: 'cost_optimized',
            qualityThreshold: 0.8,
            maxCostUsd: 0.03
        });
        
        console.log(`Provider used: ${result.provider_used}`);
        console.log(`Quality score: ${result.quality_score}`);
        console.log(`Cost: $${result.cost_usd}`);
        console.log(`Response: ${result.content}`);
        
        // Check provider status
        const status = await client.getProviderStatus();
        Object.entries(status.providers).forEach(([provider, info]) => {
            console.log(`${provider}: ${info.status}`);
        });
    } catch (error) {
        console.error('Error:', error);
    }
}

main();
```

## <optimization>Performance Optimization</optimization>

### **Provider Selection Optimization**

```yaml
# Advanced provider selection configuration
selection_optimization:
  # Machine learning-based selection
  enable_ml_selection: true
  ml_model_path: "/path/to/selection-model.pkl"
  
  # Query classification for optimal provider matching
  query_classification:
    enabled: true
    models:
      - type: "domain_classifier"
        path: "/path/to/domain-classifier.pkl"
      - type: "complexity_analyzer"
        path: "/path/to/complexity-analyzer.pkl"
  
  # Provider specialization mapping
  specializations:
    technical_queries:
      preferred_providers: ["openai", "claude"]
      quality_weight: 0.6
      speed_weight: 0.4
      
    creative_tasks:
      preferred_providers: ["openai", "gemini"]
      quality_weight: 0.7
      speed_weight: 0.3
      
    factual_research:
      preferred_providers: ["claude", "gemini"]
      quality_weight: 0.8
      speed_weight: 0.2
      
    current_events:
      preferred_providers: ["gemini", "claude"]
      quality_weight: 0.7
      speed_weight: 0.3

# Caching optimization
caching:
  enabled: true
  provider_aware: true
  
  # Cache configuration
  cache_ttl_hours: 24
  max_cache_size_mb: 1024
  
  # Provider-specific caching
  provider_cache_settings:
    openai:
      enable_semantic_cache: true
      similarity_threshold: 0.95
      
    claude:
      enable_semantic_cache: true
      similarity_threshold: 0.93
      
    gemini:
      enable_semantic_cache: false  # Lower consistency, skip semantic cache
```

### **Performance Tuning**

```bash
# Run performance optimization analysis
fortitude optimize analyze --duration 3600  # 1 hour analysis

# Apply recommended optimizations
fortitude optimize apply --recommendations auto

# Custom optimization
fortitude optimize --focus cost --target-quality 0.85

# Performance benchmarking
fortitude benchmark --providers all --queries 100 --output performance-report.json
```

## <troubleshooting>Troubleshooting</troubleshooting>

### **Common Issues and Solutions**

<troubleshooting-guide>

**Issue**: Provider authentication failures
```bash
# Solution: Verify API keys and test connectivity
fortitude providers validate-keys

# Check specific provider
fortitude providers test --provider openai --verbose

# Test with temporary key
OPENAI_API_KEY="your-key-here" fortitude providers test --provider openai
```

**Issue**: High latency with provider switching
```bash
# Solution: Optimize provider health check intervals
# Edit providers.yaml:
fallback:
  health_monitoring:
    check_interval_seconds: 15  # Reduce from 30 to 15
    parallel_checks: true       # Enable parallel health checks
```

**Issue**: Cost overruns
```bash
# Solution: Implement stricter cost controls
# Edit providers.yaml:
cost_optimization:
  enabled: true
  hard_limits: true           # Enable hard budget limits
  daily_budget_usd: 20.0      # Reduce daily budget
  auto_disable_on_limit: true # Auto-disable when budget reached
```

**Issue**: Quality inconsistency across providers
```bash
# Solution: Enable cross-validation for quality assurance
# Edit providers.yaml:
quality_control:
  cross_validation:
    enabled: true
    threshold: 0.8
    providers_per_validation: 2
    consensus_method: "weighted_average"
```

</troubleshooting-guide>

### **Diagnostic Commands**

```bash
# Comprehensive system diagnostic
fortitude diagnose --full

# Provider connectivity test
fortitude diagnose connectivity

# Performance analysis
fortitude diagnose performance --duration 300

# Configuration validation
fortitude diagnose config

# Log analysis
fortitude logs --level error --since 1h
fortitude logs --provider openai --since 30m
```

### **Debug Configuration**

```yaml
# Debug-enabled configuration
debug:
  enabled: true
  log_level: "debug"
  
  # Provider-specific debugging
  provider_debug:
    log_requests: true
    log_responses: false  # Don't log responses (may contain sensitive data)
    log_timing: true
    log_costs: true
    
  # Fallback debugging
  fallback_debug:
    log_decisions: true
    log_health_checks: true
    log_retry_attempts: true
    
  # Performance debugging
  performance_debug:
    enable_profiling: true
    profile_output: "/tmp/fortitude-profile"
    trace_slow_requests: true
    slow_request_threshold_ms: 5000
```

## <production>Production Deployment</production>

### **Production Configuration Template**

```yaml
# Production-ready multi-LLM configuration
environment: production

providers:
  openai:
    enabled: true
    api_key: ${OPENAI_API_KEY}
    model: "gpt-4"
    timeout_seconds: 45
    rate_limit:
      requests_per_minute: 100
      tokens_per_minute: 80000
      max_concurrent: 10
    retry_config:
      max_retries: 3
      base_delay_ms: 1000
      max_delay_ms: 10000

  claude:
    enabled: true
    api_key: ${ANTHROPIC_API_KEY}
    model: "claude-3-sonnet-20240229"
    timeout_seconds: 45
    rate_limit:
      requests_per_minute: 80
      tokens_per_minute: 60000
      max_concurrent: 8

  gemini:
    enabled: true
    api_key: ${GOOGLE_API_KEY}
    model: "gemini-pro"
    timeout_seconds: 30
    rate_limit:
      requests_per_minute: 100
      tokens_per_minute: 50000
      max_concurrent: 10

selection_strategy:
  type: "performance_based"
  fallback_to_round_robin: true

fallback:
  enabled: true
  strategy: "adaptive_cascade"
  circuit_breaker:
    enabled: true
    failure_threshold: 5
    recovery_timeout: 300

monitoring:
  enabled: true
  metrics_retention_days: 30
  alert_webhook: ${ALERT_WEBHOOK_URL}

security:
  api_key_rotation: true
  audit_logging: true
  rate_limiting: true
```

### **Docker Deployment**

```dockerfile
FROM fortitude:latest

# Copy production configuration
COPY production-providers.yaml /etc/fortitude/providers.yaml

# Set production environment
ENV ENVIRONMENT=production
ENV LOG_LEVEL=info
ENV METRICS_ENABLED=true

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD fortitude providers health-check || exit 1

EXPOSE 8080 8081

CMD ["fortitude", "server", "--config", "/etc/fortitude/providers.yaml"]
```

### **Kubernetes Deployment**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: fortitude-multi-llm
spec:
  replicas: 3
  selector:
    matchLabels:
      app: fortitude
  template:
    metadata:
      labels:
        app: fortitude
    spec:
      containers:
      - name: fortitude
        image: fortitude:multi-llm-latest
        env:
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: llm-api-keys
              key: openai-key
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: llm-api-keys
              key: anthropic-key
        - name: GOOGLE_API_KEY
          valueFrom:
            secretKeyRef:
              name: llm-api-keys
              key: google-key
        ports:
        - containerPort: 8080
        - containerPort: 8081
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /api/health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /api/providers/status
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

## <maintenance>Ongoing Maintenance</maintenance>

### **Regular Maintenance Tasks**

```bash
# Weekly provider performance review
fortitude reports weekly --providers all --output weekly-report.json

# Monthly cost analysis
fortitude reports cost --period monthly --breakdown provider

# Update provider configurations based on performance
fortitude optimize update-config --auto-apply false

# Rotate API keys (if automated rotation is not enabled)
fortitude security rotate-keys --provider openai --schedule

# Clean up old logs and metrics
fortitude maintenance cleanup --older-than 30d
```

### **Monitoring and Alerting**

```bash
# Set up monitoring alerts
fortitude alerts configure --webhook https://your-webhook.com/alerts

# Test alert system
fortitude alerts test --severity warning

# View current alerts
fortitude alerts list --active

# Export monitoring data
fortitude export metrics --format prometheus --output /path/to/metrics
```

## <references>See Also</references>

- [Multi-LLM Architecture](../architecture/multi-llm-architecture.md) - Technical architecture details
- [Quality Control Setup](../user-guides/learning-and-monitoring-configuration.md) - Quality configuration
- [Performance Tuning Guide](../performance/tuning-guide.md) - Optimization strategies
- [Deployment Guide](deployment-and-integration-guide.md) - Production deployment
- [Troubleshooting Guide](../troubleshooting/sprint-009-issues.md) - Issue resolution
- [API Reference](../api-reference/multi-llm-endpoints.md) - API documentation