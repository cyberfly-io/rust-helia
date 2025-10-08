#!/bin/bash

# Script to rename the main 'helia' package to 'rust-helia' to avoid conflicts

echo "Renaming 'helia' package to 'rust-helia'..."
echo ""

# Step 1: Update workspace Cargo.toml
echo "1. Updating workspace Cargo.toml..."
sed -i '' 's/"helia",/"rust-helia",/' Cargo.toml

# Step 2: Rename the helia directory
echo "2. Renaming helia/ directory to rust-helia/..."
if [ -d "helia" ] && [ ! -d "rust-helia" ]; then
    mv helia rust-helia
    echo "   ✓ Directory renamed"
else
    echo "   ℹ Directory already renamed or doesn't exist"
fi

# Step 3: Update rust-helia/Cargo.toml package name
echo "3. Updating rust-helia/Cargo.toml package name..."
sed -i '' 's/^name = "helia"/name = "rust-helia"/' rust-helia/Cargo.toml

# Step 4: Update all documentation files that reference the main helia package
echo "4. Updating documentation files..."

# Update README.md - be careful to only update package references, not project name
if [ -f "README.md" ]; then
    sed -i '' 's/helia = "0\./rust-helia = "0./g' README.md
    sed -i '' 's/use helia::/use rust_helia::/g' README.md
    echo "   ✓ README.md updated"
fi

# Update USAGE.md
if [ -f "USAGE.md" ]; then
    sed -i '' 's/helia = "0\./rust-helia = "0./g' USAGE.md
    sed -i '' 's/use helia::/use rust_helia::/g' USAGE.md
    echo "   ✓ USAGE.md updated"
fi

# Update API_REFERENCE.md
if [ -f "API_REFERENCE.md" ]; then
    sed -i '' 's/helia = "0\./rust-helia = "0./g' API_REFERENCE.md
    sed -i '' 's/use helia::/use rust_helia::/g' API_REFERENCE.md
    echo "   ✓ API_REFERENCE.md updated"
fi

# Update GETTING_STARTED.md
if [ -f "GETTING_STARTED.md" ]; then
    sed -i '' 's/helia = "0\./rust-helia = "0./g' GETTING_STARTED.md
    sed -i '' 's/use helia::/use rust_helia::/g' GETTING_STARTED.md
    echo "   ✓ GETTING_STARTED.md updated"
fi

# Update PUBLISHING.md
if [ -f "PUBLISHING.md" ]; then
    sed -i '' 's/"helia"/"rust-helia"/g' PUBLISHING.md
    echo "   ✓ PUBLISHING.md updated"
fi

# Update PUBLISH_QUICK_START.md
if [ -f "PUBLISH_QUICK_START.md" ]; then
    sed -i '' 's/"helia"/"rust-helia"/g' PUBLISH_QUICK_START.md
    echo "   ✓ PUBLISH_QUICK_START.md updated"
fi

# Step 5: Update all example files
echo "5. Updating example files..."
if [ -d "rust-helia/examples" ]; then
    find rust-helia/examples -name "*.rs" -type f -exec sed -i '' 's/use helia::/use rust_helia::/g' {} \;
    echo "   ✓ Example files updated"
fi

# Update examples README
if [ -f "rust-helia/examples/README.md" ]; then
    sed -i '' 's/helia = "0\./rust-helia = "0./g' rust-helia/examples/README.md
    sed -i '' 's/use helia::/use rust_helia::/g' rust-helia/examples/README.md
    echo "   ✓ examples/README.md updated"
fi

# Step 6: Update publish.sh script
echo "6. Updating publish.sh script..."
if [ -f "publish.sh" ]; then
    sed -i '' 's/PACKAGES=("helia")/PACKAGES=("rust-helia")/' publish.sh
    sed -i '' 's/"helia" # Main package/"rust-helia" # Main package/' publish.sh
    echo "   ✓ publish.sh updated"
fi

echo ""
echo "✓ Renaming complete!"
echo ""
echo "Note: The package name is 'rust-helia' but the crate name in code is 'rust_helia' (with underscore)."
echo ""
echo "Verifying changes with cargo check..."
cargo check --workspace --quiet 2>&1 | grep -E "(Checking|Finished|error:)" | head -20
echo ""
echo "✓ All done! The main package is now 'rust-helia'"
