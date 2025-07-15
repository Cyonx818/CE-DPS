# Fortitude API - cURL Examples

This directory contains comprehensive cURL examples for all Fortitude API endpoints.

## Quick Start

1. **Set your API key** (replace with your actual API key):
```bash
export API_KEY="your-api-key-here"
export BASE_URL="http://localhost:8080"
```

2. **Test the health endpoint** (no auth required):
```bash
curl -X GET "${BASE_URL}/health"
```

3. **Test authenticated endpoints**:
```bash
curl -X GET "${BASE_URL}/api/v1/health/protected" \
  -H "X-API-Key: ${API_KEY}"
```

## Available Examples

- [Basic Operations](./01-basic-operations.sh) - Health checks and basic API testing
- [Research Operations](./02-research-operations.sh) - Research queries and result management
- [Classification Operations](./03-classification-operations.sh) - Content classification
- [Cache Management](./04-cache-management.sh) - Cache operations and statistics
- [Error Handling](./05-error-handling.sh) - Error scenarios and handling
- [Performance Testing](./06-performance-testing.sh) - Load testing with cURL

## Authentication

All endpoints except `/health` require authentication using the `X-API-Key` header:

```bash
curl -H "X-API-Key: your-api-key-here" "${BASE_URL}/api/v1/endpoint"
```

## Rate Limiting

The API enforces rate limiting (60 requests per minute per IP). Include proper delays in scripts:

```bash
# Add delay between requests
sleep 1
```

## Response Format

All successful responses follow this format:
```json
{
  "request_id": "uuid",
  "timestamp": "ISO8601 datetime",
  "success": true,
  "data": { ... }
}
```

Error responses:
```json
{
  "error_code": "ERROR_TYPE",
  "message": "Human readable message",
  "request_id": "uuid",
  "timestamp": "ISO8601 datetime"
}
```

## Environment Variables

Set these environment variables for all examples:
- `API_KEY`: Your API authentication key
- `BASE_URL`: API server URL (default: http://localhost:8080)

## Testing Tips

1. **Use jq for JSON formatting**:
```bash
curl ... | jq '.'
```

2. **Save responses for debugging**:
```bash
curl ... -o response.json
```

3. **Check HTTP status codes**:
```bash
curl -w "%{http_code}" ...
```

4. **Verbose output for debugging**:
```bash
curl -v ...
```