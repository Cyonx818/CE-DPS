# Domain Principles Guide - Quick Reference

<meta>
  <title>Domain Principles Guide - Quick Reference</title>
  <type>quick-reference</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-08</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Create domain-principles.md that constrains architectural decisions for Phase 1 planning
- **Key Approach**: Focus on hard constraints, system boundaries, and design principles (NOT implementation)
- **Core Benefits**: Provides architectural guardrails for component design and technology decisions
- **When to use**: During Phase 1 Architecture Planning, before designing system components
- **Related docs**: [Example: Fortitude Domain Principles](../domain-principles.md)

## <implementation>Document Structure</implementation>

### <pattern>Required Sections</pattern>
```markdown
## <context>Domain Definition</context>
- **Problem domain**: What specific domain does this system operate in?
- **Domain boundaries**: What's IN SCOPE vs OUT OF SCOPE?
- **Key characteristics**: What makes this domain unique or challenging?

## <constraints>Hard Constraints</constraints>
- **Performance constraints**: Non-negotiable performance requirements
- **Interface constraints**: Required interfaces and integration points
- **Domain-specific constraints**: Unique requirements from the problem domain
- **Quality constraints**: Minimum quality thresholds and validation requirements

## <principles>Architectural Design Principles</principles>
- **Core principles**: 3-5 fundamental principles that guide all decisions
- **Architectural implications**: How each principle constrains system design
- **Non-negotiable requirements**: What must be true in any valid architecture

## <architecture>Component Architecture Requirements</architecture>
- **Required components**: What components must exist based on domain constraints
- **Component relationships**: How components must interact
- **Interface requirements**: What interfaces each component must provide
- **Evolution constraints**: How the architecture must be able to evolve

## <boundaries>System Boundaries</boundaries>
- **What the system IS**: Clear definition of system purpose and scope
- **What the system IS NOT**: Explicit exclusions to prevent scope creep
- **Integration boundaries**: What systems it integrates with and dependencies

## <decisions>Architectural Decision Constraints</decisions>
- **Technology stack constraints**: Pre-determined technology choices
- **API design constraints**: Required API patterns and standards
- **Quality constraints**: Validation and accuracy requirements
- **Evolution constraints**: How the system must evolve across phases
```

### <pattern>Content Guidelines</pattern>

#### **Focus on Constraints, Not Implementation**
```markdown
<!-- GOOD: Architectural constraint -->
**Multi-Interface Requirement**: CLI, MCP Server, and JSON API must share identical core logic
**Architectural Implication**: Shared core engine with adapter pattern for interfaces

<!-- AVOID: Implementation detail -->
Here's how to implement the adapter pattern with specific Rust code...
```

#### **Define Hard Requirements**
```markdown
<!-- GOOD: Clear constraint -->
**Response Time**: Sub-60-second research-to-documentation pipeline (non-negotiable)
**Concurrency**: Support 100+ concurrent research requests

<!-- AVOID: Vague guideline -->
The system should be fast and handle multiple users
```

#### **Specify System Boundaries**
```markdown
<!-- GOOD: Clear boundaries -->
**What System IS**: Automated research pipeline for AI-assisted development
**What System IS NOT**: General-purpose search engine, version control system

<!-- AVOID: Unclear scope -->
System helps developers with various tasks and research needs
```

## <examples>Creation Process</examples>

### <template>Step 1: Domain Analysis</template>
1. **Read vision document** - Understand overall project goals and context
2. **Identify problem domain** - What specific domain does this system address?
3. **Define domain boundaries** - What's explicitly included vs excluded?
4. **Extract domain constraints** - What limitations does this domain impose?

### <template>Step 2: Constraint Identification</template>
1. **Performance requirements** - What are the non-negotiable performance needs?
2. **Interface requirements** - What interfaces must the system provide?
3. **Integration requirements** - What systems must it integrate with?
4. **Quality requirements** - What quality thresholds must be met?

### <template>Step 3: Principle Extraction</template>
1. **Core principles** - What 3-5 principles guide all architectural decisions?
2. **Architectural implications** - How does each principle constrain design?
3. **Component requirements** - What components must exist based on constraints?
4. **Evolution requirements** - How must the architecture evolve over time?

### <template>Step 4: Validation</template>
1. **Architectural constraint check** - Do principles actually constrain design choices?
2. **Boundary clarity check** - Are system boundaries clearly defined?
3. **Requirements completeness** - Are all hard requirements captured?
4. **Phase 1 utility check** - Can Phase 1 use this for architectural decisions?

## <examples>Example Content Patterns</examples>

### <template>Hard Constraint Pattern</template>
```markdown
### <constraint>Performance Constraints</constraint>
- **Response Time**: Sub-60-second research-to-documentation pipeline (non-negotiable)
- **Concurrency**: Support 100+ concurrent research requests
- **Cache Hit Rate**: >80% for repeated research topics
- **Quality Threshold**: >90% accuracy for type-specific completion criteria
```

### <template>Architectural Principle Pattern</template>
```markdown
### <principle>Multi-Interface Consistency</principle>
Core research logic must be interface-agnostic:
- CLI commands, MCP server, and JSON API must produce identical results
- Interface adapters handle presentation, core engine handles logic
- No interface-specific business logic

**Architectural Implication**: Shared core engine with adapter pattern for interfaces.
```

### <template>System Boundary Pattern</template>
```markdown
### <boundary>What System IS</boundary>
- **Automated research pipeline** for AI-assisted development
- **Context-aware knowledge generator** with type-specific processing
- **Self-improving knowledge base** that compounds institutional memory

### <boundary>What System IS NOT</boundary>
- **General-purpose search engine** - Focused on development domain knowledge
- **Real-time collaboration platform** - Research results, not team communication
- **Version control system** - Knowledge storage, not code versioning
```

## <troubleshooting>Validation Checklist</troubleshooting>

**Before Using Domain Principles in Phase 1**:
- [ ] **Domain clearly defined** - Problem domain and boundaries are explicit
- [ ] **Hard constraints identified** - Performance, interface, and quality requirements are non-negotiable
- [ ] **Architectural principles established** - 3-5 principles that constrain all design decisions
- [ ] **Required components identified** - Based on domain constraints, what components must exist
- [ ] **System boundaries defined** - Clear scope definition with explicit inclusions/exclusions
- [ ] **Technology constraints specified** - Pre-determined technology choices documented
- [ ] **Evolution path constrained** - How architecture must evolve across development phases
- [ ] **Phase 1 utility validated** - Document provides architectural guardrails for system design

## <references>See Also</references>
- [Fortitude Domain Principles](../domain-principles.md) - Complete example implementation
- [LLM Documentation Patterns](llm-documentation-patterns.md) - AI-optimized documentation standards
- [Development Process](../../DEVELOPMENT_PROCESS.md) - Three-phase development methodology
- [Reference Library README](../README.md) - Knowledge management and organization patterns