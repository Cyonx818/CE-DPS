---
description: "Validate Phase 2 completion and prepare for Phase 3 implementation"
allowed-tools: ["bash", "read", "write"]
---

# <context>CE-DPS Phase 2: Sprint Planning Validation</context>

## <summary priority="high">
Validate Phase 2 sprint planning completion, verify human approvals for implementation approach, and prepare for Phase 3 implementation transition.

## <method>Phase 2 Validation Process</method>

### <implementation>
!echo "🔍 Validating Phase 2 sprint planning completion..."

# Validate Phase 2 setup exists
!if [ ! -f "docs/phases/phase-2-sprint-planning.md" ]; then
    echo "❌ Error: Phase 2 not found. Run '/cedps-phase2-setup' first."
    exit 1
fi

# Check for AI implementation planning completion
!if ! grep -qi "feature breakdown" docs/phases/phase-2-sprint-planning.md; then
    echo "❌ Error: AI implementation planning not completed. Run '/cedps-phase2-plan' first."
    exit 1
fi

# Validate human approvals are present
!APPROVAL_SECTIONS="Feature Selection Validation,Implementation Approach Approval,Timeline and Resource Approval,Sprint Approval"
!IFS=',' read -ra SECTIONS <<< "$APPROVAL_SECTIONS"
!for section in "${SECTIONS[@]}"; do
    if ! grep -qi "$section" docs/phases/phase-2-sprint-planning.md; then
        echo "❌ Error: Missing human approval section: $section"
        echo "💡 Ensure Claude Code provided all required approval sections."
        exit 1
    fi
done

# Check for human approval decisions
!if ! grep -q "✅ Approved" docs/phases/phase-2-sprint-planning.md; then
    echo "❌ Error: No human approvals found in Phase 2 planning."
    echo "💡 Review and approve implementation plans before proceeding."
    echo "📋 Required approvals: Feature Selection, Implementation Approach, Timeline"
    exit 1
fi

# Validate sprint scope is realistic
!if grep -q "❌ Requires Revision" docs/phases/phase-2-sprint-planning.md; then
    echo "❌ Error: Sprint planning has rejected sections requiring revision."
    echo "💡 Address rejected items before proceeding to implementation."
    exit 1
fi

# Run phase validator tool if available
!if command -v python3 >/dev/null 2>&1 && [ -f "tools/phase-validator.py" ]; then
    echo "🔧 Running phase validation tool..."
    python3 tools/phase-validator.py --phase 2 --file docs/phases/phase-2-sprint-planning.md
    if [ $? -ne 0 ]; then
        echo "❌ Error: Phase validation tool failed."
        echo "💡 Address validation issues before proceeding."
        exit 1
    fi
fi

# Extract sprint backlog for Phase 3
!mkdir -p docs/sprints/sprint-001/backlog
!if grep -A 100 "Sprint Backlog" docs/phases/phase-2-sprint-planning.md > docs/sprints/sprint-001/backlog/sprint-backlog.md; then
    echo "📋 Sprint backlog extracted for Phase 3 implementation"
fi

# Update project state
!if command -v jq >/dev/null 2>&1; then
    jq '.phases_completed += [2] | .phase_2_completed = now | .ready_for_phase_3 = true | .current_sprint = 1' docs/ce-dps-state.json > docs/ce-dps-state.tmp && mv docs/ce-dps-state.tmp docs/ce-dps-state.json
    
    # Update sprint tracking if sprint-info.json exists
    if [ -f "docs/sprints/sprint-001/sprint-info.json" ]; then
        jq '.status = "approved" | .planning_completed = now | .ready_for_implementation = true' docs/sprints/sprint-001/sprint-info.json > docs/sprints/sprint-001/sprint-info.tmp && mv docs/sprints/sprint-001/sprint-info.tmp docs/sprints/sprint-001/sprint-info.json
    fi
else
    echo "⚠️ Warning: jq not found. State update skipped."
    echo "💡 Install jq for automatic state management or update state files manually"
fi

# Create Phase 2 completion report
!mkdir -p docs/phases
!cat > docs/phases/phase-2-completion-report.md << 'EOF'
# Phase 2 Sprint Planning - Completion Report

## Completion Status
- **Phase**: 2 - Sprint Planning
- **Status**: ✅ Complete
- **Sprint**: 1
- **Completed**: $(date -u +%Y-%m-%dT%H:%M:%SZ)
- **Duration**: [Time from setup to completion]

## Deliverables Completed
- [x] Feature selection from Phase 1 roadmap
- [x] Detailed implementation planning
- [x] Technical dependency analysis
- [x] Implementation approach definition
- [x] Effort estimation and timeline
- [x] Risk assessment and mitigation
- [x] Human strategic approvals

## Sprint 1 Approved Scope
- **Selected Features**: [List of approved features]
- **Implementation Approach**: [Summary of approved technical approach]
- **Timeline**: [Approved sprint duration and milestones]
- **Resource Allocation**: [Approved team capacity and effort]

## Key Decisions Approved
- **Feature Priority**: [Summary of approved feature prioritization]
- **Technical Approach**: [Summary of approved implementation strategy]
- **Timeline**: [Summary of approved sprint timeline]
- **Quality Gates**: [Summary of approved testing and validation approach]

## Implementation Ready
- **Sprint Backlog**: Created in docs/sprints/sprint-001/backlog/
- **File-Level Tasks**: Specific implementation tasks defined
- **Quality Standards**: >95% test coverage, security-first patterns
- **Human Oversight**: Strategic decisions approved

## Next Steps
1. Proceed to Phase 3: Implementation
2. Execute sprint backlog according to approved plan
3. Apply quality gates and testing procedures
4. Maintain human oversight for business validation

## Quality Metrics
- Feature selection: Approved by business priorities
- Implementation approach: Technically sound and secure
- Timeline: Realistic and achievable
- Testing strategy: Comprehensive (>95% coverage)
- Human approval: All strategic decisions approved

## Files Created
- `docs/phases/phase-2-sprint-planning.md` - Complete sprint planning document
- `docs/phases/phase-2-completion-report.md` - This completion report
- `docs/sprints/sprint-001/backlog/sprint-backlog.md` - Implementation backlog
- `docs/ce-dps-state.json` - Updated project state

## Ready for Phase 3
Sprint 1 is fully planned and ready for implementation.
EOF

!echo "✅ Phase 2 validation complete!"
!echo "📊 Completion report: docs/phases/phase-2-completion-report.md"
!echo "🎯 Sprint backlog: docs/sprints/sprint-001/backlog/sprint-backlog.md"
!echo "🚀 Ready for Phase 3: Implementation"
</implementation>

### <constraints>
- Phase 2 must be set up and planned
- Human approvals must be present in planning document
- No rejected sections (❌ Requires Revision) allowed
- jq command required for state management
</constraints>

## <human-action-required>
**Phase 2 Validation Complete! 🎉**

### <completion-summary>
**Phase 2 Sprint Planning Successfully Completed**:
- ✅ **Feature Selection**: Sprint features selected and approved
- ✅ **Implementation Planning**: Detailed technical approach defined
- ✅ **Dependency Analysis**: Technical dependencies identified and planned
- ✅ **Effort Estimation**: Realistic timeline with appropriate buffers
- ✅ **Risk Assessment**: Implementation risks identified with mitigation
- ✅ **Human Approvals**: All strategic implementation decisions approved

### <deliverables-created>
**Documentation Generated**:
- `docs/phases/phase-2-sprint-planning.md` - Complete sprint planning document
- `docs/phases/phase-2-completion-report.md` - Phase completion summary
- `docs/sprints/sprint-001/backlog/sprint-backlog.md` - Implementation backlog
- `docs/ce-dps-state.json` - Updated project state tracking

### <sprint-ready>
**Sprint 1 Implementation Ready**:
- **Features**: [Selected features approved for implementation]
- **Approach**: Technical strategy approved and documented
- **Timeline**: Realistic sprint duration with team capacity
- **Quality Gates**: >95% test coverage, security-first patterns
- **Backlog**: File-level tasks ready for development

### <quality-validation>
**CE-DPS Standards Met**:
- [ ] Feature selection maximizes business value
- [ ] Implementation approach uses proven patterns
- [ ] Security considerations integrated throughout
- [ ] Testing strategy comprehensive and realistic
- [ ] Human strategic oversight maintained
- [ ] File-level implementation specificity provided

### <ready-for-phase-3>
**Project Status**: Ready for Phase 3 Implementation

**Next Steps**:
1. **Review Phase 2 outputs** in completion report
2. **Review sprint backlog** for implementation tasks
3. **Prepare development environment** for implementation
4. **Initialize Phase 3** when ready to begin coding

**To Begin Phase 3**:
```bash
# Review completion report
cat docs/phases/phase-2-completion-report.md

# Review sprint backlog
cat docs/sprints/sprint-001/backlog/sprint-backlog.md

# Start Phase 3 when ready
/cedps-phase3-setup
```

### <success-criteria-met>
**Phase 2 Success Validation**:
- [ ] Features selected align with business priorities
- [ ] Implementation approach is technically sound
- [ ] Timeline is realistic for team capacity
- [ ] Quality standards are comprehensive (>95% coverage)
- [ ] Dependencies are identified and manageable
- [ ] Risk mitigation strategies are actionable
- [ ] Human approval maintained for strategic decisions

**Implementation Readiness**:
- [ ] Sprint backlog provides file-level task specificity
- [ ] Technical approach uses security-first patterns
- [ ] Testing strategy ensures comprehensive coverage
- [ ] Integration patterns support existing systems
- [ ] Quality gates prepared for implementation validation
</success-criteria-met>
</human-action-required>

## <troubleshooting>
### <common-errors>
- **"Phase 2 not found"**: Run `/cedps-phase2-setup` first
- **"AI planning not completed"**: Run `/cedps-phase2-plan` first
- **"Missing approval sections"**: Ensure Claude Code provided all required approval sections
- **"No human approvals found"**: Review and approve implementation plans in planning document
- **"Rejected sections found"**: Address items marked "❌ Requires Revision"
- **"Phase validation tool failed"**: Address specific validation issues reported
</common-errors>

### <quality-validation>
**Phase 2 Validation Requirements**:
- [ ] All required approval sections exist in planning document
- [ ] Human approvals (✅ Approved) are present for strategic decisions
- [ ] Feature selection is specific and business-aligned
- [ ] Implementation approach is technically sound
- [ ] No rejected sections (❌ Requires Revision) remain
- [ ] Sprint backlog extracted for Phase 3 implementation
- [ ] Project state correctly updated to Phase 2 complete

**CE-DPS Standards Compliance**:
- [ ] Sprint planning maintains human strategic authority
- [ ] AI implementation planning is comprehensive and actionable
- [ ] Security and performance considerations integrated
- [ ] Documentation follows LLM optimization patterns
- [ ] Quality gates ensure >95% test coverage
- [ ] File-level specificity provided for implementation
</quality-validation>