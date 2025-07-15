# Fortitude MCP Server Setup Guide

<meta>
  <title>Fortitude MCP Server Setup Guide</title>
  <type>setup_guide</type>
  <audience>developer</audience>
  <complexity>medium</complexity>
  <updated>2025-07-09</updated>
</meta>

## <summary priority="high">Quick Start</summary>

**Purpose**: Complete setup and configuration guide for the Fortitude MCP Server
**Output**: Running MCP server with Claude Code integration
**Time**: 15-30 minutes for basic setup

## <prerequisites>Prerequisites</prerequisites>

### <requirement>System Requirements</requirement>
- **Operating System**: Linux, macOS, or Windows 10+
- **Rust**: Version 1.70+ (install from [rustup.rs](https://rustup.rs/))
- **Memory**: Minimum 512MB RAM, recommended 2GB+
- **Storage**: 100MB for installation, 1GB+ for reference library
- **Network**: Internet connection for initial setup and API access

### <requirement>Development Dependencies</requirement>
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version

# Install additional tools (optional)
cargo install cargo-watch  # For development
```

## <installation>Installation Methods</installation>

### <method>Method 1: Install from Source (Recommended)</method>

<implementation>
```bash
# Clone the repository
git clone https://github.com/your-org/fortitude.git
cd fortitude

# Build the MCP server
cargo build --release -p fortitude-mcp-server

# Install binary (optional)
cargo install --path crates/fortitude-mcp-server

# Verify installation
fortitude-mcp-server --version
```
</implementation>

### <method>Method 2: Install from Crates.io</method>

<implementation>
```bash
# Install from crates.io
cargo install fortitude-mcp-server

# Verify installation
fortitude-mcp-server --version
```
</implementation>

### <method>Method 3: Download Pre-built Binary</method>

<implementation>
```bash
# Download for Linux x86_64
wget https://github.com/your-org/fortitude/releases/latest/download/fortitude-mcp-server-linux-x86_64.tar.gz
tar -xzf fortitude-mcp-server-linux-x86_64.tar.gz

# Make executable and move to PATH
chmod +x fortitude-mcp-server
sudo mv fortitude-mcp-server /usr/local/bin/

# Verify installation
fortitude-mcp-server --version
```
</implementation>

## <configuration>Configuration Setup</configuration>

### <config-method>Method 1: Configuration File (Recommended)</config-method>

<implementation>
Create a configuration file at `~/.config/fortitude/mcp-server.toml`:

```toml
[mcp_server]
host = "127.0.0.1"
port = 8080
max_connections = 1000
request_timeout = 30

[mcp_server.auth]
enabled = true
jwt_secret = "your-secret-key-here-minimum-32-characters-long"
token_expiration_hours = 24

[mcp_server.auth.rate_limit]
max_requests_per_minute = 60
window_seconds = 60

[mcp_server.logging]
level = "info"
structured = true
file_path = "./fortitude-mcp-server.log"

[mcp_server.performance]
cache_size = 1000
cache_ttl = 300
enable_deduplication = true

[mcp_server.integration]
fortitude_data_dir = "./reference_library"
enable_research_pipeline = true
enable_reference_library = true
enable_classification = true
classification_threshold = 0.7
```

**Create the configuration directory:**
```bash
mkdir -p ~/.config/fortitude
```
</implementation>

### <config-method>Method 2: Environment Variables</config-method>

<implementation>
```bash
# Server configuration
export MCP_SERVER_HOST=127.0.0.1
export MCP_SERVER_PORT=8080
export MCP_MAX_CONNECTIONS=1000
export MCP_REQUEST_TIMEOUT=30

# Authentication configuration
export MCP_AUTH_ENABLED=true
export MCP_JWT_SECRET="your-secret-key-here-minimum-32-characters-long"
export MCP_AUTH_TOKEN_EXPIRATION_HOURS=24

# Rate limiting
export MCP_RATE_LIMIT_MAX_REQUESTS=60
export MCP_RATE_LIMIT_WINDOW_SECONDS=60

# Logging configuration
export MCP_LOG_LEVEL=info
export MCP_LOG_STRUCTURED=true
export MCP_LOG_FILE_PATH="./fortitude-mcp-server.log"

# Performance configuration
export MCP_PERFORMANCE_CACHE_SIZE=1000
export MCP_PERFORMANCE_CACHE_TTL=300
export MCP_PERFORMANCE_ENABLE_DEDUPLICATION=true

# Integration configuration
export MCP_INTEGRATION_FORTITUDE_DATA_DIR="./reference_library"
export MCP_INTEGRATION_ENABLE_RESEARCH_PIPELINE=true
export MCP_INTEGRATION_ENABLE_REFERENCE_LIBRARY=true
export MCP_INTEGRATION_ENABLE_CLASSIFICATION=true
export MCP_INTEGRATION_CLASSIFICATION_THRESHOLD=0.7
```

**Persist environment variables:**
```bash
# Add to ~/.bashrc or ~/.zshrc
echo 'export MCP_JWT_SECRET="your-secret-key-here-minimum-32-characters-long"' >> ~/.bashrc
source ~/.bashrc
```
</implementation>

### <config-method>Method 3: Command Line Arguments</config-method>

<implementation>
```bash
# Start server with command line arguments
fortitude-mcp-server start \
  --host 127.0.0.1 \
  --port 8080 \
  --config ./custom-config.toml \
  --log-level info
```
</implementation>

## <first-time-setup>First-Time Setup Walkthrough</first-time-setup>

### <step>Step 1: Generate JWT Secret</step>

<implementation>
```bash
# Generate a secure JWT secret (32+ characters)
openssl rand -base64 32

# Or use a custom secret
export MCP_JWT_SECRET="your-custom-secret-key-minimum-32-characters-long"
```
</implementation>

### <step>Step 2: Initialize Data Directory</step>

<implementation>
```bash
# Create data directory
mkdir -p ./reference_library

# Download sample reference library (optional)
git clone https://github.com/your-org/fortitude-reference-library.git ./reference_library

# Or create minimal structure
mkdir -p ./reference_library/{research,patterns,quick-reference}
echo "# Reference Library" > ./reference_library/README.md
```
</implementation>

### <step>Step 3: Test Configuration</step>

<implementation>
```bash
# Validate configuration
fortitude-mcp-server validate-config --config ~/.config/fortitude/mcp-server.toml

# Expected output:
# ✓ Configuration validation successful
# ✓ JWT secret valid (32+ characters)
# ✓ Port 8080 available
# ✓ Data directory accessible
# ✓ All required dependencies found
```
</implementation>

### <step>Step 4: Start Server</step>

<implementation>
```bash
# Start server in foreground (for testing)
fortitude-mcp-server start --config ~/.config/fortitude/mcp-server.toml

# Expected output:
# 2025-07-09T12:00:00Z  INFO Starting MCP server on port 8080
# 2025-07-09T12:00:00Z  INFO Authentication enabled with JWT
# 2025-07-09T12:00:00Z  INFO Research pipeline initialized
# 2025-07-09T12:00:00Z  INFO Reference library loaded (1,234 files)
# 2025-07-09T12:00:00Z  INFO MCP server ready for connections

# Test server health
curl -i http://localhost:8080/health
```
</implementation>

### <step>Step 5: Generate Authentication Token</step>

<implementation>
```bash
# Generate initial authentication token
fortitude-mcp-server generate-token \
  --permissions "fortitude:research:read,fortitude:resources:read" \
  --expiration-hours 24

# Expected output:
# Generated JWT token:
# eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
# 
# Permissions: fortitude:research:read, fortitude:resources:read
# Expires: 2025-07-10T12:00:00Z
```
</implementation>

### <step>Step 6: Test MCP Tools</step>

<implementation>
```bash
# Test research_query tool
curl -X POST http://localhost:8080/mcp/tools/research_query \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "How to implement async functions in Rust?",
    "query_type": "implementation",
    "audience": "intermediate"
  }'

# Test classify_query tool
curl -X POST http://localhost:8080/mcp/tools/classify_query \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "How to debug a segfault in Rust?"
  }'
```
</implementation>

## <development-setup>Development Setup</development-setup>

### <dev-requirement>Development Dependencies</dev-requirement>

<implementation>
```bash
# Install development tools
cargo install cargo-watch
cargo install cargo-expand
cargo install criterion

# Install testing dependencies
cargo install cargo-tarpaulin  # Code coverage
```
</implementation>

### <dev-configuration>Development Configuration</dev-configuration>

<implementation>
Create `dev-config.toml`:
```toml
[mcp_server]
host = "127.0.0.1"
port = 8080
max_connections = 100
request_timeout = 30

[mcp_server.auth]
enabled = false  # Disable auth for development
jwt_secret = "development-secret-key-not-for-production"
token_expiration_hours = 1

[mcp_server.logging]
level = "debug"
structured = false
file_path = "./dev-server.log"

[mcp_server.performance]
cache_size = 100
cache_ttl = 60
enable_deduplication = false
```

Start with development configuration:
```bash
fortitude-mcp-server start --config dev-config.toml
```
</implementation>

### <dev-workflow>Development Workflow</dev-workflow>

<implementation>
```bash
# Run tests
cargo test -p fortitude-mcp-server

# Run with auto-reload
cargo watch -x 'run -p fortitude-mcp-server -- start --config dev-config.toml'

# Run benchmarks
cargo bench -p fortitude-mcp-server

# Generate documentation
cargo doc --open -p fortitude-mcp-server
```
</implementation>

## <production-setup>Production Setup</production-setup>

### <prod-configuration>Production Configuration</prod-configuration>

<implementation>
Create `production-config.toml`:
```toml
[mcp_server]
host = "0.0.0.0"
port = 8080
max_connections = 5000
request_timeout = 60

[mcp_server.auth]
enabled = true
jwt_secret = "${MCP_JWT_SECRET}"  # From environment
token_expiration_hours = 8

[mcp_server.auth.rate_limit]
max_requests_per_minute = 120
window_seconds = 60

[mcp_server.logging]
level = "info"
structured = true
file_path = "/var/log/fortitude-mcp-server.log"

[mcp_server.performance]
cache_size = 10000
cache_ttl = 600
enable_deduplication = true
max_concurrent_connections = 5000

[mcp_server.security]
force_https = true
enable_request_validation = true
max_request_size = 5242880  # 5MB
ip_whitelist = ["10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16"]
```
</implementation>

### <prod-deployment>Production Deployment</prod-deployment>

<implementation>
**System Service Setup (systemd):**
```bash
# Create service file
sudo tee /etc/systemd/system/fortitude-mcp-server.service > /dev/null <<EOF
[Unit]
Description=Fortitude MCP Server
After=network.target

[Service]
Type=simple
User=fortitude
Group=fortitude
WorkingDirectory=/opt/fortitude
ExecStart=/usr/local/bin/fortitude-mcp-server start --config /etc/fortitude/mcp-server.toml
Restart=always
RestartSec=10
Environment=MCP_JWT_SECRET=your-production-secret-key
Environment=MCP_LOG_LEVEL=info
Environment=MCP_LOG_FILE_PATH=/var/log/fortitude-mcp-server.log

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable fortitude-mcp-server
sudo systemctl start fortitude-mcp-server

# Check status
sudo systemctl status fortitude-mcp-server
```

**Docker Deployment:**
```bash
# Build Docker image
docker build -t fortitude-mcp-server .

# Run container
docker run -d \
  --name fortitude-mcp-server \
  -p 8080:8080 \
  -v /path/to/config:/etc/fortitude \
  -v /path/to/data:/opt/fortitude/reference_library \
  -e MCP_JWT_SECRET="your-production-secret-key" \
  fortitude-mcp-server
```
</implementation>

## <troubleshooting>Common Setup Issues</troubleshooting>

### <issue>Port Already in Use</issue>

<implementation>
```bash
# Check what's using the port
lsof -i :8080
netstat -tulpn | grep 8080

# Kill process using port
sudo kill -9 $(lsof -ti:8080)

# Or change port in configuration
export MCP_SERVER_PORT=8081
```
</implementation>

### <issue>Permission Denied</issue>

<implementation>
```bash
# Run with different user
sudo -u fortitude fortitude-mcp-server start

# Check file permissions
ls -la ~/.config/fortitude/
chmod 600 ~/.config/fortitude/mcp-server.toml

# Create user for production
sudo useradd -r -s /bin/false fortitude
sudo mkdir -p /opt/fortitude
sudo chown fortitude:fortitude /opt/fortitude
```
</implementation>

### <issue>JWT Secret Invalid</issue>

<implementation>
```bash
# Check secret length
echo -n "$MCP_JWT_SECRET" | wc -c
# Should be 32 or more characters

# Generate new secret
openssl rand -base64 32 > ~/.config/fortitude/jwt-secret
export MCP_JWT_SECRET=$(cat ~/.config/fortitude/jwt-secret)
```
</implementation>

### <issue>Data Directory Not Found</issue>

<implementation>
```bash
# Check directory exists
ls -la ./reference_library/

# Create directory structure
mkdir -p ./reference_library/{research,patterns,quick-reference}

# Set correct permissions
chmod -R 755 ./reference_library/
```
</implementation>

## <validation>Setup Validation</validation>

### <validation-checklist>Validation Checklist</validation-checklist>

<implementation>
Run the following commands to validate your setup:

```bash
# 1. Binary installation
fortitude-mcp-server --version

# 2. Configuration validation
fortitude-mcp-server validate-config

# 3. Data directory access
ls -la ./reference_library/

# 4. JWT secret validation
echo -n "$MCP_JWT_SECRET" | wc -c  # Should be 32+

# 5. Port availability
nc -zv localhost 8080

# 6. Service health
curl -i http://localhost:8080/health

# 7. Authentication test
fortitude-mcp-server generate-token --permissions "fortitude:research:read"

# 8. MCP tools test
curl -X POST http://localhost:8080/mcp/tools/classify_query \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query": "test query"}'
```

**Expected Results:**
- All commands should complete without errors
- Health check should return 200 OK
- Authentication token should be generated successfully
- MCP tools should respond with valid JSON
</implementation>

## <next-steps>Next Steps</next-steps>

After completing setup:

1. **Configure Claude Code Integration**: See [claude-integration.md](claude-integration.md)
2. **Review API Documentation**: See [api-reference.md](api-reference.md)
3. **Explore Usage Examples**: See [examples.md](examples.md)
4. **Set Up Monitoring**: See [performance.md](performance.md)
5. **Configure Production Security**: See [troubleshooting.md](troubleshooting.md)

## <support>Support</support>

For additional help:
- **Documentation**: Full API reference and examples
- **Issues**: GitHub issues for bug reports
- **Discussions**: Community forum for questions
- **Security**: Security contact for vulnerabilities

---

**Setup Complete**: Your Fortitude MCP Server is now ready for Claude Code integration and AI-powered research assistance.