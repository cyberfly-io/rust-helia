use crate::{CarBlock, ExportOptions, Result};
use bytes::Bytes;
use cid::Cid;
use std::collections::{HashMap, HashSet, VecDeque};

/// Export strategies for CAR files
pub trait ExportStrategy {
    /// Determine which blocks to include in the export
    fn select_blocks(
        &self,
        roots: &[Cid],
        available_blocks: &HashMap<Cid, Bytes>,
        options: &ExportOptions,
    ) -> Result<Vec<CarBlock>>;
}

/// Simple export strategy that includes all available blocks
pub struct SimpleExportStrategy;

impl ExportStrategy for SimpleExportStrategy {
    fn select_blocks(
        &self,
        roots: &[Cid],
        available_blocks: &HashMap<Cid, Bytes>,
        options: &ExportOptions,
    ) -> Result<Vec<CarBlock>> {
        let mut selected_blocks = Vec::new();
        let mut visited = HashSet::new();
        let max_blocks = options.max_blocks.unwrap_or(usize::MAX);

        if options.recursive {
            // Use BFS to traverse from roots
            let mut queue = VecDeque::new();
            for root in roots {
                queue.push_back(*root);
            }

            while let Some(cid) = queue.pop_front() {
                if visited.contains(&cid) || selected_blocks.len() >= max_blocks {
                    continue;
                }

                visited.insert(cid);

                if let Some(data) = available_blocks.get(&cid) {
                    selected_blocks.push(CarBlock {
                        cid,
                        data: data.clone(),
                    });

                    // TODO: Parse block data to find linked CIDs
                    // This would require IPLD parsing based on codec
                }
            }
        } else {
            // Just include the root blocks
            for root in roots {
                if selected_blocks.len() >= max_blocks {
                    break;
                }

                if let Some(data) = available_blocks.get(root) {
                    selected_blocks.push(CarBlock {
                        cid: *root,
                        data: data.clone(),
                    });
                }
            }
        }

        Ok(selected_blocks)
    }
}

/// Filtered export strategy that only includes specific CIDs
pub struct FilteredExportStrategy {
    allowed_cids: HashSet<Cid>,
}

impl FilteredExportStrategy {
    /// Create a new filtered export strategy
    pub fn new(allowed_cids: HashSet<Cid>) -> Self {
        Self { allowed_cids }
    }
}

impl ExportStrategy for FilteredExportStrategy {
    fn select_blocks(
        &self,
        roots: &[Cid],
        available_blocks: &HashMap<Cid, Bytes>,
        options: &ExportOptions,
    ) -> Result<Vec<CarBlock>> {
        let mut selected_blocks = Vec::new();
        let max_blocks = options.max_blocks.unwrap_or(usize::MAX);

        for root in roots {
            if selected_blocks.len() >= max_blocks {
                break;
            }

            if self.allowed_cids.contains(root) {
                if let Some(data) = available_blocks.get(root) {
                    selected_blocks.push(CarBlock {
                        cid: *root,
                        data: data.clone(),
                    });
                }
            }
        }

        // If recursive, also check linked blocks (simplified)
        if options.recursive {
            for (cid, data) in available_blocks {
                if selected_blocks.len() >= max_blocks {
                    break;
                }

                if self.allowed_cids.contains(cid) && !roots.contains(cid) {
                    selected_blocks.push(CarBlock {
                        cid: *cid,
                        data: data.clone(),
                    });
                }
            }
        }

        Ok(selected_blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_export_strategy() {
        let strategy = SimpleExportStrategy;
        let roots = vec![Cid::default()];
        let mut blocks = HashMap::new();
        blocks.insert(Cid::default(), Bytes::from("test data"));
        
        let options = ExportOptions {
            max_blocks: None,
            recursive: false,
        };
        
        let result = strategy.select_blocks(&roots, &blocks, &options).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].cid, Cid::default());
    }

    #[test]
    fn test_simple_export_strategy_with_limit() {
        let strategy = SimpleExportStrategy;
        let roots = vec![Cid::default()];
        let mut blocks = HashMap::new();
        blocks.insert(Cid::default(), Bytes::from("test data"));
        
        let options = ExportOptions {
            max_blocks: Some(0),
            recursive: false,
        };
        
        let result = strategy.select_blocks(&roots, &blocks, &options).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filtered_export_strategy() {
        let allowed_cids = [Cid::default()].into_iter().collect();
        let strategy = FilteredExportStrategy::new(allowed_cids);
        
        let roots = vec![Cid::default()];
        let mut blocks = HashMap::new();
        blocks.insert(Cid::default(), Bytes::from("test data"));
        
        let options = ExportOptions {
            max_blocks: None,
            recursive: false,
        };
        
        let result = strategy.select_blocks(&roots, &blocks, &options).unwrap();
        assert_eq!(result.len(), 1);
    }
}