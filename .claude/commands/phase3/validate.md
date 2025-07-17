# CE-DPS Phase 3 Validation

Validate Phase 3 implementation completion and prepare for production deployment.

## Instructions

1. **Validate Prerequisites**
   - Check that docs/phases/phase-3-implementation.md exists
   - Verify on sprint-001-implementation branch using git branch --show-current
   - Confirm implementation completion indicators in phase-3-implementation.md
   - Check for required human validation sections (Feature Testing, Business Value Assessment, Production Readiness)

2. **Validate Human Approvals**
   - Look for "✅ Approved" markers in docs/phases/phase-3-implementation.md
   - Ensure no "❌ Requires Changes" sections remain
   - In SKYNET mode: auto-inject approval markers if missing
   - Validate all business validation sections are complete

3. **Run Comprehensive Quality Gates**
   - Execute cargo run --bin quality-gates -- --comprehensive-validation
   - Run full test suite with cargo test --quiet
   - Check test coverage using cargo-tarpaulin if available (must be >=95%)
   - Validate no failing tests or quality issues

4. **Execute Phase Validator Tool**
   - Run python3 tools/phase-validator.py --phase 3 --file docs/phases/phase-3-implementation.md
   - Validate phase completion criteria are met

5. **Update Project State**
   - Use jq to update docs/ce-dps-state.json:
     - Add 3 to phases_completed array
     - Set phase_3_completed timestamp
     - Set ready_for_production = true
   - Update docs/sprints/sprint-001/implementation/implementation-status.json:
     - Set status to "completed"
     - Set implementation_completed timestamp
     - Set quality_gates_passed and human_validation_complete to true

6. **Generate Quality Report**
   - Run cargo run --bin quality-gates -- --generate-report --output docs/quality-reports/sprint-001/final-quality-report.json
   - Document comprehensive quality metrics

7. **Create Completion Documentation**
   - Generate docs/phases/phase-3-completion-report.md with:
     - Implementation summary and metrics
     - Features delivered with business value
     - Quality gates results
     - Human validation results
     - Production readiness assessment
   - Create docs/phases/phase-3-artifacts/production-deployment-checklist.md

8. **SKYNET Mode Auto-Transition**
   - If SKYNET=true, automatically run quality check and prepare next sprint loop
   - Exit with special code (42) to trigger /cedps-quality-check
   - Auto-transition to Phase 2 setup for continuous development

## Expected Output

Output will show:
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

## Human Action Required

In normal mode:
- Phase 3 validation complete, ready for production deployment
- Review completion report and deployment checklist
- Execute production deployment when ready
- Consider next sprint planning

In SKYNET mode:
- Auto-transitions to comprehensive quality check
- If quality gates pass, automatically loops to next sprint
- No human intervention unless quality issues detected

## Parameters
- No parameters required
- Checks for SKYNET environment variable for autonomous mode
- Uses jq for JSON state management
- Requires cargo, python3, and quality tools to be available