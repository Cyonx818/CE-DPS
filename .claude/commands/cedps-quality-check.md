# <context>CE-DPS Quality Validation Command</context>

<meta>
  <title>CE-DPS Quality Validation Command</title>
  <type>quality-command</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-16</updated>
</meta>

## <summary priority="critical">Quality Gate Execution</summary>

- **Purpose**: Comprehensive CI/CD test suite with auto-fix capability
- **Authority**: AI must fix ALL failures using optimal solutions for software health
- **Scope**: Code formatting → linting → build → tests → security → documentation
- **Success Criteria**: ALL quality gates pass cleanly in back-to-back runs
- **Non-Negotiable**: Fix broken tests or broken code using best architectural solutions

## <implementation priority="critical">Quality Validation Pipeline</implementation>

### <phase>Phase 1: Code Quality Foundation</phase>

```bash
# ANCHOR: Rust formatting validation
cargo fmt --all -- --check
```

**Auto-fix if failed:**
```bash
cargo fmt --all
```

**Success Criteria**: Zero formatting violations

### <phase>Phase 2: Linting Validation</phase>

```bash
# ANCHOR: Clippy linting with warnings as errors
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

**Auto-fix if failed:**
```bash
cargo clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged
```

**Success Criteria**: Zero clippy warnings or errors

### <phase>Phase 3: Build Validation</phase>

```bash
# ANCHOR: Workspace build verification
cargo build --workspace --verbose
```

**Success Criteria**: Clean build with no compilation errors

### <phase>Phase 4: Test Suite Validation</phase>

```bash
# ANCHOR: Comprehensive test execution
cargo test --workspace --verbose
```

**Success Criteria**: All tests pass, >95% coverage maintained

### <phase>Phase 5: Security Validation</phase>

```bash
# ANCHOR: Security audit validation
cargo audit
```

**Success Criteria**: No critical security vulnerabilities

### <phase>Phase 6: Documentation Validation</phase>

```bash
# ANCHOR: Documentation build validation
cargo doc --workspace --no-deps
```

**Success Criteria**: Documentation builds without errors

## <validation priority="high">Extended Quality Gates</validation>

### <tool>Quality Gates Tool</tool>

```bash
# ANCHOR: Quality gates validation
cargo build --bin quality-gates
./target/debug/quality-gates --project-path .
```

**Purpose**: CE-DPS-specific quality validation

### <tool>Python Integration Tests</tool>

```bash
# ANCHOR: Python test validation
python3 -m pytest tools/phase-validator.py --doctest-modules -v
```

**Purpose**: Phase validation tool testing

### <tool>Fortitude Integration</tool>

```bash
# ANCHOR: Fortitude integration validation
cargo run --bin fortitude-integration -- check
```

**Purpose**: Knowledge management system validation

## <constraints priority="critical">Non-Negotiable Requirements</constraints>

### <constraint>Intelligent Fix Requirement</constraint>

**AI Authority**: You MUST fix ALL failures using the best possible solution for software health and longevity
- **Test Failure**: Fix broken test OR fix broken code (choose optimal solution)
- **Build Failure**: Resolve compilation errors with sound architectural decisions
- **Lint Failure**: Address warnings/errors unless fixing degrades architecture
- **Security Failure**: Resolve all critical vulnerabilities with secure patterns

**Fix Definition**: "Fix" means implementing the best possible solution for the software's health and longevity, which may involve:
- Correcting existing code when the issue is implementation-level
- Architecting and implementing new code when current design is fundamentally flawed
- Refactoring to improve architectural soundness
- Choosing architectural improvements over quick patches

**Exception Protocol**: If fixing a test/warning would degrade architecture, implement better architecture instead of band-aid fixes

### <constraint>Back-to-Back Validation</constraint>

**Success Definition**: 
1. Run complete quality pipeline
2. Achieve 100% pass rate
3. Run pipeline second time
4. Achieve 100% pass rate again

**Purpose**: Ensure stability and repeatability

## <troubleshooting priority="medium">Common Fix Patterns</troubleshooting>

### <pattern>Formatting Issues</pattern>

```bash
# Auto-fix formatting
cargo fmt --all
```

### <pattern>Clippy Warnings</pattern>

```rust
// Common fixes
#[allow(clippy::too_many_arguments)]  // For complex functions
#[allow(dead_code)]                   // For development code
#[allow(unused_variables)]            // For placeholder variables
```

### <pattern>Test Failures</pattern>

```bash
# Run specific test for debugging
cargo test test_name -- --nocapture

# Run tests in single thread
cargo test -- --test-threads=1
```

### <pattern>Build Failures</pattern>

```bash
# Clean build
cargo clean
cargo build --workspace --verbose
```

## <execution priority="critical">Command Execution Flow</execution>

**Sequential Execution Required:**
1. Format check → auto-fix if needed → verify pass
2. Clippy check → implement optimal solutions → verify pass  
3. Build check → resolve with sound architecture → verify pass
4. Test execution → fix with best architectural approach → verify pass
5. Security audit → implement secure patterns → verify pass
6. Documentation build → fix with clear documentation → verify pass
7. Extended tools → resolve with optimal solutions → verify pass

**Final Validation:**
- Complete pipeline execution #1 → 100% pass
- Complete pipeline execution #2 → 100% pass
- Success: Ready for next development phase

**Failure Protocol:**
- Any failure → implement best solution for software health → re-run from failed step
- Evaluate: Quick fix vs architectural improvement (choose architectural improvement)
- Continue until ALL steps pass cleanly with sound architecture
- No progression until quality gates satisfied with optimal solutions