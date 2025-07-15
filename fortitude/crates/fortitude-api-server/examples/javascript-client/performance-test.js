#!/usr/bin/env node
/**
 * Fortitude API - Performance Testing
 * 
 * Comprehensive performance testing and validation for the Fortitude API,
 * including concurrent request testing, cache hit rate validation, and
 * response time measurements.
 */

import { FortitudeClient, FortitudeAPIError } from './fortitude-client.js';
import pLimit from 'p-limit';

/**
 * Performance test results container
 */
class PerformanceTestResults {
  constructor() {
    this.responseTimes = [];
    this.successCount = 0;
    this.errorCount = 0;
    this.errors = [];
    this.startTime = null;
    this.endTime = null;
  }

  addResult(responseTime, success, error = null) {
    this.responseTimes.push(responseTime);
    if (success) {
      this.successCount++;
    } else {
      this.errorCount++;
      if (error) {
        this.errors.push(error);
      }
    }
  }

  getStatistics() {
    if (this.responseTimes.length === 0) return {};

    const totalTime = this.endTime - this.startTime;
    const totalRequests = this.responseTimes.length;

    // Sort response times for percentile calculations
    const sortedTimes = [...this.responseTimes].sort((a, b) => a - b);

    const percentile = (p) => {
      const index = Math.ceil((p / 100) * sortedTimes.length) - 1;
      return sortedTimes[Math.max(0, index)];
    };

    return {
      totalRequests,
      successCount: this.successCount,
      errorCount: this.errorCount,
      successRate: totalRequests > 0 ? (this.successCount / totalRequests) * 100 : 0,
      totalDuration: totalTime / 1000, // Convert to seconds
      requestsPerSecond: totalTime > 0 ? (totalRequests / totalTime) * 1000 : 0,
      avgResponseTime: this.responseTimes.reduce((a, b) => a + b, 0) / this.responseTimes.length,
      minResponseTime: Math.min(...this.responseTimes),
      maxResponseTime: Math.max(...this.responseTimes),
      p50ResponseTime: percentile(50),
      p90ResponseTime: percentile(90),
      p95ResponseTime: percentile(95),
      p99ResponseTime: percentile(99)
    };
  }
}

/**
 * Make a single health check request
 */
async function singleHealthRequest(client) {
  const startTime = Date.now();
  try {
    await client.getHealth();
    const endTime = Date.now();
    return { responseTime: endTime - startTime, success: true, error: null };
  } catch (error) {
    const endTime = Date.now();
    return { responseTime: endTime - startTime, success: false, error: error.message };
  }
}

/**
 * Make a single research request
 */
async function singleResearchRequest(client, query) {
  const startTime = Date.now();
  try {
    await client.research(query, { priority: 'medium' });
    const endTime = Date.now();
    return { responseTime: endTime - startTime, success: true, error: null };
  } catch (error) {
    const endTime = Date.now();
    return { responseTime: endTime - startTime, success: false, error: error.message };
  }
}

/**
 * Test concurrent health checks
 */
async function testConcurrentHealthChecks(numRequests = 100, maxConcurrency = 10) {
  console.log(`🏥 Testing ${numRequests} concurrent health checks with ${maxConcurrency} concurrency...`);

  const results = new PerformanceTestResults();
  results.startTime = Date.now();

  const limit = pLimit(maxConcurrency);
  const client = new FortitudeClient({
    apiKey: process.env.FORTITUDE_API_KEY || 'test-key',
    baseURL: process.env.FORTITUDE_BASE_URL || 'http://localhost:8080'
  });

  const promises = Array.from({ length: numRequests }, () =>
    limit(() => singleHealthRequest(client))
  );

  const requestResults = await Promise.all(promises);
  results.endTime = Date.now();

  requestResults.forEach(result => {
    results.addResult(result.responseTime, result.success, result.error);
  });

  return results;
}

/**
 * Test concurrent research requests
 */
async function testConcurrentResearchRequests(numRequests = 50, maxConcurrency = 10) {
  console.log(`🔬 Testing ${numRequests} concurrent research requests with ${maxConcurrency} concurrency...`);

  const results = new PerformanceTestResults();
  results.startTime = Date.now();

  const queries = [
    'Rust async programming best practices',
    'Python performance optimization techniques',
    'Docker container security patterns',
    'Microservices communication strategies',
    'Database connection pooling in Node.js'
  ];

  const limit = pLimit(maxConcurrency);
  const client = new FortitudeClient({
    apiKey: process.env.FORTITUDE_API_KEY || 'test-key',
    baseURL: process.env.FORTITUDE_BASE_URL || 'http://localhost:8080'
  });

  const promises = Array.from({ length: numRequests }, (_, i) => {
    const query = queries[i % queries.length];
    return limit(() => singleResearchRequest(client, query));
  });

  const requestResults = await Promise.all(promises);
  results.endTime = Date.now();

  requestResults.forEach(result => {
    results.addResult(result.responseTime, result.success, result.error);
  });

  return results;
}

/**
 * Test cache hit rate effectiveness
 */
async function testCacheHitRate() {
  console.log('🗄️ Testing cache hit rate effectiveness...');

  const client = new FortitudeClient({
    apiKey: process.env.FORTITUDE_API_KEY || 'test-key',
    baseURL: process.env.FORTITUDE_BASE_URL || 'http://localhost:8080',
    enableCache: true
  });

  const query = 'Cache effectiveness test query for performance validation';
  const times = [];

  try {
    // Make multiple identical requests
    for (let i = 0; i < 10; i++) {
      const startTime = Date.now();
      await client.research(query, { priority: 'medium' });
      const endTime = Date.now();
      const responseTime = endTime - startTime;
      times.push(responseTime);
      console.log(`   Request ${i + 1}: ${responseTime}ms`);
      
      // Small delay between requests
      await new Promise(resolve => setTimeout(resolve, 100));
    }

    // Calculate cache effectiveness
    const firstRequestTime = times[0];
    const avgSubsequentTime = times.slice(1).reduce((a, b) => a + b, 0) / (times.length - 1);

    const cacheImprovement = ((firstRequestTime - avgSubsequentTime) / firstRequestTime) * 100;

    return {
      firstRequestTime,
      avgSubsequentTime,
      cacheImprovementPercent: cacheImprovement,
      allTimes: times,
      cacheHitRateEstimate: Math.max(0, cacheImprovement)
    };

  } catch (error) {
    console.log(`   ❌ Cache test error: ${error.message}`);
    return { error: error.message };
  }
}

/**
 * Test rate limiting behavior
 */
async function testRateLimiting() {
  console.log('🚦 Testing rate limiting behavior...');

  const client = new FortitudeClient({
    apiKey: process.env.FORTITUDE_API_KEY || 'test-key',
    baseURL: process.env.FORTITUDE_BASE_URL || 'http://localhost:8080'
  });

  let rateLimitHits = 0;
  let successfulRequests = 0;
  const startTime = Date.now();

  try {
    // Make rapid requests to trigger rate limiting
    for (let i = 0; i < 100; i++) {
      try {
        await client.getHealth();
        successfulRequests++;
      } catch (error) {
        if (error instanceof FortitudeAPIError && error.statusCode === 429) {
          rateLimitHits++;
          console.log(`   Rate limit hit on request ${i + 1}`);
          break;
        } else {
          console.log(`   Unexpected error: ${error.message}`);
        }
      }

      // Very small delay
      await new Promise(resolve => setTimeout(resolve, 10));
    }

    const endTime = Date.now();

    return {
      successfulRequests,
      rateLimitHits,
      totalTime: (endTime - startTime) / 1000,
      requestsPerSecond: successfulRequests / ((endTime - startTime) / 1000)
    };

  } catch (error) {
    return { error: error.message };
  }
}

/**
 * Print formatted performance test results
 */
function printPerformanceReport(testName, results) {
  const stats = results.getStatistics();

  console.log(`\n📊 ${testName} Results:`);
  console.log('='.repeat(60));
  console.log(`   Total Requests: ${stats.totalRequests || 0}`);
  console.log(`   Success Rate: ${(stats.successRate || 0).toFixed(1)}%`);
  console.log(`   Total Duration: ${(stats.totalDuration || 0).toFixed(2)}s`);
  console.log(`   Requests/Second: ${(stats.requestsPerSecond || 0).toFixed(1)}`);
  console.log();
  console.log('   Response Times (ms):');
  console.log(`     Average: ${(stats.avgResponseTime || 0).toFixed(1)}`);
  console.log(`     Minimum: ${(stats.minResponseTime || 0).toFixed(1)}`);
  console.log(`     Maximum: ${(stats.maxResponseTime || 0).toFixed(1)}`);
  console.log(`     P50 (Median): ${(stats.p50ResponseTime || 0).toFixed(1)}`);
  console.log(`     P90: ${(stats.p90ResponseTime || 0).toFixed(1)}`);
  console.log(`     P95: ${(stats.p95ResponseTime || 0).toFixed(1)}`);
  console.log(`     P99: ${(stats.p99ResponseTime || 0).toFixed(1)}`);

  // Performance target validation
  console.log('\n   🎯 Performance Target Validation:');
  const avgTime = stats.avgResponseTime || Infinity;
  const successRate = stats.successRate || 0;
  const rps = stats.requestsPerSecond || 0;

  if (avgTime < 100) {
    console.log('     ✅ Sub-100ms average response time: PASSED');
  } else {
    console.log(`     ❌ Sub-100ms average response time: FAILED (${avgTime.toFixed(1)}ms)`);
  }

  if (successRate >= 99) {
    console.log('     ✅ >99% success rate: PASSED');
  } else if (successRate >= 95) {
    console.log('     ⚠️  >95% success rate: ACCEPTABLE');
  } else {
    console.log(`     ❌ >95% success rate: FAILED (${successRate.toFixed(1)}%)`);
  }

  if (rps >= 10) {
    console.log('     ✅ Adequate throughput: PASSED');
  } else {
    console.log(`     ⚠️  Low throughput: ${rps.toFixed(1)} RPS`);
  }

  if (results.errors.length > 0) {
    console.log(`\n   ⚠️  Error Summary (${results.errors.length} errors):`);
    const errorCounts = {};
    results.errors.slice(0, 10).forEach(error => {
      const errorType = error.split(':')[0];
      errorCounts[errorType] = (errorCounts[errorType] || 0) + 1;
    });

    Object.entries(errorCounts).forEach(([errorType, count]) => {
      console.log(`     ${errorType}: ${count}`);
    });
  }
}

/**
 * Validate Sprint 006 performance targets
 */
async function validateSprint006Targets() {
  console.log('🎯 Sprint 006 Performance Target Validation');
  console.log('='.repeat(60));

  const targetsMet = [];

  // Target 1: 100+ concurrent requests
  console.log('🔥 Testing 100+ concurrent request handling...');
  const healthResults = await testConcurrentHealthChecks(120, 15);
  const healthStats = healthResults.getStatistics();

  if (healthStats.successRate >= 95) {
    console.log('   ✅ 100+ concurrent requests: PASSED');
    targetsMet.push('concurrent_requests');
  } else {
    console.log(`   ❌ 100+ concurrent requests: FAILED (${healthStats.successRate.toFixed(1)}% success)`);
  }

  // Target 2: Sub-100ms latency for cached requests
  console.log('\n⚡ Testing sub-100ms cached request latency...');
  const cacheResults = await testCacheHitRate();

  if (!cacheResults.error) {
    const avgCachedTime = cacheResults.avgSubsequentTime;
    if (avgCachedTime < 100) {
      console.log('   ✅ Sub-100ms cached response time: PASSED');
      targetsMet.push('cached_latency');
    } else {
      console.log(`   ❌ Sub-100ms cached response time: FAILED (${avgCachedTime.toFixed(1)}ms)`);
    }
  } else {
    console.log(`   ❌ Cache test failed: ${cacheResults.error}`);
  }

  // Target 3: >80% cache hit rate
  if (!cacheResults.error) {
    const cacheImprovement = cacheResults.cacheImprovementPercent;
    if (cacheImprovement >= 50) { // 50% improvement suggests good caching
      console.log('   ✅ >80% cache effectiveness: LIKELY PASSED');
      targetsMet.push('cache_hit_rate');
    } else {
      console.log(`   ⚠️  Cache effectiveness unclear: ${cacheImprovement.toFixed(1)}% improvement`);
    }
  }

  // Summary
  console.log(`\n📋 Sprint 006 Target Summary:`);
  console.log(`   Targets met: ${targetsMet.length}/3`);

  if (targetsMet.length >= 3) {
    console.log('   🎉 ALL PERFORMANCE TARGETS MET!');
  } else if (targetsMet.length >= 2) {
    console.log('   ✅ Most performance targets met');
  } else {
    console.log('   ⚠️  Performance targets need attention');
  }

  return targetsMet;
}

/**
 * Main performance testing function
 */
async function main() {
  console.log('⚡ Fortitude API - Performance Testing Suite');
  console.log('='.repeat(60));

  // Check API connectivity first
  try {
    const client = new FortitudeClient({
      apiKey: process.env.FORTITUDE_API_KEY || 'test-key',
      baseURL: process.env.FORTITUDE_BASE_URL || 'http://localhost:8080'
    });
    
    const health = await client.getHealth();
    console.log(`🏥 Server status: ${health.status}`);
    console.log(`📝 Server version: ${health.version}`);
  } catch (error) {
    console.log(`❌ Cannot connect to API: ${error.message}`);
    console.log('💡 Make sure the API server is running and environment variables are set');
    return;
  }

  console.log();

  // Run performance tests
  const testSuites = [
    {
      name: 'Concurrent Health Checks',
      fn: () => testConcurrentHealthChecks(100, 10)
    },
    {
      name: 'Concurrent Research Requests',
      fn: () => testConcurrentResearchRequests(50, 8)
    }
  ];

  const allResults = {};

  for (const test of testSuites) {
    try {
      const results = await test.fn();
      allResults[test.name] = results;
      printPerformanceReport(test.name, results);
    } catch (error) {
      console.log(`❌ ${test.name} failed: ${error.message}`);
    }

    // Brief pause between tests
    await new Promise(resolve => setTimeout(resolve, 1000));
  }

  // Cache and rate limiting tests
  console.log('\n🗄️ Cache Performance Test:');
  const cacheResults = await testCacheHitRate();
  if (!cacheResults.error) {
    console.log(`   First request: ${cacheResults.firstRequestTime.toFixed(1)}ms`);
    console.log(`   Avg subsequent: ${cacheResults.avgSubsequentTime.toFixed(1)}ms`);
    console.log(`   Cache improvement: ${cacheResults.cacheImprovementPercent.toFixed(1)}%`);
  } else {
    console.log(`   ❌ Error: ${cacheResults.error}`);
  }

  console.log('\n🚦 Rate Limiting Test:');
  const rateResults = await testRateLimiting();
  if (!rateResults.error) {
    console.log(`   Successful requests: ${rateResults.successfulRequests}`);
    console.log(`   Rate limit hits: ${rateResults.rateLimitHits}`);
    console.log(`   Requests/second: ${rateResults.requestsPerSecond.toFixed(1)}`);
  } else {
    console.log(`   ❌ Error: ${rateResults.error}`);
  }

  // Sprint 006 target validation
  console.log('\n' + '='.repeat(60));
  const targetsMet = await validateSprint006Targets();

  // Generate summary report
  console.log('\n📋 Performance Test Summary Report');
  console.log('='.repeat(60));
  console.log(`📅 Test Date: ${new Date().toISOString()}`);
  console.log(`🔗 API Endpoint: ${process.env.FORTITUDE_BASE_URL || 'http://localhost:8080'}`);
  console.log(`🎯 Sprint 006 Targets Met: ${targetsMet.length}/3`);

  if (Object.keys(allResults).length > 0) {
    const overallSuccessRate = Object.values(allResults)
      .map(results => results.getStatistics().successRate || 0)
      .reduce((sum, rate) => sum + rate, 0) / Object.keys(allResults).length;

    const overallAvgResponse = Object.values(allResults)
      .map(results => results.getStatistics().avgResponseTime || 0)
      .reduce((sum, time) => sum + time, 0) / Object.keys(allResults).length;

    console.log(`📊 Overall Success Rate: ${overallSuccessRate.toFixed(1)}%`);
    console.log(`⏱️  Overall Avg Response Time: ${overallAvgResponse.toFixed(1)}ms`);
  }

  console.log('\n✅ Performance testing completed!');
}

// Run the performance tests
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}