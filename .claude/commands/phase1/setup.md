# <context>CE-DPS Phase 1 Setup</context>

<meta>
  <title>CE-DPS Phase 1 Strategic Planning Setup</title>
  <type>slash-command</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-17</updated>
  <scope>phase1-initialization</scope>
  <phase>strategic-planning</phase>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: Initialize Phase 1 strategic planning environment and business requirements template
- **Core Benefits**: Automated template deployment, SKYNET mode support, knowledge management integration
- **Prerequisites**: Initialized CE-DPS project (docs/ce-dps-state.json)
- **Output**: Phase 1 planning template and environment configuration

## <instructions priority="high">Phase 1 Setup Process</instructions>

### <step-1>Validate Project Prerequisites</step-1>
**Validation Checks**:
- Verify docs/ce-dps-state.json exists (inform user to run /cedps-init if missing)
- Check if docs/phases/phase-1-planning.md exists (inform if Phase 1 already initialized)
- Confirm project is in CE-DPS root directory with methodology structure

### <step-2>Configure Phase 1 Environment</step-2>
**Environment Configuration**:
- CE_DPS_PHASE=1
- CE_DPS_FORTITUDE_ENABLED=true (knowledge management)
- CE_DPS_QUALITY_GATES=true (validation)
- CE_DPS_HUMAN_APPROVAL_REQUIRED=false if SKYNET=true, else true
- Display appropriate mode message (SKYNET autonomous or human oversight)

### <step-3>Deploy Business Requirements Template</step-3>
**Template Deployment**:
- Copy methodology/templates/phase-1-template.md to docs/phases/phase-1-planning.md
- If copy fails, provide error message about template location
- Create docs/phases/phase-1-artifacts/ working directory

### <step-4>Handle SKYNET Auto-Population</step-4>
**SKYNET Mode** (if SKYNET=true):
- Auto-populate template with comprehensive business requirements focused on AI-assisted development
- Include problem statement about accelerating development with quality
- Add target users (development teams, product managers, QA teams)
- Set success metrics (>50% faster delivery, >95% test coverage, zero critical vulnerabilities)
- Define technical requirements (API <200ms, 10k+ users, security-first, comprehensive testing)
- Mark template as "Manifested by SKYNET"
- Announce auto-progression to analysis phase

### <step-5>Initialize Knowledge Management</step-5>
**Fortitude Integration**:
- If cargo available, run Fortitude integration initialization
- Set up pattern research capabilities for architectural analysis
- Handle gracefully if Fortitude not available

### <step-6>Update Project State</step-6>
**State Management** (docs/ce-dps-state.json):
- If jq available, update with:
  - current_phase = 1
  - phase_1_started timestamp
  - last_updated timestamp
- If jq not available, warn user about manual state management

### <step-7>Provide Clear Next Steps</step-7>
**Next Actions**:
- **Non-SKYNET mode**: Guide user to fill out template sections (Problem Statement, Target Users, Success Metrics, Technical Requirements)
- Explain validation checklist for template completion
- Direct user to run /phase1:analyze when template is complete
- **SKYNET mode**: Announce automatic progression to analysis

## <expected-behavior priority="medium">Command Operation</expected-behavior>

**Operational Requirements**:
- Handle both human oversight and autonomous SKYNET modes
- Provide comprehensive setup with clear error handling
- Include next-step guidance and robust template deployment
- Implement fallbacks for missing dependencies