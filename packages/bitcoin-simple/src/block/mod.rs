use serde::{Deserialize, Serialize};

use crate::tx::SignedTransaction;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Block {
    pub hash: String,
    pub prev_block: String,
    pub nonce: u32,
    pub transactions: Vec<SignedTransaction>,
}

impl Block {
    pub fn serialize(&self) -> String {
        let txs = self
            .transactions
            .iter()
            .fold(String::new(), |a, b| a + &b.to_string());
        format!("{}{}{}", self.prev_block, txs, self.nonce)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProposedBlock {
    pub prev_block: String,
    pub transactions: Vec<SignedTransaction>,
}

impl ProposedBlock {
    pub fn serialize(&self) -> String {
        let txs = self
            .transactions
            .iter()
            .fold(String::new(), |a, b| a + &b.to_string());
        format!("{}{}", self.prev_block, txs)
    }
}
