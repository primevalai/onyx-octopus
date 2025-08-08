# Eventuali Development Guide for Claude Code

This document provides essential context and development guidelines for working on the Eventuali project - a high-performance hybrid Python-Rust event sourcing library.

## CRITICAL DEVELOPMENT RULES

### 🚨 MANDATORY: UV-ONLY PYTHON DEVELOPMENT 🚨

**ABSOLUTE REQUIREMENT**: ALL Python operations MUST use `uv`. NO EXCEPTIONS.

**NEVER USE**:
- ❌ `pip install`
- ❌ `pip install -r requirements.txt`  
- ❌ `python script.py`
- ❌ `pytest`
- ❌ `python -m module`
- ❌ Virtual environment activation (`source venv/bin/activate`)

**ALWAYS USE**:
- ✅ `uv add package-name`
- ✅ `uv sync` 
- ✅ `uv run python script.py`
- ✅ `uv run pytest`
- ✅ `uv run module`
- ✅ `uv tool install tool-name`

**WHY THIS IS CRITICAL**: Using pip or direct Python breaks dependency consistency, bypasses performance optimizations, and causes integration failures with the Rust toolchain.

## Project Overview

Eventuali combines Rust's performance and memory safety with Python's developer experience to create a powerful event sourcing library. The project achieves 10-60x performance improvements over pure Python implementations while maintaining a Pythonic API.

**Architecture**:
- **Rust Core** (`eventuali-core`): High-performance event storage, serialization, and streaming
- **Python Bindings** (`eventuali-python`): PyO3-based bindings with Pydantic integration
- **Multi-Database Support**: Unified interface for PostgreSQL and SQLite
- **Real-time Streaming**: Event streaming with projections and sagas

## Python Tooling - UV Only

**🚨 CRITICAL REMINDER: UV ONLY - NO PIP/PYTHON DIRECT USAGE 🚨**

**MANDATORY**: All Python development work in this project MUST use `uv` for dependency management, tool installation, and script execution.

**⚠️ FORBIDDEN COMMANDS ⚠️**: Never use `pip`, `python script.py`, `pytest`, `python -m`, or any direct Python commands.

### Dependency Management
```bash
# ✅ Install all dependencies (NEVER use: pip install -r requirements.txt)
uv sync

# ✅ Install with development extras
uv sync --all-extras

# ✅ Add new runtime dependency (NEVER use: pip install package)
uv add pydantic asyncio

# ✅ Add development dependency (NEVER use: pip install --dev)
uv add --dev pytest black ruff

# ✅ Update dependencies (NEVER use: pip install --upgrade)
uv sync --upgrade
```

### Tool Installation
```bash
# ✅ Install development tools (NEVER use: pip install maturin)
uv tool install maturin

# ✅ Install multiple tools (NEVER use: pip install black ruff mypy)
uv tool install black ruff mypy isort

# List installed tools
uv tool list

# Uninstall tools
uv tool uninstall maturin
```

### Running Scripts and Examples
```bash
# ✅ Run Python scripts (NEVER use: python script.py)
uv run python examples/01_basic_event_store_simple.py
uv run python examples/06_event_versioning.py

# ✅ Run with specific Python version
uv run --python 3.11 python examples/performance_testing.py

# ✅ Run tests (NEVER use: pytest directly)
uv run pytest eventuali-python/tests/ -v

# ✅ Run development commands (NEVER use direct tool calls)
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

## UV Command Quick Reference

**🔄 COMMON COMMAND TRANSLATIONS**:
```bash
# OLD WAY (FORBIDDEN)        →  NEW WAY (REQUIRED)
pip install package         →  uv add package
pip install -r requirements →  uv sync
python script.py           →  uv run python script.py
pytest tests/              →  uv run pytest tests/
python -m module           →  uv run python -m module
pip install --dev tool     →  uv add --dev tool
pip install --upgrade pkg  →  uv sync --upgrade
```

### Why UV Only?

- **Faster**: 10-100x faster dependency resolution than pip
- **Automatic Virtual Environment**: No need to manually create/activate venvs
- **Better Reproducibility**: uv.lock ensures consistent builds
- **Unified Tool**: Single tool for all Python operations
- **Modern Standards**: Built for pyproject.toml and modern Python packaging
- **Better Integration**: Works seamlessly with Rust tooling via maturin
- **⚠️ CRITICAL**: Prevents dependency conflicts and build failures

## Development Commands Reference

### Building and Testing
```bash
# Build Rust core (for testing Rust components)
cargo build --release --package eventuali-core

# ✅ Build and install Python bindings (NEVER use: maturin develop directly)
uv run maturin develop --release

# Run Rust tests
cargo test --package eventuali-core

# ✅ Run Python tests (NEVER use: pytest directly)
uv run pytest eventuali-python/tests/ -v

# ✅ Run all examples to verify functionality (NEVER use: python directly)
uv run python examples/01_basic_event_store_simple.py
uv run python examples/02_aggregate_lifecycle.py
# ... continue for all examples
```

### Code Quality
```bash
# ✅ Format Python code (NEVER use: black directly)
uv run black eventuali-python/python/

# ✅ Lint Python code (NEVER use: ruff directly)
uv run ruff check eventuali-python/python/

# ✅ Type checking (NEVER use: mypy directly)
uv run mypy eventuali-python/python/

# ✅ Sort imports (NEVER use: isort directly)
uv run isort eventuali-python/python/
```

### Performance Testing
```bash
# Run Rust performance demo (showcases 10k+ events/sec)
cargo run --package eventuali-core --example rust_streaming_demo

# ✅ Run Python performance benchmark (NEVER use: python directly)
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
**🚨 UV-ONLY REQUIREMENT FOR ALL EXAMPLES 🚨**

All new examples should:
- **MANDATORY**: Use UV syntax: `uv run python examples/example_name.py`
- **FORBIDDEN**: Never use `python examples/example_name.py` directly
- Include comprehensive docstrings explaining the patterns demonstrated
- Follow the established naming convention: `##_descriptive_name.py`
- Demonstrate both success and failure scenarios
- Include performance metrics where applicable
- Work with in-memory SQLite for simplicity (`sqlite://:memory:`)

### Testing Requirements
- All functionality must be tested with working examples
- **UV REQUIREMENT**: Examples should execute successfully with `uv run python examples/file.py`
- Performance examples should demonstrate the 10-60x improvement claims
- Error handling examples should show proper exception handling
- **NEVER**: Run examples with direct Python calls

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
- **🚨 COMMAND ERRORS**: If you get "command not found" for Python tools, you forgot to use `uv run`

## Performance Expectations

The Eventuali library should demonstrate:
- **Event Creation**: 50,000+ events/second
- **Event Persistence**: 25,000+ events/second  
- **Event Loading**: 40,000+ events/second
- **Memory Usage**: 8-20x more efficient than pure Python
- **Concurrent Throughput**: 2x better under load

## Git Repository Management

**IMPORTANT**: All git operations for remote repositories (push/pull) must use SSH, not HTTPS.

```bash
# Always use SSH for remote operations
git remote set-url origin git@github.com:username/repository.git

# Verify SSH is configured
git remote -v

# Standard workflow
git push origin feature-branch  # Uses SSH
git pull origin main           # Uses SSH
```

## Links and Resources

- **UV Documentation**: https://docs.astral.sh/uv/
- **Maturin Guide**: https://pyo3.rs/latest/getting_started
- **PyO3 Documentation**: https://pyo3.rs/
- **Event Sourcing Patterns**: https://martinfowler.com/eaaDev/EventSourcing.html

---

## 🔒 FINAL CRITICAL REMINDER 🔒

**ABSOLUTE RULE**: `uv` is MANDATORY for ALL Python operations. NO EXCEPTIONS.

**❌ NEVER EVER USE**:
- `pip install` or any pip command
- `python script.py` (direct Python execution)
- `pytest`, `black`, `ruff`, `mypy` (direct tool execution)
- `python -m module` (direct module execution)

**✅ ALWAYS USE**:
- `uv add package` (for dependencies)
- `uv run python script.py` (for Python scripts)
- `uv run pytest` (for testing)
- `uv run tool-name` (for tools)

**This is not optional. This is not a preference. This is a hard requirement.**
