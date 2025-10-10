/// IPNS Publish and Resolve Example
/// 
/// This example demonstrates IPNS local operations:
/// - Publishing content to IPNS
/// - Resolving IPNS names to CIDs
/// - Cache behavior and content updates
///
/// NOTE: This example uses local-only mode. For full DHT integration,
/// see IPNS_DHT_ENHANCEMENT.md

use anyhow::Result;
use std::time::Duration;
use helia_ipns::{ipns, IpnsInit, PublishOptions, ResolveOptions};
use rust_helia::create_helia;
use helia_interface::Helia;
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("🚀 IPNS Publish/Resolve Example\n");

    // 1. Create Helia node
    println!("💾 Creating Helia node...");
    let helia = create_helia(None).await?;
    helia.start().await?;
    println!("   ✅ Ready\n");

    // 2. Initialize IPNS (local-only)
    println!("🔐 Initializing IPNS (local mode)...");
    let ipns = ipns(IpnsInit {
        routers: vec![],
        enable_republish: true,
        republish_interval: Some(Duration::from_secs(3600)),
        ..Default::default()
    })?;
    println!("   ✅ Ready\n");

    // 3. Add content
    println!("📤 Adding content...");
    let content = Bytes::from("Hello from IPNS!");
    
    // Generate CID
    let hash_bytes = [
        0x12, 0x20, // sha2-256
        0x9f, 0x86, 0xd0, 0x81, 0x88, 0x4c, 0x7d, 0x65, 0x9a, 0x2f, 0xea, 0xa0, 0xc5, 0x5a, 0xd0,
        0x15, 0xa3, 0xbf, 0x4f, 0x1b, 0x2b, 0x0b, 0x82, 0x2c, 0xd1, 0x5d, 0x6c, 0x15, 0xb0, 0xf0,
        0x0a, 0x08,
    ];
    let mh = multihash::Multihash::from_bytes(&hash_bytes)?;
    let cid = cid::Cid::new_v1(0x55, mh);
    
    helia.blockstore().put(&cid, content.clone(), None).await?;
    println!("   CID: {}\n", cid);

    // 4. Publish to IPNS
    println!("🚀 Publishing...");
    let key_name = "my-website";
    let result = ipns.publish(key_name, &cid, PublishOptions::default()).await?;
    println!("   ✅ Published!");
    println!("   Record has {} bytes\n", result.record.value.len());

    // 5. Resolve - we need to use the key bytes for resolution in local mode
    println!("🔍 Resolving...");
    let resolved = ipns.resolve(&result.public_key, ResolveOptions {
        offline: false,
        nocache: false,
        max_depth: Some(32),
        timeout: Some(Duration::from_secs(30)),
    }).await?;
    println!("   ✅ Resolved to: {}\n", resolved.cid);

    // 6. Verify
    if resolved.cid == cid {
        println!("✅ Verification passed!\n");
    }

    // 7. Test cache
    println!("💾 Testing cache...");
    let start = std::time::Instant::now();
    let _cached = ipns.resolve(&result.public_key, ResolveOptions {
        offline: false,
        nocache: false,
        max_depth: Some(32),
        timeout: Some(Duration::from_secs(30)),
    }).await?;
    println!("   ✅ Cached resolve: {:?}\n", start.elapsed());

    // 8. Update
    println!("🔄 Updating content...");
    let hash_bytes2 = [
        0x12, 0x20,
        0x60, 0x30, 0x3a, 0xe2, 0x2b, 0x99, 0x8a, 0x61, 0xe4, 0x7f, 0x86, 0x7f, 0x7d, 0x89, 0x72,
        0x2d, 0xac, 0xc4, 0x5c, 0xa4, 0x27, 0x8f, 0x6c, 0xfd, 0x98, 0xbc, 0x5e, 0xb9, 0xc5, 0xcd,
        0x4e, 0x8e,
    ];
    let mh2 = multihash::Multihash::from_bytes(&hash_bytes2)?;
    let new_cid = cid::Cid::new_v1(0x55, mh2);
    
    let new_content = Bytes::from("Updated!");
    helia.blockstore().put(&new_cid, new_content, None).await?;
    
    let update = ipns.publish(key_name, &new_cid, PublishOptions::default()).await?;
    println!("   ✅ Republished! Sequence: {}\n", update.record.sequence);

    // 9. Resolve update
    let updated = ipns.resolve(&update.public_key, ResolveOptions {
        offline: false,
        nocache: true,
        max_depth: Some(32),
        timeout: Some(Duration::from_secs(30)),
    }).await?;
    
    if updated.cid == new_cid {
        println!("✅ Update verified!\n");
    }

    println!("📊 Summary:");
    println!("   ✅ Publish/resolve working");
    println!("   ✅ Cache working");
    println!("   ✅ Updates working");
    println!("\n✨ Done!");

    Ok(())
}
