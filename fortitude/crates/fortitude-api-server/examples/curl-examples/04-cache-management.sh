#!/bin/bash
# Fortitude API - Cache Management Examples
# Demonstrates cache statistics, search, and management operations

set -e

# Configuration
BASE_URL="${BASE_URL:-http://localhost:8080}"
API_KEY="${API_KEY:-your-api-key-here}"

echo "🗄️ Fortitude API - Cache Management"
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

# 1. Get Cache Statistics
make_request "GET" "/api/v1/cache/stats" "-H 'X-API-Key: ${API_KEY}'" "" "Get Cache Statistics"

# 2. Search Cache Entries (default)
make_request "GET" "/api/v1/cache/search" "-H 'X-API-Key: ${API_KEY}'" "" "Search Cache Entries (default)"

# 3. Search Cache Entries with Query
make_request "GET" "/api/v1/cache/search?query=rust" "-H 'X-API-Key: ${API_KEY}'" "" "Search Cache by Query"

# 4. Search Cache with Pagination
make_request "GET" "/api/v1/cache/search?limit=10&offset=0" "-H 'X-API-Key: ${API_KEY}'" "" "Search Cache with Pagination"

# 5. Search Cache by Research Type
make_request "GET" "/api/v1/cache/search?research_type=implementation" "-H 'X-API-Key: ${API_KEY}'" "" "Search Cache by Research Type"

# 6. Search Cache with Quality Filter
make_request "GET" "/api/v1/cache/search?min_quality=0.8" "-H 'X-API-Key: ${API_KEY}'" "" "Search Cache with Quality Filter"

# 7. Search Cache with Sorting
make_request "GET" "/api/v1/cache/search?sort=hits" "-H 'X-API-Key: ${API_KEY}'" "" "Search Cache Sorted by Hits"

# 8. Search Cache with Multiple Filters
make_request "GET" "/api/v1/cache/search?query=async&research_type=troubleshooting&min_quality=0.7&sort=newest&limit=5" "-H 'X-API-Key: ${API_KEY}'" "" "Search Cache with Multiple Filters"

# First, let's try to get a cache entry ID from the search results
echo "💾 Extracting cache entry ID for detailed operations..."
SEARCH_RESPONSE=$(curl -s -X GET "${BASE_URL}/api/v1/cache/search?limit=1" \
  -H "X-API-Key: ${API_KEY}")

CACHE_ID=$(echo "$SEARCH_RESPONSE" | jq -r '.data.results[0].id // empty')

if [ -n "$CACHE_ID" ] && [ "$CACHE_ID" != "null" ]; then
    echo "🔍 Cache ID extracted: ${CACHE_ID}"
    
    # 9. Get Cache Entry by ID
    make_request "GET" "/api/v1/cache/${CACHE_ID}" "-H 'X-API-Key: ${API_KEY}'" "" "Get Cache Entry by ID"
    
    # Note: Delete operation requires Admin permissions, so we'll show the command but not execute
    echo "📋 Delete Cache Entry (Admin only - not executed)"
    echo "🔗 DELETE /api/v1/cache/${CACHE_ID}"
    echo "Command: curl -X DELETE \"${BASE_URL}/api/v1/cache/${CACHE_ID}\" -H \"X-API-Key: \${ADMIN_API_KEY}\""
    echo
else
    echo "⚠️  No cache entries found, skipping ID-based operations"
fi

# 10. Cache Invalidation (Dry Run) - Admin only
INVALIDATE_DRY_RUN='{
  "pattern": "research:*",
  "max_age_seconds": 86400,
  "dry_run": true
}'

echo "📋 Cache Invalidation Dry Run (Admin only - showing command)"
echo "🔗 POST /api/v1/cache/invalidate"
echo "Command: curl -X POST \"${BASE_URL}/api/v1/cache/invalidate\" \\"
echo "  -H \"X-API-Key: \${ADMIN_API_KEY}\" \\"
echo "  -H \"Content-Type: application/json\" \\"
echo "  -d '${INVALIDATE_DRY_RUN}'"
echo

# 11. Cache Invalidation by Keys (Dry Run) - Admin only
INVALIDATE_KEYS='{
  "keys": ["research:query:123", "classification:content:456"],
  "dry_run": true
}'

echo "📋 Cache Invalidation by Keys (Admin only - showing command)"
echo "🔗 POST /api/v1/cache/invalidate"
echo "Command: curl -X POST \"${BASE_URL}/api/v1/cache/invalidate\" \\"
echo "  -H \"X-API-Key: \${ADMIN_API_KEY}\" \\"
echo "  -H \"Content-Type: application/json\" \\"
echo "  -d '${INVALIDATE_KEYS}'"
echo

# 12. Selective Cache Invalidation (Dry Run) - Admin only
SELECTIVE_INVALIDATE='{
  "research_type": "troubleshooting",
  "min_quality": 0.5,
  "max_age_seconds": 3600,
  "dry_run": true
}'

echo "📋 Selective Cache Invalidation (Admin only - showing command)"
echo "🔗 POST /api/v1/cache/invalidate"
echo "Command: curl -X POST \"${BASE_URL}/api/v1/cache/invalidate\" \\"
echo "  -H \"X-API-Key: \${ADMIN_API_KEY}\" \\"
echo "  -H \"Content-Type: application/json\" \\"
echo "  -d '${SELECTIVE_INVALIDATE}'"
echo

# 13. Cache Cleanup - Admin only
echo "📋 Cache Cleanup (Admin only - showing command)"
echo "🔗 POST /api/v1/cache/cleanup"
echo "Command: curl -X POST \"${BASE_URL}/api/v1/cache/cleanup\" \\"
echo "  -H \"X-API-Key: \${ADMIN_API_KEY}\""
echo

# 14. Demonstrate Cache Hit Rate Improvement
echo "🚀 Demonstrating Cache Hit Rate Improvement"
echo "Making the same research query twice to show caching effect..."

CACHE_TEST_QUERY='{
  "query": "Rust async performance optimization",
  "priority": "medium"
}'

echo "📋 First Request (likely cache miss)"
echo "🔗 POST /api/v1/research"
FIRST_RESPONSE=$(curl -s -X POST "${BASE_URL}/api/v1/research" \
  -H "X-API-Key: ${API_KEY}" \
  -H "Content-Type: application/json" \
  -d "$CACHE_TEST_QUERY")

FIRST_TIME=$(echo "$FIRST_RESPONSE" | jq -r '.data.processing_time_ms // 0')
echo "Processing time: ${FIRST_TIME}ms"
echo

sleep 1

echo "📋 Second Request (likely cache hit)"
echo "🔗 POST /api/v1/research"
SECOND_RESPONSE=$(curl -s -X POST "${BASE_URL}/api/v1/research" \
  -H "X-API-Key: ${API_KEY}" \
  -H "Content-Type: application/json" \
  -d "$CACHE_TEST_QUERY")

SECOND_TIME=$(echo "$SECOND_RESPONSE" | jq -r '.data.processing_time_ms // 0')
echo "Processing time: ${SECOND_TIME}ms"
echo

if [ "$FIRST_TIME" -gt 0 ] && [ "$SECOND_TIME" -gt 0 ]; then
    if [ "$SECOND_TIME" -lt "$FIRST_TIME" ]; then
        echo "✅ Cache improvement detected: ${FIRST_TIME}ms → ${SECOND_TIME}ms"
        IMPROVEMENT=$((($FIRST_TIME - $SECOND_TIME) * 100 / $FIRST_TIME))
        echo "💰 Performance improvement: ${IMPROVEMENT}%"
    else
        echo "⚠️  No cache improvement detected (this is normal if cache was already warm)"
    fi
fi

# 15. Get Updated Cache Statistics
make_request "GET" "/api/v1/cache/stats" "-H 'X-API-Key: ${API_KEY}'" "" "Get Updated Cache Statistics"

echo "✅ Cache management operations completed!"
echo "💡 Admin operations require elevated API key permissions."
echo "📊 Check cache hit rates to verify caching effectiveness."