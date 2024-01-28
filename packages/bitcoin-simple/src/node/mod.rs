use crate::{block::Block, tx::SignedTransaction};

#[derive(Debug)]
pub struct Node {}

impl Node {
    pub fn process_block(&mut self, block: &Block) -> Result<String, String> {
        todo!()
    }

    pub fn add_transaction(&mut self, tx: &SignedTransaction) -> Result<String, String> {
        todo!()
    }
}
