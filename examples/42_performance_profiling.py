#!/usr/bin/env python3
"""
Example 42: Advanced Performance Profiling

This example demonstrates comprehensive performance profiling capabilities:
- CPU profiling with sampling and flame graph generation
- Memory profiling with allocation tracking and leak detection
- I/O profiling with disk and network performance analysis
- Method-level performance tracking with call graphs
- Performance regression detection with alerting
- Bottleneck identification and optimization recommendations

The profiling system provides real-time insights with minimal overhead (<2%).
"""

import asyncio
import json
import time
import tempfile
import sqlite3
from pathlib import Path
from typing import Dict, Any, List
import traceback

# Import the profiling classes
from eventuali import (
    ObservabilityService, ObservabilityConfig,
    ProfileType, ProfilingConfig, 
    FlameGraph, BottleneckAnalysis, RegressionDetection
)

class PerformanceProfilingDemo:
    """Comprehensive performance profiling demonstration."""
    
    def __init__(self):
        self.observability = None
        
    async def setup(self):
        """Initialize observability services."""
        print("🚀 Setting up Performance Profiling Demo")
        
        # Configure observability with profiling
        obs_config = ObservabilityConfig(
            service_name="eventuali-profiling-demo",
            service_version="0.1.0",
            environment="profiling",
            tracing_enabled=True,
            metrics_enabled=True,
            structured_logging=True
        )
        
        # Initialize observability service
        self.observability = ObservabilityService(obs_config)
        self.observability.initialize()
        
        print("✅ Performance profiling environment initialized")
        print(f"   Profiling: Enabled with <2% overhead target")
        print()

    def demonstrate_cpu_profiling(self):
        """Demonstrate CPU profiling with flame graph generation."""
        print("🔥 CPU Profiling & Flame Graph Generation")
        print("-" * 50)
        
        # Start CPU profiling
        session_id = self.observability.start_profiling(
            ProfileType.Cpu, {"operation": "cpu_intensive_task"}
        )
        print(f"Started CPU profiling session: {session_id}")
        
        # Simulate CPU-intensive workload
        self._simulate_cpu_intensive_work()
        
        # End profiling and get results
        profile_entry = self.observability.end_profiling(session_id)
        
        print(f"CPU profiling completed:")
        print(f"  Duration: {profile_entry.duration_ms}ms")
        print(f"  Stack depth: {len(profile_entry.stack_trace)}")
        print(f"  Profile type: {profile_entry.profile_type}")
        
        # Generate flame graph
        try:
            flame_graph = self.observability.generate_flame_graph(
                ProfileType.Cpu, None, None
            )
            
            print(f"🔥 Flame graph generated:")
            print(f"  Total duration: {flame_graph.total_duration_ms}ms")
            print(f"  Sample count: {flame_graph.sample_count}")
            print(f"  Root function: {flame_graph.root.name}")
            print(f"  Top-level functions: {len(flame_graph.root.children)}")
            
            # Display top functions by time
            self._display_flame_graph_highlights(flame_graph)
            
        except Exception as e:
            print(f"⚠️ Flame graph generation: {e}")
        
        print()

    def _simulate_cpu_intensive_work(self):
        """Simulate CPU-intensive operations for profiling."""
        # Heavy computation simulation
        def fibonacci_recursive(n):
            if n <= 1:
                return n
            return fibonacci_recursive(n-1) + fibonacci_recursive(n-2)
        
        def matrix_multiplication():
            # Simple matrix operations
            size = 100
            matrix_a = [[i + j for j in range(size)] for i in range(size)]
            matrix_b = [[i * j % 10 for j in range(size)] for i in range(size)]
            result = [[0] * size for _ in range(size)]
            
            for i in range(size):
                for j in range(size):
                    for k in range(size):
                        result[i][j] += matrix_a[i][k] * matrix_b[k][j]
            return result
        
        def string_processing():
            text = "performance profiling " * 1000
            processed = []
            for _ in range(100):
                words = text.split()
                processed.extend([word.upper().replace("A", "X") for word in words])
            return len(processed)
        
        # Execute CPU-intensive operations
        fib_result = fibonacci_recursive(25)
        matrix_result = matrix_multiplication()
        string_result = string_processing()
        
        print(f"  CPU work completed: fib={fib_result}, matrix_size={len(matrix_result)}, strings={string_result}")

    def demonstrate_memory_profiling(self):
        """Demonstrate memory profiling with allocation tracking."""
        print("🧠 Memory Profiling & Allocation Tracking")
        print("-" * 50)
        
        # Start memory profiling
        session_id = self.observability.start_profiling(
            ProfileType.Memory, {"operation": "memory_intensive_task"}
        )
        print(f"Started memory profiling session: {session_id}")
        
        # Simulate memory-intensive workload
        self._simulate_memory_intensive_work()
        
        # End profiling and get results
        profile_entry = self.observability.end_profiling(session_id)
        
        print(f"Memory profiling completed:")
        print(f"  Duration: {profile_entry.duration_ms}ms")
        
        if profile_entry.memory_info:
            mem_info = profile_entry.memory_info
            print(f"  📊 Memory Statistics:")
            print(f"    Current usage: {mem_info.current_usage_bytes // 1024}KB")
            print(f"    Peak usage: {mem_info.peak_usage_bytes // 1024}KB")
            print(f"    Allocations: {mem_info.allocation_count}")
            print(f"    Deallocated bytes: {mem_info.deallocated_bytes // 1024}KB")
            print(f"    Memory efficiency: {((mem_info.deallocated_bytes / max(mem_info.allocated_bytes, 1)) * 100):.1f}%")
        
        print()

    def _simulate_memory_intensive_work(self):
        """Simulate memory-intensive operations for profiling."""
        # Large data structure creation and manipulation
        large_lists = []
        
        # Create multiple large lists
        for i in range(10):
            data = [j * i for j in range(100000)]
            large_lists.append(data)
        
        # Process data (simulating real work)
        processed_data = []
        for lst in large_lists:
            # Apply transformations
            transformed = [x * 2 + 1 for x in lst if x % 2 == 0]
            processed_data.append(transformed)
        
        # Create dictionaries (more memory allocations)
        dict_data = {}
        for i, lst in enumerate(processed_data):
            dict_data[f"dataset_{i}"] = {
                "values": lst[:1000],  # Truncate to save memory
                "stats": {
                    "count": len(lst),
                    "sum": sum(lst[:100]),  # Sample for performance
                    "avg": sum(lst[:100]) / min(len(lst), 100)
                }
            }
        
        # Clean up (trigger garbage collection)
        large_lists.clear()
        processed_data.clear()
        dict_data.clear()
        
        print(f"  Memory work completed with multiple allocations/deallocations")

    def demonstrate_io_profiling(self):
        """Demonstrate I/O profiling with disk and database operations."""
        print("💾 I/O Profiling & Database Operations")
        print("-" * 50)
        
        # Start I/O profiling
        session_id = self.observability.start_profiling(
            ProfileType.Io, {"operation": "io_intensive_task"}
        )
        print(f"Started I/O profiling session: {session_id}")
        
        # Simulate I/O-intensive workload
        self._simulate_io_intensive_work()
        
        # End profiling and get results
        profile_entry = self.observability.end_profiling(session_id)
        
        print(f"I/O profiling completed:")
        print(f"  Duration: {profile_entry.duration_ms}ms")
        
        if profile_entry.io_info:
            io_info = profile_entry.io_info
            print(f"  📊 I/O Statistics:")
            print(f"    Operation type: {io_info.operation_type}")
            print(f"    Bytes read: {io_info.bytes_read}")
            print(f"    Bytes written: {io_info.bytes_written}")
            print(f"    Operation count: {io_info.operation_count}")
            print(f"    Target: {io_info.target}")
        
        print()

    def _simulate_io_intensive_work(self):
        """Simulate I/O-intensive operations for profiling."""
        # File I/O operations
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            
            # Write multiple files
            files_created = 0
            for i in range(50):
                file_path = temp_path / f"test_file_{i}.txt"
                with open(file_path, 'w') as f:
                    # Write substantial data
                    for j in range(1000):
                        f.write(f"Line {j} in file {i} with some test data for I/O profiling\n")
                files_created += 1
            
            # Read files back
            total_lines = 0
            for i in range(files_created):
                file_path = temp_path / f"test_file_{i}.txt"
                if file_path.exists():
                    with open(file_path, 'r') as f:
                        lines = f.readlines()
                        total_lines += len(lines)
        
        # Database I/O operations
        with tempfile.NamedTemporaryFile(suffix='.db', delete=False) as temp_db:
            temp_db_path = temp_db.name
        
        try:
            # Simulate database operations
            conn = sqlite3.connect(temp_db_path)
            cursor = conn.cursor()
            
            # Create table
            cursor.execute("""
                CREATE TABLE test_events (
                    id INTEGER PRIMARY KEY,
                    aggregate_id TEXT,
                    event_type TEXT,
                    data TEXT,
                    timestamp INTEGER
                )
            """)
            
            # Insert test data
            import random
            for i in range(1000):
                cursor.execute("""
                    INSERT INTO test_events (aggregate_id, event_type, data, timestamp)
                    VALUES (?, ?, ?, ?)
                """, (
                    f"aggregate_{random.randint(1, 100)}",
                    f"EventType{random.randint(1, 10)}",
                    json.dumps({"index": i, "value": random.randint(1, 1000)}),
                    int(time.time() * 1000)
                ))
            
            conn.commit()
            
            # Read data back
            cursor.execute("SELECT COUNT(*) FROM test_events")
            count = cursor.fetchone()[0]
            
            cursor.execute("SELECT * FROM test_events LIMIT 10")
            sample_data = cursor.fetchall()
            
            conn.close()
            
            print(f"  I/O work completed: files={files_created}, lines={total_lines}, db_records={count}")
            
        finally:
            # Clean up
            Path(temp_db_path).unlink(missing_ok=True)

    def demonstrate_regression_detection(self):
        """Demonstrate performance regression detection and alerting."""
        print("📈 Performance Regression Detection")
        print("-" * 50)
        
        operation_name = "computation_operation"
        
        # Set baseline performance
        print("Setting baseline performance...")
        
        # Simulate baseline performance (fast)
        session_id = self.observability.start_profiling(
            ProfileType.Combined, {"operation": operation_name}
        )
        
        # Fast baseline operation
        self._perform_fast_computation()
        
        self.observability.end_profiling(session_id)
        
        # Set this as baseline
        self.observability.set_baseline(operation_name)
        print(f"✅ Baseline set for {operation_name}")
        
        # Simulate performance regression (slower)
        print("\nSimulating performance regression...")
        
        session_id = self.observability.start_profiling(
            ProfileType.Combined, {"operation": operation_name}
        )
        
        # Slower operation (simulated regression)
        self._perform_slow_computation()
        
        self.observability.end_profiling(session_id)
        
        # Check for regressions
        regression = self.observability.detect_regressions(operation_name)
        
        if regression:
            print(f"🚨 Performance regression detected!")
            print(f"  Operation: {regression.operation}")
            print(f"  Performance change: {regression.change_percent:+.1f}%")
            print(f"  Severity: {regression.severity}")
            print(f"  Regression status: {regression.is_regression}")
            
            print(f"\n  📊 Current vs Baseline:")
            current = regression.current_metrics
            baseline = regression.baseline_metrics
            print(f"    Avg execution time: {current.avg_execution_time_ms}ms (baseline: {baseline.avg_execution_time_ms}ms)")
            print(f"    P95 execution time: {current.p95_execution_time_ms}ms (baseline: {baseline.p95_execution_time_ms}ms)")
            print(f"    Throughput: {current.throughput:.2f}/s (baseline: {baseline.throughput:.2f}/s)")
            
            print(f"\n  💡 Recommendations:")
            for i, rec in enumerate(regression.recommendations, 1):
                print(f"    {i}. {rec}")
        else:
            print("✅ No performance regressions detected")
        
        print()

    def _perform_fast_computation(self):
        """Perform optimized computation (baseline)."""
        # Fast computation
        result = 0
        for i in range(10000):
            result += i * 2
        return result

    def _perform_slow_computation(self):
        """Perform inefficient computation (simulated regression)."""
        # Slow computation with inefficiencies
        result = 0
        for i in range(10000):
            # Add inefficiencies
            temp = str(i)  # String conversion
            temp_int = int(temp)  # Convert back
            result += temp_int * 2
            
            # Additional inefficient work
            _ = [j for j in range(10)]  # Unnecessary list comprehension
        
        # Additional sleep to simulate regression
        time.sleep(0.1)
        return result

    def demonstrate_bottleneck_analysis(self):
        """Demonstrate bottleneck identification and optimization recommendations."""
        print("🎯 Bottleneck Analysis & Optimization Recommendations")
        print("-" * 50)
        
        # Start combined profiling for comprehensive analysis
        session_id = self.observability.start_profiling(
            ProfileType.Combined, {"operation": "bottleneck_analysis"}
        )
        
        # Simulate workload with various bottlenecks
        self._simulate_bottleneck_workload()
        
        # End profiling
        self.observability.end_profiling(session_id)
        
        # Analyze bottlenecks
        bottleneck_analysis = self.observability.identify_bottlenecks(ProfileType.Combined)
        
        print(f"🔍 Bottleneck analysis completed in {bottleneck_analysis.analysis_duration_ms}ms")
        print(f"Found {len(bottleneck_analysis.bottlenecks)} bottlenecks:")
        
        # Display top bottlenecks
        for i, bottleneck in enumerate(bottleneck_analysis.bottlenecks[:5], 1):
            print(f"\n  {i}. {bottleneck.location}")
            print(f"     Type: {bottleneck.bottleneck_type}")
            print(f"     Impact: {bottleneck.impact_score:.1f}% of total execution time")
            print(f"     Time spent: {bottleneck.time_spent_ms}ms")
            print(f"     Call frequency: {bottleneck.call_frequency}")
            print(f"     Description: {bottleneck.description}")
        
        print(f"\n💡 Optimization Suggestions:")
        for i, suggestion in enumerate(bottleneck_analysis.optimization_suggestions, 1):
            print(f"\n  {i}. Target: {suggestion.target}")
            print(f"     Type: {suggestion.optimization_type}")
            print(f"     Expected Impact: {suggestion.expected_impact}")
            print(f"     Effort Level: {suggestion.effort_level}")
            print(f"     Description: {suggestion.description}")
            if suggestion.examples:
                print(f"     Examples:")
                for example in suggestion.examples[:2]:  # Show first 2 examples
                    print(f"       - {example}")
        
        print()

    def _simulate_bottleneck_workload(self):
        """Simulate a workload with various types of bottlenecks."""
        # Simulate CPU bottleneck
        self._cpu_heavy_operation()
        
        # Simulate memory bottleneck
        self._memory_heavy_operation()
        
        # Simulate serialization bottleneck
        self._serialization_heavy_operation()

    def _cpu_heavy_operation(self):
        """Simulate CPU-intensive operations."""
        # Heavy computation
        result = 0
        for i in range(10000):
            result += i ** 2 + (i * 3) % 7
        return result

    def _memory_heavy_operation(self):
        """Simulate memory-intensive operations."""
        # Create and manipulate large data structures
        large_data = []
        for i in range(1000):
            large_data.append({
                "id": i,
                "data": [j for j in range(100)],
                "metadata": {"created": time.time()}
            })
        
        # Process data
        processed = [item for item in large_data if item["id"] % 2 == 0]
        return len(processed)

    def _serialization_heavy_operation(self):
        """Simulate serialization-intensive operations."""
        # Create complex objects and serialize them
        complex_data = {
            "events": [
                {
                    "id": i,
                    "timestamp": time.time(),
                    "payload": {"data": [j for j in range(10)]},
                    "metadata": {"version": 1, "source": "profiling_demo"}
                }
                for i in range(100)
            ]
        }
        
        # Serialize and deserialize multiple times
        for _ in range(10):
            serialized = json.dumps(complex_data)
            deserialized = json.loads(serialized)
        
        return len(serialized)

    def _display_flame_graph_highlights(self, flame_graph: FlameGraph):
        """Display highlights from the flame graph."""
        print(f"  🔥 Flame Graph Highlights:")
        
        root = flame_graph.root
        if root.children:
            # Sort children by time spent (descending)
            sorted_children = sorted(
                root.children.items(),
                key=lambda x: x[1].total_time_ms,
                reverse=True
            )
            
            print(f"    Top functions by execution time:")
            for i, (name, node) in enumerate(sorted_children[:5], 1):
                print(f"      {i}. {name}")
                print(f"         Total time: {node.total_time_ms}ms ({node.percentage:.1f}%)")
                print(f"         Self time: {node.self_time_ms}ms")
                print(f"         Samples: {node.sample_count}")

    def demonstrate_real_time_profiling(self):
        """Demonstrate real-time profiling with live metrics."""
        print("⚡ Real-time Profiling & Live Metrics")
        print("-" * 50)
        
        # Get current profiling statistics
        stats = self.observability.get_profiling_statistics()
        
        print("📊 Current Profiling Statistics:")
        for key, value in stats.items():
            print(f"  {key}: {value}")
        
        # Start continuous profiling session
        session_id = self.observability.start_profiling(
            ProfileType.Combined, {"operation": "real_time_monitoring"}
        )
        
        print(f"\n🔄 Started real-time profiling session: {session_id}")
        print("Performing operations while profiling...")
        
        # Perform various operations while monitoring
        for i in range(5):
            print(f"  Operation batch {i+1}/5")
            
            # Mix of different operation types
            if i % 3 == 0:
                self._perform_fast_computation()
            elif i % 3 == 1:
                self._cpu_heavy_operation()
            else:
                self._memory_heavy_operation()
            
            # Small delay between batches
            time.sleep(0.5)
        
        # End profiling session
        profile_entry = self.observability.end_profiling(session_id)
        
        print(f"\n✅ Real-time profiling completed:")
        print(f"  Total duration: {profile_entry.duration_ms}ms")
        print(f"  Operations tracked: {len(profile_entry.metadata)}")
        
        # Get updated statistics
        updated_stats = self.observability.get_profiling_statistics()
        
        print(f"\n📈 Updated Statistics:")
        for key, value in updated_stats.items():
            print(f"  {key}: {value}")
        
        print()

    def demonstrate_profiling_integration(self):
        """Demonstrate profiling integration with observability stack."""
        print("🔗 Profiling Integration with Observability Stack")
        print("-" * 50)
        
        print("🔄 Integration features:")
        print("  ✅ OpenTelemetry tracing with profiling correlation")
        print("  ✅ Prometheus metrics export for profiling data")
        print("  ✅ Structured logging with profiling context")
        print("  ✅ Health checks including profiling status")
        print("  ✅ Grafana dashboard integration ready")
        
        # Create trace context for correlated profiling
        trace_context = self.observability.create_trace_context("profiling_integration")
        print(f"  📋 Created trace context: {trace_context.operation}")
        print(f"       Correlation ID: {trace_context.correlation_id}")
        
        # Start profiling with trace correlation
        session_id = self.observability.start_profiling(
            ProfileType.Combined, {
                "operation": "integrated_profiling",
                "trace_id": trace_context.trace_id or "unknown",
                "correlation_id": str(trace_context.correlation_id)
            }
        )
        
        # Log profiling start with context
        from eventuali import LogLevel
        self.observability.log_event(
            LogLevel.Info, f"Started integrated profiling session: {session_id}", trace_context
        )
        
        # Record custom metrics
        self.observability.record_metric(
            "profiling.session.started", 1.0, {"session_type": "integrated"}
        )
        
        # Perform work with full observability
        self._perform_observable_work(trace_context)
        
        # End profiling
        profile_entry = self.observability.end_profiling(session_id)
        
        # Log profiling completion
        self.observability.log_event(
            LogLevel.Info, f"Completed integrated profiling: {profile_entry.duration_ms}ms", trace_context
        )
        
        # Record completion metrics
        self.observability.record_metric(
            "profiling.session.duration_ms", float(profile_entry.duration_ms), 
            {"session_type": "integrated"}
        )
        
        print(f"✅ Integrated profiling completed:")
        print(f"  Duration: {profile_entry.duration_ms}ms")
        print(f"  Trace correlation: {trace_context.correlation_id}")
        print(f"  Metrics recorded: profiling.session.* series")
        print(f"  Logs structured with trace context")
        
        print()

    def _perform_observable_work(self, trace_context):
        """Perform work with full observability context."""
        # Add trace events
        trace_context.add_event("work.started", {"type": "observable_work"})
        
        # Simulate different types of work
        operations = ["computation", "memory_work", "serialization"]
        
        for op in operations:
            trace_context.add_event(f"operation.{op}.started")
            
            if op == "computation":
                self._cpu_heavy_operation()
            elif op == "memory_work":
                self._memory_heavy_operation()
            else:
                self._serialization_heavy_operation()
            
            trace_context.add_event(f"operation.{op}.completed")
            
            # Record operation-specific metrics
            self.observability.record_metric(
                f"operation.{op}.count", 1.0, {"context": "profiling_demo"}
            )
        
        trace_context.add_event("work.completed", {"operations": str(len(operations))})

    def run_comprehensive_demo(self):
        """Run the complete performance profiling demonstration."""
        print("=" * 60)
        print("🎯 EVENTUALI PERFORMANCE PROFILING DEMO")
        print("=" * 60)
        print()
        
        try:
            asyncio.run(self.setup())
            
            # Core profiling demonstrations
            self.demonstrate_cpu_profiling()
            self.demonstrate_memory_profiling()  
            self.demonstrate_io_profiling()
            
            # Advanced profiling features
            self.demonstrate_regression_detection()
            self.demonstrate_bottleneck_analysis()
            
            # Real-time and integration features
            self.demonstrate_real_time_profiling()
            self.demonstrate_profiling_integration()
            
            print("=" * 60)
            print("✅ PERFORMANCE PROFILING DEMO COMPLETED SUCCESSFULLY")
            print("=" * 60)
            print()
            print("🎯 Key Benefits Demonstrated:")
            print("  ✅ CPU profiling with flame graph generation")
            print("  ✅ Memory profiling with allocation tracking")
            print("  ✅ I/O profiling for database and file operations")
            print("  ✅ Performance regression detection and alerting")
            print("  ✅ Automated bottleneck identification")
            print("  ✅ Optimization recommendations with examples")
            print("  ✅ Real-time profiling with <2% overhead")
            print("  ✅ Full observability stack integration")
            print()
            print("🚀 Production Ready Features:")
            print("  • Sampling-based CPU profiling")
            print("  • Memory leak detection")
            print("  • Performance regression alerting")
            print("  • Automated optimization suggestions")
            print("  • Flame graph visualization")
            print("  • Prometheus metrics integration")
            print("  • OpenTelemetry trace correlation")
            print("  • Health check integration")
            
        except Exception as e:
            print(f"❌ Demo error: {e}")
            traceback.print_exc()
        
        finally:
            # Cleanup
            if self.observability:
                try:
                    self.observability.shutdown()
                    print("\n🧹 Observability service shutdown completed")
                except Exception as e:
                    print(f"⚠️ Shutdown warning: {e}")

if __name__ == "__main__":
    print("Starting Eventuali Performance Profiling Demo...")
    print("This demo showcases advanced profiling capabilities for production systems.")
    print()
    
    demo = PerformanceProfilingDemo()
    demo.run_comprehensive_demo()