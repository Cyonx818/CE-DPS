# CE-DPS Phase 2 Setup

Initialize Phase 2 sprint planning with feature selection template and environment setup.

## Instructions

1. **Validate Prerequisites**
   - Check that docs/ce-dps-state.json exists 
   - Verify Phase 1 is marked complete in phases_completed array
   - Confirm methodology/templates/phase-2-template.md exists
   - If Phase 1 not complete, inform user to complete Phase 1 first

2. **Check for Existing Phase 2**
   - If docs/phases/phase-2-sprint-planning.md already exists:
     - Inform user Phase 2 already initialized
     - Suggest deleting file to restart or proceeding with existing setup

3. **Setup Phase 2 Environment**
   - Copy methodology/templates/phase-2-template.md to docs/phases/phase-2-sprint-planning.md
   - Create working directories:
     - docs/phases/phase-2-artifacts/
     - docs/sprints/sprint-001/
   - Update project state to set current_phase = 2

4. **Extract Phase 1 Information**
   - Read Phase 1 planning document for feature roadmap
   - Extract available features for sprint selection
   - Save feature list to docs/phases/phase-2-artifacts/available-features.md

5. **Initialize Sprint Tracking**
   - Create docs/sprints/sprint-001/sprint-info.json with:
     - sprint_number: 1
     - phase: 2  
     - status: "planning"
     - created_at: current timestamp
     - features_selected: []

6. **Handle SKYNET Mode** 
   - If SKYNET=true: Auto-populate template with selected features based on dependencies
   - If SKYNET=false or unset: Leave template for human completion
   - Mark documents with SKYNET header if autonomous mode

7. **Provide Next Steps**
   - If human mode: Explain need for manual feature selection
   - If SKYNET mode: Explain auto-selection and immediate progression
   - Guide user to next appropriate command

## Expected Output

```
🚀 Setting up CE-DPS Phase 2: Sprint Planning...

✅ Prerequisites validated
✅ Phase 2 template copied
✅ Working directories created  
✅ Sprint tracking initialized
✅ Feature roadmap extracted

Phase 2 environment initialized successfully!
📋 Sprint planning template: docs/phases/phase-2-sprint-planning.md
🎯 Sprint 1 directory: docs/sprints/sprint-001/
🔧 Environment configured for Phase 2

Next Steps:
[Appropriate next steps based on SKYNET mode]
```

## Notes
- Focus on file operations and state management
- Avoid complex shell logic
- Handle both SKYNET and human modes appropriately
- Provide clear guidance for next steps
- Keep error handling simple and clear