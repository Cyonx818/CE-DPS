# <context>CE-DPS Quality Check</context>

<meta>
  <title>CE-DPS Quality Check System</title>
  <type>slash-command</type>
  <audience>ai_assistant</audience>
  <complexity>advanced</complexity>
  <updated>2025-07-17</updated>
  <scope>quality-validation</scope>
  <requirements>comprehensive-testing</requirements>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: Complete CI/CD test suite with auto-fix capabilities and comprehensive validation
- **Core Benefits**: Automated quality enforcement, intelligent failure remediation, comprehensive reporting
- **Quality Gates**: >95% test coverage, zero critical vulnerabilities, <200ms response times
- **Auto-Fix**: Formatting, linting, security patterns, documentation

## <instructions priority="high">Quality Validation Process</instructions>

### <step-1>Pre-Flight Validation</step-1>
**Environment Checks**:
- Project state and current phase validation
- Tool and dependency availability verification
- Git working directory cleanliness (offer stash if dirty)
- Environment configuration validation

### <step-2>Execute Comprehensive Test Suite</step-2>
**Test Categories** (sequential execution):
- **Unit tests**: Coverage reporting
- **Integration tests**: Database validation
- **Security tests**: Vulnerability scanning
- **Performance tests**: Benchmarking
- **End-to-end tests**: If applicable

### <step-3>Code Quality Validation</step-3>
**Quality Checks** (with auto-fix):
- Linting with automatic fixes
- Code formatting corrections
- Code complexity and maintainability metrics
- Coding standards compliance validation

### <step-4>Security and Compliance Checks</step-4>
**Security Validation**:
- Comprehensive vulnerability scanning
- Secrets and sensitive data detection
- Input sanitization and validation patterns
- Authentication and authorization implementation
- Security standards compliance verification

### <step-5>Performance and Scalability Validation</step-5>
**Performance Testing**:
- Performance benchmarks execution
- Response time requirements validation (<200ms)
- Memory usage and resource efficiency
- Load testing scenarios
- Database query performance validation

### <step-6>Documentation and API Validation</step-6>
**Documentation Checks**:
- API documentation completeness (>90% coverage)
- API contract compliance
- Code documentation coverage
- Deployment and operations documentation

### <step-7>Auto-Fix Capabilities</step-7>
**Automated Remediation**:
- Code formatting issues
- Safe linting auto-fixes
- Documentation template updates
- Minor security issues (automatically resolvable)

### <step-8>Quality Gate Enforcement</step-8>
**Gate Criteria**:
- **Fail fast**: Critical issues (security vulnerabilities, test failures)
- **Detailed reporting**: Failure reports with remediation steps
- **CE-DPS standards**: All quality gates must pass
- **Status generation**: Pass/fail with comprehensive reasoning

### <step-9>Reporting and Recommendations</step-9>
**Output Generation**:
- Detailed quality report with metrics
- Specific remediation steps for failures
- Actionable improvement recommendations
- Results saved for trend analysis and continuous improvement

### <step-10>SKYNET Mode Auto-Loop</step-10>
**Autonomous Continuous Development** (if SKYNET=true):
- After successful quality validation, automatically trigger next sprint loop
- Increment sprint number and prepare new sprint environment
- Execute automatic command progression: /phase2:setup (next sprint)
- Continue autonomous development cycle indefinitely
- Only stop loop if quality gates fail or technical issues detected
- Maintain sprint tracking and project state across iterations

## <expected-output priority="medium">Quality Validation Results</expected-output>

**Cargo Command Sequence**:
- **cargo fmt --all -- --check** (auto-fix with cargo fmt --all if failed)
- **cargo clippy** with warnings as errors (auto-fix with --fix flags if failed)
- **cargo build --workspace --verbose** (resolve compilation errors if failed)
- **cargo test --workspace --verbose** (fix tests or code optimally if failed)
- **cargo audit** (implement secure patterns if vulnerabilities found)
- **cargo doc --workspace --no-deps** (fix documentation build if failed)
- **Extended quality gates** (quality-gates tool, Python tests, Fortitude integration)
- **Back-to-back validation** (complete pipeline twice)

## <parameters priority="low">Command Configuration</parameters>
**Configuration Requirements**:
- No parameters required
- Uses specific cargo commands with exact flags
- Implements auto-fix protocol for each failure type
- Requires 100% pass rate in back-to-back runs

## <implementation-notes priority="critical">Quality Standards</implementation-notes>
**Critical Requirements**:
- **AI must fix ALL failures** using optimal architectural solutions
- **Choose architectural improvements** over quick patches
- **Sequential execution** with auto-fix between failures
- **Back-to-back validation** ensures stability and repeatability
- **Non-negotiable**: ALL quality gates must pass cleanly