#!/bin/bash

# Automated script to publish all Helia modules to crates.io
# in the correct dependency order
#
# Usage: ./publish-all.sh
#
# Requirements:
# - cargo login already completed
# - All versions bumped to 0.1.3
# - Clean git status recommended

set -e  # Exit on error

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Publishing Rust Helia v0.1.3 to crates.io              ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""

# Function to publish a module and wait for indexing
publish_and_wait() {
    local module=$1
    local wait_time=${2:-30}
    local phase=$3
    
    echo -e "${YELLOW}[$phase] Publishing $module...${NC}"
    
    if [ ! -d "$module" ]; then
        echo -e "${RED}✗ Directory $module not found!${NC}"
        exit 1
    fi
    
    cd "$module"
    
    # Verify package before publishing
    if ! cargo package --quiet 2>/dev/null; then
        echo -e "${RED}✗ Failed to package $module${NC}"
        cd ..
        exit 1
    fi
    
    # Actually publish
    if ! cargo publish; then
        echo -e "${RED}✗ Failed to publish $module${NC}"
        echo -e "${YELLOW}Check if you're logged in: cargo login${NC}"
        cd ..
        exit 1
    fi
    
    cd ..
    
    echo -e "${GREEN}✓ $module published successfully${NC}"
    echo -e "${YELLOW}⏳ Waiting ${wait_time}s for crates.io to index...${NC}"
    sleep "$wait_time"
    echo ""
}

# Confirm before starting
echo -e "${YELLOW}This will publish 17 modules to crates.io.${NC}"
echo -e "${YELLOW}Make sure you have run 'cargo login' first.${NC}"
echo ""
read -p "Continue? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "Aborted."
    exit 0
fi

echo ""
echo -e "${BLUE}Starting publication process...${NC}"
echo ""

# Phase 1: Core Interface
echo -e "${BLUE}=== Phase 1: Core Interface ===${NC}"
publish_and_wait "helia-interface" 45 "1/17"

# Phase 2: Base Layer
echo -e "${BLUE}=== Phase 2: Base Layer ===${NC}"
publish_and_wait "helia-bitswap" 30 "2/17"
publish_and_wait "helia-dnslink" 30 "3/17"
publish_and_wait "helia-car" 30 "4/17"
publish_and_wait "helia-http" 30 "5/17"
publish_and_wait "helia-dag-cbor" 30 "6/17"
publish_and_wait "helia-dag-json" 30 "7/17"
publish_and_wait "helia-strings" 30 "8/17"
publish_and_wait "helia-interop" 30 "9/17"

# Phase 3: Utils
echo -e "${BLUE}=== Phase 3: Utils Layer ===${NC}"
publish_and_wait "helia-utils" 30 "10/17"

# Phase 4: Complex Modules
echo -e "${BLUE}=== Phase 4: Complex Modules ===${NC}"
publish_and_wait "helia-routers" 30 "11/17"
publish_and_wait "helia-json" 30 "12/17"
publish_and_wait "helia-unixfs" 30 "13/17"

# Phase 5: Higher-Level Modules
echo -e "${BLUE}=== Phase 5: Higher-Level Modules ===${NC}"
publish_and_wait "helia-mfs" 30 "14/17"
publish_and_wait "helia-ipns" 30 "15/17"
publish_and_wait "helia-block-brokers" 30 "16/17"

# Phase 6: Main Package
echo -e "${BLUE}=== Phase 6: Main Package ===${NC}"
publish_and_wait "rust-helia" 45 "17/17"

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   🎉 All 17 modules published successfully!              ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Verify on crates.io: https://crates.io/crates/rust-helia"
echo "2. Test installation: cargo new test && cd test"
echo "3. Add to Cargo.toml: rust-helia = \"0.1.3\""
echo "4. Run: cargo build"
echo "5. Create GitHub release with v0.1.3 tag"
echo ""
echo -e "${GREEN}✅ Publishing complete!${NC}"
