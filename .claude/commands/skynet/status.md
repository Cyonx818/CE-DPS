# SKYNET Mode Status

Display detailed SKYNET mode information and operational status.

## Instructions

1. **Display Status Header**
   - Show "🤖 SKYNET MODE STATUS CHECK" with visual separator lines
   - Check current SKYNET environment variable value

2. **Check and Display Current Mode**
   - **If SKYNET=true**: Show "🟢 STATUS: SKYNET MODE ENABLED" with autonomous operation details
   - **If SKYNET=false**: Show "🟡 STATUS: SKYNET MODE EXPLICITLY DISABLED" with human oversight details  
   - **If SKYNET unset**: Show "🔵 STATUS: SKYNET MODE NOT SET (DEFAULT: HUMAN OVERSIGHT)"

3. **Display Mode-Specific Operational Effects**
   - **For Enabled Mode**:
     - ⚡ AUTONOMOUS OPERATION ACTIVE
     - Human approval checkpoints: BYPASSED
     - Template auto-population: ENABLED
     - Continuous development loops: ENABLED
     - Technical quality gates: MAINTAINED
     - Business validation: AUTO-APPROVED
     - Workflow behavior for each phase
     - Document marking with SKYNET headers
   
   - **For Disabled/Unset Mode**:
     - 👨‍💼 HUMAN OVERSIGHT ACTIVE
     - Human approval checkpoints: REQUIRED
     - Template completion: MANUAL
     - Continuous development loops: DISABLED
     - Technical quality gates: MAINTAINED
     - Business validation: HUMAN REQUIRED
     - Standard CE-DPS collaborative workflow

4. **Show Environment Details**
   - Display all relevant environment variables:
     - SKYNET variable value
     - CE_DPS_PHASE
     - CE_DPS_FORTITUDE_ENABLED
     - CE_DPS_QUALITY_GATES
     - CE_DPS_HUMAN_APPROVAL_REQUIRED

5. **Display Quality Standards (Always Enforced)**
   - Test coverage >95% requirement
   - Security validation comprehensive framework
   - Performance standards <200ms API, <100ms DB
   - Code quality formatting, linting, documentation
   - Anchor tests for critical functionality
   - Security patterns and documentation

6. **Show Recommended Next Actions**
   - Based on current mode, show appropriate next steps
   - For autonomous: suggest /cedps-init or continue current phase
   - For human oversight: suggest /cedps-init or enable autonomous if desired
   - Always show /cedps-status, /cedps-quality-check, /cedps-help options

## Expected Output

The command will execute bash commands that:
- Display status header with visual separator lines
- Check SKYNET environment variable and show appropriate status
- For each mode (enabled/disabled/unset), display:
  - Current operational status with color indicators
  - Detailed workflow behavior for all phases
  - Control commands available
  - Environment variable details
  - Quality standards that remain enforced
  - Recommended next actions based on current mode

## Parameters
- No parameters required
- Reads SKYNET and other CE_DPS environment variables
- Displays comprehensive status information for current mode

## Notes
- Uses actual bash commands with conditional if statements
- Checks for SKYNET=true, SKYNET=false, and unset conditions
- Provides detailed operational behavior explanations
- Shows all relevant environment variables
- Emphasizes quality standards maintained in all modes
- Gives mode-appropriate next step recommendations