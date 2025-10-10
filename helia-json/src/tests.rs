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

    // ============================================================================
    // EDGE CASE TESTS - Comprehensive coverage of corner cases
    // ============================================================================

    #[tokio::test]
    async fn test_empty_object() {
        let helia = create_test_helia().await;
        let json = Json::new(helia);

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Empty {}

        let empty = Empty {};
        let cid = json.add(&empty, None).await.unwrap();
        let retrieved: Empty = json.get(&cid, None).await.unwrap();

        assert_eq!(empty, retrieved);
    }

    #[tokio::test]
    async fn test_empty_collections() {
        let helia = create_test_helia().await;
        let json = Json::new(helia);

        // Empty vector
        let empty_vec: Vec<i32> = vec![];
        let cid_vec = json.add(&empty_vec, None).await.unwrap();
        let retrieved_vec: Vec<i32> = json.get(&cid_vec, None).await.unwrap();
        assert_eq!(empty_vec, retrieved_vec);
        assert!(retrieved_vec.is_empty());

        // Empty HashMap
        let empty_map: HashMap<String, String> = HashMap::new();
        let cid_map = json.add(&empty_map, None).await.unwrap();
        let retrieved_map: HashMap<String, String> = json.get(&cid_map, None).await.unwrap();
        assert_eq!(empty_map, retrieved_map);
        assert!(retrieved_map.is_empty());
    }

    #[tokio::test]
    async fn test_deeply_nested_structure() {
        let helia = create_test_helia().await;
        let json = Json::new(helia);

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Level3 {
            value: String,
        }

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Level2 {
            inner: Level3,
            count: i32,
        }

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Level1 {
            inner: Level2,
            items: Vec<String>,
        }

        let nested = Level1 {
            items: vec!["a".to_string(), "b".to_string()],
            inner: Level2 {
                count: 42,
                inner: Level3 {
                    value: "deep value".to_string(),
                },
            },
        };

        let cid = json.add(&nested, None).await.unwrap();
        let retrieved: Level1 = json.get(&cid, None).await.unwrap();

        assert_eq!(nested, retrieved);
        assert_eq!(retrieved.inner.inner.value, "deep value");
    }

    #[tokio::test]
    async fn test_large_array() {
        let helia = create_test_helia().await;
        let json = Json::new(helia);

        // Create array with 500 elements
        let large_array: Vec<i32> = (0..500).collect();
        let cid = json.add(&large_array, None).await.unwrap();
        let retrieved: Vec<i32> = json.get(&cid, None).await.unwrap();

        assert_eq!(large_array.len(), retrieved.len());
        assert_eq!(large_array, retrieved);
        assert_eq!(retrieved[0], 0);
        assert_eq!(retrieved[499], 499);
    }

    #[tokio::test]
    async fn test_special_values() {
        let helia = create_test_helia().await;
        let json = Json::new(helia);

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct SpecialValues {
            boolean_true: bool,
            boolean_false: bool,
            zero: i32,
            negative: i32,
            float: f64,
        }

        let special = SpecialValues {
            boolean_true: true,
            boolean_false: false,
            zero: 0,
            negative: -42,
            float: 3.14159,
        };

        let cid = json.add(&special, None).await.unwrap();
        let retrieved: SpecialValues = json.get(&cid, None).await.unwrap();

        assert_eq!(special, retrieved);
        assert!(retrieved.boolean_true);
        assert!(!retrieved.boolean_false);
        assert_eq!(retrieved.zero, 0);
        assert_eq!(retrieved.negative, -42);
    }

    #[tokio::test]
    async fn test_unicode_strings() {
        let helia = create_test_helia().await;
        let json = Json::new(helia);

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct UnicodeData {
            english: String,
            japanese: String,
            emoji: String,
        }

        let unicode = UnicodeData {
            english: "Hello World".to_string(),
            japanese: "こんにちは世界".to_string(),
            emoji: "🌍🚀🎉".to_string(),
        };

        let cid = json.add(&unicode, None).await.unwrap();
        let retrieved: UnicodeData = json.get(&cid, None).await.unwrap();

        assert_eq!(unicode, retrieved);
        assert_eq!(retrieved.japanese, "こんにちは世界");
        assert_eq!(retrieved.emoji, "🌍🚀🎉");
    }

    #[tokio::test]
    async fn test_null_handling() {
        let helia = create_test_helia().await;
        let json = Json::new(helia);

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct OptionalData {
            required: String,
            optional: Option<String>,
        }

        // Test with Some value
        let data_with_some = OptionalData {
            required: "always here".to_string(),
            optional: Some("maybe here".to_string()),
        };

        let cid_some = json.add(&data_with_some, None).await.unwrap();
        let retrieved_some: OptionalData = json.get(&cid_some, None).await.unwrap();
        assert_eq!(data_with_some, retrieved_some);
        assert!(retrieved_some.optional.is_some());

        // Test with None value
        let data_with_none = OptionalData {
            required: "always here".to_string(),
            optional: None,
        };

        let cid_none = json.add(&data_with_none, None).await.unwrap();
        let retrieved_none: OptionalData = json.get(&cid_none, None).await.unwrap();
        assert_eq!(data_with_none, retrieved_none);
        assert!(retrieved_none.optional.is_none());
    }

    #[tokio::test]
    async fn test_round_trip_consistency() {
        let helia = create_test_helia().await;
        let json = Json::new(helia);

        let original = TestData {
            message: "consistency test".to_string(),
            count: 789,
        };

        // Add and retrieve multiple times
        let cid1 = json.add(&original, None).await.unwrap();
        let retrieved1: TestData = json.get(&cid1, None).await.unwrap();
        
        let cid2 = json.add(&retrieved1, None).await.unwrap();
        let retrieved2: TestData = json.get(&cid2, None).await.unwrap();

        // All CIDs should be identical (deterministic)
        assert_eq!(cid1, cid2);
        
        // Data should remain unchanged
        assert_eq!(original, retrieved1);
        assert_eq!(original, retrieved2);
    }
}
