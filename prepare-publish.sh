#!/bin/bash

# Prepare for publishing by removing circular dev-dependencies
# This script removes rust-helia dev-dependency from all modules

set -e

echo "Preparing modules for publishing..."
echo "Removing rust-helia dev-dependencies to break circular dependencies..."

# List of modules with rust-helia dev-dependency
modules=(
    "helia-dag-cbor"
    "helia-dag-json"
    "helia-json"
    "helia-strings"
    "helia-mfs"
    "helia-unixfs"
    "helia-interop"
)

for module in "${modules[@]}"; do
    if [ -f "$module/Cargo.toml" ]; then
        echo "Processing $module..."
        # Use sed to remove the rust-helia dev-dependency line
        # This works on macOS
        sed -i '' '/^rust-helia.*path.*rust-helia/d' "$module/Cargo.toml"
        echo "✓ Removed rust-helia dev-dependency from $module"
    fi
done

echo ""
echo "✓ All modules prepared for publishing"
echo "Note: Run 'git checkout .' to restore original Cargo.toml files after publishing"
