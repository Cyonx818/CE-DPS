"""
Fortitude API Python Client

A comprehensive Python client for the Fortitude API with support for
synchronous and asynchronous operations, error handling, and retries.
"""

import asyncio
import json
import logging
import os
import time
from typing import Any, Dict, List, Optional, Union
from urllib.parse import urljoin

import aiohttp
import requests


# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class FortitudeAPIError(Exception):
    """Custom exception for Fortitude API errors."""
    
    def __init__(self, message: str, error_code: str = None, status_code: int = None, 
                 request_id: str = None, details: str = None):
        super().__init__(message)
        self.message = message
        self.error_code = error_code
        self.status_code = status_code
        self.request_id = request_id
        self.details = details
    
    def __str__(self):
        return f"FortitudeAPIError({self.status_code}): {self.error_code} - {self.message}"


class FortitudeClient:
    """Synchronous Fortitude API client."""
    
    def __init__(self, 
                 api_key: str = None,
                 base_url: str = None,
                 timeout: int = None,
                 max_retries: int = None,
                 enable_cache: bool = False,
                 cache_ttl: int = 300):
        """
        Initialize the Fortitude API client.
        
        Args:
            api_key: API authentication key (or set FORTITUDE_API_KEY env var)
            base_url: API base URL (or set FORTITUDE_BASE_URL env var)
            timeout: Request timeout in seconds (default: 30)
            max_retries: Maximum retry attempts (default: 3)
            enable_cache: Enable response caching (default: False)
            cache_ttl: Cache TTL in seconds (default: 300)
        """
        self.api_key = api_key or os.getenv('FORTITUDE_API_KEY')
        self.base_url = (base_url or os.getenv('FORTITUDE_BASE_URL', 'http://localhost:8080')).rstrip('/')
        self.timeout = timeout or int(os.getenv('FORTITUDE_TIMEOUT', '30'))
        self.max_retries = max_retries or int(os.getenv('FORTITUDE_MAX_RETRIES', '3'))
        
        if not self.api_key:
            raise ValueError("API key is required. Set FORTITUDE_API_KEY environment variable or pass api_key parameter.")
        
        self.session = requests.Session()
        self.session.headers.update({
            'X-API-Key': self.api_key,
            'Content-Type': 'application/json',
            'User-Agent': 'Fortitude-Python-Client/1.0.0'
        })
        
        # Simple in-memory cache
        self.enable_cache = enable_cache
        self.cache_ttl = cache_ttl
        self._cache = {}
        
        logger.info(f"Initialized Fortitude client for {self.base_url}")
    
    def _make_request(self, method: str, endpoint: str, data: Dict = None, params: Dict = None,
                     use_cache: bool = True) -> Dict[str, Any]:
        """Make HTTP request with error handling and retries."""
        url = urljoin(self.base_url, endpoint)
        
        # Check cache for GET requests
        cache_key = f"{method}:{url}:{json.dumps(params or {}, sort_keys=True)}"
        if use_cache and self.enable_cache and method == 'GET' and cache_key in self._cache:
            cache_entry = self._cache[cache_key]
            if time.time() - cache_entry['timestamp'] < self.cache_ttl:
                logger.debug(f"Cache hit for {cache_key}")
                return cache_entry['data']
        
        for attempt in range(self.max_retries + 1):
            try:
                response = self.session.request(
                    method=method,
                    url=url,
                    json=data,
                    params=params,
                    timeout=self.timeout
                )
                
                if response.status_code == 200:
                    result = response.json()
                    
                    # Cache successful GET responses
                    if use_cache and self.enable_cache and method == 'GET':
                        self._cache[cache_key] = {
                            'data': result,
                            'timestamp': time.time()
                        }
                    
                    return result
                
                elif response.status_code in [429, 500, 502, 503, 504]:
                    # Retry on rate limiting or server errors
                    if attempt < self.max_retries:
                        delay = 2 ** attempt  # Exponential backoff
                        logger.warning(f"Request failed with {response.status_code}, retrying in {delay}s...")
                        time.sleep(delay)
                        continue
                
                # Handle client errors and final server errors
                try:
                    error_data = response.json()
                    raise FortitudeAPIError(
                        message=error_data.get('message', 'Unknown error'),
                        error_code=error_data.get('error_code'),
                        status_code=response.status_code,
                        request_id=error_data.get('request_id'),
                        details=error_data.get('details')
                    )
                except json.JSONDecodeError:
                    raise FortitudeAPIError(
                        message=f"HTTP {response.status_code}: {response.text}",
                        status_code=response.status_code
                    )
            
            except requests.exceptions.RequestException as e:
                if attempt < self.max_retries:
                    delay = 2 ** attempt
                    logger.warning(f"Request exception: {e}, retrying in {delay}s...")
                    time.sleep(delay)
                    continue
                raise FortitudeAPIError(f"Request failed: {e}")
        
        raise FortitudeAPIError("Max retries exceeded")
    
    # Health endpoints
    def get_health(self) -> Dict[str, Any]:
        """Get public health status."""
        return self._make_request('GET', '/health')
    
    def get_protected_health(self) -> Dict[str, Any]:
        """Get detailed health status (requires authentication)."""
        return self._make_request('GET', '/api/v1/health/protected')
    
    # Research endpoints
    def research(self, query: str, context: str = None, priority: str = "medium",
                audience_context: Dict = None, domain_context: Dict = None) -> Dict[str, Any]:
        """Perform a research query."""
        data = {
            'query': query,
            'priority': priority
        }
        
        if context:
            data['context'] = context
        if audience_context:
            data['audience_context'] = audience_context
        if domain_context:
            data['domain_context'] = domain_context
        
        return self._make_request('POST', '/api/v1/research', data=data, use_cache=False)
    
    def get_research_result(self, research_id: str) -> Dict[str, Any]:
        """Get a specific research result by ID."""
        return self._make_request('GET', f'/api/v1/research/{research_id}')
    
    def list_research_results(self, limit: int = 20, offset: int = 0, query: str = None) -> Dict[str, Any]:
        """List research results with pagination."""
        params = {'limit': limit, 'offset': offset}
        if query:
            params['query'] = query
        
        return self._make_request('GET', '/api/v1/research', params=params)
    
    # Classification endpoints
    def classify(self, content: str, categories: List[str] = None, 
                context_preferences: Dict = None) -> Dict[str, Any]:
        """Classify content."""
        data = {'content': content}
        
        if categories:
            data['categories'] = categories
        if context_preferences:
            data['context_preferences'] = context_preferences
        
        return self._make_request('POST', '/api/v1/classify', data=data, use_cache=False)
    
    def get_classification_result(self, classification_id: str) -> Dict[str, Any]:
        """Get a specific classification result by ID."""
        return self._make_request('GET', f'/api/v1/classify/{classification_id}')
    
    def list_classification_results(self, limit: int = 20, offset: int = 0, 
                                   category: str = None) -> Dict[str, Any]:
        """List classification results with pagination."""
        params = {'limit': limit, 'offset': offset}
        if category:
            params['category'] = category
        
        return self._make_request('GET', '/api/v1/classify', params=params)
    
    def get_classification_types(self) -> Dict[str, Any]:
        """Get available classification types."""
        return self._make_request('GET', '/api/v1/classify/types')
    
    # Cache endpoints
    def get_cache_stats(self) -> Dict[str, Any]:
        """Get cache statistics."""
        return self._make_request('GET', '/api/v1/cache/stats')
    
    def search_cache(self, query: str = None, limit: int = 20, offset: int = 0,
                    sort: str = 'newest', research_type: str = None, 
                    min_quality: float = None) -> Dict[str, Any]:
        """Search cache entries."""
        params = {'limit': limit, 'offset': offset, 'sort': sort}
        
        if query:
            params['query'] = query
        if research_type:
            params['research_type'] = research_type
        if min_quality is not None:
            params['min_quality'] = min_quality
        
        return self._make_request('GET', '/api/v1/cache/search', params=params)
    
    def get_cache_entry(self, cache_id: str) -> Dict[str, Any]:
        """Get a specific cache entry by ID."""
        return self._make_request('GET', f'/api/v1/cache/{cache_id}')
    
    def close(self):
        """Close the session."""
        self.session.close()
    
    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()


class AsyncFortitudeClient:
    """Asynchronous Fortitude API client."""
    
    def __init__(self, 
                 api_key: str = None,
                 base_url: str = None,
                 timeout: int = None,
                 max_retries: int = None,
                 enable_cache: bool = False,
                 cache_ttl: int = 300):
        """Initialize the async Fortitude API client."""
        self.api_key = api_key or os.getenv('FORTITUDE_API_KEY')
        self.base_url = (base_url or os.getenv('FORTITUDE_BASE_URL', 'http://localhost:8080')).rstrip('/')
        self.timeout = timeout or int(os.getenv('FORTITUDE_TIMEOUT', '30'))
        self.max_retries = max_retries or int(os.getenv('FORTITUDE_MAX_RETRIES', '3'))
        
        if not self.api_key:
            raise ValueError("API key is required")
        
        self.headers = {
            'X-API-Key': self.api_key,
            'Content-Type': 'application/json',
            'User-Agent': 'Fortitude-Python-AsyncClient/1.0.0'
        }
        
        self.enable_cache = enable_cache
        self.cache_ttl = cache_ttl
        self._cache = {}
        self._session = None
    
    async def __aenter__(self):
        self._session = aiohttp.ClientSession(
            headers=self.headers,
            timeout=aiohttp.ClientTimeout(total=self.timeout)
        )
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self._session:
            await self._session.close()
    
    async def _make_request(self, method: str, endpoint: str, data: Dict = None, 
                           params: Dict = None, use_cache: bool = True) -> Dict[str, Any]:
        """Make async HTTP request with error handling and retries."""
        if not self._session:
            raise RuntimeError("Client not initialized. Use 'async with' statement.")
        
        url = urljoin(self.base_url, endpoint)
        
        # Check cache for GET requests
        cache_key = f"{method}:{url}:{json.dumps(params or {}, sort_keys=True)}"
        if use_cache and self.enable_cache and method == 'GET' and cache_key in self._cache:
            cache_entry = self._cache[cache_key]
            if time.time() - cache_entry['timestamp'] < self.cache_ttl:
                return cache_entry['data']
        
        for attempt in range(self.max_retries + 1):
            try:
                async with self._session.request(
                    method=method,
                    url=url,
                    json=data,
                    params=params
                ) as response:
                    
                    if response.status == 200:
                        result = await response.json()
                        
                        # Cache successful GET responses
                        if use_cache and self.enable_cache and method == 'GET':
                            self._cache[cache_key] = {
                                'data': result,
                                'timestamp': time.time()
                            }
                        
                        return result
                    
                    elif response.status in [429, 500, 502, 503, 504]:
                        if attempt < self.max_retries:
                            delay = 2 ** attempt
                            logger.warning(f"Request failed with {response.status}, retrying in {delay}s...")
                            await asyncio.sleep(delay)
                            continue
                    
                    # Handle errors
                    try:
                        error_data = await response.json()
                        raise FortitudeAPIError(
                            message=error_data.get('message', 'Unknown error'),
                            error_code=error_data.get('error_code'),
                            status_code=response.status,
                            request_id=error_data.get('request_id'),
                            details=error_data.get('details')
                        )
                    except aiohttp.ContentTypeError:
                        text = await response.text()
                        raise FortitudeAPIError(
                            message=f"HTTP {response.status}: {text}",
                            status_code=response.status
                        )
            
            except aiohttp.ClientError as e:
                if attempt < self.max_retries:
                    delay = 2 ** attempt
                    logger.warning(f"Request exception: {e}, retrying in {delay}s...")
                    await asyncio.sleep(delay)
                    continue
                raise FortitudeAPIError(f"Request failed: {e}")
        
        raise FortitudeAPIError("Max retries exceeded")
    
    # Async versions of all methods
    async def get_health(self) -> Dict[str, Any]:
        """Get public health status."""
        return await self._make_request('GET', '/health')
    
    async def research(self, query: str, context: str = None, priority: str = "medium",
                      audience_context: Dict = None, domain_context: Dict = None) -> Dict[str, Any]:
        """Perform a research query."""
        data = {
            'query': query,
            'priority': priority
        }
        
        if context:
            data['context'] = context
        if audience_context:
            data['audience_context'] = audience_context
        if domain_context:
            data['domain_context'] = domain_context
        
        return await self._make_request('POST', '/api/v1/research', data=data, use_cache=False)
    
    async def classify(self, content: str, categories: List[str] = None, 
                      context_preferences: Dict = None) -> Dict[str, Any]:
        """Classify content."""
        data = {'content': content}
        
        if categories:
            data['categories'] = categories
        if context_preferences:
            data['context_preferences'] = context_preferences
        
        return await self._make_request('POST', '/api/v1/classify', data=data, use_cache=False)
    
    async def get_cache_stats(self) -> Dict[str, Any]:
        """Get cache statistics."""
        return await self._make_request('GET', '/api/v1/cache/stats')