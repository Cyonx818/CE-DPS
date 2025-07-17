# <context>CE-DPS Phase 1 Analysis</context>

<meta>
  <title>CE-DPS Phase 1 Strategic Analysis</title>
  <type>slash-command</type>
  <audience>ai_assistant</audience>
  <complexity>advanced</complexity>
  <updated>2025-07-17</updated>
  <scope>phase1-analysis</scope>
  <phase>strategic-planning</phase>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: Comprehensive AI analysis of business requirements with architectural research and strategic planning
- **Core Benefits**: Architecture design, technology evaluation, implementation roadmap, risk assessment
- **Prerequisites**: Completed business requirements in docs/phases/phase-1-planning.md
- **Output**: Strategic foundation with human approval points or SKYNET auto-approval

## <instructions priority="high">Strategic Analysis Process</instructions>

**Context**: Comprehensive Phase 1 strategic analysis based on completed business requirements in docs/phases/phase-1-planning.md.

### <step-1>Validate Analysis Prerequisites</step-1>
**Prerequisites Validation**:
- Confirm docs/phases/phase-1-planning.md exists from Phase 1 setup
- If not SKYNET mode, verify all template placeholders replaced with actual content
- Check business requirements sections are complete
- Update project state to mark analysis started

### <step-2>Read and Parse Business Requirements</step-2>
**Requirements Analysis**:
- Load the business requirements document
- Extract problem statement, target users, and success metrics
- Analyze technical requirements and performance constraints
- Identify integration needs and security requirements
- Note budget and timeline constraints

### <step-3>Query Fortitude Knowledge Base</step-3>
**Knowledge Research**:
- Search for existing architectural patterns matching the problem domain
- Research proven security and performance patterns
- Look up integration approaches for similar systems
- Find relevant technology stack recommendations from past projects

### <step-4>Perform Comprehensive Architectural Analysis</step-4>
**Architecture Design**:
- Design system architecture with security-first approach
- Propose component relationships and data flow
- Address scalability requirements from business needs
- Plan integration points with existing systems
- Include authentication and authorization strategies

### <step-5>Evaluate Technology Stack Options</step-5>
**Technology Evaluation**:
- Research and compare technology alternatives for each layer
- Provide specific recommendations with detailed rationale
- Consider team constraints and organizational preferences
- Address deployment and infrastructure needs
- Include development tooling and testing frameworks

### <step-6>Create Implementation Roadmap</step-6>
**Roadmap Development**:
- Break features into development phases prioritized by business value
- Estimate complexity and effort for each major component
- Identify critical path dependencies
- Plan MVP and subsequent iterations
- Define testing and quality assurance approach (>95% coverage target)

### <step-7>Conduct Risk Assessment</step-7>
**Risk Analysis**:
- Identify technical risks (performance, scalability, integration)
- Assess business risks (timeline, resource, market)
- Evaluate operational risks (deployment, maintenance, security)
- Provide specific mitigation strategies for each identified risk
- Create contingency plans for critical failure scenarios

### <step-8>Generate Human Approval Sections</step-8>
**Approval Framework**:
- Create structured approval sections for architecture decisions
- Include technology stack approval with alternatives
- Add feature roadmap approval with prioritization rationale
- Include risk assessment approval with mitigation strategies
- Mark sections clearly for human strategic review

### <step-9>Handle SKYNET Auto-Approval</step-9>
**SKYNET Mode** (if enabled):
- Automatically populate approval sections with best-practice decisions
- Mark approvals as "✅ Approved - SKYNET: [reasoning based on best practices]"
- Announce automatic progression to validation phase
**Non-SKYNET Mode**: Provide clear instructions for human review process

### <step-10>Update Planning Document</step-10>
**Document Completion**:
- Fill all AI analysis sections with comprehensive details
- Include architecture diagrams and component descriptions
- Provide technology evaluation matrices and decision rationale
- Add implementation timeline with realistic estimates
- Include complete risk register with mitigation plans

## <expected-behavior priority="medium">Strategic Analysis Output</expected-behavior>

**Analysis Requirements**:
- Execute comprehensive strategic analysis covering architecture, technology, implementation planning, and risk management
- Provide complete foundation for Phase 2 sprint planning with either human approval points (normal mode) or auto-approved decisions (SKYNET mode)
- All recommendations must be based on business requirements and include detailed rationale