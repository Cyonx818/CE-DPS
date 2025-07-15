# Fortitude API - Integration Guide

This comprehensive guide provides everything you need to integrate with the Fortitude API Server, including examples in multiple programming languages, performance best practices, and real-world implementation scenarios.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Authentication](#authentication)
3. [Client Libraries](#client-libraries)
4. [API Endpoints](#api-endpoints)
5. [Performance Optimization](#performance-optimization)
6. [Error Handling](#error-handling)
7. [Rate Limiting](#rate-limiting)
8. [Caching Strategy](#caching-strategy)
9. [Integration Examples](#integration-examples)
10. [Testing & Validation](#testing--validation)
11. [Production Deployment](#production-deployment)
12. [Troubleshooting](#troubleshooting)

## Quick Start

### 1. Environment Setup

Set your environment variables:
```bash
export FORTITUDE_API_KEY="your-api-key-here"
export FORTITUDE_BASE_URL="http://localhost:8080"
```

### 2. Test Connectivity

```bash
# Test public health endpoint (no auth required)
curl http://localhost:8080/health

# Test authenticated endpoint
curl -H "X-API-Key: your-api-key-here" http://localhost:8080/api/v1/health/protected
```

### 3. Basic Request

```bash
curl -X POST http://localhost:8080/api/v1/research \
  -H "X-API-Key: your-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{"query": "AI-powered content classification", "priority": "high"}'
```

## Authentication

The Fortitude API uses API key authentication for all endpoints except the public health check.

### API Key Header
Include your API key in all authenticated requests:
```
X-API-Key: your-api-key-here
```

### API Key Management
- **Development**: Use environment variables
- **Production**: Use secure secret management (AWS Secrets Manager, HashiCorp Vault, etc.)
- **Rotation**: Implement key rotation for security
- **Scoping**: Different keys may have different permission levels

## Client Libraries

### cURL Examples
Located in: `examples/curl-examples/`

- **Basic Operations**: `01-basic-operations.sh`
- **Research**: `02-research-operations.sh`  
- **Classification**: `03-classification-operations.sh`
- **Cache Management**: `04-cache-management.sh`
- **Error Handling**: `05-error-handling.sh`
- **Performance Testing**: `06-performance-testing.sh`

Usage:
```bash
cd examples/curl-examples
export API_KEY="your-api-key"
./01-basic-operations.sh
```

### Python Client
Located in: `examples/python-client/`

#### Installation
```bash
pip install requests aiohttp
```

#### Basic Usage
```python
from fortitude_client import FortitudeClient

client = FortitudeClient()
result = client.research("Rust async programming patterns")
print(f"Found {result['data']['total_count']} results")
```

#### Async Usage
```python
import asyncio
from fortitude_client import AsyncFortitudeClient

async def main():
    async with AsyncFortitudeClient() as client:
        result = await client.research("Machine learning algorithms")
        print(result['data']['total_count'])

asyncio.run(main())
```

### JavaScript/Node.js Client
Located in: `examples/javascript-client/`

#### Installation
```bash
npm install
```

#### Basic Usage
```javascript
import { FortitudeClient } from './fortitude-client.js';

const client = new FortitudeClient();
const result = await client.research('API design patterns');
console.log(`Found ${result.data.total_count} results`);
```

#### Concurrent Requests
```javascript
import pLimit from 'p-limit';

const limit = pLimit(10);
const queries = ['query1', 'query2', 'query3'];

const results = await Promise.all(
  queries.map(query => 
    limit(() => client.research(query))
  )
);
```

### Rust Client
Located in: `examples/rust-client/`

#### Installation
Add to `Cargo.toml`:
```toml
[dependencies]
fortitude-client = { path = "path/to/rust-client" }
tokio = { version = "1.0", features = ["full"] }
```

#### Basic Usage
```rust
use fortitude_client::FortitudeClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = FortitudeClient::new()?;
    let result = client.research("Distributed systems patterns").await?;
    println!("Found {} results", result.total_count);
    Ok(())
}
```

#### Concurrent Requests
```rust
use futures::future::join_all;

let client = FortitudeClient::new()?;
let queries = vec!["query1", "query2", "query3"];

let tasks: Vec<_> = queries.into_iter()
    .map(|query| client.research(query))
    .collect();

let results = join_all(tasks).await;
```

### Postman Collection
Located in: `examples/fortitude-api.postman_collection.json`

Import the collection into Postman for:
- All API endpoints
- Authentication examples
- Error scenarios
- Performance testing
- Environment variable templates

## API Endpoints

### Health Endpoints

#### Public Health Check
```bash
GET /health
```
No authentication required. Returns basic server status.

#### Protected Health Check
```bash
GET /api/v1/health/protected
X-API-Key: your-api-key
```
Returns detailed health information for authenticated users.

### Research Endpoints

#### Perform Research
```bash
POST /api/v1/research
X-API-Key: your-api-key
Content-Type: application/json

{
  "query": "Your research query",
  "context": "Optional context",
  "priority": "medium",
  "audience_context": {
    "level": "intermediate",
    "domain": "technology",
    "format": "markdown"
  }
}
```

#### Get Research Result
```bash
GET /api/v1/research/{id}
X-API-Key: your-api-key
```

#### List Research Results
```bash
GET /api/v1/research?limit=20&offset=0&query=optional-filter
X-API-Key: your-api-key
```

### Classification Endpoints

#### Classify Content
```bash
POST /api/v1/classify
X-API-Key: your-api-key
Content-Type: application/json

{
  "content": "Content to classify",
  "categories": ["optional", "category", "filters"],
  "context_preferences": {
    "detect_urgency": true,
    "detect_audience": true,
    "detect_domain": true
  }
}
```

#### Get Classification Types
```bash
GET /api/v1/classify/types
X-API-Key: your-api-key
```

### Cache Endpoints

#### Get Cache Statistics
```bash
GET /api/v1/cache/stats
X-API-Key: your-api-key
```

#### Search Cache
```bash
GET /api/v1/cache/search?query=search-term&limit=20&sort=newest
X-API-Key: your-api-key
```

### Learning System Endpoints ✅ **NEW IN SPRINT 009**

#### Get Learning Dashboard Data
```bash
GET /api/v1/learning/dashboard
X-API-Key: your-api-key
```
Returns comprehensive learning system dashboard data including current metrics, health status, alerts, performance graphs, and system overview.

#### Get Learning Metrics
```bash
GET /api/v1/learning/metrics?duration=24h&detailed=true
X-API-Key: your-api-key
```
Returns current learning system metrics including adaptation metrics, storage metrics, pattern recognition metrics, feedback metrics, optimization metrics, and system metrics.

#### Get Learning Health Status
```bash
GET /api/v1/learning/health
X-API-Key: your-api-key
```
Returns learning system health status with component-level health checks and overall system health assessment.

#### Get Learning Performance Summary
```bash
GET /api/v1/learning/performance
X-API-Key: your-api-key
```
Returns performance summary with key metrics, active alerts, performance trends, and optimization recommendations.

### Monitoring System Endpoints ✅ **NEW IN SPRINT 009**

#### Get Monitoring Dashboard
```bash
GET /api/v1/monitoring/dashboard?time_range_hours=24&include_graphs=true
X-API-Key: your-api-key
```
Returns comprehensive monitoring dashboard data including current metrics, health status, active alerts, performance graphs, and system overview.

#### Get System Metrics
```bash
GET /api/v1/monitoring/metrics
X-API-Key: your-api-key
```
Returns current system metrics including API metrics, provider metrics, quality metrics, cache metrics, learning metrics, and resource metrics.

#### Get System Health
```bash
GET /api/v1/monitoring/health
X-API-Key: your-api-key
```
Returns system health status with component-level health checks and overall system health assessment.

#### Get System Alerts
```bash
GET /api/v1/monitoring/alerts?limit=50&offset=0
X-API-Key: your-api-key
```
Returns system alerts with pagination support, including alert severity, component information, and acknowledgment status.

#### Get Performance Summary
```bash
GET /api/v1/monitoring/performance
X-API-Key: your-api-key
```
Returns performance summary with key metrics, active alerts, performance trends, and optimization recommendations.

## Performance Optimization

### Caching Best Practices

1. **Leverage Cache Effectively**
   ```python
   # Identical queries benefit from caching
   result1 = client.research("Rust async patterns")  # Cache miss
   result2 = client.research("Rust async patterns")  # Cache hit (faster)
   ```

2. **Use Client-Side Caching**
   ```python
   client = FortitudeClient(enable_cache=True, cache_ttl=300)
   ```

3. **Batch Similar Requests**
   ```python
   # Better than individual requests
   results = client.list_research_results(limit=100)
   ```

### Concurrent Request Management

1. **Control Concurrency**
   ```python
   # Python with async
   semaphore = asyncio.Semaphore(10)  # Max 10 concurrent
   
   async def limited_request(query):
       async with semaphore:
           return await client.research(query)
   ```

2. **JavaScript Concurrency Control**
   ```javascript
   import pLimit from 'p-limit';
   const limit = pLimit(10);
   
   const results = await Promise.all(
     queries.map(query => limit(() => client.research(query)))
   );
   ```

3. **Rust Concurrency**
   ```rust
   use tokio::sync::Semaphore;
   let semaphore = Arc::new(Semaphore::new(10));
   ```

### Response Time Optimization

1. **Cache-Aware Request Patterns**
   ```python
   # Group similar queries together
   similar_queries = [
       "Rust async programming",
       "Rust async best practices", 
       "Rust async patterns"
   ]
   
   for query in similar_queries:
       result = client.research(query)  # Later queries benefit from cache
   ```

2. **Use Appropriate Priorities**
   ```json
   {
     "query": "Urgent production issue",
     "priority": "urgent"  // Higher priority processing
   }
   ```

## Error Handling

### HTTP Status Codes

| Status | Meaning | Action |
|--------|---------|--------|
| 200 | Success | Process response |
| 400 | Bad Request | Check request format |
| 401 | Unauthorized | Verify API key |
| 404 | Not Found | Check endpoint/resource ID |
| 429 | Rate Limited | Implement backoff |
| 500 | Server Error | Retry with backoff |

### Error Response Format
```json
{
  "error_code": "VALIDATION_ERROR",
  "message": "Query must be between 1 and 1000 characters",
  "details": "Additional error context",
  "request_id": "123e4567-e89b-12d3-a456-426614174000",
  "timestamp": "2024-01-15T10:30:00Z",
  "path": "/api/v1/research"
}
```

### Retry Strategies

#### Python Example
```python
import time
import random

def exponential_backoff_retry(func, max_retries=3):
    for attempt in range(max_retries + 1):
        try:
            return func()
        except FortitudeAPIError as e:
            if e.status_code in [429, 500, 502, 503, 504] and attempt < max_retries:
                delay = (2 ** attempt) + random.uniform(0, 1)
                time.sleep(delay)
                continue
            raise
```

#### JavaScript Example
```javascript
async function retryWithBackoff(fn, maxRetries = 3) {
  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await fn();
    } catch (error) {
      if (error.statusCode >= 500 && attempt < maxRetries) {
        const delay = Math.pow(2, attempt) * 1000 + Math.random() * 1000;
        await new Promise(resolve => setTimeout(resolve, delay));
        continue;
      }
      throw error;
    }
  }
}
```

## Rate Limiting

The API enforces rate limiting to ensure fair usage:

- **Rate**: 60 requests per minute per IP address
- **Headers**: Response includes rate limit headers
- **Handling**: Implement backoff on 429 responses

### Rate Limit Headers
```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1642234567
```

### Rate Limit Handling
```python
def handle_rate_limit(response):
    if response.status_code == 429:
        reset_time = int(response.headers.get('X-RateLimit-Reset', 0))
        current_time = int(time.time())
        sleep_time = max(reset_time - current_time, 1)
        time.sleep(sleep_time)
        return True
    return False
```

## Caching Strategy

### Cache Hit Rate Optimization

The API achieves >80% cache hit rates through intelligent caching:

1. **Query Normalization**: Similar queries share cache entries
2. **Semantic Caching**: Related content benefits from caching
3. **TTL Management**: Appropriate cache lifetimes

### Cache-Aware Client Design
```python
class CacheAwareClient:
    def __init__(self):
        self.client = FortitudeClient(enable_cache=True)
        self.local_cache = {}
    
    def research_with_fallback(self, query):
        # Check local cache first
        if query in self.local_cache:
            return self.local_cache[query]
        
        # Use API (benefits from server-side cache)
        result = self.client.research(query)
        self.local_cache[query] = result
        return result
```

## Integration Examples

### Web Application Integration

#### Flask/Python Example
```python
from flask import Flask, request, jsonify
from fortitude_client import FortitudeClient

app = Flask(__name__)
client = FortitudeClient()

@app.route('/api/research', methods=['POST'])
def research():
    data = request.get_json()
    try:
        result = client.research(data['query'])
        return jsonify(result['data'])
    except FortitudeAPIError as e:
        return jsonify({'error': str(e)}), e.status_code
```

#### Express/Node.js Example
```javascript
import express from 'express';
import { FortitudeClient } from './fortitude-client.js';

const app = express();
const client = new FortitudeClient();

app.post('/api/research', async (req, res) => {
  try {
    const result = await client.research(req.body.query);
    res.json(result.data);
  } catch (error) {
    res.status(error.statusCode || 500).json({ error: error.message });
  }
});
```

### Background Job Integration

#### Celery/Python Example
```python
from celery import Celery
from fortitude_client import FortitudeClient

app = Celery('research_worker')
client = FortitudeClient()

@app.task
def process_research_request(query, user_id):
    try:
        result = client.research(query)
        # Store result in database
        save_research_result(user_id, result['data'])
        return result['data']
    except Exception as e:
        # Handle errors appropriately
        log_error(f"Research failed for user {user_id}: {e}")
        raise
```

### Microservice Integration

#### Service-to-Service Communication
```python
class ResearchService:
    def __init__(self):
        self.fortitude_client = FortitudeClient()
        self.cache = Redis()
    
    async def enhanced_research(self, query, context=None):
        # Check local cache
        cache_key = f"research:{hash(query)}"
        cached = await self.cache.get(cache_key)
        if cached:
            return json.loads(cached)
        
        # Call Fortitude API
        result = await self.fortitude_client.research(query, context=context)
        
        # Cache result locally
        await self.cache.setex(cache_key, 300, json.dumps(result['data']))
        
        return result['data']
```

### Frontend Integration

#### React Example
```javascript
import React, { useState, useEffect } from 'react';

function ResearchComponent() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState([]);
  const [loading, setLoading] = useState(false);

  const handleSearch = async () => {
    setLoading(true);
    try {
      const response = await fetch('/api/research', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query })
      });
      const data = await response.json();
      setResults(data.results);
    } catch (error) {
      console.error('Research failed:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <input value={query} onChange={(e) => setQuery(e.target.value)} />
      <button onClick={handleSearch} disabled={loading}>
        {loading ? 'Searching...' : 'Search'}
      </button>
      {results.map(result => (
        <div key={result.id}>{result.title}</div>
      ))}
    </div>
  );
}
```

#### Learning Dashboard React Component ✅ **NEW IN SPRINT 009**
```javascript
import React, { useState, useEffect } from 'react';

function LearningDashboard() {
  const [dashboardData, setDashboardData] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchDashboardData = async () => {
      try {
        const response = await fetch('/api/v1/learning/dashboard', {
          headers: {
            'X-API-Key': process.env.REACT_APP_FORTITUDE_API_KEY,
          },
        });
        const data = await response.json();
        setDashboardData(data);
      } catch (error) {
        console.error('Failed to fetch learning dashboard:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchDashboardData();
    const interval = setInterval(fetchDashboardData, 30000); // Update every 30s
    return () => clearInterval(interval);
  }, []);

  if (loading) return <div>Loading learning dashboard...</div>;
  if (!dashboardData) return <div>Failed to load dashboard</div>;

  return (
    <div className="learning-dashboard">
      <h2>Learning System Dashboard</h2>
      
      <div className="system-overview">
        <h3>System Overview</h3>
        <div className="metrics-grid">
          <div className="metric">
            <span className="label">Total Adaptations:</span>
            <span className="value">{dashboardData.system_overview.total_adaptations}</span>
          </div>
          <div className="metric">
            <span className="label">Success Rate:</span>
            <span className="value">{(dashboardData.system_overview.success_rate * 100).toFixed(1)}%</span>
          </div>
          <div className="metric">
            <span className="label">Avg Response Time:</span>
            <span className="value">{dashboardData.system_overview.average_response_time.toFixed(1)}ms</span>
          </div>
          <div className="metric">
            <span className="label">Uptime:</span>
            <span className="value">{Math.floor(dashboardData.system_overview.uptime_seconds / 3600)}h</span>
          </div>
        </div>
      </div>
      
      <div className="health-status">
        <h3>Health Status: {dashboardData.health_status.overall_status}</h3>
        <div className="component-health">
          {dashboardData.health_status.component_results.map(component => (
            <div key={component.component} className={`component ${component.status.toLowerCase()}`}>
              <span className="component-name">{component.component}</span>
              <span className="component-status">{component.status}</span>
              <span className="response-time">{component.response_time_ms}ms</span>
            </div>
          ))}
        </div>
      </div>
      
      {dashboardData.alerts.length > 0 && (
        <div className="alerts">
          <h3>Active Alerts</h3>
          {dashboardData.alerts.map(alert => (
            <div key={alert.id} className={`alert ${alert.severity.toLowerCase()}`}>
              <span className="alert-component">{alert.component}</span>
              <span className="alert-message">{alert.message}</span>
              <span className="alert-time">{new Date(alert.timestamp).toLocaleString()}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
```

#### Monitoring Dashboard React Component ✅ **NEW IN SPRINT 009**
```javascript
import React, { useState, useEffect } from 'react';

function MonitoringDashboard() {
  const [dashboardData, setDashboardData] = useState(null);
  const [timeRange, setTimeRange] = useState(24);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchDashboardData = async () => {
      try {
        const response = await fetch(
          `/api/v1/monitoring/dashboard?time_range_hours=${timeRange}&include_graphs=true`,
          {
            headers: {
              'X-API-Key': process.env.REACT_APP_FORTITUDE_API_KEY,
            },
          }
        );
        const data = await response.json();
        setDashboardData(data);
      } catch (error) {
        console.error('Failed to fetch monitoring dashboard:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchDashboardData();
    const interval = setInterval(fetchDashboardData, 15000); // Update every 15s
    return () => clearInterval(interval);
  }, [timeRange]);

  if (loading) return <div>Loading monitoring dashboard...</div>;
  if (!dashboardData) return <div>Failed to load dashboard</div>;

  return (
    <div className="monitoring-dashboard">
      <h2>System Monitoring Dashboard</h2>
      
      <div className="time-range-selector">
        <label>Time Range: </label>
        <select value={timeRange} onChange={(e) => setTimeRange(Number(e.target.value))}>
          <option value={1}>1 Hour</option>
          <option value={6}>6 Hours</option>
          <option value={24}>24 Hours</option>
          <option value={168}>7 Days</option>
        </select>
      </div>
      
      <div className="system-overview">
        <h3>System Overview - Status: {dashboardData.overall_status}</h3>
        <div className="metrics-grid">
          <div className="metric">
            <span className="label">Total Operations:</span>
            <span className="value">{dashboardData.system_overview.total_operations.toLocaleString()}</span>
          </div>
          <div className="metric">
            <span className="label">Success Rate:</span>
            <span className="value">{(dashboardData.system_overview.success_rate * 100).toFixed(1)}%</span>
          </div>
          <div className="metric">
            <span className="label">Avg Response Time:</span>
            <span className="value">{dashboardData.system_overview.average_response_time_ms.toFixed(1)}ms</span>
          </div>
          <div className="metric">
            <span className="label">Resource Utilization:</span>
            <span className="value">{dashboardData.system_overview.resource_utilization.toFixed(1)}%</span>
          </div>
          <div className="metric">
            <span className="label">Active Alerts:</span>
            <span className="value">{dashboardData.system_overview.active_alerts_count}</span>
          </div>
        </div>
      </div>
      
      <div className="current-metrics">
        <h3>Current Metrics</h3>
        <div className="metrics-sections">
          <div className="api-metrics">
            <h4>API Performance</h4>
            <div className="metric-row">
              <span>Total Requests: {dashboardData.current_metrics.api_metrics.total_requests}</span>
              <span>Error Rate: {(dashboardData.current_metrics.api_metrics.error_rate * 100).toFixed(2)}%</span>
              <span>Avg Response: {dashboardData.current_metrics.api_metrics.average_response_time_ms.toFixed(1)}ms</span>
            </div>
          </div>
          
          <div className="resource-metrics">
            <h4>Resource Usage</h4>
            <div className="metric-row">
              <span>CPU: {dashboardData.current_metrics.resource_metrics.cpu_usage_percent.toFixed(1)}%</span>
              <span>Memory: {dashboardData.current_metrics.resource_metrics.memory_usage_mb.toFixed(1)}MB</span>
              <span>Network: {(dashboardData.current_metrics.resource_metrics.network_bytes_sent / 1024 / 1024).toFixed(1)}MB sent</span>
            </div>
          </div>
        </div>
      </div>
      
      {dashboardData.active_alerts.length > 0 && (
        <div className="alerts">
          <h3>Active Alerts ({dashboardData.active_alerts.length})</h3>
          {dashboardData.active_alerts.map(alert => (
            <div key={alert.id} className={`alert ${alert.severity.toLowerCase()}`}>
              <span className="alert-component">{alert.component}</span>
              <span className="alert-message">{alert.message}</span>
              <span className="alert-value">Value: {alert.metric_value} (Threshold: {alert.threshold})</span>
              <span className="alert-time">{new Date(alert.timestamp).toLocaleString()}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
```

## Testing & Validation

### Performance Testing

Run the included performance tests:

```bash
# Python performance tests
cd examples/python-client
python performance_test.py

# JavaScript performance tests
cd examples/javascript-client
npm run performance

# Rust performance tests
cd examples/rust-client
cargo run --bin performance_test

# Criterion benchmarks
cargo bench
```

### Load Testing

```bash
# cURL-based load testing
cd examples/curl-examples
./06-performance-testing.sh

# Custom load test
cargo test --test performance_tests test_full_performance_suite -- --ignored
```

### Integration Testing

```bash
# Test all endpoints
cd examples/curl-examples
./01-basic-operations.sh
./02-research-operations.sh
./03-classification-operations.sh
./04-cache-management.sh
```

### Postman Testing

1. Import `examples/fortitude-api.postman_collection.json`
2. Set environment variables:
   - `BASE_URL`: Your API server URL
   - `API_KEY`: Your API key
3. Run the full collection to validate all endpoints

## Production Deployment

### Environment Configuration

```bash
# Production environment variables
export FORTITUDE_API_KEY="production-api-key"
export FORTITUDE_BASE_URL="https://api.fortitude.your-domain.com"
export FORTITUDE_TIMEOUT="30"
export FORTITUDE_MAX_RETRIES="3"
```

### Load Balancing

When using multiple API server instances:

```nginx
upstream fortitude_api {
    server api1.fortitude.internal:8080;
    server api2.fortitude.internal:8080;
    server api3.fortitude.internal:8080;
}

server {
    location /api/ {
        proxy_pass http://fortitude_api;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header Host $host;
    }
}
```

### Monitoring Integration

```python
import logging
import time
from prometheus_client import Counter, Histogram

# Metrics
REQUEST_COUNT = Counter('fortitude_requests_total', 'Total requests', ['endpoint', 'status'])
REQUEST_DURATION = Histogram('fortitude_request_duration_seconds', 'Request duration')

class MonitoredFortitudeClient:
    def __init__(self):
        self.client = FortitudeClient()
    
    def research(self, query, **kwargs):
        start_time = time.time()
        try:
            result = self.client.research(query, **kwargs)
            REQUEST_COUNT.labels(endpoint='research', status='success').inc()
            return result
        except Exception as e:
            REQUEST_COUNT.labels(endpoint='research', status='error').inc()
            logging.error(f"Research request failed: {e}")
            raise
        finally:
            REQUEST_DURATION.observe(time.time() - start_time)
```

### Health Checks

```python
def health_check():
    """Application health check that includes Fortitude API connectivity"""
    try:
        client = FortitudeClient()
        client.get_health()
        return {"status": "healthy", "fortitude_api": "connected"}
    except Exception as e:
        return {"status": "unhealthy", "fortitude_api": f"error: {e}"}
```

## Troubleshooting

### Common Issues

#### Authentication Problems
```bash
# Test API key
curl -H "X-API-Key: your-key" http://localhost:8080/api/v1/health/protected

# Expected: 200 OK
# If 401: Check API key validity
# If 403: Check API key permissions
```

#### Connection Issues
```bash
# Test connectivity
curl http://localhost:8080/health

# If connection refused: Check server is running
# If timeout: Check network/firewall
# If 404: Check URL path
```

#### Performance Issues
```python
# Enable debug logging
import logging
logging.basicConfig(level=logging.DEBUG)

# Check response times
start = time.time()
result = client.research("test query")
print(f"Request took {time.time() - start:.2f}s")

# Expected: <1s for cached, <5s for new queries
```

#### Rate Limiting
```python
# Handle rate limits gracefully
try:
    result = client.research("query")
except FortitudeAPIError as e:
    if e.status_code == 429:
        print("Rate limited - implement backoff")
        time.sleep(60)  # Wait for rate limit reset
```

### Debug Mode

Enable debug logging in clients:

```python
# Python
import logging
logging.getLogger('fortitude_client').setLevel(logging.DEBUG)

# Environment variable
export FORTITUDE_LOG_LEVEL=debug
```

```javascript
// JavaScript
const client = new FortitudeClient({
  debug: true,
  logLevel: 'debug'
});
```

```rust
// Rust
RUST_LOG=debug cargo run
```

### Performance Analysis

Monitor key metrics:

1. **Response Times**
   - Cached requests: <100ms
   - New requests: <1000ms
   - P95 response time: <2000ms

2. **Success Rates**
   - Target: >95%
   - Alert if: <90%

3. **Cache Hit Rates**
   - Target: >80%
   - Alert if: <70%

4. **Error Rates**
   - Target: <5%
   - Alert if: >10%

### Support Resources

- **Documentation**: Complete API documentation in OpenAPI format
- **Examples**: Comprehensive examples in multiple languages
- **Performance Reports**: Detailed performance validation reports
- **Monitoring**: Built-in metrics and health checks
- **Testing**: Complete test suites for validation

---

This integration guide provides a complete foundation for integrating with the Fortitude API. For specific implementation questions or advanced use cases, refer to the detailed examples in the `examples/` directory or the comprehensive test suites.