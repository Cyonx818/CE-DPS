# CE-DPS Quality Tools

Run CE-DPS quality gates and validation tools with comprehensive reporting.

## Instructions

1. **Validate Environment**
   - Check that we're in a CE-DPS project (docs/ce-dps-state.json exists)
   - Verify current phase and determine appropriate quality checks
   - Ensure necessary tools are available (compiler, test runner, linting tools)

2. **Execute Code Quality Checks**
   - Run linting and code formatting validation
   - Check code complexity and maintainability metrics
   - Validate coding standards compliance
   - Verify documentation coverage for public APIs

3. **Run Test Suite with Coverage**
   - Execute complete test suite (unit, integration, security tests)
   - Generate test coverage report
   - Validate coverage meets >95% requirement for business logic
   - Check that all critical paths have test coverage

4. **Perform Security Validation**
   - Run security vulnerability scanning
   - Check for common security issues (SQL injection, XSS, etc.)
   - Validate input sanitization and validation
   - Verify authentication and authorization implementation

5. **Execute Performance Testing**
   - Run performance benchmarks for critical paths
   - Validate API response times meet <200ms requirement
   - Check memory usage and resource efficiency
   - Test database query performance

6. **Validate Documentation**
   - Check API documentation completeness
   - Verify code comments and documentation quality
   - Validate deployment and troubleshooting guides
   - Ensure README and project documentation is current

7. **Generate Quality Report**
   - Create comprehensive quality metrics summary
   - Document any issues found and recommended fixes
   - Generate actionable recommendations for improvements
   - Save results to docs/quality-reports/ with timestamp

8. **Provide Recommendations**
   - Identify areas needing improvement
   - Suggest specific actions to address quality issues
   - Recommend next steps based on current phase
   - Provide guidance for maintaining quality standards

## Expected Output

The command will execute bash commands and conditional checks that:
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

## Parameters
- No parameters required
- Uses jq to read project state and current phase
- Checks for various tool availability before execution
- Provides conditional execution based on tool availability

## Notes
- Comprehensive quality validation using actual bash commands
- Tool availability affects which checks can be executed
- Provides installation instructions for missing tools
- Supports all CE-DPS phases with appropriate quality standards
- Generates actionable reports with success/failure indicators