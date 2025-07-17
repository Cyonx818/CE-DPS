# CE-DPS Quality Check

Run complete CI/CD test suite with auto-fix capabilities and comprehensive validation.

## Instructions

1. **Pre-Flight Validation**
   - Check project state and current phase
   - Verify all necessary tools and dependencies are available
   - Ensure git working directory is clean (or offer to stash changes)
   - Validate environment configuration

2. **Execute Comprehensive Test Suite**
   - Run all test categories in sequence:
     - Unit tests with coverage reporting
     - Integration tests with database validation
     - Security tests with vulnerability scanning
     - Performance tests with benchmarking
     - End-to-end tests if applicable

3. **Code Quality Validation**
   - Run linting with auto-fix where possible
   - Execute code formatting (auto-fix formatting issues)
   - Check code complexity and maintainability metrics
   - Validate coding standards compliance

4. **Security and Compliance Checks**
   - Run comprehensive security vulnerability scan
   - Check for secrets or sensitive data in code
   - Validate input sanitization and validation patterns
   - Check authentication and authorization implementation
   - Verify compliance with security standards

5. **Performance and Scalability Validation**
   - Execute performance benchmarks
   - Validate response time requirements
   - Check memory usage and resource efficiency
   - Run load testing scenarios
   - Validate database query performance

6. **Documentation and API Validation**
   - Check API documentation completeness
   - Validate API contract compliance
   - Verify code documentation coverage
   - Check deployment and operations documentation

7. **Auto-Fix Capabilities**
   - Automatically fix code formatting issues
   - Apply linting auto-fixes where safe
   - Update documentation templates where needed
   - Fix minor security issues that can be automatically resolved

8. **Quality Gate Enforcement**
   - Fail fast on critical issues (security vulnerabilities, test failures)
   - Provide detailed failure reports with remediation steps
   - Only pass if all quality gates meet CE-DPS standards
   - Generate pass/fail status with comprehensive reasoning

9. **Reporting and Recommendations**
   - Generate detailed quality report with metrics
   - Provide specific remediation steps for any failures
   - Create actionable improvement recommendations
   - Save results for trend analysis and continuous improvement

## Expected Output

The command will execute specific cargo commands in sequence:
- Run cargo fmt --all -- --check (auto-fix with cargo fmt --all if failed)
- Run cargo clippy with warnings as errors (auto-fix with --fix flags if failed)
- Run cargo build --workspace --verbose (resolve compilation errors if failed)
- Run cargo test --workspace --verbose (fix tests or code optimally if failed)
- Run cargo audit (implement secure patterns if vulnerabilities found)
- Run cargo doc --workspace --no-deps (fix documentation build if failed)
- Run extended quality gates (quality-gates tool, Python tests, Fortitude integration)
- Execute complete pipeline twice for back-to-back validation

## Parameters
- No parameters required
- Uses specific cargo commands with exact flags
- Implements auto-fix protocol for each failure type
- Requires 100% pass rate in back-to-back runs

## Notes
- AI must fix ALL failures using optimal architectural solutions
- Choose architectural improvements over quick patches
- Sequential execution with auto-fix between failures
- Back-to-back validation ensures stability and repeatability
- Non-negotiable requirement: ALL quality gates must pass cleanly