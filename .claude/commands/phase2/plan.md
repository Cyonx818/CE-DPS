# CE-DPS Phase 2 Implementation Planning

Create detailed implementation plan for selected sprint features with file-level breakdown.

## Instructions

1. **Validate Prerequisites**
   - Check that docs/phases/phase-2-sprint-planning.md exists
   - Verify features have been selected by human in "Selected Features for Sprint" section
   - Confirm sprint goal and duration are defined
   - Ensure Phase 1 is complete with approved feature roadmap

2. **Analyze Selected Features**
   - Review each selected feature for implementation complexity
   - Identify technical dependencies between features
   - Assess integration requirements with existing systems
   - Estimate development effort for each feature

3. **Research Implementation Patterns**
   - Query Fortitude for similar feature implementations if available
   - Research best practices for the technology stack
   - Identify proven patterns for selected feature types
   - Look up security and performance optimization patterns

4. **Create File-Level Implementation Plan**
   - Break down each feature into specific files to be created/modified
   - Design API endpoints, data models, and service layers
   - Plan database schema changes and migrations
   - Design test files for comprehensive coverage

5. **Design Testing Strategy**
   - Plan unit tests for all business logic (>95% coverage target)
   - Design integration tests for API endpoints
   - Create security test scenarios for input validation
   - Plan performance tests for critical paths

6. **Create Implementation Timeline**
   - Sequence features based on dependencies
   - Estimate time for each feature implementation
   - Plan quality gate checkpoints throughout sprint
   - Build in buffer time for testing and refinement

7. **Design Quality Gates**
   - Define code quality standards and linting rules
   - Plan security scan checkpoints
   - Set performance benchmarks for new features
   - Create documentation requirements for each feature

8. **Update Planning Document**
   - Fill in implementation approach section with detailed plan
   - Add resource allocation and timeline estimates
   - Include quality gate definitions and success criteria
   - Mark document ready for human approval

## Expected Output

```
📋 Creating CE-DPS Phase 2 Implementation Plan...

✅ Selected features analyzed
✅ Implementation patterns researched
✅ File-level breakdown created
✅ Testing strategy designed
✅ Quality gates defined
✅ Timeline estimated
✅ Planning document updated

Phase 2 implementation plan complete!

📊 Planning Results:
   - Features: [Number] features planned for implementation
   - Files: [Number] files to be created/modified
   - Tests: [Number] test files planned
   - Duration: [Estimated] sprint duration
   - Dependencies: [Number] feature dependencies identified

📋 Document: docs/phases/phase-2-sprint-planning.md (updated with implementation plan)

Implementation Summary:
- [Feature 1]: [Brief implementation approach]
- [Feature 2]: [Brief implementation approach]
- [Feature N]: [Brief implementation approach]

Next Steps:
1. Human review of implementation approach
2. Approval of timeline and resource allocation
3. Validation of quality gate definitions
4. Run /project:phase2:validate when human review complete
```

## Notes
- Focus on detailed, actionable implementation plans
- Ensure realistic timeline estimates with buffer time
- Plan comprehensive testing strategy upfront
- Prepare clear foundation for Phase 3 implementation