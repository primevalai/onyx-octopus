#!/usr/bin/env python3
"""
Quick build validation script for Eventuali

Tests that the package can be built and installed locally before publishing.
This is equivalent to testing before 'npm publish' or 'nuget push'.

Usage:
    uv run python scripts/test-build.py
"""

import subprocess
import sys
import os
import tempfile
import shutil
from pathlib import Path

def run_command(cmd, cwd=None, description=None):
    """Run a command and return success status"""
    if description:
        print(f"🔨 {description}")
    
    try:
        result = subprocess.run(
            cmd, 
            shell=True, 
            cwd=cwd, 
            capture_output=True,
            text=True,
            check=True
        )
        return True, result.stdout
    except subprocess.CalledProcessError as e:
        print(f"❌ Command failed: {cmd}")
        print(f"Error: {e.stderr}")
        return False, e.stderr

def main():
    print("🚀 Eventuali Build Validation")
    print("=" * 40)
    
    # Change to eventuali-python directory
    original_dir = os.getcwd()
    python_dir = Path(original_dir) / "eventuali-python"
    
    if not python_dir.exists():
        print("❌ eventuali-python directory not found")
        sys.exit(1)
    
    # Step 1: Clean previous builds
    print("\n🧹 Step 1: Clean previous builds")
    target_wheels_dir = Path(original_dir) / "target" / "wheels"
    if target_wheels_dir.exists():
        shutil.rmtree(target_wheels_dir)
        print("✅ Cleaned target/wheels/ directory")
    
    # Step 2: Build the package
    print("\n🔨 Step 2: Build package with maturin")
    success, output = run_command(
        "uv run maturin build --release", 
        cwd=python_dir,
        description="Building Rust-Python bindings"
    )
    if not success:
        sys.exit(1)
    
    # Check for built wheel
    wheels = list(target_wheels_dir.glob("*.whl")) if target_wheels_dir.exists() else []
    if not wheels:
        print("❌ No wheel file generated")
        sys.exit(1)
    
    wheel_file = wheels[0]
    print(f"✅ Built wheel: {wheel_file.name}")
    
    # Step 3: Test installation in clean environment
    print("\n🧪 Step 3: Test installation in clean environment")
    
    with tempfile.TemporaryDirectory() as temp_dir:
        temp_path = Path(temp_dir)
        venv_path = temp_path / "test-venv"
        
        # Create test environment
        success, _ = run_command(
            f"uv venv {venv_path} --python 3.11",
            description="Creating test virtual environment"
        )
        if not success:
            sys.exit(1)
        
        # Install the wheel
        success, _ = run_command(
            f"uv pip install {wheel_file} --python {venv_path}",
            description="Installing built wheel"
        )
        if not success:
            sys.exit(1)
        
        # Test basic import
        test_script = '''
import sys
sys.path.insert(0, ".")

try:
    import eventuali
    print("✅ Successfully imported eventuali")
    
    # Test basic functionality
    from eventuali import EventStore
    print("✅ EventStore import successful")
    
    # Test CLI availability
    from eventuali.cli import main
    print("✅ CLI module available")
    
    print("🎉 All basic imports successful!")
    
except Exception as e:
    print(f"❌ Import failed: {e}")
    sys.exit(1)
'''
        
        success, output = run_command(
            f"uv run --python {venv_path} python -c '{test_script}'",
            description="Testing package imports"
        )
        if not success:
            sys.exit(1)
        
        print(output)
    
    # Step 4: Validate example compatibility
    print("\n🧪 Step 4: Quick example validation")
    success, output = run_command(
        "uv run python ../examples/01_basic_event_store_simple.py",
        cwd=python_dir,
        description="Testing basic example"
    )
    if not success:
        print("⚠️  Example test failed - package may have issues")
    else:
        print("✅ Basic example runs successfully")
    
    # Step 5: Check package metadata
    print("\n📋 Step 5: Package metadata validation")
    success, output = run_command(
        "uv run maturin build --release --metadata-only",
        cwd=python_dir,
        description="Validating package metadata"
    )
    if success:
        print("✅ Package metadata is valid")
    
    print("\n🎉 Build validation complete!")
    print("\n📦 Ready for publishing:")
    print(f"   Wheel: {wheel_file}")
    print(f"   Size: {wheel_file.stat().st_size / 1024 / 1024:.1f} MB")
    print("\n📋 Next steps:")
    print("1. Publish to TestPyPI: uv run maturin publish --repository testpypi")
    print("2. Test TestPyPI install: pip install --index-url https://test.pypi.org/simple/ eventuali")
    print("3. Publish to PyPI: uv run maturin publish")
    print("4. Final test: pip install eventuali")

if __name__ == "__main__":
    main()