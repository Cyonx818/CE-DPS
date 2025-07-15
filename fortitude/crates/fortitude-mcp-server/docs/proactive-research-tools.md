# Proactive Research MCP Tools

This document describes the MCP tools for proactive research functionality in the Fortitude system. These tools enable Claude Code to control and monitor the proactive research system through the Model Context Protocol.

## Overview

The proactive research MCP tools provide seamless integration between Claude Code and Fortitude's proactive research capabilities. They allow starting/stopping the system, configuring settings, monitoring status, and retrieving task lists and notifications.

## Available Tools

### 1. proactive_start

Starts the proactive research system with optional configuration.

**Input Schema:**
```json
{
  "config": {
    "base_directory": "/path/to/monitor",
    "monitoring_interval_seconds": 300,
    "max_concurrent_tasks": 5
  }
}
```

**Response:**
```json
{
  "is_running": true,
  "status": "started",
  "health_metrics": {
    "file_monitor_status": "healthy",
    "scheduler_status": "active",
    "executor_status": "running",
    "notification_status": "operational",
    "error_count_24h": 0,
    "memory_usage_percent": 0.0,
    "cpu_usage_percent": 0.0
  },
  "processing_time_ms": 150
}
```

### 2. proactive_stop

Gracefully stops the proactive research system.

**Input Schema:**
```json
{
  "force": false,
  "timeout_seconds": 30
}
```

**Response:**
```json
{
  "is_running": false,
  "status": "stopped",
  "health_metrics": {
    "file_monitor_status": "stopped",
    "scheduler_status": "stopped",
    "executor_status": "stopped",
    "notification_status": "stopped",
    "error_count_24h": 0,
    "memory_usage_percent": 0.0,
    "cpu_usage_percent": 0.0
  },
  "processing_time_ms": 75
}
```

### 3. proactive_status

Gets detailed status information about the proactive research system.

**Input Schema:**
```json
{
  "detailed": true,
  "include_metrics": true,
  "recent_minutes": 60
}
```

**Response:**
```json
{
  "is_running": true,
  "uptime_seconds": 3600,
  "active_tasks_count": 3,
  "completed_tasks_count": 15,
  "detected_gaps_count": 8,
  "last_gap_detection": "2024-01-15T10:30:00Z",
  "health_metrics": {
    "file_monitor_status": "healthy",
    "scheduler_status": "active",
    "executor_status": "running",
    "notification_status": "operational",
    "error_count_24h": 0,
    "memory_usage_percent": 12.5,
    "cpu_usage_percent": 8.2
  },
  "recent_activity": [
    {
      "timestamp": "2024-01-15T10:29:45Z",
      "event_type": "TaskCompleted",
      "description": "Research task completed successfully",
      "task_id": "task_123",
      "gap_id": "gap_456"
    }
  ],
  "config_summary": {
    "gap_interval_minutes": 30,
    "max_concurrent_tasks": 5,
    "debounce_seconds": 5,
    "auto_persist_enabled": true,
    "auto_execute_enabled": false,
    "notification_channels": ["console"]
  },
  "processing_time_ms": 25
}
```

### 4. proactive_configure

Updates configuration settings for the proactive research system.

**Input Schema:**
```json
{
  "config": {
    "gap_interval_minutes": 15,
    "max_concurrent_tasks": 3,
    "debounce_seconds": 10,
    "auto_persist": true,
    "auto_execute": true
  }
}
```

**Response:**
```json
{
  "updated_config": {
    "gap_interval_minutes": 15,
    "max_concurrent_tasks": 3,
    "debounce_seconds": 10,
    "auto_persist_enabled": true,
    "auto_execute_enabled": true,
    "notification_channels": ["console"]
  },
  "changes_applied": [
    "gap_interval_minutes = 15",
    "max_concurrent_tasks = 3",
    "debounce_seconds = 10"
  ],
  "restart_required": false,
  "processing_time_ms": 50
}
```

### 5. proactive_list_tasks

Lists active or recent background research tasks with filtering and pagination.

**Input Schema:**
```json
{
  "status": "active",
  "priority": "high",
  "research_type": "implementation",
  "limit": 10,
  "offset": 0
}
```

**Response:**
```json
{
  "tasks": [
    {
      "id": "task_123",
      "status": "active",
      "priority": "high",
      "research_type": "implementation",
      "gap_type": "knowledge_gap",
      "description": "Research async/await patterns in Rust",
      "created_at": "2024-01-15T10:00:00Z",
      "updated_at": "2024-01-15T10:29:45Z",
      "progress_percent": 75.0
    }
  ],
  "total_count": 15,
  "pagination": {
    "offset": 0,
    "limit": 10,
    "total_pages": 2,
    "has_more": true
  },
  "task_statistics": {
    "by_status": {
      "active": 3,
      "completed": 12,
      "failed": 0
    },
    "by_priority": {
      "high": 2,
      "medium": 8,
      "low": 5
    },
    "by_research_type": {
      "implementation": 6,
      "learning": 5,
      "troubleshooting": 4
    },
    "avg_completion_time_seconds": 180.0,
    "success_rate_percent": 95.2
  },
  "processing_time_ms": 30
}
```

### 6. proactive_get_notifications

Retrieves recent proactive research notifications.

**Input Schema:**
```json
{
  "unread_only": true,
  "limit": 20,
  "since_minutes": 60
}
```

**Response:**
```json
{
  "notifications": [
    {
      "id": "notif_789",
      "notification_type": "gap_detected",
      "level": "info",
      "title": "Knowledge Gap Detected",
      "message": "New gap detected in async programming patterns",
      "created_at": "2024-01-15T10:25:00Z",
      "read": false,
      "task_id": "task_123",
      "gap_id": "gap_456"
    }
  ],
  "total_count": 5,
  "unread_count": 3,
  "notification_statistics": {
    "by_type": {
      "gap_detected": 2,
      "task_completed": 2,
      "error": 1
    },
    "by_level": {
      "info": 4,
      "warning": 1,
      "error": 0
    },
    "read_status": {
      "read": 2,
      "unread": 3
    },
    "created_last_24h": 5,
    "avg_time_to_read_seconds": 300.0
  },
  "processing_time_ms": 20
}
```

## Usage Examples

### Starting Proactive Research

```javascript
// Basic start
await mcp.call_tool("proactive_start", {});

// Start with custom configuration
await mcp.call_tool("proactive_start", {
  config: {
    base_directory: "/workspace/my-project",
    monitoring_interval_seconds: 180,
    max_concurrent_tasks: 3
  }
});
```

### Monitoring Status

```javascript
// Get basic status
const status = await mcp.call_tool("proactive_status", {});

// Get detailed status with metrics
const detailed_status = await mcp.call_tool("proactive_status", {
  detailed: true,
  include_metrics: true,
  recent_minutes: 30
});
```

### Managing Tasks

```javascript
// List active high-priority tasks
const tasks = await mcp.call_tool("proactive_list_tasks", {
  status: "active",
  priority: "high",
  limit: 5
});

// Get recent notifications
const notifications = await mcp.call_tool("proactive_get_notifications", {
  unread_only: true,
  limit: 10
});
```

### Configuration Management

```javascript
// Update configuration
await mcp.call_tool("proactive_configure", {
  config: {
    gap_interval_minutes: 20,
    max_concurrent_tasks: 4,
    auto_execute: true
  }
});
```

### Graceful Shutdown

```javascript
// Graceful stop with timeout
await mcp.call_tool("proactive_stop", {
  force: false,
  timeout_seconds: 60
});

// Force stop
await mcp.call_tool("proactive_stop", {
  force: true
});
```

## Error Handling

All tools return standardized error responses following MCP protocol:

- **Invalid Request**: Tool name not found or malformed arguments
- **Bad Request**: Validation errors for input parameters
- **Conflict**: System state conflicts (e.g., starting when already running)
- **Internal Error**: Server-side errors during processing

Example error response:
```json
{
  "error": {
    "code": -32602,
    "message": "Invalid params: gap_interval_minutes must be between 5 and 1440",
    "data": null
  }
}
```

## Input Validation

All tools validate input parameters:

- **String lengths**: Enforced minimum and maximum lengths
- **Numeric ranges**: Values must be within specified bounds
- **Enum values**: Only predefined values are accepted
- **Required fields**: Missing required parameters are rejected

## Security Features

- **Input sanitization**: All user inputs are sanitized to prevent injection
- **Permission checking**: Tools require appropriate MCP permissions
- **Rate limiting**: Built-in rate limiting prevents abuse
- **Audit logging**: All tool calls are logged for security monitoring

## Integration Notes

### Claude Code Integration

These tools are automatically available in Claude Code when connected to a Fortitude MCP server. They enable seamless control of proactive research from within the IDE.

### API Server Integration

The MCP tools complement the HTTP API endpoints, providing a more structured protocol interface for AI agents and IDE integrations.

### Real-time Updates

The tools provide real-time status information and can be used to implement monitoring dashboards or notification systems.

## Testing

Comprehensive test coverage includes:

- **Unit tests**: Individual tool functionality
- **Integration tests**: End-to-end tool workflows
- **Anchor tests**: Critical functionality protection
- **Performance tests**: Load and stress testing
- **Security tests**: Input validation and injection prevention

## Future Enhancements

Planned improvements include:

- **WebSocket support**: Real-time notifications through WebSocket
- **Advanced filtering**: More sophisticated task and notification filtering
- **Batch operations**: Bulk task management operations
- **Metrics export**: Prometheus/OpenTelemetry metrics integration
- **Custom notifications**: User-defined notification rules