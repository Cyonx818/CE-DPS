# Learning and Monitoring Systems Configuration

This document describes the configuration options for the Learning and Monitoring systems introduced in Sprint 009.

## Overview

The Learning and Monitoring systems provide comprehensive configuration options for:
- Real-time learning from user feedback and usage patterns
- Performance monitoring and observability across all system components
- Adaptive system optimization based on learning insights
- Alert system for critical performance issues

## Learning System Configuration

### Core Learning Configuration

```toml
[learning]
# Enable feedback-based learning
enable_feedback_learning = true

# Enable pattern recognition and analysis
enable_pattern_recognition = true

# Enable automated system optimization
enable_optimization = false  # Disabled by default for safety

# Minimum confidence threshold for adaptations
adaptation_threshold = 0.7

# Maximum age of learning data in days
max_data_age_days = 90

# Minimum number of feedback entries for analysis
min_feedback_threshold = 5

# Pattern frequency threshold for significance
pattern_frequency_threshold = 3

# Learning rate for adaptation algorithms
learning_rate = 0.1

[learning.storage]
# Vector database collection for learning data
collection_name = "learning_data"

# Enable vector embeddings for learning data
enable_embeddings = true

# Batch size for bulk operations
batch_size = 100

# Retention period for old data in days
retention_days = 365

[learning.adaptation]
# Algorithms to enable
enabled_algorithms = ["feedback_analyzer", "pattern_matcher"]

# Update frequency in hours
update_frequency_hours = 24

# Enable automatic application of adaptations
auto_apply_adaptations = false

[learning.monitoring]
# Enable learning system monitoring
enabled = true

# Health check interval in seconds
health_check_interval_seconds = 30

# Performance metrics collection interval in seconds
metrics_interval_seconds = 10

# Alert threshold for learning system issues
alert_threshold = 0.8

# Maximum number of metrics to store in memory
max_metrics_in_memory = 1000

[learning.monitoring.thresholds]
# Success rate threshold for alerts
min_success_rate = 0.9

# Maximum acceptable adaptation time in milliseconds
max_adaptation_time_ms = 5000.0

# Maximum storage response time in milliseconds
max_storage_response_time_ms = 1000.0

# Pattern recognition accuracy threshold
min_recognition_accuracy = 0.8

[learning.monitoring.alerts]
# Enable email alerts for learning system
enable_email = false

# Enable webhook alerts for learning system
enable_webhooks = false

# Alert rate limiting (max alerts per hour)
rate_limit_per_hour = 5
```

### Environment Variables for Learning System

| Variable | Description | Default |
|----------|-------------|---------|
| `LEARNING_ENABLE_FEEDBACK` | Enable feedback-based learning | `true` |
| `LEARNING_ENABLE_PATTERNS` | Enable pattern recognition | `true` |
| `LEARNING_ENABLE_OPTIMIZATION` | Enable automated optimization | `false` |
| `LEARNING_ADAPTATION_THRESHOLD` | Adaptation confidence threshold (0.0-1.0) | `0.7` |
| `LEARNING_MAX_DATA_AGE_DAYS` | Maximum learning data age in days | `90` |
| `LEARNING_MIN_FEEDBACK_THRESHOLD` | Minimum feedback entries for analysis | `5` |
| `LEARNING_PATTERN_FREQUENCY_THRESHOLD` | Pattern frequency threshold | `3` |
| `LEARNING_RATE` | Learning rate for algorithms (0.0-1.0) | `0.1` |
| `LEARNING_STORAGE_COLLECTION` | Vector database collection name | `learning_data` |
| `LEARNING_STORAGE_ENABLE_EMBEDDINGS` | Enable vector embeddings | `true` |
| `LEARNING_STORAGE_BATCH_SIZE` | Batch operation size | `100` |
| `LEARNING_STORAGE_RETENTION_DAYS` | Data retention period in days | `365` |
| `LEARNING_ADAPTATION_UPDATE_HOURS` | Adaptation update frequency in hours | `24` |
| `LEARNING_ADAPTATION_AUTO_APPLY` | Auto-apply adaptations | `false` |

## Monitoring System Configuration

### Core Monitoring Configuration

```toml
[monitoring]
# Enable metrics collection
enable_metrics = true

# Enable distributed tracing
enable_tracing = true

# Enable health checks
enable_health_checks = true

# Enable alerting system
enable_alerts = true

# Metrics collection interval in seconds
metrics_interval_seconds = 10

# Health check interval in seconds
health_check_interval_seconds = 30

# Maximum number of metrics to store in memory
max_metrics_in_memory = 10000

# Metrics retention period in hours
metrics_retention_hours = 24

[monitoring.collection]
# Components to monitor
enabled_components = ["api", "mcp", "learning", "storage", "classification"]

# Sampling rate for distributed tracing (0.0-1.0)
trace_sampling_rate = 0.1

# Enable performance profiling
enable_profiling = false

# Maximum trace duration before timeout
max_trace_duration_seconds = 30

[monitoring.storage]
# Storage backend for metrics
backend = "memory"  # Options: memory, redis, prometheus

# Connection string for external storage
connection_string = ""

# Metrics batch size for bulk operations
batch_size = 1000

# Compression for stored metrics
enable_compression = true

[monitoring.performance_thresholds]
# API response time thresholds
api_response_time_warning_ms = 500.0
api_response_time_critical_ms = 2000.0

# Error rate thresholds
api_error_rate_warning = 0.05
api_error_rate_critical = 0.15

# Resource utilization thresholds
cpu_usage_warning_percent = 80.0
cpu_usage_critical_percent = 95.0

memory_usage_warning_percent = 85.0
memory_usage_critical_percent = 95.0

# Cache performance thresholds
cache_hit_rate_warning = 0.7
cache_hit_rate_critical = 0.5

# Learning system thresholds
learning_success_rate_warning = 0.85
learning_success_rate_critical = 0.7

[monitoring.alerting]
# Enable email alerts
enable_email = false

# Enable webhook alerts
enable_webhooks = false

# Alert rate limiting (max alerts per hour)
rate_limit_per_hour = 10

# Alert aggregation window in seconds
aggregation_window_seconds = 300

# Cooldown period between similar alerts in seconds
cooldown_seconds = 600

[monitoring.alerting.email]
smtp_server = ""
smtp_port = 587
username = ""
password = ""
from_address = ""
to_addresses = []

[monitoring.alerting.webhooks]
urls = []
timeout_seconds = 30
retry_attempts = 3

[monitoring.dashboard]
# Enable dashboard data collection
enabled = true

# Dashboard update interval in seconds
update_interval_seconds = 5

# Historical data retention for dashboard
retention_hours = 168  # 7 days

# Enable real-time updates
enable_real_time = true

# Maximum dashboard response time
max_response_time_ms = 200
```

### Environment Variables for Monitoring System

| Variable | Description | Default |
|----------|-------------|---------|
| `MONITORING_ENABLE_METRICS` | Enable metrics collection | `true` |
| `MONITORING_ENABLE_TRACING` | Enable distributed tracing | `true` |
| `MONITORING_ENABLE_HEALTH_CHECKS` | Enable health checks | `true` |
| `MONITORING_ENABLE_ALERTS` | Enable alerting system | `true` |
| `MONITORING_METRICS_INTERVAL` | Metrics collection interval in seconds | `10` |
| `MONITORING_HEALTH_CHECK_INTERVAL` | Health check interval in seconds | `30` |
| `MONITORING_MAX_METRICS_MEMORY` | Max metrics in memory | `10000` |
| `MONITORING_RETENTION_HOURS` | Metrics retention period in hours | `24` |
| `MONITORING_TRACE_SAMPLING_RATE` | Tracing sampling rate (0.0-1.0) | `0.1` |
| `MONITORING_ENABLE_PROFILING` | Enable performance profiling | `false` |
| `MONITORING_STORAGE_BACKEND` | Storage backend (memory/redis/prometheus) | `memory` |
| `MONITORING_STORAGE_CONNECTION` | Storage connection string | `""` |
| `MONITORING_STORAGE_BATCH_SIZE` | Metrics batch size | `1000` |
| `MONITORING_API_WARNING_MS` | API response time warning threshold | `500.0` |
| `MONITORING_API_CRITICAL_MS` | API response time critical threshold | `2000.0` |
| `MONITORING_ERROR_RATE_WARNING` | Error rate warning threshold (0.0-1.0) | `0.05` |
| `MONITORING_ERROR_RATE_CRITICAL` | Error rate critical threshold (0.0-1.0) | `0.15` |
| `MONITORING_CPU_WARNING_PERCENT` | CPU usage warning threshold | `80.0` |
| `MONITORING_CPU_CRITICAL_PERCENT` | CPU usage critical threshold | `95.0` |
| `MONITORING_MEMORY_WARNING_PERCENT` | Memory usage warning threshold | `85.0` |
| `MONITORING_MEMORY_CRITICAL_PERCENT` | Memory usage critical threshold | `95.0` |
| `MONITORING_ALERT_RATE_LIMIT` | Max alerts per hour | `10` |
| `MONITORING_ALERT_AGGREGATION_WINDOW` | Alert aggregation window in seconds | `300` |
| `MONITORING_ALERT_COOLDOWN` | Cooldown between similar alerts in seconds | `600` |

## Complete Configuration Example

### Production Configuration (TOML)

```toml
# Complete Fortitude configuration with Learning and Monitoring
[api_server]
host = "0.0.0.0"
port = 8080
max_connections = 1000

[mcp_server]
host = "127.0.0.1"
port = 8081

# Learning System Configuration
[learning]
enable_feedback_learning = true
enable_pattern_recognition = true
enable_optimization = true  # Enabled in production
adaptation_threshold = 0.8  # Higher threshold for production
max_data_age_days = 180     # Longer retention for production
min_feedback_threshold = 10
pattern_frequency_threshold = 5
learning_rate = 0.05        # Conservative learning rate

[learning.storage]
collection_name = "production_learning_data"
enable_embeddings = true
batch_size = 200
retention_days = 730        # 2 years retention

[learning.adaptation]
enabled_algorithms = ["feedback_analyzer", "pattern_matcher", "quality_optimizer"]
update_frequency_hours = 8  # More frequent updates
auto_apply_adaptations = true  # Auto-apply in production

[learning.monitoring]
enabled = true
health_check_interval_seconds = 15
metrics_interval_seconds = 5
alert_threshold = 0.9
max_metrics_in_memory = 5000

[learning.monitoring.thresholds]
min_success_rate = 0.95
max_adaptation_time_ms = 3000.0
max_storage_response_time_ms = 500.0
min_recognition_accuracy = 0.85

[learning.monitoring.alerts]
enable_email = true
enable_webhooks = true
rate_limit_per_hour = 20

# Monitoring System Configuration
[monitoring]
enable_metrics = true
enable_tracing = true
enable_health_checks = true
enable_alerts = true
metrics_interval_seconds = 5
health_check_interval_seconds = 15
max_metrics_in_memory = 50000
metrics_retention_hours = 168  # 7 days

[monitoring.collection]
enabled_components = ["api", "mcp", "learning", "storage", "classification", "research", "cache"]
trace_sampling_rate = 0.2   # Higher sampling for production
enable_profiling = true
max_trace_duration_seconds = 60

[monitoring.storage]
backend = "prometheus"      # External storage for production
connection_string = "http://prometheus:9090"
batch_size = 5000
enable_compression = true

[monitoring.performance_thresholds]
api_response_time_warning_ms = 200.0   # Stricter thresholds
api_response_time_critical_ms = 1000.0
api_error_rate_warning = 0.02
api_error_rate_critical = 0.05
cpu_usage_warning_percent = 70.0
cpu_usage_critical_percent = 90.0
memory_usage_warning_percent = 80.0
memory_usage_critical_percent = 90.0
cache_hit_rate_warning = 0.8
cache_hit_rate_critical = 0.6
learning_success_rate_warning = 0.9
learning_success_rate_critical = 0.8

[monitoring.alerting]
enable_email = true
enable_webhooks = true
rate_limit_per_hour = 50
aggregation_window_seconds = 180
cooldown_seconds = 300

[monitoring.alerting.email]
smtp_server = "smtp.company.com"
smtp_port = 587
username = "fortitude-alerts@company.com"
password = "${SMTP_PASSWORD}"
from_address = "fortitude-alerts@company.com"
to_addresses = ["ops-team@company.com", "dev-team@company.com"]

[monitoring.alerting.webhooks]
urls = [
    "https://hooks.slack.com/services/your/slack/webhook",
    "https://api.pagerduty.com/integration/your-key"
]
timeout_seconds = 15
retry_attempts = 5

[monitoring.dashboard]
enabled = true
update_interval_seconds = 2
retention_hours = 336  # 14 days
enable_real_time = true
max_response_time_ms = 100
```

### Development Configuration

```toml
# Development-focused configuration
[learning]
enable_feedback_learning = true
enable_pattern_recognition = true
enable_optimization = false  # Disabled for safety
adaptation_threshold = 0.6   # Lower threshold for testing
max_data_age_days = 30
min_feedback_threshold = 3
pattern_frequency_threshold = 2
learning_rate = 0.2          # Higher learning rate for faster iteration

[learning.storage]
collection_name = "dev_learning_data"
enable_embeddings = true
batch_size = 50
retention_days = 90

[learning.adaptation]
enabled_algorithms = ["feedback_analyzer"]
update_frequency_hours = 1   # Frequent updates for development
auto_apply_adaptations = false

[monitoring]
enable_metrics = true
enable_tracing = false       # Disabled to reduce noise
enable_health_checks = true
enable_alerts = false        # Disabled for development
metrics_interval_seconds = 30
health_check_interval_seconds = 60
max_metrics_in_memory = 1000
metrics_retention_hours = 4

[monitoring.performance_thresholds]
api_response_time_warning_ms = 1000.0  # Relaxed thresholds
api_response_time_critical_ms = 5000.0
api_error_rate_warning = 0.1
api_error_rate_critical = 0.3
```

## Configuration Validation

The system performs comprehensive validation of all configuration values:

### Learning System Validation Rules

1. **Adaptation Threshold**: Must be between 0.0 and 1.0
2. **Learning Rate**: Must be between 0.0 and 1.0  
3. **Data Age**: Must be positive integer
4. **Feedback Threshold**: Must be positive integer
5. **Pattern Frequency**: Must be positive integer
6. **Collection Name**: Must be valid identifier
7. **Batch Size**: Must be positive integer between 1 and 10000
8. **Retention Days**: Must be positive integer

### Monitoring System Validation Rules

1. **Intervals**: Must be positive integers
2. **Thresholds**: Must be between 0.0 and 1.0 for rates, positive for durations
3. **Memory Limits**: Must be positive integers
4. **Sampling Rate**: Must be between 0.0 and 1.0
5. **Storage Backend**: Must be one of: memory, redis, prometheus
6. **Email Configuration**: Must have valid SMTP settings when enabled
7. **Webhook URLs**: Must be valid HTTP/HTTPS URLs

### Error Messages

Configuration validation provides clear error messages:

```
Learning configuration validation failed: Adaptation threshold must be between 0.0 and 1.0, got 1.5
Monitoring configuration validation failed: Invalid storage backend 'invalid', must be one of: memory, redis, prometheus
Learning configuration validation failed: Batch size must be between 1 and 10000, got 50000
```

## Integration with API and MCP Servers

### API Server Integration

The Learning and Monitoring systems integrate seamlessly with the API server:

- **Learning endpoints**: `/api/v1/learning/*` provide dashboard and metrics access
- **Monitoring endpoints**: `/api/v1/monitoring/*` provide system observability
- **Authentication**: All endpoints use existing API key authentication
- **Rate limiting**: Subject to existing API rate limits

### MCP Server Integration

MCP server provides tools for Learning and Monitoring interaction:

- **Learning tools**: Feedback collection, pattern analysis, insights retrieval
- **Monitoring tools**: Health checks, metrics collection, alert management
- **Real-time updates**: Learning and monitoring data available via MCP protocol

## Performance Considerations

### Learning System Performance

- **Feedback Processing**: Async processing to avoid blocking main operations
- **Pattern Recognition**: Configurable intervals to balance accuracy vs. performance
- **Adaptation Application**: Safe rollback mechanisms for failed adaptations
- **Vector Storage**: Efficient embedding and similarity search operations

### Monitoring System Performance

- **Metrics Collection**: Low-overhead collection with <5% performance impact
- **Trace Sampling**: Configurable sampling to balance observability vs. performance  
- **Storage**: Efficient batching and compression for metric storage
- **Dashboard Updates**: Optimized queries for real-time dashboard performance

## Security Considerations

### Learning System Security

- **Feedback Validation**: All user feedback validated and sanitized
- **Adaptation Safety**: Conservative thresholds and rollback mechanisms
- **Data Privacy**: Learning data anonymization and retention policies
- **Access Control**: Learning insights protected by authentication

### Monitoring System Security

- **Metric Access**: Monitoring data protected by authentication
- **Alert Security**: Webhook URLs validated and rate limited
- **Trace Data**: Sensitive data filtering in distributed traces
- **Dashboard Security**: Dashboard endpoints protected and rate limited

## Troubleshooting

### Common Learning System Issues

1. **Learning Not Enabled**: Check `enable_feedback_learning` and related flags
2. **Low Adaptation Rate**: Review `adaptation_threshold` and `min_feedback_threshold`
3. **Storage Issues**: Verify vector database connection and collection configuration
4. **Performance Issues**: Adjust `batch_size` and processing intervals

### Common Monitoring System Issues

1. **Missing Metrics**: Check `enable_metrics` and component configuration
2. **High Memory Usage**: Reduce `max_metrics_in_memory` or increase retention intervals
3. **Alert Spam**: Adjust `rate_limit_per_hour` and `cooldown_seconds`
4. **Dashboard Slow**: Increase `update_interval_seconds` or optimize queries

### Debug Mode

Enable debug mode for detailed logging:

```bash
export LEARNING_LOG_LEVEL=debug
export MONITORING_LOG_LEVEL=debug
```

This provides detailed information about:
- Configuration loading and validation
- Learning data processing and adaptation
- Metrics collection and aggregation
- Health check execution and results
- Alert processing and delivery

---

This configuration guide provides comprehensive setup options for the Learning and Monitoring systems. For specific implementation questions or advanced configuration scenarios, refer to the API documentation and system architecture guides.