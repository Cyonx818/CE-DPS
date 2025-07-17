# <context>CE-DPS Phase 3 Setup</context>

<meta>
  <title>CE-DPS Phase 3 Implementation Setup</title>
  <type>slash-command</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-17</updated>
  <scope>phase3-setup</scope>
  <phase>code-implementation</phase>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: Initialize Phase 3 implementation environment with comprehensive development workflow and quality gates
- **Core Benefits**: Environment setup, branch creation, quality tools preparation, implementation tracking
- **Prerequisites**: Completed Phases 1-2 with completion reports
- **Output**: Complete development environment ready for TDD implementation

## <instructions priority="high">Phase 3 Setup Process</instructions>

### <step-1>Validate Phase Completion</step-1>
**Prerequisites Validation**:
- Use Read tool to check docs/ce-dps-state.json exists
- Validate Phases 1-2 complete using: `jq -r '.phases_completed[]' docs/ce-dps-state.json | grep -c '^[12]$'` (should return 2)
- Verify docs/phases/phase-2-completion-report.md exists using Read tool
- Ensure Phase 3 template exists at methodology/templates/phase-3-template.md using Read tool
- Check if Phase 3 already initialized (exit if docs/phases/phase-3-implementation.md exists)

### <step-2>Set Environment Variables</step-2>
**Environment Configuration**:
- CE_DPS_PHASE=3
- CE_DPS_FORTITUDE_ENABLED=true
- CE_DPS_QUALITY_GATES=true
- CE_DPS_HUMAN_APPROVAL_REQUIRED=true

### <step-3>Update Project State</step-3>
**State Management**:
- Read current state file using Read tool
- Update specific fields using Edit tool:
  - current_phase: 3
  - phase_3_started: current timestamp (use `date -u +%Y-%m-%dT%H:%M:%SZ`)
  - last_updated: current timestamp
- Use Read tool to get methodology/templates/phase-3-template.md content
- Use Write tool to create docs/phases/phase-3-implementation.md with template content

### <step-4>Create Working Directories</step-4>
**Directory Creation**:
- docs/phases/phase-3-artifacts
- docs/sprints/sprint-001/implementation
- docs/quality-reports/sprint-001

### <step-5>Initialize Implementation Tracking</step-5>
**Tracking Setup** (docs/sprints/sprint-001/implementation/implementation-status.json):
- Check if docs/sprints/sprint-001/implementation/implementation-status.json exists using Read tool
- If file exists, read current content and update status to "setup" using Edit tool
- If file doesn't exist, use Write tool to create tracking file with:
  - Sprint metadata with status "setup"
  - Initialize empty arrays for features and quality gates

### <step-6>Create Feature Branch</step-6>
**Git Branch Management**:
- Create or switch to sprint-001-implementation branch
- Handle case where branch already exists

### <step-7>Initialize Quality Gates and Tools</step-7>
**Tool Preparation**:
- Build quality gates tool by navigating to tools/quality-gates directory and running cargo build --release
- Prepare Rust testing framework by running cargo test --no-run in the project root
- Query Fortitude for implementation patterns if available

### <step-8>Extract Sprint Backlog</step-8>
**Backlog Preparation**:
- Copy docs/sprints/sprint-001/backlog/sprint-backlog.md to docs/phases/phase-3-artifacts/implementation-backlog.md

### <step-9>Create Pre-Implementation Checklist</step-9>
**Checklist Generation** (docs/phases/phase-3-artifacts/pre-implementation-checklist.md):
- Environment setup verification
- Implementation planning validation
- Quality standards confirmation
- Human validation points identification

## <expected-output priority="medium">Setup Results</expected-output>

**Command Output**:
- Prerequisites validation (with specific error messages if missing jq, git, or templates)
- Environment variable setup and project state updates
- Directory creation and file copying operations
- Feature branch creation or switching
- Quality gates compilation and tool preparation
- Fortitude integration preparation
- Sprint backlog extraction
- Pre-implementation checklist creation
- Success confirmation with file locations
- SKYNET mode auto-transition if enabled

## <human-actions priority="high">Required Follow-up and Auto-Progression</human-actions>

**SKYNET Mode**:
- Display message "🤖 SKYNET autonomous mode - auto-progressing to /phase3:implement"
- **CRITICAL**: Read the /phase3:implement command file using Read tool: `.claude/commands/phase3/implement.md`
- Execute the /phase3:implement command instructions directly within same Claude session context
- This implements the autonomous loop progression without human intervention while maintaining context consistency

**Non-SKYNET Mode Post-Setup Actions**:
1. Review pre-implementation checklist at docs/phases/phase-3-artifacts/pre-implementation-checklist.md
2. Confirm sprint backlog at docs/phases/phase-3-artifacts/implementation-backlog.md
3. Validate development environment and tools
4. When ready, run /phase3:implement to begin implementation

## <parameters priority="low">Command Configuration</parameters>
**Configuration Details**:
- No parameters required
- Checks for SKYNET environment variable for autonomous mode
- Uses jq for JSON processing (warns if not available)
- Requires git repository and cargo toolchain