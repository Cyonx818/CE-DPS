# CE-DPS Project Status

Display comprehensive CE-DPS project status and phase progress with actionable next steps.

## Instructions

1. **Check Project Initialization**
   - Read docs/ce-dps-state.json to verify project is initialized
   - If file doesn't exist, inform user to run `/cedps-init` first

2. **Display Project Overview**
   - Show current phase from state file
   - List completed phases array
   - Display project type and initialization status

3. **Analyze Phase Status**
   - **Phase 1**: Check if phase 1 is in phases_completed array
     - If complete: Show ✅ Phase 1: Strategic Planning - Complete
     - If not complete: Show 🔄 Phase 1: Strategic Planning - In Progress
   - **Phase 2**: Check if phase 2 is in phases_completed array
     - If phase 1 complete but not phase 2: Show 🔄 Phase 2: Sprint Planning - Available
     - If phase 1 not complete: Show ⏸️ Phase 2: Sprint Planning - Blocked
   - **Phase 3**: Check if phase 3 is in phases_completed array
     - If phases 1&2 complete but not phase 3: Show 🔄 Phase 3: Implementation - Available
     - Otherwise: Show ⏸️ Phase 3: Implementation - Blocked

4. **Check Sprint Status**
   - Look for docs/sprints/sprint-001/sprint-info.json
   - If exists, show sprint status from that file

5. **Display Environment Status**
   - Show CE_DPS_PHASE, CE_DPS_FORTITUDE_ENABLED, CE_DPS_QUALITY_GATES variables
   - Show CE_DPS_HUMAN_APPROVAL_REQUIRED status

6. **Show SKYNET Mode Status**
   - Check SKYNET environment variable
   - If "true": Show 🟢 SKYNET MODE: ENABLED (Autonomous Operation)
   - If "false": Show 🟡 SKYNET MODE: EXPLICITLY DISABLED (Human Oversight)  
   - If unset: Show 🔵 SKYNET MODE: NOT SET (Default: Human Oversight)

7. **Display Git Status**
   - Show current git branch
   - Indicate if on implementation branch or planning branch

8. **Provide Next Steps**
   - Based on current phase, recommend next command to run
   - If Phase 1 incomplete: Suggest `/cedps-phase1-setup` or `/cedps-phase1-analyze`
   - If Phase 1 complete, Phase 2 incomplete: Suggest `/cedps-phase2-setup`
   - If Phase 2 complete, Phase 3 incomplete: Suggest `/cedps-phase3-setup`

9. **List Other Available Commands**
   - Show other CE-DPS commands for reference
   - Include SKYNET control commands
   - Show documentation locations

## Expected Output Format

```
📊 CE-DPS Project Status Report
================================

📈 Project Overview
===================
Project Initialized: true/false
Current Phase: 1/2/3
Phases Completed: [array]
Production Ready: true/false

🎯 Phase Status
===============
✅/🔄/⏸️ Phase 1: Strategic Planning - Status
✅/🔄/⏸️ Phase 2: Sprint Planning - Status  
✅/🔄/⏸️ Phase 3: Implementation - Status

🔧 Environment Status
=====================
CE_DPS_PHASE: value
CE_DPS_FORTITUDE_ENABLED: value
...

🤖 SKYNET Mode Status
=====================
Status and description

📝 Git Status
=============
Current branch: branch-name

🎯 Next Steps
=============
Recommended next command and purpose

💡 Other Commands
================
List of available commands
```

## Notes
- Use the corrected jq pattern: `cat file.json | jq` instead of `jq file.json`
- Focus on reading and analyzing, not executing complex shell logic
- Provide clear, actionable guidance for next steps
- Keep output clean and well-formatted