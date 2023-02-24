//! Defines for modular consensus/execution

// TODO: move to separate crate, to be shared with execution

use serde::{Deserialize, Serialize};

/// Transactions delivered to external users
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionExt {
    pub data: Vec<u8>,
}

/// Blocks delivered to external users
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockExt {
    pub height: u64,
    pub transactions: Vec<TransactionExt>,
}
