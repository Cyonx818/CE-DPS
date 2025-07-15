# Enterprise Deployment Guide

<meta>
  <title>Enterprise Deployment Guide</title>
  <type>deployment_guide</type>
  <audience>ai_assistant</audience>
  <complexity>high</complexity>
  <updated>2025-07-12</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Complete enterprise deployment guide for production-ready Fortitude systems with high availability and security
- **Key Components**: Multi-node deployment + load balancing + monitoring + security + disaster recovery = enterprise-grade system
- **Core Benefits**: 99.9% uptime, horizontal scaling, enterprise security, compliance ready
- **Deployment Time**: 4-6 hours for basic setup, 1-2 days for complete enterprise configuration
- **Related docs**: [Multi-LLM Setup](../user-guides/multi-llm-setup.md), [Security Guide](../security/compliance-guide.md)

## <context>Overview</context>

This guide provides comprehensive instructions for deploying Fortitude in enterprise environments with high availability, security, monitoring, and compliance requirements. It covers multi-cloud deployments, disaster recovery, and enterprise integration patterns.

## <architecture>Enterprise Architecture Overview</architecture>

### **High-Level Architecture**

```
┌─────────────────────────────────────────────────────────────────┐
│                        Load Balancer                           │
│                    (HAProxy/NGINX/ALB)                         │
└─────────────────┬───────────────────────────────────────────────┘
                  │
        ┌─────────┼─────────┐
        │                   │
┌───────▼──────┐    ┌───────▼──────┐    ┌─────────────────┐
│ Fortitude    │    │ Fortitude    │    │ Fortitude       │
│ Node 1       │    │ Node 2       │    │ Node N          │
│ (API+MCP)    │    │ (API+MCP)    │    │ (API+MCP)       │
└──────┬───────┘    └──────┬───────┘    └─────────┬───────┘
       │                   │                      │
       └───────────────────┼──────────────────────┘
                           │
        ┌──────────────────▼──────────────────┐
        │           Shared Services           │
        ├─────────────────────────────────────┤
        │ Vector DB │ Cache │ Monitoring     │
        │ (Qdrant)  │(Redis)│ (Prometheus)   │
        └─────────────────────────────────────┘
                           │
        ┌──────────────────▼──────────────────┐
        │         External Services           │
        ├─────────────────────────────────────┤
        │ OpenAI │ Claude │ Gemini │ Others   │
        └─────────────────────────────────────┘
```

### **Core Components**

<component-overview>

**Application Layer**:
- Multiple Fortitude instances for high availability
- Auto-scaling based on load
- Health checks and graceful degradation

**Data Layer**:
- Vector database cluster (Qdrant/Weaviate)
- Distributed cache (Redis Cluster)
- Metadata storage (PostgreSQL HA)

**Monitoring Stack**:
- Metrics collection (Prometheus + Grafana)
- Distributed tracing (Jaeger)
- Log aggregation (ELK Stack)
- Alerting (AlertManager + PagerDuty)

**Security Layer**:
- API Gateway with authentication
- Network security (VPC/firewalls)
- Secrets management (HashiCorp Vault)
- SSL/TLS encryption

</component-overview>

## <prerequisites>Prerequisites</prerequisites>

### **Infrastructure Requirements**

<infrastructure-specs>

**Minimum Production Cluster**:
- **Nodes**: 3+ application nodes, 3+ database nodes
- **CPU**: 8+ cores per application node, 4+ cores per DB node
- **Memory**: 16GB+ per application node, 8GB+ per DB node
- **Storage**: 100GB+ per node, SSD recommended
- **Network**: 1Gb/s+ bandwidth, low latency

**Recommended Enterprise Setup**:
- **Application Nodes**: 6+ nodes (3 per availability zone)
- **Database Cluster**: 3+ nodes with replication
- **Load Balancers**: 2+ for redundancy
- **Monitoring Stack**: Dedicated cluster
- **Backup Storage**: Multi-region backup solution

</infrastructure-specs>

### **Software Requirements**

```bash
# Container Runtime
- Docker 20.10+ or Podman 3.0+
- Kubernetes 1.24+ (for K8s deployment)

# Database Systems
- PostgreSQL 13+ (for metadata)
- Redis 6.0+ (for caching)
- Qdrant 1.0+ (for vector storage)

# Monitoring Stack
- Prometheus 2.35+
- Grafana 8.0+
- Jaeger 1.35+

# Load Balancing
- HAProxy 2.4+ or NGINX 1.20+
- Cloud load balancer (AWS ALB, GCP GLB, Azure LB)

# Security Tools
- HashiCorp Vault 1.10+
- Cert-Manager (for K8s)
- External secrets operator (optional)
```

### **Network and Security**

```bash
# Required Ports
8080    # Fortitude API server
8081    # Monitoring dashboard
9090    # Prometheus metrics
3000    # Grafana dashboard
6333    # Qdrant vector database
5432    # PostgreSQL
6379    # Redis

# External Connectivity
- OpenAI API (api.openai.com:443)
- Anthropic API (api.anthropic.com:443)
- Google AI API (generativelanguage.googleapis.com:443)
- Monitoring webhooks (configurable)
```

## <deployment-options>Deployment Options</deployment-options>

### **Option 1: Kubernetes Deployment (Recommended)**

#### **1.1 Namespace and Resources Setup**

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: fortitude-enterprise
  labels:
    app.kubernetes.io/name: fortitude
    app.kubernetes.io/instance: enterprise
    app.kubernetes.io/version: "1.0.0"
---
# Resource quotas for the namespace
apiVersion: v1
kind: ResourceQuota
metadata:
  name: fortitude-quota
  namespace: fortitude-enterprise
spec:
  hard:
    requests.cpu: "20"
    requests.memory: 40Gi
    limits.cpu: "40"
    limits.memory: 80Gi
    persistentvolumeclaims: "20"
```

#### **1.2 Secrets Management**

```yaml
# secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: fortitude-api-keys
  namespace: fortitude-enterprise
type: Opaque
data:
  openai-api-key: <base64-encoded-key>
  anthropic-api-key: <base64-encoded-key>
  google-api-key: <base64-encoded-key>
  jwt-secret: <base64-encoded-secret>
---
apiVersion: v1
kind: Secret
metadata:
  name: fortitude-db-credentials
  namespace: fortitude-enterprise
type: Opaque
data:
  postgres-url: <base64-encoded-connection-string>
  redis-url: <base64-encoded-connection-string>
  qdrant-url: <base64-encoded-connection-string>
```

#### **1.3 Configuration ConfigMap**

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: fortitude-config
  namespace: fortitude-enterprise
data:
  production.yaml: |
    environment: production
    
    server:
      host: "0.0.0.0"
      port: 8080
      workers: 4
      max_connections: 1000
      
    providers:
      openai:
        enabled: true
        model: "gpt-4"
        max_tokens: 4096
        timeout_seconds: 45
        rate_limit:
          requests_per_minute: 200
          tokens_per_minute: 100000
          max_concurrent: 20
      
      claude:
        enabled: true
        model: "claude-3-sonnet-20240229"
        max_tokens: 4096
        timeout_seconds: 45
        rate_limit:
          requests_per_minute: 150
          tokens_per_minute: 80000
          max_concurrent: 15
      
      gemini:
        enabled: true
        model: "gemini-pro"
        max_tokens: 2048
        timeout_seconds: 30
        rate_limit:
          requests_per_minute: 200
          tokens_per_minute: 60000
          max_concurrent: 20
    
    selection_strategy:
      type: "performance_based"
      criteria:
        response_time: 0.3
        quality_score: 0.4
        cost_efficiency: 0.2
        availability: 0.1
    
    fallback:
      enabled: true
      strategy: "adaptive_cascade"
      max_retries: 3
      circuit_breaker:
        enabled: true
        failure_threshold: 5
        recovery_timeout: 300
    
    quality_control:
      enabled: true
      cross_validation:
        enabled: true
        provider_count: 2
        agreement_threshold: 0.8
    
    learning_system:
      enabled: true
      adaptation:
        auto_apply: false  # Require manual approval in production
        confidence_threshold: 0.9
    
    monitoring:
      enabled: true
      prometheus:
        enabled: true
        port: 9090
        path: "/metrics"
      jaeger:
        enabled: true
        endpoint: "http://jaeger-collector:14268/api/traces"
      alerts:
        webhook_url: "https://alerts.company.com/webhook/fortitude"
    
    security:
      enable_auth: true
      jwt_secret_env: "JWT_SECRET"
      cors_origins: ["https://dashboard.company.com"]
      rate_limiting:
        enabled: true
        requests_per_minute: 1000
        burst_size: 100
    
    cache:
      redis:
        cluster_mode: true
        ttl_hours: 24
        max_size_mb: 2048
    
    vector_db:
      qdrant:
        collection_name: "fortitude_enterprise"
        vector_size: 1536
        distance: "cosine"
        replicas: 3
    
    storage:
      postgres:
        max_connections: 20
        connection_timeout: 30
        statement_timeout: 60
```

#### **1.4 Application Deployment**

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: fortitude-app
  namespace: fortitude-enterprise
  labels:
    app: fortitude
    component: api-server
spec:
  replicas: 6  # High availability across 3 AZs
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 2
      maxUnavailable: 1
  selector:
    matchLabels:
      app: fortitude
      component: api-server
  template:
    metadata:
      labels:
        app: fortitude
        component: api-server
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
        prometheus.io/path: "/metrics"
    spec:
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 1000
      containers:
      - name: fortitude
        image: fortitude:enterprise-v1.0.0
        imagePullPolicy: IfNotPresent
        ports:
        - containerPort: 8080
          name: api
          protocol: TCP
        - containerPort: 8081
          name: dashboard
          protocol: TCP
        - containerPort: 9090
          name: metrics
          protocol: TCP
        env:
        - name: ENVIRONMENT
          value: "production"
        - name: LOG_LEVEL
          value: "info"
        - name: CONFIG_FILE
          value: "/etc/fortitude/production.yaml"
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: fortitude-api-keys
              key: openai-api-key
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: fortitude-api-keys
              key: anthropic-api-key
        - name: GOOGLE_API_KEY
          valueFrom:
            secretKeyRef:
              name: fortitude-api-keys
              key: google-api-key
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: fortitude-api-keys
              key: jwt-secret
        - name: POSTGRES_URL
          valueFrom:
            secretKeyRef:
              name: fortitude-db-credentials
              key: postgres-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: fortitude-db-credentials
              key: redis-url
        - name: QDRANT_URL
          valueFrom:
            secretKeyRef:
              name: fortitude-db-credentials
              key: qdrant-url
        volumeMounts:
        - name: config
          mountPath: /etc/fortitude
          readOnly: true
        - name: tmp
          mountPath: /tmp
        - name: cache
          mountPath: /var/cache/fortitude
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /api/health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /api/ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
        startupProbe:
          httpGet:
            path: /api/health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 30
      volumes:
      - name: config
        configMap:
          name: fortitude-config
      - name: tmp
        emptyDir: {}
      - name: cache
        emptyDir:
          sizeLimit: 1Gi
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - fortitude
              topologyKey: kubernetes.io/hostname
      topologySpreadConstraints:
      - maxSkew: 1
        topologyKey: topology.kubernetes.io/zone
        whenUnsatisfiable: DoNotSchedule
        labelSelector:
          matchLabels:
            app: fortitude
```

#### **1.5 Service and Ingress Configuration**

```yaml
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: fortitude-service
  namespace: fortitude-enterprise
  labels:
    app: fortitude
spec:
  selector:
    app: fortitude
    component: api-server
  ports:
  - name: api
    port: 8080
    targetPort: 8080
    protocol: TCP
  - name: dashboard
    port: 8081
    targetPort: 8081
    protocol: TCP
  - name: metrics
    port: 9090
    targetPort: 9090
    protocol: TCP
  type: ClusterIP
---
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: fortitude-ingress
  namespace: fortitude-enterprise
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/use-regex: "true"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/rate-limit: "1000"
    nginx.ingress.kubernetes.io/rate-limit-window: "1m"
spec:
  tls:
  - hosts:
    - api.fortitude.company.com
    - dashboard.fortitude.company.com
    secretName: fortitude-tls
  rules:
  - host: api.fortitude.company.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: fortitude-service
            port:
              number: 8080
  - host: dashboard.fortitude.company.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: fortitude-service
            port:
              number: 8081
```

#### **1.6 Horizontal Pod Autoscaler**

```yaml
# hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: fortitude-hpa
  namespace: fortitude-enterprise
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: fortitude-app
  minReplicas: 6
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  - type: Pods
    pods:
      metric:
        name: requests_per_second
      target:
        type: AverageValue
        averageValue: "1000"
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
```

### **Option 2: Docker Swarm Deployment**

#### **2.1 Docker Compose Configuration**

```yaml
# docker-compose.enterprise.yml
version: '3.8'

services:
  # Load Balancer
  haproxy:
    image: haproxy:2.4
    ports:
      - "80:80"
      - "443:443"
      - "8404:8404"  # HAProxy stats
    volumes:
      - ./haproxy/haproxy.cfg:/usr/local/etc/haproxy/haproxy.cfg
      - ./ssl:/etc/ssl/private
    deploy:
      replicas: 2
      placement:
        constraints:
          - node.role == manager
    networks:
      - fortitude-network

  # Fortitude Application
  fortitude:
    image: fortitude:enterprise-v1.0.0
    environment:
      - ENVIRONMENT=production
      - CONFIG_FILE=/etc/fortitude/production.yaml
      - OPENAI_API_KEY_FILE=/run/secrets/openai_api_key
      - ANTHROPIC_API_KEY_FILE=/run/secrets/anthropic_api_key
      - GOOGLE_API_KEY_FILE=/run/secrets/google_api_key
      - POSTGRES_URL_FILE=/run/secrets/postgres_url
      - REDIS_URL_FILE=/run/secrets/redis_url
      - QDRANT_URL_FILE=/run/secrets/qdrant_url
    volumes:
      - ./config/production.yaml:/etc/fortitude/production.yaml:ro
      - fortitude-cache:/var/cache/fortitude
    secrets:
      - openai_api_key
      - anthropic_api_key
      - google_api_key
      - postgres_url
      - redis_url
      - qdrant_url
    deploy:
      replicas: 6
      update_config:
        parallelism: 2
        delay: 10s
        failure_action: rollback
        order: start-first
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
      resources:
        limits:
          cpus: '2.0'
          memory: 4G
        reservations:
          cpus: '1.0'
          memory: 2G
      placement:
        max_replicas_per_node: 2
        constraints:
          - node.labels.tier == app
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/api/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    networks:
      - fortitude-network
    depends_on:
      - postgres
      - redis
      - qdrant

  # PostgreSQL Database
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: fortitude
      POSTGRES_USER: fortitude
      POSTGRES_PASSWORD_FILE: /run/secrets/postgres_password
    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./scripts/init-db.sql:/docker-entrypoint-initdb.d/init.sql:ro
    secrets:
      - postgres_password
    deploy:
      replicas: 1
      placement:
        constraints:
          - node.labels.tier == data
      resources:
        limits:
          cpus: '1.0'
          memory: 2G
        reservations:
          cpus: '0.5'
          memory: 1G
    networks:
      - fortitude-network

  # Redis Cache
  redis:
    image: redis:7-alpine
    command: redis-server --requirepass-file /run/secrets/redis_password --maxmemory 1gb --maxmemory-policy allkeys-lru
    volumes:
      - redis-data:/data
    secrets:
      - redis_password
    deploy:
      replicas: 1
      placement:
        constraints:
          - node.labels.tier == data
      resources:
        limits:
          cpus: '0.5'
          memory: 1G
        reservations:
          cpus: '0.25'
          memory: 512M
    networks:
      - fortitude-network

  # Qdrant Vector Database
  qdrant:
    image: qdrant/qdrant:v1.3.0
    ports:
      - "6333:6333"
    volumes:
      - qdrant-data:/qdrant/storage
    environment:
      QDRANT__SERVICE__GRPC_PORT: 6334
      QDRANT__SERVICE__HTTP_PORT: 6333
    deploy:
      replicas: 1
      placement:
        constraints:
          - node.labels.tier == data
      resources:
        limits:
          cpus: '1.0'
          memory: 2G
        reservations:
          cpus: '0.5'
          memory: 1G
    networks:
      - fortitude-network

  # Prometheus Monitoring
  prometheus:
    image: prom/prometheus:v2.40.0
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--storage.tsdb.retention.time=30d'
      - '--web.enable-lifecycle'
    deploy:
      replicas: 1
      placement:
        constraints:
          - node.labels.tier == monitoring
    networks:
      - fortitude-network

  # Grafana Dashboard
  grafana:
    image: grafana/grafana:9.2.0
    ports:
      - "3000:3000"
    environment:
      GF_SECURITY_ADMIN_PASSWORD_FILE: /run/secrets/grafana_password
      GF_SECURITY_ADMIN_USER: admin
    volumes:
      - grafana-data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources:ro
    secrets:
      - grafana_password
    deploy:
      replicas: 1
      placement:
        constraints:
          - node.labels.tier == monitoring
    networks:
      - fortitude-network
    depends_on:
      - prometheus

networks:
  fortitude-network:
    driver: overlay
    attachable: true

volumes:
  postgres-data:
  redis-data:
  qdrant-data:
  prometheus-data:
  grafana-data:
  fortitude-cache:

secrets:
  openai_api_key:
    external: true
  anthropic_api_key:
    external: true
  google_api_key:
    external: true
  postgres_password:
    external: true
  postgres_url:
    external: true
  redis_password:
    external: true
  redis_url:
    external: true
  qdrant_url:
    external: true
  grafana_password:
    external: true
```

#### **2.2 HAProxy Configuration**

```
# haproxy/haproxy.cfg
global
    daemon
    log stdout local0
    chroot /var/lib/haproxy
    stats socket /run/haproxy/admin.sock mode 660 level admin
    stats timeout 30s
    user haproxy
    group haproxy
    
    # SSL Configuration
    ssl-default-bind-ciphers ECDHE+AESGCM:ECDHE+CHACHA20:RSA+AESGCM:RSA+AES:!aNULL:!MD5:!DSS
    ssl-default-bind-options ssl-min-ver TLSv1.2 no-tls-tickets

defaults
    mode http
    log global
    option httplog
    option dontlognull
    option http-server-close
    option forwardfor except 127.0.0.0/8
    option redispatch
    retries 3
    timeout http-request 10s
    timeout queue 1m
    timeout connect 10s
    timeout client 1m
    timeout server 1m
    timeout http-keep-alive 10s
    timeout check 10s
    maxconn 3000

# Statistics page
frontend stats
    bind *:8404
    stats enable
    stats uri /stats
    stats refresh 30s
    stats admin if TRUE

# Main frontend
frontend fortitude_frontend
    bind *:80
    bind *:443 ssl crt /etc/ssl/private/fortitude.pem
    
    # Redirect HTTP to HTTPS
    redirect scheme https if !{ ssl_fc }
    
    # Rate limiting
    stick-table type ip size 100k expire 30s store http_req_rate(10s)
    http-request track-sc0 src
    http-request reject if { sc_http_req_rate(0) gt 100 }
    
    # Route to API backend
    use_backend fortitude_api if { hdr_beg(host) -i api. }
    
    # Route to dashboard backend  
    use_backend fortitude_dashboard if { hdr_beg(host) -i dashboard. }
    
    # Default to API
    default_backend fortitude_api

# API backend
backend fortitude_api
    balance roundrobin
    option httpchk GET /api/health
    http-check expect status 200
    
    # Health check configuration
    default-server check inter 10s fall 3 rise 2
    
    # Server definitions (auto-discovered in Swarm)
    server-template fortitude 10 fortitude:8080 check resolvers docker

# Dashboard backend
backend fortitude_dashboard
    balance roundrobin
    option httpchk GET /api/health
    http-check expect status 200
    
    default-server check inter 10s fall 3 rise 2
    server-template fortitude-dashboard 10 fortitude:8081 check resolvers docker

# DNS resolver for Docker Swarm
resolvers docker
    nameserver dns1 127.0.0.11:53
    resolve_retries 3
    timeout resolve 1s
    timeout retry 1s
    hold other 10s
    hold refused 10s
    hold nx 10s
    hold timeout 10s
    hold valid 10s
    hold obsolete 10s
```

### **Option 3: Bare Metal / VM Deployment**

#### **3.1 System Preparation Script**

```bash
#!/bin/bash
# prepare-enterprise-deployment.sh

set -euo pipefail

# Configuration
FORTITUDE_USER="fortitude"
FORTITUDE_GROUP="fortitude"
INSTALL_DIR="/opt/fortitude"
DATA_DIR="/var/lib/fortitude"
LOG_DIR="/var/log/fortitude"
CONFIG_DIR="/etc/fortitude"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
    exit 1
}

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    error "This script must be run as root"
fi

log "Starting Fortitude enterprise deployment preparation..."

# Update system
log "Updating system packages..."
if command -v apt-get &> /dev/null; then
    apt-get update && apt-get upgrade -y
    apt-get install -y curl wget git unzip software-properties-common
elif command -v yum &> /dev/null; then
    yum update -y
    yum install -y curl wget git unzip
elif command -v dnf &> /dev/null; then
    dnf update -y
    dnf install -y curl wget git unzip
else
    error "Unsupported package manager"
fi

# Create fortitude user and group
log "Creating fortitude user and group..."
if ! getent group $FORTITUDE_GROUP &> /dev/null; then
    groupadd --system $FORTITUDE_GROUP
fi

if ! getent passwd $FORTITUDE_USER &> /dev/null; then
    useradd --system --gid $FORTITUDE_GROUP --home-dir $DATA_DIR \
            --shell /bin/false --comment "Fortitude Service User" $FORTITUDE_USER
fi

# Create directories
log "Creating directory structure..."
mkdir -p $INSTALL_DIR $DATA_DIR $LOG_DIR $CONFIG_DIR
mkdir -p $DATA_DIR/{cache,vector_db,backups}
mkdir -p $LOG_DIR/{api,quality,learning,monitoring}

# Set permissions
chown -R $FORTITUDE_USER:$FORTITUDE_GROUP $DATA_DIR $LOG_DIR
chown root:$FORTITUDE_GROUP $CONFIG_DIR
chmod 750 $CONFIG_DIR
chmod 755 $INSTALL_DIR

# Install Docker (if not present)
if ! command -v docker &> /dev/null; then
    log "Installing Docker..."
    curl -fsSL https://get.docker.com -o get-docker.sh
    sh get-docker.sh
    usermod -aG docker $FORTITUDE_USER
    systemctl enable docker
    systemctl start docker
fi

# Install Docker Compose
if ! command -v docker-compose &> /dev/null; then
    log "Installing Docker Compose..."
    curl -L "https://github.com/docker/compose/releases/download/v2.12.2/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
    chmod +x /usr/local/bin/docker-compose
fi

# Install PostgreSQL client (for database management)
log "Installing PostgreSQL client..."
if command -v apt-get &> /dev/null; then
    apt-get install -y postgresql-client
elif command -v yum &> /dev/null; then
    yum install -y postgresql
elif command -v dnf &> /dev/null; then
    dnf install -y postgresql
fi

# Install Redis tools
log "Installing Redis tools..."
if command -v apt-get &> /dev/null; then
    apt-get install -y redis-tools
elif command -v yum &> /dev/null; then
    yum install -y redis
elif command -v dnf &> /dev/null; then
    dnf install -y redis
fi

# Configure firewall (basic setup)
log "Configuring firewall..."
if command -v ufw &> /dev/null; then
    ufw --force enable
    ufw allow 22    # SSH
    ufw allow 80    # HTTP
    ufw allow 443   # HTTPS
    ufw allow 8080  # Fortitude API
    ufw allow 8081  # Fortitude Dashboard
    ufw allow 9090  # Prometheus
    ufw allow 3000  # Grafana
elif command -v firewall-cmd &> /dev/null; then
    systemctl enable firewalld
    systemctl start firewalld
    firewall-cmd --permanent --add-port=22/tcp
    firewall-cmd --permanent --add-port=80/tcp
    firewall-cmd --permanent --add-port=443/tcp
    firewall-cmd --permanent --add-port=8080/tcp
    firewall-cmd --permanent --add-port=8081/tcp
    firewall-cmd --permanent --add-port=9090/tcp
    firewall-cmd --permanent --add-port=3000/tcp
    firewall-cmd --reload
fi

# Install monitoring agents
log "Installing monitoring agents..."

# Install Node Exporter for Prometheus
NODE_EXPORTER_VERSION="1.4.0"
wget https://github.com/prometheus/node_exporter/releases/download/v${NODE_EXPORTER_VERSION}/node_exporter-${NODE_EXPORTER_VERSION}.linux-amd64.tar.gz
tar xvfz node_exporter-${NODE_EXPORTER_VERSION}.linux-amd64.tar.gz
mv node_exporter-${NODE_EXPORTER_VERSION}.linux-amd64/node_exporter /usr/local/bin/
rm -rf node_exporter-${NODE_EXPORTER_VERSION}*

# Create node_exporter systemd service
cat > /etc/systemd/system/node_exporter.service << EOF
[Unit]
Description=Node Exporter
Wants=network-online.target
After=network-online.target

[Service]
User=nobody
Group=nobody
Type=simple
ExecStart=/usr/local/bin/node_exporter
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable node_exporter
systemctl start node_exporter

# Configure log rotation
log "Configuring log rotation..."
cat > /etc/logrotate.d/fortitude << EOF
$LOG_DIR/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 $FORTITUDE_USER $FORTITUDE_GROUP
    postrotate
        systemctl reload fortitude || true
    endscript
}
EOF

# Set up backup directory
log "Setting up backup configuration..."
mkdir -p /opt/fortitude-backups
chown $FORTITUDE_USER:$FORTITUDE_GROUP /opt/fortitude-backups

# Create systemd service template
log "Creating systemd service template..."
cat > /etc/systemd/system/fortitude.service << EOF
[Unit]
Description=Fortitude AI Research Platform
Requires=docker.service
After=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=$INSTALL_DIR
ExecStart=/usr/local/bin/docker-compose -f docker-compose.production.yml up -d
ExecStop=/usr/local/bin/docker-compose -f docker-compose.production.yml down
ExecReload=/usr/local/bin/docker-compose -f docker-compose.production.yml restart
TimeoutStartSec=0
User=$FORTITUDE_USER
Group=$FORTITUDE_GROUP

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload

log "Enterprise deployment preparation completed successfully!"
log "Next steps:"
log "1. Copy Fortitude configuration files to $CONFIG_DIR"
log "2. Copy Docker Compose files to $INSTALL_DIR"
log "3. Set up SSL certificates"
log "4. Configure secrets and environment variables"
log "5. Start services with: systemctl start fortitude"

warn "Remember to:"
warn "- Configure SSL certificates for HTTPS"
warn "- Set up proper secrets management"
warn "- Configure monitoring and alerting"
warn "- Set up backup procedures"
warn "- Review and harden security settings"
```

## <security>Security Configuration</security>

### **SSL/TLS Configuration**

```bash
# Generate SSL certificates using Let's Encrypt
# Install Certbot
curl -fsSL https://packages.cloud.google.com/apt/doc/apt-key.gpg | sudo apt-key add -
sudo apt-get update
sudo apt-get install -y certbot

# Generate certificates
certbot certonly --standalone \
  -d api.fortitude.company.com \
  -d dashboard.fortitude.company.com \
  --email admin@company.com \
  --agree-tos \
  --non-interactive

# Set up automatic renewal
echo "0 12 * * * /usr/bin/certbot renew --quiet" | crontab -
```

### **Secrets Management with HashiCorp Vault**

```bash
# Install and configure Vault
wget https://releases.hashicorp.com/vault/1.12.0/vault_1.12.0_linux_amd64.zip
unzip vault_1.12.0_linux_amd64.zip
sudo mv vault /usr/local/bin/

# Start Vault server (development mode for setup)
vault server -dev &

# Set Vault address
export VAULT_ADDR='http://127.0.0.1:8200'

# Store secrets
vault kv put secret/fortitude/api-keys \
  openai="sk-..." \
  anthropic="sk-ant-..." \
  google="AIza..."

vault kv put secret/fortitude/database \
  postgres_url="postgresql://user:pass@host:5432/fortitude" \
  redis_url="redis://user:pass@host:6379/0" \
  qdrant_url="http://host:6333"

vault kv put secret/fortitude/auth \
  jwt_secret="your-jwt-secret-here"
```

### **Network Security Configuration**

```yaml
# network-policy.yaml (for Kubernetes)
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: fortitude-network-policy
  namespace: fortitude-enterprise
spec:
  podSelector:
    matchLabels:
      app: fortitude
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: ingress-nginx
    ports:
    - protocol: TCP
      port: 8080
    - protocol: TCP
      port: 8081
  - from:
    - podSelector:
        matchLabels:
          app: prometheus
    ports:
    - protocol: TCP
      port: 9090
  egress:
  - to: []
    ports:
    - protocol: TCP
      port: 443  # HTTPS to external APIs
    - protocol: TCP
      port: 5432 # PostgreSQL
    - protocol: TCP
      port: 6379 # Redis
    - protocol: TCP
      port: 6333 # Qdrant
```

## <monitoring>Monitoring and Observability</monitoring>

### **Prometheus Configuration**

```yaml
# monitoring/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "fortitude-alerts.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

scrape_configs:
  - job_name: 'fortitude'
    static_configs:
      - targets: ['fortitude:9090']
    scrape_interval: 5s
    metrics_path: '/metrics'
    
  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']
      
  - job_name: 'postgres-exporter'
    static_configs:
      - targets: ['postgres-exporter:9187']
      
  - job_name: 'redis-exporter'
    static_configs:
      - targets: ['redis-exporter:9121']

  - job_name: 'qdrant'
    static_configs:
      - targets: ['qdrant:6333']
    metrics_path: '/metrics'
```

### **Grafana Dashboard Configuration**

```json
{
  "dashboard": {
    "title": "Fortitude Enterprise Overview",
    "tags": ["fortitude", "enterprise"],
    "timezone": "browser",
    "panels": [
      {
        "title": "API Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "sum(rate(fortitude_api_requests_total[5m]))",
            "legendFormat": "Requests/sec"
          }
        ],
        "yAxes": [
          {
            "label": "Requests per second"
          }
        ]
      },
      {
        "title": "Provider Response Times",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, sum(rate(fortitude_provider_request_duration_seconds_bucket[5m])) by (le, provider))",
            "legendFormat": "{{provider}} 95th percentile"
          }
        ]
      },
      {
        "title": "Quality Scores",
        "type": "stat",
        "targets": [
          {
            "expr": "avg(fortitude_quality_score)",
            "legendFormat": "Average Quality Score"
          }
        ]
      },
      {
        "title": "System Health",
        "type": "table",
        "targets": [
          {
            "expr": "fortitude_component_health_status",
            "format": "table"
          }
        ]
      }
    ]
  }
}
```

### **Alert Rules**

```yaml
# monitoring/fortitude-alerts.yml
groups:
- name: fortitude.rules
  rules:
  - alert: FortitudeHighErrorRate
    expr: rate(fortitude_api_requests_total{status=~"5.."}[5m]) > 0.1
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High error rate detected"
      description: "Fortitude API error rate is {{ $value }} errors per second"
      
  - alert: FortitudeHighLatency
    expr: histogram_quantile(0.95, rate(fortitude_api_request_duration_seconds_bucket[5m])) > 2
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High API latency detected"
      description: "95th percentile latency is {{ $value }} seconds"
      
  - alert: FortitudeProviderDown
    expr: fortitude_provider_health_status == 0
    for: 2m
    labels:
      severity: critical
    annotations:
      summary: "Provider {{ $labels.provider }} is down"
      description: "Provider {{ $labels.provider }} has been unhealthy for more than 2 minutes"
      
  - alert: FortitudeQualityDegradation
    expr: avg(fortitude_quality_score) < 0.7
    for: 10m
    labels:
      severity: warning
    annotations:
      summary: "Quality score degradation"
      description: "Average quality score has dropped to {{ $value }}"
```

## <backup-recovery>Backup and Disaster Recovery</backup-recovery>

### **Backup Strategy**

```bash
#!/bin/bash
# backup-fortitude.sh

set -euo pipefail

BACKUP_DIR="/opt/fortitude-backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RETENTION_DAYS=30

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1"
}

# Create backup directory
mkdir -p $BACKUP_DIR/$TIMESTAMP

log "Starting Fortitude backup..."

# Backup PostgreSQL
log "Backing up PostgreSQL database..."
pg_dump postgresql://user:pass@localhost:5432/fortitude | gzip > $BACKUP_DIR/$TIMESTAMP/postgres_$TIMESTAMP.sql.gz

# Backup Redis (if persistent)
log "Backing up Redis data..."
redis-cli --rdb $BACKUP_DIR/$TIMESTAMP/redis_$TIMESTAMP.rdb

# Backup Qdrant collections
log "Backing up Qdrant vector database..."
curl -X POST http://localhost:6333/collections/fortitude_enterprise/snapshots
# Download snapshot (implementation depends on Qdrant version)

# Backup configuration files
log "Backing up configuration..."
tar -czf $BACKUP_DIR/$TIMESTAMP/config_$TIMESTAMP.tar.gz /etc/fortitude/

# Backup application data
log "Backing up application data..."
tar -czf $BACKUP_DIR/$TIMESTAMP/data_$TIMESTAMP.tar.gz /var/lib/fortitude/

# Create backup manifest
cat > $BACKUP_DIR/$TIMESTAMP/manifest.json << EOF
{
  "timestamp": "$TIMESTAMP",
  "components": {
    "postgres": "postgres_$TIMESTAMP.sql.gz",
    "redis": "redis_$TIMESTAMP.rdb",
    "config": "config_$TIMESTAMP.tar.gz",
    "data": "data_$TIMESTAMP.tar.gz"
  },
  "size_mb": $(du -sm $BACKUP_DIR/$TIMESTAMP | cut -f1),
  "fortitude_version": "$(docker run --rm fortitude:latest --version)"
}
EOF

# Upload to cloud storage (example with AWS S3)
if command -v aws &> /dev/null; then
    log "Uploading backup to S3..."
    aws s3 sync $BACKUP_DIR/$TIMESTAMP s3://company-fortitude-backups/$TIMESTAMP/
fi

# Cleanup old backups
log "Cleaning up old backups..."
find $BACKUP_DIR -maxdepth 1 -type d -mtime +$RETENTION_DAYS -exec rm -rf {} \;

log "Backup completed successfully"
```

### **Disaster Recovery Procedure**

```bash
#!/bin/bash
# restore-fortitude.sh

set -euo pipefail

BACKUP_TIMESTAMP=$1
BACKUP_DIR="/opt/fortitude-backups"

if [ -z "$BACKUP_TIMESTAMP" ]; then
    echo "Usage: $0 <backup_timestamp>"
    echo "Available backups:"
    ls -la $BACKUP_DIR/
    exit 1
fi

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1"
}

log "Starting Fortitude restore from backup $BACKUP_TIMESTAMP..."

# Stop services
log "Stopping Fortitude services..."
systemctl stop fortitude

# Restore PostgreSQL
log "Restoring PostgreSQL database..."
dropdb fortitude || true
createdb fortitude
gunzip -c $BACKUP_DIR/$BACKUP_TIMESTAMP/postgres_$BACKUP_TIMESTAMP.sql.gz | psql fortitude

# Restore Redis
log "Restoring Redis data..."
systemctl stop redis
cp $BACKUP_DIR/$BACKUP_TIMESTAMP/redis_$BACKUP_TIMESTAMP.rdb /var/lib/redis/dump.rdb
chown redis:redis /var/lib/redis/dump.rdb
systemctl start redis

# Restore configuration
log "Restoring configuration..."
tar -xzf $BACKUP_DIR/$BACKUP_TIMESTAMP/config_$BACKUP_TIMESTAMP.tar.gz -C /

# Restore application data
log "Restoring application data..."
tar -xzf $BACKUP_DIR/$BACKUP_TIMESTAMP/data_$BACKUP_TIMESTAMP.tar.gz -C /

# Restore Qdrant (manual process, depends on backup method)
log "Restoring Qdrant collections..."
# Implementation depends on Qdrant backup/restore procedures

# Start services
log "Starting Fortitude services..."
systemctl start fortitude

# Verify restore
log "Verifying restore..."
sleep 30
curl -f http://localhost:8080/api/health || {
    log "ERROR: Health check failed after restore"
    exit 1
}

log "Restore completed successfully"
```

## <performance>Performance Tuning</performance>

### **Database Optimization**

```sql
-- PostgreSQL optimization for Fortitude
-- /scripts/optimize-postgres.sql

-- Connection and memory settings
ALTER SYSTEM SET max_connections = 200;
ALTER SYSTEM SET shared_buffers = '2GB';
ALTER SYSTEM SET effective_cache_size = '6GB';
ALTER SYSTEM SET maintenance_work_mem = '512MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;
ALTER SYSTEM SET random_page_cost = 1.1;

-- Query optimization
ALTER SYSTEM SET work_mem = '64MB';
ALTER SYSTEM SET hash_mem_multiplier = 2.0;

-- Logging for performance analysis
ALTER SYSTEM SET log_min_duration_statement = 1000;  -- Log slow queries
ALTER SYSTEM SET log_statement = 'none';
ALTER SYSTEM SET log_duration = off;
ALTER SYSTEM SET log_lock_waits = on;

-- Apply settings
SELECT pg_reload_conf();

-- Create indexes for Fortitude tables
CREATE INDEX CONCURRENTLY idx_user_feedback_timestamp ON user_feedback(timestamp);
CREATE INDEX CONCURRENTLY idx_user_feedback_user_id ON user_feedback(user_id);
CREATE INDEX CONCURRENTLY idx_pattern_data_type ON pattern_data(pattern_type);
CREATE INDEX CONCURRENTLY idx_learning_data_created_at ON learning_data(created_at);
CREATE INDEX CONCURRENTLY idx_quality_metrics_provider ON quality_metrics(provider);
CREATE INDEX CONCURRENTLY idx_quality_metrics_timestamp ON quality_metrics(timestamp);

-- Analyze tables for query planning
ANALYZE user_feedback;
ANALYZE pattern_data;
ANALYZE learning_data;
ANALYZE quality_metrics;
```

### **Redis Optimization**

```conf
# redis.conf optimization for Fortitude

# Memory optimization
maxmemory 4gb
maxmemory-policy allkeys-lru
maxmemory-samples 5

# Persistence (adjust based on backup strategy)
save 900 1
save 300 10
save 60 10000
rdbcompression yes
rdbchecksum yes

# Network and connection optimization
tcp-keepalive 300
timeout 0
tcp-backlog 511
maxclients 10000

# Performance tuning
hash-max-ziplist-entries 512
hash-max-ziplist-value 64
list-max-ziplist-size -2
list-compress-depth 0
set-max-intset-entries 512
zset-max-ziplist-entries 128
zset-max-ziplist-value 64

# Logging
loglevel notice
logfile /var/log/redis/redis-server.log

# Security
requirepass your-redis-password
rename-command FLUSHDB ""
rename-command FLUSHALL ""
rename-command DEBUG ""
```

### **Qdrant Optimization**

```yaml
# qdrant-config.yaml
service:
  host: 0.0.0.0
  http_port: 6333
  grpc_port: 6334
  max_request_size_mb: 32
  max_workers: 0  # Auto-detect based on CPU cores

storage:
  # Storage optimizations
  performance:
    max_search_threads: 0  # Auto-detect
    max_optimization_threads: 0  # Auto-detect
  
  # Memory optimizations
  memory_threshold_mb: 16384  # 16GB threshold
  
  # On-disk optimizations
  hnsw_config:
    m: 16
    ef_construct: 100
    max_indexing_threads: 0  # Auto-detect

cluster:
  enabled: false  # Enable for multi-node setup

telemetry:
  disabled: true  # Disable for enterprise deployment
```

## <troubleshooting>Common Issues and Solutions</troubleshooting>

### **Deployment Issues**

<troubleshooting-guide>

**Issue**: Container startup failures
```bash
# Diagnosis
docker logs fortitude-app
kubectl logs -f deployment/fortitude-app

# Common solutions
# 1. Check resource limits
kubectl describe pod <pod-name>

# 2. Verify secrets
kubectl get secrets -n fortitude-enterprise

# 3. Check configuration
kubectl exec -it <pod-name> -- cat /etc/fortitude/production.yaml
```

**Issue**: Database connection failures
```bash
# Test connectivity
kubectl exec -it <pod-name> -- pg_isready -h postgres -p 5432

# Check credentials
kubectl get secret fortitude-db-credentials -o yaml

# Verify network policies
kubectl get networkpolicy -n fortitude-enterprise
```

**Issue**: Load balancer health check failures
```bash
# Check health endpoint
curl -f http://localhost:8080/api/health

# Verify readiness
curl -f http://localhost:8080/api/ready

# Check resource usage
kubectl top pods -n fortitude-enterprise
```

</troubleshooting-guide>

### **Performance Issues**

```bash
# Monitor resource usage
kubectl top nodes
kubectl top pods -n fortitude-enterprise

# Check for memory leaks
kubectl exec -it <pod-name> -- ps aux
kubectl exec -it <pod-name> -- free -h

# Database performance
kubectl exec -it postgres-pod -- psql -c "SELECT * FROM pg_stat_activity;"

# Redis performance
kubectl exec -it redis-pod -- redis-cli info memory
kubectl exec -it redis-pod -- redis-cli info stats
```

### **Security Issues**

```bash
# Check certificate status
kubectl get certificates -n fortitude-enterprise
kubectl describe certificate fortitude-tls

# Verify secrets rotation
kubectl get events -n fortitude-enterprise | grep secret

# Check network policies
kubectl get networkpolicies -n fortitude-enterprise -o yaml
```

## <maintenance>Ongoing Maintenance</maintenance>

### **Regular Maintenance Tasks**

```bash
# Weekly maintenance script
#!/bin/bash
# weekly-maintenance.sh

# Update container images
kubectl set image deployment/fortitude-app fortitude=fortitude:latest

# Restart rolling deployment
kubectl rollout restart deployment/fortitude-app

# Clean up old replica sets
kubectl delete replicaset $(kubectl get rs -o name | grep fortitude | head -n -3)

# Vacuum database
kubectl exec -it postgres-pod -- psql -c "VACUUM ANALYZE;"

# Clear Redis cache of old keys
kubectl exec -it redis-pod -- redis-cli EVAL "return redis.call('del', unpack(redis.call('keys', 'cache:*')))" 0

# Restart Qdrant for optimization
kubectl rollout restart deployment/qdrant

# Generate performance report
kubectl exec -it fortitude-pod -- fortitude reports weekly
```

### **Monitoring and Alerting Maintenance**

```bash
# Update Prometheus rules
kubectl apply -f monitoring/prometheus-rules.yaml

# Restart Grafana to load new dashboards
kubectl rollout restart deployment/grafana

# Test alert channels
kubectl exec -it alertmanager-pod -- amtool alert add alertname=test

# Clean up old metrics
kubectl exec -it prometheus-pod -- promtool query instant 'prometheus_tsdb_symbol_table_size_bytes'
```

## <references>See Also</references>

- [Multi-LLM Architecture](../architecture/multi-llm-architecture.md) - System architecture details
- [Multi-LLM Setup Guide](../user-guides/multi-llm-setup.md) - Configuration guide
- [Performance Tuning Guide](../performance/tuning-guide.md) - Optimization strategies
- [Security Guide](../security/compliance-guide.md) - Security configuration
- [Troubleshooting Guide](../troubleshooting/sprint-009-issues.md) - Issue resolution
- [Monitoring Configuration](../user-guides/learning-and-monitoring-configuration.md) - Monitoring setup