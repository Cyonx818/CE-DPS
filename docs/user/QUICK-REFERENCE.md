# CE-DPS Quick Reference Guide

A comprehensive command reference for CE-DPS slash commands with examples and usage patterns.

## Command Overview

### Project Management
```bash
/init            # Initialize new CE-DPS project
/project-status  # Show current project status and next steps
/tools           # Run quality gates and validation tools
/quality-check   # Complete CI/CD test suite with auto-fix
/help            # Show comprehensive help and command reference
```

### Phase 1: Strategic Planning
```bash
/phase1:setup    # Initialize Phase 1 environment and business template
/phase1:analyze  # Trigger AI architectural analysis of requirements
/phase1:validate # Validate Phase 1 completion and human approvals
```

### Phase 2: Sprint Planning
```bash
/phase2:setup    # Initialize Phase 2 environment and feature selection
/phase2:plan     # Trigger AI implementation planning for selected features
/phase2:validate # Validate Phase 2 completion and implementation approach
```

### Phase 3: Implementation
```bash
/phase3:setup     # Initialize Phase 3 environment with quality gates
/phase3:implement # Trigger AI implementation with test-driven development
/phase3:validate  # Validate Phase 3 completion and production readiness
```

### SKYNET Autonomous Mode
```bash
/skynet:enable       # Enable autonomous operation without human approval checkpoints
/skynet:disable      # Return to human oversight mode
/skynet:status       # Check current SKYNET mode and operational status
/skynet:resume       # Resume interrupted autonomous loops
/skynet:quick-enable # Rapid SKYNET activation for experienced users
```

## Command Usage Examples

### Starting a New Project
```bash
# Initialize project structure and environment
/init

# Check current status and next steps
/project-status
# Output: "👉 Start Phase 1: Strategic Planning"
# Command: /phase1:setup

# Set up Phase 1 strategic planning
/phase1:setup
# Creates: docs/phases/phase-1-planning.md
# Next: Fill out business requirements template
```

### Phase 1 Workflow
```bash
# After filling business requirements template
/phase1:analyze
# AI performs comprehensive architectural analysis
# Provides: System architecture, technology evaluation, implementation strategy

# After reviewing and approving AI analysis
/phase1:validate
# Validates: Human approvals, architectural decisions, readiness for Phase 2
# Output: "🎉 Phase 1 complete! Ready for Phase 2"
```

### Phase 2 Workflow
```bash
# Start sprint planning
/phase2:setup
# Creates: Sprint planning template with feature roadmap from Phase 1
# Next: Select 2-4 features for sprint implementation

# After selecting features for sprint
/phase2:plan
# AI creates: Detailed implementation plans, complexity analysis, effort estimates
# Provides: File-level task breakdown, technical approach, risk assessment

# After reviewing and approving implementation plan
/phase2:validate
# Validates: Feature selection, implementation approach, timeline
# Output: "🎉 Phase 2 complete! Ready for Phase 3"
```

### Phase 3 Workflow
```bash
# Start implementation
/phase3:setup
# Creates: Implementation environment, feature branch, quality gates
# Prepares: Testing framework, security validation, performance benchmarks

# Begin AI implementation
/phase3:implement
# AI performs: Test-driven development, comprehensive testing, security validation
# Provides: Working features with >95% test coverage, security patterns

# After validating business value of implemented features
/phase3:validate
# Validates: Feature functionality, business value, production readiness
# Output: "🎉 Implementation complete! Ready for production"
```

### Human Action Points

Each command clearly indicates when human action is required:

**Phase 1**: Fill business requirements → Review AI analysis → Approve architecture
**Phase 2**: Select sprint features → Review implementation plan → Approve approach  
**Phase 3**: Validate implemented features → Confirm business value → Approve production

### Quality Integration

Commands integrate seamlessly with quality gates:
- `/tools` runs comprehensive quality validation
- Each phase automatically runs appropriate quality checks
- Human approval required only after quality standards are met

## Quality Tools Integration

### CE-DPS Slash Commands Integration

The recommended way to run tools is through the integrated slash commands:

```bash
# Primary quality validation tool
/tools
# Runs: Quality gates, test suite, security audit, performance benchmarks
# Provides: Comprehensive quality validation with actionable recommendations

# Comprehensive CI/CD quality validation with auto-fix
/quality-check
# Runs: Complete CI/CD test suite matching .github/workflows/ci.yml
# Features: Auto-fix for formatting, linting, and dependency issues
# Validates: Rust tests, Python tests, security audit, documentation build
# Integrates: Quality gates, integration tests, and coverage reporting

# Project status and guidance
/project-status
# Shows: Current phase, completion status, next steps, SKYNET mode status
# Guides: What to do next to proceed with the methodology

# Help and command reference
/help
# Displays: All available commands, workflow guidance, troubleshooting
```

### Quality Gates Tool
```bash
# Run comprehensive quality validation
cargo run --bin quality-gates

# With custom options
cargo run --bin quality-gates -- --coverage-target 98 --performance-target 150

# Generate quality report
cargo run --bin quality-gates -- --output target/quality-report.json

# Features:
# - Pre/implementation/post quality gates
# - Code formatting and linting validation
# - Security vulnerability scanning
# - Test coverage analysis
# - TODO comment tracking
# - Comprehensive reporting
```

### Phase Validator
```bash
# Validate phase completion
./tools/phase-validator.py --phase 1  # Strategic planning
./tools/phase-validator.py --phase 2  # Sprint planning
./tools/phase-validator.py --phase 3  # Implementation

# Generates comprehensive validation reports
```

### Fortitude Integration Tool
```bash
# Check Fortitude installation
cargo run --bin fortitude-integration -- check

# Initialize Fortitude for CE-DPS
cargo run --bin fortitude-integration -- init

# Start Fortitude services
cargo run --bin fortitude-integration -- start

# Query existing patterns
cargo run --bin fortitude-integration -- query "authentication patterns"

# Update knowledge base
cargo run --bin fortitude-integration -- update

# Generate reports
cargo run --bin fortitude-integration -- report

# Setup Claude Code integration
cargo run --bin fortitude-integration -- setup-claude

# Complete installation
cargo run --bin fortitude-integration -- install
```

## Getting Started Quickly

```bash
# Initialize project and get template
/init
/phase1:setup

# Fill out docs/phases/phase-1-planning.md with your requirements
# Then trigger AI analysis
/phase1:analyze
```

## Monitoring Progress

```bash
# Check current status and next steps
/project-status

# Run quality validation
/tools

# Get help if needed
/help
```

## Need More Detail?

- **[Methodology Guide](METHODOLOGY.md)** - Understand the 3-phase development process
- **[SKYNET Mode](SKYNET-MODE.md)** - Learn about autonomous operation
- **[Collaboration Guide](COLLABORATION.md)** - Master human-AI collaboration patterns