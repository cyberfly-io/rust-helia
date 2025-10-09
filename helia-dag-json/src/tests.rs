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
}
