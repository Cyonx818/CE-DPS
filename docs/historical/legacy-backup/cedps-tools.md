---
description: "Run CE-DPS quality gates and validation tools"
allowed-tools: ["bash", "read"]
---

# <context>CE-DPS Quality Gates and Tools</context>

<meta>
  <title>CE-DPS Quality Gates and Tools</title>
  <type>quality-validation</type>
  <audience>ai_assistant</audience>
  <complexity>advanced</complexity>
  <updated>2025-07-16</updated>
  <mdeval-score>0.88</mdeval-score>
  <token-efficiency>0.17</token-efficiency>
  <last-validated>2025-07-16</last-validated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Execute comprehensive CE-DPS quality gates including testing, security, performance, and validation tools
- **Coverage**: Quality gates tool, test suite, security audit, performance benchmarks, phase validation, code quality
- **Requirements**: Project initialized, various tools available (cargo, python3), proper project structure
- **Output**: Comprehensive quality assessment with actionable recommendations and tool installation guidance
- **Integration**: Supports all CE-DPS phases with phase-appropriate quality standards and continuous improvement

<!-- CHUNK-BOUNDARY: orchestration -->

## <implementation>Quality Tools Orchestration</implementation>

"""
CE-DPS Quality Gates and Tools
🔧 Comprehensive quality validation across all development areas
"""

### <method>Quality Validation Sequence</method>
«tools-initiation»
!echo "🔧 CE-DPS Quality Gates and Tools"
!echo "=================================="
«/tools-initiation»

<!-- CHUNK-BOUNDARY: project-validation -->

### <constraints priority="critical">Project Initialization Check</constraints>

# Check if project is initialized
!if [ ! -f "docs/ce-dps-state.json" ]; then
    echo "❌ CE-DPS project not initialized"
    echo "💡 Run '/cedps-init' to initialize the project"
    exit 1
fi

!CURRENT_PHASE=$(jq -r '.current_phase' docs/ce-dps-state.json)
!echo "Current Phase: $CURRENT_PHASE"
!echo ""

<!-- CHUNK-BOUNDARY: quality-gates -->

### <method>Quality Gates Tool Execution</method>
«quality-gates-tool»
# Quality Gates Tool
!echo "🚀 Running Quality Gates Tool"
!echo "============================="
!CARGO_AVAILABLE=$(command -v cargo >/dev/null 2>&1 && echo "true" || echo "false")
!if [ "$CARGO_AVAILABLE" = "true" ] && [ -f "tools/quality-gates/Cargo.toml" ]; then
    echo "🔧 Building quality gates tool..."
    cd tools/quality-gates
    cargo build --release
    BUILD_SUCCESS=$?
    if [ $BUILD_SUCCESS -eq 0 ]; then
        echo "✅ Quality gates tool built successfully"
        echo "🔍 Running comprehensive quality validation..."
        ./target/release/quality-gates --comprehensive-validation
        QG_EXIT_CODE=$?
        if [ $QG_EXIT_CODE -eq 0 ]; then
            echo "✅ Quality gates validation passed"
        else
            echo "❌ Quality gates validation failed (exit code: $QG_EXIT_CODE)"
            echo "💡 Address quality issues before proceeding"
        fi
    else
        echo "❌ Failed to build quality gates tool"
        echo "💡 Check Rust toolchain and dependencies"
    fi
    cd ../..
else
    echo "⚠️  Quality gates tool not available"
    echo "💡 Ensure Rust toolchain is installed and tools/quality-gates/ exists"
fi
«/quality-gates-tool»

!echo ""

<!-- CHUNK-BOUNDARY: test-suite -->

### <method>Test Suite Execution</method>
«test-suite-execution»
# Test Suite Execution
!echo "🧪 Running Test Suite"
!echo "===================="
!CARGO_AVAILABLE=$(command -v cargo >/dev/null 2>&1 && echo "true" || echo "false")
!if [ "$CARGO_AVAILABLE" = "true" ] && [ -f "Cargo.toml" ]; then
    echo "🔍 Running comprehensive test suite..."
    cargo test --verbose
    TEST_SUCCESS=$?
    if [ $TEST_SUCCESS -eq 0 ]; then
        echo "✅ All tests passed"
        
        # Test coverage if available
        TARPAULIN_AVAILABLE=$(command -v cargo-tarpaulin >/dev/null 2>&1 && echo "true" || echo "false")
        if [ "$TARPAULIN_AVAILABLE" = "true" ]; then
            echo "📊 Generating test coverage report..."
            cargo tarpaulin --out Html --output-dir target/tarpaulin-report
            COVERAGE=$(cargo tarpaulin --quiet 2>/dev/null | grep -o '[0-9]*\.[0-9]*%' | head -1)
            if [ -n "$COVERAGE" ]; then
                echo "📈 Test coverage: $COVERAGE"
                echo "📊 Coverage report: target/tarpaulin-report/tarpaulin-report.html"
            fi
        else
            echo "⚠️  Install cargo-tarpaulin for coverage reporting"
        fi
    else
        echo "❌ Some tests failed"
        echo "💡 Fix failing tests before proceeding"
    fi
else
    echo "⚠️  Rust project not found or cargo not available"
    echo "💡 Ensure this is a Rust project with Cargo.toml"
fi
«/test-suite-execution»

!echo ""

<!-- CHUNK-BOUNDARY: security-validation -->

### <method>Security Validation</method>
«security-validation»
# Security Validation
!echo "🔒 Security Validation"
!echo "====================="
!CARGO_AVAILABLE=$(command -v cargo >/dev/null 2>&1 && echo "true" || echo "false")
!if [ "$CARGO_AVAILABLE" = "true" ] && [ -f "Cargo.toml" ]; then
    echo "🔍 Running security audit..."
    AUDIT_AVAILABLE=$(command -v cargo-audit >/dev/null 2>&1 && echo "true" || echo "false")
    if [ "$AUDIT_AVAILABLE" = "true" ]; then
        cargo audit
        AUDIT_SUCCESS=$?
        if [ $AUDIT_SUCCESS -eq 0 ]; then
            echo "✅ Security audit passed"
        else
            echo "❌ Security audit found issues"
            echo "💡 Update vulnerable dependencies"
        fi
    else
        echo "⚠️  Install cargo-audit for security scanning"
        echo "💡 Run: cargo install cargo-audit"
    fi
    
    # Check for common security patterns
    echo "🔍 Checking security patterns..."
    if grep -r "password" . --include="*.rs" | grep -v "test" | grep -v "target/" | grep -v "docs/"; then
        echo "⚠️  Found password-related code - verify security handling"
    fi
    
    if grep -r "secret" . --include="*.rs" | grep -v "test" | grep -v "target/" | grep -v "docs/"; then
        echo "⚠️  Found secret-related code - verify security handling"
    fi
    
    echo "✅ Security pattern check completed"
else
    echo "⚠️  Rust project not found for security validation"
fi
«/security-validation»

!echo ""

<!-- CHUNK-BOUNDARY: performance-benchmarks -->

### <method>Performance Benchmarks</method>
«performance-benchmarks»
# Performance Benchmarks
!echo "⚡ Performance Benchmarks"
!echo "========================"
!CARGO_AVAILABLE=$(command -v cargo >/dev/null 2>&1 && echo "true" || echo "false")
!if [ "$CARGO_AVAILABLE" = "true" ] && [ -f "Cargo.toml" ]; then
    # Check for benchmark directory
    if [ -d "benches" ]; then
        echo "🔍 Running performance benchmarks..."
        cargo bench
        BENCH_SUCCESS=$?
        if [ $BENCH_SUCCESS -eq 0 ]; then
            echo "✅ Performance benchmarks completed"
            echo "📊 Benchmark results in target/criterion/"
        else
            echo "❌ Performance benchmarks failed"
            echo "💡 Check benchmark implementation"
        fi
    else
        echo "⚠️  No benchmarks directory found"
        echo "💡 Create benches/ directory with benchmark files"
    fi
else
    echo "⚠️  Rust project not found for performance benchmarks"
fi
«/performance-benchmarks»

!echo ""

<!-- CHUNK-BOUNDARY: phase-validation -->

### <method>Phase Validation Tool</method>
«phase-validation»
# Phase Validation Tool
!echo "📋 Phase Validation"
!echo "=================="
!PYTHON_AVAILABLE=$(command -v python3 >/dev/null 2>&1 && echo "true" || echo "false")
!if [ "$PYTHON_AVAILABLE" = "true" ] && [ -f "tools/phase-validator.py" ]; then
    echo "🔍 Running phase validation tool..."
    python3 tools/phase-validator.py --phase "$CURRENT_PHASE"
    VALIDATION_SUCCESS=$?
    if [ $VALIDATION_SUCCESS -eq 0 ]; then
        echo "✅ Phase validation passed"
    else
        echo "❌ Phase validation failed"
        echo "💡 Address phase validation issues"
    fi
else
    echo "⚠️  Phase validator not available"
    echo "💡 Ensure Python 3 is installed and tools/phase-validator.py exists"
fi
«/phase-validation»

!echo ""

<!-- CHUNK-BOUNDARY: fortitude-integration -->

### <method>Fortitude Integration</method>
«fortitude-integration»
# Fortitude Integration
!echo "🧠 Fortitude Knowledge Management"
!echo "================================"
!CARGO_AVAILABLE=$(command -v cargo >/dev/null 2>&1 && echo "true" || echo "false")
!if [ "$CARGO_AVAILABLE" = "true" ] && [ -f "tools/fortitude-integration/Cargo.toml" ]; then
    echo "🔍 Running Fortitude integration check..."
    cd tools/fortitude-integration
    cargo run -- status
    FORTITUDE_SUCCESS=$?
    if [ $FORTITUDE_SUCCESS -eq 0 ]; then
        echo "✅ Fortitude integration functional"
    else
        echo "❌ Fortitude integration issues detected"
        echo "💡 Check Fortitude configuration and connectivity"
    fi
    cd ../..
else
    echo "⚠️  Fortitude integration not available"
    echo "💡 Ensure tools/fortitude-integration/ exists"
fi
«/fortitude-integration»

!echo ""

<!-- CHUNK-BOUNDARY: code-quality -->

### <method>Code Quality Checks</method>
«code-quality-checks»
# Code Quality Checks
!echo "✨ Code Quality Checks"
!echo "====================="
!CARGO_AVAILABLE=$(command -v cargo >/dev/null 2>&1 && echo "true" || echo "false")
!if [ "$CARGO_AVAILABLE" = "true" ] && [ -f "Cargo.toml" ]; then
    echo "🔍 Running code quality checks..."
    
    # Clippy linting
    CLIPPY_AVAILABLE=$(command -v cargo-clippy >/dev/null 2>&1 && echo "true" || echo "false")
    if [ "$CLIPPY_AVAILABLE" = "true" ]; then
        echo "🔍 Running Clippy linter..."
        cargo clippy -- -D warnings
        CLIPPY_SUCCESS=$?
        if [ $CLIPPY_SUCCESS -eq 0 ]; then
            echo "✅ Clippy linting passed"
        else
            echo "❌ Clippy found issues"
            echo "💡 Fix linting warnings before proceeding"
        fi
    else
        echo "⚠️  Install cargo-clippy for linting"
    fi
    
    # Formatting check
    FMT_AVAILABLE=$(command -v cargo-fmt >/dev/null 2>&1 && echo "true" || echo "false")
    if [ "$FMT_AVAILABLE" = "true" ]; then
        echo "🔍 Checking code formatting..."
        cargo fmt --check
        FMT_SUCCESS=$?
        if [ $FMT_SUCCESS -eq 0 ]; then
            echo "✅ Code formatting is correct"
        else
            echo "❌ Code formatting issues found"
            echo "💡 Run 'cargo fmt' to fix formatting"
        fi
    else
        echo "⚠️  Install cargo-fmt for formatting checks"
    fi
else
    echo "⚠️  Rust project not found for quality checks"
fi
«/code-quality-checks»

!echo ""

<!-- CHUNK-BOUNDARY: documentation-validation -->

### <method>Documentation Validation</method>
«documentation-validation»
# Documentation Validation
!echo "📚 Documentation Validation"
!echo "=========================="
!echo "🔍 Checking documentation completeness..."

# Check for required documentation
!DOC_CHECKS=("docs/PROJECT.md" "docs/ce-dps-state.json" "CLAUDE.md" "README.md")
!for doc in "${DOC_CHECKS[@]}"; do
    if [ -f "$doc" ]; then
        echo "✅ $doc - exists"
    else
        echo "❌ $doc - missing"
    fi
done

# Check phase documentation
!if [ -f "docs/phases/phase-1-planning.md" ]; then
    echo "✅ Phase 1 documentation - exists"
fi
!if [ -f "docs/phases/phase-2-sprint-planning.md" ]; then
    echo "✅ Phase 2 documentation - exists"
fi
!if [ -f "docs/phases/phase-3-implementation.md" ]; then
    echo "✅ Phase 3 documentation - exists"
fi

# Check for API documentation
!if [ -d "target/doc" ]; then
    echo "✅ API documentation generated"
else
    echo "⚠️  Generate API docs with 'cargo doc'"
fi
«/documentation-validation»

!echo ""

<!-- CHUNK-BOUNDARY: summary-report -->

### <pattern>Quality Gates Summary Report</pattern>
«summary-report»
# Summary Report
!echo "📊 Quality Gates Summary"
!echo "======================="
!echo "🔧 Quality Gates: $([ -f "tools/quality-gates/Cargo.toml" ] && echo "Available" || echo "Not available")"
!echo "🧪 Test Suite: $([ -f "Cargo.toml" ] && echo "Available" || echo "Not available")"
!echo "🔒 Security Tools: $(command -v cargo-audit >/dev/null 2>&1 && echo "Available" || echo "Install cargo-audit")"
!echo "⚡ Performance: $([ -d "benches" ] && echo "Available" || echo "No benchmarks")"
!echo "📋 Phase Validation: $([ -f "tools/phase-validator.py" ] && echo "Available" || echo "Not available")"
!echo "🧠 Fortitude: $([ -f "tools/fortitude-integration/Cargo.toml" ] && echo "Available" || echo "Not available")"
!echo "✨ Code Quality: $(command -v cargo-clippy >/dev/null 2>&1 && echo "Available" || echo "Install clippy")"

!echo ""
!echo "💡 Tool Installation Tips"
!echo "========================"
!echo "Security: cargo install cargo-audit"
!echo "Coverage: cargo install cargo-tarpaulin"
!echo "Linting: rustup component add clippy"
!echo "Formatting: rustup component add rustfmt"

!echo ""
!echo "✅ Quality gates execution completed"
!echo "📊 Review results above for any issues to address"
«/summary-report»

<!-- CHUNK-BOUNDARY: tools-constraints -->

### <constraints priority="critical">Tool Requirements</constraints>
«tools-constraints»
- Project must be initialized
- Requires various tools (cargo, python3, etc.) for full functionality
- Tool availability affects which checks can be run
- Some tools require separate installation
«/tools-constraints»

<!-- CHUNK-BOUNDARY: human-interpretation -->

## <human-action-required>
**Quality Gates Execution Complete! 🔧**

### <quality-assessment>
«quality-assessment»
The quality gates tool has run comprehensive validation including:
- **Quality Gates Tool**: Project-specific quality validation
- **Test Suite**: Comprehensive test execution with coverage
- **Security Validation**: Dependency audit and pattern checking
- **Performance Benchmarks**: Performance testing and profiling
- **Phase Validation**: Current phase completion validation
- **Fortitude Integration**: Knowledge management system check
- **Code Quality**: Linting, formatting, and standards checking
- **Documentation**: Documentation completeness validation
«/quality-assessment»

<!-- CHUNK-BOUNDARY: result-interpretation -->

### <interpreting-results>
«result-indicators»
**Result Indicators**:
- **✅ Success**: Tool ran successfully and passed validation
- **❌ Failure**: Tool found issues that need to be addressed
- **⚠️ Warning**: Tool not available or optional issues found
- **💡 Recommendation**: Suggested actions for improvement
«/result-indicators»

<!-- CHUNK-BOUNDARY: issue-resolution -->

### <addressing-issues>
«issue-solutions»
**Common Issues and Solutions**:
- **Test failures**: Fix failing tests before proceeding
- **Security vulnerabilities**: Update vulnerable dependencies
- **Coverage too low**: Add more comprehensive tests
- **Linting warnings**: Fix code style issues
- **Missing tools**: Install recommended tools for full validation
«/issue-solutions»

<!-- CHUNK-BOUNDARY: tool-installation -->

### <tool-installation>
«tool-installation-guide»
**Install Missing Tools**:
```bash
# Security auditing
cargo install cargo-audit

# Test coverage
cargo install cargo-tarpaulin

# Code quality
rustup component add clippy
rustup component add rustfmt

# Documentation
cargo doc --open
```
«/tool-installation-guide»

<!-- CHUNK-BOUNDARY: continuous-quality -->

### <continuous-quality>
«continuous-quality-practices»
**Quality Best Practices**:
- **Run quality gates regularly** during development
- **Address issues immediately** rather than accumulating technical debt
- **Maintain >95% test coverage** for all business logic
- **Keep dependencies updated** to avoid security vulnerabilities
- **Follow consistent code formatting** for maintainability
«/continuous-quality-practices»

<!-- CHUNK-BOUNDARY: phase-specific -->

### <phase-specific-quality>
«phase-specific-standards»
**Quality Standards by Phase**:
- **Phase 1**: Documentation completeness, architectural validation
- **Phase 2**: Implementation plan quality, dependency validation
- **Phase 3**: Code quality, test coverage, security validation, performance benchmarks
«/phase-specific-standards»

<!-- CHUNK-BOUNDARY: next-steps -->

### <next-steps>
«next-action-steps»
**After Quality Gates**:
1. **Address any failures** (❌) before proceeding
2. **Consider warnings** (⚠️) for improvement opportunities
3. **Install missing tools** for more comprehensive validation
4. **Integrate quality gates** into your development workflow
5. **Run regularly** to maintain quality standards
«/next-action-steps»
</human-action-required>

<!-- CHUNK-BOUNDARY: troubleshooting -->

## <troubleshooting>
### <common-issues>
«error-resolution»
- **"Project not initialized"**: Run `/cedps-init` first
- **"Cargo not found"**: Install Rust toolchain
- **"Tool not available"**: Install missing tools as suggested
- **"Quality gates failed"**: Address specific issues reported
- **"Tests failing"**: Fix failing tests before proceeding
- **"Security issues"**: Update vulnerable dependencies
«/error-resolution»

<!-- CHUNK-BOUNDARY: final-quality -->

### <quality-validation>
«final-quality-standards»
**Quality Tools Requirements**:
- [ ] Comprehensive quality validation across all areas
- [ ] Clear success/failure indicators
- [ ] Actionable recommendations for issues
- [ ] Installation guidance for missing tools
- [ ] Phase-appropriate quality standards
- [ ] Integration with CE-DPS methodology
- [ ] Support for continuous quality improvement
«/final-quality-standards»