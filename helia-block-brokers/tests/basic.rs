use helia_block_brokers::{BlockBroker, BrokerStats, ProviderType};

#[test]
fn test_provider_type() {
    let provider = ProviderType::Bitswap;
    assert_eq!(provider, ProviderType::Bitswap);
    
    let provider2 = ProviderType::Gateway;
    assert_eq!(provider2, ProviderType::Gateway);
}

#[test]
fn test_broker_stats_default() {
    let stats = BrokerStats::default();
    assert_eq!(stats.requests_made, 0);
    assert_eq!(stats.successful_requests, 0);
    assert_eq!(stats.failed_requests, 0);
}
