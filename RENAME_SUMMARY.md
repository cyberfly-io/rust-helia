# Package Rename Summary

## Changes Made

The main `helia` package has been renamed to `rust-helia` to avoid conflicts with existing crates on crates.io.

### 1. Directory and Package Name
- **Directory**: `helia/` → `rust-helia/`
- **Package Name**: `helia` → `rust-helia`
- **Crate Name in Code**: `rust_helia` (with underscore)

### 2. Files Updated

#### Workspace Configuration
- `Cargo.toml`: Updated workspace member from `"helia"` to `"rust-helia"`

#### Main Package
- `rust-helia/Cargo.toml`: 
  - Updated package name to `rust-helia`
  - Updated dependency versions to 0.1.2

#### Extension Packages
Updated all extension packages that depend on the main package:
- `helia-dag-cbor/Cargo.toml`
- `helia-dag-json/Cargo.toml`
- `helia-dnslink/Cargo.toml`
- `helia-ipns/Cargo.toml`
- `helia-json/Cargo.toml`
- `helia-mfs/Cargo.toml`
- `helia-routers/Cargo.toml`
- `helia-strings/Cargo.toml`
- `helia-unixfs/Cargo.toml`

Changed from:
```toml
helia = { path = "../helia" }
```

To:
```toml
rust-helia = { version = "0.1.2", path = "../rust-helia" }
```

#### Documentation Files
- `README.md`: Updated usage examples to `use rust_helia::`
- `USAGE.md`: Updated code examples
- `API_REFERENCE.md`: Updated code examples
- `GETTING_STARTED.md`: Updated code examples
- `PUBLISHING.md`: Updated package references
- `PUBLISH_QUICK_START.md`: Updated package references

#### Example Files
All examples in `rust-helia/examples/` updated to use `rust_helia`:
- `01_basic_node.rs`
- `02_block_storage.rs`
- `03_unixfs_files.rs`
- `04_dag_cbor.rs`
- `05_car_files.rs`
- `06_pinning.rs`
- `07_custom_config.rs`
- `examples/README.md`

#### Scripts
- `publish.sh`: Updated to publish `rust-helia` instead of `helia`

### 3. How to Use

#### In Cargo.toml
```toml
[dependencies]
rust-helia = "0.1.2"
```

#### In Code
```rust
use rust_helia::create_helia;
use helia_interface::Helia;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    // ... use the node
    Ok(())
}
```

### 4. Package Names on crates.io

After publishing, the following packages will be available:

**Core:**
- `rust-helia` - Main package (renamed from `helia`)
- `helia-interface` - Core traits and interfaces
- `helia-utils` - Utility functions and implementations

**Extensions:**
- `helia-bitswap` - Bitswap protocol
- `helia-block-brokers` - Block broker implementations
- `helia-car` - CAR file support
- `helia-dag-cbor` - DAG-CBOR codec
- `helia-dag-json` - DAG-JSON codec
- `helia-dnslink` - DNSLink resolution
- `helia-http` - HTTP gateway
- `helia-interop` - Interoperability testing
- `helia-ipns` - IPNS (InterPlanetary Name System)
- `helia-json` - JSON codec
- `helia-mfs` - Mutable File System
- `helia-routers` - Content routing
- `helia-strings` - String operations
- `helia-unixfs` - UnixFS support

### 5. Verification

The workspace compiles successfully after the rename:
```bash
cargo check --workspace
# ✓ All packages compile successfully
```

All examples work with the new package name:
```bash
cd rust-helia
cargo run --example 01_basic_node
# ✓ Example runs successfully
```

### 6. Next Steps

Before publishing:
1. Commit all changes to git
2. Verify all packages can be packaged: `cargo package --allow-dirty`
3. Run `./publish.sh` to publish all packages to crates.io

Note: Make sure you have a crates.io account and are logged in with `cargo login <token>`
