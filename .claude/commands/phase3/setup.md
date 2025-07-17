# CE-DPS Phase 3 Setup

Initialize Phase 3 implementation environment with comprehensive development workflow and quality gates.

## Instructions

1. **Validate Phase Completion**
   - Check that Phases 1-2 are complete using jq on docs/ce-dps-state.json
   - Verify docs/phases/phase-2-completion-report.md exists
   - Ensure Phase 3 template exists at methodology/templates/phase-3-template.md
   - Check if Phase 3 already initialized (exit if docs/phases/phase-3-implementation.md exists)

2. **Set Environment Variables**
   - Export CE_DPS_PHASE=3
   - Export CE_DPS_FORTITUDE_ENABLED=true
   - Export CE_DPS_QUALITY_GATES=true
   - Export CE_DPS_HUMAN_APPROVAL_REQUIRED=true

3. **Update Project State**
   - Use jq to update docs/ce-dps-state.json with current_phase=3, phase_3_started timestamp
   - Copy methodology/templates/phase-3-template.md to docs/phases/phase-3-implementation.md

4. **Create Working Directories**
   - Create docs/phases/phase-3-artifacts
   - Create docs/sprints/sprint-001/implementation
   - Create docs/quality-reports/sprint-001

5. **Initialize Implementation Tracking**
   - Create docs/sprints/sprint-001/implementation/implementation-status.json with sprint metadata
   - Set status to "setup", initialize empty arrays for features and quality gates

6. **Create Feature Branch**
   - Create or switch to sprint-001-implementation branch using git
   - Handle case where branch already exists

7. **Initialize Quality Gates and Tools**
   - Build quality gates tool: cd tools/quality-gates && cargo build --release
   - Prepare Rust testing framework with cargo test --no-run
   - Query Fortitude for implementation patterns if available

8. **Extract Sprint Backlog**
   - Copy docs/sprints/sprint-001/backlog/sprint-backlog.md to docs/phases/phase-3-artifacts/implementation-backlog.md

9. **Create Pre-Implementation Checklist**
   - Generate docs/phases/phase-3-artifacts/pre-implementation-checklist.md with comprehensive checklist
   - Include environment setup, implementation planning, quality standards, human validation points

## Expected Output

Output will show:
- Prerequisites validation (with specific error messages if missing jq, git, or templates)
- Environment variable setup and project state updates  
- Directory creation and file copying operations
- Feature branch creation or switching
- Quality gates compilation and tool preparation
- Fortitude integration preparation
- Sprint backlog extraction
- Pre-implementation checklist creation
- Success confirmation with file locations
- SKYNET mode auto-transition if enabled

## Human Action Required

After setup completes:
1. Review pre-implementation checklist at docs/phases/phase-3-artifacts/pre-implementation-checklist.md
2. Confirm sprint backlog at docs/phases/phase-3-artifacts/implementation-backlog.md  
3. Validate development environment and tools
4. When ready, run /cedps-phase3-implement to begin implementation

## Parameters
- No parameters required
- Checks for SKYNET environment variable for autonomous mode
- Uses jq for JSON processing (warns if not available)
- Requires git repository and cargo toolchain