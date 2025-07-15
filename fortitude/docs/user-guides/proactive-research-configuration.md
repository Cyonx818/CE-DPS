# Proactive Research Configuration Guide

<meta>
  <title>Proactive Research Configuration - Complete Guide</title>
  <type>user_guide</type>
  <audience>developer</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-11</updated>
  <sprint>008</sprint>
  <version>1.0</version>
</meta>

## <summary priority="high">Configuration Overview</summary>
- **Purpose**: Complete configuration reference for proactive research system
- **Configuration Sources**: Files, environment variables, CLI arguments, and presets
- **Hot Reload**: Runtime configuration updates without restart
- **Validation**: Comprehensive configuration validation with helpful error messages
- **Migration**: Automatic configuration version migration support

## <overview>Configuration Architecture</overview>

The proactive research system uses a hierarchical configuration system with multiple sources and precedence rules:

### <precedence>Configuration Precedence (highest to lowest)</precedence>

1. **CLI Arguments** - Runtime overrides
2. **Environment Variables** - System-level configuration  
3. **Project Configuration** - `.fortitude_config.json` in project root
4. **User Configuration** - `~/.fortitude/config.toml` 
5. **System Defaults** - Built-in fallback values

### <configuration-sections>Main Configuration Sections</configuration-sections>

- **Gap Analysis** - Detection rules, intervals, and thresholds
- **Background Research** - Task management, rate limiting, quality control
- **Notifications** - Delivery channels, rules, and throttling
- **Performance** - Resource limits, monitoring, optimization
- **User Preferences** - Personalization and interface settings
- **Workspace** - Project-specific overrides and integrations

## <file-formats>Configuration File Formats</file-formats>

### <json-config>JSON Configuration (Recommended)</json-config>

**Project Config (`.fortitude_config.json`):**
```json
{
  "version": "2.0",
  "metadata": {
    "name": "My Project Configuration",
    "description": "Optimized for web development with React",
    "target_environment": "development"
  },
  "gap_analysis": {
    "scan_intervals_seconds": 120,
    "file_patterns": ["*.js", "*.jsx", "*.ts", "*.tsx", "*.md"],
    "detection_rules": ["todo", "fixme", "hack", "bug", "deprecated"],
    "confidence_threshold": 0.75,
    "enable_semantic_analysis": true,
    "max_files_per_scan": 500,
    "priority_file_types": {
      "tsx": 1.3,
      "ts": 1.2,
      "js": 1.0,
      "md": 0.8
    },
    "custom_rules": [
      {
        "name": "API Documentation Gap",
        "description": "Detect undocumented API endpoints",
        "pattern": "app\\.(get|post|put|delete)\\([^)]+\\)(?!.*\\/\\*\\*)",
        "priority": 8,
        "enabled": true,
        "confidence_boost": 0.3
      }
    ]
  },
  "background_research": {
    "max_concurrent_tasks": 4,
    "rate_limit_requests_per_minute": 60,
    "scheduling_enabled": true,
    "priority_keywords": ["urgent", "critical", "security", "performance", "accessibility"],
    "research_timeout_seconds": 240,
    "auto_prioritization_enabled": true,
    "quality_thresholds": {
      "min_confidence": 0.7,
      "min_sources": 3,
      "enable_validation": true,
      "retry_failed_research": true,
      "max_retry_attempts": 2
    }
  },
  "notifications": {
    "channels": ["desktop", "console"],
    "delivery_preferences": {
      "desktop": true,
      "console": true,
      "email": false
    },
    "delivery_rules": [
      {
        "name": "High Priority Immediate",
        "condition": "priority >= 8",
        "channels": ["desktop"],
        "priority": 10,
        "enabled": true,
        "throttling": {
          "min_interval_seconds": 60,
          "enable_deduplication": true,
          "deduplication_window_seconds": 300
        }
      }
    ],
    "rate_limiting": {
      "max_per_hour": 15,
      "max_per_day": 100,
      "enable_burst_protection": true,
      "burst_threshold": 5
    }
  },
  "performance": {
    "resource_limits": {
      "max_memory_mb": 1024,
      "max_cpu_percent": 75,
      "max_disk_mb": 5120,
      "max_network_mbps": 50
    },
    "monitoring_enabled": true,
    "alert_thresholds": {
      "cpu_usage": 70.0,
      "memory_usage": 80.0,
      "response_time_ms": 800.0
    },
    "caching": {
      "enabled": true,
      "size_limit_mb": 256,
      "ttl_seconds": 1800,
      "eviction_policy": "lru",
      "enable_compression": true
    }
  },
  "workspace": {
    "project_paths": ["src/", "docs/", "tests/"],
    "exclude_patterns": [
      "node_modules/**",
      "build/**",
      "dist/**",
      "*.min.js",
      "*.test.js"
    ],
    "auto_discovery_enabled": true,
    "project_type_detection": {
      "auto_detection_enabled": true,
      "type_indicators": {
        "react": ["package.json", "src/App.jsx", "public/index.html"],
        "node": ["package.json", "server.js", "app.js"]
      }
    }
  }
}
```

### <toml-config>TOML Configuration</toml-config>

**User Config (`~/.fortitude/config.toml`):**
```toml
[metadata]
name = "Personal Development Configuration"
target_environment = "development"

[proactive.gap_analysis]
scan_intervals_seconds = 300
confidence_threshold = 0.7
enable_semantic_analysis = true

[proactive.background_research]
max_concurrent_tasks = 3
rate_limit_requests_per_minute = 50
auto_prioritization_enabled = true

[proactive.notifications]
channels = ["desktop", "console"]
frequency_hours = 2

[proactive.performance]
monitoring_enabled = false

[proactive.performance.resource_limits]
max_memory_mb = 2048
max_cpu_percent = 85

[proactive.user_preferences]
research_domains = ["software_development", "web_development", "documentation"]
preferred_formats = ["markdown", "json"]
notification_frequency_hours = 4

[proactive.user_preferences.personalization]
enable_adaptive_learning = true
learning_rate = 0.15
enable_preference_learning = true
```

### <yaml-config>YAML Configuration</yaml-config>

**Environment Config (`config.yaml`):**
```yaml
version: "2.0"
metadata:
  name: "Production Environment Configuration"
  target_environment: "production"

gap_analysis:
  scan_intervals_seconds: 600
  confidence_threshold: 0.8
  enable_semantic_analysis: false
  max_files_per_scan: 2000

background_research:
  max_concurrent_tasks: 2
  rate_limit_requests_per_minute: 30
  research_timeout_seconds: 180

notifications:
  channels: ["email", "webhook"]
  rate_limiting:
    max_per_hour: 5
    max_per_day: 25

performance:
  monitoring_enabled: true
  resource_limits:
    max_memory_mb: 4096
    max_cpu_percent: 60
    max_disk_mb: 20480
```

## <presets>Configuration Presets</presets>

### <development-preset>Development Preset</development-preset>

**Use Case:** Active development with frequent feedback
```bash
fortitude proactive configure preset development
```

**Key Settings:**
- Scan interval: 60 seconds (frequent)
- Desktop notifications: Enabled
- CPU limit: 90% (relaxed)
- Monitoring: Disabled (less overhead)
- Semantic analysis: Enabled
- Concurrency: 3-5 tasks

**When to Use:**
- Local development environment
- Quick feedback loops needed
- Performance monitoring not critical
- Immediate notification of gaps

### <production-preset>Production Preset</production-preset>

**Use Case:** Stable production environment
```bash
fortitude proactive configure preset production
```

**Key Settings:**
- Scan interval: 600 seconds (conservative)
- Email/webhook notifications: Enabled
- CPU limit: 70% (conservative)
- Monitoring: Full monitoring enabled
- Rate limiting: 30 requests/minute
- Concurrency: 1-2 tasks

**When to Use:**
- Production servers
- CI/CD environments
- Resource conservation important
- Audit trail required

### <research-preset>Research Preset</research-preset>

**Use Case:** Intensive research and analysis
```bash
fortitude proactive configure preset research
```

**Key Settings:**
- Scan interval: 120 seconds (frequent)
- High concurrency: 5-8 tasks
- Advanced semantic analysis: Enabled
- Lower confidence threshold: 0.6
- Extended research timeout: 600 seconds
- Memory: 4GB allocated

**When to Use:**
- Research-focused projects
- Documentation generation
- Comprehensive gap analysis
- Performance not a constraint

### <minimal-preset>Minimal Preset</minimal-preset>

**Use Case:** Resource-constrained environments
```bash
fortitude proactive configure preset minimal
```

**Key Settings:**
- Scan interval: 600 seconds (slow)
- Single-threaded: 1 task
- Memory limit: 512MB
- CPU limit: 50%
- Notifications: Desktop only
- Monitoring: Disabled

**When to Use:**
- Low-resource environments
- Background operation
- Minimal intrusion required
- Limited network bandwidth

## <environment-variables>Environment Variables</environment-variables>

### <common-variables>Common Environment Variables</common-variables>

```bash
# Core Configuration
export FORTITUDE_ENVIRONMENT="development"
export FORTITUDE_CONFIG_FILE="/path/to/config.json"

# Gap Analysis Settings
export FORTITUDE_PROACTIVE_GAP_SCAN_INTERVAL="300"
export FORTITUDE_PROACTIVE_CONFIDENCE_THRESHOLD="0.7"

# Background Research Settings
export FORTITUDE_PROACTIVE_MAX_CONCURRENT_TASKS="3"
export FORTITUDE_PROACTIVE_RATE_LIMIT_RPM="50"
export FORTITUDE_PROACTIVE_RESEARCH_TIMEOUT="300"

# Notification Settings
export FORTITUDE_PROACTIVE_NOTIFICATION_CHANNELS="desktop,console"
export FORTITUDE_PROACTIVE_QUIET_HOURS_START="22:00"
export FORTITUDE_PROACTIVE_QUIET_HOURS_END="08:00"

# Performance Settings
export FORTITUDE_PROACTIVE_MAX_MEMORY_MB="2048"
export FORTITUDE_PROACTIVE_MAX_CPU_PERCENT="80"

# API and Integration Settings
export FORTITUDE_API_KEY="your-api-key"
export FORTITUDE_API_BASE_URL="http://localhost:8080"
export FORTITUDE_CLAUDE_API_KEY="your-claude-key"

# Workspace Settings
export FORTITUDE_PROACTIVE_WATCH_PATHS="src,docs,tests"
export FORTITUDE_PROACTIVE_IGNORE_PATTERNS="target,node_modules,.git"
```

### <docker-variables>Docker Environment Variables</docker-variables>

```bash
# Docker-specific settings
export FORTITUDE_CONTAINER_MODE="true"
export FORTITUDE_PROACTIVE_ENABLE_FILE_MONITORING="true"
export FORTITUDE_PROACTIVE_HOST_MOUNT_PATH="/host/project"
export FORTITUDE_NOTIFICATION_DOCKER_SOCKET="/var/run/docker.sock"
```

## <cli-configuration>CLI Configuration Commands</cli-configuration>

### <set-commands>Setting Configuration Values</set-commands>

```bash
# Gap analysis configuration
fortitude proactive configure set gap_analysis.scan_intervals_seconds 180
fortitude proactive configure set gap_analysis.confidence_threshold 0.8
fortitude proactive configure set gap_analysis.enable_semantic_analysis true

# Background research configuration
fortitude proactive configure set background_research.max_concurrent_tasks 4
fortitude proactive configure set background_research.rate_limit_requests_per_minute 60
fortitude proactive configure set background_research.auto_prioritization_enabled true

# Notification configuration
fortitude proactive configure set notifications.channels "desktop,email"
fortitude proactive configure set notifications.rate_limiting.max_per_hour 20

# Performance configuration
fortitude proactive configure set performance.resource_limits.max_memory_mb 3072
fortitude proactive configure set performance.monitoring_enabled true

# Workspace configuration
fortitude proactive configure set workspace.project_paths "src,docs,tests,examples"
fortitude proactive configure set workspace.exclude_patterns "target,node_modules,dist"
```

### <get-commands>Getting Configuration Values</get-commands>

```bash
# View entire configuration
fortitude proactive configure show

# View specific sections
fortitude proactive configure show gap_analysis
fortitude proactive configure show notifications

# View specific values
fortitude proactive configure get gap_analysis.scan_intervals_seconds
fortitude proactive configure get background_research.max_concurrent_tasks

# View configuration with sources
fortitude proactive configure show --sources

# Export configuration
fortitude proactive configure export --format json --output my-config.json
fortitude proactive configure export --format yaml --output my-config.yaml
```

### <list-commands>Listing Configuration Options</list-commands>

```bash
# List all available configuration keys
fortitude proactive configure list

# List keys for specific section
fortitude proactive configure list gap_analysis
fortitude proactive configure list notifications

# List with descriptions and defaults
fortitude proactive configure list --detailed

# List only user-modifiable settings
fortitude proactive configure list --user-configurable
```

### <reset-commands>Resetting Configuration</reset-commands>

```bash
# Reset entire configuration to defaults
fortitude proactive configure reset --confirm

# Reset specific sections
fortitude proactive configure reset gap_analysis
fortitude proactive configure reset notifications

# Reset to specific preset
fortitude proactive configure reset --preset development
fortitude proactive configure reset --preset production

# Reset user preferences only
fortitude proactive configure reset user_preferences
```

## <validation>Configuration Validation</validation>

### <built-in-validation>Built-in Validation Rules</built-in-validation>

The system automatically validates:

- **Range Constraints**: Numeric values within specified ranges
- **Required Fields**: Essential configuration fields are present
- **Type Validation**: Correct data types for all fields
- **Pattern Validation**: Regular expression patterns for custom rules
- **Cross-Field Validation**: Logical consistency between related settings

### <custom-validation>Custom Validation Examples</custom-validation>

```bash
# Validate current configuration
fortitude proactive configure validate

# Validate specific configuration file
fortitude proactive configure validate --config /path/to/config.json

# Validate configuration with detailed output
fortitude proactive configure validate --detailed

# Check for conflicts between settings
fortitude proactive configure validate --check-conflicts
```

**Common Validation Errors:**

1. **Resource Conflicts:**
   ```
   Error: High scan frequency and concurrency require higher CPU limits
   Current: scan_interval=30s, max_tasks=8, cpu_limit=60%
   Suggestion: Increase cpu_limit to 80% or reduce scan frequency
   ```

2. **Memory Estimation:**
   ```
   Warning: Concurrent tasks may exceed memory limits
   Estimated: 800MB needed, 512MB available
   Suggestion: Reduce max_concurrent_tasks or increase max_memory_mb
   ```

3. **Rate Limiting:**
   ```
   Error: Rate limit too low for configured scan frequency
   Current: 10 RPM limit with 30s scan interval
   Suggestion: Increase rate_limit_requests_per_minute to at least 20
   ```

## <hot-reload>Hot Configuration Reload</hot-reload>

### <enabling-hot-reload>Enabling Hot Reload</enabling-hot-reload>

```bash
# Start proactive research with hot reload enabled
fortitude proactive start --hot-reload --config project-config.json

# Enable hot reload for running system
fortitude proactive configure hot-reload enable --config project-config.json
```

### <hot-reload-behavior>Hot Reload Behavior</hot-reload-behavior>

**What Gets Reloaded Immediately:**
- Gap detection rules and intervals
- Notification preferences and channels
- Performance limits and thresholds
- User preferences and personalization

**What Requires Restart:**
- Workspace project paths
- File monitoring patterns
- Integration settings (API keys, endpoints)
- Core system architecture changes

**Validation During Hot Reload:**
```bash
# Monitor hot reload events
fortitude proactive status --follow-config-changes

# Validate configuration before applying
fortitude proactive configure validate --on-change
```

## <migration>Configuration Migration</migration>

### <version-migration>Automatic Version Migration</version-migration>

The system automatically migrates older configuration versions:

```bash
# Load configuration with automatic migration
fortitude proactive configure load --migrate config.json

# Check configuration version
fortitude proactive configure version

# Manually migrate configuration file
fortitude proactive configure migrate --from 1.0 --to 2.0 config.json
```

**Migration Path:**
- **Version 1.0 → 2.0**: Adds metadata section, restructures notification rules
- **Version 2.0 → Current**: No migration needed

## <troubleshooting>Configuration Troubleshooting</troubleshooting>

### <common-issues>Common Configuration Issues</common-issues>

**Issue 1: Configuration Not Loading**
```bash
# Check configuration file syntax
fortitude proactive configure validate config.json

# Check file permissions
ls -la ~/.fortitude/config.toml

# Test with minimal configuration
fortitude proactive configure preset minimal
```

**Issue 2: Performance Issues**
```bash
# Check current resource usage
fortitude proactive status --system-check

# Reduce resource consumption
fortitude proactive configure set performance.resource_limits.max_cpu_percent 50
fortitude proactive configure set background_research.max_concurrent_tasks 1
```

**Issue 3: Too Many/Few Notifications**
```bash
# Adjust notification frequency
fortitude proactive configure set notifications.rate_limiting.max_per_hour 5

# Configure quiet hours
fortitude proactive configure set notifications.quiet_hours.enabled true
fortitude proactive configure set notifications.quiet_hours.start "22:00"
fortitude proactive configure set notifications.quiet_hours.end "08:00"
```

### <debugging-configuration>Configuration Debugging</debugging-configuration>

```bash
# Debug configuration loading
fortitude proactive configure debug --trace-loading

# Show effective configuration (with all sources merged)
fortitude proactive configure show --effective --sources

# Test configuration changes without applying
fortitude proactive configure test-change gap_analysis.scan_intervals_seconds 60

# Export effective configuration for debugging
fortitude proactive configure export --effective --format json --output debug-config.json
```

## <advanced-features>Advanced Configuration Features</advanced-features>

### <conditional-configuration>Conditional Configuration</conditional-configuration>

```json
{
  "environment_overrides": {
    "development": {
      "gap_analysis": {"scan_intervals_seconds": 60},
      "performance": {"monitoring_enabled": false}
    },
    "production": {
      "gap_analysis": {"scan_intervals_seconds": 600},
      "performance": {"monitoring_enabled": true}
    },
    "ci": {
      "background_research": {"max_concurrent_tasks": 1},
      "notifications": {"channels": ["webhook"]}
    }
  }
}
```

### <project-type-specific>Project Type-Specific Configuration</project-type-specific>

```json
{
  "workspace": {
    "project_type_detection": {
      "auto_detection_enabled": true,
      "custom_types": [
        {
          "name": "react_app",
          "patterns": ["package.json", "src/App.jsx"],
          "config_overrides": {
            "gap_analysis": {
              "file_patterns": ["*.jsx", "*.tsx", "*.css", "*.scss"],
              "priority_file_types": {"jsx": 1.5, "tsx": 1.8}
            }
          }
        },
        {
          "name": "rust_project",
          "patterns": ["Cargo.toml", "src/main.rs"],
          "config_overrides": {
            "gap_analysis": {
              "file_patterns": ["*.rs", "*.toml"],
              "custom_rules": [
                {
                  "name": "Missing Error Handling",
                  "pattern": "\\.unwrap\\(\\)|panic!",
                  "priority": 9
                }
              ]
            }
          }
        }
      ]
    }
  }
}
```

### <integration-specific-config>Integration-Specific Configuration</integration-specific-config>

```json
{
  "background_research": {
    "integration": {
      "enable_claude_integration": true,
      "enable_vector_integration": true,
      "api_rate_limits": {
        "claude": 40,
        "vector": 80,
        "github": 5000
      },
      "integration_timeouts": {
        "claude": 45,
        "vector": 15,
        "webhook": 30
      }
    }
  }
}
```

## <best-practices>Configuration Best Practices</best-practices>

### <security-practices>Security Best Practices</security-practices>

1. **API Keys**: Never store API keys in configuration files
   ```bash
   # Use environment variables instead
   export FORTITUDE_CLAUDE_API_KEY="your-key"
   
   # Or use secure key management
   fortitude proactive configure set-secret claude_api_key
   ```

2. **File Permissions**: Secure configuration files
   ```bash
   chmod 600 ~/.fortitude/config.toml
   chmod 600 .fortitude_config.json
   ```

3. **Version Control**: Exclude sensitive configurations
   ```gitignore
   .fortitude_config.json
   .env.local
   fortitude-secrets.json
   ```

### <performance-practices>Performance Best Practices</performance-practices>

1. **Resource Allocation**: Match configuration to system capabilities
   ```json
   {
     "performance": {
       "resource_limits": {
         "max_memory_mb": 1024,  // 25% of 4GB system
         "max_cpu_percent": 60   // Leave room for other processes
       }
     }
   }
   ```

2. **Scan Optimization**: Balance frequency vs. performance
   ```json
   {
     "gap_analysis": {
       "scan_intervals_seconds": 300,  // 5 minutes for most projects
       "max_files_per_scan": 1000      // Limit per scan
     }
   }
   ```

3. **Notification Throttling**: Prevent notification spam
   ```json
   {
     "notifications": {
       "rate_limiting": {
         "max_per_hour": 10,
         "enable_burst_protection": true
       }
     }
   }
   ```

### <maintenance-practices>Maintenance Best Practices</maintenance-practices>

1. **Regular Validation**: Validate configuration periodically
   ```bash
   # Add to cron or CI
   fortitude proactive configure validate --check-conflicts
   ```

2. **Configuration Backup**: Back up working configurations
   ```bash
   # Export current configuration
   fortitude proactive configure export --format json --output backup-$(date +%Y%m%d).json
   ```

3. **Performance Monitoring**: Monitor configuration effectiveness
   ```bash
   # Check system performance
   fortitude proactive status --performance-metrics
   ```

---

**Next Steps:**
- [CLI Usage Guide](proactive-research-cli.md) - Complete command reference
- [API Integration Guide](proactive-research-api.md) - HTTP API documentation
- [Troubleshooting Guide](proactive-research-troubleshooting.md) - Common issues and solutions
- [Workflow Examples](proactive-research-workflows.md) - Real-world usage patterns