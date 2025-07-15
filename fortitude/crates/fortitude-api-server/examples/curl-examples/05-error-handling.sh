#!/bin/bash
# Fortitude API - Error Handling Examples
# Demonstrates various error scenarios and response formats

set -e

# Configuration
BASE_URL="${BASE_URL:-http://localhost:8080}"
API_KEY="${API_KEY:-your-api-key-here}"

echo "⚠️ Fortitude API - Error Handling Examples"
echo "📡 Base URL: ${BASE_URL}"
echo "🔑 API Key: ${API_KEY:0:8}..."
echo

# Function to test error scenarios
test_error() {
    local method="$1"
    local endpoint="$2"
    local headers="$3"
    local data="$4"
    local description="$5"
    local expected_status="$6"
    
    echo "📋 ${description}"
    echo "🔗 ${method} ${endpoint}"
    echo "📊 Expected Status: ${expected_status}"
    
    if [ -n "$data" ]; then
        response=$(curl -s -w "\n%{http_code}" -X "$method" "${BASE_URL}${endpoint}" $headers \
          -H "Content-Type: application/json" \
          -d "$data")
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" "${BASE_URL}${endpoint}" $headers)
    fi
    
    status_code=$(echo "$response" | tail -n1)
    response_body=$(echo "$response" | head -n -1)
    
    echo "📊 Actual Status: ${status_code}"
    if [ -n "$response_body" ]; then
        echo "$response_body" | jq '.' 2>/dev/null || echo "$response_body"
    fi
    echo
}

# 1. Authentication Errors

# Missing API Key
test_error "GET" "/api/v1/health/protected" "" "" "Missing API Key" "401"

# Invalid API Key
test_error "GET" "/api/v1/health/protected" "-H 'X-API-Key: invalid-key-12345'" "" "Invalid API Key" "401"

# Empty API Key
test_error "GET" "/api/v1/health/protected" "-H 'X-API-Key: '" "" "Empty API Key" "401"

# 2. Validation Errors

# Empty Research Query
EMPTY_QUERY='{
  "query": "",
  "priority": "high"
}'
test_error "POST" "/api/v1/research" "-H 'X-API-Key: ${API_KEY}'" "$EMPTY_QUERY" "Empty Research Query" "400"

# Query Too Long (over 1000 characters)
LONG_QUERY='{
  "query": "'$(printf 'a%.0s' {1..1001})'",
  "priority": "high"
}'
test_error "POST" "/api/v1/research" "-H 'X-API-Key: ${API_KEY}'" "$LONG_QUERY" "Query Too Long" "400"

# Invalid Priority
INVALID_PRIORITY='{
  "query": "Valid query",
  "priority": "super-urgent"
}'
test_error "POST" "/api/v1/research" "-H 'X-API-Key: ${API_KEY}'" "$INVALID_PRIORITY" "Invalid Priority" "400"

# Missing Required Field
MISSING_FIELD='{
  "priority": "high"
}'
test_error "POST" "/api/v1/research" "-H 'X-API-Key: ${API_KEY}'" "$MISSING_FIELD" "Missing Required Field" "400"

# Empty Classification Content
EMPTY_CONTENT='{
  "content": ""
}'
test_error "POST" "/api/v1/classify" "-H 'X-API-Key: ${API_KEY}'" "$EMPTY_CONTENT" "Empty Classification Content" "400"

# Content Too Long (over 10000 characters)
LONG_CONTENT='{
  "content": "'$(printf 'a%.0s' {1..10001})'"
}'
test_error "POST" "/api/v1/classify" "-H 'X-API-Key: ${API_KEY}'" "$LONG_CONTENT" "Content Too Long" "400"

# 3. JSON Format Errors

# Invalid JSON
test_error "POST" "/api/v1/research" "-H 'X-API-Key: ${API_KEY}'" '{"query": "test"' "Invalid JSON Format" "400"

# Invalid Content-Type
test_error "POST" "/api/v1/research" "-H 'X-API-Key: ${API_KEY}' -H 'Content-Type: text/plain'" '"query": "test"' "Invalid Content-Type" "400"

# 4. Resource Not Found Errors

# Non-existent Research ID
test_error "GET" "/api/v1/research/00000000-0000-0000-0000-000000000000" "-H 'X-API-Key: ${API_KEY}'" "" "Non-existent Research ID" "404"

# Non-existent Classification ID
test_error "GET" "/api/v1/classify/00000000-0000-0000-0000-000000000000" "-H 'X-API-Key: ${API_KEY}'" "" "Non-existent Classification ID" "404"

# Non-existent Cache Entry ID
test_error "GET" "/api/v1/cache/non-existent-cache-key" "-H 'X-API-Key: ${API_KEY}'" "" "Non-existent Cache Entry" "404"

# Invalid UUID Format
test_error "GET" "/api/v1/research/invalid-uuid-format" "-H 'X-API-Key: ${API_KEY}'" "" "Invalid UUID Format" "400"

# 5. Invalid Endpoint Errors
test_error "GET" "/api/v1/nonexistent" "-H 'X-API-Key: ${API_KEY}'" "" "Non-existent Endpoint" "404"

test_error "GET" "/api/v2/research" "-H 'X-API-Key: ${API_KEY}'" "" "Wrong API Version" "404"

# 6. Method Not Allowed Errors
test_error "DELETE" "/api/v1/research" "-H 'X-API-Key: ${API_KEY}'" "" "Method Not Allowed on Research List" "405"

test_error "PUT" "/api/v1/classify" "-H 'X-API-Key: ${API_KEY}'" "" "Method Not Allowed on Classify" "405"

# 7. Query Parameter Validation Errors

# Invalid Pagination Parameters
test_error "GET" "/api/v1/research?limit=-1" "-H 'X-API-Key: ${API_KEY}'" "" "Negative Limit" "400"

test_error "GET" "/api/v1/research?limit=1000" "-H 'X-API-Key: ${API_KEY}'" "" "Limit Too Large" "400"

test_error "GET" "/api/v1/research?offset=-1" "-H 'X-API-Key: ${API_KEY}'" "" "Negative Offset" "400"

# Invalid Sort Parameter
test_error "GET" "/api/v1/cache/search?sort=invalid" "-H 'X-API-Key: ${API_KEY}'" "" "Invalid Sort Parameter" "400"

# Invalid Quality Range
test_error "GET" "/api/v1/cache/search?min_quality=1.5" "-H 'X-API-Key: ${API_KEY}'" "" "Quality Out of Range" "400"

# 8. Permission Errors (Admin required endpoints)

# Try to delete cache entry without admin permissions
test_error "DELETE" "/api/v1/cache/some-cache-key" "-H 'X-API-Key: ${API_KEY}'" "" "Delete Cache Entry (Non-Admin)" "403"

# Try to invalidate cache without admin permissions
INVALIDATE_REQUEST='{
  "pattern": "test:*",
  "dry_run": true
}'
test_error "POST" "/api/v1/cache/invalidate" "-H 'X-API-Key: ${API_KEY}'" "$INVALIDATE_REQUEST" "Cache Invalidation (Non-Admin)" "403"

# Try cache cleanup without admin permissions
test_error "POST" "/api/v1/cache/cleanup" "-H 'X-API-Key: ${API_KEY}'" "" "Cache Cleanup (Non-Admin)" "403"

# 9. Rate Limiting Test (if enabled)
echo "📋 Rate Limiting Test"
echo "🔗 Making rapid requests to trigger rate limiting..."

for i in {1..10}; do
    status=$(curl -s -w "%{http_code}" -o /dev/null -X GET "${BASE_URL}/health" \
      -H "X-API-Key: ${API_KEY}")
    echo "Request $i: Status $status"
    if [ "$status" = "429" ]; then
        echo "✅ Rate limiting triggered on request $i"
        break
    fi
done
echo

# 10. Server Error Simulation
echo "📋 Server Error Simulation"
echo "Note: These would require server-side conditions to trigger"
echo "- 500 Internal Server Error: Server crash or database connection failure"
echo "- 503 Service Unavailable: Server overloaded or maintenance mode"
echo "- 504 Gateway Timeout: Upstream service timeout"
echo

echo "✅ Error handling examples completed!"
echo "💡 Error responses include:"
echo "   - Consistent JSON format"
echo "   - Human-readable error messages"
echo "   - Machine-readable error codes"
echo "   - Request IDs for tracing"
echo "   - Appropriate HTTP status codes"