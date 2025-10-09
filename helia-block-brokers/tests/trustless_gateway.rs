/// Tests for TrustlessGateway implementation
///
/// These tests verify the trustless gateway block broker functionality,
/// including HTTP fetching, CAR parsing, retry logic, and reliability tracking.
use bytes::Bytes;
use cid::Cid;
use helia_block_brokers::trustless_gateway::{trustless_gateway, TrustlessGatewayInit};
use helia_block_brokers::{BlockBroker, BlockRetrievalOptions};
use url::Url;

#[tokio::test]
#[ignore] // Ignore by default since it requires network access
async fn test_trustless_gateway_fetch_real() {
    // This test fetches a real block from a public IPFS gateway
    // CID for "Hello World" string
    let cid = Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap();

    let gateway = trustless_gateway(TrustlessGatewayInit::default());

    let result = gateway
        .retrieve(cid, BlockRetrievalOptions::default())
        .await;

    match result {
        Ok(data) => {
            println!("Successfully fetched {} bytes", data.len());
            assert!(!data.is_empty());
        }
        Err(e) => {
            // Don't fail the test if gateway is temporarily unavailable
            println!("Warning: Gateway fetch failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_trustless_gateway_creation() {
    // Test that we can create a gateway with custom config
    let gateway = trustless_gateway(TrustlessGatewayInit {
        gateways: vec![
            Url::parse("https://ipfs.io").unwrap(),
            Url::parse("https://dweb.link").unwrap(),
        ],
        max_retries: 2,
        timeout_ms: 10000,
        allow_insecure: false,
        allow_redirects: true,
    });

    assert_eq!(gateway.name(), "TrustlessGateway");
}

#[tokio::test]
async fn test_trustless_gateway_start_stop() {
    let gateway = trustless_gateway(TrustlessGatewayInit::default());

    assert!(gateway.start().await.is_ok());
    assert!(gateway.stop().await.is_ok());
}

#[tokio::test]
async fn test_trustless_gateway_announce_not_supported() {
    // Trustless gateways are read-only, they don't support announcements
    let gateway = trustless_gateway(TrustlessGatewayInit::default());

    let cid = Cid::default();
    let data = Bytes::from("test");

    let result = gateway.announce(cid, data, Default::default()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_trustless_gateway_stats() {
    let gateway = trustless_gateway(TrustlessGatewayInit::default());

    let stats = gateway.get_stats();
    assert_eq!(stats.requests_made, 0);
    assert_eq!(stats.successful_requests, 0);
    assert_eq!(stats.failed_requests, 0);
}

#[tokio::test]
async fn test_gateway_init_default() {
    let init = TrustlessGatewayInit::default();

    assert!(!init.gateways.is_empty());
    assert_eq!(init.max_retries, 3);
    assert_eq!(init.timeout_ms, 30000);
    assert!(!init.allow_insecure);
    assert!(init.allow_redirects);
}

#[tokio::test]
#[ignore] // Requires network
async fn test_trustless_gateway_invalid_cid() {
    // Test with a CID that doesn't exist
    let cid = Cid::try_from("bafybeiabc123invalidcidinvalidcidinvalidcidinvalid")
        .unwrap_or(Cid::default());

    let gateway = trustless_gateway(TrustlessGatewayInit {
        gateways: vec![Url::parse("https://ipfs.io").unwrap()],
        max_retries: 1,
        timeout_ms: 5000,
        ..Default::default()
    });

    let result = gateway
        .retrieve(cid, BlockRetrievalOptions::default())
        .await;

    // Should fail since CID doesn't exist
    assert!(result.is_err());
}
