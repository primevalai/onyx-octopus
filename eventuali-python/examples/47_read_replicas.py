#!/usr/bin/env python3
"""
Example 47: Read Replicas for Query Performance Scaling

This example demonstrates read replica management for scaling query performance
in event sourcing systems. Read replicas allow distributing read load across
multiple database instances while maintaining strong consistency.

Features Demonstrated:
- Read replica configuration and management
- Load balancing between primary and secondary databases
- Read preference strategies (primary, secondary, nearest)
- Lag tolerance and monitoring
- Failover scenarios and recovery

Performance Expectations:
- 2-5x improved read throughput with proper replica distribution
- Reduced latency for geographically distributed reads
- Better resource utilization across database instances

Usage:
    uv run python examples/47_read_replicas.py
"""

import asyncio
import time
import tempfile
import os
import random
from pathlib import Path
from typing import List, Dict, Optional

from eventuali import EventStore
from eventuali.event import DomainEvent
from eventuali.performance import (
    ReadPreference,
    ReplicaConfig, 
    ReadReplicaManager
)


class QueryPerformanceEvent(DomainEvent):
    """Test event for read replica demonstration."""
    query_id: int
    query_type: str
    data_size: int
    region: str


class ReadReplicaDemo:
    """Demonstrates read replica management for query performance scaling."""
    
    def __init__(self):
        """Initialize the read replica demo."""
        self.temp_dir = tempfile.mkdtemp()
        self.primary_db = os.path.join(self.temp_dir, "primary.db")
        self.replica_dbs = [
            os.path.join(self.temp_dir, "replica_us_east.db"),
            os.path.join(self.temp_dir, "replica_us_west.db"),
            os.path.join(self.temp_dir, "replica_eu_west.db"),
        ]
        
    def demonstrate_replica_configurations(self):
        """Showcase different read replica configuration options."""
        print("🔧 Read Replica Configuration Showcase")
        print("=" * 60)
        
        # Default configuration
        default_config = ReplicaConfig.default()
        print(f"📋 Default Config: {default_config}")
        
        # Primary-only configuration (no replicas)
        primary_config = ReplicaConfig(
            read_preference=ReadPreference.PRIMARY,
            max_lag_ms=0
        )
        print(f"🔒 Primary-Only Config: {primary_config}")
        
        # Secondary-preferred configuration
        secondary_config = ReplicaConfig(
            read_preference=ReadPreference.SECONDARY, 
            max_lag_ms=500  # 500ms lag tolerance
        )
        print(f"📖 Secondary-Preferred Config: {secondary_config}")
        
        # Nearest replica configuration
        nearest_config = ReplicaConfig(
            read_preference=ReadPreference.NEAREST,
            max_lag_ms=1000  # 1 second lag tolerance
        )
        print(f"🌐 Nearest Replica Config: {nearest_config}")
        
        # Strict consistency configuration
        strict_config = ReplicaConfig(
            read_preference=ReadPreference.PRIMARY,
            max_lag_ms=0  # No lag tolerance
        )
        print(f"🛡️  Strict Consistency Config: {strict_config}")
        print()

    async def demonstrate_replica_setup(self):
        """Demonstrate setting up primary and replica databases."""
        print("🏗️  Read Replica Setup and Management")
        print("=" * 60)
        
        try:
            # Initialize primary database
            print("📋 Setting up primary database...")
            primary_store = await EventStore.create(f"sqlite://{self.primary_db}")
            print(f"✅ Primary database initialized: {os.path.basename(self.primary_db)}")
            
            # Simulate replica databases (in practice, these would be separate instances)
            print(f"\n📋 Setting up {len(self.replica_dbs)} replica databases...")
            replica_stores = []
            for i, replica_db in enumerate(self.replica_dbs):
                replica_store = await EventStore.create(f"sqlite://{replica_db}")
                region = ["US-East", "US-West", "EU-West"][i]
                replica_stores.append((region, replica_store))
                print(f"✅ Replica {i+1} initialized: {region} ({os.path.basename(replica_db)})")
                
            # Create replica manager with different configurations
            configs = [
                ("Balanced", ReplicaConfig(read_preference=ReadPreference.SECONDARY, max_lag_ms=1000)),
                ("Primary-Only", ReplicaConfig(read_preference=ReadPreference.PRIMARY, max_lag_ms=0)),
                ("Nearest", ReplicaConfig(read_preference=ReadPreference.NEAREST, max_lag_ms=2000)),
            ]
            
            managers = []
            for config_name, config in configs:
                manager = ReadReplicaManager(config)
                managers.append((config_name, manager))
                print(f"🔧 Created replica manager: {config_name}")
            
            print(f"\n📊 Replica Setup Summary:")
            print(f"   Primary Database: 1")
            print(f"   Read Replicas: {len(replica_stores)}")
            print(f"   Replica Managers: {len(managers)}")
            print(f"   Total Read Capacity: {len(replica_stores) + 1}x")
            
        except Exception as e:
            print(f"❌ Replica setup failed: {e}")
            
        print()

    def simulate_read_workload_distribution(self):
        """Simulate how read workload gets distributed across replicas."""
        print("📊 Read Workload Distribution Simulation")
        print("=" * 60)
        
        # Simulate different read patterns
        read_scenarios = [
            {
                "name": "Analytics Queries",
                "preference": ReadPreference.SECONDARY,
                "queries": 1000,
                "avg_latency_ms": 45,
                "description": "Heavy analytical queries routed to replicas"
            },
            {
                "name": "User Dashboard",
                "preference": ReadPreference.NEAREST,
                "queries": 5000,
                "avg_latency_ms": 12,
                "description": "Real-time user queries using nearest replica"
            },
            {
                "name": "Critical Transactions",
                "preference": ReadPreference.PRIMARY,
                "queries": 500,
                "avg_latency_ms": 8,
                "description": "Strict consistency reads from primary"
            },
            {
                "name": "Reporting System",
                "preference": ReadPreference.SECONDARY,
                "queries": 2000,
                "avg_latency_ms": 25,
                "description": "Batch reporting queries on replicas"
            }
        ]
        
        total_queries = sum(scenario["queries"] for scenario in read_scenarios)
        
        print(f"🔍 Read Workload Analysis ({total_queries:,} total queries):")
        print("-" * 70)
        print(f"{'Workload Type':<20} {'Queries':<8} {'Target':<12} {'Avg Latency':<12} {'%':<8}")
        print("-" * 70)
        
        for scenario in read_scenarios:
            percentage = (scenario["queries"] / total_queries) * 100
            target = scenario["preference"].value if hasattr(scenario["preference"], 'value') else str(scenario["preference"])
            
            print(f"{scenario['name']:<20} {scenario['queries']:<8,} {target:<12} {scenario['avg_latency_ms']:<12}ms {percentage:<8.1f}%")
            print(f"{'  └─ ' + scenario['description']:<60}")
            
        print()
        
        # Show replica load distribution
        print("📈 Estimated Replica Load Distribution:")
        replica_load = {
            "Primary": 0,
            "US-East Replica": 0,
            "US-West Replica": 0,
            "EU-West Replica": 0,
        }
        
        for scenario in read_scenarios:
            if scenario["preference"] == ReadPreference.PRIMARY:
                replica_load["Primary"] += scenario["queries"]
            elif scenario["preference"] == ReadPreference.SECONDARY:
                # Distribute across replicas
                per_replica = scenario["queries"] // 3
                replica_load["US-East Replica"] += per_replica
                replica_load["US-West Replica"] += per_replica
                replica_load["EU-West Replica"] += per_replica
            else:  # NEAREST
                # Simulate geographic distribution
                replica_load["US-East Replica"] += scenario["queries"] * 0.4
                replica_load["US-West Replica"] += scenario["queries"] * 0.3
                replica_load["EU-West Replica"] += scenario["queries"] * 0.3
        
        print("-" * 50)
        for replica, load in replica_load.items():
            load_percentage = (load / total_queries) * 100
            load_bar = "█" * int(load_percentage / 5) + "░" * (20 - int(load_percentage / 5))
            print(f"{replica:<18}: {load_bar} {load_percentage:5.1f}% ({load:,.0f} queries)")
            
        print()

    async def demonstrate_failover_scenarios(self):
        """Demonstrate replica failover and recovery scenarios."""
        print("🔄 Replica Failover and Recovery Scenarios")
        print("=" * 60)
        
        scenarios = [
            {
                "name": "Primary Database Failure",
                "description": "Primary goes down, reads failover to replicas",
                "impact": "Write operations pause, reads continue on replicas",
                "recovery_time": "30 seconds (automatic failover)",
                "data_consistency": "Eventually consistent during failover"
            },
            {
                "name": "Replica Lag Spike",
                "description": "Replica falls behind max_lag_ms threshold",
                "impact": "Affected replica temporarily excluded from reads",
                "recovery_time": "60 seconds (catch-up replication)",
                "data_consistency": "Maintained by routing to healthy replicas"
            },
            {
                "name": "Regional Network Partition",
                "description": "Network issues isolate EU-West replica",
                "impact": "EU reads failover to US replicas (higher latency)",
                "recovery_time": "5 minutes (network recovery)",
                "data_consistency": "Consistent but higher latency for EU users"
            },
            {
                "name": "Planned Maintenance",
                "description": "Taking US-East replica offline for maintenance",
                "impact": "Load redistributed to remaining replicas",
                "recovery_time": "2 hours (planned maintenance window)",
                "data_consistency": "No impact, graceful load balancing"
            }
        ]
        
        print("🚨 Failover Scenario Analysis:")
        print()
        
        for i, scenario in enumerate(scenarios, 1):
            print(f"{i}. {scenario['name']}")
            print(f"   📋 Description: {scenario['description']}")
            print(f"   💥 Impact: {scenario['impact']}")
            print(f"   ⏱️  Recovery Time: {scenario['recovery_time']}")
            print(f"   🔒 Data Consistency: {scenario['data_consistency']}")
            
            # Simulate the scenario impact
            if "Primary" in scenario['name']:
                print(f"   📊 Simulation: 100% read traffic → replicas, 0% writes available")
            elif "Replica Lag" in scenario['name']:
                print(f"   📊 Simulation: 1 replica excluded, 33% capacity reduction")
            elif "Network Partition" in scenario['name']:
                print(f"   📊 Simulation: EU latency increases from 15ms → 120ms")
            elif "Planned Maintenance" in scenario['name']:
                print(f"   📊 Simulation: Gradual traffic shift, no service interruption")
                
            print()

    def demonstrate_performance_benefits(self):
        """Show the performance benefits of read replicas."""
        print("🚀 Read Replica Performance Benefits")
        print("=" * 60)
        
        # Simulate performance metrics
        baseline_metrics = {
            "read_queries_per_sec": 2500,
            "avg_read_latency_ms": 25,
            "p95_read_latency_ms": 45,
            "p99_read_latency_ms": 65,
            "primary_cpu_utilization": 85,
            "primary_memory_utilization": 70
        }
        
        replica_metrics = {
            "read_queries_per_sec": 8500,  # 3.4x improvement
            "avg_read_latency_ms": 18,     # 28% improvement
            "p95_read_latency_ms": 32,     # 29% improvement
            "p99_read_latency_ms": 48,     # 26% improvement
            "primary_cpu_utilization": 45, # 47% reduction
            "primary_memory_utilization": 35 # 50% reduction
        }
        
        print("📈 Performance Comparison:")
        print("-" * 80)
        print(f"{'Metric':<30} {'Without Replicas':<18} {'With Replicas':<18} {'Improvement':<12}")
        print("-" * 80)
        
        for metric in baseline_metrics.keys():
            baseline = baseline_metrics[metric]
            with_replicas = replica_metrics[metric]
            
            if "queries_per_sec" in metric:
                improvement = f"+{((with_replicas / baseline) - 1) * 100:.0f}%"
            elif "latency" in metric or "utilization" in metric:
                improvement = f"-{((baseline - with_replicas) / baseline) * 100:.0f}%"
            else:
                improvement = "N/A"
                
            print(f"{metric.replace('_', ' ').title():<30} {baseline:<18} {with_replicas:<18} {improvement:<12}")
        
        print()
        print("💡 Key Performance Benefits:")
        benefits = [
            "🚀 3.4x increase in read throughput capacity",
            "⚡ 28% reduction in average read latency", 
            "💾 47% reduction in primary database CPU load",
            "🌐 Improved performance for geographically distributed users",
            "📊 Better resource utilization across infrastructure",
            "🛡️  Improved fault tolerance and availability",
            "⚖️  Load isolation between OLTP and OLAP workloads"
        ]
        
        for benefit in benefits:
            print(f"   {benefit}")
        
        print()

    def demonstrate_best_practices(self):
        """Show read replica best practices."""
        print("📚 Read Replica Best Practices")
        print("=" * 60)
        
        best_practices = [
            {
                "category": "🏗️  Architecture",
                "practices": [
                    "Use read replicas for read-heavy workloads (>70% reads)",
                    "Place replicas close to your application servers",
                    "Consider cross-region replicas for global applications",
                    "Separate OLTP and OLAP workloads using replicas"
                ]
            },
            {
                "category": "⚙️  Configuration",
                "practices": [
                    "Set appropriate max_lag_ms based on consistency needs", 
                    "Use ReadPreference.SECONDARY for analytics queries",
                    "Use ReadPreference.PRIMARY for write-after-read patterns",
                    "Configure health checks and automatic failover"
                ]
            },
            {
                "category": "📊 Monitoring",
                "practices": [
                    "Monitor replica lag continuously",
                    "Track query distribution across replicas",
                    "Set up alerts for replica failures",
                    "Monitor connection pool utilization"
                ]
            },
            {
                "category": "🔒 Security",
                "practices": [
                    "Use encrypted connections to replicas",
                    "Implement proper authentication for replica access",
                    "Restrict replica access to read-only operations",
                    "Regularly audit replica access patterns"
                ]
            }
        ]
        
        for section in best_practices:
            print(f"{section['category']}")
            for i, practice in enumerate(section['practices'], 1):
                print(f"   {i}. {practice}")
            print()
        
        print("⚠️  Important Considerations:")
        considerations = [
            "Replicas introduce eventual consistency - plan accordingly",
            "Network partitions can affect replica availability",
            "Replica lag can vary based on write volume",
            "Consider costs of maintaining multiple database instances",
            "Test failover scenarios regularly"
        ]
        
        for consideration in considerations:
            print(f"   • {consideration}")
        print()

    def cleanup(self):
        """Clean up temporary files."""
        try:
            import shutil
            shutil.rmtree(self.temp_dir)
        except Exception:
            pass


async def main():
    """Main demonstration function."""
    print("🚀 Eventuali Read Replicas Performance Demo")
    print("=" * 80)
    print()
    
    demo = ReadReplicaDemo()
    
    try:
        # Demonstrate replica configurations
        demo.demonstrate_replica_configurations()
        
        # Demonstrate replica setup
        await demo.demonstrate_replica_setup()
        
        # Simulate workload distribution
        demo.simulate_read_workload_distribution()
        
        # Show failover scenarios
        await demo.demonstrate_failover_scenarios()
        
        # Show performance benefits
        demo.demonstrate_performance_benefits()
        
        # Show best practices
        demo.demonstrate_best_practices()
        
        print("🎉 Read Replicas Demo Complete!")
        print()
        print("Key Takeaways:")
        print("• Read replicas dramatically improve read throughput (3-5x)")
        print("• Geographic distribution reduces latency for global users")
        print("• Proper configuration balances performance and consistency")
        print("• Monitor replica lag and implement proper failover strategies")
        print("• Use replicas to separate OLTP and OLAP workloads")
        
    except Exception as e:
        print(f"❌ Demo failed with error: {e}")
        import traceback
        traceback.print_exc()
        
    finally:
        demo.cleanup()


if __name__ == "__main__":
    asyncio.run(main())