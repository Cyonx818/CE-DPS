# Development Validation Commands

<meta>
  <title>Development Validation Commands</title>
  <type>reference</type>
  <audience>ai_assistant</audience>
  <complexity>basic</complexity>
  <updated>2025-07-11</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Quick reference for validation commands preventing technical debt accumulation
- **Key Approach**: Systematic validation throughout development
- **Core Benefits**: Early error detection, compilation verification, dependency validation
- **When to use**: Development milestones, API changes, pre-commit verification
- **Related docs**: [API Compatibility Testing](../patterns/api-compatibility-testing.md), [Testing Strategy](../../tests/README.md)

## <validation>Development Validation Commands</validation>

### <stage>Sprint Planning Validation</stage>

**API Compatibility Check**:
```bash
# Verify all targets compile after API design changes
cargo check --all-targets --all-features

# Check for unused dependencies
cargo machete

# Validate workspace dependencies
cargo tree --duplicates
```

**Dependency Validation**:
```bash
# Check missing dependencies before implementation
cargo metadata --format-version 1 | jq '.packages[].dependencies'

# Verify crate structure
cargo tree --workspace
```

### <stage>Pre-Execution Validation</stage>

**Cross-Component Compilation**:
```bash
# Ensure all test targets compile
cargo test --no-run --all

# Verify library builds
cargo build --lib --all

# Check all examples and benchmarks
cargo build --examples --benches
```

**Integration Verification**:
```bash
# Test cross-crate integration
cargo test --workspace --lib

# Verify anchor tests compile
cargo test --test "anchor_*" --no-run
```

### <stage>Implementation Validation</stage>

**Real-time Validation** (during development):
```bash
# Quick compilation check
cargo check

# Type-specific validation
cargo check --tests

# Clippy validation
cargo clippy --all-targets
```

**API Change Validation**:
```bash
# After modifying core types
cargo check --all-targets --all-features

# After adding dependencies
cargo build --all

# After changing public interfaces
cargo test --no-run --workspace
```

### <stage>Completion Validation</stage>

**Comprehensive Validation**:
```bash
# Full test suite
cargo test

# All linting issues resolved
cargo clippy --all-targets --all-features

# Documentation builds
cargo doc --all --no-deps

# Release profile validation
cargo build --release --all
```

## <automation>Automated Validation Scripts</automation>

### <script>API Compatibility Validation Script</script>

```bash
#!/bin/bash
# api-compatibility-check.sh

set -e

echo "🔍 API Compatibility Validation"
echo "================================"

# Step 1: Core type compilation
echo "📝 Checking core types..."
cargo check -p fortitude-types

# Step 2: Cross-crate compilation  
echo "🔗 Checking cross-crate dependencies..."
cargo check --all

# Step 3: Test compilation
echo "🧪 Checking test compilation..."
cargo test --no-run --all

# Step 4: Anchor test validation
echo "⚓ Validating anchor tests..."
cargo test --test "anchor_*" --no-run

echo "✅ API compatibility validation complete"
```

### <script>Development Readiness Check</script>

```bash
#!/bin/bash
# development-readiness-check.sh

set -e

echo "🚀 Development Readiness Check"
echo "=============================="

# Check git status
if [[ -n $(git status --porcelain) ]]; then
    echo "⚠️  Working directory not clean"
    git status --short
fi

# Validate branch
BRANCH=$(git branch --show-current)
if [[ "$BRANCH" == "main" || "$BRANCH" == "master" ]]; then
    echo "❌ Cannot work on main/master branch"
    exit 1
fi

# Compilation validation
echo "🔍 Running compilation checks..."
cargo check --all-targets --all-features

# Dependency validation
echo "📦 Checking dependencies..."
cargo tree --duplicates

# Test readiness
echo "🧪 Verifying test readiness..."
cargo test --no-run --workspace

echo "✅ Development environment ready"
```

### <script>Pre-Commit Validation</script>

```bash
#!/bin/bash
# pre-commit-validation.sh

set -e

echo "📋 Pre-Commit Validation"
echo "========================"

# Format check
echo "🎨 Checking code formatting..."
cargo fmt --all -- --check

# Linting
echo "🔍 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

# Test suite
echo "🧪 Running test suite..."
cargo test

# Documentation
echo "📚 Checking documentation..."
cargo doc --all --no-deps

# Anchor test validation
echo "⚓ Validating anchor tests..."
cargo test anchor --no-run

echo "✅ Pre-commit validation complete"
```

## <commands>Command Categories</commands>

### <category>Compilation Validation</category>

| Command | Purpose | When to Use |
|---------|---------|-------------|
| `cargo check` | Fast compilation check | During development |
| `cargo check --all-targets` | All targets compilation | API changes |
| `cargo check --all-features` | Feature compatibility | Before merge |
| `cargo build --all` | Full workspace build | Development completion |

### <category>Test Validation</category>

| Command | Purpose | When to Use |
|---------|---------|-------------|
| `cargo test --no-run` | Test compilation only | Quick validation |
| `cargo test --workspace` | All crates testing | Integration check |
| `cargo test anchor` | Anchor tests only | Regression protection |
| `cargo test --lib` | Unit tests only | Core logic validation |

### <category>Dependency Validation</category>

| Command | Purpose | When to Use |
|---------|---------|-------------|
| `cargo tree` | Dependency analysis | Structure verification |
| `cargo tree --duplicates` | Duplicate detection | Optimization |
| `cargo machete` | Unused dependencies | Cleanup |
| `cargo metadata` | Dependency metadata | Automation scripts |

### <category>Quality Validation</category>

| Command | Purpose | When to Use |
|---------|---------|-------------|
| `cargo clippy` | Linting | Code quality |
| `cargo fmt --check` | Format validation | Style consistency |
| `cargo doc` | Documentation build | Doc validation |
| `cargo audit` | Security audit | Security check |

## <integration>IDE Integration</integration>

### <vscode>VS Code Integration</vscode>

**tasks.json** configuration:
```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "API Compatibility Check",
            "type": "shell",
            "command": "cargo",
            "args": ["check", "--all-targets", "--all-features"],
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always"
            }
        },
        {
            "label": "Anchor Test Validation",
            "type": "shell", 
            "command": "cargo",
            "args": ["test", "--no-run", "--test", "anchor_*"],
            "group": "test"
        }
    ]
}
```

### <git-hooks>Git Hooks Integration</git-hooks>

**pre-commit hook**:
```bash
#!/bin/sh
# .git/hooks/pre-commit

# Run validation script
./scripts/pre-commit-validation.sh

# Check exit code
if [ $? -ne 0 ]; then
    echo "❌ Pre-commit validation failed"
    exit 1
fi

echo "✅ Pre-commit validation passed"
```

## <troubleshooting>Common Validation Issues</troubleshooting>

### <issue>Compilation Failures After API Changes</issue>
**Problem**: `cargo check --all-targets` fails after modifying core types
**Solution**:
```bash
# 1. Check specific error location
cargo check --message-format=json | jq -r '.message.rendered'

# 2. Fix field access patterns
cargo test --no-run --test "anchor_*"

# 3. Update affected components
cargo check -p [affected-crate]
```

### <issue>Test Compilation Failures</issue>
**Problem**: Tests fail to compile after dependency changes
**Solution**:
```bash
# 1. Check test-specific dependencies
cargo check --tests

# 2. Verify import paths
cargo test --no-run --workspace

# 3. Update test configurations
cargo metadata --format-version 1
```

### <issue>Dependency Resolution Conflicts</issue>
**Problem**: Multiple versions of same dependency
**Solution**:
```bash
# 1. Identify conflicts
cargo tree --duplicates

# 2. Update Cargo.toml constraints
# 3. Verify resolution
cargo check --locked
```

## <references>See Also</references>

- [API Compatibility Testing](../patterns/api-compatibility-testing.md)
- [Development Process](../../DEVELOPMENT_PROCESS.md)
- [Testing Strategy](../../tests/README.md)
- [Cargo Commands Reference](https://doc.rust-lang.org/cargo/commands/)

---

**Success Metrics**: Zero compilation failures during development, 100% anchor test compilation success, automated validation pipeline integration.