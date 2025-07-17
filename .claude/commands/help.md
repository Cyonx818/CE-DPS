# <context>CE-DPS Help</context>

<meta>
  <title>CE-DPS Help System</title>
  <type>slash-command</type>
  <audience>ai_assistant</audience>
  <complexity>basic</complexity>
  <updated>2025-07-17</updated>
  <scope>comprehensive-help</scope>
</meta>

## <summary priority="critical">TL;DR</summary>
- **Purpose**: Comprehensive help system for CE-DPS methodology and command reference
- **Core Benefits**: Complete command documentation, workflow guidance, troubleshooting support
- **Output**: Formatted help display with methodology overview, commands, and next steps
- **Usage**: Execute series of echo commands for comprehensive help information

## <instructions priority="high">Implementation Steps</instructions>

### <step-1>Display Comprehensive Help System</step-1>
- Execute echo commands for methodology overview
- Display CE-DPS core principles and AI-as-implementer philosophy  
- Show three-phase development process with durations

### <step-2>Show Complete Command Reference</step-2>
**Command Categories**:
- **Project Management**: /init, /project-status, /quality-check, /tools, /help
- **Phase 1**: /phase1:setup, /phase1:analyze, /phase1:validate  
- **Phase 2**: /phase2:setup, /phase2:plan, /phase2:validate
- **Phase 3**: /phase3:setup, /phase3:implement, /phase3:validate
- **SKYNET**: /skynet:enable, /skynet:disable, /skynet:status

### <step-3>Provide Detailed Usage Patterns</step-3>
- Show typical workflow with numbered steps
- Include human action points between commands
- Display command usage patterns and dependencies

### <step-4>Document Human Actions Required</step-4>
**By Phase**:
- **Phase 1**: Define business problem, review architecture, approve roadmap
- **Phase 2**: Select features, review implementation plans, approve sprint scope  
- **Phase 3**: Validate features, test user experience, approve production

### <step-5>Show Quality Standards Framework</step-5>
**Requirements Matrix**:
- **Security**: authentication, authorization, input validation, SQL injection prevention
- **Testing**: >95% coverage, unit/integration/security/performance tests
- **Performance**: response times, database optimization, scalability

### <step-6>Display Project File Structure</step-6>
- Show docs/ directory: ce-dps-state.json, phases/, sprints/, quality-reports/
- Show tools/ directory: quality-gates/, phase-validator.py, fortitude-integration/

### <step-7>Provide Environment Variables and Troubleshooting</step-7>
- List CE_DPS environment variables and their purposes
- Show common issues and solutions
- Provide getting started workflow for new and existing projects

## <expected-output priority="medium">Command Results</expected-output>

**Output Content**:
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

## <parameters priority="low">Command Parameters</parameters>
**Configuration**:
- No parameters required
- Uses echo commands to display formatted help information
- Provides comprehensive methodology and command reference

## <implementation-notes priority="low">Technical Details</implementation-notes>
**Technical Implementation**:
- Uses bash echo commands for comprehensive help display
- Organizes information with clear headers and bullet points
- Includes all specific command names and usage patterns
- Provides practical troubleshooting and getting started guidance