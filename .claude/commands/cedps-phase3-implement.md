---
description: "Trigger Phase 3 AI implementation with test-driven development and quality gates"
allowed-tools: ["read", "write", "bash"]
---

# <context>CE-DPS Phase 3: AI Implementation Execution</context>

## <summary priority="high">
Trigger comprehensive AI implementation of approved sprint features using test-driven development, quality gates, and human business validation.

## <method>AI Implementation Orchestration</method>

### <implementation>
!echo "🚀 Initiating Phase 3 AI Implementation..."

# Validate Phase 3 setup
!if [ ! -f "docs/phases/phase-3-implementation.md" ]; then
    echo "❌ Error: Phase 3 not set up. Run '/cedps-phase3-setup' first."
    exit 1
fi

# Validate sprint backlog exists
!if [ ! -f "docs/phases/phase-3-artifacts/implementation-backlog.md" ]; then
    echo "❌ Error: Sprint backlog not found. Ensure Phase 2 was completed properly."
    exit 1
fi

# Validate we're on the feature branch
!CURRENT_BRANCH=$(git branch --show-current)
!if [[ "$CURRENT_BRANCH" != *"sprint-001-implementation"* ]]; then
    echo "❌ Error: Not on implementation branch. Current branch: $CURRENT_BRANCH"
    echo "💡 Switch to sprint-001-implementation branch or run '/cedps-phase3-setup' again."
    exit 1
fi

# Validate quality gates are functional
!if command -v cargo >/dev/null 2>&1; then
    echo "🔧 Validating quality gates..."
    if ! cargo run --bin quality-gates -- --validate-environment 2>/dev/null; then
        echo "❌ Error: Quality gates validation failed."
        echo "💡 Build quality gates tools or check environment setup."
        exit 1
    fi
fi

# Update project state
!jq '.phase_3_implementation_started = now' docs/ce-dps-state.json > docs/ce-dps-state.tmp && mv docs/ce-dps-state.tmp docs/ce-dps-state.json

# Update implementation tracking
!jq '.status = "implementing" | .implementation_started = now' docs/sprints/sprint-001/implementation/implementation-status.json > docs/sprints/sprint-001/implementation/implementation-status.tmp && mv docs/sprints/sprint-001/implementation/implementation-status.tmp docs/sprints/sprint-001/implementation/implementation-status.json

!echo "✅ Environment validated. Initiating AI implementation..."
!echo "📋 Sprint backlog: docs/phases/phase-3-artifacts/implementation-backlog.md"
!echo "🧪 Test-driven development workflow activated"
!echo "🔍 Beginning systematic feature implementation..."
</implementation>

### <constraints>
- Phase 3 must be set up first
- Sprint backlog must exist from Phase 2
- Must be on feature branch for implementation
- Quality gates must be functional
- jq command required for state management
</constraints>

## <claude-prompt>
I am executing CE-DPS Phase 3 implementation based on the approved sprint backlog.

### <implementation-context>
@docs/phases/phase-3-artifacts/implementation-backlog.md
@docs/phases/phase-1-planning.md
@docs/phases/phase-2-sprint-planning.md

### <implementation-requirements>
Execute comprehensive implementation of approved sprint features using CE-DPS methodology:

#### <test-driven-development>
**TDD Implementation Cycle**:
1. **Write Failing Tests First**: Create comprehensive test suite before implementation
2. **Implement Minimal Code**: Write just enough code to pass tests
3. **Refactor for Quality**: Improve code quality while maintaining test coverage
4. **Repeat**: Continue cycle for each feature component

**Testing Requirements**:
- **Unit Tests**: Every function and method with edge cases
- **Integration Tests**: Component interaction validation
- **End-to-End Tests**: Complete user workflow validation
- **Security Tests**: Authentication, authorization, input validation
- **Performance Tests**: Load testing and benchmark validation
- **Target Coverage**: >95% test coverage for all business logic

#### <implementation-sequence>
**Sequential Implementation Approach**:
1. **Database Layer**: Migrations, models, repository patterns
2. **Business Logic**: Core functionality with comprehensive error handling
3. **API Layer**: Endpoints with validation and authentication
4. **Integration Layer**: External system connections
5. **Quality Validation**: Comprehensive testing and security validation

**For Each Feature**:
- Start with failing tests that define expected behavior
- Implement minimal code to pass tests
- Add comprehensive error handling
- Include security patterns (authentication, authorization, input validation)
- Validate performance requirements
- Update documentation with API examples

#### <quality-gates-integration>
**Quality Gate Execution**:
- **Pre-Implementation**: Validate environment and dependencies
- **During Implementation**: Run tests after each significant change
- **Post-Feature**: Comprehensive validation before moving to next feature
- **Pre-Commit**: Full quality gate validation before human review

**Quality Standards**:
- All tests pass (unit, integration, security)
- >95% test coverage with meaningful tests
- No linting warnings or security vulnerabilities
- Performance requirements met
- Documentation current and complete

#### <security-first-patterns>
**Security Implementation Requirements**:
- **Authentication**: JWT tokens with proper expiration and refresh
- **Authorization**: Role-based access control at all endpoints
- **Input Validation**: Comprehensive sanitization and validation
- **Error Handling**: No sensitive data leaked in error messages
- **SQL Injection Prevention**: Parameterized queries and ORM patterns
- **Rate Limiting**: Protection against abuse and DoS attacks

#### <human-validation-points>
**Business Value Validation**:
- After each feature implementation, provide demo environment
- Document business value delivered and user experience
- Request human validation against business requirements
- Address feedback before proceeding to next feature

### <output-format>
**Use CE-DPS LLM documentation patterns**:
- Apply semantic markup for implementation sections
- Use progressive disclosure for complex implementations
- Document all architectural decisions and trade-offs
- Include comprehensive error handling patterns
- Mark human validation points clearly

### <implementation-workflow>
**Step-by-Step Implementation Process**:

1. **Environment Preparation**:
   - Validate feature branch and development environment
   - Initialize testing framework and quality gates
   - Prepare database migrations and schema

2. **Feature Implementation Loop**:
   For each feature in the approved backlog:
   - Write comprehensive failing tests
   - Implement minimal code to pass tests
   - Add security patterns and error handling
   - Refactor for quality and performance
   - Run quality gates validation
   - Update documentation

3. **Integration Validation**:
   - Run full test suite across all features
   - Validate cross-feature integration
   - Test API contracts and data flow
   - Verify security patterns work together

4. **Human Business Validation**:
   - Provide working demo environment
   - Document business value delivered
   - Request human validation against requirements
   - Address any feedback or concerns

5. **Quality Gate Finalization**:
   - Run comprehensive quality validation
   - Ensure >95% test coverage
   - Validate security and performance
   - Prepare for human review

### <fortitude-integration>
**Knowledge Management During Implementation**:
- Reference existing implementation patterns before creating new ones
- Apply proven security and performance patterns
- Document new patterns discovered during implementation
- Update knowledge base with successful approaches

### <error-handling-requirements>
**Comprehensive Error Management**:
```rust
// Example: Structured error handling
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Authentication failed")]
    Authentication,
    
    #[error("Authorization denied")]
    Authorization,
}

// User-friendly error responses
impl From<ServiceError> for ApiError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::Authentication => ApiError::Unauthorized("Authentication required".to_string()),
            ServiceError::Authorization => ApiError::Forbidden("Access denied".to_string()),
            ServiceError::Validation(msg) => ApiError::BadRequest(msg),
            ServiceError::Database(_) => ApiError::InternalServer("Internal error".to_string()),
        }
    }
}
```

### <anchor-test-patterns>
**Permanent Regression Tests**:
For critical functionality, create anchor tests:
```rust
// ANCHOR: Core authentication regression test
#[tokio::test]
async fn test_user_authentication_flow() {
    // Comprehensive test covering:
    // - User registration
    // - Login with valid credentials
    // - JWT token generation and validation
    // - Token refresh handling
    // - Logout and token invalidation
    // - Edge cases and error scenarios
}
```

### <success-criteria>
**Phase 3 Implementation Success**:
- All approved features implemented with >95% test coverage
- Security patterns integrated throughout implementation
- Performance requirements met with validation
- Business value validated by human oversight
- Quality gates pass comprehensively
- Documentation complete and current
- Integration with existing systems seamless
- Ready for production deployment

### <human-interaction-pattern>
**Human Validation Process**:
After implementing each feature, I will:
1. Provide a working demo environment
2. Document the business value delivered
3. Show how the feature meets the original requirements
4. Request validation against business needs
5. Address any feedback before proceeding

Please begin the comprehensive Phase 3 implementation now, following the test-driven development approach with quality gates and human validation points.
</claude-prompt>

## <human-action-required>
**AI Implementation In Progress... 🚀**

### <current-status>
Claude Code is now executing comprehensive Phase 3 implementation including:
- **Test-Driven Development**: Writing failing tests first, then implementing code
- **Quality Gates**: Running comprehensive validation after each feature
- **Security-First Patterns**: Implementing authentication, authorization, input validation
- **Performance Validation**: Ensuring response times and scalability requirements
- **Human Validation Points**: Requesting business value validation for each feature

### <what-to-expect>
**Claude Code will systematically**:
1. **Implement each feature** from the approved sprint backlog
2. **Write comprehensive tests** achieving >95% coverage
3. **Apply security patterns** for authentication and authorization
4. **Validate performance** against requirements
5. **Request human validation** for business value after each feature
6. **Update documentation** with API examples and usage guides

### <implementation-workflow>
**Implementation Process**:
1. **Feature 1**: Write tests → Implement code → Quality gates → Human validation
2. **Feature 2**: Write tests → Implement code → Quality gates → Human validation
3. **Feature N**: Continue pattern for all approved features
4. **Integration**: Validate cross-feature functionality
5. **Final Validation**: Comprehensive quality gates and human approval

### <your-validation-role>
**For Each Feature, You Will**:
- **Test the functionality** in the demo environment
- **Validate business value** against original requirements
- **Confirm user experience** meets expectations
- **Approve or request changes** before proceeding to next feature

### <validation-criteria>
**Validate Each Feature Against**:
- [ ] **Business Requirements**: Does it solve the intended problem?
- [ ] **User Experience**: Is it intuitive and valuable?
- [ ] **Performance**: Does it meet speed and scalability needs?
- [ ] **Integration**: Does it work seamlessly with existing features?
- [ ] **Security**: Are authentication and authorization working correctly?

### <quality-standards>
**Automated Quality Validation**:
- [ ] >95% test coverage achieved
- [ ] All tests passing (unit, integration, security)
- [ ] No linting warnings or security vulnerabilities
- [ ] Performance benchmarks met
- [ ] Documentation updated and current
- [ ] Error handling comprehensive

### <completion-process>
**When Implementation Is Complete**:
1. **Final Quality Gates**: Comprehensive validation across all features
2. **Integration Testing**: Cross-feature functionality validation
3. **Human Business Validation**: Overall sprint success validation
4. **Run `/cedps-phase3-validate`**: Formal completion validation
</human-action-required>

## <troubleshooting>
### <common-errors>
- **"Phase 3 not set up"**: Run `/cedps-phase3-setup` first
- **"Sprint backlog not found"**: Ensure Phase 2 was completed properly
- **"Not on implementation branch"**: Switch to sprint-001-implementation branch
- **"Quality gates failed"**: Address specific quality issues reported
- **Tests failing**: Review test implementation and fix code
- **Coverage too low**: Add more comprehensive tests
</common-errors>

### <quality-validation>
**Phase 3 Implementation Requirements**:
- [ ] Test-driven development approach followed
- [ ] >95% test coverage achieved for all features
- [ ] Security patterns implemented throughout
- [ ] Performance requirements validated
- [ ] Human validation points honored
- [ ] Quality gates passing comprehensively
- [ ] Documentation complete and current
- [ ] Integration with existing systems seamless
- [ ] Ready for production deployment
</quality-validation>