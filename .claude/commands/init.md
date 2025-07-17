# CE-DPS Project Initialization

Initialize a new CE-DPS project with complete environment setup and documentation structure.

## Instructions

1. **Validate Environment**
   - Check CLAUDE.md exists in project root (required)
   - Display initialization message

2. **Check System Dependencies**
   - Check for jq availability (recommended for state management)
   - Check for git availability (required for CE-DPS)
   - Check for python3 availability (optional)
   - Display dependency status with checkmarks/warnings

3. **Create Directory Structure**
   - Execute mkdir -p commands to create:
     - docs/phases
     - docs/architecture  
     - docs/sprints
     - docs/quality-reports

4. **Set Environment Variables**
   - Export CE_DPS_PHASE=0
   - Export CE_DPS_FORTITUDE_ENABLED=true
   - Export CE_DPS_QUALITY_GATES=true
   - Export CE_DPS_HUMAN_APPROVAL_REQUIRED=true

5. **Detect SKYNET Mode**
   - Display current SKYNET mode status
   - Show autonomous operation or human oversight mode

6. **Initialize Project State**
   - Create docs/ce-dps-state.json using echo commands
   - Include project_initialized, current_phase, phases_completed, quality_gates_enabled, fortitude_enabled
   - Set SKYNET-specific fields based on current SKYNET mode
   - Add created_at timestamp using date command

7. **Create Project Documentation Template**
   - Check if docs/PROJECT.md exists
   - If not, create comprehensive PROJECT.md with:
     - CE-DPS methodology overview
     - Development phases description
     - Current status and next actions
     - Quality standards
     - Tools integration information

8. **Display Success Summary**
   - Show successful initialization message
   - List created documentation structure
   - Confirm environment variables configured
   - Note project state tracking enabled

## Expected Output

The command will execute bash commands that:
- Display initialization message and check CLAUDE.md
- Show system dependency status with checkmarks/warnings
- Create directory structure using mkdir -p commands
- Export environment variables with echo confirmations
- Display SKYNET mode status
- Create docs/ce-dps-state.json using echo commands with timestamps
- Create docs/PROJECT.md template if it doesn't exist
- Show success summary with created structure

## Human Action Required

After initialization:
1. Review project structure in docs/
2. Customize docs/PROJECT.md with project details
3. Run /cedps-status to see current state
4. Run /cedps-phase1-setup to begin strategic planning

## Parameters
- No parameters required
- Checks for SKYNET environment variable
- Uses date command for timestamps
- Creates comprehensive project structure and documentation