# <context>CE-DPS Project Initialization</context>

<meta>
  <title>CE-DPS Project Initialization</title>
  <type>slash-command</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-17</updated>
  <scope>project-setup</scope>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: Initialize new CE-DPS project with complete environment and documentation structure
- **Core Benefits**: Automated setup, SKYNET mode detection, comprehensive project scaffolding
- **Prerequisites**: CLAUDE.md must exist in project root
- **Output**: Complete project structure with state tracking and environment configuration

## <instructions priority="high">Setup Process</instructions>

### <step-1>Validate Environment</step-1>
- Check CLAUDE.md exists in project root (required)
- Display initialization message

### <step-2>Check System Dependencies</step-2>
**Required Tools**:
- jq availability (recommended for state management)
- git availability (required for CE-DPS)
- python3 availability (optional)
- Display dependency status with checkmarks/warnings

### <step-3>Create Directory Structure</step-3>
**Directory Creation** (mkdir -p):
- docs/phases
- docs/architecture
- docs/sprints
- docs/quality-reports

### <step-4>Set Environment Variables</step-4>
**Configuration Export**:
- CE_DPS_PHASE=0
- CE_DPS_FORTITUDE_ENABLED=true
- CE_DPS_QUALITY_GATES=true
- CE_DPS_HUMAN_APPROVAL_REQUIRED=true

### <step-5>Detect SKYNET Mode</step-5>
- Display current SKYNET mode status
- Show autonomous operation or human oversight mode

### <step-6>Initialize Project State</step-6>
**State File Creation** (docs/ce-dps-state.json):
- project_initialized, current_phase, phases_completed
- quality_gates_enabled, fortitude_enabled
- SKYNET-specific fields based on current mode
- created_at timestamp using date command

### <step-7>Create Project Documentation Template</step-7>
**Template Creation** (if docs/PROJECT.md missing):
- CE-DPS methodology overview
- Development phases description
- Current status and next actions
- Quality standards
- Tools integration information

### <step-8>Display Success Summary</step-8>
- Show successful initialization message
- List created documentation structure
- Confirm environment variables configured
- Note project state tracking enabled

## <expected-output priority="medium">Initialization Results</expected-output>

**Bash Command Execution**:
- Display initialization message and check CLAUDE.md
- Show system dependency status with checkmarks/warnings
- Create directory structure using mkdir -p commands
- Export environment variables with echo confirmations
- Display SKYNET mode status
- Create docs/ce-dps-state.json using echo commands with timestamps
- Create docs/PROJECT.md template if it doesn't exist
- Show success summary with created structure

## <human-actions priority="high">Required Follow-up</human-actions>

**Post-Initialization Steps**:
1. Review project structure in docs/
2. Customize docs/PROJECT.md with project details  
3. Run /project-status to see current state
4. Run /phase1:setup to begin strategic planning

## <parameters priority="low">Command Configuration</parameters>
**Configuration Details**:
- No parameters required
- Checks for SKYNET environment variable
- Uses date command for timestamps
- Creates comprehensive project structure and documentation