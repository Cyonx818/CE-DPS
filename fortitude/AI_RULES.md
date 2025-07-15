# AI Development Rules

<meta>
  <title>AI Development Rules</title>
  <type>ai_guidelines</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-08</updated>
</meta>

## <context>Primary Directive</context>
**AI Goal**: Achieve maximum development effectiveness through structured human-AI collaboration while maintaining code quality and systematic approach.

## <context>Project Context - Fortitude</context>
**Tech Stack**: Rust, Tokio, Serde, Clap, Criterion, Proptest
**Entry Point**: `fortitude/src/main.rs` (CLI), `fortitude/src/lib.rs` (Library)
**Key Dependencies**: tokio, serde, clap, thiserror, tracing
**Architecture**: Async application layer with CLI interface and comprehensive testing

## <rules>Core Rules (Never Break)</rules>

### <rule priority="critical">Test-Driven Development</rule>
- **NEVER make changes without tests** - Write failing test first, then implementation
- **NEVER break existing functionality** - Run all tests before considering done
- **NEVER skip error messages** - Every warning and error must be addressed

### <rule priority="critical">Quality Standards</rule>
- **NEVER modify code unrelated to current task** - Document observations for later
- **NEVER use placeholder code without TODO comments** - Mark all incomplete work
- **ALWAYS match existing code style** - Follow project patterns exactly

### <rule priority="critical">Documentation Updates</rule>
- **ALWAYS update architecture docs** when making structural changes
- **ALWAYS reference `fortitude/DEVELOPMENT_PROCESS.md`** for systematic approach
- **ALWAYS use `fortitude/docs/reference_library/README.md`** for knowledge management
- **ALWAYS follow LLM-optimized documentation patterns** from `fortitude/docs/reference_library/research/llm-optimized-documentation.md`

### <rule priority="critical">Reference Library Consultation</rule>

<decision-matrix>
  <conditions>
    <condition name="task_complexity" type="enum" values="[low, medium, high, critical]" />
    <condition name="task_type" type="enum" values="[research, implementation, troubleshooting]" />
    <condition name="knowledge_gap" type="boolean" />
  </conditions>
  
  <rules>
    <rule priority="mandatory">
      <if>task_complexity IN [high, critical]</if>
      <then>SEARCH reference library BEFORE any other action</then>
    </rule>
    <rule priority="recommended">
      <if>task_complexity = medium OR knowledge_gap = true</if>
      <then>SEARCH reference library for existing patterns</then>
    </rule>
    <rule priority="required">
      <if>task_type IN [research, implementation]</if>
      <then>CHECK for existing implementation guides BEFORE creating new</then>
    </rule>
  </rules>
</decision-matrix>

<search-strategy>
  <command>SEARCH using these keywords in sequence:</command>
  <keywords>
    <primary>domain terms, technology names, pattern types</primary>
    <secondary>error messages, configuration examples</secondary>
    <tertiary>related frameworks, similar implementations</tertiary>
  </keywords>
  <integration>
    <if>existing_knowledge_found = true</if>
    <then>REFERENCE found knowledge AND build upon it in current work</then>
  </integration>
</search-strategy>

**Fortitude-specific requirements**:
- **BEFORE starting any research** - Search existing reference library using relevant keywords
- **BEFORE implementing patterns** - Check for existing implementation guides and examples  
- **WHEN encountering unfamiliar concepts** - Search reference library before asking for external research
- **Integration requirement**: When existing knowledge found, reference it in current work and build upon it
- **Complexity-based requirements**:
  - **Recommended**: Medium complexity tasks or unrated tasks
  - **Mandatory**: High/Critical complexity tasks

## <process>Development Workflow (See DEVELOPMENT_PROCESS.md)</process>

### <quickref>Three-Phase Process</quickref>
1. **Plan**: Architecture + roadmap (see DEVELOPMENT_PROCESS.md Phase 1)
2. **Sprint**: Implementation planning + research (see DEVELOPMENT_PROCESS.md Phase 2)  
3. **Execute**: TDD implementation (see DEVELOPMENT_PROCESS.md Phase 3)

### <workflow>Standard Development Sequence</workflow>
1. Create feature branch: `git checkout -b fortitude-sprint-[XXX]_[description]`
2. Write failing test for new functionality
3. Write minimal code to pass test
4. Run all tests to ensure no regressions
5. Update documentation if architecture changed
6. Commit with clear message

## <standards>Code Standards</standards>

### <standard priority="critical">Rust Patterns</standard>
- **Error Handling**: Use `thiserror` for custom error types, explicit `Result<T, E>` patterns
- **Async**: Tokio-based async operations with structured logging via `tracing`
- **File Comments**: Add `// ABOUTME: [purpose]` comment at top of new files
- **Testing**: Use `#[cfg(test)]` modules, `proptest` for property-based testing

### <standard priority="critical">Documentation Standards</standard>
- **Use semantic markup**: XML-style tags for LLM parsing optimization (`<context>`, `<implementation>`, `<pattern>`)
- **Progressive disclosure**: Structure all docs as Answer → Evidence → Implementation layers
- **Token efficiency**: Target 6-8x compression ratios while maintaining actionable content
- **Metadata blocks**: Include `<meta>` blocks with title, type, audience, complexity, updated date
- **Reference patterns**: Follow `fortitude/docs/reference_library/research/llm-optimized-documentation.md`

### <standard priority="critical">Quality Gates</standard>

<completion-checklist>
  <command>EXECUTE these verification steps before marking any task complete:</command>
  <verification_steps>
    <step priority="mandatory" action="run">
      <command>EXECUTE `cargo test` AND verify all tests pass</command>
      <failure_action>FIX failing tests before proceeding</failure_action>
    </step>
    <step priority="mandatory" action="run">
      <command>EXECUTE `cargo clippy` AND resolve all warnings</command>
      <failure_action>FIX all linting issues before proceeding</failure_action>
    </step>
    <step priority="mandatory" action="validate">
      <command>EXECUTE `cargo check --all-targets --all-features` AND verify cross-component compatibility</command>
      <failure_action>FIX compilation issues across all components before proceeding</failure_action>
    </step>
    <step priority="mandatory" action="verify">
      <command>CONFIRM code follows existing patterns and conventions</command>
      <validation>Match indentation, naming, error handling patterns</validation>
    </step>
    <step priority="mandatory" action="create">
      <command>CREATE anchor tests using decision matrix in `fortitude/tests/README.md`</command>
      <criteria>Apply boolean logic framework for anchor test requirements</criteria>
    </step>
    <step priority="conditional" action="create">
      <command>CREATE API compatibility anchor tests IF api_compatibility OR cross_component_integration OR type_definition_changes</command>
      <criteria>Apply enhanced decision matrix including API compatibility criteria</criteria>
      <reference>See `docs/reference_library/patterns/api-compatibility-testing.md` for patterns</reference>
    </step>
    <step priority="mandatory" action="document">
      <command>ADD `ANCHOR:` docstring comments to all anchor tests</command>
      <format>Document purpose and regression protection rationale</format>
    </step>
    <step priority="mandatory" action="verify">
      <command>CONFIRM documentation follows LLM-optimized patterns</command>
      <validation>Semantic markup, progressive disclosure, token efficiency</validation>
    </step>
    <step priority="conditional" action="update">
      <command>UPDATE documentation IF architectural changes made</command>
      <scope>Architecture docs, API docs, integration guides</scope>
    </step>
    <step priority="mandatory" action="validate">
      <command>VERIFY no regressions introduced</command>
      <method>Run full test suite and compare against baseline</method>
    </step>
  </verification_steps>
</completion-checklist>

## <commands>Quick Reference</commands>

```bash
# Build & Test
cargo build                   # Build project
cargo test                    # Run all tests
cargo test --lib             # Run unit tests only
cargo clippy                  # Lint code

# Run
cargo run                     # Run application
cargo run -- --help          # Show CLI help
```

## <escalation>When Stuck</escalation>
1. State clearly: "I don't understand X"
2. Check error messages carefully
3. Look for working examples in codebase
4. Refer to `fortitude/DEVELOPMENT_PROCESS.md` for systematic approach
5. Ask user for clarification
6. Never pretend to know

---
**Core References**: `fortitude/DEVELOPMENT_PROCESS.md` (methodology), `fortitude/tests/README.md` (testing), `fortitude/docs/reference_library/README.md` (knowledge management), `fortitude/docs/reference_library/research/llm-optimized-documentation.md` (documentation patterns) 