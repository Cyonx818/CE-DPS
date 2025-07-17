# CE-DPS Phase 1 Setup

Initialize Phase 1 strategic planning with business requirements template.

## Instructions

1. **Validate Prerequisites**
   - Check that project is in a git repository
   - Verify CLAUDE.md exists for project guidance
   - Confirm docs/ directory exists and is writable

2. **Check Initialization Status**
   - If docs/ce-dps-state.json doesn't exist, inform user to run `/cedps-init` first
   - If docs/phases/phase-1-planning.md already exists, offer to restart or continue

3. **Setup Phase 1 Environment**
   - Set environment variables for Phase 1:
     - CE_DPS_PHASE=1
     - CE_DPS_FORTITUDE_ENABLED=true
     - CE_DPS_QUALITY_GATES=true
     - CE_DPS_HUMAN_APPROVAL_REQUIRED=true (unless SKYNET mode)
   - Create docs/phases/phase-1-artifacts/ directory
   - Copy methodology/templates/phase-1-template.md to docs/phases/phase-1-planning.md

4. **Initialize Fortitude Integration**
   - If cargo is available, prepare Fortitude for knowledge lookup
   - Set up pattern research capabilities for architectural analysis

5. **Update Project State**
   - Update docs/ce-dps-state.json:
     - Set current_phase = 1
     - Add phase_1_started timestamp
     - Update last_updated timestamp

6. **Provide Guidance**
   - Explain next steps for human business requirements input
   - Guide user to fill out business context, requirements, and constraints
   - Mention the next command: `/phase1:analyze`

## Expected Output

```
🚀 Setting up CE-DPS Phase 1: Strategic Planning...

✅ Prerequisites validated
✅ Phase 1 environment configured
✅ Strategic planning template copied
✅ Fortitude knowledge integration prepared
✅ Project state updated to Phase 1

Phase 1 initialization complete!
📋 Template location: docs/phases/phase-1-planning.md
🎯 Working directory: docs/phases/phase-1-artifacts/

Next Steps:
1. Edit docs/phases/phase-1-planning.md
2. Fill out business requirements sections:
   - Problem Statement
   - Target Users  
   - Success Metrics
   - Technical Requirements
3. Run /phase1:analyze for AI architectural analysis
```

## Notes
- Focus on setup and template preparation
- Provide clear guidance for human input requirements
- Avoid complex validation logic
- Set up environment for subsequent AI analysis