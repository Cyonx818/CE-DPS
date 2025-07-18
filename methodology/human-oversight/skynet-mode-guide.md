# SKYNET Mode: Autonomous Development Guide

## Overview

SKYNET mode enables fully autonomous CE-DPS development cycles without human approval checkpoints. When enabled, Claude Code operates independently through all three phases of development, automatically looping from Phase 3 back to Phase 2 for continuous sprint development.

This guide covers the autonomous operation system, state management, and the auto-compact resilience features that ensure continuous operation even when Claude Code's context is reset.

## How SKYNET Mode Works

### Autonomous Operation

When SKYNET mode is enabled (`SKYNET=true`), the CE-DPS methodology operates with the following changes:

**Phase 1 (Strategic Planning)**:
- Business requirements are auto-generated from project context
- Architectural decisions are made automatically based on best practices
- Human approval checkpoints are bypassed

**Phase 2 (Sprint Planning)**:
- Features are auto-selected based on complexity and dependencies
- Implementation approaches are chosen automatically
- Sprint scope is determined without human input

**Phase 3 (Implementation)**:
- Code is implemented with full test-driven development
- Business validation is auto-approved with logical justification
- All technical quality gates remain fully enforced

**Quality Gates**:
- All technical standards remain unchanged (>95% test coverage, security scans, performance requirements)
- Quality failures still block progression
- After successful quality validation, the system automatically loops back to Phase 2 for the next sprint

### Continuous Loop Workflow

```
Phase 1 → Phase 2 → Phase 3 → Quality Check → Phase 2 (next sprint) → Phase 3 → Quality Check → ...
```

The system maintains this loop indefinitely, incrementing sprint numbers and continuously developing features until:
- Quality gates fail
- Technical issues are detected
- Human intervention occurs (disabling SKYNET mode)

## State Management System

SKYNET mode uses two primary state files to maintain autonomous operation:

### 1. CE-DPS State File (`docs/ce-dps-state.json`)

Standard CE-DPS project state tracking:
- Current phase and completion status
- Phase timestamps and progression
- Project initialization details
- Quality gate settings

### 2. SKYNET Loop State File (`docs/skynet-loop-state.json`)

Autonomous loop-specific state management:

```json
{
  "skynet_active": true,
  "loop_position": "phase2:setup_complete",
  "next_command": "/phase2:plan",
  "current_sprint": 3,
  "last_execution": "2025-07-17T14:50:15Z",
  "loop_iteration": 5,
  "auto_compact_recovery": false,
  "environment_vars": {
    "SKYNET": "true",
    "CE_DPS_PHASE": "2"
  },
  "loop_history": [
    {
      "action": "phase2_setup_complete",
      "timestamp": "2025-07-17T14:50:15Z",
      "position": "phase2:setup_complete",
      "next_command": "/phase2:plan"
    }
  ]
}
```

**Key Fields**:
- `skynet_active`: Whether SKYNET mode is currently enabled
- `loop_position`: Current position in the development loop
- `next_command`: The next slash command to execute
- `current_sprint`: Current sprint number (auto-increments)
- `loop_iteration`: Total number of loop cycles completed
- `loop_history`: Audit trail of all loop activities

## Auto-Compact Resilience System

### The Problem

Claude Code's auto-compact feature resets the AI's context window, which would normally interrupt autonomous operation. When this happens:
- The `SKYNET=true` environment variable is lost
- Claude Code loses awareness of the current loop position
- Autonomous operation would halt unexpectedly

### The Solution

The resilience system detects and recovers from auto-compact interruptions:

#### 1. **Auto-Compact Detection**

The system detects interruptions by comparing:
- **Saved state**: `skynet_active: true` in the loop state file
- **Current environment**: `SKYNET` variable is missing or `false`

When this mismatch is detected, the system identifies an auto-compact interruption.

#### 2. **Recovery Process**

Recovery happens through the `/skynet:resume` command or automatic detection in other commands:

1. **Environment Restoration**: Re-exports `SKYNET=true` and other environment variables
2. **Context Regeneration**: Rebuilds understanding from project files, git history, and state files
3. **Continuation**: Executes the next command in the loop sequence
4. **Audit Trail**: Records the recovery event in the loop history

#### 3. **Detection Points**

Auto-compact detection occurs in multiple commands:
- `/project-status` - Shows recovery recommendations with high priority
- `/skynet:status` - Displays interruption details and recovery options
- `/init` - Offers recovery before proceeding with initialization
- `/skynet:resume` - Dedicated recovery command

### Recovery Commands

**Primary Recovery**: `/skynet:resume`
- Detects and validates auto-compact interruption
- Restores environment variables
- Regenerates working context
- Continues execution from the exact interruption point

**Status Check**: `/skynet:status`
- Shows current SKYNET mode status
- Displays auto-compact detection results
- Provides recovery recommendations

**Project Status**: `/project-status`
- Prioritizes auto-compact recovery in recommendations
- Shows detailed loop state information
- Guides users to appropriate recovery actions

## Control Commands

### Enable/Disable SKYNET Mode

```bash
# Enable autonomous operation
/skynet:enable

# Disable and return to human oversight
/skynet:disable

# Check current status
/skynet:status
```

### Recovery Commands

```bash
# Resume after auto-compact interruption
/skynet:resume

# Check for interruptions and view state
/project-status

# View detailed loop information
/skynet:status
```

## Troubleshooting

### Common Issues

#### 1. **Auto-Compact Detection Not Working**

**Symptoms**: SKYNET mode appears to stop without recovery notifications

**Causes**:
- Loop state file is missing or corrupted
- Environment variables not properly saved
- State file permissions issues

**Solutions**:
- Check that `docs/skynet-loop-state.json` exists and is readable
- Verify file contains valid JSON with `jq . docs/skynet-loop-state.json`
- Re-enable SKYNET mode with `/skynet:enable` if state is corrupted
- Check file permissions: `ls -la docs/skynet-loop-state.json`

#### 2. **Loop Stuck in Recovery Mode**

**Symptoms**: Recovery commands don't progress or keep showing interruption detected

**Causes**:
- Working directory has uncommitted changes
- Git repository is in conflicted state
- Sprint directories are missing or corrupted

**Solutions**:
- Commit or stash uncommitted changes: `git status`
- Resolve any git conflicts
- Check sprint directories exist: `ls docs/sprints/`
- Use `/skynet:disable` then `/skynet:enable` to reset loop state

#### 3. **Sprint Numbers Not Incrementing**

**Symptoms**: Loop continues but stays on same sprint number

**Causes**:
- Quality gates failing silently
- Sprint increment logic not executing
- State file update failures

**Solutions**:
- Check quality gate results with `/quality-check`
- Verify state file is writable and not locked
- Review loop history in state file for increment events
- Manually increment with utility: `./tools/skynet-loop-manager.sh increment-sprint`

#### 4. **Environment Variables Not Persisting**

**Symptoms**: SKYNET mode appears disabled after recovery

**Causes**:
- Environment variables not properly exported
- Shell session limitations
- Recovery process incomplete

**Solutions**:
- Manually export variables: `export SKYNET=true`
- Check environment with: `echo $SKYNET`
- Re-run recovery: `/skynet:resume`
- Verify state file has correct environment values

#### 5. **State File Corruption**

**Symptoms**: JSON parsing errors or invalid state data

**Causes**:
- Interrupted state file writes
- Manual editing with syntax errors
- File system issues

**Solutions**:
- Validate JSON syntax: `jq . docs/skynet-loop-state.json`
- Restore from backup if available
- Reset state file by disabling and re-enabling SKYNET mode
- Check disk space and file system integrity

### Emergency Recovery

If SKYNET mode becomes completely unresponsive:

1. **Force Disable**: Delete or rename the loop state file
2. **Reset Environment**: `unset SKYNET` and restart
3. **Reinitialize**: Use `/skynet:enable` to start fresh
4. **Manual Recovery**: Edit state file with correct values if necessary

### State File Backup

Consider backing up the loop state file during critical operations:

```bash
# Create backup
cp docs/skynet-loop-state.json docs/skynet-loop-state.json.backup

# Restore backup
cp docs/skynet-loop-state.json.backup docs/skynet-loop-state.json
```

## Best Practices

### When to Use SKYNET Mode

- **Ideal**: Well-defined projects with clear requirements
- **Ideal**: Established codebases with good test coverage
- **Ideal**: Repetitive development tasks (API endpoints, CRUD operations)
- **Caution**: Experimental or research projects
- **Caution**: Projects requiring frequent architectural changes

### Monitoring SKYNET Operation

- Check `/project-status` regularly for loop health
- Monitor git commits for autonomous progress
- Review loop history in state file for anomalies
- Watch for quality gate failures that might stop the loop

### Recovery Preparation

- Ensure `jq` is installed for JSON processing
- Keep the utility script executable: `chmod +x tools/skynet-loop-manager.sh`
- Understand the loop state file structure
- Know how to manually disable SKYNET mode if needed

## Integration with Human Oversight

SKYNET mode is designed to complement, not replace, human strategic oversight:

- **Strategic Decisions**: Humans still define project vision and requirements in Phase 1
- **Quality Standards**: All technical quality gates remain enforced
- **Business Validation**: Humans can review and approve completed features
- **Intervention**: Humans can disable SKYNET mode at any time for manual control

The system provides audit trails and comprehensive state information to support human oversight and decision-making throughout the autonomous development process.