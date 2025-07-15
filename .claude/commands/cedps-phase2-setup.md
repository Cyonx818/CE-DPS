---
description: "Initialize Phase 2 sprint planning environment and feature selection template"
allowed-tools: ["bash", "read", "write", "ls"]
---

# <context>CE-DPS Phase 2: Sprint Planning Setup</context>

## <summary priority="high">
Initialize Phase 2 sprint planning with feature selection template, implementation planning environment, and quality gates configuration.

## <method>Phase 2 Environment Initialization</method>

### <implementation>
!echo "🚀 Setting up CE-DPS Phase 2: Sprint Planning..."

# Validate Phase 1 completion
!if [ ! -f "docs/ce-dps-state.json" ]; then
    echo "❌ Error: CE-DPS project not initialized. Run '/cedps-init' first."
    exit 1
fi

!if ! command -v jq >/dev/null 2>&1; then
    echo "⚠️ Warning: jq not found. Cannot validate Phase 1 completion automatically."
    echo "💡 Install jq or manually verify Phase 1 is complete"
    if [ ! -f "docs/phases/phase-1-completion-report.md" ]; then
        echo "❌ Error: Phase 1 completion report not found. Complete Phase 1 first."
        exit 1
    fi
elif ! jq -e '.phases_completed | contains([1])' docs/ce-dps-state.json >/dev/null 2>&1; then
    echo "❌ Error: Phase 1 not completed. Run '/cedps-phase1-validate' first."
    exit 1
fi

# Check if Phase 2 already initialized
!if [ -f "docs/phases/phase-2-sprint-planning.md" ]; then
    echo "⚠️  Phase 2 already initialized. Existing file: docs/phases/phase-2-sprint-planning.md"
    echo "💡 To restart Phase 2, delete the file and run this command again."
    exit 0
fi

# Set Phase 2 environment variables
!export CE_DPS_PHASE=2
!export CE_DPS_FORTITUDE_ENABLED=true
!export CE_DPS_QUALITY_GATES=true
!export CE_DPS_HUMAN_APPROVAL_REQUIRED=true

# Update project state
!if command -v jq >/dev/null 2>&1; then
    jq '.current_phase = 2 | .last_updated = now | .phase_2_started = now' docs/ce-dps-state.json > docs/ce-dps-state.tmp && mv docs/ce-dps-state.tmp docs/ce-dps-state.json
else
    echo "⚠️ Warning: jq not found. State update skipped."
    echo "💡 Install jq for automatic state management"
fi

# Copy Phase 2 template
!if [ ! -f "methodology/templates/phase-2-template.md" ]; then
    echo "❌ Error: Phase 2 template not found at methodology/templates/phase-2-template.md"
    echo "💡 Ensure you're in the CE-DPS project root with complete methodology structure."
    exit 1
fi

!cp methodology/templates/phase-2-template.md docs/phases/phase-2-sprint-planning.md

# Create Phase 2 working directory
!mkdir -p docs/phases/phase-2-artifacts
!mkdir -p docs/sprints/sprint-001

# Extract feature roadmap from Phase 1 for reference
!if grep -A 50 "Feature Roadmap" docs/phases/phase-1-planning.md > docs/phases/phase-2-artifacts/available-features.md; then
    echo "📋 Feature roadmap extracted from Phase 1 planning"
fi

# Initialize sprint tracking
!cat > docs/sprints/sprint-001/sprint-info.json << 'EOF'
{
  "sprint_number": 1,
  "phase": 2,
  "status": "planning",
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "features_selected": [],
  "complexity_analysis": null,
  "implementation_plan": null,
  "human_approvals": []
}
EOF

# Prepare Fortitude for implementation pattern lookup
!if command -v cargo >/dev/null 2>&1; then
    echo "🧠 Preparing Fortitude for implementation pattern lookup..."
    cargo run --bin fortitude-integration -- query "implementation patterns" --quiet 2>/dev/null || echo "⚠️  Fortitude query skipped (optional)"
fi

!echo "✅ Phase 2 environment initialized successfully!"
!echo "📋 Sprint planning template: docs/phases/phase-2-sprint-planning.md"
!echo "🎯 Sprint 1 directory: docs/sprints/sprint-001/"
!echo "🔧 Environment variables configured for Phase 2"
</implementation>

### <constraints>
- Phase 1 must be completed first
- methodology/templates/phase-2-template.md must exist
- docs/ directory must be writable
- jq command required for JSON processing
</constraints>

## <human-action-required>
**Phase 2 Setup Complete! 🚀**

### <next-steps priority="critical">
**You must now select features for Sprint 1**:

1. **Review available features**: Check `docs/phases/phase-2-artifacts/available-features.md`
2. **Open sprint planning template**: `docs/phases/phase-2-sprint-planning.md`
3. **Complete feature selection section**:

### <feature-selection-process>
**Review Phase 1 Feature Roadmap**:
- Open `docs/phases/phase-1-planning.md` 
- Find the "Feature Roadmap" section from AI analysis
- Note feature priorities and dependencies

**Select Sprint 1 Features**:
```markdown
### Selected Features for Sprint
[Choose 2-4 features from the roadmap based on:]
- **Business Priority**: High-impact features first
- **Technical Dependencies**: Features that enable other features
- **Team Capacity**: Realistic scope for sprint timeline
- **User Value**: Features that deliver immediate user benefit

Example:
1. User Authentication System (HIGH) - Foundation for all other features
2. Basic User Profile Management (MEDIUM) - Enables user personalization
3. Core API Endpoints (HIGH) - Required for frontend integration
```

**Business Priority Input**:
```markdown
### Business Priority Input
[Provide current business context:]
- **Immediate Needs**: What must be delivered first?
- **User Feedback**: Any user research or feedback priorities?
- **Market Timing**: Time-sensitive features or opportunities?
- **Resource Constraints**: Development team capacity and timeline?
```

### <template-sections-required>
**Complete these sections in phase-2-sprint-planning.md**:
- [ ] **Available Features from Roadmap**: List all features from Phase 1
- [ ] **Business Priority Input**: Current business context and constraints
- [ ] **Selected Features for Sprint**: 2-4 features chosen for implementation
- [ ] **Sprint Goal**: High-level objective for this sprint
- [ ] **Duration**: Realistic timeline for selected features

### <validation-checklist>
**Before proceeding, ensure**:
- [ ] Phase 1 feature roadmap reviewed
- [ ] Business priorities clearly defined
- [ ] 2-4 features selected (not too many for first sprint)
- [ ] Sprint goal is clear and measurable
- [ ] Timeline is realistic for team capacity
- [ ] Dependencies between features identified

### <ready-to-proceed>
**When feature selection is complete, run**:
```bash
/cedps-phase2-plan
```

This will trigger Claude Code to perform detailed implementation planning for your selected features.
</ready-to-proceed>
</human-action-required>

## <troubleshooting>
### <common-errors>
- **"Phase 1 not completed"**: Run `/cedps-phase1-validate` first
- **"Phase 2 template not found"**: Ensure you're in CE-DPS project root
- **"Permission denied"**: Check docs/ directory write permissions
- **"jq: command not found"**: Install jq for JSON processing
- **"Phase 2 already initialized"**: Delete existing file to restart
</common-errors>

### <quality-validation>
**Phase 2 Setup Requirements**:
- [ ] Phase 1 completion validated
- [ ] Sprint planning template copied successfully
- [ ] Sprint tracking initialized
- [ ] Environment variables set for Phase 2
- [ ] Feature roadmap extracted from Phase 1
- [ ] Fortitude prepared for pattern lookup

**CE-DPS Standards Compliance**:
- [ ] Human feature selection authority maintained
- [ ] LLM-optimized semantic markup used
- [ ] Progressive disclosure implemented
- [ ] Quality gates preparation complete
- [ ] Security considerations carried forward from Phase 1
</quality-validation>