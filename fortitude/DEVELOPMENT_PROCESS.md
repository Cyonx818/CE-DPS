# Development Process

<meta>
  <title>Development Process</title>
  <type>methodology</type>
  <audience>ai_assistant</audience>
  <complexity>intermediate</complexity>
  <updated>2025-07-09</updated>
</meta>

## <summary priority="high">Three-Phase Development Methodology</summary>

**Purpose**: Systematic approach for AI-assisted development that ensures quality, maintainability, and progress tracking.

| Phase | Purpose | Output | Time |
|-------|---------|--------|------|
| **Plan** | Define what to build | Architecture + Roadmap | 30-60 min |
| **Sprint** | Plan implementation | Sprint plan + research | 15-30 min |
| **Execute** | Build the solution | Working code + tests | 60-180 min |

## <coordination>AI Assistant Roles by Phase</coordination>

**Phase 1-2**: Direct task execution + parallel research subagents for knowledge gaps
**Phase 3**: Orchestrator role ONLY - delegate ALL implementation tasks and tests to sequential subagents

| Phase | AI Role | Subagent Usage | Rationale |
|-------|---------|----------------|-----------|
| **Plan/Sprint** | Direct execution | Parallel research | Context window + domain expertise |
| **Execute** | Orchestrator only | Sequential implementation | Context management + quality oversight |

## <phase>Phase 1: Project Planning</phase>

### <planning-stage>Architecture Planning</planning-stage>
**Objective**: Design system foundation that enables immediate development

<subagent-coordination>
**Research Subagents**: Always use parallel subagents for ALL research tasks

**When to Use**: ALL research tasks (even single research lookups)
**Implementation**: See [Parallel Subtask Rules](methodology/parallel_subtask_rules.md#phase-phase-1-project-planning) for:
- Research analysis subagent specifications
- Architecture planning subagent coordination
- Context inheritance requirements for planning phase
</subagent-coordination>

<process>
**Process**:
1. **Analyze Requirements**: 
   - Read vision document for feature scope
   - **Check reference library for existing architecture patterns** (search: architecture, system design, [domain keywords])
   - **Check reference library for domain-specific principles** (search: [project domain], constraints, principles)
   - **Create or validate domain-principles.md** (see [Domain Principles Guide](fortitude/docs/reference_library/quick-reference/domain-principles-guide.md))
   - Review design principles for domain constraints
   - Identify core user workflows
2. **Design Architecture**: Define components, plan modular structure, design extensibility
3. **Document Decisions**: Create `fortitude/docs/architecture/system-design.md` with rationale
</process>

<checklist>
**Success Criteria**:
- [ ] Core components identified and relationships mapped
- [ ] Build order planned to deliver early value
- [ ] Architecture document created with diagrams
- [ ] Extensibility strategy defined
</checklist>

### <planning-stage>Roadmap Creation</planning-stage>
**Objective**: Prioritize features for maximum user value progression

<process>
**Process**:
1. **Define MVP**: Identify minimum functional experience
2. **Feature Inventory**: List all desired features from vision
3. **Value Prioritization**: Rank by user impact vs. development effort
4. **Document Roadmap**: Create `fortitude/docs/planning/master-roadmap.md`
</process>

<checklist>
**Success Criteria**:
- [ ] MVP clearly defined with specific boundaries
- [ ] Features prioritized by user value impact
- [ ] Development phases planned with dependencies
- [ ] Master roadmap document created
</checklist>

## <phase>Phase 2: Sprint Development</phase>

### <sprint-stage>Sprint Planning</sprint-stage>
**Objective**: Create detailed, actionable implementation plan

<checklist>
**Pre-Execution Checklist**:
- [ ] Create branch: `git checkout -b fortitude-sprint-[XXX]_[description]`
- [ ] Review architecture document
- [ ] Check current codebase state
</checklist>

<process>
**Process**:
1. **Scope Definition**: Select task from roadmap, define deliverables, estimate complexity
2. **Implementation Planning**: List files to create/modify, define integration points, plan testing
3. **API Compatibility Assessment**: Review planned changes for API compatibility requirements
4. **Task Breakdown**: Break into sequential tasks with clear completion criteria
5. **Document Plan**: Create `fortitude/docs/planning/sprint-[XXX]-plan.md`
</process>

<checklist>
**Success Criteria**:
- [ ] Sprint scope clearly defined and bounded
- [ ] Implementation plan documented with file lists
- [ ] **API compatibility requirements identified** (see [API Compatibility Testing](docs/reference_library/patterns/api-compatibility-testing.md))
- [ ] Tasks broken into AI-manageable chunks
- [ ] Testing strategy defined
</checklist>

### <sprint-stage>Complexity Assessment & Research</sprint-stage>
**Objective**: Identify knowledge gaps and gather needed references

<parallel-research>
**Research Subagents**: Always use parallel subagents for ALL research tasks

**When to Use**: ALL knowledge gap research (even single research lookups)
**Implementation**: See [Parallel Subtask Rules](methodology/parallel_subtask_rules.md#phase-phase-2-sprint-development) for:
- Research coordination patterns
- Context inheritance for research subagents
- Research result documentation
</parallel-research>

<process>
**Process**:
1. **Complexity Analysis**: Rate tasks (Low/Medium/High/Critical), identify challenges
2. **Knowledge Gap Identification**: 
   - **Search reference library first**: Use keywords from implementation plan to find existing knowledge
   - **Pattern search**: Look for similar implementation patterns already documented
   - **Dependency search**: Check if technologies/frameworks in plan already have reference docs
   - Compare requirements against findings to identify true gaps
   - **Reference library consultation requirements**:
     - **Recommended**: Medium complexity tasks or unrated tasks
     - **Mandatory**: High/Critical complexity tasks
   - Prioritize gaps by implementation impact
   - **Only conduct new research for confirmed gaps not covered by existing knowledge**
If research is necessary:
3. **Research Execution**: Deliver research prompts for critical gaps to user using template below. Complete template with relevant query information, written using guidelines on prompt engineering in `fortitude/docs/reference_library/quick-reference/prompt-engineering-quick-ref.md`)
4. **Update Library**: Tell user you are waiting on them to supply research. Integrate findings into reference library once user has supplied them.
</process>

<template>
**Research Request Template**:
```
RESEARCH REQUEST: [Topic Area] 
Project Context: [Brief description] 
Implementation Need: [Specific challenge] 
Required Format: Working Rust code examples with error handling 
Target: AI coding assistant implementing technical development plans 
```
</template>

<checklist>
**Success Criteria**:
- [ ] Complexity assessment completed for all major tasks
- [ ] Knowledge gaps identified and prioritized
- [ ] Research requested for critical gaps
- [ ] Reference library updated with new materials
</checklist>

## <phase>Phase 3: Implementation</phase>

### <implementation-stage>Execution</implementation-stage>
**Objective**: Build solution following plan with quality standards

<sequential-coordination>
**Sequential Subagents**: Use for ALL implementation tasks and tests to manage context window

**When to Use**: ALL Phase 3 implementation tasks (see sprint-004-plan.md for multi-task sprint example)
**Implementation**: See [Parallel Subtask Rules](methodology/parallel_subtask_rules.md#phase-phase-3-implementation) for:
- Sequential task delegation patterns
- Context inheritance for implementation subagents
- Quality assurance through independent verification
</sequential-coordination>

<checklist>
**Pre-Execution Verification**:
- [ ] On correct branch (not main/master)
- [ ] Implementation plan reviewed
- [ ] **Cross-crate compilation validated** with `cargo check --all-targets --all-features`
- [ ] **Test targets compilation verified** with `cargo test --no-run --all`
- [ ] **Reference library consulted for implementation patterns** relevant to current sprint
- [ ] **Existing code examples and best practices** identified and ready to apply
- [ ] Reference materials available in `docs/reference_library`
- [ ] Test strategy understood
</checklist>

<process>
**Process using sequential-coordination**:
1. **Delegate Tasks**: Assign implementation plan tasks and tests to sequential subagents
2. **Monitor Quality**: Verify TDD approach, pattern compliance, error handling in subagent outputs
3. **Coordinate Integration**: Oversee compatibility verification across system boundaries
4. **Orchestrate Updates**: Guide architecture documentation updates through specialized subagents
</process>

<checklist>
**Success Criteria**:
- [ ] All planned features implemented
- [ ] Unit tests written and passing
- [ ] Integration tests written and passing
- [ ] **Anchor tests created** (use decision matrix in `fortitude/tests/README.md`)
- [ ] **Anchor tests documented** with `ANCHOR:` docstring comments
- [ ] Code follows established patterns
- [ ] Documentation updated
- [ ] No regressions introduced
</checklist>

### <implementation-stage>Completion</implementation-stage>

<checklist>
**Final Verification**:
- [ ] All tests pass (`cargo test`)
- [ ] No linter warnings (`cargo clippy`)
- [ ] **Cross-component compatibility verified** with `cargo check --all-targets --all-features`
- [ ] **API compatibility anchor tests created** for interface changes (see [API Compatibility Testing](docs/reference_library/patterns/api-compatibility-testing.md))
- [ ] **Anchor tests created for critical functionality** (see decision matrix in `fortitude/tests/README.md`)
- [ ] **All anchor tests documented** with `ANCHOR:` docstring comments
- [ ] Architecture docs current
- [ ] Master roadmap updated with sprint completion
- [ ] Branch ready for review
</checklist>

<process>
**Commit Process**:
```bash
git add .
git commit -m "feat: [sprint-description] - [key deliverables]"
```
</process>

## <templates>Process Templates</templates>

### <template>Architecture Planning</template>
```markdown
# Fortitude System Architecture

## Core Components
- Component A: Purpose, location, dependencies
- Component B: Purpose, location, dependencies

## Key Design Decisions
1. Decision: Rationale
2. Decision: Rationale

## Build Order
Phase 1: [Components] - delivers [user value]
Phase 2: [Components] - delivers [user value]
```

### <template>Sprint Planning</template>
```markdown
# Sprint [XXX]: [Feature Name]

## Objectives
- Primary: [Main deliverable]
- Success: [How we know it's done]

## Implementation Plan
**Files to Create**: [List]
**Files to Modify**: [List]
**Integration Points**: [Existing systems affected]

## Task Sequence
1. [Task] - [Completion criteria]
2. [Task] - [Completion criteria]

## Testing Strategy
- Unit tests for: [Components]
- Integration tests for: [Workflows]
```

---

**Benefits**: 2-4 hours (vs. 6-8 hours), ~150 lines (vs. 300 lines), checklist-driven reliability 