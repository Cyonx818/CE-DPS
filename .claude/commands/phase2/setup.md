# CE-DPS Phase 2 Setup

Initialize Phase 2 sprint planning environment with feature selection template and comprehensive setup.

## Instructions

1. **Validate Phase 1 Completion**
   - Verify `docs/ce-dps-state.json` exists with CE-DPS project initialization
   - Check if jq is available for JSON processing (warn if not available)
   - If jq is available, validate Phase 1 is marked complete in phases_completed array
   - If jq not available, check for `docs/phases/phase-1-completion-report.md` as fallback
   - Exit with error if Phase 1 not completed

2. **Check Phase 2 Initialization Status**
   - If `docs/phases/phase-2-sprint-planning.md` already exists:
     - Inform user Phase 2 is already initialized
     - Suggest deleting file and running command again to restart
     - Exit without making changes

3. **Configure Phase 2 Environment**
   - Set environment variables:
     - CE_DPS_PHASE=2
     - CE_DPS_FORTITUDE_ENABLED=true  
     - CE_DPS_QUALITY_GATES=true
   - If SKYNET environment variable is true:
     - Set CE_DPS_HUMAN_APPROVAL_REQUIRED=false
     - Display SKYNET autonomous mode message
   - If SKYNET not true:
     - Set CE_DPS_HUMAN_APPROVAL_REQUIRED=true
     - Display human oversight mode message

4. **Deploy Phase 2 Template**
   - Verify `methodology/templates/phase-2-template.md` exists
   - Copy template to `docs/phases/phase-2-sprint-planning.md`
   - If copy fails, provide error about template location and project structure

5. **Handle SKYNET Auto-Population**
   - If SKYNET mode is enabled:
     - Add "Manifested by SKYNET" header to document
     - Auto-populate "Selected Features for Sprint 1" section with:
       - Core Authentication System (High priority, Medium complexity, no dependencies)
       - API Framework and Validation (High priority, Medium complexity, depends on auth)
       - Database Integration and ORM (High priority, Medium complexity, depends on API)
       - Basic Admin Dashboard (Medium priority, Low complexity, depends on all above)
     - Auto-populate "Implementation Approach" section with:
       - TDD approach with >95% coverage
       - Security-first patterns throughout
       - Incremental delivery in dependency order
       - Comprehensive quality gates
     - Auto-populate "Resource Allocation" section with:
       - 4-5 week sprint duration
       - 80% implementation, 15% QA, 5% documentation split
       - Success criteria including functionality, quality, performance, documentation
     - Announce auto-progression to planning phase

6. **Create Working Environment**
   - Create `docs/phases/phase-2-artifacts/` directory
   - Create `docs/sprints/sprint-001/` directory
   - Extract feature roadmap from Phase 1 planning document to `docs/phases/phase-2-artifacts/available-features.md`
   - If extraction succeeds, announce roadmap extraction

7. **Initialize Sprint Tracking**
   - Create `docs/sprints/sprint-001/sprint-info.json` with:
     - sprint_number: 1
     - phase: 2
     - status: "planning"
     - created_at: current timestamp
     - features_selected: empty array
     - complexity_analysis: null
     - implementation_plan: null
     - human_approvals: empty array

8. **Update Project State**
   - If jq is available, update `docs/ce-dps-state.json` with:
     - current_phase = 2
     - last_updated timestamp
     - phase_2_started timestamp
   - If jq not available, warn about manual state management

9. **Prepare Fortitude Integration**
   - If cargo command is available:
     - Run Fortitude query for implementation patterns
     - Handle gracefully if Fortitude query fails (optional)
   - Prepare knowledge lookup capabilities for implementation planning

10. **Provide Next Steps**
    - If SKYNET mode: Announce automatic progression to implementation planning
    - If human mode: Provide detailed instructions for feature selection including:
      - Reviewing available features from Phase 1 roadmap
      - Selecting 2-4 features based on business priority, dependencies, team capacity, user value
      - Completing business priority input section
      - Next command to run: `/phase2:plan`

## Expected Behavior

Command should handle both human oversight and autonomous SKYNET modes with comprehensive environment setup, template deployment, and clear guidance for next steps. Auto-population in SKYNET mode should provide realistic feature selection and implementation approach.