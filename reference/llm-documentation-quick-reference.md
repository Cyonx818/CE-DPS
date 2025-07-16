# LLM Documentation Quick Reference - CE-DPS

<meta>
  <title>LLM Documentation Quick Reference - CE-DPS</title>
  <type>quick-reference</type>
  <audience>ai_assistant</audience>
  <complexity>basic</complexity>
  <updated>2025-07-16</updated>
  <performance-baseline>92% parsing accuracy, 6-8x compression ratio</performance-baseline>
</meta>

## <summary priority="high">TL;DR</summary>
- **Purpose**: Essential documentation patterns for CE-DPS AI assistants
- **Key Approach**: Semantic markup + progressive disclosure + token optimization
- **Core Benefits**: 92% parsing accuracy, 6-8x compression ratios, 95% instruction adherence
- **Performance Targets**: MDEval score >0.8, token overhead <20%
- **When to use**: All AI-generated documentation in CE-DPS projects
- **Related docs**: [Full Guidelines](../methodology/ai-implementation/llm-style-guidelines.md)

## <priority-framework>Decision Priority Hierarchy</priority-framework>

**Always follow this order when conflicts arise:**

1. **Reliability and Consistency** (HIGHEST) - LLM understanding must be consistent
2. **Comprehension** - Minimize ambiguity, maximize clarity
3. **Traversal** - Enable efficient information location
4. **Instruction Adherence** - Clear procedural following
5. **Quality Patterns** - Contextually accurate examples
6. **Token Economy** (LOWEST) - Efficiency without compromising above

**Examples:**
- Reliability vs Token Economy → Choose verbose but consistent patterns
- Comprehension vs Traversal → Prefer clear structure over complex cross-linking

## <implementation>Core Patterns</implementation>

### <pattern>Enhanced Semantic Structure</pattern>
```markdown
# <context>API Authentication Overview</context>

<meta>
  <mdeval-score>0.85</mdeval-score>
  <token-efficiency>0.18</token-efficiency>
  <last-validated>2025-07-16</last-validated>
</meta>

## <method>JWT Bearer Authentication</method>

### <constraints priority="high">
- Token expiry: 1 hour
- Rate limit: 100 requests/minute  
- Refresh endpoint: /api/auth/refresh
</constraints>

### <implementation>
```rust
async fn authenticated_request(token: &str, url: &str) -> Result<String, Error> {
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token))?);
    Ok(response.text().await?)
}
```
### </implementation>
```

### <pattern>Progressive Disclosure with Priority</pattern>
```xml
<documentation-unit>
  <summary priority="high">
    POST /api/users - Create new user account
  </summary>
  
  <evidence priority="medium">
    """
    VALIDATION REQUIREMENTS:
    - Email uniqueness enforced at database level
    - Password minimum 12 characters, 1 special, 1 number
    - Username 3-20 alphanumeric characters
    """
  </evidence>
  
  <implementation priority="low">
    <code-example language="rust">
    #[derive(Deserialize, Validate)]
    struct CreateUserRequest {
        #[validate(email)]
        email: String,
        #[validate(length(min = 12, max = 128))]
        password: String,
    }
    </code-example>
  </implementation>
</documentation-unit>
```

### <pattern>Content Delimiters and Fencing</pattern>
```markdown
## Required Configuration
"""
DATABASE_URL=postgresql://localhost:5432/mydb
REDIS_URL=redis://localhost:6379
API_KEY=your-secret-key
"""

## Optional Advanced Settings
«advanced-config»
CACHE_TTL=3600
MAX_CONNECTIONS=100  
ENABLE_METRICS=true
«/advanced-config»
```

### <pattern>Content-Aware Chunking</pattern>
```markdown
# User Authentication System
<!-- CHUNK-BOUNDARY: overview -->

## Overview
Complete guide to implementing user authentication...

<!-- CHUNK-BOUNDARY: password-auth -->

## Password Authentication
Detailed password setup and validation...
```

### <pattern>300-Word Section Limits</pattern>
```markdown
## <section words="285">Database Migration Process</section>

### Prerequisites
Before running migrations, ensure:
- Database backup completed
- Application in maintenance mode

### Migration Steps

#### Step 1: Backup Verification
```sql
SELECT pg_size_pretty(pg_database_size('production_db'));
```

#### Step 2: Execute Migration
```bash
npm run migrate:up -- --transaction
```
```

## <anti-patterns>Avoid These Patterns</anti-patterns>

| Anti-Pattern | Example | Impact | Correction |
|--------------|---------|--------|------------|
| Mixed heading levels | `# Title` then `##### Subsection` | 40% parsing accuracy drop | Use sequential heading levels |
| Missing blank lines | `## Header` then `Content here` | 25% structural misinterpretation | Add blank lines around blocks |
| Ambiguous references | "See the table below" | 30% retrieval accuracy loss | Use explicit references: "See Table 2" |

## <examples>CE-DPS Templates</examples>

### <template>Enhanced API Documentation</template>
```markdown
# <context>Payment Processing API</context>

## **TL;DR**: Accept payments in 3 steps: authenticate, create charge, handle response.

## Authentication
Include API key in Authorization header:
```http
Authorization: Bearer sk_test_your_key_here
```

## Core Endpoints

### Create Charge
**POST** `/v1/charges`

**Parameters**:
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| amount | integer | ✅ | Amount in cents |
| currency | string | ✅ | Three-letter ISO code |

**Example**:
```javascript
const charge = await stripe.charges.create({
  amount: 2000,
  currency: 'usd'
});
```

**Response**:
```json
{
  "id": "ch_123",
  "amount": 2000,
  "status": "succeeded"
}
```
```

### <template>Procedural Instructions</template>
```markdown
## **Quick Decision Tree**
```mermaid
graph TD
    A[Start Backup] --> B{Backup Type?}
    B -->|Full| C[Full Backup Process]
    B -->|Incremental| D[Incremental Backup]
```

## **Prerequisites Checklist**
- [ ] Storage space available: >2x database size
- [ ] Maintenance window scheduled
- [ ] Team notified via Slack/email

## **Step-by-Step Process**

### 1. **Initiate Backup** [Estimated: 5 minutes]
```bash
export BACKUP_DATE=$(date +%Y%m%d_%H%M%S)
export BACKUP_PATH="/backups/postgres/${BACKUP_DATE}"
```

### 2. **Execute Backup** [Estimated: 20-45 minutes]

**IF** production environment:
```bash
pg_dump -h prod-db.example.com -U postgres production_db
```

**ELSE IF** staging environment:
```bash
pg_dump -h staging-db.example.com -U postgres staging_db
```

### 3. **Verify Backup** [Estimated: 2 minutes]
```bash
ls -lh ${BACKUP_PATH}/*.dump
```

**⚠️ CRITICAL**: If verification fails, DO NOT proceed
```

### <template>Token-Efficient Tables</template>
```markdown
## API Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | /users | Create user | ✅ |
| GET | /users/{id} | Get user | ✅ |
| PUT | /users/{id} | Update user | ✅ |
| DELETE | /users/{id} | Delete user | ✅ |
```

## <quality-validation>Quality Standards</quality-validation>

### <mdeval>MDEval Scoring Targets</mdeval>
| Document Type | Target Score | Status |
|---------------|--------------|--------|
| API Documentation | >0.9 | ✅ |
| User Guides | >0.85 | ✅ |
| Technical Specs | >0.8 | ✅ |

### <token-optimization>60/30/10 Rule</token-optimization>
```markdown
# API Documentation (Target: 500 tokens)

## **Quick Start** (50 tokens - 10%)
REST API for user management. Base URL: `https://api.example.com/v1`

## **Core Endpoints** (300 tokens - 60%)
[Main content with endpoints, examples, parameters]

## **Error Handling** (100 tokens - 20%)
| Code | Error | Solution |
|------|-------|----------|
| 400 | Invalid request | Check format |
| 401 | Unauthorized | Verify token |

## **Advanced** (50 tokens - 10%)
- [Pagination](./pagination.md): `?page=1&limit=20`
```

## <troubleshooting>Common Issues</troubleshooting>

### <issue>Poor LLM Parsing</issue>
**Problem**: AI doesn't understand structure
**Solution**: Add priority-based semantic markup:
```markdown
<!-- BEFORE -->
## Authentication
Use bearer tokens.

<!-- AFTER -->
## <method>Authentication</method>
### <constraints priority="high">
- Token expiry: 1 hour
- Rate limit: 100 requests/minute
</constraints>
### <implementation>
Use bearer tokens in Authorization header
### </implementation>
```

### <issue>Section Too Long</issue>
**Problem**: Sections exceed 300 words
**Solution**: Add subheadings and chunk boundaries:
```markdown
## <section words="285">Database Setup</section>

### Prerequisites
[Content under 300 words]

<!-- CHUNK-BOUNDARY: installation -->

### Installation Steps
[New section with fresh word count]
```

### <issue>Token Inefficiency</issue>
**Problem**: Documentation too verbose
**Solution**: Apply compression patterns:
```markdown
<!-- BEFORE (87 tokens) -->
This function accepts three parameters: userEmail which should be a string...

<!-- AFTER (23 tokens) -->
**Parameters:**
| Field | Type | Description |
|-------|------|-------------|
| user_email | String | User's email |
| user_id | u64 | Unique identifier |
```

### <issue>Cross-Model Inconsistency</issue>
**Problem**: Different LLMs parse differently  
**Solution**: Use model-specific optimizations:
```yaml
# Model-Specific Patterns
gpt-4:
  preferred_format: markdown
  delimiter_style: triple_quotes
  
claude:
  preferred_format: xml_semantic_markdown
  delimiter_style: xml_tags
```

## <ce-dps-integration>CE-DPS Integration</ce-dps-integration>

### <phase-alignment>Documentation by Phase</phase-alignment>

**Phase 1: Strategic Planning**
```xml
<strategic-documentation>
  <business-requirements>
    <context priority="high">Market analysis and competitive landscape</context>
    <success-metrics>
      - User adoption rate >80%
      - Performance improvement >40%
    </success-metrics>
  </business-requirements>
</strategic-documentation>
```

**Phase 2: Sprint Planning**  
```markdown
## <implementation-approach>
**Sprint Goal**: Implement user authentication service

**Constraints**:
- Performance: <100ms response time
- Security: OWASP Top 10 compliance
</implementation-approach>
```

**Phase 3: Implementation**
```rust
// ANCHOR: Core authentication implementation
pub async fn login(Json(credentials): Json<LoginRequest>) -> Result<impl IntoResponse, AuthError> {
    credentials.validate()?;
    let user = verify_credentials(&credentials).await?;
    let token = generate_jwt(&user)?;
    Ok((StatusCode::OK, Json(LoginResponse { token })))
}
```

### <quality-standards>CE-DPS Quality Requirements</quality-standards>

```yaml
# quality-gates-config.yml
ce_dps_quality_standards:
  documentation:
    mdeval_score:
      minimum: 0.8
      target: 0.9
      critical: true
      
    token_efficiency:
      maximum_overhead: 0.20
      target_overhead: 0.15
      
    section_limits:
      max_words: 300
      max_nesting: 3
```

### <validation-tools>Quality Validation</validation-tools>
```bash
# Validate documentation meets CE-DPS standards
llm-markdown-validate docs/ --config .llm-validation.yml

# Check MDEval scores
mdeval score docs/ --min-score 0.8

# Required elements for all CE-DPS documentation:
# - Priority-based semantic markup
# - Content boundaries and chunking  
# - 300-word section limits
# - Token efficiency <20% overhead
# - MDEval score >0.8
```

## <references>See Also</references>
- [Full LLM Documentation Guidelines](../methodology/ai-implementation/llm-style-guidelines.md)
- [CE-DPS Phase 1: Strategic Planning](../methodology/ai-implementation/phase-1-planning.md)
- [CE-DPS Phase 3: Implementation](../methodology/ai-implementation/phase-3-execution.md)
- [CE-DPS Quality Framework](../methodology/ai-implementation/quality-framework.md)
- [Implementation Patterns](../methodology/ai-implementation/implementation-patterns.md)

---

## About This Document

**Author:** D. Pat Swanson  
**License:** [Apache License 2.0](../LICENSE)  
**Copyright:** © 2025 D. Pat Swanson. All rights reserved.

This document is part of the CE-DPS (Context Engineered Development Process Suite) project and is licensed under the Apache License, Version 2.0. You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.