# CE-DPS Phase 2 Implementation Planning

Trigger comprehensive AI implementation planning for selected sprint features with detailed task breakdown.

## Instructions

Execute comprehensive Phase 2 implementation planning based on the completed feature selection in `docs/phases/phase-2-sprint-planning.md`.

1. **Validate Prerequisites and Setup**
   - Confirm `docs/phases/phase-2-sprint-planning.md` exists from Phase 2 setup
   - Validate that feature selection section is completed (not template placeholders)
   - Check that "Selected Features for Sprint" section contains specific features
   - If template placeholders like "[Choose features]" remain, exit with error about completing feature selection
   - Update project state to mark phase_2_planning_started
   - Update sprint tracking to set status to "ai_planning" with planning_started timestamp

2. **Load and Parse Sprint Context**
   - Read the sprint planning document to extract selected features
   - Load Phase 1 planning document for architectural context
   - Parse business priority input and sprint goals
   - Extract feature dependencies and complexity notes
   - Validate that 2-4 features are selected (appropriate sprint scope)

3. **Perform Comprehensive Feature Analysis**
   - Break down each selected feature into specific implementation tasks
   - Identify technical dependencies between features (authentication → API → database → dashboard)
   - Define acceptance criteria and success metrics for each feature
   - Estimate implementation complexity on 1-10 scale for each feature
   - Calculate effort estimates in hours/days with realistic buffers

4. **Conduct Fortitude Knowledge Research**
   - Query Fortitude knowledge base for similar feature implementations
   - Research security and performance patterns for selected feature types
   - Look up proven approaches for authentication, API design, database integration
   - Reference domain-specific development patterns and best practices
   - Apply previous sprint learnings and successful implementation templates

5. **Create Detailed Implementation Strategy**
   - Define implementation sequence based on feature dependencies
   - Specify technology patterns and frameworks to use (JWT auth, REST APIs, ORM)
   - Plan database migrations and schema changes with proper indexing
   - Design API endpoints and data models with comprehensive validation
   - Outline security-first testing approach for each feature (>95% coverage target)

6. **Generate File-Level Implementation Plan**
   - Break down features into specific files to be created/modified
   - Example detail level:
     - Feature: User Authentication
       - Database: migrations/001_create_users_table.sql, src/models/user.rs
       - Business Logic: src/auth/service.rs, src/auth/jwt.rs
       - API Layer: src/handlers/auth.rs, src/middleware/auth.rs
       - Tests: tests/auth/service_tests.rs, tests/handlers/auth_integration_tests.rs
   - Define API contracts and public interface decisions
   - Plan configuration and environment requirements

7. **Design Comprehensive Testing Strategy**
   - Plan unit tests for all business logic with edge cases and error conditions
   - Design integration tests for API endpoints with database interaction
   - Create security test scenarios for input validation and authorization
   - Plan performance tests for critical authentication and API paths
   - Define anchor tests for permanent regression protection

8. **Conduct Implementation Risk Assessment**
   - Identify technical risks for each feature (performance, complexity, integration)
   - Assess business risks (timeline, resource, market timing)
   - Evaluate operational risks (deployment, maintenance, security)
   - Define specific mitigation strategies for each identified risk
   - Create contingency plans for high-risk items with fallback approaches

9. **Create Resource Planning and Timeline**
   - Provide detailed time estimates for each task with justification
   - Factor in testing time (unit, integration, security), documentation, and review
   - Include buffer time for unexpected challenges (typically 20-30%)
   - Calculate total sprint duration and validate against team capacity
   - Plan quality gate checkpoints throughout sprint

10. **Generate Human Approval Sections**
    - Create structured approval sections for strategic implementation decisions:
      - Feature Selection Validation
      - Implementation Approach Approval  
      - Timeline and Resource Approval
      - Sprint Approval
    - Include technology stack decisions requiring human validation
    - Mark sections clearly for human strategic review with approval checkboxes
    - Provide rationale and alternatives for each major decision

11. **Handle SKYNET Auto-Approval**
    - If SKYNET mode is enabled:
      - Auto-populate all approval sections with "✅ Approved - SKYNET: [reasoning]"
      - Include best-practice reasoning for each approval decision
      - Mark document as ready for immediate validation
      - Announce automatic progression to Phase 2 validation
    - If not SKYNET mode: provide clear instructions for human review process

12. **Update Planning Document**
    - Fill all AI analysis sections with comprehensive implementation details
    - Include file-level specificity for all planned work
    - Add technology evaluation with rationale for choices
    - Include complete resource estimates and realistic timeline
    - Add comprehensive risk register with specific mitigation plans
    - Mark document ready for human review (or auto-approved if SKYNET)

## Expected Behavior

Execute comprehensive strategic analysis covering feature breakdown, technical dependencies, implementation approach, risk assessment, and effort estimation. Generate complete foundation for Phase 3 implementation with either human approval points (normal mode) or auto-approved decisions (SKYNET mode). All recommendations must include file-level specificity and detailed rationale.