use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use jsonrpc_core::Result;

use crate::{crypto, node::Node, storage, tx::SignedTransaction};

use super::Rpc;

pub struct RpcInstance {
    node: Arc<Mutex<Node>>,
}

impl RpcInstance {
    pub fn new(node: Arc<Mutex<Node>>) -> Self {
        Self { node }
    }

    pub fn data_dir(&self) -> String {
        self.node.lock().unwrap().data_dir.clone()
    }
}

impl Rpc for RpcInstance {
    fn protocol_version(&self) -> Result<String> {
        Ok("1.0.0".to_string())
    }

    fn send(&self, pubkey: crypto::key::PublicKey, amount: u32) -> Result<SignedTransaction> {
        let mut node = self.node.lock().unwrap();
        Ok(node.send_tx(pubkey, amount).unwrap())
    }

    fn blockheight(&self) -> Result<u32> {
        let block_height =
            storage::get_latest_block_number(&storage::db::blocks_metadata(true, &self.data_dir()))
                .unwrap();
        Ok(block_height)
    }

    fn getpubkey(&self) -> Result<String> {
        let node = self.node.lock().unwrap();
        Ok(node.keypair.public_key.to_string())
    }

    fn newpubkey(&self) -> Result<String> {
        let random_key = crypto::KeyPair::new();
        Ok(random_key.public_key.to_string())
    }

    fn getblock(&self, block_number: u32) -> Result<Option<crate::block::Block>> {
        let block_hash = storage::get_block_hash(
            &storage::db::blocks_metadata(true, &self.data_dir()),
            block_number,
        )
        .unwrap();
        let block = block_hash.and_then(|b| {
            storage::get_block(&storage::db::blocks(true, &self.data_dir()), &b).unwrap()
        });

        Ok(block)
    }

    fn balances(&self) -> Result<HashMap<crypto::key::PublicKey, u32>> {
        let balances =
            storage::get_balances(&storage::db::balances(true, &self.data_dir())).unwrap();

        Ok(balances)
    }

    fn getbalance(&self, pubkey: crypto::key::PublicKey) -> Result<u32> {
        let balance =
            storage::get_balance(&storage::db::balances(true, &self.data_dir()), pubkey).unwrap();
        Ok(balance.unwrap_or_default())
    }

    fn mempool(&self) -> Result<Vec<SignedTransaction>> {
        let mempool = self
            .node
            .lock()
            .unwrap()
            .mempool
            .values()
            .cloned()
            .collect();
        Ok(mempool)
    }
}
