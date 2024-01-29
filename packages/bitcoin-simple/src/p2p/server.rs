use std::io::{Read, Write};
use std::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Arc, Mutex},
};

use colored::Colorize;
use regex::Regex;

use crate::block::Block;
use crate::node::Node;
use crate::storage;
use crate::tx::SignedTransaction;

use super::{
    add_peer, ResultUnit, MESSAGE_GET_BLOCK, MESSAGE_GET_BLOCKS, MESSAGE_NEW_BLOCK,
    MESSAGE_NEW_PEER, MESSAGE_NEW_TRANSACTION, MESSAGE_PING,
};

#[derive(Debug, Clone, Default)]
pub struct P2pData {
    pub peers: Vec<String>,
}

#[derive(Clone)]
pub struct P2pServer {
    pub node: Arc<Mutex<Node>>,
    pub data: Arc<Mutex<P2pData>>,
    pub host_addr: String,
    pub miner_interrupt_tx: mpsc::Sender<()>,
}

impl P2pServer {
    pub fn new(
        node: Arc<Mutex<Node>>,
        data: Arc<Mutex<P2pData>>,
        host_addr: impl Into<String>,
        miner_interrupt_tx: mpsc::Sender<()>,
    ) -> Self {
        Self {
            node,
            data,
            host_addr: host_addr.into(),
            miner_interrupt_tx,
        }
    }

    pub fn serve(&mut self, data_dir: &str) -> ResultUnit {
        let listener = TcpListener::bind(self.host_addr.as_str())?;

        println!(
            "{:?} Listening on {:?}",
            "P2p".green(),
            listener.local_addr()?.to_string()
        );

        for stream in listener.incoming() {
            self.handle_connection(stream?, data_dir)?;
        }

        Ok(())
    }

    pub fn handle_connection(&mut self, mut stream: TcpStream, data_dir: &str) -> ResultUnit {
        let mut buffer = [0; 100000];

        stream.read_exact(&mut buffer)?;

        let msg = String::from_utf8_lossy(&buffer[..]);
        let msg = msg.as_ref();

        let response = self.response(msg, data_dir)?;

        let final_resp = format!("{}\r\n", response);

        stream.write_all(final_resp.as_bytes())?;
        stream.flush()?;

        Ok(())
    }

    fn response(&mut self, msg: &str, data_dir: &str) -> Result<String, String> {
        if msg.starts_with(MESSAGE_PING) {
            Ok(String::from("OK"))
        } else if msg.starts_with(MESSAGE_GET_BLOCKS) {
            self.handle_get_blocks(data_dir)
        } else if msg.starts_with(MESSAGE_GET_BLOCK) {
            let regex = Regex::new(&format!(r"{}\((?P<hash>.*?)\)", MESSAGE_GET_BLOCK))
                .map_err(|e| e.to_string())?;
            let caps = regex.captures(msg).unwrap();
            let block = &caps["hash"];
            self.handle_get_block(block, data_dir)
        } else if msg.starts_with(MESSAGE_NEW_BLOCK) {
            let regex = Regex::new(&format!(r"{}\((?P<block>.*?)\)", MESSAGE_GET_BLOCK))
                .map_err(|e| e.to_string())?;
            let caps = regex.captures(msg).unwrap();
            let block = &caps["block"];
            self.handle_get_block(block, data_dir)
        } else if msg.starts_with(MESSAGE_NEW_PEER) {
            let re = Regex::new(&format!(r"{}\((?P<host>.*?)\)", MESSAGE_NEW_PEER))
                .map_err(|e| e.to_string())?;
            let caps = re.captures(msg).unwrap();
            let host = &caps["host"];
            self.handle_new_peer(host, data_dir)
                .map_err(|e| e.to_string())
        } else if msg.starts_with(MESSAGE_NEW_TRANSACTION) {
            let re = Regex::new(&format!(r"{}\((?P<tx>.*?)\)", MESSAGE_NEW_TRANSACTION))
                .map_err(|e| e.to_string())?;
            let caps = re.captures(msg).unwrap();
            let tx = &caps["tx"];
            self.handle_new_transaction(tx, data_dir)
        } else {
            Err(String::from("Invalid MESSAGE"))
        }
    }

    pub fn handle_get_blocks(&mut self, data_dir: &str) -> Result<String, String> {
        let block_hashes =
            storage::get_block_hashes(&storage::db::blocks_metadata(true, data_dir))?;
        serde_json::to_string(&block_hashes).map_err(|e| e.to_string())
    }

    pub fn handle_get_block(&mut self, block_hash: &str, data_dir: &str) -> Result<String, String> {
        let block = storage::get_block(&storage::db::blocks(true, data_dir), block_hash)?;
        serde_json::to_string_pretty(&block).map_err(|e| e.to_string())
    }

    // TODO: how to handle fork
    pub fn handle_new_block(&mut self, block: String, data_dir: &str) -> Result<String, String> {
        let block: Block = serde_json::from_str(&block).map_err(|e| e.to_string())?;
        let mut node = self.node.lock().unwrap();
        let existing_block = storage::get_block(&storage::db::blocks(true, data_dir), &block.hash)?;

        if existing_block.is_none() {
            println!(
                "{} {} - Txs: {}",
                "New Block".green(),
                block.hash,
                block.transactions.len()
            );
            node.process_block(&block)?;
            self.miner_interrupt_tx.send(()).unwrap()
        }

        Ok("Ok".to_string())
    }

    pub fn handle_new_transaction(&mut self, tx: &str, data_dir: &str) -> Result<String, String> {
        let tx: SignedTransaction = serde_json::from_str(tx).unwrap();
        let mut node = self.node.lock().unwrap();
        // TODO: check tx is duplicate or not
        node.add_tx_to_mempool(&tx)?;

        Ok("Ok".to_string())
    }

    pub fn handle_new_peer(
        &mut self,
        peer: &str,
        data_dir: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        add_peer(
            self.node.clone(),
            self.data.clone(),
            self.miner_interrupt_tx.clone(),
            peer,
            data_dir,
        )?;

        let p2p_data = self.data.lock().unwrap();
        let resp_peers: Vec<_> = p2p_data
            .peers
            .clone()
            .into_iter()
            .filter(|x| *x != peer)
            .collect();

        Ok(serde_json::to_string(&resp_peers)?)
    }
}
