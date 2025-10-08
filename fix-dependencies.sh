#!/bin/bash
# Script to add version specifications to all helia path dependencies

set -e

echo "Updating all helia package dependencies with version specifications..."

# List of packages that depend on helia-interface
INTERFACE_DEPS=(
    "helia"
    "helia-bitswap"
    "helia-block-brokers"
    "helia-car"
    "helia-dag-cbor"
    "helia-dag-json"
    "helia-dnslink"
    "helia-http"
    "helia-interop"
    "helia-ipns"
    "helia-json"
    "helia-mfs"
    "helia-routers"
    "helia-strings"
    "helia-unixfs"
)

# Update helia-interface dependencies
for pkg in "${INTERFACE_DEPS[@]}"; do
    if [ -f "$pkg/Cargo.toml" ]; then
        echo "Updating $pkg/Cargo.toml..."
        sed -i.bak 's|helia-interface = { path = "../helia-interface" }|helia-interface = { version = "0.1.0", path = "../helia-interface" }|g' "$pkg/Cargo.toml"
        rm "$pkg/Cargo.toml.bak" 2>/dev/null || true
    fi
done

# List of packages that depend on helia-utils
UTILS_DEPS=(
    "helia"
    "helia-car"
    "helia-dag-cbor"
    "helia-dag-json"
    "helia-json"
    "helia-unixfs"
)

# Update helia-utils dependencies
for pkg in "${UTILS_DEPS[@]}"; do
    if [ -f "$pkg/Cargo.toml" ]; then
        echo "Updating $pkg/Cargo.toml (helia-utils)..."
        sed -i.bak 's|helia-utils = { path = "../helia-utils" }|helia-utils = { version = "0.1.0", path = "../helia-utils" }|g' "$pkg/Cargo.toml"
        rm "$pkg/Cargo.toml.bak" 2>/dev/null || true
    fi
done

# Update dev-dependencies in helia
if [ -f "helia/Cargo.toml" ]; then
    echo "Updating helia/Cargo.toml dev-dependencies..."
    sed -i.bak 's|helia-unixfs = { path = "../helia-unixfs" }|helia-unixfs = { version = "0.1.0", path = "../helia-unixfs" }|g' "helia/Cargo.toml"
    sed -i.bak 's|helia-dag-cbor = { path = "../helia-dag-cbor" }|helia-dag-cbor = { version = "0.1.0", path = "../helia-dag-cbor" }|g' "helia/Cargo.toml"
    sed -i.bak 's|helia-dag-json = { path = "../helia-dag-json" }|helia-dag-json = { version = "0.1.0", path = "../helia-dag-json" }|g' "helia/Cargo.toml"
    sed -i.bak 's|helia-car = { path = "../helia-car" }|helia-car = { version = "0.1.0", path = "../helia-car" }|g' "helia/Cargo.toml"
    rm "helia/Cargo.toml.bak" 2>/dev/null || true
fi

echo "✓ All Cargo.toml files updated with version specifications"
echo ""
echo "Verifying changes..."
cargo check --workspace

echo ""
echo "✓ All done! Packages are now ready for publishing."
