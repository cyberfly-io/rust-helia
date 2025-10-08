# helia-interop

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Interoperability testing and compatibility utilities for Helia IPFS implementations.

## Overview

This crate provides utilities for testing and verifying interoperability between different Helia implementations and IPFS nodes. It includes test utilities, compatibility checking, and benchmarking tools.

## Features

- **Test Utilities**: Helper functions for verifying Helia implementations
- **Test Data Generation**: Common patterns for generating test data
- **Version Compatibility**: Check version compatibility between implementations
- **Benchmarking**: Simple async benchmarking utilities

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
helia-interop = "0.1.0"
```

## Usage

### Verifying Helia Instances

```rust
use helia_interop::test_utils::verify_helia_basic;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), String> {
    let helia = helia::create_helia_default().await
        .map_err(|e| e.to_string())?;
    
    // Verify basic functionality
    verify_helia_basic(Arc::new(helia)).await?;
    
    println!("Helia instance verified successfully!");
    Ok(())
}
```

### Test Data Generation

```rust
use helia_interop::test_utils::patterns;

// Generate test data
let data = patterns::generate_test_data(1024); // 1KB of test data
assert_eq!(data.len(), 1024);

// Use predefined patterns
let hello = patterns::HELLO_WORLD;
let empty = patterns::EMPTY;
let large = patterns::LARGE_TEXT;
```

### Version Compatibility Checking

```rust
use helia_interop::compat::VersionInfo;

let current_version = VersionInfo::new(1, 0, 0);
let other_version = VersionInfo::new(1, 5, 3);
let incompatible = VersionInfo::new(2, 0, 0);

// Check compatibility
assert!(current_version.is_compatible_with(&other_version));
assert!(!current_version.is_compatible_with(&incompatible));

println!("Version: {}", current_version); // "1.0.0"
```

### Benchmarking

```rust
use helia_interop::bench::bench_async;

#[tokio::main]
async fn main() {
    // Benchmark an async operation
    let result = bench_async("block_put", 100, || async {
        // Your async operation here
        tokio::time::sleep(std::time::Duration::from_micros(100)).await;
    }).await;
    
    println!("{}", result);
    // Output: "block_put: 100 iterations in 10.5ms (9523.81 ops/sec)"
    
    println!("Average: {:?}", result.avg_duration());
}
```

## Modules

### `test_utils`

Test utilities for verifying Helia implementations:

- `verify_helia_basic()`: Verify basic Helia functionality
- `patterns`: Common test data patterns
  - `generate_test_data(size)`: Generate test data
  - `HELLO_WORLD`: "Hello, World!" bytes
  - `EMPTY`: Empty bytes
  - `LARGE_TEXT`: Lorem ipsum text

### `compat`

Version compatibility utilities:

- `VersionInfo`: Version information struct
  - `new(major, minor, patch)`: Create version info
  - `is_compatible_with(&other)`: Check compatibility
  - Display formatting (e.g., "1.2.3")

### `bench`

Benchmarking utilities:

- `BenchResult`: Benchmark result struct
  - `avg_duration()`: Average duration per iteration
  - `operations_per_second()`: Throughput calculation
- `bench_async()`: Run async benchmarks

## Examples

### Full Integration Test

```rust
use helia_interop::{test_utils, compat, bench};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check version compatibility
    let current = compat::VersionInfo::new(1, 0, 0);
    let required = compat::VersionInfo::new(1, 0, 0);
    
    if !current.is_compatible_with(&required) {
        return Err("Incompatible versions".into());
    }
    
    // Create and verify Helia instance
    let helia = helia::create_helia_default().await?;
    test_utils::verify_helia_basic(Arc::new(helia.clone())).await
        .map_err(|e| e.to_string())?;
    
    // Generate test data
    let test_data = test_utils::patterns::generate_test_data(1024);
    
    // Benchmark block storage
    let helia_arc = Arc::new(helia);
    let result = bench::bench_async("block_operations", 10, || {
        let helia = helia_arc.clone();
        let data = test_data.clone();
        async move {
            // Perform operations
        }
    }).await;
    
    println!("Benchmark: {}", result);
    Ok(())
}
```

## Testing

Run the test suite:

```bash
cargo test
```

Current tests:
- `test_generate_test_data`: Verify test data generation
- `test_version_compatibility`: Verify version compatibility checking
- `test_version_display`: Verify version display formatting
- `test_bench_async`: Verify async benchmarking

## Use Cases

1. **Implementation Verification**: Ensure your Helia implementation meets basic requirements
2. **Compatibility Testing**: Verify different versions can work together
3. **Performance Testing**: Benchmark operations across implementations
4. **Integration Testing**: Test interactions between components
5. **Test Data Generation**: Generate consistent test data for reproducible tests

## Future Enhancements

- Protocol version negotiation
- Cross-implementation data exchange tests
- Network interop testing utilities
- CID format compatibility checking
- Codec compatibility verification
- More sophisticated benchmarking (percentiles, histograms)
- Test fixtures and common scenarios
- Compatibility matrices

## Contributing

Contributions are welcome! Areas for improvement:
- Additional test patterns
- More comprehensive compatibility checks
- Enhanced benchmarking capabilities
- Real-world interop scenarios

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Resources

- [IPFS Specifications](https://github.com/ipfs/specs)
- [Helia Documentation](https://helia.io)
- [IPFS Testing Guide](https://github.com/ipfs/testing)
