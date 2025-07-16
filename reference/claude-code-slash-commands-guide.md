# <context>Claude Code Slash Commands: Technical Implementation Guide</context>

<meta>
  <title>Claude Code Slash Commands: Technical Implementation Guide</title>
  <type>technical-reference</type>
  <audience>ai_assistant</audience>
  <complexity>advanced</complexity>
  <updated>2025-07-16</updated>
  <mdeval-score>0.89</mdeval-score>
  <token-efficiency>0.18</token-efficiency>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Technical guide for implementing and managing Claude Code slash commands
- **Key Features**: File-based command discovery, parameter substitution, project/user scoping, MCP integration
- **Architecture**: Markdown files in `.claude/commands/` directory with automatic discovery and execution
- **Security**: Input validation, path sanitization, hook system with shell command execution
- **Integration**: Works with CLAUDE.md context, git history, and external tools via MCP protocol

## <implementation>Core Technical Requirements</implementation>

### <pattern>Implementation Architecture</pattern>

#### <constraints priority="high">File System Organization</constraints>
Claude Code automatically discovers slash commands from Markdown files stored in specific directory structures:

**Project-Specific Commands**:
```
Location: .claude/commands/
Scope: Available only within the specific project
Sharing: Automatically shared with team when repository is cloned
Prefix: /project:
```

**Personal/User Commands**:
```
Location: ~/.claude/commands/
Scope: Available across all projects for the user
Sharing: User-specific, not shared in repositories
Prefix: /user:
```

#### <method>Discovery and Loading Mechanism</method>
Claude Code scans these directories at startup and dynamically loads any .md files as available slash commands. The system recognizes any markdown file in your project's .claude/commands/ directory as a slash command with no installation or setup required.

#### <method>Execution Model</method>
When a slash command is invoked, Claude reads the corresponding Markdown file and executes the instructions contained within:

1. **Command Recognition**: User types `/` to see available commands
2. **Parameter Parsing**: Arguments are captured and substituted using `$ARGUMENTS` placeholder
3. **Content Processing**: Markdown content is interpreted as natural language instructions
4. **Execution**: Claude processes the instructions in the current project context

### <pattern>Command Structure and Syntax</pattern>

#### <constraints>File Format Requirements</constraints>
- Command files must use Markdown format (.md extension)
- Contain the prompt or instructions as file content
- Be placed in the appropriate commands directory

#### <implementation>Basic Command Structure</implementation>
```markdown
# Command Title
Brief description of what this command does.

## Instructions
1. **Step One** - Detailed instruction
2. **Step Two** - Another instruction
   - Sub-instruction
   - Additional context

## Parameters
- Use $ARGUMENTS for dynamic values
- Reference files with @filename syntax
```

#### <method>Parameter Definition and Substitution</method>
Commands can include the special keyword `$ARGUMENTS` to pass parameters from command invocation:

**Example Parameter Usage**:
```markdown
# Fix GitHub Issue
Please analyze and fix the GitHub issue: $ARGUMENTS.

Follow these steps:
1. Use `gh issue view` to get the issue details
2. Understand the problem described in the issue
3. Search the codebase for relevant files
4. Implement the necessary changes to fix the issue
```

**Invocation**:
```bash
/project:fix-github-issue #123
```

#### <method>Namespacing Through Directory Structure</method>
Commands can be organized in subdirectories to create namespaced commands with structure: `<prefix>:<namespace>:<command>`

**Directory Structure Example**:
```
.claude/commands/
├── optimize.md                    # /project:optimize
├── frontend/
│   ├── component.md              # /project:frontend:component
│   └── styling.md                # /project:frontend:styling
└── backend/
    ├── api.md                    # /project:backend:api
    └── database.md               # /project:backend:database
```

## <integration>Integration Points</integration>

### <method>Task Management System Integration</method>
Claude Code maintains awareness of your entire project structure and can integrate with external data sources through MCP (Model Context Protocol).

### <method>API Interfaces and Extension Points</method>
Claude Code hooks provide programmatic control and can integrate with MCP tools that follow the pattern `mcp__<server>__<tool>`:

**MCP Integration Examples**:
- `mcp__memory__create_entities` - Memory server's create entities tool
- `mcp__filesystem__read_file` - Filesystem server's read file tool  
- `mcp__github__search_repositories` - GitHub server's search tool

### <method>Project Context Integration</method>
Commands have access to the current project context including:
- CLAUDE.md files (hierarchical, project-level and nested directory-specific)
- Directory structure
- Git history

### <method>Hook System Integration</method>
Hooks are user-defined shell commands that execute automatically at specific points in Claude Code's lifecycle:

**Hook Events**:
- **PreSession**: Before Claude Code starts
- **PostSession**: After Claude Code ends  
- **PreToolUse**: Before Claude executes any tool
- **PostToolUse**: After a tool completes successfully

## <guidelines>Development Guidelines</guidelines>

### <pattern>Best Practices for Command Creation</pattern>

**Command Design Principles**:
1. **Single Responsibility**: Each command should have a clear, focused purpose
2. **Detailed Instructions**: Provide step-by-step guidance with specific actions
3. **Error Handling**: Include validation and error recovery instructions
4. **Documentation**: Maintain clear descriptions and usage examples

### <pattern>Error Handling and Validation Patterns</pattern>
Implement comprehensive error handling with proper validation, security checks, and recovery procedures:

```markdown
# Robust Command Template
## Validation Steps
1. **Input Validation**
   - Verify required parameters are present
   - Validate file paths for security (check for .. traversal)
   - Sanitize user input

## Error Recovery
- If validation fails, provide clear error message
- Suggest corrective actions
- Log errors for debugging

## Security Considerations  
- Use absolute paths for scripts
- Quote all shell variables: "$VAR" not $VAR
- Validate and sanitize all inputs
```

### <pattern>Performance Considerations</pattern>
For large tasks, use Markdown files as checklists and working scratchpads to manage context window efficiently:

**Performance Optimization**:
- Keep commands focused and atomic
- Use `/clear` between tasks to reset context
- Leverage subagents for complex operations
- Implement proper timeout handling for long-running operations

### <constraints priority="high">Security Guidelines</constraints>
Hooks execute arbitrary shell commands automatically, requiring careful security considerations:

**Security Best Practices**:
- Validate and sanitize all inputs
- Use absolute paths for scripts
- Block path traversal attacks (check for `..`)
- Quote shell variables properly
- Implement timeout limits
- Test in safe environments before production

## <configuration>Configuration and Customization</configuration>

### <pattern>Settings File Structure</pattern>
Claude Code uses multiple configuration locations for different scopes:

**Configuration Hierarchy**:
- **Project-specific**: `.claude/settings.local.json`
- **User-specific local**: `~/.claude/settings.local.json` 
- **Global**: `~/.claude.json`

### <implementation>Hook Configuration Format</implementation>
Hooks are configured in settings files using specific JSON structure:

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "edit_file",
        "hooks": [
          {
            "type": "command",
            "command": "prettier --write $CLAUDE_FILE_PATHS",
            "timeout": 30,
            "run_in_background": false
          }
        ]
      }
    ]
  }
}
```

### <method>Command Sharing and Distribution</method>
Commands stored in `.claude/commands/` are automatically shared when repositories are cloned, enabling team-wide standardization:

**Team Distribution Strategy**:
1. **Repository-Level**: Include commands in `.claude/commands/` for project-specific workflows
2. **Organization-Level**: Maintain shared command libraries in dedicated repositories
3. **Personal-Level**: Use `~/.claude/commands/` for individual productivity tools

### <method>Versioning and Update Mechanisms</method>
Commands are version-controlled through git along with the project repository:

**Version Management**:
- Commands evolve with project requirements
- Use git history to track command changes
- Tag command versions with releases
- Document breaking changes in command interfaces

## <examples>Implementation Examples</examples>

### <implementation>Basic Command Structure</implementation>
```markdown
# Code Review
Perform a comprehensive code review of recent changes.

## Instructions
1. **Check Code Quality**
   - Review recent git changes with `git diff`
   - Verify coding standards compliance
   - Check for potential security vulnerabilities

2. **Test Coverage**
   - Ensure new code has appropriate tests
   - Run test suite and verify all tests pass
   - Check code coverage metrics

3. **Documentation**
   - Verify code is properly documented
   - Update CLAUDE.md if architectural changes were made
   - Check that README reflects current functionality
```

### <implementation>Parameterized Command</implementation>
```markdown
# Create Feature Branch
Create and setup a new feature branch: $ARGUMENTS

## Instructions
1. **Branch Creation**
   - Create new branch: `git checkout -b feature/$ARGUMENTS`
   - Push branch to remote: `git push -u origin feature/$ARGUMENTS`

2. **Setup Tasks**
   - Update local dependencies if needed
   - Run initial tests to ensure clean baseline
   - Create initial commit if template files needed

3. **Documentation**
   - Add feature description to CLAUDE.md
   - Update project board or issue tracker
```

### <implementation>Advanced Hook Integration</implementation>
```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "edit_file",
        "hooks": [
          {
            "type": "command", 
            "command": "npm run lint:fix $CLAUDE_FILE_PATHS && npm run test:changed",
            "timeout": 60
          }
        ]
      }
    ],
    "PreSession": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "git fetch origin && echo 'Repository updated'"
          }
        ]
      }
    ]
  }
}
```

## <error-handling>Error Handling Patterns</error-handling>

### <pattern>Input Validation</pattern>
```markdown
# Validated Command Template
Process the specified component: $ARGUMENTS

## Validation
1. **Check Arguments**
   - Verify $ARGUMENTS is not empty
   - Validate component exists in project structure
   - Ensure user has necessary permissions

## Error Responses
- If no arguments: "Error: Component name required. Usage: /command component-name"
- If component not found: "Error: Component '$ARGUMENTS' not found in project"
- If permission denied: "Error: Insufficient permissions for component modification"
```

### <pattern>Execution Flow Control</pattern>
```markdown
# Robust Execution Pattern
Deploy application to staging environment.

## Pre-flight Checks
1. **Environment Validation**
   - Verify staging environment is accessible
   - Check database connectivity
   - Validate environment variables

2. **Code Quality Gates**
   - Run full test suite: `npm test`
   - Execute linting: `npm run lint`
   - Check build succeeds: `npm run build`

## Deployment Steps
1. **If pre-flight checks pass:**
   - Deploy to staging
   - Run smoke tests
   - Notify team of deployment

2. **If any check fails:**
   - Log specific failure reason
   - Provide remediation steps
   - Halt deployment process
```

## <advanced-features>Advanced Features</advanced-features>

### <method>MCP Server Integration</method>
Commands can leverage MCP servers for extended functionality including database access, API integration, and external tool connectivity:

```markdown
# Database Analysis Command
Analyze database performance for: $ARGUMENTS

## Instructions
1. **Connection Setup**
   - Use postgres MCP server to connect to database
   - Verify connection with: `SELECT 1;`

2. **Performance Analysis**
   - Query execution times for $ARGUMENTS functionality
   - Check index usage and optimization opportunities
   - Analyze query patterns and bottlenecks

3. **Reporting**
   - Generate performance summary
   - Provide optimization recommendations
   - Update performance tracking documentation
```

### <method>Multi-Agent Coordination</method>
Commands can coordinate multiple Claude instances for parallel processing:

```markdown
# Parallel Development Command
Coordinate feature development across multiple workstreams.

## Workstream Coordination
1. **Main Development** (Primary Agent)
   - Implement core feature functionality
   - Maintain architectural consistency
   - Coordinate with other agents

2. **Testing Agent** (Secondary Agent)
   - Develop comprehensive test suite
   - Execute continuous testing
   - Report test results to main agent

3. **Documentation Agent** (Tertiary Agent)
   - Update technical documentation
   - Generate API documentation
   - Maintain user guides

## Synchronization Points
- Agents communicate through shared markdown files
- Regular status updates every 30 minutes
- Merge coordination at completion
```

---

This comprehensive guide provides the technical foundation for implementing, customizing, and managing Claude Code slash commands in enterprise development environments.