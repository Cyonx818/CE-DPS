# <context>Enable SKYNET Mode</context>

<meta>
  <title>SKYNET Mode Activation</title>
  <type>slash-command</type>
  <audience>ai_assistant</audience>
  <complexity>basic</complexity>
  <updated>2025-07-17</updated>
  <scope>skynet-control</scope>
  <mode>autonomous-activation</mode>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: Enable autonomous CE-DPS operation without human approval checkpoints
- **Core Benefits**: Autonomous workflow loops, auto-populated templates, bypassed approval gates
- **Quality Standards**: All technical quality gates remain fully enforced
- **Output**: SKYNET environment activation with comprehensive status display

## <instructions priority="high">SKYNET Activation Process</instructions>

### <step-1>Load SKYNET Context and Set Environment</step-1>
**Context Loading** (CRITICAL):
- Use the Read tool to load: `methodology/ai-implementation/skynet-autonomous-mode.md`
- This document contains essential autonomous operation patterns and command progression logic
- Loading this context is required for proper SKYNET autonomous loop execution

**Environment Configuration**:
- Use the Bash tool to execute: `export SKYNET=true`
- Update the loop state file `docs/skynet-loop-state.json` with activation details
- Display activation message with visual borders
- Show autonomous operation confirmation

**Loop State Update Instructions**:
1. Use the Bash tool to run: `./tools/skynet-loop-manager.sh enable`
2. This will automatically update the loop state file with SKYNET activation details

### <step-2>Display Activation Message</step-2>
**Activation Announcement** ("🤖 SKYNET mode ENABLED"):
- ⚡ **Autonomous operation activated**
- ⚡ **Human approval checkpoints will be bypassed**
- ⚡ **Templates will be auto-populated with contextual values**
- ⚡ **Technical quality gates remain fully enforced**
- ⚡ **All documents modified by AI will be marked 'Manifested by SKYNET'**

### <step-3>Validate Mode Setting</step-3>
**Status Validation**:
- Check if SKYNET=true successfully set
- Display confirmation or error if failed
- **Technical quality standards**: MAINTAINED
- **Human approval requirements**: BYPASSED
- **Continuous development loops**: ENABLED

### <step-4>Explain Immediate Workflow Effects</step-4>
**Phase-Specific Effects**:
- **Phase 1**: Business requirements auto-generated from project context
- **Phase 2**: Features auto-selected based on complexity and dependencies
- **Phase 3**: Business validation auto-approved with logical justification
- **Quality**: All technical quality gates remain fully enforced (>95% coverage, security, performance)
- **Continuous**: After Phase 3 + quality check, automatically loops back to Phase 2

### <step-5>Show Control Commands and Workflow</step-5>
**Control Commands**:
- /skynet:disable to return to human oversight
- /skynet:status to check current mode
- /skynet:resume to recover from auto-compact interruptions

**Autonomous Loop Workflow**:
Phase1 (setup→analyze→validate) → Phase2 (setup→plan→validate) → Phase3 (setup→implement→validate) → Quality Check → Loop back to Phase2 (next sprint)

**Command Execution Flow**:
/phase1:setup → /phase1:analyze → /phase1:validate → /phase2:setup → /phase2:plan → /phase2:validate → /phase3:setup → /phase3:implement → /phase3:validate → /quality-check → /phase2:setup (increment sprint) → repeat indefinitely

### <step-6>Document Quality Standards and Transparency</step-6>
**Quality Framework**:
- **Technical standards maintained**: test coverage, security, performance, code quality
- **Business logic automation**: requirements, feature selection, approvals
- **Audit trail**: all SKYNET-modified documents marked with headers
- **Environment variable state**: tracked in project files

## <expected-output priority="medium">Activation Results</expected-output>

**Command Execution**:
- Export SKYNET=true environment variable
- Display activation message with visual borders and status indicators
- Validate the environment variable was set correctly
- Show immediate workflow effects for all three phases
- Display quality standards that remain enforced
- Show transparency and audit trail information
- Provide control commands and workflow diagram

## <parameters priority="low">Command Configuration</parameters>
**Configuration Details**:
- No parameters required
- Sets SKYNET environment variable for current session
- Displays comprehensive status and workflow information

## <implementation-notes priority="low">Technical Details</implementation-notes>
**Technical Implementation**:
- Uses bash commands with export and echo statements
- Includes validation checks with if statements
- Maintains all technical quality requirements
- Provides clear visual feedback with separator lines
- Shows workflow diagram for autonomous operation