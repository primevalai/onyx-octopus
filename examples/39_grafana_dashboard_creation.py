#!/usr/bin/env python3
"""
Example 39: Grafana Dashboard Creation with Real-time Visualization

This example demonstrates:
1. Creating comprehensive Grafana dashboard configurations
2. Real-time visualization of Eventuali metrics
3. Custom dashboard panels for event sourcing operations
4. Alerting rules and threshold monitoring
5. Integration with Prometheus metrics from Example 38
6. Dashboard templating and variables

The implementation shows how to programmatically create Grafana dashboards
that provide deep insights into event sourcing system performance.
"""

import asyncio
import json
import time
import random
from datetime import datetime, timezone
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, field, asdict
from pathlib import Path
import tempfile
import subprocess

import eventuali


@dataclass
class GrafanaDashboardPanel:
    """Represents a Grafana dashboard panel configuration"""
    id: int
    title: str
    type: str  # graph, singlestat, table, etc.
    targets: List[Dict[str, Any]]
    gridPos: Dict[str, int]
    options: Dict[str, Any] = field(default_factory=dict)
    fieldConfig: Dict[str, Any] = field(default_factory=dict)
    description: str = ""
    thresholds: List[Dict[str, Any]] = field(default_factory=list)


@dataclass 
class GrafanaDashboard:
    """Complete Grafana dashboard configuration"""
    id: Optional[int]
    uid: str
    title: str
    description: str
    tags: List[str]
    panels: List[GrafanaDashboardPanel]
    templating: Dict[str, Any] = field(default_factory=dict)
    time: Dict[str, str] = field(default_factory=lambda: {"from": "now-1h", "to": "now"})
    refresh: str = "30s"
    schemaVersion: int = 39
    version: int = 1


class GrafanaDashboardGenerator:
    """Generator for Eventuali-specific Grafana dashboards"""
    
    def __init__(self, prometheus_url: str = "http://localhost:9090"):
        self.prometheus_url = prometheus_url
        self.panel_id_counter = 1
        
    def _get_next_panel_id(self) -> int:
        """Get the next available panel ID"""
        panel_id = self.panel_id_counter
        self.panel_id_counter += 1
        return panel_id
        
    def create_event_metrics_panel(self, x: int = 0, y: int = 0, w: int = 12, h: int = 8) -> GrafanaDashboardPanel:
        """Create panel for event creation and processing metrics"""
        return GrafanaDashboardPanel(
            id=self._get_next_panel_id(),
            title="Event Processing Metrics",
            type="timeseries",
            description="Real-time metrics for event creation, storage, and loading operations",
            gridPos={"x": x, "y": y, "w": w, "h": h},
            targets=[
                {
                    "expr": "rate(eventuali_events_created_total[5m])",
                    "interval": "",
                    "legendFormat": "Events Created/sec - {{event_type}}",
                    "refId": "A"
                },
                {
                    "expr": "rate(eventuali_events_stored_total{status=\"success\"}[5m])",
                    "interval": "",
                    "legendFormat": "Events Stored/sec - {{backend_type}}",
                    "refId": "B"
                },
                {
                    "expr": "rate(eventuali_events_loaded_total[5m])",
                    "interval": "",
                    "legendFormat": "Events Loaded/sec - {{aggregate_type}}",
                    "refId": "C"
                }
            ],
            fieldConfig={
                "defaults": {
                    "color": {"mode": "palette-classic"},
                    "custom": {
                        "axisLabel": "",
                        "axisPlacement": "auto",
                        "barAlignment": 0,
                        "drawStyle": "line",
                        "fillOpacity": 10,
                        "gradientMode": "none",
                        "hideFrom": {"legend": False, "tooltip": False, "vis": False},
                        "lineInterpolation": "linear",
                        "lineWidth": 1,
                        "pointSize": 5,
                        "scaleDistribution": {"type": "linear"},
                        "showPoints": "never",
                        "spanNulls": False,
                        "stacking": {"group": "A", "mode": "none"},
                        "thresholdsStyle": {"mode": "off"}
                    },
                    "mappings": [],
                    "min": 0,
                    "unit": "ops"
                }
            },
            options={
                "legend": {"calcs": [], "displayMode": "list", "placement": "bottom"},
                "tooltip": {"mode": "single", "sort": "none"}
            }
        )
    
    def create_performance_panel(self, x: int = 0, y: int = 0, w: int = 12, h: int = 8) -> GrafanaDashboardPanel:
        """Create panel for performance and latency metrics"""
        return GrafanaDashboardPanel(
            id=self._get_next_panel_id(),
            title="Event Processing Performance",
            type="timeseries",
            description="Latency distribution and processing times for event operations",
            gridPos={"x": x, "y": y, "w": w, "h": h},
            targets=[
                {
                    "expr": "histogram_quantile(0.95, rate(eventuali_event_processing_duration_seconds_bucket[5m]))",
                    "interval": "",
                    "legendFormat": "95th Percentile - {{operation_type}}",
                    "refId": "A"
                },
                {
                    "expr": "histogram_quantile(0.50, rate(eventuali_event_processing_duration_seconds_bucket[5m]))",
                    "interval": "",
                    "legendFormat": "50th Percentile - {{operation_type}}",
                    "refId": "B"
                },
                {
                    "expr": "rate(eventuali_event_processing_duration_seconds_sum[5m]) / rate(eventuali_event_processing_duration_seconds_count[5m])",
                    "interval": "",
                    "legendFormat": "Average - {{operation_type}}",
                    "refId": "C"
                }
            ],
            fieldConfig={
                "defaults": {
                    "color": {"mode": "palette-classic"},
                    "custom": {
                        "axisLabel": "Duration",
                        "axisPlacement": "auto",
                        "drawStyle": "line",
                        "fillOpacity": 10,
                        "lineWidth": 2,
                        "pointSize": 5,
                        "showPoints": "never"
                    },
                    "min": 0,
                    "unit": "s"
                }
            },
            thresholds=[
                {"color": "green", "value": None},
                {"color": "yellow", "value": 0.1},
                {"color": "red", "value": 0.5}
            ]
        )
    
    def create_database_panel(self, x: int = 0, y: int = 0, w: int = 12, h: int = 8) -> GrafanaDashboardPanel:
        """Create panel for database performance metrics"""
        return GrafanaDashboardPanel(
            id=self._get_next_panel_id(),
            title="Database Performance",
            type="timeseries",
            description="Database query performance and connection metrics",
            gridPos={"x": x, "y": y, "w": w, "h": h},
            targets=[
                {
                    "expr": "histogram_quantile(0.95, rate(eventuali_database_query_duration_seconds_bucket[5m]))",
                    "interval": "",
                    "legendFormat": "Query Time 95th - {{query_type}}",
                    "refId": "A"
                },
                {
                    "expr": "eventuali_database_connections_active",
                    "interval": "",
                    "legendFormat": "Active Connections - {{backend}}",
                    "refId": "B"
                },
                {
                    "expr": "rate(eventuali_database_connections_total[5m])",
                    "interval": "",
                    "legendFormat": "New Connections/sec - {{backend}}",
                    "refId": "C"
                }
            ],
            fieldConfig={
                "defaults": {
                    "color": {"mode": "palette-classic"},
                    "custom": {
                        "axisLabel": "",
                        "drawStyle": "line",
                        "fillOpacity": 15,
                        "lineWidth": 1
                    },
                    "unit": "short"
                }
            }
        )
    
    def create_business_metrics_panel(self, x: int = 0, y: int = 0, w: int = 12, h: int = 8) -> GrafanaDashboardPanel:
        """Create panel for business operation metrics"""
        return GrafanaDashboardPanel(
            id=self._get_next_panel_id(),
            title="Business Operations",
            type="timeseries",
            description="Business-level metrics and operation success rates",
            gridPos={"x": x, "y": y, "w": w, "h": h},
            targets=[
                {
                    "expr": "rate(eventuali_business_operations_total{status=\"success\"}[5m])",
                    "interval": "",
                    "legendFormat": "Successful Ops/sec - {{operation_type}}",
                    "refId": "A"
                },
                {
                    "expr": "rate(eventuali_business_operations_total{status=\"error\"}[5m])",
                    "interval": "",
                    "legendFormat": "Failed Ops/sec - {{operation_type}}",
                    "refId": "B"
                },
                {
                    "expr": "eventuali_active_aggregates",
                    "interval": "",
                    "legendFormat": "Active Aggregates - {{aggregate_type}}",
                    "refId": "C"
                }
            ],
            fieldConfig={
                "defaults": {
                    "color": {"mode": "palette-classic"},
                    "unit": "short"
                }
            }
        )
    
    def create_error_panel(self, x: int = 0, y: int = 0, w: int = 12, h: int = 6) -> GrafanaDashboardPanel:
        """Create panel for error tracking"""
        return GrafanaDashboardPanel(
            id=self._get_next_panel_id(),
            title="Error Rates",
            type="stat",
            description="Error rates and types across all operations",
            gridPos={"x": x, "y": y, "w": w, "h": h},
            targets=[
                {
                    "expr": "rate(eventuali_errors_total[5m])",
                    "interval": "",
                    "legendFormat": "Errors/sec - {{error_type}}",
                    "refId": "A"
                }
            ],
            fieldConfig={
                "defaults": {
                    "color": {"mode": "thresholds"},
                    "custom": {
                        "align": "auto",
                        "displayMode": "list",
                        "inspect": False
                    },
                    "mappings": [],
                    "max": 10,
                    "min": 0,
                    "unit": "ops"
                }
            },
            options={
                "reduceOptions": {
                    "calcs": ["lastNotNull"],
                    "fields": "",
                    "values": False
                },
                "text": {},
                "textMode": "auto"
            },
            thresholds=[
                {"color": "green", "value": None},
                {"color": "yellow", "value": 1},
                {"color": "red", "value": 5}
            ]
        )
    
    def create_system_health_panel(self, x: int = 0, y: int = 0, w: int = 12, h: int = 8) -> GrafanaDashboardPanel:
        """Create panel for system health metrics"""
        return GrafanaDashboardPanel(
            id=self._get_next_panel_id(),
            title="System Health",
            type="timeseries",
            description="Memory usage, CPU usage, and throughput metrics",
            gridPos={"x": x, "y": y, "w": w, "h": h},
            targets=[
                {
                    "expr": "eventuali_memory_usage_bytes / 1024 / 1024",
                    "interval": "",
                    "legendFormat": "Memory Usage MB - {{component}}",
                    "refId": "A"
                },
                {
                    "expr": "eventuali_cpu_usage_percent",
                    "interval": "",
                    "legendFormat": "CPU Usage % - {{service}}",
                    "refId": "B"
                },
                {
                    "expr": "eventuali_throughput_events_per_second",
                    "interval": "",
                    "legendFormat": "Throughput EPS - {{operation}}",
                    "refId": "C"
                }
            ],
            fieldConfig={
                "defaults": {
                    "color": {"mode": "palette-classic"},
                    "custom": {
                        "drawStyle": "line",
                        "fillOpacity": 10,
                        "lineWidth": 1
                    },
                    "unit": "short"
                }
            }
        )
    
    def create_aggregates_heatmap_panel(self, x: int = 0, y: int = 0, w: int = 12, h: int = 8) -> GrafanaDashboardPanel:
        """Create heatmap panel for aggregate load times"""
        return GrafanaDashboardPanel(
            id=self._get_next_panel_id(),
            title="Aggregate Load Time Distribution",
            type="heatmap",
            description="Heatmap showing distribution of aggregate loading times by event count",
            gridPos={"x": x, "y": y, "w": w, "h": h},
            targets=[
                {
                    "expr": "rate(eventuali_aggregate_load_duration_seconds_bucket[5m])",
                    "interval": "",
                    "legendFormat": "{{le}}",
                    "refId": "A"
                }
            ],
            fieldConfig={
                "defaults": {
                    "custom": {
                        "hideFrom": {"legend": False, "tooltip": False, "vis": False},
                        "scaleDistribution": {"type": "linear"}
                    },
                    "unit": "s"
                }
            },
            options={
                "calculate": True,
                "cellGap": 2,
                "cellValues": {},
                "color": {
                    "exponent": 0.5,
                    "fill": "dark-orange",
                    "reverse": False,
                    "scale": "exponential",
                    "scheme": "Oranges"
                },
                "exemplars": {"color": "rgba(255,0,255,0.7)"},
                "filterValues": {"le": 1e-9},
                "legend": {"show": False},
                "rowsFrame": {"layout": "auto"},
                "tooltip": {"show": True, "yHistogram": False},
                "yAxis": {
                    "axisPlacement": "left",
                    "reverse": False,
                    "unit": "s"
                }
            }
        )
    
    def create_eventuali_dashboard(self) -> GrafanaDashboard:
        """Create the complete Eventuali monitoring dashboard"""
        
        # Dashboard template variables
        templating = {
            "list": [
                {
                    "current": {"selected": True, "text": "All", "value": "$__all"},
                    "datasource": {"type": "prometheus", "uid": "${DS_PROMETHEUS}"},
                    "definition": "label_values(eventuali_events_created_total, tenant_id)",
                    "hide": 0,
                    "includeAll": True,
                    "label": "Tenant",
                    "multi": True,
                    "name": "tenant",
                    "options": [],
                    "query": {
                        "query": "label_values(eventuali_events_created_total, tenant_id)",
                        "refId": "StandardVariableQuery"
                    },
                    "refresh": 1,
                    "regex": "",
                    "skipUrlSync": False,
                    "sort": 0,
                    "type": "query"
                },
                {
                    "current": {"selected": True, "text": "All", "value": "$__all"},
                    "datasource": {"type": "prometheus", "uid": "${DS_PROMETHEUS}"},
                    "definition": "label_values(eventuali_events_created_total, aggregate_type)",
                    "hide": 0,
                    "includeAll": True,
                    "label": "Aggregate Type",
                    "multi": True,
                    "name": "aggregate_type",
                    "options": [],
                    "query": {
                        "query": "label_values(eventuali_events_created_total, aggregate_type)",
                        "refId": "StandardVariableQuery"
                    },
                    "refresh": 1,
                    "regex": "",
                    "skipUrlSync": False,
                    "sort": 0,
                    "type": "query"
                }
            ]
        }
        
        # Create all panels with appropriate positioning
        panels = [
            # Row 1: Event metrics and performance
            self.create_event_metrics_panel(x=0, y=0, w=12, h=8),
            self.create_performance_panel(x=12, y=0, w=12, h=8),
            
            # Row 2: Database and business metrics
            self.create_database_panel(x=0, y=8, w=12, h=8),
            self.create_business_metrics_panel(x=12, y=8, w=12, h=8),
            
            # Row 3: Error tracking and system health
            self.create_error_panel(x=0, y=16, w=6, h=6),
            self.create_system_health_panel(x=6, y=16, w=18, h=8),
            
            # Row 4: Heatmap
            self.create_aggregates_heatmap_panel(x=0, y=24, w=24, h=8),
        ]
        
        return GrafanaDashboard(
            id=None,
            uid="eventuali-monitoring",
            title="Eventuali Event Sourcing Monitoring",
            description="Comprehensive monitoring dashboard for Eventuali event sourcing operations",
            tags=["eventuali", "event-sourcing", "performance", "monitoring"],
            panels=panels,
            templating=templating,
            time={"from": "now-1h", "to": "now"},
            refresh="30s"
        )
    
    def create_alerting_rules(self) -> List[Dict[str, Any]]:
        """Create Prometheus alerting rules for Eventuali metrics"""
        return [
            {
                "alert": "EventualiHighErrorRate",
                "expr": "rate(eventuali_errors_total[5m]) > 5",
                "for": "2m",
                "labels": {
                    "severity": "warning",
                    "service": "eventuali"
                },
                "annotations": {
                    "summary": "High error rate in Eventuali",
                    "description": "Error rate is {{ $value }} errors per second for {{ $labels.operation }}"
                }
            },
            {
                "alert": "EventualiHighLatency",
                "expr": "histogram_quantile(0.95, rate(eventuali_event_processing_duration_seconds_bucket[5m])) > 0.5",
                "for": "5m",
                "labels": {
                    "severity": "warning",
                    "service": "eventuali"
                },
                "annotations": {
                    "summary": "High latency in event processing",
                    "description": "95th percentile latency is {{ $value }}s for {{ $labels.operation_type }}"
                }
            },
            {
                "alert": "EventualiLowThroughput",
                "expr": "eventuali_throughput_events_per_second < 100",
                "for": "5m",
                "labels": {
                    "severity": "warning",
                    "service": "eventuali"
                },
                "annotations": {
                    "summary": "Low throughput in Eventuali",
                    "description": "Throughput is {{ $value }} events per second for {{ $labels.operation }}"
                }
            },
            {
                "alert": "EventualiHighMemoryUsage",
                "expr": "eventuali_memory_usage_bytes / 1024 / 1024 > 1000",
                "for": "10m",
                "labels": {
                    "severity": "critical",
                    "service": "eventuali"
                },
                "annotations": {
                    "summary": "High memory usage in Eventuali",
                    "description": "Memory usage is {{ $value }}MB for {{ $labels.component }}"
                }
            }
        ]


class GrafanaDashboardManager:
    """Manager for deploying and managing Grafana dashboards"""
    
    def __init__(self, grafana_url: str = "http://localhost:3000", 
                 api_key: Optional[str] = None):
        self.grafana_url = grafana_url
        self.api_key = api_key
        self.headers = {}
        if api_key:
            self.headers["Authorization"] = f"Bearer {api_key}"
        
    def deploy_dashboard(self, dashboard: GrafanaDashboard) -> Dict[str, Any]:
        """Deploy a dashboard to Grafana"""
        dashboard_json = {
            "dashboard": asdict(dashboard),
            "folderId": 0,
            "overwrite": True
        }
        
        # For this demo, we'll save to file since we don't have a real Grafana instance
        return self._save_dashboard_to_file(dashboard)
    
    def _save_dashboard_to_file(self, dashboard: GrafanaDashboard) -> Dict[str, Any]:
        """Save dashboard configuration to JSON file for manual import"""
        dashboard_dict = asdict(dashboard)
        
        # Create a temporary directory for dashboard files
        dashboard_dir = Path("/tmp/eventuali_dashboards")
        dashboard_dir.mkdir(exist_ok=True)
        
        dashboard_file = dashboard_dir / f"{dashboard.uid}.json"
        
        with open(dashboard_file, 'w') as f:
            json.dump(dashboard_dict, f, indent=2)
        
        return {
            "status": "success",
            "message": f"Dashboard saved to {dashboard_file}",
            "file_path": str(dashboard_file),
            "dashboard_uid": dashboard.uid
        }
    
    def create_prometheus_config(self) -> str:
        """Create Prometheus configuration for scraping Eventuali metrics"""
        config = """
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "eventuali_alerts.yml"

scrape_configs:
  - job_name: 'eventuali'
    static_configs:
      - targets: ['localhost:8000']
    metrics_path: '/metrics'
    scrape_interval: 15s
    scrape_timeout: 10s
    honor_labels: true

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - localhost:9093
"""
        return config
    
    def create_docker_compose_stack(self) -> str:
        """Create Docker Compose configuration for complete monitoring stack"""
        compose_config = """
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:latest
    container_name: eventuali-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - ./eventuali_alerts.yml:/etc/prometheus/eventuali_alerts.yml
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--storage.tsdb.retention.time=200h'
      - '--web.enable-lifecycle'

  grafana:
    image: grafana/grafana:latest
    container_name: eventuali-grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_USERS_ALLOW_SIGN_UP=false
    volumes:
      - grafana-storage:/var/lib/grafana
      - ./dashboards:/etc/grafana/provisioning/dashboards
      - ./datasources.yml:/etc/grafana/provisioning/datasources/datasources.yml

  alertmanager:
    image: prom/alertmanager:latest
    container_name: eventuali-alertmanager
    ports:
      - "9093:9093"
    volumes:
      - ./alertmanager.yml:/etc/alertmanager/alertmanager.yml

volumes:
  grafana-storage:
"""
        return compose_config


async def demonstrate_dashboard_creation():
    """Demonstrate creating comprehensive Grafana dashboards"""
    print("=" * 80)
    print("📊 GRAFANA DASHBOARD CREATION DEMONSTRATION")
    print("=" * 80)
    print()
    
    # Initialize dashboard generator
    generator = GrafanaDashboardGenerator()
    manager = GrafanaDashboardManager()
    
    print("🏗️  Creating comprehensive Eventuali monitoring dashboard...")
    
    # Create the main dashboard
    dashboard = generator.create_eventuali_dashboard()
    
    print(f"✅ Created dashboard: {dashboard.title}")
    print(f"   UID: {dashboard.uid}")
    print(f"   Panels: {len(dashboard.panels)}")
    print(f"   Tags: {', '.join(dashboard.tags)}")
    print()
    
    # Deploy dashboard (save to file for this demo)
    result = manager.deploy_dashboard(dashboard)
    
    print("💾 Dashboard deployment result:")
    print(f"   Status: {result['status']}")
    print(f"   File: {result['file_path']}")
    print()
    
    # Create alerting rules
    print("🚨 Creating alerting rules...")
    alerts = generator.create_alerting_rules()
    
    alerts_file = Path("/tmp/eventuali_dashboards/eventuali_alerts.yml")
    alerts_config = {
        "groups": [
            {
                "name": "eventuali-alerts",
                "rules": alerts
            }
        ]
    }
    
    # Simple YAML-like format for alerts (without yaml dependency)
    with open(alerts_file, 'w') as f:
        f.write("groups:\n")
        f.write("  - name: eventuali-alerts\n")
        f.write("    rules:\n")
        for alert in alerts:
            f.write(f"      - alert: {alert['alert']}\n")
            f.write(f"        expr: {alert['expr']}\n")
            f.write(f"        for: {alert['for']}\n")
            f.write("        labels:\n")
            for key, value in alert['labels'].items():
                f.write(f"          {key}: {value}\n")
            f.write("        annotations:\n")
            for key, value in alert['annotations'].items():
                f.write(f"          {key}: \"{value}\"\n")
            f.write("\n")
    
    print(f"✅ Created {len(alerts)} alerting rules")
    print(f"   File: {alerts_file}")
    print()
    
    # Create monitoring stack configuration
    print("🐳 Creating monitoring stack configuration...")
    
    config_dir = Path("/tmp/eventuali_dashboards")
    
    # Prometheus config
    prometheus_config = manager.create_prometheus_config()
    with open(config_dir / "prometheus.yml", 'w') as f:
        f.write(prometheus_config)
    
    # Docker Compose
    compose_config = manager.create_docker_compose_stack()
    with open(config_dir / "docker-compose.yml", 'w') as f:
        f.write(compose_config)
    
    # Grafana datasource config
    datasources_config = """
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
"""
    with open(config_dir / "datasources.yml", 'w') as f:
        f.write(datasources_config)
    
    print("✅ Created complete monitoring stack configuration")
    print(f"   Directory: {config_dir}")
    
    return dashboard, alerts, str(config_dir)


async def demonstrate_real_time_visualization():
    """Demonstrate real-time visualization concepts"""
    print("\n" + "=" * 80)
    print("📈 REAL-TIME VISUALIZATION DEMONSTRATION")
    print("=" * 80)
    print()
    
    print("🔄 Simulating real-time dashboard updates...")
    
    # Simulate metrics that would be displayed in real-time
    metrics_simulation = {
        "events_created_per_second": [],
        "average_latency_ms": [],
        "error_rate": [],
        "memory_usage_mb": [],
        "active_connections": []
    }
    
    # Generate 60 seconds of simulated data
    for second in range(60):
        timestamp = time.time() + second
        
        # Simulate varying metrics
        base_events = 1000
        events_variation = random.uniform(0.8, 1.2)
        events_per_sec = base_events * events_variation
        
        base_latency = 25  # 25ms baseline
        latency_variation = random.uniform(0.5, 2.0)
        if second > 40:  # Simulate performance degradation
            latency_variation *= 1.5
        avg_latency = base_latency * latency_variation
        
        error_rate = random.uniform(0, 5)
        if second > 45:  # Simulate error spike
            error_rate *= 3
        
        memory_usage = random.uniform(200, 600)
        active_connections = random.randint(10, 50)
        
        metrics_simulation["events_created_per_second"].append({
            "timestamp": timestamp,
            "value": events_per_sec
        })
        metrics_simulation["average_latency_ms"].append({
            "timestamp": timestamp,
            "value": avg_latency
        })
        metrics_simulation["error_rate"].append({
            "timestamp": timestamp,
            "value": error_rate
        })
        metrics_simulation["memory_usage_mb"].append({
            "timestamp": timestamp,
            "value": memory_usage
        })
        metrics_simulation["active_connections"].append({
            "timestamp": timestamp,
            "value": active_connections
        })
        
        # Show live updates every 10 seconds
        if second % 10 == 0:
            print(f"📊 Time {second:02d}s: Events={events_per_sec:.0f}/s, "
                  f"Latency={avg_latency:.1f}ms, Errors={error_rate:.1f}/s, "
                  f"Memory={memory_usage:.0f}MB")
    
    print("✅ Generated 60 seconds of simulated metrics data")
    
    # Show summary statistics
    print("\n📈 VISUALIZATION SUMMARY:")
    print("-" * 40)
    
    events_values = [m["value"] for m in metrics_simulation["events_created_per_second"]]
    latency_values = [m["value"] for m in metrics_simulation["average_latency_ms"]]
    error_values = [m["value"] for m in metrics_simulation["error_rate"]]
    
    print(f"Events/sec    - Min: {min(events_values):.0f}, Max: {max(events_values):.0f}, Avg: {sum(events_values)/len(events_values):.0f}")
    print(f"Latency (ms)  - Min: {min(latency_values):.1f}, Max: {max(latency_values):.1f}, Avg: {sum(latency_values)/len(latency_values):.1f}")
    print(f"Error Rate    - Min: {min(error_values):.1f}, Max: {max(error_values):.1f}, Avg: {sum(error_values)/len(error_values):.1f}")
    
    # Identify anomalies that would trigger alerts
    high_latency_points = [v for v in latency_values if v > 50]
    high_error_points = [v for v in error_values if v > 10]
    
    print(f"\n🚨 ANOMALY DETECTION:")
    print(f"High latency events: {len(high_latency_points)} (>{50}ms)")
    print(f"High error rate events: {len(high_error_points)} (>{10}/s)")
    
    if high_latency_points:
        print("⚠️  Performance degradation detected - would trigger alert")
    if high_error_points:
        print("🔥 Error spike detected - would trigger critical alert")
    
    return metrics_simulation


async def demonstrate_dashboard_features():
    """Demonstrate advanced dashboard features"""
    print("\n" + "=" * 80)
    print("🎛️  ADVANCED DASHBOARD FEATURES DEMONSTRATION")
    print("=" * 80)
    print()
    
    print("✨ Advanced features demonstrated in the dashboard:")
    print()
    
    features = [
        {
            "name": "Template Variables",
            "description": "Dynamic filtering by tenant and aggregate type",
            "benefit": "Allows drill-down into specific segments"
        },
        {
            "name": "Multi-Axis Visualization", 
            "description": "Combine events/sec, latency, and error rates in single view",
            "benefit": "Correlate different metrics to identify patterns"
        },
        {
            "name": "Heatmap Analysis",
            "description": "Aggregate load time distribution by event count buckets",
            "benefit": "Identify performance hotspots and outliers"
        },
        {
            "name": "Threshold Alerting",
            "description": "Color-coded thresholds for critical metrics",
            "benefit": "Immediate visual indication of problems"
        },
        {
            "name": "Statistical Aggregations",
            "description": "95th percentile, median, and average latencies",
            "benefit": "Understand complete performance distribution"
        },
        {
            "name": "Rate Calculations",
            "description": "Per-second rates from cumulative counters",
            "benefit": "Show instantaneous system activity"
        },
        {
            "name": "Time-based Correlation",
            "description": "Align all metrics on common time axis",
            "benefit": "Identify cause-effect relationships"
        },
        {
            "name": "Multi-Tenant Views",
            "description": "Separate metrics by tenant for SaaS deployments",
            "benefit": "Isolate performance issues by customer"
        }
    ]
    
    for i, feature in enumerate(features, 1):
        print(f"{i}. {feature['name']}")
        print(f"   Description: {feature['description']}")
        print(f"   Benefit: {feature['benefit']}")
        print()
    
    # Demonstrate query optimization
    print("🔍 QUERY OPTIMIZATION EXAMPLES:")
    print("-" * 50)
    
    optimized_queries = [
        {
            "metric": "Event Processing Rate",
            "basic": "eventuali_events_created_total",
            "optimized": "rate(eventuali_events_created_total[5m])",
            "reason": "Rate calculation provides per-second values instead of cumulative"
        },
        {
            "metric": "Latency Percentiles",
            "basic": "eventuali_event_processing_duration_seconds_bucket",
            "optimized": "histogram_quantile(0.95, rate(eventuali_event_processing_duration_seconds_bucket[5m]))",
            "reason": "Quantile calculation from histogram buckets shows distribution"
        },
        {
            "metric": "Error Percentage",
            "basic": "eventuali_errors_total",
            "optimized": "rate(eventuali_errors_total[5m]) / rate(eventuali_events_created_total[5m]) * 100",
            "reason": "Percentage calculation provides meaningful context"
        }
    ]
    
    for query in optimized_queries:
        print(f"📊 {query['metric']}")
        print(f"   Basic:     {query['basic']}")
        print(f"   Optimized: {query['optimized']}")
        print(f"   Why:       {query['reason']}")
        print()
    
    print("💡 Dashboard Best Practices Implemented:")
    practices = [
        "Use appropriate visualization types for data (time series, stat, heatmap)",
        "Implement consistent color schemes and legends",
        "Provide meaningful descriptions and context",
        "Use template variables for dynamic filtering",
        "Set appropriate time ranges and refresh intervals",
        "Configure reasonable thresholds and alerts",
        "Group related metrics in logical panel arrangements",
        "Optimize queries for performance and accuracy"
    ]
    
    for i, practice in enumerate(practices, 1):
        print(f"  {i}. {practice}")
    
    return features, optimized_queries


async def main():
    """Main demonstration function"""
    print("📊 Eventuali Grafana Dashboard Creation Example")
    print("=" * 80)
    print()
    print("This example demonstrates comprehensive Grafana dashboard creation")
    print("for monitoring Eventuali event sourcing applications.")
    print()
    print("Key concepts demonstrated:")
    print("• Complete dashboard configuration and panel creation")
    print("• Real-time visualization of event sourcing metrics")
    print("• Advanced features like heatmaps and template variables")
    print("• Alerting rules and threshold monitoring")
    print("• Integration with Prometheus metrics from Example 38")
    print("• Production monitoring stack deployment")
    print()
    
    try:
        # Run all demonstrations
        dashboard, alerts, config_dir = await demonstrate_dashboard_creation()
        metrics_simulation = await demonstrate_real_time_visualization()
        features, queries = await demonstrate_dashboard_features()
        
        print("\n" + "=" * 80)
        print("🎉 GRAFANA DASHBOARD DEMONSTRATION COMPLETED!")
        print("=" * 80)
        print()
        print("📁 FILES CREATED:")
        print(f"   Dashboard JSON: {config_dir}/eventuali-monitoring.json")
        print(f"   Alerts Config: {config_dir}/eventuali_alerts.yml")
        print(f"   Prometheus Config: {config_dir}/prometheus.yml")
        print(f"   Docker Compose: {config_dir}/docker-compose.yml")
        print(f"   Datasources: {config_dir}/datasources.yml")
        print()
        print("🚀 DEPLOYMENT INSTRUCTIONS:")
        print("1. Start the monitoring stack:")
        print(f"   cd {config_dir}")
        print("   docker-compose up -d")
        print()
        print("2. Import dashboard in Grafana:")
        print("   • Open http://localhost:3000 (admin/admin)")
        print("   • Go to Dashboards -> Import")
        print("   • Upload eventuali-monitoring.json")
        print()
        print("3. Start your Eventuali application with metrics:")
        print("   uv run python examples/38_prometheus_metrics_export.py")
        print()
        print("📊 DASHBOARD FEATURES:")
        for feature in features[:4]:  # Show top 4 features
            print(f"   • {feature['name']}: {feature['description']}")
        print()
        print("🔍 MONITORING CAPABILITIES:")
        print("   • Real-time event processing metrics")
        print("   • Performance analysis and bottleneck detection")
        print("   • Error tracking and alerting")
        print("   • Multi-tenant and aggregate-level visibility")
        print("   • System health and resource utilization")
        print("   • Business operation success rates")
        print()
        print("📈 Next Steps:")
        print("   • Configure alerting channels (email, Slack, PagerDuty)")
        print("   • Set up log aggregation integration")
        print("   • Create custom views for specific use cases")
        print("   • Implement automated anomaly detection")
        print("   • Add capacity planning dashboards")
        
    except Exception as e:
        print(f"❌ Error during demonstration: {e}")
        raise


if __name__ == "__main__":
    asyncio.run(main())