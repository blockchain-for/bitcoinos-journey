use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Block {
    pub hash: String,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignedTransaction {}
