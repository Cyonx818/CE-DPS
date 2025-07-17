# <context>CE-DPS Project Status</context>

<meta>
  <title>CE-DPS Project Status Display</title>
  <type>slash-command</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-17</updated>
  <scope>status-monitoring</scope>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: Comprehensive project status display with phase progress and actionable next steps
- **Core Benefits**: Real-time status monitoring, SKYNET mode detection, intelligent recommendations
- **Prerequisites**: Initialized CE-DPS project with state tracking
- **Output**: Formatted status report with environment details and next actions

## <instructions priority="high">Status Analysis Process</instructions>

### <step-1>Check Project Initialization and Auto-Compact Detection</step-1>
- Check docs/ce-dps-state.json existence (error if missing)
- Read project state using jq: current_phase, phases_completed, initialization status
- **Auto-Compact Detection**: Check for interrupted SKYNET loops

**Auto-Compact Detection Instructions**:
1. Use the Bash tool to run: `./tools/skynet-loop-manager.sh display-auto-compact`
2. Use the Bash tool to run: `./tools/skynet-loop-manager.sh check-auto-compact` to determine if auto-compact was detected
3. Store the result to influence subsequent recommendations

### <step-2>Display Project Overview</step-2>
**Status Elements**:
- project_initialized status
- current_phase from state file
- phases_completed array
- production_ready status if available

### <step-3>Analyze Phase Status</step-3>
**Phase Validation**:
- Check Phase 1: 1 in phases_completed array
- Check Phase 2: 2 in phases_completed array
- Check Phase 3: 3 in phases_completed array

**Status Icons**:
- ✅ Phase complete
- 🔄 Phase available/in progress
- ⏸️ Phase blocked (prerequisites not met)

### <step-4>Check Sprint Status</step-4>
- Look for docs/sprints/sprint-001/ directory
- Check sprint planning and implementation files
- Display sprint progress if available

### <step-5>Display Environment Status</step-5>
**Environment Variables**:
- CE_DPS_PHASE
- CE_DPS_FORTITUDE_ENABLED
- CE_DPS_QUALITY_GATES
- CE_DPS_HUMAN_APPROVAL_REQUIRED

### <step-6>Show SKYNET Mode Status and Loop Information</step-6>
**Mode Detection**:
- 🟢 **ENABLED**: Autonomous Operation (SKYNET=true)
- 🟡 **EXPLICITLY DISABLED**: Human Oversight (SKYNET=false)
- 🔵 **NOT SET**: Default Human Oversight (SKYNET unset)

**Loop State Information Instructions**:
1. Use the Bash tool to run: `./tools/skynet-loop-manager.sh display-state`
2. This will display comprehensive SKYNET loop state information if available

### <step-7>Display Git Status</step-7>
- Show current git branch (git branch --show-current)
- Identify branch type: implementation, planning, or main

### <step-8>Provide Next Steps with Auto-Compact Awareness</step-8>
**Intelligent Recommendations**:
- **Auto-compact detected**: /skynet:resume to continue autonomous operation
- **Not initialized**: /init
- **Phase 1 incomplete**: /phase1:setup or /phase1:analyze
- **Phase 1 complete, Phase 2 incomplete**: /phase2:setup
- **Phase 2 complete, Phase 3 incomplete**: /phase3:setup
- **All phases complete**: quality validation or next sprint

**Auto-Compact Priority Instructions**:
1. If auto-compact was detected (from earlier check), prioritize recovery recommendations:
   - Display "🚨 PRIORITY RECOMMENDATION: Auto-compact interruption detected"
   - Show recovery options: `/skynet:resume`, `/skynet:status`, `/skynet:disable`
2. Otherwise, proceed with normal phase-based recommendations

### <step-9>List Available Commands</step-9>
- Show relevant CE-DPS commands for current phase
- Include SKYNET control commands
- Show documentation and help commands

## <expected-output priority="medium">Status Report Format</expected-output>

**Command Execution Flow**:
- Check docs/ce-dps-state.json existence
- Use jq to read project state and parse JSON
- Display formatted project overview with current phase and completed phases
- Show phase status with appropriate icons based on completion
- Check for sprint files and display sprint status
- Show environment variables with current values
- Display SKYNET mode status with conditional logic
- Show current git branch
- Provide phase-appropriate next step recommendations
- List available commands organized by category

## <parameters priority="low">Command Configuration</parameters>
**Configuration Details**:
- No parameters required
- Uses jq for JSON parsing (warns if not available)
- Uses git commands for branch status
- Checks environment variables for configuration status

## <implementation-notes priority="low">Technical Details</implementation-notes>
**Technical Implementation**:
- Comprehensive status display using bash commands with conditionals
- Uses jq for reliable JSON parsing of project state
- Provides actionable next steps based on current project status
- Includes error handling for uninitialized projects
- Shows both technical status and strategic guidance