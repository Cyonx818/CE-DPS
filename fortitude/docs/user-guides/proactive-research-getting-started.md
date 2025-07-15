# Getting Started with Proactive Research Mode

<meta>
  <title>Proactive Research Mode - Getting Started Guide</title>
  <type>user_guide</type>
  <audience>developer</audience>
  <complexity>basic</complexity>
  <updated>2025-07-11</updated>
  <sprint>008</sprint>
  <version>1.0</version>
</meta>

## <summary priority="high">Quick Start</summary>
- **Purpose**: Automated knowledge gap detection and background research for proactive learning
- **Key Benefit**: Automatic identification and research of documentation gaps as you code
- **Setup Time**: 5 minutes to configure, immediate value
- **Interfaces**: CLI commands, HTTP API, or Claude Code MCP tools
- **Best For**: Active development projects with evolving codebases

## <overview>What is Proactive Research Mode?</overview>

Proactive Research Mode automatically monitors your codebase for knowledge gaps and conducts background research to fill them. Instead of waiting until you need information, it proactively identifies areas where documentation, examples, or explanations would be valuable and researches them in the background.

### <capabilities>Core Capabilities</capabilities>

**Automatic Gap Detection:**
- TODO comments and FIXME notes
- Undocumented public APIs and functions
- New technology imports without documentation
- Missing configuration documentation
- API usage without examples

**Background Research:**
- Runs research tasks in background without interrupting development
- Prioritizes gaps based on context and importance
- Delivers results through multiple notification channels
- Integrates with existing development workflows

**Multi-Interface Access:**
- **CLI**: Direct command-line control and status monitoring
- **HTTP API**: Programmatic integration with tools and IDEs
- **MCP Tools**: Seamless Claude Code integration for AI-assisted development

## <quick-start>5-Minute Quick Start</quick-start>

### <step>Step 1: Choose Your Interface</step>

**For CLI Users:**
```bash
# Start monitoring your current project
cd /path/to/your/project
fortitude proactive start

# Check status
fortitude proactive status
```

**For Claude Code Users:**
Use the MCP tools directly:
- `proactive_start` - Initialize proactive research
- `proactive_status` - Check system status
- `proactive_list_tasks` - View research tasks

**For API Integration:**
```bash
# Start via HTTP API
curl -X POST "http://localhost:8080/api/v1/proactive/start" \
  -H "X-API-Key: your-api-key" \
  -H "Content-Type: application/json"
```

### <step>Step 2: Verify System is Running</step>

**CLI Verification:**
```bash
fortitude proactive status --detailed
```

**Expected Output:**
```
Proactive Research Status: RUNNING
Uptime: 2 minutes
Active Tasks: 0
Detected Gaps: 3
File Monitoring: HEALTHY
```

**MCP Verification (in Claude Code):**
Call the `proactive_status` tool to see detailed system status.

### <step>Step 3: Test Gap Detection</step>

Create a test file to verify gap detection works:

```rust
// test_gap_detection.rs

// TODO: Add error handling documentation
pub fn example_function() -> Result<(), String> {
    Ok(())
}

// FIXME: This needs proper validation
pub struct Config {
    pub timeout: u64,
}
```

**Verify Detection:**
```bash
# Check if gaps were detected
fortitude proactive status --detailed

# List detected gaps
fortitude proactive tasks list --status pending
```

### <step>Step 4: Configure Notifications</step>

Set up how you want to receive research results:

```bash
# Configure desktop notifications (default)
fortitude proactive configure set notifications.channels "desktop,console"

# Set notification frequency
fortitude proactive configure set notifications.frequency "immediate"
```

**MCP Configuration:**
Use the `proactive_configure` tool with:
```json
{
  "config": {
    "notification_channels": ["desktop", "console"],
    "notification_frequency": "immediate"
  }
}
```

## <configuration>Basic Configuration</configuration>

### <presets>Configuration Presets</presets>

**Development Mode (Recommended for most users):**
```bash
fortitude proactive configure preset development
```
- Frequent gap detection (every 5 minutes)
- Immediate desktop notifications
- Moderate resource usage
- Enhanced debugging information

**Production Mode:**
```bash
fortitude proactive configure preset production
```
- Conservative gap detection (every 30 minutes)
- Rate-limited notifications
- Minimal resource usage
- Focus on critical gaps only

**Research Mode:**
```bash
fortitude proactive configure preset research
```
- Aggressive gap detection (every 2 minutes)
- High concurrency for faster research
- Detailed research output
- All gap types enabled

### <common-settings>Common Settings</common-settings>

**Monitoring Paths:**
```bash
# Add specific directories to monitor
fortitude proactive configure set watch_paths "/path/to/src,/path/to/docs"

# Exclude directories
fortitude proactive configure set ignore_patterns "target,node_modules,.git"
```

**Research Behavior:**
```bash
# Maximum concurrent research tasks
fortitude proactive configure set max_concurrent_tasks 3

# Gap detection interval
fortitude proactive configure set gap_interval_minutes 15

# Auto-execute research (vs manual approval)
fortitude proactive configure set auto_execute true
```

**Notification Settings:**
```bash
# Notification channels
fortitude proactive configure set notification_channels "desktop,console,log"

# Quiet hours (24-hour format)
fortitude proactive configure set quiet_hours.start "22:00"
fortitude proactive configure set quiet_hours.end "08:00"
```

### <file-config>Configuration Files</file-config>

**Project-Specific Config (`.fortitude_config.json`):**
```json
{
  "proactive": {
    "gap_analysis": {
      "interval_minutes": 10,
      "enabled_types": ["todo_comment", "missing_docs", "api_gaps"]
    },
    "notifications": {
      "channels": ["desktop"],
      "frequency": "batched",
      "quiet_hours": {
        "enabled": true,
        "start": "22:00",
        "end": "08:00"
      }
    },
    "workspace": {
      "watch_paths": ["src/", "docs/"],
      "ignore_patterns": ["target/", "*.tmp"]
    }
  }
}
```

**User Config (`~/.fortitude/config.toml`):**
```toml
[proactive]
max_concurrent_tasks = 5
auto_execute = false

[proactive.notifications]
channels = ["desktop", "console"]
frequency = "immediate"

[proactive.performance]
max_memory_mb = 512
max_cpu_percent = 10.0
```

## <workflow>Daily Workflow</workflow>

### <routine>Morning Routine</routine>

1. **Start Proactive Research:**
```bash
cd /path/to/project
fortitude proactive start
```

2. **Review Overnight Research:**
```bash
# Check completed research from yesterday
fortitude proactive notifications --since 24h --unread

# Review any pending tasks
fortitude proactive tasks list --status completed --limit 10
```

### <routine>During Development</routine>

**Proactive research runs automatically in the background. You'll receive notifications when:**
- New knowledge gaps are detected in your code
- Research tasks complete with useful findings
- Important patterns or best practices are discovered

**Monitor Progress:**
```bash
# Quick status check
fortitude proactive status

# Detailed view with recent activity
fortitude proactive status --detailed --recent-minutes 60
```

### <routine>End of Day</routine>

```bash
# Review day's research
fortitude proactive notifications --since 8h

# Stop proactive research (optional)
fortitude proactive stop --timeout 30
```

## <notifications>Understanding Notifications</notifications>

### <notification-types>Notification Types</notification-types>

**Gap Detected:**
```
📋 Knowledge Gap Detected
New TODO comment found: "Add error handling documentation"
Location: src/main.rs:45
Priority: Medium
Research Task: Created #task_123
```

**Research Completed:**
```
✅ Research Task Completed
Task: Error handling documentation for Result types
Duration: 2 minutes
Results: Found 3 patterns and 5 examples
View: fortitude proactive tasks show task_123
```

**System Alerts:**
```
⚠️ Proactive Research Alert
High CPU usage detected: 15% (limit: 10%)
Action: Reducing concurrent tasks from 5 to 3
```

### <notification-channels>Notification Channels</notification-channels>

**Desktop Notifications:**
- Native OS notifications
- Click to view details
- Configurable frequency and quiet hours

**Console Output:**
- Formatted terminal output
- Integrated with CLI commands
- Suitable for development workflow

**Log Files:**
- Structured JSON logs
- Integration with monitoring systems
- Full audit trail

**Webhook (Advanced):**
- HTTP POST to custom endpoints
- Integration with Slack, Discord, etc.
- Custom payload formatting

## <integration>IDE and Tool Integration</integration>

### <claude-code>Claude Code Integration</claude-code>

**Proactive Research MCP Tools:**
- `proactive_start` - Start monitoring your project
- `proactive_status` - Check system status and recent activity
- `proactive_list_tasks` - View research tasks and progress
- `proactive_configure` - Adjust settings for your workflow
- `proactive_get_notifications` - Review research findings

**Example Claude Code Workflow:**
1. Open your project in Claude Code
2. Use `proactive_start` tool to begin monitoring
3. Continue coding normally
4. Use `proactive_status` to check for new research
5. Use `proactive_get_notifications` to review findings

### <api-integration>API Integration</api-integration>

**For IDEs and Custom Tools:**
```javascript
// JavaScript example
const fortitudeClient = new FortitudeAPI('http://localhost:8080', 'your-api-key');

// Start proactive research
await fortitudeClient.proactive.start({
  base_directory: workspace.rootPath,
  max_concurrent_tasks: 3
});

// Get notifications for IDE display
const notifications = await fortitudeClient.proactive.getNotifications({
  unread_only: true,
  limit: 10
});
```

## <troubleshooting>Common Issues</troubleshooting>

### <issue>Proactive Research Won't Start</issue>

**Symptoms:** Error when running `fortitude proactive start`

**Solutions:**
1. **Check Configuration:**
```bash
fortitude proactive configure show
```

2. **Verify Permissions:**
```bash
# Ensure write access to config directory
ls -la ~/.fortitude/
```

3. **Check Resource Availability:**
```bash
# Verify system resources
fortitude proactive status --system-check
```

### <issue>No Gaps Being Detected</issue>

**Symptoms:** System running but no gaps found

**Solutions:**
1. **Verify File Monitoring:**
```bash
# Check if files are being monitored
fortitude proactive status --file-monitor
```

2. **Test with Known Gap:**
Add a TODO comment to a file and wait 1-2 minutes

3. **Check Gap Detection Settings:**
```bash
# Ensure gap types are enabled
fortitude proactive configure show gap_analysis.enabled_types
```

### <issue>Too Many Notifications</issue>

**Symptoms:** Overwhelming number of notifications

**Solutions:**
1. **Adjust Notification Frequency:**
```bash
fortitude proactive configure set notification_frequency "batched"
```

2. **Filter by Priority:**
```bash
fortitude proactive configure set min_notification_priority "medium"
```

3. **Enable Quiet Hours:**
```bash
fortitude proactive configure set quiet_hours.enabled true
fortitude proactive configure set quiet_hours.start "18:00"
fortitude proactive configure set quiet_hours.end "09:00"
```

### <issue>High Resource Usage</issue>

**Symptoms:** Slow system performance

**Solutions:**
1. **Reduce Concurrent Tasks:**
```bash
fortitude proactive configure set max_concurrent_tasks 2
```

2. **Increase Detection Interval:**
```bash
fortitude proactive configure set gap_interval_minutes 30
```

3. **Set Resource Limits:**
```bash
fortitude proactive configure set max_cpu_percent 5.0
fortitude proactive configure set max_memory_mb 256
```

## <next-steps>Next Steps</next-steps>

### <learning>Learn More</learning>
- [Configuration Guide](proactive-research-configuration.md) - Comprehensive configuration options
- [CLI Reference](proactive-research-cli.md) - Complete command reference
- [API Guide](proactive-research-api.md) - HTTP API documentation
- [MCP Tools Guide](proactive-research-mcp.md) - Claude Code integration
- [Troubleshooting Guide](proactive-research-troubleshooting.md) - Common issues and solutions

### <advanced>Advanced Usage</advanced>
- [Workflow Examples](proactive-research-workflows.md) - Team and project workflows
- [Performance Tuning](proactive-research-performance.md) - Optimization guide
- [Custom Integration](proactive-research-integration.md) - Building custom integrations

### <feedback>Get Help</feedback>
- **Documentation Issues**: Check the troubleshooting guide
- **Feature Requests**: See the roadmap for planned features
- **Community**: Join discussions about best practices and workflows

---

**Ready to enhance your development workflow with proactive research? Start with the 5-minute quick start above and gradually customize the system to match your development style.**