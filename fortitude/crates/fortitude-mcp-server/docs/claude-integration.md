# Claude Code Integration Guide

<meta>
  <title>Claude Code Integration Guide</title>
  <type>integration_guide</type>
  <audience>developer</audience>
  <complexity>medium</complexity>
  <updated>2025-07-09</updated>
</meta>

## <summary priority="high">Overview</summary>

**Purpose**: Complete guide for integrating Fortitude MCP Server with Claude Code
**Output**: Working MCP connection enabling research tools and resources in Claude Code
**Prerequisites**: Running Fortitude MCP Server (see [setup.md](setup.md))

## <mcp-configuration>MCP Configuration in Claude Code</mcp-configuration>

### <config-method>Method 1: Configuration File</config-method>

<implementation>
Create or edit your Claude Code MCP configuration file:

**Location**: `~/.config/claude-code/mcp-config.json`

```json
{
  "servers": {
    "fortitude": {
      "command": "fortitude-mcp-server",
      "args": ["start", "--config", "~/.config/fortitude/mcp-server.toml"],
      "description": "Fortitude AI research assistant with comprehensive reference library",
      "capabilities": {
        "tools": [
          "research_query",
          "classify_query", 
          "detect_context"
        ],
        "resources": [
          "reference_library",
          "cache_statistics",
          "configuration",
          "system_metrics"
        ]
      },
      "authentication": {
        "type": "jwt",
        "token_command": "fortitude-mcp-server generate-token --permissions 'fortitude:research:read,fortitude:resources:read'"
      },
      "settings": {
        "timeout": 30,
        "max_retries": 3,
        "cache_responses": true
      }
    }
  }
}
```
</implementation>

### <config-method>Method 2: Environment-Based Configuration</config-method>

<implementation>
For containerized or CI/CD environments:

```json
{
  "servers": {
    "fortitude": {
      "command": "fortitude-mcp-server",
      "args": ["start"],
      "env": {
        "MCP_SERVER_HOST": "127.0.0.1",
        "MCP_SERVER_PORT": "8080",
        "MCP_JWT_SECRET": "${FORTITUDE_JWT_SECRET}",
        "MCP_AUTH_ENABLED": "true",
        "MCP_LOG_LEVEL": "info"
      },
      "description": "Fortitude AI research assistant",
      "authentication": {
        "type": "jwt",
        "token": "${FORTITUDE_JWT_TOKEN}"
      }
    }
  }
}
```
</implementation>

## <connection-setup>Connection Setup</connection-setup>

### <step>Step 1: Start MCP Server</step>

<implementation>
```bash
# Start server in the background
fortitude-mcp-server start --config ~/.config/fortitude/mcp-server.toml &

# Or as a service
sudo systemctl start fortitude-mcp-server

# Verify server is running
curl -i http://localhost:8080/health
```
</implementation>

### <step>Step 2: Configure Claude Code</step>

<implementation>
1. **Open Claude Code Settings**:
   - Go to Settings → MCP Servers
   - Or edit `~/.config/claude-code/mcp-config.json` directly

2. **Add Fortitude Server**:
   - Click "Add Server"
   - Use the JSON configuration above
   - Test connection

3. **Verify Connection**:
   - Check "Connected Servers" tab
   - Should show "fortitude" with green status
   - Tools and resources should be listed
</implementation>

### <step>Step 3: Test Integration</step>

<implementation>
In Claude Code, try these commands:

```javascript
// Test research query tool
await mcp.tools.research_query({
  query: "How to implement async functions in Rust?",
  query_type: "implementation",
  audience: "intermediate"
});

// Test classification tool
await mcp.tools.classify_query({
  query: "How to debug a segfault in Rust?"
});

// Test context detection
await mcp.tools.detect_context({
  query: "I need help with this urgent production issue",
  research_type: "troubleshooting"
});
```
</implementation>

## <authentication-setup>Authentication Configuration</authentication-setup>

### <auth-method>Method 1: Token Generation</auth-method>

<implementation>
```bash
# Generate authentication token
fortitude-mcp-server generate-token \
  --permissions "fortitude:research:read,fortitude:resources:read,fortitude:config:read" \
  --expiration-hours 24

# Output example:
# Generated JWT token:
# eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJjbGF1ZGUtY29kZSIsInBlcm1pc3Npb25zIjpbImZvcnRpdHVkZTpyZXNlYXJjaDpyZWFkIiwiZm9ydGl0dWRlOnJlc291cmNlczpyZWFkIl0sImV4cCI6MTcwOTI1NjAwMH0.signature
```

Use this token in your Claude Code configuration:
```json
{
  "authentication": {
    "type": "jwt",
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
}
```
</implementation>

### <auth-method>Method 2: Dynamic Token Generation</auth-method>

<implementation>
For automatic token refresh:

```json
{
  "authentication": {
    "type": "jwt",
    "token_command": "fortitude-mcp-server generate-token --permissions 'fortitude:research:read,fortitude:resources:read' --expiration-hours 8",
    "refresh_interval": 21600  // 6 hours in seconds
  }
}
```
</implementation>

### <auth-method>Method 3: Development Mode (No Authentication)</auth-method>

<implementation>
For development only:

```bash
# Start server without authentication
export MCP_AUTH_ENABLED=false
fortitude-mcp-server start --config dev-config.toml
```

Claude Code configuration:
```json
{
  "authentication": {
    "type": "none"
  }
}
```

**⚠️ Warning**: Only use this in development environments!
</implementation>

## <usage-examples>Usage Examples in Claude Code</usage-examples>

### <example>Research Query Examples</example>

<implementation>
```javascript
// Basic research query
const result = await mcp.tools.research_query({
  query: "What are the best practices for error handling in Rust?"
});

// Advanced research with context
const contextualResult = await mcp.tools.research_query({
  query: "How to implement a web server in Rust?",
  query_type: "implementation",
  audience: "intermediate",
  domain: "web"
});

// Learning-focused query
const learningResult = await mcp.tools.research_query({
  query: "Explain ownership and borrowing in Rust",
  query_type: "learning",
  audience: "beginner"
});
```
</implementation>

### <example>Classification Examples</example>

<implementation>
```javascript
// Classify user queries
const classification = await mcp.tools.classify_query({
  query: "My application is crashing with a segfault"
});
// Expected: research_type: "troubleshooting", confidence: 0.95

// Classify implementation questions
const implClassification = await mcp.tools.classify_query({
  query: "How do I create a REST API in Rust?"
});
// Expected: research_type: "implementation", confidence: 0.87
```
</implementation>

### <example>Context Detection Examples</example>

<implementation>
```javascript
// Detect context for better responses
const context = await mcp.tools.detect_context({
  query: "I need help with this urgent production issue",
  research_type: "troubleshooting"
});

// Use context to tailor responses
if (context.urgency_level === "high") {
  // Prioritize immediate solutions
  const urgentHelp = await mcp.tools.research_query({
    query: "Quick fix for Rust production crash",
    query_type: "troubleshooting",
    audience: "advanced"
  });
}
```
</implementation>

### <example>Resource Access Examples</example>

<implementation>
```javascript
// Access reference library files
const docContent = await mcp.resources.read("mcp://fortitude/docs/reference_library/rust-patterns.md");

// Get cache statistics
const cacheStats = await mcp.resources.read("mcp://fortitude/cache/statistics");

// Access configuration
const config = await mcp.resources.read("mcp://fortitude/config/current");

// Get system metrics
const metrics = await mcp.resources.read("mcp://fortitude/system/metrics");
```
</implementation>

## <workflow-integration>Workflow Integration</workflow-integration>

### <workflow>Research Assistant Workflow</workflow>

<implementation>
Create a Claude Code workflow that combines multiple tools:

```javascript
async function researchAssistant(userQuery) {
  // Step 1: Classify the query
  const classification = await mcp.tools.classify_query({
    query: userQuery
  });
  
  // Step 2: Detect context
  const context = await mcp.tools.detect_context({
    query: userQuery,
    research_type: classification.research_type
  });
  
  // Step 3: Perform research with context
  const research = await mcp.tools.research_query({
    query: userQuery,
    query_type: classification.research_type,
    audience: context.audience_level,
    domain: context.technical_domain
  });
  
  // Step 4: Format response
  return {
    classification: classification.research_type,
    confidence: classification.confidence,
    context: {
      audience: context.audience_level,
      domain: context.technical_domain,
      urgency: context.urgency_level
    },
    answer: research.result,
    metadata: research.metadata
  };
}

// Usage
const result = await researchAssistant("How to optimize Rust code for performance?");
```
</implementation>

### <workflow>Adaptive Response Workflow</workflow>

<implementation>
```javascript
async function adaptiveResponse(userQuery) {
  const context = await mcp.tools.detect_context({
    query: userQuery
  });
  
  // Adjust response based on detected context
  const queryParams = {
    query: userQuery,
    query_type: "implementation", // Default
    audience: context.audience_level,
    domain: context.technical_domain
  };
  
  // Adjust based on urgency
  if (context.urgency_level === "high") {
    queryParams.query_type = "troubleshooting";
  } else if (context.audience_level === "beginner") {
    queryParams.query_type = "learning";
  }
  
  const research = await mcp.tools.research_query(queryParams);
  
  return {
    response: research.result,
    tailored_for: {
      audience: context.audience_level,
      domain: context.technical_domain,
      urgency: context.urgency_level
    }
  };
}
```
</implementation>

## <troubleshooting>Connection Troubleshooting</troubleshooting>

### <issue>Server Not Starting</issue>

<implementation>
**Check server status:**
```bash
# Check if server is running
ps aux | grep fortitude-mcp-server
netstat -tlnp | grep 8080

# Check logs
tail -f ~/.config/fortitude/mcp-server.log
journalctl -u fortitude-mcp-server -f
```

**Common fixes:**
```bash
# Port conflict
export MCP_SERVER_PORT=8081

# Permission issues
sudo chown -R $USER:$USER ~/.config/fortitude/

# Missing dependencies
cargo install fortitude-mcp-server
```
</implementation>

### <issue>Authentication Failures</issue>

<implementation>
**Check token validity:**
```bash
# Generate new token
fortitude-mcp-server generate-token \
  --permissions "fortitude:research:read,fortitude:resources:read"

# Verify token format
echo "YOUR_TOKEN" | base64 -d
```

**Update Claude Code configuration:**
```json
{
  "authentication": {
    "type": "jwt",
    "token": "NEW_TOKEN_HERE"
  }
}
```
</implementation>

### <issue>Tool Not Found</issue>

<implementation>
**Verify tool availability:**
```bash
# List available tools
curl -X GET http://localhost:8080/mcp/tools \
  -H "Authorization: Bearer YOUR_TOKEN"
```

**Check Claude Code configuration:**
```json
{
  "capabilities": {
    "tools": [
      "research_query",
      "classify_query",
      "detect_context"
    ]
  }
}
```
</implementation>

### <issue>Resource Access Denied</issue>

<implementation>
**Check permissions:**
```bash
# Generate token with resource permissions
fortitude-mcp-server generate-token \
  --permissions "fortitude:resources:read,fortitude:config:read"
```

**Verify resource URI:**
```javascript
// Correct URI format
const content = await mcp.resources.read("mcp://fortitude/docs/reference_library/README.md");

// Incorrect - will fail
const content = await mcp.resources.read("file:///path/to/file");
```
</implementation>

### <issue>Connection Timeout</issue>

<implementation>
**Increase timeout in Claude Code:**
```json
{
  "settings": {
    "timeout": 60,
    "max_retries": 5,
    "retry_delay": 1000
  }
}
```

**Optimize server performance:**
```bash
# Increase server limits
export MCP_MAX_CONNECTIONS=2000
export MCP_REQUEST_TIMEOUT=60
```
</implementation>

## <performance-optimization>Performance Optimization</performance-optimization>

### <caching>Response Caching</caching>

<implementation>
Enable caching in Claude Code:
```json
{
  "settings": {
    "cache_responses": true,
    "cache_ttl": 300,
    "cache_size": 1000
  }
}
```

Configure server-side caching:
```toml
[mcp_server.performance]
cache_size = 10000
cache_ttl = 600
enable_deduplication = true
```
</implementation>

### <connection-pooling>Connection Pooling</connection-pooling>

<implementation>
```json
{
  "settings": {
    "connection_pool_size": 10,
    "keep_alive": true,
    "max_idle_connections": 5
  }
}
```
</implementation>

### <batch-requests>Batch Requests</batch-requests>

<implementation>
```javascript
// Batch multiple queries
const batchResults = await Promise.all([
  mcp.tools.classify_query({query: "Query 1"}),
  mcp.tools.classify_query({query: "Query 2"}),
  mcp.tools.classify_query({query: "Query 3"})
]);
```
</implementation>

## <monitoring>Monitoring and Logging</monitoring>

### <logging>Enable Detailed Logging</logging>

<implementation>
**Claude Code logging:**
```json
{
  "logging": {
    "level": "debug",
    "file": "~/.config/claude-code/mcp-fortitude.log",
    "include_requests": true,
    "include_responses": true
  }
}
```

**Server-side logging:**
```bash
export MCP_LOG_LEVEL=debug
export MCP_LOG_FILE_PATH=/var/log/fortitude-mcp-server.log
```
</implementation>

### <metrics>Connection Metrics</metrics>

<implementation>
```javascript
// Get connection statistics
const stats = await mcp.resources.read("mcp://fortitude/system/metrics");

// Monitor cache performance
const cacheStats = await mcp.resources.read("mcp://fortitude/cache/statistics");

// Check server configuration
const config = await mcp.resources.read("mcp://fortitude/config/current");
```
</implementation>

## <security-considerations>Security Considerations</security-considerations>

### <token-security>Token Security</token-security>

<implementation>
**Best practices:**
- Use minimum required permissions
- Set reasonable expiration times
- Rotate tokens regularly
- Store tokens securely

```bash
# Generate tokens with minimal permissions
fortitude-mcp-server generate-token \
  --permissions "fortitude:research:read" \
  --expiration-hours 8

# Store in secure location
echo "TOKEN" > ~/.config/claude-code/fortitude-token
chmod 600 ~/.config/claude-code/fortitude-token
```
</implementation>

### <network-security>Network Security</network-security>

<implementation>
**Production configuration:**
```json
{
  "servers": {
    "fortitude": {
      "command": "fortitude-mcp-server",
      "args": ["start", "--config", "/etc/fortitude/production-config.toml"],
      "security": {
        "verify_ssl": true,
        "allowed_hosts": ["localhost", "127.0.0.1"],
        "max_request_size": 1048576
      }
    }
  }
}
```
</implementation>

## <migration>Migration Guide</migration>

### <upgrade>Upgrading from Previous Versions</upgrade>

<implementation>
**Version 0.1.x to 0.2.x:**
```bash
# Backup configuration
cp ~/.config/fortitude/mcp-server.toml ~/.config/fortitude/mcp-server.toml.backup

# Update binary
cargo install fortitude-mcp-server --force

# Update configuration format
fortitude-mcp-server migrate-config \
  --from ~/.config/fortitude/mcp-server.toml.backup \
  --to ~/.config/fortitude/mcp-server.toml
```
</implementation>

## <advanced-configuration>Advanced Configuration</advanced-configuration>

### <custom-tools>Custom Tool Configuration</custom-tools>

<implementation>
```json
{
  "servers": {
    "fortitude": {
      "tools": {
        "research_query": {
          "timeout": 30,
          "cache_ttl": 300,
          "max_retries": 3
        },
        "classify_query": {
          "timeout": 10,
          "cache_ttl": 600,
          "max_retries": 2
        }
      }
    }
  }
}
```
</implementation>

### <resource-filtering>Resource Filtering</resource-filtering>

<implementation>
```json
{
  "servers": {
    "fortitude": {
      "resources": {
        "include_patterns": [
          "mcp://fortitude/docs/reference_library/**/*.md",
          "mcp://fortitude/cache/statistics"
        ],
        "exclude_patterns": [
          "mcp://fortitude/config/**",
          "mcp://fortitude/system/metrics"
        ]
      }
    }
  }
}
```
</implementation>

## <next-steps>Next Steps</next-steps>

After successful Claude Code integration:

1. **Explore API Documentation**: [api-reference.md](api-reference.md)
2. **Review Usage Examples**: [examples.md](examples.md)
3. **Set Up Monitoring**: [performance.md](performance.md)
4. **Configure Production Security**: [troubleshooting.md](troubleshooting.md)

---

**Integration Complete**: Your Claude Code environment now has access to Fortitude's AI research capabilities through the MCP protocol.