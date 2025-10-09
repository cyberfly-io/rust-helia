// Chunking strategies for UnixFS

use bytes::Bytes;

const DEFAULT_CHUNK_SIZE: usize = 1_048_576; // 1 MB (Filecoin default)

/// Chunker trait for splitting data into chunks
pub trait Chunker {
    fn chunk_size(&self) -> usize;
    fn chunk(&self, data: Bytes) -> Vec<Bytes>;
}

/// Fixed-size chunker - splits data into equal-sized chunks
#[derive(Debug, Clone)]
pub struct FixedSizeChunker {
    chunk_size: usize,
}

impl Default for FixedSizeChunker {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
        }
    }
}

impl FixedSizeChunker {
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }
}

impl Chunker for FixedSizeChunker {
    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn chunk(&self, data: Bytes) -> Vec<Bytes> {
        if data.len() <= self.chunk_size {
            return vec![data];
        }

        let mut chunks = Vec::new();
        let mut offset = 0;

        while offset < data.len() {
            let end = std::cmp::min(offset + self.chunk_size, data.len());
            chunks.push(data.slice(offset..end));
            offset = end;
        }

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_size_chunker_small_file() {
        let chunker = FixedSizeChunker::new(1024);
        let data = Bytes::from(vec![0u8; 512]);
        let chunks = chunker.chunk(data);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), 512);
    }

    #[test]
    fn test_fixed_size_chunker_exact_size() {
        let chunker = FixedSizeChunker::new(1024);
        let data = Bytes::from(vec![0u8; 1024]);
        let chunks = chunker.chunk(data);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), 1024);
    }

    #[test]
    fn test_fixed_size_chunker_multiple_chunks() {
        let chunker = FixedSizeChunker::new(1024);
        let data = Bytes::from(vec![0u8; 3000]);
        let chunks = chunker.chunk(data);

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].len(), 1024);
        assert_eq!(chunks[1].len(), 1024);
        assert_eq!(chunks[2].len(), 952);
    }

    #[test]
    fn test_fixed_size_chunker_default() {
        let chunker = FixedSizeChunker::default();
        assert_eq!(chunker.chunk_size(), DEFAULT_CHUNK_SIZE);
    }
}
