# <context>CE-DPS Phase 3 Implementation</context>

<meta>
  <title>CE-DPS Phase 3 Implementation</title>
  <type>slash-command</type>
  <audience>ai_assistant</audience>
  <complexity>advanced</complexity>
  <updated>2025-07-17</updated>
  <scope>phase3-implementation</scope>
  <phase>code-implementation</phase>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: Trigger Phase 3 AI implementation with test-driven development and quality gates
- **Core Benefits**: TDD implementation cycle, security-first patterns, comprehensive quality validation
- **Prerequisites**: Completed Phase 2 with implementation backlog and sprint-001-implementation branch
- **Output**: Production-ready code with >95% test coverage and comprehensive validation

## <instructions priority="high">Implementation Execution Process</instructions>

### <step-1>Validate Implementation Setup</step-1>
**Setup Validation**:
- Check docs/phases/phase-3-implementation.md exists (run /phase3:setup if missing)
- Verify docs/phases/phase-3-artifacts/implementation-backlog.md exists from Phase 2
- Confirm on sprint-001-implementation branch (git branch --show-current)
- Validate quality gates (cargo run --bin quality-gates -- --validate-environment)

### <step-2>Update Project State</step-2>
**State Management**:
- Read current state file using Read tool
- Update docs/ce-dps-state.json using Edit tool:
  - phase_3_implementation_started: current timestamp (use `date -u +%Y-%m-%dT%H:%M:%SZ`)
  - last_updated: current timestamp
- Update docs/sprints/sprint-001/implementation/implementation-status.json using Edit tool:
  - status: "implementing"

### <step-3>Trigger Claude Code Implementation</step-3>
**Context Loading**:
- Load implementation backlog: @docs/phases/phase-3-artifacts/implementation-backlog.md
- Load Phase 1 planning: @docs/phases/phase-1-planning.md
- Load Phase 2 sprint planning: @docs/phases/phase-2-sprint-planning.md

**TDD Implementation Cycle**:
- Write failing tests first (unit, integration, security, performance)
- Implement minimal code to pass tests
- Refactor for quality while maintaining test coverage
- Target >95% test coverage for all business logic

**Sequential Implementation Approach**:
- **Database layer**: migrations, models, repository patterns
- **Business logic**: core functionality with error handling
- **API layer**: endpoints with validation and authentication
- **Integration layer**: external system connections
- **Quality validation**: comprehensive testing and security

**Security-First Patterns**:
- JWT authentication with proper expiration
- Role-based authorization at all endpoints
- Comprehensive input validation and sanitization
- SQL injection prevention with parameterized queries
- Rate limiting and error handling without data leakage

**Quality Gates Integration**:
- Run tests after each significant change
- Comprehensive validation before moving to next feature
- Pre-commit validation before human review
- Performance requirements (<200ms response time)
- Security vulnerability scanning

**Human Validation Points**:
- Provide demo environment after each feature
- Document business value delivered
- Request human validation against requirements
- Address feedback before proceeding (bypass in SKYNET mode)

**Anchor Test Creation**:
- Create permanent regression tests for critical functionality
- Mark with ANCHOR: comments explaining importance
- Cover external APIs, data persistence, auth flows, core business logic

**Error Handling Requirements**:
- Use structured error types (thiserror crate pattern)
- Comprehensive error propagation and context
- User-friendly error responses without sensitive data leakage

### <step-4>Implementation Workflow</step-4>
**Workflow Steps**:
- Environment preparation and validation
- Feature implementation loop with TDD
- Integration validation across features
- Human business validation (or SKYNET auto-approval)
- Quality gate finalization

### <step-5>Fortitude Integration</step-5>
**Knowledge Management**:
- Query existing implementation patterns before creating new ones
- Apply proven security and performance patterns
- Document new patterns discovered during implementation
- Update knowledge base with successful approaches

## <expected-output priority="medium">Implementation Results</expected-output>

**Claude Code Execution**:
- Validate environment and load context from implementation backlog
- Execute systematic TDD implementation of all approved features
- Apply security-first patterns throughout implementation
- Run quality gates after each feature completion
- Request human validation for business value (unless SKYNET mode)
- Generate comprehensive documentation
- Update implementation tracking and project state

## <human-actions priority="high">Required Validation</human-actions>

**Normal Mode**:
- Claude will request validation after each feature implementation
- Review demo environment and validate business value
- Approve features before Claude proceeds to next one
- Provide feedback if changes needed

**SKYNET Mode**:
- Auto-approves all business validations
- Continues implementation autonomously
- Only stops for technical failures or quality gate issues

## <parameters priority="low">Command Configuration</parameters>
**Configuration Details**:
- No parameters required
- Checks for SKYNET environment variable for autonomous validation
- Uses jq for state management
- Requires implementation backlog from Phase 2