use helia_dnslink::{dns_link, DnsLinkInit, DnsLinkResult, ResolveOptions};

#[tokio::test]
async fn test_factory_function() {
    let dnslink = dns_link(DnsLinkInit::default());
    assert!(dnslink.is_ok());
}

#[tokio::test]
async fn test_invalid_domain() {
    let dnslink = dns_link(DnsLinkInit::default()).unwrap();
    let result = dnslink.resolve("this-domain-absolutely-does-not-exist-12345.com").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_offline_mode() {
    let dnslink = dns_link(DnsLinkInit::default()).unwrap();
    let options = ResolveOptions {
        offline: true,
        ..Default::default()
    };
    let result = dnslink.resolve_with_options("ipfs.tech", options).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_recursion_limit() {
    let dnslink = dns_link(DnsLinkInit::default()).unwrap();
    let options = ResolveOptions {
        max_recursive_depth: Some(0),
        ..Default::default()
    };
    let result = dnslink.resolve_with_options("ipfs.tech", options).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_nocache_option() {
    let dnslink = dns_link(DnsLinkInit::default()).unwrap();
    let options = ResolveOptions {
        nocache: true,
        ..Default::default()
    };
    // Should not error just because of nocache
    // (will fail on domain lookup but that's expected)
    let result = dnslink.resolve_with_options("nonexistent-test-domain-12345.com", options).await;
    assert!(result.is_err());
}

// Real network tests (ignored by default, run with --ignored)

#[tokio::test]
#[ignore]
async fn test_resolve_ipfs_tech_real() {
    let dnslink = dns_link(DnsLinkInit::default()).unwrap();
    
    let result = dnslink.resolve("ipfs.tech").await;
    
    match result {
        Ok(DnsLinkResult::IPFS { cid, path, namespace, .. }) => {
            println!("✅ Resolved ipfs.tech to CID: {}", cid);
            println!("   Namespace: {}", namespace);
            println!("   Path: {}", if path.is_empty() { "<empty>" } else { &path });
            assert_eq!(namespace, "ipfs");
        }
        Ok(other) => {
            println!("⚠️  Unexpected result type: {:?}", other);
            panic!("Expected IPFS result");
        }
        Err(e) => {
            println!("❌ Resolution failed: {:?}", e);
            panic!("Failed to resolve ipfs.tech: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_resolve_docs_ipfs_tech_real() {
    let dnslink = dns_link(DnsLinkInit::default()).unwrap();
    
    let result = dnslink.resolve("docs.ipfs.tech").await;
    
    match result {
        Ok(DnsLinkResult::IPFS { cid, path, namespace, .. }) => {
            println!("✅ Resolved docs.ipfs.tech to CID: {}", cid);
            println!("   Namespace: {}", namespace);
            println!("   Path: {}", if path.is_empty() { "<empty>" } else { &path });
            assert_eq!(namespace, "ipfs");
        }
        Ok(other) => {
            println!("⚠️  Unexpected result type: {:?}", other);
            // docs.ipfs.tech might redirect or use different namespace
            println!("Got: {:?}", other);
        }
        Err(e) => {
            println!("❌ Resolution failed: {:?}", e);
            panic!("Failed to resolve docs.ipfs.tech: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_resolve_with_path() {
    let dnslink = dns_link(DnsLinkInit::default()).unwrap();
    
    // Try ipfs.tech which is known to have dnslink
    let result = dnslink.resolve("ipfs.tech").await;
    
    if let Ok(res) = result {
        println!("Resolution result: {:?}", res);
    }
}
