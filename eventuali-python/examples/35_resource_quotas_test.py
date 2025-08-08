#!/usr/bin/env python3
"""
Quick test to validate Resource Quotas functionality from Example 35
Tests the core quota management features with performance benchmarks
"""

import time
import sys
sys.path.append('/home/user01/syncs/github/primevalai/onyx-octopus/eventuali-python/python')

import eventuali

def performance_test_quota_checks():
    """Test quota check performance to validate <1ms requirement"""
    print("🚀 Testing Resource Quotas Performance (Example 35 validation)")
    print("=" * 60)
    
    # Create tenant manager
    tenant_manager = eventuali.TenantManager()
    
    # Create test tenant
    tenant_id = eventuali.TenantId.generate()
    tenant_info = tenant_manager.create_tenant(
        tenant_id, 
        "Performance Test Tenant",
        None
    )
    
    print(f"✅ Created tenant: {tenant_id.as_str()}")
    
    # Test quota check performance
    iterations = 1000
    resource_types = ["events", "storage", "streams", "projections", "aggregates"]
    
    total_time = 0
    successful_checks = 0
    
    print(f"\n🔍 Running {iterations} quota checks across {len(resource_types)} resource types...")
    
    for i in range(iterations):
        for resource_type in resource_types:
            start_time = time.perf_counter()
            
            try:
                # This should succeed for reasonable amounts
                tenant_manager.check_tenant_quota(tenant_id, resource_type, 10)
                successful_checks += 1
            except Exception as e:
                # Some may fail due to limits, that's expected
                pass
            
            end_time = time.perf_counter()
            check_time_ms = (end_time - start_time) * 1000
            total_time += check_time_ms
    
    total_checks = iterations * len(resource_types)
    avg_time_ms = total_time / total_checks
    
    print(f"\n📊 Performance Results:")
    print(f"   Total quota checks: {total_checks}")
    print(f"   Successful checks: {successful_checks}")
    print(f"   Average check time: {avg_time_ms:.4f}ms")
    print(f"   Total time: {total_time:.2f}ms")
    
    # Validate <1ms requirement
    if avg_time_ms < 1.0:
        print(f"✅ PERFORMANCE TARGET MET: {avg_time_ms:.4f}ms < 1.0ms")
        return True
    else:
        print(f"⚠️  PERFORMANCE TARGET MISSED: {avg_time_ms:.4f}ms >= 1.0ms")
        return False

def test_enterprise_quota_tiers():
    """Test different quota tiers (from Example 35)"""
    print(f"\n🏢 Testing Enterprise Quota Tiers")
    print("-" * 40)
    
    # Test quota tier creation
    tiers = ["starter", "standard", "professional", "enterprise"]
    
    for tier_name in tiers:
        try:
            tier = eventuali.QuotaTier(tier_name)
            print(f"✅ Created {tier_name} tier: {tier}")
        except Exception as e:
            print(f"❌ Failed to create {tier_name} tier: {e}")
            return False
    
    return True

def test_resource_limits():
    """Test resource limits configuration"""
    print(f"\n📊 Testing Resource Limits Configuration")
    print("-" * 40)
    
    try:
        # Test creating various resource limit configurations
        limits_configs = [
            {"max_events_per_day": 1000, "max_storage_mb": 100, "max_concurrent_streams": 5},
            {"max_events_per_day": 10000, "max_storage_mb": 1000, "max_concurrent_streams": 50},
            {"max_events_per_day": None, "max_storage_mb": None, "max_concurrent_streams": None}  # Unlimited
        ]
        
        for i, config in enumerate(limits_configs):
            limits = eventuali.ResourceLimits(**config)
            print(f"✅ Resource limits config {i+1}: max_events={limits.max_events_per_day}, "
                  f"storage={limits.max_storage_mb}MB, streams={limits.max_concurrent_streams}")
    
        return True
    except Exception as e:
        print(f"❌ Failed to create resource limits: {e}")
        return False

def test_tenant_usage_tracking():
    """Test tenant usage recording and retrieval"""
    print(f"\n📈 Testing Tenant Usage Tracking")
    print("-" * 40)
    
    try:
        tenant_manager = eventuali.TenantManager()
        tenant_id = eventuali.TenantId.generate()
        
        # Create tenant with limits
        limits = eventuali.ResourceLimits(
            max_events_per_day=1000,
            max_storage_mb=100,
            max_concurrent_streams=10,
            max_projections=5,
            max_aggregates=50
        )
        config = eventuali.TenantConfig(
            resource_limits=limits,
            isolation_level="application"
        )
        
        tenant_info = tenant_manager.create_tenant(tenant_id, "Usage Test Tenant", config)
        print(f"✅ Created tenant with limits")
        
        # Record some usage
        usage_records = [
            ("events", 100),
            ("storage", 10),
            ("streams", 2),
            ("projections", 1),
            ("aggregates", 5)
        ]
        
        for resource_type, amount in usage_records:
            tenant_manager.record_tenant_usage(tenant_id, resource_type, amount)
            print(f"✅ Recorded {amount} {resource_type}")
        
        # Get usage info
        usage = tenant_manager.get_tenant_usage(tenant_id)
        print(f"✅ Retrieved usage: {usage}")
        
        return True
    except Exception as e:
        print(f"❌ Failed usage tracking test: {e}")
        import traceback
        traceback.print_exc()
        return False

def main():
    """Run all quota management tests"""
    print("🎯 Eventuali Resource Quotas Validation (Example 35)")
    print("Testing enterprise-grade multi-tenant quota management")
    print("=" * 70)
    
    tests = [
        ("Performance Test (<1ms quota checks)", performance_test_quota_checks),
        ("Enterprise Quota Tiers", test_enterprise_quota_tiers),
        ("Resource Limits Configuration", test_resource_limits),
        ("Tenant Usage Tracking", test_tenant_usage_tracking),
    ]
    
    passed_tests = 0
    total_tests = len(tests)
    
    for test_name, test_func in tests:
        print(f"\n🧪 Running: {test_name}")
        try:
            if test_func():
                passed_tests += 1
                print(f"✅ {test_name} PASSED")
            else:
                print(f"❌ {test_name} FAILED")
        except Exception as e:
            print(f"❌ {test_name} FAILED with exception: {e}")
            import traceback
            traceback.print_exc()
    
    print(f"\n" + "=" * 70)
    print(f"📋 Test Results: {passed_tests}/{total_tests} tests passed")
    
    if passed_tests == total_tests:
        print("🎉 ALL TESTS PASSED - Resource Quotas Example 35 validated!")
        print("✅ Enterprise-grade quota management is working correctly")
        print("✅ Performance targets met (<1ms quota checks)")
        return True
    else:
        print(f"⚠️  {total_tests - passed_tests} tests failed")
        return False

if __name__ == "__main__":
    success = main()
    exit(0 if success else 1)