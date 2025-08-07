#!/usr/bin/env python3
"""
Production Monitoring Example

This example demonstrates production monitoring patterns:
- System health checks and metrics collection
- Performance monitoring and alerting
- Error tracking and incident management
- SLA monitoring and compliance reporting
- Resource utilization and capacity planning
- Operational dashboards and observability
"""

import asyncio
import sys
import os
from typing import ClassVar, Optional, Dict, List, Any, Callable
from datetime import datetime, timezone, timedelta
from enum import Enum
from dataclasses import dataclass
import json
import uuid
import time
import statistics
import random
from collections import deque, defaultdict

# Add the python package to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'eventuali-python', 'python'))

from eventuali import EventStore
from eventuali.aggregate import User
from eventuali.event import UserRegistered, Event

# System Health Events
class SystemHealthEvent(Event):
    """Base system health event."""
    component: str
    timestamp: str
    severity: str

class HealthCheckPerformed(SystemHealthEvent):
    """Health check performed event."""
    status: str
    response_time_ms: float
    error_message: Optional[str] = None

class MetricCollected(Event):
    """System metric collected event."""
    metric_name: str
    value: float
    unit: str
    tags: Dict[str, str]
    timestamp: str

class AlertTriggered(Event):
    """Alert triggered event."""
    alert_id: str
    alert_name: str
    severity: str
    condition: str
    current_value: float
    threshold: float
    component: str

class AlertResolved(Event):
    """Alert resolved event."""
    alert_id: str
    resolution_time: str
    resolved_by: str
    resolution_notes: str

class IncidentCreated(Event):
    """Incident created event."""
    incident_id: str
    title: str
    description: str
    severity: str
    affected_services: List[str]
    created_by: str

class IncidentUpdated(Event):
    """Incident status updated event."""
    incident_id: str
    status: str
    update_message: str
    updated_by: str

class PerformanceThresholdBreached(Event):
    """Performance threshold breached event."""
    metric_name: str
    current_value: float
    threshold: float
    breach_duration_seconds: int
    impact_level: str

class SLAViolation(Event):
    """SLA violation detected event."""
    sla_name: str
    target_percentage: float
    actual_percentage: float
    violation_duration: str
    customer_impact: str

# Health Check Implementations
class HealthCheck:
    """Base health check interface."""
    
    def __init__(self, name: str, component: str):
        self.name = name
        self.component = component
        self.last_check: Optional[datetime] = None
        self.status = "unknown"
        self.consecutive_failures = 0
    
    async def check(self) -> Dict[str, Any]:
        """Perform health check."""
        start_time = time.perf_counter()
        
        try:
            result = await self._perform_check()
            response_time = (time.perf_counter() - start_time) * 1000
            
            self.status = "healthy"
            self.consecutive_failures = 0
            self.last_check = datetime.now(timezone.utc)
            
            return {
                "name": self.name,
                "component": self.component,
                "status": "healthy",
                "response_time_ms": response_time,
                "timestamp": self.last_check.isoformat(),
                "details": result
            }
            
        except Exception as e:
            response_time = (time.perf_counter() - start_time) * 1000
            self.status = "unhealthy"
            self.consecutive_failures += 1
            self.last_check = datetime.now(timezone.utc)
            
            return {
                "name": self.name,
                "component": self.component,
                "status": "unhealthy",
                "response_time_ms": response_time,
                "timestamp": self.last_check.isoformat(),
                "error": str(e),
                "consecutive_failures": self.consecutive_failures
            }
    
    async def _perform_check(self) -> Dict[str, Any]:
        """Override in subclasses to implement specific health checks."""
        raise NotImplementedError

class DatabaseHealthCheck(HealthCheck):
    """Database connectivity health check."""
    
    def __init__(self, connection_string: str):
        super().__init__("database", "database")
        self.connection_string = connection_string
    
    async def _perform_check(self) -> Dict[str, Any]:
        # Simulate database check
        await asyncio.sleep(random.uniform(0.01, 0.05))
        
        # 5% failure rate
        if random.random() < 0.05:
            raise Exception("Database connection timeout")
        
        return {
            "connection": "active",
            "pool_size": random.randint(5, 20),
            "active_connections": random.randint(1, 10)
        }

class APIHealthCheck(HealthCheck):
    """API endpoint health check."""
    
    def __init__(self, endpoint: str):
        super().__init__(f"api-{endpoint}", "api")
        self.endpoint = endpoint
    
    async def _perform_check(self) -> Dict[str, Any]:
        # Simulate API check
        await asyncio.sleep(random.uniform(0.005, 0.02))
        
        # 3% failure rate
        if random.random() < 0.03:
            raise Exception(f"API endpoint {self.endpoint} returned 500")
        
        return {
            "endpoint": self.endpoint,
            "response_code": 200,
            "latency_p99": random.uniform(50, 200)
        }

class CacheHealthCheck(HealthCheck):
    """Cache system health check."""
    
    def __init__(self, cache_type: str):
        super().__init__(f"cache-{cache_type}", "cache")
        self.cache_type = cache_type
    
    async def _perform_check(self) -> Dict[str, Any]:
        # Simulate cache check
        await asyncio.sleep(random.uniform(0.001, 0.01))
        
        # 2% failure rate
        if random.random() < 0.02:
            raise Exception(f"{self.cache_type} cache not responding")
        
        return {
            "cache_type": self.cache_type,
            "hit_rate": random.uniform(0.8, 0.95),
            "memory_usage": random.uniform(0.6, 0.9)
        }

# Metric Collectors
class MetricCollector:
    """Collects system metrics."""
    
    def __init__(self, name: str, unit: str = "count"):
        self.name = name
        self.unit = unit
        self.values = deque(maxlen=1000)
        self.timestamps = deque(maxlen=1000)
    
    def record(self, value: float, timestamp: str = None) -> MetricCollected:
        """Record a metric value."""
        if timestamp is None:
            timestamp = datetime.now(timezone.utc).isoformat()
        
        self.values.append(value)
        self.timestamps.append(timestamp)
        
        return MetricCollected(
            metric_name=self.name,
            value=value,
            unit=self.unit,
            tags={},
            timestamp=timestamp
        )
    
    def get_stats(self) -> Dict[str, Any]:
        """Get statistical summary."""
        if not self.values:
            return {"count": 0}
        
        values = list(self.values)
        return {
            "count": len(values),
            "current": values[-1],
            "min": min(values),
            "max": max(values),
            "avg": statistics.mean(values),
            "p50": statistics.median(values),
            "p95": statistics.quantiles(values, n=20)[18] if len(values) >= 20 else values[-1],
            "p99": statistics.quantiles(values, n=100)[98] if len(values) >= 100 else values[-1]
        }

# Alert System
class AlertRule:
    """Defines alerting rules."""
    
    def __init__(self, rule_id: str, metric_name: str, condition: str, 
                 threshold: float, severity: str, component: str):
        self.rule_id = rule_id
        self.metric_name = metric_name
        self.condition = condition  # "gt", "lt", "eq"
        self.threshold = threshold
        self.severity = severity
        self.component = component
        self.triggered_count = 0
        self.last_triggered = None
    
    def evaluate(self, value: float) -> bool:
        """Evaluate if alert should trigger."""
        if self.condition == "gt":
            triggered = value > self.threshold
        elif self.condition == "lt":
            triggered = value < self.threshold
        elif self.condition == "eq":
            triggered = value == self.threshold
        else:
            return False
        
        if triggered:
            self.triggered_count += 1
            self.last_triggered = datetime.now(timezone.utc)
        
        return triggered

@dataclass
class ActiveAlert:
    """Represents an active alert."""
    alert_id: str
    rule_id: str
    component: str
    severity: str
    triggered_at: datetime
    current_value: float
    threshold: float
    condition: str
    resolved: bool = False

# SLA Monitor
class SLAMonitor:
    """Monitors service level agreements."""
    
    def __init__(self, sla_name: str, target_percentage: float):
        self.sla_name = sla_name
        self.target_percentage = target_percentage
        self.success_count = 0
        self.total_count = 0
        self.violations: List[Dict[str, Any]] = []
        self.current_period_start = datetime.now(timezone.utc)
    
    def record_event(self, success: bool):
        """Record an SLA event."""
        self.total_count += 1
        if success:
            self.success_count += 1
        
        # Check for violation
        if self.total_count >= 100:  # Check after minimum sample size
            current_percentage = (self.success_count / self.total_count) * 100
            if current_percentage < self.target_percentage:
                self.violations.append({
                    "timestamp": datetime.now(timezone.utc).isoformat(),
                    "target": self.target_percentage,
                    "actual": current_percentage,
                    "sample_size": self.total_count
                })
    
    def get_current_sla(self) -> Dict[str, Any]:
        """Get current SLA status."""
        if self.total_count == 0:
            return {"percentage": 100.0, "status": "healthy"}
        
        percentage = (self.success_count / self.total_count) * 100
        status = "healthy" if percentage >= self.target_percentage else "violated"
        
        return {
            "sla_name": self.sla_name,
            "target_percentage": self.target_percentage,
            "current_percentage": round(percentage, 2),
            "success_count": self.success_count,
            "total_count": self.total_count,
            "status": status,
            "violations": len(self.violations)
        }

# Production Monitor
class ProductionMonitor:
    """Main production monitoring system."""
    
    def __init__(self, event_store: EventStore):
        self.event_store = event_store
        self.health_checks: List[HealthCheck] = []
        self.metrics: Dict[str, MetricCollector] = {}
        self.alert_rules: Dict[str, AlertRule] = {}
        self.active_alerts: Dict[str, ActiveAlert] = {}
        self.sla_monitors: Dict[str, SLAMonitor] = {}
        self.incidents: Dict[str, Dict[str, Any]] = {}
        
        self.monitoring_active = False
        self.check_interval = 5.0  # seconds
        
    def add_health_check(self, health_check: HealthCheck):
        """Add a health check."""
        self.health_checks.append(health_check)
    
    def add_metric(self, name: str, unit: str = "count") -> MetricCollector:
        """Add a metric collector."""
        collector = MetricCollector(name, unit)
        self.metrics[name] = collector
        return collector
    
    def add_alert_rule(self, rule: AlertRule):
        """Add an alert rule."""
        self.alert_rules[rule.rule_id] = rule
    
    def add_sla_monitor(self, monitor: SLAMonitor):
        """Add an SLA monitor."""
        self.sla_monitors[monitor.sla_name] = monitor
    
    async def start_monitoring(self, duration_seconds: int = 30):
        """Start production monitoring."""
        self.monitoring_active = True
        
        # Start background tasks
        tasks = [
            self._health_check_loop(),
            self._metric_collection_loop(),
            self._alert_evaluation_loop()
        ]
        
        monitoring_tasks = [asyncio.create_task(task) for task in tasks]
        
        # Run for specified duration
        await asyncio.sleep(duration_seconds)
        
        # Stop monitoring
        self.monitoring_active = False
        for task in monitoring_tasks:
            task.cancel()
        
        await asyncio.gather(*monitoring_tasks, return_exceptions=True)
    
    async def _health_check_loop(self):
        """Health check monitoring loop."""
        while self.monitoring_active:
            try:
                for health_check in self.health_checks:
                    result = await health_check.check()
                    
                    # Create health check event
                    event = HealthCheckPerformed(
                        component=health_check.component,
                        timestamp=result["timestamp"],
                        severity="error" if result["status"] == "unhealthy" else "info",
                        status=result["status"],
                        response_time_ms=result["response_time_ms"],
                        error_message=result.get("error")
                    )
                    
                    # Record response time metric
                    if health_check.component in self.metrics:
                        self.metrics[health_check.component].record(result["response_time_ms"])
                
                await asyncio.sleep(self.check_interval)
                
            except asyncio.CancelledError:
                break
            except Exception as e:
                print(f"Health check error: {e}")
    
    async def _metric_collection_loop(self):
        """Metric collection loop."""
        system_metrics = ["cpu_usage", "memory_usage", "disk_usage", "network_io"]
        
        while self.monitoring_active:
            try:
                for metric_name in system_metrics:
                    if metric_name in self.metrics:
                        # Simulate metric collection
                        base_values = {
                            "cpu_usage": 45.0,
                            "memory_usage": 65.0,
                            "disk_usage": 40.0,
                            "network_io": 1024.0
                        }
                        
                        base_value = base_values.get(metric_name, 50.0)
                        variation = random.uniform(-10, 15)
                        value = max(0, base_value + variation)
                        
                        self.metrics[metric_name].record(value)
                
                await asyncio.sleep(2.0)
                
            except asyncio.CancelledError:
                break
            except Exception as e:
                print(f"Metric collection error: {e}")
    
    async def _alert_evaluation_loop(self):
        """Alert evaluation loop."""
        while self.monitoring_active:
            try:
                for rule in self.alert_rules.values():
                    if rule.metric_name in self.metrics:
                        collector = self.metrics[rule.metric_name]
                        if collector.values:
                            current_value = collector.values[-1]
                            
                            if rule.evaluate(current_value):
                                alert_id = f"{rule.rule_id}-{int(time.time())}"
                                
                                if alert_id not in self.active_alerts:
                                    # Trigger new alert
                                    alert = ActiveAlert(
                                        alert_id=alert_id,
                                        rule_id=rule.rule_id,
                                        component=rule.component,
                                        severity=rule.severity,
                                        triggered_at=datetime.now(timezone.utc),
                                        current_value=current_value,
                                        threshold=rule.threshold,
                                        condition=rule.condition
                                    )
                                    
                                    self.active_alerts[alert_id] = alert
                
                await asyncio.sleep(3.0)
                
            except asyncio.CancelledError:
                break
            except Exception as e:
                print(f"Alert evaluation error: {e}")
    
    def create_incident(self, title: str, description: str, severity: str, 
                       affected_services: List[str], created_by: str) -> str:
        """Create a new incident."""
        incident_id = f"inc-{uuid.uuid4().hex[:8]}"
        
        incident = {
            "incident_id": incident_id,
            "title": title,
            "description": description,
            "severity": severity,
            "affected_services": affected_services,
            "created_by": created_by,
            "status": "open",
            "created_at": datetime.now(timezone.utc).isoformat(),
            "updates": []
        }
        
        self.incidents[incident_id] = incident
        return incident_id
    
    def update_incident(self, incident_id: str, status: str, update_message: str, updated_by: str):
        """Update incident status."""
        if incident_id in self.incidents:
            incident = self.incidents[incident_id]
            incident["status"] = status
            incident["updated_at"] = datetime.now(timezone.utc).isoformat()
            incident["updates"].append({
                "timestamp": datetime.now(timezone.utc).isoformat(),
                "message": update_message,
                "updated_by": updated_by,
                "status": status
            })
    
    def get_system_health_summary(self) -> Dict[str, Any]:
        """Get overall system health summary."""
        healthy_checks = sum(1 for hc in self.health_checks if hc.status == "healthy")
        total_checks = len(self.health_checks)
        
        active_critical_alerts = sum(1 for alert in self.active_alerts.values() 
                                   if alert.severity == "critical" and not alert.resolved)
        
        open_incidents = sum(1 for inc in self.incidents.values() if inc["status"] == "open")
        
        return {
            "overall_status": "healthy" if healthy_checks == total_checks and active_critical_alerts == 0 else "degraded",
            "health_checks": {
                "healthy": healthy_checks,
                "total": total_checks,
                "success_rate": (healthy_checks / max(total_checks, 1)) * 100
            },
            "alerts": {
                "active": len(self.active_alerts),
                "critical": active_critical_alerts
            },
            "incidents": {
                "open": open_incidents,
                "total": len(self.incidents)
            },
            "last_updated": datetime.now(timezone.utc).isoformat()
        }
    
    def get_performance_report(self) -> Dict[str, Any]:
        """Get system performance report."""
        report = {
            "metrics": {},
            "sla_status": {},
            "performance_summary": {}
        }
        
        # Metrics summary
        for name, collector in self.metrics.items():
            stats = collector.get_stats()
            if stats["count"] > 0:
                report["metrics"][name] = {
                    "current": stats["current"],
                    "avg": round(stats["avg"], 2),
                    "p95": round(stats["p95"], 2),
                    "p99": round(stats["p99"], 2),
                    "unit": collector.unit
                }
        
        # SLA status
        for name, sla in self.sla_monitors.items():
            report["sla_status"][name] = sla.get_current_sla()
        
        # Performance thresholds
        report["performance_summary"] = {
            "response_time_p95": report["metrics"].get("api_response_time", {}).get("p95", 0),
            "error_rate": len([a for a in self.active_alerts.values() if a.severity in ["error", "critical"]]),
            "availability": report["sla_status"].get("api_availability", {}).get("current_percentage", 100)
        }
        
        return report

async def demonstrate_production_monitoring():
    """Demonstrate production monitoring patterns."""
    print("=== Production Monitoring Example ===\n")
    
    event_store = await EventStore.create("sqlite://:memory:")
    monitor = ProductionMonitor(event_store)
    
    print("1. Setting up production monitoring system...")
    
    # Add health checks
    health_checks = [
        DatabaseHealthCheck("sqlite://:memory:"),
        APIHealthCheck("users"),
        APIHealthCheck("orders"),
        APIHealthCheck("payments"),
        CacheHealthCheck("redis"),
        CacheHealthCheck("memcached")
    ]
    
    for hc in health_checks:
        monitor.add_health_check(hc)
        print(f"   ✓ Added health check: {hc.name}")
    
    # Add metrics
    system_metrics = [
        ("cpu_usage", "percent"),
        ("memory_usage", "percent"), 
        ("disk_usage", "percent"),
        ("network_io", "mbps"),
        ("api_response_time", "ms"),
        ("database", "ms"),
        ("cache-redis", "ms"),
        ("cache-memcached", "ms")
    ]
    
    for metric_name, unit in system_metrics:
        monitor.add_metric(metric_name, unit)
        print(f"   ✓ Added metric: {metric_name} ({unit})")
    
    # Add alert rules
    alert_rules = [
        AlertRule("cpu_high", "cpu_usage", "gt", 80.0, "warning", "system"),
        AlertRule("cpu_critical", "cpu_usage", "gt", 95.0, "critical", "system"),
        AlertRule("memory_high", "memory_usage", "gt", 85.0, "warning", "system"),
        AlertRule("disk_full", "disk_usage", "gt", 90.0, "critical", "system"),
        AlertRule("api_slow", "api_response_time", "gt", 500.0, "warning", "api"),
        AlertRule("api_timeout", "api_response_time", "gt", 2000.0, "critical", "api")
    ]
    
    for rule in alert_rules:
        monitor.add_alert_rule(rule)
        print(f"   ✓ Added alert rule: {rule.rule_id}")
    
    # Add SLA monitors
    sla_monitors = [
        SLAMonitor("api_availability", 99.9),
        SLAMonitor("response_time_sla", 95.0),  # 95% of requests under threshold
        SLAMonitor("error_rate_sla", 99.0)      # Less than 1% error rate
    ]
    
    for sla in sla_monitors:
        monitor.add_sla_monitor(sla)
        print(f"   ✓ Added SLA monitor: {sla.sla_name} ({sla.target_percentage}%)")
    
    print(f"   ✓ Production monitoring system initialized")
    print(f"   ✓ Health checks: {len(health_checks)}")
    print(f"   ✓ Metrics: {len(system_metrics)}")
    print(f"   ✓ Alert rules: {len(alert_rules)}")
    print(f"   ✓ SLA monitors: {len(sla_monitors)}")
    
    print("\n2. Starting monitoring simulation...")
    print("   🚀 Running 20-second monitoring simulation...")
    
    # Start monitoring
    monitoring_task = asyncio.create_task(monitor.start_monitoring(20))
    
    # Simulate some incidents during monitoring
    await asyncio.sleep(5)
    
    # Create a test incident
    print("\n   📋 Simulating incident management...")
    incident_id = monitor.create_incident(
        "API Response Time Degradation",
        "Response times for /api/orders endpoint elevated above SLA threshold",
        "medium",
        ["orders-api", "database"],
        "ops-team"
    )
    print(f"      ✓ Created incident: {incident_id}")
    
    await asyncio.sleep(3)
    monitor.update_incident(incident_id, "investigating", "Team investigating database query performance", "alice")
    print(f"      ✓ Updated incident: investigating")
    
    await asyncio.sleep(5)
    monitor.update_incident(incident_id, "resolved", "Database index optimization resolved the issue", "alice")
    print(f"      ✓ Resolved incident")
    
    # Simulate SLA events
    print("\n   📊 Simulating SLA tracking...")
    for sla_name, sla_monitor in monitor.sla_monitors.items():
        for _ in range(150):  # Generate sample events
            success_rate = 0.99 if sla_name == "api_availability" else 0.96
            success = random.random() < success_rate
            sla_monitor.record_event(success)
        print(f"      ✓ Recorded events for {sla_name}")
    
    # Wait for monitoring to complete
    await monitoring_task
    print("   ✅ Monitoring simulation completed")
    
    print("\n3. System health analysis...")
    
    health_summary = monitor.get_system_health_summary()
    print(f"   🏥 System Health Summary:")
    print(f"      Overall Status: {health_summary['overall_status']}")
    print(f"      Health Checks: {health_summary['health_checks']['healthy']}/{health_summary['health_checks']['total']} healthy ({health_summary['health_checks']['success_rate']:.1f}%)")
    print(f"      Active Alerts: {health_summary['alerts']['active']} ({health_summary['alerts']['critical']} critical)")
    print(f"      Open Incidents: {health_summary['incidents']['open']}")
    
    print("\n4. Performance analysis...")
    
    performance_report = monitor.get_performance_report()
    
    print(f"   📈 Key Performance Metrics:")
    for metric_name, stats in performance_report["metrics"].items():
        if stats:
            print(f"      {metric_name}:")
            print(f"        Current: {stats['current']:.1f} {stats['unit']}")
            print(f"        Average: {stats['avg']:.1f} {stats['unit']}")
            print(f"        P95: {stats['p95']:.1f} {stats['unit']}")
            print(f"        P99: {stats['p99']:.1f} {stats['unit']}")
    
    print(f"\n   🎯 SLA Status:")
    for sla_name, sla_status in performance_report["sla_status"].items():
        status_icon = "🟢" if sla_status["status"] == "healthy" else "🔴"
        print(f"      {status_icon} {sla_name}:")
        print(f"         Target: {sla_status['target_percentage']}%")
        print(f"         Current: {sla_status['current_percentage']}%")
        print(f"         Sample: {sla_status['success_count']}/{sla_status['total_count']}")
        print(f"         Violations: {sla_status['violations']}")
    
    print("\n5. Alert analysis...")
    
    print(f"   🚨 Active Alerts:")
    if monitor.active_alerts:
        for alert_id, alert in monitor.active_alerts.items():
            severity_icon = {"critical": "🔴", "warning": "🟠", "info": "🔵"}.get(alert.severity, "⚪")
            print(f"      {severity_icon} {alert_id}:")
            print(f"         Component: {alert.component}")
            print(f"         Condition: {alert.current_value:.1f} {alert.condition} {alert.threshold}")
            print(f"         Triggered: {alert.triggered_at.strftime('%H:%M:%S')}")
    else:
        print("      ✅ No active alerts")
    
    print("\n6. Incident management summary...")
    
    print(f"   📋 Incident Summary:")
    for incident_id, incident in monitor.incidents.items():
        status_icon = {"open": "🔴", "investigating": "🟡", "resolved": "🟢"}.get(incident["status"], "⚪")
        print(f"      {status_icon} {incident_id}: {incident['title']}")
        print(f"         Status: {incident['status']}")
        print(f"         Severity: {incident['severity']}")
        print(f"         Affected: {', '.join(incident['affected_services'])}")
        print(f"         Updates: {len(incident['updates'])}")
    
    print("\n7. Operational dashboard data...")
    
    # Generate operational dashboard
    dashboard_data = {
        "system_health": health_summary,
        "performance": performance_report["performance_summary"],
        "alerts": {
            "total": len(monitor.active_alerts),
            "by_severity": {}
        },
        "incidents": {
            "open": len([i for i in monitor.incidents.values() if i["status"] == "open"]),
            "total": len(monitor.incidents)
        },
        "uptime": "99.95%",  # Simulated
        "last_deployment": "2024-08-07T10:30:00Z"
    }
    
    # Count alerts by severity
    for alert in monitor.active_alerts.values():
        severity = alert.severity
        dashboard_data["alerts"]["by_severity"][severity] = dashboard_data["alerts"]["by_severity"].get(severity, 0) + 1
    
    print(f"   📊 Operational Dashboard:")
    print(f"      System Status: {dashboard_data['system_health']['overall_status']}")
    print(f"      Uptime: {dashboard_data['uptime']}")
    print(f"      Response Time P95: {dashboard_data['performance'].get('response_time_p95', 0):.1f}ms")
    print(f"      Active Alerts: {dashboard_data['alerts']['total']}")
    print(f"      Open Incidents: {dashboard_data['incidents']['open']}")
    print(f"      Last Deployment: {dashboard_data['last_deployment']}")
    
    # Capacity planning insights
    print("\n8. Capacity planning insights...")
    
    cpu_stats = performance_report["metrics"].get("cpu_usage", {})
    memory_stats = performance_report["metrics"].get("memory_usage", {})
    
    print(f"   🔧 Resource Utilization:")
    if cpu_stats:
        print(f"      CPU Usage: {cpu_stats['current']:.1f}% (avg: {cpu_stats['avg']:.1f}%, p95: {cpu_stats['p95']:.1f}%)")
        if cpu_stats['p95'] > 80:
            print(f"         ⚠️  CPU utilization high - consider scaling")
    
    if memory_stats:
        print(f"      Memory Usage: {memory_stats['current']:.1f}% (avg: {memory_stats['avg']:.1f}%, p95: {memory_stats['p95']:.1f}%)")
        if memory_stats['p95'] > 85:
            print(f"         ⚠️  Memory utilization high - monitor closely")
    
    # Health check summary
    healthy_components = len([hc for hc in monitor.health_checks if hc.status == "healthy"])
    total_components = len(monitor.health_checks)
    
    print(f"      Component Health: {healthy_components}/{total_components} healthy")
    if healthy_components < total_components:
        unhealthy = [hc.name for hc in monitor.health_checks if hc.status != "healthy"]
        print(f"         🔴 Unhealthy: {', '.join(unhealthy)}")
    
    return {
        "monitor": monitor,
        "health_summary": health_summary,
        "performance_report": performance_report,
        "active_alerts": len(monitor.active_alerts),
        "incidents": len(monitor.incidents),
        "dashboard_data": dashboard_data,
        "sla_compliance": all(sla["status"] == "healthy" for sla in performance_report["sla_status"].values())
    }

async def main():
    result = await demonstrate_production_monitoring()
    
    print(f"\n✅ SUCCESS! Production monitoring patterns demonstrated!")
    
    print(f"\nProduction monitoring patterns covered:")
    print(f"- ✓ System health checks and automated monitoring")
    print(f"- ✓ Performance metrics collection and analysis") 
    print(f"- ✓ Alert rules and threshold-based notifications")
    print(f"- ✓ SLA monitoring and compliance tracking")
    print(f"- ✓ Incident management and status tracking")
    print(f"- ✓ Operational dashboards and observability")
    print(f"- ✓ Capacity planning and resource analysis")
    
    print(f"\nMonitoring system performance:")
    print(f"- Health checks: {len(result['monitor'].health_checks)}")
    print(f"- Metrics tracked: {len(result['monitor'].metrics)}")
    print(f"- Alert rules: {len(result['monitor'].alert_rules)}")
    print(f"- Active alerts: {result['active_alerts']}")
    print(f"- SLA monitors: {len(result['monitor'].sla_monitors)}")
    print(f"- SLA compliance: {'✅' if result['sla_compliance'] else '❌'}")
    print(f"- System status: {result['health_summary']['overall_status']}")

if __name__ == "__main__":
    asyncio.run(main())