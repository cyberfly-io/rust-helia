//! Tests for pinning functionality

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use cid::Cid;
    use futures::StreamExt;
    use helia_interface::pins::{Pin, PinMetadataValue};
    use helia_interface::{AddOptions, IsPinnedOptions, LsOptions, Pins, RmOptions};
    use std::collections::HashMap;

    use crate::{DatastoreConfig, SimplePins, SledDatastore};

    fn create_test_datastore() -> SledDatastore {
        SledDatastore::new(DatastoreConfig {
            path: None,
            create_if_missing: true,
        })
        .unwrap()
    }

    fn create_test_pins() -> SimplePins {
        let datastore = std::sync::Arc::new(create_test_datastore());
        SimplePins::new(datastore)
    }

    fn create_test_cid() -> Cid {
        let hash_bytes = [
            0x12, 0x20, // sha2-256 code (0x12) and length (0x20 = 32 bytes)
            0x9f, 0x86, 0xd0, 0x81, 0x88, 0x4c, 0x7d, 0x65, 0x9a, 0x2f, 0xea, 0xa0, 0xc5, 0x5a,
            0xd0, 0x15, 0xa3, 0xbf, 0x4f, 0x1b, 0x2b, 0x0b, 0x82, 0x2c, 0xd1, 0x5d, 0x6c, 0x15,
            0xb0, 0xf0, 0x0a, 0x08,
        ];
        let mh = multihash::Multihash::from_bytes(&hash_bytes).unwrap();
        Cid::new_v1(0x55, mh)
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

    #[tokio::test]
    async fn test_pin_add_and_is_pinned() {
        let pins = create_test_pins();
        let cid = create_test_cid();

        // Initially not pinned
        assert!(!pins.is_pinned(&cid, None).await.unwrap());

        // Add pin
        pins.add(&cid, None).await.unwrap();

        // Now it should be pinned
        assert!(pins.is_pinned(&cid, None).await.unwrap());
    }

    #[tokio::test]
    async fn test_pin_add_with_options() {
        let pins = create_test_pins();
        let cid = create_test_cid();

        let mut metadata = HashMap::new();
        metadata.insert(
            "source".to_string(),
            PinMetadataValue::String("test".to_string()),
        );
        metadata.insert("priority".to_string(), PinMetadataValue::Number(1.0));
        metadata.insert("recursive".to_string(), PinMetadataValue::Boolean(true));

        let options = AddOptions {
            depth: Some(5),
            metadata: metadata.clone(),
            ..Default::default()
        };

        // Add pin with options
        pins.add(&cid, Some(options)).await.unwrap();

        // Verify it's pinned
        assert!(pins.is_pinned(&cid, None).await.unwrap());

        // List and verify metadata
        let ls_options = LsOptions {
            cid: Some(cid),
            ..Default::default()
        };
        let mut stream = pins.ls(Some(ls_options)).await.unwrap();

        let pin = stream.next().await.unwrap();
        assert_eq!(pin.cid, cid);
        assert_eq!(pin.depth, 5);
        assert_eq!(pin.metadata.len(), 3);
        assert_eq!(
            pin.metadata.get("source"),
            Some(&PinMetadataValue::String("test".to_string()))
        );
        assert_eq!(
            pin.metadata.get("priority"),
            Some(&PinMetadataValue::Number(1.0))
        );
        assert_eq!(
            pin.metadata.get("recursive"),
            Some(&PinMetadataValue::Boolean(true))
        );
    }

    #[tokio::test]
    async fn test_pin_remove() {
        let pins = create_test_pins();
        let cid = create_test_cid();

        // Add pin
        pins.add(&cid, None).await.unwrap();
        assert!(pins.is_pinned(&cid, None).await.unwrap());

        // Remove pin
        pins.rm(&cid, None).await.unwrap();
        assert!(!pins.is_pinned(&cid, None).await.unwrap());
    }

    #[tokio::test]
    async fn test_pin_list_all() {
        let pins = create_test_pins();
        let cid1 = create_test_cid();
        let cid2 = create_test_cid_2();

        // Add two pins
        pins.add(&cid1, None).await.unwrap();
        pins.add(&cid2, None).await.unwrap();

        // List all pins
        let mut stream = pins.ls(None).await.unwrap();
        let mut results = Vec::new();
        while let Some(pin) = stream.next().await {
            results.push(pin);
        }

        assert_eq!(results.len(), 2);
        let result_cids: Vec<Cid> = results.iter().map(|p| p.cid).collect();
        assert!(result_cids.contains(&cid1));
        assert!(result_cids.contains(&cid2));
    }

    #[tokio::test]
    async fn test_pin_list_filtered() {
        let pins = create_test_pins();
        let cid1 = create_test_cid();
        let cid2 = create_test_cid_2();

        // Add two pins
        pins.add(&cid1, None).await.unwrap();
        pins.add(&cid2, None).await.unwrap();

        // List pins filtered by cid1
        let ls_options = LsOptions {
            cid: Some(cid1),
            ..Default::default()
        };
        let mut stream = pins.ls(Some(ls_options)).await.unwrap();
        let mut results = Vec::new();
        while let Some(pin) = stream.next().await {
            results.push(pin);
        }

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].cid, cid1);
    }

    #[tokio::test]
    async fn test_pin_list_nonexistent() {
        let pins = create_test_pins();
        let cid1 = create_test_cid();
        let cid2 = create_test_cid_2();

        // Add only cid1
        pins.add(&cid1, None).await.unwrap();

        // Try to list pins for cid2 (not pinned)
        let ls_options = LsOptions {
            cid: Some(cid2),
            ..Default::default()
        };
        let mut stream = pins.ls(Some(ls_options)).await.unwrap();
        let mut results = Vec::new();
        while let Some(pin) = stream.next().await {
            results.push(pin);
        }

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_pin_default_depth() {
        let pins = create_test_pins();
        let cid = create_test_cid();

        // Add pin with default options (should be recursive/infinite depth)
        pins.add(&cid, None).await.unwrap();

        // List and verify default depth
        let ls_options = LsOptions {
            cid: Some(cid),
            ..Default::default()
        };
        let mut stream = pins.ls(Some(ls_options)).await.unwrap();

        let pin = stream.next().await.unwrap();
        assert_eq!(pin.cid, cid);
        assert_eq!(pin.depth, u64::MAX); // Default infinite depth
        assert!(pin.metadata.is_empty()); // No metadata by default
    }
}
