---
description: "Initialize Phase 1 strategic planning environment and business requirements template"
allowed-tools: ["bash", "read", "write", "ls"]
---

# <context>CE-DPS Phase 1: Strategic Planning Setup</context>

## <summary priority="high">
Initialize Phase 1 strategic planning with business requirements template, environment configuration, and quality gates preparation.

## <method>Phase 1 Environment Initialization</method>

### <implementation>
!echo "📋 Setting up CE-DPS Phase 1: Strategic Planning..."

# Validate project initialization
!if [ ! -f "docs/ce-dps-state.json" ]; then echo "❌ Error: CE-DPS project not initialized. Run '/cedps init' first."; exit 1; fi

# Check if already in Phase 1
!if [ -f "docs/phases/phase-1-planning.md" ]; then
    echo "⚠️  Phase 1 already initialized. Existing file: docs/phases/phase-1-planning.md"
    echo "💡 To restart Phase 1, delete the file and run this command again."
    exit 0
fi

# Set Phase 1 environment variables
!export CE_DPS_PHASE=1
!export CE_DPS_FORTITUDE_ENABLED=true
!export CE_DPS_QUALITY_GATES=true
!export CE_DPS_HUMAN_APPROVAL_REQUIRED=true

# Update project state
!jq '.current_phase = 1 | .last_updated = now | .phase_1_started = now' docs/ce-dps-state.json > docs/ce-dps-state.tmp && mv docs/ce-dps-state.tmp docs/ce-dps-state.json

# Copy Phase 1 template
!if [ ! -f "methodology/templates/phase-1-template.md" ]; then
    echo "❌ Error: Phase 1 template not found at methodology/templates/phase-1-template.md"
    echo "💡 Ensure you're in the CE-DPS project root with complete methodology structure."
    exit 1
fi

!cp methodology/templates/phase-1-template.md docs/phases/phase-1-planning.md

# Initialize Fortitude for pattern lookup
!if command -v cargo >/dev/null 2>&1; then
    echo "🧠 Initializing Fortitude knowledge management..."
    cargo run --bin fortitude-integration -- init --quiet 2>/dev/null || echo "⚠️  Fortitude initialization skipped (optional)"
fi

# Create Phase 1 working directory
!mkdir -p docs/phases/phase-1-artifacts

!echo "✅ Phase 1 environment initialized successfully!"
!echo "📋 Business requirements template: docs/phases/phase-1-planning.md"
!echo "🔧 Environment variables configured for Phase 1"
!echo "🧠 Fortitude integration prepared"
</implementation>

### <constraints>
- CE-DPS project must be initialized first
- methodology/templates/phase-1-template.md must exist
- docs/ directory must be writable
- jq command must be available for JSON processing
</constraints>

## <human-action-required>
**Phase 1 Setup Complete! 📋**

### <next-steps priority="critical">
**You must now fill out the business requirements template**:

1. **Open the template**: `docs/phases/phase-1-planning.md`
2. **Complete ALL required sections**:
   - **Business Context**: Problem statement, target users, success metrics
   - **Strategic Requirements**: Must-have features, technical requirements
   - **Constraints**: Technology stack, timeline, budget limitations

### <template-sections-required>
**Business Context Section**:
```markdown
### Problem Statement
[Replace with: What business problem does this project solve?]

### Target Users  
[Replace with: Who are the primary and secondary users?]

### Success Metrics
[Replace with: How will you measure project success? Be specific.]

### Budget Constraints
[Replace with: Development budget and operational cost limits]
```

**Strategic Requirements Section**:
```markdown
### Business Requirements
- **Must-Have Features**: [List critical features that must be implemented]
- **Should-Have Features**: [List important but not critical features]
- **Could-Have Features**: [List nice-to-have features]

### Technical Requirements
- **Performance**: [Response time, throughput, scalability needs]
- **Security**: [Authentication, authorization, data protection needs]
- **Integration**: [Required integrations with existing systems]

### Constraints
- **Technology Stack**: [Required or preferred technologies]
- **Timeline**: [Fixed deadlines, dependency constraints]
```

### <validation-checklist>
**Before proceeding, ensure**:
- [ ] All `[Enter...]` placeholders are replaced with actual content
- [ ] Problem statement is clear and specific
- [ ] Success metrics are measurable
- [ ] Technical requirements are realistic
- [ ] Constraints are clearly defined
- [ ] Budget considerations are included

### <ready-to-proceed>
**When template is complete, run**:
```bash
/cedps phase1 analyze
```

This will trigger Claude Code to perform comprehensive architectural analysis based on your requirements.
</ready-to-proceed>
</human-action-required>

## <troubleshooting>
### <common-errors>
- **"CE-DPS project not initialized"**: Run `/cedps init` first
- **"Phase 1 template not found"**: Ensure you're in CE-DPS project root
- **"Permission denied"**: Check docs/ directory write permissions
- **"jq: command not found"**: Install jq for JSON processing
- **"Phase 1 already initialized"**: Delete existing file to restart
</common-errors>

### <quality-validation>
**Phase 1 Setup Requirements**:
- [ ] Template successfully copied to docs/phases/
- [ ] Environment variables set correctly
- [ ] Project state updated to Phase 1
- [ ] Fortitude integration prepared
- [ ] Human given clear next-step instructions

**CE-DPS Standards Compliance**:
- [ ] LLM-optimized semantic markup used
- [ ] Progressive disclosure implemented
- [ ] Security considerations documented
- [ ] Human approval points clearly marked
- [ ] Quality gate preparation complete
</quality-validation>