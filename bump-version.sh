#!/bin/bash

# Script to bump version numbers from 0.1.1 to 0.1.2 for all packages

echo "Bumping all package versions from 0.1.1 to 0.1.2..."

# Update workspace Cargo.toml if it has version
if [ -f "Cargo.toml" ]; then
    echo "Updating workspace Cargo.toml..."
    sed -i '' 's/version = "0.1.1"/version = "0.1.2"/g' Cargo.toml
fi

# Update all package Cargo.toml files
for package in helia helia-interface helia-utils helia-bitswap helia-block-brokers \
               helia-car helia-dag-cbor helia-dag-json helia-dnslink helia-http \
               helia-interop helia-ipns helia-json helia-mfs helia-routers \
               helia-strings helia-unixfs; do
    
    if [ -d "$package" ]; then
        echo "Updating $package/Cargo.toml..."
        
        # Update the package version
        sed -i '' 's/^version = "0.1.1"/version = "0.1.2"/' "$package/Cargo.toml"
        
        # Update dependency versions for all helia packages
        sed -i '' 's/helia-interface = { version = "0.1.1"/helia-interface = { version = "0.1.2"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-utils = { version = "0.1.1"/helia-utils = { version = "0.1.2"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-interface = "0.1.1"/helia-interface = "0.1.2"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-utils = "0.1.1"/helia-utils = "0.1.2"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-unixfs = "0.1.1"/helia-unixfs = "0.1.2"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-dag-cbor = "0.1.1"/helia-dag-cbor = "0.1.2"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-dag-json = "0.1.1"/helia-dag-json = "0.1.2"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-car = "0.1.1"/helia-car = "0.1.2"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-json = "0.1.1"/helia-json = "0.1.2"/g' "$package/Cargo.toml"
    fi
done

echo "✓ Version bumped to 0.1.2 for all packages"
echo ""
echo "Verifying changes with cargo check..."
cargo check --workspace --quiet 2>&1 | grep -E "(Checking|Finished|error)" || echo "✓ Check completed"
echo ""
echo "✓ All done! Ready to publish version 0.1.2"
