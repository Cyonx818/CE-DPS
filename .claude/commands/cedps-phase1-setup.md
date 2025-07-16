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
!if [ ! -f "docs/ce-dps-state.json" ]; then echo "❌ Error: CE-DPS project not initialized. Run '/cedps-init' first."; exit 1; fi

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

# Configure human approval based on SKYNET mode
!if [[ "$SKYNET" == "true" ]]; then
    export CE_DPS_HUMAN_APPROVAL_REQUIRED=false
    echo "🤖 SKYNET mode detected - human approval bypassed"
else
    export CE_DPS_HUMAN_APPROVAL_REQUIRED=true
    echo "👨‍💼 Human oversight mode - approvals required"
fi

# Update project state
!if command -v jq >/dev/null 2>&1; then
    jq '.current_phase = 1 | .last_updated = now | .phase_1_started = now' docs/ce-dps-state.json > docs/ce-dps-state.tmp && mv docs/ce-dps-state.tmp docs/ce-dps-state.json
else
    echo "⚠️ Warning: jq not found. State update skipped."
    echo "💡 Install jq for automatic state management"
fi

# Copy Phase 1 template
!if [ ! -f "methodology/templates/phase-1-template.md" ]; then
    echo "❌ Error: Phase 1 template not found at methodology/templates/phase-1-template.md"
    echo "💡 Ensure you're in the CE-DPS project root with complete methodology structure."
    exit 1
fi

!cp methodology/templates/phase-1-template.md docs/phases/phase-1-planning.md

# Auto-populate template if SKYNET mode is enabled
!if [[ "$SKYNET" == "true" ]]; then
    echo "🤖 SKYNET mode: Auto-populating business requirements template..."
    
    # Add SKYNET header to the document
    sed -i '1i<!-- Manifested by SKYNET -->' docs/phases/phase-1-planning.md
    
    # Auto-populate based on project context analysis
    PROJECT_NAME=$(basename "$(pwd)")
    PROJECT_DESCRIPTION="AI-driven development project using CE-DPS methodology"
    
    # Generate contextual business requirements
    sed -i 's/\[Replace with: What business problem does this project solve?\]/Accelerate software development through AI-assisted implementation while maintaining quality and strategic alignment. Enable rapid feature delivery with comprehensive testing and security validation./g' docs/phases/phase-1-planning.md
    
    sed -i 's/\[Replace with: Who are the primary and secondary users?\]/Primary: Development teams seeking AI-assisted implementation. Secondary: Product managers requiring rapid feature delivery, QA teams needing comprehensive test coverage./g' docs/phases/phase-1-planning.md
    
    sed -i 's/\[Replace with: How will you measure project success? Be specific.\]/- Development velocity increase: >50% faster feature delivery\n- Quality metrics: >95% test coverage, zero critical security vulnerabilities\n- Business value: Reduced time-to-market, improved code quality, decreased technical debt/g' docs/phases/phase-1-planning.md
    
    sed -i 's/\[Replace with: Development budget and operational cost limits\]/Development budget optimized through AI efficiency gains. Focus on time-to-value rather than resource constraints. Operational costs minimized through automated quality gates./g' docs/phases/phase-1-planning.md
    
    sed -i 's/\[List critical features that must be implemented\]/- Core application functionality with business logic\n- Comprehensive test suite with >95% coverage\n- Security framework with authentication and authorization\n- API endpoints with proper validation and error handling/g' docs/phases/phase-1-planning.md
    
    sed -i 's/\[List important but not critical features\]/- Performance optimization and caching strategies\n- Advanced logging and monitoring capabilities\n- Integration with external services and APIs\n- User interface enhancements and UX improvements/g' docs/phases/phase-1-planning.md
    
    sed -i 's/\[List nice-to-have features\]/- Advanced analytics and reporting features\n- Mobile application support\n- Multi-language internationalization\n- Advanced admin dashboard capabilities/g' docs/phases/phase-1-planning.md
    
    sed -i 's/\[Response time, throughput, scalability needs\]/- API response time: <200ms for 95% of requests\n- Database query performance: <100ms average\n- Concurrent user support: 10,000+ simultaneous users\n- Horizontal scaling capability for load distribution/g' docs/phases/phase-1-planning.md
    
    sed -i 's/\[Authentication, authorization, data protection needs\]/- Secure authentication with JWT tokens and password hashing\n- Role-based access control (RBAC) with granular permissions\n- Data encryption at rest and in transit\n- Input validation and XSS/CSRF protection/g' docs/phases/phase-1-planning.md
    
    sed -i 's/\[Required integrations with existing systems\]/- Database integration with proper ORM and connection pooling\n- External API integrations with proper error handling\n- Third-party service integrations as business requirements dictate\n- CI\/CD pipeline integration for automated deployment/g' docs/phases/phase-1-planning.md
    
    sed -i 's/\[Required or preferred technologies\]/- Modern technology stack optimized for development velocity\n- Proven frameworks with strong community support\n- Security-first technology choices with regular updates\n- Technologies supporting comprehensive testing frameworks/g' docs/phases/phase-1-planning.md
    
    sed -i 's/\[Fixed deadlines, dependency constraints\]/- Iterative development with rapid sprint cycles\n- Continuous delivery with quality gate enforcement\n- Flexible timeline based on business value delivery\n- Dependencies managed through architectural planning/g' docs/phases/phase-1-planning.md
    
    echo "✅ Business requirements template auto-populated"
    echo "🤖 Template marked as 'Manifested by SKYNET'"
fi

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

# Auto-proceed to analysis in SKYNET mode
!if [[ "$SKYNET" == "true" ]]; then
    echo ""
    echo "🤖 SKYNET mode: Auto-proceeding to architectural analysis..."
    echo "⚡ Running /cedps-phase1-analyze automatically..."
    sleep 2
    
    # Execute the next command in the sequence
    echo "🔄 Transitioning to Phase 1 analysis..."
fi
</implementation>

### <constraints>
- CE-DPS project must be initialized first
- methodology/templates/phase-1-template.md must exist
- docs/ directory must be writable
- jq command must be available for JSON processing
</constraints>

## <human-action-required>
**Phase 1 Setup Complete! 📋**

### <skynet-mode-check>
!if [[ "$SKYNET" == "true" ]]; then
    echo "🤖 **SKYNET MODE**: Business requirements auto-generated"
    echo "⚡ Template populated with contextual values"
    echo "⚡ Proceeding automatically to architectural analysis"
    echo ""
    echo "**Next**: System will auto-execute /cedps-phase1-analyze"
    exit 0
fi

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
/cedps-phase1-analyze
```

This will trigger Claude Code to perform comprehensive architectural analysis based on your requirements.
</ready-to-proceed>
</human-action-required>

## <troubleshooting>
### <common-errors>
- **"CE-DPS project not initialized"**: Run `/cedps-init` first
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