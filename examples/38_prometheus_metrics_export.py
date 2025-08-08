#!/usr/bin/env python3
"""
Example 38: Prometheus Metrics Export

This example demonstrates:
1. Comprehensive metrics collection for event sourcing operations
2. Prometheus-compatible metric formats and labels
3. Performance metrics with histograms and counters
4. Business metrics tracking
5. Health and operational metrics
6. HTTP endpoint for metrics scraping

The implementation shows how to instrument an event sourcing system
with Prometheus metrics for production monitoring.
"""

import asyncio
import time
import random
import json
from datetime import datetime, timezone
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, field
from collections import defaultdict, Counter
import threading
from http.server import HTTPServer, BaseHTTPRequestHandler

import eventuali


@dataclass
class MetricValue:
    """Represents a metric value with labels"""
    name: str
    value: float
    labels: Dict[str, str] = field(default_factory=dict)
    timestamp: float = field(default_factory=time.time)
    help_text: str = ""
    metric_type: str = "counter"  # counter, gauge, histogram


class PrometheusMetricsCollector:
    """Prometheus-compatible metrics collector for Eventuali"""
    
    def __init__(self, service_name: str = "eventuali"):
        self.service_name = service_name
        self.metrics: Dict[str, MetricValue] = {}
        self.histograms: Dict[str, List[float]] = defaultdict(list)
        self.counters: Dict[str, int] = defaultdict(int)
        self.gauges: Dict[str, float] = defaultdict(float)
        self.labels: Dict[str, Dict[str, str]] = {}
        self.lock = threading.Lock()
        
        # Initialize standard metrics
        self._initialize_standard_metrics()
    
    def _initialize_standard_metrics(self):
        """Initialize standard Eventuali metrics"""
        # Event-related metrics
        self.register_counter(
            "eventuali_events_created_total",
            "Total number of events created",
            ["event_type", "aggregate_type", "tenant_id"]
        )
        
        self.register_counter(
            "eventuali_events_stored_total", 
            "Total number of events stored",
            ["status", "backend_type"]
        )
        
        self.register_counter(
            "eventuali_events_loaded_total",
            "Total number of events loaded", 
            ["aggregate_type", "tenant_id"]
        )
        
        self.register_histogram(
            "eventuali_event_processing_duration_seconds",
            "Time spent processing events",
            ["operation_type", "event_type"]
        )
        
        self.register_histogram(
            "eventuali_event_size_bytes", 
            "Size of events in bytes",
            ["event_type", "compression"]
        )
        
        # Database-related metrics
        self.register_histogram(
            "eventuali_database_query_duration_seconds",
            "Database query execution time",
            ["query_type", "backend", "table"]
        )
        
        self.register_counter(
            "eventuali_database_connections_total",
            "Total database connections created",
            ["backend", "status"]
        )
        
        self.register_gauge(
            "eventuali_database_connections_active",
            "Number of active database connections",
            ["backend"]
        )
        
        # Performance metrics
        self.register_gauge(
            "eventuali_throughput_events_per_second",
            "Current events processing throughput",
            ["service", "operation"]
        )
        
        self.register_histogram(
            "eventuali_aggregate_load_duration_seconds",
            "Time to load aggregate from events",
            ["aggregate_type", "event_count_bucket"]
        )
        
        # Business metrics
        self.register_counter(
            "eventuali_business_operations_total",
            "Total business operations processed",
            ["operation_type", "tenant_id", "status"]
        )
        
        self.register_gauge(
            "eventuali_active_aggregates",
            "Number of active aggregates in memory",
            ["aggregate_type", "tenant_id"]
        )
        
        # Error metrics
        self.register_counter(
            "eventuali_errors_total",
            "Total errors encountered",
            ["error_type", "operation", "severity"]
        )
        
        # System health metrics
        self.register_gauge(
            "eventuali_memory_usage_bytes",
            "Memory usage in bytes",
            ["component"]
        )
        
        self.register_gauge(
            "eventuali_cpu_usage_percent",
            "CPU usage percentage",
            ["service"]
        )
    
    def register_counter(self, name: str, help_text: str, label_names: List[str]):
        """Register a counter metric"""
        metric = MetricValue(
            name=name,
            value=0.0,
            help_text=help_text,
            metric_type="counter"
        )
        with self.lock:
            self.metrics[name] = metric
            self.labels[name] = {label: "" for label in label_names}
    
    def register_gauge(self, name: str, help_text: str, label_names: List[str]):
        """Register a gauge metric"""
        metric = MetricValue(
            name=name,
            value=0.0,
            help_text=help_text,
            metric_type="gauge"
        )
        with self.lock:
            self.metrics[name] = metric
            self.labels[name] = {label: "" for label in label_names}
    
    def register_histogram(self, name: str, help_text: str, label_names: List[str]):
        """Register a histogram metric"""
        metric = MetricValue(
            name=name,
            value=0.0,
            help_text=help_text,
            metric_type="histogram"
        )
        with self.lock:
            self.metrics[name] = metric
            self.labels[name] = {label: "" for label in label_names}
    
    def increment_counter(self, name: str, labels: Dict[str, str] = None, value: float = 1.0):
        """Increment a counter metric"""
        if labels is None:
            labels = {}
        
        key = self._get_metric_key(name, labels)
        with self.lock:
            self.counters[key] += value
            
        print(f"📊 Counter incremented: {name} = {self.counters[key]} {labels}")
    
    def set_gauge(self, name: str, value: float, labels: Dict[str, str] = None):
        """Set a gauge metric value"""
        if labels is None:
            labels = {}
        
        key = self._get_metric_key(name, labels)
        with self.lock:
            self.gauges[key] = value
            
        print(f"📏 Gauge set: {name} = {value} {labels}")
    
    def observe_histogram(self, name: str, value: float, labels: Dict[str, str] = None):
        """Add an observation to a histogram"""
        if labels is None:
            labels = {}
        
        key = self._get_metric_key(name, labels)
        with self.lock:
            self.histograms[key].append(value)
            
        print(f"📈 Histogram observed: {name} = {value} {labels}")
    
    def _get_metric_key(self, name: str, labels: Dict[str, str]) -> str:
        """Generate a unique key for a metric with labels"""
        label_str = ",".join(f"{k}={v}" for k, v in sorted(labels.items()))
        return f"{name}{{{label_str}}}"
    
    def get_prometheus_format(self) -> str:
        """Export all metrics in Prometheus text format"""
        output = []
        
        with self.lock:
            # Export counters
            counter_names = set()
            for key in self.counters:
                name = key.split('{')[0]
                counter_names.add(name)
            
            for name in sorted(counter_names):
                if name in self.metrics:
                    output.append(f"# HELP {name} {self.metrics[name].help_text}")
                    output.append(f"# TYPE {name} counter")
                
                for key, value in self.counters.items():
                    if key.startswith(name):
                        labels_part = key[len(name):]
                        output.append(f"{name}{labels_part} {value}")
                
                output.append("")
            
            # Export gauges
            gauge_names = set()
            for key in self.gauges:
                name = key.split('{')[0]
                gauge_names.add(name)
            
            for name in sorted(gauge_names):
                if name in self.metrics:
                    output.append(f"# HELP {name} {self.metrics[name].help_text}")
                    output.append(f"# TYPE {name} gauge")
                
                for key, value in self.gauges.items():
                    if key.startswith(name):
                        labels_part = key[len(name):]
                        output.append(f"{name}{labels_part} {value}")
                
                output.append("")
            
            # Export histograms (simplified)
            histogram_names = set()
            for key in self.histograms:
                name = key.split('{')[0]
                histogram_names.add(name)
            
            for name in sorted(histogram_names):
                if name in self.metrics:
                    output.append(f"# HELP {name} {self.metrics[name].help_text}")
                    output.append(f"# TYPE {name} histogram")
                
                for key, values in self.histograms.items():
                    if key.startswith(name):
                        labels_part = key[len(name):]
                        
                        # Calculate histogram buckets
                        buckets = [0.001, 0.01, 0.1, 1.0, 10.0, 100.0, float('inf')]
                        counts = {bucket: 0 for bucket in buckets}
                        total_count = len(values)
                        total_sum = sum(values)
                        
                        for value in values:
                            for bucket in buckets:
                                if value <= bucket:
                                    counts[bucket] += 1
                        
                        # Output bucket counts
                        cumulative = 0
                        for bucket in sorted(counts.keys()):
                            if bucket == float('inf'):
                                bucket_str = "+Inf"
                            else:
                                bucket_str = str(bucket)
                            
                            cumulative += counts[bucket]
                            bucket_labels = labels_part.rstrip('}') + f',le="{bucket_str}"' + '}'
                            if labels_part == '':
                                bucket_labels = f'{{le="{bucket_str}"}}'
                            
                            output.append(f"{name}_bucket{bucket_labels} {cumulative}")
                        
                        # Output count and sum
                        output.append(f"{name}_count{labels_part} {total_count}")
                        output.append(f"{name}_sum{labels_part} {total_sum}")
                
                output.append("")
        
        return "\n".join(output)
    
    def get_metrics_summary(self) -> Dict[str, Any]:
        """Get a summary of all metrics for debugging"""
        with self.lock:
            return {
                "counters": dict(self.counters),
                "gauges": dict(self.gauges),
                "histograms": {
                    k: {
                        "count": len(v),
                        "sum": sum(v),
                        "min": min(v) if v else 0,
                        "max": max(v) if v else 0,
                        "avg": sum(v) / len(v) if v else 0
                    } for k, v in self.histograms.items()
                },
                "timestamp": time.time()
            }


class MetricsHTTPHandler(BaseHTTPRequestHandler):
    """HTTP handler for serving Prometheus metrics"""
    
    def __init__(self, metrics_collector: PrometheusMetricsCollector):
        self.metrics_collector = metrics_collector
    
    def __call__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
    
    def do_GET(self):
        if self.path == "/metrics":
            self.send_response(200)
            self.send_header('Content-Type', 'text/plain; charset=utf-8')
            self.end_headers()
            
            metrics_text = self.metrics_collector.get_prometheus_format()
            self.wfile.write(metrics_text.encode('utf-8'))
        elif self.path == "/health":
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            
            health_data = {
                "status": "healthy",
                "timestamp": datetime.now(timezone.utc).isoformat(),
                "metrics_available": True
            }
            self.wfile.write(json.dumps(health_data).encode('utf-8'))
        else:
            self.send_response(404)
            self.end_headers()
    
    def log_message(self, format, *args):
        # Suppress default logging
        pass


class ObservabilityEventStore:
    """Event store with comprehensive Prometheus metrics"""
    
    def __init__(self, connection_string: str, metrics: PrometheusMetricsCollector):
        self.connection_string = connection_string
        self.metrics = metrics
        self.operation_count = 0
        print(f"🔗 Initialized EventStore with metrics: {connection_string}")
    
    async def create_event_with_metrics(self, aggregate_id: str, event_type: str,
                                       event_data: dict, tenant_id: str = "default") -> dict:
        """Create an event with comprehensive metrics tracking"""
        start_time = time.time()
        operation_timer = time.time()
        
        try:
            # Simulate event processing
            await asyncio.sleep(random.uniform(0.005, 0.02))
            
            # Calculate metrics
            processing_duration = time.time() - start_time
            event_size = len(json.dumps(event_data))
            
            # Increment counters
            self.metrics.increment_counter(
                "eventuali_events_created_total",
                {"event_type": event_type, "aggregate_type": "Order", "tenant_id": tenant_id}
            )
            
            self.metrics.increment_counter(
                "eventuali_events_stored_total",
                {"status": "success", "backend_type": "sqlite"}
            )
            
            # Record histograms
            self.metrics.observe_histogram(
                "eventuali_event_processing_duration_seconds",
                processing_duration,
                {"operation_type": "create", "event_type": event_type}
            )
            
            self.metrics.observe_histogram(
                "eventuali_event_size_bytes",
                event_size,
                {"event_type": event_type, "compression": "none"}
            )
            
            # Update gauges
            self.operation_count += 1
            self.metrics.set_gauge(
                "eventuali_throughput_events_per_second",
                self.operation_count / (time.time() - operation_timer + 1),
                {"service": "event_store", "operation": "create"}
            )
            
            # Simulate database metrics
            db_duration = random.uniform(0.001, 0.008)
            self.metrics.observe_histogram(
                "eventuali_database_query_duration_seconds",
                db_duration,
                {"query_type": "insert", "backend": "sqlite", "table": "events"}
            )
            
            return {
                "event_id": f"event-{self.operation_count:06d}",
                "aggregate_id": aggregate_id,
                "event_type": event_type,
                "tenant_id": tenant_id,
                "processing_duration_ms": processing_duration * 1000,
                "event_size_bytes": event_size,
                "created_at": datetime.now(timezone.utc).isoformat()
            }
            
        except Exception as e:
            # Record error metrics
            self.metrics.increment_counter(
                "eventuali_errors_total",
                {"error_type": type(e).__name__, "operation": "create_event", "severity": "error"}
            )
            
            self.metrics.increment_counter(
                "eventuali_events_stored_total",
                {"status": "error", "backend_type": "sqlite"}
            )
            raise
    
    async def load_aggregate_with_metrics(self, aggregate_id: str, 
                                         tenant_id: str = "default") -> dict:
        """Load an aggregate with metrics tracking"""
        start_time = time.time()
        
        try:
            # Simulate loading events
            event_count = random.randint(1, 50)
            await asyncio.sleep(event_count * 0.001)  # Simulate load time based on event count
            
            load_duration = time.time() - start_time
            
            # Record metrics
            self.metrics.increment_counter(
                "eventuali_events_loaded_total",
                {"aggregate_type": "Order", "tenant_id": tenant_id}
            )
            
            # Determine bucket for histogram
            if event_count <= 10:
                bucket = "0-10"
            elif event_count <= 50:
                bucket = "11-50"
            else:
                bucket = "50+"
            
            self.metrics.observe_histogram(
                "eventuali_aggregate_load_duration_seconds",
                load_duration,
                {"aggregate_type": "Order", "event_count_bucket": bucket}
            )
            
            # Update active aggregates gauge
            active_aggregates = random.randint(50, 200)
            self.metrics.set_gauge(
                "eventuali_active_aggregates",
                active_aggregates,
                {"aggregate_type": "Order", "tenant_id": tenant_id}
            )
            
            return {
                "aggregate_id": aggregate_id,
                "event_count": event_count,
                "load_duration_ms": load_duration * 1000,
                "tenant_id": tenant_id,
                "version": event_count
            }
            
        except Exception as e:
            self.metrics.increment_counter(
                "eventuali_errors_total",
                {"error_type": type(e).__name__, "operation": "load_aggregate", "severity": "error"}
            )
            raise


async def simulate_business_operations(event_store: ObservabilityEventStore, 
                                      metrics: PrometheusMetricsCollector):
    """Simulate various business operations with metrics"""
    print("\n🏪 SIMULATING BUSINESS OPERATIONS")
    print("-" * 50)
    
    # Simulate different types of business operations
    operations = [
        ("create_order", ["OrderCreated", "PaymentProcessed"]),
        ("update_inventory", ["InventoryUpdated", "StockReserved"]),
        ("process_shipment", ["OrderShipped", "TrackingCreated"]),
        ("handle_return", ["ReturnRequested", "RefundProcessed"]),
    ]
    
    tenants = ["tenant_retail", "tenant_wholesale", "tenant_marketplace"]
    
    for i in range(20):  # Simulate 20 operations
        tenant_id = random.choice(tenants)
        operation_type, event_types = random.choice(operations)
        
        start_time = time.time()
        
        try:
            # Process multiple events for this operation
            aggregate_id = f"agg-{i:04d}"
            
            for event_type in event_types:
                event_data = {
                    "operation_id": f"op-{i:04d}",
                    "timestamp": datetime.now(timezone.utc).isoformat(),
                    "amount": random.uniform(10.0, 500.0),
                    "customer_id": f"customer-{random.randint(1, 1000)}"
                }
                
                await event_store.create_event_with_metrics(
                    aggregate_id, event_type, event_data, tenant_id
                )
            
            # Load the aggregate to verify
            await event_store.load_aggregate_with_metrics(aggregate_id, tenant_id)
            
            operation_duration = time.time() - start_time
            
            # Record business operation metrics
            metrics.increment_counter(
                "eventuali_business_operations_total",
                {"operation_type": operation_type, "tenant_id": tenant_id, "status": "success"}
            )
            
            print(f"✅ {operation_type} completed for {tenant_id} in {operation_duration*1000:.2f}ms")
            
            # Random delay between operations
            await asyncio.sleep(random.uniform(0.1, 0.3))
            
        except Exception as e:
            metrics.increment_counter(
                "eventuali_business_operations_total", 
                {"operation_type": operation_type, "tenant_id": tenant_id, "status": "error"}
            )
            
            print(f"❌ {operation_type} failed for {tenant_id}: {e}")


async def simulate_system_metrics(metrics: PrometheusMetricsCollector, duration_seconds: int = 30):
    """Simulate system health metrics"""
    print(f"\n⚡ SIMULATING SYSTEM METRICS FOR {duration_seconds} SECONDS")
    print("-" * 60)
    
    start_time = time.time()
    
    while time.time() - start_time < duration_seconds:
        # Simulate memory usage
        memory_usage = random.uniform(100_000_000, 500_000_000)  # 100MB to 500MB
        metrics.set_gauge(
            "eventuali_memory_usage_bytes",
            memory_usage,
            {"component": "event_store"}
        )
        
        # Simulate CPU usage
        cpu_usage = random.uniform(10.0, 80.0)
        metrics.set_gauge(
            "eventuali_cpu_usage_percent",
            cpu_usage,
            {"service": "eventuali"}
        )
        
        # Simulate database connections
        active_connections = random.randint(5, 25)
        metrics.set_gauge(
            "eventuali_database_connections_active",
            active_connections,
            {"backend": "sqlite"}
        )
        
        # Simulate connection creation
        if random.random() < 0.3:  # 30% chance of new connection
            metrics.increment_counter(
                "eventuali_database_connections_total",
                {"backend": "sqlite", "status": "created"}
            )
        
        print(f"📊 System metrics updated: Memory={memory_usage/1000000:.1f}MB, CPU={cpu_usage:.1f}%")
        
        await asyncio.sleep(2)  # Update every 2 seconds


def start_metrics_server(metrics_collector: PrometheusMetricsCollector, port: int = 8000):
    """Start HTTP server for metrics scraping"""
    def create_handler(*args, **kwargs):
        handler = MetricsHTTPHandler(metrics_collector)
        return handler(*args, **kwargs)
    
    server = HTTPServer(('localhost', port), create_handler)
    server.metrics_collector = metrics_collector
    
    def serve_forever_wrapper():
        print(f"🌐 Metrics server started on http://localhost:{port}")
        print(f"📊 Metrics endpoint: http://localhost:{port}/metrics")
        print(f"❤️  Health endpoint: http://localhost:{port}/health")
        server.serve_forever()
    
    thread = threading.Thread(target=serve_forever_wrapper)
    thread.daemon = True
    thread.start()
    
    return server


async def demonstrate_comprehensive_metrics():
    """Demonstrate comprehensive metrics collection"""
    print("=" * 80)
    print("📊 COMPREHENSIVE PROMETHEUS METRICS DEMONSTRATION")
    print("=" * 80)
    
    # Initialize metrics collector
    metrics = PrometheusMetricsCollector("eventuali")
    
    # Start metrics server
    server = start_metrics_server(metrics, 8000)
    
    # Initialize event store with metrics
    event_store = ObservabilityEventStore("sqlite://:memory:", metrics)
    
    try:
        # Run simulations concurrently
        await asyncio.gather(
            simulate_business_operations(event_store, metrics),
            simulate_system_metrics(metrics, 20),
        )
        
        print("\n" + "=" * 60)
        print("📈 FINAL METRICS SUMMARY")
        print("=" * 60)
        
        summary = metrics.get_metrics_summary()
        
        print("\n🔢 COUNTERS:")
        for name, value in summary["counters"].items():
            print(f"  {name}: {value}")
        
        print("\n📏 GAUGES:")
        for name, value in summary["gauges"].items():
            print(f"  {name}: {value:.2f}")
        
        print("\n📈 HISTOGRAMS:")
        for name, stats in summary["histograms"].items():
            print(f"  {name}:")
            print(f"    Count: {stats['count']}")
            print(f"    Sum: {stats['sum']:.4f}")
            print(f"    Avg: {stats['avg']:.4f}")
            print(f"    Min: {stats['min']:.4f}")
            print(f"    Max: {stats['max']:.4f}")
        
        print("\n" + "=" * 60)
        print("🌐 PROMETHEUS METRICS FORMAT SAMPLE")
        print("=" * 60)
        
        prometheus_output = metrics.get_prometheus_format()
        print(prometheus_output[:2000])  # Show first 2000 characters
        if len(prometheus_output) > 2000:
            print("\n... (output truncated)")
        
        print(f"\n📊 Full metrics available at: http://localhost:8000/metrics")
        print("💡 You can scrape these metrics with Prometheus or view them manually")
        
        # Keep server running briefly for testing
        print("\n⏳ Keeping metrics server running for 10 seconds for testing...")
        await asyncio.sleep(10)
        
    finally:
        server.shutdown()
        print("🔴 Metrics server stopped")


async def main():
    """Main demonstration function"""
    print("📊 Eventuali Prometheus Metrics Export Example")
    print("=" * 80)
    print()
    print("This example demonstrates comprehensive metrics collection and export")
    print("in Prometheus format for production monitoring of Eventuali applications.")
    print()
    print("Key concepts demonstrated:")
    print("• Counter metrics for tracking events and operations")
    print("• Gauge metrics for current system state")
    print("• Histogram metrics for timing and size distributions")
    print("• Label-based metric categorization")
    print("• HTTP endpoint for metrics scraping")
    print("• Business and system health metrics")
    print()
    
    try:
        await demonstrate_comprehensive_metrics()
        
        print("\n" + "=" * 80)
        print("🎉 PROMETHEUS METRICS DEMONSTRATION COMPLETED!")
        print("=" * 80)
        print()
        print("Key takeaways:")
        print("• Comprehensive metrics provide deep system visibility")
        print("• Prometheus format enables integration with monitoring stacks")
        print("• Label-based categorization allows flexible querying")
        print("• Histogram metrics reveal performance distributions")
        print("• Counter metrics track business operations over time")
        print("• Gauge metrics show current system state")
        print()
        print("Production deployment considerations:")
        print("• Configure Prometheus to scrape /metrics endpoint")
        print("• Set appropriate scrape intervals (15s-60s)")
        print("• Use metric labels wisely to avoid cardinality explosion")
        print("• Monitor metrics collection overhead")
        print("• Set up alerting rules for key metrics")
        print("• Consider metric retention and storage requirements")
        
    except Exception as e:
        print(f"❌ Error during demonstration: {e}")
        raise


if __name__ == "__main__":
    asyncio.run(main())