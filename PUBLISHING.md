# Publishing Helia Rust to Crates.io

This guide walks through publishing the Helia Rust packages to crates.io.

## Prerequisites

1. **Crates.io Account**: Create an account at https://crates.io
2. **API Token**: Get your API token from https://crates.io/me
3. **Login**: Run `cargo login <your-token>`
4. **Repository Ownership**: Ensure you have ownership of the repository

## Pre-Publication Checklist

### 1. Update Repository Information

The workspace Cargo.toml currently points to the TypeScript Helia repository. Update to:

```toml
[workspace.package]
homepage = "https://github.com/cyberfly-io/rust-helia"
repository = "https://github.com/cyberfly-io/rust-helia"
```

### 2. Add LICENSE Files

Ensure LICENSE-MIT and LICENSE-APACHE files exist in the root directory.

### 3. Verify Package Metadata

Each package should have:
- `description`: Clear description of the package
- `readme`: Path to README (optional but recommended)
- `keywords`: Relevant keywords (max 5)
- `categories`: Relevant categories

### 4. Update Dependencies

Change all path dependencies to version dependencies after publishing.

## Publication Order

Packages must be published in dependency order:

### Phase 1: Core Interfaces (No dependencies)
1. `helia-interface` - Core traits and types

### Phase 2: Utilities (Depends on interface)
2. `helia-utils` - Utilities and implementations

### Phase 3: Extensions (Depends on interface and/or utils)
3. `helia-bitswap` - Bitswap protocol
4. `helia-block-brokers` - Block brokers
5. `helia-car` - CAR file support
6. `helia-dag-cbor` - DAG-CBOR codec
7. `helia-dag-json` - DAG-JSON codec
8. `helia-dnslink` - DNSLink resolution
9. `helia-http` - HTTP transport
10. `helia-interop` - Interoperability utilities
11. `helia-ipns` - IPNS support
12. `helia-json` - JSON utilities
13. `helia-mfs` - Mutable File System
14. `helia-routers` - Content routing
15. `helia-strings` - String utilities
16. `helia-unixfs` - UnixFS implementation

### Phase 4: Main Package
17. `helia` - Main entry point

## Publishing Commands

### Step 1: Login to crates.io
```bash
cargo login <your-api-token>
```

### Step 2: Verify Package Contents
```bash
cd helia-interface
cargo package --list
```

### Step 3: Test Package Build
```bash
cargo package --no-verify
```

### Step 4: Publish (in order)
```bash
# Phase 1
cd helia-interface && cargo publish && cd ..

# Wait a few minutes for crates.io to process
sleep 180

# Phase 2
cd helia-utils && cargo publish && cd ..
sleep 180

# Phase 3 - Publish each extension
for pkg in helia-bitswap helia-block-brokers helia-car helia-dag-cbor helia-dag-json \
           helia-dnslink helia-http helia-interop helia-ipns helia-json helia-mfs \
           helia-routers helia-strings helia-unixfs; do
    echo "Publishing $pkg..."
    cd $pkg && cargo publish && cd ..
    sleep 60
done

# Phase 4
cd helia && cargo publish && cd ..
```

## Automated Publishing Script

Use the provided `publish.sh` script for automated publishing:

```bash
./publish.sh
```

## Common Issues

### Issue: "Package name already taken"
**Solution**: Choose a different name prefix or contact crates.io support.

### Issue: "Failed to publish, dependency not found"
**Solution**: Wait a few minutes after publishing dependencies before publishing dependent packages.

### Issue: "Package too large"
**Solution**: Add more entries to .gitignore or use `exclude` in Cargo.toml.

### Issue: "Missing documentation"
**Solution**: Add doc comments to public items.

## Post-Publication

1. **Verify on crates.io**: Check that all packages appear at https://crates.io
2. **Update README**: Add crates.io badges
3. **Tag Release**: Create a git tag for the version
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
4. **Update Dependencies**: In future versions, use crates versions:
   ```toml
   helia-interface = "0.1.0"
   ```

## Version Management

For subsequent releases:

1. Update version in workspace Cargo.toml
2. Run `cargo update` to update Cargo.lock
3. Follow the same publication order
4. Use `--allow-dirty` if needed during development

## Important Notes

⚠️ **Cannot Unpublish**: Once published, versions cannot be deleted (only yanked)
⚠️ **Name Reservation**: Package names are permanent
⚠️ **Publish Rights**: Only repository owners can publish
⚠️ **Wait Between Publishes**: Allow 2-3 minutes between dependent packages

## Testing Before Publishing

```bash
# Test all packages build
cargo build --workspace --all-features

# Run all tests
cargo test --workspace

# Check for warnings
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all --check

# Generate documentation
cargo doc --workspace --no-deps
```

## Maintenance

After initial publication:
- Update version numbers for bug fixes/features
- Publish in the same dependency order
- Maintain CHANGELOG.md for each package
- Keep documentation up to date

---

For questions or issues, see: https://doc.rust-lang.org/cargo/reference/publishing.html
