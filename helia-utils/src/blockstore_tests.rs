//! Tests for blockstore implementations

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use cid::Cid;
    use futures::StreamExt;
    use helia_interface::{Blocks, InputPair};

    use crate::{BlockstoreConfig, SledBlockstore};

    fn create_test_blockstore() -> SledBlockstore {
        SledBlockstore::new(BlockstoreConfig {
            path: None,
            create_if_missing: true,
        })
        .unwrap()
    }

    fn create_test_cid() -> Cid {
        // Create a simple CID for testing using a fixed hash
        let hash_bytes = [
            0x12, 0x20, // sha2-256 code (0x12) and length (0x20 = 32 bytes)
            0x9f, 0x86, 0xd0, 0x81, 0x88, 0x4c, 0x7d, 0x65, 0x9a, 0x2f, 0xea, 0xa0, 0xc5, 0x5a,
            0xd0, 0x15, 0xa3, 0xbf, 0x4f, 0x1b, 0x2b, 0x0b, 0x82, 0x2c, 0xd1, 0x5d, 0x6c, 0x15,
            0xb0, 0xf0, 0x0a, 0x08,
        ];
        let mh = multihash::Multihash::from_bytes(&hash_bytes).unwrap();
        Cid::new_v1(0x55, mh) // 0x55 is raw codec
    }

    fn create_test_cid_2() -> Cid {
        let hash_bytes = [
            0x12, 0x20, // sha2-256 code (0x12) and length (0x20 = 32 bytes)
            0xef, 0x53, 0x7f, 0x25, 0xc8, 0x95, 0xbf, 0xa7, 0x82, 0x52, 0x65, 0x29, 0xa9, 0xb6,
            0x3d, 0x97, 0xaa, 0x63, 0x15, 0x64, 0xd5, 0xd7, 0x89, 0xc2, 0xb7, 0x65, 0x44, 0x8c,
            0x86, 0x35, 0xfb, 0x6c,
        ];
        let mh = multihash::Multihash::from_bytes(&hash_bytes).unwrap();
        Cid::new_v1(0x55, mh)
    }

    fn create_test_cid_3() -> Cid {
        let hash_bytes = [
            0x12, 0x20, // sha2-256 code (0x12) and length (0x20 = 32 bytes)
            0x31, 0x5f, 0x5b, 0xdb, 0x76, 0xd0, 0x78, 0xc4, 0x3b, 0x8a, 0xc0, 0x06, 0x4e, 0x4a,
            0x01, 0x64, 0x61, 0x2b, 0x1f, 0xce, 0x77, 0xc8, 0x69, 0x34, 0x5b, 0xfc, 0x94, 0xc7,
            0x58, 0x94, 0xed, 0xd3,
        ];
        let mh = multihash::Multihash::from_bytes(&hash_bytes).unwrap();
        Cid::new_v1(0x55, mh)
    }

    #[tokio::test]
    async fn test_put_and_get() {
        let blockstore = create_test_blockstore();
        let cid = create_test_cid();
        let data = Bytes::from("hello world");

        // Put block
        let result_cid = blockstore.put(&cid, data.clone(), None).await.unwrap();
        assert_eq!(result_cid, cid);

        // Get block
        let retrieved = blockstore.get(&cid, None).await.unwrap();
        assert_eq!(retrieved, data);

        // Check has
        let exists = blockstore.has(&cid, None).await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_get_many_cids() {
        let blockstore = create_test_blockstore();
        let cid1 = create_test_cid();
        let cid2 = create_test_cid_2();

        let data1 = Bytes::from("hello world");
        let data2 = Bytes::from("hello world 2");

        // Put blocks
        blockstore.put(&cid1, data1.clone(), None).await.unwrap();
        blockstore.put(&cid2, data2.clone(), None).await.unwrap();

        // Get many
        let mut stream = blockstore
            .get_many_cids(vec![cid1, cid2], None)
            .await
            .unwrap();
        let mut results = Vec::new();
        while let Some(result) = stream.next().await {
            results.push(result);
        }

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());

        let pair1 = results[0].as_ref().unwrap();
        let pair2 = results[1].as_ref().unwrap();

        // Results might be in different order, so check both possibilities
        if pair1.cid == cid1 {
            assert_eq!(pair1.block, data1);
            assert_eq!(pair2.cid, cid2);
            assert_eq!(pair2.block, data2);
        } else {
            assert_eq!(pair1.cid, cid2);
            assert_eq!(pair1.block, data2);
            assert_eq!(pair2.cid, cid1);
            assert_eq!(pair2.block, data1);
        }
    }

    #[tokio::test]
    async fn test_put_many_blocks() {
        let blockstore = create_test_blockstore();
        let cid1 = create_test_cid();
        let cid2 = create_test_cid_2();

        let data1 = Bytes::from("hello world");
        let data2 = Bytes::from("hello world 2");

        let input_pairs = vec![
            InputPair {
                cid: Some(cid1),
                block: data1.clone(),
            },
            InputPair {
                cid: Some(cid2),
                block: data2.clone(),
            },
        ];

        // Put many blocks
        let mut stream = blockstore.put_many_blocks(input_pairs, None).await.unwrap();
        let mut result_cids = Vec::new();
        while let Some(cid) = stream.next().await {
            result_cids.push(cid);
        }

        assert_eq!(result_cids.len(), 2);

        // Verify blocks were stored
        let retrieved1 = blockstore.get(&cid1, None).await.unwrap();
        let retrieved2 = blockstore.get(&cid2, None).await.unwrap();
        assert_eq!(retrieved1, data1);
        assert_eq!(retrieved2, data2);
    }

    #[tokio::test]
    async fn test_has_many_cids() {
        let blockstore = create_test_blockstore();
        let cid1 = create_test_cid();
        let cid2 = create_test_cid_2();
        let cid3 = create_test_cid_3();

        // Put only cid1 and cid2
        blockstore
            .put(&cid1, Bytes::from("hello world"), None)
            .await
            .unwrap();
        blockstore
            .put(&cid2, Bytes::from("hello world 2"), None)
            .await
            .unwrap();

        // Check has many
        let mut stream = blockstore
            .has_many_cids(vec![cid1, cid2, cid3], None)
            .await
            .unwrap();
        let mut results = Vec::new();
        while let Some(exists) = stream.next().await {
            results.push(exists);
        }

        assert_eq!(results.len(), 3);
        assert!(results[0]); // cid1 exists
        assert!(results[1]); // cid2 exists
        assert!(!results[2]); // cid3 doesn't exist
    }

    #[tokio::test]
    async fn test_get_all() {
        let blockstore = create_test_blockstore();
        let cid1 = create_test_cid();
        let cid2 = create_test_cid_2();

        let data1 = Bytes::from("hello world");
        let data2 = Bytes::from("hello world 2");

        // Put blocks
        blockstore.put(&cid1, data1.clone(), None).await.unwrap();
        blockstore.put(&cid2, data2.clone(), None).await.unwrap();

        // Get all
        let mut stream = blockstore.get_all(None).await.unwrap();
        let mut results = Vec::new();
        while let Some(pair) = stream.next().await {
            results.push(pair);
        }

        assert_eq!(results.len(), 2);
        // Results can be in any order, so just check that both CIDs are present
        let result_cids: Vec<Cid> = results.iter().map(|p| p.cid).collect();
        assert!(result_cids.contains(&cid1));
        assert!(result_cids.contains(&cid2));
    }

    #[tokio::test]
    async fn test_delete_many_cids() {
        let blockstore = create_test_blockstore();
        let cid1 = create_test_cid();
        let cid2 = create_test_cid_2();

        let data1 = Bytes::from("hello world");
        let data2 = Bytes::from("hello world 2");

        // Put blocks
        blockstore.put(&cid1, data1, None).await.unwrap();
        blockstore.put(&cid2, data2, None).await.unwrap();

        // Verify they exist
        assert!(blockstore.has(&cid1, None).await.unwrap());
        assert!(blockstore.has(&cid2, None).await.unwrap());

        // Delete many
        let mut stream = blockstore
            .delete_many_cids(vec![cid1, cid2], None)
            .await
            .unwrap();
        let mut deleted_cids = Vec::new();
        while let Some(cid) = stream.next().await {
            deleted_cids.push(cid);
        }

        assert_eq!(deleted_cids.len(), 2);
        assert!(deleted_cids.contains(&cid1));
        assert!(deleted_cids.contains(&cid2));

        // Verify they no longer exist
        assert!(!blockstore.has(&cid1, None).await.unwrap());
        assert!(!blockstore.has(&cid2, None).await.unwrap());
    }
}
