//! Data delivered out of consensus

use ic_logger::{warn, ReplicaLogger};
use ic_types::modular_chain::BlockExt;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

const FILE_SINK_PATH: &str = "/tmp/blocks_ext.json";

/// Used by consensus to deliver finalized blocks to external users.
pub trait ConsensusSink: Send + Sync {
    /// Deliver the finalized blocks to external users.
    fn emit(&self, blocks: Vec<BlockExt>);
}

/// File based implementation of ConsensusSink.
/// The file keeps a growing list Vec<BlockExt>.
pub struct FileSink {
    blocks_file_path: PathBuf,
    log: ReplicaLogger,
}

impl FileSink {
    /// Create the file sink
    pub fn new(blocks_file_path: &str, log: ReplicaLogger) -> Self {
        Self {
            blocks_file_path: Path::new(blocks_file_path).to_path_buf(),
            log,
        }
    }

    /// Read the current set of blocks in the file.
    fn read_blocks(&self) -> Vec<BlockExt> {
        let file = match File::open(self.blocks_file_path.as_path()) {
            Ok(file) => file,
            Err(err) => {
                warn!(
                    self.log,
                    "FileSink::get_blocks(): failed to open {:?}, err = {:?}",
                    self.blocks_file_path.as_os_str(),
                    err
                );
                return Vec::new();
            }
        };

        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap_or_else(|err| {
            warn!(
                self.log,
                "FileSink::read_blocks(): failed to parse json from {:?}, err = {:?}",
                self.blocks_file_path.as_os_str(),
                err
            );
            Vec::new()
        })
    }

    /// Update the file with the given set of blocks.
    fn write_blocks(&self, blocks: Vec<BlockExt>) {
        if let Err(err) = ic_utils::fs::write_atomically(self.blocks_file_path.as_path(), |f| {
            serde_json::to_writer_pretty(f, &blocks).map_err(|err| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Serialization error: {:?}", err),
                )
            })
        }) {
            warn!(
                self.log,
                "FileSink::write_blocks(): failed to write json to {:?}, num blocks = {}, err = {:?}",
                self.blocks_file_path.as_os_str(),
                blocks.len(),
                err
            );
        } else {
            warn!(
                self.log,
                "FileSink::write_blocks(): path = {:?}, cur blocks = {}",
                self.blocks_file_path.as_os_str(),
                blocks.len()
            );
        }
    }
}

impl ConsensusSink for FileSink {
    fn emit(&self, blocks: Vec<BlockExt>) {
        // Skip blocks with no transactions.
        let mut blocks: Vec<BlockExt> = blocks
            .into_iter()
            .filter(|block| !block.transactions.is_empty())
            .collect();
        for block in blocks.iter() {
            for (index, transaction) in block.transactions.iter().enumerate() {
                warn!(
                    self.log,
                    "FileSink::emit(): height = {}, txn = {}/{}, txn_size = {}",
                    block.height,
                    index + 1,
                    block.transactions.len(),
                    transaction.data.len()
                );
            }
        }
        if !blocks.is_empty() {
            // Read the blocks, append, write back
            // TODO: disk ops causing consensus starvation messages, move to worker thread
            let mut cur_blocks = self.read_blocks();
            cur_blocks.append(&mut blocks);
            self.write_blocks(cur_blocks);
        }
    }
}

/// Creates a new sink.
pub fn new_sink(log: ReplicaLogger) -> Box<dyn ConsensusSink> {
    Box::new(FileSink::new(FILE_SINK_PATH, log))
}
