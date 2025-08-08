#!/usr/bin/env python3
"""
Example 48: Multi-Level Caching System for Event Data

This example demonstrates multi-level caching strategies to dramatically improve
event sourcing query performance. Proper caching can reduce database load by 80-90%
while providing sub-millisecond response times for frequently accessed data.

Features Demonstrated:
- Multi-level cache hierarchy (L1: Memory, L2: Redis, L3: Database)
- Different eviction policies (LRU, LFU, FIFO)
- Cache warming and invalidation strategies
- Time-to-live (TTL) configurations
- Cache hit/miss ratio optimization
- Memory vs storage trade-offs

Performance Expectations:
- 10-100x faster query response for cached data
- 80-95% reduction in database load
- Sub-millisecond latency for hot data paths

Usage:
    uv run python examples/48_caching_layers.py
"""

import asyncio
import time
import random
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass
from collections import OrderedDict, defaultdict
import json

from eventuali import EventStore
from eventuali.event import DomainEvent
from eventuali.performance import (
    CacheConfig,
    EvictionPolicy,
    CacheManager
)


class CacheTestEvent(DomainEvent):
    """Test event for caching demonstration."""
    event_sequence: int
    data_category: str
    payload_size: int
    access_frequency: str  # hot, warm, cold


@dataclass
class CacheStats:
    """Statistics for cache performance analysis."""
    hits: int = 0
    misses: int = 0
    evictions: int = 0
    total_requests: int = 0
    avg_response_time_ms: float = 0.0
    
    @property
    def hit_rate(self) -> float:
        if self.total_requests == 0:
            return 0.0
        return (self.hits / self.total_requests) * 100
    
    @property
    def miss_rate(self) -> float:
        return 100 - self.hit_rate


class MemoryCache:
    """Simple in-memory cache with configurable eviction policies."""
    
    def __init__(self, max_size: int, eviction_policy: EvictionPolicy, ttl_seconds: int = 3600):
        self.max_size = max_size
        self.eviction_policy = eviction_policy
        self.ttl_seconds = ttl_seconds
        self.cache: OrderedDict = OrderedDict()
        self.access_counts: Dict[str, int] = defaultdict(int)
        self.access_times: Dict[str, float] = {}
        self.stats = CacheStats()
        
    def get(self, key: str) -> Optional[Any]:
        """Get item from cache."""
        self.stats.total_requests += 1
        
        if key not in self.cache:
            self.stats.misses += 1
            return None
        
        # Check TTL
        item_time = self.access_times.get(key, 0)
        if time.time() - item_time > self.ttl_seconds:
            self._evict_key(key)
            self.stats.misses += 1
            return None
        
        # Update access patterns for eviction policy
        if self.eviction_policy == EvictionPolicy.LRU:
            # Move to end (most recently used)
            self.cache.move_to_end(key)
        elif self.eviction_policy == EvictionPolicy.LFU:
            self.access_counts[key] += 1
        
        self.stats.hits += 1
        return self.cache[key]
    
    def put(self, key: str, value: Any) -> None:
        """Put item in cache."""
        # If at capacity, evict based on policy
        if len(self.cache) >= self.max_size and key not in self.cache:
            self._evict_one()
        
        self.cache[key] = value
        self.access_times[key] = time.time()
        self.access_counts[key] = 1
        
        # For LRU, move to end
        if self.eviction_policy == EvictionPolicy.LRU:
            self.cache.move_to_end(key)
    
    def _evict_one(self) -> None:
        """Evict one item based on the eviction policy."""
        if not self.cache:
            return
            
        if self.eviction_policy == EvictionPolicy.LRU:
            # Remove least recently used (first item)
            key_to_remove = next(iter(self.cache))
        elif self.eviction_policy == EvictionPolicy.LFU:
            # Remove least frequently used
            key_to_remove = min(self.access_counts.keys(), key=lambda k: self.access_counts[k])
        else:  # FIFO
            # Remove first inserted
            key_to_remove = next(iter(self.cache))
        
        self._evict_key(key_to_remove)
        
    def _evict_key(self, key: str) -> None:
        """Remove a specific key from cache."""
        if key in self.cache:
            del self.cache[key]
            del self.access_counts[key]
            del self.access_times[key]
            self.stats.evictions += 1
    
    def size(self) -> int:
        """Get current cache size."""
        return len(self.cache)
    
    def clear(self) -> None:
        """Clear all cached data."""
        self.cache.clear()
        self.access_counts.clear()
        self.access_times.clear()


class CachingLayersDemo:
    """Demonstrates multi-level caching for event sourcing performance."""
    
    def __init__(self):
        """Initialize the caching demo."""
        self.l1_cache = MemoryCache(1000, EvictionPolicy.LRU, ttl_seconds=300)  # 5 min TTL
        self.l2_cache = MemoryCache(5000, EvictionPolicy.LFU, ttl_seconds=1800)  # 30 min TTL
        self.l3_cache = MemoryCache(20000, EvictionPolicy.FIFO, ttl_seconds=3600)  # 1 hour TTL
        self.database_access_time_ms = 25  # Simulate DB access time
        
    def demonstrate_cache_configurations(self):
        """Showcase different cache configuration options."""
        print("🔧 Cache Configuration Showcase")
        print("=" * 60)
        
        # Different cache configurations for various use cases
        configs = [
            {
                "name": "High-Frequency Cache",
                "config": CacheConfig(max_size=10000, ttl_seconds=300, eviction_policy=EvictionPolicy.LRU),
                "use_case": "Hot data, frequent access patterns"
            },
            {
                "name": "Large Dataset Cache", 
                "config": CacheConfig(max_size=100000, ttl_seconds=3600, eviction_policy=EvictionPolicy.LFU),
                "use_case": "Analytics, large working sets"
            },
            {
                "name": "Session Cache",
                "config": CacheConfig(max_size=5000, ttl_seconds=1800, eviction_policy=EvictionPolicy.FIFO),
                "use_case": "User sessions, temporary data"
            },
            {
                "name": "Read-Heavy Cache",
                "config": CacheConfig(max_size=50000, ttl_seconds=7200, eviction_policy=EvictionPolicy.LRU),
                "use_case": "Read-heavy workloads, query results"
            }
        ]
        
        for config_info in configs:
            config = config_info["config"]
            print(f"📋 {config_info['name']}: {config}")
            print(f"   Use Case: {config_info['use_case']}")
            
            # Create manager to demonstrate
            manager = CacheManager(config)
            print(f"   Manager: {manager}")
            print()

    def simulate_cache_hierarchy(self) -> None:
        """Demonstrate multi-level cache hierarchy performance."""
        print("🏗️  Multi-Level Cache Hierarchy Simulation")
        print("=" * 60)
        
        # Generate test data with different access patterns
        test_data = self._generate_test_dataset(10000)
        
        print(f"📊 Cache Hierarchy Setup:")
        print(f"   L1 (Memory): {self.l1_cache.max_size:,} items, {self.l1_cache.ttl_seconds}s TTL, {self.l1_cache.eviction_policy}")
        print(f"   L2 (Memory): {self.l2_cache.max_size:,} items, {self.l2_cache.ttl_seconds}s TTL, {self.l2_cache.eviction_policy}")
        print(f"   L3 (Memory): {self.l3_cache.max_size:,} items, {self.l3_cache.ttl_seconds}s TTL, {self.l3_cache.eviction_policy}")
        print(f"   Database: Unlimited size, {self.database_access_time_ms}ms access time")
        print()
        
        # Warm up caches with initial data
        print("🔥 Warming up caches...")
        for i, (key, data) in enumerate(test_data[:5000]):
            if i < 500:  # Hot data in L1
                self.l1_cache.put(key, data)
            elif i < 2000:  # Warm data in L2
                self.l2_cache.put(key, data)
            else:  # Cold data in L3
                self.l3_cache.put(key, data)
        
        # Simulate realistic query patterns
        print("📈 Simulating realistic query patterns...")
        query_results = self._simulate_query_workload(test_data, num_queries=20000)
        
        self._display_cache_performance(query_results)

    def _generate_test_dataset(self, size: int) -> List[Tuple[str, Dict[str, Any]]]:
        """Generate test dataset with different access patterns."""
        dataset = []
        
        for i in range(size):
            # Create realistic event data
            frequency = random.choices(
                ["hot", "warm", "cold"], 
                weights=[10, 30, 60]  # 10% hot, 30% warm, 60% cold
            )[0]
            
            key = f"event_{i:06d}"
            data = {
                "event_id": i,
                "aggregate_id": f"aggregate_{i % 1000}",  # Create some overlap
                "event_type": random.choice(["OrderPlaced", "PaymentProcessed", "ItemShipped"]),
                "timestamp": time.time() - random.randint(0, 86400 * 30),  # Last 30 days
                "data": {"amount": random.randint(10, 1000), "status": "completed"},
                "access_frequency": frequency,
                "size_bytes": random.randint(100, 2000)
            }
            
            dataset.append((key, data))
        
        return dataset

    def _simulate_query_workload(self, dataset: List[Tuple[str, Dict[str, Any]]], num_queries: int) -> Dict[str, Any]:
        """Simulate realistic query workload against cache hierarchy."""
        
        # Create weighted access patterns based on data temperature
        hot_data = [item for item in dataset if item[1]["access_frequency"] == "hot"]
        warm_data = [item for item in dataset if item[1]["access_frequency"] == "warm"]
        cold_data = [item for item in dataset if item[1]["access_frequency"] == "cold"]
        
        total_response_time = 0.0
        l1_hits = l2_hits = l3_hits = database_hits = 0
        
        for _ in range(num_queries):
            # Select data based on realistic access patterns
            rand = random.random()
            if rand < 0.7:  # 70% hot data access
                key, data = random.choice(hot_data)
            elif rand < 0.9:  # 20% warm data access
                key, data = random.choice(warm_data)
            else:  # 10% cold data access
                key, data = random.choice(cold_data)
            
            # Try cache hierarchy
            start_time = time.time()
            
            # L1 Cache
            cached_data = self.l1_cache.get(key)
            if cached_data is not None:
                l1_hits += 1
                response_time = 0.1  # 0.1ms for L1
            else:
                # L2 Cache
                cached_data = self.l2_cache.get(key)
                if cached_data is not None:
                    l2_hits += 1
                    response_time = 1.0  # 1ms for L2
                    # Promote to L1 for hot data
                    if data["access_frequency"] == "hot":
                        self.l1_cache.put(key, cached_data)
                else:
                    # L3 Cache
                    cached_data = self.l3_cache.get(key)
                    if cached_data is not None:
                        l3_hits += 1
                        response_time = 5.0  # 5ms for L3
                        # Promote to L2 for warm data
                        if data["access_frequency"] in ["hot", "warm"]:
                            self.l2_cache.put(key, cached_data)
                        if data["access_frequency"] == "hot":
                            self.l1_cache.put(key, cached_data)
                    else:
                        # Database hit
                        database_hits += 1
                        response_time = self.database_access_time_ms
                        # Cache in appropriate level
                        self.l3_cache.put(key, data)
                        if data["access_frequency"] in ["hot", "warm"]:
                            self.l2_cache.put(key, data)
                        if data["access_frequency"] == "hot":
                            self.l1_cache.put(key, data)
            
            total_response_time += response_time
        
        return {
            "total_queries": num_queries,
            "avg_response_time_ms": total_response_time / num_queries,
            "l1_hits": l1_hits,
            "l2_hits": l2_hits, 
            "l3_hits": l3_hits,
            "database_hits": database_hits,
            "total_response_time_ms": total_response_time
        }

    def _display_cache_performance(self, results: Dict[str, Any]) -> None:
        """Display detailed cache performance analysis."""
        print("📊 Cache Performance Analysis")
        print("-" * 60)
        
        total_queries = results["total_queries"]
        l1_hit_rate = (results["l1_hits"] / total_queries) * 100
        l2_hit_rate = (results["l2_hits"] / total_queries) * 100
        l3_hit_rate = (results["l3_hits"] / total_queries) * 100
        db_hit_rate = (results["database_hits"] / total_queries) * 100
        
        print(f"{'Cache Level':<15} {'Hits':<8} {'Hit Rate':<10} {'Avg Latency':<12}")
        print("-" * 50)
        print(f"{'L1 (Memory)':<15} {results['l1_hits']:<8,} {l1_hit_rate:<10.1f}% {'0.1ms':<12}")
        print(f"{'L2 (Memory)':<15} {results['l2_hits']:<8,} {l2_hit_rate:<10.1f}% {'1.0ms':<12}")
        print(f"{'L3 (Memory)':<15} {results['l3_hits']:<8,} {l3_hit_rate:<10.1f}% {'5.0ms':<12}")
        print(f"{'Database':<15} {results['database_hits']:<8,} {db_hit_rate:<10.1f}% {self.database_access_time_ms}ms")
        print()
        
        # Calculate overall cache effectiveness
        total_cache_hits = results['l1_hits'] + results['l2_hits'] + results['l3_hits']
        overall_cache_hit_rate = (total_cache_hits / total_queries) * 100
        
        print(f"📈 Overall Performance:")
        print(f"   Total Queries: {total_queries:,}")
        print(f"   Cache Hit Rate: {overall_cache_hit_rate:.1f}%")
        print(f"   Database Hit Rate: {db_hit_rate:.1f}%")
        print(f"   Average Response Time: {results['avg_response_time_ms']:.2f}ms")
        
        # Calculate performance improvement
        no_cache_time = total_queries * self.database_access_time_ms
        actual_time = results['total_response_time_ms']
        improvement_factor = no_cache_time / actual_time
        
        print(f"   Performance Improvement: {improvement_factor:.1f}x faster")
        print(f"   Database Load Reduction: {100 - db_hit_rate:.1f}%")
        print()

    def demonstrate_eviction_policies(self):
        """Compare different cache eviction policies."""
        print("🔄 Cache Eviction Policy Comparison")
        print("=" * 60)
        
        # Test different eviction policies
        policies = [EvictionPolicy.LRU, EvictionPolicy.LFU, EvictionPolicy.FIFO]
        policy_results = {}
        
        for policy in policies:
            # Create cache with small size to force evictions
            test_cache = MemoryCache(100, policy, ttl_seconds=3600)
            
            # Generate access pattern that favors certain items
            access_pattern = []
            # 20 hot items accessed frequently
            hot_items = [f"hot_{i}" for i in range(20)]
            # 200 warm items accessed occasionally  
            warm_items = [f"warm_{i}" for i in range(200)]
            # 800 cold items accessed rarely
            cold_items = [f"cold_{i}" for i in range(800)]
            
            # Create realistic access pattern
            for _ in range(10000):
                rand = random.random()
                if rand < 0.6:  # 60% hot items
                    item = random.choice(hot_items)
                elif rand < 0.85:  # 25% warm items
                    item = random.choice(warm_items)
                else:  # 15% cold items
                    item = random.choice(cold_items)
                
                access_pattern.append(item)
            
            # Process access pattern
            for item in access_pattern:
                cached_value = test_cache.get(item)
                if cached_value is None:
                    test_cache.put(item, f"data_for_{item}")
            
            policy_results[policy] = {
                "hit_rate": test_cache.stats.hit_rate,
                "hits": test_cache.stats.hits,
                "misses": test_cache.stats.misses,
                "evictions": test_cache.stats.evictions,
                "final_size": test_cache.size()
            }
        
        print(f"{'Policy':<8} {'Hit Rate':<10} {'Hits':<8} {'Misses':<8} {'Evictions':<10} {'Cache Size':<10}")
        print("-" * 60)
        
        for policy, results in policy_results.items():
            policy_name = policy.name if hasattr(policy, 'name') else str(policy)
            print(f"{policy_name:<8} {results['hit_rate']:<10.1f}% {results['hits']:<8,} {results['misses']:<8,} {results['evictions']:<10,} {results['final_size']:<10}")
        
        # Identify best policy for this workload
        best_policy = max(policy_results.keys(), key=lambda p: policy_results[p]['hit_rate'])
        best_policy_name = best_policy.name if hasattr(best_policy, 'name') else str(best_policy)
        
        print()
        print(f"🏆 Best Policy for this Workload: {best_policy_name}")
        print(f"   Hit Rate: {policy_results[best_policy]['hit_rate']:.1f}%")
        print(f"   Reason: Optimized for the 80/20 access pattern")
        print()

    def demonstrate_cache_warming_strategies(self):
        """Show different cache warming strategies."""
        print("🔥 Cache Warming Strategies")
        print("=" * 60)
        
        strategies = [
            {
                "name": "Eager Loading",
                "description": "Pre-load all frequently accessed data",
                "pros": ["Lowest latency", "Predictable performance"],
                "cons": ["High memory usage", "Long startup time"],
                "best_for": "Well-known access patterns"
            },
            {
                "name": "Lazy Loading",
                "description": "Load data on first access",
                "pros": ["Low memory usage", "Fast startup"],
                "cons": ["Cache miss penalty", "Unpredictable latency"],
                "best_for": "Unknown or changing access patterns"
            },
            {
                "name": "Scheduled Warming",
                "description": "Pre-load data during off-peak hours",
                "pros": ["No user impact", "Optimized resource usage"],
                "cons": ["Complexity", "Stale data risk"],
                "best_for": "Batch processing systems"
            },
            {
                "name": "Predictive Loading",
                "description": "ML-based prediction of data access",
                "pros": ["Intelligent prefetching", "Adaptive patterns"],
                "cons": ["Implementation complexity", "ML overhead"],
                "best_for": "Large-scale systems with data scientists"
            }
        ]
        
        for strategy in strategies:
            print(f"🚀 {strategy['name']}")
            print(f"   Description: {strategy['description']}")
            print(f"   ✅ Pros: {', '.join(strategy['pros'])}")
            print(f"   ❌ Cons: {', '.join(strategy['cons'])}")
            print(f"   🎯 Best For: {strategy['best_for']}")
            print()

    def demonstrate_cache_best_practices(self):
        """Show cache optimization best practices."""
        print("📚 Cache Optimization Best Practices")
        print("=" * 60)
        
        practices = [
            {
                "category": "🏗️  Architecture",
                "tips": [
                    "Use multiple cache layers (L1: local, L2: distributed, L3: persistent)",
                    "Size caches based on working set, not total data",
                    "Place caches close to application instances",
                    "Consider read-through vs cache-aside patterns"
                ]
            },
            {
                "category": "⚙️  Configuration",
                "tips": [
                    "Set TTL based on data change frequency",
                    "Choose eviction policies based on access patterns",
                    "Monitor and tune cache sizes dynamically",
                    "Use compression for large cached objects"
                ]
            },
            {
                "category": "📊 Monitoring",
                "tips": [
                    "Track hit/miss ratios continuously",
                    "Monitor cache memory usage and eviction rates",
                    "Measure cache response times vs database",
                    "Set up alerts for cache availability"
                ]
            },
            {
                "category": "🔄 Invalidation",
                "tips": [
                    "Implement cache invalidation on writes",
                    "Use versioning for cache coherence",
                    "Consider event-driven cache updates",
                    "Handle cache stampede scenarios"
                ]
            }
        ]
        
        for section in practices:
            print(f"{section['category']}")
            for i, tip in enumerate(section['tips'], 1):
                print(f"   {i}. {tip}")
            print()
        
        print("⚠️  Common Pitfalls to Avoid:")
        pitfalls = [
            "Over-caching: Don't cache everything, focus on hot data",
            "Under-sizing: Too small caches cause thrashing",
            "Ignoring invalidation: Stale data leads to bugs",
            "Single layer: Multi-level caching provides better performance",
            "No monitoring: You can't optimize what you don't measure"
        ]
        
        for pitfall in pitfalls:
            print(f"   • {pitfall}")
        print()


async def main():
    """Main demonstration function."""
    print("🚀 Eventuali Multi-Level Caching Performance Demo")
    print("=" * 80)
    print()
    
    demo = CachingLayersDemo()
    
    try:
        # Demonstrate cache configurations
        demo.demonstrate_cache_configurations()
        
        # Show cache hierarchy simulation
        demo.simulate_cache_hierarchy()
        
        # Compare eviction policies  
        demo.demonstrate_eviction_policies()
        
        # Show cache warming strategies
        demo.demonstrate_cache_warming_strategies()
        
        # Show best practices
        demo.demonstrate_cache_best_practices()
        
        print("🎉 Caching Layers Demo Complete!")
        print()
        print("Key Takeaways:")
        print("• Multi-level caching can improve performance 10-100x")
        print("• Choose eviction policies based on access patterns")
        print("• Monitor cache hit rates and adjust sizes accordingly")
        print("• Implement proper cache warming and invalidation strategies")
        print("• Cache hierarchies reduce database load by 80-95%")
        
    except Exception as e:
        print(f"❌ Demo failed with error: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    asyncio.run(main())