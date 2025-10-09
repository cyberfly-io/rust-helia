# UnixFS Examples

This directory contains practical examples demonstrating how to use the `helia-unixfs` library.

## Running the Examples

To run any example:

```bash
cargo run --package helia-unixfs --example <example_name>
```

## Available Examples

### 1. Basic File Operations (`basic_file.rs`)

Demonstrates fundamental file operations:
- Adding simple text files
- Retrieving files by CID
- Using RAW codec option
- Getting file statistics

```bash
cargo run --package helia-unixfs --example basic_file
```

### 2. Large File Handling (`large_file.rs`)

Shows how to work with large files that require chunking:
- Automatic chunking for files >1MB
- Custom chunk sizes
- Reading chunked files
- Partial reads with offset/length
- Comparing small vs chunked file stats

```bash
cargo run --package helia-unixfs --example large_file
```

### 3. Directory Operations (`directories.rs`)

Complete directory management:
- Creating directories
- Adding files to directories
- Listing directory contents  
- Creating nested directory structures
- Using `mkdir` convenience function
- Removing entries with `rm`
- Getting directory statistics

```bash
cargo run --package helia-unixfs --example directories
```

### 4. File Metadata (`metadata.rs`)

Working with file and directory metadata:
- Setting custom file permissions (mode)
- Adding modification times (mtime)
- Various permission modes (644, 755, etc.)
- Comparing metadata between files
- Directory metadata

```bash
cargo run --package helia-unixfs --example metadata
```

## Key Concepts

### File Storage

Small files (≤1MB) are stored as single blocks:
```rust
let data = Bytes::from("Small file content");
let cid = fs.add_bytes(data, None).await?;
```

Large files (>1MB) are automatically chunked:
```rust
let options = AddOptions {
    raw_leaves: true,
    chunk_size: Some(1_048_576), // 1MB chunks
    ..Default::default()
};
let cid = fs.add_bytes(large_data, Some(options)).await?;
```

### Directory Management

Create and populate directories:
```rust
// Create directory
let dir_cid = fs.add_directory(None, None).await?;

// Add file to directory
let file_cid = fs.add_bytes(data, None).await?;
let dir_cid = fs.cp(&file_cid, &dir_cid, "filename.txt", None).await?;

// List contents
let entries = fs.ls(&dir_cid, None).await?;
```

### Metadata

Add custom metadata to files:
```rust
let file = FileCandidate {
    path: "myfile.txt".to_string(),
    content: Bytes::from("content"),
    mode: Some(0o644),  // rw-r--r--
    mtime: Some(UnixFSTime {
        seconds: timestamp,
        nanoseconds: None,
    }),
};
let cid = fs.add_file(file, None).await?;
```

## Testing

All examples are designed to be self-contained and demonstrate actual working code. They:
- Initialize a Helia node
- Perform operations
- Verify results
- Print status messages

The examples use the same patterns as the test suite, ensuring they reflect real-world usage.

## Learn More

- See the [main README](../README.md) for library documentation
- Check the [tests](../src/tests.rs) for more usage patterns
- Explore the [API documentation](https://docs.rs/helia-unixfs) (when published)
