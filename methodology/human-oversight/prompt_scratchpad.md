-------------------------------------------------------------------------------

We are doing pre-production work on Fortitude development.

Required context reading:
- Read `fortitude/AI_RULES.md` for core guidelines and collaboration framework
- Read `fortitude/DEVELOPMENT_PROCESS.md` for the systematic approach we'll follow
- Read `fortitude/docs/reference_library/README.md` for knowledge management patterns

Project context:
- Vision: `fortitude/docs/ideas/project_brainstorming.md`
- Pre-existing guidance: `fortitude/docs/ideas/prototype_lessons_learned.md`
- Domain constraints: none yet
- Reference library: `fortitude/docs/reference_library/`

Given the Vision, pre-existing guidance, and information available in the reference library, along with the methodology documentation I've had you read, do we have enough information to generate a complete and robust `docs/reference_library/domain-principles.md` file, for use with Phase 1 of the development process?

-------------------------------------------------------------------------------

We need to ingest information into fortitude's reference library. 

Create and follow a todo list:
- Read and understand general instructions in `fortitude/AI_RULES.md`
- Read and understand project and documentation context in `fortitude/README.md` and `fortitude/DEVELOPMENT_PROCESS.md`
- Find, Read, and understand writing style guidelines, regarding writing for AI consumption, at `fortitude/docs/reference_library/research/llm-optimized-documentation.md`.
- Ingest the requested research documents (both .md and .rs) in `fortitude/docs/research` into fortitude's reference library, using guidelines and instructions in `fortitude/docs/reference_library/README.md`. Ignore the `fortitude/docs/research/research_needed[DO NOT INGEST INTO REFERENCE LIBRARY].md` document
- Move the ingested research documents to `fortitude/docs/history`
- Confirm whether any research requested in `fortitude/docs/research/research_needed[DO NOT INGEST INTO REFERENCE LIBRARY].md` now exists in the reference library. If so, remove the section from the document. Do not bloat the document with information about the request being completed.     
- Commit all changes properly to the branch, including any changes I made behind the scenes 

-------------------------------------------------------------------------------

I want to create a repo template using fortitude as the base.
- create a new branch for this purpose and switch to it
- create a new `repo-template` folder in the concordia folder. do not add it to the concordia workspace. This is a template for new projects, not a new project itself.
- mirror the folder structure of fortitude to repo-template. do not populate the folders.
- copy over the core methodology documents to repo-template. Documents: fortitude/CLAUDE.md , fortitude/AI_RULES.md , fortitude/DEVELOPMENT_PROCESS.md , fortitude/GETTING_STARTED.md , fortitude/README.md , fortitude/tests/README.md , and fortitude/docs/reference_library/README.md
- remove references to fortitude and fortitude-specific information from the documents in repo-template, and "genericize" them.
- `repo-template` is not meant to be an active git repo, but it will be used as a template to create them in the future.

-------------------------------------------------------------------------------

I need you to make a couple edits to `fortitude/GETTING_STARTED.md`. 
- Each one of the phase prompts is expected to run against a clean claude code session, so no knowledge of previous phases is expected, other than what has been written to file. Therefore we need to preamble each prompt with a directive to the LLM to injest the required methodology documents that will be necessary for that phase. Since the documents have links to each other where appropriate, we only need to pre-load the minimum methodology documents necessary, with instruction to the LLM to ingest other methodology documents if necessary for context.
- Unlike other methodology documents, GETTING_STARTED.md is for humans. Format the document to be human-readable and human-friendly. It should be organized to make it easy for humans to find the information the need for their portion of the process. 
- GETTING_STARTED.md does not have to be as information dense as the other methodology documents. Take a little bit of extra time to document the process in an easily understandable (for humans) method. Do not bloat the document with unneeded documentation however.
- Audit the 3 phase prompts to make sure they efficiently do the job they're intended to do. Remove redundancy, there is no need for the prompt to tell the AI to do something that another methodology document (that the LLM should have already ingested) has already told it to do. The human should be able to paste a prompt and hit enter to start off the next phase. Because these prompts are in GETTING_STARTED, and humans will be entering them, they should be the best combination of human-understandable and LLM-context-optimied that we can make them. 

-------------------------------------------------------------------------------

The documents at the bottom are vital to the way that my development methodology works. However, I believe that there are organizational and logical redundancies among them, as well as possible conflicts regarding direction. I want you to examine these documents, determine how they fit into the overall process, and determine where we can remove redundancies and replace with references, as well as generally optimize these documents for AI consumption. 

notes:
- This methodology hinges on Human (me) providing direction, guidance, and oversight, with the LLM (you, or another agentic ai coding assistant) implementing code creation, code changes, test creation and execution, bug fixes, documentation, etc. The LLM-based agentic ai coding assistant will be using these methodology documents to perform its role.
- Determine what each document's focus should be, and make it the source of truth for that focus. some documents will need to specifically be focused on this project. other documents may need to be more "project-agnostic".
- With the exception of "GETTING_STARTED.md", these documents are all designed primarily to be read by, written to, and provide efficient contextual guidance to Agentic AI coding assistants, not humans. 
- We want to optimize the AI-targeted documents for LLM efficiency and context window usage. Use `/home/cyonx/Documents/GitHub/concordia/fortitude/docs/research/LLM-Optimized Documentation System Implementation (combined and optimized).md` for guidelines on doing so.
- No information can be lost during this optimization pass. AIs reading these documents need to be able to find the information and direction necessary for what they're doing.
- The documents were copied from another, more generic, project. clean out references to any other project.

Documents: fortitude/CLAUDE.md , fortitude/AI_RULES.md , fortitude/DEVELOPMENT_PROCESS.md , fortitude/GETTING_STARTED.md , fortitude/README.md , fortitude/tests/README.md , and fortitude/docs/reference_library/README.md 


-------------------------------------------------------------------------------

Implement `fortitude/DEVELOPMENT_PROCESS.md` Phase 3, using todo tracking for all pre-execution items, process items, sucess critera items, and final verification items. 

Required context reading:
- Read `fortitude/AI_RULES.md` for code standards and quality requirements
- Read `fortitude/tests/README.md` for testing strategy and requirements
- Review current sprint plan in: `fortitude/docs/planning/sprint-002-plan.md`

Current status: you have finished tasks 1, 2, 3, and 4 of sprint-002. continue with task 5
Implementation target: sprint-002-plan.md

Execute Phase 3:
- Implement all planned features using TDD approach
- Create comprehensive tests (unit, integration, performance)
- Update documentation and architecture as needed
- Ensure all quality gates pass

Follow DEVELOPMENT_PROCESS.md Phase 3 exactly.

-------------------------------------------------------------------------------

Start `fortitude/DEVELOPMENT_PROCESS.md` Phase 2 (Sprint Development) for next sprint, using todo tracking for all pre-execution items, process items, and success critera items.

Required context reading:
- Read `fortitude/AI_RULES.md` for development guidelines
- Read `fortitude/DEVELOPMENT_PROCESS.md` Phase 2 for sprint planning approach
- Review completed Phases in `fortitude/docs/planning/master-roadmap.md`

Current status: [Check master-roadmap.md to see current completion status] 
Target: [any uncompleted items in "Phase 1: Foundation" from master-roadmap] 

Execute Phase 2:
1. Create detailed sprint plan with implementation steps
2. Assess complexity and identify knowledge gaps
3. Request any needed research using template in DEVELOPMENT_PROCESS and guidelines in `fortitude/docs/reference_library/quick-reference/prompt-engineering-quick-ref.md`
4. Prepare for execution

Follow DEVELOPMENT_PROCESS.md Phase 2 exactly.

-------------------------------------------------------------------------------

Add to your todo list, then execute:
- Find, Read, and understand writing style guidelines, regarding writing for AI consumption, at `fortitude/docs/reference_library/research/llm-optimized-documentation.md`.
- Ingest the requested research documents (both .md and .rs) in `fortitude/docs/research` into fortitude's reference library, using guidelines and instructions in `fortitude/docs/reference_library/README.md`. Ignore the `fortitude/docs/research/research_needed[DO NOT INGEST INTO REFERENCE LIBRARY].md` document
- Move the ingested research documents to `fortitude/docs/history`
- Commit all changes properly to the branch, including any changes I made behind the scenes 

-------------------------------------------------------------------------------

We need to confirm that all tests pass, all warnings are cleaned up, and builds are pristine. spawn sub-tasks with appropriate context (per `concordia/fortitude/methodology/parallel_subtask_rules.md`) to clean up everything identified. create a todo list for that, then add everything you've planned to the todo list.

-------------------------------------------------------------------------------
It's time for fortitude codebase cleanup

Required context reading:
- Read `fortitude/AI_RULES.md` for core guidelines and collaboration framework
- Read `fortitude/DEVELOPMENT_PROCESS.md` for the systematic approach we'll follow
- Read `fortitude/methodology/parallel_subtask_rules.md` for subtask rules
- Read `fortitude/docs/planning/master-roadmap.md` for project context
- Read `fortitude/tests/README.md` for testing strategy and requirements
- Read `fortitude/docs/reference_library/README.md` for knowledge management patterns

Assess all warnings, errors, test failures, etc. Determine if any are expected at this stage in the project. create a todo list for all tasks, then spawn and orchestrate parallel sub-tasks with appropriate context (per `concordia/fortitude/methodology/parallel_subtask_rules.md`), where appropriate, to clean up everything identified.

Additionally, I want a summary of the kinds of errors we're seeing making it past the development, testing, and troubleshooting phase, to still be present at this phase (end of sprint cleanup). given the development process, ai rules, and tests documentation, speculate why these errors persisted to this cleanup phase after the implementation phases.

-------------------------------------------------------------------------------

I am building a development methodology centered around playing to the strengths of LLM coding assistants, while minimizing their downsides.

The methodology intends LLMs to write code and documentation, research and implement solutions, and do testing and debugging. 

The methodology intends humans to act as designer, producer, project manager, and final arbiter of quality. 

I am calling this new type of methodology a "Development Process Solution" (DPS).

Required context reading:
- Read `repo_template/CLAUDE.md` for claude-specific direction
- Read `repo_template/methodology/AI_RULES.md` for core guidelines and collaboration framework
- Read `repo_template/methodology//DEVELOPMENT_PROCESS.md` for the systematic approach we'll follow
- Read `repo_template/methodology//methodology/parallel_subtask_rules.md` for "Claude Code as orchestrator" subtask rules
- Read `repo-template/methodology/GETTING_STARTED.md` for the human's requirements
- Read `repo-template/tests/README.md` for testing strategy and requirements
- Read `repo-template/docs/reference_library/README.md` for knowledge management patterns
- Read `repo-template/docs/research/llm-optimized-documentation.md` for best practices regarding writing documentation for LLM understanding and context window preservation.

I want you to run an optimization assessment on the methodology used in the DPS. 

I want to know if you: 
- See any places where the intention of instructions is unclear, or conflicts with other instructions elsewhere in the methodology documents.
- See any places where we can optimize verbage and/or syntax in the markdown files to better achieve LLM comprehension and reliable execution of those instructions.
- See any places where we can optimize verbage and/or syntax in the markdown files to save context window space.

Notes: 
- All documents referenced in required context reading are considered "methodology document" and are the target of these optimization passes.
- All documents (except GETTING_STARTED.md) are intended to be read by LLMs as their primary audience and target.
- You are acting primarily in Orchestrator mode. You will spawn and orchestrate sub-tasks with appropriate context (per `repo_template/methodology/parallel_subtask_rules.md`) to perform tasks they are suited for.
- **IMPORTANT** no context or instruction content loss is acceptable. No content or context be lost during any suggested optimization, only transformed into a more optimal state.

Assess all assigned tasks, then create a to-do list to prepare for execution once I have approved the plan.

-------------------------------------------------------------------------------

I have delivered the requested information to the `fortitude/docs/research` folder. We need to ingest the research into fortitude's reference library before this phase is complete.

Create and follow a todo list:
- Read and understand writing style guidelines, regarding writing for AI consumption, at `fortitude/docs/reference_library/research/llm-optimized-documentation.md`.
- Ingest the requested research documents in `fortitude/docs/research` into fortitude's reference library, using guidelines and instructions in `fortitude/docs/reference_library/README.md`. Ignore the `fortitude/docs/research/research_needed[DO NOT INGEST INTO REFERENCE LIBRARY].md` document
- You are acting primarily in Orchestrator mode. You will spawn and orchestrate sub-tasks with appropriate context (per `fortitude/methodology/parallel_subtask_rules.md`) to perform tasks they are suited for.
- Move the ingested research documents to `fortitude/docs/history`
- Confirm whether any research requested in `fortitude/docs/research/research_needed[DO NOT INGEST INTO REFERENCE LIBRARY].md` now exists in the reference library. If so, remove the section from the document. Do not bloat the document with information about the request being completed.     
- Commit all changes properly to the branch, including any changes I made behind the scenes 

-------------------------------------------------------------------------------

RESEARCH REQUEST: File System Monitoring with Rust notify Crate

Project Context: Fortitude AI research pipeline implementing Sprint 008 Proactive Research Mode - automated knowledge gap detection system that monitors project files for changes and triggers background research

Implementation Need: Need efficient file system monitoring for automated gap analysis in potentially large Rust projects (1000+files). System must detect file changes (creation, modification, deletion) and trigger gap analysis algorithms, with debouncing to prevent excessive processing and configurable monitoring rules.

Required Format: Working Rust code examples with error handling using notify crate, including:
- Basic file watcher setup with recursive directory monitoring
- Event filtering and debouncing patterns for high-volume file changes
- Integration with tokio async runtime for background processing
- Configuration patterns for monitoring rules and exclusion patterns
- Resource optimization for monitoring large codebases without performance impact

Quality Criteria: Production-ready patterns with comprehensive error handling, performance optimization for 1000+ files, integration examples with tokio background task queues, memory usage optimization, and graceful shutdown patterns

Target: AI coding assistant implementing technical development plans for automated background processing system


-------------------------------------------------------------------------------

Implement `fortitude/DEVELOPMENT_PROCESS.md` Phase 3, using todo tracking for all pre-execution items, process items, success critera items, and final verification items. 

Required context reading:
- Read `fortitude/AI_RULES.md` for code standards and quality requirements
- Read `fortitude/tests/README.md` for testing strategy and requirements
- Review sprint plan 009 in: `fortitude/docs/planning` (format: sprint-[XXX]-plan.md)

Current status: Sprints 001 - 008 completed. Sprint 009 Tasks 1, 2, 3, and 4 completed
Implementation target: remainder of Sprint 009 

Execute Phase 3:
- Use subtasks / subagents wherever allowed
- Implement all planned features using TDD approach
- Create comprehensive tests (unit, integration, performance)
- Update documentation and architecture as needed
- Ensure all quality gates pass

Follow DEVELOPMENT_PROCESS.md Phase 3 exactly.

-------------------------------------------------------------------------------

Fortitude feature development is complete. It's time for fortitude codebase cleanup

Required context reading:
- Read `fortitude/AI_RULES.md` for core guidelines and collaboration framework
- Read `fortitude/DEVELOPMENT_PROCESS.md` for the systematic approach we'll follow
- Read `fortitude/methodology/parallel_subtask_rules.md` for subtask rules
- Read `fortitude/docs/planning/master-roadmap.md` for project context
- Read `fortitude/tests/README.md` for testing strategy and requirements
- Read `fortitude/docs/reference_library/README.md` for knowledge management patterns

- Assess any remaining warnings, errors, test failures, etc. 
- Determine root cause. 
- Fix everything. 
- Create a todo list for all tasks, then spawn and orchestrate parallel sub-tasks with appropriate context (per `concordia/fortitude/methodology/parallel_subtask_rules.md`), where appropriate, to clean up everything identified.

Success criteria: We are not complete until every warrning, error, and failure have been resolved. we need the codebase to be pristine now that feature development is complete.


-------------------------------------------------------------------------------

Fortitude feature development is complete. it's time to polish the codebase to "maintenance and bugfix" level. we want the codebase as pristine as possible.

Required context reading:
- Read `fortitude/AI_RULES.md` for core guidelines and collaboration framework
- Read `fortitude/DEVELOPMENT_PROCESS.md` for the systematic approach we'll follow
- Read `fortitude/methodology/parallel_subtask_rules.md` for subtask rules
- Read `fortitude/docs/planning/master-roadmap.md` for project context
- Read `fortitude/tests/README.md` for testing strategy and requirements
- Read `fortitude/docs/reference_library/README.md` for knowledge management patterns

your task is to sweep the entire codebase, both rust sourcecode and markdown documentation, to:
- remove temporal and development-ccentric notations (such as "sprint 007" or "phase 3"). 
- remove sprint results and implementation results types of documents documents
- remove temporary files that were built to assist development, but are not part of the standard documented set of tests, 
- Remove temporary comments. (keep all relevent code documentation comments!)
- Remove any dead code that isn't clearly targeted at future feature integration points

Create a todo list for all tasks, then spawn and orchestrate parallel sub-tasks with appropriate context (per `concordia/fortitude/methodology/parallel_subtask_rules.md`), where appropriate, to clean up everything identified.

-------------------------------------------------------------------------------

Fortitude feature development is complete.

Required context reading:
- Read `fortitude/AI_RULES.md` for core guidelines and collaboration framework
- Read `fortitude/DEVELOPMENT_PROCESS.md` for the systematic approach we'll follow
- Read `fortitude/methodology/parallel_subtask_rules.md` for subtask rules
- Read `fortitude/docs/planning/master-roadmap.md` for project context
- Read `fortitude/tests/README.md` for testing strategy and requirements
- Read `fortitude/docs/reference_library/README.md` for knowledge management patterns

I have added API keys for Gemini and OpenAI to my .env file at `fortitude/.env`. You can't see it because it's hidden, but they're there. There is no anthropic key at this time, so I commented that portion of the .env out.

I do not want you to test them yet. I want you to tell me what would happen, during a normal fortitude reference request via CLI or MCP, what what fortitude do regarding LLM calls with the .env in the state I have described to you. how would Fortitude process that .env file and then choose how to make a research request. walk me through it step by step, from incoming request until a research response is handed off by fortitude to whatever called the request in.

-------------------------------------------------------------------------------

Fortitude feature development is complete. I now need your assistance running targeted tests on fortitde. First thing I need you to do is pre-load your context window with the below required reading.

Required context reading:
- Read `fortitude/AI_RULES.md` for core guidelines and collaboration framework
- Read `fortitude/methodology/parallel_subtask_rules.md` for subtask guidelines.
- Read `fortitude/docs/architecture/system-design.md` for project context and architecture
- Read `fortitude/docs/reference_library/README.md` for current knowledge management patterns. 

notes:
- Fortitude is itself a knowledge management platform, a superior one to the patterns we've been using, but Fortitude is not production ready yet.
- I have added API keys for Gemini and OpenAI to the .env file at `fortitude/.env`. You can't see it because it's hidden, but they're there. There is no anthropic key at this time, so I commented that portion of the .env out.
- Our goal is to validate which pieces of fortitude work as designed and intended, and which pieces need logistical improvement before fortitude can go live. 
- You are an Orchestrator. In order to use your context window efficiently, you should be using subtasks / subagents for implementation, researching, documentation, and bugfixing. Follow guidelines in the parallel_subtask_rules.

Let me know when you are ready to proceed, and I will give futher instructions.

-------------------------------------------------------------------------------

Fortitude feature development is complete. It's time for fortitude codebase cleanup.
Use a todo list for all tasks and success criteria.

Required context reading:
- Read `fortitude/AI_RULES.md` for core guidelines and collaboration framework
- Read `fortitude/DEVELOPMENT_PROCESS.md` for the systematic approach we'll follow
- Read `fortitude/methodology/parallel_subtask_rules.md` for subtask rules
- Read `fortitude/docs/planning/master-roadmap.md` for project context
- Read `fortitude/tests/README.md` for testing strategy and requirements
- Read `fortitude/docs/reference_library/README.md` for knowledge management patterns

- Start your remediation with `fortitude/docs/planning/known_technical_debt.md`
- Assess any remaining warnings, errors, test failures, etc. 
- Determine root cause. 
- Fix everything. We need software architecture to work as intended, not just "pass tests".
- Use a todo list for all tasks.

You are an Orchestrator. You should be using subtasks / subagents any time you are able to do so, in order to use your context window efficiently.

Success criteria: We are not complete until every warning, error, and failure that can be resolved, has been resolved. We need the codebase to be pristine now that feature development is complete. 

-------------------------------------------------------------------------------

I am building a development methodology centered around playing to the strengths of LLM coding assistants, while minimizing their downsides.

The methodology intends LLMs to write code and documentation, research and implement solutions, and do testing and debugging. 

The methodology intends humans to act as designer, producer, project manager, and final arbiter of quality. 

I am calling this new type of methodology a "Development Process Solution" (DPS).

Required context reading:
- Read `repo_template/CLAUDE.md` for LLM-specific direction
- Read `repo_template/methodology/AI_RULES.md` for core guidelines and collaboration framework
- Read `repo_template/methodology//DEVELOPMENT_PROCESS.md` for the systematic approach we'll follow
- Read `repo_template/methodology//methodology/parallel_subtask_rules.md` for "Claude Code as orchestrator" subtask rules
- Read `repo-template/methodology/GETTING_STARTED.md` for the human's requirements
- Read `repo-template/tests/README.md` for testing strategy and requirements
- Read `repo-template/docs/reference_library/README.md` for knowledge management patterns
- Read `repo-template/docs/research/llm-optimized-documentation.md` for best practices regarding writing documentation for LLM understanding and context window preservation.

I want you to compare and contrast my "DPS" methodology against "context engineering" as described and outlined in `repo-template/Context Engineering in AI Coding.md`. I want to know what My methodology does that context engineering doesn't, and what insights from context engineering might be applied to DPS to improve its reliability, performance, and quality. report your findings/opinions in a new document called `repo-template/claude-dps-CE-comparison.md`

-------------------------------------------------------------------------------

I need you to create a new project. 

Background information: 
- The repo-template (aka DPS) and context-engineering-intro (aka CE) repos both represent design methodologies for working with ai agentic coding assistants (aka codeassist) like claude code in a development capacity.
- Fortitude represents a RAG and knowledge management platform designed specifically to be LLM-facing.
- There is an in-development example of Fortitude located at `/home/cyonx/Documents/GitHub/concordia/fortitude`

Goal:
I want to create a new version of DPS called CE-DPS that is a matured and enhanced version of the DPS methodology, which includes process improvements inspired by CE, along with the integration of a fortitude knowledge management platform which will replace both CE's RAG strategy as well as DPS's local "reference library" strategy. This new CE-DPS should keep all of the best parts of DPS, while improving individual DPS processes with any CE techniques and automation processes that are more polished or robust than the DPS methods, as well intgrates fortitude knowledge management. The end result should be a process that is easy for humans to use, repeatable, reliable, consistent, and leverages the best strategies we have to provide the best context and instruction possible to the codeassist as it executes the roles assigned to it.

**Important**: This methodology hinges on a Human providing direction, guidance, and project oversight, while the codeassist implements code creation, code changes, test creation and execution, bug fixes, documentation, etc. The codeassist will be refferencing and following the methodology documents you create in order to perform its role.

Implementation notes:
- CE-DPS should be created at `/home/cyonx/Documents/GitHub/CE-DPS`. 
- CE-DPS should not use the fortitude instance that belongs to the concordia workspace, but rather should have its own new instance of fortitude that is an integrated part of the new CE-DPS project structure. Do not copy the existing reference library from concordia/fortitude/reference_library. this new instance should start clean.
- Fortitude is implemented in rust, however the development methodology is written in markdown files. These markdown files are what instructs the codeassist.
- Create a detailed todo list for all with research / thinking steps, implementation steps, and quality check / success criteria steps.
- Create research / thinking steps that will examine the existing 3 projects to understand why they work the way they do, what they are attempting to accomplish with their methodology, and how they go about doing so.
- Create research / thinking steps before integration steps to plan out the most effective ways to combine the projects per the guidelines I've given you. Consider different ways to meet the "easy for a human to use, repeatable, reliable, consistent, etc" guidelines, and pick the best architecture.
- Create quality check / success critera steps at the end that will validate that the new CE-DPS methodology is logically consistent, and elegantly organized.
- You will be acting in "Agent orchestrator" mode in order to maintain an effective context window and enforce overall quality. 
- You will spawn subagents/subtasks for individual tasks/steps in your todo list.
- Ensure that you provide each subtask has the proper context for its role
- Virtually all markdown documents you will be writing will have an agentic AI coding assistant as the expected audience. Follow LLM-targeted style guidelines at `/home/cyonx/Documents/GitHub/llm-optimized-documentation.md`.
- There needs to be a "Getting Started" type document that acts as the friendly and helpful human-facing document for the project, explaining how the project works, what the expectactions of the human are, things like that. Also provide a walkthrough of using the process with a (fictional) example of how they would perform each step and what CE-DPS does inbetween the human's inputs. This document should be written in traditional human-facing markdown style.

-------------------------------------------------------------------------------

# <project-directive>CE-DPS Creation Prompt</project-directive>

<meta>
  <title>CE-DPS Development Methodology Creation</title>
  <type>implementation</type>
  <audience>ai_assistant</audience>
  <complexity>high</complexity>
  <priority>critical</priority>
</meta>

## <summary priority="critical">Mission Overview</summary>

**Objective**: Create CE-DPS (Context Engineering - Development Process Solution) methodology integrating DPS + CE + Fortitude
**Location**: `/home/cyonx/Documents/GitHub/CE-DPS`
**Philosophy**: AI implements code, humans provide strategic direction
**Constraint**: AI-as-implementer philosophy overrides all source repository patterns

## <philosophy priority="critical">Core Role Definition - NON-NEGOTIABLE</philosophy>

### <human-responsibilities>
**Strategic Authority Only:**
- Define project vision, goals, priorities, constraints
- Approve high-level architecture and design patterns
- Review AI implementation against business requirements
- Handle escalation when AI encounters uncertainty
- Interface with stakeholders and end users
- Guide methodology application and adaptation
</human-responsibilities>

### <ai-responsibilities>
**Tactical Implementation Authority:**
- Write ALL implementation code following human requirements
- Modify existing code based on human direction
- Create comprehensive tests and execute testing workflows
- Identify, analyze, and fix defects in codebase
- Create and maintain ALL technical documentation
- Implement quality gates, linting, validation processes
- Apply learned patterns and best practices consistently
</ai-responsibilities>

## <research-sources priority="high">Required Analysis</research-sources>

### <source-locations>
**Research Repositories:**
- **DPS Foundation**: `/home/cyonx/Documents/GitHub/repo-template` 
- **CE Techniques**: `/home/cyonx/Documents/GitHub/context-engineering-intro`
- **Fortitude Architecture**: `/home/cyonx/Documents/GitHub/concordia/fortitude`
- **LLM Documentation**: `/home/cyonx/Documents/GitHub/llm-optimized-documentation.md`
</source-locations>

### <research-constraints priority="critical">
**AUTHORITY HIERARCHY**: This prompt's AI-implementer philosophy overrides source repository patterns
**ADAPTATION REQUIRED**: Extract systematic approaches, adapt for AI implementation with human oversight
**PHILOSOPHY PRESERVATION**: Maintain AI-as-implementer constraint throughout all research tasks
</research-constraints>

## <orchestration-guidelines priority="high">Subagent Management</orchestration-guidelines>

### <subagent-use-cases>
**Recommended for Subagents:**
- Large research tasks (context window management)
- Parallel information gathering from multiple sources
- Specialized domain analysis requiring deep focus
- File creation with clear templates and patterns
- Independent quality validation with defined criteria

**Reserved for Orchestrator:**
- Integration and synthesis across domains
- Philosophy-critical decisions maintaining core constraints
- Ambiguity resolution requiring project judgment
- Final validation against requirements
- Small tasks where overhead exceeds benefit
</subagent-use-cases>

### <context-inheritance priority="critical">
**Essential Subagent Context (Required for ALL tasks):**
1. **Core Philosophy**: AI-as-implementer constraint
2. **Task Scope**: Specific deliverable and boundaries  
3. **Integration Role**: How output connects to larger project
4. **Authority Rules**: Requirements override conflicting source patterns
5. **Quality Standards**: Format, depth, validation criteria
6. **Dependencies**: Related work and cross-references

**Subagent Prompt Template:**
```
Context: [AI-as-implementer philosophy + specific constraints]
Task: [Deliverable and scope]
Sources: [Files/directories to analyze]
Constraints: [Non-negotiable requirements overriding source patterns]
Integration: [Role in larger project]
Format: [Output structure and style]
Validation: [Success criteria]
```
</context-inheritance>

### <post-task-validation>
**After Each Subagent Task:**
- Philosophy Check: Aligns with AI-implementer constraint?
- Integration Assessment: Combines coherently with other components?
- Gap Analysis: Additional context or clarification needed?
- Authority Validation: Project requirements maintained over source patterns?
</post-task-validation>

## <project-structure priority="high">Required Architecture</project-structure>

### <directory-specification>
```
CE-DPS/
├── methodology/                    # Core methodology documents
│   ├── ai-implementation/          # AI-facing implementation guides
│   │   ├── phase-1-planning.md     # AI strategic planning support
│   │   ├── phase-2-sprint.md       # AI sprint implementation
│   │   ├── phase-3-execution.md    # AI code implementation
│   │   ├── quality-framework.md    # AI quality standards
│   │   └── implementation-patterns.md # AI code patterns
│   ├── human-oversight/            # Human-facing oversight guides
│   │   ├── strategic-direction.md  # How humans provide direction
│   │   ├── oversight-framework.md  # Quality oversight processes
│   │   └── escalation-procedures.md # Intervention guidelines
│   └── templates/                  # Implementation templates
├── fortitude/                      # Integrated knowledge management
├── tools/                          # Automation and helper scripts
├── examples/                       # Example implementations
├── reference/                      # Quick reference materials
├── CLAUDE.md                       # Claude Code integration
└── README.md                       # Human-facing getting started
```
</directory-specification>

## <integration-requirements priority="high">Component Synthesis</integration-requirements>

### <dps-foundation>
**Preserve These DPS Strengths:**
- Three-Phase Methodology: Plan → Sprint → Execute with deliverables
- Systematic Quality Gates: Anchor testing and validation frameworks  
- Workflow Architecture: Structured approach with time estimates
- Documentation-Driven Development: Documentation guides implementation
</dps-foundation>

### <ce-enhancements>
**Integrate These CE Techniques:**
- Advanced Context Management: Multi-strategy search and optimization
- Progressive Disclosure: Layer information summary → implementation
- Repository Semantic Analysis: AI understanding of codebase relationships
- Dynamic Context Assembly: Task-specific information construction
</ce-enhancements>

### <fortitude-integration>
**Knowledge Management Replacement:**
- Clean Instance: Start with empty knowledge base (no existing content)
- Proactive Research: Automatic knowledge gap detection and filling
- Classification Engine: Research categorization (Decision/Implementation/etc.)
- Learning System: Continuous improvement from usage patterns
- MCP Integration: Direct Claude Code integration for real-time assistance
</fortitude-integration>

## <documentation-standards priority="high">Content Requirements</documentation-standards>

### <ai-facing-documents>
**Primary Audience: AI Assistants**
**Style**: LLM-optimized patterns with semantic markup, progressive disclosure, token efficiency

**Content Requirements:**
- Explicit AI implementation responsibility instructions
- Code quality standards and implementation patterns
- Error handling and escalation procedures for AI
- Fortitude knowledge system integration patterns
- Quality gates and validation requirements for AI-generated code
</ai-facing-documents>

### <human-facing-documents>
**Primary Audience: Human Directors**
**Style**: Traditional markdown, friendly and approachable

**Content Requirements:**
- Getting Started Guide: Comprehensive walkthrough with realistic example
- Oversight Framework: How humans effectively direct and review AI work
- Strategic Planning: How to provide effective direction to AI implementers
- Quality Assessment: How to evaluate AI implementation quality
</human-facing-documents>

## <implementation-guidelines priority="medium">Specific Requirements</implementation-guidelines>

### <role-clarity>
**Every methodology document must specify:**
- What AI should implement autonomously
- When AI should seek human approval or guidance
- How humans provide strategic direction without micromanaging
- Clear escalation paths when AI encounters uncertainty
</role-clarity>

### <knowledge-management>
**Fortitude Integration:**
- Serves as AI's persistent memory and pattern library
- AI automatically captures implementation patterns for reuse
- Research classification supports AI decision-making
- Knowledge gaps trigger automatic research supporting AI implementation
</knowledge-management>

### <quality-framework>
**AI-Implemented Quality:**
- AI implements comprehensive testing (unit, integration, end-to-end)
- Human oversight focuses on business value and strategic alignment
- Automated quality gates prevent low-quality AI implementations
- Continuous learning improves AI implementation capabilities
</quality-framework>

### <workflow-example>
**Practical Implementation Pattern:**
1. **Human**: Defines feature requirements and success criteria
2. **AI**: Researches patterns, creates implementation plan, seeks approval
3. **Human**: Reviews plan, provides guidance, approves approach
4. **AI**: Implements code, creates tests, generates documentation
5. **Human**: Reviews final implementation against business requirements
6. **AI**: Addresses feedback and ensures quality standards met
</workflow-example>

## <success-criteria priority="medium">Validation Requirements</success-criteria>

### <human-experience>
**Expected Outcomes:**
- Minimal time spent on tactical implementation oversight
- Clear visibility into AI implementation progress and quality
- Effective strategic control without implementation bottlenecks
- Confidence in AI implementation quality and consistency
</human-experience>

### <ai-implementation-quality>
**Required Capabilities:**
- Consistent application of patterns and best practices
- Comprehensive testing and quality assurance
- Clear documentation of implementation decisions
- Proactive problem resolution with minimal human intervention
</ai-implementation-quality>

### <system-integration>
**Integration Success:**
- Seamless Fortitude knowledge management supporting AI decisions
- Effective capture and reuse of implementation patterns
- Continuous improvement in AI implementation capabilities
- Reliable, repeatable development processes
</system-integration>

## <deliverables priority="medium">Required Outputs</deliverables>

### <primary-deliverables>
1. **Complete CE-DPS methodology** with clear AI-implementation focus
2. **Functional Fortitude instance** integrated for knowledge management
3. **Comprehensive documentation** for both AI assistants and human directors
4. **Templates and examples** demonstrating AI-implementation workflows
5. **Tools and automation** supporting the AI-implementation methodology
</primary-deliverables>

## <quality-validation priority="low">Final Verification</quality-validation>

### <validation-checklist>
**The completed CE-DPS methodology must demonstrate:**
- **Logical consistency** across all phases and components
- **Clear role separation** between human direction and AI implementation
- **Practical viability** through realistic examples and templates
- **Systematic approach** that is repeatable and reliable
- **Philosophy adherence** supporting AI-as-implementer model
</validation-checklist>

### <project-outcome>
**Mission Success Definition:**
Enable development teams to leverage AI for tactical implementation while maintaining human strategic control, resulting in faster development cycles with maintained quality and alignment with business objectives.
</project-outcome>

## <critical-execution-notes priority="critical">Failure Prevention</critical-execution-notes>

### <ambiguity-sources>
**Avoid These Identified Problems:**
1. **Philosophy Drift**: Establish AI-as-implementer constraint BEFORE research
2. **Source Contradiction**: Requirements in this prompt override repository patterns
3. **Context Loss**: Include philosophy in ALL subagent tasks
4. **Semantic Confusion**: "Implementation" = writing actual code, not planning
5. **Authority Confusion**: This prompt > source repository patterns
</ambiguity-sources>

### <execution-validation>
**Before Finalizing Any Component:**
- Verify role clarity: AI implements, human oversees?
- Check contradictions: Conflicts with AI-implementation philosophy?
- Test application: Can AI assistant follow these instructions to implement code?
- Validate requirements: Meets original project objectives?
</execution-validation>

-------------------------------------------------------------------------------


The version of fortitude at `CE-DPS/fortitude` was previously developed seperately from CE-DPS. I want to integrate fortitude directly into CE-DPS, and part of that means removing the artifacts of the independant development, such as 
CE-DPS/fortitude's: 
- `/home/cyonx/Documents/GitHub/CE-DPS/fortitude/docs`
- `/home/cyonx/Documents/GitHub/CE-DPS/fortitude/examples`
- `/home/cyonx/Documents/GitHub/CE-DPS/fortitude/methodology`
- `/home/cyonx/Documents/GitHub/CE-DPS/fortitude/tests`
- `/home/cyonx/Documents/GitHub/CE-DPS/fortitude/AI_RULES.md`
- `/home/cyonx/Documents/GitHub/CE-DPS/fortitude/DEVELOPMENT_PROCESS.md`
- `/home/cyonx/Documents/GitHub/CE-DPS/fortitude/GETTING_STARTED.md`. 

I need you to verify that none of these folders, nor the files they contain, serve vital purposes for fortitude to run in its current context as a project within the CE-DPS workspace, and are clear to be deleted.

-------------------------------------------------------------------------------

  build-matrix:
    name: Build Matrix
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]

-------------------------------------------------------------------------------

You are tasked with researching the latest technical guidelines and best practices for authoring markdown documents specifically optimized for consumption by Large Language Models (LLMs). This is an emerging field with active research, so prioritize recent findings, academic sources, and empirically validated approaches.

## Core Research Requirements

1. **Reliability and Consistency**
    - What markdown structures and formatting patterns ensure LLMs interpret documents identically across multiple interactions?
    - How do semantic markup patterns affect parsing consistency?
    - What are the most effective methods for disambiguation in markdown content?
    - Are there specific formatting conventions that reduce interpretation variance?

2. **Comprehension Optimization**
    - Which markdown structural elements improve LLM understanding accuracy?
    - How do heading hierarchies, lists, and code blocks affect comprehension?
    - What role do explicit semantic markers play in LLM document understanding?
    - How should complex information be structured to maximize comprehension?

3. **Traversal and Navigation**
    - What document organization patterns enable efficient LLM information retrieval?
    - How should cross-references and internal linking be implemented?
    - What indexing or tagging strategies improve content discoverability?
    - How do different markdown structural elements affect LLM search and retrieval?

4. **Instruction Following**
    - What markdown patterns most effectively signal procedural content to LLMs?
    - How should step-by-step instructions be formatted for maximum adherence?
    - What structural cues help LLMs distinguish between informational and instructional content?
    - How can sequential dependencies and conditional logic be clearly expressed?

5. **Token Economy**
    - What are the most token-efficient approaches to markdown authoring without compromising clarity?
    - How do different formatting choices affect token consumption vs. comprehension trade-offs?
    - What compression techniques maintain semantic integrity while reducing token usage?
    - How should repetitive or boilerplate content be handled efficiently?

## Required Evidence and Sources

Please prioritize findings from:
- **Academic research papers** on LLM document comprehension and markdown parsing
- **Empirical studies** with quantitative metrics on LLM performance with different markdown structures
- **Industry research** from AI companies on document formatting for LLM consumption
- **Technical benchmarks** measuring parsing accuracy, comprehension rates, and instruction following
- **Recent publications** (2024-2025) on prompt engineering and document optimization

## Specific Technical Details Needed

1. **Quantitative Metrics**
    - Parsing accuracy improvements from specific formatting approaches
    - Comprehension rate differences between structural patterns
    - Token efficiency ratios for different markdown styles
    - Instruction following success rates by formatting method

2. **Concrete Implementation Guidelines**
    - Specific markdown syntax recommendations with empirical backing
    - Document structure templates with proven effectiveness
    - Formatting patterns that consistently improve LLM performance
    - Anti-patterns that demonstrably reduce comprehension or reliability

3. **Emerging Standards and Frameworks**
    - Any proposed or emerging standards for LLM-optimized markdown
    - Frameworks or methodologies for document optimization
    - Tools or validators for assessing markdown LLM-readiness
    - Industry initiatives around LLM document formatting

## Research Context

Focus on technical approaches that would be valuable for:
- Technical documentation teams optimizing for AI consumption
- Developers creating LLM-friendly content management systems
- Researchers working on human-AI collaborative documentation
- Organizations standardizing documentation for LLM integration

## Quality Criteria

Prioritize research that includes:
- Controlled experiments with measurable outcomes
- Reproducible methodologies
- Statistical significance testing
- Peer review or industry validation
- Real-world implementation case studies

Please provide specific citations, quantitative results, and actionable technical recommendations based on empirical evidence rather
than theoretical speculation.


-------------------------------------------------------------------------------

I want to set up a formal project to complete and polish fortitude, The CE-DPS deveopment methodology's RAG solution and knowledge management platform. 

First, you need to understand Fortitude.
Read:
- `/home/cyonx/Documents/GitHub/CE-DPS/fortitude/docs/fortitude-vision-statement.md` in order to understand what fortitude is trying to be.
- `/home/cyonx/Documents/GitHub/CE-DPS/fortitude/docs/fortitude-domain-principles.md` in order to understand how fortitude wants to achieve its goals
- `/home/cyonx/Documents/GitHub/CE-DPS/fortitude/docs/fortitude-technical-integration-model.md` in order to understand how fortitude expects to communicate with the rest of CE-DPS and LLM APIs.

Note that those documents do not describe fortitude as it is, but rather as it aspires to be.

Here is an analysis of what is currently working, not working, and broken in fortitude as it is today:
- `/home/cyonx/Documents/GitHub/CE-DPS/fortitude-state-analysis_LLM-readable.md`

-------------------------------------------------------------------------------

We need to update our `/home/cyonx/Documents/GitHub/CE-DPS/methodology/ai-implementation/llm-style-guidelines.md` to optimize the guidelines with newest research into the topic of "how to write markdown that is intended to be read by an LLM".

The research is located in 3 seperate document in the `/home/cyonx/Documents/GitHub/CE-DPS/methodology/ai-implementation/style_research` folder. Read all 3. They are quite large, so determine if it would be worthwhile to your context window preservation to use subtasks to analyze and report back results. Make sure that subagents have ample context to do their jobs correctly. Have them read the existing llm-style-guidelines.md as part of their context window so they understand what they're trying to improve by reading the research papers. Do not sacrifice context quality to use subagents however. We need the best possible style guidelines since the style guideline is a foundational document for the entire project. It is of critical importance.

llm-style-guidelines.md already works. Our goal is to enhance/improve it, not replace it. Keep what is working, add what is new and useful from the 3 research documents where it fits into the existing guidelines. 

When different ideas, guidelines, or recommendations from the research papers conflict with each other or the existing guidelines, follow the core goal priority list below for choosing what information to put in the new llm-style-guidelines.

Core goals for both the llm-style-guidelines, as well as the documentation that will be produced as a result of following the llm-style-guideline's style guidelines are:
1. Reliability and consistency: The LLM should understand the document the same way each time.
2. Comprehension: The LLM should understand the document as easily as possible, with as few ambiguities as possible.
3. Traversal: The LLM should be able to locate the instructions and topics it needs to reference in a given document.
4. Reliably following steps/instructions: The LLM should understand when it is being given a list of instructions to follow, and not deviate from those instructions.
5. High Quality and easily digestible patterns and examples: LLMs need contextually accurate patterns and examples to understand concepts. We need to provide those when it will assist in comprehension. 
6. Token economy. The writing shuld be economical as possible with token usage, without losing context or undermining any of the other core goals.

You will place the new style guidelines here: `CE-DPS/llm-style-guidelines-new.md`

Do you understand these instructions?

-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------