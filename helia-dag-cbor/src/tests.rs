//! Tests for DAG-CBOR functionality

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use serde::{Deserialize, Serialize};

    use crate::{AddOptions, DagCbor, DagCborInterface};
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

    async fn create_test_dag() -> DagCbor {
        let helia = create_helia_default().await.unwrap();
        DagCbor::new(Arc::new(helia))
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

        // Create a CID with wrong codec (using raw codec instead of DAG-CBOR)
        use cid::Cid;

        let data = b"hello world";
        let mut hash_bytes = [0u8; 32];
        hash_bytes[0..data.len().min(32)].copy_from_slice(&data[0..data.len().min(32)]);

        let mh: multihash::Multihash<64> = multihash::Multihash::wrap(0x12, &hash_bytes).unwrap();
        let wrong_cid = Cid::new_v1(0x55, mh); // 0x55 is raw codec, not DAG-CBOR

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

    // ====================================================================
    // Edge Case Tests
    // ====================================================================

    #[tokio::test]
    async fn test_empty_object() {
        let dag = create_test_dag().await;

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct EmptyStruct {}

        let empty = EmptyStruct {};
        let cid = dag.add(&empty, None).await.unwrap();
        let retrieved: EmptyStruct = dag.get(&cid, None).await.unwrap();
        assert_eq!(empty, retrieved);
    }

    #[tokio::test]
    async fn test_empty_array() {
        let dag = create_test_dag().await;

        let empty_vec: Vec<i32> = vec![];
        let cid = dag.add(&empty_vec, None).await.unwrap();
        let retrieved: Vec<i32> = dag.get(&cid, None).await.unwrap();
        assert_eq!(empty_vec, retrieved);
    }

    #[tokio::test]
    async fn test_empty_hashmap() {
        let dag = create_test_dag().await;

        let empty_map: HashMap<String, String> = HashMap::new();
        let cid = dag.add(&empty_map, None).await.unwrap();
        let retrieved: HashMap<String, String> = dag.get(&cid, None).await.unwrap();
        assert_eq!(empty_map, retrieved);
    }

    #[tokio::test]
    async fn test_deeply_nested_structure() {
        let dag = create_test_dag().await;

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Level5 {
            value: i32,
        }

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Level4 {
            inner: Level5,
        }

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Level3 {
            inner: Level4,
        }

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Level2 {
            inner: Level3,
        }

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Level1 {
            inner: Level2,
        }

        let nested = Level1 {
            inner: Level2 {
                inner: Level3 {
                    inner: Level4 {
                        inner: Level5 { value: 42 },
                    },
                },
            },
        };

        let cid = dag.add(&nested, None).await.unwrap();
        let retrieved: Level1 = dag.get(&cid, None).await.unwrap();
        assert_eq!(nested, retrieved);
    }

    #[tokio::test]
    async fn test_large_array() {
        let dag = create_test_dag().await;

        // Create array with 1000 elements
        let large_array: Vec<i32> = (0..1000).collect();
        let cid = dag.add(&large_array, None).await.unwrap();
        let retrieved: Vec<i32> = dag.get(&cid, None).await.unwrap();
        assert_eq!(large_array, retrieved);
    }

    #[tokio::test]
    async fn test_large_object() {
        let dag = create_test_dag().await;

        // Create map with 100 entries
        let mut large_map = HashMap::new();
        for i in 0..100 {
            large_map.insert(format!("key{}", i), format!("value{}", i));
        }

        let cid = dag.add(&large_map, None).await.unwrap();
        let retrieved: HashMap<String, String> = dag.get(&cid, None).await.unwrap();
        assert_eq!(large_map, retrieved);
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
            float_val: f64,
        }

        let special = SpecialValues {
            boolean_true: true,
            boolean_false: false,
            zero: 0,
            negative: -123,
            float_val: 3.14159,
        };

        let cid = dag.add(&special, None).await.unwrap();
        let retrieved: SpecialValues = dag.get(&cid, None).await.unwrap();
        assert_eq!(special, retrieved);
    }

    #[tokio::test]
    async fn test_unicode_strings() {
        let dag = create_test_dag().await;

        let unicode_data = vec![
            "Hello 世界".to_string(),
            "Привет мир".to_string(),
            "مرحبا بالعالم".to_string(),
            "שלום עולם".to_string(),
            "🌍🌎🌏".to_string(),
        ];

        let cid = dag.add(&unicode_data, None).await.unwrap();
        let retrieved: Vec<String> = dag.get(&cid, None).await.unwrap();
        assert_eq!(unicode_data, retrieved);
    }

    #[tokio::test]
    async fn test_mixed_type_array() {
        let dag = create_test_dag().await;

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        #[serde(untagged)]
        enum MixedValue {
            Int(i32),
            String(String),
            Bool(bool),
        }

        let mixed = vec![
            MixedValue::Int(42),
            MixedValue::String("hello".to_string()),
            MixedValue::Bool(true),
            MixedValue::Int(-10),
        ];

        let cid = dag.add(&mixed, None).await.unwrap();
        let retrieved: Vec<MixedValue> = dag.get(&cid, None).await.unwrap();
        assert_eq!(mixed, retrieved);
    }

    #[tokio::test]
    async fn test_round_trip_multiple_times() {
        let dag = create_test_dag().await;

        let original = TestData {
            name: "RoundTrip".to_string(),
            age: 99,
            scores: vec![100, 100, 100],
        };

        // Add once
        let cid1 = dag.add(&original, None).await.unwrap();
        let retrieved1: TestData = dag.get(&cid1, None).await.unwrap();

        // Add the retrieved data (should produce same CID)
        let cid2 = dag.add(&retrieved1, None).await.unwrap();
        let retrieved2: TestData = dag.get(&cid2, None).await.unwrap();

        // All should be identical
        assert_eq!(cid1, cid2);
        assert_eq!(original, retrieved2);
    }
}
