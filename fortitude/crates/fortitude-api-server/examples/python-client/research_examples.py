#!/usr/bin/env python3
"""
Fortitude API - Research Operations Examples

Demonstrates various research query patterns and result handling.
"""

import os
import time
from typing import Dict, List

from fortitude_client import FortitudeClient, FortitudeAPIError


def setup_client() -> FortitudeClient:
    """Setup and return a configured client."""
    return FortitudeClient(
        api_key=os.getenv('FORTITUDE_API_KEY', 'your-api-key-here'),
        base_url=os.getenv('FORTITUDE_BASE_URL', 'http://localhost:8080'),
        enable_cache=True
    )


def simple_research_example(client: FortitudeClient):
    """Example: Simple research query."""
    print("🔬 Simple Research Query")
    print("=" * 50)
    
    try:
        result = client.research(
            query="AI-powered content classification algorithms",
            priority="high"
        )
        
        print(f"✅ Request ID: {result['request_id']}")
        print(f"📊 Results found: {result['data']['total_count']}")
        print(f"⏱️  Processing time: {result['data']['processing_time_ms']}ms")
        
        for i, research_result in enumerate(result['data']['results'][:3]):
            print(f"\n📄 Result {i+1}:")
            print(f"   Title: {research_result['title'][:80]}...")
            print(f"   Relevance: {research_result['relevance_score']:.2f}")
            print(f"   Source: {research_result.get('source', 'N/A')}")
        
    except FortitudeAPIError as e:
        print(f"❌ Error: {e}")
    
    print()


def detailed_research_example(client: FortitudeClient):
    """Example: Research with context and audience targeting."""
    print("🎯 Detailed Research with Context")
    print("=" * 50)
    
    try:
        result = client.research(
            query="Best practices for Rust async programming",
            context="Focus on performance and error handling patterns from 2020-2024",
            priority="medium",
            audience_context={
                "level": "intermediate",
                "domain": "rust",
                "format": "markdown"
            },
            domain_context={
                "technology": "rust",
                "architecture": "microservices"
            }
        )
        
        print(f"✅ Request ID: {result['request_id']}")
        print(f"📊 Results found: {result['data']['total_count']}")
        print(f"⏱️  Processing time: {result['data']['processing_time_ms']}ms")
        
        # Store first result ID for retrieval example
        if result['data']['results']:
            first_result_id = result['data']['results'][0]['id']
            print(f"🔍 First result ID: {first_result_id}")
            
            # Retrieve specific result
            specific_result = client.get_research_result(first_result_id)
            print(f"📄 Retrieved result: {specific_result['data']['title']}")
        
    except FortitudeAPIError as e:
        print(f"❌ Error: {e}")
    
    print()


def urgent_research_example(client: FortitudeClient):
    """Example: Urgent research query."""
    print("🚨 Urgent Research Query")
    print("=" * 50)
    
    try:
        result = client.research(
            query="Critical security vulnerability in async-std",
            context="Need immediate investigation of CVE impact",
            priority="urgent"
        )
        
        print(f"✅ Request ID: {result['request_id']}")
        print(f"📊 Results found: {result['data']['total_count']}")
        print(f"⏱️  Processing time: {result['data']['processing_time_ms']}ms")
        print(f"🚨 Priority: URGENT - expedited processing")
        
    except FortitudeAPIError as e:
        print(f"❌ Error: {e}")
    
    print()


def learning_research_example(client: FortitudeClient):
    """Example: Research for learning purposes."""
    print("📚 Learning-Focused Research")
    print("=" * 50)
    
    try:
        result = client.research(
            query="Introduction to machine learning for beginners",
            context="Educational content suitable for newcomers to ML",
            priority="medium",
            audience_context={
                "level": "beginner",
                "domain": "ai",
                "format": "markdown"
            }
        )
        
        print(f"✅ Request ID: {result['request_id']}")
        print(f"📊 Results found: {result['data']['total_count']}")
        print(f"⏱️  Processing time: {result['data']['processing_time_ms']}ms")
        print(f"🎓 Audience: Beginner-friendly content")
        
    except FortitudeAPIError as e:
        print(f"❌ Error: {e}")
    
    print()


def list_research_results_example(client: FortitudeClient):
    """Example: List and search research results."""
    print("📋 List and Search Research Results")
    print("=" * 50)
    
    try:
        # List recent results
        results = client.list_research_results(limit=5, offset=0)
        print(f"📊 Total results available: {results['data']['total_count']}")
        print(f"📄 Showing {len(results['data']['results'])} results")
        
        for i, result in enumerate(results['data']['results']):
            print(f"\n{i+1}. {result['title'][:60]}...")
            print(f"   Relevance: {result['relevance_score']:.2f}")
            print(f"   Created: {result['created_at']}")
        
        # Search with query filter
        print("\n🔍 Searching for 'rust' related results...")
        search_results = client.list_research_results(query="rust", limit=3)
        print(f"📊 Found {len(search_results['data']['results'])} rust-related results")
        
        for result in search_results['data']['results']:
            print(f"   • {result['title'][:50]}...")
        
    except FortitudeAPIError as e:
        print(f"❌ Error: {e}")
    
    print()


def batch_research_example(client: FortitudeClient):
    """Example: Batch research queries for comparison."""
    print("📦 Batch Research Queries")
    print("=" * 50)
    
    queries = [
        "Python async vs Rust async performance",
        "Node.js vs Go microservices architecture",
        "Docker vs Podman container orchestration",
    ]
    
    results = []
    
    for i, query in enumerate(queries, 1):
        print(f"🔍 Query {i}: {query}")
        try:
            result = client.research(query=query, priority="medium")
            results.append({
                'query': query,
                'total_count': result['data']['total_count'],
                'processing_time': result['data']['processing_time_ms'],
                'top_result': result['data']['results'][0] if result['data']['results'] else None
            })
            print(f"   ✅ {result['data']['total_count']} results in {result['data']['processing_time_ms']}ms")
            
            # Small delay to respect rate limiting
            time.sleep(0.5)
            
        except FortitudeAPIError as e:
            print(f"   ❌ Error: {e}")
            results.append({'query': query, 'error': str(e)})
    
    print("\n📊 Batch Results Summary:")
    for result in results:
        if 'error' not in result:
            print(f"   {result['query'][:40]}... → {result['total_count']} results ({result['processing_time']}ms)")
        else:
            print(f"   {result['query'][:40]}... → Error: {result['error']}")
    
    print()


def cache_demonstration_example(client: FortitudeClient):
    """Example: Demonstrate caching effectiveness."""
    print("🗄️ Cache Effectiveness Demonstration")
    print("=" * 50)
    
    query = "Rust async performance optimization patterns"
    
    try:
        # First request (likely cache miss)
        print("🔍 First request (cache miss expected)...")
        start_time = time.time()
        result1 = client.research(query=query, priority="medium")
        first_duration = time.time() - start_time
        first_processing_time = result1['data']['processing_time_ms']
        
        print(f"   ⏱️  Total time: {first_duration:.3f}s")
        print(f"   ⚙️  Processing time: {first_processing_time}ms")
        
        # Small delay
        time.sleep(1)
        
        # Second request (likely cache hit)
        print("\n🔍 Second request (cache hit expected)...")
        start_time = time.time()
        result2 = client.research(query=query, priority="medium")
        second_duration = time.time() - start_time
        second_processing_time = result2['data']['processing_time_ms']
        
        print(f"   ⏱️  Total time: {second_duration:.3f}s")
        print(f"   ⚙️  Processing time: {second_processing_time}ms")
        
        # Calculate improvement
        if first_processing_time > 0 and second_processing_time > 0:
            improvement = ((first_processing_time - second_processing_time) / first_processing_time) * 100
            print(f"\n📈 Cache Performance:")
            print(f"   💰 Processing time improvement: {improvement:.1f}%")
            print(f"   ⚡ Total time improvement: {((first_duration - second_duration) / first_duration) * 100:.1f}%")
            
            if improvement > 50:
                print("   ✅ Cache effectiveness confirmed!")
            else:
                print("   ⚠️  Cache may already have been warm")
        
    except FortitudeAPIError as e:
        print(f"❌ Error: {e}")
    
    print()


def error_handling_example(client: FortitudeClient):
    """Example: Error handling scenarios."""
    print("⚠️ Error Handling Examples")
    print("=" * 50)
    
    # Test various error scenarios
    error_tests = [
        ("Empty query", lambda: client.research(query="", priority="high")),
        ("Invalid priority", lambda: client.research(query="test", priority="super-urgent")),
        ("Non-existent research ID", lambda: client.get_research_result("00000000-0000-0000-0000-000000000000")),
    ]
    
    for test_name, test_func in error_tests:
        print(f"🧪 Testing: {test_name}")
        try:
            result = test_func()
            print(f"   ⚠️  Unexpected success: {result}")
        except FortitudeAPIError as e:
            print(f"   ✅ Expected error: {e.error_code} - {e.message}")
            if e.request_id:
                print(f"   🔍 Request ID: {e.request_id}")
        except Exception as e:
            print(f"   ❌ Unexpected error: {e}")
        
        print()


def main():
    """Run all research examples."""
    print("🔬 Fortitude API - Research Examples")
    print("=" * 60)
    
    try:
        with setup_client() as client:
            # Test connectivity
            health = client.get_health()
            print(f"🏥 Server status: {health['status']}")
            print(f"📝 Server version: {health['version']}")
            print()
            
            # Run examples
            simple_research_example(client)
            detailed_research_example(client)
            urgent_research_example(client)
            learning_research_example(client)
            list_research_results_example(client)
            batch_research_example(client)
            cache_demonstration_example(client)
            error_handling_example(client)
            
    except Exception as e:
        print(f"❌ Setup error: {e}")
        print("💡 Make sure to set FORTITUDE_API_KEY and FORTITUDE_BASE_URL environment variables")


if __name__ == "__main__":
    main()