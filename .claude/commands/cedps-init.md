---
description: "Initialize a new CE-DPS project with environment setup and documentation structure"
allowed-tools: ["bash", "read", "write", "ls"]
---

# <context>CE-DPS Project Initialization</context>

## <summary priority="high">
Initialize new CE-DPS project with complete environment setup, documentation structure, and quality gates configuration.

## <method>Comprehensive Project Setup</method>

### <implementation>
!echo "🚀 Initializing CE-DPS project..."

# Validate we're in the right location
!if [ ! -f "CLAUDE.md" ]; then echo "❌ Error: CLAUDE.md not found. Run this command in the CE-DPS project root."; exit 1; fi

# Check dependencies and provide helpful guidance
!echo "🔍 Checking system dependencies..."
!MISSING_DEPS=""

# Check for jq (recommended for state management)
!if ! command -v jq >/dev/null 2>&1; then
    echo "⚠️  jq not found (recommended for automatic state management)"
    echo "   Install: sudo apt-get install jq  # or brew install jq"
    MISSING_DEPS="${MISSING_DEPS}jq "
fi

# Check for git (required for branch management)
!if ! command -v git >/dev/null 2>&1; then
    echo "❌ git not found (required for CE-DPS workflow)"
    echo "   Install: sudo apt-get install git  # or download from https://git-scm.com/"
    MISSING_DEPS="${MISSING_DEPS}git "
fi

# Check if we're in a git repository
!if ! git rev-parse --git-dir >/dev/null 2>&1; then
    echo "⚠️  Not in a git repository (recommended for CE-DPS)"
    echo "   Initialize: git init && git add . && git commit -m 'Initial commit'"
fi

# Check for python3 (optional for phase validator)
!if ! command -v python3 >/dev/null 2>&1; then
    echo "⚠️  python3 not found (optional for phase validation tools)"
    echo "   Install: sudo apt-get install python3"
    MISSING_DEPS="${MISSING_DEPS}python3 "
fi

# Summary message
!if [ -n "$MISSING_DEPS" ]; then
    echo ""
    echo "💡 Some dependencies are missing but CE-DPS will still work"
    echo "   Missing: $MISSING_DEPS"
    echo "   CE-DPS will provide fallback functionality where possible"
else
    echo "✅ All dependencies found"
fi
!echo ""

# Create project documentation structure
!mkdir -p docs/phases
!mkdir -p docs/architecture
!mkdir -p docs/sprints
!mkdir -p docs/quality-reports

# Set initial environment variables
!export CE_DPS_PHASE=0
!export CE_DPS_FORTITUDE_ENABLED=true
!export CE_DPS_QUALITY_GATES=true
!export CE_DPS_HUMAN_APPROVAL_REQUIRED=true

# Initialize project state file
!cat > docs/ce-dps-state.json << 'EOF'
{
  "project_initialized": true,
  "current_phase": 0,
  "phases_completed": [],
  "quality_gates_enabled": true,
  "fortitude_enabled": true,
  "human_approval_required": true,
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

# Create project README if it doesn't exist
!if [ ! -f "docs/PROJECT.md" ]; then
cat > docs/PROJECT.md << 'EOF'
# CE-DPS Project

## Overview
This project follows the CE-DPS (Context Engineered Development Process Suite) methodology for AI-assisted development with human strategic oversight.

## Development Phases
1. **Phase 1: Strategic Planning** - Define vision, approve architecture
2. **Phase 2: Sprint Planning** - Select features, create implementation plans  
3. **Phase 3: Implementation** - Execute code development with quality gates

## Current Status
- **Phase**: Not started
- **Next Action**: Run `/cedps-phase1-setup` to begin strategic planning

## Quality Standards
- >95% test coverage required
- Security-first implementation patterns
- Comprehensive documentation with LLM optimization
- Human approval required for all strategic decisions

## Tools Integration
- **Fortitude**: Knowledge management and pattern lookup
- **Quality Gates**: Automated testing and validation
- **Phase Validator**: Completion criteria verification
EOF
fi

!echo "✅ CE-DPS project initialized successfully!"
!echo "📁 Documentation structure created in docs/"
!echo "🔧 Environment variables configured"
!echo "📊 Project state tracking enabled"
</implementation>

### <constraints>
- Must be run from CE-DPS project root directory
- Requires CLAUDE.md file to exist
- docs/ directory must be writable
- Shell environment must support export command
</constraints>

## <human-action-required>
**Project Successfully Initialized! 🎉**

### <next-steps>
**Immediate Actions**:
1. Review the project structure created in `docs/`
2. Customize `docs/PROJECT.md` with your specific project details
3. Verify environment variables are set correctly

**To Begin Development**:
```bash
# Check current project status
/cedps-status

# Start Phase 1: Strategic Planning
/cedps-phase1-setup
```

**Project Structure Created**:
```
docs/
├── PROJECT.md              # Project overview and status
├── ce-dps-state.json      # Project state tracking
├── phases/                # Phase documentation
├── architecture/          # System design docs
├── sprints/              # Sprint planning docs
└── quality-reports/      # Quality gate results
```

**Environment Variables Set**:
- `CE_DPS_PHASE=0` (initialization complete)
- `CE_DPS_FORTITUDE_ENABLED=true`
- `CE_DPS_QUALITY_GATES=true`
- `CE_DPS_HUMAN_APPROVAL_REQUIRED=true`

### <validation-checklist>
- [ ] `docs/` directory structure exists
- [ ] `docs/PROJECT.md` contains project overview
- [ ] `docs/ce-dps-state.json` shows initialization complete
- [ ] Environment variables are properly set
- [ ] Ready to proceed to Phase 1
</validation-checklist>
</human-action-required>

## <troubleshooting>
### <common-errors>
- **"CLAUDE.md not found"**: Run command from CE-DPS project root
- **"Permission denied"**: Check directory write permissions
- **"mkdir: File exists"**: Safe to ignore - directories already exist
- **Environment variables not persisting**: Add to your shell profile (.bashrc, .zshrc)
</common-errors>

### <quality-validation>
- [ ] Project follows CE-DPS documentation standards
- [ ] LLM-optimized semantic markup used
- [ ] Progressive disclosure implemented
- [ ] Security considerations documented
- [ ] Human approval points clearly marked
</quality-validation>