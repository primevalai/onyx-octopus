# Deployment Guide

**Production deployment strategies for Eventuali applications**

This guide covers deploying Eventuali applications from development to production, including database setup, monitoring, and scaling patterns.

## 🚀 Deployment Overview

### Supported Deployment Platforms

| Platform | Support Level | Configuration |
|----------|---------------|---------------|
| **Docker** | ✅ Full Support | Container deployment with multi-stage builds |
| **Kubernetes** | ✅ Full Support | Helm charts and operators available |
| **AWS** | ✅ Production Ready | ECS, EKS, Lambda, RDS integration |
| **Azure** | ✅ Production Ready | AKS, Container Instances, PostgreSQL |
| **GCP** | ✅ Production Ready | GKE, Cloud Run, Cloud SQL |
| **Digital Ocean** | ✅ Supported | App Platform, Managed Databases |
| **Railway** | ✅ Supported | Simple deployment with PostgreSQL |

### Architecture Patterns

```
┌─────────────────────────────────────────────────────────┐
│                 Production Architecture                 │
├─────────────────────────────────────────────────────────┤
│  Load Balancer    │  Application Tier  │  Database Tier │
│  - NGINX/HAProxy  │  - FastAPI/Django  │  - PostgreSQL  │
│  - SSL/TLS        │  - Event Store     │  - Read Replicas│
│  - Rate Limiting  │  - Projections     │  - Backup/DR   │
└─────────────────────────────────────────────────────────┘
```

## 📦 Package Installation

### Production Installation

```bash
# Install from PyPI (when available)
pip install eventuali

# Or with UV (recommended)
uv add eventuali

# Verify installation
python -c "import eventuali; print('✅ Eventuali installed')"
```

### Platform Support

**Supported Python Versions:** 3.8, 3.9, 3.10, 3.11, 3.12

**Supported Platforms:**
- **Linux**: x86_64, aarch64 (manylinux2014+)
- **macOS**: x86_64, Apple Silicon (M1/M2)
- **Windows**: x86_64

**Installation Methods:**
- **Wheels**: Instant installation (pre-compiled)
- **Source**: ~2-3 minutes (requires Rust toolchain)
- **Size**: ~15-25MB (includes Rust binary)

*Based on: [PUBLISHING.md](../../PUBLISHING.md)*

## 🔧 Environment Configuration

### Development Environment

```bash
# .env.development
EVENTUALI_DATABASE_URL=sqlite://events_dev.db
EVENTUALI_LOG_LEVEL=DEBUG
EVENTUALI_PERFORMANCE_MONITORING=true
EVENTUALI_CACHE_ENABLED=false
EVENTUALI_ENCRYPTION_ENABLED=false
```

### Staging Environment

```bash
# .env.staging
EVENTUALI_DATABASE_URL=postgresql://user:pass@staging-db/events
EVENTUALI_LOG_LEVEL=INFO
EVENTUALI_PERFORMANCE_MONITORING=true
EVENTUALI_CACHE_ENABLED=true
EVENTUALI_ENCRYPTION_ENABLED=true
EVENTUALI_SNAPSHOT_FREQUENCY=100
```

### Production Environment

```bash
# .env.production
EVENTUALI_DATABASE_URL=postgresql://user:pass@prod-db/events
EVENTUALI_LOG_LEVEL=WARN
EVENTUALI_PERFORMANCE_MONITORING=true
EVENTUALI_CACHE_ENABLED=true
EVENTUALI_ENCRYPTION_ENABLED=true
EVENTUALI_SNAPSHOT_FREQUENCY=50
EVENTUALI_REPLICA_URLS=postgresql://user:pass@replica1/events,postgresql://user:pass@replica2/events
EVENTUALI_MONITORING_ENDPOINT=http://prometheus:9090/metrics
```

## 🐳 Docker Deployment

### Multi-Stage Dockerfile

Create `Dockerfile`:

```dockerfile
# Build stage
FROM python:3.11-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \\
    build-essential \\
    curl \\
    && rm -rf /var/lib/apt/lists/*

# Install Rust (required for building Eventuali)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install UV for dependency management
RUN pip install uv

# Copy application code
WORKDIR /app
COPY . .

# Install dependencies and build
RUN uv sync --frozen
RUN uv run pip install eventuali

# Production stage
FROM python:3.11-slim as production

# Install runtime dependencies
RUN apt-get update && apt-get install -y \\
    libpq5 \\
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd --create-home --shell /bin/bash app

# Copy application from builder
COPY --from=builder /app /app
COPY --from=builder /root/.local /home/app/.local

# Set ownership and switch to non-root user
RUN chown -R app:app /app /home/app
USER app
WORKDIR /app

# Set environment
ENV PATH="/home/app/.local/bin:$PATH"
ENV PYTHONPATH="/app"

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \\
    CMD python -c "import eventuali; print('OK')" || exit 1

# Default command
CMD ["python", "-m", "uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]
```

### Docker Compose for Development

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "8000:8000"
    environment:
      - EVENTUALI_DATABASE_URL=postgresql://postgres:password@db:5432/events
      - EVENTUALI_LOG_LEVEL=DEBUG
    depends_on:
      - db
      - redis
    volumes:
      - ./logs:/app/logs
    restart: unless-stopped

  db:
    image: postgres:15
    environment:
      POSTGRES_DB: events
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    restart: unless-stopped

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    restart: unless-stopped

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana:/etc/grafana/provisioning
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
  prometheus_data:
  grafana_data:
```

### Production Docker Compose

Create `docker-compose.prod.yml`:

```yaml
version: '3.8'

services:
  app:
    build:
      context: .
      target: production
    deploy:
      replicas: 3
      resources:
        limits:
          memory: 512M
          cpus: '0.5'
        reservations:
          memory: 256M
          cpus: '0.25'
    environment:
      - EVENTUALI_DATABASE_URL=postgresql://user:pass@db:5432/events
      - EVENTUALI_LOG_LEVEL=WARN
      - EVENTUALI_ENCRYPTION_ENABLED=true
    secrets:
      - db_password
      - encryption_key
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.eventuali.rule=Host(`api.example.com`)"
      - "traefik.http.routers.eventuali.tls=true"
    restart: unless-stopped

secrets:
  db_password:
    external: true
  encryption_key:
    external: true
```

## ☸️ Kubernetes Deployment

### Deployment Manifest

Create `k8s/deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: eventuali-app
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
        image: your-registry/eventuali-app:latest
        ports:
        - containerPort: 8000
        env:
        - name: EVENTUALI_DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: eventuali-secrets
              key: database-url
        - name: EVENTUALI_ENCRYPTION_KEY
          valueFrom:
            secretKeyRef:
              name: eventuali-secrets
              key: encryption-key
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8000
          initialDelaySeconds: 5
          periodSeconds: 5
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
      targetPort: 8000
  type: ClusterIP
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: eventuali-ingress
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
  - hosts:
    - api.example.com
    secretName: eventuali-tls
  rules:
  - host: api.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: eventuali-service
            port:
              number: 80
```

### ConfigMap and Secrets

Create `k8s/config.yaml`:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: eventuali-config
data:
  EVENTUALI_LOG_LEVEL: "INFO"
  EVENTUALI_PERFORMANCE_MONITORING: "true"
  EVENTUALI_CACHE_ENABLED: "true"
  EVENTUALI_SNAPSHOT_FREQUENCY: "50"
---
apiVersion: v1
kind: Secret
metadata:
  name: eventuali-secrets
type: Opaque
data:
  database-url: <base64-encoded-database-url>
  encryption-key: <base64-encoded-encryption-key>
```

### Helm Chart

Create `helm/Chart.yaml`:

```yaml
apiVersion: v2
name: eventuali
description: A Helm chart for Eventuali event sourcing applications
type: application
version: 0.1.0
appVersion: "0.1.1"
```

Create `helm/values.yaml`:

```yaml
replicaCount: 3

image:
  repository: your-registry/eventuali-app
  tag: latest
  pullPolicy: IfNotPresent

service:
  type: ClusterIP
  port: 80
  targetPort: 8000

ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  hosts:
    - host: api.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: eventuali-tls
      hosts:
        - api.example.com

resources:
  limits:
    cpu: 500m
    memory: 512Mi
  requests:
    cpu: 250m
    memory: 256Mi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

database:
  host: postgresql.default.svc.cluster.local
  port: 5432
  name: events
  ssl: require

monitoring:
  enabled: true
  serviceMonitor:
    enabled: true
```

## 🗄️ Database Configuration

### PostgreSQL Production Setup

**Database Initialization:**

```sql
-- init.sql
CREATE DATABASE events;
CREATE USER eventuali WITH ENCRYPTED PASSWORD 'secure_password';
GRANT ALL PRIVILEGES ON DATABASE events TO eventuali;

\\c events

-- Create required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";

-- Create events table with optimization
CREATE TABLE IF NOT EXISTS events (
    event_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    aggregate_id TEXT NOT NULL,
    aggregate_type TEXT NOT NULL,
    event_type TEXT NOT NULL,
    event_data JSONB NOT NULL,
    event_version INTEGER NOT NULL DEFAULT 1,
    aggregate_version INTEGER NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    causation_id UUID,
    correlation_id UUID,
    user_id TEXT
);

-- Performance indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_events_aggregate_id 
    ON events(aggregate_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_events_aggregate_type 
    ON events(aggregate_type);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_events_timestamp 
    ON events(timestamp);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_events_correlation_id 
    ON events(correlation_id) WHERE correlation_id IS NOT NULL;

-- Partitioning for large datasets
CREATE TABLE events_y2024m01 PARTITION OF events
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
-- Add more partitions as needed
```

**PostgreSQL Configuration (`postgresql.conf`):**

```ini
# Memory settings
shared_buffers = 256MB
effective_cache_size = 1GB
work_mem = 16MB
maintenance_work_mem = 64MB

# WAL settings for performance
wal_buffers = 16MB
checkpoint_completion_target = 0.7
wal_writer_delay = 200ms

# Connection settings
max_connections = 200
shared_preload_libraries = 'pg_stat_statements'

# Logging
log_statement = 'mod'
log_min_duration_statement = 1000
log_checkpoints = on
log_lock_waits = on
```

### Read Replica Setup

**Primary Database Configuration:**

```bash
# Configure replication in postgresql.conf
wal_level = replica
max_wal_senders = 3
max_replication_slots = 3
synchronous_commit = on
```

**Replica Configuration:**

```python
from eventuali import EventStore
from eventuali.performance import ReplicaManager

# Configure read replicas
replica_config = {
    "primary": "postgresql://user:pass@primary:5432/events",
    "replicas": [
        "postgresql://user:pass@replica1:5432/events",
        "postgresql://user:pass@replica2:5432/events"
    ]
}

store = await EventStore.create_with_replicas(replica_config)
```

## 🔐 Security Configuration

### SSL/TLS Setup

**Database SSL Configuration:**

```python
# Secure database connection
DATABASE_URL = (
    "postgresql://user:pass@host:5432/events?"
    "sslmode=require&"
    "sslcert=/path/to/client-cert.pem&"
    "sslkey=/path/to/client-key.pem&"
    "sslrootcert=/path/to/ca-cert.pem"
)

store = await EventStore.create(DATABASE_URL)
```

### Event Encryption

**Production Encryption Setup:**

```python
from eventuali.security import EventEncryption, KeyManager

# Configure encryption
key_manager = KeyManager({
    "key_source": "environment",  # or "vault", "kms"
    "rotation_enabled": True,
    "rotation_interval": "30d"
})

encryption = EventEncryption(key_manager)

# Create encrypted event store
store = await EventStore.create(
    connection_string=DATABASE_URL,
    encryption=encryption
)
```

*Based on: [`examples/22_event_encryption_at_rest.py`](../../examples/22_event_encryption_at_rest.py)*

## 📊 Monitoring & Observability

### Prometheus Metrics

**Metrics Configuration (`monitoring/prometheus.yml`):**

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'eventuali'
    static_configs:
      - targets: ['app:8000']
    metrics_path: '/metrics'
    scrape_interval: 5s

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']
```

### Grafana Dashboards

**Dashboard Configuration:**

```python
# Automatic dashboard creation
from eventuali.monitoring import GrafanaDashboard

dashboard = GrafanaDashboard({
    "title": "Eventuali Production Metrics",
    "panels": [
        {
            "title": "Event Throughput",
            "query": "rate(eventuali_events_created_total[5m])",
            "type": "graph"
        },
        {
            "title": "Database Performance", 
            "query": "eventuali_database_query_duration_seconds",
            "type": "heatmap"
        },
        {
            "title": "Error Rate",
            "query": "rate(eventuali_errors_total[5m])",
            "type": "stat"
        }
    ]
})

await dashboard.create()
```

*Based on: [`examples/39_grafana_dashboard_creation.py`](../../examples/39_grafana_dashboard_creation.py)*

### Health Checks

**Application Health Endpoints:**

```python
from fastapi import FastAPI, HTTPException
from eventuali.monitoring import HealthChecker

app = FastAPI()
health_checker = HealthChecker()

@app.get("/health")
async def health_check():
    """Kubernetes liveness probe."""
    try:
        await health_checker.check_database()
        await health_checker.check_event_store()
        return {"status": "healthy", "timestamp": datetime.utcnow()}
    except Exception as e:
        raise HTTPException(status_code=503, detail=str(e))

@app.get("/ready")
async def readiness_check():
    """Kubernetes readiness probe."""
    checks = await health_checker.run_all_checks()
    if all(check.healthy for check in checks):
        return {"status": "ready", "checks": checks}
    else:
        raise HTTPException(status_code=503, detail="Not ready")
```

*Based on: [`examples/14_production_monitoring.py`](../../examples/14_production_monitoring.py)*

## 🚀 CI/CD Pipeline

### GitHub Actions

Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy to Production

on:
  push:
    tags:
      - 'v*'

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'
          
      - name: Install UV
        run: curl -LsSf https://astral.sh/uv/install.sh | sh
        
      - name: Install dependencies
        run: |
          cd eventuali-python
          uv sync
          uv tool install maturin
          
      - name: Build and test
        run: |
          cd eventuali-python
          uv run maturin develop
          uv run pytest tests/
          
      - name: Run examples
        run: |
          cd eventuali-python
          uv run python ../examples/01_basic_event_store_simple.py

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Build Docker image
        run: |
          docker build -t eventuali-app:${{ github.ref_name }} .
          
      - name: Push to registry
        run: |
          echo ${{ secrets.REGISTRY_PASSWORD }} | docker login -u ${{ secrets.REGISTRY_USERNAME }} --password-stdin
          docker push eventuali-app:${{ github.ref_name }}

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment: production
    steps:
      - name: Deploy to Kubernetes
        run: |
          echo "${{ secrets.KUBECONFIG }}" | base64 -d > kubeconfig
          export KUBECONFIG=kubeconfig
          
          helm upgrade --install eventuali ./helm \\
            --set image.tag=${{ github.ref_name }} \\
            --set database.password=${{ secrets.DB_PASSWORD }} \\
            --wait --timeout=300s
```

### Automated Testing

**Performance Regression Testing:**

```yaml
  performance-test:
    runs-on: ubuntu-latest
    steps:
      - name: Run performance benchmarks
        run: |
          cd eventuali-python
          uv run python ../examples/04_performance_testing.py > results.txt
          
      - name: Check performance regression
        run: |
          # Fail if throughput drops below threshold
          THROUGHPUT=$(grep "events/second" results.txt | awk '{print $1}')
          if [ "$THROUGHPUT" -lt 50000 ]; then
            echo "Performance regression detected: $THROUGHPUT events/sec"
            exit 1
          fi
```

## 📈 Scaling Strategies

### Horizontal Pod Autoscaling

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: eventuali-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: eventuali-app
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  - type: Pods
    pods:
      metric:
        name: eventuali_events_per_second
      target:
        type: AverageValue
        averageValue: "1000"
```

### Database Scaling

**Automatic Read Replica Scaling:**

```python
from eventuali.scaling import AutoScaler

# Configure automatic scaling
autoscaler = AutoScaler({
    "metrics": {
        "cpu_threshold": 70,
        "connection_threshold": 80,
        "query_latency_threshold": 100  # ms
    },
    "scaling": {
        "min_replicas": 2,
        "max_replicas": 8,
        "scale_up_cooldown": 300,   # 5 minutes
        "scale_down_cooldown": 600  # 10 minutes
    }
})

await autoscaler.start()
```

## 🔗 Related Documentation

- **[Performance Guide](../performance/README.md)** - Optimization strategies
- **[Architecture Guide](../architecture/README.md)** - System design
- **[Security Implementation](../guides/security-implementation.md)** - Security patterns
- **[Examples](../../examples/README.md)** - Production examples

---

**Next**: Set up [monitoring and observability](../guides/monitoring-observability.md) or explore [performance optimization](../performance/README.md) for your production deployment.