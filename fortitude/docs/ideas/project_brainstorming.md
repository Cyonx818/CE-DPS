# Fortitude: Automated AI Knowledge Pipeline

## 🎯 Project Overview

Fortitude is an **automated research system** designed to solve the critical knowledge gap problem in AI-assisted development. Rather than manually researching domain-specific knowledge when AI coding agents encounter unfamiliar concepts, fortitude automatically identifies knowledge gaps and generates contextual documentation optimized for AI consumption.

The system operates in two modes: **responsive** (on-demand research) and **proactive** (automated gap analysis), creating a self-improving knowledge ecosystem for the Concordia workspace.

---

## 📌 Primary Goals

1. **Eliminate AI knowledge bottlenecks** by providing on-demand research capabilities to Claude Code and other AI tools
2. **Generate AI-optimized documentation** using Claude 4.0 Sonnet API with advanced prompt engineering techniques
3. **Build institutional knowledge** that improves development velocity across all Concordia projects
4. **Create seamless integration** with existing AI development workflows through multiple interface options
5. **Enable rapid MVP deployment** to satisfy immediate research demands while allowing for future growth

---

## 🧠 Operational Modes

### 💡 Responsive Mode (MVP Priority)
**"On-demand knowledge delivery"**

- Receives specific research requests from Claude Code, other LLMs, or humans
- Multiple interface options:
  - CLI commands: `fortitude research "Rust async patterns for game engines"`
  - Model Context Protocol (MCP) server for Claude Code integration
  - JSON API for other systems
- **Output**: Targeted, immediately usable documentation saved to reference_library/
- **Speed**: Fast turnaround (seconds to minutes)
- **MVP Target**: 60-second research-to-documentation pipeline

**Example Use Cases:**
- Claude Code encounters unfamiliar Rust patterns during development
- Need best practices research for specific technologies (ratatui, tokio, etc.)
- Domain-specific knowledge for game development concepts
- Architecture decision research and rationale

### 🔍 Proactive Mode (Future Development)
**"Intelligent gap analysis and knowledge building"**

- Automated scanning of project documentation, code comments, README files
- Knowledge gap identification through:
  - TODO comments mentioning research needs
  - Architecture decisions lacking rationale documentation
  - Technologies used without best practice documentation
  - Domain concepts referenced but not explained
- **Output**: Comprehensive reference library expansion
- **Speed**: Background process (minutes to hours)
- **Strategy**: Builds institutional knowledge proactively

---

## 🛠️ MVP Implementation Strategy

### Phase 1: CLI-First Approach (Days to implement)
```bash
# Primary MVP interface
fortitude research "Terminal UI best practices with ratatui"
fortitude research "Turn-based combat state management patterns"

# Output: Well-formatted markdown in reference_library/
```

**Core MVP Architecture:**
```
fortitude/
├── src/
│   ├── research_engine.rs  // Core research logic with Claude API
│   ├── claude_client.rs    // API integration with structured prompts
│   ├── cache.rs           // File-based caching (avoid duplicate research)
│   ├── cli.rs             // CLI interface
│   └── main.rs            // Entry point
└── reference_library/     // Generated knowledge base
    ├── rust-async-patterns.md
    ├── terminal-ui-best-practices.md
    └── combat-system-architecture.md
```

**MVP Research Pipeline:**
1. Receive request (topic + optional context)
2. Check cache (reference_library/{topic-hash}.md exists?)
3. If not cached: Call Claude API with structured research prompts
4. Format response as AI-optimized markdown
5. Save to reference_library/
6. Return file path or content

### Phase 2: Multi-Interface Support (1-2 weeks)
- Add Model Context Protocol (MCP) server for seamless Claude Code integration
- Maintain CLI interface for human/script access
- Shared research engine across all interfaces
- Bidirectional communication support

---

## 🎯 Research Request Type Framework

### **Major Architectural Insight: Context-Aware Research**

Through research experimentation, we discovered that fortitude needs **multiple research modes** that match the cognitive load and decision context of the requesting system. This transforms fortitude from a "generic research tool" into a **cognitive assistant** that adapts its research style to match the user's mental model and current task context.

### **Research Request Types**

#### **1. Decision Research**
**Context**: "Which technology should I choose?"  
**Cognitive Load**: High-level architecture decisions  
**Output**: Executive summary, decision matrix, clear recommendation  
**Length**: 300-500 words  
**Prompt Pattern**: Lead with recommendation, justify with key differentiators

#### **2. Implementation Research**
**Context**: "How do I build this specific thing?"  
**Cognitive Load**: Active development work  
**Output**: Working code examples, patterns, gotchas  
**Length**: 800-1500 words  
**Prompt Pattern**: Code-heavy with production considerations

#### **3. Troubleshooting Research**
**Context**: "Why isn't this working?"  
**Cognitive Load**: Problem-solving mode  
**Output**: Diagnostic steps, common issues, solutions  
**Length**: 200-400 words  
**Prompt Pattern**: Problem → Root causes → Solutions

#### **4. Learning Research**
**Context**: "I need to understand this concept"  
**Cognitive Load**: Knowledge acquisition  
**Output**: Conceptual explanation, examples, mental models  
**Length**: 600-1000 words  
**Prompt Pattern**: Concept → Examples → Applications

#### **5. Validation Research**
**Context**: "Is this approach correct?"  
**Cognitive Load**: Confidence building  
**Output**: Best practices, trade-offs, alternatives  
**Length**: 400-600 words  
**Prompt Pattern**: Evaluate approach → Alternatives → Recommendation

### **Enhanced CLI Interface**
```bash
# Enhanced with research types
fortitude decide "Vector databases for Rust"      # Decision research
fortitude implement "Qdrant connection pooling"   # Implementation research
fortitude troubleshoot "Qdrant connection fails"  # Troubleshooting research
fortitude learn "Vector similarity search"        # Learning research
fortitude validate "Using HNSW for documents"     # Validation research
```

### **Context-Aware Research Engine**
```rust
pub enum ResearchType {
    Decision { context: String, options: Vec<String> },
    Implementation { technology: String, use_case: String },
    Troubleshooting { problem: String, context: String },
    Learning { concept: String, application: String },
    Validation { approach: String, constraints: Vec<String> },
}

pub struct ResearchEngine {
    decision_prompts: PromptTemplate,
    implementation_prompts: PromptTemplate,
    troubleshooting_prompts: PromptTemplate,
    learning_prompts: PromptTemplate,
    validation_prompts: PromptTemplate,
}
```

### **Intelligent Request Classification**
The system automatically classifies requests to determine research type:
- Phrases like "should I choose" → Decision research
- Phrases like "how to implement" → Implementation research
- Phrases like "not working" → Troubleshooting research
- Phrases like "what is" → Learning research
- Phrases like "is this correct" → Validation research

### **Progressive Research Flows**
```rust
// Start with decision research, offer follow-up implementation research
let decision_result = engine.research(ResearchRequest {
    topic: "vector databases",
    research_type: ResearchType::Decision,
}).await?;

// Suggest follow-up: "implement Qdrant connection pooling"
```

---

## 🔍 Technical Architecture Concepts

### Interface Strategy
**Layered approach for maximum compatibility:**

1. **MCP Server** (Optimal for Claude Code)
   - Follows 2025 AI-to-tool communication standards
   - Bidirectional communication for follow-up questions
   - Future-proof integration pattern

2. **JSON API** (Universal compatibility)
   - HTTP endpoints for non-MCP systems
   - Standardized request/response format
   - Can be consumed by any AI system

3. **CLI Interface** (Human/script access)
   - Direct command-line research requests
   - Debugging and manual research workflows
   - Wraps same core logic as other interfaces

### Research Engine Design
Based on 2025 AI research best practices from reference documents:

- **Meta-prompting strategies** for optimal research quality
- **Hierarchical XML formatting** with semantic tags for AI consumption
- **Few-shot prompting** with 3-5 diverse examples
- **Prompt chaining** for complex research tasks
- **Token-efficient formatting** optimized for 200k context windows

### Knowledge Storage Strategy
- **File-based storage** (MVP): Simple, debuggable, version-controllable
- **Future: Vector database** (Qdrant) for semantic search
- **Hybrid search** combining dense vectors + keyword matching
- **Semantic chunking** with 200-400 token segments

---

## 📈 Growth Roadmap

### Phase 1: MVP (Days)
- CLI interface with basic research commands
- Claude API integration with structured prompts
- File-based caching and storage
- Basic markdown output formatting

### Phase 2: Integration (1-2 weeks)
- Model Context Protocol (MCP) server implementation
- Research type classification system
- Context-aware prompt templates for each research type
- Enhanced caching with research type awareness
- Quality validation and formatting improvements

### Phase 3: Intelligence (1-2 months)
- Vector database integration (Qdrant)
- Semantic search and retrieval capabilities
- Hybrid search combining multiple strategies
- Advanced prompt optimization and quality metrics

### Phase 4: Automation (2-3 months)
- Proactive mode implementation with research type awareness
- Automated gap analysis identifying different types of missing knowledge
- Intelligent knowledge prioritization based on research type urgency
- Multi-LLM provider support with fallbacks

### Phase 5: Advanced Features (3+ months)
- Real-time learning and adaptation
- Knowledge graph integration
- Advanced quality control and validation
- Web UI and dashboard capabilities

---

## ❓ Outstanding Research Questions

### Technical Implementation
1. **What prompt structures maximize research quality** for technical documentation generation?
2. **How should we optimize token usage** while maintaining comprehensive research outputs?
3. **What caching strategies** best balance freshness vs. cost for research results?
4. **How can we implement intelligent request deduplication** across similar topics?

### Architecture Decisions
5. **MCP vs. API-first approach** for initial implementation priority?
6. **File-based vs. database storage** for MVP knowledge persistence?
7. **Single vs. multi-LLM approach** for research redundancy and quality?
8. **How to structure reference_library/** for optimal AI consumption?

### Quality and Validation
9. **What metrics define successful research output** for AI consumption?
10. **How can we automatically validate** research accuracy and completeness?
11. **What feedback mechanisms** can improve research quality over time?
12. **How should we handle research conflicts** or outdated information?

### Integration Patterns
13. **How should fortitude communicate research results** back to requesting systems?
14. **What metadata should accompany** generated research documents?
15. **How can we integrate with existing Concordia** development workflows?
16. **What notification/alerting patterns** work best for proactive mode?

---

## 📎 Immediate Next Steps

### 🔧 MVP Development (This Week)
1. Implement basic CLI interface with `fortitude research <topic>` command
2. Set up Claude API client with initial research prompt templates
3. Create file-based caching system using topic hashes
4. Establish reference_library/ structure and markdown formatting
5. Test end-to-end research pipeline with real Concordia development needs
6. **NEW**: Experiment with decision vs implementation research prompt variations

### 📚 Research Infrastructure
1. Analyze existing Concordia documentation to identify knowledge gap patterns
2. Create initial prompt templates based on 2025 AI research best practices
3. Establish quality metrics for generated research documentation
4. Set up development workflow for rapid iteration and testing

### 🧪 Validation and Testing
1. Test research quality across different technical domains
2. Validate AI-optimized markdown formatting with Claude Code consumption
3. Measure research speed and establish performance baselines
4. Create feedback collection mechanism for research quality improvement

---

## 💭 Long-term Vision

Fortitude represents a paradigm shift toward **self-improving AI development environments**. Rather than AI agents operating with static knowledge, fortitude creates a dynamic, growing knowledge base that becomes more valuable with each development cycle.

**Strategic Value:**
- Transforms "AI doesn't know our context" from a limitation into an automated solution
- Creates institutional knowledge that persists across projects and team members
- Enables AI agents to become domain experts in Concordia-specific patterns
- Positions Concordia as having state-of-the-art AI-augmented development workflows

**Success Metrics:**
- Reduction in development time spent on manual research
- Increase in AI agent effectiveness on domain-specific tasks
- Growth in reference library comprehensiveness and usage
- Improvement in code quality through better-informed AI assistance

This system essentially creates a **multiplicative effect** where each piece of research not only solves an immediate problem but contributes to the collective intelligence available for all future development work.

---

## 🏗️ Architectural Implications of Research Type Framework

### **Major System Design Changes**

#### **1. Multi-Modal Research Engine Architecture**
Instead of one research function, we need specialized engines:
- **DecisionEngine** - Optimized for choice-making with decision matrices
- **ImplementationEngine** - Code-heavy, pattern-focused with working examples
- **TroubleshootingEngine** - Problem-solution oriented with diagnostic steps
- **LearningEngine** - Concept explanation focused with mental models
- **ValidationEngine** - Best practice verification with trade-off analysis

#### **2. Context-Aware Prompt Management**
```rust
pub struct PromptManager {
    templates: HashMap<ResearchType, PromptTemplate>,
    context_adapters: HashMap<Domain, ContextAdapter>,
}

// Different domains (web dev, systems programming, AI/ML) need different 
// prompt adaptations even for the same research type
```

#### **3. Research Result Formatting**
```rust
pub enum ResearchResult {
    Decision {
        recommendation: String,
        alternatives: Vec<Alternative>,
        decision_matrix: DecisionMatrix,
        confidence_score: f32,
    },
    Implementation {
        code_examples: Vec<CodeExample>,
        patterns: Vec<ImplementationPattern>,
        gotchas: Vec<String>,
        testing_guide: TestingGuide,
    },
    Troubleshooting {
        likely_causes: Vec<String>,
        diagnostic_steps: Vec<String>,
        solutions: Vec<Solution>,
    },
    Learning {
        concept_explanation: String,
        examples: Vec<Example>,
        mental_models: Vec<String>,
        related_concepts: Vec<String>,
    },
    Validation {
        approach_assessment: String,
        alternatives: Vec<Alternative>,
        best_practices: Vec<String>,
        trade_offs: Vec<TradeOff>,
    },
}
```

#### **4. Context-Aware Caching Strategy**
```rust
// Cache keys include research type for better hit rates
#[derive(Hash, Eq, PartialEq)]
struct CacheKey {
    topic: String,
    research_type: ResearchType,
    context_hash: u64,  // Hash of relevant context
}

// Decision research for "vector databases" is different from 
// implementation research for "vector databases"
```

### **Integration Impact**

#### **For Claude Code Integration**
Claude Code can request research that matches its current cognitive context:
- **During architecture planning**: Decision research for technology choices
- **During implementation**: Implementation research for coding patterns
- **During debugging**: Troubleshooting research for problem-solving
- **During learning**: Learning research for concept understanding
- **During code review**: Validation research for approach verification

#### **For Proactive Mode Evolution**
When scanning project docs, fortitude can identify **what type of research gap** exists:
- Missing architecture decisions → Decision research
- Incomplete implementation guides → Implementation research  
- Unresolved technical debt → Troubleshooting research
- Knowledge gaps in team understanding → Learning research
- Unvalidated approaches → Validation research

#### **For MCP Server Design**
```rust
// MCP server can receive research type hints from Claude Code
pub struct MCPResearchRequest {
    topic: String,
    research_type: Option<ResearchType>,
    context: Option<String>,
    expected_output_length: Option<OutputLength>,
}
```

### **User Experience Transformation**

#### **Intelligent Request Routing**
```rust
pub struct FortitudeCore {
    classifier: RequestClassifier,
    engines: HashMap<ResearchType, Box<dyn ResearchEngine>>,
    cache: ContextAwareCache,
}

impl FortitudeCore {
    pub async fn research(&self, request: &str) -> Result<ResearchResult, ResearchError> {
        let research_type = self.classifier.classify_request(request);
        let engine = self.engines.get(&research_type).unwrap();
        
        // Check cache with research type context
        if let Some(cached) = self.cache.get(request, &research_type) {
            return Ok(cached);
        }
        
        let result = engine.research(request, research_type).await?;
        self.cache.store(request, &research_type, &result);
        Ok(result)
    }
}
```

#### **Progressive Research Workflows**
```rust
pub struct ProgressiveResearch {
    engine: ResearchEngine,
}

impl ProgressiveResearch {
    pub async fn research_with_follow_up(&self, initial_request: &str) -> Result<ResearchFlow, ResearchError> {
        // Start with decision research
        let decision_result = self.engine.research(ResearchRequest {
            topic: initial_request.to_string(),
            research_type: ResearchType::Decision,
        }).await?;
        
        // Offer follow-up research based on decision
        let follow_up_suggestions = vec![
            format!("implement {}", decision_result.recommendation),
            format!("learn {}", decision_result.key_concepts),
            format!("validate {}", decision_result.recommendation),
        ];
        
        Ok(ResearchFlow {
            initial_result: decision_result,
            suggested_follow_ups: follow_up_suggestions,
        })
    }
}
```

### **Quality and Validation Evolution**

#### **Research Type-Specific Metrics**
- **Decision Research**: Confidence scores, alternative coverage, decision clarity
- **Implementation Research**: Code completeness, error handling coverage, production readiness
- **Troubleshooting Research**: Solution success rate, diagnostic accuracy, resolution time
- **Learning Research**: Concept clarity, example relevance, comprehension metrics
- **Validation Research**: Best practice alignment, trade-off completeness, approach confidence

#### **Adaptive Quality Control**
```rust
pub struct QualityController {
    metrics: HashMap<ResearchType, QualityMetrics>,
    feedback_loops: HashMap<ResearchType, FeedbackLoop>,
}

// Different research types need different quality validation approaches
```

This research type framework transforms fortitude from a simple research tool into a **cognitive assistant** that adapts to the user's current mental state and information needs, making it significantly more valuable for AI-assisted development workflows.

---

## 📋 Framework Development Status

The research type framework has been successfully prototyped and validated through systematic testing. Key insights and lessons learned have been documented in the companion document `prototype_lessons_learned.md`.

### **Framework Maturity**
- **Research Type Classification**: Validated through 10 knowledge gap applications
- **Prompt Optimization**: Tested with dramatic usability improvements (3x length reduction)
- **Context-Aware Architecture**: Multi-dimensional classification requirements identified
- **Quality Metrics**: Type-specific and audience-aware validation approaches defined

### **Implementation Ready**
The framework has evolved from concept to implementation-ready architecture with:
- Clear research type definitions and patterns
- Validated prompt templates and completion guidelines
- Context-aware caching and quality control strategies
- Progressive disclosure patterns for optimal user experience

Detailed lessons learned, architectural implications, and implementation strategies are documented in `prototype_lessons_learned.md` for development reference.

