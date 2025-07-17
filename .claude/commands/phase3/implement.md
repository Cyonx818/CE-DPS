# CE-DPS Phase 3 Implementation

Trigger Phase 3 AI implementation with test-driven development and quality gates.

## Instructions

1. **Validate Implementation Setup**
   - Check that docs/phases/phase-3-implementation.md exists (run /cedps-phase3-setup if missing)
   - Verify docs/phases/phase-3-artifacts/implementation-backlog.md exists from Phase 2
   - Confirm on sprint-001-implementation branch using git branch --show-current
   - Validate quality gates using cargo run --bin quality-gates -- --validate-environment

2. **Update Project State**
   - Use jq to update docs/ce-dps-state.json with phase_3_implementation_started timestamp
   - Update docs/sprints/sprint-001/implementation/implementation-status.json status to "implementing"

3. **Trigger Claude Code Implementation**
   This command serves as a comprehensive prompt to Claude Code to execute:
   
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
   - Database layer: migrations, models, repository patterns
   - Business logic: core functionality with error handling
   - API layer: endpoints with validation and authentication
   - Integration layer: external system connections
   - Quality validation: comprehensive testing and security

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

4. **Implementation Workflow**:
   - Environment preparation and validation
   - Feature implementation loop with TDD
   - Integration validation across features
   - Human business validation (or SKYNET auto-approval)
   - Quality gate finalization

5. **Fortitude Integration**:
   - Query existing implementation patterns before creating new ones
   - Apply proven security and performance patterns
   - Document new patterns discovered during implementation
   - Update knowledge base with successful approaches

## Expected Output

This command triggers Claude Code to begin comprehensive implementation. Claude will:
- Validate environment and load context from implementation backlog
- Execute systematic TDD implementation of all approved features
- Apply security-first patterns throughout implementation
- Run quality gates after each feature completion
- Request human validation for business value (unless SKYNET mode)
- Generate comprehensive documentation
- Update implementation tracking and project state

## Human Action Required

In normal mode:
- Claude will request validation after each feature implementation
- Review demo environment and validate business value
- Approve features before Claude proceeds to next one
- Provide feedback if changes needed

In SKYNET mode:
- Auto-approves all business validations
- Continues implementation autonomously
- Only stops for technical failures or quality gate issues

## Parameters
- No parameters required
- Checks for SKYNET environment variable for autonomous validation
- Uses jq for state management
- Requires implementation backlog from Phase 2