# Fortitude MCP Server Configuration

This document describes the configuration system for the Fortitude MCP Server.

## Overview

The Fortitude MCP Server supports multiple configuration methods:
- Configuration files (JSON and TOML formats)
- Environment variables
- Command line arguments
- Automatic configuration discovery

## Configuration Files

### Supported Formats

The server supports both JSON and TOML configuration files. The format is detected automatically based on file extension:

- `.json` files are parsed as JSON
- `.toml` files are parsed as TOML  
- Files without extension default to JSON format

### Default Configuration Locations

The server searches for configuration files in the following order:

1. File specified with `--config` argument
2. `./fortitude-mcp-server.toml`
3. `./fortitude-mcp-server.json`
4. `~/.config/fortitude/mcp-server.toml`
5. `~/.config/fortitude/mcp-server.json`
6. Environment variables only

### Example Configuration (TOML)

```toml
[mcp_server]
host = "127.0.0.1"
port = 8080
max_connections = 1000
request_timeout = 30

[mcp_server.auth]
enabled = true
jwt_secret = "your-secret-key-here-minimum-32-characters"
token_expiration_hours = 24

[mcp_server.auth.rate_limit]
max_requests_per_minute = 60
window_seconds = 60

[mcp_server.logging]
level = "info"
structured = true
file_path = "/var/log/fortitude-mcp-server.log"

[mcp_server.performance]
cache_size = 1000
cache_ttl = 300
enable_deduplication = true
max_concurrent_connections = 1000
connection_timeout_seconds = 30
enable_http2 = true

[mcp_server.security]
allowed_origins = ["*"]
force_https = false
enable_request_validation = true
max_request_size = 1048576  # 1MB
ip_whitelist = []
enable_intrusion_detection = false

[mcp_server.security.security_headers]
x_frame_options = true
x_content_type_options = true
x_xss_protection = true
strict_transport_security = false
content_security_policy = "default-src 'self'"

[mcp_server.integration]
fortitude_data_dir = "./reference_library"
enable_research_pipeline = true
enable_reference_library = true
enable_classification = true
classification_threshold = 0.7
enable_context_detection = true
context_detection_timeout_ms = 1000
enable_research_caching = true
research_cache_ttl = 3600
```

## Environment Variables

All configuration options can be overridden using environment variables. Environment variables take precedence over configuration files.

### Server Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `MCP_SERVER_HOST` | Server host address | `127.0.0.1` |
| `MCP_SERVER_PORT` | Server port | `8080` |
| `MCP_MAX_CONNECTIONS` | Maximum concurrent connections | `1000` |
| `MCP_REQUEST_TIMEOUT` | Request timeout in seconds | `30` |

### Authentication Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `MCP_JWT_SECRET` | JWT secret key (minimum 32 characters) | Auto-generated |
| `MCP_AUTH_ENABLED` | Enable authentication | `true` |
| `MCP_AUTH_TOKEN_EXPIRATION_HOURS` | Token expiration in hours | `24` |

### Rate Limiting Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `MCP_RATE_LIMIT_MAX_REQUESTS` | Max requests per minute | `60` |
| `MCP_RATE_LIMIT_WINDOW_SECONDS` | Rate limit window in seconds | `60` |

### Logging Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `MCP_LOG_LEVEL` | Log level (trace, debug, info, warn, error) | `info` |
| `MCP_LOG_STRUCTURED` | Enable structured logging | `true` |
| `MCP_LOG_FILE_PATH` | Log file path | None |

### Performance Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `MCP_PERFORMANCE_CACHE_SIZE` | Cache size for responses | `1000` |
| `MCP_PERFORMANCE_CACHE_TTL` | Cache TTL in seconds | `300` |
| `MCP_PERFORMANCE_ENABLE_DEDUPLICATION` | Enable request deduplication | `true` |
| `MCP_PERFORMANCE_MAX_CONCURRENT_CONNECTIONS` | Max concurrent connections | `1000` |
| `MCP_PERFORMANCE_CONNECTION_TIMEOUT_SECONDS` | Connection timeout in seconds | `30` |
| `MCP_PERFORMANCE_ENABLE_HTTP2` | Enable HTTP/2 support | `true` |

### Security Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `MCP_SECURITY_ALLOWED_ORIGINS` | Allowed origins for CORS (comma-separated) | `*` |
| `MCP_SECURITY_FORCE_HTTPS` | Force HTTPS redirect | `false` |
| `MCP_SECURITY_ENABLE_REQUEST_VALIDATION` | Enable request validation | `true` |
| `MCP_SECURITY_MAX_REQUEST_SIZE` | Max request size in bytes | `1048576` |
| `MCP_SECURITY_IP_WHITELIST` | IP whitelist (comma-separated) | Empty |
| `MCP_SECURITY_ENABLE_INTRUSION_DETECTION` | Enable intrusion detection | `false` |

### Security Headers Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `MCP_SECURITY_X_FRAME_OPTIONS` | Enable X-Frame-Options header | `true` |
| `MCP_SECURITY_X_CONTENT_TYPE_OPTIONS` | Enable X-Content-Type-Options header | `true` |
| `MCP_SECURITY_X_XSS_PROTECTION` | Enable X-XSS-Protection header | `true` |
| `MCP_SECURITY_STRICT_TRANSPORT_SECURITY` | Enable Strict-Transport-Security header | `false` |
| `MCP_SECURITY_CONTENT_SECURITY_POLICY` | Content Security Policy | None |

### Integration Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `MCP_INTEGRATION_FORTITUDE_DATA_DIR` | Fortitude data directory | `./reference_library` |
| `MCP_INTEGRATION_ENABLE_RESEARCH_PIPELINE` | Enable research pipeline | `true` |
| `MCP_INTEGRATION_ENABLE_REFERENCE_LIBRARY` | Enable reference library | `true` |
| `MCP_INTEGRATION_ENABLE_CLASSIFICATION` | Enable classification system | `true` |
| `MCP_INTEGRATION_CLASSIFICATION_THRESHOLD` | Classification threshold (0.0-1.0) | `0.7` |
| `MCP_INTEGRATION_ENABLE_CONTEXT_DETECTION` | Enable context detection | `true` |
| `MCP_INTEGRATION_CONTEXT_DETECTION_TIMEOUT_MS` | Context detection timeout in ms | `1000` |
| `MCP_INTEGRATION_ENABLE_RESEARCH_CACHING` | Enable research caching | `true` |
| `MCP_INTEGRATION_RESEARCH_CACHE_TTL` | Research cache TTL in seconds | `3600` |

## CLI Commands

The MCP server provides several CLI commands for management:

### Start Server

```bash
# Start with default configuration
fortitude-mcp-server start

# Start with custom configuration
fortitude-mcp-server start --config config.toml

# Start with specific port and host
fortitude-mcp-server start --port 9090 --host 0.0.0.0

# Start in daemon mode (TODO: not yet implemented)
fortitude-mcp-server start --daemon
```

### Server Management

```bash
# Check server status
fortitude-mcp-server status

# Stop server
fortitude-mcp-server stop

# Force stop server  
fortitude-mcp-server stop --force
```

### Configuration Management

```bash
# Validate configuration
fortitude-mcp-server validate-config --config config.toml

# Generate sample configuration
fortitude-mcp-server generate-config --output config.toml --format toml

# Show configuration help
fortitude-mcp-server config-help

# Show current configuration
fortitude-mcp-server show-config --config config.toml
```

### Global Options

```bash
# Enable debug logging
fortitude-mcp-server --debug start

# Enable quiet mode
fortitude-mcp-server --quiet start

# Use specific configuration file
fortitude-mcp-server --config /path/to/config.toml start
```

## Configuration Validation

The server performs comprehensive validation of all configuration values:

### Validation Rules

1. **Port Numbers**: Must be between 1024 and 65535
2. **Connection Limits**: Must be positive integers
3. **Timeouts**: Must be positive integers
4. **JWT Secret**: Must be at least 32 characters when auth is enabled
5. **Classification Threshold**: Must be between 0.0 and 1.0
6. **Request Size**: Must be between 1KB and 100MB
7. **IP Addresses**: Must be valid IPv4/IPv6 addresses or CIDR notation
8. **File Paths**: Must be valid and accessible

### Error Messages

Configuration validation provides clear error messages:

```
Configuration validation failed: Port must be between 1024 and 65535, got 80
Configuration validation failed: JWT secret must be at least 32 characters when auth is enabled
Configuration validation failed: Invalid IP address or CIDR in whitelist: 300.300.300.300
```

## Integration with Fortitude Core

The MCP server integrates seamlessly with the Fortitude core system:

### Data Directory

The `fortitude_data_dir` setting points to the Fortitude reference library directory, enabling:
- Access to research documents
- Classification system integration
- Context detection capabilities

### Research Pipeline

When `enable_research_pipeline` is true, the MCP server provides:
- Research query processing
- Classification-based routing
- Context-aware responses

### Caching

The server supports multiple caching layers:
- Response caching (configurable TTL)
- Research result caching
- Classification result caching

## Production Deployment

### Recommended Settings

```toml
[mcp_server]
host = "0.0.0.0"
port = 8080
max_connections = 5000
request_timeout = 60

[mcp_server.auth]
enabled = true
jwt_secret = "your-production-secret-key-here"
token_expiration_hours = 8

[mcp_server.security]
force_https = true
enable_request_validation = true
max_request_size = 5242880  # 5MB
ip_whitelist = ["10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16"]
enable_intrusion_detection = true

[mcp_server.security.security_headers]
strict_transport_security = true
content_security_policy = "default-src 'self'; script-src 'self' 'unsafe-inline'"

[mcp_server.performance]
cache_size = 10000
cache_ttl = 600
enable_deduplication = true
max_concurrent_connections = 5000
```

### Environment Variables for Production

```bash
export MCP_SERVER_HOST=0.0.0.0
export MCP_SERVER_PORT=8080
export MCP_JWT_SECRET="your-production-secret-key-here"
export MCP_AUTH_ENABLED=true
export MCP_SECURITY_FORCE_HTTPS=true
export MCP_SECURITY_ENABLE_INTRUSION_DETECTION=true
export MCP_LOG_LEVEL=info
export MCP_LOG_STRUCTURED=true
export MCP_LOG_FILE_PATH=/var/log/fortitude-mcp-server.log
```

## Troubleshooting

### Common Issues

1. **Port Already in Use**: Change the port or stop the conflicting service
2. **Permission Denied**: Ensure the user has permission to bind to the port
3. **Invalid JWT Secret**: Ensure the secret is at least 32 characters
4. **Configuration File Not Found**: Check the file path and permissions
5. **Data Directory Access**: Ensure the fortitude data directory is accessible

### Debug Mode

Enable debug mode for detailed logging:

```bash
fortitude-mcp-server --debug start
```

Or set environment variable:

```bash
export MCP_LOG_LEVEL=debug
fortitude-mcp-server start
```

This provides detailed information about:
- Configuration loading process
- Request processing
- Authentication flow
- Error details