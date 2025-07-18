# <context>SKYNET Mode Status</context>

<meta>
  <title>SKYNET Mode Status Display</title>
  <type>slash-command</type>
  <audience>ai_assistant</audience>
  <complexity>basic</complexity>
  <updated>2025-07-17</updated>
  <scope>skynet-monitoring</scope>
  <mode>status-display</mode>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: Display detailed SKYNET mode information and operational status
- **Core Benefits**: Mode detection, operational effects display, environment monitoring
- **Status Types**: Enabled, explicitly disabled, or unset (default human oversight)
- **Output**: Comprehensive status with mode-specific recommendations

## <instructions priority="high">Status Display Process</instructions>

### <step-1>Display Status Header and Check Auto-Compact</step-1>
**Status Check Initialization**:
- Show "🤖 SKYNET MODE STATUS CHECK" with visual separator lines
- Use the Bash tool to check current SKYNET environment variable value
- Compare with saved loop state for auto-compact detection

**Auto-Compact Detection Instructions**:
1. Use the Bash tool to run: `./tools/skynet-loop-manager.sh display-auto-compact`
2. This will automatically detect and display any auto-compact interruption status

### <step-2>Check and Display Current Mode</step-2>
**Mode Detection**:
- **SKYNET=true**: "🟢 STATUS: SKYNET MODE ENABLED" with autonomous operation details
- **SKYNET=false**: "🟡 STATUS: SKYNET MODE EXPLICITLY DISABLED" with human oversight details
- **SKYNET unset**: "🔵 STATUS: SKYNET MODE NOT SET (DEFAULT: HUMAN OVERSIGHT)"

### <step-3>Display Mode-Specific Operational Effects</step-3>
**Enabled Mode**:
- ⚡ **AUTONOMOUS OPERATION ACTIVE**
- Human approval checkpoints: BYPASSED
- Template auto-population: ENABLED
- Continuous development loops: ENABLED
- Technical quality gates: MAINTAINED
- Business validation: AUTO-APPROVED
- Workflow behavior for each phase
- Document marking with SKYNET headers

**Disabled/Unset Mode**:
- 👨‍💼 **HUMAN OVERSIGHT ACTIVE**
- Human approval checkpoints: REQUIRED
- Template completion: MANUAL
- Continuous development loops: DISABLED
- Technical quality gates: MAINTAINED
- Business validation: HUMAN REQUIRED
- Standard CE-DPS collaborative workflow

### <step-4>Show Environment Details and Loop State</step-4>
**Environment Variables**:
- SKYNET variable value
- CE_DPS_PHASE
- CE_DPS_FORTITUDE_ENABLED
- CE_DPS_QUALITY_GATES
- CE_DPS_HUMAN_APPROVAL_REQUIRED

**Loop State Information Instructions**:
1. Use the Bash tool to run: `./tools/skynet-loop-manager.sh display-state`
2. This will display comprehensive loop state information if available

### <step-5>Display Quality Standards (Always Enforced)</step-5>
**Quality Framework**:
- Test coverage >95% requirement
- Security validation comprehensive framework
- Performance standards <200ms API, <100ms DB
- Code quality formatting, linting, documentation
- Anchor tests for critical functionality
- Security patterns and documentation

### <step-6>Show Recommended Next Actions</step-6>
**Mode-Specific Recommendations**:
- **Autonomous mode**: suggest /init or continue current phase
- **Human oversight**: suggest /init or enable autonomous if desired
- **Always available**: /project-status, /quality-check, /help options

## <expected-output priority="medium">Status Display Results</expected-output>

**Command Execution**:
- Display status header with visual separator lines
- Check SKYNET environment variable and show appropriate status
- **For each mode** (enabled/disabled/unset), display:
  - Current operational status with color indicators
  - Detailed workflow behavior for all phases
  - Control commands available
  - Environment variable details
  - Quality standards that remain enforced
  - Recommended next actions based on current mode

## <parameters priority="low">Command Configuration</parameters>
**Configuration Details**:
- No parameters required
- Reads SKYNET and other CE_DPS environment variables
- Displays comprehensive status information for current mode

## <implementation-notes priority="low">Technical Details</implementation-notes>
**Technical Implementation**:
- Uses bash commands with conditional if statements
- Checks for SKYNET=true, SKYNET=false, and unset conditions
- Provides detailed operational behavior explanations
- Shows all relevant environment variables
- Emphasizes quality standards maintained in all modes
- Gives mode-appropriate next step recommendations