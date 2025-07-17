# CE-DPS Help

Show comprehensive help for CE-DPS methodology and available commands.

## Instructions

1. **Display Overview**
   - Explain CE-DPS methodology purpose and phases
   - Show the AI-as-implementer philosophy
   - Explain human-AI collaboration model

2. **List Available Commands**
   - **Project Status**: `/cedps-status` - Show current phase and progress
   - **Phase 1 Commands**:
     - `/project:phase1:setup` - Initialize strategic planning
     - `/project:phase1:analyze` - Analyze business requirements  
     - `/project:phase1:validate` - Validate phase completion
   - **Phase 2 Commands**:
     - `/project:phase2:setup` - Initialize sprint planning
     - `/project:phase2:plan` - Create implementation plan
     - `/project:phase2:validate` - Validate phase completion
   - **Phase 3 Commands**:
     - `/project:phase3:setup` - Initialize implementation
     - `/project:phase3:implement` - Execute TDD implementation
     - `/project:phase3:validate` - Validate phase completion
   - **SKYNET Commands**:
     - `/project:skynet:enable` - Enable autonomous operation
     - `/project:skynet:disable` - Restore human oversight
     - `/project:skynet:status` - Check current mode
   - **Utility Commands**:
     - `/cedps-tools` - Run quality gates and validation
     - `/cedps-quality-check` - Complete CI/CD test suite

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

Status & Control:
- /cedps-status          Show current project status and next steps
- /project:skynet:enable  Enable autonomous operation mode
- /project:skynet:disable Restore human oversight mode

Phase 1 (Strategic Planning):
- /project:phase1:setup     Initialize strategic planning template
- /project:phase1:analyze   Analyze business requirements 
- /project:phase1:validate  Complete Phase 1 validation

Phase 2 (Sprint Planning):
- /project:phase2:setup     Initialize sprint planning template
- /project:phase2:plan      Create detailed implementation plan
- /project:phase2:validate  Complete Phase 2 validation

Phase 3 (Implementation):
- /project:phase3:setup     Initialize implementation environment
- /project:phase3:implement Execute TDD implementation
- /project:phase3:validate  Complete Phase 3 validation

Quality & Tools:
- /cedps-tools              Run validation tools
- /cedps-quality-check      Complete quality gate validation

📚 Documentation Locations
===========================
- Project state: docs/ce-dps-state.json
- Phase documents: docs/phases/
- Sprint tracking: docs/sprints/
- Quality reports: docs/quality-reports/

🆘 Troubleshooting
==================
- Start with /cedps-status to understand current state
- Each phase must complete before the next can begin
- Use SKYNET mode for autonomous operation
- Check CLAUDE.md for project-specific guidance
```

## Notes
- Provide comprehensive but concise help
- Focus on practical command usage
- Include troubleshooting guidance
- Keep output well-formatted and easy to scan