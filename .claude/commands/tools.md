# <context>CE-DPS Quality Tools</context>

<meta>
  <title>CE-DPS Quality Tools</title>
  <type>slash-command</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-17</updated>
  <scope>quality-tools</scope>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: CE-DPS quality gates and validation tools with comprehensive reporting
- **Core Benefits**: Multi-phase quality validation, tool availability detection, actionable reporting
- **Quality Standards**: >95% test coverage, security validation, performance benchmarking
- **Output**: Comprehensive quality metrics with improvement recommendations

## <instructions priority="high">Quality Tools Execution</instructions>

### <step-1>Validate Environment</step-1>
**Project Validation**:
- Check CE-DPS project status (docs/ce-dps-state.json exists)
- Verify current phase and determine appropriate quality checks
- Ensure necessary tools available (compiler, test runner, linting tools)

### <step-2>Execute Code Quality Checks</step-2>
**Quality Validation**:
- Linting and code formatting validation
- Code complexity and maintainability metrics
- Coding standards compliance
- Documentation coverage for public APIs

### <step-3>Run Test Suite with Coverage</step-3>
**Test Execution**:
- Complete test suite (unit, integration, security)
- Test coverage report generation
- Validate >95% coverage requirement for business logic
- Critical paths test coverage verification

### <step-4>Perform Security Validation</step-4>
**Security Checks**:
- Security vulnerability scanning
- Common security issues (SQL injection, XSS, etc.)
- Input sanitization and validation
- Authentication and authorization implementation

### <step-5>Execute Performance Testing</step-5>
**Performance Validation**:
- Performance benchmarks for critical paths
- API response times validation (<200ms requirement)
- Memory usage and resource efficiency
- Database query performance testing

### <step-6>Validate Documentation</step-6>
**Documentation Checks**:
- API documentation completeness
- Code comments and documentation quality
- Deployment and troubleshooting guides
- README and project documentation currency

### <step-7>Generate Quality Report</step-7>
**Reporting**:
- Comprehensive quality metrics summary
- Document issues found and recommended fixes
- Generate actionable improvement recommendations
- Save results to docs/quality-reports/ with timestamp

### <step-8>Provide Recommendations</step-8>
**Improvement Guidance**:
- Identify areas needing improvement
- Suggest specific actions to address quality issues
- Recommend next steps based on current phase
- Provide guidance for maintaining quality standards

## <expected-output priority="medium">Quality Tools Results</expected-output>

**Command Execution Flow**:
- Display "🔧 CE-DPS Quality Gates and Tools" header
- Check project initialization and read current phase with jq
- Run quality gates tool with cargo build and execution
- Execute test suite with cargo test and optional coverage reporting
- Perform security validation with cargo audit and pattern checking
- Run performance benchmarks if benches/ directory exists
- Execute phase validation tool with Python
- Check Fortitude integration connectivity
- Run code quality checks with clippy and fmt
- Validate documentation completeness
- Generate summary report with tool availability status
- Provide installation tips for missing tools

## <parameters priority="low">Command Configuration</parameters>
**Configuration Details**:
- No parameters required
- Uses jq to read project state and current phase
- Checks for various tool availability before execution
- Provides conditional execution based on tool availability

## <implementation-notes priority="low">Technical Details</implementation-notes>
**Technical Implementation**:
- Comprehensive quality validation using bash commands
- Tool availability affects which checks can be executed
- Provides installation instructions for missing tools
- Supports all CE-DPS phases with appropriate quality standards
- Generates actionable reports with success/failure indicators