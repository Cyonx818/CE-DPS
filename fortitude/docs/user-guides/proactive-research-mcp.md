# Proactive Research MCP Tools Guide

<meta>
  <title>Proactive Research MCP Tools - Claude Code Integration</title>
  <type>user_guide</type>
  <audience>developer</audience>
  <complexity>basic_to_intermediate</complexity>
  <updated>2025-07-11</updated>
  <sprint>008</sprint>
  <version>1.0</version>
</meta>

## <summary priority="high">MCP Tools Overview</summary>
- **Purpose**: Native integration with Claude Code for seamless proactive research
- **MCP Protocol**: Direct communication with Claude Code interface
- **Available Tools**: Start, status, configure, tasks, notifications management
- **Real-time Integration**: Live updates and notifications within Claude Code
- **Zero Configuration**: Works out-of-the-box with Claude Code installations

## <overview>What is MCP Integration?</overview>

The Model Context Protocol (MCP) integration provides native Claude Code tools for proactive research management. This allows you to control and monitor the proactive research system directly from within Claude Code without switching to CLI or API interfaces.

### <benefits>Key Benefits</benefits>

- **Seamless Workflow**: Manage proactive research without leaving Claude Code
- **Context Awareness**: Claude Code can automatically access research results
- **Real-time Updates**: Receive notifications and status updates in the interface
- **Intelligent Integration**: Claude can use research results to improve code suggestions
- **Zero Context Switching**: All functionality available within your development environment

### <architecture>MCP Architecture</architecture>

```
Claude Code ←→ MCP Protocol ←→ Fortitude MCP Server ←→ Proactive Manager
    ↓                                                        ↓
User Interface                                    Background Research
Notifications                                     Gap Detection
Task Management                                   File Monitoring
```

## <setup>Setup and Installation</setup>

### <automatic-setup>Automatic Setup (Recommended)</automatic-setup>

Claude Code automatically detects and configures Fortitude MCP tools when the system is installed.

**Verification:**
1. Open Claude Code
2. Type: "Show available MCP tools"
3. Look for proactive research tools in the list
4. Use any proactive tool to test connectivity

### <manual-setup>Manual Setup</manual-setup>

If automatic detection fails, configure manually:

**1. Check Fortitude Installation:**
```bash
# Verify fortitude is installed and accessible
which fortitude
fortitude --version
```

**2. Start MCP Server:**
```bash
# Start the MCP server (usually automatic)
fortitude mcp-server start --port 3000

# Check MCP server status
fortitude mcp-server status
```

**3. Configure Claude Code:**
Add to Claude Code configuration if needed:
```json
{
  "mcp_servers": {
    "fortitude": {
      "command": "fortitude",
      "args": ["mcp-server", "start"],
      "env": {
        "FORTITUDE_MCP_ENABLED": "true"
      }
    }
  }
}
```

### <troubleshooting-setup>Setup Troubleshooting</troubleshooting-setup>

**Issue: Tools not appearing in Claude Code**
```bash
# Check MCP server logs
fortitude mcp-server logs --tail 50

# Restart MCP server
fortitude mcp-server restart

# Test MCP connectivity
fortitude mcp-server test-connection
```

**Issue: Permission errors**
```bash
# Check configuration directory permissions
ls -la ~/.fortitude/

# Fix permissions if needed
chmod 755 ~/.fortitude/
chmod 644 ~/.fortitude/config.toml
```

## <available-tools>Available MCP Tools</available-tools>

### <proactive-start>proactive_start</proactive-start>

Start proactive research monitoring for the current project.

**Usage in Claude Code:**
> "Start proactive research for this project"

**Tool Parameters:**
```json
{
  "base_directory": "/path/to/project",
  "gap_interval_seconds": 300,
  "max_concurrent_tasks": 3,
  "file_patterns": ["*.rs", "*.md", "*.toml"],
  "notification_channels": ["claude_code"],
  "auto_start": true
}
```

**Example Conversations:**
```
User: "Start monitoring this Rust project for knowledge gaps"

Claude: I'll start proactive research monitoring for your Rust project. Let me configure it with appropriate settings.

[Uses proactive_start tool]

✅ Proactive research started successfully!

Configuration:
- Gap detection interval: 5 minutes
- Monitoring file types: Rust (.rs), Markdown (.md), TOML (.toml)
- Maximum concurrent research tasks: 3
- Notifications: Enabled in Claude Code

The system is now monitoring your project for:
- TODO/FIXME comments
- Undocumented functions
- Missing error handling
- API usage gaps

You'll receive notifications here in Claude Code when gaps are detected and research is completed.
```

**Advanced Configuration:**
```
User: "Start proactive research but only monitor the src directory and check every 2 minutes"

Claude: I'll start proactive research with your custom settings.

[Uses proactive_start with custom parameters]

✅ Started with custom configuration:
- Monitoring: src/ directory only
- Check interval: 2 minutes (more frequent)
- File types: All supported types in src/
- Priority threshold: Medium and above

This frequent monitoring is good for active development sessions.
```

### <proactive-status>proactive_status</proactive-status>

Check the current status of the proactive research system.

**Usage in Claude Code:**
> "What's the status of proactive research?"
> "Show me recent research activity"

**Tool Parameters:**
```json
{
  "detailed": true,
  "metrics": true,
  "recent_minutes": 60,
  "include_performance": true
}
```

**Example Conversations:**
```
User: "How's the proactive research system doing?"

Claude: Let me check the current status of your proactive research system.

[Uses proactive_status tool]

📊 **Proactive Research Status**

**System Status:** 🟢 Running (3h 42m uptime)
- Session ID: session_abc123
- Started: Today at 11:15 AM

**Gap Detection:**
- Last scan: 2 minutes ago
- Files monitored: 247 files
- Gaps detected today: 8
- Next scan: In 3 minutes

**Background Research:**
- Active tasks: 2/3
- Completed today: 15 tasks
- Success rate: 94%
- Queue: 1 pending task

**Recent Activity:**
- 2m ago: Gap detected in src/main.rs (TODO comment)
- 5m ago: Research completed for "Error handling patterns"
- 8m ago: Started research on "Async best practices"

**Performance:**
- CPU: 12% usage
- Memory: 456 MB / 2048 MB (22%)
- Status: Normal operation

Everything looks healthy! The system is actively monitoring your code and conducting research in the background.
```

**Brief Status Check:**
```
User: "Quick status check"

Claude: [Uses proactive_status with brief parameters]

🟢 **Proactive Research:** Running normally
- 2 active research tasks
- 8 gaps detected today  
- Last activity: 2 minutes ago
- Performance: Normal (12% CPU, 456 MB RAM)

All systems operational!
```

### <proactive-configure>proactive_configure</proactive-configure>

Configure proactive research settings and preferences.

**Usage in Claude Code:**
> "Change the gap detection interval to 10 minutes"
> "Enable email notifications"
> "Set up development mode configuration"

**Tool Parameters:**
```json
{
  "action": "set",
  "key": "gap_analysis.scan_intervals_seconds",
  "value": "600",
  "validate": true
}
```

**Example Conversations:**
```
User: "I want to reduce the scanning frequency to every 10 minutes to save CPU"

Claude: I'll adjust the gap detection interval to 10 minutes for you.

[Uses proactive_configure tool]

✅ **Configuration Updated**

Changed gap detection interval:
- From: 5 minutes (300 seconds)
- To: 10 minutes (600 seconds)

This will reduce CPU usage while still providing regular gap detection. The system validated this change and confirmed it's compatible with your other settings.

Your next scan will occur in 10 minutes.
```

**Preset Configuration:**
```
User: "Set up the system for production environment"

Claude: I'll configure the system with production-optimized settings.

[Uses proactive_configure with production preset]

✅ **Applied Production Preset**

Key changes made:
- Gap detection: Every 10 minutes (conservative)
- Concurrent tasks: Reduced to 2 (resource efficient)
- CPU limit: 70% (production safe)
- Notifications: Email + webhook (no desktop)
- Rate limiting: 30 requests/minute (conservative)
- Monitoring: Full monitoring enabled

These settings are optimized for production environments with emphasis on resource conservation and stability.
```

**Notification Setup:**
```
User: "Enable slack notifications for high priority gaps only"

Claude: I'll configure Slack notifications for high priority gaps.

[Uses proactive_configure for notification settings]

✅ **Notification Configuration Updated**

Slack notifications enabled:
- Priority filter: High and Critical only
- Channel: Will use configured Slack webhook
- Rate limit: Max 5 notifications per hour
- Quiet hours: 10 PM - 8 AM (if configured)

You'll now receive Slack notifications for important gaps that need immediate attention, while avoiding notification spam for minor issues.
```

### <proactive-list-tasks>proactive_list_tasks</proactive-list-tasks>

List and manage research tasks.

**Usage in Claude Code:**
> "Show me current research tasks"
> "What research has been completed recently?"
> "List high priority tasks"

**Tool Parameters:**
```json
{
  "status": "active",
  "priority": "high",
  "limit": 10,
  "include_details": true,
  "since": "2025-07-11T12:00:00Z"
}
```

**Example Conversations:**
```
User: "What research tasks are currently running?"

Claude: Let me check the current research tasks for you.

[Uses proactive_list_tasks with status="active"]

🧠 **Active Research Tasks (2)**

1. **Task #789** - Error handling patterns in Rust
   - Priority: High
   - Progress: 85% complete
   - Started: 12 minutes ago
   - Source: TODO comment in src/main.rs:45
   - Estimated completion: 2 minutes

2. **Task #790** - React state management best practices  
   - Priority: Medium
   - Progress: 22% complete
   - Started: 3 minutes ago
   - Source: FIXME comment in components/Header.jsx:112
   - Estimated completion: 8 minutes

**Pending Tasks (1)**
- API documentation standards (Medium priority)

The system is actively researching error handling patterns and should complete soon. I can show you the results when they're ready!
```

**Recent Completions:**
```
User: "Show me what research was completed today"

Claude: [Uses proactive_list_tasks with status="completed" and since="today"]

✅ **Research Completed Today (15 tasks)**

**Recent Completions:**
1. **Rust error handling patterns** (12m ago)
   - Duration: 3m 24s | Confidence: 89%
   - Found: Result<T,E> patterns, ? operator usage, custom errors

2. **CSS Grid layout techniques** (28m ago)  
   - Duration: 2m 15s | Confidence: 92%
   - Found: Modern grid patterns, responsive design, browser support

3. **Database migration strategies** (45m ago)
   - Duration: 4m 31s | Confidence: 87%
   - Found: Schema versioning, rollback strategies, testing approaches

4. **Testing best practices for React** (1h ago)
   - Duration: 2m 52s | Confidence: 91%
   - Found: Jest patterns, component testing, mocking strategies

**Success Rate Today:** 94% (14/15 successful)

Would you like me to show detailed results for any of these research topics?
```

### <proactive-get-notifications>proactive_get_notifications</proactive-get-notifications>

Retrieve and manage notifications.

**Usage in Claude Code:**
> "Show me recent notifications"
> "Any new research results?"
> "Mark all notifications as read"

**Tool Parameters:**
```json
{
  "unread_only": true,
  "limit": 20,
  "since": "2025-07-11T12:00:00Z",
  "mark_as_read": false
}
```

**Example Conversations:**
```
User: "Any new research notifications?"

Claude: Let me check for recent research notifications.

[Uses proactive_get_notifications with unread_only=true]

📬 **New Notifications (3)**

1. **Gap Detected** (5 minutes ago)
   - Priority: Medium
   - Location: src/components/Button.tsx:89
   - Description: "TODO: Add accessibility attributes"
   - Research task created: #task_793

2. **Research Completed** (8 minutes ago)
   - Task: "Error handling patterns in Rust"
   - Duration: 3m 24s
   - Results: Found 5 patterns and 8 code examples
   - Confidence: 89%

3. **System Alert** (15 minutes ago)
   - Type: Performance notice
   - Message: "High research activity detected"
   - Action: Automatically reduced scan frequency

Would you like me to show the detailed research results for the error handling patterns, or create a task for the accessibility gap?
```

**Notification Management:**
```
User: "Mark all notifications as read and show me a summary"

Claude: [Uses proactive_get_notifications with mark_as_read=true]

✅ **Marked 12 notifications as read**

**Today's Summary:**
- 8 gap detections (5 TODO, 2 FIXME, 1 undocumented)
- 4 research completions (all successful)
- 2 system alerts (performance optimization)

**Most Important Items:**
- High priority gap in security module needs attention
- Async/await research completed with excellent results
- System auto-optimized performance during high activity

All notifications cleared! The system will continue monitoring and notify you of new activity.
```

### <proactive-stop>proactive_stop</proactive-stop>

Stop the proactive research system gracefully.

**Usage in Claude Code:**
> "Stop proactive research"
> "Shut down the monitoring system"

**Tool Parameters:**
```json
{
  "timeout_seconds": 30,
  "save_state": true,
  "force": false
}
```

**Example Conversations:**
```
User: "I'm done for the day, please stop the proactive research system"

Claude: I'll gracefully stop the proactive research system for you.

[Uses proactive_stop tool]

🛑 **Proactive Research System Stopped**

Shutdown process completed:
✅ Gap detection stopped
✅ Background research tasks completed (2 active tasks finished)
✅ File monitoring stopped  
✅ Current state saved

**Final Session Statistics:**
- Uptime: 4 hours 23 minutes
- Gaps detected: 14
- Research tasks completed: 18
- Success rate: 94%
- Notifications sent: 12

The system is now stopped and your progress has been saved. You can restart it anytime with "start proactive research" when you resume development.
```

## <workflows>Common MCP Workflows</workflows>

### <development-session>Development Session Workflow</development-session>

**Starting Your Development Session:**
```
User: "I'm starting work on the user authentication module"

Claude: I'll help you set up proactive research for your authentication work.

[Uses proactive_start with auth-focused configuration]

✅ **Development Session Started**

Proactive research configured for authentication development:
- Monitoring: Enhanced focus on security patterns
- Priority keywords: "auth", "security", "validation", "encryption" 
- Scan frequency: Every 2 minutes (active development)
- File focus: Authentication-related files prioritized

I'll watch for gaps in security best practices, authentication patterns, and compliance requirements as you code. Let me know if you need any specific research on authentication topics!
```

**During Development:**
```
[Automatic notification via MCP]

Claude: 🔍 **Gap Detected**

I noticed a TODO comment in your authentication code:
- File: `src/auth/validator.rs:67`
- Comment: "TODO: Add rate limiting for login attempts"
- Priority: High (security-related)

I've started research on "rate limiting patterns for authentication" and "login attempt security measures." Results should be ready in about 3 minutes.

Would you like me to prioritize this research or continue with your current work?
```

**Research Results Integration:**
```
[Automatic research completion notification]

Claude: ✅ **Research Completed: Rate Limiting for Authentication**

I found comprehensive patterns for login rate limiting:

**Key Recommendations:**
1. **Sliding Window Rate Limiting**: Most effective for login attempts
2. **Progressive Delays**: Increase delay with failed attempts  
3. **IP + User Account Tracking**: Track both vectors
4. **Redis Implementation**: Fast, scalable storage for counters

**Code Example:**
```rust
pub struct RateLimiter {
    redis: redis::Client,
    max_attempts: u32,
    window_seconds: u64,
}

impl RateLimiter {
    pub async fn check_rate_limit(&self, identifier: &str) -> Result<bool, RateLimitError> {
        let key = format!("rate_limit:{}", identifier);
        let current_count: u32 = self.redis.incr(&key, 1).await?;
        
        if current_count == 1 {
            self.redis.expire(&key, self.window_seconds).await?;
        }
        
        Ok(current_count <= self.max_attempts)
    }
}
```

**Additional Resources:**
- OWASP Authentication Best Practices
- Redis rate limiting patterns
- Industry-standard delay algorithms

Would you like me to help implement this pattern in your authentication module?
```

### <code-review-workflow>Code Review Workflow</code-review-workflow>

**Pre-Review Setup:**
```
User: "I'm about to submit a PR for the payment processing module. Can you help me check for any knowledge gaps?"

Claude: I'll run a focused analysis on your payment processing module to identify any potential gaps before your PR.

[Uses proactive_start with PR review configuration]

✅ **PR Review Analysis Started**

Focused on payment processing:
- Security gap detection: Enhanced
- Documentation coverage: Strict checking
- Error handling: Critical priority
- Test coverage gaps: Enabled
- Compliance patterns: Financial regulations

I'll scan your recent changes and identify any areas that might need documentation, testing, or security review before submission.
```

**Gap Analysis Results:**
```
[After analysis completion]

Claude: 📋 **PR Review Analysis Complete**

I found 3 areas that should be addressed before submitting your PR:

**Critical Issues (1):**
1. **Missing Error Handling** in `payment_processor.rs:134`
   - Function `process_refund` lacks error handling for failed transactions
   - Research completed: Found industry patterns for payment error handling
   - Recommendation: Implement comprehensive error types with user-safe messages

**Important Improvements (2):**
2. **API Documentation Gap** in `payment_api.rs:45-67`
   - Public functions lack documentation for payment flow
   - Found: OpenAPI specification patterns for payment APIs

3. **Test Coverage** in refund processing logic
   - Edge cases not covered: partial refunds, failed reversals
   - Research: Payment testing strategies and mock patterns

**Generated Checklist:**
- [ ] Add error handling to refund processing
- [ ] Document payment API endpoints  
- [ ] Add test cases for refund edge cases
- [ ] Review security patterns for payment data

Would you like detailed implementation guidance for any of these items?
```

### <learning-workflow>Learning and Exploration Workflow</learning-workflow>

**Topic Exploration:**
```
User: "I'm learning about WebAssembly integration with Rust. Can you help me discover knowledge gaps as I explore?"

Claude: I'll set up proactive research to support your WebAssembly learning journey.

[Uses proactive_configure with learning mode]

✅ **Learning Mode Activated: WebAssembly + Rust**

Configuration optimized for learning:
- Keywords prioritized: "wasm", "webassembly", "wasm-bindgen", "js-sys"
- Research depth: Enhanced (more sources, detailed explanations)
- Confidence threshold: Lowered (catch more potential learning opportunities)
- Notification frequency: Immediate (faster feedback)

I'll actively research any WebAssembly patterns you encounter and provide educational context. Try creating some WASM code and I'll help identify learning opportunities!
```

**Learning Discovery:**
```
[While coding WebAssembly examples]

Claude: 💡 **Learning Opportunity Detected**

I noticed you're working with `wasm-bindgen` in your code. I've found some knowledge gaps that might enhance your learning:

**Current Code Context:**
```rust
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}
```

**Learning Opportunities:**
1. **Memory Management**: WASM linear memory vs Rust ownership
2. **Type Conversions**: Advanced `wasm-bindgen` type mappings
3. **Performance Patterns**: Optimizing Rust for WASM size and speed
4. **Debugging Techniques**: WASM debugging in browser dev tools

**Research Started:**
- "WASM memory management patterns with Rust"
- "Advanced wasm-bindgen usage patterns"  
- "WASM performance optimization techniques"

I'll have detailed explanations and examples ready in a few minutes. Want to continue coding or explore these concepts now?
```

## <integration-patterns>Advanced Integration Patterns</integration-patterns>

### <contextual-research>Contextual Research Integration</contextual-research>

Claude Code can automatically incorporate research results into its responses:

```
User: "Help me implement error handling for this function"

Claude: I notice you're working on error handling, and I just completed research on this topic! Let me combine my analysis of your code with the latest research findings.

[Automatically accesses recent research results]

Based on your code and current Rust error handling best practices I researched:

**For your specific function:**
```rust
// Your current code:
fn process_data(input: &str) -> String {
    // TODO: Add error handling
    input.to_uppercase()
}

// Recommended with error handling:
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataProcessingError {
    #[error("Invalid input format: {0}")]
    InvalidFormat(String),
    #[error("Processing failed: {0}")]
    ProcessingFailed(String),
}

fn process_data(input: &str) -> Result<String, DataProcessingError> {
    if input.is_empty() {
        return Err(DataProcessingError::InvalidFormat("Empty input".to_string()));
    }
    
    // Add your processing logic here
    Ok(input.to_uppercase())
}
```

This pattern follows the research findings I just completed on Rust error handling, using:
- Custom error types with `thiserror` for better error messages
- Result<T, E> return type for explicit error handling
- Descriptive error variants for different failure modes

Would you like me to help adapt this pattern to your specific use case?
```

### <real-time-collaboration>Real-time Research Collaboration</real-time-collaboration>

```
User: "I'm struggling with async Rust performance in this function"

Claude: I can see you're working on async performance. Let me start some targeted research while we look at your code together.

[Uses proactive_list_tasks to check existing research]
[Uses proactive_create_task for immediate research]

I've started immediate research on "async Rust performance optimization" with high priority. While that runs, let me analyze your current code...

[3 minutes later, automatic research completion]

Perfect timing! My research just completed with some highly relevant findings for your async performance issue:

**Research Results Applied to Your Code:**
1. **Async Function Overhead**: Your function creates unnecessary futures
2. **Blocking Operations**: I found blocking calls in your async context
3. **Concurrency Patterns**: Research suggests better patterns for your use case

**Specific Recommendations:**
```rust
// Instead of this (blocking in async):
async fn your_function() -> Result<String, Error> {
    let result = expensive_blocking_operation(); // ❌ Blocks async runtime
    process_async(result).await
}

// Use this (proper async pattern):
async fn your_function() -> Result<String, Error> {
    let result = tokio::task::spawn_blocking(|| {
        expensive_blocking_operation() // ✅ Proper blocking task
    }).await?;
    process_async(result).await
}
```

This research-backed approach will significantly improve your async performance. Want me to help implement these patterns in your specific code?
```

### <project-specific-intelligence>Project-Specific Intelligence</project-specific-intelligence>

The MCP integration learns from your project patterns:

```
User: "Set up proactive research for this new microservice"

Claude: I'll analyze your existing project structure to configure optimal proactive research for your new microservice.

[Uses proactive_configure with project analysis]

✅ **Project-Specific Configuration Applied**

Based on your existing microservices, I've configured:

**Detected Project Patterns:**
- Architecture: Event-driven microservices with Kafka
- Database: PostgreSQL with async-rust (sqlx)
- API: axum with OpenAPI documentation
- Testing: Integration tests with testcontainers
- Monitoring: Structured logging with tracing

**Optimized Research Focus:**
- Event sourcing patterns and best practices
- Database migration strategies for microservices  
- API versioning and backward compatibility
- Distributed tracing and observability
- Error handling across service boundaries

**Custom Gap Detection Rules:**
- Undocumented API endpoints (your team emphasizes API docs)
- Missing database transactions (pattern in your other services)  
- Event handling without dead letter queues
- Services without health check endpoints

This configuration matches the patterns I see in your other microservices and will help maintain consistency across your architecture.
```

## <troubleshooting-mcp>MCP Troubleshooting</troubleshooting-mcp>

### <common-mcp-issues>Common MCP Issues</common-mcp-issues>

**Issue 1: Tools Not Available**
```
User: "The proactive research tools aren't showing up in Claude Code"

Claude: Let me help troubleshoot the MCP tool availability issue.

This usually happens when:
1. **MCP Server Not Running**: The Fortitude MCP server may not be started
2. **Connection Issues**: Claude Code can't connect to the MCP server
3. **Configuration Problems**: MCP server configuration is incorrect

**Quick Diagnostics:**
- Check if `fortitude mcp-server status` shows "running"
- Verify Claude Code can access the MCP server port
- Look for error messages in Claude Code developer console

**Resolution Steps:**
1. Restart MCP server: `fortitude mcp-server restart`
2. Check logs: `fortitude mcp-server logs --tail 20`
3. Test connection: `fortitude mcp-server test-connection`

Would you like me to walk through these steps, or do you see any error messages I can help interpret?
```

**Issue 2: Slow Response Times**
```
User: "The proactive research tools are very slow to respond"

Claude: Slow MCP tool responses can indicate several issues:

**Common Causes:**
1. **High System Load**: Proactive research using too many resources
2. **Network Latency**: MCP server communication delays  
3. **Large Project Size**: Scanning many files takes time
4. **Database Performance**: Vector database queries running slowly

**Performance Optimization:**
- Reduce scan frequency: Increase gap detection interval
- Limit file monitoring: Focus on important directories only
- Adjust concurrency: Reduce max concurrent research tasks
- Enable performance mode: Use optimized settings

**Quick Fixes:**
```bash
# Reduce resource usage
fortitude proactive configure set gap_analysis.scan_intervals_seconds 600
fortitude proactive configure set background_research.max_concurrent_tasks 2
fortitude proactive configure set performance.resource_limits.max_cpu_percent 50
```

Would you like me to apply these optimizations or check your current performance metrics first?
```

**Issue 3: Authentication Errors**
```
User: "I'm getting authentication errors when using the MCP tools"

Claude: Authentication errors with MCP tools typically relate to API key configuration:

**Common Solutions:**

1. **Check API Key Configuration:**
   ```bash
   fortitude proactive configure show api.authentication
   ```

2. **Regenerate API Key:**
   ```bash
   fortitude proactive configure generate-api-key --name "Claude Code MCP"
   ```

3. **Update MCP Server Configuration:**
   ```bash
   fortitude mcp-server configure --api-key-file ~/.fortitude/mcp-api-key
   ```

4. **Restart MCP Server:**
   ```bash
   fortitude mcp-server restart
   ```

The MCP server needs proper API credentials to communicate with the proactive research system. Let me help you reconfigure authentication properly.
```

### <debug-mode>Debug Mode for MCP</debug-mode>

```
User: "Enable debug mode so I can see what's happening with the MCP tools"

Claude: I'll help you enable debug mode for better visibility into MCP tool operations.

**Enable MCP Debug Logging:**
```bash
# Enable debug mode
fortitude mcp-server configure set debug_enabled true
fortitude mcp-server configure set log_level debug

# Restart with debug logging
fortitude mcp-server restart --debug

# Follow debug logs
fortitude mcp-server logs --follow --level debug
```

**Claude Code Debug:**
Enable Claude Code developer console to see MCP communication:
1. Open Claude Code settings
2. Enable "Developer Mode" 
3. Open developer console (Ctrl+Shift+I)
4. Look for MCP-related messages in console

**Verbose Tool Execution:**
When using MCP tools, I can now provide detailed execution information:

[Uses proactive_status with debug information]

**Debug Information:**
- MCP call initiated at: 14:30:15.123
- Tool parameters: {"detailed": true, "metrics": true}
- MCP server response time: 245ms
- Data received: 2.3KB
- Processing time: 12ms

This debug information helps identify where delays or issues occur in the MCP communication chain.
```

## <best-practices>MCP Integration Best Practices</best-practices>

### <efficient-usage>Efficient MCP Usage</efficient-usage>

**1. Batch Operations When Possible**
```
User: "Show me status, recent tasks, and notifications"

Claude: I'll get all that information efficiently with a single batch of MCP calls.

[Uses multiple MCP tools in parallel for efficiency]

Here's your complete dashboard:

📊 **System Status**: Running (3h 42m uptime)
🧠 **Active Tasks**: 2 research tasks in progress  
📬 **New Notifications**: 3 unread items
⚡ **Performance**: Normal (12% CPU, 456 MB RAM)

This batch approach is more efficient than making separate requests for each piece of information.
```

**2. Use Appropriate Detail Levels**
```
// Efficient for quick checks
User: "Quick status check"
Claude: [Uses proactive_status with minimal parameters]

// Detailed when needed
User: "I need a full system report"  
Claude: [Uses proactive_status with full detail and metrics]
```

**3. Configure Notifications Appropriately**
```
User: "I don't want to be interrupted during deep focus work"

Claude: I'll configure the system to respect your focus time.

[Uses proactive_configure for focus-friendly settings]

✅ **Focus Mode Configured**
- Notifications: Batched (delivered every 30 minutes)
- Priority filter: High and Critical only
- Quiet hours: Extended during your focus blocks
- Background research: Continues but doesn't interrupt

You'll still get important updates, but they won't break your concentration.
```

### <project-integration>Project Integration Patterns</project-integration>

**1. Context-Aware Configuration**
```
User: "I'm switching from frontend work to backend API development"

Claude: I'll adjust the proactive research configuration for backend API focus.

[Uses proactive_configure with backend-specific settings]

✅ **Switched to Backend API Configuration**

Updated focus areas:
- File priorities: .rs, .sql, .toml files prioritized
- Gap detection: API documentation, error handling, security
- Research keywords: "api", "security", "performance", "database"
- Monitoring: Enhanced for backend patterns

The system will now focus on backend development patterns and API best practices rather than frontend concerns.
```

**2. Team Collaboration Support**
```
User: "My team lead wants to see our proactive research metrics"

Claude: I'll prepare a team-friendly summary of your proactive research activity.

[Uses proactive_list_tasks and proactive_get_notifications with team metrics]

📈 **Team Metrics Summary**

**This Week's Activity:**
- Knowledge gaps identified: 23
- Research tasks completed: 19 (83% success rate)
- Documentation gaps found: 8
- Code quality improvements suggested: 15

**Impact Areas:**
- Security: 5 important gaps addressed
- Performance: 3 optimization opportunities identified  
- Documentation: 8 missing API docs flagged
- Testing: 4 test coverage gaps found

**Time Saved:**
- Estimated research time: 12 hours automated
- Background processing: No development interruption
- Knowledge discovery: Proactive vs reactive approach

This shows how proactive research is improving code quality and team knowledge without disrupting development workflow.
```

### <customization>Customization for Different Development Styles</customization>

**1. Minimalist Integration**
```
User: "I prefer minimal interruptions - just show critical issues"

Claude: I'll configure a minimalist integration that only surfaces critical items.

[Uses proactive_configure with minimalist settings]

✅ **Minimalist Mode Activated**

You'll only receive notifications for:
- Critical security gaps
- High-priority TODO items marked as urgent
- Failed research tasks needing attention
- System errors requiring intervention

All other research continues in the background and can be accessed when you actively request updates.
```

**2. Power User Integration**
```
User: "I want full visibility and control over the proactive research system"

Claude: I'll set up power user mode with full visibility and control options.

[Uses proactive_configure with power user settings]

✅ **Power User Mode Activated**

Enhanced capabilities enabled:
- Real-time research progress updates
- Detailed performance metrics always visible
- Advanced configuration options accessible
- Research task queue management
- Custom gap detection rules
- Full notification history and analytics

You now have complete control and visibility into all aspects of the proactive research system.
```

---

**Next Steps:**
- [Configuration Guide](proactive-research-configuration.md) - Detailed configuration options
- [CLI Reference](proactive-research-cli.md) - Command-line interface
- [API Guide](proactive-research-api.md) - HTTP API integration  
- [Troubleshooting Guide](proactive-research-troubleshooting.md) - Common issues and solutions