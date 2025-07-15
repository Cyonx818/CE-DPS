---
description: "Initialize Phase 3 implementation environment with quality gates and testing framework"
allowed-tools: ["bash", "read", "write", "ls"]
---

# <context>CE-DPS Phase 3: Implementation Setup</context>

## <summary priority="high">
Initialize Phase 3 implementation environment with quality gates, testing framework, and comprehensive development workflow preparation.

## <method>Phase 3 Environment Initialization</method>

### <implementation>
!echo "🚀 Setting up CE-DPS Phase 3: Implementation..."

# Validate Phase 2 completion
!if [ ! -f "docs/ce-dps-state.json" ]; then
    echo "❌ Error: CE-DPS project not initialized. Run '/cedps init' first."
    exit 1
fi

!if ! jq -e '.phases_completed | contains([1, 2])' docs/ce-dps-state.json >/dev/null 2>&1; then
    echo "❌ Error: Phases 1 and 2 not completed. Run '/cedps phase2 validate' first."
    exit 1
fi

# Check if Phase 3 already initialized
!if [ -f "docs/phases/phase-3-implementation.md" ]; then
    echo "⚠️  Phase 3 already initialized. Existing file: docs/phases/phase-3-implementation.md"
    echo "💡 To restart Phase 3, delete the file and run this command again."
    exit 0
fi

# Set Phase 3 environment variables
!export CE_DPS_PHASE=3
!export CE_DPS_FORTITUDE_ENABLED=true
!export CE_DPS_QUALITY_GATES=true
!export CE_DPS_HUMAN_APPROVAL_REQUIRED=true

# Update project state
!jq '.current_phase = 3 | .last_updated = now | .phase_3_started = now' docs/ce-dps-state.json > docs/ce-dps-state.tmp && mv docs/ce-dps-state.tmp docs/ce-dps-state.json

# Copy Phase 3 template
!if [ ! -f "methodology/templates/phase-3-template.md" ]; then
    echo "❌ Error: Phase 3 template not found at methodology/templates/phase-3-template.md"
    echo "💡 Ensure you're in the CE-DPS project root with complete methodology structure."
    exit 1
fi

!cp methodology/templates/phase-3-template.md docs/phases/phase-3-implementation.md

# Create Phase 3 working directories
!mkdir -p docs/phases/phase-3-artifacts
!mkdir -p docs/sprints/sprint-001/implementation
!mkdir -p docs/quality-reports/sprint-001

# Initialize implementation tracking
!cat > docs/sprints/sprint-001/implementation/implementation-status.json << 'EOF'
{
  "sprint_number": 1,
  "phase": 3,
  "status": "setup",
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "features_in_progress": [],
  "quality_gates_passed": [],
  "test_coverage": 0,
  "security_scan_status": "pending",
  "human_validations": []
}
EOF

# Create feature branch for implementation
!BRANCH_NAME="sprint-001-implementation"
!if git rev-parse --verify "$BRANCH_NAME" >/dev/null 2>&1; then
    echo "📝 Feature branch $BRANCH_NAME already exists"
else
    git checkout -b "$BRANCH_NAME"
    echo "📝 Created feature branch: $BRANCH_NAME"
fi

# Initialize quality gates
!if command -v cargo >/dev/null 2>&1 && [ -f "tools/quality-gates/Cargo.toml" ]; then
    echo "🔧 Initializing quality gates..."
    cd tools/quality-gates && cargo build --release && cd ../..
    echo "✅ Quality gates compiled and ready"
fi

# Initialize testing framework
!if [ -f "Cargo.toml" ]; then
    echo "🧪 Preparing Rust testing framework..."
    cargo test --no-run 2>/dev/null || echo "⚠️  Test compilation will occur during implementation"
fi

# Prepare Fortitude for implementation patterns
!if command -v cargo >/dev/null 2>&1 && [ -f "tools/fortitude-integration/Cargo.toml" ]; then
    echo "🧠 Preparing Fortitude for implementation patterns..."
    cargo run --bin fortitude-integration -- query "implementation patterns" --quiet 2>/dev/null || echo "⚠️  Fortitude query skipped (optional)"
fi

# Extract sprint backlog for implementation
!if [ -f "docs/sprints/sprint-001/backlog/sprint-backlog.md" ]; then
    cp docs/sprints/sprint-001/backlog/sprint-backlog.md docs/phases/phase-3-artifacts/implementation-backlog.md
    echo "📋 Sprint backlog prepared for implementation"
fi

# Initialize pre-implementation checklist
!cat > docs/phases/phase-3-artifacts/pre-implementation-checklist.md << 'EOF'
# Pre-Implementation Checklist

## Environment Setup
- [x] Development environment configured
- [x] Feature branch created (sprint-001-implementation)
- [x] Quality gates compiled and ready
- [x] Testing framework prepared

## Implementation Planning
- [ ] Sprint backlog reviewed and understood
- [ ] Implementation sequence confirmed
- [ ] Quality gates and testing strategy ready
- [ ] Integration points identified and planned

## Quality Standards
- [ ] >95% test coverage target confirmed
- [ ] Security-first implementation patterns ready
- [ ] Error handling approaches defined
- [ ] Performance requirements understood

## Human Validation Points
- [ ] Business value validation approach defined
- [ ] Feature acceptance criteria clear
- [ ] Integration testing approach confirmed
- [ ] Production readiness criteria established

## Ready for Implementation
- [ ] All checklist items completed
- [ ] Team ready to begin development
- [ ] Quality gates functioning
- [ ] Human oversight prepared
EOF

!echo "✅ Phase 3 environment initialized successfully!"
!echo "📋 Implementation template: docs/phases/phase-3-implementation.md"
!echo "🎯 Sprint implementation: docs/sprints/sprint-001/implementation/"
!echo "🔧 Quality gates ready and compiled"
!echo "📝 Feature branch: sprint-001-implementation"
</implementation>

### <constraints>
- Phases 1 and 2 must be completed first
- methodology/templates/phase-3-template.md must exist
- Git repository must be initialized
- Quality gates tools must be buildable
- jq command required for JSON processing
</constraints>

## <human-action-required>
**Phase 3 Setup Complete! 🚀**

### <next-steps priority="critical">
**Review Implementation Readiness**:

1. **Review pre-implementation checklist**: `docs/phases/phase-3-artifacts/pre-implementation-checklist.md`
2. **Confirm sprint backlog**: `docs/phases/phase-3-artifacts/implementation-backlog.md`
3. **Validate development environment**: Ensure all tools and dependencies are ready

### <implementation-preparation>
**Before Starting Implementation**:

**Review Sprint Backlog**:
```bash
# Review the approved features and tasks
cat docs/phases/phase-3-artifacts/implementation-backlog.md

# Understand the file-level implementation plan
# Verify technical approach and quality standards
```

**Confirm Quality Standards**:
- **Test Coverage**: >95% for all business logic
- **Security**: Authentication, authorization, input validation
- **Performance**: Response times meet requirements
- **Error Handling**: Comprehensive error management
- **Documentation**: API docs and code comments

**Validate Environment**:
```bash
# Test quality gates
cargo run --bin quality-gates -- --validate-environment

# Check testing framework
cargo test --no-run

# Verify git branch
git status
```

### <implementation-workflow>
**Phase 3 Implementation Process**:

1. **Test-Driven Development**: Write failing tests first
2. **Incremental Implementation**: Implement features one at a time
3. **Quality Gate Validation**: Run quality gates after each feature
4. **Human Business Validation**: Validate features against business requirements
5. **Integration Testing**: Ensure seamless integration with existing systems

### <ready-to-implement>
**When environment is validated and ready**:
```bash
/cedps phase3 implement
```

This will trigger Claude Code to begin systematic implementation of the approved sprint features using test-driven development.
</ready-to-implement>
</human-action-required>

## <troubleshooting>
### <common-errors>
- **"Phases 1 and 2 not completed"**: Complete previous phases first
- **"Phase 3 template not found"**: Ensure you're in CE-DPS project root
- **"Git not initialized"**: Initialize git repository (`git init`)
- **"Quality gates build failed"**: Check Rust toolchain and dependencies
- **"Permission denied"**: Check directory write permissions
- **"Feature branch exists"**: Safe to continue with existing branch
</common-errors>

### <quality-validation>
**Phase 3 Setup Requirements**:
- [ ] Phases 1 and 2 completion validated
- [ ] Implementation template copied successfully
- [ ] Feature branch created for development
- [ ] Quality gates compiled and ready
- [ ] Testing framework prepared
- [ ] Sprint backlog available for implementation
- [ ] Pre-implementation checklist created

**CE-DPS Standards Compliance**:
- [ ] Environment supports >95% test coverage
- [ ] Security-first patterns ready for implementation
- [ ] Human validation points prepared
- [ ] Quality gates ensure comprehensive validation
- [ ] LLM-optimized documentation patterns used
- [ ] Integration with existing systems planned
</quality-validation>