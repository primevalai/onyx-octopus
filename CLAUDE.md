# Eventuali Development Guide for Claude Code

This document provides essential context and development guidelines for working on the Eventuali project - a high-performance hybrid Python-Rust event sourcing library.

## Project Overview

Eventuali combines Rust's performance and memory safety with Python's developer experience to create a powerful event sourcing library. The project achieves 10-60x performance improvements over pure Python implementations while maintaining a Pythonic API.

**Architecture**:
- **Rust Core** (`eventuali-core`): High-performance event storage, serialization, and streaming
- **Python Bindings** (`eventuali-python`): PyO3-based bindings with Pydantic integration
- **Multi-Database Support**: Unified interface for PostgreSQL and SQLite
- **Real-time Streaming**: Event streaming with projections and sagas

## Python Tooling - UV Only

**MANDATORY**: All Python development work in this project MUST use `uv` for dependency management, tool installation, and script execution.

### Dependency Management
```bash
# Install all dependencies (replaces pip install -r requirements.txt)
uv sync

# Install with development extras
uv sync --all-extras

# Add new runtime dependency
uv add pydantic asyncio

# Add development dependency  
uv add --dev pytest black ruff

# Update dependencies
uv sync --upgrade
```

### Tool Installation
```bash
# Install development tools (replaces pip install maturin)
uv tool install maturin

# Install multiple tools
uv tool install black ruff mypy isort

# List installed tools
uv tool list

# Uninstall tools
uv tool uninstall maturin
```

### Running Scripts and Examples
```bash
# Run Python scripts (replaces python script.py)
uv run python examples/01_basic_event_store_simple.py
uv run python examples/06_event_versioning.py

# Run with specific Python version
uv run --python 3.11 python examples/performance_testing.py

# Run tests (replaces pytest)
uv run pytest eventuali-python/tests/ -v

# Run development commands
uv run maturin develop --release
uv run black --check eventuali-python/python/
uv run ruff check eventuali-python/python/
```

### Project Setup for New Development
```bash
# First time setup
git clone <repository>
cd eventuali

# Install all dependencies
uv sync

# Install required development tools
uv tool install maturin

# Build Rust-Python bindings
uv run maturin develop --release

# Verify setup
uv run python examples/01_basic_event_store_simple.py
```

### Why UV Only?

- **Faster**: 10-100x faster dependency resolution than pip
- **Automatic Virtual Environment**: No need to manually create/activate venvs
- **Better Reproducibility**: uv.lock ensures consistent builds
- **Unified Tool**: Single tool for all Python operations
- **Modern Standards**: Built for pyproject.toml and modern Python packaging
- **Better Integration**: Works seamlessly with Rust tooling via maturin

## Development Commands Reference

### Building and Testing
```bash
# Build Rust core (for testing Rust components)
cargo build --release --package eventuali-core

# Build and install Python bindings
uv run maturin develop --release

# Run Rust tests
cargo test --package eventuali-core

# Run Python tests
uv run pytest eventuali-python/tests/ -v

# Run all examples to verify functionality
uv run python examples/01_basic_event_store_simple.py
uv run python examples/02_aggregate_lifecycle.py
# ... continue for all examples
```

### Code Quality
```bash
# Format Python code
uv run black eventuali-python/python/

# Lint Python code
uv run ruff check eventuali-python/python/

# Type checking
uv run mypy eventuali-python/python/

# Sort imports
uv run isort eventuali-python/python/
```

### Performance Testing
```bash
# Run Rust performance demo (showcases 10k+ events/sec)
cargo run --package eventuali-core --example rust_streaming_demo

# Run Python performance benchmark
uv run python examples/04_performance_testing.py
```

## Project Structure

```
eventuali/
├── eventuali-core/          # Rust core library
│   ├── src/                 # Core Rust implementation
│   └── examples/            # Rust examples and demos
├── eventuali-python/        # Python bindings
│   ├── src/                 # PyO3 binding code
│   ├── python/eventuali/    # Python API
│   └── tests/               # Python tests
├── examples/                # Python usage examples
│   ├── 01_basic_event_store_simple.py
│   ├── 02_aggregate_lifecycle.py
│   ├── 03_error_handling.py
│   ├── 04_performance_testing.py
│   ├── 05_multi_aggregate_simple.py
│   ├── 06_event_versioning.py
│   └── ... (more examples)
└── CLAUDE.md               # This file
```

## Key Development Patterns

### Build-Fix-Build (BFB) Loop
When implementing new features, always follow the BFB pattern:

1. **Build**: Implement the feature and create an example
2. **Fix**: Run the example, identify and fix any issues
3. **Build**: Verify the fix works and the example runs cleanly

### Example Creation Guidelines
All new examples should:
- Use UV syntax: `uv run python examples/example_name.py`
- Include comprehensive docstrings explaining the patterns demonstrated
- Follow the established naming convention: `##_descriptive_name.py`
- Demonstrate both success and failure scenarios
- Include performance metrics where applicable
- Work with in-memory SQLite for simplicity (`sqlite://:memory:`)

### Testing Requirements
- All functionality must be tested with working examples
- Examples should execute successfully with clear output
- Performance examples should demonstrate the 10-60x improvement claims
- Error handling examples should show proper exception handling

## Troubleshooting

### UV + Maturin Integration Issues
```bash
# If maturin is not found
uv tool install maturin

# If bindings fail to build
uv run maturin clean
uv run maturin develop --release

# If dependencies are out of sync
uv sync --upgrade
```

### Common Development Issues
- **Import errors**: Ensure `uv run maturin develop --release` has been run
- **Permission errors**: Use `uv tool install` instead of global pip installs
- **Environment issues**: UV manages environments automatically - don't activate venvs manually

## Performance Expectations

The Eventuali library should demonstrate:
- **Event Creation**: 50,000+ events/second
- **Event Persistence**: 25,000+ events/second  
- **Event Loading**: 40,000+ events/second
- **Memory Usage**: 8-20x more efficient than pure Python
- **Concurrent Throughput**: 2x better under load

## Links and Resources

- **UV Documentation**: https://docs.astral.sh/uv/
- **Maturin Guide**: https://pyo3.rs/latest/getting_started
- **PyO3 Documentation**: https://pyo3.rs/
- **Event Sourcing Patterns**: https://martinfowler.com/eaaDev/EventSourcing.html

---

**Remember**: Always use `uv` for all Python operations. No exceptions.