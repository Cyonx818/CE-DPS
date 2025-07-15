# Getting Started with Fortitude

*A Human Guide to AI-Assisted Development*

## What is Fortitude?

Fortitude is an **automated AI knowledge pipeline** that generates research documentation for AI assistants. Think of it as a smart research assistant that fills in knowledge gaps when your AI coding tools encounter unfamiliar concepts.

**Your Role**: Provide project vision and make key decisions  
**AI's Role**: Handle technical research, implementation, and documentation  
**The Process**: Three structured phases that build from vision to working system

## What You'll Get

✅ **On-Demand Research**: AI generates technical documentation when you need it  
✅ **Smart Knowledge Base**: Reference library that grows with each research request  
✅ **Multiple Interfaces**: CLI commands, API, and Claude Code integration  
✅ **Quality Assurance**: Built-in testing and validation for reliable results

## Before You Start

**Technical Requirements:**
- Rust toolchain installed ([rustup.rs](https://rustup.rs/))
- Git repository set up
- AI coding assistant (Claude Code recommended)

**Knowledge Requirements:**
- Basic familiarity with Rust
- Understanding of your research domain (what you want Fortitude to help with)
- Willingness to iterate and refine based on results

## Quick Setup

### 1. Get the Code
```bash
git clone [repository-url] your-fortitude-project
cd your-fortitude-project
cargo build
cargo test
```

### 2. Define Your Vision
Create `fortitude/docs/ideas/project-brainstorming.md` with:
- What research problems you want to solve
- What domains Fortitude should focus on
- Success criteria for your research system

### 3. Set Your Constraints
Create `fortitude/docs/reference_library/domain-principles.md` with:
- Technical limitations or requirements
- Quality standards for research outputs
- Integration requirements with existing tools

---

## The Three-Phase Development Process

### Overview
| Phase | What You Do | What AI Does | Time Investment |
|-------|-------------|--------------|-----------------|
| **Plan** | Review & approve architecture | Design system & create roadmap | 30-60 minutes |
| **Sprint** | Select features & priorities | Create implementation plan | 15-30 minutes |
| **Execute** | Quality review & feedback | Build, test, document | 60-180 minutes |

### Your Responsibilities Throughout

**Strategic Decisions:**
- Define project vision and priorities
- Approve architectural decisions
- Select features for each development sprint
- Review completed work before integration

**Quality Gates:**
- Review architecture documents for feasibility
- Approve sprint plans before implementation
- Validate completed features meet requirements
- Make go/no-go decisions for releases

**Research Coordination:**
- Provide domain expertise when AI needs context
- Validate research findings and outputs
- Guide AI toward the most valuable research areas

---

## Phase 1: Project Planning

**What Happens:** AI reads your vision and creates a system architecture plus development roadmap.

**Your Preparation:**
1. Ensure your vision document is complete and clear
2. Have domain principles defined
3. Be ready to review and approve architectural decisions

**AI Prompt to Use:**
```
I'm starting Fortitude development using the three-phase methodology.

Required context reading:
- Read `fortitude/AI_RULES.md` for core guidelines and collaboration framework
- Read `fortitude/DEVELOPMENT_PROCESS.md` for the systematic approach we'll follow
- Read `fortitude/docs/reference_library/README.md` for knowledge management patterns

Project context:
- Vision: `fortitude/docs/ideas/project_brainstorming.md`
- Pre-existing guidance: `fortitude/docs/ideas/prototype_lessons_learned.md`
- Domain constraints: `fortitude/docs/reference_library/domain-principles.md`

Execute Phase 1 (Project Planning):
1. Analyze requirements from vision document
2. Design system architecture 
3. Create prioritized feature roadmap

Follow DEVELOPMENT_PROCESS.md Phase 1 exactly. Use todo tracking for all checklist items and success criteria.
```

**What to Expect:**
- System architecture document with component relationships
- Feature roadmap with development phases
- Clear next steps for Sprint planning

**Your Review Focus:**
- Does the architecture solve your research problems?
- Are the prioritized features aligned with your vision?
- Are there any missing critical components?

---

## Phase 2: Sprint Planning

**What Happens:** AI creates a detailed implementation plan for the next feature set.

**Your Preparation:**
1. Review and approve the architecture from Phase 1
2. Select which features to implement first
3. Be ready to clarify any requirements

**AI Prompt to Use:**
```
Start `fortitude/DEVELOPMENT_PROCESS.md` Phase 2 (Sprint Development) for next sprint, using todo tracking for all pre-execution items, process items, and success critera items.

Required context reading:
- Read `fortitude/AI_RULES.md` for development guidelines
- Read `fortitude/DEVELOPMENT_PROCESS.md` Phase 2 for sprint planning approach
- Review completed Phases in `fortitude/docs/planning/master-roadmap.md`

Current status: [Check master-roadmap.md to see current completion status] 
Target: [Next unimplemented sprint item from master-roadmap] 

Execute Phase 2:
- Create detailed sprint plan with implementation steps
- Assess complexity and identify knowledge gaps
- Request any needed research using template in DEVELOPMENT_PROCESS and guidelines in `fortitude/docs/reference_library/quick-reference/prompt-engineering-quick-ref.md`
- Prepare for execution

Follow DEVELOPMENT_PROCESS.md Phase 2 exactly.
```

**What to Expect:**
- Detailed implementation plan with file lists
- Complexity assessment and risk identification
- Research documentation for any knowledge gaps
- Clear development tasks ready for execution

**Your Review Focus:**
- Is the scope appropriate for the time investment?
- Are the implementation steps logical and complete?
- Do you understand what will be built?

---

## Phase 3: Implementation

**What Happens:** AI builds the planned features with comprehensive testing and documentation.

**Your Preparation:**
1. Approve the sprint plan from Phase 2
2. Be available for questions during implementation
3. Prepare to review and test the completed features

**AI Prompt to Use:**
```
Implement `fortitude/DEVELOPMENT_PROCESS.md` Phase 3, using todo tracking for all pre-execution items, process items, success critera items, and final verification items. 

Required context reading:
- Read `fortitude/AI_RULES.md` for code standards and quality requirements
- Read `fortitude/tests/README.md` for testing strategy and requirements
- Review current (newest) sprint plan in: `fortitude/docs/planning` (format: sprint-[XXX]-plan.md)

Current status: Phase 2 Sprint Development complete, reference library up to date
Implementation target: [newest sprint]

Execute Phase 3:
- Implement all planned features using TDD approach
- Create comprehensive tests (unit, integration, performance)
- Update documentation and architecture as needed
- Ensure all quality gates pass

Follow DEVELOPMENT_PROCESS.md Phase 3 exactly.
```

**What to Expect:**
- Working code with comprehensive test coverage
- Updated documentation reflecting any changes
- Clean commit history with clear messages
- Passing quality gates (tests, linting, etc.)

**Your Review Focus:**
- Do the features work as expected?
- Is the code quality acceptable for your standards?
- Are there any integration issues or rough edges?

---

## Tips for Success

### Getting Better Results
- **Be specific in your vision**: Clear requirements lead to better architecture
- **Review thoroughly**: Your approval gates ensure quality
- **Stay engaged**: Regular feedback keeps development on track
- **Iterate**: Use learnings from each phase to improve the next

### Common Pitfalls to Avoid
- **Vague requirements**: AI needs clear direction to make good decisions
- **Skipping reviews**: Quality problems compound if not caught early
- **Scope creep**: Stick to planned features within each sprint
- **Ignoring quality gates**: Technical debt will slow future development

### When Things Go Wrong
- **Architecture doesn't fit**: Go back to Phase 1 with refined requirements
- **Implementation is too complex**: Break into smaller sprints
- **Quality issues**: Review AI_RULES.md and ensure standards are clear
- **Integration problems**: Check that all dependencies are properly documented

---

## Getting Help

**Process Questions**: Review `fortitude/DEVELOPMENT_PROCESS.md` for detailed methodology  
**AI Collaboration Issues**: Check `fortitude/AI_RULES.md` for guidelines and standards  
**Testing Problems**: See `fortitude/tests/README.md` for comprehensive testing strategy  
**Knowledge Management**: Consult `fortitude/docs/reference_library/README.md` for patterns

---

## Ready to Start?

1. ✅ Complete the quick setup above
2. ✅ Create your project vision document  
3. ✅ Define your domain principles
4. ✅ Start Phase 1 with the planning prompt

Your AI assistant will handle the technical complexity while you focus on strategic decisions and quality oversight. The process scales from small features to complex systems, always maintaining quality and documentation standards.

**Remember**: You're not just building Fortitude – you're creating a system that will make all your future AI-assisted development more effective!