#!/usr/bin/env python3
"""
Example 43: Health Check Endpoints for Production Monitoring

This example demonstrates:
1. Comprehensive health monitoring system for production deployments
2. Component-level health checks (Database, EventStore, Streaming, Security, Tenancy)
3. System-level health metrics (CPU, Memory, Disk, Network)
4. HTTP endpoints for Kubernetes and Docker integration
5. Health check aggregation and scoring
6. Prometheus metrics export for monitoring systems
7. Readiness and liveness probes for container orchestration
8. Production-grade health monitoring patterns

The implementation provides enterprise-ready health monitoring capabilities
with configurable thresholds, background checks, and integration endpoints
for modern container orchestration platforms.
"""

import asyncio
import time
import json
from datetime import datetime
from typing import Dict, List, Optional, Any
import threading
import signal
import sys
from dataclasses import dataclass
from contextlib import asynccontextmanager

import eventuali


@dataclass
class HealthEndpointResponse:
    """Container for HTTP endpoint responses"""
    status_code: int
    content_type: str
    body: str
    
    def format_response(self) -> str:
        """Format as HTTP-like response"""
        return f"HTTP/1.1 {self.status_code}\nContent-Type: {self.content_type}\n\n{self.body}"


class ProductionHealthMonitor:
    """Production-grade health monitoring system with HTTP endpoints"""
    
    def __init__(self, connection_string: str = "sqlite://:memory:", 
                 service_name: str = "eventuali-health-demo", 
                 environment: str = "production"):
        self.connection_string = connection_string
        self.service_name = service_name
        self.environment = environment
        self.is_shutting_down = False
        self.startup_time = datetime.utcnow()
        
        # Create health configuration
        self.health_config = eventuali.HealthConfig(
            check_interval_seconds=15,  # More frequent checks for production
            check_timeout_seconds=3,    # Shorter timeout for responsiveness
            include_system_metrics=True,
            background_checks=True,
            http_port=8080,
            http_bind_address="0.0.0.0",
            service_name=service_name,
            service_version="1.0.0",
            environment=environment
        )
        
        # Initialize health monitoring service
        self.health_service = eventuali.HealthMonitorService(self.health_config)
        self.health_service.with_database(connection_string)
        
        # Initialize HTTP server
        self.http_server = eventuali.HealthHttpServer(self.health_config)
        
        # Track service status
        self.service_status = {
            "started": False,
            "ready": False,
            "healthy": True
        }
    
    async def start_monitoring(self):
        """Start the health monitoring system"""
        print("🏥 Starting production health monitoring system...")
        print(f"   Service: {self.service_name}")
        print(f"   Environment: {self.environment}")
        print(f"   Database: {self.connection_string}")
        print()
        
        try:
            # Start health monitoring service
            self.health_service.start()
            self.service_status["started"] = True
            
            # Start HTTP endpoints
            self.http_server.start_server()
            
            print("✅ Health monitoring system started successfully")
            print()
            
            # Give some time for background checks to initialize
            await asyncio.sleep(2)
            self.service_status["ready"] = True
            
            return True
            
        except Exception as e:
            print(f"❌ Failed to start health monitoring: {e}")
            self.service_status["healthy"] = False
            return False
    
    async def stop_monitoring(self):
        """Gracefully stop the health monitoring system"""
        print("🛑 Shutting down health monitoring system...")
        self.is_shutting_down = True
        self.service_status["ready"] = False
        
        try:
            self.http_server.stop_server()
            self.health_service.stop()
            
            print("✅ Health monitoring system shut down gracefully")
            
        except Exception as e:
            print(f"⚠️  Error during shutdown: {e}")
    
    def get_health_endpoint(self) -> HealthEndpointResponse:
        """GET /health - Comprehensive health report"""
        try:
            health_json = self.http_server.get_health_json()
            health_data = json.loads(health_json)
            
            status_code = 200 if health_data.get("overall_status") in ["Healthy", "Degraded"] else 503
            
            return HealthEndpointResponse(
                status_code=status_code,
                content_type="application/json",
                body=health_json
            )
            
        except Exception as e:
            error_response = {
                "status": "error",
                "message": f"Health check failed: {str(e)}",
                "timestamp": datetime.utcnow().isoformat()
            }
            
            return HealthEndpointResponse(
                status_code=500,
                content_type="application/json",
                body=json.dumps(error_response, indent=2)
            )
    
    def get_ready_endpoint(self) -> HealthEndpointResponse:
        """GET /ready - Kubernetes readiness probe"""
        readiness_json = self.http_server.get_readiness_json()
        readiness_data = json.loads(readiness_json)
        
        return HealthEndpointResponse(
            status_code=readiness_data["http_code"],
            content_type="application/json",
            body=readiness_json
        )
    
    def get_live_endpoint(self) -> HealthEndpointResponse:
        """GET /live - Kubernetes liveness probe"""
        liveness_json = self.http_server.get_liveness_json()
        liveness_data = json.loads(liveness_json)
        
        return HealthEndpointResponse(
            status_code=liveness_data["http_code"],
            content_type="application/json",
            body=liveness_json
        )
    
    def get_metrics_endpoint(self) -> HealthEndpointResponse:
        """GET /metrics - Prometheus metrics export"""
        metrics = self.http_server.get_metrics_prometheus()
        
        return HealthEndpointResponse(
            status_code=200,
            content_type="text/plain; version=0.0.4; charset=utf-8",
            body=metrics
        )
    
    def get_info_endpoint(self) -> HealthEndpointResponse:
        """GET /info - Service information"""
        uptime_seconds = (datetime.utcnow() - self.startup_time).total_seconds()
        
        info = {
            "service": {
                "name": self.service_name,
                "version": "1.0.0",
                "environment": self.environment,
                "uptime_seconds": int(uptime_seconds),
                "startup_time": self.startup_time.isoformat(),
                "status": self.service_status
            },
            "health_config": {
                "check_interval_seconds": self.health_config.check_interval_seconds,
                "http_port": self.health_config.http_port
            },
            "endpoints": {
                "health": "/health",
                "ready": "/ready", 
                "live": "/live",
                "metrics": "/metrics",
                "info": "/info"
            }
        }
        
        return HealthEndpointResponse(
            status_code=200,
            content_type="application/json",
            body=json.dumps(info, indent=2)
        )


class HealthEndpointSimulator:
    """Simulate HTTP server behavior for demonstration"""
    
    def __init__(self, health_monitor: ProductionHealthMonitor):
        self.health_monitor = health_monitor
        self.endpoints = {
            "/health": self.health_monitor.get_health_endpoint,
            "/ready": self.health_monitor.get_ready_endpoint,
            "/live": self.health_monitor.get_live_endpoint,
            "/metrics": self.health_monitor.get_metrics_endpoint,
            "/info": self.health_monitor.get_info_endpoint,
        }
    
    def handle_request(self, path: str) -> HealthEndpointResponse:
        """Simulate handling an HTTP request"""
        if path in self.endpoints:
            return self.endpoints[path]()
        else:
            return HealthEndpointResponse(
                status_code=404,
                content_type="application/json",
                body='{"error": "Not Found", "message": "Endpoint not found"}'
            )
    
    def demonstrate_endpoints(self):
        """Demonstrate all health check endpoints"""
        print("🌐 HEALTH CHECK ENDPOINTS DEMONSTRATION")
        print("=" * 80)
        print()
        
        for path, description in [
            ("/health", "Comprehensive health report"),
            ("/ready", "Kubernetes readiness probe"),
            ("/live", "Kubernetes liveness probe"),
            ("/metrics", "Prometheus metrics export"),
            ("/info", "Service information")
        ]:
            print(f"📡 GET {path} - {description}")
            print("-" * 60)
            
            response = self.handle_request(path)
            print(response.format_response())
            print()


async def demonstrate_health_monitoring_lifecycle():
    """Demonstrate complete health monitoring lifecycle"""
    print("🏥 HEALTH MONITORING LIFECYCLE DEMONSTRATION")
    print("=" * 80)
    print()
    
    # Initialize health monitor
    health_monitor = ProductionHealthMonitor(
        connection_string="sqlite://:memory:",
        service_name="eventuali-production",
        environment="production"
    )
    
    # Start monitoring
    if not await health_monitor.start_monitoring():
        print("❌ Failed to start health monitoring system")
        return
    
    # Wait for system to stabilize
    print("⏳ Waiting for health checks to stabilize...")
    await asyncio.sleep(3)
    
    # Demonstrate endpoints
    simulator = HealthEndpointSimulator(health_monitor)
    simulator.demonstrate_endpoints()
    
    # Demonstrate health report analysis
    await analyze_health_report(health_monitor)
    
    # Demonstrate monitoring over time
    await demonstrate_continuous_monitoring(health_monitor)
    
    # Cleanup
    await health_monitor.stop_monitoring()


async def analyze_health_report(health_monitor: ProductionHealthMonitor):
    """Analyze and explain health report structure"""
    print("📊 HEALTH REPORT ANALYSIS")
    print("=" * 80)
    print()
    
    try:
        # Get latest health report
        report = health_monitor.health_service.get_latest_report()
        
        if not report:
            print("⚠️  No health report available yet")
            return
        
        print(f"🎯 OVERALL HEALTH STATUS: {report.overall_status()}")
        print(f"📈 HEALTH SCORE: {report.overall_score():.1f}/100")
        print(f"⏱️  REPORT TIMESTAMP: {datetime.fromtimestamp(report.timestamp())}")
        print(f"🚀 GENERATION TIME: {report.generation_time_ms()}ms")
        print()
        
        print("🔧 COMPONENT HEALTH CHECKS:")
        print("-" * 40)
        
        for component in report.components():
            status_emoji = {
                eventuali.HealthStatus.Healthy: "✅",
                eventuali.HealthStatus.Degraded: "⚠️ ",
                eventuali.HealthStatus.Unhealthy: "❌",
                eventuali.HealthStatus.Unknown: "❓"
            }.get(component.status(), "❓")
            
            critical_marker = " (CRITICAL)" if component.critical() else ""
            
            print(f"  {status_emoji} {component.component()}{critical_marker}")
            print(f"     Message: {component.message()}")
            print(f"     Duration: {component.duration_ms()}ms")
            
            if component.details():
                print(f"     Details: {component.details()}")
            print()
        
        print("💻 SYSTEM METRICS:")
        print("-" * 20)
        metrics = report.system_metrics()
        
        print(f"  🖥️  CPU Usage: {metrics.cpu_usage_percent():.1f}%")
        print(f"  🧠 Memory Usage: {metrics.memory_usage_percent():.1f}% ({metrics.memory_used_bytes() // (1024*1024)}MB / {metrics.memory_total_bytes() // (1024*1024)}MB)")
        print(f"  💾 Disk Usage: {metrics.disk_usage_percent():.1f}% ({metrics.disk_used_bytes() // (1024*1024*1024)}GB / {metrics.disk_total_bytes() // (1024*1024*1024)}GB)")
        print(f"  🌐 Network: {metrics.network_bytes_received() // (1024*1024)}MB received, {metrics.network_bytes_transmitted() // (1024*1024)}MB transmitted")
        print(f"  🔗 Active Connections: {metrics.active_connections()}")
        print(f"  ⏰ Uptime: {metrics.uptime_seconds()}s")
        print()
        
    except Exception as e:
        print(f"❌ Error analyzing health report: {e}")


async def demonstrate_continuous_monitoring(health_monitor: ProductionHealthMonitor):
    """Show health monitoring over time"""
    print("⏱️  CONTINUOUS HEALTH MONITORING")
    print("=" * 80)
    print()
    
    print("Monitoring health status over 30 seconds (updating every 5 seconds)...")
    print()
    
    for i in range(6):  # Monitor for 30 seconds
        if i > 0:
            await asyncio.sleep(5)
        
        # Get current status
        is_ready = health_monitor.health_service.is_ready()
        is_live = health_monitor.health_service.is_live()
        
        status_summary = health_monitor.health_service.get_health_summary()
        
        timestamp = datetime.now().strftime("%H:%M:%S")
        ready_indicator = "🟢" if is_ready else "🔴"
        live_indicator = "🟢" if is_live else "🔴"
        
        print(f"[{timestamp}] Ready: {ready_indicator} Live: {live_indicator} "
              f"Status: {status_summary.get('status', 'Unknown')} "
              f"Score: {status_summary.get('score', 'N/A')}")
    
    print()
    print("✅ Continuous monitoring demonstration completed")


async def demonstrate_kubernetes_integration():
    """Show Kubernetes/Docker integration patterns"""
    print("☸️  KUBERNETES INTEGRATION PATTERNS")
    print("=" * 80)
    print()
    
    health_monitor = ProductionHealthMonitor(
        service_name="eventuali-k8s-demo",
        environment="kubernetes"
    )
    
    await health_monitor.start_monitoring()
    await asyncio.sleep(2)  # Let it stabilize
    
    print("📋 KUBERNETES DEPLOYMENT CONFIGURATION:")
    print("-" * 40)
    
    # Show example Kubernetes configuration
    k8s_config = f"""
apiVersion: apps/v1
kind: Deployment
metadata:
  name: eventuali-service
  labels:
    app: eventuali
spec:
  replicas: 3
  selector:
    matchLabels:
      app: eventuali
  template:
    metadata:
      labels:
        app: eventuali
    spec:
      containers:
      - name: eventuali
        image: eventuali:latest
        ports:
        - containerPort: {health_monitor.health_config.http_port}
        livenessProbe:
          httpGet:
            path: /live
            port: {health_monitor.health_config.http_port}
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /ready
            port: {health_monitor.health_config.http_port}
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 2
        env:
        - name: HEALTH_CHECK_INTERVAL
          value: "15"
        - name: ENVIRONMENT
          value: "production"
---
apiVersion: v1
kind: Service
metadata:
  name: eventuali-service
spec:
  selector:
    app: eventuali
  ports:
  - protocol: TCP
    port: 80
    targetPort: {health_monitor.health_config.http_port}
  type: ClusterIP
"""
    
    print(k8s_config)
    
    print("🔍 HEALTH PROBE SIMULATION:")
    print("-" * 30)
    
    simulator = HealthEndpointSimulator(health_monitor)
    
    # Simulate Kubernetes probes
    print("Kubernetes liveness probe:")
    liveness_response = simulator.handle_request("/live")
    print(f"  HTTP {liveness_response.status_code}: {json.loads(liveness_response.body)}")
    print()
    
    print("Kubernetes readiness probe:")
    readiness_response = simulator.handle_request("/ready")
    print(f"  HTTP {readiness_response.status_code}: {json.loads(readiness_response.body)}")
    print()
    
    await health_monitor.stop_monitoring()


async def demonstrate_prometheus_integration():
    """Show Prometheus metrics integration"""
    print("📊 PROMETHEUS METRICS INTEGRATION")
    print("=" * 80)
    print()
    
    health_monitor = ProductionHealthMonitor(
        service_name="eventuali-prometheus-demo",
        environment="monitoring"
    )
    
    await health_monitor.start_monitoring()
    await asyncio.sleep(2)
    
    print("📈 PROMETHEUS CONFIGURATION:")
    print("-" * 30)
    
    prometheus_config = f"""
scrape_configs:
  - job_name: 'eventuali-health'
    static_configs:
      - targets: ['localhost:{health_monitor.health_config.http_port}']
    metrics_path: '/metrics'
    scrape_interval: 15s
    scrape_timeout: 10s
    honor_labels: true
"""
    
    print(prometheus_config)
    
    print("📊 EXPORTED METRICS:")
    print("-" * 20)
    
    metrics_response = health_monitor.get_metrics_endpoint()
    metrics_lines = metrics_response.body.split('\n')
    
    # Show key metrics
    for line in metrics_lines:
        if line.startswith('#') or line.startswith('eventuali_'):
            print(f"  {line}")
    
    print()
    
    await health_monitor.stop_monitoring()


async def demonstrate_alerting_scenarios():
    """Demonstrate different health scenarios for alerting"""
    print("🚨 HEALTH ALERTING SCENARIOS")
    print("=" * 80)
    print()
    
    health_monitor = ProductionHealthMonitor(
        service_name="eventuali-alerting-demo",
        environment="testing"
    )
    
    await health_monitor.start_monitoring()
    await asyncio.sleep(2)
    
    print("💚 SCENARIO 1: HEALTHY SYSTEM")
    print("-" * 30)
    
    health_response = health_monitor.get_health_endpoint()
    health_data = json.loads(health_response.body)
    
    print(f"Status Code: {health_response.status_code}")
    print(f"Overall Status: {health_data.get('overall_status')}")
    print(f"Health Score: {health_data.get('overall_score'):.1f}")
    print("✅ All systems operational - No alerts needed")
    print()
    
    print("📊 MONITORING INTEGRATION SUMMARY:")
    print("-" * 35)
    
    integration_summary = {
        "Kubernetes": {
            "Liveness Probe": "/live endpoint",
            "Readiness Probe": "/ready endpoint",
            "Health Checks": "Built-in component monitoring",
            "Graceful Shutdown": "SIGTERM handling"
        },
        "Prometheus": {
            "Metrics Export": "/metrics endpoint",
            "Custom Metrics": "Health scores and component status",
            "Alert Rules": "Based on health thresholds",
            "Scrape Configuration": "Standard metrics format"
        },
        "Grafana": {
            "Dashboards": "Health overview and component details",
            "Alerting": "Multi-channel notifications",
            "Visualization": "Time-series health trends",
            "SLA Tracking": "Availability and performance metrics"
        },
        "Production": {
            "Background Monitoring": "Continuous health checks",
            "Circuit Breakers": "Automatic failure detection",
            "Load Balancing": "Health-aware traffic routing",
            "Auto-scaling": "Health-based scaling decisions"
        }
    }
    
    for system, features in integration_summary.items():
        print(f"🔧 {system}:")
        for feature, description in features.items():
            print(f"   • {feature}: {description}")
        print()
    
    await health_monitor.stop_monitoring()


def setup_graceful_shutdown(health_monitor: ProductionHealthMonitor):
    """Setup graceful shutdown handling"""
    def signal_handler(signum, frame):
        print(f"\n🛑 Received signal {signum}, initiating graceful shutdown...")
        asyncio.create_task(health_monitor.stop_monitoring())
        sys.exit(0)
    
    signal.signal(signal.SIGTERM, signal_handler)
    signal.signal(signal.SIGINT, signal_handler)


async def main():
    """Main demonstration function"""
    print("🏥 Eventuali Health Check Endpoints - Production Monitoring")
    print("=" * 80)
    print()
    print("This example demonstrates comprehensive health monitoring capabilities")
    print("designed for production deployments with Kubernetes, Docker, and")
    print("modern monitoring systems integration.")
    print()
    print("🎯 FEATURES DEMONSTRATED:")
    print("• Component-level health checks (Database, EventStore, Security, etc.)")
    print("• System-level health metrics (CPU, Memory, Disk, Network)")
    print("• HTTP endpoints for container orchestration (/health, /ready, /live)")
    print("• Prometheus metrics export for monitoring systems")
    print("• Health check aggregation and intelligent scoring")
    print("• Production-ready configuration and patterns")
    print("• Kubernetes integration examples")
    print("• Graceful shutdown and error handling")
    print()
    
    try:
        # Demonstrate core health monitoring
        await demonstrate_health_monitoring_lifecycle()
        
        print("\n" + "=" * 80)
        
        # Demonstrate Kubernetes integration
        await demonstrate_kubernetes_integration()
        
        print("\n" + "=" * 80)
        
        # Demonstrate Prometheus integration
        await demonstrate_prometheus_integration()
        
        print("\n" + "=" * 80)
        
        # Demonstrate alerting scenarios
        await demonstrate_alerting_scenarios()
        
        print("🎉 HEALTH CHECK ENDPOINTS DEMONSTRATION COMPLETED!")
        print("=" * 80)
        print()
        print("💡 PRODUCTION DEPLOYMENT RECOMMENDATIONS:")
        print("• Deploy with Kubernetes liveness/readiness probes configured")
        print("• Set up Prometheus scraping of /metrics endpoint")
        print("• Configure Grafana dashboards for health visualization")
        print("• Implement alerting based on health score thresholds")
        print("• Use health-aware load balancing and auto-scaling")
        print("• Monitor health check performance and adjust timeouts")
        print("• Set up log aggregation for health check failures")
        print("• Test failure scenarios in staging environment")
        
    except Exception as e:
        print(f"❌ Error during demonstration: {e}")
        raise


if __name__ == "__main__":
    asyncio.run(main())