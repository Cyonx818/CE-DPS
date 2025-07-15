# Reference Library for Agentic AI Development

<meta>
  <title>AI Reference Library Management</title>
  <type>ai_guidelines</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-09</updated>
</meta>

## <rules priority="critical">AI Agent Rules</rules>

1. **ALWAYS search reference library BEFORE asking user** - Knowledge likely exists
2. **NEVER load entire documents** - Use targeted searches and specific sections
3. **ALWAYS check INDEX.md first** - Fastest path to relevant information
4. **NEVER assume outdated information** - Check timestamps and version compatibility
5. **ALWAYS update library when discovering new patterns** - Capture knowledge for future use
6. **USE parallel subagent coordination** - See [Parallel Subtask Rules](../../methodology/parallel_subtask_rules.md) for context inheritance and coordination patterns

## <purpose>Purpose & Scope</purpose>

**Context-efficient knowledge base** for AI coding assistants working on Fortitude project.

**Provides:**
- **Domain-specific patterns** with implementation examples
- **Technical specifications** and API documentation  
- **Configuration templates** and environment setups
- **Research findings** and proven solutions
- **Error resolution guides** and troubleshooting steps

## <structure>Folder Structure & Naming</structure>

### <structure-example>Hierarchical Organization</structure-example>
```
reference_library/
├── INDEX.md                     # Master index - READ FIRST
├── domain-principles.md         # Fortitude domain principles
├── quick-reference/             # 1-2 page summaries
│   ├── rust-patterns.md
│   ├── cargo-commands.md
│   └── testing-patterns.md
├── patterns/                    # Implementation patterns
│   ├── INDEX.md
│   ├── async-patterns.md
│   ├── error-handling.md
│   └── testing-patterns.md
├── configurations/              # Config templates and examples
│   ├── INDEX.md
│   ├── development/
│   └── testing/
├── research/                    # Technical research and findings
│   ├── INDEX.md
│   ├── rust-performance.md
│   └── ai-methodology.md
└── troubleshooting/             # Error solutions and debugging
    ├── INDEX.md
    ├── cargo-errors.md
    └── testing-issues.md
```

### <naming>Naming Conventions</naming>
- **Descriptive names**: `async-error-handling.md` not `async.md`
- **Consistent format**: `{domain}-{function}-{type}.md`
- **Version indicators**: `rust-2021-patterns.md` when versioning matters
- **Scope indicators**: `quick-{topic}.md` for summaries

## <formatting>Document Formatting Standards</formatting>

### <template>LLM-Optimized Document Structure</template>
```markdown
# {Clear Descriptive Title}

<meta>
  <title>Document Title</title>
  <type>pattern|reference|guide|research</type>
  <audience>ai_assistant</audience>
  <complexity>basic|intermediate|advanced</complexity>
  <updated>YYYY-MM-DD</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: What this solves
- **Key Approach**: Core methodology or technique
- **Core Benefits**: Quantifiable improvements (metrics, percentages)
- **When to use**: Specific scenarios and use cases
- **Related docs**: Links to connected information

## <implementation>Implementation</implementation>
[Detailed information with progressive disclosure]

### <pattern>Core Patterns</pattern>
[Reusable implementation patterns with Rust examples]

### <concept>Key Concepts</concept>
[Fundamental concepts with mental models]

## <examples>Examples</examples>
[Practical Rust code examples with semantic compression]

### <template>Working Templates</template>
[Ready-to-use code templates and patterns]

## <troubleshooting>Troubleshooting</troubleshooting>
[Common issues and solutions with error handling]

## <references>See Also</references>
[Related reference materials with cross-references]
```

### <optimization>LLM-Optimized Content Guidelines</optimization>
1. **Progressive Disclosure**: Layer information from high-level → detailed using priority attributes
2. **Semantic Compression**: Balance token efficiency with comprehensive Rust examples
3. **Hierarchical Structure**: Use XML-style semantic tags for LLM parsing optimization
4. **Token Efficiency**: Aim for 6-8x compression while maintaining actionable content
5. **Cross-Reference Network**: Link related concepts for comprehensive understanding

### <organization>Content Organization Rules</organization>
1. **Front-load critical information** - Most important details first
2. **Use semantic markup** - Enables AI parsing
3. **Include code examples** - Practical Rust implementation guidance
4. **Add cross-references** - Link related concepts
5. **Maintain currency** - Update dates and version compatibility

## <indexing>INDEX.md Strategy</indexing>

### <index-template>Master Index Structure</index-template>
```markdown
# Fortitude Reference Library Index

**Quick Navigation**
- [Quick Reference](#quick-reference) - Single-page summaries
- [Common Patterns](#patterns) - Implementation examples
- [Troubleshooting](#troubleshooting) - Error solutions

## Quick Reference
- [Rust Patterns](quick-reference/rust-patterns.md) - Common Rust patterns
- [Cargo Commands](quick-reference/cargo-commands.md) - Build and test commands
- [Testing Patterns](quick-reference/testing-patterns.md) - Test organization

## Patterns
- [Async Patterns](patterns/async-patterns.md) - Tokio and async/await
- [Error Handling](patterns/error-handling.md) - thiserror and Result patterns
- [Testing Patterns](patterns/testing-patterns.md) - Unit and integration tests

## By Use Case
### Async Development
- [Async Error Handling](patterns/async-patterns.md#error-handling)
- [Tokio Runtime](patterns/async-patterns.md#runtime)
- [Async Testing](patterns/testing-patterns.md#async-testing)

### CLI Applications
- [Clap Patterns](patterns/cli-patterns.md) - Command-line interface
- [Configuration](patterns/config-patterns.md) - Environment-based config
- [Logging](patterns/logging-patterns.md) - Structured logging with tracing

## Recent Updates
- 2025-07-08: Added Fortitude-specific patterns
- 2025-07-08: Updated testing strategies
```

### <index-template>Section-Specific Indexes</index-template>
```markdown
# Patterns Index

## Async Patterns
- [Tokio Runtime](async-patterns.md#runtime) - Runtime configuration
- [Error Handling](async-patterns.md#error-handling) - Async error patterns
- [Testing](async-patterns.md#testing) - Async test patterns

## Testing Patterns
- [Unit Testing](testing-patterns.md#unit) - Individual function testing
- [Integration Testing](testing-patterns.md#integration) - Component interaction
- [Property Testing](testing-patterns.md#property) - Property-based testing

## Error Patterns
- [thiserror Usage](error-handling.md#thiserror) - Custom error types
- [Result Patterns](error-handling.md#result) - Result handling
- [Error Propagation](error-handling.md#propagation) - Error bubbling
```

## <search>Search Strategies for AI Agents</search>

### <search-process>Systematic Knowledge Consultation</search-process>

#### <search-step>Before Any New Research</search-step>
1. **Identify search terms**: Extract key concepts, technologies, patterns from current task
2. **Search existing INDEX.md**: Look for relevant sections and documents
3. **Search specific directories**: Check patterns/, quick-reference/, troubleshooting/ as appropriate
4. **Evaluate findings**: Determine if existing knowledge addresses current need
5. **Document gaps**: Only request new research for confirmed knowledge gaps

#### <search-step>Search Query Patterns</search-step>
- **Technology research**: "[tech-name] patterns", "[tech-name] best practices"
- **Implementation help**: "[pattern-type] implementation", "[use-case] examples" 
- **Architecture decisions**: "[domain] architecture", "system design [context]"
- **Troubleshooting**: "[tech-name] errors", "[problem-type] solutions"

### <strategy>Hierarchical Search Pattern</strategy>
1. Check INDEX.md for topic overview
2. Navigate to specific section index
3. Read relevant document sections
4. Cross-reference related materials

<parallel-search>
**Parallel Search Coordination**: For complex research requiring multiple search domains, see [Parallel Subtask Rules](../../methodology/parallel_subtask_rules.md) for:
- Multi-directory parallel search patterns
- Domain separation strategies for research subagents
- Context inheritance requirements for research coordination
</parallel-search>

### <strategy>Context-Efficient Reading</strategy>
- **Use targeted searches**: "tokio async error handling"
- **Target specific directories**: `["reference_library/patterns/"]`
- **Read targeted sections** rather than entire documents
- **Follow cross-references** only when directly relevant

### <strategy>Quick Reference First</strategy>
- Check `quick-reference/` for immediate answers
- Use detailed docs only when quick reference insufficient
- Leverage `<summary priority="high">` sections for rapid context

## <maintenance>Maintenance Practices</maintenance>

### <maintenance-rule>For AI Agents During Development</maintenance-rule>
1. **Document new patterns** when implementing novel solutions
2. **Update examples** when discovering better approaches
3. **Add troubleshooting entries** when resolving complex issues
4. **Create quick references** for frequently needed information
5. **Update timestamps** when modifying documents

### <maintenance-rule>Content Quality Standards</maintenance-rule>
- **Actionable information** - Focus on implementation guidance
- **Working Rust examples** - Include tested code samples
- **Current information** - Verify compatibility and accuracy
- **Clear context** - Explain when and why to use each approach

## <integration>Integration with Development Process</integration>

### <integration-phase>During Planning Phase</integration-phase>
- **Architecture decisions**: Reference domain patterns
- **Technology choices**: Consult research findings

### <integration-phase>During Implementation</integration-phase>
- **Pattern lookup**: Find implementation examples
- **Error resolution**: Consult troubleshooting guides
- **Configuration**: Use template configurations

### <integration-phase>During Testing</integration-phase>
- **Test patterns**: Reference testing approaches
- **Error scenarios**: Check error handling patterns

## <templates>Quick Reference Templates</templates>

### <template>Pattern Document Template</template>
```markdown
# {Pattern Name}

<meta>
  <title>Pattern Name</title>
  <type>pattern</type>
  <audience>ai_assistant</audience>
  <complexity>basic|intermediate|advanced</complexity>
  <updated>YYYY-MM-DD</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: [What this solves]
- **Implementation**: [Key steps]
- **Benefits**: [Why use this approach]

## <implementation>Code Example</implementation>
```rust
// Working Rust example with comments
fn implement_pattern() -> Result<(), Error> {
    // Step 1: Setup
    // Step 2: Implementation
    // Step 3: Error handling
    Ok(())
}
```

## <troubleshooting>Troubleshooting</troubleshooting>
- **Issue**: [Common problem]
  - **Solution**: [How to fix]

## <references>See Also</references>
- [Related patterns]
- [Dependencies]
```

**Goal**: Enable AI agents to find precise information quickly without bloating context windows. Every document should be immediately actionable and efficiently navigable. 