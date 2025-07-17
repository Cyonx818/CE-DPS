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

```
🚀 Running CE-DPS Complete Quality Check...

🔄 Pre-Flight Checks:
   ✅ Project state validated
   ✅ Tools and dependencies verified
   ✅ Git working directory clean
   ✅ Environment configuration confirmed

🧪 Comprehensive Test Suite:
   ✅ Unit Tests: 287/287 passed (97.8% coverage)
   ✅ Integration Tests: 45/45 passed
   ✅ Security Tests: 23/23 passed
   ✅ Performance Tests: 12/12 passed
   ✅ E2E Tests: 8/8 passed

🔧 Code Quality with Auto-Fix:
   ✅ Linting: Auto-fixed 3 formatting issues
   ✅ Code Formatting: Applied consistent style
   ✅ Complexity Analysis: All functions within limits
   ✅ Standards Compliance: CE-DPS standards met

🔒 Security and Compliance:
   ✅ Vulnerability Scan: 0 critical, 0 high, 1 low
   ✅ Secrets Scan: No exposed credentials
   ✅ Input Validation: Comprehensive coverage
   ✅ Auth/Authorization: Properly implemented

⚡ Performance and Scalability:
   ✅ Response Times: 95th percentile 156ms (<200ms target)
   ✅ Memory Usage: Optimized, within limits
   ✅ Load Testing: Handles 1000+ concurrent users
   ✅ Database Performance: All queries optimized

📚 Documentation and API:
   ✅ API Documentation: 100% endpoint coverage
   ✅ API Contract: All endpoints validated
   ✅ Code Documentation: 92% coverage
   ✅ Operations Docs: Complete and current

🎯 QUALITY CHECK RESULT: ✅ PASSED

📊 Quality Metrics Summary:
   - Overall Score: 98/100
   - Test Coverage: 97.8%
   - Security Score: A+ (1 low-priority finding)
   - Performance: Exceeds targets
   - Documentation: Comprehensive

Auto-Fixes Applied:
✅ Fixed 3 code formatting issues
✅ Updated 2 documentation templates
✅ Resolved 1 minor linting warning

Quality Report: docs/quality-reports/complete-quality-check-[timestamp].json

🎉 All CE-DPS Quality Gates PASSED!

Recommendations for Continuous Improvement:
1. Address 1 low-priority security finding (dependency update)
2. Consider adding more edge case test scenarios
3. Monitor performance trends over time

Next Steps:
✅ Ready to proceed with development
✅ Quality standards maintained
✅ All CI/CD gates satisfied

💡 Project meets all CE-DPS quality requirements for production deployment
```

## Notes
- Comprehensive validation of all quality dimensions
- Auto-fix capabilities for common issues
- Strict quality gate enforcement with detailed reporting
- Actionable recommendations for continuous improvement
- Clear pass/fail status with complete reasoning