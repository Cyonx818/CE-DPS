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

```
🔧 Running CE-DPS Quality Tools...

📊 Code Quality Checks:
   ✅ Linting: All files pass
   ✅ Formatting: Consistent style maintained
   ✅ Complexity: All functions within limits
   ✅ Documentation: 94% API coverage

🧪 Test Suite Results:
   ✅ Unit Tests: 287 passed, 0 failed
   ✅ Integration Tests: 45 passed, 0 failed  
   ✅ Security Tests: 23 passed, 0 failed
   ✅ Coverage: 97.3% (Exceeds >95% requirement)

🔒 Security Validation:
   ✅ Vulnerability Scan: 0 critical, 2 low issues
   ✅ Input Validation: Comprehensive coverage
   ✅ Authentication: Properly implemented
   ✅ Authorization: Role-based access working

⚡ Performance Testing:
   ✅ API Response Times: 95th percentile 167ms
   ✅ Memory Usage: Within acceptable limits
   ✅ Database Performance: All queries optimized
   ✅ Load Testing: Handles expected traffic

📚 Documentation Validation:
   ✅ API Documentation: 100% endpoint coverage
   ✅ Code Comments: Adequate coverage
   ✅ Deployment Guides: Complete and current
   ⚠️ README: Needs minor updates

Quality Report Generated: docs/quality-reports/quality-check-[timestamp].json

🎯 Overall Quality Score: 96/100

Recommendations:
1. Address 2 low-priority security findings
2. Update README with recent feature additions
3. Consider adding more integration test scenarios

Next Steps:
- Address identified issues
- Re-run quality check to validate fixes
- Continue with current phase development

💡 All critical quality gates are passing ✅
```

## Notes
- Comprehensive quality validation across all dimensions
- Generate actionable reports with specific recommendations
- Adapt quality checks based on current project phase
- Provide clear guidance for addressing any issues found