#!/bin/bash
# Automated script to publish Helia Rust packages to crates.io

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Helia Rust - Crates.io Publisher     ║${NC}"
echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo ""

# Check if logged in to crates.io
if ! cargo login --help &> /dev/null; then
    echo -e "${RED}Error: cargo not found${NC}"
    exit 1
fi

echo -e "${YELLOW}⚠️  Important Notes:${NC}"
echo "1. Make sure you're logged in: cargo login <token>"
echo "2. All changes should be committed"
echo "3. This will publish packages in dependency order"
echo "4. Already published packages will be skipped"
echo ""
read -p "Continue with publishing? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "Publishing cancelled."
    exit 0
fi

# Function to check if a package version is already published
is_published() {
    local package=$1
    local version=$2
    
    # Query crates.io API
    local response=$(curl -s "https://crates.io/api/v1/crates/$package" 2>/dev/null)
    
    # Check if the version exists in the response
    if echo "$response" | grep -q "\"num\":\"$version\""; then
        return 0  # Already published
    else
        return 1  # Not published
    fi
}

# Function to publish a package
publish_package() {
    local package=$1
    local wait_time=${2:-60}
    
    echo ""
    echo -e "${BLUE}════════════════════════════════════════${NC}"
    echo -e "${GREEN}Publishing: $package${NC}"
    echo -e "${BLUE}════════════════════════════════════════${NC}"
    
    cd "$package" || exit 1
    
    # Get version from Cargo.toml - handle workspace version
    local version=$(grep -E "^version\s*=" Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/' || true)
    
    # If version is empty or contains 'workspace', get it from workspace Cargo.toml
    if [ -z "$version" ] || echo "$version" | grep -q "workspace"; then
        version=$(grep -E "^version\s*=" ../Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
    fi
    
    # Check if already published
    echo -e "${YELLOW}Checking if $package v$version is already published...${NC}"
    if is_published "$package" "$version"; then
        echo -e "${YELLOW}⊗ $package v$version is already published on crates.io, skipping...${NC}"
        cd ..
        return 0
    fi
    
    echo -e "${YELLOW}✓ Version $version not found, proceeding with publish...${NC}"
    
    # Verify package
    echo -e "${YELLOW}Verifying package...${NC}"
    if ! cargo package --no-verify; then
        echo -e "${RED}Failed to package $package${NC}"
        cd ..
        return 1
    fi
    
    # Publish
    echo -e "${YELLOW}Publishing to crates.io...${NC}"
    if cargo publish; then
        echo -e "${GREEN}✓ Successfully published $package v$version${NC}"
        cd ..
        return 0
    else
        echo -e "${RED}Failed to publish $package${NC}"
        cd ..
        return 1
    fi
}

# Store current directory
ROOT_DIR=$(pwd)

echo ""
echo -e "${GREEN}Starting publication process...${NC}"
echo ""

# Phase 1: Core Interface
echo -e "${BLUE}═══ PHASE 1: Core Interface ═══${NC}"
publish_package "helia-interface" || {
    echo -e "${YELLOW}Warning: Failed to publish helia-interface, continuing...${NC}"
    cd "$ROOT_DIR"
}

# Phase 2: Utilities
echo -e "${BLUE}═══ PHASE 2: Utilities ═══${NC}"
publish_package "helia-utils" || {
    echo -e "${YELLOW}Warning: Failed to publish helia-utils, continuing...${NC}"
    cd "$ROOT_DIR"
}

# Phase 3: Extensions
echo -e "${BLUE}═══ PHASE 3: Extensions ═══${NC}"
EXTENSIONS=(
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

for pkg in "${EXTENSIONS[@]}"; do
    publish_package "$pkg" || {
        echo -e "${YELLOW}Warning: Failed to publish $pkg, continuing...${NC}"
        cd "$ROOT_DIR"
    }
done

# Phase 4: Main Package
echo -e "${BLUE}═══ PHASE 4: Main Package ═══${NC}"
publish_package "rust-helia" || {
    echo -e "${YELLOW}Warning: Failed to publish rust-helia${NC}"
    cd "$ROOT_DIR"
}

echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  Publication Complete! 🎉              ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
echo ""
echo "Next steps:"
echo "1. Verify packages at: https://crates.io/search?q=helia"
echo "2. Create a release tag: git tag v0.1.0 && git push origin v0.1.0"
echo "3. Update README with crates.io badges"
echo ""
