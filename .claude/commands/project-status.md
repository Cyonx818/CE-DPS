# CE-DPS Project Status

Display comprehensive CE-DPS project status and phase progress with actionable next steps.

## Instructions

1. **Check Project Initialization**
   - Check if docs/ce-dps-state.json exists (error if not initialized)
   - Read project state using jq to get current_phase, phases_completed, project initialization status

2. **Display Project Overview**
   - Show project_initialized status
   - Display current_phase from state file
   - List phases_completed array
   - Show production_ready status if available

3. **Analyze Phase Status**
   - Check if 1 is in phases_completed array for Phase 1 status
   - Check if 2 is in phases_completed array for Phase 2 status  
   - Check if 3 is in phases_completed array for Phase 3 status
   - Display appropriate status icons:
     - ✅ Phase complete
     - 🔄 Phase available/in progress
     - ⏸️ Phase blocked (prerequisites not met)

4. **Check Sprint Status** 
   - Look for docs/sprints/sprint-001/ directory
   - Check for sprint planning and implementation files
   - Display sprint progress if available

5. **Display Environment Status**
   - Show CE_DPS_PHASE environment variable
   - Show CE_DPS_FORTITUDE_ENABLED status
   - Show CE_DPS_QUALITY_GATES status
   - Show CE_DPS_HUMAN_APPROVAL_REQUIRED status

6. **Show SKYNET Mode Status**
   - Check SKYNET environment variable value
   - Display appropriate status:
     - 🟢 SKYNET MODE: ENABLED (Autonomous Operation) if true
     - 🟡 SKYNET MODE: EXPLICITLY DISABLED (Human Oversight) if false
     - 🔵 SKYNET MODE: NOT SET (Default: Human Oversight) if unset

7. **Display Git Status**
   - Show current git branch using git branch --show-current
   - Identify if on implementation, planning, or main branch

8. **Provide Next Steps**
   - Based on current phase and completion status, recommend next command:
     - If not initialized: /cedps-init
     - If Phase 1 incomplete: /cedps-phase1-setup or /cedps-phase1-analyze
     - If Phase 1 complete, Phase 2 incomplete: /cedps-phase2-setup
     - If Phase 2 complete, Phase 3 incomplete: /cedps-phase3-setup
     - If all phases complete: quality validation or next sprint

9. **List Available Commands**
   - Show relevant CE-DPS commands for current phase
   - Include SKYNET control commands
   - Show documentation and help commands

## Expected Output

The command will execute bash commands that:
- Check for docs/ce-dps-state.json existence
- Use jq to read project state and parse JSON
- Display formatted project overview with current phase and completed phases
- Show phase status with appropriate icons based on completion
- Check for sprint files and display sprint status
- Show environment variables with their current values
- Display SKYNET mode status with conditional logic
- Show current git branch
- Provide phase-appropriate next step recommendations
- List available commands organized by category

## Parameters
- No parameters required
- Uses jq for JSON parsing (warns if not available)
- Uses git commands for branch status
- Checks environment variables for configuration status

## Notes
- Comprehensive status display using actual bash commands with conditionals
- Uses jq for reliable JSON parsing of project state
- Provides actionable next steps based on current project status
- Includes error handling for uninitialized projects
- Shows both technical status and strategic guidance