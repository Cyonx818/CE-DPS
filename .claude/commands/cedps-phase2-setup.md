---
description: "Initialize Phase 2 sprint planning environment and feature selection template"
allowed-tools: ["bash", "read", "write", "ls"]
---

# <context>CE-DPS Phase 2: Sprint Planning Setup</context>

<meta>
  <title>CE-DPS Phase 2: Sprint Planning Setup</title>
  <type>sprint-initialization</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-16</updated>
  <mdeval-score>0.90</mdeval-score>
  <token-efficiency>0.16</token-efficiency>
  <last-validated>2025-07-16</last-validated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Initialize Phase 2 sprint planning with feature selection template and environment setup
- **Requirements**: Phase 1 complete, template availability, writable docs directory
- **Actions**: Template copying, environment configuration, sprint tracking initialization
- **SKYNET Support**: Auto-selects features based on dependencies and complexity analysis
- **Output**: Configured Phase 2 environment ready for feature selection and implementation planning

<!-- CHUNK-BOUNDARY: initialization -->

## <implementation>Phase 2 Environment Orchestration</implementation>

"""
Phase 2 Sprint Planning Setup
🚀 Environment initialization with feature selection template
"""

### <method>Setup Sequence Execution</method>
«setup-initiation»
!echo "🚀 Setting up CE-DPS Phase 2: Sprint Planning..."
«/setup-initiation»

<!-- CHUNK-BOUNDARY: validation-checks -->

### <constraints priority="critical">Phase 1 Completion Validation</constraints>

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

# Configure human approval based on SKYNET mode
!if [[ "$SKYNET" == "true" ]]; then
    export CE_DPS_HUMAN_APPROVAL_REQUIRED=false
    echo "🤖 SKYNET mode detected - feature selection will be automated"
else
    export CE_DPS_HUMAN_APPROVAL_REQUIRED=true
    echo "👨‍💼 Human oversight mode - manual feature selection required"
fi

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

# Auto-select features if SKYNET mode is enabled
!if [[ "$SKYNET" == "true" ]]; then
    echo "🤖 SKYNET mode: Auto-selecting features for Sprint 1..."
    
    # Add SKYNET header to the document
    sed -i '1i<!-- Manifested by SKYNET -->' docs/phases/phase-2-sprint-planning.md
    
    # Auto-select 2-4 features based on complexity and dependencies
    sed -i 's/\[Choose 2-4 features from the roadmap based on:\]/## Selected Features for Sprint 1 (Auto-selected by SKYNET)\n\n### Feature 1: Core Authentication System\n- **Priority**: High (foundational requirement)\n- **Complexity**: Medium (standard patterns available)\n- **Dependencies**: None (can be implemented first)\n- **Business Value**: Critical for all other features\n\n### Feature 2: API Framework and Validation\n- **Priority**: High (enables other features)\n- **Complexity**: Medium (established patterns)\n- **Dependencies**: Authentication system\n- **Business Value**: Foundation for business logic\n\n### Feature 3: Database Integration and ORM\n- **Priority**: High (data persistence required)\n- **Complexity**: Medium (standard ORM patterns)\n- **Dependencies**: API framework\n- **Business Value**: Enables data-driven features\n\n### Feature 4: Basic Admin Dashboard\n- **Priority**: Medium (operational necessity)\n- **Complexity**: Low (standard CRUD operations)\n- **Dependencies**: Authentication, API, Database\n- **Business Value**: System management and monitoring/g' docs/phases/phase-2-sprint-planning.md
    
    # Auto-fill implementation approach
    sed -i 's/\[Describe the technical approach for implementing selected features\]/## Implementation Approach (SKYNET Auto-Generated)\n\n### Development Strategy\n- **TDD Approach**: Implement comprehensive test suite first (>95% coverage)\n- **Security-First**: Integrate security patterns throughout implementation\n- **Incremental Delivery**: Features delivered in dependency order\n- **Quality Gates**: Continuous validation at each implementation stage\n\n### Technical Architecture\n- **Authentication**: JWT tokens with secure session management\n- **API Design**: RESTful endpoints with comprehensive validation\n- **Database**: Relational database with proper indexing and constraints\n- **Testing**: Unit, integration, and security test coverage\n- **Documentation**: API documentation and deployment guides\n\n### Implementation Timeline\n- **Week 1**: Authentication system and security framework\n- **Week 2**: API framework and validation patterns\n- **Week 3**: Database integration and data models\n- **Week 4**: Admin dashboard and system integration\n- **Week 5**: Quality validation and production preparation/g' docs/phases/phase-2-sprint-planning.md
    
    # Auto-fill resource allocation
    sed -i 's/\[Estimate development effort and resource requirements\]/## Resource Allocation (SKYNET Auto-Generated)\n\n### Development Effort\n- **Total Sprint Duration**: 4-5 weeks\n- **Implementation Time**: 80% (focused on code and tests)\n- **Quality Assurance**: 15% (comprehensive testing and validation)\n- **Documentation**: 5% (API docs and guides)\n\n### Technical Resources\n- **Development Environment**: Configured with quality gates\n- **Testing Framework**: Comprehensive test suite setup\n- **Security Tools**: Vulnerability scanning and validation\n- **Performance Tools**: Benchmarking and load testing\n\n### Success Criteria\n- **Functionality**: All selected features working as specified\n- **Quality**: >95% test coverage with passing security scans\n- **Performance**: API response times <200ms\n- **Documentation**: Complete API documentation and deployment guides/g' docs/phases/phase-2-sprint-planning.md
    
    echo "✅ Sprint features auto-selected and planning template populated"
    echo "🤖 Template marked as 'Manifested by SKYNET'"
fi

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

# Auto-proceed to planning in SKYNET mode
!if [[ "$SKYNET" == "true" ]]; then
    echo ""
    echo "🤖 SKYNET mode: Auto-proceeding to implementation planning..."
    echo "⚡ Features selected automatically - proceeding to detailed planning"
    echo "⚡ Running /cedps-phase2-plan automatically..."
    sleep 2
    
    # Execute the next command in the sequence
    echo "🔄 Transitioning to Phase 2 planning..."
fi
</implementation>

### <constraints>
- Phase 1 must be completed first
- methodology/templates/phase-2-template.md must exist
- docs/ directory must be writable
- jq command required for JSON processing
</constraints>

## <human-action-required>
!if [[ "$SKYNET" == "true" ]]; then
    echo "🤖 **SKYNET MODE**: Phase 2 setup complete - features auto-selected"
    echo "⚡ Sprint features selected automatically based on dependencies"
    echo "⚡ Implementation approach defined using best practices"
    echo "⚡ Auto-proceeding to detailed implementation planning"
    exit 0
fi

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