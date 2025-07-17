# CE-DPS Project Initialization

Initialize a new CE-DPS project with required directory structure and configuration.

## Instructions

1. **Validate Environment**
   - Check that we're in a git repository (git rev-parse --git-dir)
   - Verify CLAUDE.md exists in project root
   - Confirm we have write permissions for creating directories

2. **Create Directory Structure**
   - Create docs/ directory if it doesn't exist
   - Create subdirectories:
     - docs/phases/
     - docs/phases/phase-1-artifacts/
     - docs/phases/phase-2-artifacts/  
     - docs/phases/phase-3-artifacts/
     - docs/sprints/
     - docs/quality-reports/

3. **Initialize Project State**
   - Create docs/ce-dps-state.json with initial state:
     ```json
     {
       "project_initialized": true,
       "current_phase": 0,
       "project_type": "default",
       "phases_completed": [],
       "quality_gates_enabled": true,
       "fortitude_enabled": true,
       "human_approval_required": true,
       "skynet_mode": "false",
       "created_at": "current_timestamp",
       "last_updated": "current_timestamp"
     }
     ```

4. **Set Initial Environment**
   - Set CE_DPS environment variables:
     - CE_DPS_PHASE=0
     - CE_DPS_FORTITUDE_ENABLED=true
     - CE_DPS_QUALITY_GATES=true
     - CE_DPS_HUMAN_APPROVAL_REQUIRED=true

5. **Verify Methodology Templates**
   - Check that methodology/templates/ directory exists
   - Verify presence of phase templates:
     - phase-1-template.md
     - phase-2-template.md
     - phase-3-template.md
   - If missing, inform user to ensure complete CE-DPS installation

6. **Initialize Git Integration**
   - Check git status and current branch
   - Ensure docs/ directory will be tracked by git

## Expected Output

```
🚀 Initializing CE-DPS Project...

✅ Git repository validated
✅ CLAUDE.md found  
✅ Directory structure created
✅ Project state initialized
✅ Environment variables configured
✅ Methodology templates validated

CE-DPS project initialized successfully!

📂 Created directories:
   - docs/phases/ (phase documentation)
   - docs/sprints/ (sprint tracking)
   - docs/quality-reports/ (quality metrics)

📄 Project state: docs/ce-dps-state.json
🎯 Current phase: 0 (Ready to start)

Next Steps:
1. Run /cedps-status to see current project state
2. Run /project:phase1:setup to begin strategic planning
3. Review CLAUDE.md for project-specific guidance

💡 Need help? Run /cedps-help for command overview
```

## Notes
- Ensure clean initialization without conflicts
- Validate all prerequisites before creating files
- Provide clear next steps for getting started
- Handle errors gracefully with helpful messages