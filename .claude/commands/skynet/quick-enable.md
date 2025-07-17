# <context>Quick Enable SKYNET Mode</context>

<meta>
  <title>SKYNET Quick Mode Activation</title>
  <type>slash-command</type>
  <audience>ai_assistant</audience>
  <complexity>basic</complexity>
  <updated>2025-07-17</updated>
  <scope>skynet-control</scope>
  <mode>fast-activation</mode>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: Fast SKYNET mode activation without recovery checks
- **Core Benefits**: Minimal output, immediate activation
- **Quality Standards**: All technical quality gates remain fully enforced
- **Output**: Simple activation confirmation

## <instructions priority="high">Quick SKYNET Activation</instructions>

### <step-1>Load Context and Set Environment Variable</step-1>
**Context Loading** (CRITICAL):
- Use the Read tool to load: `methodology/ai-implementation/skynet-autonomous-mode.md`
- This document contains essential autonomous operation patterns and command progression logic
- Loading this context is required for proper SKYNET autonomous loop execution

**Environment Configuration**:
- Use the Bash tool to execute: `export SKYNET=true`
- Update the loop state file: `./tools/skynet-loop-manager.sh enable`
- Display simple activation confirmation

### <step-2>Validate and Confirm</step-2>
**Status Validation**:
- Check if SKYNET=true successfully set
- Display "🤖 SKYNET mode ENABLED" or error if failed
- Show basic control commands

## <expected-output priority="medium">Activation Results</expected-output>

**Command Execution**:
- Export SKYNET=true environment variable
- Update loop state file
- Display simple activation confirmation
- Show basic control commands

**Control Commands**:
- `/skynet:disable` - Return to human oversight
- `/skynet:status` - Check current mode
- `/skynet:enable` - Full activation with recovery checks

## <parameters priority="low">Command Configuration</parameters>
**Configuration Details**:
- No parameters required
- Sets SKYNET environment variable for current session
- Minimal output for fast activation