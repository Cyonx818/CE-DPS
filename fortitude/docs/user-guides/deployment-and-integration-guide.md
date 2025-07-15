# Fortitude System Deployment and Integration Guide

This comprehensive guide covers deployment and integration for the complete Fortitude system, including the new Learning and Monitoring systems introduced in Sprint 009.

## Table of Contents

1. [Overview](#overview)
2. [System Requirements](#system-requirements)
3. [Dependency Management](#dependency-management)
4. [Component Installation](#component-installation)
5. [Configuration Setup](#configuration-setup)
6. [Service Deployment](#service-deployment)
7. [Learning System Setup](#learning-system-setup)
8. [Monitoring System Setup](#monitoring-system-setup)
9. [Integration Validation](#integration-validation)
10. [Production Deployment](#production-deployment)
11. [Docker Deployment](#docker-deployment)
12. [Troubleshooting](#troubleshooting)

## Overview

The Fortitude system consists of multiple integrated components:
- **Core Engine**: Research pipeline, classification, and storage
- **API Server**: RESTful HTTP API with authentication and caching
- **MCP Server**: Claude Code integration via MCP protocol
- **Learning System**: Real-time learning from user feedback and patterns ✅ **NEW**
- **Monitoring System**: Performance monitoring and observability ✅ **NEW**
- **Vector Database**: Semantic search and storage backend
- **Reference Library**: Knowledge base and research output storage

## System Requirements

### Minimum Requirements
- **Operating System**: Linux, macOS, or Windows 10+
- **Rust**: Version 1.75+ (required for async traits and latest features)
- **Memory**: 4GB RAM minimum, 8GB+ recommended for production
- **Storage**: 2GB for installation, 10GB+ for reference library and learning data
- **Network**: Internet connection for LLM API access and vector database
- **CPU**: 2 cores minimum, 4+ cores recommended for concurrent processing

### Enhanced Requirements for Learning and Monitoring ✅ **NEW**
- **Memory**: Additional 2GB RAM for learning data processing and metrics storage
- **Storage**: Additional 5GB for learning data persistence and monitoring metrics
- **Database**: Vector database (Qdrant) for learning data and similarity search
- **Monitoring Stack**: Optional Prometheus/Grafana for external monitoring
- **Network**: Additional bandwidth for metrics export and alert webhooks

### Production Requirements
- **Memory**: 16GB+ RAM for high-throughput operations
- **Storage**: 50GB+ SSD for optimal performance
- **CPU**: 8+ cores for concurrent request handling
- **Network**: High-speed internet for LLM API calls
- **Load Balancer**: For multi-instance deployments
- **Database**: Dedicated vector database cluster
- **Monitoring**: External monitoring infrastructure (Prometheus, Grafana, etc.)

## Dependency Management

### Core Dependencies

```toml
# Cargo.toml workspace dependencies
[workspace.dependencies]
# Core runtime
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
thiserror = "1.0"
anyhow = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# Web server and API
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace", "timeout"] }
hyper = "1.0"
reqwest = { version = "0.11", features = ["json", "stream"] }

# Database and storage
qdrant-client = "1.0"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "chrono", "uuid"] }

# Monitoring and observability ✅ NEW
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
metrics = "0.21"
metrics-exporter-prometheus = "0.12"
opentelemetry = "0.20"
opentelemetry-jaeger = "0.19"

# Learning system dependencies ✅ NEW
candle-core = "0.3"
candle-nn = "0.3"
ndarray = "0.15"
linfa = "0.7"

# Security and authentication
jsonwebtoken = "9.0"
argon2 = "0.5"
ring = "0.17"

# Configuration and CLI
clap = { version = "4.0", features = ["derive", "env"] }
config = "0.13"
dotenv = "0.15"

# Testing
proptest = "1.0"
criterion = { version = "0.5", features = ["html_reports"] }
mockall = "0.11"
```

### External Dependencies

#### Vector Database (Qdrant)
```bash
# Install Qdrant using Docker
docker run -d \
  --name qdrant \
  -p 6333:6333 \
  -p 6334:6334 \
  -v $(pwd)/qdrant_storage:/qdrant/storage:z \
  qdrant/qdrant:v1.7.0
```

#### Monitoring Stack (Optional) ✅ **NEW**
```bash
# Prometheus for metrics collection
docker run -d \
  --name prometheus \
  -p 9090:9090 \
  -v $(pwd)/prometheus.yml:/etc/prometheus/prometheus.yml \
  prom/prometheus

# Grafana for visualization
docker run -d \
  --name grafana \
  -p 3000:3000 \
  -e "GF_SECURITY_ADMIN_PASSWORD=admin" \
  grafana/grafana
```

#### Redis (Optional for distributed caching)
```bash
# Redis for distributed caching
docker run -d \
  --name redis \
  -p 6379:6379 \
  redis:7-alpine
```

## Component Installation

### Install from Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/your-org/fortitude.git
cd fortitude

# Build all components
cargo build --release

# Install all binaries
cargo install --path crates/fortitude-api-server
cargo install --path crates/fortitude-mcp-server
cargo install --path crates/fortitude-cli

# Verify installations
fortitude-api-server --version
fortitude-mcp-server --version
fortitude-cli --version
```

### Install Specific Components

```bash
# Install only API server
cargo install --path crates/fortitude-api-server

# Install only MCP server
cargo install --path crates/fortitude-mcp-server

# Install CLI tool
cargo install --path crates/fortitude-cli
```

### Binary Distribution Installation

```bash
# Download and install pre-built binaries
wget https://github.com/your-org/fortitude/releases/latest/download/fortitude-linux-x86_64.tar.gz
tar -xzf fortitude-linux-x86_64.tar.gz

# Install binaries
sudo mv fortitude-* /usr/local/bin/
sudo chmod +x /usr/local/bin/fortitude-*

# Create system user
sudo useradd -r -s /bin/false fortitude
sudo mkdir -p /opt/fortitude
sudo chown fortitude:fortitude /opt/fortitude
```

## Configuration Setup

### Base Configuration Structure

Create the configuration directory structure:

```bash
# Create configuration directories
sudo mkdir -p /etc/fortitude/{api,mcp,learning,monitoring}
sudo mkdir -p /opt/fortitude/{data,logs,reference_library}
sudo chown -R fortitude:fortitude /opt/fortitude
```

### API Server Configuration

Create `/etc/fortitude/api/config.toml`:

```toml
[server]
host = "0.0.0.0"
port = 8080
max_connections = 1000
request_timeout_seconds = 30

[auth]
enabled = true
api_key_header = "X-API-Key"
rate_limit_per_minute = 60

[database]
url = "sqlite:///opt/fortitude/data/fortitude.db"
max_connections = 10
connection_timeout_seconds = 30

[cache]
enabled = true
ttl_seconds = 3600
max_size = 10000

[research]
enable_classification = true
enable_context_detection = true
enable_research_caching = true
reference_library_path = "/opt/fortitude/reference_library"

[vector_database]
url = "http://localhost:6333"
collection_name = "fortitude_knowledge"
timeout_seconds = 30

# Learning System Configuration ✅ NEW
[learning]
enable_feedback_learning = true
enable_pattern_recognition = true
enable_optimization = true
adaptation_threshold = 0.8
max_data_age_days = 180
min_feedback_threshold = 10
learning_rate = 0.1

[learning.storage]
collection_name = "learning_data"
enable_embeddings = true
batch_size = 200
retention_days = 730

[learning.monitoring]
enabled = true
health_check_interval_seconds = 30
metrics_interval_seconds = 10

# Monitoring System Configuration ✅ NEW
[monitoring]
enable_metrics = true
enable_tracing = true
enable_health_checks = true
enable_alerts = true
metrics_interval_seconds = 5
max_metrics_in_memory = 50000

[monitoring.performance_thresholds]
api_response_time_warning_ms = 200.0
api_response_time_critical_ms = 1000.0
api_error_rate_warning = 0.02
api_error_rate_critical = 0.05
cpu_usage_warning_percent = 70.0
cpu_usage_critical_percent = 90.0

[monitoring.alerting]
enable_email = true
enable_webhooks = true
rate_limit_per_hour = 50

[logging]
level = "info"
format = "json"
file_path = "/opt/fortitude/logs/api-server.log"
max_file_size_mb = 100
max_files = 10
```

### MCP Server Configuration

Create `/etc/fortitude/mcp/config.toml`:

```toml
[mcp_server]
host = "127.0.0.1"
port = 8081
max_connections = 500
request_timeout = 30

[mcp_server.auth]
enabled = true
jwt_secret = "${MCP_JWT_SECRET}"
token_expiration_hours = 8

[mcp_server.integration]
fortitude_data_dir = "/opt/fortitude/reference_library"
enable_research_pipeline = true
enable_reference_library = true
enable_classification = true
enable_learning_integration = true  # ✅ NEW
enable_monitoring_integration = true  # ✅ NEW

[mcp_server.performance]
cache_size = 5000
cache_ttl = 600
enable_deduplication = true

[mcp_server.logging]
level = "info"
structured = true
file_path = "/opt/fortitude/logs/mcp-server.log"
```

### Environment Variables

Create `/etc/fortitude/environment`:

```bash
# Core Configuration
FORTITUDE_ENV=production
FORTITUDE_DATA_DIR=/opt/fortitude/data
FORTITUDE_LOG_DIR=/opt/fortitude/logs

# Authentication
API_KEY_SECRET=your-api-key-secret-here
MCP_JWT_SECRET=your-jwt-secret-minimum-32-characters-long

# Vector Database
QDRANT_URL=http://localhost:6333
QDRANT_API_KEY=optional-api-key

# LLM Provider
ANTHROPIC_API_KEY=your-anthropic-api-key
OPENAI_API_KEY=your-openai-api-key

# Learning System Configuration ✅ NEW
LEARNING_ENABLE_OPTIMIZATION=true
LEARNING_ADAPTATION_THRESHOLD=0.8
LEARNING_STORAGE_RETENTION_DAYS=730

# Monitoring Configuration ✅ NEW
MONITORING_ENABLE_METRICS=true
MONITORING_PROMETHEUS_URL=http://localhost:9090
MONITORING_ALERT_WEBHOOK_URL=https://hooks.slack.com/your-webhook

# Optional: External Monitoring
PROMETHEUS_URL=http://localhost:9090
GRAFANA_URL=http://localhost:3000
JAEGER_URL=http://localhost:14268
```

## Service Deployment

### Systemd Service Files

#### API Server Service

Create `/etc/systemd/system/fortitude-api-server.service`:

```ini
[Unit]
Description=Fortitude API Server
After=network.target qdrant.service
Wants=qdrant.service

[Service]
Type=simple
User=fortitude
Group=fortitude
WorkingDirectory=/opt/fortitude
ExecStart=/usr/local/bin/fortitude-api-server --config /etc/fortitude/api/config.toml
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=10
EnvironmentFile=/etc/fortitude/environment

# Security
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/opt/fortitude

# Resource limits
LimitNOFILE=65536
MemoryHigh=4G
MemoryMax=8G

[Install]
WantedBy=multi-user.target
```

#### MCP Server Service

Create `/etc/systemd/system/fortitude-mcp-server.service`:

```ini
[Unit]
Description=Fortitude MCP Server
After=network.target fortitude-api-server.service
Wants=fortitude-api-server.service

[Service]
Type=simple
User=fortitude
Group=fortitude
WorkingDirectory=/opt/fortitude
ExecStart=/usr/local/bin/fortitude-mcp-server start --config /etc/fortitude/mcp/config.toml
Restart=always
RestartSec=10
EnvironmentFile=/etc/fortitude/environment

# Security
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/opt/fortitude

[Install]
WantedBy=multi-user.target
```

#### Vector Database Service

Create `/etc/systemd/system/qdrant.service`:

```ini
[Unit]
Description=Qdrant Vector Database
After=network.target

[Service]
Type=simple
User=fortitude
Group=fortitude
ExecStart=/usr/local/bin/qdrant --config-path /etc/fortitude/qdrant/config.yaml
Restart=always
RestartSec=5
EnvironmentFile=/etc/fortitude/environment

# Security
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/opt/fortitude/data

[Install]
WantedBy=multi-user.target
```

### Service Management

```bash
# Enable and start services
sudo systemctl daemon-reload
sudo systemctl enable qdrant fortitude-api-server fortitude-mcp-server
sudo systemctl start qdrant
sudo systemctl start fortitude-api-server
sudo systemctl start fortitude-mcp-server

# Check service status
sudo systemctl status fortitude-api-server
sudo systemctl status fortitude-mcp-server
sudo systemctl status qdrant

# View logs
sudo journalctl -f -u fortitude-api-server
sudo journalctl -f -u fortitude-mcp-server
```

## Learning System Setup ✅ **NEW**

### Initialize Learning Data Storage

```bash
# Create learning data directories
sudo mkdir -p /opt/fortitude/learning/{data,models,cache}
sudo chown -R fortitude:fortitude /opt/fortitude/learning

# Initialize vector database collection for learning data
curl -X PUT "http://localhost:6333/collections/learning_data" \
  -H "Content-Type: application/json" \
  -d '{
    "vectors": {
      "size": 384,
      "distance": "Cosine"
    }
  }'
```

### Learning System Configuration Validation

```bash
# Validate learning configuration
fortitude-cli validate-config --component learning

# Test learning data processing
fortitude-cli learning test-processing \
  --feedback-samples 10 \
  --pattern-samples 5

# Initialize learning models
fortitude-cli learning init-models \
  --model-type feedback_analyzer \
  --model-type pattern_matcher
```

### Learning System Health Check

```bash
# Check learning system health
curl -H "X-API-Key: your-api-key" \
  http://localhost:8080/api/v1/learning/health

# Expected response:
# {
#   "overall_status": "Healthy",
#   "component_results": [
#     {
#       "component": "adaptation",
#       "status": "Healthy",
#       "message": "Adaptation system operating normally"
#     }
#   ]
# }
```

## Monitoring System Setup ✅ **NEW**

### Initialize Monitoring Infrastructure

```bash
# Create monitoring data directories
sudo mkdir -p /opt/fortitude/monitoring/{metrics,traces,alerts}
sudo chown -R fortitude:fortitude /opt/fortitude/monitoring

# Configure Prometheus scraping (if using external Prometheus)
cat > /opt/fortitude/monitoring/prometheus.yml << EOF
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'fortitude-api'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    
  - job_name: 'fortitude-mcp'
    static_configs:
      - targets: ['localhost:8081']
    metrics_path: '/metrics'
EOF
```

### Monitoring Dashboard Setup

```bash
# Initialize monitoring dashboard data
curl -H "X-API-Key: your-api-key" \
  http://localhost:8080/api/v1/monitoring/dashboard

# Test alert system
curl -X POST -H "X-API-Key: your-api-key" \
  -H "Content-Type: application/json" \
  http://localhost:8080/api/v1/monitoring/test-alert \
  -d '{
    "severity": "warning",
    "component": "api",
    "message": "Test alert for deployment validation"
  }'
```

### Configure External Monitoring Integration

```bash
# Set up Grafana dashboard import
curl -X POST http://admin:admin@localhost:3000/api/dashboards/db \
  -H "Content-Type: application/json" \
  -d @/opt/fortitude/monitoring/grafana-dashboard.json

# Configure alert webhooks for Slack
export MONITORING_ALERT_WEBHOOK_URL="https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK"

# Test webhook integration
fortitude-cli monitoring test-webhook \
  --url "$MONITORING_ALERT_WEBHOOK_URL" \
  --message "Fortitude monitoring system deployed successfully"
```

## Integration Validation

### Complete System Health Check

```bash
# 1. Check all services are running
sudo systemctl is-active fortitude-api-server
sudo systemctl is-active fortitude-mcp-server
sudo systemctl is-active qdrant

# 2. Validate API endpoints
curl -i http://localhost:8080/health
curl -i http://localhost:8081/health

# 3. Test authentication
curl -H "X-API-Key: your-api-key" \
  http://localhost:8080/api/v1/health/protected

# 4. Test research functionality
curl -X POST -H "X-API-Key: your-api-key" \
  -H "Content-Type: application/json" \
  http://localhost:8080/api/v1/research \
  -d '{"query": "How to deploy Rust applications?", "priority": "medium"}'

# 5. Test classification
curl -X POST -H "X-API-Key: your-api-key" \
  -H "Content-Type: application/json" \
  http://localhost:8080/api/v1/classify \
  -d '{"content": "Deployment guide for production systems"}'

# 6. Test learning system ✅ NEW
curl -H "X-API-Key: your-api-key" \
  http://localhost:8080/api/v1/learning/metrics

# 7. Test monitoring system ✅ NEW
curl -H "X-API-Key: your-api-key" \
  http://localhost:8080/api/v1/monitoring/metrics

# 8. Test MCP integration
curl -X POST -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  http://localhost:8081/mcp/tools/research_query \
  -d '{"query": "Rust deployment patterns", "query_type": "implementation"}'
```

### Performance Validation

```bash
# Run performance tests
fortitude-cli performance test \
  --concurrent-requests 100 \
  --duration 60s \
  --endpoints api,mcp,learning,monitoring

# Expected results:
# - API response time: <200ms (95th percentile)
# - Success rate: >95%
# - Learning system latency: <100ms
# - Monitoring overhead: <5%
```

## Production Deployment

### Load Balancer Configuration (nginx)

Create `/etc/nginx/sites-available/fortitude`:

```nginx
upstream fortitude_api {
    least_conn;
    server 127.0.0.1:8080 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:8081 max_fails=3 fail_timeout=30s backup;
}

upstream fortitude_mcp {
    server 127.0.0.1:8081 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    listen [::]:80;
    server_name api.fortitude.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name api.fortitude.example.com;

    ssl_certificate /etc/ssl/certs/fortitude.crt;
    ssl_certificate_key /etc/ssl/private/fortitude.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512:ECDHE-RSA-AES256-GCM-SHA384:DHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;

    client_max_body_size 10M;
    client_body_timeout 60s;
    client_header_timeout 60s;

    location / {
        proxy_pass http://fortitude_api;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    location /mcp/ {
        proxy_pass http://fortitude_mcp/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Monitoring endpoints with basic auth
    location /metrics {
        auth_basic "Monitoring";
        auth_basic_user_file /etc/nginx/.htpasswd;
        proxy_pass http://fortitude_api;
    }
}
```

### SSL Certificate Setup

```bash
# Generate SSL certificate using Let's Encrypt
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d api.fortitude.example.com

# Or use self-signed certificate for development
sudo openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout /etc/ssl/private/fortitude.key \
  -out /etc/ssl/certs/fortitude.crt \
  -subj "/C=US/ST=State/L=City/O=Organization/CN=api.fortitude.example.com"
```

### Database Backup and Recovery

```bash
# Create backup script
cat > /opt/fortitude/scripts/backup.sh << 'EOF'
#!/bin/bash
set -e

BACKUP_DIR="/opt/fortitude/backups"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Backup SQLite database
sqlite3 /opt/fortitude/data/fortitude.db ".backup $BACKUP_DIR/fortitude_$DATE.db"

# Backup vector database
curl -X POST "http://localhost:6333/collections/fortitude_knowledge/snapshots" > "$BACKUP_DIR/qdrant_snapshot_$DATE.json"

# Backup learning data ✅ NEW
curl -X POST "http://localhost:6333/collections/learning_data/snapshots" > "$BACKUP_DIR/learning_snapshot_$DATE.json"

# Backup reference library
tar -czf "$BACKUP_DIR/reference_library_$DATE.tar.gz" /opt/fortitude/reference_library

# Backup configuration
tar -czf "$BACKUP_DIR/config_$DATE.tar.gz" /etc/fortitude

# Clean old backups (keep 30 days)
find "$BACKUP_DIR" -name "*.db" -mtime +30 -delete
find "$BACKUP_DIR" -name "*.json" -mtime +30 -delete
find "$BACKUP_DIR" -name "*.tar.gz" -mtime +30 -delete

echo "Backup completed: $DATE"
EOF

# Make backup script executable
chmod +x /opt/fortitude/scripts/backup.sh

# Set up daily backup cron job
echo "0 2 * * * fortitude /opt/fortitude/scripts/backup.sh" | sudo crontab -u fortitude -
```

## Docker Deployment

### Multi-Container Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  qdrant:
    image: qdrant/qdrant:v1.7.0
    container_name: fortitude-qdrant
    ports:
      - "6333:6333"
      - "6334:6334"
    volumes:
      - qdrant_storage:/qdrant/storage
    environment:
      - QDRANT__SERVICE__HTTP_PORT=6333
      - QDRANT__SERVICE__GRPC_PORT=6334
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:6333/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  fortitude-api:
    build:
      context: .
      dockerfile: crates/fortitude-api-server/Dockerfile
    container_name: fortitude-api-server
    ports:
      - "8080:8080"
    volumes:
      - ./reference_library:/opt/fortitude/reference_library:ro
      - fortitude_data:/opt/fortitude/data
      - fortitude_logs:/opt/fortitude/logs
    environment:
      - FORTITUDE_ENV=production
      - QDRANT_URL=http://qdrant:6333
      - API_KEY_SECRET=${API_KEY_SECRET}
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
      - LEARNING_ENABLE_OPTIMIZATION=true
      - MONITORING_ENABLE_METRICS=true
    depends_on:
      qdrant:
        condition: service_healthy
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  fortitude-mcp:
    build:
      context: .
      dockerfile: crates/fortitude-mcp-server/Dockerfile
    container_name: fortitude-mcp-server
    ports:
      - "8081:8081"
    volumes:
      - ./reference_library:/opt/fortitude/reference_library:ro
      - fortitude_data:/opt/fortitude/data
      - fortitude_logs:/opt/fortitude/logs
    environment:
      - FORTITUDE_ENV=production
      - MCP_JWT_SECRET=${MCP_JWT_SECRET}
      - FORTITUDE_API_URL=http://fortitude-api:8080
    depends_on:
      fortitude-api:
        condition: service_healthy
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8081/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  prometheus:
    image: prom/prometheus:v2.45.0
    container_name: fortitude-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--storage.tsdb.retention.time=30d'
      - '--web.enable-lifecycle'
    restart: unless-stopped

  grafana:
    image: grafana/grafana:10.0.0
    container_name: fortitude-grafana
    ports:
      - "3000:3000"
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources:ro
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_ADMIN_PASSWORD:-admin}
      - GF_USERS_ALLOW_SIGN_UP=false
    depends_on:
      - prometheus
    restart: unless-stopped

volumes:
  qdrant_storage:
  fortitude_data:
  fortitude_logs:
  prometheus_data:
  grafana_data:

networks:
  default:
    name: fortitude-network
```

### Docker Environment Configuration

Create `.env` file:

```bash
# Core secrets
API_KEY_SECRET=your-secure-api-key-secret-here
MCP_JWT_SECRET=your-jwt-secret-minimum-32-characters-long
ANTHROPIC_API_KEY=your-anthropic-api-key

# Monitoring
GRAFANA_ADMIN_PASSWORD=secure-grafana-password

# Learning system
LEARNING_ENABLE_OPTIMIZATION=true
LEARNING_ADAPTATION_THRESHOLD=0.8

# Monitoring system
MONITORING_ENABLE_METRICS=true
MONITORING_PROMETHEUS_URL=http://prometheus:9090
```

### Deploy with Docker Compose

```bash
# Deploy the complete stack
docker-compose up -d

# Check service health
docker-compose ps
docker-compose logs -f fortitude-api
docker-compose logs -f fortitude-mcp

# Scale API servers (if needed)
docker-compose up -d --scale fortitude-api=3

# Monitor resources
docker stats
```

## Troubleshooting

### Common Deployment Issues

#### 1. Service Won't Start

```bash
# Check service logs
sudo journalctl -fu fortitude-api-server
sudo journalctl -fu fortitude-mcp-server

# Check configuration
fortitude-api-server --validate-config
fortitude-mcp-server validate-config

# Test connectivity
nc -zv localhost 8080
nc -zv localhost 6333  # Qdrant
```

#### 2. Database Connection Issues

```bash
# Test Qdrant connectivity
curl -f http://localhost:6333/health

# Check collection status
curl -X GET http://localhost:6333/collections

# Recreate collections if needed
curl -X DELETE http://localhost:6333/collections/fortitude_knowledge
curl -X PUT http://localhost:6333/collections/fortitude_knowledge \
  -H "Content-Type: application/json" \
  -d '{"vectors": {"size": 384, "distance": "Cosine"}}'
```

#### 3. Learning System Issues ✅ **NEW**

```bash
# Check learning system health
curl -H "X-API-Key: your-api-key" \
  http://localhost:8080/api/v1/learning/health

# Validate learning configuration
fortitude-cli learning validate-config

# Reset learning data (if needed)
curl -X DELETE -H "X-API-Key: your-api-key" \
  http://localhost:8080/api/v1/learning/reset \
  -d '{"confirm": true}'
```

#### 4. Monitoring System Issues ✅ **NEW**

```bash
# Check monitoring system health
curl -H "X-API-Key: your-api-key" \
  http://localhost:8080/api/v1/monitoring/health

# Test metrics collection
curl -H "X-API-Key: your-api-key" \
  http://localhost:8080/metrics

# Check alert system
curl -X POST -H "X-API-Key: your-api-key" \
  http://localhost:8080/api/v1/monitoring/test-alert
```

#### 5. Performance Issues

```bash
# Check resource usage
top -p $(pgrep fortitude)
iostat -x 1 5
free -h

# Monitor database performance
curl -X GET http://localhost:6333/metrics

# Check cache hit rates
curl -H "X-API-Key: your-api-key" \
  http://localhost:8080/api/v1/cache/stats
```

### Debug Mode Configuration

Enable debug logging for troubleshooting:

```bash
# Environment variables for debug mode
export RUST_LOG=fortitude=debug,tower_http=debug
export FORTITUDE_LOG_LEVEL=debug
export LEARNING_LOG_LEVEL=debug
export MONITORING_LOG_LEVEL=debug

# Restart services with debug logging
sudo systemctl restart fortitude-api-server
sudo systemctl restart fortitude-mcp-server

# Monitor debug logs
sudo journalctl -f -u fortitude-api-server | grep DEBUG
```

### Health Check Commands

```bash
# Complete system health check script
#!/bin/bash

echo "=== Fortitude System Health Check ==="

# 1. Service status
echo "Checking services..."
systemctl is-active fortitude-api-server
systemctl is-active fortitude-mcp-server
systemctl is-active qdrant

# 2. Network connectivity
echo "Checking network connectivity..."
nc -zv localhost 8080 2>&1 | grep -q "succeeded" && echo "API Server: OK" || echo "API Server: FAIL"
nc -zv localhost 8081 2>&1 | grep -q "succeeded" && echo "MCP Server: OK" || echo "MCP Server: FAIL"
nc -zv localhost 6333 2>&1 | grep -q "succeeded" && echo "Qdrant: OK" || echo "Qdrant: FAIL"

# 3. API health
echo "Checking API health..."
curl -s -f http://localhost:8080/health > /dev/null && echo "API Health: OK" || echo "API Health: FAIL"
curl -s -f http://localhost:8081/health > /dev/null && echo "MCP Health: OK" || echo "MCP Health: FAIL"

# 4. Learning system health ✅ NEW
echo "Checking learning system..."
curl -s -H "X-API-Key: $API_KEY" \
  http://localhost:8080/api/v1/learning/health | \
  jq -r '.overall_status' 2>/dev/null || echo "Learning Health: FAIL"

# 5. Monitoring system health ✅ NEW
echo "Checking monitoring system..."
curl -s -H "X-API-Key: $API_KEY" \
  http://localhost:8080/api/v1/monitoring/health | \
  jq -r '.overall_status' 2>/dev/null || echo "Monitoring Health: FAIL"

echo "=== Health Check Complete ==="
```

---

This comprehensive deployment and integration guide provides everything needed to successfully deploy the complete Fortitude system with Learning and Monitoring capabilities. For specific deployment scenarios or advanced configuration, refer to the component-specific documentation and configuration guides.

<function_calls>
<invoke name="TodoWrite">
<parameter name="todos">[{"id": "doc-update-1", "content": "Update main system architecture documentation to include Learning and Monitoring systems", "status": "completed", "priority": "high"}, {"id": "doc-update-2", "content": "Update README.md to reflect new Sprint 009 capabilities and architecture", "status": "completed", "priority": "high"}, {"id": "doc-update-3", "content": "Create/update API documentation for new learning and monitoring endpoints", "status": "completed", "priority": "high"}, {"id": "doc-update-4", "content": "Update configuration documentation for learning and monitoring systems", "status": "completed", "priority": "medium"}, {"id": "doc-update-5", "content": "Update deployment and integration documentation with new dependencies", "status": "completed", "priority": "medium"}]