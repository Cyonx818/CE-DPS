---
description: "Trigger Phase 2 AI implementation planning with complexity analysis and detailed task breakdown"
allowed-tools: ["read", "write", "bash"]
---

# <context>CE-DPS Phase 2: AI Implementation Planning</context>

## <summary priority="high">
Trigger comprehensive AI implementation planning for selected sprint features, including complexity analysis, detailed task breakdown, and resource estimation for human approval.

## <method>AI Implementation Planning Orchestration</method>

### <implementation>
!echo "🧠 Initiating Phase 2 AI Implementation Planning..."

# Validate Phase 2 setup
!if [ ! -f "docs/phases/phase-2-sprint-planning.md" ]; then
    echo "❌ Error: Phase 2 not set up. Run '/cedps phase2 setup' first."
    exit 1
fi

# Validate feature selection is completed
!if ! grep -q "Selected Features for Sprint" docs/phases/phase-2-sprint-planning.md; then
    echo "❌ Error: Feature selection not completed."
    echo "💡 Complete feature selection in docs/phases/phase-2-sprint-planning.md"
    exit 1
fi

# Check that features are actually selected (not just template)
!if grep -q "\[Choose.*features\]" docs/phases/phase-2-sprint-planning.md; then
    echo "❌ Error: Feature selection template not completed."
    echo "💡 Replace template text with actual feature selections"
    echo "📋 Required: Select 2-4 specific features from Phase 1 roadmap"
    exit 1
fi

# Update project state
!jq '.phase_2_planning_started = now' docs/ce-dps-state.json > docs/ce-dps-state.tmp && mv docs/ce-dps-state.tmp docs/ce-dps-state.json

# Update sprint tracking
!jq '.status = "ai_planning" | .planning_started = now' docs/sprints/sprint-001/sprint-info.json > docs/sprints/sprint-001/sprint-info.tmp && mv docs/sprints/sprint-001/sprint-info.tmp docs/sprints/sprint-001/sprint-info.json

!echo "✅ Feature selection validated. Triggering AI implementation planning..."
!echo "📋 Sprint features loaded from docs/phases/phase-2-sprint-planning.md"
!echo "🔍 Beginning detailed implementation analysis..."
</implementation>

### <constraints>
- Phase 2 must be set up first
- Feature selection must be completed
- Selected features must be specific (not template placeholders)
- jq command required for state management
</constraints>

## <claude-prompt>
I am executing CE-DPS Phase 2 implementation planning based on the selected sprint features.

### <sprint-context>
@docs/phases/phase-2-sprint-planning.md
@docs/phases/phase-1-planning.md

### <planning-requirements>
Based on the selected features above, provide comprehensive implementation planning in the following areas:

#### <feature-breakdown>
**Detailed Feature Analysis**:
- Break down each selected feature into specific implementation tasks
- Identify technical dependencies and prerequisites
- Define acceptance criteria and success metrics
- Estimate implementation complexity (1-10 scale)
- Calculate effort estimates in hours/days

**Requirements**: File-level task breakdown, realistic estimates, clear dependencies.

#### <technical-dependencies>
**Dependency Analysis**:
- Map dependencies between selected features
- Identify external system integrations required
- Define database schema changes needed
- Specify API contracts and interfaces
- Plan configuration and environment requirements

**Requirements**: Complete dependency mapping, integration planning, data architecture.

#### <implementation-approach>
**Technical Implementation Strategy**:
- Define implementation sequence and order
- Specify technology patterns and frameworks to use
- Plan database migrations and schema changes
- Design API endpoints and data models
- Outline testing approach for each feature

**Requirements**: Security-first patterns, proven approaches, comprehensive testing (>95% coverage).

#### <risk-assessment>
**Implementation Risk Analysis**:
- Identify technical risks for each feature
- Assess integration complexity and challenges
- Plan for performance and scalability concerns
- Define fallback strategies for high-risk items
- Estimate timeline risks and mitigation approaches

**Requirements**: Specific risk mitigation, contingency planning, realistic timeline buffers.

#### <effort-estimation>
**Resource Planning**:
- Provide detailed time estimates for each task
- Factor in testing, documentation, and review time
- Include buffer time for unexpected challenges
- Calculate total sprint duration
- Validate scope against typical sprint capacity

**Requirements**: Realistic estimates, comprehensive scope, sustainable pace.

### <output-format>
**Use CE-DPS LLM documentation patterns**:
- Apply semantic markup with `<feature-breakdown>`, `<technical-dependencies>`, etc.
- Use progressive disclosure for complex implementation details
- Include structured data for estimates and dependencies
- Provide token-efficient but comprehensive coverage
- Mark sections requiring human approval with `<human-approval-required>`

### <quality-requirements>
**CE-DPS Implementation Standards**:
- **Security-First**: Authentication, authorization, input validation in all features
- **Testing**: >95% coverage with unit, integration, and security tests
- **Performance**: Response times and scalability considerations
- **Documentation**: API documentation and code comments
- **Error Handling**: Comprehensive error management and logging
- **Integration**: Seamless integration with existing systems

### <human-approval-points>
**Strategic Decisions Requiring Human Approval**:
- Final feature selection and sprint scope
- Implementation approach and technology choices
- Database schema changes and migrations
- API design and public interface decisions
- Timeline estimates and resource allocation
- Quality gates and acceptance criteria

### <fortitude-integration>
**Knowledge Management**:
- Query existing implementation patterns for similar features
- Reference proven security and performance approaches
- Apply domain-specific development patterns
- Leverage previous sprint learnings and templates

### <deliverables>
**Phase 2 Planning Output**:
1. **Feature Implementation Plan**: Detailed task breakdown with file-level specificity
2. **Technical Architecture**: Component design and integration approach
3. **Sprint Backlog**: Prioritized tasks with effort estimates
4. **Risk Mitigation Plan**: Specific risks with actionable mitigation strategies
5. **Quality Gates**: Testing and validation criteria for each feature
6. **Human Approval Checklist**: Structured review process for implementation decisions

### <implementation-specificity>
**File-Level Implementation Planning**:
- Specify exact files to be created or modified
- Define database migration scripts needed
- List API endpoints to be implemented
- Identify test files and testing approaches
- Plan configuration changes required

**Example Level of Detail**:
```
Feature: User Authentication
├── Database Layer
│   ├── migrations/001_create_users_table.sql
│   ├── src/models/user.rs
│   └── tests/models/user_tests.rs
├── Business Logic
│   ├── src/auth/service.rs
│   ├── src/auth/jwt.rs
│   └── tests/auth/service_tests.rs
├── API Layer
│   ├── src/handlers/auth.rs
│   ├── src/middleware/auth.rs
│   └── tests/handlers/auth_integration_tests.rs
└── Quality Gates
    ├── Security: Input validation, password hashing
    ├── Performance: Token caching, session management
    └── Testing: Unit (90%), Integration (5%), Security (5%)
```

### <success-criteria>
**Phase 2 Planning Success**:
- All selected features have detailed implementation plans
- Technical approach is sound and uses proven patterns
- Effort estimates are realistic and include appropriate buffers
- Dependencies are clearly identified and manageable
- Quality gates ensure >95% test coverage and security standards
- Human approval points maintain strategic oversight

Please begin the comprehensive Phase 2 implementation planning now. When complete, provide structured human review sections for approval before proceeding to Phase 3.
</claude-prompt>

## <human-action-required>
**AI Implementation Planning In Progress... 🧠**

### <current-status>
Claude Code is now performing comprehensive Phase 2 implementation planning including:
- **Feature Breakdown**: Detailed task analysis with file-level specificity
- **Technical Dependencies**: Complete dependency mapping and integration planning
- **Implementation Approach**: Security-first patterns with proven technologies
- **Risk Assessment**: Specific risks with actionable mitigation strategies
- **Effort Estimation**: Realistic time estimates with appropriate buffers

### <what-to-expect>
**Claude Code will provide**:
1. **Detailed implementation plans** for each selected feature
2. **File-level task breakdown** with specific files to create/modify
3. **Technical architecture** with component relationships
4. **Resource estimates** with realistic timelines
5. **Risk mitigation strategies** for identified challenges
6. **Quality gates** ensuring >95% test coverage and security standards
7. **Human approval sections** for strategic implementation decisions

### <your-next-actions>
**When planning is complete, you will need to**:
1. **Review feature implementation plans** for technical soundness
2. **Validate effort estimates** against team capacity and timeline
3. **Assess technical approach** for alignment with organizational standards
4. **Approve sprint scope** or request adjustments
5. **Run `/cedps phase2 validate`** when ready to proceed to implementation

### <approval-process>
**You will approve/reject each section**:
- ✅ **Approved**: Implementation approach accepted, proceed as planned
- ❓ **Needs Adjustment**: Request changes to scope, timeline, or approach
- ❌ **Requires Revision**: Reject plan, request alternative implementation strategy

### <quality-standards>
**Validate that planning includes**:
- [ ] File-level implementation specificity
- [ ] Security patterns integrated throughout
- [ ] >95% test coverage planned
- [ ] Performance considerations addressed
- [ ] Integration approaches defined
- [ ] Realistic effort estimates with buffers
- [ ] Risk mitigation strategies actionable

### <sprint-scope-validation>
**Ensure sprint scope is**:
- [ ] Realistic for team capacity
- [ ] Delivers meaningful user value
- [ ] Has manageable technical risk
- [ ] Includes comprehensive testing
- [ ] Supports continuous integration
</human-action-required>

## <troubleshooting>
### <common-errors>
- **"Phase 2 not set up"**: Run `/cedps phase2 setup` first
- **"Feature selection not completed"**: Complete feature selection in phase-2-sprint-planning.md
- **"Template not completed"**: Replace template placeholders with actual feature selections
- **Planning too vague**: Request more specific implementation details
- **Scope too large**: Select fewer features for manageable sprint
</common-errors>

### <quality-validation>
**Phase 2 Planning Requirements**:
- [ ] Selected features have detailed implementation plans
- [ ] Technical approach uses security-first patterns
- [ ] File-level specificity provided for all tasks
- [ ] Effort estimates are realistic and include buffers
- [ ] Dependencies clearly identified and manageable
- [ ] Quality gates ensure comprehensive testing
- [ ] Human approval points maintain strategic oversight
- [ ] LLM-optimized documentation patterns used
</quality-validation>