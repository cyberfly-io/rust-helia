#!/bin/bash
# Helper script to run Helia examples

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Helia Rust Examples Runner${NC}"
echo ""

# Change to helia directory
cd "$(dirname "$0")/helia" || exit 1

if [ $# -eq 0 ]; then
    echo "Available examples:"
    echo "  01_basic_node       - Basic node creation and management"
    echo "  02_block_storage    - Block storage operations"
    echo "  03_unixfs_files     - UnixFS file operations"
    echo "  04_dag_cbor         - DAG-CBOR structured data"
    echo "  05_car_files        - CAR file import/export"
    echo "  06_pinning          - Content pinning"
    echo "  07_custom_config    - Custom configuration"
    echo ""
    echo "Usage: ./run-example.sh <example_name>"
    echo "Example: ./run-example.sh 01_basic_node"
    exit 0
fi

EXAMPLE_NAME=$1

echo -e "${YELLOW}Running example: ${EXAMPLE_NAME}${NC}"
echo ""

cargo run --example "$EXAMPLE_NAME"
