# Multi-LLM API Reference

<meta>
  <title>Multi-LLM API Reference</title>
  <type>api_reference</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-12</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Complete API reference for Sprint 009 multi-LLM, quality control, learning, and monitoring endpoints
- **Base URL**: `http://localhost:8080/api` (development), `https://api.fortitude.company.com/api` (production)
- **Authentication**: Bearer token required for all endpoints except health checks
- **Response Format**: JSON with consistent error handling and metadata
- **Rate Limits**: 1000 requests/minute per user, configurable per endpoint
- **Related docs**: [Multi-LLM Setup](../user-guides/multi-llm-setup.md), [Enterprise Deployment](../deployment/enterprise-deployment.md)

## <context>API Overview</context>

This reference documents all Sprint 009 API endpoints including multi-LLM provider management, quality control, real-time learning, and monitoring systems. All endpoints follow REST conventions with JSON request/response formats.

## <authentication>Authentication</authentication>

### **Bearer Token Authentication**

```http
Authorization: Bearer <your-jwt-token>
```

### **Authentication Endpoints**

#### **POST /api/auth/login**
Authenticate user and obtain JWT token.

```http
POST /api/auth/login
Content-Type: application/json

{
  "username": "user@company.com",
  "password": "secure-password"
}
```

**Response**:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "refresh-token-here",
  "user": {
    "id": "user-123",
    "username": "user@company.com",
    "role": "researcher"
  }
}
```

#### **POST /api/auth/refresh**
Refresh expired JWT token.

```http
POST /api/auth/refresh
Content-Type: application/json

{
  "refresh_token": "refresh-token-here"
}
```

## <research-endpoints>Research Endpoints</research-endpoints>

### **POST /api/research**
Execute research query with multi-LLM support.

```http
POST /api/research
Authorization: Bearer <token>
Content-Type: application/json

{
  "query": "Explain quantum computing applications in cryptography",
  "provider_preference": "auto",
  "quality_threshold": 0.8,
  "max_cost_usd": 0.10,
  "enable_cross_validation": true,
  "context": {
    "domain": "technology",
    "audience": "expert",
    "urgency": "normal"
  }
}
```

**Parameters**:
- `query` (string, required): Research query text
- `provider_preference` (string, optional): `"auto"`, `"openai"`, `"claude"`, `"gemini"`, `"cost_optimized"`, `"quality_optimized"`
- `quality_threshold` (number, optional): Minimum quality score (0.0-1.0), default: 0.7
- `max_cost_usd` (number, optional): Maximum cost limit in USD
- `enable_cross_validation` (boolean, optional): Enable cross-provider validation, default: true
- `context` (object, optional): Additional context for optimization

**Response**:
```json
{
  "content": "Quantum computing represents a paradigm shift in computational capabilities...",
  "provider_used": "claude",
  "quality_score": {
    "relevance": 0.92,
    "accuracy": 0.89,
    "completeness": 0.85,
    "clarity": 0.91,
    "credibility": 0.88,
    "timeliness": 0.75,
    "specificity": 0.87,
    "composite": 0.87,
    "confidence": 0.91
  },
  "cost_usd": 0.045,
  "response_time_ms": 2150,
  "tokens_used": {
    "input": 45,
    "output": 1200
  },
  "cross_validation": {
    "performed": true,
    "consistency_score": 0.89,
    "alternative_providers": ["openai", "gemini"]
  },
  "metadata": {
    "request_id": "req-123456",
    "timestamp": "2025-07-12T10:30:00Z",
    "fallback_used": false,
    "cache_hit": false
  }
}
```

#### **POST /api/research/batch**
Execute multiple research queries in batch.

```http
POST /api/research/batch
Authorization: Bearer <token>
Content-Type: application/json

{
  "queries": [
    {
      "id": "query-1",
      "query": "What is machine learning?",
      "provider_preference": "auto"
    },
    {
      "id": "query-2", 
      "query": "Explain neural networks",
      "provider_preference": "claude"
    }
  ],
  "batch_options": {
    "parallel_execution": true,
    "max_concurrent": 5,
    "quality_threshold": 0.8
  }
}
```

**Response**:
```json
{
  "results": [
    {
      "id": "query-1",
      "status": "success",
      "content": "Machine learning is a subset of artificial intelligence...",
      "provider_used": "openai",
      "quality_score": { /* quality scores */ },
      "cost_usd": 0.032
    },
    {
      "id": "query-2",
      "status": "success", 
      "content": "Neural networks are computing systems...",
      "provider_used": "claude",
      "quality_score": { /* quality scores */ },
      "cost_usd": 0.028
    }
  ],
  "summary": {
    "total_queries": 2,
    "successful": 2,
    "failed": 0,
    "total_cost_usd": 0.060,
    "average_quality": 0.85,
    "total_time_ms": 3200
  }
}
```

## <provider-endpoints>Provider Management Endpoints</provider-endpoints>

### **GET /api/providers/status**
Get status of all configured providers.

```http
GET /api/providers/status
Authorization: Bearer <token>
```

**Response**:
```json
{
  "providers": {
    "openai": {
      "status": "healthy",
      "response_time_ms": 1200,
      "success_rate": 0.98,
      "rate_limit_remaining": 85,
      "cost_per_1k_tokens": 0.002,
      "last_check": "2025-07-12T10:29:00Z"
    },
    "claude": {
      "status": "healthy",
      "response_time_ms": 1100,
      "success_rate": 0.97,
      "rate_limit_remaining": 92,
      "cost_per_1k_tokens": 0.0015,
      "last_check": "2025-07-12T10:29:00Z"
    },
    "gemini": {
      "status": "degraded",
      "response_time_ms": 2100,
      "success_rate": 0.89,
      "rate_limit_remaining": 78,
      "cost_per_1k_tokens": 0.0,
      "last_check": "2025-07-12T10:29:00Z",
      "issues": ["elevated_latency"]
    }
  },
  "fallback_active": false,
  "overall_health": "healthy"
}
```

### **POST /api/providers/{provider}/health-check**
Perform manual health check for specific provider.

```http
POST /api/providers/openai/health-check
Authorization: Bearer <token>
```

**Response**:
```json
{
  "provider": "openai",
  "status": "healthy",
  "response_time_ms": 1150,
  "test_query_successful": true,
  "api_key_valid": true,
  "rate_limits": {
    "requests_remaining": 85,
    "tokens_remaining": 45000,
    "reset_time": "2025-07-12T11:00:00Z"
  },
  "timestamp": "2025-07-12T10:30:00Z"
}
```

### **GET /api/providers/performance**
Get detailed performance metrics for all providers.

```http
GET /api/providers/performance?time_range=24h
Authorization: Bearer <token>
```

**Response**:
```json
{
  "time_range": "24h",
  "providers": {
    "openai": {
      "total_requests": 1250,
      "successful_requests": 1225,
      "failed_requests": 25,
      "success_rate": 0.98,
      "average_response_time_ms": 1200,
      "p95_response_time_ms": 2100,
      "p99_response_time_ms": 3500,
      "total_cost_usd": 15.75,
      "average_quality_score": 0.87,
      "rate_limit_hits": 5
    },
    "claude": {
      "total_requests": 980,
      "successful_requests": 950,
      "failed_requests": 30,
      "success_rate": 0.97,
      "average_response_time_ms": 1100,
      "p95_response_time_ms": 1900,
      "p99_response_time_ms": 2800,
      "total_cost_usd": 8.90,
      "average_quality_score": 0.89,
      "rate_limit_hits": 2
    }
  }
}
```

### **PUT /api/providers/{provider}/config**
Update provider configuration.

```http
PUT /api/providers/openai/config
Authorization: Bearer <token>
Content-Type: application/json

{
  "rate_limit": {
    "requests_per_minute": 120,
    "tokens_per_minute": 80000
  },
  "timeout_seconds": 45,
  "max_concurrent": 10,
  "cost_limit_daily": 50.0
}
```

**Response**:
```json
{
  "provider": "openai",
  "config_updated": true,
  "changes": [
    "rate_limit.requests_per_minute: 100 -> 120",
    "timeout_seconds: 30 -> 45"
  ],
  "effective_time": "2025-07-12T10:31:00Z"
}
```

## <quality-endpoints>Quality Control Endpoints</quality-endpoints>

### **POST /api/quality/evaluate**
Evaluate quality of research content.

```http
POST /api/quality/evaluate
Authorization: Bearer <token>
Content-Type: application/json

{
  "query": "What is artificial intelligence?",
  "response": "Artificial intelligence (AI) refers to the simulation of human intelligence...",
  "weights": {
    "relevance": 0.25,
    "accuracy": 0.25,
    "completeness": 0.20,
    "clarity": 0.10,
    "credibility": 0.15
  },
  "context": {
    "domain": "technology",
    "audience": "general"
  }
}
```

**Response**:
```json
{
  "quality_score": {
    "relevance": 0.92,
    "accuracy": 0.89,
    "completeness": 0.85,
    "clarity": 0.91,
    "credibility": 0.88,
    "timeliness": 0.75,
    "specificity": 0.87,
    "composite": 0.87,
    "confidence": 0.91
  },
  "evaluation_time_ms": 85,
  "dimension_analysis": {
    "strengths": ["high_relevance", "clear_explanation"],
    "weaknesses": ["could_be_more_specific"],
    "recommendations": ["add_concrete_examples", "include_recent_developments"]
  },
  "metadata": {
    "evaluator": "comprehensive_scorer_v1.2",
    "timestamp": "2025-07-12T10:32:00Z"
  }
}
```

### **POST /api/quality/cross-validate**
Perform cross-validation of research response.

```http
POST /api/quality/cross-validate
Authorization: Bearer <token>
Content-Type: application/json

{
  "query": "Explain climate change impacts",
  "primary_response": "Climate change is causing...",
  "primary_provider": "openai",
  "validation_providers": ["claude", "gemini"],
  "consensus_threshold": 0.8
}
```

**Response**:
```json
{
  "validation_result": {
    "is_valid": true,
    "consistency_score": 0.89,
    "consensus_achieved": true
  },
  "provider_responses": {
    "claude": {
      "response": "Climate change impacts include...",
      "quality_score": 0.91,
      "similarity_to_primary": 0.87
    },
    "gemini": {
      "response": "The effects of climate change are...",
      "quality_score": 0.85,
      "similarity_to_primary": 0.83
    }
  },
  "bias_analysis": {
    "detected_biases": [],
    "confidence": 0.92
  },
  "recommendations": {
    "use_primary": true,
    "reason": "high_consistency_across_providers"
  }
}
```

### **GET /api/quality/metrics**
Get quality metrics and trends.

```http
GET /api/quality/metrics?time_range=7d&provider=all
Authorization: Bearer <token>
```

**Response**:
```json
{
  "time_range": "7d",
  "overall_metrics": {
    "average_quality_score": 0.85,
    "total_evaluations": 5420,
    "quality_trend": "improving",
    "improvement_rate": 0.03
  },
  "provider_metrics": {
    "openai": {
      "average_quality": 0.87,
      "evaluations_count": 2150,
      "trend": "stable"
    },
    "claude": {
      "average_quality": 0.89,
      "evaluations_count": 1890,
      "trend": "improving"
    },
    "gemini": {
      "average_quality": 0.82,
      "evaluations_count": 1380,
      "trend": "stable"
    }
  },
  "dimension_breakdown": {
    "relevance": 0.88,
    "accuracy": 0.86,
    "completeness": 0.83,
    "clarity": 0.87,
    "credibility": 0.84
  }
}
```

## <learning-endpoints>Learning System Endpoints</learning-endpoints>

### **POST /api/learning/feedback**
Submit user feedback for learning system.

```http
POST /api/learning/feedback
Authorization: Bearer <token>
Content-Type: application/json

{
  "content_id": "research-response-123",
  "feedback_type": "quality_rating",
  "score": 0.85,
  "text_feedback": "Good response but could include more recent examples",
  "context": {
    "query_type": "factual",
    "provider_used": "claude",
    "user_expertise": "intermediate"
  }
}
```

**Response**:
```json
{
  "feedback_id": "feedback-456",
  "status": "accepted",
  "message": "Feedback will be processed for system improvement",
  "processing_eta": "5m",
  "contribution_score": 0.92
}
```

### **GET /api/learning/insights**
Get learning insights and patterns.

```http
GET /api/learning/insights?time_range=30d&user_id=user-123
Authorization: Bearer <token>
```

**Response**:
```json
{
  "user_patterns": {
    "preferred_providers": {
      "claude": 0.45,
      "openai": 0.35,
      "gemini": 0.20
    },
    "query_types": {
      "technical": 0.60,
      "general": 0.25,
      "creative": 0.15
    },
    "quality_preferences": {
      "accuracy_weight": 0.35,
      "completeness_weight": 0.30,
      "clarity_weight": 0.25,
      "speed_weight": 0.10
    }
  },
  "system_improvements": {
    "quality_improvement": 0.05,
    "response_time_improvement": 0.12,
    "cost_optimization": 0.18
  },
  "recommendations": [
    {
      "type": "provider_optimization",
      "description": "Consider using Claude more often for technical queries",
      "confidence": 0.87
    },
    {
      "type": "quality_tuning", 
      "description": "Adjust accuracy weighting for better satisfaction",
      "confidence": 0.79
    }
  ]
}
```

### **GET /api/learning/patterns/{user_id}**
Get usage patterns for specific user.

```http
GET /api/learning/patterns/user-123?time_range=60d
Authorization: Bearer <token>
```

**Response**:
```json
{
  "user_id": "user-123",
  "patterns": [
    {
      "pattern_type": "query_preference",
      "data": "prefers technical queries with detailed explanations",
      "frequency": 45,
      "confidence": 0.89,
      "last_seen": "2025-07-12T09:15:00Z"
    },
    {
      "pattern_type": "provider_preference", 
      "data": "shows preference for Claude over OpenAI for analytical tasks",
      "frequency": 23,
      "confidence": 0.76,
      "last_seen": "2025-07-12T08:30:00Z"
    }
  ],
  "personalization_opportunities": [
    {
      "type": "provider_selection",
      "description": "Auto-select Claude for analytical queries",
      "potential_improvement": 0.15
    },
    {
      "type": "quality_weights",
      "description": "Increase accuracy weight for this user",
      "potential_improvement": 0.08
    }
  ]
}
```

### **POST /api/learning/optimize**
Trigger learning-based optimization.

```http
POST /api/learning/optimize
Authorization: Bearer <token>
Content-Type: application/json

{
  "optimization_scope": "user",
  "user_id": "user-123",
  "auto_apply": false
}
```

**Response**:
```json
{
  "optimization_id": "opt-789",
  "status": "completed",
  "optimizations": [
    {
      "type": "provider_selection",
      "current_strategy": "round_robin",
      "recommended_strategy": "user_preference_based",
      "confidence": 0.91,
      "expected_improvement": 0.12
    },
    {
      "type": "quality_weights",
      "current_weights": { "accuracy": 0.25, "clarity": 0.25 },
      "recommended_weights": { "accuracy": 0.35, "clarity": 0.20 },
      "confidence": 0.83,
      "expected_improvement": 0.07
    }
  ],
  "auto_applied": false,
  "manual_approval_required": true
}
```

## <monitoring-endpoints>Monitoring Endpoints</monitoring-endpoints>

### **GET /api/monitoring/health**
Get comprehensive system health status.

```http
GET /api/monitoring/health
Authorization: Bearer <token>
```

**Response**:
```json
{
  "overall_status": "healthy",
  "components": {
    "api_server": {
      "status": "healthy",
      "response_time_ms": 45,
      "memory_usage_percent": 68,
      "cpu_usage_percent": 35
    },
    "providers": {
      "status": "healthy",
      "active_providers": 3,
      "failed_providers": 0,
      "fallback_active": false
    },
    "quality_system": {
      "status": "healthy",
      "evaluation_queue_size": 5,
      "average_evaluation_time_ms": 78
    },
    "learning_system": {
      "status": "healthy",
      "feedback_queue_size": 12,
      "pattern_processing_active": true
    },
    "database": {
      "status": "healthy",
      "connection_pool_active": 18,
      "query_performance_ms": 25
    },
    "cache": {
      "status": "healthy",
      "hit_ratio": 0.87,
      "memory_usage_percent": 72
    }
  },
  "last_updated": "2025-07-12T10:35:00Z"
}
```

### **GET /api/monitoring/metrics**
Get system performance metrics.

```http
GET /api/monitoring/metrics?time_range=1h&component=all
Authorization: Bearer <token>
```

**Response**:
```json
{
  "time_range": "1h",
  "api_metrics": {
    "total_requests": 1450,
    "successful_requests": 1425,
    "error_rate": 0.017,
    "average_response_time_ms": 185,
    "p95_response_time_ms": 320,
    "requests_per_minute": 24.2
  },
  "provider_metrics": {
    "total_provider_requests": 1425,
    "provider_distribution": {
      "openai": 0.42,
      "claude": 0.38,
      "gemini": 0.20
    },
    "average_provider_response_ms": 1350,
    "fallback_rate": 0.03
  },
  "quality_metrics": {
    "evaluations_performed": 1425,
    "average_quality_score": 0.86,
    "evaluation_time_ms": 82,
    "cross_validations": 285
  },
  "resource_metrics": {
    "cpu_usage_percent": 42,
    "memory_usage_percent": 68,
    "disk_usage_percent": 34,
    "network_io_mbps": 12.5
  }
}
```

### **GET /api/monitoring/alerts**
Get active alerts and alert history.

```http
GET /api/monitoring/alerts?status=active&severity=critical,warning
Authorization: Bearer <token>
```

**Response**:
```json
{
  "active_alerts": [
    {
      "id": "alert-123",
      "severity": "warning",
      "title": "Elevated Response Time",
      "message": "API response time above 200ms threshold",
      "component": "api_server",
      "started_at": "2025-07-12T10:20:00Z",
      "duration_minutes": 15,
      "threshold_value": 200,
      "current_value": 235
    }
  ],
  "alert_history": [
    {
      "id": "alert-122",
      "severity": "critical",
      "title": "Provider Unavailable",
      "message": "OpenAI provider failed health check",
      "component": "providers",
      "started_at": "2025-07-12T08:45:00Z",
      "resolved_at": "2025-07-12T08:52:00Z",
      "duration_minutes": 7
    }
  ],
  "summary": {
    "total_active": 1,
    "critical": 0,
    "warning": 1,
    "info": 0
  }
}
```

### **POST /api/monitoring/alerts/test**
Test alert system configuration.

```http
POST /api/monitoring/alerts/test
Authorization: Bearer <token>
Content-Type: application/json

{
  "alert_type": "test",
  "severity": "warning",
  "channels": ["email", "webhook"]
}
```

**Response**:
```json
{
  "test_id": "test-456",
  "status": "completed",
  "results": {
    "email": {
      "status": "success",
      "delivered_at": "2025-07-12T10:36:00Z"
    },
    "webhook": {
      "status": "success",
      "response_code": 200,
      "delivered_at": "2025-07-12T10:36:01Z"
    }
  }
}
```

## <cost-endpoints>Cost Management Endpoints</cost-endpoints>

### **GET /api/costs/usage**
Get cost and usage analytics.

```http
GET /api/costs/usage?time_range=30d&breakdown=provider,user
Authorization: Bearer <token>
```

**Response**:
```json
{
  "time_range": "30d",
  "total_cost_usd": 245.67,
  "cost_breakdown": {
    "by_provider": {
      "openai": 142.35,
      "claude": 78.90,
      "gemini": 0.00
    },
    "by_user": {
      "user-123": 89.45,
      "user-456": 67.23,
      "user-789": 88.99
    }
  },
  "usage_metrics": {
    "total_requests": 12450,
    "total_tokens": 2450000,
    "average_cost_per_request": 0.0197,
    "cost_trend": "decreasing"
  },
  "budget_status": {
    "monthly_budget": 500.00,
    "used_percent": 49.1,
    "projected_monthly": 367.51,
    "status": "on_track"
  }
}
```

### **PUT /api/costs/budget**
Update cost budgets and limits.

```http
PUT /api/costs/budget
Authorization: Bearer <token>
Content-Type: application/json

{
  "monthly_budget_usd": 600.00,
  "daily_budget_usd": 25.00,
  "user_limits": {
    "user-123": 150.00,
    "user-456": 100.00
  },
  "provider_limits": {
    "openai": 350.00,
    "claude": 200.00
  },
  "alert_thresholds": {
    "daily_80_percent": true,
    "monthly_90_percent": true
  }
}
```

**Response**:
```json
{
  "budget_updated": true,
  "changes": [
    "monthly_budget: $500.00 -> $600.00",
    "daily_budget: $20.00 -> $25.00"
  ],
  "effective_date": "2025-07-12T10:37:00Z",
  "next_budget_period": "2025-08-01T00:00:00Z"
}
```

## <error-handling>Error Handling</error-handling>

### **Standard Error Response Format**

```json
{
  "error": {
    "code": "PROVIDER_UNAVAILABLE",
    "message": "All providers are currently unavailable",
    "details": {
      "providers_checked": ["openai", "claude", "gemini"],
      "last_successful_request": "2025-07-12T10:20:00Z"
    },
    "request_id": "req-789",
    "timestamp": "2025-07-12T10:37:00Z"
  }
}
```

### **Common Error Codes**

| Error Code | HTTP Status | Description |
|------------|-------------|-------------|
| `AUTHENTICATION_FAILED` | 401 | Invalid or expired token |
| `INSUFFICIENT_PERMISSIONS` | 403 | User lacks required permissions |
| `INVALID_REQUEST` | 400 | Malformed request body or parameters |
| `RESOURCE_NOT_FOUND` | 404 | Requested resource does not exist |
| `RATE_LIMIT_EXCEEDED` | 429 | API rate limit exceeded |
| `PROVIDER_UNAVAILABLE` | 503 | LLM provider temporarily unavailable |
| `QUALITY_THRESHOLD_NOT_MET` | 422 | Response quality below threshold |
| `COST_LIMIT_EXCEEDED` | 402 | Request would exceed cost limits |
| `INTERNAL_SERVER_ERROR` | 500 | Unexpected server error |

### **Rate Limiting Headers**

```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 985
X-RateLimit-Reset: 1625097600
X-RateLimit-Retry-After: 60
```

## <pagination>Pagination</pagination>

### **Pagination Parameters**

For endpoints returning lists of items:

```http
GET /api/learning/feedback?page=2&per_page=50&sort=timestamp&order=desc
```

**Parameters**:
- `page` (integer, optional): Page number (1-based), default: 1
- `per_page` (integer, optional): Items per page (max 100), default: 20
- `sort` (string, optional): Sort field
- `order` (string, optional): Sort order (`asc` or `desc`), default: `desc`

**Response with Pagination**:
```json
{
  "data": [ /* array of items */ ],
  "pagination": {
    "page": 2,
    "per_page": 50,
    "total_items": 1250,
    "total_pages": 25,
    "has_next": true,
    "has_prev": true,
    "next_page": 3,
    "prev_page": 1
  }
}
```

## <filtering>Filtering and Search</filtering>

### **Common Filter Parameters**

```http
GET /api/research/history?provider=claude&quality_min=0.8&date_from=2025-07-01&date_to=2025-07-12
```

**Parameters**:
- `provider` (string): Filter by provider name
- `quality_min` (number): Minimum quality score
- `quality_max` (number): Maximum quality score
- `date_from` (string): Start date (ISO 8601)
- `date_to` (string): End date (ISO 8601)
- `user_id` (string): Filter by user ID
- `cost_min` (number): Minimum cost in USD
- `cost_max` (number): Maximum cost in USD

### **Search Parameters**

```http
GET /api/research/history?search=quantum+computing&search_fields=query,content
```

**Parameters**:
- `search` (string): Search query
- `search_fields` (string): Comma-separated fields to search in

## <webhooks>Webhooks</webhooks>

### **Webhook Configuration**

Configure webhooks for real-time notifications:

```http
POST /api/webhooks
Authorization: Bearer <token>
Content-Type: application/json

{
  "url": "https://your-app.com/webhook/fortitude",
  "events": [
    "research.completed",
    "quality.evaluation.completed", 
    "provider.status.changed",
    "alert.triggered"
  ],
  "secret": "webhook-secret-key",
  "active": true
}
```

### **Webhook Payload Example**

```json
{
  "event": "research.completed",
  "timestamp": "2025-07-12T10:38:00Z",
  "data": {
    "request_id": "req-123456",
    "user_id": "user-123",
    "query": "What is machine learning?",
    "provider_used": "claude",
    "quality_score": 0.87,
    "cost_usd": 0.032,
    "response_time_ms": 1200
  },
  "signature": "sha256=abc123..."
}
```

## <sdks>Client SDKs and Examples</sdks>

### **JavaScript/TypeScript SDK**

```javascript
import { FortitudeClient } from '@company/fortitude-client';

const client = new FortitudeClient({
  baseUrl: 'https://api.fortitude.company.com',
  apiKey: 'your-api-key'
});

// Execute research query
const result = await client.research({
  query: 'Explain quantum computing',
  providerPreference: 'auto',
  qualityThreshold: 0.8
});

console.log(`Response: ${result.content}`);
console.log(`Quality: ${result.qualityScore.composite}`);
console.log(`Cost: $${result.costUsd}`);
```

### **Python SDK**

```python
from fortitude_client import FortitudeClient

client = FortitudeClient(
    base_url='https://api.fortitude.company.com',
    api_key='your-api-key'
)

# Execute research query
result = client.research(
    query='Explain quantum computing',
    provider_preference='auto',
    quality_threshold=0.8
)

print(f"Response: {result.content}")
print(f"Quality: {result.quality_score.composite}")
print(f"Cost: ${result.cost_usd}")
```

### **cURL Examples**

```bash
# Research query
curl -X POST https://api.fortitude.company.com/api/research \
  -H "Authorization: Bearer $API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "Explain quantum computing",
    "provider_preference": "auto",
    "quality_threshold": 0.8
  }'

# Get provider status
curl -X GET https://api.fortitude.company.com/api/providers/status \
  -H "Authorization: Bearer $API_TOKEN"

# Submit feedback
curl -X POST https://api.fortitude.company.com/api/learning/feedback \
  -H "Authorization: Bearer $API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "content_id": "research-response-123",
    "feedback_type": "quality_rating",
    "score": 0.85
  }'
```

## <references>See Also</references>

- [Multi-LLM Setup Guide](../user-guides/multi-llm-setup.md) - Configuration and setup
- [Multi-LLM Architecture](../architecture/multi-llm-architecture.md) - System architecture
- [Quality Control Design](../architecture/quality-control-design.md) - Quality system details
- [Learning System Design](../architecture/learning-system-design.md) - Learning system architecture
- [Enterprise Deployment](../deployment/enterprise-deployment.md) - Production deployment
- [Troubleshooting Guide](../troubleshooting/sprint-009-issues.md) - Issue resolution