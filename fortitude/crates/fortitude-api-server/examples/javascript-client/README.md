# Fortitude API - JavaScript/Node.js Client Examples

This directory contains JavaScript and Node.js client examples for integrating with the Fortitude API.

## Requirements

Node.js 16+ required. Install dependencies:

```bash
npm install
```

## Quick Start

1. **Set environment variables**:
```bash
export FORTITUDE_API_KEY="your-api-key-here"
export FORTITUDE_BASE_URL="http://localhost:8080"
```

2. **Basic usage**:
```javascript
import { FortitudeClient } from './fortitude-client.js';

const client = new FortitudeClient();
const health = await client.getHealth();
console.log(`Server status: ${health.status}`);
```

## Available Examples

- [Client Library](./fortitude-client.js) - Main client implementation
- [Research Examples](./research-examples.js) - Research operation examples
- [Classification Examples](./classification-examples.js) - Content classification examples
- [Cache Examples](./cache-examples.js) - Cache management examples
- [Performance Test](./performance-test.js) - Load testing and benchmarks
- [Integration Examples](./integration-examples.js) - Real-world scenarios

## Running Examples

```bash
# Run research examples
npm run research

# Run classification examples
npm run classification

# Run cache examples
npm run cache

# Run performance tests
npm run performance

# Run all tests
npm test
```

## Client Features

- **Promise-based API** with async/await support
- **Automatic retries** with exponential backoff
- **Request/response interceptors** for logging and debugging
- **TypeScript-ready** with JSDoc type annotations
- **Rate limiting** handling
- **Concurrent request** management
- **Response caching** support
- **Error handling** with detailed error information

## Environment Variables

- `FORTITUDE_API_KEY`: Your API authentication key (required)
- `FORTITUDE_BASE_URL`: API server URL (default: http://localhost:8080)
- `FORTITUDE_TIMEOUT`: Request timeout in milliseconds (default: 30000)
- `FORTITUDE_MAX_RETRIES`: Maximum retry attempts (default: 3)
- `FORTITUDE_LOG_LEVEL`: Logging level (default: info)

## Error Handling

The client provides structured error handling:

```javascript
import { FortitudeClient, FortitudeAPIError } from './fortitude-client.js';

try {
  const client = new FortitudeClient();
  const result = await client.research("query");
} catch (error) {
  if (error instanceof FortitudeAPIError) {
    console.error(`API Error: ${error.errorCode} - ${error.message}`);
    console.error(`Request ID: ${error.requestId}`);
    console.error(`Status Code: ${error.statusCode}`);
  } else {
    console.error('Unexpected error:', error);
  }
}
```

## Performance Tips

1. **Use concurrent requests wisely**:
```javascript
import pLimit from 'p-limit';

const limit = pLimit(10); // Limit to 10 concurrent requests
const queries = ['query1', 'query2', 'query3'];

const results = await Promise.all(
  queries.map(query => 
    limit(() => client.research(query))
  )
);
```

2. **Enable response caching**:
```javascript
const client = new FortitudeClient({
  enableCache: true,
  cacheTTL: 300000 // 5 minutes
});
```

3. **Use request batching**:
```javascript
// Use list endpoints instead of individual requests
const results = await client.listResearchResults({ limit: 100 });
```

## Browser Usage

For browser usage, you may need to handle CORS and use a different fetch implementation:

```html
<script type="module">
import { FortitudeClient } from './fortitude-client.js';

// Configure for browser environment
const client = new FortitudeClient({
  apiKey: 'your-api-key',
  baseURL: 'https://api.fortitude.example.com',
  // Add CORS headers if needed
  headers: {
    'Access-Control-Allow-Origin': '*'
  }
});
</script>
```

## Testing

Run the test suite:
```bash
npm test
```

Run performance tests:
```bash
npm run performance
```

## TypeScript Support

The client includes JSDoc type annotations for TypeScript support:

```typescript
import { FortitudeClient } from './fortitude-client.js';

const client: FortitudeClient = new FortitudeClient();
const result: Promise<any> = client.research("query");
```