#!/bin/bash

# CE-DPS Fortitude Integration Script
# Integrates Fortitude knowledge management with CE-DPS workflow

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

FORTITUDE_DIR="$(dirname "$0")/../fortitude"
PROJECT_DIR="$(pwd)"

echo -e "${BLUE}🧠 CE-DPS Fortitude Integration${NC}"
echo "============================================"

# Function to log with timestamp
log() {
    echo -e "$(date '+%Y-%m-%d %H:%M:%S') - $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check Fortitude installation
check_fortitude() {
    log "Checking Fortitude installation..."
    
    if [ ! -d "$FORTITUDE_DIR" ]; then
        echo -e "${RED}❌ Fortitude directory not found: $FORTITUDE_DIR${NC}"
        return 1
    fi
    
    cd "$FORTITUDE_DIR"
    
    if ! command_exists cargo; then
        echo -e "${RED}❌ Cargo not found - Rust installation required${NC}"
        return 1
    fi
    
    if [ ! -f "Cargo.toml" ]; then
        echo -e "${RED}❌ Fortitude Cargo.toml not found${NC}"
        return 1
    fi
    
    echo -e "${GREEN}✅ Fortitude installation verified${NC}"
    return 0
}

# Function to build Fortitude
build_fortitude() {
    log "Building Fortitude..."
    
    cd "$FORTITUDE_DIR"
    
    if cargo build --release; then
        echo -e "${GREEN}✅ Fortitude build successful${NC}"
    else
        echo -e "${RED}❌ Fortitude build failed${NC}"
        return 1
    fi
}

# Function to initialize Fortitude for CE-DPS
init_fortitude() {
    log "Initializing Fortitude for CE-DPS..."
    
    cd "$FORTITUDE_DIR"
    
    # Initialize the knowledge base
    if ./target/release/fortitude-cli init --project-type ce-dps; then
        echo -e "${GREEN}✅ Fortitude knowledge base initialized${NC}"
    else
        echo -e "${YELLOW}⚠️ Fortitude initialization may have failed - continuing...${NC}"
    fi
    
    # Configure for CE-DPS
    cat > config/ce-dps.toml << EOF
[fortitude]
name = "CE-DPS Knowledge Management"
description = "AI implementation pattern library"
version = "1.0.0"

[classification]
types = ["Decision", "Implementation", "Troubleshooting", "Learning", "Validation"]
default_type = "Implementation"

[gap_detection]
enabled = true
focus_areas = [
    "authentication_patterns",
    "database_patterns",
    "api_patterns",
    "testing_patterns",
    "quality_patterns"
]

[research_prioritization]
ai_implementation_focus = true
security_first = true
testing_comprehensive = true

[learning]
human_collaboration = true
pattern_recognition = true
continuous_improvement = true

[notifications]
channels = ["terminal", "log"]
delivery_verification = true
EOF
    
    echo -e "${GREEN}✅ CE-DPS configuration created${NC}"
}

# Function to start Fortitude services
start_fortitude() {
    log "Starting Fortitude services..."
    
    cd "$FORTITUDE_DIR"
    
    # Start the MCP server in background
    if ./target/release/fortitude-mcp-server --config config/ce-dps.toml &
    then
        echo -e "${GREEN}✅ Fortitude MCP server started${NC}"
        echo $! > .fortitude-mcp.pid
    else
        echo -e "${RED}❌ Failed to start Fortitude MCP server${NC}"
        return 1
    fi
    
    # Start proactive research
    if ./target/release/fortitude-cli proactive-start --project-path "$PROJECT_DIR" &
    then
        echo -e "${GREEN}✅ Fortitude proactive research started${NC}"
        echo $! > .fortitude-proactive.pid
    else
        echo -e "${RED}❌ Failed to start Fortitude proactive research${NC}"
        return 1
    fi
}

# Function to stop Fortitude services
stop_fortitude() {
    log "Stopping Fortitude services..."
    
    cd "$FORTITUDE_DIR"
    
    # Stop MCP server
    if [ -f .fortitude-mcp.pid ]; then
        if kill $(cat .fortitude-mcp.pid) 2>/dev/null; then
            echo -e "${GREEN}✅ Fortitude MCP server stopped${NC}"
        fi
        rm -f .fortitude-mcp.pid
    fi
    
    # Stop proactive research
    if [ -f .fortitude-proactive.pid ]; then
        if kill $(cat .fortitude-proactive.pid) 2>/dev/null; then
            echo -e "${GREEN}✅ Fortitude proactive research stopped${NC}"
        fi
        rm -f .fortitude-proactive.pid
    fi
}

# Function to check Fortitude status
status_fortitude() {
    log "Checking Fortitude status..."
    
    cd "$FORTITUDE_DIR"
    
    # Check MCP server
    if [ -f .fortitude-mcp.pid ] && kill -0 $(cat .fortitude-mcp.pid) 2>/dev/null; then
        echo -e "${GREEN}✅ Fortitude MCP server: RUNNING${NC}"
    else
        echo -e "${RED}❌ Fortitude MCP server: STOPPED${NC}"
    fi
    
    # Check proactive research
    if [ -f .fortitude-proactive.pid ] && kill -0 $(cat .fortitude-proactive.pid) 2>/dev/null; then
        echo -e "${GREEN}✅ Fortitude proactive research: RUNNING${NC}"
    else
        echo -e "${RED}❌ Fortitude proactive research: STOPPED${NC}"
    fi
}

# Function to update implementation patterns
update_patterns() {
    log "Updating implementation patterns..."
    
    cd "$FORTITUDE_DIR"
    
    if ./target/release/fortitude-cli update-patterns --project-path "$PROJECT_DIR"; then
        echo -e "${GREEN}✅ Implementation patterns updated${NC}"
    else
        echo -e "${RED}❌ Failed to update implementation patterns${NC}"
        return 1
    fi
}

# Function to query knowledge base
query_knowledge() {
    local query="$1"
    
    log "Querying knowledge base: $query"
    
    cd "$FORTITUDE_DIR"
    
    if ./target/release/fortitude-cli research-query --query "$query" --project-path "$PROJECT_DIR"; then
        echo -e "${GREEN}✅ Knowledge query completed${NC}"
    else
        echo -e "${RED}❌ Knowledge query failed${NC}"
        return 1
    fi
}

# Function to generate knowledge report
generate_report() {
    log "Generating knowledge report..."
    
    cd "$FORTITUDE_DIR"
    
    local report_file="$PROJECT_DIR/target/fortitude-report-$(date +%Y%m%d-%H%M%S).json"
    
    if ./target/release/fortitude-cli generate-report --output "$report_file" --project-path "$PROJECT_DIR"; then
        echo -e "${GREEN}✅ Knowledge report generated: $report_file${NC}"
    else
        echo -e "${RED}❌ Failed to generate knowledge report${NC}"
        return 1
    fi
}

# Function to setup Claude Code integration
setup_claude_integration() {
    log "Setting up Claude Code integration..."
    
    local claude_config="$HOME/.config/claude-code/mcp.json"
    
    if [ ! -f "$claude_config" ]; then
        mkdir -p "$(dirname "$claude_config")"
        cat > "$claude_config" << EOF
{
  "mcpServers": {
    "fortitude": {
      "command": "cargo",
      "args": ["run", "--bin", "fortitude-mcp-server", "--", "--config", "config/ce-dps.toml"],
      "cwd": "$FORTITUDE_DIR"
    }
  }
}
EOF
    else
        # Update existing configuration
        python3 -c "
import json
import sys

config_file = '$claude_config'
fortitude_config = {
    'command': 'cargo',
    'args': ['run', '--bin', 'fortitude-mcp-server', '--', '--config', 'config/ce-dps.toml'],
    'cwd': '$FORTITUDE_DIR'
}

try:
    with open(config_file, 'r') as f:
        config = json.load(f)
    
    if 'mcpServers' not in config:
        config['mcpServers'] = {}
    
    config['mcpServers']['fortitude'] = fortitude_config
    
    with open(config_file, 'w') as f:
        json.dump(config, f, indent=2)
    
    print('✅ Claude Code configuration updated')
except Exception as e:
    print(f'❌ Failed to update Claude Code configuration: {e}')
    sys.exit(1)
"
    fi
    
    echo -e "${GREEN}✅ Claude Code integration configured${NC}"
}

# Main command handling
case "${1:-help}" in
    "check")
        check_fortitude
        ;;
    "build")
        check_fortitude && build_fortitude
        ;;
    "init")
        check_fortitude && build_fortitude && init_fortitude
        ;;
    "start")
        check_fortitude && start_fortitude
        ;;
    "stop")
        stop_fortitude
        ;;
    "status")
        status_fortitude
        ;;
    "restart")
        stop_fortitude && sleep 2 && start_fortitude
        ;;
    "update")
        update_patterns
        ;;
    "query")
        if [ -z "$2" ]; then
            echo -e "${RED}❌ Usage: $0 query \"your query here\"${NC}"
            exit 1
        fi
        query_knowledge "$2"
        ;;
    "report")
        generate_report
        ;;
    "setup-claude")
        setup_claude_integration
        ;;
    "install")
        check_fortitude && build_fortitude && init_fortitude && setup_claude_integration
        echo -e "${GREEN}🎉 Fortitude integration setup complete!${NC}"
        echo "You can now use Fortitude with CE-DPS methodology."
        ;;
    "help"|*)
        echo -e "${BLUE}CE-DPS Fortitude Integration${NC}"
        echo "Usage: $0 <command>"
        echo ""
        echo "Commands:"
        echo "  check        - Check Fortitude installation"
        echo "  build        - Build Fortitude"
        echo "  init         - Initialize Fortitude for CE-DPS"
        echo "  start        - Start Fortitude services"
        echo "  stop         - Stop Fortitude services"
        echo "  status       - Check Fortitude service status"
        echo "  restart      - Restart Fortitude services"
        echo "  update       - Update implementation patterns"
        echo "  query <text> - Query knowledge base"
        echo "  report       - Generate knowledge report"
        echo "  setup-claude - Setup Claude Code integration"
        echo "  install      - Complete installation and setup"
        echo "  help         - Show this help message"
        ;;
esac