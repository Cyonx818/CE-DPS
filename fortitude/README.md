# Fortitude

<meta>
  <title>Fortitude - AI-Assisted Development Methodology</title>
  <type>project_overview</type>
  <audience>mixed</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-08</updated>
</meta>

## <summary priority="high">Project Overview</summary>

Fortitude is an **intelligent AI knowledge pipeline** that solves the knowledge gap problem in AI-assisted development. It automatically generates contextual documentation optimized for AI consumption, featuring real-time learning from user feedback, comprehensive performance monitoring, and advanced intelligence capabilities. The system operates in responsive (on-demand research) and proactive (automated gap analysis) modes with self-improving quality through adaptive learning algorithms.

## <philosophy>Development Philosophy</philosophy>

Fortitude emphasizes:
- **Structured AI-Human Collaboration**: Clear roles and responsibilities
- **Quality-First Development**: Comprehensive testing and documentation
- **Methodology Evolution**: Continuous improvement of development processes
- **Knowledge Management**: Organized reference systems for AI assistants
- **Self-Improving Intelligence**: Real-time learning and adaptation from user feedback
- **Enterprise Reliability**: Production-ready monitoring, health checks, and observability

## <features>What Fortitude Provides</features>

### <feature-category>AI Development Infrastructure</feature-category>
- **CE-DPS Integration**: Fortitude operates as an integrated knowledge management platform within the CE-DPS methodology
- **Multi-crate Workspace**: Modular Rust architecture with specialized crates
- **MCP Server**: Model Context Protocol server for Claude Code integration

### <feature-category>Core Capabilities</feature-category>
- **Research Pipeline**: Automated AI knowledge generation and gap analysis
- **Learning System**: Real-time adaptation from user feedback and performance metrics
- **Classification Engine**: Intelligent content categorization and prioritization
- **Monitoring Infrastructure**: Comprehensive health checks and observability

### <feature-category>Testing Infrastructure</feature-category>
- **Multi-layered testing**: Unit, integration, property-based, and performance tests
- **Anchor tests**: Permanent regression tests for critical functionality
- **Test organization**: Structured directories with comprehensive documentation
- **Quality gates**: Automated quality checks and requirements

### <feature-category>Real-time Learning System</feature-category>
- **User feedback collection**: Automated quality improvement through feedback analysis
- **Pattern recognition**: Usage pattern analysis from API and MCP interactions
- **Adaptive algorithms**: System optimization based on learning insights
- **Performance optimization**: Provider selection and caching optimization
- **Template integration**: Prompt optimization through learning data
- **Vector storage**: Learning data persistence with semantic search capabilities

### <feature-category>Performance Monitoring and Observability</feature-category>
- **Metrics collection**: Performance data gathering for all system components
- **Health monitoring**: System health checks and status reporting
- **Alert system**: Automated notifications for critical performance issues
- **Dashboard integration**: Real-time metrics for API and MCP dashboards
- **Distributed tracing**: Request flow tracking across system boundaries
- **Resource monitoring**: CPU, memory, network, and disk utilization tracking

### <feature-category>Development Configuration</feature-category>
- **Rust project setup**: `Cargo.toml` with development dependencies
- **Git configuration**: Comprehensive `.gitignore` patterns
- **IDE integration**: Settings for various development environments

## <quickstart>Quick Start</quickstart>

### <quickstart-step>1. Clone Fortitude</quickstart-step>
```bash
git clone [repository-url] your-new-project
cd your-new-project
```

### <quickstart-step>2. Build and Test</quickstart-step>
```bash
# Build the project
cargo build

# Run tests
cargo test
```

### <quickstart-step>3. Set Up API Keys</quickstart-step>

Fortitude supports multiple LLM providers. Set up your API keys using either environment variables or a `.env` file:

#### **Option A: Using .env file (Recommended)**
```bash
# Copy the example environment file
cp .env.example .env

# Edit .env and add your API keys
# OPENAI_API_KEY=sk-your-openai-key-here
# ANTHROPIC_API_KEY=sk-ant-your-anthropic-key-here  
# GOOGLE_API_KEY=AIza-your-google-key-here
```

#### **Option B: Using Environment Variables**
```bash
export OPENAI_API_KEY="sk-your-openai-key-here"
export ANTHROPIC_API_KEY="sk-ant-your-anthropic-key-here"
export GOOGLE_API_KEY="AIza-your-google-key-here"
```

#### **Get API Keys:**
- **OpenAI**: [platform.openai.com/api-keys](https://platform.openai.com/api-keys)
- **Anthropic Claude**: [console.anthropic.com](https://console.anthropic.com/)
- **Google Gemini**: [makersuite.google.com/app/apikey](https://makersuite.google.com/app/apikey)

#### **Test Your Setup:**
```bash
# Test provider connectivity
cargo run -- provider health-check

# Run a research query
cargo run -- research --topic "Rust async programming patterns"
```

### <quickstart-step>4. Customize for Your Project</quickstart-step>
- Update `AI_RULES.md` with project-specific context
- Create `docs/ideas/project-brainstorming.md` with your vision
- Define `docs/reference_library/domain-principles.md`
- Review and modify methodology documents as needed

### <quickstart-step>5. Start AI-Assisted Development</quickstart-step>
- Reference `AI_RULES.md` when working with AI coding assistants
- Use `DEVELOPMENT_PROCESS.md` for systematic development
- Follow the three-phase development process (Plan → Sprint → Execute)

## <methodology>Core Methodology</methodology>

**Three-Phase Development Process** optimized for AI assistance:

1. **Plan**: Architecture design and roadmap creation (30-60 min)
2. **Sprint**: Implementation planning and research (15-30 min)  
3. **Execute**: TDD-based development (60-180 min)

See `DEVELOPMENT_PROCESS.md` for complete details.

## <key-features>Key Features</key-features>

### <feature>Documentation-Driven Development</feature>
- Documentation drives development, not the reverse
- AI-friendly organization with semantic markup
- Context-rich guidance for optimal AI assistance

### <feature>Comprehensive Testing Strategy</feature>
- Multi-layered testing approach (unit, integration, property, performance)
- Test-driven development methodology
- Anchor tests for critical functionality

### <feature>AI Partnership Framework</feature>
- Treat AI as development partner, not just code generator
- Communication standards that promote technical judgment
- Context management for optimal AI assistance

### <feature>Quality Assurance Integration</feature>
- Built-in quality gates and review requirements
- Systematic debugging processes
- Code standards and testing requirements

## <structure>Directory Structure</structure>

```
fortitude/
├── docs/                        # Comprehensive documentation
│   ├── architecture/            # System design docs
│   ├── planning/                # Implementation plans
│   ├── research/                # Technical research
│   └── reference_library/       # AI knowledge management
├── src/                         # Rust source code
│   ├── lib.rs                  # Library root
│   ├── main.rs                 # Binary root
│   ├── learning/               # Real-time learning system
│   │   ├── mod.rs              # Learning system interfaces
│   │   ├── adaptation.rs       # System adaptation algorithms
│   │   ├── pattern_recognition.rs # Usage pattern analysis
│   │   ├── optimization.rs     # Performance optimization
│   │   ├── storage.rs          # Learning data persistence
│   │   ├── monitoring.rs       # Learning system monitoring
│   │   └── config.rs           # Learning configuration
│   ├── monitoring/             # Performance monitoring
│   │   ├── mod.rs              # Monitoring interfaces
│   │   ├── metrics.rs          # Performance metrics collection
│   │   ├── health.rs           # Health check system
│   │   ├── alerts.rs           # Alert system
│   │   ├── tracing.rs          # Distributed tracing
│   │   └── config.rs           # Monitoring configuration
│   └── [other modules]         # Classification, research, storage, etc.
├── tests/                       # Multi-layered testing
│   ├── README.md               # Testing guide
│   ├── unit/                   # Unit tests
│   ├── integration/            # Integration tests
│   ├── anchor/                 # Permanent regression tests
│   ├── property/               # Property-based tests
├── crates/                      # Multi-crate workspace
│   ├── fortitude-api-server/   # JSON API server with learning/monitoring endpoints
│   ├── fortitude-mcp-server/   # MCP server with learning/monitoring tools
│   ├── fortitude-core/         # Core research and classification logic
│   ├── fortitude-cli/          # Command-line interface
│   ├── fortitude-types/        # Shared type definitions
│   └── fortitude-test-utils/   # Testing utilities
├── benches/                     # Performance benchmarks
├── reference_library/           # Local knowledge cache
├── monitoring_data/             # Runtime monitoring data
└── Cargo.toml                   # Rust workspace configuration
```

## <usage>CE-DPS Integration</usage>

Fortitude operates as an integrated knowledge management platform within the CE-DPS (Context Engineered Development Process Suite) methodology:

1. **Follow CE-DPS methodology** for all development work
2. **Use the main CE-DPS CLAUDE.md** for AI assistant context and guidance
3. **Leverage Fortitude MCP server** for knowledge management and research
4. **Reference CE-DPS documentation** for implementation patterns and quality standards
5. **Follow testing requirements** - never skip TDD process
6. **Update documentation** as part of every development cycle

## <best-practices>Best Practices</best-practices>

- **Plan Before Coding**: Use the planning directory structure
- **Document Decisions**: Capture architectural decisions and rationale
- **Test Everything**: Follow comprehensive testing requirements
- **Manage Technical Debt**: Track and address systematically
- **Version Control**: Commit frequently with clear messages
- **AI Partnership**: Use AI as development partner, not tool

## <getting-started>Getting Started</getting-started>

1. **Review `GETTING_STARTED.md`** for human onboarding guide
2. **Create your project vision** in `docs/ideas/project-brainstorming.md`
3. **Define domain principles** in `docs/reference_library/domain-principles.md`
4. **Start Phase 1 planning** using the AI prompts in `GETTING_STARTED.md`

**Ready to start building?** Update this README with your project details and begin your AI-assisted development journey!

---

