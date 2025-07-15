# Proactive Research API Guide

<meta>
  <title>Proactive Research API - HTTP Integration Guide</title>
  <type>user_guide</type>
  <audience>developer</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-11</updated>
  <sprint>008</sprint>
  <version>1.0</version>
</meta>

## <summary priority="high">API Overview</summary>
- **Base URL**: `http://localhost:8080/api/v1` (configurable)
- **Authentication**: API key via `X-API-Key` header
- **Content Type**: `application/json` for requests and responses
- **Rate Limiting**: 60 requests per minute per API key (configurable)
- **WebSocket Support**: Real-time status updates and notifications

## <overview>API Architecture</overview>

The Proactive Research API provides comprehensive HTTP endpoints for integrating the proactive research system with external tools, IDEs, and custom applications.

### <authentication>Authentication</authentication>

All API endpoints (except `/health`) require authentication using API keys:

```http
X-API-Key: your-api-key-here
```

**Get API Key:**
```bash
# Generate new API key via CLI
fortitude proactive configure generate-api-key --name "My Integration"

# List existing keys
fortitude proactive configure list-api-keys

# Revoke API key
fortitude proactive configure revoke-api-key --key-id abc123
```

### <rate-limiting>Rate Limiting</rate-limiting>

- **Default Limit**: 60 requests per minute per API key
- **Headers**: Rate limit information included in response headers
- **Burst Protection**: Short burst allowance with gradual rate limiting

**Rate Limit Headers:**
```http
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1625097600
X-RateLimit-Retry-After: 30
```

### <response-format>Response Format</response-format>

**Success Response:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-07-11T14:30:15.123Z",
  "success": true,
  "data": {
    // Response data
  }
}
```

**Error Response:**
```json
{
  "error_code": "INVALID_REQUEST",
  "message": "Gap interval must be between 30 and 3600 seconds",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-07-11T14:30:15.123Z",
  "details": {
    "field": "gap_interval_seconds",
    "provided_value": 10,
    "valid_range": [30, 3600]
  }
}
```

## <endpoints>API Endpoints</endpoints>

### <health-endpoints>Health and Status Endpoints</health-endpoints>

#### <health-check>Health Check</health-check>

**Public endpoint - no authentication required**

```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-07-11T14:30:15.123Z",
  "version": "1.0.0",
  "uptime_seconds": 86400
}
```

#### <protected-health>Protected Health Check</protected-health>

```http
GET /api/v1/health/protected
```

**Headers:**
```http
X-API-Key: your-api-key
```

**Response:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-07-11T14:30:15.123Z",
  "success": true,
  "data": {
    "status": "healthy",
    "api_version": "v1",
    "features_enabled": ["proactive_research", "notifications", "webhooks"],
    "rate_limit_remaining": 58
  }
}
```

### <proactive-endpoints>Proactive Research Endpoints</proactive-endpoints>

#### <start-proactive>Start Proactive Research</start-proactive>

```http
POST /api/v1/proactive/start
```

**Request Body:**
```json
{
  "base_directory": "/path/to/project",
  "gap_interval_seconds": 300,
  "max_concurrent_tasks": 3,
  "file_patterns": ["*.rs", "*.md", "*.toml"],
  "ignore_patterns": ["target/**", "node_modules/**"],
  "notification_channels": ["webhook", "console"],
  "priority_threshold": 0.7,
  "performance_limits": {
    "max_memory_mb": 2048,
    "max_cpu_percent": 80
  }
}
```

**Response:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-07-11T14:30:15.123Z",
  "success": true,
  "data": {
    "status": "started",
    "session_id": "session_abc123",
    "configuration": {
      "base_directory": "/path/to/project",
      "gap_interval_seconds": 300,
      "max_concurrent_tasks": 3,
      "estimated_memory_usage_mb": 456
    },
    "monitoring": {
      "files_discovered": 247,
      "file_types": ["rs", "md", "toml", "js"],
      "excluded_files": 1432
    }
  }
}
```

#### <stop-proactive>Stop Proactive Research</stop-proactive>

```http
POST /api/v1/proactive/stop
```

**Request Body:**
```json
{
  "timeout_seconds": 30,
  "force": false,
  "save_state": true
}
```

**Response:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-07-11T14:30:15.123Z",
  "success": true,
  "data": {
    "status": "stopped",
    "shutdown_duration_seconds": 12,
    "final_statistics": {
      "uptime_seconds": 7200,
      "gaps_detected": 14,
      "tasks_completed": 12,
      "notifications_sent": 8
    }
  }
}
```

#### <get-status>Get System Status</get-status>

```http
GET /api/v1/proactive/status?detailed=true&metrics=true&recent_minutes=60
```

**Query Parameters:**
- `detailed` (boolean): Include detailed status information
- `metrics` (boolean): Include performance metrics
- `recent_minutes` (integer): Show activity from last N minutes
- `format` (string): Response format (json, compact)

**Response:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-07-11T14:30:15.123Z",
  "success": true,
  "data": {
    "system_status": {
      "is_running": true,
      "uptime_seconds": 7200,
      "session_id": "session_abc123",
      "started_at": "2025-07-11T12:30:15.123Z"
    },
    "gap_detection": {
      "status": "active",
      "last_scan": "2025-07-11T14:28:45.123Z",
      "next_scan": "2025-07-11T14:33:45.123Z",
      "files_monitored": 247,
      "gaps_detected_today": 8
    },
    "background_research": {
      "active_tasks": 2,
      "max_concurrent_tasks": 3,
      "queue_length": 1,
      "completed_today": 15,
      "success_rate": 0.94
    },
    "notifications": {
      "delivered_today": 12,
      "channels": ["desktop", "webhook"],
      "rate_limit_remaining": 8
    },
    "performance": {
      "cpu_usage_percent": 12.3,
      "memory_usage_mb": 456,
      "memory_limit_mb": 2048,
      "disk_io_mb_per_sec": 1.2,
      "network_kb_per_sec": 245
    },
    "recent_activity": [
      {
        "timestamp": "2025-07-11T14:28:45.123Z",
        "type": "gap_detected",
        "description": "TODO comment found in src/main.rs:45",
        "priority": "medium"
      },
      {
        "timestamp": "2025-07-11T14:26:12.123Z",
        "type": "task_completed",
        "description": "Research completed for 'Rust error handling patterns'",
        "duration_seconds": 204
      }
    ]
  }
}
```

### <configuration-endpoints>Configuration Endpoints</configuration-endpoints>

#### <get-configuration>Get Configuration</get-configuration>

```http
GET /api/v1/proactive/config?section=gap_analysis&sources=true
```

**Query Parameters:**
- `section` (string): Specific configuration section
- `sources` (boolean): Include configuration sources
- `effective` (boolean): Show effective (merged) configuration

**Response:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-07-11T14:30:15.123Z",
  "success": true,
  "data": {
    "configuration": {
      "gap_analysis": {
        "scan_intervals_seconds": 300,
        "confidence_threshold": 0.7,
        "enable_semantic_analysis": true,
        "file_patterns": ["*.rs", "*.md", "*.toml"]
      },
      "background_research": {
        "max_concurrent_tasks": 3,
        "rate_limit_requests_per_minute": 50
      }
    },
    "sources": {
      "gap_analysis.scan_intervals_seconds": "project_config",
      "gap_analysis.confidence_threshold": "user_config",
      "background_research.max_concurrent_tasks": "environment"
    }
  }
}
```

#### <update-configuration>Update Configuration</update-configuration>

```http
PUT /api/v1/proactive/config
```

**Request Body:**
```json
{
  "gap_analysis": {
    "scan_intervals_seconds": 240,
    "confidence_threshold": 0.8
  },
  "notifications": {
    "rate_limiting": {
      "max_per_hour": 15
    }
  },
  "validate": true,
  "backup": true
}
```

**Response:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-07-11T14:30:15.123Z",
  "success": true,
  "data": {
    "updated": true,
    "backup_created": "config_backup_20250711_143015.json",
    "validation_result": {
      "valid": true,
      "warnings": [
        "Reduced scan interval may increase CPU usage"
      ]
    },
    "changes_applied": {
      "gap_analysis.scan_intervals_seconds": {
        "old_value": 300,
        "new_value": 240
      },
      "gap_analysis.confidence_threshold": {
        "old_value": 0.7,
        "new_value": 0.8
      }
    }
  }
}
```

### <tasks-endpoints>Tasks Endpoints</tasks-endpoints>

#### <list-tasks>List Research Tasks</list-tasks>

```http
GET /api/v1/proactive/tasks?status=active&priority=high&limit=20&since=2025-07-11T12:00:00Z
```

**Query Parameters:**
- `status` (string): Filter by status (pending, active, completed, failed)
- `priority` (string): Filter by priority (low, medium, high, critical)
- `limit` (integer): Maximum number of results
- `offset` (integer): Pagination offset
- `since` (ISO 8601): Show tasks since timestamp
- `task_type` (string): Filter by task type

**Response:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-07-11T14:30:15.123Z",
  "success": true,
  "data": {
    "tasks": [
      {
        "task_id": "task_789",
        "status": "active",
        "priority": "high",
        "description": "Error handling patterns in Rust",
        "created_at": "2025-07-11T14:15:30.123Z",
        "started_at": "2025-07-11T14:16:00.123Z",
        "progress_percent": 85,
        "estimated_completion": "2025-07-11T14:32:00.123Z",
        "gap_source": {
          "file_path": "src/main.rs",
          "line_number": 45,
          "gap_type": "todo_comment"
        }
      }
    ],
    "pagination": {
      "total": 156,
      "limit": 20,
      "offset": 0,
      "has_more": true
    },
    "statistics": {
      "active": 2,
      "pending": 1,
      "completed_today": 15,
      "failed_today": 1
    }
  }
}
```

#### <get-task>Get Task Details</get-task>

```http
GET /api/v1/proactive/tasks/task_789?include_research=true&include_sources=true
```

**Query Parameters:**
- `include_research` (boolean): Include research results
- `include_sources` (boolean): Include source information
- `include_history` (boolean): Include task history

**Response:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-07-11T14:30:15.123Z",
  "success": true,
  "data": {
    "task": {
      "task_id": "task_789",
      "status": "active",
      "priority": "high",
      "description": "Error handling patterns in Rust",
      "created_at": "2025-07-11T14:15:30.123Z",
      "started_at": "2025-07-11T14:16:00.123Z",
      "progress_percent": 85,
      "gap_source": {
        "file_path": "src/main.rs",
        "line_number": 45,
        "gap_type": "todo_comment",
        "context": "// TODO: Add proper error handling here"
      },
      "research_results": {
        "sources_found": 5,
        "confidence_score": 0.89,
        "key_findings": [
          "Result<T, E> is the standard error handling type in Rust",
          "Use ? operator for error propagation",
          "Consider custom error types for complex applications"
        ],
        "code_examples": [
          {
            "title": "Basic error handling with Result",
            "code": "fn divide(a: f64, b: f64) -> Result<f64, String> {\n    if b == 0.0 {\n        Err(\"Division by zero\".to_string())\n    } else {\n        Ok(a / b)\n    }\n}",
            "explanation": "Simple error handling using Result type"
          }
        ],
        "related_resources": [
          {
            "title": "The Rust Programming Language - Error Handling",
            "url": "https://doc.rust-lang.org/book/ch09-00-error-handling.html",
            "relevance_score": 0.95
          }
        ]
      }
    }
  }
}
```

#### <create-task>Create Research Task</create-task>

```http
POST /api/v1/proactive/tasks
```

**Request Body:**
```json
{
  "description": "Best practices for async/await in Rust",
  "priority": "medium",
  "task_type": "manual_research",
  "context": {
    "file_path": "src/async_handler.rs",
    "line_number": 23,
    "surrounding_code": "// Context around the area needing research"
  },
  "requirements": {
    "min_sources": 3,
    "min_confidence": 0.8,
    "include_examples": true,
    "max_duration_seconds": 300
  },
  "tags": ["rust", "async", "performance"]
}
```

**Response:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-07-11T14:30:15.123Z",
  "success": true,
  "data": {
    "task_id": "task_792",
    "status": "pending",
    "priority": "medium",
    "description": "Best practices for async/await in Rust",
    "created_at": "2025-07-11T14:30:15.123Z",
    "estimated_start": "2025-07-11T14:32:00.123Z",
    "queue_position": 2
  }
}
```

#### <cancel-task>Cancel Task</cancel-task>

```http
DELETE /api/v1/proactive/tasks/task_792
```

**Request Body:**
```json
{
  "reason": "Duplicate research request",
  "force": false
}
```

**Response:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-07-11T14:30:15.123Z",
  "success": true,
  "data": {
    "task_id": "task_792",
    "status": "cancelled",
    "cancelled_at": "2025-07-11T14:30:15.123Z",
    "reason": "Duplicate research request"
  }
}
```

### <notifications-endpoints>Notifications Endpoints</notifications-endpoints>

#### <list-notifications>List Notifications</list-notifications>

```http
GET /api/v1/proactive/notifications?unread=true&channel=webhook&limit=50
```

**Query Parameters:**
- `unread` (boolean): Show only unread notifications
- `channel` (string): Filter by channel
- `type` (string): Filter by notification type
- `since` (ISO 8601): Show notifications since timestamp
- `limit` (integer): Maximum number of results

**Response:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-07-11T14:30:15.123Z",
  "success": true,
  "data": {
    "notifications": [
      {
        "notification_id": "notif_456",
        "type": "gap_detected",
        "priority": "medium",
        "title": "Knowledge Gap Detected",
        "message": "TODO comment found: 'Add error handling documentation'",
        "created_at": "2025-07-11T14:25:30.123Z",
        "read": false,
        "channel": "webhook",
        "metadata": {
          "task_id": "task_789",
          "file_path": "src/main.rs",
          "line_number": 45
        }
      }
    ],
    "pagination": {
      "total": 89,
      "limit": 50,
      "offset": 0,
      "has_more": true
    },
    "summary": {
      "unread_count": 12,
      "total_today": 23,
      "by_type": {
        "gap_detected": 8,
        "task_completed": 12,
        "system_alert": 3
      }
    }
  }
}
```

#### <mark-notifications-read>Mark Notifications as Read</mark-notifications-read>

```http
PUT /api/v1/proactive/notifications/read
```

**Request Body:**
```json
{
  "notification_ids": ["notif_456", "notif_457"],
  "mark_all": false,
  "since": "2025-07-11T12:00:00Z"
}
```

**Response:**
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-07-11T14:30:15.123Z",
  "success": true,
  "data": {
    "marked_read": 2,
    "notification_ids": ["notif_456", "notif_457"]
  }
}
```

#### <notification-preferences>Get/Set Notification Preferences</notification-preferences>

**Get Preferences:**
```http
GET /api/v1/proactive/notifications/preferences
```

**Set Preferences:**
```http
PUT /api/v1/proactive/notifications/preferences
```

**Request Body:**
```json
{
  "channels": ["webhook", "console"],
  "frequency": "immediate",
  "min_priority": "medium",
  "quiet_hours": {
    "enabled": true,
    "start": "22:00",
    "end": "08:00",
    "timezone": "UTC"
  },
  "rate_limiting": {
    "max_per_hour": 10,
    "enable_burst_protection": true
  },
  "webhook_settings": {
    "url": "https://your-app.com/webhooks/fortitude",
    "headers": {
      "Authorization": "Bearer your-token"
    },
    "retry_attempts": 3
  }
}
```

## <client-libraries>Client Libraries and Examples</client-libraries>

### <javascript-client>JavaScript Client</javascript-client>

```javascript
// fortitude-client.js
class FortitudeClient {
  constructor(baseUrl, apiKey) {
    this.baseUrl = baseUrl;
    this.apiKey = apiKey;
    this.headers = {
      'Content-Type': 'application/json',
      'X-API-Key': apiKey
    };
  }

  async request(method, endpoint, data = null) {
    const config = {
      method,
      headers: this.headers
    };

    if (data) {
      config.body = JSON.stringify(data);
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, config);
    
    if (!response.ok) {
      const error = await response.json();
      throw new Error(`API Error: ${error.message}`);
    }

    return await response.json();
  }

  // Proactive research methods
  async startProactiveResearch(config) {
    return this.request('POST', '/api/v1/proactive/start', config);
  }

  async stopProactiveResearch(options = {}) {
    return this.request('POST', '/api/v1/proactive/stop', options);
  }

  async getStatus(options = {}) {
    const params = new URLSearchParams(options);
    return this.request('GET', `/api/v1/proactive/status?${params}`);
  }

  async listTasks(filters = {}) {
    const params = new URLSearchParams(filters);
    return this.request('GET', `/api/v1/proactive/tasks?${params}`);
  }

  async getTask(taskId, options = {}) {
    const params = new URLSearchParams(options);
    return this.request('GET', `/api/v1/proactive/tasks/${taskId}?${params}`);
  }

  async createTask(taskData) {
    return this.request('POST', '/api/v1/proactive/tasks', taskData);
  }

  async listNotifications(filters = {}) {
    const params = new URLSearchParams(filters);
    return this.request('GET', `/api/v1/proactive/notifications?${params}`);
  }

  async markNotificationsRead(data) {
    return this.request('PUT', '/api/v1/proactive/notifications/read', data);
  }
}

// Usage example
const client = new FortitudeClient('http://localhost:8080', 'your-api-key');

async function main() {
  try {
    // Start proactive research
    const startResult = await client.startProactiveResearch({
      base_directory: '/path/to/project',
      gap_interval_seconds: 300,
      max_concurrent_tasks: 3
    });
    console.log('Started:', startResult.data);

    // Monitor status
    const status = await client.getStatus({ detailed: true, metrics: true });
    console.log('Status:', status.data);

    // List active tasks
    const tasks = await client.listTasks({ status: 'active' });
    console.log('Active tasks:', tasks.data.tasks.length);

  } catch (error) {
    console.error('Error:', error.message);
  }
}

main();
```

### <python-client>Python Client</python-client>

```python
# fortitude_client.py
import requests
import json
from typing import Dict, Any, Optional
from urllib.parse import urlencode

class FortitudeClient:
    def __init__(self, base_url: str, api_key: str):
        self.base_url = base_url
        self.api_key = api_key
        self.session = requests.Session()
        self.session.headers.update({
            'Content-Type': 'application/json',
            'X-API-Key': api_key
        })

    def _request(self, method: str, endpoint: str, data: Optional[Dict] = None, params: Optional[Dict] = None) -> Dict[str, Any]:
        url = f"{self.base_url}{endpoint}"
        
        kwargs = {}
        if data:
            kwargs['json'] = data
        if params:
            kwargs['params'] = params

        response = self.session.request(method, url, **kwargs)
        
        if not response.ok:
            error_data = response.json()
            raise Exception(f"API Error: {error_data.get('message', 'Unknown error')}")
        
        return response.json()

    def start_proactive_research(self, config: Dict[str, Any]) -> Dict[str, Any]:
        return self._request('POST', '/api/v1/proactive/start', data=config)

    def stop_proactive_research(self, options: Dict[str, Any] = None) -> Dict[str, Any]:
        return self._request('POST', '/api/v1/proactive/stop', data=options or {})

    def get_status(self, **options) -> Dict[str, Any]:
        return self._request('GET', '/api/v1/proactive/status', params=options)

    def list_tasks(self, **filters) -> Dict[str, Any]:
        return self._request('GET', '/api/v1/proactive/tasks', params=filters)

    def get_task(self, task_id: str, **options) -> Dict[str, Any]:
        return self._request('GET', f'/api/v1/proactive/tasks/{task_id}', params=options)

    def create_task(self, task_data: Dict[str, Any]) -> Dict[str, Any]:
        return self._request('POST', '/api/v1/proactive/tasks', data=task_data)

    def list_notifications(self, **filters) -> Dict[str, Any]:
        return self._request('GET', '/api/v1/proactive/notifications', params=filters)

    def mark_notifications_read(self, data: Dict[str, Any]) -> Dict[str, Any]:
        return self._request('PUT', '/api/v1/proactive/notifications/read', data=data)

# Usage example
def main():
    client = FortitudeClient('http://localhost:8080', 'your-api-key')
    
    try:
        # Start proactive research
        start_result = client.start_proactive_research({
            'base_directory': '/path/to/project',
            'gap_interval_seconds': 300,
            'max_concurrent_tasks': 3
        })
        print(f"Started: {start_result['data']}")

        # Monitor status
        status = client.get_status(detailed=True, metrics=True)
        print(f"Status: {status['data']['system_status']['is_running']}")

        # List recent notifications
        notifications = client.list_notifications(unread=True, limit=10)
        print(f"Unread notifications: {len(notifications['data']['notifications'])}")

    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    main()
```

### <rust-client>Rust Client</rust-client>

```rust
// fortitude-client/src/lib.rs
use reqwest::{Client, header::{HeaderMap, HeaderValue}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FortitudeError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error: {message}")]
    Api { message: String },
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, FortitudeError>;

#[derive(Debug, Clone)]
pub struct FortitudeClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Serialize)]
pub struct StartProactiveConfig {
    pub base_directory: String,
    pub gap_interval_seconds: u64,
    pub max_concurrent_tasks: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_patterns: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_channels: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub request_id: String,
    pub timestamp: String,
    pub success: bool,
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct SystemStatus {
    pub is_running: bool,
    pub uptime_seconds: u64,
    pub session_id: Option<String>,
}

impl FortitudeClient {
    pub fn new(base_url: String, api_key: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("X-API-Key", HeaderValue::from_str(&api_key).unwrap());

        let client = Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self { client, base_url })
    }

    async fn request<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<Value>,
        query: Option<&[(&str, &str)]>,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut request = self.client.request(method, &url);

        if let Some(query_params) = query {
            request = request.query(query_params);
        }

        if let Some(json_body) = body {
            request = request.json(&json_body);
        }

        let response = request.send().await?;

        if response.status().is_success() {
            let api_response: ApiResponse<T> = response.json().await?;
            Ok(api_response.data)
        } else {
            let error: Value = response.json().await?;
            let message = error.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown API error")
                .to_string();
            Err(FortitudeError::Api { message })
        }
    }

    pub async fn start_proactive_research(&self, config: StartProactiveConfig) -> Result<Value> {
        self.request(
            reqwest::Method::POST,
            "/api/v1/proactive/start",
            Some(serde_json::to_value(config)?),
            None,
        ).await
    }

    pub async fn stop_proactive_research(&self) -> Result<Value> {
        self.request(
            reqwest::Method::POST,
            "/api/v1/proactive/stop",
            Some(serde_json::json!({})),
            None,
        ).await
    }

    pub async fn get_status(&self, detailed: bool, metrics: bool) -> Result<SystemStatus> {
        let query = vec![
            ("detailed", if detailed { "true" } else { "false" }),
            ("metrics", if metrics { "true" } else { "false" }),
        ];

        self.request(
            reqwest::Method::GET,
            "/api/v1/proactive/status",
            None,
            Some(&query),
        ).await
    }

    pub async fn list_tasks(&self, status: Option<&str>, limit: Option<u32>) -> Result<Value> {
        let mut query = Vec::new();
        
        if let Some(s) = status {
            query.push(("status", s));
        }
        if let Some(l) = limit {
            query.push(("limit", &l.to_string()));
        }

        let query_slice = if query.is_empty() { None } else { Some(query.as_slice()) };

        self.request(
            reqwest::Method::GET,
            "/api/v1/proactive/tasks",
            None,
            query_slice,
        ).await
    }
}

// Usage example
#[tokio::main]
async fn main() -> Result<()> {
    let client = FortitudeClient::new(
        "http://localhost:8080".to_string(),
        "your-api-key".to_string(),
    )?;

    // Start proactive research
    let config = StartProactiveConfig {
        base_directory: "/path/to/project".to_string(),
        gap_interval_seconds: 300,
        max_concurrent_tasks: 3,
        file_patterns: Some(vec!["*.rs".to_string(), "*.md".to_string()]),
        notification_channels: Some(vec!["webhook".to_string()]),
    };

    let start_result = client.start_proactive_research(config).await?;
    println!("Started: {:?}", start_result);

    // Get status
    let status = client.get_status(true, true).await?;
    println!("Running: {}", status.is_running);

    // List active tasks
    let tasks = client.list_tasks(Some("active"), Some(10)).await?;
    println!("Tasks: {:?}", tasks);

    Ok(())
}
```

## <webhooks>Webhook Integration</webhooks>

### <webhook-setup>Setting Up Webhooks</webhook-setup>

Configure webhook endpoints to receive real-time notifications:

```http
PUT /api/v1/proactive/notifications/preferences
```

```json
{
  "channels": ["webhook"],
  "webhook_settings": {
    "url": "https://your-app.com/webhooks/fortitude",
    "headers": {
      "Authorization": "Bearer your-webhook-token",
      "X-Webhook-Source": "fortitude"
    },
    "retry_attempts": 3,
    "timeout_seconds": 30,
    "verify_ssl": true
  }
}
```

### <webhook-payloads>Webhook Payloads</webhook-payloads>

**Gap Detected Webhook:**
```json
{
  "event_type": "gap_detected",
  "timestamp": "2025-07-11T14:30:15.123Z",
  "webhook_id": "webhook_123",
  "data": {
    "gap_id": "gap_456",
    "task_id": "task_789",
    "priority": "medium",
    "gap_type": "todo_comment",
    "description": "TODO comment found: 'Add error handling documentation'",
    "file_path": "src/main.rs",
    "line_number": 45,
    "context": "// TODO: Add proper error handling here\nfn process_data(data: &str) -> String {",
    "confidence_score": 0.78
  }
}
```

**Task Completed Webhook:**
```json
{
  "event_type": "task_completed",
  "timestamp": "2025-07-11T14:35:22.456Z",
  "webhook_id": "webhook_124",
  "data": {
    "task_id": "task_789",
    "description": "Error handling patterns in Rust",
    "duration_seconds": 312,
    "success": true,
    "research_results": {
      "sources_found": 5,
      "confidence_score": 0.89,
      "summary": "Found comprehensive error handling patterns using Result<T, E> type..."
    }
  }
}
```

### <webhook-verification>Webhook Verification</webhook-verification>

Verify webhook authenticity using HMAC signatures:

```javascript
const crypto = require('crypto');

function verifyWebhook(payload, signature, secret) {
  const expectedSignature = crypto
    .createHmac('sha256', secret)
    .update(payload)
    .digest('hex');
  
  return crypto.timingSafeEqual(
    Buffer.from(signature, 'hex'),
    Buffer.from(expectedSignature, 'hex')
  );
}

// Express.js webhook handler
app.post('/webhooks/fortitude', (req, res) => {
  const signature = req.headers['x-fortitude-signature'];
  const payload = JSON.stringify(req.body);
  
  if (!verifyWebhook(payload, signature, process.env.WEBHOOK_SECRET)) {
    return res.status(401).json({ error: 'Invalid signature' });
  }
  
  // Process webhook payload
  const { event_type, data } = req.body;
  
  switch (event_type) {
    case 'gap_detected':
      handleGapDetected(data);
      break;
    case 'task_completed':
      handleTaskCompleted(data);
      break;
    default:
      console.log('Unknown event type:', event_type);
  }
  
  res.status(200).json({ received: true });
});
```

## <error-handling>Error Handling</error-handling>

### <error-codes>Common Error Codes</error-codes>

| Error Code | HTTP Status | Description |
|------------|-------------|-------------|
| `INVALID_API_KEY` | 401 | API key missing or invalid |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |
| `INVALID_REQUEST` | 400 | Malformed request body or parameters |
| `RESOURCE_NOT_FOUND` | 404 | Task, notification, or resource not found |
| `CONFIGURATION_ERROR` | 400 | Invalid configuration values |
| `SYSTEM_NOT_RUNNING` | 409 | Proactive research system not started |
| `CONCURRENT_OPERATION` | 409 | Conflicting operation in progress |
| `INTERNAL_ERROR` | 500 | Server-side error |

### <retry-logic>Implementing Retry Logic</retry-logic>

```javascript
class FortitudeClientWithRetry extends FortitudeClient {
  async requestWithRetry(method, endpoint, data = null, maxRetries = 3) {
    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        return await this.request(method, endpoint, data);
      } catch (error) {
        const isRetryable = this.isRetryableError(error);
        const isLastAttempt = attempt === maxRetries;
        
        if (!isRetryable || isLastAttempt) {
          throw error;
        }
        
        // Exponential backoff
        const delay = Math.min(1000 * Math.pow(2, attempt - 1), 10000);
        await this.sleep(delay);
      }
    }
  }

  isRetryableError(error) {
    // Retry on rate limiting, network errors, and server errors
    return error.status === 429 || 
           error.status >= 500 || 
           error.code === 'NETWORK_ERROR';
  }

  sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}
```

---

**Next Steps:**
- [MCP Tools Guide](proactive-research-mcp.md) - Claude Code integration
- [CLI Reference](proactive-research-cli.md) - Command-line interface
- [Configuration Guide](proactive-research-configuration.md) - Detailed configuration
- [Troubleshooting Guide](proactive-research-troubleshooting.md) - Common issues and solutions