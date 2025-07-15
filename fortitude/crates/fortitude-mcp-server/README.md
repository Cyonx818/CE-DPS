# Fortitude MCP Server

<meta>
  <title>Fortitude MCP Server</title>
  <type>documentation</type>
  <audience>developers</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-09</updated>
</meta>

## <summary priority="high">Overview</summary>

The Fortitude MCP Server provides seamless integration between Claude Code and the Fortitude AI research pipeline through the Model Context Protocol (MCP). It exposes Fortitude's research capabilities as MCP tools and resources, enabling AI assistants to conduct automated research and access reference library materials.

## <features>Key Features</features>

- **Research Tools**: Access Fortitude's research pipeline through MCP tools
- **Reference Library**: Browse and access documentation via MCP resources
- **Authentication**: JWT-based security with permission management
- **Performance**: 100+ concurrent requests with sub-100ms latency
- **Configuration**: Comprehensive configuration management with environment variables

## <quickstart>Quick Start</quickstart>

### Installation

```bash
# Build the MCP server
cargo build -p fortitude-mcp-server

# Run with default configuration
cargo run -p fortitude-mcp-server start

# Show CLI help
cargo run -p fortitude-mcp-server -- --help
```

### Basic Configuration

Create `config.toml`:

```toml
[server]
host = "127.0.0.1"
port = 8080

[auth]
enabled = true
jwt_secret = "your-secret-here"
token_expiry_hours = 24

[rate_limiting]
max_requests_per_minute = 60
```

Start the server:

```bash
cargo run -p fortitude-mcp-server start --config config.toml
```

## <integration>Claude Code Integration</integration>

### Configure MCP in Claude Code

Add to your Claude Code MCP configuration:

```json
{
  "mcpServers": {
    "fortitude": {
      "command": "cargo",
      "args": ["run", "-p", "fortitude-mcp-server", "start"],
      "env": {
        "MCP_SERVER_HOST": "127.0.0.1",
        "MCP_SERVER_PORT": "8080",
        "MCP_AUTH_ENABLED": "false"
      }
    }
  }
}
```

### Generate Authentication Token

```bash
# Generate a development token
cargo run -p fortitude-mcp-server generate-token --user claude --permissions research:read,resources:read
```

## <tools>Available Tools</tools>

### research_query
Execute research queries using the Fortitude pipeline:

```json
{
  "query": "Rust async programming patterns",
  "query_type": "technical",
  "audience": "developer",
  "domain": "rust"
}
```

### classify_query
Classify research queries for better targeting:

```json
{
  "query": "How to implement JWT authentication in Rust?"
}
```

### detect_context
Detect context dimensions from queries:

```json
{
  "query": "urgent security vulnerability in production system"
}
```

## <resources>Available Resources</resources>

### Reference Library Files
- URI: `mcp://fortitude/docs/reference_library/{path}`
- Access documentation files and guides

### Cache Statistics
- URI: `mcp://fortitude/cache/statistics`
- View cache performance metrics

### Configuration State
- URI: `mcp://fortitude/config/current`
- Current server configuration (sanitized)

## <configuration>Configuration</configuration>

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `MCP_SERVER_HOST` | Server host address | `127.0.0.1` |
| `MCP_SERVER_PORT` | Server port | `8080` |
| `MCP_AUTH_ENABLED` | Enable authentication | `true` |
| `MCP_JWT_SECRET` | JWT secret key | Required |
| `MCP_RATE_LIMIT_MAX_REQUESTS` | Rate limit per minute | `60` |

See [CONFIG.md](CONFIG.md) for complete configuration reference.

## <security>Security</security>

### Authentication
- JWT-based authentication with configurable expiry
- Permission-based authorization (read/write/admin)
- Rate limiting to prevent abuse

### Input Validation
- Comprehensive input sanitization
- SQL injection prevention
- XSS protection
- Path traversal protection

### Security Headers
- X-Frame-Options, X-Content-Type-Options
- X-XSS-Protection, Content-Security-Policy
- Optional HTTPS enforcement

## <performance>Performance</performance>

### Benchmarks
- **Concurrent Requests**: 100+ simultaneous connections
- **Latency**: Sub-100ms response times
- **Throughput**: 1000+ requests per minute
- **Memory**: Efficient resource usage with caching

### Optimization
- Response caching with configurable TTL
- Connection pooling for external services
- Request deduplication for expensive operations
- Async processing throughout

## <cli>CLI Commands</cli>

```bash
# Start the server
fortitude-mcp-server start --config config.toml

# Validate configuration
fortitude-mcp-server validate-config --config config.toml

# Generate sample config
fortitude-mcp-server generate-config --output sample-config.toml

# Show current configuration
fortitude-mcp-server show-config

# Server status
fortitude-mcp-server status
```

## <testing>Testing</testing>

Run the comprehensive test suite:

```bash
# All tests
cargo test -p fortitude-mcp-server

# Integration tests only
cargo test -p fortitude-mcp-server --test '*'

# Performance tests (marked with #[ignore])
cargo test -p fortitude-mcp-server -- --ignored performance

# Security tests
cargo test -p fortitude-mcp-server security
```

## <development>Development</development>

### Building

```bash
# Build debug version
cargo build -p fortitude-mcp-server

# Build release version
cargo build -p fortitude-mcp-server --release
```

### Development Mode

```bash
# Run with debug logging
MCP_LOG_LEVEL=debug cargo run -p fortitude-mcp-server start

# Disable authentication for development
MCP_AUTH_ENABLED=false cargo run -p fortitude-mcp-server start
```

## <architecture>Architecture</architecture>

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Claude Code   │───▶│  MCP Protocol   │───▶│ Fortitude MCP   │
│                 │    │                 │    │    Server       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                       │
                                                       ▼
                                              ┌─────────────────┐
                                              │ Fortitude Core  │
                                              │ Research        │
                                              │ Pipeline        │
                                              └─────────────────┘
```

### Components
- **MCP Server**: Protocol implementation and request handling
- **Tools**: Research query, classification, and context detection
- **Resources**: Reference library and system state access
- **Auth**: JWT authentication and permission management
- **Config**: Comprehensive configuration management

## <troubleshooting>Troubleshooting</troubleshooting>

### Common Issues

**Connection Refused**
```bash
# Check if server is running
cargo run -p fortitude-mcp-server status

# Check port availability
netstat -an | grep 8080
```

**Authentication Errors**
```bash
# Verify JWT secret is set
echo $MCP_JWT_SECRET

# Check token permissions
cargo run -p fortitude-mcp-server verify-token --token <token>
```

**Rate Limiting**
```bash
# Check current rate limits
cargo run -p fortitude-mcp-server show-config | grep rate_limiting

# Adjust rate limits
export MCP_RATE_LIMIT_MAX_REQUESTS=120
```

## <support>Support</support>

- **Documentation**: See `docs/` directory for detailed guides
- **Configuration**: See [CONFIG.md](CONFIG.md) for all options
- **API Reference**: See [docs/api-reference.md](docs/api-reference.md)
- **Performance**: See [docs/performance.md](docs/performance.md)

## <license>License</license>

Licensed under the same terms as the Fortitude project.