# Proactive Research Troubleshooting Guide

<meta>
  <title>Proactive Research Troubleshooting - Common Issues and Solutions</title>
  <type>user_guide</type>
  <audience>developer</audience>
  <complexity>basic_to_advanced</complexity>
  <updated>2025-07-11</updated>
  <sprint>008</sprint>
  <version>1.0</version>
</meta>

## <summary priority="high">Troubleshooting Overview</summary>
- **Quick Diagnostics**: Automated system health checks and issue detection
- **Common Issues**: Step-by-step solutions for frequent problems
- **Performance Problems**: Resource usage optimization and tuning
- **Configuration Issues**: Configuration validation and repair
- **Integration Problems**: API, CLI, and MCP connectivity issues

## <overview>Troubleshooting Approach</overview>

This guide follows a systematic troubleshooting methodology:

1. **Quick Diagnostics** - Automated health checks to identify the issue
2. **Symptom Analysis** - Understanding what's actually happening
3. **Root Cause Investigation** - Finding the underlying problem
4. **Solution Implementation** - Step-by-step fixes
5. **Verification** - Confirming the problem is resolved
6. **Prevention** - Avoiding the issue in the future

### <emergency-commands>Emergency Commands</emergency-commands>

**System Health Check:**
```bash
# Complete system diagnostic
fortitude proactive status --system-check --detailed

# Quick health check
fortitude proactive status --brief

# Check if system is responsive
fortitude proactive configure validate
```

**Emergency Recovery:**
```bash
# Stop system safely
fortitude proactive stop --force --timeout 10

# Reset configuration to defaults
fortitude proactive configure reset --confirm

# Restart with minimal settings
fortitude proactive configure preset minimal
fortitude proactive start --auto-start
```

## <startup-issues>System Won't Start</startup-issues>

### <startup-diagnosis>Diagnosis</startup-diagnosis>

**Symptoms:**
- `fortitude proactive start` fails or hangs
- Error messages about missing dependencies
- Configuration validation errors

**Quick Diagnosis:**
```bash
# Check system dependencies
fortitude proactive status --system-check

# Validate configuration
fortitude proactive configure validate --detailed

# Check file permissions
ls -la ~/.fortitude/
```

### <startup-solutions>Solutions</startup-solutions>

#### <issue-missing-dependencies>Missing Dependencies</issue-missing-dependencies>

**Error:** "Command not found" or "Missing required dependencies"

**Solution:**
```bash
# Verify installation
which fortitude
fortitude --version

# Check required system dependencies
fortitude proactive status --system-check --dependencies

# Install missing dependencies (example for Ubuntu/Debian)
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# Reinstall if necessary
curl -sSL https://install.fortitude.dev | sh
```

#### <issue-configuration-errors>Configuration Errors</issue-configuration-errors>

**Error:** "Invalid configuration" or validation failures

**Solution:**
```bash
# Check specific configuration errors
fortitude proactive configure validate --detailed

# Common fixes for configuration issues:

# 1. Invalid scan interval
fortitude proactive configure set gap_analysis.scan_intervals_seconds 300

# 2. Invalid resource limits
fortitude proactive configure set performance.resource_limits.max_memory_mb 2048
fortitude proactive configure set performance.resource_limits.max_cpu_percent 80

# 3. Invalid file patterns
fortitude proactive configure set gap_analysis.file_patterns "*.rs,*.md,*.toml"

# 4. Reset to known good configuration
fortitude proactive configure preset development
```

#### <issue-permission-errors>Permission Errors</issue-permission-errors>

**Error:** "Permission denied" or "Cannot write to configuration directory"

**Solution:**
```bash
# Check current permissions
ls -la ~/.fortitude/

# Fix configuration directory permissions
chmod 755 ~/.fortitude/
chmod 644 ~/.fortitude/config.toml

# Fix binary permissions if needed
chmod +x $(which fortitude)

# Check if running as correct user
whoami
id

# If running as root (not recommended):
# Create proper user configuration
sudo -u $USER fortitude proactive configure show
```

#### <issue-port-conflicts>Port Conflicts</issue-port-conflicts>

**Error:** "Port already in use" or "Cannot bind to port"

**Solution:**
```bash
# Check what's using the port
sudo netstat -tlnp | grep :8080
# or
sudo lsof -i :8080

# Kill conflicting process (if safe)
sudo kill $(sudo lsof -t -i:8080)

# Use different port
fortitude proactive configure set api.port 8081

# Start with custom port
fortitude proactive start --api-port 8081
```

## <performance-issues>Performance Issues</performance-issues>

### <high-resource-usage>High Resource Usage</high-resource-usage>

**Symptoms:**
- High CPU usage (>90%)
- Excessive memory consumption
- System becomes slow or unresponsive
- Fans spinning at high speed

**Diagnosis:**
```bash
# Check current resource usage
fortitude proactive status --performance --detailed

# Monitor resource usage over time
fortitude proactive status --follow --refresh 5

# Check system resources
top -p $(pgrep fortitude)
# or
htop -p $(pgrep fortitude)
```

**Solutions:**

#### <reduce-scan-frequency>Reduce Scan Frequency</reduce-scan-frequency>

```bash
# Current scan interval
fortitude proactive configure get gap_analysis.scan_intervals_seconds

# Increase interval to reduce CPU usage
fortitude proactive configure set gap_analysis.scan_intervals_seconds 600  # 10 minutes

# For very large projects
fortitude proactive configure set gap_analysis.scan_intervals_seconds 1800  # 30 minutes
```

#### <limit-concurrent-tasks>Limit Concurrent Tasks</limit-concurrent-tasks>

```bash
# Current concurrent task limit
fortitude proactive configure get background_research.max_concurrent_tasks

# Reduce concurrent tasks
fortitude proactive configure set background_research.max_concurrent_tasks 2

# For single-core or low-memory systems
fortitude proactive configure set background_research.max_concurrent_tasks 1
```

#### <adjust-resource-limits>Adjust Resource Limits</adjust-resource-limits>

```bash
# Set CPU limit
fortitude proactive configure set performance.resource_limits.max_cpu_percent 50

# Set memory limit
fortitude proactive configure set performance.resource_limits.max_memory_mb 1024

# Enable resource monitoring
fortitude proactive configure set performance.monitoring_enabled true
```

#### <optimize-file-monitoring>Optimize File Monitoring</optimize-file-monitoring>

```bash
# Current watch paths
fortitude proactive configure get workspace.project_paths

# Limit to essential directories only
fortitude proactive configure set workspace.project_paths "src,docs"

# Add more ignore patterns
fortitude proactive configure set workspace.exclude_patterns "target,node_modules,dist,build,*.tmp,*.log"

# Limit file types to essential ones
fortitude proactive configure set gap_analysis.file_patterns "*.rs,*.md"
```

### <slow-response-times>Slow Response Times</slow-response-times>

**Symptoms:**
- Commands take a long time to complete
- Web interface is sluggish
- Research tasks take too long

**Diagnosis:**
```bash
# Check response times
time fortitude proactive status

# Check task queue
fortitude proactive tasks list --status pending

# Check system performance
fortitude proactive status --performance --metrics
```

**Solutions:**

#### <database-optimization>Database Optimization</database-optimization>

```bash
# Check vector database status
fortitude proactive status --database-check

# Rebuild vector database indexes (if supported)
fortitude proactive maintenance rebuild-indexes

# Clear old cache data
fortitude proactive maintenance clear-cache --older-than 7d

# Compact database
fortitude proactive maintenance compact-database
```

#### <network-optimization>Network Optimization</network-optimization>

```bash
# Check API response times
fortitude proactive status --api-check

# Reduce research timeout for faster failures
fortitude proactive configure set background_research.research_timeout_seconds 180

# Increase rate limiting for faster processing
fortitude proactive configure set background_research.rate_limit_requests_per_minute 30
```

## <gap-detection-issues>Gap Detection Issues</gap-detection-issues>

### <no-gaps-detected>No Gaps Being Detected</no-gaps-detected>

**Symptoms:**
- System running but no gaps found
- Expected TODO comments not detected
- File changes not triggering scans

**Diagnosis:**
```bash
# Check file monitoring status
fortitude proactive status --file-monitor

# Check gap detection configuration
fortitude proactive configure show gap_analysis

# Test with known gap
echo "// TODO: Test gap detection" >> test_gap.rs
```

**Solutions:**

#### <verify-file-monitoring>Verify File Monitoring</verify-file-monitoring>

```bash
# Check if files are being monitored
fortitude proactive status --file-monitor --detailed

# Verify file patterns
fortitude proactive configure get gap_analysis.file_patterns

# Test file pattern matching
fortitude proactive configure test-patterns --file test_gap.rs

# Common file pattern fixes:
fortitude proactive configure set gap_analysis.file_patterns "*.rs,*.md,*.toml,*.js,*.py"
```

#### <adjust-detection-rules>Adjust Detection Rules</adjust-detection-rules>

```bash
# Check current detection rules
fortitude proactive configure get gap_analysis.detection_rules

# Add common patterns
fortitude proactive configure set gap_analysis.detection_rules "todo,fixme,hack,bug,note,xxx"

# Lower confidence threshold
fortitude proactive configure get gap_analysis.confidence_threshold
fortitude proactive configure set gap_analysis.confidence_threshold 0.5

# Enable semantic analysis
fortitude proactive configure set gap_analysis.enable_semantic_analysis true
```

#### <check-ignore-patterns>Check Ignore Patterns</check-ignore-patterns>

```bash
# Review ignore patterns
fortitude proactive configure get workspace.exclude_patterns

# Common issue: too aggressive ignoring
# Remove overly broad patterns like "*test*" or "src"

# Fix common ignore pattern issues:
fortitude proactive configure set workspace.exclude_patterns "target,node_modules,.git,dist,build"
```

### <false-positives>Too Many False Positives</false-positives>

**Symptoms:**
- Detecting gaps in comments or documentation
- Flagging intentional TODO items
- Too many low-priority gaps

**Solutions:**

#### <increase-confidence-threshold>Increase Confidence Threshold</increase-confidence-threshold>

```bash
# Current threshold
fortitude proactive configure get gap_analysis.confidence_threshold

# Increase for fewer false positives
fortitude proactive configure set gap_analysis.confidence_threshold 0.8

# For very strict detection
fortitude proactive configure set gap_analysis.confidence_threshold 0.9
```

#### <refine-detection-rules>Refine Detection Rules</refine-detection-rules>

```bash
# Add custom rules to ignore certain patterns
fortitude proactive configure add-custom-rule \
  --name "Ignore Documentation TODOs" \
  --pattern "// TODO.*(?:doc|documentation)" \
  --action ignore

# Ignore test-related TODOs
fortitude proactive configure add-custom-rule \
  --name "Ignore Test TODOs" \
  --pattern "// TODO.*test" \
  --action ignore
```

#### <adjust-priority-filters>Adjust Priority Filters</adjust-priority-filters>

```bash
# Only detect medium and high priority gaps
fortitude proactive configure set notifications.min_priority medium

# Adjust priority keywords
fortitude proactive configure set background_research.priority_keywords "urgent,critical,security,performance"
```

## <notification-issues>Notification Issues</notification-issues>

### <no-notifications>Not Receiving Notifications</no-notifications>

**Symptoms:**
- No notifications despite active system
- Expected notifications not appearing
- Notification channels not working

**Diagnosis:**
```bash
# Check notification status
fortitude proactive notifications test

# Check notification preferences
fortitude proactive configure show notifications

# Check recent notifications
fortitude proactive notifications list --since 1h
```

**Solutions:**

#### <verify-notification-channels>Verify Notification Channels</verify-notification-channels>

```bash
# Check enabled channels
fortitude proactive configure get notifications.channels

# Enable desktop notifications
fortitude proactive configure set notifications.channels "desktop,console"

# Test specific channel
fortitude proactive notifications test --channel desktop

# Check channel-specific settings
fortitude proactive configure show notifications.channel_settings
```

#### <check-quiet-hours>Check Quiet Hours</check-quiet-hours>

```bash
# Check if quiet hours are blocking notifications
fortitude proactive configure get notifications.quiet_hours

# Disable quiet hours temporarily
fortitude proactive configure set notifications.quiet_hours.enabled false

# Adjust quiet hours
fortitude proactive configure set notifications.quiet_hours.start "23:00"
fortitude proactive configure set notifications.quiet_hours.end "07:00"
```

#### <check-rate-limiting>Check Rate Limiting</check-rate-limiting>

```bash
# Check rate limiting status
fortitude proactive configure get notifications.rate_limiting

# Check if rate limit reached
fortitude proactive notifications stats --today

# Increase rate limits
fortitude proactive configure set notifications.rate_limiting.max_per_hour 20
fortitude proactive configure set notifications.rate_limiting.max_per_day 100
```

### <too-many-notifications>Too Many Notifications</too-many-notifications>

**Symptoms:**
- Overwhelmed by notification volume
- Constant interruptions
- Important notifications buried

**Solutions:**

#### <adjust-notification-frequency>Adjust Notification Frequency</adjust-notification-frequency>

```bash
# Switch from immediate to batched notifications
fortitude proactive configure set notifications.frequency batched

# Set batch interval
fortitude proactive configure set notifications.batch_interval_minutes 30

# Enable notification summary
fortitude proactive configure set notifications.enable_summary true
```

#### <filter-by-priority>Filter by Priority</filter-priority>

```bash
# Only notify for high priority items
fortitude proactive configure set notifications.min_priority high

# Only critical notifications
fortitude proactive configure set notifications.min_priority critical

# Custom priority filtering
fortitude proactive configure set notifications.priority_filter "high,critical"
```

#### <enable-quiet-hours>Enable Quiet Hours</enable-quiet-hours>

```bash
# Enable quiet hours
fortitude proactive configure set notifications.quiet_hours.enabled true
fortitude proactive configure set notifications.quiet_hours.start "18:00"
fortitude proactive configure set notifications.quiet_hours.end "09:00"

# Weekend quiet mode
fortitude proactive configure set notifications.quiet_hours.include_weekends true
```

## <integration-issues>Integration Issues</integration-issues>

### <api-connection-problems>API Connection Problems</api-connection-problems>

**Symptoms:**
- API calls timeout or fail
- 401 authentication errors
- Connection refused errors

**Diagnosis:**
```bash
# Test API connectivity
curl -H "X-API-Key: your-key" http://localhost:8080/health

# Check API server status
fortitude proactive status --api-server

# Check API logs
fortitude proactive logs --component api --tail 50
```

**Solutions:**

#### <api-authentication-issues>API Authentication Issues</api-authentication-issues>

```bash
# Generate new API key
fortitude proactive configure generate-api-key --name "troubleshooting"

# Check current API key
fortitude proactive configure get api.authentication.api_key

# Test with new key
export API_KEY="new-generated-key"
curl -H "X-API-Key: $API_KEY" http://localhost:8080/api/v1/health/protected
```

#### <api-server-not-running>API Server Not Running</api-server-not-running>

```bash
# Check if API server is enabled
fortitude proactive configure get api.enabled

# Enable API server
fortitude proactive configure set api.enabled true

# Start with API server enabled
fortitude proactive start --enable-api --api-port 8080

# Check if port is available
sudo netstat -tlnp | grep :8080
```

### <mcp-integration-problems>MCP Integration Problems</mcp-integration-problems>

**Symptoms:**
- MCP tools not available in Claude Code
- MCP calls timeout or fail
- Authentication errors with MCP

**Diagnosis:**
```bash
# Check MCP server status
fortitude mcp-server status

# Test MCP connectivity
fortitude mcp-server test-connection

# Check MCP logs
fortitude mcp-server logs --tail 20
```

**Solutions:**

#### <mcp-server-not-running>MCP Server Not Running</mcp-server-not-running>

```bash
# Start MCP server
fortitude mcp-server start

# Enable auto-start
fortitude mcp-server configure set auto_start true

# Check MCP configuration
fortitude mcp-server configure show
```

#### <mcp-authentication-issues>MCP Authentication Issues</mcp-authentication-issues>

```bash
# Generate MCP-specific API key
fortitude proactive configure generate-api-key --name "MCP Integration" --scope mcp

# Configure MCP server with new key
fortitude mcp-server configure set api_key "new-mcp-key"

# Restart MCP server
fortitude mcp-server restart
```

### <cli-issues>CLI Issues</cli-issues>

**Symptoms:**
- Commands not found
- Unexpected command behavior
- Configuration not persisting

**Solutions:**

#### <command-not-found>Command Not Found</command-not-found>

```bash
# Check if fortitude is in PATH
which fortitude
echo $PATH

# Add to PATH if needed
export PATH=$PATH:/path/to/fortitude

# Permanent PATH addition (add to ~/.bashrc or ~/.zshrc)
echo 'export PATH=$PATH:/path/to/fortitude' >> ~/.bashrc
source ~/.bashrc
```

#### <configuration-not-persisting>Configuration Not Persisting</configuration-not-persisting>

```bash
# Check configuration file location
fortitude proactive configure show --sources

# Check file permissions
ls -la ~/.fortitude/config.toml

# Fix permissions
chmod 644 ~/.fortitude/config.toml

# Create configuration directory if missing
mkdir -p ~/.fortitude/
```

## <data-issues>Data and Storage Issues</data-issues>

### <corrupted-data>Corrupted or Missing Data</corrupted-data>

**Symptoms:**
- Database errors
- Missing research results
- Cache corruption errors

**Diagnosis:**
```bash
# Check data directory
fortitude proactive status --data-check

# Check database integrity
fortitude proactive maintenance check-integrity

# Check available disk space
df -h ~/.fortitude/
```

**Solutions:**

#### <database-corruption>Database Corruption</database-corruption>

```bash
# Stop system
fortitude proactive stop

# Backup current data
cp -r ~/.fortitude/data ~/.fortitude/data.backup

# Rebuild database
fortitude proactive maintenance rebuild-database

# Restart system
fortitude proactive start
```

#### <cache-issues>Cache Issues</cache-issues>

```bash
# Clear cache
fortitude proactive maintenance clear-cache

# Rebuild cache
fortitude proactive maintenance rebuild-cache

# Check cache statistics
fortitude proactive status --cache-stats
```

### <disk-space-issues>Disk Space Issues</disk-space-issues>

**Symptoms:**
- "No space left on device" errors
- Slow performance
- Failed to write data

**Solutions:**

#### <clean-old-data>Clean Old Data</clean-old-data>

```bash
# Check disk usage
du -sh ~/.fortitude/

# Clean old research results
fortitude proactive maintenance clean-old-results --older-than 30d

# Clean old logs
fortitude proactive maintenance clean-logs --older-than 7d

# Compact database
fortitude proactive maintenance compact-database
```

#### <adjust-storage-limits>Adjust Storage Limits</adjust-storage-limits>

```bash
# Set cache size limit
fortitude proactive configure set performance.caching.size_limit_mb 512

# Set log rotation
fortitude proactive configure set logging.max_log_size_mb 100
fortitude proactive configure set logging.max_log_files 5

# Enable automatic cleanup
fortitude proactive configure set maintenance.auto_cleanup_enabled true
```

## <advanced-troubleshooting>Advanced Troubleshooting</advanced-troubleshooting>

### <debug-mode>Debug Mode</debug-mode>

**Enable comprehensive debugging:**
```bash
# Enable debug logging
fortitude proactive configure set logging.level debug

# Start with debug mode
fortitude proactive start --debug

# Follow debug logs
fortitude proactive logs --follow --level debug

# Enable trace mode for detailed debugging
fortitude proactive start --trace
```

### <system-information>System Information Collection</system-information>

**Collect comprehensive system information for support:**
```bash
# Generate diagnostic report
fortitude proactive maintenance generate-diagnostics --output diagnostics.json

# System information
fortitude proactive status --system-info --detailed

# Configuration dump
fortitude proactive configure export --effective --output current-config.json

# Recent logs
fortitude proactive logs --since 1h --output recent-logs.txt
```

### <safe-mode>Safe Mode Recovery</safe-mode-recovery>

**Start system in safe mode for troubleshooting:**
```bash
# Stop current system
fortitude proactive stop --force

# Start in safe mode (minimal configuration)
fortitude proactive start --safe-mode

# Safe mode characteristics:
# - Minimal resource usage
# - Basic gap detection only
# - Local notifications only
# - Reduced concurrent tasks
# - Verbose logging enabled
```

### <factory-reset>Factory Reset</factory-reset>

**Complete system reset (use as last resort):**
```bash
# WARNING: This will delete all configuration and data

# Stop system
fortitude proactive stop --force

# Backup current state (optional)
cp -r ~/.fortitude/ ~/.fortitude.backup/

# Remove all fortitude data
rm -rf ~/.fortitude/

# Reinitialize with defaults
fortitude proactive configure init

# Start fresh
fortitude proactive start
```

## <prevention>Prevention and Maintenance</prevention>

### <regular-maintenance>Regular Maintenance Tasks</regular-maintenance>

**Weekly maintenance checklist:**
```bash
# Check system health
fortitude proactive status --system-check

# Validate configuration
fortitude proactive configure validate --check-conflicts

# Clean old data
fortitude proactive maintenance clean-old-results --older-than 14d

# Check resource usage trends
fortitude proactive status --performance --historical
```

**Monthly maintenance:**
```bash
# Update to latest version (if available)
fortitude update

# Backup configuration
fortitude proactive configure export --output monthly-backup.json

# Performance optimization review
fortitude proactive maintenance optimize --dry-run

# Security review
fortitude proactive configure audit-security
```

### <monitoring-setup>Monitoring Setup</monitoring-setup>

**Set up automated monitoring:**
```bash
# Enable health check monitoring
fortitude proactive configure set monitoring.health_check_enabled true
fortitude proactive configure set monitoring.health_check_interval_minutes 5

# Configure alerts
fortitude proactive configure set monitoring.alerts.cpu_threshold 80
fortitude proactive configure set monitoring.alerts.memory_threshold 80
fortitude proactive configure set monitoring.alerts.error_rate_threshold 10

# Set up automated reporting
fortitude proactive configure set monitoring.daily_report_enabled true
fortitude proactive configure set monitoring.weekly_summary_enabled true
```

### <backup-strategy>Backup Strategy</backup-strategy>

**Automated backup setup:**
```bash
# Enable automatic configuration backups
fortitude proactive configure set backup.auto_backup_enabled true
fortitude proactive configure set backup.backup_interval_hours 24
fortitude proactive configure set backup.max_backups 7

# Manual backup
fortitude proactive maintenance backup --full --output backup-$(date +%Y%m%d).tar.gz

# Test backup restoration
fortitude proactive maintenance test-restore --backup backup-20250711.tar.gz --dry-run
```

---

**Need More Help?**

If you continue to experience issues after following this troubleshooting guide:

1. **Check the Logs**: Always start with `fortitude proactive logs --tail 50`
2. **Generate Diagnostics**: Use `fortitude proactive maintenance generate-diagnostics`
3. **Try Safe Mode**: Start with `fortitude proactive start --safe-mode`
4. **Reset Configuration**: Use `fortitude proactive configure reset --confirm`
5. **Community Support**: Check documentation and community forums
6. **Report Issues**: Create detailed issue reports with diagnostic information

**Related Guides:**
- [Configuration Guide](proactive-research-configuration.md) - Detailed configuration reference
- [CLI Reference](proactive-research-cli.md) - Complete command reference
- [API Guide](proactive-research-api.md) - HTTP API documentation
- [MCP Tools Guide](proactive-research-mcp.md) - Claude Code integration