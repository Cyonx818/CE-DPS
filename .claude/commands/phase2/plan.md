# <context>CE-DPS Phase 2 Implementation Planning</context>

<meta>
  <title>CE-DPS Phase 2 Implementation Planning</title>
  <type>slash-command</type>
  <audience>ai_assistant</audience>
  <complexity>advanced</complexity>
  <updated>2025-07-17</updated>
  <scope>phase2-planning</scope>
  <phase>sprint-development</phase>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: Comprehensive AI implementation planning for selected sprint features with detailed task breakdown
- **Core Benefits**: Feature analysis, technical dependencies, risk assessment, file-level implementation plans
- **Prerequisites**: Completed feature selection in docs/phases/phase-2-sprint-planning.md
- **Output**: Detailed implementation strategy with human approval points or SKYNET auto-approval

## <instructions priority="high">Implementation Planning Process</instructions>

**Context**: Execute comprehensive Phase 2 implementation planning based on completed feature selection in docs/phases/phase-2-sprint-planning.md.

### <step-1>Validate Prerequisites and Setup</step-1>
**Prerequisites Validation**:
- Confirm docs/phases/phase-2-sprint-planning.md exists from Phase 2 setup
- Validate feature selection section completed (not template placeholders)
- Check "Selected Features for Sprint" section contains specific features
- If template placeholders like "[Choose features]" remain, exit with error
- Update project state: mark phase_2_planning_started
- Update sprint tracking: status to "ai_planning" with planning_started timestamp

### <step-2>Load and Parse Sprint Context</step-2>
**Context Loading**:
- Read sprint planning document to extract selected features
- Load Phase 1 planning document for architectural context
- Parse business priority input and sprint goals
- Extract feature dependencies and complexity notes
- Validate 2-4 features selected (appropriate sprint scope)

### <step-3>Perform Comprehensive Feature Analysis</step-3>
**Feature Breakdown**:
- Break down each selected feature into specific implementation tasks
- Identify technical dependencies (authentication → API → database → dashboard)
- Define acceptance criteria and success metrics for each feature
- Estimate implementation complexity on 1-10 scale
- Calculate effort estimates in hours/days with realistic buffers

### <step-4>Conduct Fortitude Knowledge Research</step-4>
**Knowledge Base Integration**:
- Query Fortitude for similar feature implementations
- Research security and performance patterns for selected feature types
- Look up proven approaches (authentication, API design, database integration)
- Reference domain-specific development patterns and best practices
- Apply previous sprint learnings and successful implementation templates

### <step-5>Create Detailed Implementation Strategy</step-5>
**Strategy Development**:
- Define implementation sequence based on feature dependencies
- Specify technology patterns and frameworks (JWT auth, REST APIs, ORM)
- Plan database migrations and schema changes with proper indexing
- Design API endpoints and data models with comprehensive validation
- Outline security-first testing approach (>95% coverage target)

### <step-6>Generate File-Level Implementation Plan</step-6>
**File-Level Specificity** (Example: User Authentication):
- **Database**: migrations/001_create_users_table.sql, src/models/user.rs
- **Business Logic**: src/auth/service.rs, src/auth/jwt.rs
- **API Layer**: src/handlers/auth.rs, src/middleware/auth.rs
- **Tests**: tests/auth/service_tests.rs, tests/handlers/auth_integration_tests.rs
- Define API contracts and public interface decisions
- Plan configuration and environment requirements

### <step-7>Design Comprehensive Testing Strategy</step-7>
**Testing Approach**:
- Plan unit tests for all business logic with edge cases and error conditions
- Design integration tests for API endpoints with database interaction
- Create security test scenarios for input validation and authorization
- Plan performance tests for critical authentication and API paths
- Define anchor tests for permanent regression protection

### <step-8>Conduct Implementation Risk Assessment</step-8>
**Risk Analysis**:
- Identify technical risks (performance, complexity, integration)
- Assess business risks (timeline, resource, market timing)
- Evaluate operational risks (deployment, maintenance, security)
- Define specific mitigation strategies for each identified risk
- Create contingency plans for high-risk items with fallback approaches

### <step-9>Create Resource Planning and Timeline</step-9>
**Resource Estimation**:
- Provide detailed time estimates for each task with justification
- Factor in testing time (unit, integration, security), documentation, and review
- Include buffer time for unexpected challenges (typically 20-30%)
- Calculate total sprint duration and validate against team capacity
- Plan quality gate checkpoints throughout sprint

### <step-10>Generate Human Approval Sections</step-10>
**Approval Framework**:
- Create structured approval sections for strategic implementation decisions:
  - Feature Selection Validation
  - Implementation Approach Approval
  - Timeline and Resource Approval
  - Sprint Approval
- Include technology stack decisions requiring human validation
- Mark sections clearly for human strategic review with approval checkboxes
- Provide rationale and alternatives for each major decision

### <step-11>Handle SKYNET Auto-Approval</step-11>
**SKYNET Mode** (if enabled):
- Auto-populate all approval sections with "✅ Approved - SKYNET: [reasoning]"
- Include best-practice reasoning for each approval decision
- Mark document as ready for immediate validation
- Announce automatic progression to Phase 2 validation
**Non-SKYNET Mode**: Provide clear instructions for human review process

### <step-12>Update Planning Document</step-12>
**Document Completion**:
- Fill all AI analysis sections with comprehensive implementation details
- Include file-level specificity for all planned work
- Add technology evaluation with rationale for choices
- Include complete resource estimates and realistic timeline
- Add comprehensive risk register with specific mitigation plans
- Mark document ready for human review (or auto-approved if SKYNET)

## <expected-behavior priority="medium">Strategic Analysis Output</expected-behavior>

**Analysis Requirements**:
- Execute comprehensive strategic analysis covering feature breakdown, technical dependencies, implementation approach, risk assessment, and effort estimation
- Generate complete foundation for Phase 3 implementation with either human approval points (normal mode) or auto-approved decisions (SKYNET mode)
- All recommendations must include file-level specificity and detailed rationale