# <context>CE-DPS Phase 3 Validation</context>

<meta>
  <title>CE-DPS Phase 3 Validation</title>
  <type>slash-command</type>
  <audience>ai_assistant</audience>
  <complexity>advanced</complexity>
  <updated>2025-07-17</updated>
  <scope>phase3-validation</scope>
  <phase>code-implementation</phase>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: Validate Phase 3 implementation completion and prepare for production deployment
- **Core Benefits**: Comprehensive quality validation, production readiness assessment, deployment preparation
- **Prerequisites**: Complete Phase 3 implementation with all quality gates
- **Output**: Production deployment authorization with comprehensive quality reports

## <instructions priority="high">Validation Process</instructions>

### <step-1>Validate Prerequisites</step-1>
**Prerequisites Validation**:
- Check docs/phases/phase-3-implementation.md exists
- Verify on sprint-001-implementation branch (git branch --show-current)
- Confirm implementation completion indicators in phase-3-implementation.md
- Check required human validation sections (Feature Testing, Business Value Assessment, Production Readiness)

### <step-2>Validate Human Approvals</step-2>
**Approval Verification**:
- Look for "✅ Approved" markers in docs/phases/phase-3-implementation.md
- Ensure no "❌ Requires Changes" sections remain
- **SKYNET mode**: Auto-inject approval markers if missing
- Validate all business validation sections are complete

### <step-3>Run Comprehensive Quality Gates</step-3>
**Quality Validation**:
- Execute cargo run --bin quality-gates -- --comprehensive-validation
- Run full test suite with cargo test --quiet
- Check test coverage using cargo-tarpaulin if available (must be >=95%)
- Validate no failing tests or quality issues

### <step-4>Execute Phase Validator Tool</step-4>
**Tool Validation**:
- Run python3 tools/phase-validator.py --phase 3 --file docs/phases/phase-3-implementation.md
- Validate phase completion criteria are met

### <step-5>Update Project State and Loop State</step-5>
**State Management**:
- Read current state file using Read tool
- Update docs/ce-dps-state.json using Edit tool:
  - Add 3 to phases_completed array
  - Set phase_3_completed: current timestamp (use `date -u +%Y-%m-%dT%H:%M:%SZ`)
  - Update last_updated: current timestamp
  - Set ready_for_production = true
- **Update docs/sprints/sprint-001/implementation/implementation-status.json**:
  - Set status to "completed"
  - Set implementation_completed timestamp
  - Set quality_gates_passed and human_validation_complete to true

**Loop State Update** (if SKYNET mode is enabled):
- Check if SKYNET environment variable is set to "true"
- If enabled, use the skynet-loop-manager.sh tool to update the loop state
- Mark phase3_validation_complete status
- Set next command to /quality-check
- Record completion in the loop state tracking

### <step-6>Generate Quality Report</step-6>
**Report Generation**:
- Run cargo run --bin quality-gates -- --generate-report --output docs/quality-reports/sprint-001/final-quality-report.json
- Document comprehensive quality metrics

### <step-7>Create Completion Documentation</step-7>
**Documentation Generation**:
- **Generate docs/phases/phase-3-completion-report.md** with:
  - Implementation summary and metrics
  - Features delivered with business value
  - Quality gates results
  - Human validation results
  - Production readiness assessment
- **Create docs/phases/phase-3-artifacts/production-deployment-checklist.md**

### <step-8>SKYNET Mode Auto-Transition</step-8>
**Autonomous Transition** (if SKYNET=true):
- Automatically run quality check and prepare next sprint loop
- Exit with special code (42) to trigger /quality-check
- Auto-transition to Phase 2 setup for continuous development
- Increment sprint number for next iteration
- Execute automatic command progression: /quality-check → /phase2:setup (next sprint)

## <expected-output priority="medium">Validation Results</expected-output>

**Command Output**:
- Prerequisites validation with specific error messages if issues found
- Human approval validation (or SKYNET auto-approval injection)
- Comprehensive quality gates execution results
- Test coverage validation and metrics
- Phase validator tool results
- Project state updates with completion timestamps
- Quality report generation
- Completion documentation creation
- Success confirmation with file locations
- SKYNET mode auto-transition to quality check if enabled

## <human-actions priority="high">Required Validation</human-actions>

**Normal Mode**:
- Phase 3 validation complete, ready for production deployment
- Review completion report and deployment checklist
- Execute production deployment when ready
- Consider next sprint planning

**SKYNET Mode**:
- Auto-transitions to comprehensive quality check (/quality-check)
- If quality gates pass, automatically loops to next sprint (/phase2:setup)
- Increments sprint number and continues development cycle indefinitely
- No human intervention unless quality issues detected
- Continuous loop: Phase3 → Quality Check → Phase2 → Phase3 → repeat

## <parameters priority="low">Command Configuration</parameters>
**Configuration Details**:
- No parameters required
- Checks for SKYNET environment variable for autonomous mode
- Uses jq for JSON state management
- Requires cargo, python3, and quality tools to be available