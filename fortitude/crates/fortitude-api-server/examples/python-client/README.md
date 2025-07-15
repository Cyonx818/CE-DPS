# Fortitude API - Python Client Examples

This directory contains Python client examples for integrating with the Fortitude API.

## Requirements

```bash
pip install requests aiohttp asyncio typing-extensions
```

## Quick Start

1. **Set your API key**:
```python
import os
os.environ['FORTITUDE_API_KEY'] = 'your-api-key-here'
os.environ['FORTITUDE_BASE_URL'] = 'http://localhost:8080'
```

2. **Basic usage**:
```python
from fortitude_client import FortitudeClient

client = FortitudeClient()
health = client.get_health()
print(f"Server status: {health['status']}")
```

## Available Examples

- [Basic Client](./basic_client.py) - Simple synchronous client implementation
- [Async Client](./async_client.py) - Asynchronous client with asyncio
- [Research Examples](./research_examples.py) - Research operation examples
- [Classification Examples](./classification_examples.py) - Content classification examples
- [Cache Examples](./cache_examples.py) - Cache management examples
- [Performance Testing](./performance_test.py) - Load testing with Python
- [Error Handling](./error_handling.py) - Error handling examples
- [Integration Examples](./integration_examples.py) - Real-world integration scenarios

## Client Features

- **Automatic retries** with exponential backoff
- **Rate limiting** respect
- **Error handling** with detailed error information
- **Type hints** for better IDE support
- **Async/await support** for high-performance applications
- **Request/response logging** for debugging
- **Automatic response parsing** and validation

## Environment Variables

- `FORTITUDE_API_KEY`: Your API authentication key
- `FORTITUDE_BASE_URL`: API server URL (default: http://localhost:8080)
- `FORTITUDE_TIMEOUT`: Request timeout in seconds (default: 30)
- `FORTITUDE_MAX_RETRIES`: Maximum retry attempts (default: 3)
- `FORTITUDE_LOG_LEVEL`: Logging level (default: INFO)

## Error Handling

The client provides structured error handling:

```python
from fortitude_client import FortitudeClient, FortitudeAPIError

try:
    client = FortitudeClient()
    result = client.research("query")
except FortitudeAPIError as e:
    print(f"API Error: {e.error_code} - {e.message}")
    print(f"Request ID: {e.request_id}")
    print(f"Status Code: {e.status_code}")
```

## Performance Tips

1. **Use async client for high concurrency**:
```python
import asyncio
from fortitude_client import AsyncFortitudeClient

async def main():
    async with AsyncFortitudeClient() as client:
        tasks = [client.research(f"query {i}") for i in range(100)]
        results = await asyncio.gather(*tasks)
```

2. **Enable response caching**:
```python
client = FortitudeClient(enable_cache=True, cache_ttl=300)
```

3. **Batch requests when possible**:
```python
# Use list endpoints instead of individual requests
results = client.list_research_results(limit=100)
```

## Testing

Run the test suite:
```bash
python -m pytest tests/
```

Run performance tests:
```bash
python performance_test.py
```