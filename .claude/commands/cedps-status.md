---
description: "Show current CE-DPS project status, phase progress, and next steps"
allowed-tools: ["bash", "read"]
---

# <context>CE-DPS Project Status</context>

## <summary priority="high">
Display comprehensive CE-DPS project status including current phase, completion progress, and clear next steps.

## <method>Project Status Analysis</method>

### <implementation>
!echo "📊 CE-DPS Project Status Report"
!echo "================================"

# Check if project is initialized
!if [ ! -f "docs/ce-dps-state.json" ]; then
    echo "❌ CE-DPS project not initialized"
    echo "💡 Run '/cedps-init' to initialize the project"
    exit 0
fi

# Read current project state
!echo "🔍 Reading project state..."
!if command -v jq >/dev/null 2>&1; then
    CURRENT_PHASE=$(jq -r '.current_phase' docs/ce-dps-state.json)
    PHASES_COMPLETED=$(jq -r '.phases_completed // []' docs/ce-dps-state.json)
    PROJECT_INITIALIZED=$(jq -r '.project_initialized // false' docs/ce-dps-state.json)
    READY_FOR_PRODUCTION=$(jq -r '.ready_for_production // false' docs/ce-dps-state.json)
else
    echo "⚠️ Warning: jq not found. Using fallback status detection."
    CURRENT_PHASE="unknown"
    PHASES_COMPLETED="[]"
    PROJECT_INITIALIZED="false"
    READY_FOR_PRODUCTION="false"
fi

!echo ""
!echo "📈 Project Overview"
!echo "==================="
!echo "Project Initialized: $PROJECT_INITIALIZED"
!echo "Current Phase: $CURRENT_PHASE"
!echo "Phases Completed: $PHASES_COMPLETED"
!echo "Production Ready: $READY_FOR_PRODUCTION"

# Show phase-specific status
!echo ""
!echo "🎯 Phase Status"
!echo "==============="

# Phase 1 Status
!if command -v jq >/dev/null 2>&1 && jq -e '.phases_completed | contains([1])' docs/ce-dps-state.json >/dev/null 2>&1; then
    echo "✅ Phase 1: Strategic Planning - Complete"
    if [ -f "docs/phases/phase-1-completion-report.md" ]; then
        echo "   📊 Report: docs/phases/phase-1-completion-report.md"
    fi
else
    echo "🔄 Phase 1: Strategic Planning - In Progress"
    if [ -f "docs/phases/phase-1-planning.md" ]; then
        if grep -q "✅ Approved" docs/phases/phase-1-planning.md; then
            echo "   ⏳ Awaiting validation - run '/cedps-phase1-validate'"
        else
            echo "   ⏳ Awaiting human approval of architectural analysis"
        fi
    else
        echo "   ⏳ Not started - run '/cedps-phase1-setup'"
    fi
fi

# Phase 2 Status
!if command -v jq >/dev/null 2>&1 && jq -e '.phases_completed | contains([2])' docs/ce-dps-state.json >/dev/null 2>&1; then
    echo "✅ Phase 2: Sprint Planning - Complete"
    if [ -f "docs/phases/phase-2-completion-report.md" ]; then
        echo "   📊 Report: docs/phases/phase-2-completion-report.md"
    fi
elif command -v jq >/dev/null 2>&1 && jq -e '.phases_completed | contains([1])' docs/ce-dps-state.json >/dev/null 2>&1; then
    echo "🔄 Phase 2: Sprint Planning - Available"
    if [ -f "docs/phases/phase-2-sprint-planning.md" ]; then
        if grep -q "✅ Approved" docs/phases/phase-2-sprint-planning.md; then
            echo "   ⏳ Awaiting validation - run '/cedps-phase2-validate'"
        else
            echo "   ⏳ Awaiting human approval of implementation plan"
        fi
    else
        echo "   ⏳ Not started - run '/cedps-phase2-setup'"
    fi
else
    echo "⏸️  Phase 2: Sprint Planning - Blocked (complete Phase 1 first)"
fi

# Phase 3 Status
!if command -v jq >/dev/null 2>&1 && jq -e '.phases_completed | contains([3])' docs/ce-dps-state.json >/dev/null 2>&1; then
    echo "✅ Phase 3: Implementation - Complete"
    if [ -f "docs/phases/phase-3-completion-report.md" ]; then
        echo "   📊 Report: docs/phases/phase-3-completion-report.md"
    fi
elif command -v jq >/dev/null 2>&1 && jq -e '.phases_completed | contains([1, 2])' docs/ce-dps-state.json >/dev/null 2>&1; then
    echo "🔄 Phase 3: Implementation - Available"
    if [ -f "docs/phases/phase-3-implementation.md" ]; then
        if grep -q "✅ Approved" docs/phases/phase-3-implementation.md; then
            echo "   ⏳ Awaiting validation - run '/cedps-phase3-validate'"
        else
            echo "   ⏳ Implementation in progress or awaiting human validation"
        fi
    else
        echo "   ⏳ Not started - run '/cedps-phase3-setup'"
    fi
else
    echo "⏸️  Phase 3: Implementation - Blocked (complete Phases 1 and 2 first)"
fi

# Show current sprint status if applicable
!if [ -f "docs/sprints/sprint-001/sprint-info.json" ]; then
    echo ""
    echo "🚀 Sprint Status"
    echo "================"
    SPRINT_STATUS=$(jq -r '.status' docs/sprints/sprint-001/sprint-info.json 2>/dev/null || echo "unknown")
    echo "Sprint 1: $SPRINT_STATUS"
    if [ -f "docs/sprints/sprint-001/implementation/implementation-status.json" ]; then
        IMPL_STATUS=$(jq -r '.status' docs/sprints/sprint-001/implementation/implementation-status.json 2>/dev/null || echo "unknown")
        echo "Implementation: $IMPL_STATUS"
    fi
fi

# Show quality metrics if available
!if [ -f "docs/quality-reports/sprint-001/final-quality-report.json" ]; then
    echo ""
    echo "📊 Quality Metrics"
    echo "=================="
    echo "Quality report available: docs/quality-reports/sprint-001/final-quality-report.json"
fi

# Environment status
!echo ""
!echo "🔧 Environment Status"
!echo "====================="
!echo "CE_DPS_PHASE: ${CE_DPS_PHASE:-Not set}"
!echo "CE_DPS_FORTITUDE_ENABLED: ${CE_DPS_FORTITUDE_ENABLED:-Not set}"
!echo "CE_DPS_QUALITY_GATES: ${CE_DPS_QUALITY_GATES:-Not set}"
!echo "CE_DPS_HUMAN_APPROVAL_REQUIRED: ${CE_DPS_HUMAN_APPROVAL_REQUIRED:-Not set}"

# SKYNET mode status
!echo ""
!echo "🤖 SKYNET Mode Status"
!echo "====================="
!SKYNET_STATUS="${SKYNET:-false}"
!if [[ "$SKYNET" == "true" ]]; then
    echo "🟢 SKYNET MODE: ENABLED (Autonomous Operation)"
    echo "   ⚡ Human approval checkpoints: BYPASSED"
    echo "   ⚡ Template auto-population: ENABLED"
    echo "   ⚡ Continuous development loops: ENABLED"
    echo "   ⚡ Technical quality gates: MAINTAINED"
    echo "   ⚡ Next steps will execute automatically"
elif [[ "$SKYNET" == "false" ]]; then
    echo "🟡 SKYNET MODE: EXPLICITLY DISABLED (Human Oversight)"
    echo "   👨‍💼 Human approval checkpoints: REQUIRED"
    echo "   👨‍💼 Template completion: MANUAL"
    echo "   👨‍💼 Manual command execution between phases"
else
    echo "🔵 SKYNET MODE: NOT SET (Default: Human Oversight)"
    echo "   👨‍💼 Human approval checkpoints: REQUIRED"
    echo "   👨‍💼 Template completion: MANUAL"
    echo "   👨‍💼 Manual command execution between phases"
fi
!echo "   🎛️ Control: /skynet-enable, /skynet-disable, /skynet-status"

# Git status
!if git rev-parse --git-dir >/dev/null 2>&1; then
    echo ""
    echo "📝 Git Status"
    echo "============="
    CURRENT_BRANCH=$(git branch --show-current)
    echo "Current branch: $CURRENT_BRANCH"
    if [[ "$CURRENT_BRANCH" == *"sprint-001-implementation"* ]]; then
        echo "🔧 On implementation branch"
    else
        echo "📋 On main/planning branch"
    fi
fi

# Next steps recommendation
!echo ""
!echo "🎯 Next Steps"
!echo "============="

# Check SKYNET mode for next step behavior
!if [[ "$SKYNET" == "true" ]]; then
    echo "🤖 SKYNET MODE: Commands will execute automatically"
    echo "   ⚡ No human intervention required"
    echo "   ⚡ Quality gates must still pass"
    echo ""
fi

!if [ "$CURRENT_PHASE" = "0" ] || [ "$CURRENT_PHASE" = "unknown" ]; then
    echo "👉 Start Phase 1: Strategic Planning"
    echo "   Command: /cedps-phase1-setup"
    echo "   Purpose: Define project vision and approve architecture"
    if [[ "$SKYNET" == "true" ]]; then
        echo "   🤖 SKYNET: Will auto-generate business requirements"
    fi
elif [ "$CURRENT_PHASE" = "1" ]; then
    if command -v jq >/dev/null 2>&1 && jq -e '.phases_completed | contains([1])' docs/ce-dps-state.json >/dev/null 2>&1; then
        echo "👉 Start Phase 2: Sprint Planning"
        echo "   Command: /cedps-phase2-setup"
        echo "   Purpose: Select features and create implementation plan"
    else
        if [ -f "docs/phases/phase-1-planning.md" ]; then
            if grep -q "✅ Approved" docs/phases/phase-1-planning.md; then
                echo "👉 Validate Phase 1 completion"
                echo "   Command: /cedps-phase1-validate"
            else
                echo "👉 Complete Phase 1 analysis"
                echo "   Command: /cedps-phase1-analyze"
                echo "   Note: Fill out business requirements first"
            fi
        else
            echo "👉 Start Phase 1: Strategic Planning"
            echo "   Command: /cedps-phase1-setup"
        fi
    fi
elif [ "$CURRENT_PHASE" = "2" ]; then
    if command -v jq >/dev/null 2>&1 && jq -e '.phases_completed | contains([2])' docs/ce-dps-state.json >/dev/null 2>&1; then
        echo "👉 Start Phase 3: Implementation"
        echo "   Command: /cedps-phase3-setup"
        echo "   Purpose: Implement approved features with quality gates"
    else
        if [ -f "docs/phases/phase-2-sprint-planning.md" ]; then
            if grep -q "✅ Approved" docs/phases/phase-2-sprint-planning.md; then
                echo "👉 Validate Phase 2 completion"
                echo "   Command: /cedps-phase2-validate"
            else
                echo "👉 Complete Phase 2 planning"
                echo "   Command: /cedps-phase2-plan"
                echo "   Note: Select features for sprint first"
            fi
        else
            echo "👉 Start Phase 2: Sprint Planning"
            echo "   Command: /cedps-phase2-setup"
        fi
    fi
elif [ "$CURRENT_PHASE" = "3" ]; then
    if command -v jq >/dev/null 2>&1 && jq -e '.phases_completed | contains([3])' docs/ce-dps-state.json >/dev/null 2>&1; then
        echo "🎉 Implementation Complete!"
        echo "👉 Ready for production deployment"
        echo "   Checklist: docs/phases/phase-3-artifacts/production-deployment-checklist.md"
        echo "   Or start next sprint: /cedps-phase2-setup"
    else
        if [ -f "docs/phases/phase-3-implementation.md" ]; then
            if grep -q "✅ Approved" docs/phases/phase-3-implementation.md; then
                echo "👉 Validate Phase 3 completion"
                echo "   Command: /cedps-phase3-validate"
            else
                echo "👉 Complete Phase 3 implementation"
                echo "   Command: /cedps-phase3-implement"
                echo "   Note: Comprehensive TDD implementation with quality gates"
            fi
        else
            echo "👉 Start Phase 3: Implementation"
            echo "   Command: /cedps-phase3-setup"
        fi
    fi
fi

!echo ""
!echo "💡 Other Commands"
!echo "================"
!echo "/cedps-tools        - Run quality gates and validation tools"
!echo "/cedps-quality-check - Run complete CI/CD test suite with auto-fix"
!echo "/cedps-help         - Show detailed command help"
!echo "/skynet-enable      - Enable autonomous operation mode"
!echo "/skynet-disable     - Return to human oversight mode"
!echo "/skynet-status      - Show detailed SKYNET mode information"
!echo ""
!echo "📚 Documentation"
!echo "================"
!echo "Project overview: docs/PROJECT.md"
!echo "State tracking: docs/ce-dps-state.json"
!echo "Phase documents: docs/phases/"
!echo "Sprint tracking: docs/sprints/"
</implementation>

### <constraints>
- Project must be initialized to show meaningful status
- Requires jq for JSON parsing
- Git repository needed for branch status
- Status based on file existence and content analysis
</constraints>

## <human-action-required>
**Status Report Complete! 📊**

### <status-overview>
The status report shows:
- **Project Initialization**: Whether CE-DPS is set up
- **Current Phase**: Which phase is active (0-3)
- **Phase Completion**: Which phases have been completed
- **Next Steps**: Clear recommendations for proceeding

### <phase-meanings>
**Phase Definitions**:
- **Phase 0**: Project initialized, ready to start
- **Phase 1**: Strategic Planning - Define vision, approve architecture
- **Phase 2**: Sprint Planning - Select features, create implementation plan
- **Phase 3**: Implementation - Code features with quality gates and human validation

### <status-indicators>
**Status Meanings**:
- **✅ Complete**: Phase finished and validated
- **🔄 In Progress**: Phase active, work in progress
- **⏸️ Blocked**: Phase cannot start until prerequisites complete
- **⏳ Awaiting**: Phase needs human action or validation

### <next-steps-guidance>
**Following Next Steps**:
1. **Read the recommendation** in the "Next Steps" section
2. **Run the suggested command** to proceed with the workflow
3. **Complete any human actions** required (approvals, validations, etc.)
4. **Run `/cedps-status`** again to see updated progress

### <troubleshooting-status>
**If Status Seems Wrong**:
- Check that you're in the correct project directory
- Verify that files exist in the expected locations
- Ensure JSON state files are not corrupted
- Run suggested commands even if status seems incorrect

### <quality-tracking>
**Quality Metrics**:
- Status includes quality gate information when available
- Sprint tracking shows implementation progress
- Environment variables show configuration status
- Git status shows current branch context
</human-action-required>

## <troubleshooting>
### <common-issues>
- **"Project not initialized"**: Run `/cedps-init` first
- **"jq: command not found"**: Install jq for JSON processing
- **"Git not found"**: Initialize git repository or install git
- **Status seems outdated**: State files may be corrupted or missing
- **Phase status incorrect**: Check file existence and content
</common-issues>

### <quality-validation>
**Status Command Requirements**:
- [ ] Shows clear current phase and completion status
- [ ] Provides actionable next steps
- [ ] Includes quality metrics when available
- [ ] Shows environment and git status
- [ ] Follows LLM-optimized documentation patterns
- [ ] Helps users understand project progress
- [ ] Guides users toward next actions
</quality-validation>