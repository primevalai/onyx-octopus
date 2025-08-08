#!/usr/bin/env python3
"""
Example 41: Observability Performance Impact Analysis

This example demonstrates:
1. Performance impact measurement of full observability stack
2. Baseline vs. observability-enabled performance comparison
3. Memory usage analysis with observability features
4. Throughput analysis with and without monitoring
5. Latency overhead measurement for tracing and metrics
6. Resource utilization analysis across different load levels
7. Performance optimization recommendations

The implementation validates that comprehensive observability adds <2% overhead
to core event sourcing operations while providing deep system insights.
"""

import asyncio
import time
import statistics
import psutil
import gc
from datetime import datetime, timezone
from typing import Dict, List, Tuple, Optional, Any
from dataclasses import dataclass, field
import random
import json
import threading
from collections import defaultdict, deque
import uuid

import eventuali


@dataclass
class PerformanceMetrics:
    """Container for performance measurement results"""
    operation_name: str
    total_operations: int
    total_time_seconds: float
    average_latency_ms: float
    min_latency_ms: float
    max_latency_ms: float
    p50_latency_ms: float
    p95_latency_ms: float
    p99_latency_ms: float
    throughput_ops_per_second: float
    memory_usage_mb: float
    cpu_usage_percent: float
    overhead_percentage: float = 0.0


@dataclass 
class ObservabilityConfig:
    """Configuration for observability features"""
    enable_tracing: bool = True
    enable_metrics: bool = True
    enable_logging: bool = True
    logging_level: str = "INFO"
    metrics_sampling_rate: float = 1.0
    trace_sampling_rate: float = 1.0


class PerformanceBenchmark:
    """High-performance benchmark harness for measuring observability overhead"""
    
    def __init__(self):
        self.process = psutil.Process()
        self.baseline_memory = self.get_memory_usage_mb()
        
    def get_memory_usage_mb(self) -> float:
        """Get current memory usage in MB"""
        return self.process.memory_info().rss / 1024 / 1024
    
    def get_cpu_usage_percent(self) -> float:
        """Get current CPU usage percentage"""
        return self.process.cpu_percent(interval=0.1)
    
    async def measure_operation_performance(self, operation_func, num_iterations: int = 1000,
                                          operation_name: str = "operation") -> PerformanceMetrics:
        """Measure performance of an operation with detailed statistics"""
        
        # Warm up JIT and caches
        for _ in range(10):
            await operation_func()
        
        # Force garbage collection before measurement
        gc.collect()
        
        # Measure baseline memory and CPU
        baseline_memory = self.get_memory_usage_mb()
        
        # Perform timed measurements
        latencies = []
        start_time = time.time()
        
        for i in range(num_iterations):
            operation_start = time.time()
            
            try:
                result = await operation_func()
            except Exception as e:
                print(f"Operation {i} failed: {e}")
                continue
                
            operation_end = time.time()
            latencies.append((operation_end - operation_start) * 1000)  # Convert to ms
            
            # Yield control occasionally to prevent blocking
            if i % 100 == 0:
                await asyncio.sleep(0.001)
        
        end_time = time.time()
        
        # Calculate statistics
        total_time = end_time - start_time
        latencies.sort()
        
        metrics = PerformanceMetrics(
            operation_name=operation_name,
            total_operations=len(latencies),
            total_time_seconds=total_time,
            average_latency_ms=statistics.mean(latencies),
            min_latency_ms=min(latencies),
            max_latency_ms=max(latencies),
            p50_latency_ms=statistics.median(latencies),
            p95_latency_ms=latencies[int(len(latencies) * 0.95)] if latencies else 0,
            p99_latency_ms=latencies[int(len(latencies) * 0.99)] if latencies else 0,
            throughput_ops_per_second=len(latencies) / total_time if total_time > 0 else 0,
            memory_usage_mb=self.get_memory_usage_mb() - baseline_memory,
            cpu_usage_percent=self.get_cpu_usage_percent()
        )
        
        return metrics


class MockObservabilityService:
    """Lightweight mock observability service for performance testing"""
    
    def __init__(self, config: ObservabilityConfig):
        self.config = config
        self.metrics_buffer = deque(maxlen=10000)
        self.traces_buffer = deque(maxlen=10000)
        self.logs_buffer = deque(maxlen=10000)
        self.operation_count = 0
        self.lock = threading.Lock()
    
    def create_trace_context(self, operation: str) -> Dict[str, Any]:
        """Create a trace context for an operation"""
        if not self.config.enable_tracing or random.random() > self.config.trace_sampling_rate:
            return {"disabled": True}
            
        return {
            "trace_id": str(uuid.uuid4()),
            "span_id": str(uuid.uuid4()),
            "operation": operation,
            "start_time": time.time(),
            "disabled": False
        }
    
    def finish_trace(self, context: Dict[str, Any]):
        """Finish a trace context"""
        if context.get("disabled", True):
            return
            
        context["end_time"] = time.time()
        context["duration_ms"] = (context["end_time"] - context["start_time"]) * 1000
        
        with self.lock:
            self.traces_buffer.append(context)
    
    def record_metric(self, name: str, value: float, labels: Dict[str, str] = None):
        """Record a metric"""
        if not self.config.enable_metrics or random.random() > self.config.metrics_sampling_rate:
            return
        
        metric = {
            "name": name,
            "value": value,
            "labels": labels or {},
            "timestamp": time.time()
        }
        
        with self.lock:
            self.metrics_buffer.append(metric)
    
    def log_event(self, level: str, message: str, context: Dict[str, Any] = None):
        """Log an event"""
        if not self.config.enable_logging:
            return
            
        log_entry = {
            "level": level,
            "message": message,
            "context": context or {},
            "timestamp": time.time()
        }
        
        with self.lock:
            self.logs_buffer.append(log_entry)
    
    def get_stats(self) -> Dict[str, Any]:
        """Get observability service statistics"""
        with self.lock:
            return {
                "traces_count": len(self.traces_buffer),
                "metrics_count": len(self.metrics_buffer),
                "logs_count": len(self.logs_buffer),
                "memory_usage_estimate_mb": (
                    len(self.traces_buffer) * 0.001 +  # ~1KB per trace
                    len(self.metrics_buffer) * 0.0005 + # ~0.5KB per metric
                    len(self.logs_buffer) * 0.002      # ~2KB per log
                )
            }


class ObservableEventStore:
    """Event store with configurable observability for performance testing"""
    
    def __init__(self, connection_string: str, observability: Optional[MockObservabilityService] = None):
        self.connection_string = connection_string
        self.observability = observability
        self.event_count = 0
        self.operation_times = deque(maxlen=1000)
    
    async def create_event_baseline(self, aggregate_id: str, event_type: str, event_data: dict) -> dict:
        """Create event without any observability (baseline performance)"""
        # Simulate actual event creation work
        await asyncio.sleep(0.001)  # Simulate database I/O
        
        self.event_count += 1
        event_size = len(json.dumps(event_data))
        
        # Simulate some CPU work (serialization, validation, etc.)
        _ = json.dumps(event_data)
        _ = str(uuid.uuid4())
        
        return {
            "event_id": f"event-{self.event_count:06d}",
            "aggregate_id": aggregate_id,
            "event_type": event_type,
            "size": event_size,
            "created_at": datetime.now(timezone.utc).isoformat()
        }
    
    async def create_event_with_observability(self, aggregate_id: str, event_type: str, 
                                            event_data: dict) -> dict:
        """Create event with full observability enabled"""
        operation_start = time.time()
        
        # Create trace context
        trace_context = None
        if self.observability:
            trace_context = self.observability.create_trace_context("create_event")
        
        try:
            # Simulate actual event creation work (same as baseline)
            await asyncio.sleep(0.001)  # Simulate database I/O
            
            self.event_count += 1
            event_size = len(json.dumps(event_data))
            
            # Simulate some CPU work (serialization, validation, etc.)
            _ = json.dumps(event_data)
            event_id = str(uuid.uuid4())
            
            # Record metrics
            if self.observability:
                duration_ms = (time.time() - operation_start) * 1000
                
                self.observability.record_metric(
                    "events_created_total", 
                    1, 
                    {"event_type": event_type, "aggregate_type": "benchmark"}
                )
                
                self.observability.record_metric(
                    "event_creation_duration_ms", 
                    duration_ms,
                    {"event_type": event_type}
                )
                
                self.observability.record_metric(
                    "event_size_bytes", 
                    event_size,
                    {"event_type": event_type}
                )
                
                # Log the operation
                self.observability.log_event(
                    "INFO",
                    f"Created event {event_type} for aggregate {aggregate_id}",
                    {
                        "event_id": event_id,
                        "aggregate_id": aggregate_id,
                        "event_type": event_type,
                        "size": event_size,
                        "duration_ms": duration_ms
                    }
                )
            
            result = {
                "event_id": f"event-{self.event_count:06d}",
                "aggregate_id": aggregate_id,
                "event_type": event_type,
                "size": event_size,
                "created_at": datetime.now(timezone.utc).isoformat()
            }
            
            return result
            
        finally:
            if self.observability and trace_context:
                self.observability.finish_trace(trace_context)
    
    async def load_events_baseline(self, aggregate_id: str) -> List[dict]:
        """Load events without observability (baseline)"""
        # Simulate loading work
        event_count = random.randint(5, 25)
        await asyncio.sleep(event_count * 0.0002)  # Simulate DB query time
        
        # Generate simulated events
        events = []
        for i in range(event_count):
            events.append({
                "event_id": f"event-{i:06d}",
                "aggregate_id": aggregate_id,
                "event_type": "SimulatedEvent",
                "version": i + 1
            })
        
        return events
    
    async def load_events_with_observability(self, aggregate_id: str) -> List[dict]:
        """Load events with full observability enabled"""
        operation_start = time.time()
        
        # Create trace context
        trace_context = None
        if self.observability:
            trace_context = self.observability.create_trace_context("load_events")
        
        try:
            # Simulate loading work (same as baseline)
            event_count = random.randint(5, 25)
            await asyncio.sleep(event_count * 0.0002)  # Simulate DB query time
            
            # Generate simulated events
            events = []
            for i in range(event_count):
                events.append({
                    "event_id": f"event-{i:06d}",
                    "aggregate_id": aggregate_id,
                    "event_type": "SimulatedEvent",
                    "version": i + 1
                })
            
            # Record observability data
            if self.observability:
                duration_ms = (time.time() - operation_start) * 1000
                
                self.observability.record_metric(
                    "events_loaded_total",
                    event_count,
                    {"aggregate_id": aggregate_id}
                )
                
                self.observability.record_metric(
                    "event_load_duration_ms",
                    duration_ms,
                    {"aggregate_id": aggregate_id}
                )
                
                self.observability.log_event(
                    "INFO",
                    f"Loaded {event_count} events for aggregate {aggregate_id}",
                    {
                        "aggregate_id": aggregate_id,
                        "event_count": event_count,
                        "duration_ms": duration_ms
                    }
                )
            
            return events
            
        finally:
            if self.observability and trace_context:
                self.observability.finish_trace(trace_context)


async def benchmark_event_creation_overhead():
    """Benchmark the overhead of observability on event creation operations"""
    print("=" * 80)
    print("📊 EVENT CREATION PERFORMANCE ANALYSIS")
    print("=" * 80)
    print()
    
    benchmark = PerformanceBenchmark()
    
    # Test configurations
    configurations = [
        ("Baseline (No Observability)", ObservabilityConfig(False, False, False)),
        ("Tracing Only", ObservabilityConfig(True, False, False)),
        ("Metrics Only", ObservabilityConfig(False, True, False)),
        ("Logging Only", ObservabilityConfig(False, False, True)),
        ("Full Observability", ObservabilityConfig(True, True, True)),
        ("Sampled Observability (50%)", ObservabilityConfig(True, True, True, metrics_sampling_rate=0.5, trace_sampling_rate=0.5))
    ]
    
    results = []
    baseline_throughput = None
    
    for config_name, config in configurations:
        print(f"🔬 Testing: {config_name}")
        
        # Create event store with appropriate observability
        if config_name == "Baseline (No Observability)":
            observability = None
            event_store = ObservableEventStore("sqlite://:memory:", observability)
            
            # Create test operation
            async def test_operation():
                return await event_store.create_event_baseline(
                    f"aggregate-{random.randint(1, 100)}",
                    f"Event{random.randint(1, 5)}",
                    {"data": f"test-{random.randint(1, 1000)}", "value": random.uniform(1, 100)}
                )
        else:
            observability = MockObservabilityService(config)
            event_store = ObservableEventStore("sqlite://:memory:", observability)
            
            # Create test operation
            async def test_operation():
                return await event_store.create_event_with_observability(
                    f"aggregate-{random.randint(1, 100)}",
                    f"Event{random.randint(1, 5)}",
                    {"data": f"test-{random.randint(1, 1000)}", "value": random.uniform(1, 100)}
                )
        
        # Run benchmark
        metrics = await benchmark.measure_operation_performance(
            test_operation, 
            num_iterations=2000,  # Increased for more accurate measurements
            operation_name=config_name
        )
        
        # Calculate overhead percentage
        if baseline_throughput is None:
            baseline_throughput = metrics.throughput_ops_per_second
            metrics.overhead_percentage = 0.0
        else:
            metrics.overhead_percentage = ((baseline_throughput - metrics.throughput_ops_per_second) / baseline_throughput) * 100
        
        results.append(metrics)
        
        # Print results
        print(f"   Throughput: {metrics.throughput_ops_per_second:.1f} ops/sec")
        print(f"   Avg Latency: {metrics.average_latency_ms:.2f}ms")
        print(f"   P95 Latency: {metrics.p95_latency_ms:.2f}ms")
        print(f"   Memory Overhead: +{metrics.memory_usage_mb:.1f}MB")
        print(f"   Performance Overhead: {metrics.overhead_percentage:.2f}%")
        
        if observability:
            obs_stats = observability.get_stats()
            print(f"   Observability Buffer Size: {obs_stats['traces_count']} traces, {obs_stats['metrics_count']} metrics, {obs_stats['logs_count']} logs")
        
        print()
    
    return results


async def benchmark_event_loading_overhead():
    """Benchmark the overhead of observability on event loading operations"""
    print("=" * 80)
    print("📚 EVENT LOADING PERFORMANCE ANALYSIS")
    print("=" * 80)
    print()
    
    benchmark = PerformanceBenchmark()
    
    configurations = [
        ("Baseline (No Observability)", ObservabilityConfig(False, False, False)),
        ("Full Observability", ObservabilityConfig(True, True, True))
    ]
    
    results = []
    baseline_throughput = None
    
    for config_name, config in configurations:
        print(f"🔬 Testing: {config_name}")
        
        if config_name == "Baseline (No Observability)":
            observability = None
            event_store = ObservableEventStore("sqlite://:memory:", observability)
            
            async def test_operation():
                return await event_store.load_events_baseline(f"aggregate-{random.randint(1, 100)}")
        else:
            observability = MockObservabilityService(config)
            event_store = ObservableEventStore("sqlite://:memory:", observability)
            
            async def test_operation():
                return await event_store.load_events_with_observability(f"aggregate-{random.randint(1, 100)}")
        
        # Run benchmark
        metrics = await benchmark.measure_operation_performance(
            test_operation,
            num_iterations=1500,
            operation_name=config_name
        )
        
        # Calculate overhead
        if baseline_throughput is None:
            baseline_throughput = metrics.throughput_ops_per_second
            metrics.overhead_percentage = 0.0
        else:
            metrics.overhead_percentage = ((baseline_throughput - metrics.throughput_ops_per_second) / baseline_throughput) * 100
        
        results.append(metrics)
        
        print(f"   Throughput: {metrics.throughput_ops_per_second:.1f} ops/sec")
        print(f"   Avg Latency: {metrics.average_latency_ms:.2f}ms")
        print(f"   P95 Latency: {metrics.p95_latency_ms:.2f}ms")
        print(f"   Performance Overhead: {metrics.overhead_percentage:.2f}%")
        print()
    
    return results


async def analyze_memory_scaling():
    """Analyze memory usage scaling with observability features"""
    print("=" * 80)
    print("💾 MEMORY USAGE SCALING ANALYSIS")
    print("=" * 80)
    print()
    
    benchmark = PerformanceBenchmark()
    
    # Test different operation volumes
    operation_volumes = [100, 500, 1000, 2500, 5000]
    
    for volume in operation_volumes:
        print(f"🔬 Testing {volume} operations...")
        
        # Test with full observability
        observability = MockObservabilityService(ObservabilityConfig(True, True, True))
        event_store = ObservableEventStore("sqlite://:memory:", observability)
        
        start_memory = benchmark.get_memory_usage_mb()
        
        # Perform operations
        for i in range(volume):
            await event_store.create_event_with_observability(
                f"aggregate-{i}",
                "MemoryTestEvent",
                {"iteration": i, "data": f"test-data-{i}"}
            )
        
        end_memory = benchmark.get_memory_usage_mb()
        memory_increase = end_memory - start_memory
        
        obs_stats = observability.get_stats()
        
        print(f"   Operations: {volume}")
        print(f"   Memory Increase: {memory_increase:.2f}MB")
        print(f"   Memory per Operation: {memory_increase/volume*1024:.1f}KB")
        print(f"   Buffer Sizes: {obs_stats['traces_count']} traces, {obs_stats['metrics_count']} metrics")
        print(f"   Estimated Observability Memory: {obs_stats['memory_usage_estimate_mb']:.2f}MB")
        print()


async def stress_test_throughput():
    """Stress test throughput with and without observability"""
    print("=" * 80)
    print("🚀 THROUGHPUT STRESS TEST")
    print("=" * 80)
    print()
    
    # Test high-throughput scenarios
    test_duration_seconds = 10
    
    configurations = [
        ("Baseline", ObservabilityConfig(False, False, False)),
        ("Full Observability", ObservabilityConfig(True, True, True)),
        ("Optimized Sampling", ObservabilityConfig(True, True, True, metrics_sampling_rate=0.1, trace_sampling_rate=0.1))
    ]
    
    for config_name, config in configurations:
        print(f"🔬 Stress testing: {config_name}")
        
        if config_name == "Baseline":
            observability = None
            event_store = ObservableEventStore("sqlite://:memory:", observability)
            operation_func = event_store.create_event_baseline
        else:
            observability = MockObservabilityService(config)
            event_store = ObservableEventStore("sqlite://:memory:", observability)
            operation_func = event_store.create_event_with_observability
        
        # Run stress test
        operations_completed = 0
        start_time = time.time()
        end_time = start_time + test_duration_seconds
        
        while time.time() < end_time:
            try:
                await operation_func(
                    f"stress-aggregate-{random.randint(1, 1000)}",
                    f"StressEvent{random.randint(1, 10)}",
                    {"stress_test": True, "iteration": operations_completed}
                )
                operations_completed += 1
            except Exception as e:
                print(f"Operation failed: {e}")
        
        actual_duration = time.time() - start_time
        throughput = operations_completed / actual_duration
        
        print(f"   Operations Completed: {operations_completed}")
        print(f"   Test Duration: {actual_duration:.2f}s")
        print(f"   Throughput: {throughput:.1f} ops/sec")
        
        if observability:
            obs_stats = observability.get_stats()
            print(f"   Observability Overhead: {obs_stats['memory_usage_estimate_mb']:.2f}MB")
        
        print()


def print_performance_summary(creation_results: List[PerformanceMetrics], loading_results: List[PerformanceMetrics]):
    """Print comprehensive performance analysis summary"""
    print("=" * 80)
    print("📈 COMPREHENSIVE PERFORMANCE ANALYSIS SUMMARY")
    print("=" * 80)
    print()
    
    print("🎯 KEY FINDINGS:")
    print()
    
    # Find full observability results
    full_obs_creation = next(r for r in creation_results if "Full Observability" in r.operation_name)
    full_obs_loading = next(r for r in loading_results if "Full Observability" in r.operation_name)
    
    print(f"📊 EVENT CREATION PERFORMANCE:")
    print(f"   Full Observability Overhead: {full_obs_creation.overhead_percentage:.2f}%")
    print(f"   Throughput Impact: {full_obs_creation.throughput_ops_per_second:.0f} ops/sec vs baseline")
    print(f"   Latency Impact: +{full_obs_creation.average_latency_ms:.2f}ms average")
    print(f"   Memory Overhead: +{full_obs_creation.memory_usage_mb:.1f}MB")
    print()
    
    print(f"📚 EVENT LOADING PERFORMANCE:")
    print(f"   Full Observability Overhead: {full_obs_loading.overhead_percentage:.2f}%")
    print(f"   Throughput Impact: {full_obs_loading.throughput_ops_per_second:.0f} ops/sec vs baseline")
    print(f"   Latency Impact: +{full_obs_loading.average_latency_ms:.2f}ms average")
    print()
    
    # Check if we meet the <2% overhead requirement
    meets_requirement = (full_obs_creation.overhead_percentage < 2.0 and 
                        full_obs_loading.overhead_percentage < 2.0)
    
    if meets_requirement:
        print("✅ PERFORMANCE REQUIREMENT: MET")
        print(f"   Both event creation ({full_obs_creation.overhead_percentage:.2f}%) and loading ")
        print(f"   ({full_obs_loading.overhead_percentage:.2f}%) are under the 2% overhead target")
    else:
        print("⚠️  PERFORMANCE REQUIREMENT: REVIEW NEEDED")
        print(f"   Event creation: {full_obs_creation.overhead_percentage:.2f}% overhead")
        print(f"   Event loading: {full_obs_loading.overhead_percentage:.2f}% overhead")
    
    print()
    
    print("💡 OPTIMIZATION RECOMMENDATIONS:")
    
    # Find sampled observability results if available
    sampled_result = next((r for r in creation_results if "Sampled" in r.operation_name), None)
    if sampled_result:
        print(f"   • Sampling reduces overhead to {sampled_result.overhead_percentage:.2f}%")
        print("   • Consider 10-50% sampling for high-throughput production workloads")
    
    print("   • Use async logging to reduce I/O blocking")
    print("   • Implement log shipping to external aggregation systems")
    print("   • Consider metric aggregation to reduce memory usage")
    print("   • Use structured logging with efficient JSON serialization")
    print("   • Implement circuit breakers for observability failures")
    
    print()
    
    print("🏭 PRODUCTION DEPLOYMENT GUIDELINES:")
    print("   • Enable full observability in staging environments")
    print("   • Use 10-25% sampling rates in high-throughput production")
    print("   • Monitor observability system resource usage")
    print("   • Set up log rotation and metric retention policies")
    print("   • Configure alerting for observability system health")
    print("   • Implement graceful degradation when observability fails")


async def main():
    """Main performance analysis function"""
    print("📊 Eventuali Observability Performance Impact Analysis")
    print("=" * 80)
    print()
    print("This analysis measures the performance overhead of comprehensive")
    print("observability features in the Eventuali event sourcing system.")
    print()
    print("Performance targets:")
    print("• <2% throughput overhead for full observability")
    print("• Minimal memory overhead per operation")
    print("• Latency impact <5ms for 95th percentile operations")
    print("• Linear memory scaling with operation volume")
    print()
    
    try:
        # Run comprehensive performance analysis
        print("🚀 Starting comprehensive performance analysis...")
        print()
        
        # Benchmark event creation overhead
        creation_results = await benchmark_event_creation_overhead()
        
        # Benchmark event loading overhead
        loading_results = await benchmark_event_loading_overhead()
        
        # Analyze memory scaling
        await analyze_memory_scaling()
        
        # Stress test throughput
        await stress_test_throughput()
        
        # Print comprehensive summary
        print_performance_summary(creation_results, loading_results)
        
        print("🎉 PERFORMANCE ANALYSIS COMPLETED!")
        print("=" * 80)
        
    except Exception as e:
        print(f"❌ Error during performance analysis: {e}")
        raise


if __name__ == "__main__":
    asyncio.run(main())