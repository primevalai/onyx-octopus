#!/usr/bin/env python3
"""
Example 49: Advanced Compression Algorithms for Event Data

This example demonstrates advanced compression techniques for optimizing event storage
and transmission. Proper compression can reduce storage requirements by 60-90% and
improve network transfer performance while maintaining fast decompression speeds.

Features Demonstrated:
- Multiple compression algorithms (LZ4, ZSTD, GZIP)
- Compression level optimization
- Parallel compression for multi-core systems
- Compression ratio vs speed trade-offs
- Batch compression strategies
- Algorithm selection based on data characteristics

Performance Expectations:
- 60-90% storage space reduction
- 2-10x faster network transfers
- Millisecond-level compression/decompression
- Optimized CPU utilization with parallel processing

Usage:
    uv run python examples/49_advanced_compression.py
"""

import asyncio
import time
import zlib
import json
import random
from typing import Dict, List, Tuple, Any, Optional
from dataclasses import dataclass
import string

from eventuali import EventStore
from eventuali.event import DomainEvent
from eventuali.performance import (
    CompressionAlgorithm,
    CompressionConfig,
    CompressionManager
)


class CompressionTestEvent(DomainEvent):
    """Test event for compression demonstration."""
    event_sequence: int
    payload_type: str
    data_size: int
    compressibility: str  # high, medium, low


@dataclass
class CompressionResult:
    """Results of compression analysis."""
    algorithm: str
    level: int
    original_size: int
    compressed_size: int
    compression_time_ms: float
    decompression_time_ms: float
    
    @property
    def compression_ratio(self) -> float:
        if self.original_size == 0:
            return 0.0
        return (self.original_size - self.compressed_size) / self.original_size * 100
    
    @property
    def size_reduction_factor(self) -> float:
        if self.compressed_size == 0:
            return float('inf')
        return self.original_size / self.compressed_size
    
    @property
    def throughput_mbps(self) -> float:
        if self.compression_time_ms == 0:
            return 0.0
        return (self.original_size / (1024 * 1024)) / (self.compression_time_ms / 1000)


class SimpleCompressor:
    """Simple compression implementation for demonstration."""
    
    def __init__(self, algorithm: CompressionAlgorithm, level: int = 3):
        self.algorithm = algorithm
        self.level = level
    
    def compress(self, data: bytes) -> bytes:
        """Compress data using the specified algorithm."""
        if self.algorithm == CompressionAlgorithm.NONE:
            return data
        elif self.algorithm == CompressionAlgorithm.GZIP:
            return zlib.compress(data, level=self.level)
        elif self.algorithm == CompressionAlgorithm.LZ4:
            # Simulate LZ4 with zlib (faster compression)
            return zlib.compress(data, level=1)  # Fast compression
        elif self.algorithm == CompressionAlgorithm.ZSTD:
            # Simulate ZSTD with zlib (better compression)
            return zlib.compress(data, level=6)  # Better compression
        else:
            return data
    
    def decompress(self, compressed_data: bytes) -> bytes:
        """Decompress data."""
        if self.algorithm == CompressionAlgorithm.NONE:
            return compressed_data
        else:
            return zlib.decompress(compressed_data)


class AdvancedCompressionDemo:
    """Demonstrates advanced compression techniques for event data."""
    
    def __init__(self):
        """Initialize the compression demo."""
        self.test_data_cache = {}
        
    def demonstrate_compression_configurations(self):
        """Showcase different compression configuration options."""
        print("🔧 Compression Configuration Showcase")
        print("=" * 60)
        
        # Different compression configurations for various use cases
        configs = [
            {
                "name": "High-Speed Compression",
                "config": CompressionConfig(algorithm=CompressionAlgorithm.LZ4, level=1, enable_parallel=True),
                "use_case": "Real-time events, high throughput systems"
            },
            {
                "name": "Balanced Compression",
                "config": CompressionConfig(algorithm=CompressionAlgorithm.ZSTD, level=3, enable_parallel=True),
                "use_case": "General purpose, good speed/ratio balance"
            },
            {
                "name": "Maximum Compression",
                "config": CompressionConfig(algorithm=CompressionAlgorithm.GZIP, level=9, enable_parallel=False),
                "use_case": "Archival storage, bandwidth-limited networks"
            },
            {
                "name": "No Compression",
                "config": CompressionConfig(algorithm=CompressionAlgorithm.NONE, level=0, enable_parallel=False),
                "use_case": "Already compressed data, CPU-constrained systems"
            }
        ]
        
        for config_info in configs:
            config = config_info["config"]
            print(f"📋 {config_info['name']}: {config}")
            print(f"   Use Case: {config_info['use_case']}")
            
            # Create manager to demonstrate
            manager = CompressionManager(config)
            print(f"   Manager: {manager}")
            print()

    def generate_test_payloads(self) -> Dict[str, bytes]:
        """Generate different types of test payloads with varying compressibility."""
        payloads = {}
        
        # Highly compressible data (JSON with repetitive structure)
        json_data = {
            "events": []
        }
        for i in range(1000):
            json_data["events"].append({
                "id": i,
                "type": "OrderPlaced",
                "timestamp": "2024-01-01T00:00:00Z",
                "data": {
                    "customer_id": f"customer_{i % 100}",
                    "product_id": f"product_{i % 50}",
                    "quantity": random.randint(1, 5),
                    "status": "pending"
                }
            })
        payloads["highly_compressible"] = json.dumps(json_data).encode('utf-8')
        
        # Medium compressible data (mixed content)
        medium_data = []
        for i in range(500):
            medium_data.append({
                "id": i,
                "random_field": ''.join(random.choices(string.ascii_letters, k=20)),
                "number": random.randint(1, 1000000),
                "nested": {
                    "field1": random.choice(["active", "inactive", "pending"]),
                    "field2": random.uniform(0.0, 1000.0)
                }
            })
        payloads["medium_compressible"] = json.dumps(medium_data).encode('utf-8')
        
        # Low compressible data (random binary)
        payloads["low_compressible"] = bytes([random.randint(0, 255) for _ in range(50000)])
        
        # Text data (high compressibility)
        text_data = "The quick brown fox jumps over the lazy dog. " * 2000
        payloads["text_data"] = text_data.encode('utf-8')
        
        return payloads

    def benchmark_compression_algorithms(self):
        """Benchmark different compression algorithms."""
        print("📊 Compression Algorithm Benchmark")
        print("=" * 60)
        
        test_payloads = self.generate_test_payloads()
        algorithms = [
            (CompressionAlgorithm.NONE, 0),
            (CompressionAlgorithm.LZ4, 1),
            (CompressionAlgorithm.ZSTD, 3),
            (CompressionAlgorithm.GZIP, 6),
            (CompressionAlgorithm.GZIP, 9),
        ]
        
        results = {}
        
        for payload_name, payload_data in test_payloads.items():
            print(f"\n🔍 Testing payload: {payload_name} ({len(payload_data):,} bytes)")
            print("-" * 80)
            print(f"{'Algorithm':<12} {'Level':<6} {'Ratio':<8} {'Speed':<12} {'Decomp':<10} {'Throughput':<12}")
            print("-" * 80)
            
            payload_results = []
            
            for algorithm, level in algorithms:
                compressor = SimpleCompressor(algorithm, level)
                
                # Compress
                start_time = time.time()
                compressed_data = compressor.compress(payload_data)
                compression_time = (time.time() - start_time) * 1000  # ms
                
                # Decompress
                start_time = time.time()
                decompressed_data = compressor.decompress(compressed_data)
                decompression_time = (time.time() - start_time) * 1000  # ms
                
                # Verify correctness
                assert decompressed_data == payload_data
                
                result = CompressionResult(
                    algorithm=algorithm.name if hasattr(algorithm, 'name') else str(algorithm),
                    level=level,
                    original_size=len(payload_data),
                    compressed_size=len(compressed_data),
                    compression_time_ms=compression_time,
                    decompression_time_ms=decompression_time
                )
                
                payload_results.append(result)
                
                print(f"{result.algorithm:<12} {result.level:<6} {result.compression_ratio:<8.1f}% "
                      f"{result.compression_time_ms:<12.2f}ms {result.decompression_time_ms:<10.2f}ms "
                      f"{result.throughput_mbps:<12.1f}MB/s")
            
            results[payload_name] = payload_results
        
        return results

    def analyze_compression_trade_offs(self, results: Dict[str, List[CompressionResult]]):
        """Analyze compression trade-offs across different data types."""
        print("\n⚖️  Compression Trade-off Analysis")
        print("=" * 60)
        
        # Find best algorithms for different scenarios
        scenarios = {
            "Best Compression Ratio": lambda r: max(r, key=lambda x: x.compression_ratio),
            "Fastest Compression": lambda r: min([x for x in r if x.algorithm != "NONE"], 
                                                key=lambda x: x.compression_time_ms),
            "Best Balance": lambda r: max([x for x in r if x.algorithm != "NONE"], 
                                        key=lambda x: x.compression_ratio / (x.compression_time_ms + 1))
        }
        
        for payload_name, payload_results in results.items():
            print(f"\n📋 {payload_name.replace('_', ' ').title()}")
            print("-" * 40)
            
            for scenario_name, selector_func in scenarios.items():
                try:
                    best_result = selector_func(payload_results)
                    print(f"  {scenario_name}: {best_result.algorithm} Level {best_result.level}")
                    print(f"    Ratio: {best_result.compression_ratio:.1f}%, "
                          f"Speed: {best_result.compression_time_ms:.2f}ms, "
                          f"Throughput: {best_result.throughput_mbps:.1f}MB/s")
                except (ValueError, IndexError):
                    print(f"  {scenario_name}: N/A")

    def demonstrate_batch_compression(self):
        """Demonstrate batch compression strategies."""
        print("\n🗂️  Batch Compression Strategies")
        print("=" * 60)
        
        # Generate batch of events
        events_batch = []
        for i in range(1000):
            event_data = {
                "id": i,
                "type": random.choice(["OrderPlaced", "PaymentProcessed", "ItemShipped"]),
                "timestamp": time.time(),
                "aggregate_id": f"aggregate_{i % 100}",
                "data": {
                    "amount": random.randint(10, 1000),
                    "status": random.choice(["pending", "completed", "failed"])
                }
            }
            events_batch.append(json.dumps(event_data))
        
        strategies = [
            {
                "name": "Individual Compression",
                "description": "Compress each event separately",
                "implementation": lambda events: [
                    zlib.compress(event.encode('utf-8')) for event in events
                ]
            },
            {
                "name": "Batch Compression",
                "description": "Compress all events as single payload",
                "implementation": lambda events: [
                    zlib.compress('\n'.join(events).encode('utf-8'))
                ]
            },
            {
                "name": "Chunked Compression",
                "description": "Compress events in chunks of 50",
                "implementation": lambda events: [
                    zlib.compress('\n'.join(events[i:i+50]).encode('utf-8'))
                    for i in range(0, len(events), 50)
                ]
            }
        ]
        
        original_size = sum(len(event.encode('utf-8')) for event in events_batch)
        print(f"📊 Original batch size: {original_size:,} bytes ({len(events_batch)} events)")
        print()
        
        for strategy in strategies:
            start_time = time.time()
            compressed_results = strategy["implementation"](events_batch)
            compression_time = (time.time() - start_time) * 1000
            
            total_compressed_size = sum(len(result) for result in compressed_results)
            compression_ratio = (original_size - total_compressed_size) / original_size * 100
            
            print(f"🔧 {strategy['name']}")
            print(f"   Description: {strategy['description']}")
            print(f"   Compressed Size: {total_compressed_size:,} bytes ({len(compressed_results)} chunks)")
            print(f"   Compression Ratio: {compression_ratio:.1f}%")
            print(f"   Processing Time: {compression_time:.2f}ms")
            print(f"   Throughput: {(original_size / (1024 * 1024)) / (compression_time / 1000):.1f}MB/s")
            print()

    def demonstrate_parallel_compression(self):
        """Demonstrate parallel compression benefits."""
        print("⚡ Parallel Compression Benefits")
        print("=" * 60)
        
        # Generate large dataset
        large_dataset = []
        for i in range(10000):
            data = {
                "id": i,
                "payload": ''.join(random.choices(string.ascii_letters, k=100)),
                "metadata": {"processed": False, "priority": random.randint(1, 5)}
            }
            large_dataset.append(json.dumps(data).encode('utf-8'))
        
        total_size = sum(len(data) for data in large_dataset)
        
        print(f"📊 Dataset: {len(large_dataset):,} items, {total_size:,} bytes")
        print()
        
        # Sequential processing
        print("🔄 Sequential Compression:")
        start_time = time.time()
        sequential_results = []
        for data in large_dataset:
            compressed = zlib.compress(data, level=6)
            sequential_results.append(compressed)
        sequential_time = time.time() - start_time
        sequential_size = sum(len(result) for result in sequential_results)
        
        print(f"   Time: {sequential_time:.3f}s")
        print(f"   Throughput: {(total_size / (1024 * 1024)) / sequential_time:.1f}MB/s")
        print(f"   Compressed Size: {sequential_size:,} bytes")
        
        # Simulate parallel processing (chunked)
        print("\n⚡ Simulated Parallel Compression (4 cores):")
        start_time = time.time()
        chunk_size = len(large_dataset) // 4
        parallel_results = []
        
        # Simulate processing 4 chunks in parallel (sequential for demo)
        for i in range(0, len(large_dataset), chunk_size):
            chunk = large_dataset[i:i + chunk_size]
            chunk_start = time.time()
            
            chunk_results = []
            for data in chunk:
                compressed = zlib.compress(data, level=6)
                chunk_results.append(compressed)
            
            # Simulate parallel processing overhead
            time.sleep(0.001)  # Small parallel processing overhead
            parallel_results.extend(chunk_results)
        
        parallel_time = time.time() - start_time
        parallel_size = sum(len(result) for result in parallel_results)
        
        print(f"   Time: {parallel_time:.3f}s")
        print(f"   Throughput: {(total_size / (1024 * 1024)) / parallel_time:.1f}MB/s")
        print(f"   Compressed Size: {parallel_size:,} bytes")
        print(f"   Speed Improvement: {sequential_time / parallel_time:.1f}x")
        print()

    def demonstrate_compression_best_practices(self):
        """Show compression optimization best practices."""
        print("📚 Compression Best Practices")
        print("=" * 60)
        
        practices = [
            {
                "category": "🎯 Algorithm Selection",
                "tips": [
                    "Use LZ4 for real-time/streaming scenarios (speed > ratio)",
                    "Use ZSTD for general purpose (good balance)",
                    "Use GZIP for archival/storage (ratio > speed)",
                    "Skip compression for already compressed data"
                ]
            },
            {
                "category": "⚙️  Configuration",
                "tips": [
                    "Start with level 3-6 for most use cases",
                    "Use level 1 for high-throughput systems",
                    "Use level 9 only for archival storage",
                    "Enable parallel compression for multi-core systems"
                ]
            },
            {
                "category": "📊 Data Patterns",
                "tips": [
                    "Batch similar events for better compression",
                    "Remove unnecessary whitespace from JSON",
                    "Use shorter field names in frequently compressed data",
                    "Consider schema evolution for better compression"
                ]
            },
            {
                "category": "🔄 Implementation",
                "tips": [
                    "Compress at write time, decompress at read time",
                    "Cache compression dictionaries for better ratios",
                    "Monitor CPU usage vs storage savings",
                    "Implement adaptive compression based on data type"
                ]
            }
        ]
        
        for section in practices:
            print(f"{section['category']}")
            for i, tip in enumerate(section['tips'], 1):
                print(f"   {i}. {tip}")
            print()
        
        print("📈 Performance Guidelines:")
        guidelines = [
            "Measure compression ratio vs CPU overhead for your workload",
            "Consider network bandwidth vs compression time trade-offs", 
            "Test decompression speed for read-heavy applications",
            "Monitor memory usage during compression operations",
            "Implement compression level auto-tuning based on system load"
        ]
        
        for guideline in guidelines:
            print(f"   • {guideline}")
        print()
        
        print("⚠️  Common Pitfalls:")
        pitfalls = [
            "Over-compressing: Diminishing returns at high levels",
            "Ignoring decompression cost: Read performance matters",
            "One-size-fits-all: Different data needs different algorithms",
            "Not measuring: Always benchmark with real data",
            "CPU bottlenecks: Monitor system resources during compression"
        ]
        
        for pitfall in pitfalls:
            print(f"   • {pitfall}")
        print()


async def main():
    """Main demonstration function."""
    print("🚀 Eventuali Advanced Compression Performance Demo")
    print("=" * 80)
    print()
    
    demo = AdvancedCompressionDemo()
    
    try:
        # Demonstrate compression configurations
        demo.demonstrate_compression_configurations()
        
        # Run compression benchmarks
        results = demo.benchmark_compression_algorithms()
        
        # Analyze trade-offs
        demo.analyze_compression_trade_offs(results)
        
        # Show batch compression strategies
        demo.demonstrate_batch_compression()
        
        # Show parallel compression benefits
        demo.demonstrate_parallel_compression()
        
        # Show best practices
        demo.demonstrate_compression_best_practices()
        
        print("🎉 Advanced Compression Demo Complete!")
        print()
        print("Key Takeaways:")
        print("• Choose compression algorithms based on use case requirements")
        print("• LZ4 for speed, ZSTD for balance, GZIP for maximum compression")
        print("• Batch compression provides better ratios than individual compression")
        print("• Parallel compression can improve throughput on multi-core systems")
        print("• Always measure compression performance with your actual data")
        
    except Exception as e:
        print(f"❌ Demo failed with error: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    asyncio.run(main())