# Eventuali CLI Documentation

The Eventuali CLI is a comprehensive command-line interface for managing event sourcing operations, including database initialization, migrations, querying, replay, and performance benchmarking.

## Installation

The CLI is automatically installed when you install the Eventuali package:

```bash
cd eventuali-python
uv sync
uv run maturin develop
```

The `eventuali` command will be available via UV. Test it:

```bash
# Test the CLI is working
uv run eventuali --help
uv run eventuali config --list
```

**Important**: Always use `uv run eventuali` (not just `eventuali`) to ensure you're using the correct environment.

## Basic Usage

```bash
eventuali --help
```

All commands support verbose output with the `-v` or `--verbose` flag:

```bash
eventuali -v <command> [options]
```

## Configuration

The CLI uses a configuration file stored at `~/.eventuali/config.json` with the following default settings:

```json
{
  "database_url": "sqlite://:memory:",
  "migration_version": "1.0.0",
  "benchmark_duration": 10,
  "benchmark_events_per_second": 1000,
  "output_format": "table"
}
```

### Managing Configuration

```bash
# List current configuration
eventuali config --list

# Set a configuration value
eventuali config --key database_url --value "postgresql://user:pass@localhost/events"

# Get a specific configuration value
eventuali config --key database_url
```

## Commands

### init - Initialize Database

Initialize a new event store database with the required schema.

```bash
eventuali init [OPTIONS]
```

**Options:**
- `-d, --database-url TEXT`: Database URL (e.g., postgresql://user:pass@host/db)
- `-f, --force`: Force initialization even if database exists
- `--help`: Show help message

**Examples:**

```bash
# Initialize with SQLite in-memory database
eventuali init --database-url "sqlite://:memory:"

# Initialize with PostgreSQL
eventuali init --database-url "postgresql://user:password@localhost/eventuali"

# Force initialization (overwrites existing)
eventuali init --force
```

**What it does:**
- Creates the event store schema
- Tests basic operations (create/save/load)
- Updates CLI configuration with new database URL
- Provides verification that the setup is working correctly

### migrate - Schema Migrations

Run database schema migrations to upgrade the event store structure.

```bash
eventuali migrate [OPTIONS]
```

**Options:**
- `-v, --version TEXT`: Migration version to apply
- `-d, --database-url TEXT`: Database URL override
- `--help`: Show help message

**Examples:**

```bash
# Migrate to specific version
eventuali migrate --version 2.0.0

# Migrate using different database
eventuali migrate --database-url "postgresql://user:pass@host/db" --version 1.5.0
```

**What it does:**
- Applies schema changes for version compatibility
- Updates database structure for new features
- Maintains backward compatibility where possible
- Updates configuration with new migration version

### query - Event Stream Inspection

Query and inspect stored events with flexible filtering options.

```bash
eventuali query [OPTIONS]
```

**Options:**
- `-a, --aggregate-id TEXT`: Specific aggregate ID to query
- `-f, --from-version INTEGER`: Start from specific version
- `-t, --to-version INTEGER`: End at specific version
- `-l, --limit INTEGER`: Maximum number of events (default: 100)
- `-o, --output [table|json]`: Output format
- `-d, --database-url TEXT`: Database URL override
- `--help`: Show help message

**Examples:**

```bash
# Query all events (shows sample data)
eventuali query

# Query specific aggregate
eventuali query --aggregate-id user-123

# Query version range
eventuali query --aggregate-id user-123 --from-version 5 --to-version 10

# Limit results and use JSON output
eventuali query --limit 50 --output json

# Query from specific database
eventuali query --database-url "sqlite:///events.db" --aggregate-id order-456
```

**Output Example:**

```
+----------------+-----------+------------------+----------------------------+
| aggregate_id   |   version | event_type       | timestamp                  |
+================+===========+==================+============================+
| user-123       |         1 | UserRegistered   | 2024-03-15T10:30:00.000Z   |
+----------------+-----------+------------------+----------------------------+
| user-123       |         2 | EmailChanged     | 2024-03-15T11:15:30.000Z   |
+----------------+-----------+------------------+----------------------------+
| user-123       |         3 | UserDeactivated  | 2024-03-15T14:45:00.000Z   |
+----------------+-----------+------------------+----------------------------+
```

### replay - Event Replay and Projection Rebuilding

Replay events from specific positions to rebuild projections or recover state.

```bash
eventuali replay [OPTIONS]
```

**Options:**
- `-p, --projection TEXT`: Projection name to rebuild
- `-f, --from-position INTEGER`: Start replay from specific position
- `-a, --aggregate-id TEXT`: Replay events for specific aggregate
- `-d, --database-url TEXT`: Database URL override
- `--help`: Show help message

**Examples:**

```bash
# Global replay (all events)
eventuali replay

# Rebuild specific projection
eventuali replay --projection user-analytics

# Replay specific aggregate from position
eventuali replay --aggregate-id user-123 --from-position 5

# Replay with custom database
eventuali replay --database-url "postgresql://user:pass@host/db" --projection sales-summary
```

**What it does:**
- Replays events in chronological order
- Rebuilds projections with fresh data
- Supports partial replay from specific positions
- Shows progress with visual progress bars
- Validates event consistency during replay

### benchmark - Performance Testing

Run comprehensive performance benchmarks against the event store.

```bash
eventuali benchmark [OPTIONS]
```

**Options:**
- `-d, --duration INTEGER`: Benchmark duration in seconds
- `-r, --events-per-second INTEGER`: Target events per second
- `-ops, --operations [create|persist|load|all]`: Operations to benchmark (default: all)
- `-o, --output [table|json]`: Output format
- `-d_url, --database-url TEXT`: Database URL override
- `--help`: Show help message

**Examples:**

```bash
# Run full benchmark suite
eventuali benchmark

# Quick 30-second benchmark
eventuali benchmark --duration 30 --events-per-second 5000

# Test only event creation
eventuali benchmark --operations create --duration 10

# Benchmark against PostgreSQL
eventuali benchmark --database-url "postgresql://user:pass@localhost/test" --duration 60

# Get JSON results for analysis
eventuali benchmark --output json --duration 15
```

**Benchmark Results Example:**

```
Benchmark Results:
+-------------------+-------------------+---------------+----------------------+--------------+---------------+
| operation         |   duration_seconds |   total_events |   events_per_second |   target_eps | efficiency    |
+===================+===================+===============+======================+==============+===============+
| Event Creation    |               10.0 |          8543 |              854.30 |         1000 | 85.4%         |
+-------------------+-------------------+---------------+----------------------+--------------+---------------+
| Event Persistence|               10.0 |           127 |               12.70 |          100 | 127.0%        |
+-------------------+-------------------+---------------+----------------------+--------------+---------------+
| Event Loading     |               10.0 |           456 |               45.60 |          200 | 22.8%         |
+-------------------+-------------------+---------------+----------------------+--------------+---------------+

Benchmark Summary:
┏━━━━━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Metric                   ┃ Value                   ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━╇━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ Total Events Processed   │ 9126                    │
│ Average Efficiency       │ 78.4%                   │
│ Database Backend         │ SQLITE                  │
│ Benchmark Duration       │ 10s                     │
└──────────────────────────┴─────────────────────────┘
```

## Database Support

The CLI supports multiple database backends:

### SQLite

```bash
# In-memory (for testing)
eventuali init --database-url "sqlite://:memory:"

# File-based
eventuali init --database-url "sqlite:///path/to/events.db"
```

### PostgreSQL

```bash
# Basic connection
eventuali init --database-url "postgresql://username:password@localhost/eventuali"

# With custom port and options
eventuali init --database-url "postgresql://user:pass@localhost:5433/events?sslmode=require"
```

## Performance Optimization Tips

### For SQLite
- Use file-based SQLite for persistence: `sqlite:///events.db`
- Enable WAL mode for better concurrency
- Consider periodic VACUUM operations

### For PostgreSQL
- Use connection pooling for high-throughput scenarios
- Optimize `work_mem` and `shared_buffers` settings
- Consider partitioning for very large event tables

## Common Workflows

### Development Setup

```bash
# 1. Initialize development database
eventuali init --database-url "sqlite:///dev_events.db"

# 2. Run benchmarks to validate performance
eventuali benchmark --duration 30

# 3. Query events during development
eventuali query --limit 20
```

### Production Deployment

```bash
# 1. Initialize production database
eventuali init --database-url "postgresql://user:pass@prod-db/events"

# 2. Run migrations for schema updates
eventuali migrate --version 2.1.0

# 3. Benchmark production performance
eventuali benchmark --duration 120 --events-per-second 10000

# 4. Set up monitoring queries
eventuali query --from-version 1000 --limit 100 --output json > recent_events.json
```

### Disaster Recovery

```bash
# 1. Replay all events to rebuild projections
eventuali replay --from-position 0

# 2. Rebuild specific projection
eventuali replay --projection user-analytics

# 3. Verify data integrity
eventuali query --limit 1000 --output json | jq '.[] | select(.version == null)'
```

## Troubleshooting

### Connection Issues

```bash
# Test database connection
eventuali init --database-url "your-db-url" --force

# Check configuration
eventuali config --list
```

### Performance Issues

```bash
# Run diagnostics
eventuali benchmark --operations all --duration 60

# Check for slow operations
eventuali benchmark --operations persist --duration 30
```

### Data Issues

```bash
# Inspect specific aggregate
eventuali query --aggregate-id problematic-id

# Check event consistency
eventuali replay --aggregate-id problematic-id --from-position 0
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Eventuali CLI Tests
on: [push, pull_request]

jobs:
  test-cli:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Eventuali
        run: |
          cd eventuali-python
          uv sync
          uv run maturin develop
      
      - name: Test CLI Commands
        run: |
          uv run eventuali init --database-url "sqlite://:memory:"
          uv run eventuali benchmark --duration 10 --operations create
          uv run eventuali query --limit 5
```

### Docker Integration

```dockerfile
FROM python:3.11-slim

# Install Eventuali
COPY eventuali-python /app/eventuali-python
WORKDIR /app/eventuali-python
RUN pip install uv && uv sync && uv run maturin develop

# CLI is now available
ENTRYPOINT ["uv", "run", "eventuali"]
```

## Advanced Configuration

### Environment Variables

The CLI respects these environment variables:

- `EVENTUALI_DATABASE_URL`: Default database URL
- `EVENTUALI_LOG_LEVEL`: Logging level (DEBUG, INFO, WARNING, ERROR)
- `EVENTUALI_CONFIG_DIR`: Custom configuration directory

### Custom Configuration File

```bash
# Use custom config file
eventuali --config /path/to/custom-config.json init
```

### Batch Operations

```bash
# Run multiple commands in sequence
eventuali init --database-url "sqlite:///batch.db" && \
eventuali benchmark --duration 30 && \
eventuali replay --projection analytics
```

## API Integration

The CLI can be used programmatically:

```python
import subprocess
import json

# Run benchmark and parse results
result = subprocess.run(
    ['uv', 'run', 'eventuali', 'benchmark', '--output', 'json', '--duration', '10'],
    capture_output=True,
    text=True
)

benchmark_data = json.loads(result.stdout)
print(f"Events per second: {benchmark_data[0]['events_per_second']}")
```

## Version Information

```bash
# Check CLI version
eventuali --version

# Get detailed version information
eventuali config --list | grep version
```

## Support and Development

- **Source Code**: https://github.com/primevalai/eventuali
- **Issues**: Report bugs and feature requests via GitHub Issues
- **Documentation**: Additional docs available in the repository
- **Examples**: See the `examples/` directory for usage patterns

The Eventuali CLI provides a powerful interface for managing event sourcing operations from development through production deployment. Its comprehensive feature set supports the full lifecycle of event-driven applications.