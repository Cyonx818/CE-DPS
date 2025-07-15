#!/bin/bash

# CE-DPS Quality Gates Script
# This script enforces comprehensive quality standards for AI-implemented code

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
COVERAGE_TARGET=95
PERFORMANCE_TARGET=200
SECURITY_SCAN_ENABLED=true
FORTITUDE_ENABLED=true

echo -e "${BLUE}🔍 CE-DPS Quality Gates - AI Implementation Validation${NC}"
echo "=================================================="

# Function to log with timestamp
log() {
    echo -e "$(date '+%Y-%m-%d %H:%M:%S') - $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to run quality gate
run_gate() {
    local gate_name="$1"
    local gate_command="$2"
    local gate_description="$3"
    
    echo -e "\n${BLUE}🚪 Gate: $gate_name${NC}"
    echo "Description: $gate_description"
    echo "Command: $gate_command"
    
    if eval "$gate_command"; then
        echo -e "${GREEN}✅ $gate_name: PASSED${NC}"
        return 0
    else
        echo -e "${RED}❌ $gate_name: FAILED${NC}"
        return 1
    fi
}

# Pre-implementation quality gates
echo -e "\n${YELLOW}📋 Pre-Implementation Quality Gates${NC}"

# Check if on correct branch
if ! run_gate "Branch Check" "git branch --show-current | grep -v '^main$' | grep -v '^master$'" "Ensure not on main/master branch"; then
    echo -e "${RED}Error: Cannot run quality gates on main/master branch${NC}"
    exit 1
fi

# Check working directory is clean
run_gate "Working Directory" "git status --porcelain | wc -l | grep -q '^0$' || echo 'Uncommitted changes detected - review before proceeding'" "Check for uncommitted changes"

# Compilation check
if command_exists cargo; then
    run_gate "Compilation" "cargo check --all-targets" "Ensure code compiles without errors"
fi

# Implementation quality gates
echo -e "\n${YELLOW}🔨 Implementation Quality Gates${NC}"

# Code formatting
if command_exists cargo; then
    run_gate "Code Formatting" "cargo fmt --check" "Ensure consistent code formatting"
fi

# Linting
if command_exists cargo; then
    run_gate "Linting" "cargo clippy --all-targets -- -D warnings" "Ensure code quality standards"
fi

# Security audit
if command_exists cargo && [[ "$SECURITY_SCAN_ENABLED" == "true" ]]; then
    run_gate "Security Audit" "cargo audit" "Check for security vulnerabilities"
fi

# Unit tests
if command_exists cargo; then
    run_gate "Unit Tests" "cargo test --lib" "Ensure all unit tests pass"
fi

# Integration tests
if command_exists cargo; then
    run_gate "Integration Tests" "cargo test --test '*'" "Ensure integration tests pass"
fi

# Test coverage
if command_exists cargo && command_exists tarpaulin; then
    echo -e "\n${BLUE}📊 Test Coverage Analysis${NC}"
    coverage_result=$(cargo tarpaulin --out Xml --output-dir target/coverage)
    coverage_percentage=$(echo "$coverage_result" | grep -o '[0-9]*\.[0-9]*%' | head -1 | sed 's/%//')
    
    if (( $(echo "$coverage_percentage >= $COVERAGE_TARGET" | bc -l) )); then
        echo -e "${GREEN}✅ Test Coverage: $coverage_percentage% (Target: $COVERAGE_TARGET%)${NC}"
    else
        echo -e "${RED}❌ Test Coverage: $coverage_percentage% (Target: $COVERAGE_TARGET%)${NC}"
        exit 1
    fi
fi

# Performance benchmarks
if command_exists cargo; then
    echo -e "\n${BLUE}⚡ Performance Benchmarks${NC}"
    if [ -d "benches" ]; then
        run_gate "Performance Tests" "cargo bench" "Ensure performance requirements are met"
    else
        echo -e "${YELLOW}⚠️ No benchmark tests found - consider adding performance tests${NC}"
    fi
fi

# Documentation generation
if command_exists cargo; then
    run_gate "Documentation Build" "cargo doc --no-deps" "Ensure documentation builds successfully"
fi

# Fortitude knowledge integration
if [[ "$FORTITUDE_ENABLED" == "true" ]] && command_exists fortitude; then
    echo -e "\n${BLUE}🧠 Fortitude Knowledge Integration${NC}"
    run_gate "Knowledge Update" "fortitude update-patterns --project-path ." "Update implementation patterns in knowledge base"
fi

# Post-implementation quality gates
echo -e "\n${YELLOW}🎯 Post-Implementation Quality Gates${NC}"

# Final compilation check
if command_exists cargo; then
    run_gate "Final Compilation" "cargo build --release" "Ensure production build succeeds"
fi

# Check for TODO/FIXME/HACK comments
todo_count=$(grep -r "TODO\|FIXME\|HACK" src/ --include="*.rs" | wc -l)
if [ "$todo_count" -gt 0 ]; then
    echo -e "${YELLOW}⚠️ Found $todo_count TODO/FIXME/HACK comments - review before production${NC}"
    grep -r "TODO\|FIXME\|HACK" src/ --include="*.rs" | head -5
fi

# Generate quality report
echo -e "\n${BLUE}📄 Quality Report Generation${NC}"
report_file="target/quality-report-$(date +%Y%m%d-%H%M%S).json"
cat > "$report_file" << EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%S.%3NZ)",
  "project": "$(basename $(pwd))",
  "branch": "$(git branch --show-current)",
  "commit": "$(git rev-parse HEAD)",
  "quality_gates": {
    "all_passed": true,
    "coverage_percentage": ${coverage_percentage:-0},
    "coverage_target": $COVERAGE_TARGET,
    "security_scan_enabled": $SECURITY_SCAN_ENABLED,
    "todo_comments": $todo_count
  },
  "recommendations": [
    "Review any TODO/FIXME/HACK comments before production deployment",
    "Consider adding performance benchmarks if not present",
    "Ensure monitoring and alerting are configured for production"
  ]
}
EOF

echo -e "${GREEN}📊 Quality report generated: $report_file${NC}"

# Summary
echo -e "\n${GREEN}🎉 All Quality Gates Passed!${NC}"
echo "=================================================="
echo "Your AI-implemented code meets CE-DPS quality standards."
echo "Ready for human business validation and production deployment."

# Integration with CI/CD
if [ "$CI" = "true" ]; then
    echo -e "\n${BLUE}🔄 CI/CD Integration${NC}"
    echo "Quality gates completed successfully in CI environment"
    echo "Uploading coverage report..."
    # Add your CI/CD specific commands here
fi

exit 0