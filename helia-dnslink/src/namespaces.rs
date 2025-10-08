use crate::{DnsLinkError, DnsLinkResult, resolver::TxtRecord};
use cid::Cid;
use libp2p_identity::PeerId;
use tracing::debug;

pub fn parse_ipfs(value: &str, answer: TxtRecord) -> Result<DnsLinkResult, DnsLinkError> {
    let parts: Vec<&str> = value.split('/').filter(|s| !s.is_empty()).collect();
    
    if parts.is_empty() || parts[0] != "ipfs" {
        return Err(DnsLinkError::InvalidNamespace(format!(
            "Expected 'ipfs' namespace, got: {}",
            parts.get(0).unwrap_or(&"<empty>")
        )));
    }

    if parts.len() < 2 {
        return Err(DnsLinkError::InvalidFormat(
            "Missing CID after /ipfs/".to_string()
        ));
    }

    let cid_str = parts[1];
    let cid = Cid::try_from(cid_str)
        .map_err(|e| DnsLinkError::InvalidCid(format!("{}: {}", cid_str, e)))?;

    let path = if parts.len() > 2 {
        format!("/{}", parts[2..].join("/"))
    } else {
        String::new()
    };

    debug!("Parsed IPFS: cid={}, path={}", cid, path);

    Ok(DnsLinkResult::IPFS {
        answer,
        namespace: "ipfs".to_string(),
        cid,
        path,
    })
}

pub fn parse_ipns(value: &str, answer: TxtRecord) -> Result<DnsLinkResult, DnsLinkError> {
    let parts: Vec<&str> = value.split('/').filter(|s| !s.is_empty()).collect();
    
    if parts.is_empty() || parts[0] != "ipns" {
        return Err(DnsLinkError::InvalidNamespace(format!(
            "Expected 'ipns' namespace, got: {}",
            parts.get(0).unwrap_or(&"<empty>")
        )));
    }

    if parts.len() < 2 {
        return Err(DnsLinkError::InvalidFormat(
            "Missing peer ID after /ipns/".to_string()
        ));
    }

    let peer_id_str = parts[1];
    let peer_id = peer_id_str.parse::<PeerId>()
        .map_err(|e| DnsLinkError::InvalidPeerId(format!("{}: {}", peer_id_str, e)))?;

    let path = if parts.len() > 2 {
        format!("/{}", parts[2..].join("/"))
    } else {
        String::new()
    };

    debug!("Parsed IPNS: peer_id={}, path={}", peer_id, path);

    Ok(DnsLinkResult::IPNS {
        answer,
        namespace: "ipns".to_string(),
        peer_id,
        path,
    })
}

pub fn extract_dnslink_domain(value: &str) -> Result<String, DnsLinkError> {
    let parts: Vec<&str> = value.split('/').filter(|s| !s.is_empty()).collect();
    
    if parts.is_empty() || parts[0] != "dnslink" {
        return Err(DnsLinkError::InvalidNamespace(format!(
            "Expected 'dnslink' namespace, got: {}",
            parts.get(0).unwrap_or(&"<empty>")
        )));
    }

    if parts.len() < 2 {
        return Err(DnsLinkError::InvalidFormat(
            "Missing domain after /dnslink/".to_string()
        ));
    }

    let domain = parts[1].to_string();
    debug!("Extracted dnslink domain: {}", domain);
    Ok(domain)
}

pub fn parse_txt_value(txt_data: &str) -> Option<String> {
    let mut value = txt_data.trim();

    if (value.starts_with('"') && value.ends_with('"')) ||
       (value.starts_with('\'') && value.ends_with('\'')) {
        value = &value[1..value.len() - 1];
    }

    if !value.starts_with("dnslink=") {
        return None;
    }

    let value = value.strip_prefix("dnslink=")?;
    
    if !value.starts_with('/') {
        return None;
    }

    Some(value.to_string())
}
