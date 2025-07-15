# Fortitude Domain Principles

<meta>
  <title>Fortitude Domain Principles</title>
  <type>domain-guide</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-08</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Define architectural constraints and design principles for Fortitude system design
- **Domain**: AI knowledge pipeline with context-aware research classification
- **Core Constraints**: Sub-60s research pipeline, 5 research types, multi-interface support
- **Key Principles**: Progressive disclosure, audience adaptation, self-improving knowledge base
- **Related docs**: [Vision](../ideas/project_brainstorming.md), [Development Process](../DEVELOPMENT_PROCESS.md), [Prototype Lessons](../ideas/prototype_lessons_learned.md)

## <context>Domain Definition</context>

Fortitude operates in the **AI-assisted development tooling domain**, specifically as an **automated knowledge pipeline** that bridges the gap between AI coding assistants and domain-specific technical knowledge.

**Domain Boundaries:**
- **IN SCOPE**: Research request processing, knowledge generation, AI-optimized documentation, multi-interface access
- **OUT OF SCOPE**: General-purpose search, human-readable documentation, real-time collaboration, version control integration

## <constraints>Hard Constraints</constraints>

### <constraint>Performance Constraints</constraint>
- **Response Time**: Sub-60-second research-to-documentation pipeline (non-negotiable)
- **Concurrency**: Support 100+ concurrent research requests
- **Cache Hit Rate**: >80% for repeated research topics
- **Quality Threshold**: >90% accuracy for type-specific completion criteria

### <constraint>Interface Constraints</constraint>
- **Multi-Interface Requirement**: CLI, MCP Server, and JSON API must share identical core logic
- **AI-First Design**: All outputs optimized for AI consumption over human readability
- **Backward Compatibility**: API stability across development phases required
- **Integration Constraint**: Must integrate with existing Claude Code workflows without disruption

### <constraint>Research Classification Constraints</constraint>
- **Five Research Types**: Decision, Implementation, Troubleshooting, Learning, Validation (fixed taxonomy)
- **Context Dimensions**: ResearchType × AudienceContext × DomainContext × UrgencyLevel
- **Progressive Disclosure**: All outputs must follow Answer → Evidence → Implementation pattern
- **Length Constraints**: Type-specific word limits to maintain cognitive load matching

### <constraint>Knowledge Management Constraints</constraint>
- **Self-Improving Requirement**: Each research result must contribute to future capability
- **Institutional Memory**: Knowledge must persist and compound across development cycles
- **Semantic Markup**: All documentation must use XML-style tags for LLM parsing optimization
- **Token Efficiency**: 6-8x compression ratios while maintaining actionable content

## <principles>Architectural Design Principles</principles>

### <principle>Research Type Taxonomy (Non-Negotiable)</principle>
The system must support exactly five research types with fixed boundaries:

- **Decision Research**: Technology choice decisions → Decision matrices, recommendations
- **Implementation Research**: Active development work → Working code examples, patterns
- **Troubleshooting Research**: Problem-solving → Diagnostic steps, solutions
- **Learning Research**: Knowledge acquisition → Concept explanations, mental models
- **Validation Research**: Approach verification → Best practice analysis, trade-offs

**Architectural Implication**: Core engine must route requests through type-specific processors.

### <principle>Context-Aware Processing</principle>
All research requests must be classified across multiple dimensions:
- **Research Type** (fixed taxonomy above)
- **Audience Context** (Technical, Executive, Architectural)
- **Domain Context** (Technical systems, business strategy)
- **Urgency Level** (Immediate, Standard, Background)

**Architectural Implication**: Classification engine required before research processing.

### <principle>Progressive Disclosure Pattern</principle>
All outputs must follow a three-layer information hierarchy:
1. **Immediate Answer** (high priority) - Direct response to query
2. **Supporting Evidence** (medium priority) - Justification and context
3. **Implementation Details** (low priority) - Full examples and specifics

**Architectural Implication**: Output formatting must support layered content delivery.

### <principle>Multi-Interface Consistency</principle>
Core research logic must be interface-agnostic:
- CLI commands, MCP server, and JSON API must produce identical results
- Interface adapters handle presentation, core engine handles logic
- No interface-specific business logic

**Architectural Implication**: Shared core engine with adapter pattern for interfaces.

### <principle>Self-Improving Knowledge Base</principle>
System must capture and reuse research results:
- Each research result becomes part of institutional memory
- Knowledge compounds across development cycles
- Proactive mode identifies and fills knowledge gaps

**Architectural Implication**: Storage layer must support semantic search and knowledge graph relationships.

## <architecture>Component Architecture Requirements</architecture>

### <requirement>Core System Components</requirement>

Based on domain constraints, the system must include:

#### **Request Classification Engine**
- **Purpose**: Multi-dimensional request analysis (type, audience, domain, urgency)
- **Constraint**: Must classify requests within 100ms
- **Interface**: Accepts raw research requests, outputs structured classification
- **Dependency**: Required by all research processors

#### **Research Processing Pipeline**
- **Purpose**: Type-specific research execution with LLM integration
- **Constraint**: Must complete research within 60-second total pipeline
- **Interface**: Accepts classified requests, outputs structured research results
- **Dependency**: Depends on classification engine and LLM client

#### **Knowledge Storage System**
- **Purpose**: Persistent storage with semantic search capabilities
- **Constraint**: Must support >80% cache hit rate for repeated topics
- **Interface**: Key-value storage with semantic search API
- **Evolution**: File-based → Vector database → Hybrid approach

#### **Multi-Interface Adapter Layer**
- **Purpose**: Present identical functionality through CLI, MCP, and JSON API
- **Constraint**: Interface-agnostic core logic, no business logic in adapters
- **Interface**: Standardized internal API, interface-specific presentations
- **Dependency**: All interfaces depend on core research pipeline

### <requirement>Quality Assurance Component</requirement>
- **Purpose**: Validate research output against type-specific completion criteria
- **Constraint**: >90% accuracy threshold for all research types
- **Interface**: Accepts research results, outputs quality scores and validation
- **Integration**: Real-time validation during research generation

### <requirement>Caching and Performance Layer</requirement>
- **Purpose**: Context-aware caching with multi-dimensional cache keys
- **Constraint**: Support 100+ concurrent requests with sub-second cache response
- **Interface**: Transparent caching layer between interfaces and core processing
- **Evolution**: In-memory → Redis → Hybrid multi-tier caching

## <boundaries>System Boundaries</boundaries>

### <boundary>What Fortitude IS</boundary>
- **Automated research pipeline** for AI-assisted development
- **Context-aware knowledge generator** with type-specific processing
- **Self-improving knowledge base** that compounds institutional memory
- **Multi-interface research service** for AI agents and human developers

### <boundary>What Fortitude IS NOT</boundary>
- **General-purpose search engine** - Focused on development domain knowledge
- **Real-time collaboration platform** - Research results, not team communication
- **Version control system** - Knowledge storage, not code versioning
- **Human-readable documentation generator** - AI-optimized output format

### <boundary>Integration Boundaries</boundary>
- **Integrates WITH**: Claude Code, development workflows, existing documentation
- **Does NOT integrate WITH**: IDEs directly, code execution, deployment pipelines
- **Provides TO**: Structured research results, knowledge references, decision support
- **Depends ON**: LLM providers, file systems, optional vector databases

## <decisions>Architectural Decision Constraints</decisions>

### <decision>Technology Stack Constraints</decision>
- **Language**: Rust (non-negotiable for performance and ecosystem compatibility)
- **Async Runtime**: Tokio (required for concurrent request processing)
- **CLI Framework**: Clap (established pattern in Concordia workspace)
- **LLM Integration**: Claude API (primary), with multi-provider support
- **Storage Evolution**: File-based → Qdrant → Hybrid (progressive enhancement)

### <decision>API Design Constraints</decision>
- **Request Format**: Structured types with clear research classification
- **Response Format**: Progressive disclosure with semantic markup
- **Error Handling**: Graceful degradation with fallback mechanisms
- **Versioning**: Semantic versioning with backward compatibility guarantees

### <decision>Quality Constraints</decision>
- **Validation**: Multi-stage verification including LLM cross-checking
- **Completeness**: Type-specific completion criteria must be enforced
- **Accuracy**: >90% correctness validated through automated testing
- **Consistency**: Standardized output formats for predictable consumption

## <evolution>Architectural Evolution Constraints</evolution>

### <constraint>MVP Architecture</constraint>
- **Components**: CLI + Basic Classification + File Storage + Claude API
- **Constraints**: Single-threaded processing acceptable, basic caching
- **Success Criteria**: 60-second research pipeline operational
- **Non-negotiable**: Research type taxonomy must be implemented

### <constraint>Production Architecture</constraint>
- **Components**: Multi-interface + Advanced Classification + Hybrid Storage
- **Constraints**: Concurrent processing required, sophisticated caching
- **Success Criteria**: 100+ concurrent requests, <60s response time
- **Non-negotiable**: API stability and backward compatibility

### <constraint>Advanced Architecture</constraint>
- **Components**: Vector Search + Semantic Knowledge + Proactive Mode
- **Constraints**: Semantic search, knowledge graph relationships
- **Success Criteria**: Self-improving knowledge ecosystem
- **Non-negotiable**: Knowledge persistence and institutional memory

## <references>See Also</references>
- [Project Vision](../ideas/project_brainstorming.md) - Complete vision and strategic context
- [Development Process](../DEVELOPMENT_PROCESS.md) - Three-phase development methodology
- [Prototype Lessons](../ideas/prototype_lessons_learned.md) - Validated insights from prototyping
- [AI Rules](../AI_RULES.md) - Core development guidelines and quality standards
- [LLM Documentation Patterns](quick-reference/llm-documentation-patterns.md) - AI-optimized documentation standards
- [Reference Library README](README.md) - Knowledge management and organization patterns

---

**Usage**: These domain principles constrain architectural decisions by defining hard requirements (5 research types, sub-60s pipeline, multi-interface support), system boundaries (what Fortitude is/isn't), required components (classification engine, processing pipeline, storage system), and evolution constraints (MVP → Production → Advanced architecture).