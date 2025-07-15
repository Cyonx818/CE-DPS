# CE-DPS Claude Code Integration

## Overview

This document configures Claude Code for effective use with the CE-DPS (Context Engineered Development Process Suite) methodology. The configuration ensures AI assistants can implement code effectively under human strategic oversight.

## Core Principles

### AI Implementation Authority
- **Code Implementation**: AI writes ALL code, tests, and technical documentation
- **Quality Enforcement**: AI implements comprehensive testing and validation
- **Pattern Application**: AI applies established patterns and best practices
- **Knowledge Integration**: AI leverages Fortitude for pattern lookup and learning
- **Documentation Standards**: AI follows [LLM Documentation Guidelines](methodology/ai-implementation/llm-documentation-guidelines.md) for all generated documentation

### Human Strategic Authority
- **Vision & Direction**: Humans define project vision and business requirements
- **Architecture Approval**: Humans approve system architecture and design decisions
- **Feature Prioritization**: Humans select features and approve sprint scope
- **Business Validation**: Humans validate business value and strategic alignment

## Development Workflow

### Phase 1: Strategic Planning (Human-Led)
```markdown
## Human Responsibilities
1. Define business requirements and success metrics
2. Set strategic constraints and technical requirements
3. Review and approve AI-proposed architecture
4. Validate feature roadmap and timeline

## AI Responsibilities
1. Research architectural patterns and best practices
2. Design system architecture with security and scalability
3. Create detailed feature roadmap with effort estimates
4. Identify risks and propose mitigation strategies

## Approval Process
- Human reviews AI analysis and architecture proposals
- Human approves or requests changes to approach
- Human signs off on Phase 1 completion
```

### Phase 2: Sprint Planning (AI-Led with Human Approval)
```markdown
## AI Responsibilities
1. Analyze selected features for implementation complexity
2. Create detailed implementation plans with file-level breakdown
3. Research knowledge gaps using parallel subagents
4. Estimate effort and identify dependencies

## Human Responsibilities
1. Select features for sprint based on business priorities
2. Review and approve implementation approach
3. Validate timeline and resource allocation
4. Authorize sprint execution

## Quality Gates
- Implementation plan detailed to file level
- All dependencies identified and researched
- Security and performance considerations addressed
- Testing approach comprehensive and realistic
```

### Phase 3: Implementation (AI-Led with Human Validation)
```markdown
## AI Responsibilities
1. Implement all code using test-driven development
2. Create comprehensive test coverage (unit, integration, security)
3. Enforce quality gates and standards
4. Generate complete technical documentation

## Human Responsibilities
1. Validate features against business requirements
2. Confirm user experience meets expectations
3. Approve features for production deployment
4. Provide feedback for continuous improvement

## Quality Standards
- >95% test coverage for all business logic
- Security scan passes with no critical vulnerabilities
- Performance meets defined requirements
- Documentation is complete and current
```

## Code Quality Standards

### Security Requirements
```rust
// Always implement comprehensive security patterns
use bcrypt::hash;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation};

// Input validation is mandatory
use validator::Validate;

#[derive(Deserialize, Validate)]
struct UserInput {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8, max = 128))]
    password: String,
}
```

### Error Handling Requirements
```rust
// Use structured error types
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Authentication failed")]
    Authentication,
}
```

### Testing Requirements
```rust
// Comprehensive testing is mandatory
#[cfg(test)]
mod tests {
    use super::*;
    
    // ANCHOR: Core functionality regression test
    #[tokio::test]
    async fn test_user_authentication() {
        // Test implementation with realistic data
        // Must cover success and failure scenarios
        // Must include security validation
    }
}
```

## Fortitude Integration

### Knowledge Management
```bash
# Query existing patterns before implementation
./tools/fortitude-integration.sh query "authentication patterns"

# Update patterns after successful implementation
./tools/fortitude-integration.sh update

# Generate knowledge report
./tools/fortitude-integration.sh report
```

### Pattern Lookup Process
1. **Before Implementation**: Query Fortitude for existing patterns
2. **During Implementation**: Reference proven approaches and templates
3. **After Implementation**: Update knowledge base with new patterns
4. **Continuous Learning**: Capture successful human-AI collaboration patterns

## Tool Integration

### Quality Gates
```bash
# Run comprehensive quality validation
./tools/quality-gates.sh

# Validate phase completion
./tools/phase-validator.py --phase 3
```

### Phase Validation
```bash
# Validate Phase 1 completion
./tools/phase-validator.py --phase 1

# Validate Phase 2 completion
./tools/phase-validator.py --phase 2

# Validate Phase 3 completion
./tools/phase-validator.py --phase 3
```

## Implementation Guidelines

### Code Style
- Use consistent formatting (rustfmt, prettier, etc.)
- Follow established naming conventions
- Include comprehensive error handling
- Add meaningful comments for business logic

### Testing Strategy
- Write tests first (TDD approach)
- Include unit, integration, and security tests
- Achieve >95% test coverage
- Create anchor tests for critical functionality

### Documentation Requirements
- API documentation with examples
- Code comments explaining business logic
- Deployment and configuration guides
- Troubleshooting and error resolution guides

## Communication Patterns

### Escalation Procedures
```markdown
## When to Escalate to Human
1. Ambiguous business requirements
2. Strategic architectural decisions
3. Resource or timeline constraints
4. Quality gate failures requiring business decision

## Escalation Format
- Clear description of issue and impact
- Analysis of options and trade-offs
- Specific decision or guidance needed
- Timeline for decision requirement
```

### Progress Reporting
```markdown
## Implementation Progress Updates
- Daily: Current feature implementation status
- Weekly: Sprint progress against goals
- End of Phase: Comprehensive completion report
- Issues: Immediate escalation with impact assessment
```

## Success Metrics

### Technical Metrics
- Test coverage >95%
- Security scan passes with no critical issues
- Performance meets defined requirements
- Documentation completeness >90%

### Business Metrics
- Features deliver expected business value
- User satisfaction scores meet targets
- Timeline and budget adherence
- Strategic goals advancement

## Continuous Improvement

### Learning Integration
- Capture successful implementation patterns
- Document effective human-AI collaboration approaches
- Refine quality gates based on outcomes
- Optimize development velocity while maintaining quality

### Pattern Evolution
- Update implementation templates based on experience
- Refine testing approaches for better coverage
- Enhance security patterns based on threat landscape
- Improve documentation patterns for clarity

## MCP Server Configuration

```json
{
  "mcpServers": {
    "fortitude": {
      "command": "cargo",
      "args": ["run", "--bin", "fortitude-mcp-server", "--", "--config", "config/ce-dps.toml"],
      "cwd": "/path/to/CE-DPS/fortitude"
    }
  }
}
```

## Environment Variables

```bash
# Set these in your development environment
export CE_DPS_PHASE=1  # Current phase (1=Planning, 2=Sprint, 3=Implementation)
export CE_DPS_FORTITUDE_ENABLED=true  # Enable Fortitude integration
export CE_DPS_QUALITY_GATES=true  # Enable quality gate enforcement
export CE_DPS_HUMAN_APPROVAL_REQUIRED=true  # Require human approval for strategic decisions
```

This configuration ensures Claude Code operates effectively within the CE-DPS methodology, maintaining the AI-as-implementer philosophy while enabling human strategic oversight.