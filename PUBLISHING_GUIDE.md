# Publishing Guide for Rust Helia v0.1.3

## The Problem

When you tried to publish `rust-helia` v0.1.3, it failed because:
- `rust-helia` depends on `helia-utils = "0.1.3"`
- Only `helia-utils = "0.1.2"` is available on crates.io
- **You must publish dependencies before dependents**

## Solution: Publish in Dependency Order

### Publishing Order (17 modules)

```
Phase 1: Core Interface (no internal deps)
├─ 1. helia-interface

Phase 2: Base Layer (depends on helia-interface only)
├─ 2. helia-bitswap
├─ 3. helia-dnslink
├─ 4. helia-car
├─ 5. helia-http
├─ 6. helia-dag-cbor
├─ 7. helia-dag-json
├─ 8. helia-strings
└─ 9. helia-interop

Phase 3: Utils (depends on helia-interface + helia-bitswap)
└─ 10. helia-utils

Phase 4: Complex Modules
├─ 11. helia-routers (deps: helia-interface, helia-utils)
├─ 12. helia-json (deps: helia-interface, helia-utils, helia-bitswap)
└─ 13. helia-unixfs (deps: helia-interface, helia-utils, helia-bitswap)

Phase 5: Higher-Level Modules
├─ 14. helia-mfs (deps: helia-interface, helia-unixfs)
├─ 15. helia-ipns (deps: helia-interface, helia-dnslink)
└─ 16. helia-block-brokers (deps: helia-interface, helia-bitswap, helia-car, helia-utils)

Phase 6: Main Package
└─ 17. rust-helia (depends on most modules)
```

## Step-by-Step Publishing Commands

### Important Notes Before Starting:
1. **Wait 30-60 seconds** between publishes for crates.io to index
2. Check https://crates.io/crates/[module-name] to verify it's indexed
3. If a publish fails, check the error and fix before continuing
4. You can only publish a version once (can't republish the same version)

### Commands:

```bash
# Phase 1
cd helia-interface
cargo publish
echo "Waiting for crates.io to index..."
sleep 45
cd ..

# Phase 2
cd helia-bitswap
cargo publish
sleep 30
cd ..

cd helia-dnslink
cargo publish
sleep 30
cd ..

cd helia-car
cargo publish
sleep 30
cd ..

cd helia-http
cargo publish
sleep 30
cd ..

cd helia-dag-cbor
cargo publish
sleep 30
cd ..

cd helia-dag-json
cargo publish
sleep 30
cd ..

cd helia-strings
cargo publish
sleep 30
cd ..

cd helia-interop
cargo publish
sleep 30
cd ..

# Phase 3
cd helia-utils
cargo publish
sleep 30
cd ..

# Phase 4
cd helia-routers
cargo publish
sleep 30
cd ..

cd helia-json
cargo publish
sleep 30
cd ..

cd helia-unixfs
cargo publish
sleep 30
cd ..

# Phase 5
cd helia-mfs
cargo publish
sleep 30
cd ..

cd helia-ipns
cargo publish
sleep 30
cd ..

cd helia-block-brokers
cargo publish
sleep 30
cd ..

# Phase 6 - Main package
cd rust-helia
cargo publish
sleep 45
cd ..

echo "✅ All modules published!"
```

## Automated Publishing Script

I'll create a script that does this automatically:

```bash
#!/bin/bash
# Run: ./publish-all.sh

set -e

publish_and_wait() {
    local module=$1
    local wait_time=${2:-30}
    
    echo "📦 Publishing $module..."
    cd "$module"
    cargo publish
    cd ..
    echo "⏳ Waiting ${wait_time}s for crates.io to index..."
    sleep "$wait_time"
    echo "✅ $module published"
    echo ""
}

echo "Starting publication of all Helia modules..."
echo ""

# Phase 1
publish_and_wait "helia-interface" 45

# Phase 2
publish_and_wait "helia-bitswap" 30
publish_and_wait "helia-dnslink" 30
publish_and_wait "helia-car" 30
publish_and_wait "helia-http" 30
publish_and_wait "helia-dag-cbor" 30
publish_and_wait "helia-dag-json" 30
publish_and_wait "helia-strings" 30
publish_and_wait "helia-interop" 30

# Phase 3
publish_and_wait "helia-utils" 30

# Phase 4
publish_and_wait "helia-routers" 30
publish_and_wait "helia-json" 30
publish_and_wait "helia-unixfs" 30

# Phase 5
publish_and_wait "helia-mfs" 30
publish_and_wait "helia-ipns" 30
publish_and_wait "helia-block-brokers" 30

# Phase 6
publish_and_wait "rust-helia" 45

echo "🎉 All modules successfully published!"
```

## Troubleshooting

### Error: "crate version X already uploaded"
- You can't republish the same version
- Solution: Bump version number and try again

### Error: "failed to select a version for the requirement"
- A dependency hasn't been published yet
- Solution: Publish dependencies first, wait for indexing

### Error: "authentication required"
- Need to login to crates.io
- Solution: `cargo login <your-token>`

### Error: "rate limited"
- Published too many crates too quickly
- Solution: Wait a few minutes and continue

## Verification After Publishing

Check each module on crates.io:
- https://crates.io/crates/rust-helia
- https://crates.io/crates/helia-interface
- https://crates.io/crates/helia-utils
- etc.

## What's Next After Publishing?

1. ✅ Verify all modules are on crates.io
2. ✅ Test installation: `cargo new test-helia && cd test-helia`
3. ✅ Add to Cargo.toml: `rust-helia = "0.1.3"`
4. ✅ Run: `cargo build`
5. ✅ Update GitHub release with v0.1.3 tag
6. ✅ Announce on social media/forums

## Quick Reference: What Failed and Why

Your command:
```bash
cd rust-helia && cargo publish
```

Failed because:
```
rust-helia v0.1.3 requires:
├─ helia-utils = "0.1.3"      ❌ Only 0.1.2 on crates.io
├─ helia-interface = "0.1.3"   ❌ Only 0.1.2 on crates.io
├─ helia-unixfs = "0.1.3"      ❌ Only 0.1.2 on crates.io
└─ ... and 10 more modules     ❌ All still at 0.1.2
```

Solution: Publish all dependencies first!

## Current Status

- ✅ All modules bumped to v0.1.3 locally
- ✅ All internal dependencies updated to 0.1.3
- ✅ Documentation updated to 0.1.3
- ❌ Modules not yet published to crates.io (still at 0.1.2)
- 🎯 Next: Publish in dependency order

**Start with Phase 1 above and work your way through all phases.**
