# <context>CE-DPS Phase 2 Validation</context>

<meta>
  <title>CE-DPS Phase 2 Validation</title>
  <type>slash-command</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-17</updated>
  <scope>phase2-validation</scope>
  <phase>sprint-development</phase>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: Validate Phase 2 sprint planning completion and authorize transition to Phase 3 implementation
- **Core Benefits**: Comprehensive validation, sprint backlog extraction, completion reporting, phase transition
- **Prerequisites**: Complete Phase 2 planning with all approvals
- **Output**: Phase 2 completion report and authorization for Phase 3 implementation

## <instructions priority="high">Validation Process</instructions>

### <step-1>Validate Phase 2 Completion Requirements</step-1>
**Completion Validation**:
- Confirm docs/phases/phase-2-sprint-planning.md exists and contains complete implementation planning
- Verify feature selection section completed with specific features (not template placeholders)
- Check AI implementation planning sections populated with detailed analysis
- Ensure all required approval sections exist in the document

### <step-2>Verify Strategic Approvals</step-2>
**Approval Verification**:
- Check for specific approval sections:
  - "Feature Selection Validation" with "✅ Approved" marker
  - "Implementation Approach Approval" with "✅ Approved" marker
  - "Timeline and Resource Approval" with "✅ Approved" marker
  - "Sprint Approval" with "✅ Approved" marker
- **SKYNET mode**: Auto-inject approval markers if missing with "✅ Approved - SKYNET: [reasoning]"
- Validate no rejected sections (❌ Requires Revision) remain in document
- Exit with error if any approvals missing in human oversight mode

### <step-3>Run Phase Validation Tool</step-3>
**Tool Validation**:
- If tools/phase-validator.py exists and python3 available:
  - Run phase validation for Phase 2 with sprint planning document
  - Validate all Phase 2 deliverables meet CE-DPS standards
  - Check implementation planning completeness and quality
  - Ensure strategic decisions are properly documented
- If validation tool fails, exit with error about addressing issues

### <step-4>Extract Sprint Backlog for Phase 3</step-4>
**Backlog Preparation**:
- Create docs/sprints/sprint-001/backlog/ directory
- Extract "Sprint Backlog" section from planning document to docs/sprints/sprint-001/backlog/sprint-backlog.md
- If extraction succeeds, announce sprint backlog preparation for Phase 3

### <step-5>Validate Implementation Readiness</step-5>
**Readiness Validation**:
- Confirm all selected features have detailed file-level implementation plans
- Verify technical dependencies properly identified and sequenced
- Check testing strategy covers all quality requirements (>95% coverage)
- Ensure resource allocation and timeline are realistic for team capacity
- Validate quality gates are comprehensive and measurable

### <step-6>Update Project State for Phase 3 Transition</step-6>
**State Management**:
- Read current state file using Read tool
- Update specific fields using Edit tool:
  - Add 2 to phases_completed array
  - Set phase_2_completed: current timestamp (use `date -u +%Y-%m-%dT%H:%M:%SZ`)
  - Set ready_for_phase_3: true
  - Update current_sprint: 1
  - Update last_updated: current timestamp
- Validate update was successful by reading the file again
- If update fails, provide clear error message and manual steps
- **Update sprint tracking** (docs/sprints/sprint-001/sprint-info.json):
  - Set status = "approved"
  - Set planning_completed timestamp
  - Set ready_for_implementation = true

### <step-7>Generate Phase 2 Completion Report</step-7>
**Report Generation** (docs/phases/phase-2-completion-report.md):
- Completion status (Phase 2 - Sprint Planning, Complete)
- Sprint 1 approved scope with selected features
- Key decisions approved (feature priority, technical approach, timeline, quality gates)
- Implementation readiness status with sprint backlog location
- Quality metrics (>95% test coverage target, security-first patterns)
- Files created during Phase 2
- Ready for Phase 3 confirmation

### <step-8>Handle SKYNET Auto-Transition</step-8>
**Transition Management**:
- **SKYNET mode** (if enabled):
  - Execute automatic progression to Phase 3 setup (/phase3:setup)
  - Auto-inject any missing approval markers with best-practice reasoning
  - Display autonomous transition messaging
  - Note about continuing to Phase 3 implementation setup
- **Non-SKYNET mode**: Provide clear guidance for manual Phase 3 initiation

### <step-9>Validate Sprint Environment Readiness</step-9>
**Environment Validation**:
- Confirm all Phase 2 artifacts properly documented and approved
- Verify project state correctly reflects Phase 2 completion
- Check implementation foundation is complete for development
- Ensure human approval authority maintained for strategic decisions

## <expected-behavior priority="medium">Validation Output</expected-behavior>

**Validation Requirements**:
- Perform strict validation of Phase 2 completion with comprehensive approval verification
- Generate detailed completion documentation and prepare sprint environment
- Handle both human oversight mode (requiring explicit approvals) and SKYNET autonomous mode (auto-approving with best-practice reasoning)
- Only authorize Phase 3 transition when all validation criteria are satisfied and sprint backlog is ready