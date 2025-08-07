#!/usr/bin/env python3
"""
Enterprise Features Example

This example demonstrates enterprise-grade event sourcing features:
- Security and access control with audit trails
- Compliance and regulatory requirements (GDPR, SOX, etc.)
- Data governance with retention policies
- High availability and disaster recovery
- Advanced analytics and business intelligence
- Enterprise integration patterns and API management
"""

import asyncio
import sys
import os
from typing import ClassVar, Optional, Dict, List, Any, Union, Set
from datetime import datetime, timezone, timedelta
from enum import Enum
from dataclasses import dataclass, field
from abc import ABC, abstractmethod
import json
import uuid
import time
import hashlib
import hmac
import secrets
from collections import defaultdict, deque

# Add the python package to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'eventuali-python', 'python'))

from eventuali import EventStore
from eventuali.aggregate import User
from eventuali.event import UserRegistered, Event

# Security and Access Control
class SecurityLevel(Enum):
    PUBLIC = "public"
    INTERNAL = "internal"
    CONFIDENTIAL = "confidential"
    RESTRICTED = "restricted"

class Permission(Enum):
    READ = "read"
    WRITE = "write"
    DELETE = "delete"
    ADMIN = "admin"
    AUDIT = "audit"

@dataclass
class SecurityPrincipal:
    """Security principal (user or service)."""
    principal_id: str
    principal_type: str  # "user", "service", "system"
    roles: Set[str] = field(default_factory=set)
    permissions: Set[Permission] = field(default_factory=set)
    security_clearance: SecurityLevel = SecurityLevel.PUBLIC
    created_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))
    last_login: Optional[datetime] = None
    is_active: bool = True

class SecurityEvent(Event):
    """Base security event."""
    principal_id: str
    action: str
    resource: str
    timestamp: str
    ip_address: Optional[str] = None
    user_agent: Optional[str] = None
    success: bool = True
    risk_score: float = 0.0

class AccessControlEvent(SecurityEvent):
    """Access control event."""
    permission_required: str
    permission_granted: bool
    reason: Optional[str] = None

class AuditEvent(SecurityEvent):
    """Audit trail event."""
    event_type: str
    data_classification: str
    compliance_tags: List[str] = field(default_factory=list)
    retention_period_days: int = 2555  # 7 years default

class DataAccessEvent(SecurityEvent):
    """Data access tracking event."""
    data_type: str
    record_ids: List[str] = field(default_factory=list)
    access_pattern: str = "read"  # read, write, delete, export
    data_size_bytes: int = 0

class SecurityManager:
    """Enterprise security manager."""
    
    def __init__(self):
        self.principals: Dict[str, SecurityPrincipal] = {}
        self.access_policies: Dict[str, Dict[str, Any]] = {}
        self.security_events: List[SecurityEvent] = []
        self.failed_attempts: Dict[str, List[datetime]] = defaultdict(list)
        self.encryption_keys: Dict[str, str] = {}
        
        # Security thresholds
        self.max_failed_attempts = 5
        self.lockout_duration = timedelta(minutes=30)
        self.session_timeout = timedelta(hours=8)
    
    def create_principal(self, principal_id: str, principal_type: str, 
                        roles: Set[str] = None, clearance: SecurityLevel = SecurityLevel.PUBLIC) -> SecurityPrincipal:
        """Create security principal."""
        principal = SecurityPrincipal(
            principal_id=principal_id,
            principal_type=principal_type,
            roles=roles or set(),
            security_clearance=clearance
        )
        
        # Grant default permissions based on roles
        if "admin" in principal.roles:
            principal.permissions.update([Permission.READ, Permission.WRITE, Permission.DELETE, Permission.ADMIN])
        elif "auditor" in principal.roles:
            principal.permissions.update([Permission.READ, Permission.AUDIT])
        else:
            principal.permissions.add(Permission.READ)
        
        self.principals[principal_id] = principal
        return principal
    
    def authenticate(self, principal_id: str, credentials: str, ip_address: str = None) -> bool:
        """Authenticate principal."""
        if principal_id not in self.principals:
            self._record_security_event(principal_id, "authentication", "system", False, 
                                       "Principal not found", ip_address)
            return False
        
        principal = self.principals[principal_id]
        
        # Check for account lockout
        if self._is_account_locked(principal_id):
            self._record_security_event(principal_id, "authentication", "system", False,
                                       "Account locked", ip_address)
            return False
        
        # Simulate credential verification (in real implementation, use proper hashing)
        success = self._verify_credentials(credentials)
        
        if success:
            principal.last_login = datetime.now(timezone.utc)
            self._clear_failed_attempts(principal_id)
            self._record_security_event(principal_id, "authentication", "system", True,
                                       "Login successful", ip_address)
        else:
            self._record_failed_attempt(principal_id)
            self._record_security_event(principal_id, "authentication", "system", False,
                                       "Invalid credentials", ip_address, risk_score=3.0)
        
        return success
    
    def authorize(self, principal_id: str, resource: str, permission: Permission, 
                 context: Dict[str, Any] = None) -> bool:
        """Authorize access to resource."""
        if principal_id not in self.principals:
            return False
        
        principal = self.principals[principal_id]
        
        # Check if principal has required permission
        has_permission = permission in principal.permissions
        
        # Check role-based access
        if not has_permission and "admin" in principal.roles:
            has_permission = True
        
        # Check resource-specific policies
        if resource in self.access_policies:
            policy = self.access_policies[resource]
            required_clearance = SecurityLevel(policy.get("required_clearance", "public"))
            
            if principal.security_clearance.value != required_clearance.value:
                # Compare security levels (simplified)
                clearance_levels = ["public", "internal", "confidential", "restricted"]
                principal_level = clearance_levels.index(principal.security_clearance.value)
                required_level = clearance_levels.index(required_clearance.value)
                
                if principal_level < required_level:
                    has_permission = False
        
        # Record access control event
        event = AccessControlEvent(
            principal_id=principal_id,
            action="authorize",
            resource=resource,
            timestamp=datetime.now(timezone.utc).isoformat(),
            permission_required=permission.value,
            permission_granted=has_permission,
            reason="Policy evaluation completed"
        )
        
        self.security_events.append(event)
        return has_permission
    
    def track_data_access(self, principal_id: str, data_type: str, record_ids: List[str],
                         access_pattern: str = "read", data_size: int = 0):
        """Track data access for audit."""
        event = DataAccessEvent(
            principal_id=principal_id,
            action="data_access",
            resource=data_type,
            timestamp=datetime.now(timezone.utc).isoformat(),
            data_type=data_type,
            record_ids=record_ids,
            access_pattern=access_pattern,
            data_size_bytes=data_size
        )
        
        self.security_events.append(event)
    
    def _verify_credentials(self, credentials: str) -> bool:
        """Verify credentials (simplified)."""
        # In real implementation, use proper password hashing
        return len(credentials) >= 8
    
    def _is_account_locked(self, principal_id: str) -> bool:
        """Check if account is locked."""
        failed_attempts = self.failed_attempts.get(principal_id, [])
        if len(failed_attempts) < self.max_failed_attempts:
            return False
        
        # Check if lockout period has expired
        last_attempt = max(failed_attempts)
        return datetime.now(timezone.utc) - last_attempt < self.lockout_duration
    
    def _record_failed_attempt(self, principal_id: str):
        """Record failed authentication attempt."""
        now = datetime.now(timezone.utc)
        self.failed_attempts[principal_id].append(now)
        
        # Keep only recent attempts
        cutoff = now - self.lockout_duration
        self.failed_attempts[principal_id] = [
            attempt for attempt in self.failed_attempts[principal_id]
            if attempt > cutoff
        ]
    
    def _clear_failed_attempts(self, principal_id: str):
        """Clear failed attempts after successful login."""
        if principal_id in self.failed_attempts:
            del self.failed_attempts[principal_id]
    
    def _record_security_event(self, principal_id: str, action: str, resource: str, 
                             success: bool, reason: str = None, ip_address: str = None,
                             risk_score: float = 0.0):
        """Record security event."""
        event = SecurityEvent(
            principal_id=principal_id,
            action=action,
            resource=resource,
            timestamp=datetime.now(timezone.utc).isoformat(),
            ip_address=ip_address,
            success=success,
            risk_score=risk_score
        )
        
        self.security_events.append(event)
    
    def get_security_report(self) -> Dict[str, Any]:
        """Generate security report."""
        recent_events = [e for e in self.security_events 
                        if datetime.fromisoformat(e.timestamp.replace('Z', '+00:00')) > 
                           datetime.now(timezone.utc) - timedelta(hours=24)]
        
        failed_logins = [e for e in recent_events if e.action == "authentication" and not e.success]
        successful_logins = [e for e in recent_events if e.action == "authentication" and e.success]
        
        return {
            "total_events": len(self.security_events),
            "recent_events": len(recent_events),
            "successful_logins": len(successful_logins),
            "failed_logins": len(failed_logins),
            "active_principals": len([p for p in self.principals.values() if p.is_active]),
            "locked_accounts": len([p for p in self.principals.keys() if self._is_account_locked(p)]),
            "high_risk_events": len([e for e in recent_events if e.risk_score > 2.0])
        }

# Compliance and Regulatory Management
class ComplianceStandard(Enum):
    GDPR = "gdpr"
    SOX = "sox"
    HIPAA = "hipaa"
    PCI_DSS = "pci_dss"
    ISO_27001 = "iso_27001"

class DataClassification(Enum):
    PUBLIC = "public"
    INTERNAL = "internal"
    PERSONAL = "personal"
    FINANCIAL = "financial"
    MEDICAL = "medical"

@dataclass
class ComplianceRule:
    """Compliance rule definition."""
    rule_id: str
    standard: ComplianceStandard
    description: str
    data_types: Set[DataClassification]
    retention_period_days: int
    deletion_required: bool = False
    encryption_required: bool = False
    audit_required: bool = True

class ComplianceEvent(Event):
    """Compliance-related event."""
    compliance_standard: str
    rule_id: str
    violation_type: Optional[str] = None
    severity: str = "medium"
    remediation_required: bool = False

class ComplianceManager:
    """Compliance and regulatory manager."""
    
    def __init__(self):
        self.rules: Dict[str, ComplianceRule] = {}
        self.violations: List[ComplianceEvent] = []
        self.data_inventory: Dict[str, Dict[str, Any]] = {}
        self.retention_policies: Dict[DataClassification, int] = {}
        
        self._initialize_default_rules()
    
    def _initialize_default_rules(self):
        """Initialize default compliance rules."""
        # GDPR rules
        gdpr_personal_data = ComplianceRule(
            rule_id="gdpr_personal_data",
            standard=ComplianceStandard.GDPR,
            description="Personal data processing and retention",
            data_types={DataClassification.PERSONAL},
            retention_period_days=730,  # 2 years
            deletion_required=True,
            encryption_required=True,
            audit_required=True
        )
        
        # SOX financial data
        sox_financial = ComplianceRule(
            rule_id="sox_financial",
            standard=ComplianceStandard.SOX,
            description="Financial data retention for audit",
            data_types={DataClassification.FINANCIAL},
            retention_period_days=2555,  # 7 years
            deletion_required=False,
            encryption_required=True,
            audit_required=True
        )
        
        self.rules[gdpr_personal_data.rule_id] = gdpr_personal_data
        self.rules[sox_financial.rule_id] = sox_financial
    
    def add_data_to_inventory(self, data_id: str, data_type: DataClassification,
                            owner: str, created_date: datetime = None):
        """Add data to compliance inventory."""
        created_date = created_date or datetime.now(timezone.utc)
        
        self.data_inventory[data_id] = {
            "data_id": data_id,
            "data_type": data_type,
            "owner": owner,
            "created_date": created_date.isoformat(),
            "last_accessed": None,
            "retention_status": "active",
            "compliance_tags": self._get_compliance_tags(data_type)
        }
    
    def check_retention_compliance(self) -> List[Dict[str, Any]]:
        """Check data retention compliance."""
        violations = []
        current_time = datetime.now(timezone.utc)
        
        for data_id, data_info in self.data_inventory.items():
            data_type = data_info["data_type"]
            created_date = datetime.fromisoformat(data_info["created_date"])
            
            # Find applicable rules
            applicable_rules = [rule for rule in self.rules.values() 
                              if data_type in rule.data_types]
            
            for rule in applicable_rules:
                retention_period = timedelta(days=rule.retention_period_days)
                expiry_date = created_date + retention_period
                
                if current_time > expiry_date and rule.deletion_required:
                    violations.append({
                        "data_id": data_id,
                        "rule_id": rule.rule_id,
                        "violation_type": "retention_exceeded",
                        "days_overdue": (current_time - expiry_date).days,
                        "action_required": "delete",
                        "compliance_standard": rule.standard.value
                    })
        
        return violations
    
    def process_data_subject_request(self, subject_id: str, request_type: str) -> Dict[str, Any]:
        """Process data subject request (GDPR Article 17, etc.)."""
        affected_data = [
            data_id for data_id, data_info in self.data_inventory.items()
            if data_info.get("subject_id") == subject_id or subject_id in str(data_info)
        ]
        
        if request_type == "deletion":
            # Mark for deletion
            for data_id in affected_data:
                self.data_inventory[data_id]["retention_status"] = "deletion_requested"
                self.data_inventory[data_id]["deletion_request_date"] = datetime.now(timezone.utc).isoformat()
            
            # Record compliance event
            event = ComplianceEvent(
                compliance_standard="gdpr",
                rule_id="gdpr_personal_data",
                violation_type=None,
                severity="high",
                remediation_required=True
            )
            self.violations.append(event)
        
        return {
            "request_id": str(uuid.uuid4()),
            "subject_id": subject_id,
            "request_type": request_type,
            "affected_records": len(affected_data),
            "status": "processed",
            "completion_date": datetime.now(timezone.utc).isoformat()
        }
    
    def _get_compliance_tags(self, data_type: DataClassification) -> List[str]:
        """Get compliance tags for data type."""
        tags = []
        
        if data_type == DataClassification.PERSONAL:
            tags.extend(["gdpr", "privacy"])
        elif data_type == DataClassification.FINANCIAL:
            tags.extend(["sox", "financial_audit"])
        elif data_type == DataClassification.MEDICAL:
            tags.extend(["hipaa", "medical_privacy"])
        
        return tags
    
    def generate_compliance_report(self) -> Dict[str, Any]:
        """Generate comprehensive compliance report."""
        violations = self.check_retention_compliance()
        
        # Count by compliance standard
        standards_summary = defaultdict(int)
        for rule in self.rules.values():
            standards_summary[rule.standard.value] += 1
        
        # Data inventory summary
        data_by_type = defaultdict(int)
        for data_info in self.data_inventory.values():
            data_by_type[data_info["data_type"].value] += 1
        
        return {
            "compliance_standards": dict(standards_summary),
            "data_inventory_summary": dict(data_by_type),
            "active_rules": len(self.rules),
            "total_data_records": len(self.data_inventory),
            "retention_violations": len(violations),
            "deletion_pending": len([d for d in self.data_inventory.values() 
                                   if d.get("retention_status") == "deletion_requested"]),
            "report_generated": datetime.now(timezone.utc).isoformat()
        }

# High Availability and Disaster Recovery
class HAStatus(Enum):
    ACTIVE = "active"
    STANDBY = "standby"
    FAILED = "failed"
    MAINTENANCE = "maintenance"

@dataclass
class HANode:
    """High availability node."""
    node_id: str
    status: HAStatus
    last_heartbeat: datetime
    role: str  # "primary", "secondary", "witness"
    region: str
    health_score: float = 100.0
    
class DisasterRecoveryManager:
    """Disaster recovery and high availability manager."""
    
    def __init__(self):
        self.nodes: Dict[str, HANode] = {}
        self.primary_node: Optional[str] = None
        self.backup_schedule: Dict[str, Any] = {}
        self.recovery_procedures: Dict[str, List[str]] = {}
        self.rto_target = 4  # Recovery Time Objective in hours
        self.rpo_target = 1  # Recovery Point Objective in hours
    
    def add_node(self, node_id: str, role: str, region: str) -> HANode:
        """Add HA node."""
        node = HANode(
            node_id=node_id,
            status=HAStatus.ACTIVE,
            last_heartbeat=datetime.now(timezone.utc),
            role=role,
            region=region
        )
        
        self.nodes[node_id] = node
        
        if role == "primary" and self.primary_node is None:
            self.primary_node = node_id
        
        return node
    
    def update_node_health(self, node_id: str, health_score: float):
        """Update node health score."""
        if node_id in self.nodes:
            node = self.nodes[node_id]
            node.health_score = health_score
            node.last_heartbeat = datetime.now(timezone.utc)
            
            # Trigger failover if primary is unhealthy
            if node_id == self.primary_node and health_score < 20.0:
                self._trigger_failover()
    
    def _trigger_failover(self) -> Optional[str]:
        """Trigger automatic failover."""
        if not self.primary_node:
            return None
        
        # Find best secondary node
        secondary_nodes = [n for n in self.nodes.values() 
                          if n.role == "secondary" and n.status == HAStatus.ACTIVE]
        
        if not secondary_nodes:
            return None
        
        # Select secondary with highest health score
        new_primary = max(secondary_nodes, key=lambda x: x.health_score)
        
        # Perform failover
        old_primary = self.nodes[self.primary_node]
        old_primary.status = HAStatus.FAILED
        old_primary.role = "secondary"
        
        new_primary.role = "primary"
        self.primary_node = new_primary.node_id
        
        return new_primary.node_id
    
    def create_backup(self, backup_type: str = "incremental") -> Dict[str, Any]:
        """Create backup."""
        backup_id = str(uuid.uuid4())
        backup_info = {
            "backup_id": backup_id,
            "backup_type": backup_type,
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "size_bytes": 1024 * 1024 * 100,  # Simulated 100MB
            "checksum": hashlib.md5(backup_id.encode()).hexdigest(),
            "status": "completed"
        }
        
        self.backup_schedule[backup_id] = backup_info
        return backup_info
    
    def get_ha_status(self) -> Dict[str, Any]:
        """Get high availability status."""
        active_nodes = [n for n in self.nodes.values() if n.status == HAStatus.ACTIVE]
        failed_nodes = [n for n in self.nodes.values() if n.status == HAStatus.FAILED]
        
        return {
            "primary_node": self.primary_node,
            "total_nodes": len(self.nodes),
            "active_nodes": len(active_nodes),
            "failed_nodes": len(failed_nodes),
            "average_health": sum(n.health_score for n in self.nodes.values()) / len(self.nodes) if self.nodes else 0,
            "rto_target_hours": self.rto_target,
            "rpo_target_hours": self.rpo_target,
            "last_backup": max(self.backup_schedule.values(), key=lambda x: x["timestamp"])["timestamp"] if self.backup_schedule else None
        }

# Advanced Analytics and Business Intelligence
class AnalyticsMetric:
    """Analytics metric definition."""
    
    def __init__(self, name: str, aggregation: str = "sum", dimension: str = None):
        self.name = name
        self.aggregation = aggregation  # sum, count, avg, min, max
        self.dimension = dimension
        self.values: List[float] = []
        self.timestamps: List[str] = []
    
    def record_value(self, value: float, timestamp: str = None):
        """Record metric value."""
        timestamp = timestamp or datetime.now(timezone.utc).isoformat()
        self.values.append(value)
        self.timestamps.append(timestamp)
    
    def compute(self) -> Dict[str, Any]:
        """Compute metric statistics."""
        if not self.values:
            return {"value": 0, "count": 0}
        
        if self.aggregation == "sum":
            result = sum(self.values)
        elif self.aggregation == "count":
            result = len(self.values)
        elif self.aggregation == "avg":
            result = sum(self.values) / len(self.values)
        elif self.aggregation == "min":
            result = min(self.values)
        elif self.aggregation == "max":
            result = max(self.values)
        else:
            result = sum(self.values)
        
        return {
            "value": result,
            "count": len(self.values),
            "last_updated": self.timestamps[-1] if self.timestamps else None
        }

class BusinessIntelligenceEngine:
    """Advanced business intelligence and analytics engine."""
    
    def __init__(self):
        self.metrics: Dict[str, AnalyticsMetric] = {}
        self.dashboards: Dict[str, Dict[str, Any]] = {}
        self.kpis: Dict[str, Dict[str, Any]] = {}
        self.alerts: List[Dict[str, Any]] = []
    
    def define_metric(self, name: str, aggregation: str = "sum", dimension: str = None) -> AnalyticsMetric:
        """Define a new metric."""
        metric = AnalyticsMetric(name, aggregation, dimension)
        self.metrics[name] = metric
        return metric
    
    def record_business_event(self, event_type: str, value: float, dimensions: Dict[str, str] = None):
        """Record business event for analytics."""
        dimensions = dimensions or {}
        
        # Record base metric
        if event_type in self.metrics:
            self.metrics[event_type].record_value(value)
        
        # Record dimensional metrics
        for dim_key, dim_value in dimensions.items():
            dim_metric_name = f"{event_type}_{dim_key}_{dim_value}"
            if dim_metric_name not in self.metrics:
                self.metrics[dim_metric_name] = AnalyticsMetric(dim_metric_name, "sum", dim_key)
            
            self.metrics[dim_metric_name].record_value(value)
    
    def create_dashboard(self, dashboard_id: str, title: str, metrics: List[str]) -> Dict[str, Any]:
        """Create analytics dashboard."""
        dashboard = {
            "dashboard_id": dashboard_id,
            "title": title,
            "metrics": metrics,
            "created_at": datetime.now(timezone.utc).isoformat(),
            "last_updated": None,
            "auto_refresh": True
        }
        
        self.dashboards[dashboard_id] = dashboard
        return dashboard
    
    def define_kpi(self, kpi_id: str, name: str, target_value: float, 
                  metric_name: str, comparison: str = "gte") -> Dict[str, Any]:
        """Define Key Performance Indicator."""
        kpi = {
            "kpi_id": kpi_id,
            "name": name,
            "target_value": target_value,
            "metric_name": metric_name,
            "comparison": comparison,  # gte, lte, eq
            "status": "unknown",
            "current_value": 0,
            "achievement_percentage": 0
        }
        
        self.kpis[kpi_id] = kpi
        return kpi
    
    def update_kpis(self):
        """Update all KPI statuses."""
        for kpi in self.kpis.values():
            if kpi["metric_name"] in self.metrics:
                metric_result = self.metrics[kpi["metric_name"]].compute()
                current_value = metric_result["value"]
                
                kpi["current_value"] = current_value
                
                # Calculate achievement
                if kpi["comparison"] == "gte":
                    achievement = (current_value / kpi["target_value"]) * 100
                    status = "achieved" if current_value >= kpi["target_value"] else "below_target"
                elif kpi["comparison"] == "lte":
                    achievement = (kpi["target_value"] / max(current_value, 1)) * 100
                    status = "achieved" if current_value <= kpi["target_value"] else "above_target"
                else:
                    achievement = 100 if current_value == kpi["target_value"] else 0
                    status = "achieved" if current_value == kpi["target_value"] else "missed"
                
                kpi["achievement_percentage"] = min(achievement, 100)
                kpi["status"] = status
    
    def generate_executive_report(self) -> Dict[str, Any]:
        """Generate executive summary report."""
        self.update_kpis()
        
        # KPI summary
        achieved_kpis = [kpi for kpi in self.kpis.values() if kpi["status"] == "achieved"]
        critical_metrics = [metric for metric in self.metrics.values() if len(metric.values) > 0]
        
        # Business trends
        revenue_metrics = [m for name, m in self.metrics.items() if "revenue" in name.lower()]
        total_revenue = sum(sum(m.values) for m in revenue_metrics)
        
        return {
            "executive_summary": {
                "total_kpis": len(self.kpis),
                "achieved_kpis": len(achieved_kpis),
                "kpi_achievement_rate": (len(achieved_kpis) / max(len(self.kpis), 1)) * 100,
                "total_metrics_tracked": len(self.metrics),
                "active_dashboards": len(self.dashboards),
                "total_revenue": total_revenue
            },
            "kpi_status": {kpi_id: kpi["status"] for kpi_id, kpi in self.kpis.items()},
            "top_performing_metrics": sorted(
                [(name, m.compute()["value"]) for name, m in self.metrics.items()],
                key=lambda x: x[1], reverse=True
            )[:5],
            "generated_at": datetime.now(timezone.utc).isoformat()
        }

async def demonstrate_enterprise_features():
    """Demonstrate enterprise-grade event sourcing features."""
    print("=== Enterprise Features Example ===\n")
    
    event_store = await EventStore.create("sqlite://:memory:")
    
    print("1. Security and access control...")
    
    # Set up security manager
    security_mgr = SecurityManager()
    
    # Create security principals
    principals = [
        ("admin_user", "user", {"admin"}, SecurityLevel.RESTRICTED),
        ("audit_user", "user", {"auditor"}, SecurityLevel.CONFIDENTIAL),
        ("regular_user", "user", {"employee"}, SecurityLevel.INTERNAL),
        ("api_service", "service", {"api"}, SecurityLevel.INTERNAL)
    ]
    
    for principal_id, principal_type, roles, clearance in principals:
        principal = security_mgr.create_principal(principal_id, principal_type, roles, clearance)
        print(f"   ✓ Created principal: {principal_id} ({principal_type}) - {clearance.value}")
        
        # Test authentication
        success = security_mgr.authenticate(principal_id, "secure_password_123", "192.168.1.100")
        status = "✅" if success else "❌"
        print(f"     {status} Authentication: {principal_id}")
    
    # Test authorization
    print(f"\n   🔐 Authorization Tests:")
    test_resources = [
        ("financial_data", Permission.READ, SecurityLevel.CONFIDENTIAL),
        ("customer_data", Permission.WRITE, SecurityLevel.INTERNAL),
        ("system_config", Permission.ADMIN, SecurityLevel.RESTRICTED)
    ]
    
    for resource, permission, required_level in test_resources:
        # Set access policy
        security_mgr.access_policies[resource] = {"required_clearance": required_level.value}
        
        for principal_id, _, _, _ in principals[:3]:  # Test first 3 principals
            authorized = security_mgr.authorize(principal_id, resource, permission)
            status = "✅" if authorized else "❌"
            print(f"     {status} {principal_id} → {resource} ({permission.value})")
    
    # Track data access
    security_mgr.track_data_access("admin_user", "customer_records", 
                                 ["cust-1", "cust-2", "cust-3"], "read", 1024)
    
    security_report = security_mgr.get_security_report()
    print(f"\n   📊 Security Report:")
    print(f"     Total security events: {security_report['total_events']}")
    print(f"     Successful logins: {security_report['successful_logins']}")
    print(f"     Failed logins: {security_report['failed_logins']}")
    print(f"     Active principals: {security_report['active_principals']}")
    
    print("\n2. Compliance and regulatory management...")
    
    # Set up compliance manager
    compliance_mgr = ComplianceManager()
    
    # Add data to inventory
    test_data = [
        ("customer_email_1", DataClassification.PERSONAL, "customer_service"),
        ("financial_report_2024", DataClassification.FINANCIAL, "finance_team"),
        ("marketing_campaign", DataClassification.INTERNAL, "marketing_team"),
        ("user_profile_123", DataClassification.PERSONAL, "product_team")
    ]
    
    # Add historical data (some overdue)
    historical_date = datetime.now(timezone.utc) - timedelta(days=800)  # Over 2 years old
    
    for data_id, data_type, owner in test_data:
        # Some data is old to test retention compliance
        created_date = historical_date if "email" in data_id else None
        compliance_mgr.add_data_to_inventory(data_id, data_type, owner, created_date)
        print(f"   ✓ Added to inventory: {data_id} ({data_type.value})")
    
    # Check compliance violations
    violations = compliance_mgr.check_retention_compliance()
    print(f"\n   ⚖️  Compliance Analysis:")
    print(f"     Total data records: {len(compliance_mgr.data_inventory)}")
    print(f"     Retention violations: {len(violations)}")
    
    for violation in violations:
        print(f"       🚨 {violation['data_id']}: {violation['days_overdue']} days overdue")
        print(f"          Action: {violation['action_required']} ({violation['compliance_standard']})")
    
    # Process data subject request (GDPR)
    dsr_result = compliance_mgr.process_data_subject_request("user_123", "deletion")
    print(f"\n   📋 Data Subject Request Processed:")
    print(f"     Request ID: {dsr_result['request_id']}")
    print(f"     Type: {dsr_result['request_type']}")
    print(f"     Affected records: {dsr_result['affected_records']}")
    
    compliance_report = compliance_mgr.generate_compliance_report()
    print(f"\n   📊 Compliance Report:")
    print(f"     Standards tracked: {list(compliance_report['compliance_standards'].keys())}")
    print(f"     Data by type: {compliance_report['data_inventory_summary']}")
    print(f"     Retention violations: {compliance_report['retention_violations']}")
    
    print("\n3. High availability and disaster recovery...")
    
    # Set up DR manager
    dr_mgr = DisasterRecoveryManager()
    
    # Add HA nodes
    ha_nodes = [
        ("primary-us-east", "primary", "us-east-1"),
        ("secondary-us-west", "secondary", "us-west-2"), 
        ("secondary-eu-west", "secondary", "eu-west-1"),
        ("witness-ap-south", "witness", "ap-south-1")
    ]
    
    for node_id, role, region in ha_nodes:
        node = dr_mgr.add_node(node_id, role, region)
        print(f"   ✓ Added HA node: {node_id} ({role}) in {region}")
    
    # Simulate health monitoring
    print(f"\n   ❤️  Health Monitoring:")
    for node_id, _, _ in ha_nodes:
        health_score = 95.0 if "primary" not in node_id else 85.0
        dr_mgr.update_node_health(node_id, health_score)
        print(f"     {node_id}: {health_score}% health")
    
    # Simulate failover scenario
    print(f"\n   🔄 Failover Simulation:")
    dr_mgr.update_node_health("primary-us-east", 15.0)  # Trigger failover
    
    ha_status = dr_mgr.get_ha_status()
    print(f"     New primary node: {ha_status['primary_node']}")
    print(f"     Active nodes: {ha_status['active_nodes']}/{ha_status['total_nodes']}")
    print(f"     Average health: {ha_status['average_health']:.1f}%")
    
    # Create backup
    backup_result = dr_mgr.create_backup("incremental")
    print(f"\n   💾 Backup Created:")
    print(f"     Backup ID: {backup_result['backup_id']}")
    print(f"     Type: {backup_result['backup_type']}")
    print(f"     Size: {backup_result['size_bytes'] / (1024*1024):.1f} MB")
    
    print("\n4. Advanced analytics and business intelligence...")
    
    # Set up BI engine
    bi_engine = BusinessIntelligenceEngine()
    
    # Define business metrics
    metrics_config = [
        ("revenue", "sum"),
        ("orders", "count"),
        ("customer_acquisition", "count"),
        ("response_time", "avg"),
        ("error_rate", "avg")
    ]
    
    for name, aggregation in metrics_config:
        bi_engine.define_metric(name, aggregation)
        print(f"   ✓ Defined metric: {name} ({aggregation})")
    
    # Simulate business events
    print(f"\n   📈 Recording Business Events:")
    business_events = [
        ("revenue", 15000.0, {"region": "us", "product": "premium"}),
        ("revenue", 8500.0, {"region": "eu", "product": "standard"}),
        ("orders", 45, {"channel": "web"}),
        ("orders", 23, {"channel": "mobile"}),
        ("customer_acquisition", 12, {"source": "organic"}),
        ("customer_acquisition", 8, {"source": "paid"}),
        ("response_time", 150.0, {"endpoint": "api"}),
        ("error_rate", 0.5, {"service": "payment"})
    ]
    
    for event_type, value, dimensions in business_events:
        bi_engine.record_business_event(event_type, value, dimensions)
    
    print(f"     Recorded {len(business_events)} business events")
    
    # Create dashboard
    dashboard = bi_engine.create_dashboard(
        "executive_dashboard",
        "Executive Performance Dashboard", 
        ["revenue", "orders", "customer_acquisition"]
    )
    print(f"   ✓ Created dashboard: {dashboard['title']}")
    
    # Define KPIs
    kpis_config = [
        ("monthly_revenue", "Monthly Revenue Target", 50000.0, "revenue", "gte"),
        ("order_volume", "Order Volume Target", 100, "orders", "gte"),
        ("response_time", "Response Time SLA", 200.0, "response_time", "lte"),
        ("error_rate", "Error Rate Target", 1.0, "error_rate", "lte")
    ]
    
    for kpi_id, name, target, metric, comparison in kpis_config:
        kpi = bi_engine.define_kpi(kpi_id, name, target, metric, comparison)
        print(f"   ✓ Defined KPI: {name} (target: {target})")
    
    # Generate executive report
    exec_report = bi_engine.generate_executive_report()
    print(f"\n   📊 Executive Report:")
    summary = exec_report["executive_summary"]
    print(f"     KPIs Achieved: {summary['achieved_kpis']}/{summary['total_kpis']} ({summary['kpi_achievement_rate']:.1f}%)")
    print(f"     Metrics Tracked: {summary['total_metrics_tracked']}")
    print(f"     Total Revenue: ${summary['total_revenue']:,.2f}")
    print(f"     Active Dashboards: {summary['active_dashboards']}")
    
    print(f"\n     KPI Status:")
    for kpi_id, status in exec_report["kpi_status"].items():
        status_icon = "✅" if status == "achieved" else "🔴"
        print(f"       {status_icon} {kpi_id}: {status}")
    
    print(f"\n     Top Performing Metrics:")
    for name, value in exec_report["top_performing_metrics"][:3]:
        print(f"       📊 {name}: {value:,.2f}")
    
    print("\n5. Enterprise integration summary...")
    
    # Summary of all enterprise features
    enterprise_summary = {
        "security": {
            "principals": len(security_mgr.principals),
            "security_events": len(security_mgr.security_events),
            "policies": len(security_mgr.access_policies)
        },
        "compliance": {
            "standards": len(compliance_mgr.rules),
            "data_records": len(compliance_mgr.data_inventory),
            "violations": len(violations)
        },
        "high_availability": {
            "nodes": len(dr_mgr.nodes),
            "backups": len(dr_mgr.backup_schedule),
            "uptime_percentage": ha_status["average_health"]
        },
        "analytics": {
            "metrics": len(bi_engine.metrics),
            "dashboards": len(bi_engine.dashboards),
            "kpis": len(bi_engine.kpis),
            "kpi_achievement": summary["kpi_achievement_rate"]
        }
    }
    
    print(f"   🏢 Enterprise Feature Summary:")
    for category, stats in enterprise_summary.items():
        print(f"     {category.title()}:")
        for metric, value in stats.items():
            print(f"       - {metric.replace('_', ' ').title()}: {value}")
    
    return {
        "security_manager": security_mgr,
        "compliance_manager": compliance_mgr,
        "dr_manager": dr_mgr,
        "bi_engine": bi_engine,
        "enterprise_summary": enterprise_summary,
        "security_report": security_report,
        "compliance_report": compliance_report,
        "ha_status": ha_status,
        "executive_report": exec_report
    }

async def main():
    result = await demonstrate_enterprise_features()
    
    print(f"\n✅ SUCCESS! Enterprise features demonstrated!")
    
    print(f"\nEnterprise features covered:")
    print(f"- ✓ Security and access control with role-based permissions")
    print(f"- ✓ Compliance management (GDPR, SOX, HIPAA standards)")
    print(f"- ✓ Data governance with automated retention policies")
    print(f"- ✓ High availability with automatic failover")
    print(f"- ✓ Disaster recovery with backup management")
    print(f"- ✓ Advanced business intelligence and analytics")
    print(f"- ✓ Executive reporting and KPI monitoring")
    print(f"- ✓ Audit trails and regulatory compliance")
    
    summary = result["enterprise_summary"]
    print(f"\nEnterprise deployment metrics:")
    print(f"- Security principals: {summary['security']['principals']}")
    print(f"- Compliance standards: {summary['compliance']['standards']}")
    print(f"- HA nodes deployed: {summary['high_availability']['nodes']}")
    print(f"- System uptime: {summary['high_availability']['uptime_percentage']:.1f}%")
    print(f"- Business metrics: {summary['analytics']['metrics']}")
    print(f"- KPI achievement: {summary['analytics']['kpi_achievement']:.1f}%")

if __name__ == "__main__":
    asyncio.run(main())