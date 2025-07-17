# <context>SKYNET Auto-Compact Recovery</context>

<meta>
  <title>SKYNET Auto-Compact Recovery Command</title>
  <type>slash-command</type>
  <audience>ai_assistant</audience>
  <complexity>advanced</complexity>
  <updated>2025-07-17</updated>
  <scope>skynet-recovery</scope>
  <mode>auto-compact-recovery</mode>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: Detect and recover from auto-compact interruptions in SKYNET mode
- **Core Benefits**: Seamless loop continuation, context regeneration, state restoration
- **Prerequisites**: Interrupted SKYNET loop with saved state in docs/skynet-loop-state.json
- **Output**: Restored autonomous operation with context continuation

## <instructions priority="high">Auto-Compact Recovery Process</instructions>

### <step-1>Detect Auto-Compact Interruption</step-1>
**Interruption Detection**:
- Check for existence of docs/skynet-loop-state.json
- Compare saved state with current environment
- Validate that recovery is needed

**Detection Instructions**:
1. Check if `docs/skynet-loop-state.json` exists using file system tools
2. If it doesn't exist, display "❌ No SKYNET loop state found. Use /skynet:enable to start." and exit
3. Read the file and extract the `skynet_active` field
4. Check current `SKYNET` environment variable using the Bash tool
5. If saved state shows `skynet_active` is not `true`, display "❌ No active SKYNET loop to resume. Use /skynet:enable to start." and exit
6. If current environment already shows `SKYNET=true`, display "✅ SKYNET already active. Use /skynet:status for current state." and exit

### <step-2>Display Recovery Status</step-2>
**Recovery Announcement**:
- Show "🔄 SKYNET AUTO-COMPACT RECOVERY" with visual borders
- Display interruption details and recovery plan
- Show last known loop position and next command

**Status Display Instructions**:
1. Display "🔄 SKYNET AUTO-COMPACT RECOVERY INITIATED" with separator lines
2. Show "Auto-compact interruption detected and recovery starting..."
3. Read the loop state file and display:
   - Loop Position from `loop_position` field
   - Current Sprint from `current_sprint` field
   - Next Command from `next_command` field
   - Last Updated from `last_updated` field

### <step-3>Validate Project State Consistency</step-3>
**State Validation**:
- Check docs/ce-dps-state.json for consistency
- Validate sprint directories and files
- Verify git repository state
- Check for manual changes during interruption

**Consistency Checks**:
```bash
# Validate project state consistency
if [[ ! -f "docs/ce-dps-state.json" ]]; then
    echo "❌ Project state file missing. Run /init to reinitialize."
    exit 1
fi

# Check for git working directory changes
if ! git diff --quiet; then
    echo "⚠️  Working directory has uncommitted changes."
    echo "Consider stashing or committing changes before recovery."
    read -p "Continue with recovery anyway? (y/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Validate sprint directory exists
current_sprint=$(jq -r '.current_sprint' docs/skynet-loop-state.json)
if [[ ! -d "docs/sprints/sprint-$(printf "%03d" $current_sprint)" ]]; then
    echo "⚠️  Sprint directory missing. Will create during recovery."
fi
```

### <step-4>Restore Environment Variables</step-4>
**Environment Restoration Instructions**:
1. Use the Bash tool to execute: `export SKYNET=true`
2. Read the loop state file to get the saved CE_DPS_PHASE value
3. Use the Bash tool to execute: `export CE_DPS_PHASE=<saved_phase_value>`
4. Display "🔧 Environment Variables Restored:" with the current values

### <step-5>Regenerate Working Context</step-5>
**Context Regeneration**:
- Read phase planning documents
- Scan implementation files for current state
- Review sprint goals and progress
- Rebuild understanding of current tasks

**Context Rebuilding**:
```bash
# Regenerate working context
echo "🧠 Regenerating Working Context..."

# Read current phase state
current_phase=$(jq -r '.current_phase' docs/ce-dps-state.json)
echo "  Current Phase: $current_phase"

# Check sprint status
current_sprint=$(jq -r '.current_sprint' docs/skynet-loop-state.json)
sprint_dir="docs/sprints/sprint-$(printf "%03d" $current_sprint)"
if [[ -d "$sprint_dir" ]]; then
    echo "  Sprint $current_sprint: Active"
    if [[ -f "$sprint_dir/sprint-info.json" ]]; then
        echo "  Sprint Goals: $(jq -r '.goals[]' "$sprint_dir/sprint-info.json" | head -3)"
    fi
else
    echo "  Sprint $current_sprint: Directory missing (will create)"
fi

# Check recent git activity
echo "  Recent Commits:"
git log --oneline -5 | sed 's/^/    /'
echo ""
```

### <step-6>Update Loop State for Recovery</step-6>
**Recovery State Update**:
- Mark recovery as complete
- Update last execution timestamp
- Add recovery event to loop history

**State Update**:
```bash
# Update loop state with recovery
current_time=$(date -u +%Y-%m-%dT%H:%M:%SZ)
jq --arg timestamp "$current_time" \
   '.auto_compact_recovery = true |
    .last_execution = $timestamp |
    .loop_history += [{
      "action": "auto_compact_recovery",
      "timestamp": $timestamp,
      "recovered_from": .loop_position,
      "next_command": .next_command
    }]' docs/skynet-loop-state.json > tmp.json && mv tmp.json docs/skynet-loop-state.json
```

### <step-7>Execute Next Command in Sequence</step-7>
**Seamless Continuation**:
- Retrieve next command from loop state
- Display continuation message
- Execute next command automatically
- Resume autonomous operation

**Command Execution**:
```bash
# Get next command and execute
next_command=$(jq -r '.next_command' docs/skynet-loop-state.json)
loop_position=$(jq -r '.loop_position' docs/skynet-loop-state.json)

echo "✅ Recovery Complete - Resuming Autonomous Operation"
echo "=================================================="
echo "Continuing from: $loop_position"
echo "Executing next command: $next_command"
echo ""

# Execute next command
if [[ -n "$next_command" && "$next_command" != "null" ]]; then
    echo "🚀 Executing: $next_command"
    # The next command will be executed automatically
else
    echo "⚠️  No next command specified. Use /project-status to determine next steps."
fi
```

### <step-8>Display Recovery Summary</step-8>
**Recovery Completion**:
- Show recovery success message
- Display current loop status
- Show available control commands
- Confirm autonomous operation restored

## <expected-output priority="medium">Recovery Results</expected-output>

**Command Execution Flow**:
- Detect auto-compact interruption by comparing saved vs current state
- Display recovery status with last known position and next command
- Validate project state consistency and check for manual changes
- Restore environment variables (SKYNET=true, CE_DPS_PHASE)
- Regenerate working context from project files and git history
- Update loop state with recovery completion and timestamp
- Execute next command in sequence to resume autonomous operation
- Display recovery summary with current status and control commands

## <parameters priority="low">Command Configuration</parameters>
**Configuration Details**:
- No parameters required
- Reads from docs/skynet-loop-state.json for recovery state
- Uses jq for JSON parsing and git for repository state
- Automatically executes next command in autonomous sequence

## <implementation-notes priority="critical">Recovery Standards</implementation-notes>
**Critical Requirements**:
- **Complete state validation** before attempting recovery
- **Context regeneration** from multiple sources (files, git, state)
- **Seamless continuation** exactly where loop was interrupted
- **Safety checks** for working directory changes and consistency
- **Audit trail** of recovery events in loop history
- **Automatic execution** of next command to resume autonomy