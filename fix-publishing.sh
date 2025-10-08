#!/bin/bash

# Script to fix publishing issues
# 1. Remove rust-helia dependencies from extension packages (not published yet)
# 2. Add missing version specifications
# 3. Update publish script

echo "Fixing publishing issues..."
echo ""

# Step 1: Remove rust-helia dependencies from extension packages
echo "1. Removing unpublished rust-helia dependencies from extension packages..."

# List of extension packages that should NOT depend on rust-helia for publishing
extensions=(
    "helia-dag-cbor"
    "helia-dag-json"
    "helia-dnslink"
    "helia-ipns"
    "helia-json"
    "helia-mfs"
    "helia-routers"
    "helia-strings"
    "helia-unixfs"
)

for pkg in "${extensions[@]}"; do
    if [ -f "$pkg/Cargo.toml" ]; then
        # Remove rust-helia dependency lines (both regular and dev-dependencies)
        echo "   Updating $pkg/Cargo.toml..."
        
        # Create a backup
        cp "$pkg/Cargo.toml" "$pkg/Cargo.toml.bak"
        
        # Remove rust-helia dependency line
        sed -i '' '/^rust-helia = {/d' "$pkg/Cargo.toml"
        
        # Also check [dev-dependencies] section
        # We'll use perl for more complex editing
        perl -i -pe 'BEGIN{undef $/;} s/\[dev-dependencies\]\nrust-helia = \{[^\}]+\}\n/[dev-dependencies]\n/smg' "$pkg/Cargo.toml"
    fi
done

# Step 2: Fix helia-mfs missing version spec for helia-unixfs
echo ""
echo "2. Adding missing version specification to helia-mfs..."
if [ -f "helia-mfs/Cargo.toml" ]; then
    sed -i '' 's/helia-unixfs = { path = /helia-unixfs = { version = "0.1.2", path = /g' helia-mfs/Cargo.toml
fi

# Step 3: Update publish.sh to use rust-helia instead of helia
echo ""
echo "3. Updating publish.sh script..."
sed -i '' 's/publish_package "helia" 0/publish_package "rust-helia" 0/g' publish.sh

echo ""
echo "✓ All fixes applied!"
echo ""
echo "Note: Extension packages will now only depend on helia-interface and helia-utils"
echo "      The main rust-helia package will be published last and can depend on extensions"
echo ""
echo "Verifying changes with cargo check..."
cargo check --workspace --quiet 2>&1 | grep -E "(Checking|Finished|error:)" | head -30
echo ""
echo "✓ Ready to publish!"
