# Proactive Research CLI Guide

<meta>
  <title>Proactive Research CLI - Complete Command Reference</title>
  <type>user_guide</type>
  <audience>developer</audience>
  <complexity>basic_to_advanced</complexity>
  <updated>2025-07-11</updated>
  <sprint>008</sprint>
  <version>1.0</version>
</meta>

## <summary priority="high">CLI Command Overview</summary>
- **Main Command**: `fortitude proactive` - All proactive research operations
- **Core Subcommands**: `start`, `stop`, `status`, `configure`, `tasks`, `notifications`
- **Global Options**: Configuration files, verbosity, data directories
- **Output Formats**: Human-readable, JSON, CSV for automation
- **Integration**: Works with configuration files, environment variables, and presets

## <overview>Command Structure</overview>

```bash
fortitude proactive [GLOBAL_OPTIONS] <SUBCOMMAND> [SUBCOMMAND_OPTIONS] [ARGUMENTS]
```

### <global-options>Global Options</global-options>

```bash
# Configuration and data options
--config, -c <FILE>          Use specific configuration file
--data-dir, -d <DIR>         Set data directory [default: ./reference_library]
--verbose, -v                Enable verbose output
--quiet, -q                  Suppress non-essential output
--format <FORMAT>            Output format: human, json, csv [default: human]

# Environment and debugging
--environment <ENV>          Target environment: dev, staging, prod
--debug                      Enable debug logging
--trace                      Enable trace-level logging
--dry-run                    Show what would be done without executing
```

**Example Global Usage:**
```bash
# Use custom configuration with verbose output
fortitude proactive --config ./my-config.json --verbose status

# JSON output for scripting
fortitude proactive --format json status

# Debug mode for troubleshooting
fortitude proactive --debug start
```

## <start-command>Start Command</start-command>

Start proactive research monitoring and background processing.

### <start-syntax>Syntax</start-syntax>

```bash
fortitude proactive start [OPTIONS]
```

### <start-options>Options</start-options>

```bash
# Core Configuration
--gap-interval <SECONDS>     Gap detection interval [default: 300]
--max-tasks <COUNT>          Maximum concurrent research tasks [default: 3]
--debounce <SECONDS>         File change debounce time [default: 5]

# Monitoring Configuration
--watch-paths <PATHS>        Comma-separated paths to monitor [default: .]
--ignore-patterns <PATTERNS> File patterns to ignore [default: target,node_modules]
--file-types <TYPES>         File extensions to monitor [default: rs,md,toml,js,py]

# Behavior Configuration
--auto-start                 Start immediately without confirmation
--background                 Run in background (daemon mode)
--pid-file <FILE>           Write process ID to file
--log-file <FILE>           Write logs to file

# Performance Configuration
--cpu-limit <PERCENT>        Maximum CPU usage [default: 80]
--memory-limit <MB>          Maximum memory usage [default: 2048]
--priority <LEVEL>           Process priority: low, normal, high [default: normal]

# Notification Configuration
--notify-channels <CHANNELS> Notification channels [default: desktop]
--notify-threshold <LEVEL>   Minimum priority for notifications [default: medium]
--quiet-hours <START:END>    Quiet hours in HH:MM format

# Integration Configuration
--enable-api                 Enable HTTP API server
--api-port <PORT>           API server port [default: 8080]
--enable-mcp                Enable MCP server integration
```

### <start-examples>Examples</start-examples>

**Basic Start:**
```bash
# Start with default settings
fortitude proactive start

# Start with custom interval and higher concurrency
fortitude proactive start --gap-interval 120 --max-tasks 5
```

**Development Setup:**
```bash
# Development mode with frequent monitoring
fortitude proactive start \
  --gap-interval 60 \
  --max-tasks 4 \
  --watch-paths "src,docs,tests" \
  --notify-channels "desktop,console" \
  --cpu-limit 90
```

**Production Setup:**
```bash
# Production mode with conservative settings
fortitude proactive start \
  --gap-interval 600 \
  --max-tasks 2 \
  --background \
  --pid-file /var/run/fortitude.pid \
  --log-file /var/log/fortitude.log \
  --cpu-limit 60 \
  --memory-limit 1024
```

**Custom Project Monitoring:**
```bash
# Monitor specific project structure
fortitude proactive start \
  --watch-paths "src/components,src/pages,docs" \
  --file-types "jsx,tsx,md,css" \
  --ignore-patterns "node_modules,build,dist,*.test.*" \
  --notify-threshold high
```

**Resource-Constrained Environment:**
```bash
# Minimal resource usage
fortitude proactive start \
  --gap-interval 900 \
  --max-tasks 1 \
  --cpu-limit 30 \
  --memory-limit 512 \
  --priority low
```

### <start-output>Expected Output</start-output>

```
🚀 Starting Proactive Research System
=====================================

Configuration:
  Gap Detection Interval: 300 seconds
  Maximum Concurrent Tasks: 3
  Monitored Paths: src/, docs/, tests/
  File Types: rs, md, toml, js, py
  Notification Channels: desktop

System Check:
  ✅ Configuration valid
  ✅ File monitoring ready
  ✅ Notification system ready
  ✅ Research engine ready

🔍 File monitoring started
📋 Gap detection active
🧠 Background research enabled

Status: RUNNING
Started: 2025-07-11 14:30:15 UTC
Process ID: 12345

Use 'fortitude proactive status' to monitor progress
Use 'fortitude proactive stop' to stop the system
```

## <stop-command>Stop Command</stop-command>

Stop proactive research system gracefully.

### <stop-syntax>Syntax</stop-syntax>

```bash
fortitude proactive stop [OPTIONS]
```

### <stop-options>Options</stop-options>

```bash
--timeout <SECONDS>          Graceful shutdown timeout [default: 30]
--force                      Force immediate shutdown
--save-state                 Save current state before stopping
--pid-file <FILE>           Use specific PID file for daemon
--wait                       Wait for all tasks to complete
--no-wait                    Stop immediately without waiting
```

### <stop-examples>Examples</stop-examples>

```bash
# Graceful stop with default timeout
fortitude proactive stop

# Force immediate stop
fortitude proactive stop --force

# Wait for all tasks to complete (up to 60 seconds)
fortitude proactive stop --timeout 60 --wait

# Stop daemon using PID file
fortitude proactive stop --pid-file /var/run/fortitude.pid

# Stop and save current state for later resume
fortitude proactive stop --save-state
```

### <stop-output>Expected Output</stop-output>

```
🛑 Stopping Proactive Research System
====================================

Shutdown Process:
  📋 Stopping gap detection...       ✅ Done
  🧠 Stopping background research... ⏳ Waiting for 2 active tasks
  🧠 Tasks completed...              ✅ Done
  🔍 Stopping file monitoring...     ✅ Done
  💾 Saving state...                 ✅ Done

Final Statistics:
  Uptime: 2 hours 15 minutes
  Gaps Detected: 14
  Research Tasks Completed: 12
  Notifications Sent: 8

System stopped successfully
```

## <status-command>Status Command</status-command>

Display current system status and recent activity.

### <status-syntax>Syntax</status-syntax>

```bash
fortitude proactive status [OPTIONS]
```

### <status-options>Options</status-options>

```bash
# Output Detail Level
--detailed                   Show detailed status information
--brief                      Show brief status only
--metrics                    Include performance metrics
--recent-minutes <MINUTES>   Show activity from last N minutes [default: 60]

# Status Components
--system-check               Run system health checks
--file-monitor               Show file monitoring status
--task-queue                 Show research task queue status
--notifications              Show notification status
--performance                Show performance and resource usage

# Output Options
--follow                     Follow status updates (live mode)
--refresh <SECONDS>          Refresh interval for follow mode [default: 5]
--no-color                   Disable colored output
--compact                    Use compact output format
```

### <status-examples>Examples</status-examples>

**Basic Status:**
```bash
# Quick status overview
fortitude proactive status

# Detailed status with metrics
fortitude proactive status --detailed --metrics
```

**System Health Check:**
```bash
# Comprehensive system check
fortitude proactive status --system-check --performance

# Check file monitoring status
fortitude proactive status --file-monitor --detailed
```

**Live Monitoring:**
```bash
# Follow status updates in real-time
fortitude proactive status --follow

# Follow with custom refresh rate
fortitude proactive status --follow --refresh 10

# Monitor recent activity
fortitude proactive status --follow --recent-minutes 30
```

**Automation-Friendly Output:**
```bash
# JSON output for scripting
fortitude proactive --format json status

# Brief status for health checks
fortitude proactive status --brief --no-color
```

### <status-output>Expected Output</status-output>

**Basic Status:**
```
📊 Proactive Research System Status
==================================

System Status: 🟢 RUNNING
Uptime: 3 hours 42 minutes
Started: 2025-07-11 11:15:32 UTC

📋 Gap Detection:
  Status: Active
  Last Scan: 2 minutes ago
  Scan Interval: 300 seconds
  Files Monitored: 247
  Gaps Detected Today: 8

🧠 Background Research:
  Active Tasks: 2/3
  Completed Today: 15
  Queue Length: 1
  Success Rate: 94%

📬 Notifications:
  Delivered Today: 12
  Channels: desktop, console
  Last Notification: 18 minutes ago

⚡ Performance:
  CPU Usage: 12%
  Memory Usage: 456 MB / 2048 MB (22%)
  Queue Processing: Normal
```

**Detailed Status with Metrics:**
```
📊 Proactive Research System Status - Detailed
===============================================

🔧 System Information:
  Status: 🟢 RUNNING
  Uptime: 3 hours 42 minutes 18 seconds
  Process ID: 12345
  Configuration: .fortitude_config.json
  Environment: development

📋 Gap Detection Engine:
  Status: 🟢 Active
  Last Scan: 2025-07-11 14:45:23 UTC (2m 15s ago)
  Next Scan: 2025-07-11 14:50:23 UTC (2m 45s)
  Scan Interval: 300 seconds
  
  File Monitoring:
    Monitored Paths: src/, docs/, tests/
    Total Files: 247
    File Types: rs (89), md (42), toml (8), js (108)
    Excluded Files: 1,432
    
  Detection Statistics:
    Gaps Detected Today: 8
    Confidence Score: 0.76 (average)
    False Positives: 1
    Manual Confirmations: 7

🧠 Background Research Engine:
  Status: 🟢 Active
  Active Tasks: 2/3
  
  Current Tasks:
    #task_789: "Error handling patterns in Rust" (85% complete)
    #task_790: "React state management best practices" (22% complete)
  
  Queue Status:
    Pending Tasks: 1
    Priority Queue Length: 0
    Failed Tasks (retry): 0
    
  Performance Today:
    Completed Tasks: 15
    Average Duration: 3m 24s
    Success Rate: 94% (14/15)
    Rate Limited: 3 times

📬 Notification System:
  Status: 🟢 Active
  Delivered Today: 12
  
  Channels:
    Desktop: 8 sent, 0 failed
    Console: 12 sent, 0 failed
    Email: Disabled
    
  Rate Limiting:
    Current Hour: 3/10 notifications
    Today: 12/50 notifications
    Quiet Hours: 22:00-08:00 (inactive)

⚡ Performance Metrics:
  System Resources:
    CPU Usage: 12.3% (limit: 80%)
    Memory: 456 MB / 2048 MB (22.3%)
    Disk I/O: 1.2 MB/s read, 0.3 MB/s write
    Network: 245 KB/s (API calls)
    
  Response Times:
    Gap Detection: 145ms (average)
    Research Queries: 2.1s (average)
    Notification Delivery: 89ms (average)
    
  Quality Metrics:
    Research Confidence: 0.82 (average)
    User Satisfaction: N/A (no feedback)
    Cache Hit Rate: 67%

🔍 Recent Activity (last 60 minutes):
  14:45 - Gap detected: "TODO: Add error handling" in src/main.rs:45
  14:43 - Research completed: task_788 "Rust error handling patterns"
  14:41 - Notification sent: Desktop notification for high priority gap
  14:38 - Gap detection scan completed (247 files, 2.1s)
  14:35 - Research started: task_789 "Error handling patterns in Rust"
  14:30 - File change detected: src/components/Header.jsx modified
```

## <configure-command>Configure Command</configure-command>

Manage configuration settings and presets.

### <configure-syntax>Syntax</configure-syntax>

```bash
fortitude proactive configure <SUBCOMMAND> [OPTIONS] [ARGUMENTS]
```

### <configure-subcommands>Configure Subcommands</configure-subcommands>

#### <configure-set>Set Configuration Values</configure-set>

```bash
fortitude proactive configure set <KEY> <VALUE> [OPTIONS]

# Options:
--global                     Set in global user configuration
--local                      Set in local project configuration [default]
--session                    Set for current session only
--validate                   Validate value before setting
--backup                     Create backup before changing
```

**Examples:**
```bash
# Basic configuration setting
fortitude proactive configure set gap_analysis.scan_intervals_seconds 240

# Set notification channels
fortitude proactive configure set notifications.channels "desktop,email,slack"

# Set with validation
fortitude proactive configure set performance.resource_limits.max_memory_mb 4096 --validate

# Set global user preference
fortitude proactive configure set user_preferences.notification_frequency_hours 6 --global
```

#### <configure-get>Get Configuration Values</configure-get>

```bash
fortitude proactive configure get <KEY> [OPTIONS]

# Options:
--source                     Show where value comes from
--default                    Show default value
--all-sources                Show value from all sources
```

**Examples:**
```bash
# Get specific value
fortitude proactive configure get gap_analysis.scan_intervals_seconds

# Get with source information
fortitude proactive configure get notifications.channels --source

# Get all gap analysis settings
fortitude proactive configure get gap_analysis
```

#### <configure-show>Show Configuration</configure-show>

```bash
fortitude proactive configure show [SECTION] [OPTIONS]

# Options:
--format <FORMAT>            Output format: human, json, yaml, toml
--sources                    Show value sources
--effective                  Show effective configuration (merged)
--defaults                   Show default values only
--user-only                  Show user-modified values only
```

**Examples:**
```bash
# Show all configuration
fortitude proactive configure show

# Show specific section
fortitude proactive configure show notifications

# Show as JSON
fortitude proactive configure show --format json

# Show with sources
fortitude proactive configure show --sources --effective
```

#### <configure-list>List Configuration Options</configure-list>

```bash
fortitude proactive configure list [SECTION] [OPTIONS]

# Options:
--detailed                   Show descriptions and constraints
--user-configurable          Show only user-modifiable settings
--deprecated                 Include deprecated settings
--search <PATTERN>           Search for specific keys
```

**Examples:**
```bash
# List all available settings
fortitude proactive configure list

# List with descriptions
fortitude proactive configure list --detailed

# Search for notification settings
fortitude proactive configure list --search notification
```

#### <configure-preset>Configuration Presets</configure-preset>

```bash
fortitude proactive configure preset <PRESET_NAME> [OPTIONS]

# Available presets:
#   development    - Active development with frequent feedback
#   production     - Conservative production settings
#   research       - Enhanced research capabilities
#   minimal        - Resource-constrained environments

# Options:
--preview                    Show preset without applying
--backup                     Backup current config before applying
--merge                      Merge with current config instead of replacing
```

**Examples:**
```bash
# Apply development preset
fortitude proactive configure preset development

# Preview production preset
fortitude proactive configure preset production --preview

# Apply research preset with backup
fortitude proactive configure preset research --backup
```

#### <configure-reset>Reset Configuration</configure-reset>

```bash
fortitude proactive configure reset [SECTION] [OPTIONS]

# Options:
--confirm                    Skip confirmation prompt
--backup                     Create backup before reset
--to-preset <PRESET>         Reset to specific preset instead of defaults
```

**Examples:**
```bash
# Reset all configuration
fortitude proactive configure reset --confirm

# Reset specific section
fortitude proactive configure reset notifications

# Reset to production preset
fortitude proactive configure reset --to-preset production
```

#### <configure-export>Export Configuration</configure-export>

```bash
fortitude proactive configure export [OPTIONS]

# Options:
--output <FILE>              Output file path
--format <FORMAT>            Format: json, yaml, toml [default: json]
--effective                  Export effective configuration
--minimal                    Export only non-default values
--include-secrets            Include sensitive values (use carefully)
```

**Examples:**
```bash
# Export to JSON file
fortitude proactive configure export --output my-config.json

# Export effective configuration
fortitude proactive configure export --effective --format yaml

# Export minimal configuration (changes only)
fortitude proactive configure export --minimal --output changes.toml
```

#### <configure-import>Import Configuration</configure-import>

```bash
fortitude proactive configure import <FILE> [OPTIONS]

# Options:
--merge                      Merge with existing configuration
--backup                     Backup current configuration first
--validate                   Validate before importing
--dry-run                    Show what would be imported
```

**Examples:**
```bash
# Import configuration file
fortitude proactive configure import my-config.json --backup

# Import with validation
fortitude proactive configure import team-config.yaml --validate --merge

# Preview import
fortitude proactive configure import new-config.toml --dry-run
```

#### <configure-validate>Validate Configuration</configure-validate>

```bash
fortitude proactive configure validate [FILE] [OPTIONS]

# Options:
--fix                        Attempt to fix validation errors
--detailed                   Show detailed validation results
--check-conflicts            Check for conflicting settings
--performance-check          Validate performance implications
```

**Examples:**
```bash
# Validate current configuration
fortitude proactive configure validate

# Validate specific file
fortitude proactive configure validate my-config.json --detailed

# Check for conflicts
fortitude proactive configure validate --check-conflicts
```

## <tasks-command>Tasks Command</tasks-command>

Manage research tasks and view task status.

### <tasks-syntax>Syntax</tasks-syntax>

```bash
fortitude proactive tasks <SUBCOMMAND> [OPTIONS]
```

### <tasks-subcommands>Task Subcommands</tasks-subcommands>

#### <tasks-list>List Tasks</tasks-list>

```bash
fortitude proactive tasks list [OPTIONS]

# Options:
--status <STATUS>            Filter by status: pending, active, completed, failed
--priority <PRIORITY>        Filter by priority: low, medium, high, critical
--limit <COUNT>              Limit number of results [default: 20]
--since <TIME>               Show tasks since time (e.g., 1h, 2d, 2025-07-11)
--format <FORMAT>            Output format: table, json, csv
--show-details               Include task details and progress
```

**Examples:**
```bash
# List recent tasks
fortitude proactive tasks list

# List active tasks with details
fortitude proactive tasks list --status active --show-details

# List completed tasks from today
fortitude proactive tasks list --status completed --since 1d

# List high priority tasks
fortitude proactive tasks list --priority high --limit 10
```

#### <tasks-show>Show Task Details</tasks-show>

```bash
fortitude proactive tasks show <TASK_ID> [OPTIONS]

# Options:
--include-research           Include research results
--include-sources            Include source information
--follow                     Follow task progress (for active tasks)
```

**Examples:**
```bash
# Show task details
fortitude proactive tasks show task_123

# Show with research results
fortitude proactive tasks show task_123 --include-research

# Follow active task progress
fortitude proactive tasks show task_123 --follow
```

#### <tasks-cancel>Cancel Tasks</tasks-cancel>

```bash
fortitude proactive tasks cancel <TASK_ID> [OPTIONS]

# Options:
--force                      Force cancellation without confirmation
--reason <REASON>            Cancellation reason
```

**Examples:**
```bash
# Cancel specific task
fortitude proactive tasks cancel task_123

# Cancel with reason
fortitude proactive tasks cancel task_123 --reason "Duplicate research"
```

#### <tasks-retry>Retry Failed Tasks</tasks-retry>

```bash
fortitude proactive tasks retry <TASK_ID> [OPTIONS]

# Options:
--priority <PRIORITY>        Set new priority for retry
--force                      Retry even if max attempts reached
```

**Examples:**
```bash
# Retry failed task
fortitude proactive tasks retry task_123

# Retry with high priority
fortitude proactive tasks retry task_123 --priority high
```

#### <tasks-statistics>Task Statistics</tasks-statistics>

```bash
fortitude proactive tasks stats [OPTIONS]

# Options:
--period <PERIOD>            Time period: hour, day, week, month
--detailed                   Include detailed breakdown
--format <FORMAT>            Output format: table, json
```

**Examples:**
```bash
# Show today's statistics
fortitude proactive tasks stats --period day

# Detailed weekly statistics
fortitude proactive tasks stats --period week --detailed
```

### <tasks-output>Task List Output</tasks-output>

```
📋 Research Tasks
================

Active Tasks (2):
  #task_789  🔄 Error handling patterns in Rust           Priority: High     Progress: 85%
  #task_790  🔄 React state management best practices     Priority: Medium   Progress: 22%

Pending Tasks (1):
  #task_791  ⏳ API documentation standards               Priority: Medium   Created: 5m ago

Recent Completed (5):
  #task_788  ✅ Rust error handling patterns              Completed: 12m ago  Duration: 3m 24s
  #task_787  ✅ CSS Grid layout techniques                Completed: 28m ago  Duration: 2m 15s
  #task_786  ✅ Database migration strategies             Completed: 45m ago  Duration: 4m 31s
  #task_785  ✅ Testing best practices for React          Completed: 1h ago   Duration: 2m 52s
  #task_784  ✅ Performance optimization in Node.js       Completed: 1h ago   Duration: 3m 18s

Task Statistics Today:
  Total: 18 tasks
  Completed: 15 (83%)
  Failed: 1 (6%)
  Average Duration: 3m 12s
  Success Rate: 94%
```

## <notifications-command>Notifications Command</notifications-command>

Manage notifications and view notification history.

### <notifications-syntax>Syntax</notifications-syntax>

```bash
fortitude proactive notifications <SUBCOMMAND> [OPTIONS]
```

### <notifications-subcommands>Notification Subcommands</notifications-subcommands>

#### <notifications-list>List Notifications</notifications-list>

```bash
fortitude proactive notifications list [OPTIONS]

# Options:
--unread                     Show only unread notifications
--since <TIME>               Show notifications since time
--channel <CHANNEL>          Filter by channel: desktop, email, console, webhook
--type <TYPE>                Filter by type: gap_detected, task_completed, system_alert
--limit <COUNT>              Limit number of results [default: 20]
--mark-read                  Mark listed notifications as read
```

**Examples:**
```bash
# List recent notifications
fortitude proactive notifications list

# List unread notifications
fortitude proactive notifications list --unread

# List today's desktop notifications
fortitude proactive notifications list --channel desktop --since 1d
```

#### <notifications-show>Show Notification Details</notifications-show>

```bash
fortitude proactive notifications show <NOTIFICATION_ID> [OPTIONS]

# Options:
--mark-read                  Mark notification as read after showing
```

#### <notifications-mark-read>Mark Notifications as Read</notifications-read>

```bash
fortitude proactive notifications mark-read [OPTIONS]

# Options:
--all                        Mark all notifications as read
--since <TIME>               Mark notifications since time as read
--ids <IDS>                  Mark specific notification IDs as read
```

#### <notifications-test>Test Notifications</notifications-test>

```bash
fortitude proactive notifications test [OPTIONS]

# Options:
--channel <CHANNEL>          Test specific channel
--type <TYPE>                Test specific notification type
--message <MESSAGE>          Custom test message
```

**Examples:**
```bash
# Test all notification channels
fortitude proactive notifications test

# Test desktop notifications
fortitude proactive notifications test --channel desktop

# Test with custom message
fortitude proactive notifications test --message "This is a test notification"
```

#### <notifications-preferences>Notification Preferences</notifications-preferences>

```bash
fortitude proactive notifications preferences [OPTIONS]

# Options:
--set <KEY=VALUE>            Set preference value
--get <KEY>                  Get preference value
--list                       List all preferences
--reset                      Reset to default preferences
```

**Examples:**
```bash
# Show current preferences
fortitude proactive notifications preferences --list

# Set notification frequency
fortitude proactive notifications preferences --set frequency=immediate

# Enable quiet hours
fortitude proactive notifications preferences --set quiet_hours.enabled=true
fortitude proactive notifications preferences --set quiet_hours.start=22:00
fortitude proactive notifications preferences --set quiet_hours.end=08:00
```

## <advanced-usage>Advanced CLI Usage</advanced-usage>

### <scripting>Scripting and Automation</scripting>

#### <json-output>JSON Output for Scripts</json-output>

```bash
# Get status as JSON
STATUS=$(fortitude proactive --format json status)
echo $STATUS | jq '.data.system_status'

# Check if system is running
IS_RUNNING=$(fortitude proactive --format json status | jq -r '.data.is_running')
if [ "$IS_RUNNING" = "true" ]; then
    echo "System is running"
fi

# Get task count
ACTIVE_TASKS=$(fortitude proactive --format json tasks list --status active | jq '.data.tasks | length')
echo "Active tasks: $ACTIVE_TASKS"
```

#### <monitoring-scripts>Monitoring Scripts</monitoring-scripts>

```bash
#!/bin/bash
# monitoring-script.sh - Monitor proactive research system

while true; do
    # Check system health
    STATUS=$(fortitude proactive --format json status --brief)
    IS_RUNNING=$(echo $STATUS | jq -r '.data.is_running')
    
    if [ "$IS_RUNNING" != "true" ]; then
        echo "$(date): System not running, attempting restart"
        fortitude proactive start --background
        sleep 10
    fi
    
    # Check resource usage
    CPU_USAGE=$(echo $STATUS | jq -r '.data.performance.cpu_usage_percent')
    if (( $(echo "$CPU_USAGE > 90" | bc -l) )); then
        echo "$(date): High CPU usage: $CPU_USAGE%"
        fortitude proactive configure set performance.resource_limits.max_cpu_percent 70
    fi
    
    sleep 60
done
```

#### <backup-scripts>Configuration Backup Scripts</backup-scripts>

```bash
#!/bin/bash
# backup-config.sh - Backup proactive research configuration

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="$HOME/.fortitude/backups"
mkdir -p "$BACKUP_DIR"

# Backup effective configuration
fortitude proactive configure export \
    --effective \
    --format json \
    --output "$BACKUP_DIR/config_$DATE.json"

# Backup user preferences
fortitude proactive configure export \
    --minimal \
    --format toml \
    --output "$BACKUP_DIR/preferences_$DATE.toml"

echo "Configuration backed up to $BACKUP_DIR"

# Clean old backups (keep last 10)
ls -t "$BACKUP_DIR"/config_*.json | tail -n +11 | xargs -r rm
ls -t "$BACKUP_DIR"/preferences_*.toml | tail -n +11 | xargs -r rm
```

### <integration>Integration Examples</integration>

#### <git-hooks>Git Hook Integration</git-hooks>

```bash
#!/bin/bash
# .git/hooks/post-commit - Start proactive research after commits

# Check if proactive research is installed
if command -v fortitude >/dev/null 2>&1; then
    # Start proactive research if not already running
    if ! fortitude proactive status --brief >/dev/null 2>&1; then
        echo "Starting proactive research..."
        fortitude proactive start --auto-start --background
    fi
    
    # Trigger immediate gap detection
    fortitude proactive tasks create \
        --priority high \
        --description "Post-commit gap analysis" \
        --immediate
fi
```

#### <ci-cd-integration>CI/CD Integration</ci-cd-integration>

```yaml
# .github/workflows/proactive-research.yml
name: Proactive Research Analysis

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  research-analysis:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    
    - name: Install Fortitude
      run: |
        # Install fortitude binary
        curl -L https://github.com/your-org/fortitude/releases/latest/download/fortitude-linux.tar.gz | tar xz
        sudo mv fortitude /usr/local/bin/
    
    - name: Run Gap Analysis
      run: |
        # Configure for CI environment
        fortitude proactive configure preset ci
        
        # Start proactive research
        fortitude proactive start --background --timeout 300
        
        # Wait for analysis completion
        sleep 60
        
        # Export results
        fortitude proactive tasks list --status completed --format json > research-results.json
        fortitude proactive notifications list --format json > notifications.json
    
    - name: Upload Results
      uses: actions/upload-artifact@v2
      with:
        name: research-results
        path: |
          research-results.json
          notifications.json
```

#### <vscode-integration>VS Code Integration</vscode-integration>

```json
// .vscode/tasks.json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Start Proactive Research",
            "type": "shell",
            "command": "fortitude",
            "args": ["proactive", "start", "--auto-start"],
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "shared"
            },
            "runOptions": {
                "runOn": "folderOpen"
            }
        },
        {
            "label": "Check Research Status",
            "type": "shell",
            "command": "fortitude",
            "args": ["proactive", "status", "--detailed"],
            "group": "test"
        },
        {
            "label": "Stop Proactive Research",
            "type": "shell",
            "command": "fortitude",
            "args": ["proactive", "stop"],
            "group": "build"
        }
    ]
}
```

### <troubleshooting-cli>CLI Troubleshooting</troubleshooting-cli>

#### <common-cli-issues>Common CLI Issues</common-cli-issues>

**Issue 1: Command Not Found**
```bash
# Check if fortitude is installed
which fortitude

# Check PATH
echo $PATH

# Install or add to PATH
export PATH=$PATH:/path/to/fortitude
```

**Issue 2: Permission Denied**
```bash
# Check permissions
ls -la $(which fortitude)

# Fix permissions if needed
chmod +x $(which fortitude)

# Check configuration directory permissions
ls -la ~/.fortitude/
```

**Issue 3: Configuration Errors**
```bash
# Validate configuration
fortitude proactive configure validate --detailed

# Reset to defaults if corrupted
fortitude proactive configure reset --confirm

# Check for conflicting settings
fortitude proactive configure validate --check-conflicts
```

#### <debug-commands>Debug Commands</debug-commands>

```bash
# Enable debug logging
fortitude proactive --debug status

# Trace mode for detailed debugging
fortitude proactive --trace start

# Dry run to see what would happen
fortitude proactive --dry-run start --gap-interval 60

# Check system dependencies
fortitude proactive status --system-check --detailed
```

### <performance-tuning>Performance Tuning via CLI</performance-tuning>

```bash
# Monitor resource usage
fortitude proactive status --performance --follow

# Adjust for low-resource environments
fortitude proactive configure set performance.resource_limits.max_memory_mb 512
fortitude proactive configure set background_research.max_concurrent_tasks 1
fortitude proactive configure set gap_analysis.scan_intervals_seconds 600

# Optimize for high-performance environments
fortitude proactive configure set performance.resource_limits.max_memory_mb 4096
fortitude proactive configure set background_research.max_concurrent_tasks 8
fortitude proactive configure set gap_analysis.scan_intervals_seconds 60

# Enable performance monitoring
fortitude proactive configure set performance.monitoring_enabled true
fortitude proactive configure set performance.optimization.auto_optimization_enabled true
```

---

**Next Steps:**
- [API Usage Guide](proactive-research-api.md) - HTTP API integration
- [Configuration Guide](proactive-research-configuration.md) - Detailed configuration reference
- [MCP Tools Guide](proactive-research-mcp.md) - Claude Code integration
- [Troubleshooting Guide](proactive-research-troubleshooting.md) - Common issues and solutions