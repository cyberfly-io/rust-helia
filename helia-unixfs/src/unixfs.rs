// UnixFS implementation - simplified version

use std::sync::Arc;
use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use prost::Message;
use futures::stream;

use helia_interface::{Helia, AwaitIterable};
use crate::*;
use crate::pb::{Data, data};
use crate::dag_pb::PBNode;
use crate::chunker::{Chunker, FixedSizeChunker};

const DAG_PB_CODE: u64 = 0x70;
const RAW_CODE: u64 = 0x55;

/// Main UnixFS implementation
pub struct UnixFS {
    helia: Arc<dyn Helia>,
    chunker: Box<dyn Chunker + Send + Sync>,
}

impl UnixFS {
    pub fn new(helia: Arc<dyn Helia>) -> Self {
        Self {
            helia,
            chunker: Box::new(FixedSizeChunker::default()),
        }
    }

    fn create_raw_cid(&self, data: &[u8]) -> Result<Cid, UnixFSError> {
        // Create a simple hash for RAW codec
        let mut hash_bytes = [0u8; 32];
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let hash_value = hasher.finish();
        hash_bytes[0..8].copy_from_slice(&hash_value.to_be_bytes());
        hash_bytes[8..16].copy_from_slice(&(data.len() as u64).to_be_bytes());
        
        for (i, &byte) in data.iter().take(16).enumerate() {
            hash_bytes[16 + i] = byte;
        }

        let mh: multihash::Multihash<64> = multihash::Multihash::wrap(0x12, &hash_bytes)
            .map_err(|e| UnixFSError::other(format!("Multihash error: {}", e)))?;

        Ok(Cid::new_v1(RAW_CODE, mh))
    }

    fn create_dag_pb_cid(&self, data: &[u8]) -> Result<Cid, UnixFSError> {
        // Create a simple hash for DAG-PB codec
        let mut hash_bytes = [0u8; 32];
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let hash_value = hasher.finish();
        hash_bytes[0..8].copy_from_slice(&hash_value.to_be_bytes());
        hash_bytes[8..16].copy_from_slice(&(data.len() as u64).to_be_bytes());
        
        for (i, &byte) in data.iter().take(16).enumerate() {
            hash_bytes[16 + i] = byte;
        }

        let mh: multihash::Multihash<64> = multihash::Multihash::wrap(0x12, &hash_bytes)
            .map_err(|e| UnixFSError::other(format!("Multihash error: {}", e)))?;

        Ok(Cid::new_v1(DAG_PB_CODE, mh))
    }

    async fn put_block(&self, data: Bytes, codec: u64) -> Result<Cid, UnixFSError> {
        let cid = if codec == RAW_CODE {
            self.create_raw_cid(&data)?
        } else {
            self.create_dag_pb_cid(&data)?
        };
        
        self.helia.blockstore().put(&cid, data, None).await?;
        Ok(cid)
    }

    async fn get_block(&self, cid: &Cid) -> Result<Bytes, UnixFSError> {
        self.helia.blockstore().get(cid, None).await.map_err(|e| e.into())
    }

    async fn add_small_file(&self, data: Bytes, raw_leaves: bool, mode: Option<u32>, mtime: Option<UnixFSTime>) -> Result<Cid, UnixFSError> {
        if raw_leaves {
            return self.put_block(data, RAW_CODE).await;
        }

        let unixfs_data = Data {
            r#type: data::DataType::File as i32,
            data: Some(data.to_vec()),
            filesize: data.len() as u64,
            mode: mode.unwrap_or(0),
            mtime: mtime.map(|t| pb::UnixTime {
                seconds: t.seconds as i64,
                fractional_nanoseconds: t.nanoseconds.unwrap_or(0),
            }),
            ..Default::default()
        };

        let mut unixfs_bytes = Vec::new();
        unixfs_data.encode(&mut unixfs_bytes)
            .map_err(|e| UnixFSError::other(format!("Encode error: {}", e)))?;

        let pb_node = PBNode::with_data(Bytes::from(unixfs_bytes));
        let pb_bytes = pb_node.encode()
            .map_err(|e| UnixFSError::other(format!("DAG-PB error: {}", e)))?;

        self.put_block(pb_bytes, DAG_PB_CODE).await
    }
}

#[async_trait]
impl UnixFSInterface for UnixFS {
    async fn add_bytes(&self, bytes: Bytes, options: Option<AddOptions>) -> Result<Cid, UnixFSError> {
        let raw_leaves = options.as_ref().map(|o| o.raw_leaves).unwrap_or(false);
        self.add_small_file(bytes, raw_leaves, None, None).await
    }

    async fn add_file(&self, file: FileCandidate, options: Option<AddOptions>) -> Result<Cid, UnixFSError> {
        let raw_leaves = options.as_ref().map(|o| o.raw_leaves).unwrap_or(false);
        self.add_small_file(file.content, raw_leaves, file.mode, file.mtime).await
    }

    async fn add_directory(&self, dir: Option<DirectoryCandidate>, _options: Option<AddOptions>) -> Result<Cid, UnixFSError> {
        let (mode, mtime) = dir.map(|d| (d.mode, d.mtime)).unwrap_or((None, None));
        
        let dir_unixfs = Data {
            r#type: data::DataType::Directory as i32,
            mode: mode.unwrap_or(0),
            mtime: mtime.map(|t| pb::UnixTime {
                seconds: t.seconds as i64,
                fractional_nanoseconds: t.nanoseconds.unwrap_or(0),
            }),
            ..Default::default()
        };

        let mut dir_bytes = Vec::new();
        dir_unixfs.encode(&mut dir_bytes)
            .map_err(|e| UnixFSError::other(format!("Encode error: {}", e)))?;

        let pb_node = PBNode::with_data(Bytes::from(dir_bytes));
        let pb_bytes = pb_node.encode()
            .map_err(|e| UnixFSError::other(format!("DAG-PB error: {}", e)))?;

        self.put_block(pb_bytes, DAG_PB_CODE).await
    }

    async fn cat(&self, cid: &Cid, options: Option<CatOptions>) -> Result<Bytes, UnixFSError> {
        let block = self.get_block(cid).await?;

        let data = if cid.codec() == RAW_CODE {
            block
        } else {
            let pb_node = PBNode::decode(&block)
                .map_err(|e| UnixFSError::other(format!("DAG-PB decode: {}", e)))?;

            if let Some(unixfs_bytes) = pb_node.data {
                let unixfs_data = Data::decode(&unixfs_bytes[..])
                    .map_err(|e| UnixFSError::other(format!("UnixFS decode: {}", e)))?;

                if let Some(data) = unixfs_data.data {
                    Bytes::from(data)
                } else {
                    return Err(UnixFSError::other("No data in file"));
                }
            } else {
                return Err(UnixFSError::other("No data in file"));
            }
        };

        // Apply offset and length if specified
        if let Some(opts) = options {
            let offset = opts.offset.unwrap_or(0) as usize;
            let length = opts.length.map(|l| l as usize);

            if offset >= data.len() {
                return Ok(Bytes::new());
            }

            let end = if let Some(len) = length {
                std::cmp::min(offset + len, data.len())
            } else {
                data.len()
            };

            Ok(data.slice(offset..end))
        } else {
            Ok(data)
        }
    }

    async fn cp(&self, source: &Cid, target: &Cid, name: &str, _options: Option<CpOptions>) -> Result<Cid, UnixFSError> {
        let target_block = self.get_block(target).await?;
        let mut target_pb = PBNode::decode(&target_block)
            .map_err(|e| UnixFSError::other(format!("Decode error: {}", e)))?;

        // Get the actual file size from the source
        let source_block = self.get_block(source).await?;
        let source_size = if source.codec() == RAW_CODE {
            source_block.len() as u64
        } else {
            // Decode to get the UnixFS filesize
            match PBNode::decode(&source_block) {
                Ok(source_pb) => {
                    if let Some(unixfs_bytes) = source_pb.data {
                        match Data::decode(&unixfs_bytes[..]) {
                            Ok(unixfs_data) => unixfs_data.filesize,
                            _ => source_block.len() as u64,
                        }
                    } else {
                        source_block.len() as u64
                    }
                }
                _ => source_block.len() as u64,
            }
        };

        target_pb.add_link(Some(name.to_string()), *source, source_size);

        let new_target_bytes = target_pb.encode()
            .map_err(|e| UnixFSError::other(format!("Encode error: {}", e)))?;

        self.put_block(new_target_bytes, DAG_PB_CODE).await
    }

    async fn ls(&self, cid: &Cid, _options: Option<LsOptions>) -> Result<AwaitIterable<UnixFSEntry>, UnixFSError> {
        let block = self.get_block(cid).await?;
        let pb_node = PBNode::decode(&block)
            .map_err(|e| UnixFSError::other(format!("Decode error: {}", e)))?;

        let mut entries = Vec::new();
        for link in pb_node.links {
            if let (Some(name), Some(hash), Some(size)) = (link.name, link.hash, link.tsize) {
                // Determine type by checking the linked block
                let type_ = if hash.codec() == RAW_CODE {
                    UnixFSType::Raw
                } else {
                    // Try to get the block and decode to determine type
                    match self.get_block(&hash).await {
                        Ok(link_block) => {
                            match PBNode::decode(&link_block) {
                                Ok(link_pb) => {
                                    if let Some(unixfs_bytes) = link_pb.data {
                                        match Data::decode(&unixfs_bytes[..]) {
                                            Ok(unixfs_data) => {
                                                match data::DataType::try_from(unixfs_data.r#type) {
                                                    Ok(data::DataType::Directory) => UnixFSType::Directory,
                                                    Ok(data::DataType::File) | Ok(data::DataType::Raw) => UnixFSType::File,
                                                    Ok(data::DataType::Symlink) => UnixFSType::Symlink,
                                                    _ => UnixFSType::File,
                                                }
                                            }
                                            _ => UnixFSType::File,
                                        }
                                    } else {
                                        UnixFSType::File
                                    }
                                }
                                _ => UnixFSType::File,
                            }
                        }
                        _ => UnixFSType::File,
                    }
                };

                entries.push(UnixFSEntry {
                    name,
                    cid: hash,
                    size,
                    type_,
                    mode: None,
                    mtime: None,
                });
            }
        }

        Ok(Box::pin(stream::iter(entries)))
    }

    async fn mkdir(&self, cid: &Cid, dirname: &str, _options: Option<MkdirOptions>) -> Result<Cid, UnixFSError> {
        let new_dir_cid = self.add_directory(None, None).await?;
        self.cp(&new_dir_cid, cid, dirname, None).await
    }

    async fn rm(&self, cid: &Cid, path: &str, _options: Option<RmOptions>) -> Result<Cid, UnixFSError> {
        let block = self.get_block(cid).await?;
        let mut pb_node = PBNode::decode(&block)
            .map_err(|e| UnixFSError::other(format!("Decode error: {}", e)))?;

        pb_node.links.retain(|link| link.name.as_ref().map(|n| n != path).unwrap_or(true));

        let new_bytes = pb_node.encode()
            .map_err(|e| UnixFSError::other(format!("Encode error: {}", e)))?;

        self.put_block(new_bytes, DAG_PB_CODE).await
    }

    async fn stat(&self, cid: &Cid, _options: Option<StatOptions>) -> Result<UnixFSStat, UnixFSError> {
        let block = self.get_block(cid).await?;

        if cid.codec() == RAW_CODE {
            return Ok(UnixFSStat::File(FileStat {
                cid: *cid,
                size: block.len() as u64,
                blocks: 1,
                type_: UnixFSType::Raw,
                mode: Some(0o644),
                mtime: None,
            }));
        }

        let pb_node = PBNode::decode(&block)
            .map_err(|e| UnixFSError::other(format!("Decode error: {}", e)))?;

        if let Some(unixfs_bytes) = pb_node.data {
            let unixfs_data = Data::decode(&unixfs_bytes[..])
                .map_err(|e| UnixFSError::other(format!("UnixFS decode: {}", e)))?;

            let type_ = match data::DataType::try_from(unixfs_data.r#type) {
                Ok(data::DataType::File) | Ok(data::DataType::Raw) => UnixFSType::File,
                Ok(data::DataType::Directory) => UnixFSType::Directory,
                _ => UnixFSType::Raw,
            };

            if type_ == UnixFSType::Directory {
                return Ok(UnixFSStat::Directory(DirectoryStat {
                    cid: *cid,
                    size: unixfs_data.filesize,
                    blocks: pb_node.links.len() as u64 + 1,
                    type_,
                    mode: if unixfs_data.mode != 0 { Some(unixfs_data.mode) } else { None },
                    mtime: None,
                    entries: pb_node.links.len() as u64,
                }));
            }

            Ok(UnixFSStat::File(FileStat {
                cid: *cid,
                size: unixfs_data.filesize,
                blocks: pb_node.links.len() as u64 + 1,
                type_,
                mode: if unixfs_data.mode != 0 { Some(unixfs_data.mode) } else { None },
                mtime: None,
            }))
        } else {
            Err(UnixFSError::other("No UnixFS data"))
        }
    }
}
