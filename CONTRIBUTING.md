# Contributing to CE-DPS

Thank you for your interest in contributing to CE-DPS (Context Engineered Development Process Suite)! This document provides guidelines for contributing to both the methodology and the integrated Fortitude knowledge management platform.

## Overview

CE-DPS is an integrated system consisting of:
- **Methodology**: AI-implementation process with human strategic oversight
- **Fortitude**: Knowledge management platform for AI development workflows
- **Tools**: Automation and validation tools for the development process

## Development Setup

### Prerequisites

- Rust 1.70+ with Cargo
- Python 3.8+ (for phase-validator tool)
- Git
- Claude Code (recommended for AI-assisted development)

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/your-org/CE-DPS.git
cd CE-DPS

# Build the complete workspace
cargo build --workspace

# Install Fortitude integration
cargo run --bin fortitude-integration -- install

# Validate installation
cargo run --bin quality-gates
```

## Project Structure

```
CE-DPS/
├── methodology/           # CE-DPS methodology documentation
├── fortitude/            # Fortitude knowledge management platform
├── tools/                # CE-DPS development tools
│   ├── quality-gates/    # Rust: Quality validation tool
│   ├── fortitude-integration/ # Rust: Fortitude management tool
│   └── phase-validator.py # Python: Phase completion validation
├── examples/             # Real-world implementation examples
└── reference/            # Quick reference materials
```

## Development Workflow

### For Methodology Changes

1. **Follow CE-DPS Process**: Use the three-phase methodology for changes
   - Phase 1: Plan the methodology change
   - Phase 2: Design the implementation approach  
   - Phase 3: Implement and validate

2. **Update Documentation**: Ensure all methodology documents remain consistent
3. **Validate Examples**: Update examples to reflect methodology changes
4. **Test with Tools**: Ensure quality gates and phase validator work with changes

### For Fortitude Changes

1. **Follow Fortitude Development Process**: See `fortitude/DEVELOPMENT_PROCESS.md`
2. **Maintain API Compatibility**: Ensure CE-DPS tools continue to work
3. **Update Integration**: Modify CE-DPS integration if Fortitude APIs change
4. **Test Knowledge Management**: Validate that AI workflow improvements work

### For Tool Changes

1. **Test Integration**: Ensure tools work with both methodology and Fortitude
2. **Cross-Platform Support**: Test on multiple operating systems
3. **Documentation**: Update tool documentation and help text
4. **Examples**: Provide usage examples in tool README files

## Coding Standards

### Rust Code (Tools and Fortitude)

- Follow standard Rust conventions and `rustfmt` formatting
- Use `clippy` for linting and fix all warnings
- Implement comprehensive error handling with `thiserror`
- Add unit tests for all business logic
- Use `tracing` for structured logging

### Python Code (Phase Validator)

- Follow PEP 8 style guidelines
- Use type hints where applicable
- Add docstrings for functions and classes
- Include unit tests with `pytest`

### Documentation

- Use LLM-optimized patterns with semantic markup
- Follow progressive disclosure: Summary → Evidence → Implementation
- Include practical examples and code snippets
- Maintain consistent formatting across all documentation

## Testing Requirements

### Before Submitting Changes

```bash
# Run quality gates
cargo run --bin quality-gates

# Test phase validation
python tools/phase-validator.py --phase 1
python tools/phase-validator.py --phase 2  
python tools/phase-validator.py --phase 3

# Test Fortitude integration
cargo run --bin fortitude-integration -- status
cargo run --bin fortitude-integration -- query "test query"

# Run full test suite
cargo test --workspace
```

### Test Coverage

- Maintain >95% test coverage for Rust code
- Include integration tests for cross-component functionality
- Test error conditions and edge cases
- Validate tool output with sample projects

## Pull Request Process

### 1. Branch Naming

- `methodology/feature-name` - Methodology changes
- `fortitude/feature-name` - Fortitude platform changes
- `tools/feature-name` - Development tool changes
- `docs/feature-name` - Documentation updates
- `examples/feature-name` - Example implementations

### 2. Commit Messages

Follow conventional commit format:
```
type(scope): description

Examples:
feat(methodology): add Phase 4 for deployment automation
fix(fortitude): resolve knowledge gap detection bug
docs(examples): update authentication example for new patterns
test(tools): add integration tests for quality gates
```

### 3. Pull Request Content

- **Clear Description**: Explain the problem and solution
- **Testing**: Show that all quality gates pass
- **Documentation**: Update relevant documentation
- **Examples**: Update examples if methodology changes
- **Breaking Changes**: Clearly identify any breaking changes

### 4. Review Process

1. **Automated Checks**: All CI/CD checks must pass
2. **Code Review**: At least one maintainer review required
3. **Testing**: Demonstrate that changes work with real projects
4. **Documentation Review**: Ensure documentation is clear and complete

## Issue Guidelines

### Bug Reports

Include:
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, Rust version, etc.)
- Output from quality gates or error messages

### Feature Requests

Include:
- Problem statement and use case
- Proposed solution approach
- Impact on existing functionality
- Examples of how feature would be used

### Methodology Improvements

Include:
- Current methodology limitation
- Proposed improvement
- Evidence or reasoning for change
- Impact on AI-human collaboration

## Community Guidelines

- **Be Respectful**: Treat all contributors with respect and professionalism
- **Be Constructive**: Provide helpful feedback and suggestions
- **Be Patient**: Remember that this is an open source project with volunteer contributors
- **Ask Questions**: Don't hesitate to ask for clarification or help

## Getting Help

- **Documentation**: Start with README.md and methodology documents
- **Issues**: Search existing issues before creating new ones
- **Discussions**: Use GitHub Discussions for questions and ideas
- **AI Assistance**: The project is designed to work well with AI assistants like Claude Code

## Recognition

Contributors are recognized in:
- CHANGELOG.md for significant contributions
- README.md acknowledgments section
- Git commit history and authorship

Thank you for contributing to CE-DPS! Your contributions help improve AI-human collaboration in software development.