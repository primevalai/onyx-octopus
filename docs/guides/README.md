# Integration Guides

**Step-by-step tutorials for integrating Eventuali into your applications**

These guides use working examples from the `examples/` directory to demonstrate real-world integration patterns with proven code.

## 🚀 Quick Start Guides

### [Quick Start](quick-start.md)
**5-minute introduction to Eventuali**
- Basic event sourcing concepts
- Your first EventStore
- Simple aggregate creation
- Event persistence and loading

*Based on: [`examples/01_basic_event_store_simple.py`](../../examples/01_basic_event_store_simple.py)*

### [Migration Guide](migration-guide.md)
**Moving from pure Python event sourcing**
- Performance comparison benchmarks
- API migration patterns
- Data migration strategies
- Common pitfalls and solutions

*Based on: [`examples/04_performance_testing.py`](../../examples/04_performance_testing.py)*

## 🔧 Framework Integration

### [FastAPI Integration](fastapi-integration.md)
**Building REST APIs with event sourcing**
- Dependency injection patterns
- Request/response handling
- Error handling and validation
- Background task processing

*Using patterns from: [`examples/12_microservices_integration.py`](../../examples/12_microservices_integration.py)*

### [Django Integration](django-integration.md)
**Integrating with Django applications**
- Model layer integration
- Custom management commands
- Signal handling patterns
- Admin interface integration

### [Flask Integration](flask-integration.md)
**Lightweight web applications**
- Application factory patterns
- Blueprint organization
- Error handling
- Testing strategies

## 📊 Data Science Integration

### [Pandas Integration](pandas-integration.md)
**Event analytics with Pandas**
- Event stream to DataFrame conversion
- Time-series analysis patterns
- Performance optimization
- Visualization examples

*Based on: [`examples/08_projections.py`](../../examples/08_projections.py)*

### [Jupyter Notebook Integration](jupyter-integration.md)
**Interactive event sourcing exploration**
- Notebook setup and configuration
- Event exploration patterns
- Visualization techniques
- Reproducible analysis

### [Dask Integration](dask-integration.md)
**Distributed event processing**
- Parallel event processing
- Large dataset handling
- Scaling patterns
- Performance considerations

## 🏗️ Architecture Patterns

### [CQRS Implementation](cqrs-implementation.md)
**Command Query Responsibility Segregation**
- Command handling patterns
- Query optimization
- Read model projections
- Eventual consistency

*Based on: [`examples/09_cqrs_patterns.py`](../../examples/09_cqrs_patterns.py)*

### [Saga Pattern Implementation](saga-implementation.md)
**Distributed transaction management**
- Orchestration patterns
- Compensation handling
- State machine design
- Error recovery

*Based on: [`examples/07_saga_patterns.py`](../../examples/07_saga_patterns.py)*

### [Event Streaming](event-streaming.md)
**Real-time event processing**
- Stream setup and configuration
- Projection building
- Error handling
- Performance tuning

*Based on: [`examples/08_projections.py`](../../examples/08_projections.py)*

## 🔐 Security & Compliance

### [Security Implementation](security-implementation.md)
**Enterprise security patterns**
- Event encryption at rest
- Role-based access control
- Audit trail implementation
- Security best practices

*Based on: [`examples/22_event_encryption_at_rest.py`](../../examples/22_event_encryption_at_rest.py)*

### [Multi-tenant Architecture](multi-tenant-architecture.md)
**Building SaaS applications**
- Tenant isolation strategies
- Resource quota management
- Configuration management
- Monitoring and metrics

*Based on: [`examples/30_tenant_isolation_architecture.py`](../../examples/30_tenant_isolation_architecture.py)*

### [GDPR Compliance](gdpr-compliance.md)
**Data privacy and compliance**
- Data subject rights
- Consent management
- Data retention policies
- Breach notification

*Based on: [`examples/27_gdpr_compliance.py`](../../examples/27_gdpr_compliance.py)*

## 📈 Production Deployment

### [Production Deployment](production-deployment.md)
**Going live with Eventuali**
- Environment configuration
- Database setup
- Monitoring implementation
- Health checks

*Based on: [`examples/14_production_monitoring.py`](../../examples/14_production_monitoring.py)*

### [Performance Optimization](performance-optimization.md)
**Maximizing throughput and efficiency**
- Connection pooling
- Batch processing
- Caching strategies
- Monitoring and profiling

*Based on: [`examples/44_connection_pooling_performance.py`](../../examples/44_connection_pooling_performance.py)*

### [Monitoring & Observability](monitoring-observability.md)
**Production monitoring setup**
- OpenTelemetry integration
- Prometheus metrics
- Grafana dashboards
- Alert configuration

*Based on: [`examples/37_opentelemetry_integration.py`](../../examples/37_opentelemetry_integration.py)*

## 🧪 Testing Strategies

### [Testing Guide](testing-guide.md)
**Comprehensive testing approaches**
- Unit testing events and aggregates
- Integration testing with EventStore
- Performance testing
- Property-based testing

*Based on: [`examples/03_error_handling.py`](../../examples/03_error_handling.py)*

### [Test Fixtures & Utilities](test-fixtures.md)
**Reusable testing components**
- Event builders
- Aggregate factories
- Store mocking
- Performance helpers

## 🔄 Advanced Patterns

### [Event Versioning](event-versioning.md)
**Schema evolution and backward compatibility**
- Versioning strategies
- Migration patterns
- Upcasting techniques
- Breaking change management

*Based on: [`examples/06_event_versioning.py`](../../examples/06_event_versioning.py)*

### [Snapshot Optimization](snapshot-optimization.md)
**Performance optimization with snapshots**
- Snapshot strategies
- Compression techniques
- Cleanup policies
- Performance analysis

*Based on: [`examples/21_snapshots.py`](../../examples/21_snapshots.py)*

### [Distributed Events](distributed-events.md)
**Multi-node coordination and consensus**
- Consensus protocols
- Failover strategies
- Conflict resolution
- Network partition handling

*Based on: [`examples/11_distributed_events.py`](../../examples/11_distributed_events.py)*

## 📋 Guide Categories

### 🟢 **Beginner Guides**
- [Quick Start](quick-start.md) - Essential concepts
- [Migration Guide](migration-guide.md) - Moving from other libraries
- [Testing Guide](testing-guide.md) - Testing fundamentals

### 🟡 **Intermediate Guides**  
- [FastAPI Integration](fastapi-integration.md) - Web API patterns
- [CQRS Implementation](cqrs-implementation.md) - Architecture patterns
- [Security Implementation](security-implementation.md) - Enterprise security

### 🔴 **Advanced Guides**
- [Multi-tenant Architecture](multi-tenant-architecture.md) - SaaS patterns
- [Distributed Events](distributed-events.md) - Multi-node systems
- [Performance Optimization](performance-optimization.md) - Production tuning

## 📊 Performance Expectations

All integration patterns are validated with real benchmarks:

| Pattern | Throughput | Latency | Example |
|---------|------------|---------|---------|
| **Basic CRUD** | 79k+ events/sec | <1ms | [Quick Start](quick-start.md) |
| **REST API** | 25k+ requests/sec | <5ms | [FastAPI](fastapi-integration.md) |
| **Streaming** | 78k+ events/sec | <1ms | [Event Streaming](event-streaming.md) |
| **CQRS** | 40k+ commands/sec | <2ms | [CQRS](cqrs-implementation.md) |
| **Sagas** | ~214ms average | - | [Saga Pattern](saga-implementation.md) |

## 🛠️ Prerequisites

All guides assume you have:

1. **UV installed** for Python dependency management
2. **Rust toolchain** (for building from source)
3. **Database access** (PostgreSQL or SQLite)
4. **Python 3.8+** with async/await support

### Quick Setup

```bash
# Install UV and setup project
curl -LsSf https://astral.sh/uv/install.sh | sh
cd your-project
uv init

# Install Eventuali
uv add eventuali

# Verify installation
uv run python -c "import eventuali; print('✅ Ready to go!')"
```

## 🔗 Related Documentation

- **[API Reference](../api/README.md)** - Complete API documentation
- **[Architecture Guide](../architecture/README.md)** - System design deep dive
- **[Examples](../../examples/README.md)** - 46+ working examples
- **[Performance Guide](../performance/README.md)** - Optimization strategies

---

**Next**: Start with the [Quick Start Guide](quick-start.md) or explore specific [framework integrations](#-framework-integration).