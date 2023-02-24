//! Defines for modular consensus/execution

// TODO: move to separate crate, to be shared with execution

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    data: Vec<u8>,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    height: u64,
    transactions: Vec<Transaction>,
}
