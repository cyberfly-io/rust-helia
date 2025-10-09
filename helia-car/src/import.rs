use crate::{CarBlock, ImportOptions, Result};
use cid::Cid;
use helia_interface::HeliaError;
use std::collections::HashSet;

/// Import strategies for CAR files
pub trait ImportStrategy {
    /// Validate and filter blocks during import
    fn validate_block(&self, block: &CarBlock, options: &ImportOptions) -> Result<bool>;

    /// Post-process imported blocks
    fn post_process(&self, imported_cids: &[Cid], options: &ImportOptions) -> Result<()>;
}

/// Simple import strategy that validates basic block integrity
pub struct SimpleImportStrategy;

impl ImportStrategy for SimpleImportStrategy {
    fn validate_block(&self, block: &CarBlock, options: &ImportOptions) -> Result<bool> {
        if options.verify_blocks {
            // Basic validation
            if block.data.is_empty() {
                return Err(HeliaError::other("Block data is empty"));
            }

            // TODO: Verify CID matches block data
            // This would require hashing the block data and comparing with CID
        }

        Ok(true)
    }

    fn post_process(&self, _imported_cids: &[Cid], _options: &ImportOptions) -> Result<()> {
        // No post-processing needed for simple strategy
        Ok(())
    }
}

/// Filtered import strategy that only allows specific CIDs
pub struct FilteredImportStrategy {
    allowed_cids: HashSet<Cid>,
}

impl FilteredImportStrategy {
    /// Create a new filtered import strategy
    pub fn new(allowed_cids: HashSet<Cid>) -> Self {
        Self { allowed_cids }
    }
}

impl ImportStrategy for FilteredImportStrategy {
    fn validate_block(&self, block: &CarBlock, options: &ImportOptions) -> Result<bool> {
        // First check if CID is allowed
        if !self.allowed_cids.contains(&block.cid) {
            return Ok(false); // Skip this block
        }

        // Then apply standard validation
        let simple_strategy = SimpleImportStrategy;
        simple_strategy.validate_block(block, options)
    }

    fn post_process(&self, imported_cids: &[Cid], _options: &ImportOptions) -> Result<()> {
        // Verify that only allowed CIDs were imported
        for cid in imported_cids {
            if !self.allowed_cids.contains(cid) {
                return Err(HeliaError::other(format!(
                    "Unexpected CID imported: {}",
                    cid
                )));
            }
        }

        Ok(())
    }
}

/// Validating import strategy with comprehensive block verification
pub struct ValidatingImportStrategy {
    max_block_size: usize,
}

impl ValidatingImportStrategy {
    /// Create a new validating import strategy
    pub fn new(max_block_size: usize) -> Self {
        Self { max_block_size }
    }
}

impl ImportStrategy for ValidatingImportStrategy {
    fn validate_block(&self, block: &CarBlock, options: &ImportOptions) -> Result<bool> {
        // Check block size
        if block.data.len() > self.max_block_size {
            return Err(HeliaError::other(format!(
                "Block size {} exceeds maximum {}",
                block.data.len(),
                self.max_block_size
            )));
        }

        // Apply standard validation
        let simple_strategy = SimpleImportStrategy;
        simple_strategy.validate_block(block, options)
    }

    fn post_process(&self, imported_cids: &[Cid], _options: &ImportOptions) -> Result<()> {
        if imported_cids.is_empty() {
            return Err(HeliaError::other("No blocks were imported"));
        }

        Ok(())
    }
}

/// Import context for tracking import progress
pub struct ImportContext {
    pub imported_count: usize,
    pub skipped_count: usize,
    pub error_count: usize,
    pub imported_cids: Vec<Cid>,
}

impl ImportContext {
    /// Create a new import context
    pub fn new() -> Self {
        Self {
            imported_count: 0,
            skipped_count: 0,
            error_count: 0,
            imported_cids: Vec::new(),
        }
    }

    /// Record a successful import
    pub fn record_import(&mut self, cid: Cid) {
        self.imported_count += 1;
        self.imported_cids.push(cid);
    }

    /// Record a skipped block
    pub fn record_skip(&mut self) {
        self.skipped_count += 1;
    }

    /// Record an error
    pub fn record_error(&mut self) {
        self.error_count += 1;
    }

    /// Get total blocks processed
    pub fn total_processed(&self) -> usize {
        self.imported_count + self.skipped_count + self.error_count
    }
}

impl Default for ImportContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_simple_import_strategy() {
        let strategy = SimpleImportStrategy;
        let block = CarBlock {
            cid: Cid::default(),
            data: Bytes::from("test data"),
        };

        let options = ImportOptions {
            max_blocks: None,
            verify_blocks: false,
        };

        assert!(strategy.validate_block(&block, &options).unwrap());
        assert!(strategy.post_process(&[Cid::default()], &options).is_ok());
    }

    #[test]
    fn test_simple_import_strategy_empty_block() {
        let strategy = SimpleImportStrategy;
        let block = CarBlock {
            cid: Cid::default(),
            data: Bytes::new(),
        };

        let options = ImportOptions {
            max_blocks: None,
            verify_blocks: true,
        };

        // Should fail with empty block when verification is enabled
        assert!(strategy.validate_block(&block, &options).is_err());
    }

    #[test]
    fn test_filtered_import_strategy() {
        let allowed_cids = [Cid::default()].into_iter().collect();
        let strategy = FilteredImportStrategy::new(allowed_cids);

        let block = CarBlock {
            cid: Cid::default(),
            data: Bytes::from("test data"),
        };

        let options = ImportOptions::default();

        assert!(strategy.validate_block(&block, &options).unwrap());
        assert!(strategy.post_process(&[Cid::default()], &options).is_ok());
    }

    #[test]
    fn test_validating_import_strategy() {
        let strategy = ValidatingImportStrategy::new(1024);

        let block = CarBlock {
            cid: Cid::default(),
            data: Bytes::from("test data"),
        };

        let options = ImportOptions::default();

        assert!(strategy.validate_block(&block, &options).unwrap());
        assert!(strategy.post_process(&[Cid::default()], &options).is_ok());
    }

    #[test]
    fn test_validating_import_strategy_oversized_block() {
        let strategy = ValidatingImportStrategy::new(5); // Very small limit

        let block = CarBlock {
            cid: Cid::default(),
            data: Bytes::from("test data that is too long"),
        };

        let options = ImportOptions::default();

        // Should fail due to block size
        assert!(strategy.validate_block(&block, &options).is_err());
    }

    #[test]
    fn test_import_context() {
        let mut context = ImportContext::new();

        context.record_import(Cid::default());
        context.record_skip();
        context.record_error();

        assert_eq!(context.imported_count, 1);
        assert_eq!(context.skipped_count, 1);
        assert_eq!(context.error_count, 1);
        assert_eq!(context.total_processed(), 3);
        assert_eq!(context.imported_cids.len(), 1);
    }
}
