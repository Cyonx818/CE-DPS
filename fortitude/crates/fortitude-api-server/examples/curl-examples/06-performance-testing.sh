#!/bin/bash
# Fortitude API - Performance Testing Examples
# Demonstrates load testing and performance measurement with cURL

set -e

# Configuration
BASE_URL="${BASE_URL:-http://localhost:8080}"
API_KEY="${API_KEY:-your-api-key-here}"
CONCURRENT_REQUESTS="${CONCURRENT_REQUESTS:-10}"
TOTAL_REQUESTS="${TOTAL_REQUESTS:-100}"

echo "⚡ Fortitude API - Performance Testing"
echo "📡 Base URL: ${BASE_URL}"
echo "🔑 API Key: ${API_KEY:0:8}..."
echo "🔢 Concurrent Requests: ${CONCURRENT_REQUESTS}"
echo "📊 Total Requests: ${TOTAL_REQUESTS}"
echo

# Create temporary directory for results
RESULTS_DIR="/tmp/fortitude-perf-test-$$"
mkdir -p "$RESULTS_DIR"

echo "📁 Results Directory: ${RESULTS_DIR}"
echo

# Function to run single request and measure time
single_request() {
    local endpoint="$1"
    local method="$2"
    local headers="$3"
    local data="$4"
    local result_file="$5"
    
    start_time=$(date +%s%3N)
    
    if [ -n "$data" ]; then
        response=$(curl -s -w "%{http_code},%{time_total}" -X "$method" "${BASE_URL}${endpoint}" $headers \
          -H "Content-Type: application/json" \
          -d "$data")
    else
        response=$(curl -s -w "%{http_code},%{time_total}" -X "$method" "${BASE_URL}${endpoint}" $headers)
    fi
    
    end_time=$(date +%s%3N)
    
    status_code=$(echo "$response" | tail -n1 | cut -d',' -f1)
    curl_time=$(echo "$response" | tail -n1 | cut -d',' -f2)
    total_time=$((end_time - start_time))
    
    echo "${total_time},${curl_time},${status_code}" >> "$result_file"
}

# Function to run concurrent requests
run_concurrent_test() {
    local test_name="$1"
    local endpoint="$2"
    local method="$3"
    local headers="$4"
    local data="$5"
    local num_requests="$6"
    local concurrency="$7"
    
    echo "🚀 Running ${test_name}"
    echo "📊 ${num_requests} requests with ${concurrency} concurrent connections"
    
    result_file="${RESULTS_DIR}/${test_name// /_}.csv"
    echo "total_time_ms,curl_time_s,status_code" > "$result_file"
    
    # Run requests in background
    pids=()
    for ((i=1; i<=num_requests; i++)); do
        single_request "$endpoint" "$method" "$headers" "$data" "$result_file" &
        pids+=($!)
        
        # Limit concurrency
        if [ ${#pids[@]} -ge $concurrency ]; then
            wait ${pids[0]}
            pids=("${pids[@]:1}")
        fi
        
        # Small delay to prevent overwhelming
        sleep 0.01
    done
    
    # Wait for remaining requests
    for pid in "${pids[@]}"; do
        wait $pid
    done
    
    # Analyze results
    analyze_results "$result_file" "$test_name"
    echo
}

# Function to analyze results
analyze_results() {
    local result_file="$1"
    local test_name="$2"
    
    if [ ! -f "$result_file" ] || [ "$(wc -l < "$result_file")" -lt 2 ]; then
        echo "❌ No results to analyze for ${test_name}"
        return
    fi
    
    # Calculate statistics using awk
    stats=$(awk -F',' 'NR>1 {
        total_time += $1
        curl_time += $2
        if ($3 == 200) success++
        else failure++
        
        if (NR == 2) {
            min_time = max_time = $1
            min_curl = max_curl = $2
        } else {
            if ($1 < min_time) min_time = $1
            if ($1 > max_time) max_time = $1
            if ($2 < min_curl) min_curl = $2
            if ($2 > max_curl) max_curl = $2
        }
        count++
    } END {
        if (count > 0) {
            printf "%.2f,%.2f,%.0f,%.0f,%.4f,%.4f,%d,%d\n", 
                total_time/count, max_time, min_time, max_time,
                curl_time/count, max_curl, success, failure
        }
    }' "$result_file")
    
    if [ -n "$stats" ]; then
        IFS=',' read -r avg_time max_time min_time max_time_dup avg_curl max_curl success failure <<< "$stats"
        
        echo "📊 Results for ${test_name}:"
        echo "   ✅ Successful requests: ${success}"
        echo "   ❌ Failed requests: ${failure}"
        echo "   ⏱️  Average response time: ${avg_time}ms"
        echo "   🔥 Max response time: ${max_time}ms"
        echo "   ⚡ Min response time: ${min_time}ms"
        echo "   🌐 Average cURL time: ${avg_curl}s"
        echo "   📈 Max cURL time: ${max_curl}s"
        
        if [ "$success" -gt 0 ]; then
            success_rate=$(echo "scale=2; $success * 100 / ($success + $failure)" | bc)
            echo "   📊 Success rate: ${success_rate}%"
        fi
        
        # Check performance targets
        if [ "$(echo "$avg_time < 100" | bc)" = "1" ]; then
            echo "   ✅ Sub-100ms average response time target MET"
        else
            echo "   ⚠️  Sub-100ms average response time target MISSED"
        fi
    fi
}

# 1. Health Endpoint Performance Test
run_concurrent_test "Health Check Performance" "/health" "GET" "" "" "$TOTAL_REQUESTS" "$CONCURRENT_REQUESTS"

# 2. Protected Health Endpoint Performance Test  
run_concurrent_test "Protected Health Performance" "/api/v1/health/protected" "GET" "-H 'X-API-Key: ${API_KEY}'" "" "$TOTAL_REQUESTS" "$CONCURRENT_REQUESTS"

# 3. Research Query Performance Test
RESEARCH_QUERY='{
  "query": "Rust async performance optimization patterns",
  "priority": "medium"
}'

run_concurrent_test "Research Query Performance" "/api/v1/research" "POST" "-H 'X-API-Key: ${API_KEY}'" "$RESEARCH_QUERY" "$((TOTAL_REQUESTS / 2))" "$CONCURRENT_REQUESTS"

# 4. Classification Performance Test
CLASSIFICATION_QUERY='{
  "content": "This is a technical document about implementing high-performance web servers in Rust with async/await patterns."
}'

run_concurrent_test "Classification Performance" "/api/v1/classify" "POST" "-H 'X-API-Key: ${API_KEY}'" "$CLASSIFICATION_QUERY" "$((TOTAL_REQUESTS / 2))" "$CONCURRENT_REQUESTS"

# 5. Cache Statistics Performance Test
run_concurrent_test "Cache Stats Performance" "/api/v1/cache/stats" "GET" "-H 'X-API-Key: ${API_KEY}'" "" "$TOTAL_REQUESTS" "$CONCURRENT_REQUESTS"

# 6. Cache Search Performance Test
run_concurrent_test "Cache Search Performance" "/api/v1/cache/search?limit=10" "GET" "-H 'X-API-Key: ${API_KEY}'" "" "$TOTAL_REQUESTS" "$CONCURRENT_REQUESTS"

# 7. Research List Performance Test
run_concurrent_test "Research List Performance" "/api/v1/research?limit=20" "GET" "-H 'X-API-Key: ${API_KEY}'" "" "$TOTAL_REQUESTS" "$CONCURRENT_REQUESTS"

# 8. Classification Types Performance Test
run_concurrent_test "Classification Types Performance" "/api/v1/classify/types" "GET" "-H 'X-API-Key: ${API_KEY}'" "" "$TOTAL_REQUESTS" "$CONCURRENT_REQUESTS"

# 9. Cache Hit Rate Demonstration
echo "🎯 Cache Hit Rate Demonstration"
echo "Testing cache effectiveness with repeated queries..."

CACHE_TEST_QUERY='{
  "query": "Redis caching strategies for high-throughput applications",
  "priority": "high"
}'

echo "📊 Making 10 identical requests to demonstrate caching:"
cache_result_file="${RESULTS_DIR}/cache_hit_test.csv"
echo "request_num,total_time_ms,curl_time_s,status_code" > "$cache_result_file"

for i in {1..10}; do
    echo -n "Request $i: "
    start_time=$(date +%s%3N)
    
    response=$(curl -s -w "%{http_code},%{time_total}" -X POST "${BASE_URL}/api/v1/research" \
      -H "X-API-Key: ${API_KEY}" \
      -H "Content-Type: application/json" \
      -d "$CACHE_TEST_QUERY")
    
    end_time=$(date +%s%3N)
    total_time=$((end_time - start_time))
    
    status_code=$(echo "$response" | tail -n1 | cut -d',' -f1)
    curl_time=$(echo "$response" | tail -n1 | cut -d',' -f2)
    
    echo "${i},${total_time},${curl_time},${status_code}" >> "$cache_result_file"
    echo "${total_time}ms (status: ${status_code})"
    
    sleep 0.5
done

# Analyze cache hit improvement
echo
echo "📊 Cache Hit Rate Analysis:"
first_time=$(awk -F',' 'NR==2 {print $2}' "$cache_result_file")
last_time=$(awk -F',' 'NR==11 {print $2}' "$cache_result_file")

if [ -n "$first_time" ] && [ -n "$last_time" ]; then
    improvement=$(echo "scale=2; ($first_time - $last_time) * 100 / $first_time" | bc 2>/dev/null || echo "0")
    echo "   🚀 First request: ${first_time}ms"
    echo "   ⚡ Last request: ${last_time}ms"
    echo "   📈 Performance improvement: ${improvement}%"
    
    if [ "$(echo "$improvement > 50" | bc)" = "1" ]; then
        echo "   ✅ Cache hit rate target likely MET (>50% improvement)"
    else
        echo "   ⚠️  Cache hit rate target may be MISSED (<50% improvement)"
    fi
fi

# 10. Mixed Workload Test
echo "🔄 Mixed Workload Performance Test"
echo "Simulating realistic API usage patterns..."

mixed_result_file="${RESULTS_DIR}/mixed_workload.csv"
echo "endpoint,total_time_ms,curl_time_s,status_code" > "$mixed_result_file"

# Define mixed workload
endpoints=(
    "/health GET"
    "/api/v1/health/protected GET -H 'X-API-Key: ${API_KEY}'"
    "/api/v1/research POST -H 'X-API-Key: ${API_KEY}' ${RESEARCH_QUERY}"
    "/api/v1/classify POST -H 'X-API-Key: ${API_KEY}' ${CLASSIFICATION_QUERY}"
    "/api/v1/cache/stats GET -H 'X-API-Key: ${API_KEY}'"
    "/api/v1/research?limit=10 GET -H 'X-API-Key: ${API_KEY}'"
)

echo "📊 Running mixed workload (${#endpoints[@]} different endpoints, 50 total requests):"
for i in {1..50}; do
    # Select random endpoint
    endpoint_config="${endpoints[$((RANDOM % ${#endpoints[@]}))]}"
    
    # Parse endpoint config (simplified)
    if echo "$endpoint_config" | grep -q "POST"; then
        endpoint=$(echo "$endpoint_config" | cut -d' ' -f1)
        method="POST"
        if echo "$endpoint_config" | grep -q "research"; then
            data="$RESEARCH_QUERY"
        else
            data="$CLASSIFICATION_QUERY"
        fi
        headers="-H 'X-API-Key: ${API_KEY}'"
    else
        endpoint=$(echo "$endpoint_config" | cut -d' ' -f1)
        method="GET"
        data=""
        if echo "$endpoint_config" | grep -q "X-API-Key"; then
            headers="-H 'X-API-Key: ${API_KEY}'"
        else
            headers=""
        fi
    fi
    
    start_time=$(date +%s%3N)
    
    if [ -n "$data" ]; then
        response=$(curl -s -w "%{http_code},%{time_total}" -X "$method" "${BASE_URL}${endpoint}" $headers \
          -H "Content-Type: application/json" \
          -d "$data" 2>/dev/null || echo "000,0.000")
    else
        response=$(curl -s -w "%{http_code},%{time_total}" -X "$method" "${BASE_URL}${endpoint}" $headers 2>/dev/null || echo "000,0.000")
    fi
    
    end_time=$(date +%s%3N)
    total_time=$((end_time - start_time))
    
    status_code=$(echo "$response" | tail -n1 | cut -d',' -f1)
    curl_time=$(echo "$response" | tail -n1 | cut -d',' -f2)
    
    echo "${endpoint},${total_time},${curl_time},${status_code}" >> "$mixed_result_file"
    echo "Request $i: ${endpoint} - ${total_time}ms (${status_code})"
    
    sleep 0.1
done

analyze_results "$mixed_result_file" "Mixed Workload"

# 11. Summary Report
echo "📋 Performance Test Summary"
echo "=================================="
echo "🔗 Base URL: ${BASE_URL}"
echo "📊 Total Requests: ${TOTAL_REQUESTS}"
echo "🔢 Concurrency: ${CONCURRENT_REQUESTS}"
echo "📁 Results saved to: ${RESULTS_DIR}"
echo
echo "🎯 Performance Targets:"
echo "   ⚡ Sub-100ms response time for cached requests"
echo "   📈 >80% cache hit rate for repeated queries"
echo "   🚀 100+ concurrent requests supported"
echo "   ✅ <1% error rate under normal load"
echo
echo "📊 To analyze detailed results:"
echo "   ls ${RESULTS_DIR}/"
echo "   cat ${RESULTS_DIR}/*.csv"
echo
echo "🧹 Clean up results:"
echo "   rm -rf ${RESULTS_DIR}"

echo "✅ Performance testing completed!"