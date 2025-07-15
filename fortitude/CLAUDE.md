# CLAUDE.md

<meta>
  <title>Fortitude - Claude Code Integration</title>
  <type>ai_context</type>
  <audience>claude_code</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-08</updated>
</meta>

## <context>Fortitude Project Context</context>

Fortitude is an **automated AI knowledge pipeline** that generates research documentation optimized for AI consumption. This project focuses on building an intelligent research system that solves knowledge gaps in AI-assisted development.

> **REQUIRED CONTEXT:**
Include these AI guidelines files in your context window: `fortitude/AI_RULES.md`

## <commands>Development Commands</commands>

### <commands-category>Build and Test</commands-category>
```bash
# Build the project
cargo build

# Run tests
cargo test

# Run specific test
cargo test test_name

# Run unit tests only
cargo test --lib

# Run with output (for debugging tests)
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

### <commands-category>Development Tools</commands-category>
```bash
# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy

# Run the application
cargo run

# Run with arguments
cargo run -- --help
```

## <architecture>Architecture Overview</architecture>

**Project Type**: Development Process Solution (DPS) - Rust methodology template for AI-assisted development

### <architecture-section>Core Structure</architecture-section>

**Entry Points:**
- `fortitude/src/lib.rs` - Library with `Config` and `Application` structs
- `fortitude/src/main.rs` - CLI application with subcommand architecture

**Key Components:**
- **Configuration Management**: Environment-based config with validation (`serde`)
- **Async Application Layer**: Tokio-based async runtime with structured error handling
- **CLI Interface**: Subcommand architecture using `clap`
- **Comprehensive Testing**: Multi-layered testing (unit, integration, benchmarks)

### <architecture-section>Technology Stack</architecture-section>

**Core Dependencies:**
- `tokio` - Async runtime
- `serde` - Serialization with derive macros
- `clap` - CLI parsing with derive macros
- `thiserror` - Custom error types
- `tracing` - Structured logging

**Development Dependencies:**
- `proptest` - Property-based testing
- `criterion` - Performance benchmarking
- `mockall` - Mocking framework

### <patterns>Development Patterns</patterns>

**Configuration Pattern:**
```rust
// Environment-based configuration with validation
pub struct Config {
    pub api_endpoint: String,
    pub timeout: Duration,
    pub api_key: String,
}
```

**Error Handling:**
- Uses `thiserror` for custom error types
- Explicit `Result<T, E>` patterns throughout
- Descriptive error messages with context

**Async Processing:**
- Tokio-based async operations
- Structured logging with `tracing`

## <documentation>Documentation-Driven Development</documentation>

**Structure:**
- `fortitude/docs/architecture/` - System design and core architecture
- `fortitude/docs/planning/` - Implementation planning and roadmaps
- `fortitude/docs/research/` - Technical research and knowledge gaps
- `fortitude/docs/reference_library/` - Development references and patterns

## <process>AI Development Process</process>

**Three-Phase Workflow:** (see `DEVELOPMENT_PROCESS.md`)
1. **Plan Phase**: Architecture design and roadmap creation (30-60 min)
2. **Sprint Phase**: Implementation planning and research (15-30 min)
3. **Execute Phase**: TDD-based development (60-180 min)

**Branch Management:**
- Create feature branches: `git checkout -b fortitude-sprint-[XXX]_[description]`
- Never work directly on main/master
- Follow TDD approach: failing test → minimal code → refactor

## <testing>Testing Strategy</testing>

**Test Organization:** (see `fortitude/tests/README.md`)
- `fortitude/tests/integration_test.rs` - Integration tests
- `fortitude/tests/common/mod.rs` - Shared test utilities
- `fortitude/benches/benchmarks.rs` - Performance benchmarks
- Unit tests embedded in source files with `#[cfg(test)]`

**Test Patterns:**
- Use `tokio-test` for async testing
- `proptest` for property-based testing
- `tempfile` for temporary file management

## <quality>Quality Requirements</quality>

**Before Completing Tasks:**
- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code follows existing patterns
- [ ] **Anchor tests created** (use decision matrix in `tests/README.md`)
- [ ] **Anchor tests documented** with `ANCHOR:` docstring comments
- [ ] Documentation updated if architecture changed
- [ ] No regressions introduced

**Code Standards:**
- Match existing code style exactly
- Use descriptive variable/function names
- Handle errors explicitly
- Test edge cases and error conditions
- Add file purpose comments: `// ABOUTME: [purpose]`

## <references>Core References</references>

**AI Development Guidelines:**
- `fortitude/AI_RULES.md` - AI collaboration guidelines and core rules
- `fortitude/DEVELOPMENT_PROCESS.md` - Three-phase development methodology
- `fortitude/tests/README.md` - Testing strategy and requirements
- `fortitude/docs/reference_library/README.md` - Knowledge management system

**Project Focus**: Development methodology optimization for AI-assisted Rust development, emphasizing quality through comprehensive testing, explicit error handling, and documentation-driven development.