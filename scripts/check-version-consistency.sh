#!/bin/bash
# Oracle's Version Consistency Guard
# Ensures single source of truth for all versions

set -e

echo "🔍 Checking version consistency..."

# Check for duplicate major versions in cargo tree
echo "Checking for duplicate major versions..."
if cargo tree --duplicates --format "{p}" | grep -v "^$"; then
    echo "❌ Found duplicate major versions in dependency tree"
    echo "Run: cargo tree --duplicates"
    exit 1
fi

# Verify workspace dependencies are used consistently
echo "Verifying workspace inheritance..."
if find crates -name "Cargo.toml" -exec grep -l "version = \"" {} \; | grep -v workspace; then
    echo "❌ Found hardcoded versions in crate Cargo.toml files"
    echo "All versions should use 'workspace = true'"
    exit 1
fi

# Check for exact version locks on engine nucleus
echo "Verifying engine nucleus version locks..."
if ! grep -q 'bevy = "=0\.16\.1"' Cargo.toml; then
    echo "❌ Bevy version not patch-locked to =0.16.1"
    exit 1
fi

if ! grep -q 'bevy_rapier3d = "=0\.26\.0"' Cargo.toml; then
    echo "❌ bevy_rapier3d version not patch-locked to =0.26.0"
    exit 1
fi

# Check for patch.crates-io consistency
echo "Verifying patch.crates-io consistency..."
if ! grep -q '\[patch\.crates-io\]' Cargo.toml; then
    echo "❌ Missing [patch.crates-io] section"
    exit 1
fi

echo "✅ Version consistency checks passed"
