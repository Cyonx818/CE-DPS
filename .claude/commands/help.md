# CE-DPS Help

Show comprehensive help for CE-DPS methodology and available commands.

## Instructions

1. **Display Overview**
   - Explain CE-DPS methodology purpose and phases
   - Show the AI-as-implementer philosophy
   - Explain human-AI collaboration model

2. **List Available Commands**
   - **Phase 1 Commands**:
     - `/phase1:setup` - Initialize strategic planning
     - `/phase1:analyze` - Analyze business requirements  
     - `/phase1:validate` - Validate phase completion
   - **Phase 2 Commands**:
     - `/phase2:setup` - Initialize sprint planning
     - `/phase2:plan` - Create implementation plan
     - `/phase2:validate` - Validate phase completion
   - **Phase 3 Commands**:
     - `/phase3:setup` - Initialize implementation
     - `/phase3:implement` - Execute TDD implementation
     - `/phase3:validate` - Validate phase completion
   - **SKYNET Commands**:
     - `/skynet:enable` - Enable autonomous operation
     - `/skynet:disable` - Restore human oversight
     - `/skynet:status` - Check current mode
   - **Utility Commands**:
     - `/tools` - Run quality gates and validation
     - `/quality-check` - Complete CI/CD test suite
     - `/init` - Initialize new CE-DPS project

3. **Explain Workflow**
   - Phase 1: Strategic Planning (Human-led with AI analysis)
   - Phase 2: Sprint Development (AI-led with human approval)  
   - Phase 3: Code Implementation (AI-led with human validation)

4. **Show Quality Standards**
   - >95% test coverage requirement
   - Security-first implementation
   - Performance targets (<200ms response time)
   - Comprehensive documentation

5. **Provide Troubleshooting**
   - Common issues and solutions
   - When to use each command
   - How to recover from errors

## Expected Output

```
🛠️ CE-DPS Methodology Help
==========================

📋 Overview
-----------
CE-DPS (Comprehensive Engineering Development Process System) is a three-phase 
methodology for AI-assisted software development with human strategic oversight.

Phase 1: Strategic Planning - Define vision, approve architecture
Phase 2: Sprint Development - Select features, plan implementation  
Phase 3: Code Implementation - TDD development with quality gates

🎯 Available Commands
====================

SKYNET Commands:
- /skynet:enable      Enable autonomous operation mode
- /skynet:disable     Restore human oversight mode
- /skynet:status      Check current operation mode

Phase 1 (Strategic Planning):
- /phase1:setup       Initialize strategic planning template
- /phase1:analyze     Analyze business requirements 
- /phase1:validate    Complete Phase 1 validation

Phase 2 (Sprint Planning):
- /phase2:setup       Initialize sprint planning template
- /phase2:plan        Create detailed implementation plan
- /phase2:validate    Complete Phase 2 validation

Phase 3 (Implementation):
- /phase3:setup       Initialize implementation environment
- /phase3:implement   Execute TDD implementation
- /phase3:validate    Complete Phase 3 validation

Quality & Tools:
- /tools              Run validation tools
- /quality-check      Complete quality gate validation
- /init               Initialize new CE-DPS project

📚 Documentation Locations
===========================
- Project state: docs/ce-dps-state.json
- Phase documents: docs/phases/
- Sprint tracking: docs/sprints/
- Quality reports: docs/quality-reports/

🆘 Troubleshooting
==================
- Use /skynet:status to check current operation mode
- Each phase must complete before the next can begin
- Use /skynet:enable for autonomous operation
- Check CLAUDE.md for project-specific guidance
```

## Notes
- Provide comprehensive but concise help
- Focus on practical command usage
- Include troubleshooting guidance
- Keep output well-formatted and easy to scan