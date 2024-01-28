use std::sync::mpsc;

use crate::{
    block::Block,
    crypto::{self, KeyPair},
    storage::{self, Store},
    tx::SignedTransaction,
};

// TODO: Difficulty adjustment
pub static DIFFICULTY: usize = 2;
pub static GENESIS_PREV_BLOCK_HASH: &str =
    "000000000000000000000000000000000000000000000000000000000000000";

pub struct Node {
    pub mempool: Vec<SignedTransaction>,
    pub keypair: KeyPair,
    pub db_blocks: Store,
    pub db_blocks_metadata: Store,
    pub db_balances: Store,

    block_tx: mpsc::Sender<Block>,
    transaction_tx: mpsc::Sender<SignedTransaction>,
}

impl Node {
    pub fn process_block(&mut self, block: &Block) -> Result<(), String> {
        self.verify_block(block)?;
        self.process_block_transactions(block)?;

        let prev_block_number = storage::get_latest_block_number(&self.db_blocks_metadata)?;
        storage::add_block(&self.db_blocks, block)?;
        storage::set_latest_block_hash(
            &self.db_blocks_metadata,
            &block.hash,
            prev_block_number + 1,
        )?;

        Ok(())
    }

    pub fn add_transaction(&mut self, tx: &SignedTransaction) -> Result<String, String> {
        todo!()
    }

    pub fn verify_block(&self, block: &Block) -> Result<(), String> {
        if !block.hash.starts_with(&"0".repeat(DIFFICULTY)) {
            return Err(
                "Block verificatoin failed: Must contains corrent PoW according to difficulty"
                    .to_string(),
            );
        }

        let block_hash = crypto::sha256(block.serialize());
        if hex::encode(block_hash) != block.hash {
            return Err("Block verificatoin failed: Hash mismatch".to_string());
        }

        let prev_block = self.get_latest_block()?;
        let prev_block_hash =
            prev_block.map_or(GENESIS_PREV_BLOCK_HASH.to_string(), |b| b.hash.clone());

        if block.prev_block != prev_block_hash {
            return Err("Block verificatoin failed: Previous block hash mismatch".to_string());
        }

        let prev_block_number = storage::get_latest_block_number(&self.db_blocks_metadata)?;

        for (i, tx) in block.transactions.iter().enumerate() {
            if i == 0 {
                self.verify_coinbase_tx(tx, prev_block_number + 1)?;
            } else {
                self.verify_reg_tx(tx)?;
            }
        }

        // TODO: verify more

        Ok(())
    }

    pub fn process_block_transactions(&mut self, block: &Block) -> Result<(), String> {
        todo!()
    }

    pub fn get_latest_block(&self) -> Result<Option<Block>, String> {
        let block_hash = match storage::get_latest_block_hash(&self.db_blocks_metadata)? {
            Some(block_hash) => block_hash,
            None => return Ok(None),
        };

        storage::get_block(&self.db_blocks, &block_hash)
    }

    pub fn verify_coinbase_tx(
        &self,
        tx: &SignedTransaction,
        block_number: u32,
    ) -> Result<(), String> {
        todo!()
    }

    pub fn verify_reg_tx(&self, tx: &SignedTransaction) -> Result<(), String> {
        todo!()
    }
}
