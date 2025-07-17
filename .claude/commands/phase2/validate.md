# CE-DPS Phase 2 Validation

Validate Phase 2 sprint planning completion and authorize transition to Phase 3 implementation.

## Instructions

1. **Validate Phase 2 Completion Requirements**
   - Confirm `docs/phases/phase-2-sprint-planning.md` exists and contains complete implementation planning
   - Verify feature selection section is completed with specific features (not template placeholders)
   - Check that AI implementation planning sections are populated with detailed analysis
   - Ensure all required approval sections exist in the document

2. **Verify Strategic Approvals**
   - Check for specific approval sections:
     - "Feature Selection Validation" with "✅ Approved" marker
     - "Implementation Approach Approval" with "✅ Approved" marker  
     - "Timeline and Resource Approval" with "✅ Approved" marker
     - "Sprint Approval" with "✅ Approved" marker
   - If SKYNET mode: auto-inject approval markers if missing with "✅ Approved - SKYNET: [reasoning]"
   - Validate no rejected sections (❌ Requires Revision) remain in document
   - Exit with error if any approvals missing in human oversight mode

3. **Run Phase Validation Tool**
   - If `tools/phase-validator.py` exists and python3 is available:
     - Run phase validation for Phase 2 with sprint planning document
     - Validate that all Phase 2 deliverables meet CE-DPS standards
     - Check implementation planning completeness and quality
     - Ensure strategic decisions are properly documented
   - If validation tool fails, exit with error about addressing issues

4. **Extract Sprint Backlog for Phase 3**
   - Create `docs/sprints/sprint-001/backlog/` directory
   - Extract "Sprint Backlog" section from planning document to `docs/sprints/sprint-001/backlog/sprint-backlog.md`
   - If extraction succeeds, announce sprint backlog preparation for Phase 3

5. **Validate Implementation Readiness**
   - Confirm all selected features have detailed file-level implementation plans
   - Verify technical dependencies are properly identified and sequenced  
   - Check that testing strategy covers all quality requirements (>95% coverage)
   - Ensure resource allocation and timeline are realistic for team capacity
   - Validate that quality gates are comprehensive and measurable

6. **Update Project State for Phase 3 Transition**
   - If jq available, update `docs/ce-dps-state.json` with:
     - Add 2 to phases_completed array
     - Set phase_2_completed timestamp
     - Set ready_for_phase_3 = true
     - Update current_sprint = 1
   - If jq not available, warn about manual state management
   - Update sprint tracking if `docs/sprints/sprint-001/sprint-info.json` exists:
     - Set status = "approved"
     - Set planning_completed timestamp
     - Set ready_for_implementation = true

7. **Generate Phase 2 Completion Report**
   - Create `docs/phases/phase-2-completion-report.md` with comprehensive summary including:
     - Completion status (Phase 2 - Sprint Planning, Complete)
     - Sprint 1 approved scope with selected features
     - Key decisions approved (feature priority, technical approach, timeline, quality gates)
     - Implementation readiness status with sprint backlog location
     - Quality metrics (>95% test coverage target, security-first patterns)
     - Files created during Phase 2
     - Ready for Phase 3 confirmation

8. **Handle SKYNET Auto-Transition**
   - If SKYNET mode enabled:
     - Announce automatic progression to Phase 3 setup
     - Auto-inject any missing approval markers with best-practice reasoning
     - Display autonomous transition messaging
     - Note about continuing to Phase 3 implementation setup
   - If not SKYNET mode: provide clear guidance for manual Phase 3 initiation

9. **Validate Sprint Environment Readiness**
   - Confirm all Phase 2 artifacts are properly documented and approved
   - Verify project state correctly reflects Phase 2 completion
   - Check that implementation foundation is complete for development
   - Ensure human approval authority is maintained for strategic decisions

## Expected Behavior

Perform strict validation of Phase 2 completion with comprehensive approval verification. Generate detailed completion documentation and prepare sprint environment. Handle both human oversight mode (requiring explicit approvals) and SKYNET autonomous mode (auto-approving with best-practice reasoning). Only authorize Phase 3 transition when all validation criteria are satisfied and sprint backlog is ready.