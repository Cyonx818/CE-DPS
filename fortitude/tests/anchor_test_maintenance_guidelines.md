# Anchor Test Maintenance Guidelines

## Overview

This document provides comprehensive maintenance guidelines for Fortitude's anchor test suite, which protects critical functionality from regression during development. These guidelines ensure anchor tests remain effective and relevant as the system evolves.

## Anchor Test Philosophy

### Purpose
Anchor tests are permanent integration tests that:
- **Protect critical workflows** from breaking during refactoring
- **Ensure system stability** across development cycles
- **Validate complete user workflows** rather than individual functions
- **Maintain quality standards** for mission-critical functionality

### When NOT to Modify Anchor Tests
- ❌ **Never delete anchor tests** without explicit justification
- ❌ **Never make anchor tests more permissive** to "fix" failing tests
- ❌ **Never ignore failing anchor tests** - they indicate real problems
- ❌ **Never modify tests to match broken implementations**

## Maintenance Workflow

### 1. Failing Anchor Test Response

When an anchor test fails:

```bash
# 1. Identify the failing test
cargo test anchor --no-capture

# 2. Understand what functionality is protected
# Read the ANCHOR: docstring comment to understand the critical functionality

# 3. Determine the root cause
# Is this a real regression or a legitimate change?

# 4. Fix the implementation, not the test
# Anchor tests should guide implementation fixes
```

#### Decision Matrix for Failing Tests

| Scenario | Action |
|----------|---------|
| **Implementation regression** | Fix the implementation |
| **API change with same semantics** | Update test to match new API |
| **Legitimate requirement change** | Update test with justification |
| **Test environment issue** | Fix test environment, not test |

### 2. Adding New Anchor Tests

Follow the decision matrix from `tests/README.md`:

```
Does this code interact with:
├── External systems (APIs, databases, files)? → CREATE ANCHOR TEST
├── User authentication/authorization? → CREATE ANCHOR TEST  
├── Data persistence (save/load/delete)? → CREATE ANCHOR TEST
├── User input validation? → CREATE ANCHOR TEST
├── Core business logic? → CREATE ANCHOR TEST
└── Otherwise → regular integration test sufficient
```

#### New Anchor Test Checklist

- [ ] **Meets decision matrix criteria** - covers critical functionality
- [ ] **Proper naming** - `test_anchor_[functionality]_[workflow|accuracy|stability]`
- [ ] **ANCHOR: docstring** - explains what functionality is protected
- [ ] **Complete workflow** - tests end-to-end scenarios, not units
- [ ] **Independent execution** - no external dependencies
- [ ] **Both success and failure** - covers normal and error cases
- [ ] **Performance validation** - includes performance requirements where applicable
- [ ] **Security boundaries** - validates security constraints

#### Template for New Anchor Tests

```rust
/// ANCHOR: Verifies [critical functionality] works end-to-end
/// Tests: [specific areas], [performance requirements], [security boundaries]
/// Protects: [what regressions this prevents]
#[tokio::test]
async fn test_anchor_[functionality]_workflow() {
    // Arrange - Set up test environment
    let test_env = create_test_environment().await;
    
    // Act - Execute complete workflow
    let result = execute_critical_workflow(&test_env).await;
    
    // Assert - Validate all critical aspects
    assert!(result.is_ok(), "Critical workflow must succeed");
    
    // Verify performance requirements
    assert!(result.duration < Duration::from_secs(10), "Must meet performance target");
    
    // Verify security boundaries
    assert!(result.validates_security(), "Must maintain security boundaries");
    
    // Verify data integrity
    assert!(result.maintains_data_integrity(), "Must preserve data integrity");
}
```

### 3. Updating Existing Anchor Tests

#### When Updates Are Justified

- **API signature changes** - Method signatures change but semantics remain
- **Configuration schema changes** - Config structure changes but validation remains
- **Response format changes** - Output format changes but content requirements remain
- **Dependency updates** - External library updates change interfaces but not behavior

#### Update Process

1. **Document the change** - Add comments explaining why the update was necessary
2. **Maintain test intent** - Ensure the test still validates the same critical functionality
3. **Update docstring** - Revise ANCHOR: comment if protection scope changes
4. **Validate coverage** - Ensure updated test still provides equivalent protection

#### Example Update Documentation

```rust
/// ANCHOR: Verifies [critical functionality] works end-to-end
/// Tests: [updated areas], [maintained requirements]
/// Protects: [same critical functionality]
/// 
/// UPDATE HISTORY:
/// - 2025-07-11: Updated API calls for new authentication interface (same semantics)
/// - Previous: Used deprecated auth.validate() method
/// - Current: Uses auth.verify_credentials() method with same validation logic
#[tokio::test]
async fn test_anchor_authentication_workflow() {
    // Updated implementation using new API
    let auth_result = auth_manager.verify_credentials(&token).await;
    // Same assertions - functionality protection unchanged
    assert!(auth_result.is_ok(), "Authentication must work");
}
```

## Anchor Test Organization

### File Structure

```
tests/
├── anchor_[feature]_tests.rs          # Feature-specific anchor tests
├── anchor_test_coverage_analysis.md   # Coverage analysis and gaps
├── anchor_test_maintenance_guidelines.md  # This document
└── README.md                          # Testing guidelines with decision matrix

crates/[crate]/tests/
└── anchor_tests.rs                    # Crate-specific anchor tests
```

### Naming Conventions

| Type | Pattern | Example |
|------|---------|---------|
| **Test Files** | `anchor_[feature]_tests.rs` | `anchor_notification_tests.rs` |
| **Test Functions** | `test_anchor_[functionality]_[type]` | `test_anchor_notification_delivery_workflow` |
| **Helper Functions** | `[action]_test_[resource]` | `create_test_notification_system` |

## Quality Assurance

### Code Review Checklist

When reviewing anchor test changes:

- [ ] **Justification provided** - Clear reason for any modifications
- [ ] **Functionality protection maintained** - Test still protects critical workflows
- [ ] **Documentation updated** - ANCHOR: docstring reflects current scope
- [ ] **Performance requirements preserved** - Performance targets still validated
- [ ] **Security boundaries maintained** - Security validations still present
- [ ] **Error scenarios covered** - Failure cases still tested
- [ ] **Test independence verified** - No new external dependencies

### Regular Maintenance Tasks

#### Monthly Review
- [ ] **Execute full anchor test suite** - `cargo test anchor`
- [ ] **Review failing tests** - Investigate any intermittent failures
- [ ] **Check test performance** - Ensure tests complete within reasonable time
- [ ] **Validate documentation** - Ensure ANCHOR: comments remain accurate

#### Quarterly Review
- [ ] **Coverage analysis** - Identify any new critical functionality needing protection
- [ ] **Performance benchmarking** - Validate performance requirements remain achievable
- [ ] **Security review** - Ensure security boundaries remain comprehensive
- [ ] **Documentation update** - Update maintenance guidelines as needed

#### Annual Review
- [ ] **Complete anchor test audit** - Review all tests for continued relevance
- [ ] **Decision matrix validation** - Ensure decision matrix remains appropriate
- [ ] **Test strategy evolution** - Consider improvements to anchor test approach
- [ ] **Training update** - Update developer training on anchor test practices

## Common Scenarios

### Scenario 1: API Breaking Change

```rust
// OLD API (deprecated)
let result = service.process_query(query, context).await;

// NEW API (current)
let request = ProcessRequest::new(query).with_context(context);
let result = service.process(request).await;

// ANCHOR TEST UPDATE
// Update the test to use new API but maintain same validations
#[tokio::test]
async fn test_anchor_query_processing_workflow() {
    // Updated: Use new API structure
    let request = ProcessRequest::new("test query").with_context(test_context);
    let result = service.process(request).await;
    
    // Unchanged: Same critical functionality validation
    assert!(result.is_ok(), "Query processing must succeed");
    assert!(!result.unwrap().answer.is_empty(), "Must provide non-empty answer");
}
```

### Scenario 2: Performance Requirement Change

```rust
/// ANCHOR: Verifies query processing meets performance requirements
/// Tests: Response time under load, memory usage, concurrent handling
/// Protects: System responsiveness and scalability
/// 
/// UPDATE HISTORY:
/// - 2025-07-11: Updated timeout from 30s to 60s due to new LLM integration
/// - Justification: Additional LLM processing requires longer timeout
/// - Validation: Still maintains acceptable user experience
#[tokio::test]
async fn test_anchor_query_performance_requirements() {
    let start = Instant::now();
    let result = process_query("complex query").await;
    let duration = start.elapsed();
    
    // Updated: New performance requirement
    assert!(duration < Duration::from_secs(60), "Must complete within 60 seconds");
    assert!(result.is_ok(), "Must handle complex queries successfully");
}
```

### Scenario 3: New Critical Functionality

```rust
/// ANCHOR: Verifies new multi-modal input processing works end-to-end
/// Tests: Text + image input, validation, security boundaries, performance
/// Protects: Multi-modal input handling, content validation, injection prevention
/// 
/// CREATION JUSTIFICATION:
/// - New feature: Multi-modal input processing (text + images)
/// - Decision matrix: User input processing + Security boundaries
/// - Critical: Handles external user content with security implications
#[tokio::test]
async fn test_anchor_multimodal_input_processing_workflow() {
    // Test new critical functionality
    let input = MultiModalInput::new()
        .with_text("Analyze this image")
        .with_image(test_image_data);
    
    let result = processor.process_multimodal(input).await;
    
    // Validate all critical aspects
    assert!(result.is_ok(), "Multi-modal processing must succeed");
    assert!(result.validates_content(), "Must validate all input content");
    assert!(result.maintains_security(), "Must prevent injection attacks");
}
```

## Tools and Automation

### Running Anchor Tests

```bash
# Run all anchor tests
cargo test anchor

# Run specific anchor test file
cargo test --test anchor_notification_tests

# Run anchor tests with output
cargo test anchor -- --nocapture

# Run anchor tests in specific crate
cargo test -p fortitude-api-server anchor
```

### Anchor Test Metrics

```bash
# Count anchor tests
find . -name "*anchor*.rs" -exec grep -c "fn test_anchor" {} \; | awk '{sum+=$1} END {print "Total anchor tests:", sum}'

# Check anchor test coverage
grep -r "ANCHOR:" tests/ crates/*/tests/ | wc -l

# Find anchor tests missing ANCHOR: documentation
grep -L "ANCHOR:" tests/anchor_*.rs crates/*/tests/anchor*.rs
```

### CI Integration

Ensure anchor tests are run in CI pipeline:

```yaml
# .github/workflows/tests.yml
- name: Run Anchor Tests
  run: cargo test anchor --all-features
  # Anchor tests must pass for PR approval
```

## Troubleshooting

### Common Issues

#### Issue: Anchor test fails after dependency update
**Solution**: Check if dependency change affects API but not functionality. Update test to use new API while maintaining same validations.

#### Issue: Anchor test becomes flaky
**Solution**: Investigate test environment dependencies. Anchor tests should be deterministic. Fix underlying timing or environment issues.

#### Issue: New feature lacks anchor test
**Solution**: Apply decision matrix. If feature meets criteria, create anchor test before merging.

#### Issue: Anchor test takes too long
**Solution**: Review test implementation for efficiency. Anchor tests should complete quickly. Consider mocking expensive operations while maintaining critical validation.

## Documentation Standards

### ANCHOR: Docstring Format

```rust
/// ANCHOR: Verifies [critical functionality] works end-to-end  
/// Tests: [specific areas tested], [performance requirements], [security boundaries]
/// Protects: [specific regressions prevented], [critical workflows preserved]
/// 
/// [Optional: UPDATE HISTORY or CREATION JUSTIFICATION]
```

### Test Documentation Requirements

1. **Clear purpose** - What critical functionality is protected
2. **Specific scope** - What exactly is tested
3. **Protection goal** - What regressions are prevented
4. **Maintenance history** - Record of updates and justifications

## Conclusion

Anchor tests are critical infrastructure for maintaining system stability. These guidelines ensure they remain effective protection against regressions while evolving with the system. 

**Remember**: Anchor tests should guide implementation fixes, not be modified to accommodate broken implementations. They are the guardians of critical functionality.

For questions or guidance on anchor test maintenance, refer to the decision matrix in `tests/README.md` or consult the development team.