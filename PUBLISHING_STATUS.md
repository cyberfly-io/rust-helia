# Publishing Status and Updates

## Recent Changes

### 1. Package Renamed
- Main package: `helia` → `rust-helia`
- Directory: `helia/` → `rust-helia/`
- All dependencies updated to reflect new name

### 2. Version Updates
- All packages bumped to version **0.1.2**
- All internal dependencies updated to use 0.1.2

### 3. Dependency Fixes
- Removed `rust-helia` dependencies from extension packages (since it wasn't published yet)
- Extension packages now only depend on `helia-interface` and `helia-utils`
- Added missing version specifications for all dependencies
- Fixed `helia-mfs` to include version for `helia-unixfs`

### 4. Publish Script Improvements
- **✓ Skip already published packages**: Script now checks crates.io API before attempting to publish
- **✓ Better error handling**: Failed packages no longer stop the entire process
- **✓ Workspace version support**: Correctly reads version from workspace Cargo.toml
- **✓ Wait times**: Added appropriate wait times between publishes for crates.io indexing

## Current Publishing Status

### Already Published on crates.io ✅

#### Core Packages
- `helia-interface` v0.1.2 ✅ (also 0.1.1, 0.1.0)
- `helia-utils` v0.1.2 ✅

### Ready to Publish 📦

#### Extension Packages
- `helia-bitswap` v0.1.2
- `helia-block-brokers` v0.1.2
- `helia-car` v0.1.2
- `helia-dag-cbor` v0.1.2
- `helia-dag-json` v0.1.2
- `helia-dnslink` v0.1.2
- `helia-http` v0.1.2
- `helia-interop` v0.1.2
- `helia-ipns` v0.1.2
- `helia-json` v0.1.2
- `helia-mfs` v0.1.2
- `helia-routers` v0.1.2
- `helia-strings` v0.1.2
- `helia-unixfs` v0.1.2

#### Main Package
- `rust-helia` v0.1.2 (new name!)

## How the Publish Script Works

### Smart Skip Logic
```bash
# Before publishing each package:
1. Query crates.io API for the package
2. Check if version 0.1.2 is already published
3. If yes: Skip with message "already published"
4. If no: Proceed with packaging and publishing
```

### Phase Order
1. **Phase 1**: Core Interface (helia-interface) - Already published ✅
2. **Phase 2**: Utilities (helia-utils) - Already published ✅
3. **Phase 3**: Extensions (14 packages) - Ready to publish
4. **Phase 4**: Main Package (rust-helia) - Ready to publish

### Error Handling
- Individual package failures don't stop the entire process
- Script continues to next package if one fails
- Returns to ROOT_DIR after each package
- Clear warning messages for any failures

## Next Steps to Complete Publishing

### 1. Commit All Changes
```bash
git add .
git commit -m "Prepare for publishing v0.1.2

- Renamed main package to rust-helia
- Updated all versions to 0.1.2
- Fixed all dependency specifications
- Improved publish script with skip logic"
```

### 2. Run the Publish Script
```bash
./publish.sh
```

The script will:
- ✓ Skip helia-interface (already published)
- ✓ Skip helia-utils (already published)
- ✓ Publish all 14 extension packages (with wait times)
- ✓ Publish rust-helia main package

**Expected Duration**: ~20-25 minutes (60s wait × 15 packages)

### 3. Verify Published Packages
```bash
# Check all packages are published
for pkg in helia-interface helia-utils helia-bitswap helia-block-brokers helia-car \
           helia-dag-cbor helia-dag-json helia-dnslink helia-http helia-interop \
           helia-ipns helia-json helia-mfs helia-routers helia-strings \
           helia-unixfs rust-helia; do
    echo "Checking $pkg..."
    curl -s "https://crates.io/api/v1/crates/$pkg" | grep -o '"num":"0.1.2"' && echo "✓ Published" || echo "✗ Not found"
done
```

### 4. Update Documentation
After successful publishing:
- Add crates.io badges to README.md
- Update installation instructions to use crates.io versions
- Create a GitHub release tag

## Package Dependencies After Publishing

### End User Installation
```toml
[dependencies]
# Main package (includes create_helia and core functionality)
rust-helia = "0.1.2"

# Extensions (optional, as needed)
helia-unixfs = "0.1.2"    # File system operations
helia-dag-cbor = "0.1.2"  # CBOR data structures
helia-dag-json = "0.1.2"  # JSON data structures
helia-car = "0.1.2"       # CAR file import/export
helia-ipns = "0.1.2"      # Name system
# ... other extensions as needed
```

### Usage in Code
```rust
use rust_helia::create_helia;  // Note: underscore in use statement
use helia_unixfs::UnixFS;
use helia_interface::Helia;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    // ... use the node
    Ok(())
}
```

## Files Modified

### Configuration Files
- `Cargo.toml` - Updated workspace members
- All `*/Cargo.toml` - Updated versions and dependencies

### Scripts
- `publish.sh` - Added skip logic, better error handling
- `fix-publishing.sh` - Created to remove unpublished dependencies
- `update-all-versions.sh` - Created to update all versions to 0.1.2

### Documentation
- `README.md` - Updated with rust_helia usage
- `RENAME_SUMMARY.md` - Documented the rename process
- `PUBLISHING_STATUS.md` - This file

## Troubleshooting

### If a Package Fails to Publish
1. Check the error message
2. Fix the issue in that package's Cargo.toml
3. Run `cargo package --list` in that package to verify
4. Run `./publish.sh` again - it will skip already published packages

### Common Issues
- **"already exists"**: Package version already on crates.io (script now skips these)
- **"no matching package"**: Dependency not published yet (fixed by removing rust-helia deps)
- **"missing version"**: Added version specifications to all dependencies (fixed)
- **"uncommitted changes"**: Commit changes before publishing

## Success Criteria

✅ All packages compile: `cargo check --workspace`
✅ All examples work: `cargo run --example 01_basic_node`
✅ Core packages published: helia-interface, helia-utils
⏳ Extension packages ready: 14 packages ready to publish
⏳ Main package ready: rust-helia ready to publish

**Status**: Ready to run `./publish.sh` to complete publishing! 🚀
