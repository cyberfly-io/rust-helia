// DAG-PB encoding/decoding for UnixFS
// Based on: https://github.com/ipld/specs/blob/master/block-layer/codecs/dag-pb.md

use bytes::{BufMut, Bytes, BytesMut};
use cid::Cid;

/// DAG-PB Link
#[derive(Debug, Clone, PartialEq)]
pub struct PBLink {
    pub hash: Option<Cid>,
    pub name: Option<String>,
    pub tsize: Option<u64>,
}

/// DAG-PB Node
#[derive(Debug, Clone, PartialEq)]
pub struct PBNode {
    pub links: Vec<PBLink>,
    pub data: Option<Bytes>,
}

impl PBNode {
    pub fn new() -> Self {
        Self {
            links: Vec::new(),
            data: None,
        }
    }

    pub fn with_data(data: Bytes) -> Self {
        Self {
            links: Vec::new(),
            data: Some(data),
        }
    }

    pub fn add_link(&mut self, name: Option<String>, cid: Cid, size: u64) {
        self.links.push(PBLink {
            hash: Some(cid),
            name,
            tsize: Some(size),
        });
    }

    /// Encode to protobuf bytes
    pub fn encode(&self) -> Result<Bytes, String> {
        let mut buf = BytesMut::new();

        // Encode links (field 2)
        for link in &self.links {
            // Links are encoded as embedded messages (tag 2, wire type 2)
            let link_bytes = encode_link(link)?;
            encode_field(&mut buf, 2, WireType::LengthDelimited, &link_bytes);
        }

        // Encode data (field 1)
        if let Some(ref data) = self.data {
            encode_field(&mut buf, 1, WireType::LengthDelimited, data);
        }

        Ok(buf.freeze())
    }

    /// Decode from protobuf bytes
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let mut node = PBNode::new();
        let mut cursor = bytes;

        while !cursor.is_empty() {
            let (field_number, wire_type, rest) = decode_field_header(cursor)?;
            cursor = rest;

            match (field_number, wire_type) {
                (1, WireType::LengthDelimited) => {
                    // Data field
                    let (data, rest) = decode_bytes(cursor)?;
                    node.data = Some(Bytes::copy_from_slice(data));
                    cursor = rest;
                }
                (2, WireType::LengthDelimited) => {
                    // Link field
                    let (link_bytes, rest) = decode_bytes(cursor)?;
                    let link = decode_link(link_bytes)?;
                    node.links.push(link);
                    cursor = rest;
                }
                _ => {
                    return Err(format!(
                        "Unknown field: {} with wire type {:?}",
                        field_number, wire_type
                    ));
                }
            }
        }

        Ok(node)
    }
}

impl Default for PBNode {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum WireType {
    Varint = 0,
    LengthDelimited = 2,
}

fn encode_field(buf: &mut BytesMut, field_number: u32, wire_type: WireType, data: &[u8]) {
    // Encode tag (field_number << 3 | wire_type)
    let tag = (field_number << 3) | (wire_type as u32);
    encode_varint(buf, tag as u64);

    // Encode length for length-delimited fields
    if wire_type == WireType::LengthDelimited {
        encode_varint(buf, data.len() as u64);
    }

    // Encode data
    buf.put_slice(data);
}

fn encode_varint(buf: &mut BytesMut, mut value: u64) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buf.put_u8(byte);
        if value == 0 {
            break;
        }
    }
}

fn decode_field_header(data: &[u8]) -> Result<(u32, WireType, &[u8]), String> {
    let (tag, rest) = decode_varint(data)?;
    let field_number = (tag >> 3) as u32;
    let wire_type = match tag & 0x07 {
        0 => WireType::Varint,
        2 => WireType::LengthDelimited,
        wt => return Err(format!("Unsupported wire type: {}", wt)),
    };
    Ok((field_number, wire_type, rest))
}

fn decode_varint(data: &[u8]) -> Result<(u64, &[u8]), String> {
    let mut value = 0u64;
    let mut shift = 0;

    for (i, &byte) in data.iter().enumerate() {
        value |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            return Ok((value, &data[i + 1..]));
        }
        shift += 7;
        if shift >= 64 {
            return Err("Varint overflow".to_string());
        }
    }

    Err("Unexpected end of varint".to_string())
}

fn decode_bytes(data: &[u8]) -> Result<(&[u8], &[u8]), String> {
    let (length, rest) = decode_varint(data)?;
    let length = length as usize;

    if rest.len() < length {
        return Err(format!(
            "Not enough bytes: expected {}, got {}",
            length,
            rest.len()
        ));
    }

    Ok((&rest[..length], &rest[length..]))
}

fn encode_link(link: &PBLink) -> Result<Bytes, String> {
    let mut buf = BytesMut::new();

    // Encode hash (field 1) - CID bytes
    if let Some(ref cid) = link.hash {
        let cid_bytes = cid.to_bytes();
        encode_field(&mut buf, 1, WireType::LengthDelimited, &cid_bytes);
    }

    // Encode name (field 2)
    if let Some(ref name) = link.name {
        encode_field(&mut buf, 2, WireType::LengthDelimited, name.as_bytes());
    }

    // Encode tsize (field 3)
    if let Some(size) = link.tsize {
        let mut size_buf = BytesMut::new();
        encode_varint(&mut size_buf, size);
        let tag = (3 << 3) | (WireType::Varint as u32);
        encode_varint(&mut buf, tag as u64);
        buf.put_slice(&size_buf);
    }

    Ok(buf.freeze())
}

fn decode_link(bytes: &[u8]) -> Result<PBLink, String> {
    let mut link = PBLink {
        hash: None,
        name: None,
        tsize: None,
    };

    let mut cursor = bytes;

    while !cursor.is_empty() {
        let (field_number, wire_type, rest) = decode_field_header(cursor)?;
        cursor = rest;

        match (field_number, wire_type) {
            (1, WireType::LengthDelimited) => {
                // Hash field (CID)
                let (cid_bytes, rest) = decode_bytes(cursor)?;
                let cid = Cid::try_from(cid_bytes).map_err(|e| format!("Invalid CID: {}", e))?;
                link.hash = Some(cid);
                cursor = rest;
            }
            (2, WireType::LengthDelimited) => {
                // Name field
                let (name_bytes, rest) = decode_bytes(cursor)?;
                let name = String::from_utf8(name_bytes.to_vec())
                    .map_err(|e| format!("Invalid UTF-8 in name: {}", e))?;
                link.name = Some(name);
                cursor = rest;
            }
            (3, WireType::Varint) => {
                // Tsize field
                let (size, rest) = decode_varint(cursor)?;
                link.tsize = Some(size);
                cursor = rest;
            }
            _ => {
                return Err(format!("Unknown link field: {}", field_number));
            }
        }
    }

    Ok(link)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_empty_node() {
        let node = PBNode::new();
        let encoded = node.encode().unwrap();
        let decoded = PBNode::decode(&encoded).unwrap();
        assert_eq!(node, decoded);
    }

    #[test]
    fn test_encode_decode_node_with_data() {
        let node = PBNode::with_data(Bytes::from("hello world"));
        let encoded = node.encode().unwrap();
        let decoded = PBNode::decode(&encoded).unwrap();
        assert_eq!(node, decoded);
    }

    #[test]
    fn test_varint_encoding() {
        let test_cases = vec![0u64, 1, 127, 128, 255, 256, 65535, 1000000];

        for value in test_cases {
            let mut buf = BytesMut::new();
            encode_varint(&mut buf, value);
            let (decoded, rest) = decode_varint(&buf).unwrap();
            assert_eq!(value, decoded);
            assert!(rest.is_empty());
        }
    }
}
