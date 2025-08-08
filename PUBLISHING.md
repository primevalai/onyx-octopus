# Eventuali Publishing Guide

This guide explains how to publish Eventuali to PyPI, similar to npm for Node.js and NuGet for .NET.

## Quick Summary

**Like npm/NuGet, but for Python:**
- **Repository**: PyPI (Python Package Index) - equivalent to npmjs.com or nuget.org
- **Install Command**: `pip install eventuali` or `uv add eventuali` 
- **Publishing Tool**: `maturin publish` (specialized for Rust-Python bindings)
- **Testing**: TestPyPI (staging) → PyPI (production)

## Publishing Workflow Overview

```mermaid
graph LR
    A[Local Development] --> B[TestPyPI]
    B --> C[Validation]
    C --> D[PyPI Production]
    D --> E[pip install eventuali]
```

## Prerequisites

### 1. Account Setup (Manual - One Time)

You'll need accounts on both platforms:

**TestPyPI (Staging)**:
- Visit: https://test.pypi.org/account/register/
- Create account: `username@primeval.ai`
- Purpose: Test package uploads safely

**PyPI (Production)**:
- Visit: https://pypi.org/account/register/
- Create account: `username@primeval.ai` 
- Purpose: Production package distribution

### 2. Configure Trusted Publishing (Recommended 2025 Security)

**TestPyPI Setup**:
1. Go to: https://test.pypi.org/manage/account/publishing/
2. Add publisher: `primevalai/onyx-octopus` (GitHub repo)
3. Project name: `eventuali`
4. Workflow: `publish.yml`
5. Environment: `testpypi`

**PyPI Setup**:
1. Go to: https://pypi.org/manage/account/publishing/
2. Add publisher: `primevalai/onyx-octopus` (GitHub repo)  
3. Project name: `eventuali`
4. Workflow: `publish.yml`
5. Environment: `pypi`

*Note: Trusted publishing eliminates the need for API tokens - GitHub generates short-lived tokens automatically.*

## Automated Publishing (GitHub Actions)

### Current Configuration

The project now includes automated CI/CD workflows:

**📁 `.github/workflows/publish.yml`**:
- **Triggers**: Push to master (TestPyPI), Release tags (PyPI)
- **Platforms**: Windows, macOS, Linux (x86_64 + ARM64)
- **Python versions**: 3.8 - 3.12
- **Security**: Trusted publishing (no stored secrets)

**📁 `.github/workflows/build-manylinux.yml`**:
- **Purpose**: Build manylinux wheels for broad Linux compatibility
- **Targets**: x86_64, aarch64 architectures
- **Testing**: All Python versions 3.8-3.12

### Automated Workflow Triggers

1. **Development**: Every push to `master` → TestPyPI
2. **Production**: Create GitHub release → PyPI
3. **Validation**: PR builds test wheels but don't publish

## Manual Publishing (Local Development)

### TestPyPI (Staging)

```bash
# Build the package
cd eventuali-python
uv run maturin build --release

# Publish to TestPyPI
uv run maturin publish --repository testpypi

# Test installation
pip install --index-url https://test.pypi.org/simple/ eventuali
```

### PyPI (Production)

```bash
# Build the package  
cd eventuali-python
uv run maturin build --release

# Publish to PyPI
uv run maturin publish

# Verify installation
pip install eventuali
```

## Release Process

### Using the Release Preparation Script

```bash
# Test what would change
uv run python scripts/prepare-release.py --version 0.2.0 --dry-run

# Execute the release preparation
uv run python scripts/prepare-release.py --version 0.2.0 --execute
```

### Manual Release Steps

1. **Prepare Release**:
   ```bash
   # Update version numbers
   # eventuali-python/pyproject.toml: version = "0.2.0"
   # eventuali-core/Cargo.toml: version = "0.2.0"  
   # eventuali-python/Cargo.toml: version = "0.2.0"
   
   # Test everything works
   uv run python examples/01_basic_event_store_simple.py
   cargo test --workspace
   uv run pytest eventuali-python/tests/
   ```

2. **Create Release**:
   ```bash
   # Commit version changes
   git add .
   git commit -m "chore: bump version to 0.2.0"
   
   # Create and push tag
   git tag v0.2.0
   git push origin v0.2.0
   
   # Create GitHub release (triggers PyPI publish)
   gh release create v0.2.0 --title "Eventuali v0.2.0" --notes-file release-notes-0.2.0.md
   ```

3. **Verify Publication**:
   ```bash
   # Check PyPI page
   open https://pypi.org/project/eventuali/
   
   # Test installation
   pip install eventuali==0.2.0
   ```

## Package Distribution

### Installation Methods

Once published, users can install Eventuali using:

```bash
# Traditional pip
pip install eventuali

# Modern UV (recommended)
uv add eventuali  

# With specific version
pip install eventuali==0.2.0
uv add eventuali==0.2.0

# Development installation
pip install eventuali[dev]
uv add eventuali[dev]
```

### Platform Support

**Supported Platforms** (via automated wheels):
- **Linux**: x86_64, aarch64 (manylinux2014+)
- **macOS**: x86_64, Apple Silicon (M1/M2)
- **Windows**: x86_64
- **Python**: 3.8, 3.9, 3.10, 3.11, 3.12

**Fallback**: Source distribution (sdist) compiles on any platform with Rust toolchain.

## Performance & Features

### Installation Speed
- **Wheels**: Instant installation (pre-compiled)
- **Source**: ~2-3 minutes (Rust compilation)
- **Size**: ~15-25MB (includes Rust binary)

### Key Selling Points
- **10-60x faster** than pure Python event sourcing
- **Drop-in replacement** for pyeventsourcing
- **Multi-database**: PostgreSQL, SQLite built-in
- **Enterprise ready**: Security, compliance, multi-tenancy

## Troubleshooting

### Common Issues

**"No matching wheel found"**:
- Platform not supported → Install from source
- Python version too old → Upgrade to 3.8+

**"Failed building wheel"**:
- Missing Rust toolchain → Install via rustup.rs
- Missing protobuf → Install protobuf-compiler

**"Import error after installation"**:
- ABI compatibility issue → Reinstall with `--force-reinstall`
- Check Python version compatibility

### Development Issues

**Local publishing test**:
```bash
# Test build locally before publishing
cd eventuali-python
uv run maturin develop --release
uv run python -c "import eventuali; print('✓ Local build works')"
```

## Comparison to npm/NuGet

| Feature | npm (Node.js) | NuGet (.NET) | PyPI (Python) |
|---------|---------------|---------------|---------------|
| **Registry** | npmjs.com | nuget.org | pypi.org |
| **Install** | `npm install pkg` | `dotnet add package Pkg` | `pip install pkg` |
| **Publish** | `npm publish` | `nuget push` | `maturin publish` |
| **Staging** | npm tags | NuGet preview | TestPyPI |
| **Config** | package.json | .csproj | pyproject.toml |
| **CI/CD** | GitHub Actions | GitHub Actions | GitHub Actions |

## Next Steps

1. **Setup accounts** on PyPI and TestPyPI
2. **Configure trusted publishing** in repository settings
3. **Test workflow** by creating a pre-release
4. **Go live** with v0.1.0 release

After setup, releasing new versions is as simple as creating a GitHub release - just like publishing to npm or NuGet registries.