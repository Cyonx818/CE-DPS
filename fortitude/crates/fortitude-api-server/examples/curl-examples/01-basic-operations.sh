#!/bin/bash
# Fortitude API - Basic Operations Examples
# Basic health checks and API connectivity testing

set -e

# Configuration
BASE_URL="${BASE_URL:-http://localhost:8080}"
API_KEY="${API_KEY:-your-api-key-here}"

echo "🔧 Fortitude API - Basic Operations"
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
        curl -s -X "$method" "${BASE_URL}${endpoint}" $headers -d "$data" | jq '.'
    else
        curl -s -X "$method" "${BASE_URL}${endpoint}" $headers | jq '.'
    fi
    echo
}

# 1. Public Health Check (no authentication required)
make_request "GET" "/health" "" "" "Public Health Check"

# 2. Protected Health Check (authentication required)
make_request "GET" "/api/v1/health/protected" "-H 'X-API-Key: ${API_KEY}'" "" "Protected Health Check"

# 3. Test Invalid Endpoint (404 error)
echo "📋 Test Invalid Endpoint (404 error)"
echo "🔗 GET /api/v1/invalid"
curl -s -w "%{http_code}" -X GET "${BASE_URL}/api/v1/invalid" \
  -H "X-API-Key: ${API_KEY}" | head -c 3
echo " - Expected 404"
echo

# 4. Test Unauthorized Access (missing API key)
echo "📋 Test Unauthorized Access (missing API key)"
echo "🔗 GET /api/v1/health/protected"
curl -s -w "%{http_code}" -X GET "${BASE_URL}/api/v1/health/protected" | head -c 3
echo " - Expected 401"
echo

# 5. Test Invalid API Key
echo "📋 Test Invalid API Key"
echo "🔗 GET /api/v1/health/protected"
curl -s -w "%{http_code}" -X GET "${BASE_URL}/api/v1/health/protected" \
  -H "X-API-Key: invalid-key" | head -c 3
echo " - Expected 401"
echo

# 6. Get Classification Types (reference data)
make_request "GET" "/api/v1/classify/types" "-H 'X-API-Key: ${API_KEY}'" "" "Get Classification Types"

echo "✅ Basic operations test completed!"
echo "💡 If all tests passed, your API server is properly configured."