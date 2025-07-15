# Parallel Subagent Coordination Rules

<meta>
  <title>Parallel Subagent Coordination Rules</title>
  <type>methodology</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-09</updated>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Enable coordinated parallel subagent execution within Fortitude methodology
- **Key Approach**: Domain separation + context inheritance + file system coordination
- **Core Benefits**: 40-60% sprint execution time reduction, resource conflict prevention, quality assurance
- **When to use**: Research (always parallel), Testing (parallel when no conflicts), Implementation (always sequential)
- **Related docs**: [Claude Code Subagent Guide](../../../reference_library/Claude Code Subagent Guide.md), [DEVELOPMENT_PROCESS](../DEVELOPMENT_PROCESS.md)

## <decision>When to Use Subagents</decision>

### <pattern>Subagent Coordination Decision Matrix</pattern>

<subagent-decision-table>
  <conditions>
    <workflow_type>research | testing | implementation</workflow_type>
    <conflict_detection>true | false</conflict_detection>
    <resource_sharing>true | false</resource_sharing>
    <file_modifications>overlapping | independent</file_modifications>
  </conditions>
  
  <decision_rules>
    <rule workflow="research">
      <pattern>parallel</pattern>
      <rationale>Context window management and research isolation</rationale>
      <applies_to>ALL research tasks (even single lookups)</applies_to>
    </rule>
    
    <rule workflow="testing">
      <if>conflict_detection = false AND resource_sharing = false</if>
      <pattern>parallel</pattern>
      <examples>Unit tests + Integration tests, Security + Performance tests</examples>
    </rule>
    
    <rule workflow="testing">
      <if>conflict_detection = true OR resource_sharing = true OR file_modifications = overlapping</if>
      <pattern>sequential</pattern>
      <examples>Same source files, shared fixtures, execution order dependencies</examples>
    </rule>
    
    <rule workflow="implementation">
      <pattern>sequential</pattern>
      <rationale>Context window management in orchestrator agent</rationale>
      <applies_to>ALL implementation tasks</applies_to>
    </rule>
  </decision_rules>
</subagent-decision-table>

<conflict-detection-criteria>
  <command>ASSESS conflicts using these criteria:</command>
  <parallel_safe>
    <criterion>Different test types (unit vs integration)</criterion>
    <criterion>Different domains (security vs performance)</criterion>
    <criterion>Independent source files</criterion>
    <criterion>No shared test data or fixtures</criterion>
  </parallel_safe>
  
  <sequential_required>
    <criterion>Tests modifying same source files</criterion>
    <criterion>Tests sharing test data or fixtures</criterion>
    <criterion>Tests requiring specific execution order</criterion>
    <criterion>Resource contention (database, network, file system)</criterion>
  </sequential_required>
</conflict-detection-criteria>

**Fortitude-Specific Guidelines:**
- **Research Workflows**: Always use parallel subagents (even single research lookups)
- **Testing Workflows**: Use parallel subagents when no conflicts exist
- **Implementation Workflows**: ALL implementation tasks use sequential subagents (see sprint-004-plan.md example)
- **Purpose**: Context window management and research isolation
- **Pattern**: One subagent per research domain/topic
- **Benefit**: Orchestrator maintains sprint overview without implementation details

### <example>Decision Examples</example>

**Research Workflows (Always Parallel):**
- Research agent: Authentication patterns (single lookup)
- Research agent: API design patterns (single lookup)
- Research agent: Rust performance patterns (single lookup)
- Research agent: Testing framework options (single lookup)

**Testing Workflows (Parallel When Possible):**
- **Parallel**: Unit tests agent + Integration tests agent (different test types)
- **Parallel**: Security tests agent + Performance tests agent (different domains)
- **Sequential**: Tests that modify same files or shared test data

**Implementation Workflows (Always Sequential):**
- Implementation agent: Task 1 (Enhanced Classification Types)
- Implementation agent: Task 2 (Context Detection Implementation)
- Implementation agent: Task 3 (Advanced Classification Engine)
- Implementation agent: Task 4 (Pipeline Integration)

## <implementation>Context Inheritance Framework</implementation>

### <pattern>Universal Requirements (All Subagents)</pattern>

**Mandatory Context Reading** (every subagent must read):
```markdown
Required context reading:
- Read `fortitude/AI_RULES.md` for core collaboration guidelines and quality standards
- Read `fortitude/DEVELOPMENT_PROCESS.md` for systematic methodology framework
- Read `fortitude/methodology/parallel_subtask_rules.md` for subagent coordination rules
```

### <pattern>Role-Specific Documentation Matrix</pattern>

<documentation-unit>
  <summary priority="high">
    Context inheritance mapping for specialized subagent roles
  </summary>
  
  <evidence priority="medium">
    <validation>Based on existing Fortitude methodology context patterns</validation>
    <constraints>All subagents inherit universal requirements plus role-specific docs</constraints>
    <alternatives>Single-agent sequential execution (less efficient)</alternatives>
  </evidence>
  
  <implementation priority="low">
    <role-mapping>
      <role name="research-analysis">
        <purpose>Knowledge gap analysis, reference library research, documentation research</purpose>
        <required-context>
          <universal>AI_RULES.md, DEVELOPMENT_PROCESS.md, methodology/parallel_subtask_rules.md</universal>
          <specific>
            - docs/reference_library/README.md (knowledge management patterns)
            - docs/reference_library/research/llm-optimized-documentation.md (writing standards)
            - docs/reference_library/INDEX.md (navigation and search patterns)
          </specific>
        </required-context>
      </role>
      <role name="implementation-coding">
        <purpose>Code implementation, feature development, integration work</purpose>
        <required-context>
          <universal>AI_RULES.md, DEVELOPMENT_PROCESS.md, methodology/parallel_subtask_rules.md</universal>
          <specific>
            - tests/README.md (testing methodology and anchor test requirements)
            - docs/reference_library/patterns/INDEX.md (implementation patterns)
            - docs/planning/sprint-[XXX]-plan.md (current sprint context)
            - docs/architecture/system-design.md (architectural constraints)
          </specific>
        </required-context>
      </role>
      <role name="documentation-writing">
        <purpose>Documentation creation, style guide compliance, content optimization</purpose>
        <required-context>
          <universal>AI_RULES.md, DEVELOPMENT_PROCESS.md, methodology/parallel_subtask_rules.md</universal>
          <specific>
            - docs/reference_library/research/llm-optimized-documentation.md (style guide and patterns)
            - docs/reference_library/README.md (organization and structure patterns)
            - docs/reference_library/quick-reference/INDEX.md (reference documentation structure)
          </specific>
        </required-context>
      </role>
      <role name="quality-assurance-testing">
        <purpose>Test creation, quality verification, anchor test development</purpose>
        <required-context>
          <universal>AI_RULES.md, DEVELOPMENT_PROCESS.md, methodology/parallel_subtask_rules.md</universal>
          <specific>
            - tests/README.md (comprehensive testing strategy and anchor test requirements)
            - docs/reference_library/patterns/INDEX.md (quality and testing patterns)
            - docs/planning/sprint-[XXX]-plan.md (implementation context for test development)
          </specific>
        </required-context>
      </role>
      <role name="api-compatibility-validation">
        <purpose>API compatibility verification, cross-component testing, interface regression protection</purpose>
        <required-context>
          <universal>AI_RULES.md, DEVELOPMENT_PROCESS.md, methodology/parallel_subtask_rules.md</universal>
          <specific>
            - docs/reference_library/patterns/api-compatibility-testing.md (API compatibility patterns)
            - tests/README.md (anchor test decision matrix with API compatibility criteria)
            - docs/reference_library/quick-reference/development-validation-commands.md (validation commands)
            - docs/planning/sprint-[XXX]-plan.md (API changes in current sprint)
          </specific>
        </required-context>
      </role>
      <role name="architecture-planning">
        <purpose>System design, architectural decisions, strategic planning</purpose>
        <required-context>
          <universal>AI_RULES.md, DEVELOPMENT_PROCESS.md, methodology/parallel_subtask_rules.md</universal>
          <specific>
            - docs/reference_library/domain-principles.md (domain constraints and principles)
            - docs/architecture/system-design.md (current system architecture)
            - docs/planning/master-roadmap.md (strategic direction and priorities)
            - docs/reference_library/README.md (knowledge organization patterns)
          </specific>
        </required-context>
      </role>
    </role-mapping>
  </implementation>
</documentation-unit>

## <coordination>File System Coordination Patterns</coordination>

### <pattern>Domain Separation Strategy</pattern>

**Objective**: Prevent resource conflicts through clear domain boundaries

<process>
**Directory-Based Separation**:
- `/src/config/` - Configuration subagent
- `/src/api/` - API implementation subagent  
- `/src/cli/` - CLI interface subagent
- `/tests/` - Testing and QA subagent
- `/docs/` - Documentation subagent
</process>

**Anti-Pattern**: Multiple subagents working on same files simultaneously

### <pattern>State Coordination Mechanisms</pattern>

```rust
// File system coordination pattern
// Subagents coordinate through structured file creation

// Agent A creates coordination file
// coordination/agent-a-status.json
{
  "agent_id": "config-implementation",
  "status": "in_progress", 
  "files_claimed": ["src/config.rs", "src/config/mod.rs"],
  "dependencies": ["api-implementation"],
  "completion_signal": "coordination/config-complete.flag"
}

// Agent B checks coordination before starting
// coordination/agent-b-status.json  
{
  "agent_id": "api-implementation",
  "status": "waiting",
  "blocked_by": [],
  "ready_to_start": true
}
```

### <pattern>Resource Conflict Recovery</pattern>

**Common Conflicts**:
- Multiple agents modifying same file
- Dependency chain deadlocks
- Context overflow from coordination overhead

**Recovery Strategies**:
1. **Retry with refined scope**: Narrow agent domain boundaries
2. **Redistribute work**: Reassign conflicting work to single agent
3. **Sequential fallback**: Revert to single-agent execution for complex areas
4. **Validate results**: Cross-check agent outputs for consistency

## <examples>Subagent Task Specification Templates</examples>

### <template>Research Analysis Subagent</template>

```markdown
**Subagent Task Specification**

**Objective**: Analyze authentication patterns in existing codebase and reference library
**Output Format**: Structured markdown report with code references and implementation recommendations
**Tool Guidance**: Use Grep, Read, and reference library search patterns from docs/reference_library/README.md
**Scope**: /src/auth directory and authentication-related reference docs only
**Success Criteria**: 
- Identifies all authentication flows with file:line references
- Documents security patterns and potential improvements  
- Provides actionable recommendations with implementation examples

**Required Context Reading**:
- Read `fortitude/AI_RULES.md` for core collaboration guidelines
- Read `fortitude/DEVELOPMENT_PROCESS.md` for methodology framework  
- Read `fortitude/methodology/parallel_subtask_rules.md` for coordination rules
- Read `docs/reference_library/README.md` for knowledge management patterns
- Read `docs/reference_library/research/llm-optimized-documentation.md` for writing standards
- Read `docs/reference_library/INDEX.md` for navigation patterns

**Coordination**:
- Create status file: coordination/auth-analysis-status.json
- Signal completion: coordination/auth-analysis-complete.flag
- Dependencies: None (can start immediately)
```

### <template>Implementation Coding Subagent</template>

```markdown
**Subagent Task Specification**

**Objective**: Implement configuration management system with validation and error handling
**Output Format**: Working Rust code with comprehensive tests and documentation
**Tool Guidance**: Follow TDD approach, create anchor tests per tests/README.md decision matrix
**Scope**: /src/config/ directory and related test files only  
**Success Criteria**:
- Configuration struct with serde derive macros
- Environment variable loading with validation
- Comprehensive error handling with thiserror
- Unit tests and integration tests passing
- Anchor tests created and documented per testing guidelines

**Required Context Reading**:
- Read `fortitude/AI_RULES.md` for code standards and quality requirements
- Read `fortitude/DEVELOPMENT_PROCESS.md` for TDD methodology  
- Read `fortitude/methodology/parallel_subtask_rules.md` for coordination rules
- Read `tests/README.md` for testing strategy and anchor test requirements
- Read `docs/reference_library/patterns/INDEX.md` for implementation patterns
- Read `docs/planning/sprint-[XXX]-plan.md` for current sprint context

**Coordination**:
- Create status file: coordination/config-impl-status.json
- Claim files: ["src/config.rs", "src/config/mod.rs", "tests/config_tests.rs"]
- Dependencies: ["auth-analysis"] (wait for auth patterns analysis)
- Signal completion: coordination/config-impl-complete.flag
```

### <template>Quality Assurance Testing Subagent</template>

```markdown
**Subagent Task Specification**

**Objective**: Create comprehensive test suite with anchor tests for critical functionality
**Output Format**: Test files with anchor tests properly documented using ANCHOR: docstring comments
**Tool Guidance**: Use testing decision matrix from tests/README.md, focus on external API, data persistence, user input processing
**Scope**: /tests/ directory and test creation only
**Success Criteria**:
- Anchor tests created for all critical functionality per decision matrix
- All anchor tests documented with ANCHOR: docstring comments  
- Unit, integration, and property-based tests implemented
- Test coverage >90% on core business logic
- All tests passing with cargo test

**Required Context Reading**:
- Read `fortitude/AI_RULES.md` for quality standards
- Read `fortitude/DEVELOPMENT_PROCESS.md` for testing integration
- Read `fortitude/methodology/parallel_subtask_rules.md` for coordination rules  
- Read `tests/README.md` for comprehensive testing strategy and anchor test requirements
- Read `docs/reference_library/patterns/INDEX.md` for testing patterns
- Read `docs/planning/sprint-[XXX]-plan.md` for implementation context

**Coordination**:
- Create status file: coordination/qa-testing-status.json
- Dependencies: ["config-impl", "api-impl"] (wait for implementation completion)
- Signal completion: coordination/qa-testing-complete.flag
```

## <workflows>Multi-Stage Coordination Workflows</workflows>

### <pattern>Sequential-Parallel Hybrid</pattern>

**Phase 1: Parallel Research** (can run simultaneously)
- Research Analysis Agent: Authentication patterns
- Research Analysis Agent: API design patterns  
- Architecture Planning Agent: System design analysis

**Phase 2: Parallel Implementation** (after research complete)
- Implementation Agent: Configuration system
- Implementation Agent: API endpoints
- Documentation Agent: Architecture documentation updates

**Phase 3: Parallel Verification** (after implementation complete)
- QA Testing Agent: Anchor test creation
- QA Testing Agent: Integration test development
- Documentation Agent: API documentation

### <pattern>Dependency Chain Management</pattern>

```markdown
# Coordination dependency graph
research-auth → config-impl → config-tests
research-api → api-impl → api-tests  
architecture-analysis → system-design → integration-tests

# Parallel execution groups
Group 1 (no dependencies): [research-auth, research-api, architecture-analysis]
Group 2 (depends on Group 1): [config-impl, api-impl, system-design]  
Group 3 (depends on Group 2): [config-tests, api-tests, integration-tests]
```

## <troubleshooting>Common Issues and Solutions</troubleshooting>

### <issue>Duplicate Work Prevention</issue>
**Problem**: Multiple agents working on overlapping functionality
**Solution**: 
- Clarify domain boundaries in task specification
- Use file claiming in coordination status files
- Cross-reference agent scopes before task assignment

### <issue>Context Inheritance Failures</issue>
**Problem**: Subagent missing critical methodology context
**Solution**:
- Verify all universal requirements are specified
- Add role-specific documentation per matrix above
- Test subagent understanding with sample queries

### <issue>Resource Conflicts</issue>
**Problem**: Agents modifying same files or dependencies
**Solution**:
- Implement file claiming coordination pattern
- Use dependency chain management for sequential requirements
- Fall back to single-agent execution for complex conflicts

### <issue>Inconsistent Output Formats</issue>
**Problem**: Agents producing incompatible results
**Solution**:
- Specify exact output formats in task specifications
- Include working examples in task descriptions
- Use shared templates from reference library

## <integration>Integration with Fortitude Methodology</integration>

### <phase>Phase 1: Project Planning</phase>
**Parallel Subagent Usage**: Use for domain research and architectural analysis (no conflicts)
```markdown
<subagent-coordination>
**Objective**: Accelerate architecture planning through parallel domain research

**When to Use**: Multiple independent research domains (architecture patterns, domain principles, technology choices)
**Implementation**: See methodology/parallel_subtask_rules.md for:
- Research analysis subagent specifications  
- Architecture planning subagent coordination
- Context inheritance requirements for planning phase
</subagent-coordination>
```

### <phase>Phase 2: Sprint Development</phase>  
**Parallel Subagent Usage**: Use for knowledge gap analysis and research (no conflicts)
```markdown
<parallel-research>
**Objective**: Coordinate parallel knowledge gap analysis and research

**When to Use**: Multiple independent knowledge gaps or research domains identified
**Implementation**: See methodology/parallel_subtask_rules.md for:
- Multi-perspective research coordination
- Domain separation for research areas
- File system coordination for research results
</parallel-research>
```

### <phase>Phase 3: Implementation</phase>
**Sequential Subagent Usage**: Use sequential subagents for all implementation tasks (context window management)
```markdown
<sequential-coordination>
**Objective**: Manage context window in orchestrator agent by delegating implementation tasks sequentially

**When to Use**: ALL Phase 3 implementation tasks (see sprint-004-plan.md for example of multi-task sprint)
**Implementation**: See methodology/parallel_subtask_rules.md for:
- Sequential task delegation patterns
- Context inheritance for implementation subagents
- Quality assurance through independent verification
</sequential-coordination>
```

## <references>See Also</references>

- [Claude Code Subagent Guide](../../../reference_library/Claude Code Subagent Guide.md) - Comprehensive subagent best practices
- [DEVELOPMENT_PROCESS](../DEVELOPMENT_PROCESS.md) - Three-phase methodology framework
- [AI_RULES](../AI_RULES.md) - Core collaboration guidelines
- [Testing Strategy](../tests/README.md) - Quality assurance and anchor test requirements  
- [Reference Library Management](../docs/reference_library/README.md) - Knowledge management patterns

---

**Success Metrics**: 40-60% sprint execution time reduction, zero resource conflicts, consistent quality through context inheritance, scalable coordination patterns for complex multi-component sprints.