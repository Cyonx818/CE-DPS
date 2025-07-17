# CE-DPS Help

Show comprehensive help for CE-DPS methodology and available commands.

## Instructions

1. **Display Comprehensive Help System**
   - Execute series of echo commands to show methodology overview
   - Display CE-DPS core principles and AI-as-implementer philosophy
   - Show three-phase development process with durations

2. **Show Complete Command Reference**
   - Project Management commands: /cedps-init, /cedps-status, /cedps-tools, /cedps-help
   - Phase 1 commands: /cedps-phase1-setup, /cedps-phase1-analyze, /cedps-phase1-validate
   - Phase 2 commands: /cedps-phase2-setup, /cedps-phase2-plan, /cedps-phase2-validate
   - Phase 3 commands: /cedps-phase3-setup, /cedps-phase3-implement, /cedps-phase3-validate
   - SKYNET commands: /skynet-enable, /skynet-disable, /skynet-status

3. **Provide Detailed Usage Patterns**
   - Show typical workflow with numbered steps
   - Include human action points between commands
   - Display command usage patterns and dependencies

4. **Document Human Actions Required**
   - Phase 1: Define business problem, review architecture, approve roadmap
   - Phase 2: Select features, review implementation plans, approve sprint scope
   - Phase 3: Validate features, test user experience, approve production

5. **Show Quality Standards Framework**
   - Security requirements: authentication, authorization, input validation, SQL injection prevention
   - Testing requirements: >95% coverage, unit/integration/security/performance tests
   - Performance requirements: response times, database optimization, scalability

6. **Display Project File Structure**
   - Show docs/ directory structure with ce-dps-state.json, phases/, sprints/, quality-reports/
   - Show tools/ directory with quality-gates/, phase-validator.py, fortitude-integration/

7. **Provide Environment Variables and Troubleshooting**
   - List CE_DPS environment variables and their purposes
   - Show common issues and solutions
   - Provide getting started workflow for new and existing projects

## Expected Output

The command will execute a series of echo commands that display:
- CE-DPS methodology overview with core principles
- Three-phase development process with detailed descriptions and durations
- Complete command reference organized by category
- Typical workflow patterns with numbered steps
- Human action requirements for each phase
- Quality standards framework (security, testing, performance)
- Project file structure and organization
- Environment variables configuration
- Comprehensive troubleshooting guide
- Getting started workflows for new and existing projects
- Documentation references and next steps

## Parameters
- No parameters required
- Uses echo commands to display formatted help information
- Provides comprehensive methodology and command reference

## Notes
- Uses actual bash echo commands to display comprehensive help
- Organizes information with clear headers and bullet points
- Follows the detailed structure from the legacy backup version
- Includes all specific command names and usage patterns
- Provides practical troubleshooting and getting started guidance