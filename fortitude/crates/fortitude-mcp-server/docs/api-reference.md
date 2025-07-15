# Fortitude MCP Server API Reference

<meta>
  <title>Fortitude MCP Server API Reference</title>
  <type>api_reference</type>
  <audience>developer</audience>
  <complexity>high</complexity>
  <updated>2025-07-09</updated>
</meta>

## <summary priority="high">API Overview</summary>

**Purpose**: Complete technical reference for all MCP tools, resources, and protocol endpoints
**Scope**: Model Context Protocol (MCP) server implementation with JWT authentication
**Version**: 0.2.0 (Protocol Version: 2024-11-05)

## <protocol-info>Protocol Information</protocol-info>

### <transport>Transport Layer</transport>

<implementation>
**Protocol**: Model Context Protocol (MCP) v2024-11-05
**Transport**: stdio (standard input/output) 
**Message Format**: JSON-RPC 2.0
**Authentication**: JWT Bearer tokens

```json
{
  "jsonrpc": "2.0",
  "id": "request-id",
  "method": "tools/call",
  "params": {
    "name": "research_query",
    "arguments": {
      "query": "How to implement async functions in Rust?"
    }
  }
}
```
</implementation>

### <capabilities>Server Capabilities</capabilities>

<implementation>
```json
{
  "capabilities": {
    "tools": {
      "enabled": true,
      "listChanged": false
    },
    "resources": {
      "enabled": true,
      "listChanged": false,
      "subscribe": false
    },
    "prompts": {
      "enabled": false
    },
    "logging": {
      "enabled": true
    }
  }
}
```
</implementation>

## <tools>MCP Tools</tools>

### <tool>research_query</tool>

Execute research queries using the Fortitude pipeline with context detection.

<implementation>
**Method**: `tools/call`
**Name**: `research_query`

**Request Schema**:
```json
{
  "type": "object",
  "properties": {
    "query": {
      "type": "string",
      "description": "Research query to execute",
      "minLength": 1,
      "maxLength": 1000
    },
    "query_type": {
      "type": "string",
      "description": "Type of research query",
      "enum": ["research", "troubleshooting", "learning", "implementation", "decision", "validation"],
      "default": "learning"
    },
    "audience": {
      "type": "string",
      "description": "Target audience context",
      "maxLength": 100,
      "examples": ["beginner", "intermediate", "advanced"]
    },
    "domain": {
      "type": "string",
      "description": "Technical domain context",
      "maxLength": 100,
      "examples": ["rust", "web", "mobile", "systems"]
    }
  },
  "required": ["query"]
}
```

**Response Schema**:
```json
{
  "type": "object",
  "properties": {
    "result": {
      "type": "string",
      "description": "Research result with comprehensive answer"
    },
    "metadata": {
      "type": "object",
      "properties": {
        "research_type": {
          "type": "string",
          "description": "Classified research type"
        },
        "confidence": {
          "type": "number",
          "minimum": 0.0,
          "maximum": 1.0,
          "description": "Classification confidence score"
        },
        "processing_time_ms": {
          "type": "integer",
          "description": "Processing time in milliseconds"
        },
        "context_detection_used": {
          "type": "boolean",
          "description": "Whether context detection was applied"
        },
        "cache_key": {
          "type": "string",
          "description": "Cache key for the result"
        }
      }
    }
  }
}
```

**Example Request**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-1",
  "method": "tools/call",
  "params": {
    "name": "research_query",
    "arguments": {
      "query": "How to implement async functions in Rust?",
      "query_type": "implementation",
      "audience": "intermediate",
      "domain": "rust"
    }
  }
}
```

**Example Response**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-1",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"result\":\"Async functions in Rust are implemented using the `async` keyword and the `await` operator. Here's a comprehensive guide:\\n\\n## Basic Async Function\\n\\n```rust\\nasync fn fetch_data() -> Result<String, reqwest::Error> {\\n    let response = reqwest::get(\\\"https://api.example.com/data\\\")\\n        .await?\\n        .text()\\n        .await?;\\n    Ok(response)\\n}\\n```\\n\\n## Key Concepts\\n\\n1. **Async Runtime**: Use `tokio` or `async-std`\\n2. **Futures**: Lazy evaluation until awaited\\n3. **Error Handling**: Use `Result` types with `?` operator\\n4. **Concurrency**: Use `tokio::spawn` for parallel execution\\n\\n## Best Practices\\n\\n- Use `#[tokio::main]` for main function\\n- Handle errors properly with `Result` types\\n- Use `Arc<Mutex<T>>` for shared state\\n- Prefer `tokio::select!` for multiple futures\",\"metadata\":{\"research_type\":\"implementation\",\"confidence\":0.92,\"processing_time_ms\":245,\"context_detection_used\":true,\"cache_key\":\"impl_async_rust_intermediate_1a2b3c\"}}"
      }
    ],
    "isError": false
  }
}
```

**Permissions Required**: `fortitude:research:read`
</implementation>

### <tool>classify_query</tool>

Classify research queries using the Fortitude classification engine.

<implementation>
**Method**: `tools/call`
**Name**: `classify_query`

**Request Schema**:
```json
{
  "type": "object",
  "properties": {
    "query": {
      "type": "string",
      "description": "Query to classify",
      "minLength": 1,
      "maxLength": 1000
    }
  },
  "required": ["query"]
}
```

**Response Schema**:
```json
{
  "type": "object",
  "properties": {
    "research_type": {
      "type": "string",
      "enum": ["research", "troubleshooting", "learning", "implementation", "decision", "validation"],
      "description": "Primary classified research type"
    },
    "confidence": {
      "type": "number",
      "minimum": 0.0,
      "maximum": 1.0,
      "description": "Classification confidence score"
    },
    "matched_keywords": {
      "type": "array",
      "items": {
        "type": "string"
      },
      "description": "Keywords that influenced classification"
    },
    "candidates": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "research_type": {
            "type": "string",
            "description": "Candidate research type"
          },
          "confidence": {
            "type": "number",
            "description": "Confidence score for this candidate"
          },
          "matched_keywords": {
            "type": "array",
            "items": {
              "type": "string"
            },
            "description": "Keywords that matched this candidate"
          }
        }
      },
      "description": "All classification candidates with scores"
    }
  }
}
```

**Example Request**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-2",
  "method": "tools/call",
  "params": {
    "name": "classify_query",
    "arguments": {
      "query": "My application is crashing with a segfault"
    }
  }
}
```

**Example Response**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-2",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"research_type\":\"troubleshooting\",\"confidence\":0.95,\"matched_keywords\":[\"crashing\",\"segfault\",\"application\"],\"candidates\":[{\"research_type\":\"troubleshooting\",\"confidence\":0.95,\"matched_keywords\":[\"crashing\",\"segfault\"]},{\"research_type\":\"implementation\",\"confidence\":0.15,\"matched_keywords\":[\"application\"]}]}"
      }
    ],
    "isError": false
  }
}
```

**Permissions Required**: `fortitude:research:read`
</implementation>

### <tool>detect_context</tool>

Detect audience, domain, and urgency context from queries.

<implementation>
**Method**: `tools/call`
**Name**: `detect_context`

**Request Schema**:
```json
{
  "type": "object",
  "properties": {
    "query": {
      "type": "string",
      "description": "Query to analyze for context",
      "minLength": 1,
      "maxLength": 1000
    },
    "research_type": {
      "type": "string",
      "description": "Research type hint for better context detection",
      "enum": ["research", "troubleshooting", "learning", "implementation", "decision", "validation"]
    }
  },
  "required": ["query"]
}
```

**Response Schema**:
```json
{
  "type": "object",
  "properties": {
    "audience_level": {
      "type": "string",
      "enum": ["beginner", "intermediate", "advanced", "expert"],
      "description": "Detected audience level"
    },
    "technical_domain": {
      "type": "string",
      "description": "Detected technical domain"
    },
    "urgency_level": {
      "type": "string",
      "enum": ["low", "medium", "high", "critical"],
      "description": "Detected urgency level"
    },
    "overall_confidence": {
      "type": "number",
      "minimum": 0.0,
      "maximum": 1.0,
      "description": "Overall confidence score"
    },
    "processing_time_ms": {
      "type": "integer",
      "description": "Processing time in milliseconds"
    },
    "fallback_used": {
      "type": "boolean",
      "description": "Whether fallback detection was used"
    },
    "dimension_confidences": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "dimension": {
            "type": "string",
            "description": "Dimension name (audience, domain, urgency)"
          },
          "confidence": {
            "type": "number",
            "description": "Confidence score for this dimension"
          },
          "keywords": {
            "type": "array",
            "items": {
              "type": "string"
            },
            "description": "Keywords that influenced this dimension"
          },
          "explanation": {
            "type": "string",
            "description": "Explanation of the detection"
          }
        }
      },
      "description": "Dimension-specific confidence information"
    }
  }
}
```

**Example Request**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-3",
  "method": "tools/call",
  "params": {
    "name": "detect_context",
    "arguments": {
      "query": "I need help with this urgent production issue",
      "research_type": "troubleshooting"
    }
  }
}
```

**Example Response**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-3",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"audience_level\":\"intermediate\",\"technical_domain\":\"systems\",\"urgency_level\":\"high\",\"overall_confidence\":0.87,\"processing_time_ms\":156,\"fallback_used\":false,\"dimension_confidences\":[{\"dimension\":\"urgency\",\"confidence\":0.95,\"keywords\":[\"urgent\",\"production\",\"issue\"],\"explanation\":\"Strong urgency indicators detected\"},{\"dimension\":\"audience\",\"confidence\":0.75,\"keywords\":[\"help\",\"issue\"],\"explanation\":\"Professional language suggests intermediate level\"},{\"dimension\":\"domain\",\"confidence\":0.65,\"keywords\":[\"production\"],\"explanation\":\"Production context suggests systems domain\"}]}"
      }
    ],
    "isError": false
  }
}
```

**Permissions Required**: `fortitude:research:read`
</implementation>

## <resources>MCP Resources</resources>

### <resource>Reference Library</resource>

Access to the Fortitude reference library documentation.

<implementation>
**URI Pattern**: `mcp://fortitude/docs/reference_library/{path}`
**Method**: `resources/read`

**Available Resources**:
- `mcp://fortitude/docs/reference_library/README.md` - Library overview
- `mcp://fortitude/docs/reference_library/research/` - Research documentation
- `mcp://fortitude/docs/reference_library/patterns/` - Implementation patterns
- `mcp://fortitude/docs/reference_library/quick-reference/` - Quick reference guides

**Example Request**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-4",
  "method": "resources/read",
  "params": {
    "uri": "mcp://fortitude/docs/reference_library/research/rust-patterns.md"
  }
}
```

**Example Response**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-4",
  "result": {
    "contents": [
      {
        "type": "text",
        "text": "# Rust Implementation Patterns\n\n## Error Handling Patterns\n\n### Result Type Usage\n\n```rust\nuse std::error::Error;\nuse std::fmt;\n\n#[derive(Debug)]\nstruct CustomError {\n    message: String,\n}\n\nimpl fmt::Display for CustomError {\n    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {\n        write!(f, \"Custom error: {}\", self.message)\n    }\n}\n\nimpl Error for CustomError {}\n```\n\n## Async Patterns\n\n### Concurrent Processing\n\n```rust\nuse tokio::task;\nuse std::sync::Arc;\n\nasync fn process_items(items: Vec<Item>) -> Vec<Result<Output, ProcessError>> {\n    let tasks: Vec<_> = items.into_iter().map(|item| {\n        task::spawn(async move {\n            process_item(item).await\n        })\n    }).collect();\n    \n    let results = futures::future::join_all(tasks).await;\n    results.into_iter().map(|r| r.unwrap()).collect()\n}\n```",
        "uri": "mcp://fortitude/docs/reference_library/research/rust-patterns.md"
      }
    ]
  }
}
```

**Permissions Required**: `fortitude:resources:read`
</implementation>

### <resource>Cache Statistics</resource>

Access to cache performance metrics and statistics.

<implementation>
**URI**: `mcp://fortitude/cache/statistics`
**Method**: `resources/read`

**Example Request**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-5",
  "method": "resources/read",
  "params": {
    "uri": "mcp://fortitude/cache/statistics"
  }
}
```

**Example Response**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-5",
  "result": {
    "contents": [
      {
        "type": "text",
        "text": "{\n  \"total_entries\": 1247,\n  \"hit_rate\": 0.78,\n  \"miss_rate\": 0.22,\n  \"total_size_bytes\": 2048576,\n  \"average_age_seconds\": 1800.5,\n  \"expired_entries\": 23,\n  \"by_research_type\": {\n    \"implementation\": 456,\n    \"troubleshooting\": 321,\n    \"learning\": 287,\n    \"decision\": 183\n  },\n  \"last_updated\": \"2025-07-09T12:34:56.789Z\",\n  \"cache_enabled\": true,\n  \"max_size_bytes\": 104857600,\n  \"ttl_seconds\": 3600\n}",
        "uri": "mcp://fortitude/cache/statistics"
      }
    ]
  }
}
```

**Permissions Required**: `fortitude:resources:read`
</implementation>

### <resource>Configuration State</resource>

Access to current server configuration (sanitized).

<implementation>
**URI**: `mcp://fortitude/config/current`
**Method**: `resources/read`

**Example Request**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-6",
  "method": "resources/read",
  "params": {
    "uri": "mcp://fortitude/config/current"
  }
}
```

**Example Response**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-6",
  "result": {
    "contents": [
      {
        "type": "text",
        "text": "{\n  \"port\": 8080,\n  \"host\": \"127.0.0.1\",\n  \"max_connections\": 1000,\n  \"request_timeout\": 30,\n  \"auth\": {\n    \"enabled\": true,\n    \"token_expiration_hours\": 24,\n    \"jwt_secret\": \"[REDACTED]\"\n  },\n  \"logging\": {\n    \"level\": \"info\",\n    \"structured\": true,\n    \"file_path\": \"./fortitude-mcp-server.log\"\n  },\n  \"performance\": {\n    \"cache_size\": 1000,\n    \"cache_ttl\": 300,\n    \"enable_deduplication\": true\n  }\n}",
        "uri": "mcp://fortitude/config/current"
      }
    ]
  }
}
```

**Permissions Required**: `fortitude:config:read`
</implementation>

### <resource>System Metrics</resource>

Access to system performance and health metrics.

<implementation>
**URI**: `mcp://fortitude/system/metrics`
**Method**: `resources/read`

**Example Request**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-7",
  "method": "resources/read",
  "params": {
    "uri": "mcp://fortitude/system/metrics"
  }
}
```

**Example Response**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-7",
  "result": {
    "contents": [
      {
        "type": "text",
        "text": "{\n  \"timestamp\": \"2025-07-09T12:34:56.789Z\",\n  \"uptime_seconds\": 86400,\n  \"memory_usage\": {\n    \"total_bytes\": 67108864,\n    \"used_bytes\": 33554432,\n    \"free_bytes\": 33554432\n  },\n  \"cpu_usage\": {\n    \"percentage\": 12.5,\n    \"load_average\": [0.8, 1.2, 0.9]\n  },\n  \"network\": {\n    \"connections_active\": 45,\n    \"bytes_sent\": 1048576,\n    \"bytes_received\": 2097152\n  },\n  \"disk\": {\n    \"total_bytes\": 10737418240,\n    \"used_bytes\": 3221225472,\n    \"free_bytes\": 7516192768\n  },\n  \"process\": {\n    \"pid\": 12345,\n    \"threads\": 8,\n    \"files_open\": 23\n  }\n}",
        "uri": "mcp://fortitude/system/metrics"
      }
    ]
  }
}
```

**Permissions Required**: `fortitude:admin`
</implementation>

## <authentication>Authentication</authentication>

### <jwt-auth>JWT Authentication</jwt-auth>

<implementation>
**Token Format**: JSON Web Token (JWT) with HS256 algorithm
**Header**: `Authorization: Bearer <token>`
**Required Claims**: `sub` (subject), `permissions` (array), `exp` (expiration)

**Token Generation**:
```bash
fortitude-mcp-server generate-token \
  --permissions "fortitude:research:read,fortitude:resources:read" \
  --expiration-hours 24
```

**Token Structure**:
```json
{
  "header": {
    "alg": "HS256",
    "typ": "JWT"
  },
  "payload": {
    "sub": "client-id",
    "permissions": [
      "fortitude:research:read",
      "fortitude:resources:read"
    ],
    "exp": 1709256000,
    "iat": 1709169600,
    "iss": "fortitude-mcp-server"
  }
}
```
</implementation>

### <permissions>Permission System</permissions>

<implementation>
**Available Permissions**:
- `fortitude:research:read` - Access to research tools
- `fortitude:resources:read` - Access to reference library resources
- `fortitude:config:read` - Access to configuration resources
- `fortitude:admin` - Full administrative access (includes all permissions)

**Permission Mapping**:
```json
{
  "tools": {
    "research_query": ["fortitude:research:read"],
    "classify_query": ["fortitude:research:read"],
    "detect_context": ["fortitude:research:read"]
  },
  "resources": {
    "mcp://fortitude/docs/**": ["fortitude:resources:read"],
    "mcp://fortitude/cache/**": ["fortitude:resources:read"],
    "mcp://fortitude/config/**": ["fortitude:config:read"],
    "mcp://fortitude/system/**": ["fortitude:admin"]
  }
}
```
</implementation>

## <error-handling>Error Handling</error-handling>

### <error-codes>Error Codes</error-codes>

<implementation>
**MCP Error Codes**:
- `-32700` - Parse error (invalid JSON)
- `-32600` - Invalid request (malformed JSON-RPC)
- `-32601` - Method not found
- `-32602` - Invalid params
- `-32603` - Internal error
- `-32000` to `-32099` - Server error range

**Fortitude-Specific Error Codes**:
- `-32000` - Authentication error
- `-32001` - Permission denied
- `-32002` - Rate limit exceeded
- `-32003` - Invalid input
- `-32004` - Resource not found
- `-32005` - Service unavailable
</implementation>

### <error-responses>Error Response Format</error-responses>

<implementation>
**Standard Error Response**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-id",
  "error": {
    "code": -32000,
    "message": "Authentication failed",
    "data": {
      "type": "AUTHENTICATION_ERROR",
      "details": "Invalid JWT token",
      "retry_after": null,
      "documentation_url": "https://docs.fortitude.ai/authentication"
    }
  }
}
```

**Permission Denied Error**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-id",
  "error": {
    "code": -32001,
    "message": "Permission denied",
    "data": {
      "type": "PERMISSION_DENIED",
      "details": "Required permission: fortitude:resources:read",
      "required_permissions": ["fortitude:resources:read"],
      "current_permissions": ["fortitude:research:read"]
    }
  }
}
```

**Rate Limit Error**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-id",
  "error": {
    "code": -32002,
    "message": "Rate limit exceeded",
    "data": {
      "type": "RATE_LIMIT_EXCEEDED",
      "details": "Maximum 60 requests per minute exceeded",
      "retry_after": 30,
      "limit": 60,
      "window_seconds": 60
    }
  }
}
```

**Input Validation Error**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-id",
  "error": {
    "code": -32003,
    "message": "Invalid input",
    "data": {
      "type": "VALIDATION_ERROR",
      "details": "Query length must be between 1 and 1000 characters",
      "field": "query",
      "validation_errors": [
        {
          "field": "query",
          "message": "Length must be between 1 and 1000 characters",
          "code": "LENGTH_VALIDATION"
        }
      ]
    }
  }
}
```
</implementation>

## <rate-limiting>Rate Limiting</rate-limiting>

### <rate-limit-configuration>Rate Limit Configuration</rate-limit-configuration>

<implementation>
**Default Limits**:
- **Research Tools**: 60 requests per minute per client
- **Resource Access**: 120 requests per minute per client
- **Configuration Access**: 30 requests per minute per client

**Rate Limit Headers**:
```http
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1709169600
X-RateLimit-Window: 60
```

**Rate Limit Algorithm**: Token bucket with time-based windows
</implementation>

## <pagination>Pagination</pagination>

### <resource-pagination>Resource Pagination</resource-pagination>

<implementation>
**List Resources with Pagination**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-8",
  "method": "resources/list",
  "params": {
    "cursor": "eyJvZmZzZXQiOjEwMCwibGltaXQiOjUwfQ==",
    "limit": 50
  }
}
```

**Paginated Response**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-8",
  "result": {
    "resources": [
      {
        "uri": "mcp://fortitude/docs/reference_library/file1.md",
        "name": "file1.md",
        "description": "Reference library file",
        "mimeType": "text/markdown",
        "size": 1024
      }
    ],
    "nextCursor": "eyJvZmZzZXQiOjE1MCwibGltaXQiOjUwfQ=="
  }
}
```
</implementation>

## <security-considerations>Security Considerations</security-considerations>

### <input-validation>Input Validation</input-validation>

<implementation>
**Validation Rules**:
- **Query Length**: 1-1000 characters
- **String Fields**: No control characters, proper encoding
- **URI Paths**: No path traversal (`../`, `~`)
- **JSON Structure**: Proper schema validation
- **Rate Limits**: Per-client request limiting

**Sanitization**:
- HTML entity encoding
- SQL injection prevention
- Command injection prevention
- Path traversal protection
</implementation>

### <security-headers>Security Headers</security-headers>

<implementation>
**Response Headers**:
```http
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Strict-Transport-Security: max-age=31536000; includeSubDomains
Content-Security-Policy: default-src 'self'
```
</implementation>

## <monitoring>Monitoring and Observability</monitoring>

### <logging>Structured Logging</logging>

<implementation>
**Log Format**: JSON structured logging
**Log Levels**: TRACE, DEBUG, INFO, WARN, ERROR

**Example Log Entry**:
```json
{
  "timestamp": "2025-07-09T12:34:56.789Z",
  "level": "INFO",
  "message": "Tool call completed",
  "fields": {
    "tool_name": "research_query",
    "processing_time_ms": 245,
    "client_id": "claude-code-client",
    "request_id": "req-123",
    "cache_hit": true
  }
}
```
</implementation>

### <metrics>Performance Metrics</metrics>

<implementation>
**Available Metrics**:
- Request latency (p50, p95, p99)
- Request rate (requests/second)
- Error rate (errors/second)
- Cache hit rate
- Memory usage
- CPU usage
- Active connections

**Metrics Endpoint**: Available via system metrics resource
</implementation>

## <versioning>API Versioning</versioning>

### <version-compatibility>Version Compatibility</version-compatibility>

<implementation>
**Current Version**: 0.2.0
**Protocol Version**: 2024-11-05
**Compatibility**: Backward compatible with v0.1.x

**Version Detection**:
```json
{
  "jsonrpc": "2.0",
  "id": "req-9",
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "roots": {
        "listChanged": true
      }
    },
    "clientInfo": {
      "name": "claude-code",
      "version": "1.0.0"
    }
  }
}
```
</implementation>

## <examples>Complete Examples</examples>

### <example>Full Research Workflow</example>

<implementation>
```json
// 1. Classify query
{
  "jsonrpc": "2.0",
  "id": "workflow-1",
  "method": "tools/call",
  "params": {
    "name": "classify_query",
    "arguments": {
      "query": "How to optimize Rust code for performance?"
    }
  }
}

// 2. Detect context
{
  "jsonrpc": "2.0",
  "id": "workflow-2",
  "method": "tools/call",
  "params": {
    "name": "detect_context",
    "arguments": {
      "query": "How to optimize Rust code for performance?",
      "research_type": "implementation"
    }
  }
}

// 3. Research with context
{
  "jsonrpc": "2.0",
  "id": "workflow-3",
  "method": "tools/call",
  "params": {
    "name": "research_query",
    "arguments": {
      "query": "How to optimize Rust code for performance?",
      "query_type": "implementation",
      "audience": "intermediate",
      "domain": "rust"
    }
  }
}

// 4. Access related resources
{
  "jsonrpc": "2.0",
  "id": "workflow-4",
  "method": "resources/read",
  "params": {
    "uri": "mcp://fortitude/docs/reference_library/patterns/rust-optimization.md"
  }
}
```
</implementation>

---

**API Reference Complete**: This documentation covers all available MCP tools, resources, authentication, and error handling for the Fortitude MCP Server.