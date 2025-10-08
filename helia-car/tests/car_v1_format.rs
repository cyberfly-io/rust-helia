/// Tests for CAR v1 format compliance
/// 
/// These tests verify that our implementation correctly reads and writes
/// CAR v1 files according to the specification.

use bytes::Bytes;
use cid::Cid;
use helia_car::{CarBlock, CarHeader, CarReader, CarWriter};
use std::io::Cursor;

#[tokio::test]
async fn test_car_v1_round_trip() {
    // Create a simple CAR file
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let mut writer = CarWriter::new(cursor);
    
    // Create a test CID (using a simple identity hash for testing)
    let test_data = b"hello world";
    let cid = Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap();
    
    // Write header
    let header = CarHeader {
        version: 1,
        roots: vec![cid],
    };
    writer.write_header(&header).await.unwrap();
    
    // Write a block
    let block = CarBlock {
        cid,
        data: Bytes::from(&test_data[..]),
    };
    writer.write_block(&block).await.unwrap();
    
    // Finish writing
    writer.finish().await.unwrap();
    
    // Now read it back
    let cursor = Cursor::new(&buffer);
    let mut reader = CarReader::new(cursor);
    
    // Read header
    let read_header = reader.read_header().await.unwrap();
    assert_eq!(read_header.version, 1);
    assert_eq!(read_header.roots.len(), 1);
    assert_eq!(read_header.roots[0], cid);
    
    // Read block
    let read_block = reader.read_block().await.unwrap().unwrap();
    assert_eq!(read_block.cid, cid);
    assert_eq!(read_block.data.as_ref(), test_data);
    
    // Verify end of file
    assert!(reader.read_block().await.unwrap().is_none());
}

#[tokio::test]
async fn test_car_v1_multiple_blocks() {
    // Create a CAR file with multiple blocks
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let mut writer = CarWriter::new(cursor);
    
    // Create test CIDs
    let cid1 = Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap();
    let cid2 = Cid::try_from("bafybeihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku").unwrap();
    
    // Write header with multiple roots
    let header = CarHeader {
        version: 1,
        roots: vec![cid1, cid2],
    };
    writer.write_header(&header).await.unwrap();
    
    // Write blocks
    let block1 = CarBlock {
        cid: cid1,
        data: Bytes::from("first block"),
    };
    let block2 = CarBlock {
        cid: cid2,
        data: Bytes::from("second block"),
    };
    
    writer.write_block(&block1).await.unwrap();
    writer.write_block(&block2).await.unwrap();
    writer.finish().await.unwrap();
    
    // Read it back
    let cursor = Cursor::new(&buffer);
    let mut reader = CarReader::new(cursor);
    
    let read_header = reader.read_header().await.unwrap();
    assert_eq!(read_header.roots.len(), 2);
    
    let blocks = reader.read_all_blocks().await.unwrap();
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].cid, cid1);
    assert_eq!(blocks[0].data.as_ref(), b"first block");
    assert_eq!(blocks[1].cid, cid2);
    assert_eq!(blocks[1].data.as_ref(), b"second block");
}

#[tokio::test]
async fn test_car_v1_empty_roots() {
    // CAR file with no roots
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let mut writer = CarWriter::new(cursor);
    
    let header = CarHeader {
        version: 1,
        roots: vec![],
    };
    writer.write_header(&header).await.unwrap();
    writer.finish().await.unwrap();
    
    // Read it back
    let cursor = Cursor::new(&buffer);
    let mut reader = CarReader::new(cursor);
    
    let read_header = reader.read_header().await.unwrap();
    assert_eq!(read_header.version, 1);
    assert_eq!(read_header.roots.len(), 0);
    
    // No blocks to read
    assert!(reader.read_block().await.unwrap().is_none());
}

#[tokio::test]
async fn test_car_v1_large_block() {
    // Test with a larger block (1MB)
    let large_data = vec![0u8; 1024 * 1024];
    
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let mut writer = CarWriter::new(cursor);
    
    let cid = Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap();
    
    let header = CarHeader {
        version: 1,
        roots: vec![cid],
    };
    writer.write_header(&header).await.unwrap();
    
    let block = CarBlock {
        cid,
        data: Bytes::from(large_data.clone()),
    };
    writer.write_block(&block).await.unwrap();
    writer.finish().await.unwrap();
    
    // Read it back
    let cursor = Cursor::new(&buffer);
    let mut reader = CarReader::new(cursor);
    
    reader.read_header().await.unwrap();
    let read_block = reader.read_block().await.unwrap().unwrap();
    assert_eq!(read_block.data.len(), 1024 * 1024);
    assert_eq!(read_block.data, large_data.as_slice());
}

#[tokio::test]
async fn test_car_v1_invalid_version() {
    // Try to write a header with wrong version
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let mut writer = CarWriter::new(cursor);
    
    let header = CarHeader {
        version: 2, // Invalid - we only support v1
        roots: vec![],
    };
    
    // Should fail
    assert!(writer.write_header(&header).await.is_err());
}

#[tokio::test]
async fn test_car_v1_find_block() {
    // Test the find_block method
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let mut writer = CarWriter::new(cursor);
    
    let cid1 = Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap();
    let cid2 = Cid::try_from("bafybeihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku").unwrap();
    let cid3 = Cid::try_from("bafybeihkoviema7g3gxyt6la7vd5ho32ictqbilu3wnlo3rs7ewhnp7lly").unwrap();
    
    let header = CarHeader {
        version: 1,
        roots: vec![cid1],
    };
    writer.write_header(&header).await.unwrap();
    
    writer.write_block(&CarBlock {
        cid: cid1,
        data: Bytes::from("first"),
    }).await.unwrap();
    
    writer.write_block(&CarBlock {
        cid: cid2,
        data: Bytes::from("second"),
    }).await.unwrap();
    
    writer.write_block(&CarBlock {
        cid: cid3,
        data: Bytes::from("third"),
    }).await.unwrap();
    
    writer.finish().await.unwrap();
    
    // Find the second block
    let cursor = Cursor::new(&buffer);
    let mut reader = CarReader::new(cursor);
    reader.read_header().await.unwrap();
    
    let found = reader.find_block(&cid2).await.unwrap().unwrap();
    assert_eq!(found.as_ref(), b"second");
}
