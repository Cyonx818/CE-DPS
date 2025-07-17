# <context>CE-DPS Phase 2 Setup</context>

<meta>
  <title>CE-DPS Phase 2 Sprint Planning Setup</title>
  <type>slash-command</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-17</updated>
  <scope>phase2-setup</scope>
  <phase>sprint-development</phase>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: Initialize Phase 2 sprint planning environment with feature selection template and comprehensive setup
- **Core Benefits**: Template deployment, SKYNET auto-population, sprint tracking, environment configuration
- **Prerequisites**: Completed Phase 1 with completion report
- **Output**: Phase 2 planning template with feature roadmap and sprint environment

## <instructions priority="high">Phase 2 Setup Process</instructions>

### <step-1>Validate Phase 1 Completion</step-1>
**Prerequisites Validation**:
- Verify docs/ce-dps-state.json exists with CE-DPS project initialization
- Check if jq available for JSON processing (warn if not available)
- If jq available, validate Phase 1 marked complete using: `jq -r '.phases_completed[]' docs/ce-dps-state.json | grep -q '^1$'`
- If jq not available, check for docs/phases/phase-1-completion-report.md as fallback
- Exit with error message and suggested next steps if Phase 1 not completed

### <step-2>Check Phase 2 Initialization Status</step-2>
**Initialization Check**:
- Use Read tool to check if docs/phases/phase-2-sprint-planning.md already exists
- If file exists:
  - Inform user Phase 2 is already initialized
  - Suggest: "Delete docs/phases/phase-2-sprint-planning.md and run /phase2:setup again to restart"
  - Provide command: `rm docs/phases/phase-2-sprint-planning.md`
  - Exit without making changes to avoid overwriting existing work

### <step-3>Configure Phase 2 Environment</step-3>
**Environment Configuration**:
- Set environment variables:
  - CE_DPS_PHASE=2
  - CE_DPS_FORTITUDE_ENABLED=true
  - CE_DPS_QUALITY_GATES=true
- **SKYNET mode** (if SKYNET=true):
  - CE_DPS_HUMAN_APPROVAL_REQUIRED=false
  - Display SKYNET autonomous mode message
- **Non-SKYNET mode**:
  - CE_DPS_HUMAN_APPROVAL_REQUIRED=true
  - Display human oversight mode message

### <step-4>Deploy Phase 2 Template</step-4>
**Template Deployment**:
- Use Read tool to verify methodology/templates/phase-2-template.md exists
- If template exists, read its content and use Write tool to create docs/phases/phase-2-sprint-planning.md
- If template doesn't exist, provide error message:
  - "Error: Template not found at methodology/templates/phase-2-template.md"
  - "Check project structure and ensure templates directory exists"
  - "Run /cedps-init to reinitialize project structure if needed"

### <step-5>Handle SKYNET Auto-Population</step-5>
**SKYNET Mode** (if enabled):
- Add "Manifested by SKYNET" header to document
- **Auto-populate "Selected Features for Sprint 1"**:
  - Core Authentication System (High priority, Medium complexity, no dependencies)
  - API Framework and Validation (High priority, Medium complexity, depends on auth)
  - Database Integration and ORM (High priority, Medium complexity, depends on API)
  - Basic Admin Dashboard (Medium priority, Low complexity, depends on all above)
- **Auto-populate "Implementation Approach"**:
  - TDD approach with >95% coverage
  - Security-first patterns throughout
  - Incremental delivery in dependency order
  - Comprehensive quality gates
- **Auto-populate "Resource Allocation"**:
  - 4-5 week sprint duration
  - 80% implementation, 15% QA, 5% documentation split
  - Success criteria including functionality, quality, performance, documentation
- Execute auto-progression to planning phase (/phase2:plan)

### <step-6>Create Working Environment</step-6>
**Directory Creation**:
- Use Bash tool to create directories: `mkdir -p docs/phases/phase-2-artifacts docs/sprints/sprint-001`
- Verify directories were created successfully
- Use Read tool to check if docs/phases/phase-1-roadmap.md exists
- If Phase 1 roadmap exists, extract features to docs/phases/phase-2-artifacts/available-features.md
- If extraction succeeds, announce: "Feature roadmap extracted from Phase 1 planning"
- If Phase 1 roadmap doesn't exist, create placeholder file with note about manual feature definition

### <step-7>Initialize Sprint Tracking</step-7>
**Sprint Info Creation** (docs/sprints/sprint-001/sprint-info.json):
- Use Write tool to create sprint tracking file with structure:
  - sprint_number: 1
  - phase: 2
  - status: "planning"
  - created_at: current timestamp (use `date -u +%Y-%m-%dT%H:%M:%SZ`)
  - features_selected: empty array
  - complexity_analysis: null
  - implementation_plan: null
  - human_approvals: empty array
- Verify file was created successfully

### <step-8>Update Project State and Loop State</step-8>
**State Management** (docs/ce-dps-state.json):
- Read current state file using Read tool
- Update specific fields using Edit tool:
  - current_phase: 2
  - phase_2_started: current timestamp (use `date -u +%Y-%m-%dT%H:%M:%SZ`)
  - last_updated: current timestamp
- Validate update was successful by reading the file again
- If update fails, provide clear error message and manual steps

**Loop State Update** (if SKYNET=true):
- Check if SKYNET environment variable is set to "true"
- Verify ./tools/skynet-loop-manager.sh exists and is executable
- Update loop state: `./tools/skynet-loop-manager.sh update-state "phase2_setup_complete" "phase2:setup_complete" "/phase2:plan"`
- Handle gracefully if skynet-loop-manager.sh fails (warn but continue)
- This tracks autonomous loop progression for recovery purposes

### <step-9>Prepare Fortitude Integration</step-9>
**Knowledge Management**:
- Use Bash tool to check if cargo command is available: `command -v cargo`
- If cargo available, attempt Fortitude query for implementation patterns
- Use error handling: if Fortitude query fails, continue with warning message
- Log: "Fortitude integration available/unavailable for implementation planning"
- This step is optional and shouldn't block Phase 2 setup completion

### <step-10>Provide Next Steps</step-10>
**Next Actions**:
- **SKYNET mode**: Display message "🤖 SKYNET autonomous mode - run /phase2:plan to continue"
- **Human mode**: Provide detailed instructions for feature selection:
  - Reviewing available features from Phase 1 roadmap
  - Selecting 2-4 features based on business priority, dependencies, team capacity, user value
  - Completing business priority input section
  - Next command to run: /phase2:plan

## <expected-behavior priority="medium">Setup Operation</expected-behavior>

**Operational Requirements**:
- Handle both human oversight and autonomous SKYNET modes
- Provide comprehensive environment setup and template deployment
- Include clear guidance for next steps
- Auto-population in SKYNET mode should provide realistic feature selection and implementation approach