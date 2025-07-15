# Fortitude Reference Library Index

<meta>
  <title>Fortitude Reference Library Index</title>
  <type>index</type>
  <audience>ai_assistant</audience>
  <complexity>basic</complexity>
  <updated>2025-07-08</updated>
</meta>

## <summary priority="high">Quick Navigation</summary>
- **[Quick Reference](#quick-reference)** - Essential patterns and concepts (1-2 pages)
- **[Research](#research)** - Comprehensive guides and implementation details
- **[Patterns](#patterns)** - Reusable implementation patterns
- **[Troubleshooting](#troubleshooting)** - Error solutions and debugging guides

## <section>Quick Reference</section>
Fast access to essential patterns and concepts.

### <category>Documentation and AI Optimization</category>
- **[LLM Documentation Patterns](quick-reference/llm-documentation-patterns.md)** - Semantic markup patterns for AI consumption
- **[Prompt Engineering Quick Reference](quick-reference/prompt-engineering-quick-ref.md)** - Essential prompt patterns and frameworks
- **[Domain Principles Guide](quick-reference/domain-principles-guide.md)** - Creating domain-principles.md for architecture planning

### <category>Development</category>
- **[Rust Patterns](quick-reference/rust-patterns.md)** - Common Rust patterns and idioms
- **[Cargo Commands](quick-reference/cargo-commands.md)** - Build and test commands
- **[Testing Patterns](quick-reference/testing-patterns.md)** - Test organization and strategies

## <section>Research</section>
Comprehensive guides with detailed implementation examples.

### <category>Domain and Architecture</category>
- **[Domain Principles](domain-principles.md)** - Fortitude architectural constraints and design principles

### <category>AI and Documentation</category>
- **[LLM-Optimized Documentation](research/llm-optimized-documentation.md)** - Complete implementation guide for AI-friendly documentation
- **[Prompt Engineering Concepts](research/prompt-engineering-concepts.md)** - Systematic approach to prompt engineering patterns

### <category>Technology Decisions</category>
- **[Vector Database Technology Decision](research/vector-database-technology-decision.md)** - Qdrant vs alternatives for Rust applications
- **[Embedding Model Technology Decision](research/embedding-model-technology-decision.md)** - OpenAI vs local models for technical content
- **[CLI Framework Technology Decision](research/cli-framework-technology-decision.md)** - Clap vs alternatives for Rust CLI applications

### <category>Implementation Guides</category>
- **[Claude API Implementation Guide](research/claude-api-implementation-guide.md)** - Production-ready Claude API client with rate limiting and retry logic
- **[Research Template System Implementation](research/research-template-system-implementation.md)** - Type-safe template system for research types with progressive disclosure
- **[Research Quality Validation Implementation](research/research-quality-validation-implementation.md)** - Comprehensive quality validation framework for AI-generated research
- **[MCP Server Implementation Guide](research/mcp-server-implementation-guide.md)** - Production-ready Model Context Protocol servers in Rust
- **[Multi-LLM Provider System](research/multi-llm-provider-system.md)** - Multi-provider LLM system with intelligent routing and fallback
- **[Hybrid Storage System Implementation](research/hybrid-storage-system-implementation.md)** - Scalable hybrid storage with deduplication and caching
- **[Production-Ready Rust API System](research/production-ready-rust-api-system.md)** - Comprehensive Rust API implementation with Axum
- **[Rust Production APIs Quick Reference](research/rust-production-apis-quick-reference.md)** - Essential patterns and dependencies for production Rust APIs
- **[Quality Evaluation System Implementation](research/quality-evaluation-system-implementation.md)** - Automated quality assessment for research content
- **[Observability System Implementation](research/observability-system-implementation.md)** - Real-time monitoring and alerting system

### <category>Advanced Classification</category>
- **[Dynamic Threshold Adjustment](research/dynamic-threshold-adjustment.md)** - Real-time threshold optimization for classification systems
- **[Rule Composition Algorithms](research/rule-composition-algorithms.md)** - Multi-dimensional signal combination with conflict resolution
- **[Semantic Analysis Integration](research/semantic-analysis-integration.md)** - Context-aware classification with semantic understanding

### <category>System Reference Documentation</category>
- **[Hybrid Storage System](systems/hybrid-storage.md)** - Production-ready hybrid storage implementation reference

### <category>Development Process</category>
- **[AI Development Methodology](research/ai-development-methodology.md)** - Three-phase development process
- **[Testing Strategies](research/testing-strategies.md)** - Multi-layered testing approach

## <section>Patterns</section>
Reusable implementation patterns organized by domain.

### <category>Rust Development</category>
- **[Rust Workspace Patterns](patterns/rust-workspace-patterns.md)** - Multi-crate workspace configuration with shared dependencies
- **[Classification Algorithms](patterns/classification-algorithms.md)** - Text classification with keyword matching and confidence scoring
- **[CLI Testing Patterns](patterns/cli-testing-patterns.md)** - Comprehensive CLI testing strategies with async support

### <category>Async Development</category>
- **[Async Error Handling](patterns/async-patterns.md#error-handling)** - thiserror with async/await
- **[Tokio Runtime](patterns/async-patterns.md#runtime)** - Runtime configuration patterns
- **[Async Testing](patterns/async-patterns.md#testing)** - Testing async code

### <category>CLI Applications</category>
- **[Clap Patterns](patterns/cli-patterns.md)** - Command-line interface patterns
- **[Configuration](patterns/config-patterns.md)** - Environment-based configuration
- **[Logging](patterns/logging-patterns.md)** - Structured logging with tracing

### <category>AI Integration</category>
- **[LLM Integration](patterns/llm-integration.md)** - Patterns for AI assistant integration
- **[Prompt Templates](patterns/prompt-templates.md)** - Reusable prompt structures

## <section>Troubleshooting</section>
Error solutions and debugging guides.

### <category>Common Issues</category>
- **[Cargo Errors](troubleshooting/cargo-errors.md)** - Build and dependency issues
- **[Testing Issues](troubleshooting/testing-issues.md)** - Test failures and debugging
- **[Async Issues](troubleshooting/async-issues.md)** - Async/await troubleshooting

### <category>AI-Specific Issues</category>
- **[LLM Integration Issues](troubleshooting/llm-issues.md)** - AI assistant integration problems
- **[Prompt Engineering Issues](troubleshooting/prompt-issues.md)** - Prompt optimization problems

## <navigation>By Use Case</navigation>

### <use-case>AI-Assisted Development</use-case>
Start here for AI coding assistant integration:
1. **[AI Rules](../AI_RULES.md)** - Core development guidelines
2. **[LLM Documentation Patterns](quick-reference/llm-documentation-patterns.md)** - Document structure
3. **[Prompt Engineering](quick-reference/prompt-engineering-quick-ref.md)** - Effective AI communication
4. **[Development Process](../DEVELOPMENT_PROCESS.md)** - Three-phase methodology

### <use-case>Architecture Planning</use-case>
For architecture planning and domain analysis:
1. **[Domain Principles Guide](quick-reference/domain-principles-guide.md)** - How to create domain-principles.md
2. **[Domain Principles](domain-principles.md)** - Fortitude example implementation
3. **[Development Process](../DEVELOPMENT_PROCESS.md)** - Three-phase methodology

### <use-case>Documentation Creation</use-case>
For creating AI-friendly documentation:
1. **[LLM-Optimized Documentation](research/llm-optimized-documentation.md)** - Complete guide
2. **[LLM Documentation Patterns](quick-reference/llm-documentation-patterns.md)** - Quick patterns
3. **[Updated README Guidelines](README.md)** - Formatting standards

### <use-case>Rust Development</use-case>
For Rust-specific patterns and best practices:
1. **[Rust Workspace Patterns](patterns/rust-workspace-patterns.md)** - Multi-crate workspace configuration
2. **[Classification Algorithms](patterns/classification-algorithms.md)** - Text classification implementation
3. **[CLI Testing Patterns](patterns/cli-testing-patterns.md)** - Comprehensive CLI testing strategies
4. **[Async Patterns](patterns/async-patterns.md)** - Async development

### <use-case>Research and Analysis</use-case>
For systematic research and knowledge gaps:
1. **[Prompt Engineering Concepts](research/prompt-engineering-concepts.md)** - Research methodology
2. **[LLM-Optimized Documentation](research/llm-optimized-documentation.md)** - Implementation patterns
3. **[Knowledge Gap Analysis](../research/research_needed.md)** - Systematic gap identification

## <recent>Recent Updates</recent>
- **2025-07-09**: Added advanced classification research: dynamic threshold adjustment, rule composition algorithms, semantic analysis integration
- **2025-07-09**: Added research implementations: Claude API integration, research template system, quality validation framework
- **2025-07-08**: Added Rust development patterns: workspace configuration, classification algorithms, CLI testing
- **2025-07-08**: Integrated research results into reference library
- **2025-07-08**: Moved and organized Rust Production APIs Quick Reference to correct location
- **2025-07-08**: Added Domain Principles Guide for architecture planning
- **2025-07-08**: Added Fortitude Domain Principles as example implementation
- **2025-07-08**: Added Systems Reference Documentation section
- **2025-07-08**: Added Production-Ready Rust APIs implementation guide
- **2025-07-08**: Added Quality Evaluation System for research content assessment
- **2025-07-08**: Added Observability System for monitoring and performance optimization
- **2025-07-08**: Initial reference library structure created
- **2025-07-08**: Added LLM-optimized documentation patterns
- **2025-07-08**: Added prompt engineering concepts and patterns
- **2025-07-08**: Created quick reference summaries

## <search>Search Tips for AI Agents</search>

### <search-strategy>Before New Research</search-strategy>
1. **Check Quick Reference** - Start with `quick-reference/` for immediate answers
2. **Search Research Section** - Use `research/` for comprehensive implementation guides
3. **Look for Patterns** - Check `patterns/` for reusable implementation examples
4. **Review Troubleshooting** - Check `troubleshooting/` for error solutions

### <search-strategy>Effective Search Queries</search-strategy>
- **Technology research**: "rust async", "tokio patterns", "error handling"
- **Documentation help**: "llm documentation", "semantic markup", "ai friendly"
- **Implementation help**: "prompt engineering", "testing patterns", "configuration"
- **Troubleshooting**: "cargo errors", "async issues", "llm integration"

### <search-strategy>Cross-Reference Navigation</search-strategy>
- Documents include `<references>See Also</references>` sections
- Quick references link to comprehensive guides
- Research documents cross-reference related patterns
- Use INDEX.md as navigation hub

---

**Goal**: Enable AI agents to find precise, actionable information quickly without context window bloat. Every document follows LLM-optimized patterns for maximum comprehension and utility.