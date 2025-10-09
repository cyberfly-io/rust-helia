//! Tests for UnixFS functionality

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use bytes::Bytes;
    
    use rust_helia::create_helia_default;
    use crate::{UnixFS, UnixFSInterface, FileCandidate, DirectoryCandidate, UnixFSType, UnixFSStat, AddOptions, CatOptions};
    use futures::StreamExt;

    async fn create_test_unixfs() -> UnixFS {
        let helia = create_helia_default().await.unwrap();
        UnixFS::new(Arc::new(helia))
    }

    #[tokio::test]
    async fn test_add_and_cat_bytes() {
        let fs = create_test_unixfs().await;
        
        let data = Bytes::from("hello world");
        let cid = fs.add_bytes(data.clone(), None).await.unwrap();
        
        let retrieved_data = fs.cat(&cid, None).await.unwrap();
        assert_eq!(retrieved_data, data);
    }

    #[tokio::test]
    async fn test_add_file_with_metadata() {
        let fs = create_test_unixfs().await;
        
        let file = FileCandidate {
            path: "test.txt".to_string(),
            content: Bytes::from("hello world"),
            mode: Some(0o644),
            mtime: None,
        };
        
        let cid = fs.add_file(file, None).await.unwrap();
        
        // Verify we can read the file back
        let data = fs.cat(&cid, None).await.unwrap();
        assert_eq!(data, Bytes::from("hello world"));
        
        // Check file stats
        let stat = fs.stat(&cid, None).await.unwrap();
        match stat {
            UnixFSStat::File(file_stat) => {
                assert_eq!(file_stat.cid, cid);
                assert_eq!(file_stat.size, 11);
                assert_eq!(file_stat.type_, UnixFSType::File);
                assert_eq!(file_stat.mode, Some(0o644));
            }
            _ => panic!("Expected file stat"),
        }
    }

    #[tokio::test]
    async fn test_cat_with_options() {
        let fs = create_test_unixfs().await;
        
        let data = Bytes::from("hello world");
        let cid = fs.add_bytes(data, None).await.unwrap();
        
        // Test offset
        let options = CatOptions { offset: Some(6), length: None };
        let partial_data = fs.cat(&cid, Some(options)).await.unwrap();
        assert_eq!(partial_data, Bytes::from("world"));
        
        // Test length
        let options = CatOptions { offset: None, length: Some(5) };
        let partial_data = fs.cat(&cid, Some(options)).await.unwrap();
        assert_eq!(partial_data, Bytes::from("hello"));
        
        // Test offset and length
        let options = CatOptions { offset: Some(6), length: Some(3) };
        let partial_data = fs.cat(&cid, Some(options)).await.unwrap();
        assert_eq!(partial_data, Bytes::from("wor"));
    }

    #[tokio::test]
    async fn test_add_directory() {
        let fs = create_test_unixfs().await;
        
        let dir = DirectoryCandidate {
            path: "test_dir".to_string(),
            mode: Some(0o755),
            mtime: None,
        };
        
        let cid = fs.add_directory(Some(dir), None).await.unwrap();
        
        // Check directory stats
        let stat = fs.stat(&cid, None).await.unwrap();
        match stat {
            UnixFSStat::Directory(dir_stat) => {
                assert_eq!(dir_stat.cid, cid);
                assert_eq!(dir_stat.type_, UnixFSType::Directory);
                assert_eq!(dir_stat.mode, Some(0o755));
                assert_eq!(dir_stat.entries, 0); // Empty directory
            }
            _ => panic!("Expected directory stat"),
        }
    }

    #[tokio::test]
    async fn test_copy_file_to_directory() {
        let fs = create_test_unixfs().await;
        
        // Create a file
        let file_data = Bytes::from("hello world");
        let file_cid = fs.add_bytes(file_data.clone(), None).await.unwrap();
        
        // Create an empty directory
        let dir_cid = fs.add_directory(None, None).await.unwrap();
        
        // Copy file to directory
        let updated_dir_cid = fs.cp(&file_cid, &dir_cid, "hello.txt", None).await.unwrap();
        
        // Verify the file is in the directory
        let mut entries_stream = fs.ls(&updated_dir_cid, None).await.unwrap();
        let entries: Vec<_> = entries_stream.collect().await;
        
        assert_eq!(entries.len(), 1);
        let entry = &entries[0];
        assert_eq!(entry.name, "hello.txt");
        assert_eq!(entry.cid, file_cid);
        assert_eq!(entry.size, file_data.len() as u64);
        assert_eq!(entry.type_, UnixFSType::File);
    }

    #[tokio::test]
    async fn test_mkdir() {
        let fs = create_test_unixfs().await;
        
        // Create a parent directory
        let parent_cid = fs.add_directory(None, None).await.unwrap();
        
        // Create a subdirectory
        let updated_parent_cid = fs.mkdir(&parent_cid, "subdir", None).await.unwrap();
        
        // List contents
        let mut entries_stream = fs.ls(&updated_parent_cid, None).await.unwrap();
        let entries: Vec<_> = entries_stream.collect().await;
        
        assert_eq!(entries.len(), 1);
        let entry = &entries[0];
        assert_eq!(entry.name, "subdir");
        assert_eq!(entry.type_, UnixFSType::Directory);
    }

    #[tokio::test]
    async fn test_remove_from_directory() {
        let fs = create_test_unixfs().await;
        
        // Create a file and directory
        let file_cid = fs.add_bytes(Bytes::from("test"), None).await.unwrap();
        let dir_cid = fs.add_directory(None, None).await.unwrap();
        
        // Add file to directory
        let dir_with_file_cid = fs.cp(&file_cid, &dir_cid, "test.txt", None).await.unwrap();
        
        // Verify file is there
        let mut entries_stream = fs.ls(&dir_with_file_cid, None).await.unwrap();
        let entries: Vec<_> = entries_stream.collect().await;
        assert_eq!(entries.len(), 1);
        
        // Remove file
        let empty_dir_cid = fs.rm(&dir_with_file_cid, "test.txt", None).await.unwrap();
        
        // Verify file is gone
        let mut entries_stream = fs.ls(&empty_dir_cid, None).await.unwrap();
        let entries: Vec<_> = entries_stream.collect().await;
        assert_eq!(entries.len(), 0);
    }

    #[tokio::test]
    async fn test_pinning_with_add_options() {
        let fs = create_test_unixfs().await;
        
        let options = AddOptions {
            pin: true,
            ..Default::default()
        };
        
        let data = Bytes::from("pinned data");
        let cid = fs.add_bytes(data, Some(options)).await.unwrap();
        
        // Verify content is pinned (this would require access to helia.pins())
        // For now, just verify we can still read the data
        let retrieved = fs.cat(&cid, None).await.unwrap();
        assert_eq!(retrieved, Bytes::from("pinned data"));
    }

    #[tokio::test]
    async fn test_complex_directory_structure() {
        let fs = create_test_unixfs().await;
        
        // Create root directory
        let root_cid = fs.add_directory(None, None).await.unwrap();
        
        // Add a file to root
        let file1_cid = fs.add_bytes(Bytes::from("file1 content"), None).await.unwrap();
        let root_with_file1 = fs.cp(&file1_cid, &root_cid, "file1.txt", None).await.unwrap();
        
        // Create subdirectory
        let root_with_subdir = fs.mkdir(&root_with_file1, "subdir", None).await.unwrap();
        
        // Add file to subdirectory
        let file2_cid = fs.add_bytes(Bytes::from("file2 content"), None).await.unwrap();
        
        // First get the subdirectory CID
        let mut entries_stream = fs.ls(&root_with_subdir, None).await.unwrap();
        let entries: Vec<_> = entries_stream.collect().await;
        let subdir_entry = entries.iter().find(|e| e.name == "subdir").unwrap();
        let subdir_cid = subdir_entry.cid;
        
        // Add file to subdirectory
        let subdir_with_file = fs.cp(&file2_cid, &subdir_cid, "file2.txt", None).await.unwrap();
        
        // Verify directory structure
        let mut root_entries_stream = fs.ls(&root_with_subdir, None).await.unwrap();
        let root_entries: Vec<_> = root_entries_stream.collect().await;
        assert_eq!(root_entries.len(), 2); // file1.txt and subdir
        
        let mut subdir_entries_stream = fs.ls(&subdir_with_file, None).await.unwrap();
        let subdir_entries: Vec<_> = subdir_entries_stream.collect().await;
        assert_eq!(subdir_entries.len(), 1); // file2.txt
        
        // Verify file contents
        let file1_data = fs.cat(&file1_cid, None).await.unwrap();
        assert_eq!(file1_data, Bytes::from("file1 content"));
        
        let file2_data = fs.cat(&file2_cid, None).await.unwrap();
        assert_eq!(file2_data, Bytes::from("file2 content"));
    }
}