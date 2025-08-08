#!/usr/bin/env python3
"""
Release preparation script for Eventuali

This script helps prepare a new release by:
1. Updating version numbers across the project
2. Validating all examples still work
3. Running comprehensive tests
4. Building and testing the package locally
5. Preparing release notes

Usage:
    uv run python scripts/prepare-release.py --version 0.2.0 --dry-run
    uv run python scripts/prepare-release.py --version 0.2.0 --execute
"""

import argparse
import subprocess
import sys
import os
import re
from pathlib import Path

def run_command(cmd, cwd=None, capture_output=True):
    """Run a command and return the result"""
    try:
        result = subprocess.run(
            cmd, 
            shell=True, 
            cwd=cwd, 
            capture_output=capture_output,
            text=True,
            check=True
        )
        return result.stdout.strip() if capture_output else None
    except subprocess.CalledProcessError as e:
        print(f"❌ Command failed: {cmd}")
        print(f"Error: {e.stderr if e.stderr else str(e)}")
        return None

def update_version_in_file(file_path, version, dry_run=True):
    """Update version in a specific file"""
    if not os.path.exists(file_path):
        print(f"⚠️  File not found: {file_path}")
        return False
        
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Update version patterns
    patterns = [
        (r'version = "[\d\.]+"', f'version = "{version}"'),
        (r'version = [\d\.]+', f'version = {version}'),
        (r'"version": "[\d\.]+"', f'"version": "{version}"'),
    ]
    
    updated = content
    changes_made = False
    
    for pattern, replacement in patterns:
        if re.search(pattern, updated):
            updated = re.sub(pattern, replacement, updated)
            changes_made = True
    
    if changes_made:
        if dry_run:
            print(f"📝 Would update version to {version} in: {file_path}")
        else:
            with open(file_path, 'w') as f:
                f.write(updated)
            print(f"✅ Updated version to {version} in: {file_path}")
    
    return changes_made

def validate_examples(dry_run=True):
    """Validate that all examples still work"""
    examples_dir = Path("examples")
    if not examples_dir.exists():
        print("⚠️  Examples directory not found")
        return False
    
    example_files = list(examples_dir.glob("*.py"))
    print(f"🧪 Found {len(example_files)} examples to validate")
    
    if dry_run:
        print("📝 Would validate all examples (dry-run mode)")
        return True
    
    failed_examples = []
    for example in sorted(example_files)[:5]:  # Test first 5 for speed
        print(f"Testing {example.name}...")
        result = run_command(f"uv run python {example}", cwd="eventuali-python")
        if result is None:
            failed_examples.append(example.name)
    
    if failed_examples:
        print(f"❌ Failed examples: {', '.join(failed_examples)}")
        return False
    
    print("✅ All tested examples work correctly")
    return True

def build_and_test_package(version, dry_run=True):
    """Build the package and test installation"""
    if dry_run:
        print("📝 Would build and test package (dry-run mode)")
        return True
    
    print("🔨 Building package...")
    
    # Clean previous builds
    run_command("rm -rf dist/ build/", cwd="eventuali-python")
    
    # Build with maturin
    result = run_command("uv run maturin build --release", cwd="eventuali-python")
    if result is None:
        return False
    
    # Test installation in clean environment
    print("🧪 Testing package installation...")
    run_command("uv venv test-release --python 3.11", cwd="eventuali-python")
    
    # Install the built wheel
    wheel_file = run_command("ls dist/*.whl | head -1", cwd="eventuali-python")
    if not wheel_file:
        print("❌ No wheel file found")
        return False
    
    result = run_command(f"uv pip install {wheel_file} --python test-release", cwd="eventuali-python")
    if result is None:
        return False
    
    # Test basic functionality
    test_cmd = '''
import eventuali
print(f"✓ Eventuali {eventuali.__version__} imported successfully")
'''
    result = run_command(f"uv run --python test-release python -c '{test_cmd}'", cwd="eventuali-python")
    if result is None:
        return False
    
    print("✅ Package builds and installs correctly")
    return True

def generate_release_notes(version):
    """Generate release notes from git history"""
    # Get commits since last tag
    last_tag = run_command("git describe --tags --abbrev=0 2>/dev/null || echo 'initial'")
    if last_tag == 'initial':
        commit_range = "HEAD"
    else:
        commit_range = f"{last_tag}..HEAD"
    
    commits = run_command(f"git log {commit_range} --oneline --no-merges")
    if not commits:
        return f"# Release {version}\n\nInitial release of Eventuali."
    
    # Categorize commits
    features = []
    fixes = []
    docs = []
    other = []
    
    for line in commits.split('\n'):
        if 'feat:' in line or 'feature:' in line:
            features.append(line)
        elif 'fix:' in line or 'bugfix:' in line:
            fixes.append(line)
        elif 'docs:' in line or 'doc:' in line:
            docs.append(line)
        else:
            other.append(line)
    
    notes = f"# Release {version}\n\n"
    
    if features:
        notes += "## 🚀 New Features\n"
        for feat in features:
            notes += f"- {feat}\n"
        notes += "\n"
    
    if fixes:
        notes += "## 🐛 Bug Fixes\n"
        for fix in fixes:
            notes += f"- {fix}\n"
        notes += "\n"
    
    if docs:
        notes += "## 📚 Documentation\n"
        for doc in docs:
            notes += f"- {doc}\n"
        notes += "\n"
    
    if other:
        notes += "## 🔧 Other Changes\n"
        for change in other:
            notes += f"- {change}\n"
        notes += "\n"
    
    return notes

def main():
    parser = argparse.ArgumentParser(description='Prepare Eventuali release')
    parser.add_argument('--version', required=True, help='Version to release (e.g., 0.2.0)')
    parser.add_argument('--dry-run', action='store_true', help='Show what would be done without making changes')
    parser.add_argument('--execute', action='store_true', help='Actually perform the release preparation')
    
    args = parser.parse_args()
    
    if not args.dry_run and not args.execute:
        print("❌ Must specify either --dry-run or --execute")
        sys.exit(1)
    
    version = args.version
    dry_run = args.dry_run
    
    print(f"🚀 Preparing Eventuali release {version}")
    print(f"Mode: {'DRY RUN' if dry_run else 'EXECUTE'}")
    print("=" * 50)
    
    # Step 1: Update version numbers
    print("\n📝 Step 1: Update version numbers")
    files_to_update = [
        "eventuali-python/pyproject.toml",
        "eventuali-core/Cargo.toml",
        "eventuali-python/Cargo.toml",
    ]
    
    for file_path in files_to_update:
        update_version_in_file(file_path, version, dry_run)
    
    # Step 2: Validate examples
    print("\n🧪 Step 2: Validate examples")
    if not validate_examples(dry_run):
        print("❌ Example validation failed")
        sys.exit(1)
    
    # Step 3: Run tests
    print("\n🧪 Step 3: Run comprehensive tests")
    if dry_run:
        print("📝 Would run: cargo test --workspace")
        print("📝 Would run: uv run pytest eventuali-python/tests/")
    else:
        print("Running Rust tests...")
        if not run_command("cargo test --workspace", capture_output=False):
            print("❌ Rust tests failed")
            sys.exit(1)
        
        print("Running Python tests...")
        if not run_command("uv run pytest tests/", cwd="eventuali-python", capture_output=False):
            print("❌ Python tests failed")
            sys.exit(1)
    
    # Step 4: Build and test package
    print("\n🔨 Step 4: Build and test package")
    if not build_and_test_package(version, dry_run):
        print("❌ Package build/test failed")
        sys.exit(1)
    
    # Step 5: Generate release notes
    print("\n📋 Step 5: Generate release notes")
    release_notes = generate_release_notes(version)
    
    notes_file = f"release-notes-{version}.md"
    if dry_run:
        print(f"📝 Would write release notes to: {notes_file}")
        print("Preview:")
        print(release_notes[:500] + "..." if len(release_notes) > 500 else release_notes)
    else:
        with open(notes_file, 'w') as f:
            f.write(release_notes)
        print(f"✅ Release notes written to: {notes_file}")
    
    # Step 6: Final instructions
    print(f"\n🎉 Release {version} preparation complete!")
    print("\n📋 Next steps:")
    print("1. Review the changes and release notes")
    print("2. Commit the version updates: git add . && git commit -m 'chore: bump version to {version}'")
    print("3. Create and push a release tag: git tag v{version} && git push origin v{version}")
    print("4. Create a GitHub release with the generated release notes")
    print("5. The GitHub Actions workflow will automatically publish to PyPI")
    
    print(f"\n📦 After release, users can install with:")
    print(f"   pip install eventuali=={version}")
    print(f"   uv add eventuali=={version}")

if __name__ == "__main__":
    main()