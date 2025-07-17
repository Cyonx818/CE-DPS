# CE-DPS Phase 1 Setup

Initialize Phase 1 strategic planning environment and business requirements template.

## Instructions

1. **Validate Project Prerequisites**
   - Verify that `docs/ce-dps-state.json` exists (if not, inform user to run `/cedps-init` first)
   - Check if `docs/phases/phase-1-planning.md` already exists and inform user if Phase 1 is already initialized
   - Confirm project is in CE-DPS root directory with methodology structure

2. **Configure Phase 1 Environment**
   - Set environment variable CE_DPS_PHASE=1
   - Set CE_DPS_FORTITUDE_ENABLED=true for knowledge management
   - Set CE_DPS_QUALITY_GATES=true for validation
   - If SKYNET environment variable is true, set CE_DPS_HUMAN_APPROVAL_REQUIRED=false, otherwise set it to true
   - Display appropriate mode message (SKYNET autonomous or human oversight)

3. **Deploy Business Requirements Template**
   - Copy `methodology/templates/phase-1-template.md` to `docs/phases/phase-1-planning.md`
   - If copy fails, provide error message about template location
   - Create `docs/phases/phase-1-artifacts/` working directory

4. **Handle SKYNET Auto-Population**
   - If SKYNET environment variable is true:
     - Auto-populate the template with comprehensive business requirements focused on AI-assisted development
     - Include problem statement about accelerating development with quality
     - Add target users (development teams, product managers, QA teams)
     - Set success metrics (>50% faster delivery, >95% test coverage, zero critical vulnerabilities)
     - Define technical requirements (API <200ms, 10k+ users, security-first, comprehensive testing)
     - Mark template as "Manifested by SKYNET"
     - Announce auto-progression to analysis phase

5. **Initialize Knowledge Management**
   - If cargo command is available, run Fortitude integration initialization
   - Set up pattern research capabilities for architectural analysis
   - Handle gracefully if Fortitude is not available

6. **Update Project State**
   - If jq is available, update `docs/ce-dps-state.json` with:
     - current_phase = 1
     - phase_1_started timestamp
     - last_updated timestamp
   - If jq not available, warn user about manual state management

7. **Provide Clear Next Steps**
   - If not SKYNET mode: Guide user to fill out template sections (Problem Statement, Target Users, Success Metrics, Technical Requirements)
   - Explain validation checklist for template completion
   - Direct user to run `/phase1:analyze` when template is complete
   - If SKYNET mode: Announce automatic progression to analysis

## Expected Behavior

The command should handle both human oversight and autonomous SKYNET modes, providing comprehensive setup with clear error handling and next-step guidance. Template deployment should be robust with fallbacks for missing dependencies.