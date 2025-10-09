//! Interoperability testing and compatibility utilities for Helia
//!
//! This crate provides utilities for testing interoperability between
//! different Helia implementations and IPFS nodes.

use helia_interface::Helia;
use std::sync::Arc;

/// Test utilities for verifying Helia implementations
pub mod test_utils {
    use super::*;

    /// Verify that a Helia instance implements the basic required functionality
    pub async fn verify_helia_basic(helia: Arc<dyn Helia>) -> Result<(), String> {
        // Verify blockstore is accessible
        let _blockstore = helia.blockstore();

        // Verify datastore is accessible
        let _datastore = helia.datastore();

        // Verify pins interface is accessible
        let _pins = helia.pins();

        println!("✓ All basic interfaces accessible");
        Ok(())
    }

    /// Test data patterns for interop testing
    pub mod patterns {
        use bytes::Bytes;

        /// Generate test data of specified size
        pub fn generate_test_data(size: usize) -> Bytes {
            let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
            Bytes::from(data)
        }

        /// Common test strings
        pub const HELLO_WORLD: &[u8] = b"Hello, World!";
        pub const EMPTY: &[u8] = b"";
        pub const LARGE_TEXT: &[u8] = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
            Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
    }
}

/// Compatibility testing utilities
pub mod compat {
    /// Version information for compatibility checking
    #[derive(Debug, Clone)]
    pub struct VersionInfo {
        pub major: u32,
        pub minor: u32,
        pub patch: u32,
    }

    impl VersionInfo {
        pub fn new(major: u32, minor: u32, patch: u32) -> Self {
            Self {
                major,
                minor,
                patch,
            }
        }

        /// Check if this version is compatible with another version
        pub fn is_compatible_with(&self, other: &VersionInfo) -> bool {
            // Major version must match for compatibility
            self.major == other.major
        }
    }

    impl std::fmt::Display for VersionInfo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

/// Benchmarking utilities for performance testing
pub mod bench {
    use std::time::{Duration, Instant};

    /// Simple benchmark result
    #[derive(Debug, Clone)]
    pub struct BenchResult {
        pub operation: String,
        pub duration: Duration,
        pub iterations: usize,
    }

    impl BenchResult {
        pub fn avg_duration(&self) -> Duration {
            self.duration / self.iterations as u32
        }

        pub fn operations_per_second(&self) -> f64 {
            self.iterations as f64 / self.duration.as_secs_f64()
        }
    }

    impl std::fmt::Display for BenchResult {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}: {} iterations in {:?} ({:.2} ops/sec)",
                self.operation,
                self.iterations,
                self.duration,
                self.operations_per_second()
            )
        }
    }

    /// Simple benchmark runner
    pub async fn bench_async<F, Fut>(name: &str, iterations: usize, mut f: F) -> BenchResult
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let start = Instant::now();
        for _ in 0..iterations {
            f().await;
        }
        let duration = start.elapsed();

        BenchResult {
            operation: name.to_string(),
            duration,
            iterations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compat::VersionInfo;
    use crate::test_utils::patterns;

    #[test]
    fn test_generate_test_data() {
        let data = patterns::generate_test_data(10);
        assert_eq!(data.len(), 10);
        // First bytes should be 0..9
        assert_eq!(data[0], 0);
        assert_eq!(data[9], 9);
    }

    #[test]
    fn test_version_compatibility() {
        let v1 = VersionInfo::new(1, 0, 0);
        let v2 = VersionInfo::new(1, 5, 3);
        let v3 = VersionInfo::new(2, 0, 0);

        assert!(v1.is_compatible_with(&v2));
        assert!(!v1.is_compatible_with(&v3));
    }

    #[test]
    fn test_version_display() {
        let v = VersionInfo::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[tokio::test]
    async fn test_bench_async() {
        let result = bench::bench_async("test_op", 10, || async {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        })
        .await;

        assert_eq!(result.iterations, 10);
        assert!(result.duration.as_millis() >= 10);
    }
}
