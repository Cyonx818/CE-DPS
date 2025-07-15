/**
 * Fortitude API JavaScript Client
 * 
 * A comprehensive JavaScript/Node.js client for the Fortitude API with support for
 * async operations, error handling, retries, and caching.
 */

import fetch from 'node-fetch';

/**
 * Custom error class for Fortitude API errors
 */
export class FortitudeAPIError extends Error {
  /**
   * @param {string} message - Error message
   * @param {string|null} errorCode - API error code
   * @param {number|null} statusCode - HTTP status code
   * @param {string|null} requestId - Request ID for tracing
   * @param {string|null} details - Additional error details
   */
  constructor(message, errorCode = null, statusCode = null, requestId = null, details = null) {
    super(message);
    this.name = 'FortitudeAPIError';
    this.message = message;
    this.errorCode = errorCode;
    this.statusCode = statusCode;
    this.requestId = requestId;
    this.details = details;
  }

  toString() {
    return `FortitudeAPIError(${this.statusCode}): ${this.errorCode} - ${this.message}`;
  }
}

/**
 * Fortitude API Client
 */
export class FortitudeClient {
  /**
   * Initialize the Fortitude API client
   * 
   * @param {Object} options - Configuration options
   * @param {string} options.apiKey - API authentication key
   * @param {string} options.baseURL - API base URL
   * @param {number} options.timeout - Request timeout in milliseconds
   * @param {number} options.maxRetries - Maximum retry attempts
   * @param {boolean} options.enableCache - Enable response caching
   * @param {number} options.cacheTTL - Cache TTL in milliseconds
   * @param {Object} options.headers - Additional headers
   */
  constructor(options = {}) {
    this.apiKey = options.apiKey || process.env.FORTITUDE_API_KEY;
    this.baseURL = (options.baseURL || process.env.FORTITUDE_BASE_URL || 'http://localhost:8080').replace(/\/$/, '');
    this.timeout = options.timeout || parseInt(process.env.FORTITUDE_TIMEOUT || '30000');
    this.maxRetries = options.maxRetries || parseInt(process.env.FORTITUDE_MAX_RETRIES || '3');
    this.enableCache = options.enableCache || false;
    this.cacheTTL = options.cacheTTL || 300000; // 5 minutes default
    
    if (!this.apiKey) {
      throw new Error('API key is required. Set FORTITUDE_API_KEY environment variable or pass apiKey option.');
    }

    this.headers = {
      'X-API-Key': this.apiKey,
      'Content-Type': 'application/json',
      'User-Agent': 'Fortitude-JavaScript-Client/1.0.0',
      ...options.headers
    };

    // Simple in-memory cache
    this._cache = new Map();
    
    console.log(`Initialized Fortitude client for ${this.baseURL}`);
  }

  /**
   * Make HTTP request with error handling and retries
   * 
   * @param {string} method - HTTP method
   * @param {string} endpoint - API endpoint
   * @param {Object|null} data - Request body data
   * @param {Object|null} params - URL query parameters
   * @param {boolean} useCache - Whether to use caching for this request
   * @returns {Promise<Object>} Response data
   */
  async _makeRequest(method, endpoint, data = null, params = null, useCache = true) {
    const url = new URL(endpoint, this.baseURL);
    
    // Add query parameters
    if (params) {
      Object.keys(params).forEach(key => {
        if (params[key] !== null && params[key] !== undefined) {
          url.searchParams.append(key, params[key]);
        }
      });
    }

    // Check cache for GET requests
    const cacheKey = `${method}:${url.toString()}`;
    if (useCache && this.enableCache && method === 'GET' && this._cache.has(cacheKey)) {
      const cacheEntry = this._cache.get(cacheKey);
      if (Date.now() - cacheEntry.timestamp < this.cacheTTL) {
        console.debug(`Cache hit for ${cacheKey}`);
        return cacheEntry.data;
      } else {
        this._cache.delete(cacheKey);
      }
    }

    const requestOptions = {
      method,
      headers: this.headers,
      timeout: this.timeout
    };

    if (data) {
      requestOptions.body = JSON.stringify(data);
    }

    for (let attempt = 0; attempt <= this.maxRetries; attempt++) {
      try {
        const response = await fetch(url.toString(), requestOptions);
        
        if (response.status === 200) {
          const result = await response.json();
          
          // Cache successful GET responses
          if (useCache && this.enableCache && method === 'GET') {
            this._cache.set(cacheKey, {
              data: result,
              timestamp: Date.now()
            });
          }
          
          return result;
        }
        
        // Retry on rate limiting or server errors
        if ([429, 500, 502, 503, 504].includes(response.status) && attempt < this.maxRetries) {
          const delay = Math.pow(2, attempt) * 1000; // Exponential backoff
          console.warn(`Request failed with ${response.status}, retrying in ${delay}ms...`);
          await this._sleep(delay);
          continue;
        }
        
        // Handle client errors and final server errors
        let errorData;
        try {
          errorData = await response.json();
        } catch {
          errorData = { message: await response.text() };
        }
        
        throw new FortitudeAPIError(
          errorData.message || 'Unknown error',
          errorData.error_code,
          response.status,
          errorData.request_id,
          errorData.details
        );
        
      } catch (error) {
        if (error instanceof FortitudeAPIError) {
          throw error;
        }
        
        if (attempt < this.maxRetries) {
          const delay = Math.pow(2, attempt) * 1000;
          console.warn(`Request exception: ${error.message}, retrying in ${delay}ms...`);
          await this._sleep(delay);
          continue;
        }
        
        throw new FortitudeAPIError(`Request failed: ${error.message}`);
      }
    }
    
    throw new FortitudeAPIError('Max retries exceeded');
  }

  /**
   * Sleep for specified milliseconds
   * @param {number} ms - Milliseconds to sleep
   */
  async _sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  // Health endpoints

  /**
   * Get public health status
   * @returns {Promise<Object>} Health status
   */
  async getHealth() {
    return this._makeRequest('GET', '/health');
  }

  /**
   * Get detailed health status (requires authentication)
   * @returns {Promise<Object>} Detailed health status
   */
  async getProtectedHealth() {
    return this._makeRequest('GET', '/api/v1/health/protected');
  }

  // Research endpoints

  /**
   * Perform a research query
   * 
   * @param {string} query - Research query
   * @param {Object} options - Research options
   * @param {string} options.context - Optional context
   * @param {string} options.priority - Priority level (low, medium, high, urgent)
   * @param {Object} options.audienceContext - Audience context
   * @param {Object} options.domainContext - Domain context
   * @returns {Promise<Object>} Research results
   */
  async research(query, options = {}) {
    const data = {
      query,
      priority: options.priority || 'medium'
    };

    if (options.context) data.context = options.context;
    if (options.audienceContext) data.audience_context = options.audienceContext;
    if (options.domainContext) data.domain_context = options.domainContext;

    return this._makeRequest('POST', '/api/v1/research', data, null, false);
  }

  /**
   * Get a specific research result by ID
   * @param {string} researchId - Research result ID
   * @returns {Promise<Object>} Research result
   */
  async getResearchResult(researchId) {
    return this._makeRequest('GET', `/api/v1/research/${researchId}`);
  }

  /**
   * List research results with pagination
   * 
   * @param {Object} options - List options
   * @param {number} options.limit - Number of results to return
   * @param {number} options.offset - Number of results to skip
   * @param {string} options.query - Filter by query text
   * @returns {Promise<Object>} Research results list
   */
  async listResearchResults(options = {}) {
    const params = {
      limit: options.limit || 20,
      offset: options.offset || 0
    };

    if (options.query) params.query = options.query;

    return this._makeRequest('GET', '/api/v1/research', null, params);
  }

  // Classification endpoints

  /**
   * Classify content
   * 
   * @param {string} content - Content to classify
   * @param {Object} options - Classification options
   * @param {string[]} options.categories - Optional categories to consider
   * @param {Object} options.contextPreferences - Context detection preferences
   * @returns {Promise<Object>} Classification results
   */
  async classify(content, options = {}) {
    const data = { content };

    if (options.categories) data.categories = options.categories;
    if (options.contextPreferences) data.context_preferences = options.contextPreferences;

    return this._makeRequest('POST', '/api/v1/classify', data, null, false);
  }

  /**
   * Get a specific classification result by ID
   * @param {string} classificationId - Classification result ID
   * @returns {Promise<Object>} Classification result
   */
  async getClassificationResult(classificationId) {
    return this._makeRequest('GET', `/api/v1/classify/${classificationId}`);
  }

  /**
   * List classification results with pagination
   * 
   * @param {Object} options - List options
   * @param {number} options.limit - Number of results to return
   * @param {number} options.offset - Number of results to skip
   * @param {string} options.category - Filter by category
   * @returns {Promise<Object>} Classification results list
   */
  async listClassificationResults(options = {}) {
    const params = {
      limit: options.limit || 20,
      offset: options.offset || 0
    };

    if (options.category) params.category = options.category;

    return this._makeRequest('GET', '/api/v1/classify', null, params);
  }

  /**
   * Get available classification types
   * @returns {Promise<Object>} Available classification types
   */
  async getClassificationTypes() {
    return this._makeRequest('GET', '/api/v1/classify/types');
  }

  // Cache endpoints

  /**
   * Get cache statistics
   * @returns {Promise<Object>} Cache statistics
   */
  async getCacheStats() {
    return this._makeRequest('GET', '/api/v1/cache/stats');
  }

  /**
   * Search cache entries
   * 
   * @param {Object} options - Search options
   * @param {string} options.query - Search query
   * @param {number} options.limit - Number of results to return
   * @param {number} options.offset - Number of results to skip
   * @param {string} options.sort - Sort order (newest, oldest, relevance, hits)
   * @param {string} options.researchType - Filter by research type
   * @param {number} options.minQuality - Minimum quality threshold
   * @returns {Promise<Object>} Cache search results
   */
  async searchCache(options = {}) {
    const params = {
      limit: options.limit || 20,
      offset: options.offset || 0,
      sort: options.sort || 'newest'
    };

    if (options.query) params.query = options.query;
    if (options.researchType) params.research_type = options.researchType;
    if (options.minQuality !== undefined) params.min_quality = options.minQuality;

    return this._makeRequest('GET', '/api/v1/cache/search', null, params);
  }

  /**
   * Get a specific cache entry by ID
   * @param {string} cacheId - Cache entry ID
   * @returns {Promise<Object>} Cache entry
   */
  async getCacheEntry(cacheId) {
    return this._makeRequest('GET', `/api/v1/cache/${cacheId}`);
  }

  // Utility methods

  /**
   * Clear the local cache
   */
  clearCache() {
    this._cache.clear();
    console.log('Cache cleared');
  }

  /**
   * Get cache statistics
   * @returns {Object} Cache stats
   */
  getCacheInfo() {
    return {
      size: this._cache.size,
      enabled: this.enableCache,
      ttl: this.cacheTTL
    };
  }

  /**
   * Test API connectivity
   * @returns {Promise<boolean>} True if API is reachable
   */
  async testConnection() {
    try {
      await this.getHealth();
      return true;
    } catch (error) {
      console.error('Connection test failed:', error.message);
      return false;
    }
  }
}

export default FortitudeClient;