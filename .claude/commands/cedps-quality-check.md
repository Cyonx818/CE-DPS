Run comprehensive quality validation for CE-DPS project. This command executes the complete CI/CD test suite with auto-fix capability.

Execute the following quality validation steps:

1. Check Rust code formatting
2. Run clippy linting 
3. Build workspace
4. Execute test suite
5. Run security audit
6. Validate documentation
7. Check integration tests

Auto-fix capability will attempt to resolve formatting and linting issues automatically.

Check formatting and apply fixes if needed:

```bash
cargo fmt --all -- --check
```

If formatting check fails, auto-fix:

```bash
cargo fmt --all
```

Run clippy linting with warnings as errors:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

If clippy fails, attempt auto-fix:

```bash
cargo clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged
```

Build the workspace:

```bash
cargo build --workspace --verbose
```

Run all tests:

```bash
cargo test --workspace --verbose
```

Run security audit:

```bash
cargo audit
```

Build documentation:

```bash
cargo doc --workspace --no-deps
```

Check for quality gates tool and run if available:

```bash
cargo build --bin quality-gates
./target/debug/quality-gates --project-path .
```

Run Python tests if available:

```bash
python3 -m pytest tools/phase-validator.py --doctest-modules -v
```

Test Fortitude integration if available:

```bash
cargo run --bin fortitude-integration -- check
```

This validation ensures all CE-DPS technical standards are met before progression to next development phase.