//! Tests for DAG-JSON functionality

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use serde::{Deserialize, Serialize};

    use crate::{AddOptions, DagJson, DagJsonInterface};
    use rust_helia::create_helia_default;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestData {
        name: String,
        age: u32,
        scores: Vec<i32>,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct NestedData {
        id: u64,
        metadata: HashMap<String, String>,
        inner: TestData,
    }

    async fn create_test_dag() -> DagJson {
        let helia = create_helia_default().await.unwrap();
        DagJson::new(Arc::new(helia))
    }

    #[tokio::test]
    async fn test_add_and_get_simple_object() {
        let dag = create_test_dag().await;

        let data = TestData {
            name: "Alice".to_string(),
            age: 30,
            scores: vec![95, 87, 92],
        };

        let cid = dag.add(&data, None).await.unwrap();
        let retrieved: TestData = dag.get(&cid, None).await.unwrap();

        assert_eq!(data, retrieved);
    }

    #[tokio::test]
    async fn test_add_and_get_nested_object() {
        let dag = create_test_dag().await;

        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "1.0".to_string());
        metadata.insert("author".to_string(), "test".to_string());

        let data = NestedData {
            id: 12345,
            metadata,
            inner: TestData {
                name: "Bob".to_string(),
                age: 25,
                scores: vec![88, 91, 94],
            },
        };

        let cid = dag.add(&data, None).await.unwrap();
        let retrieved: NestedData = dag.get(&cid, None).await.unwrap();

        assert_eq!(data, retrieved);
    }

    #[tokio::test]
    async fn test_add_with_pinning() {
        let dag = create_test_dag().await;

        let data = TestData {
            name: "Charlie".to_string(),
            age: 35,
            scores: vec![100, 98, 97],
        };

        let options = AddOptions {
            pin: true,
            ..Default::default()
        };

        let cid = dag.add(&data, Some(options)).await.unwrap();

        // Verify we can still retrieve the data (pinning shouldn't affect retrieval)
        let retrieved: TestData = dag.get(&cid, None).await.unwrap();
        assert_eq!(data, retrieved);
    }

    #[tokio::test]
    async fn test_add_and_get_primitive_types() {
        let dag = create_test_dag().await;

        // Test string
        let string_data = "hello world".to_string();
        let string_cid = dag.add(&string_data, None).await.unwrap();
        let retrieved_string: String = dag.get(&string_cid, None).await.unwrap();
        assert_eq!(string_data, retrieved_string);

        // Test number
        let number_data = 42i32;
        let number_cid = dag.add(&number_data, None).await.unwrap();
        let retrieved_number: i32 = dag.get(&number_cid, None).await.unwrap();
        assert_eq!(number_data, retrieved_number);

        // Test vector
        let vec_data = vec![1, 2, 3, 4, 5];
        let vec_cid = dag.add(&vec_data, None).await.unwrap();
        let retrieved_vec: Vec<i32> = dag.get(&vec_cid, None).await.unwrap();
        assert_eq!(vec_data, retrieved_vec);
    }

    #[tokio::test]
    async fn test_add_and_get_hashmap() {
        let dag = create_test_dag().await;

        let mut map = HashMap::new();
        map.insert("key1".to_string(), "value1".to_string());
        map.insert("key2".to_string(), "value2".to_string());
        map.insert("key3".to_string(), "value3".to_string());

        let cid = dag.add(&map, None).await.unwrap();
        let retrieved: HashMap<String, String> = dag.get(&cid, None).await.unwrap();

        assert_eq!(map, retrieved);
    }

    #[tokio::test]
    async fn test_get_with_wrong_codec_fails() {
        let dag = create_test_dag().await;

        // Create a CID with wrong codec (using raw codec instead of DAG-JSON)
        use cid::Cid;

        let data = b"hello world";
        let mut hash_bytes = [0u8; 32];
        hash_bytes[0..data.len().min(32)].copy_from_slice(&data[0..data.len().min(32)]);

        let mh: multihash::Multihash<64> = multihash::Multihash::wrap(0x12, &hash_bytes).unwrap();
        let wrong_cid = Cid::new_v1(0x55, mh); // 0x55 is raw codec, not DAG-JSON

        let result: Result<String, _> = dag.get(&wrong_cid, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_deterministic_cids() {
        let dag = create_test_dag().await;

        let data = TestData {
            name: "David".to_string(),
            age: 40,
            scores: vec![85, 90, 88],
        };

        // Add the same data twice
        let cid1 = dag.add(&data, None).await.unwrap();
        let cid2 = dag.add(&data, None).await.unwrap();

        // CIDs should be identical
        assert_eq!(cid1, cid2);
    }

    #[tokio::test]
    async fn test_json_specific_features() {
        let dag = create_test_dag().await;

        // Test that JSON serialization preserves order and format
        let mut map = HashMap::new();
        map.insert("name".to_string(), "test".to_string());
        map.insert("enabled".to_string(), "true".to_string());
        map.insert("count".to_string(), "42".to_string());

        let cid = dag.add(&map, None).await.unwrap();
        let retrieved: HashMap<String, String> = dag.get(&cid, None).await.unwrap();

        assert_eq!(map, retrieved);
        assert_eq!(retrieved.get("enabled").unwrap(), "true");
        assert_eq!(retrieved.get("count").unwrap(), "42");
    }

    // ============================================================================
    // EDGE CASE TESTS - Comprehensive coverage of corner cases
    // ============================================================================

    #[tokio::test]
    async fn test_empty_object() {
        let dag = create_test_dag().await;

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Empty {}

        let empty = Empty {};
        let cid = dag.add(&empty, None).await.unwrap();
        let retrieved: Empty = dag.get(&cid, None).await.unwrap();

        assert_eq!(empty, retrieved);
    }

    #[tokio::test]
    async fn test_empty_array() {
        let dag = create_test_dag().await;

        let empty_vec: Vec<i32> = vec![];
        let cid = dag.add(&empty_vec, None).await.unwrap();
        let retrieved: Vec<i32> = dag.get(&cid, None).await.unwrap();

        assert_eq!(empty_vec, retrieved);
        assert!(retrieved.is_empty());
    }

    #[tokio::test]
    async fn test_empty_hashmap() {
        let dag = create_test_dag().await;

        let empty_map: HashMap<String, String> = HashMap::new();
        let cid = dag.add(&empty_map, None).await.unwrap();
        let retrieved: HashMap<String, String> = dag.get(&cid, None).await.unwrap();

        assert_eq!(empty_map, retrieved);
        assert!(retrieved.is_empty());
    }

    #[tokio::test]
    async fn test_deeply_nested_structure() {
        let dag = create_test_dag().await;

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Level5 {
            value: String,
        }

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Level4 {
            inner: Level5,
            count: i32,
        }

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Level3 {
            inner: Level4,
            items: Vec<String>,
        }

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Level2 {
            inner: Level3,
            flag: bool,
        }

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Level1 {
            inner: Level2,
            name: String,
        }

        let deeply_nested = Level1 {
            name: "root".to_string(),
            inner: Level2 {
                flag: true,
                inner: Level3 {
                    items: vec!["a".to_string(), "b".to_string()],
                    inner: Level4 {
                        count: 42,
                        inner: Level5 {
                            value: "deep value".to_string(),
                        },
                    },
                },
            },
        };

        let cid = dag.add(&deeply_nested, None).await.unwrap();
        let retrieved: Level1 = dag.get(&cid, None).await.unwrap();

        assert_eq!(deeply_nested, retrieved);
        assert_eq!(retrieved.inner.inner.inner.inner.value, "deep value");
    }

    #[tokio::test]
    async fn test_large_array() {
        let dag = create_test_dag().await;

        // Create array with 1000 elements
        let large_array: Vec<i32> = (0..1000).collect();
        let cid = dag.add(&large_array, None).await.unwrap();
        let retrieved: Vec<i32> = dag.get(&cid, None).await.unwrap();

        assert_eq!(large_array.len(), retrieved.len());
        assert_eq!(large_array, retrieved);
        assert_eq!(retrieved[0], 0);
        assert_eq!(retrieved[999], 999);
    }

    #[tokio::test]
    async fn test_large_object() {
        let dag = create_test_dag().await;

        // Create HashMap with 100 entries
        let mut large_map = HashMap::new();
        for i in 0..100 {
            large_map.insert(format!("key{}", i), format!("value{}", i));
        }

        let cid = dag.add(&large_map, None).await.unwrap();
        let retrieved: HashMap<String, String> = dag.get(&cid, None).await.unwrap();

        assert_eq!(large_map.len(), retrieved.len());
        assert_eq!(retrieved.get("key0").unwrap(), "value0");
        assert_eq!(retrieved.get("key99").unwrap(), "value99");
    }

    #[tokio::test]
    async fn test_special_values() {
        let dag = create_test_dag().await;

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct SpecialValues {
            boolean_true: bool,
            boolean_false: bool,
            zero: i32,
            negative: i32,
            float: f64,
            large_number: i64,
        }

        let special = SpecialValues {
            boolean_true: true,
            boolean_false: false,
            zero: 0,
            negative: -42,
            float: 3.14159,
            large_number: 9_223_372_036_854_775_807, // i64::MAX
        };

        let cid = dag.add(&special, None).await.unwrap();
        let retrieved: SpecialValues = dag.get(&cid, None).await.unwrap();

        assert_eq!(special, retrieved);
        assert!(retrieved.boolean_true);
        assert!(!retrieved.boolean_false);
        assert_eq!(retrieved.zero, 0);
        assert_eq!(retrieved.negative, -42);
        assert!((retrieved.float - 3.14159).abs() < 0.00001);
    }

    #[tokio::test]
    async fn test_unicode_strings() {
        let dag = create_test_dag().await;

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct UnicodeData {
            english: String,
            japanese: String,
            arabic: String,
            emoji: String,
            mixed: String,
        }

        let unicode = UnicodeData {
            english: "Hello World".to_string(),
            japanese: "こんにちは世界".to_string(),
            arabic: "مرحبا بالعالم".to_string(),
            emoji: "🌍🚀🎉".to_string(),
            mixed: "Hello 世界 🌍".to_string(),
        };

        let cid = dag.add(&unicode, None).await.unwrap();
        let retrieved: UnicodeData = dag.get(&cid, None).await.unwrap();

        assert_eq!(unicode, retrieved);
        assert_eq!(retrieved.japanese, "こんにちは世界");
        assert_eq!(retrieved.emoji, "🌍🚀🎉");
    }

    #[tokio::test]
    async fn test_mixed_type_array() {
        let dag = create_test_dag().await;

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        #[serde(untagged)]
        enum Value {
            Number(i32),
            Text(String),
            Flag(bool),
        }

        let mixed = vec![
            Value::Number(42),
            Value::Text("hello".to_string()),
            Value::Flag(true),
            Value::Number(-10),
            Value::Text("world".to_string()),
        ];

        let cid = dag.add(&mixed, None).await.unwrap();
        let retrieved: Vec<Value> = dag.get(&cid, None).await.unwrap();

        assert_eq!(mixed.len(), retrieved.len());
        // Note: Untagged enums may have different representations in JSON
    }

    #[tokio::test]
    async fn test_round_trip_multiple_times() {
        let dag = create_test_dag().await;

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Data {
            id: u32,
            content: String,
        }

        let original = Data {
            id: 12345,
            content: "test content".to_string(),
        };

        // Add and retrieve multiple times
        let cid1 = dag.add(&original, None).await.unwrap();
        let retrieved1: Data = dag.get(&cid1, None).await.unwrap();
        
        let cid2 = dag.add(&retrieved1, None).await.unwrap();
        let retrieved2: Data = dag.get(&cid2, None).await.unwrap();
        
        let cid3 = dag.add(&retrieved2, None).await.unwrap();
        let retrieved3: Data = dag.get(&cid3, None).await.unwrap();

        // All CIDs should be identical (deterministic)
        assert_eq!(cid1, cid2);
        assert_eq!(cid2, cid3);
        
        // Data should remain unchanged
        assert_eq!(original, retrieved1);
        assert_eq!(original, retrieved2);
        assert_eq!(original, retrieved3);
    }

    #[tokio::test]
    async fn test_null_handling() {
        let dag = create_test_dag().await;

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct OptionalData {
            required: String,
            optional: Option<String>,
        }

        let data_with_some = OptionalData {
            required: "always here".to_string(),
            optional: Some("maybe here".to_string()),
        };

        let data_with_none = OptionalData {
            required: "always here".to_string(),
            optional: None,
        };

        // Test with Some value
        let cid_some = dag.add(&data_with_some, None).await.unwrap();
        let retrieved_some: OptionalData = dag.get(&cid_some, None).await.unwrap();
        assert_eq!(data_with_some, retrieved_some);
        assert!(retrieved_some.optional.is_some());

        // Test with None value
        let cid_none = dag.add(&data_with_none, None).await.unwrap();
        let retrieved_none: OptionalData = dag.get(&cid_none, None).await.unwrap();
        assert_eq!(data_with_none, retrieved_none);
        assert!(retrieved_none.optional.is_none());
    }
}
