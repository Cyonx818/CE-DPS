#!/bin/bash
# Fortitude API - Research Operations Examples
# Demonstrates research queries, result retrieval, and listing

set -e

# Configuration
BASE_URL="${BASE_URL:-http://localhost:8080}"
API_KEY="${API_KEY:-your-api-key-here}"

echo "🔬 Fortitude API - Research Operations"
echo "📡 Base URL: ${BASE_URL}"
echo "🔑 API Key: ${API_KEY:0:8}..."
echo

# Function to make requests with proper formatting
make_request() {
    local method="$1"
    local endpoint="$2"
    local headers="$3"
    local data="$4"
    local description="$5"
    
    echo "📋 ${description}"
    echo "🔗 ${method} ${endpoint}"
    
    if [ -n "$data" ]; then
        curl -s -X "$method" "${BASE_URL}${endpoint}" $headers \
          -H "Content-Type: application/json" \
          -d "$data" | jq '.'
    else
        curl -s -X "$method" "${BASE_URL}${endpoint}" $headers | jq '.'
    fi
    echo
}

# 1. Simple Research Query
SIMPLE_QUERY='{
  "query": "AI-powered content classification algorithms",
  "priority": "high"
}'

make_request "POST" "/api/v1/research" "-H 'X-API-Key: ${API_KEY}'" "$SIMPLE_QUERY" "Simple Research Query"

# 2. Detailed Research Query with Context
DETAILED_QUERY='{
  "query": "Best practices for Rust async programming",
  "context": "Focus on performance and error handling patterns from 2020-2024",
  "priority": "medium",
  "audience_context": {
    "level": "intermediate",
    "domain": "rust",
    "format": "markdown"
  },
  "domain_context": {
    "technology": "rust",
    "architecture": "microservices"
  }
}'

make_request "POST" "/api/v1/research" "-H 'X-API-Key: ${API_KEY}'" "$DETAILED_QUERY" "Detailed Research Query with Context"

# Store research ID for later retrieval (extract from response)
echo "💾 Extracting research ID for retrieval test..."
RESEARCH_RESPONSE=$(curl -s -X POST "${BASE_URL}/api/v1/research" \
  -H "X-API-Key: ${API_KEY}" \
  -H "Content-Type: application/json" \
  -d "$SIMPLE_QUERY")

RESEARCH_ID=$(echo "$RESEARCH_RESPONSE" | jq -r '.data.results[0].id // empty')

if [ -n "$RESEARCH_ID" ] && [ "$RESEARCH_ID" != "null" ]; then
    echo "🔍 Research ID extracted: ${RESEARCH_ID}"
    
    # 3. Retrieve Research Result by ID
    make_request "GET" "/api/v1/research/${RESEARCH_ID}" "-H 'X-API-Key: ${API_KEY}'" "" "Retrieve Research Result by ID"
else
    echo "⚠️  Could not extract research ID, skipping retrieval test"
fi

# 4. List Research Results (default pagination)
make_request "GET" "/api/v1/research" "-H 'X-API-Key: ${API_KEY}'" "" "List Research Results (default)"

# 5. List Research Results with Pagination
make_request "GET" "/api/v1/research?limit=5&offset=0" "-H 'X-API-Key: ${API_KEY}'" "" "List Research Results (paginated)"

# 6. Search Research Results by Query
make_request "GET" "/api/v1/research?query=rust" "-H 'X-API-Key: ${API_KEY}'" "" "Search Research Results by Query"

# 7. Research Query with Urgency
URGENT_QUERY='{
  "query": "Critical security vulnerability in async-std",
  "context": "Need immediate investigation of CVE impact",
  "priority": "urgent"
}'

make_request "POST" "/api/v1/research" "-H 'X-API-Key: ${API_KEY}'" "$URGENT_QUERY" "Urgent Research Query"

# 8. Research Query for Learning Context
LEARNING_QUERY='{
  "query": "Introduction to machine learning for beginners",
  "context": "Educational content suitable for newcomers to ML",
  "priority": "medium",
  "audience_context": {
    "level": "beginner",
    "domain": "ai",
    "format": "markdown"
  }
}'

make_request "POST" "/api/v1/research" "-H 'X-API-Key: ${API_KEY}'" "$LEARNING_QUERY" "Learning-Focused Research Query"

# 9. Complex Research Query
COMPLEX_QUERY='{
  "query": "Distributed caching strategies for microservices",
  "context": "Compare Redis Cluster vs Hazelcast vs Apache Ignite for high-throughput applications",
  "priority": "high",
  "audience_context": {
    "level": "advanced",
    "domain": "devops",
    "format": "json"
  },
  "domain_context": {
    "technology": "distributed-systems",
    "architecture": "microservices"
  }
}'

make_request "POST" "/api/v1/research" "-H 'X-API-Key: ${API_KEY}'" "$COMPLEX_QUERY" "Complex Research Query"

echo "✅ Research operations test completed!"
echo "💡 Research results should include relevance scores and processing times."