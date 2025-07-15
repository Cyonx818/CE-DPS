#!/bin/bash
# Fortitude API - Classification Operations Examples
# Demonstrates content classification, result retrieval, and listing

set -e

# Configuration
BASE_URL="${BASE_URL:-http://localhost:8080}"
API_KEY="${API_KEY:-your-api-key-here}"

echo "📊 Fortitude API - Classification Operations"
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

# 1. Simple Content Classification
SIMPLE_CLASSIFICATION='{
  "content": "This document discusses machine learning approaches for text classification using transformer models and attention mechanisms."
}'

make_request "POST" "/api/v1/classify" "-H 'X-API-Key: ${API_KEY}'" "$SIMPLE_CLASSIFICATION" "Simple Content Classification"

# 2. Classification with Custom Categories
CUSTOM_CATEGORIES='{
  "content": "How do I implement async functions in Rust with proper error handling and performance optimization?",
  "categories": ["technical", "tutorial", "rust", "programming"],
  "context_preferences": {
    "detect_urgency": true,
    "detect_audience": true,
    "detect_domain": true
  }
}'

make_request "POST" "/api/v1/classify" "-H 'X-API-Key: ${API_KEY}'" "$CUSTOM_CATEGORIES" "Classification with Custom Categories"

# Store classification ID for later retrieval
echo "💾 Extracting classification ID for retrieval test..."
CLASSIFICATION_RESPONSE=$(curl -s -X POST "${BASE_URL}/api/v1/classify" \
  -H "X-API-Key: ${API_KEY}" \
  -H "Content-Type: application/json" \
  -d "$SIMPLE_CLASSIFICATION")

CLASSIFICATION_ID=$(echo "$CLASSIFICATION_RESPONSE" | jq -r '.data.id // empty')

if [ -n "$CLASSIFICATION_ID" ] && [ "$CLASSIFICATION_ID" != "null" ]; then
    echo "🔍 Classification ID extracted: ${CLASSIFICATION_ID}"
    
    # 3. Retrieve Classification Result by ID
    make_request "GET" "/api/v1/classify/${CLASSIFICATION_ID}" "-H 'X-API-Key: ${API_KEY}'" "" "Retrieve Classification Result by ID"
else
    echo "⚠️  Could not extract classification ID, skipping retrieval test"
fi

# 4. Technical Documentation Classification
TECHNICAL_DOC='{
  "content": "## Database Connection Pooling in Rust\n\nThis guide explains how to implement efficient database connection pooling using sqlx and deadpool. Connection pooling is essential for high-performance applications that need to handle many concurrent database operations.\n\n### Key Benefits\n- Reduced connection overhead\n- Better resource utilization\n- Improved application performance",
  "context_preferences": {
    "detect_urgency": false,
    "detect_audience": true,
    "detect_domain": true
  }
}'

make_request "POST" "/api/v1/classify" "-H 'X-API-Key: ${API_KEY}'" "$TECHNICAL_DOC" "Technical Documentation Classification"

# 5. Support Request Classification
SUPPORT_REQUEST='{
  "content": "URGENT: Our production API is returning 500 errors for all requests. The error started 10 minutes ago and is affecting all users. We need immediate assistance to diagnose and fix this issue.",
  "categories": ["support", "urgent", "production", "error"],
  "context_preferences": {
    "detect_urgency": true,
    "detect_audience": false,
    "detect_domain": true
  }
}'

make_request "POST" "/api/v1/classify" "-H 'X-API-Key: ${API_KEY}'" "$SUPPORT_REQUEST" "Support Request Classification"

# 6. Educational Content Classification
EDUCATIONAL_CONTENT='{
  "content": "Welcome to Introduction to Programming! In this course, you will learn the fundamentals of computer programming using Python. We will start with basic concepts like variables and functions, then progress to more advanced topics like object-oriented programming and data structures.",
  "categories": ["educational", "beginner", "tutorial"],
  "context_preferences": {
    "detect_urgency": false,
    "detect_audience": true,
    "detect_domain": true
  }
}'

make_request "POST" "/api/v1/classify" "-H 'X-API-Key: ${API_KEY}'" "$EDUCATIONAL_CONTENT" "Educational Content Classification"

# 7. List Classification Results (default pagination)
make_request "GET" "/api/v1/classify" "-H 'X-API-Key: ${API_KEY}'" "" "List Classification Results (default)"

# 8. List Classification Results with Pagination
make_request "GET" "/api/v1/classify?limit=5&offset=0" "-H 'X-API-Key: ${API_KEY}'" "" "List Classification Results (paginated)"

# 9. Filter Classification Results by Category
make_request "GET" "/api/v1/classify?category=technical" "-H 'X-API-Key: ${API_KEY}'" "" "Filter by Technical Category"

# 10. Get Available Classification Types
make_request "GET" "/api/v1/classify/types" "-H 'X-API-Key: ${API_KEY}'" "" "Get Available Classification Types"

# 11. Complex Multi-Category Content
COMPLEX_CONTENT='{
  "content": "This research paper presents a novel approach to distributed machine learning using federated learning techniques. The proposed method demonstrates significant improvements in privacy preservation while maintaining model accuracy. Implementation details are provided for both TensorFlow and PyTorch frameworks.",
  "categories": ["research", "academic", "machine-learning", "distributed-systems"],
  "context_preferences": {
    "detect_urgency": false,
    "detect_audience": true,
    "detect_domain": true
  }
}'

make_request "POST" "/api/v1/classify" "-H 'X-API-Key: ${API_KEY}'" "$COMPLEX_CONTENT" "Complex Multi-Category Classification"

# 12. Code Review Request Classification
CODE_REVIEW='{
  "content": "Please review this Rust function for async database operations. I am concerned about potential race conditions and error handling. The function needs to handle high concurrency and should be production-ready.\n\n```rust\npub async fn get_user_data(pool: &Pool, user_id: i64) -> Result<User, DatabaseError> {\n    // implementation\n}\n```",
  "categories": ["code-review", "rust", "async", "database"],
  "context_preferences": {
    "detect_urgency": true,
    "detect_audience": true,
    "detect_domain": true
  }
}'

make_request "POST" "/api/v1/classify" "-H 'X-API-Key: ${API_KEY}'" "$CODE_REVIEW" "Code Review Request Classification"

echo "✅ Classification operations test completed!"
echo "💡 Classification results should include confidence scores and metadata."