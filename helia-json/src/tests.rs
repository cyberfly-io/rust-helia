#[cfg(test)]
mod tests {
    use crate::{AddOptions, Json, JsonError, JsonInterface};
    use helia_interface::Helia;
    use rust_helia::create_helia_default;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::sync::Arc;

    async fn create_test_helia() -> Arc<dyn Helia> {
        let helia = create_helia_default()
            .await
            .expect("Failed to create Helia");
        Arc::new(helia)
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestData {
        message: String,
        count: u32,
    }

    #[tokio::test]
    async fn test_add_and_get_simple_object() {
        let helia = create_test_helia().await;
        let json = Json::new(helia);

        let data = TestData {
            message: "hello world".to_string(),
            count: 42,
        };

        let cid = json.add(&data, None).await.unwrap();
        let retrieved: TestData = json.get(&cid, None).await.unwrap();

        assert_eq!(data, retrieved);
    }

    #[tokio::test]
    async fn test_add_and_get_hashmap() {
        let helia = create_test_helia().await;
        let json = Json::new(helia);

        let mut data = HashMap::new();
        data.insert("key1".to_string(), "value1".to_string());
        data.insert("key2".to_string(), "value2".to_string());

        let cid = json.add(&data, None).await.unwrap();
        let retrieved: HashMap<String, String> = json.get(&cid, None).await.unwrap();

        assert_eq!(data, retrieved);
    }

    #[tokio::test]
    async fn test_add_and_get_primitive_types() {
        let helia = create_test_helia().await;
        let json = Json::new(helia);

        // Test string
        let string_data = "hello world".to_string();
        let string_cid = json.add(&string_data, None).await.unwrap();
        let retrieved_string: String = json.get(&string_cid, None).await.unwrap();
        assert_eq!(string_data, retrieved_string);

        // Test number
        let number_data = 42i32;
        let number_cid = json.add(&number_data, None).await.unwrap();
        let retrieved_number: i32 = json.get(&number_cid, None).await.unwrap();
        assert_eq!(number_data, retrieved_number);

        // Test vector
        let vec_data = vec!["a", "b", "c"];
        let vec_cid = json.add(&vec_data, None).await.unwrap();
        let retrieved_vec: Vec<String> = json.get(&vec_cid, None).await.unwrap();
        assert_eq!(vec_data, retrieved_vec);
    }

    #[tokio::test]
    async fn test_add_and_get_nested_object() {
        let helia = create_test_helia().await;
        let json = Json::new(helia);

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct NestedData {
            inner: TestData,
            numbers: Vec<i32>,
        }

        let data = NestedData {
            inner: TestData {
                message: "nested".to_string(),
                count: 123,
            },
            numbers: vec![1, 2, 3, 4, 5],
        };

        let cid = json.add(&data, None).await.unwrap();
        let retrieved: NestedData = json.get(&cid, None).await.unwrap();

        assert_eq!(data, retrieved);
    }

    #[tokio::test]
    async fn test_deterministic_cids() {
        let helia1 = create_test_helia().await;
        let helia2 = create_test_helia().await;
        let json1 = Json::new(helia1);
        let json2 = Json::new(helia2);

        let data = TestData {
            message: "deterministic".to_string(),
            count: 999,
        };

        let cid1 = json1.add(&data, None).await.unwrap();
        let cid2 = json2.add(&data, None).await.unwrap();

        assert_eq!(cid1, cid2, "Same data should produce same CID");
    }

    #[tokio::test]
    async fn test_get_with_wrong_codec_fails() {
        let helia = create_test_helia().await;
        let json = Json::new(helia);

        // Create a CID with wrong codec (using DAG-CBOR codec)
        let data = b"test";
        let mut hash_bytes = [0u8; 32];
        hash_bytes[0..data.len().min(32)].copy_from_slice(&data[0..data.len().min(32)]);

        let mh: multihash::Multihash<64> = multihash::Multihash::wrap(0x12, &hash_bytes).unwrap();
        let wrong_cid = cid::Cid::new_v1(0x71, mh); // DAG-CBOR codec

        let result: Result<TestData, JsonError> = json.get(&wrong_cid, None).await;

        match result {
            Err(JsonError::InvalidCodec { expected, actual }) => {
                assert_eq!(expected, 0x0200); // JSON codec
                assert_eq!(actual, 0x71); // DAG-CBOR codec
            }
            _ => panic!("Expected InvalidCodec error"),
        }
    }

    #[tokio::test]
    async fn test_add_with_pinning() {
        let helia = create_test_helia().await;
        let json = Json::new(helia.clone());

        let data = TestData {
            message: "pinned data".to_string(),
            count: 456,
        };

        let options = AddOptions {
            pin: true,
            ..Default::default()
        };

        let cid = json.add(&data, Some(options)).await.unwrap();

        // Verify we can still retrieve the data (pinning shouldn't affect retrieval)
        let retrieved: TestData = json.get(&cid, None).await.unwrap();
        assert_eq!(data, retrieved);
    }
}
