//! Data delivered out of consensus

use ic_types::modular_chain::{BlockExt, TransactionExt};

/// Used by consensus to deliver finalized blocks to external users.
pub trait ConsensusSink: Send + Sync {
    /// Deliver the finalized block to external users.
    fn emit(&self, blocks: Vec<BlockExt>);
}

/// File based implementation of ConsensusSink.
pub struct FileSink;

impl ConsensusSink for FileSink {
    fn emit(&self, _blocks: Vec<BlockExt>) {
        unimplemented!()
    }
}

/// Creates a new sink.
pub fn new_sink() -> Box<dyn ConsensusSink> {
    Box::new(FileSink)
}
