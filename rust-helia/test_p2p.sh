#!/bin/bash
# Test P2P Content Sharing with Separate Blockstores

echo "🧪 P2P Content Sharing Test"
echo "============================="
echo ""
echo "This test uses SEPARATE blockstore directories to prove"
echo "whether blocks can be retrieved from the network."
echo ""
echo "📁 Store node:    /tmp/helia-store"
echo "📁 Retrieve node: /tmp/helia-retrieve"
echo ""

# Clean up old directories
echo "🧹 Cleaning up old blockstores..."
rm -rf /tmp/helia-store /tmp/helia-retrieve /tmp/helia-p2p-demo 2>/dev/null
echo ""

echo "📋 Instructions:"
echo ""
echo "Terminal 1 (this one) - Store content and keep running:"
echo "  cargo run --example 09_p2p_content_sharing -- store \"Hello from P2P!\""
echo ""
echo "Terminal 2 - Retrieve content:"
echo "  # Wait for store node to start and display CID"
echo "  cargo run --example 09_p2p_content_sharing -- get <CID>"
echo ""
echo "🔬 What this tests:"
echo "  ✅ If block is found: P2P Bitswap is working!"
echo "  ❌ If block not found: NetworkBehaviour needs implementation"
echo ""
echo "Press Enter to continue..."
read

echo ""
echo "Starting test..."
echo ""
