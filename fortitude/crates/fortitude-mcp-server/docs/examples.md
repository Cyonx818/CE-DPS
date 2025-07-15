# Fortitude MCP Server Usage Examples

<meta>
  <title>Fortitude MCP Server Usage Examples</title>
  <type>usage_examples</type>
  <audience>developer</audience>
  <complexity>medium</complexity>
  <updated>2025-07-09</updated>
</meta>

## <summary priority="high">Overview</summary>

**Purpose**: Practical examples and workflows for using the Fortitude MCP Server
**Scope**: Common research workflows, tool usage patterns, resource access, and CLI commands
**Prerequisites**: Running MCP server and Claude Code integration

## <research-workflows>Common Research Workflows</research-workflows>

### <workflow>Basic Research Query</workflow>

<implementation>
**Scenario**: Simple research question about Rust programming

```javascript
// Claude Code usage
const result = await mcp.tools.research_query({
  query: "How to handle errors in Rust?",
  query_type: "learning",
  audience: "beginner"
});

console.log(result.result);
// Output: Comprehensive guide on Rust error handling with examples
```

**Raw MCP Request**:
```json
{
  "jsonrpc": "2.0",
  "id": "example-1",
  "method": "tools/call",
  "params": {
    "name": "research_query",
    "arguments": {
      "query": "How to handle errors in Rust?",
      "query_type": "learning",
      "audience": "beginner"
    }
  }
}
```

**Expected Response**:
```json
{
  "jsonrpc": "2.0",
  "id": "example-1",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"result\":\"Error handling in Rust is fundamentally different from other languages. Here's a comprehensive guide:\\n\\n## The Result Type\\n\\nRust uses the `Result<T, E>` type for error handling:\\n\\n```rust\\nfn divide(a: i32, b: i32) -> Result<i32, String> {\\n    if b == 0 {\\n        Err(\\\"Division by zero\\\".to_string())\\n    } else {\\n        Ok(a / b)\\n    }\\n}\\n```\\n\\n## Error Propagation\\n\\nUse the `?` operator for clean error propagation:\\n\\n```rust\\nfn read_file() -> Result<String, std::io::Error> {\\n    let content = std::fs::read_to_string(\\\"file.txt\\\")?;\\n    Ok(content)\\n}\\n```\\n\\n## Custom Error Types\\n\\nCreate custom error types for better error handling:\\n\\n```rust\\nuse std::fmt;\\n\\n#[derive(Debug)]\\nstruct MyError {\\n    details: String,\\n}\\n\\nimpl fmt::Display for MyError {\\n    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {\\n        write!(f, \\\"{}\\\", self.details)\\n    }\\n}\\n\\nimpl std::error::Error for MyError {}\\n```\",\"metadata\":{\"research_type\":\"learning\",\"confidence\":0.91,\"processing_time_ms\":189,\"context_detection_used\":true,\"cache_key\":\"learning_rust_errors_beginner_abc123\"}}"
      }
    ],
    "isError": false
  }
}
```
</implementation>

### <workflow>Adaptive Research Workflow</workflow>

<implementation>
**Scenario**: Context-aware research that adapts to user level and urgency

```javascript
async function adaptiveResearch(query) {
  // Step 1: Classify the query type
  const classification = await mcp.tools.classify_query({
    query: query
  });
  
  // Step 2: Detect context (audience, domain, urgency)
  const context = await mcp.tools.detect_context({
    query: query,
    research_type: classification.research_type
  });
  
  // Step 3: Perform research with full context
  const research = await mcp.tools.research_query({
    query: query,
    query_type: classification.research_type,
    audience: context.audience_level,
    domain: context.technical_domain
  });
  
  // Step 4: Access relevant resources if needed
  let additionalResources = [];
  if (context.technical_domain === "rust") {
    const rustPatterns = await mcp.resources.read(
      "mcp://fortitude/docs/reference_library/patterns/rust-patterns.md"
    );
    additionalResources.push(rustPatterns);
  }
  
  return {
    classification: {
      type: classification.research_type,
      confidence: classification.confidence
    },
    context: {
      audience: context.audience_level,
      domain: context.technical_domain,
      urgency: context.urgency_level
    },
    research: research.result,
    processing_time: research.metadata.processing_time_ms,
    additional_resources: additionalResources
  };
}

// Usage examples
const result1 = await adaptiveResearch("How to implement async functions in Rust?");
const result2 = await adaptiveResearch("My production server is crashing!");
const result3 = await adaptiveResearch("What are the best practices for web development?");
```
</implementation>

### <workflow>Learning Path Workflow</workflow>

<implementation>
**Scenario**: Progressive learning with context building

```javascript
class LearningPathAssistant {
  constructor() {
    this.userContext = {
      level: "beginner",
      completedTopics: [],
      currentDomain: "rust"
    };
  }
  
  async getNextLesson(topic) {
    // Classify to understand what type of learning this is
    const classification = await mcp.tools.classify_query({
      query: `Learn about ${topic} in ${this.userContext.currentDomain}`
    });
    
    // Get contextual research
    const research = await mcp.tools.research_query({
      query: `Explain ${topic} for ${this.userContext.level} level`,
      query_type: "learning",
      audience: this.userContext.level,
      domain: this.userContext.currentDomain
    });
    
    // Track progress
    this.userContext.completedTopics.push(topic);
    
    // Suggest next topic based on progression
    const nextTopic = await this.suggestNextTopic();
    
    return {
      current_lesson: research.result,
      user_level: this.userContext.level,
      completion_status: this.userContext.completedTopics.length,
      next_suggested_topic: nextTopic
    };
  }
  
  async suggestNextTopic() {
    const progressQuery = `What should I learn next after ${this.userContext.completedTopics.join(", ")} in ${this.userContext.currentDomain}?`;
    
    const suggestion = await mcp.tools.research_query({
      query: progressQuery,
      query_type: "decision",
      audience: this.userContext.level,
      domain: this.userContext.currentDomain
    });
    
    return suggestion.result;
  }
}

// Usage
const learningAssistant = new LearningPathAssistant();

const lesson1 = await learningAssistant.getNextLesson("variables and data types");
const lesson2 = await learningAssistant.getNextLesson("ownership and borrowing");
const lesson3 = await learningAssistant.getNextLesson("error handling");
```
</implementation>

### <workflow>Troubleshooting Workflow</workflow>

<implementation>
**Scenario**: Systematic problem-solving approach

```javascript
async function troubleshootingAssistant(problem) {
  // Step 1: Understand the problem urgency and context
  const context = await mcp.tools.detect_context({
    query: problem,
    research_type: "troubleshooting"
  });
  
  // Step 2: Get immediate troubleshooting steps
  const troubleshooting = await mcp.tools.research_query({
    query: problem,
    query_type: "troubleshooting",
    audience: context.audience_level,
    domain: context.technical_domain
  });
  
  // Step 3: Access relevant troubleshooting resources
  const resources = await mcp.resources.read(
    "mcp://fortitude/docs/reference_library/troubleshooting/common-issues.md"
  );
  
  // Step 4: Provide structured response based on urgency
  const response = {
    urgency: context.urgency_level,
    immediate_steps: troubleshooting.result,
    confidence: troubleshooting.metadata.confidence,
    processing_time: troubleshooting.metadata.processing_time_ms
  };
  
  // Add additional resources for non-urgent issues
  if (context.urgency_level !== "high") {
    response.additional_resources = resources.contents[0].text;
  }
  
  return response;
}

// Usage examples
const issue1 = await troubleshootingAssistant("My Rust application is segfaulting");
const issue2 = await troubleshootingAssistant("Getting compiler errors I don't understand");
const issue3 = await troubleshootingAssistant("Performance issues in production");
```
</implementation>

## <tool-usage-patterns>Tool Usage Patterns</tool-usage-patterns>

### <pattern>Classification-First Pattern</pattern>

<implementation>
**Use Case**: When you need to understand query intent before processing

```javascript
async function classificationFirst(userQuery) {
  const classification = await mcp.tools.classify_query({
    query: userQuery
  });
  
  // Route based on classification
  switch (classification.research_type) {
    case "troubleshooting":
      return await handleTroubleshooting(userQuery, classification);
    case "implementation":
      return await handleImplementation(userQuery, classification);
    case "learning":
      return await handleLearning(userQuery, classification);
    default:
      return await handleGeneral(userQuery, classification);
  }
}

async function handleTroubleshooting(query, classification) {
  // Prioritize immediate solutions
  const context = await mcp.tools.detect_context({
    query: query,
    research_type: "troubleshooting"
  });
  
  return await mcp.tools.research_query({
    query: query,
    query_type: "troubleshooting",
    audience: context.audience_level,
    domain: context.technical_domain
  });
}
```
</implementation>

### <pattern>Context-Aware Pattern</pattern>

<implementation>
**Use Case**: When responses need to be tailored to user expertise

```javascript
async function contextAwareResponse(query) {
  const context = await mcp.tools.detect_context({
    query: query
  });
  
  // Adjust response complexity based on audience
  const audienceModifier = {
    "beginner": "simple explanation with examples",
    "intermediate": "detailed explanation with best practices",
    "advanced": "comprehensive technical details",
    "expert": "advanced concepts and optimizations"
  };
  
  const enhancedQuery = `${query} - provide ${audienceModifier[context.audience_level]}`;
  
  return await mcp.tools.research_query({
    query: enhancedQuery,
    query_type: "learning",
    audience: context.audience_level,
    domain: context.technical_domain
  });
}
```
</implementation>

### <pattern>Multi-Tool Validation Pattern</pattern>

<implementation>
**Use Case**: When you need to validate responses across multiple tools

```javascript
async function validatedResponse(query) {
  // Get initial classification
  const classification = await mcp.tools.classify_query({
    query: query
  });
  
  // Get context detection
  const context = await mcp.tools.detect_context({
    query: query,
    research_type: classification.research_type
  });
  
  // Get research response
  const research = await mcp.tools.research_query({
    query: query,
    query_type: classification.research_type,
    audience: context.audience_level,
    domain: context.technical_domain
  });
  
  // Validate consistency
  const validation = {
    classification_confidence: classification.confidence,
    context_confidence: context.overall_confidence,
    research_confidence: research.metadata.confidence,
    overall_confidence: (
      classification.confidence + 
      context.overall_confidence + 
      research.metadata.confidence
    ) / 3
  };
  
  return {
    result: research.result,
    validation: validation,
    reliable: validation.overall_confidence > 0.7
  };
}
```
</implementation>

## <resource-access-examples>Resource Access Examples</resource-access-examples>

### <example>Reference Library Navigation</example>

<implementation>
```javascript
async function exploreReferenceLibrary() {
  // Get list of available resources
  const resources = await mcp.resources.list();
  
  // Filter for reference library files
  const referenceFiles = resources.resources.filter(r => 
    r.uri.includes("reference_library")
  );
  
  // Read specific documentation
  const rustPatterns = await mcp.resources.read(
    "mcp://fortitude/docs/reference_library/patterns/rust-patterns.md"
  );
  
  const quickRef = await mcp.resources.read(
    "mcp://fortitude/docs/reference_library/quick-reference/rust-quick-ref.md"
  );
  
  return {
    available_files: referenceFiles.length,
    rust_patterns: rustPatterns.contents[0].text,
    quick_reference: quickRef.contents[0].text
  };
}
```
</implementation>

### <example>System Monitoring</example>

<implementation>
```javascript
async function monitorSystem() {
  // Get system metrics
  const metrics = await mcp.resources.read(
    "mcp://fortitude/system/metrics"
  );
  
  // Get cache statistics
  const cacheStats = await mcp.resources.read(
    "mcp://fortitude/cache/statistics"
  );
  
  // Get current configuration
  const config = await mcp.resources.read(
    "mcp://fortitude/config/current"
  );
  
  const systemData = JSON.parse(metrics.contents[0].text);
  const cacheData = JSON.parse(cacheStats.contents[0].text);
  const configData = JSON.parse(config.contents[0].text);
  
  return {
    health: {
      cpu_usage: systemData.cpu_usage.percentage,
      memory_usage: (systemData.memory_usage.used_bytes / systemData.memory_usage.total_bytes) * 100,
      active_connections: systemData.network.connections_active
    },
    performance: {
      cache_hit_rate: cacheData.hit_rate,
      cache_size: cacheData.total_entries,
      average_response_time: cacheData.average_age_seconds
    },
    configuration: {
      max_connections: configData.max_connections,
      cache_enabled: configData.performance.cache_ttl > 0,
      auth_enabled: configData.auth.enabled
    }
  };
}
```
</implementation>

### <example>Performance Analysis</example>

<implementation>
```javascript
async function analyzePerformance() {
  const cacheStats = await mcp.resources.read(
    "mcp://fortitude/cache/statistics"
  );
  
  const systemMetrics = await mcp.resources.read(
    "mcp://fortitude/system/metrics"
  );
  
  const cacheData = JSON.parse(cacheStats.contents[0].text);
  const systemData = JSON.parse(systemMetrics.contents[0].text);
  
  const analysis = {
    cache_efficiency: {
      hit_rate: cacheData.hit_rate,
      miss_rate: cacheData.miss_rate,
      recommendation: cacheData.hit_rate > 0.7 ? "Good" : "Consider tuning"
    },
    memory_usage: {
      percentage: (systemData.memory_usage.used_bytes / systemData.memory_usage.total_bytes) * 100,
      recommendation: systemData.memory_usage.used_bytes / systemData.memory_usage.total_bytes > 0.8 ? "High" : "Normal"
    },
    research_patterns: {
      most_common_type: Object.keys(cacheData.by_research_type).reduce((a, b) => 
        cacheData.by_research_type[a] > cacheData.by_research_type[b] ? a : b
      ),
      total_queries: Object.values(cacheData.by_research_type).reduce((a, b) => a + b, 0)
    }
  };
  
  return analysis;
}
```
</implementation>

## <cli-command-examples>CLI Command Examples</cli-command-examples>

### <cli-basic>Basic Server Management</cli-basic>

<implementation>
```bash
# Start server with default configuration
fortitude-mcp-server start

# Start with custom configuration
fortitude-mcp-server start --config ~/.config/fortitude/custom-config.toml

# Start with specific port and host
fortitude-mcp-server start --port 9090 --host 0.0.0.0

# Start with environment variable overrides
MCP_AUTH_ENABLED=false fortitude-mcp-server start

# Start in debug mode
fortitude-mcp-server start --log-level debug

# Start with custom data directory
fortitude-mcp-server start --data-dir /opt/fortitude/data
```
</implementation>

### <cli-auth>Authentication Management</cli-auth>

<implementation>
```bash
# Generate token with basic permissions
fortitude-mcp-server generate-token \
  --permissions "fortitude:research:read"

# Generate token with multiple permissions
fortitude-mcp-server generate-token \
  --permissions "fortitude:research:read,fortitude:resources:read,fortitude:config:read" \
  --expiration-hours 24

# Generate admin token
fortitude-mcp-server generate-token \
  --permissions "fortitude:admin" \
  --expiration-hours 8

# Generate token with custom client ID
fortitude-mcp-server generate-token \
  --client-id "claude-code-production" \
  --permissions "fortitude:research:read,fortitude:resources:read"

# Validate existing token
fortitude-mcp-server validate-token \
  --token "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# List active tokens (requires admin permissions)
fortitude-mcp-server list-tokens
```
</implementation>

### <cli-config>Configuration Management</cli-config>

<implementation>
```bash
# Validate configuration file
fortitude-mcp-server validate-config \
  --config ~/.config/fortitude/mcp-server.toml

# Show current configuration
fortitude-mcp-server show-config \
  --config ~/.config/fortitude/mcp-server.toml

# Generate sample configuration
fortitude-mcp-server generate-config \
  --output sample-config.toml \
  --format toml

# Generate production configuration
fortitude-mcp-server generate-config \
  --output production-config.toml \
  --format toml \
  --preset production

# Migrate configuration from older version
fortitude-mcp-server migrate-config \
  --from old-config.toml \
  --to new-config.toml

# Test configuration with dry run
fortitude-mcp-server test-config \
  --config ~/.config/fortitude/mcp-server.toml \
  --dry-run
```
</implementation>

### <cli-monitoring>Monitoring and Diagnostics</cli-monitoring>

<implementation>
```bash
# Check server health
fortitude-mcp-server health-check

# Get server status
fortitude-mcp-server status

# Show performance metrics
fortitude-mcp-server metrics

# Display cache statistics
fortitude-mcp-server cache-stats

# Show active connections
fortitude-mcp-server connections

# Export metrics to file
fortitude-mcp-server metrics --output metrics.json --format json

# Continuous monitoring
fortitude-mcp-server monitor --interval 5 --duration 300

# Performance benchmark
fortitude-mcp-server benchmark \
  --requests 1000 \
  --concurrency 10 \
  --tool research_query
```
</implementation>

### <cli-maintenance>Maintenance Operations</cli-maintenance>

<implementation>
```bash
# Clear cache
fortitude-mcp-server clear-cache

# Rebuild reference library index
fortitude-mcp-server rebuild-index

# Backup configuration and data
fortitude-mcp-server backup \
  --output backup-$(date +%Y%m%d).tar.gz

# Restore from backup
fortitude-mcp-server restore \
  --input backup-20250709.tar.gz

# Clean up old log files
fortitude-mcp-server cleanup-logs \
  --older-than 30d

# Rotate log files
fortitude-mcp-server rotate-logs

# Update reference library
fortitude-mcp-server update-library \
  --source https://github.com/your-org/fortitude-reference-library.git
```
</implementation>

## <batch-operations>Batch Operations</batch-operations>

### <batch-research>Batch Research Processing</batch-research>

<implementation>
```javascript
async function batchResearch(queries) {
  const results = [];
  
  // Process queries in batches to avoid rate limits
  const batchSize = 5;
  for (let i = 0; i < queries.length; i += batchSize) {
    const batch = queries.slice(i, i + batchSize);
    
    const batchPromises = batch.map(async (query) => {
      try {
        const classification = await mcp.tools.classify_query({
          query: query.text
        });
        
        const research = await mcp.tools.research_query({
          query: query.text,
          query_type: classification.research_type,
          audience: query.audience || "intermediate",
          domain: query.domain || "general"
        });
        
        return {
          query: query.text,
          result: research.result,
          classification: classification.research_type,
          confidence: classification.confidence,
          processing_time: research.metadata.processing_time_ms
        };
      } catch (error) {
        return {
          query: query.text,
          error: error.message,
          failed: true
        };
      }
    });
    
    const batchResults = await Promise.all(batchPromises);
    results.push(...batchResults);
    
    // Brief pause between batches
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  
  return results;
}

// Usage
const queries = [
  { text: "How to handle errors in Rust?", audience: "beginner", domain: "rust" },
  { text: "Best practices for async programming", audience: "intermediate", domain: "rust" },
  { text: "Memory management in systems programming", audience: "advanced", domain: "systems" }
];

const results = await batchResearch(queries);
```
</implementation>

### <batch-analysis>Batch Analysis</batch-analysis>

<implementation>
```javascript
async function analyzeQueries(queries) {
  const classifications = await Promise.all(
    queries.map(q => mcp.tools.classify_query({ query: q }))
  );
  
  const contexts = await Promise.all(
    queries.map((q, i) => mcp.tools.detect_context({
      query: q,
      research_type: classifications[i].research_type
    }))
  );
  
  return queries.map((query, i) => ({
    query: query,
    classification: classifications[i],
    context: contexts[i],
    recommended_approach: getRecommendedApproach(classifications[i], contexts[i])
  }));
}

function getRecommendedApproach(classification, context) {
  if (context.urgency_level === "high") {
    return "immediate_assistance";
  } else if (classification.research_type === "learning" && context.audience_level === "beginner") {
    return "guided_learning";
  } else if (classification.research_type === "implementation") {
    return "step_by_step_implementation";
  } else {
    return "comprehensive_research";
  }
}
```
</implementation>

## <integration-examples>Integration Examples</integration-examples>

### <integration-webhook>Webhook Integration</integration-webhook>

<implementation>
```javascript
// Express.js webhook endpoint
app.post('/webhook/research', async (req, res) => {
  try {
    const { query, user_id, context } = req.body;
    
    // Perform research using MCP tools
    const research = await mcp.tools.research_query({
      query: query,
      query_type: context.type || "learning",
      audience: context.audience || "intermediate",
      domain: context.domain || "general"
    });
    
    // Store result in database
    await db.storeResearchResult({
      user_id,
      query,
      result: research.result,
      metadata: research.metadata,
      timestamp: new Date()
    });
    
    res.json({
      success: true,
      result: research.result,
      processing_time: research.metadata.processing_time_ms
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});
```
</implementation>

### <integration-chatbot>Chatbot Integration</integration-chatbot>

<implementation>
```javascript
class FortitudeChatbot {
  constructor() {
    this.conversationHistory = new Map();
  }
  
  async handleMessage(userId, message) {
    // Get or create conversation context
    const context = this.conversationHistory.get(userId) || {
      previous_queries: [],
      user_level: "intermediate",
      current_domain: "general"
    };
    
    // Classify the message
    const classification = await mcp.tools.classify_query({
      query: message
    });
    
    // Detect context with conversation history
    const detectedContext = await mcp.tools.detect_context({
      query: message,
      research_type: classification.research_type
    });
    
    // Update user context based on conversation
    if (detectedContext.audience_level !== "unknown") {
      context.user_level = detectedContext.audience_level;
    }
    if (detectedContext.technical_domain !== "general") {
      context.current_domain = detectedContext.technical_domain;
    }
    
    // Get research response
    const research = await mcp.tools.research_query({
      query: message,
      query_type: classification.research_type,
      audience: context.user_level,
      domain: context.current_domain
    });
    
    // Update conversation history
    context.previous_queries.push({
      query: message,
      response: research.result,
      classification: classification.research_type,
      timestamp: new Date()
    });
    
    this.conversationHistory.set(userId, context);
    
    return {
      response: research.result,
      context: {
        user_level: context.user_level,
        current_domain: context.current_domain,
        classification: classification.research_type
      }
    };
  }
}

// Usage
const chatbot = new FortitudeChatbot();
const response = await chatbot.handleMessage("user123", "How do I debug Rust code?");
```
</implementation>

## <testing-examples>Testing Examples</testing-examples>

### <testing-unit>Unit Testing with MCP Tools</testing-unit>

<implementation>
```javascript
// Jest test example
describe('Fortitude MCP Tools', () => {
  test('should classify troubleshooting queries correctly', async () => {
    const result = await mcp.tools.classify_query({
      query: "My application is crashing"
    });
    
    expect(result.research_type).toBe('troubleshooting');
    expect(result.confidence).toBeGreaterThan(0.8);
    expect(result.matched_keywords).toContain('crashing');
  });
  
  test('should detect high urgency correctly', async () => {
    const result = await mcp.tools.detect_context({
      query: "URGENT: Production server down",
      research_type: "troubleshooting"
    });
    
    expect(result.urgency_level).toBe('high');
    expect(result.overall_confidence).toBeGreaterThan(0.7);
  });
  
  test('should provide appropriate responses for beginners', async () => {
    const result = await mcp.tools.research_query({
      query: "What is Rust?",
      query_type: "learning",
      audience: "beginner"
    });
    
    expect(result.result).toContain('Rust is');
    expect(result.result.length).toBeGreaterThan(100);
    expect(result.metadata.context_detection_used).toBe(true);
  });
});
```
</implementation>

### <testing-integration>Integration Testing</testing-integration>

<implementation>
```javascript
describe('MCP Integration Tests', () => {
  test('should handle complete research workflow', async () => {
    const query = "How to implement async functions in Rust?";
    
    // Step 1: Classification
    const classification = await mcp.tools.classify_query({ query });
    
    // Step 2: Context detection
    const context = await mcp.tools.detect_context({
      query,
      research_type: classification.research_type
    });
    
    // Step 3: Research
    const research = await mcp.tools.research_query({
      query,
      query_type: classification.research_type,
      audience: context.audience_level,
      domain: context.technical_domain
    });
    
    // Step 4: Resource access
    const resources = await mcp.resources.read(
      "mcp://fortitude/docs/reference_library/patterns/async-patterns.md"
    );
    
    // Verify complete workflow
    expect(classification.research_type).toBe('implementation');
    expect(context.technical_domain).toBe('rust');
    expect(research.result).toContain('async');
    expect(resources.contents).toHaveLength(1);
  });
});
```
</implementation>

## <performance-examples>Performance Examples</performance-examples>

### <performance-optimization>Performance Optimization</performance-optimization>

<implementation>
```javascript
// Optimized batch processing with connection pooling
class OptimizedMCPClient {
  constructor() {
    this.requestQueue = [];
    this.processing = false;
    this.maxConcurrent = 10;
    this.requestDelay = 50; // ms between requests
  }
  
  async queueRequest(toolName, args) {
    return new Promise((resolve, reject) => {
      this.requestQueue.push({
        toolName,
        args,
        resolve,
        reject,
        timestamp: Date.now()
      });
      
      this.processQueue();
    });
  }
  
  async processQueue() {
    if (this.processing || this.requestQueue.length === 0) {
      return;
    }
    
    this.processing = true;
    
    while (this.requestQueue.length > 0) {
      const batch = this.requestQueue.splice(0, this.maxConcurrent);
      
      const promises = batch.map(async (request) => {
        try {
          const result = await mcp.tools[request.toolName](request.args);
          request.resolve(result);
        } catch (error) {
          request.reject(error);
        }
      });
      
      await Promise.all(promises);
      
      // Brief pause between batches
      await new Promise(resolve => setTimeout(resolve, this.requestDelay));
    }
    
    this.processing = false;
  }
}

// Usage
const client = new OptimizedMCPClient();

const results = await Promise.all([
  client.queueRequest('classify_query', { query: 'Query 1' }),
  client.queueRequest('classify_query', { query: 'Query 2' }),
  client.queueRequest('research_query', { query: 'Query 3', query_type: 'learning' })
]);
```
</implementation>

---

**Usage Examples Complete**: This documentation provides practical examples for common workflows, tool usage patterns, CLI commands, and integration scenarios with the Fortitude MCP Server.