# Testing Guide

<meta>
  <title>Testing Guide</title>
  <type>reference</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-09</updated>
</meta>

## <summary priority="high">Testing Strategy</summary>

Comprehensive testing guidance for Fortitude project. Follow these guidelines to ensure consistent, reliable, and maintainable test coverage using Rust testing frameworks.

## <navigation>Quick Navigation</navigation>

- [Testing Architecture](#testing-architecture) - Multi-layered testing approach
- [Unit Testing](#unit-testing) - Individual function testing
- [Integration Testing](#integration-testing) - Component interaction testing
- [Property-Based Testing](#property-based-testing) - Property verification with proptest
- [Anchor Tests](#anchor-tests) - Permanent regression tests
- [Test Organization](#test-organization) - File structure and organization
- [Best Practices](#best-practices) - Quality guidelines

## <architecture>Testing Architecture</architecture>

Multi-layered testing approach with Rust's built-in test framework:

- **Unit Tests**: Test individual functions and methods in isolation using `#[test]`
- **Integration Tests**: Test component interactions and full workflows
- **Property-Based Tests**: Verify system properties across input ranges using `proptest`
- **Performance Tests**: Test performance characteristics with `criterion`
- **Anchor Tests**: Permanent regression tests for critical functionality

## <guidelines>Testing Guidelines</guidelines>

### <principles>Core Principles</principles>
1. **Test Behavior, Not Implementation**: Focus on what the code should do
2. **Test Happy Path and Edge Cases**: Cover expected usage and boundary conditions
3. **Test Error Conditions**: Verify proper error handling and recovery
4. **Keep Tests Independent**: Each test should run in isolation
5. **Use Descriptive Names**: Test names should clearly describe what is being tested
6. **Follow AAA Pattern**: Arrange, Act, Assert

### <guidelines-rust>Rust-Specific Guidelines</guidelines-rust>
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_loads_required_values() {
        // Arrange
        let config = AppConfig::new();
        
        // Act & Assert
        assert!(config.is_valid());
    }
}
```

- **Use Test Modules**: Organize tests in `#[cfg(test)]` modules within source files
- **Integration Tests**: Place in `tests/` directory for cross-crate testing
- **Mock External Dependencies**: Use `mockall` crate for mocking traits and structs

## Unit Testing

### Guidelines for Unit Tests

Unit tests are organized within the same files as the code being tested using `#[cfg(test)]` modules.

#### Example Structure:
```rust
// src/config.rs
use std::env;

pub struct AppConfig {
    pub api_key: Option<String>,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            api_key: env::var("API_KEY").ok(),
        }
    }
    
    pub fn validate(&self) -> bool {
        self.api_key.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_config_loads_api_key() {
        // Use a unique key to avoid test interference
        let test_key = format!("TEST_KEY_{}", std::process::id());
        env::set_var(&test_key, "test_value");
        
        // Test with explicit cleanup
        struct TestGuard(String);
        impl Drop for TestGuard {
            fn drop(&mut self) {
                env::remove_var(&self.0);
            }
        }
        let _guard = TestGuard(test_key.clone());
        
        // Modified to use test key
        env::set_var("API_KEY", "test_key");
        let config = AppConfig::new();
        assert_eq!(config.api_key, Some("test_key".to_string()));
    }
    
    #[test]
    fn test_validate_returns_true_with_api_key() {
        let config = AppConfig {
            api_key: Some("test_key".to_string()),
        };
        assert!(config.validate());
    }
    
    #[test]
    fn test_validate_returns_false_without_api_key() {
        let config = AppConfig { api_key: None };
        assert!(!config.validate());
    }
}
```

#### Unit Test Guidelines:
- Use `#[test]` attribute for test functions
- Test public interface, not private implementation
- Use descriptive test names that explain the scenario
- Test edge cases (empty strings, None/Option values, invalid inputs)
- Use Rust's built-in `assert!`, `assert_eq!`, `assert_ne!` macros
- Group related tests in `mod tests` blocks within source files
- Use `#[should_panic]` for testing expected panics

## Integration Testing

Integration tests are in `tests/` directory and test the **public API** without mocking internal components.

### Structure for Integration Tests

```rust
//! Integration tests for the public API.
//! These tests should use the crate as an external user would.

use your_crate::{Config, Application};
use std::time::Duration;

#[tokio::test]
async fn test_application_workflow() {
    // Test the public API without mocking internals
    let config = Config {
        api_endpoint: "http://localhost:8080".to_string(),
        timeout: Duration::from_secs(10),
    };
    
    let app = Application::new(config).expect("Failed to create application");
    
    // Test actual integration - this might require test infrastructure
    // or use test-specific configurations
    let result = app.process_data("sample input").await;
    
    match result {
        Ok(output) => {
            assert!(!output.is_empty());
            assert!(output.contains("expected_pattern"));
        }
        Err(e) => panic!("Integration test failed: {:?}", e),
    }
}

#[test]
fn test_config_validation() {
    // Test public configuration validation
    let invalid_config = Config {
        api_endpoint: "".to_string(),  // Invalid empty endpoint
        timeout: Duration::from_secs(0),  // Invalid zero timeout
    };
    
    let result = Application::new(invalid_config);
    assert!(result.is_err(), "Should reject invalid configuration");
}
```

**Key Principle**: Integration tests should test your crate's public interface as an external user would, without mocking internal components.

### Integration Test Guidelines:
- Place integration tests in `tests/` directory
- **Test the public API only** - don't mock internal components
- Test realistic scenarios that external users would encounter
- Use `tests/common/mod.rs` for shared test utilities
- Test both success and failure paths using `Result<T, E>` patterns
- Focus on component boundaries and external interfaces
- Use test-specific configurations rather than mocks when possible
- Use `#[tokio::test]` for async integration tests
- Consider using test containers or embedded databases for external dependencies

## Property-Based Testing

Use the `proptest` crate for property-based testing:

```rust
use proptest::prelude::*;
use crate::pattern_analyzer::PatternAnalyzer;

proptest! {
    #[test]
    fn test_analyze_patterns_always_returns_valid_structure(
        log_messages in prop::collection::vec(".*", 1..100)
    ) {
        let analyzer = PatternAnalyzer::new();
        let result = analyzer.analyze_patterns(&log_messages);
        
        // Property assertions
        prop_assert!(result.is_ok());
        let patterns = result.unwrap();
        prop_assert!(!patterns.is_empty() || log_messages.is_empty());
    }
    
    #[test]
    fn test_similarity_score_is_symmetric(
        text1 in ".*",
        text2 in ".*"
    ) {
        let analyzer = PatternAnalyzer::new();
        let score1 = analyzer.calculate_similarity(&text1, &text2);
        let score2 = analyzer.calculate_similarity(&text2, &text1);
        
        prop_assert!((score1 - score2).abs() < 0.001); // Account for floating point precision
    }
}
```

### Property Test Guidelines:
- Use `proptest!` macro to define property-based tests
- Define meaningful properties that should always hold
- Use appropriate strategies from `proptest::prelude::*` for your application domain
- Test invariants (things that should never change)
- Test postconditions (what should be true after operations)
- Use `prop_assert!` for assertions within property tests

## End-to-End Testing

End-to-end tests should be part of your integration tests and test complete user workflows.

### CLI Testing in Integration Tests:

```rust
// In tests/integration_test.rs or tests/cli_test.rs
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_cli_basic_functionality() {
    let temp_dir = tempdir().unwrap();
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .env("TEST_CONFIG_DIR", temp_dir.path())
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("USAGE:"));
}

#[test]
#[ignore] // Mark as slow test
fn test_cli_full_workflow() {
    let output = Command::new("cargo")
        .args(["run", "--", "process", "--input", "test_data"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Processing complete"));
}
```

## <anchor>Anchor Tests</anchor>

### <summary priority="high">Anchor Test Definition</summary>
**Anchor tests** are permanent integration tests that protect critical system functionality from regression. They verify core workflows and must be maintained as the system evolves.

### <criteria>Mandatory Anchor Test Criteria</criteria>
### <anchor-test-decision-framework>
<command>CREATE anchor tests using this decision matrix:</command>

<decision-criteria>
  <conditions>
    <external_api_integration>boolean</external_api_integration>
    <data_persistence>boolean</data_persistence>
    <authentication>boolean</authentication>
    <user_input_processing>boolean</user_input_processing>
    <critical_error_handling>boolean</critical_error_handling>
    <business_logic>boolean</business_logic>
    <api_compatibility>boolean</api_compatibility>
    <cross_component_integration>boolean</cross_component_integration>
    <type_definition_changes>boolean</type_definition_changes>
  </conditions>
  
  <decision_rule>
    <if>external_api_integration OR data_persistence OR authentication OR user_input_processing OR critical_error_handling OR business_logic OR api_compatibility OR cross_component_integration OR type_definition_changes</if>
    <then>CREATE_ANCHOR_TEST</then>
    <else>REGULAR_INTEGRATION_TEST</else>
  </decision_rule>
</decision-criteria>

<criteria_definitions>
  <criterion name="external_api_integration">
    <description>HTTP requests, database connections, file system operations</description>
    <examples>REST API calls, SQL queries, file read/write</examples>
  </criterion>
  
  <criterion name="data_persistence">
    <description>Save, load, delete operations affecting system state</description>
    <examples>Database writes, config file updates, cache operations</examples>
  </criterion>
  
  <criterion name="authentication">
    <description>Login, permissions, token validation functionality</description>
    <examples>User verification, role checks, session management</examples>
  </criterion>
  
  <criterion name="user_input_processing">
    <description>Form validation, command parsing, data transformation</description>
    <examples>CLI argument parsing, JSON validation, input sanitization</examples>
  </criterion>
  
  <criterion name="critical_error_handling">
    <description>Failure recovery, graceful degradation for system stability</description>
    <examples>Network timeout handling, corrupted data recovery</examples>
  </criterion>
  
  <criterion name="business_logic">
    <description>Calculations, algorithms, core domain operations</description>
    <examples>Pricing calculations, search algorithms, workflow processing</examples>
  </criterion>
  
  <criterion name="api_compatibility">
    <description>Changes to public interfaces, method signatures, or return types</description>
    <examples>Adding/removing struct fields, changing enum variants, modifying function signatures</examples>
  </criterion>
  
  <criterion name="cross_component_integration">
    <description>Interactions between separate modules, crates, or system boundaries</description>
    <examples>Pipeline → Classifier, Storage → API, Core types → Test files</examples>
  </criterion>
  
  <criterion name="type_definition_changes">
    <description>Modifications to core data structures used across components</description>
    <examples>ResearchResult, ClassificationMetadata, DocumentMetadata, CacheStats</examples>
  </criterion>
</criteria_definitions>
</anchor-test-decision-framework>

**Fortitude-Specific Anchor Test Requirements** (objective criteria):
- **External API integrations** (HTTP requests, database connections, file system)
- **Data persistence operations** (save, load, delete operations)
- **Authentication and authorization** (login, permissions, token validation)
- **User input processing** (form validation, command parsing, data transformation)
- **Error handling for critical paths** (failure recovery, graceful degradation)
- **Business logic functions** (calculations, algorithms, core domain operations)
- **API compatibility changes** (struct fields, enum variants, method signatures)
- **Cross-component integration** (module boundaries, crate dependencies, system interfaces)
- **Core type definition changes** (shared data structures, serialization contracts)

### <workflow>Anchor Test Creation Workflow</workflow>
**Fortitude Decision Matrix** (use during Phase 3 implementation):
```
Does this code interact with:
├── External systems (APIs, databases, files)? → CREATE ANCHOR TEST
├── User authentication/authorization? → CREATE ANCHOR TEST
├── Data persistence (save/load/delete)? → CREATE ANCHOR TEST
├── User input validation? → CREATE ANCHOR TEST
├── Core business logic? → CREATE ANCHOR TEST
├── API compatibility (field access, method signatures)? → CREATE ANCHOR TEST
├── Cross-component integration (module boundaries)? → CREATE ANCHOR TEST
├── Core type definitions (shared data structures)? → CREATE ANCHOR TEST
└── Otherwise → regular integration test sufficient
```

<parallel-verification>
**Testing Subagents**: Use parallel subagents when no conflicts exist, sequential otherwise

**When to Use Parallel**: Independent test types (unit + integration, security + performance)
**When to Use Sequential**: Tests modifying same files or shared test data
**Implementation**: See [Parallel Subtask Rules](../methodology/parallel_subtask_rules.md) for:
- Testing workflow coordination patterns
- Conflict detection and resolution strategies
- Context inheritance for testing subagents
</parallel-verification>

### <integration>Process Integration</integration>
**Phase 3 Implementation Checklist** (mandatory):
- [ ] Implementation complete
- [ ] Unit tests passing
- [ ] Integration tests passing
- [ ] **Anchor tests created** (use decision matrix above)
- [ ] **Anchor tests documented** with `ANCHOR:` docstring comments
- [ ] All tests passing (`cargo test`)

### <template>Anchor Test Templates</template>

#### <template>External API Integration Template</template>
```rust
//! Anchor integration tests for core application functionality.
//!
//! These tests verify critical functionality and should be maintained
//! as the system evolves. Do not delete these tests.

use mockall::predicate::*;
use your_crate::Application;

/// ANCHOR: Verifies external API integration works end-to-end.
/// Tests: HTTP requests, error handling, data transformation
#[tokio::test]
async fn test_anchor_external_api_integration() {
    // Template for external API integration testing
    let mut app = Application::new();
    
    // Set up mock expectations for external API
    app.api_client
        .expect_fetch_data()
        .returning(|| {
            Ok(vec![
                serde_json::json!({"id": "1", "status": "active"}),
                serde_json::json!({"id": "2", "status": "inactive"}),
            ])
        });
    
    let data = app.fetch_external_data().await.unwrap();
    
    assert!(!data.is_empty());
    assert!(data.iter().all(|entry| entry.get("id").is_some()));
    assert!(data.iter().all(|entry| entry.get("status").is_some()));
}
```

#### <template>Data Persistence Template</template>
```rust
/// ANCHOR: Verifies data persistence operations work end-to-end.
/// Tests: Save, load, delete operations with error handling
#[tokio::test]
async fn test_anchor_data_persistence_workflow() {
    // Template for data persistence testing
    let mut app = Application::new();
    let test_data = TestData::new("test_id", "test_content");
    
    // Test save operation
    app.save_data(&test_data).await.unwrap();
    
    // Test load operation
    let loaded_data = app.load_data("test_id").await.unwrap();
    assert_eq!(loaded_data.id, "test_id");
    assert_eq!(loaded_data.content, "test_content");
    
    // Test delete operation
    app.delete_data("test_id").await.unwrap();
    
    // Verify deletion
    let result = app.load_data("test_id").await;
    assert!(result.is_err());
}
```

#### <template>User Input Processing Template</template>
```rust
/// ANCHOR: Verifies user input processing works end-to-end.
/// Tests: Validation, transformation, error handling
#[tokio::test]
async fn test_anchor_user_input_processing() {
    // Template for user input processing testing
    let mut app = Application::new();
    
    // Test valid input
    let valid_input = UserInput::new("valid@email.com", "strong_password123");
    let result = app.process_user_input(valid_input).await.unwrap();
    assert!(result.is_valid());
    
    // Test invalid input
    let invalid_input = UserInput::new("invalid-email", "weak");
    let result = app.process_user_input(invalid_input).await;
    assert!(result.is_err());
}
```

### <guidelines>Anchor Test Guidelines</guidelines>
**Naming Convention** (mandatory):
- Use descriptive names: `test_anchor_[functionality]_workflow`
- Add `ANCHOR:` comment in docstring explaining what critical functionality is protected
- Include test scope in docstring: `/// Tests: [specific areas covered]`

**Implementation Requirements**:
- Place anchor tests in integration test files in `tests/` directory
- Use `#[test]` or `#[tokio::test]` (not `#[ignore]`) so they run by default
- Cover complete workflows, not individual functions
- Test both success and failure paths
- Update when APIs change, never delete

**Quality Standards**:
- Each anchor test must verify end-to-end functionality
- Include comprehensive error handling verification
- Test realistic data scenarios
- Document expected behavior and test scope

**Multi-Perspective Quality Assurance**: Use parallel testing subagents when tests are independent (different test types, different domains). Use sequential subagents when conflicts exist (same files, shared data). See [Parallel Subtask Rules](../methodology/parallel_subtask_rules.md) for testing workflow coordination patterns.

## Test Organization

### File Structure:
```
project/
├── src/
│   ├── lib.rs                     # Unit tests in #[cfg(test)] modules
│   ├── config.rs                  # Unit tests in #[cfg(test)] modules
│   └── ...
├── tests/                         # Integration tests only
│   ├── integration_test.rs        # Basic integration tests
│   └── common/                    # Shared test utilities
│       └── mod.rs
├── benches/                       # Performance benchmarks
│   └── benchmarks.rs
└── examples/                      # Example code (often tested)
    └── basic_usage.rs
```

**Key Principles:**
- **Unit tests**: In `#[cfg(test)]` modules within source files
- **Integration tests**: In `tests/` directory, test the public API
- **Benchmarks**: In `benches/` directory with `criterion`
- **Examples**: In `examples/` directory, often serve as documentation tests

### Test Attributes and Organization

Use Rust's built-in test attributes:

```rust
#[test]                            // Standard unit test
#[tokio::test]                     // Async test (requires tokio-test)
#[cfg(test)]                       // Test-only module
#[ignore]                          // Slow tests (run with --ignored)
#[should_panic]                    // Expected panic
#[should_panic(expected = "msg")]  // Expected panic with message
```

**Test Organization Strategy:**
- **Unit tests**: `#[cfg(test)]` modules in source files
- **Integration tests**: Files in `tests/` directory
- **Documentation tests**: In docstrings (tested with `cargo test --doc`)
- **Slow tests**: Use `#[ignore]` and run with `cargo test -- --ignored`

```rust
// Example of proper test organization
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fast_function() {
        // Fast unit test
    }
    
    #[test]
    #[ignore]
    fn test_slow_integration() {
        // Slow test, run separately
    }
}
```

### Temporary Test Files:
**Important**: Tests created temporarily for troubleshooting should follow Rust conventions:

- Use `#[ignore]` attribute for tests under development
- Place temporary unit tests in `#[cfg(test)]` modules
- Use descriptive names prefixed with `temp_` or `debug_`
- Move to proper integration tests in `tests/` only when permanent
- Clean up temporary tests before committing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[ignore] // Remove when ready
    fn temp_debug_issue_123() {
        // Temporary debugging test
    }
}
```

## Best Practices

### Documentation
- Document test purpose and expected behavior
- Use docstrings for test methods and classes
- Explain non-obvious assertions
- Reference relevant issues or requirements

### Error Messages
- Use descriptive assertion messages:
  ```rust
  assert_eq!(result.len(), 3, "Expected exactly 3 errors, got {}", result.len());
  // Or with custom messages
  assert!(result.is_ok(), "Operation failed: {:?}", result.err());
  ```
- Include context in error assertions
- Test specific error types and messages with `Result<T, E>` patterns:
  ```rust
  #[test]
  fn test_error_handling() {
      let result = risky_operation();
      match result {
          Err(MyError::InvalidInput(msg)) => assert_eq!(msg, "expected message"),
          _ => panic!("Expected InvalidInput error"),
      }
  }
  ```

### Test Data
- Use realistic test data from your application domain
- Create test utilities in `tests/common/mod.rs` for shared setup:
  ```rust
  // tests/common/mod.rs
  pub fn sample_config() -> Config {
      Config {
          api_key: "test_key".to_string(),
          timeout: std::time::Duration::from_secs(30),
      }
  }
  ```
- Use constants for magic numbers and strings:
  ```rust
  const TEST_TIMEOUT: Duration = Duration::from_secs(5);
  const SAMPLE_DATA: &str = "test data";
  ```
- Make test data maintainable and readable

### Performance
- Keep tests fast - aim for milliseconds, not seconds
- Use mocking for expensive operations (API calls, file I/O) with `mockall`
- Rust runs tests in parallel by default; use `--test-threads=1` to serialize if needed
- Use `#[ignore]` for slow tests and run separately:
  ```bash
  cargo test                    # Fast tests only
  cargo test -- --ignored      # Slow tests only
  cargo test -- --include-ignored  # All tests
  ```
- Profile slow tests with `cargo test --release` for faster execution

### Maintenance
- Regularly review and update tests
- Remove obsolete tests when refactoring
- Keep test dependencies minimal
- Update tests when requirements change

## Running Tests

```bash
# Run all tests
cargo test

# Run unit tests only (tests in src/ files)
cargo test --lib

# Run integration tests only
cargo test --test '*'

# Run specific test
cargo test test_config

# Run tests matching pattern
cargo test config

# Run tests with verbose output
cargo test -- --nocapture

# Run tests with specific features
cargo test --features integration-tests
cargo test --features e2e-tests

# Run only anchor tests (by naming convention)
cargo test anchor

# Run tests in release mode (faster execution)
cargo test --release

# Run performance benchmarks
cargo bench

# Skip slow/ignored tests
cargo test -- --skip ignored

# Run tests with thread count
cargo test -- --test-threads=1
```

## Coverage

Monitor test coverage to ensure comprehensive testing:

```bash
# Install cargo-tarpaulin for coverage
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out html --output-dir coverage

# Generate coverage with specific features
cargo tarpaulin --features integration-tests --out html

# View coverage report
open coverage/tarpaulin-report.html

# Alternative: use cargo-llvm-cov
cargo install cargo-llvm-cov
cargo llvm-cov --html
open target/llvm-cov/html/index.html
```

Aim for >90% coverage on core business logic while focusing on meaningful tests over coverage percentage.

## Test Development and Troubleshooting Directive

<test-development-workflow>
  <command>FOLLOW this workflow when developing or troubleshooting tests:</command>
  
  <steps>
    <step priority="1" action="validate">
      <command>VERIFY test process is possible given current codebase status</command>
      <validation>Confirm dependencies exist and application state supports test</validation>
    </step>
    
    <step priority="2" action="fix_tests">
      <command>FIX failing tests to work as intended (DO NOT make permissive)</command>
      <exception>Only make permissive if testing process itself is fundamentally wrong</exception>
    </step>
    
    <step priority="3" action="fix_source">
      <command>FIX source code when tests reveal implementation issues</command>
      <rationale>Core purpose of testing is identifying and fixing implementation bugs</rationale>
    </step>
    
    <step priority="4" action="preserve">
      <command>PRESERVE functionality in both source code and tests</command>
      <constraint>Do not lose intended behavior from either code or tests</constraint>
    </step>
    
    <step priority="5" action="guide_fixes">
      <command>USE failing tests to guide implementation corrections</command>
      <approach>Fix broken implementations rather than adjusting tests to match</approach>
    </step>
  </steps>
</test-development-workflow>

**Fortitude-Specific Testing Guidelines**:

1. **Test Process Validation**: First confirm that the test process being attempted should be possible given the current status of the application codebase.

2. **Fix Tests, Not Make Them Permissive**: If a test should be possible but fails, fix the test to work as intended. Do not make tests more permissive unless the testing process itself is fundamentally wrong.

3. **Fix Source Code When Tests Reveal Issues**: If a test does not complete successfully but should, fix the sections of code that the test is testing. This is the core purpose of running tests - to identify and fix issues in the implementation.

4. **Preserve Functionality**: Do not lose functionality from either the source code or the test in an attempt to "fix" the test. Both the code and tests should maintain their intended behavior.

5. **Test-Driven Fixes**: Use failing tests as a guide to identify what needs to be implemented or corrected in the source code, rather than adjusting tests to match broken implementations.

6. **Missing Functionality Workarounds**: When testing errors are caused by missing functionality in the application codebase:
   - Work around the issue temporarily in the test
   - Document the workaround in the test using comments, noting that when the functionality is implemented, the test needs to be adjusted
   - Track the missing functionality in the project's documentation or issue tracker