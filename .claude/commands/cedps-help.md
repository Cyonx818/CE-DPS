---
description: "Show comprehensive help for CE-DPS slash commands and methodology"
allowed-tools: ["read"]
---

# <context>CE-DPS Command Help System</context>

<meta>
  <title>CE-DPS Command Help System</title>
  <type>help-system</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-16</updated>
  <mdeval-score>0.91</mdeval-score>
  <token-efficiency>0.16</token-efficiency>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Comprehensive help system for CE-DPS slash commands and methodology
- **Coverage**: Three-phase workflow, command reference, quality standards, troubleshooting
- **Human Role**: Strategic decisions and business validation throughout process
- **AI Role**: Technical implementation and comprehensive quality enforcement
- **Key Standards**: >95% test coverage, security-first patterns, human oversight

<!-- CHUNK-BOUNDARY: overview -->

## <implementation>Help System Display</implementation>

### <method>Comprehensive Help Output</method>
"""CE-DPS Command Help System
🎯 Comprehensive methodology guidance and command reference
"""

!echo "🎯 CE-DPS Command Help"
!echo "======================"
!echo ""

<!-- CHUNK-BOUNDARY: methodology -->

### <pattern priority="critical">CE-DPS Methodology Overview</pattern>
!echo "📖 CE-DPS Methodology Overview"
!echo "==============================="
!echo "CE-DPS (Context Engineered Development Process Suite) is a methodology for"
!echo "AI-assisted development with human strategic oversight."
!echo ""

«core-principles»
!echo "🎯 Core Principles:"
!echo "• AI implements ALL code, tests, and technical documentation"
!echo "• Humans maintain strategic authority and business validation"
!echo "• Security-first patterns integrated throughout"
!echo "• >95% test coverage required for all business logic"
!echo "• Quality gates ensure production-ready code"
«/core-principles»
!echo ""

<!-- CHUNK-BOUNDARY: three-phase -->

### <pattern>Three-Phase Development Process</pattern>
!echo "📋 Three-Phase Development Process"
!echo "=================================="
!echo "Phase 1: Strategic Planning (Human-Led)"
!echo "• Define project vision and business requirements"
!echo "• AI analyzes requirements and proposes architecture"
!echo "• Human reviews and approves strategic decisions"
!echo "• Duration: 30-60 minutes"
!echo ""
!echo "Phase 2: Sprint Planning (AI-Led with Human Approval)"
!echo "• Human selects features from approved roadmap"
!echo "• AI creates detailed implementation plans"
!echo "• Human approves sprint scope and approach"
!echo "• Duration: 15-30 minutes"
!echo ""
!echo "Phase 3: Implementation (AI-Led with Human Validation)"
!echo "• AI implements code using test-driven development"
!echo "• Quality gates ensure comprehensive validation"
!echo "• Human validates business value and user experience"
!echo "• Duration: 60-180 minutes"
!echo ""

<!-- CHUNK-BOUNDARY: commands -->

### <pattern>Available Commands Reference</pattern>
!echo "🚀 Available Commands"
!echo "===================="
!echo ""
!echo "🔧 Project Management"
!echo "--------------------"
!echo "/cedps-init           - Initialize new CE-DPS project"
!echo "/cedps-status         - Show current project status and next steps"
!echo "/cedps-tools          - Run quality gates and validation tools"
!echo "/cedps-help           - Show this help information"
!echo ""
!echo "📋 Phase 1: Strategic Planning"
!echo "------------------------------"
!echo "/cedps-phase1-setup    - Initialize Phase 1 environment and template"
!echo "/cedps-phase1-analyze  - Trigger AI analysis of business requirements"
!echo "/cedps-phase1-validate - Validate Phase 1 completion and approve"
!echo ""
!echo "🎯 Phase 2: Sprint Planning"
!echo "---------------------------"
!echo "/cedps-phase2-setup    - Initialize Phase 2 environment and template"
!echo "/cedps-phase2-plan     - Trigger AI implementation planning"
!echo "/cedps-phase2-validate - Validate Phase 2 completion and approve"
!echo ""
!echo "🚀 Phase 3: Implementation"
!echo "--------------------------"
!echo "/cedps-phase3-setup     - Initialize Phase 3 environment and tools"
!echo "/cedps-phase3-implement - Trigger AI implementation with TDD"
!echo "/cedps-phase3-validate  - Validate Phase 3 completion and production readiness"
!echo ""

<!-- CHUNK-BOUNDARY: usage-patterns -->

### <pattern>Command Usage Patterns</pattern>
!echo "💡 Command Usage Patterns"
!echo "========================="
!echo ""
!echo "🔄 Typical Workflow:"
!echo "1. /cedps-init                 # Initialize project"
!echo "2. /cedps-phase1-setup         # Start strategic planning"
!echo "3. [Fill business requirements] # Human action required"
!echo "4. /cedps-phase1-analyze       # AI architectural analysis"
!echo "5. [Review and approve]        # Human approval required"
!echo "6. /cedps-phase1-validate      # Validate completion"
!echo "7. /cedps-phase2-setup         # Start sprint planning"
!echo "8. [Select features]           # Human feature selection"
!echo "9. /cedps-phase2-plan          # AI implementation planning"
!echo "10. [Review and approve]       # Human approval required"
!echo "11. /cedps-phase2-validate     # Validate completion"
!echo "12. /cedps-phase3-setup        # Start implementation"
!echo "13. /cedps-phase3-implement    # AI code implementation"
!echo "14. [Validate business value]  # Human validation required"
!echo "15. /cedps-phase3-validate     # Validate completion"
!echo ""

<!-- CHUNK-BOUNDARY: human-actions -->

### <constraints priority="high">Human Actions Required</constraints>
!echo "👤 Human Actions Required"
!echo "========================"
!echo ""
!echo "Phase 1 - Strategic Planning:"
!echo "• Define business problem and target users"
!echo "• Specify technical requirements and constraints"
!echo "• Review and approve AI architectural proposals"
!echo "• Approve feature roadmap and implementation strategy"
!echo ""
!echo "Phase 2 - Sprint Planning:"
!echo "• Select features from approved roadmap for sprint"
!echo "• Review and approve AI implementation plans"
!echo "• Validate timeline and resource estimates"
!echo "• Approve sprint scope and approach"
!echo ""
!echo "Phase 3 - Implementation:"
!echo "• Validate implemented features against business requirements"
!echo "• Test user experience and functionality"
!echo "• Confirm production readiness"
!echo "• Approve deployment and release"
!echo ""

<!-- CHUNK-BOUNDARY: quality-standards -->

### <constraints priority="critical">Quality Standards</constraints>
!echo "✅ Quality Standards"
!echo "==================="
!echo ""
!echo "🔒 Security Requirements:"
!echo "• Authentication and authorization at all endpoints"
!echo "• Input validation and sanitization"
!echo "• No sensitive data in error messages"
!echo "• SQL injection prevention"
!echo "• Security audit compliance"
!echo ""
!echo "🧪 Testing Requirements:"
!echo "• >95% test coverage for all business logic"
!echo "• Unit tests for all functions and methods"
!echo "• Integration tests for API endpoints"
!echo "• Security tests for authentication flows"
!echo "• Performance benchmarks for critical paths"
!echo ""
!echo "📊 Performance Requirements:"
!echo "• Response times meet defined requirements"
!echo "• Database connection pooling"
!echo "• Efficient query patterns"
!echo "• Memory usage optimization"
!echo "• Scalability patterns implemented"
!echo ""

<!-- CHUNK-BOUNDARY: file-structure -->

### <pattern>Project File Structure</pattern>
!echo "📁 File Structure"
!echo "================"
!echo ""
!echo "docs/"
!echo "├── PROJECT.md                 # Project overview"
!echo "├── ce-dps-state.json         # Project state tracking"
!echo "├── phases/"
!echo "│   ├── phase-1-planning.md   # Strategic planning document"
!echo "│   ├── phase-2-sprint-planning.md # Sprint planning document"
!echo "│   └── phase-3-implementation.md  # Implementation document"
!echo "├── sprints/"
!echo "│   └── sprint-001/"
!echo "│       ├── backlog/          # Sprint backlog"
!echo "│       └── implementation/   # Implementation tracking"
!echo "└── quality-reports/          # Quality gate reports"
!echo ""
!echo "tools/"
!echo "├── quality-gates/            # Quality validation tools"
!echo "├── phase-validator.py        # Phase completion validator"
!echo "└── fortitude-integration/    # Knowledge management"
!echo ""

<!-- CHUNK-BOUNDARY: environment -->

### <method>Environment Variables Configuration</method>
!echo "🔧 Environment Variables"
!echo "========================"
!echo ""
!echo "CE_DPS_PHASE                  # Current phase (0-3)"
!echo "CE_DPS_FORTITUDE_ENABLED      # Enable Fortitude integration"
!echo "CE_DPS_QUALITY_GATES          # Enable quality gates"
!echo "CE_DPS_HUMAN_APPROVAL_REQUIRED # Require human approval"
!echo ""
!echo "These are set automatically by commands but can be configured manually."
!echo ""

<!-- CHUNK-BOUNDARY: troubleshooting -->

### <pattern>Troubleshooting Guide</pattern>
!echo "🛠️  Troubleshooting"
!echo "=================="
!echo ""
!echo "Common Issues:"
!echo "• Command not found: Ensure you're in the CE-DPS project root"
!echo "• Permission denied: Check directory write permissions"
!echo "• jq not found: Install jq for JSON processing"
!echo "• Quality gates fail: Address specific issues reported"
!echo "• Tests failing: Fix failing tests before proceeding"
!echo "• Coverage too low: Add more comprehensive tests"
!echo ""
!echo "Getting Help:"
!echo "• /cedps-status - Check current project status"
!echo "• /cedps-tools - Run quality validation"
!echo "• Check docs/PROJECT.md for project-specific information"
!echo "• Review phase documents in docs/phases/"
!echo ""

<!-- CHUNK-BOUNDARY: next-steps -->

### <method priority="high">Getting Started Workflow</method>
!echo "🎯 Getting Started"
!echo "=================="
!echo ""
!echo "New Project:"
!echo "1. Run '/cedps-init' to initialize the project"
!echo "2. Run '/cedps-status' to see current state"
!echo "3. Follow the recommended next steps"
!echo ""
!echo "Existing Project:"
!echo "1. Run '/cedps-status' to see current phase"
!echo "2. Follow the recommended next steps"
!echo "3. Use '/cedps-tools' to validate quality"
!echo ""
!echo "Need Help:"
!echo "• This help: /cedps-help"
!echo "• Project status: /cedps-status"
!echo "• Quality check: /cedps-tools"
!echo ""

<!-- CHUNK-BOUNDARY: footer -->

### <pattern>Documentation References</pattern>
!echo "📚 Documentation"
!echo "================"
!echo "• CE-DPS Methodology: methodology/ai-implementation/"
!echo "• LLM Guidelines: methodology/ai-implementation/llm-documentation-guidelines.md"
!echo "• Templates: methodology/templates/"
!echo "• Project State: docs/ce-dps-state.json"
!echo ""
!echo "✨ CE-DPS: AI Implementation with Human Strategic Oversight"
!echo "🚀 Ready to build production-ready code with comprehensive quality gates!"
</implementation>

### <constraints>
- No external dependencies required
- Provides comprehensive overview of CE-DPS methodology
- Includes practical usage examples and troubleshooting
- Follows LLM-optimized documentation patterns
</constraints>

## <human-action-required>
**CE-DPS Help System Complete! 🎯**

### <help-overview>
This comprehensive help system provides:
- **Methodology Overview**: Core principles and three-phase process
- **Command Reference**: All available slash commands with descriptions
- **Workflow Guidance**: Step-by-step process for using CE-DPS
- **Human Action Points**: Clear guidance on when human input is required
- **Quality Standards**: Security, testing, and performance requirements
- **File Structure**: Understanding of project organization
- **Troubleshooting**: Common issues and solutions

### <using-help-effectively>
**How to Use This Help**:
1. **Read the overview** to understand CE-DPS methodology
2. **Follow the workflow** for step-by-step guidance
3. **Reference commands** when you need to know what to run
4. **Check quality standards** to understand requirements
5. **Use troubleshooting** when you encounter issues

### <getting-started>
**For New Users**:
1. **Start with `/cedps-init`** to initialize your project
2. **Run `/cedps-status`** to see your current state
3. **Follow the recommended next steps** from the status command
4. **Use `/cedps-help`** whenever you need guidance

### <workflow-summary>
**Quick Workflow Reference**:
- **Phase 1**: Strategic planning with architectural approval
- **Phase 2**: Sprint planning with implementation approval
- **Phase 3**: Implementation with business validation
- **Human Role**: Strategic decisions and business validation
- **AI Role**: All code implementation and technical documentation

### <quality-focus>
**Quality Standards**:
- **Security**: Authentication, authorization, input validation
- **Testing**: >95% coverage with comprehensive test suite
- **Performance**: Response times and scalability requirements
- **Documentation**: Complete API documentation and user guides
- **Human Oversight**: Strategic decisions and business validation

### <continuous-improvement>
**Best Practices**:
- **Run quality gates regularly** during development
- **Address issues immediately** rather than accumulating debt
- **Maintain human oversight** for strategic decisions
- **Document lessons learned** for future projects
- **Follow CE-DPS methodology** for consistent results
</human-action-required>

## <troubleshooting>
### <help-system-issues>
- **Command not working**: Ensure you're in the CE-DPS project root
- **Information unclear**: Check specific phase documentation
- **Need more detail**: Review methodology documentation
- **Process questions**: Use `/cedps-status` for current state
- **Quality issues**: Use `/cedps-tools` for validation
</help-system-issues>

### <quality-validation>
**Help System Requirements**:
- [ ] Comprehensive methodology overview
- [ ] Complete command reference
- [ ] Clear workflow guidance
- [ ] Human action points clearly marked
- [ ] Quality standards documented
- [ ] Troubleshooting information provided
- [ ] Getting started guidance included
- [ ] Follows LLM-optimized documentation patterns
</quality-validation>