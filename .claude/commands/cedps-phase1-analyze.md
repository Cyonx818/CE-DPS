---
description: "Trigger Phase 1 AI analysis of business requirements with comprehensive architectural research"
allowed-tools: ["read", "write", "bash"]
---

# <context>CE-DPS Phase 1: AI Strategic Analysis</context>

## <summary priority="high">
Trigger comprehensive AI analysis of business requirements to produce architectural recommendations, technology evaluations, and implementation strategies for human strategic approval.

## <method>AI Analysis Orchestration</method>

### <implementation>
!echo "🧠 Initiating Phase 1 AI Analysis..."

# Validate Phase 1 setup
!if [ ! -f "docs/phases/phase-1-planning.md" ]; then 
    echo "❌ Error: Phase 1 not set up. Run '/cedps-phase1-setup' first."
    exit 1
fi

# Validate business requirements are filled out (bypass in SKYNET mode)
!if [[ "$SKYNET" != "true" ]]; then
    if grep -q "\[Enter" docs/phases/phase-1-planning.md; then
        echo "❌ Error: Business requirements template not completed."
        echo "💡 Complete all [Enter...] sections in docs/phases/phase-1-planning.md"
        echo "📋 Required sections: Business Context, Strategic Requirements, Constraints"
        exit 1
    fi
else
    echo "🤖 SKYNET mode: Business requirements auto-populated, proceeding with analysis"
fi

# Update project state
!jq '.phase_1_analysis_started = now' docs/ce-dps-state.json > docs/ce-dps-state.tmp && mv docs/ce-dps-state.tmp docs/ce-dps-state.json

!echo "✅ Requirements validated. Triggering AI analysis..."
!echo "📋 Business requirements loaded from docs/phases/phase-1-planning.md"
!echo "🔍 Beginning comprehensive architectural analysis..."

# Auto-proceed in SKYNET mode after analysis
!if [[ "$SKYNET" == "true" ]]; then
    echo ""
    echo "🤖 SKYNET mode: AI analysis will auto-approve architectural decisions"
    echo "⚡ Strategic decisions will be made autonomously based on best practices"
    echo "⚡ Auto-transitioning to Phase 1 validation after analysis completion"
fi
</implementation>

### <constraints>
- Phase 1 must be set up first
- Business requirements template must be completed
- All placeholders must be replaced with actual content
- jq command required for state management
</constraints>

## <claude-prompt>
I am executing CE-DPS Phase 1 strategic analysis based on the completed business requirements.

### <business-context>
@docs/phases/phase-1-planning.md

### <analysis-requirements>
Based on the business requirements above, provide comprehensive analysis in the following areas:

#### <architecture-analysis>
**System Architecture Design**:
- Propose overall system architecture with component relationships
- Include security-first design patterns and authentication flows
- Address scalability requirements and performance constraints
- Define data architecture and storage strategies
- Identify integration points and API design approaches

**Requirements**: Use proven architectural patterns, emphasize security, plan for scalability defined in constraints.

#### <technology-evaluation>
**Technology Stack Recommendations**:
- Evaluate and recommend specific technologies for each layer
- Provide rationale for each technology choice
- Consider alternatives and trade-offs
- Address technical constraints specified in requirements
- Include deployment and infrastructure considerations

**Requirements**: Support business requirements, align with team constraints, prioritize security and maintainability.

#### <implementation-strategy>
**Development Approach**:
- Create phased implementation roadmap
- Define feature prioritization strategy
- Estimate development timeline and resource requirements
- Plan testing and quality assurance approach
- Define deployment and release strategies

**Requirements**: Maximize early user value, realistic timelines, comprehensive testing (>95% coverage).

#### <risk-assessment>
**Risk Analysis and Mitigation**:
- Identify technical, business, and operational risks
- Provide specific mitigation strategies for each risk
- Define contingency plans for critical risks
- Address security vulnerabilities and compliance requirements
- Plan for performance and scalability challenges

**Requirements**: Comprehensive risk coverage, actionable mitigation strategies, business continuity planning.

### <output-format>
**Use CE-DPS LLM documentation patterns**:
- Apply semantic markup with `<architecture-analysis>`, `<technology-evaluation>`, etc.
- Use progressive disclosure (summary → evidence → implementation)
- Include structured data where appropriate
- Provide token-efficient but comprehensive coverage
- Mark sections requiring human approval with `<human-approval-required>`

### <quality-requirements>
**CE-DPS Standards**:
- Security-first architecture with comprehensive authentication/authorization
- Performance requirements explicitly addressed
- >95% test coverage approach defined
- Integration patterns with existing systems identified
- Deployment and operational considerations included
- Clear separation of strategic (human) and tactical (AI) concerns

### <human-approval-points>
**Strategic Decisions Requiring Human Approval**:
- Overall system architecture and component design
- Technology stack selection and alternatives
- Security architecture and compliance approaches
- Performance and scalability trade-offs
- Integration strategies with existing systems
- Implementation timeline and resource allocation

### <fortitude-integration>
**Knowledge Management**:
- Query existing architectural patterns before proposing new approaches
- Reference proven security and performance patterns
- Apply domain-specific best practices
- Document architectural decisions for future reference

### <deliverables>
**Phase 1 Analysis Output**:
1. **Architecture Document**: Complete system design with diagrams
2. **Technology Evaluation**: Detailed stack recommendations with rationale
3. **Implementation Roadmap**: Phased development plan with timelines
4. **Risk Assessment**: Comprehensive risk analysis with mitigation strategies
5. **Human Review Checklist**: Structured approval process for strategic decisions

**Success Criteria**:
- All business requirements addressed in architectural design
- Technology choices align with constraints and capabilities
- Implementation approach maximizes business value delivery
- Risks are identified and have actionable mitigation plans
- Human approval points are clearly marked and justified

Please begin the comprehensive Phase 1 analysis now. When complete:

**If SKYNET mode is enabled ($SKYNET=true):**
- Auto-generate strategic approval decisions based on best practices
- Mark all analysis sections as "✅ Approved - SKYNET: [reasoning]"
- Proceed automatically to Phase 1 validation

**If human oversight mode ($SKYNET=false or unset):**
- Provide structured human review sections for strategic approval before proceeding to Phase 2
</claude-prompt>

## <human-action-required>
!if [[ "$SKYNET" == "true" ]]; then
    echo "🤖 **SKYNET MODE**: AI analysis will auto-approve strategic decisions"
    echo "⚡ Architectural decisions made autonomously using best practices"
    echo "⚡ System will auto-proceed to Phase 1 validation"
    echo "⚡ No human strategic review required"
    exit 0
fi

**AI Analysis In Progress... 🧠**

### <current-status>
Claude Code is now performing comprehensive Phase 1 analysis including:
- **Architecture Analysis**: System design with security-first patterns
- **Technology Evaluation**: Stack recommendations with rationale
- **Implementation Strategy**: Phased development roadmap
- **Risk Assessment**: Comprehensive risk analysis with mitigation

### <what-to-expect>
**Claude Code will provide**:
1. **Detailed architectural recommendations** based on your requirements
2. **Technology stack evaluation** with specific tool recommendations
3. **Implementation timeline** with realistic effort estimates
4. **Risk analysis** with specific mitigation strategies
5. **Human approval sections** for strategic decision making

### <your-next-actions>
**When analysis is complete, you will need to**:
1. **Review architecture proposals** against business requirements
2. **Evaluate technology recommendations** for organizational fit
3. **Assess implementation timeline** for feasibility
4. **Approve or request changes** to strategic decisions
5. **Run `/cedps-phase1-validate`** when ready to proceed

### <approval-process>
**You will approve/reject each section**:
- ✅ **Approved**: Strategic decision accepted, proceed as planned
- ❓ **Needs Clarification**: Request more information or alternatives
- ❌ **Requires Changes**: Reject proposal, request revised approach

### <quality-standards>
**Validate that analysis includes**:
- [ ] Security-first architecture design
- [ ] Performance requirements addressed
- [ ] Scalability patterns identified
- [ ] Integration approaches defined
- [ ] Testing strategy comprehensive (>95% coverage)
- [ ] Risk mitigation strategies actionable
</human-action-required>

## <troubleshooting>
### <common-errors>
- **"Phase 1 not set up"**: Run `/cedps-phase1-setup` first
- **"Template not completed"**: Fill out all [Enter...] sections in phase-1-planning.md
- **"Requirements too vague"**: Provide specific, measurable requirements
- **Analysis incomplete**: Claude Code may need clarification on requirements
</common-errors>

### <quality-validation>
**Phase 1 Analysis Requirements**:
- [ ] All business requirements addressed in analysis
- [ ] Security considerations prominent throughout
- [ ] Performance requirements explicitly planned
- [ ] Testing approach comprehensive and realistic
- [ ] Human approval points clearly marked
- [ ] LLM-optimized documentation patterns used
</quality-validation>