# <context>CE-DPS Phase 1 Validation</context>

<meta>
  <title>CE-DPS Phase 1 Validation</title>
  <type>slash-command</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-17</updated>
  <scope>phase1-validation</scope>
  <phase>strategic-planning</phase>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: Validate Phase 1 strategic planning completion and authorize transition to Phase 2 sprint planning
- **Core Benefits**: Comprehensive validation, completion reporting, state management, phase transition
- **Prerequisites**: Complete Phase 1 analysis with all approvals
- **Output**: Phase 1 completion report and authorization for Phase 2 transition

## <instructions priority="high">Validation Process</instructions>

### <step-1>Validate Phase 1 Completion Requirements</step-1>
**Completion Validation**:
- Confirm docs/phases/phase-1-planning.md exists and contains complete analysis
- Verify all business requirements sections filled out by human or SKYNET
- Check AI analysis sections populated with architectural recommendations
- Ensure all required approval sections exist in the document

### <step-2>Verify Strategic Approvals</step-2>
**Approval Verification**:
- Check Architecture Approval section with "✅ Approved" marker
- Validate Feature Roadmap Approval section is approved
- Confirm Risk Acceptance section shows approval
- Verify Final Phase 1 Sign-off is complete
- **SKYNET mode**: Auto-inject approval markers if missing with "✅ Approved - SKYNET: [best practice reasoning]"

### <step-3>Run Phase Validation Tool</step-3>
**Tool Validation**:
- If tools/phase-validator.py exists and python3 available, run phase validation
- Validate all Phase 1 deliverables meet CE-DPS standards
- Check documentation completeness and quality
- Ensure strategic decisions are properly documented

### <step-4>Validate Technical Foundation</step-4>
**Technical Validation**:
- Confirm proposed architecture addresses all stated business requirements
- Verify technology stack choices align with technical constraints
- Check feature roadmap prioritization matches business objectives
- Ensure risk mitigation strategies are actionable and comprehensive

### <step-5>Generate Phase 1 Completion Report</step-5>
**Report Generation** (docs/phases/phase-1-completion-report.md):
- Document approved architecture approach and key design decisions
- Record final technology stack selections with rationale
- Include complete feature roadmap with business value priorities
- List all identified risks with approved mitigation strategies
- Add completion metrics and quality validation results

### <step-6>Update Project State for Phase 2 Transition</step-6>
**State Management** (docs/ce-dps-state.json):
- **If jq available**, update with:
  - Add 1 to phases_completed array
  - Set phase_1_completed timestamp
  - Set ready_for_phase_2 = true
  - Update current_phase to 2
  - Update last_updated timestamp
- **If jq not available**: warn about manual state management

### <step-7>Handle SKYNET Auto-Transition</step-7>
**Transition Management**:
- **SKYNET mode**: Announce automatic progression to Phase 2 setup
- Auto-inject any missing approval markers with best-practice reasoning
- Display autonomous transition messaging
- **Non-SKYNET mode**: Provide clear guidance for manual Phase 2 initiation

### <step-8>Validate Phase 2 Readiness</step-8>
**Readiness Validation**:
- Confirm all Phase 1 artifacts properly documented and approved
- Verify project state correctly reflects Phase 1 completion
- Check strategic foundation is complete for sprint planning
- Ensure human approval authority maintained for strategic decisions

## <expected-behavior priority="medium">Validation Output</expected-behavior>

**Validation Requirements**:
- Perform strict validation of Phase 1 completion with comprehensive approval verification
- Generate detailed completion documentation and update project state
- Handle both human oversight mode (requiring explicit approvals) and SKYNET autonomous mode (auto-approving with best-practice reasoning)
- Only authorize Phase 2 transition when all validation criteria are satisfied