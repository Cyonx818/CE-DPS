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
- If jq available, validate Phase 1 marked complete in phases_completed array
- If jq not available, check for docs/phases/phase-1-completion-report.md as fallback
- Exit with error if Phase 1 not completed

### <step-2>Check Phase 2 Initialization Status</step-2>
**Initialization Check**:
- If docs/phases/phase-2-sprint-planning.md already exists:
  - Inform user Phase 2 is already initialized
  - Suggest deleting file and running command again to restart
  - Exit without making changes

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
- Verify methodology/templates/phase-2-template.md exists
- Copy template to docs/phases/phase-2-sprint-planning.md
- If copy fails, provide error about template location and project structure

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
- Create docs/phases/phase-2-artifacts/ directory
- Create docs/sprints/sprint-001/ directory
- Extract feature roadmap from Phase 1 planning document to docs/phases/phase-2-artifacts/available-features.md
- If extraction succeeds, announce roadmap extraction

### <step-7>Initialize Sprint Tracking</step-7>
**Sprint Info Creation** (docs/sprints/sprint-001/sprint-info.json):
- sprint_number: 1
- phase: 2
- status: "planning"
- created_at: current timestamp
- features_selected: empty array
- complexity_analysis: null
- implementation_plan: null
- human_approvals: empty array

### <step-8>Update Project State and Loop State</step-8>
**State Management** (docs/ce-dps-state.json):
- **If jq available**, update with:
  - current_phase = 2
  - last_updated timestamp
  - phase_2_started timestamp
- **If jq not available**: warn about manual state management

**Loop State Update** (if SKYNET=true):
```bash
# Update loop state for phase 2 setup
current_time=$(date -u +%Y-%m-%dT%H:%M:%SZ)
if [[ "$SKYNET" == "true" ]]; then
    current_sprint=$(jq -r '.current_sprint // 1' docs/skynet-loop-state.json 2>/dev/null || echo "1")
    jq --arg timestamp "$current_time" \
       --arg sprint "$current_sprint" \
       '.loop_position = "phase2:setup_complete" |
        .next_command = "/phase2:plan" |
        .last_execution = $timestamp |
        .environment_vars.CE_DPS_PHASE = "2" |
        .loop_history += [{
          "action": "phase2_setup_complete",
          "timestamp": $timestamp,
          "sprint": $sprint,
          "next_step": "phase2_plan"
        }]' docs/skynet-loop-state.json > tmp.json && mv tmp.json docs/skynet-loop-state.json
fi
```

### <step-9>Prepare Fortitude Integration</step-9>
**Knowledge Management**:
- If cargo command available:
  - Run Fortitude query for implementation patterns
  - Handle gracefully if Fortitude query fails (optional)
- Prepare knowledge lookup capabilities for implementation planning

### <step-10>Provide Next Steps</step-10>
**Next Actions**:
- **SKYNET mode**: Execute automatic progression to implementation planning (/phase2:plan)
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