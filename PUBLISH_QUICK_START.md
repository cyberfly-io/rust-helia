# Quick Start: Publishing to Crates.io

## Prerequisites Checklist

Before publishing, ensure:
- [x] License files created (LICENSE-MIT, LICENSE-APACHE)
- [x] Repository URLs updated in Cargo.toml
- [x] All packages build successfully
- [ ] Changes committed to git
- [ ] You have a crates.io account
- [ ] You have your API token from crates.io

## Step-by-Step Publishing Guide

### Step 1: Commit Your Changes

```bash
git add .
git commit -m "Prepare for crates.io publication"
git push origin main
```

### Step 2: Login to Crates.io

1. Go to https://crates.io/me
2. Copy your API token
3. Run:
```bash
cargo login <your-token>
```

### Step 3: Test Package First Package

```bash
cd helia-interface
cargo package --list
cargo package
cd ..
```

### Step 4: Publish Packages in Order

**Important:** Packages must be published in dependency order. Wait 2-3 minutes between publishing dependent packages.

#### Phase 1: Core Interface (no dependencies)
```bash
cd helia-interface
cargo publish
cd ..
```
**Wait 3 minutes** for crates.io to process.

#### Phase 2: Utilities (depends on helia-interface)

After publishing helia-interface, you need to update helia-utils/Cargo.toml to use the crates.io version:

```toml
[dependencies]
helia-interface = "0.1.0"  # Change from path to version
```

Then publish:
```bash
cd helia-utils
cargo publish
cd ..
```
**Wait 3 minutes**.

#### Phase 3: Extensions

Update each package's Cargo.toml to use crates.io versions, then publish:

```bash
# Update dependencies in Cargo.toml first, then:
cd helia-car && cargo publish && cd ..
sleep 60

cd helia-dag-cbor && cargo publish && cd ..
sleep 60

cd helia-dag-json && cargo publish && cd ..
sleep 60

cd helia-json && cargo publish && cd ..
sleep 60

cd helia-unixfs && cargo publish && cd ..
sleep 60

# Continue for other packages...
```

#### Phase 4: Main Package

Finally, publish the main helia package:

```bash
cd helia
cargo publish
```

## Alternative: Use the Automated Script

**Note:** The automated script still uses path dependencies. For first-time publishing, you'll need to either:

1. Manually update Cargo.toml files to use crates versions, OR
2. Publish manually following the steps above

To use the script (after updating dependencies):
```bash
./publish.sh
```

## Important Notes

### Package Names
- Package names on crates.io are first-come, first-served
- Check if "helia-*" names are available at https://crates.io
- Consider alternative naming if needed (e.g., "helia-rs-*", "rust-helia-*")

### Version Updates
- First publication: 0.1.0
- Bug fixes: 0.1.1, 0.1.2, etc.
- New features: 0.2.0, 0.3.0, etc.
- Breaking changes: 1.0.0, 2.0.0, etc.

### Common Issues

**Issue: Package name already taken**
```bash
# Check if name exists
cargo search helia-interface
```
Solution: Choose a different name or namespace

**Issue: Dependency not found**
Solution: Wait longer between publishes (3-5 minutes)

**Issue: Git not clean**
Solution: Commit all changes or use `--allow-dirty` flag

### Dry Run (Test Without Publishing)

Test packaging without actually publishing:
```bash
cargo package --no-verify
```

Check package contents:
```bash
cargo package --list
```

### Quick Command Reference

```bash
# Login
cargo login <token>

# Test package
cargo package --list

# Publish (with explicit allow-dirty if needed)
cargo publish --allow-dirty

# Check published package
cargo search <package-name>

# View package on crates.io
# https://crates.io/crates/<package-name>
```

## After Publishing

1. **Verify on crates.io**: Check https://crates.io/crates/helia-interface
2. **Test installation**: `cargo add helia-interface`
3. **Update README**: Add crates.io badges
4. **Create git tag**: 
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
5. **Create GitHub release**: Document changes

## Need Help?

- Cargo Book: https://doc.rust-lang.org/cargo/reference/publishing.html
- Crates.io Guide: https://doc.rust-lang.org/cargo/reference/registries.html
- Full guide: See PUBLISHING.md

---

**Ready to publish?** Start with Step 1 above!
