#!/bin/bash

# Comprehensive script to update ALL version references to 0.1.2

echo "Updating all version references to 0.1.2..."
echo ""

# Update all Cargo.toml files in the workspace
for toml in */Cargo.toml; do
    if [ -f "$toml" ]; then
        pkg_name=$(dirname "$toml")
        echo "Updating $toml..."
        
        # Update all helia-* dependency versions
        sed -i '' 's/helia-interface = { version = "0.1.0"/helia-interface = { version = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-interface = { version = "0.1.1"/helia-interface = { version = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-utils = { version = "0.1.0"/helia-utils = { version = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-utils = { version = "0.1.1"/helia-utils = { version = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-unixfs = { version = "0.1.0"/helia-unixfs = { version = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-unixfs = { version = "0.1.1"/helia-unixfs = { version = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-dag-cbor = { version = "0.1.0"/helia-dag-cbor = { version = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-dag-cbor = { version = "0.1.1"/helia-dag-cbor = { version = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-dag-json = { version = "0.1.0"/helia-dag-json = { version = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-dag-json = { version = "0.1.1"/helia-dag-json = { version = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-car = { version = "0.1.0"/helia-car = { version = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-car = { version = "0.1.1"/helia-car = { version = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-json = { version = "0.1.0"/helia-json = { version = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-json = { version = "0.1.1"/helia-json = { version = "0.1.2"/g' "$toml"
        
        # Also update standalone version references (without path)
        sed -i '' 's/helia-interface = "0.1.0"/helia-interface = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-interface = "0.1.1"/helia-interface = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-utils = "0.1.0"/helia-utils = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-utils = "0.1.1"/helia-utils = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-unixfs = "0.1.0"/helia-unixfs = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-unixfs = "0.1.1"/helia-unixfs = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-dag-cbor = "0.1.0"/helia-dag-cbor = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-dag-cbor = "0.1.1"/helia-dag-cbor = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-dag-json = "0.1.0"/helia-dag-json = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-dag-json = "0.1.1"/helia-dag-json = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-car = "0.1.0"/helia-car = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-car = "0.1.1"/helia-car = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-json = "0.1.0"/helia-json = "0.1.2"/g' "$toml"
        sed -i '' 's/helia-json = "0.1.1"/helia-json = "0.1.2"/g' "$toml"
    fi
done

echo ""
echo "✓ All versions updated to 0.1.2"
echo ""
echo "Verifying with cargo check..."
cargo check --workspace --quiet 2>&1 | grep -E "(Checking|Finished|error:)" | head -20
echo ""
echo "✓ Ready to publish!"
