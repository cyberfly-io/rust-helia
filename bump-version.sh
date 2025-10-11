#!/bin/bash

# Script to bump version nuecho "✓ Version bumped to 0.1.3 for all packages"
echo ""
echo "Verifying changes with cargo check..."
cargo check --workspace --quiet 2>&1 | grep -E "(Checking|Finished|error)" || echo "✓ Check completed"
echo ""
echo "✓ All done! Ready to publish version 0.1.3"from 0.1.2 to 0.1.3 for all packages

echo "Bumping all package versions from 0.1.2 to 0.1.3..."

# Update workspace Cargo.toml if it has version
if [ -f "Cargo.toml" ]; then
    echo "Updating workspace Cargo.toml..."
    sed -i '' 's/version = "0.1.2"/version = "0.1.3"/g' Cargo.toml
fi

# Update all package Cargo.toml files
for package in rust-helia helia-interface helia-utils helia-bitswap helia-block-brokers \
               helia-car helia-dag-cbor helia-dag-json helia-dnslink helia-http \
               helia-interop helia-ipns helia-json helia-mfs helia-routers \
               helia-strings helia-unixfs; do
    
    if [ -d "$package" ]; then
        echo "Updating $package/Cargo.toml..."
        
        # Update the package version
        sed -i '' 's/^version = "0.1.2"/version = "0.1.3"/' "$package/Cargo.toml"
        
        # Update dependency versions for all helia packages
        sed -i '' 's/helia-interface = { version = "0.1.2"/helia-interface = { version = "0.1.3"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-utils = { version = "0.1.2"/helia-utils = { version = "0.1.3"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-bitswap = { version = "0.1.2"/helia-bitswap = { version = "0.1.3"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-unixfs = { version = "0.1.2"/helia-unixfs = { version = "0.1.3"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-dag-cbor = { version = "0.1.2"/helia-dag-cbor = { version = "0.1.3"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-dag-json = { version = "0.1.2"/helia-dag-json = { version = "0.1.3"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-car = { version = "0.1.2"/helia-car = { version = "0.1.3"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-json = { version = "0.1.2"/helia-json = { version = "0.1.3"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-mfs = { version = "0.1.2"/helia-mfs = { version = "0.1.3"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-dnslink = { version = "0.1.2"/helia-dnslink = { version = "0.1.3"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-routers = { version = "0.1.2"/helia-routers = { version = "0.1.3"/g' "$package/Cargo.toml"
        sed -i '' 's/helia-ipns = { version = "0.1.2"/helia-ipns = { version = "0.1.3"/g' "$package/Cargo.toml"
    fi
done

echo "✓ Version bumped to 0.1.3 for all packages"
echo ""
echo "Verifying changes with cargo check..."
cargo check --workspace --quiet 2>&1 | grep -E "(Checking|Finished|error)" || echo "✓ Check completed"
echo ""
echo "✓ All done! Ready to publish version 0.1.2"
